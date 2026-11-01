doc /* Translated from Level 0 OEM Markdown specification */
package BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine {
    attribute Blueprint_ID : String = "BP-10";
    attribute Requirement_ID : String = "REQ-0610";
    attribute Title : String = "Incremental Lossless SysML v2 and Polyglot Compiler Engine (logos + rowan + salsa)";
    attribute Status : String = "Draft";
    attribute Target_Crates : String = "crates/deap-core, crates/compile-sysml, crates/ingest-sysml, crates/verify-baseline";
    attribute Tracked_Issues : String = "#742";
    attribute Generation_Mode : String = "subagent";
    attribute Specification_Source : String = "crates/compile-sysml/src/lexer/scanner.rs:1-60";
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
    doc /* ingestPolyglotToHir(fileId, format) */
    action def ingestPolyglotToHir;
    doc /* parseLosslessSysmlCst(source) */
    action def parseLosslessSysmlCst;
    doc /* evaluateSafetyAndEarlyCutoff(fileId) */
    action def evaluateSafetyAndEarlyCutoff;
    doc /* computeBidirectionalSplice(fileId) */
    action def computeBidirectionalSplice;
    doc /* verifyPostSpliceCst(splicedSource) */
    action def verifyPostSpliceCst;
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
        polyglot_frontend_count >= 5 and polyglot_frontend_count <= 5;
    }
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0610_BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine {
        id = "REQ-0610";
        text = "IncrementalLosslessSysmlAndPolyglotCompilerEngine (**REQ-0610** / BP-10 / DEAP-BP-DET-10, UUIDv5 6ba7b810-9dad-11d1-80b4-00c04fd430ca, Class P linear DFA lexing, lossless Red/Green syntax-tree construction, and O(delta) early-cutoff incremental query complexity, governed by DO-178C Level A, DO-330";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_lossless_logos_and_rowan_concrete_syntax_tree_full_fidelity_roundtrip_and_trivia_preservation : String = "- **Given**: A .sysml fixture containing nested package, part def, port def, action def, state def, transition, requirement def, and constraint def blocks interleaved with inline // comments, multi-line / * ... * / architect notes, doc / * ... * / specifications, @Safety and @Layer stereotypes, composite generic types, and custom vertical whitespace alignment. - **When**: LogosRowanSysmlCstEngine lexes the buffer via logos and constructs the SysmlLanguage Red/Green concrete syntax tree via rowan::GreenNodeBuilder. - **Then**: Root SyntaxNode::text() equals the input UTF-8 byte buffer.";
        attribute ac_02_resilient_error_recovery_parsing_on_incomplete_syntax_and_fail_closed_max_ast_depth_guard : String = "- **Given**: Two negative .sysml fixtures: Fixture A containing an unclosed part def attribute missing ; followed by a valid requirement def block (simulating mid-keystroke IDE/agent editing), and Fixture B containing 65 nested package / part blocks exceeding MAX_AST_DEPTH = 64 (**C-INC-01**). - **When**: LogosRowanSysmlCstEngine parses Fixture A and Fixture B under --message-format=json. - **Then**: Neither parse panics or overflows the call stack; Fixture A wraps the malformed attribute in SysmlSyntaxKind::ERROR_NODE, recovers cleanly to parse the subsequent requirement def into a valid red.";
        attribute ac_03_formatting_preserving_bidirectional_cst_splicing_across_forward_and_reverse_sync : String = "- **Given**: An existing .sysml schema containing human-authored architectural comments, custom indentation, and three part def subsystems alongside synchronized docs/features/ and docs/epics/ Markdown specifications, where only one part def port signature is updated in Markdown during reverse sync (and reciprocally one constraint def bound is updated in .sysml during forward sync), plus a negative splice request containing overlapping TextRange intervals. - **When**: FormattingPreservingSyncSplicer executes crates/compile-sysml/src/sync/reverse.rs and crates/compile-sysml/src/sync/forward/.";
        attribute ac_04_salsa_early_cutoff_durability_preventing_downstream_recomputations_on_trivia_edits : String = "- **Given**: A multi-package .sysml workspace loaded into SalsaIncrementalCompilerDatabase across all 4 query layers (SourceDatabase, CstDatabase, HirDatabase, SafetyVerificationDatabase) with query execution counters attached to parse_sysml_cst, lower_sysml_hir, validate_semantics, derive_stpa_fmeca, and emit_kani_harnesses. - **When**: Revision 2 appends comments and reformats whitespace inside one .sysml file without altering semantic declarations, followed by Revision 3 modifying a port def [min..max] bound inside that file. - **Then**: On Revision 2, parse_sysml_cst and lower_sysml_hir.";
        attribute ac_05_fail_closed_salsa_cycle_detection_and_referential_integrity_verification : String = "- **Given**: A negative multi-file .sysml fixture introducing a cyclic import / cyclic part def :> BasePart specialization loop across two packages in HirDatabase. - **When**: SalsaIncrementalCompilerDatabase evaluates scoped symbol resolution and HIR lowering for the cyclic workspace. - **Then**: Cycle recovery intercepts the cycle deterministically without infinite recursion or thread deadlock, fails closed with non-zero exit code, and emits a single-line --message-format=json diagnostic DEAP-E1004 listing the ordered cycle DefId chain and primary TextRange spans.";
        attribute ac_06_typed_cst_lowering_and_unified_json_span_diagnostics_across_all_five_polyglot_frontends : String = "- **Given**: Five valid polyglot specification fixtures (Rust .rs, Markdown .md BOM/Port/Constraint tables, OMG IDL .idl, RFC 7950 YANG .yang, and OpenAPI 3.1 .json) alongside five malformed counterparts containing a syntax or unresolved schema-pointer error at a known byte offset. - **When**: TypedPolyglotIngestionFrontend lowers each fixture through its typed syntax tree (ra_ap_syntax, MdSyntaxKind, IdlSyntaxKind, YangSyntaxKind, OpenApiSyntaxKind) into HirDatabase with --message-format=json. - **Then**: All 5 valid fixtures lower deterministically into canonical HirPackage and SourceMap.";
        attribute ac_07_twelve_gate_blueprint_verification_and_baseline_cleanliness_gate_conformance : String = "- **Given**: docs/designs/blueprint-10-rowan-salsa-incremental-sysml-compiler.md residing in the upstream compiler repository alongside workspace governance rules. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-10-rowan-salsa-incremental-sysml-compiler.md and ./target/release/verify-baseline . --no-domain are executed. - **Then**: All 12 mechanical blueprint verification gates and all upstream workspace baseline checks pass with exit code 0, atomically persisting docs/designs/blueprint-10-rowan-salsa-incremental-sysml-compiler.sysml and.";
        doc /* \begin{aligned} \forall S \in \Sigma^*, \quad \bigoplus_{i=1}^{k} \text{bytes}(t_i) &= S \land \sum_{i=1}^{k} \lvert \text{range}(t_i) \rvert = \lvert S \rvert \\ \text{depth}(T_{\text{green}}(S)) &\le 64 \land \forall e \in \mathcal{D}_{\text{err}}(S), \quad \exists v \in V(T_{\text{green}}(S)) : \text{kind}(v) = \text{ERROR\_NODE} \end{aligned} */
        constraint def Invariant_REQ_0610_SC_01;
        doc /* \begin{aligned} \forall i \in \{1, \dots, m - 1\}, \quad 0 \le a_i \le b_i \le a_{i+1} \le \lvert S \rvert &\land \Delta_{\text{trivia}}(\mathcal{U}(S, \mathcal{E}), S') = 0 \\ \text{Lower}_{\text{HIR}}(\text{Parse}_{\text{CST}}(\text{Emit}_{\text{canonical}}(H))) = H &\land \text{SHA-256}(\mathcal{T}_{\text{Stage 1}}) = \text{SHA-256}(\mathcal{T}_{\text{Stage 2}}) \end{aligned} */
        constraint def Invariant_REQ_0610_SC_02;
        doc /* \begin{aligned} H_{f^*}^{(r+1)} = H_{f^*}^{(r)} &\implies \lvert \text{Recomputed}(Q_4, r \to r+1) \rvert = 0 \\ T_{\text{inc}}(r \to r+1) &= O\left( \sum_{f \in \Delta_{\text{src}}} \lvert S_f \rvert + \sum_{q \in \text{Affected}(Q_3 \cup Q_4, \Delta_{\text{HIR}})} \text{cost}(q) \right) \end{aligned} */
        constraint def Invariant_REQ_0610_SC_03_Incremental_Complexity;
        doc /* \begin{aligned} \forall p \in \mathcal{P}, \forall S \in \Sigma^*, \quad \text{SHA-256}(\Phi_p(S)) &= \text{const}(p, S) \land \forall d \in \mathcal{D}_{\text{json}}, \quad \text{count}(\text{encode}(d), \text{0x1B}) = 0 \end{aligned} */
        constraint def Invariant_REQ_0610_SC_04;
        require Invariant_REQ_0610_SC_01;
        require Invariant_REQ_0610_SC_02;
        require Invariant_REQ_0610_SC_03_Incremental_Complexity;
        require Invariant_REQ_0610_SC_04;
        derived from REQ_0603;
        derived from REQ_0607;
        derived from REQ_0608;
        derived from REQ_0609;
        verify by AC_01_Lossless_logos_and_rowan_Concrete_Syntax_Tree_Full_Fidelity_Roundtrip_and_Trivia_Preservation;
        verify by AC_02_Resilient_Error_Recovery_Parsing_on_Incomplete_Syntax_and_Fail_Closed_MAX_AST_DEPTH_Guard;
        verify by AC_03_Formatting_Preserving_Bidirectional_CST_Splicing_Across_Forward_and_Reverse_Sync;
        verify by AC_04_Salsa_Early_Cutoff_Durability_Preventing_Downstream_Recomputations_on_Trivia_Edits;
        verify by AC_05_Fail_Closed_Salsa_Cycle_Detection_and_Referential_Integrity_Verification;
        verify by AC_06_Typed_CST_Lowering_and_Unified_JSON_Span_Diagnostics_Across_All_Five_Polyglot_Frontends;
        verify by AC_07_Twelve_Gate_Blueprint_Verification_and_Baseline_Cleanliness_Gate_Conformance;
        satisfy by FormattingPreservingSyncSplicer;
    }
    doc /* Extracted from Mermaid stateDiagram specification. */
    state def BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine_LifecycleState {
        state Idle;
        state RecordingSalsaInputRevision;
        state LexingAndBuildingRowanCst;
        state EmittingJsonDiagnostics;
        state LoweringTriviaErasedHir;
        state EarlyCutoffShortCircuited;
        state EvaluatingSafetyAndKaniQueries;
        state SplicingFormattingPreservingSync;
        state VerifiedIncrementalClean;
        transition Initial_to_Idle first Initial then Idle;
        transition Idle_to_RecordingSalsaInputRevision_on_setFileSource first Idle accept setFileSource then RecordingSalsaInputRevision;
        transition RecordingSalsaInputRevision_to_LexingAndBuildingRowanCst_on_queryCstDatabase first RecordingSalsaInputRevision accept queryCstDatabase then LexingAndBuildingRowanCst;
        transition LexingAndBuildingRowanCst_to_EmittingJsonDiagnostics_on_rejectMaxAstDepthOverflowE1002 first LexingAndBuildingRowanCst accept rejectMaxAstDepthOverflowE1002 then EmittingJsonDiagnostics;
        transition LexingAndBuildingRowanCst_to_LoweringTriviaErasedHir_on_greenCstReadyWithOptionalRecoveredErrors first LexingAndBuildingRowanCst accept greenCstReadyWithOptionalRecoveredErrors then LoweringTriviaErasedHir;
        transition LoweringTriviaErasedHir_to_EmittingJsonDiagnostics_on_rejectCyclicSalsaQueryE1004 first LoweringTriviaErasedHir accept rejectCyclicSalsaQueryE1004 then EmittingJsonDiagnostics;
        transition LoweringTriviaErasedHir_to_EarlyCutoffShortCircuited_on_hirDigestUnchangedZeroDownstreamInvalidation first LoweringTriviaErasedHir accept hirDigestUnchangedZeroDownstreamInvalidation then EarlyCutoffShortCircuited;
        transition LoweringTriviaErasedHir_to_EvaluatingSafetyAndKaniQueries_on_hirSemanticDeltaDetected first LoweringTriviaErasedHir accept hirSemanticDeltaDetected then EvaluatingSafetyAndKaniQueries;
        transition EvaluatingSafetyAndKaniQueries_to_SplicingFormattingPreservingSync_on_safetyAndSemanticGatesValid first EvaluatingSafetyAndKaniQueries accept safetyAndSemanticGatesValid then SplicingFormattingPreservingSync;
        transition SplicingFormattingPreservingSync_to_EmittingJsonDiagnostics_on_rejectOverlappingSpliceE1003 first SplicingFormattingPreservingSync accept rejectOverlappingSpliceE1003 then EmittingJsonDiagnostics;
        transition SplicingFormattingPreservingSync_to_VerifiedIncrementalClean_on_bijectiveAndStage2FixedPointVerified first SplicingFormattingPreservingSync accept bijectiveAndStage2FixedPointVerified then VerifiedIncrementalClean;
        transition EarlyCutoffShortCircuited_to_VerifiedIncrementalClean_on_reuseMemoizedSafetyVerificationReceipt first EarlyCutoffShortCircuited accept reuseMemoizedSafetyVerificationReceipt then VerifiedIncrementalClean;
        transition EmittingJsonDiagnostics_to_Terminal first EmittingJsonDiagnostics then Terminal;
        transition VerifiedIncrementalClean_to_Terminal first VerifiedIncrementalClean then Terminal;
    }
    doc /* - **Given**: A .sysml fixture containing nested package, part def, port def, action def, state def, transition, requirement def, and constraint def blocks interleaved with inline // comments, multi-line / * ... * / architect notes, doc / * ... * / specifications, @Safety and @Layer stereotypes, composite generic types, and custom vertical whitespace alignment. - **When**: LogosRowanSysmlCstEngine lexes the buffer via logos and constructs the SysmlLanguage Red/Green concrete syntax tree via rowan::GreenNodeBuilder. - **Then**: Root SyntaxNode::text() equals the input UTF-8 byte buffer. */
    use case def UC_AC_01_Lossless_logos_and_rowan_Concrete_Syntax_Tree_Full_Fidelity_Roundtrip_and_Trivia_Preservation {
        subject FormattingPreservingSyncSplicer;
        objective "Lossless logos and rowan Concrete Syntax Tree Full Fidelity Roundtrip and Trivia Preservation";
    }
    doc /* - **Given**: Two negative .sysml fixtures: Fixture A containing an unclosed part def attribute missing ; followed by a valid requirement def block (simulating mid-keystroke IDE/agent editing), and Fixture B containing 65 nested package / part blocks exceeding MAX_AST_DEPTH = 64 (**C-INC-01**). - **When**: LogosRowanSysmlCstEngine parses Fixture A and Fixture B under --message-format=json. - **Then**: Neither parse panics or overflows the call stack; Fixture A wraps the malformed attribute in SysmlSyntaxKind::ERROR_NODE, recovers cleanly to parse the subsequent requirement def into a valid red. */
    use case def UC_AC_02_Resilient_Error_Recovery_Parsing_on_Incomplete_Syntax_and_Fail_Closed_MAX_AST_DEPTH_Guard {
        subject FormattingPreservingSyncSplicer;
        objective "Resilient Error Recovery Parsing on Incomplete Syntax and Fail Closed MAX AST DEPTH Guard";
    }
    doc /* - **Given**: An existing .sysml schema containing human-authored architectural comments, custom indentation, and three part def subsystems alongside synchronized docs/features/ and docs/epics/ Markdown specifications, where only one part def port signature is updated in Markdown during reverse sync (and reciprocally one constraint def bound is updated in .sysml during forward sync), plus a negative splice request containing overlapping TextRange intervals. - **When**: FormattingPreservingSyncSplicer executes crates/compile-sysml/src/sync/reverse.rs and crates/compile-sysml/src/sync/forward/. */
    use case def UC_AC_03_Formatting_Preserving_Bidirectional_CST_Splicing_Across_Forward_and_Reverse_Sync {
        subject FormattingPreservingSyncSplicer;
        objective "Formatting Preserving Bidirectional CST Splicing Across Forward and Reverse Sync";
    }
    doc /* - **Given**: A multi-package .sysml workspace loaded into SalsaIncrementalCompilerDatabase across all 4 query layers (SourceDatabase, CstDatabase, HirDatabase, SafetyVerificationDatabase) with query execution counters attached to parse_sysml_cst, lower_sysml_hir, validate_semantics, derive_stpa_fmeca, and emit_kani_harnesses. - **When**: Revision 2 appends comments and reformats whitespace inside one .sysml file without altering semantic declarations, followed by Revision 3 modifying a port def [min..max] bound inside that file. - **Then**: On Revision 2, parse_sysml_cst and lower_sysml_hir. */
    use case def UC_AC_04_Salsa_Early_Cutoff_Durability_Preventing_Downstream_Recomputations_on_Trivia_Edits {
        subject FormattingPreservingSyncSplicer;
        objective "Salsa Early Cutoff Durability Preventing Downstream Recomputations on Trivia Edits";
    }
    doc /* - **Given**: A negative multi-file .sysml fixture introducing a cyclic import / cyclic part def :> BasePart specialization loop across two packages in HirDatabase. - **When**: SalsaIncrementalCompilerDatabase evaluates scoped symbol resolution and HIR lowering for the cyclic workspace. - **Then**: Cycle recovery intercepts the cycle deterministically without infinite recursion or thread deadlock, fails closed with non-zero exit code, and emits a single-line --message-format=json diagnostic DEAP-E1004 listing the ordered cycle DefId chain and primary TextRange spans. */
    use case def UC_AC_05_Fail_Closed_Salsa_Cycle_Detection_and_Referential_Integrity_Verification {
        subject FormattingPreservingSyncSplicer;
        objective "Fail Closed Salsa Cycle Detection and Referential Integrity Verification";
    }
    doc /* - **Given**: Five valid polyglot specification fixtures (Rust .rs, Markdown .md BOM/Port/Constraint tables, OMG IDL .idl, RFC 7950 YANG .yang, and OpenAPI 3.1 .json) alongside five malformed counterparts containing a syntax or unresolved schema-pointer error at a known byte offset. - **When**: TypedPolyglotIngestionFrontend lowers each fixture through its typed syntax tree (ra_ap_syntax, MdSyntaxKind, IdlSyntaxKind, YangSyntaxKind, OpenApiSyntaxKind) into HirDatabase with --message-format=json. - **Then**: All 5 valid fixtures lower deterministically into canonical HirPackage and SourceMap. */
    use case def UC_AC_06_Typed_CST_Lowering_and_Unified_JSON_Span_Diagnostics_Across_All_Five_Polyglot_Frontends {
        subject FormattingPreservingSyncSplicer;
        objective "Typed CST Lowering and Unified JSON Span Diagnostics Across All Five Polyglot Frontends";
    }
    doc /* - **Given**: docs/designs/blueprint-10-rowan-salsa-incremental-sysml-compiler.md residing in the upstream compiler repository alongside workspace governance rules. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-10-rowan-salsa-incremental-sysml-compiler.md and ./target/release/verify-baseline . --no-domain are executed. - **Then**: All 12 mechanical blueprint verification gates and all upstream workspace baseline checks pass with exit code 0, atomically persisting docs/designs/blueprint-10-rowan-salsa-incremental-sysml-compiler.sysml and. */
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
        message ingestPolyglotToHir from TypedPolyglotIngestionFrontend to SalsaIncrementalCompilerDatabase;
        message reply_CanonicalHirDatabaseInput from SalsaIncrementalCompilerDatabase to TypedPolyglotIngestionFrontend;
        message parseLosslessSysmlCst from SalsaIncrementalCompilerDatabase to LogosRowanSysmlCstEngine;
        message reply_RowanGreenSysmlParse from LogosRowanSysmlCstEngine to SalsaIncrementalCompilerDatabase;
        message evaluateSafetyAndEarlyCutoff from SalsaIncrementalCompilerDatabase to SalsaIncrementalCompilerDatabase;
        message computeBidirectionalSplice from FormattingPreservingSyncSplicer to SalsaIncrementalCompilerDatabase;
        message reply_DisjointCstSplicePlan from SalsaIncrementalCompilerDatabase to FormattingPreservingSyncSplicer;
        message verifyPostSpliceCst from FormattingPreservingSyncSplicer to LogosRowanSysmlCstEngine;
        message reply_VerifiedGreenSysmlReceipt from LogosRowanSysmlCstEngine to FormattingPreservingSyncSplicer;
    }
    doc /* Permitted Stage-1 versus Stage-2 SHA-256 digest mismatch count across self-hosted modular codegen (`E0705`). [unit: files] */
    part def FormattingPreservingSyncSplicer {
        attribute stage_fixed_point_mismatch_files : Integer = 0;
        doc /* Permitted byte mutations to comments and whitespace outside modified declaration `TextRange` intervals during incremental sync. [unit: bytes] */
        attribute untouched_trivia_mutation_bytes : Integer = 0;
        doc /* Enforces pairwise-disjoint UTF-8 boundary verification and post-splice CST re-parse checks before atomic file commit. [unit: flag] */
        attribute atomic_disjoint_splice : Boolean = true;
        doc /* Receives forward (`SysML` to `Markdown`/ICD/WBS) or reverse (`Markdown` to `SysML`) structural HIR diff descriptors. [protocol: BidirectionalHirDelta] */
        port in sync_delta_in : String {
            flow in sync_delta_in_flow : String;
        }
        doc /* Commits formatting-preserving `rowan` CST child splices and disjoint byte patches atomically to workspace storage. [protocol: LosslessSplicedSysmlAndMarkdown] */
        port out spliced_artifact_out : String {
            flow out spliced_artifact_out_flow : String;
        }
        doc /* Emits fail-closed `DEAP-E1003` diagnostics on overlapping splice intervals, non-UTF-8 boundaries, or post-splice syntax regressions. [protocol: RustcJsonDiagnosticStream] */
        port out splice_diag_out : String {
            flow out splice_diag_out_flow : String;
        }
    }
    doc /* Hard structural nesting ceiling across nested `package`, `part`, `port`, `action`, `state`, and expression CST blocks. [unit: levels] */
    part def LogosRowanSysmlCstEngine {
        attribute max_ast_depth : Integer = 64;
        doc /* Requires root `rowan::GreenNode` byte concatenation to match the input UTF-8 `.sysml` buffer byte-for-byte including comments and whitespace. [unit: flag] */
        attribute lossless_trivia_preserved : Boolean = true;
        doc /* Wraps malformed token spans in `ERROR_NODE` subtrees and continues parsing at recovery synchronization tokens rather than aborting on first error. [unit: flag] */
        attribute resilient_error_recovery : Boolean = true;
        doc /* Canonical deterministic blueprint component identifier (`DEAP-BP-DET-10`). [unit: uuidv5] */
        attribute deterministic_uuid : String = "6ba7b810-9dad-11d1-80b4-00c04fd430ca";
        doc /* Receives raw UTF-8 `.sysml` or `.kerml` source text from `SourceDatabase` without pre-lexer sigil rewriting. [protocol: Utf8SysmlSourceBuffer] */
        port in sysml_source_in : String {
            flow in sysml_source_in_flow : String;
        }
        doc /* Emits immutable, thread-safe `rowan::GreenNode` concrete syntax tree and typed red `SyntaxNode` projection. [protocol: RowanGreenSysmlTree] */
        port out green_cst_out : String {
            flow out green_cst_out_flow : String;
        }
        doc /* Streams single-line `DEAP-E1001` syntax-recovery and `DEAP-E1002` depth-overflow NDJSON diagnostics with exact UTF-8 `TextRange` spans. [protocol: RustcJsonDiagnosticStream] */
        port out syntax_diag_out : String {
            flow out syntax_diag_out_flow : String;
        }
    }
    doc /* Mandatory count of stratified `salsa` query groups (`SourceDatabase`, `CstDatabase`, `HirDatabase`, `SafetyVerificationDatabase`). [unit: layers] */
    part def SalsaIncrementalCompilerDatabase {
        attribute query_layer_count : Integer = 4;
        doc /* Required count of downstream STPA, Kani, and baseline verification query recomputations when edits alter only non-semantic trivia. [unit: queries] */
        attribute trivia_edit_downstream_recomputations : Integer = 0;
        doc /* Mandates `BTreeMap` and `BTreeSet` deterministic ordering and forbids filesystem or clock side effects inside derived queries. [unit: flag] */
        attribute pure_deterministic_btreemap : Boolean = true;
        doc /* Intercepts cyclic `salsa` dependency chains across imports or specializations and emits deterministic `DEAP-E1004` diagnostics. [unit: flag] */
        attribute cycle_recovery_fail_closed : Boolean = true;
        doc /* Receives `FileId` buffer updates, `Durability` classifications (`LOW`, `MEDIUM`, `HIGH`), and SHA-256 input digests. [protocol: SalsaSourceInputRevision] */
        port in source_revision_in : String {
            flow in source_revision_in_flow : String;
        }
        doc /* Emits trivia-erased `HirPackage` semantic graphs decoupled from `DefId` to `TextRange` `SourceMap` tables for early cutoff. [protocol: InternedHirPackageAndSourceMap] */
        port out hir_model_out : String {
            flow out hir_model_out_flow : String;
        }
        doc /* Emits memoized STPA UCA/FMECA matrices, Kani CBMC proof harnesses, and `verify-baseline` parity verification results. [protocol: MemoizedStpaKaniBaselineReceipt] */
        port out safety_verification_out : String {
            flow out safety_verification_out_flow : String;
        }
    }
    doc /* Count of typed CST translator frontends in `crates/ingest-sysml/src/translators/` (`code.rs`, `markdown.rs`, `idl.rs`, `yang.rs`, `openapi.rs`). [unit: frontends] */
    part def TypedPolyglotIngestionFrontend {
        attribute polyglot_frontend_count : Integer = 5;
        doc /* Emits unified `rustc`-compatible `--message-format=json` span diagnostics (`DEAP-E1005`) across all polyglot ingestion formats. [unit: flag] */
        attribute json_span_diagnostics_enabled : Boolean = true;
        doc /* Enforces linear-time DFA/CST scanning across polyglot frontends with zero NFA backtracking engines (`Check 34`). [unit: flag] */
        attribute zero_backtracking_lexers : Boolean = true;
        doc /* Receives Rust (`.rs`), Dart (`.dart`), Markdown (`.md`), OMG IDL (`.idl`), RFC 7950 YANG (`.yang`), or OpenAPI 3.x (`.json`/`.yaml`) buffers. [protocol: PolyglotSpecBuffer] */
        port in polyglot_source_in : String {
            flow in polyglot_source_in_flow : String;
        }
        doc /* Lowers typed polyglot syntax trees directly into interned `HirDatabase` `PackageDef` and `SourceMap` records. [protocol: CanonicalHirDatabaseInput] */
        port out lowered_hir_out : String {
            flow out lowered_hir_out_flow : String;
        }
        doc /* Streams `DEAP-E1005` ingestion syntax, schema-reference, and type-mapping diagnostics with exact source byte/line spans. [protocol: RustcJsonDiagnosticStream] */
        port out ingest_diag_out : String {
            flow out ingest_diag_out_flow : String;
        }
    }
}