//! Segment allocation matrix, port taxonomy, reusable recovery, JSON dictionary ingestion, and Section 4.5 supervisory authority handoff synthesis.

use serde_json::Value;

use crate::sanitize::sanitize_level_1b_operational_text;

use super::{SubsystemPart, SysMLParameterBindingEngine};

impl SysMLParameterBindingEngine {
    /// Populates reusable recovery containment and terminal facility parameters based on AST components.
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-DERIVE-LIFECYCLE-CONTRACT]
    pub(super) fn derive_reusable_recovery_bindings(&mut self) {
        let has_chute = self.has_ast_component("parachute")
            || self.has_ast_component("chute")
            || self
                .parameter_bindings
                .get("CONTAINMENT_MITIGATION_TYPE")
                .is_some_and(|s| s.contains("PARACHUTE"));
        let has_buoyant = self.has_ast_component("buoyancy")
            || self.has_ast_component("drop_weight")
            || self.has_ast_component("ballast");
        let has_brake = self.has_ast_component("brake");

        let (mit_type, dev_term, sys_desc, tier4_desc, seq_desc) = if has_chute {
            (
                "PARACHUTE_DESCENT",
                "parachute recovery system",
                "certified parachute recovery system",
                "autonomous parachute deployment and motor drive power isolation",
                "autonomous return-to-base or parachute recovery sequence",
            )
        } else if has_buoyant {
            (
                "BUOYANT_SURFACE_ASCENT",
                "positive buoyancy drop-weight",
                "galvanic drop-weight ballast release and passive buoyancy ascent system",
                "galvanic drop-weight release and passive buoyancy ascent",
                "ballast expulsion and passive buoyant surface recovery sequence",
            )
        } else if has_brake {
            (
                "MECHANICAL_BRAKE_DECELERATION",
                "emergency brake",
                "autonomous friction brake deceleration system",
                "friction caliper clamping and actuator power cutoff",
                "autonomous deceleration and safe parking sequence",
            )
        } else {
            (
                "FAILSAFE_CONTAINMENT",
                "failsafe containment",
                "autonomous failsafe containment system",
                "autonomous emergency containment actuation or actuator power cutoff",
                "autonomous return-to-base sequence",
            )
        };

        for (k, v) in [
            ("CONTAINMENT_MITIGATION_TYPE", mit_type),
            ("RECOVERY_DEVICE_TERM", dev_term),
            ("FAILSAFE_CONTAINMENT_SYSTEM", sys_desc),
            ("TIER4_CONTAINMENT_DESC", tier4_desc),
            ("LIFECYCLE_FAILSAFE_SEQUENCE", seq_desc),
            ("PRIMARY_TERMINAL_TARGET", "Primary Recovery Base"),
            ("SECONDARY_TERMINAL_TARGET", "Secondary Divert Site RS-DIVERT-ALPHA"),
            ("PRIMARY_RECOVERY_FACILITY", "Primary Recovery Base"),
            ("SECONDARY_RECOVERY_FACILITY", "Secondary Divert Base"),
            ("LIFECYCLE_TRANSIT_MODE", "Autonomous_RTB_Transit"),
            ("LIFECYCLE_END_STATE", "All assigned operational corridor waypoints fully traversed and verified; zero unauthorized state boundary excursions; zero unmitigated collision or interference hazards; all state conditions positively identified and verified ($C_{\\mathrm{condition}} \\ge C_{\\mathrm{threshold}}$); and successful recovery at the primary base or designated secondary divert recovery site with residual resources strictly satisfying $R_{\\mathrm{reserve}} \\ge \\text{Ratio}_{\\text{reserve\\_min}} \\cdot R_{\\mathrm{capacity}}$."),
            ("LIFECYCLE_BINGO_SAFETY_ACTION", "Continuously compute closed-loop dynamic resource state and execute autonomous return-to-base (RTB) or secondary divert routing upon reaching safety thresholds ($R(t) \\le R_{\\mathrm{threshold}}(t)$)."),
            ("LIFECYCLE_POST_OP_STATE", "Post-operation stationary rest at recovery site with actuator safe locking, secure memory state purge, and diagnostic log offloading"),
        ] {
            self.parameter_bindings
                .entry(k.to_string())
                .or_insert_with(|| v.to_string());
        }
    }

    /// Synthesizes `SEGMENT_ALLOCATION_MATRIX` and `PORT_TAXONOMY_SECTION` from resolved AST parts.
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-DERIVE-SUBSYSTEM-ARCHITECTURE]
    pub(super) fn derive_segment_allocation_and_ports(&mut self, parts_to_render: &[SubsystemPart]) {
        let mut seg_lines = vec![
            "| Subsystem Part | Operational Segment | SSOT Ground Truth Source & Citation | Primary Functional Role |".to_string(),
            "| :--- | :--- | :--- | :--- |".to_string(),
        ];
        for p in parts_to_render {
            let raw_p_name = &p.name;
            let clean_p = sanitize_level_1b_operational_text(raw_p_name);
            let clean_p = if clean_p.is_empty() {
                raw_p_name.clone()
            } else {
                clean_p
            };
            let part_tier = if !p.epistemic_tier.is_empty() {
                &p.epistemic_tier
            } else {
                "[TIER-1: OEM]"
            };
            let part_prov_str = if !p.source_file.is_empty() {
                if let Some(line) = p.source_line {
                    format!("[{}]({}#L{}) {}", clean_p, p.source_file, line, part_tier)
                } else {
                    format!("[{}]({}) {}", clean_p, p.source_file, part_tier)
                }
            } else {
                format!("[{clean_p}](schema/DEAP_MODEL.sysml) {part_tier}")
            };
            seg_lines.push(format!("| `{clean_p}` | Primary Operational Segment | {part_prov_str} | SysML v2: `part def {raw_p_name}` | Realizes core operational mission functions |"));
        }
        self.parameter_bindings.insert(
            "SEGMENT_ALLOCATION_MATRIX".to_string(),
            seg_lines.join("\n"),
        );

        let port_tax = "- **PORT-C2-INOUT (INOUT):** Bidirectional supervisory command and telemetry interface.\n- **PORT-NAV-IN (IN):** Navigation, positioning, and reference frame telemetry input interface.\n- **PORT-ACT-OUT (OUT):** Deterministic actuator demand vector and containment control interface.\n- **PORT-PWR-IN (IN):** Regulated primary/auxiliary power bus distribution interface.";
        self.parameter_bindings
            .insert("PORT_TAXONOMY_SECTION".to_string(), port_tax.to_string());
    }

    /// Builds Section 4.5.2 bounded timeout recovery and deterministic rejection protocol lines (`[REQ-ASSEMBLE-CONOPS]`).
    fn build_handoff_timeout_and_rejection_lines() -> Vec<String> {
        [
            "#### 4.5.2 Bounded Timeout Recovery & Rejection Protocol",
            "To guarantee deterministic execution and prevent command deadlocks during supervisory control delegation, the protocol incorporates bounded timeout recovery and explicit rejection safety criteria:",
            "",
            "$$",
            "\\begin{aligned}",
            "\\tau_{\\mathrm{handoff}} &\\le \\tau_{\\mathrm{timeout}} = 5.0 \\\\",
            "t_{\\mathrm{RTT}} &\\le \\tau_{\\text{RTT\\_max}} = 100.0 \\\\",
            "P_{\\mathrm{loss}} &\\le P_{\\text{loss\\_max}} = 1.0",
            "\\end{aligned}",
            "$$",
            "",
            "- Parameter Definitions & Engineering Units:",
            "- tau_handoff: Total elapsed duration of the supervisory authority delegation sequence (s).",
            "- tau_timeout: Maximum allowable timeout window before automatic abort (tau_timeout = 5.0 s).",
            "- t_RTT: Round-trip transport latency between supervisory consoles and SystemController (ms).",
            "- tau_RTT_max: Maximum permissible round-trip time for supervisory transfer (tau_RTT_max = 100.0 ms).",
            "- P_loss: Measured packet loss rate on candidate supervisory transport channel (%).",
            "- P_loss_max: Maximum permissible packet loss rate threshold (P_loss_max = 1.0%).",
            "",
            "##### Bounded Timeout Recovery",
            "The timeout recovery mechanism guarantees bounded execution time and fail-safe state preservation:",
            "1. **Watchdog Timer Activation:** `SystemController` initializes a dedicated hardware watchdog timer with deadline $\\tau_{\\mathrm{timeout}} = 5.0\\text{ s}$ upon receipt of the initial transfer request (Step 3).",
            "2. **Deterministic Abort:** If the full 9-step delegation sequence fails to complete within $\\tau_{\\mathrm{timeout}} = 5.0\\text{ s}$, `SystemController` unconditionally aborts the transaction and drops pending session tokens.",
            "3. **Safe State Retention:** Active supervisory authority remains locked at `PrimarySupervisor`. If `PrimarySupervisor` has entered an unresponsive state, `SystemController` transitions to autonomous contingency failsafe mode.",
            "4. **Session Termination & Alerting:** All ephemeral session tokens, temporary state buffers, and uncommitted transaction contexts associated with the failed transfer are safely purged from volatile memory, and timeout alerts are dispatched across supervisory interfaces.",
            "",
            "##### Deterministic Rejection Safety Criteria",
            "The system deterministically rejects supervisory authority delegation requests upon meeting any of the four safety criteria specified below:",
            "",
            "| Rejection Criterion ID | Rejection Trigger Condition | Detection & Verification Mechanism | Deterministic System Action |",
            "| :--- | :--- | :--- | :--- |",
            "| **REJ-01** | Authentication Token Invalidation | Digital signature verification failure, expired token validity timestamp, or revoked credentials | Immediate transaction reject; security audit event logged; authority retained at PrimarySupervisor |",
            "| **REJ-02** | Telemetry / State Desynchronization | State parameter buffer discrepancy Delta_s > Delta_s_threshold between PrimarySupervisor and CandidateSupervisor | Immediate transaction reject; state re-synchronization alert dispatched; authority retained at PrimarySupervisor |",
            "| **REJ-03** | Active Contingency / Emergency State | System actively executing emergency containment procedures (EMG-01 through EMG-07) or operating in degraded containment mode | Interlock inhibition of handoff; transfer rejected; autonomous containment state machine takes precedence |",
            "| **REJ-04** | Transport Channel Quality Degradation | Candidate control link exceeds round-trip latency bound (t_RTT > tau_RTT_max) or packet loss threshold (P_loss > P_loss_max) | Immediate transaction reject; link quality alert dispatched; authority retained at PrimarySupervisor |",
        ]
        .into_iter()
        .map(str::to_string)
        .collect()
    }

    /// Synthesizes Section 4.5 abstract supervisory authority delegation sequence and timeout recovery protocol.
    /// Inputs: `&mut self` | Outputs: `()` | Invariants: cites ISO/IEC 27001 and ISO/IEC/IEEE 15288:2023
    ///Errors: never fails | Complexity: O(1)
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-DERIVE-SUPERVISORY-AUTHORITY-HANDOFF] | SysML v2: ActionDef(SysMLParameterBindingEngine::_derive_supervisory_authority_handoff)
    pub fn _derive_supervisory_authority_handoff(&mut self) {
        let sys_id = self
            .parameter_bindings
            .get("SYSTEM_IDENTIFIER")
            .or(self.inferred_system_identifier.as_ref())
            .cloned()
            .unwrap_or_else(|| "AutonomousSystem".to_string());

        let target_display = if let Some(first_part) = self.ast_part_names.iter().next() {
            sanitize_level_1b_operational_text(first_part)
        } else {
            format!("{sys_id} Core Controller")
        };

        let mut lines = vec![
            "#### 4.5.1 Abstract Supervisory Authority Delegation Sequence Diagram".to_string(),
            "In accordance with ISO/IEC 27001 (§A.9.4.2 Secure Log-on Procedures), ISO/IEC/IEEE 15288:2023 (§6.4.4 Architecture Definition Process), and generic SysML interface flows, supervisory control authority transfer between active and candidate supervisory stations enforces mutual authentication, session state transfer, and bounded delegation confirmation:".to_string(),
            String::new(),
            "```mermaid".to_string(),
            "sequenceDiagram".to_string(),
            "    autonumber".to_string(),
            "    participant CandidateSupervisor as \"Candidate Supervisory Console (Standby)\"".to_string(),
            "    participant AuthorizationAuthority as \"Security & Authorization Authority\"".to_string(),
            format!("    participant SystemController as \"{target_display}\""),
            "    participant PrimarySupervisor as \"Primary Supervisory Console (Active)\"".to_string(),
            String::new(),
            "    CandidateSupervisor->>AuthorizationAuthority: 1. Request Delegation Token (SessionID, Nonce_C, RoleCredentials)".to_string(),
            "    AuthorizationAuthority-->>CandidateSupervisor: 2. Issue Signed Delegation Token (TokenID, RoleClaims, ValidWindow)".to_string(),
            "    CandidateSupervisor->>SystemController: 3. Transmit Supervisory Delegation Request (TokenID, RoleClaims, Nonce_C)".to_string(),
            "    SystemController->>AuthorizationAuthority: 4. Verify Delegation Token & Policy (TokenID, RoleClaims)".to_string(),
            "    AuthorizationAuthority-->>SystemController: 5. Token Verification Confirmation (Status_Valid, Signature_Auth)".to_string(),
            "    SystemController->>PrimarySupervisor: 6. Relinquish Active Supervisory Authority (TransferID, TargetRole)".to_string(),
            "    PrimarySupervisor-->>SystemController: 7. Acknowledge Relinquish & State Digest (StateDigest, SyncAck)".to_string(),
            "    SystemController->>CandidateSupervisor: 8. Grant Supervisory Authority (SessionID, StateInitVector)".to_string(),
            "    CandidateSupervisor-->>SystemController: 9. Confirm Active Supervisory State (TelemetrySyncAck, Nonce_C)".to_string(),
            "```".to_string(),
            String::new(),
            "The 9 discrete supervisory delegation protocol steps are defined as follows:".to_string(),
            "1. **Step 1 (Delegation Token Request):** `CandidateSupervisor` transmits a delegation request containing its unique station identity, session nonce (`Nonce_C`), and certified operator credentials to `AuthorizationAuthority`.".to_string(),
            "2. **Step 2 (Token Issuance):** `AuthorizationAuthority` validates access control privileges and issues a signed, time-stamped authorization token (`TokenID`, `RoleClaims`, `ValidWindow`) back to `CandidateSupervisor`.".to_string(),
            "3. **Step 3 (Delegation Request Transmission):** `CandidateSupervisor` transmits the signed authorization token and session challenge nonce (`Nonce_C`) to `SystemController`.".to_string(),
            "4. **Step 4 (Token Verification Request):** `SystemController` queries `AuthorizationAuthority` to verify the digital signature, token validity period, and operational authorization scope.".to_string(),
            "5. **Step 5 (Verification Confirmation):** `AuthorizationAuthority` confirms cryptographic validity and returns `Status_Valid` with verification signature `Signature_Auth` to `SystemController`.".to_string(),
            "6. **Step 6 (Relinquish Authority Command):** `SystemController` issues a formal command to `PrimarySupervisor` to relinquish active supervisory authority (`RelinquishAuthorityCommand(TransferID, TargetRole)`).".to_string(),
            "7. **Step 7 (Relinquish Acknowledgment):** `PrimarySupervisor` returns an acknowledgment containing the latest state telemetry digest (`StateDigest`) and transitions to monitor-only standby state.".to_string(),
            "8. **Step 8 (Supervisory Authority Grant):** `SystemController` transmits `GrantSupervisoryAuthority` with newly negotiated session parameters and state initialization vectors to `CandidateSupervisor`.".to_string(),
            "9. **Step 9 (Confirmation & Committal):** `CandidateSupervisor` transmits `ConfirmActiveSupervisoryState` with telemetry synchronization confirmation (`TelemetrySyncAck`), committing active supervisory authority.".to_string(),
            String::new(),
        ];
        lines.extend(Self::build_handoff_timeout_and_rejection_lines());

        self.parameter_bindings.insert(
            "SUPERVISORY_AUTHORITY_HANDOFF_SECTION".to_string(),
            lines.join("\n"),
        );
    }

    /// Binds a single JSON key-value pair into `parameter_bindings` and maps semantic aliases (`[REQ-ASSEMBLE-CONOPS]`).
    fn ingest_json_entry(&mut self, k: &str, v: &Value) {
        self.explicit_keys.insert(k.to_uppercase());
        match v {
            Value::String(s) => {
                self.parameter_bindings.insert(k.to_string(), s.clone());
                self.parameter_bindings.insert(k.to_uppercase(), s.clone());
                self._map_semantic_aliases(k, s);
            }
            Value::Number(n) => {
                let str_val = n.to_string();
                self.parameter_bindings.insert(k.to_string(), str_val.clone());
                self.parameter_bindings.insert(k.to_uppercase(), str_val.clone());
                self._map_semantic_aliases(k, &str_val);
            }
            Value::Bool(b) => {
                let str_val = b.to_string();
                self.parameter_bindings.insert(k.to_string(), str_val.clone());
                self.parameter_bindings.insert(k.to_uppercase(), str_val.clone());
                self._map_semantic_aliases(k, &str_val);
            }
            Value::Array(arr) => {
                let lower_k = k.to_lowercase();
                if [
                    "core_mission_capabilities",
                    "core_capabilities",
                    "capabilities",
                    "mission_capabilities",
                ]
                .contains(&lower_k.as_str())
                {
                    let mut lines = Vec::new();
                    for (idx, item) in arr.iter().enumerate() {
                        let item_str = match item {
                            Value::String(s) => s.trim().to_string(),
                            other => other.to_string(),
                        };
                        if item_str.starts_with(&format!("{}.", idx + 1)) || item_str.starts_with('-') {
                            lines.push(format!("  {item_str}"));
                        } else {
                            lines.push(format!("  {}. {item_str}", idx + 1));
                        }
                    }
                    let formatted = lines.join("\n");
                    self.parameter_bindings.insert(k.to_string(), formatted.clone());
                    self.parameter_bindings.insert(k.to_uppercase(), formatted.clone());
                    self.parameter_bindings.insert("CORE_MISSION_CAPABILITIES".to_string(), formatted);
                }
            }
            _ => {}
        }
    }

    /// Extracts package/system identifier and AST part names from `schema_nodes` (`[REQ-ASSEMBLE-CONOPS]`).
    fn ingest_schema_nodes(&mut self, nodes: &[Value]) {
        for node in nodes {
            if let Value::String(s) = node {
                if let Some((kind, name)) = s.split_once(':') {
                    if (kind == "package" || kind == "system")
                        && self.inferred_system_identifier.is_none()
                    {
                        self.inferred_system_identifier = Some(name.trim().to_string());
                    }
                    if kind == "part" {
                        self.ast_part_names.insert(name.trim().to_string());
                    }
                }
            }
        }
    }

    /// Recursively ingests nested parameter dictionaries and `schema_nodes` from a JSON `Value`.
    /// Inputs: `data: &Value` | Outputs: `()` | Invariants: maps semantic aliases for all scalar entries
    ///Errors: ignores non-object JSON values | Complexity: O(N) over JSON tree nodes
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-INGEST-DICTIONARY] | SysML v2: ActionDef(SysMLParameterBindingEngine::ingest_dictionary)
    pub fn ingest_dictionary(&mut self, data: &Value) {
        if let Value::Object(map) = data {
            let containers = [
                "parameters", "domain_parameters", "domain_params",
                "specs", "attributes", "metadata",
            ];
            for container_key in &containers {
                if let Some(sub) = map.get(*container_key) {
                    self.ingest_dictionary(sub);
                }
            }

            for name_key in &[
                "system_identifier", "system_name", "system",
                "name", "SYSTEM_IDENTIFIER", "MISSION_SYSTEM_NAME",
            ] {
                if let Some(Value::String(s)) = map.get(*name_key) {
                    let trimmed = s.trim();
                    if !trimmed.is_empty() {
                        self.inferred_system_identifier = Some(trimmed.to_string());
                    }
                }
            }

            for (k, v) in map {
                if !containers.contains(&k.as_str()) && k != "schema_nodes" {
                    self.ingest_json_entry(k, v);
                }
            }

            if let Some(Value::Array(nodes)) = map.get("schema_nodes") {
                self.ingest_schema_nodes(nodes);
            }
        }
    }
}
