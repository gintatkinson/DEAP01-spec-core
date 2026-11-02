# Failure Mode Registry (Step 1.5 Universal Failure Dimensions)

| Package | Component | 1. Interface Gamma (Omission) | 2. State Phi (Commission) | 3. Action Omega (Timing/Ordering) | 4. Resource Psi (Value/CST-Path Drift) | STPA UCA / SC | Official Crate Mitigation |
|---|---|---|---|---|---|---|---|
| BP_01 | CompilerLintEnforcementGate | Missing lint check | Invalid lint rules | Late lint execution | Wrong lint path | UCA-BP01-01 / SC-BP01-01 | cargo-dylint |
| BP_01 | WorkspaceManifestForbidGate | Omit manifest check | Extra manifest rule | Post-build check | Manifest drift | UCA-BP01-02 / SC-BP01-02 | toml_edit |
| BP_01 | ClippyAttributeCstParser | Miss clippy attr | Parse extra token | Parse out of order | CST path shift | UCA-BP01-03 / SC-BP01-03 | ra_ap_syntax |
| BP_01 | CrateRootAttributeSanitizer | Skip root check | Extra root attr | Late sanitization | Path drift | UCA-BP01-04 / SC-BP01-04 | syn |
| BP_01 | TestTargetLintEnforcer | Miss test target | Extra test rule | Late test check | Target path drift | UCA-BP01-05 / SC-BP01-05 | cargo_metadata |
| BP_02 | AntiGamingTestBodyAuditor | Omit body check | Extra audit rule | Late audit run | AST body drift | UCA-BP02-01 / SC-BP02-01 | syn |
| BP_02 | Check34AstPurityAndAntiGamingGate | Miss purity check | Extra check rule | Late check run | Purity path drift | UCA-BP02-02 / SC-BP02-02 | quote |
| BP_02 | FailClosedIoAstVerifier | Miss IO check | Extra IO rule | Late IO verify | IO path drift | UCA-BP02-03 / SC-BP02-03 | ra_ap_text_edit |
| BP_02 | RangeSliceAstDetector | Miss range check | Extra range rule | Late range scan | Slice path drift | UCA-BP02-04 / SC-BP02-04 | rowan |
| BP_02 | SsotSymbolDeduplicationGate | Miss symbol check | Extra dedupe rule | Late dedupe run | Symbol path drift | UCA-BP02-05 / SC-BP02-05 | serde_json |
| BP_03 | BijectiveSysmlSerializer | Omit serialization | Extra serialization | Late serialization | Schema drift | UCA-BP03-01 / SC-BP03-01 | prettyplease |
| BP_03 | CompilerRoundtripVerificationGate | Miss roundtrip check | Extra verification | Late verification | Roundtrip drift | UCA-BP03-02 / SC-BP03-02 | ra_ap_syntax |
| BP_03 | FailClosedRecursiveDescentParser | Miss parse branch | Extra parse branch | Late parse order | Parse tree drift | UCA-BP03-03 / SC-BP03-03 | logos |
| BP_03 | NativeSigilAndLifetimeLexer | Miss sigil check | Extra lex token | Late lexing order | Token path drift | UCA-BP03-04 / SC-BP03-04 | logos |
| BP_03 | SysmlBijectiveRoundtripCompilerGate | Miss compiler gate | Extra compiler rule | Late compile order | Compiler path drift | UCA-BP03-05 / SC-BP03-05 | syn |
| BP_04 | AgileSpecForwardProjector | Omit project step | Extra project rule | Late projection | Spec path drift | UCA-BP04-01 / SC-BP04-01 | pulldown-cmark |
| BP_04 | BidirectionalBacklogReconciler | Miss reconciliation | Extra reconcile rule | Late reconciliation | Backlog drift | UCA-BP04-02 / SC-BP04-02 | serde_json |
| BP_04 | DeterministicSpecIcdWbsForwardCompiler| Miss compiler step | Extra compile rule | Late compilation | Schema path drift | UCA-BP04-03 / SC-BP04-03 | toml_edit |
| BP_04 | LogicalIcdMatrixCompiler | Miss matrix check | Extra matrix rule | Late matrix build | Matrix path drift | UCA-BP04-04 / SC-BP04-04 | cargo_metadata |
| BP_04 | MilStd881eWbsExporter | Miss WBS export | Extra export rule | Late WBS export | WBS schema drift | UCA-BP04-05 / SC-BP04-05 | serde_json |
| BP_05 | Check36AstCompletenessVerifier | Fails to verify completeness | Verifies incorrect nodes | Late verification sequence | Value drift in AST | UCA-BP05-01 / SC-BP05-01 | syn |
| BP_05 | ScopedSchemaContextResolver | Fails to resolve context | Resolves wrong context | Out of order resolution | Context value drift | UCA-BP05-02 / SC-BP05-02 | salsa |
| BP_05 | SevenSectionDocstringGenerator | Fails to generate docstring | Generates invalid sections | Delayed docstring output | Docstring text drift | UCA-BP05-03 / SC-BP05-03 | pulldown-cmark |
| BP_05 | UnifiedRustDartCstParser | Fails to parse CST | Parses malformed tree | Incorrect parsing order | CST node drift | UCA-BP05-04 / SC-BP05-04 | ra_ap_syntax |
| BP_05 | UnifiedSafetyDocstringSynthesizer | Fails to synthesize safety | Synthesizes wrong safety | Late safety synthesis | Safety text drift | UCA-BP05-05 / SC-BP05-05 | quote |
| BP_06 | ClosedLoopDefectPublisherGate | Fails to publish defect | Publishes duplicate defect | Late defect publication | Defect ID drift | UCA-BP06-01 / SC-BP06-01 | serde_json |
| BP_06 | DeterministicDossierSynthesizer | Fails to synthesize dossier | Synthesizes wrong dossier | Delayed dossier output | Dossier data drift | UCA-BP06-02 / SC-BP06-02 | pulldown-cmark |
| BP_06 | DeterministicStaticDefectAuditor | Fails to audit defects | Audits false positives | Late defect audit | Audit metric drift | UCA-BP06-03 / SC-BP06-03 | cargo-dylint |
| BP_06 | FivePillarStaticAstAnalyzer | Fails to analyze AST | Analyzes incorrect AST | Late AST analysis | Pillar score drift | UCA-BP06-04 / SC-BP06-04 | syn |
| BP_06 | MermaidUmlDiagramGenerator | Fails to generate diagram | Generates broken diagram | Delayed diagram render | Diagram syntax drift | UCA-BP06-05 / SC-BP06-05 | serde_json |
| BP_07 | FourLevelAstPartitionerAndHoister | Fails to partition AST | Incorrect partitioning | Out of order partitioning | Level boundary drift | UCA-BP07-01 / SC-BP07-01 | rowan |
| BP_07 | ModularFlutterMultiFileEmitter | Fails to emit files | Emits corrupt files | Late file emission | File path drift | UCA-BP07-02 / SC-BP07-02 | prettyplease |
| BP_07 | ModularRustMultiFileEmitter | Fails to emit Rust files | Emits malformed Rust | Late Rust emission | Rust code drift | UCA-BP07-03 / SC-BP07-03 | quote |
| BP_07 | SelfHostingModularCodegenCompiler | Fails to compile codegen | Compiles invalid code | Late compilation step | Codegen output drift | UCA-BP07-04 / SC-BP07-04 | syn |
| BP_07 | TopologicalLayerDagVerifier | Fails to verify DAG | Verifies cyclic DAG | Out of order verification | Layer weight drift | UCA-BP07-05 / SC-BP07-05 | salsa |
| BP_08 | AirGappedCargoLockGovernor | Fails to govern lockfile | Modifies lockfile invalidly | Late lock check | Lockfile version drift | UCA-BP08-01 / SC-BP08-01 | cargo-lock |
| BP_08 | LosslessRustCstAnalyzer | Fails to analyze CST | Analyzes truncated CST | Delayed CST analysis | CST token drift | UCA-BP08-02 / SC-BP08-02 | ra_ap_syntax |
| BP_08 | StructuredModularCodegenSynthesizer | Fails to synthesize code | Synthesizes bad code | Late synthesis step | Code structure drift | UCA-BP08-03 / SC-BP08-03 | prettyplease |
| BP_08 | UnifiedRustcJsonDiagnosticEmitter | Fails to emit JSON | Emits malformed JSON | Late diagnostic emit | Diagnostic field drift | UCA-BP08-04 / SC-BP08-04 | serde_json |
| BP_08 | ZeroTokenRustSpanCodemodEngine | Fails to apply codemod | Applies incorrect codemod | Late codemod trigger | Span offset drift | UCA-BP08-05 / SC-BP08-05 | ra_ap_text_edit |
| BP_09 | FrbIcdBridgeAndMiriUbGate | Omit bridge check | Extra bridge rule | Late bridge run | Bridge path drift | UCA-BP09-01 / SC-BP09-01 | cargo miri |
| BP_09 | IsolatedDylintMirAllocationGate | Miss mir check | Extra mir rule | Late mir run | Mir path drift | UCA-BP09-02 / SC-BP09-02 | cargo-dylint |
| BP_09 | KaniCbmcProofSynthesizer | Omit proof synth | Extra proof rule | Late proof synth | Proof path drift | UCA-BP09-03 / SC-BP09-03 | cargo kani |
| BP_09 | MultiStageVerificationOrchestrator | Omit stage run | Extra verification rule | Late orchestration run | Stage path drift | UCA-BP09-04 / SC-BP09-04 | cargo_metadata |
| BP_09 | StableClippyTotalFunctionGate | Miss clippy gate | Extra function rule | Late function check | Function path drift | UCA-BP09-05 / SC-BP09-05 | cargo-dylint |
| BP_10 | FormattingPreservingSyncSplicer | Omit sync splicer | Extra sync rule | Late sync splice | Splicer path drift | UCA-BP10-01 / SC-BP10-01 | prettyplease |
| BP_10 | LogosRowanSysmlCstEngine | Miss cst engine | Extra CST rule | Late CST parse | CST path drift | UCA-BP10-02 / SC-BP10-02 | rowan |
| BP_10 | SalsaIncrementalCompilerDatabase | Omit db query | Extra db rule | Late db update | Database path drift | UCA-BP10-03 / SC-BP10-03 | salsa |
| BP_10 | TypedPolyglotIngestionFrontend | Omit frontend parse | Extra frontend rule | Late frontend run | Frontend path drift | UCA-BP10-04 / SC-BP10-04 | toml_edit |
| BP_11 | CertifiedCoreSubsetAndCoverageGate | Miss core subset | Extra coverage rule | Late coverage check | Coverage path drift | UCA-BP11-01 / SC-BP11-01 | criticalup |
| BP_11 | DualModeFerroceneToolchainGate | Omit toolchain check | Extra toolchain rule | Late toolchain switch | Toolchain path drift | UCA-BP11-02 / SC-BP11-02 | criticalup |
| BP_11 | FlsParagraphTraceabilitySynthesizer | Miss traceability check | Extra trace rule | Late trace synth | Trace path drift | UCA-BP11-03 / SC-BP11-03 | pulldown-cmark |
| BP_11 | ThreeRingAssurancePartitionCompiler | Omit partition check | Extra partition rule | Late partition compile | Partition path drift | UCA-BP11-04 / SC-BP11-04 | cargo_metadata |
