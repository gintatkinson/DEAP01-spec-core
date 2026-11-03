//! Mass budgets, quadratic kinetic physics, energy reserves, and regulatory standards derivation (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).

use super::cmark_aliases::parse_f64_opt;
use super::cmark_engine::SysMLParameterBindingEngine;

impl SysMLParameterBindingEngine {
    /// [DOC-TIER: A] Derives subsystem mass partition budgets from total system mass and mass fraction percentages.
    /// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-DERIVE-MASS-BUDGETS] [FLS: fls_conops_cmark]
    /// Contract: Computes structure, avionics, propulsion, energy, payload, and containment mass allocations conserving `TOTAL_MTOW_KG`.
    /// Bounds & Faults: O(1) arithmetic; uses default fractions when optional percentage keys are absent.
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
            && [
                "MASS_FRACTION_AVIONICS_PCT",
                "MASS_FRACTION_PROPULSION_PCT",
                "MASS_FRACTION_ENERGY_PCT",
                "MASS_FRACTION_PAYLOAD_PCT",
            ]
            .iter()
            .all(|k| self.parameter_bindings.contains_key(*k));
        if has_fractions {
            let pct = |key: &str, fb: f64| -> f64 {
                self.parameter_bindings
                    .get(key)
                    .and_then(|s| parse_f64_opt(s))
                    .unwrap_or(fb)
                    / 100.0
            };
            let airframe_pct = self
                .parameter_bindings
                .get("MASS_FRACTION_STRUCTURE_PCT")
                .or_else(|| self.parameter_bindings.get("MASS_FRACTION_AIRFRAME_PCT"))
                .and_then(|s| parse_f64_opt(s))
                .unwrap_or(30.0)
                / 100.0;
            let airframe = (airframe_pct * mtow * 100.0).round() / 100.0;
            let avionics = (pct("MASS_FRACTION_AVIONICS_PCT", 15.0) * mtow * 100.0).round() / 100.0;
            let propulsion =
                (pct("MASS_FRACTION_PROPULSION_PCT", 25.0) * mtow * 100.0).round() / 100.0;
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

    /// Computes mitigated residual velocity (`V_RESIDUAL_MPS`) (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
    fn compute_residual_velocity(&self, m: f64, v_max: f64) -> f64 {
        if let Some(vr) = self
            .parameter_bindings
            .get("V_RESIDUAL_MPS")
            .or_else(|| self.parameter_bindings.get("V_RESIDUAL"))
            .and_then(|s| parse_f64_opt(s))
        {
            return vr;
        }
        let has_chute = self.has_ast_component("parachute")
            || self.has_ast_component("chute")
            || self.parameter_bindings.contains_key("S_CANOPY")
            || self.parameter_bindings.contains_key("PARACHUTE_DRAG_COEFFICIENT");
        if has_chute {
            let rho = self
                .parameter_bindings
                .get("AIR_DENSITY_KGM3")
                .or_else(|| self.parameter_bindings.get("FLUID_DENSITY_KGM3"))
                .and_then(|s| parse_f64_opt(s))
                .unwrap_or(1.225);
            let s_can = self
                .parameter_bindings
                .get("S_CANOPY")
                .or_else(|| self.parameter_bindings.get("S_MIT"))
                .and_then(|s| parse_f64_opt(s))
                .unwrap_or(84.0);
            let cd = self
                .parameter_bindings
                .get("PARACHUTE_DRAG_COEFFICIENT")
                .or_else(|| self.parameter_bindings.get("C_D_MIT"))
                .and_then(|s| parse_f64_opt(s))
                .unwrap_or(1.75);
            let denom = rho * s_can * cd;
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

    /// [DOC-TIER: A] Derives unmitigated and mitigated kinetic energy and terminal velocity bounds.
    /// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-DERIVE-QUADRATIC-PHYSICS] [FLS: fls_conops_cmark]
    /// Contract: Computes $E_k = \frac{1}{2} m v^2$ for unmitigated and mitigated states and binds velocity/energy tokens.
    /// Bounds & Faults: O(1) arithmetic; falls back to positive constants when parameters are omitted.
    pub fn _derive_quadratic_physics(&mut self) {
        let m = self._get_mtow_value();
        let v_max = self
            .parameter_bindings
            .get("V_MAX_MPS")
            .or_else(|| self.parameter_bindings.get("V_MAX_NOMINAL_MPS"))
            .and_then(|s| parse_f64_opt(s))
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

    /// [DOC-TIER: A] Populates `DOMAIN_RISK_ASSESSMENT_CLAUSES` if not explicitly bound.
    /// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-DERIVE-DOMAIN-RISK-ASSESSMENT] [FLS: fls_conops_cmark]
    /// Contract: Inserts a deterministic domain risk assessment comment clause if unset.
    /// Bounds & Faults: O(1) time and space; infallible.
    pub fn _derive_domain_risk_assessment(&mut self) {
        self.parameter_bindings
            .entry("DOMAIN_RISK_ASSESSMENT_CLAUSES".to_string())
            .or_insert_with(|| "<!-- Domain-specific risk containment and safety integrity clauses derived from active domain schema -->".to_string());
    }

    /// [DOC-TIER: A] Populates `DOMAIN_REGULATORY_STANDARDS_TABLE_ROWS` with baseline ISO/IEC/IEEE lifecycle standards if unset.
    /// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-DERIVE-DOMAIN-REGULATORY-STANDARDS] [FLS: fls_conops_cmark]
    /// Contract: Binds ISO/IEC/IEEE 15288 and 29148 regulatory table rows when no explicit rows are provided.
    /// Bounds & Faults: O(1) time and space; infallible.
    pub fn _derive_domain_regulatory_standards(&mut self) {
        let table_rows = self
            .parameter_bindings
            .get("DOMAIN_REGULATORY_STANDARDS_TABLE_ROWS")
            .or_else(|| self.parameter_bindings.get("REGULATORY_STANDARDS_TABLE_ROWS"))
            .cloned()
            .unwrap_or_else(|| {
                [
                    "| Standard Identifier | Standard Organization | Scope & Operational Governance | Applicable Life Cycle Clauses & Safety Objectives |",
                    "| ISO/IEC/IEEE 15288 | ISO / IEC / IEEE | Systems and software engineering -- System life cycle processes | §6.4 Technical processes, §6.4.2 Stakeholder requirements definition |",
                    "| ISO/IEC/IEEE 29148 | ISO / IEC / IEEE | Systems and software engineering -- Life cycle processes -- Requirements engineering | §6.4.2 Concept of operations (ConOps), §6.4.3 Operational concept (OpsCon) |",
                    "| Generic Functional Safety Standard | Standard Body | Functional safety and risk mitigation baseline | System safety integrity, hazard mitigation, and verification coverage |",
                ]
                .join("\n")
            });
        self.parameter_bindings
            .insert("DOMAIN_REGULATORY_STANDARDS_TABLE_ROWS".to_string(), table_rows.clone());
        self.parameter_bindings
            .insert("REGULATORY_STANDARDS_TABLE_ROWS".to_string(), table_rows);
    }

    /// [DOC-TIER: A] Derives battery capacity, endurance, sustainable power, and energy reserve thresholds.
    /// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-DERIVE-ENERGY-BUDGETS] [FLS: fls_conops_cmark]
    /// Contract: Computes nominal/peak power and reserve energy partitions ($E_{\mathrm{bingo}} = E_{\mathrm{return}} + E_{\mathrm{divert}} + E_{\mathrm{reserve}} + E_{\mathrm{contingency}}$).
    /// Bounds & Faults: O(1) arithmetic; uses default endurance and capacity when unset.
    pub fn _derive_energy_budgets(&mut self) {
        let t_hours = self
            .parameter_bindings
            .get("ENDURANCE_HOURS")
            .and_then(|s| parse_f64_opt(s))
            .unwrap_or(2.0);
        let t_sec = t_hours * 3600.0;
        let t_min = (t_hours * 60.0 * 10.0).round() / 10.0;
        let e_joules = self
            .parameter_bindings
            .get("BATTERY_CAPACITY_JOULES")
            .or_else(|| self.parameter_bindings.get("E_CAPACITY_JOULES"))
            .and_then(|s| parse_f64_opt(s))
            .unwrap_or(500_000.0);
        let e_kwh = (e_joules / 3.6e6 * 10000.0).round() / 10000.0;
        let p_nom = (0.70 * (e_joules / t_sec) * 10.0).round() / 10.0;
        let p_peak = (2.0 * p_nom * 10.0).round() / 10.0;
        let e_reserve = (0.20 * e_joules * 10.0).round() / 10.0;
        let e_return = (0.35 * e_joules * 10.0).round() / 10.0;
        let e_divert = (0.15 * e_joules * 10.0).round() / 10.0;
        let e_contingency = (0.10 * e_joules * 10.0).round() / 10.0;
        let e_bingo = e_return + e_divert + e_reserve + e_contingency;
        for (k, v) in [
            ("BATTERY_CAPACITY_KWH", e_kwh.to_string()),
            ("BATTERY_CAPACITY_JOULES", e_joules.to_string()),
            ("ENDURANCE_HOURS", t_hours.to_string()),
            ("ENDURANCE_NOMINAL_MIN", t_min.to_string()),
            ("TOTAL_POWER_NOMINAL_W", p_nom.to_string()),
            ("TOTAL_POWER_PEAK_W", p_peak.to_string()),
            ("E_RESERVE_JOULES", e_reserve.to_string()),
            ("E_RETURN_JOULES", e_return.to_string()),
            ("E_DIVERT_JOULES", e_divert.to_string()),
            ("E_CONTINGENCY_JOULES", e_contingency.to_string()),
            ("E_CRITICAL_JOULES", e_bingo.to_string()),
            ("E_CRITICAL_THRESHOLD_JOULES", e_bingo.to_string()),
            ("E_BINGO_JOULES", e_bingo.to_string()),
            ("E_BINGO_THRESHOLD_JOULES", e_bingo.to_string()),
        ] {
            self.parameter_bindings.entry(k.to_string()).or_insert(v);
        }
    }
}
