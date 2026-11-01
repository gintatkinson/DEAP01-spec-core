| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | TBD |
| **Title** | Epic 09: BP 09 Multi Stage Compiler and Formal Verification Gatekeeper |
| **Type** | epic |
| **Package** | BP_09_Multi_Stage_Compiler_and_Formal_Verification_Gatekeeper |
| **Subsystem** | BP 09 Multi Stage Compiler and Formal Verification Gatekeeper |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Epic 09: BP 09 Multi Stage Compiler and Formal Verification Gatekeeper

## 1. Context
Subsystem package for BP_09_Multi_Stage_Compiler_and_Formal_Verification_Gatekeeper

## 2. Requirements & Checklist
- [ ] TBD - [Feature 46: [BP 09 Multi Stage Compiler and Formal Verification Gatekeeper] FrbIcdBridgeAndMiriUbGate](../features/FEAT-46-FrbIcdBridgeAndMiriUbGate.md) (b_p_09_multi_stage_compiler_and_formal_verification_gatekeeper.sysml)
- [ ] TBD - [Feature 47: [BP 09 Multi Stage Compiler and Formal Verification Gatekeeper] IsolatedDylintMirAllocationGate](../features/FEAT-47-IsolatedDylintMirAllocationGate.md) (b_p_09_multi_stage_compiler_and_formal_verification_gatekeeper.sysml)
- [ ] TBD - [Feature 48: [BP 09 Multi Stage Compiler and Formal Verification Gatekeeper] KaniCbmcProofSynthesizer](../features/FEAT-48-KaniCbmcProofSynthesizer.md) (b_p_09_multi_stage_compiler_and_formal_verification_gatekeeper.sysml)
- [ ] TBD - [Feature 49: [BP 09 Multi Stage Compiler and Formal Verification Gatekeeper] MultiStageVerificationOrchestrator](../features/FEAT-49-MultiStageVerificationOrchestrator.md) (b_p_09_multi_stage_compiler_and_formal_verification_gatekeeper.sysml)
- [ ] TBD - [Feature 50: [BP 09 Multi Stage Compiler and Formal Verification Gatekeeper] StableClippyTotalFunctionGate](../features/FEAT-50-StableClippyTotalFunctionGate.md) (b_p_09_multi_stage_compiler_and_formal_verification_gatekeeper.sysml)

### Associated Use Cases & User Stories

#### Associated Use Cases
- [ ] TBD - [Use Case 45: Stage 1 Universal Stable Clippy Total Function and clippy toml Threshold Enforcement](../use-cases/UC-45-Stage_1_Universal_Stable_Clippy_Total_Function_and_clippy_toml_Threshold_Enforcement.md)
- [ ] TBD - [Use Case 46: Stage 2 Isolated Nightly Dylint MIR Zero Allocation Proof on Real Time Control Loops](../use-cases/UC-46-Stage_2_Isolated_Nightly_Dylint_MIR_Zero_Allocation_Proof_on_Real_Time_Control_Loops.md)
- [ ] TBD - [Use Case 47: Stage 2 Compile Time MIR Requirement Binding Verification Against Active SysML Digests](../use-cases/UC-47-Stage_2_Compile_Time_MIR_Requirement_Binding_Verification_Against_Active_SysML_Digests.md)
- [ ] TBD - [Use Case 48: Stage 3 Deterministic flutter rust bridge FFI Synthesis with Fail Closed ICD Boundary Guards](../use-cases/UC-48-Stage_3_Deterministic_flutter_rust_bridge_FFI_Synthesis_with_Fail_Closed_ICD_Boundary_Guards.md)
- [ ] TBD - [Use Case 49: Stage 3 Miri Tree Borrows Abstract Machine Undefined Behavior Rejection on FFI and Ring Buffer Bridges](../use-cases/UC-49-Stage_3_Miri_Tree_Borrows_Abstract_Machine_Undefined_Behavior_Rejection_on_FFI_and_Ring_Buffer_Bridges.md)
- [ ] TBD - [Use Case 50: Stage 4 Deterministic Kani CBMC Proof Synthesis and STPA Unsafe Control Action Falsification](../use-cases/UC-50-Stage_4_Deterministic_Kani_CBMC_Proof_Synthesis_and_STPA_Unsafe_Control_Action_Falsification.md)
- [ ] TBD - [Use Case 51: Fail Closed Four Stage Orchestration and Twelve Gate Blueprint Bijective SysML Persistence](../use-cases/UC-51-Fail_Closed_Four_Stage_Orchestration_and_Twelve_Gate_Blueprint_Bijective_SysML_Persistence.md)

#### Associated User Stories
- [ ] TBD - [User Story 57: run Stable Clippy Gate](../user-stories/US-57-runStableClippyGate.md)
- [ ] TBD - [User Story 58: verify Mir Allocations And Reqs](../user-stories/US-58-verifyMirAllocationsAndReqs.md)
- [ ] TBD - [User Story 59: synthesize Frb Icd Facade](../user-stories/US-59-synthesizeFrbIcdFacade.md)
- [ ] TBD - [User Story 60: run Miri And Dart Gate](../user-stories/US-60-runMiriAndDartGate.md)
- [ ] TBD - [User Story 61: synthesize Kani Proof Module](../user-stories/US-61-synthesizeKaniProofModule.md)
- [ ] TBD - [User Story 62: run Kani Cbmc Proofs](../user-stories/US-62-runKaniCbmcProofs.md)

## 3. Architecture
Classifiers: `FrbIcdBridgeAndMiriUbGate` (ports: `in sysml_icd_ast_in : String`, `out frb_facade_and_dart_out : String`, `out miri_and_dart_receipt_out : String`); `IsolatedDylintMirAllocationGate` (ports: `in mir_body_graph_in : String`, `out dylint_ndjson_out : String`); `KaniCbmcProofSynthesizer` (ports: `in sysml_safety_ast_in : String`, `out kani_proof_module_out : String`, `out cbmc_proof_receipt_out : String`); `MultiStageVerificationOrchestrator` (ports: `in gate_invocation_in : String`, `out unified_gate_ndjson_out : String`); `StableClippyTotalFunctionGate` (ports: `in workspace_manifest_in : String`, `out clippy_ndjson_out : String`).

## 4. Operational Considerations
Lifecycle state machines: `BP_09_Multi_Stage_Compiler_and_Formal_Verification_Gatekeeper_LifecycleState` (Initial -> Idle; Idle -> RunningStage1Clippy on runAllVerificationStages; RunningStage1Clippy -> FailedClosedJsonDiagnostic on rejectStage1ClippyViolation; RunningStage1Clippy -> RunningStage2DylintMir on stage1TotalFunctionsVerified; RunningStage2DylintMir -> FailedClosedJsonDiagnostic on rejectMirHeapAllocOrDanglingReq; RunningStage2DylintMir -> RunningStage3FrbAndMiri on stage2ZeroRtAllocVerified; RunningStage3FrbAndMiri -> FailedClosedJsonDiagnostic on rejectMiriTreeBorrowsOrDartFatalInfo; RunningStage3FrbAndMiri -> RunningStage4KaniCbmc on stage3FrbMiriAndDartVerified; RunningStage4KaniCbmc -> FailedClosedJsonDiagnostic on rejectKaniCbmcCounterexample; RunningStage4KaniCbmc -> VerifiedAllFourStages on stage4AllSafetyProofsUnsat; FailedClosedJsonDiagnostic -> Terminal; VerifiedAllFourStages -> Terminal). Operational actions: `runStableClippyGate`, `verifyMirAllocationsAndReqs`, `synthesizeFrbIcdFacade`, `runMiriAndDartGate`, `synthesizeKaniProofModule`, `runKaniCbmcProofs`. Interaction sequences: `BP_09_Multi_Stage_Compiler_and_Formal_Verification_Gatekeeper_SequenceInteraction`.

## 5. Security & Governance
Allocated requirements: `REQ-0609 (REQ_0609_BP_09_Multi_Stage_Compiler_and_Formal_Verification_Gatekeeper)`. Enforced invariant constraints: `C_VER_01 (too_many_lines_threshold >= 1 and too_many_lines_threshold <= 80)`, `C_VER_02 (too_many_arguments_threshold >= 0 and too_many_arguments_threshold <= 5)`, `C_VER_03 (max_rt_heap_allocations >= 0 and max_rt_heap_allocations <= 0)`, `C_VER_04 (max_miri_ub_violations >= 0 and max_miri_ub_violations <= 0)`, `C_VER_05 (max_kani_counterexamples >= 0 and max_kani_counterexamples <= 0)`, `C_VER_06 (default_kani_unwind_bound >= 1 and default_kani_unwind_bound <= 64)`, `Invariant_REQ_0609_SC_01`, `Invariant_REQ_0609_SC_02`, `Invariant_REQ_0609_SC_03`, `Invariant_REQ_0609_SC_04`.

## 6. Source References
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 15288:2023 §6.4.3 Architecture Definition Process

Subsystem architectural composition and formal invariants derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 15288.

## System-Level UML Class Diagram

```mermaid
classDiagram
    class FrbIcdBridgeAndMiriUbGate {
        +String sysml_icd_ast_in (in)
        +String frb_facade_and_dart_out (out)
        +String miri_and_dart_receipt_out (out)
        +Integer max_miri_ub_violations
        +Integer max_dart_analyze_infos
        +Boolean forbid_unsafe_in_domain
        +String miri_aliasing_model
    }
    class IsolatedDylintMirAllocationGate {
        +String mir_body_graph_in (in)
        +String dylint_ndjson_out (out)
        +Integer max_rt_heap_allocations
        +Integer max_unverified_req_links
        +Boolean isolated_nightly_workspace
        +String plugin_toolchain_channel
    }
    class KaniCbmcProofSynthesizer {
        +String sysml_safety_ast_in (in)
        +String kani_proof_module_out (out)
        +String cbmc_proof_receipt_out (out)
        +Integer max_kani_counterexamples
        +Integer default_kani_unwind_bound
        +Boolean structured_prettyplease_codegen
    }
    class MultiStageVerificationOrchestrator {
        +String gate_invocation_in (in)
        +String unified_gate_ndjson_out (out)
        +Integer verification_stage_count
        +Boolean fail_closed_short_circuit
    }
    class StableClippyTotalFunctionGate {
        +String workspace_manifest_in (in)
        +String clippy_ndjson_out (out)
        +Integer denied_clippy_lint_count
        +Integer too_many_lines_threshold
        +Integer too_many_arguments_threshold
        +Integer disallowed_types_count
        +Boolean json_diagnostic_stream
        +String deterministic_uuid
    }
    FrbIcdBridgeAndMiriUbGate --> IsolatedDylintMirAllocationGate : coordinates
    FrbIcdBridgeAndMiriUbGate --> KaniCbmcProofSynthesizer : coordinates
    FrbIcdBridgeAndMiriUbGate --> MultiStageVerificationOrchestrator : coordinates
    FrbIcdBridgeAndMiriUbGate --> StableClippyTotalFunctionGate : coordinates
```

## System State Machine Diagram

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> RunningStage1Clippy : runAllVerificationStages
    RunningStage1Clippy --> FailedClosedJsonDiagnostic : rejectStage1ClippyViolation
    RunningStage1Clippy --> RunningStage2DylintMir : stage1TotalFunctionsVerified
    RunningStage2DylintMir --> FailedClosedJsonDiagnostic : rejectMirHeapAllocOrDanglingReq
    RunningStage2DylintMir --> RunningStage3FrbAndMiri : stage2ZeroRtAllocVerified
    RunningStage3FrbAndMiri --> FailedClosedJsonDiagnostic : rejectMiriTreeBorrowsOrDartFatalInfo
    RunningStage3FrbAndMiri --> RunningStage4KaniCbmc : stage3FrbMiriAndDartVerified
    RunningStage4KaniCbmc --> FailedClosedJsonDiagnostic : rejectKaniCbmcCounterexample
    RunningStage4KaniCbmc --> VerifiedAllFourStages : stage4AllSafetyProofsUnsat
    FailedClosedJsonDiagnostic --> [*]
    VerifiedAllFourStages --> [*]
```
