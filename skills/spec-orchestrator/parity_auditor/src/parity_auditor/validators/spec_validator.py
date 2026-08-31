import os
import re
from typing import List, Dict, Any, Set, Tuple
import yaml

from .base import IValidator
from ..core.findings import Finding
from ..core.workspace import WorkspaceRepository


def _extract_frontmatter(content: str) -> Dict[str, Any]:
    m = re.match(r"^---\s*\n(.*?)\n---\s*\n", content, re.DOTALL)
    if not m:
        return {}
    try:
        data = yaml.safe_load(m.group(1).replace('\x01', ''))
        return data if isinstance(data, dict) else {}
    except Exception:
        return {}


class SpecValidator(IValidator):
    def validate_schema_container_paths(self, repo: WorkspaceRepository, **kwargs) -> List[Finding]:
        rules = repo.get_codebase_rules()
        backlog_dirs = rules.backlog_directories

        features_dir = kwargs.get("features_dir")
        if not features_dir:
            features_dir_rel = getattr(backlog_dirs, "features", None) if backlog_dirs else None
            if not features_dir_rel:
                return []
            features_dir = os.path.join(repo.workspace_dir, features_dir_rel)

        if not os.path.exists(features_dir):
            return []

        errors = []
        for filename in sorted(os.listdir(features_dir)):
            if not filename.endswith(".md"):
                continue
            filepath = os.path.join(features_dir, filename)
            try:
                with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
                    content = f.read()
            except Exception:
                continue

            fm = _extract_frontmatter(content)
            containers = fm.get("schema_containers")
            if not containers or not isinstance(containers, list):
                continue

            for container in containers:
                path = None
                if isinstance(container, dict):
                    path = container.get("path")
                elif isinstance(container, str):
                    path = container

                if path and ":" not in path:
                    errors.append(Finding(
                        "schema-container-path-must-be-fully-qualified",
                        f"Feature '{filename}': schema container path '{path}' is unqualified (missing module prefix colon ':'). "
                        f"Expected format e.g. 'ietf-geo-location:geo-location/reference-frame'.",
                        location=os.path.basename(os.path.normpath(features_dir)),
                    ))

        return errors

    def load_disallowed_technologies(self, repo: WorkspaceRepository) -> Dict[str, str]:
        """
        Dynamically loads disallowed technologies from:
        1. .pipeline/profiles/*.md (frontmatter or sections)
        2. deap_harness_config.yaml / deap_harness_config.json
        3. codebase_rules.json (spec_rules.forbidden_standards_blocklist or disallowed_technologies)
        """
        disallowed: Dict[str, str] = {}
        workspace_dir = repo.workspace_dir

        # 1. Scan profile files in .pipeline/profiles/
        profiles_dir = os.path.join(workspace_dir, ".pipeline", "profiles")
        if os.path.isdir(profiles_dir):
            for filename in sorted(os.listdir(profiles_dir)):
                if filename.endswith(".md"):
                    filepath = os.path.join(profiles_dir, filename)
                    try:
                        with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
                            content = f.read()
                        fm = _extract_frontmatter(content)
                        dt = fm.get("disallowed_technologies")
                        if isinstance(dt, list):
                            for item in dt:
                                if isinstance(item, str) and item.strip():
                                    disallowed[item.strip()] = filename
                        elif isinstance(dt, str) and dt.strip():
                            disallowed[dt.strip()] = filename

                        # Also scan section headers like ## Disallowed Technologies
                        disallowed_sec = re.search(r'##\s+(?:Disallowed|Forbidden)\s+Technologies\s*(.*?)(?=##|\Z)', content, re.DOTALL | re.IGNORECASE)
                        if disallowed_sec:
                            for line in disallowed_sec.group(1).splitlines():
                                line_clean = line.strip().lstrip("-* ").strip()
                                for token in re.split(r'[,;]\s*', line_clean):
                                    t = token.strip()
                                    if t and len(t) > 1:
                                        disallowed[t] = filename
                    except Exception:
                        pass

        # 2. Check deap_harness_config.yaml or deap_harness_config.json
        for cfg_name in ("deap_harness_config.yaml", "deap_harness_config.yml", "deap_harness_config.json"):
            cfg_path = os.path.join(workspace_dir, cfg_name)
            if os.path.isfile(cfg_path):
                try:
                    with open(cfg_path, "r", encoding="utf-8", errors="ignore") as f:
                        if cfg_name.endswith(".json"):
                            import json
                            data = json.load(f)
                        else:
                            data = yaml.safe_load(f)
                    if isinstance(data, dict):
                        dt = data.get("disallowed_technologies")
                        if isinstance(dt, list):
                            for item in dt:
                                if isinstance(item, str) and item.strip():
                                    disallowed[item.strip()] = cfg_name
                except Exception:
                    pass

        # 3. Check codebase rules
        rules = repo.get_codebase_rules()
        if rules and rules.spec_rules:
            for item in (rules.spec_rules.forbidden_standards_blocklist or []):
                if item.strip():
                    disallowed[item.strip()] = "codebase_rules.json"

        return disallowed

    def validate_disallowed_technologies(self, repo: WorkspaceRepository, **kwargs) -> List[Finding]:
        """Validates that specifications do not reference disallowed technologies forbidden by active profiles."""
        disallowed_map = self.load_disallowed_technologies(repo)
        if not disallowed_map:
            return []

        workspace_dir = repo.workspace_dir
        rules = repo.get_codebase_rules()
        backlog = rules.backlog_directories if rules else None

        scan_dirs = ["docs/conops"]
        if backlog:
            for attr in ("epics", "features", "user_stories", "use_cases"):
                rel = getattr(backlog, attr, None)
                if rel and rel not in scan_dirs:
                    scan_dirs.append(rel)
        else:
            scan_dirs.extend(["docs/epics", "docs/features", "docs/user-stories", "docs/use-cases", "docs/safety"])

        errors: List[Finding] = []

        for sdir in scan_dirs:
            full_dir = os.path.join(workspace_dir, sdir)
            if not os.path.isdir(full_dir):
                continue
            for root, _, files in os.walk(full_dir):
                for f in sorted(files):
                    if f.endswith(".md") and f != "README.md":
                        filepath = os.path.join(root, f)
                        rel_path = os.path.relpath(filepath, workspace_dir)
                        try:
                            with open(filepath, "r", encoding="utf-8", errors="ignore") as md_f:
                                lines = md_f.read().splitlines()
                        except Exception:
                            continue

                        for lineno_1idx, line in enumerate(lines, start=1):
                            # Skip comments or frontmatter
                            if line.strip().startswith("<!--") or line.strip().startswith("---"):
                                continue
                            for tech, source in disallowed_map.items():
                                if re.search(r'\b' + re.escape(tech) + r'\b', line, re.IGNORECASE):
                                    errors.append(Finding(
                                        "profile-disallowed-technology",
                                        f"{rel_path}:{lineno_1idx}: Specification references disallowed technology '{tech}' forbidden by active profile/configuration ({source}).",
                                        location=rel_path,
                                        detail={"technology": tech, "source": source, "line": lineno_1idx}
                                    ))

        return errors

    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[Finding]:
        return self.validate_schema_container_paths(repo, **kwargs) + self.validate_disallowed_technologies(repo, **kwargs)
