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

4. **Brittle Top-Level YAML Imports in Core Validators**:
   - Modules across `crates/verify-baseline` and `crates/reconcile-backlog` required resilient YAML frontmatter parsing without external runtime dependencies.
   - In environments without external interpreters pre-installed, baseline verification must execute natively as a self-contained binary.

5. **Absence of Dedicated Semantic Parity Regression Suite**:
   - No automated unit/regression suite existed to prove multi-generation AST round-trip idempotency and lossless fidelity.

---

## 2. Changes Implemented

### 2.1 Lossless SysML v2 AST Parser & Serializer (`crates/sysml-core`)

- **Typed Part Usage Modeling**:
  - Extended `PartDef` data structure with `is_def: bool = true` and `type_name: Option<String>`.
  - Updated `SysMLParser` tokenizer to identify both single-line usages (`part <name> : <Type>;`) and block usages (`part <name> : <Type> { ... }`).
  - Flagged usages with `is_def = false` and captured `type_name`.
  - In `PartDef::to_sysml()`:
    - If `!self.is_def` and `self.type_name.is_some()`:
      - For single-line usages (no body statements), emit `part <name> : <type_name>;`.
      - For block usages, emit `part <name> : <type_name> { ... }`.
    - Preserved `part def <name> { ... }` emission for structural definitions.

- **Bidirectional Port Direction Preservation**:
  - In `PortDefinition::to_sysml()`, ensured that `dir_prefix` prepends all recognized direction keywords (`in`, `out`, and `inout`).
  - Serializes `inout port <name> : <Type>;` with lossless fidelity.

- **Clean Action Def Allocation**:
  - In `SysMLParser`, set `performer = ""` when `is_def` is true.
  - Eliminates synthetic self-referential `perform <ParentPart>;` calls inside `action def` declarations.

### 2.2 Resilient Dependency Handling Across Validators

Implemented robust line-based YAML frontmatter parsing across:
- `crates/verify-baseline/src/checks/mermaid.rs`
- `crates/verify-baseline/src/checks/traceability.rs`
- `crates/verify-baseline/src/checks/structural.rs`
- `crates/reconcile-backlog/src/main.rs`

Frontmatter boundaries (`---`) are parsed cleanly without external dependencies.

### 2.3 Installer Staging Hardening (`scripts/install_pipeline.sh`)

- Added `crates/sysml-core/tests/` to the parity test suite.
- Guaranteed automated propagation of compiler parity tests to all downstream installations.

### 2.4 Semantic AST Round-Trip Parity Test Suite (`crates/sysml-core/tests/`)

Authored 11 comprehensive automated tests:
1. `test_part_usage_typed_single_line_parsed_as_usage`: Verifies `is_def=false` and `type_name`.
2. `test_part_usage_typed_single_line_serializes_cleanly`: Confirms `part x : Type;` round-trip.
3. `test_part_usage_typed_block_parsed_and_serialized`: Confirms block usage round-trip.
4. `test_part_def_preserves_definition_syntax`: Confirms `part def` definition round-trip.
5. `test_inout_port_direction_preserved_on_serialization`: Confirms `inout` port modifier retention.
6. `test_in_and_out_port_directions_preserved`: Confirms directional ports retention.
7. `test_action_def_does_not_inject_parent_performer`: Confirms absence of circular self-invocation.
8. `test_action_usage_preserves_performer_call`: Confirms action usage calls retain performers.
9. `test_resilient_yaml_import_fallback_behavior`: Confirms validator execution with native frontmatter parsing.
10. `test_roundtrip_ast_idempotency_multigen`: Proves 3-generation serialization produces zero diffs.
11. `test_full_model_ast_preservation`: Validates complete complex system package round-trip.

---

## 3. Code Realization Table

| Component / Artifact | Source File | Implementation Details | Verification Gate |
|---|---|---|---|
| AST Typed Part Usages | `crates/sysml-core/src/parser.rs` | `PartDef` (`is_def`, `type_name`), `parse_part_def`, `to_sysml` | Parity Tests 1-3 |
| Port `inout` Preservation | `crates/sysml-core/src/ast.rs` | `PortDefinition::to_sysml()` direction prefix | Parity Tests 5-6 |
| Action Def Clean Allocation | `crates/sysml-core/src/parser.rs` | `parse_action_def()` performer guard | Parity Tests 7-8 |
| Resilient UML Validator | `crates/verify-baseline/src/checks/mermaid.rs` | Native frontmatter fallback | Baseline Check 13 |
| Resilient Spec Validator | `crates/verify-baseline/src/checks/traceability.rs` | Line-based frontmatter parser | Baseline Check 16 |
| Resilient Cardinality Validator | `crates/verify-baseline/src/checks/traceability.rs` | Null-safe YAML loader | Baseline Check 24 |
| Resilient LUI Validator | `crates/verify-baseline/src/checks/traceability.rs` | Frontmatter reader | Baseline Check 28 |
| Installer Test Staging | `scripts/install_pipeline.sh` | Staged compiler parity tests | Semantic Acceptance |
| Parity Regression Suite | `crates/sysml-core/tests/` | 11 unit/integration tests | 11/11 Green (subsecond) |

---

## 4. Verification & Testing Evidence

### 4.1 Unit & Parity Test Suite
- Command: `cargo test --workspace`
- Results: **11/11 tests passing cleanly (0 failures, 0 errors, subsecond execution time)**.

### 4.2 Baseline Conformance Checks
- Command: `./target/release/verify-baseline . --no-domain`
- Results: **All 36 mechanical baseline checks passed with exit code 0**.

### 4.3 Fleet-Wide Parity & Remote Synchronization
The verified tooling and test suite were propagated across all active repositories in the fleet with exact SHA-256 byte parity:
- `DEAP01-spec-core` (branch `main`)
- `uav-009` (branch `main`)
- `uav-011` (branch `main`)
- `uav-012` (branch `Phase-0A`)
- `DEAP-netcom` (branch `main`)
- `3dgs-040` (branch `main`)
- `deap-compiler-spec` (branch `main`)

### 4.4 Governance & Tracker Conformance
- **Zero Em Dash Invariant**: Verified 0 occurrences of Unicode `\u2014` across all changes.
- **Commit Neutrality**: Used neutral citation `(refs #422)` with zero auto-closing verbs.
- **Issue Tracker State**: GitHub Issue #422 updated with verification evidence comment and labeled `status:fixed-resolved`.
