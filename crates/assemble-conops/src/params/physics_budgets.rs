//! Mass budgets, quadratic kinetic physics, energy reserves, and regulatory standards derivation.

use super::SysMLParameterBindingEngine;

impl SysMLParameterBindingEngine {
    /// Derives subsystem mass partition budgets from total system mass and mass fraction percentages.
    /// Inputs: `&mut self` | Outputs: `()` | Invariants: partition sum conserves `TOTAL_MTOW_KG`
    ///Errors: uses default fractions if parsing fails | Complexity: O(1)
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-DERIVE-MASS-BUDGETS] | SysML v2: ActionDef(SysMLParameterBindingEngine::_derive_mass_budgets)
    pub fn _derive_mass_budgets(&mut self) {
        let mtow = self._get_mtow_value();
        if !self.explicit_keys.contains("TOTAL_MTOW_KG")
            && !self.parameter_bindings.contains_key("TOTAL_MTOW_KG")
        {
            self.parameter_bindings
                .insert("TOTAL_MTOW_KG".to_string(), format!("{mtow:.1}"));
        }

        let has_fractions = (self.parameter_bindings.contains_key("MASS_FRACTION_AIRFRAME_PCT")
            || self.parameter_bindings.contains_key("MASS_FRACTION_STRUCTURE_PCT"))
            && ["MASS_FRACTION_AVIONICS_PCT", "MASS_FRACTION_PROPULSION_PCT", "MASS_FRACTION_ENERGY_PCT", "MASS_FRACTION_PAYLOAD_PCT"]
                .iter()
                .all(|k| self.parameter_bindings.contains_key(*k));

        if has_fractions {
            let pct = |key: &str, fallback: f64| -> f64 {
                self.parameter_bindings
                    .get(key)
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(fallback)
                    / 100.0
            };
            let airframe_pct = self
                .parameter_bindings
                .get("MASS_FRACTION_STRUCTURE_PCT")
                .or_else(|| self.parameter_bindings.get("MASS_FRACTION_AIRFRAME_PCT"))
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(30.0)
                / 100.0;
            let airframe = (airframe_pct * mtow * 100.0).round() / 100.0;
            let avionics = (pct("MASS_FRACTION_AVIONICS_PCT", 15.0) * mtow * 100.0).round() / 100.0;
            let propulsion = (pct("MASS_FRACTION_PROPULSION_PCT", 25.0) * mtow * 100.0).round() / 100.0;
            let energy = (pct("MASS_FRACTION_ENERGY_PCT", 20.0) * mtow * 100.0).round() / 100.0;
            let payload = (pct("MASS_FRACTION_PAYLOAD_PCT", 7.0) * mtow * 100.0).round() / 100.0;
            let containment =
                ((mtow - (airframe + avionics + propulsion + energy + payload)) * 100.0).round()
                    / 100.0;

            for (k, val) in [
                ("MASS_BUDGET_AIRFRAME_KG", airframe),
                ("MASS_BUDGET_STRUCTURE_KG", airframe),
                ("MASS_BUDGET_AVIONICS_KG", avionics),
                ("MASS_BUDGET_PROPULSION_KG", propulsion),
                ("MASS_BUDGET_ENERGY_KG", energy),
                ("MASS_BUDGET_PAYLOAD_KG", payload),
                ("MASS_BUDGET_CONTAINMENT_KG", containment),
            ] {
                self.parameter_bindings
                    .entry(k.to_string())
                    .or_insert_with(|| val.to_string());
            }
        }
    }

    /// Computes mitigated residual velocity (`V_RESIDUAL_MPS`) from parachute drag or lifecycle archetype (`[REQ-ASSEMBLE-CONOPS]`).
    fn compute_residual_velocity(&self, m: f64, v_max: f64) -> f64 {
        if let Some(vr) = self
            .parameter_bindings
            .get("V_RESIDUAL_MPS")
            .or_else(|| self.parameter_bindings.get("V_RESIDUAL"))
            .and_then(|s| s.parse::<f64>().ok())
        {
            return vr;
        }
        let has_parachute = self.has_ast_component("parachute")
            || self.has_ast_component("chute")
            || self.parameter_bindings.contains_key("S_CANOPY")
            || self.parameter_bindings.contains_key("PARACHUTE_DRAG_COEFFICIENT");
        if has_parachute {
            let rho = self
                .parameter_bindings
                .get("AIR_DENSITY_KGM3")
                .or_else(|| self.parameter_bindings.get("FLUID_DENSITY_KGM3"))
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(1.225);
            let s_canopy = self
                .parameter_bindings
                .get("S_CANOPY")
                .or_else(|| self.parameter_bindings.get("S_MIT"))
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(84.0);
            let cd_canopy = self
                .parameter_bindings
                .get("PARACHUTE_DRAG_COEFFICIENT")
                .or_else(|| self.parameter_bindings.get("C_D_MIT"))
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(1.75);
            let denom = rho * s_canopy * cd_canopy;
            if denom > 0.0 && m > 0.0 {
                ((2.0 * m * 9.80665 / denom).sqrt() * 100.0).round() / 100.0
            } else {
                1.5
            }
        } else if self
            .parameter_bindings
            .get("LIFECYCLE_TYPE")
            .is_some_and(|s| s == "EXPENDABLE_DISPOSAL")
            || self
                .parameter_bindings
                .get("IS_EXPENDABLE")
                .is_some_and(|s| s == "true")
        {
            v_max
        } else if self.detected_domain == "medical" {
            0.0
        } else {
            1.5
        }
    }

    /// Derives unmitigated and mitigated kinetic energy and terminal velocity bounds from mass and velocity parameters.
    /// Inputs: `&mut self` | Outputs: `()` | Invariants: $E_k = \frac{1}{2} m v^2$
    ///Errors: falls back to safe positive constants on missing parameters | Complexity: O(1)
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-DERIVE-QUADRATIC-PHYSICS] | SysML v2: ActionDef(SysMLParameterBindingEngine::_derive_quadratic_physics)
    pub fn _derive_quadratic_physics(&mut self) {
        let m = self._get_mtow_value();
        let v_max = self
            .parameter_bindings
            .get("V_MAX_MPS")
            .or_else(|| self.parameter_bindings.get("V_MAX_NOMINAL_MPS"))
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(25.0);

        let ek_unmit = (0.5 * m * v_max.powi(2) * 10.0).round() / 10.0;
        let v_residual = self.compute_residual_velocity(m, v_max);
        let ek_mit = (0.5 * m * v_residual.powi(2) * 10.0).round() / 10.0;

        for (k, val) in [
            ("V_BOUNDARY_MAX_MPS", format!("{v_max:.1}")),
            ("E_BOUNDARY_IMPACT_JOULES", format!("{ek_unmit:.1}")),
            ("E_K_UNMITIGATED_JOULES", format!("{ek_unmit:.1}")),
            ("V_RESIDUAL_MPS", format!("{v_residual:.1}")),
            ("E_K_MITIGATED_JOULES", format!("{ek_mit:.1}")),
            ("MITIGATED_KINETIC_ENERGY_J", format!("{ek_mit:.1}")),
            ("V_TERMINAL_UNMITIGATED_MPS", format!("{v_max:.1}")),
            ("V_TERMINAL_MITIGATED_MPS", format!("{v_residual:.1}")),
            ("SAFE_CONTAINMENT_VELOCITY_MS", format!("{v_residual:.1}")),
            ("CONTAINMENT_DEPLOY_TIME_MAX_S", "0.5".to_string()),
        ] {
            self.parameter_bindings.entry(k.to_string()).or_insert(val);
        }
    }

    /// Populates `DOMAIN_RISK_ASSESSMENT_CLAUSES` if not explicitly bound.
    /// Inputs: `&mut self` | Outputs: `()` | Invariants: non-empty risk clause binding
    ///Errors: never fails | Complexity: O(1)
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-DERIVE-DOMAIN-RISK-ASSESSMENT] | SysML v2: ActionDef(SysMLParameterBindingEngine::_derive_domain_risk_assessment)
    pub fn _derive_domain_risk_assessment(&mut self) {
        if !self
            .parameter_bindings
            .contains_key("DOMAIN_RISK_ASSESSMENT_CLAUSES")
        {
            let clauses = "<!-- Domain-specific risk containment and safety integrity clauses derived from active domain schema -->";
            self.parameter_bindings.insert(
                "DOMAIN_RISK_ASSESSMENT_CLAUSES".to_string(),
                clauses.to_string(),
            );
        }
    }

    /// Populates `DOMAIN_REGULATORY_STANDARDS_TABLE_ROWS` with baseline ISO/IEC/IEEE lifecycle standards if unset.
    /// Inputs: `&mut self` | Outputs: `()` | Invariants: valid Markdown table rows
    ///Errors: never fails | Complexity: O(1)
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-DERIVE-DOMAIN-REGULATORY-STANDARDS] | SysML v2: ActionDef(SysMLParameterBindingEngine::_derive_domain_regulatory_standards)
    pub fn _derive_domain_regulatory_standards(&mut self) {
        let existing = self
            .parameter_bindings
            .get("DOMAIN_REGULATORY_STANDARDS_TABLE_ROWS")
            .or_else(|| {
                self.parameter_bindings
                    .get("REGULATORY_STANDARDS_TABLE_ROWS")
            });

        let table_rows = if let Some(rows) = existing {
            rows.clone()
        } else {
            let rows = [
                "| Standard Identifier | Standard Organization | Scope & Operational Governance | Applicable Life Cycle Clauses & Safety Objectives |",
                "| ISO/IEC/IEEE 15288 | ISO / IEC / IEEE | Systems and software engineering -- System life cycle processes | §6.4 Technical processes, §6.4.2 Stakeholder requirements definition |",
                "| ISO/IEC/IEEE 29148 | ISO / IEC / IEEE | Systems and software engineering -- Life cycle processes -- Requirements engineering | §6.4.2 Concept of operations (ConOps), §6.4.3 Operational concept (OpsCon) |",
                "| Generic Functional Safety Standard | Standard Body | Functional safety and risk mitigation baseline | System safety integrity, hazard mitigation, and verification coverage |",
            ];
            rows.join("\n")
        };

        self.parameter_bindings.insert(
            "DOMAIN_REGULATORY_STANDARDS_TABLE_ROWS".to_string(),
            table_rows.clone(),
        );
        self.parameter_bindings
            .insert("REGULATORY_STANDARDS_TABLE_ROWS".to_string(), table_rows);
    }

    /// Derives battery capacity, endurance, sustainable power, and energy reserve thresholds.
    /// Inputs: `&mut self` | Outputs: `()` | Invariants: $E_{\mathrm{bingo}} = E_{\mathrm{return}} + E_{\mathrm{divert}} + E_{\mathrm{reserve}} + E_{\mathrm{contingency}}$
    ///Errors: uses default endurance/capacity if unset | Complexity: O(1)
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-DERIVE-ENERGY-BUDGETS] | SysML v2: ActionDef(SysMLParameterBindingEngine::_derive_energy_budgets)
    pub fn _derive_energy_budgets(&mut self) {
        let t_hours: f64 = self
            .parameter_bindings
            .get("ENDURANCE_HOURS")
            .and_then(|s| s.parse().ok())
            .unwrap_or(2.0);
        let t_sec = t_hours * 3600.0;
        let t_min = (t_hours * 60.0 * 10.0).round() / 10.0;

        let e_joules: f64 = self
            .parameter_bindings
            .get("BATTERY_CAPACITY_JOULES")
            .or_else(|| self.parameter_bindings.get("E_CAPACITY_JOULES"))
            .and_then(|s| s.parse().ok())
            .unwrap_or(500_000.0);

        let e_kwh = (e_joules / 3.6e6 * 10000.0).round() / 10000.0;

        self.parameter_bindings
            .entry("BATTERY_CAPACITY_KWH".to_string())
            .or_insert_with(|| e_kwh.to_string());
        self.parameter_bindings
            .entry("BATTERY_CAPACITY_JOULES".to_string())
            .or_insert_with(|| e_joules.to_string());
        self.parameter_bindings
            .entry("ENDURANCE_HOURS".to_string())
            .or_insert_with(|| t_hours.to_string());
        self.parameter_bindings
            .entry("ENDURANCE_NOMINAL_MIN".to_string())
            .or_insert_with(|| t_min.to_string());

        let p_sustainable = e_joules / t_sec;
        let p_nom = (0.70 * p_sustainable * 10.0).round() / 10.0;
        let p_peak = (2.0 * p_nom * 10.0).round() / 10.0;

        self.parameter_bindings
            .entry("TOTAL_POWER_NOMINAL_W".to_string())
            .or_insert_with(|| p_nom.to_string());
        self.parameter_bindings
            .entry("TOTAL_POWER_PEAK_W".to_string())
            .or_insert_with(|| p_peak.to_string());

        let e_reserve = (0.20 * e_joules * 10.0).round() / 10.0;
        let e_return = (0.35 * e_joules * 10.0).round() / 10.0;
        let e_divert = (0.15 * e_joules * 10.0).round() / 10.0;
        let e_contingency = (0.10 * e_joules * 10.0).round() / 10.0;
        let e_bingo = e_return + e_divert + e_reserve + e_contingency;

        self.parameter_bindings
            .entry("E_RESERVE_JOULES".to_string())
            .or_insert_with(|| e_reserve.to_string());
        self.parameter_bindings
            .entry("E_RETURN_JOULES".to_string())
            .or_insert_with(|| e_return.to_string());
        self.parameter_bindings
            .entry("E_DIVERT_JOULES".to_string())
            .or_insert_with(|| e_divert.to_string());
        self.parameter_bindings
            .entry("E_CONTINGENCY_JOULES".to_string())
            .or_insert_with(|| e_contingency.to_string());
        self.parameter_bindings
            .entry("E_CRITICAL_JOULES".to_string())
            .or_insert_with(|| e_bingo.to_string());
        self.parameter_bindings
            .entry("E_CRITICAL_THRESHOLD_JOULES".to_string())
            .or_insert_with(|| e_bingo.to_string());
        self.parameter_bindings
            .entry("E_BINGO_JOULES".to_string())
            .or_insert_with(|| e_bingo.to_string());
        self.parameter_bindings
            .entry("E_BINGO_THRESHOLD_JOULES".to_string())
            .or_insert_with(|| e_bingo.to_string());
    }
}
