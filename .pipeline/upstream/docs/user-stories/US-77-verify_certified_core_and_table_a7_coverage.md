| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #827 |
| **Title** | User Story 77: verify certified core and table a7 coverage |
| **Type** | user-story |
| **Parent Epic** | [EPIC-11-BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture](../epics/EPIC-11-BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture.md) |
| **SysML Interaction** | `BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture_SequenceInteraction` |
| **SysML Test Case** | `Test_verify_certified_core_and_table_a7_coverage` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# User Story 77: verify certified core and table a7 coverage

## User Story
As a `DualModeFerroceneToolchainGate`, I want to invoke `verify_certified_core_and_table_a7_coverage()` on `ThreeRingAssurancePartitionCompiler`, so that `verify_certified_core_and_table_a7_coverage(summary, certified_core_prefixes)` audits Ring-1 ELF/MIR `symbol-report` inventories against the certified `core` subset allowlist (`uncertified_core_symbol_count == 0`, `C_CERT_04`, rejecting `std`, `alloc`, and dynamic formatting symbols) and verifies `blanket` / `llvm-profdata` DO-178C Table A-7 `100%` Statement, Decision, and MC/DC coverage (`dal_a_mcdc_coverage_min_pct == 100`, `overcounted_span_count == 0`, `require_sysml_test_case_link == true`, `C_CERT_05`) via typed `serde_json` schemas with zero hand-rolled scanners.

## Architectural Realization (3-Layer Definition of Done)

### Layer 1: Domain State & Data Model
- Domain state attributes on `ThreeRingAssurancePartitionCompiler` and `CertifiedCoreSubsetAndCoverageGate`: `assurance_ring_count: Integer`, `max_ring1_heap_allocations: Integer`, `max_unguarded_ffi_entrypoints: Integer`, `enable_ring1_safety_monitor: Boolean`, `uncertified_core_symbol_count: Integer`, `dal_a_mcdc_coverage_min_pct: Integer`, `overcounted_span_count: Integer`, `require_sysml_test_case_link: Boolean`, `Blueprint_ID: String`, `Requirement_ID: String`, `Title: String`, `Status: String`, `Target_Crates: String`, `Tracked_Issues: String`, `Generation_Mode: String`, `Specification_Source: String`, `max_toolchain_version_drift: Real`, `stage_fixed_point_mismatch_files: Real`, `unmapped_fls_item_count: Real`, `uncertified_core_symbol_count: Real`, `dal_a_mcdc_coverage_min_pct: Real`, `assurance_ring_count: Real`, `max_module_exec_sloc: Real`, `min_toolchain_discovery_probes: Real`.
- Action `verify_certified_core_and_table_a7_coverage` invocation message envelope bound to `ThreeRingAssurancePartitionCompiler` and `CertifiedCoreSubsetAndCoverageGate` typed domain schemas (`serde_json`) and full OMG SysML v2.0 + KerML 1.0 `test case def` / `verification def` CST/AST (`deap_core::sysml_cst` via `logos` + `rowan` + `salsa` + `ariadne`, Contract A, zero 16-construct caps).
- Declared lifecycle state model: `BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture_LifecycleState`, `Idle`, `ProbingCrossPlatformToolchain`, `RejectedFailClosed`, `VerifyingDualToolchainParity`, `SynthesizingThreeRingPartitions`, `EmbeddingFlsTraceability`, `AuditingCoreSubsetAndMcdc`, `VerifyingRing1SafetyMonitor`, `CertifiedPackageReady`.

### Layer 2: Logic & State Management
- Behavioral execution contract: invokes `verify_certified_core_and_table_a7_coverage()` on `ThreeRingAssurancePartitionCompiler` and `CertifiedCoreSubsetAndCoverageGate` with atomic precondition verification, enforcing certified `core` subset compliance (`C_CERT_04`) and DO-178C Table A-7 zero-overcount MC/DC coverage (`C_CERT_05`) under `Invariant_REQ_0611_SC_03` (Contract B).
- State transition management: `Idle -> ProbingCrossPlatformToolchain` (trigger `probeToolchainPath`), `EmbeddingFlsTraceability -> AuditingCoreSubsetAndMcdc` (trigger `fiveLinkFlsChainComplete`), and `AuditingCoreSubsetAndMcdc -> VerifyingRing1SafetyMonitor` (trigger `coreSubsetAndTableA7Verified`) or `RejectedFailClosed` (trigger `rejectUncertifiedSymbolOrCoverageE1103E1104` with `DEAP-E1103` / `DEAP-E1104`).
- Formal constraints and requirements enforced: `C_CERT_01 (max_toolchain_version_drift >= 0 and max_toolchain_version_drift <= 0)`, `C_CERT_02 (stage_fixed_point_mismatch_files >= 0 and stage_fixed_point_mismatch_files <= 0)`, `C_CERT_03 (unmapped_fls_item_count >= 0 and unmapped_fls_item_count <= 0)`, `C_CERT_04 (uncertified_core_symbol_count >= 0 and uncertified_core_symbol_count <= 0)`, `C_CERT_05 (dal_a_mcdc_coverage_min_pct >= 100 and dal_a_mcdc_coverage_min_pct <= 100)`, `C_CERT_06 (assurance_ring_count >= 3 and assurance_ring_count <= 3)`, `C_CERT_07 (max_module_exec_sloc >= 1 and max_module_exec_sloc <= 380)`, `C_CERT_08 (min_toolchain_discovery_probes >= 1 and min_toolchain_discovery_probes <= 3)`, `REQ-0611 (REQ_0611_BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C)`.
- Modularity & net-negative ratchet target: bounds implementation inside `< 380` exec SLOC submodules (`crates/compile-sysml/src/ring_partition/`, `crates/verify-baseline/src/ferrocene_gate/`, `crates/deap-core/src/fls_trace/`) while explicitly deleting legacy hand-rolled symbol-table and coverage line scanners, retiring unpartitioned workspace templates, and enforcing zero Check 34 ratchet violations.

### Layer 3: Presentation & Actuator Interface Binding
- Component port bindings on `ThreeRingAssurancePartitionCompiler` and `CertifiedCoreSubsetAndCoverageGate`: `sysml_assurance_ast_in: String (in)`, `partitioned_workspace_out: String (out)`, `partition_diag_out: String (out)`, `elf_and_profdata_in: String (in)`, `table_a7_evidence_out: String (out)`, `coverage_diag_out: String (out)`.
- Interaction binding (`BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture_SequenceInteraction`): routes `verify_certified_core_and_table_a7_coverage` invocation from `DualModeFerroceneToolchainGate` to `ThreeRingAssurancePartitionCompiler` and returns `verify_dual_toolchain_parity`.
- Diagnostic fault surface: reports deterministic rejection diagnostics (`DEAP-E1103`, `DEAP-E1104`) to `DualModeFerroceneToolchainGate` across the declared interface boundary when guard or invariant checks fail.

## UML Sequence Diagram

```mermaid
sequenceDiagram
    autonumber
    actor Operator
    participant DualModeFerroceneToolchainGate
    participant ThreeRingAssurancePartitionCompiler
    Operator->>DualModeFerroceneToolchainGate: verify_certified_core_and_table_a7_coverage()
    DualModeFerroceneToolchainGate->>ThreeRingAssurancePartitionCompiler: verify_certified_core_and_table_a7_coverage()
    ThreeRingAssurancePartitionCompiler-->>DualModeFerroceneToolchainGate: verify_dual_toolchain_parity
    DualModeFerroceneToolchainGate-->>Operator: StatusReport
```

## Formal SysML Test Case & Verification Binding
- **SysML Test Case Def:** `Test_verify_certified_core_and_table_a7_coverage`
- **Subject Part:** `CertifiedCoreSubsetAndCoverageGate`
- **Verified Safety Requirement:** `REQ_0611_BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C` (`REQ-0611`)
- **Verification Objective:** "Verify Ferrocene `symbol-report` certified `core` subset compliance (`C_CERT_04`, `DEAP-E1103`) and DO-178C Table A-7 zero-overcount `100%` Statement, Decision, and MC/DC structural coverage (`C_CERT_05`, `DEAP-E1104`) under `Invariant_REQ_0611_SC_03`."

```sysml
test case def Test_verify_certified_core_and_table_a7_coverage {
    subject CertifiedCoreSubsetAndCoverageGate;
    verify requirement REQ_0611_BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C;
    objective "Verify certified core symbol-report gate and DO-178C Table A-7 100% MC/DC coverage.";
    step verify_certified_core_and_table_a7_coverage;
    step verify_three_ring_edge_and_monitor_readback;
}
```

## Acceptance Criteria (BDD)
- [ ] AC-US-77-01: Given `ThreeRingAssurancePartitionCompiler` is in declared state `Idle` with domain state attributes (`assurance_ring_count: Integer`, `max_ring1_heap_allocations: Integer`, `max_unguarded_ffi_entrypoints: Integer`, `enable_ring1_safety_monitor: Boolean`), When `DualModeFerroceneToolchainGate` invokes `verify_certified_core_and_table_a7_coverage` on `ThreeRingAssurancePartitionCompiler`, Then `ThreeRingAssurancePartitionCompiler` validates and binds the request against `ThreeRingAssurancePartitionCompiler` domain state (`assurance_ring_count: Integer`, `max_ring1_heap_allocations: Integer`, `max_unguarded_ffi_entrypoints: Integer`, `enable_ring1_safety_monitor: Boolean`) and transitions to `ProbingCrossPlatformToolchain` and `AuditingCoreSubsetAndMcdc`.
- [ ] AC-US-77-02: Given `ThreeRingAssurancePartitionCompiler` active state management during `verify_certified_core_and_table_a7_coverage`, When `ThreeRingAssurancePartitionCompiler` evaluates formal constraint `C_CERT_01 (max_toolchain_version_drift >= 0 and max_toolchain_version_drift <= 0)`, Then `ThreeRingAssurancePartitionCompiler` verifies constraint satisfaction before committing state updates.
- [ ] AC-US-77-03: Given `ThreeRingAssurancePartitionCompiler` active state management during `verify_certified_core_and_table_a7_coverage`, When `ThreeRingAssurancePartitionCompiler` evaluates formal constraint `C_CERT_02 (stage_fixed_point_mismatch_files >= 0 and stage_fixed_point_mismatch_files <= 0)`, Then `ThreeRingAssurancePartitionCompiler` verifies constraint satisfaction before committing state updates.
- [ ] AC-US-77-04: Given `ThreeRingAssurancePartitionCompiler` active state management during `verify_certified_core_and_table_a7_coverage`, When `ThreeRingAssurancePartitionCompiler` evaluates formal constraint `C_CERT_03 (unmapped_fls_item_count >= 0 and unmapped_fls_item_count <= 0)`, Then `ThreeRingAssurancePartitionCompiler` verifies constraint satisfaction before committing state updates.
- [ ] AC-US-77-05: Given `ThreeRingAssurancePartitionCompiler` active state management during `verify_certified_core_and_table_a7_coverage`, When `ThreeRingAssurancePartitionCompiler` evaluates formal constraint `C_CERT_04 (uncertified_core_symbol_count >= 0 and uncertified_core_symbol_count <= 0)`, Then `ThreeRingAssurancePartitionCompiler` verifies zero `std`, `alloc`, or dynamic formatting symbols before committing state updates.
- [ ] AC-US-77-06: Given `ThreeRingAssurancePartitionCompiler` active state management during `verify_certified_core_and_table_a7_coverage`, When `ThreeRingAssurancePartitionCompiler` evaluates formal constraint `C_CERT_05 (dal_a_mcdc_coverage_min_pct >= 100 and dal_a_mcdc_coverage_min_pct <= 100)`, Then `ThreeRingAssurancePartitionCompiler` verifies `100%` Statement, Decision, and MC/DC coverage with `overcounted_span_count == 0` and `TEST-* -> REQ-*` bindings before committing state updates.
- [ ] AC-US-77-07: Given `ThreeRingAssurancePartitionCompiler` ingress port binding (`sysml_assurance_ast_in: String`), When `ThreeRingAssurancePartitionCompiler` completes `verify_certified_core_and_table_a7_coverage` inside `< 380`-SLOC submodules (`C_CERT_07`), Then `ThreeRingAssurancePartitionCompiler` delivers `verify_dual_toolchain_parity` and `Do178cTableA7Receipt` across egress port (`partitioned_workspace_out: String`, `partition_diag_out: String`, `table_a7_evidence_out: String`) to `DualModeFerroceneToolchainGate`.
- [ ] AC-US-77-08: Given violated formal constraint `C_CERT_01 (max_toolchain_version_drift >= 0 and max_toolchain_version_drift <= 0)`, `C_CERT_04 (uncertified_core_symbol_count >= 0 and uncertified_core_symbol_count <= 0)`, or `C_CERT_05 (dal_a_mcdc_coverage_min_pct >= 100 and dal_a_mcdc_coverage_min_pct <= 100)`, When `DualModeFerroceneToolchainGate` invokes `verify_certified_core_and_table_a7_coverage` on `ThreeRingAssurancePartitionCompiler`, Then `ThreeRingAssurancePartitionCompiler` rejects the request without partial state mutation, preserves prior verified state, and emits deterministic fault diagnostic `DEAP-E1103` or `DEAP-E1104` across the interface binding.

## Required Features
- [ ] #848 - [Feature 50: [BP 11 Ferrocene Qualified Toolchain FLS Traceability Structural Coverage and 3 Ring Rust_Flutter DO 178 C DO 330 Certification Architecture] CertifiedCoreSubsetAndCoverageGate](../features/FEAT-50-CertifiedCoreSubsetAndCoverageGate.md)
- [ ] #849 - [Feature 51: [BP 11 Ferrocene Qualified Toolchain FLS Traceability Structural Coverage and 3 Ring Rust_Flutter DO 178 C DO 330 Certification Architecture] DualModeFerroceneToolchainGate](../features/FEAT-51-DualModeFerroceneToolchainGate.md)
- [ ] #851 - [Feature 53: [BP 11 Ferrocene Qualified Toolchain FLS Traceability Structural Coverage and 3 Ring Rust_Flutter DO 178 C DO 330 Certification Architecture] ThreeRingAssurancePartitionCompiler](../features/FEAT-53-ThreeRingAssurancePartitionCompiler.md)

## Source References
Operational concept definitions and system sequence interaction models:
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Authoritative user story flows and interaction lifelines derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148.
