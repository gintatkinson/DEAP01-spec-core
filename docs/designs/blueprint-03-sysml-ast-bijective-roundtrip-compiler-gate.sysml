doc /* Translated from Level 0 OEM Markdown specification */
package BP_03_SysML_v2_Lossless_Red_Green_CST_Salsa_Incremental_Query_DB_and_Bijective_Roundtrip_Compiler_Gate_Blueprint {
    attribute Blueprint_ID : String = "BP-03";
    attribute Requirement_ID : String = "REQ-0603";
    attribute Title : String = "SysML v2 Lossless Red/Green CST, Salsa Incremental Query DB, and Bijective Roundtrip Compiler Gate Blueprint (DEAP-BP-DET-03)";
    attribute Status : String = "Approved";
    attribute Target_Crates : String = "crates/compile-sysml, crates/deap-core, crates/ingest-sysml, crates/verify-baseline";
    attribute Tracked_Issues : String = "#493, #495, #496, #497, #504, #505, #509";
    attribute Generation_Mode : String = "subagent";
    attribute Specification_Source : String = "crates/compile-sysml/src/parser/grammar/mod.rs:182-335 / crates/deap-core/src/sysml_ast/mod.rs:1-400 / #493";
    doc /* `SysmlSyntaxKind`, `SysmlLanguage`, `CstParseOutput`, `SysmlHirPackage`, `SourceInput`, and `BijectiveDigestReceipt` in `deap_core::sysml_cst` and `compile_sysml::salsa_db` with `camino::Utf8PathBuf` paths andpinned `rowan`. */
    attribute Layer_1_Domain_State_Data_Model : String = "Lossless Red/Green CST, typed HIR, and incremental query inputs";
    doc /* `normalize_line_endings`, `parse_sysml_cst`, `lower_sysml_hir`, `emit_canonical_sysml`, and `compute_bijective_digest` enforcing fail-closed `E0100..E0400` diagnostics and cross-platform SHA-256 digest invariance. */
    attribute Layer_2_Logic_State_Management : String = "Deterministic `logos` + `rowan` parsing, `salsa` memoized lowering, and Stage-1 == Stage-2 bijective verification";
    doc /* `compile-sysml`, `ingest-sysml`, and `verify-baseline check-blueprint` CLI actuators emitting deterministic diagnostics, canonical `.sysml` models, and `.digest.json` receipts (`ra_ap_syntax`, `deap_core::dart_cst`, `serde_json`, `logos`+`rowan` IDL/YANG). */
    attribute Layer_3_Presentation_Actuator_Interface_Binding : String = "CLI compiler gate output, persisted `.sysml` / `.digest.json` artifacts, and polyglot ingestion CLI actuators";
    doc /* [unit: `bytes`] */
    attribute span_offset_drift_bytes : Real;
    doc /* [unit: `branches`] */
    attribute silent_skip_branch_count : Real;
    doc /* [unit: `fields`] */
    attribute dropped_ast_field_count : Real;
    doc /* [unit: `nodes`] */
    attribute roundtrip_ast_mismatch_count : Real;
    doc /* normalize_line_endings(raw_source) */
    action def normalize_line_endings;
    doc /* compute_bijective_digest(db, input) */
    action def compute_bijective_digest;
    doc /* parse_sysml_cst(db, input) */
    action def parse_sysml_cst;
    doc /* lex_sysml_tokens(source) */
    action def lex_sysml_tokens;
    doc /* lower_sysml_hir(db, input) */
    action def lower_sysml_hir;
    doc /* emit_canonical_sysml(db, input) */
    action def emit_canonical_sysml;
    doc /* sha256_hex_digest(canonical_sysml) */
    action def sha256_hex_digest;
    doc /* [unit: `bytes`] */
    assert constraint C_RT_01 {
        span_offset_drift_bytes >= 0 and span_offset_drift_bytes <= 0;
    }
    doc /* [unit: `branches`] */
    assert constraint C_RT_02 {
        silent_skip_branch_count >= 0 and silent_skip_branch_count <= 0;
    }
    doc /* [unit: `fields`] */
    assert constraint C_RT_03 {
        dropped_ast_field_count >= 0 and dropped_ast_field_count <= 0;
    }
    doc /* [unit: `nodes`] */
    assert constraint C_RT_04 {
        roundtrip_ast_mismatch_count >= 0 and roundtrip_ast_mismatch_count <= 0;
    }
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0603_BP_03_SysML_v2_Lossless_Red_Green_CST_Salsa_Incremental_Query_DB_and_Bijective_Roundtrip_Compiler_Gate_Blueprint {
        id = "REQ-0603";
        text = "The Digital Engineering Agent Platform (DEAP) compiler pipeline (DEAP-BP-DET-03, subsystem SysmlBijectiveRoundtripCompilerGate, deterministic UUIDv5 6ba7b810-9dad-11d1-80b4-00c04fd430c3, complexity class Class P, diagnostic code bindings E0100, E0200, E0300, E0400, governed by ISO/IEC/IEEE";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_lossless_logos_rowan_red_green_cst_construction_pinned_rowan_parity : String = "- **Given**: A SysML v2 model containing doc / * ... * / blocks, line comments, reference/optionality sigils (&, ?), lifetimes ('a), qualified paths (::), and mixed CRLF/LF line endings (#495). - **When**: normalize_line_endings and parse_sysml_cst execute via logos::Logos and rowan::GreenNodeBuilder (with workspace rowan pinned identically to ra_ap_syntax's rowan). - **Then**: The resulting SyntaxNode<SysmlLanguage> preserves 100% of trivia, doc comments, and exact UTF-8 byte spans (span_offset_drift_bytes == 0) with zero pre-lexer string rewriting.";
        attribute ac_02_fail_closed_typed_cst_lowering_without_silent_skips_or_text_heuristics : String = "- **Given**: A SysML v2 declaration block (port, action, requirement, state, connection, use case, item, hazard, risk, test case, capability, interaction) containing composite generic types, anonymous transitions (first S1 accept trig if grd do eff then S2;), nested sub_states, or an invalid inner token (#504, #496, #509). - **When**: lower_sysml_hir traverses typed rowan::ast::AstNode wrappers over the Red/Green CST. - **Then**: Valid declarations lower losslessly preserving name: None on unnamed transitions and recursive sub_states, zero .syntax().text().contains(...) calls exist, and any.";
        attribute ac_03_demand_driven_salsa_incremental_query_memoization_cross_platform_sha_256_parity : String = "- **Given**: Identical SysML v2 models ingested from LF (macOS/Linux) and CRLF (Windows) files via camino::Utf8Path into compile_sysml::salsa_db::SourceInput, or a model with a simulated Stage-2 field mutation (#493, #497, #505). - **When**: compute_bijective_digest evaluates parse_sysml_cst -> lower_sysml_hir -> emit_canonical_sysml -> Stage-2 re-parse and lowering. - **Then**: Unchanged SourceInput revisions reuse memoized salsa query outputs, LF and CRLF inputs produce bit-identical BijectiveDigestReceipt SHA-256 hex digests with Stage-1 == Stage-2 (roundtrip_ast_mismatch_count == 0), and.";
        attribute ac_04_cohesive_polyglot_typed_cst_ast_ingestion_legacy_scanner_retirement : String = "- **Given**: Polyglot engineering inputs across Rust (*.rs), Dart (*.dart), OpenAPI / JSON-Schema (*.json), OMG IDL (*.idl), and IETF YANG (*.yang), alongside legacy scanner deletion targets in ./crates/deap-core/src/sysml_ast/mod.rs (-2,400+ SLOC) and ./crates/ingest-sysml/src/translators/{code,idl,yang,openapi}.rs (-1,800+ SLOC). - **When**: ingest-sysml and validate_polyglot_frontend bind each input format to its canonical typed parser frontend. - **Then**: Rust parses via ra_ap_syntax, Dart via deap_core::dart_cst (logos + rowan), JSON-Schema/OpenAPI via typed serde_json ASTs, and.";
        attribute ac_05_dogfood_blueprint_to_sysml_verification_12_gate_closure : String = "- **Given**: This blueprint document ./docs/designs/blueprint-03-sysml-ast-bijective-roundtrip-compiler-gate.md (**REQ-0603**). - **When**: ./target/release/verify-baseline check-blueprint ./docs/designs/blueprint-03-sysml-ast-bijective-roundtrip-compiler-gate.md and ./target/release/verify-baseline . --no-domain are executed. - **Then**: All 12 mechanical blueprint verification gates pass (12/12), docs/designs/blueprint-03-sysml-ast-bijective-roundtrip-compiler-gate.sysml and .digest.json are deterministically persisted, and workspace baseline verification exits with status code 0.";
        doc /* \begin{aligned} \forall s \in \mathcal{L}_{\text{raw}}: \quad \text{text}(P_{\text{cst}}(N_{\text{LF}}(s))) &= N_{\text{LF}}(s) \\ \forall s \in \mathcal{L}_{\text{raw}}, \quad \mathcal{H}_1 = L_{\text{hir}}(P_{\text{cst}}(N_{\text{LF}}(s))), \quad \mathcal{H}_2 &= L_{\text{hir}}(P_{\text{cst}}(E_{\text{sysml}}(\mathcal{H}_1))) \implies \mathcal{H}_1 = \mathcal{H}_2 \\ D_{256}(E_{\text{sysml}}(\mathcal{H}_1)) &= D_{256}(E_{\text{sysml}}(\mathcal{H}_2)) \\ \Delta_{\text{span}}(t_k) &= \text{byte\_offset}_{\text{rowan}}(t_k) - \text{byte\_offset}_{\text{LF}}(t_k) = 0 \\ T_{\text{salsa\_incremental}}(\Delta N, N) &= O(\Delta N), \qquad T_{\text{cold}}(N) = O(N) \end{aligned} */
        constraint def Invariant_REQ_0603_Formal_Lossless_CST_Salsa_Memoization_Bijective_Digest_Invariants;
        doc /* \begin{aligned} \text{AdmissibleCompile}(s) &\iff \Delta_{\text{span}} = 0 \land N_{\text{skip}} = 0 \land N_{\text{dropped}} = 0 \land (\mathcal{H}_1 = \mathcal{H}_2) \land \bigl(D_{256}(E_{\text{sysml}}(\mathcal{H}_1)) = D_{256}(E_{\text{sysml}}(\mathcal{H}_2))\bigr) \end{aligned} */
        constraint def Invariant_REQ_0603_SC_03;
        require Invariant_REQ_0603_Formal_Lossless_CST_Salsa_Memoization_Bijective_Digest_Invariants;
        require Invariant_REQ_0603_SC_03;
        verify by AC_01_Lossless_logos_rowan_Red_Green_CST_Construction_Pinned_rowan_Parity;
        verify by AC_02_Fail_Closed_Typed_CST_Lowering_Without_Silent_Skips_or_Text_Heuristics;
        verify by AC_03_Demand_Driven_salsa_Incremental_Query_Memoization_Cross_Platform_SHA_256_Parity;
        verify by AC_04_Cohesive_Polyglot_Typed_CST_AST_Ingestion_Legacy_Scanner_Retirement;
        verify by AC_05_Dogfood_Blueprint_to_SysML_Verification_12_Gate_Closure;
        satisfy by BijectiveSha256GateVerifier;
    }
    doc /* Extracted from Mermaid stateDiagram specification. */
    state def BP_03_SysML_v2_Lossless_Red_Green_CST_Salsa_Incremental_Query_DB_and_Bijective_Roundtrip_Compiler_Gate_Blueprint_LifecycleState {
        state NormalizingSourceInput;
        state LogosRowanCstParsing;
        state TypedHirLowering;
        state RejectedE0100;
        state CanonicalSysmlPrinting;
        state RejectedE0200;
        state Stage2BijectiveReparse;
        state Sha256DigestSealing;
        state RejectedE0400;
        state VerifiedBijectiveDigest;
        transition Initial_to_NormalizingSourceInput first Initial then NormalizingSourceInput;
        transition NormalizingSourceInput_to_LogosRowanCstParsing_on_crlfNormalizedToLf first NormalizingSourceInput accept crlfNormalizedToLf then LogosRowanCstParsing;
        transition LogosRowanCstParsing_to_TypedHirLowering_on_greenNodeLosslessSpanExact first LogosRowanCstParsing accept greenNodeLosslessSpanExact then TypedHirLowering;
        transition LogosRowanCstParsing_to_RejectedE0100_on_logosLexicalOrSyntaxError first LogosRowanCstParsing accept logosLexicalOrSyntaxError then RejectedE0100;
        transition TypedHirLowering_to_CanonicalSysmlPrinting_on_stage1HirLowered first TypedHirLowering accept stage1HirLowered then CanonicalSysmlPrinting;
        transition TypedHirLowering_to_RejectedE0200_on_unexpectedTypedAstNode first TypedHirLowering accept unexpectedTypedAstNode then RejectedE0200;
        transition CanonicalSysmlPrinting_to_Stage2BijectiveReparse_on_canonicalLfTextEmitted first CanonicalSysmlPrinting accept canonicalLfTextEmitted then Stage2BijectiveReparse;
        transition Stage2BijectiveReparse_to_Sha256DigestSealing_on_stage1HirEqualsStage2Hir first Stage2BijectiveReparse accept stage1HirEqualsStage2Hir then Sha256DigestSealing;
        transition Stage2BijectiveReparse_to_RejectedE0400_on_stage2DivergenceDetected first Stage2BijectiveReparse accept stage2DivergenceDetected then RejectedE0400;
        transition Sha256DigestSealing_to_VerifiedBijectiveDigest_on_sha256ReceiptPersisted first Sha256DigestSealing accept sha256ReceiptPersisted then VerifiedBijectiveDigest;
        transition RejectedE0100_to_Terminal first RejectedE0100 then Terminal;
        transition RejectedE0200_to_Terminal first RejectedE0200 then Terminal;
        transition RejectedE0400_to_Terminal first RejectedE0400 then Terminal;
        transition VerifiedBijectiveDigest_to_Terminal first VerifiedBijectiveDigest then Terminal;
    }
    doc /* - **Given**: A SysML v2 model containing doc / * ... * / blocks, line comments, reference/optionality sigils (&, ?), lifetimes ('a), qualified paths (::), and mixed CRLF/LF line endings (#495). - **When**: normalize_line_endings and parse_sysml_cst execute via logos::Logos and rowan::GreenNodeBuilder (with workspace rowan pinned identically to ra_ap_syntax's rowan). - **Then**: The resulting SyntaxNode<SysmlLanguage> preserves 100% of trivia, doc comments, and exact UTF-8 byte spans (span_offset_drift_bytes == 0) with zero pre-lexer string rewriting. */
    use case def UC_AC_01_Lossless_logos_rowan_Red_Green_CST_Construction_Pinned_rowan_Parity {
        subject BijectiveSha256GateVerifier;
        objective "Lossless logos rowan Red Green CST Construction Pinned rowan Parity";
    }
    doc /* - **Given**: A SysML v2 declaration block (port, action, requirement, state, connection, use case, item, hazard, risk, test case, capability, interaction) containing composite generic types, anonymous transitions (first S1 accept trig if grd do eff then S2;), nested sub_states, or an invalid inner token (#504, #496, #509). - **When**: lower_sysml_hir traverses typed rowan::ast::AstNode wrappers over the Red/Green CST. - **Then**: Valid declarations lower losslessly preserving name: None on unnamed transitions and recursive sub_states, zero .syntax().text().contains(...) calls exist, and any. */
    use case def UC_AC_02_Fail_Closed_Typed_CST_Lowering_Without_Silent_Skips_or_Text_Heuristics {
        subject BijectiveSha256GateVerifier;
        objective "Fail Closed Typed CST Lowering Without Silent Skips or Text Heuristics";
    }
    doc /* - **Given**: Identical SysML v2 models ingested from LF (macOS/Linux) and CRLF (Windows) files via camino::Utf8Path into compile_sysml::salsa_db::SourceInput, or a model with a simulated Stage-2 field mutation (#493, #497, #505). - **When**: compute_bijective_digest evaluates parse_sysml_cst -> lower_sysml_hir -> emit_canonical_sysml -> Stage-2 re-parse and lowering. - **Then**: Unchanged SourceInput revisions reuse memoized salsa query outputs, LF and CRLF inputs produce bit-identical BijectiveDigestReceipt SHA-256 hex digests with Stage-1 == Stage-2 (roundtrip_ast_mismatch_count == 0), and. */
    use case def UC_AC_03_Demand_Driven_salsa_Incremental_Query_Memoization_Cross_Platform_SHA_256_Parity {
        subject BijectiveSha256GateVerifier;
        objective "Demand Driven salsa Incremental Query Memoization Cross Platform SHA 256 Parity";
    }
    doc /* - **Given**: Polyglot engineering inputs across Rust (*.rs), Dart (*.dart), OpenAPI / JSON-Schema (*.json), OMG IDL (*.idl), and IETF YANG (*.yang), alongside legacy scanner deletion targets in ./crates/deap-core/src/sysml_ast/mod.rs (-2,400+ SLOC) and ./crates/ingest-sysml/src/translators/{code,idl,yang,openapi}.rs (-1,800+ SLOC). - **When**: ingest-sysml and validate_polyglot_frontend bind each input format to its canonical typed parser frontend. - **Then**: Rust parses via ra_ap_syntax, Dart via deap_core::dart_cst (logos + rowan), JSON-Schema/OpenAPI via typed serde_json ASTs, and. */
    use case def UC_AC_04_Cohesive_Polyglot_Typed_CST_AST_Ingestion_Legacy_Scanner_Retirement {
        subject BijectiveSha256GateVerifier;
        objective "Cohesive Polyglot Typed CST AST Ingestion Legacy Scanner Retirement";
    }
    doc /* - **Given**: This blueprint document ./docs/designs/blueprint-03-sysml-ast-bijective-roundtrip-compiler-gate.md (**REQ-0603**). - **When**: ./target/release/verify-baseline check-blueprint ./docs/designs/blueprint-03-sysml-ast-bijective-roundtrip-compiler-gate.md and ./target/release/verify-baseline . --no-domain are executed. - **Then**: All 12 mechanical blueprint verification gates pass (12/12), docs/designs/blueprint-03-sysml-ast-bijective-roundtrip-compiler-gate.sysml and .digest.json are deterministically persisted, and workspace baseline verification exits with status code 0. */
    use case def UC_AC_05_Dogfood_Blueprint_to_SysML_Verification_12_Gate_Closure {
        subject BijectiveSha256GateVerifier;
        objective "Dogfood Blueprint to SysML Verification 12 Gate Closure";
    }
    doc /* Extracted from Mermaid sequenceDiagram specification. */
    interaction def BP_03_SysML_v2_Lossless_Red_Green_CST_Salsa_Incremental_Query_DB_and_Bijective_Roundtrip_Compiler_Gate_Blueprint_SequenceInteraction {
        lifeline CLI : Compiler_CLI_Caller;
        lifeline DB : SalsaIncrementalQueryDb;
        lifeline CST : LogosRowanSysmlCstFrontend;
        lifeline HIR : TypedHirLowererAndPrinter;
        lifeline Gate : BijectiveSha256GateVerifier;
        message normalize_line_endings from CLI to CST;
        message reply_LfNormalizedUtf8Source from CST to CLI;
        message compute_bijective_digest from CLI to DB;
        message parse_sysml_cst from DB to CST;
        message lex_sysml_tokens from CST to CST;
        message reply_CstParseOutput from CST to DB;
        message lower_sysml_hir from DB to HIR;
        message reply_Stage1SysmlHirPackage from HIR to DB;
        message emit_canonical_sysml from DB to HIR;
        message reply_Stage1CanonicalSysMLV2Text from HIR to DB;
        message reply_Stage2CstParseOutput from CST to DB;
        message reply_Stage2SysmlHirPackage from HIR to DB;
        message sha256_hex_digest from DB to Gate;
        message reply_DeterministicSha256HexDigest from Gate to DB;
        message reply_Ok from DB to CLI;
    }
    doc /* Cross-platform CRLF-normalized Stage-1 == Stage-2 HIR equality and SHA-256 semantic digest gate */
    part def BijectiveSha256GateVerifier {
        attribute quantity : Integer = 1;
        attribute type : String = "Subcomponent";
    }
    doc /* `logos::Logos` lexer and `rowan::GreenNodeBuilder` lossless Red/Green CST builder preserving trivia, doc comments, and exact spans */
    part def LogosRowanSysmlCstFrontend {
        attribute quantity : Integer = 1;
        attribute type : String = "Subcomponent";
    }
    doc /* Unified polyglot frontend binding `ra_ap_syntax` (Rust), `deap_core::dart_cst` (Dart), typed `serde_json` (OpenAPI/JSON-Schema), and `logos`+`rowan` (OMG IDL / IETF YANG) */
    part def PolyglotTypedCstIngestor {
        attribute quantity : Integer = 1;
        attribute type : String = "Subcomponent";
    }
    doc /* Demand-driven `salsa` query engine memoizing `parse_sysml_cst`, `lower_sysml_hir`, `emit_canonical_sysml`, and `compute_bijective_digest` */
    part def SalsaIncrementalQueryDb {
        attribute quantity : Integer = 1;
        attribute type : String = "Subcomponent";
    }
    doc /* Cumulative byte-offset drift between normalized source bytes and `rowan` GreenNode token spans [unit: bytes] */
    part def SysmlBijectiveRoundtripCompilerGate {
        attribute span_offset_drift_bytes : Integer = 0;
        doc /* Count of silent token-skip fallback branches during CST parsing and typed HIR lowering [unit: branches] */
        attribute silent_skip_branch_count : Integer = 0;
        doc /* Count of HIR semantic fields or docstrings omitted during `emit_canonical_sysml` printing [unit: fields] */
        attribute dropped_ast_field_count : Integer = 0;
        doc /* Count of HIR or canonical CST nodes differing between Stage-1 and Stage-2 bijective compilation [unit: nodes] */
        attribute roundtrip_ast_mismatch_count : Integer = 0;
        doc /* Ingests UTF-8 SysML v2 or polyglot CST-lowered source via `camino::Utf8Path` with CRLF-to-LF normalization [protocol: SalsaSourceInput] */
        port in RawSysmlSourceInputPort : String {
            flow in RawSysmlSourceInputPort_flow : String;
        }
        doc /* Streams immutable `rowan` Red/Green CST trees (`SyntaxNode~SysmlLanguage~`) across `salsa` query stages [protocol: RowanGreenNode] */
        port inout LosslessRedGreenCstPort : String;
        doc /* Emits canonical LF-normalized SysML v2 text synthesized by `emit_canonical_sysml` [protocol: CanonicalSysmlV2] */
        port out SerializedSysmlOutputPort : String {
            flow out SerializedSysmlOutputPort_flow : String;
        }
        doc /* Emits SHA-256 `BijectiveDigestReceipt` or fail-closed `E0100..E0400` structural diff diagnostics [protocol: StructuredDiagnostic] */
        port out BijectiveDiffDiagnosticPort : String {
            flow out BijectiveDiffDiagnosticPort_flow : String;
        }
    }
}