# Verification and Testing Facilities Survey & Test Strategy Report

**Explorer ID:** explorer_r4_3  
**Date:** 2026-09-21T16:37:00Z  
**Repository Classification:** UPSTREAM_SPEC_CORE_COMPILER  
**Target Milestone:** Three-Tier Onboarding & README Template Overhaul  

---

## 1. Executive Summary

This report surveys the verification and testing facilities in `DEAP01-spec-core` to establish an empirical test harness for the Three-Tier Onboarding and README Overhaul. 

The investigation inspected:
1. `scripts/verify_downstream_baseline.py` (downstream conformance validator).
2. Existing test suites in `tests/` (`tests/test_domain_url_synthesis.py` and `tests/test_sysmlv2_markdown_ingest.py`).
3. The upstream README (`DEAP01-spec-core/README.md`) and installer scaffolding logic (`scripts/install_pipeline.sh` and `scripts/scaffold_downstream_agents.py`).

### Key Findings:
- `scripts/verify_downstream_baseline.py` runs 24 automated checks (Checks 10 through 30, plus platform-specific baseline and type checks). However:
  - **Check 14** only tests that `README.md` exists and contains either `# Downstream Cyber-Physical Infrastructure Safety Project` or `Operator Prompt Catalog`. It performs **zero** validation of shell code fences, syntax validity, unescaped parentheses in comments, or role classification.
  - `--no-domain` on the CLI is strictly preserved (cannot be overridden by existing domain directories), bypassing Flutter/React domain type checks and compile/build execution while executing Checks 10-30.
  - The script distinguishes upstream from downstream solely by the presence of `.pipeline/upstream`, but does not inspect or enforce the three explicit repository classifications (`UPSTREAM_SPEC_CORE_COMPILER`, `DOMAIN_DISTRIBUTION_TEMPLATE`, `DOWNSTREAM_CUSTOMER_PROJECT`).
- In `DEAP01-spec-core/README.md`, Section 5.4 contains an 80-line inline Python monkeypatching script and manual `cp` commands that violate clean repository boundaries. Section 5.3 contains hardcoded customer onboarding commands cloning `DEAP-uas-infrastructure-safety.git`.
- In `scripts/install_pipeline.sh`, the generated README hardcodes `Repository Role: DOWNSTREAM_APPLICATION_WORKSPACE` for all targets. When run in a customer project (e.g. `uav-011`), it generates a circular command instructing the user inside `uav-011` to clone `uav-011` to install a pipeline.
- Unescaped parentheses in shell comments (`# ... (...)`) and unquoted angle-bracket placeholders (`<url>`) trigger syntax and parsing errors in interactive shells (e.g., zsh command substitution/glob syntax errors and bash redirection syntax errors).

---

## 2. Examination of `scripts/verify_downstream_baseline.py`

### 2.1 Complete Inventory of Executed Checks

When executed, `scripts/verify_downstream_baseline.py` runs a series of sequential gates:

| Check # | Function Name | Scope / Target | Description |
|---|---|---|---|
| **Check 10** | `check_gitignore_exists` | Root | Verifies `.gitignore` exists in the repository root. |
| **Check 11** | `check_no_ds_store_files` | Root | Asserts 0 tracked `.DS_Store` files in git index; cleans untracked `.DS_Store`. |
| **Check 12** | `check_no_duplicate_master_blueprints` | Downstream only | Verifies downstream repo contains no duplicate master core blueprints (`DEAP_MASTER_ARCHITECTURE.md`, `THREE_TIER_GOVERNANCE_BLUEPRINT.md`, `DEAP_SYSML_V2_SAFETY_MODEL_SPECIFICATION.sysml`). Skipped if `.pipeline/upstream` exists. |
| **Check 13** | `check_latex_katex_syntax` | All `.md` files | Validates display math delimiters (`$$`), forbids top-level `\begin{align}`, forbids bare alignment operator `&` outside alignment environments, and runs `KatexValidator`. |
| **Mermaid** | `check_mermaid_syntax` | All `.md` files | Validates Mermaid diagram code fences, closing fence integrity, class diagram member colons, and flowchart node label quoting. |
| **Check 14** | `check_downstream_instructions_exist` | Root | Asserts `README.md` exists and contains `# Downstream Cyber-Physical Infrastructure Safety Project` or `Operator Prompt Catalog`. Asserts agent entrypoints (`AGENTS.md`, `CLAUDE.md`, or `.agents/AGENTS.md`) and `rules/sysml-ssot-completeness.md` exist. |
| **Check 15** | `check_reconcile_backlog_tooling_exists` | Root | Asserts `scripts/reconcile_backlog.py` exists, is non-empty, and has execute bit (`+x`). |
| **Check 16** | `check_upstream_template_clean_landing_zones` | Upstream only | Verifies that landing zones (`docs/conops`, `docs/safety`, `docs/epics`, `docs/features`, `docs/user-stories`, `docs/use-cases`, `docs/management`, `schema`) contain only `.gitkeep` and `README.md`. Skipped if downstream. |
| **Check 17** | `check_safety_integrity_and_sora_completeness` | Downstream / Safety | Validates STPA 8-pillar schema, FMECA 15+ rows, and SORA OSO table if models exist; skips if landing zone clean. |
| **Check 18** | `verify_upstream_blueprint_domain_cleanliness` | Upstream only | Verifies upstream architecture blueprints have zero domain-specific concept papers or models. |
| **Check 19** | `check_domain_agnostic_ast_cleanliness` | Upstream only | Verifies upstream compiler contains zero static hardcoded parameter dictionaries in validators. |
| **Check 20** | `_check_wbs_suite_integrity` | Root / WBS | Validates Level 4 WBS deliverables if present; passes if pending. |
| **Checks 21-30** | Gates 21 through 30 | Downstream / Specs | Validates semantic diagram AST parity (21), semantic prose invariants (22), factual grounding & numeric provenance (23), Level 1C ICD completeness (24), operational allocation (25), standards measurement (26), cross-document diagram parity (27), ConOps completeness (28), research inventory (29), executive traceability (30), coverage digest, obligation witness, and viewpoint diagrams. |
| **Flutter Baseline** | `_run_verification` (is_flutter) | `app_flutter/` | Asserts baseline files (`pubspec.yaml`, `analysis_options.yaml`, `lib/main.dart`, `lib/domain/validation.dart`, and `repository_resolver.dart` if domain enabled). |
| **React Baseline** | `_run_verification` (is_react) | `web_react/` | Asserts baseline files (`tsconfig.json`, entrypoint, `src/domain/validation.*` if domain enabled). |

### 2.2 Detailed Behavior of `--no-domain`

Lines 242–255 and 3538–3654 of `scripts/verify_downstream_baseline.py` govern `--no-domain`:

1. **CLI Precedence**:
   - `args.no_domain` is checked first. If passed via CLI, `no_domain_for_target = True` is guaranteed and cannot be overridden by domain directory presence on disk.
   - If `--no-domain` is omitted, the script checks configuration files (`.pipeline/logical-ui/codebase_rules.json`, `codebase_rules.json`, `baseline_manifest.json`). If configuration specifies `no_domain: true`, but a domain directory (`lib/domain` or `src/domain`) exists, the script overrides the stored configuration and enables domain verification.
2. **Execution Bypass**:
   - **Flutter**: Bypasses `lib/domain/repository_resolver.dart` existence check. Bypasses `_validate_domain_types`. Bypasses `flutter pub get`, `flutter analyze`, `flutter test`, `flutter build macos --release`, and release archive packaging.
   - **React**: Bypasses `src/domain/validation.*` existence check. Bypasses `_validate_domain_types`. Bypasses `npm install` and `npm run build`.
3. **Execution Retention**:
   - Checks 10 through 30 in `run_all_checks(repo_root)` **always run**, regardless of `--no-domain`.

### 2.3 Verification Gaps in Baseline Script

- **README Syntax & Structure**:
  - `check_downstream_instructions_exist` only verifies substring presence. It does not check markdown heading order, link validity, or content appropriateness for repository tier.
- **Code Fence Syntax**:
  - Code fences are only inspected for unclosed fences in `check_mermaid_syntax` or stripped for KaTeX parsing. There is no parser or syntax checker for shell (`bash`/`sh`) code blocks.
- **Parentheses & Character Hygiene in Comments**:
  - Zero checks exist for unescaped parentheses in shell comments or command blocks.
- **Repository Classification Enactment**:
  - No check validates whether the repository declares its classification (`UPSTREAM_SPEC_CORE_COMPILER`, `DOMAIN_DISTRIBUTION_TEMPLATE`, or `DOWNSTREAM_CUSTOMER_PROJECT`) in `README.md` or `AGENTS.md`.

---

## 3. Survey of Existing Testing Facilities in `tests/`

### 3.1 `tests/test_domain_url_synthesis.py`

This test suite provides the standard model for testing `scripts/install_pipeline.sh`:
- **Isolation Pattern**: Uses `tempfile.TemporaryDirectory()` to create throwaway target directories. Operates completely outside the repository tree, strictly conforming to the "Forbidden Test Workspace Creation" rule.
- **Subprocess Invocation**:
  ```python
  cmd = ["bash", INSTALL_SCRIPT] + list(args)
  proc = subprocess.run(cmd, cwd=cwd or PROJECT_ROOT, capture_output=True, text=True)
  ```
- **Verification Assertions**:
  - Verifies exit codes (`assertEqual(proc.returncode, 0)` or `assertEqual(proc.returncode, 1)`).
  - Reads generated `<temp_dir>/README.md` and asserts presence of expected clone strings (`assertIn`) and absence of forbidden strings (`assertNotIn("gitlab.com", line)`).
  - Tests CLI parameter edge cases: `--help`, missing arguments, argument followed by flag, equals syntax (`--domain-url=...`).
  - Tests git configuration preservation: initializes a git repo with a mock `origin` remote and verifies installer preserves it.

### 3.2 `tests/test_sysmlv2_markdown_ingest.py`

This suite tests schema ingestion and SysML compilation fallback:
- Tests AST translators and round-trip parsing without hardcoded domain models.
- Tests compilation gate error handling and remediation guidance.

### 3.3 Test Runner & CI Integration

All tests in `tests/` execute via standard unittest discovery:
```bash
python3 -m unittest discover tests
```
Both test suites pass in ~6.5 seconds with zero external network dependencies.

---

## 4. Test Strategy for Three-Tier README Scaffolding

To test and prevent regressions across the three repository tiers, a new dedicated test suite `tests/test_readme_scaffolding.py` should be established.

### 4.1 Architecture of `tests/test_readme_scaffolding.py`

```
tests/test_readme_scaffolding.py
├── TestUpstreamCompilerReadme (Tier 1)
│   ├── test_upstream_readme_no_inline_python_scripts
│   ├── test_upstream_readme_no_domain_clone_commands
│   ├── test_upstream_readme_compiler_focus_commands
│   └── test_upstream_readme_repository_classification
├── TestDomainDistributionTemplateScaffolding (Tier 2)
│   ├── test_domain_template_readme_role_declaration
│   ├── test_domain_template_readme_clean_landing_zone_guidance
│   ├── test_domain_template_readme_customer_onboarding_clone_command
│   └── test_domain_template_readme_in_place_update_command
├── TestCustomerWorkspaceScaffolding (Tier 3)
│   ├── test_customer_workspace_readme_role_declaration
│   ├── test_customer_workspace_readme_no_circular_clone
│   ├── test_customer_workspace_readme_local_project_commands
│   └── test_customer_workspace_git_origin_no_circular_reference
└── TestShellCodeFenceHygiene (Universal)
    ├── test_upstream_readme_shell_code_fences_syntax_valid
    ├── test_scaffolded_domain_readme_shell_code_fences_syntax_valid
    ├── test_scaffolded_customer_readme_shell_code_fences_syntax_valid
    ├── test_shell_comments_zero_unescaped_parentheses
    └── test_shell_commands_zero_unquoted_angle_brackets
```

### 4.2 Detailed Test Outlines

#### Suite 1: Upstream Compiler README (`DEAP01-spec-core/README.md`)
1. **`test_upstream_readme_no_inline_python_scripts`**:
   - Reads `DEAP01-spec-core/README.md`.
   - Asserts `python3 -c` and `python -c` do NOT occur anywhere in `README.md`.
   - Specifically asserts Section 5.4's 80-line inline Python script modifying `AGENTS.md` has been purged.
2. **`test_upstream_readme_no_domain_clone_commands`**:
   - Extracts all ```` ```bash ```` and ```` ```sh ```` code blocks from `README.md`.
   - Asserts no `git clone` command targets `DEAP-uas-infrastructure-safety` or any domain repository.
   - Asserts no customer onboarding commands instructing users to clone domain templates into customer workspaces exist in the compiler README.
3. **`test_upstream_readme_compiler_focus_commands`**:
   - Asserts `README.md` documents compiler verification: `python3 scripts/compile_sysml.py` and `pytest` / `unittest`.
   - Asserts `README.md` documents compiler tooling propagation into domain templates:
     `bash scripts/install_pipeline.sh <path-to-domain-template>`
     and remote bootstrap:
     `git clone https://github.com/gintatkinson/DEAP01-spec-core.git ... && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline`
4. **`test_upstream_readme_repository_classification`**:
   - Asserts `README.md` and `AGENTS.md` explicitly declare `UPSTREAM_SPEC_CORE_COMPILER`.

#### Suite 2: Domain Distribution Template Scaffolding (`DEAP-*`)
1. **`test_domain_template_readme_role_declaration`**:
   - Runs `bash scripts/install_pipeline.sh <temp_dir> --domain-name DEAP-test-domain`.
   - Asserts generated `<temp_dir>/README.md` declares:
     `> **Repository Role:** \`DOMAIN_DISTRIBUTION_TEMPLATE\``
     (and NOT `DOWNSTREAM_APPLICATION_WORKSPACE`).
2. **`test_domain_template_readme_clean_landing_zone_guidance`**:
   - Asserts generated README explains that this repository is a Domain Distribution Template with clean landing zones for schema and specifications.
3. **`test_domain_template_readme_customer_onboarding_clone_command`**:
   - Asserts generated README contains the authoritative single-line command for end-user customer onboarding:
     `git clone https://github.com/gintatkinson/DEAP-test-domain.git ./.tmp-pipeline && bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline`

#### Suite 3: Customer Application Workspace Scaffolding (`uav-*`)
1. **`test_customer_workspace_readme_role_declaration`**:
   - Runs installer targeting a customer workspace (e.g., `<temp_dir>` named `uav-011` or initialized with git remote `https://gitlab.com/customer/uav-011.git`, or via `--role DOWNSTREAM_CUSTOMER_PROJECT`).
   - Asserts generated `<temp_dir>/README.md` declares:
     `> **Repository Role:** \`DOWNSTREAM_CUSTOMER_PROJECT\``
2. **`test_customer_workspace_readme_no_circular_clone`**:
   - Asserts generated README does NOT contain any command cloning `uav-011` (e.g. `git clone ... uav-011.git`).
   - Asserts `README.md` does not instruct the developer inside `uav-011` to clone `uav-011` to install a pipeline.
3. **`test_customer_workspace_readme_local_project_commands`**:
   - Asserts generated README documents:
     - Baseline verification: `python3 scripts/verify_downstream_baseline.py --no-domain`
     - Step 0.0 Level 0 ingestion: `python3 skills/spec-orchestrator/scripts/sysmlv2_ingest.py`
     - In-place pipeline update: `bash scripts/install_pipeline.sh .`

#### Suite 4: Shell Code Fence Executability & Comment Hygiene
1. **`test_shell_code_fences_parse_cleanly`**:
   - Uses regex to extract all `bash` and `sh` code fences from:
     - Upstream `README.md`
     - Generated Domain Template `README.md`
     - Generated Customer Workspace `README.md`
   - For every code fence, runs `bash -n` via `subprocess.run(["bash", "-n"], input=fence_content, capture_output=True, text=True)`.
   - Asserts `proc.returncode == 0` for every block (guarantees executable shell syntax, no stray tokens, no unclosed quotes).
2. **`test_shell_comments_zero_unescaped_parentheses`**:
   - Iterates through all comment lines (`^\s*#.*`) inside shell code fences.
   - Searches for unescaped parentheses: `re.search(r'(?<!\\)[()]', line)`.
   - Asserts 0 violations. Ensures comments use plain text, hyphens, or escaped parentheses to prevent zsh subshell / command substitution parse failures.
3. **`test_shell_commands_zero_unquoted_angle_brackets`**:
   - Iterates through all executable command lines (non-comments) inside shell code fences.
   - Searches for unquoted angle-bracket placeholders (e.g. `<target-dir>` not enclosed in `"..."` or `'...'`).
   - Asserts 0 occurrences, preventing shell input redirection parse errors (`bash: syntax error near unexpected token 'newline'`).

---

## 5. Implementation Recommendations for Downstream Tools

1. **`scripts/install_pipeline.sh` Dynamic Role Detection**:
   - Implement tier detection logic:
     - If `$TARGET_DIR` name starts with `DEAP-` or `--domain-name` starts with `DEAP-` or explicit `--role DOMAIN_DISTRIBUTION_TEMPLATE`:
       Set `REPO_ROLE="DOMAIN_DISTRIBUTION_TEMPLATE"`.
     - Otherwise:
       Set `REPO_ROLE="DOWNSTREAM_CUSTOMER_PROJECT"`.
   - Branch README generation:
     - In `DOMAIN_DISTRIBUTION_TEMPLATE`: emit `Repository Role: DOMAIN_DISTRIBUTION_TEMPLATE`, clean landing zone explanation, and the customer clone command targeting the domain remote.
     - In `DOWNSTREAM_CUSTOMER_PROJECT`: emit `Repository Role: DOWNSTREAM_CUSTOMER_PROJECT`, omit the customer clone command, and emit local workspace commands (`verify_downstream_baseline.py`, `sysmlv2_ingest.py`, and `bash scripts/install_pipeline.sh .`).
2. **`scripts/verify_downstream_baseline.py` Check 14 Enhancement**:
   - Expand Check 14 to validate:
     - Proper role declaration matching repository structure.
     - Pure executable shell code fences (`bash -n`).
     - Absence of unescaped parentheses in comments within code fences.

---

## 6. Verification Method

To verify the test suite once implemented:
```bash
# Run unit test suite
python3 -m unittest tests/test_readme_scaffolding.py

# Run full project test discovery
python3 -m unittest discover tests

# Run baseline verification gate
python3 scripts/verify_downstream_baseline.py --no-domain
```
