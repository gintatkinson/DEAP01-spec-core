doc /* Translated from Level 0 OEM Markdown specification */
package BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine {
    attribute Blueprint_ID : String = "BP-10";
    attribute Requirement_ID : String = "REQ-0610";
    attribute Title : String = "Incremental Lossless SysML v2 and Polyglot Compiler Engine (logos + rowan + salsa + ariadne)";
    attribute Status : String = "Draft";
    attribute Target_Crates : String = "crates/deap-core, crates/compile-sysml, crates/ingest-sysml, crates/verify-baseline";
    attribute Tracked_Issues : String = "#742";
    attribute Generation_Mode : String = "subagent";
    attribute Specification_Source : String = "crates/compile-sysml/src/lexer/scanner.rs:1-60 / crates/deap-core/src/sysml_ast/mod.rs:1-80 / #742";
    doc /* `SysmlToken`, `SysmlSyntaxKind`, `SysmlLanguage`, `CstParseReceipt`, `HirPackageSummary`, `HirSourceMap`, `SourceInput`, `PolyglotFormat`, `PolyglotBackendKind`, and `AriadneSpanReport` across `deap_core::sysml_cst`, `deap_core::dart_cst`, `compile_sysml::salsa_db`, and `ingest_sysml::polyglot_cst`. */
    attribute Layer_1_Domain_State_Data_Model : String = "Lossless Red/Green CSTs (`rowan` pinned to `ra_ap_syntax`), trivia-erased HIR records, `HirSourceMap` byte-span tables, and cross-platform `camino::Utf8PathBuf` inputs";
    doc /* `normalize_line_endings`, `lex_sysml_tokens`, `parse_lossless_sysml_cst`, `verify_lossless_cst_invariants`, `evaluate_hir_early_cutoff`, `apply_formatting_preserving_splices`, and `resolve_polyglot_backend` with 100% typed CST traversal and zero `.syntax().text().contains(...)`. */
    attribute Layer_2_Logic_State_Management : String = "Pure `logos` DFA lexing, `rowan` GreenNode building (`MAX_AST_DEPTH <= 64`), 4-layer `salsa` memoized early-cutoff queries, disjoint CST splicing, and typed polyglot CST lowering";
    doc /* `render_ariadne_diagnostic` plus `compile-sysml`, `ingest-sysml`, and `verify-baseline check-blueprint` CLI actuators emitting ANSI-free deterministic `ariadne` / NDJSON diagnostics (`DEAP-E1000..DEAP-E1005`) and bijective `.sysml` + `.digest.json` artifacts across macOS, Linux, and Windows. */
    attribute Layer_3_Presentation_Actuator_Interface_Binding : String = "Deterministic `ariadne` byte-span diagnostic renderer, `--message-format=json` NDJSON CLI stream, and atomic `.sysml` / `.digest.json` workspace actuators";
    doc /* [unit: `levels`] */
    attribute max_ast_depth : Real;
    doc /* [unit: `bytes`] */
    attribute untouched_trivia_mutation_bytes : Real;
    doc /* [unit: `files`] */
    attribute stage_fixed_point_mismatch_files : Real;
    doc /* [unit: `layers`] */
    attribute query_layer_count : Real;
    doc /* [unit: `queries`] */
    attribute trivia_edit_downstream_recomputations : Real;
    doc /* [unit: `frontends`] */
    attribute polyglot_frontend_count : Real;
    doc /* [unit: `crates`] */
    attribute rustc_private_feature_count : Real;
    doc /* resolve_polyglot_backend(format) */
    action def resolve_polyglot_backend;
    doc /* normalize_line_endings(raw) */
    action def normalize_line_endings;
    doc /* parse_lossless_sysml_cst(input) */
    action def parse_lossless_sysml_cst;
    doc /* verify_lossless_cst_invariants(source, parsed) */
    action def verify_lossless_cst_invariants;
    doc /* evaluate_hir_early_cutoff(previous_hir, current_hir) */
    action def evaluate_hir_early_cutoff;
    doc /* apply_formatting_preserving_splices(file_id, source, patches) */
    action def apply_formatting_preserving_splices;
    doc /* render_ariadne_diagnostic(input, diag) */
    action def render_ariadne_diagnostic;
    doc /* [unit: `levels`] */
    assert constraint C_INC_01 {
        max_ast_depth >= 1 and max_ast_depth <= 64;
    }
    doc /* [unit: `bytes`] */
    assert constraint C_INC_02 {
        untouched_trivia_mutation_bytes >= 0 and untouched_trivia_mutation_bytes <= 0;
    }
    doc /* [unit: `files`] */
    assert constraint C_INC_03 {
        stage_fixed_point_mismatch_files >= 0 and stage_fixed_point_mismatch_files <= 0;
    }
    doc /* [unit: `layers`] */
    assert constraint C_INC_04 {
        query_layer_count >= 4 and query_layer_count <= 4;
    }
    doc /* [unit: `queries`] */
    assert constraint C_INC_05 {
        trivia_edit_downstream_recomputations >= 0 and trivia_edit_downstream_recomputations <= 0;
    }
    doc /* [unit: `frontends`] */
    assert constraint C_INC_06 {
        polyglot_frontend_count >= 6 and polyglot_frontend_count <= 6;
    }
    doc /* [unit: `crates`] */
    assert constraint C_INC_07 {
        rustc_private_feature_count >= 0 and rustc_private_feature_count <= 0;
    }
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0610_BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine {
        id = "REQ-0610";
        text = "IncrementalLosslessSysmlAndPolyglotCompilerEngine (**REQ-0610** / BP-10 / DEAP-BP-DET-10, UUIDv5 6ba7b810-9dad-11d1-80b4-00c04fd430ca, Class P linear DFA lexing, lossless Red/Green syntax-tree construction, ariadne deterministic byte-span rendering, and O(delta) early-cutoff incremental query";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_lossless_logos_and_rowan_concrete_syntax_tree_full_fidelity_roundtrip_and_cross_platform_crlf_normalization : String = "- **Given**: A .sysml fixture encoded with both LF and CRLF line endings containing nested package, part def, port def, action def, state def, transition, requirement def, and constraint def blocks interleaved with inline // comments, multi-line / * ... * / architect notes, doc / * ... * / specifications, @Safety and @Layer stereotypes, and custom vertical whitespace alignment. - **When**: deap_core::sysml_cst normalizes CRLF to LF via normalize_line_endings, lexes the buffer via logos, and constructs the SysmlLanguage Red/Green CST via rowan::GreenNodeBuilder (rowan pinned identically to.";
        attribute ac_02_resilient_error_recovery_parsing_max_ast_depth_guard_and_deterministic_ariadne_byte_span_diagnostics : String = "- **Given**: Two negative .sysml fixtures: Fixture A containing an unclosed part def attribute missing ; followed by a valid requirement def block, and Fixture B containing 65 nested package / part blocks exceeding MAX_AST_DEPTH = 64 (**C-INC-01**). - **When**: parse_lossless_sysml_cst and render_ariadne_diagnostic process Fixture A and Fixture B. - **Then**: Neither parse panics or overflows the call stack; Fixture A wraps the malformed attribute in SysmlSyntaxKind::ErrorNode, recovers cleanly to parse the subsequent requirement def, and emits both an ANSI-free ariadne byte-span report and a.";
        attribute ac_03_formatting_preserving_bidirectional_cst_splicing_across_forward_and_reverse_sync : String = "- **Given**: An existing .sysml schema containing human-authored architectural comments, custom indentation, and three part def subsystems alongside synchronized docs/features/ and docs/epics/ Markdown specifications, where only one part def port signature is updated during reverse sync, plus a negative splice request containing overlapping Utf8TextRange intervals. - **When**: apply_formatting_preserving_splices executes forward/reverse splice plans and evaluates the negative overlapping splice plan. - **Then**: Only the modified declaration's CST span is spliced while 100% of untouched.";
        attribute ac_04_salsa_early_cutoff_durability_preventing_downstream_recomputations_on_trivia_edits : String = "- **Given**: A multi-package .sysml workspace loaded into compile_sysml::salsa_db across all 4 query layers (SourceDatabase, CstDatabase, HirDatabase, SafetyVerificationDatabase) with query execution counters attached to CST, HIR, semantic validation, STPA/FMECA, and Kani queries. - **When**: Revision 2 appends comments and reformats whitespace inside one .sysml file without altering semantic declarations, followed by Revision 3 modifying a port def [min..max] bound inside that file. - **Then**: On Revision 2, evaluate_hir_early_cutoff returns EarlyCutoffDecision::CutoffPreservedDownstream.";
        attribute ac_05_fail_closed_salsa_cycle_detection_and_net_negative_legacy_scanner_retirement : String = "- **Given**: A negative multi-file .sysml fixture introducing a cyclic import / cyclic part def :> BasePart specialization loop across two packages, alongside workspace SLOC and module-size audits. - **When**: compile_sysml::salsa_db evaluates scoped symbol resolution and verify-baseline audits legacy scanner deletion targets. - **Then**: Cycle recovery intercepts the cycle deterministically without infinite recursion or thread deadlock and fails closed with DEAP-E1004 (ariadne + NDJSON), while legacy hand-rolled scanners in ./crates/deap-core/src/sysml_ast/mod.rs (-2,400+ SLOC),.";
        attribute ac_06_cohesive_typed_polyglot_cst_ast_lowering_with_zero_rustc_private_and_exact_byte_span_diagnostics : String = "- **Given**: Six valid polyglot specification fixtures (Rust .rs via ra_ap_syntax, Dart .dart via deap_core::dart_cst, OpenAPI/JSON-Schema .json via typed serde_json AST, OMG IDL .idl, IETF YANG .yang, and Markdown .md via logos+rowan CSTs in ingest_sysml::polyglot_cst) alongside malformed counterparts at known byte offsets. - **When**: ingest_sysml::polyglot_cst resolves each backend via resolve_polyglot_backend and lowers each fixture into compile_sysml::salsa_db. - **Then**: All 6 valid fixtures lower deterministically into canonical HirPackageSummary and HirSourceMap records.";
        attribute ac_07_twelve_gate_blueprint_verification_and_baseline_cleanliness_gate_conformance : String = "- **Given**: ./docs/designs/blueprint-10-rowan-salsa-incremental-sysml-compiler.md residing in the upstream compiler repository alongside workspace governance rules. - **When**: ./target/release/verify-baseline check-blueprint ./docs/designs/blueprint-10-rowan-salsa-incremental-sysml-compiler.md and ./target/release/verify-baseline . --no-domain are executed. - **Then**: All 12 mechanical blueprint verification gates and all upstream workspace baseline checks pass with exit code 0, atomically persisting ./docs/designs/blueprint-10-rowan-salsa-incremental-sysml-compiler.sysml and.";
        doc /* \begin{aligned} \forall S \in \Sigma^*, \quad \bigoplus_{i=1}^{k} \text{bytes}(t_i) &= S \land \sum_{i=1}^{k} \lvert \text{range}(t_i) \rvert = \lvert S \rvert \\ \text{depth}(T_{\text{green}}(S)) &\le 64 \land \forall e \in \mathcal{D}_{\text{err}}(S), \quad \exists v \in V(T_{\text{green}}(S)) : \text{kind}(v) = \text{ErrorNode} \end{aligned} */
        constraint def Invariant_REQ_0610_SC_01;
        doc /* \begin{aligned} \forall i \in \{1, \dots, m - 1\}, \quad 0 \le a_i \le b_i \le a_{i+1} \le \lvert S \rvert &\land \Delta_{\text{trivia}}(\mathcal{U}(S, \mathcal{E}), S') = 0 \\ \text{Lower}_{\text{HIR}}(\text{Parse}_{\text{CST}}(\text{Emit}_{\text{canonical}}(H))) = H &\land \text{SHA-256}(\mathcal{T}_{\text{Stage 1}}) = \text{SHA-256}(\mathcal{T}_{\text{Stage 2}}) \end{aligned} */
        constraint def Invariant_REQ_0610_SC_02;
        doc /* \begin{aligned} H_{f^*}^{(r+1)} = H_{f^*}^{(r)} &\implies \lvert \text{Recomputed}(Q_4, r \to r+1) \rvert = 0 \\ T_{\text{inc}}(r \to r+1) &= O\left( \sum_{f \in \Delta_{\text{src}}} \lvert S_f \rvert + \sum_{q \in \text{Affected}(Q_3 \cup Q_4, \Delta_{\text{HIR}})} \text{cost}(q) \right) \end{aligned} */
        constraint def Invariant_REQ_0610_SC_03_Incremental_Complexity;
        doc /* \begin{aligned} \forall p \in \mathcal{P}, \forall S \in \Sigma^*, \quad \text{SHA-256}(\Phi_p(S)) &= \text{const}(p, S) \land \forall d \in \mathcal{D}_{\text{span}}, \quad \text{count}(\text{encode}(d), \text{0x1B}) = 0 \end{aligned} */
        constraint def Invariant_REQ_0610_SC_04;
        require Invariant_REQ_0610_SC_01;
        require Invariant_REQ_0610_SC_02;
        require Invariant_REQ_0610_SC_03_Incremental_Complexity;
        require Invariant_REQ_0610_SC_04;
        derived from REQ_0603;
        derived from REQ_0607;
        derived from REQ_0608;
        derived from REQ_0609;
        verify by AC_01_Lossless_logos_and_rowan_Concrete_Syntax_Tree_Full_Fidelity_Roundtrip_and_Cross_Platform_CRLF_Normalization;
        verify by AC_02_Resilient_Error_Recovery_Parsing_MAX_AST_DEPTH_Guard_and_Deterministic_Ariadne_Byte_Span_Diagnostics;
        verify by AC_03_Formatting_Preserving_Bidirectional_CST_Splicing_Across_Forward_and_Reverse_Sync;
        verify by AC_04_Salsa_Early_Cutoff_Durability_Preventing_Downstream_Recomputations_on_Trivia_Edits;
        verify by AC_05_Fail_Closed_Salsa_Cycle_Detection_and_Net_Negative_Legacy_Scanner_Retirement;
        verify by AC_06_Cohesive_Typed_Polyglot_CST_AST_Lowering_with_Zero_rustc_private_and_Exact_Byte_Span_Diagnostics;
        verify by AC_07_Twelve_Gate_Blueprint_Verification_and_Baseline_Cleanliness_Gate_Conformance;
        satisfy by FormattingPreservingSyncSplicer;
    }
    doc /* Extracted from Mermaid stateDiagram specification. */
    state def BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine_LifecycleState {
        state Idle;
        state NormalizingSourceRevision;
        state LexingAndBuildingRowanCst;
        state EmittingAriadneAndJsonDiagnostics;
        state LoweringTriviaErasedHir;
        state EarlyCutoffShortCircuited;
        state EvaluatingSafetyAndKaniQueries;
        state SplicingFormattingPreservingSync;
        state VerifiedIncrementalClean;
        transition Initial_to_Idle first Initial then Idle;
        transition Idle_to_NormalizingSourceRevision_on_normalize_line_endings first Idle accept normalize_line_endings then NormalizingSourceRevision;
        transition NormalizingSourceRevision_to_LexingAndBuildingRowanCst_on_parse_lossless_sysml_cst first NormalizingSourceRevision accept parse_lossless_sysml_cst then LexingAndBuildingRowanCst;
        transition LexingAndBuildingRowanCst_to_EmittingAriadneAndJsonDiagnostics_on_rejectMaxAstDepthOverflowE1002 first LexingAndBuildingRowanCst accept rejectMaxAstDepthOverflowE1002 then EmittingAriadneAndJsonDiagnostics;
        transition LexingAndBuildingRowanCst_to_LoweringTriviaErasedHir_on_verify_lossless_cst_invariants first LexingAndBuildingRowanCst accept verify_lossless_cst_invariants then LoweringTriviaErasedHir;
        transition LoweringTriviaErasedHir_to_EmittingAriadneAndJsonDiagnostics_on_rejectCyclicSalsaQueryE1004 first LoweringTriviaErasedHir accept rejectCyclicSalsaQueryE1004 then EmittingAriadneAndJsonDiagnostics;
        transition LoweringTriviaErasedHir_to_EarlyCutoffShortCircuited_on_cutoffPreservedDownstream first LoweringTriviaErasedHir accept cutoffPreservedDownstream then EarlyCutoffShortCircuited;
        transition LoweringTriviaErasedHir_to_EvaluatingSafetyAndKaniQueries_on_recomputeDownstreamDependents first LoweringTriviaErasedHir accept recomputeDownstreamDependents then EvaluatingSafetyAndKaniQueries;
        transition EvaluatingSafetyAndKaniQueries_to_SplicingFormattingPreservingSync_on_apply_formatting_preserving_splices first EvaluatingSafetyAndKaniQueries accept apply_formatting_preserving_splices then SplicingFormattingPreservingSync;
        transition SplicingFormattingPreservingSync_to_EmittingAriadneAndJsonDiagnostics_on_rejectOverlappingSpliceE1003 first SplicingFormattingPreservingSync accept rejectOverlappingSpliceE1003 then EmittingAriadneAndJsonDiagnostics;
        transition SplicingFormattingPreservingSync_to_VerifiedIncrementalClean_on_bijectiveAndStage2FixedPointVerified first SplicingFormattingPreservingSync accept bijectiveAndStage2FixedPointVerified then VerifiedIncrementalClean;
        transition EarlyCutoffShortCircuited_to_VerifiedIncrementalClean_on_reuseMemoizedSafetyVerificationReceipt first EarlyCutoffShortCircuited accept reuseMemoizedSafetyVerificationReceipt then VerifiedIncrementalClean;
        transition EmittingAriadneAndJsonDiagnostics_to_Terminal first EmittingAriadneAndJsonDiagnostics then Terminal;
        transition VerifiedIncrementalClean_to_Terminal first VerifiedIncrementalClean then Terminal;
    }
    doc /* - **Given**: A .sysml fixture encoded with both LF and CRLF line endings containing nested package, part def, port def, action def, state def, transition, requirement def, and constraint def blocks interleaved with inline // comments, multi-line / * ... * / architect notes, doc / * ... * / specifications, @Safety and @Layer stereotypes, and custom vertical whitespace alignment. - **When**: deap_core::sysml_cst normalizes CRLF to LF via normalize_line_endings, lexes the buffer via logos, and constructs the SysmlLanguage Red/Green CST via rowan::GreenNodeBuilder (rowan pinned identically to. */
    use case def UC_AC_01_Lossless_logos_and_rowan_Concrete_Syntax_Tree_Full_Fidelity_Roundtrip_and_Cross_Platform_CRLF_Normalization {
        subject FormattingPreservingSyncSplicer;
        objective "Lossless logos and rowan Concrete Syntax Tree Full Fidelity Roundtrip and Cross Platform CRLF Normalization";
    }
    doc /* - **Given**: Two negative .sysml fixtures: Fixture A containing an unclosed part def attribute missing ; followed by a valid requirement def block, and Fixture B containing 65 nested package / part blocks exceeding MAX_AST_DEPTH = 64 (**C-INC-01**). - **When**: parse_lossless_sysml_cst and render_ariadne_diagnostic process Fixture A and Fixture B. - **Then**: Neither parse panics or overflows the call stack; Fixture A wraps the malformed attribute in SysmlSyntaxKind::ErrorNode, recovers cleanly to parse the subsequent requirement def, and emits both an ANSI-free ariadne byte-span report and a. */
    use case def UC_AC_02_Resilient_Error_Recovery_Parsing_MAX_AST_DEPTH_Guard_and_Deterministic_Ariadne_Byte_Span_Diagnostics {
        subject FormattingPreservingSyncSplicer;
        objective "Resilient Error Recovery Parsing MAX AST DEPTH Guard and Deterministic Ariadne Byte Span Diagnostics";
    }
    doc /* - **Given**: An existing .sysml schema containing human-authored architectural comments, custom indentation, and three part def subsystems alongside synchronized docs/features/ and docs/epics/ Markdown specifications, where only one part def port signature is updated during reverse sync, plus a negative splice request containing overlapping Utf8TextRange intervals. - **When**: apply_formatting_preserving_splices executes forward/reverse splice plans and evaluates the negative overlapping splice plan. - **Then**: Only the modified declaration's CST span is spliced while 100% of untouched. */
    use case def UC_AC_03_Formatting_Preserving_Bidirectional_CST_Splicing_Across_Forward_and_Reverse_Sync {
        subject FormattingPreservingSyncSplicer;
        objective "Formatting Preserving Bidirectional CST Splicing Across Forward and Reverse Sync";
    }
    doc /* - **Given**: A multi-package .sysml workspace loaded into compile_sysml::salsa_db across all 4 query layers (SourceDatabase, CstDatabase, HirDatabase, SafetyVerificationDatabase) with query execution counters attached to CST, HIR, semantic validation, STPA/FMECA, and Kani queries. - **When**: Revision 2 appends comments and reformats whitespace inside one .sysml file without altering semantic declarations, followed by Revision 3 modifying a port def [min..max] bound inside that file. - **Then**: On Revision 2, evaluate_hir_early_cutoff returns EarlyCutoffDecision::CutoffPreservedDownstream. */
    use case def UC_AC_04_Salsa_Early_Cutoff_Durability_Preventing_Downstream_Recomputations_on_Trivia_Edits {
        subject FormattingPreservingSyncSplicer;
        objective "Salsa Early Cutoff Durability Preventing Downstream Recomputations on Trivia Edits";
    }
    doc /* - **Given**: A negative multi-file .sysml fixture introducing a cyclic import / cyclic part def :> BasePart specialization loop across two packages, alongside workspace SLOC and module-size audits. - **When**: compile_sysml::salsa_db evaluates scoped symbol resolution and verify-baseline audits legacy scanner deletion targets. - **Then**: Cycle recovery intercepts the cycle deterministically without infinite recursion or thread deadlock and fails closed with DEAP-E1004 (ariadne + NDJSON), while legacy hand-rolled scanners in ./crates/deap-core/src/sysml_ast/mod.rs (-2,400+ SLOC),. */
    use case def UC_AC_05_Fail_Closed_Salsa_Cycle_Detection_and_Net_Negative_Legacy_Scanner_Retirement {
        subject FormattingPreservingSyncSplicer;
        objective "Fail Closed Salsa Cycle Detection and Net Negative Legacy Scanner Retirement";
    }
    doc /* - **Given**: Six valid polyglot specification fixtures (Rust .rs via ra_ap_syntax, Dart .dart via deap_core::dart_cst, OpenAPI/JSON-Schema .json via typed serde_json AST, OMG IDL .idl, IETF YANG .yang, and Markdown .md via logos+rowan CSTs in ingest_sysml::polyglot_cst) alongside malformed counterparts at known byte offsets. - **When**: ingest_sysml::polyglot_cst resolves each backend via resolve_polyglot_backend and lowers each fixture into compile_sysml::salsa_db. - **Then**: All 6 valid fixtures lower deterministically into canonical HirPackageSummary and HirSourceMap records. */
    use case def UC_AC_06_Cohesive_Typed_Polyglot_CST_AST_Lowering_with_Zero_rustc_private_and_Exact_Byte_Span_Diagnostics {
        subject FormattingPreservingSyncSplicer;
        objective "Cohesive Typed Polyglot CST AST Lowering with Zero rustc private and Exact Byte Span Diagnostics";
    }
    doc /* - **Given**: ./docs/designs/blueprint-10-rowan-salsa-incremental-sysml-compiler.md residing in the upstream compiler repository alongside workspace governance rules. - **When**: ./target/release/verify-baseline check-blueprint ./docs/designs/blueprint-10-rowan-salsa-incremental-sysml-compiler.md and ./target/release/verify-baseline . --no-domain are executed. - **Then**: All 12 mechanical blueprint verification gates and all upstream workspace baseline checks pass with exit code 0, atomically persisting ./docs/designs/blueprint-10-rowan-salsa-incremental-sysml-compiler.sysml and. */
    use case def UC_AC_07_Twelve_Gate_Blueprint_Verification_and_Baseline_Cleanliness_Gate_Conformance {
        subject FormattingPreservingSyncSplicer;
        objective "Twelve Gate Blueprint Verification and Baseline Cleanliness Gate Conformance";
    }
    doc /* Extracted from Mermaid sequenceDiagram specification. */
    interaction def BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine_SequenceInteraction {
        lifeline TypedPolyglotIngestionFrontend : TypedPolyglotIngestionFrontend;
        lifeline SalsaIncrementalCompilerDatabase : SalsaIncrementalCompilerDatabase;
        lifeline LogosRowanSysmlCstEngine : LogosRowanSysmlCstEngine;
        lifeline FormattingPreservingSyncSplicer : FormattingPreservingSyncSplicer;
        message resolve_polyglot_backend from TypedPolyglotIngestionFrontend to TypedPolyglotIngestionFrontend;
        message normalize_line_endings from SalsaIncrementalCompilerDatabase to LogosRowanSysmlCstEngine;
        message reply_String from LogosRowanSysmlCstEngine to SalsaIncrementalCompilerDatabase;
        message parse_lossless_sysml_cst from SalsaIncrementalCompilerDatabase to LogosRowanSysmlCstEngine;
        message reply_CstParseReceipt from LogosRowanSysmlCstEngine to SalsaIncrementalCompilerDatabase;
        message verify_lossless_cst_invariants from SalsaIncrementalCompilerDatabase to LogosRowanSysmlCstEngine;
        message reply_Ok from LogosRowanSysmlCstEngine to SalsaIncrementalCompilerDatabase;
        message evaluate_hir_early_cutoff from SalsaIncrementalCompilerDatabase to SalsaIncrementalCompilerDatabase;
        message apply_formatting_preserving_splices from FormattingPreservingSyncSplicer to FormattingPreservingSyncSplicer;
        message render_ariadne_diagnostic from SalsaIncrementalCompilerDatabase to SalsaIncrementalCompilerDatabase;
    }
    doc /* Permitted Stage-1 versus Stage-2 SHA-256 digest mismatch count across bijective (`BP-03`) and self-hosted modular codegen (`E0705`). [unit: files] */
    part def FormattingPreservingSyncSplicer {
        attribute stage_fixed_point_mismatch_files : Integer = 0;
        doc /* Permitted byte mutations to comments and whitespace outside modified declaration `Utf8TextRange` intervals during incremental sync. [unit: bytes] */
        attribute untouched_trivia_mutation_bytes : Integer = 0;
        doc /* Enforces pairwise-disjoint UTF-8 boundary verification and post-splice CST re-parse checks in `deap_core::sysml_cst::printer` before atomic commit. [unit: flag] */
        attribute atomic_disjoint_splice : Boolean = true;
        doc /* Receives forward (`SysML` to `Markdown`/ICD/WBS) or reverse (`Markdown` to `SysML`) structural HIR diff descriptors. [protocol: BidirectionalHirDelta] */
        port in sync_delta_in : String {
            flow in sync_delta_in_flow : String;
        }
        doc /* Commits formatting-preserving `rowan` CST child splices and disjoint byte patches atomically to workspace storage. [protocol: LosslessSplicedSysmlAndMarkdown] */
        port out spliced_artifact_out : String {
            flow out spliced_artifact_out_flow : String;
        }
        doc /* Emits fail-closed `ariadne` and NDJSON `DEAP-E1003` diagnostics on overlapping splice intervals or post-splice syntax regressions. [protocol: AriadneAndJsonDiagnosticStream] */
        port out splice_diag_out : String {
            flow out splice_diag_out_flow : String;
        }
    }
    doc /* Hard structural nesting ceiling across nested `package`, `part`, `port`, `action`, `state`, and expression CST blocks in `deap_core::sysml_cst::parser`. [unit: levels] */
    part def LogosRowanSysmlCstEngine {
        attribute max_ast_depth : Integer = 64;
        doc /* Requires root `rowan::GreenNode` byte concatenation to match the LF-normalized UTF-8 `.sysml` buffer byte-for-byte including comments and whitespace. [unit: flag] */
        attribute lossless_trivia_preserved : Boolean = true;
        doc /* Wraps malformed token spans in `ErrorNode` subtrees and continues parsing at synchronization tokens while emitting `ariadne` / JSON `DEAP-E1001` diagnostics. [unit: flag] */
        attribute resilient_error_recovery : Boolean = true;
        doc /* Canonical deterministic blueprint component identifier (`DEAP-BP-DET-10`). [unit: uuidv5] */
        attribute deterministic_uuid : String = "6ba7b810-9dad-11d1-80b4-00c04fd430ca";
        doc /* Receives CRLF-to-LF normalized UTF-8 `.sysml` or `.kerml` source text via `camino::Utf8Path` from `SourceDatabase` without pre-lexer sigil rewriting. [protocol: Utf8SysmlSourceBuffer] */
        port in sysml_source_in : String {
            flow in sysml_source_in_flow : String;
        }
        doc /* Emits immutable, thread-safe `rowan::GreenNode` (`rowan` pinned to `ra_ap_syntax`) and typed `SysmlSyntaxNode` projection. [protocol: RowanGreenSysmlTree] */
        port out green_cst_out : String {
            flow out green_cst_out_flow : String;
        }
        doc /* Streams deterministic `ariadne` byte-span reports and single-line `DEAP-E1001` / `DEAP-E1002` NDJSON diagnostics. [protocol: AriadneAndJsonDiagnosticStream] */
        port out syntax_diag_out : String {
            flow out syntax_diag_out_flow : String;
        }
    }
    doc /* Mandatory count of stratified `salsa` query groups in `compile_sysml::salsa_db` (`SourceDatabase`, `CstDatabase`, `HirDatabase`, `SafetyVerificationDatabase`). [unit: layers] */
    part def SalsaIncrementalCompilerDatabase {
        attribute query_layer_count : Integer = 4;
        doc /* Required count of downstream STPA, Kani, and baseline verification query recomputations when edits alter only non-semantic trivia. [unit: queries] */
        attribute trivia_edit_downstream_recomputations : Integer = 0;
        doc /* Mandates `BTreeMap` and `BTreeSet` deterministic ordering and forbids filesystem or clock side effects inside derived `salsa` queries. [unit: flag] */
        attribute pure_deterministic_btreemap : Boolean = true;
        doc /* Intercepts cyclic `salsa` dependency chains across imports or specializations and emits deterministic `DEAP-E1004` `ariadne` + JSON diagnostics. [unit: flag] */
        attribute cycle_recovery_fail_closed : Boolean = true;
        doc /* Receives `FileId` buffer updates, `camino::Utf8PathBuf` paths, `DurabilityTier` (`Low`, `Medium`, `High`), and LF-normalized SHA-256 input digests. [protocol: SalsaSourceInputRevision] */
        port in source_revision_in : String {
            flow in source_revision_in_flow : String;
        }
        doc /* Emits trivia-erased `HirPackageSummary` semantic graphs decoupled from `DefId` to `Utf8TextRange` `HirSourceMap` tables for early cutoff. [protocol: InternedHirPackageAndSourceMap] */
        port out hir_model_out : String {
            flow out hir_model_out_flow : String;
        }
        doc /* Emits memoized STPA UCA/FMECA matrices, Kani CBMC proof harnesses, `ariadne` diagnostics, and `verify-baseline` parity receipts. [protocol: MemoizedStpaKaniBaselineReceipt] */
        port out safety_verification_out : String {
            flow out safety_verification_out_flow : String;
        }
    }
    doc /* Count of typed CST/AST frontends in `ingest_sysml::polyglot_cst` (`ra_ap_syntax` Rust, `deap_core::dart_cst` Dart, typed `serde_json` OpenAPI/JSON-Schema, `logos`+`rowan` OMG IDL, IETF YANG, Markdown). [unit: frontends] */
    part def TypedPolyglotIngestionFrontend {
        attribute polyglot_frontend_count : Integer = 6;
        doc /* Enforces `ZERO #![feature(rustc_private)]` occurrences across all workspace crates under `./crates/ *`. [unit: crates] */
        attribute rustc_private_feature_count : Integer = 0;
        doc /* Emits deterministic `ariadne` byte-span reports and `--message-format=json` diagnostics (`DEAP-E1005`) across all polyglot formats. [unit: flag] */
        attribute json_and_ariadne_span_diagnostics : Boolean = true;
        doc /* Enforces linear-time `logos` DFA / `rowan` CST and typed `serde_json` AST parsing with zero hand-rolled line scanners or NFA backtracking (`Check 34`). [unit: flag] */
        attribute zero_backtracking_lexers : Boolean = true;
        doc /* Receives LF-normalized Rust (`.rs`), Dart (`.dart`), JSON-Schema/OpenAPI (`.json`), OMG IDL (`.idl`), IETF YANG (`.yang`), or Markdown (`.md`) buffers. [protocol: PolyglotSpecBuffer] */
        port in polyglot_source_in : String {
            flow in polyglot_source_in_flow : String;
        }
        doc /* Lowers typed polyglot syntax trees directly into interned `compile_sysml::salsa_db` `HirPackageSummary` and `HirSourceMap` records. [protocol: CanonicalHirDatabaseInput] */
        port out lowered_hir_out : String {
            flow out lowered_hir_out_flow : String;
        }
        doc /* Streams deterministic `ariadne` byte-span reports and single-line `DEAP-E1005` NDJSON diagnostics on syntax or schema-pointer errors. [protocol: AriadneAndJsonDiagnosticStream] */
        port out ingest_diag_out : String {
            flow out ingest_diag_out_flow : String;
        }
    }
}