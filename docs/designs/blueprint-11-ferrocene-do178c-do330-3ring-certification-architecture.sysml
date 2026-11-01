doc /* Translated from Level 0 OEM Markdown specification */
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
    doc /* verifyToolchainAndStageParity(workspacePath) */
    action def verifyToolchainAndStageParity;
    doc /* synthesizeThreeRingWorkspace(sysmlPackageId) */
    action def synthesizeThreeRingWorkspace;
    doc /* attachAndAuditFlsTraceability(ringWorkspacePlan) */
    action def attachAndAuditFlsTraceability;
    doc /* auditCoreSubsetAndMcdcCoverage(flsTraceBundle) */
    action def auditCoreSubsetAndMcdcCoverage;
    doc /* verifyRingBoundariesAndMonitor(workspacePath) */
    action def verifyRingBoundariesAndMonitor;
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
    doc /* UUIDv5:  | Complexity: Class P | Standard: IEEE 29148-2018 / RFC 2119 | Diagnostics: E0100 */
    requirement def REQ_0611_BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C {
        id = "REQ-0611";
        text = "FerroceneThreeRingCertificationArchitecture (**REQ-0611** / BP-11 / DEAP-BP-DET-11, UUIDv5 6ba7b810-9dad-11d1-80b4-00c04fd430cb, Class P linear AST, ELF symbol-table, and LLVM coverage-profile verification complexity, governed by RTCA DO-178C DAL-A through DAL-E, RTCA DO-330 TQL-1 and TQL-5, RTCA";
        attribute uuidv5 : String = "";
        attribute complexity_class : String = "Class P";
        attribute governing_standard : String = "IEEE 29148-2018 / RFC 2119";
        attribute diagnostic_codes : String = "E0100";
        attribute ac_01_dual_mode_criticalup_toml_and_rust_toolchain_toml_parity_and_do_330_stage_2_fixed_point_qualification : String = "- **Given**: A workspace containing paired criticalup.toml (ferrocene-stable pinned to an exact qualified release and target triple set) and rust-toolchain.toml (pinned to the matching upstream rustc semantic version and target set), alongside a negative fixture where rust-toolchain.toml drifts by one minor version or omits a qualified target triple. - **When**: DualModeFerroceneToolchainGate executes verify_dual_toolchain_parity and evaluates Stage-1 versus Stage-2 self-hosted modular codegen SHA-256 digests (E0705) under --message-format=json. - **Then**: The version-matched workspace.";
        attribute ac_02_end_to_end_fls_normative_paragraph_traceability_emission_and_fail_closed_orphan_detection : String = "- **Given**: A SysML v2 package declaring RequirementDef (REQ-*), STPA safety constraint (SC-*), Ring-1 PartDef / StateDef / ConstraintDef constructs, and Ring-2 FFI PortDef contracts compiled through FlsParagraphTraceabilitySynthesizer, plus a negative fixture containing a Ring-1 function missing its [FLS: fls_*] docstring tag and a Ring-2 unsafe block citing an unknown paragraph fls_nonexistent999. - **When**: crates/compile-sysml/src/modular_codegen/ emits the annotated Rust units and crates/verify-baseline audits the 5-link FLS traceability chain against the pinned FLS paragraph catalog.";
        attribute ac_03_certified_core_library_subset_symbol_report_gate_rejecting_std_alloc_and_dynamic_formatting_symbols : String = "- **Given**: Two Ring-1 #![no_std] safety_kernel symbol-report fixtures: Fixture A linking exclusively certified core::option, core::result, core::num, and fixed-size array operations, and Fixture B inadvertently linking __rust_alloc, alloc::vec::Vec, or std::io::stdio::_print. - **When**: CertifiedCoreSubsetAndCoverageGate audits both symbol inventories against the certified core allowlist prefix catalog. - **Then**: Fixture A passes with uncertified_core_symbol_count == 0 (**C-CERT-04**), whereas Fixture B fails closed immediately with DEAP-E1103 enumerating every prohibited std, alloc, or.";
        attribute ac_04_zero_overcount_structural_and_mc_dc_coverage_verification_mapped_to_sysml_verification_cases : String = "- **Given**: A DAL-A Ring-1 SafetyKernel module exercised by blanket / llvm-profdata MC/DC condition-vector profiles linked to SysML TestCaseDef declarations (TEST-01 -> REQ-0611), alongside three negative fixtures: (1) a boolean decision a && (b || c) missing independent flip coverage for condition c (99% MC/DC), (2) an overcounted macro-duplicated source span, and (3) a coverage test lacking a SysML TEST-* -> REQ-* verified_by binding. - **When**: CertifiedCoreSubsetAndCoverageGate evaluates DO-178C Table A-7 coverage thresholds for DalLevel::DalA. - **Then**: The fully covered,.";
        attribute ac_05_schema_driven_3_ring_workspace_partitioning_and_ring_2_miri_guarded_ffi_precondition_enforcement : String = "- **Given**: A SysML v2 system model annotated with AssuranceLevel and @Layer stereotypes (Domain, Application, Presentation) and PortDef [min..max] ranges, plus two negative probes: Probe A introducing an illegal safety_kernel -> ffi_guard_bridge or safety_kernel -> app_flutter dependency edge, and Probe B injecting an out-of-range or NaN OperatorConsole payload across the Ring-2 FFI boundary. - **When**: ThreeRingAssurancePartitionCompiler synthesizes the 3-ring workspace (safety_kernel/, ffi_guard_bridge/, app_flutter/) and executes the Ring-2 boundary verification suite. - **Then**: The.";
        attribute ac_06_ring_1_challenge_response_command_interlock_and_telemetryframe_digest_readback_safetymonitor_against_ring_3_faults : String = "- **Given**: An active Ring-1 SafetyMonitor issuing sequenced TelemetryFrame envelopes (seq_t, FIPS 180-4 SHA-256 digest h_t) and two-step challenge nonces (eta_t) through Ring 2 to the untrusted Ring-3 Flutter OperatorConsole, tested against three Ring-3 fault injections: (1) corrupted display frame-digest readback (hat_h_t != h_t), (2) stale frame sequence counter (hat_seq_t < seq_t), and (3) spurious single-step critical ActuatorCommand lacking matching challenge nonce eta_t. - **When**: verify_three_ring_edge_and_monitor_readback evaluates the inbound confirmation and readback envelope.";
        attribute ac_07_twelve_gate_blueprint_verification_and_upstream_baseline_cleanliness_conformance : String = "- **Given**: docs/designs/blueprint-11-ferrocene-do178c-do330-3ring-certification-architecture.md persisted in the upstream DEAP01-spec-core repository. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-11-ferrocene-do178c-do330-3ring-certification-architecture.md and ./target/release/verify-baseline . --no-domain are executed. - **Then**: All 12 mechanical blueprint verification gates and all upstream workspace baseline gates (Check 10 through Check 41) exit with code 0, atomically persisting.";
        doc /* \begin{aligned} v(M_{\text{crit}}) = v(M_{\text{rust}}) &\land T_{\text{cert}} \subseteq T(M_{\text{crit}}) \cap T(M_{\text{rust}}) \\ \text{SHA-256}(\mathcal{A}_{\text{Stage 1}}^{\text{Ferrocene}}) = \text{SHA-256}(\mathcal{A}_{\text{Stage 2}}^{\text{Ferrocene}}) &\land \text{Codebase}(G_{\text{TQL1}}) \cap \text{RuleEngine}(V_{\text{TQL5}}) = \emptyset \end{aligned} */
        constraint def Invariant_REQ_0611_SC_01;
        doc /* \begin{aligned} \forall u \in \mathcal{I}_{R_1 \cup R_2}, \quad \emptyset \neq \pi_{\text{FLS}}(u) \subseteq \mathcal{P}_{\text{FLS}} &\implies N_{\text{unmapped}}(\mathcal{I}_{R_1 \cup R_2}) = 0 \\ \forall u \in \mathcal{I}_{R_1}, \quad \exists (r, s, p, k, c) \in \mathcal{T}_5 &: \text{item}(r, s, p, k, c) = u \end{aligned} */
        constraint def Invariant_REQ_0611_SC_02;
        doc /* \begin{aligned} \mathcal{S}_{\text{ELF}}(R_1) \subseteq \mathcal{S}_{\text{core}}^{\text{cert}} &\iff \lvert \mathcal{S}_{\text{ELF}}(R_1) \setminus \mathcal{S}_{\text{core}}^{\text{cert}} \rvert = 0 \\ \text{DAL}(R_1) = \text{DAL-A} \implies C_{\text{stmt}}(R_1) = 100\% \land C_{\text{dec}}(R_1) = 100\% &\land \forall d \in \mathcal{D}(R_1), \forall i \in \{1, \dots, m\}, \exists (v^+, v^-) \in \mathcal{V}_{\text{TEST}}(d) : \\ (v_i^+ \neq v_i^-) \land (\forall j \neq i, \; v_j^+ = v_j^-) &\land (d(v^+) \neq d(v^-)) \end{aligned} */
        constraint def Invariant_REQ_0611_SC_03;
        doc /* \begin{aligned} \Gamma_{R_2}(x, \sigma) = 1 &\iff \text{finite}(x) \land x \in [x_{\min}, x_{\max}] \land \text{Precond}_{\text{FSM}}(\sigma, x) \\ \text{Commit}_{R_1}(u_{\text{crit}}, \hat{\eta}_t, \hat{h}_t, \widehat{\text{seq}}_t) = 1 &\iff \Gamma_{R_2}(u_{\text{crit}}, \sigma_t) = 1 \land (\hat{\eta}_t = \eta_t) \land (\hat{h}_t = h_t) \land (\widehat{\text{seq}}_t = \text{seq}_t) \end{aligned} */
        constraint def Invariant_REQ_0611_SC_04;
        require Invariant_REQ_0611_SC_01;
        require Invariant_REQ_0611_SC_02;
        require Invariant_REQ_0611_SC_03;
        require Invariant_REQ_0611_SC_04;
        derived from REQ_0607;
        derived from REQ_0609;
        derived from REQ_0610;
        verify by AC_01_Dual_Mode_criticalup_toml_and_rust_toolchain_toml_Parity_and_DO_330_Stage_2_Fixed_Point_Qualification;
        verify by AC_02_End_to_End_FLS_Normative_Paragraph_Traceability_Emission_and_Fail_Closed_Orphan_Detection;
        verify by AC_03_Certified_core_Library_Subset_Symbol_Report_Gate_Rejecting_std_alloc_and_Dynamic_Formatting_Symbols;
        verify by AC_04_Zero_Overcount_Structural_and_MC_DC_Coverage_Verification_Mapped_to_SysML_Verification_Cases;
        verify by AC_05_Schema_Driven_3_Ring_Workspace_Partitioning_and_Ring_2_Miri_Guarded_FFI_Precondition_Enforcement;
        verify by AC_06_Ring_1_Challenge_Response_Command_Interlock_and_TelemetryFrame_Digest_Readback_SafetyMonitor_Against_Ring_3_Faults;
        verify by AC_07_Twelve_Gate_Blueprint_Verification_and_Upstream_Baseline_Cleanliness_Conformance;
        satisfy by CertifiedCoreSubsetAndCoverageGate;
    }
    doc /* Extracted from Mermaid stateDiagram specification. */
    state def BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture_LifecycleState {
        state Idle;
        state VerifyingDualToolchainParity;
        state RejectedFailClosed;
        state SynthesizingThreeRingPartitions;
        state EmbeddingFlsTraceability;
        state AuditingCoreSubsetAndMcdc;
        state VerifyingRing1SafetyMonitor;
        state CertifiedPackageReady;
        transition Initial_to_Idle first Initial then Idle;
        transition Idle_to_VerifyingDualToolchainParity_on_loadToolchainManifests first Idle accept loadToolchainManifests then VerifyingDualToolchainParity;
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
    doc /* - **Given**: A workspace containing paired criticalup.toml (ferrocene-stable pinned to an exact qualified release and target triple set) and rust-toolchain.toml (pinned to the matching upstream rustc semantic version and target set), alongside a negative fixture where rust-toolchain.toml drifts by one minor version or omits a qualified target triple. - **When**: DualModeFerroceneToolchainGate executes verify_dual_toolchain_parity and evaluates Stage-1 versus Stage-2 self-hosted modular codegen SHA-256 digests (E0705) under --message-format=json. - **Then**: The version-matched workspace. */
    use case def UC_AC_01_Dual_Mode_criticalup_toml_and_rust_toolchain_toml_Parity_and_DO_330_Stage_2_Fixed_Point_Qualification {
        subject CertifiedCoreSubsetAndCoverageGate;
        objective "Dual Mode criticalup toml and rust toolchain toml Parity and DO 330 Stage 2 Fixed Point Qualification";
    }
    doc /* - **Given**: A SysML v2 package declaring RequirementDef (REQ-*), STPA safety constraint (SC-*), Ring-1 PartDef / StateDef / ConstraintDef constructs, and Ring-2 FFI PortDef contracts compiled through FlsParagraphTraceabilitySynthesizer, plus a negative fixture containing a Ring-1 function missing its [FLS: fls_*] docstring tag and a Ring-2 unsafe block citing an unknown paragraph fls_nonexistent999. - **When**: crates/compile-sysml/src/modular_codegen/ emits the annotated Rust units and crates/verify-baseline audits the 5-link FLS traceability chain against the pinned FLS paragraph catalog. */
    use case def UC_AC_02_End_to_End_FLS_Normative_Paragraph_Traceability_Emission_and_Fail_Closed_Orphan_Detection {
        subject CertifiedCoreSubsetAndCoverageGate;
        objective "End to End FLS Normative Paragraph Traceability Emission and Fail Closed Orphan Detection";
    }
    doc /* - **Given**: Two Ring-1 #![no_std] safety_kernel symbol-report fixtures: Fixture A linking exclusively certified core::option, core::result, core::num, and fixed-size array operations, and Fixture B inadvertently linking __rust_alloc, alloc::vec::Vec, or std::io::stdio::_print. - **When**: CertifiedCoreSubsetAndCoverageGate audits both symbol inventories against the certified core allowlist prefix catalog. - **Then**: Fixture A passes with uncertified_core_symbol_count == 0 (**C-CERT-04**), whereas Fixture B fails closed immediately with DEAP-E1103 enumerating every prohibited std, alloc, or. */
    use case def UC_AC_03_Certified_core_Library_Subset_Symbol_Report_Gate_Rejecting_std_alloc_and_Dynamic_Formatting_Symbols {
        subject CertifiedCoreSubsetAndCoverageGate;
        objective "Certified core Library Subset Symbol Report Gate Rejecting std alloc and Dynamic Formatting Symbols";
    }
    doc /* - **Given**: A DAL-A Ring-1 SafetyKernel module exercised by blanket / llvm-profdata MC/DC condition-vector profiles linked to SysML TestCaseDef declarations (TEST-01 -> REQ-0611), alongside three negative fixtures: (1) a boolean decision a && (b || c) missing independent flip coverage for condition c (99% MC/DC), (2) an overcounted macro-duplicated source span, and (3) a coverage test lacking a SysML TEST-* -> REQ-* verified_by binding. - **When**: CertifiedCoreSubsetAndCoverageGate evaluates DO-178C Table A-7 coverage thresholds for DalLevel::DalA. - **Then**: The fully covered,. */
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
    doc /* - **Given**: docs/designs/blueprint-11-ferrocene-do178c-do330-3ring-certification-architecture.md persisted in the upstream DEAP01-spec-core repository. - **When**: ./target/release/verify-baseline check-blueprint docs/designs/blueprint-11-ferrocene-do178c-do330-3ring-certification-architecture.md and ./target/release/verify-baseline . --no-domain are executed. - **Then**: All 12 mechanical blueprint verification gates and all upstream workspace baseline gates (Check 10 through Check 41) exit with code 0, atomically persisting. */
    use case def UC_AC_07_Twelve_Gate_Blueprint_Verification_and_Upstream_Baseline_Cleanliness_Conformance {
        subject CertifiedCoreSubsetAndCoverageGate;
        objective "Twelve Gate Blueprint Verification and Upstream Baseline Cleanliness Conformance";
    }
    doc /* Extracted from Mermaid sequenceDiagram specification. */
    interaction def BP_11_Ferrocene_Qualified_Toolchain_FLS_Traceability_Structural_Coverage_and_3_Ring_Rust_Flutter_DO_178C_DO_330_Certification_Architecture_SequenceInteraction {
        lifeline DualModeFerroceneToolchainGate : DualModeFerroceneToolchainGate;
        lifeline ThreeRingAssurancePartitionCompiler : ThreeRingAssurancePartitionCompiler;
        lifeline FlsParagraphTraceabilitySynthesizer : FlsParagraphTraceabilitySynthesizer;
        lifeline CertifiedCoreSubsetAndCoverageGate : CertifiedCoreSubsetAndCoverageGate;
        message verifyToolchainAndStageParity from DualModeFerroceneToolchainGate to DualModeFerroceneToolchainGate;
        message synthesizeThreeRingWorkspace from DualModeFerroceneToolchainGate to ThreeRingAssurancePartitionCompiler;
        message reply_ThreeRingWorkspacePlan from ThreeRingAssurancePartitionCompiler to DualModeFerroceneToolchainGate;
        message attachAndAuditFlsTraceability from ThreeRingAssurancePartitionCompiler to FlsParagraphTraceabilitySynthesizer;
        message reply_FlsTaggedTraceBundle from FlsParagraphTraceabilitySynthesizer to ThreeRingAssurancePartitionCompiler;
        message auditCoreSubsetAndMcdcCoverage from FlsParagraphTraceabilitySynthesizer to CertifiedCoreSubsetAndCoverageGate;
        message reply_Do178cTableA7Receipt from CertifiedCoreSubsetAndCoverageGate to FlsParagraphTraceabilitySynthesizer;
        message verifyRingBoundariesAndMonitor from DualModeFerroceneToolchainGate to ThreeRingAssurancePartitionCompiler;
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