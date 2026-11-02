doc /* Translated from Level 0 OEM Markdown specification */
package BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint {
    attribute Blueprint_ID : String = "BP-06";
    attribute Requirement_ID : String = "REQ-0606";
    attribute Title : String = "Deterministic Static 5-Pillar Code Auditor and Defect Dossier Synthesizer Blueprint";
    attribute Status : String = "Approved";
    attribute Target_Crates : String = "crates/verify-baseline, crates/deap-core";
    attribute Tracked_Issues : String = "#465, #466, #467, #468";
    attribute Generation_Mode : String = "subagent";
    attribute Specification_Source : String = "crates/verify-baseline/src/main.rs:1-40";
    doc /* 5 to 10 LLM subagents per audit wave */
    attribute Subagent_Dispatches : String = "Zero subagents (100% native Rust binary execution)";
    doc /* LLM token window / hand-rolled line scanners */
    attribute Source_Parsing_Engine : String = "Typed ra_ap_syntax AST and logos/rowan deap_core::dart_cst engine";
    doc /* Heuristic prompt interpretation (hallucination-prone) */
    attribute Semantic_MIR_Analysis : String = "Stage-2 cargo-dylint driver (./lints/deap-dylint-mir/) reading rustc_middle::mir::Body (TyCtxt)";
    doc /* Manual LLM markdown drafting */
    attribute Dossier_Construction : String = "Structured AST emission via pulldown-cmark-to-cmark CommonMark serializer";
    doc /* Hand-authored Mermaid (error-prone) */
    attribute Mermaid_Diagram_Synthesis : String = "Deterministic Check 13B-compliant sequenceDiagram / classDiagram builder";
    doc /* External subprocess retry loop */
    attribute Schema_Validation_Gate : String = "In-process defect_dossier::validate_defect_dossier (12 checks)";
    doc /* 180 s to 600 s per module wave */
    attribute End_to_End_Execution_Time : String = "Sub-second (< 1.0 s) local scan and synthesis (O(N) linear complexity)";
    doc /* [unit: count] */
    attribute auditor_subagent_count : Real;
    doc /* [unit: checks] */
    attribute dossier_schema_check_count : Real;
    doc /* [unit: pillars] */
    attribute pillar_coverage_count : Real;
    doc /* [unit: B] */
    attribute closed_loop_mismatch_bytes : Real;
    doc /* [unit: SLOC] */
    attribute legacy_scanner_sloc_target : Real;
    doc /* [unit: SLOC] */
    attribute max_module_exec_sloc : Real;
    doc /* run_audit_code(workspace_root, args) */
    action def run_audit_code;
    doc /* audit_rust_cst(path, source) */
    action def audit_rust_cst;
    doc /* run_stage2_mir_analysis(workspace_root, target_path) */
    action def run_stage2_mir_analysis;
    doc /* audit_dart_cst(path, source) */
    action def audit_dart_cst;
    doc /* synthesize_defect_dossier_markdown(finding) */
    action def synthesize_defect_dossier_markdown;
    doc /* run_check_defect(args) */
    action def run_check_defect;
    doc /* gh issue create or gh issue edit with body-file */
    action def GhIssueCreateOrGhIssue;
    doc /* gh issue view --json title,body,url */
    action def GhIssueViewJsonTitleBody;
    doc /* [unit: count] */
    assert constraint C_AUD_01 {
        auditor_subagent_count >= 0 and auditor_subagent_count <= 0;
    }
    doc /* [unit: checks] */
    assert constraint C_AUD_02 {
        dossier_schema_check_count >= 12 and dossier_schema_check_count <= 12;
    }
    doc /* [unit: pillars] */
    assert constraint C_AUD_03 {
        pillar_coverage_count >= 5 and pillar_coverage_count <= 5;
    }
    doc /* [unit: B] */
    assert constraint C_AUD_04 {
        closed_loop_mismatch_bytes >= 0 and closed_loop_mismatch_bytes <= 0;
    }
    doc /* [unit: SLOC] */
    assert constraint C_AUD_05 {
        legacy_scanner_sloc_target >= -2399 and legacy_scanner_sloc_target <= -2399;
    }
    doc /* [unit: SLOC] */
    assert constraint C_AUD_06 {
        max_module_exec_sloc >= 0 and max_module_exec_sloc <= 380;
    }
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0606_BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint {
        id = "REQ-0606";
        text = "The DeterministicStaticDefectAuditor subsystem (**REQ-0606** / BP-06 / DEAP-BP-DET-06, UUIDv5: 6ba7b810-9dad-11d1-80b4-00c04fd430c6, complexity class Class P linear single-pass syntax tree and MIR traversal, governed by DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C) SHALL";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_contract_a_static_five_pillar_typed_ast_defect_detection : String = "- **Given**: A target Rust source file containing representative defects across the 5 pillars (unchecked as casts in Memory Safety E0601, unbounded cache mutations in Resource Lifecycle E0602, TOCTOU file checks in Concurrency E0603, 0-assertion tests in Test Integrity E0604, or missing Realises: tags in Semantic Traceability E0605). - **When**: ./target/release/verify-baseline audit-code --path <PATH> --dry-run is executed. - **Then**: CstPillarAuditor deterministically parses the syntax tree using ra_ap_syntax under Contract A, rejects any .syntax().text().contains(...) or regex fallback.";
        attribute ac_02_contract_b_stage_2_dynamic_dylint_mir_analysis_integration : String = "- **Given**: A workspace crate targeted for semantic audit where Stage-2 MIR rules are enabled. - **When**: DylintMirBridge::run_stage2_mir_analysis executes against the out-of-tree dynamic driver in ./lints/deap-dylint-mir/ resolved via camino::Utf8Path and std::env::consts. - **Then**: The bridge receives structured NDJSON receipts capturing rustc_middle::mir::Body and rustc_middle::ty::TyCtxt borrow and dataflow diagnostics across all 5 pillars without adding #![feature(rustc_private)] to core crates with diagnostic code E0602.";
        attribute ac_03_isolated_dart_cst_extraction : String = "- **Given**: A Flutter or Dart source file with unclosed stream controllers or asynchronous setState() calls. - **When**: DartCstExtractionEngine::audit_dart_cst parses the source text using logos lexical tokens and rowan lossless syntax trees. - **Then**: The isolated < 380-SLOC deap_core::dart_cst engine emits line-exact AuditFinding records without contaminating core Rust AST data models with diagnostic code E0603.";
        attribute ac_04_fail_closed_deterministic_seven_section_dossier_synthesis_via_pulldown_cmark_to_cmark : String = "- **Given**: An AuditFinding emitted by CST or MIR audit stages with Critical or Important severity, or a malformed candidate missing required headers or containing Mermaid semicolons. - **When**: synthesize_defect_dossier_markdown(&finding) constructs the .pipeline/defects/DEFECT-<SLUG>.md artifact and validates it through validate_defect_dossier. - **Then**: Valid CommonMark AST output populates all 7 canonical sections and ## Audit Source with Check 13B-compliant Mermaid diagrams (dossier_schema_check_count == 12), whereas any malformed dossier fails closed and is rejected with a non-zero.";
        attribute ac_05_idempotent_local_and_remote_ssot_deduplication : String = "- **Given**: A workspace where .pipeline/defects/DEFECT-<SLUG>.md and its corresponding GitHub issue already exist for a specific (FILE_LOCATION, AuditPillar) tuple. - **When**: ./target/release/verify-baseline audit-code --path <PATH> --publish is re-run on unchanged source trees. - **Then**: The auditor detects existing artifacts, skips duplicate file or issue creation, and exits cleanly with code 0 and diagnostic code E0605.";
        attribute ac_06_closed_loop_github_ssot_publishing_and_byte_parity_verification : String = "- **Given**: A newly synthesized .pipeline/defects/DEFECT-<SLUG>.md dossier and --publish flag. - **When**: ClosedLoopDefectPublisherGate invokes defect_dossier::run_check_defect. - **Then**: The verifier creates or edits the issue via gh issue, queries gh issue view <ID> --json title,body,url, verifies that closed_loop_mismatch_bytes == 0 after CRLF normalization, fails closed on any remote byte mismatch, and prints [CLOSED-LOOP VERIFIED] with diagnostic code E0606.";
        attribute ac_07_contract_a_contract_b_net_negative_source_ast_rs_deletion_and_380_sloc_submodule_gate_conformance : String = "- **Given**: This blueprint specification at docs/designs/blueprint-06-deterministic-static-defect-dossier-auditor.md. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-06-deterministic-static-defect-dossier-auditor.md --dry-run is executed. - **Then**: All 12 blueprint verification gates pass, verifying complete deletion of crates/deap-core/src/source_ast.rs and hand-rolled scanners (legacy_scanner_sloc_target == -2399), modular < 380-SLOC submodule layout (max_module_exec_sloc <= 380), Contract A and Contract B compliance, and bijective roundtrip SysML v2.";
        doc /* \begin{aligned} \text{AuditorSoundness} &\iff (N_{\text{subagents}} = 0) \land (N_{\text{checks}} = 12) \land (N_{\text{pillars}} = 5) \land (\Delta_{\text{SSOT}} = 0) \land (\Delta_{\text{legacy}} = -2399) \\ \text{GatePass}(D) &\iff \prod_{m=1}^{12} C_m(D) = 1 \end{aligned} */
        constraint def Invariant_REQ_0606_SC_01;
        doc /* \begin{aligned} T_{\text{audit}}(N, V, K) &= O(N + V + K) \\ S_{\text{audit}}(N, V, K) &= O(N + V + K) \\ \Delta_{\text{SSOT}}(D_{\text{local}}, D_{\text{remote}}) &= \lVert \text{norm}_{\text{LF}}(D_{\text{local}}) - \text{norm}_{\text{LF}}(D_{\text{remote}}) \rVert_{1} = 0 \end{aligned} */
        constraint def Invariant_REQ_0606_SC_02;
        require Invariant_REQ_0606_SC_01;
        require Invariant_REQ_0606_SC_02;
        verify by AC_01_Contract_A_Static_Five_Pillar_Typed_AST_Defect_Detection;
        verify by AC_02_Contract_B_Stage_2_Dynamic_Dylint_MIR_Analysis_Integration;
        verify by AC_03_Isolated_Dart_CST_Extraction;
        verify by AC_04_Fail_Closed_Deterministic_Seven_Section_Dossier_Synthesis_via_pulldown_cmark_to_cmark;
        verify by AC_05_Idempotent_Local_and_Remote_SSOT_Deduplication;
        verify by AC_06_Closed_Loop_GitHub_SSOT_Publishing_and_Byte_Parity_Verification;
        verify by AC_07_Contract_A_Contract_B_Net_Negative_source_ast_rs_Deletion_and_380_SLOC_Submodule_Gate_Conformance;
        satisfy by ClosedLoopDefectPublisherGate;
    }
    doc /* Extracted from Mermaid stateDiagram specification. */
    state def BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint_LifecycleState {
        state Idle;
        state IngestingSourceAst;
        state ParsingRustCst;
        state ParsingDartCst;
        state EvaluatingMirAnalysis;
        state SynthesizingDossier;
        state Verified;
        state Validating12Gates;
        state Rejected;
        state DeduplicatingSsot;
        state PublishingClosedLoop;
        transition Initial_to_Idle first Initial then Idle;
        transition Idle_to_IngestingSourceAst_on_verify_baseline_audit_code_path first Idle accept verify_baseline_audit_code_path then IngestingSourceAst;
        transition IngestingSourceAst_to_ParsingRustCst_on_Rust_source_detected first IngestingSourceAst accept Rust_source_detected then ParsingRustCst;
        transition IngestingSourceAst_to_ParsingDartCst_on_Dart_source_detected first IngestingSourceAst accept Dart_source_detected then ParsingDartCst;
        transition ParsingRustCst_to_EvaluatingMirAnalysis_on_typed_AST_parsed_via_ra_ap_syntax first ParsingRustCst accept typed_AST_parsed_via_ra_ap_syntax then EvaluatingMirAnalysis;
        transition EvaluatingMirAnalysis_to_SynthesizingDossier_on_AuditFinding_emitted first EvaluatingMirAnalysis accept AuditFinding_emitted then SynthesizingDossier;
        transition ParsingDartCst_to_SynthesizingDossier_on_isolated_logos_and_rowan_finding first ParsingDartCst accept isolated_logos_and_rowan_finding then SynthesizingDossier;
        transition EvaluatingMirAnalysis_to_Verified_on_zero_findings_detected first EvaluatingMirAnalysis accept zero_findings_detected then Verified;
        transition SynthesizingDossier_to_Validating12Gates_on_synthesize_defect_dossier_markdown first SynthesizingDossier accept synthesize_defect_dossier_markdown then Validating12Gates;
        transition Validating12Gates_to_Rejected_on_schema_or_Mermaid_check_violation first Validating12Gates accept schema_or_Mermaid_check_violation then Rejected;
        transition Validating12Gates_to_DeduplicatingSsot_on_all_12_checks_passed first Validating12Gates accept all_12_checks_passed then DeduplicatingSsot;
        transition DeduplicatingSsot_to_Verified_on_existing_finding_skipped first DeduplicatingSsot accept existing_finding_skipped then Verified;
        transition DeduplicatingSsot_to_PublishingClosedLoop_on_new_finding_persisted_to_pipeline_defects first DeduplicatingSsot accept new_finding_persisted_to_pipeline_defects then PublishingClosedLoop;
        transition PublishingClosedLoop_to_Verified_on_closed_loop_mismatch_bytes_0 first PublishingClosedLoop accept closed_loop_mismatch_bytes_0 then Verified;
        transition PublishingClosedLoop_to_Rejected_on_remote_payload_mismatch first PublishingClosedLoop accept remote_payload_mismatch then Rejected;
        transition Rejected_to_Terminal first Rejected then Terminal;
        transition Verified_to_Terminal first Verified then Terminal;
    }
    doc /* - **Given**: A target Rust source file containing representative defects across the 5 pillars (unchecked as casts in Memory Safety E0601, unbounded cache mutations in Resource Lifecycle E0602, TOCTOU file checks in Concurrency E0603, 0-assertion tests in Test Integrity E0604, or missing Realises: tags in Semantic Traceability E0605). - **When**: ./target/release/verify-baseline audit-code --path <PATH> --dry-run is executed. - **Then**: CstPillarAuditor deterministically parses the syntax tree using ra_ap_syntax under Contract A, rejects any .syntax().text().contains(...) or regex fallback. */
    use case def UC_AC_01_Contract_A_Static_Five_Pillar_Typed_AST_Defect_Detection {
        subject ClosedLoopDefectPublisherGate;
        objective "Contract A Static Five Pillar Typed AST Defect Detection";
    }
    doc /* - **Given**: A workspace crate targeted for semantic audit where Stage-2 MIR rules are enabled. - **When**: DylintMirBridge::run_stage2_mir_analysis executes against the out-of-tree dynamic driver in ./lints/deap-dylint-mir/ resolved via camino::Utf8Path and std::env::consts. - **Then**: The bridge receives structured NDJSON receipts capturing rustc_middle::mir::Body and rustc_middle::ty::TyCtxt borrow and dataflow diagnostics across all 5 pillars without adding #![feature(rustc_private)] to core crates with diagnostic code E0602. */
    use case def UC_AC_02_Contract_B_Stage_2_Dynamic_Dylint_MIR_Analysis_Integration {
        subject ClosedLoopDefectPublisherGate;
        objective "Contract B Stage 2 Dynamic Dylint MIR Analysis Integration";
    }
    doc /* - **Given**: A Flutter or Dart source file with unclosed stream controllers or asynchronous setState() calls. - **When**: DartCstExtractionEngine::audit_dart_cst parses the source text using logos lexical tokens and rowan lossless syntax trees. - **Then**: The isolated < 380-SLOC deap_core::dart_cst engine emits line-exact AuditFinding records without contaminating core Rust AST data models with diagnostic code E0603. */
    use case def UC_AC_03_Isolated_Dart_CST_Extraction {
        subject ClosedLoopDefectPublisherGate;
        objective "Isolated Dart CST Extraction";
    }
    doc /* - **Given**: An AuditFinding emitted by CST or MIR audit stages with Critical or Important severity, or a malformed candidate missing required headers or containing Mermaid semicolons. - **When**: synthesize_defect_dossier_markdown(&finding) constructs the .pipeline/defects/DEFECT-<SLUG>.md artifact and validates it through validate_defect_dossier. - **Then**: Valid CommonMark AST output populates all 7 canonical sections and ## Audit Source with Check 13B-compliant Mermaid diagrams (dossier_schema_check_count == 12), whereas any malformed dossier fails closed and is rejected with a non-zero. */
    use case def UC_AC_04_Fail_Closed_Deterministic_Seven_Section_Dossier_Synthesis_via_pulldown_cmark_to_cmark {
        subject ClosedLoopDefectPublisherGate;
        objective "Fail Closed Deterministic Seven Section Dossier Synthesis via pulldown cmark to cmark";
    }
    doc /* - **Given**: A workspace where .pipeline/defects/DEFECT-<SLUG>.md and its corresponding GitHub issue already exist for a specific (FILE_LOCATION, AuditPillar) tuple. - **When**: ./target/release/verify-baseline audit-code --path <PATH> --publish is re-run on unchanged source trees. - **Then**: The auditor detects existing artifacts, skips duplicate file or issue creation, and exits cleanly with code 0 and diagnostic code E0605. */
    use case def UC_AC_05_Idempotent_Local_and_Remote_SSOT_Deduplication {
        subject ClosedLoopDefectPublisherGate;
        objective "Idempotent Local and Remote SSOT Deduplication";
    }
    doc /* - **Given**: A newly synthesized .pipeline/defects/DEFECT-<SLUG>.md dossier and --publish flag. - **When**: ClosedLoopDefectPublisherGate invokes defect_dossier::run_check_defect. - **Then**: The verifier creates or edits the issue via gh issue, queries gh issue view <ID> --json title,body,url, verifies that closed_loop_mismatch_bytes == 0 after CRLF normalization, fails closed on any remote byte mismatch, and prints [CLOSED-LOOP VERIFIED] with diagnostic code E0606. */
    use case def UC_AC_06_Closed_Loop_GitHub_SSOT_Publishing_and_Byte_Parity_Verification {
        subject ClosedLoopDefectPublisherGate;
        objective "Closed Loop GitHub SSOT Publishing and Byte Parity Verification";
    }
    doc /* - **Given**: This blueprint specification at docs/designs/blueprint-06-deterministic-static-defect-dossier-auditor.md. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-06-deterministic-static-defect-dossier-auditor.md --dry-run is executed. - **Then**: All 12 blueprint verification gates pass, verifying complete deletion of crates/deap-core/src/source_ast.rs and hand-rolled scanners (legacy_scanner_sloc_target == -2399), modular < 380-SLOC submodule layout (max_module_exec_sloc <= 380), Contract A and Contract B compliance, and bijective roundtrip SysML v2. */
    use case def UC_AC_07_Contract_A_Contract_B_Net_Negative_source_ast_rs_Deletion_and_380_SLOC_Submodule_Gate_Conformance {
        subject ClosedLoopDefectPublisherGate;
        objective "Contract A Contract B Net Negative source ast rs Deletion and 380 SLOC Submodule Gate Conformance";
    }
    doc /* Extracted from Mermaid sequenceDiagram specification. */
    interaction def BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint_SequenceInteraction {
        lifeline User : Operator_Or_CI_Pipeline;
        lifeline CLI : DeterministicStaticDefectAuditor;
        lifeline CST : CstPillarAuditor;
        lifeline MIR : DylintMirBridge;
        lifeline Dart : DartCstExtractionEngine;
        lifeline Emitter : DossierEmitter;
        lifeline Gate : ClosedLoopDefectPublisherGate;
        lifeline GH : GitHub_SSOT_Tracker;
        message run_audit_code from User to CLI;
        message audit_rust_cst from CLI to CST;
        message reply_Ok from CST to CLI;
        message run_stage2_mir_analysis from CLI to MIR;
        message reply_Ok from MIR to CLI;
        message audit_dart_cst from CLI to Dart;
        message reply_Ok from Dart to CLI;
        message synthesize_defect_dossier_markdown from CLI to Emitter;
        message reply_Ok from Emitter to CLI;
        message run_check_defect from CLI to Gate;
        message GhIssueCreateOrGhIssue from Gate to GH;
        message reply_CreatedOrUpdatedGitHubIssueUrl from GH to Gate;
        message GhIssueViewJsonTitleBody from Gate to GH;
        message reply_LiveRemotePayloadForByteFor from GH to Gate;
        message reply_Ok from Gate to CLI;
        message reply_Ok from CLI to User;
    }
    doc /* Enforces all 12 validate_defect_dossier checks prior to persistence or publication. [unit: checks] */
    part def ClosedLoopDefectPublisherGate {
        attribute schemaGateCount : Integer = 12;
        doc /* Requires zero byte discrepancy between local .pipeline/defects/ file and remote GitHub issue body. [unit: flag] */
        attribute requireByteParity : Boolean = true;
        doc /* Receives synthesized markdown payload for in-process 12-check verification. [protocol: CommonMarkText] */
        port in dossierCandidateInPort : String {
            flow in dossierCandidateInPort_flow : String;
        }
        doc /* Emits verified GitHub SSOT issue URL after gh issue view closed-loop payload verification. [protocol: GhCliJson] */
        port out verifiedReceiptOutPort : String {
            flow out verifiedReceiptOutPort_flow : String;
        }
    }
    doc /* Count of active static AST evaluators (E0601 through E0605). [unit: pillars] */
    part def CstPillarAuditor {
        attribute activePillarCount : Integer = 5;
        doc /* Enforces typed ra_ap_syntax CST traversal without regular expression crates. [unit: flag] */
        attribute zeroRegexMode : Boolean = true;
        doc /* Maximum executable SLOC permitted per function body. [unit: SLOC] */
        attribute maxFnExecSloc : Integer = 45;
        doc /* Maximum permitted formal parameter count per callable. [unit: params] */
        attribute maxFnParamCount : Integer = 5;
        doc /* Receives Rust source text and camino::Utf8Path for typed CST construction. [protocol: Utf8SourceBuffer] */
        port in rustSourceInPort : String {
            flow in rustSourceInPort_flow : String;
        }
        doc /* Emits line-exact AuditFinding records with diagnostic codes E0601 through E0605. [protocol: AuditFindingVec] */
        port out cstFindingOutPort : String {
            flow out cstFindingOutPort_flow : String;
        }
    }
    doc /* Hard upper bound on executable lines of code in deap_core::dart_cst. [unit: SLOC] */
    part def DartCstExtractionEngine {
        attribute maxDartModuleSloc : Integer = 380;
        doc /* Target Dart language grammar level parsed by logos + rowan. [unit: version] */
        attribute grammarVersion : Integer = 3;
        doc /* Strictly isolates Dart grammar tokens from core Rust CST AST structures. [unit: flag] */
        attribute isolateSubmoduleMode : Boolean = true;
        doc /* Receives Dart source file text for lexical tokenization and CST tree construction. [protocol: Utf8DartBuffer] */
        port in dartSourceInPort : String {
            flow in dartSourceInPort_flow : String;
        }
        doc /* Emits line-exact Dart audit findings for lifecycle and concurrency pillars. [protocol: AuditFindingVec] */
        port out dartFindingOutPort : String {
            flow out dartFindingOutPort_flow : String;
        }
    }
    doc /* Number of LLM subagents dispatched during static code auditing and defect dossier generation. [unit: count] */
    part def DeterministicStaticDefectAuditor {
        attribute auditor_subagent_count : Integer = 0;
        doc /* Number of deterministic structural, lexical, and Mermaid validation checks enforced per synthesized dossier. [unit: checks] */
        attribute dossier_schema_check_count : Integer = 12;
        doc /* Number of adversarial correctness risk pillars evaluated per source AST scan. [unit: pillars] */
        attribute pillar_coverage_count : Integer = 5;
        doc /* Byte discrepancy between local .pipeline/defects/ dossier and live GitHub issue body after CRLF normalization. [unit: B] */
        attribute closed_loop_mismatch_bytes : Integer = 0;
        doc /* Receives target Rust and Dart source files for static inspection. [protocol: RustDartSource] */
        port in targetSourceInPort : String {
            flow in targetSourceInPort_flow : String;
        }
        doc /* Receives existing .pipeline/defects/ *.md records and live GitHub issue metadata for deduplication. [protocol: GhCliJson] */
        port in gitHubIssueQueryPort : String {
            flow in gitHubIssueQueryPort_flow : String;
        }
        doc /* Persists validated 7-section .pipeline/defects/DEFECT-<SLUG>.md dossiers to repository storage. [protocol: AtomicFsWrite] */
        port out defectDossierOutPort : String {
            flow out defectDossierOutPort_flow : String;
        }
        doc /* Publishes or updates GitHub SSOT issues and verifies byte parity via gh issue view. [protocol: GhCliSubprocess] */
        port out closedLoopPublishPort : String {
            flow out closedLoopPublishPort_flow : String;
        }
    }
    doc /* Enforces all 7 canonical DossierSection headers plus the ## Audit Source footer. [unit: sections] */
    part def DossierEmitter {
        attribute mandatorySectionCount : Integer = 7;
        doc /* Enforces 5 numbered causal Why ...? Because ... root-cause steps in Section 2. [unit: steps] */
        attribute whysChainDepth : Integer = 5;
        doc /* Guarantees zero semicolons and quoted special tokens inside Section 4 Mermaid blocks. [unit: flag] */
        attribute forbidSemicolons : Boolean = true;
        doc /* Enforces Rules E1 through E5 across synthesized sequenceDiagram and classDiagram blocks. [unit: flag] */
        attribute check13bCompliant : Boolean = true;
        doc /* Receives a typed AuditFinding struct for CommonMark serialization. [protocol: AuditFindingRef] */
        port in findingInPort : String {
            flow in findingInPort_flow : String;
        }
        doc /* Emits a 7-section defect dossier payload serialized via pulldown-cmark-to-cmark. [protocol: CommonMarkText] */
        port out dossierMarkdownOutPort : String {
            flow out dossierMarkdownOutPort_flow : String;
        }
    }
    doc /* Controls Stage-2 cargo-dylint MIR driver execution. [unit: flag] */
    part def DylintMirBridge {
        attribute mirAnalysisEnabled : Boolean = true;
        doc /* Millisecond timeout for Stage-2 dynamic driver execution. [unit: ms] */
        attribute driverExitTimeoutMs : Integer = 10000;
        doc /* Canonical schema version for NDJSON diagnostic receipt streaming. [unit: semver] */
        attribute ndjsonReceiptSchemaVersion : Integer = 1;
        doc /* Receives workspace crate path for out-of-tree Stage-2 MIR execution. [protocol: WorkspaceCratePath] */
        port in crateTargetInPort : String {
            flow in crateTargetInPort_flow : String;
        }
        doc /* Emits structured MIR-level findings parsed from dynamic driver NDJSON output. [protocol: NdjsonFindingStream] */
        port out mirFindingOutPort : String {
            flow out mirFindingOutPort_flow : String;
        }
    }
}