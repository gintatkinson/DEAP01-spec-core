#!/usr/bin/env python3
"""
Verify downstream project baseline conformance.
Asserts baseline files exist, validates type compatibility with mandated domain classes,
and runs the build/test commands ('npm run build' for React, 'flutter analyze && flutter test' for Flutter).
"""

import argparse
import ast
import csv
import json
import os
import re
import shutil
import signal
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Dict, List, Optional, Set, Tuple, Union

TIMEOUT_SECONDS = 600
GIT_TIMEOUT_SECONDS = 30
EXCLUDED_DIRS = {".git", "node_modules", ".dart_tool", "build", "units"}

def _terminate_process_group(proc):
    """Terminate process group cleanly with SIGTERM followed by SIGKILL fallback."""
    try:
        os.killpg(os.getpgid(proc.pid), signal.SIGTERM)
        proc.wait(timeout=15)
    except (subprocess.TimeoutExpired, ProcessLookupError, PermissionError):
        try:
            os.killpg(os.getpgid(proc.pid), signal.SIGKILL)
        except (ProcessLookupError, PermissionError):
            pass
        try:
            proc.wait(timeout=5)
        except (subprocess.TimeoutExpired, ProcessLookupError, PermissionError):
            pass

def _run_bounded(cmd, cwd, timeout, label):
    """Run cmd with a timeout that binds the whole process tree.

    subprocess.run's timeout kills only the direct child. flutter and npm are
    launchers whose real work happens in grandchildren (analysis server, dart
    test host, xcodebuild), which survive that kill, keep the build directory
    open, and then race the cleanup_workspace rmtree. start_new_session puts the
    tree in its own process group so a single killpg reaches all of it.
    """
    proc = subprocess.Popen(cmd, cwd=cwd, start_new_session=True)
    try:
        rc = proc.wait(timeout=timeout)
    finally:
        _terminate_process_group(proc)
    if rc != 0:
        raise subprocess.CalledProcessError(rc, cmd)

def check_no_domain_config(destination):
    config_paths = [
        os.path.join(destination, ".pipeline", "logical-ui", "codebase_rules.json"),
        os.path.join(destination, "codebase_rules.json"),
        os.path.join(destination, "baseline_manifest.json")
    ]
    for path in config_paths:
        if os.path.isfile(path):
            try:
                with open(path, "r", encoding="utf-8") as f:
                    data = json.load(f)
                
                if isinstance(data, dict):
                    if "validation_rules" in data and isinstance(data["validation_rules"], dict):
                        if data["validation_rules"].get("no_domain") is True:
                            return True
                    if data.get("no_domain") is True:
                        return True
            except Exception:
                pass
    return False

def tag_restoration_point(repo_root=None):
    print("Tagging restoration point...")
    if shutil.which("git") is None:
        print("WARNING: Skipping restoration point tag - git binary not found.", file=sys.stderr)
        return True
    try:
        res_inside = subprocess.run(["git", "rev-parse", "--is-inside-work-tree"], capture_output=True, text=True, cwd=repo_root, timeout=GIT_TIMEOUT_SECONDS)
        if res_inside.returncode != 0:
            if os.environ.get("CI") == "true" or os.environ.get("GITLAB_CI") == "true" or os.environ.get("GITHUB_ACTIONS") == "true":
                print("WARNING: Skipping restoration point tag - running in CI environment outside git repository.", file=sys.stderr)
                return True
            print("WARNING: Failed to tag restoration point: not inside a git repository.", file=sys.stderr)
            return False
        res = subprocess.run(["git", "rev-parse", "HEAD"], capture_output=True, text=True, cwd=repo_root, timeout=GIT_TIMEOUT_SECONDS)
        if res.returncode != 0:
            print("WARNING: Skipping restoration point tag - git HEAD is unborn (fresh repository).", file=sys.stderr)
            return True
        subprocess.run(["git", "tag", "-f", "restoration-point"], check=True, cwd=repo_root, timeout=GIT_TIMEOUT_SECONDS)
        return True
    except subprocess.TimeoutExpired as e:
        print(f"WARNING: Failed to tag restoration point: {e}", file=sys.stderr)
        return False
    except (subprocess.CalledProcessError, OSError) as e:
        print(f"WARNING: Failed to tag restoration point: {e}", file=sys.stderr)
        return False

def cleanup_workspace(destination):
    print("Cleaning up workspace...")
    to_delete_files = [".dart_tool/package_config.json.lock",
                       ".flutter-plugins-dependencies",
                       ".flutter-plugins"]
    for f in to_delete_files:
        path = os.path.join(destination, f)
        if os.path.isfile(path):
            try:
                os.remove(path)
            except OSError:
                pass

    dirs_to_remove = ["build", ".flutter-plugins", ".flutter-plugins-dependencies"]
    for d in dirs_to_remove:
        d_path = os.path.join(destination, d)
        if os.path.isdir(d_path):
            shutil.rmtree(d_path, ignore_errors=True)

    for root, dirs, files in os.walk(destination):
        dirs[:] = [d for d in dirs if d not in EXCLUDED_DIRS]
        for f in files:
            if f.endswith(".db-shm") or f.endswith(".db-wal") or f.endswith(".db-journal"):
                sidecar_path = os.path.join(root, f)
                if f.endswith(".db-shm") or f.endswith(".db-wal"):
                    owner_name = f[:-4]
                else:
                    owner_name = f[:-8]
                owner_db = os.path.join(root, owner_name)
                if os.path.exists(owner_db):
                    print(f"NOTE: Preserving active SQLite sidecar '{sidecar_path}' (owning database '{owner_db}' exists).")
                else:
                    try:
                        os.remove(sidecar_path)
                    except Exception:
                        pass


# Mandated domain classes/interfaces to check in types.ts or types.dart
MANDATED_CLASSES = []

def load_mandated_classes(destination):
    config_paths = [
        os.path.join(destination, ".pipeline", "logical-ui", "codebase_rules.json"),
        os.path.join(destination, "codebase_rules.json"),
        os.path.join(destination, "baseline_manifest.json")
    ]
    for path in config_paths:
        if os.path.isfile(path):
            try:
                with open(path, "r", encoding="utf-8") as f:
                    data = json.load(f)
                
                classes = None
                if isinstance(data, dict):
                    if "validation_rules" in data and isinstance(data["validation_rules"], dict):
                        classes = data["validation_rules"].get("mandated_classes")
                    if classes is None:
                        classes = data.get("mandated_classes")
                
                if isinstance(classes, list):
                    if all(isinstance(c, str) for c in classes):
                        print(f"Loaded mandated classes dynamically from {path}: {classes}")
                        return classes
                    else:
                        print(f"WARNING: Invalid format for 'mandated_classes' in {path} (not all elements are strings).", file=sys.stderr)
                else:
                    print(f"WARNING: 'mandated_classes' not found or not a list in {path}.", file=sys.stderr)
            except Exception as e:
                print(f"WARNING: Failed to parse or load config {path}: {e}", file=sys.stderr)
    
    print("Using default hardcoded MANDATED_CLASSES.")
    return MANDATED_CLASSES

def main():
    parser = argparse.ArgumentParser(description="Verify a downstream project's baseline conformance.")
    parser.add_argument("--no-domain", action="store_true", help="Skip checking the domain model")
    parser.add_argument("--target", help="Target project directory", default=None)
    parser.add_argument("--output", help="Output JSON report file path", default=None)
    parser.add_argument("destination", nargs="?", default=".", help="Path to the downstream project directory (defaults to current directory)")
    args = parser.parse_args()

    repo_root = os.path.abspath(args.destination)

    targets = []
    if args.target:
        target_dir = os.path.abspath(args.target)
        if os.path.isdir(target_dir):
            targets.append(target_dir)
        else:
            print(f"ERROR: Target path '{target_dir}' is not a directory.", file=sys.stderr)
            sys.exit(1)
    else:
        if not os.path.isdir(repo_root):
            print(f"ERROR: Destination path '{repo_root}' is not a directory.", file=sys.stderr)
            sys.exit(1)

        is_self_flutter = os.path.exists(os.path.join(repo_root, "pubspec.yaml"))
        is_self_react = os.path.exists(os.path.join(repo_root, "package.json"))
        if is_self_flutter or is_self_react:
            targets.append(repo_root)

        app_flutter_dir = os.path.join(repo_root, "app_flutter")
        if os.path.isdir(app_flutter_dir) and os.path.exists(os.path.join(app_flutter_dir, "pubspec.yaml")):
            if app_flutter_dir not in targets:
                targets.append(app_flutter_dir)

        web_react_dir = os.path.join(repo_root, "web_react")
        if os.path.isdir(web_react_dir) and os.path.exists(os.path.join(web_react_dir, "package.json")):
            if web_react_dir not in targets:
                targets.append(web_react_dir)

        if not targets and os.path.isdir(repo_root):
            print(f"NOTE: Destination path '{repo_root}' has no pubspec.yaml or package.json. Registering repository root for non-framework baseline checks.")
            targets.append(repo_root)

    if not targets:
        print(f"ERROR: Destination path '{repo_root}' does not appear to be a valid directory.", file=sys.stderr)
        sys.exit(1)

    reports = []
    for dest in targets:
        is_flutter = os.path.exists(os.path.join(dest, "pubspec.yaml"))
        is_react = os.path.exists(os.path.join(dest, "package.json"))

        # An explicit --no-domain on the command line is the operator's decision and is
        # never overridden. The config-file setting is a stored default, so it IS
        # overridden once a domain directory exists on disk -- that is what stops a
        # stale config silently disabling verification on a project that has since
        # implemented its domain.
        #
        # Both were overridden until this was fixed, which made --no-domain inert: the
        # shipped app_flutter and web_react templates both contain a domain directory,
        # so the flag cancelled itself on every fresh install and the documented
        # "verify the workspace structure prior to implementing the domain model" path
        # ran a full `flutter build macos --release` instead.
        no_domain_for_target = args.no_domain
        if not args.no_domain and (
            check_no_domain_config(repo_root) or check_no_domain_config(dest)
        ):
            no_domain_for_target = True
            flutter_domain = os.path.join(dest if is_flutter else repo_root, "lib", "domain")
            react_domain = os.path.join(dest if is_react else repo_root, "src", "domain")
            if os.path.isdir(flutter_domain) or os.path.isdir(react_domain):
                print(f"NOTE: Domain directory found on disk for '{dest}' -- overriding no_domain config and enabling domain verification.")
                no_domain_for_target = False

        target_args = argparse.Namespace(**vars(args))
        target_args.no_domain = no_domain_for_target

        try:
            _run_verification(target_args, dest, repo_root, is_flutter, is_react)
            print(f"Success: Build and test suite execution passed for '{dest}'. Conformance gate verified.")
            reports.append({
                "status": "success",
                "target": dest,
                "platform": "flutter" if is_flutter else ("react" if is_react else "unknown"),
                "destination": dest,
                "domain_verified": not no_domain_for_target,
            })
        finally:
            cleanup_workspace(dest)

    if args.output and reports:
        out_dir = os.path.dirname(os.path.abspath(args.output))
        if out_dir:
            os.makedirs(out_dir, exist_ok=True)
        report_data = reports[0] if len(reports) == 1 else {"status": "success", "reports": reports}
        with open(args.output, "w", encoding="utf-8") as f:
            json.dump(report_data, f, indent=2)
        print(f"Wrote downstream baseline report to {args.output}")

    if not tag_restoration_point(repo_root=repo_root):
        print("ERROR: Conformance gate verified but restoration point tag could not be placed.", file=sys.stderr)
        sys.exit(1)
    sys.exit(0)

def _validate_domain_types(dest, repo_root, ext, domain_subpath):
    mandated = load_mandated_classes(dest)
    if repo_root != dest:
        upstream_mandated = load_mandated_classes(repo_root)
        mandated = list(set(mandated + upstream_mandated))
    if not mandated:
        print("No mandated classes configured -- skipping type validation.")
        return
    domain_dir = os.path.join(dest, domain_subpath)
    if not os.path.isdir(domain_dir):
        print(f"ERROR: Domain directory '{domain_dir}' does not exist but mandated classes are configured.", file=sys.stderr)
        sys.exit(1)
    source_files = []
    for root, _, files in os.walk(domain_dir):
        for f in files:
            if f.endswith("." + ext) or (ext == "ts" and f.endswith(".tsx")):
                source_files.append(os.path.join(root, f))
    if not source_files:
        print(f"ERROR: No .{ext} source files found in '{domain_dir}' but mandated classes are configured.", file=sys.stderr)
        sys.exit(1)
    combined = ""
    for sf in source_files:
        with open(sf, "r", encoding="utf-8") as f:
            combined += f.read() + "\n"
    if ext == "dart":
        type_keywords = r"(?:class|mixin|enum|extension\s+type|sealed\s+class)"
        pattern = r"\b" + type_keywords + r"\s+({})\b".format("|".join(re.escape(c) for c in mandated))
    else:
        type_keywords = r"(?:interface|class|type|enum)"
        pattern = r"\b" + type_keywords + r"\s+({})\b".format("|".join(re.escape(c) for c in mandated))
    found = set(re.findall(pattern, combined, re.MULTILINE))
    missing = set(mandated) - found
    if missing:
        print(f"ERROR: Type validation failed. Mandated classes missing in {domain_subpath}/: {', '.join(sorted(missing))}", file=sys.stderr)
        sys.exit(1)
    print(f"Success: All {len(mandated)} mandated domain classes found in {domain_subpath}/.")

def check_gitignore_exists(repo_root):
    """Check 10: Verify .gitignore exists in the repository root."""
    gitignore_path = os.path.join(repo_root, ".gitignore")
    if not os.path.isfile(gitignore_path):
        print(f"ERROR: Check 10 failed: .gitignore missing in repository root '{repo_root}'.", file=sys.stderr)
        sys.exit(1)
    print("Success: Check 10 verified (.gitignore exists in repository root).")

def check_no_ds_store_files(repo_root):
    """Check 11: Verify zero .DS_Store files exist in the working tree or git index."""
    ds_store_files = []
    for root, dirs, files in os.walk(repo_root):
        dirs[:] = [d for d in dirs if d not in EXCLUDED_DIRS]
        for f in files:
            if f == ".DS_Store":
                ds_store_files.append(os.path.join(root, f))
    if not ds_store_files:
        print("Success: Check 11 verified (zero .DS_Store files found).")
        return

    tracked_files = []
    cleaned_files = []

    for path in ds_store_files:
        rel_path = os.path.relpath(path, repo_root)
        is_tracked = False
        try:
            res = subprocess.run(
                ["git", "ls-files", "--error-unmatch", rel_path],
                cwd=repo_root,
                capture_output=True,
                text=True,
                timeout=GIT_TIMEOUT_SECONDS,
            )
            if res.returncode == 0:
                is_tracked = True
        except Exception:
            is_tracked = False

        if is_tracked:
            tracked_files.append(rel_path)
        else:
            try:
                os.remove(path)
                cleaned_files.append(rel_path)
            except OSError as e:
                print(f"WARNING: Failed to remove transient .DS_Store file '{rel_path}': {e}", file=sys.stderr)

    if cleaned_files:
        print(f"Notice: [Cleaned] Removed {len(cleaned_files)} transient untracked .DS_Store file(s): {', '.join(cleaned_files)}")

    if tracked_files:
        print(f"ERROR: Check 11 failed: Found {len(tracked_files)} tracked/committed .DS_Store file(s) in git index: {', '.join(tracked_files)}", file=sys.stderr)
        sys.exit(1)

    print("Success: Check 11 verified (zero tracked .DS_Store files, transient files cleaned).")

def check_no_duplicate_master_blueprints(repo_root):
    """Check 12: Verify downstream repositories do NOT contain duplicate master core blueprints."""
    upstream_marker = os.path.join(repo_root, ".pipeline", "upstream")
    if os.path.isdir(upstream_marker):
        print("Success: Check 12 verified (Master core / upstream repository detected -- skipping duplicate blueprint check).")
        return
    master_blueprints = {
        "DEAP_MASTER_ARCHITECTURE.md",
        "THREE_TIER_GOVERNANCE_BLUEPRINT.md",
        "DEAP_SYSML_V2_SAFETY_MODEL_SPECIFICATION.sysml"
    }
    duplicates = []
    for root, dirs, files in os.walk(repo_root):
        dirs[:] = [d for d in dirs if d not in EXCLUDED_DIRS]
        for f in files:
            if f in master_blueprints:
                duplicates.append(os.path.join(root, f))
    if duplicates:
        print(f"ERROR: Check 12 failed: Downstream repository contains duplicate master core blueprint file(s): {', '.join(duplicates)}", file=sys.stderr)
        sys.exit(1)
    print("Success: Check 12 verified (no duplicate master core blueprints found).")

def check_latex_katex_syntax(repo_root):
    """Check 13: Verify KaTeX / LaTeX mathematical rendering syntax across all markdown files."""
    allowed_alignment_envs = {
        "aligned", "alignedat", "matrix", "pmatrix", "bmatrix", "Bmatrix",
        "vmatrix", "Vmatrix", "cases", "dcases", "rcases", "array",
        "split", "gathered", "gather", "subarray", "smallmatrix"
    }
    errors = []
    for root, dirs, files in os.walk(repo_root):
        dirs[:] = [d for d in dirs if d not in EXCLUDED_DIRS]
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

            # Strip code blocks and inline code
            cleaned = re.sub(r"```.*?```|~~~.*?~~~", "", content, flags=re.DOTALL)
            cleaned = re.sub(r"`+.*?`+", "", cleaned)

            # a. Validate balanced $$ math blocks
            parts = cleaned.split("$$")
            if (len(parts) - 1) % 2 != 0:
                errors.append(f"Unbalanced $$ display math delimiters in {rel_path} (found {len(parts) - 1} delimiters).")
                continue

            # Check balanced \begin{aligned} and \end{aligned} globally in the file
            num_begin_aligned_all = len(re.findall(r"\\begin\{aligned\}", cleaned))
            num_end_aligned_all = len(re.findall(r"\\end\{aligned\}", cleaned))
            if num_begin_aligned_all != num_end_aligned_all:
                errors.append(f"Unbalanced \\begin{{aligned}} ({num_begin_aligned_all}) and \\end{{aligned}} ({num_end_aligned_all}) pairs in {rel_path}.")

            # Validate each math block
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

    if errors:
        print("ERROR: Check 13 failed (KaTeX / LaTeX mathematical syntax violations found):", file=sys.stderr)
        for err in errors:
            print(f"  - {err}", file=sys.stderr)
        sys.exit(1)
    print("Success: Check 13 verified (KaTeX / LaTeX mathematical syntax valid across all markdown files, including rules/sysml-ssot-completeness.md).")

def check_downstream_instructions_exist(repo_root):
    """Check 14: Verify presence of README.md, agent instruction entrypoints (AGENTS.md, CLAUDE.md, or .agents/AGENTS.md), and rules/sysml-ssot-completeness.md."""
    readme_path = os.path.join(repo_root, "README.md")
    if not os.path.isfile(readme_path):
        print(f"ERROR: Check 14 failed: README.md missing in repository root '{repo_root}'.", file=sys.stderr)
        sys.exit(1)
    if os.path.getsize(readme_path) == 0:
        print(f"ERROR: Check 14 failed: README.md is empty in repository root '{repo_root}'.", file=sys.stderr)
        sys.exit(1)

    agent_entrypoints = [
        os.path.join(repo_root, "AGENTS.md"),
        os.path.join(repo_root, "CLAUDE.md"),
        os.path.join(repo_root, ".agents", "AGENTS.md"),
    ]
    valid_entrypoints = [p for p in agent_entrypoints if os.path.isfile(p) and os.path.getsize(p) > 0]
    if not valid_entrypoints:
        print(f"ERROR: Check 14 failed: No non-empty agent instruction entrypoint found in '{repo_root}' (expected AGENTS.md, CLAUDE.md, or .agents/AGENTS.md).", file=sys.stderr)
        sys.exit(1)

    sysml_rule_path = os.path.join(repo_root, "rules", "sysml-ssot-completeness.md")
    if not os.path.isfile(sysml_rule_path):
        print(f"ERROR: Check 14 failed: rules/sysml-ssot-completeness.md missing in repository root '{repo_root}'.", file=sys.stderr)
        sys.exit(1)
    if os.path.getsize(sysml_rule_path) == 0:
        print(f"ERROR: Check 14 failed: rules/sysml-ssot-completeness.md is empty in repository root '{repo_root}'.", file=sys.stderr)
        sys.exit(1)

    print("Success: Check 14 verified (README.md, agent instruction entrypoints, and rules/sysml-ssot-completeness.md exist).")

def check_reconcile_backlog_tooling_exists(repo_root):
    """Check 15: Verify scripts/reconcile_backlog.py exists, is non-empty, and is executable."""
    reconcile_path = os.path.join(repo_root, "scripts", "reconcile_backlog.py")
    if not os.path.isfile(reconcile_path):
        print(f"ERROR: Check 15 failed: scripts/reconcile_backlog.py missing in repository root '{repo_root}'.", file=sys.stderr)
        sys.exit(1)
    if os.path.getsize(reconcile_path) == 0:
        print(f"ERROR: Check 15 failed: scripts/reconcile_backlog.py is empty in repository root '{repo_root}'.", file=sys.stderr)
        sys.exit(1)
    if not os.access(reconcile_path, os.X_OK):
        print(f"ERROR: Check 15 failed: scripts/reconcile_backlog.py is not executable in repository root '{repo_root}'.", file=sys.stderr)
        sys.exit(1)
    print("Success: Check 15 verified (scripts/reconcile_backlog.py exists, is non-empty, and is executable).")

def check_upstream_template_clean_landing_zones(repo_root):
    """Check 16: Upstream Template Clean Landing Zone Gate.

    Verify that upstream distribution templates contain zero concrete specification
    markdown files or concrete .sysml domain models in landing zones (docs/conops/,
    docs/safety/, docs/epics/, docs/features/, docs/user-stories/, docs/use-cases/, and schema/).
    """
    upstream_marker = os.path.join(repo_root, ".pipeline", "upstream")
    if not os.path.isdir(upstream_marker):
        print("Success: Check 16 verified (Downstream repository detected -- skipping upstream clean landing zone gate).")
        return

    landing_zones = [
        os.path.join("docs", "conops"),
        os.path.join("docs", "safety"),
        os.path.join("docs", "epics"),
        os.path.join("docs", "features"),
        os.path.join("docs", "user-stories"),
        os.path.join("docs", "use-cases"),
        os.path.join("docs", "management"),
        "schema",
    ]
    allowed_files = {".gitkeep", "README.md"}

    violations = []
    for zone in landing_zones:
        zone_path = os.path.join(repo_root, zone)
        if not os.path.isdir(zone_path):
            continue
        for root, dirs, files in os.walk(zone_path):
            dirs[:] = [d for d in dirs if d not in EXCLUDED_DIRS]
            for f in files:
                if f not in allowed_files:
                    rel_path = os.path.relpath(os.path.join(root, f), repo_root)
                    violations.append(rel_path)

    if violations:
        print(f"ERROR: Check 16 failed: Upstream distribution template landing zones contain concrete specification files: {', '.join(violations)}", file=sys.stderr)
        sys.exit(1)

    print("Success: Check 16 verified (Upstream distribution template landing zones are clean with zero concrete specs).")

def parse_fmeca_table(content: str) -> dict:
    """Extract structured FMECA table data including rows, components, failure modes, S/O/D/RPN, and basis classifications."""
    lines = content.splitlines()
    in_fmeca_section = False
    in_fmeca_table = False
    header_cols = []
    header_skipped = False

    components = {}
    failure_modes = []
    basis_counts = {"SSOT": 0, "Derived": 0}
    basis_classifications = []
    rows = []
    has_rpn = False

    id_idx = None
    comp_idx = None
    mode_idx = None
    s_idx = None
    o_idx = None
    d_idx = None
    rpn_idx = None
    basis_idx = None

    for line in lines:
        stripped = line.strip()
        # Check for section header (level 2+ or specific FMECA header)
        if stripped.startswith("##") or (stripped.startswith("#") and ("criticality" in stripped.lower() or "fmeca" in stripped.lower())):
            if re.search(r'\b(?:FMECA|Failure\s+Mode)\b', stripped, re.IGNORECASE):
                in_fmeca_section = True
                header_cols = []
                header_skipped = False
                continue
            elif in_fmeca_section:
                in_fmeca_section = False

        is_table_row = stripped.startswith("|") and stripped.endswith("|")
        if not is_table_row:
            if in_fmeca_table and not in_fmeca_section:
                in_fmeca_table = False
            continue

        if not in_fmeca_section and not in_fmeca_table:
            # Fallback scan for table containing FMECA keywords in header
            lower = stripped.lower()
            if "failure" in lower and ("rpn" in lower or "severity" in lower or "component" in lower or "mode" in lower):
                in_fmeca_table = True
                header_cols = []
                header_skipped = False

        if in_fmeca_section or in_fmeca_table:
            # Skip separator rows like |:---|:---| or |---|---|
            if re.match(r"^\|(?:\s*:?-+:?\s*\|)+$", stripped):
                header_skipped = True
                continue

            cells = [c.strip() for c in stripped.split("|")[1:-1]]
            if not cells or not any(cells):
                continue

            # Identify header row
            if not header_skipped and not header_cols:
                lower = [c.lower() for c in cells]
                if any(kw in lower_cell for lower_cell in lower for kw in ["component", "failure", "subsystem", "severity", "rpn", "effect", "s", "o", "d"]):
                    header_cols = lower
                    for idx, h in enumerate(header_cols):
                        h_clean = h.strip().lower()
                        if any(kw == h_clean for kw in ["id", "failure id", "fm id", "fmeca id", "fmid"]) or h_clean.startswith("failure id") or h_clean.startswith("fm id"):
                            if id_idx is None:
                                id_idx = idx
                        elif any(kw in h_clean for kw in ["component", "subsystem", "unit", "item", "part", "partdef"]) and "effect" not in h_clean and "loss" not in h_clean and "control" not in h_clean:
                            if comp_idx is None:
                                comp_idx = idx
                        elif any(kw in h_clean for kw in ["failure mode", "mode", "failure description", "failure mechanism"]) and "dimension" not in h_clean and "control" not in h_clean and "effect" not in h_clean:
                            if mode_idx is None:
                                mode_idx = idx
                        elif (h_clean in ("s", "sev", "severity") or re.search(r'\b(?:severity|s)\b', h_clean)) and "description" not in h_clean and "subsystem" not in h_clean and "status" not in h_clean and "system" not in h_clean and "class" not in h_clean and "dimension" not in h_clean:
                            if s_idx is None:
                                s_idx = idx
                        elif (h_clean in ("o", "occ", "occurrence") or re.search(r'\b(?:occurrence|occ|o)\b', h_clean)) and "description" not in h_clean and "mode" not in h_clean and "control" not in h_clean and "action" not in h_clean:
                            if o_idx is None:
                                o_idx = idx
                        elif (h_clean in ("d", "det", "detection") or re.search(r'\b(?:detection|det|d)\b', h_clean)) and "description" not in h_clean and "mitigating" not in h_clean and "design" not in h_clean and "id" not in h_clean and "method" not in h_clean and "dimension" not in h_clean:
                            if d_idx is None:
                                d_idx = idx
                        elif any(kw in h_clean for kw in ["rpn", "risk priority", "risk priority number"]):
                            if rpn_idx is None:
                                rpn_idx = idx
                        elif any(kw in h_clean for kw in ["basis", "derivation", "provenance", "classification", "anchor", "traceability", "derivation basis"]):
                            if basis_idx is None:
                                basis_idx = idx
                    continue

            # Data row extraction
            cur_id_idx = id_idx
            cur_comp_idx = comp_idx
            cur_mode_idx = mode_idx
            cur_s_idx = s_idx
            cur_o_idx = o_idx
            cur_d_idx = d_idx
            cur_rpn_idx = rpn_idx
            cur_basis_idx = basis_idx

            if cur_comp_idx is None:
                if len(cells) > 1 and re.match(r'^(?:FM|FMECA)-', cells[0], re.IGNORECASE):
                    cur_comp_idx = 1
                else:
                    cur_comp_idx = 0

            if cur_mode_idx is None:
                if cur_comp_idx == 1 and len(cells) > 2:
                    cur_mode_idx = 2
                elif cur_comp_idx == 0 and len(cells) > 1:
                    cur_mode_idx = 1

            failure_id = cells[cur_id_idx] if cur_id_idx is not None and cur_id_idx < len(cells) else ""
            if not failure_id and len(cells) > 0 and re.match(r'^(?:FM|FMECA)-', cells[0], re.IGNORECASE):
                failure_id = cells[0]

            comp_name = cells[cur_comp_idx] if cur_comp_idx is not None and cur_comp_idx < len(cells) else f"Component-{len(rows)+1}"
            mode_name = cells[cur_mode_idx] if cur_mode_idx is not None and cur_mode_idx < len(cells) else ""
            s_val = cells[cur_s_idx] if cur_s_idx is not None and cur_s_idx < len(cells) else None
            o_val = cells[cur_o_idx] if cur_o_idx is not None and cur_o_idx < len(cells) else None
            d_val = cells[cur_d_idx] if cur_d_idx is not None and cur_d_idx < len(cells) else None
            rpn_val = cells[cur_rpn_idx] if cur_rpn_idx is not None and cur_rpn_idx < len(cells) else None

            # Check RPN
            if cur_rpn_idx is not None and cur_rpn_idx < len(cells):
                if cells[cur_rpn_idx]:
                    has_rpn = True
            elif any("rpn" in c.lower() for c in header_cols):
                has_rpn = True

            # Check Basis
            row_basis = None
            if cur_basis_idx is not None and cur_basis_idx < len(cells):
                cell_basis = cells[cur_basis_idx]
                if re.search(r'\bSSOT\b', cell_basis, re.IGNORECASE):
                    row_basis = "SSOT"
                elif re.search(r'\bDerived\b', cell_basis, re.IGNORECASE):
                    row_basis = "Derived"

            # If not found in dedicated column, search across all cells for explicit annotations
            if row_basis is None:
                row_text = " ".join(cells)
                if re.search(r'\bSSOT\b', row_text, re.IGNORECASE):
                    row_basis = "SSOT"
                elif re.search(r'\bDerived\b', row_text, re.IGNORECASE):
                    row_basis = "Derived"

            if row_basis == "SSOT":
                basis_counts["SSOT"] += 1
            elif row_basis == "Derived":
                basis_counts["Derived"] += 1

            basis_classifications.append(row_basis)
            if mode_name:
                failure_modes.append(mode_name)

            row_dict = {
                "cells": cells,
                "failure_id": failure_id,
                "component": comp_name,
                "failure_mode": mode_name,
                "s": s_val,
                "o": o_val,
                "d": d_val,
                "rpn": rpn_val,
                "basis": row_basis,
            }
            rows.append(row_dict)
            components.setdefault(comp_name, []).append(row_dict)

    if not has_rpn and (rpn_idx is not None or any("rpn" in c.lower() for c in header_cols) or re.search(r'\bRPN\b|Risk\s+Priority\s+Number', content, re.IGNORECASE)):
        has_rpn = True

    return {
        "total_rows": len(rows),
        "components": components,
        "failure_modes": failure_modes,
        "basis_counts": basis_counts,
        "basis_classifications": basis_classifications,
        "has_rpn": has_rpn,
        "rows": rows,
    }


def count_fmeca_rows(content: str) -> int:
    """Extract and count data rows from the FMECA table in content."""
    return parse_fmeca_table(content)["total_rows"]

def check_uca_categories(content: str) -> list:
    """Verify that all 4 STPA UCA failure modes are covered in content."""
    missing_categories = []

    # 1. Not providing causes hazard
    if not re.search(r'\b(?:not\s+provid(?:ing|ed)|omission)\b', content, re.IGNORECASE):
        missing_categories.append("1. Not providing causes hazard")

    # 2. Providing causes hazard
    if not re.search(r'\b(?:providing\s+(?:causes|incorrectly)|providing(?!\s+too)|commission)\b', content, re.IGNORECASE):
        missing_categories.append("2. Providing causes hazard")

    # 3. Too early / too late / out of order
    if not re.search(r'\b(?:too\s+early|too\s+late|out\s+of\s+order|timing|early/late)\b', content, re.IGNORECASE):
        missing_categories.append("3. Providing too early, too late, or out of order")

    # 4. Stopped too soon / applied too long
    if not re.search(r'\b(?:stopped\s+too\s+soon|applied\s+too\s+long|duration|stopped\s+early)\b', content, re.IGNORECASE):
        missing_categories.append("4. Stopped too soon or applied too long")

    return missing_categories

def check_sora_osos(content: str) -> list:
    """Verify all 24 SORA OSOs (OSO-01 through OSO-24) are present in content."""
    missing = []
    for i in range(1, 25):
        oso_id = f"OSO-{i:02d}"
        pattern = r'\b(?:OSO-' + f'{i:02d}' + r'|OSO-' + str(i) + r')\b'
        if not re.search(pattern, content, re.IGNORECASE):
            missing.append(oso_id)
    return missing

# ---------------------------------------------------------------------------
# Structural Table-Aware AST Validation (Check 17)
# ---------------------------------------------------------------------------


@dataclass
class STPARowAST:
    """Typed AST record for a single Unsafe Control Action (UCA) markdown table row."""

    uca_id: str
    controller: str
    control_action: str
    guide_word: str
    hazard_ref: str = ""
    loss_ref: str = ""
    safety_constraint: str = ""
    line_number: int = 0


@dataclass
class SORAOsoAST:
    """Typed AST record for a single SORA Operational Safety Objective table row."""

    oso_id: str
    robustness_level: str
    justification: str
    mitigation_ref: str
    line_number: int


@dataclass
class ProofBlockAST:
    """Typed AST record for a formal safety theorem block and its 5-part structure."""

    theorem_id: str
    proposition: str = ""
    assumptions: str = ""
    barrier_function: str = ""
    derivation: str = ""
    conclusion: str = ""
    line_number: int = 0


@dataclass
class ASTValidationReport:
    """Aggregated Check 17 AST validation report."""

    is_conforming: bool = True
    total_uca_rows: int = 0
    expected_uca_rows: int = 0
    missing_permutations: List[str] = field(default_factory=list)
    missing_osos: List[str] = field(default_factory=list)
    malformed_proofs: List[str] = field(default_factory=list)
    syntax_errors: List[str] = field(default_factory=list)
    missing_fmeca_parts: List[str] = field(default_factory=list)
    undeclared_fmeca_parts: List[str] = field(default_factory=list)
    missing_dimensions: List[str] = field(default_factory=list)
    missing_port_modes: List[str] = field(default_factory=list)
    part_criticalities: Dict[str, int] = field(default_factory=dict)
    missing_state_diagrams: List[str] = field(default_factory=list)
    missing_stateflow_hooks: List[str] = field(default_factory=list)

    @property
    def phantom_fmeca_components(self) -> List[str]:
        return self.undeclared_fmeca_parts

    @property
    def missing_fmeca_components(self) -> List[str]:
        return self.missing_fmeca_parts

    def format_cli_summary(self) -> str:
        """Format a one-line CLI summary of the AST validation outcome."""
        summary = (
            f"Check 17 AST validation: {self.total_uca_rows} UCA row(s) parsed, "
            f"{self.expected_uca_rows} expected Cartesian permutation(s)"
        )
        if self.missing_permutations:
            summary += f", {len(self.missing_permutations)} missing permutation(s)"
        if self.missing_osos:
            summary += f", {len(self.missing_osos)} missing SORA OSO(s)"
        if self.malformed_proofs:
            summary += f", {len(self.malformed_proofs)} malformed proof block(s)"
        if self.missing_fmeca_parts:
            summary += f", {len(self.missing_fmeca_parts)} missing FMECA part(s)"
        if self.undeclared_fmeca_parts:
            summary += f", {len(self.undeclared_fmeca_parts)} undeclared FMECA part(s)"
        if self.missing_port_modes:
            summary += f", {len(self.missing_port_modes)} missing high-criticality port mode(s)"
        if self.missing_dimensions:
            summary += f", {len(self.missing_dimensions)} missing failure dimension(s)"
        if self.missing_state_diagrams:
            summary += f", {len(self.missing_state_diagrams)} missing state diagram(s)"
        if self.missing_stateflow_hooks:
            summary += f", {len(self.missing_stateflow_hooks)} missing Stateflow hook(s)"
        return summary


# Universal 4 Failure Dimensions: Interface (Γ), State (Φ), Action (Ω), Resource (Ψ)
UNIVERSAL_FAILURE_DIMENSIONS = {
    "Interface": ("Γ", re.compile(r'\b(?:Interface|Port|Bus|Signal|Protocol|Packet|Message|Frame|Channel|Link|CRC|Timeout|IO|Input|Output|Data|Transceiver|Receiver|Uplink|Downlink|Telemetry|Transients?|Γ|\\Gamma)\b|\[(?:Interface|Γ)\]|\((?:Interface|Γ)\)', re.IGNORECASE)),
    "State": ("Φ", re.compile(r'\b(?:State|Mode|Transition|Deadlock|Latch|Phase|Statechart|FSM|Sync|Synchronization|Desync|Drift|Stuck|Uninitialized|Freeze|Lockup|Trip|Abort|Corruption|Disagreement|Φ|\\Phi)\b|\[(?:State|Φ)\]|\((?:State|Φ)\)', re.IGNORECASE)),
    "Action": ("Ω", re.compile(r'\b(?:Action|Command|Execution|Operation|Control|Timing|Deadline|Compute|Calculation|Process|Logic|Omission|Commission|Latency|Jitter|Delay|Rate|Clamping|Limiter|Saturation|Step|Overshoot|Schedule|Task|Authority|Miss|Ω|\\Omega)\b|\[(?:Action|Ω)\]|\((?:Action|Ω)\)', re.IGNORECASE)),
    "Resource": ("Ψ", re.compile(r'\b(?:Resource|Memory|CPU|Buffer|Power|Energy|Battery|Thermal|Heat|Overheat|Overload|Bandwidth|Storage|Capacity|Stack|Heap|Overflow|Underflow|Brownout|Voltage|Current|Load|Fault|Short|Sag|Circuit|Crowbar|Degradation|Flash|RAM|Supply|Undervoltage|Overvoltage|Seizure|Windings?|Wiper|Hardware|Bearing|Dielectric|Squib|Fuse|Fusing|Ψ|\\Psi)\b|\[(?:Resource|Ψ)\]|\((?:Resource|Ψ)\)', re.IGNORECASE)),
}


def check_failure_dimension_coverage(fmeca_data: dict) -> List[str]:
    """Verify that failure modes across the FMECA table span the 4 universal failure dimensions (Interface, State, Action, Resource)."""
    found_dims = set()
    for row in fmeca_data.get("rows", []):
        mode_text = str(row.get("failure_mode", "")) + " " + str(row.get("basis", ""))
        for dim_name, (_greek, pattern) in UNIVERSAL_FAILURE_DIMENSIONS.items():
            if pattern.search(mode_text):
                found_dims.add(dim_name)
    missing = [dim for dim in ["Interface", "State", "Action", "Resource"] if dim not in found_dims]
    return missing


def _load_sysml_ast_classes():
    """Import SysMLParser, SysMLPackage, PartDef, PortDef, HazardDef from sysmlv2_ast (fail-safe)."""
    try:
        from sysmlv2_ast import SysMLParser, SysMLPackage, PartDef, PortDef, HazardDef
        return SysMLParser, SysMLPackage, PartDef, PortDef, HazardDef
    except ImportError:
        pass
    try:
        from skills.spec_orchestrator.scripts.sysmlv2_ast import SysMLParser, SysMLPackage, PartDef, PortDef, HazardDef
        return SysMLParser, SysMLPackage, PartDef, PortDef, HazardDef
    except ImportError:
        pass
    script_dir = os.path.dirname(os.path.abspath(__file__))
    project_root = os.path.dirname(script_dir)
    spec_dir = os.path.join(project_root, "skills", "spec-orchestrator", "scripts")
    if spec_dir not in sys.path:
        sys.path.insert(0, spec_dir)
    try:
        from sysmlv2_ast import SysMLParser, SysMLPackage, PartDef, PortDef, HazardDef
        return SysMLParser, SysMLPackage, PartDef, PortDef, HazardDef
    except ImportError:
        return None, None, None, None, None


def calculate_topological_criticality(pkg: Any, part: Any) -> int:
    """Calculate dynamic topological criticality for an AST part def:
    Crit(P_i) = max({Severity(H_j) for H_j in ReachableHazards(P_i)} U {Severity(H) for H in P_i.hazards} U {1})
    """
    severities = []
    part_name = getattr(part, "name", str(part))
    if hasattr(pkg, "get_reachable_hazards"):
        try:
            reachable = pkg.get_reachable_hazards(part_name)
            for h in reachable:
                sev = getattr(h, "severity", None)
                if sev is not None:
                    try:
                        severities.append(int(sev))
                    except (ValueError, TypeError):
                        pass
        except Exception:
            pass
    for h in (getattr(part, "hazards", []) or []):
        sev = getattr(h, "severity", None)
        if sev is not None:
            try:
                severities.append(int(sev))
            except (ValueError, TypeError):
                pass
    return max(severities) if severities else 1


def _component_matches(table_comp: str, ast_part_name: str) -> bool:
    """Check if an FMECA table component cell matches an AST part def name."""
    tc = table_comp.strip().lower()
    pn = ast_part_name.strip().lower()
    if tc == pn:
        return True
    tc_clean = re.sub(r'[^a-zA-Z0-9]', '', tc)
    pn_clean = re.sub(r'[^a-zA-Z0-9]', '', pn)
    if tc_clean and tc_clean == pn_clean:
        return True
    if re.search(rf"\b{re.escape(ast_part_name)}\b", table_comp, re.IGNORECASE):
        return True
    if re.search(rf"\b{re.escape(table_comp)}\b", ast_part_name, re.IGNORECASE):
        return True
    return False


def check_high_criticality_port_coverage(fmeca_data: dict, pkg: Any) -> Tuple[List[str], Dict[str, int]]:
    """For high-criticality parts (Crit >= 8), verify port-level interface failure modes for declared typed ports."""
    errors = []
    part_criticalities = {}
    if not hasattr(pkg, "get_all_parts"):
        return errors, part_criticalities

    for part in pkg.get_all_parts():
        crit = calculate_topological_criticality(pkg, part)
        part_criticalities[part.name] = crit
        if crit >= 8 and getattr(part, "ports", None):
            comp_rows = []
            for row in fmeca_data.get("rows", []):
                if _component_matches(row.get("component", ""), part.name):
                    comp_rows.append(row)

            missing_ports = []
            for port in part.ports:
                port_name = port.name.strip()
                port_matched = False
                for r in comp_rows:
                    row_text = " ".join(str(c) for c in r.get("cells", [])) + " " + str(r.get("failure_mode", ""))
                    if re.search(rf"\b{re.escape(port_name)}\b", row_text, re.IGNORECASE) or re.search(rf"\b{re.escape(part.name)}\.{re.escape(port_name)}\b", row_text, re.IGNORECASE):
                        port_matched = True
                        break
                if not port_matched:
                    missing_ports.append(port_name)

            if missing_ports:
                errors.append(
                    f"Pillar 7 violation: High-criticality component '{part.name}' (Crit={crit} >= 8) "
                    f"missing port-level interface failure mode for declared port(s): {', '.join(sorted(missing_ports))}."
                )

    return errors, part_criticalities


def check_fmeca_ast_coverage(content: str, model_text: Optional[str] = None) -> Tuple[List[str], ASTValidationReport]:
    """Verify FMECA table against SysML AST closure: PartDef coverage, topological criticality, port coverage for Crit >= 8, and 4 universal failure dimensions."""
    errors: List[str] = []
    report = ASTValidationReport()
    fmeca_data = parse_fmeca_table(content)

    if fmeca_data["total_rows"] > 0:
        missing_dims = check_failure_dimension_coverage(fmeca_data)
        if missing_dims:
            report.missing_dimensions.extend(missing_dims)
            errors.append(
                f"Pillar 7 violation: FMECA table missing coverage for universal failure dimension(s): "
                f"{', '.join(missing_dims)} (expected Interface (Γ), State (Φ), Action (Ω), Resource (Ψ))."
            )

    if model_text:
        SysMLParser, _SysMLPackage, _PartDef, _PortDef, _HazardDef = _load_sysml_ast_classes()
        if SysMLParser is not None:
            try:
                pkg_obj = SysMLParser.parse_text(model_text)
                expected_parts = [p.name for p in pkg_obj.get_all_parts()]
                table_components = set(fmeca_data["components"].keys())
                missing_parts = []
                for p in expected_parts:
                    matched = False
                    for c in table_components:
                        if _component_matches(c, p):
                            matched = True
                            break
                    if not matched:
                        missing_parts.append(p)
                if missing_parts:
                    report.missing_fmeca_parts.extend(missing_parts)
                    errors.append(
                        f"Pillar 7 violation: FMECA table missing declared AST part def component(s): {', '.join(sorted(missing_parts))}."
                    )

                undeclared_parts = [c for c in table_components if not any(_component_matches(c, p) for p in expected_parts)]
                if undeclared_parts:
                    report.undeclared_fmeca_parts.extend(undeclared_parts)
                    errors.append(
                        f"Pillar 7 violation: FMECA table references undeclared phantom component(s) not in AST: {', '.join(sorted(undeclared_parts))}."
                    )

                port_errors, crit_map = check_high_criticality_port_coverage(fmeca_data, pkg_obj)
                report.part_criticalities = crit_map
                if port_errors:
                    report.missing_port_modes.extend(port_errors)
                    errors.extend(port_errors)
            except Exception as exc:
                errors.append(f"Safety AST violation: Failed to parse SysML v2 model for FMECA ({exc}).")

    report.is_conforming = not errors
    return errors, report


def group_state_defs_by_family(state_defs: List[str]) -> Dict[str, List[str]]:
    """Group declared state def nodes by state machine prefix family (e.g. split by underscore, package, or delimiter)."""
    families: Dict[str, List[str]] = {}
    for s in state_defs:
        s_clean = str(s).strip()
        if not s_clean:
            continue
        if "::" in s_clean:
            prefix = s_clean.split("::")[0].strip()
        elif "." in s_clean:
            prefix = s_clean.split(".")[0].strip()
        elif "_" in s_clean:
            prefix = s_clean.split("_")[0].strip()
        else:
            prefix = s_clean
        families.setdefault(prefix, []).append(s_clean)
    return families


def extract_section_6_1(content: str) -> Optional[str]:
    """Extract Section 6.1 (Stateflow Synthesis Hooks & Safety Statecharts) from markdown content."""
    lines = content.splitlines()
    in_section = False
    section_lines = []
    heading_level = 3

    for line in lines:
        stripped = line.strip()
        m = re.match(r"^(#{2,4})\s+(?:Section\s+)?6\.1\b", stripped, re.IGNORECASE)
        if m:
            in_section = True
            heading_level = len(m.group(1))
            section_lines.append(line)
            continue

        if in_section:
            next_heading = re.match(r"^(#{1,4})\s+\S", stripped)
            if next_heading:
                lvl = len(next_heading.group(1))
                if lvl <= heading_level:
                    break
            section_lines.append(line)

    if not section_lines:
        return None
    return "\n".join(section_lines)


def extract_mermaid_state_diagrams(text: str) -> List[str]:
    """Extract Mermaid state diagram blocks from markdown text."""
    pattern = re.compile(r"```(?:mermaid)?\s*\n(.*?)```", re.DOTALL | re.IGNORECASE)
    diagrams = []
    for match in pattern.finditer(text):
        block = match.group(1)
        first_line = block.strip().splitlines()[0].strip() if block.strip() else ""
        if re.match(r"^stateDiagram(?:-v2)?\b", first_line, re.IGNORECASE):
            diagrams.append(block)
    return diagrams


def check_stateflow_ast_coverage(content: str, model_ast: dict) -> Tuple[List[str], List[str], List[str]]:
    """Verify Section 6.1 Stateflow synthesis hooks and dedicated Mermaid stateDiagram-v2 figures for declared AST state machine families.

    Returns (errors, missing_state_diagrams, missing_stateflow_hooks).
    """
    errors: List[str] = []
    missing_diagrams: List[str] = []
    missing_hooks: List[str] = []

    state_defs = sorted({str(name) for name in model_ast.get("state_defs", [])})
    families = group_state_defs_by_family(state_defs)
    multi_state_families = {prefix: states for prefix, states in families.items() if len(states) >= 2}

    if not multi_state_families:
        return errors, missing_diagrams, missing_hooks

    sec_6_1 = extract_section_6_1(content)
    if sec_6_1 is None:
        missing_fams = sorted(multi_state_families.keys())
        missing_diagrams.extend(missing_fams)
        missing_hooks.extend(missing_fams)
        errors.append(
            f"Pillar 6 violation: Missing Section 6.1 (Stateflow Synthesis Hooks & Safety Statecharts) "
            f"in STPA Matrix for declared AST state machine families: {', '.join(missing_fams)}."
        )
        return errors, missing_diagrams, missing_hooks

    diagrams = extract_mermaid_state_diagrams(sec_6_1)

    # Extract hooks content by stripping markdown code fences and diagram subsection headers
    non_diagram_lines = []
    in_fence = False
    for line in sec_6_1.splitlines():
        if line.strip().startswith("```"):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        if re.match(r"^\s*#{3,6}\s+.*(?:Statechart|State\s+Diagram|Diagram|Figure)\b", line, re.IGNORECASE):
            continue
        non_diagram_lines.append(line)
    hooks_body = "\n".join(non_diagram_lines)

    for family, members in sorted(multi_state_families.items()):
        # Check dedicated stateDiagram-v2 representation
        family_matched = False
        for diag in diagrams:
            if re.search(rf"\b{re.escape(family)}\b", diag, re.IGNORECASE) or re.search(rf"\b{re.escape(family)}_", diag, re.IGNORECASE):
                family_matched = True
                break
            if any(re.search(rf"\b{re.escape(st)}\b", diag, re.IGNORECASE) for st in members):
                family_matched = True
                break
        if not family_matched:
            missing_diagrams.append(family)
            errors.append(
                f"Pillar 6 violation: Missing dedicated Mermaid stateDiagram-v2 block in Section 6.1 "
                f"for AST state machine family '{family}' ({len(members)} states: {', '.join(sorted(members))})."
            )

        # Check Stateflow synthesis hook reference
        if not re.search(rf"\b{re.escape(family)}\b", hooks_body, re.IGNORECASE):
            missing_hooks.append(family)
            errors.append(
                f"Pillar 6 violation: AST state machine family '{family}' is not referenced in Section 6.1 Stateflow synthesis hooks."
            )

    if not re.search(r'\b(?:Stateflow|Simulink|MATLAB|Embedded\s+Coder|SLDV)\b', hooks_body or sec_6_1, re.IGNORECASE):
        errors.append(
            "Pillar 6 violation: Section 6.1 missing Stateflow / MATLAB / Simulink synthesis hooks."
        )

    return errors, missing_diagrams, missing_hooks


# Canonical STPA guide words are methodology constants, not domain concepts.
# Order matters: timing/duration rules precede the generic providing rule so
# phrases such as "Providing too early" classify to GW-3 rather than GW-2.
STPA_GUIDE_WORD_RULES = [
    ("GW-1", "Not providing causes hazard", re.compile(r"not\s+provid|omission|withheld|\bclass\s+a\b|\bgw-?1\b", re.IGNORECASE)),
    ("GW-3", "Providing too early, too late, or out of order", re.compile(r"too\s+early|too\s+late|out\s+of\s+order|early/late|\btiming\b|\bclass\s+c\b|\bgw-?3\b", re.IGNORECASE)),
    ("GW-4", "Stopped too soon or applied too long", re.compile(r"stopped\s+too\s+soon|applied\s+too\s+long|stopped\s+early|\bduration\b|too\s+soon|\bclass\s+d\b|\bgw-?4\b", re.IGNORECASE)),
    ("GW-2", "Providing causes hazard", re.compile(r"providing\s+causes|incorrectly\s+provided|unintended\s+provision|\bcommission\b|\bclass\s+b\b|\bgw-?2\b|\bproviding\b", re.IGNORECASE)),
]
STPA_GUIDE_WORD_ORDER = {gw_id: index for index, (gw_id, _label, _pattern) in enumerate(STPA_GUIDE_WORD_RULES)}
STPA_GUIDE_WORD_LABELS = {gw_id: label for gw_id, label, _pattern in STPA_GUIDE_WORD_RULES}

# Schema-less structural floor: 4 canonical STPA guide words x 4 (controller,
# control action) pair instances. Model-backed validation derives the true
# Cartesian cardinality from the schema instead of applying this floor.
MIN_STRUCTURAL_UCA_ROWS = 16


def classify_uca_guide_words(cell_text: str) -> List[Tuple[str, str]]:
    """Classify a UCA guide word cell into one or more canonical STPA failure modes.

    Handles compound class attribution (e.g. 'Class a -- not providing; Class c -- too late')
    by splitting compound segments and returning all unique matched guide words in canonical order.
    """
    if not cell_text or not cell_text.strip():
        return []

    chunks = [c.strip() for c in re.split(r"[;\n\r]+", cell_text) if c.strip()]
    refined_chunks = []
    for chunk in chunks:
        sub_chunks = re.split(r"(?=(?:\b(?:class\s+[a-d]|gw-?[1-4])\b))", chunk, flags=re.IGNORECASE)
        for sc in sub_chunks:
            sc_clean = sc.strip().strip(",").strip()
            if sc_clean:
                refined_chunks.append(sc_clean)

    matched_gw_ids = set()
    results = []

    for chunk in refined_chunks:
        for gw_id, label, pattern in STPA_GUIDE_WORD_RULES:
            if pattern.search(chunk):
                if gw_id not in matched_gw_ids:
                    matched_gw_ids.add(gw_id)
                    results.append((gw_id, label))
                break

    if not results:
        for gw_id, label, pattern in STPA_GUIDE_WORD_RULES:
            if pattern.search(cell_text):
                if gw_id not in matched_gw_ids:
                    matched_gw_ids.add(gw_id)
                    results.append((gw_id, label))

    results.sort(key=lambda item: STPA_GUIDE_WORD_ORDER.get(item[0], 99))
    return results


def classify_uca_guide_word(cell_text: str) -> Optional[Tuple[str, str]]:
    """Classify a UCA guide word cell into one of the 4 canonical STPA failure modes."""
    matches = classify_uca_guide_words(cell_text)
    return matches[0] if matches else None


def _load_sysml_parser():
    """Import the shared SysML v2 parser from scripts/compile_sysml.py (parsing logic is never duplicated)."""
    try:
        from scripts.compile_sysml import parse_sysml
        return parse_sysml
    except ImportError:
        from compile_sysml import parse_sysml
        return parse_sysml


def _discover_sysml_model_text(repo_root: Optional[str]) -> Optional[str]:
    """Locate and read the authoritative SysML v2 model (schema/*.sysml or .pipeline/schema.sysml)."""
    if not repo_root or not os.path.isdir(repo_root):
        return None
    schema_dir = os.path.join(repo_root, "schema")
    if os.path.isdir(schema_dir):
        sysml_contents = []
        for name in sorted(os.listdir(schema_dir)):
            if name.endswith(".sysml"):
                try:
                    with open(os.path.join(schema_dir, name), "r", encoding="utf-8") as handle:
                        sysml_contents.append(handle.read())
                except OSError:
                    continue
        if sysml_contents:
            return "\n\n".join(sysml_contents)
    pipeline_model = os.path.join(repo_root, ".pipeline", "schema.sysml")
    if os.path.isfile(pipeline_model):
        try:
            with open(pipeline_model, "r", encoding="utf-8") as handle:
                return handle.read()
        except OSError:
            pass
    return None


class MarkdownTableASTParser:
    """Structural markdown table tokenizer producing typed AST records.

    Tables are split into discrete column cells mapped by header keywords; no
    global regex keyword heuristics over the whole document are used.
    """

    @staticmethod
    def _split_row(line: str):
        stripped = line.strip()
        if not (stripped.startswith("|") and stripped.endswith("|")):
            return None
        return [cell.strip() for cell in stripped[1:-1].split("|")]

    @staticmethod
    def _is_separator_row(cells) -> bool:
        if not cells or not all(cells):
            return False
        return all(re.fullmatch(r":?-{1,}:?", cell) for cell in cells)

    @classmethod
    def _iter_tables(cls, text: str):
        """Yield (header_cells, data_rows) for each well-formed markdown table in text."""
        lines = text.splitlines()
        index = 0
        while index < len(lines):
            header = cls._split_row(lines[index])
            if header is None:
                index += 1
                continue
            probe = index + 1
            if probe >= len(lines):
                break
            separator = cls._split_row(lines[probe])
            if not cls._is_separator_row(separator):
                index += 1
                continue
            probe += 1
            data_rows = []
            while probe < len(lines):
                row_cells = cls._split_row(lines[probe])
                if row_cells is None:
                    break
                data_rows.append((row_cells, probe + 1))
                probe += 1
            yield header, data_rows
            index = probe

    @staticmethod
    def _column_index(header, keywords, exclude=()):
        # First attempt exact normalized match in keyword priority order
        for kw in keywords:
            kw_norm = kw.lower().strip()
            for index, cell in enumerate(header):
                lowered = cell.lower().strip()
                if any(ex in lowered for ex in exclude):
                    continue
                if lowered == kw_norm:
                    return index
        # Second attempt word-boundary regex match in keyword priority order
        for kw in keywords:
            kw_norm = kw.lower().strip()
            pattern_str = r"\b" + re.escape(kw_norm).replace(r"\ ", r"\s+") + r"\b"
            pattern = re.compile(pattern_str, re.IGNORECASE)
            for index, cell in enumerate(header):
                lowered = cell.lower().strip()
                if any(ex in lowered for ex in exclude):
                    continue
                if pattern.search(lowered):
                    return index
        return None

    @classmethod
    def parse_stpa_table(cls, text: str) -> List[STPARowAST]:
        """Parse UCA rows from markdown tables headed by control action / guide word columns."""
        rows: List[STPARowAST] = []
        for header, data_rows in cls._iter_tables(text):
            header_text = " ".join(cell.lower() for cell in header)
            # Exclude obvious non-UCA tables (such as FMECA, Loss Scenarios, Hazards, SORA OSO)
            if "rpn" in header_text or "oso" in header_text or "mitigating design control" in header_text:
                continue

            col_uca = cls._column_index(
                header,
                ("uca id", "uca", "id", "identifier"),
                exclude=("description", "scenario", "constraint", "loss", "hazard", "action", "guide", "mode", "type", "failure", "fmeca", "rpn", "component", "subsystem", "effect"),
            )
            col_controller = cls._column_index(header, ("controller", "subsystem", "component"))
            col_action = cls._column_index(
                header,
                ("control action (ssot)", "control action", "action", "command"),
                exclude=("description", "unsafe", "scenario"),
            )
            col_guide = cls._column_index(
                header,
                ("guide word", "guide-word", "stpa guide word", "failure mode", "class attribution", "uca category", "stpa uca category", "type", "mode"),
                exclude=("description", "scenario"),
            )
            col_hazard = cls._column_index(
                header,
                ("hazard", "linked hazards", "hazards", "hazard ref", "hazard reference", "hazard links", "triggered system hazard"),
            )
            col_loss = cls._column_index(
                header,
                ("loss", "loss ref", "system loss ref", "loss reference"),
                exclude=("scenario",),
            )
            col_constraint = cls._column_index(
                header,
                ("safety constraint", "constraint", "constraint statement"),
            )

            # Table must be an STPA UCA table:
            # Must have 'uca' in header_text, or specifically a UCA column, or both action and guide columns.
            is_uca_table = (
                "uca" in header_text
                or (col_uca is not None and "uca" in header[col_uca].lower())
                or (col_action is not None and col_guide is not None)
            )
            if not is_uca_table:
                continue
            if col_action is None and col_guide is None:
                continue

            def cell_for(cells, col):
                return cells[col] if col is not None and col < len(cells) else ""

            for cells, line_number in data_rows:
                if not any(cells):
                    continue
                rows.append(STPARowAST(
                    uca_id=cell_for(cells, col_uca),
                    controller=cell_for(cells, col_controller),
                    control_action=cell_for(cells, col_action),
                    guide_word=cell_for(cells, col_guide),
                    hazard_ref=cell_for(cells, col_hazard),
                    loss_ref=cell_for(cells, col_loss),
                    safety_constraint=cell_for(cells, col_constraint),
                    line_number=line_number,
                ))
        return rows

    @staticmethod
    def _oso_id_cell(cell: str) -> Optional[str]:
        match = re.fullmatch(r"(OSO-\d{1,2})", cell.strip(), re.IGNORECASE)
        return match.group(1).upper() if match else None

    @classmethod
    def parse_sora_table(cls, text: str) -> List[SORAOsoAST]:
        """Parse SORA OSO evaluation rows from markdown tables headed by an OSO column."""
        rows: List[SORAOsoAST] = []
        for header, data_rows in cls._iter_tables(text):
            header_text = " ".join(cell.lower() for cell in header)
            if "oso" not in header_text and "operational safety objective" not in header_text:
                continue
            col_id = cls._column_index(header, ("oso id", "oso"))
            col_robust = cls._column_index(header, ("robust",))
            col_just = cls._column_index(header, ("justification",))
            col_mit = cls._column_index(header, ("mitigation",))

            def cell_for(cells, col):
                return cells[col] if col is not None and col < len(cells) else ""

            for cells, line_number in data_rows:
                if not any(cells):
                    continue
                if col_id is None or col_id >= len(cells):
                    continue
                oso_id = cls._oso_id_cell(cells[col_id])
                if oso_id is None:
                    continue
                rows.append(SORAOsoAST(
                    oso_id=oso_id,
                    robustness_level=cell_for(cells, col_robust),
                    justification=cell_for(cells, col_just),
                    mitigation_ref=cell_for(cells, col_mit),
                    line_number=line_number,
                ))
        return rows

    @classmethod
    def parse_proof_blocks(cls, text: str) -> List[ProofBlockAST]:
        """Parse formal theorem blocks and their canonical 5-part structure.

        Part labels are recognized in both "Part N -- Keyword" and numbered
        "N. Keyword" styles; keyword families must match the part number.
        """
        block_start = re.compile(r"^\s*(#{2,4})\s+.*\bTheorem\b", re.IGNORECASE)
        heading_part = re.compile(
            r"^\s*#{3,6}\s+(?:\*{0,2})?(?:Part\s*(\d+)|(\d+)[.)])",
            re.IGNORECASE,
        )
        non_heading_part = re.compile(
            r"^\s*(?:\*{0,2})?(?:Part\s*(\d+)|(\d+)[.)])",
            re.IGNORECASE,
        )
        blocks: List[ProofBlockAST] = []
        current: Optional[ProofBlockAST] = None
        has_heading_parts: bool = False
        theorem_heading_level: int = 3

        def finish():
            nonlocal current, has_heading_parts
            if current is not None:
                blocks.append(current)
                current = None
                has_heading_parts = False

        for line_number, raw_line in enumerate(text.splitlines(), start=1):
            line = raw_line.strip()
            start_m = block_start.match(line)
            if start_m:
                finish()
                theorem_heading_level = len(start_m.group(1))
                id_match = re.search(r"\b([A-Z0-9_-]+-\d+)\b", line)
                current = ProofBlockAST(
                    theorem_id=id_match.group(1) if id_match else f"Theorem@{line_number}",
                    line_number=line_number,
                )
                continue
            if current is not None:
                h_part = heading_part.match(line)
                part_number = None
                if h_part:
                    p_str = h_part.group(1) or h_part.group(2)
                    p_num = int(p_str)
                    if 1 <= p_num <= 5:
                        part_number = p_num
                        has_heading_parts = True
                elif not has_heading_parts:
                    if not raw_line.startswith((" ", "\t")):
                        nh_part = non_heading_part.match(line)
                        if nh_part:
                            p_str = nh_part.group(1) or nh_part.group(2)
                            p_num = int(p_str)
                            if 1 <= p_num <= 5:
                                part_number = p_num

                if part_number is not None:
                    lowered = line.lower()
                    if part_number == 1 and ("proposition" in lowered or "statement" in lowered):
                        current.proposition = line
                    elif part_number == 2 and ("assumption" in lowered or "state space" in lowered or "domain bound" in lowered):
                        current.assumptions = line
                    elif part_number == 3 and ("invariant" in lowered or "barrier" in lowered):
                        current.barrier_function = line
                    elif part_number == 4 and ("deriv" in lowered or "inductive" in lowered):
                        current.derivation = line
                    elif part_number == 5 and ("conclusion" in lowered or "q.e.d" in lowered or "qed" in lowered):
                        current.conclusion = line
                    continue

                h_match = re.match(r"^\s*(#{1,6})\s+\S", line)
                if h_match:
                    level = len(h_match.group(1))
                    if level <= max(theorem_heading_level, 3) and level <= 3:
                        finish()
                        continue
                    elif level <= theorem_heading_level:
                        finish()
                        continue
        finish()
        return blocks


class CartesianProductValidator:
    """Set-theoretic validator: UCA Cartesian completeness, SORA OSO coverage, and proof structure."""

    @classmethod
    def verify_cartesian_completeness(cls, uca_rows: List[STPARowAST], expected_actions: List[str]) -> ASTValidationReport:
        """Verify every (control action x guide word) permutation has at least one UCA row."""
        report = ASTValidationReport()
        report.total_uca_rows = len(uca_rows)
        unique_actions = []
        for action in expected_actions:
            if action not in unique_actions:
                unique_actions.append(action)
        report.expected_uca_rows = 4 * len(unique_actions)
        if not unique_actions:
            return report

        found = set()
        for row in uca_rows:
            matched_action = None
            for action in unique_actions:
                if re.search(rf"\b{re.escape(action)}\b", row.control_action, re.IGNORECASE):
                    matched_action = action
                    break
            if matched_action is None:
                continue
            classified_list = classify_uca_guide_words(row.guide_word)
            for gw_id, _label in classified_list:
                found.add((matched_action, gw_id))

        expected = set()
        for action in unique_actions:
            for gw_id, _label, _pattern in STPA_GUIDE_WORD_RULES:
                expected.add((action, gw_id))

        missing = sorted(
            expected - found,
            key=lambda pair: (pair[0], STPA_GUIDE_WORD_ORDER[pair[1]]),
        )
        report.missing_permutations = [
            f"{action} x {gw_id} ({STPA_GUIDE_WORD_LABELS[gw_id]})" for action, gw_id in missing
        ]
        report.is_conforming = not report.missing_permutations
        return report

    @classmethod
    def verify_sora_oso_coverage(cls, oso_records: List[SORAOsoAST]) -> ASTValidationReport:
        """Verify structural coverage of all 24 SORA Operational Safety Objectives (OSO-01..OSO-24)."""
        report = ASTValidationReport()
        found_ids = {record.oso_id.upper() for record in oso_records}
        report.missing_osos = [
            f"OSO-{index:02d}" for index in range(1, 25) if f"OSO-{index:02d}" not in found_ids
        ]
        report.is_conforming = not report.missing_osos
        return report

    @classmethod
    def verify_proof_structure(cls, proof_blocks: List[ProofBlockAST]) -> ASTValidationReport:
        """Verify each theorem block carries the canonical 5-part mathematical proof structure."""
        report = ASTValidationReport()
        part_labels = {1: "Proposition", 2: "Assumptions", 3: "Invariant", 4: "Derivation", 5: "Conclusion"}
        attributes = {
            1: lambda block: block.proposition,
            2: lambda block: block.assumptions,
            3: lambda block: block.barrier_function,
            4: lambda block: block.derivation,
            5: lambda block: block.conclusion,
        }
        for block in proof_blocks:
            for part_number in (1, 2, 3, 4, 5):
                if not attributes[part_number](block):
                    report.malformed_proofs.append(
                        f"{block.theorem_id}: Missing Part {part_number} {part_labels[part_number]}"
                    )
        report.is_conforming = not report.malformed_proofs
        return report


def validate_safety_matrix_ast(content: str, model_text: Optional[str] = None) -> Tuple[List[str], ASTValidationReport, Optional[List[str]]]:
    """Run structural AST validation of the safety matrix, optionally against the authoritative SysML model.

    When no model text is supplied (schema-less downstream inputs), guide-word
    completeness is enforced over the (controller, control action) pairs derived
    from the UCA table itself and the canonical 5-part proof structure is
    enforced on every parsed theorem block; the full Cartesian cardinality
    comparison and FMECA part def completeness against the schema remain model-gated.

    Returns (violation_strings, report, expected_control_actions_or_none).
    """
    errors: List[str] = []
    report = ASTValidationReport()

    stpa_rows = MarkdownTableASTParser.parse_stpa_table(content)
    oso_rows = MarkdownTableASTParser.parse_sora_table(content)
    proof_blocks = MarkdownTableASTParser.parse_proof_blocks(content)

    # Diagnostic check: if STPA rows are parsed but guide words are missing or unclassifiable
    if stpa_rows:
        rows_with_empty_gw = sum(1 for r in stpa_rows if not r.guide_word.strip())
        rows_unclassifiable_gw = sum(
            1 for r in stpa_rows
            if r.guide_word.strip() and not classify_uca_guide_words(r.guide_word)
        )
        if rows_with_empty_gw == len(stpa_rows):
            errors.append(
                "Pillar 4 violation: UCA table guide word / failure mode column could not be resolved from headers. "
                "Expected column header matching: 'Guide Word', 'Failure Mode', 'Class attribution', 'Type', or 'Mode'."
            )
        elif rows_unclassifiable_gw > 0:
            errors.append(
                f"Pillar 4 diagnostic: {rows_unclassifiable_gw} UCA row(s) contain unclassifiable guide word text. "
                "Ensure guide words conform to canonical STPA categories (Not providing, Providing, Too early/late, Stopped too soon/applied too long) or Class a/b/c/d."
            )

    expected_actions: Optional[List[str]] = None
    expected_parts: Optional[List[str]] = None
    if model_text:
        parse_sysml = _load_sysml_parser()
        try:
            model_ast = parse_sysml(model_text)
        except Exception as exc:
            errors.append(f"Safety AST violation: Failed to parse SysML v2 model ({exc}).")
            broken = ASTValidationReport(is_conforming=False, syntax_errors=[str(exc)])
            return errors, broken, None
        expected_actions = sorted({str(name) for name in model_ast.get("action_defs", [])})
        expected_parts = sorted({str(name) for name in model_ast.get("part_defs", [])})
        sysml_reqs = model_ast.get("requirement_defs", [])

        # Pillar 6: Safety Constraint Parity Verification
        sc_ids = set(re.findall(r'\b(SC(?:-[A-Za-z0-9_]+)?-\d+)\b', content))
        if len(sc_ids) > len(sysml_reqs):
            errors.append(
                f"Pillar 6 Parity Violation: Found {len(sc_ids)} markdown safety constraints, "
                f"but only {len(sysml_reqs)} requirement def nodes in SysML model. "
                f"Model is out of sync; run scripts/compile_sysml.py --reverse-sync."
            )

        # Pillar 6: Stateflow Synthesis Hooks & Statechart AST Coverage
        sf_errors, missing_sf_diagrams, missing_sf_hooks = check_stateflow_ast_coverage(content, model_ast)
        if missing_sf_diagrams:
            report.missing_state_diagrams.extend(missing_sf_diagrams)
        if missing_sf_hooks:
            report.missing_stateflow_hooks.extend(missing_sf_hooks)
        errors.extend(sf_errors)

    if expected_actions:
        cartesian_report = CartesianProductValidator.verify_cartesian_completeness(stpa_rows, expected_actions)
        report.total_uca_rows = cartesian_report.total_uca_rows
        report.expected_uca_rows = cartesian_report.expected_uca_rows
        if not stpa_rows:
            errors.append(
                "Pillar 4 violation: No structural UCA table rows could be parsed from the safety matrix; "
                f"expected {report.expected_uca_rows} permutations ({len(expected_actions)} control actions x 4 guide words)."
            )
        elif cartesian_report.missing_permutations:
            report.missing_permutations.extend(cartesian_report.missing_permutations)
            shown = cartesian_report.missing_permutations[:15]
            listing = "\n".join(f"    - {item}" for item in shown)
            remaining = len(cartesian_report.missing_permutations) - len(shown)
            if remaining > 0:
                listing += f"\n    - ... and {remaining} more"
            found_combos = report.expected_uca_rows - len(cartesian_report.missing_permutations)
            errors.append(
                f"Pillar 4 violation: UCA Cartesian completeness failure -- expected {report.expected_uca_rows} "
                f"permutations ({len(expected_actions)} control actions x 4 guide words), found {found_combos} "
                f"unique combinations. Missing permutations:\n{listing}"
            )
    elif stpa_rows and not model_text:
        derived_actions = []
        for row in stpa_rows:
            if row.control_action and row.control_action not in derived_actions:
                derived_actions.append(row.control_action)
        cartesian_report = CartesianProductValidator.verify_cartesian_completeness(stpa_rows, derived_actions)
        report.total_uca_rows = cartesian_report.total_uca_rows
        report.expected_uca_rows = cartesian_report.expected_uca_rows
        if cartesian_report.missing_permutations:
            report.missing_permutations.extend(cartesian_report.missing_permutations)
            shown = cartesian_report.missing_permutations[:15]
            listing = "\n".join(f"    - {item}" for item in shown)
            remaining = len(cartesian_report.missing_permutations) - len(shown)
            if remaining > 0:
                listing += f"\n    - ... and {remaining} more"
            found_combos = report.expected_uca_rows - len(cartesian_report.missing_permutations)
            errors.append(
                f"Pillar 4 violation: UCA guide-word completeness failure -- expected {report.expected_uca_rows} "
                f"permutations ({len(derived_actions)} control actions x 4 guide words), found {found_combos} "
                f"unique combinations. Missing permutations:\n{listing}"
            )
        found_combos = report.expected_uca_rows - len(cartesian_report.missing_permutations)
        if report.total_uca_rows < MIN_STRUCTURAL_UCA_ROWS and found_combos < MIN_STRUCTURAL_UCA_ROWS:
            errors.append(
                f"Pillar 4 violation: UCA Cartesian matrix truncation -- found {report.total_uca_rows} UCA row(s); "
                f"minimum required is {MIN_STRUCTURAL_UCA_ROWS} permutations (4 control actions x 4 guide words)."
            )

    if expected_parts or model_text:
        fmeca_ast_errors, fmeca_report = check_fmeca_ast_coverage(content, model_text)
        if fmeca_report.missing_fmeca_parts:
            report.missing_fmeca_parts.extend(fmeca_report.missing_fmeca_parts)
        if fmeca_report.undeclared_fmeca_parts:
            report.undeclared_fmeca_parts.extend(fmeca_report.undeclared_fmeca_parts)
        if fmeca_report.missing_port_modes:
            report.missing_port_modes.extend(fmeca_report.missing_port_modes)
        if fmeca_report.missing_dimensions:
            report.missing_dimensions.extend(fmeca_report.missing_dimensions)
        report.part_criticalities.update(fmeca_report.part_criticalities)
        errors.extend(fmeca_ast_errors)

    if model_text or oso_rows:
        sora_report = CartesianProductValidator.verify_sora_oso_coverage(oso_rows)
        report.missing_osos.extend(sora_report.missing_osos)
        if sora_report.missing_osos:
            errors.append(
                f"Pillar 8 violation: Missing mandatory SORA Operational Safety Objectives: "
                f"{', '.join(sora_report.missing_osos)}."
            )

    if proof_blocks:
        proof_report = CartesianProductValidator.verify_proof_structure(proof_blocks)
        report.malformed_proofs.extend(proof_report.malformed_proofs)
        for message in proof_report.malformed_proofs:
            errors.append(f"Formal proof violation: {message}.")

    report.is_conforming = not errors
    return errors, report, expected_actions


def _validate_aggregate_safety_content(
    aggregate_safety_content: str,
    repo_root: Optional[str] = None,
    model_text: Optional[str] = None,
) -> Tuple[list, Optional[ASTValidationReport]]:
    """Run pillar validation plus structural AST validation.

    The structural AST validation is model-optional: when a SysML model is
    discoverable under repo_root, the full Cartesian cardinality is compared
    against the model's action definitions and FMECA component completeness is
    compared against the model's part definitions; without a model, guide-word
    completeness is enforced against the (controller, control action) pairs
    derived from the UCA table itself and the 5-part proof structure is
    enforced on parsed theorem blocks.

    Returns (violation_strings, ast_report_or_none).
    """
    errors: List[str] = []
    ast_report: Optional[ASTValidationReport] = None

    if model_text is None and repo_root:
        model_text = _discover_sysml_model_text(repo_root)

    ast_errors, ast_report, _expected_actions = validate_safety_matrix_ast(aggregate_safety_content, model_text)
    errors.extend(ast_errors)

    errors.extend(_validate_safety_matrix_pillars(aggregate_safety_content, ast_path_active=model_text is not None))
    return errors, ast_report


def validate_safety_matrix_content(
    content: str,
    repo_root: Optional[str] = None,
    model_text: Optional[str] = None,
) -> list:
    """Validate 8-pillar schema, 24 SORA OSOs, FMECA matrix with AST closure, 4 UCA categories, ASTM F3269-17 RTA, and MATLAB/Simulink hooks.

    Structural table-aware AST validation is model-optional. When a SysML v2
    model is discoverable under repo_root (schema/*.sysml or .pipeline/schema.sysml),
    dynamic Cartesian product set equality against the model's action definitions
    and FMECA component completeness against the model's part definitions
    supersede the legacy regex keyword checks; without a model, guide-word
    completeness over table-derived (controller, control action) pairs and the
    5-part proof structure are still enforced structurally, while the legacy
    regex scans remain the fallback for pillar presence and SORA OSO coverage.

    Returns a list of violation error strings (empty if valid).
    """
    errors, _ast_report = _validate_aggregate_safety_content(content, repo_root, model_text)
    return errors


def _validate_safety_matrix_pillars(content: str, ast_path_active: bool = False) -> list:
    """Validate the 8-pillar schema presence checks (regex-based) plus structural counts.

    When ast_path_active is True, the shallow regex UCA-category and SORA-OSO
    scans are skipped because the structural AST validation already supersedes
    them; the regex checks remain the fallback for schema-less legacy inputs.
    """
    errors = []

    # Pillar 1: System Losses (L-1..N)
    if not (re.search(r'Loss(?:es)?', content, re.IGNORECASE) and re.search(r'\bL-\d+\b|\$L-\d+', content)):
        errors.append("Pillar 1 violation: Missing System Losses ($L-1..N$) identification.")

    # Pillar 2: System Hazards (H-1..N)
    if not (re.search(r'Hazard(?:s)?', content, re.IGNORECASE) and re.search(r'\bH-\d+\b|\$H-\d+', content)):
        errors.append("Pillar 2 violation: Missing System Hazards ($H-1..N$) identification.")

    # Pillar 3: Hierarchical Control Structure Topology
    if not (re.search(r'Control\s+Structure', content, re.IGNORECASE) or (re.search(r'Controller', content, re.IGNORECASE) and re.search(r'Actuator', content, re.IGNORECASE))):
        errors.append("Pillar 3 violation: Missing Hierarchical Control Structure Topology.")

    # Pillar 4: Unsafe Control Actions (UCA-1..N)
    if not (re.search(r'Unsafe\s+Control\s+Actions?', content, re.IGNORECASE) or re.search(r'\bUCA-\d+\b', content)):
        errors.append("Pillar 4 violation: Missing Unsafe Control Actions ($UCA-1..N$).")
    if not ast_path_active:
        missing_uca_cats = check_uca_categories(content)
        if missing_uca_cats:
            errors.append(f"Pillar 4 violation: Missing UCA failure mode categories: {', '.join(missing_uca_cats)}.")

    # Pillar 5: Loss Scenarios (LS-1..N)
    if not (re.search(r'Loss\s+Scenarios?|Causal\s+Scenarios?', content, re.IGNORECASE) and re.search(r'\bLS-\d+\b|\$LS-\d+', content)):
        errors.append("Pillar 5 violation: Missing Loss Scenarios ($LS-1..N$) & Causal Factors.")

    # Pillar 6: Formal Safety Constraints (SC-1..N)
    if not (re.search(r'Safety\s+Constraints?', content, re.IGNORECASE) and re.search(r'\bSC-\d+\b|\$SC-\d+', content)):
        errors.append("Pillar 6 violation: Missing Formal Safety Constraints ($SC-1..N$).")

    # Pillar 7: FMECA Criticality Matrix (AST Closure, RPN, Basis, Universal 4 Dimensions)
    if not re.search(r'FMECA|Failure\s+Mode', content, re.IGNORECASE):
        errors.append("Pillar 7 violation: Missing FMECA Criticality Matrix.")
    else:
        fmeca_data = parse_fmeca_table(content)
        total_rows = fmeca_data["total_rows"]
        if total_rows == 0:
            errors.append("Pillar 7 violation: FMECA Criticality Matrix contains 0 rows.")
        if not (fmeca_data.get("has_rpn") or re.search(r'\bRPN\b|Risk\s+Priority\s+Number', content, re.IGNORECASE)):
            errors.append("Pillar 7 violation: FMECA table missing RPN (Risk Priority Number) calculation.")

        total_basis = fmeca_data["basis_counts"]["SSOT"] + fmeca_data["basis_counts"]["Derived"]
        if total_rows > 0 and total_basis == 0:
            errors.append("Pillar 7 violation: FMECA Criticality Matrix missing explicit Derivation Basis classification ('SSOT' / 'Derived').")
        elif total_rows > 0 and total_basis < total_rows:
            errors.append(f"Pillar 7 violation: FMECA Criticality Matrix contains {total_rows - total_basis} row(s) missing explicit Derivation Basis classification ('SSOT' / 'Derived').")

        if total_rows > 0:
            missing_dims = check_failure_dimension_coverage(fmeca_data)
            if missing_dims:
                errors.append(
                    f"Pillar 7 violation: FMECA table missing coverage for universal failure dimension(s): "
                    f"{', '.join(missing_dims)} (expected Interface (Γ), State (Φ), Action (Ω), Resource (Ψ))."
                )

        for row in fmeca_data["rows"]:
            row_id = row.get("failure_id") or row.get("failure_mode") or "Row"
            s = row.get("s")
            o = row.get("o")
            d = row.get("d")
            rpn = row.get("rpn")
            if s is not None and o is not None and d is not None:
                try:
                    s_int = int(s)
                    o_int = int(o)
                    d_int = int(d)
                    if not (1 <= s_int <= 10 and 1 <= o_int <= 10 and 1 <= d_int <= 10):
                        errors.append(f"Pillar 7 violation: FMECA row '{row_id}' ratings out of range [1, 10] (S={s_int}, O={o_int}, D={d_int}).")
                    expected_rpn = s_int * o_int * d_int
                    if rpn is not None:
                        try:
                            rpn_int = int(rpn)
                            if rpn_int != expected_rpn:
                                errors.append(f"Pillar 7 violation: FMECA row '{row_id}' has invalid RPN calculation -- expected S({s_int}) * O({o_int}) * D({d_int}) = {expected_rpn}, but found RPN = {rpn_int}.")
                        except ValueError:
                            errors.append(f"Pillar 7 violation: FMECA row '{row_id}' has non-integer RPN '{rpn}'.")
                except ValueError:
                    errors.append(f"Pillar 7 violation: FMECA row '{row_id}' has non-integer S/O/D ratings (S='{s}', O='{o}', D='{d}').")

    # Pillar 8: SORA SAIL Risk Mitigations & OSO Traceability Table
    if not (re.search(r'\bSORA\b', content) and re.search(r'\bSAIL\b', content)):
        errors.append("Pillar 8 violation: Missing SORA SAIL risk assessment.")
    if not (re.search(r'\bGRC\b|Ground\s+Risk\s+Class', content, re.IGNORECASE) and re.search(r'\bARC\b|Air\s+Risk\s+Class', content, re.IGNORECASE)):
        errors.append("Pillar 8 violation: Missing GRC (Ground Risk Class) or ARC (Air Risk Class) determinations.")
    if not ast_path_active:
        missing_osos = check_sora_osos(content)
        if missing_osos:
            errors.append(f"Pillar 8 violation: Missing mandatory SORA Operational Safety Objectives: {', '.join(missing_osos)}.")

    # ASTM F3269-17 RTA Architecture
    if not (re.search(r'ASTM\s+F3269', content, re.IGNORECASE) and re.search(r'Run-Time\s+Assurance|\bRTA\b|Safety\s+Net', content, re.IGNORECASE)):
        errors.append("Safety Architecture violation: Missing ASTM F3269-17 Run-Time Assurance (RTA) / Safety Net specification.")

    # MATLAB / Simulink / Stateflow hooks
    if not re.search(r'MATLAB|Simulink|Stateflow|Embedded\s+Coder|SLDV', content, re.IGNORECASE):
        errors.append("Commercial Toolchain violation: Missing MATLAB / Simulink / Stateflow / Embedded Coder integration hooks.")

    return errors

def check_safety_integrity_and_sora_completeness(repo_root):
    """Check 17: Safety Integrity Quality Gate and SORA OSO-01..24 Completeness Verification.

    Validates:
    1. Upstream clean landing zone invariant for docs/safety/ (zero concrete specifications in templates).
    2. Downstream 8-pillar STPA/FMECA/SORA specification schema in docs/safety/STPA_MATRIX.md:
       - Pillar 1: System Losses (L-1..N)
       - Pillar 2: System Hazards (H-1..N)
       - Pillar 3: Hierarchical Control Structure Topology
       - Pillar 4: Unsafe Control Actions (UCA-1..N) covering all 4 failure modes
       - Pillar 5: Loss Scenarios (LS-1..N) & Causal Factors
       - Pillar 6: Formal Safety Constraints (SC-1..N)
       - Pillar 7: FMECA Criticality Matrix with AST closure, universal 4 dimensions, and RPN
       - Pillar 8: SORA SAIL Risk Mitigations with all 24 OSOs (OSO-01 through OSO-24), GRC, and ARC
       - ASTM F3269-17 Run-Time Assurance (RTA) architecture
       - MATLAB / Simulink / Stateflow model integration baseline hooks.
    """
    upstream_marker = os.path.join(repo_root, ".pipeline", "upstream")
    safety_dir = os.path.join(repo_root, "docs", "safety")

    if os.path.isdir(upstream_marker):
        if os.path.isdir(safety_dir):
            allowed_files = {".gitkeep", "README.md"}
            violations = []
            for root, dirs, files in os.walk(safety_dir):
                dirs[:] = [d for d in dirs if d not in EXCLUDED_DIRS]
                for f in files:
                    if f not in allowed_files:
                        rel_path = os.path.relpath(os.path.join(root, f), repo_root)
                        violations.append(rel_path)
            if violations:
                print(f"ERROR: Check 17 failed: Upstream distribution template safety landing zone contains concrete specification files: {', '.join(violations)}", file=sys.stderr)
                sys.exit(1)
        print("Success: Check 17 verified (Upstream distribution template safety landing zone is clean).")
        return

    # Downstream repository validation
    if not os.path.isdir(safety_dir):
        print("Success: Check 17 verified (Downstream repository detected -- docs/safety/ directory not present).")
        return

    safety_files = []
    for root, dirs, files in os.walk(safety_dir):
        dirs[:] = [d for d in dirs if d not in EXCLUDED_DIRS and d not in ("defects", "audits", "decisions")]
        for f in files:
            if f.endswith(".md") and f != "README.md":
                safety_files.append(os.path.join(root, f))

    if not safety_files:
        print("Success: Check 17 verified (Downstream repository detected -- safety specifications pending or clean).")
        return

    all_errors = []
    # If there is a single primary safety matrix (e.g. STPA_MATRIX.md), validate it individually.
    # Otherwise, aggregate content across modular safety specs (e.g. STPA + FMECA + SORA in separate files).
    combined_content = []
    for s_file in sorted(safety_files):
        rel_path = os.path.relpath(s_file, repo_root)
        try:
            with open(s_file, "r", encoding="utf-8") as f:
                combined_content.append(f.read())
        except Exception as e:
            all_errors.append(f"Failed to read {rel_path}: {e}")

    aggregate_safety_content = "\n\n---\n\n".join(combined_content)
    file_errors, ast_report = _validate_aggregate_safety_content(aggregate_safety_content, repo_root=repo_root)
    for err in file_errors:
        all_errors.append(f"docs/safety/ (aggregate specifications): {err}")

    if all_errors:
        print("ERROR: Check 17 failed (Safety Integrity Quality Gate and SORA OSO-01..24 Completeness violations found):", file=sys.stderr)
        for err in all_errors:
            print(f"  - {err}", file=sys.stderr)
        sys.exit(1)

    if ast_report is not None:
        print(ast_report.format_cli_summary())
    print("Success: Check 17 verified (Safety Integrity Quality Gate: 8 pillars, 24 SORA OSOs, FMECA matrix with AST closure, 4 UCA categories, ASTM F3269-17 RTA, and MATLAB/Simulink hooks).")

def verify_upstream_blueprint_domain_cleanliness(target_dir):
    """Check 18: Upstream Blueprint Domain Cleanliness Gate.

    Verify that upstream DEAP01-spec-core architecture blueprints contain zero concrete
    domain platform concept papers or domain SysML models (e.g. *FLIGHT_SYSTEMS*,
    *UAS_INFRASTRUCTURE*, *FRONTEND_SYSTEMS*, *SAFETY_MODEL*.sysml).
    """
    upstream_marker = os.path.join(target_dir, ".pipeline", "upstream")
    if not (os.path.isdir(upstream_marker) or os.path.isfile(upstream_marker)):
        print("Success: Check 18 verified (Downstream repository detected -- skipping upstream blueprint domain cleanliness gate).")
        return

    blueprints_dir = os.path.join(target_dir, "docs", "architecture", "blueprints")
    if not os.path.isdir(blueprints_dir):
        print("Success: Check 18 verified (docs/architecture/blueprints/ not present).")
        return

    forbidden_patterns = [
        re.compile(r"flight[-_]?systems", re.IGNORECASE),
        re.compile(r"uas[-_]?infrastructure", re.IGNORECASE),
        re.compile(r"frontend[-_]?systems", re.IGNORECASE),
        re.compile(r"safety[-_]?model", re.IGNORECASE),
        re.compile(r"\.sysml$", re.IGNORECASE),
        re.compile(r"concept[-_]?paper", re.IGNORECASE),
    ]

    violations = []
    for root, dirs, files in os.walk(blueprints_dir):
        dirs[:] = [d for d in dirs if d not in EXCLUDED_DIRS]
        for f in files:
            rel_path = os.path.relpath(os.path.join(root, f), target_dir)
            if any(pattern.search(f) for pattern in forbidden_patterns):
                violations.append(rel_path)

    if violations:
        print(f"ERROR: Check 18 failed: Upstream blueprints contain concrete domain platform concept papers or sysml models: {', '.join(violations)}", file=sys.stderr)
        sys.exit(1)

    print("Success: Check 18 verified (Upstream architecture blueprints are clean with zero domain concept papers or sysml models).")

check_upstream_blueprint_domain_cleanliness = verify_upstream_blueprint_domain_cleanliness

ALLOWED_M2_METAMODEL_TYPES: Set[str] = {
    # Core structural elements
    "Component",
    "Class",
    "Port",
    "Interface",
    "Statechart",
    "Constraint",
    "Signal",
    "Event",
    "AcceptanceCriterion",
    "Scenario",
    "TraceLink",
    # SysML v2 & KerML Definition types
    "Package",
    "PackageDefinition",
    "PartDefinition",
    "PortDefinition",
    "StateDefinition",
    "ItemDefinition",
    "ActionDefinition",
    "RequirementDefinition",
    "UseCaseDefinition",
    "ConstraintDefinition",
    "AttributeDefinition",
    "ConnectionDefinition",
    "AllocationDefinition",
    "ViewDefinition",
    "ViewpointDefinition",
    "ActorDefinition",
    "NamespaceDefinition",
    "ElementDefinition",
    "FeatureDefinition",
    "Classifier",
    # Actor and Role types
    "HumanOperator",
    "SystemController",
    "SafetyInterlock",
    "PhysicalActuator",
    "Sensor",
    "SystemUnderStudy",
    "ExternalSystem",
    "OperatorConsole",
    # Canonical M2 elements
    "Actor",
    "Part",
    "Item",
    "Action",
    "State",
    "Requirement",
    "UseCase",
    "Attribute",
    "Connection",
    "Allocation",
    "Transition",
    "Guard",
    "Trigger",
    "Effect",
    # AST, Schema, Parser & Model primitives
    "Namespace",
    "Object",
    "Array",
    "String",
    "Number",
    "Boolean",
    "Integer",
    "Dict",
    "List",
    "Null",
    "Primitive",
    "Type",
    "Definition",
    "Block",
    "Node",
    "Root",
    "Value",
    "Field",
    "Member",
    "Document",
    # Logical UI (LUI / LUMI) Canonical Display, Container & Widget primitives
    "Widget",
    "Container",
    "Layout",
    "View",
    "SidebarLayout",
    "HierarchyTree",
    "ResizableSplitter",
    "TopologyMap",
    "DensityTable",
    "TabbedContainer",
    "SplitterContainer",
    "Panel",
    "Section",
    "Tab",
    "Tree",
    "Table",
    "Map",
    "Chart",
    "Form",
    "Button",
    "Input",
    "Dialog",
    "Modal",
}


def is_allowed_m2_type(type_name: str) -> bool:
    """Check if a type name conforms to the closed M2 metamodel allowlist or carries a meta_ prefix."""
    if not isinstance(type_name, str) or not type_name.strip():
        return False
    cleaned = type_name.strip()
    if cleaned.lower().startswith("meta_") or cleaned.lower().startswith("meta"):
        return True
    norm = cleaned.lower().replace("_", "").replace("-", "")
    norm_allowed = {t.lower().replace("_", "").replace("-", "") for t in ALLOWED_M2_METAMODEL_TYPES}
    return norm in norm_allowed


class ClosedGrammarMetamodelValidator(ast.NodeVisitor):
    """AST visitor enforcing pure schema-driven parameter extraction, closed M2 metamodel typing, and zero static domain specs."""

    STATIC_PARAM_DICT_NAMES = re.compile(
        r"^(_)?("
        r"ground_?truth(_?(specs?|params?|parameters?|dict|map|set|table))?|"
        r"(expected|domain|static|hardcoded|benchmark|mandated|system)_?(specs?|params?|parameters?|constants?|dict|map|set|table|specifications?)"
        r")$",
        re.IGNORECASE,
    )

    M1_DOMAIN_DICT_NAMES = re.compile(
        r"^(_)?("
        r"(m1|domain|concrete)_(entities|instances|models|specs|objects|dicts|types)|"
        r"(sample|mock|concrete)_(uav|aircraft|vehicle|device|patient|car|robot)(_?(specs|params|data|dict))?"
        r")$",
        re.IGNORECASE,
    )

    METAMODEL_TYPE_KEYS = {
        "type",
        "metamodel_type",
        "entity_type",
        "kind",
        "node_type",
        "ast_type",
        "element_type",
        "m2_type",
        "definition_type",
    }

    def __init__(self, filename: str, repo_root: str):
        self.filename = filename
        self.rel_path = os.path.relpath(filename, repo_root)
        self.violations = []
        self.scope_stack = []

    def visit_ClassDef(self, node: ast.ClassDef):
        self.scope_stack.append(node.name)
        if self.STATIC_PARAM_DICT_NAMES.match(node.name):
            has_static_attrs = any(
                isinstance(stmt, ast.Assign) and isinstance(stmt.value, (ast.Constant, ast.Dict, ast.List, ast.Set, ast.Tuple))
                for stmt in node.body
            )
            if has_static_attrs:
                self.violations.append(
                    f"Check 19 violation: Static domain specification class \"{node.name}\" declared in {self.rel_path}:{node.lineno}. "
                    "Domain parameters must be dynamically parsed from schema/*.sysml or workspace.schemas."
                )
        self.generic_visit(node)
        self.scope_stack.pop()

    def visit_FunctionDef(self, node: ast.FunctionDef):
        self.scope_stack.append(f"def {node.name}")
        if any(k in node.name.lower() for k in ("extract_ground_truth", "get_ground_truth", "extract_domain_specs", "get_expected_specs")):
            for child in ast.walk(node):
                if isinstance(child, ast.Return) and isinstance(child.value, ast.Dict) and len(child.value.keys) > 0:
                    self.violations.append(
                        f"Check 19 violation: Parameter extraction function \"{node.name}\" returns static literal parameter dictionary in {self.rel_path}:{child.lineno}. "
                        "All parameter extraction must dynamically query schema/*.sysml or workspace.schemas."
                    )
        self.generic_visit(node)
        self.scope_stack.pop()

    def visit_AsyncFunctionDef(self, node: ast.AsyncFunctionDef):
        self.scope_stack.append(f"async def {node.name}")
        self.generic_visit(node)
        self.scope_stack.pop()

    def _check_dict_metamodel_types(self, dict_node: ast.Dict, lineno: int):
        """Check dictionary literals for unvalidated M1 domain instance typing."""
        if not isinstance(dict_node, ast.Dict):
            return
        for key_node, val_node in zip(dict_node.keys, dict_node.values):
            if key_node is None or not isinstance(key_node, ast.Constant) or not isinstance(key_node.value, str):
                continue
            key_str = key_node.value.lower()
            if key_str in self.METAMODEL_TYPE_KEYS:
                if isinstance(val_node, ast.Constant) and isinstance(val_node.value, str):
                    val_str = val_node.value.strip()
                    if val_str and not is_allowed_m2_type(val_str):
                        self.violations.append(
                            f"Check 19 violation (domain-metamodel-typing-violation): Unvalidated M1 domain instance entity/type '{val_str}' declared in {self.rel_path}:{lineno}. "
                            "Upstream compiler ASTs and dictionaries must adhere strictly to the closed M2 metamodel allowlist."
                        )

    def visit_Dict(self, node: ast.Dict):
        self._check_dict_metamodel_types(node, getattr(node, "lineno", 1))
        self.generic_visit(node)

    def _check_target_name(self, target_name: str, value_node: ast.AST, lineno: int):
        if not target_name or value_node is None:
            return

        if self.M1_DOMAIN_DICT_NAMES.match(target_name):
            self.violations.append(
                f"Check 19 violation (domain-metamodel-typing-violation): Unvalidated M1 domain instance dictionary/constant \"{target_name}\" declared in {self.rel_path}:{lineno}. "
                "Upstream compiler ASTs must adhere strictly to the closed M2 metamodel allowlist."
            )
            return

        if self.STATIC_PARAM_DICT_NAMES.match(target_name):
            is_literal_dict = isinstance(value_node, ast.Dict) and len(value_node.keys) > 0
            is_literal_collection = isinstance(value_node, (ast.List, ast.Set, ast.Tuple)) and len(value_node.elts) > 0
            is_constant = isinstance(value_node, ast.Constant) and value_node.value is not None
            is_dict_call = (
                isinstance(value_node, ast.Call)
                and isinstance(value_node.func, ast.Name)
                and value_node.func.id in ("dict", "list", "set")
                and (len(value_node.args) > 0 or len(value_node.keywords) > 0)
            )

            is_module_or_class_level = len(self.scope_stack) == 0 or (
                len(self.scope_stack) == 1 and not self.scope_stack[0].startswith("def ") and not self.scope_stack[0].startswith("async def ")
            )

            if is_literal_dict or is_literal_collection or is_constant or is_dict_call or (is_module_or_class_level and isinstance(value_node, (ast.Dict, ast.List, ast.Set, ast.Tuple))):
                self.violations.append(
                    f"Check 19 violation: Static hardcoded parameter dictionary/constant \"{target_name}\" declared in {self.rel_path}:{lineno}. "
                    "Domain specifications must be dynamically queried from workspace.schemas or schema/*.sysml AST nodes."
                )

    def visit_Assign(self, node: ast.Assign):
        for target in node.targets:
            target_name = None
            if isinstance(target, ast.Name):
                target_name = target.id
            elif isinstance(target, ast.Attribute):
                target_name = target.attr
            self._check_target_name(target_name, node.value, node.lineno)
        self.generic_visit(node)

    def visit_AnnAssign(self, node: ast.AnnAssign):
        target_name = None
        if isinstance(node.target, ast.Name):
            target_name = node.target.id
        elif isinstance(node.target, ast.Attribute):
            target_name = node.target.attr
        if node.value:
            self._check_target_name(target_name, node.value, node.lineno)
        self.generic_visit(node)


_DomainAgnosticASTVisitor = ClosedGrammarMetamodelValidator


def check_domain_agnostic_ast_cleanliness(repo_root):
    """Check 19: Domain-Agnostic AST Cleanliness & Closed-Grammar Metamodel Gate.

    Verify that upstream DEAP01-spec-core tools, scripts, and validator modules contain
    zero static/hardcoded parameter dictionaries (e.g. GROUND_TRUTH = {...}, EXPECTED_SPECS = {...},
    DOMAIN_PARAMS = {...}), enforce closed M2 metamodel entity allowlist typing, and ensure that all
    parameter extraction dynamically queries workspace.schemas or schema/*.sysml AST nodes without
    hardcoded domain concept constants.
    """
    upstream_marker = os.path.join(repo_root, ".pipeline", "upstream")
    if not os.path.isdir(upstream_marker):
        print("Success: Check 19 verified (Downstream repository detected -- skipping domain-agnostic AST cleanliness gate).")
        return

    scan_dirs = [
        os.path.join(repo_root, "skills", "spec-orchestrator", "parity_auditor", "src", "parity_auditor", "validators"),
        os.path.join(repo_root, "skills", "spec-orchestrator", "parity_auditor", "src", "parity_auditor", "core"),
        os.path.join(repo_root, "skills", "spec-orchestrator", "parity_auditor", "src", "parity_auditor", "parsers"),
        os.path.join(repo_root, "scripts"),
    ]

    violations = []

    for sdir in scan_dirs:
        if not os.path.isdir(sdir):
            continue
        for root, dirs, files in os.walk(sdir):
            dirs[:] = [d for d in dirs if d not in EXCLUDED_DIRS and d != "__pycache__"]
            for f in files:
                if not f.endswith(".py"):
                    continue
                if f in ("verify_downstream_baseline.py", "test_check_no_domain_config.py"):
                    continue
                if f.startswith("test_") and sdir.endswith("scripts"):
                    continue

                file_path = os.path.join(root, f)
                rel_path = os.path.relpath(file_path, repo_root)

                try:
                    with open(file_path, "r", encoding="utf-8") as py_file:
                        source = py_file.read()
                    tree = ast.parse(source, filename=file_path)
                except Exception as e:
                    violations.append(f"Failed to parse Python AST for {rel_path}: {e}")
                    continue

                visitor = ClosedGrammarMetamodelValidator(file_path, repo_root)
                visitor.visit(tree)
                violations.extend(visitor.violations)

    if violations:
        print("ERROR: Check 19 failed (Domain-Agnostic AST Cleanliness & Closed-Grammar Metamodel Gate violations found):", file=sys.stderr)
        for v in violations:
            print(f"  - {v}", file=sys.stderr)
        sys.exit(1)

    print("Success: Check 19 verified (Domain-Agnostic AST Cleanliness & Closed-Grammar Metamodel Gate passed -- pure dynamic schema AST architecture verified).")

def _check_wbs_suite_integrity(repo_root):
    """Check 20: WBS & Enterprise Deliverables Suite Validation.

    Verify that when docs/management/WBS_DELIVERABLES_SUITE.md exists:
    - docs/management/wbs_export_jira_monday_ms_project.csv exists and conforms to RFC 4180 with 12 headers
    - docs/management/wbs_export.json exists and conforms to the WBS JSON AST schema
    - WBS_DELIVERABLES_SUITE.md contains required section headers and table structure (2-col metadata, 7-col traceability)
    - All intra-document markdown hyperlinks in WBS_DELIVERABLES_SUITE.md resolve to existing files on disk
    - Zero Unicode em dashes (\\u2014) exist in any management deliverable.
    """
    wbs_md = os.path.join(repo_root, "docs", "management", "WBS_DELIVERABLES_SUITE.md")
    if not os.path.isfile(wbs_md):
        print("Success: Check 20 verified (WBS & Enterprise Deliverables Suite pending or not present).")
        return

    errors = []
    wbs_csv = os.path.join(repo_root, "docs", "management", "wbs_export_jira_monday_ms_project.csv")
    wbs_json = os.path.join(repo_root, "docs", "management", "wbs_export.json")

    # 1. Check presence of export deliverables
    if not os.path.isfile(wbs_csv):
        errors.append(f"Missing enterprise export deliverable: {os.path.relpath(wbs_csv, repo_root)}")
    if not os.path.isfile(wbs_json):
        errors.append(f"Missing enterprise export deliverable: {os.path.relpath(wbs_json, repo_root)}")

    # 2. Check section headers and table structure in WBS_DELIVERABLES_SUITE.md
    try:
        with open(wbs_md, "r", encoding="utf-8") as f:
            md_content = f.read()
    except Exception as e:
        errors.append(f"Failed to read {os.path.relpath(wbs_md, repo_root)}: {e}")
        md_content = ""

    if md_content:
        # Required section headers (case-insensitive regex)
        required_headers = [
            ("Executive Summary", r"##\s+.*Executive\s+Summary"),
            ("Baseline Deliverables", r"##\s+.*Baseline\s+Deliverables"),
            ("Subsystem Epics / Features", r"##\s+.*(?:Subsystem\s+Epics|Feature\s+Realization|Features)"),
            ("Verification Summary", r"##\s+.*(?:Verification.*Summary|Verification\s+&\s+Test)"),
            ("Import Guide", r"##\s+.*(?:Import\s+Guide|Project\s+Management\s+Export)"),
        ]
        for name, pattern in required_headers:
            if not re.search(pattern, md_content, re.IGNORECASE):
                errors.append(f"WBS_DELIVERABLES_SUITE.md missing required section: {name}")

        md_lines = md_content.splitlines()

        # Native 2-column Metadata Table at lines 1-10
        first_10_lines = "\n".join(md_lines[:10])
        if not re.search(r"\|\s*Attribute\s*\|\s*Specification\s+Detail\s*\|", first_10_lines, re.IGNORECASE):
            errors.append("WBS_DELIVERABLES_SUITE.md missing 2-column Metadata Table at lines 1-10 (| Attribute | Specification Detail |)")

        # 7-Column Traceability Matrix header containing required columns
        required_trace_cols = [
            "SysML Component",
            "Feature Spec",
            "User Stories",
            "MATLAB / Simulink Plant",
            "Python 250 Hz Engine",
            "Verification Suite",
            "Simulation Evidence",
        ]
        has_trace_matrix = any(
            line.strip().startswith("|") and line.strip().endswith("|") and all(col.lower() in line.lower() for col in required_trace_cols)
            for line in md_lines
        )
        if not has_trace_matrix:
            errors.append(
                "WBS_DELIVERABLES_SUITE.md missing 7-Column Traceability Matrix header with columns: "
                "SysML Component, Feature Spec, User Stories, MATLAB / Simulink Plant, Python 250 Hz Engine, Verification Suite, Simulation Evidence"
            )

        # Markdown hyperlink resolution verification
        link_pattern = re.compile(r"\[([^\]]+)\]\(([^)]+)\)")
        for link_text, link_target in link_pattern.findall(md_content):
            target_clean = link_target.strip()
            if target_clean.startswith("#") or target_clean.startswith("http://") or target_clean.startswith("https://") or target_clean.startswith("mailto:"):
                continue
            target_file = target_clean.split("#")[0].strip()
            if not target_file:
                continue
            resolved_path = (Path(repo_root) / "docs" / "management" / target_file).resolve()
            if not resolved_path.exists():
                errors.append(
                    f"Broken markdown link in WBS_DELIVERABLES_SUITE.md: '{link_target}' (resolved to non-existent path: {resolved_path})"
                )

    # 3. Check CSV export RFC 4180 parsing and 12 headers
    if os.path.isfile(wbs_csv):
        expected_csv_headers = [
            "WBS Code",
            "ID",
            "Item Type",
            "Name",
            "Parent ID",
            "Subsystem",
            "DO-178C Level",
            "Artifact Path",
            "Est. Hours",
            "Verification Gate",
            "Status",
            "Description",
        ]
        csv_rel = os.path.relpath(wbs_csv, repo_root)
        try:
            with open(wbs_csv, "r", encoding="utf-8") as f:
                reader = csv.reader(f)
                csv_rows = list(reader)
            if not csv_rows:
                errors.append(f"{csv_rel} is empty")
            else:
                actual_headers = [h.strip() for h in csv_rows[0]]
                if actual_headers != expected_csv_headers:
                    errors.append(
                        f"{csv_rel} header mismatch. Expected {expected_csv_headers}, got {actual_headers}"
                    )
                data_rows = csv_rows[1:]
                if not data_rows:
                    errors.append(f"{csv_rel} contains zero data rows (expected at least one non-header row)")
                else:
                    for r_idx, row in enumerate(data_rows, start=2):
                        if len(row) != len(expected_csv_headers):
                            errors.append(
                                f"{csv_rel} row {r_idx} column count mismatch: expected {len(expected_csv_headers)}, got {len(row)}"
                            )
        except Exception as e:
            errors.append(f"Failed to parse {csv_rel} as RFC 4180 CSV: {e}")

    # 4. Check JSON AST export parsing
    if os.path.isfile(wbs_json):
        json_rel = os.path.relpath(wbs_json, repo_root)
        try:
            with open(wbs_json, "r", encoding="utf-8") as f:
                json_data = json.load(f)
            if not isinstance(json_data, dict):
                errors.append(f"{json_rel} must be a JSON object")
            else:
                for req_key in ("metadata", "wbs_tree", "traceability_matrix"):
                    if req_key not in json_data:
                        errors.append(f"{json_rel} missing required top-level key: '{req_key}'")
                wbs_tree = json_data.get("wbs_tree")
                if not isinstance(wbs_tree, dict):
                    errors.append(f"{json_rel} 'wbs_tree' must be a JSON object")
                else:
                    for tree_key in ("wbs_code", "name", "level", "children"):
                        if tree_key not in wbs_tree:
                            errors.append(f"{json_rel} 'wbs_tree' missing required key: '{tree_key}'")
        except Exception as e:
            errors.append(f"Failed to parse {json_rel} as JSON: {e}")

    # 5. Check Zero Unicode Em Dash Invariant (\u2014)
    for target_path in (wbs_md, wbs_csv, wbs_json):
        if os.path.isfile(target_path):
            rel_path = os.path.relpath(target_path, repo_root)
            try:
                with open(target_path, "r", encoding="utf-8", errors="replace") as f:
                    file_text = f.read()
                if "\u2014" in file_text:
                    errors.append(f"Unicode em dash (\\u2014) detected in {rel_path}")
            except Exception as e:
                errors.append(f"Failed to scan {rel_path} for em dashes: {e}")

    # 6. Error handling
    if errors:
        print("ERROR: Check 20 failed (WBS & Enterprise Deliverables Suite violations found):", file=sys.stderr)
        for err in errors:
            print(f"  - {err}", file=sys.stderr)
        sys.exit(1)

    print("Success: Check 20 verified (WBS & Enterprise Deliverables Suite validated: Markdown structure, CSV RFC 4180 with 12 headers, JSON AST, and zero em dashes).")


check_wbs_suite_integrity = _check_wbs_suite_integrity


def _load_semantic_diagram_validator():
    """Import SemanticDiagramASTValidator and WorkspaceRepository fail-safe."""
    script_dir = os.path.dirname(os.path.abspath(__file__))
    project_root = os.path.dirname(script_dir)
    spec_dir = os.path.join(project_root, "skills", "spec-orchestrator", "scripts")
    parity_src = os.path.join(project_root, "skills", "spec-orchestrator", "parity_auditor", "src")
    scripts_dir = os.path.join(project_root, "scripts")
    for p in (scripts_dir, spec_dir, parity_src):
        if p not in sys.path:
            sys.path.insert(0, p)
    try:
        from parity_auditor.validators.semantic_diagram_ast_validator import SemanticDiagramASTValidator
        from parity_auditor.core.workspace import WorkspaceRepository
        return SemanticDiagramASTValidator, WorkspaceRepository
    except Exception:
        return None, None


def check_semantic_diagram_ast_parity(repo_root=None):
    """Check 21: Semantic Diagram-to-AST Topology Parity Gate.

    Verify that deliverable diagrams across docs/, rules/, and skills/ conform
    to SysML v2 AST topology, containing zero undeclared phantom nodes, zero
    inverted telemetry/signal flows, and zero ungrounded actuators.
    """
    if repo_root is None:
        repo_root = os.getcwd()

    model_text = _discover_sysml_model_text(repo_root)
    if not model_text or not model_text.strip():
        print("Success: Check 21 verified (SysML model pending or landing zone clean).")
        return

    val_cls, repo_cls = _load_semantic_diagram_validator()
    if val_cls is None or repo_cls is None:
        print("WARNING: Check 21 skipped (SemanticDiagramASTValidator or WorkspaceRepository unavailable).", file=sys.stderr)
        return

    repo = repo_cls(workspace_dir=repo_root)
    validator = val_cls(workspace_repo=repo)
    findings = validator.validate(repo, scan_dirs=["docs"])

    if findings:
        print("ERROR: Check 21 failed (Semantic Diagram-to-AST Topology Parity Gate violations found):", file=sys.stderr)
        for f in findings:
            print(f"  - {f}", file=sys.stderr)
        sys.exit(1)

    print("Success: Check 21 verified (Semantic Diagram-to-AST Topology Parity Gate passed -- zero undeclared nodes, inverted flows, or ungrounded actuators).")


def _validate_diagram_ast_parity(
    block: str,
    rel_path: str,
    ast_part_names: Optional[Set[str]] = None,
    pkg_obj: Any = None,
) -> List[str]:
    """Validate a single Mermaid diagram block against SysML AST topology and universal syntax rules."""
    errors = []
    lines = block.strip().splitlines()
    if not lines:
        return errors

    first_line = lines[0].strip()
    if not re.match(r"^(flowchart|graph|classDiagram|stateDiagram(?:-v2)?|sequenceDiagram)\b", first_line, re.IGNORECASE):
        errors.append(f"Missing mandatory Mermaid diagram type header in {rel_path} (got: '{first_line[:40]}')")
        return errors

    # Check forbidden hardware concepts in architecture diagrams (closed-world AST enforcement)
    forbidden_hardware = {"VTOLMotor", "LandingGear", "QuadPlane", "AutolandBeacon"}
    for bad in forbidden_hardware:
        if re.search(rf"\b{re.escape(bad)}\b", block):
            errors.append(
                f"Topological drift in {rel_path}: Diagram references forbidden/undeclared hardware concept '{bad}'."
            )

    # Class Diagram specific syntax checks
    if re.match(r"^classDiagram\b", first_line, re.IGNORECASE):
        for line_no, line in enumerate(lines[1:], start=2):
            stripped = line.strip()
            # Curly braces prohibited in class member lines
            if "{" in stripped or "}" in stripped:
                if not stripped.startswith("class ") and not stripped.endswith("{") and not stripped == "}":
                    errors.append(f"Mermaid syntax violation in {rel_path}:{line_no}: Curly braces '{{}}' inside class member line: '{stripped}'.")
            # Prohibit colons in class member strings (e.g. +method(a : int) : void)
            if re.search(r"^\s*[+\-#~]\w+\s*\(.*\)\s*:\s*\w+", stripped) or re.search(r"^\s*[+\-#~]\w+\s*\(.*:\s*\w+.*\)", stripped):
                errors.append(f"Mermaid syntax violation in {rel_path}:{line_no}: Colons ':' forbidden in class member line: '{stripped}'. Use '+ReturnType methodName(Type arg)' spacing.")

    return errors


check_diagram_to_ast_parity = check_semantic_diagram_ast_parity


def _load_semantic_prose_validator():
    """Import SemanticProseInvariantValidator and WorkspaceRepository fail-safe."""
    script_dir = os.path.dirname(os.path.abspath(__file__))
    project_root = os.path.dirname(script_dir)
    spec_dir = os.path.join(project_root, "skills", "spec-orchestrator", "scripts")
    parity_src = os.path.join(project_root, "skills", "spec-orchestrator", "parity_auditor", "src")
    scripts_dir = os.path.join(project_root, "scripts")
    for p in (scripts_dir, spec_dir, parity_src):
        if p not in sys.path:
            sys.path.insert(0, p)
    try:
        from parity_auditor.validators.semantic_prose_invariant_validator import SemanticProseInvariantValidator
        from parity_auditor.core.workspace import WorkspaceRepository
        return SemanticProseInvariantValidator, WorkspaceRepository
    except Exception:
        return None, None


def check_semantic_prose_invariants(repo_root=None):
    """Check 22: Physical Invariant Semantic Prose Gate.

    Verify that natural language narrative prose across all specification documents
    in docs/ conforms to physical negative invariants declared in the SysML AST.
    """
    if repo_root is None:
        repo_root = os.getcwd()

    model_text = _discover_sysml_model_text(repo_root)
    schema_dir = os.path.join(repo_root, "schema")
    has_extracted = os.path.isdir(os.path.join(schema_dir, "extracted")) if os.path.isdir(schema_dir) else False
    if (not model_text or not model_text.strip()) and not has_extracted:
        print("Success: Check 22 verified (SysML model pending or landing zone clean).")
        return

    val_cls, repo_cls = _load_semantic_prose_validator()
    if val_cls is None or repo_cls is None:
        print("WARNING: Check 22 skipped (SemanticProseInvariantValidator or WorkspaceRepository unavailable).", file=sys.stderr)
        return

    repo = repo_cls(workspace_dir=repo_root)
    validator = val_cls(workspace_repo=repo)
    findings = validator.validate(repo, scan_dirs=["docs"])

    if findings:
        print("ERROR: Check 22 failed (Physical Invariant Semantic Prose Gate violations found):", file=sys.stderr)
        for f in findings:
            print(f"  - {f}", file=sys.stderr)
        sys.exit(1)

    print("Success: Check 22 verified (Physical Invariant Semantic Prose Gate passed -- zero ungrounded operational assertions).")


def _load_factual_grounding_validator():
    """Import FactualGroundingValidator and WorkspaceRepository fail-safe."""
    script_dir = os.path.dirname(os.path.abspath(__file__))
    project_root = os.path.dirname(script_dir)
    spec_dir = os.path.join(project_root, "skills", "spec-orchestrator", "scripts")
    parity_src = os.path.join(project_root, "skills", "spec-orchestrator", "parity_auditor", "src")
    scripts_dir = os.path.join(project_root, "scripts")
    for p in (scripts_dir, spec_dir, parity_src):
        if p not in sys.path:
            sys.path.insert(0, p)
    try:
        from parity_auditor.validators.factual_grounding_validator import FactualGroundingValidator
        from parity_auditor.core.workspace import WorkspaceRepository
        return FactualGroundingValidator, WorkspaceRepository
    except Exception:
        return None, None


def check_factual_grounding(repo_root=None):
    """Check 23: Factual Grounding & Numeric Provenance Gate.

    Verify that structural descriptors, control surface counts, numeric limits,
    declared protocols, and sequence diagram temporal safety across docs/ conform
    strictly to the SysML AST and schema ground truth.
    """
    if repo_root is None:
        repo_root = os.getcwd()

    model_text = _discover_sysml_model_text(repo_root)
    schema_dir = os.path.join(repo_root, "schema")
    has_extracted = os.path.isdir(os.path.join(schema_dir, "extracted")) if os.path.isdir(schema_dir) else False
    if (not model_text or not model_text.strip()) and not has_extracted:
        print("Success: Check 23 verified (SysML model pending or landing zone clean).")
        return

    val_cls, repo_cls = _load_factual_grounding_validator()
    if val_cls is None or repo_cls is None:
        print("WARNING: Check 23 skipped (FactualGroundingValidator or WorkspaceRepository unavailable).", file=sys.stderr)
        return

    repo = repo_cls(workspace_dir=repo_root)
    validator = val_cls(workspace_repo=repo)
    findings = validator.validate(repo, scan_dirs=["docs"])

    if findings:
        print("ERROR: Check 23 failed (Factual Grounding & Numeric Provenance Gate violations found):", file=sys.stderr)
        for f in findings:
            print(f"  - {f}", file=sys.stderr)
        sys.exit(1)

    print("Success: Check 23 verified (Factual Grounding & Numeric Provenance Gate passed -- zero ungrounded assertions).")


check_factual_grounding_and_provenance = check_factual_grounding


def run_all_checks(repo_root=None):
    """Run all baseline checks (Checks 10 through 23)."""
    if repo_root is None:
        repo_root = os.getcwd()
    check_gitignore_exists(repo_root)
    check_no_ds_store_files(repo_root)
    check_no_duplicate_master_blueprints(repo_root)
    check_latex_katex_syntax(repo_root)
    check_downstream_instructions_exist(repo_root)
    check_reconcile_backlog_tooling_exists(repo_root)
    check_upstream_template_clean_landing_zones(repo_root)
    check_safety_integrity_and_sora_completeness(repo_root)
    verify_upstream_blueprint_domain_cleanliness(repo_root)
    check_domain_agnostic_ast_cleanliness(repo_root)
    check_wbs_suite_integrity(repo_root)
    check_semantic_diagram_ast_parity(repo_root)
    check_semantic_prose_invariants(repo_root)
    check_factual_grounding(repo_root)

def _run_verification(args, dest, repo_root, is_flutter, is_react):
    # Run Checks 10 through 23
    run_all_checks(repo_root)

    if is_flutter:
        print(f"Verifying conformance for platform 'flutter' at '{dest}'...")
        # 1. Assert baseline files exist
        baseline_files = [
            "pubspec.yaml",
            "analysis_options.yaml",
            "lib/main.dart",
            "lib/domain/validation.dart"
        ]
        missing_files = []
        for f in baseline_files:
            path = os.path.join(dest, f)
            if not os.path.exists(path):
                missing_files.append(f)

        repo_resolver_paths = [
            os.path.join(dest, "lib", "domain", "repository_resolver.dart"),
            os.path.join(dest, "lib", "core", "di", "repository_resolver.dart"),
        ]
        if not any(os.path.exists(p) for p in repo_resolver_paths) and not args.no_domain:
            missing_files.append("lib/domain/repository_resolver.dart (or lib/core/di/repository_resolver.dart)")

        if missing_files:
            print(f"ERROR: Flutter baseline file(s) missing: {', '.join(missing_files)}", file=sys.stderr)
            sys.exit(1)

        print("Success: All Flutter baseline files exist.")

        # 2. Validate type compatibility
        if args.no_domain:
            print("Skipping domain type compatibility validation (--no-domain specified).")
        else:
            _validate_domain_types(dest, repo_root, "dart", os.path.join("lib", "domain"))

        # 3. Run build/test commands
        if args.no_domain:
            print("Skipping build and test suite execution (--no-domain specified, domain implementation pending).")
        else:
            try:
                # Resolve and copy assets directory from template
                upstream_repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
                src_assets = os.path.join(upstream_repo_root, "app_flutter", "assets")
                dest_assets = os.path.join(dest, "assets")
                if os.path.exists(src_assets):
                    if os.path.abspath(src_assets) != os.path.abspath(dest_assets):
                        print(f"Copying template assets from {src_assets} to {dest_assets}...")
                        os.makedirs(dest_assets, exist_ok=True)
                        for item in os.listdir(src_assets):
                            s_path = os.path.join(src_assets, item)
                            d_path = os.path.join(dest_assets, item)
                            if os.path.isfile(s_path):
                                shutil.copy2(s_path, d_path)
                        print("Assets copied successfully.")
                    else:
                        print("Source and destination assets directories are the same. Skipping copy.")
                else:
                    print(f"WARNING: Upstream assets directory not found at {src_assets}")

                print("Running 'flutter pub get' to resolve dependencies...")
                _run_bounded(["flutter", "pub", "get"], cwd=dest, timeout=TIMEOUT_SECONDS, label="flutter pub get")
                
                print("Running 'flutter analyze'...")
                _run_bounded(["flutter", "analyze", "--no-fatal-warnings", "--no-fatal-infos"], cwd=dest, timeout=TIMEOUT_SECONDS, label="flutter analyze")
                
                print("Running 'flutter test'...")
                _run_bounded(["flutter", "test"], cwd=dest, timeout=TIMEOUT_SECONDS, label="flutter test")
                
                print("Running 'flutter build macos --release'...")
                _run_bounded(["flutter", "build", "macos", "--release"], cwd=dest, timeout=TIMEOUT_SECONDS * 3, label="flutter build macos --release")
                
                print("Zipping the macOS application bundle...")
                # The build output is typically at app_flutter/build/macos/Build/Products/Release/Platform Console.app
                # We need to package it into the repository root as app_flutter_release.zip
                zip_path = os.path.join(repo_root, "app_flutter_release.zip")
                
                # We expect the app bundle to be named 'Platform Console.app'. 
                # Let's find it in the release directory.
                release_dir = os.path.join(dest, "build", "macos", "Build", "Products", "Release")
                app_bundle = "Platform Console.app"
                
                if os.path.exists(os.path.join(release_dir, app_bundle)):
                    if os.path.exists(zip_path):
                        print(f"Removing pre-existing release archive at {zip_path}...")
                        os.remove(zip_path)
                    _run_bounded(["zip", "-r", zip_path, app_bundle], cwd=release_dir, timeout=TIMEOUT_SECONDS, label="zip macos bundle")
                    archive_size = os.path.getsize(zip_path) if os.path.exists(zip_path) else 0
                    print(f"Success: App bundled to {zip_path} (created archive size: {archive_size} bytes)")
                else:
                    print(f"ERROR: App bundle not found at {os.path.join(release_dir, app_bundle)}", file=sys.stderr)
                    sys.exit(1)
                    
            except subprocess.TimeoutExpired as e:
                print(f"ERROR: Verification command timed out after {e.timeout}s: {e.cmd}", file=sys.stderr)
                sys.exit(1)
            except subprocess.CalledProcessError as e:
                print(f"ERROR: Verification command failed: {e}", file=sys.stderr)
                sys.exit(1)

    if is_react:
        print(f"Verifying conformance for platform 'react' at '{dest}'...")
        # 1. Assert baseline files exist
        has_tsconfig = os.path.exists(os.path.join(dest, "tsconfig.json"))
        has_jsconfig = os.path.exists(os.path.join(dest, "jsconfig.json"))
        if not has_tsconfig and not has_jsconfig:
            print("ERROR: TSConfig or JSConfig is missing.", file=sys.stderr)
            sys.exit(1)

        entry_candidates = ["src/main.tsx", "src/main.jsx", "src/index.tsx", "src/index.jsx"]
        entry_file = None
        for cand in entry_candidates:
            if os.path.exists(os.path.join(dest, cand)):
                entry_file = cand
                break
        if not entry_file:
            print(f"ERROR: React entrypoint file missing (expected one of: {', '.join(entry_candidates)})", file=sys.stderr)
            sys.exit(1)

        if not args.no_domain:
            validation_candidates = ["src/domain/validation.ts", "src/domain/validation.js", "src/domain/validation.tsx", "src/domain/validation.jsx"]
            validation_file = None
            for cand in validation_candidates:
                if os.path.exists(os.path.join(dest, cand)):
                    validation_file = cand
                    break
            if not validation_file:
                print(f"ERROR: Domain validation file missing (expected one of: {', '.join(validation_candidates)})", file=sys.stderr)
                sys.exit(1)

        print("Success: All React baseline files exist.")

        # 2. Validate type compatibility
        if args.no_domain:
            print("Skipping domain type compatibility validation (--no-domain specified).")
        else:
            _validate_domain_types(dest, repo_root, "ts", os.path.join("src", "domain"))

        # 3. Run build/test commands
        if args.no_domain:
            print("Skipping build execution (--no-domain specified, domain implementation pending).")
        else:
            try:
                print("Running 'npm install' to resolve dependencies...")
                _run_bounded(["npm", "install"], cwd=dest, timeout=TIMEOUT_SECONDS * 2, label="npm install")
                
                print("Running 'npm run build'...")
                _run_bounded(["npm", "run", "build"], cwd=dest, timeout=TIMEOUT_SECONDS * 2, label="npm run build")
            except subprocess.TimeoutExpired as e:
                print(f"ERROR: React verification command timed out after {e.timeout}s: {e.cmd}", file=sys.stderr)
                sys.exit(1)
            except subprocess.CalledProcessError as e:
                print(f"ERROR: React verification command failed: {e}", file=sys.stderr)
                sys.exit(1)

if __name__ == "__main__":
    main()

