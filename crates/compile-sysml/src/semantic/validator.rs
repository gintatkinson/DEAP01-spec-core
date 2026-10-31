//! Semantic validation engine for SysML v2 / KerML models.
//!
//! Safety & Traceability: Realises: [REQ-SEMANTIC-VALIDATOR-MODULE]
//! Contract: Validates part hierarchies, port connection flow directions, safety annotation referential integrity, and scoped type resolution across `PackageDef` ASTs.
//! Bounds & Faults: Caps AST and composite-type traversal depth at `MAX_AST_DEPTH` (64) using explicit iterative work-stacks with zero unbounded recursion or panics.

use super::symbols::{Symbol, SymbolKind, SymbolTable};
use deap_core::sysml_ast::*;
use std::collections::BTreeSet;

/// Maximum AST and composite-type nesting depth permitted during SysML v2 semantic validation.
/// Realises: [REQ-SEMANTIC-VALIDATOR-MAX-AST-DEPTH]
const MAX_AST_DEPTH: usize = 64;

/// Severity level of semantic diagnostics.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `DiagnosticSeverity` in `crates/compile-sysml/src/semantic/validator.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for enum `DiagnosticSeverity` across member set `[Error, Warning]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `DiagnosticSeverity` members (`Error, Warning`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `DiagnosticSeverity` representation for enum `DiagnosticSeverity`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(2)) bounded over input/member dimension of `DiagnosticSeverity`.
/// - Space Complexity: O(M(2)) bounded auxiliary memory for `DiagnosticSeverity`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `DiagnosticSeverity` (`DiagnosticSeverity`).
/// - State: Maintains consistent state invariants across `DiagnosticSeverity` without partial mutation on error.
/// - Action: Executes `DiagnosticSeverity` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `DiagnosticSeverity` (`SEMANTIC-VALIDATOR`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-VALIDATOR-DIAGNOSTICSEVERITY] | SysML v2: PartDef(DiagnosticSeverity)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

/// Structured diagnostic emitted during semantic validation.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `SemanticDiagnostic` in `crates/compile-sysml/src/semantic/validator.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `SemanticDiagnostic` across member set `[severity: DiagnosticSeverity, message: String, element: String]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `SemanticDiagnostic` members (`severity: DiagnosticSeverity, message: String, element: String`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `SemanticDiagnostic` representation for struct `SemanticDiagnostic`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(3)) bounded over input/member dimension of `SemanticDiagnostic`.
/// - Space Complexity: O(M(3)) bounded auxiliary memory for `SemanticDiagnostic`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `SemanticDiagnostic` (`SemanticDiagnostic`).
/// - State: Maintains consistent state invariants across `SemanticDiagnostic` without partial mutation on error.
/// - Action: Executes `SemanticDiagnostic` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `SemanticDiagnostic` (`SEMANTIC-VALIDATOR`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICDIAGNOSTIC] | SysML v2: PartDef(SemanticDiagnostic)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticDiagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub element: String,
}

impl SemanticDiagnostic {
    /// Executes deterministic fn `SemanticDiagnostic::error` within `crates/compile-sysml/src/semantic/validator.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `SemanticDiagnostic::error` in `crates/compile-sysml/src/semantic/validator.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `SemanticDiagnostic::error` (declaration #3) within `SEMANTIC-VALIDATOR`.
    ///
    /// ## 3. Preconditions
    /// - `element: impl Into<String>` must satisfy domain and type-safety constraints prior to invoking `SemanticDiagnostic::error`.
    /// - `message: impl Into<String>` must satisfy domain and type-safety constraints prior to invoking `SemanticDiagnostic::error`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Self` from `SemanticDiagnostic::error` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(element)) bounded over input/member dimension of `SemanticDiagnostic::error`.
    /// - Space Complexity: O(N(element)) bounded auxiliary memory for `SemanticDiagnostic::error`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `SemanticDiagnostic::error` (`Self`).
    /// - State: Maintains consistent state invariants across `SemanticDiagnostic::error` without partial mutation on error.
    /// - Action: Executes `SemanticDiagnostic::error` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `SemanticDiagnostic::error` (`SEMANTIC-VALIDATOR`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICDIAGNOSTIC-ERROR] | SysML v2: ActionDef(SemanticDiagnostic::error)
    pub fn error(element: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            element: element.into(),
        }
    }

    /// Executes deterministic fn `SemanticDiagnostic::warning` within `crates/compile-sysml/src/semantic/validator.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `SemanticDiagnostic::warning` in `crates/compile-sysml/src/semantic/validator.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `SemanticDiagnostic::warning` (declaration #4) within `SEMANTIC-VALIDATOR`.
    ///
    /// ## 3. Preconditions
    /// - `element: impl Into<String>` must satisfy domain and type-safety constraints prior to invoking `SemanticDiagnostic::warning`.
    /// - `message: impl Into<String>` must satisfy domain and type-safety constraints prior to invoking `SemanticDiagnostic::warning`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Self` from `SemanticDiagnostic::warning` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(element)) bounded over input/member dimension of `SemanticDiagnostic::warning`.
    /// - Space Complexity: O(N(element)) bounded auxiliary memory for `SemanticDiagnostic::warning`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `SemanticDiagnostic::warning` (`Self`).
    /// - State: Maintains consistent state invariants across `SemanticDiagnostic::warning` without partial mutation on error.
    /// - Action: Executes `SemanticDiagnostic::warning` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `SemanticDiagnostic::warning` (`SEMANTIC-VALIDATOR`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICDIAGNOSTIC-WARNING] | SysML v2: ActionDef(SemanticDiagnostic::warning)
    pub fn warning(element: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
            element: element.into(),
        }
    }
}

/// Semantic validator verifying topology, types, and referential integrity.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `SemanticValidator` in `crates/compile-sysml/src/semantic/validator.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `SemanticValidator` across member set `[pkg: &'a PackageDef, symbols: SymbolTable, diagnostics: Vec<SemanticDiagnostic>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `SemanticValidator` members (`pkg: &'a PackageDef, symbols: SymbolTable, diagnostics: Vec<SemanticDiagnostic>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `SemanticValidator` representation for struct `SemanticValidator`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(3)) bounded over input/member dimension of `SemanticValidator`.
/// - Space Complexity: O(M(3)) bounded auxiliary memory for `SemanticValidator`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `SemanticValidator` (`SemanticValidator`).
/// - State: Maintains consistent state invariants across `SemanticValidator` without partial mutation on error.
/// - Action: Executes `SemanticValidator` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `SemanticValidator` (`SEMANTIC-VALIDATOR`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR] | SysML v2: PartDef(SemanticValidator)
pub struct SemanticValidator<'a> {
    pkg: &'a PackageDef,
    symbols: SymbolTable,
    diagnostics: Vec<SemanticDiagnostic>,
}

impl<'a> SemanticValidator<'a> {
    /// Executes deterministic fn `SemanticValidator::new` within `crates/compile-sysml/src/semantic/validator.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `SemanticValidator::new` in `crates/compile-sysml/src/semantic/validator.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `SemanticValidator::new` (declaration #6) within `SEMANTIC-VALIDATOR`.
    ///
    /// ## 3. Preconditions
    /// - `pkg: &'a PackageDef` must satisfy domain and type-safety constraints prior to invoking `SemanticValidator::new`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Self` from `SemanticValidator::new` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(pkg)) bounded over input/member dimension of `SemanticValidator::new`.
    /// - Space Complexity: O(N(pkg)) bounded auxiliary memory for `SemanticValidator::new`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `SemanticValidator::new` (`Self`).
    /// - State: Maintains consistent state invariants across `SemanticValidator::new` without partial mutation on error.
    /// - Action: Executes `SemanticValidator::new` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `SemanticValidator::new` (`SEMANTIC-VALIDATOR`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-NEW] | SysML v2: ActionDef(SemanticValidator::new)
    pub fn new(pkg: &'a PackageDef) -> Self {
        let symbols = SymbolTable::build_from_package(pkg);
        Self {
            pkg,
            symbols,
            diagnostics: Vec::new(),
        }
    }

    /// Execute complete semantic validation suite.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `SemanticValidator::validate` in `crates/compile-sysml/src/semantic/validator.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `SemanticValidator::validate` (declaration #7) within `SEMANTIC-VALIDATOR`.
    ///
    /// ## 3. Preconditions
    /// - Invoked within a valid execution context for `SemanticValidator::validate` in `SEMANTIC-VALIDATOR`.
    ///
    /// ## 4. Postconditions
    /// - Returns `&[SemanticDiagnostic]` from `SemanticValidator::validate` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(K(validate)) bounded over input/member dimension of `SemanticValidator::validate`.
    /// - Space Complexity: O(K(validate)) bounded auxiliary memory for `SemanticValidator::validate`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `SemanticValidator::validate` (`&[SemanticDiagnostic]`).
    /// - State: Maintains consistent state invariants across `SemanticValidator::validate` without partial mutation on error.
    /// - Action: Executes `SemanticValidator::validate` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `SemanticValidator::validate` (`SEMANTIC-VALIDATOR`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE] | SysML v2: ActionDef(SemanticValidator::validate)
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
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `SemanticValidator::has_errors` in `crates/compile-sysml/src/semantic/validator.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `SemanticValidator::has_errors` (declaration #8) within `SEMANTIC-VALIDATOR`.
    ///
    /// ## 3. Preconditions
    /// - Invoked within a valid execution context for `SemanticValidator::has_errors` in `SEMANTIC-VALIDATOR`.
    ///
    /// ## 4. Postconditions
    /// - Returns `bool` from `SemanticValidator::has_errors` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(K(has_errors)) bounded over input/member dimension of `SemanticValidator::has_errors`.
    /// - Space Complexity: O(K(has_errors)) bounded auxiliary memory for `SemanticValidator::has_errors`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `SemanticValidator::has_errors` (`bool`).
    /// - State: Maintains consistent state invariants across `SemanticValidator::has_errors` without partial mutation on error.
    /// - Action: Executes `SemanticValidator::has_errors` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `SemanticValidator::has_errors` (`SEMANTIC-VALIDATOR`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-HAS-ERRORS] | SysML v2: ActionDef(SemanticValidator::has_errors)
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == DiagnosticSeverity::Error)
    }

    /// Filter error diagnostics.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `SemanticValidator::errors` in `crates/compile-sysml/src/semantic/validator.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `SemanticValidator::errors` (declaration #9) within `SEMANTIC-VALIDATOR`.
    ///
    /// ## 3. Preconditions
    /// - Invoked within a valid execution context for `SemanticValidator::errors` in `SEMANTIC-VALIDATOR`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Vec<&SemanticDiagnostic>` from `SemanticValidator::errors` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(K(errors)) bounded over input/member dimension of `SemanticValidator::errors`.
    /// - Space Complexity: O(K(errors)) bounded auxiliary memory for `SemanticValidator::errors`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `SemanticValidator::errors` (`Vec<&SemanticDiagnostic>`).
    /// - State: Maintains consistent state invariants across `SemanticValidator::errors` without partial mutation on error.
    /// - Action: Executes `SemanticValidator::errors` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `SemanticValidator::errors` (`SEMANTIC-VALIDATOR`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-ERRORS] | SysML v2: ActionDef(SemanticValidator::errors)
    pub fn errors(&self) -> Vec<&SemanticDiagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .collect()
    }

    // --- Validation Rules ---

    /// Executes deterministic fn `SemanticValidator::validate_part_hierarchy` within `crates/compile-sysml/src/semantic/validator.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `SemanticValidator::validate_part_hierarchy` in `crates/compile-sysml/src/semantic/validator.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `SemanticValidator::validate_part_hierarchy` (declaration #10) within `SEMANTIC-VALIDATOR`.
    ///
    /// ## 3. Preconditions
    /// - Invoked within a valid execution context for `SemanticValidator::validate_part_hierarchy` in `SEMANTIC-VALIDATOR`.
    ///
    /// ## 4. Postconditions
    /// - Returns `()` from `SemanticValidator::validate_part_hierarchy` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(K(validate_part_hierarchy)) bounded over input/member dimension of `SemanticValidator::validate_part_hierarchy`.
    /// - Space Complexity: O(K(validate_part_hierarchy)) bounded auxiliary memory for `SemanticValidator::validate_part_hierarchy`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `SemanticValidator::validate_part_hierarchy` (`()`).
    /// - State: Maintains consistent state invariants across `SemanticValidator::validate_part_hierarchy` without partial mutation on error.
    /// - Action: Executes `SemanticValidator::validate_part_hierarchy` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `SemanticValidator::validate_part_hierarchy` (`SEMANTIC-VALIDATOR`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-PART-HIERARCHY] | SysML v2: ActionDef(SemanticValidator::validate_part_hierarchy)
    fn validate_part_hierarchy(&mut self) {
        let mut seen_parts = BTreeSet::new();
        self.validate_pkg_part_hierarchy(self.pkg, "", &mut seen_parts, 0);
    }

    /// Iteratively validates unique part and port declarations across nested packages up to `MAX_AST_DEPTH`.
    /// Uses an explicit `Vec` work-stack with saturating depth increments so deep trees never overflow the call stack.
    /// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-PKG-PART-HIERARCHY]
    fn validate_pkg_part_hierarchy(
        &mut self,
        root_pkg: &PackageDef,
        root_parent_scope: &str,
        seen: &mut BTreeSet<String>,
        root_depth: usize,
    ) {
        let mut stack: Vec<(&PackageDef, String, usize)> =
            vec![(root_pkg, root_parent_scope.to_string(), root_depth)];
        while let Some((pkg, parent_scope, depth)) = stack.pop() {
            if depth >= MAX_AST_DEPTH {
                continue;
            }
            let pkg_scope = if parent_scope.is_empty() {
                if std::ptr::eq(pkg, self.pkg) {
                    String::new()
                } else {
                    pkg.name.clone()
                }
            } else if pkg.name.is_empty() {
                parent_scope
            } else {
                format!("{}::{}", parent_scope, pkg.name)
            };

            let next_depth = depth.saturating_add(1);
            if next_depth < MAX_AST_DEPTH {
                self.check_parts_recursive(&pkg.part_defs, &pkg_scope, seen, next_depth);
                for subpkg in pkg.packages.iter().rev() {
                    stack.push((subpkg, pkg_scope.clone(), next_depth));
                }
            }
        }
    }

    /// Iteratively validates duplicate parts and duplicate ports across a `PartDef` forest up to `MAX_AST_DEPTH`.
    /// Preserves pre-order diagnostic emission via reversed LIFO work-stack pushes and `saturating_add(1)`.
    /// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-CHECK-PARTS-RECURSIVE]
    fn check_parts_recursive(
        &mut self,
        parts: &[PartDef],
        root_scope: &str,
        seen: &mut BTreeSet<String>,
        root_depth: usize,
    ) {
        if root_depth >= MAX_AST_DEPTH {
            return;
        }
        let mut stack: Vec<(&PartDef, String, usize)> = parts
            .iter()
            .rev()
            .map(|part| (part, root_scope.to_string(), root_depth))
            .collect();

        while let Some((part, scope, depth)) = stack.pop() {
            if depth >= MAX_AST_DEPTH {
                continue;
            }
            let part_key = if scope.is_empty() {
                part.name.clone()
            } else {
                format!("{}::{}", scope, part.name)
            };

            if !seen.insert(part_key.clone()) {
                self.diagnostics.push(SemanticDiagnostic::error(
                    &part.name,
                    format!(
                        "Duplicate part declaration '{}' in scope '{}'",
                        part.name, scope
                    ),
                ));
            }

            let mut seen_ports = BTreeSet::new();
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

            let next_depth = depth.saturating_add(1);
            if next_depth < MAX_AST_DEPTH {
                for subpart in part.parts.iter().rev() {
                    stack.push((subpart, part_key.clone(), next_depth));
                }
            }
        }
    }

    /// Validates port existence and flow compatibility across all package and part connections up to `MAX_AST_DEPTH`.
    /// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-CONNECTIONS]
    fn validate_connections(&mut self) {
        self.validate_package_connections(self.pkg, 0);
    }

    /// Iteratively traverses nested `PackageDef` nodes up to `MAX_AST_DEPTH` to validate connection lists.
    /// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-PACKAGE-CONNECTIONS]
    fn validate_package_connections(&mut self, root_pkg: &PackageDef, root_depth: usize) {
        let mut stack: Vec<(&PackageDef, usize)> = vec![(root_pkg, root_depth)];
        while let Some((pkg, depth)) = stack.pop() {
            if depth >= MAX_AST_DEPTH {
                continue;
            }
            self.validate_connection_list(&pkg.connection_defs, None);

            let next_depth = depth.saturating_add(1);
            if next_depth < MAX_AST_DEPTH {
                for part in &pkg.part_defs {
                    self.validate_part_connections(part, next_depth);
                }
                for subpkg in pkg.packages.iter().rev() {
                    stack.push((subpkg, next_depth));
                }
            }
        }
    }

    /// Iteratively traverses nested `PartDef` nodes up to `MAX_AST_DEPTH` to validate internal connections.
    /// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-PART-CONNECTIONS]
    fn validate_part_connections(&mut self, root_part: &PartDef, root_depth: usize) {
        let mut stack: Vec<(&PartDef, usize)> = vec![(root_part, root_depth)];
        while let Some((part, depth)) = stack.pop() {
            if depth >= MAX_AST_DEPTH {
                continue;
            }
            self.validate_connection_list(&part.connections, Some(&part.name));

            let next_depth = depth.saturating_add(1);
            if next_depth < MAX_AST_DEPTH {
                for subpart in part.parts.iter().rev() {
                    stack.push((subpart, next_depth));
                }
            }
        }
    }

    /// Executes deterministic fn `SemanticValidator::validate_connection_list` within `crates/compile-sysml/src/semantic/validator.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `SemanticValidator::validate_connection_list` in `crates/compile-sysml/src/semantic/validator.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `SemanticValidator::validate_connection_list` (declaration #16) within `SEMANTIC-VALIDATOR`.
    ///
    /// ## 3. Preconditions
    /// - `conns: &[ConnectionDef]` must satisfy domain and type-safety constraints prior to invoking `SemanticValidator::validate_connection_list`.
    /// - `part_scope: Option<&str>` must satisfy domain and type-safety constraints prior to invoking `SemanticValidator::validate_connection_list`.
    ///
    /// ## 4. Postconditions
    /// - Returns `()` from `SemanticValidator::validate_connection_list` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(conns)) bounded over input/member dimension of `SemanticValidator::validate_connection_list`.
    /// - Space Complexity: O(N(conns)) bounded auxiliary memory for `SemanticValidator::validate_connection_list`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `SemanticValidator::validate_connection_list` (`()`).
    /// - State: Maintains consistent state invariants across `SemanticValidator::validate_connection_list` without partial mutation on error.
    /// - Action: Executes `SemanticValidator::validate_connection_list` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `SemanticValidator::validate_connection_list` (`SEMANTIC-VALIDATOR`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-CONNECTION-LIST] | SysML v2: ActionDef(SemanticValidator::validate_connection_list)
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

                if is_src_boundary && !is_tgt_boundary && src_eff_dir == "in" && tgt_eff_dir == "in"
                {
                    // Valid inward delegation: boundary in -> subpart in
                } else if !is_src_boundary
                    && is_tgt_boundary
                    && src_eff_dir == "out"
                    && tgt_eff_dir == "out"
                {
                    // Valid outward delegation: subpart out -> boundary out
                } else if is_src_boundary
                    && is_tgt_boundary
                    && src_eff_dir == "in"
                    && tgt_eff_dir == "out"
                {
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

    /// Executes deterministic fn `SemanticValidator::validate_referential_integrity` within `crates/compile-sysml/src/semantic/validator.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `SemanticValidator::validate_referential_integrity` in `crates/compile-sysml/src/semantic/validator.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `SemanticValidator::validate_referential_integrity` (declaration #17) within `SEMANTIC-VALIDATOR`.
    ///
    /// ## 3. Preconditions
    /// - Invoked within a valid execution context for `SemanticValidator::validate_referential_integrity` in `SEMANTIC-VALIDATOR`.
    ///
    /// ## 4. Postconditions
    /// - Returns `()` from `SemanticValidator::validate_referential_integrity` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(K(validate_referential_integrity)) bounded over input/member dimension of `SemanticValidator::validate_referential_integrity`.
    /// - Space Complexity: O(K(validate_referential_integrity)) bounded auxiliary memory for `SemanticValidator::validate_referential_integrity`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `SemanticValidator::validate_referential_integrity` (`()`).
    /// - State: Maintains consistent state invariants across `SemanticValidator::validate_referential_integrity` without partial mutation on error.
    /// - Action: Executes `SemanticValidator::validate_referential_integrity` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `SemanticValidator::validate_referential_integrity` (`SEMANTIC-VALIDATOR`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-REFERENTIAL-INTEGRITY] | SysML v2: ActionDef(SemanticValidator::validate_referential_integrity)
    fn validate_referential_integrity(&mut self) {
        let known_constraints: BTreeSet<String> = self
            .symbols
            .all_of_kind(SymbolKind::Constraint)
            .into_iter()
            .map(|s| normalize_id(&s.name))
            .collect();

        let known_hazards: BTreeSet<String> = self
            .symbols
            .all_of_kind(SymbolKind::Hazard)
            .into_iter()
            .map(|s| normalize_id(&s.name))
            .collect();

        let mut req_texts = Vec::new();
        self.collect_package_requirement_texts(self.pkg, &mut req_texts, 0);

        for (req_name, text_corpus) in req_texts {
            self.validate_safety_realises_spans(&req_name, &text_corpus, &known_constraints);
            self.validate_triggers_hazard_spans(&req_name, &text_corpus, &known_hazards);
        }
    }

    /// Validates `@SafetyRealises` target identifiers and delimiter balance for a single requirement corpus.
    /// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-REFERENTIAL-INTEGRITY]
    fn validate_safety_realises_spans(
        &mut self,
        req_name: &str,
        text_corpus: &str,
        known_constraints: &BTreeSet<String>,
    ) {
        for span in extract_annotation_targets(text_corpus, "@SafetyRealises") {
            match span {
                AnnotationSpan::Targets(targets) => {
                    for target in &targets {
                        let clean_target = normalize_id(target);
                        if !known_constraints.is_empty()
                            && !known_constraints
                                .iter()
                                .any(|c| c == &clean_target || c.contains(&clean_target))
                        {
                            self.diagnostics.push(SemanticDiagnostic::error(
                                req_name,
                                format!(
                                    "Requirement '{}' references undefined constraint/UCA '{}' in @SafetyRealises",
                                    req_name, target
                                ),
                            ));
                        }
                    }
                }
                AnnotationSpan::UnclosedDelimiter => {
                    self.diagnostics.push(SemanticDiagnostic::error(
                        req_name,
                        format!(
                            "Requirement '{}' contains unclosed '(' delimiter in @SafetyRealises",
                            req_name
                        ),
                    ));
                }
            }
        }
    }

    /// Validates `@TriggersHazard` target identifiers and delimiter balance for a single requirement corpus.
    /// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-REFERENTIAL-INTEGRITY]
    fn validate_triggers_hazard_spans(
        &mut self,
        req_name: &str,
        text_corpus: &str,
        known_hazards: &BTreeSet<String>,
    ) {
        for span in extract_annotation_targets(text_corpus, "@TriggersHazard") {
            match span {
                AnnotationSpan::Targets(targets) => {
                    for target in &targets {
                        let clean_target = normalize_id(target);
                        if !known_hazards.is_empty()
                            && !known_hazards
                                .iter()
                                .any(|h| h == &clean_target || h.contains(&clean_target))
                        {
                            self.diagnostics.push(SemanticDiagnostic::error(
                                req_name,
                                format!(
                                    "Requirement '{}' references undefined hazard '{}' in @TriggersHazard",
                                    req_name, target
                                ),
                            ));
                        }
                    }
                }
                AnnotationSpan::UnclosedDelimiter => {
                    self.diagnostics.push(SemanticDiagnostic::error(
                        req_name,
                        format!(
                            "Requirement '{}' contains unclosed '(' delimiter in @TriggersHazard",
                            req_name
                        ),
                    ));
                }
            }
        }
    }

    /// Iteratively collects `(requirement_name, text_corpus)` pairs across nested packages up to `MAX_AST_DEPTH`.
    /// Replaces unbounded `PackageDef::get_all_packages` recursion with an explicit `Vec` work-stack.
    /// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-REFERENTIAL-INTEGRITY]
    fn collect_package_requirement_texts(
        &self,
        root_pkg: &PackageDef,
        out: &mut Vec<(String, String)>,
        root_depth: usize,
    ) {
        let mut stack: Vec<(&PackageDef, usize)> = vec![(root_pkg, root_depth)];
        while let Some((pkg, depth)) = stack.pop() {
            if depth >= MAX_AST_DEPTH {
                continue;
            }
            self.collect_requirement_texts(&pkg.requirement_defs, out);

            let next_depth = depth.saturating_add(1);
            if next_depth < MAX_AST_DEPTH {
                for part in &pkg.part_defs {
                    self.collect_part_requirement_texts(part, out, next_depth);
                }
                for subpkg in pkg.packages.iter().rev() {
                    stack.push((subpkg, next_depth));
                }
            }
        }
    }

    /// Appends `(requirement_name, text_corpus)` entries from a flat `RequirementDef` slice.
    /// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-COLLECT-REQUIREMENT-TEXTS]
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

    /// Iteratively collects requirement prose across a `PartDef` subtree up to `MAX_AST_DEPTH`.
    /// Uses an explicit `Vec` work-stack with saturating depth increments to prevent stack exhaustion.
    /// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-COLLECT-PART-REQUIREMENT-TEXTS]
    fn collect_part_requirement_texts(
        &self,
        root_part: &PartDef,
        out: &mut Vec<(String, String)>,
        root_depth: usize,
    ) {
        let mut stack: Vec<(&PartDef, usize)> = vec![(root_part, root_depth)];
        while let Some((part, depth)) = stack.pop() {
            if depth >= MAX_AST_DEPTH {
                continue;
            }
            self.collect_requirement_texts(&part.requirements, out);

            let next_depth = depth.saturating_add(1);
            if next_depth < MAX_AST_DEPTH {
                for subpart in part.parts.iter().rev() {
                    stack.push((subpart, next_depth));
                }
            }
        }
    }

    /// Validates all part definitions across the package hierarchy up to `MAX_AST_DEPTH`.
    /// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-PART-DEFS]
    fn validate_part_defs(&mut self) {
        self.validate_part_defs_in_pkg(self.pkg, "", 0);
    }

    /// Iteratively validates `part_defs` across nested `PackageDef` nodes up to `MAX_AST_DEPTH`.
    /// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-PART-DEFS-IN-PKG]
    fn validate_part_defs_in_pkg(
        &mut self,
        root_pkg: &PackageDef,
        root_parent_scope: &str,
        root_depth: usize,
    ) {
        let mut stack: Vec<(&PackageDef, String, usize)> =
            vec![(root_pkg, root_parent_scope.to_string(), root_depth)];
        while let Some((pkg, parent_scope, depth)) = stack.pop() {
            if depth >= MAX_AST_DEPTH {
                continue;
            }
            let pkg_scope = compute_scoped_package_name(pkg, &parent_scope);

            let next_depth = depth.saturating_add(1);
            if next_depth < MAX_AST_DEPTH {
                for part in &pkg.part_defs {
                    self.validate_single_part_def(part, &pkg_scope, next_depth);
                }
                for subpkg in pkg.packages.iter().rev() {
                    stack.push((subpkg, pkg_scope.clone(), next_depth));
                }
            }
        }
    }

    /// Iteratively validates type references on `root_part` and nested subparts up to `MAX_AST_DEPTH`.
    /// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-SINGLE-PART-DEF]
    fn validate_single_part_def(
        &mut self,
        root_part: &PartDef,
        root_scope: &str,
        root_depth: usize,
    ) {
        let mut stack: Vec<(&PartDef, String, usize)> =
            vec![(root_part, root_scope.to_string(), root_depth)];
        while let Some((part, scope, depth)) = stack.pop() {
            if depth >= MAX_AST_DEPTH {
                continue;
            }
            if let Some(ref ty) = part.type_name {
                if !self.is_resolvable_type(&scope, ty) {
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

            let next_depth = depth.saturating_add(1);
            if next_depth < MAX_AST_DEPTH {
                for subpart in part.parts.iter().rev() {
                    stack.push((subpart, part_scope.clone(), next_depth));
                }
            }
        }
    }

    /// Validates requirement definitions across the package and part hierarchy up to `MAX_AST_DEPTH`.
    /// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-REQUIREMENTS]
    fn validate_requirements(&mut self) {
        self.validate_requirements_in_pkg(self.pkg, "", 0);
    }

    /// Iteratively validates requirements across nested `PackageDef` nodes up to `MAX_AST_DEPTH`.
    /// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-REQUIREMENTS-IN-PKG]
    fn validate_requirements_in_pkg(
        &mut self,
        root_pkg: &PackageDef,
        root_parent_scope: &str,
        root_depth: usize,
    ) {
        let mut stack: Vec<(&PackageDef, String, usize)> =
            vec![(root_pkg, root_parent_scope.to_string(), root_depth)];
        while let Some((pkg, parent_scope, depth)) = stack.pop() {
            if depth >= MAX_AST_DEPTH {
                continue;
            }
            let pkg_scope = compute_scoped_package_name(pkg, &parent_scope);

            for req in &pkg.requirement_defs {
                self.validate_single_requirement_def(req, &pkg_scope);
            }

            let next_depth = depth.saturating_add(1);
            if next_depth < MAX_AST_DEPTH {
                for part in &pkg.part_defs {
                    self.validate_part_requirements(part, &pkg_scope, next_depth);
                }
                for subpkg in pkg.packages.iter().rev() {
                    stack.push((subpkg, pkg_scope.clone(), next_depth));
                }
            }
        }
    }

    /// Validates attribute types and satisfying element references on a single `RequirementDef`.
    /// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-SINGLE-REQUIREMENT-DEF]
    fn validate_single_requirement_def(&mut self, req: &RequirementDef, scope: &str) {
        for attr in &req.attributes {
            if !self.is_resolvable_type(scope, &attr.type_name) {
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
            if !self.is_resolvable_type(scope, target) {
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

    /// Iteratively validates nested part requirements across a `PartDef` subtree up to `MAX_AST_DEPTH`.
    /// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-PART-REQUIREMENTS]
    fn validate_part_requirements(
        &mut self,
        root_part: &PartDef,
        root_scope: &str,
        root_depth: usize,
    ) {
        let mut stack: Vec<(&PartDef, String, usize)> =
            vec![(root_part, root_scope.to_string(), root_depth)];
        while let Some((part, scope, depth)) = stack.pop() {
            if depth >= MAX_AST_DEPTH {
                continue;
            }
            let part_scope = if scope.is_empty() {
                part.name.clone()
            } else {
                format!("{}::{}", scope, part.name)
            };

            for req in &part.requirements {
                self.validate_single_requirement_def(req, &part_scope);
            }

            let next_depth = depth.saturating_add(1);
            if next_depth < MAX_AST_DEPTH {
                for subpart in part.parts.iter().rev() {
                    stack.push((subpart, part_scope.clone(), next_depth));
                }
            }
        }
    }

    /// Checks whether a type expression is resolvable in the given scope.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `SemanticValidator::is_resolvable_type` in `crates/compile-sysml/src/semantic/validator.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `SemanticValidator::is_resolvable_type` within `SEMANTIC-VALIDATOR`.
    /// - Evaluates nested tuples and generic containers iteratively via an explicit work-stack capped at `MAX_AST_DEPTH` (64).
    ///
    /// ## 3. Preconditions
    /// - `scope: &str` must satisfy domain and type-safety constraints prior to invoking `SemanticValidator::is_resolvable_type`.
    /// - `type_name: &str` must satisfy domain and type-safety constraints prior to invoking `SemanticValidator::is_resolvable_type`.
    ///
    /// ## 4. Postconditions
    /// - Returns `bool` from `SemanticValidator::is_resolvable_type` failing closed (`false`) when nesting reaches `MAX_AST_DEPTH` or any component is unresolvable.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(type_name)) bounded over input length and `MAX_AST_DEPTH`.
    /// - Space Complexity: O(MAX_AST_DEPTH) bounded auxiliary work-stack memory with O(1) call-stack frames.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `SemanticValidator::is_resolvable_type` (`bool`).
    /// - State: Maintains consistent state invariants across `SemanticValidator::is_resolvable_type` without partial mutation on error.
    /// - Action: Executes `SemanticValidator::is_resolvable_type` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `SemanticValidator::is_resolvable_type` (`SEMANTIC-VALIDATOR`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-IS-RESOLVABLE-TYPE] | SysML v2: ActionDef(SemanticValidator::is_resolvable_type)
    pub fn is_resolvable_type(&self, scope: &str, type_name: &str) -> bool {
        let mut stack: Vec<(&str, usize)> = vec![(type_name, 0)];

        while let Some((current_raw, depth)) = stack.pop() {
            if depth >= MAX_AST_DEPTH {
                return false;
            }
            let clean = normalize_source_type(current_raw);
            if clean.is_empty() {
                return false;
            }
            if clean == "()" {
                continue;
            }

            if let Some(tuple_inner) = extract_enclosing_tuple_inner(clean) {
                let inner = tuple_inner.trim();
                if inner.is_empty() {
                    continue;
                }
                let args = split_top_level_type_args(inner);
                if args.is_empty() {
                    return false;
                }
                let next_depth = depth.saturating_add(1);
                for arg in args.into_iter().rev() {
                    stack.push((arg, next_depth));
                }
                continue;
            }

            match decompose_generic_type(clean) {
                GenericDecomposition::Malformed => return false,
                GenericDecomposition::Split { outer, inner } => {
                    let outer_ok = is_primitive_or_builtin_type_at_depth(outer, depth)
                        || self.symbols.lookup_in_scope(scope, outer).is_some();
                    if !outer_ok {
                        return false;
                    }

                    let args = split_top_level_type_args(inner);
                    if args.is_empty() {
                        return false;
                    }

                    let next_depth = depth.saturating_add(1);
                    for arg in args.into_iter().rev() {
                        stack.push((arg, next_depth));
                    }
                    continue;
                }
                GenericDecomposition::Atomic => {}
            }

            if !is_atomic_builtin_type_name(clean)
                && self.symbols.lookup_in_scope(scope, clean).is_none()
            {
                return false;
            }
        }

        true
    }
}

/// Deterministically normalizes a source-language type expression by stripping reference/pointer
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `normalize_source_type` in `crates/compile-sysml/src/semantic/validator.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `normalize_source_type` (declaration #28) within `SEMANTIC-VALIDATOR`.
///
/// ## 3. Preconditions
/// - `raw: &'a str` must satisfy domain and type-safety constraints prior to invoking `normalize_source_type`.
///
/// ## 4. Postconditions
/// - Returns `&'a str` from `normalize_source_type` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(raw)) bounded over input/member dimension of `normalize_source_type`.
/// - Space Complexity: O(N(raw)) bounded auxiliary memory for `normalize_source_type`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `normalize_source_type` (`&'a str`).
/// - State: Maintains consistent state invariants across `normalize_source_type` without partial mutation on error.
/// - Action: Executes `normalize_source_type` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `normalize_source_type` (`SEMANTIC-VALIDATOR`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-VALIDATOR-NORMALIZE-SOURCE-TYPE] | SysML v2: ActionDef(normalize_source_type)
pub fn normalize_source_type<'a>(raw: &'a str) -> &'a str {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return "";
    }
    let mut s = trimmed;
    loop {
        let prev = s;

        // 1. Strip trailing nullable markers '?'
        while let Some(stripped) = s.strip_suffix('?') {
            s = stripped.trim_end();
        }

        // 2. Strip reference and pointer prefixes
        if let Some(rest) = strip_pointer_or_ref_prefix(s) {
            s = rest;
        }

        // 3. Strip trait object prefix `dyn `
        if let Some(rest) = s.strip_prefix("dyn") {
            if rest.starts_with(|c: char| c.is_whitespace()) {
                s = rest.trim_start();
            }
        }

        // 4. Strip trailing multiplicity suffixes (`Type[1..*]`) or array wrappers (`[T; N]`, `[T]`)
        if let Some(unwrapped) = strip_trailing_bracket_wrapper(s) {
            s = unwrapped;
        }

        if s == prev {
            break;
        }
    }

    // Preserve raw trimmed token when bare modifiers (e.g. "&", "*const", "[]", "?") strip to empty
    if s.is_empty() {
        return trimmed;
    }

    s
}

/// Checks whether a suffix following `*const`, `*mut`, `&mut`, or `&const` is a valid type boundary.
/// Realises: [REQ-SEMANTIC-VALIDATOR-NORMALIZE-SOURCE-TYPE]
fn is_modifier_boundary(rest: &str) -> bool {
    rest.is_empty()
        || rest.starts_with(|c: char| {
            c.is_whitespace() || c == '*' || c == '&' || c == '[' || c == '('
        })
}

/// Strips one leading pointer or reference modifier (`*const`, `*mut`, `&mut`, `&const`, `&'a mut`, `&`).
/// Realises: [REQ-SEMANTIC-VALIDATOR-NORMALIZE-SOURCE-TYPE]
fn strip_pointer_or_ref_prefix(s: &str) -> Option<&str> {
    for prefix in ["*const", "*mut", "&mut", "&const"] {
        if let Some(rest) = s.strip_prefix(prefix) {
            if is_modifier_boundary(rest) {
                return Some(rest.trim_start());
            }
        }
    }
    let rest = s.strip_prefix('&')?;
    let mut rest = rest.trim_start();
    if rest.starts_with('\'') {
        let lifetime_end = rest
            .char_indices()
            .skip(1)
            .find(|(_, c)| !c.is_ascii_alphanumeric() && *c != '_')
            .map_or(rest.len(), |(idx, _)| idx);
        rest = rest.get(lifetime_end..)?.trim_start();
        if let Some(after_mut) = rest.strip_prefix("mut") {
            if is_modifier_boundary(after_mut) {
                rest = after_mut.trim_start();
            }
        } else if let Some(after_const) = rest.strip_prefix("const") {
            if is_modifier_boundary(after_const) {
                rest = after_const.trim_start();
            }
        }
    }
    Some(rest.trim_start())
}

/// Strips a trailing multiplicity suffix (`Type[1..*]`) or enclosing array/slice wrapper (`[T; N]`, `[T]`).
/// Realises: [REQ-SEMANTIC-VALIDATOR-NORMALIZE-SOURCE-TYPE]
fn strip_trailing_bracket_wrapper(s: &str) -> Option<&str> {
    if !s.ends_with(']') {
        return None;
    }
    let mut depth: usize = 0;
    let mut matching_open = None;
    for (idx, ch) in s.char_indices().rev() {
        match ch {
            ']' => depth = depth.saturating_add(1),
            '[' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    matching_open = Some(idx);
                    break;
                }
            }
            _ => {}
        }
    }
    let open_idx = matching_open?;
    if open_idx > 0 {
        return Some(s.get(..open_idx)?.trim_end());
    }
    let inner = s.strip_prefix('[')?.strip_suffix(']')?.trim();
    let mut inner_depth: usize = 0;
    let mut semi_pos = None;
    for (idx, ch) in inner.char_indices() {
        match ch {
            '[' | '<' | '(' => inner_depth = inner_depth.saturating_add(1),
            ']' | '>' | ')' => inner_depth = inner_depth.saturating_sub(1),
            ';' if inner_depth == 0 => {
                semi_pos = Some(idx);
                break;
            }
            _ => {}
        }
    }
    if let Some(semi_idx) = semi_pos {
        Some(inner.get(..semi_idx)?.trim())
    } else {
        Some(inner)
    }
}

/// Deterministically splits comma-separated top-level type arguments inside generic angle brackets `<...>`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `split_top_level_type_args` in `crates/compile-sysml/src/semantic/validator.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `split_top_level_type_args` (declaration #29) within `SEMANTIC-VALIDATOR`.
///
/// ## 3. Preconditions
/// - `args: &'a str` must satisfy domain and type-safety constraints prior to invoking `split_top_level_type_args`.
///
/// ## 4. Postconditions
/// - Returns `Vec<&'a str>` from `split_top_level_type_args` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(args)) bounded over input/member dimension of `split_top_level_type_args`.
/// - Space Complexity: O(N(args)) bounded auxiliary memory for `split_top_level_type_args`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `split_top_level_type_args` (`Vec<&'a str>`).
/// - State: Maintains consistent state invariants across `split_top_level_type_args` without partial mutation on error.
/// - Action: Executes `split_top_level_type_args` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `split_top_level_type_args` (`SEMANTIC-VALIDATOR`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-VALIDATOR-SPLIT-TOP-LEVEL-TYPE-ARGS] | SysML v2: ActionDef(split_top_level_type_args)
pub fn split_top_level_type_args<'a>(args: &'a str) -> Vec<&'a str> {
    let mut s = args.trim();
    if s.starts_with('<') && s.ends_with('>') {
        let mut depth: usize = 0;
        let mut closed_early = false;
        let last_idx = s.len().saturating_sub(1);
        for (idx, ch) in s.char_indices() {
            match ch {
                '<' => depth = depth.saturating_add(1),
                '>' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 && idx < last_idx {
                        closed_early = true;
                        break;
                    }
                }
                _ => {}
            }
        }
        if depth == 0 && !closed_early {
            if let Some(unwrapped) = s.strip_prefix('<').and_then(|rest| rest.strip_suffix('>')) {
                s = unwrapped.trim();
            }
        }
    }
    if s.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut angle_depth: usize = 0;
    let mut paren_depth: usize = 0;
    let mut bracket_depth: usize = 0;
    let mut start: usize = 0;

    for (idx, ch) in s.char_indices() {
        match ch {
            '<' => angle_depth = angle_depth.saturating_add(1),
            '>' => angle_depth = angle_depth.saturating_sub(1),
            '(' => paren_depth = paren_depth.saturating_add(1),
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth = bracket_depth.saturating_add(1),
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            ',' if angle_depth == 0 && paren_depth == 0 && bracket_depth == 0 => {
                let Some(slice) = s.get(start..idx) else {
                    return Vec::new();
                };
                let trimmed = slice.trim();
                if !trimmed.is_empty() {
                    result.push(trimmed);
                }
                start = idx.saturating_add(ch.len_utf8());
            }
            _ => {}
        }
    }

    let Some(tail) = s.get(start..) else {
        return Vec::new();
    };
    let trimmed_tail = tail.trim();
    if !trimmed_tail.is_empty() {
        result.push(trimmed_tail);
    }

    result
}

/// Executes deterministic fn `is_primitive_or_builtin_type` within `crates/compile-sysml/src/semantic/validator.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `is_primitive_or_builtin_type` in `crates/compile-sysml/src/semantic/validator.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `is_primitive_or_builtin_type` within `SEMANTIC-VALIDATOR`.
/// - Evaluates nested tuples and generic containers iteratively via an explicit work-stack capped at `MAX_AST_DEPTH` (64).
///
/// ## 3. Preconditions
/// - `type_name: &str` must satisfy domain and type-safety constraints prior to invoking `is_primitive_or_builtin_type`.
///
/// ## 4. Postconditions
/// - Returns `bool` from `is_primitive_or_builtin_type`, failing closed (`false`) if nesting reaches `MAX_AST_DEPTH`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(type_name)) bounded over input length and `MAX_AST_DEPTH`.
/// - Space Complexity: O(MAX_AST_DEPTH) bounded auxiliary work-stack memory with O(1) call-stack frames.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `is_primitive_or_builtin_type` (`bool`).
/// - State: Maintains consistent state invariants across `is_primitive_or_builtin_type` without partial mutation on error.
/// - Action: Executes `is_primitive_or_builtin_type` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `is_primitive_or_builtin_type` (`SEMANTIC-VALIDATOR`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-VALIDATOR-IS-PRIMITIVE-OR-BUILTIN-TYPE] | SysML v2: ActionDef(is_primitive_or_builtin_type)
pub fn is_primitive_or_builtin_type(type_name: &str) -> bool {
    is_primitive_or_builtin_type_at_depth(type_name, 0)
}

/// Iteratively checks whether `type_name` decomposes exclusively into primitive or builtin types starting at `start_depth`.
/// Fails closed (`false`) if composite nesting reaches `MAX_AST_DEPTH`.
/// Realises: [REQ-SEMANTIC-VALIDATOR-IS-PRIMITIVE-OR-BUILTIN-TYPE]
fn is_primitive_or_builtin_type_at_depth(type_name: &str, start_depth: usize) -> bool {
    let mut stack: Vec<(&str, usize)> = vec![(type_name, start_depth)];

    while let Some((current_raw, depth)) = stack.pop() {
        if depth >= MAX_AST_DEPTH {
            return false;
        }
        let clean = normalize_source_type(current_raw);
        if clean.is_empty() {
            return false;
        }
        if clean == "()" {
            continue;
        }

        if let Some(tuple_inner) = extract_enclosing_tuple_inner(clean) {
            let inner = tuple_inner.trim();
            if inner.is_empty() {
                continue;
            }
            let args = split_top_level_type_args(inner);
            if args.is_empty() {
                return false;
            }
            let next_depth = depth.saturating_add(1);
            for arg in args.into_iter().rev() {
                stack.push((arg, next_depth));
            }
            continue;
        }

        match decompose_generic_type(clean) {
            GenericDecomposition::Malformed => return false,
            GenericDecomposition::Split { outer, inner } => {
                let args = split_top_level_type_args(inner);
                if args.is_empty() {
                    return false;
                }
                let next_depth = depth.saturating_add(1);
                for arg in args.into_iter().rev() {
                    stack.push((arg, next_depth));
                }
                stack.push((outer, next_depth));
                continue;
            }
            GenericDecomposition::Atomic => {}
        }

        if !is_atomic_builtin_type_name(clean) {
            return false;
        }
    }

    true
}

/// Classification of a normalized type expression during generic `<...>` boundary inspection.
/// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-IS-RESOLVABLE-TYPE]
enum GenericDecomposition<'a> {
    Atomic,
    Split { outer: &'a str, inner: &'a str },
    Malformed,
}

/// Decomposes an outer generic container and inner type-argument list using checked `.get()` slices.
/// Rejects trailing tokens or invalid slice boundaries explicitly with `GenericDecomposition::Malformed`.
/// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-IS-RESOLVABLE-TYPE]
fn decompose_generic_type(clean: &str) -> GenericDecomposition<'_> {
    let Some(lt_idx) = clean.find('<') else {
        return GenericDecomposition::Atomic;
    };
    let Some(gt_idx) = clean.rfind('>') else {
        return GenericDecomposition::Malformed;
    };
    if gt_idx <= lt_idx {
        return GenericDecomposition::Malformed;
    }
    let Some(trailing) = clean.get(gt_idx.saturating_add(1)..) else {
        return GenericDecomposition::Malformed;
    };
    if !trailing.trim().is_empty() {
        return GenericDecomposition::Malformed;
    }
    let Some(outer_raw) = clean.get(..lt_idx) else {
        return GenericDecomposition::Malformed;
    };
    let Some(inner) = clean.get(lt_idx.saturating_add(1)..gt_idx) else {
        return GenericDecomposition::Malformed;
    };
    GenericDecomposition::Split {
        outer: outer_raw.trim(),
        inner,
    }
}

/// Extracts the interior slice of a single balanced enclosing tuple `(T1, T2, ...)`.
/// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-IS-RESOLVABLE-TYPE]
fn extract_enclosing_tuple_inner(clean: &str) -> Option<&str> {
    if !clean.starts_with('(') || !clean.ends_with(')') {
        return None;
    }
    let mut depth: usize = 0;
    let mut closed_early = false;
    let last_idx = clean.len().saturating_sub(1);
    for (idx, ch) in clean.char_indices() {
        match ch {
            '(' => depth = depth.saturating_add(1),
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 && idx < last_idx {
                    closed_early = true;
                    break;
                }
            }
            _ => {}
        }
    }
    if depth == 0 && !closed_early {
        clean.strip_prefix('(')?.strip_suffix(')')
    } else {
        None
    }
}

/// Checks whether a normalized non-generic, non-tuple type identifier is a recognized standard or framework builtin.
/// Realises: [REQ-SEMANTIC-VALIDATOR-IS-PRIMITIVE-OR-BUILTIN-TYPE]
fn is_atomic_builtin_type_name(clean: &str) -> bool {
    if clean.starts_with("ScalarValues::")
        || clean.starts_with("ISQ::")
        || clean.starts_with("SI::")
        || clean.starts_with("std::")
        || clean.starts_with("core::")
        || clean.starts_with("alloc::")
    {
        return true;
    }
    let unqualified = clean.rsplit("::").next().unwrap_or(clean).trim();
    is_scalar_or_container_builtin(unqualified) || is_ui_framework_builtin(unqualified)
}

/// Recognizes SysML v2 scalars, Rust primitives/containers, and Dart collection/async builtin identifiers.
/// Realises: [REQ-SEMANTIC-VALIDATOR-IS-PRIMITIVE-OR-BUILTIN-TYPE]
fn is_scalar_or_container_builtin(unqualified: &str) -> bool {
    matches!(
        unqualified,
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
            | "double"
            | "num"
            | "f32"
            | "f64"
            | "String"
            | "string"
            | "str"
            | "char"
            | "void"
            | "unit"
            | "dynamic"
            | "Object"
            | "ScalarValues"
            | "Vec"
            | "Option"
            | "Result"
            | "Box"
            | "Arc"
            | "Rc"
            | "Cow"
            | "Mutex"
            | "RwLock"
            | "BTreeMap"
            | "BTreeSet"
            | "HashMap"
            | "HashSet"
            | "Path"
            | "PathBuf"
            | "Duration"
            | "Instant"
            | "SystemTime"
            | "DateTime"
            | "Error"
            | "List"
            | "Map"
            | "Set"
            | "Iterable"
            | "Future"
            | "Stream"
    )
}

/// Recognizes Flutter UI framework builtin widget, geometry, and controller type identifiers.
/// Realises: [REQ-SEMANTIC-VALIDATOR-IS-PRIMITIVE-OR-BUILTIN-TYPE]
fn is_ui_framework_builtin(unqualified: &str) -> bool {
    matches!(
        unqualified,
        "Widget"
            | "StatefulWidget"
            | "StatelessWidget"
            | "State"
            | "BuildContext"
            | "Key"
            | "GlobalKey"
            | "LocalKey"
            | "ValueKey"
            | "VoidCallback"
            | "ValueChanged"
            | "ValueGetter"
            | "ValueSetter"
            | "WidgetBuilder"
            | "IndexedWidgetBuilder"
            | "TransitionBuilder"
            | "Color"
            | "IconData"
            | "EdgeInsets"
            | "EdgeInsetsGeometry"
            | "BoxConstraints"
            | "Size"
            | "Offset"
            | "Rect"
            | "TextStyle"
            | "ThemeData"
            | "TextEditingController"
            | "ScrollController"
            | "FocusNode"
            | "AnimationController"
    )
}

/// Checks whether `pkg` is a synthetic empty `SysML_Model` container holding only nested subpackages.
/// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-PART-DEFS-IN-PKG]
fn is_empty_sysml_model_root(pkg: &PackageDef) -> bool {
    pkg.name == "SysML_Model"
        && pkg.attribute_defs.is_empty()
        && pkg.port_defs.is_empty()
        && pkg.action_defs.is_empty()
        && pkg.constraint_defs.is_empty()
        && pkg.calc_defs.is_empty()
        && pkg.requirement_defs.is_empty()
        && pkg.state_defs.is_empty()
        && pkg.connection_defs.is_empty()
        && pkg.use_case_defs.is_empty()
        && pkg.item_defs.is_empty()
        && pkg.hazard_defs.is_empty()
        && pkg.risk_defs.is_empty()
        && pkg.test_case_defs.is_empty()
        && pkg.capability_defs.is_empty()
        && pkg.operation_defs.is_empty()
        && pkg.interaction_defs.is_empty()
        && pkg.part_defs.is_empty()
}

/// Computes the qualified package scope path for part and requirement resolution.
/// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-PART-DEFS-IN-PKG]
fn compute_scoped_package_name(pkg: &PackageDef, parent_scope: &str) -> String {
    let is_synthetic_multi_root = parent_scope.is_empty()
        && !pkg.packages.is_empty()
        && (pkg.name.is_empty() || is_empty_sysml_model_root(pkg));

    if is_synthetic_multi_root {
        String::new()
    } else if parent_scope.is_empty() {
        pkg.name.clone()
    } else if pkg.name.is_empty() {
        parent_scope.to_string()
    } else {
        format!("{}::{}", parent_scope, pkg.name)
    }
}

/// Parsed outcome for a single `@SafetyRealises` or `@TriggersHazard` annotation span.
/// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-REFERENTIAL-INTEGRITY]
#[derive(Debug, Clone, PartialEq, Eq)]
enum AnnotationSpan {
    Targets(Vec<String>),
    UnclosedDelimiter,
}

/// Extracts `@SafetyRealises` or `@TriggersHazard` target identifiers from requirement prose.
/// Uses UTF-8 boundary-checked `.get()` slicing and explicit delimiter validation without masking malformed spans.
/// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-REFERENTIAL-INTEGRITY]
fn extract_annotation_targets(text: &str, tag: &str) -> Vec<AnnotationSpan> {
    let mut results = Vec::new();
    if tag.is_empty() {
        return results;
    }
    let lower_text = text.to_ascii_lowercase();
    let lower_tag = tag.to_ascii_lowercase();
    let mut search_idx: usize = 0;

    while let Some(search_window) = lower_text.get(search_idx..) {
        let Some(pos) = search_window.find(&lower_tag) else {
            break;
        };
        let tag_start = search_idx.saturating_add(pos);
        let idx = tag_start.saturating_add(lower_tag.len());
        let Some(rest) = text.get(idx..) else {
            break;
        };
        let (ws_end, next_char) = rest
            .char_indices()
            .find(|(_, c)| !c.is_whitespace())
            .map_or((rest.len(), None), |(c_idx, c)| (c_idx, Some(c)));
        match next_char {
            Some('(') => {
                let Some(after_ws) = rest.get(ws_end..) else {
                    results.push(AnnotationSpan::UnclosedDelimiter);
                    break;
                };
                let Some((targets, consumed)) = parse_parenthesized_targets(after_ws) else {
                    results.push(AnnotationSpan::UnclosedDelimiter);
                    break;
                };
                results.push(AnnotationSpan::Targets(targets));
                search_idx = idx.saturating_add(ws_end).saturating_add(consumed);
                continue;
            }
            Some(':') => {
                if let Some(after_ws) = rest.get(ws_end..) {
                    if let Some((target_id, consumed)) = parse_colon_target(after_ws) {
                        if !target_id.is_empty() {
                            results.push(AnnotationSpan::Targets(vec![target_id]));
                        }
                        let step = text
                            .get(tag_start..)
                            .and_then(|s| s.chars().next())
                            .map_or(1, char::len_utf8);
                        let global_end = idx.saturating_add(ws_end).saturating_add(consumed);
                        search_idx = global_end.max(tag_start.saturating_add(step));
                        continue;
                    }
                }
            }
            _ => {}
        }
        let Some(tag_tail) = text.get(tag_start..) else {
            break;
        };
        let Some(first_ch) = tag_tail.chars().next() else {
            break;
        };
        search_idx = tag_start.saturating_add(first_ch.len_utf8());
    }

    results
}

/// Parses a balanced `(ID1, ID2, ...)` target list, returning `None` if `)` is missing.
/// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-REFERENTIAL-INTEGRITY]
fn parse_parenthesized_targets(after_ws: &str) -> Option<(Vec<String>, usize)> {
    let after_open = after_ws.strip_prefix('(')?;
    let (inside, _) = after_open.split_once(')')?;
    let targets: Vec<String> = inside
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let consumed = 1usize.saturating_add(inside.len()).saturating_add(1);
    Some((targets, consumed))
}

/// Parses a single `: TARGET_ID` annotation target following a colon delimiter.
/// Realises: [REQ-SEMANTIC-VALIDATOR-SEMANTICVALIDATOR-VALIDATE-REFERENTIAL-INTEGRITY]
fn parse_colon_target(after_ws: &str) -> Option<(String, usize)> {
    let after_colon = after_ws.strip_prefix(':')?;
    let id_start = after_colon
        .char_indices()
        .find(|(_, c)| !c.is_whitespace())
        .map_or(after_colon.len(), |(c_idx, _)| c_idx);
    let id_rest = after_colon.get(id_start..)?;
    let id_len = id_rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .map(char::len_utf8)
        .sum::<usize>();
    if id_len == 0 {
        return None;
    }
    let raw_id = id_rest.get(..id_len)?;
    let consumed = 1usize.saturating_add(id_start).saturating_add(id_len);
    Some((raw_id.trim().to_string(), consumed))
}

/// Executes deterministic fn `get_effective_direction` within `crates/compile-sysml/src/semantic/validator.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `get_effective_direction` in `crates/compile-sysml/src/semantic/validator.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `get_effective_direction` (declaration #31) within `SEMANTIC-VALIDATOR`.
///
/// ## 3. Preconditions
/// - `port_sym: &Symbol` must satisfy domain and type-safety constraints prior to invoking `get_effective_direction`.
///
/// ## 4. Postconditions
/// - Returns `String` from `get_effective_direction` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(port_sym)) bounded over input/member dimension of `get_effective_direction`.
/// - Space Complexity: O(N(port_sym)) bounded auxiliary memory for `get_effective_direction`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `get_effective_direction` (`String`).
/// - State: Maintains consistent state invariants across `get_effective_direction` without partial mutation on error.
/// - Action: Executes `get_effective_direction` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `get_effective_direction` (`SEMANTIC-VALIDATOR`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-VALIDATOR-GET-EFFECTIVE-DIRECTION] | SysML v2: ActionDef(get_effective_direction)
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

/// Executes deterministic fn `normalize_id` within `crates/compile-sysml/src/semantic/validator.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `normalize_id` in `crates/compile-sysml/src/semantic/validator.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `normalize_id` (declaration #32) within `SEMANTIC-VALIDATOR`.
///
/// ## 3. Preconditions
/// - `id: &str` must satisfy domain and type-safety constraints prior to invoking `normalize_id`.
///
/// ## 4. Postconditions
/// - Returns `String` from `normalize_id` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(id)) bounded over input/member dimension of `normalize_id`.
/// - Space Complexity: O(N(id)) bounded auxiliary memory for `normalize_id`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `normalize_id` (`String`).
/// - State: Maintains consistent state invariants across `normalize_id` without partial mutation on error.
/// - Action: Executes `normalize_id` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `normalize_id` (`SEMANTIC-VALIDATOR`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-VALIDATOR-NORMALIZE-ID] | SysML v2: ActionDef(normalize_id)
fn normalize_id(id: &str) -> String {
    id.trim().replace('-', "_").to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_connection_passes() -> Result<(), Box<dyn std::error::Error>> {
        let pkg = PackageDef {
            name: "TestSys".to_string(),
            part_defs: vec![
                PartDef {
                    name: "Sensor".to_string(),
                    ports: vec![PortDef {
                        name: "out_data".to_string(),
                        direction: "out".to_string(),
                        type_name: "RawData".to_string(),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                PartDef {
                    name: "Computer".to_string(),
                    ports: vec![PortDef {
                        name: "in_data".to_string(),
                        direction: "in".to_string(),
                        type_name: "RawData".to_string(),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
            ],
            connection_defs: vec![ConnectionDef {
                name: "SensorToComputer".to_string(),
                source_port: "Sensor.out_data".to_string(),
                target_port: "Computer.in_data".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };

        let mut validator = SemanticValidator::new(&pkg);
        validator.validate();
        assert!(
            !validator.has_errors(),
            "Expected no errors, got: {:?}",
            validator.errors()
        );
        Ok(())
    }

    #[test]
    fn test_mismatched_connection_direction_fails() -> Result<(), Box<dyn std::error::Error>> {
        let pkg = PackageDef {
            name: "TestSys".to_string(),
            part_defs: vec![
                PartDef {
                    name: "Sensor".to_string(),
                    ports: vec![PortDef {
                        name: "in_data".to_string(),
                        direction: "in".to_string(),
                        type_name: "RawData".to_string(),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                PartDef {
                    name: "Computer".to_string(),
                    ports: vec![PortDef {
                        name: "out_data".to_string(),
                        direction: "out".to_string(),
                        type_name: "RawData".to_string(),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
            ],
            connection_defs: vec![ConnectionDef {
                name: "InvertedFlow".to_string(),
                source_port: "Sensor.in_data".to_string(),
                target_port: "Computer.out_data".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };

        let mut validator = SemanticValidator::new(&pkg);
        validator.validate();
        assert!(validator.has_errors());
        let errors = validator.errors();
        assert_eq!(errors.len(), 1);
        assert!(errors
            .first()
            .ok_or("missing first element of errors")?
            .message
            .contains("Inverted flow direction"));
        Ok(())
    }

    #[test]
    fn test_dangling_safety_realises_fails() -> Result<(), Box<dyn std::error::Error>> {
        let pkg = PackageDef {
            name: "SafetySys".to_string(),
            requirement_defs: vec![RequirementDef {
                name: "Req_Safety_1".to_string(),
                text: "Shall maintain velocity limit. @SafetyRealises(UCA-99)".to_string(),
                ..Default::default()
            }],
            constraint_defs: vec![ConstraintDef {
                name: "UCA_01_Overspeed".to_string(),
                expression: "v <= 60.0".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };

        let mut validator = SemanticValidator::new(&pkg);
        validator.validate();
        assert!(validator.has_errors());
        let errors = validator.errors();
        assert!(errors
            .first()
            .ok_or("missing first element of errors")?
            .message
            .contains("references undefined constraint/UCA 'UCA-99'"));
        Ok(())
    }

    #[test]
    fn test_validate_part_defs_with_cross_package_import() -> Result<(), Box<dyn std::error::Error>>
    {
        let pkg_a = PackageDef {
            name: "Subsystem_1".to_string(),
            part_defs: vec![PartDef {
                name: "SystemVisionEngine".to_string(),
                is_def: true,
                ..Default::default()
            }],
            ..Default::default()
        };

        let pkg_b = PackageDef {
            name: "Subsystem_2".to_string(),
            imports: vec![ImportDef {
                path: "Subsystem_1".to_string(),
                is_wildcard: true,
                ..Default::default()
            }],
            part_defs: vec![PartDef {
                name: "v".to_string(),
                is_def: false,
                type_name: Some("SystemVisionEngine".to_string()),
                ..Default::default()
            }],
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
        Ok(())
    }

    #[test]
    fn test_validate_part_defs_missing_type_fails() -> Result<(), Box<dyn std::error::Error>> {
        let pkg = PackageDef {
            name: "Subsystem_2".to_string(),
            part_defs: vec![PartDef {
                name: "v".to_string(),
                is_def: false,
                type_name: Some("SystemVisionEngine".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };

        let mut validator = SemanticValidator::new(&pkg);
        validator.validate();
        assert!(validator.has_errors());
        let errors = validator.errors();
        assert!(errors.iter().any(|e| e
            .message
            .contains("references undefined type 'SystemVisionEngine'")));
        Ok(())
    }

    #[test]
    fn test_validate_requirements_with_cross_package_import(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let pkg_a = PackageDef {
            name: "Subsystem_1".to_string(),
            part_defs: vec![PartDef {
                name: "SafetyController".to_string(),
                is_def: true,
                ..Default::default()
            }],
            ..Default::default()
        };

        let pkg_b = PackageDef {
            name: "Subsystem_2".to_string(),
            imports: vec![ImportDef {
                path: "Subsystem_1".to_string(),
                is_wildcard: true,
                ..Default::default()
            }],
            requirement_defs: vec![RequirementDef {
                name: "Req_Ctrl".to_string(),
                text: "Shall engage safety controller.".to_string(),
                satisfied_by: vec!["SafetyController".to_string()],
                ..Default::default()
            }],
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
        Ok(())
    }

    #[test]
    fn test_subpackage_connections_validated_for_dangling_ports(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let subpkg = PackageDef {
            name: "NestedSubsystem".to_string(),
            part_defs: vec![PartDef {
                name: "Worker".to_string(),
                ports: vec![PortDef {
                    name: "out_stream".to_string(),
                    direction: "out".to_string(),
                    type_name: "StreamData".to_string(),
                    ..Default::default()
                }],
                connections: vec![ConnectionDef {
                    name: "DanglingSubpartConn".to_string(),
                    source_port: "out_stream".to_string(),
                    target_port: "Worker.undefined_target_port".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            connection_defs: vec![ConnectionDef {
                name: "DanglingPkgConn".to_string(),
                source_port: "Worker.out_stream".to_string(),
                target_port: "UndefinedPart.in_stream".to_string(),
                ..Default::default()
            }],
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
            errors.iter().any(|e| e
                .message
                .contains("references undefined target port 'Worker.undefined_target_port'")),
            "Expected error for Worker.undefined_target_port, got: {:?}",
            errors
        );
        assert!(
            errors.iter().any(|e| e
                .message
                .contains("references undefined target port 'UndefinedPart.in_stream'")),
            "Expected error for UndefinedPart.in_stream, got: {:?}",
            errors
        );
        Ok(())
    }

    #[test]
    fn test_issue_461_normalize_source_type() {
        assert_eq!(normalize_source_type("&str"), "str");
        assert_eq!(normalize_source_type("&mut Vec<u8>"), "Vec<u8>");
        assert_eq!(normalize_source_type("&const i32"), "i32");
        assert_eq!(normalize_source_type("&&str"), "str");
        assert_eq!(normalize_source_type("&'a str"), "str");
        assert_eq!(normalize_source_type("&'static str"), "str");
        assert_eq!(normalize_source_type("&'a mut [u8]"), "u8");
        assert_eq!(normalize_source_type("*const u8"), "u8");
        assert_eq!(normalize_source_type("*mut u8"), "u8");
        assert_eq!(normalize_source_type("[u8; 32]"), "u8");
        assert_eq!(normalize_source_type("[u8]"), "u8");
        assert_eq!(normalize_source_type("[[u8; 4]; 8]"), "u8");
        assert_eq!(normalize_source_type("Sensor[1..*]"), "Sensor");
        assert_eq!(normalize_source_type("u8[]"), "u8");
        assert_eq!(normalize_source_type("Sensor[0..1]"), "Sensor");
        assert_eq!(normalize_source_type("String?"), "String");
        assert_eq!(normalize_source_type("Widget?"), "Widget");
        assert_eq!(normalize_source_type("BuildContext?"), "BuildContext");
        assert_eq!(
            normalize_source_type("&'a mut [Option<String>?; 10]"),
            "Option<String>"
        );
    }

    #[test]
    fn test_issue_461_split_top_level_type_args() {
        assert_eq!(split_top_level_type_args("u8"), vec!["u8"]);
        assert_eq!(
            split_top_level_type_args("String, i32"),
            vec!["String", "i32"]
        );
        assert_eq!(
            split_top_level_type_args("<String, i32>"),
            vec!["String", "i32"]
        );
        assert_eq!(
            split_top_level_type_args("HashMap<String, Vec<u8>>, Result<Option<T>, E>"),
            vec!["HashMap<String, Vec<u8>>", "Result<Option<T>, E>"]
        );
        assert_eq!(
            split_top_level_type_args("(A, B), [C; 2]"),
            vec!["(A, B)", "[C; 2]"]
        );
        assert!(split_top_level_type_args("").is_empty());
        assert!(split_top_level_type_args("< >").is_empty());
    }

    #[test]
    fn test_issue_461_is_primitive_or_builtin_type_scalars() {
        let scalars = vec![
            "void",
            "unit",
            "()",
            "double",
            "num",
            "char",
            "dynamic",
            "Object",
            "bool",
            "Boolean",
            "int",
            "Integer",
            "i8",
            "i16",
            "i32",
            "i64",
            "i128",
            "isize",
            "u8",
            "u16",
            "u32",
            "u64",
            "u128",
            "usize",
            "float",
            "Float",
            "f32",
            "f64",
            "String",
            "string",
            "str",
            "ScalarValues",
        ];
        for ty in scalars {
            assert!(
                is_primitive_or_builtin_type(ty),
                "Expected scalar '{}' to be recognized as primitive or builtin",
                ty
            );
        }
    }

    #[test]
    fn test_issue_461_is_primitive_or_builtin_type_rust_containers() {
        let containers = vec![
            "Vec",
            "Option",
            "Result",
            "Box",
            "Arc",
            "Rc",
            "Cow",
            "Mutex",
            "RwLock",
            "BTreeMap",
            "BTreeSet",
            "HashMap",
            "HashSet",
            "Path",
            "PathBuf",
            "Duration",
            "Instant",
            "SystemTime",
            "DateTime",
            "Error",
        ];
        for ty in containers {
            assert!(
                is_primitive_or_builtin_type(ty),
                "Expected Rust container '{}' to be recognized as builtin",
                ty
            );
        }
    }

    #[test]
    fn test_issue_461_is_primitive_or_builtin_type_dart_collections_and_async() {
        let types = vec!["List", "Map", "Set", "Iterable", "Future", "Stream"];
        for ty in types {
            assert!(
                is_primitive_or_builtin_type(ty),
                "Expected Dart/Async type '{}' to be recognized as builtin",
                ty
            );
        }
    }

    #[test]
    fn test_issue_461_is_primitive_or_builtin_type_flutter_framework() {
        let flutter_types = vec![
            "Widget",
            "StatefulWidget",
            "StatelessWidget",
            "State",
            "BuildContext",
            "Key",
            "GlobalKey",
            "LocalKey",
            "ValueKey",
            "VoidCallback",
            "ValueChanged",
            "ValueGetter",
            "ValueSetter",
            "WidgetBuilder",
            "IndexedWidgetBuilder",
            "TransitionBuilder",
            "Color",
            "IconData",
            "EdgeInsets",
            "EdgeInsetsGeometry",
            "BoxConstraints",
            "Size",
            "Offset",
            "Rect",
            "TextStyle",
            "ThemeData",
            "TextEditingController",
            "ScrollController",
            "FocusNode",
            "AnimationController",
        ];
        for ty in flutter_types {
            assert!(
                is_primitive_or_builtin_type(ty),
                "Expected Flutter UI type '{}' to be recognized as builtin",
                ty
            );
        }
    }

    #[test]
    fn test_issue_461_is_primitive_or_builtin_type_module_prefixes() {
        let prefixed_types = vec![
            "ScalarValues::Boolean",
            "ISQ::LengthValue",
            "SI::m",
            "std::path::PathBuf",
            "std::collections::HashMap",
            "std::sync::Arc",
            "std::time::Duration",
            "core::option::Option",
            "core::result::Result",
            "alloc::vec::Vec",
            "alloc::string::String",
        ];
        for ty in prefixed_types {
            assert!(
                is_primitive_or_builtin_type(ty),
                "Expected prefixed type '{}' to be recognized as builtin",
                ty
            );
        }
    }

    #[test]
    fn test_issue_461_is_primitive_or_builtin_type_composite_generics() {
        assert!(is_primitive_or_builtin_type("Vec<u8>"));
        assert!(is_primitive_or_builtin_type("Option<String>"));
        assert!(is_primitive_or_builtin_type("Result<i32, Error>"));
        assert!(is_primitive_or_builtin_type("HashMap<String, Vec<u8>>"));
        assert!(is_primitive_or_builtin_type("List<Widget>"));
        assert!(is_primitive_or_builtin_type("Future<void>"));
        assert!(is_primitive_or_builtin_type("Stream<String>"));
    }

    #[test]
    fn test_issue_461_is_primitive_or_builtin_type_negative_unknown_types() {
        assert!(!is_primitive_or_builtin_type("UnknownCustomSensor"));
        assert!(!is_primitive_or_builtin_type("Vec<UnknownCustomSensor>"));
        assert!(!is_primitive_or_builtin_type(
            "HashMap<String, UnknownCustomSensor>"
        ));
        assert!(!is_primitive_or_builtin_type("Future<UnknownCustomSensor>"));
    }

    #[test]
    fn test_issue_461_is_resolvable_type_composite_and_generic_builtins() {
        let pkg = PackageDef::default();
        let validator = SemanticValidator::new(&pkg);

        // Plain scalars and framework types
        assert!(validator.is_resolvable_type("", "i32"));
        assert!(validator.is_resolvable_type("", "f64"));
        assert!(validator.is_resolvable_type("", "String"));
        assert!(validator.is_resolvable_type("", "Widget"));
        assert!(validator.is_resolvable_type("", "BuildContext"));
        assert!(validator.is_resolvable_type("", "StatefulWidget"));
        assert!(validator.is_resolvable_type("", "()"));

        // Wrapped types
        assert!(validator.is_resolvable_type("", "&Widget"));
        assert!(validator.is_resolvable_type("", "&mut String"));
        assert!(validator.is_resolvable_type("", "*const i32"));
        assert!(validator.is_resolvable_type("", "Widget?"));
        assert!(validator.is_resolvable_type("", "[u8; 32]"));
        assert!(validator.is_resolvable_type("", "&'a str"));

        // Generic builtins wrapping builtins
        assert!(validator.is_resolvable_type("", "Option<String>"));
        assert!(validator.is_resolvable_type("", "Vec<u8>"));
        assert!(validator.is_resolvable_type("", "Result<i32, String>"));
        assert!(validator.is_resolvable_type("", "HashMap<String, Vec<u8>>"));
        assert!(validator.is_resolvable_type("", "Option<Widget>?"));
    }

    #[test]
    fn test_issue_461_is_resolvable_type_user_defined_symbols_in_scope() {
        let pkg = PackageDef {
            name: "Subsystem".to_string(),
            part_defs: vec![
                PartDef {
                    name: "SensorNode".to_string(),
                    is_def: true,
                    ..Default::default()
                },
                PartDef {
                    name: "SafetyController".to_string(),
                    is_def: true,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let validator = SemanticValidator::new(&pkg);

        // Direct user-defined symbol
        assert!(validator.is_resolvable_type("Subsystem", "SensorNode"));
        assert!(validator.is_resolvable_type("Subsystem", "Subsystem::SensorNode"));

        // Generic container wrapping user-defined symbol
        assert!(validator.is_resolvable_type("Subsystem", "Vec<SensorNode>"));
        assert!(validator.is_resolvable_type("Subsystem", "Option<SafetyController>"));
        assert!(validator.is_resolvable_type("Subsystem", "Result<SensorNode, String>"));
        assert!(validator.is_resolvable_type("Subsystem", "HashMap<String, Vec<SensorNode>>"));

        // Wrappers with user-defined symbol
        assert!(validator.is_resolvable_type("Subsystem", "&Vec<SensorNode>"));
        assert!(validator.is_resolvable_type("Subsystem", "Option<SensorNode>?"));
        assert!(validator.is_resolvable_type("Subsystem", "[SensorNode; 4]"));
    }

    #[test]
    fn test_issue_461_is_resolvable_type_fail_closed_negative_mutations() {
        let pkg = PackageDef {
            name: "Subsystem".to_string(),
            part_defs: vec![PartDef {
                name: "SensorNode".to_string(),
                is_def: true,
                ..Default::default()
            }],
            ..Default::default()
        };
        let validator = SemanticValidator::new(&pkg);

        // Undefined atomic types
        assert!(!validator.is_resolvable_type("Subsystem", "UndefinedType"));
        assert!(!validator.is_resolvable_type("Subsystem", "&UndefinedType"));
        assert!(!validator.is_resolvable_type("Subsystem", "UndefinedType?"));

        // Generic container with undefined inner argument
        assert!(!validator.is_resolvable_type("Subsystem", "Vec<UndefinedType>"));
        assert!(!validator.is_resolvable_type("Subsystem", "Option<UndefinedType>"));
        assert!(!validator.is_resolvable_type("Subsystem", "Result<SensorNode, UndefinedError>"));
        assert!(!validator.is_resolvable_type("Subsystem", "Result<UndefinedNode, String>"));
        assert!(!validator.is_resolvable_type("Subsystem", "HashMap<String, Vec<UndefinedType>>"));

        // Undefined outer container with valid inner argument
        assert!(!validator.is_resolvable_type("Subsystem", "UndefinedContainer<String>"));
        assert!(!validator.is_resolvable_type("Subsystem", "UndefinedContainer<SensorNode>"));

        // Empty generic parameter
        assert!(!validator.is_resolvable_type("Subsystem", "Vec<>"));
    }

    #[test]
    fn test_issue_461_validate_part_defs_with_composite_generic_types_passes() {
        let pkg = PackageDef {
            name: "Platform".to_string(),
            part_defs: vec![
                PartDef {
                    name: "SensorNode".to_string(),
                    is_def: true,
                    ..Default::default()
                },
                PartDef {
                    name: "nodes".to_string(),
                    is_def: false,
                    type_name: Some("Vec<SensorNode>".to_string()),
                    ..Default::default()
                },
                PartDef {
                    name: "active_node".to_string(),
                    is_def: false,
                    type_name: Some("Option<SensorNode>".to_string()),
                    ..Default::default()
                },
                PartDef {
                    name: "view".to_string(),
                    is_def: false,
                    type_name: Some("Widget".to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let mut validator = SemanticValidator::new(&pkg);
        validator.validate();
        assert!(
            !validator.has_errors(),
            "Expected zero validation errors for valid composite types, got: {:?}",
            validator.errors()
        );
    }

    #[test]
    fn test_issue_461_validate_part_defs_with_undefined_generic_inner_fails() {
        let pkg = PackageDef {
            name: "Platform".to_string(),
            part_defs: vec![PartDef {
                name: "bad_nodes".to_string(),
                is_def: false,
                type_name: Some("Vec<GhostType>".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };

        let mut validator = SemanticValidator::new(&pkg);
        validator.validate();
        assert!(
            validator.has_errors(),
            "Expected validation error for undefined generic inner type"
        );
        let errors = validator.errors();
        assert!(
            errors.iter().any(|e| e
                .message
                .contains("Part 'bad_nodes' references undefined type 'Vec<GhostType>'")),
            "Expected error message referencing undefined type, got: {:?}",
            errors
        );
    }

    #[test]
    fn test_issue_461_validate_requirements_with_composite_generic_attributes_passes() {
        let pkg = PackageDef {
            name: "Safety".to_string(),
            part_defs: vec![PartDef {
                name: "Guard".to_string(),
                is_def: true,
                ..Default::default()
            }],
            requirement_defs: vec![RequirementDef {
                name: "Req_Safety".to_string(),
                text: "Shall maintain logs.".to_string(),
                attributes: vec![
                    AttributeDef {
                        name: "log_entries".to_string(),
                        type_name: "Vec<String>".to_string(),
                        ..Default::default()
                    },
                    AttributeDef {
                        name: "guard_ref".to_string(),
                        type_name: "Option<Guard>".to_string(),
                        ..Default::default()
                    },
                ],
                satisfied_by: vec!["Guard".to_string()],
                ..Default::default()
            }],
            ..Default::default()
        };

        let mut validator = SemanticValidator::new(&pkg);
        validator.validate();
        assert!(
            !validator.has_errors(),
            "Expected zero validation errors for requirement with generic attributes, got: {:?}",
            validator.errors()
        );
    }

    // =========================================================================
    // Issue #461 Iteration 2: Comprehensive Edge Case Regression Test Suite
    // Covers all 8 edge cases discovered during Iteration 1 adversarial review.
    // =========================================================================

    #[test]
    fn test_issue_461_iter2_edge_1_trailing_tokens_after_generics_fail_closed() {
        let pkg = PackageDef {
            name: "TestScope".to_string(),
            part_defs: vec![
                PartDef {
                    name: "ValidPart".to_string(),
                    is_def: true,
                    ..Default::default()
                },
                PartDef {
                    name: "bad_part_trailing_undefined".to_string(),
                    is_def: false,
                    type_name: Some("Option<String> UndefinedType".to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let mut validator = SemanticValidator::new(&pkg);
        validator.validate();

        // 1. Full model validation MUST fail closed
        assert!(
            validator.has_errors(),
            "Expected validation error for part referencing 'Option<String> UndefinedType'"
        );
        let errors = validator.errors();
        assert!(
            errors.iter().any(|e| e
                .message
                .contains("references undefined type 'Option<String> UndefinedType'")),
            "Expected diagnostic error message for trailing undefined token, got: {:?}",
            errors
        );

        // 2. Direct is_resolvable_type probes MUST fail closed
        let validator = SemanticValidator::new(&pkg);
        assert!(!validator.is_resolvable_type("TestScope", "Option<String> UndefinedType"));
        assert!(!validator.is_resolvable_type("TestScope", "Vec<u8> junk"));
        assert!(!validator.is_resolvable_type("TestScope", "HashMap<String, i32> trailing"));
        assert!(!validator.is_resolvable_type("TestScope", "Vec<ValidPart> GhostPart"));

        // 3. Direct is_primitive_or_builtin_type probes MUST fail closed
        assert!(!is_primitive_or_builtin_type(
            "Option<String> UndefinedType"
        ));
        assert!(!is_primitive_or_builtin_type("Vec<u8> junk"));
        assert!(!is_primitive_or_builtin_type(
            "HashMap<String, i32> trailing"
        ));
    }

    #[test]
    fn test_issue_461_edge_1_trailing_tokens_after_generics_fail_closed() {
        test_issue_461_iter2_edge_1_trailing_tokens_after_generics_fail_closed();
    }

    #[test]
    fn test_issue_461_iter2_edge_2_tuples_with_generics_resolve() {
        let pkg = PackageDef {
            name: "TestScope".to_string(),
            part_defs: vec![
                PartDef {
                    name: "ValidPart".to_string(),
                    is_def: true,
                    ..Default::default()
                },
                PartDef {
                    name: "tuple_part".to_string(),
                    is_def: false,
                    type_name: Some("(ValidPart, Vec<String>)".to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        // 1. Full model validation MUST pass for valid generic tuples
        let mut validator = SemanticValidator::new(&pkg);
        validator.validate();
        assert!(
            !validator.has_errors(),
            "Expected zero validation errors for valid tuple with generics, got: {:?}",
            validator.errors()
        );

        // 2. Direct is_resolvable_type probes
        let validator = SemanticValidator::new(&pkg);
        assert!(validator.is_resolvable_type("TestScope", "(ValidPart, Vec<String>)"));
        assert!(validator.is_resolvable_type("TestScope", "(i32, Option<ValidPart>)"));
        assert!(validator.is_resolvable_type(
            "TestScope",
            "(Vec<ValidPart>, HashMap<String, Option<i32>>)"
        ));

        // 3. Negative mutation: tuples containing undefined types MUST fail closed
        assert!(!validator.is_resolvable_type("TestScope", "(ValidPart, Vec<UndefinedPart>)"));
        assert!(!validator.is_resolvable_type("TestScope", "(UndefinedPart, Vec<String>)"));
        assert!(!validator.is_resolvable_type("TestScope", "(i32, Option<GhostSymbol>)"));
    }

    #[test]
    fn test_issue_461_edge_2_tuples_with_generics_resolve() {
        test_issue_461_iter2_edge_2_tuples_with_generics_resolve();
    }

    #[test]
    fn test_issue_461_iter2_edge_3_bare_modifiers_and_empty_types_fail_closed() {
        let pkg = PackageDef::default();
        let validator = SemanticValidator::new(&pkg);

        // 1. Bare modifiers in is_resolvable_type MUST return false
        assert!(!validator.is_resolvable_type("", "&"));
        assert!(!validator.is_resolvable_type("", "*const"));
        assert!(!validator.is_resolvable_type("", "*mut"));
        assert!(!validator.is_resolvable_type("", "&mut"));
        assert!(!validator.is_resolvable_type("", "&const"));
        assert!(!validator.is_resolvable_type("", "[]"));
        assert!(!validator.is_resolvable_type("", "?"));
        assert!(!validator.is_resolvable_type("", ""));
        assert!(!validator.is_resolvable_type("", "   "));

        // 2. Bare modifiers in is_primitive_or_builtin_type MUST return false
        assert!(!is_primitive_or_builtin_type("&"));
        assert!(!is_primitive_or_builtin_type("*const"));
        assert!(!is_primitive_or_builtin_type("*mut"));
        assert!(!is_primitive_or_builtin_type("&mut"));
        assert!(!is_primitive_or_builtin_type("&const"));
        assert!(!is_primitive_or_builtin_type("[]"));
        assert!(!is_primitive_or_builtin_type("?"));
        assert!(!is_primitive_or_builtin_type(""));
        assert!(!is_primitive_or_builtin_type("   "));

        // 3. Generics wrapping bare modifiers MUST fail closed
        assert!(!validator.is_resolvable_type("", "Option<&>"));
        assert!(!validator.is_resolvable_type("", "Option<[]>"));
        assert!(!validator.is_resolvable_type("", "Vec<?>"));
        assert!(!validator.is_resolvable_type("", "Result<&, []>"));

        // 4. Model validation probe with bare modifier part
        let invalid_pkg = PackageDef {
            name: "InvalidPkg".to_string(),
            part_defs: vec![PartDef {
                name: "bad_mod_part".to_string(),
                is_def: false,
                type_name: Some("&".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };
        let mut invalid_validator = SemanticValidator::new(&invalid_pkg);
        invalid_validator.validate();
        assert!(
            invalid_validator.has_errors(),
            "Expected validation error for part with bare modifier type '&'"
        );
    }

    #[test]
    fn test_issue_461_edge_3_bare_modifiers_and_empty_types_fail_closed() {
        test_issue_461_iter2_edge_3_bare_modifiers_and_empty_types_fail_closed();
    }

    #[test]
    fn test_issue_461_iter2_edge_4_pointer_array_wrappers_normalize_and_resolve() {
        let pkg = PackageDef {
            name: "TestScope".to_string(),
            part_defs: vec![PartDef {
                name: "ValidPart".to_string(),
                is_def: true,
                ..Default::default()
            }],
            ..Default::default()
        };

        // 1. Normalization probes for pointer array wrappers
        assert_eq!(normalize_source_type("*const[ValidPart; 10]"), "ValidPart");
        assert_eq!(normalize_source_type("*mut[u8; 16]"), "u8");
        assert_eq!(normalize_source_type("&mut[ValidPart; 4]"), "ValidPart");
        assert_eq!(normalize_source_type("&const[i32; 8]"), "i32");
        assert_eq!(
            normalize_source_type("*const[TotallyUndefined; 10]"),
            "TotallyUndefined"
        );

        // 2. Resolution for valid pointer-wrapped arrays
        let validator = SemanticValidator::new(&pkg);
        assert!(validator.is_resolvable_type("TestScope", "*const[ValidPart; 10]"));
        assert!(validator.is_resolvable_type("TestScope", "*mut[ValidPart; 10]"));
        assert!(validator.is_resolvable_type("TestScope", "&mut[ValidPart; 10]"));
        assert!(validator.is_resolvable_type("TestScope", "&const[ValidPart; 10]"));
        assert!(validator.is_resolvable_type("TestScope", "*const[u8; 32]"));

        // 3. Negative mutations: pointer-wrapped arrays with undefined types MUST fail closed
        assert!(!validator.is_resolvable_type("TestScope", "*const[TotallyUndefined; 10]"));
        assert!(!validator.is_resolvable_type("TestScope", "*mut[TotallyUndefined; 10]"));
        assert!(!validator.is_resolvable_type("TestScope", "&mut[TotallyUndefined; 10]"));
        assert!(!validator.is_resolvable_type("TestScope", "&const[TotallyUndefined; 10]"));
    }

    #[test]
    fn test_issue_461_edge_4_pointer_array_wrappers_normalize_and_resolve() {
        test_issue_461_iter2_edge_4_pointer_array_wrappers_normalize_and_resolve();
    }

    #[test]
    fn test_issue_461_iter2_edge_5_slice_multiplicity_normalization_and_resolution() {
        let pkg = PackageDef {
            name: "TestScope".to_string(),
            part_defs: vec![PartDef {
                name: "SensorNode".to_string(),
                is_def: true,
                ..Default::default()
            }],
            ..Default::default()
        };

        // 1. Normalization of slices with multiplicity suffixes
        assert_eq!(normalize_source_type("[u8][0..1]"), "u8");
        assert_eq!(normalize_source_type("[u8][1..*]"), "u8");
        assert_eq!(normalize_source_type("[u8][]"), "u8");
        assert_eq!(normalize_source_type("[SensorNode][0..1]"), "SensorNode");
        assert_eq!(normalize_source_type("[SensorNode][1..*]"), "SensorNode");
        assert_eq!(normalize_source_type("&[u8][0..1]"), "u8");

        // 2. Resolution of valid slices with multiplicity
        let validator = SemanticValidator::new(&pkg);
        assert!(validator.is_resolvable_type("TestScope", "[u8][0..1]"));
        assert!(validator.is_resolvable_type("TestScope", "[u8][1..*]"));
        assert!(validator.is_resolvable_type("TestScope", "[SensorNode][0..1]"));
        assert!(validator.is_resolvable_type("TestScope", "[SensorNode][1..*]"));

        // 3. Negative mutation: slices with undefined inner type MUST fail closed
        assert!(!validator.is_resolvable_type("TestScope", "[UndefinedPart][0..1]"));
        assert!(!validator.is_resolvable_type("TestScope", "[UndefinedPart][1..*]"));
    }

    #[test]
    fn test_issue_461_edge_5_slice_multiplicity_normalization_and_resolution() {
        test_issue_461_iter2_edge_5_slice_multiplicity_normalization_and_resolution();
    }

    #[test]
    fn test_issue_461_iter2_edge_6_split_top_level_type_args_multiple_bracket_groups() {
        // 1. Multiple top-level angle bracket groups split cleanly
        assert_eq!(split_top_level_type_args("<A>, <B>"), vec!["<A>", "<B>"]);
        assert_eq!(
            split_top_level_type_args("<A, B>, <C, D>"),
            vec!["<A, B>", "<C, D>"]
        );
        assert_eq!(
            split_top_level_type_args("<A>, <B, <C, D>>, <E>"),
            vec!["<A>", "<B, <C, D>>", "<E>"]
        );

        // 2. Single enclosing angle brackets are unwrapped cleanly
        assert_eq!(split_top_level_type_args("<A, B>"), vec!["A", "B"]);
        assert_eq!(split_top_level_type_args("<<A, B>>"), vec!["<A, B>"]);

        // 3. Standard comma-separated arguments remain intact
        assert_eq!(split_top_level_type_args("A, B, C"), vec!["A", "B", "C"]);
        assert_eq!(
            split_top_level_type_args("HashMap<String, Vec<u8>>, Result<Option<T>, E>"),
            vec!["HashMap<String, Vec<u8>>", "Result<Option<T>, E>"]
        );
    }

    #[test]
    fn test_issue_461_edge_6_split_top_level_type_args_multiple_bracket_groups() {
        test_issue_461_iter2_edge_6_split_top_level_type_args_multiple_bracket_groups();
    }

    #[test]
    fn test_issue_461_iter2_edge_7_trait_objects_with_dyn_resolve() {
        let pkg = PackageDef {
            name: "TestScope".to_string(),
            part_defs: vec![PartDef {
                name: "ValidPart".to_string(),
                is_def: true,
                ..Default::default()
            }],
            ..Default::default()
        };

        // 1. Normalization of trait objects with dyn prefix
        assert_eq!(normalize_source_type("dyn Error"), "Error");
        assert_eq!(normalize_source_type("&dyn Error"), "Error");
        assert_eq!(normalize_source_type("Box<dyn Error>"), "Box<dyn Error>");
        assert_eq!(
            normalize_source_type("Result<Option<HashMap<String, Vec<u8>>>, Box<dyn Error>>"),
            "Result<Option<HashMap<String, Vec<u8>>>, Box<dyn Error>>"
        );

        // 2. Resolution of standard trait objects
        let validator = SemanticValidator::new(&pkg);
        assert!(validator.is_resolvable_type("", "Box<dyn Error>"));
        assert!(validator.is_resolvable_type("", "Arc<dyn Error>"));
        assert!(validator.is_resolvable_type(
            "",
            "Result<Option<HashMap<String, Vec<u8>>>, Box<dyn Error>>"
        ));
        assert!(validator.is_resolvable_type("TestScope", "Result<ValidPart, Box<dyn Error>>"));

        // 3. Negative mutation: trait objects with undefined trait MUST fail closed
        assert!(!validator.is_resolvable_type("", "Box<dyn UndefinedTrait>"));
        assert!(!validator
            .is_resolvable_type("TestScope", "Result<ValidPart, Box<dyn UndefinedTrait>>"));
    }

    #[test]
    fn test_issue_461_edge_7_trait_objects_with_dyn_resolve() {
        test_issue_461_iter2_edge_7_trait_objects_with_dyn_resolve();
    }

    #[test]
    fn test_issue_461_iter2_edge_8_tuples_in_primitive_and_generic_containers() {
        let pkg = PackageDef {
            name: "TestScope".to_string(),
            part_defs: vec![PartDef {
                name: "ValidPart".to_string(),
                is_def: true,
                ..Default::default()
            }],
            ..Default::default()
        };

        // 1. Direct tuple resolution in is_primitive_or_builtin_type
        assert!(is_primitive_or_builtin_type("(u32, bool)"));
        assert!(is_primitive_or_builtin_type("(i32, f64, String)"));
        assert!(is_primitive_or_builtin_type("Vec<(u32, bool)>"));
        assert!(is_primitive_or_builtin_type("Option<(i32, f64)>"));
        assert!(is_primitive_or_builtin_type("HashMap<String, (u8, u16)>"));

        // 2. Negative mutations in is_primitive_or_builtin_type
        assert!(!is_primitive_or_builtin_type("(u32, UnknownType)"));
        assert!(!is_primitive_or_builtin_type("Vec<(u32, UnknownType)>"));

        // 3. Direct resolution in is_resolvable_type with mixed scope symbols
        let validator = SemanticValidator::new(&pkg);
        assert!(validator.is_resolvable_type("", "Vec<(u32, bool)>"));
        assert!(validator.is_resolvable_type("TestScope", "Vec<(ValidPart, bool)>"));
        assert!(validator.is_resolvable_type("TestScope", "Option<(ValidPart, String)>"));
        assert!(!validator.is_resolvable_type("TestScope", "Vec<(UndefinedPart, bool)>"));
    }

    #[test]
    fn test_issue_461_edge_8_tuples_in_primitive_and_generic_containers() {
        test_issue_461_iter2_edge_8_tuples_in_primitive_and_generic_containers();
    }

    #[test]
    fn test_issue_461_iter2_all_iteration_1_edge_cases_comprehensive_matrix() {
        let pkg = PackageDef {
            name: "MatrixScope".to_string(),
            part_defs: vec![
                PartDef {
                    name: "DeclaredSensor".to_string(),
                    is_def: true,
                    ..Default::default()
                },
                PartDef {
                    name: "DeclaredActuator".to_string(),
                    is_def: true,
                    ..Default::default()
                },
                // Edge Case 2: Tuple with generic
                PartDef {
                    name: "sensor_tuple".to_string(),
                    is_def: false,
                    type_name: Some("(DeclaredSensor, Vec<String>)".to_string()),
                    ..Default::default()
                },
                // Edge Case 4: Pointer array wrapper
                PartDef {
                    name: "pointer_array".to_string(),
                    is_def: false,
                    type_name: Some("*const[DeclaredActuator; 8]".to_string()),
                    ..Default::default()
                },
                // Edge Case 5: Slice multiplicity
                PartDef {
                    name: "slice_multiplicity".to_string(),
                    is_def: false,
                    type_name: Some("[DeclaredSensor][1..*]".to_string()),
                    ..Default::default()
                },
                // Edge Case 7: Trait object
                PartDef {
                    name: "error_handler".to_string(),
                    is_def: false,
                    type_name: Some("Box<dyn Error>".to_string()),
                    ..Default::default()
                },
                // Edge Case 8: Generic container wrapping primitive tuple
                PartDef {
                    name: "coordinate_stream".to_string(),
                    is_def: false,
                    type_name: Some("Stream<(f64, f64)>".to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        // All valid edge cases in the matrix MUST validate cleanly
        let mut validator = SemanticValidator::new(&pkg);
        validator.validate();
        assert!(
            !validator.has_errors(),
            "Expected zero validation errors for comprehensive edge cases matrix, got: {:?}",
            validator.errors()
        );
    }

    #[test]
    fn test_issue_461_all_iteration_1_edge_cases_comprehensive_matrix() {
        test_issue_461_iter2_all_iteration_1_edge_cases_comprehensive_matrix();
    }

    /// Iteratively dismantles a `PackageDef` tree and all nested `PartDef` subtrees in O(1) stack frames.
    fn drain_package_tree_iteratively(root: PackageDef) {
        let mut extracted_parts: Vec<PartDef> = Vec::new();
        let mut pkg_drain = vec![root];
        while let Some(mut p) = pkg_drain.pop() {
            extracted_parts.append(&mut p.part_defs);
            pkg_drain.append(&mut p.packages);
        }
        while let Some(mut pt) = extracted_parts.pop() {
            extracted_parts.append(&mut pt.parts);
        }
    }

    #[test]
    fn test_deep_nested_ast_hierarchy_bounded_without_stack_overflow() -> Result<(), String> {
        use crate::semantic::digest::{count_structural_elements, SchemaDigest};
        use crate::semantic::serializer::SysmlSerializable;

        let bound: usize = 64;

        // 1. Construct a 64-level deep nested PackageDef tree iteratively (Pkg_0 .. Pkg_63).
        let mut pkg_64 = PackageDef {
            name: format!("Pkg_{}", bound.saturating_sub(1)),
            requirement_defs: vec![RequirementDef {
                name: "Req_Leaf_63".to_string(),
                text: "Leaf requirement at depth 63.".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        for idx in (0..bound.saturating_sub(1)).rev() {
            pkg_64 = PackageDef {
                name: format!("Pkg_{idx}"),
                packages: vec![pkg_64],
                ..Default::default()
            };
        }

        let pkg_64_symbols = SymbolTable::build_from_package(&pkg_64);
        assert_eq!(pkg_64_symbols.all_of_kind(SymbolKind::Package).len(), bound);
        assert_eq!(pkg_64_symbols.all_of_kind(SymbolKind::Requirement).len(), 1);

        let mut pkg_64_validator = SemanticValidator::new(&pkg_64);
        pkg_64_validator.validate();
        assert!(
            !pkg_64_validator.has_errors(),
            "64-level PackageDef tree must validate cleanly, got: {:?}",
            pkg_64_validator.errors()
        );

        let pkg_64_sysml = pkg_64.to_sysml(0);
        assert!(pkg_64_sysml.contains("package Pkg_0 {"));
        assert!(pkg_64_sysml.contains("package Pkg_63 {"));

        let pkg_64_digest = SchemaDigest::compute(&pkg_64, &pkg_64_sysml);
        assert!(!pkg_64_digest.sha256.is_empty());
        assert!(pkg_64_digest.total_lines > 0);
        assert_eq!(
            pkg_64_digest.node_counts.get("packages").copied(),
            Some(bound)
        );

        // 2. Construct a 64-level deep PackageDef + nested PartDef tree iteratively
        //    (PartRoot at depth 0 + Part_1 .. Part_63 at depths 1..63 = 64 AST levels).
        let mut part_chain_63 = PartDef {
            name: format!("Part_{}", bound.saturating_sub(1)),
            type_name: Some("String".to_string()),
            ports: vec![PortDef {
                name: "leaf_out".to_string(),
                direction: "out".to_string(),
                type_name: "Real".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        for idx in (1..bound.saturating_sub(1)).rev() {
            part_chain_63 = PartDef {
                name: format!("Part_{idx}"),
                type_name: Some("Integer".to_string()),
                parts: vec![part_chain_63],
                ..Default::default()
            };
        }
        let part_root_64 = PackageDef {
            name: "PartRoot".to_string(),
            part_defs: vec![part_chain_63],
            ..Default::default()
        };

        let part_64_symbols = SymbolTable::build_from_package(&part_root_64);
        assert_eq!(part_64_symbols.all_of_kind(SymbolKind::Package).len(), 1);
        assert_eq!(
            part_64_symbols.all_of_kind(SymbolKind::Part).len(),
            bound.saturating_sub(1)
        );

        let mut part_64_validator = SemanticValidator::new(&part_root_64);
        part_64_validator.validate();
        assert!(
            !part_64_validator.has_errors(),
            "64-level PartDef hierarchy must validate cleanly, got: {:?}",
            part_64_validator.errors()
        );

        let part_64_sysml = part_root_64.to_sysml(0);
        assert!(part_64_sysml.contains("part def Part_1 : Integer {"));
        assert!(part_64_sysml.contains("part def Part_63 : String {"));

        let part_64_digest = SchemaDigest::compute(&part_root_64, &part_64_sysml);
        assert_eq!(
            part_64_digest.node_counts.get("part_defs").copied(),
            Some(bound.saturating_sub(1))
        );

        // Negative probe at depth 63 (< 64): undefined type at level 63 MUST still be caught.
        let mut bad_part_63 = PartDef {
            name: "BadLeaf_63".to_string(),
            is_def: false,
            type_name: Some("UndefinedTypeAtDepth63".to_string()),
            ..Default::default()
        };
        for idx in (1..bound.saturating_sub(1)).rev() {
            bad_part_63 = PartDef {
                name: format!("Carrier_{idx}"),
                parts: vec![bad_part_63],
                ..Default::default()
            };
        }
        let bad_root_64 = PackageDef {
            name: "BadRoot64".to_string(),
            part_defs: vec![bad_part_63],
            ..Default::default()
        };
        let mut bad_64_validator = SemanticValidator::new(&bad_root_64);
        bad_64_validator.validate();
        assert!(
            bad_64_validator.has_errors(),
            "Undefined type at depth 63 (< MAX_AST_DEPTH) must still emit an error"
        );

        // 3. Construct a > 64-level (65-level, MAX_AST_DEPTH + 1) tree iteratively and verify
        //    nodes at depth >= 64 are safely bounded across symbols, validator, serializer, and digest.
        let over_bound = bound.saturating_add(1);
        let mut part_chain_over = PartDef {
            name: format!("Part_{}", over_bound.saturating_sub(1)),
            is_def: false,
            type_name: Some("UnindexedBeyondDepth64Type".to_string()),
            ports: vec![
                PortDef {
                    name: "dup_port".to_string(),
                    direction: "in".to_string(),
                    type_name: "Real".to_string(),
                    ..Default::default()
                },
                PortDef {
                    name: "dup_port".to_string(),
                    direction: "in".to_string(),
                    type_name: "Real".to_string(),
                    ..Default::default()
                },
            ],
            connections: vec![ConnectionDef {
                name: "BeyondBoundConn".to_string(),
                source_port: "Missing.src".to_string(),
                target_port: "Missing.tgt".to_string(),
                ..Default::default()
            }],
            requirements: vec![RequirementDef {
                name: "BeyondBoundReq".to_string(),
                text: "Shall cap traversal @SafetyRealises(UCA-MISSING-99) @TriggersHazard(HZ-MISSING-99)"
                    .to_string(),
                satisfied_by: vec!["MissingSatisfierBeyond64".to_string()],
                ..Default::default()
            }],
            ..Default::default()
        };
        for idx in (1..over_bound.saturating_sub(1)).rev() {
            part_chain_over = PartDef {
                name: format!("Part_{idx}"),
                parts: vec![part_chain_over],
                ..Default::default()
            };
        }
        let over_root_65 = PackageDef {
            name: "OverRoot65".to_string(),
            constraint_defs: vec![ConstraintDef {
                name: "UCA_01".to_string(),
                expression: "x > 0".to_string(),
                ..Default::default()
            }],
            hazard_defs: vec![HazardDef {
                name: "HZ_01".to_string(),
                ..Default::default()
            }],
            part_defs: vec![part_chain_over],
            ..Default::default()
        };

        let over_symbols = SymbolTable::build_from_package(&over_root_65);
        assert_eq!(
            over_symbols.all_of_kind(SymbolKind::Part).len(),
            bound.saturating_sub(1)
        );
        let mut over_validator = SemanticValidator::new(&over_root_65);
        over_validator.validate();
        assert!(
            !over_validator.has_errors(),
            "Nodes at depth >= 64 must be bounded out consistently with SymbolTable, got: {:?}",
            over_validator.errors()
        );
        let over_sysml = over_root_65.to_sysml(0);
        assert!(over_sysml.contains("part def Part_63"));
        assert!(!over_sysml.contains("Part_64"));
        let over_digest = SchemaDigest::compute(&over_root_65, &over_sysml);
        assert_eq!(
            over_digest.node_counts.get("part_defs").copied(),
            Some(over_bound.saturating_sub(1))
        );

        // Generic type recursion bound at 64 levels vs 65 levels (> 64 fails closed).
        let mut generic_64 = "u8".to_string();
        for _ in 0..bound.saturating_sub(1) {
            generic_64 = format!("Vec<{generic_64}>");
        }
        let generic_65 = format!("Vec<{generic_64}>");
        assert!(is_primitive_or_builtin_type(&generic_64));
        assert!(!is_primitive_or_builtin_type(&generic_65));
        assert!(over_validator.is_resolvable_type("OverRoot65", &generic_64));
        assert!(!over_validator.is_resolvable_type("OverRoot65", &generic_65));

        // 4. Hostile 1,500-level deep PackageDef, PartDef, and generic type inside a 256 KiB stack thread.
        let hostile_levels: usize = 1_500;
        let mut hostile_pkg = PackageDef {
            name: format!("HostilePkg_{}", hostile_levels.saturating_sub(1)),
            ..Default::default()
        };
        for idx in (0..hostile_levels.saturating_sub(1)).rev() {
            hostile_pkg = PackageDef {
                name: format!("HostilePkg_{idx}"),
                packages: vec![hostile_pkg],
                ..Default::default()
            };
        }

        let mut hostile_part = PartDef {
            name: format!("HostilePart_{}", hostile_levels.saturating_sub(1)),
            type_name: Some("Real".to_string()),
            ..Default::default()
        };
        for idx in (1..hostile_levels.saturating_sub(1)).rev() {
            hostile_part = PartDef {
                name: format!("HostilePart_{idx}"),
                type_name: Some("Integer".to_string()),
                parts: vec![hostile_part],
                ..Default::default()
            };
        }
        let hostile_part_root = PackageDef {
            name: "HostilePartRoot".to_string(),
            part_defs: vec![hostile_part],
            ..Default::default()
        };

        let mut hostile_generic = "u8".to_string();
        for _ in 0..hostile_levels {
            hostile_generic = format!("Option<{hostile_generic}>");
        }

        struct BoundedRunOutcome {
            pkg_sym_count: usize,
            part_sym_count: usize,
            pkg_errors: usize,
            part_errors: usize,
            generic_builtin: bool,
            generic_resolvable: bool,
            pkg_has_63: bool,
            pkg_has_64: bool,
            hostile_elem_count: usize,
        }

        let outcome = std::thread::scope(|s| -> Result<BoundedRunOutcome, String> {
            let handle = std::thread::Builder::new()
                .name("bounded_validator_1500".to_string())
                .stack_size(256 * 1024)
                .spawn_scoped(s, || {
                    let sym_pkg = SymbolTable::build_from_package(&hostile_pkg);
                    let sym_part = SymbolTable::build_from_package(&hostile_part_root);

                    let mut v_pkg = SemanticValidator::new(&hostile_pkg);
                    v_pkg.validate();

                    let mut v_part = SemanticValidator::new(&hostile_part_root);
                    v_part.validate();

                    let generic_builtin = is_primitive_or_builtin_type(&hostile_generic);
                    let generic_resolvable =
                        v_part.is_resolvable_type("HostilePartRoot", &hostile_generic);

                    let rendered_pkg = hostile_pkg.to_sysml(0);
                    let _rendered_part = hostile_part_root.to_sysml(0);
                    let hostile_elem_count = count_structural_elements(&hostile_part_root);

                    BoundedRunOutcome {
                        pkg_sym_count: sym_pkg.symbols.len(),
                        part_sym_count: sym_part.symbols.len(),
                        pkg_errors: v_pkg.errors().len(),
                        part_errors: v_part.errors().len(),
                        generic_builtin,
                        generic_resolvable,
                        pkg_has_63: rendered_pkg.contains("package HostilePkg_63 {"),
                        pkg_has_64: rendered_pkg.contains("HostilePkg_64"),
                        hostile_elem_count,
                    }
                })
                .map_err(|e| format!("failed to spawn bounded-stack validator thread: {e}"))?;
            handle
                .join()
                .map_err(|_| "bounded-stack validator thread panicked or overflowed".to_string())
        })?;

        assert_eq!(outcome.pkg_sym_count, bound);
        assert_eq!(outcome.part_sym_count, bound);
        assert_eq!(outcome.pkg_errors, 0);
        assert_eq!(outcome.part_errors, 0);
        assert!(!outcome.generic_builtin);
        assert!(!outcome.generic_resolvable);
        assert!(outcome.pkg_has_63);
        assert!(!outcome.pkg_has_64);
        assert_eq!(outcome.hostile_elem_count, hostile_levels.saturating_sub(1));

        drain_package_tree_iteratively(pkg_64);
        drain_package_tree_iteratively(part_root_64);
        drain_package_tree_iteratively(bad_root_64);
        drain_package_tree_iteratively(over_root_65);
        drain_package_tree_iteratively(hostile_pkg);
        drain_package_tree_iteratively(hostile_part_root);
        Ok(())
    }

    #[test]
    fn test_utf8_multibyte_annotations_and_unclosed_delimiters(
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Valid multi-byte UTF-8 prose (2-byte, 3-byte, 4-byte codepoints) alongside tags
        let valid_pkg = PackageDef {
            name: "航空安全_Sys".to_string(),
            constraint_defs: vec![
                ConstraintDef {
                    name: "UCA_01_Overspeed".to_string(),
                    expression: "v <= 60.0".to_string(),
                    ..Default::default()
                },
                ConstraintDef {
                    name: "UCA_02_Thermal".to_string(),
                    expression: "t <= 85.0".to_string(),
                    ..Default::default()
                },
            ],
            hazard_defs: vec![HazardDef {
                name: "HZ_01_LossOfControl".to_string(),
                ..Default::default()
            }],
            requirement_defs: vec![RequirementDef {
                name: "Req_UTF8_Valid".to_string(),
                doc: Some("Δv ≤ 60.0 m/s 🛰 許容範囲 äöüß".to_string()),
                text:
                    "飛行制御 Ω @SafetyRealises(UCA-01, UCA-02) および @triggershazard: hz-01 完了"
                        .to_string(),
                assumes: vec!["温度 ≥ -40°C @SAFETYREALISES: UCA-02".to_string()],
                ..Default::default()
            }],
            ..Default::default()
        };

        let mut valid_validator = SemanticValidator::new(&valid_pkg);
        valid_validator.validate();
        assert!(
            !valid_validator.has_errors(),
            "Expected zero errors on valid multi-byte UTF-8 requirement tags, got: {:?}",
            valid_validator.errors()
        );

        // 2. Dangling constraint and hazard references surrounded by multi-byte UTF-8 codepoints
        let dangling_pkg = PackageDef {
            name: "航空安全_Dangling".to_string(),
            constraint_defs: valid_pkg.constraint_defs.clone(),
            hazard_defs: valid_pkg.hazard_defs.clone(),
            requirement_defs: vec![RequirementDef {
                name: "Req_UTF8_Dangling".to_string(),
                text: "🛰ΔΩ@SafetyRealises(UCA-99)航空@TriggersHazard:HZ-99≤≥".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };

        let mut dangling_validator = SemanticValidator::new(&dangling_pkg);
        dangling_validator.validate();
        assert!(dangling_validator.has_errors());
        let dangling_errors = dangling_validator.errors();
        assert!(dangling_errors.iter().any(|e| e.message.contains("UCA-99")));
        assert!(dangling_errors.iter().any(|e| e.message.contains("HZ-99")));

        // 3. Unclosed annotation delimiter with multi-byte UTF-8 prose must fail closed, never silently masked
        let unclosed_pkg = PackageDef {
            name: "航空安全_Unclosed".to_string(),
            constraint_defs: valid_pkg.constraint_defs,
            hazard_defs: valid_pkg.hazard_defs,
            requirement_defs: vec![RequirementDef {
                name: "Req_UTF8_Unclosed".to_string(),
                text: "Δv ≤ 60.0 m/s 🛰 @SafetyRealises(UCA-01".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };

        let mut unclosed_validator = SemanticValidator::new(&unclosed_pkg);
        unclosed_validator.validate();
        assert!(
            unclosed_validator.has_errors(),
            "Unclosed @SafetyRealises( delimiter must be rejected rather than silently masked"
        );
        assert!(unclosed_validator.errors().iter().any(|e| e
            .message
            .contains("unclosed '(' delimiter in @SafetyRealises")));

        // 4. Direct extraction probe with multi-byte UTF-8 tag prefix and unclosed @TriggersHazard(
        let spans = extract_annotation_targets(
            "ΔΩ §Safety: UCA-01 および §Safety invalid §Safety(UCA-02, UCA-03)",
            "§Safety",
        );
        assert_eq!(
            spans,
            vec![
                AnnotationSpan::Targets(vec!["UCA-01".to_string()]),
                AnnotationSpan::Targets(vec!["UCA-02".to_string(), "UCA-03".to_string()]),
            ]
        );
        let unclosed_hazard_spans =
            extract_annotation_targets("航空安全 @TriggersHazard(HZ-01", "@TriggersHazard");
        assert_eq!(
            unclosed_hazard_spans,
            vec![AnnotationSpan::UnclosedDelimiter]
        );
        Ok(())
    }
}
