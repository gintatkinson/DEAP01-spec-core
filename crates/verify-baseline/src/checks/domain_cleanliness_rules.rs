//! Closed M2 metamodel allowlist constants and lexical typing predicates for Check 19.
//! Realises: [REQ-CHECKS-DOMAIN-CLEANLINESS-GATE-IS-ALLOWED-M2-TYPE]

/// Closed set of abstract M2 metamodel entity types conforming to rules/sysml-ssot-completeness.md.
pub const ALLOWED_M2_METAMODEL_TYPES: &[&str] = &[
    // Core structural elements
    "Component",
    "Class",
    "Port",
    "Interface",
    "Statechart",
    "Constraint",
    "Signal",
    "Event",
    "AcceptanceCriterion",
    "Scenario",
    "TraceLink",
    // SysML v2 & KerML Definition types
    "Package",
    "PackageDefinition",
    "PartDefinition",
    "PortDefinition",
    "StateDefinition",
    "ItemDefinition",
    "ActionDefinition",
    "RequirementDefinition",
    "UseCaseDefinition",
    "ConstraintDefinition",
    "AttributeDefinition",
    "ConnectionDefinition",
    "AllocationDefinition",
    "ViewDefinition",
    "ViewpointDefinition",
    "ActorDefinition",
    "NamespaceDefinition",
    "ElementDefinition",
    "FeatureDefinition",
    "Classifier",
    // Actor and Role types
    "HumanOperator",
    "SystemController",
    "SafetyInterlock",
    "PhysicalActuator",
    "Sensor",
    "SystemUnderStudy",
    "ExternalSystem",
    "OperatorConsole",
    // Canonical M2 elements
    "Actor",
    "Part",
    "Item",
    "Action",
    "State",
    "Requirement",
    "UseCase",
    "Attribute",
    "Connection",
    "Allocation",
    "Transition",
    "Guard",
    "Trigger",
    "Effect",
    // AST, Schema, Parser & Model primitives
    "Namespace",
    "Object",
    "Array",
    "String",
    "Number",
    "Boolean",
    "Integer",
    "Dict",
    "List",
    "Null",
    "Primitive",
    "Type",
    "Definition",
    "Block",
    "Node",
    "Root",
    "Value",
    "Field",
    "Member",
    "Document",
    // Logical UI (LUI / LUMI) Canonical Display, Container & Widget primitives
    "Widget",
    "Container",
    "Layout",
    "View",
    "SidebarLayout",
    "HierarchyTree",
    "ResizableSplitter",
    "TopologyMap",
    "DensityTable",
    "TabbedContainer",
    "SplitterContainer",
    "Panel",
    "Section",
    "Tab",
    "Tree",
    "Table",
    "Map",
    "Chart",
    "Form",
    "Button",
    "Input",
    "Dialog",
    "Modal",
];

/// Known keys representing metamodel or entity types in ASTs, dictionaries, and configurations.
pub const METAMODEL_TYPE_KEYS: &[&str] = &[
    "metamodel_type",
    "entity_type",
    "m2_type",
    "definition_type",
    "ast_type",
    "node_type",
    "element_type",
];

/// Checks if a type name conforms to the closed M2 metamodel allowlist or carries a `meta_` prefix.
/// Realises: [REQ-CHECKS-DOMAIN-CLEANLINESS-GATE-IS-ALLOWED-M2-TYPE]
pub fn is_allowed_m2_type(type_name: &str) -> bool {
    let cleaned = type_name.trim();
    if cleaned.is_empty() {
        return false;
    }
    let lower = cleaned.to_lowercase();
    if lower.starts_with("meta_") || lower.starts_with("meta") {
        return true;
    }
    let norm = lower.replace(['_', '-'], "");
    ALLOWED_M2_METAMODEL_TYPES
        .iter()
        .any(|t| t.to_lowercase().replace(['_', '-'], "") == norm)
}

/// Extracts a declared identifier name from a `const` or `static` line in native Rust.
/// Realises: [REQ-CHECKS-DOMAIN-CLEANLINESS-GATE-EXTRACT-RUST-CONST-OR-STATIC-IDENT]
pub(crate) fn extract_rust_const_or_static_ident(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
        return None;
    }

    let mut rest = trimmed;
    if rest.starts_with("pub") {
        rest = rest.get(3..)?.trim_start();
        if rest.starts_with('(') {
            if let Some(close) = rest.find(')') {
                rest = rest.get(close + 1..)?.trim_start();
            }
        }
    }

    let is_decl = if let Some(after_const) = rest.strip_prefix("const ") {
        rest = after_const.trim_start();
        true
    } else if let Some(after_static) = rest.strip_prefix("static ") {
        rest = after_static.trim_start();
        if let Some(after_mut) = rest.strip_prefix("mut ") {
            rest = after_mut.trim_start();
        }
        true
    } else {
        false
    };

    if !is_decl {
        return None;
    }

    let ident_end = rest
        .find(|c: char| c == ':' || c == '=' || c.is_whitespace())
        .unwrap_or(rest.len());
    let ident = rest.get(..ident_end)?.trim();
    if !ident.is_empty() && ident.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        Some(ident)
    } else {
        None
    }
}

/// Detects unvalidated M1 domain instance dictionary or constant identifier names.
/// Realises: [REQ-CHECKS-DOMAIN-CLEANLINESS-GATE-IS-M1-DOMAIN-DICT-NAME]
pub(crate) fn is_m1_domain_dict_name(ident: &str) -> bool {
    let lower = ident.to_lowercase();
    if lower.starts_with("m1_")
        || lower.contains("m1_domain")
        || lower.contains("domain_entities")
        || lower.contains("domain_instances")
        || lower.contains("domain_models")
        || lower.contains("domain_objects")
        || lower.contains("domain_dicts")
        || lower.contains("domain_types")
        || lower.contains("concrete_entities")
        || lower.contains("concrete_instances")
        || lower.contains("concrete_models")
        || lower.contains("concrete_specs")
        || lower.contains("concrete_objects")
        || lower.contains("concrete_dicts")
        || lower.contains("concrete_types")
    {
        return true;
    }

    let domain_nouns = [
        "uav", "aircraft", "vehicle", "device", "patient", "car", "robot",
    ];
    let prefixes = ["sample_", "mock_", "concrete_"];
    for p in &prefixes {
        if let Some(after) = lower.strip_prefix(p) {
            for noun in &domain_nouns {
                if after.starts_with(noun) {
                    return true;
                }
            }
        }
    }

    let concrete_domain_concepts = [
        "flight_envelope",
        "sensor_calibration",
        "payload_config",
        "flight_dynamics",
        "propulsion_map",
        "avionics_config",
        "mission_profile_map",
        "vehicle_specs",
        "aircraft_config",
        "patient_vitals",
        "diagnostic_thresholds",
    ];
    for concept in &concrete_domain_concepts {
        if lower.contains(concept) {
            return true;
        }
    }

    false
}

/// Detects static hardcoded parameter dictionary identifier names.
/// Realises: [REQ-CHECKS-DOMAIN-CLEANLINESS-GATE-IS-STATIC-PARAM-DICT-NAME]
pub(crate) fn is_static_param_dict_name(ident: &str) -> bool {
    let lower = ident.to_lowercase();
    lower.contains("ground_truth")
        || lower.contains("groundtruth")
        || lower.contains("expected_specs")
        || lower.contains("expected_params")
        || lower.contains("domain_specs")
        || lower.contains("domain_params")
        || lower.contains("sysml_params")
        || lower.contains("mandated_specs")
        || lower.contains("mandated_params")
        || lower.contains("physical_limits")
        || lower.contains("hardware_registry")
        || lower.contains("hardcoded_specs")
        || lower.contains("hardcoded_params")
        || lower.contains("benchmark_specs")
        || lower.contains("system_specs")
}

/// Checks if a line contains an unvalidated M1 domain instance entity/type assignment.
/// Realises: [REQ-CHECKS-DOMAIN-CLEANLINESS-GATE-CHECK-LINE-FOR-METAMODEL-TYPING-VIOLATION]
pub(crate) fn check_line_for_metamodel_typing_violation(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
        return None;
    }
    let lower = trimmed.to_lowercase();
    for &key in METAMODEL_TYPE_KEYS {
        let search_patterns = [
            format!("\"{key}\""),
            format!("{key}:"),
            format!("{key} ="),
        ];
        for pat in &search_patterns {
            if let Some(pos) = lower.find(pat.as_str()) {
                let rest = trimmed.get(pos + pat.len()..)?;
                if let Some(first_quote) = rest.find('"') {
                    let after_first = rest.get(first_quote + 1..)?;
                    if let Some(second_quote) = after_first.find('"') {
                        let candidate_val = after_first.get(..second_quote)?;
                        if !candidate_val.is_empty()
                            && (candidate_val.starts_with(|c: char| c.is_ascii_uppercase())
                                || candidate_val.starts_with("meta_")
                                || candidate_val.starts_with("Meta"))
                            && candidate_val
                                .chars()
                                .all(|c| c.is_ascii_alphanumeric() || c == '_')
                            && !is_allowed_m2_type(candidate_val)
                        {
                            return Some(candidate_val.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}
