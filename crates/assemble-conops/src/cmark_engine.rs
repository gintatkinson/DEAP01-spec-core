//! `SubsystemPort`, `SubsystemPart`, `LifecycleType`, and `SysMLParameterBindingEngine` types (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// [DOC-TIER: A] Subsystem port representing a physical or logical interface boundary.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SUBSYSTEMPORT] [FLS: fls_conops_cmark]
/// Contract: Stores port name, direction (`IN`, `OUT`, `INOUT`), type, docstring, and OEM file/line provenance.
/// Bounds & Faults: O(1) struct storage; defaults to `"INOUT"` and `"[TIER-1: OEM]"`.
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
    /// Returns a default `SubsystemPort` with `"INOUT"` direction and `"[TIER-1: OEM]"` tier (`[REQ-ASSEMBLE-CONOPS-PARAMS-SUBSYSTEMPORT-DEFAULT]`, `[FLS: fls_conops_cmark]`).
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
    /// [DOC-TIER: A] Constructs a `SubsystemPort` with the given port `name`.
    /// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SUBSYSTEMPORT-NEW] [FLS: fls_conops_cmark]
    /// Contract: Initializes `SubsystemPort` with `name` and default `"INOUT"` / `"[TIER-1: OEM]"` fields.
    /// Bounds & Faults: O(1) allocation; infallible.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
}

/// [DOC-TIER: A] Canonical model for a subsystem part definition with OEM provenance.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SUBSYSTEMPART] [FLS: fls_conops_cmark]
/// Contract: Captures part name, ports, mass/power budgets, actions, attributes, constraints, and source provenance.
/// Bounds & Faults: O(P + A) storage over ports and attributes; defaults to `"[TIER-1: OEM]"`.
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
    /// Returns a default `SubsystemPart` with `"[TIER-1: OEM]"` epistemic tier (`[REQ-ASSEMBLE-CONOPS-PARAMS-SUBSYSTEMPART-DEFAULT]`, `[FLS: fls_conops_cmark]`).
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
    /// [DOC-TIER: A] Constructs a `SubsystemPart` with the given part `name`.
    /// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SUBSYSTEMPART-NEW] [FLS: fls_conops_cmark]
    /// Contract: Initializes `SubsystemPart` with `name` and empty port/attribute collections.
    /// Bounds & Faults: O(1) allocation; infallible.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
}

/// Canonical default ConOps parameters dictionary (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
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

/// [DOC-TIER: A] Formal lifecycle archetypes for cyber-physical mission systems.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-LIFECYCLETYPE] [FLS: fls_conops_cmark]
/// Contract: Enumerates the 5 canonical system lifecycle recovery/disposal archetypes.
/// Bounds & Faults: O(1) copy enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleType {
    ReusableRecovery,
    ExpendableDisposal,
    ContinuousStationary,
    PersistentOrbital,
    TrackBoundGuided,
}

impl LifecycleType {
    /// [DOC-TIER: A] Returns the canonical uppercase identifier string for this `LifecycleType`.
    /// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-LIFECYCLETYPE-AS-STR] [FLS: fls_conops_cmark]
    /// Contract: Maps each `LifecycleType` variant to its static uppercase key string.
    /// Bounds & Faults: O(1) time and space; infallible.
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

/// [DOC-TIER: A] Automated SysML AST Parameter Binding Engine.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE] [FLS: fls_conops_cmark]
/// Contract: Maintains deterministic `BTreeMap`/`BTreeSet` parameter bindings and extracted AST `SubsystemPart` records.
/// Bounds & Faults: O(N) memory over bound parameters and AST parts.
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
    /// [DOC-TIER: A] Constructs and initializes a `SysMLParameterBindingEngine` with optional auto-discovery and overrides.
    /// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-NEW] [FLS: fls_conops_cmark]
    /// Contract: Loads default parameters, ingests workspace/config schemas when enabled, and derives ontology/physics/architecture bindings.
    /// Bounds & Faults: O(N) over ingested workspace files; never panics.
    pub fn new(
        parameter_values: Option<BTreeMap<String, String>>,
        config_path: Option<&Path>,
        workspace_dir: Option<&Path>,
        auto_detect: bool,
        domain: Option<&str>,
    ) -> Self {
        let ws = workspace_dir
            .map(Path::to_path_buf)
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
        engine.derive_master_physical_limits_from_ast();
        engine
    }

    /// [DOC-TIER: A] Checks whether any ingested AST part name matches or contains `name` case-insensitively.
    /// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-HAS-AST-COMPONENT] [FLS: fls_conops_cmark]
    /// Contract: Returns `true` if any entry in `ast_part_names` equals or contains `name` case-insensitively.
    /// Bounds & Faults: O(P) over `ast_part_names`; infallible.
    pub fn has_ast_component(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        self.ast_part_names.iter().any(|p| {
            let p_lower = p.to_lowercase();
            p_lower == name_lower || p_lower.contains(&name_lower)
        })
    }
}

/// [DOC-TIER: A] Substitutes `{{TOKEN}}` placeholders in `text` using the provided `engine` or a default instance.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-BIND-PARAMETERS] [FLS: fls_conops_cmark]
/// Contract: Delegates placeholder expansion and Level 1B sanitization to `SysMLParameterBindingEngine::substitute`.
/// Bounds & Faults: O(N) over `text.len()`; infallible.
pub fn bind_parameters(text: &str, engine: Option<&SysMLParameterBindingEngine>) -> String {
    if let Some(eng) = engine {
        eng.substitute(text)
    } else {
        SysMLParameterBindingEngine::new(None, None, None, false, None).substitute(text)
    }
}
