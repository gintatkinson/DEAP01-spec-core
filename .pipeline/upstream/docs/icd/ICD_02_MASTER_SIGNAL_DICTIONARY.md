| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #640 |
| **Title** | Master Signal Flow Dictionary & Safety Invariants |
| **Type** | icd |
| **Interface Level** | Level 1C Logical Interface |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Level 1C: Master Signal Flow Dictionary & Safety Invariants

## 1. Executive Summary & Signal Flow Overview

This Level 1C Master Signal Flow Dictionary (`ICD_02_MASTER_SIGNAL_DICTIONARY.md`) specifies the complete signal metrology, typing contracts, SI units, valid mathematical domain intervals, update rates, fail-safe default values, and STPA safety invariants for all 11 architectural blueprint packages (`BP-01` through `BP-11`) and 53 `part def` components in `./.pipeline/upstream/schema/model.sysml`.

### 1.1 Contract A: 100% Normative OMG SysML v2.0 + KerML 1.0 Grammar Coverage
Every signal flow entry (`SIG-BP01-001` through `SIG-BP11-106`) is projected directly from the lossless SysML v2 Concrete/Abstract Syntax Tree (`deap_core::sysml_cst`) with full Definition and Usage symmetry across all six normative OMG SysML v2.0 and KerML 1.0 construct families:
- **Namespaces & Metadata**: `package`, `library package`, `standard library package`, `import` (`all`, `::*`, `::**`, filter `[...]`), `alias`, `doc`, `comment`, `rep`, `dependency`, and `metadata def` / `metadata` / `@Metadata` / `#user_keyword` (modeling `#hazard`, `#risk`, `#capability`, `#rt_loop`, `#dal`, `#fls` via standard OMG SysML v2 `metadata def` keywords alongside backward-compatible parsing of legacy `hazard def`, `risk def`, and `capability def`).
- **Structural & Data Types**: `attribute def`/`attribute`, `enum def`/`enum`, `item def`/`item`, `part def`/`part`, `port def`/`port` (`in`, `out`, `inout`, `~` conjugation), `connection def`/`connection` (`connect ... to ...`), `interface def`/`interface`, `flow def`/`flow` (`of ... from ... to ...`), `allocation def`/`allocation` (`allocate ... to ...`), and `occurrence def`/`occurrence` (`individual`, `snapshot`, `timeslice`).
- **Behavioral & State**: `action def`/`action` (`perform`, `accept`, `send`, `assign`, `if`/`else`, `while`/`loop`, `for`, `fork`, `join`, `merge`, `decide`, `first ... then ...` succession) and `state def`/`state` (`exhibit`, `parallel`, `entry`, `do`, `exit`, `transition first ... accept ... if ... do ... then ...`).
- **Calculations, Constraints & Requirements**: `calc def`/`calc` (`return`), `constraint def`/`constraint` (`assert`, `assume`, `require`), `requirement def`/`requirement` (`subject`, `actor`, `stakeholder`, `framed concern`, `satisfy ... by ...`), and `concern def`/`concern`.
- **Use Cases, Analysis, Verification & Views**: `use case def`/`use case` (`include use case`), `analysis def`/`analysis` (`objective`), `verification def`/`verification` (`verify requirement`), `test case def`/`test case`, `interaction def`/`interaction`, `view def`/`view`, `viewpoint def`/`viewpoint`, `rendering def`/`rendering` (`expose`, `filter`, `render`), and `variation`/`variant`.
- **KerML 1.0 Specialization, Multiplicity & Expressions**: `:` (`defined by`), `:>` (`subsets`), `:>>` (`redefines`), `::>` (`references`), `~` (`conjugates`), `ref`, `abstract`, `readonly`, `derived`, `end`, `default`, `:=`, `[lo..hi]`, `[*]`, `{ordered, nonunique}`, full KerML logical/conditional/relational/arithmetic/collection expressions (`implies`, `xor`, `??`, `->`, `collect`, `select`, `forAll`, `exists`), and `ISQ`/`SI`/`ScalarValues` standard libraries.

### 1.2 Contract B: 100% Official Rust Compiler, Cargo, `rust-analyzer` & Rustdoc Crate Stack
All signal producers and consumers across `BP-01..11` execute on the official Rust compiler and `rust-analyzer` crate stack with zero legacy hand-rolled scanners:
- **Workspace & Manifests**: `cargo_metadata`, `cargo-lock`, `cargo-util-schemas`, `cargo-platform`, and `toml_edit`.
- **Lossless Rust CST, VFS & Codemods**: `ra_ap_syntax`, `ra_ap_vfs`, `ra_ap_paths`, `ra_ap_line_index`, `ra_ap_text_edit`, `rowan`, `camino`, and `ignore` (`ClippyAttributeCstParser`, `UnifiedRustDartCstParser`, `Check34AstPurityAndAntiGamingGate`, `FivePillarStaticAstAnalyzer`, `LosslessRustCstAnalyzer`, `ZeroTokenRustSpanCodemodEngine`).
- **Lossless SysML v2, Dart & Polyglot CST + Incremental Query DB**: `logos` + `rowan` + `salsa` + `ariadne` (`deap_core::sysml_cst`, `deap_core::dart_cst`, `ingest_sysml::cst::{idl_cst, yang_cst}`, typed `serde_json` OpenAPI/JSON-Schema).
- **Lossless Markdown AST**: `pulldown-cmark` + `pulldown-cmark-to-cmark`.
- **Deterministic Fixed-Point Codegen**: `syn` + `proc-macro2` + `quote` + `prettyplease`.
- **5-Stage Verification & Ferrocene Qualification**: Stage 1 `rustc`/`cargo clippy --message-format=json` -> Stage 2 isolated `./lints/deap-dylint-mir/` (`feature(rustc_private)`, `rustc_middle::mir`) -> Stage 3 `cargo miri` + `flutter_rust_bridge` -> Stage 4 `cargo kani` (`kani::proof`) -> Stage 5 Ferrocene `criticalup.toml` + `FLS: fls_*` + `symbol-report`/`blanket` MC/DC + 3-Ring DAL-A..E partitioning.

## 2. Master Signal Flow Dictionary Table

| Signal ID | Signal Name | Source Port | Dest Port | Data Type | SI Units | Valid Range | Update Rate | Safe Default Value | Schema Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `SIG-BP01-001` | `Bp01ForbidLintPortFlow` | `PORT-BP01-FORBID_LINT_DEF` | `PORT-BP01-MANIFEST_IN` | Record | dimensionless | [1, 65536] bytes | Aperiodic [1, 10] ms | `"REJECTED_EMPTY_MANIFEST"` | `schema/model.sysml#L5` |
| `SIG-BP01-002` | `Bp01ManifestToForbidGateFlow` | `PORT-BP01-STATUS_OUT` | `PORT-BP01-MANIFEST_IN` | String | dimensionless | [1, 65536] bytes | Aperiodic [1, 10] ms | `"FORBID_VIOLATION_E0453"` | `schema/model.sysml#L66` |
| `SIG-BP01-003` | `CargoManifestInputSignal` | `PORT-BP01-STATUS_OUT` | `PORT-BP01-MANIFEST_IN` | String | dimensionless | [1, 65536] bytes | Aperiodic [1, 10] ms | `"FORBID_VIOLATION_E0453"` | `schema/model.sysml#L152` |
| `SIG-BP01-004` | `RustSourceTokenStreamSignal` | `PORT-BP01-CACP_STATUS_OUT` | `PORT-BP01-TOKEN_STREAM_IN` | String | dimensionless | [1, 1048576] bytes | 100 Hz | `"EMPTY_RUST_CST_STREAM"` | `schema/model.sysml#L158` |
| `SIG-BP01-005` | `CompilerDiagnosticOutputSignal` | `PORT-BP01-COMPILER_DIAG_OUT` | `PORT-BP08-FINDING_IN` | String | dimensionless | [0, 262144] bytes | Aperiodic [1, 10] ms | `"error[E0453]: fail-closed"` | `schema/model.sysml#L164` |
| `SIG-BP01-006` | `BaselineGateVerdictSignal` | `PORT-BP01-GATE_VERDICT_OUT` | `PORT-BP09-GATE_IN` | Bool | dimensionless | [false, true] | Aperiodic [1, 10] ms | `false` | `schema/model.sysml#L170` |
| `SIG-BP01-007` | `WorkspaceManifestCmdSignal` | `PORT-BP01-FORBID_LINT_DEF` | `PORT-BP01-WMFG_CMD_IN` | String | dimensionless | [1, 4096] bytes | Aperiodic [1, 10] ms | `"ABORT_MANIFEST_CHECK"` | `schema/model.sysml#L179` |
| `SIG-BP01-008` | `WorkspaceManifestStatusSignal` | `PORT-BP01-STATUS_OUT` | `PORT-BP01-MANIFEST_IN` | String | dimensionless | [1, 65536] bytes | Aperiodic [1, 10] ms | `"MANIFEST_FORBID_FAILED"` | `schema/model.sysml#L180` |
| `SIG-BP01-009` | `ClippyAttributeCstCmdSignal` | `PORT-BP01-FORBID_LINT_DEF` | `PORT-BP01-CACP_CMD_IN` | String | dimensionless | [1, 4096] bytes | Aperiodic [1, 10] ms | `"ABORT_CST_PARSE"` | `schema/model.sysml#L185` |
| `SIG-BP01-010` | `ClippyAttributeCstStatusSignal` | `PORT-BP01-CACP_STATUS_OUT` | `PORT-BP01-TOKEN_STREAM_IN` | String | dimensionless | [1, 65536] bytes | Aperiodic [1, 10] ms | `"ALLOW_ATTRIBUTE_DETECTED"` | `schema/model.sysml#L186` |
| `SIG-BP01-011` | `CrateRootSanitizerCmdSignal` | `PORT-BP01-FORBID_LINT_DEF` | `PORT-BP01-CRAS_CMD_IN` | String | dimensionless | [1, 4096] bytes | Aperiodic [1, 10] ms | `"ABORT_SANITIZE"` | `schema/model.sysml#L191` |
| `SIG-BP01-012` | `CrateRootSanitizerStatusSignal` | `PORT-BP01-CRAS_STATUS_OUT` | `PORT-BP01-TOKEN_STREAM_IN` | String | dimensionless | [1, 65536] bytes | Aperiodic [1, 10] ms | `"CRATE_ROOT_UNSANITIZED"` | `schema/model.sysml#L192` |
| `SIG-BP01-013` | `TestTargetLintCmdSignal` | `PORT-BP01-FORBID_LINT_DEF` | `PORT-BP01-TTLE_CMD_IN` | String | dimensionless | [1, 4096] bytes | Aperiodic [1, 10] ms | `"ABORT_TEST_LINT"` | `schema/model.sysml#L197` |
| `SIG-BP01-014` | `TestTargetLintStatusSignal` | `PORT-BP01-TTLE_STATUS_OUT` | `PORT-BP01-TOKEN_STREAM_IN` | String | dimensionless | [1, 65536] bytes | Aperiodic [1, 10] ms | `"TEST_EXEMPTION_DETECTED"` | `schema/model.sysml#L198` |
| `SIG-BP02-015` | `Bp02PurityDiagnosticPortFlow` | `PORT-BP02-PURITY_DIAG_DEF` | `PORT-BP02-WS_RUST_CST_IN` | Record | dimensionless | [0, 1] exit_code | Aperiodic [1, 15] ms | `1` | `schema/model.sysml#L204` |
| `SIG-BP02-016` | `Bp02AuditorToPurityGateFlow` | `PORT-BP02-AUDIT_AST_OUT` | `PORT-BP02-TEST_AST_IN` | String | dimensionless | [1, 524288] bytes | 100 Hz | `"VACUOUS_TEST_REJECTED"` | `schema/model.sysml#L257` |
| `SIG-BP02-017` | `AntiGamingAuditAstSignal` | `PORT-BP02-AUDIT_AST_OUT` | `PORT-BP02-TEST_AST_IN` | String | dimensionless | [1, 524288] bytes | 100 Hz | `"VACUOUS_TEST_REJECTED"` | `schema/model.sysml#L331` |
| `SIG-BP02-018` | `WorkspaceRustCstSignal` | `PORT-BP02-PURITY_DIAG_DEF` | `PORT-BP02-WS_RUST_CST_IN` | String | dimensionless | [1, 1048576] bytes | 100 Hz | `"UNPARSED_RUST_CST"` | `schema/model.sysml#L345` |
| `SIG-BP02-019` | `TestFunctionAstStreamSignal` | `PORT-BP02-AUDIT_AST_OUT` | `PORT-BP02-TEST_AST_IN` | String | dimensionless | [1, 524288] bytes | 100 Hz | `"VACUOUS_TEST_REJECTED"` | `schema/model.sysml#L351` |
| `SIG-BP02-020` | `PurityViolationDiagSignal` | `PORT-BP02-PURITY_DIAG_OUT` | `PORT-BP08-FINDING_IN` | String | dimensionless | [0, 262144] bytes | Aperiodic [1, 15] ms | `"C_PURITY_VIOLATION_ERR"` | `schema/model.sysml#L357` |
| `SIG-BP02-021` | `Check34GateStatusSignal` | `PORT-BP02-GATE_STATUS_OUT` | `PORT-BP09-GATE_IN` | Int32 | dimensionless | [0, 1] | Aperiodic [1, 15] ms | `1` | `schema/model.sysml#L363` |
| `SIG-BP03-022` | `Bp03BijectiveRoundtripPortFlow` | `PORT-BP03-ROUNDTRIP_DEF` | `PORT-BP03-RAW_SYSML_IN` | Record | dimensionless | [1, 2097152] bytes | Aperiodic [1, 25] ms | `"ROUNDTRIP_MISMATCH_E0400"` | `schema/model.sysml#L388` |
| `SIG-BP03-023` | `Bp03SerializerToRoundtripFlow` | `PORT-BP03-SYSML_OUT` | `PORT-BP03-RAW_SYSML_IN` | String | dimensionless | [1, 2097152] bytes | Aperiodic [1, 25] ms | `"ROUNDTRIP_MISMATCH_E0400"` | `schema/model.sysml#L456` |
| `SIG-BP03-024` | `SerializedSysmlOutSignal` | `PORT-BP03-SYSML_OUT` | `PORT-BP03-RAW_SYSML_IN` | String | dimensionless | [1, 2097152] bytes | Aperiodic [1, 25] ms | `"ROUNDTRIP_MISMATCH_E0400"` | `schema/model.sysml#L534` |
| `SIG-BP03-025` | `RawSysmlSourceInputSignal` | `PORT-BP03-SYSML_OUT` | `PORT-BP03-RAW_SYSML_IN` | String | dimensionless | [1, 2097152] bytes | Aperiodic [1, 25] ms | `"EMPTY_SYSML_SOURCE_E0100"` | `schema/model.sysml#L561` |
| `SIG-BP03-026` | `TokenStreamChannelSignal` | `PORT-BP03-TOKEN_CH_INOUT` | `PORT-BP10-SYSML_SRC_IN` | String | dimensionless | [1, 2097152] bytes | 100 Hz | `"LEXER_DRIFT_E0100"` | `schema/model.sysml#L567` |
| `SIG-BP03-027` | `CanonicalSysmlOutputSignal` | `PORT-BP03-SER_SYSML_OUT` | `PORT-BP04-PKG_AST_IN` | String | dimensionless | [1, 2097152] bytes | Aperiodic [1, 25] ms | `"INVALID_SYSML_AST_E0200"` | `schema/model.sysml#L571` |
| `SIG-BP03-028` | `BijectiveDiffDiagSignal` | `PORT-BP03-DIFF_DIAG_OUT` | `PORT-BP08-FINDING_IN` | String | dimensionless | [0, 262144] bytes | Aperiodic [1, 25] ms | `"AST_DIFF_DETECTED_E0400"` | `schema/model.sysml#L577` |
| `SIG-BP04-029` | `Bp04ForwardProjectionPortFlow` | `PORT-BP04-FWD_PROJ_DEF` | `PORT-BP04-PKG_AST_IN` | Record | dimensionless | [1, 2097152] bytes | Aperiodic [5, 50] ms | `"FORWARD_SYNC_ABORTED"` | `schema/model.sysml#L587` |
| `SIG-BP04-030` | `Bp04ReconcilerToCompilerFlow` | `PORT-BP04-SYNC_OUT` | `PORT-BP04-PKG_AST_IN` | String | dimensionless | [1, 2097152] bytes | Aperiodic [5, 50] ms | `"REVERSE_SYNC_REJECTED"` | `schema/model.sysml#L665` |
| `SIG-BP04-031` | `AgileSpecPackageAstInSignal` | `PORT-BP04-FWD_PROJ_DEF` | `PORT-BP04-ASFP_AST_IN` | String | dimensionless | [1, 2097152] bytes | Aperiodic [5, 50] ms | `"EMPTY_PACKAGE_AST"` | `schema/model.sysml#L771` |
| `SIG-BP04-032` | `AgileSpecMarkdownOutSignal` | `PORT-BP04-ASFP_MD_OUT` | `PORT-BP04-BBR_DOCS_INOUT` | String | dimensionless | [1, 1048576] bytes | Aperiodic [5, 50] ms | `"SPEC_PROJECTION_FAILED"` | `schema/model.sysml#L777` |
| `SIG-BP04-033` | `SpecDocsInOutSignal` | `PORT-BP04-BBR_DOCS_INOUT` | `PORT-BP04-PKG_AST_IN` | String | dimensionless | [1, 1048576] bytes | Aperiodic [5, 50] ms | `"RECONCILE_DRIFT_ERR"` | `schema/model.sysml#L789` |
| `SIG-BP04-034` | `SubpackageSyncOutSignal` | `PORT-BP04-SYNC_OUT` | `PORT-BP04-PKG_AST_IN` | String | dimensionless | [1, 2097152] bytes | Aperiodic [5, 50] ms | `"SUBPACKAGE_SYNC_ABORTED"` | `schema/model.sysml#L793` |
| `SIG-BP04-035` | `SysmlPackageAstInputSignal` | `PORT-BP04-SYNC_OUT` | `PORT-BP04-PKG_AST_IN` | String | dimensionless | [1, 2097152] bytes | Aperiodic [5, 50] ms | `"SUBPACKAGE_SYNC_ABORTED"` | `schema/model.sysml#L809` |
| `SIG-BP04-036` | `MarkdownSpecSuiteOutSignal` | `PORT-BP04-MD_SUITE_OUT` | `PORT-BP04-BBR_DOCS_INOUT` | String | dimensionless | [1, 2097152] bytes | Aperiodic [5, 50] ms | `"MARKDOWN_SUITE_ABORTED"` | `schema/model.sysml#L815` |
| `SIG-BP04-037` | `IcdMatrixArtifactOutSignal` | `PORT-BP04-ICD_MAT_OUT` | `PORT-BP09-ICD_AST_IN` | String | dimensionless | [1, 524288] bytes | Aperiodic [5, 50] ms | `"ICD_MATRIX_ABORTED"` | `schema/model.sysml#L821` |
| `SIG-BP04-038` | `WbsEnterpriseExportOutSignal` | `PORT-BP04-WBS_EXP_OUT` | `PORT-BP04-WBS_META_IN` | String | dimensionless | [1, 524288] bytes | Aperiodic [5, 50] ms | `"WBS_EXPORT_ABORTED"` | `schema/model.sysml#L827` |
| `SIG-BP04-039` | `InterconnectAstInSignal` | `PORT-BP04-FWD_PROJ_DEF` | `PORT-BP04-LIMC_AST_IN` | String | dimensionless | [1, 524288] bytes | Aperiodic [5, 50] ms | `"EMPTY_INTERCONNECT_AST"` | `schema/model.sysml#L839` |
| `SIG-BP04-040` | `IcdMarkdownOutSignal` | `PORT-BP04-LIMC_ICD_OUT` | `PORT-BP09-ICD_AST_IN` | String | dimensionless | [1, 524288] bytes | Aperiodic [5, 50] ms | `"ICD_SYNTHESIS_FAILED"` | `schema/model.sysml#L845` |
| `SIG-BP04-041` | `WbsAstMetadataInSignal` | `PORT-BP04-WBS_EXP_OUT` | `PORT-BP04-WBS_META_IN` | String | dimensionless | [1, 524288] bytes | Aperiodic [5, 50] ms | `"EMPTY_WBS_METADATA"` | `schema/model.sysml#L857` |
| `SIG-BP04-042` | `WbsArtifactsOutSignal` | `PORT-BP04-WBS_ART_OUT` | `PORT-BP04-BBR_DOCS_INOUT` | String | dimensionless | [1, 524288] bytes | Aperiodic [5, 50] ms | `"WBS_ARTIFACTS_ABORTED"` | `schema/model.sysml#L863` |
| `SIG-BP05-043` | `Bp05SafetyDocstringPortFlow` | `PORT-BP05-DOCSTRING_DEF` | `PORT-BP05-CTX_IN` | Record | dimensionless | [false, true] | Aperiodic [1, 20] ms | `false` | `schema/model.sysml#L873` |
| `SIG-BP05-044` | `Bp05ContextToDocstringFlow` | `PORT-BP05-CTX_OUT` | `PORT-BP05-CTX_IN` | String | dimensionless | [1, 524288] bytes | Aperiodic [1, 20] ms | `"UNRESOLVED_SCHEMA_CTX"` | `schema/model.sysml#L922` |
| `SIG-BP05-045` | `ResolvedSchemaContextOutSignal` | `PORT-BP05-CTX_OUT` | `PORT-BP05-CTX_IN` | String | dimensionless | [1, 524288] bytes | Aperiodic [1, 20] ms | `"UNRESOLVED_SCHEMA_CTX"` | `schema/model.sysml#L1002` |
| `SIG-BP05-046` | `SchemaContextInSignal` | `PORT-BP05-CTX_OUT` | `PORT-BP05-CTX_IN` | String | dimensionless | [1, 524288] bytes | Aperiodic [1, 20] ms | `"UNRESOLVED_SCHEMA_CTX"` | `schema/model.sysml#L1008` |
| `SIG-BP06-047` | `Bp06DefectDossierPortFlow` | `PORT-BP06-DOSSIER_DEF` | `PORT-BP06-DOSSIER_IN` | Record | dimensionless | [1, 262144] bytes | Aperiodic [2, 30] ms | `"DOSSIER_CHECK_FAILED"` | `schema/model.sysml#L1021` |
| `SIG-BP06-048` | `Bp06DossierToPublisherFlow` | `PORT-BP06-DOSSIER_OUT` | `PORT-BP06-DOSSIER_IN` | String | dimensionless | [1, 262144] bytes | Aperiodic [2, 30] ms | `"INVALID_7_SECTION_SCHEMA"` | `schema/model.sysml#L1095` |
| `SIG-BP06-049` | `DossierCandidateInSignal` | `PORT-BP06-DOSSIER_OUT` | `PORT-BP06-DOSSIER_IN` | String | dimensionless | [1, 262144] bytes | Aperiodic [2, 30] ms | `"INVALID_7_SECTION_SCHEMA"` | `schema/model.sysml#L1189` |
| `SIG-BP06-050` | `VerifiedReceiptOutSignal` | `PORT-BP06-RECEIPT_OUT` | `PORT-BP09-GATE_IN` | String | dimensionless | [1, 65536] bytes | Aperiodic [2, 30] ms | `"PUBLISH_UNVERIFIED_ERR"` | `schema/model.sysml#L1195` |
| `SIG-BP06-051` | `FindingInPortSignal` | `PORT-BP06-FINDING_OUT` | `PORT-BP06-FINDING_IN` | String | dimensionless | [1, 262144] bytes | 100 Hz | `"PILLAR_AUDIT_FINDING"` | `schema/model.sysml#L1207` |
| `SIG-BP06-052` | `DossierMarkdownOutSignal` | `PORT-BP06-DOSSIER_OUT` | `PORT-BP06-DOSSIER_IN` | String | dimensionless | [1, 262144] bytes | Aperiodic [2, 30] ms | `"INVALID_7_SECTION_SCHEMA"` | `schema/model.sysml#L1213` |
| `SIG-BP06-053` | `TargetRustCstInputSignal` | `PORT-BP06-DOSSIER_DEF` | `PORT-BP06-TARGET_CST_IN` | String | dimensionless | [1, 1048576] bytes | 100 Hz | `"EMPTY_TARGET_CST"` | `schema/model.sysml#L1229` |
| `SIG-BP06-054` | `GitHubIssueSsotQuerySignal` | `PORT-BP06-DOSSIER_DEF` | `PORT-BP06-GH_SSOT_IN` | String | dimensionless | [1, 65536] bytes | Aperiodic [2, 30] ms | `"TRACKER_QUERY_FAILED"` | `schema/model.sysml#L1235` |
| `SIG-BP06-055` | `DefectDossierArtifactSignal` | `PORT-BP06-DOSSIER_ART_OUT` | `PORT-BP06-DOSSIER_IN` | String | dimensionless | [1, 262144] bytes | Aperiodic [2, 30] ms | `"DOSSIER_WRITE_ABORTED"` | `schema/model.sysml#L1241` |
| `SIG-BP06-056` | `ClosedLoopTrackerPublishSignal` | `PORT-BP06-TRACKER_PUB_OUT` | `PORT-BP06-DOSSIER_IN` | String | dimensionless | [1, 65536] bytes | Aperiodic [2, 30] ms | `"TRACKER_PUBLISH_ABORTED"` | `schema/model.sysml#L1247` |
| `SIG-BP06-057` | `FivePillarAstStreamInSignal` | `PORT-BP06-DOSSIER_DEF` | `PORT-BP06-AST_STREAM_IN` | String | dimensionless | [1, 1048576] bytes | 100 Hz | `"EMPTY_AST_STREAM"` | `schema/model.sysml#L1259` |
| `SIG-BP06-058` | `FivePillarFindingOutSignal` | `PORT-BP06-FINDING_OUT` | `PORT-BP06-FINDING_IN` | String | dimensionless | [1, 262144] bytes | 100 Hz | `"PILLAR_VIOLATION_FOUND"` | `schema/model.sysml#L1265` |
| `SIG-BP06-059` | `UmlSpecInPortSignal` | `PORT-BP06-DOSSIER_DEF` | `PORT-BP06-UML_SPEC_IN` | String | dimensionless | [1, 131072] bytes | Aperiodic [2, 30] ms | `"EMPTY_UML_SPEC"` | `schema/model.sysml#L1277` |
| `SIG-BP06-060` | `MermaidBlockOutPortSignal` | `PORT-BP06-MERMAID_OUT` | `PORT-BP06-DOSSIER_IN` | String | dimensionless | [1, 131072] bytes | Aperiodic [2, 30] ms | `"MERMAID_E1_E5_VIOLATION"` | `schema/model.sysml#L1283` |
| `SIG-BP07-061` | `Bp07ModularCodegenPortFlow` | `PORT-BP07-CODEGEN_DEF` | `PORT-BP07-VAL_AST_IN` | Record | dimensionless | [1, 2097152] bytes | Aperiodic [5, 40] ms | `"CODEGEN_CYCLE_E0702"` | `schema/model.sysml#L1323` |
| `SIG-BP07-062` | `Bp07PartitionerToRustEmitterFlow` | `PORT-BP07-GRAPH_OUT` | `PORT-BP07-GRAPH_IN` | String | dimensionless | [1, 2097152] bytes | Aperiodic [5, 40] ms | `"UPWARD_LAYER_EDGE_E0703"` | `schema/model.sysml#L1426` |
| `SIG-BP07-063` | `ValidatedAstInSignal` | `PORT-BP07-CODEGEN_DEF` | `PORT-BP07-VAL_AST_IN` | String | dimensionless | [1, 2097152] bytes | Aperiodic [5, 40] ms | `"INVALID_AST_INPUT"` | `schema/model.sysml#L1521` |
| `SIG-BP07-064` | `PartitionedModuleGraphOutSignal` | `PORT-BP07-GRAPH_OUT` | `PORT-BP07-GRAPH_IN` | String | dimensionless | [1, 2097152] bytes | Aperiodic [5, 40] ms | `"MODULE_SLOC_EXCEEDED"` | `schema/model.sysml#L1525` |
| `SIG-BP07-065` | `FlutterModuleGraphInSignal` | `PORT-BP07-GRAPH_OUT` | `PORT-BP07-FLUTTER_GRAPH_IN` | String | dimensionless | [1, 2097152] bytes | Aperiodic [5, 40] ms | `"FLUTTER_GRAPH_INVALID"` | `schema/model.sysml#L1535` |
| `SIG-BP07-066` | `FlutterManifestOutSignal` | `PORT-BP07-FLUTTER_MAN_OUT` | `PORT-BP09-ICD_AST_IN` | String | dimensionless | [64, 65536] bytes | Aperiodic [5, 40] ms | `"FLUTTER_EMIT_ABORTED"` | `schema/model.sysml#L1539` |
| `SIG-BP07-067` | `RustModuleGraphInSignal` | `PORT-BP07-GRAPH_OUT` | `PORT-BP07-GRAPH_IN` | String | dimensionless | [1, 2097152] bytes | Aperiodic [5, 40] ms | `"RUST_GRAPH_INVALID"` | `schema/model.sysml#L1555` |
| `SIG-BP07-068` | `RustManifestOutSignal` | `PORT-BP07-RUST_MAN_OUT` | `PORT-BP08-LOWERED_IR_IN` | String | dimensionless | [64, 65536] bytes | Aperiodic [5, 40] ms | `"FIXED_POINT_DRIFT_E0705"` | `schema/model.sysml#L1559` |
| `SIG-BP07-069` | `SelfHostingPackageAstInSignal` | `PORT-BP07-CODEGEN_DEF` | `PORT-BP07-PKG_AST_IN` | String | dimensionless | [1, 2097152] bytes | Aperiodic [5, 40] ms | `"EMPTY_CODEGEN_PACKAGE"` | `schema/model.sysml#L1579` |
| `SIG-BP07-070` | `SelfHostingRustTreeOutSignal` | `PORT-BP07-RUST_TREE_OUT` | `PORT-BP08-LOWERED_IR_IN` | String | dimensionless | [1, 2097152] bytes | Aperiodic [5, 40] ms | `"RUST_TREE_ROLLBACK"` | `schema/model.sysml#L1583` |
| `SIG-BP07-071` | `SelfHostingFlutterTreeOutSignal` | `PORT-BP07-FLUTTER_TREE_OUT` | `PORT-BP09-ICD_AST_IN` | String | dimensionless | [1, 2097152] bytes | Aperiodic [5, 40] ms | `"FLUTTER_TREE_ROLLBACK"` | `schema/model.sysml#L1587` |
| `SIG-BP07-072` | `SelfHostingCodegenDiagSignal` | `PORT-BP07-CODEGEN_DIAG_OUT` | `PORT-BP08-FINDING_IN` | String | dimensionless | [0, 262144] bytes | Aperiodic [5, 40] ms | `"CODEGEN_BOUNDS_ERR_E0704"` | `schema/model.sysml#L1591` |
| `SIG-BP07-073` | `DependencyEdgesInSignal` | `PORT-BP07-CODEGEN_DEF` | `PORT-BP07-DEP_EDGES_IN` | String | dimensionless | [1, 524288] bytes | Aperiodic [5, 40] ms | `"DAG_CYCLE_DETECTED_E0702"` | `schema/model.sysml#L1601` |
| `SIG-BP07-074` | `TopologicalOrderOutSignal` | `PORT-BP07-TOPO_ORDER_OUT` | `PORT-BP07-VAL_AST_IN` | String | dimensionless | [1, 524288] bytes | Aperiodic [5, 40] ms | `"DAG_CYCLE_DETECTED_E0702"` | `schema/model.sysml#L1605` |
| `SIG-BP08-075` | `Bp08LosslessRustCstPortFlow` | `PORT-BP08-RUST_CST_DEF` | `PORT-BP08-RUST_SRC_IN` | Record | dimensionless | [1, 2097152] bytes | Aperiodic [1, 15] ms | `"CST_CODEMOD_REJECTED"` | `schema/model.sysml#L1657` |
| `SIG-BP08-076` | `Bp08CodemodToDiagEmitterFlow` | `PORT-BP08-ERR_OUT` | `PORT-BP08-FINDING_IN` | String | dimensionless | [0, 262144] bytes | Aperiodic [1, 15] ms | `"OVERLAPPING_TEXT_EDIT_ERR"` | `schema/model.sysml#L1758` |
| `SIG-BP08-077` | `ManifestLockInSignal` | `PORT-BP08-RUST_CST_DEF` | `PORT-BP08-LOCK_IN` | String | dimensionless | [1, 524288] bytes | Aperiodic [1, 15] ms | `"UNLOCKED_CARGO_MANIFEST"` | `schema/model.sysml#L1844` |
| `SIG-BP08-078` | `LockReceiptOutSignal` | `PORT-BP08-LOCK_OUT` | `PORT-BP09-WS_MAN_IN` | String | dimensionless | [1, 65536] bytes | Aperiodic [1, 15] ms | `"CARGO_LOCK_DRIFT_ERR"` | `schema/model.sysml#L1848` |
| `SIG-BP08-079` | `LosslessRustSourceInSignal` | `PORT-BP08-RUST_CST_DEF` | `PORT-BP08-RUST_SRC_IN` | String | dimensionless | [1, 1048576] bytes | 100 Hz | `"EMPTY_RUST_SOURCE"` | `schema/model.sysml#L1866` |
| `SIG-BP08-080` | `RustCstItemInventoryOutSignal` | `PORT-BP08-ITEM_INV_OUT` | `PORT-BP08-CODEMOD_CMD_IN` | String | dimensionless | [1, 524288] bytes | Aperiodic [1, 15] ms | `"EMPTY_ITEM_INVENTORY"` | `schema/model.sysml#L1870` |
| `SIG-BP08-081` | `RustCstMetricsReportOutSignal` | `PORT-BP08-METRICS_OUT` | `PORT-BP08-FINDING_IN` | String | dimensionless | [1, 262144] bytes | Aperiodic [1, 15] ms | `"SLOC_CAP_EXCEEDED_380"` | `schema/model.sysml#L1874` |
| `SIG-BP08-082` | `LoweredIrInSignal` | `PORT-BP07-RUST_TREE_OUT` | `PORT-BP08-LOWERED_IR_IN` | String | dimensionless | [1, 2097152] bytes | Aperiodic [1, 15] ms | `"INVALID_SYN_TOKEN_STREAM"` | `schema/model.sysml#L1884` |
| `SIG-BP08-083` | `FormattedRustOutSignal` | `PORT-BP08-FMT_RUST_OUT` | `PORT-BP09-MIR_GRAPH_IN` | String | dimensionless | [1, 2097152] bytes | Aperiodic [1, 15] ms | `"PRETTYPLEASE_PARSE_ERR"` | `schema/model.sysml#L1888` |
| `SIG-BP08-084` | `DiagnosticFindingInSignal` | `PORT-BP08-ERR_OUT` | `PORT-BP08-FINDING_IN` | String | dimensionless | [0, 262144] bytes | 100 Hz | `"DIAGNOSTIC_FINDING_ERR"` | `schema/model.sysml#L1902` |
| `SIG-BP08-085` | `RustcCompatibleNdjsonOutSignal` | `PORT-BP08-NDJSON_OUT` | `PORT-BP09-GATE_IN` | String | dimensionless | [1, 524288] bytes | 100 Hz | `"{\"$message_type\":\"diagnostic\",\"level\":\"error\"}"` | `schema/model.sysml#L1906` |
| `SIG-BP08-086` | `MachineApplicableFixSignal` | `PORT-BP08-MACHINE_FIX_INOUT` | `PORT-BP08-CODEMOD_CMD_IN` | String | dimensionless | [1, 262144] bytes | Aperiodic [1, 15] ms | `"REJECT_UNSAFE_REPLACEMENT"` | `schema/model.sysml#L1910` |
| `SIG-BP08-087` | `CodemodCommandInSignal` | `PORT-BP08-ITEM_INV_OUT` | `PORT-BP08-CODEMOD_CMD_IN` | String | dimensionless | [1, 65536] bytes | Aperiodic [1, 15] ms | `"ABORT_SPAN_CODEMOD"` | `schema/model.sysml#L1922` |
| `SIG-BP08-088` | `AtomicPatchOutSignal` | `PORT-BP08-ATOMIC_PATCH_OUT` | `PORT-BP08-RUST_SRC_IN` | String | dimensionless | [1, 1048576] bytes | Aperiodic [1, 15] ms | `"ROLLBACK_PATCH_BUFFER"` | `schema/model.sysml#L1926` |
| `SIG-BP08-089` | `CodemodErrorOutSignal` | `PORT-BP08-ERR_OUT` | `PORT-BP08-FINDING_IN` | String | dimensionless | [0, 262144] bytes | Aperiodic [1, 15] ms | `"OVERLAPPING_TEXT_EDIT_ERR"` | `schema/model.sysml#L1930` |
| `SIG-BP09-090` | `Bp09FormalVerificationPortFlow` | `PORT-BP09-FORMAL_VER_DEF` | `PORT-BP09-GATE_IN` | Record | dimensionless | [1, 1048576] bytes | Aperiodic [2, 25] ms | `"FORMAL_GATE_REJECTED"` | `schema/model.sysml#L1962` |
| `SIG-BP09-091` | `Bp09ClippyToOrchestratorFlow` | `PORT-BP09-CLIPPY_OUT` | `PORT-BP09-GATE_IN` | String | dimensionless | [1, 524288] bytes | 100 Hz | `"CLIPPY_TOTAL_FN_VIOLATION"` | `schema/model.sysml#L2076` |
| `SIG-BP09-092` | `SysmlIcdAstInSignal` | `PORT-BP04-LIMC_ICD_OUT` | `PORT-BP09-ICD_AST_IN` | String | dimensionless | [1, 524288] bytes | Aperiodic [2, 25] ms | `"INVALID_ICD_BOUNDS"` | `schema/model.sysml#L2171` |
| `SIG-BP09-093` | `FrbFacadeAndDartOutSignal` | `PORT-BP09-FRB_DART_OUT` | `PORT-BP11-ASSURANCE_AST_IN` | String | dimensionless | [1, 1048576] bytes | Aperiodic [2, 25] ms | `"FFI_BOUNDARY_GUARD_ERR"` | `schema/model.sysml#L2175` |
| `SIG-BP09-094` | `MiriAndDartReceiptOutSignal` | `PORT-BP09-MIRI_RECEIPT_OUT` | `PORT-BP09-GATE_IN` | String | dimensionless | [1, 131072] bytes | Aperiodic [2, 25] ms | `"MIRI_UB_DETECTED"` | `schema/model.sysml#L2179` |
| `SIG-BP09-095` | `MirBodyGraphInSignal` | `PORT-BP08-FMT_RUST_OUT` | `PORT-BP09-MIR_GRAPH_IN` | String | dimensionless | [1, 2097152] bytes | 100 Hz | `"EMPTY_MIR_BODY_GRAPH"` | `schema/model.sysml#L2193` |
| `SIG-BP09-096` | `DylintMirNdjsonOutSignal` | `PORT-BP09-DYLINT_OUT` | `PORT-BP09-GATE_IN` | String | dimensionless | [0, 524288] bytes | 100 Hz | `"RT_LOOP_HEAP_ALLOC_ERR"` | `schema/model.sysml#L2197` |
| `SIG-BP09-097` | `SysmlSafetyAstInSignal` | `PORT-BP09-FORMAL_VER_DEF` | `PORT-BP09-SAFETY_AST_IN` | String | dimensionless | [1, 1048576] bytes | Aperiodic [2, 25] ms | `"MISSING_SAFETY_INVARIANT"` | `schema/model.sysml#L2209` |
| `SIG-BP09-098` | `KaniProofModuleOutSignal` | `PORT-BP09-KANI_MOD_OUT` | `PORT-BP09-GATE_IN` | String | dimensionless | [1, 524288] bytes | Aperiodic [2, 25] ms | `"KANI_HARNESS_SYNTH_ERR"` | `schema/model.sysml#L2213` |
| `SIG-BP09-099` | `CbmcProofReceiptOutSignal` | `PORT-BP09-CBMC_RECEIPT_OUT` | `PORT-BP09-GATE_IN` | String | dimensionless | [1, 131072] bytes | Aperiodic [2, 25] ms | `"KANI_COUNTEREXAMPLE_FOUND"` | `schema/model.sysml#L2217` |
| `SIG-BP09-100` | `GateInvocationInSignal` | `PORT-BP09-CLIPPY_OUT` | `PORT-BP09-GATE_IN` | String | dimensionless | [1, 524288] bytes | 100 Hz | `"GATE_STAGE_ABORTED"` | `schema/model.sysml#L2229` |
| `SIG-BP09-101` | `UnifiedGateNdjsonOutSignal` | `PORT-BP09-UNIFIED_NDJSON_OUT` | `PORT-BP11-ASSURANCE_AST_IN` | String | dimensionless | [1, 1048576] bytes | 100 Hz | `"STAGE_VERIFICATION_FAILED"` | `schema/model.sysml#L2233` |
| `SIG-BP09-102` | `WorkspaceManifestInSignal` | `PORT-BP08-LOCK_OUT` | `PORT-BP09-WS_MAN_IN` | String | dimensionless | [1, 65536] bytes | Aperiodic [2, 25] ms | `"CLIPPY_POLICY_DRIFT"` | `schema/model.sysml#L2251` |
| `SIG-BP09-103` | `ClippyNdjsonOutSignal` | `PORT-BP09-CLIPPY_OUT` | `PORT-BP09-GATE_IN` | String | dimensionless | [1, 524288] bytes | 100 Hz | `"CLIPPY_TOTAL_FN_VIOLATION"` | `schema/model.sysml#L2255` |
| `SIG-BP10-104` | `Bp10IncrementalCstPortFlow` | `PORT-BP10-INC_CST_DEF` | `PORT-BP10-SYSML_SRC_IN` | Record | dimensionless | [1, 2097152] bytes | Aperiodic [1, 20] ms | `"LOSSLESS_CST_DRIFT_ERR"` | `schema/model.sysml#L2291` |
| `SIG-BP10-105` | `Bp10PolyglotToSalsaDbFlow` | `PORT-BP10-HIR_OUT` | `PORT-BP10-REV_IN` | String | dimensionless | [1, 2097152] bytes | Aperiodic [1, 20] ms | `"POLYGLOT_LOWERING_ERR"` | `schema/model.sysml#L2403` |
| `SIG-BP10-106` | `SyncDeltaInSignal` | `PORT-BP10-HIR_MODEL_OUT` | `PORT-BP10-SYNC_DELTA_IN` | String | dimensionless | [1, 524288] bytes | Aperiodic [1, 20] ms | `"EMPTY_SYNC_DELTA"` | `schema/model.sysml#L2493` |
| `SIG-BP10-107` | `SplicedArtifactOutSignal` | `PORT-BP10-SPLICED_ART_OUT` | `PORT-BP04-PKG_AST_IN` | String | dimensionless | [1, 2097152] bytes | Aperiodic [1, 20] ms | `"TRIVIA_MUTATION_ABORT"` | `schema/model.sysml#L2497` |
| `SIG-BP10-108` | `SpliceDiagnosticOutSignal` | `PORT-BP10-SPLICE_DIAG_OUT` | `PORT-BP08-FINDING_IN` | String | dimensionless | [0, 262144] bytes | Aperiodic [1, 20] ms | `"SPAN_SPLICE_OVERLAP_ERR"` | `schema/model.sysml#L2501` |
| `SIG-BP10-109` | `SysmlSourceInSignal` | `PORT-BP10-INC_CST_DEF` | `PORT-BP10-SYSML_SRC_IN` | String | dimensionless | [1, 2097152] bytes | 100 Hz | `"EMPTY_SYSML_BUFFER"` | `schema/model.sysml#L2515` |
| `SIG-BP10-110` | `GreenCstOutSignal` | `PORT-BP10-GREEN_CST_OUT` | `PORT-BP10-REV_IN` | String | dimensionless | [1, 2097152] bytes | Aperiodic [1, 20] ms | `"ROWAN_GREEN_NODE_ERR"` | `schema/model.sysml#L2519` |
| `SIG-BP10-111` | `SyntaxDiagnosticOutSignal` | `PORT-BP10-SYNTAX_DIAG_OUT` | `PORT-BP08-FINDING_IN` | String | dimensionless | [0, 262144] bytes | Aperiodic [1, 20] ms | `"ARIADNE_SYNTAX_ERROR"` | `schema/model.sysml#L2523` |
| `SIG-BP10-112` | `SalsaSourceRevisionInSignal` | `PORT-BP10-HIR_OUT` | `PORT-BP10-REV_IN` | String | dimensionless | [1, 2097152] bytes | Aperiodic [1, 20] ms | `"SALSA_REVISION_ABORTED"` | `schema/model.sysml#L2537` |
| `SIG-BP10-113` | `SalsaHirModelOutSignal` | `PORT-BP10-HIR_MODEL_OUT` | `PORT-BP10-SYNC_DELTA_IN` | String | dimensionless | [1, 2097152] bytes | Aperiodic [1, 20] ms | `"HIR_EARLY_CUTOFF_MISS"` | `schema/model.sysml#L2541` |
| `SIG-BP10-114` | `SalsaSafetyVerificationOutSignal` | `PORT-BP10-SAFETY_VER_OUT` | `PORT-BP09-SAFETY_AST_IN` | String | dimensionless | [1, 524288] bytes | Aperiodic [1, 20] ms | `"SAFETY_QUERY_FAILED"` | `schema/model.sysml#L2545` |
| `SIG-BP10-115` | `PolyglotSourceInSignal` | `PORT-BP10-INC_CST_DEF` | `PORT-BP10-POLYGLOT_SRC_IN` | String | dimensionless | [1, 2097152] bytes | 100 Hz | `"UNSUPPORTED_SCHEMA_KIND"` | `schema/model.sysml#L2559` |
| `SIG-BP10-116` | `LoweredHirOutSignal` | `PORT-BP10-HIR_OUT` | `PORT-BP10-REV_IN` | String | dimensionless | [1, 2097152] bytes | Aperiodic [1, 20] ms | `"POLYGLOT_LOWERING_ERR"` | `schema/model.sysml#L2563` |
| `SIG-BP10-117` | `IngestDiagnosticOutSignal` | `PORT-BP10-INGEST_DIAG_OUT` | `PORT-BP08-FINDING_IN` | String | dimensionless | [0, 262144] bytes | Aperiodic [1, 20] ms | `"POLYGLOT_INGEST_ERR"` | `schema/model.sysml#L2567` |
| `SIG-BP11-118` | `Bp11FerroceneAssurancePortFlow` | `PORT-BP11-ASSURANCE_DEF` | `PORT-BP11-ASSURANCE_AST_IN` | Record | dimensionless | [1, 1048576] bytes | Aperiodic [2, 30] ms | `"FERROCENE_QUAL_FAILED"` | `schema/model.sysml#L2599` |
| `SIG-BP11-119` | `Bp11PartitionToFlsFlow` | `PORT-BP11-PART_OUT` | `PORT-BP11-MOD_GRAPH_IN` | String | dimensionless | [1, 2097152] bytes | Aperiodic [2, 30] ms | `"RING_PARTITION_VIOLATION"` | `schema/model.sysml#L2717` |
| `SIG-BP11-120` | `ElfAndProfdataInSignal` | `PORT-BP11-ASSURANCE_DEF` | `PORT-BP11-ELF_PROF_IN` | String | dimensionless | [1, 2097152] bytes | Aperiodic [2, 30] ms | `"MISSING_PROFDATA_STREAM"` | `schema/model.sysml#L2806` |
| `SIG-BP11-121` | `TableA7EvidenceOutSignal` | `PORT-BP11-TABLE_A7_OUT` | `PORT-BP11-ASSURANCE_DEF` | String | dimensionless | [1, 524288] bytes | Aperiodic [2, 30] ms | `"MCDC_COVERAGE_ DEFICIT"` | `schema/model.sysml#L2810` |
| `SIG-BP11-122` | `CoverageDiagnosticOutSignal` | `PORT-BP11-COV_DIAG_OUT` | `PORT-BP08-FINDING_IN` | String | dimensionless | [0, 262144] bytes | Aperiodic [2, 30] ms | `"CERTIFIED_CORE_PANIC_SYM"` | `schema/model.sysml#L2814` |
| `SIG-BP11-123` | `ToolchainManifestsInSignal` | `PORT-BP11-ASSURANCE_DEF` | `PORT-BP11-TC_MAN_IN` | String | dimensionless | [1, 65536] bytes | Aperiodic [2, 30] ms | `"MISSING_CRITICALUP_TOML"` | `schema/model.sysml#L2830` |
| `SIG-BP11-124` | `QualificationDossierOutSignal` | `PORT-BP11-QUAL_DOSSIER_OUT` | `PORT-BP11-ASSURANCE_DEF` | String | dimensionless | [1, 524288] bytes | Aperiodic [2, 30] ms | `"DO_330_TQL_1_UNVERIFIED"` | `schema/model.sysml#L2834` |
| `SIG-BP11-125` | `ToolchainDiagnosticOutSignal` | `PORT-BP11-TC_DIAG_OUT` | `PORT-BP08-FINDING_IN` | String | dimensionless | [0, 262144] bytes | Aperiodic [2, 30] ms | `"TOOLCHAIN_DRIFT_DETECTED"` | `schema/model.sysml#L2838` |
| `SIG-BP11-126` | `LoweredModuleGraphInSignal` | `PORT-BP11-PART_OUT` | `PORT-BP11-MOD_GRAPH_IN` | String | dimensionless | [1, 2097152] bytes | Aperiodic [2, 30] ms | `"UNPARTITIONED_MODULE_ERR"` | `schema/model.sysml#L2850` |
| `SIG-BP11-127` | `FlsAnnotatedUnitsOutSignal` | `PORT-BP11-FLS_UNITS_OUT` | `PORT-BP11-ELF_PROF_IN` | String | dimensionless | [1, 2097152] bytes | Aperiodic [2, 30] ms | `"MISSING_FLS_PARAGRAPH_ID"` | `schema/model.sysml#L2854` |
| `SIG-BP11-128` | `FlsDiagnosticOutSignal` | `PORT-BP11-FLS_DIAG_OUT` | `PORT-BP08-FINDING_IN` | String | dimensionless | [0, 262144] bytes | Aperiodic [2, 30] ms | `"ORPHAN_FLS_TAG_DETECTED"` | `schema/model.sysml#L2858` |
| `SIG-BP11-129` | `SysmlAssuranceAstInSignal` | `PORT-BP09-UNIFIED_NDJSON_OUT` | `PORT-BP11-ASSURANCE_AST_IN` | String | dimensionless | [1, 1048576] bytes | Aperiodic [2, 30] ms | `"MISSING_DAL_METADATA"` | `schema/model.sysml#L2872` |
| `SIG-BP11-130` | `PartitionedWorkspaceOutSignal` | `PORT-BP11-PART_OUT` | `PORT-BP11-MOD_GRAPH_IN` | String | dimensionless | [1, 2097152] bytes | Aperiodic [2, 30] ms | `"RING1_FORBID_VIOLATION"` | `schema/model.sysml#L2876` |
| `SIG-BP11-131` | `PartitionDiagnosticOutSignal` | `PORT-BP11-PART_DIAG_OUT` | `PORT-BP08-FINDING_IN` | String | dimensionless | [0, 262144] bytes | Aperiodic [2, 30] ms | `"UPWARD_DAL_CALL_DETECTED"` | `schema/model.sysml#L2880` |

## 3. Mathematical Formulations & Safety Invariants

$$
\begin{aligned}
\tau_{\mathrm{transport}} &\le \tau_{\mathrm{max}} \\
\Delta_{\mathrm{span}}(t_k) &= b_{\mathrm{cst}}(t_k) - b_{\mathrm{raw}}(t_k) = 0 \\
d_{\mathrm{AST}}\left(\mathcal{A}, \mathrm{parse}(\mathrm{serialize}(\mathcal{A}))\right) &= 0 \\
V_{\mathrm{total}}(W) &= E_{\mathrm{forbid}}(W) + C_{\mathrm{purity}}(W) + C_{\mathrm{mir}}(W) + C_{\mathrm{ub}}(W) + C_{\mathrm{kani}}(W) = 0 \\
\Omega_{\mathrm{coverage}} &= \frac{\lvert \Sigma_{\mathrm{mapped}} \rvert}{\lvert \Sigma_{\mathrm{schema}} \rvert} = 1.0
\end{aligned}
$$

- Parameter Definitions & Engineering Units:
- $\tau_{\mathrm{transport}}$: End-to-end logical signal transport duration between source port and destination port governed by `LogicalIcdMatrixCompiler.maxIcdLatencyMs` in `schema/model.sysml` pursuant to ISO/IEC/IEEE 15288:2023 (ms).
- $\tau_{\mathrm{max}}$: Maximum allowable connection transport threshold bounded by `LogicalIcdMatrixCompiler.maxIcdLatencyMs` (`10.0 ms`) and `AgileSpecForwardProjector.maxForwardLatencyMs` (`50.0 ms`) in `schema/model.sysml` pursuant to ISO/IEC/IEEE 15288:2023 (ms).
- $\Delta_{\mathrm{span}}(t_k)$: Lexical and CST byte-offset drift for token $t_k$ across `logos` + `rowan` + `ra_ap_syntax` lossless syntax trees (`NativeSigilAndLifetimeLexer`, required `0 bytes`).
- $b_{\mathrm{cst}}(t_k)$: Reconstructed byte offset of token $t_k$ in the lossless green/red CST (`LogosRowanSysmlCstEngine`, bytes).
- $b_{\mathrm{raw}}(t_k)$: Ground-truth UTF-8 byte offset of token $t_k$ in the source buffer (`NativeSigilAndLifetimeLexer`, bytes).
- $d_{\mathrm{AST}}(\mathcal{A}, \mathrm{parse}(\mathrm{serialize}(\mathcal{A})))$: Structural AST edit distance across OMG SysML v2.0 + KerML 1.0 bijective roundtrip serialization (`SysmlBijectiveRoundtripCompilerGate`, dimensionless, required `0`).
- $V_{\mathrm{total}}(W)$: Total safety-critical gate violations across workspace $W$ summing Stage 1 `forbid` lint violations $E_{\mathrm{forbid}}(W)$, Check 34 CST purity violations $C_{\mathrm{purity}}(W)$, Stage 2 Dylint MIR heap allocations $C_{\mathrm{mir}}(W)$, Stage 3 Miri UB violations $C_{\mathrm{ub}}(W)$, and Stage 4 Kani counterexamples $C_{\mathrm{kani}}(W)$ (`MultiStageVerificationOrchestrator`, dimensionless count, required `0`).
- $\Omega_{\mathrm{coverage}}$: Ratio of mapped signal dictionary leaves $\lvert \Sigma_{\mathrm{mapped}} \rvert$ to total declared schema interface leaves $\lvert \Sigma_{\mathrm{schema}} \rvert$ (`LogicalIcdMatrixCompiler`, dimensionless ratio, required `1.0`).

## 4. Safety-Critical Signal Allocation & STPA Hazard Mapping

| Signal ID | Safety Criticality | Hazard Ref | Safety Constraint | Failsafe Action |
| :--- | :--- | :--- | :--- | :--- |
| `SIG-BP01-002` | Critical (DAL-A) | **H-1**, **UCA-001** | **SC-001**: Zero `allow(clippy::...)` suppression attributes permitted across workspace manifests or CSTs | Emit `error[E0453]` diagnostic, assert `BaselineGateVerdictSignal = false`, and halt compilation fail-closed |
| `SIG-BP02-016` | Critical (DAL-A) | **H-6**, **H-10** | **SC-002**: Zero panicking range slices, fail-open I/O calls, or vacuous `test` function bodies permitted in `ra_ap_syntax` CST | Emit sorted `C_PURITY_VIOLATION_ERR` diagnostic, set `Check34GateStatusSignal = 1`, and abort verification |
| `SIG-BP03-023` | Critical (DAL-A) | **H-12**, **H-15** | **SC-003**: Lossless SysML v2.0 + KerML 1.0 CST/AST roundtrip must preserve zero byte drift and zero structural diff | Emit `ROUNDTRIP_MISMATCH_E0400` on `PORT-BP03-DIFF_DIAG_OUT` and reject unverified AST |
| `SIG-BP04-030` | High (DAL-B) | **H-18**, **H-29** | **SC-004**: Forward projection and subpackage-scoped reverse sync must preserve SHA-256 digest parity | Abort file emission and retain prior verified specification tree on digest mismatch |
| `SIG-BP05-044` | High (DAL-B) | **H-31**, **H-34** | **SC-005**: 100% of Rust (`ra_ap_syntax`) and Dart (`deap_core::dart_cst`) items must carry 7-section safety docstrings | Reject unannotated source items in Check 36 and assert `Bp05SafetyDocstringPortFlow = false` |
| `SIG-BP06-048` | High (DAL-B) | **H-37**, **H-42** | **SC-006**: Every 5-pillar static AST finding must synthesize a 12-check validated 7-section defect dossier | Block tracker publication on `PORT-BP06-DOSSIER_IN` and emit `INVALID_7_SECTION_SCHEMA` |
| `SIG-BP07-062` | Critical (DAL-A) | **H-45**, **H-52** | **SC-007**: 4-level modular codegen must enforce acyclic topological DAG, `< 380` SLOC per file, and Stage 1 == Stage 2 fixed point | Trigger atomic rollback of staging directory and emit `FIXED_POINT_DRIFT_E0705` |
| `SIG-BP08-076` | High (DAL-B) | **H-53**, **H-57** | **SC-008**: `ra_ap_syntax` + `ra_ap_text_edit` span codemods must be disjoint and parse cleanly in-memory before write | Reject overlapping `TextEdit` batch, preserve original file bytes, and emit `rustc` JSON error |
| `SIG-BP09-091` | Critical (DAL-A) | **H-59**, **H-65** | **SC-009**: All 5 verification stages (Clippy, Dylint MIR `rt_loop`, Miri + FRB, Kani CBMC) must pass with zero violations | Short-circuit downstream stages, emit unified NDJSON failure receipt, and fail closed |
| `SIG-BP10-105` | Critical (DAL-A) | **H-66**, **H-71** | **SC-010**: `logos` + `rowan` + `salsa` incremental polyglot compilation must preserve 100% trivia fidelity (`text() == raw_source`) | Reject lossy CST or overlapping splice, retain prior `salsa` revision, and emit `ariadne` report |
| `SIG-BP11-119` | Critical (DAL-A) | **H-73**, **H-78** | **SC-011**: 3-Ring DAL-A..E partitioning must enforce `no_std` + `forbid(unsafe_code)` in Ring 1 with 100% `FLS: fls_*` and MC/DC coverage | Halt release qualification, emit `RING1_FORBID_VIOLATION`, and invalidate Table A-7 receipt |

## 5. Source References
- System Architecture Model: [schema/model.sysml](../../schema/model.sysml)
- Modular Codegen Schema: [schema/modular_codegen.sysml](../../schema/modular_codegen.sysml)
- Safety & Hazard Analysis Matrix: [docs/safety/STPA_MATRIX.md](../safety/STPA_MATRIX.md)
- Normative Systems Engineering Standard: ISO/IEC/IEEE 15288:2023 §6.4.3 Architecture Definition Process

Subsystem architectural composition and formal invariants derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 15288.
