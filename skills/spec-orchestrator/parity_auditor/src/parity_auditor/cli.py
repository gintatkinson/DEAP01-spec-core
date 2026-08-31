"""
CLI entry point for the Model Coverage Parity Audit tool.

Parses command-line arguments, locates the workspace, initialises all
validators, and orchestrates the full audit pipeline.  Delegates to
UML-, behavioral-, codebase-, docs-, dependency-, sync-, schema-mapping-,
profile-scoping- and test-completeness validators.
"""

import os
import sys
import re
import argparse
import json
import shutil
from typing import Set, List

from .core.workspace import WorkspaceRepository, extract_metadata_from_content
from .parsers.schema_router import parse_schema_file
from .validators.uml import UmlValidator
from .validators.behavioral import BehavioralValidator
from .validators.codebase import CodebaseValidator
from .validators.docs import DocsValidator
from .validators.dependency_validator import DependencyValidator
from .validators.sync_validator import SyncValidator
from .validators.schema_mapping_validator import SchemaMappingValidator
from .validators.profile_scoping_validator import ProfileScopingValidator
from .validators.test_completeness_validator import TestCompletenessValidator
from .validators.logical_ui_validator import LogicalUiValidator
from .validators.cardinality_validator import SchemaCardinalityValidator
from .validators.mermaid_syntax_validator import MermaidSyntaxValidator
from .validators.katex_validator import KatexValidator
from .validators.spec_filename_validator import SpecFilenameValidator
from .validators.spec_title_uniqueness_validator import SpecTitleUniquenessValidator
from .validators.source_reference_validator import SourceReferenceValidator
from .validators.link_validator import LinkValidator
from .validators.docstring_validator import DocstringValidator
from .validators.profile_compliance_validator import ProfileComplianceValidator
from .validators.concept_provenance_validator import ConceptProvenanceValidator
from .validators.safety_trace_validator import SafetyTraceValidator
from .validators.doc_metadata_validator import DocMetadataValidator
from .utils.diagnostics import serialize_diagnostics
from .utils.comment_utils import strip_comments_and_strings

def sanitize_github_token_env():
    """
    Sanitize environment by removing dummy or placeholder GITHUB_TOKEN and GH_TOKEN
    values that interfere with git/gh terminal operations.
    """
    dummy_keywords = ("antigravity", "dummy", "placeholder", "invalid", "mock")
    for var in ("GITHUB_TOKEN", "GH_TOKEN"):
        val = os.environ.get(var)
        if val and any(kw in val.lower() for kw in dummy_keywords):
            os.environ.pop(var, None)

# NOTE: sanitize_github_token_env() is deliberately NOT called at module level.
# Doing so mutated os.environ on import, so importing this module stripped
# GITHUB_TOKEN/GH_TOKEN from the process and unrelated tests failed depending on
# import order (issue #276). Both real entry points, main() and _main_impl(),
# already invoke it, so nothing is lost.

def assert_no_mock_cli(workspace_dir: str = None):
    if not workspace_dir:
        curr = os.getcwd()
        while True:
            if os.path.exists(os.path.join(curr, ".pipeline", "logical-ui", "codebase_rules.json")):
                workspace_dir = curr
                break
            parent = os.path.dirname(curr)
            if parent == curr:
                break
            curr = parent
        if not workspace_dir:
            workspace_dir = os.getcwd()

    workspace_dir = os.path.abspath(workspace_dir)
    scratch_dir = os.path.abspath(os.path.join(workspace_dir, "scratch"))
    scratch_bin = os.path.join(scratch_dir, "bin")
    forbidden_cmds = ["gh", "git", "flutter"]

    for cmd in forbidden_cmds:
        binary_path = os.path.join(scratch_bin, cmd)
        if os.path.exists(binary_path):
            print(f"[FATAL] Zero-mocking policy violation: Forbidden mock CLI binary detected at {binary_path}", file=sys.stderr)
            sys.exit(1)

        resolved = shutil.which(cmd)
        if resolved:
            resolved_abs = os.path.abspath(resolved)
            if resolved_abs.startswith(scratch_dir + os.sep) or resolved_abs == scratch_dir:
                print(f"[FATAL] Zero-mocking policy violation: Forbidden mock CLI binary detected at {resolved_abs}", file=sys.stderr)
                sys.exit(1)


def get_open_feature_issues(workspace_dir: str = None):
    """
    Fetch open feature issues from GitHub via ``gh issue list``.

    Filters out issues whose title contains known defect/bug/tooling keywords.

    Returns:
        List of issue dicts with 'number' and 'title' keys, or None when the
        ``gh`` CLI is unavailable, offline, returns a non-zero exit code, or times out.
    """
    assert_no_mock_cli(workspace_dir)

    if os.environ.get("OFFLINE") or not shutil.which("gh"):
        return None

    import subprocess
    import json
    try:
        timeout = float(os.environ.get("PARITY_AUDITOR_GH_TIMEOUT", "3.0"))
    except (ValueError, TypeError):
        timeout = 3.0

    try:
        result = subprocess.run(
            ["gh", "issue", "list", "--state", "open", "--label", "feature", "--json", "number,title"],
            capture_output=True,
            text=True,
            timeout=timeout
        )
        if result.returncode == 0:
            issues = json.loads(result.stdout)
            keywords = ["defect", "bug", "repro", "tooling"]
            return [
                issue for issue in issues
                if not any(kw in issue.get("title", "").lower() for kw in keywords)
            ]
        else:
            print(f"ERROR: gh CLI exited with code {result.returncode}: {result.stderr.strip()}", file=sys.stderr)
            return None
    except subprocess.TimeoutExpired:
        print(f"ERROR: gh CLI timed out after {timeout} seconds.", file=sys.stderr)
        return None
    except Exception as e:
        print(f"ERROR: Failed to run gh CLI to fetch open feature issues: {e}", file=sys.stderr)
        return None

def parse_ignore_issues(ignore_str: str) -> set:
    ignored = set()
    if not ignore_str:
        return ignored
    for part in ignore_str.split(","):
        part = part.strip()
        if not part:
            continue
        if "-" in part:
            try:
                start, end = part.split("-", 1)
                ignored.update(range(int(start), int(end) + 1))
            except ValueError:
                pass
        else:
            try:
                ignored.add(int(part))
            except ValueError:
                pass
    return ignored


def _extract_issue_id_from_frontmatter(fm_text: str, issue_number: int) -> bool:
    data = extract_metadata_from_content(fm_text)
    if data:
        val = data.get("issue_id")
        if val is not None and int(val) == issue_number:
            return True
    try:
        import yaml
        data = yaml.safe_load(fm_text.replace('\x01', ''))
        if isinstance(data, dict):
            val = data.get("issue_id")
            if val is not None and int(val) == issue_number:
                return True
    except Exception:
        pass
    try:
        m = re.search(r'(?:^|\n)\s*issue_id\s*:\s*(\d+)', fm_text)
        if m and int(m.group(1)) == issue_number:
            return True
    except Exception:
        pass
    return False


def _scope_findings(errors, only):
    """Findings naming ``only``; everything else is another item's problem.

    Whole-corpus invariants still ran — a duplicate title or a dangling cross-reference
    is only visible across files — but a finding that does not name this item belongs to
    a different one. Matching on the basename keeps a collision report, which names every
    colliding file, visible to each of them (issues #331, #321).
    """
    if not only:
        return errors
    target = os.path.basename(str(only)).strip()
    if not target:
        return errors
    return [e for e in errors if target in str(e)]


def _main_impl():
    """
    Orchestrate the full parity audit pipeline.

    Discovers the workspace, resolves schema/features/epics directories,
    parses schemas and feature files, then runs each validator in sequence
    (UML, behavioral, codebase AST, docs, dependencies, sync, schema
    mapping, profile scoping, test completeness).  Exits with code 1 on
    any failure, writing a diagnostics JSON artifact.

    Side effects:
        - Reads codebase_rules.json for configuration.
        - Invokes ``gh`` CLI for open-feature-issue discovery.
        - Serialises diagnostics JSON to ``.pipeline/logical-ui/`` on failure.
        - Prints audit progress and results to stdout.
    """
    sanitize_github_token_env()
    parser = argparse.ArgumentParser(description="Model Coverage Parity Audit CLI")
    parser.add_argument("schema_dir", nargs="?", help="Path to schema directory")
    parser.add_argument("features_dir", nargs="?", help="Path to feature specs directory")
    parser.add_argument("--spec-only", action="store_true", help="Run in specification-only mode, bypassing codebase checks")
    parser.add_argument("--allow-missing-specs", action="store_true", default=True, help="Skip exiting with status code 1 when there are missing specification files")
    parser.add_argument("--no-allow-missing-specs", dest="allow_missing_specs", action="store_false", help="Exit with error code when specification files are missing (strict mode)")
    parser.add_argument("--ignore-issues", help="Comma-separated list of issue numbers or ranges to ignore (e.g., 14,16-18)")
    parser.add_argument("--only", metavar="SPEC",
                        help="Scope reported findings to a single specification "
                             "(file name or path). Whole-corpus invariants still "
                             "run - uniqueness and cross-references cannot be "
                             "checked per file - but only findings naming this "
                             "item are reported. Lets a single-item gate be strict "
                             "without blocking on unrelated drafts (issues #331, #321).")
    parser.add_argument("--scope-all", action="store_true", help="Check against ALL open feature issues (entire repo, not just local specs)")
    parser.add_argument("--sysml", action="store_true", help="Run SysML v2 model coverage parity validation")
    
    args = parser.parse_args()
    
    # 1. Locate workspace directory dynamically starting from current working directory
    workspace_dir = None
    curr = os.getcwd()
    while True:
        if os.path.exists(os.path.join(curr, ".pipeline", "logical-ui", "codebase_rules.json")):
            workspace_dir = curr
            break
        parent = os.path.dirname(curr)
        if parent == curr:
            break
        curr = parent
        
    # Fall back to script's directory traversal if not found in cwd hierarchy
    if not workspace_dir:
        script_dir = os.path.dirname(os.path.abspath(__file__))
        curr = script_dir
        while True:
            if os.path.exists(os.path.join(curr, ".pipeline", "logical-ui", "codebase_rules.json")):
                workspace_dir = curr
                break
            parent = os.path.dirname(curr)
            if parent == curr:
                break
            curr = parent
            
    if not workspace_dir:
        workspace_dir = os.getcwd()
        
    workspace_dir = os.path.abspath(workspace_dir)
    assert_no_mock_cli(workspace_dir)
    
    # 2. Initialize WorkspaceRepository with the determined workspace_dir
    repo = WorkspaceRepository(workspace_dir)
    
    # 3. Check if the codebase rules file exists
    rules_path = repo.get_codebase_rules_path()
    if not os.path.exists(rules_path):
        print(f"Error: codebase_rules.json not found at: {rules_path}")
        print("Please ensure the configuration file is present at '.pipeline/logical-ui/codebase_rules.json'.")
        sys.exit(1)
        
    # 4. Check if rules is empty or invalid, or if rules.meta.upstream_repository is empty
    try:
        with open(rules_path, "r", encoding="utf-8") as f:
            json.load(f)
    except Exception:
        print("Error: Configuration is empty, invalid, or missing required metadata.")
        print("Please check '.pipeline/logical-ui/codebase_rules.json' and ensure it has a valid 'meta.upstream_repository' set.")
        sys.exit(1)
        
    rules = repo.get_codebase_rules()
    if not rules or not rules.meta or not rules.meta.upstream_repository:
        print("Error: Configuration is empty, invalid, or missing required metadata.")
        print("Please check '.pipeline/logical-ui/codebase_rules.json' and ensure it has a valid 'meta.upstream_repository' set.")
        sys.exit(1)
        
    backlog_dirs = rules.backlog_directories
    
    schema_dir = args.schema_dir
    if not schema_dir:
        schema_dir = os.environ.get("SCHEMA_DIR")
    if not schema_dir:
        schema_dir_rel = backlog_dirs.schemas
        if not schema_dir_rel:
            raise ValueError("Missing 'backlog_directories.schemas' in codebase_rules.json")
        schema_dir = os.path.join(repo.workspace_dir, schema_dir_rel)
    else:
        schema_dir = os.path.abspath(schema_dir)
        
    features_dir = args.features_dir
    if not features_dir:
        features_dir = os.environ.get("FEATURES_DIR")
    if not features_dir:
        features_dir_rel = backlog_dirs.features
        if not features_dir_rel:
            raise ValueError("Missing 'backlog_directories.features' in codebase_rules.json")
        features_dir = os.path.join(repo.workspace_dir, features_dir_rel)
    else:
        features_dir = os.path.abspath(features_dir)
        
    epics_dir_rel = backlog_dirs.epics
    epics_dir = os.path.join(repo.workspace_dir, epics_dir_rel) if epics_dir_rel else None
        
    has_failed = False
    print("=== Model Coverage Parity Audit ===")
    print(f"Scanning schemas in: {schema_dir}")
    print(f"Scanning feature specifications in: {features_dir}\n")
    
    # 1. Parse all modules
    modules = {}
    module_sources = {}
    if os.path.exists(schema_dir):
        for filename in os.listdir(schema_dir):
            filepath = os.path.join(schema_dir, filename)
            if os.path.isdir(filepath):
                continue
            try:
                module_name, definitions = parse_schema_file(filepath)
                if module_name:
                    if module_name in modules:
                        print(
                            f"Warning: Duplicate module name '{module_name}' found in "
                            f"'{filename}' (previously defined in '{module_sources[module_name]}'). "
                            f"Definitions from '{module_sources[module_name]}' will be overwritten.",
                            file=sys.stderr
                        )
                    modules[module_name] = definitions
                    module_sources[module_name] = filename
            except Exception as e:
                print(f"Warning: Failed to parse schema file {filename}: {e}")
                
    # A second pass over schema_dir used to classify files as parseable or
    # merely alternative-extension. Both flags were dead: nothing read them.
    # The surviving guard is the `all_definitions` emptiness check below, which
    # keys on what was actually parsed rather than on what could have been.
    # Removed in issue #303.

    # 2. Load all feature markdown files
    features = repo.get_feature_files(features_dir)
    print(f"Loaded {len(features)} feature specifications.\n")
    
    # Cross-reference local docs/features/ spec files against open feature issues fetched via gh CLI
    ignored_set = set()
    if args.ignore_issues:
        ignored_set.update(parse_ignore_issues(args.ignore_issues))
    rule_ignore = rules.tracker_rules.get("ignore_issues")
    if rule_ignore:
        if isinstance(rule_ignore, list):
            for ri in rule_ignore:
                ignored_set.update(parse_ignore_issues(str(ri)))
        else:
            ignored_set.update(parse_ignore_issues(str(rule_ignore)))

    open_issues = get_open_feature_issues(workspace_dir)
    if open_issues is None:
        if not args.allow_missing_specs:
            has_failed = True
            print("[!] ERROR: Could not fetch open feature issues from GitHub while --no-allow-missing-specs is enabled.", file=sys.stderr)
            open_issues = []
        else:
            print("[!] WARNING: Could not fetch open feature issues from GitHub. Cross-reference verification skipped.", file=sys.stderr)
            open_issues = []

    if ignored_set:
        open_issues = [issue for issue in open_issues if issue.get("number") not in ignored_set]

    if not args.scope_all:
        local_issue_ids = set()
        for f in features:
            fm_data = extract_metadata_from_content(f.content)
            if fm_data and "issue_id" in fm_data:
                try:
                    local_issue_ids.add(int(fm_data["issue_id"]))
                except (ValueError, TypeError):
                    pass
        if local_issue_ids:
            open_issues = [issue for issue in open_issues if issue.get("number") in local_issue_ids]

    missing_specs = []
    for issue in open_issues:
        issue_number = issue.get("number")
        issue_title = issue.get("title", "")
        found = False
        for f in features:
            fm_data = extract_metadata_from_content(f.content)
            if fm_data and fm_data.get("issue_id") == issue_number:
                found = True
                break

            # Existing filename check as fallback only when no metadata present
            if not fm_data:
                basename = os.path.splitext(f.filename)[0]
                m = re.search(r'(?:^|\D)(\d+)(?:$|\D)', basename)
                if m and int(m.group(1)) == issue_number:
                    found = True
                    break
        if not found:
            missing_specs.append(f"Issue #{issue_number}: '{issue_title}'")
            
    missing_spec_errors = []
    if missing_specs:
        print("[!] Missing local specification files for open feature issues:")
        for spec in missing_specs:
            print(f"  - {spec}")
        if not args.allow_missing_specs:
            missing_spec_errors = missing_specs[:]
            has_failed = True
        
    epic_files = []
    if epics_dir and os.path.exists(epics_dir):
        epic_files = [f for f in os.listdir(epics_dir) if f.endswith(".md")]

    skip_coverage_checks = False
    if args.spec_only or (not features and not epic_files):
        if args.spec_only:
            print("Note: Running in spec-only mode. Skipping model coverage checks.")
        else:
            print("Note: No feature or epic specifications found in directory. Skipping model coverage checks.")
        skip_coverage_checks = True
    else:
        react_dir_name = rules.target_directories.react
        flutter_dir_name = rules.target_directories.flutter
        react_exists = os.path.exists(os.path.join(repo.workspace_dir, react_dir_name)) if react_dir_name else False
        flutter_exists = os.path.exists(os.path.join(repo.workspace_dir, flutter_dir_name)) if flutter_dir_name else False
        if not react_exists and not flutter_exists:
            print("Note: Target directories (React and Flutter) do not exist. Skipping model coverage checks.")
            skip_coverage_checks = True
        
    # 3. Audit codebase coverage of UML classes
    total_defined = 0
    total_covered = 0
    coverage_gaps = []
    
    uml_validator = UmlValidator()
    global_classes = uml_validator.build_global_classes(repo, features_dir, epics_dir)
    
    if not skip_coverage_checks and (features or epic_files):
        # Read codebase source files
        codebase_contents = []
        
        # React
        react_dir_name = rules.target_directories.react
        if react_dir_name and rules.react_rules:
            react_dir = os.path.join(repo.workspace_dir, react_dir_name)
            if os.path.exists(react_dir):
                react_exts = tuple(rules.react_rules.file_extensions)
                react_exclusions = set(rules.react_rules.exclusions)
                for root, dirs, files in os.walk(react_dir):
                    dirs[:] = [d for d in dirs if d not in react_exclusions]
                    for file in files:
                        if file.endswith(react_exts):
                            try:
                                with open(os.path.join(root, file), "r", encoding="utf-8") as f:
                                    codebase_contents.append(f.read())
                            except Exception:
                                pass
                                
        # Flutter
        flutter_dir_name = rules.target_directories.flutter
        if flutter_dir_name:
            flutter_dir = os.path.join(repo.workspace_dir, flutter_dir_name)
            if os.path.exists(flutter_dir):
                flutter_exts = tuple(rules.flutter_rules.file_extensions)
                flutter_exclusions = set(rules.flutter_rules.exclusions)
                for root, dirs, files in os.walk(flutter_dir):
                    dirs[:] = [d for d in dirs if d not in flutter_exclusions]
                    for file in files:
                        if file.endswith(flutter_exts):
                            try:
                                with open(os.path.join(root, file), "r", encoding="utf-8") as f:
                                    codebase_contents.append(f.read())
                            except Exception:
                                pass

        react_dir_exists = False
        react_dir_name = rules.target_directories.react
        if react_dir_name:
            react_dir = os.path.join(repo.workspace_dir, react_dir_name)
            if os.path.exists(react_dir):
                react_dir_exists = True
                
        flutter_dir_exists = False
        flutter_dir_name = rules.target_directories.flutter
        if flutter_dir_name:
            flutter_dir = os.path.join(repo.workspace_dir, flutter_dir_name)
            if os.path.exists(flutter_dir):
                flutter_dir_exists = True
                
        if (react_dir_exists or flutter_dir_exists) and not codebase_contents:
            print("[!] Error: Target codebase directories exist but contain no source files.")
            has_failed = True

        # Helper to generate variants for a name
        def get_variants(name: str) -> Set[str]:
            variants = {name}
            if '-' in name or '_' in name or '.' in name:
                parts = re.split(r'[-_.]', name)
                variants.add(parts[0] + "".join(p.capitalize() for p in parts[1:]))
                variants.add("".join(p.capitalize() for p in parts))
                variants.add("_".join(p.lower() for p in parts))
            else:
                if name:
                    variants.add(name[0].lower() + name[1:])
                    variants.add(name[0].upper() + name[1:])
            return variants

        common_words = {"id", "name", "type", "status", "value", "height", "width", "time", "x", "y", "z", "t", "date", "info", "data", "key", "code", "save", "edit", "view", "vector"}
        
        def is_present_in_codebase(v: str, codebase: List[str]) -> bool:
            v_escaped = re.escape(v)
            if v.lower() not in common_words:
                return any(re.search(r'\b' + v_escaped + r'\b', strip_comments_and_strings(content)) for content in codebase)
            
            patterns = [
                r'\.\s*' + v_escaped + r'\b',                                  # member access: obj.id / obj?.id
                r'\bthis\s*\.\s*' + v_escaped + r'\b',                          # constructor assignment: this.id
                r'\b[a-zA-Z_][a-zA-Z0-9_<>\s]*\s+' + v_escaped + r'\b',          # type declaration: String name
                r'\b' + v_escaped + r'\s*:',                                   # named parameter: name: value
                r'\bconst\s*\{\s*[^}]*\b' + v_escaped + r'\b[^}]*\}\s*=',       # destructuring
                r'\blet\s*\{\s*[^}]*\b' + v_escaped + r'\b[^}]*\}\s*=',         # destructuring
            ]
            return any(any(re.search(pat, strip_comments_and_strings(content)) for pat in patterns) for content in codebase)

        for cls_name, cls_info in sorted(global_classes.items()):
            # Check class name
            cls_variants = get_variants(cls_name)
            cls_found = False
            for content in codebase_contents:
                if any(re.search(r'\b' + re.escape(v) + r'\b', strip_comments_and_strings(content)) for v in cls_variants):
                    cls_found = True
                    break
            
            total_defined += 1
            if cls_found:
                total_covered += 1
            else:
                coverage_gaps.append(f"Class '{cls_name}'")

            # Check attributes
            for attr in cls_info["attributes"]:
                attr_name = attr["name"]
                attr_variants = get_variants(attr_name)
                attr_found = any(is_present_in_codebase(v, codebase_contents) for v in attr_variants)
                total_defined += 1
                if attr_found:
                    total_covered += 1
                else:
                    coverage_gaps.append(f"Attribute '{cls_name}.{attr_name}'")

            # Check methods
            for method in cls_info["methods"]:
                method_name = method["name"]
                method_variants = get_variants(method_name)
                method_found = any(is_present_in_codebase(v, codebase_contents) for v in method_variants)
                total_defined += 1
                if method_found:
                    total_covered += 1
                else:
                    coverage_gaps.append(f"Method '{cls_name}.{method_name}'")

        print("\n=== Audit Summary ===")
        if total_defined > 0:
            overall_pct = (total_covered / total_defined) * 100
            print(f"Total UML Elements Defined: {total_defined}")
            print(f"Total UML Elements Covered: {total_covered}")
            print(f"Overall Model Coverage:     {overall_pct:.2f}%")
        else:
            print("No UML elements found in specifications to verify.")
            sys.exit(1)
            
    elif args.spec_only and (features or epic_files):
        print("\n=== Spec-Only Model Coverage Validation ===")
        all_definitions = {}
        for module_name, module_defs in modules.items():
            # Support both string types (production) and dict types (testing mock formats)
            def_types = {v.get("type") if isinstance(v, dict) else v for v in module_defs.values()}
            has_functional_nodes = any(t in def_types for t in ("container", "list", "leaf", "leaf-list", "choice", "case", "rpc", "notification", "action"))
            if not has_functional_nodes:
                continue
            all_definitions.update(module_defs)

        if not all_definitions:
            print("[-] Warning: No schema definitions were parsed. Skipping spec-only coverage validation.")
        else:
            spec_coverage_gaps = []
            spec_elements = set()
            for cls_name, cls_info in global_classes.items():
                spec_elements.add(cls_name.lower())
                for attr in cls_info.get("attributes", []):
                    spec_elements.add(attr["name"].lower())
                for method in cls_info.get("methods", []):
                    spec_elements.add(method["name"].lower())
            all_spec_contents = []
            if features:
                for feat in features:
                    all_spec_contents.append(feat.content)
                    if hasattr(feat, "frontmatter") and isinstance(feat.frontmatter, dict):
                        for container in feat.frontmatter.get("schema_containers", []):
                            if isinstance(container, dict):
                                path = container.get("path", "")
                            else:
                                path = str(container)
                            if path:
                                leaf = path.split("/")[-1]
                                if ":" in leaf:
                                    leaf = leaf.split(":", 1)[-1]
                                spec_elements.add(leaf.lower())
            if epics_dir and os.path.exists(epics_dir):
                for ep_file in os.listdir(epics_dir):
                    if ep_file.endswith(".md"):
                        try:
                            with open(os.path.join(epics_dir, ep_file), "r", encoding="utf-8") as f:
                                content = f.read()
                                all_spec_contents.append(content)
                                
                                data = extract_metadata_from_content(content)
                                if isinstance(data, dict):
                                    for container in data.get("schema_containers", []):
                                        if isinstance(container, dict):
                                            path = container.get("path", "")
                                        else:
                                            path = str(container)
                                        if path:
                                            leaf = path.split("/")[-1]
                                            if ":" in leaf:
                                                leaf = leaf.split(":", 1)[-1]
                                            spec_elements.add(leaf.lower())
                        except Exception:
                            pass

            for content in all_spec_contents:
                pass

            for key in sorted(all_definitions):
                name = key.split(":", 1)[1] if ":" in key else key
                if "/" in name:
                    name = name.split("/")[-1]
                
                variants = {name}
                if '-' in name or '_' in name or '.' in name:
                    parts = re.split(r'[-_.]', name)
                    variants.add(parts[0] + "".join(p.capitalize() for p in parts[1:]))
                    variants.add("".join(p.capitalize() for p in parts))
                    variants.add("_".join(p.lower() for p in parts))
                else:
                    if name:
                        variants.add(name[0].lower() + name[1:])
                        variants.add(name[0].upper() + name[1:])
                        
                mapped = False
                for v in variants:
                    if v.lower() in spec_elements:
                        mapped = True
                        break
                if not mapped:
                    spec_coverage_gaps.append(f"Schema node '{name}'")
                    
            if spec_coverage_gaps:
                print("[!] Spec-Only Model Coverage Gaps Identified:")
                for gap in sorted(spec_coverage_gaps):
                    print(f"  - {gap}")
                print("\nError: 100% spec-only model coverage validation failed.")
                has_failed = True
            elif all_definitions:
                print("Success: 100% spec-only model coverage verified across all specification files.")

    print("\n=== UML Diagrams Compliance Audit ===")
    uml_errors = []
    if not features and not epic_files:
        print("Note: No feature or epic specifications found. Skipping UML Diagrams Compliance Audit.")
    else:
        uml_errors = _scope_findings(uml_validator.validate(repo, global_classes=global_classes, epics_dir=epics_dir), getattr(args, 'only', None))
        
    if uml_errors:
        print("[!] UML Compliance Violations Identified:")
        for err in uml_errors:
            print(f"  - {err}")
        has_failed = True
    else:
        if features or epic_files:
            print("Success: All specification files are fully UML-compliant (no ERDs or invalid syntax found).")
            
    if coverage_gaps:
        print("\n[!] Codebase Coverage Gaps Identified:")
        for gap in sorted(coverage_gaps):
            print(f"  - {gap}")
        print("\nError: 100% model coverage validation failed.")
        has_failed = True
    else:
        if not skip_coverage_checks and (features or epic_files):
            print("\nSuccess: 100% model coverage verified across all specification files.")
            
    print("\n=== Behavioral Coverage Triggers Audit ===")
    behavioral_validator = BehavioralValidator()
    behavioral_errors = []
    if not features and not epic_files:
        print("Note: No feature or epic specifications found. Skipping Behavioral Coverage Triggers Audit.")
    else:
        behavioral_errors = _scope_findings(behavioral_validator.validate(repo, schema_dir=schema_dir, modules=modules), getattr(args, 'only', None))
        
    if behavioral_errors:
        print("[!] Behavioral Coverage Violations Identified:")
        for err in behavioral_errors:
            print(f"  - {err}")
        has_failed = True
    else:
        print("Success: All behavioral coverage triggers passed.")
        
    if args.spec_only:
        print("Note: Running in spec-only mode. Skipping Codebase AST / Compliance Audit.")
        codebase_errors = []
    else:
        print("\n=== Codebase AST / Compliance Audit ===")
        codebase_validator = CodebaseValidator()
        codebase_errors = _scope_findings(codebase_validator.validate(repo), getattr(args, 'only', None))
    
    if codebase_errors:
        print("[!] Codebase Compliance Violations Identified:")
        for err in codebase_errors:
            print(f"  - {err}")
        has_failed = True
    else:
        print("Success: Codebase compliance checks passed.")
        
    print("\n=== Documentation Consistency Audit ===")
    docs_validator = DocsValidator()
    doc_errors = _scope_findings(docs_validator.validate(repo), getattr(args, 'only', None))
    
    if doc_errors:
        print("[!] Documentation Consistency Violations Identified:")
        for err in doc_errors:
            print(f"  - {err}")
        has_failed = True
    else:
        print("Success: Documentation consistency checks passed.")
        
    print("\n=== Schema Dependency Validation ===")
    dependency_validator = DependencyValidator()
    dependency_errors = _scope_findings(dependency_validator.validate(repo, schema_dir=schema_dir), getattr(args, 'only', None))
    
    if dependency_errors:
        print("[!] Schema Dependency Violations Identified:")
        for err in dependency_errors:
            print(f"  - {err}")
        has_failed = True
    else:
        print("Success: Schema dependency checks passed.")
        
    print("\n=== Out-of-Sync Backlog Validation ===")
    sync_validator = SyncValidator()
    sync_errors = _scope_findings(sync_validator.validate(repo), getattr(args, 'only', None))
    
    if sync_errors:
        print("[!] Out-of-Sync Backlog Violations Identified:")
        for err in sync_errors:
            print(f"  - {err}")
        has_failed = True
    else:
        print("Success: Out-of-Sync Backlog checks passed.")
        
    print("\n=== Schema Mapping Validation ===")
    if args.spec_only:
        print("Note: Running in spec-only mode. Skipping Schema Mapping Validation.")
        schema_mapping_errors = []
    else:
        schema_mapping_validator = SchemaMappingValidator()
        schema_mapping_errors = _scope_findings(schema_mapping_validator.validate(repo), getattr(args, 'only', None))
        if schema_mapping_errors:
            print("[!] Schema Mapping Violations Identified:")
            for err in schema_mapping_errors:
                print(f"  - {err}")
            has_failed = True
        else:
            print("Success: Schema mapping checks passed.")

    print("\n=== Profile Scoping Validation ===")
    if args.spec_only:
        print("Note: Running in spec-only mode. Skipping Profile Scoping Validation.")
        profile_scoping_errors = []
    else:
        profile_scoping_validator = ProfileScopingValidator()
        profile_scoping_errors = _scope_findings(profile_scoping_validator.validate(repo), getattr(args, 'only', None))
        if profile_scoping_errors:
            print("[!] Profile Scoping Violations Identified:")
            for err in profile_scoping_errors:
                print(f"  - {err}")
            has_failed = True
        else:
            print("Success: Profile scoping checks passed.")

    print("\n=== Test Completeness Validation ===")
    if args.spec_only:
        print("Note: Running in spec-only mode. Skipping Test Completeness Validation.")
        test_completeness_errors = []
    else:
        test_completeness_validator = TestCompletenessValidator()
        test_completeness_errors = _scope_findings(test_completeness_validator.validate(repo), getattr(args, 'only', None))
        if test_completeness_errors:
            print("[!] Test Completeness Violations Identified:")
            for err in test_completeness_errors:
                print(f"  - {err}")
            has_failed = True
        else:
            print("Success: Test completeness checks passed.")
        
    print("\n=== Schema Cardinality Validation ===")
    cardinality_validator = SchemaCardinalityValidator()
    cardinality_errors = _scope_findings(cardinality_validator.validate(repo, is_sysml=args.sysml), getattr(args, 'only', None))
    if cardinality_errors:
        print("[!] Schema Cardinality Violations Identified:")
        for err in cardinality_errors:
            print(f"  - {err}")
        has_failed = True
    else:
        print("Success: 1:1 container-to-file cardinality verified.")

    print("\n=== Spec Filename Validation ===")
    spec_filename_validator = SpecFilenameValidator()
    spec_filename_errors = _scope_findings(spec_filename_validator.validate(repo), getattr(args, 'only', None))
    if spec_filename_errors:
        print("[!] Spec Filename Violations Identified:")
        for err in spec_filename_errors:
            print(f"  - {err}")
        has_failed = True
    else:
        print("Success: Spec filename convention verified.")

    print("\n=== Spec Title Uniqueness Validation ===")
    spec_title_validator = SpecTitleUniquenessValidator()
    spec_title_errors = _scope_findings(spec_title_validator.validate(repo), getattr(args, 'only', None))
    if spec_title_errors:
        print("[!] Spec Title Uniqueness Violations Identified:")
        for err in spec_title_errors:
            print(f"  - {err}")
        has_failed = True
    else:
        print("Success: Specification titles are unique within each spec type.")

    print("\n=== Source Reference Integrity Validation ===")
    source_ref_validator = SourceReferenceValidator()
    source_ref_errors = _scope_findings(source_ref_validator.validate(repo), getattr(args, 'only', None))
    if source_ref_errors:
        print("[!] Source Reference Violations Identified:")
        for err in source_ref_errors:
            print(f"  - {err}")
        has_failed = True
    else:
        print("Success: Source References carry authoritative locators.")

    print("\n=== Markdown Link Integrity Validation ===")
    link_validator = LinkValidator()
    link_errors = _scope_findings(link_validator.validate(repo), getattr(args, 'only', None))
    if link_errors:
        print("[!] Markdown Link Violations Identified:")
        for err in link_errors:
            print(f"  - {err}")
        has_failed = True
    else:
        print("Success: All markdown cross-references are valid.")

    print("\n=== Mermaid Syntax Validation ===")
    mermaid_syntax_validator = MermaidSyntaxValidator()
    mermaid_syntax_errors = _scope_findings(mermaid_syntax_validator.validate(repo), getattr(args, 'only', None))
    if mermaid_syntax_errors:
        print("[!] Mermaid Syntax Violations Identified:")
        for err in mermaid_syntax_errors:
            print(f"  - {err}")
        has_failed = True
    else:
        print("Success: Mermaid syntax rules verified.")

    print("\n=== KaTeX Mathematical Rendering Integrity Validation ===")
    katex_validator = KatexValidator()
    katex_errors = _scope_findings(katex_validator.validate(repo), getattr(args, 'only', None))
    if katex_errors:
        print("[!] KaTeX Integrity Violations Identified:")
        for err in katex_errors:
            print(f"  - {err}")
        has_failed = True
    else:
        print("Success: KaTeX math syntax and AST integrity verified.")

    print("\n=== Logical UI Validation ===")
    logical_ui_validator = LogicalUiValidator()
    logical_ui_errors = _scope_findings(logical_ui_validator.validate(repo, features_dir=features_dir), getattr(args, 'only', None))
    if logical_ui_errors:
        print("[!] Logical UI Violations Identified:")
        for err in logical_ui_errors:
            print(f"  - {err}")
        has_failed = True
    else:
        print("Success: Logical UI checks passed.")

    print("\n=== Public Member Docstring Validation ===")
    docstring_validator = DocstringValidator()
    docstring_errors = _scope_findings(docstring_validator.validate(repo), getattr(args, 'only', None))
    if docstring_errors:
        print("[!] Public Member Docstring Violations Identified:")
        for err in docstring_errors:
            print(f"  - {err}")
        has_failed = True
    else:
        print("Success: Public member docstrings verified.")

    print("\n=== Profile Compliance Validation ===")
    profile_compliance_validator = ProfileComplianceValidator()
    profile_compliance_errors = _scope_findings(profile_compliance_validator.validate(repo), getattr(args, 'only', None))
    if profile_compliance_errors:
        print("[!] Profile Compliance Violations Identified:")
        for err in profile_compliance_errors:
            print(f"  - {err}")
        has_failed = True
    else:
        print("Success: Profile compliance checks passed.")

    print("\n=== Package Structure & Subsystem Allocation Audit ===")
    package_allocation_errors = _scope_findings(cardinality_validator.validate_package_structure_and_subsystem_allocation(repo), getattr(args, 'only', None))
    if package_allocation_errors:
        print("[!] Package Structure & Subsystem Allocation Violations Identified:")
        for err in package_allocation_errors:
            print(f"  - {err}")
        has_failed = True
    else:
        print("Success: Package structure and subsystem capability allocations verified.")

    print("\n=== Feature Operation & Schema Constraint Coverage ===")
    feature_op_errors = _scope_findings(cardinality_validator.validate_feature_operation_and_constraint_coverage(repo), getattr(args, 'only', None))
    if feature_op_errors:
        print("[!] Feature Operation & Schema Constraint Violations Identified:")
        for err in feature_op_errors:
            print(f"  - {err}")
        has_failed = True
    else:
        print("Success: Feature operation and schema constraint coverage verified.")

    print("\n=== User Story Interaction & Sequence Lifeline Audit ===")
    interaction_errors = _scope_findings(uml_validator.validate_user_story_interactions_and_lifelines(repo, global_classes=global_classes), getattr(args, 'only', None))
    if interaction_errors:
        print("[!] User Story Interaction & Sequence Lifeline Violations Identified:")
        for err in interaction_errors:
            print(f"  - {err}")
        has_failed = True
    else:
        print("Success: User Story interaction sequences and lifelines verified.")

    print("\n=== Safety Invariant & RTA Constraint Assertion Audit ===")
    safety_constraint_errors = _scope_findings(uml_validator.validate_safety_invariants_and_rta_constraints(repo), getattr(args, 'only', None))
    if safety_constraint_errors:
        print("[!] Safety Invariant & RTA Constraint Assertion Violations Identified:")
        for err in safety_constraint_errors:
            print(f"  - {err}")
        has_failed = True
    else:
        print("Success: Safety invariants and RTA constraint assertions verified.")

    print("\n=== Acceptance Criteria Test Case & Verification Binding Audit ===")
    acceptance_test_errors = _scope_findings(uml_validator.validate_acceptance_criteria_and_test_cases(repo), getattr(args, 'only', None))
    if acceptance_test_errors:
        print("[!] Acceptance Criteria Test Case & Verification Binding Violations Identified:")
        for err in acceptance_test_errors:
            print(f"  - {err}")
        has_failed = True
    else:
        print("Success: Acceptance criteria test cases and verification bindings verified.")

    print("\n=== Concept Provenance & Parametric SSOT Audit ===")
    concept_provenance_validator = ConceptProvenanceValidator()
    concept_provenance_errors = _scope_findings(concept_provenance_validator.validate(repo), getattr(args, 'only', None))
    if concept_provenance_errors:
        print("[!] Concept Provenance Violations Identified:")
        for err in concept_provenance_errors:
            print(f"  - {err}")
        has_failed = True
    else:
        print("Success: Concept provenance and parametric assertions verified.")

    print("\n=== Safety Traceability & Set-Equality Audit ===")
    safety_trace_validator = SafetyTraceValidator()
    safety_trace_errors = _scope_findings(safety_trace_validator.validate(repo), getattr(args, 'only', None))
    if safety_trace_errors:
        print("[!] Safety Traceability Violations Identified:")
        for err in safety_trace_errors:
            print(f"  - {err}")
        has_failed = True
    else:
        print("Success: Safety traceability set-equality verified.")

    print("\n=== Document Metadata & Frontmatter Audit ===")
    doc_metadata_validator = DocMetadataValidator()
    doc_metadata_errors = _scope_findings(doc_metadata_validator.validate(repo), getattr(args, 'only', None))
    if doc_metadata_errors:
        print("[!] Document Metadata Violations Identified:")
        for err in doc_metadata_errors:
            print(f"  - {err}")
        has_failed = True
    else:
        print("Success: Document metadata tables and frontmatter valid across all markdown documents.")

    if has_failed:
        all_errors = (uml_errors or []) + (behavioral_errors or []) + (codebase_errors or []) + (doc_errors or []) + (dependency_errors or []) + (sync_errors or []) + (schema_mapping_errors or []) + (profile_scoping_errors or []) + (test_completeness_errors or []) + (cardinality_errors or []) + (spec_filename_errors or []) + (spec_title_errors or []) + (mermaid_syntax_errors or []) + (katex_errors or []) + (logical_ui_errors or []) + (docstring_errors or []) + (profile_compliance_errors or []) + (package_allocation_errors or []) + (feature_op_errors or []) + (interaction_errors or []) + (safety_constraint_errors or []) + (acceptance_test_errors or []) + (missing_spec_errors or []) + (source_ref_errors or []) + (link_errors or []) + (concept_provenance_errors or []) + (safety_trace_errors or []) + (doc_metadata_errors or [])
        compiled_errors = all_errors
        target_file = None
        snippet_content = None
        for err in compiled_errors:
            match = re.search(r'docs/[a-zA-Z0-9_\-/]+\.md', err)
            if match:
                rel_path = match.group(0)
                abs_path = os.path.join(workspace_dir, rel_path)
                if os.path.exists(abs_path):
                    target_file = rel_path
                    try:
                        with open(abs_path, "r", encoding="utf-8") as f:
                            snippet_content = f.read()
                    except Exception:
                        pass
                    break
        
        serialize_diagnostics(
            workspace_dir=workspace_dir,
            tool_name="parity_auditor",
            exit_code=1,
            errors=compiled_errors,
            traceback_str="",
            target_file=target_file,
            snippet_content=snippet_content
        )
        sys.exit(1)
    else:
        print("\nSuccess: All verification checks passed.")
        sys.exit(0)

def main():
    """
    Entry point: strips dummy GITHUB_TOKEN and GH_TOKEN, runs ``_main_impl()``, and
    catches unhandled exceptions with a diagnostic report.

    Side effects:
        - Removes ``GITHUB_TOKEN`` and ``GH_TOKEN`` from the environment if they contain
          dummy token keywords.
        - Exits with code 1 on failure after writing a diagnostics JSON
          artifact.
    """
    sanitize_github_token_env()
    try:
        _main_impl()
    except SystemExit:
        raise
    except Exception:
        import traceback
        traceback.print_exc()
        upstream_repo = os.environ.get("UPSTREAM_REPOSITORY") or os.environ.get("GIT_REMOTE_ORIGIN") or "gintatkinson/uas-003"
        try:
            script_dir = os.path.dirname(os.path.abspath(__file__))
            workspace_dir = None
            curr = script_dir
            while True:
                if os.path.exists(os.path.join(curr, ".pipeline", "logical-ui", "codebase_rules.json")):
                    workspace_dir = curr
                    break
                parent = os.path.dirname(curr)
                if parent == curr:
                    break
                curr = parent
            if not workspace_dir:
                workspace_dir = os.getcwd()
            rules_path = os.path.join(workspace_dir, ".pipeline", "logical-ui", "codebase_rules.json")
            if os.path.exists(rules_path):
                with open(rules_path, "r", encoding="utf-8") as f:
                    rules_data = json.load(f)
                    upstream_repo = rules_data.get("meta", {}).get("upstream_repository", upstream_repo)
        except Exception:
            pass
        print("\n[!] If you believe this failure is due to a bug or limitation in the pipeline tooling, please report it upstream:")
        print(f"    gh issue create --repo {upstream_repo} --title \"Tooling Bug: [Brief description]\" --body \"Context: UML/Coverage validation failed in downstream execution.\"")
        sys.exit(1)

if __name__ == "__main__":
    main()
