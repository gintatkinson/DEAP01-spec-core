doc /* Translated from Level 0 OEM Markdown specification */
package BP_07_Self_Hosting_4_Level_Modular_Multi_File_Code_Generator_Blueprint {
    attribute Blueprint_ID : String = "BP-07";
    attribute Requirement_ID : String = "REQ-0607";
    attribute Title : String = "Self-Hosting 4-Level Modular Multi-File Code Generator Blueprint";
    attribute Status : String = "Approved";
    attribute Target_Crates : String = "crates/compile-sysml, crates/deap-core";
    attribute Tracked_Issues : String = "#742, #733, #739, #740, #741";
    attribute Generation_Mode : String = "subagent";
    attribute Specification_Source : String = "crates/compile-sysml/src/lib.rs:1-40";
    doc /* [unit: `LOC`] */
    attribute max_file_lines : Real;
    doc /* [unit: `items`] */
    attribute max_items_per_file : Real;
    doc /* [unit: `vG`] */
    attribute max_cyclomatic_complexity : Real;
    doc /* [unit: `files`] */
    attribute stage_fixed_point_mismatch_files : Real;
    doc /* [unit: `count`] */
    attribute runtime_llm_subagent_count : Real;
    doc /* [unit: `LOC`] */
    attribute submodule_max_exec_sloc : Real;
    doc /* verify_dag_and_layers(node_layers, edges) */
    action def verify_dag_and_layers;
    doc /* partition_and_hoist(pkg) */
    action def partition_and_hoist;
    doc /* synthesize_module_tokens(module_name) */
    action def synthesize_module_tokens;
    doc /* build_syn_file(tokens) */
    action def build_syn_file;
    doc /* format_pretty_syn_file(file) */
    action def format_pretty_syn_file;
    doc /* emit_modular_rust_tree(target_dir, units, node_layers, edges, bounds) */
    action def emit_modular_rust_tree;
    doc /* emit_modular_flutter_tree(graph) */
    action def emit_modular_flutter_tree;
    doc /* verify_fixed_point_digest(stage1, stage2) */
    action def verify_fixed_point_digest;
    doc /* [unit: `LOC`] */
    assert constraint C_MOD_01 {
        max_file_lines >= 50 and max_file_lines <= 400;
    }
    doc /* [unit: `items`] */
    assert constraint C_MOD_02 {
        max_items_per_file >= 1 and max_items_per_file <= 12;
    }
    doc /* [unit: `vG`] */
    assert constraint C_MOD_03 {
        max_cyclomatic_complexity >= 1 and max_cyclomatic_complexity <= 8;
    }
    doc /* [unit: `files`] */
    assert constraint C_MOD_04 {
        stage_fixed_point_mismatch_files >= 0 and stage_fixed_point_mismatch_files <= 0;
    }
    doc /* [unit: `count`] */
    assert constraint C_MOD_05 {
        runtime_llm_subagent_count >= 0 and runtime_llm_subagent_count <= 0;
    }
    doc /* [unit: `LOC`] */
    assert constraint C_MOD_06 {
        submodule_max_exec_sloc >= 1 and submodule_max_exec_sloc <= 380;
    }
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0607_BP_07_Self_Hosting_4_Level_Modular_Multi_File_Code_Generator_Blueprint {
        id = "REQ-0607";
        text = "SelfHostingModularCodegenCompiler (**REQ-0607** / BP-07 / DEAP-BP-DET-07, UUIDv5: 6ba7b810-9dad-11d1-80b4-00c04fd430c7, Class P linear Kahn DAG and AST traversal complexity, governed by DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C) eliminates monolithic single-file code";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_fail_closed_rejection_on_directed_dependency_cycles_and_upward_layer_imports : String = "- **Given**: A SysML v2 PackageDef fixture containing either a directed dependency cycle across packages/classifiers or a @Layer(Domain) classifier referencing a @Layer(Presentation) or @Layer(Application) classifier. - **When**: compile-sysml --emit-modular-rust <OUT_DIR> --schema <FIXTURE> is executed. - **Then**: TopologicalLayerDagVerifier aborts transactionally with exit code 1, emits structured diagnostic E0702 (cycle) or E0703 (illegal upward layer dependency), and leaves <OUT_DIR> with zero generated files written.";
        attribute ac_02_contract_a_contract_b_four_level_lowering_with_typed_syn_file_ast_prettyplease_formatting_and_380_sloc_submodules : String = "- **Given**: A valid multi-package SysML v2 model containing nested PackageDef hierarchies, multi-facet PartDef components, a 40-step ActionDef sequence, a 30-transition StateDef state machine, and a 25-clause ConstraintDef boolean Expr tree. - **When**: compile-sysml --emit-modular-rust <OUT_DIR> --schema <FIXTURE> compiles and lowers the model across Levels 1 through 4 via proc-macro2 + quote::quote! + syn::parse2::<syn::File> and prettyplease::unparse. - **Then**: Compilation succeeds with exit code 0, emitting hierarchical mod.rs barrels, facet submodules (model, invariants, ports, fsm,.";
        attribute ac_03_contract_a_metacircular_self_hosting_stage_1_to_stage_2_sha_256_fixed_point_identity : String = "- **Given**: The authoritative self-hosting compiler schema and the committed Stage-1 tree under crates/compile-sysml/src/modular_codegen/generated/ verified parse-clean by ra_ap_syntax and wired into compile-sysml. - **When**: The Stage-1-linked compile-sysml binary re-compiles the self-hosting schema into an isolated Stage-2 temporary directory and compares GeneratedTreeManifest SHA-256 digests. - **Then**: SHA-256(Stage 1) == SHA-256(Stage 2) holds across 100% of relative paths and file contents (**C-MOD-04** stage_fixed_point_mismatch_files == 0), whereas any injected perturbation.";
        attribute ac_04_modular_multi_file_flutter_contract_emission_and_legacy_monolith_deletion : String = "- **Given**: A multi-package SysML v2 schema compiled via compile-sysml --emit-flutter-contracts <APP_DIR> --schema <FIXTURE> and the crates/compile-sysml/src/lib.rs module hierarchy. - **When**: Modular Flutter contract generation executes and workspace modularity tests inspect both the emitted Dart tree and crates/compile-sysml/src/. - **Then**: Per-package and per-classifier .g.dart units (models/generated/<pkg>/<part>_view.g.dart, data/generated/ffi/ *.g.dart, data/generated/dto/<pkg>/<dto>_dto.g.dart, data/seeds/generated/ *.g.dart, test/generated/support/ *.g.dart) are emitted alongside.";
        attribute ac_05_zero_hand_rolled_scanner_string_concatenation_codegen_do_178c_seven_section_docstring_and_clippy_forbid_compliance : String = "- **Given**: All hand-authored < 380-SLOC modules under crates/compile-sysml/src/modular_codegen/ ** and crates/deap-core/src/ **, plus all Stage-1 generated Rust modules under crates/compile-sysml/src/modular_codegen/generated/ **. - **When**: cargo clippy --workspace --all-targets -- -D warnings and ./target/release/verify-baseline audit-code --path crates/compile-sysml/src/modular_codegen --dry-run are executed. - **Then**: Both verification gates exit 0 with zero Clippy forbid violations, zero Check 34 violation tokens, zero Check 19 hardcoded domain concepts, zero legacy hand-rolled.";
        attribute ac_06_pre_built_binary_compatibility_guard_and_twelve_gate_closed_loop_blueprint_conformance : String = "- **Given**: This engineering blueprint at docs/designs/blueprint-07-self-hosting-modular-codegen-compiler.md with all .sysml constructs restricted to the token/grammar subset accepted by ./target/release/verify-baseline and all file:line anchors citing stable workspace files (Cargo.toml, crates/ * /src/lib.rs, crates/ * /src/main.rs). - **When**: ./target/release/verify-baseline check-blueprint ./docs/designs/blueprint-07-self-hosting-modular-codegen-compiler.md is executed. - **Then**: All 12 blueprint verification gates pass with exit code 0, deterministically synthesizing.";
        doc /* \begin{aligned} \text{Acyclic}(G) &\iff \exists \pi : V \to \{1, \dots, \lvert V \rvert\}, \quad \forall (u, v) \in E, \quad \pi(u) < \pi(v) \\ \text{LayerMonotonic}(G) &\iff \forall (u, v) \in E, \quad \text{tier}(u) \ge \text{tier}(v) \end{aligned} */
        constraint def Invariant_REQ_0607_SC_02;
        doc /* \begin{aligned} \forall m \in \mathcal{M}_{\text{sub}}, \quad \text{SLOC}_{\text{exec}}(m) &< S_{\max} = 380 \\ \forall u \in \mathcal{U}, \quad \text{LOC}(u) &\le L_{\max} = 400 \land \text{Items}(u) \le K_{\max} = 12 \\ \forall f \in \text{Funcs}(u), \quad v(G_f) &= \lvert E_f \rvert - \lvert V_f \rvert + 2 \le C_{\max} = 8 \end{aligned} */
        constraint def Invariant_REQ_0607_SC_01;
        doc /* \begin{aligned} \forall u \in \mathcal{U}, \quad \text{syn::parse2}(\text{tokens}(u)) &\in \mathcal{L}(\text{syn::File}) \\ \text{Content}(u) &= \text{normalize\_lf}(\text{prettyplease::unparse}(\text{syn::parse2}(\text{tokens}(u)))) \\ \text{SLOC}_{\text{legacy\_scanner\_codegen}} &= 0 \end{aligned} */
        constraint def Invariant_REQ_0607_SC_04;
        doc /* \begin{aligned} T_{\text{codegen}}(V, E, B) &= O(\lvert V \rvert + \lvert E \rvert + \lvert B \rvert) \\ S_{\text{codegen}}(V, E, B) &= O(\lvert V \rvert + \lvert E \rvert + \lvert B \rvert) \end{aligned} */
        constraint def Invariant_REQ_0607_REQ_0607;
        doc /* \begin{aligned} \text{SHA-256}(\mathcal{T}_{\text{Stage 1}}) &= \text{SHA-256}(\mathcal{T}_{\text{Stage 2}}) \\ \Delta_{\text{mismatch}}(\mathcal{T}_{\text{Stage 1}}, \mathcal{T}_{\text{Stage 2}}) &= \lvert \{ p \in \text{Paths}(\mathcal{T}_{\text{Stage 1}} \cup \mathcal{T}_{\text{Stage 2}}) \mid H_1(p) \neq H_2(p) \} \rvert = 0 \end{aligned} */
        constraint def Invariant_REQ_0607_SC_03;
        require Invariant_REQ_0607_SC_02;
        require Invariant_REQ_0607_SC_01;
        require Invariant_REQ_0607_SC_04;
        require Invariant_REQ_0607_REQ_0607;
        require Invariant_REQ_0607_SC_03;
        verify by AC_01_Fail_Closed_Rejection_on_Directed_Dependency_Cycles_and_Upward_Layer_Imports;
        verify by AC_02_Contract_A_Contract_B_Four_Level_Lowering_with_Typed_syn_File_AST_prettyplease_Formatting_and_380_SLOC_Submodules;
        verify by AC_03_Contract_A_Metacircular_Self_Hosting_Stage_1_to_Stage_2_SHA_256_Fixed_Point_Identity;
        verify by AC_04_Modular_Multi_File_Flutter_Contract_Emission_and_Legacy_Monolith_Deletion;
        verify by AC_05_Zero_Hand_Rolled_Scanner_String_Concatenation_Codegen_DO_178C_Seven_Section_Docstring_and_Clippy_Forbid_Compliance;
        verify by AC_06_Pre_Built_Binary_Compatibility_Guard_and_Twelve_Gate_Closed_Loop_Blueprint_Conformance;
        satisfy by FourLevelAstPartitionerAndHoister;
    }
    doc /* Extracted from Mermaid stateDiagram specification. */
    state def BP_07_Self_Hosting_4_Level_Modular_Multi_File_Code_Generator_Blueprint_LifecycleState {
        state Idle;
        state VerifyingDagAndLayers;
        state RejectedFailClosed;
        state LoweringFourLevels;
        state SynthesizingInMemoryUnits;
        state BuildingSynAst;
        state FormattingPrettyPlease;
        state AtomicTreePersisted;
        state VerifyingSelfHostFixedPoint;
        state VerifiedFixedPoint;
        transition Initial_to_Idle first Initial then Idle;
        transition Idle_to_VerifyingDagAndLayers_on_loadPackageAst first Idle accept loadPackageAst then VerifyingDagAndLayers;
        transition VerifyingDagAndLayers_to_RejectedFailClosed_on_rejectCycleOrLayerViolation first VerifyingDagAndLayers accept rejectCycleOrLayerViolation then RejectedFailClosed;
        transition VerifyingDagAndLayers_to_LoweringFourLevels_on_verify_dag_and_layers first VerifyingDagAndLayers accept verify_dag_and_layers then LoweringFourLevels;
        transition LoweringFourLevels_to_SynthesizingInMemoryUnits_on_partitionAndHoist first LoweringFourLevels accept partitionAndHoist then SynthesizingInMemoryUnits;
        transition SynthesizingInMemoryUnits_to_BuildingSynAst_on_synthesize_module_tokens first SynthesizingInMemoryUnits accept synthesize_module_tokens then BuildingSynAst;
        transition BuildingSynAst_to_FormattingPrettyPlease_on_build_syn_file first BuildingSynAst accept build_syn_file then FormattingPrettyPlease;
        transition FormattingPrettyPlease_to_AtomicTreePersisted_on_format_pretty_syn_file first FormattingPrettyPlease accept format_pretty_syn_file then AtomicTreePersisted;
        transition AtomicTreePersisted_to_VerifyingSelfHostFixedPoint_on_emit_modular_rust_tree first AtomicTreePersisted accept emit_modular_rust_tree then VerifyingSelfHostFixedPoint;
        transition VerifyingSelfHostFixedPoint_to_RejectedFailClosed_on_rejectCycleOrLayerViolation first VerifyingSelfHostFixedPoint accept rejectCycleOrLayerViolation then RejectedFailClosed;
        transition VerifyingSelfHostFixedPoint_to_VerifiedFixedPoint_on_verify_fixed_point_digest first VerifyingSelfHostFixedPoint accept verify_fixed_point_digest then VerifiedFixedPoint;
        transition RejectedFailClosed_to_Terminal first RejectedFailClosed then Terminal;
        transition VerifiedFixedPoint_to_Terminal first VerifiedFixedPoint then Terminal;
    }
    doc /* - **Given**: A SysML v2 PackageDef fixture containing either a directed dependency cycle across packages/classifiers or a @Layer(Domain) classifier referencing a @Layer(Presentation) or @Layer(Application) classifier. - **When**: compile-sysml --emit-modular-rust <OUT_DIR> --schema <FIXTURE> is executed. - **Then**: TopologicalLayerDagVerifier aborts transactionally with exit code 1, emits structured diagnostic E0702 (cycle) or E0703 (illegal upward layer dependency), and leaves <OUT_DIR> with zero generated files written. */
    use case def UC_AC_01_Fail_Closed_Rejection_on_Directed_Dependency_Cycles_and_Upward_Layer_Imports {
        subject FourLevelAstPartitionerAndHoister;
        objective "Fail Closed Rejection on Directed Dependency Cycles and Upward Layer Imports";
    }
    doc /* - **Given**: A valid multi-package SysML v2 model containing nested PackageDef hierarchies, multi-facet PartDef components, a 40-step ActionDef sequence, a 30-transition StateDef state machine, and a 25-clause ConstraintDef boolean Expr tree. - **When**: compile-sysml --emit-modular-rust <OUT_DIR> --schema <FIXTURE> compiles and lowers the model across Levels 1 through 4 via proc-macro2 + quote::quote! + syn::parse2::<syn::File> and prettyplease::unparse. - **Then**: Compilation succeeds with exit code 0, emitting hierarchical mod.rs barrels, facet submodules (model, invariants, ports, fsm,. */
    use case def UC_AC_02_Contract_A_Contract_B_Four_Level_Lowering_with_Typed_syn_File_AST_prettyplease_Formatting_and_380_SLOC_Submodules {
        subject FourLevelAstPartitionerAndHoister;
        objective "Contract A Contract B Four Level Lowering with Typed syn File AST prettyplease Formatting and 380 SLOC Submodules";
    }
    doc /* - **Given**: The authoritative self-hosting compiler schema and the committed Stage-1 tree under crates/compile-sysml/src/modular_codegen/generated/ verified parse-clean by ra_ap_syntax and wired into compile-sysml. - **When**: The Stage-1-linked compile-sysml binary re-compiles the self-hosting schema into an isolated Stage-2 temporary directory and compares GeneratedTreeManifest SHA-256 digests. - **Then**: SHA-256(Stage 1) == SHA-256(Stage 2) holds across 100% of relative paths and file contents (**C-MOD-04** stage_fixed_point_mismatch_files == 0), whereas any injected perturbation. */
    use case def UC_AC_03_Contract_A_Metacircular_Self_Hosting_Stage_1_to_Stage_2_SHA_256_Fixed_Point_Identity {
        subject FourLevelAstPartitionerAndHoister;
        objective "Contract A Metacircular Self Hosting Stage 1 to Stage 2 SHA 256 Fixed Point Identity";
    }
    doc /* - **Given**: A multi-package SysML v2 schema compiled via compile-sysml --emit-flutter-contracts <APP_DIR> --schema <FIXTURE> and the crates/compile-sysml/src/lib.rs module hierarchy. - **When**: Modular Flutter contract generation executes and workspace modularity tests inspect both the emitted Dart tree and crates/compile-sysml/src/. - **Then**: Per-package and per-classifier .g.dart units (models/generated/<pkg>/<part>_view.g.dart, data/generated/ffi/ *.g.dart, data/generated/dto/<pkg>/<dto>_dto.g.dart, data/seeds/generated/ *.g.dart, test/generated/support/ *.g.dart) are emitted alongside. */
    use case def UC_AC_04_Modular_Multi_File_Flutter_Contract_Emission_and_Legacy_Monolith_Deletion {
        subject FourLevelAstPartitionerAndHoister;
        objective "Modular Multi File Flutter Contract Emission and Legacy Monolith Deletion";
    }
    doc /* - **Given**: All hand-authored < 380-SLOC modules under crates/compile-sysml/src/modular_codegen/ ** and crates/deap-core/src/ **, plus all Stage-1 generated Rust modules under crates/compile-sysml/src/modular_codegen/generated/ **. - **When**: cargo clippy --workspace --all-targets -- -D warnings and ./target/release/verify-baseline audit-code --path crates/compile-sysml/src/modular_codegen --dry-run are executed. - **Then**: Both verification gates exit 0 with zero Clippy forbid violations, zero Check 34 violation tokens, zero Check 19 hardcoded domain concepts, zero legacy hand-rolled. */
    use case def UC_AC_05_Zero_Hand_Rolled_Scanner_String_Concatenation_Codegen_DO_178C_Seven_Section_Docstring_and_Clippy_Forbid_Compliance {
        subject FourLevelAstPartitionerAndHoister;
        objective "Zero Hand Rolled Scanner String Concatenation Codegen DO 178C Seven Section Docstring and Clippy Forbid Compliance";
    }
    doc /* - **Given**: This engineering blueprint at docs/designs/blueprint-07-self-hosting-modular-codegen-compiler.md with all .sysml constructs restricted to the token/grammar subset accepted by ./target/release/verify-baseline and all file:line anchors citing stable workspace files (Cargo.toml, crates/ * /src/lib.rs, crates/ * /src/main.rs). - **When**: ./target/release/verify-baseline check-blueprint ./docs/designs/blueprint-07-self-hosting-modular-codegen-compiler.md is executed. - **Then**: All 12 blueprint verification gates pass with exit code 0, deterministically synthesizing. */
    use case def UC_AC_06_Pre_Built_Binary_Compatibility_Guard_and_Twelve_Gate_Closed_Loop_Blueprint_Conformance {
        subject FourLevelAstPartitionerAndHoister;
        objective "Pre Built Binary Compatibility Guard and Twelve Gate Closed Loop Blueprint Conformance";
    }
    doc /* Extracted from Mermaid sequenceDiagram specification. */
    interaction def BP_07_Self_Hosting_4_Level_Modular_Multi_File_Code_Generator_Blueprint_SequenceInteraction {
        lifeline Compiler : SelfHostingModularCodegenCompiler;
        lifeline DagVerifier : TopologicalLayerDagVerifier;
        lifeline Partitioner : FourLevelAstPartitionerAndHoister;
        lifeline RustEmitter : ModularRustMultiFileEmitter;
        lifeline FlutterEmitter : ModularFlutterMultiFileEmitter;
        message verify_dag_and_layers from Compiler to DagVerifier;
        message reply_ResultVecUsizeVecModularityDiagnostic from DagVerifier to Compiler;
        message partition_and_hoist from Compiler to Partitioner;
        message reply_ResultModuleGraphVecModularityDiagnostic from Partitioner to Compiler;
        message synthesize_module_tokens from Compiler to RustEmitter;
        message reply_TokenBuffer from RustEmitter to Compiler;
        message build_syn_file from Compiler to RustEmitter;
        message reply_ResultSynFileModularityDiagnostic from RustEmitter to Compiler;
        message format_pretty_syn_file from Compiler to RustEmitter;
        message reply_ResultStringModularityDiagnostic from RustEmitter to Compiler;
        message emit_modular_rust_tree from Compiler to RustEmitter;
        message reply_ResultGeneratedTreeManifestVecModularityDiagnostic from RustEmitter to Compiler;
        message emit_modular_flutter_tree from Compiler to FlutterEmitter;
        message reply_ResultFlutterContractReportVecModularityDiagnostic from FlutterEmitter to Compiler;
        message verify_fixed_point_digest from Compiler to RustEmitter;
        message reply_ResultBoolModularityDiagnostic from RustEmitter to Compiler;
    }
    doc /* Maximum sequential `ActionDef::steps` statements grouped into a single outlined stage function. [unit: steps] */
    part def FourLevelAstPartitionerAndHoister {
        attribute max_action_steps_per_stage : Integer = 6;
        doc /* Maximum outgoing `StateDef::transitions` arms per `step_from_()` handler unit. [unit: arms] */
        attribute max_transitions_per_handler : Integer = 6;
        doc /* Maximum boolean branching operators retained before hoisting `eval__term_` helpers. [unit: branches] */
        attribute anf_branch_budget : Integer = 6;
        doc /* Receives DAG-verified hierarchical `PackageDef`, `PartDef`, `ActionDef`, `StateDef`, `ConstraintDef`, and `CalcDef` nodes. [protocol: ValidatedPackageAst] */
        port in validated_ast_in : String {
            flow in validated_ast_in_flow : String;
        }
        doc /* Emits 4-level partitioned `ModuleGraph` of bounded `CompilationUnit` descriptors. [protocol: LoweredModuleGraph] */
        port out module_graph_out : String {
            flow out module_graph_out_flow : String;
        }
    }
    doc /* Emits top-level Dart barrel facades re-exporting per-package and per-classifier `.g.dart` units. [unit: flag] */
    part def ModularFlutterMultiFileEmitter {
        attribute emit_barrel_reexports : Boolean = true;
        doc /* Strict upper bound on `crates/compile-sysml/src/flutter_codegen.rs` barrel facade line count. [unit: LOC] */
        attribute facade_max_lines : Integer = 100;
        doc /* Receives unflattened `PackageDef` and lowered `ModuleGraph` for `--emit-flutter-contracts`. [protocol: LoweredModuleGraph] */
        port in flutter_module_graph_in : String {
            flow in flutter_module_graph_in_flow : String;
        }
        doc /* Emits modular `.g.dart` models, FFI/DTO bridges, DDL/seeds, spatial UI contract tests, and barrels. [protocol: FlutterContractReport] */
        port out flutter_manifest_out : String {
            flow out flutter_manifest_out_flow : String;
        }
    }
    doc /* Validates every synthesized token stream via syn::parse2:: failing closed on syntax errors. [unit: flag] */
    part def ModularRustMultiFileEmitter {
        attribute enforce_syn_parse_grammar : Boolean = true;
        doc /* Formats Rust AST via prettyplease::unparse with zero external rustfmt subprocess invocations. [unit: flag] */
        attribute canonical_prettyplease_format : Boolean = true;
        doc /* Verifies ra_ap_syntax lossless parse-clean AST and Stage-1 == Stage-2 SHA-256 fixed-point equality. [unit: flag] */
        attribute fixed_point_sha256_verification : Boolean = true;
        doc /* Net-negative legacy hand-rolled scanner and string-concatenation emitter deletion target in compile-sysml. [unit: LOC] */
        attribute legacy_string_codegen_deleted_sloc : Integer = 1100;
        doc /* Strict execution SLOC ceiling per modular codegen and CST extraction submodule under Contract B. [unit: LOC] */
        attribute submodule_max_exec_sloc : Integer = 380;
        doc /* Enforces Contract A typed AST fixed-point synthesis and Contract B submodule and binary compatibility bounds. [unit: flag] */
        attribute contract_a_b_enforced : Boolean = true;
        doc /* Receives lowered `ModuleGraph` targeted for `--emit-modular-rust`. [protocol: LoweredModuleGraph] */
        port in rust_module_graph_in : String {
            flow in rust_module_graph_in_flow : String;
        }
        doc /* Emits persisted Rust submodule tree and aggregate SHA-256 tree digest. [protocol: GeneratedTreeManifest] */
        port out rust_manifest_out : String {
            flow out rust_manifest_out_flow : String;
        }
    }
    doc /* Upper bound on physical lines of code permitted per generated compilation unit. [unit: LOC] */
    part def SelfHostingModularCodegenCompiler {
        attribute max_file_lines : Integer = 400;
        doc /* Upper bound on top-level type, trait, or function declarations per compilation unit. [unit: items] */
        attribute max_items_per_file : Integer = 12;
        doc /* Upper bound on McCabe cyclomatic complexity per generated or hand-authored function. [unit: vG] */
        attribute max_cyclomatic_complexity : Integer = 8;
        doc /* Count of Stage-1 versus Stage-2 SHA-256 file digest mismatches permitted at bootstrap verification. [unit: files] */
        attribute stage_fixed_point_mismatch_files : Integer = 0;
        doc /* Mandatory zero runtime LLM or subagent invocation count during deterministic compilation. [unit: count] */
        attribute runtime_llm_subagent_count : Integer = 0;
        doc /* Aborts all filesystem writes whenever any DAG, layer, or bounds invariant fails. [unit: flag] */
        attribute fail_closed_transactional : Boolean = true;
        doc /* Canonical deterministic blueprint component identifier (`DEAP-BP-DET-07`). [unit: uuidv5] */
        attribute deterministic_uuid : String = "6ba7b810-9dad-11d1-80b4-00c04fd430c7";
        doc /* Receives hierarchical `PackageDef` AST validated by `SemanticValidator`. [protocol: SysmlV2PackageAst] */
        port in package_ast_in : String {
            flow in package_ast_in_flow : String;
        }
        doc /* Emits verified multi-file Rust module hierarchy and `GeneratedTreeManifest`. [protocol: AtomicRustModuleTree] */
        port out rust_tree_out : String {
            flow out rust_tree_out_flow : String;
        }
        doc /* Emits verified multi-file `.g.dart` contract hierarchy and barrel facades. [protocol: AtomicDartModuleTree] */
        port out flutter_tree_out : String {
            flow out flutter_tree_out_flow : String;
        }
        doc /* Emits deterministic `E0701`, `E0702`, `E0703`, and `E0705` diagnostics on fail-closed abort. [protocol: CompilerDiagnosticStream] */
        port out diagnostic_out : String {
            flow out diagnostic_out_flow : String;
        }
    }
    doc /* Maximum architectural layer ordinal (`Domain = 1`, `Application = 2`, `Presentation = 3`). [unit: tier] */
    part def TopologicalLayerDagVerifier {
        attribute max_layer_tier : Integer = 3;
        doc /* Enforces iterative work-queue Kahn topological sorting without call-stack recursion. [unit: flag] */
        attribute zero_recursion_kahn : Boolean = true;
        doc /* Receives directed classifier and package dependency edges with resolved `LayerRole` tiers. [protocol: ModuleDependencyGraph] */
        port in dependency_edges_in : String {
            flow in dependency_edges_in_flow : String;
        }
        doc /* Emits deterministic topological emission sequence or `E0702`/`E0703` rejection diagnostics. [protocol: ValidatedDagOrder] */
        port out topological_order_out : String {
            flow out topological_order_out_flow : String;
        }
    }
}