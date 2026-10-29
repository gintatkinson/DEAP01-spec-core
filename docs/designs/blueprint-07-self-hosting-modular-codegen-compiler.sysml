doc /* Translated from Level 0 OEM Markdown specification */
package BP_07_Self_Hosting_4_Level_Modular_Multi_File_Code_Generator_Blueprint {
    attribute Blueprint_ID : String = "BP-07";
    attribute Requirement_ID : String = "REQ-0607";
    attribute Title : String = "Self-Hosting 4-Level Modular Multi-File Code Generator Blueprint";
    attribute Status : String = "Approved";
    attribute Target_Crates : String = "crates/compile-sysml, crates/deap-core";
    attribute Tracked_Issues : String = "#742, #733, #739, #740, #741";
    attribute Generation_Mode : String = "subagent";
    attribute Specification_Source : String = ".pipeline/upstream/schema/modular_codegen.sysml:1-120";
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
    doc /* verifyDagAndLayers(pkg) */
    action def verifyDagAndLayers;
    doc /* partitionAndHoist(pkg) */
    action def partitionAndHoist;
    doc /* emitModularRustTree(graph) */
    action def emitModularRustTree;
    doc /* emitModularFlutterTree(graph) */
    action def emitModularFlutterTree;
    doc /* verifyFixedPointHash(stage1) */
    action def verifyFixedPointHash;
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
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0607_BP_07_Self_Hosting_4_Level_Modular_Multi_File_Code_Generator_Blueprint {
        id = "REQ-0607";
        text = "SelfHostingModularCodegenCompiler (**REQ-0607** / BP-07 / DEAP-BP-DET-07, UUIDv5 6ba7b810-9dad-11d1-80b4-00c04fd430c7, Class P linear Kahn DAG and AST traversal complexity, governed by DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C) eliminates monolithic single-file code";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_fail_closed_rejection_on_directed_dependency_cycles_and_upward_layer_imports : String = "- **Given**: A SysML v2 PackageDef fixture containing either a directed dependency cycle across packages/classifiers or a @Layer(Domain) classifier referencing a @Layer(Presentation) or @Layer(Application) classifier. - **When**: compile-sysml --emit-modular-rust <OUT_DIR> --schema <FIXTURE> is executed. - **Then**: TopologicalLayerDagVerifier aborts transactionally with exit code 1, emits structured diagnostic E0702 (cycle) or E0703 (illegal upward layer dependency), and leaves <OUT_DIR> with zero generated files written.";
        attribute ac_02_four_level_zero_dead_end_deterministic_lowering_within_modularity_bounds : String = "- **Given**: A valid multi-package SysML v2 model containing nested PackageDef hierarchies, multi-facet PartDef components, a 40-step ActionDef sequence, a 30-transition StateDef state machine, and a 25-clause ConstraintDef boolean Expr tree. - **When**: compile-sysml --emit-modular-rust <OUT_DIR> --schema <FIXTURE> compiles and lowers the model across Levels 1 through 4. - **Then**: Compilation succeeds with exit code 0, emitting hierarchical mod.rs barrels, facet submodules (model, invariants, ports, fsm, actions, calcs), sequential action stages, step_from_<state>() handlers, and.";
        attribute ac_03_metacircular_self_hosting_stage_1_to_stage_2_sha_256_fixed_point_identity : String = "- **Given**: The authoritative self-hosting compiler schema .pipeline/upstream/schema/modular_codegen.sysml and the committed Stage-1 tree under crates/compile-sysml/src/modular_codegen/generated/ wired into compile-sysml. - **When**: The Stage-1-linked compile-sysml binary re-compiles .pipeline/upstream/schema/modular_codegen.sysml into an isolated Stage-2 temporary directory and compares GeneratedTreeManifest SHA-256 digests. - **Then**: SHA-256(Stage 1) == SHA-256(Stage 2) holds across 100% of relative paths and file contents (**C-MOD-04** stage_fixed_point_mismatch_files == 0), whereas.";
        attribute ac_04_modular_multi_file_flutter_contract_emission_and_monolith_barrel_decomposition : String = "- **Given**: A multi-package SysML v2 schema compiled via compile-sysml --emit-flutter-contracts <APP_DIR> --schema <FIXTURE> and the Rust workspace source file crates/compile-sysml/src/flutter_codegen.rs. - **When**: Modular Flutter contract generation executes and workspace modularity tests inspect both the emitted Dart tree and crates/compile-sysml/src/. - **Then**: Per-package and per-classifier .g.dart units (models/generated/<pkg>/<part>_view.g.dart, data/generated/ffi/ *.g.dart, data/generated/dto/<pkg>/<dto>_dto.g.dart, data/seeds/generated/ *.g.dart, test/generated/support/ *.g.dart).";
        attribute ac_05_zero_regex_do_178c_seven_section_docstring_clippy_forbid_and_audit_code_compliance : String = "- **Given**: All hand-authored modules under crates/compile-sysml/src/modular_codegen/ ** and all Stage-1 generated Rust modules under crates/compile-sysml/src/modular_codegen/generated/ **. - **When**: cargo clippy --workspace --all-targets -- -D warnings and ./target/release/verify-baseline audit-code --path crates/compile-sysml/src/modular_codegen --dry-run are executed. - **Then**: Both verification gates exit 0 with zero Clippy forbid violations, zero Check 34 violation tokens, zero Check 19 hardcoded domain concepts, and 100% Check 36 unique 7-section DO-178C safety docstrings.";
        attribute ac_06_dogfood_sysml_v2_blueprint_ingestion_and_twelve_gate_closed_loop_conformance : String = "- **Given**: This engineering blueprint at docs/designs/blueprint-07-self-hosting-modular-codegen-compiler.md. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-07-self-hosting-modular-codegen-compiler.md is executed. - **Then**: All 12 blueprint verification gates pass with exit code 0, deterministically synthesizing docs/designs/blueprint-07-self-hosting-modular-codegen-compiler.sysml and .digest.json via roundtrip SysML v2 compilation and verifying closed-loop GitHub SSOT parity.";
        doc /* \begin{aligned} \text{Acyclic}(G) &\iff \exists \pi : V \to \{1, \dots, \lvert V \rvert\}, \quad \forall (u, v) \in E, \quad \pi(u) < \pi(v) \\ \text{LayerMonotonic}(G) &\iff \forall (u, v) \in E, \quad \text{tier}(u) \ge \text{tier}(v) \end{aligned} */
        constraint def Invariant_REQ_0607_SC_02;
        doc /* \begin{aligned} \forall u \in \mathcal{U}, \quad \text{LOC}(u) &\le L_{\max} = 400 \land \text{Items}(u) \le K_{\max} = 12 \\ \forall f \in \text{Funcs}(u), \quad v(G_f) &= \lvert E_f \rvert - \lvert V_f \rvert + 2 \le C_{\max} = 8 \end{aligned} */
        constraint def Invariant_REQ_0607_SC_01;
        doc /* \begin{aligned} T_{\text{codegen}}(V, E, B) &= O(\lvert V \rvert + \lvert E \rvert + \lvert B \rvert) \\ S_{\text{codegen}}(V, E, B) &= O(\lvert V \rvert + \lvert E \rvert + \lvert B \rvert) \end{aligned} */
        constraint def Invariant_REQ_0607_REQ_0607;
        doc /* \begin{aligned} \text{SHA-256}(\mathcal{T}_{\text{Stage 1}}) &= \text{SHA-256}(\mathcal{T}_{\text{Stage 2}}) \\ \Delta_{\text{mismatch}}(\mathcal{T}_{\text{Stage 1}}, \mathcal{T}_{\text{Stage 2}}) &= \lvert \{ p \in \text{Paths}(\mathcal{T}_{\text{Stage 1}} \cup \mathcal{T}_{\text{Stage 2}}) \mid H_1(p) \neq H_2(p) \} \rvert = 0 \end{aligned} */
        constraint def Invariant_REQ_0607_SC_03;
        require Invariant_REQ_0607_SC_02;
        require Invariant_REQ_0607_SC_01;
        require Invariant_REQ_0607_REQ_0607;
        require Invariant_REQ_0607_SC_03;
        verify by AC_01_Fail_Closed_Rejection_on_Directed_Dependency_Cycles_and_Upward_Layer_Imports;
        verify by AC_02_Four_Level_Zero_Dead_End_Deterministic_Lowering_Within_Modularity_Bounds;
        verify by AC_03_Metacircular_Self_Hosting_Stage_1_to_Stage_2_SHA_256_Fixed_Point_Identity;
        verify by AC_04_Modular_Multi_File_Flutter_Contract_Emission_and_Monolith_Barrel_Decomposition;
        verify by AC_05_Zero_Regex_DO_178C_Seven_Section_Docstring_Clippy_Forbid_and_Audit_Code_Compliance;
        verify by AC_06_Dogfood_SysML_v2_Blueprint_Ingestion_and_Twelve_Gate_Closed_Loop_Conformance;
        satisfy by FourLevelAstPartitionerAndHoister;
    }
    doc /* Extracted from Mermaid stateDiagram specification. */
    state def BP_07_Self_Hosting_4_Level_Modular_Multi_File_Code_Generator_Blueprint_LifecycleState {
        state Idle;
        state VerifyingDagAndLayers;
        state RejectedFailClosed;
        state LoweringFourLevels;
        state SynthesizingInMemoryUnits;
        state AtomicTreePersisted;
        state VerifyingSelfHostFixedPoint;
        state VerifiedFixedPoint;
        transition Initial_to_Idle first Initial then Idle;
        transition Idle_to_VerifyingDagAndLayers_on_loadPackageAst first Idle accept loadPackageAst then VerifyingDagAndLayers;
        transition VerifyingDagAndLayers_to_RejectedFailClosed_on_rejectCycleOrLayerViolation first VerifyingDagAndLayers accept rejectCycleOrLayerViolation then RejectedFailClosed;
        transition VerifyingDagAndLayers_to_LoweringFourLevels_on_verifyDagAndLayers first VerifyingDagAndLayers accept verifyDagAndLayers then LoweringFourLevels;
        transition LoweringFourLevels_to_SynthesizingInMemoryUnits_on_partitionAndHoist first LoweringFourLevels accept partitionAndHoist then SynthesizingInMemoryUnits;
        transition SynthesizingInMemoryUnits_to_AtomicTreePersisted_on_emitModularRustTree first SynthesizingInMemoryUnits accept emitModularRustTree then AtomicTreePersisted;
        transition AtomicTreePersisted_to_VerifyingSelfHostFixedPoint_on_verifyFixedPointHash first AtomicTreePersisted accept verifyFixedPointHash then VerifyingSelfHostFixedPoint;
        transition VerifyingSelfHostFixedPoint_to_RejectedFailClosed_on_rejectCycleOrLayerViolation first VerifyingSelfHostFixedPoint accept rejectCycleOrLayerViolation then RejectedFailClosed;
        transition VerifyingSelfHostFixedPoint_to_VerifiedFixedPoint_on_stage1Sha256EqualsStage2Sha256 first VerifyingSelfHostFixedPoint accept stage1Sha256EqualsStage2Sha256 then VerifiedFixedPoint;
        transition RejectedFailClosed_to_Terminal first RejectedFailClosed then Terminal;
        transition VerifiedFixedPoint_to_Terminal first VerifiedFixedPoint then Terminal;
    }
    doc /* - **Given**: A SysML v2 PackageDef fixture containing either a directed dependency cycle across packages/classifiers or a @Layer(Domain) classifier referencing a @Layer(Presentation) or @Layer(Application) classifier. - **When**: compile-sysml --emit-modular-rust <OUT_DIR> --schema <FIXTURE> is executed. - **Then**: TopologicalLayerDagVerifier aborts transactionally with exit code 1, emits structured diagnostic E0702 (cycle) or E0703 (illegal upward layer dependency), and leaves <OUT_DIR> with zero generated files written. */
    use case def UC_AC_01_Fail_Closed_Rejection_on_Directed_Dependency_Cycles_and_Upward_Layer_Imports {
        subject FourLevelAstPartitionerAndHoister;
        objective "Fail Closed Rejection on Directed Dependency Cycles and Upward Layer Imports";
    }
    doc /* - **Given**: A valid multi-package SysML v2 model containing nested PackageDef hierarchies, multi-facet PartDef components, a 40-step ActionDef sequence, a 30-transition StateDef state machine, and a 25-clause ConstraintDef boolean Expr tree. - **When**: compile-sysml --emit-modular-rust <OUT_DIR> --schema <FIXTURE> compiles and lowers the model across Levels 1 through 4. - **Then**: Compilation succeeds with exit code 0, emitting hierarchical mod.rs barrels, facet submodules (model, invariants, ports, fsm, actions, calcs), sequential action stages, step_from_<state>() handlers, and. */
    use case def UC_AC_02_Four_Level_Zero_Dead_End_Deterministic_Lowering_Within_Modularity_Bounds {
        subject FourLevelAstPartitionerAndHoister;
        objective "Four Level Zero Dead End Deterministic Lowering Within Modularity Bounds";
    }
    doc /* - **Given**: The authoritative self-hosting compiler schema .pipeline/upstream/schema/modular_codegen.sysml and the committed Stage-1 tree under crates/compile-sysml/src/modular_codegen/generated/ wired into compile-sysml. - **When**: The Stage-1-linked compile-sysml binary re-compiles .pipeline/upstream/schema/modular_codegen.sysml into an isolated Stage-2 temporary directory and compares GeneratedTreeManifest SHA-256 digests. - **Then**: SHA-256(Stage 1) == SHA-256(Stage 2) holds across 100% of relative paths and file contents (**C-MOD-04** stage_fixed_point_mismatch_files == 0), whereas. */
    use case def UC_AC_03_Metacircular_Self_Hosting_Stage_1_to_Stage_2_SHA_256_Fixed_Point_Identity {
        subject FourLevelAstPartitionerAndHoister;
        objective "Metacircular Self Hosting Stage 1 to Stage 2 SHA 256 Fixed Point Identity";
    }
    doc /* - **Given**: A multi-package SysML v2 schema compiled via compile-sysml --emit-flutter-contracts <APP_DIR> --schema <FIXTURE> and the Rust workspace source file crates/compile-sysml/src/flutter_codegen.rs. - **When**: Modular Flutter contract generation executes and workspace modularity tests inspect both the emitted Dart tree and crates/compile-sysml/src/. - **Then**: Per-package and per-classifier .g.dart units (models/generated/<pkg>/<part>_view.g.dart, data/generated/ffi/ *.g.dart, data/generated/dto/<pkg>/<dto>_dto.g.dart, data/seeds/generated/ *.g.dart, test/generated/support/ *.g.dart). */
    use case def UC_AC_04_Modular_Multi_File_Flutter_Contract_Emission_and_Monolith_Barrel_Decomposition {
        subject FourLevelAstPartitionerAndHoister;
        objective "Modular Multi File Flutter Contract Emission and Monolith Barrel Decomposition";
    }
    doc /* - **Given**: All hand-authored modules under crates/compile-sysml/src/modular_codegen/ ** and all Stage-1 generated Rust modules under crates/compile-sysml/src/modular_codegen/generated/ **. - **When**: cargo clippy --workspace --all-targets -- -D warnings and ./target/release/verify-baseline audit-code --path crates/compile-sysml/src/modular_codegen --dry-run are executed. - **Then**: Both verification gates exit 0 with zero Clippy forbid violations, zero Check 34 violation tokens, zero Check 19 hardcoded domain concepts, and 100% Check 36 unique 7-section DO-178C safety docstrings. */
    use case def UC_AC_05_Zero_Regex_DO_178C_Seven_Section_Docstring_Clippy_Forbid_and_Audit_Code_Compliance {
        subject FourLevelAstPartitionerAndHoister;
        objective "Zero Regex DO 178C Seven Section Docstring Clippy Forbid and Audit Code Compliance";
    }
    doc /* - **Given**: This engineering blueprint at docs/designs/blueprint-07-self-hosting-modular-codegen-compiler.md. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-07-self-hosting-modular-codegen-compiler.md is executed. - **Then**: All 12 blueprint verification gates pass with exit code 0, deterministically synthesizing docs/designs/blueprint-07-self-hosting-modular-codegen-compiler.sysml and .digest.json via roundtrip SysML v2 compilation and verifying closed-loop GitHub SSOT parity. */
    use case def UC_AC_06_Dogfood_SysML_v2_Blueprint_Ingestion_and_Twelve_Gate_Closed_Loop_Conformance {
        subject FourLevelAstPartitionerAndHoister;
        objective "Dogfood SysML v2 Blueprint Ingestion and Twelve Gate Closed Loop Conformance";
    }
    doc /* Extracted from Mermaid sequenceDiagram specification. */
    interaction def BP_07_Self_Hosting_4_Level_Modular_Multi_File_Code_Generator_Blueprint_SequenceInteraction {
        lifeline SelfHostingModularCodegenCompiler : SelfHostingModularCodegenCompiler;
        lifeline TopologicalLayerDagVerifier : TopologicalLayerDagVerifier;
        lifeline FourLevelAstPartitionerAndHoister : FourLevelAstPartitionerAndHoister;
        lifeline ModularRustMultiFileEmitter : ModularRustMultiFileEmitter;
        lifeline ModularFlutterMultiFileEmitter : ModularFlutterMultiFileEmitter;
        message verifyDagAndLayers from SelfHostingModularCodegenCompiler to TopologicalLayerDagVerifier;
        message reply_ValidatedDagOrder from TopologicalLayerDagVerifier to SelfHostingModularCodegenCompiler;
        message partitionAndHoist from SelfHostingModularCodegenCompiler to FourLevelAstPartitionerAndHoister;
        message reply_ModuleGraph from FourLevelAstPartitionerAndHoister to SelfHostingModularCodegenCompiler;
        message emitModularRustTree from SelfHostingModularCodegenCompiler to ModularRustMultiFileEmitter;
        message reply_GeneratedTreeManifest from ModularRustMultiFileEmitter to SelfHostingModularCodegenCompiler;
        message emitModularFlutterTree from SelfHostingModularCodegenCompiler to ModularFlutterMultiFileEmitter;
        message reply_FlutterContractReport from ModularFlutterMultiFileEmitter to SelfHostingModularCodegenCompiler;
        message verifyFixedPointHash from SelfHostingModularCodegenCompiler to SelfHostingModularCodegenCompiler;
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
    doc /* Emits unique Check 36 7-section DO-178C safety headers on every generated Rust module and item. [unit: flag] */
    part def ModularRustMultiFileEmitter {
        attribute enforce_seven_section_docs : Boolean = true;
        doc /* Verifies in-memory `ModularityBounds` across all units prior to `write_atomic` commit. [unit: flag] */
        attribute atomic_staging_enabled : Boolean = true;
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
        doc /* Emits deterministic `E0702`, `E0703`, and `E0705` diagnostics on fail-closed abort. [protocol: CompilerDiagnosticStream] */
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