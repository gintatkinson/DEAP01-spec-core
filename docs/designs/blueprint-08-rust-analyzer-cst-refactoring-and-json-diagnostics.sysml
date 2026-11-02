doc /* Translated from Level 0 OEM Markdown specification */
package BP_08_Lossless_Rust_CST_Codemod_Engine_and_Unified_rustc_Compatible_JSON_Diagnostics {
    attribute Blueprint_ID : String = "BP-08";
    attribute Requirement_ID : String = "REQ-0608";
    attribute Title : String = "Lossless Rust CST Codemod Engine and Unified rustc-Compatible JSON Diagnostics";
    attribute Status : String = "Draft";
    attribute Target_Crates : String = "crates/deap-core, crates/compile-sysml, crates/verify-baseline, crates/ingest-sysml, crates/assemble-conops, crates/reconcile-backlog";
    attribute Tracked_Issues : String = "#742";
    attribute Generation_Mode : String = "subagent";
    attribute Specification_Source : String = "crates/deap-core/src/lib.rs:1-40";
    doc /* `ra_ap_vfs`, `ra_ap_paths`, `camino::Utf8Path`, and `ignore::WalkBuilder` virtual file system with CRLF/LF normalization. */
    attribute crates_deap_core_src_rust_cst_vfs_rs : String = "**New `< 380` SLOC**";
    doc /* Lossless Red/Green CST parser wrapping `ra_ap_syntax::SourceFile`, `rowan`, and `ra_ap_line_index::LineIndex`. */
    attribute crates_deap_core_src_rust_cst_parse_rs : String = "**New `< 380` SLOC**";
    doc /* Deterministic `BTreeMap`-ordered AST item inventory and DEAP v2.0 metrics extraction. */
    attribute crates_deap_core_src_rust_cst_items_rs : String = "**New `< 380` SLOC**";
    doc /* Atomic `ra_ap_text_edit` module partitioner with `pulldown-cmark` and `pulldown-cmark-to-cmark` Check-37 blueprint path updates. */
    attribute crates_deap_core_src_rust_cst_extract_module_rs : String = "**New `< 380` SLOC**";
    doc /* Lossless `DocTier` classifier and `ra_ap_text_edit` 2-line docblock compactor (`--compact-tier-bc`). */
    attribute crates_deap_core_src_rust_cst_doc_edit_rs : String = "**New `< 380` SLOC**";
    doc /* Single-line `rustc`-compatible NDJSON diagnostic emitter and `ra_ap_text_edit` `MachineApplicable` fix engine. */
    attribute crates_deap_core_src_rust_cst_diagnostics_ndjson_rs : String = "**New `< 380` SLOC**";
    doc /* Official Cargo manifest and lockfile verifier built on `cargo-lock`, `cargo_metadata`, `cargo-util-schemas`, `cargo-platform`, and `toml_edit`. */
    attribute crates_deap_core_src_rust_cst_manifest_lock_rs : String = "**New `< 380` SLOC**";
    doc /* Completely deleted (`-7,558` physical LOC / `-1,049` residual scanner SLOC) and superseded by typed `rust_cst` modules. */
    attribute crates_deap_core_src_source_ast_rs : String = "**Complete Deletion `-7,558` LOC**";
    doc /* Retires 5,631-LOC hand-rolled docstring scanner in favor of `deap_core::rust_cst::doc_edit` CST docblock operations. */
    attribute crates_compile_sysml_src_docstring_rs : String = "**Net Deletion `-4,900+` SLOC**";
    doc /* Retires 3,751-LOC hand-rolled line scanner in favor of `DocTier` CST metrics. */
    attribute crates_verify_baseline_src_checks_docstring_gate_rs : String = "**Net Deletion `-3,300+` SLOC**";
    doc /* Retires 3,214-LOC hand-rolled 5-pillar AST scanner in favor of `ra_ap_syntax` CST metrics and `DEAP-E*` NDJSON diagnostics. */
    attribute crates_verify_baseline_src_audit_code_rs : String = "**Net Deletion `-2,800+` SLOC**";
    doc /* Migrated from token/line heuristics to exact `ra_ap_syntax` and `pulldown-cmark` AST traversal. */
    attribute crates_verify_baseline_src_checks_structural_rs : String = "**Net Deletion `-4,200+` SLOC**";
    doc /* Migrated manifest and workspace directory traversal to `cargo-lock`, `cargo_metadata`, `cargo-util-schemas`, `cargo-platform`, `toml_edit`, `camino`, `ignore`, and `ra_ap_vfs`. */
    attribute crates_deap_core_src_workspace_rs : String = "**Net Deletion `-620+` SLOC**";
    doc /* Downward ratchet converges strictly to `0` scanner exemptions across all 6 workspace crates (`<= 500` SLOC ceiling). */
    attribute crates_verify_baseline_src_ratchet_table_rs : String = "**Ratchet Convergence to `0`**";
    doc /* [unit: `SLOC`] */
    attribute max_fn_exec_sloc : Real;
    doc /* [unit: `params`] */
    attribute max_fn_param_count : Real;
    doc /* [unit: `flags`] */
    attribute max_mut_bool_flags : Real;
    doc /* [unit: `files`] */
    attribute stage_fixed_point_mismatch_files : Real;
    doc /* [unit: `tokens`] */
    attribute runtime_llm_token_count : Real;
    doc /* [unit: `lines`] */
    attribute tier_bc_max_doc_lines : Real;
    doc /* verify_offline_locked_build(lock_path) */
    action def verify_offline_locked_build;
    doc /* list_rust_items(path) */
    action def list_rust_items;
    doc /* extract_rust_module(from_path, to_path, items_csv) */
    action def extract_rust_module;
    doc /* verify_stage_fixed_point(stage1_digest) */
    action def verify_stage_fixed_point;
    doc /* format_ndjson_line(msg) */
    action def format_ndjson_line;
    doc /* analyze_metrics(source) */
    action def analyze_metrics;
    doc /* [unit: `SLOC`] */
    assert constraint C_CST_01 {
        max_fn_exec_sloc >= 1 and max_fn_exec_sloc <= 50;
    }
    doc /* [unit: `params`] */
    assert constraint C_CST_02 {
        max_fn_param_count >= 0 and max_fn_param_count <= 5;
    }
    doc /* [unit: `flags`] */
    assert constraint C_CST_03 {
        max_mut_bool_flags >= 0 and max_mut_bool_flags <= 0;
    }
    doc /* [unit: `files`] */
    assert constraint C_CST_04 {
        stage_fixed_point_mismatch_files >= 0 and stage_fixed_point_mismatch_files <= 0;
    }
    doc /* [unit: `tokens`] */
    assert constraint C_CST_05 {
        runtime_llm_token_count >= 0 and runtime_llm_token_count <= 0;
    }
    doc /* [unit: `lines`] */
    assert constraint C_CST_06 {
        tier_bc_max_doc_lines >= 2 and tier_bc_max_doc_lines <= 2;
    }
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0608_BP_08_Lossless_Rust_CST_Codemod_Engine_and_Unified_rustc_Compatible_JSON_Diagnostics {
        id = "REQ-0608";
        text = "LosslessRustCstAndJsonDiagnosticsEngine (**REQ-0608** / BP-08 / DEAP-BP-DET-08, UUIDv5 6ba7b810-9dad-11d1-80b4-00c04fd430c8, Class P linear syntax-tree traversal and byte-span splicing complexity, governed by DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C) replaces approximately";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_lossless_red_green_cst_roundtrip_and_exact_deap_v2_0_ast_metrics_extraction : String = "- **Given**: A Rust .rs fixture containing outer/inner docblocks, inline and block comments, raw strings containing braces, #[rt_loop] and #[cfg(test)] modules, methods taking &self and &mut self, a function with let mut flag: bool = false, a 4-clause compound boolean predicate, a pub fn returning Result<usize, String>, and a pub(crate) fn helper (plus a second fixture with an incomplete trailing expression). - **When**: LosslessRustCstAnalyzer parses both fixtures into ra_ap_syntax + rowan Red/Green syntax trees and computes DEAP v2.0 metrics and DocTier classifications. - **Then**: Root.";
        attribute ac_02_deterministic_btreemap_item_inventory_and_atomic_module_extraction_with_check_37_path_rewriting : String = "- **Given**: An existing workspace module <name>.rs referenced in docs/architecture/blueprints/ *.md and README.md, containing multiple top-level structs and functions with leading /// docblocks and #[...] outer attributes inside an isolated temporary workspace fixture. - **When**: compile-sysml --list-rust-items <name>.rs followed by compile-sysml --extract-rust-module --from <name>/mod.rs --to <name>/<sub>.rs --items <ItemA,ItemB> is executed after transitioning <name>.rs to <name>/mod.rs. - **Then**: --list-rust-items emits a deterministic BTreeMap-ordered inventory (kind, name,.";
        attribute ac_03_fail_closed_rejection_on_span_collisions_missing_symbols_or_post_splice_syntax_errors : String = "- **Given**: An --extract-rust-module or --apply-machine-fixes invocation targeting either an overlapping pair of TextRange byte intervals, a non-existent AST item identifier, or a replacement payload that introduces a CST syntax error. - **When**: ZeroTokenRustSpanCodemodEngine or UnifiedRustcJsonDiagnosticEmitter evaluates the proposed SpanSpliceEdit transaction. - **Then**: Execution fails closed with exit code 1, emits a single-line rustc-compatible JSON diagnostic (DEAP-E0801 missing item, DEAP-E0802 overlapping span collision, or DEAP-E0803 post-splice syntax error), and leaves all.";
        attribute ac_04_lossless_tier_b_c_docblock_compaction_skipping_self_hosted_generated_modules : String = "- **Given**: A Rust directory tree containing DocTier::RustTierA public items with 7-section docstrings, DocTier::RustTierBC private/pub(crate) items carrying legacy 28-line 7-section docblocks with Realises: [REQ-*] bindings, and a crates/compile-sysml/src/modular_codegen/generated/ subtree. - **When**: compile-sysml --compact-tier-bc <DIR> executes across the directory tree. - **Then**: Legacy 28-line docblocks on DocTier::RustTierBC items outside generated/ are losslessly compacted to 2-line /// <summary> + /// Realises: [REQ-*] contracts (**C-CST-06**) without mutating executable tokens.";
        attribute ac_05_structured_codegen_boundary_preserving_stage_1_to_stage_2_metacircular_sha_256_fixed_point : String = "- **Given**: The self-hosting modular codegen schema .pipeline/upstream/schema/modular_codegen.sysml lowered through syn + proc-macro2 + quote + prettyplease inside crates/compile-sysml/src/modular_codegen/. - **When**: Stage-1 generated sources under crates/compile-sysml/src/modular_codegen/generated/ are re-emitted into an isolated Stage-2 directory and verified for SHA-256 tree parity. - **Then**: SHA-256(Stage 1) == SHA-256(Stage 2) holds across all generated relative paths (**C-CST-04** stage_fixed_point_mismatch_files == 0), whereas any injected formatting or AST divergence fails closed.";
        attribute ac_06_unified_rustc_compatible_single_line_json_diagnostics_and_machineapplicable_auto_fix_across_all_five_clis : String = "- **Given**: Validation failures triggered across compile-sysml, verify-baseline, ingest-sysml, assemble-conops, and reconcile-backlog with --message-format=json, including diagnostics carrying both \"MachineApplicable\" and \"MaybeIncorrect\" child replacement spans. - **When**: Standard output is parsed line-by-line as JSON and --apply-machine-fixes is subsequently invoked on the offending workspace fixture. - **Then**: Every CLI emits strictly single-line NDJSON objects with reason: \"compiler-message\", message.code.code prefixed with \"DEAP-E\", exact UTF-8 byte_start/byte_end and.";
        attribute ac_07_air_gapped_offline_cargo_lock_reproducibility_and_twelve_gate_blueprint_conformance : String = "- **Given**: Workspace Cargo.toml and Cargo.lock populated via cargo fetch alongside docs/designs/blueprint-08-rust-analyzer-cst-refactoring-and-json-diagnostics.md. - **When**: cargo check --workspace --offline --locked and ./target/release/verify-baseline check-blueprint docs/designs/blueprint-08-rust-analyzer-cst-refactoring-and-json-diagnostics.md are executed. - **Then**: Offline locked dependency resolution succeeds without network access (cargo vendor-compatible), and all 12 mechanical blueprint verification gates pass with exit code 0, persisting.";
        doc /* \begin{aligned} \text{Lossless}(S) &\iff \bigoplus_{i=1}^{k} \text{bytes}(t_i) = S \land \sum_{i=1}^{k} \lvert \text{range}(t_i) \rvert = \lvert S \rvert \\ \text{ProductionExecSloc}(S) &= \lvert \{ \ell \in \text{Lines}(S) \mid \exists t \in \tau(T_{\text{red}}(S) \setminus T_{\text{test}}), \quad \text{kind}(t) \notin \mathcal{K}_{\text{trivia}} \land \ell \cap \text{lines}(t) \neq \emptyset \} \rvert \\ \forall f \in \text{Fns}(T_{\text{red}}(S) \setminus T_{\text{test}}), \quad \text{ExecSloc}(f) &\le 50 \land \lvert \text{Params}(f) \setminus \{\text{self}, \&\text{self}, \&\text{mut self}\} \rvert \le 5 \land \text{MutBools}(f) = 0 \end{aligned} */
        constraint def Invariant_REQ_0608_SC_01;
        doc /* \begin{aligned} \text{DisjointValid}(S, \mathcal{E}) &\iff \forall i \in \{1, \dots, m\}, \quad 0 \le a_i \le b_i \le \lvert S \rvert \land \text{is\_char\_boundary}(S, a_i) \land \text{is\_char\_boundary}(S, b_i) \\ &\land \forall i \in \{1, \dots, m - 1\}, \quad b_i \le a_{i+1} \\ \text{AdmissibleCommit}(S, \mathcal{E}) &\iff \text{DisjointValid}(S, \mathcal{E}) \land \lvert \text{errors}(\text{SourceFile::parse}(\text{Splice}(S, \mathcal{E}))) \rvert = 0 \end{aligned} */
        constraint def Invariant_REQ_0608_SC_03;
        doc /* \begin{aligned} \text{Emit}_{\text{modular}}(M) &= \text{prettyplease::unparse}(\text{syn::parse2}(\text{quote}(M))) \\ \text{SHA-256}(\mathcal{T}_{\text{Stage 1}}) &= \text{SHA-256}(\mathcal{T}_{\text{Stage 2}}) \iff \Delta_{\text{mismatch}}(\mathcal{T}_{\text{Stage 1}}, \mathcal{T}_{\text{Stage 2}}) = 0 \end{aligned} */
        constraint def Invariant_REQ_0608_SC_02;
        doc /* \begin{aligned} \forall d_j \in \mathcal{D}_{\text{json}}, \quad \text{count}(\text{encode}(d_j), \text{0x0A}) &= 0 \land \text{count}(\text{encode}(d_j), \text{0x1B}) = 0 \\ T_{\text{cst}}(N, M) &= O(N + M \log M), \quad S_{\text{cst}}(N, M) = O(N + M), \quad K_{\text{LLM}} = 0 \end{aligned} */
        constraint def Invariant_REQ_0608_SC_04_REQ_0608;
        require Invariant_REQ_0608_SC_01;
        require Invariant_REQ_0608_SC_03;
        require Invariant_REQ_0608_SC_02;
        require Invariant_REQ_0608_SC_04_REQ_0608;
        verify by AC_01_Lossless_Red_Green_CST_Roundtrip_and_Exact_DEAP_v2_0_AST_Metrics_Extraction;
        verify by AC_02_Deterministic_BTreeMap_Item_Inventory_and_Atomic_Module_Extraction_with_Check_37_Path_Rewriting;
        verify by AC_03_Fail_Closed_Rejection_on_Span_Collisions_Missing_Symbols_or_Post_Splice_Syntax_Errors;
        verify by AC_04_Lossless_Tier_B_C_Docblock_Compaction_Skipping_Self_Hosted_Generated_Modules;
        verify by AC_05_Structured_Codegen_Boundary_Preserving_Stage_1_to_Stage_2_Metacircular_SHA_256_Fixed_Point;
        verify by AC_06_Unified_rustc_Compatible_Single_Line_JSON_Diagnostics_and_MachineApplicable_Auto_Fix_Across_All_Five_CLIs;
        verify by AC_07_Air_Gapped_Offline_Cargo_Lock_Reproducibility_and_Twelve_Gate_Blueprint_Conformance;
        satisfy by AirGappedCargoLockGovernor;
    }
    doc /* Extracted from Mermaid stateDiagram specification. */
    state def BP_08_Lossless_Rust_CST_Codemod_Engine_and_Unified_rustc_Compatible_JSON_Diagnostics_LifecycleState {
        state Idle;
        state ParsingLosslessCst;
        state AnalyzingMetricsAndTiers;
        state ValidatingDisjointSpans;
        state EmittingJsonDiagnostics;
        state SplicingAndFormatting;
        state RewritingCheck37Paths;
        state VerifyingSelfHostFixedPoint;
        state VerifiedClean;
        transition Initial_to_Idle first Initial then Idle;
        transition Idle_to_ParsingLosslessCst_on_parseRustCst first Idle accept parseRustCst then ParsingLosslessCst;
        transition ParsingLosslessCst_to_AnalyzingMetricsAndTiers_on_losslessRedGreenTreeReady first ParsingLosslessCst accept losslessRedGreenTreeReady then AnalyzingMetricsAndTiers;
        transition AnalyzingMetricsAndTiers_to_ValidatingDisjointSpans_on_planCodemodOrMachineFix first AnalyzingMetricsAndTiers accept planCodemodOrMachineFix then ValidatingDisjointSpans;
        transition ValidatingDisjointSpans_to_EmittingJsonDiagnostics_on_rejectSpanCollisionOrMissingSymbol first ValidatingDisjointSpans accept rejectSpanCollisionOrMissingSymbol then EmittingJsonDiagnostics;
        transition ValidatingDisjointSpans_to_SplicingAndFormatting_on_disjointUtf8SpansVerified first ValidatingDisjointSpans accept disjointUtf8SpansVerified then SplicingAndFormatting;
        transition SplicingAndFormatting_to_EmittingJsonDiagnostics_on_rejectPostSpliceSyntaxError first SplicingAndFormatting accept rejectPostSpliceSyntaxError then EmittingJsonDiagnostics;
        transition SplicingAndFormatting_to_RewritingCheck37Paths_on_rustfmtAndCstReparseClean first SplicingAndFormatting accept rustfmtAndCstReparseClean then RewritingCheck37Paths;
        transition RewritingCheck37Paths_to_VerifyingSelfHostFixedPoint_on_atomicFilesCommitted first RewritingCheck37Paths accept atomicFilesCommitted then VerifyingSelfHostFixedPoint;
        transition VerifyingSelfHostFixedPoint_to_EmittingJsonDiagnostics_on_rejectFixedPointDriftE0705 first VerifyingSelfHostFixedPoint accept rejectFixedPointDriftE0705 then EmittingJsonDiagnostics;
        transition VerifyingSelfHostFixedPoint_to_VerifiedClean_on_stage1EqualsStage2Sha256 first VerifyingSelfHostFixedPoint accept stage1EqualsStage2Sha256 then VerifiedClean;
        transition EmittingJsonDiagnostics_to_Terminal first EmittingJsonDiagnostics then Terminal;
        transition VerifiedClean_to_Terminal first VerifiedClean then Terminal;
    }
    doc /* - **Given**: A Rust .rs fixture containing outer/inner docblocks, inline and block comments, raw strings containing braces, #[rt_loop] and #[cfg(test)] modules, methods taking &self and &mut self, a function with let mut flag: bool = false, a 4-clause compound boolean predicate, a pub fn returning Result<usize, String>, and a pub(crate) fn helper (plus a second fixture with an incomplete trailing expression). - **When**: LosslessRustCstAnalyzer parses both fixtures into ra_ap_syntax + rowan Red/Green syntax trees and computes DEAP v2.0 metrics and DocTier classifications. - **Then**: Root. */
    use case def UC_AC_01_Lossless_Red_Green_CST_Roundtrip_and_Exact_DEAP_v2_0_AST_Metrics_Extraction {
        subject AirGappedCargoLockGovernor;
        objective "Lossless Red Green CST Roundtrip and Exact DEAP v2 0 AST Metrics Extraction";
    }
    doc /* - **Given**: An existing workspace module <name>.rs referenced in docs/architecture/blueprints/ *.md and README.md, containing multiple top-level structs and functions with leading /// docblocks and #[...] outer attributes inside an isolated temporary workspace fixture. - **When**: compile-sysml --list-rust-items <name>.rs followed by compile-sysml --extract-rust-module --from <name>/mod.rs --to <name>/<sub>.rs --items <ItemA,ItemB> is executed after transitioning <name>.rs to <name>/mod.rs. - **Then**: --list-rust-items emits a deterministic BTreeMap-ordered inventory (kind, name,. */
    use case def UC_AC_02_Deterministic_BTreeMap_Item_Inventory_and_Atomic_Module_Extraction_with_Check_37_Path_Rewriting {
        subject AirGappedCargoLockGovernor;
        objective "Deterministic BTreeMap Item Inventory and Atomic Module Extraction with Check 37 Path Rewriting";
    }
    doc /* - **Given**: An --extract-rust-module or --apply-machine-fixes invocation targeting either an overlapping pair of TextRange byte intervals, a non-existent AST item identifier, or a replacement payload that introduces a CST syntax error. - **When**: ZeroTokenRustSpanCodemodEngine or UnifiedRustcJsonDiagnosticEmitter evaluates the proposed SpanSpliceEdit transaction. - **Then**: Execution fails closed with exit code 1, emits a single-line rustc-compatible JSON diagnostic (DEAP-E0801 missing item, DEAP-E0802 overlapping span collision, or DEAP-E0803 post-splice syntax error), and leaves all. */
    use case def UC_AC_03_Fail_Closed_Rejection_on_Span_Collisions_Missing_Symbols_or_Post_Splice_Syntax_Errors {
        subject AirGappedCargoLockGovernor;
        objective "Fail Closed Rejection on Span Collisions Missing Symbols or Post Splice Syntax Errors";
    }
    doc /* - **Given**: A Rust directory tree containing DocTier::RustTierA public items with 7-section docstrings, DocTier::RustTierBC private/pub(crate) items carrying legacy 28-line 7-section docblocks with Realises: [REQ-*] bindings, and a crates/compile-sysml/src/modular_codegen/generated/ subtree. - **When**: compile-sysml --compact-tier-bc <DIR> executes across the directory tree. - **Then**: Legacy 28-line docblocks on DocTier::RustTierBC items outside generated/ are losslessly compacted to 2-line /// <summary> + /// Realises: [REQ-*] contracts (**C-CST-06**) without mutating executable tokens. */
    use case def UC_AC_04_Lossless_Tier_B_C_Docblock_Compaction_Skipping_Self_Hosted_Generated_Modules {
        subject AirGappedCargoLockGovernor;
        objective "Lossless Tier B C Docblock Compaction Skipping Self Hosted Generated Modules";
    }
    doc /* - **Given**: The self-hosting modular codegen schema .pipeline/upstream/schema/modular_codegen.sysml lowered through syn + proc-macro2 + quote + prettyplease inside crates/compile-sysml/src/modular_codegen/. - **When**: Stage-1 generated sources under crates/compile-sysml/src/modular_codegen/generated/ are re-emitted into an isolated Stage-2 directory and verified for SHA-256 tree parity. - **Then**: SHA-256(Stage 1) == SHA-256(Stage 2) holds across all generated relative paths (**C-CST-04** stage_fixed_point_mismatch_files == 0), whereas any injected formatting or AST divergence fails closed. */
    use case def UC_AC_05_Structured_Codegen_Boundary_Preserving_Stage_1_to_Stage_2_Metacircular_SHA_256_Fixed_Point {
        subject AirGappedCargoLockGovernor;
        objective "Structured Codegen Boundary Preserving Stage 1 to Stage 2 Metacircular SHA 256 Fixed Point";
    }
    doc /* - **Given**: Validation failures triggered across compile-sysml, verify-baseline, ingest-sysml, assemble-conops, and reconcile-backlog with --message-format=json, including diagnostics carrying both \"MachineApplicable\" and \"MaybeIncorrect\" child replacement spans. - **When**: Standard output is parsed line-by-line as JSON and --apply-machine-fixes is subsequently invoked on the offending workspace fixture. - **Then**: Every CLI emits strictly single-line NDJSON objects with reason: \"compiler-message\", message.code.code prefixed with \"DEAP-E\", exact UTF-8 byte_start/byte_end and. */
    use case def UC_AC_06_Unified_rustc_Compatible_Single_Line_JSON_Diagnostics_and_MachineApplicable_Auto_Fix_Across_All_Five_CLIs {
        subject AirGappedCargoLockGovernor;
        objective "Unified rustc Compatible Single Line JSON Diagnostics and MachineApplicable Auto Fix Across All Five CLIs";
    }
    doc /* - **Given**: Workspace Cargo.toml and Cargo.lock populated via cargo fetch alongside docs/designs/blueprint-08-rust-analyzer-cst-refactoring-and-json-diagnostics.md. - **When**: cargo check --workspace --offline --locked and ./target/release/verify-baseline check-blueprint docs/designs/blueprint-08-rust-analyzer-cst-refactoring-and-json-diagnostics.md are executed. - **Then**: Offline locked dependency resolution succeeds without network access (cargo vendor-compatible), and all 12 mechanical blueprint verification gates pass with exit code 0, persisting. */
    use case def UC_AC_07_Air_Gapped_Offline_Cargo_Lock_Reproducibility_and_Twelve_Gate_Blueprint_Conformance {
        subject AirGappedCargoLockGovernor;
        objective "Air Gapped Offline Cargo Lock Reproducibility and Twelve Gate Blueprint Conformance";
    }
    doc /* Extracted from Mermaid sequenceDiagram specification. */
    interaction def BP_08_Lossless_Rust_CST_Codemod_Engine_and_Unified_rustc_Compatible_JSON_Diagnostics_SequenceInteraction {
        lifeline AirGappedCargoLockGovernor : AirGappedCargoLockGovernor;
        lifeline LosslessRustCstAnalyzer : LosslessRustCstAnalyzer;
        lifeline ZeroTokenRustSpanCodemodEngine : ZeroTokenRustSpanCodemodEngine;
        lifeline StructuredModularCodegenSynthesizer : StructuredModularCodegenSynthesizer;
        lifeline UnifiedRustcJsonDiagnosticEmitter : UnifiedRustcJsonDiagnosticEmitter;
        message verify_offline_locked_build from AirGappedCargoLockGovernor to LosslessRustCstAnalyzer;
        message reply_SupplyChainReceipt from LosslessRustCstAnalyzer to AirGappedCargoLockGovernor;
        message list_rust_items from ZeroTokenRustSpanCodemodEngine to LosslessRustCstAnalyzer;
        message reply_RustItemInventory from LosslessRustCstAnalyzer to ZeroTokenRustSpanCodemodEngine;
        message extract_rust_module from ZeroTokenRustSpanCodemodEngine to ZeroTokenRustSpanCodemodEngine;
        message verify_stage_fixed_point from StructuredModularCodegenSynthesizer to StructuredModularCodegenSynthesizer;
        message format_ndjson_line from ZeroTokenRustSpanCodemodEngine to UnifiedRustcJsonDiagnosticEmitter;
        message reply_String from UnifiedRustcJsonDiagnosticEmitter to ZeroTokenRustSpanCodemodEngine;
        message analyze_metrics from UnifiedRustcJsonDiagnosticEmitter to LosslessRustCstAnalyzer;
        message reply_DeapV2AstMetrics from LosslessRustCstAnalyzer to UnifiedRustcJsonDiagnosticEmitter;
    }
    doc /* Mandates reproducible `--offline --locked` workspace builds against pinned `Cargo.lock` dependency checksums. [unit: flag] */
    part def AirGappedCargoLockGovernor {
        attribute offline_locked_enforced : Boolean = true;
        doc /* Supports deterministic `cargo vendor` source bundling for zero-network air-gapped DO-330 TQL-4 build runners. [unit: flag] */
        attribute vendor_bundle_supported : Boolean = true;
        doc /* Reads workspace `Cargo.toml` and `Cargo.lock` crate versions and SHA-256 package checksums. [protocol: CargoTomlAndLockfile] */
        port in manifest_lock_in : String {
            flow in manifest_lock_in_flow : String;
        }
        doc /* Emits verified offline dependency lock and vendoring readiness status. [protocol: SupplyChainVerificationReceipt] */
        port out lock_receipt_out : String {
            flow out lock_receipt_out_flow : String;
        }
    }
    doc /* Upper bound on executable lines of code per Rust function excluding comments, docblocks, attributes, and blank lines. [unit: SLOC] */
    part def LosslessRustCstAnalyzer {
        attribute max_fn_exec_sloc : Integer = 50;
        doc /* Upper bound on formal parameters per Rust callable excluding `self`, `&self`, and `&mut self` receivers. [unit: params] */
        attribute max_fn_param_count : Integer = 5;
        doc /* Permitted count of mutable boolean local state flags (`let mut flag: bool`) within a single function body. [unit: flags] */
        attribute max_mut_bool_flags : Integer = 0;
        doc /* Maximum boolean operator clause count permitted in a single unhoisted compound predicate expression. [unit: clauses] */
        attribute max_predicate_arity : Integer = 3;
        doc /* Requires root `rowan` GreenNode byte projection to equal the input UTF-8 `.rs` source buffer bit-for-bit. [unit: flag] */
        attribute lossless_roundtrip_required : Boolean = true;
        doc /* Canonical deterministic blueprint component identifier (`DEAP-BP-DET-08`). [unit: uuidv5] */
        attribute deterministic_uuid : String = "6ba7b810-9dad-11d1-80b4-00c04fd430c8";
        doc /* Receives raw `.rs` UTF-8 bytes for lossless `ra_ap_syntax::SourceFile::parse` Red/Green tree construction. [protocol: Utf8RustSourceBuffer] */
        port in rust_source_in : String {
            flow in rust_source_in_flow : String;
        }
        doc /* Emits deterministic `BTreeMap`-ordered AST item descriptors with `TextRange` byte/line spans and `DocTier`. [protocol: BTreeOrderedItemInventory] */
        port out item_inventory_out : String {
            flow out item_inventory_out_flow : String;
        }
        doc /* Emits `production_exec_sloc`, per-`fn` `exec_sloc`, `param_count`, mutable bool count, predicate arity, and `Result` flags. [protocol: DeapV2AstMetricsReport] */
        port out metrics_report_out : String {
            flow out metrics_report_out_flow : String;
        }
    }
    doc /* Permitted Stage-1 versus Stage-2 SHA-256 file digest mismatch count across self-hosted modular codegen (`E0705`). [unit: files] */
    part def StructuredModularCodegenSynthesizer {
        attribute stage_fixed_point_mismatch_files : Integer = 0;
        doc /* Formats `quote` `TokenStream` ASTs deterministically via `syn::parse2` and `prettyplease::unparse` without external subprocesses. [unit: flag] */
        attribute in_process_prettyplease : Boolean = true;
        doc /* Receives 4-level lowered SysML v2 `ModuleGraph` descriptors inside `crates/compile-sysml/src/modular_codegen/`. [protocol: LoweredModuleGraphIr] */
        port in lowered_ir_in : String {
            flow in lowered_ir_in_flow : String;
        }
        doc /* Emits deterministic formatted Rust compilation units preserving `E0705` Stage-1 == Stage-2 SHA-256 identity. [protocol: PrettypleaseRustUnit] */
        port out formatted_rust_out : String {
            flow out formatted_rust_out_flow : String;
        }
    }
    doc /* Count of workspace CLI binaries unified on `--message-format=json` (`compile-sysml`, `verify-baseline`, `ingest-sysml`, `assemble-conops`, `reconcile-backlog`). [unit: binaries] */
    part def UnifiedRustcJsonDiagnosticEmitter {
        attribute cli_binary_count : Integer = 5;
        doc /* Suppresses all ANSI escape sequences and multi-line unstructured banner logs when `--message-format=json` is active. [unit: flag] */
        attribute zero_ansi_in_json_mode : Boolean = true;
        doc /* Restricts `--apply-machine-fixes` exclusively to disjoint child suggestions tagged `"MachineApplicable"`. [unit: flag] */
        attribute machine_applicable_only : Boolean = true;
        doc /* Canonical top-level JSON discriminator matching `rustc` and `cargo check --message-format=json`. [unit: schema] */
        attribute envelope_reason : String = "compiler-message";
        doc /* Receives structured compiler, baseline check, ingestion, ConOps, or backlog findings with exact UTF-8 `TextRange` spans. [protocol: DeapDiagnosticFinding] */
        port in finding_in : String {
            flow in finding_in_flow : String;
        }
        doc /* Emits single-line newline-delimited `rustc`-compatible JSON diagnostic objects to standard output. [protocol: RustcNdjsonLineStream] */
        port out ndjson_out : String {
            flow out ndjson_out_flow : String;
        }
        doc /* Applies reverse-byte-ordered `MachineApplicable` replacements and re-verifies lossless CST syntax integrity. [protocol: AtomicSpanFixTransaction] */
        port inout machine_fix_inout : String;
    }
    doc /* Mandatory zero runtime LLM token consumption across item listing, module extraction, and Tier B/C docstring compaction. [unit: tokens] */
    part def ZeroTokenRustSpanCodemodEngine {
        attribute runtime_llm_token_count : Integer = 0;
        doc /* Target compacted docstring line count (`/// ` + `/// Realises: [REQ-*]`) on `DocTier::RustTierBC` items. [unit: lines] */
        attribute tier_bc_max_doc_lines : Integer = 2;
        doc /* Excludes `crates/compile-sysml/src/modular_codegen/generated/` from `--compact-tier-bc` docstring rewriting. [unit: flag] */
        attribute skip_generated_tree : Boolean = true;
        doc /* Deterministically updates `.rs` citations to `/mod.rs` across `docs/architecture/blueprints/ *.md` and `README.md`. [unit: flag] */
        attribute rewrite_check37_citations : Boolean = true;
        doc /* Receives `--list-rust-items`, `--extract-rust-module`, or `--compact-tier-bc` CLI invocation payloads. [protocol: CstCodemodCommand] */
        port in codemod_command_in : String {
            flow in codemod_command_in_flow : String;
        }
        doc /* Commits non-overlapping CST byte-span splices, Tier A `//!` headers, `rustfmt` formatting, and Check-37 path updates atomically. [protocol: AtomicWorkspacePatch] */
        port out atomic_patch_out : String {
            flow out atomic_patch_out_flow : String;
        }
        doc /* Emits fail-closed `DEAP-E0801`..`DEAP-E0804` diagnostics on syntax errors, missing items, or span collisions. [protocol: RustcJsonDiagnosticStream] */
        port out codemod_error_out : String {
            flow out codemod_error_out_flow : String;
        }
    }
}