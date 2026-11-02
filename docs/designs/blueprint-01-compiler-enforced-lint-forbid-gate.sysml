doc /* Translated from Level 0 OEM Markdown specification */
package BP_01_Compiler_Enforced_Forbid_Lint_Gate_Lossless_CST_Attribute_Verification_and_Modular_SLOC_Ceiling_Architecture_Blueprint {
    attribute Blueprint_ID : String = "BP-01";
    attribute Requirement_ID : String = "REQ-0601";
    attribute Title : String = "Compiler-Enforced Forbid Lint Gate, Lossless CST Attribute Verification, and Modular SLOC Ceiling Architecture Blueprint";
    attribute Status : String = "Approved";
    attribute Target_Crates : String = "crates/deap-core, crates/verify-baseline, crates/compile-sysml, crates/ingest-sysml, crates/assemble-conops, crates/reconcile-backlog";
    attribute Tracked_Issues : String = "#491, #492, #496, #497, #498, #499, #500, #501, #502, #506, #508";
    attribute Generation_Mode : String = "subagent";
    attribute Specification_Source : String = "Cargo.toml:1-25 / crates/deap-core/src/lib.rs:1-40 / crates/verify-baseline/src/lib.rs:1-40 / crates/verify-baseline/src/main.rs:1-46 / #491";
    doc /* Invoke cargo clippy --workspace --all-targets --message-format=json */
    action def InvokeCargoClippyWorkspaceAllTargets;
    doc /* Load [workspace.lints] forbid table and clippy.toml (80 lines, 5 args, disallowed-types) */
    action def LoadWorkspaceLintsForbidTableAnd;
    doc /* Execute verify-baseline . --no-domain */
    action def ExecuteVerifyBaselineNoDomain;
    doc /* Parse crates/ ** / *.rs via ra_ap_syntax::SourceFile (audit ast::Attr + <= 500 exec SLOC ratchet) */
    action def ParseCratesRsViaRaAp;
    doc /* Execute verify-baseline check-blueprint blueprint-01 --dry-run */
    action def ExecuteVerifyBaselineCheckBlueprintBlueprint;
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0601_BP_01_Compiler_Enforced_Forbid_Lint_Gate_Lossless_CST_Attribute_Verification_and_Modular_SLOC_Ceiling_Architecture_Blueprint {
        id = "REQ-0601";
        text = "The Digital Engineering Agent Platform (DEAP) specification and verification compiler (CompilerLintEnforcementGate, legacy blueprint reference DEAP-BP-DET-01, UUIDv5 6ba7b810-9dad-11d1-80b4-00c04fd430c1, Complexity Class P, diagnostic bindings **E0453**, **E0100**, **E0304**, governed by DO-178C";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_official_rustc_e0453_cargo_clippy_json_threshold_enforcement : String = "- **Given**: ./Cargo.toml configures [workspace.lints.rust] and [workspace.lints.clippy] at \"forbid\", and ./clippy.toml configures too-many-lines-threshold = 80, too-many-arguments-threshold = 5, and disallowed-types. - **When**: cargo clippy --workspace --all-targets --message-format=json is executed against any production or test module in ./crates/ * that introduces #![allow(clippy::unwrap_used)], exceeds the configured Clippy structural thresholds under denied Clippy pedantic/complexity gates, or references a disallowed type. - **Then**: The official compiler pipeline fails closed with.";
        attribute ac_02_lossless_ra_ap_syntax_cst_rejection_of_allow_suppressions_and_rustc_private : String = "- **Given**: The ra_ap_syntax lossless CST attribute verifier in ./crates/verify-baseline/src/checks/structural.rs operating on CRLF/LF-normalized UTF-8 source trees. - **When**: A Rust file in ./crates/ * contains #![allow(clippy::expect_used)], multi-line #[ cfg_attr ( test , allow ( clippy :: panic ) ) ], #![feature(rustc_private)], or lacks required crate/workspace forbid/deny governance, while string literals and comments mentioning those tokens are ignored by CST node typing. - **Then**: ./target/release/verify-baseline . --no-domain identifies the exact ast::Attr CST node, reports the.";
        attribute ac_03_modular_380_sloc_target_500_exec_sloc_ceiling_and_monotonic_downward_legacy_ratchet : String = "- **Given**: New and refactored Rust modules under ./crates/ * governed by the < 380 executable SLOC modular target, <= 500 executable SLOC hard ceiling, and the downward legacy ratchet table. - **When**: Any unratcheted module exceeds 500 non-trivia/non-test CST executable lines, or a ratcheted legacy module increases above its recorded ratchet ceiling R_t(f). - **Then**: The CST SLOC gate fails closed with exit code 1, whereas reducing a legacy module below 500 executable SLOC permanently graduates it out of the legacy ratchet toward 0 remaining exceptions.";
        attribute ac_04_cross_platform_portability_semantic_type_alias_compliance_across_macos_linux_and_windows : String = "- **Given**: Workspace binaries, unit tests (#[cfg(test)]), integration tests (./crates/ * /tests/ *.rs), and canonical semantic type aliases (ExtractedMethodSignature, ComponentMethodTable) migrated into the new < 380-SLOC ra_ap_syntax typed item extractor deap_core::rust_cst::items (rust_cst/items.rs under ./crates/deap-core/src/, Streams 4.2 and 4.4) replacing the completely deleted ./crates/deap-core/src/source_ast.rs (-1,049 SLOC). - **When**: cargo test --workspace and ./target/release/verify-baseline . --no-domain execute across macOS, Linux, and Windows environments with CRLF or LF.";
        attribute ac_05_dogfood_ingestion_and_sysml_v2_compilation_of_req_0601 : String = "- **Given**: This blueprint specification ./docs/designs/blueprint-01-compiler-enforced-lint-forbid-gate.md. - **When**: ./target/release/verify-baseline check-blueprint ./docs/designs/blueprint-01-compiler-enforced-lint-forbid-gate.md is executed. - **Then**: All 12 mechanical blueprint verification gates pass (12/12), and the translator synthesizes **REQ-0601**, CompilerLintEnforcementGate, its interface ports, state attributes, formal constraints (**C-LINT-01**..**C-LINT-04**), and **AC-01**..**AC-05** into ./docs/designs/blueprint-01-compiler-enforced-lint-forbid-gate.sysml and.";
        doc /* \begin{aligned} T_{\text{cst}}(F, N) &= \sum_{i=1}^{F} O(N_i) = O(N_{\text{total}}) \\ \text{ExecSLOC}(f_i) &= \left\lvert \left\{ \ell \in \text{Lines}_{\text{LF}}(f_i) \setminus \text{CfgTest}(f_i) \;\middle|\; \exists \tau \in \text{Tokens}_{\text{CST}}(\ell), \; \neg \text{IsTrivia}(\tau) \right\} \right\rvert \\ \text{Budget}(f_i, t) &= \begin{cases} 500 & \text{if } f_i \notin \mathcal{L}_t \quad (\text{target } < 380) \\ R_t(f_i) & \text{if } f_i \in \mathcal{L}_t \quad \text{with } R_{t+1}(f_i) = \min\left(R_t(f_i), \text{ExecSLOC}(f_i)\right) \to 0 \end{cases} \end{aligned} */
        constraint def Invariant_REQ_0601_C_LINT_04;
        doc /* \begin{aligned} \text{ValidWorkspace}(W) &\iff (E_{\text{allow}}(W) = 0) \land (E_{\text{rustc\_private}}(W) = 0) \land (\text{Cov}_{\text{forbid}}(W) = 100) \land (N_{\text{crates}}(W) = 6) \\ \text{GateVerdict}(W, t) &\iff \text{ClippyJsonOk}(W) \cdot \prod_{i=1}^{F} \mathbb{I}\left(\lvert \text{CstViolations}(f_i) \rvert = 0 \land \text{ExecSLOC}(f_i) \le \text{Budget}(f_i, t)\right) = 1 \end{aligned} */
        constraint def Invariant_REQ_0601_FormalInvariant;
        require Invariant_REQ_0601_C_LINT_04;
        require Invariant_REQ_0601_FormalInvariant;
        verify by AC_01_Official_Rustc_E0453_Cargo_Clippy_JSON_Threshold_Enforcement;
        verify by AC_02_Lossless_ra_ap_syntax_CST_Rejection_of_Allow_Suppressions_and_rustc_private;
        verify by AC_03_Modular_380_SLOC_Target_500_Exec_SLOC_Ceiling_and_Monotonic_Downward_Legacy_Ratchet;
        verify by AC_04_Cross_Platform_Portability_Semantic_Type_Alias_Compliance_Across_macOS_Linux_and_Windows;
        verify by AC_05_Dogfood_Ingestion_and_SysML_v2_Compilation_of_REQ_0601;
        satisfy by CompilerLintEnforcementGate;
    }
    doc /* Extracted from Mermaid stateDiagram specification. */
    state def BP_01_Compiler_Enforced_Forbid_Lint_Gate_Lossless_CST_Attribute_Verification_and_Modular_SLOC_Ceiling_Architecture_Blueprint_LifecycleState {
        state RunningClippyJson;
        state GateRejected;
        state ParsingLosslessCst;
        state AuditingAstAttrs;
        state MeasuringExecSloc;
        state RatchetTightened;
        state GateVerified;
        transition Initial_to_RunningClippyJson first Initial then RunningClippyJson;
        transition RunningClippyJson_to_GateRejected_on_clippyJsonErrorOrE0453 first RunningClippyJson accept clippyJsonErrorOrE0453 then GateRejected;
        transition RunningClippyJson_to_ParsingLosslessCst_on_clippyJsonClean first RunningClippyJson accept clippyJsonClean then ParsingLosslessCst;
        transition ParsingLosslessCst_to_GateRejected_on_cstSyntaxErrorsDetected first ParsingLosslessCst accept cstSyntaxErrorsDetected then GateRejected;
        transition ParsingLosslessCst_to_AuditingAstAttrs_on_cstLosslessTreeReady first ParsingLosslessCst accept cstLosslessTreeReady then AuditingAstAttrs;
        transition AuditingAstAttrs_to_GateRejected_on_allowOrRustcPrivateAttrFound first AuditingAstAttrs accept allowOrRustcPrivateAttrFound then GateRejected;
        transition AuditingAstAttrs_to_MeasuringExecSloc_on_allAstAttrsCompliant first AuditingAstAttrs accept allAstAttrsCompliant then MeasuringExecSloc;
        transition MeasuringExecSloc_to_GateRejected_on_execSlocExceeds500OrRatchet first MeasuringExecSloc accept execSlocExceeds500OrRatchet then GateRejected;
        transition MeasuringExecSloc_to_RatchetTightened_on_legacyFileReducedBelowBudget first MeasuringExecSloc accept legacyFileReducedBelowBudget then RatchetTightened;
        transition MeasuringExecSloc_to_GateVerified_on_execSlocWithinModularBudget first MeasuringExecSloc accept execSlocWithinModularBudget then GateVerified;
        transition RatchetTightened_to_GateVerified_on_ratchetMonotonicallyUpdated first RatchetTightened accept ratchetMonotonicallyUpdated then GateVerified;
        transition GateRejected_to_Terminal first GateRejected then Terminal;
        transition GateVerified_to_Terminal first GateVerified then Terminal;
    }
    doc /* - **Given**: ./Cargo.toml configures [workspace.lints.rust] and [workspace.lints.clippy] at \"forbid\", and ./clippy.toml configures too-many-lines-threshold = 80, too-many-arguments-threshold = 5, and disallowed-types. - **When**: cargo clippy --workspace --all-targets --message-format=json is executed against any production or test module in ./crates/ * that introduces #![allow(clippy::unwrap_used)], exceeds the configured Clippy structural thresholds under denied Clippy pedantic/complexity gates, or references a disallowed type. - **Then**: The official compiler pipeline fails closed with. */
    use case def UC_AC_01_Official_Rustc_E0453_Cargo_Clippy_JSON_Threshold_Enforcement {
        subject CompilerLintEnforcementGate;
        objective "Official Rustc E0453 Cargo Clippy JSON Threshold Enforcement";
    }
    doc /* - **Given**: The ra_ap_syntax lossless CST attribute verifier in ./crates/verify-baseline/src/checks/structural.rs operating on CRLF/LF-normalized UTF-8 source trees. - **When**: A Rust file in ./crates/ * contains #![allow(clippy::expect_used)], multi-line #[ cfg_attr ( test , allow ( clippy :: panic ) ) ], #![feature(rustc_private)], or lacks required crate/workspace forbid/deny governance, while string literals and comments mentioning those tokens are ignored by CST node typing. - **Then**: ./target/release/verify-baseline . --no-domain identifies the exact ast::Attr CST node, reports the. */
    use case def UC_AC_02_Lossless_ra_ap_syntax_CST_Rejection_of_Allow_Suppressions_and_rustc_private {
        subject CompilerLintEnforcementGate;
        objective "Lossless ra ap syntax CST Rejection of Allow Suppressions and rustc private";
    }
    doc /* - **Given**: New and refactored Rust modules under ./crates/ * governed by the < 380 executable SLOC modular target, <= 500 executable SLOC hard ceiling, and the downward legacy ratchet table. - **When**: Any unratcheted module exceeds 500 non-trivia/non-test CST executable lines, or a ratcheted legacy module increases above its recorded ratchet ceiling R_t(f). - **Then**: The CST SLOC gate fails closed with exit code 1, whereas reducing a legacy module below 500 executable SLOC permanently graduates it out of the legacy ratchet toward 0 remaining exceptions. */
    use case def UC_AC_03_Modular_380_SLOC_Target_500_Exec_SLOC_Ceiling_and_Monotonic_Downward_Legacy_Ratchet {
        subject CompilerLintEnforcementGate;
        objective "Modular 380 SLOC Target 500 Exec SLOC Ceiling and Monotonic Downward Legacy Ratchet";
    }
    doc /* - **Given**: Workspace binaries, unit tests (#[cfg(test)]), integration tests (./crates/ * /tests/ *.rs), and canonical semantic type aliases (ExtractedMethodSignature, ComponentMethodTable) migrated into the new < 380-SLOC ra_ap_syntax typed item extractor deap_core::rust_cst::items (rust_cst/items.rs under ./crates/deap-core/src/, Streams 4.2 and 4.4) replacing the completely deleted ./crates/deap-core/src/source_ast.rs (-1,049 SLOC). - **When**: cargo test --workspace and ./target/release/verify-baseline . --no-domain execute across macOS, Linux, and Windows environments with CRLF or LF. */
    use case def UC_AC_04_Cross_Platform_Portability_Semantic_Type_Alias_Compliance_Across_macOS_Linux_and_Windows {
        subject CompilerLintEnforcementGate;
        objective "Cross Platform Portability Semantic Type Alias Compliance Across macOS Linux and Windows";
    }
    doc /* - **Given**: This blueprint specification ./docs/designs/blueprint-01-compiler-enforced-lint-forbid-gate.md. - **When**: ./target/release/verify-baseline check-blueprint ./docs/designs/blueprint-01-compiler-enforced-lint-forbid-gate.md is executed. - **Then**: All 12 mechanical blueprint verification gates pass (12/12), and the translator synthesizes **REQ-0601**, CompilerLintEnforcementGate, its interface ports, state attributes, formal constraints (**C-LINT-01**..**C-LINT-04**), and **AC-01**..**AC-05** into ./docs/designs/blueprint-01-compiler-enforced-lint-forbid-gate.sysml and. */
    use case def UC_AC_05_Dogfood_Ingestion_and_SysML_v2_Compilation_of_REQ_0601 {
        subject CompilerLintEnforcementGate;
        objective "Dogfood Ingestion and SysML v2 Compilation of REQ 0601";
    }
    doc /* Extracted from Mermaid sequenceDiagram specification. */
    interaction def BP_01_Compiler_Enforced_Forbid_Lint_Gate_Lossless_CST_Attribute_Verification_and_Modular_SLOC_Ceiling_Architecture_Blueprint_SequenceInteraction {
        lifeline Eng : Compiler_Engineer_CI;
        lifeline Cargo : cargo_clippy_message_format_json;
        lifeline Config : Cargo_toml_clippy_toml;
        lifeline CstGate : verify_baseline;
        lifeline Ingest : ingest_sysml;
        message InvokeCargoClippyWorkspaceAllTargets from Eng to Cargo;
        message LoadWorkspaceLintsForbidTableAnd from Cargo to Config;
        message reply_ReturnCompilerLintPolicyAcross6 from Config to Cargo;
        message reply_StreamStructuredNdjsonDiagnostics from Cargo to Eng;
        message ExecuteVerifyBaselineNoDomain from Eng to CstGate;
        message ParseCratesRsViaRaAp from CstGate to CstGate;
        message reply_ReturnExitCode0WhenSuppressions from CstGate to Eng;
        message ExecuteVerifyBaselineCheckBlueprintBlueprint from Eng to Ingest;
        message reply_EmitCanonicalSysMLV2AstAnd from Ingest to Eng;
    }
    doc /* Configures `clippy::unwrap_used = "forbid"` in `./Cargo.toml` [unit: flag] */
    part def CompilerLintEnforcementGate {
        attribute forbid_unwrap_used : Boolean = true;
        doc /* Configures `clippy::expect_used = "forbid"` in `./Cargo.toml` [unit: flag] */
        attribute forbid_expect_used : Boolean = true;
        doc /* Configures `clippy::panic = "forbid"` in `./Cargo.toml` [unit: flag] */
        attribute forbid_panic : Boolean = true;
        doc /* Configures `clippy::indexing_slicing = "forbid"` in `./Cargo.toml` [unit: flag] */
        attribute forbid_indexing_slicing : Boolean = true;
        doc /* Configures `clippy::disallowed_types = "forbid"` and `./clippy.toml` `disallowed-types` [unit: flag] */
        attribute forbid_disallowed_types : Boolean = true;
        doc /* Configures `clippy::type_complexity = "forbid"` in `./Cargo.toml` [unit: flag] */
        attribute forbid_type_complexity : Boolean = true;
        doc /* Function body ceiling configured in `./clippy.toml` (`too-many-lines-threshold = 80`) [unit: lines] */
        attribute too_many_lines_threshold : Integer = 80;
        doc /* Function parameter ceiling configured in `./clippy.toml` (`too-many-arguments-threshold = 5`) [unit: count] */
        attribute too_many_arguments_threshold : Integer = 5;
        doc /* Target modular executable SLOC budget (`< 380` SLOC) per Rust module [unit: lines] */
        attribute target_module_exec_sloc : Integer = 380;
        doc /* Hard fail-closed executable SLOC ceiling (`<= 500` SLOC) enforced with downward ratchet to `0` [unit: lines] */
        attribute max_module_exec_sloc_ceiling : Integer = 500;
        doc /* Maximum permitted `#[allow(...)]` or `#![feature(rustc_private)]` CST nodes across `./crates/ *` [unit: count] */
        attribute allowed_suppression_count : Integer = 0;
        doc /* Percentage of workspace safety lints enforced at `forbid` level (**C-LINT-02**) [unit: percent] */
        attribute forbid_rule_coverage : Integer = 100;
        doc /* Number of workspace member crates inheriting `[lints] workspace = true` (**C-LINT-03**) [unit: count] */
        attribute crates_enforced_count : Integer = 6;
        doc /* Ingests `./Cargo.toml` (`[workspace.lints]`) and `./clippy.toml` thresholds via `camino::Utf8Path` [protocol: TomlDocument] */
        port in CargoWorkspaceConfigPort : String {
            flow in CargoWorkspaceConfigPort_flow : String;
        }
        doc /* Consumes structured NDJSON compiler diagnostics from `cargo clippy --message-format=json` [protocol: CargoJsonLines] */
        port in ClippyJsonStreamPort : String {
            flow in ClippyJsonStreamPort_flow : String;
        }
        doc /* Parses CRLF/LF-normalized UTF-8 `.rs` files into lossless `ra_ap_syntax::SourceFile` syntax trees [protocol: RaApSyntaxCst] */
        port in RaSyntaxCstInputPort : String {
            flow in RaSyntaxCstInputPort_flow : String;
        }
        doc /* Emits deterministic `rustc` `E0453` and Clippy threshold diagnostics across macOS, Linux, and Windows [protocol: RustcDiagnostic] */
        port out CompilerDiagnosticOutputPort : String {
            flow out CompilerDiagnosticOutputPort_flow : String;
        }
        doc /* Emits fail-closed CST attribute and SLOC ratchet verification verdict (`Ok` or sorted `Err` vector) [protocol: GateVerdict] */
        port out BaselineGateVerdictPort : Boolean {
            flow out BaselineGateVerdictPort_flow : Boolean;
        }
    }
    doc /* Invokes cargo clippy --workspace --all-targets --message-format=json using std::path / camino and ./clippy.toml (too-many-lines-threshold = 80, too-many-arguments-threshold = 5, disallowed-types) and parses structured compiler-message payloads. */
    part def ClippyJsonDiagnosticGate {
        attribute quantity : Integer = 1;
        port in cmd_in : String;
        port out status_out : String;
    }
    doc /* Parses .rs files into lossless ra_ap_syntax::SourceFile CSTs (zero hand-rolled lexers, zero rustc_private) to audit #![forbid(...)], #![deny(...)], and forbidden #[allow(...)] / cfg_attr attributes. */
    part def RaSyntaxCstAttributeVerifier {
        attribute quantity : Integer = 1;
        port in cmd_in : String;
        port out status_out : String;
    }
    doc /* Computes executable SLOC directly from lossless CST non-trivia tokens, enforcing < 380 target SLOC, <= 500 hard executable SLOC ceiling, and monotonic downward legacy ratchet convergence to 0. */
    part def ModularSlocRatchetEnforcer {
        attribute quantity : Integer = 1;
        port in cmd_in : String;
        port out status_out : String;
    }
    doc /* Enforces identical forbid lint discipline across unit tests (#[cfg(test)]) and integration tests (tests/ *.rs) with CRLF/LF normalization across macOS, Linux, and Windows. */
    part def CrossPlatformTestTargetEnforcer {
        attribute quantity : Integer = 1;
        port in cmd_in : String;
        port out status_out : String;
    }
}