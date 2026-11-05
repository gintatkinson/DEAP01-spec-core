#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Bounded iterative AST indexing for SysML v2 packages and parts into `SymbolTable`.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for symbol table indexing.
//! ## 2. Invariants & Mathematical Properties
//! Indexes package and part hierarchies deterministically in pre-order using explicit work-stacks capped at `MAX_AST_DEPTH` (64).
//! ## 3. Preconditions
//! Caller provides a borrowed `PackageDef` or `PartDef` AST node and a parent scope string slice.
//! ## 4. Postconditions
//! Populates `SymbolTable::symbols`, `SymbolTable::by_name`, and `SymbolTable::package_imports` without stack overflow.
//! ## 5. Algorithmic Complexity Bounds
//! Time Complexity: O(V) over indexed AST nodes within depth 64. Space Complexity: O(V) auxiliary heap storage.
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface / State / Action / Resource: Truncates traversal safely at `MAX_AST_DEPTH` using saturating arithmetic and zero panics.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603, REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-INDEX-PACKAGE, REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-INDEX-PART]

use super::symbols::{Symbol, SymbolKind, SymbolTable, MAX_AST_DEPTH};
use deap_core::sysml_ast::{PackageDef, PartDef};

/// Work-stack frame for bounded iterative `PackageDef` symbol indexing.
/// Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-INDEX-PACKAGE]
struct PackageIndexFrame<'a> {
    pkg: &'a PackageDef,
    parent_scope: String,
    depth: usize,
}

/// Work-stack frame for bounded iterative `PartDef` symbol indexing.
/// Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-INDEX-PART]
struct PartIndexFrame<'a> {
    part: &'a PartDef,
    parent_scope: String,
    depth: usize,
}

impl SymbolTable {
    /// Inserts a standard non-port leaf symbol under `parent_scope`.
    /// Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-INDEX-PACKAGE]
    fn insert_scoped_symbol(
        &mut self,
        parent_scope: &str,
        name: &str,
        kind: SymbolKind,
        doc: &Option<String>,
    ) {
        self.insert(Symbol {
            name: name.to_string(),
            qualified_name: format!("{parent_scope}::{name}"),
            kind,
            parent: Some(parent_scope.to_string()),
            type_name: None,
            direction: None,
            is_conjugated: false,
            doc: doc.clone(),
        });
    }

    /// Iteratively indexes `root_pkg` and nested packages/parts in pre-order up to `MAX_AST_DEPTH`.
    /// Uses an explicit `Vec` work-stack with saturating depth increments so deep trees never overflow the call stack.
    /// Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-INDEX-PACKAGE]
    pub(crate) fn index_package(&mut self, root_pkg: &PackageDef, root_parent_scope: &str) {
        let mut stack = vec![PackageIndexFrame {
            pkg: root_pkg,
            parent_scope: root_parent_scope.to_string(),
            depth: 0,
        }];

        while let Some(frame) = stack.pop() {
            if frame.depth >= MAX_AST_DEPTH {
                continue;
            }
            let pkg_qname = self.register_package_scope(frame.pkg, &frame.parent_scope);
            self.register_package_leaf_members(frame.pkg, &pkg_qname);

            let next_depth = frame.depth.saturating_add(1);
            if next_depth < MAX_AST_DEPTH {
                for part in &frame.pkg.part_defs {
                    self.index_part(part, &pkg_qname, next_depth);
                }
                for subpkg in frame.pkg.packages.iter().rev() {
                    stack.push(PackageIndexFrame {
                        pkg: subpkg,
                        parent_scope: pkg_qname.clone(),
                        depth: next_depth,
                    });
                }
            }
        }
    }

    /// Computes the qualified package scope name and registers package-level symbol and import bindings.
    /// Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-INDEX-PACKAGE]
    fn register_package_scope(&mut self, pkg: &PackageDef, parent_scope: &str) -> String {
        let is_synthetic_multi_root =
            parent_scope.is_empty() && crate::parser::grammar::is_synthetic_forest_root(pkg);

        let pkg_qname = if is_synthetic_multi_root {
            String::new()
        } else if parent_scope.is_empty() {
            pkg.name.clone()
        } else if pkg.name.is_empty() {
            parent_scope.to_string()
        } else {
            format!("{}::{}", parent_scope, pkg.name)
        };

        if !pkg_qname.is_empty() {
            self.package_imports
                .insert(pkg_qname.clone(), pkg.imports.clone());
            if !pkg.name.is_empty() && pkg.name != pkg_qname {
                self.package_imports
                    .entry(pkg.name.clone())
                    .or_default()
                    .extend(pkg.imports.clone());
            }

            self.insert(Symbol {
                name: pkg.name.clone(),
                qualified_name: pkg_qname.clone(),
                kind: SymbolKind::Package,
                parent: (!parent_scope.is_empty()).then(|| parent_scope.to_string()),
                type_name: None,
                direction: None,
                is_conjugated: false,
                doc: pkg.doc.clone(),
            });
        } else if !pkg.imports.is_empty() {
            self.package_imports
                .entry(String::new())
                .or_default()
                .extend(pkg.imports.clone());
        }

        pkg_qname
    }

    /// Registers non-recursive leaf member definitions declared directly inside `pkg`.
    /// Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-INDEX-PACKAGE]
    fn register_package_leaf_members(&mut self, pkg: &PackageDef, pkg_qname: &str) {
        for attr in &pkg.attribute_defs {
            self.insert(Symbol {
                name: attr.name.clone(),
                qualified_name: format!("{}::{}", pkg_qname, attr.name),
                kind: SymbolKind::Attribute,
                parent: Some(pkg_qname.to_string()),
                type_name: Some(attr.type_name.clone()),
                direction: None,
                is_conjugated: false,
                doc: attr.doc.clone(),
            });
        }
        for port in &pkg.port_defs {
            self.insert(Symbol {
                name: port.name.clone(),
                qualified_name: format!("{}::{}", pkg_qname, port.name),
                kind: SymbolKind::Port,
                parent: Some(pkg_qname.to_string()),
                type_name: Some(port.type_name.clone()),
                direction: Some(port.direction.clone()),
                is_conjugated: port.is_conjugated,
                doc: port.doc.clone(),
            });
        }
        for act in &pkg.action_defs {
            self.insert_scoped_symbol(pkg_qname, &act.name, SymbolKind::Action, &act.doc);
        }
        for con in &pkg.constraint_defs {
            self.insert_scoped_symbol(pkg_qname, &con.name, SymbolKind::Constraint, &con.doc);
        }
        for req in &pkg.requirement_defs {
            self.insert_scoped_symbol(pkg_qname, &req.name, SymbolKind::Requirement, &req.doc);
        }
        for state in &pkg.state_defs {
            self.insert_scoped_symbol(pkg_qname, &state.name, SymbolKind::State, &state.doc);
        }
        for conn in &pkg.connection_defs {
            self.insert_scoped_symbol(pkg_qname, &conn.name, SymbolKind::Connection, &conn.doc);
        }
        for hz in &pkg.hazard_defs {
            self.insert_scoped_symbol(pkg_qname, &hz.name, SymbolKind::Hazard, &hz.doc);
        }
        for rk in &pkg.risk_defs {
            self.insert_scoped_symbol(pkg_qname, &rk.name, SymbolKind::Risk, &rk.doc);
        }
    }

    /// Iteratively indexes `root_part` and nested subparts in pre-order starting at `root_depth` up to `MAX_AST_DEPTH`.
    /// Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-INDEX-PART]
    fn index_part(&mut self, root_part: &PartDef, root_parent_scope: &str, root_depth: usize) {
        let mut stack = vec![PartIndexFrame {
            part: root_part,
            parent_scope: root_parent_scope.to_string(),
            depth: root_depth,
        }];

        while let Some(frame) = stack.pop() {
            if frame.depth >= MAX_AST_DEPTH {
                continue;
            }
            let part_qname = self.register_part_scope_and_members(frame.part, &frame.parent_scope);
            let next_depth = frame.depth.saturating_add(1);
            if next_depth < MAX_AST_DEPTH {
                for subpart in frame.part.parts.iter().rev() {
                    stack.push(PartIndexFrame {
                        part: subpart,
                        parent_scope: part_qname.clone(),
                        depth: next_depth,
                    });
                }
            }
        }
    }

    /// Registers `part` and its direct leaf attributes, ports, actions, constraints, requirements, states, and connections.
    /// Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-INDEX-PART]
    fn register_part_scope_and_members(&mut self, part: &PartDef, parent_scope: &str) -> String {
        let part_qname = format!("{}::{}", parent_scope, part.name);
        self.insert(Symbol {
            name: part.name.clone(),
            qualified_name: part_qname.clone(),
            kind: SymbolKind::Part,
            parent: Some(parent_scope.to_string()),
            type_name: part.type_name.clone(),
            direction: None,
            is_conjugated: false,
            doc: part.doc.clone(),
        });
        for attr in &part.attributes {
            self.insert(Symbol {
                name: attr.name.clone(),
                qualified_name: format!("{}::{}", part_qname, attr.name),
                kind: SymbolKind::Attribute,
                parent: Some(part_qname.clone()),
                type_name: Some(attr.type_name.clone()),
                direction: None,
                is_conjugated: false,
                doc: attr.doc.clone(),
            });
        }
        for port in &part.ports {
            self.insert(Symbol {
                name: port.name.clone(),
                qualified_name: format!("{}::{}", part_qname, port.name),
                kind: SymbolKind::Port,
                parent: Some(part_qname.clone()),
                type_name: Some(port.type_name.clone()),
                direction: Some(port.direction.clone()),
                is_conjugated: port.is_conjugated,
                doc: port.doc.clone(),
            });
        }
        for act in &part.actions {
            self.insert_scoped_symbol(&part_qname, &act.name, SymbolKind::Action, &act.doc);
        }
        for con in &part.constraints {
            self.insert_scoped_symbol(&part_qname, &con.name, SymbolKind::Constraint, &con.doc);
        }
        for req in &part.requirements {
            self.insert_scoped_symbol(&part_qname, &req.name, SymbolKind::Requirement, &req.doc);
        }
        for state in &part.states {
            self.insert_scoped_symbol(&part_qname, &state.name, SymbolKind::State, &state.doc);
        }
        for conn in &part.connections {
            self.insert_scoped_symbol(&part_qname, &conn.name, SymbolKind::Connection, &conn.doc);
        }
        part_qname
    }
}
