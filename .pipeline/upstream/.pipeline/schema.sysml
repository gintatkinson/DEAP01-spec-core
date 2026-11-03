doc /* Consolidated System Architecture */

doc /* Subsystem package for BP_01_Compiler_Enforced_Forbid_Lint_Gate_and_Zero_Suppression_AST_Architecture_Blueprint */
package BP_01_Compiler_Enforced_Forbid_Lint_Gate_and_Zero_Suppression_AST_Architecture_Blueprint {
    port def BP_01_ForbidLintPortDef {
        flow in manifest_in : String;
        flow out verdict_out : Boolean;
    }
    doc /* Execute cargo clippy --workspace --all-targets --message-format=json across all workspace crates */
    action def ExecuteCargoClippyWorkspaceAllTargets;
    doc /* Parse and verify [workspace.lints.clippy] forbid table via toml_edit, cargo_metadata, cargo-lock, cargo-util-schemas, and cargo-platform */
    action def ReadWorkspaceLintsClippyForbidTable;
    doc /* Execute verify-baseline . --no-domain with lossless ra_ap_syntax CST and cargo clippy JSON enforcement */
    action def ExecuteVerifyBaselineNoDomain;
    doc /* Audit #[allow(clippy::...)] and #![cfg_attr(..., allow(clippy::...))] attributes across crates/ via ra_ap_syntax ast::Attr and ast::HasAttrs CST nodes */
    action def RunScanClippyAllowTokensOn;
    doc /* Execute ingest-sysml --schema blueprint-01 --format markdown via pulldown-cmark and logos+rowan SysML v2 CST */
    action def ExecuteIngestSysmlSchemaBlueprint01;
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0601_BP_01_Compiler_Enforced_Forbid_Lint_Gate_and_Zero_Suppression_AST_Architecture_Blueprint {
        id = "REQ-0601";
        text = "The Digital Engineering Agent Platform (DEAP) specification and verification compiler (CompilerLintEnforcementGate, legacy blueprint reference DEAP-BP-DET-01, UUIDv5 6ba7b810-9dad-11d1-80b4-00c04fd430c1, Complexity Class P, diagnostic bindings E0453, E0100, E0304, governed by DO-178C DAL-A and DO-330 TQL-4) SHALL enforce Contract A (full 6-clause OMG SysML v2.0 + KerML 1.0 grammar coverage with zero 16-construct caps) and Contract B (rustc/cargo clippy --message-format=json + ra_ap_syntax ast::Attr and ast::HasAttrs lossless CST verification, ra_ap_vfs, ra_ap_paths, ra_ap_line_index, ra_ap_text_edit, rowan, camino, ignore, cargo_metadata, cargo-lock, cargo-util-schemas, cargo-platform, and toml_edit inside < 380-SLOC submodules with zero hand-rolled token scanners).";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_rustc_e0453_hard_rejection_of_inner_and_outer_clippy_allow_attributes : String = "- **Given**: [workspace.lints.clippy] in root Cargo.toml parsed via toml_edit and cargo_metadata configures unwrap_used = \"forbid\", expect_used = \"forbid\", panic = \"forbid\", indexing_slicing = \"forbid\", disallowed_types = \"forbid\", and type_complexity = \"forbid\". - **When**: Any production or test module in crates/ introduces #![allow(clippy::unwrap_used)] or #[allow(clippy::indexing_slicing)]. - **Then**: cargo check --workspace --all-targets and cargo clippy --workspace --all-targets --message-format=json fail closed with non-zero exit status and emit error[E0453]: allow(clippy::...) overruled by outer forbid(clippy::...).";
        attribute ac_02_check_34_multi_line_whitespace_insensitive_suppression_rejection : String = "- **Given**: The Check 34 lossless ra_ap_syntax (ast::Attr, ast::HasAttrs) + rowan CST attribute verifier in crates/verify-baseline/src/clippy_forbid/ and crates/deap-core/src/rust_cst/ with byte-to-line resolution via ra_ap_line_index (zero hand-rolled token scanners). - **When**: A Rust file contains a single-line or multi-line whitespace-split attribute such as #[ allow ( clippy :: expect_used ) ] or #![ cfg_attr ( test , allow ( clippy :: panic ) ) ]. - **Then**: ./target/release/verify-baseline . --no-domain inspects the typed ast::Attr Meta token tree, reports the exact camino::Utf8Path file path and starting 1-based line number via ra_ap_line_index, and fails closed with exit code 1.";
        attribute ac_03_unconditional_test_target_enforcement_without_conditional_exemption : String = "- **Given**: Unit test modules (#[cfg(test)] mod tests) and integration test files (crates/ * /tests/ *.rs) across all six workspace crates audited via ra_ap_syntax and cargo clippy --workspace --all-targets. - **When**: cargo test --workspace and ./target/release/verify-baseline . --no-domain are executed. - **Then**: Zero #![cfg_attr(test, allow(...))] exemptions exist, all test assertions use fallible Result return signatures or pattern matching instead of .unwrap(), .expect(), or direct slice indexing, and all workspace tests pass.";
        attribute ac_04_semantic_type_alias_replacement_for_type_complexity : String = "- **Given**: Complex nested return or collection signatures in crates/deap-core/src/rust_cst/, crates/deap-core/src/dart_cst/, and crates/compile-sysml/src/lib.rs. - **When**: clippy::type_complexity = \"forbid\" is active across the workspace. - **Then**: All #[allow(clippy::type_complexity)] attributes are removed and replaced by documented semantic pub type aliases (ExtractedMethodSignature and ComponentMethodTable) or domain structs in < 380-SLOC submodules that compile with zero warnings under cargo clippy --workspace --all-targets -- -D warnings.";
        attribute ac_05_dogfood_ingestion_and_sysml_v2_compilation_of_req_0601 : String = "- **Given**: This blueprint specification docs/designs/blueprint-01-compiler-enforced-lint-forbid-gate.md. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-01-compiler-enforced-lint-forbid-gate.md --dry-run is executed. - **Then**: All 12 mechanical blueprint verification gates pass, and the translator synthesizes REQ-0601, CompilerLintEnforcementGate, its interface ports, state attributes, formal constraints (C-LINT-01..C-LINT-03), and AC-01..AC-05 into docs/designs/blueprint-01-compiler-enforced-lint-forbid-gate.sysml and .digest.json with zero bijective roundtrip drift.";
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
        transition Initial_to_ScanningSource first Initial then ScanningSource;
        transition ScanningSource_to_SkippingLineComment_on_slashSlashDetected first ScanningSource accept slashSlashDetected then SkippingLineComment;
        transition SkippingLineComment_to_ScanningSource_on_newlineReached first SkippingLineComment accept newlineReached then ScanningSource;
        transition ScanningSource_to_SkippingBlockComment_on_slashStarDetected first ScanningSource accept slashStarDetected then SkippingBlockComment;
        transition SkippingBlockComment_to_ScanningSource_on_starSlashReached first SkippingBlockComment accept starSlashReached then ScanningSource;
        transition ScanningSource_to_InspectingAttribute_on_hashTokenDetected first ScanningSource accept hashTokenDetected then InspectingAttribute;
        transition InspectingAttribute_to_NormalizingBracketBody_on_openingBracketDetected first InspectingAttribute accept openingBracketDetected then NormalizingBracketBody;
        transition InspectingAttribute_to_ScanningSource_on_nonAttributeHash first InspectingAttribute accept nonAttributeHash then ScanningSource;
        transition NormalizingBracketBody_to_ViolationRecorded_on_forbiddenNeedleMatched first NormalizingBracketBody accept forbiddenNeedleMatched then ViolationRecorded;
        transition NormalizingBracketBody_to_ScanningSource_on_depthZeroClean first NormalizingBracketBody accept depthZeroClean then ScanningSource;
        transition ViolationRecorded_to_ScanningSource_on_continueLinearScan first ViolationRecorded accept continueLinearScan then ScanningSource;
        transition ScanningSource_to_Terminal_on_endOfStream first ScanningSource accept endOfStream then Terminal;
    }
    connection def BP_01_ManifestToForbidGateConnection {
        connect WorkspaceManifestForbidGate.status_out to CompilerLintEnforcementGate.CargoManifestInputPort;
    }
    doc /* - **Given**: [workspace.lints.clippy] in root Cargo.toml parsed via toml_edit and cargo_metadata configures unwrap_used = \"forbid\", expect_used = \"forbid\", panic = \"forbid\", indexing_slicing = \"forbid\", disallowed_types = \"forbid\", and type_complexity = \"forbid\". - **When**: Any production or test module in crates/ introduces #![allow(clippy::unwrap_used)] or #[allow(clippy::indexing_slicing)]. - **Then**: cargo check --workspace --all-targets and cargo clippy --workspace --all-targets --message-format=json fail closed with non-zero exit status and emit error[E0453]: allow(clippy::...) overruled by outer forbid(clippy::...). */
    use case def UC_AC_01_Rustc_E0453_Hard_Rejection_of_Inner_and_Outer_Clippy_Allow_Attributes {
        subject CompilerLintEnforcementGate;
        objective "Rustc E0453 Hard Rejection of Inner and Outer Clippy Allow Attributes";
    }
    doc /* - **Given**: The Check 34 lossless ra_ap_syntax (ast::Attr, ast::HasAttrs) + rowan CST attribute verifier in crates/verify-baseline/src/clippy_forbid/ and crates/deap-core/src/rust_cst/ with byte-to-line resolution via ra_ap_line_index (zero hand-rolled token scanners). - **When**: A Rust file contains a single-line or multi-line whitespace-split attribute such as #[ allow ( clippy :: expect_used ) ] or #![ cfg_attr ( test , allow ( clippy :: panic ) ) ]. - **Then**: ./target/release/verify-baseline . --no-domain inspects the typed ast::Attr Meta token tree, reports the exact camino::Utf8Path file path and starting 1-based line number via ra_ap_line_index, and fails closed with exit code 1. */
    use case def UC_AC_02_Check_34_Multi_Line_Whitespace_Insensitive_Suppression_Rejection {
        subject CompilerLintEnforcementGate;
        objective "Check 34 Multi Line Whitespace Insensitive Suppression Rejection";
    }
    doc /* - **Given**: Unit test modules (#[cfg(test)] mod tests) and integration test files (crates/ * /tests/ *.rs) across all six workspace crates audited via ra_ap_syntax and cargo clippy --workspace --all-targets. - **When**: cargo test --workspace and ./target/release/verify-baseline . --no-domain are executed. - **Then**: Zero #![cfg_attr(test, allow(...))] exemptions exist, all test assertions use fallible Result return signatures or pattern matching instead of .unwrap(), .expect(), or direct slice indexing, and all workspace tests pass. */
    use case def UC_AC_03_Unconditional_Test_Target_Enforcement_Without_Conditional_Exemption {
        subject CompilerLintEnforcementGate;
        objective "Unconditional Test Target Enforcement Without Conditional Exemption";
    }
    doc /* - **Given**: Complex nested return or collection signatures in crates/deap-core/src/rust_cst/, crates/deap-core/src/dart_cst/, and crates/compile-sysml/src/lib.rs. - **When**: clippy::type_complexity = \"forbid\" is active across the workspace. - **Then**: All #[allow(clippy::type_complexity)] attributes are removed and replaced by documented semantic pub type aliases (ExtractedMethodSignature and ComponentMethodTable) or domain structs in < 380-SLOC submodules that compile with zero warnings under cargo clippy --workspace --all-targets -- -D warnings. */
    use case def UC_AC_04_Semantic_Type_Alias_Replacement_for_Type_Complexity {
        subject CompilerLintEnforcementGate;
        objective "Semantic Type Alias Replacement for Type Complexity";
    }
    doc /* - **Given**: This blueprint specification docs/designs/blueprint-01-compiler-enforced-lint-forbid-gate.md. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-01-compiler-enforced-lint-forbid-gate.md --dry-run is executed. - **Then**: All 12 mechanical blueprint verification gates pass, and the translator synthesizes REQ-0601, CompilerLintEnforcementGate, its interface ports, state attributes, formal constraints (C-LINT-01..C-LINT-03), and AC-01..AC-05 into docs/designs/blueprint-01-compiler-enforced-lint-forbid-gate.sysml and .digest.json with zero bijective roundtrip drift. */
    use case def UC_AC_05_Dogfood_Ingestion_and_SysML_v2_Compilation_of_REQ_0601 {
        subject CompilerLintEnforcementGate;
        objective "Dogfood Ingestion and SysML v2 Compilation of REQ 0601";
    }
    test case def TC_REQ_0601_Forbid_Lint_Gate {
        verify requirement REQ_0601;
    }
    test case def TC_REQ_0601_Blueprint_Conformance {
        verify requirement REQ_0601_BP_01_Compiler_Enforced_Forbid_Lint_Gate_and_Zero_Suppression_AST_Architecture_Blueprint;
    }
    test case def Test_ExecuteCargoClippyWorkspaceAllTargets {
        verify requirement REQ_0601;
    }
    test case def Test_ReadWorkspaceLintsClippyForbidTable {
        verify requirement REQ_0601;
    }
    test case def Test_ExecuteVerifyBaselineNoDomain {
        verify requirement REQ_0601;
    }
    test case def Test_RunScanClippyAllowTokensOn {
        verify requirement REQ_0601;
    }
    test case def Test_ExecuteIngestSysmlSchemaBlueprint01 {
        verify requirement REQ_0601;
    }
    doc /* Extracted from Mermaid sequenceDiagram specification. */
    interaction def BP_01_Compiler_Enforced_Forbid_Lint_Gate_and_Zero_Suppression_AST_Architecture_Blueprint_SequenceInteraction {
        lifeline Eng : Compiler_Engineer_CI;
        lifeline Cargo : cargo_clippy_rustc;
        lifeline Manifest : Root_Cargo_toml;
        lifeline Check34 : verify_baseline;
        lifeline Ingest : ingest_sysml;
        message ExecuteCargoClippyWorkspaceAllTargets from Eng to Cargo;
        message ReadWorkspaceLintsClippyForbidTable from Cargo to Manifest;
        message reply_ReturnForbidRulesForUnwrapExpect from Manifest to Cargo;
        message reply_EmitE0453HardErrorIfAny from Cargo to Eng;
        message ExecuteVerifyBaselineNoDomain from Eng to Check34;
        message RunScanClippyAllowTokensOn from Check34 to Check34;
        message reply_ReturnExitCode0WhenAllowed from Check34 to Eng;
        message ExecuteIngestSysmlSchemaBlueprint01 from Eng to Ingest;
        message reply_EmitCanonicalSysMLV2AstAnd from Ingest to Eng;
    }
    doc /* Configures `clippy::unwrap_used = "forbid"` across the workspace [unit: flag] */
    part def CompilerLintEnforcementGate {
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
        doc /* Ingests root `Cargo.toml` and member crate manifest files via `toml_edit`, `cargo_metadata`, `cargo-lock`, `cargo-util-schemas`, and `cargo-platform` for `[workspace.lints]` verification */
        port in CargoManifestInputPort : String {
            attribute protocol_family : String = "FilesystemUTF8";
            doc /* Ingests root `Cargo.toml` and member crate manifest files via `toml_edit`, `cargo_metadata`, `cargo-lock`, `cargo-util-schemas`, and `cargo-platform` for `[workspace.lints]` verification */
            flow in CargoManifestInputPort_flow : String;
        }
        doc /* Streams UTF-8 bytes from production and test `.rs` files into the `ra_ap_syntax` (`ast::Attr`, `ast::HasAttrs`) + `rowan` lossless CST attribute verifier */
        port in RustSourceTokenStreamPort : String {
            attribute protocol_family : String = "TokenStream";
            doc /* Streams UTF-8 bytes from production and test `.rs` files into the `ra_ap_syntax` (`ast::Attr`, `ast::HasAttrs`) + `rowan` lossless CST attribute verifier */
            flow in RustSourceTokenStreamPort_flow : String;
        }
        doc /* Emits deterministic `rustc` `E0453` `--message-format=json` diagnostics when `allow(...)` conflicts with workspace `forbid(...)` */
        port out CompilerDiagnosticOutputPort : String {
            attribute protocol_family : String = "RustcDiagnostic";
            doc /* Emits deterministic `rustc` `E0453` `--message-format=json` diagnostics when `allow(...)` conflicts with workspace `forbid(...)` */
            flow out CompilerDiagnosticOutputPort_flow : String;
        }
        doc /* Emits fail-closed Check 34 verification verdict (`Ok` or sorted `Err` violation vector with `ra_ap_line_index` coordinates) */
        port out BaselineGateVerdictPort : Boolean {
            attribute protocol_family : String = "GateVerdict";
            doc /* Emits fail-closed Check 34 verification verdict (`Ok` or sorted `Err` violation vector with `ra_ap_line_index` coordinates) */
            flow out BaselineGateVerdictPort_flow : Boolean;
        }
    }
    doc /* Enforces forbid lint levels in root Cargo.toml and workspace = true inheritance across all member crates via toml_edit, cargo_metadata, cargo-lock, cargo-util-schemas, and cargo-platform. */
    part def WorkspaceManifestForbidGate {
        attribute quantity : Integer = 1;
        port in cmd_in : String;
        port out status_out : String;
    }
    doc /* Lossless ra_ap_syntax (ast::Attr, ast::HasAttrs) and rowan CST verifier in Check 34 rejecting allow(clippy::...) and cfg_attr(..., allow(clippy::...)) attribute trees with ra_ap_line_index span resolution. */
    part def ClippyAttributeCstParser {
        attribute quantity : Integer = 1;
        port in cmd_in : String;
        port out status_out : String;
    }
    doc /* Strips crate-level #![allow(clippy::...)] headers via ra_ap_text_edit and replaces complex tuples with named structs or type aliases in < 380-SLOC submodules. */
    part def CrateRootAttributeSanitizer {
        attribute quantity : Integer = 1;
        port in cmd_in : String;
        port out status_out : String;
    }
    doc /* Enforces identical forbid lint discipline across unit tests (#[cfg(test)]) and integration tests (tests/ *.rs) via cargo clippy --all-targets and ra_ap_syntax CST checks. */
    part def TestTargetLintEnforcer {
        attribute quantity : Integer = 1;
        port in cmd_in : String;
        port out status_out : String;
    }
}

doc /* Subsystem package for BP_02_Check_34_AST_Purity_Fail_Closed_IO_and_Anti_Gaming_Test_Gate_Blueprint */
package BP_02_Check_34_AST_Purity_Fail_Closed_IO_and_Anti_Gaming_Test_Gate_Blueprint {
    port def BP_02_PurityDiagnosticPortDef {
        flow in cst_stream_in : String;
        flow out purity_status_out : Integer;
    }
    doc /* Execute check_compiler_ast_purity(repo_root) across crates/ via ra_ap_syntax, ra_ap_vfs, ra_ap_paths, ra_ap_line_index, rowan, camino, and ignore */
    action def check_compiler_ast_purity;
    doc /* Partition trivia, string literals, and comments losslessly via ra_ap_syntax::SourceFile::parse and rowan Red/Green CST tokens */
    action def mask_literals_and_comments;
    doc /* Inspect typed ra_ap_syntax ast::IndexExpr CST nodes for direct range-slice indexing operands outside .get(..) calls */
    action def contains_direct_range_slice;
    doc /* Audit fail-closed I/O patterns via typed ra_ap_syntax ast::{MatchExpr, MatchArmList, CallExpr, MethodCallExpr, TryExpr} CST nodes */
    action def audit_fail_closed_io_patterns;
    doc /* Audit #[test] function bodies via typed ra_ap_syntax ast::{Fn, MacroCall, CallExpr} CST nodes for assertion and RAII isolation integrity */
    action def audit_test_integrity_and_anti_gaming;
    doc /* Audit SSOT symbol deduplication across crates/deap-core/src/digest.rs, crates/deap-core/src/rust_cst/, and crates/deap-core/src/dart_cst/ via ra_ap_syntax and cargo_metadata */
    action def audit_ssot_symbol_deduplication;
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0602_BP_02_Check_34_AST_Purity_Fail_Closed_IO_and_Anti_Gaming_Test_Gate_Blueprint {
        id = "REQ-0602";
        text = "Check34AstPurityAndAntiGamingGate (REQ-0602 / BP-02 / DEAP-BP-DET-02, UUIDv5 6ba7b810-9dad-11d1-80b4-00c04fd430c2, Complexity Class P, governed by DO-178C DAL-A and DO-330 TQL-4) SHALL enforce Contract A (full 6-clause OMG SysML v2.0 + KerML 1.0 grammar coverage with zero 16-construct caps) and Contract B (lossless ra_ap_syntax ast::{MatchExpr, MatchArmList, CallExpr, MethodCallExpr, TryExpr, IndexExpr, Fn, MacroCall} + rowan CST verification, ra_ap_vfs, ra_ap_paths, ra_ap_line_index, ra_ap_text_edit, camino, ignore, cargo_metadata, cargo-lock, cargo-util-schemas, cargo-platform, and toml_edit inside < 380-SLOC submodules with zero hand-rolled line/character scanners).";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_general_direct_range_slice_indexing_rejection : String = "- **Given**: Any audited Rust source file in crates/ containing a direct range-slice indexing expression (&s[a..b], &rest[..idx], &buf[start..], or &items[..=end]) parsed into a lossless ra_ap_syntax::SourceFile CST. - **When**: ./target/release/verify-baseline . --no-domain executes Check 34 (C-PURITY-01). - **Then**: RangeSliceAstDetector inspects typed ast::IndexExpr nodes with range operands via ra_ap_syntax and rowan, emits a camino::Utf8Path file-and-line diagnostic via ra_ap_line_index instructing the caller to use .get(..), and exits fail-closed with code 1.";
        attribute ac_02_fail_closed_filesystem_and_walkdir_enforcement : String = "- **Given**: Any workspace Rust module containing .into_iter().flatten(), .filter_map(Result::ok), fs::read_to_string(...).unwrap_or_default(), or let _ = fs::remove_dir_all(...) in its ra_ap_syntax CST. - **When**: Check 34 executes audit_fail_closed_io_patterns over typed ast::{MatchExpr, MatchArmList, CallExpr, MethodCallExpr, TryExpr} nodes (C-PURITY-02). - **Then**: FailClosedIoAstVerifier fails closed, flags every fail-open error-swallowing site with its exact 1-based line number via ra_ap_line_index, and terminates with a non-zero exit code.";
        attribute ac_03_anti_gaming_test_body_ast_enforcement : String = "- **Given**: A #[test] function in any workspace crate that contains zero assert* macros, asserts std::mem::size_of::<T>() > 0, wraps its assertions in an unguarded if Path::new(\"...\").exists() check, or constructs a hardcoded std::env::temp_dir().join(\"...\") path. - **When**: Check 34 executes audit_test_integrity_and_anti_gaming over typed ra_ap_syntax ast::{Fn, MacroCall, CallExpr} nodes (C-PURITY-03). - **Then**: AntiGamingTestBodyAuditor fails closed, rejects the vacuous or un-isolated test function, and halts baseline verification.";
        attribute ac_04_ssot_cryptographic_and_lexer_deduplication : String = "- **Given**: A Rust source file outside crates/deap-core/src/digest.rs that defines SHA-256 round constant 0x428a2f98 or a file outside crates/deap-core/src/rust_cst/ and crates/deap-core/src/dart_cst/ that defines a duplicate Rust or Dart CST parser. - **When**: Check 34 executes audit_ssot_symbol_deduplication via ra_ap_syntax and cargo_metadata (C-PURITY-04 and C-PURITY-05). - **Then**: SsotSymbolDeduplicationGate reports an SSOT duplication violation and exits fail-closed with a non-zero status code.";
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
    connection def BP_02_AuditorToPurityGateConnection {
        connect AntiGamingTestBodyAuditor.audit_ast_out to Check34AstPurityAndAntiGamingGate.TestFunctionAstStreamPort;
    }
    doc /* - **Given**: Any audited Rust source file in crates/ containing a direct range-slice indexing expression (&s[a..b], &rest[..idx], &buf[start..], or &items[..=end]) parsed into a lossless ra_ap_syntax::SourceFile CST. - **When**: ./target/release/verify-baseline . --no-domain executes Check 34 (C-PURITY-01). - **Then**: RangeSliceAstDetector inspects typed ast::IndexExpr nodes with range operands via ra_ap_syntax and rowan, emits a camino::Utf8Path file-and-line diagnostic via ra_ap_line_index instructing the caller to use .get(..), and exits fail-closed with code 1. */
    use case def UC_AC_01_General_Direct_Range_Slice_Indexing_Rejection {
        subject AntiGamingTestBodyAuditor;
        objective "General Direct Range Slice Indexing Rejection";
    }
    doc /* - **Given**: Any workspace Rust module containing .into_iter().flatten(), .filter_map(Result::ok), fs::read_to_string(...).unwrap_or_default(), or let _ = fs::remove_dir_all(...) in its ra_ap_syntax CST. - **When**: Check 34 executes audit_fail_closed_io_patterns over typed ast::{MatchExpr, MatchArmList, CallExpr, MethodCallExpr, TryExpr} nodes (C-PURITY-02). - **Then**: FailClosedIoAstVerifier fails closed, flags every fail-open error-swallowing site with its exact 1-based line number via ra_ap_line_index, and terminates with a non-zero exit code. */
    use case def UC_AC_02_Fail_Closed_Filesystem_and_WalkDir_Enforcement {
        subject AntiGamingTestBodyAuditor;
        objective "Fail Closed Filesystem and WalkDir Enforcement";
    }
    doc /* - **Given**: A #[test] function in any workspace crate that contains zero assert* macros, asserts std::mem::size_of::<T>() > 0, wraps its assertions in an unguarded if Path::new(\"...\").exists() check, or constructs a hardcoded std::env::temp_dir().join(\"...\") path. - **When**: Check 34 executes audit_test_integrity_and_anti_gaming over typed ra_ap_syntax ast::{Fn, MacroCall, CallExpr} nodes (C-PURITY-03). - **Then**: AntiGamingTestBodyAuditor fails closed, rejects the vacuous or un-isolated test function, and halts baseline verification. */
    use case def UC_AC_03_Anti_Gaming_Test_Body_AST_Enforcement {
        subject AntiGamingTestBodyAuditor;
        objective "Anti Gaming Test Body AST Enforcement";
    }
    doc /* - **Given**: A Rust source file outside crates/deap-core/src/digest.rs that defines SHA-256 round constant 0x428a2f98 or a file outside crates/deap-core/src/rust_cst/ and crates/deap-core/src/dart_cst/ that defines a duplicate Rust or Dart CST parser. - **When**: Check 34 executes audit_ssot_symbol_deduplication via ra_ap_syntax and cargo_metadata (C-PURITY-04 and C-PURITY-05). - **Then**: SsotSymbolDeduplicationGate reports an SSOT duplication violation and exits fail-closed with a non-zero status code. */
    use case def UC_AC_04_SSOT_Cryptographic_and_Lexer_Deduplication {
        subject AntiGamingTestBodyAuditor;
        objective "SSOT Cryptographic and Lexer Deduplication";
    }
    doc /* - **Given**: This blueprint document (docs/designs/blueprint-02-check34-ast-purity-and-anti-gaming-gate.md). - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-02-check34-ast-purity-and-anti-gaming-gate.md --dry-run and ./target/release/ingest-sysml are executed. - **Then**: The blueprint verifier passes all 12 mechanical gates, extracts Check34AstPurityAndAntiGamingGate, its subcomponents, 4 typed ports, 5 state attributes, 5 formal constraints (C-PURITY-01 through C-PURITY-05), and 5 BDD acceptance criteria into valid SysML v2, and exits with code 0. */
    use case def UC_AC_05_Deterministic_Dogfood_Ingestion_into_Canonical_SysML_v2 {
        subject AntiGamingTestBodyAuditor;
        objective "Deterministic Dogfood Ingestion into Canonical SysML v2";
    }
    test case def TC_REQ_0602_Check34_Purity_Gate {
        verify requirement REQ_0602;
    }
    test case def TC_REQ_0602_Blueprint_Conformance {
        verify requirement REQ_0602_BP_02_Check_34_AST_Purity_Fail_Closed_IO_and_Anti_Gaming_Test_Gate_Blueprint;
    }
    test case def Test_check_compiler_ast_purity {
        verify requirement REQ_0602;
    }
    test case def Test_mask_literals_and_comments {
        verify requirement REQ_0602;
    }
    test case def Test_contains_direct_range_slice {
        verify requirement REQ_0602;
    }
    test case def Test_audit_fail_closed_io_patterns {
        verify requirement REQ_0602;
    }
    test case def Test_audit_test_integrity_and_anti_gaming {
        verify requirement REQ_0602;
    }
    test case def Test_audit_ssot_symbol_deduplication {
        verify requirement REQ_0602;
    }
    doc /* Extracted from Mermaid sequenceDiagram specification. */
    interaction def BP_02_Check_34_AST_Purity_Fail_Closed_IO_and_Anti_Gaming_Test_Gate_Blueprint_SequenceInteraction {
        lifeline Runner : VerifyBaselineCLI;
        lifeline Gate : Check34PurityGate;
        lifeline Lexer : deap_core_rust_cst;
        lifeline TestAud : AntiGamingAuditor;
        lifeline SSOT : SsotSymbolDeduplicationGate;
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
    doc /* Lossless ra_ap_syntax test AST auditor that inspects typed ast::{Fn, MacroCall, CallExpr} nodes in #[test] functions and rejects zero-assertion bodies, mem::size_of tautologies, unguarded .exists() skips, and non-RAII temp paths */
    part def AntiGamingTestBodyAuditor {
        attribute quantity : Integer = 1;
        attribute type : String = "Subcomponent";
        port out audit_ast_out : String;
    }
    doc /* Total detected panicking direct range-slice indexing expressions (`ast::IndexExpr` with range operands) across audited workspace modules [unit: count] */
    part def Check34AstPurityAndAntiGamingGate {
        attribute direct_range_slice_count : Integer = 0;
        doc /* Total detected fail-open filesystem or directory traversal error-swallowing call sites across workspace crates [unit: count] */
        attribute fail_open_io_site_count : Integer = 0;
        doc /* Total `#[test]` functions exhibiting zero assertions, `mem::size_of` dummy checks, conditional `.exists()` skips, or shared temp paths [unit: count] */
        attribute vacuous_test_count : Integer = 0;
        doc /* Total occurrences of SHA-256 round constant `0x428a2f98` outside `crates/deap-core/src/digest.rs` [unit: count] */
        attribute duplicate_sha256_impl_count : Integer = 0;
        doc /* Total duplicate Rust or Dart CST parsers defined outside `crates/deap-core/src/rust_cst/` and `crates/deap-core/src/dart_cst/` [unit: count] */
        attribute duplicate_lexer_impl_count : Integer = 0;
        doc /* Receives lossless `ra_ap_syntax::SourceFile` Red/Green CSTs (`RustCstSourceFile`) and structural declarations across `crates/ * /src/ ** / *.rs` via `ra_ap_vfs` and `camino::Utf8Path` */
        port in WorkspaceRustCstPort : String {
            attribute protocol_family : String = "RustTokenStream";
            doc /* Receives lossless `ra_ap_syntax::SourceFile` Red/Green CSTs (`RustCstSourceFile`) and structural declarations across `crates/ * /src/ ** / *.rs` via `ra_ap_vfs` and `camino::Utf8Path` */
            flow in WorkspaceRustCstPort_flow : String;
        }
        doc /* Receives parsed `#[test]` function `ra_ap_syntax` `ast::Fn` nodes (`RustFnAst`), statement blocks, and `ast::MacroCall` invocation trees */
        port in TestFunctionAstStreamPort : String {
            attribute protocol_family : String = "RustTestAstStream";
            doc /* Receives parsed `#[test]` function `ra_ap_syntax` `ast::Fn` nodes (`RustFnAst`), statement blocks, and `ast::MacroCall` invocation trees */
            flow in TestFunctionAstStreamPort_flow : String;
        }
        doc /* Emits sorted, `ra_ap_line_index`-anchored diagnostic violations (`PurityViolationRecord`) when any Check 34 sub-gate fails */
        port out PurityViolationDiagnosticPort : String {
            attribute protocol_family : String = "DeterministicDiag";
            doc /* Emits sorted, `ra_ap_line_index`-anchored diagnostic violations (`PurityViolationRecord`) when any Check 34 sub-gate fails */
            flow out PurityViolationDiagnosticPort_flow : String;
        }
        doc /* Emits deterministic pass (`0`) or fail-closed (`1`) status (`GateVerdict`) to the `verify-baseline` runner */
        port out Check34GateStatusPort : Integer {
            attribute protocol_family : String = "ExitCodeContract";
            doc /* Emits deterministic pass (`0`) or fail-closed (`1`) status (`GateVerdict`) to the `verify-baseline` runner */
            flow out Check34GateStatusPort_flow : Integer;
        }
    }
    doc /* Typed ra_ap_syntax AST pattern verifier inspecting ast::{MatchExpr, MatchArmList, CallExpr, MethodCallExpr, TryExpr} nodes to reject fail-open iterator flattening, .filter_map(Result::ok), .unwrap_or_default() on filesystem reads, and let _ = fs:: error discarding */
    part def FailClosedIoAstVerifier {
        attribute quantity : Integer = 1;
        attribute type : String = "Subcomponent";
    }
    doc /* Lossless ra_ap_syntax + rowan CST analyzer inspecting typed ast::IndexExpr nodes to reject panicking Rust bracket range indexing (expr[a..b], expr[..b], expr[a..]) outside .get(..) calls */
    part def RangeSliceAstDetector {
        attribute quantity : Integer = 1;
        attribute type : String = "Subcomponent";
    }
    doc /* Workspace-wide ra_ap_syntax, cargo_metadata, cargo-lock, cargo-util-schemas, cargo-platform, and toml_edit symbol CST/AST verifier enforcing single-module residency for SHA-256 constants (0x428a2f98) in digest.rs and CST parsers in rust_cst/ and dart_cst/ */
    part def SsotSymbolDeduplicationGate {
        attribute quantity : Integer = 1;
        attribute type : String = "Subcomponent";
    }
}

doc /* Subsystem package for BP_03_SysML_v2_AST_Bijective_Roundtrip_and_Token_Stream_Parser_Gate_Blueprint */
package BP_03_SysML_v2_AST_Bijective_Roundtrip_and_Token_Stream_Parser_Gate_Blueprint {
    port def BP_03_BijectiveRoundtripPortDef {
        flow in raw_sysml_in : String;
        flow out canonical_sysml_out : String;
    }
    doc /* Tokenize raw SysML v2 and KerML 1.0 source via logos DFA lexer into rowan GreenNode tokens with zero span drift */
    action def scan_all;
    doc /* Parse full 6-clause OMG SysML v2.0 + KerML 1.0 grammar into lossless rowan Red/Green CST (deap_core::sysml_cst) and salsa HIR */
    action def parse_package_or_file;
    doc /* Parse KerML 1.0 type expressions, specialization operators (:>, :>>, ::>, ~), and nested state def transitions over rowan CST */
    action def ParseTypeExprAndParseTransition;
    doc /* Verify Stage-1 == Stage-2 bijective CST/AST roundtrip parity via verify_bijective_ast_roundtrip(ast_orig) */
    action def verify_bijective_ast_roundtrip;
    doc /* Emit canonical SysML v2.0 text from lossless rowan CST and PackageDef HIR via ast_orig.to_sysml(0) */
    action def ast_orig_to_sysml;
    doc /* Assert Stage-1 PackageDef HIR equals Stage-2 reparsed PackageDef HIR with ariadne span diagnostics on drift */
    action def AssertAstOrigEqualsAstReparsed;
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0603_BP_03_SysML_v2_AST_Bijective_Roundtrip_and_Token_Stream_Parser_Gate_Blueprint {
        id = "REQ-0603";
        text = "The Digital Engineering Agent Platform (DEAP) compiler pipeline (DEAP-BP-DET-03, subsystem SysmlBijectiveRoundtripCompilerGate, deterministic UUIDv5 6ba7b810-9dad-11d1-80b4-00c04fd430c3, complexity class Class P, diagnostic code bindings E0100, E0200, E0300, E0400, governed by ISO/IEC/IEEE 15288:2023, OMG SysML v2.0, KerML 1.0, DO-178C DAL-A, and DO-330 TQL-4) SHALL enforce Contract A (full normative OMG SysML v2.0 + KerML 1.0 textual notation coverage across all 6 clauses with full Definition & Usage symmetry and zero 16-construct subset caps: package/library package, import, alias, comment/doc/rep, attribute def/attribute, enum def/enum, item def/item, part def/part, port def/port with ~ conjugation and in/out/inout, connection def/connection, interface def/interface, flow def/flow/succession, allocation def/allocation/allocate, occurrence def/occurrence/individual/snapshot/timeslice, action def/action with perform/accept/send/assign/if/while/for/fork/join/merge/decide/first ... then, state def/state/exhibit with entry/do/exit/transition, calc def/calc, constraint def/constraint with assert/assume/require, requirement def/requirement with subject/actor/stakeholder/framed concern/satisfy, concern def/concern, use case def/use case with include use case, analysis def/analysis with objective, verification def/verification with verify, view def/view, viewpoint def/viewpoint, rendering def/rendering with expose/filter, metadata def/metadata/@/# user-defined keyword extensions supporting #hazard/#risk/#capability/#rt_loop/#dal/#fls via standard OMG SysML v2 metadata keywords while retaining backward-compatible parsing of legacy hazard def/risk def/capability def, variation/variant, and full KerML specialization & feature operators : defined by, :> subsets, :>> redefines, ::> references, ~ conjugation, ref, abstract, readonly, derived, end, multiplicity [lo..hi] / {ordered, nonunique}, full KerML operator expressions, and ISQ/SI/ScalarValues standard library imports) and Contract B (logos DFA + rowan lossless Red/Green CST deap_core::sysml_cst, deap_core::dart_cst, salsa incremental query database compile_sysml::salsa_db, ariadne source-span diagnostics, ingest_sysml::cst::{idl_cst, yang_cst}, ra_ap_syntax, and serde_json inside < 380-SLOC submodules with zero hand-rolled recursive-descent scanner monoliths).";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_native_lexer_sigil_and_lifetime_tokenization : String = "- **Given**: A SysML v2.0 / KerML 1.0 model containing reference type sigils (&), optionality sigils (?), lifetime annotations ('a), qualified paths (::), specialization and feature operators (: defined by, :> subsets, :>> redefines, ::> references, ~ conjugation, ref, abstract, readonly, derived, end, [lo..hi] / {ordered, nonunique}), and logical && expressions (#495). - **When**: deap_core::sysml_cst::lexer (logos DFA) tokenizes the raw UTF-8 source buffer into rowan GreenNode tokens without pre-lexer string rewriting. - **Then**: The logos lexer emits native token kinds with exact 1:1 UTF-8 byte and column offsets (span_offset_drift_bytes == 0) and preserves 100% of comments and whitespace trivia in rowan.";
        attribute ac_02_fail_closed_inner_block_and_composite_type_parsing : String = "- **Given**: Any construct across all 6 clauses of the full normative OMG SysML v2.0 + KerML 1.0 grammar with full Definition & Usage symmetry (package/library package, import, alias, comment/doc/rep, attribute def/attribute, enum def/enum, item def/item, part def/part, port def/port with ~ and in/out/inout, connection def/connection, interface def/interface, flow def/flow/succession, allocation def/allocation/allocate, occurrence def/occurrence/individual/snapshot/timeslice, action def/action, state def/state/exhibit, calc def/calc, constraint def/constraint, requirement def/requirement, concern def/concern, use case def/use case, analysis def/analysis, verification def/verification, view def/view, viewpoint def/viewpoint, rendering def/rendering, metadata def/metadata/@/# for #hazard/#risk/#capability + backward-compatible hazard def/risk def/capability def, variation/variant, and KerML 1.0 specialization/multiplicity/expressions with ISQ/SI/ScalarValues, zero 16-construct caps) or an invalid inner statement (#504). - **When**: deap_core::sysml_cst builds the rowan Red/Green CST and lowers into salsa HIR. - **Then**: Valid composite types and 6-clause constructs are preserved verbatim and any unrecognized statement fails closed with an ariadne byte-span diagnostic E0200 (silent_skip_branch_count == 0).";
        attribute ac_03_token_stream_state_transition_parsing_and_recursive_mbd_synthesis : String = "- **Given**: A hierarchical state def / state usage containing anonymous and named transition statements (first S1 accept trig if grd do eff then S2;), entry/do/exit actions, and nested composite sub_states with their own transitions and doc blocks (#496, #509). - **When**: deap_core::sysml_cst and the MBD .sfx synthesizer execute via salsa tracked queries. - **Then**: Transitions are lowered directly from typed rowan CST nodes without fabricating {s}_to_{t} names (name: None preserved when unnamed), sub-state doc blocks attach to their target StateDef or TransitionDef, and MBD .sfx synthesis recursively emits transitions across all nesting depths.";
        attribute ac_04_lossless_to_sysml_serialization_and_fail_closed_bijective_roundtrip_gate : String = "- **Given**: A populated PackageDef HIR and rowan CST containing 6-clause SysML v2.0 + KerML 1.0 definitions and usages (#493, #497, #505), or a mutated AST with a simulated field divergence. - **When**: verify_bijective_ast_roundtrip(&pkg) executes during compile-sysml via salsa memoized queries. - **Then**: pkg.to_sysml(0) emits valid canonical SysML v2.0 text, re-parsing via deap_core::sysml_cst returns a PackageDef structurally equal to pkg (roundtrip_ast_mismatch_count == 0), and any field divergence fails closed with ariadne diagnostic E0400.";
        attribute ac_05_dogfood_markdown_to_sysml_ingestion_and_baseline_verification : String = "- **Given**: This blueprint document docs/designs/blueprint-03-sysml-ast-bijective-roundtrip-compiler-gate.md (REQ-0603). - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-03-sysml-ast-bijective-roundtrip-compiler-gate.md --dry-run is executed. - **Then**: verify-baseline check-blueprint passes all 12 mechanical gates, extracts SysmlBijectiveRoundtripCompilerGate with all BOM parts, interface ports, attributes, parametric constraints (C-RT-01..C-RT-04), and REQ_0603 requirement definitions, and persists .sysml and .digest.json with zero bijective roundtrip drift.";
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
        satisfy by BijectiveSysmlSerializer;
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
        transition Initial_to_LexicalScanning first Initial then LexicalScanning;
        transition LexicalScanning_to_RecursiveDescentParsing_on_zeroDriftTokenStream first LexicalScanning accept zeroDriftTokenStream then RecursiveDescentParsing;
        transition LexicalScanning_to_RejectedE0100_on_lexicalSpanError first LexicalScanning accept lexicalSpanError then RejectedE0100;
        transition RecursiveDescentParsing_to_CanonicalSerialization_on_packageDefConstructed first RecursiveDescentParsing accept packageDefConstructed then CanonicalSerialization;
        transition RecursiveDescentParsing_to_RejectedE0200_on_unexpectedInnerBlockToken first RecursiveDescentParsing accept unexpectedInnerBlockToken then RejectedE0200;
        transition CanonicalSerialization_to_RoundtripReparsing_on_sysmlTextEmitted first CanonicalSerialization accept sysmlTextEmitted then RoundtripReparsing;
        transition RoundtripReparsing_to_AstStructuralEqualityCheck_on_reparsedPackageDef first RoundtripReparsing accept reparsedPackageDef then AstStructuralEqualityCheck;
        transition RoundtripReparsing_to_RejectedE0400_on_reparseSyntaxFailure first RoundtripReparsing accept reparseSyntaxFailure then RejectedE0400;
        transition AstStructuralEqualityCheck_to_VerifiedBijectiveAst_on_astOrigEqualsAstReparsed first AstStructuralEqualityCheck accept astOrigEqualsAstReparsed then VerifiedBijectiveAst;
        transition AstStructuralEqualityCheck_to_RejectedE0400_on_structuralAstDiffDetected first AstStructuralEqualityCheck accept structuralAstDiffDetected then RejectedE0400;
        transition RejectedE0100_to_Terminal first RejectedE0100 then Terminal;
        transition RejectedE0200_to_Terminal first RejectedE0200 then Terminal;
        transition RejectedE0400_to_Terminal first RejectedE0400 then Terminal;
        transition VerifiedBijectiveAst_to_Terminal first VerifiedBijectiveAst then Terminal;
    }
    connection def BP_03_SerializerToRoundtripGateConnection {
        connect BijectiveSysmlSerializer.serialized_sysml_out to SysmlBijectiveRoundtripCompilerGate.RawSysmlSourceInputPort;
    }
    doc /* - **Given**: A SysML v2.0 / KerML 1.0 model containing reference type sigils (&), optionality sigils (?), lifetime annotations ('a), qualified paths (::), specialization and feature operators (: defined by, :> subsets, :>> redefines, ::> references, ~ conjugation, ref, abstract, readonly, derived, end, [lo..hi] / {ordered, nonunique}), and logical && expressions (#495). - **When**: deap_core::sysml_cst::lexer (logos DFA) tokenizes the raw UTF-8 source buffer into rowan GreenNode tokens without pre-lexer string rewriting. - **Then**: The logos lexer emits native token kinds with exact 1:1 UTF-8 byte and column offsets (span_offset_drift_bytes == 0) and preserves 100% of comments and whitespace trivia in rowan. */
    use case def UC_AC_01_Native_Lexer_Sigil_and_Lifetime_Tokenization {
        subject BijectiveSysmlSerializer;
        objective "Native Lexer Sigil and Lifetime Tokenization";
    }
    doc /* - **Given**: Any construct across all 6 clauses of the full normative OMG SysML v2.0 + KerML 1.0 grammar with full Definition & Usage symmetry (package/library package, import, alias, comment/doc/rep, attribute def/attribute, enum def/enum, item def/item, part def/part, port def/port with ~ and in/out/inout, connection def/connection, interface def/interface, flow def/flow/succession, allocation def/allocation/allocate, occurrence def/occurrence/individual/snapshot/timeslice, action def/action, state def/state/exhibit, calc def/calc, constraint def/constraint, requirement def/requirement, concern def/concern, use case def/use case, analysis def/analysis, verification def/verification, view def/view, viewpoint def/viewpoint, rendering def/rendering, metadata def/metadata/@/# for #hazard/#risk/#capability + backward-compatible hazard def/risk def/capability def, variation/variant, and KerML 1.0 specialization/multiplicity/expressions with ISQ/SI/ScalarValues, zero 16-construct caps) or an invalid inner statement (#504). - **When**: deap_core::sysml_cst builds the rowan Red/Green CST and lowers into salsa HIR. - **Then**: Valid composite types and 6-clause constructs are preserved verbatim and any unrecognized statement fails closed with an ariadne byte-span diagnostic E0200 (silent_skip_branch_count == 0). */
    use case def UC_AC_02_Fail_Closed_Inner_Block_and_Composite_Type_Parsing {
        subject BijectiveSysmlSerializer;
        objective "Fail Closed Inner Block and Composite Type Parsing";
    }
    doc /* - **Given**: A hierarchical state def / state usage containing anonymous and named transition statements (first S1 accept trig if grd do eff then S2;), entry/do/exit actions, and nested composite sub_states with their own transitions and doc blocks (#496, #509). - **When**: deap_core::sysml_cst and the MBD .sfx synthesizer execute via salsa tracked queries. - **Then**: Transitions are lowered directly from typed rowan CST nodes without fabricating {s}_to_{t} names (name: None preserved when unnamed), sub-state doc blocks attach to their target StateDef or TransitionDef, and MBD .sfx synthesis recursively emits transitions across all nesting depths. */
    use case def UC_AC_03_Token_Stream_State_Transition_Parsing_and_Recursive_MBD_Synthesis {
        subject BijectiveSysmlSerializer;
        objective "Token Stream State Transition Parsing and Recursive MBD Synthesis";
    }
    doc /* - **Given**: A populated PackageDef HIR and rowan CST containing 6-clause SysML v2.0 + KerML 1.0 definitions and usages (#493, #497, #505), or a mutated AST with a simulated field divergence. - **When**: verify_bijective_ast_roundtrip(&pkg) executes during compile-sysml via salsa memoized queries. - **Then**: pkg.to_sysml(0) emits valid canonical SysML v2.0 text, re-parsing via deap_core::sysml_cst returns a PackageDef structurally equal to pkg (roundtrip_ast_mismatch_count == 0), and any field divergence fails closed with ariadne diagnostic E0400. */
    use case def UC_AC_04_Lossless_to_sysml_Serialization_and_Fail_Closed_Bijective_Roundtrip_Gate {
        subject BijectiveSysmlSerializer;
        objective "Lossless to sysml Serialization and Fail Closed Bijective Roundtrip Gate";
    }
    doc /* - **Given**: This blueprint document docs/designs/blueprint-03-sysml-ast-bijective-roundtrip-compiler-gate.md (REQ-0603). - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-03-sysml-ast-bijective-roundtrip-compiler-gate.md --dry-run is executed. - **Then**: verify-baseline check-blueprint passes all 12 mechanical gates, extracts SysmlBijectiveRoundtripCompilerGate with all BOM parts, interface ports, attributes, parametric constraints (C-RT-01..C-RT-04), and REQ_0603 requirement definitions, and persists .sysml and .digest.json with zero bijective roundtrip drift. */
    use case def UC_AC_05_Dogfood_Markdown_to_SysML_Ingestion_and_Baseline_Verification {
        subject BijectiveSysmlSerializer;
        objective "Dogfood Markdown to SysML Ingestion and Baseline Verification";
    }
    test case def TC_REQ_0603_Bijective_Roundtrip_Gate {
        verify requirement REQ_0603;
    }
    test case def TC_REQ_0603_Blueprint_Conformance {
        verify requirement REQ_0603_BP_03_SysML_v2_AST_Bijective_Roundtrip_and_Token_Stream_Parser_Gate_Blueprint;
    }
    test case def Test_scan_all {
        verify requirement REQ_0603;
    }
    test case def Test_parse_package_or_file {
        verify requirement REQ_0603;
    }
    test case def Test_ParseTypeExprAndParseTransition {
        verify requirement REQ_0603;
    }
    test case def Test_verify_bijective_ast_roundtrip {
        verify requirement REQ_0603;
    }
    test case def Test_ast_orig_to_sysml {
        verify requirement REQ_0603;
    }
    test case def Test_AssertAstOrigEqualsAstReparsed {
        verify requirement REQ_0603;
    }
    doc /* Extracted from Mermaid sequenceDiagram specification. */
    interaction def BP_03_SysML_v2_AST_Bijective_Roundtrip_and_Token_Stream_Parser_Gate_Blueprint_SequenceInteraction {
        lifeline CLI : Compiler_CLI_Caller;
        lifeline Scan : NativeSigilAndLifetimeLexer;
        lifeline Parse : FailClosedRecursiveDescentParser;
        lifeline Ser : BijectiveSysmlSerializer;
        lifeline Gate : CompilerRoundtripVerificationGate;
        message scan_all from CLI to Scan;
        message reply_VecOfTokenWithZeroSpan from Scan to CLI;
        message parse_package_or_file from CLI to Parse;
        message ParseTypeExprAndParseTransition from Parse to Parse;
        message reply_CanonicalPackageDefAst from Parse to CLI;
        message verify_bijective_ast_roundtrip from CLI to Gate;
        message ast_orig_to_sysml from Gate to Ser;
        message reply_CanonicalSysMLV2Text from Ser to Gate;
        message scan_all from Gate to Scan;
        message reply_ReparsedTokenStream from Scan to Gate;
        message parse_package_or_file from Gate to Parse;
        message reply_ReparsedPackageDefAst from Parse to Gate;
        message AssertAstOrigEqualsAstReparsed from Gate to Gate;
        message reply_OkOrErr from Gate to CLI;
    }
    doc /* Canonical rowan CST printer and SysmlSerializable::to_sysml formatter preserving 100% of 6-clause OMG SysML v2.0 + KerML 1.0 AST fields and trivia without double-quoting */
    part def BijectiveSysmlSerializer {
        attribute quantity : Integer = 1;
        attribute type : String = "Subcomponent";
        port out serialized_sysml_out : String;
    }
    doc /* Built-in salsa-tracked compiler verification stage asserting parse_sysml(to_sysml(ast)) == ast and Stage-1 == Stage-2 SHA-256 parity on every compile-sysml run */
    part def CompilerRoundtripVerificationGate {
        attribute quantity : Integer = 1;
        attribute type : String = "Subcomponent";
    }
    doc /* Lossless rowan Red/Green CST parser (deap_core::sysml_cst) covering all 6 clauses of the normative OMG SysML v2.0 + KerML 1.0 grammar with zero 16-construct caps and zero silent-skip branches */
    part def FailClosedRecursiveDescentParser {
        attribute quantity : Integer = 1;
        attribute type : String = "Subcomponent";
    }
    doc /* Zero-rewrite logos DFA lexer (deap_core::sysml_cst::lexer) emitting native sigil, KerML 1.0 specialization (:>, :>>, ::>, ~), lifetime, and trivia tokens with exact UTF-8 byte spans */
    part def NativeSigilAndLifetimeLexer {
        attribute quantity : Integer = 1;
        attribute type : String = "Subcomponent";
    }
    doc /* Cumulative byte-offset drift between raw source character positions and emitted logos + rowan token spans [unit: bytes] */
    part def SysmlBijectiveRoundtripCompilerGate {
        attribute span_offset_drift_bytes : Integer = 0;
        doc /* Count of silent erasure fallback branches in deap_core::sysml_cst 6-clause SysML v2.0 + KerML 1.0 CST parsers [unit: branches] */
        attribute silent_skip_branch_count : Integer = 0;
        doc /* Count of AST struct fields omitted during rowan CST printing and SysmlSerializable::to_sysml serialization [unit: fields] */
        attribute dropped_ast_field_count : Integer = 0;
        doc /* Count of structural AST nodes differing between ast and parse_sysml(to_sysml(ast)) in salsa roundtrip verification [unit: nodes] */
        attribute roundtrip_ast_mismatch_count : Integer = 0;
        doc /* Ingests unmodified UTF-8 SysML v2.0 / KerML 1.0 source text via camino::Utf8Path without pre-lexer string normalization */
        port in RawSysmlSourceInputPort : String {
            attribute protocol_family : String = "UTF8Buffer";
            doc /* Ingests unmodified UTF-8 SysML v2.0 / KerML 1.0 source text via camino::Utf8Path without pre-lexer string normalization */
            flow in RawSysmlSourceInputPort_flow : String;
        }
        doc /* Streams span-exact logos + rowan GreenNode token records from NativeSigilAndLifetimeLexer to FailClosedRecursiveDescentParser */
        port inout TokenStreamChannelPort : String {
            attribute protocol_family : String = "ZeroCopySlice";
        }
        doc /* Emits deterministic, human-readable SysML v2.0 text synthesized by BijectiveSysmlSerializer */
        port out SerializedSysmlOutputPort : String {
            attribute protocol_family : String = "CanonicalSysmlV2";
            doc /* Emits deterministic, human-readable SysML v2.0 text synthesized by BijectiveSysmlSerializer */
            flow out SerializedSysmlOutputPort_flow : String;
        }
        doc /* Emits fail-closed ariadne source-span and single-line JSON structural AST diff diagnostics (E0100..E0400) on roundtrip mismatch */
        port out BijectiveDiffDiagnosticPort : String {
            attribute protocol_family : String = "StructuredDiagnostic";
            doc /* Emits fail-closed ariadne source-span and single-line JSON structural AST diff diagnostics (E0100..E0400) on roundtrip mismatch */
            flow out BijectiveDiffDiagnosticPort_flow : String;
        }
    }
}

doc /* Subsystem package for BP_04_Deterministic_Specification_ICD_and_WBS_Forward_Compiler_Blueprint */
package BP_04_Deterministic_Specification_ICD_and_WBS_Forward_Compiler_Blueprint {
    port def BP_04_ForwardProjectionPortDef {
        flow in sysml_ast_in : String;
        flow out markdown_spec_out : String;
    }
    doc /* Execute compile-sysml --forward-sync docs via salsa derived queries and pulldown-cmark-to-cmark */
    action def CompileSysmlForwardSyncDocs;
    doc /* Resolve Parent Epic by pkg_name == ConOps across 6-clause SysML v2.0 PackageDef HIR */
    action def ResolveParentEpicByPkgName;
    doc /* Bind all package stories to Use Case matrices via BTreeMap-ordered salsa queries */
    action def BindAllPackageStoriesToUse;
    doc /* Emit symmetric ActionDef signatures and Check 13B E1-E5 Mermaid via pulldown-cmark-to-cmark */
    action def EmitSymmetricActionDefSignaturesAndE1;
    doc /* Execute compile-sysml --emit-icd docs/interfaces via salsa interconnect projection */
    action def CompileSysmlEmitIcdDocsInterfaces;
    doc /* Project NxN ConnectionDef matrix and FlowDef signals via BTreeMap-ordered pulldown-cmark-to-cmark tables */
    action def ProjectNxNConnectionDefMatrixAndFlowDef;
    doc /* Execute compile-sysml --emit-wbs docs/management via MIL-STD-881E salsa projection */
    action def CompileSysmlEmitWbsDocsManagement;
    doc /* Synthesize MIL-STD-881E tree and RFC 4180 CSV / serde_json AST exports */
    action def SynthesizeMilStd881eTreeAnd;
    doc /* Execute reconcile-backlog reconcile --docs-dir docs via pulldown-cmark Event::Html and table AST */
    action def ReconcileBacklogReconcileDocsDirDocs;
    doc /* Parse CommonMark tables and HTML comment events via pulldown-cmark Event::Html and Tag::Table AST */
    action def ParseCommonMarkTablesWithInHtml;
    doc /* Reconcile checklists with parent-inclusive epic count via pulldown-cmark-to-cmark lossless serialization */
    action def ReconcileChecklistsWithParentInclusiveEpic;
    doc /* Execute compile-sysml --reverse-sync docs via pulldown-cmark AST and rowan CST splicing */
    action def CompileSysmlReverseSyncDocs;
    doc /* Merge extracted entities into owning subpackages via deap_core::sysml_cst formatting-preserving splices */
    action def MergeExtractedEntitiesIntoOwningSubpackages;
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0604_BP_04_Deterministic_Specification_ICD_and_WBS_Forward_Compiler_Blueprint {
        id = "REQ-0604";
        text = "The Digital Engineering Agent Platform (DEAP) compiler pipeline (DeterministicSpecIcdWbsForwardCompiler, REQ-0604 / BP-04 / DEAP-BP-DET-04, UUIDv5 6ba7b810-9dad-11d1-80b4-00c04fd430c4, Complexity Class P, governed by ISO/IEC/IEEE 29148:2018, MIL-STD-881E, DO-178C DAL-A, and DO-330 TQL-4) SHALL deterministically project all Agile functional specifications (docs/epics/, docs/features/, docs/user-stories/, docs/use-cases/), Logical Interface Control Documents (docs/interfaces/ICD_01_SYSTEM_INTERFACE_MATRIX.md, ICD_02_MASTER_SIGNAL_DICTIONARY.md), and MIL-STD-881E Work Breakdown Structures (docs/management/WBS_01..03) under Contract A (full 6-clause OMG SysML v2.0 + KerML 1.0 grammar coverage with zero 16-construct caps) and Contract B (pulldown-cmark + pulldown-cmark-to-cmark lossless CommonMark AST, toml_edit, salsa derived projection queries, Logos+rowan SysML CST splicing, and camino::Utf8Path inside < 380-SLOC submodules with zero hand-rolled Markdown table/HTML scanners and zero LLM subagents).";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_deterministic_multi_story_use_case_and_conops_parent_epic_projection : String = "- **Given**: A multi-package 6-clause OMG SysML v2.0 + KerML 1.0 model loaded into compile_sysml::salsa_db. - **When**: compile-sysml --forward-sync docs executes via salsa derived queries and pulldown-cmark-to-cmark. - **Then**: Epics, Features, User Stories, and Use Cases are deterministically projected with ConOps parent epic resolution, full story-to-use-case realization matrices, symmetric ActionDef signatures, and Check 13B E1-E5 Mermaid diagrams with zero LLM subagents (Diagnostic: E0100).";
        attribute ac_02_deterministic_logical_icd_matrix_and_signal_dictionary_synthesis : String = "- **Given**: SysML v2.0 PartDef, PortDef, InterfaceDef, ConnectionDef, FlowDef, and ItemDef HIR nodes in compile_sysml::salsa_db. - **When**: compile-sysml --emit-icd docs/interfaces executes. - **Then**: LogicalIcdMatrixCompiler emits BTreeMap-ordered ICD_01_SYSTEM_INTERFACE_MATRIX.md and ICD_02_MASTER_SIGNAL_DICTIONARY.md via pulldown-cmark-to-cmark in <= 10 ms (Diagnostic: E0200).";
        attribute ac_03_deterministic_mil_std_881e_wbs_and_enterprise_export_synthesis : String = "- **Given**: Synchronized PackageDef HIR hierarchy and docs/ frontmatter tables parsed via pulldown-cmark and toml_edit. - **When**: compile-sysml --emit-wbs docs/management executes. - **Then**: MilStd881eWbsExporter emits 4-level MIL-STD-881E WBS_01..WBS_03 Markdown registers via pulldown-cmark-to-cmark alongside RFC 4180 CSV and typed serde_json AST exports in <= 15 ms (Diagnostic: E0300).";
        attribute ac_04_fail_closed_stateful_html_comment_frontmatter_guard_and_parent_inclusive_checklist_reconciliation : String = "- **Given**: Specification Markdown files containing HTML comment blocks and CommonMark frontmatter tables under docs/. - **When**: reconcile-backlog reconcile --docs-dir docs parses the files via pulldown-cmark Event::Html and Tag::Table AST. - **Then**: Commented example tables inside Event::Html blocks never spoof frontmatter issue IDs, parent-inclusive epic counts prevent wildcard mismatches, and checklists are losslessly updated via pulldown-cmark-to-cmark (Diagnostic: E0300).";
        attribute ac_05_fail_closed_subpackage_scoped_reverse_sync_and_dogfood_blueprint_ingestion_parity : String = "- **Given**: Modified docs/ specifications and docs/designs/blueprint-04-deterministic-spec-icd-wbs-forward-compiler.md. - **When**: compile-sysml --reverse-sync docs and verify-baseline check-blueprint execute. - **Then**: BidirectionalBacklogReconciler merges entities into their owning PackageDef subpackages via deap_core::sysml_cst formatting-preserving rowan splices with zero root AST pollution, and all 12 blueprint gates pass with exit code 0 (Diagnostic: E0400).";
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
        transition Initial_to_AstLoaded first Initial then AstLoaded;
        transition AstLoaded_to_ForwardProjected_on_forwardSyncSpecs first AstLoaded accept forwardSyncSpecs then ForwardProjected;
        transition ForwardProjected_to_IcdSynthesized_on_emitLogicalIcd first ForwardProjected accept emitLogicalIcd then IcdSynthesized;
        transition IcdSynthesized_to_WbsExported_on_emitMilStd881eWbs first IcdSynthesized accept emitMilStd881eWbs then WbsExported;
        transition WbsExported_to_BacklogReconciled_on_reconcileBacklog first WbsExported accept reconcileBacklog then BacklogReconciled;
        transition BacklogReconciled_to_ReverseSynced_on_reverseSyncToSubpackages first BacklogReconciled accept reverseSyncToSubpackages then ReverseSynced;
        transition BacklogReconciled_to_Rejected_on_htmlCommentSpoofOrCardinalityError first BacklogReconciled accept htmlCommentSpoofOrCardinalityError then Rejected;
        transition ReverseSynced_to_VerifiedRoundtrip_on_sha256DigestParityZero first ReverseSynced accept sha256DigestParityZero then VerifiedRoundtrip;
        transition ReverseSynced_to_Rejected_on_rootAstPollutionOrDriftDetected first ReverseSynced accept rootAstPollutionOrDriftDetected then Rejected;
        transition Rejected_to_Terminal first Rejected then Terminal;
        transition VerifiedRoundtrip_to_Terminal first VerifiedRoundtrip then Terminal;
    }
    connection def BP_04_ReconcilerToForwardCompilerConnection {
        connect BidirectionalBacklogReconciler.subpackageSyncOutPort to DeterministicSpecIcdWbsForwardCompiler.SysmlPackageAstInputPort;
    }
    doc /* - **Given**: A multi-package 6-clause OMG SysML v2.0 + KerML 1.0 model loaded into compile_sysml::salsa_db. - **When**: compile-sysml --forward-sync docs executes via salsa derived queries and pulldown-cmark-to-cmark. - **Then**: Epics, Features, User Stories, and Use Cases are deterministically projected with ConOps parent epic resolution, full story-to-use-case realization matrices, symmetric ActionDef signatures, and Check 13B E1-E5 Mermaid diagrams with zero LLM subagents (Diagnostic: E0100). */
    use case def UC_AC_01_Deterministic_Multi_Story_Use_Case_and_ConOps_Parent_Epic_Projection {
        subject AgileSpecForwardProjector;
        objective "Deterministic Multi Story Use Case and ConOps Parent Epic Projection";
    }
    doc /* - **Given**: SysML v2.0 PartDef, PortDef, InterfaceDef, ConnectionDef, FlowDef, and ItemDef HIR nodes in compile_sysml::salsa_db. - **When**: compile-sysml --emit-icd docs/interfaces executes. - **Then**: LogicalIcdMatrixCompiler emits BTreeMap-ordered ICD_01_SYSTEM_INTERFACE_MATRIX.md and ICD_02_MASTER_SIGNAL_DICTIONARY.md via pulldown-cmark-to-cmark in <= 10 ms (Diagnostic: E0200). */
    use case def UC_AC_02_Deterministic_Logical_ICD_Matrix_and_Signal_Dictionary_Synthesis {
        subject AgileSpecForwardProjector;
        objective "Deterministic Logical ICD Matrix and Signal Dictionary Synthesis";
    }
    doc /* - **Given**: Synchronized PackageDef HIR hierarchy and docs/ frontmatter tables parsed via pulldown-cmark and toml_edit. - **When**: compile-sysml --emit-wbs docs/management executes. - **Then**: MilStd881eWbsExporter emits 4-level MIL-STD-881E WBS_01..WBS_03 Markdown registers via pulldown-cmark-to-cmark alongside RFC 4180 CSV and typed serde_json AST exports in <= 15 ms (Diagnostic: E0300). */
    use case def UC_AC_03_Deterministic_MIL_STD_881E_WBS_and_Enterprise_Export_Synthesis {
        subject AgileSpecForwardProjector;
        objective "Deterministic MIL STD 881E WBS and Enterprise Export Synthesis";
    }
    doc /* - **Given**: Specification Markdown files containing HTML comment blocks and CommonMark frontmatter tables under docs/. - **When**: reconcile-backlog reconcile --docs-dir docs parses the files via pulldown-cmark Event::Html and Tag::Table AST. - **Then**: Commented example tables inside Event::Html blocks never spoof frontmatter issue IDs, parent-inclusive epic counts prevent wildcard mismatches, and checklists are losslessly updated via pulldown-cmark-to-cmark (Diagnostic: E0300). */
    use case def UC_AC_04_Fail_Closed_Stateful_HTML_Comment_Frontmatter_Guard_and_Parent_Inclusive_Checklist_Reconciliation {
        subject AgileSpecForwardProjector;
        objective "Fail Closed Stateful HTML Comment Frontmatter Guard and Parent Inclusive Checklist Reconciliation";
    }
    doc /* - **Given**: Modified docs/ specifications and docs/designs/blueprint-04-deterministic-spec-icd-wbs-forward-compiler.md. - **When**: compile-sysml --reverse-sync docs and verify-baseline check-blueprint execute. - **Then**: BidirectionalBacklogReconciler merges entities into their owning PackageDef subpackages via deap_core::sysml_cst formatting-preserving rowan splices with zero root AST pollution, and all 12 blueprint gates pass with exit code 0 (Diagnostic: E0400). */
    use case def UC_AC_05_Fail_Closed_Subpackage_Scoped_Reverse_Sync_and_Dogfood_Blueprint_Ingestion_Parity {
        subject AgileSpecForwardProjector;
        objective "Fail Closed Subpackage Scoped Reverse Sync and Dogfood Blueprint Ingestion Parity";
    }
    test case def TC_REQ_0604_Forward_Sync_Icd_Wbs {
        verify requirement REQ_0604;
    }
    test case def TC_REQ_0604_Blueprint_Conformance {
        verify requirement REQ_0604_BP_04_Deterministic_Specification_ICD_and_WBS_Forward_Compiler_Blueprint;
    }
    test case def Test_CompileSysmlForwardSyncDocs {
        verify requirement REQ_0604;
    }
    test case def Test_ResolveParentEpicByPkgName {
        verify requirement REQ_0604;
    }
    test case def Test_BindAllPackageStoriesToUse {
        verify requirement REQ_0604;
    }
    test case def Test_EmitSymmetricActionDefSignaturesAndE1 {
        verify requirement REQ_0604;
    }
    test case def Test_CompileSysmlEmitIcdDocsInterfaces {
        verify requirement REQ_0604;
    }
    test case def Test_ProjectNxNConnectionDefMatrixAndFlowDef {
        verify requirement REQ_0604;
    }
    test case def Test_CompileSysmlEmitWbsDocsManagement {
        verify requirement REQ_0604;
    }
    test case def Test_SynthesizeMilStd881eTreeAnd {
        verify requirement REQ_0604;
    }
    test case def Test_ReconcileBacklogReconcileDocsDirDocs {
        verify requirement REQ_0604;
    }
    test case def Test_ParseCommonMarkTablesWithInHtml {
        verify requirement REQ_0604;
    }
    test case def Test_ReconcileChecklistsWithParentInclusiveEpic {
        verify requirement REQ_0604;
    }
    test case def Test_CompileSysmlReverseSyncDocs {
        verify requirement REQ_0604;
    }
    test case def Test_MergeExtractedEntitiesIntoOwningSubpackages {
        verify requirement REQ_0604;
    }
    doc /* Extracted from Mermaid sequenceDiagram specification. */
    interaction def BP_04_Deterministic_Specification_ICD_and_WBS_Forward_Compiler_Blueprint_SequenceInteraction {
        lifeline CLI : Operator_Or_CI_Runner;
        lifeline Fwd : compile_sysml_Forward;
        lifeline Icd : LogicalIcdMatrixCompiler;
        lifeline Wbs : MilStd881eWbsExporter;
        lifeline Rec : reconcile_backlog_Tool;
        lifeline Rev : compile_sysml_Reverse;
        message CompileSysmlForwardSyncDocs from CLI to Fwd;
        message ResolveParentEpicByPkgName from Fwd to Fwd;
        message BindAllPackageStoriesToUse from Fwd to Fwd;
        message EmitSymmetricActionDefSignaturesAndE1 from Fwd to Fwd;
        message reply_EpicsFeaturesStoriesAndUseCases from Fwd to CLI;
        message CompileSysmlEmitIcdDocsInterfaces from CLI to Icd;
        message ProjectNxNConnectionDefMatrixAndFlowDef from Icd to Icd;
        message reply_Icd01AndIcd02Written from Icd to CLI;
        message CompileSysmlEmitWbsDocsManagement from CLI to Wbs;
        message SynthesizeMilStd881eTreeAnd from Wbs to Wbs;
        message reply_Wbs0103AndEnterpriseExports from Wbs to CLI;
        message ReconcileBacklogReconcileDocsDirDocs from CLI to Rec;
        message ParseCommonMarkTablesWithInHtml from Rec to Rec;
        message ReconcileChecklistsWithParentInclusiveEpic from Rec to Rec;
        message reply_ZeroUnpopulatedPlaceholdersOrSpoofedIDs from Rec to CLI;
        message CompileSysmlReverseSyncDocs from CLI to Rev;
        message MergeExtractedEntitiesIntoOwningSubpackages from Rev to Rev;
        message reply_BitIdenticalSysMLV2AstAnd from Rev to CLI;
    }
    doc /* Maximum deterministic execution bound for `--forward-sync` across all package subtrees. [unit: ms] */
    part def AgileSpecForwardProjector {
        attribute maxForwardLatencyMs : Integer = 25;
        doc /* Enforces symmetric `in` and `out` `ActionDef` parameter emission across Mermaid and Markdown bullets. [unit: flag] */
        attribute preserveTypedActionParams : Boolean = true;
        doc /* Receives ordered `PackageDef`, `PartDef`, `InteractionDef`, and `UseCaseDef` AST slices. */
        port in packageAstInPort : String {
            attribute protocol_family : String = "MemoryBus";
            doc /* Receives ordered `PackageDef`, `PartDef`, `InteractionDef`, and `UseCaseDef` AST slices. */
            flow in packageAstInPort_flow : String;
        }
        doc /* Emits Check 13B E1-E5 compliant Epics, Features, User Stories, and Use Cases. */
        port out specMarkdownOutPort : String {
            attribute protocol_family : String = "AtomicFsWrite";
            doc /* Emits Check 13B E1-E5 compliant Epics, Features, User Stories, and Use Cases. */
            flow out specMarkdownOutPort_flow : String;
        }
    }
    doc /* Tracks multi-line `in_html_comment` state to prevent commented example tables from spoofing frontmatter issue IDs. [unit: flag] */
    part def BidirectionalBacklogReconciler {
        attribute trackHtmlCommentState : Boolean = true;
        doc /* Computes parent-inclusive Epic count so child-only slices never trigger single-epic wildcard matching. [unit: flag] */
        attribute parentInclusiveEpics : Boolean = true;
        doc /* Reads and reconciles CommonMark metadata tables and bidirectional Epic/Feature checklists under `docs/`. */
        port inout specDocsInOutPort : String {
            attribute protocol_family : String = "AtomicFsWrite";
        }
        doc /* Merges reverse-synchronized specification entities into their owning subsystem `PackageDef` subpackages. */
        port out subpackageSyncOutPort : String {
            attribute protocol_family : String = "AtomicFsWrite";
            doc /* Merges reverse-synchronized specification entities into their owning subsystem `PackageDef` subpackages. */
            flow out subpackageSyncOutPort_flow : String;
        }
    }
    doc /* Total number of generative LLM subagents invoked during specification, ICD, and WBS forward compilation (`C-FWD-01 == 0`). [unit: count] */
    part def DeterministicSpecIcdWbsForwardCompiler {
        attribute llm_subagent_invocation_count : Integer = 0;
        doc /* Percentage of declared SysML v2 AST packages, parts, ports, flows, interactions, and use cases projected into specifications (`C-FWD-02 == 100.0`). [unit: percent] */
        attribute ast_node_coverage_percent : Real = 100;
        doc /* Count of Check 13B E1-E5 Mermaid syntax or layout violations across all compiler-synthesized diagrams (`C-FWD-03 == 0`). [unit: count] */
        attribute mermaid_e1_e5_violation_count : Integer = 0;
        doc /* Count of AST structural or subpackage placement divergences after a forward-then-reverse synchronization cycle (`C-FWD-04 == 0`). [unit: count] */
        attribute forward_reverse_drift_count : Integer = 0;
        doc /* Receives the parsed, type-checked `SysmlPackage` (`PackageDef`) AST and SHA-256 `SchemaDigest` from `deap-core`. */
        port in SysmlPackageAstInputPort : String {
            attribute protocol_family : String = "MemoryBus";
            doc /* Receives the parsed, type-checked `SysmlPackage` (`PackageDef`) AST and SHA-256 `SchemaDigest` from `deap-core`. */
            flow in SysmlPackageAstInputPort_flow : String;
        }
        doc /* Emits deterministic `SpecSuiteBundle` Markdown specifications across `docs/epics/`, `docs/features/`, `docs/user-stories/`, and `docs/use-cases/`. */
        port out MarkdownSpecSuiteOutputPort : String {
            attribute protocol_family : String = "AtomicFsWrite";
            doc /* Emits deterministic `SpecSuiteBundle` Markdown specifications across `docs/epics/`, `docs/features/`, `docs/user-stories/`, and `docs/use-cases/`. */
            flow out MarkdownSpecSuiteOutputPort_flow : String;
        }
        doc /* Emits `IcdDocumentPair` (`ICD_01_SYSTEM_INTERFACE_MATRIX.md` and `ICD_02_MASTER_SIGNAL_DICTIONARY.md`) under `docs/interfaces/`. */
        port out IcdMatrixArtifactPort : String {
            attribute protocol_family : String = "AtomicFsWrite";
            doc /* Emits `IcdDocumentPair` (`ICD_01_SYSTEM_INTERFACE_MATRIX.md` and `ICD_02_MASTER_SIGNAL_DICTIONARY.md`) under `docs/interfaces/`. */
            flow out IcdMatrixArtifactPort_flow : String;
        }
        doc /* Emits `WbsExportBundle` (`WBS_01`..`WBS_03` Markdown artifacts and RFC 4180 CSV / JSON AST exports) under `docs/management/`. */
        port out WbsEnterpriseExportPort : String {
            attribute protocol_family : String = "AtomicFsWrite";
            doc /* Emits `WbsExportBundle` (`WBS_01`..`WBS_03` Markdown artifacts and RFC 4180 CSV / JSON AST exports) under `docs/management/`. */
            flow out WbsEnterpriseExportPort_flow : String;
        }
    }
    doc /* Maximum deterministic execution bound for `--emit-icd` interface matrix and signal dictionary compilation. [unit: ms] */
    part def LogicalIcdMatrixCompiler {
        attribute maxIcdLatencyMs : Integer = 10;
        doc /* Orders all `PartDef` matrix axes and `ConnectionDef` channels via `BTreeMap` deterministic traversal. [unit: flag] */
        attribute lexicographicalSort : Boolean = true;
        doc /* Receives `PartDef`, `PortDef`, `ConnectionDef`, `FlowDef`, and `ItemDef` AST nodes. */
        port in interconnectAstInPort : String {
            attribute protocol_family : String = "MemoryBus";
            doc /* Receives `PartDef`, `PortDef`, `ConnectionDef`, `FlowDef`, and `ItemDef` AST nodes. */
            flow in interconnectAstInPort_flow : String;
        }
        doc /* Emits `ICD_01_SYSTEM_INTERFACE_MATRIX.md` and `ICD_02_MASTER_SIGNAL_DICTIONARY.md`. */
        port out icdMarkdownOutPort : String {
            attribute protocol_family : String = "AtomicFsWrite";
            doc /* Emits `ICD_01_SYSTEM_INTERFACE_MATRIX.md` and `ICD_02_MASTER_SIGNAL_DICTIONARY.md`. */
            flow out icdMarkdownOutPort_flow : String;
        }
    }
    doc /* Canonical 4-level MIL-STD-881E decomposition depth (System, Subsystem, Work Package, Deliverable Task). [unit: levels] */
    part def MilStd881eWbsExporter {
        attribute wbsHierarchyDepth : Integer = 4;
        doc /* Maximum deterministic execution bound for `--emit-wbs` Markdown and CSV/JSON export generation. [unit: ms] */
        attribute maxWbsLatencyMs : Integer = 15;
        doc /* Receives `PackageDef` AST hierarchy and synchronized `docs/` specification frontmatter metadata. */
        port in wbsAstMetadataInPort : String {
            attribute protocol_family : String = "MemoryBus";
            doc /* Receives `PackageDef` AST hierarchy and synchronized `docs/` specification frontmatter metadata. */
            flow in wbsAstMetadataInPort_flow : String;
        }
        doc /* Emits `WBS_01`..`WBS_03` Markdown registers and enterprise CSV/JSON AST exports. */
        port out wbsArtifactsOutPort : String {
            attribute protocol_family : String = "AtomicFsWrite";
            doc /* Emits `WBS_01`..`WBS_03` Markdown registers and enterprise CSV/JSON AST exports. */
            flow out wbsArtifactsOutPort_flow : String;
        }
    }
}

doc /* Subsystem package for BP_05_Unified_AST_Safety_Docstring_Synthesizer_and_Check_36_Verification_Blueprint */
package BP_05_Unified_AST_Safety_Docstring_Synthesizer_and_Check_36_Verification_Blueprint {
    port def BP_05_SafetyDocstringPortDef {
        flow in source_cst_in : String;
        flow out docstring_verdict_out : Boolean;
    }
    doc /* Execute compile-sysml --inject-safety-docstrings <DIR> and --compact-tier-bc <DIR> via ra_ap_syntax, ra_ap_text_edit, and rowan */
    action def RunCompileSysmlInjectSafetyDocstrings;
    doc /* Parse lossless Rust and Dart CSTs via deap_core::rust_cst (ra_ap_syntax + rowan) and deap_core::dart_cst (logos + rowan) */
    action def ParseSourceCstViaRustAndDartCst;
    doc /* Resolve schema context via salsa-tracked collect_all_parts across 6-clause OMG SysML v2.0 PackageDef trees without .first() fallback */
    action def ResolveSchemaContextViaCollectAll;
    doc /* Execute verify-baseline . --no-domain (Check 36 Tier-A vs Tier-B/C CST verification & Check 34 uniqueness) */
    action def RunVerifyBaselineNoDomain;
    doc /* Extract typed Rust (ra_ap_syntax ast::{HasDocComments, HasAttrs}) and Dart (deap_core::dart_cst) declaration CST nodes */
    action def TokenizeRsAndDartFilesUsing;
    doc /* Execute ingest-sysml --schema blueprint-05 --format markdown via pulldown-cmark and logos+rowan SysML v2 CST */
    action def RunIngestSysmlSchemaBlueprint05;
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0605_BP_05_Unified_AST_Safety_Docstring_Synthesizer_and_Check_36_Verification_Blueprint {
        id = "REQ-0605";
        text = "UnifiedSafetyDocstringSynthesizer (REQ-0605 / BP-05 / DEAP-BP-DET-05, UUIDv5 6ba7b810-9dad-11d1-80b4-00c04fd430c5, Complexity Class P, governed by DO-178C DAL-A and DO-330 TQL-4) SHALL enforce Contract A (full 6-clause OMG SysML v2.0 + KerML 1.0 grammar coverage with zero 16-construct caps, #dal/#fls/#rt_loop metadata def bindings) and Contract B (lossless ra_ap_syntax ast::{HasDocComments, HasAttrs}, rowan::TextRange, ra_ap_line_index::LineIndex, ra_ap_text_edit::TextEdit, ra_ap_vfs, ra_ap_paths, camino, ignore, and logos+rowan deap_core::dart_cst inside < 380-SLOC submodules, enforcing complete 7-section safety docstrings on explicit Tier-A #[doc = \"[DOC-TIER: A]\"] boundaries while compacting Tier-B/C internal items to <= 4-line [REQ-*] / [FLS: fls_*] docblocks and deleting -8,200+ SLOC of legacy line-scanner docstring code).";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_unified_rust_and_dart_lexical_state_machine_without_magic_offsets : String = "- **Given**: .rs and .dart source files containing nested / * outer / * inner * / * / block comments, hashed raw strings r#\"...\"# and r##\"...\"##, Rust string literals containing \"{\", Dart interpolated strings \"value: ${map['key']}\" containing nested quotes and comments, multi-byte UTF-8 character literals, and multiple short lifetimes fn foo<'a, 'b>(x: &'a str, y: &'b str). - **When**: deap_core::rust_cst (ra_ap_syntax + rowan), deap_core::dart_cst (logos + rowan), compile-sysml --inject-safety-docstrings, and verify-baseline Check 36 parse the files. - **Then**: All crates execute the lossless Red/Green CST parsers with exact rowan::TextRange and ra_ap_line_index::LineIndex span resolution and zero magic byte offsets.";
        attribute ac_02_strict_schema_context_resolution_and_recursive_part_traversal : String = "- **Given**: A multi-part 6-clause SysML v2.0 PackageDef containing PrimaryController (REQ-PC-001, constraint ThermalLimit), a nested PartDef NestedSubunit (REQ-NS-002) inside SecondaryMonitor.parts, and an unmapped Rust struct SecondaryMonitor with method pub fn primary_controller(&self) and free function pub fn unmapped_helper(). - **When**: resolve_schema_context is invoked during compile-sysml --inject-safety-docstrings. - **Then**: PrimaryController resolves REQ-PC-001 and ThermalLimit, NestedSubunit resolves REQ-NS-002 via collect_all_parts, and unmapped items never falsely inherit PrimaryController requirements via .first() fallback.";
        attribute ac_03_scoped_mod_tests_brace_tracking_unsafe_extern_classification_and_arrow_aware_impl_headers : String = "- **Given**: A Rust module containing #[cfg(test)] mod tests { ... } followed by a production pub unsafe fn post_test_api(), a #[cfg(test)] impl MockHelper { ... } block, an impl<F: Fn(u32) -> bool> FilterEngine<F> block, and a Dart class method preceded by a multi-line @pragma('vm:prefer-inline') annotation. - **When**: compile-sysml --inject-safety-docstrings and verify-baseline Check 36 inspect the typed ra_ap_syntax and deap_core::dart_cst syntax trees. - **Then**: #[cfg(test)] subtrees are scoped via typed ast::HasAttrs CST nodes so post_test_api is audited, Tier-A items receive 7-section docstrings, and Tier-B/C items enforce compact <= 4-line [REQ-*] / [FLS: fls_*] docblocks.";
        attribute ac_04_bit_for_bit_idempotency_and_fail_closed_check_34_per_function_normalized_body_uniqueness : String = "- **Given**: Any Rust crate or Dart source directory enriched by compile-sysml --inject-safety-docstrings <DIR> and --compact-tier-bc <DIR> via ra_ap_text_edit::TextEdit, as well as a negative fixture containing duplicate normalized Tier-A docstring bodies or missing sections. - **When**: compile-sysml --inject-safety-docstrings <DIR> is executed a second time and followed by ./target/release/verify-baseline . --no-domain. - **Then**: The second synthesis pass produces 0 modified bytes (docstring_idempotency_diff_bytes == 0), every Tier-A docblock passes Check 36 7-section completeness and Check 34 uniqueness, and any invalid or duplicate docblock fails closed with exit code 1.";
        attribute ac_05_dogfood_ingestion_and_sysml_v2_compilation_of_req_0605 : String = "- **Given**: This blueprint specification docs/designs/blueprint-05-deterministic-safety-docstring-synthesizer.md. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-05-deterministic-safety-docstring-synthesizer.md --dry-run and ./target/release/ingest-sysml --schema docs/designs/blueprint-05-deterministic-safety-docstring-synthesizer.md --format markdown are executed. - **Then**: The translator parses REQ-0605, UnifiedSafetyDocstringSynthesizer, all BOM subcomponents, interface ports, state attributes, and parametric constraints (C-DOC-01..C-DOC-04) into .sysml and .digest.json with exit code 0.";
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
        transition ScopedSchemaContextResolver_State_to_SevenSectionDocstringGenerator_State_on_ParseSourceCstViaRustAndDartCst first ScopedSchemaContextResolver_State accept ParseSourceCstViaRustAndDartCst then SevenSectionDocstringGenerator_State;
        transition SevenSectionDocstringGenerator_State_to_Terminal first SevenSectionDocstringGenerator_State then Terminal;
    }
    connection def BP_05_ContextToDocstringGeneratorConnection {
        connect ScopedSchemaContextResolver.resolved_context_out to SevenSectionDocstringGenerator.schema_context_in;
    }
    doc /* - **Given**: .rs and .dart source files containing nested / * outer / * inner * / * / block comments, hashed raw strings r#\"...\"# and r##\"...\"##, Rust string literals containing \"{\", Dart interpolated strings \"value: ${map['key']}\" containing nested quotes and comments, multi-byte UTF-8 character literals, and multiple short lifetimes fn foo<'a, 'b>(x: &'a str, y: &'b str). - **When**: deap_core::rust_cst (ra_ap_syntax + rowan), deap_core::dart_cst (logos + rowan), compile-sysml --inject-safety-docstrings, and verify-baseline Check 36 parse the files. - **Then**: All crates execute the lossless Red/Green CST parsers with exact rowan::TextRange and ra_ap_line_index::LineIndex span resolution and zero magic byte offsets. */
    use case def UC_AC_01_Unified_Rust_and_Dart_Lexical_State_Machine_Without_Magic_Offsets {
        subject Check36AstCompletenessVerifier;
        objective "Unified Rust and Dart Lexical State Machine Without Magic Offsets";
    }
    doc /* - **Given**: A multi-part 6-clause SysML v2.0 PackageDef containing PrimaryController (REQ-PC-001, constraint ThermalLimit), a nested PartDef NestedSubunit (REQ-NS-002) inside SecondaryMonitor.parts, and an unmapped Rust struct SecondaryMonitor with method pub fn primary_controller(&self) and free function pub fn unmapped_helper(). - **When**: resolve_schema_context is invoked during compile-sysml --inject-safety-docstrings. - **Then**: PrimaryController resolves REQ-PC-001 and ThermalLimit, NestedSubunit resolves REQ-NS-002 via collect_all_parts, and unmapped items never falsely inherit PrimaryController requirements via .first() fallback. */
    use case def UC_AC_02_Strict_Schema_Context_Resolution_and_Recursive_Part_Traversal {
        subject Check36AstCompletenessVerifier;
        objective "Strict Schema Context Resolution and Recursive Part Traversal";
    }
    doc /* - **Given**: A Rust module containing #[cfg(test)] mod tests { ... } followed by a production pub unsafe fn post_test_api(), a #[cfg(test)] impl MockHelper { ... } block, an impl<F: Fn(u32) -> bool> FilterEngine<F> block, and a Dart class method preceded by a multi-line @pragma('vm:prefer-inline') annotation. - **When**: compile-sysml --inject-safety-docstrings and verify-baseline Check 36 inspect the typed ra_ap_syntax and deap_core::dart_cst syntax trees. - **Then**: #[cfg(test)] subtrees are scoped via typed ast::HasAttrs CST nodes so post_test_api is audited, Tier-A items receive 7-section docstrings, and Tier-B/C items enforce compact <= 4-line [REQ-*] / [FLS: fls_*] docblocks. */
    use case def UC_AC_03_Scoped_mod_tests_Brace_Tracking_unsafe_extern_Classification_and_Arrow_Aware_impl_Headers {
        subject Check36AstCompletenessVerifier;
        objective "Scoped mod tests Brace Tracking unsafe extern Classification and Arrow Aware impl Headers";
    }
    doc /* - **Given**: Any Rust crate or Dart source directory enriched by compile-sysml --inject-safety-docstrings <DIR> and --compact-tier-bc <DIR> via ra_ap_text_edit::TextEdit, as well as a negative fixture containing duplicate normalized Tier-A docstring bodies or missing sections. - **When**: compile-sysml --inject-safety-docstrings <DIR> is executed a second time and followed by ./target/release/verify-baseline . --no-domain. - **Then**: The second synthesis pass produces 0 modified bytes (docstring_idempotency_diff_bytes == 0), every Tier-A docblock passes Check 36 7-section completeness and Check 34 uniqueness, and any invalid or duplicate docblock fails closed with exit code 1. */
    use case def UC_AC_04_Bit_for_Bit_Idempotency_and_Fail_Closed_Check_34_Per_Function_Normalized_Body_Uniqueness {
        subject Check36AstCompletenessVerifier;
        objective "Bit for Bit Idempotency and Fail Closed Check 34 Per Function Normalized Body Uniqueness";
    }
    doc /* - **Given**: This blueprint specification docs/designs/blueprint-05-deterministic-safety-docstring-synthesizer.md. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-05-deterministic-safety-docstring-synthesizer.md --dry-run and ./target/release/ingest-sysml --schema docs/designs/blueprint-05-deterministic-safety-docstring-synthesizer.md --format markdown are executed. - **Then**: The translator parses REQ-0605, UnifiedSafetyDocstringSynthesizer, all BOM subcomponents, interface ports, state attributes, and parametric constraints (C-DOC-01..C-DOC-04) into .sysml and .digest.json with exit code 0. */
    use case def UC_AC_05_Dogfood_Ingestion_and_SysML_v2_Compilation_of_REQ_0605 {
        subject Check36AstCompletenessVerifier;
        objective "Dogfood Ingestion and SysML v2 Compilation of REQ 0605";
    }
    test case def TC_REQ_0605_Safety_Docstring_Synthesizer {
        verify requirement REQ_0605;
    }
    test case def TC_REQ_0605_Blueprint_Conformance {
        verify requirement REQ_0605_BP_05_Unified_AST_Safety_Docstring_Synthesizer_and_Check_36_Verification_Blueprint;
    }
    test case def Test_RunCompileSysmlInjectSafetyDocstrings {
        verify requirement REQ_0605;
    }
    test case def Test_ParseSourceCstViaRustAndDartCst {
        verify requirement REQ_0605;
    }
    test case def Test_ScanSourceLinesViaRustDartLexStateAnd {
        verify requirement REQ_0605;
    }
    test case def Test_ResolveSchemaContextViaCollectAll {
        verify requirement REQ_0605;
    }
    test case def Test_RunVerifyBaselineNoDomain {
        verify requirement REQ_0605;
    }
    test case def Test_TokenizeRsAndDartFilesUsing {
        verify requirement REQ_0605;
    }
    test case def Test_RunIngestSysmlSchemaBlueprint05 {
        verify requirement REQ_0605;
    }
    doc /* Extracted from Mermaid sequenceDiagram specification. */
    interaction def BP_05_Unified_AST_Safety_Docstring_Synthesizer_and_Check_36_Verification_Blueprint_SequenceInteraction {
        lifeline CLI : Compiler_CLI_CI_Pipeline;
        lifeline Core : deap_core_rust_and_dart_cst;
        lifeline Synth : compile_sysml_docstring;
        lifeline Gate : verify_baseline_docstring_tier_verifier;
        lifeline Ingest : ingest_sysml;
        message RunCompileSysmlInjectSafetyDocstrings from CLI to Synth;
        message ParseSourceCstViaRustAndDartCst from Synth to Core;
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
    doc /* Baseline verification gate in verify-baseline auditing Tier-A 7-section docstring completeness, Tier-B/C <= 4-line [REQ-*] / [FLS: fls_*] compaction, and Check 34 normalized body uniqueness using deap_core::rust_cst (ra_ap_syntax + rowan) and deap_core::dart_cst (logos + rowan) */
    part def Check36AstCompletenessVerifier {
        attribute quantity : Integer = 1;
        attribute type : String = "Component";
    }
    doc /* Strict 6-clause OMG SysML v2.0 + KerML 1.0 requirement and constraint resolver in compile-sysml traversing recursive part.parts trees with owner_name isolation and zero .first() fallback */
    part def ScopedSchemaContextResolver {
        attribute quantity : Integer = 1;
        attribute type : String = "Component";
        port out resolved_context_out : String;
    }
    doc /* Deterministic CST synthesizer using ra_ap_syntax ast::HasDocComments and ra_ap_text_edit::TextEdit to interpolate AST parameters, return types, struct fields, invariants, and [REQ-*] / [FLS: fls_*] bindings into Tier-A 7-section and Tier-B/C <= 4-line docblocks */
    part def SevenSectionDocstringGenerator {
        attribute quantity : Integer = 1;
        attribute type : String = "Component";
        port in schema_context_in : String;
    }
    doc /* Lossless Red/Green CST parser facade combining deap_core::rust_cst (ra_ap_syntax + rowan, ra_ap_vfs, ra_ap_paths, ra_ap_line_index, camino, ignore) and deap_core::dart_cst (logos + rowan) with zero hand-rolled line scanners */
    part def UnifiedRustDartCstParser {
        attribute quantity : Integer = 1;
        attribute type : String = "Component";
    }
    doc /* Top-level unified safety docstring synthesizer and Check 36 CST verification orchestrator enforcing < 380-SLOC submodules and -8,200+ SLOC legacy docstring scanner deletion */
    part def UnifiedSafetyDocstringSynthesizer {
    }
}

doc /* Subsystem package for BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint */
package BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint {
    port def BP_06_DefectDossierPortDef {
        flow in audit_finding_in : String;
        flow out verified_dossier_out : String;
    }
    doc /* Execute verify-baseline audit-code --path crates/ --publish via ra_ap_syntax and Stage-2 Dylint MIR */
    action def VerifyBaselineAuditCodePathCrates;
    doc /* Parse lossless Rust and Dart CSTs via deap_core::rust_cst (ra_ap_syntax + rowan) and deap_core::dart_cst (logos + rowan) */
    action def ParseRustCstAndParseDartCst;
    doc /* Evaluate 5 correctness pillars via ra_ap_syntax AST, Stage-2 Dylint MIR (rustc_middle::mir::Body, TyCtxt), and deap_core::dart_cst */
    action def EvaluateSourceFilePillarsAcross5;
    doc /* Synthesize 7-section defect dossier Markdown via pulldown-cmark-to-cmark (synthesize_defect_dossier_markdown) */
    action def synthesize_defect_dossier_markdown;
    doc /* Execute run_check_defect(DefectDossierArgs) 12-check validation and closed-loop publisher gate */
    action def run_check_defect;
    doc /* Execute 12-check validate_defect_dossier gate over pulldown-cmark CommonMark AST and Mermaid E1-E5 blocks */
    action def Execute12CheckValidateDefectDossier;
    doc /* Execute gh issue create or gh issue edit with --body-file and serde_json receipt parsing */
    action def GhIssueCreateOrGhIssue;
    doc /* Execute gh issue view --json title,body,url and verify LF-normalized byte parity via serde_json */
    action def GhIssueViewJsonTitleBody;
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0606_BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint {
        id = "REQ-0606";
        text = "The DeterministicStaticDefectAuditor subsystem (REQ-0606 / BP-06 / DEAP-BP-DET-06, deterministic UUIDv5 6ba7b810-9dad-11d1-80b4-00c04fd430c6, complexity class Class P, governed by DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C) SHALL enforce Contract A (full 6-clause OMG SysML v2.0 + KerML 1.0 grammar coverage with zero 16-construct caps) and Contract B (lossless ra_ap_syntax AST evaluation across 5 correctness pillars, Stage-2 isolated ./lints/deap-dylint-mir/ reading rustc_middle::mir::Body and TyCtxt, logos+rowan deap_core::dart_cst, pulldown-cmark + pulldown-cmark-to-cmark 7-section dossier synthesis, ra_ap_line_index, ra_ap_vfs, camino, ignore, and serde_json inside < 380-SLOC submodules with zero LLM subagents and zero hand-rolled token scanners).";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_static_five_pillar_ast_defect_detection : String = "- **Given**: Rust (.rs) and Dart (.dart) source files parsed via deap_core::rust_cst (ra_ap_syntax + rowan), Stage-2 Dylint MIR (rustc_middle::mir::Body, TyCtxt), and deap_core::dart_cst (logos + rowan). - **When**: verify-baseline audit-code evaluates all 5 adversarial correctness pillars. - **Then**: Every violation is deterministically detected with exact 1-based line spans via ra_ap_line_index and zero LLM subagents (Diagnostic: E0601).";
        attribute ac_02_deterministic_seven_section_dossier_and_mermaid_synthesis : String = "- **Given**: A line-exact AuditFinding emitted by FivePillarStaticAstAnalyzer. - **When**: DeterministicDossierSynthesizer and MermaidUmlDiagramGenerator execute via pulldown-cmark-to-cmark. - **Then**: A compliant 7-section .pipeline/defects/DEFECT-*.md dossier with a 5-Whys causal chain and Check 13B E1-E5 Mermaid diagram is synthesized deterministically (Diagnostic: E0602).";
        attribute ac_03_idempotent_local_and_remote_ssot_deduplication : String = "- **Given**: Existing .pipeline/defects/ *.md dossiers parsed via pulldown-cmark and live GitHub issues queried via serde_json. - **When**: verify-baseline audit-code evaluates candidate findings. - **Then**: Duplicate findings are idempotently skipped without creating redundant local files or duplicate GitHub issues (Diagnostic: E0603).";
        attribute ac_04_closed_loop_github_ssot_publishing_and_byte_parity_verification : String = "- **Given**: A validated 7-section defect dossier in .pipeline/defects/. - **When**: ClosedLoopDefectPublisherGate publishes the dossier via gh issue create/edit and queries gh issue view --json title,body,url. - **Then**: LF-normalized byte discrepancy between local dossier and remote GitHub issue body is 0 bytes (closed_loop_mismatch_bytes == 0, Diagnostic: E0604).";
        attribute ac_05_fail_closed_rejection_on_malformed_dossier_schema_ephemeral_paths_or_payload_divergence : String = "- **Given**: A candidate defect dossier missing any of the 7 mandatory sections, residing in an ephemeral /tmp path, or exhibiting remote payload drift. - **When**: validate_defect_dossier executes its 12 mechanical gates via pulldown-cmark. - **Then**: Validation fails closed with exit code 1 and emits a structured E0605 diagnostic (Diagnostic: E0605).";
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
        state IngestingRustAndDartCst;
        state EvaluatingFivePillars;
        state SynthesizingDossier;
        state Verified;
        state Validating12Gates;
        state Rejected;
        state DeduplicatingSsot;
        state PublishingClosedLoop;
        transition Initial_to_Idle first Initial then Idle;
        transition Idle_to_IngestingRustAndDartCst_on_verify_baseline_audit_code_path first Idle accept verify_baseline_audit_code_path then IngestingRustAndDartCst;
        transition IngestingRustAndDartCst_to_EvaluatingFivePillars_on_RustCstItem_slice_parsed first IngestingRustAndDartCst accept RustCstItem_slice_parsed then EvaluatingFivePillars;
        transition EvaluatingFivePillars_to_SynthesizingDossier_on_AuditFinding_emitted first EvaluatingFivePillars accept AuditFinding_emitted then SynthesizingDossier;
        transition EvaluatingFivePillars_to_Verified_on_zero_findings_detected first EvaluatingFivePillars accept zero_findings_detected then Verified;
        transition SynthesizingDossier_to_Validating12Gates_on_synthesize_defect_dossier_markdown first SynthesizingDossier accept synthesize_defect_dossier_markdown then Validating12Gates;
        transition Validating12Gates_to_Rejected_on_schema_or_Mermaid_check_violation first Validating12Gates accept schema_or_Mermaid_check_violation then Rejected;
        transition Validating12Gates_to_DeduplicatingSsot_on_all_12_checks_passed first Validating12Gates accept all_12_checks_passed then DeduplicatingSsot;
        transition DeduplicatingSsot_to_Verified_on_existing_finding_skipped first DeduplicatingSsot accept existing_finding_skipped then Verified;
        transition DeduplicatingSsot_to_PublishingClosedLoop_on_new_finding_persisted_to_pipeline_defects first DeduplicatingSsot accept new_finding_persisted_to_pipeline_defects then PublishingClosedLoop;
        transition PublishingClosedLoop_to_Verified_on_closed_loop_mismatch_bytes_0 first PublishingClosedLoop accept closed_loop_mismatch_bytes_0 then Verified;
        transition PublishingClosedLoop_to_Rejected_on_remote_payload_mismatch first PublishingClosedLoop accept remote_payload_mismatch then Rejected;
        transition Rejected_to_Terminal first Rejected then Terminal;
        transition Verified_to_Terminal first Verified then Terminal;
    }
    connection def BP_06_DossierToPublisherGateConnection {
        connect DeterministicDossierSynthesizer.dossierMarkdownOutPort to ClosedLoopDefectPublisherGate.dossierCandidateInPort;
    }
    doc /* - **Given**: Rust (.rs) and Dart (.dart) source files parsed via deap_core::rust_cst (ra_ap_syntax + rowan), Stage-2 Dylint MIR (rustc_middle::mir::Body, TyCtxt), and deap_core::dart_cst (logos + rowan). - **When**: verify-baseline audit-code evaluates all 5 adversarial correctness pillars. - **Then**: Every violation is deterministically detected with exact 1-based line spans via ra_ap_line_index and zero LLM subagents (Diagnostic: E0601). */
    use case def UC_AC_01_Static_Five_Pillar_AST_Defect_Detection {
        subject ClosedLoopDefectPublisherGate;
        objective "Static Five Pillar AST Defect Detection";
    }
    doc /* - **Given**: A line-exact AuditFinding emitted by FivePillarStaticAstAnalyzer. - **When**: DeterministicDossierSynthesizer and MermaidUmlDiagramGenerator execute via pulldown-cmark-to-cmark. - **Then**: A compliant 7-section .pipeline/defects/DEFECT-*.md dossier with a 5-Whys causal chain and Check 13B E1-E5 Mermaid diagram is synthesized deterministically (Diagnostic: E0602). */
    use case def UC_AC_02_Deterministic_Seven_Section_Dossier_and_Mermaid_Synthesis {
        subject ClosedLoopDefectPublisherGate;
        objective "Deterministic Seven Section Dossier and Mermaid Synthesis";
    }
    doc /* - **Given**: Existing .pipeline/defects/ *.md dossiers parsed via pulldown-cmark and live GitHub issues queried via serde_json. - **When**: verify-baseline audit-code evaluates candidate findings. - **Then**: Duplicate findings are idempotently skipped without creating redundant local files or duplicate GitHub issues (Diagnostic: E0603). */
    use case def UC_AC_03_Idempotent_Local_and_Remote_SSOT_Deduplication {
        subject ClosedLoopDefectPublisherGate;
        objective "Idempotent Local and Remote SSOT Deduplication";
    }
    doc /* - **Given**: A validated 7-section defect dossier in .pipeline/defects/. - **When**: ClosedLoopDefectPublisherGate publishes the dossier via gh issue create/edit and queries gh issue view --json title,body,url. - **Then**: LF-normalized byte discrepancy between local dossier and remote GitHub issue body is 0 bytes (closed_loop_mismatch_bytes == 0, Diagnostic: E0604). */
    use case def UC_AC_04_Closed_Loop_GitHub_SSOT_Publishing_and_Byte_Parity_Verification {
        subject ClosedLoopDefectPublisherGate;
        objective "Closed Loop GitHub SSOT Publishing and Byte Parity Verification";
    }
    doc /* - **Given**: A candidate defect dossier missing any of the 7 mandatory sections, residing in an ephemeral /tmp path, or exhibiting remote payload drift. - **When**: validate_defect_dossier executes its 12 mechanical gates via pulldown-cmark. - **Then**: Validation fails closed with exit code 1 and emits a structured E0605 diagnostic (Diagnostic: E0605). */
    use case def UC_AC_05_Fail_Closed_Rejection_on_Malformed_Dossier_Schema_Ephemeral_Paths_or_Payload_Divergence {
        subject ClosedLoopDefectPublisherGate;
        objective "Fail Closed Rejection on Malformed Dossier Schema Ephemeral Paths or Payload Divergence";
    }
    doc /* - **Given**: This blueprint specification at docs/designs/blueprint-06-deterministic-static-defect-dossier-auditor.md. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-06-deterministic-static-defect-dossier-auditor.md --dry-run is executed. - **Then**: All 12 blueprint verification gates pass, DeterministicStaticDefectAuditor and its component metamodel are lowered into docs/designs/blueprint-06-deterministic-static-defect-dossier-auditor.sysml and .digest.json with bijective roundtrip parity, and the command exits with code 0. */
    use case def UC_AC_06_Dogfood_SysML_v2_Markdown_Ingestion_and_Blueprint_Gate_Conformance {
        subject ClosedLoopDefectPublisherGate;
        objective "Dogfood SysML v2 Markdown Ingestion and Blueprint Gate Conformance";
    }
    test case def TC_REQ_0606_Static_Defect_Auditor {
        verify requirement REQ_0606;
    }
    test case def TC_REQ_0606_Blueprint_Conformance {
        verify requirement REQ_0606_BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint;
    }
    test case def Test_VerifyBaselineAuditCodePathCrates {
        verify requirement REQ_0606;
    }
    test case def Test_ParseRustCstAndParseDartCst {
        verify requirement REQ_0606;
    }
    test case def Test_EvaluateSourceFilePillarsAcross5 {
        verify requirement REQ_0606;
    }
    test case def Test_synthesize_defect_dossier_markdown {
        verify requirement REQ_0606;
    }
    test case def Test_run_check_defect {
        verify requirement REQ_0606;
    }
    test case def Test_Execute12CheckValidateDefectDossier {
        verify requirement REQ_0606;
    }
    test case def Test_GhIssueCreateOrGhIssue {
        verify requirement REQ_0606;
    }
    test case def Test_GhIssueViewJsonTitleBody {
        verify requirement REQ_0606;
    }
    doc /* Extracted from Mermaid sequenceDiagram specification. */
    interaction def BP_06_Deterministic_Static_5_Pillar_Code_Auditor_and_Defect_Dossier_Synthesizer_Blueprint_SequenceInteraction {
        lifeline User : Operator_Or_CI_Pipeline;
        lifeline CLI : verify_baseline_audit_code;
        lifeline AST : deap_core_rust_and_dart_cst;
        lifeline Engine : FivePillarStaticAstAnalyzer;
        lifeline Synth : DeterministicDossierSynthesizer;
        lifeline Gate : defect_dossier_run_check_defect;
        lifeline GH : GitHub_SSOT_Tracker;
        message VerifyBaselineAuditCodePathCrates from User to CLI;
        message ParseRustCstAndParseDartCst from CLI to AST;
        message reply_DeterministicVecOfRustCstItemNodes from AST to CLI;
        message EvaluateSourceFilePillarsAcross5 from CLI to Engine;
        message reply_LineExactAuditFindingRecords from Engine to CLI;
        message synthesize_defect_dossier_markdown from CLI to Synth;
        message reply_Compliant7SectionDefectSlugMd from Synth to CLI;
        message run_check_defect from CLI to Gate;
        message Execute12CheckValidateDefectDossier from Gate to Gate;
        message GhIssueCreateOrGhIssue from Gate to GH;
        message reply_CreatedOrUpdatedGitHubIssueUrl from GH to Gate;
        message GhIssueViewJsonTitleBody from Gate to GH;
        message reply_LiveRemotePayloadForByteFor from GH to Gate;
        message reply_ClosedLoopVerifiedReceipt from Gate to CLI;
        message reply_ExitCode0WithVerifiedIssue from CLI to User;
    }
    doc /* Enforces all 12 `validate_defect_dossier` checks prior to persistence or publication. [unit: checks] */
    part def ClosedLoopDefectPublisherGate {
        attribute schemaGateCount : Integer = 12;
        doc /* Requires zero byte discrepancy between local `.pipeline/defects/` file and remote GitHub issue body. [unit: flag] */
        attribute requireByteParity : Boolean = true;
        doc /* Receives synthesized markdown payload for in-process 12-check verification. */
        port in dossierCandidateInPort : String {
            attribute protocol_family : String = "CommonMarkText";
            doc /* Receives synthesized markdown payload for in-process 12-check verification. */
            flow in dossierCandidateInPort_flow : String;
        }
        doc /* Emits verified GitHub SSOT issue URL after `gh issue view` closed-loop payload verification. */
        port out verifiedReceiptOutPort : String {
            attribute protocol_family : String = "GhCliJson";
            doc /* Emits verified GitHub SSOT issue URL after `gh issue view` closed-loop payload verification. */
            flow out verifiedReceiptOutPort_flow : String;
        }
    }
    doc /* Enforces all 7 canonical `DossierSection` headers plus the `## Audit Source` footer. [unit: sections] */
    part def DeterministicDossierSynthesizer {
        attribute mandatorySectionCount : Integer = 7;
        doc /* Enforces 5 numbered causal `Why ...? Because ...` root-cause steps in Section 2. [unit: steps] */
        attribute whysChainDepth : Integer = 5;
        doc /* Receives a deterministic `AuditFinding` struct for markdown serialization. */
        port in findingInPort : String {
            attribute protocol_family : String = "AuditFindingRef";
            doc /* Receives a deterministic `AuditFinding` struct for markdown serialization. */
            flow in findingInPort_flow : String;
        }
        doc /* Emits a 7-section defect dossier payload starting with `## 1. Context and References`. */
        port out dossierMarkdownOutPort : String {
            attribute protocol_family : String = "CommonMarkText";
            doc /* Emits a 7-section defect dossier payload starting with `## 1. Context and References`. */
            flow out dossierMarkdownOutPort_flow : String;
        }
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
        doc /* Receives parsed Rust (.rs) via deap_core::rust_cst (ra_ap_syntax + rowan), Stage-2 Dylint MIR (rustc_middle::mir::Body, TyCtxt), and Dart (.dart) via deap_core::dart_cst (logos + rowan). */
        port in TargetRustCstInputPort : String {
            attribute protocol_family : String = "RustSliceAst";
            doc /* Receives parsed Rust (.rs) via deap_core::rust_cst (ra_ap_syntax + rowan), Stage-2 Dylint MIR (rustc_middle::mir::Body, TyCtxt), and Dart (.dart) via deap_core::dart_cst (logos + rowan). */
            flow in TargetRustCstInputPort_flow : String;
        }
        doc /* Receives existing `.pipeline/defects/ *.md` records and live GitHub issue metadata for idempotent finding deduplication. */
        port in GitHubIssueSsotQueryPort : String {
            attribute protocol_family : String = "GhCliJson";
            doc /* Receives existing `.pipeline/defects/ *.md` records and live GitHub issue metadata for idempotent finding deduplication. */
            flow in GitHubIssueSsotQueryPort_flow : String;
        }
        doc /* Persists validated 7-section `.pipeline/defects/DEFECT-.md` dossiers to the repository defect registry. */
        port out DefectDossierArtifactPort : String {
            attribute protocol_family : String = "AtomicFsWrite";
            doc /* Persists validated 7-section `.pipeline/defects/DEFECT-.md` dossiers to the repository defect registry. */
            flow out DefectDossierArtifactPort_flow : String;
        }
        doc /* Publishes or updates GitHub SSOT issues via `gh issue create` or `gh issue edit` and verifies byte parity via `gh issue view`. */
        port out ClosedLoopTrackerPublishPort : String {
            attribute protocol_family : String = "GhCliSubprocess";
            doc /* Publishes or updates GitHub SSOT issues via `gh issue create` or `gh issue edit` and verifies byte parity via `gh issue view`. */
            flow out ClosedLoopTrackerPublishPort_flow : String;
        }
    }
    doc /* Count of active static AST and MIR evaluators (`E0601` through `E0605`) over `ra_ap_syntax`, Stage-2 `rustc_middle::mir::Body`, and `deap_core::dart_cst`. [unit: pillars] */
    part def FivePillarStaticAstAnalyzer {
        attribute activePillarCount : Integer = 5;
        doc /* Enforces lossless `ra_ap_syntax` + `rowan` CST, Stage-2 Dylint MIR, and `logos`+`rowan` Dart CST traversal without regular expressions or hand-rolled scanners. [unit: flag] */
        attribute zeroRegexMode : Boolean = true;
        doc /* Accepts normalized `camino::Utf8Path` file paths, `ra_ap_line_index` tables, and lossless `rust_cst` / `dart_cst` item trees. */
        port in astStreamInPort : String {
            attribute protocol_family : String = "RustSliceAst";
            doc /* Accepts normalized `camino::Utf8Path` file paths, `ra_ap_line_index` tables, and lossless `rust_cst` / `dart_cst` item trees. */
            flow in astStreamInPort_flow : String;
        }
        doc /* Emits line-exact `AuditFinding` records with diagnostic bindings `E0601` through `E0605`. */
        port out findingStreamOutPort : String {
            attribute protocol_family : String = "AuditFindingVec";
            doc /* Emits line-exact `AuditFinding` records with diagnostic bindings `E0601` through `E0605`. */
            flow out findingStreamOutPort_flow : String;
        }
    }
    doc /* Guarantees zero semicolons and quoted special tokens inside Section 4 Mermaid blocks. [unit: flag] */
    part def MermaidUmlDiagramGenerator {
        attribute forbidSemicolons : Boolean = true;
        doc /* Enforces Rules E1 through E5 across synthesized `sequenceDiagram` and `classDiagram` blocks. [unit: flag] */
        attribute check13bCompliant : Boolean = true;
        doc /* Receives caller/callee symbols, line spans, and violated invariant metadata. */
        port in umlSpecInPort : String {
            attribute protocol_family : String = "AuditFindingRef";
            doc /* Receives caller/callee symbols, line spans, and violated invariant metadata. */
            flow in umlSpecInPort_flow : String;
        }
        doc /* Emits a fenced Mermaid diagram or severity-scoped `N/A --` declaration for Section 4. */
        port out mermaidBlockOutPort : String {
            attribute protocol_family : String = "MermaidText";
            doc /* Emits a fenced Mermaid diagram or severity-scoped `N/A --` declaration for Section 4. */
            flow out mermaidBlockOutPort_flow : String;
        }
    }
}

doc /* Subsystem package for BP_07_Self_Hosting_4_Level_Modular_Multi_File_Code_Generator_Blueprint */
package BP_07_Self_Hosting_4_Level_Modular_Multi_File_Code_Generator_Blueprint {
    attribute Blueprint_ID : String = "BP-07";
    attribute Requirement_ID : String = "REQ-0607";
    attribute Title : String = "Self-Hosting 4-Level Modular Multi-File Code Generator Blueprint";
    attribute Status : String = "Approved";
    attribute Target_Crates : String = "crates/compile-sysml, crates/deap-core";
    attribute Tracked_Issues : String = "#742, #733, #739, #740, #741";
    attribute Generation_Mode : String = "subagent";
    attribute Specification_Source : String = ".pipeline/upstream/schema/modular_codegen.sysml:1-120";
    doc /* Removes legacy string-concatenation / `writeln!` Rust code emission templates. */
    attribute crates_compile_sysml_src_driver_rs : String = "**Delete `-450+` SLOC**";
    doc /* Decomposes monolithic string-template Flutter/Dart and bridge emitters into bounded submodules. */
    attribute crates_compile_sysml_src_flutter_codegen_rs : String = "**Delete `-650+` SLOC**";
    doc /* `proc-macro2` + `quote::quote!` typed AST token synthesis for models, ports, and state machines. */
    attribute compile_sysml_codegen_ast_token_builder : String = "**New `< 380` exec SLOC**";
    doc /* `syn::parse2::` grammar validation and `prettyplease::unparse` canonical LF formatting. */
    attribute compile_sysml_codegen_ast_pretty_emitter : String = "**New `< 380` exec SLOC**";
    doc /* `ra_ap_syntax` lossless parse-clean AST checks and Stage-1 == Stage-2 SHA-256 fixed-point verification. */
    attribute compile_sysml_codegen_ast_fixed_point_gate : String = "**New `< 380` exec SLOC**";
    doc /* Typed `ra_ap_syntax` AST item extraction and modular code-movement partitioner. */
    attribute deap_core_rust_cst_extract_module : String = "**New `< 380` exec SLOC**";
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
    port def BP_07_ModularCodegenPortDef {
        flow in module_graph_in : String;
        flow out tree_manifest_out : String;
    }
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
        text = "SelfHostingModularCodegenCompiler (REQ-0607 / BP-07 / DEAP-BP-DET-07, UUIDv5 6ba7b810-9dad-11d1-80b4-00c04fd430c7, Class P linear Kahn DAG and AST traversal complexity, governed by DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C) SHALL enforce Contract A (full 6-clause OMG SysML v2.0 + KerML 1.0 grammar lowering with zero 16-construct caps) and Contract B (typed syn + proc-macro2 + quote + prettyplease Rust AST synthesis, ra_ap_syntax lossless parse-clean verification, Stage-1 == Stage-2 SHA-256 metacircular fixed-point parity, and < 380-SLOC submodules with zero string-concatenation codegen).";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_fail_closed_rejection_on_directed_dependency_cycles_and_upward_layer_imports : String = "- **Given**: A SysML v2 PackageDef fixture containing either a directed dependency cycle across packages/classifiers or a @Layer(Domain) classifier referencing a @Layer(Presentation) or @Layer(Application) classifier. - **When**: compile-sysml --emit-modular-rust <OUT_DIR> --schema <FIXTURE> is executed. - **Then**: TopologicalLayerDagVerifier aborts transactionally with exit code 1, emits structured diagnostic E0702 (cycle) or E0703 (illegal upward layer dependency), and leaves <OUT_DIR> with zero generated files written.";
        attribute ac_02_four_level_deterministic_lowering_with_typed_syn_file_ast_and_prettyplease_formatting : String = "- **Given**: A valid multi-package SysML v2 model containing nested PackageDef hierarchies, multi-facet PartDef components, a 40-step ActionDef sequence, a 30-transition StateDef state machine, and a 25-clause ConstraintDef boolean Expr tree. - **When**: compile-sysml --emit-modular-rust <OUT_DIR> --schema <FIXTURE> compiles and lowers the model across Levels 1 through 4 via proc-macro2 + quote::quote! + syn::parse2::<syn::File> and prettyplease::unparse. - **Then**: Compilation succeeds with exit code 0, emitting hierarchical mod.rs barrels, facet submodules (model, invariants, ports, fsm,.";
        attribute ac_03_metacircular_self_hosting_stage_1_to_stage_2_sha_256_fixed_point_identity : String = "- **Given**: The authoritative self-hosting compiler schema .pipeline/upstream/schema/modular_codegen.sysml and the committed Stage-1 tree under crates/compile-sysml/src/modular_codegen/generated/ verified parse-clean by ra_ap_syntax and wired into compile-sysml. - **When**: The Stage-1-linked compile-sysml binary re-compiles .pipeline/upstream/schema/modular_codegen.sysml into an isolated Stage-2 temporary directory and compares GeneratedTreeManifest SHA-256 digests. - **Then**: SHA-256(Stage 1) == SHA-256(Stage 2) holds across 100% of relative paths and file contents (**C-MOD-04**.";
        attribute ac_04_modular_multi_file_flutter_contract_emission_and_monolith_barrel_decomposition : String = "- **Given**: A multi-package SysML v2 schema compiled via compile-sysml --emit-flutter-contracts <APP_DIR> --schema <FIXTURE> and the Rust workspace source file crates/compile-sysml/src/flutter_codegen.rs. - **When**: Modular Flutter contract generation executes and workspace modularity tests inspect both the emitted Dart tree and crates/compile-sysml/src/. - **Then**: Per-package and per-classifier .g.dart units (models/generated/<pkg>/<part>_view.g.dart, data/generated/ffi/ *.g.dart, data/generated/dto/<pkg>/<dto>_dto.g.dart, data/seeds/generated/ *.g.dart, test/generated/support/ *.g.dart).";
        attribute ac_05_zero_string_concatenation_codegen_do_178c_seven_section_docstring_and_clippy_forbid_compliance : String = "- **Given**: All hand-authored modules under crates/compile-sysml/src/modular_codegen/ ** and all Stage-1 generated Rust modules under crates/compile-sysml/src/modular_codegen/generated/ **. - **When**: cargo clippy --workspace --all-targets -- -D warnings and ./target/release/verify-baseline audit-code --path crates/compile-sysml/src/modular_codegen --dry-run are executed. - **Then**: Both verification gates exit 0 with zero Clippy forbid violations, zero Check 34 violation tokens, zero Check 19 hardcoded domain concepts, zero legacy string-concatenation / format! code emitters, and 100%.";
        attribute ac_06_dogfood_sysml_v2_blueprint_ingestion_and_twelve_gate_closed_loop_conformance : String = "- **Given**: This engineering blueprint at docs/designs/blueprint-07-self-hosting-modular-codegen-compiler.md. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-07-self-hosting-modular-codegen-compiler.md is executed. - **Then**: All 12 blueprint verification gates pass with exit code 0, deterministically synthesizing docs/designs/blueprint-07-self-hosting-modular-codegen-compiler.sysml and .digest.json via roundtrip SysML v2 compilation and verifying closed-loop GitHub SSOT parity.";
        doc /* \begin{aligned} \text{Acyclic}(G) &\iff \exists \pi : V \to \{1, \dots, \lvert V \rvert\}, \quad \forall (u, v) \in E, \quad \pi(u) < \pi(v) \\ \text{LayerMonotonic}(G) &\iff \forall (u, v) \in E, \quad \text{tier}(u) \ge \text{tier}(v) \end{aligned} */
        constraint def Invariant_REQ_0607_SC_02;
        doc /* \begin{aligned} \forall u \in \mathcal{U}, \quad \text{LOC}(u) &\le L_{\max} = 400 \land \text{Items}(u) \le K_{\max} = 12 \\ \forall f \in \text{Funcs}(u), \quad v(G_f) &= \lvert E_f \rvert - \lvert V_f \rvert + 2 \le C_{\max} = 8 \end{aligned} */
        constraint def Invariant_REQ_0607_SC_01;
        doc /* \begin{aligned} \forall u \in \mathcal{U}, \quad \text{syn::parse2}(\text{tokens}(u)) &\in \mathcal{L}(\text{syn::File}) \\ \text{Content}(u) &= \text{normalize\_lf}(\text{prettyplease::unparse}(\text{syn::parse2}(\text{tokens}(u)))) \end{aligned} */
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
        verify by AC_02_Four_Level_Deterministic_Lowering_with_Typed_syn_File_AST_and_prettyplease_Formatting;
        verify by AC_03_Metacircular_Self_Hosting_Stage_1_to_Stage_2_SHA_256_Fixed_Point_Identity;
        verify by AC_04_Modular_Multi_File_Flutter_Contract_Emission_and_Monolith_Barrel_Decomposition;
        verify by AC_05_Zero_String_Concatenation_Codegen_DO_178C_Seven_Section_Docstring_and_Clippy_Forbid_Compliance;
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
    connection def BP_07_PartitionerToRustEmitterConnection {
        connect FourLevelAstPartitionerAndHoister.module_graph_out to ModularRustMultiFileEmitter.rust_module_graph_in;
    }
    doc /* - **Given**: A SysML v2 PackageDef fixture containing either a directed dependency cycle across packages/classifiers or a @Layer(Domain) classifier referencing a @Layer(Presentation) or @Layer(Application) classifier. - **When**: compile-sysml --emit-modular-rust <OUT_DIR> --schema <FIXTURE> is executed. - **Then**: TopologicalLayerDagVerifier aborts transactionally with exit code 1, emits structured diagnostic E0702 (cycle) or E0703 (illegal upward layer dependency), and leaves <OUT_DIR> with zero generated files written. */
    use case def UC_AC_01_Fail_Closed_Rejection_on_Directed_Dependency_Cycles_and_Upward_Layer_Imports {
        subject FourLevelAstPartitionerAndHoister;
        objective "Fail Closed Rejection on Directed Dependency Cycles and Upward Layer Imports";
    }
    doc /* - **Given**: A valid multi-package SysML v2 model containing nested PackageDef hierarchies, multi-facet PartDef components, a 40-step ActionDef sequence, a 30-transition StateDef state machine, and a 25-clause ConstraintDef boolean Expr tree. - **When**: compile-sysml --emit-modular-rust <OUT_DIR> --schema <FIXTURE> compiles and lowers the model across Levels 1 through 4 via proc-macro2 + quote::quote! + syn::parse2::<syn::File> and prettyplease::unparse. - **Then**: Compilation succeeds with exit code 0, emitting hierarchical mod.rs barrels, facet submodules (model, invariants, ports, fsm,. */
    use case def UC_AC_02_Four_Level_Deterministic_Lowering_with_Typed_syn_File_AST_and_prettyplease_Formatting {
        subject FourLevelAstPartitionerAndHoister;
        objective "Four Level Deterministic Lowering with Typed syn File AST and prettyplease Formatting";
    }
    doc /* - **Given**: The authoritative self-hosting compiler schema .pipeline/upstream/schema/modular_codegen.sysml and the committed Stage-1 tree under crates/compile-sysml/src/modular_codegen/generated/ verified parse-clean by ra_ap_syntax and wired into compile-sysml. - **When**: The Stage-1-linked compile-sysml binary re-compiles .pipeline/upstream/schema/modular_codegen.sysml into an isolated Stage-2 temporary directory and compares GeneratedTreeManifest SHA-256 digests. - **Then**: SHA-256(Stage 1) == SHA-256(Stage 2) holds across 100% of relative paths and file contents (**C-MOD-04**. */
    use case def UC_AC_03_Metacircular_Self_Hosting_Stage_1_to_Stage_2_SHA_256_Fixed_Point_Identity {
        subject FourLevelAstPartitionerAndHoister;
        objective "Metacircular Self Hosting Stage 1 to Stage 2 SHA 256 Fixed Point Identity";
    }
    doc /* - **Given**: A multi-package SysML v2 schema compiled via compile-sysml --emit-flutter-contracts <APP_DIR> --schema <FIXTURE> and the Rust workspace source file crates/compile-sysml/src/flutter_codegen.rs. - **When**: Modular Flutter contract generation executes and workspace modularity tests inspect both the emitted Dart tree and crates/compile-sysml/src/. - **Then**: Per-package and per-classifier .g.dart units (models/generated/<pkg>/<part>_view.g.dart, data/generated/ffi/ *.g.dart, data/generated/dto/<pkg>/<dto>_dto.g.dart, data/seeds/generated/ *.g.dart, test/generated/support/ *.g.dart). */
    use case def UC_AC_04_Modular_Multi_File_Flutter_Contract_Emission_and_Monolith_Barrel_Decomposition {
        subject FourLevelAstPartitionerAndHoister;
        objective "Modular Multi File Flutter Contract Emission and Monolith Barrel Decomposition";
    }
    doc /* - **Given**: All hand-authored modules under crates/compile-sysml/src/modular_codegen/ ** and all Stage-1 generated Rust modules under crates/compile-sysml/src/modular_codegen/generated/ **. - **When**: cargo clippy --workspace --all-targets -- -D warnings and ./target/release/verify-baseline audit-code --path crates/compile-sysml/src/modular_codegen --dry-run are executed. - **Then**: Both verification gates exit 0 with zero Clippy forbid violations, zero Check 34 violation tokens, zero Check 19 hardcoded domain concepts, zero legacy string-concatenation / format! code emitters, and 100%. */
    use case def UC_AC_05_Zero_String_Concatenation_Codegen_DO_178C_Seven_Section_Docstring_and_Clippy_Forbid_Compliance {
        subject FourLevelAstPartitionerAndHoister;
        objective "Zero String Concatenation Codegen DO 178C Seven Section Docstring and Clippy Forbid Compliance";
    }
    doc /* - **Given**: This engineering blueprint at docs/designs/blueprint-07-self-hosting-modular-codegen-compiler.md. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-07-self-hosting-modular-codegen-compiler.md is executed. - **Then**: All 12 blueprint verification gates pass with exit code 0, deterministically synthesizing docs/designs/blueprint-07-self-hosting-modular-codegen-compiler.sysml and .digest.json via roundtrip SysML v2 compilation and verifying closed-loop GitHub SSOT parity. */
    use case def UC_AC_06_Dogfood_SysML_v2_Blueprint_Ingestion_and_Twelve_Gate_Closed_Loop_Conformance {
        subject FourLevelAstPartitionerAndHoister;
        objective "Dogfood SysML v2 Blueprint Ingestion and Twelve Gate Closed Loop Conformance";
    }
    test case def TC_REQ_0607_Self_Hosting_Modular_Codegen {
        verify requirement REQ_0607;
    }
    test case def TC_REQ_0607_Blueprint_Conformance {
        verify requirement REQ_0607_BP_07_Self_Hosting_4_Level_Modular_Multi_File_Code_Generator_Blueprint;
    }
    test case def Test_verify_dag_and_layers {
        verify requirement REQ_0607;
    }
    test case def Test_partition_and_hoist {
        verify requirement REQ_0607;
    }
    test case def Test_synthesize_module_tokens {
        verify requirement REQ_0607;
    }
    test case def Test_build_syn_file {
        verify requirement REQ_0607;
    }
    test case def Test_format_pretty_syn_file {
        verify requirement REQ_0607;
    }
    test case def Test_emit_modular_rust_tree {
        verify requirement REQ_0607;
    }
    test case def Test_emit_modular_flutter_tree {
        verify requirement REQ_0607;
    }
    test case def Test_verify_fixed_point_digest {
        verify requirement REQ_0607;
    }
    test case def Test_ExecuteFourLevelLoweringPipeline {
        verify requirement REQ_0607;
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
        doc /* Net-negative legacy string-concatenation emitter deletion target in compile-sysml. [unit: LOC] */
        attribute legacy_string_codegen_deleted_sloc : Integer = 1100;
        doc /* Strict execution SLOC ceiling per modular codegen submodule. [unit: LOC] */
        attribute submodule_max_exec_sloc : Integer = 380;
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

doc /* Subsystem package for BP_08_Lossless_Rust_CST_Codemod_Engine_and_Unified_rustc_Compatible_JSON_Diagnostics */
package BP_08_Lossless_Rust_CST_Codemod_Engine_and_Unified_rustc_Compatible_JSON_Diagnostics {
    attribute Blueprint_ID : String = "BP-08";
    attribute Requirement_ID : String = "REQ-0608";
    attribute Title : String = "Lossless Rust CST Codemod Engine and Unified rustc-Compatible JSON Diagnostics";
    attribute Status : String = "Draft";
    attribute Target_Crates : String = "crates/deap-core, crates/compile-sysml, crates/verify-baseline, crates/ingest-sysml, crates/assemble-conops, crates/reconcile-backlog";
    attribute Tracked_Issues : String = "#742";
    attribute Generation_Mode : String = "subagent";
    attribute Specification_Source : String = "crates/deap-core/src/rust_cst/mod.rs:1-80";
    doc /* `ra_ap_vfs` and `camino::Utf8Path` cross-platform virtual file system with CRLF/LF normalization. */
    attribute crates_deap_core_src_rust_cst_vfs_rs : String = "**New `< 380` SLOC**";
    doc /* Lossless Red/Green CST parser wrapping `ra_ap_syntax::SourceFile` with zero syntax-tree text-contains scans. */
    attribute crates_deap_core_src_rust_cst_parse_rs : String = "**New `< 380` SLOC**";
    doc /* Deterministic `BTreeMap`-ordered AST item inventory and DEAP v2.0 metrics extraction. */
    attribute crates_deap_core_src_rust_cst_items_rs : String = "**New `< 380` SLOC**";
    doc /* Atomic module partitioner with CST byte-span splicing and Check-37 blueprint path updates. */
    attribute crates_deap_core_src_rust_cst_extract_module_rs : String = "**New `< 380` SLOC**";
    doc /* Lossless `DocTier` classifier and 2-line docblock compactor (`--compact-tier-bc`). */
    attribute crates_deap_core_src_rust_cst_doc_edit_rs : String = "**New `< 380` SLOC**";
    doc /* Single-line `rustc`-compatible NDJSON diagnostic emitter and `MachineApplicable` fix engine. */
    attribute crates_deap_core_src_rust_cst_diagnostics_ndjson_rs : String = "**New `< 380` SLOC**";
    doc /* `cargo-lock`, `cargo_metadata`, `cargo-util-schemas`, `cargo-platform`, and `toml_edit` offline lockfile and manifest verifier. */
    attribute crates_deap_core_src_rust_cst_manifest_lock_rs : String = "**New `< 380` SLOC**";
    doc /* Completely deleted and superseded by typed `rust_cst` modules. */
    attribute crates_deap_core_src_rust_cst_mod_rs : String = "**Complete Replacement `-1,049` SLOC**";
    doc /* Replaced by `deap_core::rust_cst::doc_edit` CST docblock operations. */
    attribute crates_compile_sysml_src_docstring_rs : String = "**Net Deletion `-4,900+` SLOC**";
    doc /* Migrated from hand-rolled line parsers to `DocTier` CST metrics. */
    attribute crates_verify_baseline_src_checks_docstring_tier_rules_rs : String = "**Net Deletion `-3,300+` SLOC**";
    doc /* Migrated from regex/token heuristics to exact `ra_ap_syntax` item traversal. */
    attribute crates_verify_baseline_src_checks_structural_rs : String = "**Net Deletion `-4,200+` SLOC**";
    doc /* Migrated workspace directory traversal to `camino` + `ignore` + `ra_ap_vfs`. */
    attribute crates_deap_core_src_workspace_rs : String = "**Net Deletion `-620+` SLOC**";
    doc /* Downward ratchet converges strictly to `0` legacy parser exemptions across all crates. */
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
    port def BP_08_LosslessRustCstPortDef {
        flow in rust_cst_in : String;
        flow out ndjson_diag_out : String;
    }
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
        text = "LosslessRustCstAndJsonDiagnosticsEngine (REQ-0608 / BP-08 / DEAP-BP-DET-08, UUIDv5 6ba7b810-9dad-11d1-80b4-00c04fd430c8, Class P linear syntax-tree traversal and byte-span splicing complexity, governed by DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C) SHALL enforce Contract A (full 6-clause OMG SysML v2.0 + KerML 1.0 grammar coverage with zero 16-construct caps) and Contract B (lossless ra_ap_syntax + rowan Red/Green CST codemods, ra_ap_vfs, ra_ap_paths, ra_ap_line_index, ra_ap_text_edit, camino, ignore, cargo-lock, cargo_metadata, cargo-util-schemas, cargo-platform, toml_edit, pulldown-cmark, pulldown-cmark-to-cmark, and single-line rustc-compatible JSON diagnostics inside < 380-SLOC submodules while deleting -14,000+ SLOC of legacy hand-rolled scanners).";
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
    connection def BP_08_CodemodToDiagnosticEmitterConnection {
        connect ZeroTokenRustSpanCodemodEngine.codemod_error_out to UnifiedRustcJsonDiagnosticEmitter.finding_in;
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
    test case def TC_REQ_0608_Lossless_Rust_Cst_Codemod {
        verify requirement REQ_0608;
    }
    test case def TC_REQ_0608_Blueprint_Conformance {
        verify requirement REQ_0608_BP_08_Lossless_Rust_CST_Codemod_Engine_and_Unified_rustc_Compatible_JSON_Diagnostics;
    }
    test case def Test_verify_offline_locked_build {
        verify requirement REQ_0608;
    }
    test case def Test_list_rust_items {
        verify requirement REQ_0608;
    }
    test case def Test_extract_rust_module {
        verify requirement REQ_0608;
    }
    test case def Test_verify_stage_fixed_point {
        verify requirement REQ_0608;
    }
    test case def Test_format_ndjson_line {
        verify requirement REQ_0608;
    }
    test case def Test_analyze_metrics {
        verify requirement REQ_0608;
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

doc /* Subsystem package for BP_09_Multi_Stage_Compiler_and_Formal_Verification_Gatekeeper */
package BP_09_Multi_Stage_Compiler_and_Formal_Verification_Gatekeeper {
    attribute Blueprint_ID : String = "BP-09";
    attribute Requirement_ID : String = "REQ-0609";
    attribute Title : String = "Multi-Stage Compiler and Formal Verification Gatekeeper (Clippy, Dylint MIR, Miri Tree Borrows, Kani CBMC, and flutter_rust_bridge FFI Synthesis)";
    attribute Status : String = "Draft";
    attribute Target_Crates : String = "crates/deap-core, crates/compile-sysml, crates/verify-baseline";
    attribute Tracked_Issues : String = "#742";
    attribute Generation_Mode : String = "subagent";
    attribute Specification_Source : String = "Cargo.toml:12-25";
    doc /* [unit: `lines`] */
    attribute too_many_lines_threshold : Real;
    doc /* [unit: `params`] */
    attribute too_many_arguments_threshold : Real;
    doc /* [unit: `allocs`] */
    attribute max_rt_heap_allocations : Real;
    doc /* [unit: `violations`] */
    attribute max_miri_ub_violations : Real;
    doc /* [unit: `counterexamples`] */
    attribute max_kani_counterexamples : Real;
    doc /* [unit: `iterations`] */
    attribute default_kani_unwind_bound : Real;
    doc /* [unit: `SLOC`] */
    attribute max_module_exec_sloc : Real;
    doc /* [unit: `probes`] */
    attribute min_toolchain_discovery_probes : Real;
    port def BP_09_FormalVerificationPortDef {
        flow in verification_ir_in : String;
        flow out verification_receipt_out : String;
    }
    doc /* probe_formal_toolchain(config) */
    action def probe_formal_toolchain;
    doc /* verify_clippy_policy(policy, manifest_lints) */
    action def verify_clippy_policy;
    doc /* evaluate_stage2_dylint_mir(input) */
    action def evaluate_stage2_dylint_mir;
    doc /* verify_in_process_cst_syntax(module_path, max_exec_sloc) */
    action def verify_in_process_cst_syntax;
    doc /* guard_inbound_frb_f64(spec, raw_value, active_fsm_state) */
    action def guard_inbound_frb_f64;
    doc /* validate_kani_harness_spec(spec) */
    action def validate_kani_harness_spec;
    doc /* evaluate_four_stage_gatekeeper(receipts) */
    action def evaluate_four_stage_gatekeeper;
    doc /* [unit: `lines`] */
    assert constraint C_VER_01 {
        too_many_lines_threshold >= 1 and too_many_lines_threshold <= 80;
    }
    doc /* [unit: `params`] */
    assert constraint C_VER_02 {
        too_many_arguments_threshold >= 0 and too_many_arguments_threshold <= 5;
    }
    doc /* [unit: `allocs`] */
    assert constraint C_VER_03 {
        max_rt_heap_allocations >= 0 and max_rt_heap_allocations <= 0;
    }
    doc /* [unit: `violations`] */
    assert constraint C_VER_04 {
        max_miri_ub_violations >= 0 and max_miri_ub_violations <= 0;
    }
    doc /* [unit: `counterexamples`] */
    assert constraint C_VER_05 {
        max_kani_counterexamples >= 0 and max_kani_counterexamples <= 0;
    }
    doc /* [unit: `iterations`] */
    assert constraint C_VER_06 {
        default_kani_unwind_bound >= 1 and default_kani_unwind_bound <= 64;
    }
    doc /* [unit: `SLOC`] */
    assert constraint C_VER_07 {
        max_module_exec_sloc >= 1 and max_module_exec_sloc <= 380;
    }
    doc /* [unit: `probes`] */
    assert constraint C_VER_08 {
        min_toolchain_discovery_probes >= 1 and min_toolchain_discovery_probes <= 3;
    }
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0609_BP_09_Multi_Stage_Compiler_and_Formal_Verification_Gatekeeper {
        id = "REQ-0609";
        text = "MultiStageFormalVerificationGatekeeper (REQ-0609 / BP-09 / DEAP-BP-DET-09, UUIDv5 6ba7b810-9dad-11d1-80b4-00c04fd430c9, Class P static HIR/THIR/MIR linear pass complexity paired with bounded CBMC state-space unwinding, governed by DO-178C Level A, DO-330 TQL-4, DO-333 Formal Methods Supplement, and ISO 26262 ASIL D) SHALL enforce Contract A (full 6-clause OMG SysML v2.0 + KerML 1.0 grammar coverage with zero 16-construct caps, #rt_loop/#dal/#fls/#hazard/#risk/#capability metadata def bindings) and Contract B (5-stage verification pipeline: Stage 1 stable cargo clippy --message-format=json, Stage 2 isolated ./lints/deap-dylint-mir/ reading rustc_middle::mir::Body and TyCtxt, Stage 3 flutter_rust_bridge + cargo +nightly miri test -Zmiri-tree-borrows, Stage 4 cargo kani CBMC synthesized via syn + proc-macro2 + quote + prettyplease, and Stage 5 Ferrocene criticalup inside < 380-SLOC submodules).";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_stage_1_universal_stable_clippy_total_function_and_clippy_toml_threshold_enforcement : String = "- **Given**: An upstream or generated downstream Rust workspace configured with [workspace.lints.clippy] enforcing all 14 total-function, arithmetic, cast, unsafe-doc, and complexity denials plus root clippy.toml (too-many-lines-threshold = 80, too-many-arguments-threshold = 5, and disallowed-types banning non-deterministic HashMap / HashSet and backtracking pattern-matching types), alongside a negative fixture containing unchecked + arithmetic, direct slice indexing, an 81-line function, a 6-argument function, or std::collections::HashMap. - **When**: StableClippyTotalFunctionGate executes.";
        attribute ac_02_stage_2_isolated_nightly_dylint_mir_zero_allocation_proof_on_real_time_control_loops : String = "- **Given**: A standalone nightly Dylint MIR lint workspace isolated from root stable Cargo.toml, evaluating two #[rt_loop] control functions: one pure stack-only total function and one negative fixture that transitively invokes Box::new, Vec::push growth, format! heap allocation, or dynamic drop-glue deallocation through an indirect helper function. - **When**: IsolatedDylintMirAllocationGate traverses rustc_middle::mir::Body basic-block call and drop terminators across the #[rt_loop] call graph while stable cargo check --workspace --offline --locked runs independently on the root workspace.";
        attribute ac_03_stage_2_compile_time_mir_requirement_binding_verification_against_active_sysml_digests : String = "- **Given**: Rust items annotated with #[realises(\"REQ-0609\")] or /// Realises: [REQ-0609] verified against an active .digest.json compiled from .sysml, plus a negative fixture referencing a non-existent or retired identifier REQ-9999. - **When**: IsolatedDylintMirAllocationGate cross-checks extracted HIR/MIR DefId requirement bindings against the active SysML requirement digest inventory. - **Then**: Valid REQ-0609 bindings pass cleanly, whereas the dangling REQ-9999 annotation fails closed with non-zero exit status and single-line JSON diagnostic DEAP-E0902.";
        attribute ac_04_stage_3_deterministic_flutter_rust_bridge_ffi_synthesis_with_fail_closed_icd_boundary_guards : String = "- **Given**: A SysML v2 package and ICD_01 / ICD_02 specification declaring port def and item flow signals with numeric [min..max] bounds and admissible state def operational modes, lowered by crates/compile-sysml/src/modular_codegen/. - **When**: compile-sysml synthesizes the #![forbid(unsafe_code)] pure domain crate, the isolated flutter_rust_bridge FFI boundary facade, and the Dart DTO/binding modules, and executes inbound FFI boundary tests alongside dart analyze --fatal-infos. - **Then**: Pure domain modules contain #![forbid(unsafe_code)], synthesized Dart bindings pass dart analyze.";
        attribute ac_05_stage_3_miri_tree_borrows_abstract_machine_undefined_behavior_rejection_on_ffi_and_ring_buffer_bridges : String = "- **Given**: An isolated Stage-3 FFI marshaling and lock-free SPSC ring-buffer bridge module containing documented // SAFETY: blocks, paired with a negative mutation probe introducing a Tree Borrows pointer-aliasing violation or uninitialized byte read. - **When**: FrbIcdBridgeAndMiriUbGate executes cargo +nightly miri test with MIRIFLAGS=\"-Zmiri-tree-borrows\". - **Then**: The compliant FFI/ring-buffer bridge passes Miri with 0 undefined-behavior findings (**C-VER-04**) and 0 leaked allocations, whereas the mutated aliasing/uninitialized probe fails closed with a non-zero exit code and.";
        attribute ac_06_stage_4_deterministic_kani_cbmc_proof_synthesis_and_stpa_unsafe_control_action_falsification : String = "- **Given**: SysML constraint def invariants, port def [min..max] intervals, state def transition tables, and STPA **UCA-01** / **SC-01** safety constraints lowered into #[cfg(kani)] modules via syn + quote + prettyplease across deap-core, compile-sysml, and a generated downstream crate, plus a negative mutation weakening an overflow check or STPA state guard. - **When**: KaniCbmcProofSynthesizer emits the #[kani::proof] + #[kani::unwind(N)] harnesses (kani::any() + kani::assume(...) + assert!(...)) and executes cargo kani. - **Then**: Emitted #[cfg(kani)] source files are byte-deterministic.";
        attribute ac_07_cross_platform_toolchain_discovery_modular_submodule_layout_and_fail_closed_qualification_gate : String = "- **Given**: docs/designs/blueprint-09-clippy-dylint-miri-kani-formal-verification-gate.md specifying dynamic cross-platform driver probing across macOS, Linux, and Windows (std::env::consts::{DLL_PREFIX, DLL_SUFFIX, EXE_SUFFIX}, camino::Utf8Path), modular submodules under < 380 SLOC (**C-VER-07**), and the --require-formal-toolchains qualification flag, evaluated in an environment where a required driver is absent versus present. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-09-clippy-dylint-miri-kani-formal-verification-gate.md and.";
        doc /* \begin{aligned} \text{Stage1Pass}(\mathcal{W}) &\iff \forall C \in \mathcal{W}, \quad \forall \ell \in \mathcal{L}_{\text{deny}}, \quad \text{Violations}_{\ell}(C) = \emptyset \land \text{Types}(C) \cap \mathcal{T}_{\text{banned}} = \emptyset \\ \forall f \in \text{Fns}(\mathcal{W}), \quad \text{Lines}_{\text{HIR}}(f) &\le 80 \land \lvert \text{Args}_{\text{HIR}}(f) \rvert \le 5 \land \text{BoolParams}_{\text{HIR}}(f) \le 3 \end{aligned} */
        constraint def Invariant_REQ_0609_SC_01;
        doc /* \begin{aligned} \text{ZeroRtHeap}(G_{\text{MIR}}) &\iff \forall f_{\text{rt}} \in V_{\text{rt\_loop}}, \quad \mathcal{R}^*(f_{\text{rt}}) \cap \mathcal{A}_{\text{heap}} = \emptyset \\ \text{TraceableMIR}(V_{\text{def}}, \mathcal{Q}_{\text{sysml}}) &\iff \forall d \in V_{\text{def}}, \quad \text{Realises}(d) \neq \emptyset \implies \text{Realises}(d) \subseteq \mathcal{Q}_{\text{sysml}} \end{aligned} */
        constraint def Invariant_REQ_0609_SC_02;
        doc /* \begin{aligned} G_p(u, s) &\iff \text{is\_finite}(u) \land (p_{\min} \le u \le p_{\max}) \land (s \in S_p) \\ \text{Facade}_p(u, s) &= \begin{cases} \text{Ok}(\delta_{\text{domain}}(u, s)) & \text{if } G_p(u, s) \\ \text{Err}(\text{DEAP\_E0903}) & \text{if } \neg G_p(u, s) \end{cases} \\ \text{Stage3Pass}(\mathcal{W}_{\text{domain}}, \mathcal{B}_{\text{FFI}}) &\iff \text{UnsafeBlocks}(\mathcal{W}_{\text{domain}}) = 0 \land \forall \sigma \in \text{Traces}_{\text{Miri}}(\mathcal{B}_{\text{FFI}}), \quad \text{UB}_{\text{TreeBorrows}}(\sigma) = 0 \end{aligned} */
        constraint def Invariant_REQ_0609_SC_03;
        doc /* \begin{aligned} \text{VC}_f(x) &\equiv \text{Pre}_{\text{sysml}}(x) \implies \Big( \neg \text{Panic}_N(f, x) \land \neg \text{ArithOverflow}_N(f, x) \land \neg \text{OutOfBounds}_N(f, x) \\ &\land \text{Post}_{\text{sysml}}(x, f(x)) \land \bigwedge_{j=1}^{m} \neg \text{UCA}_j(x, f(x)) \Big) \\ \text{Stage4Pass}(f, N) &\iff \text{SAT}\big(\exists x \in \Sigma_{\text{CBMC}}, \quad \neg \text{VC}_f(x)\big) = \text{UNSAT} \end{aligned} */
        constraint def Invariant_REQ_0609_SC_04;
        doc /* \begin{aligned} \text{ToolchainPass}(\mathcal{D}, Q_{\text{ci}}) &\iff \big( Q_{\text{ci}} \implies \forall d \in \mathcal{D}, \quad \text{Discovered}(d) = \text{true} \big) \\ &\land \forall m \in \mathcal{M}_{\text{formal}}, \quad \text{ExecSLOC}(m) \le 380 \land \text{SyntaxErrors}(m) = 0 \end{aligned} */
        constraint def Invariant_REQ_0609_SC_05;
        require Invariant_REQ_0609_SC_01;
        require Invariant_REQ_0609_SC_02;
        require Invariant_REQ_0609_SC_03;
        require Invariant_REQ_0609_SC_04;
        require Invariant_REQ_0609_SC_05;
        verify by AC_01_Stage_1_Universal_Stable_Clippy_Total_Function_and_clippy_toml_Threshold_Enforcement;
        verify by AC_02_Stage_2_Isolated_Nightly_Dylint_MIR_Zero_Allocation_Proof_on_Real_Time_Control_Loops;
        verify by AC_03_Stage_2_Compile_Time_MIR_Requirement_Binding_Verification_Against_Active_SysML_Digests;
        verify by AC_04_Stage_3_Deterministic_flutter_rust_bridge_FFI_Synthesis_with_Fail_Closed_ICD_Boundary_Guards;
        verify by AC_05_Stage_3_Miri_Tree_Borrows_Abstract_Machine_Undefined_Behavior_Rejection_on_FFI_and_Ring_Buffer_Bridges;
        verify by AC_06_Stage_4_Deterministic_Kani_CBMC_Proof_Synthesis_and_STPA_Unsafe_Control_Action_Falsification;
        verify by AC_07_Cross_Platform_Toolchain_Discovery_Modular_Submodule_Layout_and_Fail_Closed_Qualification_Gate;
        satisfy by FrbIcdBridgeAndMiriUbGate;
    }
    doc /* Extracted from Mermaid stateDiagram specification. */
    state def BP_09_Multi_Stage_Compiler_and_Formal_Verification_Gatekeeper_LifecycleState {
        state Idle;
        state ProbingFormalDrivers;
        state RunningStage1Clippy;
        state FailedClosedJsonDiagnostic;
        state RunningStage2DylintMir;
        state RunningStage3FrbAndMiri;
        state RunningStage4KaniCbmc;
        state VerifiedAllFourStages;
        transition Initial_to_Idle first Initial then Idle;
        transition Idle_to_ProbingFormalDrivers_on_probe_formal_toolchain first Idle accept probe_formal_toolchain then ProbingFormalDrivers;
        transition ProbingFormalDrivers_to_RunningStage1Clippy_on_toolchainsDiscoveredOrPermitted first ProbingFormalDrivers accept toolchainsDiscoveredOrPermitted then RunningStage1Clippy;
        transition ProbingFormalDrivers_to_FailedClosedJsonDiagnostic_on_rejectMissingToolchainInQualification first ProbingFormalDrivers accept rejectMissingToolchainInQualification then FailedClosedJsonDiagnostic;
        transition RunningStage1Clippy_to_FailedClosedJsonDiagnostic_on_rejectStage1ClippyViolation first RunningStage1Clippy accept rejectStage1ClippyViolation then FailedClosedJsonDiagnostic;
        transition RunningStage1Clippy_to_RunningStage2DylintMir_on_stage1TotalFunctionsVerified first RunningStage1Clippy accept stage1TotalFunctionsVerified then RunningStage2DylintMir;
        transition RunningStage2DylintMir_to_FailedClosedJsonDiagnostic_on_rejectMirHeapAllocOrDanglingReq first RunningStage2DylintMir accept rejectMirHeapAllocOrDanglingReq then FailedClosedJsonDiagnostic;
        transition RunningStage2DylintMir_to_RunningStage3FrbAndMiri_on_stage2ZeroRtAllocVerified first RunningStage2DylintMir accept stage2ZeroRtAllocVerified then RunningStage3FrbAndMiri;
        transition RunningStage3FrbAndMiri_to_FailedClosedJsonDiagnostic_on_rejectMiriTreeBorrowsOrDartFatalInfo first RunningStage3FrbAndMiri accept rejectMiriTreeBorrowsOrDartFatalInfo then FailedClosedJsonDiagnostic;
        transition RunningStage3FrbAndMiri_to_RunningStage4KaniCbmc_on_stage3FrbMiriAndDartVerified first RunningStage3FrbAndMiri accept stage3FrbMiriAndDartVerified then RunningStage4KaniCbmc;
        transition RunningStage4KaniCbmc_to_FailedClosedJsonDiagnostic_on_rejectKaniCbmcCounterexample first RunningStage4KaniCbmc accept rejectKaniCbmcCounterexample then FailedClosedJsonDiagnostic;
        transition RunningStage4KaniCbmc_to_VerifiedAllFourStages_on_stage4AllSafetyProofsUnsat first RunningStage4KaniCbmc accept stage4AllSafetyProofsUnsat then VerifiedAllFourStages;
        transition FailedClosedJsonDiagnostic_to_Terminal first FailedClosedJsonDiagnostic then Terminal;
        transition VerifiedAllFourStages_to_Terminal first VerifiedAllFourStages then Terminal;
    }
    connection def BP_09_ClippyToOrchestratorConnection {
        connect StableClippyTotalFunctionGate.clippy_ndjson_out to MultiStageVerificationOrchestrator.gate_invocation_in;
    }
    doc /* - **Given**: An upstream or generated downstream Rust workspace configured with [workspace.lints.clippy] enforcing all 14 total-function, arithmetic, cast, unsafe-doc, and complexity denials plus root clippy.toml (too-many-lines-threshold = 80, too-many-arguments-threshold = 5, and disallowed-types banning non-deterministic HashMap / HashSet and backtracking pattern-matching types), alongside a negative fixture containing unchecked + arithmetic, direct slice indexing, an 81-line function, a 6-argument function, or std::collections::HashMap. - **When**: StableClippyTotalFunctionGate executes. */
    use case def UC_AC_01_Stage_1_Universal_Stable_Clippy_Total_Function_and_clippy_toml_Threshold_Enforcement {
        subject FrbIcdBridgeAndMiriUbGate;
        objective "Stage 1 Universal Stable Clippy Total Function and clippy toml Threshold Enforcement";
    }
    doc /* - **Given**: A standalone nightly Dylint MIR lint workspace isolated from root stable Cargo.toml, evaluating two #[rt_loop] control functions: one pure stack-only total function and one negative fixture that transitively invokes Box::new, Vec::push growth, format! heap allocation, or dynamic drop-glue deallocation through an indirect helper function. - **When**: IsolatedDylintMirAllocationGate traverses rustc_middle::mir::Body basic-block call and drop terminators across the #[rt_loop] call graph while stable cargo check --workspace --offline --locked runs independently on the root workspace. */
    use case def UC_AC_02_Stage_2_Isolated_Nightly_Dylint_MIR_Zero_Allocation_Proof_on_Real_Time_Control_Loops {
        subject FrbIcdBridgeAndMiriUbGate;
        objective "Stage 2 Isolated Nightly Dylint MIR Zero Allocation Proof on Real Time Control Loops";
    }
    doc /* - **Given**: Rust items annotated with #[realises(\"REQ-0609\")] or /// Realises: [REQ-0609] verified against an active .digest.json compiled from .sysml, plus a negative fixture referencing a non-existent or retired identifier REQ-9999. - **When**: IsolatedDylintMirAllocationGate cross-checks extracted HIR/MIR DefId requirement bindings against the active SysML requirement digest inventory. - **Then**: Valid REQ-0609 bindings pass cleanly, whereas the dangling REQ-9999 annotation fails closed with non-zero exit status and single-line JSON diagnostic DEAP-E0902. */
    use case def UC_AC_03_Stage_2_Compile_Time_MIR_Requirement_Binding_Verification_Against_Active_SysML_Digests {
        subject FrbIcdBridgeAndMiriUbGate;
        objective "Stage 2 Compile Time MIR Requirement Binding Verification Against Active SysML Digests";
    }
    doc /* - **Given**: A SysML v2 package and ICD_01 / ICD_02 specification declaring port def and item flow signals with numeric [min..max] bounds and admissible state def operational modes, lowered by crates/compile-sysml/src/modular_codegen/. - **When**: compile-sysml synthesizes the #![forbid(unsafe_code)] pure domain crate, the isolated flutter_rust_bridge FFI boundary facade, and the Dart DTO/binding modules, and executes inbound FFI boundary tests alongside dart analyze --fatal-infos. - **Then**: Pure domain modules contain #![forbid(unsafe_code)], synthesized Dart bindings pass dart analyze. */
    use case def UC_AC_04_Stage_3_Deterministic_flutter_rust_bridge_FFI_Synthesis_with_Fail_Closed_ICD_Boundary_Guards {
        subject FrbIcdBridgeAndMiriUbGate;
        objective "Stage 3 Deterministic flutter rust bridge FFI Synthesis with Fail Closed ICD Boundary Guards";
    }
    doc /* - **Given**: An isolated Stage-3 FFI marshaling and lock-free SPSC ring-buffer bridge module containing documented // SAFETY: blocks, paired with a negative mutation probe introducing a Tree Borrows pointer-aliasing violation or uninitialized byte read. - **When**: FrbIcdBridgeAndMiriUbGate executes cargo +nightly miri test with MIRIFLAGS=\"-Zmiri-tree-borrows\". - **Then**: The compliant FFI/ring-buffer bridge passes Miri with 0 undefined-behavior findings (**C-VER-04**) and 0 leaked allocations, whereas the mutated aliasing/uninitialized probe fails closed with a non-zero exit code and. */
    use case def UC_AC_05_Stage_3_Miri_Tree_Borrows_Abstract_Machine_Undefined_Behavior_Rejection_on_FFI_and_Ring_Buffer_Bridges {
        subject FrbIcdBridgeAndMiriUbGate;
        objective "Stage 3 Miri Tree Borrows Abstract Machine Undefined Behavior Rejection on FFI and Ring Buffer Bridges";
    }
    doc /* - **Given**: SysML constraint def invariants, port def [min..max] intervals, state def transition tables, and STPA **UCA-01** / **SC-01** safety constraints lowered into #[cfg(kani)] modules via syn + quote + prettyplease across deap-core, compile-sysml, and a generated downstream crate, plus a negative mutation weakening an overflow check or STPA state guard. - **When**: KaniCbmcProofSynthesizer emits the #[kani::proof] + #[kani::unwind(N)] harnesses (kani::any() + kani::assume(...) + assert!(...)) and executes cargo kani. - **Then**: Emitted #[cfg(kani)] source files are byte-deterministic. */
    use case def UC_AC_06_Stage_4_Deterministic_Kani_CBMC_Proof_Synthesis_and_STPA_Unsafe_Control_Action_Falsification {
        subject FrbIcdBridgeAndMiriUbGate;
        objective "Stage 4 Deterministic Kani CBMC Proof Synthesis and STPA Unsafe Control Action Falsification";
    }
    doc /* - **Given**: docs/designs/blueprint-09-clippy-dylint-miri-kani-formal-verification-gate.md specifying dynamic cross-platform driver probing across macOS, Linux, and Windows (std::env::consts::{DLL_PREFIX, DLL_SUFFIX, EXE_SUFFIX}, camino::Utf8Path), modular submodules under < 380 SLOC (**C-VER-07**), and the --require-formal-toolchains qualification flag, evaluated in an environment where a required driver is absent versus present. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-09-clippy-dylint-miri-kani-formal-verification-gate.md and. */
    use case def UC_AC_07_Cross_Platform_Toolchain_Discovery_Modular_Submodule_Layout_and_Fail_Closed_Qualification_Gate {
        subject FrbIcdBridgeAndMiriUbGate;
        objective "Cross Platform Toolchain Discovery Modular Submodule Layout and Fail Closed Qualification Gate";
    }
    test case def TC_REQ_0609_Multi_Stage_Formal_Gatekeeper {
        verify requirement REQ_0609;
    }
    test case def TC_REQ_0609_Blueprint_Conformance {
        verify requirement REQ_0609_BP_09_Multi_Stage_Compiler_and_Formal_Verification_Gatekeeper;
    }
    test case def Test_probe_formal_toolchain {
        verify requirement REQ_0609;
    }
    test case def Test_verify_clippy_policy {
        verify requirement REQ_0609;
    }
    test case def Test_evaluate_stage2_dylint_mir {
        verify requirement REQ_0609;
    }
    test case def Test_verify_in_process_cst_syntax {
        verify requirement REQ_0609;
    }
    test case def Test_guard_inbound_frb_f64 {
        verify requirement REQ_0609;
    }
    test case def Test_validate_kani_harness_spec {
        verify requirement REQ_0609;
    }
    test case def Test_evaluate_four_stage_gatekeeper {
        verify requirement REQ_0609;
    }
    doc /* Extracted from Mermaid sequenceDiagram specification. */
    interaction def BP_09_Multi_Stage_Compiler_and_Formal_Verification_Gatekeeper_SequenceInteraction {
        lifeline MultiStageVerificationOrchestrator : MultiStageVerificationOrchestrator;
        lifeline StableClippyTotalFunctionGate : StableClippyTotalFunctionGate;
        lifeline IsolatedDylintMirAllocationGate : IsolatedDylintMirAllocationGate;
        lifeline FrbIcdBridgeAndMiriUbGate : FrbIcdBridgeAndMiriUbGate;
        lifeline KaniCbmcProofSynthesizer : KaniCbmcProofSynthesizer;
        message probe_formal_toolchain from MultiStageVerificationOrchestrator to MultiStageVerificationOrchestrator;
        message verify_clippy_policy from MultiStageVerificationOrchestrator to StableClippyTotalFunctionGate;
        message reply_StageGateReceipt from StableClippyTotalFunctionGate to MultiStageVerificationOrchestrator;
        message evaluate_stage2_dylint_mir from MultiStageVerificationOrchestrator to IsolatedDylintMirAllocationGate;
        message reply_StageGateReceipt from IsolatedDylintMirAllocationGate to MultiStageVerificationOrchestrator;
        message verify_in_process_cst_syntax from MultiStageVerificationOrchestrator to IsolatedDylintMirAllocationGate;
        message reply_InProcessCstReceipt from IsolatedDylintMirAllocationGate to MultiStageVerificationOrchestrator;
        message guard_inbound_frb_f64 from MultiStageVerificationOrchestrator to FrbIcdBridgeAndMiriUbGate;
        message reply_f64 from FrbIcdBridgeAndMiriUbGate to MultiStageVerificationOrchestrator;
        message validate_kani_harness_spec from MultiStageVerificationOrchestrator to KaniCbmcProofSynthesizer;
        message reply_StageGateReceipt from KaniCbmcProofSynthesizer to MultiStageVerificationOrchestrator;
        message evaluate_four_stage_gatekeeper from MultiStageVerificationOrchestrator to MultiStageVerificationOrchestrator;
    }
    doc /* Permitted Rust Abstract Machine undefined-behavior findings under `cargo +nightly miri test` with Tree Borrows. [unit: violations] */
    part def FrbIcdBridgeAndMiriUbGate {
        attribute max_miri_ub_violations : Integer = 0;
        doc /* Permitted static analysis errors, warnings, or info findings under `dart analyze --fatal-infos`. [unit: diagnostics] */
        attribute max_dart_analyze_infos : Integer = 0;
        doc /* Enforces `#![forbid(unsafe_code)]` across all pure domain, lowering, and FSM crates outside isolated FFI/HAL bridges. [unit: flag] */
        attribute forbid_unsafe_in_domain : Boolean = true;
        doc /* Active Miri pointer-provenance and aliasing operational semantics flag enforced on FFI and ring-buffer test suites. [unit: flag] */
        attribute miri_aliasing_model : String = "-Zmiri-tree-borrows";
        doc /* Receives validated SysML `port def`, `item flow`, `state def`, and `ICD_01`/`ICD_02` signal bounds in `compile-sysml`. [protocol: SysmlPortAndItemFlowIcd] */
        port in sysml_icd_ast_in : String {
            flow in sysml_icd_ast_in_flow : String;
        }
        doc /* Emits fail-closed guarded `flutter_rust_bridge` Rust facades and strongly-typed Dart binding modules. [protocol: FrbRustAndDartContractTree] */
        port out frb_facade_and_dart_out : String {
            flow out frb_facade_and_dart_out_flow : String;
        }
        doc /* Emits `DEAP-E0903` boundary-guard, Miri Tree Borrows UB, or `dart analyze --fatal-infos` failure diagnostics. [protocol: MiriAndDartVerificationReceipt] */
        port out miri_and_dart_receipt_out : String {
            flow out miri_and_dart_receipt_out_flow : String;
        }
    }
    doc /* Permitted transitive heap allocation or dynamic drop-deallocation terminators reachable from any `#[rt_loop]` function. [unit: allocs] */
    part def IsolatedDylintMirAllocationGate {
        attribute max_rt_heap_allocations : Integer = 0;
        doc /* Permitted dangling `outer realises("REQ-*")` or doc-contract requirement identifiers absent from compiled `.sysml` digests. [unit: links] */
        attribute max_unverified_req_links : Integer = 0;
        doc /* Isolates `rustc_lint` and `rustc_middle::mir` plugin manifest from stable workspace `Cargo.toml`. [unit: flag] */
        attribute isolated_nightly_workspace : Boolean = true;
        doc /* Pinned nightly `rustc_private` toolchain channel executing the standalone Dylint MIR analysis library. [unit: toolchain] */
        attribute plugin_toolchain_channel : String = "nightly-pinned";
        doc /* Receives lowered `rustc_middle::mir::Body` basic blocks, terminators, and `DefId` attributes alongside `.digest.json`. [protocol: RustcMiddleMirStream] */
        port in mir_body_graph_in : String {
            flow in mir_body_graph_in_flow : String;
        }
        doc /* Emits `DEAP-E0901` real-time heap-allocation violations and `DEAP-E0902` broken requirement-binding diagnostics. [protocol: RustcCompilerMessageNdjson] */
        port out dylint_ndjson_out : String {
            flow out dylint_ndjson_out_flow : String;
        }
    }
    doc /* Permitted reachable CBMC property failures (panics, overflows, div-by-zero, OOB, or STPA `UCA-*` breaches). [unit: counterexamples] */
    part def KaniCbmcProofSynthesizer {
        attribute max_kani_counterexamples : Integer = 0;
        doc /* Deterministic loop unwinding ceiling emitted into `outer kani::unwind(N)` when AST loops carry finite static bounds. [unit: iterations] */
        attribute default_kani_unwind_bound : Integer = 16;
        doc /* Emits `outer cfg(kani)` proof modules via `syn` + `quote` + `prettyplease` without raw string-template concatenation. [unit: flag] */
        attribute structured_prettyplease_codegen : Boolean = true;
        doc /* Receives SysML `constraint def`, `port def` `[min..max]` ranges, `state def` transitions, and STPA `UCA-*`/`SC-*` specs. [protocol: SysmlConstraintStateStpaIr] */
        port in sysml_safety_ast_in : String {
            flow in sysml_safety_ast_in_flow : String;
        }
        doc /* Emits deterministic `#[cfg(kani)]` modules containing `kani::any()`, `kani::assume(...)`, and `assert!(...)` harnesses. [protocol: PrettypleaseKaniRustUnit] */
        port out kani_proof_module_out : String {
            flow out kani_proof_module_out_flow : String;
        }
        doc /* Emits CBMC verification status and `DEAP-E0904` counterexample traces in `rustc`-compatible JSON format. [protocol: KaniVerificationReceipt] */
        port out cbmc_proof_receipt_out : String {
            flow out cbmc_proof_receipt_out_flow : String;
        }
    }
    doc /* Ordered verification stage count (`Stage 1 Clippy` -> `Stage 2 Dylint MIR` -> `Stage 3 FRB + Miri` -> `Stage 4 Kani CBMC` -> `Stage 5 Ferrocene criticalup`). [unit: stages] */
    part def MultiStageVerificationOrchestrator {
        attribute verification_stage_count : Integer = 5;
        doc /* Halts downstream stage promotion whenever any preceding verification stage exits non-zero. [unit: flag] */
        attribute fail_closed_short_circuit : Boolean = true;
        doc /* Mandates live execution of discovered drivers and fails closed if any driver is absent during qualification CI. [unit: flag] */
        attribute require_formal_toolchains : Boolean = false;
        doc /* Receives upstream or downstream target workspace verification commands in `verify-baseline` and `compile-sysml`. [protocol: VerificationGateCommand] */
        port in gate_invocation_in : String {
            flow in gate_invocation_in_flow : String;
        }
        doc /* Emits consolidated single-line NDJSON diagnostics (`DEAP-E0900`..`DEAP-E0905`) and final 5-stage verification receipt. [protocol: RustcCompilerMessageNdjson] */
        port out unified_gate_ndjson_out : String {
            flow out unified_gate_ndjson_out_flow : String;
        }
    }
    doc /* Mandatory count of total-function, cast-safety, arithmetic, unsafe-doc, and structural complexity Clippy lints set to `deny` or `forbid`. [unit: lints] */
    part def StableClippyTotalFunctionGate {
        attribute denied_clippy_lint_count : Integer = 14;
        doc /* Maximum HIR function body line count enforced in root `clippy.toml` (`too-many-lines-threshold`). [unit: lines] */
        attribute too_many_lines_threshold : Integer = 80;
        doc /* Maximum callable parameter count enforced in root `clippy.toml` (`too-many-arguments-threshold`). [unit: params] */
        attribute too_many_arguments_threshold : Integer = 5;
        doc /* Minimum banned non-deterministic hash container and backtracking pattern-engine entries in `clippy.toml`. [unit: types] */
        attribute disallowed_types_count : Integer = 3;
        doc /* Emits single-line `cargo clippy --message-format=json` NDJSON diagnostics into the autonomous remediation loop. [unit: flag] */
        attribute json_diagnostic_stream : Boolean = true;
        doc /* Canonical deterministic blueprint identifier (`DEAP-BP-DET-09`). [unit: uuidv5] */
        attribute deterministic_uuid : String = "6ba7b810-9dad-11d1-80b4-00c04fd430c9";
        doc /* Receives workspace `Cargo.toml`, `clippy.toml`, and target Rust crate source trees. [protocol: CargoAndClippyTomlBundle] */
        port in workspace_manifest_in : String {
            flow in workspace_manifest_in_flow : String;
        }
        doc /* Streams machine-applicable `cargo clippy --message-format=json` lint findings and exit status. [protocol: RustcCompilerMessageNdjson] */
        port out clippy_ndjson_out : String {
            flow out clippy_ndjson_out_flow : String;
        }
    }
}

doc /* Subsystem package for BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine */
package BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine {
    attribute Blueprint_ID : String = "BP-10";
    attribute Requirement_ID : String = "REQ-0610";
    attribute Title : String = "Incremental Lossless SysML v2 and Polyglot Compiler Engine (logos + rowan + salsa + ariadne)";
    attribute Status : String = "Draft";
    attribute Target_Crates : String = "crates/deap-core, crates/compile-sysml, crates/ingest-sysml, crates/verify-baseline";
    attribute Tracked_Issues : String = "#742";
    attribute Generation_Mode : String = "subagent";
    attribute Specification_Source : String = "crates/compile-sysml/src/lexer/logos_lexer.rs:1-60 / crates/deap-core/src/sysml_cst/mod.rs:1-80 / #742";
    doc /* `SysmlToken`, `SysmlSyntaxKind`, `SysmlLanguage`, `CstParseReceipt`, `HirPackageSummary`, `HirSourceMap`, `SourceInput`, `PolyglotFormat`, `PolyglotBackendKind`, and `AriadneSpanReport` across `deap_core::sysml_cst`, `deap_core::dart_cst`, `compile_sysml::salsa_db`, and `ingest_sysml::polyglot_cst`. */
    attribute Layer_1_Domain_State_Data_Model : String = "Lossless Red/Green CSTs (`rowan` pinned to `ra_ap_syntax`), trivia-erased HIR records, `HirSourceMap` byte-span tables, and cross-platform `camino::Utf8PathBuf` inputs";
    doc /* `normalize_line_endings`, `lex_sysml_tokens`, `parse_lossless_sysml_cst`, `verify_lossless_cst_invariants`, `evaluate_hir_early_cutoff`, `apply_formatting_preserving_splices`, and `resolve_polyglot_backend` with 100% typed CST traversal and zero `.syntax().text().contains(...)`. */
    attribute Layer_2_Logic_State_Management : String = "Pure `logos` DFA lexing, `rowan` GreenNode building (`MAX_AST_DEPTH <= 64`), 4-layer `salsa` memoized early-cutoff queries, disjoint CST splicing, and typed polyglot CST lowering";
    doc /* `render_ariadne_diagnostic` plus `compile-sysml`, `ingest-sysml`, and `verify-baseline check-blueprint` CLI actuators emitting ANSI-free deterministic `ariadne` / NDJSON diagnostics (`DEAP-E1000..DEAP-E1005`) and bijective `.sysml` + `.digest.json` artifacts across macOS, Linux, and Windows. */
    attribute Layer_3_Presentation_Actuator_Interface_Binding : String = "Deterministic `ariadne` byte-span diagnostic renderer, `--message-format=json` NDJSON CLI stream, and atomic `.sysml` / `.digest.json` workspace actuators";
    doc /* [unit: `levels`] */
    attribute max_ast_depth : Real;
    doc /* [unit: `bytes`] */
    attribute untouched_trivia_mutation_bytes : Real;
    doc /* [unit: `files`] */
    attribute stage_fixed_point_mismatch_files : Real;
    doc /* [unit: `layers`] */
    attribute query_layer_count : Real;
    doc /* [unit: `queries`] */
    attribute trivia_edit_downstream_recomputations : Real;
    doc /* [unit: `frontends`] */
    attribute polyglot_frontend_count : Real;
    doc /* [unit: `crates`] */
    attribute rustc_private_feature_count : Real;
    port def BP_10_IncrementalCstPortDef {
        flow in source_buffer_in : String;
        flow out green_cst_out : String;
    }
    doc /* resolve_polyglot_backend(format) */
    action def resolve_polyglot_backend;
    doc /* normalize_line_endings(raw) */
    action def normalize_line_endings;
    doc /* parse_lossless_sysml_cst(input) */
    action def parse_lossless_sysml_cst;
    doc /* verify_lossless_cst_invariants(source, parsed) */
    action def verify_lossless_cst_invariants;
    doc /* evaluate_hir_early_cutoff(previous_hir, current_hir) */
    action def evaluate_hir_early_cutoff;
    doc /* apply_formatting_preserving_splices(file_id, source, patches) */
    action def apply_formatting_preserving_splices;
    doc /* render_ariadne_diagnostic(input, diag) */
    action def render_ariadne_diagnostic;
    doc /* [unit: `levels`] */
    assert constraint C_INC_01 {
        max_ast_depth >= 1 and max_ast_depth <= 64;
    }
    doc /* [unit: `bytes`] */
    assert constraint C_INC_02 {
        untouched_trivia_mutation_bytes >= 0 and untouched_trivia_mutation_bytes <= 0;
    }
    doc /* [unit: `files`] */
    assert constraint C_INC_03 {
        stage_fixed_point_mismatch_files >= 0 and stage_fixed_point_mismatch_files <= 0;
    }
    doc /* [unit: `layers`] */
    assert constraint C_INC_04 {
        query_layer_count >= 4 and query_layer_count <= 4;
    }
    doc /* [unit: `queries`] */
    assert constraint C_INC_05 {
        trivia_edit_downstream_recomputations >= 0 and trivia_edit_downstream_recomputations <= 0;
    }
    doc /* [unit: `frontends`] */
    assert constraint C_INC_06 {
        polyglot_frontend_count >= 6 and polyglot_frontend_count <= 6;
    }
    doc /* [unit: `crates`] */
    assert constraint C_INC_07 {
        rustc_private_feature_count >= 0 and rustc_private_feature_count <= 0;
    }
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0610_BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine {
        id = "REQ-0610";
        text = "IncrementalLosslessSysmlAndPolyglotCompilerEngine (REQ-0610 / BP-10 / DEAP-BP-DET-10, UUIDv5 6ba7b810-9dad-11d1-80b4-00c04fd430ca, Class P linear DFA lexing, lossless Red/Green syntax-tree construction, ariadne deterministic byte-span rendering, and O(delta) early-cutoff incremental query evaluation, governed by OMG SysML v2.0, KerML 1.0, DO-178C Level A, and DO-330 TQL-4) SHALL enforce Contract A (full normative OMG SysML v2.0 + KerML 1.0 textual notation coverage across all 6 clauses with full Definition & Usage symmetry and zero 16-construct subset caps: package/library package, import, alias, comment/doc/rep, attribute def/attribute, enum def/enum, item def/item, part def/part, port def/port with ~ conjugation and in/out/inout, connection def/connection, interface def/interface, flow def/flow/succession, allocation def/allocation/allocate, occurrence def/occurrence/individual/snapshot/timeslice, action def/action with perform/accept/send/assign/if/while/for/fork/join/merge/decide/first ... then, state def/state/exhibit with entry/do/exit/transition, calc def/calc, constraint def/constraint with assert/assume/require, requirement def/requirement with subject/actor/stakeholder/framed concern/satisfy, concern def/concern, use case def/use case with include use case, analysis def/analysis with objective, verification def/verification with verify, view def/view, viewpoint def/viewpoint, rendering def/rendering with expose/filter, metadata def/metadata/@/# user-defined keyword extensions supporting #hazard/#risk/#capability/#rt_loop/#dal/#fls via standard OMG SysML v2 metadata keywords while retaining backward-compatible parsing of legacy hazard def/risk def/capability def, variation/variant, and full KerML specialization & feature operators : defined by, :> subsets, :>> redefines, ::> references, ~ conjugation, ref, abstract, readonly, derived, end, multiplicity [lo..hi] / {ordered, nonunique}, full KerML operator expressions, and ISQ/SI/ScalarValues standard library imports) and Contract B (logos + rowan lossless Red/Green CST deap_core::sysml_cst and deap_core::dart_cst, 4-layer salsa incremental query database compile_sysml::salsa_db, ariadne source-span diagnostics, ra_ap_syntax, serde_json, pulldown-cmark, and ingest_sysml::cst::{idl_cst, yang_cst, markdown_cst, openapi_ast} inside < 380-SLOC submodules while retiring -6,600+ SLOC of legacy hand-rolled parsers).";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_lossless_logos_and_rowan_concrete_syntax_tree_full_fidelity_roundtrip_and_cross_platform_crlf_normalization : String = "- **Given**: A .sysml and .kerml fixture encoded with both LF and CRLF line endings exercising full normative OMG SysML v2.0 + KerML 1.0 Definition and Usage symmetry (package/library package, import, alias, comment/doc/rep, attribute def/attribute, enum def/enum, item def/item, part def/part, port def/port with ~ and in/out/inout, connection def/connection, interface def/interface, flow def/flow/succession, allocation def/allocation/allocate, occurrence def/occurrence/individual/snapshot/timeslice, action def/action, state def/state/exhibit, calc def/calc, constraint def/constraint, requirement def/requirement, concern def/concern, use case def/use case, analysis def/analysis, verification def/verification, view def/view, viewpoint def/viewpoint, rendering def/rendering, metadata def/metadata/@/# for #hazard/#risk/#capability + backward-compatible hazard def/risk def/capability def, variation/variant, and KerML : defined by, :> subsets, :>> redefines, ::> references, ~ conjugation, ref, abstract, readonly, derived, end, [lo..hi] / {ordered, nonunique}, KerML expressions, ISQ/SI/ScalarValues). - **When**: deap_core::sysml_cst normalizes CRLF to LF via normalize_line_endings, lexes via logos, and constructs the SysmlLanguage Red/Green CST via rowan::GreenNodeBuilder. - **Then**: Root SysmlSyntaxNode::text() equals the LF-normalized UTF-8 buffer byte-for-byte with zero 16-construct caps.";
        attribute ac_02_resilient_error_recovery_parsing_max_ast_depth_guard_and_deterministic_ariadne_byte_span_diagnostics : String = "- **Given**: Two negative .sysml fixtures: Fixture A containing an unclosed part def attribute missing ; followed by a valid requirement def block, and Fixture B containing 65 nested package / part blocks exceeding MAX_AST_DEPTH = 64 (**C-INC-01**). - **When**: parse_lossless_sysml_cst and render_ariadne_diagnostic process Fixture A and Fixture B. - **Then**: Neither parse panics or overflows the call stack; Fixture A wraps the malformed attribute in SysmlSyntaxKind::ErrorNode, recovers cleanly to parse the subsequent requirement def, and emits both an ANSI-free ariadne byte-span report and a.";
        attribute ac_03_formatting_preserving_bidirectional_cst_splicing_across_forward_and_reverse_sync : String = "- **Given**: An existing .sysml schema containing human-authored architectural comments, custom indentation, and three part def subsystems alongside synchronized docs/features/ and docs/epics/ Markdown specifications, where only one part def port signature is updated during reverse sync, plus a negative splice request containing overlapping Utf8TextRange intervals. - **When**: apply_formatting_preserving_splices executes forward/reverse splice plans and evaluates the negative overlapping splice plan. - **Then**: Only the modified declaration's CST span is spliced while 100% of untouched.";
        attribute ac_04_salsa_early_cutoff_durability_preventing_downstream_recomputations_on_trivia_edits : String = "- **Given**: A multi-package .sysml workspace loaded into compile_sysml::salsa_db across all 4 query layers (SourceDatabase, CstDatabase, HirDatabase, SafetyVerificationDatabase) with query execution counters attached to CST, HIR, semantic validation, STPA/FMECA, and Kani queries. - **When**: Revision 2 appends comments and reformats whitespace inside one .sysml file without altering semantic declarations, followed by Revision 3 modifying a port def [min..max] bound inside that file. - **Then**: On Revision 2, evaluate_hir_early_cutoff returns EarlyCutoffDecision::CutoffPreservedDownstream.";
        attribute ac_05_fail_closed_salsa_cycle_detection_and_net_negative_legacy_cst_parser_retirement : String = "- **Given**: A negative multi-file .sysml fixture introducing a cyclic import / cyclic part def :> BasePart specialization loop across two packages, alongside workspace SLOC and module-size audits. - **When**: compile_sysml::salsa_db evaluates scoped symbol resolution and verify-baseline audits legacy cst_parser deletion targets. - **Then**: Cycle recovery intercepts the cycle deterministically without infinite recursion or thread deadlock and fails closed with DEAP-E1004 (ariadne + NDJSON), while legacy hand-rolled parsers in ./crates/deap-core/src/sysml_ast/mod.rs (-2,400+ SLOC),.";
        attribute ac_06_cohesive_typed_polyglot_cst_ast_lowering_with_zero_rustc_private_and_exact_byte_span_diagnostics : String = "- **Given**: Six valid polyglot specification fixtures (Rust .rs via ra_ap_syntax, Dart .dart via deap_core::dart_cst, OpenAPI/JSON-Schema .json via typed serde_json AST, OMG IDL .idl, IETF YANG .yang, and Markdown .md via logos+rowan CSTs in ingest_sysml::polyglot_cst) alongside malformed counterparts at known byte offsets. - **When**: ingest_sysml::polyglot_cst resolves each backend via resolve_polyglot_backend and lowers each fixture into compile_sysml::salsa_db. - **Then**: All 6 valid fixtures lower deterministically into canonical HirPackageSummary and HirSourceMap records.";
        attribute ac_07_twelve_gate_blueprint_verification_and_baseline_cleanliness_gate_conformance : String = "- **Given**: ./docs/designs/blueprint-10-rowan-salsa-incremental-sysml-compiler.md residing in the upstream compiler repository alongside workspace governance rules. - **When**: ./target/release/verify-baseline check-blueprint ./docs/designs/blueprint-10-rowan-salsa-incremental-sysml-compiler.md and ./target/release/verify-baseline . --no-domain are executed. - **Then**: All 12 mechanical blueprint verification gates and all upstream workspace baseline checks pass with exit code 0, atomically persisting ./docs/designs/blueprint-10-rowan-salsa-incremental-sysml-compiler.sysml and.";
        doc /* \begin{aligned} \forall S \in \Sigma^*, \quad \bigoplus_{i=1}^{k} \text{bytes}(t_i) &= S \land \sum_{i=1}^{k} \lvert \text{range}(t_i) \rvert = \lvert S \rvert \\ \text{depth}(T_{\text{green}}(S)) &\le 64 \land \forall e \in \mathcal{D}_{\text{err}}(S), \quad \exists v \in V(T_{\text{green}}(S)) : \text{kind}(v) = \text{ErrorNode} \end{aligned} */
        constraint def Invariant_REQ_0610_SC_01;
        doc /* \begin{aligned} \forall i \in \{1, \dots, m - 1\}, \quad 0 \le a_i \le b_i \le a_{i+1} \le \lvert S \rvert &\land \Delta_{\text{trivia}}(\mathcal{U}(S, \mathcal{E}), S') = 0 \\ \text{Lower}_{\text{HIR}}(\text{Parse}_{\text{CST}}(\text{Emit}_{\text{canonical}}(H))) = H &\land \text{SHA-256}(\mathcal{T}_{\text{Stage 1}}) = \text{SHA-256}(\mathcal{T}_{\text{Stage 2}}) \end{aligned} */
        constraint def Invariant_REQ_0610_SC_02;
        doc /* \begin{aligned} H_{f^*}^{(r+1)} = H_{f^*}^{(r)} &\implies \lvert \text{Recomputed}(Q_4, r \to r+1) \rvert = 0 \\ T_{\text{inc}}(r \to r+1) &= O\left( \sum_{f \in \Delta_{\text{src}}} \lvert S_f \rvert + \sum_{q \in \text{Affected}(Q_3 \cup Q_4, \Delta_{\text{HIR}})} \text{cost}(q) \right) \end{aligned} */
        constraint def Invariant_REQ_0610_SC_03_Incremental_Complexity;
        doc /* \begin{aligned} \forall p \in \mathcal{P}, \forall S \in \Sigma^*, \quad \text{SHA-256}(\Phi_p(S)) &= \text{const}(p, S) \land \forall d \in \mathcal{D}_{\text{span}}, \quad \text{count}(\text{encode}(d), \text{0x1B}) = 0 \end{aligned} */
        constraint def Invariant_REQ_0610_SC_04;
        require Invariant_REQ_0610_SC_01;
        require Invariant_REQ_0610_SC_02;
        require Invariant_REQ_0610_SC_03_Incremental_Complexity;
        require Invariant_REQ_0610_SC_04;
        derived from REQ_0603;
        derived from REQ_0607;
        derived from REQ_0608;
        derived from REQ_0609;
        verify by AC_01_Lossless_logos_and_rowan_Concrete_Syntax_Tree_Full_Fidelity_Roundtrip_and_Cross_Platform_CRLF_Normalization;
        verify by AC_02_Resilient_Error_Recovery_Parsing_MAX_AST_DEPTH_Guard_and_Deterministic_Ariadne_Byte_Span_Diagnostics;
        verify by AC_03_Formatting_Preserving_Bidirectional_CST_Splicing_Across_Forward_and_Reverse_Sync;
        verify by AC_04_Salsa_Early_Cutoff_Durability_Preventing_Downstream_Recomputations_on_Trivia_Edits;
        verify by AC_05_Fail_Closed_Salsa_Cycle_Detection_and_Net_Negative_Legacy_Cst_Parser_Retirement;
        verify by AC_06_Cohesive_Typed_Polyglot_CST_AST_Lowering_with_Zero_rustc_private_and_Exact_Byte_Span_Diagnostics;
        verify by AC_07_Twelve_Gate_Blueprint_Verification_and_Baseline_Cleanliness_Gate_Conformance;
        satisfy by FormattingPreservingSyncSplicer;
    }
    doc /* Extracted from Mermaid stateDiagram specification. */
    state def BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine_LifecycleState {
        state Idle;
        state NormalizingSourceRevision;
        state LexingAndBuildingRowanCst;
        state EmittingAriadneAndJsonDiagnostics;
        state LoweringTriviaErasedHir;
        state EarlyCutoffShortCircuited;
        state EvaluatingSafetyAndKaniQueries;
        state SplicingFormattingPreservingSync;
        state VerifiedIncrementalClean;
        transition Initial_to_Idle first Initial then Idle;
        transition Idle_to_NormalizingSourceRevision_on_normalize_line_endings first Idle accept normalize_line_endings then NormalizingSourceRevision;
        transition NormalizingSourceRevision_to_LexingAndBuildingRowanCst_on_parse_lossless_sysml_cst first NormalizingSourceRevision accept parse_lossless_sysml_cst then LexingAndBuildingRowanCst;
        transition LexingAndBuildingRowanCst_to_EmittingAriadneAndJsonDiagnostics_on_rejectMaxAstDepthOverflowE1002 first LexingAndBuildingRowanCst accept rejectMaxAstDepthOverflowE1002 then EmittingAriadneAndJsonDiagnostics;
        transition LexingAndBuildingRowanCst_to_LoweringTriviaErasedHir_on_verify_lossless_cst_invariants first LexingAndBuildingRowanCst accept verify_lossless_cst_invariants then LoweringTriviaErasedHir;
        transition LoweringTriviaErasedHir_to_EmittingAriadneAndJsonDiagnostics_on_rejectCyclicSalsaQueryE1004 first LoweringTriviaErasedHir accept rejectCyclicSalsaQueryE1004 then EmittingAriadneAndJsonDiagnostics;
        transition LoweringTriviaErasedHir_to_EarlyCutoffShortCircuited_on_cutoffPreservedDownstream first LoweringTriviaErasedHir accept cutoffPreservedDownstream then EarlyCutoffShortCircuited;
        transition LoweringTriviaErasedHir_to_EvaluatingSafetyAndKaniQueries_on_recomputeDownstreamDependents first LoweringTriviaErasedHir accept recomputeDownstreamDependents then EvaluatingSafetyAndKaniQueries;
        transition EvaluatingSafetyAndKaniQueries_to_SplicingFormattingPreservingSync_on_apply_formatting_preserving_splices first EvaluatingSafetyAndKaniQueries accept apply_formatting_preserving_splices then SplicingFormattingPreservingSync;
        transition SplicingFormattingPreservingSync_to_EmittingAriadneAndJsonDiagnostics_on_rejectOverlappingSpliceE1003 first SplicingFormattingPreservingSync accept rejectOverlappingSpliceE1003 then EmittingAriadneAndJsonDiagnostics;
        transition SplicingFormattingPreservingSync_to_VerifiedIncrementalClean_on_bijectiveAndStage2FixedPointVerified first SplicingFormattingPreservingSync accept bijectiveAndStage2FixedPointVerified then VerifiedIncrementalClean;
        transition EarlyCutoffShortCircuited_to_VerifiedIncrementalClean_on_reuseMemoizedSafetyVerificationReceipt first EarlyCutoffShortCircuited accept reuseMemoizedSafetyVerificationReceipt then VerifiedIncrementalClean;
        transition EmittingAriadneAndJsonDiagnostics_to_Terminal first EmittingAriadneAndJsonDiagnostics then Terminal;
        transition VerifiedIncrementalClean_to_Terminal first VerifiedIncrementalClean then Terminal;
    }
    connection def BP_10_PolyglotToSalsaDbConnection {
        connect TypedPolyglotIngestionFrontend.lowered_hir_out to SalsaIncrementalCompilerDatabase.source_revision_in;
    }
    doc /* - **Given**: A .sysml and .kerml fixture encoded with both LF and CRLF line endings exercising full normative OMG SysML v2.0 + KerML 1.0 Definition and Usage symmetry (package/library package, import, alias, comment/doc/rep, attribute def/attribute, enum def/enum, item def/item, part def/part, port def/port with ~ and in/out/inout, connection def/connection, interface def/interface, flow def/flow/succession, allocation def/allocation/allocate, occurrence def/occurrence/individual/snapshot/timeslice, action def/action, state def/state/exhibit, calc def/calc, constraint def/constraint, requirement def/requirement, concern def/concern, use case def/use case, analysis def/analysis, verification def/verification, view def/view, viewpoint def/viewpoint, rendering def/rendering, metadata def/metadata/@/# for #hazard/#risk/#capability + backward-compatible hazard def/risk def/capability def, variation/variant, and KerML : defined by, :> subsets, :>> redefines, ::> references, ~ conjugation, ref, abstract, readonly, derived, end, [lo..hi] / {ordered, nonunique}, KerML expressions, ISQ/SI/ScalarValues). - **When**: deap_core::sysml_cst normalizes CRLF to LF via normalize_line_endings, lexes via logos, and constructs the SysmlLanguage Red/Green CST via rowan::GreenNodeBuilder. - **Then**: Root SysmlSyntaxNode::text() equals the LF-normalized UTF-8 buffer byte-for-byte with zero 16-construct caps. */
    use case def UC_AC_01_Lossless_logos_and_rowan_Concrete_Syntax_Tree_Full_Fidelity_Roundtrip_and_Cross_Platform_CRLF_Normalization {
        subject FormattingPreservingSyncSplicer;
        objective "Lossless logos and rowan Concrete Syntax Tree Full Fidelity Roundtrip and Cross Platform CRLF Normalization";
    }
    doc /* - **Given**: Two negative .sysml fixtures: Fixture A containing an unclosed part def attribute missing ; followed by a valid requirement def block, and Fixture B containing 65 nested package / part blocks exceeding MAX_AST_DEPTH = 64 (**C-INC-01**). - **When**: parse_lossless_sysml_cst and render_ariadne_diagnostic process Fixture A and Fixture B. - **Then**: Neither parse panics or overflows the call stack; Fixture A wraps the malformed attribute in SysmlSyntaxKind::ErrorNode, recovers cleanly to parse the subsequent requirement def, and emits both an ANSI-free ariadne byte-span report and a. */
    use case def UC_AC_02_Resilient_Error_Recovery_Parsing_MAX_AST_DEPTH_Guard_and_Deterministic_Ariadne_Byte_Span_Diagnostics {
        subject FormattingPreservingSyncSplicer;
        objective "Resilient Error Recovery Parsing MAX AST DEPTH Guard and Deterministic Ariadne Byte Span Diagnostics";
    }
    doc /* - **Given**: An existing .sysml schema containing human-authored architectural comments, custom indentation, and three part def subsystems alongside synchronized docs/features/ and docs/epics/ Markdown specifications, where only one part def port signature is updated during reverse sync, plus a negative splice request containing overlapping Utf8TextRange intervals. - **When**: apply_formatting_preserving_splices executes forward/reverse splice plans and evaluates the negative overlapping splice plan. - **Then**: Only the modified declaration's CST span is spliced while 100% of untouched. */
    use case def UC_AC_03_Formatting_Preserving_Bidirectional_CST_Splicing_Across_Forward_and_Reverse_Sync {
        subject FormattingPreservingSyncSplicer;
        objective "Formatting Preserving Bidirectional CST Splicing Across Forward and Reverse Sync";
    }
    doc /* - **Given**: A multi-package .sysml workspace loaded into compile_sysml::salsa_db across all 4 query layers (SourceDatabase, CstDatabase, HirDatabase, SafetyVerificationDatabase) with query execution counters attached to CST, HIR, semantic validation, STPA/FMECA, and Kani queries. - **When**: Revision 2 appends comments and reformats whitespace inside one .sysml file without altering semantic declarations, followed by Revision 3 modifying a port def [min..max] bound inside that file. - **Then**: On Revision 2, evaluate_hir_early_cutoff returns EarlyCutoffDecision::CutoffPreservedDownstream. */
    use case def UC_AC_04_Salsa_Early_Cutoff_Durability_Preventing_Downstream_Recomputations_on_Trivia_Edits {
        subject FormattingPreservingSyncSplicer;
        objective "Salsa Early Cutoff Durability Preventing Downstream Recomputations on Trivia Edits";
    }
    doc /* - **Given**: A negative multi-file .sysml fixture introducing a cyclic import / cyclic part def :> BasePart specialization loop across two packages, alongside workspace SLOC and module-size audits. - **When**: compile_sysml::salsa_db evaluates scoped symbol resolution and verify-baseline audits legacy cst_parser deletion targets. - **Then**: Cycle recovery intercepts the cycle deterministically without infinite recursion or thread deadlock and fails closed with DEAP-E1004 (ariadne + NDJSON), while legacy hand-rolled parsers in ./crates/deap-core/src/sysml_ast/mod.rs (-2,400+ SLOC),. */
    use case def UC_AC_05_Fail_Closed_Salsa_Cycle_Detection_and_Net_Negative_Legacy_Cst_Parser_Retirement {
        subject FormattingPreservingSyncSplicer;
        objective "Fail Closed Salsa Cycle Detection and Net Negative Legacy CST Parser Retirement";
    }
    doc /* - **Given**: Six valid polyglot specification fixtures (Rust .rs via ra_ap_syntax, Dart .dart via deap_core::dart_cst, OpenAPI/JSON-Schema .json via typed serde_json AST, OMG IDL .idl, IETF YANG .yang, and Markdown .md via logos+rowan CSTs in ingest_sysml::polyglot_cst) alongside malformed counterparts at known byte offsets. - **When**: ingest_sysml::polyglot_cst resolves each backend via resolve_polyglot_backend and lowers each fixture into compile_sysml::salsa_db. - **Then**: All 6 valid fixtures lower deterministically into canonical HirPackageSummary and HirSourceMap records. */
    use case def UC_AC_06_Cohesive_Typed_Polyglot_CST_AST_Lowering_with_Zero_rustc_private_and_Exact_Byte_Span_Diagnostics {
        subject FormattingPreservingSyncSplicer;
        objective "Cohesive Typed Polyglot CST AST Lowering with Zero rustc private and Exact Byte Span Diagnostics";
    }
    doc /* - **Given**: ./docs/designs/blueprint-10-rowan-salsa-incremental-sysml-compiler.md residing in the upstream compiler repository alongside workspace governance rules. - **When**: ./target/release/verify-baseline check-blueprint ./docs/designs/blueprint-10-rowan-salsa-incremental-sysml-compiler.md and ./target/release/verify-baseline . --no-domain are executed. - **Then**: All 12 mechanical blueprint verification gates and all upstream workspace baseline checks pass with exit code 0, atomically persisting ./docs/designs/blueprint-10-rowan-salsa-incremental-sysml-compiler.sysml and. */
    use case def UC_AC_07_Twelve_Gate_Blueprint_Verification_and_Baseline_Cleanliness_Gate_Conformance {
        subject FormattingPreservingSyncSplicer;
        objective "Twelve Gate Blueprint Verification and Baseline Cleanliness Gate Conformance";
    }
    test case def TC_REQ_0610_Incremental_Sysml_Engine {
        verify requirement REQ_0610;
    }
    test case def TC_REQ_0610_Blueprint_Conformance {
        verify requirement REQ_0610_BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine;
    }
    test case def Test_resolve_polyglot_backend {
        verify requirement REQ_0610;
    }
    test case def Test_normalize_line_endings {
        verify requirement REQ_0610;
    }
    test case def Test_parse_lossless_sysml_cst {
        verify requirement REQ_0610;
    }
    test case def Test_verify_lossless_cst_invariants {
        verify requirement REQ_0610;
    }
    test case def Test_evaluate_hir_early_cutoff {
        verify requirement REQ_0610;
    }
    test case def Test_apply_formatting_preserving_splices {
        verify requirement REQ_0610;
    }
    test case def Test_render_ariadne_diagnostic {
        verify requirement REQ_0610;
    }
    doc /* Extracted from Mermaid sequenceDiagram specification. */
    interaction def BP_10_Incremental_Lossless_SysML_v2_and_Polyglot_Compiler_Engine_SequenceInteraction {
        lifeline TypedPolyglotIngestionFrontend : TypedPolyglotIngestionFrontend;
        lifeline SalsaIncrementalCompilerDatabase : SalsaIncrementalCompilerDatabase;
        lifeline LogosRowanSysmlCstEngine : LogosRowanSysmlCstEngine;
        lifeline FormattingPreservingSyncSplicer : FormattingPreservingSyncSplicer;
        message resolve_polyglot_backend from TypedPolyglotIngestionFrontend to TypedPolyglotIngestionFrontend;
        message normalize_line_endings from SalsaIncrementalCompilerDatabase to LogosRowanSysmlCstEngine;
        message reply_String from LogosRowanSysmlCstEngine to SalsaIncrementalCompilerDatabase;
        message parse_lossless_sysml_cst from SalsaIncrementalCompilerDatabase to LogosRowanSysmlCstEngine;
        message reply_CstParseReceipt from LogosRowanSysmlCstEngine to SalsaIncrementalCompilerDatabase;
        message verify_lossless_cst_invariants from SalsaIncrementalCompilerDatabase to LogosRowanSysmlCstEngine;
        message reply_Ok from LogosRowanSysmlCstEngine to SalsaIncrementalCompilerDatabase;
        message evaluate_hir_early_cutoff from SalsaIncrementalCompilerDatabase to SalsaIncrementalCompilerDatabase;
        message apply_formatting_preserving_splices from FormattingPreservingSyncSplicer to FormattingPreservingSyncSplicer;
        message render_ariadne_diagnostic from SalsaIncrementalCompilerDatabase to SalsaIncrementalCompilerDatabase;
    }
    doc /* Permitted Stage-1 versus Stage-2 SHA-256 digest mismatch count across bijective (`BP-03`) and self-hosted modular codegen (`E0705`). [unit: files] */
    part def FormattingPreservingSyncSplicer {
        attribute stage_fixed_point_mismatch_files : Integer = 0;
        doc /* Permitted byte mutations to comments and whitespace outside modified declaration `Utf8TextRange` intervals during incremental sync. [unit: bytes] */
        attribute untouched_trivia_mutation_bytes : Integer = 0;
        doc /* Enforces pairwise-disjoint UTF-8 boundary verification and post-splice CST re-parse checks in `deap_core::sysml_cst::printer` before atomic commit. [unit: flag] */
        attribute atomic_disjoint_splice : Boolean = true;
        doc /* Receives forward (`SysML` to `Markdown`/ICD/WBS) or reverse (`Markdown` to `SysML`) structural HIR diff descriptors. [protocol: BidirectionalHirDelta] */
        port in sync_delta_in : String {
            flow in sync_delta_in_flow : String;
        }
        doc /* Commits formatting-preserving `rowan` CST child splices and disjoint byte patches atomically to workspace storage. [protocol: LosslessSplicedSysmlAndMarkdown] */
        port out spliced_artifact_out : String {
            flow out spliced_artifact_out_flow : String;
        }
        doc /* Emits fail-closed `ariadne` and NDJSON `DEAP-E1003` diagnostics on overlapping splice intervals or post-splice syntax regressions. [protocol: AriadneAndJsonDiagnosticStream] */
        port out splice_diag_out : String {
            flow out splice_diag_out_flow : String;
        }
    }
    doc /* Hard structural nesting ceiling across nested `package`, `part`, `port`, `action`, `state`, and expression CST blocks in `deap_core::sysml_cst::parser`. [unit: levels] */
    part def LogosRowanSysmlCstEngine {
        attribute max_ast_depth : Integer = 64;
        doc /* Requires root `rowan::GreenNode` byte concatenation to match the LF-normalized UTF-8 `.sysml` buffer byte-for-byte including comments and whitespace. [unit: flag] */
        attribute lossless_trivia_preserved : Boolean = true;
        doc /* Wraps malformed token spans in `ErrorNode` subtrees and continues parsing at synchronization tokens while emitting `ariadne` / JSON `DEAP-E1001` diagnostics. [unit: flag] */
        attribute resilient_error_recovery : Boolean = true;
        doc /* Canonical deterministic blueprint component identifier (`DEAP-BP-DET-10`). [unit: uuidv5] */
        attribute deterministic_uuid : String = "6ba7b810-9dad-11d1-80b4-00c04fd430ca";
        doc /* Receives CRLF-to-LF normalized UTF-8 `.sysml` or `.kerml` source text via `camino::Utf8Path` from `SourceDatabase` without pre-lexer sigil rewriting. [protocol: Utf8SysmlSourceBuffer] */
        port in sysml_source_in : String {
            flow in sysml_source_in_flow : String;
        }
        doc /* Emits immutable, thread-safe `rowan::GreenNode` (`rowan` pinned to `ra_ap_syntax`) and typed `SysmlSyntaxNode` projection. [protocol: RowanGreenSysmlTree] */
        port out green_cst_out : String {
            flow out green_cst_out_flow : String;
        }
        doc /* Streams deterministic `ariadne` byte-span reports and single-line `DEAP-E1001` / `DEAP-E1002` NDJSON diagnostics. [protocol: AriadneAndJsonDiagnosticStream] */
        port out syntax_diag_out : String {
            flow out syntax_diag_out_flow : String;
        }
    }
    doc /* Mandatory count of stratified `salsa` query groups in `compile_sysml::salsa_db` (`SourceDatabase`, `CstDatabase`, `HirDatabase`, `SafetyVerificationDatabase`). [unit: layers] */
    part def SalsaIncrementalCompilerDatabase {
        attribute query_layer_count : Integer = 4;
        doc /* Required count of downstream STPA, Kani, and baseline verification query recomputations when edits alter only non-semantic trivia. [unit: queries] */
        attribute trivia_edit_downstream_recomputations : Integer = 0;
        doc /* Mandates `BTreeMap` and `BTreeSet` deterministic ordering and forbids filesystem or clock side effects inside derived `salsa` queries. [unit: flag] */
        attribute pure_deterministic_btreemap : Boolean = true;
        doc /* Intercepts cyclic `salsa` dependency chains across imports or specializations and emits deterministic `DEAP-E1004` `ariadne` + JSON diagnostics. [unit: flag] */
        attribute cycle_recovery_fail_closed : Boolean = true;
        doc /* Receives `FileId` buffer updates, `camino::Utf8PathBuf` paths, `DurabilityTier` (`Low`, `Medium`, `High`), and LF-normalized SHA-256 input digests. [protocol: SalsaSourceInputRevision] */
        port in source_revision_in : String {
            flow in source_revision_in_flow : String;
        }
        doc /* Emits trivia-erased `HirPackageSummary` semantic graphs decoupled from `DefId` to `Utf8TextRange` `HirSourceMap` tables for early cutoff. [protocol: InternedHirPackageAndSourceMap] */
        port out hir_model_out : String {
            flow out hir_model_out_flow : String;
        }
        doc /* Emits memoized STPA UCA/FMECA matrices, Kani CBMC proof harnesses, `ariadne` diagnostics, and `verify-baseline` parity receipts. [protocol: MemoizedStpaKaniBaselineReceipt] */
        port out safety_verification_out : String {
            flow out safety_verification_out_flow : String;
        }
    }
    doc /* Count of typed CST/AST frontends in `ingest_sysml::polyglot_cst` (`ra_ap_syntax` Rust, `deap_core::dart_cst` Dart, typed `serde_json` OpenAPI/JSON-Schema, `logos`+`rowan` OMG IDL, IETF YANG, Markdown). [unit: frontends] */
    part def TypedPolyglotIngestionFrontend {
        attribute polyglot_frontend_count : Integer = 6;
        doc /* Enforces `ZERO #![feature(rustc_private)]` occurrences across all workspace crates under `./crates/ *`. [unit: crates] */
        attribute rustc_private_feature_count : Integer = 0;
        doc /* Emits deterministic `ariadne` byte-span reports and `--message-format=json` diagnostics (`DEAP-E1005`) across all polyglot formats. [unit: flag] */
        attribute json_and_ariadne_span_diagnostics : Boolean = true;
        doc /* Enforces linear-time `logos` DFA / `rowan` CST and typed `serde_json` AST parsing with zero hand-rolled line lexers or NFA backtracking (`Check 34`). [unit: flag] */
        attribute zero_backtracking_lexers : Boolean = true;
        doc /* Receives LF-normalized Rust (`.rs`), Dart (`.dart`), JSON-Schema/OpenAPI (`.json`), OMG IDL (`.idl`), IETF YANG (`.yang`), or Markdown (`.md`) buffers. [protocol: PolyglotSpecBuffer] */
        port in polyglot_source_in : String {
            flow in polyglot_source_in_flow : String;
        }
        doc /* Lowers typed polyglot syntax trees directly into interned `compile_sysml::salsa_db` `HirPackageSummary` and `HirSourceMap` records. [protocol: CanonicalHirDatabaseInput] */
        port out lowered_hir_out : String {
            flow out lowered_hir_out_flow : String;
        }
        doc /* Streams deterministic `ariadne` byte-span reports and single-line `DEAP-E1005` NDJSON diagnostics on syntax or schema-pointer errors. [protocol: AriadneAndJsonDiagnosticStream] */
        port out ingest_diag_out : String {
            flow out ingest_diag_out_flow : String;
        }
    }
}

doc /* Subsystem package for BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture */
package BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture {
    attribute Blueprint_ID : String = "BP-11";
    attribute Requirement_ID : String = "REQ-0611";
    attribute Title : String = "Ferrocene Qualified Toolchain, FLS Traceability, Structural Coverage, and 3-Ring Rust/Flutter DO-178C & DO-330 Certification Architecture";
    attribute Status : String = "Draft";
    attribute Target_Crates : String = "crates/deap-core, crates/compile-sysml, crates/verify-baseline";
    attribute Tracked_Issues : String = "#742";
    attribute Generation_Mode : String = "subagent";
    attribute Specification_Source : String = "docs/architecture/blueprints/DEAP_MULTI_TOOLCHAIN_SYNTHESIS_ARCHITECTURE.md:1-30";
    doc /* [unit: `versions`] */
    attribute max_toolchain_version_drift : Real;
    doc /* [unit: `files`] */
    attribute stage_fixed_point_mismatch_files : Real;
    doc /* [unit: `items`] */
    attribute unmapped_fls_item_count : Real;
    doc /* [unit: `symbols`] */
    attribute uncertified_core_symbol_count : Real;
    doc /* [unit: `percent`] */
    attribute dal_a_mcdc_coverage_min_pct : Real;
    doc /* [unit: `rings`] */
    attribute assurance_ring_count : Real;
    doc /* [unit: `SLOC`] */
    attribute max_module_exec_sloc : Real;
    doc /* [unit: `probes`] */
    attribute min_toolchain_discovery_probes : Real;
    port def BP_11_FerroceneAssurancePortDef {
        flow in assurance_ast_in : String;
        flow out table_a7_receipt_out : String;
    }
    doc /* probe_criticalup_toolchain(config) */
    action def probe_criticalup_toolchain;
    doc /* verify_dual_toolchain_parity(spec) */
    action def verify_dual_toolchain_parity;
    doc /* synthesize_three_ring_workspace(spec) */
    action def synthesize_three_ring_workspace;
    doc /* verify_fls_traceability_chain(records, pinned_fls_catalog) */
    action def verify_fls_traceability_chain;
    doc /* verify_certified_core_and_table_a7_coverage(summary, certified_core_prefixes) */
    action def verify_certified_core_and_table_a7_coverage;
    doc /* verify_three_ring_edge_and_monitor_readback(from_ring, to_ring, envelope) */
    action def verify_three_ring_edge_and_monitor_readback;
    doc /* [unit: `versions`] */
    assert constraint C_CERT_01 {
        max_toolchain_version_drift >= 0 and max_toolchain_version_drift <= 0;
    }
    doc /* [unit: `files`] */
    assert constraint C_CERT_02 {
        stage_fixed_point_mismatch_files >= 0 and stage_fixed_point_mismatch_files <= 0;
    }
    doc /* [unit: `items`] */
    assert constraint C_CERT_03 {
        unmapped_fls_item_count >= 0 and unmapped_fls_item_count <= 0;
    }
    doc /* [unit: `symbols`] */
    assert constraint C_CERT_04 {
        uncertified_core_symbol_count >= 0 and uncertified_core_symbol_count <= 0;
    }
    doc /* [unit: `percent`] */
    assert constraint C_CERT_05 {
        dal_a_mcdc_coverage_min_pct >= 100 and dal_a_mcdc_coverage_min_pct <= 100;
    }
    doc /* [unit: `rings`] */
    assert constraint C_CERT_06 {
        assurance_ring_count >= 3 and assurance_ring_count <= 3;
    }
    doc /* [unit: `SLOC`] */
    assert constraint C_CERT_07 {
        max_module_exec_sloc >= 1 and max_module_exec_sloc <= 380;
    }
    doc /* [unit: `probes`] */
    assert constraint C_CERT_08 {
        min_toolchain_discovery_probes >= 1 and min_toolchain_discovery_probes <= 3;
    }
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0611_BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C {
        id = "REQ-0611";
        text = "FerroceneThreeRingCertificationArchitecture (REQ-0611 / BP-11 / DEAP-BP-DET-11, UUIDv5 6ba7b810-9dad-11d1-80b4-00c04fd430cb, Class P linear AST, ELF symbol-table, and LLVM coverage-profile verification complexity, governed by RTCA DO-178C DAL-A through DAL-E, RTCA DO-330 TQL-1 and TQL-5, RTCA DO-333, and ISO 26262 ASIL D) SHALL enforce Contract A (full 6-clause OMG SysML v2.0 + KerML 1.0 grammar coverage with zero 16-construct caps, #dal/#fls/#rt_loop/#hazard/#risk/#capability metadata def + backward-compatible hazard def/risk def/capability def) and Contract B (paired criticalup.toml and rust-toolchain.toml verification via toml_edit and cargo-platform, 5-link FLS paragraph traceability via ra_ap_syntax and syn + proc-macro2 + quote + prettyplease, certified core symbol-report gating, zero-overcount blanket / llvm-profdata MC/DC structural coverage via serde_json, and 3-ring safety_kernel / ffi_guard_bridge / app_flutter partitioning inside < 380-SLOC submodules).";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_dual_mode_criticalup_toml_and_rust_toolchain_toml_parity_cross_platform_discovery_and_do_330_fixed_point_qualification : String = "- **Given**: A workspace containing paired criticalup.toml (ferrocene-stable pinned to an exact qualified release and target triple set) and rust-toolchain.toml (pinned to the matching upstream rustc semantic version and target set) parsed via toml_edit and cargo-platform, alongside a negative fixture where rust-toolchain.toml drifts by one minor version or omits a qualified target triple. - **When**: DualModeFerroceneToolchainGate executes probe_criticalup_toolchain and verify_dual_toolchain_parity, evaluating Stage-1 versus Stage-2 self-hosted modular codegen SHA-256 digests (E0705) under.";
        attribute ac_02_end_to_end_fls_normative_paragraph_traceability_emission_and_fail_closed_orphan_detection : String = "- **Given**: A SysML v2 package declaring RequirementDef (REQ-*), STPA safety constraint (SC-*), Ring-1 PartDef / StateDef / ConstraintDef constructs, and Ring-2 FFI PortDef contracts compiled through FlsParagraphTraceabilitySynthesizer, plus a negative fixture containing a Ring-1 function missing its [FLS: fls_*] docstring tag and a Ring-2 unsafe block citing an unknown paragraph fls_nonexistent999. - **When**: crates/compile-sysml/src/ring_partition/ emits the annotated Rust units and crates/verify-baseline audits the 5-link FLS traceability chain against the pinned FLS paragraph catalog.";
        attribute ac_03_certified_core_library_subset_symbol_report_gate_rejecting_std_alloc_and_dynamic_formatting_symbols : String = "- **Given**: Two Ring-1 #![no_std] safety_kernel symbol-report fixtures: Fixture A linking exclusively certified core::option, core::result, core::num, and fixed-size array operations, and Fixture B inadvertently linking __rust_alloc, alloc::vec::Vec, or std::io::stdio::_print. - **When**: CertifiedCoreSubsetAndCoverageGate audits both symbol inventories against the certified core allowlist prefix catalog. - **Then**: Fixture A passes with uncertified_core_symbol_count == 0 (**C-CERT-04**), whereas Fixture B fails closed immediately with DEAP-E1103 enumerating every prohibited std, alloc, or.";
        attribute ac_04_zero_overcount_structural_and_mc_dc_coverage_verification_mapped_to_sysml_verification_cases : String = "- **Given**: A DAL-A Ring-1 SafetyKernel module exercised by blanket / llvm-profdata MC/DC condition-vector profiles linked to SysML TestCaseDef declarations (TEST-01 -> REQ-0611), alongside three negative fixtures: (1) a boolean decision a && (b || c) missing independent flip coverage for condition c (99% MC/DC), (2) an overcounted macro-duplicated source span, and (3) a coverage test lacking a SysML TEST-* -> REQ-* verified_by binding. - **When**: CertifiedCoreSubsetAndCoverageGate evaluates DO-178C Table A-7 coverage thresholds for DalLevel::DalA using typed serde_json schemas. - **Then**:.";
        attribute ac_05_schema_driven_3_ring_workspace_partitioning_and_ring_2_miri_guarded_ffi_precondition_enforcement : String = "- **Given**: A SysML v2 system model annotated with AssuranceLevel and @Layer stereotypes (Domain, Application, Presentation) and PortDef [min..max] ranges, plus two negative probes: Probe A introducing an illegal safety_kernel -> ffi_guard_bridge or safety_kernel -> app_flutter dependency edge, and Probe B injecting an out-of-range or NaN OperatorConsole payload across the Ring-2 FFI boundary. - **When**: ThreeRingAssurancePartitionCompiler synthesizes the 3-ring workspace (safety_kernel/, ffi_guard_bridge/, app_flutter/) and executes the Ring-2 boundary verification suite. - **Then**: The.";
        attribute ac_06_ring_1_challenge_response_command_interlock_and_telemetryframe_digest_readback_safetymonitor_against_ring_3_faults : String = "- **Given**: An active Ring-1 SafetyMonitor issuing sequenced TelemetryFrame envelopes (seq_t, FIPS 180-4 SHA-256 digest h_t) and two-step challenge nonces (eta_t) through Ring 2 to the untrusted Ring-3 Flutter OperatorConsole, tested against three Ring-3 fault injections: (1) corrupted display frame-digest readback (hat_h_t != h_t), (2) stale frame sequence counter (hat_seq_t < seq_t), and (3) spurious single-step critical ActuatorCommand lacking matching challenge nonce eta_t. - **When**: verify_three_ring_edge_and_monitor_readback evaluates the inbound confirmation and readback envelope.";
        attribute ac_07_cross_platform_execution_submodule_layout_conformance_and_twelve_gate_blueprint_verification_gate : String = "- **Given**: docs/designs/blueprint-11-ferrocene-do178c-do330-3ring-certification-architecture.md specifying dynamic cross-platform driver probing across macOS, Linux, and Windows (std::env::consts::{DLL_PREFIX, DLL_SUFFIX, EXE_SUFFIX}, camino::Utf8Path), modular submodules under < 380 SLOC (**C-CERT-07**), and the --require-formal-toolchains qualification flag, evaluated in an environment where a required driver is absent versus present. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-11-ferrocene-do178c-do330-3ring-certification-architecture.md and.";
        doc /* \begin{aligned} v(M_{\text{crit}}) = v(M_{\text{rust}}) &\land T_{\text{cert}} \subseteq T(M_{\text{crit}}) \cap T(M_{\text{rust}}) \\ \text{SHA-256}(\mathcal{A}_{\text{Stage 1}}^{\text{Ferrocene}}) = \text{SHA-256}(\mathcal{A}_{\text{Stage 2}}^{\text{Ferrocene}}) &\land \text{Codebase}(G_{\text{TQL1}}) \cap \text{RuleEngine}(V_{\text{TQL5}}) = \emptyset \end{aligned} */
        constraint def Invariant_REQ_0611_SC_01;
        doc /* \begin{aligned} \forall u \in \mathcal{I}_{R_1 \cup R_2}, \quad \emptyset \neq \pi_{\text{FLS}}(u) \subseteq \mathcal{P}_{\text{FLS}} &\implies N_{\text{unmapped}}(\mathcal{I}_{R_1 \cup R_2}) = 0 \\ \forall u \in \mathcal{I}_{R_1}, \quad \exists (r, s, p, k, c) \in \mathcal{T}_5 &: \text{item}(r, s, p, k, c) = u \end{aligned} */
        constraint def Invariant_REQ_0611_SC_02;
        doc /* \begin{aligned} \mathcal{S}_{\text{ELF}}(R_1) \subseteq \mathcal{S}_{\text{core}}^{\text{cert}} &\iff \lvert \mathcal{S}_{\text{ELF}}(R_1) \setminus \mathcal{S}_{\text{core}}^{\text{cert}} \rvert = 0 \\ \text{DAL}(R_1) = \text{DAL-A} \implies C_{\text{stmt}}(R_1) = 100\% \land C_{\text{dec}}(R_1) = 100\% &\land \forall d \in \mathcal{D}(R_1), \forall i \in \{1, \dots, m\}, \exists (v^+, v^-) \in \mathcal{V}_{\text{TEST}}(d) : \\ (v_i^+ \neq v_i^-) \land (\forall j \neq i, \; v_j^+ = v_j^-) &\land (d(v^+) \neq d(v^-)) \end{aligned} */
        constraint def Invariant_REQ_0611_SC_03;
        doc /* \begin{aligned} \Gamma_{R_2}(x, \sigma) = 1 &\iff \text{finite}(x) \land x \in [x_{\min}, x_{\max}] \land \text{Precond}_{\text{FSM}}(\sigma, x) \\ \text{Commit}_{R_1}(u_{\text{crit}}, \hat{\eta}_t, \hat{h}_t, \widehat{\text{seq}}_t) = 1 &\iff \Gamma_{R_2}(u_{\text{crit}}, \sigma_t) = 1 \land (\hat{\eta}_t = \eta_t) \land (\hat{h}_t = h_t) \land (\widehat{\text{seq}}_t = \text{seq}_t) \end{aligned} */
        constraint def Invariant_REQ_0611_SC_04;
        doc /* \begin{aligned} \text{ToolchainPass}(\mathcal{D}, Q_{\text{ci}}) &\iff \big( Q_{\text{ci}} \implies \forall d \in \mathcal{D}, \quad \text{Discovered}(d) = \text{true} \big) \\ &\land \forall m \in \mathcal{M}_{\text{cert}}, \quad \text{ExecSLOC}(m) \le 380 \land \text{SyntaxErrors}(m) = 0 \end{aligned} */
        constraint def Invariant_REQ_0611_SC_05;
        require Invariant_REQ_0611_SC_01;
        require Invariant_REQ_0611_SC_02;
        require Invariant_REQ_0611_SC_03;
        require Invariant_REQ_0611_SC_04;
        require Invariant_REQ_0611_SC_05;
        derived from REQ_0607;
        derived from REQ_0609;
        derived from REQ_0610;
        verify by AC_01_Dual_Mode_criticalup_toml_and_rust_toolchain_toml_Parity_Cross_Platform_Discovery_and_DO_330_Fixed_Point_Qualification;
        verify by AC_02_End_to_End_FLS_Normative_Paragraph_Traceability_Emission_and_Fail_Closed_Orphan_Detection;
        verify by AC_03_Certified_core_Library_Subset_Symbol_Report_Gate_Rejecting_std_alloc_and_Dynamic_Formatting_Symbols;
        verify by AC_04_Zero_Overcount_Structural_and_MC_DC_Coverage_Verification_Mapped_to_SysML_Verification_Cases;
        verify by AC_05_Schema_Driven_3_Ring_Workspace_Partitioning_and_Ring_2_Miri_Guarded_FFI_Precondition_Enforcement;
        verify by AC_06_Ring_1_Challenge_Response_Command_Interlock_and_TelemetryFrame_Digest_Readback_SafetyMonitor_Against_Ring_3_Faults;
        verify by AC_07_Cross_Platform_Execution_Submodule_Layout_Conformance_and_Twelve_Gate_Blueprint_Verification_Gate;
        satisfy by CertifiedCoreSubsetAndCoverageGate;
    }
    doc /* Extracted from Mermaid stateDiagram specification. */
    state def BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture_LifecycleState {
        state Idle;
        state ProbingCrossPlatformToolchain;
        state RejectedFailClosed;
        state VerifyingDualToolchainParity;
        state SynthesizingThreeRingPartitions;
        state EmbeddingFlsTraceability;
        state AuditingCoreSubsetAndMcdc;
        state VerifyingRing1SafetyMonitor;
        state CertifiedPackageReady;
        transition Initial_to_Idle first Initial then Idle;
        transition Idle_to_ProbingCrossPlatformToolchain_on_probeToolchainPath first Idle accept probeToolchainPath then ProbingCrossPlatformToolchain;
        transition ProbingCrossPlatformToolchain_to_RejectedFailClosed_on_rejectMissingToolchainE1101 first ProbingCrossPlatformToolchain accept rejectMissingToolchainE1101 then RejectedFailClosed;
        transition ProbingCrossPlatformToolchain_to_VerifyingDualToolchainParity_on_toolchainDiscovered first ProbingCrossPlatformToolchain accept toolchainDiscovered then VerifyingDualToolchainParity;
        transition VerifyingDualToolchainParity_to_RejectedFailClosed_on_rejectToolchainDriftE1101 first VerifyingDualToolchainParity accept rejectToolchainDriftE1101 then RejectedFailClosed;
        transition VerifyingDualToolchainParity_to_SynthesizingThreeRingPartitions_on_toolchainAndStage2ParityLocked first VerifyingDualToolchainParity accept toolchainAndStage2ParityLocked then SynthesizingThreeRingPartitions;
        transition SynthesizingThreeRingPartitions_to_RejectedFailClosed_on_rejectCrossRingCouplingE1105 first SynthesizingThreeRingPartitions accept rejectCrossRingCouplingE1105 then RejectedFailClosed;
        transition SynthesizingThreeRingPartitions_to_EmbeddingFlsTraceability_on_threeRingTopologyIsolated first SynthesizingThreeRingPartitions accept threeRingTopologyIsolated then EmbeddingFlsTraceability;
        transition EmbeddingFlsTraceability_to_RejectedFailClosed_on_rejectMissingFlsAnchorE1102 first EmbeddingFlsTraceability accept rejectMissingFlsAnchorE1102 then RejectedFailClosed;
        transition EmbeddingFlsTraceability_to_AuditingCoreSubsetAndMcdc_on_fiveLinkFlsChainComplete first EmbeddingFlsTraceability accept fiveLinkFlsChainComplete then AuditingCoreSubsetAndMcdc;
        transition AuditingCoreSubsetAndMcdc_to_RejectedFailClosed_on_rejectUncertifiedSymbolOrCoverageE1103E1104 first AuditingCoreSubsetAndMcdc accept rejectUncertifiedSymbolOrCoverageE1103E1104 then RejectedFailClosed;
        transition AuditingCoreSubsetAndMcdc_to_VerifyingRing1SafetyMonitor_on_coreSubsetAndTableA7Verified first AuditingCoreSubsetAndMcdc accept coreSubsetAndTableA7Verified then VerifyingRing1SafetyMonitor;
        transition VerifyingRing1SafetyMonitor_to_RejectedFailClosed_on_rejectMonitorReadbackFaultE1106 first VerifyingRing1SafetyMonitor accept rejectMonitorReadbackFaultE1106 then RejectedFailClosed;
        transition VerifyingRing1SafetyMonitor_to_CertifiedPackageReady_on_certificationEvidenceVerified first VerifyingRing1SafetyMonitor accept certificationEvidenceVerified then CertifiedPackageReady;
        transition RejectedFailClosed_to_Terminal first RejectedFailClosed then Terminal;
        transition CertifiedPackageReady_to_Terminal first CertifiedPackageReady then Terminal;
    }
    connection def BP_11_PartitionToFlsTraceabilityConnection {
        connect ThreeRingAssurancePartitionCompiler.partitioned_workspace_out to FlsParagraphTraceabilitySynthesizer.lowered_module_graph_in;
    }
    doc /* - **Given**: A workspace containing paired criticalup.toml (ferrocene-stable pinned to an exact qualified release and target triple set) and rust-toolchain.toml (pinned to the matching upstream rustc semantic version and target set) parsed via toml_edit and cargo-platform, alongside a negative fixture where rust-toolchain.toml drifts by one minor version or omits a qualified target triple. - **When**: DualModeFerroceneToolchainGate executes probe_criticalup_toolchain and verify_dual_toolchain_parity, evaluating Stage-1 versus Stage-2 self-hosted modular codegen SHA-256 digests (E0705) under. */
    use case def UC_AC_01_Dual_Mode_criticalup_toml_and_rust_toolchain_toml_Parity_Cross_Platform_Discovery_and_DO_330_Fixed_Point_Qualification {
        subject CertifiedCoreSubsetAndCoverageGate;
        objective "Dual Mode criticalup toml and rust toolchain toml Parity Cross Platform Discovery and DO 330 Fixed Point Qualification";
    }
    doc /* - **Given**: A SysML v2 package declaring RequirementDef (REQ-*), STPA safety constraint (SC-*), Ring-1 PartDef / StateDef / ConstraintDef constructs, and Ring-2 FFI PortDef contracts compiled through FlsParagraphTraceabilitySynthesizer, plus a negative fixture containing a Ring-1 function missing its [FLS: fls_*] docstring tag and a Ring-2 unsafe block citing an unknown paragraph fls_nonexistent999. - **When**: crates/compile-sysml/src/ring_partition/ emits the annotated Rust units and crates/verify-baseline audits the 5-link FLS traceability chain against the pinned FLS paragraph catalog. */
    use case def UC_AC_02_End_to_End_FLS_Normative_Paragraph_Traceability_Emission_and_Fail_Closed_Orphan_Detection {
        subject CertifiedCoreSubsetAndCoverageGate;
        objective "End to End FLS Normative Paragraph Traceability Emission and Fail Closed Orphan Detection";
    }
    doc /* - **Given**: Two Ring-1 #![no_std] safety_kernel symbol-report fixtures: Fixture A linking exclusively certified core::option, core::result, core::num, and fixed-size array operations, and Fixture B inadvertently linking __rust_alloc, alloc::vec::Vec, or std::io::stdio::_print. - **When**: CertifiedCoreSubsetAndCoverageGate audits both symbol inventories against the certified core allowlist prefix catalog. - **Then**: Fixture A passes with uncertified_core_symbol_count == 0 (**C-CERT-04**), whereas Fixture B fails closed immediately with DEAP-E1103 enumerating every prohibited std, alloc, or. */
    use case def UC_AC_03_Certified_core_Library_Subset_Symbol_Report_Gate_Rejecting_std_alloc_and_Dynamic_Formatting_Symbols {
        subject CertifiedCoreSubsetAndCoverageGate;
        objective "Certified core Library Subset Symbol Report Gate Rejecting std alloc and Dynamic Formatting Symbols";
    }
    doc /* - **Given**: A DAL-A Ring-1 SafetyKernel module exercised by blanket / llvm-profdata MC/DC condition-vector profiles linked to SysML TestCaseDef declarations (TEST-01 -> REQ-0611), alongside three negative fixtures: (1) a boolean decision a && (b || c) missing independent flip coverage for condition c (99% MC/DC), (2) an overcounted macro-duplicated source span, and (3) a coverage test lacking a SysML TEST-* -> REQ-* verified_by binding. - **When**: CertifiedCoreSubsetAndCoverageGate evaluates DO-178C Table A-7 coverage thresholds for DalLevel::DalA using typed serde_json schemas. - **Then**:. */
    use case def UC_AC_04_Zero_Overcount_Structural_and_MC_DC_Coverage_Verification_Mapped_to_SysML_Verification_Cases {
        subject CertifiedCoreSubsetAndCoverageGate;
        objective "Zero Overcount Structural and MC DC Coverage Verification Mapped to SysML Verification Cases";
    }
    doc /* - **Given**: A SysML v2 system model annotated with AssuranceLevel and @Layer stereotypes (Domain, Application, Presentation) and PortDef [min..max] ranges, plus two negative probes: Probe A introducing an illegal safety_kernel -> ffi_guard_bridge or safety_kernel -> app_flutter dependency edge, and Probe B injecting an out-of-range or NaN OperatorConsole payload across the Ring-2 FFI boundary. - **When**: ThreeRingAssurancePartitionCompiler synthesizes the 3-ring workspace (safety_kernel/, ffi_guard_bridge/, app_flutter/) and executes the Ring-2 boundary verification suite. - **Then**: The. */
    use case def UC_AC_05_Schema_Driven_3_Ring_Workspace_Partitioning_and_Ring_2_Miri_Guarded_FFI_Precondition_Enforcement {
        subject CertifiedCoreSubsetAndCoverageGate;
        objective "Schema Driven 3 Ring Workspace Partitioning and Ring 2 Miri Guarded FFI Precondition Enforcement";
    }
    doc /* - **Given**: An active Ring-1 SafetyMonitor issuing sequenced TelemetryFrame envelopes (seq_t, FIPS 180-4 SHA-256 digest h_t) and two-step challenge nonces (eta_t) through Ring 2 to the untrusted Ring-3 Flutter OperatorConsole, tested against three Ring-3 fault injections: (1) corrupted display frame-digest readback (hat_h_t != h_t), (2) stale frame sequence counter (hat_seq_t < seq_t), and (3) spurious single-step critical ActuatorCommand lacking matching challenge nonce eta_t. - **When**: verify_three_ring_edge_and_monitor_readback evaluates the inbound confirmation and readback envelope. */
    use case def UC_AC_06_Ring_1_Challenge_Response_Command_Interlock_and_TelemetryFrame_Digest_Readback_SafetyMonitor_Against_Ring_3_Faults {
        subject CertifiedCoreSubsetAndCoverageGate;
        objective "Ring 1 Challenge Response Command Interlock and TelemetryFrame Digest Readback SafetyMonitor Against Ring 3 Faults";
    }
    doc /* - **Given**: docs/designs/blueprint-11-ferrocene-do178c-do330-3ring-certification-architecture.md specifying dynamic cross-platform driver probing across macOS, Linux, and Windows (std::env::consts::{DLL_PREFIX, DLL_SUFFIX, EXE_SUFFIX}, camino::Utf8Path), modular submodules under < 380 SLOC (**C-CERT-07**), and the --require-formal-toolchains qualification flag, evaluated in an environment where a required driver is absent versus present. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-11-ferrocene-do178c-do330-3ring-certification-architecture.md and. */
    use case def UC_AC_07_Cross_Platform_Execution_Submodule_Layout_Conformance_and_Twelve_Gate_Blueprint_Verification_Gate {
        subject CertifiedCoreSubsetAndCoverageGate;
        objective "Cross Platform Execution Submodule Layout Conformance and Twelve Gate Blueprint Verification Gate";
    }
    test case def TC_REQ_0611_Ferrocene_Three_Ring_Certification {
        verify requirement REQ_0611;
    }
    test case def TC_REQ_0611_Blueprint_Conformance {
        verify requirement REQ_0611_BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C;
    }
    test case def Test_probe_criticalup_toolchain {
        verify requirement REQ_0611;
    }
    test case def Test_verify_dual_toolchain_parity {
        verify requirement REQ_0611;
    }
    test case def Test_synthesize_three_ring_workspace {
        verify requirement REQ_0611;
    }
    test case def Test_verify_fls_traceability_chain {
        verify requirement REQ_0611;
    }
    test case def Test_verify_certified_core_and_table_a7_coverage {
        verify requirement REQ_0611;
    }
    test case def Test_verify_three_ring_edge_and_monitor_readback {
        verify requirement REQ_0611;
    }
    doc /* Extracted from Mermaid sequenceDiagram specification. */
    interaction def BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture_SequenceInteraction {
        lifeline DualModeFerroceneToolchainGate : DualModeFerroceneToolchainGate;
        lifeline ThreeRingAssurancePartitionCompiler : ThreeRingAssurancePartitionCompiler;
        lifeline FlsParagraphTraceabilitySynthesizer : FlsParagraphTraceabilitySynthesizer;
        lifeline CertifiedCoreSubsetAndCoverageGate : CertifiedCoreSubsetAndCoverageGate;
        message probe_criticalup_toolchain from DualModeFerroceneToolchainGate to DualModeFerroceneToolchainGate;
        message verify_dual_toolchain_parity from DualModeFerroceneToolchainGate to DualModeFerroceneToolchainGate;
        message synthesize_three_ring_workspace from DualModeFerroceneToolchainGate to ThreeRingAssurancePartitionCompiler;
        message reply_ThreeRingWorkspacePlan from ThreeRingAssurancePartitionCompiler to DualModeFerroceneToolchainGate;
        message verify_fls_traceability_chain from ThreeRingAssurancePartitionCompiler to FlsParagraphTraceabilitySynthesizer;
        message reply_FlsTaggedTraceBundle from FlsParagraphTraceabilitySynthesizer to ThreeRingAssurancePartitionCompiler;
        message verify_certified_core_and_table_a7_coverage from FlsParagraphTraceabilitySynthesizer to CertifiedCoreSubsetAndCoverageGate;
        message reply_Do178cTableA7Receipt from CertifiedCoreSubsetAndCoverageGate to FlsParagraphTraceabilitySynthesizer;
        message verify_three_ring_edge_and_monitor_readback from DualModeFerroceneToolchainGate to ThreeRingAssurancePartitionCompiler;
        message reply_SafetyMonitorReadbackReceipt from ThreeRingAssurancePartitionCompiler to DualModeFerroceneToolchainGate;
    }
    doc /* Permitted count of linked Ring-1 ELF or MIR symbols resolving outside the Ferrocene certified `core` subset. [unit: symbols] */
    part def CertifiedCoreSubsetAndCoverageGate {
        attribute uncertified_core_symbol_count : Integer = 0;
        doc /* Minimum Statement, Decision, and MC/DC structural coverage percentage required for Ring-1 DAL-A units (DO-178C Table A-7). [unit: percent] */
        attribute dal_a_mcdc_coverage_min_pct : Integer = 100;
        doc /* Permitted macro-duplicated or inlined source-span coverage overcount anomalies in `blanket` / `llvm-profdata` reports. [unit: spans] */
        attribute overcounted_span_count : Integer = 0;
        doc /* Requires every structural coverage record to resolve via a SysML `TEST-*` verification case to at least one `REQ-*`. [unit: flag] */
        attribute require_sysml_test_case_link : Boolean = true;
        doc /* Receives Ring-1 ELF/MIR `symbol-report` inventories, `llvm-profdata` MC/DC bitmaps, and SysML `TestCaseDef` bindings. [protocol: SymbolReportAndLlvmProfdata] */
        port in elf_and_profdata_in : String {
            flow in elf_and_profdata_in_flow : String;
        }
        doc /* Emits certified `core` symbol attestation and zero-overcount Statement, Decision, and MC/DC traceability evidence. [protocol: Do178cTableA7CoverageMatrix] */
        port out table_a7_evidence_out : String {
            flow out table_a7_evidence_out_flow : String;
        }
        doc /* Streams `DEAP-E1103` uncertified symbol violations and `DEAP-E1104` Table A-7 coverage or `TEST-*` linkage failures. [protocol: RustcJsonDiagnosticStream] */
        port out coverage_diag_out : String {
            flow out coverage_diag_out_flow : String;
        }
    }
    doc /* Permitted semantic `rustc` version or target-triple delta between `criticalup.toml` and `rust-toolchain.toml`. [unit: versions] */
    part def DualModeFerroceneToolchainGate {
        attribute max_toolchain_version_drift : Integer = 0;
        doc /* Permitted Stage-1 versus Stage-2 SHA-256 digest mismatch count under Ferrocene and standard `rustc` bootstrap (`E0705`). [unit: files] */
        attribute stage_fixed_point_mismatch_files : Integer = 0;
        doc /* Enforces `--locked --offline` deterministic compilation and independent `verify-baseline` TQL-5 cross-checking. [unit: flag] */
        attribute reproducible_ferrocene_lock : Boolean = true;
        doc /* When asserted in qualification CI, fails closed if Ferrocene criticalup or toolchains are absent. [unit: flag] */
        attribute require_formal_toolchains : Boolean = false;
        doc /* Canonical deterministic blueprint component identifier (`DEAP-BP-DET-11`). [unit: uuidv5] */
        attribute deterministic_uuid : String = "6ba7b810-9dad-11d1-80b4-00c04fd430cb";
        doc /* Receives workspace `criticalup.toml`, `rust-toolchain.toml`, `Cargo.lock`, and Stage-1/Stage-2 tree digests. [protocol: CriticalupAndRustToolchainToml] */
        port in toolchain_manifests_in : String {
            flow in toolchain_manifests_in_flow : String;
        }
        doc /* Emits DO-330 TQL-1/TQL-5 and ISO 26262-8 TCL3 qualification parity evidence and reproducible build SHA-256 receipts. [protocol: Do330ToolQualificationReceipt] */
        port out qualification_dossier_out : String {
            flow out qualification_dossier_out_flow : String;
        }
        doc /* Streams fail-closed `DEAP-E1101` toolchain drift and `E0705` fixed-point mismatch diagnostics in `--message-format=json`. [protocol: RustcJsonDiagnosticStream] */
        port out toolchain_diag_out : String {
            flow out toolchain_diag_out_flow : String;
        }
    }
    doc /* Permitted count of Ring-1 AST items or Ring-2 `unsafe` blocks lacking a verified `[FLS: fls_*]` normative paragraph tag. [unit: items] */
    part def FlsParagraphTraceabilitySynthesizer {
        attribute unmapped_fls_item_count : Integer = 0;
        doc /* Mandatory end-to-end traceability link count (`REQ-*`, `SC-*`, `RustItem`, `fls_*`, `KaniAndCoverage`). [unit: links] */
        attribute trace_chain_depth : Integer = 5;
        doc /* Requires every Ring-2 FFI `unsafe` block to carry `// SAFETY: [SC-*] [FLS: fls_*]` matching the pinned FLS catalog. [unit: flag] */
        attribute enforce_safety_comment_fls : Boolean = true;
        doc /* Receives lowered `ModuleGraph` compilation units, `RequirementDef` / `SC-*` links, and pinned FLS paragraph catalog IDs. [protocol: SysmlAndRustModuleGraph] */
        port in lowered_module_graph_in : String {
            flow in lowered_module_graph_in_flow : String;
        }
        doc /* Emits 7-section documented Rust items stamped with `[FLS: fls_*]` anchors and deterministic FLS traceability matrices. [protocol: FlsTaggedRustAndTraceMatrix] */
        port out fls_annotated_units_out : String {
            flow out fls_annotated_units_out_flow : String;
        }
        doc /* Emits fail-closed `DEAP-E1102` diagnostics on missing, malformed, or unregistered `fls_*` paragraph identifiers. [protocol: RustcJsonDiagnosticStream] */
        port out fls_diag_out : String {
            flow out fls_diag_out_flow : String;
        }
    }
    doc /* Mandatory count of isolated downstream assurance rings (`Ring 1 safety_kernel`, `Ring 2 ffi_guard_bridge`, `Ring 3 hmi_presentation_shell`). [unit: rings] */
    part def ThreeRingAssurancePartitionCompiler {
        attribute assurance_ring_count : Integer = 3;
        doc /* Permitted heap allocation primitives (`alloc`, `Box`, `Vec`, `String`) inside Ring-1 `#![no_std]` safety kernels. [unit: allocs] */
        attribute max_ring1_heap_allocations : Integer = 0;
        doc /* Permitted Ring-2 C-ABI or `flutter_rust_bridge` entrypoints bypassing SysML port range, finiteness, or FSM state preconditions. [unit: entrypoints] */
        attribute max_unguarded_ffi_entrypoints : Integer = 0;
        doc /* Synthesizes Ring-1 challenge-response command interlocks and SHA-256/CRC-32C `TelemetryFrame` digest readback monitors. [unit: flag] */
        attribute enable_ring1_safety_monitor : Boolean = true;
        doc /* Receives validated SysML `PackageDef`, `AssuranceLevel` / `@Layer` annotations, `PortDef` bounds, and `StateDef` machines. [protocol: SysmlAssuranceAndIcdPackage] */
        port in sysml_assurance_ast_in : String {
            flow in sysml_assurance_ast_in_flow : String;
        }
        doc /* Emits partitioned `safety_kernel/`, `ffi_guard_bridge/`, and `app_flutter/` artifacts with Ring-1 `SafetyMonitor` contracts. [protocol: ThreeRingRustAndFlutterTree] */
        port out partitioned_workspace_out : String {
            flow out partitioned_workspace_out_flow : String;
        }
        doc /* Emits `DEAP-E1105` cross-ring coupling / guard violations and `DEAP-E1106` challenge-response or frame-digest monitor faults. [protocol: RustcJsonDiagnosticStream] */
        port out partition_diag_out : String {
            flow out partition_diag_out_flow : String;
        }
    }
}

// Authoritative Self-Hosting 4-Level Modular Multi-File Code Generator Metamodel (BP-07 / REQ-0607)
// Pure abstract MBSE compiler specification -- zero hardcoded domain concepts.

package ModularCodegen {
    package domain {
        part def ModularityBoundsSpec {
            @Layer("Domain");
            attribute max_file_lines : Integer = 400;
            attribute max_items_per_file : Integer = 12;
            attribute max_cyclomatic_complexity : Integer = 8;
            attribute max_action_steps_per_stage : Integer = 5;
            attribute max_transitions_per_handler : Integer = 6;

            constraint def ValidBoundsEnvelope(
                in max_file_lines : Integer,
                in max_items_per_file : Integer,
                in max_cyclomatic_complexity : Integer,
                in max_action_steps_per_stage : Integer,
                in max_transitions_per_handler : Integer
            ) {
                max_file_lines >= 50
                    and max_file_lines <= 400
                    and max_items_per_file >= 1
                    and max_items_per_file <= 12
                    and max_cyclomatic_complexity >= 1
                    and max_cyclomatic_complexity <= 8
                    and max_action_steps_per_stage >= 1
                    and max_action_steps_per_stage <= 8
                    and max_transitions_per_handler >= 1
                    and max_transitions_per_handler <= 8;
            }
        }

        part def LayerHierarchySpec {
            @Layer("Domain");
            attribute domain_tier : Integer = 1;
            attribute application_tier : Integer = 2;
            attribute presentation_tier : Integer = 3;

            constraint def NonUpwardLayerImport(
                in importer_tier : Integer,
                in imported_tier : Integer
            ) {
                importer_tier >= 1
                    and importer_tier <= 3
                    and imported_tier >= 1
                    and imported_tier <= 3
                    and importer_tier >= imported_tier;
            }
        }

        part def CompilationUnitSpec {
            @Layer("Domain");
            attribute relative_path : String = "mod.rs";
            attribute layer_tier : Integer = 1;
            attribute item_count : Integer = 1;
            attribute line_count : Integer = 10;
            attribute max_function_vg : Integer = 1;

            constraint def UnitObeysBounds(
                in line_count : Integer,
                in max_file_lines : Integer,
                in item_count : Integer,
                in max_items_per_file : Integer,
                in max_function_vg : Integer,
                in max_cyclomatic_complexity : Integer
            ) {
                line_count >= 1
                    and line_count <= max_file_lines
                    and item_count >= 1
                    and item_count <= max_items_per_file
                    and max_function_vg >= 1
                    and max_function_vg <= max_cyclomatic_complexity;
            }
        }

        calc def ComputeCyclomaticComplexity(in decision_points : Integer) : Integer {
            decision_points + 1;
        }
    }

    package lowering {
        port def SchemaPackageAstPort {
            attribute protocol_family : String = "SysmlPackageAst";
            flow in package_ast_stream : String;
        }

        port def VerifiedModuleGraphPort {
            attribute protocol_family : String = "VerifiedModuleGraph";
            flow out module_graph_stream : String;
        }

        part def FourLevelLoweringEngineSpec {
            @Layer("Application");
            attribute active_level_count : Integer = 4;
            attribute fail_closed_on_cycle : Boolean = true;
            attribute fail_closed_on_upward_layer : Boolean = true;
            part bounds_ref : ModularityBoundsSpec;
            part layer_ref : LayerHierarchySpec;
            port in schema_in : SchemaPackageAstPort;
            port out graph_out : VerifiedModuleGraphPort;

            constraint def LoweringLevelsComplete(in active_level_count : Integer) {
                active_level_count == 4;
            }
        }

        action def ExecuteFourLevelLoweringPipeline {
            in package_count : Integer;
            out unit_count : Integer;
            step resolve_package_hierarchy;
            step extract_part_layer_annotations;
            step build_directed_dependency_edges;
            step run_kahn_topological_cycle_check;
            step verify_non_upward_layer_edges;
            step partition_package_directories;
            step split_part_responsibility_facets;
            step outline_sequential_action_stages;
            step outline_source_state_handlers;
            step hoist_anf_boolean_subpredicates;
            step synthesize_barrel_reexport_facades;
            step verify_in_memory_modularity_bounds;
        }

        state def ModularCodegenLifecycle {
            state Idle;
            state ResolvingLayers;
            state VerifyingDag;
            state PartitioningPackages;
            state SplittingFacets;
            state OutliningBehaviors;
            state HoistingPredicates;
            state SynthesizingTree;
            state VerifyingBounds;
            state PersistingAtomic;
            state Verified;
            state AbortedFailClosed;

            transition Initial_to_Idle first Initial then Idle;
            transition Idle_to_ResolvingLayers first Idle accept start_compilation then ResolvingLayers;
            transition ResolvingLayers_to_VerifyingDag first ResolvingLayers accept layers_resolved then VerifyingDag;
            transition VerifyingDag_to_PartitioningPackages first VerifyingDag accept dag_verified then PartitioningPackages;
            transition VerifyingDag_to_AbortedOnCycle first VerifyingDag accept on_cycle_e0702 then AbortedFailClosed;
            transition VerifyingDag_to_AbortedOnUpwardLayer first VerifyingDag accept on_upward_layer_e0703 then AbortedFailClosed;
            transition PartitioningPackages_to_SplittingFacets first PartitioningPackages accept packages_partitioned then SplittingFacets;
            transition SplittingFacets_to_OutliningBehaviors first SplittingFacets accept facets_split then OutliningBehaviors;
            transition OutliningBehaviors_to_HoistingPredicates first OutliningBehaviors accept behaviors_outlined then HoistingPredicates;
            transition HoistingPredicates_to_SynthesizingTree first HoistingPredicates accept predicates_hoisted then SynthesizingTree;
            transition SynthesizingTree_to_VerifyingBounds first SynthesizingTree accept tree_synthesized then VerifyingBounds;
            transition VerifyingBounds_to_PersistingAtomic first VerifyingBounds accept bounds_verified then PersistingAtomic;
            transition VerifyingBounds_to_AbortedOnDrift first VerifyingBounds accept on_fixed_point_drift_e0705 then AbortedFailClosed;
            transition PersistingAtomic_to_Verified first PersistingAtomic accept atomic_commit_succeeded then Verified;
            transition Verified_to_Idle first Verified accept reset_lifecycle then Idle;
            transition AbortedFailClosed_to_Idle first AbortedFailClosed accept reset_lifecycle then Idle;
        }
    }

    package emitters {
        port def ModularRustEmitPort {
            attribute protocol_family : String = "AtomicRustTree";
            flow out rust_manifest_sha256 : String;
        }

        port def ModularFlutterEmitPort {
            attribute protocol_family : String = "AtomicDartTree";
            flow out dart_manifest_sha256 : String;
        }

        part def ModularEmitterSuiteSpec {
            @Layer("Presentation");
            attribute atomic_persistence : Boolean = true;
            attribute check36_docstrings_enabled : Boolean = true;
            attribute stage_mismatch_files : Integer = 0;
            part engine_ref : FourLevelLoweringEngineSpec;
            port out rust_tree_out : ModularRustEmitPort;
            port out flutter_tree_out : ModularFlutterEmitPort;

            constraint def FixedPointBootstrapParity(in stage_mismatch_files : Integer) {
                stage_mismatch_files == 0;
            }
        }
    }
}
