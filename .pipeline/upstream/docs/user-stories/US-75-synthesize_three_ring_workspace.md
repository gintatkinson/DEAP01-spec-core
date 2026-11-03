| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #825 |
| **Title** | User Story 75: synthesize three ring workspace |
| **Type** | user-story |
| **Parent Epic** | [EPIC-11-BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture](../epics/EPIC-11-BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture.md) |
| **SysML Interaction** | `BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture_SequenceInteraction` |
| **SysML Test Case** | `Test_synthesize_three_ring_workspace` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# User Story 75: synthesize three ring workspace

## User Story
As a `DualModeFerroceneToolchainGate`, I want to invoke `synthesize_three_ring_workspace()` on `ThreeRingAssurancePartitionCompiler`, so that `synthesize_three_ring_workspace(spec)` partitions the OMG SysML v2.0 + KerML 1.0 assurance AST into a 3-Ring DAL-A..E `Rust_Flutter` workspace (`assurance_ring_count == 3`, `C_CERT_06`: Ring 1 `safety_kernel/` `#![no_std]` + `#![forbid(unsafe_code)]`, Ring 2 `ffi_guard_bridge/` Miri-verified FFI precondition guards, Ring 3 `app_flutter/` untrusted presentation) via `syn` + `proc-macro2` + `quote` + `prettyplease` with zero hand-rolled scanners.

## Architectural Realization (3-Layer Definition of Done)

### Layer 1: Domain State & Data Model
- Domain state attributes on `ThreeRingAssurancePartitionCompiler`: `assurance_ring_count: Integer`, `max_ring1_heap_allocations: Integer`, `max_unguarded_ffi_entrypoints: Integer`, `enable_ring1_safety_monitor: Boolean`, `Blueprint_ID: String`, `Requirement_ID: String`, `Title: String`, `Status: String`, `Target_Crates: String`, `Tracked_Issues: String`, `Generation_Mode: String`, `Specification_Source: String`, `max_toolchain_version_drift: Real`, `stage_fixed_point_mismatch_files: Real`, `unmapped_fls_item_count: Real`, `uncertified_core_symbol_count: Real`, `dal_a_mcdc_coverage_min_pct: Real`, `assurance_ring_count: Real`, `max_module_exec_sloc: Real`, `min_toolchain_discovery_probes: Real`.
- Action `synthesize_three_ring_workspace` invocation message envelope bound to `ThreeRingAssurancePartitionCompiler` typed domain schema, `ThreeRingWorkspacePlan`, and full OMG SysML v2.0 + KerML 1.0 CST/AST (`deap_core::sysml_cst` via `logos` + `rowan` + `salsa` + `ariadne`, Contract A, zero 16-construct caps).
- Declared lifecycle state model: `BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture_LifecycleState`, `Idle`, `ProbingCrossPlatformToolchain`, `RejectedFailClosed`, `VerifyingDualToolchainParity`, `SynthesizingThreeRingPartitions`, `EmbeddingFlsTraceability`, `AuditingCoreSubsetAndMcdc`, `VerifyingRing1SafetyMonitor`, `CertifiedPackageReady`.

### Layer 2: Logic & State Management
- Behavioral execution contract: invokes `synthesize_three_ring_workspace()` on `ThreeRingAssurancePartitionCompiler` with atomic precondition verification, enforcing one-way ring dependency isolation (`safety_kernel` has zero dependencies on `ffi_guard_bridge` or `app_flutter`), `max_ring1_heap_allocations == 0`, and Miri-guarded Ring-2 FFI preconditions (`max_unguarded_ffi_entrypoints == 0`) under `Invariant_REQ_0611_SC_04` (Contract B).
- State transition management: `Idle -> ProbingCrossPlatformToolchain` (trigger `probeToolchainPath`), `VerifyingDualToolchainParity -> SynthesizingThreeRingPartitions` (trigger `toolchainAndStage2ParityLocked`), and `SynthesizingThreeRingPartitions -> EmbeddingFlsTraceability` (trigger `threeRingTopologyIsolated`) or `RejectedFailClosed` (trigger `rejectCrossRingCouplingE1105` with `DEAP-E1105`).
- Formal constraints and requirements enforced: `C_CERT_01 (max_toolchain_version_drift >= 0 and max_toolchain_version_drift <= 0)`, `C_CERT_02 (stage_fixed_point_mismatch_files >= 0 and stage_fixed_point_mismatch_files <= 0)`, `C_CERT_03 (unmapped_fls_item_count >= 0 and unmapped_fls_item_count <= 0)`, `C_CERT_04 (uncertified_core_symbol_count >= 0 and uncertified_core_symbol_count <= 0)`, `C_CERT_05 (dal_a_mcdc_coverage_min_pct >= 100 and dal_a_mcdc_coverage_min_pct <= 100)`, `C_CERT_06 (assurance_ring_count >= 3 and assurance_ring_count <= 3)`, `C_CERT_07 (max_module_exec_sloc >= 1 and max_module_exec_sloc <= 380)`, `C_CERT_08 (min_toolchain_discovery_probes >= 1 and min_toolchain_discovery_probes <= 3)`, `REQ-0611 (REQ_0611_BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C)`.
- Modularity & net-negative ratchet target: bounds implementation inside `< 380` exec SLOC submodules (`crates/compile-sysml/src/ring_partition/`, `crates/verify-baseline/src/ferrocene_gate/`, `crates/deap-core/src/fls_trace/`) while explicitly deleting legacy hand-rolled workspace/FFI string scanners, retiring unpartitioned workspace templates, and enforcing zero Check 34 ratchet violations.

### Layer 3: Presentation & Actuator Interface Binding
- Component port bindings on `ThreeRingAssurancePartitionCompiler`: `sysml_assurance_ast_in: String (in)`, `partitioned_workspace_out: String (out)`, `partition_diag_out: String (out)`.
- Interaction binding (`BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture_SequenceInteraction`): routes `synthesize_three_ring_workspace` invocation from `DualModeFerroceneToolchainGate` to `ThreeRingAssurancePartitionCompiler` and returns `verify_dual_toolchain_parity`.
- Diagnostic fault surface: reports deterministic rejection diagnostics (`DEAP-E1105`) to `DualModeFerroceneToolchainGate` across the declared interface boundary when guard or invariant checks fail.

## UML Sequence Diagram

```mermaid
sequenceDiagram
    autonumber
    actor Operator
    participant DualModeFerroceneToolchainGate
    participant ThreeRingAssurancePartitionCompiler
    Operator->>DualModeFerroceneToolchainGate: synthesize_three_ring_workspace()
    DualModeFerroceneToolchainGate->>ThreeRingAssurancePartitionCompiler: synthesize_three_ring_workspace()
    ThreeRingAssurancePartitionCompiler-->>DualModeFerroceneToolchainGate: verify_dual_toolchain_parity
    DualModeFerroceneToolchainGate-->>Operator: StatusReport
```

## Formal SysML Test Case & Verification Binding
- **SysML Test Case Def:** `Test_synthesize_three_ring_workspace`
- **Subject Part:** `ThreeRingAssurancePartitionCompiler`
- **Verified Safety Requirement:** `REQ_0611_BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C` (`REQ-0611`)
- **Verification Objective:** "Verify schema-driven 3-Ring DAL-A..E workspace partitioning (`safety_kernel/`, `ffi_guard_bridge/`, `app_flutter/`, `C_CERT_06`) and Ring-2 Miri-guarded FFI precondition enforcement (`Invariant_REQ_0611_SC_04`, `DEAP-E1105`)."

```sysml
test case def Test_synthesize_three_ring_workspace {
    subject ThreeRingAssurancePartitionCompiler;
    verify requirement REQ_0611_BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C;
    objective "Verify 3-Ring DAL-A..E Rust/Flutter workspace partitioning and Ring-2 FFI precondition guards.";
    step synthesize_three_ring_workspace;
    step verify_fls_traceability_chain;
}
```

## Acceptance Criteria (BDD)
- [ ] AC-US-75-01: Given `ThreeRingAssurancePartitionCompiler` is in declared state `Idle` with domain state attributes (`assurance_ring_count: Integer`, `max_ring1_heap_allocations: Integer`, `max_unguarded_ffi_entrypoints: Integer`, `enable_ring1_safety_monitor: Boolean`), When `DualModeFerroceneToolchainGate` invokes `synthesize_three_ring_workspace` on `ThreeRingAssurancePartitionCompiler`, Then `ThreeRingAssurancePartitionCompiler` validates and binds the request against `ThreeRingAssurancePartitionCompiler` domain state (`assurance_ring_count: Integer`, `max_ring1_heap_allocations: Integer`, `max_unguarded_ffi_entrypoints: Integer`, `enable_ring1_safety_monitor: Boolean`) and transitions to `ProbingCrossPlatformToolchain` and `SynthesizingThreeRingPartitions`.
- [ ] AC-US-75-02: Given `ThreeRingAssurancePartitionCompiler` active state management during `synthesize_three_ring_workspace`, When `ThreeRingAssurancePartitionCompiler` evaluates formal constraint `C_CERT_01 (max_toolchain_version_drift >= 0 and max_toolchain_version_drift <= 0)`, Then `ThreeRingAssurancePartitionCompiler` verifies constraint satisfaction before committing state updates.
- [ ] AC-US-75-03: Given `ThreeRingAssurancePartitionCompiler` active state management during `synthesize_three_ring_workspace`, When `ThreeRingAssurancePartitionCompiler` evaluates formal constraint `C_CERT_02 (stage_fixed_point_mismatch_files >= 0 and stage_fixed_point_mismatch_files <= 0)`, Then `ThreeRingAssurancePartitionCompiler` verifies constraint satisfaction before committing state updates.
- [ ] AC-US-75-04: Given `ThreeRingAssurancePartitionCompiler` active state management during `synthesize_three_ring_workspace`, When `ThreeRingAssurancePartitionCompiler` evaluates formal constraint `C_CERT_03 (unmapped_fls_item_count >= 0 and unmapped_fls_item_count <= 0)`, Then `ThreeRingAssurancePartitionCompiler` verifies constraint satisfaction before committing state updates.
- [ ] AC-US-75-05: Given `ThreeRingAssurancePartitionCompiler` active state management during `synthesize_three_ring_workspace`, When `ThreeRingAssurancePartitionCompiler` evaluates formal constraint `C_CERT_04 (uncertified_core_symbol_count >= 0 and uncertified_core_symbol_count <= 0)`, Then `ThreeRingAssurancePartitionCompiler` verifies constraint satisfaction before committing state updates.
- [ ] AC-US-75-06: Given `ThreeRingAssurancePartitionCompiler` active state management during `synthesize_three_ring_workspace`, When `ThreeRingAssurancePartitionCompiler` evaluates formal constraint `C_CERT_05 (dal_a_mcdc_coverage_min_pct >= 100 and dal_a_mcdc_coverage_min_pct <= 100)` and `C_CERT_06 (assurance_ring_count >= 3 and assurance_ring_count <= 3)`, Then `ThreeRingAssurancePartitionCompiler` verifies constraint satisfaction before committing state updates.
- [ ] AC-US-75-07: Given `ThreeRingAssurancePartitionCompiler` ingress port binding (`sysml_assurance_ast_in: String`), When `ThreeRingAssurancePartitionCompiler` completes `synthesize_three_ring_workspace` inside `< 380`-SLOC submodules (`C_CERT_07`), Then `ThreeRingAssurancePartitionCompiler` delivers `verify_dual_toolchain_parity` and `ThreeRingWorkspacePlan` across egress port (`partitioned_workspace_out: String`, `partition_diag_out: String`) to `DualModeFerroceneToolchainGate` and `FlsParagraphTraceabilitySynthesizer`.
- [ ] AC-US-75-08: Given violated formal constraint `C_CERT_01 (max_toolchain_version_drift >= 0 and max_toolchain_version_drift <= 0)`, `C_CERT_06 (assurance_ring_count >= 3 and assurance_ring_count <= 3)`, or an upward cross-ring dependency (`safety_kernel -> ffi_guard_bridge` or `app_flutter`), When `DualModeFerroceneToolchainGate` invokes `synthesize_three_ring_workspace` on `ThreeRingAssurancePartitionCompiler`, Then `ThreeRingAssurancePartitionCompiler` rejects the request without partial state mutation, preserves prior verified state, and emits deterministic fault diagnostic `DEAP-E1105` across the interface binding.

## Required Features
- [ ] #849 - [Feature 51: [BP 11 Ferrocene Qualified Toolchain FLS Traceability Structural Coverage and 3 Ring Rust_Flutter DO 178 C DO 330 Certification Architecture] DualModeFerroceneToolchainGate](../features/FEAT-51-DualModeFerroceneToolchainGate.md)
- [ ] #851 - [Feature 53: [BP 11 Ferrocene Qualified Toolchain FLS Traceability Structural Coverage and 3 Ring Rust_Flutter DO 178 C DO 330 Certification Architecture] ThreeRingAssurancePartitionCompiler](../features/FEAT-53-ThreeRingAssurancePartitionCompiler.md)

## Source References
Operational concept definitions and system sequence interaction models:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Authoritative user story flows and interaction lifelines derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
