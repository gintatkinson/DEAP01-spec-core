doc /* Translated from Level 0 OEM Markdown specification */
package BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint {
    attribute Blueprint_ID : String = "BP-06";
    attribute Requirement_ID : String = "REQ-0606";
    attribute Title : String = "Deterministic Static 5-Pillar Code Auditor and Defect Dossier Synthesizer Blueprint";
    attribute Status : String = "Approved";
    attribute Target_Crates : String = "crates/verify-baseline, crates/deap-core";
    attribute Tracked_Issues : String = "#465, #466, #467, #468";
    attribute Generation_Mode : String = "subagent";
    attribute Specification_Source : String = "crates/verify-baseline/src/defect_dossier.rs:1-2276";
    doc /* `0` subagents (100% native Rust binary execution) */
    attribute Subagent_Dispatches : String = "5 to 10 LLM subagents per audit wave";
    doc /* `ra_ap_syntax` typed AST + `logos`/`rowan` Dart CST */
    attribute Source_Parsing_Engine : String = "LLM token window / hand-rolled line scanners";
    doc /* Stage-2 `cargo-dylint` driver reading `rustc_middle::mir::Body` */
    attribute Semantic_MIR_Analysis : String = "Heuristic prompt interpretation (hallucination-prone)";
    doc /* Structured AST emission via `pulldown-cmark-to-cmark` */
    attribute Dossier_Construction : String = "Manual LLM markdown drafting";
    doc /* Deterministic Check 13B-compliant `sequenceDiagram` / `classDiagram` builder */
    attribute Mermaid_Diagram_Synthesis : String = "Hand-authored Mermaid (error-prone)";
    doc /* In-process `defect_dossier::validate_defect_dossier` (12 checks) */
    attribute Schema_Validation_Gate : String = "External subprocess retry loop";
    doc /* `< 1.0 s` local scan and synthesis (`O(N)` linear complexity) */
    attribute End_to_End_Execution_Time : String = "180 s to 600 s per module wave";
    doc /* [unit: `count`] */
    attribute auditor_subagent_count : Real;
    doc /* [unit: `checks`] */
    attribute dossier_schema_check_count : Real;
    doc /* [unit: `pillars`] */
    attribute pillar_coverage_count : Real;
    doc /* [unit: `B`] */
    attribute closed_loop_mismatch_bytes : Real;
    doc /* [unit: `SLOC`] */
    attribute legacy_scanner_sloc_target : Real;
    doc /* [unit: `SLOC`] */
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
    doc /* [unit: `count`] */
    assert constraint C_AUD_01 {
        auditor_subagent_count >= 0 and auditor_subagent_count <= 0;
    }
    doc /* [unit: `checks`] */
    assert constraint C_AUD_02 {
        dossier_schema_check_count >= 12 and dossier_schema_check_count <= 12;
    }
    doc /* [unit: `pillars`] */
    assert constraint C_AUD_03 {
        pillar_coverage_count >= 5 and pillar_coverage_count <= 5;
    }
    doc /* [unit: `B`] */
    assert constraint C_AUD_04 {
        closed_loop_mismatch_bytes >= 0 and closed_loop_mismatch_bytes <= 0;
    }
    doc /* [unit: `SLOC`] */
    assert constraint C_AUD_05 {
        legacy_scanner_sloc_target >= -2399 and legacy_scanner_sloc_target <= -2399;
    }
    doc /* [unit: `SLOC`] */
    assert constraint C_AUD_06 {
        max_module_exec_sloc >= 0 and max_module_exec_sloc <= 380;
    }
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0606_BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint {
        id = "REQ-0606";
        text = "The DeterministicStaticDefectAuditor subsystem (**REQ-0606** / BP-06 / DEAP-BP-DET-06, deterministic UUIDv5 6ba7b810-9dad-11d1-80b4-00c04fd430c6, complexity class Class P linear single-pass syntax tree and MIR traversal, governed by DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_static_five_pillar_typed_ast_defect_detection : String = "Diagnostic: E0601).";
        attribute ac_02_stage_2_dynamic_dylint_mir_analysis_integration : String = "Diagnostic: E0602).";
        attribute ac_03_isolated_dart_cst_extraction : String = "Diagnostic: E0603).";
        attribute ac_04_deterministic_seven_section_dossier_synthesis_via_pulldown_cmark_to_cmark : String = "Diagnostic: E0604).";
        attribute ac_05_idempotent_local_and_remote_ssot_deduplication : String = "Diagnostic: E0605).";
        attribute ac_06_closed_loop_github_ssot_publishing_and_byte_parity_verification : String = "Diagnostic: E0606).";
        attribute ac_07_net_negative_scanner_retirement_and_dogfood_blueprint_gate_conformance : String = "- **Given**: This blueprint specification at docs/designs/blueprint-06-deterministic-static-defect-dossier-auditor.md. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-06-deterministic-static-defect-dossier-auditor.md --dry-run is executed. - **Then**: All 12 blueprint verification gates pass, verifying complete retirement of hand-rolled scanners (legacy_scanner_sloc_target == -2399), modular submodule layout (max_module_exec_sloc <= 380), and bijective roundtrip SysML v2 metamodel lowering with exit code 0.";
        doc /* \begin{aligned} \text{AuditorSoundness} &\iff (N_{\text{subagents}} = 0) \land (N_{\text{checks}} = 12) \land (N_{\text{pillars}} = 5) \land (\Delta_{\text{SSOT}} = 0) \land (\Delta_{\text{legacy}} = -2399) \\ \text{GatePass}(D) &\iff \prod_{m=1}^{12} C_m(D) = 1 \end{aligned} */
        constraint def Invariant_REQ_0606_SC_01;
        doc /* \begin{aligned} T_{\text{audit}}(N, V, K) &= O(N + V + K) \\ S_{\text{audit}}(N, V, K) &= O(N + V + K) \\ \Delta_{\text{SSOT}}(D_{\text{local}}, D_{\text{remote}}) &= \lVert \text{norm}_{\text{LF}}(D_{\text{local}}) - \text{norm}_{\text{LF}}(D_{\text{remote}}) \rVert_{1} = 0 \end{aligned} */
        constraint def Invariant_REQ_0606_SC_02;
        require Invariant_REQ_0606_SC_01;
        require Invariant_REQ_0606_SC_02;
        verify by AC_01_Static_Five_Pillar_Typed_AST_Defect_Detection;
        verify by AC_02_Stage_2_Dynamic_Dylint_MIR_Analysis_Integration;
        verify by AC_03_Isolated_Dart_CST_Extraction;
        verify by AC_04_Deterministic_Seven_Section_Dossier_Synthesis_via_pulldown_cmark_to_cmark;
        verify by AC_05_Idempotent_Local_and_Remote_SSOT_Deduplication;
        verify by AC_06_Closed_Loop_GitHub_SSOT_Publishing_and_Byte_Parity_Verification;
        verify by AC_07_Net_Negative_Scanner_Retirement_and_Dogfood_Blueprint_Gate_Conformance;
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
    doc /* Diagnostic: E0601). */
    use case def UC_AC_01_Static_Five_Pillar_Typed_AST_Defect_Detection {
        subject ClosedLoopDefectPublisherGate;
        objective "Static Five Pillar Typed AST Defect Detection";
    }
    doc /* Diagnostic: E0602). */
    use case def UC_AC_02_Stage_2_Dynamic_Dylint_MIR_Analysis_Integration {
        subject ClosedLoopDefectPublisherGate;
        objective "Stage 2 Dynamic Dylint MIR Analysis Integration";
    }
    doc /* Diagnostic: E0603). */
    use case def UC_AC_03_Isolated_Dart_CST_Extraction {
        subject ClosedLoopDefectPublisherGate;
        objective "Isolated Dart CST Extraction";
    }
    doc /* Diagnostic: E0604). */
    use case def UC_AC_04_Deterministic_Seven_Section_Dossier_Synthesis_via_pulldown_cmark_to_cmark {
        subject ClosedLoopDefectPublisherGate;
        objective "Deterministic Seven Section Dossier Synthesis via pulldown cmark to cmark";
    }
    doc /* Diagnostic: E0605). */
    use case def UC_AC_05_Idempotent_Local_and_Remote_SSOT_Deduplication {
        subject ClosedLoopDefectPublisherGate;
        objective "Idempotent Local and Remote SSOT Deduplication";
    }
    doc /* Diagnostic: E0606). */
    use case def UC_AC_06_Closed_Loop_GitHub_SSOT_Publishing_and_Byte_Parity_Verification {
        subject ClosedLoopDefectPublisherGate;
        objective "Closed Loop GitHub SSOT Publishing and Byte Parity Verification";
    }
    doc /* - **Given**: This blueprint specification at docs/designs/blueprint-06-deterministic-static-defect-dossier-auditor.md. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-06-deterministic-static-defect-dossier-auditor.md --dry-run is executed. - **Then**: All 12 blueprint verification gates pass, verifying complete retirement of hand-rolled scanners (legacy_scanner_sloc_target == -2399), modular submodule layout (max_module_exec_sloc <= 380), and bijective roundtrip SysML v2 metamodel lowering with exit code 0. */
    use case def UC_AC_07_Net_Negative_Scanner_Retirement_and_Dogfood_Blueprint_Gate_Conformance {
        subject ClosedLoopDefectPublisherGate;
        objective "Net Negative Scanner Retirement and Dogfood Blueprint Gate Conformance";
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
    doc /* Enforces all 12 `validate_defect_dossier` checks prior to persistence or publication. [unit: checks] */
    part def ClosedLoopDefectPublisherGate {
        attribute schemaGateCount : Integer = 12;
        doc /* Requires zero byte discrepancy between local `.pipeline/defects/` file and remote GitHub issue body. [unit: flag] */
        attribute requireByteParity : Boolean = true;
        doc /* Receives synthesized markdown payload for in-process 12-check verification. [protocol: CommonMarkText] */
        port in dossierCandidateInPort : String {
            flow in dossierCandidateInPort_flow : String;
        }
        doc /* Emits verified GitHub SSOT issue URL after `gh issue view` closed-loop payload verification. [protocol: GhCliJson] */
        port out verifiedReceiptOutPort : String {
            flow out verifiedReceiptOutPort_flow : String;
        }
    }
    doc /* Count of active static AST evaluators (`E0601` through `E0605`). [unit: pillars] */
    part def CstPillarAuditor {
        attribute activePillarCount : Integer = 5;
        doc /* Enforces typed `ra_ap_syntax` CST traversal without regular expression crates. [unit: flag] */
        attribute zeroRegexMode : Boolean = true;
        doc /* Maximum executable SLOC permitted per function body. [unit: SLOC] */
        attribute maxFnExecSloc : Integer = 45;
        doc /* Maximum permitted formal parameter count per callable. [unit: params] */
        attribute maxFnParamCount : Integer = 5;
        doc /* Receives Rust source text and `camino::Utf8Path` for typed CST construction. [protocol: Utf8SourceBuffer] */
        port in rustSourceInPort : String {
            flow in rustSourceInPort_flow : String;
        }
        doc /* Emits line-exact `AuditFinding` records with diagnostic codes `E0601` through `E0605`. [protocol: AuditFindingVec] */
        port out cstFindingOutPort : String {
            flow out cstFindingOutPort_flow : String;
        }
    }
    doc /* Hard upper bound on executable lines of code in `deap_core::dart_cst`. [unit: SLOC] */
    part def DartCstExtractionEngine {
        attribute maxDartModuleSloc : Integer = 380;
        doc /* Target Dart language grammar level parsed by `logos` + `rowan`. [unit: version] */
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
        doc /* Byte discrepancy between local `.pipeline/defects/` dossier and live GitHub issue body after CRLF normalization. [unit: B] */
        attribute closed_loop_mismatch_bytes : Integer = 0;
        doc /* Receives target Rust and Dart source files for static inspection. [protocol: RustDartSource] */
        port in targetSourceInPort : String {
            flow in targetSourceInPort_flow : String;
        }
        doc /* Receives existing `.pipeline/defects/ *.md` records and live GitHub issue metadata for deduplication. [protocol: GhCliJson] */
        port in gitHubIssueQueryPort : String {
            flow in gitHubIssueQueryPort_flow : String;
        }
        doc /* Persists validated 7-section `.pipeline/defects/DEFECT-.md` dossiers to repository storage. [protocol: AtomicFsWrite] */
        port out defectDossierOutPort : String {
            flow out defectDossierOutPort_flow : String;
        }
        doc /* Publishes or updates GitHub SSOT issues and verifies byte parity via `gh issue view`. [protocol: GhCliSubprocess] */
        port out closedLoopPublishPort : String {
            flow out closedLoopPublishPort_flow : String;
        }
    }
    doc /* Enforces all 7 canonical `DossierSection` headers plus the `## Audit Source` footer. [unit: sections] */
    part def DossierEmitter {
        attribute mandatorySectionCount : Integer = 7;
        doc /* Enforces 5 numbered causal `Why ...? Because ...` root-cause steps in Section 2. [unit: steps] */
        attribute whysChainDepth : Integer = 5;
        doc /* Guarantees zero semicolons and quoted special tokens inside Section 4 Mermaid blocks. [unit: flag] */
        attribute forbidSemicolons : Boolean = true;
        doc /* Enforces Rules E1 through E5 across synthesized `sequenceDiagram` and `classDiagram` blocks. [unit: flag] */
        attribute check13bCompliant : Boolean = true;
        doc /* Receives a typed `AuditFinding` struct for CommonMark serialization. [protocol: AuditFindingRef] */
        port in findingInPort : String {
            flow in findingInPort_flow : String;
        }
        doc /* Emits a 7-section defect dossier payload serialized via `pulldown-cmark-to-cmark`. [protocol: CommonMarkText] */
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