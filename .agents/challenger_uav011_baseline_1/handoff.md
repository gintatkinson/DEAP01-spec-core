# Handoff Report — Empirical Baseline Gate Verification for Application Workspace uav-011 (WP-05)

## 1. Observation

### 1.1 Direct Baseline Verification Command & Result (Default Invocation)
- **Command executed**:
  ```bash
  python3 /Users/perkunas/jail/DEAP01-spec-core/scripts/verify_downstream_baseline.py /Users/perkunas/jail/uav-011
  ```
- **Exit Code**: `1` (FAIL)
- **Verbatim Output**:
  ```text
  NOTE: Destination path '/Users/perkunas/jail/uav-011' has no pubspec.yaml or package.json. Registering repository root for non-framework baseline checks.
  Success: Check 10 verified (.gitignore exists in repository root).
  Success: Check 11 verified (zero .DS_Store files found).
  Success: Check 12 verified (no duplicate master core blueprints found).
  Success: Check 13 verified (KaTeX / LaTeX mathematical syntax valid across all markdown files, including rules/sysml-ssot-completeness.md).
  Success: Mermaid syntax verified across all markdown files.
  Success: Check 14 verified (README.md, agent instruction entrypoints, and rules/sysml-ssot-completeness.md exist).
  Success: Check 15 verified (scripts/reconcile_backlog.py exists, is non-empty, and is executable).
  Success: Check 16 verified (Downstream repository detected -- skipping upstream clean landing zone gate).
  ERROR: Check 17 failed: Safety specification directory 'docs/safety/' is missing.
  Cleaning up workspace...
  ```

### 1.2 Direct Baseline Verification with `--allow-missing-specs`
- **Command executed**:
  ```bash
  python3 /Users/perkunas/jail/DEAP01-spec-core/scripts/verify_downstream_baseline.py /Users/perkunas/jail/uav-011 --allow-missing-specs
  ```
- **Exit Code**: `1` (FAIL)
- **Verbatim Output**:
  ```text
  NOTE: Destination path '/Users/perkunas/jail/uav-011' has no pubspec.yaml or package.json. Registering repository root for non-framework baseline checks.
  Success: Check 10 verified (.gitignore exists in repository root).
  Success: Check 11 verified (zero .DS_Store files found).
  Success: Check 12 verified (no duplicate master core blueprints found).
  Success: Check 13 verified (KaTeX / LaTeX mathematical syntax valid across all markdown files, including rules/sysml-ssot-completeness.md).
  Success: Mermaid syntax verified across all markdown files.
  Success: Check 14 verified (README.md, agent instruction entrypoints, and rules/sysml-ssot-completeness.md exist).
  Success: Check 15 verified (scripts/reconcile_backlog.py exists, is non-empty, and is executable).
  Success: Check 16 verified (Downstream repository detected -- skipping upstream clean landing zone gate).
  Success: Check 17 verified (Downstream repository detected -- docs/safety/ directory not present).
  Success: Check 18 verified (Downstream repository detected -- skipping upstream blueprint domain cleanliness gate).
  Success: Check 19 verified (Downstream repository detected -- skipping domain-agnostic AST cleanliness gate).
  Success: Check 20 verified (WBS & Enterprise Deliverables Suite pending or not present).
  Success: Check 21 verified (Semantic Diagram-to-AST Topology Parity Gate passed -- zero undeclared nodes, inverted flows, or ungrounded actuators).
  Success: Check 22 verified (Physical Invariant Semantic Prose Gate passed -- zero ungrounded operational assertions).
  Success: Check 23 verified (Factual Grounding & Numeric Provenance Gate passed -- zero ungrounded assertions).
  Success: Level 1C ICD Completeness verified (Downstream repository detected -- docs/interfaces/ directory not present).
  Success: Check 24 verified (Operational-to-Resource Allocation passed -- zero orphan activities or phantom allocation tags).
  Success: Check 25 verified (Standards & SI 7D Parameter Metrology passed -- all parameter dimensions, units, and SDO baselines valid).
  Success: Check 25 verified (Cross-Document Diagram Parity Gate passed -- zero disparity in subgraphs, nodes, ports, or connections).
  Success: Check 26 verified (Downstream repository detected -- docs/conops/ directory not present).
  Success: Check 27 verified (Cited Research Inventory & Declared-Total Population Register passed).
  Success: Check 27 verified (Executive Deliverable Traceability Gate passed -- all tables and diagrams anchored to SSOT).
  Success: Check 28 verified (Coverage-Digest Population Gate passed -- zero phantom realizations).
  Success: Check 29 verified (Obligation-Witness Registry Gate passed -- zero phantom witnesses).
  ERROR: Check 30 failed (Architecture Viewpoint & Diagram Completeness violations found):
    - Architecture specification corpus is missing in workspace ('docs/conops', 'docs/interfaces', 'docs/safety').
  Cleaning up workspace...
  ```

### 1.3 Local Workspace Script Verification (`uav-011/scripts/verify_downstream_baseline.py`)
- **Command executed**:
  ```bash
  python3 /Users/perkunas/jail/uav-011/scripts/verify_downstream_baseline.py /Users/perkunas/jail/uav-011
  ```
- **Exit Code**: `1` (FAIL)
- **Output**: Identical failure at Check 17: `ERROR: Check 17 failed: Safety specification directory 'docs/safety/' is missing.`

### 1.4 Isolated Check Harness Execution
Because `verify_downstream_baseline.py` exits fail-closed (`sys.exit(1)`) on the first gate error encountered (Check 17 in default mode, Check 30 in `--allow-missing-specs` mode), subsequent checks are bypassed during CLI execution. An isolated empirical test harness was executed to run each check independently against `/Users/perkunas/jail/uav-011`:

| Check Identifier | Gate Name | Isolated Result (Default) | Isolated Result (`--allow-missing-specs`) | Verbatim Finding / Detail |
|---|---|---|---|---|
| **Check 10** | `.gitignore` existence | **PASS** | **PASS** | `.gitignore` exists in repository root |
| **Check 11** | Zero `.DS_Store` files | **PASS** | **PASS** | Zero `.DS_Store` files found |
| **Check 12** | Master blueprints duplicate check | **PASS** | **PASS** | No duplicate master core blueprints found |
| **Check 13** | KaTeX / LaTeX mathematical syntax | **PASS** | **PASS** | Valid mathematical syntax across markdown files |
| **Mermaid** | Offline Mermaid syntax gate | **PASS** | **PASS** | Valid Mermaid syntax across markdown files |
| **Check 14** | Instructions & entrypoint existence | **PASS** | **PASS** | README.md and entrypoints exist |
| **Check 15** | `scripts/reconcile_backlog.py` existence | **PASS** | **PASS** | File exists, non-empty, and executable |
| **Check 16** | Clean landing zone gate | **PASS** | **PASS** | Downstream repository detected -- skipped |
| **Check 17** | Safety Integrity Quality Gate & SORA | **FAIL** (Exit 1) | **PASS** | Default: `Safety specification directory 'docs/safety/' is missing.`<br/>With `--allow-missing-specs`: `docs/safety/ directory not present` |
| **Check 18** | Blueprint domain cleanliness | **PASS** | **PASS** | Downstream repository detected -- skipped |
| **Check 19** | Domain-agnostic AST cleanliness | **PASS** | **PASS** | Downstream repository detected -- skipped |
| **Check 20** | WBS & Enterprise Deliverables Suite | **FAIL** (Exit 1) | **PASS** | Default: `WBS & Enterprise Deliverables Suite ('docs/management/WBS_DELIVERABLES_SUITE.md') is missing in downstream customer mode.`<br/>With `--allow-missing-specs`: `WBS & Enterprise Deliverables Suite pending or not present.` |
| **Check 21** | Semantic Diagram-to-AST Topology Parity | **PASS** | **PASS** | Zero undeclared nodes, inverted flows, or ungrounded actuators |
| **Check 22** | Physical Invariant Semantic Prose | **PASS** | **PASS** | Zero ungrounded operational assertions |
| **Check 23** | Factual Grounding & Numeric Provenance | **PASS** | **PASS** | Zero ungrounded assertions |
| **Gate 23 (ICD)** | Level 1C ICD Completeness Gate | **PASS** | **PASS** | Downstream repository detected -- `docs/interfaces/` not present |
| **Check 24** | Operational-to-Resource Allocation | **PASS** | **PASS** | Zero orphan activities or phantom allocation tags |
| **Check 25A** | Standards & SI 7D Parameter Metrology | **PASS** | **PASS** | All parameter dimensions, units, and SDO baselines valid |
| **Check 25B** | Cross-Document Diagram Parity Gate | **PASS** | **PASS** | Zero disparity in subgraphs, nodes, ports, or connections |
| **Check 26** | ConOps & Mission Intent Completeness | **PASS** | **PASS** | Downstream repository detected -- `docs/conops/` not present |
| **Check 27A** | Cited Research Inventory & Population Register | **PASS** | **PASS** | Valid |
| **Check 27B** | Executive Deliverable Traceability Gate | **PASS** | **PASS** | All tables and diagrams anchored to SSOT |
| **Check 28** | Coverage-Digest Population Gate | **PASS** | **PASS** | Zero phantom realizations |
| **Check 29** | Obligation-Witness Registry Gate | **PASS** | **PASS** | Zero phantom witnesses |
| **Check 30** | Architecture Viewpoint & Diagram Compl. | **FAIL** (Exit 1) | **FAIL** (Exit 1) | `Architecture specification corpus is missing in workspace ('docs/conops', 'docs/interfaces', 'docs/safety').` |
| **Check 31** | Dual-Schema SSOT Parity Gate | **PASS** | **PASS** | `schema/*.sysml` and `.pipeline/schema.sysml` AST definitions are identical |

### 1.5 Isolated Execution of Check 31 (Dual-Schema SSOT Parity Gate)
- **Command executed**:
  ```python
  import sys
  sys.path.insert(0, '/Users/perkunas/jail/DEAP01-spec-core/scripts')
  from verify_downstream_baseline import check_dual_schema_ssot_parity
  check_dual_schema_ssot_parity('/Users/perkunas/jail/uav-011')
  ```
- **Exit Code**: `0` (PASS)
- **Verbatim Output**:
  ```text
  Success: Check 31 verified (Dual-Schema SSOT Parity Gate passed -- schema/*.sysml and .pipeline/schema.sysml AST definitions are identical).
  ```

---

## 2. Logic Chain

1. **Premise 1 (Test Objective)**: Task WP-05 specifies running `python3 /Users/perkunas/jail/DEAP01-spec-core/scripts/verify_downstream_baseline.py /Users/perkunas/jail/uav-011`, recording verbatim output and exit code, testing with flags (such as `--allow-missing-specs`), and documenting the status of Checks 10 through 31.
2. **Observation 1.1**: Under default invocation without flags, `verify_downstream_baseline.py` exits at **Check 17** with `exit code 1` because `docs/safety/` is missing in `/Users/perkunas/jail/uav-011`.
3. **Observation 1.2**: In `scripts/verify_downstream_baseline.py`, line 2084 (`check_safety_integrity_and_sora_completeness`) enforces that downstream repositories require `docs/safety/` unless `effective_allow_missing` is True (`allow_missing_specs and not is_strict`).
4. **Observation 1.3**: When `--allow-missing-specs` is supplied, Check 17 and Check 20 pass cleanly (recognizing that specifications are pending/clean). Checks 21, 22, 23, 24, 25A, 25B, 26, 27A, 27B, 28, and 29 all pass cleanly.
5. **Observation 1.4**: Execution terminates at **Check 30** with `exit code 1` with the error:
   `ERROR: Check 30 failed (Architecture Viewpoint & Diagram Completeness violations found):`
   `  - Architecture specification corpus is missing in workspace ('docs/conops', 'docs/interfaces', 'docs/safety').`
6. **Observation 1.5 & Code Inspection**:
   - In `scripts/verify_downstream_baseline.py`:
     - Line 3644: `check_architecture_viewpoint_diagrams(repo_root)` is called without passing `allow_missing_specs` or `strict`.
     - Line 3450: `check_architecture_viewpoint_diagrams(repo_root=None)` hardcodes `allow_missing_specs=False` on line 3465: `validator.validate(repo, allow_missing_specs=False, spec_only=True)`.
   - In `skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/architecture_viewpoint_validator.py`:
     - Lines 1239–1246:
       ```python
       # Upstream Clean Landing Zone Invariant:
       if is_upstream:
           if not active_spec_files:
               return []

       # If downstream and no specification documents found
       if not active_spec_files:
           return [Finding(
               RULE_CORPUS_MISSING,
               "Architecture specification corpus is missing in workspace ('docs/conops', 'docs/interfaces', 'docs/safety').",
               location="docs/conops",
               detail={"target_dirs": target_dirs}
           )]
       ```
     - For downstream repositories, if `active_spec_files` is empty, it unconditionally emits `RULE_CORPUS_MISSING`, completely ignoring `allow_missing_specs` even if it were passed into `validate()`.
7. **Observation 1.6 (Check 31 Behavior)**:
   - When run in isolation, Check 31 (`check_dual_schema_ssot_parity`) passes with exit code 0.
   - In `uav-011`, `schema/*.sysml` (`model.sysml`, `DEAP_MODEL.sysml`) and `.pipeline/schema.sysml` declare identical AST definitions (`part def`, `port def`, `action def`, `item def`), satisfying the Dual-Schema SSOT Parity Gate.
8. **Deduction**: The hypothesis that `verify_downstream_baseline.py /Users/perkunas/jail/uav-011` exits with code 0 is **empirically false**.
   - Default run fails at **Check 17** (missing `docs/safety/`).
   - Run with `--allow-missing-specs` fails at **Check 30** (missing architecture corpus in clean landing zone downstream workspace).

---

## 3. Caveats

- **Review-Only Constraint**: Adhering strictly to the `Review-only — do NOT modify implementation code` constraint and AGENTS.md coordinator/challenger boundaries, no edits were made to `verify_downstream_baseline.py`, `architecture_viewpoint_validator.py`, or `/Users/perkunas/jail/uav-011` files.
- **Clean Landing Zone Workspace**: Unlike `uav-009` (which contains 75 generated specifications and safety matrices), `uav-011` is currently an unelaborated application workspace where specification landing zones (`docs/epics/`, `docs/features/`, `docs/user-stories/`, `docs/use-cases/`) maintain clean `.gitkeep` files, and `docs/safety/`, `docs/conops/`, and `docs/interfaces/` do not yet exist.
- **Check 31 Unreachable in CLI Run**: Because `verify_downstream_baseline.py` exits fail-closed at Check 17 (or Check 30 with `--allow-missing-specs`), Check 31 is never reached during unified execution of `verify_downstream_baseline.py`, although it passes when executed in isolation.

---

## 4. Conclusion

1. **Overall Run Result**:
   - `python3 /Users/perkunas/jail/DEAP01-spec-core/scripts/verify_downstream_baseline.py /Users/perkunas/jail/uav-011` **FAILED** with `exit code 1` at **Check 17**.
   - `python3 /Users/perkunas/jail/DEAP01-spec-core/scripts/verify_downstream_baseline.py /Users/perkunas/jail/uav-011 --allow-missing-specs` **FAILED** with `exit code 1` at **Check 30**.
2. **Gate Summary (Checks 10–31)**:
   - **Passing Gates (21 checks)**: Checks 10, 11, 12, 13, Mermaid, 14, 15, 16, 18, 19, 21, 22, 23 (factual grounding), Gate 23 (ICD completeness), 24, 25A, 25B, 26, 27A, 27B, 28, 29, 31.
   - **Failing Gates in Default Mode (2 checks)**:
     - Check 17: Fails because `docs/safety/` is missing in downstream mode when `--allow-missing-specs` is not provided.
     - Check 20: Fails because `docs/management/WBS_DELIVERABLES_SUITE.md` is missing when `--allow-missing-specs` is not provided.
   - **Failing Gate in `--allow-missing-specs` Mode (1 check)**:
     - Check 30: Fails because `check_architecture_viewpoint_diagrams` and `ArchitectureViewpointValidator` fail closed when no architecture specifications exist in a downstream workspace, even when `--allow-missing-specs` is specified.
3. **Check 31 (Dual-Schema SSOT Parity)**:
   - **Verified PASS**: Identical AST definitions between `schema/*.sysml` and `.pipeline/schema.sysml`.
4. **Actionable Recommendation for Parent Orchestrator**:
   - In `scripts/verify_downstream_baseline.py`:
     1. Pass `allow_missing_specs` and `strict` into `check_architecture_viewpoint_diagrams(repo_root, allow_missing_specs=allow_missing_specs, strict=strict)`.
     2. Update `architecture_viewpoint_validator.py` (lines 1240–1246) so that when `allow_missing_specs=True`, a downstream repository with clean landing zones and no active spec files returns an empty finding list (or informational notice) instead of a fatal `RULE_CORPUS_MISSING` finding.
   - Alternatively, if `uav-011` is expected to pass all baseline checks under default mode (without `--allow-missing-specs`), the specification orchestration pipeline (Phases 0.5, 0.75, 1, 1.5, 2, 3) must be run on `uav-011` to synthesize `docs/safety/`, `docs/conops/`, `docs/interfaces/`, and `docs/management/WBS_DELIVERABLES_SUITE.md`.

---

## 5. Verification Method

To reproduce and independently verify these findings, execute the following commands from `/Users/perkunas/jail/DEAP01-spec-core`:

1. **Verify Default Verification Failure at Check 17 (Exit Code 1)**:
   ```bash
   python3 scripts/verify_downstream_baseline.py /Users/perkunas/jail/uav-011
   echo "Exit code: $?"
   ```
   *Expected output*: Terminates with `ERROR: Check 17 failed: Safety specification directory 'docs/safety/' is missing.` and exit code `1`.

2. **Verify `--allow-missing-specs` Failure at Check 30 (Exit Code 1)**:
   ```bash
   python3 scripts/verify_downstream_baseline.py /Users/perkunas/jail/uav-011 --allow-missing-specs
   echo "Exit code: $?"
   ```
   *Expected output*: Checks 10–29 pass, terminates with `ERROR: Check 30 failed... Architecture specification corpus is missing in workspace ('docs/conops', 'docs/interfaces', 'docs/safety').` and exit code `1`.

3. **Verify Isolated Check 31 Pass (Exit Code 0)**:
   ```bash
   python3 -c "
   import sys; sys.path.insert(0, 'scripts')
   from verify_downstream_baseline import check_dual_schema_ssot_parity
   check_dual_schema_ssot_parity('/Users/perkunas/jail/uav-011')
   "
   ```
   *Expected output*: `Success: Check 31 verified (Dual-Schema SSOT Parity Gate passed -- schema/*.sysml and .pipeline/schema.sysml AST definitions are identical).`

4. **Verify Full Isolated Check Matrix**:
   ```bash
   python3 -c "
   import sys; sys.path.insert(0, 'scripts')
   import verify_downstream_baseline as v
   target = '/Users/perkunas/jail/uav-011'
   for name, fn in [
       ('Check 17 default', lambda: v.check_safety_integrity_and_sora_completeness(target, allow_missing_specs=False)),
       ('Check 17 allow_missing', lambda: v.check_safety_integrity_and_sora_completeness(target, allow_missing_specs=True)),
       ('Check 20 default', lambda: v.check_wbs_suite_integrity(target, allow_missing_specs=False)),
       ('Check 20 allow_missing', lambda: v.check_wbs_suite_integrity(target, allow_missing_specs=True)),
       ('Check 30', lambda: v.check_architecture_viewpoint_diagrams(target)),
       ('Check 31', lambda: v.check_dual_schema_ssot_parity(target)),
   ]:
       try:
           fn()
           print(f'{name}: PASS')
       except SystemExit as e:
           print(f'{name}: FAIL (code {e.code})')
   "
   ```
