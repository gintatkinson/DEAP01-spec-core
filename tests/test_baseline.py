"""
Downstream Environment & Runtime Integrity Verification Suite.
/// Realises: [BaselineVerification]
"""
import sys
import os
import re
import json
import subprocess
import tempfile
import shutil
import unittest
import pytest

repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if repo_root not in sys.path:
    sys.path.insert(0, repo_root)

from scripts.verify_downstream_baseline import (
    verify_upstream_blueprint_domain_cleanliness,
    run_all_checks,
    MarkdownTableASTParser,
    CartesianProductValidator,
    ProofBlockAST,
    _validate_domain_types,
)

def test_python_runtime_environment():
    """Verify Python runtime version and core interpreter executable exist and function."""
    assert sys.version_info >= (3, 8), f"Python version {sys.version} is below required 3.8+"
    assert os.path.exists(sys.executable), "Python interpreter path invalid"

def test_disk_io_and_permissions():
    """Verify local file system read, write, and permission capabilities."""
    with tempfile.NamedTemporaryFile(mode="w+", delete=True) as temp_file:
        test_payload = "DEAP_ENVIRONMENT_INTEGRITY_CHECK_PAYLOAD_2026"
        temp_file.write(test_payload)
        temp_file.seek(0)
        read_back = temp_file.read()
        assert read_back == test_payload, "Disk I/O payload mismatch during environment validation"

def test_schema_directory_accessible():
    """Verify schema directory exists and is accessible for domain specification contracts."""
    schema_dir = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "schema")
    assert os.path.isdir(schema_dir) or os.path.isdir("schema"), "Schema directory missing or inaccessible"

def test_latex_katex_integrity():
    """Verify KaTeX / LaTeX mathematical rendering syntax across all markdown files.

    Ensures:
    - Balanced $$ math blocks
    - No bare alignment operators & outside alignment environments (aligned, matrix, bmatrix, etc.)
    - No forbidden \\begin{align} or \\begin{align*} in math blocks (\\begin{aligned} must be used)
    - Balanced \\begin{aligned} and \\end{aligned} pairs
    """
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    if not os.path.isdir(repo_root):
        repo_root = os.getcwd()

    excluded_dirs = {".git", "node_modules", ".dart_tool", "build"}
    allowed_alignment_envs = {
        "aligned", "alignedat", "matrix", "pmatrix", "bmatrix", "Bmatrix",
        "vmatrix", "Vmatrix", "cases", "dcases", "rcases", "array",
        "split", "gathered", "gather", "subarray", "smallmatrix"
    }

    errors = []
    for root, dirs, files in os.walk(repo_root):
        dirs[:] = [d for d in dirs if d not in excluded_dirs]
        for f in files:
            if not f.endswith(".md"):
                continue
            file_path = os.path.join(root, f)
            rel_path = os.path.relpath(file_path, repo_root)
            try:
                with open(file_path, "r", encoding="utf-8") as md_file:
                    content = md_file.read()
            except Exception as e:
                errors.append(f"Failed to read {rel_path}: {e}")
                continue

            cleaned = re.sub(r"```.*?```|~~~.*?~~~", "", content, flags=re.DOTALL)
            cleaned = re.sub(r"`+.*?`+", "", cleaned)

            # a. Validate balanced $$ math blocks
            parts = cleaned.split("$$")
            if (len(parts) - 1) % 2 != 0:
                errors.append(f"Unbalanced $$ display math delimiters in {rel_path} (found {len(parts) - 1} delimiters).")
                continue

            # Check balanced \begin{aligned} and \end{aligned} globally in file
            num_begin_aligned_all = len(re.findall(r"\\begin\{aligned\}", cleaned))
            num_end_aligned_all = len(re.findall(r"\\end\{aligned\}", cleaned))
            if num_begin_aligned_all != num_end_aligned_all:
                errors.append(f"Unbalanced \\begin{{aligned}} ({num_begin_aligned_all}) and \\end{{aligned}} ({num_end_aligned_all}) pairs in {rel_path}.")

            # Validate each display math block
            for i in range(1, len(parts), 2):
                block = parts[i]

                # c. Detect top-level \begin{align} or \begin{align*}
                if re.search(r"\\begin\{align\*?\}", block):
                    errors.append(
                        f"Forbidden \\begin{{align}} or \\begin{{align*}} found in display math block in {rel_path}. "
                        f"In markdown KaTeX, \\begin{{aligned}} must be used instead."
                    )

                # d. Validate balanced \begin{aligned} and \end{aligned} pairs within the block
                num_begin_aligned = len(re.findall(r"\\begin\{aligned\}", block))
                num_end_aligned = len(re.findall(r"\\end\{aligned\}", block))
                if num_begin_aligned != num_end_aligned:
                    errors.append(
                        f"Unbalanced \\begin{{aligned}} ({num_begin_aligned}) and \\end{{aligned}} ({num_end_aligned}) in math block in {rel_path}."
                    )

                # b. Detect bare alignment operators & outside alignment environments
                token_pattern = re.compile(r"\\begin\{([a-zA-Z*]+)\}|\\end\{([a-zA-Z*]+)\}|\\&|&")
                env_stack = []
                for match in token_pattern.finditer(block):
                    token = match.group(0)
                    if token.startswith(r"\begin{"):
                        env_stack.append(match.group(1))
                    elif token.startswith(r"\end{"):
                        end_name = match.group(2)
                        if end_name in env_stack:
                            while env_stack:
                                popped = env_stack.pop()
                                if popped == end_name:
                                    break
                    elif token == r"\&":
                        continue
                    elif token == "&":
                        if not any(env in allowed_alignment_envs for env in env_stack):
                            snippet = block[max(0, match.start() - 20):min(len(block), match.end() + 20)].strip().replace("\n", " ")
                            errors.append(
                                f"Bare alignment operator '&' outside alignment environment in {rel_path}: \"...{snippet}...\""
                            )

    assert not errors, "KaTeX / LaTeX mathematical syntax violations found:\n" + "\n".join(errors)

def test_instructions_and_readme_accessible():
    """Verify README.md and agent instruction entrypoints exist and are accessible."""
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    if not os.path.isdir(repo_root):
        repo_root = os.getcwd()

    readme_path = os.path.join(repo_root, "README.md")
    assert os.path.isfile(readme_path), f"Root README.md missing in repository at {repo_root}"
    assert os.path.getsize(readme_path) > 0, f"Root README.md is empty in repository at {repo_root}"

    agent_entrypoints = [
        os.path.join(repo_root, "AGENTS.md"),
        os.path.join(repo_root, "CLAUDE.md"),
        os.path.join(repo_root, ".agents", "AGENTS.md"),
    ]
    valid_entrypoints = [p for p in agent_entrypoints if os.path.isfile(p) and os.path.getsize(p) > 0]
    assert len(valid_entrypoints) > 0, (
        f"No non-empty agent instruction entrypoint found at {repo_root} "
        f"(checked AGENTS.md, CLAUDE.md, .agents/AGENTS.md)"
    )

def test_readme_section_5_3_does_not_wipe_schema():
    """Verify README.md Section 5.3 manual setup instructions do not wipe schema directory."""
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    if not os.path.isdir(repo_root):
        repo_root = os.getcwd()

    readme_path = os.path.join(repo_root, "README.md")
    assert os.path.isfile(readme_path), f"README.md not found at {readme_path}"
    with open(readme_path, "r", encoding="utf-8") as f:
        content = f.read()

    # Locate Section 5.3
    assert "5.3 Direct Copy" in content, "Section 5.3 Direct Copy missing from README.md"
    sec_5_3 = content.split("5.3 Direct Copy", 1)[1].split("```bash", 1)[1].split("```", 1)[0]

    # Assert rm -rf line does not delete ./schema
    rm_lines = [line for line in sec_5_3.splitlines() if line.strip().startswith("rm -rf")]
    assert len(rm_lines) > 0, "No rm -rf command found in Section 5.3 snippet"
    for rm_line in rm_lines:
        assert "./schema" not in rm_line.split(), f"Section 5.3 rm -rf line unexpectedly deletes ./schema: {rm_line}"

    # Assert safe schema copy logic is present
    assert "if [ ! -d ./schema ]; then" in sec_5_3, "Section 5.3 missing safe schema conditional copy logic"

def test_reconcile_backlog_tooling_accessible():
    """Verify scripts/reconcile_backlog.py exists, is executable, and runs to completion."""
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    if not os.path.isdir(repo_root):
        repo_root = os.getcwd()

    reconcile_path = os.path.join(repo_root, "scripts", "reconcile_backlog.py")
    assert os.path.isfile(reconcile_path), f"scripts/reconcile_backlog.py missing at {repo_root}"
    assert os.path.getsize(reconcile_path) > 0, f"scripts/reconcile_backlog.py is empty at {repo_root}"
    assert os.access(reconcile_path, os.R_OK), f"scripts/reconcile_backlog.py is not readable at {repo_root}"

    res = subprocess.run([sys.executable, reconcile_path], cwd=repo_root, capture_output=True, text=True, timeout=60)
    assert res.returncode == 0, f"scripts/reconcile_backlog.py failed with exit code {res.returncode}:\nSTDOUT:\n{res.stdout}\nSTDERR:\n{res.stderr}"
    assert "Traceback" not in res.stderr, f"scripts/reconcile_backlog.py produced unhandled exception:\n{res.stderr}"

def test_setup_git_hooks_help_accessible():
    """Verify scripts/setup_git_hooks.py --help prints usage cleanly without error or side effects."""
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    if not os.path.isdir(repo_root):
        repo_root = os.getcwd()

    script_path = os.path.join(repo_root, "scripts", "setup_git_hooks.py")
    assert os.path.isfile(script_path), f"scripts/setup_git_hooks.py missing at {repo_root}"

    res = subprocess.run([sys.executable, script_path, "--help"], cwd=repo_root, capture_output=True, text=True, timeout=10)
    assert res.returncode == 0, f"scripts/setup_git_hooks.py --help failed with exit code {res.returncode}:\n{res.stderr}"
    assert "usage:" in res.stdout, "Expected usage message in --help output"

def test_sysml_ssot_completeness_rule_accessible():
    """Verify rules/sysml-ssot-completeness.md exists, is non-empty, and satisfies governance requirements."""
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    if not os.path.isdir(repo_root):
        repo_root = os.getcwd()

    rule_path = os.path.join(repo_root, "rules", "sysml-ssot-completeness.md")
    assert os.path.isfile(rule_path), f"rules/sysml-ssot-completeness.md missing at {repo_root}"
    assert os.path.getsize(rule_path) > 0, f"rules/sysml-ssot-completeness.md is empty at {repo_root}"

    with open(rule_path, "r", encoding="utf-8") as f:
        content = f.read()

    # Verify key architectural and governance markers
    required_phrases = [
        "SysML v2",
        "Single Source of Truth",
        "Primary Tier-1 Commercial Toolchain Integration Context",
        "MATLAB / Simulink / Stateflow / Embedded Coder",
        "use case def",
        "requirement def",
    ]
    for phrase in required_phrases:
        assert phrase in content, f"Missing required governance marker '{phrase}' in rules/sysml-ssot-completeness.md"

def test_upstream_template_clean_landing_zones():
    """Verify upstream template landing zones remain pristine with zero concrete specs.

    If repository is an upstream template (.pipeline/upstream/ exists), asserts that
    docs/conops/, docs/safety/, docs/epics/, docs/features/, docs/user-stories/,
    docs/use-cases/, and schema/ contain only .gitkeep and README.md, and zero concrete
    specification files or concrete .sysml domain models.
    """
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    if not os.path.isdir(repo_root):
        repo_root = os.getcwd()

    upstream_marker = os.path.join(repo_root, ".pipeline", "upstream")
    if not os.path.isdir(upstream_marker):
        pytest.skip("Downstream project detected -- skipping upstream landing zone clean check.")

    landing_zones = [
        os.path.join("docs", "conops"),
        os.path.join("docs", "safety"),
        os.path.join("docs", "epics"),
        os.path.join("docs", "features"),
        os.path.join("docs", "user-stories"),
        os.path.join("docs", "use-cases"),
        "schema",
    ]
    allowed_files = {".gitkeep", "README.md"}
    excluded_dirs = {".git", "node_modules", ".dart_tool", "build"}

    violations = []
    for zone in landing_zones:
        zone_path = os.path.join(repo_root, zone)
        if not os.path.isdir(zone_path):
            continue
        for root, dirs, files in os.walk(zone_path):
            dirs[:] = [d for d in dirs if d not in excluded_dirs]
            for f in files:
                if f not in allowed_files:
                    rel_path = os.path.relpath(os.path.join(root, f), repo_root)
                    violations.append(rel_path)

    assert not violations, (
        f"Upstream distribution template landing zones contain concrete specification files: {violations}"
    )

def test_zero_machine_paths_in_repository():
    """Verify that no tracked repository files contain hardcoded developer workstation or machine paths."""
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    if not os.path.isdir(repo_root):
        repo_root = os.getcwd()

    res = subprocess.run(["git", "ls-files"], cwd=repo_root, capture_output=True, text=True)
    assert res.returncode == 0, f"git ls-files failed: {res.stderr}"
    tracked_files = [f for f in res.stdout.strip().split("\n") if f]

    # Pattern constructed without self-matching string
    user_prefix = "/" + "Users/"
    jail_prefix = "/" + "jail/"
    pattern = re.compile(rf"({user_prefix}[a-zA-Z0-9_-]+|{jail_prefix}[a-zA-Z0-9_-]+|file:///{user_prefix})")

    violations = []
    for rel_path in tracked_files:
        if rel_path == "tests/test_baseline.py":
            continue
        file_path = os.path.join(repo_root, rel_path)
        if os.path.islink(file_path):
            try:
                link_target = os.readlink(file_path)
                match = pattern.search(link_target)
                if match:
                    violations.append(f"{rel_path} (symlink target: {link_target})")
            except Exception as e:
                violations.append(f"Failed to read symlink {rel_path}: {e}")
            if os.path.isdir(file_path) or not os.path.exists(file_path):
                continue
        if os.path.isdir(file_path):
            continue
        try:
            with open(file_path, "r", encoding="utf-8", errors="ignore") as file_obj:
                for line_idx, line in enumerate(file_obj, start=1):
                    match = pattern.search(line)
                    if match:
                        violations.append(f"{rel_path}:{line_idx}: {line.strip()}")
        except Exception as e:
            violations.append(f"Failed to read {rel_path}: {e}")

    assert not violations, f"Found hardcoded machine paths in repository:\n" + "\n".join(violations[:20])

def test_zero_machine_paths_symlink_safety():
    """Verify that relative directory/file symlinks and dirty symlink targets are correctly processed."""
    with tempfile.TemporaryDirectory() as tmpdir:
        # 1. Clean relative directory symlink
        target_dir = os.path.join(tmpdir, "target_dir")
        os.makedirs(target_dir, exist_ok=True)
        link_dir = os.path.join(tmpdir, "link_dir")
        os.symlink("target_dir", link_dir)

        # 2. Clean relative file symlink
        target_file = os.path.join(tmpdir, "target_file.txt")
        with open(target_file, "w", encoding="utf-8") as f:
            f.write("clean content\n")
        link_file = os.path.join(tmpdir, "link_file.txt")
        os.symlink("target_file.txt", link_file)

        # 3. Dirty symlink target with machine path
        fake_machine_path = "/" + "Users/" + "dev/secret"
        dirty_link = os.path.join(tmpdir, "dirty_link")
        os.symlink(fake_machine_path, dirty_link)

        user_prefix = "/" + "Users/"
        jail_prefix = "/" + "jail/"
        pattern = re.compile(rf"({user_prefix}[a-zA-Z0-9_-]+|{jail_prefix}[a-zA-Z0-9_-]+|file:///{user_prefix})")

        violations = []
        for rel_path in ["link_dir", "link_file.txt", "dirty_link"]:
            file_path = os.path.join(tmpdir, rel_path)
            if os.path.islink(file_path):
                try:
                    link_target = os.readlink(file_path)
                    match = pattern.search(link_target)
                    if match:
                        violations.append(f"{rel_path} (symlink target: {link_target})")
                except Exception as e:
                    violations.append(f"Failed to read symlink {rel_path}: {e}")
                if os.path.isdir(file_path) or not os.path.exists(file_path):
                    continue
            if os.path.isdir(file_path):
                continue
            try:
                with open(file_path, "r", encoding="utf-8", errors="ignore") as file_obj:
                    for line_idx, line in enumerate(file_obj, start=1):
                        match = pattern.search(line)
                        if match:
                            violations.append(f"{rel_path}:{line_idx}: {line.strip()}")
            except Exception as e:
                violations.append(f"Failed to read {rel_path}: {e}")

        assert len(violations) == 1, f"Expected exactly 1 violation for dirty symlink, got: {violations}"
        assert "dirty_link" in violations[0]


def test_installer_excludes_pipeline_diagnostics():
    """Verify that scripts/install_pipeline.sh explicitly excludes/removes .pipeline/diagnostics."""
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    if not os.path.isdir(repo_root):
        repo_root = os.getcwd()

    installer_path = os.path.join(repo_root, "scripts", "install_pipeline.sh")
    assert os.path.isfile(installer_path), f"scripts/install_pipeline.sh missing at {repo_root}"

    with open(installer_path, "r", encoding="utf-8") as f:
        installer_content = f.read()

    assert 'rm -rf "$TARGET_DIR/.pipeline/diagnostics"' in installer_content or 'rm -rf "${TARGET_DIR}/.pipeline/diagnostics"' in installer_content, (
        "scripts/install_pipeline.sh does not exclude/remove .pipeline/diagnostics during downstream distribution"
    )


def test_gitignore_excludes_pipeline_diagnostics():
    """Verify that .gitignore excludes .pipeline/diagnostics/."""
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    if not os.path.isdir(repo_root):
        repo_root = os.getcwd()

    gitignore_path = os.path.join(repo_root, ".gitignore")
    assert os.path.isfile(gitignore_path), f".gitignore missing at {repo_root}"

    with open(gitignore_path, "r", encoding="utf-8") as f:
        lines = [line.strip() for line in f]

    assert ".pipeline/diagnostics/" in lines or ".pipeline/diagnostics" in lines, (
        ".gitignore does not contain .pipeline/diagnostics/"
    )


def test_upstream_blueprint_domain_cleanliness_clean_upstream():
    """Verify Check 18 passes on the clean upstream repository."""
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    if not os.path.isdir(repo_root):
        repo_root = os.getcwd()
    verify_upstream_blueprint_domain_cleanliness(repo_root)


def test_upstream_blueprint_domain_cleanliness_skips_downstream():
    """Verify Check 18 skips cleanly when no upstream marker is present."""
    with tempfile.TemporaryDirectory() as tmpdir:
        # Downstream repo (no .pipeline/upstream)
        blueprints_dir = os.path.join(tmpdir, "docs", "architecture", "blueprints")
        os.makedirs(blueprints_dir, exist_ok=True)
        # Even with domain files present, downstream check should skip
        with open(os.path.join(blueprints_dir, "DEAP_FLIGHT_SYSTEMS_SAFETY_CONCEPT_PAPER.md"), "w", encoding="utf-8") as f:
            f.write("# Flight Systems Concept Paper\n")
        with open(os.path.join(blueprints_dir, "DEAP_SYSML_V2_SAFETY_MODEL_SPECIFICATION.sysml"), "w", encoding="utf-8") as f:
            f.write("package SafetyModel {}\n")

        # Should not raise SystemExit
        verify_upstream_blueprint_domain_cleanliness(tmpdir)


def test_upstream_blueprint_domain_cleanliness_detects_domain_concept_papers_and_sysml():
    """Verify Check 18 detects and rejects domain platform concept papers and sysml models in upstream blueprints."""
    test_filenames = [
        "DEAP_FLIGHT_SYSTEMS_SAFETY_CONCEPT_PAPER.md",
        "DEAP_UAS_INFRASTRUCTURE_SAFETY_CONCEPT_PAPER.md",
        "DEAP_PIPELINE_0_FRONTEND_SYSTEMS_SAFETY_BLUEPRINT.md",
        "DEAP_SYSML_V2_SAFETY_MODEL_SPECIFICATION.sysml",
        "FLIGHT_SYSTEMS_BLUEPRINT.md",
        "UAS_INFRASTRUCTURE_ARCH.md",
        "FRONTEND_SYSTEMS_SPEC.md",
        "CUSTOM_SAFETY_MODEL.sysml",
        "AUTONOMY_CONCEPT_PAPER.md",
    ]

    for bad_file in test_filenames:
        with tempfile.TemporaryDirectory() as tmpdir:
            upstream_marker = os.path.join(tmpdir, ".pipeline", "upstream")
            os.makedirs(upstream_marker, exist_ok=True)

            blueprints_dir = os.path.join(tmpdir, "docs", "architecture", "blueprints")
            os.makedirs(blueprints_dir, exist_ok=True)

            with open(os.path.join(blueprints_dir, bad_file), "w", encoding="utf-8") as f:
                f.write(f"# Polluted domain concept paper {bad_file}\n")

            with pytest.raises(SystemExit) as exc_info:
                verify_upstream_blueprint_domain_cleanliness(tmpdir)
            assert exc_info.value.code == 1


def test_installer_scaffolds_downstream_agents_md():
    """Verify that scripts/install_pipeline.sh scaffolds downstream AGENTS.md files with DOWNSTREAM_CUSTOMER_PROJECT and full governance armor."""
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    if not os.path.isdir(repo_root):
        repo_root = os.getcwd()

    installer_path = os.path.join(repo_root, "scripts", "install_pipeline.sh")
    assert os.path.isfile(installer_path), f"scripts/install_pipeline.sh missing at {repo_root}"

    with tempfile.TemporaryDirectory() as tmpdir:
        # Pre-seed a dummy .DS_Store file to verify installer purges it
        dummy_ds = os.path.join(tmpdir, ".DS_Store")
        with open(dummy_ds, "w", encoding="utf-8") as f:
            f.write("test_ds_store")

        res = subprocess.run(
            ["bash", installer_path, tmpdir, "-p", "github"],
            cwd=repo_root,
            capture_output=True,
            text=True,
            timeout=60,
        )
        assert res.returncode == 0, f"install_pipeline.sh failed with exit code {res.returncode}:\nSTDOUT:\n{res.stdout}\nSTDERR:\n{res.stderr}"

        agents_dot_path = os.path.join(tmpdir, ".agents", "AGENTS.md")
        agents_root_path = os.path.join(tmpdir, "AGENTS.md")

        assert os.path.isfile(agents_dot_path), f".agents/AGENTS.md missing in target directory {tmpdir}"
        assert os.path.isfile(agents_root_path), f"AGENTS.md missing in target directory {tmpdir}"

        with open(agents_dot_path, "r", encoding="utf-8") as f:
            dot_content = f.read()
        with open(agents_root_path, "r", encoding="utf-8") as f:
            root_content = f.read()

        assert "Mandatory Subagent Self-Rejection Pre-Flight Gate" in dot_content, ".agents/AGENTS.md missing Mandatory Subagent Self-Rejection Pre-Flight Gate"
        assert "Mandatory Subagent Self-Rejection Pre-Flight Gate" in root_content, "AGENTS.md missing Mandatory Subagent Self-Rejection Pre-Flight Gate"

        assert "DOWNSTREAM_CUSTOMER_PROJECT" in dot_content, ".agents/AGENTS.md does not contain DOWNSTREAM_CUSTOMER_PROJECT"
        assert "DOWNSTREAM_CUSTOMER_PROJECT" in root_content, "AGENTS.md does not contain DOWNSTREAM_CUSTOMER_PROJECT"

        assert "UPSTREAM_SPEC_CORE_COMPILER" not in dot_content, ".agents/AGENTS.md unexpectedly contains UPSTREAM_SPEC_CORE_COMPILER"
        assert "UPSTREAM_SPEC_CORE_COMPILER" not in root_content, "AGENTS.md unexpectedly contains UPSTREAM_SPEC_CORE_COMPILER"

        assert "Mandatory Subagent Dispatch for Research, Specification & Implementation Loops" in dot_content, ".agents/AGENTS.md missing Mandatory Subagent Dispatch"
        assert "Mandatory Subagent Dispatch for Research, Specification & Implementation Loops" in root_content, "AGENTS.md missing Mandatory Subagent Dispatch"

        ds_store_files = []
        for root, dirs, files in os.walk(tmpdir):
            for f in files:
                if f == ".DS_Store":
                    ds_store_files.append(os.path.join(root, f))
        assert not ds_store_files, f"Found .DS_Store files in target directory: {ds_store_files}"


class TestCheck18BlueprintDomainCleanliness(unittest.TestCase):
    """Unit tests for Check 18 (Upstream Blueprint Domain Cleanliness Gate)."""

    def test_clean_upstream_passes(self):
        repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        if not os.path.isdir(repo_root):
            repo_root = os.getcwd()
        verify_upstream_blueprint_domain_cleanliness(repo_root)

    def test_downstream_skips(self):
        test_upstream_blueprint_domain_cleanliness_skips_downstream()

    def test_failure_injection_domain_concept_papers_and_sysml(self):
        test_filenames = [
            "DEAP_FLIGHT_SYSTEMS_SAFETY_CONCEPT_PAPER.md",
            "DEAP_UAS_INFRASTRUCTURE_SAFETY_CONCEPT_PAPER.md",
            "DEAP_PIPELINE_0_FRONTEND_SYSTEMS_SAFETY_BLUEPRINT.md",
            "DEAP_SYSML_V2_SAFETY_MODEL_SPECIFICATION.sysml",
            "FLIGHT_SYSTEMS_BLUEPRINT.md",
            "UAS_INFRASTRUCTURE_ARCH.md",
            "FRONTEND_SYSTEMS_SPEC.md",
            "CUSTOM_SAFETY_MODEL.sysml",
            "AUTONOMY_CONCEPT_PAPER.md",
        ]
        for bad_file in test_filenames:
            with tempfile.TemporaryDirectory() as tmpdir:
                upstream_marker = os.path.join(tmpdir, ".pipeline", "upstream")
                os.makedirs(upstream_marker, exist_ok=True)

                blueprints_dir = os.path.join(tmpdir, "docs", "architecture", "blueprints")
                os.makedirs(blueprints_dir, exist_ok=True)

                with open(os.path.join(blueprints_dir, bad_file), "w", encoding="utf-8") as f:
                    f.write(f"# Polluted domain concept paper {bad_file}\n")

                with self.assertRaises(SystemExit) as cm:
                    verify_upstream_blueprint_domain_cleanliness(tmpdir)
                self.assertEqual(cm.exception.code, 1)

    def test_run_all_checks_includes_check_18(self):
        repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        if not os.path.isdir(repo_root):
            repo_root = os.getcwd()
        run_all_checks(repo_root)


def test_installer_refuses_self_target():
    """Verify scripts/install_pipeline.sh refuses self-targeting in upstream and downstream environments without deleting files."""
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    if not os.path.isdir(repo_root):
        repo_root = os.getcwd()

    installer_path = os.path.join(repo_root, "scripts", "install_pipeline.sh")
    assert os.path.isfile(installer_path), f"scripts/install_pipeline.sh missing at {repo_root}"

    # 1. Upstream environment (with .pipeline/upstream marker)
    with tempfile.TemporaryDirectory() as tmpdir:
        scripts_dir = os.path.join(tmpdir, "scripts")
        skills_dir = os.path.join(tmpdir, "skills")
        rules_dir = os.path.join(tmpdir, "rules")
        pipeline_dir = os.path.join(tmpdir, ".pipeline")
        upstream_marker = os.path.join(pipeline_dir, "upstream")
        agents_dir = os.path.join(tmpdir, ".agents")

        os.makedirs(scripts_dir, exist_ok=True)
        os.makedirs(skills_dir, exist_ok=True)
        os.makedirs(rules_dir, exist_ok=True)
        os.makedirs(pipeline_dir, exist_ok=True)
        os.makedirs(upstream_marker, exist_ok=True)
        os.makedirs(agents_dir, exist_ok=True)

        shutil.copy(installer_path, os.path.join(scripts_dir, "install_pipeline.sh"))
        with open(os.path.join(skills_dir, "upstream_skill.md"), "w", encoding="utf-8") as f:
            f.write("# Upstream Skill\n")
        with open(os.path.join(rules_dir, "upstream_rule.md"), "w", encoding="utf-8") as f:
            f.write("# Upstream Rule\n")

        # Test self-target with "."
        res_dot = subprocess.run(
            ["bash", os.path.join("scripts", "install_pipeline.sh"), "."],
            cwd=tmpdir,
            capture_output=True,
            text=True,
            timeout=30,
        )
        assert res_dot.returncode == 1, f"Expected returncode 1 for upstream self-target with '.', got {res_dot.returncode}"
        assert "REFUSING: target is the pipeline repository itself, not a downstream project." in res_dot.stderr
        assert os.path.isfile(os.path.join(skills_dir, "upstream_skill.md")), "Upstream self-target deleted skills directory/files"
        assert os.path.isfile(os.path.join(rules_dir, "upstream_rule.md")), "Upstream self-target deleted rules directory/files"

        # Test self-target with absolute path
        res_abs = subprocess.run(
            ["bash", os.path.join("scripts", "install_pipeline.sh"), tmpdir],
            cwd=tmpdir,
            capture_output=True,
            text=True,
            timeout=30,
        )
        assert res_abs.returncode == 1, f"Expected returncode 1 for upstream self-target with absolute path, got {res_abs.returncode}"
        assert "REFUSING: target is the pipeline repository itself, not a downstream project." in res_abs.stderr
        assert os.path.isfile(os.path.join(skills_dir, "upstream_skill.md")), "Upstream self-target deleted skills directory/files"
        assert os.path.isfile(os.path.join(rules_dir, "upstream_rule.md")), "Upstream self-target deleted rules directory/files"

    # 2. Downstream environment (NO .pipeline/upstream marker)
    with tempfile.TemporaryDirectory() as tmpdir:
        scripts_dir = os.path.join(tmpdir, "scripts")
        skills_dir = os.path.join(tmpdir, "skills")
        rules_dir = os.path.join(tmpdir, "rules")
        pipeline_dir = os.path.join(tmpdir, ".pipeline")
        agents_dir = os.path.join(tmpdir, ".agents")

        os.makedirs(scripts_dir, exist_ok=True)
        os.makedirs(skills_dir, exist_ok=True)
        os.makedirs(rules_dir, exist_ok=True)
        os.makedirs(pipeline_dir, exist_ok=True)
        os.makedirs(agents_dir, exist_ok=True)

        shutil.copy(installer_path, os.path.join(scripts_dir, "install_pipeline.sh"))
        with open(os.path.join(skills_dir, "downstream_skill.md"), "w", encoding="utf-8") as f:
            f.write("# Downstream Skill\n")
        with open(os.path.join(rules_dir, "downstream_rule.md"), "w", encoding="utf-8") as f:
            f.write("# Downstream Rule\n")
        with open(os.path.join(pipeline_dir, "config.json"), "w", encoding="utf-8") as f:
            f.write("{}\n")

        # Test self-target with "."
        res_down_dot = subprocess.run(
            ["bash", os.path.join("scripts", "install_pipeline.sh"), "."],
            cwd=tmpdir,
            capture_output=True,
            text=True,
            timeout=30,
        )
        assert res_down_dot.returncode == 1, f"Expected returncode 1 for downstream self-target with '.', got {res_down_dot.returncode}"
        assert "REFUSING: target directory is identical to installer root" in res_down_dot.stderr
        assert os.path.isfile(os.path.join(skills_dir, "downstream_skill.md")), "Downstream self-target deleted skills directory/files"
        assert os.path.isfile(os.path.join(rules_dir, "downstream_rule.md")), "Downstream self-target deleted rules directory/files"
        assert os.path.isfile(os.path.join(pipeline_dir, "config.json")), "Downstream self-target deleted pipeline config"

        # Test self-target with absolute path
        res_down_abs = subprocess.run(
            ["bash", os.path.join("scripts", "install_pipeline.sh"), tmpdir],
            cwd=tmpdir,
            capture_output=True,
            text=True,
            timeout=30,
        )
        assert res_down_abs.returncode == 1, f"Expected returncode 1 for downstream self-target with absolute path, got {res_down_abs.returncode}"
        assert "REFUSING: target directory is identical to installer root" in res_down_abs.stderr
        assert os.path.isfile(os.path.join(skills_dir, "downstream_skill.md")), "Downstream self-target deleted skills directory/files"
        assert os.path.isfile(os.path.join(rules_dir, "downstream_rule.md")), "Downstream self-target deleted rules directory/files"
        assert os.path.isfile(os.path.join(pipeline_dir, "config.json")), "Downstream self-target deleted pipeline config"


class TestInstallerScaffolding(unittest.TestCase):
    """Unit tests for installer scaffolding behavior."""

    def test_installer_scaffolds_downstream_agents_md(self):
        test_installer_scaffolds_downstream_agents_md()

    def test_installer_refuses_self_target(self):
        test_installer_refuses_self_target()


class TestProofBlockASTParser(unittest.TestCase):
    """Unit tests for MarkdownTableASTParser.parse_proof_blocks (Issue #189)."""

    def test_parse_proof_blocks_markdown_subheadings(self):
        """Verify proof blocks structured with markdown subheadings for parts 1 to 5 are correctly parsed."""
        content = """
# Safety Assurance Cases

### Theorem SAF-01: Forward Invariance of Conflict-Free Minimum Separation

#### Part 1 -- Proposition Statement
For any initial state $x_0 \in \mathcal{C}$, the safety envelope $h(x(t)) \ge 0$ holds for all $t \ge 0$.

#### Part 2 -- Assumptions
1. Continuous differentiability of system dynamics $\dot{x} = f(x) + g(x)u$.
2. Actuator saturation limits $|u| \le u_{\max}$.

#### Part 3 -- Barrier Function / Invariant
We define candidate zeroing control barrier function $B(x) = d(x) - d_{\min}$.

#### Part 4 -- Derivation & Inductive Step
Taking the Lie derivative:
$$\dot{B}(x) = L_f B(x) + L_g B(x) u \ge -\alpha(B(x))$$

#### Part 5 -- Conclusion & Q.E.D.
By Nagumo's theorem, the set $\mathcal{C}$ is forward invariant under feedback controller $k(x)$. Q.E.D.

### Next Unrelated Section
This section is outside the proof block.
"""
        blocks = MarkdownTableASTParser.parse_proof_blocks(content)
        self.assertEqual(len(blocks), 1)
        block = blocks[0]
        self.assertEqual(block.theorem_id, "SAF-01")
        self.assertIn("Part 1", block.proposition)
        self.assertIn("Part 2", block.assumptions)
        self.assertIn("Part 3", block.barrier_function)
        self.assertIn("Part 4", block.derivation)
        self.assertIn("Part 5", block.conclusion)

        report = CartesianProductValidator.verify_proof_structure(blocks)
        self.assertTrue(report.is_conforming)
        self.assertEqual(len(report.malformed_proofs), 0)

    def test_parse_proof_blocks_numbered_and_subheadings_multi_block(self):
        """Verify multiple proof blocks with mixed formatting styles parse cleanly."""
        content = """
### Theorem SEC-01: Collision Avoidance
#### Part 1 -- Proposition
Proposition statement here.
#### Part 2 -- Assumptions
Assumptions text.
#### Part 3 -- Invariant Barrier
Invariant definition.
#### Part 4 -- Derivation
Inductive derivation.
#### Part 5 -- Conclusion QED
Conclusion statement.

### Theorem SEC-02: Geofence Containment
1. Proposition: Aircraft stays inside geofence.
2. Assumptions: GPS accuracy within 1m.
3. Invariant: Distance to boundary is positive.
4. Derivation: Velocity vector bounded.
5. Conclusion: System containment verified (qed).
"""
        blocks = MarkdownTableASTParser.parse_proof_blocks(content)
        self.assertEqual(len(blocks), 2)
        self.assertEqual(blocks[0].theorem_id, "SEC-01")
        self.assertEqual(blocks[1].theorem_id, "SEC-02")

        report = CartesianProductValidator.verify_proof_structure(blocks)
        self.assertTrue(report.is_conforming)
        self.assertEqual(len(report.malformed_proofs), 0)


class TestValidateDomainTypes(unittest.TestCase):
    """Unit and regression tests for _validate_domain_types (#210)."""

    def test_validate_domain_types_typescript_enum_and_types_accepted(self):
        """Verify TypeScript enum declarations are accepted alongside interface, class, and type."""
        with tempfile.TemporaryDirectory() as tmpdir:
            rules = {
                "validation_rules": {
                    "mandated_classes": [
                        "FlightMode",
                        "DroneState",
                        "TelemetryData",
                        "NavigationCommand",
                    ]
                }
            }
            with open(os.path.join(tmpdir, "codebase_rules.json"), "w", encoding="utf-8") as f:
                json.dump(rules, f)

            domain_dir = os.path.join(tmpdir, "src", "domain")
            os.makedirs(domain_dir, exist_ok=True)
            types_ts = os.path.join(domain_dir, "types.ts")
            with open(types_ts, "w", encoding="utf-8") as f:
                f.write(
                    "export enum FlightMode {\n"
                    "    AUTO = 'AUTO',\n"
                    "    MANUAL = 'MANUAL',\n"
                    "}\n\n"
                    "export interface DroneState {\n"
                    "    id: string;\n"
                    "}\n\n"
                    "export class TelemetryData {\n"
                    "    timestamp: number;\n"
                    "}\n\n"
                    "export type NavigationCommand = {\n"
                    "    action: string;\n"
                    "};\n"
                )

            # Should complete without error / exit
            _validate_domain_types(tmpdir, tmpdir, "ts", "src/domain")

    def test_validate_domain_types_typescript_missing_type_fails(self):
        """Verify TypeScript type validation fails when a mandated enum/type is missing."""
        with tempfile.TemporaryDirectory() as tmpdir:
            rules = {
                "validation_rules": {
                    "mandated_classes": [
                        "FlightMode",
                        "MissingDomainType",
                    ]
                }
            }
            with open(os.path.join(tmpdir, "codebase_rules.json"), "w", encoding="utf-8") as f:
                json.dump(rules, f)

            domain_dir = os.path.join(tmpdir, "src", "domain")
            os.makedirs(domain_dir, exist_ok=True)
            types_ts = os.path.join(domain_dir, "types.ts")
            with open(types_ts, "w", encoding="utf-8") as f:
                f.write(
                    "export enum FlightMode {\n"
                    "    AUTO = 'AUTO',\n"
                    "    MANUAL = 'MANUAL',\n"
                    "}\n"
                )

            with self.assertRaises(SystemExit) as cm:
                _validate_domain_types(tmpdir, tmpdir, "ts", "src/domain")
            self.assertEqual(cm.exception.code, 1)

    def test_validate_domain_types_dart_enum_and_classes_accepted(self):
        """Verify Dart enum declarations are accepted alongside class and mixin."""
        with tempfile.TemporaryDirectory() as tmpdir:
            rules = {
                "validation_rules": {
                    "mandated_classes": [
                        "FlightMode",
                        "DroneState",
                    ]
                }
            }
            with open(os.path.join(tmpdir, "codebase_rules.json"), "w", encoding="utf-8") as f:
                json.dump(rules, f)

            domain_dir = os.path.join(tmpdir, "lib", "src", "domain")
            os.makedirs(domain_dir, exist_ok=True)
            types_dart = os.path.join(domain_dir, "types.dart")
            with open(types_dart, "w", encoding="utf-8") as f:
                f.write(
                    "enum FlightMode {\n"
                    "    auto,\n"
                    "    manual,\n"
                    "}\n\n"
                    "class DroneState {\n"
                    "    final String id;\n"
                    "    DroneState(this.id);\n"
                    "}\n"
                )

            # Should complete without error / exit
            _validate_domain_types(tmpdir, tmpdir, "dart", "lib/src/domain")


if __name__ == "__main__":
    unittest.main()


