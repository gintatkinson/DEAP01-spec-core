//! Semantic parameter alias mapping, domain detection, and AST physical limit derivation.

use std::collections::BTreeMap;
use std::fs;

use serde_json::Value;

use super::SysMLParameterBindingEngine;

/// Extracts the first signed/decimal numeric substring from a raw attribute value string.
/// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-EXTRACT-NUMERIC-SUBSTRING] | SysML v2: ActionDef(extract_numeric_substring)
pub(super) fn extract_numeric_substring(val: &str) -> Option<&str> {
    let bytes = val.as_bytes();
    let mut i = 0;
    while let Some(&b) = bytes.get(i) {
        let is_sign = (b == b'+' || b == b'-')
            && (bytes.get(i + 1).is_some_and(u8::is_ascii_digit)
                || (bytes.get(i + 1) == Some(&b'.')
                    && bytes.get(i + 2).is_some_and(u8::is_ascii_digit)));
        let is_digit = b.is_ascii_digit();
        let is_dot_digit = b == b'.' && bytes.get(i + 1).is_some_and(u8::is_ascii_digit);
        if is_sign || is_digit || is_dot_digit {
            let start = i;
            let mut j = if is_sign { i + 1 } else { i };
            let mut has_dot = false;
            let mut has_digits = false;
            while let Some(&bj) = bytes.get(j) {
                if bj.is_ascii_digit() {
                    has_digits = true;
                    j += 1;
                } else if bj == b'.' && !has_dot
                    && (has_digits || bytes.get(j + 1).is_some_and(u8::is_ascii_digit))
                {
                    has_dot = true;
                    j += 1;
                } else {
                    break;
                }
            }
            if has_digits {
                return val.get(start..j);
            }
        }
        i += 1;
    }
    None
}

impl SysMLParameterBindingEngine {
    /// Maps identity and velocity keys into `aliases` (`[REQ-ASSEMBLE-CONOPS]`).
    fn map_identity_and_velocity_aliases(
        &self,
        lower: &str,
        val: &str,
        num_val: &str,
        aliases: &mut BTreeMap<&'static str, String>,
    ) -> bool {
        if lower.contains("system_identifier")
            || lower.contains("mission_system_name")
            || ((lower == "system" || lower == "system_name")
                && !self.parameter_bindings.contains_key("SYSTEM_IDENTIFIER"))
        {
            for k in ["SYSTEM_IDENTIFIER", "MISSION_SYSTEM_NAME", "SYSTEM_NAME"] {
                aliases.insert(k, val.to_string());
            }
        } else if lower.contains("cruise") {
            for k in ["V_CRUISE_NOMINAL_MPS", "MAX_CRUISE_SPEED_MS", "CRUISE_SPEED_MPS"] {
                aliases.insert(k, num_val.to_string());
            }
            if lower.contains("min") {
                aliases.insert("V_CRUISE_MIN_MPS", num_val.to_string());
            } else if lower.contains("max") {
                aliases.insert("V_CRUISE_MAX_MPS", num_val.to_string());
            } else if let Ok(v) = num_val.parse::<f64>() {
                aliases.insert("V_CRUISE_MIN_MPS", format!("{:.1}", v * 0.7));
                aliases.insert("V_CRUISE_MAX_MPS", format!("{:.1}", v * 1.3));
            }
        } else if ["max_speed", "v_max", "max_horizontal", "speed_max", "terminal_speed", "dive_speed"]
            .iter()
            .any(|k| lower.contains(k))
            || lower == "vmax"
        {
            for k in ["V_MAX_MPS", "MAX_SPEED_MS", "MAX_HORIZONTAL_SPEED_MPS"] {
                aliases.insert(k, num_val.to_string());
            }
            if let Ok(v) = num_val.parse::<f64>() {
                aliases.insert("V_MAX_NOMINAL_MPS", format!("{:.1}", v * 0.85));
            }
        } else if ["stall", "min_speed", "v_min"].iter().any(|k| lower.contains(k)) || lower == "vmin" {
            for k in ["V_MIN_MAX_MPS", "V_STALL_MAX_MPS", "STALL_SPEED_MPS"] {
                aliases.insert(k, num_val.to_string());
            }
            if let Ok(v) = num_val.parse::<f64>() {
                let s = format!("{:.1}", v * 0.8);
                aliases.insert("V_MIN_NOMINAL_MPS", s.clone());
                aliases.insert("V_STALL_NOMINAL_MPS", s);
            }
        } else {
            return false;
        }
        true
    }

    /// Maps mass, payload, and endurance keys into `aliases` (`[REQ-ASSEMBLE-CONOPS]`).
    fn map_mass_and_endurance_aliases(
        &self,
        lower: &str,
        num_val: &str,
        aliases: &mut BTreeMap<&'static str, String>,
    ) -> bool {
        if ["mass", "weight", "mtow", "m_max"].iter().any(|k| lower.contains(k))
            && !lower.contains("payload")
        {
            let is_total = ["total", "system", "mtow", "takeoff", "m_max", "gross"]
                .iter()
                .any(|k| lower.contains(k));
            if is_total || !self.parameter_bindings.contains_key("M_MAX_LIMIT_KG") {
                for k in ["TOTAL_MTOW_KG", "MTOW_MAX_KG", "TOTAL_MASS_KG", "M_MAX_LIMIT_KG"] {
                    aliases.insert(k, num_val.to_string());
                }
                let nom_opt = if lower.contains("nominal") {
                    Some(num_val.to_string())
                } else {
                    num_val.parse::<f64>().ok().map(|m| format!("{:.1}", m * 0.9))
                };
                if let Some(nom) = nom_opt {
                    aliases.insert("MTOW_NOMINAL_KG", nom.clone());
                    aliases.insert("M_MAX_NOMINAL_KG", nom);
                }
            }
        } else if lower.contains("payload") {
            aliases.insert("PAYLOAD_MAX_KG", num_val.to_string());
            if lower.contains("nominal") {
                aliases.insert("PAYLOAD_NOMINAL_KG", num_val.to_string());
            } else if let Ok(p) = num_val.parse::<f64>() {
                aliases.insert("PAYLOAD_NOMINAL_KG", format!("{:.1}", p * 0.8));
            }
        } else if ["endurance", "flight_time", "mission_duration"].iter().any(|k| lower.contains(k)) {
            aliases.insert("ENDURANCE_NOMINAL_MIN", num_val.to_string());
            let min_str = num_val
                .parse::<f64>()
                .map(|e| format!("{:.1}", e * 0.67))
                .unwrap_or_else(|_| num_val.to_string());
            aliases.insert("ENDURANCE_MIN_MIN", min_str);
        } else {
            return false;
        }
        true
    }

    /// Maps dimensional, spatial, and range keys into `aliases` (`[REQ-ASSEMBLE-CONOPS]`).
    fn map_dim_and_range_aliases(
        &self,
        lower: &str,
        num_val: &str,
        aliases: &mut BTreeMap<&'static str, String>,
    ) -> bool {
        if lower.contains("length") || lower.contains("dim_l") || lower.contains("dim_max_l") {
            aliases.insert("DIM_MAX_L_M", num_val.to_string());
            if let Ok(d) = num_val.parse::<f64>() {
                aliases.insert("DIM_NOM_L_M", format!("{:.1}", d * 0.9));
            }
        } else if lower.contains("width") || lower.contains("dim_w") || lower.contains("dim_max_w") {
            aliases.insert("DIM_MAX_W_M", num_val.to_string());
            if let Ok(d) = num_val.parse::<f64>() {
                aliases.insert("DIM_NOM_W_M", format!("{:.1}", d * 0.9));
            }
        } else if lower.contains("height") || lower.contains("dim_h") || lower.contains("dim_max_h") {
            aliases.insert("DIM_MAX_H_M", num_val.to_string());
            if let Ok(d) = num_val.parse::<f64>() {
                aliases.insert("DIM_NOM_H_M", format!("{:.1}", d * 0.9));
            }
        } else if [
            "sector", "spatial_limit", "spatial_boundary", "boundary_radius",
            "containment_radius", "geofence", "altitude_limit", "service_ceiling",
        ]
        .iter()
        .any(|k| lower.contains(k))
        {
            aliases.insert("SPATIAL_LIMIT_MAX_M", num_val.to_string());
            if let Ok(s) = num_val.parse::<f64>() {
                aliases.insert("SPATIAL_LIMIT_NOMINAL_M", format!("{:.1}", s * 0.8));
            }
            aliases.insert("SPATIAL_LIMIT_UNIT", "m".to_string());
        } else if ["detection_range", "sensor_range", "radar_range"].iter().any(|k| lower.contains(k)) {
            aliases.insert("SENSOR_DETECTION_RANGE_M", num_val.to_string());
            if !self.parameter_bindings.contains_key("SPATIAL_LIMIT_MAX_M") {
                aliases.insert("SPATIAL_LIMIT_MAX_M", num_val.to_string());
                if let Ok(s) = num_val.parse::<f64>() {
                    aliases.insert("SPATIAL_LIMIT_NOMINAL_M", format!("{:.1}", s * 0.8));
                }
            }
            if let Ok(m) = num_val.parse::<f64>() {
                let km = if m >= 100.0 { m / 1000.0 } else { m };
                aliases.insert("C2_RANGE_NOMINAL_KM", format!("{:.1}", km));
                aliases.insert("C2_RANGE_MIN_KM", format!("{:.1}", km * 0.6));
            }
        } else {
            return false;
        }
        true
    }

    /// Maps effector range, C2 range, latency, and environmental keys into `aliases` (`[REQ-ASSEMBLE-CONOPS]`).
    fn map_latency_and_env_aliases(
        &self,
        lower: &str,
        val: &str,
        num_val: &str,
        aliases: &mut BTreeMap<&'static str, String>,
    ) {
        if [
            "engagement_range", "effector_range", "interception_range",
            "mitigation_range", "effector_envelope",
        ]
        .iter()
        .any(|k| lower.contains(k))
        {
            aliases.insert("EFFECTOR_ENGAGEMENT_RANGE_M", num_val.to_string());
            aliases.insert("ENGAGEMENT_RANGE_M", num_val.to_string());
        } else if ["c2_range", "datalink_range", "comm_range"].iter().any(|k| lower.contains(k)) {
            aliases.insert("C2_RANGE_NOMINAL_KM", num_val.to_string());
            if let Ok(km) = num_val.parse::<f64>() {
                aliases.insert("C2_RANGE_MIN_KM", format!("{:.1}", km * 0.6));
            }
        } else if ["latency", "response_time", "loop_time"].iter().any(|k| lower.contains(k)) {
            for k in [
                "MAX_INFERENCE_LATENCY_MS", "MAX_LATENCY_MS",
                "PROCESSING_LATENCY_MS", "TAU_RTT_MAX_MS",
            ] {
                aliases.insert(k, num_val.to_string());
            }
        } else if ["temperature_min", "temp_min", "operating_temp_min"].iter().any(|k| lower.contains(k)) {
            aliases.insert("TEMP_MIN_DEGC", num_val.to_string());
            aliases.insert("OPERATING_TEMPERATURE_MIN_C", num_val.to_string());
        } else if ["temperature_max", "temp_max", "operating_temp_max"].iter().any(|k| lower.contains(k)) {
            aliases.insert("TEMP_MAX_DEGC", num_val.to_string());
            aliases.insert("OPERATING_TEMPERATURE_MAX_C", num_val.to_string());
        } else if lower.contains("temp_nominal") || lower.contains("temperature_nominal") {
            aliases.insert("TEMP_NOMINAL_DEGC", num_val.to_string());
        } else if lower.contains("wind") || lower.contains("gust") {
            aliases.insert("WIND_LIMIT_MAX_MPS", num_val.to_string());
            if let Ok(w) = num_val.parse::<f64>() {
                aliases.insert("WIND_LIMIT_NOMINAL_MPS", format!("{:.1}", w * 0.8));
            }
        } else if lower.contains("ip_rating") || lower.contains("ingress_protection") {
            aliases.insert("INGRESS_PROTECTION_RATING", val.to_string());
            aliases.insert("INGRESS_PROTECTION_TARGET", val.to_string());
        }
    }

    /// Maps raw schema attribute keys to canonical ConOps physical and operational limit tokens.
    /// Inputs: `key: &str, val: &str` | Outputs: `()` | Invariants: deterministic alias expansion
    ///Errors: ignores unparseable numeric substrings for scaled bounds | Complexity: O(K) over key length
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-MAP-SEMANTIC-ALIASES] | SysML v2: ActionDef(SysMLParameterBindingEngine::_map_semantic_aliases)
    pub fn _map_semantic_aliases(&mut self, key: &str, val: &str) {
        let lower = key.to_lowercase().replace(['-', ' '], "_");
        let num_val = extract_numeric_substring(val).unwrap_or(val).to_string();
        let mut aliases = BTreeMap::new();

        if !self.map_identity_and_velocity_aliases(&lower, val, &num_val, &mut aliases)
            && !self.map_mass_and_endurance_aliases(&lower, &num_val, &mut aliases)
            && !self.map_dim_and_range_aliases(&lower, &num_val, &mut aliases)
        {
            self.map_latency_and_env_aliases(&lower, val, &num_val, &mut aliases);
        }

        for (k, v) in aliases {
            self.parameter_bindings.insert(k.to_string(), v);
            self.explicit_keys.insert(k.to_string());
        }
    }

    /// Detects the active operational domain type from parameter bindings or `.pipeline/schema-digest.json`.
    /// Inputs: `&self` | Outputs: `String` | Invariants: defaults to `"generic"` when unspecified
    ///Errors: falls back to `"generic"` on missing digest | Complexity: O(1) bounded lookup
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-DETECT-DOMAIN-TYPE] | SysML v2: ActionDef(SysMLParameterBindingEngine::_detect_domain_type)
    pub fn _detect_domain_type(&self) -> String {
        for key in &[
            "DOMAIN_TYPE",
            "TARGET_DOMAIN",
            "OPERATIONAL_DOMAIN",
            "DOMAIN",
        ] {
            if let Some(val) = self.parameter_bindings.get(*key) {
                let clean = val.trim().to_lowercase();
                if !clean.is_empty()
                    && !["unknown", "none", "null", "undefined"].contains(&clean.as_str())
                {
                    let last = clean.split("::").last().unwrap_or(&clean).trim();
                    return last.to_string();
                }
            }
        }

        let mut curr = self.workspace_dir.clone();
        for _ in 0..5 {
            let digest_path = curr.join(".pipeline").join("schema-digest.json");
            if digest_path.is_file() {
                if let Ok(content) = fs::read_to_string(&digest_path) {
                    if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(&content) {
                        for k in &[
                            "domain",
                            "domain_type",
                            "target_domain",
                            "operational_domain",
                        ] {
                            if let Some(Value::String(s)) = map.get(*k) {
                                let clean = s.trim().to_lowercase();
                                if !clean.is_empty()
                                    && !["unknown", "none", "null"].contains(&clean.as_str())
                                {
                                    return clean;
                                }
                            }
                        }
                    }
                }
                break;
            }
            if let Some(parent) = curr.parent() {
                if parent == curr {
                    break;
                }
                curr = parent.to_path_buf();
            } else {
                break;
            }
        }

        "generic".to_string()
    }

    /// Resolves the maximum operating gross mass (MTOW) in kilograms from bound parameters.
    /// Inputs: `&self` | Outputs: `f64` | Invariants: positive mass fallback (`50.0`)
    ///Errors: falls back to `50.0` if no valid mass parameter is bound | Complexity: O(1)
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-GET-MTOW-VALUE] | SysML v2: ActionDef(SysMLParameterBindingEngine::_get_mtow_value)
    pub fn _get_mtow_value(&self) -> f64 {
        for key in &[
            "TOTAL_MTOW_KG",
            "TOTAL_MASS_KG",
            "M_MAX_LIMIT_KG",
            "MTOW_NOMINAL_KG",
            "MTOW_MAX_KG",
            "SYSTEM_MASS_MAX_KG",
            "SYSTEM_MASS_KG",
        ] {
            if let Some(val) = self.parameter_bindings.get(*key) {
                if let Some(m) = extract_numeric_substring(val) {
                    if let Ok(v) = m.parse::<f64>() {
                        return v;
                    }
                }
            }
        }
        50.0
    }

    /// Derives ConOps Section 1.3.3 master physical and operational limits from AST nodes.
    /// Inputs: `&mut self` | Outputs: `()` | Invariants: maps part-level attributes to system physical limits
    ///Errors: never panics | Complexity: O(P * A) over AST parts and attributes
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-DERIVE-MASTER-PHYSICAL-LIMITS-FROM-AST] | SysML v2: ActionDef(SysMLParameterBindingEngine::derive_master_physical_limits_from_ast)
    pub fn derive_master_physical_limits_from_ast(&mut self) {
        let parts = self.ast_parts.clone();
        for part in &parts {
            for (attr_key, attr_val) in &part.attributes {
                self._map_semantic_aliases(attr_key, attr_val);
            }
            if let Some(mass) = part.mass_kg {
                let m_str = format!("{mass:.1}");
                if !self.parameter_bindings.contains_key("M_MAX_LIMIT_KG") {
                    self._map_semantic_aliases("total_mass_kg", &m_str);
                }
            }
        }
    }
}
