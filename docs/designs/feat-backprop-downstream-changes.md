# Solution Walkthrough: Back-propagation of Downstream Changes

This document details the changes back-propagated from downstream repositories, including the import of the `adversarial-code-auditor` skill and the modularization of the `parity_auditor` tool to support optional React rules, allowing single-platform codebases to validate cleanly.

---

## 1. Overview of Changes

### New Files

* **[skills/adversarial-code-auditor/SKILL.md](skills/adversarial-code-auditor/SKILL.md)**:
  - Imported the generic pre-emptive auditing skill. This defines the protocol for subagents to audit codebase files against four correctness risk pillars (Memory Safety, Resource Lifecycle, Concurrency, and Test Integrity) and file identified issues via GitHub CLI.

### Modified Files

* **[skills/spec-orchestrator/parity_auditor/src/parity_auditor/core/models.py](skills/spec-orchestrator/parity_auditor/src/parity_auditor/core/models.py)**:
  - Made the `react` directory and `react_rules` fields optional in the codebase rules models (`TargetDirectories` and `CodebaseRules`).
  - Refactored `load_from_dict` to gracefully handle the absence of `react_rules`.

* **[skills/spec-orchestrator/parity_auditor/src/parity_auditor/cli.py](skills/spec-orchestrator/parity_auditor/src/parity_auditor/cli.py)**:
  - Added a new CLI argument `--allow-missing-specs` to optionally bypass strict validation failures when specification files are missing.
  - Guarded React-specific configuration references to prevent crashes when React target settings or rules are omitted.

* **[skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/codebase.py](skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/codebase.py)**:
  - Guarded React codebase compliance validation so it skips React check steps and does not crash when `react_rules` is omitted.

* **[skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/profile_scoping_validator.py](skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/profile_scoping_validator.py)**:
  - Guarded profile scoping checks to avoid crashing and skip react source files validation when `react_rules` is `None`.

* **[skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/schema_mapping_validator.py](skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/schema_mapping_validator.py)**:
  - Guarded schema mapping checks to skip collecting React UI files when `react_rules` is `None`.

---

## 2. Rationale

1. **Downstream Skill Import**:
   The `adversarial-code-auditor` skill enables rigorous subagent-based auditing. Bringing it upstream guarantees that all repositories extending this tooling can execute pre-emptive safety audits against critical correctness vectors (memory leaks, thread concurrency issues, test integrity, and FFI boundary safety).

2. **Support for Single-Platform Codebases**:
   Previously, the `parity_auditor` assumed both Flutter and React folders were present and defined in the rules file (`codebase_rules.json`). For codebases that only build a mobile Flutter client (or only a React web frontend), this restriction forced the creation of dummy folders and rules files. By making `react_rules` and `react` optional, the validator dynamically scales to the platform(s) defined, avoiding unnecessary exceptions.

3. **Incremental Parity Auditing**:
   By adding `--allow-missing-specs`, pipeline verification checks can be run in situations where specs are still being drafted or backlogged, reporting the discrepancies without blocking CI/CD pipelines with non-zero exit codes.

---

## 3. Detailed Diff Walkthrough

### `models.py`
```diff
@@ -24,7 +24,7 @@ class BacklogDirectories:
 
 @dataclass
 class TargetDirectories:
-    react: str = "web_react"
+    react: Optional[str] = None
     flutter: str = "app_flutter"
 
 @dataclass
@@ -129,7 +129,7 @@ class CodebaseRules:
     tracker_rules: Dict[str, Any] = field(default_factory=dict)
     backlog_directories: BacklogDirectories = field(default_factory=BacklogDirectories)
     target_directories: TargetDirectories = field(default_factory=TargetDirectories)
-    react_rules: ReactRules = field(default_factory=ReactRules)
+    react_rules: Optional[ReactRules] = None
     flutter_rules: FlutterRules = field(default_factory=FlutterRules)
     python_rules: PythonRules = field(default_factory=PythonRules)
     spec_rules: SpecRules = field(default_factory=SpecRules)
@@ -145,8 +145,11 @@ def load_from_dict(data: dict) -> CodebaseRules:
     td_data = data.get("target_directories", {})
     target_directories = TargetDirectories(**{k: v for k, v in td_data.items() if k in TargetDirectories.__dataclass_fields__})
     
-    react_data = data.get("react_rules", {})
-    react_rules = ReactRules(**{k: v for k, v in react_data.items() if k in ReactRules.__dataclass_fields__})
+    react_data = data.get("react_rules")
+    if react_data is not None:
+        react_rules = ReactRules(**{k: v for k, v in react_data.items() if k in ReactRules.__dataclass_fields__})
+    else:
+        react_rules = None
```

### `cli.py`
```diff
@@ -79,6 +79,7 @@ def _main_impl():
     parser.add_argument("schema_dir", nargs="?", help="Path to schema directory")
     parser.add_argument("features_dir", nargs="?", help="Path to feature specs directory")
     parser.add_argument("--spec-only", action="store_true", help="Run in specification-only mode, bypassing codebase checks")
+    parser.add_argument("--allow-missing-specs", action="store_true", help="Skip exiting with status code 1 when there are missing specification files")
     
     args = parser.parse_args()
     
@@ -230,7 +231,8 @@ def _main_impl():
         print("[!] Missing local specification files for open feature issues:")
         for spec in missing_specs:
             print(f"  - {spec}")
-        sys.exit(1)
+        if not args.allow_missing_specs:
+            sys.exit(1)
     
     skip_coverage_checks = False
     if args.spec_only or not features:
@@ -262,7 +264,7 @@ def _main_impl():
         
         # React
         react_dir_name = rules.target_directories.react
-        if react_dir_name:
+        if react_dir_name and rules.react_rules:
             react_dir = os.path.join(repo.workspace_dir, react_dir_name)
             if os.path.exists(react_dir):
```

### `codebase.py`
```diff
@@ -69,7 +69,7 @@ class CodebaseValidator(IValidator):
         react_dir_name = target_dirs.react
         react_dir = os.path.join(workspace_dir, react_dir_name) if react_dir_name else None
         if react_dir_name and not os.path.exists(react_dir):
-            if has_files_with_extensions(react_rules.file_extensions):
+            if react_rules and has_files_with_extensions(react_rules.file_extensions):
                 errors.append(f"Compliance Bypass Loophole: Configured React directory '{react_dir_name}' does not exist on disk.")
         
         flutter_dir_name = target_dirs.flutter
@@ -115,7 +115,7 @@ class CodebaseValidator(IValidator):
             hardcoded_colors_flutter[flutter_hex] = f"Forbidden design token color (0x{flutter_hex.upper()})"
             
         # 1. React Web Codebase Compliance
-        if react_dir and os.path.exists(react_dir):
+        if react_dir and os.path.exists(react_dir) and react_rules:
             react_exts = tuple(react_rules.file_extensions)
             react_exclusions = set(react_rules.exclusions)
```

### `profile_scoping_validator.py`
```diff
@@ -29,7 +29,10 @@ class ProfileScopingValidator(IValidator):
             return matched
             
         react_rules = rules.react_rules
-        react_files = find_files(react_dir, react_rules.file_extensions, set(react_rules.exclusions))
+        if react_rules:
+            react_files = find_files(react_dir, react_rules.file_extensions, set(react_rules.exclusions))
+        else:
+            react_files = []
         
         flutter_rules = rules.flutter_rules
```

### `schema_mapping_validator.py`
```diff
@@ -40,8 +40,12 @@ class SchemaMappingValidator(IValidator):
         ui_files = []
         
         react_rules = rules.react_rules
-        react_exclusions = set(react_rules.exclusions)
-        react_ui_dirs = set(react_rules.ui_directories)
+        if react_rules:
+            react_exclusions = set(react_rules.exclusions)
+            react_ui_dirs = set(react_rules.ui_directories)
+        else:
+            react_exclusions = set()
+            react_ui_dirs = set()
         
         flutter_rules = rules.flutter_rules
@@ -62,7 +66,8 @@ class SchemaMappingValidator(IValidator):
                            if is_ui:
                                ui_files.append(filepath)
                                  
-        collect_files(react_dir, react_rules.file_extensions, react_exclusions, react_ui_dirs)
+        if react_rules:
+            collect_files(react_dir, react_rules.file_extensions, react_exclusions, react_ui_dirs)
         collect_files(flutter_dir, flutter_rules.file_extensions, flutter_exclusions, flutter_ui_dirs)
```
