| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #736 |
| **Title** | Epic 11: BP 11 Ferrocene Qualified Toolchain FLS Traceability Structural Coverage and 3 Ring Rust_Flutter DO 178 C DO 330 Certification Architecture |
| **Type** | epic |
| **Package** | BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture |
| **Subsystem** | BP 11 Ferrocene Qualified Toolchain FLS Traceability Structural Coverage and 3 Ring Rust_Flutter DO 178 C DO 330 Certification Architecture |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Epic 11: BP 11 Ferrocene Qualified Toolchain FLS Traceability Structural Coverage and 3 Ring Rust_Flutter DO 178 C DO 330 Certification Architecture

## 1. Context
Subsystem package for `BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture` (**REQ-0611** / `BP-11` / `DEAP-BP-DET-11`, deterministic UUIDv5 `6ba7b810-9dad-11d1-80b4-00c04fd430cb`), establishing the multi-platform Ferrocene safety-qualified Rust toolchain, normative Ferrocene Language Specification (`[FLS: fls_*]`) traceability synthesizer, certified `core` subset `symbol-report` and `blanket` / `llvm-profdata` DO-178C Table A-7 MC/DC structural coverage gate, and 3-Ring DAL-A through DAL-E Rust/Flutter assurance partition architecture governed by RTCA DO-178C, RTCA DO-330 (TQL-1 and TQL-5), ISO 26262 (ASIL-D), and IEC 61508 (SIL-4).

- **Contract A (100% Normative OMG SysML v2.0 + KerML 1.0 Textual Grammar Coverage -- Zero 16-Construct Caps)**: Enforces full Definition and Usage symmetry across the complete OMG SysML v2.0 and KerML 1.0 grammar (`package`, `library package`, `standard library package`, `import` (`all`, `::*`, `::**`, filter `[...]`), `alias`, `doc`, `comment`, `rep`, `dependency`, `metadata def` / `metadata` / `@Metadata` / `#dal` / `#fls` / `#rt_loop` / `#hazard` / `#risk` / `#capability` with backward-compatible `hazard def` / `risk def` / `capability def` parsing, `part def`/`part`, `item def`/`item`, `attribute def`/`attribute`, `enum def`/`enum`, `port def`/`port` (`in`, `out`, `inout`, `~`), `interface def`/`interface`, `connection def`/`connection`, `flow def`/`flow`, `allocation def`/`allocation`, `occurrence def`/`occurrence`, `action def`/`action`, `state def`/`state`, `calc def`/`calc`, `constraint def`/`constraint`, `requirement def`/`requirement`, `concern def`/`concern`, `use case def`/`use case`, `analysis def`/`analysis`, `verification def`/`verification`, `test case def`/`test case`, `interaction def`/`interaction`, `view def`/`view`, `viewpoint def`/`viewpoint`, `rendering def`/`rendering`, `variation`/`variant`, KerML specialization `:`, `:>`, `:>>`, `::>`, `~`, multiplicities, expressions, and `ISQ`/`SI`/`ScalarValues` libraries) with zero artificial 16-construct caps.
- **Contract B (Multi-Platform Ferrocene DO-178C / DO-330 Qualification & 100% Official Rust Compiler Crate Stack -- Zero Hand-Rolled Scanners)**:
  - **Dual-Mode Toolchain Parity & Dynamic Runtime `PATH` Discovery**: Parses `criticalup.toml` and `rust-toolchain.toml` via `toml_edit`, `cargo-platform`, `cargo_metadata`, `cargo-lock`, and `cargo-util-schemas` to enforce zero semantic `rustc` version or target-triple drift (`max_toolchain_version_drift == 0`, **C_CERT_01**, `DEAP-E1101`) and Stage-1 == Stage-2 self-hosted modular codegen SHA-256 fixed-point identity (`stage_fixed_point_mismatch_files == 0`, **C_CERT_02**, `E0705`), while discovering qualified Ferrocene binaries dynamically across macOS, Linux, and Windows via runtime `PATH` probing (`std::env::consts::{DLL_PREFIX, DLL_SUFFIX, EXE_SUFFIX}`, `camino::Utf8Path`, `min_toolchain_discovery_probes` in `1..3`, **C_CERT_08**) with zero hardcoded OS/user/machine paths.
  - **End-to-End `[FLS: fls_*]` Traceability & Official CST/AST Stack**: Uses `logos` + `rowan` + `salsa` + `ariadne` (`deap_core::sysml_cst`, `deap_core::dart_cst`), `ra_ap_syntax` (`ast::{Fn, HasDocComments, HasAttrs}`), `ra_ap_vfs`, `ra_ap_paths`, `ra_ap_line_index`, `ra_ap_text_edit`, and `syn` + `proc-macro2` + `quote` + `prettyplease` to stamp and audit a 5-link normative Ferrocene Language Specification (`[FLS: fls_*]`) traceability chain (`REQ-*` -> `SC-*` -> `RustItem` -> `fls_*` -> `KaniAndCoverage`, `unmapped_fls_item_count == 0`, `trace_chain_depth == 5`, **C_CERT_03**, `DEAP-E1102`) across all Ring-1 items and Ring-2 FFI `// SAFETY: [SC-*] [FLS: fls_*]` blocks.
  - **Certified `core` Subset `symbol-report` & Zero-Overcount `blanket` MC/DC Coverage**: Audits Ring-1 ELF/MIR `symbol-report` inventories against the certified `core` subset allowlist (`uncertified_core_symbol_count == 0`, **C_CERT_04**, `DEAP-E1103`, rejecting `std`, `alloc`, `__rust_alloc`, `alloc::vec::Vec`, and `std::io::stdio::_print` dynamic formatting symbols) and verifies `blanket` / `llvm-profdata` JSON profiles via typed `serde_json` schemas for DO-178C Table A-7 `100%` Statement, Decision, and MC/DC coverage (`dal_a_mcdc_coverage_min_pct == 100`, `overcounted_span_count == 0`, `require_sysml_test_case_link == true`, **C_CERT_05**, `DEAP-E1104`).
  - **3-Ring DAL-A..E Rust/Flutter Assurance Partitioning**: Synthesizes a 3-ring workspace (`assurance_ring_count == 3`, **C_CERT_06**, `DEAP-E1105`) comprising Ring 1 (`safety_kernel/`, DAL-A/B `#![no_std]` + `#![forbid(unsafe_code)]`, `max_ring1_heap_allocations == 0`), Ring 2 (`ffi_guard_bridge/`, DAL-C Miri-verified FFI precondition guards `Gamma_R2`, `max_unguarded_ffi_entrypoints == 0`), and Ring 3 (`app_flutter/`, DAL-D/E untrusted presentation isolated by Ring-1 `SafetyMonitor` two-step challenge-response command interlocks and FIPS 180-4 SHA-256 `TelemetryFrame` digest readback `DEAP-E1106`), with zero hand-rolled string/line/char-by-char scanners.
- **3-Layer Definition of Done (DoD) & Submodule Modularity**: Closes Layer 1 (Domain State & Data Model), Layer 2 (Logic & State Management), and Layer 3 (Presentation & Actuator Interface Binding) within `< 380`-SLOC submodules (`crates/compile-sysml/src/ring_partition/`, `crates/verify-baseline/src/ferrocene_gate/`, `crates/deap-core/src/fls_trace/`, `max_module_exec_sloc <= 380`, **C_CERT_07**) and mandates explicit deletion of legacy hand-rolled line/string toolchain, FLS, symbol-table, and coverage scanners.

## 2. Requirements & Checklist
- [ ] #756 - [Feature 50: [BP 11 Ferrocene Qualified Toolchain FLS Traceability Structural Coverage and 3 Ring Rust_Flutter DO 178 C DO 330 Certification Architecture] CertifiedCoreSubsetAndCoverageGate](../features/FEAT-50-CertifiedCoreSubsetAndCoverageGate.md)
- [ ] #757 - [Feature 51: [BP 11 Ferrocene Qualified Toolchain FLS Traceability Structural Coverage and 3 Ring Rust_Flutter DO 178 C DO 330 Certification Architecture] DualModeFerroceneToolchainGate](../features/FEAT-51-DualModeFerroceneToolchainGate.md)
- [ ] #758 - [Feature 52: [BP 11 Ferrocene Qualified Toolchain FLS Traceability Structural Coverage and 3 Ring Rust_Flutter DO 178 C DO 330 Certification Architecture] FlsParagraphTraceabilitySynthesizer](../features/FEAT-52-FlsParagraphTraceabilitySynthesizer.md)
- [ ] #759 - [Feature 53: [BP 11 Ferrocene Qualified Toolchain FLS Traceability Structural Coverage and 3 Ring Rust_Flutter DO 178 C DO 330 Certification Architecture] ThreeRingAssurancePartitionCompiler](../features/FEAT-53-ThreeRingAssurancePartitionCompiler.md)

### Associated Use Cases & User Stories

#### Associated Use Cases
- [ ] #787 - [Use Case 59: Dual Mode criticalup toml and rust toolchain toml Parity Cross Platform Discovery and DO 330 Fixed Point Qualification](../use-cases/UC-59-Dual_Mode_criticalup_toml_and_rust_toolchain_toml_Parity_Cross_Platform_Discovery_and_DO_330_Fixed_Point_Qualification.md)
- [ ] #788 - [Use Case 60: End to End FLS Normative Paragraph Traceability Emission and Fail Closed Orphan Detection](../use-cases/UC-60-End_to_End_FLS_Normative_Paragraph_Traceability_Emission_and_Fail_Closed_Orphan_Detection.md)
- [ ] #789 - [Use Case 61: Certified core Library Subset Symbol Report Gate Rejecting std alloc and Dynamic Formatting Symbols](../use-cases/UC-61-Certified_core_Library_Subset_Symbol_Report_Gate_Rejecting_std_alloc_and_Dynamic_Formatting_Symbols.md)
- [ ] #790 - [Use Case 62: Zero Overcount Structural and MC DC Coverage Verification Mapped to SysML Verification Cases](../use-cases/UC-62-Zero_Overcount_Structural_and_MC_DC_Coverage_Verification_Mapped_to_SysML_Verification_Cases.md)
- [ ] #791 - [Use Case 63: Schema Driven 3 Ring Workspace Partitioning and Ring 2 Miri Guarded FFI Precondition Enforcement](../use-cases/UC-63-Schema_Driven_3_Ring_Workspace_Partitioning_and_Ring_2_Miri_Guarded_FFI_Precondition_Enforcement.md)
- [ ] #792 - [Use Case 64: Ring 1 Challenge Response Command Interlock and Telemetry Frame Digest Readback Safety Monitor Against Ring 3 Faults](../use-cases/UC-64-Ring_1_Challenge_Response_Command_Interlock_and_TelemetryFrame_Digest_Readback_SafetyMonitor_Against_Ring_3_Faults.md)
- [ ] #793 - [Use Case 65: Cross Platform Execution Submodule Layout Conformance and Twelve Gate Blueprint Verification Gate](../use-cases/UC-65-Cross_Platform_Execution_Submodule_Layout_Conformance_and_Twelve_Gate_Blueprint_Verification_Gate.md)

#### Associated User Stories
- [ ] #822 - [User Story 73: probe criticalup toolchain](../user-stories/US-73-probe_criticalup_toolchain.md)
- [ ] #823 - [User Story 74: verify dual toolchain parity](../user-stories/US-74-verify_dual_toolchain_parity.md)
- [ ] #824 - [User Story 75: synthesize three ring workspace](../user-stories/US-75-synthesize_three_ring_workspace.md)
- [ ] #825 - [User Story 76: verify fls traceability chain](../user-stories/US-76-verify_fls_traceability_chain.md)
- [ ] #826 - [User Story 77: verify certified core and table a7 coverage](../user-stories/US-77-verify_certified_core_and_table_a7_coverage.md)
- [ ] #827 - [User Story 78: verify three ring edge and monitor readback](../user-stories/US-78-verify_three_ring_edge_and_monitor_readback.md)

## 3. Architecture
Classifiers: `CertifiedCoreSubsetAndCoverageGate` (ports: `in elf_and_profdata_in : String`, `out table_a7_evidence_out : String`, `out coverage_diag_out : String`); `DualModeFerroceneToolchainGate` (ports: `in toolchain_manifests_in : String`, `out qualification_dossier_out : String`, `out toolchain_diag_out : String`); `FlsParagraphTraceabilitySynthesizer` (ports: `in lowered_module_graph_in : String`, `out fls_annotated_units_out : String`, `out fls_diag_out : String`); `ThreeRingAssurancePartitionCompiler` (ports: `in sysml_assurance_ast_in : String`, `out partitioned_workspace_out : String`, `out partition_diag_out : String`). Package port definitions: `BP_11_FerroceneAssurancePortDef`. Structural connectors: `BP_11_PartitionToFlsTraceabilityConnection` (`ThreeRingAssurancePartitionCompiler.ThreeRingAssurancePartitionCompiler.partitioned_workspace_out -> FlsParagraphTraceabilitySynthesizer.FlsParagraphTraceabilitySynthesizer.lowered_module_graph_in`).

### Subsystem Capability Allocations & 3-Layer Definition of Done
| Capability Name | Subsystem Package | Realizing Classifier(s) | Description / Objective |
| :--- | :--- | :--- | :--- |
| `DualModeFerroceneToolchainQualification` | `BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture` | `DualModeFerroceneToolchainGate` | Enforces `criticalup.toml` + `rust-toolchain.toml` semantic parity (`C_CERT_01`), Stage-1 == Stage-2 SHA-256 fixed-point codegen (`C_CERT_02`), and dynamic cross-platform runtime `PATH` probing (`C_CERT_08`) across macOS, Linux, and Windows. |
| `FlsNormativeParagraphTraceability` | `BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture` | `FlsParagraphTraceabilitySynthesizer` | Synthesizes and audits the 5-link `[FLS: fls_*]` traceability chain (`REQ-*` -> `SC-*` -> `RustItem` -> `fls_*` -> `KaniAndCoverage`, `C_CERT_03`) across Ring-1 docstrings and Ring-2 `// SAFETY:` comments via `syn`, `quote`, `prettyplease`, and `ra_ap_syntax`. |
| `CertifiedCoreAndTableA7CoverageGate` | `BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture` | `CertifiedCoreSubsetAndCoverageGate` | Verifies Ring-1 ELF/MIR `symbol-report` compliance against the certified `core` subset (`C_CERT_04`) and `blanket` / `llvm-profdata` DO-178C Table A-7 `100%` Statement, Decision, and MC/DC coverage (`C_CERT_05`) with zero span overcounting. |
| `ThreeRingAssurancePartitioning` | `BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture` | `ThreeRingAssurancePartitionCompiler` | Compiles SysML v2 `@Layer` / `#dal` AST models into Ring 1 (`safety_kernel/`), Ring 2 (`ffi_guard_bridge/`), and Ring 3 (`app_flutter/`) partitions (`C_CERT_06`) with Ring-1 `SafetyMonitor` challenge-response interlocks and SHA-256 `TelemetryFrame` readback. |

- **Layer 1 (Domain State & Data Model)**: Typed SysML v2 CST/AST (`deap_core::sysml_cst` via `logos` + `rowan` + `salsa` + `ariadne`), manifest structures (`criticalup.toml`, `rust-toolchain.toml` via `toml_edit` + `cargo-platform`), 5-link `[FLS: fls_*]` trace records, `symbol-report` / `blanket` coverage schemas (`serde_json`), and 3-Ring partition plans.
- **Layer 2 (Logic & State Management)**: Deterministic qualification state machine (`BP_11_..._LifecycleState`), fixed-point codegen verification, 5-link FLS orphan detection, DO-178C Table A-7 MC/DC condition-flip evaluation, and Ring-1 `SafetyMonitor` challenge-response interlock logic bounded inside `< 380`-SLOC submodules (`crates/compile-sysml/src/ring_partition/`, `crates/verify-baseline/src/ferrocene_gate/`, `crates/deap-core/src/fls_trace/`, `C_CERT_07`) with explicit deletion of legacy hand-rolled scanners.
- **Layer 3 (Presentation & Actuator Interface Binding)**: Typed ingress/egress ports (`toolchain_manifests_in`, `qualification_dossier_out`, `toolchain_diag_out`, `sysml_assurance_ast_in`, `partitioned_workspace_out`, `partition_diag_out`, `lowered_module_graph_in`, `fls_annotated_units_out`, `fls_diag_out`, `elf_and_profdata_in`, `table_a7_evidence_out`, `coverage_diag_out`), single-line NDJSON diagnostic emission (`DEAP-E1101..E1106`, `E0705`), and Ring-2 `flutter_rust_bridge` / C-ABI FFI guard bindings.

## 4. Operational Considerations
Lifecycle state machines: `BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture_LifecycleState` (Initial -> Idle; Idle -> ProbingCrossPlatformToolchain on probeToolchainPath; ProbingCrossPlatformToolchain -> RejectedFailClosed on rejectMissingToolchainE1101; ProbingCrossPlatformToolchain -> VerifyingDualToolchainParity on toolchainDiscovered; VerifyingDualToolchainParity -> RejectedFailClosed on rejectToolchainDriftE1101; VerifyingDualToolchainParity -> SynthesizingThreeRingPartitions on toolchainAndStage2ParityLocked; SynthesizingThreeRingPartitions -> RejectedFailClosed on rejectCrossRingCouplingE1105; SynthesizingThreeRingPartitions -> EmbeddingFlsTraceability on threeRingTopologyIsolated; EmbeddingFlsTraceability -> RejectedFailClosed on rejectMissingFlsAnchorE1102; EmbeddingFlsTraceability -> AuditingCoreSubsetAndMcdc on fiveLinkFlsChainComplete; AuditingCoreSubsetAndMcdc -> RejectedFailClosed on rejectUncertifiedSymbolOrCoverageE1103E1104; AuditingCoreSubsetAndMcdc -> VerifyingRing1SafetyMonitor on coreSubsetAndTableA7Verified; VerifyingRing1SafetyMonitor -> RejectedFailClosed on rejectMonitorReadbackFaultE1106; VerifyingRing1SafetyMonitor -> CertifiedPackageReady on certificationEvidenceVerified; RejectedFailClosed -> Terminal; CertifiedPackageReady -> Terminal). Operational actions: `probe_criticalup_toolchain`, `verify_dual_toolchain_parity`, `synthesize_three_ring_workspace`, `verify_fls_traceability_chain`, `verify_certified_core_and_table_a7_coverage`, `verify_three_ring_edge_and_monitor_readback`. Interaction sequences: `BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture_SequenceInteraction`.

## 5. Security & Governance
Allocated requirements: `REQ-0611 (REQ_0611_BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C)`. Enforced invariant constraints: `C_CERT_01 (max_toolchain_version_drift >= 0 and max_toolchain_version_drift <= 0)`, `C_CERT_02 (stage_fixed_point_mismatch_files >= 0 and stage_fixed_point_mismatch_files <= 0)`, `C_CERT_03 (unmapped_fls_item_count >= 0 and unmapped_fls_item_count <= 0)`, `C_CERT_04 (uncertified_core_symbol_count >= 0 and uncertified_core_symbol_count <= 0)`, `C_CERT_05 (dal_a_mcdc_coverage_min_pct >= 100 and dal_a_mcdc_coverage_min_pct <= 100)`, `C_CERT_06 (assurance_ring_count >= 3 and assurance_ring_count <= 3)`, `C_CERT_07 (max_module_exec_sloc >= 1 and max_module_exec_sloc <= 380)`, `C_CERT_08 (min_toolchain_discovery_probes >= 1 and min_toolchain_discovery_probes <= 3)`, `Invariant_REQ_0611_SC_01`, `Invariant_REQ_0611_SC_02`, `Invariant_REQ_0611_SC_03`, `Invariant_REQ_0611_SC_04`, `Invariant_REQ_0611_SC_05`.

## 6. Source References
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 15288:2023 §6.4.3 Architecture Definition Process

Subsystem architectural composition and formal invariants derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 15288.

## System-Level UML Class Diagram

```mermaid
classDiagram
    class CertifiedCoreSubsetAndCoverageGate {
        +String elf_and_profdata_in (in)
        +String table_a7_evidence_out (out)
        +String coverage_diag_out (out)
        +Integer uncertified_core_symbol_count
        +Integer dal_a_mcdc_coverage_min_pct
        +Integer overcounted_span_count
        +Boolean require_sysml_test_case_link
    }
    class DualModeFerroceneToolchainGate {
        +String toolchain_manifests_in (in)
        +String qualification_dossier_out (out)
        +String toolchain_diag_out (out)
        +Integer max_toolchain_version_drift
        +Integer stage_fixed_point_mismatch_files
        +Boolean reproducible_ferrocene_lock
        +Boolean require_formal_toolchains
        +String deterministic_uuid
    }
    class FlsParagraphTraceabilitySynthesizer {
        +String lowered_module_graph_in (in)
        +String fls_annotated_units_out (out)
        +String fls_diag_out (out)
        +Integer unmapped_fls_item_count
        +Integer trace_chain_depth
        +Boolean enforce_safety_comment_fls
    }
    class ThreeRingAssurancePartitionCompiler {
        +String sysml_assurance_ast_in (in)
        +String partitioned_workspace_out (out)
        +String partition_diag_out (out)
        +Integer assurance_ring_count
        +Integer max_ring1_heap_allocations
        +Integer max_unguarded_ffi_entrypoints
        +Boolean enable_ring1_safety_monitor
    }
    CertifiedCoreSubsetAndCoverageGate --> DualModeFerroceneToolchainGate : coordinates
    CertifiedCoreSubsetAndCoverageGate --> FlsParagraphTraceabilitySynthesizer : coordinates
    CertifiedCoreSubsetAndCoverageGate --> ThreeRingAssurancePartitionCompiler : coordinates
```

## System State Machine Diagram

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> ProbingCrossPlatformToolchain : probeToolchainPath
    ProbingCrossPlatformToolchain --> RejectedFailClosed : rejectMissingToolchainE1101
    ProbingCrossPlatformToolchain --> VerifyingDualToolchainParity : toolchainDiscovered
    VerifyingDualToolchainParity --> RejectedFailClosed : rejectToolchainDriftE1101
    VerifyingDualToolchainParity --> SynthesizingThreeRingPartitions : toolchainAndStage2ParityLocked
    SynthesizingThreeRingPartitions --> RejectedFailClosed : rejectCrossRingCouplingE1105
    SynthesizingThreeRingPartitions --> EmbeddingFlsTraceability : threeRingTopologyIsolated
    EmbeddingFlsTraceability --> RejectedFailClosed : rejectMissingFlsAnchorE1102
    EmbeddingFlsTraceability --> AuditingCoreSubsetAndMcdc : fiveLinkFlsChainComplete
    AuditingCoreSubsetAndMcdc --> RejectedFailClosed : rejectUncertifiedSymbolOrCoverageE1103E1104
    AuditingCoreSubsetAndMcdc --> VerifyingRing1SafetyMonitor : coreSubsetAndTableA7Verified
    VerifyingRing1SafetyMonitor --> RejectedFailClosed : rejectMonitorReadbackFaultE1106
    VerifyingRing1SafetyMonitor --> CertifiedPackageReady : certificationEvidenceVerified
    RejectedFailClosed --> [*]
    CertifiedPackageReady --> [*]
```
