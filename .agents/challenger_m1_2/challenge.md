# Adversarial Challenge Report: README.md Correctness & Consistency (M1.2)

**Evaluator**: Empirical Challenger (`challenger_m1_2`)  
**Target**: `README.md` in `DEAP01-spec-core` (Work Package 1 / R1)  
**Date**: 2026-09-21T16:45:00Z  
**Verdict**: **APPROVE** (with low-risk advisory observations)

---

## Challenge Summary

**Overall risk assessment**: **LOW**

Adversarial testing confirmed that the changes to `README.md` under Work Package 1 (R1) successfully purged the fragile 80-line inline Python monkeypatching script and manual `cp` loops from Section 5.4, established a clean compiler-centric documentation focus, and removed conflated domain customer clone commands from upstream compiler sections.

All core documented commands run as expected. The turnkey installer `scripts/install_pipeline.sh` robustly handles directory arguments across edge cases (existing directories, non-existent nested paths, paths with spaces, and initialized git repositories), and properly refuses self-overwrites when pointed at the compiler itself (`.`).

---

## Challenges

### [Low] Challenge 1: Section 4 Anchor Link Mismatch in Section 5.4

- **Assumption challenged**: That the markdown cross-reference link `[Section 4](#4-supported-tier-1-domain-distribution-templates-canonical-taxonomies)` at line 291 correctly resolves to a heading anchor in `README.md`.
- **Attack scenario**: A user clicking the cross-reference link in GitHub or a markdown viewer will experience a dead anchor link that does not scroll to Section 4.
- **Blast radius**: Minor navigation inconvenience; does not break command execution or compilation.
- **Root Cause**: The Section 4 heading is `## 4. Repository Structure & Canonical Specifications` (GitHub anchor: `#4-repository-structure--canonical-specifications`). The text "Supported Tier 1 Domain Distribution Templates" at line 177 is plain body text, not a markdown heading.
- **Mitigation**: Update the link anchor at line 291 to `[Section 4](#4-repository-structure--canonical-specifications)` or add a subsection heading `### 4.1 Supported Tier 1 Domain Distribution Templates`.

### [Low] Challenge 2: Legacy Unit Test Path in Worker 0A Operator Prompt (Section 9.1.1)

- **Assumption challenged**: That the validation command `python3 -m unittest tests.test_conops_and_mission_intent_validators` documented in the Worker 0A prompt (line 701) can be executed as written.
- **Attack scenario**: An operator or subagent copying the Worker 0A prompt runs the command and receives:
  `ModuleNotFoundError: No module named 'tests.test_conops_and_mission_intent_validators'`.
- **Blast radius**: Pre-existing operator prompt copy-paste failure if Worker 0A is executed manually using that specific command. Note: This reference predates M1 (originating in commit `6a233462d`). Gate 26 validation is currently handled by `python3 skills/spec-orchestrator/scripts/verify_model_coverage.py --gate 26 --spec-only` or `python3 scripts/verify_downstream_baseline.py`.
- **Mitigation**: Update line 701 in Worker 0A prompt to cite `python3 skills/spec-orchestrator/scripts/verify_model_coverage.py --gate 26 --spec-only`.

### [Low] Challenge 3: Relative Import Failure in Worker 0D Validator Command (Section 9.1.4)

- **Assumption challenged**: That executing `python3 skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/icd_completeness_validator.py` as a standalone script succeeds without `PYTHONPATH`.
- **Attack scenario**: Directly running the script as written at line 797 triggers:
  `ImportError: attempted relative import with no known parent package`.
- **Blast radius**: Fails when executed as an independent script outside the package module context.
- **Mitigation**: Update line 797 in Worker 0D prompt to cite `PYTHONPATH=skills/spec-orchestrator/parity_auditor/src python3 -m parity_auditor.cli --workspace . --allow-missing-specs`.

---

## Stress Test Results

### 1. Documented Verification Commands

| Command | Expected Result | Actual Result | Status |
|---|---|---|---|
| `python3 scripts/compile_sysml.py --compile` | Exit code 1 with clear remediation directing to Step 0.0 ingestion (since `schema/` in `DEAP01-spec-core` is clean landing zone with `.gitkeep` only) | Exit code 1; printed exact 3-step remediation message to stderr | **PASS** |
| `python3 scripts/compile_sysml.py --schema /tmp/valid.sysml --compile` | Exit code 0; compiles schema to `.pipeline/schema.sysml` and `schema-digest.json` | Exit code 0; generated valid AST digest and `.pipeline/schema.sysml` | **PASS** |
| `python3 -m pytest tests/` | Exit code 0; all unit/integration tests pass | Exit code 0; 23 passed in 6.46s | **PASS** |
| `python3 scripts/verify_downstream_baseline.py --no-domain` | Exit code 0; all 30 baseline checks pass | Exit code 0; all 30 checks verified cleanly | **PASS** |

### 2. `scripts/install_pipeline.sh` Directory Argument Edge Cases

| Scenario | Invocation | Expected Behavior | Actual Behavior | Status |
|---|---|---|---|---|
| **Case A: Existing Empty Directory** | `bash scripts/install_pipeline.sh "/tmp/test_dir_a"` | Installs full pipeline infrastructure; exit code 0 | Exit code 0; skills, rules, .pipeline, scripts, AGENTS.md copied | **PASS** |
| **Case B: Non-Existent Nested Directory** | `bash scripts/install_pipeline.sh "/tmp/test_dir_b/nested/sub"` | Creates directory hierarchy and installs; exit code 0 | Exit code 0; created path and installed cleanly | **PASS** |
| **Case C: Directory with Spaces** | `bash scripts/install_pipeline.sh "/tmp/test domain dir c"` | Quoting handles spaces without path splitting; exit code 0 | Exit code 0; correctly handled path with spaces | **PASS** |
| **Case D: Initialized Git Repository** | `bash scripts/install_pipeline.sh "/tmp/test_domain_git"` | Detects git remote, auto-configures platform, installs hooks & tracker labels; exit code 0 | Exit code 0; detected GitHub, installed pre-commit/commit-msg hooks, provisioned 5 labels | **PASS** |
| **Case E: Target Is Compiler Root (`.`)** | `bash scripts/install_pipeline.sh .` inside `DEAP01-spec-core` | Refuses self-overwrite due to `.pipeline/upstream`; exit code 1 | Exit code 1; printed `REFUSING: target is the pipeline repository itself, not a downstream project.` | **PASS** |

### 3. Syntax, Hygiene, and Code Block Verification

| Check | Tool / Method | Findings | Status |
|---|---|---|---|
| **Inline Python Script Purge** | Regex search for `import json`, `sys.argv`, `open(p)` in `README.md` | Zero inline Python scripts found | **PASS** |
| **Manual `cp` Loop Purge** | Regex search for `for d in` and `cp -r` in `README.md` | Zero manual `cp` loops found | **PASS** |
| **Shell Syntax & Code Blocks** | 37 code fences scanned for unquoted angle brackets and unescaped parentheses in comments | Zero unquoted `<...>` placeholders; zero unescaped parentheses in comments | **PASS** |
| **Markdown Local File Links** | Verified all relative file links in `README.md` against filesystem | All local file links resolve to real files on disk | **PASS** |

---

## Unchallenged Areas

- **Work Package 2 (Dynamic Role Detection in `scripts/install_pipeline.sh`)**: Currently queued for Milestone 2. Tested only existing argument handling and installer behavior; new role detection logic is not yet implemented.
- **Remote Synchronization**: Git pushing to remote tracking branches is deferred to the coordinator following milestone review.

---

## Final Verdict

**APPROVE**  
`DEAP01-spec-core/README.md` satisfies all requirements of R1 with zero blocking flaws, robust command execution, and verified script parameter handling.
