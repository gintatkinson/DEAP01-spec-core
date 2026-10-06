---
issue_id: 422
title: "Solution Walkthrough: Lossless SysML v2 AST Compilation and Resilient Validators"
version: "1.0"
date: "2026-10-06"
---

# Solution Walkthrough: Lossless SysML v2 AST Compilation and Resilient Validators (Issue #422)

## 1. Overview & Problem Context

During SysML v2 model compilation and downstream round-trip synchronization, several AST serialization regressions and environment dependency vulnerabilities were identified (Issue #422):

1. **Lossy Part Usage Serialization**:
   - Typed part usages (e.g. `part fcc_board : AutopilotController;` or compound block usages `part payload : GimbalPayload { ... }`) were parsed without recording instance usage semantics.
   - Upon serialization via `PartDef.to_sysml()`, typed usages degraded into empty definition blocks (`part def fcc_board { }`), discarding instance type definitions and destroying hierarchical system architecture semantics.

2. **Bidirectional Port Modifier Loss**:
   - Ports declared with bidirectional flow directions (`inout port <name> : <Type>;`) lost their direction modifier during serialization.
   - `PortDefinition.to_sysml()` omitted direction prefix serialization for `inout`, rendering bidirectional hardware buses (RS-485, DroneCAN, Ethernet) directionless upon re-compilation.

3. **Circular Action Def Performer Invocations**:
   - `SysMLParser._parse_action_def()` defaulted performer/allocation attributes to the parent part container name.
   - This caused `action def` constructs to synthesize circular self-performing calls (`perform <ParentPart>;`), corrupting execution sequence semantics.

4. **Brittle Top-Level PyYAML Imports in Core Validators**:
   - Modules across `parity_auditor` (`uml.py`, `spec_validator.py`, `cardinality_validator.py`, `logical_ui_validator.py`) and `reconcile_backlog.py` relied on unconditional top-level `import yaml`.
   - In environments without PyYAML pre-installed, baseline verification crashed immediately, violating the lightweight zero-dependency verification mandate.

5. **Absence of Dedicated Semantic Parity Regression Suite**:
   - No automated unit/regression suite existed to prove multi-generation AST round-trip idempotency and lossless fidelity.

---

## 2. Changes Implemented

### 2.1 Lossless SysML v2 AST Parser & Serializer (`skills/spec-orchestrator/scripts/sysmlv2_ast.py`)

- **Typed Part Usage Modeling**:
  - Extended `PartDef` data class with `is_def: bool = True` and `type_name: Optional[str] = None`.
  - Updated `SysMLParser._parse_part_def()` regex and tokenizer to identify both single-line usages (`part <name> : <Type>;`) and block usages (`part <name> : <Type> { ... }`).
  - Flagged usages with `is_def = False` and captured `type_name`.
  - In `PartDef.to_sysml()`:
    - If `not self.is_def` and `self.type_name`:
      - For single-line usages (no body statements), emit `part <name> : <type_name>;`.
      - For block usages, emit `part <name> : <type_name> { ... }`.
    - Preserved `part def <name> { ... }` emission for structural definitions.

- **Bidirectional Port Direction Preservation**:
  - In `PortDefinition.to_sysml()`, ensured that `dir_prefix = f"{self.direction} "` prepends all recognized direction keywords (`in`, `out`, and `inout`).
  - Serializes `inout port <name> : <Type>;` with lossless fidelity.

- **Clean Action Def Allocation**:
  - In `SysMLParser._parse_action_def()`, set `performer = "" if is_def else parent_name`.
  - Eliminates synthetic self-referential `perform <ParentPart>;` calls inside `action def` declarations.

### 2.2 Resilient Dependency Handling Across Validators

Guarded `import yaml` with `try...except ImportError: yaml = None` and robust line-based YAML frontmatter parsing fallbacks across:
- `skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/uml.py`
- `skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/spec_validator.py`
- `skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/cardinality_validator.py`
- `skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/logical_ui_validator.py`
- `skills/spec-orchestrator/scripts/reconcile_backlog.py`
- `scripts/reconcile_backlog.py`

When `yaml` is unavailable, frontmatter boundaries (`---`) are parsed cleanly without crashing baseline verification gates.

### 2.3 Installer Staging Hardening (`scripts/installer/staging.py`)

- Added `tests/test_sysml_compiler_parity.py` to `PARITY_TEST_FILES` manifest.
- Guaranteed automated propagation of compiler parity tests to all downstream installations.

### 2.4 Semantic AST Round-Trip Parity Test Suite (`tests/test_sysml_compiler_parity.py`)

Authored 11 comprehensive automated tests:
1. `test_part_usage_typed_single_line_parsed_as_usage`: Verifies `is_def=False` and `type_name`.
2. `test_part_usage_typed_single_line_serializes_cleanly`: Confirms `part x : Type;` round-trip.
3. `test_part_usage_typed_block_parsed_and_serialized`: Confirms block usage round-trip.
4. `test_part_def_preserves_definition_syntax`: Confirms `part def` definition round-trip.
5. `test_inout_port_direction_preserved_on_serialization`: Confirms `inout` port modifier retention.
6. `test_in_and_out_port_directions_preserved`: Confirms directional ports retention.
7. `test_action_def_does_not_inject_parent_performer`: Confirms absence of circular self-invocation.
8. `test_action_usage_preserves_performer_call`: Confirms action usage calls retain performers.
9. `test_resilient_yaml_import_fallback_behavior`: Confirms validator execution when `yaml is None`.
10. `test_roundtrip_ast_idempotency_multigen`: Proves 3-generation serialization produces zero diffs.
11. `test_full_model_ast_preservation`: Validates complete complex system package round-trip.

---

## 3. Code Realization Table

| Component / Artifact | Source File | Implementation Details | Verification Gate |
|---|---|---|---|
| AST Typed Part Usages | `skills/spec-orchestrator/scripts/sysmlv2_ast.py` | `PartDef` (`is_def`, `type_name`), `_parse_part_def`, `to_sysml` | Parity Tests 1-3 |
| Port `inout` Preservation | `skills/spec-orchestrator/scripts/sysmlv2_ast.py` | `PortDefinition.to_sysml()` direction prefix | Parity Tests 5-6 |
| Action Def Clean Allocation | `skills/spec-orchestrator/scripts/sysmlv2_ast.py` | `_parse_action_def()` performer guard | Parity Tests 7-8 |
| Resilient UML Validator | `.../validators/uml.py` | `try...except ImportError` fallback | Baseline Check 13 |
| Resilient Spec Validator | `.../validators/spec_validator.py` | Line-based frontmatter parser fallback | Baseline Check 16 |
| Resilient Cardinality Validator | `.../validators/cardinality_validator.py` | Null-safe YAML loader | Baseline Check 24 |
| Resilient LUI Validator | `.../validators/logical_ui_validator.py` | Regex-assisted frontmatter reader | Baseline Check 28 |
| Installer Test Staging | `scripts/installer/staging.py` | Staged `test_sysml_compiler_parity.py` | Semantic Acceptance |
| Parity Regression Suite | `tests/test_sysml_compiler_parity.py` | 11 unit/integration tests | 11/11 Green (0.34s) |

---

## 4. Verification & Testing Evidence

### 4.1 Unit & Parity Test Suite
- Command: `python3 -m unittest tests/test_sysml_compiler_parity.py -v`
- Results: **11/11 tests passing cleanly (0 failures, 0 errors, 0.34s execution time)**.

### 4.2 Baseline Conformance Checks
- Command: `python3 scripts/verify_downstream_baseline.py --no-domain`
- Results: **All 31 mechanical baseline checks passed with exit code 0**.

### 4.3 Fleet-Wide Parity & Remote Synchronization
The verified tooling and test suite were propagated across all active repositories in the fleet with exact SHA-256 byte parity:
- `DEAP01-spec-core` (branch `main`)
- `uav-009` (branch `main`)
- `uav-011` (branch `main`)
- `uav-012` (branch `0A`)
- `DEAP-netcom` (branch `main`)
- `3dgs-040` (branch `main`)
- `deap-compiler-spec` (branch `main`)

### 4.4 Governance & Tracker Conformance
- **Zero Em Dash Invariant**: Verified 0 occurrences of Unicode `\u2014` across all changes.
- **Commit Neutrality**: Used neutral citation `(refs #422)` with zero auto-closing verbs.
- **Issue Tracker State**: GitHub Issue #422 updated with verification evidence comment and labeled `status:fixed-resolved`.
