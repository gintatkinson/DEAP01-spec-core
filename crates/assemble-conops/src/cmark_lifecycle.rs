//! Domain ontology, lifecycle contract derivation, token substitution, and operational text replacement (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).

use chrono::Utc;

use super::cmark_engine::{LifecycleType, SysMLParameterBindingEngine, DEFAULT_CONOPS_PARAMS};
use super::cmark_sanitize::sanitize_level_1b_operational_text;

/// Replaces `{{KEY}}` and `{{KEY:DEFAULT}}` tokens using `resolver` (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
pub(super) fn replace_placeholders<F: FnMut(&str, Option<&str>) -> String>(text: &str, mut resolver: F) -> (String, bool) {
    let bytes = text.as_bytes();
    let (mut result, mut idx, mut changed) = (String::with_capacity(text.len()), 0, false);
    while idx < bytes.len() {
        if idx + 4 <= bytes.len() && bytes.get(idx..idx + 2) == Some(b"{{") {
            let (start, mut j, mut found_end) = (idx, idx + 2, None);
            while j + 1 < bytes.len() && bytes.get(j) != Some(&b'\n') {
                if bytes.get(j..j + 2) == Some(b"}}") { found_end = Some(j); break; }
                j += 1;
            }
            if let Some(end_pos) = found_end {
                let inner = text.get(start + 2..end_pos).unwrap_or("");
                let (key, default_val) = inner.split_once(':').map_or((inner, None), |(k, d)| (k, Some(d)));
                if !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    result.push_str(&resolver(key, default_val));
                    changed = true;
                    idx = end_pos + 2;
                    continue;
                }
            }
        }
        let Some(ch) = text.get(idx..).and_then(|s| s.chars().next()) else { break };
        result.push(ch);
        idx += ch.len_utf8();
    }
    (result, changed)
}

/// Replaces whole-word occurrences of `target_word` case-insensitively (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
pub(super) fn replace_word_case_insensitive(text: &str, target_word: &str, replacement: &str) -> String {
    let (lower_text, lower_target, bytes) = (text.to_ascii_lowercase(), target_word.to_ascii_lowercase(), text.as_bytes());
    let (mut result, mut last_idx, mut search_idx) = (String::with_capacity(text.len()), 0, 0);
    while let Some(pos) = lower_text.get(search_idx..).and_then(|s| s.find(&lower_target)) {
        let (start, end) = (search_idx + pos, search_idx + pos + lower_target.len());
        let left_ok = start == 0 || !bytes.get(start - 1).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_');
        let right_ok = end == bytes.len() || !bytes.get(end).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_');
        if left_ok && right_ok {
            if let Some(prefix) = text.get(last_idx..start) { result.push_str(prefix); }
            result.push_str(replacement);
            (last_idx, search_idx) = (end, end);
        } else { search_idx = start + 1; }
    }
    if let Some(tail) = text.get(last_idx..) { result.push_str(tail); }
    result
}

/// Replaces `"altitude AGL"` or `"m AGL"` occurrences (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
pub(super) fn replace_altitude_agl(text: &str, replacement: &str) -> String {
    let (lower, bytes) = (text.to_ascii_lowercase(), text.as_bytes());
    let (mut result, mut last_idx, mut i) = (String::with_capacity(text.len()), 0, 0);
    while let Some(pos) = lower.get(i..).and_then(|s| s.find("agl")) {
        let (agl_start, agl_end) = (i + pos, i + pos + 3);
        let right_ok = agl_end == bytes.len() || !bytes.get(agl_end).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_');
        if right_ok && agl_start > 0 && bytes.get(agl_start - 1).is_some_and(u8::is_ascii_whitespace) {
            let mut prev = agl_start - 1;
            while prev > 0 && bytes.get(prev).is_some_and(u8::is_ascii_whitespace) { prev -= 1; }
            let (prev_end, mut word_start) = (prev + 1, prev);
            while word_start > 0 && bytes.get(word_start - 1).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_') { word_start -= 1; }
            let left_ok = word_start == 0 || !bytes.get(word_start - 1).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_');
            if left_ok && matches!(lower.get(word_start..prev_end).unwrap_or(""), "altitude" | "m") {
                if let Some(prefix) = text.get(last_idx..word_start) { result.push_str(prefix); }
                result.push_str(replacement);
                last_idx = agl_end;
            }
        }
        i = agl_end;
    }
    if let Some(tail) = text.get(last_idx..) { result.push_str(tail); }
    result
}

/// Replaces `"ASTM F3411"` or `"ASTM F3411-22a"` references (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
pub(super) fn replace_astm_f3411(text: &str, replacement: &str) -> String {
    let (lower, bytes) = (text.to_ascii_lowercase(), text.as_bytes());
    let (mut result, mut last_idx, mut i) = (String::with_capacity(text.len()), 0, 0);
    while let Some(pos) = lower.get(i..).and_then(|s| s.find("astm")) {
        let (start, mut j) = (i + pos, i + pos + 4);
        if start == 0 || !bytes.get(start - 1).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_') {
            while bytes.get(j).is_some_and(u8::is_ascii_whitespace) { j += 1; }
            if lower.get(j..).is_some_and(|s| s.starts_with("f3411")) {
                j += 5 + if lower.get(j + 5..).is_some_and(|s| s.starts_with("-22a")) { 4 } else { 0 };
                if let Some(prefix) = text.get(last_idx..start) { result.push_str(prefix); }
                result.push_str(replacement);
                last_idx = j;
                i = j;
                continue;
            }
        }
        i = start + 4;
    }
    if let Some(tail) = text.get(last_idx..) { result.push_str(tail); }
    result
}

/// Replaces `"Rules of Engagement"` and `"Rules of Engagement (ROE)"` (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
pub(super) fn replace_roe_phrase(text: &str) -> String {
    let (lower, bytes) = (text.to_ascii_lowercase(), text.as_bytes());
    let (mut result, mut last_idx, mut i) = (String::with_capacity(text.len()), 0, 0);
    while let Some(pos) = lower.get(i..).and_then(|s| s.find("rules of engagement")) {
        let start = i + pos;
        let mut end = start + "rules of engagement".len();
        let mut j = end;
        while bytes.get(j).is_some_and(|&b| b == b' ' || b == b'\t') { j += 1; }
        if lower.get(j..).is_some_and(|s| s.starts_with("(roe)")) { end = j + 5; }
        if let Some(prefix) = text.get(last_idx..start) { result.push_str(prefix); }
        result.push_str("operational safety interlocks");
        last_idx = end;
        i = end;
    }
    if let Some(tail) = text.get(last_idx..) { result.push_str(tail); }
    result
}

impl SysMLParameterBindingEngine {
    /// [DOC-TIER: A] Derives abstract MBSE operational safety interlock and positive condition verification tokens.
    /// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-DERIVE-DOMAIN-ONTOLOGY] [FLS: fls_conops_cmark]
    /// Contract: Populates `INTERLOCK_*`, `TARGET_VERIFICATION_*`, and `HIGH_CONSEQUENCE_ACTION` with zero kinetic/weapon bias.
    /// Bounds & Faults: O(1) time and space; infallible.
    pub fn _derive_domain_ontology(&mut self) {
        let dom = self.detected_domain.to_lowercase();
        self.is_non_aircraft = ["medical", "rail", "marine", "space", "industrial"].contains(&dom.as_str());
        self.is_civilian = true;
        self.parameter_bindings.entry("INTERLOCK_PREFIX".to_string()).or_insert_with(|| "INT".to_string());
        self.parameter_bindings.entry("INTERLOCK_SECTION_TITLE".to_string()).or_insert_with(|| "Safety Constraints & Subsystem Interlocks".to_string());
        self.parameter_bindings.entry("INTERLOCK_POLICY_NAME".to_string()).or_insert_with(|| "Operational Safety Interlocks".to_string());
        let pfx = self.parameter_bindings.get("INTERLOCK_PREFIX").cloned().unwrap_or_else(|| "INT".to_string());
        for idx in 1..=6 {
            self.parameter_bindings.entry(format!("INTERLOCK_0{idx}_TAG")).or_insert_with(|| format!("{pfx}-0{idx}"));
        }
        for (k, v) in [
            ("TARGET_VERIFICATION_LABEL", "Positive Condition Verification (PCV)"),
            ("TARGET_VERIFICATION_ACRONYM", "PCV"),
            ("TARGET_VERIFICATION_PHRASE", "positive condition verification"),
            ("HIGH_CONSEQUENCE_ACTION", "high-consequence actuation"),
            ("COLLATERAL_RISK_PHRASE", "adjacent operational risk"),
        ] {
            self.parameter_bindings.insert(k.to_string(), v.to_string());
        }
    }

    /// [DOC-TIER: A] Derives operational purpose, primary mission statement, and core capabilities list.
    /// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-DERIVE-OPERATIONAL-INTENT] [FLS: fls_conops_cmark]
    /// Contract: Populates `OPERATIONAL_PURPOSE`, `PRIMARY_OPERATIONAL_MISSION`, and `CORE_MISSION_CAPABILITIES`.
    /// Bounds & Faults: O(1) time and space; infallible.
    pub fn _derive_operational_intent(&mut self) {
        let sys_name = self.parameter_bindings.get("SYSTEM_IDENTIFIER").or_else(|| self.parameter_bindings.get("SYSTEM_NAME")).or(self.inferred_system_identifier.as_ref()).cloned().unwrap_or_else(|| "Autonomous Cyber-Physical System".to_string());
        let dom = self.detected_domain.clone();
        self.parameter_bindings.entry("OPERATIONAL_PURPOSE".to_string()).or_insert_with(|| format!("The primary operational purpose of {sys_name} is to execute deterministic autonomous operations in {dom} environments."));
        self.parameter_bindings.entry("PRIMARY_OPERATIONAL_MISSION".to_string()).or_insert_with(|| format!("The {sys_name} is engineered to execute high-assurance {dom} missions."));
        self.parameter_bindings.entry("CORE_CAPABILITY_1".to_string()).or_insert_with(|| format!("Autonomous trajectory tracking and corridor execution in {dom}."));
        self.parameter_bindings.entry("CORE_CAPABILITY_2".to_string()).or_insert_with(|| "Multi-modal sensor data fusion combining redundant state estimation sensors.".to_string());
        self.parameter_bindings.entry("CORE_CAPABILITY_3".to_string()).or_insert_with(|| "Real-time high-throughput telemetry streaming and deterministic command processing.".to_string());
        self.parameter_bindings.entry("CORE_CAPABILITY_4".to_string()).or_insert_with(|| "Deterministic failsafe state machine ensuring autonomous containment.".to_string());
        if !self.parameter_bindings.contains_key("CORE_MISSION_CAPABILITIES") {
            let caps: Vec<String> = (1..=4).map(|i| format!("  {i}. {}", self.parameter_bindings.get(&format!("CORE_CAPABILITY_{i}")).cloned().unwrap_or_default())).collect();
            self.parameter_bindings.insert("CORE_MISSION_CAPABILITIES".to_string(), caps.join("\n"));
        }
    }

    /// Returns default lifecycle bindings for non-reusable lifecycle archetypes (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
    fn lifecycle_preset(selected: LifecycleType) -> &'static [(&'static str, &'static str)] {
        match selected {
            LifecycleType::ExpendableDisposal => &[
                ("PRIMARY_TERMINAL_TARGET", "Designated Safe Terminal Containment Zone"), ("SECONDARY_TERMINAL_TARGET", "Secondary Safe Containment Area RS-DIVERT-ALPHA"),
                ("PRIMARY_RECOVERY_FACILITY", "Designated Containment Area (Single-Use Disposal)"), ("SECONDARY_RECOVERY_FACILITY", "Secondary Containment Area (Single-Use Disposal)"),
                ("LIFECYCLE_TRANSIT_MODE", "Autonomous_Terminal_Containment_Transit"), ("LIFECYCLE_FAILSAFE_SEQUENCE", "autonomous terminal containment and safe de-energization sequence"),
                ("LIFECYCLE_END_STATE", "All assigned operational corridor waypoints fully traversed and verified; zero unauthorized state boundary excursions; zero unmitigated collision or interference hazards; and successful arrival in the designated terminal containment zone with complete power de-energization and safe disposal."),
                ("LIFECYCLE_BINGO_SAFETY_ACTION", "Continuously compute closed-loop dynamic resource state and execute safe terminal containment and power de-energization upon reaching safety thresholds ($R(t) \\le R_{\\mathrm{threshold}}(t)$)."),
                ("LIFECYCLE_POST_OP_STATE", "Terminal safe containment and de-energization with complete power bus isolation, memory state purge, and structural containment verification"),
                ("CONTAINMENT_MITIGATION_TYPE", "SAFE_TERMINAL_CONTAINMENT"), ("RECOVERY_DEVICE_TERM", "containment"),
                ("FAILSAFE_CONTAINMENT_SYSTEM", "autonomous containment watchdog and kinetic energy absorption"), ("TIER4_CONTAINMENT_DESC", "autonomous terminal containment and safe power isolation"),
            ],
            LifecycleType::ContinuousStationary => &[
                ("PRIMARY_TERMINAL_TARGET", "Primary Sterile Field Docking Station"), ("SECONDARY_TERMINAL_TARGET", "Secondary Clinical Safe Staging Console"),
                ("PRIMARY_RECOVERY_FACILITY", "Clinical Operating Suite"), ("SECONDARY_RECOVERY_FACILITY", "Secondary Maintenance Staging Bay"),
                ("LIFECYCLE_TRANSIT_MODE", "Autonomous_Clinical_Safing"), ("LIFECYCLE_FAILSAFE_SEQUENCE", "electromechanical joint brake lock and sterile field safing sequence"),
                ("LIFECYCLE_END_STATE", "All assigned operational trajectory segments fully traversed and verified; zero unauthorized workspace boundary excursions; and successful safe parking at the primary sterile console with residual power reserves strictly satisfying $R_{\\mathrm{reserve}} \\ge \\text{Ratio}_{\\text{reserve\\_min}} \\cdot R_{\\mathrm{capacity}}$."),
                ("LIFECYCLE_BINGO_SAFETY_ACTION", "Continuously compute closed-loop dynamic resource state and execute automated electromechanical joint brake locking and sterile field preservation upon reaching safety thresholds ($R(t) \\le R_{\\mathrm{threshold}}(t)$)."),
                ("LIFECYCLE_POST_OP_STATE", "Post-operation stationary safe rest at sterile field console with engaged electromechanical joint brake locks, secure memory state purge, and diagnostic log offloading"),
                ("CONTAINMENT_MITIGATION_TYPE", "ELECTROMECHANICAL_BRAKE_LOCK"), ("RECOVERY_DEVICE_TERM", "failsafe joint brake"),
                ("FAILSAFE_CONTAINMENT_SYSTEM", "electromechanical joint brake lock and sterile field interlocks"), ("TIER4_CONTAINMENT_DESC", "instant electromechanical joint brake locking and actuator power bus disconnect"),
            ],
            LifecycleType::TrackBoundGuided => &[
                ("PRIMARY_TERMINAL_TARGET", "Primary Rail Maintenance Siding"), ("SECONDARY_TERMINAL_TARGET", "Secondary Controlled Deceleration Spur RS-DIVERT-ALPHA"),
                ("PRIMARY_RECOVERY_FACILITY", "Classification Yard Maintenance Depot"), ("SECONDARY_RECOVERY_FACILITY", "Secondary Service Siding"),
                ("LIFECYCLE_TRANSIT_MODE", "Autonomous_Track_Deceleration"), ("LIFECYCLE_FAILSAFE_SEQUENCE", "controlled track deceleration and siding brake sequence"),
                ("LIFECYCLE_END_STATE", "All assigned track blocks and route sections fully traversed and verified; zero unauthorized siding excursions; and successful controlled arrival at the primary maintenance siding or designated secondary spur with residual resources strictly satisfying $R_{\\mathrm{reserve}} \\ge \\text{Ratio}_{\\text{reserve\\_min}} \\cdot R_{\\mathrm{capacity}}$."),
                ("LIFECYCLE_BINGO_SAFETY_ACTION", "Continuously compute closed-loop dynamic resource state and execute controlled track deceleration or secondary maintenance siding divert upon reaching safety thresholds ($R(t) \\le R_{\\mathrm{threshold}}(t)$)."),
                ("LIFECYCLE_POST_OP_STATE", "Post-operation stationary rest at rail siding with mechanical parking brakes engaged, pneumatic systems vented, secure memory state purge, and diagnostic log offloading"),
                ("CONTAINMENT_MITIGATION_TYPE", "TRACK_SIDING_BRAKE"), ("RECOVERY_DEVICE_TERM", "pneumatic emergency brake"),
                ("FAILSAFE_CONTAINMENT_SYSTEM", "pneumatic emergency brake and track siding deceleration system"), ("TIER4_CONTAINMENT_DESC", "emergency brake pipe venting and mechanical friction caliper clamping"),
            ],
            _ => &[
                ("PRIMARY_TERMINAL_TARGET", "Designated De-Orbit Reentry Corridor / Graveyard Orbit"), ("SECONDARY_TERMINAL_TARGET", "Secondary Disposal Orbit Node RS-DIVERT-ALPHA"),
                ("PRIMARY_RECOVERY_FACILITY", "Designated Atmospheric Demise Footprint"), ("SECONDARY_RECOVERY_FACILITY", "Stable Graveyard Disposal Orbit"),
                ("LIFECYCLE_TRANSIT_MODE", "Autonomous_Disposal_Burn_Transit"), ("LIFECYCLE_FAILSAFE_SEQUENCE", "autonomous de-orbit disposal and passivation sequence"),
                ("LIFECYCLE_END_STATE", "All assigned orbital mission phases fully executed and verified; zero unauthorized constellation slot excursions; zero unmitigated orbital debris collision hazards; and successful de-orbit disposal or transfer to graveyard orbit with residual propellant satisfying disposal containment standards."),
                ("LIFECYCLE_BINGO_SAFETY_ACTION", "Continuously compute closed-loop dynamic resource state and execute autonomous de-orbit disposal burn or graveyard orbit maneuver upon reaching safety thresholds ($R(t) \\le R_{\\mathrm{threshold}}(t)$)."),
                ("LIFECYCLE_POST_OP_STATE", "Post-mission passivation and safe disposal state with reaction wheels desaturated, batteries passivated, secure memory state purge, and final ephemeris offload"),
                ("CONTAINMENT_MITIGATION_TYPE", "DEORBIT_DISPOSAL_BURN"), ("RECOVERY_DEVICE_TERM", "autonomous de-orbit propulsion"),
                ("FAILSAFE_CONTAINMENT_SYSTEM", "autonomous de-orbit propulsion and orbital passivation unit"), ("TIER4_CONTAINMENT_DESC", "autonomous retro-burn execution, reaction wheel desaturation, and battery passivation"),
            ],
        }
    }

    /// [DOC-TIER: A] Derives lifecycle archetype, terminal containment targets, and post-operational state bindings.
    /// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-DERIVE-LIFECYCLE-CONTRACT] [FLS: fls_conops_cmark]
    /// Contract: Selects `LifecycleType` from schema parameters/AST components and populates terminal/containment tokens.
    /// Bounds & Faults: O(P) over AST component names; infallible.
    pub fn _derive_lifecycle_contract(&mut self) {
        let dom = self.detected_domain.to_lowercase();
        let is_exp = self.parameter_bindings.get("IS_EXPENDABLE").is_some_and(|s| s == "true" || s == "1")
            || ["disposable", "single_use", "expendable"].contains(&dom.as_str())
            || self.has_ast_component("expendable") || self.has_ast_component("disposable");
        let selected = match self.parameter_bindings.get("LIFECYCLE_TYPE").map(String::as_str) {
            Some("EXPENDABLE_DISPOSAL") => LifecycleType::ExpendableDisposal,
            Some("CONTINUOUS_STATIONARY") => LifecycleType::ContinuousStationary,
            Some("TRACK_BOUND_GUIDED") => LifecycleType::TrackBoundGuided,
            Some("PERSISTENT_ORBITAL") => LifecycleType::PersistentOrbital,
            Some(_) => LifecycleType::ReusableRecovery,
            None if is_exp => LifecycleType::ExpendableDisposal,
            None if dom == "medical" || self.has_ast_component("joint_brake") => LifecycleType::ContinuousStationary,
            None if dom == "rail" || self.has_ast_component("rail") => LifecycleType::TrackBoundGuided,
            None if dom == "space" || self.has_ast_component("orbit") => LifecycleType::PersistentOrbital,
            None => LifecycleType::ReusableRecovery,
        };
        self.parameter_bindings.insert("LIFECYCLE_TYPE".to_string(), selected.as_str().to_string());
        if selected == LifecycleType::ReusableRecovery {
            self.derive_reusable_recovery_bindings();
        } else {
            for &(k, v) in Self::lifecycle_preset(selected) {
                self.parameter_bindings.entry(k.to_string()).or_insert_with(|| v.to_string());
            }
        }
    }

    /// [DOC-TIER: A] Resolves a template token against bound parameters or returns the fallback default.
    /// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-RESOLVE-TOKEN] [FLS: fls_conops_cmark]
    /// Contract: Looks up `token_name` (exact, uppercase, lowercase) in `parameter_bindings` or delegates to `get_fallback_default`.
    /// Bounds & Faults: O(1) map lookup; infallible.
    pub fn resolve_token(&self, token_name: &str, inline_default: Option<&str>) -> String {
        self.parameter_bindings.get(token_name)
            .or_else(|| self.parameter_bindings.get(&token_name.to_uppercase()))
            .or_else(|| self.parameter_bindings.get(&token_name.to_lowercase()))
            .cloned()
            .unwrap_or_else(|| self.get_fallback_default(token_name, inline_default))
    }

    /// Resolves numeric physical limit or suffix-based fallback defaults (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
    fn match_numeric_or_suffix_fallback(token_upper: &str) -> String {
        match token_upper {
            "WEIGHT_CRIT_1" => "0.40".to_string(), "WEIGHT_CRIT_2" => "0.35".to_string(), "WEIGHT_CRIT_3" => "0.25".to_string(),
            "SCORE_A_1" | "SCORE_B_2" | "SCORE_C_1" => "1.0".to_string(), "SCORE_A_2" | "SCORE_B_3" | "SCORE_C_3" => "0.8".to_string(),
            "SCORE_A_3" | "SCORE_B_1" | "SCORE_C_2" => "0.9".to_string(), "WEIGHTED_SCORE_A" => "0.90".to_string(),
            "WEIGHTED_SCORE_B" => "0.79".to_string(), "WEIGHTED_SCORE_C" => "0.80".to_string(),
            "TOTAL_MASS_KG" | "M_MAX_LIMIT_KG" | "TOTAL_MTOW_KG" | "MTOW_MAX_KG" => "50.0".to_string(),
            "M_MAX_NOMINAL_KG" | "MTOW_NOMINAL_KG" => "45.0".to_string(), "PAYLOAD_MAX_KG" | "V_MIN_NOMINAL_MPS" => "5.0".to_string(),
            "PAYLOAD_NOMINAL_KG" => "3.0".to_string(), "DIM_MAX_L_M" => "2.5".to_string(), "DIM_MAX_W_M" => "1.5".to_string(),
            "DIM_MAX_H_M" => "1.0".to_string(), "DIM_NOM_L_M" => "2.0".to_string(), "DIM_NOM_W_M" => "1.2".to_string(),
            "DIM_NOM_H_M" => "0.8".to_string(), "V_CRUISE_MIN_MPS" | "WIND_LIMIT_MAX_MPS" => "12.0".to_string(),
            "V_CRUISE_MAX_MPS" => "24.0".to_string(), "V_CRUISE_NOMINAL_MPS" => "18.0".to_string(), "V_MAX_MPS" => "25.0".to_string(),
            "V_MAX_NOMINAL_MPS" => "22.0".to_string(), "V_MIN_MAX_MPS" => "8.0".to_string(), "SPATIAL_LIMIT_MAX_M" => "5000.0".to_string(),
            "SPATIAL_LIMIT_NOMINAL_M" => "4000.0".to_string(), "SPATIAL_LIMIT_UNIT" => "m".to_string(),
            "C2_RANGE_MIN_KM" | "WIND_LIMIT_NOMINAL_MPS" => "10.0".to_string(), "C2_RANGE_NOMINAL_KM" => "15.0".to_string(),
            "ENDURANCE_MIN_MIN" => "60.0".to_string(), "ENDURANCE_NOMINAL_MIN" => "90.0".to_string(), "TEMP_MIN_DEGC" => "-20.0".to_string(),
            "TEMP_MAX_DEGC" => "+55.0".to_string(), "TEMP_NOMINAL_DEGC" => "+25.0".to_string(),
            "INGRESS_PROTECTION_RATING" => "IP54".to_string(), "INGRESS_PROTECTION_TARGET" => "IP67".to_string(),
            _ if token_upper.ends_with("_PERCENT") || token_upper.ends_with("_PCT") => "10.0".to_string(),
            _ if token_upper.ends_with("_KG") => "5.0".to_string(), _ if token_upper.ends_with("_MPS") => "15.0".to_string(),
            _ if token_upper.ends_with("_KM") => "10.0".to_string(), _ if token_upper.ends_with("_M") => "100.0".to_string(),
            _ if token_upper.ends_with("_SEC") || token_upper.ends_with("_S") => "1.0".to_string(),
            _ if token_upper.ends_with("_MIN") => "30.0".to_string(), _ if token_upper.ends_with("_COUNT") => "5".to_string(),
            _ => "N/A".to_string(),
        }
    }

    /// [DOC-TIER: A] Computes a deterministic fallback default value for an unbound template token.
    /// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-GET-FALLBACK-DEFAULT] [FLS: fls_conops_cmark]
    /// Contract: Prioritizes `inline_default`, then `DEFAULT_CONOPS_PARAMS`, then canonical metadata/numeric fallbacks.
    /// Bounds & Faults: O(K) over `DEFAULT_CONOPS_PARAMS`; infallible.
    pub fn get_fallback_default(&self, token_name: &str, inline_default: Option<&str>) -> String {
        if let Some(def) = inline_default { return def.to_string(); }
        let token_upper = token_name.to_uppercase();
        if let Some(val) = self.parameter_bindings.get(&token_upper) { return val.clone(); }
        if let Some(&(_, v)) = DEFAULT_CONOPS_PARAMS.iter().find(|&&(k, _)| k == token_upper) { return v.to_string(); }
        let sys_id = self.inferred_system_identifier.as_deref().unwrap_or("AutonomousCyberPhysicalSystem");
        match token_upper.as_str() {
            "SYSTEM_IDENTIFIER" | "MISSION_SYSTEM_NAME" | "SYSTEM_NAME" => sys_id.to_string(),
            "DOCUMENT_VERSION" => "1.0.0".to_string(), "DOCUMENT_DATE" => Utc::now().date_naive().to_string(),
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

    /// [DOC-TIER: A] Performs up to 3 passes of `{{TOKEN}}` substitution followed by domain ontology and Level 1B sanitization.
    /// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-SUBSTITUTE] [FLS: fls_conops_cmark]
    /// Contract: Expands nested `{{KEY}}` tokens (up to 3 passes), applies domain ontology replacements, and strips Level 2 wire details.
    /// Bounds & Faults: O(N) time and space over `content.len()`; infallible.
    pub fn substitute(&self, content: &str) -> String {
        if content.is_empty() { return String::new(); }
        let mut current = content.to_string();
        for _ in 0..3 {
            let (next, changed) = replace_placeholders(&current, |key, def| self.resolve_token(key, def));
            if !changed || next == current { break; }
            current = next;
        }
        self.apply_domain_ontology_sanitization(&mut current);
        let sys_target = self.parameter_bindings.get("SYSTEM_IDENTIFIER").or_else(|| self.parameter_bindings.get("SYSTEM_NAME")).or(self.inferred_system_identifier.as_ref()).map(String::as_str).unwrap_or("Autonomous Cyber-Physical System");
        for pat in ["the Abstract Cyber-Physical System Archetype", "an Abstract Cyber-Physical System Archetype"] {
            current = current.replace(pat, &format!("the {sys_target}"));
        }
        current = current.replace("The Abstract Cyber-Physical System Archetype", &format!("The {sys_target}"));
        for pat in ["Abstract Cyber-Physical System Archetype", "Autonomous Cyber-Physical System Archetype", "AutonomousSystemArchetype"] {
            current = current.replace(pat, sys_target);
        }
        sanitize_level_1b_operational_text(&current)
    }

    /// [DOC-TIER: A] Sanitizes domain-inappropriate recovery, altitude, standard, and kinetic/ROE phrases in `current`.
    /// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-APPLY-DOMAIN-ONTOLOGY-SANITIZATION] [FLS: fls_conops_cmark]
    /// Contract: Replaces ungrounded parachute/AGL/ASTM/ROE terminology with AST-derived domain containment and interlock terms.
    /// Bounds & Faults: O(N) over string length; preserves AST-grounded components.
    pub fn apply_domain_ontology_sanitization(&self, current: &mut String) {
        let dom = self.detected_domain.to_lowercase();
        let has_chute = self.has_ast_component("parachute") || self.has_ast_component("chute") || self.parameter_bindings.contains_key("S_CANOPY");
        let domain_override = match dom.as_str() {
            "medical" => Some(("failsafe joint brake", "JOINT_BRAKE", "mm", "IEC 62304 / ISO 14971")),
            "rail" => Some(("pneumatic emergency brake", "EMERGENCY_BRAKE", "m", "EN 50128 SIL 4")),
            "marine" => Some(("positive buoyancy drop-weight", "DROP_WEIGHT", "m Depth", "DNV-GL-ST-E403")),
            "space" => Some(("autonomous de-orbit propulsion", "DEORBIT_THRUSTER", "km", "ECSS-E-ST-40C")),
            _ => None,
        };
        if let Some((term, upper, alt_unit, std_str)) = domain_override {
            *current = replace_word_case_insensitive(current, "parachute", term).replace("PARACHUTE", upper);
            *current = replace_altitude_agl(current, alt_unit);
            *current = replace_astm_f3411(current, std_str);
        } else if self.parameter_bindings.get("LIFECYCLE_TYPE").is_some_and(|s| s == "EXPENDABLE_DISPOSAL")
            || self.parameter_bindings.get("IS_EXPENDABLE").is_some_and(|s| s == "true")
        {
            *current = replace_word_case_insensitive(current, "parachute", "controlled impact containment").replace("PARACHUTE", "CONTAINMENT");
        } else if !has_chute {
            let rec_term = self.parameter_bindings.get("RECOVERY_DEVICE_TERM").cloned().unwrap_or_else(|| "failsafe containment".to_string());
            let rec_upper = self.parameter_bindings.get("CONTAINMENT_MITIGATION_TYPE").cloned().unwrap_or_else(|| "FAILSAFE_CONTAINMENT".to_string());
            *current = replace_word_case_insensitive(current, "parachute", &rec_term).replace("PARACHUTE", &rec_upper);
        } else if self.is_non_aircraft {
            *current = replace_word_case_insensitive(current, "parachute", "recovery system").replace("PARACHUTE", "RECOVERY");
        }
        let pfx = self.parameter_bindings.get("INTERLOCK_PREFIX").map(String::as_str).unwrap_or("INT");
        for idx in 1..=6 {
            let repl = format!("{pfx}-0{idx}");
            *current = replace_word_case_insensitive(current, &format!("ROE-0{idx}"), &repl);
            if pfx == "INT" { *current = replace_word_case_insensitive(current, &format!("SAF-0{idx}"), &repl); }
        }
        *current = replace_roe_phrase(current);
        *current = replace_word_case_insensitive(current, "weapons release", "high-consequence actuation");
        *current = replace_word_case_insensitive(current, "collateral damage", "adjacent operational risk");
    }
}
