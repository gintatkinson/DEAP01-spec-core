//! SysML AST Parameter Binding Engine and template token resolution.

mod aliases;
mod arch_synthesis;
mod handoff_synthesis;
mod ingest;
mod lifecycle_ontology;
mod physics_budgets;
mod schema_extractor;
mod substitution;
mod sysml_extractor;
mod sysml_lexer;
mod text_replace;

#[cfg(test)]
mod tests_core;
#[cfg(test)]
mod tests_schema;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub use schema_extractor::{
    extract_subsystem_parts, extract_subsystem_parts_from_interface_schema,
    extract_subsystem_parts_from_json, extract_subsystem_parts_from_markdown,
    extract_subsystem_parts_from_yaml,
};
pub use sysml_extractor::{extract_subsystem_parts_from_sysml, parse_subsystem_ports_from_body};
pub use sysml_lexer::find_line_number;

/// Subsystem port representing a physical or logical interface boundary.
/// Inputs: port metadata fields | Outputs: `SubsystemPort` | Invariants: preserves source provenance
///Errors: none | Complexity: O(1)
/// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SUBSYSTEMPORT] | SysML v2: PartDef(SubsystemPort)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubsystemPort {
    pub name: String,
    pub direction: String,
    pub type_name: String,
    pub doc: String,
    pub source_file: String,
    pub source_line: Option<usize>,
    pub epistemic_tier: String,
}

impl Default for SubsystemPort {
    /// Returns a default `SubsystemPort` with `"INOUT"` direction and `"[TIER-1: OEM]"` tier.
    /// Inputs: none | Outputs: `Self` | Invariants: default tier is `[TIER-1: OEM]`
    ///Errors: none | Complexity: O(1)
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SUBSYSTEMPORT-DEFAULT] | SysML v2: ActionDef(SubsystemPort::default)
    fn default() -> Self {
        Self {
            name: String::new(),
            direction: "INOUT".to_string(),
            type_name: "Port".to_string(),
            doc: String::new(),
            source_file: String::new(),
            source_line: None,
            epistemic_tier: "[TIER-1: OEM]".to_string(),
        }
    }
}

impl SubsystemPort {
    /// Constructs a `SubsystemPort` with the given port `name`.
    /// Inputs: `name: impl Into<String>` | Outputs: `Self` | Invariants: default direction and tier
    ///Errors: none | Complexity: O(1)
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SUBSYSTEMPORT-NEW] | SysML v2: ActionDef(SubsystemPort::new)
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
}

/// Canonical model for a subsystem part definition with OEM provenance.
/// Inputs: part metadata fields | Outputs: `SubsystemPart` | Invariants: preserves ports, attributes, and source provenance
///Errors: none | Complexity: O(1)
/// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SUBSYSTEMPART] | SysML v2: PartDef(SubsystemPart)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubsystemPart {
    pub name: String,
    pub doc: String,
    pub ports: Vec<SubsystemPort>,
    pub mass_kg: Option<f64>,
    pub power_w: Option<f64>,
    pub actions: Vec<String>,
    pub attributes: BTreeMap<String, String>,
    pub constraints: Vec<String>,
    pub source_file: String,
    pub source_line: Option<usize>,
    pub epistemic_tier: String,
}

impl Default for SubsystemPart {
    /// Returns a default `SubsystemPart` with `"[TIER-1: OEM]"` epistemic tier.
    /// Inputs: none | Outputs: `Self` | Invariants: empty ports and attributes
    ///Errors: none | Complexity: O(1)
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SUBSYSTEMPART-DEFAULT] | SysML v2: ActionDef(SubsystemPart::default)
    fn default() -> Self {
        Self {
            name: String::new(),
            doc: String::new(),
            ports: Vec::new(),
            mass_kg: None,
            power_w: None,
            actions: Vec::new(),
            attributes: BTreeMap::new(),
            constraints: Vec::new(),
            source_file: String::new(),
            source_line: None,
            epistemic_tier: "[TIER-1: OEM]".to_string(),
        }
    }
}

impl SubsystemPart {
    /// Constructs a `SubsystemPart` with the given part `name`.
    /// Inputs: `name: impl Into<String>` | Outputs: `Self` | Invariants: default tier is `[TIER-1: OEM]`
    ///Errors: none | Complexity: O(1)
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SUBSYSTEMPART-NEW] | SysML v2: ActionDef(SubsystemPart::new)
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
}

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
    ("MASS_FRACTION_STRUCTURE_PCT", "30.0"),
    ("POWER_NOMINAL_STRUCTURE_W", "0.0"),
    ("POWER_PEAK_STRUCTURE_W", "0.0"),
    ("MASS_FRACTION_CONTROLLER_PCT", "15.0"),
    ("POWER_NOMINAL_CONTROLLER_W", "25.0"),
    ("POWER_PEAK_CONTROLLER_W", "50.0"),
    ("SPATIAL_LIMIT_UNIT", "m"),
];

/// Formal lifecycle archetypes for cyber-physical mission systems.
/// Inputs: lifecycle variant | Outputs: `LifecycleType` | Invariants: 5 canonical lifecycle archetypes
///Errors: none | Complexity: O(1)
/// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-LIFECYCLETYPE] | SysML v2: PartDef(LifecycleType)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleType {
    ReusableRecovery,
    ExpendableDisposal,
    ContinuousStationary,
    PersistentOrbital,
    TrackBoundGuided,
}

impl LifecycleType {
    /// Returns the canonical uppercase identifier string for this `LifecycleType`.
    /// Inputs: `&self` | Outputs: `&'static str` | Invariants: non-empty static string
    ///Errors: none | Complexity: O(1)
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-LIFECYCLETYPE-AS-STR] | SysML v2: ActionDef(LifecycleType::as_str)
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
/// Inputs: workspace paths and parameter dictionaries | Outputs: `SysMLParameterBindingEngine` | Invariants: deterministic BTreeMap/BTreeSet state
///Errors: none | Complexity: O(N) over bound parameters
/// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE] | SysML v2: PartDef(SysMLParameterBindingEngine)
#[derive(Debug, Clone)]
pub struct SysMLParameterBindingEngine {
    pub workspace_dir: PathBuf,
    pub parameter_bindings: BTreeMap<String, String>,
    pub explicit_keys: BTreeSet<String>,
    pub inferred_system_identifier: Option<String>,
    pub detected_domain: String,
    pub is_non_aircraft: bool,
    pub is_civilian: bool,
    pub ast_part_names: BTreeSet<String>,
    pub ast_parts: Vec<SubsystemPart>,
}

impl SysMLParameterBindingEngine {
    /// Constructs and initializes a `SysMLParameterBindingEngine` with optional auto-discovery and parameter overrides.
    /// Inputs: `parameter_values, config_path, workspace_dir, auto_detect, domain` | Outputs: `Self` | Invariants: derives all ontology, physics, and architecture sections
    ///Errors: never panics | Complexity: O(N) over ingested workspace files
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-NEW] | SysML v2: ActionDef(SysMLParameterBindingEngine::new)
    pub fn new(
        parameter_values: Option<BTreeMap<String, String>>,
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
            parameter_bindings: BTreeMap::new(),
            explicit_keys: BTreeSet::new(),
            inferred_system_identifier: None,
            detected_domain: domain.unwrap_or("generic").to_string(),
            is_non_aircraft: false,
            is_civilian: false,
            ast_part_names: BTreeSet::new(),
            ast_parts: Vec::new(),
        };

        for &(k, v) in DEFAULT_CONOPS_PARAMS {
            engine
                .parameter_bindings
                .insert(k.to_string(), v.to_string());
        }

        if let Some(dom) = domain {
            engine
                .parameter_bindings
                .insert("DOMAIN_TYPE".to_string(), dom.to_string());
            engine
                .parameter_bindings
                .insert("TARGET_DOMAIN".to_string(), dom.to_string());
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
                engine
                    .parameter_bindings
                    .insert(k.to_uppercase(), v.clone());
                engine._map_semantic_aliases(&k, &v);
            }
        }

        engine.detected_domain = engine._detect_domain_type();
        let dom_clone = engine.detected_domain.clone();
        engine
            .parameter_bindings
            .insert("DETECTED_DOMAIN".to_string(), dom_clone.clone());
        engine
            .parameter_bindings
            .insert("DOMAIN_TYPE".to_string(), dom_clone);
        engine.derive_all_bindings();

        engine
    }

    /// Executes the full sequence of ontology, physics, and architecture derivations (`[REQ-ASSEMBLE-CONOPS]`).
    fn derive_all_bindings(&mut self) {
        self._derive_operational_intent();
        self._derive_mass_budgets();
        self._derive_domain_ontology();
        self._derive_lifecycle_contract();
        self._derive_quadratic_physics();
        self._derive_energy_budgets();
        self._derive_domain_regulatory_standards();
        self._derive_domain_risk_assessment();
        self._derive_subsystem_architecture();
        self.derive_master_physical_limits_from_ast();
    }

    /// Checks whether any ingested AST part name matches or contains `name` case-insensitively.
    /// Inputs: `name: &str` | Outputs: `bool` | Invariants: case-insensitive substring match
    ///Errors: never fails | Complexity: O(P) over `ast_part_names`
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-HAS-AST-COMPONENT] | SysML v2: ActionDef(SysMLParameterBindingEngine::has_ast_component)
    pub fn has_ast_component(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        self.ast_part_names.iter().any(|p| {
            let p_lower = p.to_lowercase();
            p_lower == name_lower || p_lower.contains(&name_lower)
        })
    }
}

/// Substitutes `{{TOKEN}}` placeholders in `text` using the provided `engine` or a default instance.
/// Inputs: `text: &str, engine: Option<&SysMLParameterBindingEngine>` | Outputs: `String` | Invariants: delegates to `SysMLParameterBindingEngine::substitute`
///Errors: never panics | Complexity: O(N) over `text` length
/// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-BIND-PARAMETERS] | SysML v2: ActionDef(bind_parameters)
pub fn bind_parameters(text: &str, engine: Option<&SysMLParameterBindingEngine>) -> String {
    if let Some(eng) = engine {
        eng.substitute(text)
    } else {
        let default_eng = SysMLParameterBindingEngine::new(None, None, None, false, None);
        default_eng.substitute(text)
    }
}
