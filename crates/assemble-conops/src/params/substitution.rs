//! Token resolution, fallback defaults, multi-pass substitution, and domain ontology sanitization.

use chrono::Utc;

use crate::sanitize::sanitize_level_1b_operational_text;

use super::text_replace::{
    replace_altitude_agl, replace_astm_f3411, replace_placeholders, replace_roe_phrase,
    replace_word_case_insensitive,
};
use super::{SysMLParameterBindingEngine, DEFAULT_CONOPS_PARAMS};

impl SysMLParameterBindingEngine {
    /// Resolves a template token against bound parameters or returns the fallback default.
    /// Inputs: `token_name: &str, inline_default: Option<&str>` | Outputs: `String` | Invariants: case-insensitive fallback lookup
    ///Errors: never fails | Complexity: O(1) map lookup
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-RESOLVE-TOKEN] | SysML v2: ActionDef(SysMLParameterBindingEngine::resolve_token)
    pub fn resolve_token(&self, token_name: &str, inline_default: Option<&str>) -> String {
        if let Some(val) = self.parameter_bindings.get(token_name) {
            return val.clone();
        }
        let upper = token_name.to_uppercase();
        if let Some(val) = self.parameter_bindings.get(&upper) {
            return val.clone();
        }
        let lower = token_name.to_lowercase();
        if let Some(val) = self.parameter_bindings.get(&lower) {
            return val.clone();
        }
        self.get_fallback_default(token_name, inline_default)
    }

    /// Resolves numeric physical limit or suffix-based fallback defaults for `token_upper` (`[REQ-ASSEMBLE-CONOPS]`).
    fn match_numeric_or_suffix_fallback(token_upper: &str) -> String {
        match token_upper {
            "WEIGHT_CRIT_1" => "0.40".to_string(),
            "WEIGHT_CRIT_2" => "0.35".to_string(),
            "WEIGHT_CRIT_3" => "0.25".to_string(),
            "SCORE_A_1" | "SCORE_B_2" | "SCORE_C_1" => "1.0".to_string(),
            "SCORE_A_2" | "SCORE_B_3" | "SCORE_C_3" => "0.8".to_string(),
            "SCORE_A_3" | "SCORE_B_1" | "SCORE_C_2" => "0.9".to_string(),
            "WEIGHTED_SCORE_A" => "0.90".to_string(),
            "WEIGHTED_SCORE_B" => "0.79".to_string(),
            "WEIGHTED_SCORE_C" => "0.80".to_string(),
            "TOTAL_MASS_KG" | "M_MAX_LIMIT_KG" | "TOTAL_MTOW_KG" | "MTOW_MAX_KG" => "50.0".to_string(),
            "M_MAX_NOMINAL_KG" | "MTOW_NOMINAL_KG" => "45.0".to_string(),
            "PAYLOAD_MAX_KG" | "V_MIN_NOMINAL_MPS" => "5.0".to_string(),
            "PAYLOAD_NOMINAL_KG" => "3.0".to_string(),
            "DIM_MAX_L_M" => "2.5".to_string(),
            "DIM_MAX_W_M" => "1.5".to_string(),
            "DIM_MAX_H_M" => "1.0".to_string(),
            "DIM_NOM_L_M" => "2.0".to_string(),
            "DIM_NOM_W_M" => "1.2".to_string(),
            "DIM_NOM_H_M" => "0.8".to_string(),
            "V_CRUISE_MIN_MPS" | "WIND_LIMIT_MAX_MPS" => "12.0".to_string(),
            "V_CRUISE_MAX_MPS" => "24.0".to_string(),
            "V_CRUISE_NOMINAL_MPS" => "18.0".to_string(),
            "V_MAX_MPS" => "25.0".to_string(),
            "V_MAX_NOMINAL_MPS" => "22.0".to_string(),
            "V_MIN_MAX_MPS" => "8.0".to_string(),
            "SPATIAL_LIMIT_MAX_M" => "5000.0".to_string(),
            "SPATIAL_LIMIT_NOMINAL_M" => "4000.0".to_string(),
            "SPATIAL_LIMIT_UNIT" => "m".to_string(),
            "C2_RANGE_MIN_KM" | "WIND_LIMIT_NOMINAL_MPS" => "10.0".to_string(),
            "C2_RANGE_NOMINAL_KM" => "15.0".to_string(),
            "ENDURANCE_MIN_MIN" => "60.0".to_string(),
            "ENDURANCE_NOMINAL_MIN" => "90.0".to_string(),
            "TEMP_MIN_DEGC" => "-20.0".to_string(),
            "TEMP_MAX_DEGC" => "+55.0".to_string(),
            "TEMP_NOMINAL_DEGC" => "+25.0".to_string(),
            "INGRESS_PROTECTION_RATING" => "IP54".to_string(),
            "INGRESS_PROTECTION_TARGET" => "IP67".to_string(),
            _ if token_upper.ends_with("_PERCENT") || token_upper.ends_with("_PCT") => "10.0".to_string(),
            _ if token_upper.ends_with("_KG") => "5.0".to_string(),
            _ if token_upper.ends_with("_MPS") => "15.0".to_string(),
            _ if token_upper.ends_with("_KM") => "10.0".to_string(),
            _ if token_upper.ends_with("_M") => "100.0".to_string(),
            _ if token_upper.ends_with("_SEC") || token_upper.ends_with("_S") => "1.0".to_string(),
            _ if token_upper.ends_with("_MIN") => "30.0".to_string(),
            _ if token_upper.ends_with("_COUNT") => "5".to_string(),
            _ => "N/A".to_string(),
        }
    }

    /// Computes a deterministic fallback default value for an unbound template token.
    /// Inputs: `token_name: &str, inline_default: Option<&str>` | Outputs: `String` | Invariants: prioritizes `inline_default`
    ///Errors: never fails | Complexity: O(K) over `DEFAULT_CONOPS_PARAMS`
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-GET-FALLBACK-DEFAULT] | SysML v2: ActionDef(SysMLParameterBindingEngine::get_fallback_default)
    pub fn get_fallback_default(&self, token_name: &str, inline_default: Option<&str>) -> String {
        if let Some(def) = inline_default {
            return def.to_string();
        }
        let token_upper = token_name.to_uppercase();
        if let Some(val) = self.parameter_bindings.get(&token_upper) {
            return val.clone();
        }
        for &(k, v) in DEFAULT_CONOPS_PARAMS {
            if token_upper == k {
                return v.to_string();
            }
        }
        let sys_id = self
            .inferred_system_identifier
            .as_deref()
            .unwrap_or("AutonomousCyberPhysicalSystem");

        match token_upper.as_str() {
            "SYSTEM_IDENTIFIER" | "MISSION_SYSTEM_NAME" | "SYSTEM_NAME" => sys_id.to_string(),
            "DOCUMENT_VERSION" => "1.0.0".to_string(),
            "DOCUMENT_DATE" => Utc::now().date_naive().to_string(),
            "SECURITY_CLASSIFICATION" => "UNCLASSIFIED // PUBLIC RELEASE".to_string(),
            "TARGET_SYSTEM_REALIZATION" => "Cyber-Physical System (Hardware / Software / MBD)".to_string(),
            "AUTHORING_ORGANIZATION" => "Systems Engineering Directorate".to_string(),
            "OPERATIONAL_DOMAIN" => "Cyber-Physical Autonomous Systems".to_string(),
            "OPERATIONAL_BOUNDARIES" => "Defined operational theater within designated geographic boundary.".to_string(),
            "STAKEHOLDER_ROSTER" => "Operations Officer, Lead Systems Engineer, Safety Officer, Operator in Command.".to_string(),
            "CURRENT_OPERATIONAL_BASELINE" => "Legacy manual / tele-operated baseline with analog telemetry.".to_string(),
            "OPERATIONAL_DEFICIENCIES" => "Lack of autonomous failsafe containment, manual telemetry latency, non-deterministic failover.".to_string(),
            "MISSION_DRIVERS_AND_VALUE_PROPOSITION" => "High-assurance autonomous operation with deterministic safety containment.".to_string(),
            "TRADE_OFF_ANALYSIS" => "Dedicated backup communication link vs payload mass and thermal budget allocation.".to_string(),
            "USER_CLASSES_AND_STAKEHOLDERS" => "- **UCL-01 System Operator (SO):** Direct supervisory mission management and boundary oversight.\n- **UCL-02 Range Safety Officer (RSO):** Operational containment enforcement and failsafe override authority.\n- **UCL-03 Payload Specialist (PS):** Multi-modal sensor data interpretation and payload stream management.\n- **UCL-04 Maintenance Technician (MT):** O-Level pre-operation inspections, modular LRU swaps, and BIT checks.\n- **UCL-05 Safety Monitor (SM):** Continuous perimeter monitoring, environmental anomaly detection, and safety oversight.".to_string(),
            "OPERATIONAL_LIFECYCLE_MODES" => "- **Phase_Startup:** Power-on Built-In-Test (PBIT), sensor alignment, and pre-operation validation.\n- **Phase_NominalExecution:** Autonomous mission start, state corridor tracking, and real-time telemetry streaming.\n- **Phase_DegradedMode:** Non-critical sensor failover, PACE datalink fallback, and degraded envelope limits.\n- **Phase_ContingencyFailsafe:** Autonomous contingency execution, divert to recovery site, or state containment.\n- **Phase_SecureShutdown:** Autonomous precision arrival, power de-energization, and diagnostic blackbox archival.\n- **Phase_MaintenanceMode:** Diagnostic telemetry offload, calibration, firmware update, and hardware inspection.".to_string(),
            other => Self::match_numeric_or_suffix_fallback(other),
        }
    }

    /// Performs up to 3 passes of `{{TOKEN}}` substitution followed by domain ontology and Level 1B sanitization.
    /// Inputs: `content: &str` | Outputs: `String` | Invariants: bounded 3-pass expansion, zero unresolved placeholders
    ///Errors: never panics | Complexity: O(N) over content length
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-SUBSTITUTE] | SysML v2: ActionDef(SysMLParameterBindingEngine::substitute)
    pub fn substitute(&self, content: &str) -> String {
        if content.is_empty() {
            return String::new();
        }

        let mut current = content.to_string();

        for _ in 0..3 {
            let (next, changed) = replace_placeholders(&current, |key, default_val| {
                self.resolve_token(key, default_val)
            });

            if !changed || next == current {
                break;
            }
            current = next;
        }

        self.apply_domain_ontology_sanitization(&mut current);

        let sys_target = self
            .parameter_bindings
            .get("SYSTEM_IDENTIFIER")
            .or_else(|| self.parameter_bindings.get("SYSTEM_NAME"))
            .or(self.inferred_system_identifier.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("Autonomous Cyber-Physical System");

        current = current.replace(
            "the Abstract Cyber-Physical System Archetype",
            &format!("the {sys_target}"),
        );
        current = current.replace(
            "The Abstract Cyber-Physical System Archetype",
            &format!("The {sys_target}"),
        );
        current = current.replace(
            "an Abstract Cyber-Physical System Archetype",
            &format!("the {sys_target}"),
        );
        current = current.replace("Abstract Cyber-Physical System Archetype", sys_target);
        current = current.replace("Autonomous Cyber-Physical System Archetype", sys_target);
        current = current.replace("AutonomousSystemArchetype", sys_target);

        sanitize_level_1b_operational_text(&current)
    }

    /// Sanitizes domain-inappropriate parachute, altitude, and standard references in `current` (`[REQ-ASSEMBLE-CONOPS]`).
    fn sanitize_domain_parachute_and_standards(&self, current: &mut String) {
        let dom = self.detected_domain.to_lowercase();
        let has_chute_in_ast = self.has_ast_component("parachute")
            || self.has_ast_component("chute")
            || self.parameter_bindings.contains_key("S_CANOPY");

        let domain_override = match dom.as_str() {
            "medical" => Some(("failsafe joint brake", "JOINT_BRAKE", "mm", "IEC 62304 / ISO 14971")),
            "rail" => Some(("pneumatic emergency brake", "EMERGENCY_BRAKE", "m", "EN 50128 SIL 4")),
            "marine" => Some(("positive buoyancy drop-weight", "DROP_WEIGHT", "m Depth", "DNV-GL-ST-E403")),
            "space" => Some(("autonomous de-orbit propulsion", "DEORBIT_THRUSTER", "km", "ECSS-E-ST-40C")),
            _ => None,
        };

        if let Some((term, upper, alt_unit, std_str)) = domain_override {
            *current = replace_word_case_insensitive(current, "parachute", term);
            *current = current.replace("PARACHUTE", upper);
            *current = replace_altitude_agl(current, alt_unit);
            *current = replace_astm_f3411(current, std_str);
        } else if self
            .parameter_bindings
            .get("LIFECYCLE_TYPE")
            .is_some_and(|s| s == "EXPENDABLE_DISPOSAL")
            || self
                .parameter_bindings
                .get("IS_EXPENDABLE")
                .is_some_and(|s| s == "true")
        {
            *current = replace_word_case_insensitive(current, "parachute", "controlled impact containment");
            *current = current.replace("PARACHUTE", "CONTAINMENT");
        } else if !has_chute_in_ast {
            let rec_term = self
                .parameter_bindings
                .get("RECOVERY_DEVICE_TERM")
                .cloned()
                .unwrap_or_else(|| "failsafe containment".to_string());
            let rec_upper = self
                .parameter_bindings
                .get("CONTAINMENT_MITIGATION_TYPE")
                .cloned()
                .unwrap_or_else(|| "FAILSAFE_CONTAINMENT".to_string());
            *current = replace_word_case_insensitive(current, "parachute", &rec_term);
            *current = current.replace("PARACHUTE", &rec_upper);
        } else if self.is_non_aircraft {
            *current = replace_word_case_insensitive(current, "parachute", "recovery system");
            *current = current.replace("PARACHUTE", "RECOVERY");
        }
    }

    /// Sanitizes domain-inappropriate recovery, altitude, standard, and kinetic/ROE phrases in `current`.
    /// Inputs: `current: &mut String` | Outputs: `()` | Invariants: preserves AST-grounded components
    ///Errors: never panics | Complexity: O(N) over string length
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-APPLY-DOMAIN-ONTOLOGY-SANITIZATION] | SysML v2: ActionDef(SysMLParameterBindingEngine::apply_domain_ontology_sanitization)
    pub fn apply_domain_ontology_sanitization(&self, current: &mut String) {
        self.sanitize_domain_parachute_and_standards(current);

        let pfx = self
            .parameter_bindings
            .get("INTERLOCK_PREFIX")
            .map(|s| s.as_str())
            .unwrap_or("INT");
        for idx in 1..=6 {
            let target = format!("ROE-0{idx}");
            let repl = format!("{pfx}-0{idx}");
            *current = replace_word_case_insensitive(current, &target, &repl);
            if pfx == "INT" {
                let target_saf = format!("SAF-0{idx}");
                *current = replace_word_case_insensitive(current, &target_saf, &repl);
            }
        }
        *current = replace_roe_phrase(current);
        *current =
            replace_word_case_insensitive(current, "weapons release", "high-consequence actuation");
        *current = replace_word_case_insensitive(
            current,
            "collateral damage",
            "adjacent operational risk",
        );
    }
}
