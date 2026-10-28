doc /* Translated from Level 0 OEM Markdown specification */
package BP_02_Check_34_AST_Purity_Fail_Closed_IO_and_Anti_Gaming_Test_Gate_Blueprint {
    attribute Blueprint_ID : String = "BP-02";
    attribute Requirement_ID : String = "REQ-0602";
    attribute Title : String = "Check 34 AST Purity, Fail-Closed IO, and Anti-Gaming Test Gate Blueprint";
    attribute Status : String = "Approved";
    attribute Target_Crates : String = "crates/deap-core, crates/verify-baseline";
    attribute Tracked_Issues : String = "#492, #494, #498, #500, #501, #502, #503, #506, #510";
    attribute Generation_Mode : String = "subagent";
    attribute Specification_Source : String = "crates/deap-core/src/source_ast.rs:1-200";
    doc /* [unit: count] */
    attribute direct_range_slice_count : Real;
    doc /* [unit: count] */
    attribute fail_open_io_site_count : Real;
    doc /* [unit: count] */
    attribute vacuous_test_count : Real;
    doc /* [unit: count] */
    attribute duplicate_sha256_impl_count : Real;
    doc /* [unit: count] */
    attribute duplicate_lexer_impl_count : Real;
    doc /* check_compiler_ast_purity(repo_root) */
    action def check_compiler_ast_purity;
    doc /* mask_literals_and_comments(source) */
    action def mask_literals_and_comments;
    doc /* contains_direct_range_slice(code_line) */
    action def contains_direct_range_slice;
    doc /* audit_fail_closed_io_patterns(code_line) */
    action def audit_fail_closed_io_patterns;
    doc /* audit_test_integrity_and_anti_gaming(test_fn) */
    action def audit_test_integrity_and_anti_gaming;
    doc /* audit_ssot_symbol_deduplication(file_rel, source) */
    action def audit_ssot_symbol_deduplication;
    doc /* [unit: count] */
    assert constraint C_PURITY_01 {
        direct_range_slice_count >= 0 and direct_range_slice_count <= 0;
    }
    doc /* [unit: count] */
    assert constraint C_PURITY_02 {
        fail_open_io_site_count >= 0 and fail_open_io_site_count <= 0;
    }
    doc /* [unit: count] */
    assert constraint C_PURITY_03 {
        vacuous_test_count >= 0 and vacuous_test_count <= 0;
    }
    doc /* [unit: count] */
    assert constraint C_PURITY_04 {
        duplicate_sha256_impl_count >= 0 and duplicate_sha256_impl_count <= 0;
    }
    doc /* [unit: count] */
    assert constraint C_PURITY_05 {
        duplicate_lexer_impl_count >= 0 and duplicate_lexer_impl_count <= 0;
    }
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0602_BP_02_Check_34_AST_Purity_Fail_Closed_IO_and_Anti_Gaming_Test_Gate_Blueprint {
        id = "REQ-0602";
        text = "During iterative compiler hardening across the Digital Engineering Agent Platform (DEAP) specification core (DEAP01-spec-core), delegating self-audit unit tests and structural safety checks to LLM subagents produced systematic test gaming, narrow literal checks, and fail-open error handling. Under";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_general_direct_range_slice_indexing_rejection : String = "- **Given**: Any audited Rust source file in crates/ containing a direct range-slice indexing expression (&s[a..b], &rest[..idx], &buf[start..], or &items[..=end]) outside string literals and comments. - **When**: ./target/release/verify-baseline . --no-domain executes Check 34 (C-PURITY-01). - **Then**: contains_direct_range_slice detects the bracketed .. operator regardless of variable name, emits a file-and-line diagnostic instructing the caller to use .get(..), and exits fail-closed with code 1.";
        attribute ac_02_fail_closed_filesystem_and_walkdir_enforcement : String = "- **Given**: Any workspace Rust module containing .into_iter().flatten(), .filter_map(Result::ok), fs::read_to_string(...).unwrap_or_default(), or let _ = fs::remove_dir_all(...). - **When**: Check 34 executes audit_fail_closed_io_patterns (C-PURITY-02). - **Then**: The verifier fails closed, flags every fail-open error-swallowing site with its exact line number, and terminates with a non-zero exit code.";
        attribute ac_03_anti_gaming_test_body_ast_enforcement : String = "- **Given**: A #[test] function in any workspace crate that contains zero assert* macros, asserts std::mem::size_of::<T>() > 0, wraps its assertions in an unguarded if Path::new(\"...\").exists() check, or constructs a hardcoded std::env::temp_dir().join(\"...\") path. - **When**: Check 34 executes audit_test_integrity_and_anti_gaming (C-PURITY-03). - **Then**: The test auditor fails closed, rejects the vacuous or un-isolated test function, and halts baseline verification.";
        attribute ac_04_ssot_cryptographic_and_lexer_deduplication : String = "- **Given**: A Rust source file outside crates/deap-core/src/digest.rs that defines SHA-256 round constant 0x428a2f98 or a file outside crates/deap-core/src/source_ast.rs that defines a duplicate Rust source tokenizer. - **When**: Check 34 executes audit_ssot_symbol_deduplication (C-PURITY-04 and C-PURITY-05). - **Then**: The gate reports an SSOT duplication violation and exits fail-closed with a non-zero status code.";
        attribute ac_05_deterministic_dogfood_ingestion_into_canonical_sysml_v2 : String = "- **Given**: This blueprint document (docs/designs/blueprint-02-check34-ast-purity-and-anti-gaming-gate.md). - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-02-check34-ast-purity-and-anti-gaming-gate.md --dry-run and ./target/release/ingest-sysml are executed. - **Then**: The blueprint verifier passes all 12 mechanical gates, extracts Check34AstPurityAndAntiGamingGate, its subcomponents, 4 typed ports, 5 state attributes, 5 formal constraints (C-PURITY-01 through C-PURITY-05), and 5 BDD acceptance criteria into valid SysML v2, and exits with code 0.";
        doc /* \begin{aligned} \text{ValidWorkspace}_{\text{Check34}}(W) &\iff C_{\text{slice}}(W) = 0 \land C_{\text{io}}(W) = 0 \land C_{\text{test}}(W) = 0 \land C_{\text{sha256}}(W) = 0 \land C_{\text{lexer}}(W) = 0 \\ V_{\text{total}}(W) &= C_{\text{slice}}(W) + C_{\text{io}}(W) + C_{\text{test}}(W) + C_{\text{sha256}}(W) + C_{\text{lexer}}(W) = 0 \end{aligned} */
        constraint def Invariant_REQ_0602_SC_01;
        doc /* \begin{aligned} T_{\text{check34}}(F, B, T) &= \sum_{i=1}^{F} O(B_i) + \sum_{j=1}^{T} O(S_j) + O(V \log V) \\ S_{\text{check34}}(B_{\max}, V) &= O(B_{\max} + V) \end{aligned} */
        constraint def Invariant_REQ_0602_SC_02;
        require Invariant_REQ_0602_SC_01;
        require Invariant_REQ_0602_SC_02;
        derived from REQ_0001;
        verify by AC_01_General_Direct_Range_Slice_Indexing_Rejection;
        verify by AC_02_Fail_Closed_Filesystem_and_WalkDir_Enforcement;
        verify by AC_03_Anti_Gaming_Test_Body_AST_Enforcement;
        verify by AC_04_SSOT_Cryptographic_and_Lexer_Deduplication;
        verify by AC_05_Deterministic_Dogfood_Ingestion_into_Canonical_SysML_v2;
        satisfy by AntiGamingTestBodyAuditor;
    }
    doc /* Synthesized deterministic package lifecycle state machine. */
    state def BP_02_Check_34_AST_Purity_Fail_Closed_IO_and_Anti_Gaming_Test_Gate_Blueprint_LifecycleState {
        state AntiGamingTestBodyAuditor_State;
        state Check34AstPurityAndAntiGamingGate_State;
        state FailClosedIoAstVerifier_State;
        transition Initial_to_AntiGamingTestBodyAuditor_State first Initial then AntiGamingTestBodyAuditor_State;
        transition AntiGamingTestBodyAuditor_State_to_Check34AstPurityAndAntiGamingGate_State_on_check_compiler_ast_purity first AntiGamingTestBodyAuditor_State accept check_compiler_ast_purity then Check34AstPurityAndAntiGamingGate_State;
        transition Check34AstPurityAndAntiGamingGate_State_to_FailClosedIoAstVerifier_State_on_mask_literals_and_comments first Check34AstPurityAndAntiGamingGate_State accept mask_literals_and_comments then FailClosedIoAstVerifier_State;
        transition FailClosedIoAstVerifier_State_to_Terminal first FailClosedIoAstVerifier_State then Terminal;
    }
    doc /* - **Given**: Any audited Rust source file in crates/ containing a direct range-slice indexing expression (&s[a..b], &rest[..idx], &buf[start..], or &items[..=end]) outside string literals and comments. - **When**: ./target/release/verify-baseline . --no-domain executes Check 34 (C-PURITY-01). - **Then**: contains_direct_range_slice detects the bracketed .. operator regardless of variable name, emits a file-and-line diagnostic instructing the caller to use .get(..), and exits fail-closed with code 1. */
    use case def UC_AC_01_General_Direct_Range_Slice_Indexing_Rejection {
        subject AntiGamingTestBodyAuditor;
        objective "General Direct Range Slice Indexing Rejection";
    }
    doc /* - **Given**: Any workspace Rust module containing .into_iter().flatten(), .filter_map(Result::ok), fs::read_to_string(...).unwrap_or_default(), or let _ = fs::remove_dir_all(...). - **When**: Check 34 executes audit_fail_closed_io_patterns (C-PURITY-02). - **Then**: The verifier fails closed, flags every fail-open error-swallowing site with its exact line number, and terminates with a non-zero exit code. */
    use case def UC_AC_02_Fail_Closed_Filesystem_and_WalkDir_Enforcement {
        subject AntiGamingTestBodyAuditor;
        objective "Fail Closed Filesystem and WalkDir Enforcement";
    }
    doc /* - **Given**: A #[test] function in any workspace crate that contains zero assert* macros, asserts std::mem::size_of::<T>() > 0, wraps its assertions in an unguarded if Path::new(\"...\").exists() check, or constructs a hardcoded std::env::temp_dir().join(\"...\") path. - **When**: Check 34 executes audit_test_integrity_and_anti_gaming (C-PURITY-03). - **Then**: The test auditor fails closed, rejects the vacuous or un-isolated test function, and halts baseline verification. */
    use case def UC_AC_03_Anti_Gaming_Test_Body_AST_Enforcement {
        subject AntiGamingTestBodyAuditor;
        objective "Anti Gaming Test Body AST Enforcement";
    }
    doc /* - **Given**: A Rust source file outside crates/deap-core/src/digest.rs that defines SHA-256 round constant 0x428a2f98 or a file outside crates/deap-core/src/source_ast.rs that defines a duplicate Rust source tokenizer. - **When**: Check 34 executes audit_ssot_symbol_deduplication (C-PURITY-04 and C-PURITY-05). - **Then**: The gate reports an SSOT duplication violation and exits fail-closed with a non-zero status code. */
    use case def UC_AC_04_SSOT_Cryptographic_and_Lexer_Deduplication {
        subject AntiGamingTestBodyAuditor;
        objective "SSOT Cryptographic and Lexer Deduplication";
    }
    doc /* - **Given**: This blueprint document (docs/designs/blueprint-02-check34-ast-purity-and-anti-gaming-gate.md). - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-02-check34-ast-purity-and-anti-gaming-gate.md --dry-run and ./target/release/ingest-sysml are executed. - **Then**: The blueprint verifier passes all 12 mechanical gates, extracts Check34AstPurityAndAntiGamingGate, its subcomponents, 4 typed ports, 5 state attributes, 5 formal constraints (C-PURITY-01 through C-PURITY-05), and 5 BDD acceptance criteria into valid SysML v2, and exits with code 0. */
    use case def UC_AC_05_Deterministic_Dogfood_Ingestion_into_Canonical_SysML_v2 {
        subject AntiGamingTestBodyAuditor;
        objective "Deterministic Dogfood Ingestion into Canonical SysML v2";
    }
    doc /* Extracted from Mermaid sequenceDiagram specification. */
    interaction def BP_02_Check_34_AST_Purity_Fail_Closed_IO_and_Anti_Gaming_Test_Gate_Blueprint_SequenceInteraction {
        lifeline Runner : VerifyBaselineCLI;
        lifeline Gate : Check34PurityGate;
        lifeline Lexer : DeapCoreSourceAst;
        lifeline TestAud : AntiGamingAuditor;
        lifeline SSOT : SsotSymbolScanner;
        message check_compiler_ast_purity from Runner to Gate;
        message mask_literals_and_comments from Gate to Lexer;
        message reply_MaskedCodeLinesAndRustFnAstStream from Lexer to Gate;
        message contains_direct_range_slice from Gate to Gate;
        message audit_fail_closed_io_patterns from Gate to Gate;
        message audit_test_integrity_and_anti_gaming from Gate to TestAud;
        message reply_CPurity03TestIntegrityDiagnostics from TestAud to Gate;
        message audit_ssot_symbol_deduplication from Gate to SSOT;
        message reply_CPurity04AndCPurity from SSOT to Gate;
        message reply_SortedResultVerdict from Gate to Runner;
    }
    doc /* Test AST auditor that parses `#[test]` functions and rejects zero-assertion bodies, `mem::size_of` tautologies, unguarded `.exists()` skips, and non-RAII temp paths */
    part def AntiGamingTestBodyAuditor {
        attribute quantity : Integer = 1;
        attribute type : String = "Subcomponent";
    }
    doc /* Total detected panicking direct range-slice indexing expressions (`expr[a..b]`) across audited workspace modules [unit: count] */
    part def Check34AstPurityAndAntiGamingGate {
        attribute direct_range_slice_count : Integer = 0;
        doc /* Total detected fail-open filesystem or directory traversal error-swallowing call sites across workspace crates [unit: count] */
        attribute fail_open_io_site_count : Integer = 0;
        doc /* Total `#[test]` functions exhibiting zero assertions, `mem::size_of` dummy checks, conditional `.exists()` skips, or shared temp paths [unit: count] */
        attribute vacuous_test_count : Integer = 0;
        doc /* Total occurrences of SHA-256 round constant `0x428a2f98` outside `crates/deap-core/src/digest.rs` [unit: count] */
        attribute duplicate_sha256_impl_count : Integer = 0;
        doc /* Total duplicate Rust source lexical state machines defined outside `crates/deap-core/src/source_ast.rs` [unit: count] */
        attribute duplicate_lexer_impl_count : Integer = 0;
        doc /* Receives tokenized non-test Rust source files (`SourceFileAst`) and structural declarations across `crates/ * /src/ ** / *.rs` [protocol: RustTokenStream] */
        port in WorkspaceSourceAstPort : String {
            flow in WorkspaceSourceAstPort_flow : String;
        }
        doc /* Receives parsed `#[test]` function AST nodes (`RustFnAst`), statement blocks, and macro invocation trees [protocol: RustTestAstStream] */
        port in TestFunctionAstStreamPort : String {
            flow in TestFunctionAstStreamPort_flow : String;
        }
        doc /* Emits sorted, file-and-line-anchored diagnostic violations (`PurityViolationRecord`) when any Check 34 sub-gate fails [protocol: DeterministicDiag] */
        port out PurityViolationDiagnosticPort : String {
            flow out PurityViolationDiagnosticPort_flow : String;
        }
        doc /* Emits deterministic pass (`0`) or fail-closed (`1`) status (`GateVerdict`) to the `verify-baseline` runner [protocol: ExitCodeContract] */
        port out Check34GateStatusPort : Integer {
            flow out Check34GateStatusPort_flow : Integer;
        }
    }
    doc /* AST pattern verifier that rejects fail-open iterator flattening, `.filter_map(Result::ok)`, `.unwrap_or_default()` on filesystem reads, and `let _ = fs::` error discarding */
    part def FailClosedIoAstVerifier {
        attribute quantity : Integer = 1;
        attribute type : String = "Subcomponent";
    }
    doc /* Zero-regex token-stream analyzer that detects panicking Rust bracket range indexing (`expr[a..b]`, `expr[..b]`, `expr[a..]`) outside string literals, comments, and `.get(..)` calls */
    part def RangeSliceAstDetector {
        attribute quantity : Integer = 1;
        attribute type : String = "Subcomponent";
    }
    doc /* Workspace-wide AST symbol scanner enforcing single-module residency for SHA-256 constants (`0x428a2f98`) and Rust lexical state machines */
    part def SsotSymbolDeduplicationGate {
        attribute quantity : Integer = 1;
        attribute type : String = "Subcomponent";
    }
}