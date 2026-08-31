"""
Downstream Environment & Runtime Integrity Verification Suite.
/// Realises: [BaselineVerification]
"""
import sys
import os
import re
import subprocess
import tempfile
import pytest

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
        pytest.skip("Downstream project detected — skipping upstream landing zone clean check.")

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
        try:
            with open(file_path, "r", encoding="utf-8", errors="ignore") as file_obj:
                for line_idx, line in enumerate(file_obj, start=1):
                    match = pattern.search(line)
                    if match:
                        violations.append(f"{rel_path}:{line_idx}: {line.strip()}")
        except Exception as e:
            violations.append(f"Failed to read {rel_path}: {e}")

    assert not violations, f"Found hardcoded machine paths in repository:\n" + "\n".join(violations[:20])

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
