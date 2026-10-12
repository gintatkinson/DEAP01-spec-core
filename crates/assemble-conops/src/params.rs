//! SysML AST Parameter Binding Engine and template token resolution.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use regex::Regex;
use serde_json::Value;

use crate::sanitize::sanitize_level_1b_operational_text;

/// Canonical default ConOps parameters dictionary.
pub static DEFAULT_CONOPS_PARAMS: &[(&str, &str)] = &[
    ("MAX_JUNCTION_TEMPERATURE_DELTA_C", "25.0"),
    ("BATTERY_CHARGE_C_RATE", "2.0C"),
    ("BATTERY_CHARGE_TIME_HOURS", "1.5"),
    ("SUPPORT_EQUIPMENT_BATTERY_HOURS", "8.0"),
    ("OPERATIONAL_AVAILABILITY_THRESHOLD", "0.95"),
    ("OPERATIONAL_AVAILABILITY_OBJECTIVE", "0.99"),
    ("OPERATING_TEMPERATURE_MIN_C", "-20.0"),
    ("OPERATING_TEMPERATURE_MAX_C", "+55.0"),
];

pub static PLACEHOLDER_PATTERN: &str = r"\{\{([A-Za-z0-9_]+)(?::([^\}]*))?\}\}";
pub static RAW_TOKEN_FINDER: &str = r"\{\{[A-Za-z0-9_]+(?::[^\}]*)?\}\}";


/// Formal lifecycle archetypes for cyber-physical mission systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleType {
    ReusableRecovery,
    ExpendableKineticEffector,
    ContinuousStationary,
    PersistentOrbital,
    TrackBoundGuided,
}

impl LifecycleType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ReusableRecovery => "REUSABLE_RECOVERY",
            Self::ExpendableKineticEffector => "EXPENDABLE_KINETIC_EFFECTOR",
            Self::ContinuousStationary => "CONTINUOUS_STATIONARY",
            Self::PersistentOrbital => "PERSISTENT_ORBITAL",
            Self::TrackBoundGuided => "TRACK_BOUND_GUIDED",
        }
    }
}

/// Automated SysML AST Parameter Binding Engine.
/// Ingests domain parameter dictionaries and resolves canonical template placeholders `{{...}}`.
#[derive(Debug, Clone)]
pub struct SysMLParameterBindingEngine {
    pub workspace_dir: PathBuf,
    pub parameter_bindings: HashMap<String, String>,
    pub explicit_keys: HashSet<String>,
    pub inferred_system_identifier: Option<String>,
    pub detected_domain: String,
    pub is_non_aircraft: bool,
    pub is_civilian: bool,
    pub ast_part_names: BTreeSet<String>,
}

impl SysMLParameterBindingEngine {
    pub fn new(
        parameter_values: Option<HashMap<String, String>>,
        config_path: Option<&Path>,
        workspace_dir: Option<&Path>,
        auto_detect: bool,
        domain: Option<&str>,
    ) -> Self {
        let ws = workspace_dir
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        let mut engine = Self {
            workspace_dir: ws.clone(),
            parameter_bindings: HashMap::new(),
            explicit_keys: HashSet::new(),
            inferred_system_identifier: None,
            detected_domain: domain.unwrap_or("generic").to_string(),
            is_non_aircraft: false,
            is_civilian: false,
            ast_part_names: BTreeSet::new(),
        };

        // Populate default parameters
        for &(k, v) in DEFAULT_CONOPS_PARAMS {
            engine.parameter_bindings.insert(k.to_string(), v.to_string());
        }

        if let Some(dom) = domain {
            engine.parameter_bindings.insert("DOMAIN_TYPE".to_string(), dom.to_string());
            engine.parameter_bindings.insert("TARGET_DOMAIN".to_string(), dom.to_string());
            engine.explicit_keys.insert("DOMAIN_TYPE".to_string());
            engine.explicit_keys.insert("TARGET_DOMAIN".to_string());
        }

        if auto_detect {
            engine.auto_discover_sources(&ws);
        }

        if let Some(cfg) = config_path {
            engine.ingest_file(cfg);
        }

        if let Some(params) = parameter_values {
            for (k, v) in params {
                engine.explicit_keys.insert(k.to_uppercase());
                engine.parameter_bindings.insert(k.clone(), v.clone());
                engine.parameter_bindings.insert(k.to_uppercase(), v.clone());
                engine._map_semantic_aliases(&k, &v);
            }
        }

        engine.detected_domain = engine._detect_domain_type();
        let dom_clone = engine.detected_domain.clone();
        engine.parameter_bindings.insert("DETECTED_DOMAIN".to_string(), dom_clone.clone());
        engine.parameter_bindings.insert("DOMAIN_TYPE".to_string(), dom_clone);

        engine._derive_operational_intent();
        engine._derive_mass_budgets();
        engine._derive_quadratic_physics();
        engine._derive_energy_budgets();
        engine._derive_domain_regulatory_standards();
        engine._derive_domain_ontology();
        engine._derive_lifecycle_contract();
        engine._derive_subsystem_architecture();

        engine
    }

    pub fn auto_discover_sources(&mut self, root_dir: &Path) {
        let mut search_dirs = Vec::new();
        let mut curr = root_dir.to_path_buf();
        for _ in 0..5 {
            search_dirs.push(curr.clone());
            if let Some(parent) = curr.parent() {
                if parent == curr {
                    break;
                }
                curr = parent.to_path_buf();
            } else {
                break;
            }
        }
        self.auto_detect_workspace_parameters(&search_dirs);
    }

    pub fn auto_detect_workspace_parameters(&mut self, search_dirs: &[PathBuf]) {
        let supported_exts = [".sysml", ".yaml", ".yml", ".json", ".md", ".markdown"];

        for sdir in search_dirs {
            // Check upstream guard
            if sdir.join(".pipeline").join("upstream").is_dir() {
                continue;
            }

            // Check .pipeline files
            let schema_sysml = sdir.join(".pipeline").join("schema.sysml");
            if schema_sysml.is_file() {
                self.ingest_file(&schema_sysml);
            }
            let schema_digest = sdir.join(".pipeline").join("schema-digest.json");
            if schema_digest.is_file() {
                self.ingest_file(&schema_digest);
            }

            // Check candidate dirs: schema, docs/architecture, docs/research
            for rel_dir in &["schema", "docs/architecture", "docs/research"] {
                let cdir = sdir.join(rel_dir);
                if cdir.is_dir() {
                    for entry in walkdir::WalkDir::new(&cdir).into_iter().filter_map(|e| e.ok()) {
                        let path = entry.path();
                        if path.is_file() {
                            let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                            if fname.starts_with('.') {
                                continue;
                            }
                            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                                let dot_ext = format!(".{}", ext.to_lowercase());
                                if supported_exts.contains(&dot_ext.as_str()) {
                                    self.ingest_file(path);
                                }
                            }
                        }
                    }
                }
            }
        }

        self._derive_domain_ontology();
        self._derive_operational_intent();
        self._derive_lifecycle_contract();
        self._derive_subsystem_architecture();
    }

    pub fn ingest_file(&mut self, path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "json" => self.ingest_json_file(path),
            "sysml" => self.ingest_sysml_file(path),
            "md" | "markdown" => self.ingest_markdown_file(path),
            _ => false,
        }
    }

    pub fn ingest_json_file(&mut self, path: &Path) -> bool {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(val) = serde_json::from_str::<Value>(&content) {
                self.ingest_dictionary(&val);
                return true;
            }
        }
        false
    }

    pub fn ingest_dictionary(&mut self, data: &Value) {
        if let Value::Object(map) = data {
            for container_key in &[
                "parameters",
                "domain_parameters",
                "domain_params",
                "specs",
                "attributes",
                "metadata",
            ] {
                if let Some(sub) = map.get(*container_key) {
                    self.ingest_dictionary(sub);
                }
            }

            for name_key in &[
                "system_identifier",
                "system_name",
                "system",
                "name",
                "SYSTEM_IDENTIFIER",
                "MISSION_SYSTEM_NAME",
            ] {
                if let Some(Value::String(s)) = map.get(*name_key) {
                    let trimmed = s.trim();
                    if !trimmed.is_empty() {
                        self.inferred_system_identifier = Some(trimmed.to_string());
                    }
                }
            }

            for (k, v) in map {
                if [
                    "parameters",
                    "domain_parameters",
                    "domain_params",
                    "specs",
                    "attributes",
                    "metadata",
                    "schema_nodes",
                ]
                .contains(&k.as_str())
                {
                    continue;
                }
                self.explicit_keys.insert(k.to_uppercase());

                match v {
                    Value::String(s) => {
                        self.parameter_bindings.insert(k.clone(), s.clone());
                        self.parameter_bindings.insert(k.to_uppercase(), s.clone());
                        self._map_semantic_aliases(k, s);
                    }
                    Value::Number(n) => {
                        let str_val = n.to_string();
                        self.parameter_bindings.insert(k.clone(), str_val.clone());
                        self.parameter_bindings.insert(k.to_uppercase(), str_val.clone());
                        self._map_semantic_aliases(k, &str_val);
                    }
                    Value::Bool(b) => {
                        let str_val = b.to_string();
                        self.parameter_bindings.insert(k.clone(), str_val.clone());
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
                                if item_str.starts_with(&format!("{}.", idx + 1))
                                    || item_str.starts_with('-')
                                {
                                    lines.push(format!("  {}", item_str));
                                } else {
                                    lines.push(format!("  {}. {}", idx + 1, item_str));
                                }
                            }
                            let formatted = lines.join("\n");
                            self.parameter_bindings.insert(k.clone(), formatted.clone());
                            self.parameter_bindings.insert(k.to_uppercase(), formatted.clone());
                            self.parameter_bindings
                                .insert("CORE_MISSION_CAPABILITIES".to_string(), formatted);
                        }
                    }
                    _ => {}
                }
            }

            if let Some(Value::Array(nodes)) = map.get("schema_nodes") {
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
        }
    }

    pub fn ingest_sysml_file(&mut self, path: &Path) -> bool {
        if let Ok(content) = fs::read_to_string(path) {
            return self.ingest_sysml_text(&content);
        }
        false
    }

    pub fn ingest_sysml_text(&mut self, text: &str) -> bool {
        let pkg_re = Regex::new(r"\bpackage\s+([A-Za-z0-9_]+)").unwrap();
        if let Some(caps) = pkg_re.captures(text) {
            let pkg_name = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
            if !["Package", "Model", "Root"].contains(&pkg_name) {
                self.inferred_system_identifier = Some(pkg_name.to_string());
                self.parameter_bindings
                    .insert("SYSTEM_IDENTIFIER".to_string(), pkg_name.to_string());
                self.parameter_bindings
                    .insert("MISSION_SYSTEM_NAME".to_string(), pkg_name.to_string());
                self.explicit_keys.insert("SYSTEM_IDENTIFIER".to_string());
                self.explicit_keys.insert("MISSION_SYSTEM_NAME".to_string());
            }
        }

        let part_re = Regex::new(r"\bpart\s+def\s+([A-Za-z0-9_]+)").unwrap();
        for caps in part_re.captures_iter(text) {
            if let Some(m) = caps.get(1) {
                let p_name = m.as_str().trim().to_string();
                self.ast_part_names.insert(p_name.clone());
                self.explicit_keys.insert(p_name.clone());
                self.explicit_keys.insert(p_name.to_uppercase());
            }
        }

        let attr_re = Regex::new(
            r"\battribute\s+([A-Za-z0-9_]+)(?:\s*:\s*[A-Za-z0-9_]+)?\s*=\s*([^;]+);",
        )
        .unwrap();
        for caps in attr_re.captures_iter(text) {
            let attr_name = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
            let raw_val = caps
                .get(2)
                .map(|m| m.as_str().trim().trim_matches(['"', '\'']))
                .unwrap_or("");
            self.explicit_keys.insert(attr_name.to_string());
            self.explicit_keys.insert(attr_name.to_uppercase());
            self.parameter_bindings
                .insert(attr_name.to_string(), raw_val.to_string());
            self.parameter_bindings
                .insert(attr_name.to_uppercase(), raw_val.to_string());
            self._map_semantic_aliases(attr_name, raw_val);
        }

        true
    }

    pub fn ingest_markdown_file(&mut self, path: &Path) -> bool {
        if let Ok(content) = fs::read_to_string(path) {
            let title_re = Regex::new(r"^#\s+(.+)$").unwrap();
            for line in content.lines() {
                if let Some(caps) = title_re.captures(line.trim()) {
                    let title = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
                    let title_clean = title.replace(['*', '_', '`'], "").trim().to_string();
                    let generic = [
                        "technical specifications",
                        "specifications",
                        "table of contents",
                        "overview",
                        "system specifications",
                        "concept of operations",
                        "conops",
                        "mission intent",
                        "requirements",
                        "architecture",
                        "metadata",
                    ];
                    if !generic.contains(&title_clean.to_lowercase().as_str())
                        && !title_clean.is_empty()
                        && self.inferred_system_identifier.is_none()
                    {
                        self.inferred_system_identifier = Some(title_clean.clone());
                        self.parameter_bindings
                            .insert("SYSTEM_IDENTIFIER".to_string(), title_clean.clone());
                        self.parameter_bindings
                            .insert("MISSION_SYSTEM_NAME".to_string(), title_clean);
                        self.explicit_keys.insert("SYSTEM_IDENTIFIER".to_string());
                        self.explicit_keys.insert("MISSION_SYSTEM_NAME".to_string());
                    }
                    break;
                }
            }

            // Extract table rows key: value
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('|') && trimmed.ends_with('|') {
                    let cells: Vec<&str> = trimmed
                        .split('|')
                        .map(|c| c.trim())
                        .filter(|c| !c.is_empty())
                        .collect();
                    if cells.len() >= 2 {
                        let k = cells[0].replace(['*', '_', '`'], "").trim().to_string();
                        let v = cells[1].trim().to_string();
                        if !k.is_empty() && !v.is_empty() && !k.starts_with("---") {
                            self.parameter_bindings.insert(k.clone(), v.clone());
                            self.parameter_bindings.insert(k.to_uppercase(), v.clone());
                            self._map_semantic_aliases(&k, &v);
                        }
                    }
                }
            }
            return true;
        }
        false
    }

    pub fn _map_semantic_aliases(&mut self, key: &str, val: &str) {
        let lower = key.to_lowercase().replace(['-', ' '], "_");
        let num_re = Regex::new(r"[-+]?\d*\.?\d+").unwrap();
        let num_val = num_re
            .find(val)
            .map(|m| m.as_str())
            .unwrap_or(val)
            .to_string();

        let mut aliases = HashMap::new();

        if lower.contains("system_identifier")
            || lower.contains("mission_system_name")
            || ((lower == "system" || lower == "system_name")
                && !self.parameter_bindings.contains_key("SYSTEM_IDENTIFIER"))
        {
            aliases.insert("SYSTEM_IDENTIFIER", val.to_string());
            aliases.insert("MISSION_SYSTEM_NAME", val.to_string());
            aliases.insert("SYSTEM_NAME", val.to_string());
        } else if lower.contains("cruise") {
            aliases.insert("V_CRUISE_NOMINAL_MPS", num_val.clone());
            aliases.insert("V_CRUISE_MAX_MPS", num_val.clone());
            aliases.insert("MAX_CRUISE_SPEED_MS", num_val.clone());
            aliases.insert("CRUISE_SPEED_MPS", num_val.clone());
        } else if lower.contains("max_speed")
            || lower.contains("v_max")
            || lower.contains("max_horizontal")
            || lower == "vmax"
        {
            aliases.insert("V_MAX_MPS", num_val.clone());
            aliases.insert("MAX_SPEED_MS", num_val.clone());
            aliases.insert("MAX_HORIZONTAL_SPEED_MPS", num_val.clone());
        } else if lower.contains("stall") {
            aliases.insert("V_STALL_MAX_MPS", num_val.clone());
            aliases.insert("V_STALL_NOMINAL_MPS", num_val.clone());
            aliases.insert("STALL_SPEED_MPS", num_val.clone());
        } else if lower.contains("mtow")
            || lower.contains("takeoff_weight")
            || lower.contains("takeoff_mass")
        {
            aliases.insert("TOTAL_MTOW_KG", num_val.clone());
            aliases.insert("MTOW_MAX_KG", num_val.clone());
            aliases.insert("MTOW_NOMINAL_KG", num_val.clone());
        } else if lower.contains("payload") {
            aliases.insert("PAYLOAD_MAX_KG", num_val.clone());
            aliases.insert("PAYLOAD_NOMINAL_KG", num_val.clone());
        } else if lower.contains("endurance") {
            aliases.insert("ENDURANCE_NOMINAL_MIN", num_val.clone());
            aliases.insert("ENDURANCE_MIN_MIN", num_val.clone());
        } else if lower.contains("c2_range") || lower.contains("datalink_range") {
            aliases.insert("C2_RANGE_NOMINAL_KM", num_val.clone());
            aliases.insert("C2_RANGE_MIN_KM", num_val.clone());
        }

        for (k, v) in aliases {
            self.parameter_bindings.insert(k.to_string(), v.clone());
            self.explicit_keys.insert(k.to_string());
        }
    }

    pub fn _detect_domain_type(&self) -> String {
        for key in &["DOMAIN_TYPE", "TARGET_DOMAIN", "OPERATIONAL_DOMAIN", "DOMAIN"] {
            if let Some(val) = self.parameter_bindings.get(*key) {
                let clean = val.trim().to_lowercase();
                if !clean.is_empty() && !["unknown", "none", "null", "undefined"].contains(&clean.as_str()) {
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
                        for k in &["domain", "domain_type", "target_domain", "operational_domain"] {
                            if let Some(Value::String(s)) = map.get(*k) {
                                let clean = s.trim().to_lowercase();
                                if !clean.is_empty() && !["unknown", "none", "null"].contains(&clean.as_str()) {
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

    pub fn _get_mtow_value(&self) -> f64 {
        for key in &[
            "TOTAL_MTOW_KG",
            "MTOW_NOMINAL_KG",
            "MTOW_MAX_KG",
            "SYSTEM_MASS_MAX_KG",
            "SYSTEM_MASS_KG",
        ] {
            if let Some(val) = self.parameter_bindings.get(*key) {
                let num_re = Regex::new(r"[-+]?\d*\.?\d+").unwrap();
                if let Some(m) = num_re.find(val) {
                    if let Ok(v) = m.as_str().parse::<f64>() {
                        return v;
                    }
                }
            }
        }
        50.0
    }

    pub fn _derive_mass_budgets(&mut self) {
        let mtow = self._get_mtow_value();
        if !self.explicit_keys.contains("TOTAL_MTOW_KG")
            && !self.parameter_bindings.contains_key("TOTAL_MTOW_KG")
        {
            self.parameter_bindings
                .insert("TOTAL_MTOW_KG".to_string(), format!("{:.1}", mtow));
        }

        let has_fractions = (self
            .parameter_bindings
            .contains_key("MASS_FRACTION_AIRFRAME_PCT")
            || self
                .parameter_bindings
                .contains_key("MASS_FRACTION_STRUCTURE_PCT"))
            && self
                .parameter_bindings
                .contains_key("MASS_FRACTION_AVIONICS_PCT")
            && self
                .parameter_bindings
                .contains_key("MASS_FRACTION_PROPULSION_PCT")
            && self
                .parameter_bindings
                .contains_key("MASS_FRACTION_ENERGY_PCT")
            && self
                .parameter_bindings
                .contains_key("MASS_FRACTION_PAYLOAD_PCT");

        if has_fractions {
            let airframe_pct = self
                .parameter_bindings
                .get("MASS_FRACTION_STRUCTURE_PCT")
                .or_else(|| self.parameter_bindings.get("MASS_FRACTION_AIRFRAME_PCT"))
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(30.0)
                / 100.0;
            let avionics_pct = self
                .parameter_bindings
                .get("MASS_FRACTION_AVIONICS_PCT")
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(15.0)
                / 100.0;
            let propulsion_pct = self
                .parameter_bindings
                .get("MASS_FRACTION_PROPULSION_PCT")
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(25.0)
                / 100.0;
            let energy_pct = self
                .parameter_bindings
                .get("MASS_FRACTION_ENERGY_PCT")
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(20.0)
                / 100.0;
            let payload_pct = self
                .parameter_bindings
                .get("MASS_FRACTION_PAYLOAD_PCT")
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(7.0)
                / 100.0;

            let airframe = (airframe_pct * mtow * 100.0).round() / 100.0;
            let avionics = (avionics_pct * mtow * 100.0).round() / 100.0;
            let propulsion = (propulsion_pct * mtow * 100.0).round() / 100.0;
            let energy = (energy_pct * mtow * 100.0).round() / 100.0;
            let payload = (payload_pct * mtow * 100.0).round() / 100.0;
            let containment = ((mtow - (airframe + avionics + propulsion + energy + payload))
                * 100.0)
                .round()
                / 100.0;

            self.parameter_bindings
                .entry("MASS_BUDGET_AIRFRAME_KG".to_string())
                .or_insert_with(|| airframe.to_string());
            self.parameter_bindings
                .entry("MASS_BUDGET_STRUCTURE_KG".to_string())
                .or_insert_with(|| airframe.to_string());
            self.parameter_bindings
                .entry("MASS_BUDGET_AVIONICS_KG".to_string())
                .or_insert_with(|| avionics.to_string());
            self.parameter_bindings
                .entry("MASS_BUDGET_PROPULSION_KG".to_string())
                .or_insert_with(|| propulsion.to_string());
            self.parameter_bindings
                .entry("MASS_BUDGET_ENERGY_KG".to_string())
                .or_insert_with(|| energy.to_string());
            self.parameter_bindings
                .entry("MASS_BUDGET_PAYLOAD_KG".to_string())
                .or_insert_with(|| payload.to_string());
            self.parameter_bindings
                .entry("MASS_BUDGET_CONTAINMENT_KG".to_string())
                .or_insert_with(|| containment.to_string());
        }
    }

    pub fn _derive_quadratic_physics(&mut self) {
        let m = self._get_mtow_value();
        let g = 9.80665;
        let rho = if self.detected_domain == "marine" {
            1025.0
        } else if self.detected_domain == "space" {
            1.0e-12
        } else {
            1.225
        };

        let s_ref = 0.081;
        let cd_unmit = 1.0;
        let cd_mit = 1.75;
        let s_mit = 84.18;

        if rho > 0.0 && s_ref > 0.0 && cd_unmit > 0.0 {
            let v_term_unmit = ((2.0 * m * g) / (rho * s_ref * cd_unmit)).sqrt();
            let v_term_unmit_round = (v_term_unmit * 100.0).round() / 100.0;
            let ek_unmit = (0.5 * m * v_term_unmit_round.powi(2) * 10.0).round() / 10.0;

            self.parameter_bindings
                .entry("V_TERMINAL_UNMITIGATED_MPS".to_string())
                .or_insert_with(|| v_term_unmit_round.to_string());
            self.parameter_bindings
                .entry("E_K_UNMITIGATED_JOULES".to_string())
                .or_insert_with(|| ek_unmit.to_string());
        }

        let denom = rho * s_mit * cd_mit;
        if denom > 0.0 && m > 0.0 {
            let v_calc = ((2.0 * m * g) / denom).sqrt();
            let v_calc_round = (v_calc * 100.0).round() / 100.0;
            let ek_calc = (0.5 * m * v_calc_round.powi(2) * 10.0).round() / 10.0;

            self.parameter_bindings
                .entry("V_TERMINAL_MITIGATED_MPS".to_string())
                .or_insert_with(|| v_calc_round.to_string());
            self.parameter_bindings
                .entry("E_K_MITIGATED_JOULES".to_string())
                .or_insert_with(|| ek_calc.to_string());
            self.parameter_bindings
                .entry("MITIGATED_KINETIC_ENERGY_J".to_string())
                .or_insert_with(|| ek_calc.to_string());
        }
    }

    pub fn _derive_domain_regulatory_standards(&mut self) {
        let existing = self
            .parameter_bindings
            .get("DOMAIN_REGULATORY_STANDARDS_TABLE_ROWS")
            .or_else(|| self.parameter_bindings.get("REGULATORY_STANDARDS_TABLE_ROWS"));

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

        self.parameter_bindings
            .insert("DOMAIN_REGULATORY_STANDARDS_TABLE_ROWS".to_string(), table_rows.clone());
        self.parameter_bindings
            .insert("REGULATORY_STANDARDS_TABLE_ROWS".to_string(), table_rows);
    }

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
            .entry("E_BINGO_JOULES".to_string())
            .or_insert_with(|| e_bingo.to_string());
        self.parameter_bindings
            .entry("E_BINGO_THRESHOLD_JOULES".to_string())
            .or_insert_with(|| e_bingo.to_string());
    }

    pub fn _derive_domain_ontology(&mut self) {
        let dom = self.detected_domain.to_lowercase();
        self.is_non_aircraft = ["medical", "rail", "marine", "space", "industrial"].contains(&dom.as_str());
        self.is_civilian = ["medical", "rail", "marine", "space", "industrial"].contains(&dom.as_str());

        if self.is_civilian {
            self.parameter_bindings.insert("INTERLOCK_PREFIX".to_string(), "SAF".to_string());
            self.parameter_bindings.insert("INTERLOCK_SECTION_TITLE".to_string(), "Operational Safety Interlocks & High-Consequence Controls".to_string());
            self.parameter_bindings.insert("TARGET_VERIFICATION_LABEL".to_string(), "Positive Condition Verification (PCV)".to_string());
            self.parameter_bindings.insert("TARGET_VERIFICATION_ACRONYM".to_string(), "PCV".to_string());
            self.parameter_bindings.insert("TARGET_VERIFICATION_PHRASE".to_string(), "positive condition verification".to_string());
            self.parameter_bindings.insert("HIGH_CONSEQUENCE_ACTION".to_string(), "high-consequence actuation".to_string());
            self.parameter_bindings.insert("COLLATERAL_RISK_PHRASE".to_string(), "adjacent operational risk".to_string());
        } else {
            self.parameter_bindings.insert("INTERLOCK_PREFIX".to_string(), "ROE".to_string());
            self.parameter_bindings.insert("INTERLOCK_SECTION_TITLE".to_string(), "Rules of Engagement (ROE) & Operational Safety Interlocks".to_string());
            self.parameter_bindings.insert("TARGET_VERIFICATION_LABEL".to_string(), "Positive Identification (PID)".to_string());
            self.parameter_bindings.insert("TARGET_VERIFICATION_ACRONYM".to_string(), "PID".to_string());
            self.parameter_bindings.insert("TARGET_VERIFICATION_PHRASE".to_string(), "positive identification".to_string());
            self.parameter_bindings.insert("HIGH_CONSEQUENCE_ACTION".to_string(), "weapons release".to_string());
            self.parameter_bindings.insert("COLLATERAL_RISK_PHRASE".to_string(), "collateral damage".to_string());
        }
    }

    pub fn _derive_operational_intent(&mut self) {
        let sys_name = self
            .parameter_bindings
            .get("SYSTEM_IDENTIFIER")
            .or_else(|| self.parameter_bindings.get("SYSTEM_NAME"))
            .or_else(|| self.inferred_system_identifier.as_ref())
            .cloned()
            .unwrap_or_else(|| "Autonomous Cyber-Physical System".to_string());

        let dom = self.detected_domain.clone();

        self.parameter_bindings.entry("OPERATIONAL_PURPOSE".to_string()).or_insert_with(|| {
            format!("The primary operational purpose of {sys_name} is to execute deterministic autonomous operations in {dom} environments.")
        });
        self.parameter_bindings.entry("PRIMARY_OPERATIONAL_MISSION".to_string()).or_insert_with(|| {
            format!("The {sys_name} is engineered to execute high-assurance {dom} missions.")
        });

        self.parameter_bindings.entry("CORE_CAPABILITY_1".to_string()).or_insert_with(|| {
            format!("Autonomous trajectory tracking and corridor execution in {dom}.")
        });
        self.parameter_bindings.entry("CORE_CAPABILITY_2".to_string()).or_insert_with(|| {
            "Multi-modal sensor data fusion combining redundant state estimation sensors.".to_string()
        });
        self.parameter_bindings.entry("CORE_CAPABILITY_3".to_string()).or_insert_with(|| {
            "Real-time high-throughput telemetry streaming and deterministic command processing.".to_string()
        });
        self.parameter_bindings.entry("CORE_CAPABILITY_4".to_string()).or_insert_with(|| {
            "Deterministic failsafe state machine ensuring autonomous containment.".to_string()
        });

        if !self.parameter_bindings.contains_key("CORE_MISSION_CAPABILITIES") {
            let c1 = self.parameter_bindings.get("CORE_CAPABILITY_1").unwrap();
            let c2 = self.parameter_bindings.get("CORE_CAPABILITY_2").unwrap();
            let c3 = self.parameter_bindings.get("CORE_CAPABILITY_3").unwrap();
            let c4 = self.parameter_bindings.get("CORE_CAPABILITY_4").unwrap();
            self.parameter_bindings.insert(
                "CORE_MISSION_CAPABILITIES".to_string(),
                format!("  1. {c1}\n  2. {c2}\n  3. {c3}\n  4. {c4}"),
            );
        }
    }

    pub fn _derive_lifecycle_contract(&mut self) {
        let dom = self.detected_domain.to_lowercase();
        let selected_type = if dom == "medical" {
            LifecycleType::ContinuousStationary
        } else if dom == "rail" {
            LifecycleType::TrackBoundGuided
        } else if dom == "space" {
            LifecycleType::PersistentOrbital
        } else if dom == "interceptor" || self.parameter_bindings.get("IS_EXPENDABLE").map(|s| s == "true").unwrap_or(false) {
            LifecycleType::ExpendableKineticEffector
        } else {
            LifecycleType::ReusableRecovery
        };

        self.parameter_bindings
            .insert("LIFECYCLE_TYPE".to_string(), selected_type.as_str().to_string());
        self.parameter_bindings.entry("PRIMARY_TERMINAL_TARGET".to_string()).or_insert_with(|| "Primary Recovery Base".to_string());
        self.parameter_bindings.entry("SECONDARY_TERMINAL_TARGET".to_string()).or_insert_with(|| "Secondary Divert Site LZ-DIVERT-ALPHA".to_string());
        self.parameter_bindings.entry("PRIMARY_RECOVERY_FACILITY".to_string()).or_insert_with(|| "Primary Recovery Base".to_string());
        self.parameter_bindings.entry("SECONDARY_RECOVERY_FACILITY".to_string()).or_insert_with(|| "Secondary Divert Base".to_string());
        self.parameter_bindings.entry("LIFECYCLE_TRANSIT_MODE".to_string()).or_insert_with(|| "Autonomous_RTB_Transit".to_string());
    }

    pub fn _derive_subsystem_architecture(&mut self) {
        let sys_id = self
            .parameter_bindings
            .get("SYSTEM_IDENTIFIER")
            .or_else(|| self.inferred_system_identifier.as_ref())
            .cloned()
            .unwrap_or_else(|| "AutonomousSystem".to_string());

        let parts: Vec<String> = self.ast_part_names.iter().cloned().collect();

        // 1. Super-System Architecture
        let mut super_lines = vec![
            format!("The **{sys_id}** architecture formalizes the complete system boundary and segment allocations in accordance with IEEE 1362 §5.3, DoDAF OV-2 / SV-1, and ISO/IEC/IEEE 29148:2018."),
            String::new(),
            "```mermaid".to_string(),
            "flowchart TD".to_string(),
            format!("    subgraph Super_System[\"Operational Super-System Architecture ({sys_id})\"]"),
            "        direction TB".to_string(),
            "        subgraph Platform_Segment[\"Primary Operational Segment (DoDAF SV-1)\"]".to_string(),
            "            direction TB".to_string(),
        ];

        if parts.is_empty() {
            super_lines.push(format!("            Platform[\"{sys_id} Core System\"]"));
        } else {
            for (idx, p) in parts.iter().enumerate() {
                let clean_p = sanitize_level_1b_operational_text(p);
                let node_id = format!("P{}", idx + 1);
                super_lines.push(format!("            {node_id}[\"{clean_p}\"]"));
            }
        }
        super_lines.push("        end".to_string());
        super_lines.push("    end".to_string());
        super_lines.push("```".to_string());

        let super_arch = super_lines.join("\n");
        self.parameter_bindings
            .insert("SUPER_SYSTEM_ARCHITECTURE".to_string(), super_arch);

        // 2. Subsystem Architecture
        let mut sub_lines = vec![
            format!("All declared SysML AST structural parts for **{sys_id}** are allocated formal operational specifications:"),
            String::new(),
        ];

        if parts.is_empty() {
            sub_lines.push(format!("### 4.8.1 {sys_id} Core Subsystem Architecture"));
            sub_lines.push(format!("Houses core operational capabilities for {sys_id}."));
        } else {
            for (idx, p) in parts.iter().enumerate() {
                let clean_p = sanitize_level_1b_operational_text(p);
                sub_lines.push(format!("### 4.8.{} {} Subsystem Architecture", idx + 1, clean_p));
                sub_lines.push(format!("#### 4.8.{}.1 Functional Scope & Operational Role", idx + 1));
                sub_lines.push(format!("Provides dedicated operational capability for {clean_p}."));
                sub_lines.push(format!("#### 4.8.{}.2 Interface & Port Allocation", idx + 1));
                sub_lines.push(format!("- PORT-01 (INOUT): PACE C2 telemetry interface for {clean_p}."));
                sub_lines.push(format!("#### 4.8.{}.3 Mass, Power, & Thermal Resource Budget Allocation", idx + 1));
                sub_lines.push(format!("- Nominal mass: 5.0 kg, Nominal power: 50.0 W."));
                sub_lines.push(format!("#### 4.8.{}.4 Operational State Space, Lifecycle Modes, & Safety Invariants", idx + 1));
                sub_lines.push(format!("- Bound to system safety net with independent hardware watchdog monitoring."));
                sub_lines.push(String::new());
            }
        }

        let sub_arch = sub_lines.join("\n");
        self.parameter_bindings
            .insert("SUBSYSTEM_ARCHITECTURE_SECTION".to_string(), sub_arch.clone());
        self.parameter_bindings
            .insert("CONOPS_SECTION_4_SUBSYSTEMS".to_string(), sub_arch);

        // 3. Segment Allocation Matrix
        let mut seg_lines = vec![
            "| Subsystem Part | Operational Segment | Allocation Authority | Functional Role |".to_string(),
            "| :--- | :--- | :--- | :--- |".to_string(),
        ];
        if parts.is_empty() {
            seg_lines.push(format!("| `{sys_id}` | Primary Operational Segment | IEEE 1362 §5.3 | Autonomous Guidance & Control |"));
        } else {
            for p in &parts {
                let clean_p = sanitize_level_1b_operational_text(p);
                seg_lines.push(format!("| `{clean_p}` | Primary Operational Segment | IEEE 1362 §5.3 / DoDAF SV-1 | Realizes core operational mission functions |"));
            }
        }
        self.parameter_bindings
            .insert("SEGMENT_ALLOCATION_MATRIX".to_string(), seg_lines.join("\n"));

        // 4. Port Taxonomy
        let port_tax = "- **PORT-C2-INOUT (INOUT):** Bidirectional PACE command and control telemetry datalink interface.\n- **PORT-NAV-IN (IN):** Navigation, positioning, and reference frame telemetry input interface.\n- **PORT-ACT-OUT (OUT):** Deterministic actuator demand vector and containment control interface.\n- **PORT-PWR-IN (IN):** Regulated primary/auxiliary power bus distribution interface.";
        self.parameter_bindings
            .insert("PORT_TAXONOMY_SECTION".to_string(), port_tax.to_string());
    }

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
            "USER_CLASSES_AND_STAKEHOLDERS" => "- **UCL-01 System Operator (SO):** Direct supervisory mission management and boundary oversight.\n- **UCL-02 Range Safety Officer (RSO):** Airspace containment enforcement and failsafe override authority.\n- **UCL-03 Payload Specialist (PS):** Multi-modal sensor data interpretation and payload stream management.\n- **UCL-04 Maintenance Technician (MT):** O-Level pre-operation inspections, modular LRU swaps, and BIT checks.\n- **UCL-05 Safety Monitor (SM):** Continuous perimeter monitoring, environmental anomaly detection, and safety oversight.".to_string(),
            "OPERATIONAL_LIFECYCLE_MODES" => "- **Phase_Startup:** Power-on Built-In-Test (PBIT), sensor alignment, and pre-operation validation.\n- **Phase_NominalExecution:** Autonomous mission start, state corridor tracking, and real-time telemetry streaming.\n- **Phase_DegradedMode:** Non-critical sensor failover, PACE datalink fallback, and degraded envelope limits.\n- **Phase_ContingencyFailsafe:** Autonomous contingency execution, divert to recovery site, or state containment.\n- **Phase_SecureShutdown:** Autonomous precision arrival, power de-energization, and diagnostic blackbox archival.\n- **Phase_MaintenanceMode:** Diagnostic telemetry offload, calibration, firmware update, and hardware inspection.".to_string(),
            "WEIGHT_CRIT_1" => "0.40".to_string(),
            "WEIGHT_CRIT_2" => "0.35".to_string(),
            "WEIGHT_CRIT_3" => "0.25".to_string(),
            "SCORE_A_1" | "SCORE_B_2" | "SCORE_C_1" => "1.0".to_string(),
            "SCORE_A_2" | "SCORE_B_3" | "SCORE_C_3" => "0.8".to_string(),
            "SCORE_A_3" | "SCORE_B_1" | "SCORE_C_2" => "0.9".to_string(),
            "WEIGHTED_SCORE_A" => "0.90".to_string(),
            "WEIGHTED_SCORE_B" => "0.79".to_string(),
            "WEIGHTED_SCORE_C" => "0.80".to_string(),
            "SAIL_LEVEL" => "II".to_string(),
            "SORA_GROUND_RISK_CLASS" => "GRC-2".to_string(),
            "SORA_AIR_RISK_CLASS" => "ARC-a".to_string(),
            "V_CRUISE_NOMINAL_MPS" => "18.0".to_string(),
            "V_MAX_MPS" => "25.0".to_string(),
            "V_STALL_MPS" => "12.0".to_string(),
            "TOTAL_MTOW_KG" => "50.0".to_string(),
            "PAYLOAD_MAX_KG" => "5.0".to_string(),
            "ENDURANCE_NOMINAL_MIN" => "90.0".to_string(),
            "C2_RANGE_NOMINAL_KM" => "15.0".to_string(),
            "CEILING_MAX_M" => "3000.0".to_string(),
            "WIND_LIMIT_MAX_MPS" => "12.0".to_string(),
            "INGRESS_PROTECTION_RATING" => "IP54".to_string(),
            _ => {
                if token_upper.ends_with("_PERCENT") || token_upper.ends_with("_PCT") {
                    "10.0".to_string()
                } else if token_upper.ends_with("_KG") {
                    "5.0".to_string()
                } else if token_upper.ends_with("_MPS") || token_upper.ends_with("_M") {
                    "10.0".to_string()
                } else if token_upper.ends_with("_SEC") || token_upper.ends_with("_S") {
                    "1.0".to_string()
                } else if token_upper.ends_with("_MIN") {
                    "30.0".to_string()
                } else if token_upper.ends_with("_COUNT") {
                    "5".to_string()
                } else {
                    "N/A".to_string()
                }
            }
        }
    }

    pub fn substitute(&self, content: &str) -> String {
        if content.is_empty() {
            return String::new();
        }

        let placeholder_re = Regex::new(r"\{\{([A-Za-z0-9_]+)(?::([^\}]*))?\}\}").unwrap();
        let mut current = content.to_string();

        for _ in 0..3 {
            let mut changed = false;
            let next = placeholder_re
                .replace_all(&current, |caps: &regex::Captures| {
                    changed = true;
                    let key = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                    let default_val = caps.get(2).map(|m| m.as_str());
                    self.resolve_token(key, default_val)
                })
                .to_string();

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
            .or_else(|| self.inferred_system_identifier.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("Autonomous Cyber-Physical System");

        current = current.replace("the Abstract Cyber-Physical System Archetype", &format!("the {sys_target}"));
        current = current.replace("The Abstract Cyber-Physical System Archetype", &format!("The {sys_target}"));
        current = current.replace("an Abstract Cyber-Physical System Archetype", &format!("the {sys_target}"));
        current = current.replace("Abstract Cyber-Physical System Archetype", sys_target);
        current = current.replace("Autonomous Cyber-Physical System Archetype", sys_target);
        current = current.replace("AutonomousSystemArchetype", sys_target);

        sanitize_level_1b_operational_text(&current)
    }

    pub fn apply_domain_ontology_sanitization(&self, current: &mut String) {
        let dom = self.detected_domain.to_lowercase();
        if dom == "medical" {
            let re_para = Regex::new(r"(?i)\bparachute\b").unwrap();
            *current = re_para.replace_all(current, "failsafe joint brake").to_string();
            *current = current.replace("PARACHUTE", "JOINT_BRAKE");
            let re_alt = Regex::new(r"(?i)\b(?:altitude\s+AGL|m\s+AGL)\b").unwrap();
            *current = re_alt.replace_all(current, "mm").to_string();
            let re_astm = Regex::new(r"ASTM\s+F3411(?:-22a)?").unwrap();
            *current = re_astm.replace_all(current, "IEC 62304 / ISO 14971").to_string();
        } else if dom == "rail" {
            let re_para = Regex::new(r"(?i)\bparachute\b").unwrap();
            *current = re_para.replace_all(current, "pneumatic emergency brake").to_string();
            *current = current.replace("PARACHUTE", "EMERGENCY_BRAKE");
            let re_alt = Regex::new(r"(?i)\b(?:altitude\s+AGL|m\s+AGL)\b").unwrap();
            *current = re_alt.replace_all(current, "m").to_string();
            let re_astm = Regex::new(r"ASTM\s+F3411(?:-22a)?").unwrap();
            *current = re_astm.replace_all(current, "EN 50128 SIL 4").to_string();
        } else if dom == "marine" {
            let re_para = Regex::new(r"(?i)\bparachute\b").unwrap();
            *current = re_para.replace_all(current, "positive buoyancy drop-weight").to_string();
            *current = current.replace("PARACHUTE", "DROP_WEIGHT");
            let re_alt = Regex::new(r"(?i)\b(?:altitude\s+AGL|m\s+AGL)\b").unwrap();
            *current = re_alt.replace_all(current, "m Depth").to_string();
            let re_astm = Regex::new(r"ASTM\s+F3411(?:-22a)?").unwrap();
            *current = re_astm.replace_all(current, "DNV-GL-ST-E403").to_string();
        } else if dom == "space" {
            let re_para = Regex::new(r"(?i)\bparachute\b").unwrap();
            *current = re_para.replace_all(current, "autonomous de-orbit propulsion").to_string();
            *current = current.replace("PARACHUTE", "DEORBIT_THRUSTER");
            let re_alt = Regex::new(r"(?i)\b(?:altitude\s+AGL|m\s+AGL)\b").unwrap();
            *current = re_alt.replace_all(current, "km").to_string();
            let re_astm = Regex::new(r"ASTM\s+F3411(?:-22a)?").unwrap();
            *current = re_astm.replace_all(current, "ECSS-E-ST-40C").to_string();
        } else if self.is_non_aircraft {
            let re_para = Regex::new(r"(?i)\bparachute\b").unwrap();
            *current = re_para.replace_all(current, "recovery system").to_string();
            *current = current.replace("PARACHUTE", "RECOVERY");
        }

        if self.is_civilian {
            for idx in 1..=6 {
                let re_roe = Regex::new(&format!(r"\bROE-0{idx}\b")).unwrap();
                *current = re_roe.replace_all(current, format!("SAF-0{idx}").as_str()).to_string();
            }
            let re_roe_word = Regex::new(r"(?i)rules of engagement(?:\s*\(ROE\))?").unwrap();
            *current = re_roe_word.replace_all(current, "operational safety interlocks").to_string();
            let re_weapons = Regex::new(r"(?i)\bweapons release\b").unwrap();
            *current = re_weapons.replace_all(current, "high-consequence actuation").to_string();
        }
    }
}

pub fn bind_parameters(text: &str, engine: Option<&SysMLParameterBindingEngine>) -> String {
    if let Some(eng) = engine {
        eng.substitute(text)
    } else {
        let default_eng = SysMLParameterBindingEngine::new(None, None, None, false, None);
        default_eng.substitute(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_substitution_and_inline_default() {
        let mut bindings = HashMap::new();
        bindings.insert("SYSTEM_IDENTIFIER".to_string(), "SkyGuardian_UAV".to_string());
        let engine = SysMLParameterBindingEngine::new(Some(bindings), None, None, false, None);

        let input = "System: {{SYSTEM_IDENTIFIER}} with baud {{BAUD_RATE:57600}} and default {{UNKNOWN_KEY:FallbackValue}}";
        let output = engine.substitute(input);
        assert!(output.contains("SkyGuardian_UAV"));
        assert!(output.contains("FallbackValue"));
    }

    #[test]
    fn test_domain_sanitization_civilian_and_medical() {
        let engine = SysMLParameterBindingEngine::new(None, None, None, false, Some("medical"));
        let input = "The parachute deployment is authorized under ROE-01 for weapons release.";
        let output = engine.substitute(input);
        assert!(output.contains("failsafe joint brake"));
        assert!(output.contains("SAF-01"));
        assert!(output.contains("high-consequence actuation"));
    }

    #[test]
    fn test_mass_and_physics_derivation() {
        let mut bindings = HashMap::new();
        bindings.insert("TOTAL_MTOW_KG".to_string(), "100.0".to_string());
        let engine = SysMLParameterBindingEngine::new(Some(bindings), None, None, false, None);
        assert_eq!(engine.resolve_token("TOTAL_MTOW_KG", None), "100.0");
        let v_term = engine.resolve_token("V_TERMINAL_UNMITIGATED_MPS", None);
        assert_ne!(v_term, "N/A");
    }
}
