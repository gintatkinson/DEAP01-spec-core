doc /* Translated from Level 0 OEM Markdown specification */
package SysML_v2_AST_Bijective_Roundtrip_and_Token_Stream_Parser_Gate_Blueprint {
    attribute Blueprint_ID : String = "BP-03";
    attribute Requirement_ID : String = "REQ-0603";
    attribute Title : String = "SysML v2 AST Bijective Roundtrip and Token-Stream Parser Gate Blueprint (DEAP-BP-DET-03)";
    attribute Status : String = "Approved";
    attribute Target_Crates : String = "crates/compile-sysml, crates/deap-core, crates/ingest-sysml, crates/verify-baseline";
    attribute Tracked_Issues : String = "#493, #495, #496, #497, #504, #505, #509";
    attribute Generation_Mode : String = "subagent";
    attribute Specification_Source : String = "crates/compile-sysml/src/parser/grammar.rs:182-335 / crates/compile-sysml/src/semantic/serializer.rs:250-820 / #493";
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0603_SysML_v2_AST_Bijective_Roundtrip_and_Token_Stream_Parser_Gate_Blueprint {
        id = "REQ-0603";
        text = "The Digital Engineering Agent Platform (DEAP) compiler pipeline (DEAP-BP-DET-03, subsystem SysmlBijectiveRoundtripCompilerGate, deterministic UUIDv5 6ba7b810-9dad-11d1-80b4-00c04fd430c3, complexity class Class P, diagnostic code bindings E0100, E0200, E0300, E0400, governed by ISO/IEC/IEEE";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_native_lexer_sigil_and_lifetime_tokenization : String = "- **Given**: A SysML v2 model containing reference type sigils (&), optionality sigils (?), lifetime annotations ('a), qualified paths (::), and logical && expressions (#495). - **When**: Scanner::scan_all tokenizes the raw source buffer without invoking normalize_type_sigils. - **Then**: The lexer emits TokenKind::Ampersand, TokenKind::Question, TokenKind::Lifetime, TokenKind::DoubleColon, and TokenKind::AmpAmp with exact 1:1 byte and column offsets (span_offset_drift_bytes == 0).";
        attribute ac_02_fail_closed_inner_block_and_composite_type_parsing : String = "- **Given**: Any of the 12 SysML v2 inner declaration blocks (port, action, requirement, state, connection, use case, item, hazard, risk, test case, capability, interaction) containing either composite parameter types (Result<PackageDef, ParseError>, &'a [Token]) or an invalid inner statement (#504). - **When**: SysmlParser parses the declaration block. - **Then**: Valid composite types are preserved verbatim via parse_type_expr() and any unrecognized statement fails closed and is rejected with Err(ParseError) reporting diagnostic code E0200 (silent_skip_branch_count == 0).";
        attribute ac_03_token_stream_state_transition_parsing_and_recursive_mbd_synthesis : String = "- **Given**: A hierarchical state def containing anonymous and named transition statements (first S1 accept trig if grd do eff then S2;) and nested composite sub_states with their own transitions and doc blocks (#496, #509). - **When**: parse_state_decl and the MBD .sfx synthesizer execute. - **Then**: Transitions are parsed directly from the token stream without fabricating {s}_to_{t} names (name: None preserved when unnamed), sub-state doc blocks attach to their target StateDef or TransitionDef, and MBD .sfx synthesis recursively emits transitions across all nesting depths.";
        attribute ac_04_lossless_to_sysml_serialization_and_fail_closed_bijective_roundtrip_gate : String = "- **Given**: A populated PackageDef AST containing StateDef transitions (source, trigger, guard, effect, target, doc), PortDef / HazardDef / RiskDef attributes, and UseCaseDef / ConnectionDef / RequirementDef / ActionDef blocks (#493, #497, #505), or a mutated AST with a simulated field divergence. - **When**: verify_bijective_ast_roundtrip(&pkg) executes during compile-sysml. - **Then**: pkg.to_sysml(0) emits valid un-double-quoted SysML v2 text, SysmlParser::parse_source(&pkg.to_sysml(0), &pkg.name) returns a PackageDef structurally equal to pkg (roundtrip_ast_mismatch_count == 0), and any.";
        attribute ac_05_dogfood_markdown_to_sysml_ingestion_and_baseline_verification : String = "- **Given**: This blueprint document docs/designs/blueprint-03-sysml-ast-bijective-roundtrip-compiler-gate.md (**REQ-0603**). - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-03-sysml-ast-bijective-roundtrip-compiler-gate.md --dry-run is executed. - **Then**: verify-baseline check-blueprint passes all 12 mechanical gates, extracts SysmlBijectiveRoundtripCompilerGate with all 4 BOM parts, 4 interface ports, 4 attributes, 4 parametric constraints (C-RT-01..C-RT-04), and REQ_0603 requirement definitions, persists.";
        doc /* \begin{aligned} \forall \mathcal{A} \in \text{PackageDef}: \quad \text{parse\_sysml}(\text{to\_sysml}(\mathcal{A})) &= \mathcal{A} \\ \forall s \in \mathcal{L}: \quad \text{to\_sysml}(\text{parse\_sysml}(s)) &= \text{to\_sysml}(\text{parse\_sysml}(\text{to\_sysml}(\text{parse\_sysml}(s)))) \\ \Delta_{\text{span}}(t_k) &= \text{byte\_offset}_{\text{lexer}}(t_k) - \text{byte\_offset}_{\text{raw}}(t_k) = 0 \\ T_{\text{roundtrip}}(N) &= O(N) \\ S_{\text{roundtrip}}(N) &= O(N) \end{aligned} */
        constraint def Invariant_REQ_0603_Formal_Bijectivity_Complexity_Invariants;
        doc /* \begin{aligned} \text{AdmissibleCompile}(\mathcal{A}) &\iff \Delta_{\text{span}} = 0 \land N_{\text{skip}} = 0 \land N_{\text{dropped}} = 0 \land d_{\text{AST}}(\mathcal{A}, \text{parse\_sysml}(\text{to\_sysml}(\mathcal{A}))) = 0 \end{aligned} */
        constraint def Invariant_REQ_0603_SC_04;
        require Invariant_REQ_0603_Formal_Bijectivity_Complexity_Invariants;
        require Invariant_REQ_0603_SC_04;
        verify by AC_01_Native_Lexer_Sigil_and_Lifetime_Tokenization;
        verify by AC_02_Fail_Closed_Inner_Block_and_Composite_Type_Parsing;
        verify by AC_03_Token_Stream_State_Transition_Parsing_and_Recursive_MBD_Synthesis;
        verify by AC_04_Lossless_to_sysml_Serialization_and_Fail_Closed_Bijective_Roundtrip_Gate;
        verify by AC_05_Dogfood_Markdown_to_SysML_Ingestion_and_Baseline_Verification;
        satisfy by NativeSigilAndLifetimeLexer;
    }
    doc /* Zero-rewrite scanner emitting native `Ampersand`, `Question`, `Lifetime`, and `DoubleColon` tokens with exact byte spans */
    part def NativeSigilAndLifetimeLexer {
        attribute quantity : Integer = 1;
        attribute type : String = "Subcomponent";
    }
    doc /* Strict token-stream parser for all 12 inner declaration blocks, composite type expressions, and nested state transitions */
    part def FailClosedRecursiveDescentParser {
        attribute quantity : Integer = 1;
        attribute type : String = "Subcomponent";
    }
    doc /* Canonical `SysmlSerializable::to_sysml` formatter preserving 100% of `PackageDef` AST fields without double-quoting */
    part def BijectiveSysmlSerializer {
        attribute quantity : Integer = 1;
        attribute type : String = "Subcomponent";
    }
    doc /* Built-in compiler verification stage asserting `parse_sysml(to_sysml(ast)) == ast` on every `compile-sysml` run */
    part def CompilerRoundtripVerificationGate {
        attribute quantity : Integer = 1;
        attribute type : String = "Subcomponent";
    }
    doc /* --- | Cumulative byte-offset drift between raw source character positions and emitted token spans [unit: bytes] */
    part def SysmlBijectiveRoundtripCompilerGate {
        attribute span_offset_drift_bytes : Integer = 0;
        doc /* Count of `skip_until_semi_or_brace` silent erasure fallback branches in inner block parsers [unit: branches] */
        attribute silent_skip_branch_count : Integer = 0;
        doc /* Count of AST struct fields omitted during `SysmlSerializable::to_sysml` serialization [unit: fields] */
        attribute dropped_ast_field_count : Integer = 0;
        doc /* Count of structural AST nodes differing between `ast` and `parse_sysml(to_sysml(ast))` [unit: nodes] */
        attribute roundtrip_ast_mismatch_count : Integer = 0;
        doc /* Ingests unmodified UTF-8 SysML v2 source text without pre-lexer string normalization [protocol: UTF8Buffer] */
        port inout RawSysmlSourceInputPort : String;
        doc /* Streams span-exact `Token` records from `NativeSigilAndLifetimeLexer` to `FailClosedRecursiveDescentParser` [protocol: ZeroCopySlice] */
        port inout TokenStreamChannelPort : String;
        doc /* Emits deterministic, human-readable SysML v2 text synthesized by `BijectiveSysmlSerializer` [protocol: CanonicalSysmlV2] */
        port inout SerializedSysmlOutputPort : String;
        doc /* Emits fail-closed structural AST diff and span location diagnostics (`E0100`..`E0400`) on roundtrip mismatch [protocol: StructuredDiagnostic] */
        port inout BijectiveDiffDiagnosticPort : String;
        doc /* [unit: `bytes`] */
        assert constraint assert_span_offset_drift_bytes_range {
            span_offset_drift_bytes >= 0 and span_offset_drift_bytes <= 0;
        }
        doc /* [unit: `branches`] */
        assert constraint assert_silent_skip_branch_count_range {
            silent_skip_branch_count >= 0 and silent_skip_branch_count <= 0;
        }
        doc /* [unit: `fields`] */
        assert constraint assert_dropped_ast_field_count_range {
            dropped_ast_field_count >= 0 and dropped_ast_field_count <= 0;
        }
        doc /* [unit: `nodes`] */
        assert constraint assert_roundtrip_ast_mismatch_count_range {
            roundtrip_ast_mismatch_count >= 0 and roundtrip_ast_mismatch_count <= 0;
        }
    }
}