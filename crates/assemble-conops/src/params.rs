//! SysML AST Parameter Binding Engine and template token resolution.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde_json::Value;

use crate::sanitize::{sanitize_level_1b_operational_text, wrap_mermaid_label};

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
    ("TOTAL_MASS_KG", "25.0"),
    ("MASS_FRACTION_STRUCTURE_PCT", "30.0"),
    ("MASS_BUDGET_STRUCTURE_KG", "7.5"),
    ("POWER_NOMINAL_STRUCTURE_W", "0.0"),
    ("POWER_PEAK_STRUCTURE_W", "0.0"),
    ("MASS_FRACTION_CONTROLLER_PCT", "15.0"),
    ("MASS_BUDGET_CONTROLLER_KG", "3.75"),
    ("POWER_NOMINAL_CONTROLLER_W", "25.0"),
    ("POWER_PEAK_CONTROLLER_W", "50.0"),
    ("M_MAX_LIMIT_KG", "25.0"),
    ("M_MAX_NOMINAL_KG", "20.0"),
    ("V_NOMINAL_MIN_MPS", "15.0"),
    ("V_NOMINAL_MAX_MPS", "25.0"),
    ("V_NOMINAL_MPS", "20.0"),
    ("V_MAX_MPS", "35.0"),
    ("V_MAX_NOMINAL_MPS", "30.0"),
    ("V_MIN_MAX_MPS", "10.0"),
    ("V_MIN_NOMINAL_MPS", "8.0"),
    ("SPATIAL_LIMIT_MAX_M", "120.0"),
    ("SPATIAL_LIMIT_NOMINAL_M", "100.0"),
    ("SPATIAL_LIMIT_UNIT", "m"),
    ("DISTURBANCE_LIMIT_MAX_MPS", "15.0"),
    ("DISTURBANCE_LIMIT_NOMINAL_MPS", "12.0"),
];

pub static PLACEHOLDER_PATTERN: &str = r"\{\{([A-Za-z0-9_]+)(?::([^\}]*))?\}\}";
pub static RAW_TOKEN_FINDER: &str = r"\{\{[A-Za-z0-9_]+(?::[^\}]*)?\}\}";


/// Formal lifecycle archetypes for cyber-physical mission systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleType {
    ReusableRecovery,
    ExpendableDisposal,
    ContinuousStationary,
    PersistentOrbital,
    TrackBoundGuided,
}

impl LifecycleType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ReusableRecovery => "REUSABLE_RECOVERY",
            Self::ExpendableDisposal => "EXPENDABLE_DISPOSAL",
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

fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn find_first_package(text: &str) -> Option<&str> {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if let Some(pos) = text[i..].find("package") {
            let start = i + pos;
            let end = start + 7;
            let left_ok = start == 0 || !is_ident_char(bytes[start - 1]);
            let right_ok = end < bytes.len() && bytes[end].is_ascii_whitespace();
            if left_ok && right_ok {
                let mut j = end;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                let ident_start = j;
                while j < bytes.len() && is_ident_char(bytes[j]) {
                    j += 1;
                }
                if j > ident_start {
                    return Some(&text[ident_start..j]);
                }
            }
            i = end;
        } else {
            break;
        }
    }
    None
}

fn find_all_part_defs(text: &str) -> Vec<&str> {
    let mut results = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if let Some(pos) = text[i..].find("part") {
            let start = i + pos;
            let end = start + 4;
            let left_ok = start == 0 || !is_ident_char(bytes[start - 1]);
            let right_ok = end < bytes.len() && bytes[end].is_ascii_whitespace();
            if left_ok && right_ok {
                let mut j = end;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if text[j..].starts_with("def") {
                    let def_end = j + 3;
                    if def_end < bytes.len() && bytes[def_end].is_ascii_whitespace() {
                        let mut k = def_end;
                        while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                            k += 1;
                        }
                        let ident_start = k;
                        while k < bytes.len() && is_ident_char(bytes[k]) {
                            k += 1;
                        }
                        if k > ident_start {
                            results.push(&text[ident_start..k]);
                            i = k;
                            continue;
                        }
                    }
                }
            }
            i = end;
        } else {
            break;
        }
    }
    results
}

fn find_all_attributes(text: &str) -> Vec<(&str, &str)> {
    let mut results = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if let Some(pos) = text[i..].find("attribute") {
            let start = i + pos;
            let end = start + 9;
            let left_ok = start == 0 || !is_ident_char(bytes[start - 1]);
            let right_ok = end < bytes.len() && bytes[end].is_ascii_whitespace();
            if left_ok && right_ok {
                let mut j = end;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                let name_start = j;
                while j < bytes.len() && is_ident_char(bytes[j]) {
                    j += 1;
                }
                if j > name_start {
                    let attr_name = &text[name_start..j];
                    let mut k = j;
                    while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                        k += 1;
                    }
                    if k < bytes.len() && bytes[k] == b':' {
                        k += 1;
                        while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                            k += 1;
                        }
                        while k < bytes.len() && is_ident_char(bytes[k]) {
                            k += 1;
                        }
                        while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                            k += 1;
                        }
                    }
                    if k < bytes.len() && bytes[k] == b'=' {
                        k += 1;
                        let val_start = k;
                        while k < bytes.len() && bytes[k] != b';' && bytes[k] != b'\n' {
                            k += 1;
                        }
                        if k < bytes.len() && bytes[k] == b';' {
                            let raw_val = text[val_start..k].trim().trim_matches(['"', '\'']);
                            results.push((attr_name, raw_val));
                            i = k + 1;
                            continue;
                        }
                    }
                }
            }
            i = end;
        } else {
            break;
        }
    }
    results
}

fn extract_numeric_substring(val: &str) -> Option<&str> {
    let bytes = val.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        let is_sign = (b == b'+' || b == b'-')
            && i + 1 < bytes.len()
            && (bytes[i + 1].is_ascii_digit()
                || (bytes[i + 1] == b'.' && i + 2 < bytes.len() && bytes[i + 2].is_ascii_digit()));
        let is_digit = b.is_ascii_digit();
        let is_dot_digit = b == b'.' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_digit();

        if is_sign || is_digit || is_dot_digit {
            let start = i;
            let mut j = if is_sign { i + 1 } else { i };
            let mut has_dot = false;
            let mut has_digits = false;
            while j < bytes.len() {
                if bytes[j].is_ascii_digit() {
                    has_digits = true;
                    j += 1;
                } else if bytes[j] == b'.' && !has_dot {
                    if has_digits || (j + 1 < bytes.len() && bytes[j + 1].is_ascii_digit()) {
                        has_dot = true;
                        j += 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            if has_digits {
                return Some(&val[start..j]);
            }
        }
        i += 1;
    }
    None
}

fn replace_placeholders<F>(text: &str, mut resolver: F) -> (String, bool)
where
    F: FnMut(&str, Option<&str>) -> String,
{
    let bytes = text.as_bytes();
    let mut result = String::with_capacity(text.len());
    let mut idx = 0;
    let mut changed = false;

    while idx < bytes.len() {
        if idx + 4 <= bytes.len() && bytes[idx] == b'{' && bytes[idx + 1] == b'{' {
            let start = idx;
            let mut j = idx + 2;
            let mut found_end = None;
            while j + 1 < bytes.len() {
                if bytes[j] == b'}' && bytes[j + 1] == b'}' {
                    found_end = Some(j);
                    break;
                }
                if bytes[j] == b'\n' {
                    break;
                }
                j += 1;
            }

            if let Some(end_pos) = found_end {
                let inner = &text[start + 2..end_pos];
                let (key, default_val) = match inner.split_once(':') {
                    Some((k, d)) => (k, Some(d)),
                    None => (inner, None),
                };
                if !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    let replacement = resolver(key, default_val);
                    result.push_str(&replacement);
                    changed = true;
                    idx = end_pos + 2;
                    continue;
                }
            }
        }

        let ch = text[idx..].chars().next().unwrap();
        result.push(ch);
        idx += ch.len_utf8();
    }

    (result, changed)
}

fn replace_word_case_insensitive(text: &str, target_word: &str, replacement: &str) -> String {
    let lower_text = text.to_lowercase();
    let lower_target = target_word.to_lowercase();
    let target_len = lower_target.len();
    let bytes = text.as_bytes();

    let mut result = String::with_capacity(text.len());
    let mut last_idx = 0;
    let mut search_idx = 0;

    while search_idx < lower_text.len() {
        if let Some(pos) = lower_text[search_idx..].find(&lower_target) {
            let start = search_idx + pos;
            let end = start + target_len;

            let left_ok = start == 0 || !(bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_');
            let right_ok = end == bytes.len() || !(bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_');

            if left_ok && right_ok {
                result.push_str(&text[last_idx..start]);
                result.push_str(replacement);
                last_idx = end;
                search_idx = end;
            } else {
                search_idx = start + 1;
            }
        } else {
            break;
        }
    }
    result.push_str(&text[last_idx..]);
    result
}

fn replace_altitude_agl(text: &str, replacement: &str) -> String {
    let lower = text.to_lowercase();
    let bytes = text.as_bytes();
    let mut result = String::with_capacity(text.len());
    let mut last_idx = 0;
    let mut i = 0;

    while i < lower.len() {
        if let Some(pos) = lower[i..].find("agl") {
            let agl_start = i + pos;
            let agl_end = agl_start + 3;
            let right_ok = agl_end == bytes.len() || !(bytes[agl_end].is_ascii_alphanumeric() || bytes[agl_end] == b'_');

            if right_ok && agl_start > 0 && bytes[agl_start - 1].is_ascii_whitespace() {
                let mut prev = agl_start - 1;
                while prev > 0 && bytes[prev].is_ascii_whitespace() {
                    prev -= 1;
                }
                let prev_end = prev + 1;
                let mut word_start = prev;
                while word_start > 0 && (bytes[word_start - 1].is_ascii_alphanumeric() || bytes[word_start - 1] == b'_') {
                    word_start -= 1;
                }
                let left_ok = word_start == 0 || !(bytes[word_start - 1].is_ascii_alphanumeric() || bytes[word_start - 1] == b'_');
                let prev_word = &lower[word_start..prev_end];

                if left_ok && (prev_word == "altitude" || prev_word == "m") {
                    result.push_str(&text[last_idx..word_start]);
                    result.push_str(replacement);
                    last_idx = agl_end;
                    i = agl_end;
                    continue;
                }
            }
            i = agl_end;
        } else {
            break;
        }
    }
    result.push_str(&text[last_idx..]);
    result
}

fn replace_astm_f3411(text: &str, replacement: &str) -> String {
    let lower = text.to_lowercase();
    let bytes = text.as_bytes();
    let mut result = String::with_capacity(text.len());
    let mut last_idx = 0;
    let mut i = 0;

    while i < lower.len() {
        if let Some(pos) = lower[i..].find("astm") {
            let start = i + pos;
            let mut j = start + 4;
            let left_ok = start == 0 || !(bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_');
            if left_ok {
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if lower[j..].starts_with("f3411") {
                    j += 5;
                    if lower[j..].starts_with("-22a") {
                        j += 4;
                    }
                    result.push_str(&text[last_idx..start]);
                    result.push_str(replacement);
                    last_idx = j;
                    i = j;
                    continue;
                }
            }
            i = start + 4;
        } else {
            break;
        }
    }
    result.push_str(&text[last_idx..]);
    result
}

fn replace_roe_phrase(text: &str) -> String {
    let lower = text.to_lowercase();
    let bytes = text.as_bytes();
    let mut result = String::with_capacity(text.len());
    let mut last_idx = 0;
    let mut i = 0;

    while i < lower.len() {
        if let Some(pos) = lower[i..].find("rules of engagement") {
            let start = i + pos;
            let mut end = start + "rules of engagement".len();
            let mut j = end;
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                j += 1;
            }
            if lower[j..].starts_with("(roe)") {
                end = j + 5;
            }
            result.push_str(&text[last_idx..start]);
            result.push_str("operational safety interlocks");
            last_idx = end;
            i = end;
        } else {
            break;
        }
    }
    result.push_str(&text[last_idx..]);
    result
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
        engine._derive_domain_ontology();
        engine._derive_lifecycle_contract();
        engine._derive_quadratic_physics();
        engine._derive_energy_budgets();
        engine._derive_domain_regulatory_standards();
        engine._derive_domain_risk_assessment();
        engine._derive_subsystem_architecture();

        engine
    }

    pub fn has_ast_component(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        self.ast_part_names.iter().any(|p| {
            let p_lower = p.to_lowercase();
            p_lower == name_lower || p_lower.contains(&name_lower)
        })
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
        self._derive_domain_risk_assessment();
        self._derive_lifecycle_contract();
        self._derive_quadratic_physics();
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
        if let Some(pkg_name) = find_first_package(text) {
            let pkg_name = pkg_name.trim();
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

        for p_name in find_all_part_defs(text) {
            let p_name = p_name.trim().to_string();
            self.ast_part_names.insert(p_name.clone());
            self.explicit_keys.insert(p_name.clone());
            self.explicit_keys.insert(p_name.to_uppercase());
        }

        for (attr_name, raw_val) in find_all_attributes(text) {
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
            for line in content.lines() {
                let trimmed = line.trim();
                if let Some(rest) = trimmed.strip_prefix('#') {
                    if !rest.starts_with('#') {
                        let title = rest.trim();
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
        let num_val = extract_numeric_substring(val)
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
                if let Some(m) = extract_numeric_substring(val) {
                    if let Ok(v) = m.parse::<f64>() {
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
        let v_max = self
            .parameter_bindings
            .get("V_MAX_MPS")
            .or_else(|| self.parameter_bindings.get("V_MAX_NOMINAL_MPS"))
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(25.0);

        let ek_unmit = (0.5 * m * v_max.powi(2) * 10.0).round() / 10.0;

        let explicit_v_res = self
            .parameter_bindings
            .get("V_RESIDUAL_MPS")
            .or_else(|| self.parameter_bindings.get("V_RESIDUAL"))
            .and_then(|s| s.parse::<f64>().ok());

        let has_parachute_component = self.has_ast_component("parachute") || self.has_ast_component("chute");
        let has_parachute_params = self.parameter_bindings.contains_key("S_CANOPY")
            || self.parameter_bindings.contains_key("PARACHUTE_DRAG_COEFFICIENT");

        let v_residual = if let Some(vr) = explicit_v_res {
            vr
        } else if has_parachute_component || has_parachute_params {
            // Dynamic derivation strictly from user-supplied AST / parameter definitions
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
            let g = 9.80665;
            if denom > 0.0 && m > 0.0 {
                ((2.0 * m * g / denom).sqrt() * 100.0).round() / 100.0
            } else {
                1.5
            }
        } else if self.parameter_bindings.get("LIFECYCLE_TYPE").map(|s| s == "EXPENDABLE_DISPOSAL").unwrap_or(false)
            || self.parameter_bindings.get("IS_EXPENDABLE").map(|s| s == "true").unwrap_or(false)
        {
            // For expendable platforms without deceleration containment, residual perimeter impact velocity equals v_max
            v_max
        } else if self.detected_domain == "medical" {
            0.0
        } else {
            1.5
        };

        let ek_mit = (0.5 * m * v_residual.powi(2) * 10.0).round() / 10.0;

        self.parameter_bindings
            .entry("V_BOUNDARY_MAX_MPS".to_string())
            .or_insert_with(|| format!("{:.1}", v_max));
        self.parameter_bindings
            .entry("E_BOUNDARY_IMPACT_JOULES".to_string())
            .or_insert_with(|| format!("{:.1}", ek_unmit));
        self.parameter_bindings
            .entry("E_K_UNMITIGATED_JOULES".to_string())
            .or_insert_with(|| format!("{:.1}", ek_unmit));
        self.parameter_bindings
            .entry("V_RESIDUAL_MPS".to_string())
            .or_insert_with(|| format!("{:.1}", v_residual));
        self.parameter_bindings
            .entry("E_K_MITIGATED_JOULES".to_string())
            .or_insert_with(|| format!("{:.1}", ek_mit));
        self.parameter_bindings
            .entry("MITIGATED_KINETIC_ENERGY_J".to_string())
            .or_insert_with(|| format!("{:.1}", ek_mit));
        self.parameter_bindings
            .entry("V_TERMINAL_UNMITIGATED_MPS".to_string())
            .or_insert_with(|| format!("{:.1}", v_max));
        self.parameter_bindings
            .entry("V_TERMINAL_MITIGATED_MPS".to_string())
            .or_insert_with(|| format!("{:.1}", v_residual));
        self.parameter_bindings
            .entry("SAFE_CONTAINMENT_VELOCITY_MS".to_string())
            .or_insert_with(|| format!("{:.1}", v_residual));
        self.parameter_bindings
            .entry("CONTAINMENT_DEPLOY_TIME_MAX_S".to_string())
            .or_insert_with(|| "0.5".to_string());
    }

    pub fn _derive_domain_risk_assessment(&mut self) {
        if !self.parameter_bindings.contains_key("DOMAIN_RISK_ASSESSMENT_CLAUSES") {
            let clauses = "<!-- Domain-specific risk containment and safety integrity clauses derived from active domain schema -->";
            self.parameter_bindings.insert("DOMAIN_RISK_ASSESSMENT_CLAUSES".to_string(), clauses.to_string());
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
        let is_military = ["military", "defense"].contains(&dom.as_str());
        self.is_non_aircraft = ["medical", "rail", "marine", "space", "industrial"].contains(&dom.as_str());
        self.is_civilian = !is_military;

        if self.is_civilian {
            self.parameter_bindings.insert("INTERLOCK_PREFIX".to_string(), "SAF".to_string());
            self.parameter_bindings.insert("INTERLOCK_SECTION_TITLE".to_string(), "Operational Safety Interlocks & High-Consequence Controls".to_string());
            self.parameter_bindings.insert("INTERLOCK_POLICY_NAME".to_string(), "Operational Safety Interlocks".to_string());
            self.parameter_bindings.insert("TARGET_VERIFICATION_LABEL".to_string(), "Positive Condition Verification (PCV)".to_string());
            self.parameter_bindings.insert("TARGET_VERIFICATION_ACRONYM".to_string(), "PCV".to_string());
            self.parameter_bindings.insert("TARGET_VERIFICATION_PHRASE".to_string(), "positive condition verification".to_string());
            self.parameter_bindings.insert("HIGH_CONSEQUENCE_ACTION".to_string(), "high-consequence actuation".to_string());
            self.parameter_bindings.insert("COLLATERAL_RISK_PHRASE".to_string(), "adjacent operational risk".to_string());
        } else {
            self.parameter_bindings.insert("INTERLOCK_PREFIX".to_string(), "ROE".to_string());
            self.parameter_bindings.insert("INTERLOCK_SECTION_TITLE".to_string(), "Rules of Engagement (ROE) & Operational Safety Interlocks".to_string());
            self.parameter_bindings.insert("INTERLOCK_POLICY_NAME".to_string(), "Rules of Engagement (ROE)".to_string());
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
        let is_expendable = self
            .parameter_bindings
            .get("IS_EXPENDABLE")
            .map(|s| s == "true" || s == "1")
            .unwrap_or(false)
            || dom == "disposable"
            || dom == "single_use"
            || dom == "expendable"
            || self.has_ast_component("expendable")
            || self.has_ast_component("effector")
            || self.has_ast_component("munition");

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

        self.parameter_bindings
            .insert("LIFECYCLE_TYPE".to_string(), selected_type.as_str().to_string());

        match selected_type {
            LifecycleType::ExpendableDisposal => {
                self.parameter_bindings.entry("PRIMARY_TERMINAL_TARGET".to_string()).or_insert_with(|| "Designated Kinetic Intercept / Terminal Containment Zone".to_string());
                self.parameter_bindings.entry("SECONDARY_TERMINAL_TARGET".to_string()).or_insert_with(|| "Safe Containment Ditching Zone LZ-DIVERT-ALPHA".to_string());
                self.parameter_bindings.entry("PRIMARY_RECOVERY_FACILITY".to_string()).or_insert_with(|| "Safe Impact Containment Grid (No Vehicle Recovery)".to_string());
                self.parameter_bindings.entry("SECONDARY_RECOVERY_FACILITY".to_string()).or_insert_with(|| "Secondary Safe Ditching Area (No Vehicle Recovery)".to_string());
                self.parameter_bindings.entry("LIFECYCLE_TRANSIT_MODE".to_string()).or_insert_with(|| "Terminal_Engagement_Transit".to_string());
                self.parameter_bindings.entry("LIFECYCLE_FAILSAFE_SEQUENCE".to_string()).or_insert_with(|| "autonomous terminal containment ditching and zeroization sequence".to_string());
                self.parameter_bindings.entry("LIFECYCLE_END_STATE".to_string()).or_insert_with(|| "All assigned operational corridor waypoints fully traversed and verified; zero unauthorized state boundary excursions; zero unmitigated collision or interference hazards; and successful terminal impact or safe containment ditching in the designated containment zone with complete cryptographic zeroization.".to_string());
                self.parameter_bindings.entry("LIFECYCLE_BINGO_SAFETY_ACTION".to_string()).or_insert_with(|| "Continuously compute closed-loop dynamic resource state and execute terminal target engagement or safe containment ditching with cryptographic zeroization upon reaching safety thresholds ($R(t) \\le R_{\\mathrm{threshold}}(t)$).".to_string());
                self.parameter_bindings.entry("LIFECYCLE_POST_OP_STATE".to_string()).or_insert_with(|| "Terminal kinetic impact or safe containment ditching with complete cryptographic key zeroization, volatile memory purge, and structural containment verification".to_string());
                self.parameter_bindings.entry("CONTAINMENT_MITIGATION_TYPE".to_string()).or_insert_with(|| "SAFE_IMPACT_ZEROIZATION".to_string());
                self.parameter_bindings.entry("RECOVERY_DEVICE_TERM".to_string()).or_insert_with(|| "containment".to_string());
                self.parameter_bindings.entry("FAILSAFE_CONTAINMENT_SYSTEM".to_string()).or_insert_with(|| "autonomous containment watchdog and kinetic energy absorption".to_string());
                self.parameter_bindings.entry("TIER4_CONTAINMENT_DESC".to_string()).or_insert_with(|| "autonomous terminal containment ditching or safe impact zeroization".to_string());
            }
            LifecycleType::ContinuousStationary => {
                self.parameter_bindings.entry("PRIMARY_TERMINAL_TARGET".to_string()).or_insert_with(|| "Primary Sterile Field Docking Station".to_string());
                self.parameter_bindings.entry("SECONDARY_TERMINAL_TARGET".to_string()).or_insert_with(|| "Secondary Clinical Safe Staging Console".to_string());
                self.parameter_bindings.entry("PRIMARY_RECOVERY_FACILITY".to_string()).or_insert_with(|| "Clinical Operating Suite".to_string());
                self.parameter_bindings.entry("SECONDARY_RECOVERY_FACILITY".to_string()).or_insert_with(|| "Secondary Maintenance Staging Bay".to_string());
                self.parameter_bindings.entry("LIFECYCLE_TRANSIT_MODE".to_string()).or_insert_with(|| "Autonomous_Clinical_Safing".to_string());
                self.parameter_bindings.entry("LIFECYCLE_FAILSAFE_SEQUENCE".to_string()).or_insert_with(|| "electromechanical joint brake lock and sterile field safing sequence".to_string());
                self.parameter_bindings.entry("LIFECYCLE_END_STATE".to_string()).or_insert_with(|| "All assigned operational trajectory segments fully traversed and verified; zero unauthorized workspace boundary excursions; and successful safe parking at the primary sterile console with residual power reserves strictly satisfying $R_{\\mathrm{reserve}} \\ge \\text{Ratio}_{\\text{reserve\\_min}} \\cdot R_{\\mathrm{capacity}}$.".to_string());
                self.parameter_bindings.entry("LIFECYCLE_BINGO_SAFETY_ACTION".to_string()).or_insert_with(|| "Continuously compute closed-loop dynamic resource state and execute automated electromechanical joint brake locking and sterile field preservation upon reaching safety thresholds ($R(t) \\le R_{\\mathrm{threshold}}(t)$).".to_string());
                self.parameter_bindings.entry("LIFECYCLE_POST_OP_STATE".to_string()).or_insert_with(|| "Post-operation stationary safe rest at sterile field console with engaged electromechanical joint brake locks, complete cryptographic key zeroization, and diagnostic log offloading".to_string());
                self.parameter_bindings.entry("CONTAINMENT_MITIGATION_TYPE".to_string()).or_insert_with(|| "ELECTROMECHANICAL_BRAKE_LOCK".to_string());
                self.parameter_bindings.entry("RECOVERY_DEVICE_TERM".to_string()).or_insert_with(|| "failsafe joint brake".to_string());
                self.parameter_bindings.entry("FAILSAFE_CONTAINMENT_SYSTEM".to_string()).or_insert_with(|| "electromechanical joint brake lock and sterile field interlocks".to_string());
                self.parameter_bindings.entry("TIER4_CONTAINMENT_DESC".to_string()).or_insert_with(|| "instant electromechanical joint brake locking and actuator power bus disconnect".to_string());
            }
            LifecycleType::TrackBoundGuided => {
                self.parameter_bindings.entry("PRIMARY_TERMINAL_TARGET".to_string()).or_insert_with(|| "Primary Rail Maintenance Siding".to_string());
                self.parameter_bindings.entry("SECONDARY_TERMINAL_TARGET".to_string()).or_insert_with(|| "Secondary Controlled Deceleration Spur LZ-DIVERT-ALPHA".to_string());
                self.parameter_bindings.entry("PRIMARY_RECOVERY_FACILITY".to_string()).or_insert_with(|| "Classification Yard Maintenance Depot".to_string());
                self.parameter_bindings.entry("SECONDARY_RECOVERY_FACILITY".to_string()).or_insert_with(|| "Secondary Service Siding".to_string());
                self.parameter_bindings.entry("LIFECYCLE_TRANSIT_MODE".to_string()).or_insert_with(|| "Autonomous_Track_Deceleration".to_string());
                self.parameter_bindings.entry("LIFECYCLE_FAILSAFE_SEQUENCE".to_string()).or_insert_with(|| "controlled track deceleration and siding brake sequence".to_string());
                self.parameter_bindings.entry("LIFECYCLE_END_STATE".to_string()).or_insert_with(|| "All assigned track blocks and route sections fully traversed and verified; zero unauthorized siding excursions; and successful controlled arrival at the primary maintenance siding or designated secondary spur with residual resources strictly satisfying $R_{\\mathrm{reserve}} \\ge \\text{Ratio}_{\\text{reserve\\_min}} \\cdot R_{\\mathrm{capacity}}$.".to_string());
                self.parameter_bindings.entry("LIFECYCLE_BINGO_SAFETY_ACTION".to_string()).or_insert_with(|| "Continuously compute closed-loop dynamic resource state and execute controlled track deceleration or secondary maintenance siding divert upon reaching safety thresholds ($R(t) \\le R_{\\mathrm{threshold}}(t)$).".to_string());
                self.parameter_bindings.entry("LIFECYCLE_POST_OP_STATE".to_string()).or_insert_with(|| "Post-operation stationary rest at rail siding with mechanical parking brakes engaged, pneumatic systems vented, cryptographic key zeroization, and diagnostic log offloading".to_string());
                self.parameter_bindings.entry("CONTAINMENT_MITIGATION_TYPE".to_string()).or_insert_with(|| "TRACK_SIDING_BRAKE".to_string());
                self.parameter_bindings.entry("RECOVERY_DEVICE_TERM".to_string()).or_insert_with(|| "pneumatic emergency brake".to_string());
                self.parameter_bindings.entry("FAILSAFE_CONTAINMENT_SYSTEM".to_string()).or_insert_with(|| "pneumatic emergency brake and track siding deceleration system".to_string());
                self.parameter_bindings.entry("TIER4_CONTAINMENT_DESC".to_string()).or_insert_with(|| "emergency brake pipe venting and mechanical friction caliper clamping".to_string());
            }
            LifecycleType::PersistentOrbital => {
                self.parameter_bindings.entry("PRIMARY_TERMINAL_TARGET".to_string()).or_insert_with(|| "Designated De-Orbit Reentry Corridor / Graveyard Orbit".to_string());
                self.parameter_bindings.entry("SECONDARY_TERMINAL_TARGET".to_string()).or_insert_with(|| "Secondary Disposal Orbit Node LZ-DIVERT-ALPHA".to_string());
                self.parameter_bindings.entry("PRIMARY_RECOVERY_FACILITY".to_string()).or_insert_with(|| "Designated Atmospheric Demise Footprint".to_string());
                self.parameter_bindings.entry("SECONDARY_RECOVERY_FACILITY".to_string()).or_insert_with(|| "Stable Graveyard Disposal Orbit".to_string());
                self.parameter_bindings.entry("LIFECYCLE_TRANSIT_MODE".to_string()).or_insert_with(|| "Autonomous_Disposal_Burn_Transit".to_string());
                self.parameter_bindings.entry("LIFECYCLE_FAILSAFE_SEQUENCE".to_string()).or_insert_with(|| "autonomous de-orbit disposal and passivation sequence".to_string());
                self.parameter_bindings.entry("LIFECYCLE_END_STATE".to_string()).or_insert_with(|| "All assigned orbital mission phases fully executed and verified; zero unauthorized constellation slot excursions; zero unmitigated orbital debris collision hazards; and successful de-orbit disposal or transfer to graveyard orbit with residual propellant satisfying disposal containment standards.".to_string());
                self.parameter_bindings.entry("LIFECYCLE_BINGO_SAFETY_ACTION".to_string()).or_insert_with(|| "Continuously compute closed-loop dynamic resource state and execute autonomous de-orbit disposal burn or graveyard orbit maneuver upon reaching safety thresholds ($R(t) \\le R_{\\mathrm{threshold}}(t)$).".to_string());
                self.parameter_bindings.entry("LIFECYCLE_POST_OP_STATE".to_string()).or_insert_with(|| "Post-mission passivation and safe disposal state with reaction wheels desaturated, batteries passivated, cryptographic key zeroization, and final ephemeris offload".to_string());
                self.parameter_bindings.entry("CONTAINMENT_MITIGATION_TYPE".to_string()).or_insert_with(|| "DEORBIT_DISPOSAL_BURN".to_string());
                self.parameter_bindings.entry("RECOVERY_DEVICE_TERM".to_string()).or_insert_with(|| "autonomous de-orbit propulsion".to_string());
                self.parameter_bindings.entry("FAILSAFE_CONTAINMENT_SYSTEM".to_string()).or_insert_with(|| "autonomous de-orbit propulsion and orbital passivation unit".to_string());
                self.parameter_bindings.entry("TIER4_CONTAINMENT_DESC".to_string()).or_insert_with(|| "autonomous retro-burn execution, reaction wheel desaturation, and battery passivation".to_string());
            }
            LifecycleType::ReusableRecovery => {
                let has_chute = self.has_ast_component("parachute")
                    || self.has_ast_component("chute")
                    || self.parameter_bindings.get("CONTAINMENT_MITIGATION_TYPE").map(|s| s.contains("PARACHUTE")).unwrap_or(false);
                let has_buoyant = self.has_ast_component("buoyancy")
                    || self.has_ast_component("drop_weight")
                    || self.has_ast_component("ballast");
                let has_brake = self.has_ast_component("brake");

                if has_chute {
                    self.parameter_bindings.entry("CONTAINMENT_MITIGATION_TYPE".to_string()).or_insert_with(|| "PARACHUTE_DESCENT".to_string());
                    self.parameter_bindings.entry("RECOVERY_DEVICE_TERM".to_string()).or_insert_with(|| "parachute recovery system".to_string());
                    self.parameter_bindings.entry("FAILSAFE_CONTAINMENT_SYSTEM".to_string()).or_insert_with(|| "certified parachute recovery system".to_string());
                    self.parameter_bindings.entry("TIER4_CONTAINMENT_DESC".to_string()).or_insert_with(|| "autonomous parachute deployment and motor drive power isolation".to_string());
                    self.parameter_bindings.entry("LIFECYCLE_FAILSAFE_SEQUENCE".to_string()).or_insert_with(|| "autonomous return-to-base or parachute recovery sequence".to_string());
                } else if has_buoyant {
                    self.parameter_bindings.entry("CONTAINMENT_MITIGATION_TYPE".to_string()).or_insert_with(|| "BUOYANT_SURFACE_ASCENT".to_string());
                    self.parameter_bindings.entry("RECOVERY_DEVICE_TERM".to_string()).or_insert_with(|| "positive buoyancy drop-weight".to_string());
                    self.parameter_bindings.entry("FAILSAFE_CONTAINMENT_SYSTEM".to_string()).or_insert_with(|| "galvanic drop-weight ballast release and passive buoyancy ascent system".to_string());
                    self.parameter_bindings.entry("TIER4_CONTAINMENT_DESC".to_string()).or_insert_with(|| "galvanic drop-weight release and passive buoyancy ascent".to_string());
                    self.parameter_bindings.entry("LIFECYCLE_FAILSAFE_SEQUENCE".to_string()).or_insert_with(|| "ballast expulsion and passive buoyant surface recovery sequence".to_string());
                } else if has_brake {
                    self.parameter_bindings.entry("CONTAINMENT_MITIGATION_TYPE".to_string()).or_insert_with(|| "MECHANICAL_BRAKE_DECELERATION".to_string());
                    self.parameter_bindings.entry("RECOVERY_DEVICE_TERM".to_string()).or_insert_with(|| "emergency brake".to_string());
                    self.parameter_bindings.entry("FAILSAFE_CONTAINMENT_SYSTEM".to_string()).or_insert_with(|| "autonomous friction brake deceleration system".to_string());
                    self.parameter_bindings.entry("TIER4_CONTAINMENT_DESC".to_string()).or_insert_with(|| "friction caliper clamping and actuator power cutoff".to_string());
                    self.parameter_bindings.entry("LIFECYCLE_FAILSAFE_SEQUENCE".to_string()).or_insert_with(|| "autonomous deceleration and safe parking sequence".to_string());
                } else {
                    self.parameter_bindings.entry("CONTAINMENT_MITIGATION_TYPE".to_string()).or_insert_with(|| "FAILSAFE_CONTAINMENT".to_string());
                    self.parameter_bindings.entry("RECOVERY_DEVICE_TERM".to_string()).or_insert_with(|| "failsafe containment".to_string());
                    self.parameter_bindings.entry("FAILSAFE_CONTAINMENT_SYSTEM".to_string()).or_insert_with(|| "autonomous failsafe containment system".to_string());
                    self.parameter_bindings.entry("TIER4_CONTAINMENT_DESC".to_string()).or_insert_with(|| "autonomous emergency containment actuation or actuator power cutoff".to_string());
                    self.parameter_bindings.entry("LIFECYCLE_FAILSAFE_SEQUENCE".to_string()).or_insert_with(|| "autonomous return-to-base sequence".to_string());
                }

                self.parameter_bindings.entry("PRIMARY_TERMINAL_TARGET".to_string()).or_insert_with(|| "Primary Recovery Base".to_string());
                self.parameter_bindings.entry("SECONDARY_TERMINAL_TARGET".to_string()).or_insert_with(|| "Secondary Divert Site LZ-DIVERT-ALPHA".to_string());
                self.parameter_bindings.entry("PRIMARY_RECOVERY_FACILITY".to_string()).or_insert_with(|| "Primary Recovery Base".to_string());
                self.parameter_bindings.entry("SECONDARY_RECOVERY_FACILITY".to_string()).or_insert_with(|| "Secondary Divert Base".to_string());
                self.parameter_bindings.entry("LIFECYCLE_TRANSIT_MODE".to_string()).or_insert_with(|| "Autonomous_RTB_Transit".to_string());
                self.parameter_bindings.entry("LIFECYCLE_END_STATE".to_string()).or_insert_with(|| "All assigned operational corridor waypoints fully traversed and verified; zero unauthorized state boundary excursions; zero unmitigated collision or interference hazards; all state conditions positively identified and verified ($C_{\\mathrm{condition}} \\ge C_{\\mathrm{threshold}}$); and successful recovery at the primary base or designated secondary divert recovery site with residual resources strictly satisfying $R_{\\mathrm{reserve}} \\ge \\text{Ratio}_{\\text{reserve\\_min}} \\cdot R_{\\mathrm{capacity}}$.".to_string());
                self.parameter_bindings.entry("LIFECYCLE_BINGO_SAFETY_ACTION".to_string()).or_insert_with(|| "Continuously compute closed-loop dynamic resource state and execute autonomous return-to-base (RTB) or secondary divert routing upon reaching safety thresholds ($R(t) \\le R_{\\mathrm{threshold}}(t)$).".to_string());
                self.parameter_bindings.entry("LIFECYCLE_POST_OP_STATE".to_string()).or_insert_with(|| "Post-operation stationary rest at recovery site with actuator safe locking, cryptographic data zeroization, and diagnostic log offloading".to_string());
            }
        }
    }

    pub fn _derive_subsystem_architecture(&mut self) {
        let sys_id = self
            .parameter_bindings
            .get("SYSTEM_IDENTIFIER")
            .or_else(|| self.inferred_system_identifier.as_ref())
            .cloned()
            .unwrap_or_else(|| "AutonomousSystem".to_string());

        let parts: Vec<String> = self.ast_part_names.iter().cloned().collect();

        // 1. Super-System Architecture (Option 3: Compact Blocks & Vertical Tiers)
        let mut super_lines = vec![
            format!("The **{sys_id}** architecture formalizes the complete system boundary and segment allocations in accordance with IEEE 1362 §5.3, DoDAF OV-2 / SV-1, and ISO/IEC/IEEE 29148:2018."),
            String::new(),
            "```mermaid".to_string(),
            "flowchart TD".to_string(),
            "    subgraph External_Actors[\"External Operating Environment & Actors (IEEE 1362 §5.1)\"]".to_string(),
            "        direction TB".to_string(),
            "        Operator[\"Operator and<br/>Supervisory Entity\"]".to_string(),
            "        Environment[\"External Operating Environment<br/>and Boundary Dynamics\"]".to_string(),
            "        ExternalAuthority[\"External Oversight and<br/>Assurance Authority\"]".to_string(),
            "    end".to_string(),
            String::new(),
            "    subgraph Supervisory_Segment[\"Supervisory & Control Segment (IEEE 1362 §5.3)\"]".to_string(),
            "        direction TB".to_string(),
            "        SupervisoryStation[\"Supervisory Control Station<br/>• PORT_SUP_C2 (INOUT)<br/>• PORT_SUP_DISP (OUT)\"]".to_string(),
            "    end".to_string(),
            String::new(),
            "    subgraph Core_Execution_Segment[\"Core System Execution Segment (DoDAF SV-1)\"]".to_string(),
            "        direction TB".to_string(),
        ];

        let mut node_ids = Vec::new();

        if parts.is_empty() {
            super_lines.push("        subgraph Core_Tier1[\"Tier 1 - Mission Management & Processing\"]".to_string());
            super_lines.push("            direction TB".to_string());
            super_lines.push(format!("            Platform[\"<b>{sys_id} Core Controller</b><br/>• PORT_CTRL_C2 (INOUT)<br/>• PORT_CTRL_CMD (OUT)<br/>• PORT_CTRL_TLM (IN)\"]"));
            super_lines.push("        end".to_string());
            node_ids.push("Platform".to_string());
        } else {
            // Partition parts into vertical tiers of at most 3 parts per tier (Option 3 layout)
            let chunk_size = 3;
            let mut tier_names = Vec::new();
            for (tier_idx, chunk) in parts.chunks(chunk_size).enumerate() {
                let tier_name = format!("Core_Tier{}", tier_idx + 1);
                tier_names.push(tier_name.clone());
                super_lines.push(format!("        subgraph {tier_name}[\"Tier {} - Functional Subsystems\"]", tier_idx + 1));
                super_lines.push("            direction TB".to_string());
                for (p_idx, p) in chunk.iter().enumerate() {
                    let global_idx = tier_idx * chunk_size + p_idx + 1;
                    let clean_p = sanitize_level_1b_operational_text(p);
                    let clean_p_wrapped = wrap_mermaid_label(&clean_p, 28);
                    let node_id = format!("P{global_idx}");
                    node_ids.push(node_id.clone());
                    let p_upper = clean_p.to_uppercase().replace([' ', '_', '.'], "-");
                    super_lines.push(format!(
                        "            {node_id}[\"<b>{clean_p_wrapped}</b><br/>• PORT-{p_upper}-IN (IN)<br/>• PORT-{p_upper}-OUT (OUT)\"]"
                    ));
                }
                super_lines.push("        end".to_string());
            }

            // Rank spacers between internal tiers
            for i in 0..tier_names.len().saturating_sub(1) {
                super_lines.push(format!("        {} ~~~ {}", tier_names[i], tier_names[i + 1]));
            }
        }

        super_lines.push("    end".to_string());
        super_lines.push(String::new());
        super_lines.push("    subgraph Support_Segment[\"Auxiliary & Support Segment (IEEE 1362 §5.3)\"]".to_string());
        super_lines.push("        direction TB".to_string());
        super_lines.push("        SupportEquipment[\"Support and Maintenance<br/>Infrastructure<br/>• PORT_SUPP_PWR (OUT)\"]".to_string());
        super_lines.push("    end".to_string());
        super_lines.push(String::new());

        // Inter-segment vertical spacers (mandatory for top-to-bottom layout)
        super_lines.push("    External_Actors ~~~ Supervisory_Segment".to_string());
        super_lines.push("    Supervisory_Segment ~~~ Core_Execution_Segment".to_string());
        super_lines.push("    Core_Execution_Segment ~~~ Support_Segment".to_string());
        super_lines.push(String::new());

        // Connection links (CONN-01..CONN-N)
        super_lines.push("    %% External Interface Connections".to_string());
        super_lines.push("    Operator -->|\"CONN-01: Supervisory Command & Control\"| SupervisoryStation".to_string());
        super_lines.push("    ExternalAuthority -->|\"CONN-02: Authorization & Safety Consent\"| SupervisoryStation".to_string());

        let target_node = node_ids.first().cloned().unwrap_or_else(|| "Platform".to_string());
        super_lines.push(format!("    SupervisoryStation -->|\"CONN-03: System C2 Datalink [PORT_SUP_C2 -> PORT_CTRL_C2]\"| {target_node}"));
        super_lines.push(format!("    Environment -.->|\"CONN-04: Environmental & Boundary Telemetry\"| {target_node}"));

        let mut conn_idx = 5;
        if node_ids.len() > 1 {
            for i in 0..node_ids.len().saturating_sub(1) {
                super_lines.push(format!(
                    "    {} -->|\"CONN-{:02}: Inter-Subsystem Interface Bus\"| {}",
                    node_ids[i], conn_idx, node_ids[i + 1]
                ));
                conn_idx += 1;
            }
        }
        super_lines.push(format!("    SupportEquipment -.->|\"CONN-{:02}: Auxiliary Power & Diagnostics [PORT_SUPP_PWR]\"| {target_node}", conn_idx));
        super_lines.push("```".to_string());

        let super_arch = super_lines.join("\n");
        self.parameter_bindings
            .insert("SUPER_SYSTEM_ARCHITECTURE".to_string(), super_arch);

        // 2. Subsystem Architecture (100% AST Part Coverage with 6-Bullet Invariant)
        let mut sub_lines = vec![
            format!("All declared SysML AST structural parts for **{sys_id}** are allocated formal operational specifications:"),
            String::new(),
        ];

        if parts.is_empty() {
            sub_lines.push(format!("#### 4.3.1 {sys_id} Core Subsystem Architecture"));
            sub_lines.push(format!("- **Functional Purpose & Scope:** Dedicated operational capability, deterministic state processing, and safety-critical execution for {sys_id}."));
            sub_lines.push("- **Physical & Logical Interface / Port Allocations:** Declared input port `PORT_CTRL_CMD (IN)`, output port `PORT_CTRL_TLM (OUT)`, and high-level system bus interconnects.".to_string());
            sub_lines.push("- **Power, Mass & Resource Envelopes:** Operating electrical power draw: 50.0 W, mass partition budget ($m_{\\mathrm{alloc}}$): 5.0 kg, and thermal operating envelopes: -20 °C to +55 °C.".to_string());
            sub_lines.push("- **Operational Role & Statechart Integration:** Lifecycle mode allocation ($\\Phi_{\\mathrm{lifecycle}}$) and active operational states across nominal and degraded phases.".to_string());
            sub_lines.push("- **Safety Invariants, Containment Interlocks & FMECA Linkage:** Watchdog interlocks, emergency trigger containment bindings (`EMG-01`..`EMG-07`), and safety criticalities.".to_string());
            sub_lines.push(format!("- **SSOT Ground Truth Mapping:** [schema/model.sysml](schema/model.sysml#{sys_id}) (\"OEM Clause Title\") | SysML v2: `part def {sys_id}`"));
            sub_lines.push(String::new());
        } else {
            for (idx, p) in parts.iter().enumerate() {
                let clean_p = sanitize_level_1b_operational_text(p);
                let p_upper = clean_p.to_uppercase().replace([' ', '_', '.'], "-");
                sub_lines.push(format!("#### 4.3.{} {} Subsystem Architecture", idx + 1, clean_p));
                sub_lines.push(format!("- **Functional Purpose & Scope:** Dedicated operational capability, deterministic state processing, and safety-critical execution for {clean_p}."));
                sub_lines.push(format!("- **Physical & Logical Interface / Port Allocations:** Declared input port `PORT-{p_upper}-IN (IN)`, output port `PORT-{p_upper}-OUT (OUT)`, and high-level system bus interconnects."));
                sub_lines.push("- **Power, Mass & Resource Envelopes:** Operating electrical power draw: 50.0 W, mass partition budget ($m_{\\mathrm{alloc}}$): 5.0 kg, and thermal operating envelopes: -20 °C to +55 °C.".to_string());
                sub_lines.push("- **Operational Role & Statechart Integration:** Lifecycle mode allocation ($\\Phi_{\\mathrm{lifecycle}}$) and active operational states across nominal and degraded phases.".to_string());
                sub_lines.push("- **Safety Invariants, Containment Interlocks & FMECA Linkage:** Watchdog interlocks, emergency trigger containment bindings (`EMG-01`..`EMG-07`), and safety criticalities.".to_string());
                sub_lines.push(format!("- **SSOT Ground Truth Mapping:** [schema/model.sysml](schema/model.sysml#{p}) (\"OEM Clause Title\") | SysML v2: `part def {p}`"));
                sub_lines.push(String::new());
            }
        }

        let sub_arch = sub_lines.join("\n");
        self.parameter_bindings
            .insert("SUBSYSTEM_ARCHITECTURE_SECTION".to_string(), sub_arch.clone());
        self.parameter_bindings
            .insert("CONOPS_SECTION_4_SUBSYSTEMS".to_string(), sub_arch);

        // 3. Segment Allocation Matrix (Table 4.2 with SSOT Column)
        let mut seg_lines = vec![
            "| Subsystem Part | Operational Segment | SSOT Ground Truth Source & Citation | Primary Functional Role |".to_string(),
            "| :--- | :--- | :--- | :--- |".to_string(),
        ];
        if parts.is_empty() {
            seg_lines.push(format!("| `{sys_id}` | Primary Operational Segment | [schema/model.sysml](schema/model.sysml#{sys_id}) (\"OEM Clause Title\") | SysML v2: `part def {sys_id}` | Realizes core operational mission functions |"));
        } else {
            for p in &parts {
                let clean_p = sanitize_level_1b_operational_text(p);
                seg_lines.push(format!("| `{clean_p}` | Primary Operational Segment | [schema/model.sysml](schema/model.sysml#{p}) (\"OEM Clause Title\") | SysML v2: `part def {p}` | Realizes core operational mission functions |"));
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
            "USER_CLASSES_AND_STAKEHOLDERS" => "- **UCL-01 System Operator (SO):** Direct supervisory mission management and boundary oversight.\n- **UCL-02 Range Safety Officer (RSO):** Operational containment enforcement and failsafe override authority.\n- **UCL-03 Payload Specialist (PS):** Multi-modal sensor data interpretation and payload stream management.\n- **UCL-04 Maintenance Technician (MT):** O-Level pre-operation inspections, modular LRU swaps, and BIT checks.\n- **UCL-05 Safety Monitor (SM):** Continuous perimeter monitoring, environmental anomaly detection, and safety oversight.".to_string(),
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
            "V_CRUISE_NOMINAL_MPS" => "18.0".to_string(),
            "V_MAX_MPS" => "25.0".to_string(),
            "TOTAL_MTOW_KG" => "50.0".to_string(),
            "PAYLOAD_MAX_KG" => "5.0".to_string(),
            "ENDURANCE_NOMINAL_MIN" => "90.0".to_string(),
            "C2_RANGE_NOMINAL_KM" => "15.0".to_string(),
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
        let has_chute_in_ast = self.has_ast_component("parachute")
            || self.has_ast_component("chute")
            || self.parameter_bindings.contains_key("S_CANOPY");

        if dom == "medical" {
            *current = replace_word_case_insensitive(current, "parachute", "failsafe joint brake");
            *current = current.replace("PARACHUTE", "JOINT_BRAKE");
            *current = replace_altitude_agl(current, "mm");
            *current = replace_astm_f3411(current, "IEC 62304 / ISO 14971");
        } else if dom == "rail" {
            *current = replace_word_case_insensitive(current, "parachute", "pneumatic emergency brake");
            *current = current.replace("PARACHUTE", "EMERGENCY_BRAKE");
            *current = replace_altitude_agl(current, "m");
            *current = replace_astm_f3411(current, "EN 50128 SIL 4");
        } else if dom == "marine" {
            *current = replace_word_case_insensitive(current, "parachute", "positive buoyancy drop-weight");
            *current = current.replace("PARACHUTE", "DROP_WEIGHT");
            *current = replace_altitude_agl(current, "m Depth");
            *current = replace_astm_f3411(current, "DNV-GL-ST-E403");
        } else if dom == "space" {
            *current = replace_word_case_insensitive(current, "parachute", "autonomous de-orbit propulsion");
            *current = current.replace("PARACHUTE", "DEORBIT_THRUSTER");
            *current = replace_altitude_agl(current, "km");
            *current = replace_astm_f3411(current, "ECSS-E-ST-40C");
        } else if self.parameter_bindings.get("LIFECYCLE_TYPE").map(|s| s == "EXPENDABLE_DISPOSAL").unwrap_or(false)
            || self.parameter_bindings.get("IS_EXPENDABLE").map(|s| s == "true").unwrap_or(false)
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

        if self.is_civilian {
            for idx in 1..=6 {
                let target = format!("ROE-0{idx}");
                let repl = format!("SAF-0{idx}");
                *current = replace_word_case_insensitive(current, &target, &repl);
            }
            *current = replace_roe_phrase(current);
            *current = replace_word_case_insensitive(current, "weapons release", "high-consequence actuation");
            *current = replace_word_case_insensitive(current, "collateral damage", "adjacent operational risk");
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
    fn test_default_domain_ontology_pure_schema_driven_no_kinetic_bias() {
        let engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
        assert_eq!(engine.resolve_token("HIGH_CONSEQUENCE_ACTION", None), "high-consequence actuation");
        assert_eq!(engine.resolve_token("COLLATERAL_RISK_PHRASE", None), "adjacent operational risk");
        assert_eq!(engine.resolve_token("INTERLOCK_PREFIX", None), "SAF");
        assert_eq!(engine.resolve_token("TARGET_VERIFICATION_ACRONYM", None), "PCV");
        assert_eq!(engine.resolve_token("INTERLOCK_POLICY_NAME", None), "Operational Safety Interlocks");
    }

    #[test]
    fn test_mass_and_physics_derivation() {
        let mut bindings = HashMap::new();
        bindings.insert("TOTAL_MTOW_KG".to_string(), "100.0".to_string());
        bindings.insert("V_MAX_MPS".to_string(), "20.0".to_string());
        let engine = SysMLParameterBindingEngine::new(Some(bindings), None, None, false, None);
        assert_eq!(engine.resolve_token("TOTAL_MTOW_KG", None), "100.0");
        assert_eq!(engine.resolve_token("V_BOUNDARY_MAX_MPS", None), "20.0");
        assert_eq!(engine.resolve_token("E_BOUNDARY_IMPACT_JOULES", None), "20000.0");
        assert_eq!(engine.resolve_token("E_K_UNMITIGATED_JOULES", None), "20000.0");
        assert_eq!(engine.resolve_token("V_RESIDUAL_MPS", None), "1.5");
        assert_eq!(engine.resolve_token("E_K_MITIGATED_JOULES", None), "112.5");
        let v_term = engine.resolve_token("V_TERMINAL_UNMITIGATED_MPS", None);
        assert_ne!(v_term, "N/A");
        let risk_clauses = engine.resolve_token("DOMAIN_RISK_ASSESSMENT_CLAUSES", None);
        assert!(!risk_clauses.is_empty());

        // Verify purged aviation SORA defaults
        assert_eq!(engine.resolve_token("SORA_GROUND_RISK_CLASS", None), "N/A");
        assert_eq!(engine.resolve_token("SORA_AIR_RISK_CLASS", None), "N/A");
        assert_eq!(engine.resolve_token("SAIL_LEVEL", None), "N/A");
    }

    #[test]
    fn test_super_system_and_subsystem_architecture_synthesis_contract() {
        let mut engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
        engine.ast_part_names.insert("Avionics".to_string());
        engine.ast_part_names.insert("Actuation".to_string());
        engine._derive_subsystem_architecture();

        let super_arch = engine.parameter_bindings.get("SUPER_SYSTEM_ARCHITECTURE").unwrap();
        assert!(super_arch.contains("External_Actors"), "Super-system architecture must include External_Actors");
        assert!(super_arch.contains("Supervisory_Segment"), "Super-system architecture must include Supervisory_Segment");
        assert!(super_arch.contains("Support_Segment"), "Super-system architecture must include Support_Segment");
        assert!(super_arch.contains("~~~"), "Super-system architecture must include vertical rank spacers");
        assert!(super_arch.contains("CONN-"), "Super-system architecture must include CONN- connection labels");
        assert!(super_arch.contains("• PORT-"), "Super-system architecture must include embedded port attributes");

        let sub_arch = engine.parameter_bindings.get("SUBSYSTEM_ARCHITECTURE_SECTION").unwrap();
        assert!(sub_arch.contains("- **Functional Purpose & Scope:**"), "Subsystem architecture must contain Functional Purpose & Scope");
        assert!(sub_arch.contains("- **Physical & Logical Interface / Port Allocations:**"), "Subsystem architecture must contain Physical & Logical Interface");
        assert!(sub_arch.contains("- **Power, Mass & Resource Envelopes:**"), "Subsystem architecture must contain Power, Mass & Resource Envelopes");
        assert!(sub_arch.contains("- **Operational Role & Statechart Integration:**"), "Subsystem architecture must contain Operational Role");
        assert!(sub_arch.contains("- **Safety Invariants, Containment Interlocks & FMECA Linkage:**"), "Subsystem architecture must contain Safety Invariants");
        assert!(sub_arch.contains("- **SSOT Ground Truth Mapping:**"), "Subsystem architecture must contain SSOT Ground Truth Mapping bullet");
        assert!(sub_arch.contains("SysML v2: `part def Avionics`"), "Subsystem architecture must bind AST part def node");

        let seg_matrix = engine.parameter_bindings.get("SEGMENT_ALLOCATION_MATRIX").unwrap();
        assert!(seg_matrix.contains("SSOT Ground Truth Source & Citation"), "Segment allocation matrix must contain SSOT Ground Truth column");
        assert!(seg_matrix.contains("SysML v2: `part def Avionics`"), "Segment allocation matrix must bind AST part def node");
    }

    #[test]
    fn test_issue_248_pure_schema_driven_containment_and_zero_fallback_recovery() {
        // 1. Expendable / Zero-Recovery platform must NOT have hardcoded recovery base or RTB transit
        let mut exp_bindings = HashMap::new();
        exp_bindings.insert("IS_EXPENDABLE".to_string(), "true".to_string());
        let exp_engine = SysMLParameterBindingEngine::new(Some(exp_bindings), None, None, false, None);

        assert_ne!(
            exp_engine.resolve_token("PRIMARY_TERMINAL_TARGET", None),
            "Primary Recovery Base",
            "Expendable system must not default to Primary Recovery Base"
        );
        assert_ne!(
            exp_engine.resolve_token("PRIMARY_RECOVERY_FACILITY", None),
            "Primary Recovery Base",
            "Expendable system must not default to Primary Recovery Base facility"
        );
        assert_ne!(
            exp_engine.resolve_token("LIFECYCLE_TRANSIT_MODE", None),
            "Autonomous_RTB_Transit",
            "Expendable system must not default to Autonomous_RTB_Transit"
        );
        assert_ne!(
            exp_engine.resolve_token("LIFECYCLE_FAILSAFE_SEQUENCE", None),
            "N/A",
            "Lifecycle failsafe sequence must be derived"
        );
        assert_ne!(
            exp_engine.resolve_token("CONTAINMENT_MITIGATION_TYPE", None),
            "N/A",
            "Containment mitigation type must be derived"
        );

        // 2. Generic platform with ZERO parachute in AST must decontaminate ungrounded parachute references
        let generic_engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
        let input_text = "Emergency containment initiates parachute deployment for recovery.";
        let substituted = generic_engine.substitute(input_text);
        assert!(
            !substituted.to_lowercase().contains("parachute"),
            "System with zero parachute in AST must not retain parachute: got '{substituted}'"
        );

        // 3. User-supplied parachute AST parameters must be dynamically calculated (not hardcoded 1.5 m/s)
        let mut chute_bindings = HashMap::new();
        chute_bindings.insert("TOTAL_MTOW_KG".to_string(), "200.0".to_string());
        chute_bindings.insert("AIR_DENSITY_KGM3".to_string(), "1.2".to_string());
        chute_bindings.insert("S_CANOPY".to_string(), "50.0".to_string());
        chute_bindings.insert("PARACHUTE_DRAG_COEFFICIENT".to_string(), "1.5".to_string());
        let mut chute_engine = SysMLParameterBindingEngine::new(Some(chute_bindings), None, None, false, None);
        chute_engine.ast_part_names.insert("ParachuteRecoveryUnit".to_string());
        chute_engine._derive_quadratic_physics();

        let v_mit: f64 = chute_engine
            .resolve_token("V_TERMINAL_MITIGATED_MPS", None)
            .parse()
            .expect("V_TERMINAL_MITIGATED_MPS must be numeric");
        // v_calc = sqrt(2 * 200 * 9.80665 / (1.2 * 50 * 1.5)) = sqrt(3922.66 / 90) = 6.60 m/s
        assert!(
            (v_mit - 6.60).abs() < 0.1,
            "Mitigated terminal velocity must be dynamically derived from AST parameters: got {v_mit}"
        );
    }
}


