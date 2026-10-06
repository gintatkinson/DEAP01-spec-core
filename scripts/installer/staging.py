#!/usr/bin/env python3
"""
Atomic staging in TemporaryDirectory, framework asset copying, and atomic workspace promotion.
/// Realises: [InstallerStagingManager]
"""
import os
import shutil
import stat
import tempfile
from pathlib import Path
from typing import List, Optional

from scripts.installer.metadata import (
    InstallOptions,
    PreservedMetadata,
    RepositoryRole,
    restore_preserved_metadata,
)
from scripts.installer.rollback import (
    PromotionError,
    RollbackManager,
    StagingError,
)


class StagingManager:
    """
    Encapsulates isolated filesystem staging and atomic promotion.
    """

    def __init__(self, options: InstallOptions, preserved: PreservedMetadata):
        self.options = options
        self.preserved = preserved
        self.temp_dir: Optional[tempfile.TemporaryDirectory] = None
        self.staging_path: Optional[Path] = None

    def __enter__(self) -> "StagingManager":
        self.temp_dir = tempfile.TemporaryDirectory(prefix="deap_stage_")
        self.staging_path = Path(self.temp_dir.name).resolve()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb) -> None:
        if self.temp_dir:
            try:
                self.temp_dir.cleanup()
            except Exception:
                pass

    def assemble_assets(self) -> None:
        """
        Copies core framework assets into the staging tree:
        - skills/ -> staging/skills/
        - rules/ -> staging/rules/
        - .pipeline/ -> staging/.pipeline/ (pruning .pipeline/upstream and diagnostics)
        - .agents/ -> staging/.agents/
        - scripts/ -> staging/scripts/ (ensuring 0o755 executable permissions)
        - requirements.txt, pyproject.toml -> staging/ (if present)
        - tests/fixtures/, tests/test_sysml_compiler_parity.py -> staging/tests/
        - docs/conops/README.md, docs/safety/README.md, docs/OPERATOR_PROMPT_CATALOG.md,
          docs/JIRA_INTEGRATION_GUIDE.md -> staging/docs/
        """
        if not self.staging_path:
            raise StagingError("Staging directory is not initialized.")

        installer_root = self.options.installer_root

        try:
            # 1. Copy framework directories
            for dname in ("skills", "rules", ".pipeline", ".agents", "scripts"):
                src_d = installer_root / dname
                if src_d.is_dir():
                    dest_d = self.staging_path / dname
                    shutil.copytree(src_d, dest_d, symlinks=True)

            # 2. Prune upstream-only directories from staging
            for prune_name in ("upstream", "diagnostics"):
                prune_p = self.staging_path / ".pipeline" / prune_name
                if prune_p.exists():
                    if prune_p.is_dir():
                        shutil.rmtree(prune_p)
                    else:
                        prune_p.unlink()

            # 3. Create required pipeline directories
            for sub in ("contracts", "domain_specs", "profiles"):
                (self.staging_path / ".pipeline" / sub).mkdir(parents=True, exist_ok=True)

            # 4. Copy root project configuration files if present
            for fname in ("requirements.txt", "pyproject.toml"):
                src_f = installer_root / fname
                if src_f.is_file():
                    shutil.copy2(src_f, self.staging_path / fname)

            # 5. Copy tests/fixtures and parity test suite
            fixtures_src = installer_root / "tests" / "fixtures"
            if fixtures_src.is_dir():
                fixtures_dest = self.staging_path / "tests" / "fixtures"
                fixtures_dest.parent.mkdir(parents=True, exist_ok=True)
                shutil.copytree(fixtures_src, fixtures_dest, symlinks=True)

            parity_test_src = installer_root / "tests" / "test_sysml_compiler_parity.py"
            if parity_test_src.is_file():
                parity_test_dest = self.staging_path / "tests" / "test_sysml_compiler_parity.py"
                parity_test_dest.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(parity_test_src, parity_test_dest)

            # 6. Copy governance docs if present
            docs_map = [
                ("docs/conops/README.md", "docs/conops/README.md"),
                ("docs/safety/README.md", "docs/safety/README.md"),
                ("docs/OPERATOR_PROMPT_CATALOG.md", "docs/OPERATOR_PROMPT_CATALOG.md"),
                ("docs/JIRA_INTEGRATION_GUIDE.md", "docs/JIRA_INTEGRATION_GUIDE.md"),
            ]
            for src_rel, dest_rel in docs_map:
                src_file = installer_root / src_rel
                if src_file.is_file():
                    dest_file = self.staging_path / dest_rel
                    dest_file.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(src_file, dest_file)

            # 7. Schema directory setup
            schema_dest = self.staging_path / "schema"
            schema_dest.mkdir(parents=True, exist_ok=True)
            if self.options.target_role not in (
                RepositoryRole.DOWNSTREAM_CUSTOMER_PROJECT,
                RepositoryRole.DOMAIN_DISTRIBUTION_TEMPLATE,
            ):
                schema_src = installer_root / "schema"
                if schema_src.is_dir():
                    for item in schema_src.iterdir():
                        if item.is_file():
                            shutil.copy2(item, schema_dest / item.name)
                        elif item.is_dir():
                            shutil.copytree(item, schema_dest / item.name, symlinks=True)

            # 8. Restore preserved metadata
            restore_preserved_metadata(self.staging_path, self.preserved)

            # 9. Merge .gitignore
            self.merge_gitignore()

        except Exception as e:
            raise StagingError(f"Failed to assemble assets into staging directory: {e}") from e

    def enforce_clean_landing_zones(self) -> None:
        """
        If target_role is DOMAIN_DISTRIBUTION_TEMPLATE:
        - Creates doc unit directories: docs/conops/units/conops, docs/conops/units/mission_intent,
          docs/interfaces, docs/safety, docs/architecture/blueprints, docs/epics, docs/features,
          docs/user-stories, docs/use-cases, docs/management.
        - For landing zone directories (schema, docs/epics, docs/features, docs/user-stories,
          docs/use-cases, docs/management, docs/conops, docs/safety):
          places .gitkeep if no concrete specifications exist.
        - Removes template READMEs from schema/, docs/conops/, docs/safety/.
        """
        if not self.staging_path:
            return

        if self.options.target_role == RepositoryRole.DOMAIN_DISTRIBUTION_TEMPLATE:
            doc_dirs = [
                "docs/conops",
                "docs/conops/units/conops",
                "docs/conops/units/mission_intent",
                "docs/interfaces",
                "docs/safety",
                "docs/architecture/blueprints",
                "docs/epics",
                "docs/features",
                "docs/user-stories",
                "docs/use-cases",
                "docs/management",
                "schema",
            ]
            for d in doc_dirs:
                (self.staging_path / d).mkdir(parents=True, exist_ok=True)

            keep_dirs = [
                "docs/management",
                "docs/epics",
                "docs/features",
                "docs/user-stories",
                "docs/use-cases",
                "schema",
                "docs/conops",
                "docs/safety",
            ]
            for rel_d in keep_dirs:
                target_d = self.staging_path / rel_d
                has_concrete_specs = False
                if target_d.is_dir():
                    for f in target_d.iterdir():
                        if f.is_file() and f.name not in (".gitkeep", "README.md", ".DS_Store"):
                            has_concrete_specs = True
                            break
                if not has_concrete_specs:
                    (target_d / ".gitkeep").touch()

            # Remove template READMEs in clean landing zones
            for rm_rel in ("schema/README.md", "docs/conops/README.md", "docs/safety/README.md"):
                rm_file = self.staging_path / rm_rel
                if rm_file.is_file():
                    rm_file.unlink()

        elif self.options.target_role == RepositoryRole.DOWNSTREAM_CUSTOMER_PROJECT:
            schema_dir = self.staging_path / "schema"
            schema_dir.mkdir(parents=True, exist_ok=True)
            files = [f for f in schema_dir.iterdir() if f.name != ".DS_Store"]
            if not files:
                (schema_dir / ".gitkeep").touch()

    def merge_gitignore(self) -> None:
        """
        Deduplicates and merges installer root .gitignore with preserved target .gitignore,
        preserving custom downstream entries.
        """
        if not self.staging_path:
            return

        installer_gitignore = self.options.installer_root / ".gitignore"
        lines: List[str] = []

        if installer_gitignore.is_file():
            try:
                lines.extend(installer_gitignore.read_text(encoding="utf-8").splitlines())
            except Exception:
                pass

        lines.extend(self.preserved.gitignore_lines)

        # Deduplicate while sorting unique non-empty lines
        unique_lines = sorted(list(set(line.strip() for line in lines if line.strip())))
        content = "\n".join(unique_lines) + "\n" if unique_lines else ""
        (self.staging_path / ".gitignore").write_text(content, encoding="utf-8")

    def promote_to_target(self, rollback_mgr: RollbackManager) -> None:
        """
        Promotes verified staging tree to target directory non-destructively:
        1. Registers files for snapshot tracking in rollback_mgr.
        2. Prunes obsolete files in managed framework directories.
        3. Copies/synchronizes staging assets to target directory with correct permissions.
        4. Sets executable bits on target scripts/*.sh and scripts/*.py.
        """
        if not self.staging_path:
            raise PromotionError("Staging path is invalid.")

        target_dir = self.options.target_dir.resolve()
        target_dir.mkdir(parents=True, exist_ok=True)

        try:
            # 1. Prune obsolete files in DEAP-managed framework directories if target differs from root
            if target_dir != self.options.installer_root.resolve():
                for fw_name in ("skills", "rules", "scripts", ".agents", ".pipeline"):
                    target_fw = target_dir / fw_name
                    staging_fw = self.staging_path / fw_name
                    if target_fw.is_dir():
                        for p in list(target_fw.rglob("*")):
                            if p.is_file():
                                rel = p.relative_to(target_fw)
                                if not (staging_fw / rel).exists():
                                    rollback_mgr.record_pre_modification(p)
                                    p.unlink()

            # 2. Synchronize all staged items to target
            for src_p in self.staging_path.rglob("*"):
                rel_p = src_p.relative_to(self.staging_path)
                dest_p = target_dir / rel_p

                if src_p.is_dir():
                    if not dest_p.exists():
                        dest_p.mkdir(parents=True, exist_ok=True)
                        rollback_mgr.record_creation(dest_p)
                elif src_p.is_file():
                    dest_p.parent.mkdir(parents=True, exist_ok=True)
                    if dest_p.exists():
                        rollback_mgr.record_pre_modification(dest_p)
                    else:
                        rollback_mgr.record_creation(dest_p)

                    shutil.copy2(src_p, dest_p)

                    # Normalize permissions
                    if dest_p.suffix in (".sh", ".py") and "scripts" in dest_p.parts:
                        dest_p.chmod(dest_p.stat().st_mode | 0o755)
                    else:
                        dest_p.chmod(dest_p.stat().st_mode | 0o644)

            # Ensure all scripts have executable bit
            scripts_dir = target_dir / "scripts"
            if scripts_dir.is_dir():
                for sf in scripts_dir.iterdir():
                    if sf.is_file() and sf.suffix in (".sh", ".py"):
                        try:
                            sf.chmod(sf.stat().st_mode | 0o755)
                        except OSError:
                            pass

        except Exception as e:
            raise PromotionError(f"Failed to promote staged assets to target {target_dir}: {e}") from e
