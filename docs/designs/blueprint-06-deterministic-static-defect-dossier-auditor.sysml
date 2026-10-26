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
    doc /* `deap_core::source_ast` recursive-descent AST & token stream */
    attribute Source_Parsing_Engine : String = "LLM token window reading raw text";
    doc /* Deterministic 5-pillar static AST & control-flow analyzers */
    attribute Pillar_Rule_Evaluation : String = "Heuristic prompt interpretation";
    doc /* `synthesize_defect_dossier_markdown(&AuditFinding)` builder */
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
    doc /* verify-baseline audit-code --path crates/ --publish */
    action def verify_baseline;
    doc /* parse_rust_source_ast and parse_dart_source_ast */
    action def parse_rust_source_ast;
    doc /* Deterministic Vec of SourceAstItem nodes */
    action def Deterministic;
    doc /* evaluate_source_file_pillars across 5 pillars */
    action def evaluate_source_file_pillars;
    doc /* Line-exact AuditFinding records */
    action def Line_exact;
    doc /* synthesize_defect_dossier_markdown(finding) */
    action def synthesize_defect_dossier_markdown;
    doc /* Compliant 7-section DEFECT-SLUG.md payload */
    action def Compliant;
    doc /* run_check_defect(DefectDossierArgs) */
    action def run_check_defect;
    doc /* Execute 12-check validate_defect_dossier gate */
    action def Execute;
    doc /* gh issue create or gh issue edit with body-file */
    action def gh;
    doc /* Created or updated GitHub Issue URL */
    action def Created;
    doc /* Live remote payload for byte-for-byte comparison */
    action def Live;
    doc /* CLOSED-LOOP VERIFIED receipt (0 mismatch bytes) */
    action def CLOSED_LOOP;
    doc /* Exit code 0 with verified issue URLs */
    action def Exit_item;
    doc /* [unit: `count`] */
    assert constraint assert_auditor_subagent_count_range {
        auditor_subagent_count >= 0 and auditor_subagent_count <= 0;
    }
    doc /* [unit: `checks`] */
    assert constraint assert_dossier_schema_check_count_range {
        dossier_schema_check_count >= 12 and dossier_schema_check_count <= 12;
    }
    doc /* [unit: `pillars`] */
    assert constraint assert_pillar_coverage_count_range {
        pillar_coverage_count >= 5 and pillar_coverage_count <= 5;
    }
    doc /* [unit: `B`] */
    assert constraint assert_closed_loop_mismatch_bytes_range {
        closed_loop_mismatch_bytes >= 0 and closed_loop_mismatch_bytes <= 0;
    }
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0606_BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint {
        id = "REQ-0606";
        text = "The DeterministicStaticDefectAuditor subsystem (**REQ-0606** / BP-06 / DEAP-BP-DET-06, deterministic UUIDv5 6ba7b810-9dad-11d1-80b4-00c04fd430c6, complexity class Class P linear single-pass token-stream and AST traversal, governed by DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_static_five_pillar_ast_defect_detection : String = "Diagnostic: E0601).";
        attribute ac_02_deterministic_seven_section_dossier_and_mermaid_synthesis : String = "Diagnostic: E0602).";
        attribute ac_03_idempotent_local_and_remote_ssot_deduplication : String = "Diagnostic: E0603).";
        attribute ac_04_closed_loop_github_ssot_publishing_and_byte_parity_verification : String = "Diagnostic: E0604).";
        attribute ac_05_fail_closed_rejection_on_malformed_dossier_schema_ephemeral_paths_or_payload_divergence : String = "Diagnostic: E0605).";
        attribute ac_06_dogfood_sysml_v2_markdown_ingestion_and_blueprint_gate_conformance : String = "- **Given**: This blueprint specification at docs/designs/blueprint-06-deterministic-static-defect-dossier-auditor.md. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-06-deterministic-static-defect-dossier-auditor.md --dry-run is executed. - **Then**: All 12 blueprint verification gates pass, DeterministicStaticDefectAuditor and its component metamodel are lowered into docs/designs/blueprint-06-deterministic-static-defect-dossier-auditor.sysml and .digest.json with bijective roundtrip parity, and the command exits with code 0.";
        doc /* \begin{aligned} \text{AuditorSoundness} &\iff (N_{\text{subagents}} = 0) \land (N_{\text{checks}} = 12) \land (N_{\text{pillars}} = 5) \land (\Delta_{\text{SSOT}} = 0) \\ \text{GatePass}(D) &\iff \prod_{m=1}^{12} C_m(D) = 1 \end{aligned} */
        constraint def Invariant_REQ_0606_SC_01;
        doc /* \begin{aligned} T_{\text{audit}}(N, V, K) &= O(N + V + K) \\ S_{\text{audit}}(N, V, K) &= O(N + V + K) \\ \Delta_{\text{SSOT}}(D_{\text{local}}, D_{\text{remote}}) &= \lVert \text{norm}_{\text{LF}}(D_{\text{local}}) - \text{norm}_{\text{LF}}(D_{\text{remote}}) \rVert_{1} = 0 \end{aligned} */
        constraint def Invariant_REQ_0606_SC_02;
        require Invariant_REQ_0606_SC_01;
        require Invariant_REQ_0606_SC_02;
        verify by AC_01_Static_Five_Pillar_AST_Defect_Detection;
        verify by AC_02_Deterministic_Seven_Section_Dossier_and_Mermaid_Synthesis;
        verify by AC_03_Idempotent_Local_and_Remote_SSOT_Deduplication;
        verify by AC_04_Closed_Loop_GitHub_SSOT_Publishing_and_Byte_Parity_Verification;
        verify by AC_05_Fail_Closed_Rejection_on_Malformed_Dossier_Schema_Ephemeral_Paths_or_Payload_Divergence;
        verify by AC_06_Dogfood_SysML_v2_Markdown_Ingestion_and_Blueprint_Gate_Conformance;
        satisfy by ClosedLoopDefectPublisherGate;
    }
    doc /* Extracted from Mermaid stateDiagram specification. */
    state def BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint_LifecycleState {
        transition Initial_to_Idle;
        transition Idle_to_IngestingSourceAst_on_verify_baseline_audit_code_path;
        transition IngestingSourceAst_to_EvaluatingFivePillars_on_SourceAstItem_slice_parsed;
        transition EvaluatingFivePillars_to_SynthesizingDossier_on_AuditFinding_emitted;
        transition EvaluatingFivePillars_to_Verified_on_zero_findings_detected;
        transition SynthesizingDossier_to_Validating12Gates_on_synthesize_defect_dossier_markdown;
        transition Validating12Gates_to_Rejected_on_schema_or_Mermaid_check_violation;
        transition Validating12Gates_to_DeduplicatingSsot_on_all_12_checks_passed;
        transition DeduplicatingSsot_to_Verified_on_existing_finding_skipped;
        transition DeduplicatingSsot_to_PublishingClosedLoop_on_new_finding_persisted_to_pipeline_defects;
        transition PublishingClosedLoop_to_Verified_on_closed_loop_mismatch_bytes_0;
        transition PublishingClosedLoop_to_Rejected_on_remote_payload_mismatch;
        transition Rejected_to_Terminal;
        transition Verified_to_Terminal;
    }
    doc /* Diagnostic: E0601). */
    use case def UC_AC_01_Static_Five_Pillar_AST_Defect_Detection {
        subject ClosedLoopDefectPublisherGate;
        objective "Static Five Pillar AST Defect Detection";
    }
    doc /* Diagnostic: E0602). */
    use case def UC_AC_02_Deterministic_Seven_Section_Dossier_and_Mermaid_Synthesis {
        subject ClosedLoopDefectPublisherGate;
        objective "Deterministic Seven Section Dossier and Mermaid Synthesis";
    }
    doc /* Diagnostic: E0603). */
    use case def UC_AC_03_Idempotent_Local_and_Remote_SSOT_Deduplication {
        subject ClosedLoopDefectPublisherGate;
        objective "Idempotent Local and Remote SSOT Deduplication";
    }
    doc /* Diagnostic: E0604). */
    use case def UC_AC_04_Closed_Loop_GitHub_SSOT_Publishing_and_Byte_Parity_Verification {
        subject ClosedLoopDefectPublisherGate;
        objective "Closed Loop GitHub SSOT Publishing and Byte Parity Verification";
    }
    doc /* Diagnostic: E0605). */
    use case def UC_AC_05_Fail_Closed_Rejection_on_Malformed_Dossier_Schema_Ephemeral_Paths_or_Payload_Divergence {
        subject ClosedLoopDefectPublisherGate;
        objective "Fail Closed Rejection on Malformed Dossier Schema Ephemeral Paths or Payload Divergence";
    }
    doc /* - **Given**: This blueprint specification at docs/designs/blueprint-06-deterministic-static-defect-dossier-auditor.md. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-06-deterministic-static-defect-dossier-auditor.md --dry-run is executed. - **Then**: All 12 blueprint verification gates pass, DeterministicStaticDefectAuditor and its component metamodel are lowered into docs/designs/blueprint-06-deterministic-static-defect-dossier-auditor.sysml and .digest.json with bijective roundtrip parity, and the command exits with code 0. */
    use case def UC_AC_06_Dogfood_SysML_v2_Markdown_Ingestion_and_Blueprint_Gate_Conformance {
        subject ClosedLoopDefectPublisherGate;
        objective "Dogfood SysML v2 Markdown Ingestion and Blueprint Gate Conformance";
    }
    doc /* Extracted from Mermaid sequenceDiagram specification. */
    interaction def BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint_SequenceInteraction {
        lifeline User : Operator_Or_CI_Pipeline;
        lifeline CLI : verify_baseline_audit_code;
        lifeline AST : deap_core_source_ast;
        lifeline Engine : FivePillarStaticAstAnalyzer;
        lifeline Synth : DeterministicDossierSynthesizer;
        lifeline Gate : defect_dossier_run_check_defect;
        lifeline GH : GitHub_SSOT_Tracker;
        message verify_baseline from User to CLI;
        message parse_rust_source_ast from CLI to AST;
        message Deterministic from AST to CLI;
        message evaluate_source_file_pillars from CLI to Engine;
        message Line_exact from Engine to CLI;
        message synthesize_defect_dossier_markdown from CLI to Synth;
        message Compliant from Synth to CLI;
        message run_check_defect from CLI to Gate;
        message Execute from Gate to Gate;
        message gh from Gate to GH;
        message Created from GH to Gate;
        message Live from GH to Gate;
        message CLOSED_LOOP from Gate to CLI;
        message Exit_item from CLI to User;
    }
    doc /* Enforces all 12 `validate_defect_dossier` checks prior to persistence or publication. [unit: checks] */
    part def ClosedLoopDefectPublisherGate {
        attribute schemaGateCount : Integer = 12;
        doc /* Requires zero byte discrepancy between local `.pipeline/defects/` file and remote GitHub issue body. [unit: flag] */
        attribute requireByteParity : Boolean = true;
        doc /* Receives synthesized markdown payload for in-process 12-check verification. [protocol: CommonMarkText] */
        port inout dossierCandidateInPort : String;
        doc /* Emits verified GitHub SSOT issue URL after `gh issue view` closed-loop payload verification. [protocol: GhCliJson] */
        port inout verifiedReceiptOutPort : String;
    }
    doc /* Enforces all 7 canonical `DossierSection` headers plus the `## Audit Source` footer. [unit: sections] */
    part def DeterministicDossierSynthesizer {
        attribute mandatorySectionCount : Integer = 7;
        doc /* Enforces 5 numbered causal `Why ...? Because ...` root-cause steps in Section 2. [unit: steps] */
        attribute whysChainDepth : Integer = 5;
        doc /* Receives a deterministic `AuditFinding` struct for markdown serialization. [protocol: AuditFindingRef] */
        port inout findingInPort : String;
        doc /* Emits a 7-section defect dossier payload starting with `## 1. Context and References`. [protocol: CommonMarkText] */
        port inout dossierMarkdownOutPort : String;
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
        doc /* Receives parsed Rust (`.rs`) and Dart (`.dart`) `SourceAstItem` declarations and token spans from `deap_core::source_ast`. [protocol: RustSliceAst] */
        port inout TargetSourceAstInputPort : String;
        doc /* Receives existing `.pipeline/defects/ *.md` records and live GitHub issue metadata for idempotent finding deduplication. [protocol: GhCliJson] */
        port inout GitHubIssueSsotQueryPort : String;
        doc /* Persists validated 7-section `.pipeline/defects/DEFECT-.md` dossiers to the repository defect registry. [protocol: AtomicFsWrite] */
        port inout DefectDossierArtifactPort : String;
        doc /* Publishes or updates GitHub SSOT issues via `gh issue create` or `gh issue edit` and verifies byte parity via `gh issue view`. [protocol: GhCliSubprocess] */
        port inout ClosedLoopTrackerPublishPort : String;
    }
    doc /* Count of active static AST evaluators (`E0601` through `E0605`). [unit: pillars] */
    part def FivePillarStaticAstAnalyzer {
        attribute activePillarCount : Integer = 5;
        doc /* Enforces linear token-stream and `SourceAstItem` slice traversal without regular expressions. [unit: flag] */
        attribute zeroRegexMode : Boolean = true;
        doc /* Accepts normalized file paths, source lines, and `SourceAstItem` slices. [protocol: RustSliceAst] */
        port inout astStreamInPort : String;
        doc /* Emits line-exact `AuditFinding` records with diagnostic bindings `E0601` through `E0605`. [protocol: AuditFindingVec] */
        port inout findingStreamOutPort : String;
    }
    doc /* Guarantees zero semicolons and quoted special tokens inside Section 4 Mermaid blocks. [unit: flag] */
    part def MermaidUmlDiagramGenerator {
        attribute forbidSemicolons : Boolean = true;
        doc /* Enforces Rules E1 through E5 across synthesized `sequenceDiagram` and `classDiagram` blocks. [unit: flag] */
        attribute check13bCompliant : Boolean = true;
        doc /* Receives caller/callee symbols, line spans, and violated invariant metadata. [protocol: AuditFindingRef] */
        port inout umlSpecInPort : String;
        doc /* Emits a fenced Mermaid diagram or severity-scoped `N/A --` declaration for Section 4. [protocol: MermaidText] */
        port inout mermaidBlockOutPort : String;
    }
}