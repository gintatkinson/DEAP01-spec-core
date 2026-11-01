doc /* Translated from Level 0 OEM Markdown specification */
package BP_02_Check_34_Typed_CST_Purity_Fail_Closed_IO_and_Official_Cargo_Tooling_Gate_Blueprint {
    attribute Blueprint_ID : String = "BP-02";
    attribute Requirement_ID : String = "REQ-0602";
    attribute Title : String = "Check 34 Typed CST Purity, Fail-Closed IO, and Official Cargo Tooling Gate Blueprint";
    attribute Status : String = "Approved";
    attribute Target_Crates : String = "crates/deap-core, crates/verify-baseline";
    attribute Tracked_Issues : String = "#492, #494, #498, #500, #501, #502, #503, #506, #510";
    attribute Generation_Mode : String = "subagent";
    attribute Specification_Source : String = "crates/verify-baseline/src/checks/structural.rs:1-120";
    doc /* Eliminates hand-rolled Rust/Dart character-stream lexer and recursive-descent brace scanner. */
    attribute crates_deap_core_src_source_ast_rs : String = "**Delete `-1,049` SLOC (100% removal)**";
    doc /* Removes hand-rolled `Cargo.toml` section splitters and `Cargo.lock` text readers. */
    attribute crates_deap_core_src_workspace_rs : String = "**Delete `-620+` SLOC**";
    doc /* Removes line-masked string-contains anti-gaming heuristics, brace matchers, and ad-hoc TOML scanners. */
    attribute crates_verify_baseline_src_checks_structural_rs : String = "**Delete `-1,400+` SLOC**";
    doc /* Typed workspace manifest, target `cfg(...)` platform, lint table, and `Cargo.lock` loader using `cargo_metadata`, `cargo-lock`, `cargo-util-schemas`, `cargo-platform`, and `toml_edit`. */
    attribute deap_core_cargo_manifest_ast : String = "**New `< 380` exec SLOC**";
    doc /* `ra_ap_syntax` + `ra_ap_line_index` gate rejecting `IndexExpr`, `.unwrap()`/`.expect()`/`panic!`, vacuous `#[test]` bodies, and `rustc_private` attributes. */
    attribute verify_baseline_checks_cst_purity_gate : String = "**New `< 380` exec SLOC**";
    doc /* Typed `MatchExpr`/`MatchArm`/`Pat`/`MethodCallExpr`/`TryExpr` verifier rejecting `Err(_) => Ok(default)` and fail-open I/O combinators. */
    attribute verify_baseline_checks_io_fail_closed_gate : String = "**New `< 380` exec SLOC**";
    doc /* Enforces workspace `forbid` clippy lints, banned dependency policy, and lockfile integrity over `deap_core::cargo_manifest_ast`. */
    attribute verify_baseline_checks_cargo_policy_gate : String = "**New `< 380` exec SLOC**";
    doc /* [unit: count] */
    attribute cst_purity_violation_count : Real;
    doc /* [unit: count] */
    attribute fail_open_io_site_count : Real;
    doc /* [unit: count] */
    attribute vacuous_test_count : Real;
    doc /* [unit: count] */
    attribute cargo_policy_violation_count : Real;
    doc /* [unit: count] */
    attribute legacy_scanner_residue_count : Real;
    doc /* Typed `WorkspaceCargoSnapshot`, `PurityConstraintId`, and `PurityViolationRecord` models backed by `cargo_metadata::Metadata`, `cargo_lock::Lockfile`, `cargo_util_schemas::manifest::TomlManifest`, `cargo_platform::Platform`, and `toml_edit::DocumentMut` (`-620+` SLOC deleted from `./crates/deap-core/src/workspace.rs`; `-1,049` SLOC deleted via removal of `./crates/deap-core/src/source_ast.rs`). */
    attribute Layer_1_Domain_State_Data_Model : String = "deap_core::cargo_manifest_ast` (registered in `./crates/deap-core/src/lib.rs`)";
    doc /* Lossless `ra_ap_syntax` (`Edition::Edition2024`) CST walkers over `ast::{SourceFile, Fn, MatchExpr, MatchArmList, MatchArm, Pat, WildcardPat, IdentPat, CallExpr, MethodCallExpr, IndexExpr, TryExpr, MacroCall}` + `ra_ap_line_index::LineIndex` (`-1,400+` SLOC deleted from `./crates/verify-baseline/src/checks/structural.rs`). */
    attribute Layer_2_Logic_State_Management : String = "verify_baseline::checks::cst_purity_gate`, `verify_baseline::checks::io_fail_closed_gate`, `verify_baseline::checks::cargo_policy_gate";
    doc /* Deterministic sorted `file_rel:line:col: [C-PURITY-XX] ` stderr/JSON diagnostic emission and fail-closed non-zero process exit code (`1`) wired into `verify-baseline . --no-domain` and `check-blueprint`. */
    attribute Layer_3_Presentation_Actuator_Interface_Binding : String = "verify_baseline` CLI check runner (`./crates/verify-baseline/src/checks/structural.rs`)";
    doc /* load_workspace_cargo_snapshot(repo_root) */
    action def load_workspace_cargo_snapshot;
    doc /* audit_cargo_workspace_policy(snapshot, banned_crates) */
    action def audit_cargo_workspace_policy;
    doc /* parse_lossless_rust_cst(file_rel, raw_source) */
    action def parse_lossless_rust_cst;
    doc /* audit_io_fail_closed_cst(ctx, source_file) */
    action def audit_io_fail_closed_cst;
    doc /* [unit: count] */
    assert constraint C_PURITY_01 {
        cst_purity_violation_count >= 0 and cst_purity_violation_count <= 0;
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
        cargo_policy_violation_count >= 0 and cargo_policy_violation_count <= 0;
    }
    doc /* [unit: count] */
    assert constraint C_PURITY_05 {
        legacy_scanner_residue_count >= 0 and legacy_scanner_residue_count <= 0;
    }
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0602_BP_02_Check_34_Typed_CST_Purity_Fail_Closed_IO_and_Official_Cargo_Tooling_Gate_Blueprint {
        id = "REQ-0602";
        text = "During iterative compiler hardening across the Digital Engineering Agent Platform (DEAP) specification core (DEAP01-spec-core), delegating self-audit unit tests, source purity gates, and Cargo workspace inspections to hand-rolled line/brace text scanners produced systematic test gaming, false";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_multi_line_panicking_index_unwrap_expect_panic_and_rustc_private_cst_rejection : String = "- **Given**: Any workspace Rust file under ./crates/ containing a multi-line ast::IndexExpr (&slice[ start..end]), ast::MethodCallExpr invoking .unwrap() or .expect(...), ast::MacroCall invoking panic! or todo!, or an inner attribute #![feature(rustc_private)]. - **When**: verify_baseline::checks::cst_purity_gate parses the CRLF-normalized file into ra_ap_syntax::SourceFile and resolves offsets through ra_ap_line_index::LineIndex (C-PURITY-01). - **Then**: Every violation is reported with exact 1-based (line, col) coordinates regardless of multi-line formatting, while comments and string.";
        attribute ac_02_fail_closed_matcharm_and_combinator_i_o_error_propagation_enforcement : String = "- **Given**: A Rust function containing a MatchExpr with an Err(_) or Err(_e) MatchArm pattern (Pat::TupleStructPat wrapping WildcardPat or IdentPat) whose body returns Ok(Default::default()), Ok(Vec::new()), or continue without recording a diagnostic, or a MethodCallExpr chaining .into_iter().flatten(), .filter_map(Result::ok), or read_to_string(...).unwrap_or_default(). - **When**: verify_baseline::checks::io_fail_closed_gate traverses ast::{MatchExpr, MatchArmList, MatchArm, Pat, WildcardPat, IdentPat, CallExpr, MethodCallExpr, TryExpr} (C-PURITY-02). - **Then**: The gate rejects the.";
        attribute ac_03_typed_ast_fn_anti_gaming_body_enforcement : String = "- **Given**: An ast::Fn node carrying a #[test] attribute (and no #[should_panic]) whose body lacks behavioral assert! / assert_eq! / assert_ne! ast::MacroCall nodes, invokes size_of / size_of_val in an ast::CallExpr, guards assertions behind an if ... .exists() filesystem check, or calls temp_dir().join(...) instead of an RAII temp directory guard. - **When**: verify_baseline::checks::cst_purity_gate audits all #[test] ast::Fn declarations (C-PURITY-03). - **Then**: The gate fails closed, identifies the offending test function symbol and LineIndex line number, and terminates with exit code 1.";
        attribute ac_04_official_cargo_toml_manifest_platform_cfg_lint_table_and_lockfile_policy_gate : String = "- **Given**: A workspace Cargo.toml, member Cargo.toml, or Cargo.lock containing a banned dependency (regex), missing [workspace.lints.clippy] forbid entries (unwrap_used, expect_used, panic, indexing_slicing), an invalid [target.'cfg(...)'.dependencies] platform expression, or an out-of-sync lockfile entry. - **When**: deap_core::cargo_manifest_ast and verify_baseline::checks::cargo_policy_gate evaluate the workspace via cargo_metadata, cargo-lock, cargo-util-schemas, cargo-platform, and toml_edit (C-PURITY-04). - **Then**: The policy gate reports the exact manifest key or lockfile package.";
        attribute ac_05_net_negative_legacy_scanner_retirement_blueprint_sysml_v2_roundtrip : String = "- **Given**: The refactored workspace with ./crates/deap-core/src/source_ast.rs (-1,049 SLOC) deleted, -620+ SLOC removed from ./crates/deap-core/src/workspace.rs, -1,400+ SLOC removed from ./crates/verify-baseline/src/checks/structural.rs, all four replacement submodules < 380 executable SLOC, and this blueprint at ./docs/designs/blueprint-02-check34-ast-purity-and-anti-gaming-gate.md. - **When**: ./target/release/verify-baseline check-blueprint ./docs/designs/blueprint-02-check34-ast-purity-and-anti-gaming-gate.md and ./target/release/verify-baseline . --no-domain are executed. - **Then**:.";
        doc /* \begin{aligned} \text{ValidWorkspace}_{\text{Check34}}(W) &\iff \forall f \in F_{\text{rs}}(W), \; \lvert \text{parseErrors}(\text{CST}(f)) \rvert = 0 \land \sum_{k=1}^{5} C_{\text{purity},k}(W) = 0 \\ \sum_{k=1}^{5} C_{\text{purity},k}(W) &= C_{\text{cst}}(W) + C_{\text{io}}(W) + C_{\text{test}}(W) + C_{\text{cargo}}(W) + C_{\text{residue}}(W) = 0 \end{aligned} */
        constraint def Invariant_REQ_0602_SC_01;
        doc /* \begin{aligned} \Delta \text{SLOC}_{\text{legacy}} &\le -(1049 + 620 + 1400) = -3069 \text{ SLOC} \\ \forall m \in M_{\text{new}}, \quad \text{ExecSLOC}(m) &< 380 \\ T_{\text{check34}}(F, N, D, V) &= \sum_{i=1}^{F} O(N_i) + O(D) + O(V \log V) \end{aligned} */
        constraint def Invariant_REQ_0602_SC_02;
        require Invariant_REQ_0602_SC_01;
        require Invariant_REQ_0602_SC_02;
        verify by AC_01_Multi_Line_Panicking_Index_Unwrap_Expect_Panic_and_rustc_private_CST_Rejection;
        verify by AC_02_Fail_Closed_MatchArm_and_Combinator_I_O_Error_Propagation_Enforcement;
        verify by AC_03_Typed_ast_Fn_Anti_Gaming_Body_Enforcement;
        verify by AC_04_Official_Cargo_TOML_Manifest_Platform_cfg_Lint_Table_and_Lockfile_Policy_Gate;
        verify by AC_05_Net_Negative_Legacy_Scanner_Retirement_Blueprint_SysML_v2_Roundtrip;
        satisfy by CargoManifestAstLoader;
    }
    doc /* Synthesized deterministic package lifecycle state machine. */
    state def BP_02_Check_34_Typed_CST_Purity_Fail_Closed_IO_and_Official_Cargo_Tooling_Gate_Blueprint_LifecycleState {
        state CargoManifestAstLoader_State;
        state CargoPolicyGate_State;
        state Check34AstPurityAndAntiGamingGate_State;
        transition Initial_to_CargoManifestAstLoader_State first Initial then CargoManifestAstLoader_State;
        transition CargoManifestAstLoader_State_to_CargoPolicyGate_State_on_load_workspace_cargo_snapshot first CargoManifestAstLoader_State accept load_workspace_cargo_snapshot then CargoPolicyGate_State;
        transition CargoPolicyGate_State_to_Check34AstPurityAndAntiGamingGate_State_on_audit_cargo_workspace_policy first CargoPolicyGate_State accept audit_cargo_workspace_policy then Check34AstPurityAndAntiGamingGate_State;
        transition Check34AstPurityAndAntiGamingGate_State_to_Terminal first Check34AstPurityAndAntiGamingGate_State then Terminal;
    }
    doc /* - **Given**: Any workspace Rust file under ./crates/ containing a multi-line ast::IndexExpr (&slice[ start..end]), ast::MethodCallExpr invoking .unwrap() or .expect(...), ast::MacroCall invoking panic! or todo!, or an inner attribute #![feature(rustc_private)]. - **When**: verify_baseline::checks::cst_purity_gate parses the CRLF-normalized file into ra_ap_syntax::SourceFile and resolves offsets through ra_ap_line_index::LineIndex (C-PURITY-01). - **Then**: Every violation is reported with exact 1-based (line, col) coordinates regardless of multi-line formatting, while comments and string. */
    use case def UC_AC_01_Multi_Line_Panicking_Index_Unwrap_Expect_Panic_and_rustc_private_CST_Rejection {
        subject CargoManifestAstLoader;
        objective "Multi Line Panicking Index Unwrap Expect Panic and rustc private CST Rejection";
    }
    doc /* - **Given**: A Rust function containing a MatchExpr with an Err(_) or Err(_e) MatchArm pattern (Pat::TupleStructPat wrapping WildcardPat or IdentPat) whose body returns Ok(Default::default()), Ok(Vec::new()), or continue without recording a diagnostic, or a MethodCallExpr chaining .into_iter().flatten(), .filter_map(Result::ok), or read_to_string(...).unwrap_or_default(). - **When**: verify_baseline::checks::io_fail_closed_gate traverses ast::{MatchExpr, MatchArmList, MatchArm, Pat, WildcardPat, IdentPat, CallExpr, MethodCallExpr, TryExpr} (C-PURITY-02). - **Then**: The gate rejects the. */
    use case def UC_AC_02_Fail_Closed_MatchArm_and_Combinator_I_O_Error_Propagation_Enforcement {
        subject CargoManifestAstLoader;
        objective "Fail Closed MatchArm and Combinator I O Error Propagation Enforcement";
    }
    doc /* - **Given**: An ast::Fn node carrying a #[test] attribute (and no #[should_panic]) whose body lacks behavioral assert! / assert_eq! / assert_ne! ast::MacroCall nodes, invokes size_of / size_of_val in an ast::CallExpr, guards assertions behind an if ... .exists() filesystem check, or calls temp_dir().join(...) instead of an RAII temp directory guard. - **When**: verify_baseline::checks::cst_purity_gate audits all #[test] ast::Fn declarations (C-PURITY-03). - **Then**: The gate fails closed, identifies the offending test function symbol and LineIndex line number, and terminates with exit code 1. */
    use case def UC_AC_03_Typed_ast_Fn_Anti_Gaming_Body_Enforcement {
        subject CargoManifestAstLoader;
        objective "Typed ast Fn Anti Gaming Body Enforcement";
    }
    doc /* - **Given**: A workspace Cargo.toml, member Cargo.toml, or Cargo.lock containing a banned dependency (regex), missing [workspace.lints.clippy] forbid entries (unwrap_used, expect_used, panic, indexing_slicing), an invalid [target.'cfg(...)'.dependencies] platform expression, or an out-of-sync lockfile entry. - **When**: deap_core::cargo_manifest_ast and verify_baseline::checks::cargo_policy_gate evaluate the workspace via cargo_metadata, cargo-lock, cargo-util-schemas, cargo-platform, and toml_edit (C-PURITY-04). - **Then**: The policy gate reports the exact manifest key or lockfile package. */
    use case def UC_AC_04_Official_Cargo_TOML_Manifest_Platform_cfg_Lint_Table_and_Lockfile_Policy_Gate {
        subject CargoManifestAstLoader;
        objective "Official Cargo TOML Manifest Platform cfg Lint Table and Lockfile Policy Gate";
    }
    doc /* - **Given**: The refactored workspace with ./crates/deap-core/src/source_ast.rs (-1,049 SLOC) deleted, -620+ SLOC removed from ./crates/deap-core/src/workspace.rs, -1,400+ SLOC removed from ./crates/verify-baseline/src/checks/structural.rs, all four replacement submodules < 380 executable SLOC, and this blueprint at ./docs/designs/blueprint-02-check34-ast-purity-and-anti-gaming-gate.md. - **When**: ./target/release/verify-baseline check-blueprint ./docs/designs/blueprint-02-check34-ast-purity-and-anti-gaming-gate.md and ./target/release/verify-baseline . --no-domain are executed. - **Then**:. */
    use case def UC_AC_05_Net_Negative_Legacy_Scanner_Retirement_Blueprint_SysML_v2_Roundtrip {
        subject CargoManifestAstLoader;
        objective "Net Negative Legacy Scanner Retirement Blueprint SysML v2 Roundtrip";
    }
    doc /* Extracted from Mermaid sequenceDiagram specification. */
    interaction def BP_02_Check_34_Typed_CST_Purity_Fail_Closed_IO_and_Official_Cargo_Tooling_Gate_Blueprint_SequenceInteraction {
        lifeline Runner : VerifyBaselineCLI;
        lifeline CargoLdr : CargoManifestAstLoader;
        lifeline CargoPol : CargoPolicyGate;
        lifeline CstGate : CstPurityGate;
        lifeline IoGate : IoFailClosedGate;
        message load_workspace_cargo_snapshot from Runner to CargoLdr;
        message reply_WorkspaceCargoSnapshot from CargoLdr to Runner;
        message audit_cargo_workspace_policy from Runner to CargoPol;
        message reply_CPurity04Diagnostics from CargoPol to Runner;
        message parse_lossless_rust_cst from Runner to CstGate;
        message audit_io_fail_closed_cst from CstGate to IoGate;
        message reply_CPurity02MatchArmAndMethodCallExpr from IoGate to CstGate;
        message reply_SortedCPurity010203 from CstGate to Runner;
    }
    doc /* `deap_core::cargo_manifest_ast` (`< 380` exec SLOC) loading workspace packages, `toml_edit` lint tables, `cargo-util-schemas` manifests, `cargo-platform` targets, and `cargo-lock` dependency graphs */
    part def CargoManifestAstLoader {
        attribute quantity : Integer = 1;
        attribute type : String = "Subcomponent";
    }
    doc /* `verify_baseline::checks::cargo_policy_gate` (`< 380` exec SLOC) verifying banned crates (`regex`, `rustc_private`), mandatory `forbid` clippy lints, and lockfile parity */
    part def CargoPolicyGate {
        attribute quantity : Integer = 1;
        attribute type : String = "Subcomponent";
    }
    doc /* Total typed `ra_ap_syntax` violations for panicking `IndexExpr`, `.unwrap()`, `.expect()`, `panic!`, or `#![feature(rustc_private)]` [unit: count] */
    part def Check34AstPurityAndAntiGamingGate {
        attribute cst_purity_violation_count : Integer = 0;
        doc /* Total typed `MatchArm` (`Err(_) => Ok(default)`) or combinator (`.flatten()`, `Result::ok`, `.unwrap_or_default()`) fail-open I/O sites [unit: count] */
        attribute fail_open_io_site_count : Integer = 0;
        doc /* Total `ast::Fn` test bodies exhibiting zero behavioral assertions, `size_of` tautologies, `.exists()` skip guards, or shared temp paths [unit: count] */
        attribute vacuous_test_count : Integer = 0;
        doc /* Total manifest or lockfile policy violations detected via `cargo_metadata`, `cargo-lock`, `cargo-util-schemas`, `cargo-platform`, and `toml_edit` [unit: count] */
        attribute cargo_policy_violation_count : Integer = 0;
        doc /* Total residual hand-rolled Rust/TOML scanner modules, duplicate SHA-256 constants, or submodules exceeding `380` executable SLOC [unit: count] */
        attribute legacy_scanner_residue_count : Integer = 0;
        doc /* Receives CRLF-normalized `ra_ap_syntax::SourceFile` lossless CST trees and `ra_ap_line_index::LineIndex` tables across `./crates/ ** / *.rs` [protocol: RaApSyntaxCst] */
        port in WorkspaceRustCstPort : String {
            flow in WorkspaceRustCstPort_flow : String;
        }
        doc /* Receives typed `cargo_metadata::Metadata`, `cargo_lock::Lockfile`, `cargo_util_schemas`, `cargo_platform::Platform`, and `toml_edit::DocumentMut` models [protocol: CargoMetadataTomlAst] */
        port in CargoWorkspaceAstPort : String {
            flow in CargoWorkspaceAstPort_flow : String;
        }
        doc /* Emits sorted `camino::Utf8Path` relative file-and-line diagnostics (`PurityViolationRecord`) when any Check 34 sub-gate fails [protocol: DeterministicDiag] */
        port out PurityViolationDiagnosticPort : String {
            flow out PurityViolationDiagnosticPort_flow : String;
        }
        doc /* Emits deterministic pass (`0`) or fail-closed (`1`) status to the `verify-baseline` CLI runner [protocol: ExitCodeContract] */
        port out Check34GateStatusPort : Integer {
            flow out Check34GateStatusPort_flow : Integer;
        }
    }
    doc /* `verify_baseline::checks::cst_purity_gate` (`< 380` exec SLOC) traversing `ast::{SourceFile, Fn, IndexExpr, CallExpr, MethodCallExpr, MacroCall}` with `LineIndex` span resolution */
    part def CstPurityAndAntiGamingGate {
        attribute quantity : Integer = 1;
        attribute type : String = "Subcomponent";
    }
    doc /* `verify_baseline::checks::io_fail_closed_gate` (`< 380` exec SLOC) traversing `ast::{MatchExpr, MatchArmList, MatchArm, Pat, WildcardPat, IdentPat, MethodCallExpr, TryExpr}` to reject silent error swallowing */
    part def IoFailClosedGate {
        attribute quantity : Integer = 1;
        attribute type : String = "Subcomponent";
    }
}