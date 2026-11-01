doc /* Translated from Level 0 OEM Markdown specification */
package BP_09_Multi_Stage_Compiler_and_Formal_Verification_Gatekeeper {
    attribute Blueprint_ID : String = "BP-09";
    attribute Requirement_ID : String = "REQ-0609";
    attribute Title : String = "Multi-Stage Compiler and Formal Verification Gatekeeper (Clippy, Dylint MIR, Miri Tree Borrows, Kani CBMC, and flutter_rust_bridge FFI Synthesis)";
    attribute Status : String = "Draft";
    attribute Target_Crates : String = "crates/deap-core, crates/compile-sysml, crates/verify-baseline";
    attribute Tracked_Issues : String = "#742";
    attribute Generation_Mode : String = "subagent";
    attribute Specification_Source : String = "Cargo.toml:12-25";
    doc /* [unit: `lines`] */
    attribute too_many_lines_threshold : Real;
    doc /* [unit: `params`] */
    attribute too_many_arguments_threshold : Real;
    doc /* [unit: `allocs`] */
    attribute max_rt_heap_allocations : Real;
    doc /* [unit: `violations`] */
    attribute max_miri_ub_violations : Real;
    doc /* [unit: `counterexamples`] */
    attribute max_kani_counterexamples : Real;
    doc /* [unit: `iterations`] */
    attribute default_kani_unwind_bound : Real;
    doc /* runStableClippyGate(workspacePath) */
    action def runStableClippyGate;
    doc /* verifyMirAllocationsAndReqs(digestPath) */
    action def verifyMirAllocationsAndReqs;
    doc /* synthesizeFrbIcdFacade(icdSchemaPath) */
    action def synthesizeFrbIcdFacade;
    doc /* runMiriAndDartGate(bridgeCratePath) */
    action def runMiriAndDartGate;
    doc /* synthesizeKaniProofModule(safetyAstIr) */
    action def synthesizeKaniProofModule;
    doc /* runKaniCbmcProofs(targetCratePath) */
    action def runKaniCbmcProofs;
    doc /* [unit: `lines`] */
    assert constraint C_VER_01 {
        too_many_lines_threshold >= 1 and too_many_lines_threshold <= 80;
    }
    doc /* [unit: `params`] */
    assert constraint C_VER_02 {
        too_many_arguments_threshold >= 0 and too_many_arguments_threshold <= 5;
    }
    doc /* [unit: `allocs`] */
    assert constraint C_VER_03 {
        max_rt_heap_allocations >= 0 and max_rt_heap_allocations <= 0;
    }
    doc /* [unit: `violations`] */
    assert constraint C_VER_04 {
        max_miri_ub_violations >= 0 and max_miri_ub_violations <= 0;
    }
    doc /* [unit: `counterexamples`] */
    assert constraint C_VER_05 {
        max_kani_counterexamples >= 0 and max_kani_counterexamples <= 0;
    }
    doc /* [unit: `iterations`] */
    assert constraint C_VER_06 {
        default_kani_unwind_bound >= 1 and default_kani_unwind_bound <= 64;
    }
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0609_BP_09_Multi_Stage_Compiler_and_Formal_Verification_Gatekeeper {
        id = "REQ-0609";
        text = "MultiStageFormalVerificationGatekeeper (**REQ-0609** / BP-09 / DEAP-BP-DET-09, UUIDv5 6ba7b810-9dad-11d1-80b4-00c04fd430c9, Class P static HIR/THIR/MIR linear pass complexity paired with bounded CBMC state-space unwinding, governed by DO-178C Level A, DO-330 TQL-4, DO-333 Formal Methods Supplement,";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_stage_1_universal_stable_clippy_total_function_and_clippy_toml_threshold_enforcement : String = "- **Given**: An upstream or generated downstream Rust workspace configured with [workspace.lints.clippy] enforcing all 14 total-function, arithmetic, cast, unsafe-doc, and complexity denials plus root clippy.toml (too-many-lines-threshold = 80, too-many-arguments-threshold = 5, and disallowed-types banning non-deterministic HashMap / HashSet and backtracking pattern-matching types), alongside a negative fixture containing unchecked + arithmetic, direct slice indexing, an 81-line function, a 6-argument function, or std::collections::HashMap. - **When**: StableClippyTotalFunctionGate executes.";
        attribute ac_02_stage_2_isolated_nightly_dylint_mir_zero_allocation_proof_on_real_time_control_loops : String = "- **Given**: A standalone nightly Dylint MIR lint workspace isolated from root stable Cargo.toml, evaluating two #[rt_loop] control functions: one pure stack-only total function and one negative fixture that transitively invokes Box::new, Vec::push growth, format! heap allocation, or dynamic drop-glue deallocation through an indirect helper function. - **When**: IsolatedDylintMirAllocationGate traverses rustc_middle::mir::Body basic-block call and drop terminators across the #[rt_loop] call graph while stable cargo check --workspace --offline --locked runs independently on the root workspace.";
        attribute ac_03_stage_2_compile_time_mir_requirement_binding_verification_against_active_sysml_digests : String = "- **Given**: Rust items annotated with #[realises(\"REQ-0609\")] or /// Realises: [REQ-0609] verified against an active .digest.json compiled from .sysml, plus a negative fixture referencing a non-existent or retired identifier REQ-9999. - **When**: IsolatedDylintMirAllocationGate cross-checks extracted HIR/MIR DefId requirement bindings against the active SysML requirement digest inventory. - **Then**: Valid REQ-0609 bindings pass cleanly, whereas the dangling REQ-9999 annotation fails closed with non-zero exit status and single-line JSON diagnostic DEAP-E0902.";
        attribute ac_04_stage_3_deterministic_flutter_rust_bridge_ffi_synthesis_with_fail_closed_icd_boundary_guards : String = "- **Given**: A SysML v2 package and ICD_01 / ICD_02 specification declaring port def and item flow signals with numeric [min..max] bounds and admissible state def operational modes, lowered by crates/compile-sysml/src/modular_codegen/. - **When**: compile-sysml synthesizes the #![forbid(unsafe_code)] pure domain crate, the isolated flutter_rust_bridge FFI boundary facade, and the Dart DTO/binding modules, and executes inbound FFI boundary tests alongside dart analyze --fatal-infos. - **Then**: Pure domain modules contain #![forbid(unsafe_code)], synthesized Dart bindings pass dart analyze.";
        attribute ac_05_stage_3_miri_tree_borrows_abstract_machine_undefined_behavior_rejection_on_ffi_and_ring_buffer_bridges : String = "- **Given**: An isolated Stage-3 FFI marshaling and lock-free SPSC ring-buffer bridge module containing documented // SAFETY: blocks, paired with a negative mutation probe introducing a Tree Borrows pointer-aliasing violation or uninitialized byte read. - **When**: FrbIcdBridgeAndMiriUbGate executes cargo +nightly miri test with MIRIFLAGS=\"-Zmiri-tree-borrows\". - **Then**: The compliant FFI/ring-buffer bridge passes Miri with 0 undefined-behavior findings (**C-VER-04**) and 0 leaked allocations, whereas the mutated aliasing/uninitialized probe fails closed with a non-zero exit code and.";
        attribute ac_06_stage_4_deterministic_kani_cbmc_proof_synthesis_and_stpa_unsafe_control_action_falsification : String = "- **Given**: SysML constraint def invariants, port def [min..max] intervals, state def transition tables, and STPA **UCA-01** / **SC-01** safety constraints lowered into #[cfg(kani)] modules via syn + quote + prettyplease across deap-core, compile-sysml, and a generated downstream crate, plus a negative mutation weakening an overflow check or STPA state guard. - **When**: KaniCbmcProofSynthesizer emits the #[kani::proof] + #[kani::unwind(N)] harnesses (kani::any() + kani::assume(...) + assert!(...)) and executes cargo kani. - **Then**: Emitted #[cfg(kani)] source files are byte-deterministic.";
        attribute ac_07_fail_closed_four_stage_orchestration_and_twelve_gate_blueprint_bijective_sysml_persistence : String = "- **Given**: docs/designs/blueprint-09-clippy-dylint-miri-kani-formal-verification-gate.md specifying all 4 stages across deap-core, compile-sysml, and verify-baseline. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-09-clippy-dylint-miri-kani-formal-verification-gate.md and ./target/release/compile-sysml execute over the blueprint and emitted .sysml metamodel. - **Then**: All 12 mechanical blueprint verification gates pass with exit code 0, bijective SysML v2 roundtrip parity holds with zero semantic errors, and.";
        doc /* \begin{aligned} \text{Stage1Pass}(\mathcal{W}) &\iff \forall C \in \mathcal{W}, \quad \forall \ell \in \mathcal{L}_{\text{deny}}, \quad \text{Violations}_{\ell}(C) = \emptyset \land \text{Types}(C) \cap \mathcal{T}_{\text{banned}} = \emptyset \\ \forall f \in \text{Fns}(\mathcal{W}), \quad \text{Lines}_{\text{HIR}}(f) &\le 80 \land \lvert \text{Args}_{\text{HIR}}(f) \rvert \le 5 \land \text{BoolParams}_{\text{HIR}}(f) \le 3 \end{aligned} */
        constraint def Invariant_REQ_0609_SC_01;
        doc /* \begin{aligned} \text{ZeroRtHeap}(G_{\text{MIR}}) &\iff \forall f_{\text{rt}} \in V_{\text{rt\_loop}}, \quad \mathcal{R}^*(f_{\text{rt}}) \cap \mathcal{A}_{\text{heap}} = \emptyset \\ \text{TraceableMIR}(V_{\text{def}}, \mathcal{Q}_{\text{sysml}}) &\iff \forall d \in V_{\text{def}}, \quad \text{Realises}(d) \neq \emptyset \implies \text{Realises}(d) \subseteq \mathcal{Q}_{\text{sysml}} \end{aligned} */
        constraint def Invariant_REQ_0609_SC_02;
        doc /* \begin{aligned} G_p(u, s) &\iff \text{is\_finite}(u) \land (p_{\min} \le u \le p_{\max}) \land (s \in S_p) \\ \text{Facade}_p(u, s) &= \begin{cases} \text{Ok}(\delta_{\text{domain}}(u, s)) & \text{if } G_p(u, s) \\ \text{Err}(\text{DEAP\_E0903}) & \text{if } \neg G_p(u, s) \end{cases} \\ \text{Stage3Pass}(\mathcal{W}_{\text{domain}}, \mathcal{B}_{\text{FFI}}) &\iff \text{UnsafeBlocks}(\mathcal{W}_{\text{domain}}) = 0 \land \forall \sigma \in \text{Traces}_{\text{Miri}}(\mathcal{B}_{\text{FFI}}), \quad \text{UB}_{\text{TreeBorrows}}(\sigma) = 0 \end{aligned} */
        constraint def Invariant_REQ_0609_SC_03;
        doc /* \begin{aligned} \text{VC}_f(x) &\equiv \text{Pre}_{\text{sysml}}(x) \implies \Big( \neg \text{Panic}_N(f, x) \land \neg \text{ArithOverflow}_N(f, x) \land \neg \text{OutOfBounds}_N(f, x) \\ &\land \text{Post}_{\text{sysml}}(x, f(x)) \land \bigwedge_{j=1}^{m} \neg \text{UCA}_j(x, f(x)) \Big) \\ \text{Stage4Pass}(f, N) &\iff \text{SAT}\big(\exists x \in \Sigma_{\text{CBMC}}, \quad \neg \text{VC}_f(x)\big) = \text{UNSAT} \end{aligned} */
        constraint def Invariant_REQ_0609_SC_04;
        require Invariant_REQ_0609_SC_01;
        require Invariant_REQ_0609_SC_02;
        require Invariant_REQ_0609_SC_03;
        require Invariant_REQ_0609_SC_04;
        verify by AC_01_Stage_1_Universal_Stable_Clippy_Total_Function_and_clippy_toml_Threshold_Enforcement;
        verify by AC_02_Stage_2_Isolated_Nightly_Dylint_MIR_Zero_Allocation_Proof_on_Real_Time_Control_Loops;
        verify by AC_03_Stage_2_Compile_Time_MIR_Requirement_Binding_Verification_Against_Active_SysML_Digests;
        verify by AC_04_Stage_3_Deterministic_flutter_rust_bridge_FFI_Synthesis_with_Fail_Closed_ICD_Boundary_Guards;
        verify by AC_05_Stage_3_Miri_Tree_Borrows_Abstract_Machine_Undefined_Behavior_Rejection_on_FFI_and_Ring_Buffer_Bridges;
        verify by AC_06_Stage_4_Deterministic_Kani_CBMC_Proof_Synthesis_and_STPA_Unsafe_Control_Action_Falsification;
        verify by AC_07_Fail_Closed_Four_Stage_Orchestration_and_Twelve_Gate_Blueprint_Bijective_SysML_Persistence;
        satisfy by FrbIcdBridgeAndMiriUbGate;
    }
    doc /* Extracted from Mermaid stateDiagram specification. */
    state def BP_09_Multi_Stage_Compiler_and_Formal_Verification_Gatekeeper_LifecycleState {
        state Idle;
        state RunningStage1Clippy;
        state FailedClosedJsonDiagnostic;
        state RunningStage2DylintMir;
        state RunningStage3FrbAndMiri;
        state RunningStage4KaniCbmc;
        state VerifiedAllFourStages;
        transition Initial_to_Idle first Initial then Idle;
        transition Idle_to_RunningStage1Clippy_on_runAllVerificationStages first Idle accept runAllVerificationStages then RunningStage1Clippy;
        transition RunningStage1Clippy_to_FailedClosedJsonDiagnostic_on_rejectStage1ClippyViolation first RunningStage1Clippy accept rejectStage1ClippyViolation then FailedClosedJsonDiagnostic;
        transition RunningStage1Clippy_to_RunningStage2DylintMir_on_stage1TotalFunctionsVerified first RunningStage1Clippy accept stage1TotalFunctionsVerified then RunningStage2DylintMir;
        transition RunningStage2DylintMir_to_FailedClosedJsonDiagnostic_on_rejectMirHeapAllocOrDanglingReq first RunningStage2DylintMir accept rejectMirHeapAllocOrDanglingReq then FailedClosedJsonDiagnostic;
        transition RunningStage2DylintMir_to_RunningStage3FrbAndMiri_on_stage2ZeroRtAllocVerified first RunningStage2DylintMir accept stage2ZeroRtAllocVerified then RunningStage3FrbAndMiri;
        transition RunningStage3FrbAndMiri_to_FailedClosedJsonDiagnostic_on_rejectMiriTreeBorrowsOrDartFatalInfo first RunningStage3FrbAndMiri accept rejectMiriTreeBorrowsOrDartFatalInfo then FailedClosedJsonDiagnostic;
        transition RunningStage3FrbAndMiri_to_RunningStage4KaniCbmc_on_stage3FrbMiriAndDartVerified first RunningStage3FrbAndMiri accept stage3FrbMiriAndDartVerified then RunningStage4KaniCbmc;
        transition RunningStage4KaniCbmc_to_FailedClosedJsonDiagnostic_on_rejectKaniCbmcCounterexample first RunningStage4KaniCbmc accept rejectKaniCbmcCounterexample then FailedClosedJsonDiagnostic;
        transition RunningStage4KaniCbmc_to_VerifiedAllFourStages_on_stage4AllSafetyProofsUnsat first RunningStage4KaniCbmc accept stage4AllSafetyProofsUnsat then VerifiedAllFourStages;
        transition FailedClosedJsonDiagnostic_to_Terminal first FailedClosedJsonDiagnostic then Terminal;
        transition VerifiedAllFourStages_to_Terminal first VerifiedAllFourStages then Terminal;
    }
    doc /* - **Given**: An upstream or generated downstream Rust workspace configured with [workspace.lints.clippy] enforcing all 14 total-function, arithmetic, cast, unsafe-doc, and complexity denials plus root clippy.toml (too-many-lines-threshold = 80, too-many-arguments-threshold = 5, and disallowed-types banning non-deterministic HashMap / HashSet and backtracking pattern-matching types), alongside a negative fixture containing unchecked + arithmetic, direct slice indexing, an 81-line function, a 6-argument function, or std::collections::HashMap. - **When**: StableClippyTotalFunctionGate executes. */
    use case def UC_AC_01_Stage_1_Universal_Stable_Clippy_Total_Function_and_clippy_toml_Threshold_Enforcement {
        subject FrbIcdBridgeAndMiriUbGate;
        objective "Stage 1 Universal Stable Clippy Total Function and clippy toml Threshold Enforcement";
    }
    doc /* - **Given**: A standalone nightly Dylint MIR lint workspace isolated from root stable Cargo.toml, evaluating two #[rt_loop] control functions: one pure stack-only total function and one negative fixture that transitively invokes Box::new, Vec::push growth, format! heap allocation, or dynamic drop-glue deallocation through an indirect helper function. - **When**: IsolatedDylintMirAllocationGate traverses rustc_middle::mir::Body basic-block call and drop terminators across the #[rt_loop] call graph while stable cargo check --workspace --offline --locked runs independently on the root workspace. */
    use case def UC_AC_02_Stage_2_Isolated_Nightly_Dylint_MIR_Zero_Allocation_Proof_on_Real_Time_Control_Loops {
        subject FrbIcdBridgeAndMiriUbGate;
        objective "Stage 2 Isolated Nightly Dylint MIR Zero Allocation Proof on Real Time Control Loops";
    }
    doc /* - **Given**: Rust items annotated with #[realises(\"REQ-0609\")] or /// Realises: [REQ-0609] verified against an active .digest.json compiled from .sysml, plus a negative fixture referencing a non-existent or retired identifier REQ-9999. - **When**: IsolatedDylintMirAllocationGate cross-checks extracted HIR/MIR DefId requirement bindings against the active SysML requirement digest inventory. - **Then**: Valid REQ-0609 bindings pass cleanly, whereas the dangling REQ-9999 annotation fails closed with non-zero exit status and single-line JSON diagnostic DEAP-E0902. */
    use case def UC_AC_03_Stage_2_Compile_Time_MIR_Requirement_Binding_Verification_Against_Active_SysML_Digests {
        subject FrbIcdBridgeAndMiriUbGate;
        objective "Stage 2 Compile Time MIR Requirement Binding Verification Against Active SysML Digests";
    }
    doc /* - **Given**: A SysML v2 package and ICD_01 / ICD_02 specification declaring port def and item flow signals with numeric [min..max] bounds and admissible state def operational modes, lowered by crates/compile-sysml/src/modular_codegen/. - **When**: compile-sysml synthesizes the #![forbid(unsafe_code)] pure domain crate, the isolated flutter_rust_bridge FFI boundary facade, and the Dart DTO/binding modules, and executes inbound FFI boundary tests alongside dart analyze --fatal-infos. - **Then**: Pure domain modules contain #![forbid(unsafe_code)], synthesized Dart bindings pass dart analyze. */
    use case def UC_AC_04_Stage_3_Deterministic_flutter_rust_bridge_FFI_Synthesis_with_Fail_Closed_ICD_Boundary_Guards {
        subject FrbIcdBridgeAndMiriUbGate;
        objective "Stage 3 Deterministic flutter rust bridge FFI Synthesis with Fail Closed ICD Boundary Guards";
    }
    doc /* - **Given**: An isolated Stage-3 FFI marshaling and lock-free SPSC ring-buffer bridge module containing documented // SAFETY: blocks, paired with a negative mutation probe introducing a Tree Borrows pointer-aliasing violation or uninitialized byte read. - **When**: FrbIcdBridgeAndMiriUbGate executes cargo +nightly miri test with MIRIFLAGS=\"-Zmiri-tree-borrows\". - **Then**: The compliant FFI/ring-buffer bridge passes Miri with 0 undefined-behavior findings (**C-VER-04**) and 0 leaked allocations, whereas the mutated aliasing/uninitialized probe fails closed with a non-zero exit code and. */
    use case def UC_AC_05_Stage_3_Miri_Tree_Borrows_Abstract_Machine_Undefined_Behavior_Rejection_on_FFI_and_Ring_Buffer_Bridges {
        subject FrbIcdBridgeAndMiriUbGate;
        objective "Stage 3 Miri Tree Borrows Abstract Machine Undefined Behavior Rejection on FFI and Ring Buffer Bridges";
    }
    doc /* - **Given**: SysML constraint def invariants, port def [min..max] intervals, state def transition tables, and STPA **UCA-01** / **SC-01** safety constraints lowered into #[cfg(kani)] modules via syn + quote + prettyplease across deap-core, compile-sysml, and a generated downstream crate, plus a negative mutation weakening an overflow check or STPA state guard. - **When**: KaniCbmcProofSynthesizer emits the #[kani::proof] + #[kani::unwind(N)] harnesses (kani::any() + kani::assume(...) + assert!(...)) and executes cargo kani. - **Then**: Emitted #[cfg(kani)] source files are byte-deterministic. */
    use case def UC_AC_06_Stage_4_Deterministic_Kani_CBMC_Proof_Synthesis_and_STPA_Unsafe_Control_Action_Falsification {
        subject FrbIcdBridgeAndMiriUbGate;
        objective "Stage 4 Deterministic Kani CBMC Proof Synthesis and STPA Unsafe Control Action Falsification";
    }
    doc /* - **Given**: docs/designs/blueprint-09-clippy-dylint-miri-kani-formal-verification-gate.md specifying all 4 stages across deap-core, compile-sysml, and verify-baseline. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-09-clippy-dylint-miri-kani-formal-verification-gate.md and ./target/release/compile-sysml execute over the blueprint and emitted .sysml metamodel. - **Then**: All 12 mechanical blueprint verification gates pass with exit code 0, bijective SysML v2 roundtrip parity holds with zero semantic errors, and. */
    use case def UC_AC_07_Fail_Closed_Four_Stage_Orchestration_and_Twelve_Gate_Blueprint_Bijective_SysML_Persistence {
        subject FrbIcdBridgeAndMiriUbGate;
        objective "Fail Closed Four Stage Orchestration and Twelve Gate Blueprint Bijective SysML Persistence";
    }
    doc /* Extracted from Mermaid sequenceDiagram specification. */
    interaction def BP_09_Multi_Stage_Compiler_and_Formal_Verification_Gatekeeper_SequenceInteraction {
        lifeline MultiStageVerificationOrchestrator : MultiStageVerificationOrchestrator;
        lifeline StableClippyTotalFunctionGate : StableClippyTotalFunctionGate;
        lifeline IsolatedDylintMirAllocationGate : IsolatedDylintMirAllocationGate;
        lifeline FrbIcdBridgeAndMiriUbGate : FrbIcdBridgeAndMiriUbGate;
        lifeline KaniCbmcProofSynthesizer : KaniCbmcProofSynthesizer;
        message runStableClippyGate from MultiStageVerificationOrchestrator to StableClippyTotalFunctionGate;
        message reply_StageGateReceipt from StableClippyTotalFunctionGate to MultiStageVerificationOrchestrator;
        message verifyMirAllocationsAndReqs from MultiStageVerificationOrchestrator to IsolatedDylintMirAllocationGate;
        message reply_StageGateReceipt from IsolatedDylintMirAllocationGate to MultiStageVerificationOrchestrator;
        message synthesizeFrbIcdFacade from MultiStageVerificationOrchestrator to FrbIcdBridgeAndMiriUbGate;
        message reply_FrbSynthesisReceipt from FrbIcdBridgeAndMiriUbGate to MultiStageVerificationOrchestrator;
        message runMiriAndDartGate from MultiStageVerificationOrchestrator to FrbIcdBridgeAndMiriUbGate;
        message reply_StageGateReceipt from FrbIcdBridgeAndMiriUbGate to MultiStageVerificationOrchestrator;
        message synthesizeKaniProofModule from MultiStageVerificationOrchestrator to KaniCbmcProofSynthesizer;
        message reply_PrettypleaseKaniRustUnit from KaniCbmcProofSynthesizer to MultiStageVerificationOrchestrator;
        message runKaniCbmcProofs from MultiStageVerificationOrchestrator to KaniCbmcProofSynthesizer;
        message reply_StageGateReceipt from KaniCbmcProofSynthesizer to MultiStageVerificationOrchestrator;
    }
    doc /* Permitted Rust Abstract Machine undefined-behavior findings under `cargo +nightly miri test` with Tree Borrows. [unit: violations] */
    part def FrbIcdBridgeAndMiriUbGate {
        attribute max_miri_ub_violations : Integer = 0;
        doc /* Permitted static analysis errors, warnings, or info findings under `dart analyze --fatal-infos`. [unit: diagnostics] */
        attribute max_dart_analyze_infos : Integer = 0;
        doc /* Enforces `#![forbid(unsafe_code)]` across all pure domain, lowering, and FSM crates outside isolated FFI/HAL bridges. [unit: flag] */
        attribute forbid_unsafe_in_domain : Boolean = true;
        doc /* Active Miri pointer-provenance and aliasing operational semantics flag enforced on FFI and ring-buffer test suites. [unit: flag] */
        attribute miri_aliasing_model : String = "-Zmiri-tree-borrows";
        doc /* Receives validated SysML `port def`, `item flow`, `state def`, and `ICD_01`/`ICD_02` signal bounds in `compile-sysml`. [protocol: SysmlPortAndItemFlowIcd] */
        port in sysml_icd_ast_in : String {
            flow in sysml_icd_ast_in_flow : String;
        }
        doc /* Emits fail-closed guarded `flutter_rust_bridge` Rust facades and strongly-typed Dart binding modules. [protocol: FrbRustAndDartContractTree] */
        port out frb_facade_and_dart_out : String {
            flow out frb_facade_and_dart_out_flow : String;
        }
        doc /* Emits `DEAP-E0903` boundary-guard, Miri Tree Borrows UB, or `dart analyze --fatal-infos` failure diagnostics. [protocol: MiriAndDartVerificationReceipt] */
        port out miri_and_dart_receipt_out : String {
            flow out miri_and_dart_receipt_out_flow : String;
        }
    }
    doc /* Permitted transitive heap allocation or dynamic drop-deallocation terminators reachable from any `#[rt_loop]` function. [unit: allocs] */
    part def IsolatedDylintMirAllocationGate {
        attribute max_rt_heap_allocations : Integer = 0;
        doc /* Permitted dangling `#[realises("REQ-*")]` or doc-contract requirement identifiers absent from compiled `.sysml` digests. [unit: links] */
        attribute max_unverified_req_links : Integer = 0;
        doc /* Isolates `rustc_lint` and `rustc_middle::mir` plugin manifest from stable workspace `Cargo.toml`. [unit: flag] */
        attribute isolated_nightly_workspace : Boolean = true;
        doc /* Pinned nightly `rustc_private` toolchain channel executing the standalone Dylint MIR analysis library. [unit: toolchain] */
        attribute plugin_toolchain_channel : String = "nightly-pinned";
        doc /* Receives lowered `rustc_middle::mir::Body` basic blocks, terminators, and `DefId` attributes alongside `.digest.json`. [protocol: RustcMiddleMirStream] */
        port in mir_body_graph_in : String {
            flow in mir_body_graph_in_flow : String;
        }
        doc /* Emits `DEAP-E0901` real-time heap-allocation violations and `DEAP-E0902` broken requirement-binding diagnostics. [protocol: RustcCompilerMessageNdjson] */
        port out dylint_ndjson_out : String {
            flow out dylint_ndjson_out_flow : String;
        }
    }
    doc /* Permitted reachable CBMC property failures (panics, overflows, div-by-zero, OOB, or STPA `UCA-*` breaches). [unit: counterexamples] */
    part def KaniCbmcProofSynthesizer {
        attribute max_kani_counterexamples : Integer = 0;
        doc /* Deterministic loop unwinding ceiling emitted into `#[kani::unwind(N)]` when AST loops carry finite static bounds. [unit: iterations] */
        attribute default_kani_unwind_bound : Integer = 16;
        doc /* Emits `#[cfg(kani)]` proof modules via `syn` + `quote` + `prettyplease` without raw string-template concatenation. [unit: flag] */
        attribute structured_prettyplease_codegen : Boolean = true;
        doc /* Receives SysML `constraint def`, `port def` `[min..max]` ranges, `state def` transitions, and STPA `UCA-*`/`SC-*` specs. [protocol: SysmlConstraintStateStpaIr] */
        port in sysml_safety_ast_in : String {
            flow in sysml_safety_ast_in_flow : String;
        }
        doc /* Emits deterministic `#[cfg(kani)]` modules containing `kani::any()`, `kani::assume(...)`, and `assert!(...)` harnesses. [protocol: PrettypleaseKaniRustUnit] */
        port out kani_proof_module_out : String {
            flow out kani_proof_module_out_flow : String;
        }
        doc /* Emits CBMC verification status and `DEAP-E0904` counterexample traces in `rustc`-compatible JSON format. [protocol: KaniVerificationReceipt] */
        port out cbmc_proof_receipt_out : String {
            flow out cbmc_proof_receipt_out_flow : String;
        }
    }
    doc /* Ordered verification stage count (`Stage 1 Clippy` -> `Stage 2 Dylint MIR` -> `Stage 3 FRB + Miri` -> `Stage 4 Kani CBMC`). [unit: stages] */
    part def MultiStageVerificationOrchestrator {
        attribute verification_stage_count : Integer = 4;
        doc /* Halts downstream stage promotion whenever any preceding verification stage exits non-zero. [unit: flag] */
        attribute fail_closed_short_circuit : Boolean = true;
        doc /* Receives upstream or downstream target workspace verification commands in `verify-baseline` and `compile-sysml`. [protocol: VerificationGateCommand] */
        port in gate_invocation_in : String {
            flow in gate_invocation_in_flow : String;
        }
        doc /* Emits consolidated single-line NDJSON diagnostics (`DEAP-E0900`..`DEAP-E0905`) and final 4-stage verification receipt. [protocol: RustcCompilerMessageNdjson] */
        port out unified_gate_ndjson_out : String {
            flow out unified_gate_ndjson_out_flow : String;
        }
    }
    doc /* Mandatory count of total-function, cast-safety, arithmetic, unsafe-doc, and structural complexity Clippy lints set to `deny` or `forbid`. [unit: lints] */
    part def StableClippyTotalFunctionGate {
        attribute denied_clippy_lint_count : Integer = 14;
        doc /* Maximum HIR function body line count enforced in root `clippy.toml` (`too-many-lines-threshold`). [unit: lines] */
        attribute too_many_lines_threshold : Integer = 80;
        doc /* Maximum callable parameter count enforced in root `clippy.toml` (`too-many-arguments-threshold`). [unit: params] */
        attribute too_many_arguments_threshold : Integer = 5;
        doc /* Minimum banned non-deterministic hash container and backtracking pattern-engine entries in `clippy.toml`. [unit: types] */
        attribute disallowed_types_count : Integer = 3;
        doc /* Emits single-line `cargo clippy --message-format=json` NDJSON diagnostics into the autonomous remediation loop. [unit: flag] */
        attribute json_diagnostic_stream : Boolean = true;
        doc /* Canonical deterministic blueprint identifier (`DEAP-BP-DET-09`). [unit: uuidv5] */
        attribute deterministic_uuid : String = "6ba7b810-9dad-11d1-80b4-00c04fd430c9";
        doc /* Receives workspace `Cargo.toml`, `clippy.toml`, and target Rust crate source trees. [protocol: CargoAndClippyTomlBundle] */
        port in workspace_manifest_in : String {
            flow in workspace_manifest_in_flow : String;
        }
        doc /* Streams machine-applicable `cargo clippy --message-format=json` lint findings and exit status. [protocol: RustcCompilerMessageNdjson] */
        port out clippy_ndjson_out : String {
            flow out clippy_ndjson_out_flow : String;
        }
    }
}