//! # Salsa Safety Validation, Rowan CST Span Splicer & Ariadne Diagnostics (`safety_db`)
//! [DOC-TIER: A]
//! ## 1. Safety Intent & Regulatory Scope -- Provides `#[salsa::tracked]` semantic/safety validation, formatting-preserving Rowan CST span splicing (`DEAP-E1005`), and `ariadne` multi-span diagnostic rendering (`BP-03`, `BP-10`).
//! ## 2. Invariants & Mathematical Properties -- Validates part hierarchies, port flow directions, `@SafetyRealises`/`@TriggersHazard` referential integrity, and disjoint reverse-byte-order CST span patches.
//! ## 3. Preconditions -- `PackageDef` and `SalsaFileInput` conform to `MAX_AST_DEPTH` (`64`).
//! ## 4. Postconditions -- Emits deterministic `SemanticDiagnostic` slices and spliced UTF-8 source text.
//! ## 5. Algorithmic Complexity Bounds -- Time: O(N log N); Space: O(N).
//! ## 6. Failure Modes & Fault Tolerance -- Caps AST/type traversal at depth 64; overlapping CST splices fail closed with `DEAP-E1005`.
//! ## 7. Realization & Traceability -- Realises: [REQ-0201], [REQ-0603], [REQ-0608] | [FLS: fls_66m4rnbssgig]

#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]

use std::collections::BTreeSet;

pub use deap_core::sysml_cst::{
    apply_formatting_preserving_splices, compute_bijective_digest, print_lossless_cst,
    render_ariadne_diagnostic, validate_disjoint_splice_ranges, AriadneSpanReport,
    BijectiveDigestReceipt, ConnectionDef, CstSplicePatch, PackageDef, PartDef,
};

use super::hir_symbols::{Symbol, SymbolKind, SymbolTable};
pub use super::safety_types::{
    is_primitive_or_builtin_type, normalize_source_type, split_top_level_type_args,
};
use super::safety_types::{
    check_type_resolvable_with, compute_scoped_package_name, extract_annotation_targets,
    AnnotationSpan, MAX_AST_DEPTH,
};
use super::source_db::SalsaFileInput;

/// Salsa tracked query executing semantic and STPA safety validation over a `SalsaFileInput`.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope -- Memoizes semantic and STPA referential integrity validation (`BP-10`).
/// ## 2. Invariants & Mathematical Properties -- Re-executes only when `input.text(db)` mutates.
/// ## 3. Preconditions -- `input` is a valid `SalsaFileInput` in `db`.
/// ## 4. Postconditions -- Returns `Vec<SemanticDiagnostic>` in deterministic validation order.
/// ## 5. Algorithmic Complexity Bounds -- Time: O(1) memoized hit, O(N log N) miss; Space: O(N).
/// ## 6. Failure Modes & Fault Tolerance -- Syntax errors are reported as `DiagnosticSeverity::Error` without panicking.
/// ## 7. Realization & Traceability -- Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
#[salsa::tracked(returns(clone))]
pub fn validate_safety_tracked(
    db: &dyn salsa::Database,
    input: SalsaFileInput,
) -> Vec<SemanticDiagnostic> {
    let text = input.text(db);
    match crate::parse_sysml(text) {
        Ok(pkg) => {
            let mut validator = SemanticValidator::new(&pkg);
            validator.validate().to_vec()
        }
        Err(err) => vec![SemanticDiagnostic::error(
            input.path(db).clone(),
            err.to_string(),
        )],
    }
}

/// Severity level of semantic diagnostics (`Error` or `Warning`). Realises: [REQ-SEMANTIC-VALIDATOR-DIAGNOSTICSEVERITY] | [FLS: fls_8tsynkj2cufj]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

/// Structured diagnostic emitted during SysML v2 semantic validation. Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICDIAGNOSTIC] | [FLS: fls_66m4rnbssgig]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticDiagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub element: String,
}

impl SemanticDiagnostic {
    /// Constructs an `Error`-level `SemanticDiagnostic` for `element`. Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICDIAGNOSTIC-ERROR] | [FLS: fls_66m4rnbssgig]
    #[must_use]
    pub fn error(element: impl Into<String>, message: impl Into<String>) -> Self {
        Self { severity: DiagnosticSeverity::Error, message: message.into(), element: element.into() }
    }

    /// Constructs a `Warning`-level `SemanticDiagnostic` for `element`. Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICDIAGNOSTIC-WARNING] | [FLS: fls_66m4rnbssgig]
    #[must_use]
    pub fn warning(element: impl Into<String>, message: impl Into<String>) -> Self {
        Self { severity: DiagnosticSeverity::Warning, message: message.into(), element: element.into() }
    }
}

/// Semantic validator verifying part hierarchy uniqueness, port flow compatibility, safety annotations, and type resolution. Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR] | [FLS: fls_66m4rnbssgig]
pub struct SemanticValidator<'a> {
    pkg: &'a PackageDef,
    symbols: SymbolTable,
    diagnostics: Vec<SemanticDiagnostic>,
}

impl<'a> SemanticValidator<'a> {
    /// Constructs a `SemanticValidator` and builds the scoped `SymbolTable` for `pkg`. Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-NEW] | [FLS: fls_66m4rnbssgig]
    #[must_use]
    pub fn new(pkg: &'a PackageDef) -> Self {
        Self { pkg, symbols: SymbolTable::build_from_package(pkg), diagnostics: Vec::new() }
    }

    /// Executes the complete SysML v2 semantic validation suite and returns all accumulated diagnostics. Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE] | [FLS: fls_66m4rnbssgig]
    pub fn validate(&mut self) -> &[SemanticDiagnostic] {
        self.diagnostics.clear();
        self.validate_part_hierarchy();
        self.validate_connections();
        self.validate_referential_integrity();
        self.validate_part_defs();
        self.validate_requirements();
        &self.diagnostics
    }

    /// Returns `true` if any `DiagnosticSeverity::Error` diagnostic was recorded. Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-HAS-ERRORS] | [FLS: fls_66m4rnbssgig]
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == DiagnosticSeverity::Error)
    }

    /// Returns all recorded `DiagnosticSeverity::Error` diagnostics. Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-ERRORS] | [FLS: fls_66m4rnbssgig]
    #[must_use]
    pub fn errors(&self) -> Vec<&SemanticDiagnostic> {
        self.diagnostics.iter().filter(|d| d.severity == DiagnosticSeverity::Error).collect()
    }

    /// Iteratively validates unique part and port declarations across nested packages up to `MAX_AST_DEPTH`. Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-PART-HIERARCHY] | [FLS: fls_66m4rnbssgig]
    fn validate_part_hierarchy(&mut self) {
        let mut seen = BTreeSet::new();
        for node in collect_scoped_ast(self.pkg).parts {
            let key = if node.hier_scope.is_empty() {
                node.part.name.clone()
            } else {
                format!("{}::{}", node.hier_scope, node.part.name)
            };
            if !seen.insert(key) {
                self.diagnostics.push(SemanticDiagnostic::error(
                    &node.part.name,
                    format!("Duplicate part declaration '{}' in scope '{}'", node.part.name, node.hier_scope),
                ));
            }
            let mut seen_ports = BTreeSet::new();
            for port in &node.part.ports {
                if !seen_ports.insert(&port.name) {
                    self.diagnostics.push(SemanticDiagnostic::error(
                        format!("{}.{}", node.part.name, port.name),
                        format!("Duplicate port declaration '{}' on part '{}'", port.name, node.part.name),
                    ));
                }
            }
        }
    }

    /// Validates port existence and flow compatibility across all package and part connections up to `MAX_AST_DEPTH`. Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-CONNECTIONS] | [FLS: fls_66m4rnbssgig]
    fn validate_connections(&mut self) {
        let ast = collect_scoped_ast(self.pkg);
        for (pkg, _) in &ast.pkgs {
            self.validate_connection_list(&pkg.connection_defs, None);
        }
        for node in &ast.parts {
            self.validate_connection_list(&node.part.connections, Some(&node.part.name));
        }
    }

    /// Validates endpoint port resolution and flow direction compatibility for `conns`. Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-CONNECTION-LIST] | [FLS: fls_66m4rnbssgig]
    fn validate_connection_list(&mut self, conns: &[ConnectionDef], part_scope: Option<&str>) {
        for conn in conns {
            let src_sym = self.symbols.lookup_port(&conn.source_port, part_scope);
            let tgt_sym = self.symbols.lookup_port(&conn.target_port, part_scope);
            if src_sym.is_none() {
                self.diagnostics.push(SemanticDiagnostic::error(
                    &conn.name,
                    format!("Connection '{}' references undefined source port '{}'", conn.name, conn.source_port),
                ));
            }
            if tgt_sym.is_none() {
                self.diagnostics.push(SemanticDiagnostic::error(
                    &conn.name,
                    format!("Connection '{}' references undefined target port '{}'", conn.name, conn.target_port),
                ));
            }
            if let (Some(src), Some(tgt)) = (src_sym, tgt_sym) {
                let (src_eff, tgt_eff) = (get_effective_direction(src), get_effective_direction(tgt));
                let is_src_b = part_scope.is_some() && !conn.source_port.contains('.');
                let is_tgt_b = part_scope.is_some() && !conn.target_port.contains('.');
                let valid_delegation = (is_src_b && !is_tgt_b && src_eff == "in" && tgt_eff == "in")
                    || (!is_src_b && is_tgt_b && src_eff == "out" && tgt_eff == "out")
                    || (is_src_b && is_tgt_b && src_eff == "in" && tgt_eff == "out");
                if !valid_delegation {
                    let msg = match (src_eff.as_str(), tgt_eff.as_str()) {
                        ("out" | "inout", "in" | "inout") => None,
                        ("in", "out") => Some(format!(
                            "Inverted flow direction in connection '{}': source port '{}' ({}) cannot feed target port '{}' ({})",
                            conn.name, conn.source_port, src_eff, conn.target_port, tgt_eff
                        )),
                        ("in", "in") => Some(format!(
                            "Invalid connection '{}': cannot connect input port '{}' to input port '{}'",
                            conn.name, conn.source_port, conn.target_port
                        )),
                        ("out", "out") => Some(format!(
                            "Invalid connection '{}': cannot connect output port '{}' to output port '{}'",
                            conn.name, conn.source_port, conn.target_port
                        )),
                        _ => None,
                    };
                    if let Some(m) = msg {
                        self.diagnostics.push(SemanticDiagnostic::error(&conn.name, m));
                    }
                }
            }
        }
    }

    /// Validates `@SafetyRealises` and `@TriggersHazard` referential integrity across all requirements. Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-REFERENTIAL-INTEGRITY] | [FLS: fls_66m4rnbssgig]
    fn validate_referential_integrity(&mut self) {
        let known_constraints: BTreeSet<String> =
            self.symbols.all_of_kind(SymbolKind::Constraint).into_iter().map(|s| normalize_id(&s.name)).collect();
        let known_hazards: BTreeSet<String> =
            self.symbols.all_of_kind(SymbolKind::Hazard).into_iter().map(|s| normalize_id(&s.name)).collect();
        let ast = collect_scoped_ast(self.pkg);
        let reqs = ast.pkgs.iter().flat_map(|(p, _)| &p.requirement_defs).chain(ast.parts.iter().flat_map(|n| &n.part.requirements));
        for r in reqs {
            let corpus = format!("{} {} {}", r.doc.as_deref().unwrap_or(""), r.text, r.assumes.join(" "));
            self.validate_annotation_spans(&r.name, &corpus, ("@SafetyRealises", "constraint/UCA"), &known_constraints);
            self.validate_annotation_spans(&r.name, &corpus, ("@TriggersHazard", "hazard"), &known_hazards);
        }
    }

    /// Validates extracted annotation spans for `tag` against `known_ids`. Realises: [REQ-SEMANTIC-VALIDATOR] | [FLS: fls_66m4rnbssgig]
    fn validate_annotation_spans(&mut self, req_name: &str, text_corpus: &str, (tag, kind_label): (&str, &str), known_ids: &BTreeSet<String>) {
        for span in extract_annotation_targets(text_corpus, tag) {
            match span {
                AnnotationSpan::Targets(targets) => {
                    for target in &targets {
                        let clean = normalize_id(target);
                        if !known_ids.is_empty() && !known_ids.iter().any(|k| k == &clean || k.contains(&clean)) {
                            self.diagnostics.push(SemanticDiagnostic::error(
                                req_name,
                                format!("Requirement '{req_name}' references undefined {kind_label} '{target}' in {tag}"),
                            ));
                        }
                    }
                }
                AnnotationSpan::UnclosedDelimiter => self.diagnostics.push(SemanticDiagnostic::error(
                    req_name,
                    format!("Requirement '{req_name}' contains unclosed '(' delimiter in {tag}"),
                )),
            }
        }
    }

    /// Iteratively validates `part_defs` type references up to `MAX_AST_DEPTH`. Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-PART-DEFS] | [FLS: fls_66m4rnbssgig]
    fn validate_part_defs(&mut self) {
        for node in collect_scoped_ast(self.pkg).parts {
            if let Some(ref ty) = node.part.type_name {
                if !self.is_resolvable_type(&node.enclosing_scope, ty) {
                    self.diagnostics.push(SemanticDiagnostic::error(
                        &node.part.name,
                        format!("Part '{}' references undefined type '{}'", node.part.name, ty),
                    ));
                }
            }
        }
    }

    /// Iteratively validates requirement attribute types and `satisfied_by` references up to `MAX_AST_DEPTH`. Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-REQUIREMENTS] | [FLS: fls_66m4rnbssgig]
    fn validate_requirements(&mut self) {
        let ast = collect_scoped_ast(self.pkg);
        let scoped_reqs = ast.pkgs.iter().flat_map(|(pkg, scope)| pkg.requirement_defs.iter().map(move |r| (r, scope.as_str())))
            .chain(ast.parts.iter().flat_map(|n| n.part.requirements.iter().map(move |r| (r, n.part_scope.as_str()))));
        for (req, scope) in scoped_reqs {
            for attr in &req.attributes {
                if !self.is_resolvable_type(scope, &attr.type_name) {
                    self.diagnostics.push(SemanticDiagnostic::error(
                        &req.name,
                        format!("Requirement '{}' attribute '{}' references undefined type '{}'", req.name, attr.name, attr.type_name),
                    ));
                }
            }
            for target in &req.satisfied_by {
                if !self.is_resolvable_type(scope, target) {
                    self.diagnostics.push(SemanticDiagnostic::error(
                        &req.name,
                        format!("Requirement '{}' references undefined satisfying element '{}'", req.name, target),
                    ));
                }
            }
        }
    }

    /// Checks whether a type expression is resolvable in `scope` within `MAX_AST_DEPTH` (64). Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-IS-RESOLVABLE-TYPE] | [FLS: fls_66m4rnbssgig]
    #[must_use]
    pub fn is_resolvable_type(&self, scope: &str, type_name: &str) -> bool {
        check_type_resolvable_with(type_name, 0, |sym| self.symbols.lookup_in_scope(scope, sym).is_some())
    }
}

/// Flattened part AST node paired with its hierarchical and symbol-table scopes. Realises: [REQ-SEMANTIC-VALIDATOR] | [FLS: fls_66m4rnbssgig]
struct ScopedPartNode<'a> {
    part: &'a PartDef,
    hier_scope: String,
    enclosing_scope: String,
    part_scope: String,
}

/// Flattened package and part hierarchy collected within `MAX_AST_DEPTH`. Realises: [REQ-SEMANTIC-VALIDATOR] | [FLS: fls_66m4rnbssgig]
struct ScopedAst<'a> {
    pkgs: Vec<(&'a PackageDef, String)>,
    parts: Vec<ScopedPartNode<'a>>,
}

/// Iteratively collects all scoped packages and parts from `root` up to `MAX_AST_DEPTH`. Realises: [REQ-SEMANTIC-VALIDATOR] | [FLS: fls_66m4rnbssgig]
fn collect_scoped_ast(root: &PackageDef) -> ScopedAst<'_> {
    let (mut pkgs, mut parts) = (Vec::new(), Vec::new());
    let mut pkg_stack = vec![(root, String::new(), String::new(), 0usize)];
    while let Some((pkg, parent_scope, hier_parent, depth)) = pkg_stack.pop() {
        if depth >= MAX_AST_DEPTH {
            continue;
        }
        let pkg_scope = compute_scoped_package_name(pkg, &parent_scope);
        let hier_pkg = if hier_parent.is_empty() {
            if std::ptr::eq(pkg, root) { String::new() } else { pkg.name.clone() }
        } else if pkg.name.is_empty() {
            hier_parent
        } else {
            format!("{hier_parent}::{}", pkg.name)
        };
        pkgs.push((pkg, pkg_scope.clone()));
        let next_depth = depth.saturating_add(1);
        if next_depth < MAX_AST_DEPTH {
            for root_part in &pkg.part_defs {
                let mut part_stack = vec![(root_part, pkg_scope.clone(), hier_pkg.clone(), next_depth)];
                while let Some((part, enclosing, h_scope, p_depth)) = part_stack.pop() {
                    if p_depth >= MAX_AST_DEPTH {
                        continue;
                    }
                    let part_scope = if enclosing.is_empty() { part.name.clone() } else { format!("{enclosing}::{}", part.name) };
                    let next_hier = if h_scope.is_empty() { part.name.clone() } else { format!("{h_scope}::{}", part.name) };
                    parts.push(ScopedPartNode { part, hier_scope: h_scope, enclosing_scope: enclosing, part_scope: part_scope.clone() });
                    let next_p = p_depth.saturating_add(1);
                    if next_p < MAX_AST_DEPTH {
                        for sub in part.parts.iter().rev() {
                            part_stack.push((sub, part_scope.clone(), next_hier.clone(), next_p));
                        }
                    }
                }
            }
            for subpkg in pkg.packages.iter().rev() {
                pkg_stack.push((subpkg, pkg_scope.clone(), hier_pkg.clone(), next_depth));
            }
        }
    }
    ScopedAst { pkgs, parts }
}

/// Computes the effective port flow direction (`in`, `out`, `inout`) accounting for conjugation. Realises: [REQ-SEMANTIC-VALIDATOR] | [FLS: fls_66m4rnbssgig]
fn get_effective_direction(port_sym: &Symbol) -> String {
    let dir = port_sym.direction.as_deref().unwrap_or("inout");
    match (port_sym.is_conjugated, dir) {
        (true, "in") => "out".to_string(),
        (true, "out") => "in".to_string(),
        _ => dir.to_string(),
    }
}

/// Normalizes a requirement, hazard, or constraint identifier for case-insensitive matching. Realises: [REQ-SEMANTIC-VALIDATOR] | [FLS: fls_66m4rnbssgig]
fn normalize_id(id: &str) -> String {
    id.trim().replace('-', "_").to_lowercase()
}
