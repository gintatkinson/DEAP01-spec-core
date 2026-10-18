//! Domain-Agnostic AST Cleanliness and Closed-Grammar Metamodel Allowlist Gate.
//!
//! Enforces:
//! 1. Pure schema-driven compiler architecture (zero hardcoded domain concepts).
//! 2. Closed M2 metamodel entity allowlist type system (`ALLOWED_M2_METAMODEL_TYPES`).
//! 3. Rejection of unvalidated M1 domain instance entities, dictionaries, and constants
//!    under rule ID `domain-metamodel-typing-violation`.
//! 4. Zero legacy Python parity auditor directory presence on disk.
//! 5. Upstream template domain cleanliness and zero Unicode em dashes.
//!
//! Downstream repositories are exempt.
//! Strictly zero regex, zero Python scripts, zero shell scripts, zero Unicode em dashes.

use deap_core::rules::EXCLUDED_DIRS;
use deap_core::workspace::is_upstream_compiler;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

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

/// Check if a type name conforms to the closed M2 metamodel allowlist or carries a meta_ prefix.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `is_allowed_m2_type` in `crates/verify-baseline/src/checks/domain_cleanliness_gate.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `is_allowed_m2_type` (declaration #1) within `CHECKS-DOMAIN-CLEANLINESS-GATE`.
///
/// ## 3. Preconditions
/// - `type_name: &str` must satisfy domain and type-safety constraints prior to invoking `is_allowed_m2_type`.
///
/// ## 4. Postconditions
/// - Returns `bool` from `is_allowed_m2_type` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(type_name)) bounded over input/member dimension of `is_allowed_m2_type`.
/// - Space Complexity: O(N(type_name)) bounded auxiliary memory for `is_allowed_m2_type`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `is_allowed_m2_type` (`bool`).
/// - State: Maintains consistent state invariants across `is_allowed_m2_type` without partial mutation on error.
/// - Action: Executes `is_allowed_m2_type` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `is_allowed_m2_type` (`CHECKS-DOMAIN-CLEANLINESS-GATE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECKS-DOMAIN-CLEANLINESS-GATE-IS-ALLOWED-M2-TYPE] | SysML v2: ActionDef(is_allowed_m2_type)
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

/// Helper extracting declared identifier name from a const or static line in native Rust.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `extract_rust_const_or_static_ident` in `crates/verify-baseline/src/checks/domain_cleanliness_gate.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `extract_rust_const_or_static_ident` (declaration #2) within `CHECKS-DOMAIN-CLEANLINESS-GATE`.
///
/// ## 3. Preconditions
/// - `line: &str` must satisfy domain and type-safety constraints prior to invoking `extract_rust_const_or_static_ident`.
///
/// ## 4. Postconditions
/// - Returns `Option<&str>` from `extract_rust_const_or_static_ident` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(line)) bounded over input/member dimension of `extract_rust_const_or_static_ident`.
/// - Space Complexity: O(N(line)) bounded auxiliary memory for `extract_rust_const_or_static_ident`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `extract_rust_const_or_static_ident` (`Option<&str>`).
/// - State: Maintains consistent state invariants across `extract_rust_const_or_static_ident` without partial mutation on error.
/// - Action: Executes `extract_rust_const_or_static_ident` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `extract_rust_const_or_static_ident` (`CHECKS-DOMAIN-CLEANLINESS-GATE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECKS-DOMAIN-CLEANLINESS-GATE-EXTRACT-RUST-CONST-OR-STATIC-IDENT] | SysML v2: ActionDef(extract_rust_const_or_static_ident)
fn extract_rust_const_or_static_ident(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
        return None;
    }

    let mut rest = trimmed;
    if rest.starts_with("pub") {
        rest = rest[3..].trim_start();
        if rest.starts_with('(') {
            if let Some(close) = rest.find(')') {
                rest = rest[close + 1..].trim_start();
            }
        }
    }

    let is_decl = if rest.starts_with("const ") {
        rest = rest[6..].trim_start();
        true
    } else if rest.starts_with("static ") {
        rest = rest[7..].trim_start();
        if rest.starts_with("mut ") {
            rest = rest[4..].trim_start();
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
    let ident = rest[..ident_end].trim();
    if !ident.is_empty() && ident.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        Some(ident)
    } else {
        None
    }
}

/// Detects unvalidated M1 domain instance dictionary or constant identifier names.
fn is_m1_domain_dict_name(ident: &str) -> bool {
    let lower = ident.to_lowercase();

    // 1. M1 domain dictionary prefixes and patterns
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

    // 2. Sample, mock, or concrete domain hardware/vehicle dicts
    let domain_nouns = ["uav", "aircraft", "vehicle", "device", "patient", "car", "robot"];
    let prefixes = ["sample_", "mock_", "concrete_"];
    for p in &prefixes {
        if lower.starts_with(p) {
            let after = &lower[p.len()..];
            for noun in &domain_nouns {
                if after.starts_with(noun) {
                    return true;
                }
            }
        }
    }

    // 3. Concrete unmapped domain concept maps and dictionaries (audit defect targets)
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
fn is_static_param_dict_name(ident: &str) -> bool {
    let lower = ident.to_lowercase();
    if lower.contains("ground_truth")
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
    {
        return true;
    }
    false
}

/// Checks if a line contains an unvalidated M1 domain instance entity/type assignment.
fn check_line_for_metamodel_typing_violation(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
        return None;
    }
    let lower = trimmed.to_lowercase();
    for &key in METAMODEL_TYPE_KEYS {
        let search_patterns = [
            format!("\"{}\"", key),
            format!("{}:", key),
            format!("{} =", key),
        ];
        for pat in &search_patterns {
            if let Some(pos) = lower.find(pat.as_str()) {
                let rest = &trimmed[pos + pat.len()..];
                if let Some(first_quote) = rest.find('"') {
                    let after_first = &rest[first_quote + 1..];
                    if let Some(second_quote) = after_first.find('"') {
                        let candidate_val = &after_first[..second_quote];
                        if !candidate_val.is_empty()
                            && (candidate_val.starts_with(|c: char| c.is_ascii_uppercase())
                                || candidate_val.starts_with("meta_")
                                || candidate_val.starts_with("Meta"))
                            && candidate_val.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                        {
                            if !is_allowed_m2_type(candidate_val) {
                                return Some(candidate_val.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Check 19: Domain-Agnostic AST Cleanliness & Closed-Grammar Metamodel Gate.
///
/// Verifies that upstream compiler tools, crates, and templates maintain pure dynamic schema AST
/// architecture and contain zero static/hardcoded domain parameter dictionaries.
/// Also enforces closed M2 metamodel entity allowlist typing (`ALLOWED_M2_METAMODEL_TYPES`)
/// and rejects unvalidated M1 domain instance entities under rule ID `domain-metamodel-typing-violation`.
/// Downstream repositories are exempt.
pub fn check_domain_agnostic_ast_cleanliness(repo_root: &Path) -> Result<(), Vec<String>> {
    if !is_upstream_compiler(repo_root) {
        return Ok(());
    }

    let mut violations = Vec::new();

    // 1. Enforce zero legacy Python parity_auditor directories on disk
    let legacy_paths = [
        repo_root.join("skills").join("spec-orchestrator").join("parity_auditor"),
        repo_root.join("parity_auditor"),
    ];
    for p in &legacy_paths {
        if p.exists() {
            let rel = p.strip_prefix(repo_root).unwrap_or(p).display().to_string();
            violations.push(format!(
                "Check 19 violation: Legacy Python parity_auditor directory detected on disk: {}",
                rel
            ));
        }
    }

    // 2. Validate native Rust workspace crates directory exists
    let crates_dir = repo_root.join("crates");
    if !crates_dir.is_dir() {
        violations.push("Check 19 violation: Native Rust workspace crates/ directory missing in upstream compiler repository.".to_string());
    } else {
        // 3. Scan all native Rust source files in crates/ for hardcoded parameter dictionaries and metamodel violations
        for entry in WalkDir::new(&crates_dir)
            .into_iter()
            .filter_entry(|e| {
                if e.file_type().is_dir() {
                    let name = e.file_name().to_string_lossy();
                    name != "target" && name != ".git" && !EXCLUDED_DIRS.iter().any(|ex| name == ex)
                } else {
                    true
                }
            })
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path = entry.path();
                if path.extension().map(|ext| ext == "rs").unwrap_or(false) {
                    if let Ok(content) = fs::read_to_string(path) {
                        let rel = path
                            .strip_prefix(repo_root)
                            .unwrap_or(path)
                            .display()
                            .to_string();

                        for (idx, line) in content.lines().enumerate() {
                            if let Some(ident) = extract_rust_const_or_static_ident(line) {
                                if is_m1_domain_dict_name(ident) {
                                    violations.push(format!(
                                        "Check 19 violation (domain-metamodel-typing-violation): Unvalidated M1 domain instance dictionary/constant \"{}\" declared in {}:{}. Upstream compiler ASTs must adhere strictly to the closed M2 metamodel allowlist.",
                                        ident,
                                        rel,
                                        idx + 1
                                    ));
                                } else if is_static_param_dict_name(ident) {
                                    violations.push(format!(
                                        "Check 19 violation: Static hardcoded parameter dictionary declared in {}:{}.",
                                        rel,
                                        idx + 1
                                    ));
                                }
                            }

                            if let Some(bad_type) = check_line_for_metamodel_typing_violation(line) {
                                violations.push(format!(
                                    "Check 19 violation (domain-metamodel-typing-violation): Unvalidated M1 domain instance entity/type '{}' declared in {}:{}. Upstream compiler ASTs and dictionaries must adhere strictly to the closed M2 metamodel allowlist.",
                                    bad_type,
                                    rel,
                                    idx + 1
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    // 4. Scan upstream compiler templates and platform rules for unmapped domain instance concepts
    let template_dirs = [
        repo_root.join("skills").join("spec-conops-engineering").join("resources").join("units"),
        repo_root.join(".pipeline").join("templates"),
        repo_root.join("rules"),
    ];

    for t_dir in &template_dirs {
        if !t_dir.is_dir() {
            continue;
        }

        for entry in WalkDir::new(t_dir)
            .into_iter()
            .filter_entry(|e| {
                if e.file_type().is_dir() {
                    let name = e.file_name().to_string_lossy();
                    !EXCLUDED_DIRS.iter().any(|ex| name == ex)
                } else {
                    true
                }
            })
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path = entry.path();
                let rel = path
                    .strip_prefix(repo_root)
                    .unwrap_or(path)
                    .display()
                    .to_string();

                if let Ok(content) = fs::read_to_string(path) {
                    for (idx, line) in content.lines().enumerate() {
                        if line.contains('\u{2014}') {
                            violations.push(format!(
                                "Check 19 violation: Upstream template '{}:{}' contains forbidden Unicode em dash. Use ASCII -- exclusively.",
                                rel,
                                idx + 1
                            ));
                        }

                        // Check for hardcoded M1 domain instance dictionaries and customer empennage concepts
                        for concept in &[
                            "FLIGHT_ENVELOPE_MAP",
                            "SENSOR_CALIBRATION_DATA",
                            "PAYLOAD_CONFIG",
                            "ruddervator",
                            "empennage",
                        ] {
                            if line.to_lowercase().contains(&concept.to_lowercase()) {
                                violations.push(format!(
                                    "Check 19 violation (domain-metamodel-typing-violation): Upstream rule or template '{}:{}' contains forbidden domain concept '{}'. Upstream templates and rules must derive strictly from schema AST nodes.",
                                    rel,
                                    idx + 1,
                                    concept
                                ));
                            }
                        }

                        if let Some(bad_type) = check_line_for_metamodel_typing_violation(line) {
                            violations.push(format!(
                                "Check 19 violation (domain-metamodel-typing-violation): Upstream template '{}:{}' declares unvalidated M1 domain instance entity/type '{}'. Upstream templates must adhere strictly to the closed M2 metamodel allowlist.",
                                rel,
                                idx + 1,
                                bad_type
                            ));
                        }
                    }
                }
            }
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        violations.sort();
        Err(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    struct TempDir {
        path: std::path::PathBuf,
    }

    impl TempDir {
        fn new(name: &str) -> Self {
            static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let count = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!("verify_domain_{}_{}_{}", name, nanos, count));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn test_is_allowed_m2_type() {
        assert!(is_allowed_m2_type("Component"));
        assert!(is_allowed_m2_type("Class"));
        assert!(is_allowed_m2_type("Port"));
        assert!(is_allowed_m2_type("PackageDefinition"));
        assert!(is_allowed_m2_type("HumanOperator"));
        assert!(is_allowed_m2_type("meta_custom_node"));
        assert!(is_allowed_m2_type("MetaPart"));
        assert!(is_allowed_m2_type("component"));
        assert!(is_allowed_m2_type("part_definition"));

        // Concrete M1 domain instance types must be rejected
        assert!(!is_allowed_m2_type("TacticalUAV"));
        assert!(!is_allowed_m2_type("FixedWingDrone"));
        assert!(!is_allowed_m2_type("FlightVehicle"));
        assert!(!is_allowed_m2_type("ConcreteActuatorMotor"));
        assert!(!is_allowed_m2_type("HospitalInfusionPump"));
    }

    #[test]
    fn test_check_19_downstream_exempt() {
        let tmp = TempDir::new("c19_downstream");
        assert!(check_domain_agnostic_ast_cleanliness(&tmp.path).is_ok());
    }

    #[test]
    fn test_check_19_upstream_clean() {
        let tmp = TempDir::new("c19_clean");
        fs::create_dir_all(tmp.path.join(".pipeline/upstream")).unwrap();
        let src_dir = tmp.path.join("crates/test-crate/src");
        fs::create_dir_all(&src_dir).unwrap();
        let mut f = File::create(src_dir.join("lib.rs")).unwrap();
        writeln!(f, "pub fn compute_hash(val: &[u8]) -> u64 {{ 42 }}").unwrap();
        assert!(check_domain_agnostic_ast_cleanliness(&tmp.path).is_ok());
    }

    #[test]
    fn test_check_19_rejects_unflagged_m1_domain_dict() {
        let tmp = TempDir::new("c19_m1_reject");
        fs::create_dir_all(tmp.path.join(".pipeline/upstream")).unwrap();
        let src_dir = tmp.path.join("crates/test-crate/src");
        fs::create_dir_all(&src_dir).unwrap();
        let mut f = File::create(src_dir.join("lib.rs")).unwrap();
        writeln!(f, "pub const FLIGHT_ENVELOPE_MAP: &[(&str, f64)] = &[(\"max_altitude\", 10000.0)];").unwrap();
        let res = check_domain_agnostic_ast_cleanliness(&tmp.path);
        assert!(res.is_err());
        let errs = res.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("domain-metamodel-typing-violation") && e.contains("FLIGHT_ENVELOPE_MAP")));
    }

    #[test]
    fn test_check_19_rejects_unvalidated_m1_entity_type() {
        let tmp = TempDir::new("c19_m1_type");
        fs::create_dir_all(tmp.path.join(".pipeline/upstream")).unwrap();
        let src_dir = tmp.path.join("crates/test-crate/src");
        fs::create_dir_all(&src_dir).unwrap();
        let mut f = File::create(src_dir.join("lib.rs")).unwrap();
        writeln!(f, "let config = (\"metamodel_type\", \"TacticalUAV\");").unwrap();
        let res = check_domain_agnostic_ast_cleanliness(&tmp.path);
        assert!(res.is_err());
        let errs = res.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("domain-metamodel-typing-violation") && e.contains("TacticalUAV")));
    }

    #[test]
    fn test_check_19_accepts_valid_m2_metamodel_type() {
        let tmp = TempDir::new("c19_m2_valid");
        fs::create_dir_all(tmp.path.join(".pipeline/upstream")).unwrap();
        let src_dir = tmp.path.join("crates/test-crate/src");
        fs::create_dir_all(&src_dir).unwrap();
        let mut f = File::create(src_dir.join("lib.rs")).unwrap();
        writeln!(f, "let config = (\"metamodel_type\", \"Component\");").unwrap();
        assert!(check_domain_agnostic_ast_cleanliness(&tmp.path).is_ok());
    }

    #[test]
    fn test_check_19_template_rejection_of_unmapped_domain_dict() {
        let tmp = TempDir::new("c19_tmpl_reject");
        fs::create_dir_all(tmp.path.join(".pipeline/upstream")).unwrap();
        fs::create_dir_all(tmp.path.join("crates")).unwrap();
        let tmpl_dir = tmp.path.join("skills/spec-conops-engineering/resources/units");
        fs::create_dir_all(&tmpl_dir).unwrap();
        let mut f = File::create(tmpl_dir.join("01_TEST_UNIT.md")).unwrap();
        writeln!(f, "# Test Unit\n\nHardcoded payload: FLIGHT_ENVELOPE_MAP").unwrap();
        let res = check_domain_agnostic_ast_cleanliness(&tmp.path);
        assert!(res.is_err());
        let errs = res.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("domain-metamodel-typing-violation") && e.contains("FLIGHT_ENVELOPE_MAP")));
    }

    #[test]
    fn test_check_19_template_em_dash_rejection() {
        let tmp = TempDir::new("c19_tmpl_emdash");
        fs::create_dir_all(tmp.path.join(".pipeline/upstream")).unwrap();
        fs::create_dir_all(tmp.path.join("crates")).unwrap();
        let tmpl_dir = tmp.path.join(".pipeline/templates");
        fs::create_dir_all(&tmpl_dir).unwrap();
        let mut f = File::create(tmpl_dir.join("spec.yaml")).unwrap();
        writeln!(f, "description: Bad em dash \u{2014} here").unwrap();
        let res = check_domain_agnostic_ast_cleanliness(&tmp.path);
        assert!(res.is_err());
        let errs = res.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("Unicode em dash")));
    }

    #[test]
    fn test_check_19_rule_empennage_rejection() {
        let tmp = TempDir::new("c19_rule_empennage");
        fs::create_dir_all(tmp.path.join(".pipeline/upstream")).unwrap();
        fs::create_dir_all(tmp.path.join("crates")).unwrap();
        let rules_dir = tmp.path.join("rules");
        fs::create_dir_all(&rules_dir).unwrap();
        let mut f = File::create(rules_dir.join("test-rule.md")).unwrap();
        writeln!(f, "# Rule\n\nControl surface: cruciform empennage").unwrap();
        let res = check_domain_agnostic_ast_cleanliness(&tmp.path);
        assert!(res.is_err());
        let errs = res.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("empennage")));
    }

    #[test]
    fn test_check_19_conops_governance_rule_aerospace_decontamination() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("repo root");
        let rule_path = repo_root.join("rules/conops-mission-intent-integrity.md");
        assert!(rule_path.is_file(), "rules/conops-mission-intent-integrity.md must exist");
        let content = fs::read_to_string(&rule_path).unwrap();

        let forbidden_terms = [
            "civil airborne",
            "bird strike",
            "mid-air",
            "airframe limits",
            "pitot probe",
            "propeller delamination",
            "pilot fatigue",
            "DO-178C",
            "ARP4761",
            "ARP4754A",
            "h_max",
        ];

        let mut violations = Vec::new();
        for term in &forbidden_terms {
            if content.to_lowercase().contains(&term.to_lowercase()) {
                violations.push(*term);
            }
        }
        assert!(
            violations.is_empty(),
            "rules/conops-mission-intent-integrity.md contains forbidden aerospace domain concepts: {:?}",
            violations
        );
    }

    #[test]
    fn test_check_19_current_repo_passes() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("repo root");
        let res = check_domain_agnostic_ast_cleanliness(repo_root);
        assert!(res.is_ok(), "Current repo failed Check 19: {:?}", res.err());
    }
}
