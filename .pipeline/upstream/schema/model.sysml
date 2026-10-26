doc /* Consolidated System Architecture */

doc /* Subsystem package for BP_01_Compiler_Enforced_Forbid_Lint_Gate_and_Zero_Suppression_AST_Architecture_Blueprint */
package BP_01_Compiler_Enforced_Forbid_Lint_Gate_and_Zero_Suppression_AST_Architecture_Blueprint {
    doc /* Execute cargo clippy --workspace --all-targets */
    action def Execute;
    doc /* Read [workspace.lints.clippy] forbid table */
    action def Read;
    doc /* Return forbid rules for unwrap, expect, panic, indexing */
    action def Return;
    doc /* Emit E0453 hard error if any allow(clippy::) exists */
    action def Emit;
    doc /* Run scan_clippy_allow_tokens on all crates/ ** / *.rs */
    action def Run;
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0601_BP_01_Compiler_Enforced_Forbid_Lint_Gate_and_Zero_Suppression_AST_Architecture_Blueprint {
        id = "REQ-0601";
        text = "The Digital Engineering Agent Platform (DEAP) specification and verification compiler (CompilerLintEnforcementGate, legacy blueprint reference DEAP-BP-DET-01, UUIDv5 6ba7b810-9dad-11d1-80b4-00c04fd430c1, Complexity Class P, diagnostic bindings **E0453**, **E0100**, **E0304**, governed by DO-178C";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_rustc_e0453_hard_rejection_of_inner_and_outer_clippy_allow_attributes : String = "- **Given**: [workspace.lints.clippy] in root Cargo.toml configures unwrap_used = \"forbid\", expect_used = \"forbid\", panic = \"forbid\", indexing_slicing = \"forbid\", disallowed_types = \"forbid\", and type_complexity = \"forbid\". - **When**: Any production or test module in crates/ introduces #![allow(clippy::unwrap_used)] or #[allow(clippy::indexing_slicing)]. - **Then**: cargo check --workspace --all-targets and cargo clippy --workspace --all-targets fail closed with non-zero exit status and emit error[E0453]: allow(clippy::...) overruled by outer forbid(clippy::...).";
        attribute ac_02_check_34_multi_line_whitespace_insensitive_suppression_rejection : String = "- **Given**: The Check 34 zero-regex token-stream scanner scan_clippy_allow_tokens in crates/verify-baseline/src/checks/structural.rs. - **When**: A Rust file contains a single-line or multi-line whitespace-split attribute such as #[ allow ( clippy :: expect_used ) ] or #![ cfg_attr ( test , allow ( clippy :: panic ) ) ]. - **Then**: ./target/release/verify-baseline . --no-domain detects the normalized #![allow(clippy::...)] or #[allow(clippy::...)] token sequence, reports the exact file path and starting 1-based line number, and fails closed with exit code 1.";
        attribute ac_03_unconditional_test_target_enforcement_without_conditional_exemption : String = "- **Given**: Unit test modules (#[cfg(test)] mod tests) and integration test files (crates/*/tests/*.rs) across all six workspace crates. - **When**: cargo test --workspace and ./target/release/verify-baseline . --no-domain are executed. - **Then**: Zero #![cfg_attr(test, allow(...))] exemptions exist, all test assertions use fallible Result return signatures or pattern matching instead of .unwrap(), .expect(), or direct slice indexing, and all workspace tests pass.";
        attribute ac_04_semantic_type_alias_replacement_for_type_complexity : String = "- **Given**: Complex nested return or collection signatures in crates/deap-core/src/source_ast.rs and crates/compile-sysml/src/lib.rs. - **When**: clippy::type_complexity = \"forbid\" is active across the workspace. - **Then**: All #[allow(clippy::type_complexity)] attributes are removed and replaced by documented semantic pub type aliases (ExtractedMethodSignature and ComponentMethodTable) or domain structs that compile with zero warnings under cargo clippy --workspace --all-targets -- -D warnings.";
        attribute ac_05_dogfood_ingestion_and_sysml_v2_compilation_of_req_0601 : String = "- **Given**: This blueprint specification docs/designs/blueprint-01-compiler-enforced-lint-forbid-gate.md. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-01-compiler-enforced-lint-forbid-gate.md --dry-run is executed. - **Then**: All 12 mechanical blueprint verification gates pass, and the translator synthesizes REQ-0601, CompilerLintEnforcementGate, its interface ports, state attributes, formal constraints (C-LINT-01..C-LINT-03), and AC-01..AC-05 into docs/designs/blueprint-01-compiler-enforced-lint-forbid-gate.sysml and.";
        doc /* \begin{aligned} T_{\text{scan}}(F, B) &= \sum_{i=1}^{F} O(B_i) = O(B_{\text{total}}) \\ S_{\text{scan}}(A_{\max}, V) &= O(A_{\max} + V) \\ E_{\text{suppression}} &= 0 \end{aligned} */
        constraint def Invariant_REQ_0601_C_LINT_03;
        doc /* \begin{aligned} \text{ValidWorkspace}(W) &\iff (E_{\text{suppression}}(W) = 0) \land (\text{Cov}_{\text{forbid}}(W) = 100) \land (N_{\text{crates}}(W) = 6) \\ \text{GateVerdict}(W) &\iff \prod_{c=1}^{6} \text{ForbidInherited}(c) \cdot \prod_{i=1}^{F} \mathbb{I}\left(\lvert \text{ScanAllow}(f_i) \rvert = 0\right) = 1 \end{aligned} */
        constraint def Invariant_REQ_0601_FormalInvariant;
        require Invariant_REQ_0601_C_LINT_03;
        require Invariant_REQ_0601_FormalInvariant;
        verify by AC_01_Rustc_E0453_Hard_Rejection_of_Inner_and_Outer_Clippy_Allow_Attributes;
        verify by AC_02_Check_34_Multi_Line_Whitespace_Insensitive_Suppression_Rejection;
        verify by AC_03_Unconditional_Test_Target_Enforcement_Without_Conditional_Exemption;
        verify by AC_04_Semantic_Type_Alias_Replacement_for_Type_Complexity;
        verify by AC_05_Dogfood_Ingestion_and_SysML_v2_Compilation_of_REQ_0601;
        satisfy by CompilerLintEnforcementGate;
    }
    doc /* Extracted from Mermaid stateDiagram specification. */
    state def BP_01_Compiler_Enforced_Forbid_Lint_Gate_and_Zero_Suppression_AST_Architecture_Blueprint_LifecycleState {
        state ScanningSource;
        state SkippingLineComment;
        state SkippingBlockComment;
        state InspectingAttribute;
        state NormalizingBracketBody;
        state ViolationRecorded;
        transition Initial_to_ScanningSource;
        transition ScanningSource_to_SkippingLineComment_on_slashSlashDetected;
        transition SkippingLineComment_to_ScanningSource_on_newlineReached;
        transition ScanningSource_to_SkippingBlockComment_on_slashStarDetected;
        transition SkippingBlockComment_to_ScanningSource_on_starSlashReached;
        transition ScanningSource_to_InspectingAttribute_on_hashTokenDetected;
        transition InspectingAttribute_to_NormalizingBracketBody_on_openingBracketDetected;
        transition InspectingAttribute_to_ScanningSource_on_nonAttributeHash;
        transition NormalizingBracketBody_to_ViolationRecorded_on_forbiddenNeedleMatched;
        transition NormalizingBracketBody_to_ScanningSource_on_depthZeroClean;
        transition ViolationRecorded_to_ScanningSource_on_continueLinearScan;
        transition ScanningSource_to_Terminal_on_endOfStream;
    }
    doc /* - **Given**: [workspace.lints.clippy] in root Cargo.toml configures unwrap_used = \"forbid\", expect_used = \"forbid\", panic = \"forbid\", indexing_slicing = \"forbid\", disallowed_types = \"forbid\", and type_complexity = \"forbid\". - **When**: Any production or test module in crates/ introduces #![allow(clippy::unwrap_used)] or #[allow(clippy::indexing_slicing)]. - **Then**: cargo check --workspace --all-targets and cargo clippy --workspace --all-targets fail closed with non-zero exit status and emit error[E0453]: allow(clippy::...) overruled by outer forbid(clippy::...). */
    use case def UC_AC_01_Rustc_E0453_Hard_Rejection_of_Inner_and_Outer_Clippy_Allow_Attributes {
        subject CompilerLintEnforcementGate;
        objective "Rustc E0453 Hard Rejection of Inner and Outer Clippy Allow Attributes";
    }
    doc /* - **Given**: The Check 34 zero-regex token-stream scanner scan_clippy_allow_tokens in crates/verify-baseline/src/checks/structural.rs. - **When**: A Rust file contains a single-line or multi-line whitespace-split attribute such as #[ allow ( clippy :: expect_used ) ] or #![ cfg_attr ( test , allow ( clippy :: panic ) ) ]. - **Then**: ./target/release/verify-baseline . --no-domain detects the normalized #![allow(clippy::...)] or #[allow(clippy::...)] token sequence, reports the exact file path and starting 1-based line number, and fails closed with exit code 1. */
    use case def UC_AC_02_Check_34_Multi_Line_Whitespace_Insensitive_Suppression_Rejection {
        subject CompilerLintEnforcementGate;
        objective "Check 34 Multi Line Whitespace Insensitive Suppression Rejection";
    }
    doc /* - **Given**: Unit test modules (#[cfg(test)] mod tests) and integration test files (crates/ * /tests/ *.rs) across all six workspace crates. - **When**: cargo test --workspace and ./target/release/verify-baseline . --no-domain are executed. - **Then**: Zero #![cfg_attr(test, allow(...))] exemptions exist, all test assertions use fallible Result return signatures or pattern matching instead of .unwrap(), .expect(), or direct slice indexing, and all workspace tests pass. */
    use case def UC_AC_03_Unconditional_Test_Target_Enforcement_Without_Conditional_Exemption {
        subject CompilerLintEnforcementGate;
        objective "Unconditional Test Target Enforcement Without Conditional Exemption";
    }
    doc /* - **Given**: Complex nested return or collection signatures in crates/deap-core/src/source_ast.rs and crates/compile-sysml/src/lib.rs. - **When**: clippy::type_complexity = \"forbid\" is active across the workspace. - **Then**: All #[allow(clippy::type_complexity)] attributes are removed and replaced by documented semantic pub type aliases (ExtractedMethodSignature and ComponentMethodTable) or domain structs that compile with zero warnings under cargo clippy --workspace --all-targets -- -D warnings. */
    use case def UC_AC_04_Semantic_Type_Alias_Replacement_for_Type_Complexity {
        subject CompilerLintEnforcementGate;
        objective "Semantic Type Alias Replacement for Type Complexity";
    }
    doc /* - **Given**: This blueprint specification docs/designs/blueprint-01-compiler-enforced-lint-forbid-gate.md. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-01-compiler-enforced-lint-forbid-gate.md --dry-run is executed. - **Then**: All 12 mechanical blueprint verification gates pass, and the translator synthesizes REQ-0601, CompilerLintEnforcementGate, its interface ports, state attributes, formal constraints (C-LINT-01..C-LINT-03), and AC-01..AC-05 into docs/designs/blueprint-01-compiler-enforced-lint-forbid-gate.sysml and. */
    use case def UC_AC_05_Dogfood_Ingestion_and_SysML_v2_Compilation_of_REQ_0601 {
        subject CompilerLintEnforcementGate;
        objective "Dogfood Ingestion and SysML v2 Compilation of REQ 0601";
    }
    doc /* Extracted from Mermaid sequenceDiagram specification. */
    interaction def BP_01_Compiler_Enforced_Forbid_Lint_Gate_and_Zero_Suppression_AST_Architecture_Blueprint_SequenceInteraction {
        lifeline Eng : Compiler_Engineer_CI;
        lifeline Cargo : cargo_clippy_rustc;
        lifeline Manifest : Root_Cargo_toml;
        lifeline Check34 : verify_baseline;
        lifeline Ingest : ingest_sysml;
        message Execute from Eng to Cargo;
        message Read from Cargo to Manifest;
        message Return from Manifest to Cargo;
        message Emit from Cargo to Eng;
        message Execute from Eng to Check34;
        message Run from Check34 to Check34;
        message Return from Check34 to Eng;
        message Execute from Eng to Ingest;
        message Emit from Ingest to Eng;
    }
    part def CompilerLintEnforcementGate {
        doc /* Configures `clippy::unwrap_used = "forbid"` across the workspace [unit: flag] */
        attribute forbid_unwrap_used : Boolean = true;
        doc /* Configures `clippy::expect_used = "forbid"` across the workspace [unit: flag] */
        attribute forbid_expect_used : Boolean = true;
        doc /* Configures `clippy::panic = "forbid"` across the workspace [unit: flag] */
        attribute forbid_panic : Boolean = true;
        doc /* Configures `clippy::indexing_slicing = "forbid"` across the workspace [unit: flag] */
        attribute forbid_indexing_slicing : Boolean = true;
        doc /* Configures `clippy::disallowed_types = "forbid"` across the workspace [unit: flag] */
        attribute forbid_disallowed_types : Boolean = true;
        doc /* Configures `clippy::type_complexity = "forbid"` across the workspace [unit: flag] */
        attribute forbid_type_complexity : Boolean = true;
        doc /* Maximum permitted `allow(clippy::` attribute occurrences across `crates/` [unit: count] */
        attribute allowed_suppression_count : Integer = 0;
        doc /* Percentage of workspace safety lints enforced at `forbid` level (**C-LINT-02**) [unit: percent] */
        attribute forbid_rule_coverage : Integer = 100;
        doc /* Number of workspace member crates inheriting `[lints] workspace = true` (**C-LINT-03**) [unit: count] */
        attribute crates_enforced_count : Integer = 6;
        doc /* Ingests root `Cargo.toml` and member crate manifest files for `[workspace.lints]` verification */
        port inout CargoManifestInputPort : String {
            attribute protocol_family : String = "FilesystemUTF8";
        }
        doc /* Streams UTF-8 bytes from production and test `.rs` files into the Check 34 attribute scanner */
        port inout RustSourceTokenStreamPort : String {
            attribute protocol_family : String = "TokenStream";
        }
        doc /* Emits deterministic `rustc` `E0453` diagnostics when `allow(...)` conflicts with workspace `forbid(...)` */
        port inout CompilerDiagnosticOutputPort : String {
            attribute protocol_family : String = "RustcDiagnostic";
        }
        doc /* Emits fail-closed Check 34 verification verdict (`Ok` or sorted `Err` violation vector) */
        port inout BaselineGateVerdictPort : Boolean {
            attribute protocol_family : String = "GateVerdict";
        }
    }
    doc /* Enforces forbid lint levels in root Cargo.toml and workspace = true inheritance across all member crates. */
    part def WorkspaceManifestForbidGate {
        attribute quantity : Integer = 1;
        port in cmd_in : String;
        port out status_out : String;
    }
    doc /* Zero-regex multi-line token-stream scanner in Check 34 rejecting allow(clippy:: attribute sequences. */
    part def ClippyAttributeAstScanner {
        attribute quantity : Integer = 1;
        port in cmd_in : String;
        port out status_out : String;
    }
    doc /* Strips crate-level #![allow(clippy::...)] headers and replaces complex tuples with named structs or type aliases. */
    part def CrateRootAttributeSanitizer {
        attribute quantity : Integer = 1;
        port in cmd_in : String;
        port out status_out : String;
    }
    doc /* Enforces identical forbid lint discipline across unit tests (#[cfg(test)]) and integration tests (tests/ *.rs). */
    part def TestTargetLintEnforcer {
        attribute quantity : Integer = 1;
        port in cmd_in : String;
        port out status_out : String;
    }
}

doc /* Subsystem package for BP_02_Check_34_AST_Purity_Fail_Closed_IO_and_Anti_Gaming_Test_Gate_Blueprint */
package BP_02_Check_34_AST_Purity_Fail_Closed_IO_and_Anti_Gaming_Test_Gate_Blueprint {
    doc /* check_compiler_ast_purity(repo_root) */
    action def check_compiler_ast_purity;
    doc /* mask_literals_and_comments(source) */
    action def mask_literals_and_comments;
    doc /* Masked code lines and RustFnAst stream */
    action def Masked;
    doc /* contains_direct_range_slice(code_line) */
    action def contains_direct_range_slice;
    doc /* audit_fail_closed_io_patterns(code_line) */
    action def audit_fail_closed_io_patterns;
    doc /* audit_test_integrity_and_anti_gaming(test_fn) */
    action def audit_test_integrity_and_anti_gaming;
    doc /* C-PURITY-03 test integrity diagnostics */
    action def C_PURITY_03;
    doc /* audit_ssot_symbol_deduplication(file_rel, source) */
    action def audit_ssot_symbol_deduplication;
    doc /* C-PURITY-04 and C-PURITY-05 diagnostics */
    action def C_PURITY_04;
    doc /* Sorted Result verdict (Ok or Err diagnostics) */
    action def Sorted;
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
        satisfy by RangeSliceAstDetector;
    }
    doc /* Synthesized deterministic package lifecycle state machine. */
    state def BP_02_Check_34_AST_Purity_Fail_Closed_IO_and_Anti_Gaming_Test_Gate_Blueprint_LifecycleState {
        state RangeSliceAstDetector_State;
        state FailClosedIoAstVerifier_State;
        state AntiGamingTestBodyAuditor_State;
        transition Initial_to_RangeSliceAstDetector_State;
        transition RangeSliceAstDetector_State_to_FailClosedIoAstVerifier_State_on_check_compiler_ast_purity;
        transition FailClosedIoAstVerifier_State_to_AntiGamingTestBodyAuditor_State_on_mask_literals_and_comments;
        transition AntiGamingTestBodyAuditor_State_to_Terminal;
    }
    doc /* - **Given**: Any audited Rust source file in crates/ containing a direct range-slice indexing expression (&s[a..b], &rest[..idx], &buf[start..], or &items[..=end]) outside string literals and comments. - **When**: ./target/release/verify-baseline . --no-domain executes Check 34 (C-PURITY-01). - **Then**: contains_direct_range_slice detects the bracketed .. operator regardless of variable name, emits a file-and-line diagnostic instructing the caller to use .get(..), and exits fail-closed with code 1. */
    use case def UC_AC_01_General_Direct_Range_Slice_Indexing_Rejection {
        subject RangeSliceAstDetector;
        objective "General Direct Range Slice Indexing Rejection";
    }
    doc /* - **Given**: Any workspace Rust module containing .into_iter().flatten(), .filter_map(Result::ok), fs::read_to_string(...).unwrap_or_default(), or let _ = fs::remove_dir_all(...). - **When**: Check 34 executes audit_fail_closed_io_patterns (C-PURITY-02). - **Then**: The verifier fails closed, flags every fail-open error-swallowing site with its exact line number, and terminates with a non-zero exit code. */
    use case def UC_AC_02_Fail_Closed_Filesystem_and_WalkDir_Enforcement {
        subject RangeSliceAstDetector;
        objective "Fail Closed Filesystem and WalkDir Enforcement";
    }
    doc /* - **Given**: A #[test] function in any workspace crate that contains zero assert* macros, asserts std::mem::size_of::<T>() > 0, wraps its assertions in an unguarded if Path::new(\"...\").exists() check, or constructs a hardcoded std::env::temp_dir().join(\"...\") path. - **When**: Check 34 executes audit_test_integrity_and_anti_gaming (C-PURITY-03). - **Then**: The test auditor fails closed, rejects the vacuous or un-isolated test function, and halts baseline verification. */
    use case def UC_AC_03_Anti_Gaming_Test_Body_AST_Enforcement {
        subject RangeSliceAstDetector;
        objective "Anti Gaming Test Body AST Enforcement";
    }
    doc /* - **Given**: A Rust source file outside crates/deap-core/src/digest.rs that defines SHA-256 round constant 0x428a2f98 or a file outside crates/deap-core/src/source_ast.rs that defines a duplicate Rust source tokenizer. - **When**: Check 34 executes audit_ssot_symbol_deduplication (C-PURITY-04 and C-PURITY-05). - **Then**: The gate reports an SSOT duplication violation and exits fail-closed with a non-zero status code. */
    use case def UC_AC_04_SSOT_Cryptographic_and_Lexer_Deduplication {
        subject RangeSliceAstDetector;
        objective "SSOT Cryptographic and Lexer Deduplication";
    }
    doc /* - **Given**: This blueprint document (docs/designs/blueprint-02-check34-ast-purity-and-anti-gaming-gate.md). - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-02-check34-ast-purity-and-anti-gaming-gate.md --dry-run and ./target/release/ingest-sysml are executed. - **Then**: The blueprint verifier passes all 12 mechanical gates, extracts Check34AstPurityAndAntiGamingGate, its subcomponents, 4 typed ports, 5 state attributes, 5 formal constraints (C-PURITY-01 through C-PURITY-05), and 5 BDD acceptance criteria into valid SysML v2, and exits with code 0. */
    use case def UC_AC_05_Deterministic_Dogfood_Ingestion_into_Canonical_SysML_v2 {
        subject RangeSliceAstDetector;
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
        message Masked from Lexer to Gate;
        message contains_direct_range_slice from Gate to Gate;
        message audit_fail_closed_io_patterns from Gate to Gate;
        message audit_test_integrity_and_anti_gaming from Gate to TestAud;
        message C_PURITY_03 from TestAud to Gate;
        message audit_ssot_symbol_deduplication from Gate to SSOT;
        message C_PURITY_04 from SSOT to Gate;
        message Sorted from Gate to Runner;
    }
    doc /* Zero-regex token-stream analyzer that detects panicking Rust bracket range indexing (`expr[a..b]`, `expr[..b]`, `expr[a..]`) outside string literals, comments, and `.get(..)` calls */
    part def RangeSliceAstDetector {
        attribute quantity : Integer = 1;
        attribute type : String = "Subcomponent";
    }
    doc /* AST pattern verifier that rejects fail-open iterator flattening, `.filter_map(Result::ok)`, `.unwrap_or_default()` on filesystem reads, and `let _ = fs::` error discarding */
    part def FailClosedIoAstVerifier {
        attribute quantity : Integer = 1;
        attribute type : String = "Subcomponent";
    }
    doc /* Test AST auditor that parses `#[test]` functions and rejects zero-assertion bodies, `mem::size_of` tautologies, unguarded `.exists()` skips, and non-RAII temp paths */
    part def AntiGamingTestBodyAuditor {
        attribute quantity : Integer = 1;
        attribute type : String = "Subcomponent";
    }
    doc /* Workspace-wide AST symbol scanner enforcing single-module residency for SHA-256 constants (`0x428a2f98`) and Rust lexical state machines */
    part def SsotSymbolDeduplicationGate {
        attribute type : String = "Subcomponent";
        attribute quantity : Integer = 1;
    }
    part def Check34AstPurityAndAntiGamingGate {
        doc /* Total detected panicking direct range-slice indexing expressions (`expr[a..b]`) across audited workspace modules [unit: count] */
        attribute direct_range_slice_count : Integer = 0;
        doc /* Total detected fail-open filesystem or directory traversal error-swallowing call sites across workspace crates [unit: count] */
        attribute fail_open_io_site_count : Integer = 0;
        doc /* Total `#[test]` functions exhibiting zero assertions, `mem::size_of` dummy checks, conditional `.exists()` skips, or shared temp paths [unit: count] */
        attribute vacuous_test_count : Integer = 0;
        doc /* Total occurrences of SHA-256 round constant `0x428a2f98` outside `crates/deap-core/src/digest.rs` [unit: count] */
        attribute duplicate_sha256_impl_count : Integer = 0;
        doc /* Total duplicate Rust source lexical state machines defined outside `crates/deap-core/src/source_ast.rs` [unit: count] */
        attribute duplicate_lexer_impl_count : Integer = 0;
        doc /* Receives tokenized non-test Rust source files (`SourceFileAst`) and structural declarations across `crates/ * /src/ ** / *.rs` */
        port inout WorkspaceSourceAstPort : String {
            attribute protocol_family : String = "RustTokenStream";
        }
        doc /* Receives parsed `#[test]` function AST nodes (`RustFnAst`), statement blocks, and macro invocation trees */
        port inout TestFunctionAstStreamPort : String {
            attribute protocol_family : String = "RustTestAstStream";
        }
        doc /* Emits sorted, file-and-line-anchored diagnostic violations (`PurityViolationRecord`) when any Check 34 sub-gate fails */
        port inout PurityViolationDiagnosticPort : String {
            attribute protocol_family : String = "DeterministicDiag";
        }
        doc /* Emits deterministic pass (`0`) or fail-closed (`1`) status (`GateVerdict`) to the `verify-baseline` runner */
        port inout Check34GateStatusPort : Integer {
            attribute protocol_family : String = "ExitCodeContract";
        }
        doc /* [unit: count] */
        assert constraint assert_direct_range_slice_count_range {
            direct_range_slice_count >= 0 and direct_range_slice_count <= 0;
        }
        doc /* [unit: count] */
        assert constraint assert_fail_open_io_site_count_range {
            fail_open_io_site_count >= 0 and fail_open_io_site_count <= 0;
        }
        doc /* [unit: count] */
        assert constraint assert_vacuous_test_count_range {
            vacuous_test_count >= 0 and vacuous_test_count <= 0;
        }
        doc /* [unit: count] */
        assert constraint assert_duplicate_sha256_impl_count_range {
            duplicate_sha256_impl_count >= 0 and duplicate_sha256_impl_count <= 0;
        }
        doc /* [unit: count] */
        assert constraint assert_duplicate_lexer_impl_count_range {
            duplicate_lexer_impl_count >= 0 and duplicate_lexer_impl_count <= 0;
        }
    }
}

doc /* Subsystem package for BP_03_SysML_v2_AST_Bijective_Roundtrip_and_Token_Stream_Parser_Gate_Blueprint */
package BP_03_SysML_v2_AST_Bijective_Roundtrip_and_Token_Stream_Parser_Gate_Blueprint {
    doc /* scan_all(raw_sysml_source) */
    action def scan_all;
    doc /* Vec of Token with zero span drift */
    action def Vec;
    doc /* parse_package_or_file(tokens) */
    action def parse_package_or_file;
    doc /* parse_type_expr and parse_transition_decl_tokens */
    action def parse_type_expr;
    doc /* Canonical PackageDef AST (ast_orig) */
    action def Canonical;
    doc /* verify_bijective_ast_roundtrip(ast_orig) */
    action def verify_bijective_ast_roundtrip;
    doc /* ast_orig.to_sysml(0) */
    action def ast_orig_to_sysml;
    doc /* Reparsed token stream */
    action def Reparsed;
    doc /* Assert ast_orig equals ast_reparsed */
    action def Assert_item;
    doc /* Ok or Err(E0400 Structural AST Diff) */
    action def Ok;
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0603_BP_03_SysML_v2_AST_Bijective_Roundtrip_and_Token_Stream_Parser_Gate_Blueprint {
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
    doc /* Extracted from Mermaid stateDiagram specification. */
    state def BP_03_SysML_v2_AST_Bijective_Roundtrip_and_Token_Stream_Parser_Gate_Blueprint_LifecycleState {
        state LexicalScanning;
        state RecursiveDescentParsing;
        state RejectedE0100;
        state CanonicalSerialization;
        state RejectedE0200;
        state RoundtripReparsing;
        state AstStructuralEqualityCheck;
        state RejectedE0400;
        state VerifiedBijectiveAst;
        transition Initial_to_LexicalScanning;
        transition LexicalScanning_to_RecursiveDescentParsing_on_zeroDriftTokenStream;
        transition LexicalScanning_to_RejectedE0100_on_lexicalSpanError;
        transition RecursiveDescentParsing_to_CanonicalSerialization_on_packageDefConstructed;
        transition RecursiveDescentParsing_to_RejectedE0200_on_unexpectedInnerBlockToken;
        transition CanonicalSerialization_to_RoundtripReparsing_on_sysmlTextEmitted;
        transition RoundtripReparsing_to_AstStructuralEqualityCheck_on_reparsedPackageDef;
        transition RoundtripReparsing_to_RejectedE0400_on_reparseSyntaxFailure;
        transition AstStructuralEqualityCheck_to_VerifiedBijectiveAst_on_astOrigEqualsAstReparsed;
        transition AstStructuralEqualityCheck_to_RejectedE0400_on_structuralAstDiffDetected;
        transition RejectedE0100_to_Terminal;
        transition RejectedE0200_to_Terminal;
        transition RejectedE0400_to_Terminal;
        transition VerifiedBijectiveAst_to_Terminal;
    }
    doc /* - **Given**: A SysML v2 model containing reference type sigils (&), optionality sigils (?), lifetime annotations ('a), qualified paths (::), and logical && expressions (#495). - **When**: Scanner::scan_all tokenizes the raw source buffer without invoking normalize_type_sigils. - **Then**: The lexer emits TokenKind::Ampersand, TokenKind::Question, TokenKind::Lifetime, TokenKind::DoubleColon, and TokenKind::AmpAmp with exact 1:1 byte and column offsets (span_offset_drift_bytes == 0). */
    use case def UC_AC_01_Native_Lexer_Sigil_and_Lifetime_Tokenization {
        subject NativeSigilAndLifetimeLexer;
        objective "Native Lexer Sigil and Lifetime Tokenization";
    }
    doc /* - **Given**: Any of the 12 SysML v2 inner declaration blocks (port, action, requirement, state, connection, use case, item, hazard, risk, test case, capability, interaction) containing either composite parameter types (Result<PackageDef, ParseError>, &'a [Token]) or an invalid inner statement (#504). - **When**: SysmlParser parses the declaration block. - **Then**: Valid composite types are preserved verbatim via parse_type_expr() and any unrecognized statement fails closed and is rejected with Err(ParseError) reporting diagnostic code E0200 (silent_skip_branch_count == 0). */
    use case def UC_AC_02_Fail_Closed_Inner_Block_and_Composite_Type_Parsing {
        subject NativeSigilAndLifetimeLexer;
        objective "Fail Closed Inner Block and Composite Type Parsing";
    }
    doc /* - **Given**: A hierarchical state def containing anonymous and named transition statements (first S1 accept trig if grd do eff then S2;) and nested composite sub_states with their own transitions and doc blocks (#496, #509). - **When**: parse_state_decl and the MBD .sfx synthesizer execute. - **Then**: Transitions are parsed directly from the token stream without fabricating {s}_to_{t} names (name: None preserved when unnamed), sub-state doc blocks attach to their target StateDef or TransitionDef, and MBD .sfx synthesis recursively emits transitions across all nesting depths. */
    use case def UC_AC_03_Token_Stream_State_Transition_Parsing_and_Recursive_MBD_Synthesis {
        subject NativeSigilAndLifetimeLexer;
        objective "Token Stream State Transition Parsing and Recursive MBD Synthesis";
    }
    doc /* - **Given**: A populated PackageDef AST containing StateDef transitions (source, trigger, guard, effect, target, doc), PortDef / HazardDef / RiskDef attributes, and UseCaseDef / ConnectionDef / RequirementDef / ActionDef blocks (#493, #497, #505), or a mutated AST with a simulated field divergence. - **When**: verify_bijective_ast_roundtrip(&pkg) executes during compile-sysml. - **Then**: pkg.to_sysml(0) emits valid un-double-quoted SysML v2 text, SysmlParser::parse_source(&pkg.to_sysml(0), &pkg.name) returns a PackageDef structurally equal to pkg (roundtrip_ast_mismatch_count == 0), and any. */
    use case def UC_AC_04_Lossless_to_sysml_Serialization_and_Fail_Closed_Bijective_Roundtrip_Gate {
        subject NativeSigilAndLifetimeLexer;
        objective "Lossless to sysml Serialization and Fail Closed Bijective Roundtrip Gate";
    }
    doc /* - **Given**: This blueprint document docs/designs/blueprint-03-sysml-ast-bijective-roundtrip-compiler-gate.md (**REQ-0603**). - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-03-sysml-ast-bijective-roundtrip-compiler-gate.md --dry-run is executed. - **Then**: verify-baseline check-blueprint passes all 12 mechanical gates, extracts SysmlBijectiveRoundtripCompilerGate with all 4 BOM parts, 4 interface ports, 4 attributes, 4 parametric constraints (C-RT-01..C-RT-04), and REQ_0603 requirement definitions, persists. */
    use case def UC_AC_05_Dogfood_Markdown_to_SysML_Ingestion_and_Baseline_Verification {
        subject NativeSigilAndLifetimeLexer;
        objective "Dogfood Markdown to SysML Ingestion and Baseline Verification";
    }
    doc /* Extracted from Mermaid sequenceDiagram specification. */
    interaction def BP_03_SysML_v2_AST_Bijective_Roundtrip_and_Token_Stream_Parser_Gate_Blueprint_SequenceInteraction {
        lifeline CLI : Compiler_CLI_Caller;
        lifeline Scan : NativeSigilAndLifetimeLexer;
        lifeline Parse : FailClosedRecursiveDescentParser;
        lifeline Ser : BijectiveSysmlSerializer;
        lifeline Gate : CompilerRoundtripVerificationGate;
        message scan_all from CLI to Scan;
        message Vec from Scan to CLI;
        message parse_package_or_file from CLI to Parse;
        message parse_type_expr from Parse to Parse;
        message Canonical from Parse to CLI;
        message verify_bijective_ast_roundtrip from CLI to Gate;
        message ast_orig_to_sysml from Gate to Ser;
        message Canonical from Ser to Gate;
        message scan_all from Gate to Scan;
        message Reparsed from Scan to Gate;
        message parse_package_or_file from Gate to Parse;
        message Reparsed from Parse to Gate;
        message Assert_item from Gate to Gate;
        message Ok from Gate to CLI;
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
    doc /* --- */
    part def SysmlBijectiveRoundtripCompilerGate {
        doc /* Cumulative byte-offset drift between raw source character positions and emitted token spans [unit: bytes] */
        attribute span_offset_drift_bytes : Integer = 0;
        doc /* Count of `skip_until_semi_or_brace` silent erasure fallback branches in inner block parsers [unit: branches] */
        attribute silent_skip_branch_count : Integer = 0;
        doc /* Count of AST struct fields omitted during `SysmlSerializable::to_sysml` serialization [unit: fields] */
        attribute dropped_ast_field_count : Integer = 0;
        doc /* Count of structural AST nodes differing between `ast` and `parse_sysml(to_sysml(ast))` [unit: nodes] */
        attribute roundtrip_ast_mismatch_count : Integer = 0;
        doc /* Ingests unmodified UTF-8 SysML v2 source text without pre-lexer string normalization */
        port inout RawSysmlSourceInputPort : String {
            attribute protocol_family : String = "UTF8Buffer";
        }
        doc /* Streams span-exact `Token` records from `NativeSigilAndLifetimeLexer` to `FailClosedRecursiveDescentParser` */
        port inout TokenStreamChannelPort : String {
            attribute protocol_family : String = "ZeroCopySlice";
        }
        doc /* Emits deterministic, human-readable SysML v2 text synthesized by `BijectiveSysmlSerializer` */
        port inout SerializedSysmlOutputPort : String {
            attribute protocol_family : String = "CanonicalSysmlV2";
        }
        doc /* Emits fail-closed structural AST diff and span location diagnostics (`E0100`..`E0400`) on roundtrip mismatch */
        port inout BijectiveDiffDiagnosticPort : String {
            attribute protocol_family : String = "StructuredDiagnostic";
        }
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

doc /* Subsystem package for BP_04_Deterministic_Specification_ICD_and_WBS_Forward_Compiler_Blueprint */
package BP_04_Deterministic_Specification_ICD_and_WBS_Forward_Compiler_Blueprint {
    doc /* compile-sysml --forward-sync docs */
    action def compile_sysml;
    doc /* Resolve Parent Epic by pkg_name == ConOps */
    action def Resolve;
    doc /* Bind all package stories to Use Case matrices */
    action def Bind;
    doc /* Emit symmetric ActionDef signatures and E1-E5 Mermaid */
    action def Emit;
    doc /* Epics, Features, Stories, and Use Cases written */
    action def Epics;
    doc /* Project NxN ConnectionDef matrix and FlowDef signals */
    action def Project;
    doc /* ICD_01 and ICD_02 written deterministically */
    action def ICD_01;
    doc /* Synthesize MIL-STD-881E tree and CSV/JSON exports */
    action def Synthesize;
    doc /* WBS_01..03 and enterprise exports written */
    action def WBS_01_03;
    doc /* reconcile-backlog reconcile --docs-dir docs */
    action def reconcile_backlog;
    doc /* Parse CommonMark tables with in_html_comment guard */
    action def Parse;
    doc /* Reconcile checklists with parent-inclusive epic count */
    action def Reconcile;
    doc /* Zero unpopulated placeholders or spoofed IDs */
    action def Zero;
    doc /* Merge extracted entities into owning subpackages */
    action def Merge;
    doc /* Bit-identical SysML v2 AST and SHA-256 digest */
    action def Bit_identical;
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0604_BP_04_Deterministic_Specification_ICD_and_WBS_Forward_Compiler_Blueprint {
        id = "REQ-0604";
        text = "The Digital Engineering Agent Platform (DEAP) compiler pipeline SHALL deterministically project all Agile functional specifications (docs/epics/, docs/features/, docs/user-stories/, docs/use-cases/), Logical Interface Control Documents (docs/interfaces/ICD_01_SYSTEM_INTERFACE_MATRIX.md,";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_deterministic_multi_story_use_case_and_conops_parent_epic_projection : String = "Diagnostic: E0100).";
        attribute ac_02_deterministic_logical_icd_matrix_and_signal_dictionary_synthesis : String = "Diagnostic: E0200).";
        attribute ac_03_deterministic_mil_std_881e_wbs_and_enterprise_export_synthesis : String = "Diagnostic: E0300).";
        attribute ac_04_fail_closed_stateful_html_comment_frontmatter_guard_and_parent_inclusive_checklist_reconciliation : String = "Diagnostic: E0300).";
        attribute ac_05_fail_closed_subpackage_scoped_reverse_sync_and_dogfood_blueprint_ingestion_parity : String = "Diagnostic: E0400).";
        doc /* \begin{aligned} \text{Invariant}_{\text{FWD}} &\iff (N_{\text{subagent}} = 0) \land (\text{Coverage}_{\text{AST}} = 100) \land (V_{\text{E1..E5}} = 0) \land (\Delta_{\text{sync}} = 0) \end{aligned} */
        constraint def Invariant_REQ_0604_SC_01;
        doc /* \begin{aligned} T_{\text{forward}}(P, C, I, U) &= O(P \log P + C^{2} + I \cdot \log P + U \cdot I) \\ S_{\text{forward}}(P, C, I, U) &= O(P + C^{2} + I + U) \\ \Delta_{\text{roundtrip}}(\mathcal{M}) &= H(\text{ReverseSync}(\text{ForwardSync}(\mathcal{M}))) \oplus H(\mathcal{M}) = 0 \end{aligned} */
        constraint def Invariant_REQ_0604_SC_02;
        require Invariant_REQ_0604_SC_01;
        require Invariant_REQ_0604_SC_02;
        verify by AC_01_Deterministic_Multi_Story_Use_Case_and_ConOps_Parent_Epic_Projection;
        verify by AC_02_Deterministic_Logical_ICD_Matrix_and_Signal_Dictionary_Synthesis;
        verify by AC_03_Deterministic_MIL_STD_881E_WBS_and_Enterprise_Export_Synthesis;
        verify by AC_04_Fail_Closed_Stateful_HTML_Comment_Frontmatter_Guard_and_Parent_Inclusive_Checklist_Reconciliation;
        verify by AC_05_Fail_Closed_Subpackage_Scoped_Reverse_Sync_and_Dogfood_Blueprint_Ingestion_Parity;
        satisfy by AgileSpecForwardProjector;
    }
    doc /* Extracted from Mermaid stateDiagram specification. */
    state def BP_04_Deterministic_Specification_ICD_and_WBS_Forward_Compiler_Blueprint_LifecycleState {
        state AstLoaded;
        state ForwardProjected;
        state IcdSynthesized;
        state WbsExported;
        state BacklogReconciled;
        state ReverseSynced;
        state Rejected;
        state VerifiedRoundtrip;
        transition Initial_to_AstLoaded;
        transition AstLoaded_to_ForwardProjected_on_forwardSyncSpecs;
        transition ForwardProjected_to_IcdSynthesized_on_emitLogicalIcd;
        transition IcdSynthesized_to_WbsExported_on_emitMilStd881eWbs;
        transition WbsExported_to_BacklogReconciled_on_reconcileBacklog;
        transition BacklogReconciled_to_ReverseSynced_on_reverseSyncToSubpackages;
        transition BacklogReconciled_to_Rejected_on_htmlCommentSpoofOrCardinalityError;
        transition ReverseSynced_to_VerifiedRoundtrip_on_sha256DigestParityZero;
        transition ReverseSynced_to_Rejected_on_rootAstPollutionOrDriftDetected;
        transition Rejected_to_Terminal;
        transition VerifiedRoundtrip_to_Terminal;
    }
    doc /* Diagnostic: E0100). */
    use case def UC_AC_01_Deterministic_Multi_Story_Use_Case_and_ConOps_Parent_Epic_Projection {
        subject AgileSpecForwardProjector;
        objective "Deterministic Multi Story Use Case and ConOps Parent Epic Projection";
    }
    doc /* Diagnostic: E0200). */
    use case def UC_AC_02_Deterministic_Logical_ICD_Matrix_and_Signal_Dictionary_Synthesis {
        subject AgileSpecForwardProjector;
        objective "Deterministic Logical ICD Matrix and Signal Dictionary Synthesis";
    }
    doc /* Diagnostic: E0300). */
    use case def UC_AC_03_Deterministic_MIL_STD_881E_WBS_and_Enterprise_Export_Synthesis {
        subject AgileSpecForwardProjector;
        objective "Deterministic MIL STD 881E WBS and Enterprise Export Synthesis";
    }
    doc /* Diagnostic: E0300). */
    use case def UC_AC_04_Fail_Closed_Stateful_HTML_Comment_Frontmatter_Guard_and_Parent_Inclusive_Checklist_Reconciliation {
        subject AgileSpecForwardProjector;
        objective "Fail Closed Stateful HTML Comment Frontmatter Guard and Parent Inclusive Checklist Reconciliation";
    }
    doc /* Diagnostic: E0400). */
    use case def UC_AC_05_Fail_Closed_Subpackage_Scoped_Reverse_Sync_and_Dogfood_Blueprint_Ingestion_Parity {
        subject AgileSpecForwardProjector;
        objective "Fail Closed Subpackage Scoped Reverse Sync and Dogfood Blueprint Ingestion Parity";
    }
    doc /* Extracted from Mermaid sequenceDiagram specification. */
    interaction def BP_04_Deterministic_Specification_ICD_and_WBS_Forward_Compiler_Blueprint_SequenceInteraction {
        lifeline CLI : Operator_Or_CI_Runner;
        lifeline Fwd : compile_sysml_Forward;
        lifeline Icd : LogicalIcdMatrixCompiler;
        lifeline Wbs : MilStd881eWbsExporter;
        lifeline Rec : reconcile_backlog_Tool;
        lifeline Rev : compile_sysml_Reverse;
        message compile_sysml from CLI to Fwd;
        message Resolve from Fwd to Fwd;
        message Bind from Fwd to Fwd;
        message Emit from Fwd to Fwd;
        message Epics from Fwd to CLI;
        message compile_sysml from CLI to Icd;
        message Project from Icd to Icd;
        message ICD_01 from Icd to CLI;
        message compile_sysml from CLI to Wbs;
        message Synthesize from Wbs to Wbs;
        message WBS_01_03 from Wbs to CLI;
        message reconcile_backlog from CLI to Rec;
        message Parse from Rec to Rec;
        message Reconcile from Rec to Rec;
        message Zero from Rec to CLI;
        message compile_sysml from CLI to Rev;
        message Merge from Rev to Rev;
        message Bit_identical from Rev to CLI;
    }
    part def AgileSpecForwardProjector {
        doc /* Maximum deterministic execution bound for `--forward-sync` across all package subtrees. [unit: ms] */
        attribute maxForwardLatencyMs : Integer = 25;
        doc /* Enforces symmetric `in` and `out` `ActionDef` parameter emission across Mermaid and Markdown bullets. [unit: flag] */
        attribute preserveTypedActionParams : Boolean = true;
        doc /* Receives ordered `PackageDef`, `PartDef`, `InteractionDef`, and `UseCaseDef` AST slices. */
        port inout packageAstInPort : String {
            attribute protocol_family : String = "MemoryBus";
        }
        doc /* Emits Check 13B E1-E5 compliant Epics, Features, User Stories, and Use Cases. */
        port inout specMarkdownOutPort : String {
            attribute protocol_family : String = "AtomicFsWrite";
        }
    }
    part def BidirectionalBacklogReconciler {
        doc /* Tracks multi-line `in_html_comment` state to prevent commented example tables from spoofing frontmatter issue IDs. [unit: flag] */
        attribute trackHtmlCommentState : Boolean = true;
        doc /* Computes parent-inclusive Epic count so child-only slices never trigger single-epic wildcard matching. [unit: flag] */
        attribute parentInclusiveEpics : Boolean = true;
        doc /* Reads and reconciles CommonMark metadata tables and bidirectional Epic/Feature checklists under `docs/`. */
        port inout specDocsInOutPort : String {
            attribute protocol_family : String = "AtomicFsWrite";
        }
        doc /* Merges reverse-synchronized specification entities into their owning subsystem `PackageDef` subpackages. */
        port inout subpackageSyncOutPort : String {
            attribute protocol_family : String = "AtomicFsWrite";
        }
    }
    part def DeterministicSpecIcdWbsForwardCompiler {
        doc /* Total number of generative LLM subagents invoked during specification, ICD, and WBS forward compilation (`C-FWD-01 == 0`). [unit: count] */
        attribute llm_subagent_invocation_count : Integer = 0;
        doc /* Percentage of declared SysML v2 AST packages, parts, ports, flows, interactions, and use cases projected into specifications (`C-FWD-02 == 100.0`). [unit: percent] */
        attribute ast_node_coverage_percent : Real = 100;
        doc /* Count of Check 13B E1-E5 Mermaid syntax or layout violations across all compiler-synthesized diagrams (`C-FWD-03 == 0`). [unit: count] */
        attribute mermaid_e1_e5_violation_count : Integer = 0;
        doc /* Count of AST structural or subpackage placement divergences after a forward-then-reverse synchronization cycle (`C-FWD-04 == 0`). [unit: count] */
        attribute forward_reverse_drift_count : Integer = 0;
        doc /* Receives the parsed, type-checked `SysmlPackage` (`PackageDef`) AST and SHA-256 `SchemaDigest` from `deap-core`. */
        port inout SysmlPackageAstInputPort : String {
            attribute protocol_family : String = "MemoryBus";
        }
        doc /* Emits deterministic `SpecSuiteBundle` Markdown specifications across `docs/epics/`, `docs/features/`, `docs/user-stories/`, and `docs/use-cases/`. */
        port inout MarkdownSpecSuiteOutputPort : String {
            attribute protocol_family : String = "AtomicFsWrite";
        }
        doc /* Emits `IcdDocumentPair` (`ICD_01_SYSTEM_INTERFACE_MATRIX.md` and `ICD_02_MASTER_SIGNAL_DICTIONARY.md`) under `docs/interfaces/`. */
        port inout IcdMatrixArtifactPort : String {
            attribute protocol_family : String = "AtomicFsWrite";
        }
        doc /* Emits `WbsExportBundle` (`WBS_01`..`WBS_03` Markdown artifacts and RFC 4180 CSV / JSON AST exports) under `docs/management/`. */
        port inout WbsEnterpriseExportPort : String {
            attribute protocol_family : String = "AtomicFsWrite";
        }
    }
    part def LogicalIcdMatrixCompiler {
        doc /* Maximum deterministic execution bound for `--emit-icd` interface matrix and signal dictionary compilation. [unit: ms] */
        attribute maxIcdLatencyMs : Integer = 10;
        doc /* Orders all `PartDef` matrix axes and `ConnectionDef` channels via `BTreeMap` deterministic traversal. [unit: flag] */
        attribute lexicographicalSort : Boolean = true;
        doc /* Receives `PartDef`, `PortDef`, `ConnectionDef`, `FlowDef`, and `ItemDef` AST nodes. */
        port inout interconnectAstInPort : String {
            attribute protocol_family : String = "MemoryBus";
        }
        doc /* Emits `ICD_01_SYSTEM_INTERFACE_MATRIX.md` and `ICD_02_MASTER_SIGNAL_DICTIONARY.md`. */
        port inout icdMarkdownOutPort : String {
            attribute protocol_family : String = "AtomicFsWrite";
        }
    }
    part def MilStd881eWbsExporter {
        doc /* Canonical 4-level MIL-STD-881E decomposition depth (System, Subsystem, Work Package, Deliverable Task). [unit: levels] */
        attribute wbsHierarchyDepth : Integer = 4;
        doc /* Maximum deterministic execution bound for `--emit-wbs` Markdown and CSV/JSON export generation. [unit: ms] */
        attribute maxWbsLatencyMs : Integer = 15;
        doc /* Receives `PackageDef` AST hierarchy and synchronized `docs/` specification frontmatter metadata. */
        port inout wbsAstMetadataInPort : String {
            attribute protocol_family : String = "MemoryBus";
        }
        doc /* Emits `WBS_01`..`WBS_03` Markdown registers and enterprise CSV/JSON AST exports. */
        port inout wbsArtifactsOutPort : String {
            attribute protocol_family : String = "AtomicFsWrite";
        }
    }
}

doc /* Subsystem package for BP_05_Unified_AST_Safety_Docstring_Synthesizer_and_Check_36_Verification_Blueprint */
package BP_05_Unified_AST_Safety_Docstring_Synthesizer_and_Check_36_Verification_Blueprint {
    doc /* Run compile-sysml --inject-safety-docstrings <DIR> */
    action def Run;
    doc /* Scan source lines via RustDartLexState and ScopedItemScannerState */
    action def Scan;
    doc /* Return declaration items, owner_name, params, fields, invariants */
    action def Return;
    doc /* Resolve schema context via collect_all_parts without .first() fallback */
    action def Resolve;
    doc /* Write 7-section docstrings atomically with 0 diff on second pass */
    action def Write;
    doc /* Tokenize .rs and .dart files using shared deap-core lexical state */
    action def Tokenize;
    doc /* Verify 7 sections, non-empty Realises tag, and unique normalized body */
    action def Verify_item;
    doc /* Emit canonical SysML v2 AST for REQ-0605 and SHA-256 digest */
    action def Emit;
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0605_BP_05_Unified_AST_Safety_Docstring_Synthesizer_and_Check_36_Verification_Blueprint {
        id = "REQ-0605";
        text = "The Digital Engineering Agent Platform (DEAP) requires 100% of production Rust (.rs) and Dart (.dart) declarations to carry complete, non-placeholder 7-section high-assurance safety docstrings (## 1. Safety Intent & Regulatory Scope through ## 7. Realization & Traceability) bound to formal SysML v2";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_unified_rust_and_dart_lexical_state_machine_without_magic_offsets : String = "- **Given**: .rs and .dart source files containing nested /* outer /* inner */ */ block comments, hashed raw strings r#\"...\"# and r##\"...\"##, Rust string literals containing \"{\", Dart interpolated strings \"value: {map['key']}\" containing nested quotes and comments, multi-byte UTF-8 character literals '↑', and multiple short lifetimes fn foo<'a, 'b>(x: &'a str, y: &'b str) within 10 bytes. - **When**: deap_core::source_ast, compile-sysml --inject-safety-docstrings, and verify-baseline Check 36 scan the files. - **Then**: All three crates execute the single unified.";
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
        satisfy by UnifiedRustDartLexicalScanner;
    }
    doc /* Synthesized deterministic package lifecycle state machine. */
    state def BP_05_Unified_AST_Safety_Docstring_Synthesizer_and_Check_36_Verification_Blueprint_LifecycleState {
        state UnifiedRustDartLexicalScanner_State;
        state ScopedSchemaContextResolver_State;
        state SevenSectionDocstringGenerator_State;
        transition Initial_to_UnifiedRustDartLexicalScanner_State;
        transition UnifiedRustDartLexicalScanner_State_to_ScopedSchemaContextResolver_State_on_Run;
        transition ScopedSchemaContextResolver_State_to_SevenSectionDocstringGenerator_State_on_Scan;
        transition SevenSectionDocstringGenerator_State_to_Terminal;
    }
    doc /* - **Given**: .rs and .dart source files containing nested / * outer / * inner * / * / block comments, hashed raw strings r#\"...\"# and r##\"...\"##, Rust string literals containing \"{\", Dart interpolated strings \"value: {map['key']}\" containing nested quotes and comments, multi-byte UTF-8 character literals '↑', and multiple short lifetimes fn foo<'a, 'b>(x: &'a str, y: &'b str) within 10 bytes. - **When**: deap_core::source_ast, compile-sysml --inject-safety-docstrings, and verify-baseline Check 36 scan the files. - **Then**: All three crates execute the single unified. */
    use case def UC_AC_01_Unified_Rust_and_Dart_Lexical_State_Machine_Without_Magic_Offsets {
        subject UnifiedRustDartLexicalScanner;
        objective "Unified Rust and Dart Lexical State Machine Without Magic Offsets";
    }
    doc /* - **Given**: A multi-part SysML v2 PackageDef containing PrimaryController (REQ-PC-001, constraint ThermalLimit), a nested PartDef NestedSubunit (REQ-NS-002) inside SecondaryMonitor.parts, and an unmapped Rust struct SecondaryMonitor with method pub fn primary_controller(&self) and free function pub fn unmapped_helper(). - **When**: resolve_schema_context is invoked during compile-sysml --inject-safety-docstrings. - **Then**: PrimaryController resolves REQ-PC-001 and ThermalLimit, NestedSubunit resolves REQ-NS-002 via collect_all_parts, and both SecondaryMonitor::primary_controller and. */
    use case def UC_AC_02_Strict_Schema_Context_Resolution_and_Recursive_Part_Traversal {
        subject UnifiedRustDartLexicalScanner;
        objective "Strict Schema Context Resolution and Recursive Part Traversal";
    }
    doc /* - **Given**: A Rust module containing #[cfg(test)] mod tests { ... } followed by a production pub unsafe fn post_test_api(), a #[cfg(test)] impl MockHelper { ... } block, an impl<F: Fn(u32) -> bool> FilterEngine<F> block, and a Dart class method preceded by a multi-line @pragma( 'vm:prefer-inline',) annotation. - **When**: compile-sysml --inject-safety-docstrings and verify-baseline Check 36 inspect the files. - **Then**: mod tests exits cleanly at its closing brace so post_test_api is audited (and fail-closed rejected with a non-zero violation error if undocumented), #[cfg(test)] impl. */
    use case def UC_AC_03_Scoped_mod_tests_Brace_Tracking_unsafe_extern_Classification_and_Arrow_Aware_impl_Headers {
        subject UnifiedRustDartLexicalScanner;
        objective "Scoped mod tests Brace Tracking unsafe extern Classification and Arrow Aware impl Headers";
    }
    doc /* - **Given**: Any Rust crate or Dart source directory enriched by compile-sysml --inject-safety-docstrings <DIR>, as well as a negative fixture containing duplicate normalized docstring bodies or missing sections. - **When**: compile-sysml --inject-safety-docstrings <DIR> is executed a second time and followed by ./target/release/verify-baseline . --no-domain. - **Then**: The second synthesis pass produces 0 modified bytes (docstring_idempotency_diff_bytes == 0), every synthesized docblock passes Check 36 7-section completeness and Check 34 seen_docblocks uniqueness, and any invalid, missing,. */
    use case def UC_AC_04_Bit_for_Bit_Idempotency_and_Fail_Closed_Check_34_Per_Function_Normalized_Body_Uniqueness {
        subject UnifiedRustDartLexicalScanner;
        objective "Bit for Bit Idempotency and Fail Closed Check 34 Per Function Normalized Body Uniqueness";
    }
    doc /* - **Given**: This blueprint specification docs/designs/blueprint-05-deterministic-safety-docstring-synthesizer.md. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-05-deterministic-safety-docstring-synthesizer.md --dry-run and ./target/release/ingest-sysml --schema docs/designs/blueprint-05-deterministic-safety-docstring-synthesizer.md --format markdown are executed. - **Then**: The translator parses **REQ-0605**, UnifiedSafetyDocstringSynthesizer, all four BOM subcomponents, all four interface ports, all four state attributes, all four parametric. */
    use case def UC_AC_05_Dogfood_Ingestion_and_SysML_v2_Compilation_of_REQ_0605 {
        subject UnifiedRustDartLexicalScanner;
        objective "Dogfood Ingestion and SysML v2 Compilation of REQ 0605";
    }
    doc /* Extracted from Mermaid sequenceDiagram specification. */
    interaction def BP_05_Unified_AST_Safety_Docstring_Synthesizer_and_Check_36_Verification_Blueprint_SequenceInteraction {
        lifeline CLI : Compiler_CLI_CI_Pipeline;
        lifeline Core : deap_core_source_ast;
        lifeline Synth : compile_sysml_docstring;
        lifeline Gate : verify_baseline_docstring_gate;
        lifeline Ingest : ingest_sysml;
        message Run from CLI to Synth;
        message Scan from Synth to Core;
        message Return from Core to Synth;
        message Resolve from Synth to Synth;
        message Write from Synth to CLI;
        message Run from CLI to Gate;
        message Tokenize from Gate to Core;
        message Return from Core to Gate;
        message Verify_item from Gate to CLI;
        message Run from CLI to Ingest;
        message Emit from Ingest to CLI;
    }
    doc /* Canonical zero-regex lexical state machine in `deap_core::source_ast` handling nested `/ * * /`, `r#"..."#`, Dart `${...}` frames, and exact char-literal vs. lifetime disambiguation */
    part def UnifiedRustDartLexicalScanner {
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
    doc /* Baseline verification gate in `verify-baseline` auditing 100% per-item 7-section docstring coverage and Check 34 normalized body uniqueness using `deap_core::source_ast` */
    part def Check36AstCompletenessVerifier {
        attribute quantity : Integer = 1;
        attribute type : String = "Component";
    }
    doc /* --- */
    part def UnifiedSafetyDocstringSynthesizer {
        doc /* Count of hardcoded byte-offset lookahead constants (`10` or `14`) in character/lifetime lexing [unit: count] */
        attribute magic_offset_constant_count : Integer = 0;
        doc /* Count of duplicate Rust/Dart lexical state machines outside `deap_core::source_ast` [unit: count] */
        attribute lexer_duplication_count : Integer = 0;
        doc /* Count of unmapped source items receiving `.first()` fallback or cross-entity requirement bindings [unit: count] */
        attribute false_attribution_count : Integer = 0;
        doc /* Byte diff produced when re-running `compile-sysml --inject-safety-docstrings` on enriched source [unit: bytes] */
        attribute docstring_idempotency_diff_bytes : Integer = 0;
        doc /* Ingests raw UTF-8 `.rs` and `.dart` source files from target workspace directories */
        port inout SourceCodeFileInputPort : String {
            attribute protocol_family : String = "FilesystemUTF8";
        }
        doc /* Supplies parsed SysML v2 `PartDef`, `ActionDef`, `RequirementDef`, and `ConstraintDef` AST hierarchies */
        port inout SysmlSchemaTraceabilityPort : PackageDef {
            attribute protocol_family : String = "AstReference";
        }
        doc /* Emits bit-for-bit idempotent `.rs` and `.dart` source files enriched with 7-section safety docstrings */
        port inout EnrichedSourceOutputPort : String {
            attribute protocol_family : String = "AtomicFileWrite";
        }
        doc /* Emits deterministic Check 36 and Check 34 docstring completeness and uniqueness verdicts */
        port inout Check36VerificationReportPort : Boolean {
            attribute protocol_family : String = "GateVerdict";
        }
        doc /* [unit: count] */
        assert constraint assert_magic_offset_constant_count_range {
            magic_offset_constant_count >= 0 and magic_offset_constant_count <= 0;
        }
        doc /* [unit: count] */
        assert constraint assert_lexer_duplication_count_range {
            lexer_duplication_count >= 0 and lexer_duplication_count <= 0;
        }
        doc /* [unit: count] */
        assert constraint assert_false_attribution_count_range {
            false_attribution_count >= 0 and false_attribution_count <= 0;
        }
        doc /* [unit: bytes] */
        assert constraint assert_docstring_idempotency_diff_bytes_range {
            docstring_idempotency_diff_bytes >= 0 and docstring_idempotency_diff_bytes <= 0;
        }
    }
}

doc /* Subsystem package for BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint */
package BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint {
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
        state Idle;
        state IngestingSourceAst;
        state EvaluatingFivePillars;
        state SynthesizingDossier;
        state Verified;
        state Validating12Gates;
        state Rejected;
        state DeduplicatingSsot;
        state PublishingClosedLoop;
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
    part def ClosedLoopDefectPublisherGate {
        doc /* Enforces all 12 `validate_defect_dossier` checks prior to persistence or publication. [unit: checks] */
        attribute schemaGateCount : Integer = 12;
        doc /* Requires zero byte discrepancy between local `.pipeline/defects/` file and remote GitHub issue body. [unit: flag] */
        attribute requireByteParity : Boolean = true;
        doc /* Receives synthesized markdown payload for in-process 12-check verification. */
        port inout dossierCandidateInPort : String {
            attribute protocol_family : String = "CommonMarkText";
        }
        doc /* Emits verified GitHub SSOT issue URL after `gh issue view` closed-loop payload verification. */
        port inout verifiedReceiptOutPort : String {
            attribute protocol_family : String = "GhCliJson";
        }
    }
    part def DeterministicDossierSynthesizer {
        doc /* Enforces all 7 canonical `DossierSection` headers plus the `## Audit Source` footer. [unit: sections] */
        attribute mandatorySectionCount : Integer = 7;
        doc /* Enforces 5 numbered causal `Why ...? Because ...` root-cause steps in Section 2. [unit: steps] */
        attribute whysChainDepth : Integer = 5;
        doc /* Receives a deterministic `AuditFinding` struct for markdown serialization. */
        port inout findingInPort : String {
            attribute protocol_family : String = "AuditFindingRef";
        }
        doc /* Emits a 7-section defect dossier payload starting with `## 1. Context and References`. */
        port inout dossierMarkdownOutPort : String {
            attribute protocol_family : String = "CommonMarkText";
        }
    }
    part def DeterministicStaticDefectAuditor {
        doc /* Number of LLM subagents dispatched during static code auditing and defect dossier generation. [unit: count] */
        attribute auditor_subagent_count : Integer = 0;
        doc /* Number of deterministic structural, lexical, and Mermaid validation checks enforced per synthesized dossier. [unit: checks] */
        attribute dossier_schema_check_count : Integer = 12;
        doc /* Number of adversarial correctness risk pillars evaluated per source AST scan. [unit: pillars] */
        attribute pillar_coverage_count : Integer = 5;
        doc /* Byte discrepancy between local `.pipeline/defects/` dossier and live GitHub issue body after CRLF normalization. [unit: B] */
        attribute closed_loop_mismatch_bytes : Integer = 0;
        doc /* Receives parsed Rust (`.rs`) and Dart (`.dart`) `SourceAstItem` declarations and token spans from `deap_core::source_ast`. */
        port inout TargetSourceAstInputPort : String {
            attribute protocol_family : String = "RustSliceAst";
        }
        doc /* Receives existing `.pipeline/defects/ *.md` records and live GitHub issue metadata for idempotent finding deduplication. */
        port inout GitHubIssueSsotQueryPort : String {
            attribute protocol_family : String = "GhCliJson";
        }
        doc /* Persists validated 7-section `.pipeline/defects/DEFECT-.md` dossiers to the repository defect registry. */
        port inout DefectDossierArtifactPort : String {
            attribute protocol_family : String = "AtomicFsWrite";
        }
        doc /* Publishes or updates GitHub SSOT issues via `gh issue create` or `gh issue edit` and verifies byte parity via `gh issue view`. */
        port inout ClosedLoopTrackerPublishPort : String {
            attribute protocol_family : String = "GhCliSubprocess";
        }
    }
    part def FivePillarStaticAstAnalyzer {
        doc /* Count of active static AST evaluators (`E0601` through `E0605`). [unit: pillars] */
        attribute activePillarCount : Integer = 5;
        doc /* Enforces linear token-stream and `SourceAstItem` slice traversal without regular expressions. [unit: flag] */
        attribute zeroRegexMode : Boolean = true;
        doc /* Accepts normalized file paths, source lines, and `SourceAstItem` slices. */
        port inout astStreamInPort : String {
            attribute protocol_family : String = "RustSliceAst";
        }
        doc /* Emits line-exact `AuditFinding` records with diagnostic bindings `E0601` through `E0605`. */
        port inout findingStreamOutPort : String {
            attribute protocol_family : String = "AuditFindingVec";
        }
    }
    part def MermaidUmlDiagramGenerator {
        doc /* Guarantees zero semicolons and quoted special tokens inside Section 4 Mermaid blocks. [unit: flag] */
        attribute forbidSemicolons : Boolean = true;
        doc /* Enforces Rules E1 through E5 across synthesized `sequenceDiagram` and `classDiagram` blocks. [unit: flag] */
        attribute check13bCompliant : Boolean = true;
        doc /* Receives caller/callee symbols, line spans, and violated invariant metadata. */
        port inout umlSpecInPort : String {
            attribute protocol_family : String = "AuditFindingRef";
        }
        doc /* Emits a fenced Mermaid diagram or severity-scoped `N/A --` declaration for Section 4. */
        port inout mermaidBlockOutPort : String {
            attribute protocol_family : String = "MermaidText";
        }
    }
}