# Master Atomic Dispatch Queue -- Compiler Architecture & Specification Realization

## 1. Executive Architecture & Governance Directives

### 1.1 Contract A: Normative OMG SysML v2.0 + KerML 1.0 Grammar Coverage
All compiler frontend, ingestion, and specification projection passes SHALL enforce strict **Contract A** architectural compliance:
- **100% Normative Grammar Coverage Across All 6 Clauses**:
  - **Clause 7**: Root Packages, Namespaces, Imports, and Visibility (`package`, `library package`, `import`).
  - **Clause 8**: Structural Elements, Part Definitions, and Usages (`part def`, `part`, `attribute def`, `attribute`, `port def`, `port`, `connection def`, `connection`, `interface def`, `interface`).
  - **Clause 9**: Behavioral Elements, Action Definitions, and Usages (`action def`, `action`, `perform action`, `flow def`, `flow`, `item def`, `item`, `send`, `accept`).
  - **Clause 10**: State & Stateflow Transitions (`state def`, `state`, `entry action`, `do action`, `exit action`, `transition`, `first`, `then`, `accept`).
  - **Clause 11**: Requirements & Constraints (`requirement def`, `requirement`, `satisfy`, `verify`, `assert constraint`, `assume constraint`, `constraint def`, `constraint`).
  - **Clause 12**: Calculations, Libraries, Quantities, and Values (`calc def`, `calc`, `return`, `ISQ`, `SI`, `ScalarValues`).
- **Zero 16-Construct Heuristic Caps**: Eliminates all arbitrary token and construct counting limits; full recursive grammar parsing is enforced.
- **Definition/Usage Symmetry**: Every element definition (`* def`) pairs with symmetrical instance usage (`*`) syntax.
- **Safety & Metadata Bindings**: Unifies `#hazard`, `#risk`, `#capability`, `#rt_loop`, `#dal`, `#fls` via typed `metadata def` semantics alongside backwards-compatible stereotyping.

### 1.2 Contract B: Official rust-lang/* & rust-analyzer Crate Stack
All compiler crates under `crates/` SHALL enforce strict **Contract B** architectural compliance:
- **Mandatory Official Crate Stack**:
  - Cargo & Manifest Governance: `cargo_metadata`, `cargo-lock`, `cargo-util-schemas`, `cargo-platform`, `toml_edit`
  - Lossless CST & Analysis: `ra_ap_syntax`, `ra_ap_vfs`, `ra_ap_paths`, `ra_ap_line_index`, `ra_ap_text_edit`, `rowan`
  - Filesystem & Path Management: `camino`, `ignore`
  - Lexing & Incremental DB: `logos`, `salsa`
  - Diagnostics & CommonMark: `ariadne`, `pulldown-cmark`, `pulldown-cmark-to-cmark`
  - Typed Rust Codegen: `syn`, `proc-macro2`, `quote`, `prettyplease`
- **Zero Hand-Rolled Scanners**: Eliminates all ad-hoc line/character/string tokenizers, delimiter splitters, and string-concatenation (`format!`, `writeln!`) code generation templates.
- **5-Stage Formal Verification Pipeline**:
  - Stage 1: Stable Clippy Total Function Gate (`cargo clippy --workspace --all-targets --message-format=json`).
  - Stage 2: Isolated Dylint MIR Allocation Gate (`./lints/deap-dylint-mir/` inspecting `rustc_middle::mir::Body` and `TyCtxt`).
  - Stage 3: In-Process CST & Miri UB Gate (`flutter_rust_bridge` + `cargo +nightly miri test -Zmiri-tree-borrows`).
  - Stage 4: Kani CBMC Formal Proof Synthesizer (`cargo kani`).
  - Stage 5: Ferrocene Qualified Toolchain (`criticalup run cargo build`).

### 1.3 Submodule Modularity & Net-Negative Legacy-Scanner Deletion Ratchet
- **Modularity Ceiling**: Submodules under `crates/<crate>/src/<module>/<submodule>.rs` must strictly adhere to `< 380` SLOC (with `<= 500` SLOC hard fail-closed gate ceiling).
- **Explicit Net-Negative Deletion of ~19,700+ SLOC of Legacy Hand-Rolled Scanners**:
  - Monolithic legacy scanners permanently retired and deleted:
    - `crates/deap-core/src/source_ast.rs` (7,558 LOC)
    - `crates/compile-sysml/src/docstring.rs` (5,631 LOC)
    - `crates/verify-baseline/src/checks/docstring_gate.rs` (3,751 LOC)
    - `crates/verify-baseline/src/audit_code.rs` (3,214 LOC)
    - `crates/verify-baseline/src/checks/structural.rs` (4,200+ LOC)
    - `crates/deap-core/src/workspace.rs` (620+ LOC)
    - `crates/verify-baseline/src/ratchet_table.rs`
  - Ratchet table tracks downward convergence monotonically to `0` legacy lines.

### 1.4 Mandatory 3-Layer Definition of Done (DoD)
Every dispatch queue item must satisfy all three layers without exception (`N/A` is strictly forbidden):
1. **Layer 1: Domain State & Data Model**: Typed AST/CST/DTO definitions, zero mock data structures, fail-closed constructors.
2. **Layer 2: Logic & State Management**: Deterministic stateflow machines, linear Kahn DAG order, `< 380`-SLOC submodules.
3. **Layer 3: Presentation & Actuator Interface Binding**: CLI argument binding, machine-readable NDJSON diagnostics, VFS projection, reports.

### 1.5 Formatting, Path & Verification Invariants
- **Zero Em Dash Policy**: Unicode em dashes (`\u2014`) are strictly forbidden; use `--` or `-`.
- **Workspace-Relative Paths**: Only workspace-relative paths (`./...`) are permitted; zero hardcoded OS, user, machine, or home paths.
- **Fail-Closed Verification**: Validated on every stage via `./target/release/verify-baseline check-output`.

---

## 2. Master Atomic Dispatch Queue Matrix

| Wave | Blueprint ID | Scope & Features | Target Submodule Path (< 380 SLOC) | Contract B Mandated Official Crates | Ratchet Target (Legacy Scanner Deletion) | 3-Layer DoD Gate Criteria |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| Wave 1 | BP-01 (`CompilerLintEnforcementGate`) | EPIC-01, FEAT-01..05 | `crates/verify-baseline/src/checks/lint_forbid.rs`, `crates/verify-baseline/src/checks/manifest_gate.rs` | `toml_edit`, `cargo_metadata`, `ra_ap_syntax`, `rowan`, `camino`, `ignore` | Monolithic manifest line scanner in `structural.rs` (-980 LOC) | L1: `ClippyAttribute` AST; L2: `CompilerLintEnforcementGate` state machine; L3: CLI diagnostics with error code `E0453` |
| Wave 1 | BP-02 (`Check34AstPurityAndAntiGamingGate`) | EPIC-02, FEAT-06..10 | `crates/verify-baseline/src/checks/ast_purity.rs`, `crates/verify-baseline/src/checks/range_slice.rs` | `ra_ap_syntax`, `rowan`, `ra_ap_line_index`, `camino`, `ignore`, `cargo_metadata` | Hand-rolled string slicing scanners in `audit_code.rs` (-1,200 LOC) | L1: Typed `ast::IndexExpr` CST; L2: Fail-closed Check 34 gate; L3: File-and-line terminal diagnostics |
| Wave 1 | BP-08 (`LosslessRustCstAndJsonDiagnosticsEngine`) | EPIC-08, FEAT-36..40 | `crates/deap-core/src/rust_cst/mod.rs`, `crates/deap-core/src/rust_cst/parse.rs`, `crates/deap-core/src/json_diag/mod.rs` | `ra_ap_syntax`, `ra_ap_vfs`, `ra_ap_paths`, `ra_ap_line_index`, `ra_ap_text_edit`, `rowan`, `cargo_metadata`, `cargo-lock`, `cargo-util-schemas`, `cargo-platform`, `toml_edit`, `serde_json`, `camino`, `ignore` | Core hand-rolled scanner retirement across 6 crates (~19,700+ LOC ratchet) | L1: Lossless Red/Green CST; L2: Zero-token span-codemod engine; L3: Single-line rustc NDJSON emitter |
| Wave 2 | BP-03 (`SysML_v2_AST_Bijective_Roundtrip_Gate`) | EPIC-03, FEAT-11..15 | `crates/deap-core/src/sysml_cst/mod.rs`, `crates/compile-sysml/src/roundtrip/mod.rs` | `logos`, `rowan`, `ariadne`, `camino`, `ignore`, `pulldown-cmark` | Legacy recursive-descent scanner monoliths (-3,400 LOC) | L1: Clause 7-12 AST nodes with Definition/Usage symmetry; L2: Bijective roundtrip verification; L3: Ariadne span diagnostics |
| Wave 2 | BP-04 (`DeterministicSpecIcdWbsForwardCompiler`) | EPIC-04, FEAT-16..20 | `crates/compile-sysml/src/forward_projector/mod.rs`, `crates/reconcile-backlog/src/reconciler/mod.rs` | `pulldown-cmark`, `pulldown-cmark-to-cmark`, `toml_edit`, `salsa`, `camino`, `ignore` | Ad-hoc regex/hand-rolled Markdown table & HTML parsers (-1,800 LOC) | L1: Lossless CommonMark AST; L2: Bidirectional backlog reconciler; L3: File projections to `docs/` |
| Wave 2 | BP-10 (`IncrementalLosslessSysmlAndPolyglotCompilerEngine`) | EPIC-10, FEAT-46..49 | `crates/compile-sysml/src/salsa_db/mod.rs`, `crates/ingest-sysml/src/cst/mod.rs` | `logos`, `rowan`, `salsa`, `ariadne`, `ra_ap_syntax`, `serde_json`, `camino`, `ignore` | Ad-hoc IDL/YANG/OpenAPI scanners (-6,600+ LOC) | L1: Lossless Red/Green CST; L2: 4-layer Salsa incremental database; L3: Ariadne source-span diagnostic CLI |
| Wave 3 | BP-05 (`UnifiedSafetyDocstringSynthesizer`) | EPIC-05, FEAT-21..25 | `crates/compile-sysml/src/docstring_gen/mod.rs`, `crates/verify-baseline/src/checks/check_36.rs` | `ra_ap_syntax`, `rowan`, `ra_ap_line_index`, `ra_ap_text_edit`, `ra_ap_vfs`, `ra_ap_paths`, `camino`, `ignore` | Monolithic line-scanner docstring code across workspace (-8,200+ LOC) | L1: `ast::{HasDocComments, HasAttrs}`; L2: 7-section docstring synthesizer; L3: In-place `TextEdit` rewrites |
| Wave 3 | BP-06 (`DeterministicStaticDefectAuditor`) | EPIC-06, FEAT-26..30 | `crates/verify-baseline/src/auditor/mod.rs`, `crates/verify-baseline/src/dossier/mod.rs` | `ra_ap_syntax`, `rowan`, `pulldown-cmark`, `pulldown-cmark-to-cmark`, `serde_json`, `camino`, `ignore` | Ad-hoc substring security checkers in `audit_code.rs` (-1,500 LOC) | L1: 5-pillar defect AST nodes; L2: Static defect classification engine; L3: 7-section Markdown defect dossier |
| Wave 3 | BP-07 (`SelfHostingModularCodegenCompiler`) | EPIC-07, FEAT-31..35 | `crates/compile-sysml/src/modular_codegen/mod.rs`, `crates/compile-sysml/src/modular_codegen/rust_emitter.rs` | `syn`, `proc-macro2`, `quote`, `prettyplease`, `ra_ap_syntax`, `camino`, `ignore` | String-concatenation codegen templates (`format!`, `writeln!`) (-1,100+ LOC) | L1: Kahn DAG AST; L2: Stage-1 == Stage-2 SHA-256 fixed-point verification; L3: Multi-file canonical formatting |
| Wave 4 | BP-09 (`MultiStageFormalVerificationGatekeeper`) | EPIC-09, FEAT-41..45 | `crates/verify-baseline/src/formal_gate/mod.rs`, `lints/deap-dylint-mir/src/lib.rs` | `syn`, `proc-macro2`, `quote`, `prettyplease`, `serde_json`, `camino`, `ignore` | Brittle shell scripts and unverified lint checks (-850 LOC) | L1: Verification stage AST models; L2: 5-stage formal pipeline orchestrator; L3: rustc JSON diagnostics reporter |
| Wave 4 | BP-11 (`FerroceneThreeRingCertificationArchitecture`) | EPIC-11, FEAT-50..53 | `crates/compile-sysml/src/ferrocene/mod.rs`, `crates/verify-baseline/src/checks/ferrocene_gate.rs` | `toml_edit`, `cargo-platform`, `ra_ap_syntax`, `syn`, `proc-macro2`, `quote`, `prettyplease`, `serde_json`, `camino`, `ignore` | Ad-hoc toolchain discovery and coverage scripts (-650 LOC) | L1: FLS paragraph traceability models; L2: Dual toolchain parity validator; L3: DO-178C / DO-330 certification artifact generator |

---

## 3. Wave Execution & Feature Traceability Details

### Wave 1: Compiler Lint Governance, Baseline Purity & Lossless CST
- **BP-01 (EPIC-01: Compiler-Enforced Forbid Lint Gate & Zero Suppression AST)**:
  - `FEAT-01`: `CompilerLintEnforcementGate`
  - `FEAT-02`: `WorkspaceManifestForbidGate`
  - `FEAT-03`: `ClippyAttributeCstParser`
  - `FEAT-04`: `CrateRootAttributeSanitizer`
  - `FEAT-05`: `TestTargetLintEnforcer`
  - *Compliance Invariants*: Rejects inner/outer `allow` attributes with `E0453`. Submodules strictly `< 380` SLOC.
- **BP-02 (EPIC-02: Check 34 AST Purity, Fail-Closed I/O & Anti-Gaming Test Gate)**:
  - `FEAT-06`: `AntiGamingTestBodyAuditor`
  - `FEAT-07`: `Check34AstPurityAndAntiGamingGate`
  - `FEAT-08`: `FailClosedIoAstVerifier`
  - `FEAT-09`: `RangeSliceAstDetector`
  - `FEAT-10`: `SsotSymbolDeduplicationGate`
  - *Compliance Invariants*: Zero direct range slicing on collections. Fail-closed I/O error enforcement.
- **BP-08 (EPIC-08: Lossless Rust CST Codemod Engine & Unified rustc-Compatible JSON Diagnostics)**:
  - `FEAT-36`: `AirGappedCargoLockGovernor`
  - `FEAT-37`: `LosslessRustCstAnalyzer`
  - `FEAT-38`: `StructuredModularCodegenSynthesizer`
  - `FEAT-39`: `UnifiedRustcJsonDiagnosticEmitter`
  - `FEAT-40`: `ZeroTokenRustSpanCodemodEngine`
  - *Compliance Invariants*: Replaces ~19,700+ LOC of hand-rolled scanners. Single-line NDJSON format.

### Wave 2: Grammar Coverage, Parser Gate & Specification Forward Projection
- **BP-03 (EPIC-03: SysML v2 AST Bijective Roundtrip & Token Stream Parser Gate)**:
  - `FEAT-11`: `BijectiveSysmlSerializer`
  - `FEAT-12`: `CompilerRoundtripVerificationGate`
  - `FEAT-13`: `FailClosedRecursiveDescentParser`
  - `FEAT-14`: `NativeSigilAndLifetimeLexer`
  - `FEAT-15`: `SysmlBijectiveRoundtripCompilerGate`
  - *Compliance Invariants*: Full Clauses 7-12 coverage. Bidirectional AST serialization without loss.
- **BP-04 (EPIC-04: Deterministic Specification ICD & WBS Forward Compiler)**:
  - `FEAT-16`: `AgileSpecForwardProjector`
  - `FEAT-17`: `BidirectionalBacklogReconciler`
  - `FEAT-18`: `DeterministicSpecIcdWbsForwardCompiler`
  - `FEAT-19`: `LogicalIcdMatrixCompiler`
  - `FEAT-20`: `MilStd881eWbsExporter`
  - *Compliance Invariants*: Deterministic CommonMark AST parsing. Zero regex-based table manipulation.
- **BP-10 (EPIC-10: Incremental Lossless SysML v2 & Polyglot Compiler Engine)**:
  - `FEAT-46`: `FormattingPreservingSyncSplicer`
  - `FEAT-47`: `LogosRowanSysmlCstEngine`
  - `FEAT-48`: `SalsaIncrementalCompilerDatabase`
  - `FEAT-49`: `TypedPolyglotIngestionFrontend`
  - *Compliance Invariants*: Salsa incremental computation with early-cutoff. Ariadne error visualization.

### Wave 3: Safety Docstrings, Static Audit & Self-Hosting Codegen
- **BP-05 (EPIC-05: Unified AST Safety Docstring Synthesizer & Check 36 Verification)**:
  - `FEAT-21`: `Check36AstCompletenessVerifier`
  - `FEAT-22`: `ScopedSchemaContextResolver`
  - `FEAT-23`: `SevenSectionDocstringGenerator`
  - `FEAT-24`: `UnifiedRustDartCstParser`
  - `FEAT-25`: `UnifiedSafetyDocstringSynthesizer`
  - *Compliance Invariants*: Complete 7-section docstring on Tier-A boundaries. Compaction on Tier-B/C items.
- **BP-06 (EPIC-06: Deterministic Static 5-Pillar Code Auditor & Defect Dossier Synthesizer)**:
  - `FEAT-26`: `ClosedLoopDefectPublisherGate`
  - `FEAT-27`: `DeterministicDossierSynthesizer`
  - `FEAT-28`: `DeterministicStaticDefectAuditor`
  - `FEAT-29`: `FivePillarStaticAstAnalyzer`
  - `FEAT-30`: `MermaidUmlDiagramGenerator`
  - *Compliance Invariants*: 5-pillar AST audit. Validated 7-section defect dossiers.
- **BP-07 (EPIC-07: Self-Hosting 4-Level Modular Multi-File Code Generator)**:
  - `FEAT-31`: `FourLevelAstPartitionerAndHoister`
  - `FEAT-32`: `ModularFlutterMultiFileEmitter`
  - `FEAT-33`: `ModularRustMultiFileEmitter`
  - `FEAT-34`: `SelfHostingModularCodegenCompiler`
  - `FEAT-35`: `TopologicalLayerDagVerifier`
  - *Compliance Invariants*: Metacircular fixed-point identity `SHA-256(Stage 1) == SHA-256(Stage 2)`.

### Wave 4: Multi-Stage Formal Verification & Ferrocene Toolchain
- **BP-09 (EPIC-09: Multi-Stage Compiler & Formal Verification Gatekeeper)**:
  - `FEAT-41`: `FrbIcdBridgeAndMiriUbGate`
  - `FEAT-42`: `IsolatedDylintMirAllocationGate`
  - `FEAT-43`: `KaniCbmcProofSynthesizer`
  - `FEAT-44`: `MultiStageVerificationOrchestrator`
  - `FEAT-45`: `StableClippyTotalFunctionGate`
  - *Compliance Invariants*: 5-stage formal pipeline. Zero undetected undefined behavior.
- **BP-11 (EPIC-11: Ferrocene Qualified Toolchain FLS Traceability, Structural Coverage & 3-Ring Certification)**:
  - `FEAT-50`: `CertifiedCoreSubsetAndCoverageGate`
  - `FEAT-51`: `DualModeFerroceneToolchainGate`
  - `FEAT-52`: `FlsParagraphTraceabilitySynthesizer`
  - `FEAT-53`: `ThreeRingAssurancePartitionCompiler`
  - *Compliance Invariants*: Paired `criticalup.toml` and `rust-toolchain.toml`. FLS paragraph traceability.

---

## 4. Verification & Quality Gates

Each dispatch item execution must be validated against the three-tier baseline gates:
1. **Tier 1 (Specification & Governance Check)**:
   ```bash
   ./target/release/verify-baseline check-output ./.pipeline/upstream/docs/management/MASTER_ATOMIC_DISPATCH_QUEUE.md
   ```
2. **Tier 2 (Submodule Modularity & Net-Negative Ratchet)**:
   - Confirm all added submodules under `crates/<crate>/src/<module>/<submodule>.rs` remain strictly `< 380` executable SLOC.
   - Confirm net-negative deletion of legacy scanners in `crates/` monotonically advances toward zero.
3. **Tier 3 (Formatting & Path Invariants)**:
   - Zero Unicode em dashes (`\u2014`) in any emitted specification or source file.
   - Zero absolute or host-specific machine paths (workspace-relative `./...` paths only).

