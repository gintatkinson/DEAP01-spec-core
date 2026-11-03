//! Domain ontology, operational mission intent, and lifecycle contract derivation.

use super::{LifecycleType, SysMLParameterBindingEngine};

impl SysMLParameterBindingEngine {
    /// Derives abstract MBSE operational safety interlock and positive condition verification tokens.
    /// Inputs: `&mut self` | Outputs: `()` | Invariants: zero ungrounded kinetic or weapon terminology
    ///Errors: never fails | Complexity: O(1)
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-DERIVE-DOMAIN-ONTOLOGY] | SysML v2: ActionDef(SysMLParameterBindingEngine::_derive_domain_ontology)
    pub fn _derive_domain_ontology(&mut self) {
        let dom = self.detected_domain.to_lowercase();
        self.is_non_aircraft =
            ["medical", "rail", "marine", "space", "industrial"].contains(&dom.as_str());
        self.is_civilian = true;

        self.parameter_bindings
            .entry("INTERLOCK_PREFIX".to_string())
            .or_insert_with(|| "INT".to_string());
        self.parameter_bindings
            .entry("INTERLOCK_SECTION_TITLE".to_string())
            .or_insert_with(|| "Safety Constraints & Subsystem Interlocks".to_string());
        self.parameter_bindings
            .entry("INTERLOCK_POLICY_NAME".to_string())
            .or_insert_with(|| "Operational Safety Interlocks".to_string());

        let pfx = self
            .parameter_bindings
            .get("INTERLOCK_PREFIX")
            .cloned()
            .unwrap_or_else(|| "INT".to_string());
        for idx in 1..=6 {
            self.parameter_bindings
                .entry(format!("INTERLOCK_0{idx}_TAG"))
                .or_insert_with(|| format!("{pfx}-0{idx}"));
        }
        self.parameter_bindings.insert(
            "TARGET_VERIFICATION_LABEL".to_string(),
            "Positive Condition Verification (PCV)".to_string(),
        );
        self.parameter_bindings
            .insert("TARGET_VERIFICATION_ACRONYM".to_string(), "PCV".to_string());
        self.parameter_bindings.insert(
            "TARGET_VERIFICATION_PHRASE".to_string(),
            "positive condition verification".to_string(),
        );
        self.parameter_bindings.insert(
            "HIGH_CONSEQUENCE_ACTION".to_string(),
            "high-consequence actuation".to_string(),
        );
        self.parameter_bindings.insert(
            "COLLATERAL_RISK_PHRASE".to_string(),
            "adjacent operational risk".to_string(),
        );
    }

    /// Derives operational purpose, primary mission statement, and core capabilities list.
    /// Inputs: `&mut self` | Outputs: `()` | Invariants: populates `CORE_MISSION_CAPABILITIES`
    ///Errors: never fails | Complexity: O(1)
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-DERIVE-OPERATIONAL-INTENT] | SysML v2: ActionDef(SysMLParameterBindingEngine::_derive_operational_intent)
    pub fn _derive_operational_intent(&mut self) {
        let sys_name = self
            .parameter_bindings
            .get("SYSTEM_IDENTIFIER")
            .or_else(|| self.parameter_bindings.get("SYSTEM_NAME"))
            .or(self.inferred_system_identifier.as_ref())
            .cloned()
            .unwrap_or_else(|| "Autonomous Cyber-Physical System".to_string());

        let dom = self.detected_domain.clone();

        self.parameter_bindings.entry("OPERATIONAL_PURPOSE".to_string()).or_insert_with(|| {
            format!("The primary operational purpose of {sys_name} is to execute deterministic autonomous operations in {dom} environments.")
        });
        self.parameter_bindings
            .entry("PRIMARY_OPERATIONAL_MISSION".to_string())
            .or_insert_with(|| {
                format!("The {sys_name} is engineered to execute high-assurance {dom} missions.")
            });

        self.parameter_bindings
            .entry("CORE_CAPABILITY_1".to_string())
            .or_insert_with(|| {
                format!("Autonomous trajectory tracking and corridor execution in {dom}.")
            });
        self.parameter_bindings
            .entry("CORE_CAPABILITY_2".to_string())
            .or_insert_with(|| {
                "Multi-modal sensor data fusion combining redundant state estimation sensors."
                    .to_string()
            });
        self.parameter_bindings.entry("CORE_CAPABILITY_3".to_string()).or_insert_with(|| {
            "Real-time high-throughput telemetry streaming and deterministic command processing.".to_string()
        });
        self.parameter_bindings
            .entry("CORE_CAPABILITY_4".to_string())
            .or_insert_with(|| {
                "Deterministic failsafe state machine ensuring autonomous containment.".to_string()
            });

        if !self
            .parameter_bindings
            .contains_key("CORE_MISSION_CAPABILITIES")
        {
            let c1 = self
                .parameter_bindings
                .get("CORE_CAPABILITY_1")
                .cloned()
                .unwrap_or_default();
            let c2 = self
                .parameter_bindings
                .get("CORE_CAPABILITY_2")
                .cloned()
                .unwrap_or_default();
            let c3 = self
                .parameter_bindings
                .get("CORE_CAPABILITY_3")
                .cloned()
                .unwrap_or_default();
            let c4 = self
                .parameter_bindings
                .get("CORE_CAPABILITY_4")
                .cloned()
                .unwrap_or_default();
            self.parameter_bindings.insert(
                "CORE_MISSION_CAPABILITIES".to_string(),
                format!("  1. {c1}\n  2. {c2}\n  3. {c3}\n  4. {c4}"),
            );
        }
    }

    /// Applies a slice of default `(key, value)` lifecycle bindings when not already bound (`[REQ-ASSEMBLE-CONOPS]`).
    fn apply_lifecycle_preset(&mut self, preset: &[(&str, &str)]) {
        for &(k, v) in preset {
            self.parameter_bindings
                .entry(k.to_string())
                .or_insert_with(|| v.to_string());
        }
    }

    /// Returns default lifecycle bindings for `ExpendableDisposal` and `ContinuousStationary` archetypes (`[REQ-ASSEMBLE-CONOPS]`).
    fn expendable_or_stationary_preset(selected: LifecycleType) -> &'static [(&'static str, &'static str)] {
        match selected {
            LifecycleType::ExpendableDisposal => &[
                ("PRIMARY_TERMINAL_TARGET", "Designated Safe Terminal Containment Zone"),
                ("SECONDARY_TERMINAL_TARGET", "Secondary Safe Containment Area RS-DIVERT-ALPHA"),
                ("PRIMARY_RECOVERY_FACILITY", "Designated Containment Area (Single-Use Disposal)"),
                ("SECONDARY_RECOVERY_FACILITY", "Secondary Containment Area (Single-Use Disposal)"),
                ("LIFECYCLE_TRANSIT_MODE", "Autonomous_Terminal_Containment_Transit"),
                ("LIFECYCLE_FAILSAFE_SEQUENCE", "autonomous terminal containment and safe de-energization sequence"),
                ("LIFECYCLE_END_STATE", "All assigned operational corridor waypoints fully traversed and verified; zero unauthorized state boundary excursions; zero unmitigated collision or interference hazards; and successful arrival in the designated terminal containment zone with complete power de-energization and safe disposal."),
                ("LIFECYCLE_BINGO_SAFETY_ACTION", "Continuously compute closed-loop dynamic resource state and execute safe terminal containment and power de-energization upon reaching safety thresholds ($R(t) \\le R_{\\mathrm{threshold}}(t)$)."),
                ("LIFECYCLE_POST_OP_STATE", "Terminal safe containment and de-energization with complete power bus isolation, memory state purge, and structural containment verification"),
                ("CONTAINMENT_MITIGATION_TYPE", "SAFE_TERMINAL_CONTAINMENT"),
                ("RECOVERY_DEVICE_TERM", "containment"),
                ("FAILSAFE_CONTAINMENT_SYSTEM", "autonomous containment watchdog and kinetic energy absorption"),
                ("TIER4_CONTAINMENT_DESC", "autonomous terminal containment and safe power isolation"),
            ],
            _ => &[
                ("PRIMARY_TERMINAL_TARGET", "Primary Sterile Field Docking Station"),
                ("SECONDARY_TERMINAL_TARGET", "Secondary Clinical Safe Staging Console"),
                ("PRIMARY_RECOVERY_FACILITY", "Clinical Operating Suite"),
                ("SECONDARY_RECOVERY_FACILITY", "Secondary Maintenance Staging Bay"),
                ("LIFECYCLE_TRANSIT_MODE", "Autonomous_Clinical_Safing"),
                ("LIFECYCLE_FAILSAFE_SEQUENCE", "electromechanical joint brake lock and sterile field safing sequence"),
                ("LIFECYCLE_END_STATE", "All assigned operational trajectory segments fully traversed and verified; zero unauthorized workspace boundary excursions; and successful safe parking at the primary sterile console with residual power reserves strictly satisfying $R_{\\mathrm{reserve}} \\ge \\text{Ratio}_{\\text{reserve\\_min}} \\cdot R_{\\mathrm{capacity}}$."),
                ("LIFECYCLE_BINGO_SAFETY_ACTION", "Continuously compute closed-loop dynamic resource state and execute automated electromechanical joint brake locking and sterile field preservation upon reaching safety thresholds ($R(t) \\le R_{\\mathrm{threshold}}(t)$)."),
                ("LIFECYCLE_POST_OP_STATE", "Post-operation stationary safe rest at sterile field console with engaged electromechanical joint brake locks, secure memory state purge, and diagnostic log offloading"),
                ("CONTAINMENT_MITIGATION_TYPE", "ELECTROMECHANICAL_BRAKE_LOCK"),
                ("RECOVERY_DEVICE_TERM", "failsafe joint brake"),
                ("FAILSAFE_CONTAINMENT_SYSTEM", "electromechanical joint brake lock and sterile field interlocks"),
                ("TIER4_CONTAINMENT_DESC", "instant electromechanical joint brake locking and actuator power bus disconnect"),
            ],
        }
    }

    /// Returns default lifecycle bindings for `TrackBoundGuided` and `PersistentOrbital` archetypes (`[REQ-ASSEMBLE-CONOPS]`).
    fn guided_or_orbital_preset(selected: LifecycleType) -> &'static [(&'static str, &'static str)] {
        match selected {
            LifecycleType::TrackBoundGuided => &[
                ("PRIMARY_TERMINAL_TARGET", "Primary Rail Maintenance Siding"),
                ("SECONDARY_TERMINAL_TARGET", "Secondary Controlled Deceleration Spur RS-DIVERT-ALPHA"),
                ("PRIMARY_RECOVERY_FACILITY", "Classification Yard Maintenance Depot"),
                ("SECONDARY_RECOVERY_FACILITY", "Secondary Service Siding"),
                ("LIFECYCLE_TRANSIT_MODE", "Autonomous_Track_Deceleration"),
                ("LIFECYCLE_FAILSAFE_SEQUENCE", "controlled track deceleration and siding brake sequence"),
                ("LIFECYCLE_END_STATE", "All assigned track blocks and route sections fully traversed and verified; zero unauthorized siding excursions; and successful controlled arrival at the primary maintenance siding or designated secondary spur with residual resources strictly satisfying $R_{\\mathrm{reserve}} \\ge \\text{Ratio}_{\\text{reserve\\_min}} \\cdot R_{\\mathrm{capacity}}$."),
                ("LIFECYCLE_BINGO_SAFETY_ACTION", "Continuously compute closed-loop dynamic resource state and execute controlled track deceleration or secondary maintenance siding divert upon reaching safety thresholds ($R(t) \\le R_{\\mathrm{threshold}}(t)$)."),
                ("LIFECYCLE_POST_OP_STATE", "Post-operation stationary rest at rail siding with mechanical parking brakes engaged, pneumatic systems vented, secure memory state purge, and diagnostic log offloading"),
                ("CONTAINMENT_MITIGATION_TYPE", "TRACK_SIDING_BRAKE"),
                ("RECOVERY_DEVICE_TERM", "pneumatic emergency brake"),
                ("FAILSAFE_CONTAINMENT_SYSTEM", "pneumatic emergency brake and track siding deceleration system"),
                ("TIER4_CONTAINMENT_DESC", "emergency brake pipe venting and mechanical friction caliper clamping"),
            ],
            _ => &[
                ("PRIMARY_TERMINAL_TARGET", "Designated De-Orbit Reentry Corridor / Graveyard Orbit"),
                ("SECONDARY_TERMINAL_TARGET", "Secondary Disposal Orbit Node RS-DIVERT-ALPHA"),
                ("PRIMARY_RECOVERY_FACILITY", "Designated Atmospheric Demise Footprint"),
                ("SECONDARY_RECOVERY_FACILITY", "Stable Graveyard Disposal Orbit"),
                ("LIFECYCLE_TRANSIT_MODE", "Autonomous_Disposal_Burn_Transit"),
                ("LIFECYCLE_FAILSAFE_SEQUENCE", "autonomous de-orbit disposal and passivation sequence"),
                ("LIFECYCLE_END_STATE", "All assigned orbital mission phases fully executed and verified; zero unauthorized constellation slot excursions; zero unmitigated orbital debris collision hazards; and successful de-orbit disposal or transfer to graveyard orbit with residual propellant satisfying disposal containment standards."),
                ("LIFECYCLE_BINGO_SAFETY_ACTION", "Continuously compute closed-loop dynamic resource state and execute autonomous de-orbit disposal burn or graveyard orbit maneuver upon reaching safety thresholds ($R(t) \\le R_{\\mathrm{threshold}}(t)$)."),
                ("LIFECYCLE_POST_OP_STATE", "Post-mission passivation and safe disposal state with reaction wheels desaturated, batteries passivated, secure memory state purge, and final ephemeris offload"),
                ("CONTAINMENT_MITIGATION_TYPE", "DEORBIT_DISPOSAL_BURN"),
                ("RECOVERY_DEVICE_TERM", "autonomous de-orbit propulsion"),
                ("FAILSAFE_CONTAINMENT_SYSTEM", "autonomous de-orbit propulsion and orbital passivation unit"),
                ("TIER4_CONTAINMENT_DESC", "autonomous retro-burn execution, reaction wheel desaturation, and battery passivation"),
            ],
        }
    }

    /// Derives lifecycle archetype, terminal containment targets, and post-operational state bindings.
    /// Inputs: `&mut self` | Outputs: `()` | Invariants: pure schema-driven containment derivation
    ///Errors: never fails | Complexity: O(P) over AST component names
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-DERIVE-LIFECYCLE-CONTRACT] | SysML v2: ActionDef(SysMLParameterBindingEngine::_derive_lifecycle_contract)
    pub fn _derive_lifecycle_contract(&mut self) {
        let dom = self.detected_domain.to_lowercase();
        let is_expendable = self
            .parameter_bindings
            .get("IS_EXPENDABLE")
            .is_some_and(|s| s == "true" || s == "1")
            || ["disposable", "single_use", "expendable"].contains(&dom.as_str())
            || self.has_ast_component("expendable")
            || self.has_ast_component("disposable");

        let selected_type = if let Some(lt) = self.parameter_bindings.get("LIFECYCLE_TYPE") {
            match lt.as_str() {
                "EXPENDABLE_DISPOSAL" => LifecycleType::ExpendableDisposal,
                "CONTINUOUS_STATIONARY" => LifecycleType::ContinuousStationary,
                "TRACK_BOUND_GUIDED" => LifecycleType::TrackBoundGuided,
                "PERSISTENT_ORBITAL" => LifecycleType::PersistentOrbital,
                _ => LifecycleType::ReusableRecovery,
            }
        } else if is_expendable {
            LifecycleType::ExpendableDisposal
        } else if dom == "medical" || self.has_ast_component("joint_brake") {
            LifecycleType::ContinuousStationary
        } else if dom == "rail" || self.has_ast_component("rail") {
            LifecycleType::TrackBoundGuided
        } else if dom == "space" || self.has_ast_component("orbit") {
            LifecycleType::PersistentOrbital
        } else {
            LifecycleType::ReusableRecovery
        };

        self.parameter_bindings.insert(
            "LIFECYCLE_TYPE".to_string(),
            selected_type.as_str().to_string(),
        );

        match selected_type {
            LifecycleType::ExpendableDisposal | LifecycleType::ContinuousStationary => {
                self.apply_lifecycle_preset(Self::expendable_or_stationary_preset(selected_type));
            }
            LifecycleType::TrackBoundGuided | LifecycleType::PersistentOrbital => {
                self.apply_lifecycle_preset(Self::guided_or_orbital_preset(selected_type));
            }
            LifecycleType::ReusableRecovery => {
                self.derive_reusable_recovery_bindings();
            }
        }
    }
}
