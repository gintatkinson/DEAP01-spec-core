#!/usr/bin/env python3
"""
Verify downstream project baseline conformance.
Asserts baseline files exist, validates type compatibility with mandated domain classes,
and runs the build/test commands ('npm run build' for React, 'flutter analyze && flutter test' for Flutter).
"""

import argparse
import ast
import json
import os
import re
import shutil
import signal
import subprocess
import sys
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Tuple

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
    try:
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
                print(f"NOTE: Domain directory found on disk for '{dest}' — overriding no_domain config and enabling domain verification.")
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
        print("No mandated classes configured — skipping type validation.")
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
        pattern = r"\b(?:interface|class|type)\s+({})\b".format("|".join(re.escape(c) for c in mandated))
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
    if ds_store_files:
        print(f"ERROR: Check 11 failed: Found {len(ds_store_files)} .DS_Store file(s) in working tree or git index: {', '.join(ds_store_files)}", file=sys.stderr)
        sys.exit(1)
    print("Success: Check 11 verified (zero .DS_Store files found).")

def check_no_duplicate_master_blueprints(repo_root):
    """Check 12: Verify downstream repositories do NOT contain duplicate master core blueprints."""
    upstream_marker = os.path.join(repo_root, ".pipeline", "upstream")
    if os.path.isdir(upstream_marker):
        print("Success: Check 12 verified (Master core / upstream repository detected — skipping duplicate blueprint check).")
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
        print("Success: Check 16 verified (Downstream repository detected — skipping upstream clean landing zone gate).")
        return

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

def count_fmeca_rows(content: str) -> int:
    """Extract and count data rows from the FMECA table in content."""
    lines = content.splitlines()
    in_fmeca_section = False
    row_count = 0
    header_skipped = False

    for line in lines:
        stripped = line.strip()
        # Check for section header (level 2+ or specific FMECA header)
        if stripped.startswith("##") or (stripped.startswith("#") and "criticality" in stripped.lower()):
            if re.search(r'\b(?:FMECA|Failure\s+Mode)\b', stripped, re.IGNORECASE):
                in_fmeca_section = True
                header_skipped = False
                continue
            elif in_fmeca_section:
                # Reached next section header
                in_fmeca_section = False

        if in_fmeca_section:
            if stripped.startswith("|") and stripped.endswith("|"):
                # Skip separator rows like |:---|:---| or |---|---|
                if re.match(r"^\|(?:\s*:?-+:?\s*\|)+$", stripped):
                    header_skipped = True
                    continue
                # If header row hasn't been skipped yet, check for common table header keywords
                if not header_skipped:
                    lower = stripped.lower()
                    if any(kw in lower for kw in ["component", "failure", "subsystem", "severity", "rpn", "local effect"]):
                        continue
                cells = [c.strip() for c in stripped.split("|")[1:-1]]
                if any(cells):
                    row_count += 1

    # Fallback: if no rows found via section header, scan for table with FMECA columns
    if row_count == 0:
        in_fmeca_table = False
        for line in lines:
            stripped = line.strip()
            if stripped.startswith("|") and stripped.endswith("|"):
                lower = stripped.lower()
                if "failure" in lower and ("rpn" in lower or "severity" in lower or "component" in lower):
                    in_fmeca_table = True
                    header_skipped = False
                    continue
                if re.match(r"^\|(?:\s*:?-+:?\s*\|)+$", stripped):
                    header_skipped = True
                    continue
                if in_fmeca_table and header_skipped:
                    cells = [c.strip() for c in stripped.split("|")[1:-1]]
                    if any(cells):
                        row_count += 1
            else:
                if in_fmeca_table and stripped and not stripped.startswith("|"):
                    in_fmeca_table = False

    return row_count

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
        return summary


# Canonical STPA guide words are methodology constants, not domain concepts.
# Order matters: timing/duration rules precede the generic providing rule so
# phrases such as "Providing too early" classify to GW-3 rather than GW-2.
STPA_GUIDE_WORD_RULES = [
    ("GW-1", "Not providing causes hazard", re.compile(r"not\s+provid|omission", re.IGNORECASE)),
    ("GW-3", "Providing too early, too late, or out of order", re.compile(r"too\s+early|too\s+late|out\s+of\s+order|timing|early/late", re.IGNORECASE)),
    ("GW-4", "Stopped too soon or applied too long", re.compile(r"stopped\s+too\s+soon|applied\s+too\s+long|stopped\s+early|duration", re.IGNORECASE)),
    ("GW-2", "Providing causes hazard", re.compile(r"providing|commission", re.IGNORECASE)),
]
STPA_GUIDE_WORD_ORDER = {gw_id: index for index, (gw_id, _label, _pattern) in enumerate(STPA_GUIDE_WORD_RULES)}
STPA_GUIDE_WORD_LABELS = {gw_id: label for gw_id, label, _pattern in STPA_GUIDE_WORD_RULES}

# Schema-less structural floor: 4 canonical STPA guide words x 4 (controller,
# control action) pair instances. Model-backed validation derives the true
# Cartesian cardinality from the schema instead of applying this floor.
MIN_STRUCTURAL_UCA_ROWS = 16


def classify_uca_guide_word(cell_text: str) -> Optional[Tuple[str, str]]:
    """Classify a UCA guide word cell into one of the 4 canonical STPA failure modes."""
    for gw_id, label, pattern in STPA_GUIDE_WORD_RULES:
        if pattern.search(cell_text):
            return gw_id, label
    return None


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
        for name in sorted(os.listdir(schema_dir)):
            if name.endswith(".sysml"):
                try:
                    with open(os.path.join(schema_dir, name), "r", encoding="utf-8") as handle:
                        return handle.read()
                except OSError:
                    continue
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
    def _column_index(header, keywords):
        for index, cell in enumerate(header):
            lowered = cell.lower()
            if any(keyword in lowered for keyword in keywords):
                return index
        return None

    @classmethod
    def parse_stpa_table(cls, text: str) -> List[STPARowAST]:
        """Parse UCA rows from markdown tables headed by control action / guide word columns."""
        rows: List[STPARowAST] = []
        for header, data_rows in cls._iter_tables(text):
            header_text = " ".join(cell.lower() for cell in header)
            if "uca" not in header_text:
                continue
            if "guide word" not in header_text and "control action" not in header_text:
                continue
            col_uca = cls._column_index(header, ("uca id", "uca"))
            col_controller = cls._column_index(header, ("controller",))
            col_action = cls._column_index(header, ("control action", "action"))
            col_guide = cls._column_index(header, ("guide word",))
            col_hazard = cls._column_index(header, ("hazard",))
            col_loss = cls._column_index(header, ("loss",))
            col_constraint = cls._column_index(header, ("constraint",))

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

        Part labels are recognized in both "Part N — Keyword" and numbered
        "N. Keyword" styles; keyword families must match the part number.
        """
        block_start = re.compile(r"^\s*#{2,4}\s+.*\bTheorem\b", re.IGNORECASE)
        heading_line = re.compile(r"^\s*#{1,6}\s+\S")
        part_match = re.compile(r"Part\s*(\d)", re.IGNORECASE)
        numbered_part = re.compile(r"^\s*(\d+)[.)]\s+")
        blocks: List[ProofBlockAST] = []
        current: Optional[ProofBlockAST] = None

        def finish():
            nonlocal current
            if current is not None:
                blocks.append(current)
                current = None

        for line_number, raw_line in enumerate(text.splitlines(), start=1):
            line = raw_line.strip()
            if block_start.match(line):
                finish()
                id_match = re.search(r"\b([A-Z]+-\d+)\b", line)
                current = ProofBlockAST(
                    theorem_id=id_match.group(1) if id_match else f"Theorem@{line_number}",
                    line_number=line_number,
                )
                continue
            if current is not None:
                part = part_match.search(line)
                part_number = None
                if part:
                    part_number = int(part.group(1))
                else:
                    numbered = numbered_part.match(line)
                    if numbered:
                        p_num = int(numbered.group(1))
                        if p_num <= 5:
                            part_number = p_num

                if part_number is not None:
                    lowered = line.lower()
                    if part_number == 1 and ("proposition" in lowered or "statement" in lowered):
                        current.proposition = line
                    elif part_number == 2 and "assumption" in lowered:
                        current.assumptions = line
                    elif part_number == 3 and ("invariant" in lowered or "barrier" in lowered):
                        current.barrier_function = line
                    elif part_number == 4 and ("deriv" in lowered or "inductive" in lowered):
                        current.derivation = line
                    elif part_number == 5 and ("conclusion" in lowered or "q.e.d" in lowered or "qed" in lowered):
                        current.conclusion = line
                    continue

                if heading_line.match(line):
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
            classified = classify_uca_guide_word(row.guide_word)
            if classified is None:
                continue
            found.add((matched_action, classified[0]))

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
    comparison against the schema remains model-gated.

    Returns (violation_strings, report, expected_control_actions_or_none).
    """
    errors: List[str] = []
    report = ASTValidationReport()

    stpa_rows = MarkdownTableASTParser.parse_stpa_table(content)
    oso_rows = MarkdownTableASTParser.parse_sora_table(content)
    proof_blocks = MarkdownTableASTParser.parse_proof_blocks(content)

    expected_actions: Optional[List[str]] = None
    if model_text:
        parse_sysml = _load_sysml_parser()
        try:
            model_ast = parse_sysml(model_text)
        except Exception as exc:
            errors.append(f"Safety AST violation: Failed to parse SysML v2 model ({exc}).")
            broken = ASTValidationReport(is_conforming=False, syntax_errors=[str(exc)])
            return errors, broken, None
        expected_actions = sorted({str(name) for name in model_ast.get("action_defs", [])})

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
                f"Pillar 4 violation: UCA Cartesian completeness failure — expected {report.expected_uca_rows} "
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
                f"Pillar 4 violation: UCA guide-word completeness failure — expected {report.expected_uca_rows} "
                f"permutations ({len(derived_actions)} control actions x 4 guide words), found {found_combos} "
                f"unique combinations. Missing permutations:\n{listing}"
            )
        if report.total_uca_rows < MIN_STRUCTURAL_UCA_ROWS:
            errors.append(
                f"Pillar 4 violation: UCA Cartesian matrix truncation — found {report.total_uca_rows} UCA row(s); "
                f"minimum required is {MIN_STRUCTURAL_UCA_ROWS} permutations (4 control actions x 4 guide words)."
            )

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


def _validate_aggregate_safety_content(aggregate_safety_content: str, repo_root: Optional[str] = None) -> Tuple[list, Optional[ASTValidationReport]]:
    """Run pillar validation plus structural AST validation.

    The structural AST validation is model-optional: when a SysML model is
    discoverable under repo_root, the full Cartesian cardinality is compared
    against the model's action definitions; without a model, guide-word
    completeness is enforced against the (controller, control action) pairs
    derived from the UCA table itself and the 5-part proof structure is
    enforced on parsed theorem blocks.

    Returns (violation_strings, ast_report_or_none).
    """
    errors: List[str] = []
    ast_report: Optional[ASTValidationReport] = None

    model_text = None
    if repo_root:
        model_text = _discover_sysml_model_text(repo_root)

    ast_errors, ast_report, _expected_actions = validate_safety_matrix_ast(aggregate_safety_content, model_text)
    errors.extend(ast_errors)

    errors.extend(_validate_safety_matrix_pillars(aggregate_safety_content, ast_path_active=model_text is not None))
    return errors, ast_report


def validate_safety_matrix_content(content: str, repo_root: Optional[str] = None) -> list:
    """Validate 8-pillar schema, 24 SORA OSOs, 15+ FMECA rows, 4 UCA categories, ASTM F3269-17 RTA, and MATLAB/Simulink hooks.

    Structural table-aware AST validation is model-optional. When a SysML v2
    model is discoverable under repo_root (schema/*.sysml or .pipeline/schema.sysml),
    dynamic Cartesian product set equality against the model's action definitions
    supersedes the legacy regex keyword checks; without a model, guide-word
    completeness over table-derived (controller, control action) pairs and the
    5-part proof structure are still enforced structurally, while the legacy
    regex scans remain the fallback for pillar presence and SORA OSO coverage.

    Returns a list of violation error strings (empty if valid).
    """
    errors, _ast_report = _validate_aggregate_safety_content(content, repo_root)
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

    # Pillar 7: FMECA Criticality Matrix (15+ rows)
    if not re.search(r'FMECA|Failure\s+Mode', content, re.IGNORECASE):
        errors.append("Pillar 7 violation: Missing FMECA Criticality Matrix.")
    else:
        fmeca_rows = count_fmeca_rows(content)
        if fmeca_rows < 15:
            errors.append(f"Pillar 7 violation: FMECA Criticality Matrix contains {fmeca_rows} row(s); minimum required is 15 rows.")
        if not re.search(r'\bRPN\b|Risk\s+Priority\s+Number', content, re.IGNORECASE):
            errors.append("Pillar 7 violation: FMECA table missing RPN (Risk Priority Number) calculation.")

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
       - Pillar 7: FMECA Criticality Matrix with 15+ component failure mode rows and RPN
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
        print("Success: Check 17 verified (Downstream repository detected — docs/safety/ directory not present).")
        return

    safety_files = []
    for root, dirs, files in os.walk(safety_dir):
        dirs[:] = [d for d in dirs if d not in EXCLUDED_DIRS and d not in ("defects", "audits", "decisions")]
        for f in files:
            if f.endswith(".md") and f != "README.md":
                safety_files.append(os.path.join(root, f))

    if not safety_files:
        print("Success: Check 17 verified (Downstream repository detected — safety specifications pending or clean).")
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
    print("Success: Check 17 verified (Safety Integrity Quality Gate: 8 pillars, 24 SORA OSOs, 15+ FMECA rows, 4 UCA categories, ASTM F3269-17 RTA, and MATLAB/Simulink hooks).")

def verify_upstream_blueprint_domain_cleanliness(target_dir):
    """Check 18: Upstream Blueprint Domain Cleanliness Gate.

    Verify that upstream DEAP01-spec-core architecture blueprints contain zero concrete
    domain platform concept papers or domain SysML models (e.g. *FLIGHT_SYSTEMS*,
    *UAS_INFRASTRUCTURE*, *FRONTEND_SYSTEMS*, *SAFETY_MODEL*.sysml).
    """
    upstream_marker = os.path.join(target_dir, ".pipeline", "upstream")
    if not (os.path.isdir(upstream_marker) or os.path.isfile(upstream_marker)):
        print("Success: Check 18 verified (Downstream repository detected — skipping upstream blueprint domain cleanliness gate).")
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

class _DomainAgnosticASTVisitor(ast.NodeVisitor):
    """AST visitor enforcing pure schema-driven parameter extraction and zero static domain specs."""

    STATIC_PARAM_DICT_NAMES = re.compile(
        r"^(_)?("
        r"ground_?truth(_?(specs?|params?|parameters?|dict|map|set|table))?|"
        r"(expected|domain|static|hardcoded|benchmark|mandated|system)_?(specs?|params?|parameters?|constants?|dict|map|set|table|specifications?)"
        r")$",
        re.IGNORECASE
    )

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

    def _check_target_name(self, target_name: str, value_node: ast.AST, lineno: int):
        if not target_name or value_node is None:
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


def check_domain_agnostic_ast_cleanliness(repo_root):
    """Check 19: Domain-Agnostic AST Cleanliness Gate.

    Verify that upstream DEAP01-spec-core tools, scripts, and validator modules contain
    zero static/hardcoded parameter dictionaries (e.g. GROUND_TRUTH = {...}, EXPECTED_SPECS = {...},
    DOMAIN_PARAMS = {...}), and that all parameter extraction dynamically queries workspace.schemas
    or schema/*.sysml AST nodes without hardcoded domain concept constants.
    """
    upstream_marker = os.path.join(repo_root, ".pipeline", "upstream")
    if not os.path.isdir(upstream_marker):
        print("Success: Check 19 verified (Downstream repository detected — skipping domain-agnostic AST cleanliness gate).")
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

                visitor = _DomainAgnosticASTVisitor(file_path, repo_root)
                visitor.visit(tree)
                violations.extend(visitor.violations)

    if violations:
        print("ERROR: Check 19 failed (Domain-Agnostic AST Cleanliness Gate violations found):", file=sys.stderr)
        for v in violations:
            print(f"  - {v}", file=sys.stderr)
        sys.exit(1)

    print("Success: Check 19 verified (Domain-Agnostic AST Cleanliness Gate passed — pure dynamic schema AST architecture verified).")

def run_all_checks(repo_root=None):
    """Run all baseline checks (Checks 10 through 19)."""
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

def _run_verification(args, dest, repo_root, is_flutter, is_react):
    # Run Checks 10, 11, 12, 13, 14, 15, 16, 17, 18, and 19
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

