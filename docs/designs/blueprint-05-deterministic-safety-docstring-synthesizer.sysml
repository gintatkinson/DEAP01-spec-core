doc /* Translated from Level 0 OEM Markdown specification */
package BP_05_Lossless_CST_Safety_Docstring_Synthesizer_and_Check_36_Verification_Blueprint {
    attribute Blueprint_ID : String = "BP-05";
    attribute Requirement_ID : String = "REQ-0605";
    attribute Title : String = "Lossless CST Safety Docstring Synthesizer and Check 36 Verification Blueprint";
    attribute Status : String = "Approved";
    attribute Target_Crates : String = "crates/deap-core, crates/compile-sysml, crates/verify-baseline";
    attribute Tracked_Issues : String = "#498, #500, #501, #508, #510, #742";
    attribute Generation_Mode : String = "subagent";
    attribute Specification_Source : String = "Cargo.toml:1-40 / crates/deap-core/src/lib.rs:1-40 / crates/compile-sysml/src/lib.rs:1-40 / crates/verify-baseline/src/lib.rs:1-40";
    doc /* Net-negative SLOC target for legacy hand-rolled docstring scanner deletion [unit: lines] */
    attribute legacy_scanner_deletion_sloc : Integer = 8200;
    doc /* Maximum permissible executable SLOC ceiling for newly decomposed CST submodules [unit: lines] */
    attribute max_submodule_exec_sloc : Integer = 380;
    doc /* Maximum allowed docstring line count for Tier-B/C non-actuator declarations [unit: lines] */
    attribute max_tier_bc_doc_lines : Integer = 4;
    doc /* Byte diff produced when re-running docstring synthesis on an already synthesized file [unit: bytes] */
    attribute docstring_idempotency_diff_bytes : Integer = 0;
    doc /* Ingests CRLF/LF-normalized UTF-8 `.rs` source files via `camino::Utf8Path` */
    port in SourceCodeFileInputPort : String {
        doc /* Ingests CRLF/LF-normalized UTF-8 `.rs` source files via `camino::Utf8Path` */
        flow in SourceCodeFileInputPort_flow : String;
    }
    doc /* Supplies parsed SysML v2 `RequirementDef` and `ConstraintDef` AST hierarchies */
    port in SysmlSchemaTraceabilityPort : PackageDef {
        doc /* Supplies parsed SysML v2 `RequirementDef` and `ConstraintDef` AST hierarchies */
        flow in SysmlSchemaTraceabilityPort_flow : PackageDef;
    }
    doc /* Emits bit-for-bit idempotent `.rs` files enriched with surgically edited tiered docstrings */
    port out EnrichedSourceOutputPort : String {
        doc /* Emits bit-for-bit idempotent `.rs` files enriched with surgically edited tiered docstrings */
        flow out EnrichedSourceOutputPort_flow : String;
    }
    doc /* Emits deterministic Check 36 completeness, tier compliance, and idempotency verdicts */
    port out Check36VerificationReportPort : Boolean {
        doc /* Emits deterministic Check 36 completeness, tier compliance, and idempotency verdicts */
        flow out Check36VerificationReportPort_flow : Boolean;
    }
    doc /* plan_docstring_surgical_edits(source, path, compact) */
    action def plan_docstring_surgical_edits;
    doc /* extract_item_doc_info(item, line_index) */
    action def extract_item_doc_info;
    doc /* apply_docstring_synthesis(source, plan) */
    action def apply_docstring_synthesis;
    doc /* audit_source_file_docstrings(path, source) */
    action def audit_source_file_docstrings;
    doc /* Run ingest-sysml --schema blueprint-05 --format markdown */
    action def RunIngestSysmlSchemaBlueprint05;
    doc /* [unit: lines] */
    assert constraint C_DOC_01 {
        legacy_scanner_deletion_sloc >= 8000 and legacy_scanner_deletion_sloc <= 12000;
    }
    doc /* [unit: lines] */
    assert constraint C_DOC_02 {
        max_submodule_exec_sloc >= 0 and max_submodule_exec_sloc <= 380;
    }
    doc /* [unit: lines] */
    assert constraint C_DOC_03 {
        max_tier_bc_doc_lines >= 1 and max_tier_bc_doc_lines <= 4;
    }
    doc /* [unit: bytes] */
    assert constraint C_DOC_04 {
        docstring_idempotency_diff_bytes >= 0 and docstring_idempotency_diff_bytes <= 0;
    }
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0605_BP_05_Lossless_CST_Safety_Docstring_Synthesizer_and_Check_36_Verification_Blueprint {
        id = "REQ-0605";
        text = "The Digital Engineering Agent Platform (DEAP) specification and verification toolchain (UnifiedSafetyDocstringSynthesizer, blueprint reference DEAP-BP-DET-05, requirement **REQ-0605**, UUIDv5 6ba7b810-9dad-11d1-80b4-00c04fd430c5, Complexity Class P, diagnostic code bindings **E0100**, **E0304**,";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_lossless_cst_docstring_traversal_via_ra_ap_syntax : String = "- **Given**: Rust source files discovered via ignore::WalkBuilder, camino::Utf8Path, ra_ap_paths::AbsPathBuf, and ra_ap_vfs::Vfs containing /// outer line docstrings, //! inner module docstrings, #[doc = \"...\"] attributes, raw string literals containing doc markers (r#\"/// not a doc\"#), and interior block comments. - **When**: deap_core::rust_cst::doc_edit::extract_item_doc_info parses the source file under **Contract A**. - **Then**: Exact docstring tokens and rowan::TextRange spans are resolved via ast::HasDocComments and ast::HasAttrs without misclassifying string literals or interior.";
        attribute ac_02_deterministic_surgical_byte_span_synthesis_via_ra_ap_text_edit : String = "- **Given**: A Rust source file with undocumented or partially documented ast::Item nodes indexed with ra_ap_line_index::LineIndex. - **When**: compile_sysml::docstring_cst::plan_docstring_surgical_edits generates a DocstringSynthesisPlan and applies it via apply_docstring_synthesis under **Contract A** and **Contract B**. - **Then**: Docstrings are inserted or updated using non-overlapping TextEditBuilder and TextEdit byte spans, subsequent execution yields zero edits (_{idempotent}(S) = 0), and compilation requires zero #![feature(rustc_private)].";
        attribute ac_03_anti_bloat_tier_a_vs_tier_b_c_contract_and_line_ceilings : String = "- **Given**: A codebase containing both explicit [DOC-TIER: A] public safety-actuator entrypoints and internal Tier-B/C helper functions, structs, and type aliases. - **When**: verify_baseline::checks::docstring_tier_rules and docstring_cst_gate audit the declarations under **Contract B**. - **Then**: Multi-section Tier-A docblocks are validated exclusively on items with explicit [DOC-TIER: A] markers, while all other documented items enforce concise <= 4-line Tier-B/C docstrings carrying [REQ-*] and [FLS: fls_*] anchors (failing closed with a non-zero exit code if any Tier-B/C docstring.";
        attribute ac_04_net_negative_legacy_scanner_deletion_and_380_exec_sloc_ceiling : String = "- **Given**: The legacy codebase containing ~8,200+ SLOC of hand-rolled string scanners in compile-sysml (-4,900+ SLOC) and verify-baseline (-3,300+ SLOC). - **When**: The refactored CST architecture is deployed under **Contract A** and **Contract B**. - **Then**: At least 8,000 legacy hand-rolled scanner SLOC are explicitly deleted, and every newly created 3-Layer DoD submodule (doc_edit.rs, docstring_cst.rs, docstring_cst_gate.rs, docstring_tier_rules.rs) strictly complies with the < 380 executable SLOC ceiling (failing closed if any submodule exceeds 380 executable SLOC).";
        attribute ac_05_dogfood_ingestion_and_sysml_v2_compilation_of_req_0605 : String = "- **Given**: This blueprint specification docs/designs/blueprint-05-deterministic-safety-docstring-synthesizer.md citing stable workspace file:line anchors (Cargo.toml, crates/ * /src/lib.rs, crates/ * /src/main.rs). - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-05-deterministic-safety-docstring-synthesizer.md --dry-run and ./target/release/ingest-sysml --schema docs/designs/blueprint-05-deterministic-safety-docstring-synthesizer.md --format markdown are executed. - **Then**: All 12 mechanical verification gates pass (12/12), canonical SysML v2 AST.";
        doc /* \begin{aligned} T_{\text{cst}}(B) &= O(B) \\ T_{\text{edit}}(E) &= O(E \log E) \\ \Delta_{\text{idempotent}}(S) &= \lVert f_{\text{inject}}(f_{\text{inject}}(S, M), M) - f_{\text{inject}}(S, M) \rVert_1 = 0 \end{aligned} */
        constraint def Invariant_REQ_0605_Under_REQ_0605_and_parametric_constraints_C_DOC_01_through_C_DOC_04_lossless_CST_parsing_surgical_text_editing_and_forward_docstring_injection_satisfy_the_following_linear_complexity_and_idempotency_bounds;
        require Invariant_REQ_0605_Under_REQ_0605_and_parametric_constraints_C_DOC_01_through_C_DOC_04_lossless_CST_parsing_surgical_text_editing_and_forward_docstring_injection_satisfy_the_following_linear_complexity_and_idempotency_bounds;
        verify by AC_01_Lossless_CST_Docstring_Traversal_via_ra_ap_syntax;
        verify by AC_02_Deterministic_Surgical_Byte_Span_Synthesis_via_ra_ap_text_edit;
        verify by AC_03_Anti_Bloat_Tier_A_vs_Tier_B_C_Contract_and_Line_Ceilings;
        verify by AC_04_Net_Negative_Legacy_Scanner_Deletion_and_380_Exec_SLOC_Ceiling;
        verify by AC_05_Dogfood_Ingestion_and_SysML_v2_Compilation_of_REQ_0605;
        satisfy by Check36CstCompletenessVerifier;
    }
    doc /* Synthesized deterministic package lifecycle state machine. */
    state def BP_05_Lossless_CST_Safety_Docstring_Synthesizer_and_Check_36_Verification_Blueprint_LifecycleState {
        state Check36CstCompletenessVerifier_State;
        state DocstringTierEnforcementGate_State;
        state LosslessCstDocstringScanner_State;
        transition Initial_to_Check36CstCompletenessVerifier_State first Initial then Check36CstCompletenessVerifier_State;
        transition Check36CstCompletenessVerifier_State_to_DocstringTierEnforcementGate_State_on_plan_docstring_surgical_edits first Check36CstCompletenessVerifier_State accept plan_docstring_surgical_edits then DocstringTierEnforcementGate_State;
        transition DocstringTierEnforcementGate_State_to_LosslessCstDocstringScanner_State_on_extract_item_doc_info first DocstringTierEnforcementGate_State accept extract_item_doc_info then LosslessCstDocstringScanner_State;
        transition LosslessCstDocstringScanner_State_to_Terminal first LosslessCstDocstringScanner_State then Terminal;
    }
    doc /* - **Given**: Rust source files discovered via ignore::WalkBuilder, camino::Utf8Path, ra_ap_paths::AbsPathBuf, and ra_ap_vfs::Vfs containing /// outer line docstrings, //! inner module docstrings, #[doc = \"...\"] attributes, raw string literals containing doc markers (r#\"/// not a doc\"#), and interior block comments. - **When**: deap_core::rust_cst::doc_edit::extract_item_doc_info parses the source file under **Contract A**. - **Then**: Exact docstring tokens and rowan::TextRange spans are resolved via ast::HasDocComments and ast::HasAttrs without misclassifying string literals or interior. */
    use case def UC_AC_01_Lossless_CST_Docstring_Traversal_via_ra_ap_syntax {
        subject Check36CstCompletenessVerifier;
        objective "Lossless CST Docstring Traversal via ra ap syntax";
    }
    doc /* - **Given**: A Rust source file with undocumented or partially documented ast::Item nodes indexed with ra_ap_line_index::LineIndex. - **When**: compile_sysml::docstring_cst::plan_docstring_surgical_edits generates a DocstringSynthesisPlan and applies it via apply_docstring_synthesis under **Contract A** and **Contract B**. - **Then**: Docstrings are inserted or updated using non-overlapping TextEditBuilder and TextEdit byte spans, subsequent execution yields zero edits (_{idempotent}(S) = 0), and compilation requires zero #![feature(rustc_private)]. */
    use case def UC_AC_02_Deterministic_Surgical_Byte_Span_Synthesis_via_ra_ap_text_edit {
        subject Check36CstCompletenessVerifier;
        objective "Deterministic Surgical Byte Span Synthesis via ra ap text edit";
    }
    doc /* - **Given**: A codebase containing both explicit [DOC-TIER: A] public safety-actuator entrypoints and internal Tier-B/C helper functions, structs, and type aliases. - **When**: verify_baseline::checks::docstring_tier_rules and docstring_cst_gate audit the declarations under **Contract B**. - **Then**: Multi-section Tier-A docblocks are validated exclusively on items with explicit [DOC-TIER: A] markers, while all other documented items enforce concise <= 4-line Tier-B/C docstrings carrying [REQ-*] and [FLS: fls_*] anchors (failing closed with a non-zero exit code if any Tier-B/C docstring. */
    use case def UC_AC_03_Anti_Bloat_Tier_A_vs_Tier_B_C_Contract_and_Line_Ceilings {
        subject Check36CstCompletenessVerifier;
        objective "Anti Bloat Tier A vs Tier B C Contract and Line Ceilings";
    }
    doc /* - **Given**: The legacy codebase containing ~8,200+ SLOC of hand-rolled string scanners in compile-sysml (-4,900+ SLOC) and verify-baseline (-3,300+ SLOC). - **When**: The refactored CST architecture is deployed under **Contract A** and **Contract B**. - **Then**: At least 8,000 legacy hand-rolled scanner SLOC are explicitly deleted, and every newly created 3-Layer DoD submodule (doc_edit.rs, docstring_cst.rs, docstring_cst_gate.rs, docstring_tier_rules.rs) strictly complies with the < 380 executable SLOC ceiling (failing closed if any submodule exceeds 380 executable SLOC). */
    use case def UC_AC_04_Net_Negative_Legacy_Scanner_Deletion_and_380_Exec_SLOC_Ceiling {
        subject Check36CstCompletenessVerifier;
        objective "Net Negative Legacy Scanner Deletion and 380 Exec SLOC Ceiling";
    }
    doc /* - **Given**: This blueprint specification docs/designs/blueprint-05-deterministic-safety-docstring-synthesizer.md citing stable workspace file:line anchors (Cargo.toml, crates/ * /src/lib.rs, crates/ * /src/main.rs). - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-05-deterministic-safety-docstring-synthesizer.md --dry-run and ./target/release/ingest-sysml --schema docs/designs/blueprint-05-deterministic-safety-docstring-synthesizer.md --format markdown are executed. - **Then**: All 12 mechanical verification gates pass (12/12), canonical SysML v2 AST. */
    use case def UC_AC_05_Dogfood_Ingestion_and_SysML_v2_Compilation_of_REQ_0605 {
        subject Check36CstCompletenessVerifier;
        objective "Dogfood Ingestion and SysML v2 Compilation of REQ 0605";
    }
    doc /* Extracted from Mermaid sequenceDiagram specification. */
    interaction def BP_05_Lossless_CST_Safety_Docstring_Synthesizer_and_Check_36_Verification_Blueprint_SequenceInteraction {
        lifeline CLI : Compiler_CLI_CI_Pipeline;
        lifeline Core : deap_core_rust_cst_doc_edit;
        lifeline Synth : compile_sysml_docstring_cst;
        lifeline Gate : verify_baseline_checks_docstring_cst_gate;
        lifeline Ingest : ingest_sysml;
        message plan_docstring_surgical_edits from CLI to Synth;
        message extract_item_doc_info from Synth to Core;
        message reply_ReturnDocCommentInfoWithTierSpanAnchors from Core to Synth;
        message apply_docstring_synthesis from Synth to Synth;
        message reply_EmitBitForBitIdempotentSource from Synth to CLI;
        message audit_source_file_docstrings from CLI to Gate;
        message extract_item_doc_info from Gate to Core;
        message reply_ReturnDocCommentInfo from Core to Gate;
        message reply_EmitVecOfDocstringGateViolation from Gate to CLI;
        message RunIngestSysmlSchemaBlueprint05 from CLI to Ingest;
        message reply_EmitCanonicalSysMLV2AstFor from Ingest to CLI;
    }
    doc /* Check 36 gate in `< 380`-SLOC `verify_baseline::checks::docstring_cst_gate` auditing 100% item coverage, legacy scanner retirement, and bit-for-bit idempotency under Contract A and Contract B */
    part def Check36CstCompletenessVerifier {
        attribute quantity : Integer = 1;
        attribute type : String = "Component";
    }
    doc /* Verifier in `< 380`-SLOC `verify_baseline::checks::docstring_tier_rules` enforcing explicit `[DOC-TIER: A]` actuator restriction and `<= 4`-line Tier-B/C `[REQ-*]` / `[FLS: fls_*]` ceiling under Contract B */
    part def DocstringTierEnforcementGate {
        attribute quantity : Integer = 1;
        attribute type : String = "Component";
    }
    doc /* Canonical `< 380`-SLOC CST traversal engine in `deap_core::rust_cst::doc_edit` inspecting `ra_ap_syntax::ast::{HasDocComments, HasAttrs}`, `rowan::TextRange`, `ra_ap_vfs`, `ra_ap_paths`, `camino`, and `ignore` under Contract A */
    part def LosslessCstDocstringScanner {
        attribute quantity : Integer = 1;
        attribute type : String = "Component";
    }
    doc /* Forward `< 380`-SLOC docstring injector in `compile_sysml::docstring_cst` executing `ra_ap_text_edit` and `ra_ap_line_index` surgical plans with `--compact-tier-bc` under Contract A and Contract B */
    part def TieredDocstringSurgicalSynthesizer {
        attribute quantity : Integer = 1;
        attribute type : String = "Component";
    }
    part def UnifiedSafetyDocstringSynthesizer {
    }
}