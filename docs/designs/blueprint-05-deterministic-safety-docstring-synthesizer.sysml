doc /* Translated from Level 0 OEM Markdown specification */
package BP_05_Unified_AST_Safety_Docstring_Synthesizer_and_Check_36_Verification_Blueprint {
    attribute Blueprint_ID : String = "BP-05";
    attribute Requirement_ID : String = "REQ-0605";
    attribute Title : String = "Unified AST Safety Docstring Synthesizer and Check 36 Verification Blueprint";
    attribute Status : String = "Approved";
    attribute Target_Crates : String = "crates/deap-core, crates/compile-sysml, crates/verify-baseline";
    attribute Tracked_Issues : String = "#498, #500, #501, #508, #510";
    attribute Generation_Mode : String = "subagent";
    attribute Specification_Source : String = "crates/deap-core/src/source_ast.rs:1-120";
    doc /* Count of hardcoded byte-offset lookahead constants (`10` or `14`) in character/lifetime lexing [unit: count] */
    attribute magic_offset_constant_count : Integer = 0;
    doc /* Count of duplicate Rust/Dart lexical state machines outside `deap_core::source_ast` [unit: count] */
    attribute lexer_duplication_count : Integer = 0;
    doc /* Count of unmapped source items receiving `.first()` fallback or cross-entity requirement bindings [unit: count] */
    attribute false_attribution_count : Integer = 0;
    doc /* Byte diff produced when re-running `compile-sysml --inject-safety-docstrings` on enriched source [unit: bytes] */
    attribute docstring_idempotency_diff_bytes : Integer = 0;
    doc /* Ingests raw UTF-8 `.rs` and `.dart` source files from target workspace directories */
    port in SourceCodeFileInputPort : String {
        doc /* Ingests raw UTF-8 `.rs` and `.dart` source files from target workspace directories */
        flow in SourceCodeFileInputPort_flow : String;
    }
    doc /* Supplies parsed SysML v2 `PartDef`, `ActionDef`, `RequirementDef`, and `ConstraintDef` AST hierarchies */
    port in SysmlSchemaTraceabilityPort : PackageDef {
        doc /* Supplies parsed SysML v2 `PartDef`, `ActionDef`, `RequirementDef`, and `ConstraintDef` AST hierarchies */
        flow in SysmlSchemaTraceabilityPort_flow : PackageDef;
    }
    doc /* Emits bit-for-bit idempotent `.rs` and `.dart` source files enriched with 7-section safety docstrings */
    port out EnrichedSourceOutputPort : String {
        doc /* Emits bit-for-bit idempotent `.rs` and `.dart` source files enriched with 7-section safety docstrings */
        flow out EnrichedSourceOutputPort_flow : String;
    }
    doc /* Emits deterministic Check 36 and Check 34 docstring completeness and uniqueness verdicts */
    port out Check36VerificationReportPort : Boolean {
        doc /* Emits deterministic Check 36 and Check 34 docstring completeness and uniqueness verdicts */
        flow out Check36VerificationReportPort_flow : Boolean;
    }
    doc /* Run compile-sysml --inject-safety-docstrings <DIR> */
    action def RunCompileSysmlInjectSafetyDocstrings;
    doc /* Scan source lines via RustDartLexState and ScopedItemScannerState */
    action def ScanSourceLinesViaRustDartLexStateAnd;
    doc /* Resolve schema context via collect_all_parts without .first() fallback */
    action def ResolveSchemaContextViaCollectAll;
    doc /* Run verify-baseline . --no-domain (Check 36 & Check 34) */
    action def RunVerifyBaselineNoDomain;
    doc /* Tokenize .rs and .dart files using shared deap-core lexical state */
    action def TokenizeRsAndDartFilesUsing;
    doc /* Run ingest-sysml --schema blueprint-05 --format markdown */
    action def RunIngestSysmlSchemaBlueprint05;
    doc /* [unit: count] */
    assert constraint C_DOC_01 {
        magic_offset_constant_count >= 0 and magic_offset_constant_count <= 0;
    }
    doc /* [unit: count] */
    assert constraint C_DOC_02 {
        lexer_duplication_count >= 0 and lexer_duplication_count <= 0;
    }
    doc /* [unit: count] */
    assert constraint C_DOC_03 {
        false_attribution_count >= 0 and false_attribution_count <= 0;
    }
    doc /* [unit: bytes] */
    assert constraint C_DOC_04 {
        docstring_idempotency_diff_bytes >= 0 and docstring_idempotency_diff_bytes <= 0;
    }
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0605_BP_05_Unified_AST_Safety_Docstring_Synthesizer_and_Check_36_Verification_Blueprint {
        id = "REQ-0605";
        text = "The Digital Engineering Agent Platform (DEAP) requires 100% of production Rust (.rs) and Dart (.dart) declarations to carry complete, non-placeholder 7-section high-assurance safety docstrings (## 1. Safety Intent & Regulatory Scope through ## 7. Realization & Traceability) bound to formal SysML v2";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_unified_rust_and_dart_lexical_state_machine_without_magic_offsets : String = "- **Given**: .rs and .dart source files containing nested / * outer / * inner * / * / block comments, hashed raw strings r#\"...\"# and r##\"...\"##, Rust string literals containing \"{\", Dart interpolated strings \"value: {map['key']}\" containing nested quotes and comments, multi-byte UTF-8 character literals '↑', and multiple short lifetimes fn foo<'a, 'b>(x: &'a str, y: &'b str) within 10 bytes. - **When**: deap_core::source_ast, compile-sysml --inject-safety-docstrings, and verify-baseline Check 36 scan the files. - **Then**: All three crates execute the single unified.";
        attribute ac_02_strict_schema_context_resolution_and_recursive_part_traversal : String = "- **Given**: A multi-part SysML v2 PackageDef containing PrimaryController (REQ-PC-001, constraint ThermalLimit), a nested PartDef NestedSubunit (REQ-NS-002) inside SecondaryMonitor.parts, and an unmapped Rust struct SecondaryMonitor with method pub fn primary_controller(&self) and free function pub fn unmapped_helper(). - **When**: resolve_schema_context is invoked during compile-sysml --inject-safety-docstrings. - **Then**: PrimaryController resolves REQ-PC-001 and ThermalLimit, NestedSubunit resolves REQ-NS-002 via collect_all_parts, and both SecondaryMonitor::primary_controller and.";
        attribute ac_03_scoped_mod_tests_brace_tracking_unsafe_extern_classification_and_arrow_aware_impl_headers : String = "- **Given**: A Rust module containing #[cfg(test)] mod tests { ... } followed by a production pub unsafe fn post_test_api(), a #[cfg(test)] impl MockHelper { ... } block, an impl<F: Fn(u32) -> bool> FilterEngine<F> block, and a Dart class method preceded by a multi-line @pragma( 'vm:prefer-inline',) annotation. - **When**: compile-sysml --inject-safety-docstrings and verify-baseline Check 36 inspect the files. - **Then**: mod tests exits cleanly at its closing brace so post_test_api is audited (and fail-closed rejected with a non-zero violation error if undocumented), #[cfg(test)] impl.";
        attribute ac_04_bit_for_bit_idempotency_and_fail_closed_check_34_per_function_normalized_body_uniqueness : String = "- **Given**: Any Rust crate or Dart source directory enriched by compile-sysml --inject-safety-docstrings <DIR>, as well as a negative fixture containing duplicate normalized docstring bodies or missing sections. - **When**: compile-sysml --inject-safety-docstrings <DIR> is executed a second time and followed by ./target/release/verify-baseline . --no-domain. - **Then**: The second synthesis pass produces 0 modified bytes (docstring_idempotency_diff_bytes == 0), every synthesized docblock passes Check 36 7-section completeness and Check 34 seen_docblocks uniqueness, and any invalid, missing,.";
        attribute ac_05_dogfood_ingestion_and_sysml_v2_compilation_of_req_0605 : String = "- **Given**: This blueprint specification docs/designs/blueprint-05-deterministic-safety-docstring-synthesizer.md. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-05-deterministic-safety-docstring-synthesizer.md --dry-run and ./target/release/ingest-sysml --schema docs/designs/blueprint-05-deterministic-safety-docstring-synthesizer.md --format markdown are executed. - **Then**: The translator parses **REQ-0605**, UnifiedSafetyDocstringSynthesizer, all four BOM subcomponents, all four interface ports, all four state attributes, all four parametric.";
        doc /* \begin{aligned} T_{\text{lex}}(B) &= O(B) \\ T_{\text{resolve}}(I, P, R, C) &= O(I \cdot (P + R + C)) \\ \Delta_{\text{idempotent}}(S) &= \lVert f_{\text{inject}}(f_{\text{inject}}(S, M), M) - f_{\text{inject}}(S, M) \rVert_1 = 0 \end{aligned} */
        constraint def Invariant_REQ_0605_Under_REQ_0605_and_parametric_constraints_C_DOC_01_through_C_DOC_04_lexical_scanning_schema_context_resolution_and_forward_docstring_injection_satisfy_the_following_linear_complexity_and_idempotency_bounds;
        require Invariant_REQ_0605_Under_REQ_0605_and_parametric_constraints_C_DOC_01_through_C_DOC_04_lexical_scanning_schema_context_resolution_and_forward_docstring_injection_satisfy_the_following_linear_complexity_and_idempotency_bounds;
        verify by AC_01_Unified_Rust_and_Dart_Lexical_State_Machine_Without_Magic_Offsets;
        verify by AC_02_Strict_Schema_Context_Resolution_and_Recursive_Part_Traversal;
        verify by AC_03_Scoped_mod_tests_Brace_Tracking_unsafe_extern_Classification_and_Arrow_Aware_impl_Headers;
        verify by AC_04_Bit_for_Bit_Idempotency_and_Fail_Closed_Check_34_Per_Function_Normalized_Body_Uniqueness;
        verify by AC_05_Dogfood_Ingestion_and_SysML_v2_Compilation_of_REQ_0605;
        satisfy by Check36AstCompletenessVerifier;
    }
    doc /* Synthesized deterministic package lifecycle state machine. */
    state def BP_05_Unified_AST_Safety_Docstring_Synthesizer_and_Check_36_Verification_Blueprint_LifecycleState {
        state Check36AstCompletenessVerifier_State;
        state ScopedSchemaContextResolver_State;
        state SevenSectionDocstringGenerator_State;
        transition Initial_to_Check36AstCompletenessVerifier_State first Initial then Check36AstCompletenessVerifier_State;
        transition Check36AstCompletenessVerifier_State_to_ScopedSchemaContextResolver_State_on_RunCompileSysmlInjectSafetyDocstrings first Check36AstCompletenessVerifier_State accept RunCompileSysmlInjectSafetyDocstrings then ScopedSchemaContextResolver_State;
        transition ScopedSchemaContextResolver_State_to_SevenSectionDocstringGenerator_State_on_ScanSourceLinesViaRustDartLexStateAnd first ScopedSchemaContextResolver_State accept ScanSourceLinesViaRustDartLexStateAnd then SevenSectionDocstringGenerator_State;
        transition SevenSectionDocstringGenerator_State_to_Terminal first SevenSectionDocstringGenerator_State then Terminal;
    }
    doc /* - **Given**: .rs and .dart source files containing nested / * outer / * inner * / * / block comments, hashed raw strings r#\"...\"# and r##\"...\"##, Rust string literals containing \"{\", Dart interpolated strings \"value: {map['key']}\" containing nested quotes and comments, multi-byte UTF-8 character literals '↑', and multiple short lifetimes fn foo<'a, 'b>(x: &'a str, y: &'b str) within 10 bytes. - **When**: deap_core::source_ast, compile-sysml --inject-safety-docstrings, and verify-baseline Check 36 scan the files. - **Then**: All three crates execute the single unified. */
    use case def UC_AC_01_Unified_Rust_and_Dart_Lexical_State_Machine_Without_Magic_Offsets {
        subject Check36AstCompletenessVerifier;
        objective "Unified Rust and Dart Lexical State Machine Without Magic Offsets";
    }
    doc /* - **Given**: A multi-part SysML v2 PackageDef containing PrimaryController (REQ-PC-001, constraint ThermalLimit), a nested PartDef NestedSubunit (REQ-NS-002) inside SecondaryMonitor.parts, and an unmapped Rust struct SecondaryMonitor with method pub fn primary_controller(&self) and free function pub fn unmapped_helper(). - **When**: resolve_schema_context is invoked during compile-sysml --inject-safety-docstrings. - **Then**: PrimaryController resolves REQ-PC-001 and ThermalLimit, NestedSubunit resolves REQ-NS-002 via collect_all_parts, and both SecondaryMonitor::primary_controller and. */
    use case def UC_AC_02_Strict_Schema_Context_Resolution_and_Recursive_Part_Traversal {
        subject Check36AstCompletenessVerifier;
        objective "Strict Schema Context Resolution and Recursive Part Traversal";
    }
    doc /* - **Given**: A Rust module containing #[cfg(test)] mod tests { ... } followed by a production pub unsafe fn post_test_api(), a #[cfg(test)] impl MockHelper { ... } block, an impl<F: Fn(u32) -> bool> FilterEngine<F> block, and a Dart class method preceded by a multi-line @pragma( 'vm:prefer-inline',) annotation. - **When**: compile-sysml --inject-safety-docstrings and verify-baseline Check 36 inspect the files. - **Then**: mod tests exits cleanly at its closing brace so post_test_api is audited (and fail-closed rejected with a non-zero violation error if undocumented), #[cfg(test)] impl. */
    use case def UC_AC_03_Scoped_mod_tests_Brace_Tracking_unsafe_extern_Classification_and_Arrow_Aware_impl_Headers {
        subject Check36AstCompletenessVerifier;
        objective "Scoped mod tests Brace Tracking unsafe extern Classification and Arrow Aware impl Headers";
    }
    doc /* - **Given**: Any Rust crate or Dart source directory enriched by compile-sysml --inject-safety-docstrings <DIR>, as well as a negative fixture containing duplicate normalized docstring bodies or missing sections. - **When**: compile-sysml --inject-safety-docstrings <DIR> is executed a second time and followed by ./target/release/verify-baseline . --no-domain. - **Then**: The second synthesis pass produces 0 modified bytes (docstring_idempotency_diff_bytes == 0), every synthesized docblock passes Check 36 7-section completeness and Check 34 seen_docblocks uniqueness, and any invalid, missing,. */
    use case def UC_AC_04_Bit_for_Bit_Idempotency_and_Fail_Closed_Check_34_Per_Function_Normalized_Body_Uniqueness {
        subject Check36AstCompletenessVerifier;
        objective "Bit for Bit Idempotency and Fail Closed Check 34 Per Function Normalized Body Uniqueness";
    }
    doc /* - **Given**: This blueprint specification docs/designs/blueprint-05-deterministic-safety-docstring-synthesizer.md. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-05-deterministic-safety-docstring-synthesizer.md --dry-run and ./target/release/ingest-sysml --schema docs/designs/blueprint-05-deterministic-safety-docstring-synthesizer.md --format markdown are executed. - **Then**: The translator parses **REQ-0605**, UnifiedSafetyDocstringSynthesizer, all four BOM subcomponents, all four interface ports, all four state attributes, all four parametric. */
    use case def UC_AC_05_Dogfood_Ingestion_and_SysML_v2_Compilation_of_REQ_0605 {
        subject Check36AstCompletenessVerifier;
        objective "Dogfood Ingestion and SysML v2 Compilation of REQ 0605";
    }
    doc /* Extracted from Mermaid sequenceDiagram specification. */
    interaction def BP_05_Unified_AST_Safety_Docstring_Synthesizer_and_Check_36_Verification_Blueprint_SequenceInteraction {
        lifeline CLI : Compiler_CLI_CI_Pipeline;
        lifeline Core : deap_core_source_ast;
        lifeline Synth : compile_sysml_docstring;
        lifeline Gate : verify_baseline_docstring_gate;
        lifeline Ingest : ingest_sysml;
        message RunCompileSysmlInjectSafetyDocstrings from CLI to Synth;
        message ScanSourceLinesViaRustDartLexStateAnd from Synth to Core;
        message reply_ReturnDeclarationItemsOwnerNameParams from Core to Synth;
        message ResolveSchemaContextViaCollectAll from Synth to Synth;
        message reply_Write7SectionDocstringsAtomicallyWith from Synth to CLI;
        message RunVerifyBaselineNoDomain from CLI to Gate;
        message TokenizeRsAndDartFilesUsing from Gate to Core;
        message reply_ReturnScopedProductionItemsExcludingMod from Core to Gate;
        message reply_Verify7SectionsNonEmptyRealises from Gate to CLI;
        message RunIngestSysmlSchemaBlueprint05 from CLI to Ingest;
        message reply_EmitCanonicalSysMLV2AstFor from Ingest to CLI;
    }
    doc /* Baseline verification gate in `verify-baseline` auditing 100% per-item 7-section docstring coverage and Check 34 normalized body uniqueness using `deap_core::source_ast` */
    part def Check36AstCompletenessVerifier {
        attribute quantity : Integer = 1;
        attribute type : String = "Component";
    }
    doc /* Strict SysML v2 requirement and constraint resolver in `compile-sysml` traversing recursive `part.parts` trees with `owner_name` isolation and zero `.first()` fallback */
    part def ScopedSchemaContextResolver {
        attribute quantity : Integer = 1;
        attribute type : String = "Component";
    }
    doc /* Deterministic forward synthesizer interpolating AST parameters, return types, struct fields, invariants, and `Realises:` bindings into unique 7-section docblocks */
    part def SevenSectionDocstringGenerator {
        attribute quantity : Integer = 1;
        attribute type : String = "Component";
    }
    doc /* Canonical zero-regex lexical state machine in `deap_core::source_ast` handling nested `/ * * /`, `r#"..."#`, Dart `${...}` frames, and exact char-literal vs. lifetime disambiguation */
    part def UnifiedRustDartLexicalScanner {
        attribute quantity : Integer = 1;
        attribute type : String = "Component";
    }
    part def UnifiedSafetyDocstringSynthesizer {
    }
}