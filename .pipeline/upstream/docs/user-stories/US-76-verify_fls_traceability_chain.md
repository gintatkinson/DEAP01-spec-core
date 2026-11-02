| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #825 |
| **Title** | User Story 76: verify fls traceability chain |
| **Type** | user-story |
| **Parent Epic** | [EPIC-11-BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture](../epics/EPIC-11-BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture.md) |
| **SysML Interaction** | `BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture_SequenceInteraction` |
| **SysML Test Case** | `Test_verify_fls_traceability_chain` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# User Story 76: verify fls traceability chain

## User Story
As a `DualModeFerroceneToolchainGate`, I want to invoke `verify_fls_traceability_chain()` on `ThreeRingAssurancePartitionCompiler`, so that `verify_fls_traceability_chain(records, pinned_fls_catalog)` audits the 5-link normative Ferrocene Language Specification (`[FLS: fls_*]`) traceability chain (`REQ-*` -> `SC-*` -> `RustItem` -> `fls_*` -> `KaniAndCoverage`, `unmapped_fls_item_count == 0`, `trace_chain_depth == 5`, `C_CERT_03`) across Ring-1 docstrings and Ring-2 `// SAFETY: [SC-*] [FLS: fls_*]` blocks via `syn` + `proc-macro2` + `quote` + `prettyplease` and `ra_ap_syntax` with zero hand-rolled scanners.

## Architectural Realization (3-Layer Definition of Done)

### Layer 1: Domain State & Data Model
- Domain state attributes on `ThreeRingAssurancePartitionCompiler` and `FlsParagraphTraceabilitySynthesizer`: `assurance_ring_count: Integer`, `max_ring1_heap_allocations: Integer`, `max_unguarded_ffi_entrypoints: Integer`, `enable_ring1_safety_monitor: Boolean`, `unmapped_fls_item_count: Integer`, `trace_chain_depth: Integer`, `enforce_safety_comment_fls: Boolean`, `Blueprint_ID: String`, `Requirement_ID: String`, `Title: String`, `Status: String`, `Target_Crates: String`, `Tracked_Issues: String`, `Generation_Mode: String`, `Specification_Source: String`, `max_toolchain_version_drift: Real`, `stage_fixed_point_mismatch_files: Real`, `unmapped_fls_item_count: Real`, `uncertified_core_symbol_count: Real`, `dal_a_mcdc_coverage_min_pct: Real`, `assurance_ring_count: Real`, `max_module_exec_sloc: Real`, `min_toolchain_discovery_probes: Real`.
- Action `verify_fls_traceability_chain` invocation message envelope bound to `ThreeRingAssurancePartitionCompiler` and `FlsParagraphTraceabilitySynthesizer` typed domain schemas and full OMG SysML v2.0 + KerML 1.0 CST/AST (`deap_core::sysml_cst` via `logos` + `rowan` + `salsa` + `ariadne`, Contract A, zero 16-construct caps).
- Declared lifecycle state model: `BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture_LifecycleState`, `Idle`, `ProbingCrossPlatformToolchain`, `RejectedFailClosed`, `VerifyingDualToolchainParity`, `SynthesizingThreeRingPartitions`, `EmbeddingFlsTraceability`, `AuditingCoreSubsetAndMcdc`, `VerifyingRing1SafetyMonitor`, `CertifiedPackageReady`.

### Layer 2: Logic & State Management
- Behavioral execution contract: invokes `verify_fls_traceability_chain()` on `ThreeRingAssurancePartitionCompiler` and `FlsParagraphTraceabilitySynthesizer` with atomic precondition verification, inspecting Rust CST docstrings and `// SAFETY:` comments via `ra_ap_syntax` (`ast::{Fn, HasDocComments, HasAttrs}`) under `Invariant_REQ_0611_SC_02` (Contract B).
- State transition management: `Idle -> ProbingCrossPlatformToolchain` (trigger `probeToolchainPath`), `SynthesizingThreeRingPartitions -> EmbeddingFlsTraceability` (trigger `threeRingTopologyIsolated`), and `EmbeddingFlsTraceability -> AuditingCoreSubsetAndMcdc` (trigger `fiveLinkFlsChainComplete`) or `RejectedFailClosed` (trigger `rejectMissingFlsAnchorE1102` with `DEAP-E1102`).
- Formal constraints and requirements enforced: `C_CERT_01 (max_toolchain_version_drift >= 0 and max_toolchain_version_drift <= 0)`, `C_CERT_02 (stage_fixed_point_mismatch_files >= 0 and stage_fixed_point_mismatch_files <= 0)`, `C_CERT_03 (unmapped_fls_item_count >= 0 and unmapped_fls_item_count <= 0)`, `C_CERT_04 (uncertified_core_symbol_count >= 0 and uncertified_core_symbol_count <= 0)`, `C_CERT_05 (dal_a_mcdc_coverage_min_pct >= 100 and dal_a_mcdc_coverage_min_pct <= 100)`, `C_CERT_06 (assurance_ring_count >= 3 and assurance_ring_count <= 3)`, `C_CERT_07 (max_module_exec_sloc >= 1 and max_module_exec_sloc <= 380)`, `C_CERT_08 (min_toolchain_discovery_probes >= 1 and min_toolchain_discovery_probes <= 3)`, `REQ-0611 (REQ_0611_BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C)`.
- Modularity & net-negative ratchet target: bounds implementation inside `< 380` exec SLOC submodules (`crates/compile-sysml/src/ring_partition/`, `crates/verify-baseline/src/ferrocene_gate/`, `crates/deap-core/src/fls_trace/`) while explicitly deleting legacy hand-rolled docstring/FLS line scanners, retiring unpartitioned workspace templates, and enforcing zero Check 34 ratchet violations.

### Layer 3: Presentation & Actuator Interface Binding
- Component port bindings on `ThreeRingAssurancePartitionCompiler` and `FlsParagraphTraceabilitySynthesizer`: `sysml_assurance_ast_in: String (in)`, `partitioned_workspace_out: String (out)`, `partition_diag_out: String (out)`, `lowered_module_graph_in: String (in)`, `fls_annotated_units_out: String (out)`, `fls_diag_out: String (out)`.
- Interaction binding (`BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture_SequenceInteraction`): routes `verify_fls_traceability_chain` invocation from `DualModeFerroceneToolchainGate` to `ThreeRingAssurancePartitionCompiler` and returns `verify_dual_toolchain_parity`.
- Diagnostic fault surface: reports deterministic rejection diagnostics (`DEAP-E1102`) to `DualModeFerroceneToolchainGate` across the declared interface boundary when guard or invariant checks fail.

## UML Sequence Diagram

```mermaid
sequenceDiagram
    autonumber
    actor Operator
    participant DualModeFerroceneToolchainGate
    participant ThreeRingAssurancePartitionCompiler
    Operator->>DualModeFerroceneToolchainGate: verify_fls_traceability_chain()
    DualModeFerroceneToolchainGate->>ThreeRingAssurancePartitionCompiler: verify_fls_traceability_chain()
    ThreeRingAssurancePartitionCompiler-->>DualModeFerroceneToolchainGate: verify_dual_toolchain_parity
    DualModeFerroceneToolchainGate-->>Operator: StatusReport
```

## Formal SysML Test Case & Verification Binding
- **SysML Test Case Def:** `Test_verify_fls_traceability_chain`
- **Subject Part:** `FlsParagraphTraceabilitySynthesizer`
- **Verified Safety Requirement:** `REQ_0611_BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C` (`REQ-0611`)
- **Verification Objective:** "Verify 5-link normative Ferrocene Language Specification (`[FLS: fls_*]`) traceability (`C_CERT_03`, `Invariant_REQ_0611_SC_02`) and fail-closed `DEAP-E1102` rejection on missing or orphan `fls_*` paragraph IDs."

```sysml
test case def Test_verify_fls_traceability_chain {
    subject FlsParagraphTraceabilitySynthesizer;
    verify requirement REQ_0611_BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C;
    objective "Verify 5-link FLS paragraph traceability and fail-closed DEAP-E1102 orphan detection.";
    step verify_fls_traceability_chain;
    step verify_certified_core_and_table_a7_coverage;
}
```

## Acceptance Criteria (BDD)
- [ ] AC-US-76-01: Given `ThreeRingAssurancePartitionCompiler` is in declared state `Idle` with domain state attributes (`assurance_ring_count: Integer`, `max_ring1_heap_allocations: Integer`, `max_unguarded_ffi_entrypoints: Integer`, `enable_ring1_safety_monitor: Boolean`), When `DualModeFerroceneToolchainGate` invokes `verify_fls_traceability_chain` on `ThreeRingAssurancePartitionCompiler`, Then `ThreeRingAssurancePartitionCompiler` validates and binds the request against `ThreeRingAssurancePartitionCompiler` domain state (`assurance_ring_count: Integer`, `max_ring1_heap_allocations: Integer`, `max_unguarded_ffi_entrypoints: Integer`, `enable_ring1_safety_monitor: Boolean`) and transitions to `ProbingCrossPlatformToolchain` and `EmbeddingFlsTraceability`.
- [ ] AC-US-76-02: Given `ThreeRingAssurancePartitionCompiler` active state management during `verify_fls_traceability_chain`, When `ThreeRingAssurancePartitionCompiler` evaluates formal constraint `C_CERT_01 (max_toolchain_version_drift >= 0 and max_toolchain_version_drift <= 0)`, Then `ThreeRingAssurancePartitionCompiler` verifies constraint satisfaction before committing state updates.
- [ ] AC-US-76-03: Given `ThreeRingAssurancePartitionCompiler` active state management during `verify_fls_traceability_chain`, When `ThreeRingAssurancePartitionCompiler` evaluates formal constraint `C_CERT_02 (stage_fixed_point_mismatch_files >= 0 and stage_fixed_point_mismatch_files <= 0)`, Then `ThreeRingAssurancePartitionCompiler` verifies constraint satisfaction before committing state updates.
- [ ] AC-US-76-04: Given `ThreeRingAssurancePartitionCompiler` active state management during `verify_fls_traceability_chain`, When `ThreeRingAssurancePartitionCompiler` evaluates formal constraint `C_CERT_03 (unmapped_fls_item_count >= 0 and unmapped_fls_item_count <= 0)` via `ra_ap_syntax`, Then `ThreeRingAssurancePartitionCompiler` verifies 5-link `[FLS: fls_*]` constraint satisfaction before committing state updates.
- [ ] AC-US-76-05: Given `ThreeRingAssurancePartitionCompiler` active state management during `verify_fls_traceability_chain`, When `ThreeRingAssurancePartitionCompiler` evaluates formal constraint `C_CERT_04 (uncertified_core_symbol_count >= 0 and uncertified_core_symbol_count <= 0)`, Then `ThreeRingAssurancePartitionCompiler` verifies constraint satisfaction before committing state updates.
- [ ] AC-US-76-06: Given `ThreeRingAssurancePartitionCompiler` active state management during `verify_fls_traceability_chain`, When `ThreeRingAssurancePartitionCompiler` evaluates formal constraint `C_CERT_05 (dal_a_mcdc_coverage_min_pct >= 100 and dal_a_mcdc_coverage_min_pct <= 100)`, Then `ThreeRingAssurancePartitionCompiler` verifies constraint satisfaction before committing state updates.
- [ ] AC-US-76-07: Given `ThreeRingAssurancePartitionCompiler` ingress port binding (`sysml_assurance_ast_in: String`), When `ThreeRingAssurancePartitionCompiler` completes `verify_fls_traceability_chain` inside `< 380`-SLOC submodules (`C_CERT_07`), Then `ThreeRingAssurancePartitionCompiler` delivers `verify_dual_toolchain_parity` and `FlsTaggedTraceBundle` across egress port (`partitioned_workspace_out: String`, `partition_diag_out: String`, `fls_annotated_units_out: String`) to `DualModeFerroceneToolchainGate`.
- [ ] AC-US-76-08: Given violated formal constraint `C_CERT_01 (max_toolchain_version_drift >= 0 and max_toolchain_version_drift <= 0)` or `C_CERT_03 (unmapped_fls_item_count >= 0 and unmapped_fls_item_count <= 0)` (missing `fls_*` tag or unknown `fls_nonexistent999` anchor), When `DualModeFerroceneToolchainGate` invokes `verify_fls_traceability_chain` on `ThreeRingAssurancePartitionCompiler`, Then `ThreeRingAssurancePartitionCompiler` rejects the request without partial state mutation, preserves prior verified state, and emits deterministic fault diagnostic `DEAP-E1102` across the interface binding.

## Required Features
- [ ] #757 - [Feature 51: [BP 11 Ferrocene Qualified Toolchain FLS Traceability Structural Coverage and 3 Ring Rust_Flutter DO 178 C DO 330 Certification Architecture] DualModeFerroceneToolchainGate](../features/FEAT-51-DualModeFerroceneToolchainGate.md)
- [ ] #758 - [Feature 52: [BP 11 Ferrocene Qualified Toolchain FLS Traceability Structural Coverage and 3 Ring Rust_Flutter DO 178 C DO 330 Certification Architecture] FlsParagraphTraceabilitySynthesizer](../features/FEAT-52-FlsParagraphTraceabilitySynthesizer.md)
- [ ] #759 - [Feature 53: [BP 11 Ferrocene Qualified Toolchain FLS Traceability Structural Coverage and 3 Ring Rust_Flutter DO 178 C DO 330 Certification Architecture] ThreeRingAssurancePartitionCompiler](../features/FEAT-53-ThreeRingAssurancePartitionCompiler.md)

## Source References
Operational concept definitions and system sequence interaction models:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Authoritative user story flows and interaction lifelines derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
