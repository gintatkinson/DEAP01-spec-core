//! Semantic validation engine for SysML v2 / KerML models.

use super::symbols::{Symbol, SymbolKind, SymbolTable};
use deap_core::sysml_ast::*;
use regex::Regex;
use std::collections::HashSet;

/// Severity level of semantic diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

/// Structured diagnostic emitted during semantic validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticDiagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub element: String,
}

impl SemanticDiagnostic {
    pub fn error(element: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            element: element.into(),
        }
    }

    pub fn warning(element: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
            element: element.into(),
        }
    }
}

/// Semantic validator verifying topology, types, and referential integrity.
pub struct SemanticValidator<'a> {
    pkg: &'a PackageDef,
    symbols: SymbolTable,
    diagnostics: Vec<SemanticDiagnostic>,
}

impl<'a> SemanticValidator<'a> {
    pub fn new(pkg: &'a PackageDef) -> Self {
        let symbols = SymbolTable::build_from_package(pkg);
        Self {
            pkg,
            symbols,
            diagnostics: Vec::new(),
        }
    }

    /// Execute complete semantic validation suite.
    pub fn validate(&mut self) -> &[SemanticDiagnostic] {
        self.diagnostics.clear();
        self.validate_part_hierarchy();
        self.validate_connections();
        self.validate_referential_integrity();
        self.validate_part_defs();
        self.validate_requirements();
        &self.diagnostics
    }

    /// Check if any error-level diagnostics were recorded.
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == DiagnosticSeverity::Error)
    }

    /// Filter error diagnostics.
    pub fn errors(&self) -> Vec<&SemanticDiagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .collect()
    }

    // --- Validation Rules ---

    fn validate_part_hierarchy(&mut self) {
        let mut seen_parts = HashSet::new();
        self.check_parts_recursive(&self.pkg.part_defs, "", &mut seen_parts);
    }

    fn check_parts_recursive(
        &mut self,
        parts: &[PartDef],
        scope: &str,
        seen: &mut HashSet<String>,
    ) {
        for part in parts {
            let part_key = if scope.is_empty() {
                part.name.clone()
            } else {
                format!("{}::{}", scope, part.name)
            };

            if !seen.insert(part_key.clone()) {
                self.diagnostics.push(SemanticDiagnostic::error(
                    &part.name,
                    format!("Duplicate part declaration '{}' in scope '{}'", part.name, scope),
                ));
            }

            // Check duplicate ports on this part
            let mut seen_ports = HashSet::new();
            for port in &part.ports {
                if !seen_ports.insert(port.name.clone()) {
                    self.diagnostics.push(SemanticDiagnostic::error(
                        format!("{}.{}", part.name, port.name),
                        format!(
                            "Duplicate port declaration '{}' on part '{}'",
                            port.name, part.name
                        ),
                    ));
                }
            }

            self.check_parts_recursive(&part.parts, &part_key, seen);
        }
    }

    fn validate_connections(&mut self) {
        self.validate_package_connections(self.pkg);
    }

    fn validate_package_connections(&mut self, pkg: &PackageDef) {
        self.validate_connection_list(&pkg.connection_defs, None);

        for part in &pkg.part_defs {
            self.validate_part_connections(part);
        }

        for subpkg in &pkg.packages {
            self.validate_package_connections(subpkg);
        }
    }

    fn validate_part_connections(&mut self, part: &PartDef) {
        self.validate_connection_list(&part.connections, Some(&part.name));
        for subpart in &part.parts {
            self.validate_part_connections(subpart);
        }
    }

    fn validate_connection_list(&mut self, conns: &[ConnectionDef], part_scope: Option<&str>) {
        for conn in conns {
            let src_sym = self.symbols.lookup_port(&conn.source_port, part_scope);
            let tgt_sym = self.symbols.lookup_port(&conn.target_port, part_scope);

            if src_sym.is_none() {
                self.diagnostics.push(SemanticDiagnostic::error(
                    &conn.name,
                    format!(
                        "Connection '{}' references undefined source port '{}'",
                        conn.name, conn.source_port
                    ),
                ));
            }

            if tgt_sym.is_none() {
                self.diagnostics.push(SemanticDiagnostic::error(
                    &conn.name,
                    format!(
                        "Connection '{}' references undefined target port '{}'",
                        conn.name, conn.target_port
                    ),
                ));
            }

            if let (Some(src), Some(tgt)) = (src_sym, tgt_sym) {
                let src_eff_dir = get_effective_direction(src);
                let tgt_eff_dir = get_effective_direction(tgt);

                // Boundary delegation:
                // Inside a part definition, connecting an input boundary port of the enclosing part
                // inward to a subpart's input port (in -> in), or an internal subpart's output port
                // outward to an output boundary port (out -> out), represents valid delegation in SysML v2.
                let is_src_boundary = part_scope.is_some() && !conn.source_port.contains('.');
                let is_tgt_boundary = part_scope.is_some() && !conn.target_port.contains('.');

                if is_src_boundary && !is_tgt_boundary && src_eff_dir == "in" && tgt_eff_dir == "in" {
                    // Valid inward delegation: boundary in -> subpart in
                } else if !is_src_boundary && is_tgt_boundary && src_eff_dir == "out" && tgt_eff_dir == "out" {
                    // Valid outward delegation: subpart out -> boundary out
                } else if is_src_boundary && is_tgt_boundary && src_eff_dir == "in" && tgt_eff_dir == "out" {
                    // Valid boundary pass-through delegation: boundary in -> boundary out
                } else {
                    match (src_eff_dir.as_str(), tgt_eff_dir.as_str()) {
                        ("out", "in") | ("inout", "in") | ("out", "inout") | ("inout", "inout") => {
                            // Compatible flow direction
                        }
                        ("in", "out") => {
                            self.diagnostics.push(SemanticDiagnostic::error(
                                &conn.name,
                                format!(
                                    "Inverted flow direction in connection '{}': source port '{}' ({}) cannot feed target port '{}' ({})",
                                    conn.name, conn.source_port, src_eff_dir, conn.target_port, tgt_eff_dir
                                ),
                            ));
                        }
                        ("in", "in") => {
                            self.diagnostics.push(SemanticDiagnostic::error(
                                &conn.name,
                                format!(
                                    "Invalid connection '{}': cannot connect input port '{}' to input port '{}'",
                                    conn.name, conn.source_port, conn.target_port
                                ),
                            ));
                        }
                        ("out", "out") => {
                            self.diagnostics.push(SemanticDiagnostic::error(
                                &conn.name,
                                format!(
                                    "Invalid connection '{}': cannot connect output port '{}' to output port '{}'",
                                    conn.name, conn.source_port, conn.target_port
                                ),
                            ));
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn validate_referential_integrity(&mut self) {
        let known_constraints: HashSet<String> = self
            .symbols
            .all_of_kind(SymbolKind::Constraint)
            .into_iter()
            .map(|s| normalize_id(&s.name))
            .collect();

        let known_hazards: HashSet<String> = self
            .symbols
            .all_of_kind(SymbolKind::Hazard)
            .into_iter()
            .map(|s| normalize_id(&s.name))
            .collect();

        let safety_realises_re = Regex::new(
            r#"(?i)@SafetyRealises\s*(?:\(([^)]+)\)|:\s*([A-Za-z0-9_\-]+))"#,
        )
        .unwrap();

        let triggers_hazard_re = Regex::new(
            r#"(?i)@TriggersHazard\s*(?:\(([^)]+)\)|:\s*([A-Za-z0-9_\-]+))"#,
        )
        .unwrap();

        let mut req_texts = Vec::new();
        self.collect_requirement_texts(&self.pkg.requirement_defs, &mut req_texts);
        for part in &self.pkg.part_defs {
            self.collect_part_requirement_texts(part, &mut req_texts);
        }

        for (req_name, text_corpus) in req_texts {
            // Check @SafetyRealises
            for cap in safety_realises_re.captures_iter(&text_corpus) {
                let targets = if let Some(grp) = cap.get(1) {
                    grp.as_str()
                        .split(',')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .collect()
                } else if let Some(grp) = cap.get(2) {
                    vec![grp.as_str().trim()]
                } else {
                    vec![]
                };

                for target in targets {
                    let clean_target = normalize_id(target);
                    if !known_constraints.is_empty()
                        && !known_constraints.iter().any(|c| c == &clean_target || c.contains(&clean_target))
                    {
                        self.diagnostics.push(SemanticDiagnostic::error(
                            &req_name,
                            format!(
                                "Requirement '{}' references undefined constraint/UCA '{}' in @SafetyRealises",
                                req_name, target
                            ),
                        ));
                    }
                }
            }

            // Check @TriggersHazard
            for cap in triggers_hazard_re.captures_iter(&text_corpus) {
                let targets = if let Some(grp) = cap.get(1) {
                    grp.as_str()
                        .split(',')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .collect()
                } else if let Some(grp) = cap.get(2) {
                    vec![grp.as_str().trim()]
                } else {
                    vec![]
                };

                for target in targets {
                    let clean_target = normalize_id(target);
                    if !known_hazards.is_empty()
                        && !known_hazards.iter().any(|h| h == &clean_target || h.contains(&clean_target))
                    {
                        self.diagnostics.push(SemanticDiagnostic::error(
                            &req_name,
                            format!(
                                "Requirement '{}' references undefined hazard '{}' in @TriggersHazard",
                                req_name, target
                            ),
                        ));
                    }
                }
            }
        }
    }

    fn collect_requirement_texts(&self, reqs: &[RequirementDef], out: &mut Vec<(String, String)>) {
        for req in reqs {
            let corpus = format!(
                "{} {} {}",
                req.doc.as_deref().unwrap_or(""),
                req.text,
                req.assumes.join(" ")
            );
            out.push((req.name.clone(), corpus));
        }
    }

    fn collect_part_requirement_texts(&self, part: &PartDef, out: &mut Vec<(String, String)>) {
        self.collect_requirement_texts(&part.requirements, out);
        for subpart in &part.parts {
            self.collect_part_requirement_texts(subpart, out);
        }
    }

    /// Validate part definitions, ensuring all referenced types resolve within local scope, imported namespaces, or global symbols.
    fn validate_part_defs(&mut self) {
        self.validate_part_defs_in_pkg(self.pkg, "");
    }

    fn validate_part_defs_in_pkg(&mut self, pkg: &PackageDef, parent_scope: &str) {
        let pkg_scope = if parent_scope.is_empty() {
            pkg.name.clone()
        } else if pkg.name.is_empty() {
            parent_scope.to_string()
        } else {
            format!("{}::{}", parent_scope, pkg.name)
        };

        for part in &pkg.part_defs {
            self.validate_single_part_def(part, &pkg_scope);
        }

        for subpkg in &pkg.packages {
            self.validate_part_defs_in_pkg(subpkg, &pkg_scope);
        }
    }

    fn validate_single_part_def(&mut self, part: &PartDef, scope: &str) {
        if let Some(ref ty) = part.type_name {
            if !is_primitive_or_builtin_type(ty) && self.symbols.lookup_in_scope(scope, ty).is_none() {
                self.diagnostics.push(SemanticDiagnostic::error(
                    &part.name,
                    format!("Part '{}' references undefined type '{}'", part.name, ty),
                ));
            }
        }

        let part_scope = if scope.is_empty() {
            part.name.clone()
        } else {
            format!("{}::{}", scope, part.name)
        };

        for subpart in &part.parts {
            self.validate_single_part_def(subpart, &part_scope);
        }
    }

    /// Validate requirement definitions, ensuring satisfying elements and attribute types resolve in scope.
    fn validate_requirements(&mut self) {
        self.validate_requirements_in_pkg(self.pkg, "");
    }

    fn validate_requirements_in_pkg(&mut self, pkg: &PackageDef, parent_scope: &str) {
        let pkg_scope = if parent_scope.is_empty() {
            pkg.name.clone()
        } else if pkg.name.is_empty() {
            parent_scope.to_string()
        } else {
            format!("{}::{}", parent_scope, pkg.name)
        };

        for req in &pkg.requirement_defs {
            self.validate_single_requirement_def(req, &pkg_scope);
        }

        for part in &pkg.part_defs {
            self.validate_part_requirements(part, &pkg_scope);
        }

        for subpkg in &pkg.packages {
            self.validate_requirements_in_pkg(subpkg, &pkg_scope);
        }
    }

    fn validate_single_requirement_def(&mut self, req: &RequirementDef, scope: &str) {
        for attr in &req.attributes {
            if !is_primitive_or_builtin_type(&attr.type_name)
                && self.symbols.lookup_in_scope(scope, &attr.type_name).is_none()
            {
                self.diagnostics.push(SemanticDiagnostic::error(
                    &req.name,
                    format!(
                        "Requirement '{}' attribute '{}' references undefined type '{}'",
                        req.name, attr.name, attr.type_name
                    ),
                ));
            }
        }

        for target in &req.satisfied_by {
            if !is_primitive_or_builtin_type(target)
                && self.symbols.lookup_in_scope(scope, target).is_none()
            {
                self.diagnostics.push(SemanticDiagnostic::error(
                    &req.name,
                    format!(
                        "Requirement '{}' references undefined satisfying element '{}'",
                        req.name, target
                    ),
                ));
            }
        }
    }

    fn validate_part_requirements(&mut self, part: &PartDef, scope: &str) {
        let part_scope = if scope.is_empty() {
            part.name.clone()
        } else {
            format!("{}::{}", scope, part.name)
        };

        for req in &part.requirements {
            self.validate_single_requirement_def(req, &part_scope);
        }

        for subpart in &part.parts {
            self.validate_part_requirements(subpart, &part_scope);
        }
    }
}

fn is_primitive_or_builtin_type(type_name: &str) -> bool {
    let clean = type_name.trim();
    if clean.is_empty() {
        return true;
    }
    let base = if let Some(bracket_idx) = clean.find('[') {
        clean[..bracket_idx].trim()
    } else {
        clean
    };

    if base.starts_with("ScalarValues::") || base.starts_with("ISQ::") || base.starts_with("SI::") {
        return true;
    }

    matches!(
        base,
        "Boolean"
            | "bool"
            | "boolean"
            | "Integer"
            | "int"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "Natural"
            | "natural"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "Real"
            | "real"
            | "Float"
            | "float"
            | "f32"
            | "f64"
            | "String"
            | "string"
            | "str"
            | "ScalarValues"
    )
}

fn get_effective_direction(port_sym: &Symbol) -> String {
    let dir = port_sym.direction.as_deref().unwrap_or("inout");
    if port_sym.is_conjugated {
        match dir {
            "in" => "out".to_string(),
            "out" => "in".to_string(),
            other => other.to_string(),
        }
    } else {
        dir.to_string()
    }
}

fn normalize_id(id: &str) -> String {
    id.trim().replace('-', "_").to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_connection_passes() {
        let pkg = PackageDef {
            name: "TestSys".to_string(),
            part_defs: vec![
                PartDef {
                    name: "Sensor".to_string(),
                    ports: vec![
                        PortDef {
                            name: "out_data".to_string(),
                            direction: "out".to_string(),
                            type_name: "RawData".to_string(),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                },
                PartDef {
                    name: "Computer".to_string(),
                    ports: vec![
                        PortDef {
                            name: "in_data".to_string(),
                            direction: "in".to_string(),
                            type_name: "RawData".to_string(),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                },
            ],
            connection_defs: vec![
                ConnectionDef {
                    name: "SensorToComputer".to_string(),
                    source_port: "Sensor.out_data".to_string(),
                    target_port: "Computer.in_data".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let mut validator = SemanticValidator::new(&pkg);
        validator.validate();
        assert!(!validator.has_errors(), "Expected no errors, got: {:?}", validator.errors());
    }

    #[test]
    fn test_mismatched_connection_direction_fails() {
        let pkg = PackageDef {
            name: "TestSys".to_string(),
            part_defs: vec![
                PartDef {
                    name: "Sensor".to_string(),
                    ports: vec![
                        PortDef {
                            name: "in_data".to_string(),
                            direction: "in".to_string(),
                            type_name: "RawData".to_string(),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                },
                PartDef {
                    name: "Computer".to_string(),
                    ports: vec![
                        PortDef {
                            name: "out_data".to_string(),
                            direction: "out".to_string(),
                            type_name: "RawData".to_string(),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                },
            ],
            connection_defs: vec![
                ConnectionDef {
                    name: "InvertedFlow".to_string(),
                    source_port: "Sensor.in_data".to_string(),
                    target_port: "Computer.out_data".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let mut validator = SemanticValidator::new(&pkg);
        validator.validate();
        assert!(validator.has_errors());
        let errors = validator.errors();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("Inverted flow direction"));
    }

    #[test]
    fn test_dangling_safety_realises_fails() {
        let pkg = PackageDef {
            name: "SafetySys".to_string(),
            requirement_defs: vec![
                RequirementDef {
                    name: "Req_Safety_1".to_string(),
                    text: "Shall maintain velocity limit. @SafetyRealises(UCA-99)".to_string(),
                    ..Default::default()
                },
            ],
            constraint_defs: vec![
                ConstraintDef {
                    name: "UCA_01_Overspeed".to_string(),
                    expression: "v <= 60.0".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let mut validator = SemanticValidator::new(&pkg);
        validator.validate();
        assert!(validator.has_errors());
        let errors = validator.errors();
        assert!(errors[0].message.contains("references undefined constraint/UCA 'UCA-99'"));
    }

    #[test]
    fn test_validate_part_defs_with_cross_package_import() {
        let pkg_a = PackageDef {
            name: "Subsystem_1".to_string(),
            part_defs: vec![
                PartDef {
                    name: "SystemVisionEngine".to_string(),
                    is_def: true,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let pkg_b = PackageDef {
            name: "Subsystem_2".to_string(),
            imports: vec![
                ImportDef {
                    path: "Subsystem_1".to_string(),
                    is_wildcard: true,
                    ..Default::default()
                },
            ],
            part_defs: vec![
                PartDef {
                    name: "v".to_string(),
                    is_def: false,
                    type_name: Some("SystemVisionEngine".to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let root = PackageDef {
            name: "".to_string(),
            packages: vec![pkg_a, pkg_b],
            ..Default::default()
        };

        let mut validator = SemanticValidator::new(&root);
        validator.validate();
        assert!(
            !validator.has_errors(),
            "Expected no validation errors with imported type, got: {:?}",
            validator.errors()
        );
    }

    #[test]
    fn test_validate_part_defs_missing_type_fails() {
        let pkg = PackageDef {
            name: "Subsystem_2".to_string(),
            part_defs: vec![
                PartDef {
                    name: "v".to_string(),
                    is_def: false,
                    type_name: Some("SystemVisionEngine".to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let mut validator = SemanticValidator::new(&pkg);
        validator.validate();
        assert!(validator.has_errors());
        let errors = validator.errors();
        assert!(errors
            .iter()
            .any(|e| e.message.contains("references undefined type 'SystemVisionEngine'")));
    }

    #[test]
    fn test_validate_requirements_with_cross_package_import() {
        let pkg_a = PackageDef {
            name: "Subsystem_1".to_string(),
            part_defs: vec![
                PartDef {
                    name: "SafetyController".to_string(),
                    is_def: true,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let pkg_b = PackageDef {
            name: "Subsystem_2".to_string(),
            imports: vec![
                ImportDef {
                    path: "Subsystem_1".to_string(),
                    is_wildcard: true,
                    ..Default::default()
                },
            ],
            requirement_defs: vec![
                RequirementDef {
                    name: "Req_Ctrl".to_string(),
                    text: "Shall engage safety controller.".to_string(),
                    satisfied_by: vec!["SafetyController".to_string()],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let root = PackageDef {
            name: "".to_string(),
            packages: vec![pkg_a, pkg_b],
            ..Default::default()
        };

        let mut validator = SemanticValidator::new(&root);
        validator.validate();
        assert!(
            !validator.has_errors(),
            "Expected no validation errors, got: {:?}",
            validator.errors()
        );
    }

    #[test]
    fn test_subpackage_connections_validated_for_dangling_ports() {
        let subpkg = PackageDef {
            name: "NestedSubsystem".to_string(),
            part_defs: vec![
                PartDef {
                    name: "Worker".to_string(),
                    ports: vec![
                        PortDef {
                            name: "out_stream".to_string(),
                            direction: "out".to_string(),
                            type_name: "StreamData".to_string(),
                            ..Default::default()
                        },
                    ],
                    connections: vec![
                        ConnectionDef {
                            name: "DanglingSubpartConn".to_string(),
                            source_port: "out_stream".to_string(),
                            target_port: "Worker.undefined_target_port".to_string(),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                },
            ],
            connection_defs: vec![
                ConnectionDef {
                    name: "DanglingPkgConn".to_string(),
                    source_port: "Worker.out_stream".to_string(),
                    target_port: "UndefinedPart.in_stream".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let root = PackageDef {
            name: "RootSystem".to_string(),
            packages: vec![subpkg],
            ..Default::default()
        };

        let mut validator = SemanticValidator::new(&root);
        validator.validate();
        assert!(
            validator.has_errors(),
            "Expected validation errors for dangling ports inside nested subpackage, but validator reported no errors"
        );
        let errors = validator.errors();
        assert!(
            errors.iter().any(|e| e.message.contains("references undefined target port 'Worker.undefined_target_port'")),
            "Expected error for Worker.undefined_target_port, got: {:?}",
            errors
        );
        assert!(
            errors.iter().any(|e| e.message.contains("references undefined target port 'UndefinedPart.in_stream'")),
            "Expected error for UndefinedPart.in_stream, got: {:?}",
            errors
        );
    }
}

