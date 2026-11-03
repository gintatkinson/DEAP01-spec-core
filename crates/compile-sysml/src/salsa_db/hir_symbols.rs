#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! # Bounded Iterative Symbol Table & Scope Resolution (`hir_symbols`)
//! [DOC-TIER: A]
//! ## 1. Safety Intent & Regulatory Scope -- Indexes SysML v2 / KerML packages, parts, ports, attributes, requirements, and states into deterministic `BTreeMap` lookup tables (`BP-10`).
//! ## 2. Invariants & Mathematical Properties -- Caps AST traversal depth at `MAX_AST_DEPTH` (`64`) using explicit iterative work-stacks.
//! ## 3. Preconditions -- AST inputs conform to `deap_core::sysml_cst` structural invariants.
//! ## 4. Postconditions -- Provides deterministic qualified, scoped, imported, and port-level symbol resolution.
//! ## 5. Algorithmic Complexity Bounds -- Time: O(log N) qualified lookup; Space: O(N) symbol storage.
//! ## 6. Failure Modes & Fault Tolerance -- Zero recursion and zero panics on deep hierarchies.
//! ## 7. Realization & Traceability -- Realises: [REQ-0603], [REQ-SEMANTIC-SYMBOLS-MODULE] | [FLS: fls_66m4rnbssgig]

use std::collections::BTreeMap;
use deap_core::sysml_cst::{ImportDef, PackageDef, PartDef};

/// Maximum AST nesting depth permitted during SysML v2 symbol indexing and serialization. Realises: [REQ-SEMANTIC-SYMBOLS-MAX-AST-DEPTH] | [FLS: fls_66m4rnbssgig]
pub const MAX_AST_DEPTH: usize = 64;

/// Classification of SysML v2 entities in the symbol table. Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLKIND] | [FLS: fls_8tsynkj2cufj]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Package, Part, Port, Attribute, Action, Operation, Constraint, Requirement,
    State, Transition, Connection, Item, Hazard, Risk, UseCase, TestCase, Capability, Interaction,
}

/// A resolved symbol entry in the symbol table. Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOL] | [FLS: fls_66m4rnbssgig]
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub qualified_name: String,
    pub kind: SymbolKind,
    pub parent: Option<String>,
    pub type_name: Option<String>,
    pub direction: Option<String>,
    pub is_conjugated: bool,
    pub doc: Option<String>,
}

/// Symbol table providing scope resolution and entity lookup. Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE] | [FLS: fls_66m4rnbssgig]
#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    pub symbols: BTreeMap<String, Symbol>,
    pub by_name: BTreeMap<String, Vec<String>>,
    pub package_imports: BTreeMap<String, Vec<ImportDef>>,
}

/// Work-stack frame for bounded iterative `PackageDef` symbol indexing. Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-INDEX-PACKAGE] | [FLS: fls_66m4rnbssgig]
struct PackageIndexFrame<'a> { pkg: &'a PackageDef, parent_scope: String, depth: usize }

/// Work-stack frame for bounded iterative `PartDef` symbol indexing. Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-INDEX-PART] | [FLS: fls_66m4rnbssgig]
struct PartIndexFrame<'a> { part: &'a PartDef, parent_scope: String, depth: usize }

impl SymbolTable {
    /// Constructs an empty `SymbolTable`. Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-NEW] | [FLS: fls_66m4rnbssgig]
    #[must_use]
    pub fn new() -> Self { Self::default() }

    /// Builds a comprehensive symbol table from a `PackageDef` hierarchy. Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-BUILD-FROM-PACKAGE] | [FLS: fls_66m4rnbssgig]
    #[must_use]
    pub fn build_from_package(pkg: &PackageDef) -> Self {
        let mut table = Self::new();
        table.index_package(pkg, "");
        table
    }

    /// Looks up a symbol by exact qualified name. Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-LOOKUP] | [FLS: fls_66m4rnbssgig]
    #[must_use]
    pub fn lookup(&self, qualified_name: &str) -> Option<&Symbol> {
        self.symbols.get(qualified_name)
    }

    /// Looks up a symbol within a given parent scope or falls back to global scope. Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-LOOKUP-IN-SCOPE] | [FLS: fls_66m4rnbssgig]
    #[must_use]
    pub fn lookup_in_scope(&self, scope: &str, name: &str) -> Option<&Symbol> {
        if name.contains("::") {
            if let Some(sym) = self.lookup(name) { return Some(sym); }
            if !scope.is_empty() {
                if let Some(sym) = self.lookup(&format!("{scope}::{name}")) { return Some(sym); }
            }
        }
        if !scope.is_empty() {
            if let Some(sym) = self.symbols.get(&format!("{scope}::{name}")) { return Some(sym); }
        }
        if let Some(sym) = self.lookup_via_imports(scope, name) { return Some(sym); }
        if let Some(sym) = self.symbols.get(name) { return Some(sym); }
        self.by_name.get(name).and_then(|qnames| qnames.first()).and_then(|first| self.symbols.get(first))
    }

    /// Resolves `name` by traversing `package_imports` from `scope` up to enclosing parent scopes. Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-LOOKUP-IN-SCOPE] | [FLS: fls_66m4rnbssgig]
    fn lookup_via_imports(&self, scope: &str, name: &str) -> Option<&Symbol> {
        let mut curr_scope = Some(scope);
        while let Some(s) = curr_scope {
            if let Some(imports) = self.package_imports.get(s) {
                for import in imports {
                    if let Some(sym) = self.resolve_single_import(s, import, name) { return Some(sym); }
                }
            }
            curr_scope = if let Some((parent, _)) = s.rsplit_once("::") {
                Some(parent)
            } else if !s.is_empty() {
                Some("")
            } else {
                None
            };
        }
        None
    }

    /// Evaluates a single wildcard or explicit `ImportDef` within `scope_str` for `name`. Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-LOOKUP-IN-SCOPE] | [FLS: fls_66m4rnbssgig]
    fn resolve_single_import(&self, scope_str: &str, import: &ImportDef, name: &str) -> Option<&Symbol> {
        if import.is_wildcard || import.path.ends_with("::*") || import.path.ends_with("::**") {
            let base = import.path.trim_end_matches("::**").trim_end_matches("::*");
            if let Some(sym) = self.symbols.get(&format!("{base}::{name}")) { return Some(sym); }
            if import.is_recursive || import.path.ends_with("::**") {
                let prefix = format!("{base}::");
                if let Some(qnames) = self.by_name.get(name) {
                    for q in qnames {
                        if q.starts_with(&prefix) {
                            if let Some(sym) = self.symbols.get(q) { return Some(sym); }
                        }
                    }
                }
            }
            if let Some((root_scope, _)) = scope_str.split_once("::") {
                if let Some(sym) = self.symbols.get(&format!("{root_scope}::{base}::{name}")) { return Some(sym); }
            }
        } else {
            let target_elem = import.path.rsplit("::").next().unwrap_or(&import.path);
            if target_elem == name {
                if let Some(sym) = self.symbols.get(&import.path) { return Some(sym); }
                if let Some((root_scope, _)) = scope_str.split_once("::") {
                    if let Some(sym) = self.symbols.get(&format!("{root_scope}::{}", import.path)) { return Some(sym); }
                }
            }
        }
        None
    }

    /// Looks up a port on a part given `"PartName.portName"` or in a specific part scope. Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-LOOKUP-PORT] | [FLS: fls_66m4rnbssgig]
    #[must_use]
    pub fn lookup_port(&self, port_ref: &str, current_part: Option<&str>) -> Option<&Symbol> {
        if let Some((part_name, port_name)) = port_ref.split_once('.') {
            if !port_name.contains('.') {
                if let Some(sym) = self.lookup_dotted_port(part_name, port_name) { return Some(sym); }
            }
        } else if let Some(part) = current_part {
            if let Some(sym) = self.symbols.get(&format!("{part}::{port_ref}")).filter(|s| s.kind == SymbolKind::Port) {
                return Some(sym);
            }
            if let Some(part_qnames) = self.by_name.get(part) {
                for part_q in part_qnames {
                    if let Some(sym) = self.symbols.get(&format!("{part_q}::{port_ref}")).filter(|s| s.kind == SymbolKind::Port) {
                        return Some(sym);
                    }
                }
            }
        }
        if let Some(port_qnames) = self.by_name.get(port_ref) {
            for q in port_qnames {
                if let Some(sym) = self.symbols.get(q).filter(|s| s.kind == SymbolKind::Port) { return Some(sym); }
            }
        }
        None
    }

    /// Resolves `"part_name.port_name"` across direct part symbols and typed part definitions. Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-LOOKUP-PORT] | [FLS: fls_66m4rnbssgig]
    fn lookup_dotted_port(&self, part_name: &str, port_name: &str) -> Option<&Symbol> {
        let part_qnames = self.by_name.get(part_name)?;
        for part_q in part_qnames {
            if let Some(sym) = self.symbols.get(&format!("{part_q}::{port_name}")).filter(|s| s.kind == SymbolKind::Port) {
                return Some(sym);
            }
            if let Some(type_name) = self.symbols.get(part_q).and_then(|part_sym| part_sym.type_name.as_ref()) {
                if let Some(type_qnames) = self.by_name.get(type_name) {
                    for type_q in type_qnames {
                        if let Some(sym) = self.symbols.get(&format!("{type_q}::{port_name}")).filter(|s| s.kind == SymbolKind::Port) {
                            return Some(sym);
                        }
                    }
                }
            }
        }
        None
    }

    /// Gets all symbols of a specific kind. Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-ALL-OF-KIND] | [FLS: fls_66m4rnbssgig]
    #[must_use]
    pub fn all_of_kind(&self, kind: SymbolKind) -> Vec<&Symbol> {
        self.symbols.values().filter(|s| s.kind == kind).collect()
    }

    /// Inserts `symbol` into both `symbols` and `by_name`. Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-INSERT] | [FLS: fls_66m4rnbssgig]
    pub(crate) fn insert(&mut self, symbol: Symbol) {
        self.by_name.entry(symbol.name.clone()).or_default().push(symbol.qualified_name.clone());
        self.symbols.insert(symbol.qualified_name.clone(), symbol);
    }

    /// Inserts a standard non-port leaf symbol under `parent_scope`. Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-INDEX-PACKAGE] | [FLS: fls_66m4rnbssgig]
    fn insert_scoped_symbol(&mut self, parent_scope: &str, name: &str, kind: SymbolKind, doc: &Option<String>) {
        self.insert(Symbol {
            name: name.to_string(), qualified_name: format!("{parent_scope}::{name}"), kind,
            parent: Some(parent_scope.to_string()), type_name: None, direction: None, is_conjugated: false, doc: doc.clone(),
        });
    }

    /// Iteratively indexes `root_pkg` and nested packages/parts up to `MAX_AST_DEPTH`. Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-INDEX-PACKAGE] | [FLS: fls_66m4rnbssgig]
    pub(crate) fn index_package(&mut self, root_pkg: &PackageDef, root_parent_scope: &str) {
        let mut stack = vec![PackageIndexFrame { pkg: root_pkg, parent_scope: root_parent_scope.to_string(), depth: 0 }];
        while let Some(frame) = stack.pop() {
            if frame.depth >= MAX_AST_DEPTH { continue; }
            let pkg_qname = self.register_package_scope(frame.pkg, &frame.parent_scope);
            self.register_package_leaf_members(frame.pkg, &pkg_qname);
            let next_depth = frame.depth.saturating_add(1);
            if next_depth < MAX_AST_DEPTH {
                for part in &frame.pkg.part_defs {
                    self.index_part(part, &pkg_qname, next_depth);
                }
                for subpkg in frame.pkg.packages.iter().rev() {
                    stack.push(PackageIndexFrame { pkg: subpkg, parent_scope: pkg_qname.clone(), depth: next_depth });
                }
            }
        }
    }

    /// Computes the qualified package scope name and registers package-level symbol and import bindings. Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-INDEX-PACKAGE] | [FLS: fls_66m4rnbssgig]
    fn register_package_scope(&mut self, pkg: &PackageDef, parent_scope: &str) -> String {
        let is_synthetic_multi_root = parent_scope.is_empty() && super::hir_db::is_synthetic_forest_root(pkg);
        let pkg_qname = if is_synthetic_multi_root {
            String::new()
        } else if parent_scope.is_empty() {
            pkg.name.clone()
        } else if pkg.name.is_empty() {
            parent_scope.to_string()
        } else {
            format!("{parent_scope}::{}", pkg.name)
        };
        if !pkg_qname.is_empty() {
            self.package_imports.insert(pkg_qname.clone(), pkg.imports.clone());
            if !pkg.name.is_empty() && pkg.name != pkg_qname {
                self.package_imports.entry(pkg.name.clone()).or_default().extend(pkg.imports.clone());
            }
            self.insert(Symbol {
                name: pkg.name.clone(), qualified_name: pkg_qname.clone(), kind: SymbolKind::Package,
                parent: (!parent_scope.is_empty()).then(|| parent_scope.to_string()),
                type_name: None, direction: None, is_conjugated: false, doc: pkg.doc.clone(),
            });
        } else if !pkg.imports.is_empty() {
            self.package_imports.entry(String::new()).or_default().extend(pkg.imports.clone());
        }
        pkg_qname
    }

    /// Registers non-recursive leaf member definitions declared directly inside `pkg`. Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-INDEX-PACKAGE] | [FLS: fls_66m4rnbssgig]
    fn register_package_leaf_members(&mut self, pkg: &PackageDef, pkg_qname: &str) {
        for attr in &pkg.attribute_defs {
            self.insert(Symbol {
                name: attr.name.clone(), qualified_name: format!("{pkg_qname}::{}", attr.name), kind: SymbolKind::Attribute,
                parent: Some(pkg_qname.to_string()), type_name: Some(attr.type_name.clone()),
                direction: None, is_conjugated: false, doc: attr.doc.clone(),
            });
        }
        for port in &pkg.port_defs {
            self.insert(Symbol {
                name: port.name.clone(), qualified_name: format!("{pkg_qname}::{}", port.name), kind: SymbolKind::Port,
                parent: Some(pkg_qname.to_string()), type_name: Some(port.type_name.clone()),
                direction: Some(port.direction.clone()), is_conjugated: port.is_conjugated, doc: port.doc.clone(),
            });
        }
        for act in &pkg.action_defs { self.insert_scoped_symbol(pkg_qname, &act.name, SymbolKind::Action, &act.doc); }
        for con in &pkg.constraint_defs { self.insert_scoped_symbol(pkg_qname, &con.name, SymbolKind::Constraint, &con.doc); }
        for req in &pkg.requirement_defs { self.insert_scoped_symbol(pkg_qname, &req.name, SymbolKind::Requirement, &req.doc); }
        for state in &pkg.state_defs { self.insert_scoped_symbol(pkg_qname, &state.name, SymbolKind::State, &state.doc); }
        for conn in &pkg.connection_defs { self.insert_scoped_symbol(pkg_qname, &conn.name, SymbolKind::Connection, &conn.doc); }
        for hz in &pkg.hazard_defs { self.insert_scoped_symbol(pkg_qname, &hz.name, SymbolKind::Hazard, &hz.doc); }
        for rk in &pkg.risk_defs { self.insert_scoped_symbol(pkg_qname, &rk.name, SymbolKind::Risk, &rk.doc); }
    }

    /// Iteratively indexes `root_part` and nested subparts in pre-order up to `MAX_AST_DEPTH`. Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-INDEX-PART] | [FLS: fls_66m4rnbssgig]
    fn index_part(&mut self, root_part: &PartDef, root_parent_scope: &str, root_depth: usize) {
        let mut stack = vec![PartIndexFrame { part: root_part, parent_scope: root_parent_scope.to_string(), depth: root_depth }];
        while let Some(frame) = stack.pop() {
            if frame.depth >= MAX_AST_DEPTH { continue; }
            let part_qname = self.register_part_scope_and_members(frame.part, &frame.parent_scope);
            let next_depth = frame.depth.saturating_add(1);
            if next_depth < MAX_AST_DEPTH {
                for subpart in frame.part.parts.iter().rev() {
                    stack.push(PartIndexFrame { part: subpart, parent_scope: part_qname.clone(), depth: next_depth });
                }
            }
        }
    }

    /// Registers `part` and its direct leaf attributes, ports, actions, constraints, requirements, states, and connections. Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-INDEX-PART] | [FLS: fls_66m4rnbssgig]
    fn register_part_scope_and_members(&mut self, part: &PartDef, parent_scope: &str) -> String {
        let part_qname = format!("{parent_scope}::{}", part.name);
        self.insert(Symbol {
            name: part.name.clone(), qualified_name: part_qname.clone(), kind: SymbolKind::Part,
            parent: Some(parent_scope.to_string()), type_name: part.type_name.clone(),
            direction: None, is_conjugated: false, doc: part.doc.clone(),
        });
        for attr in &part.attributes {
            self.insert(Symbol {
                name: attr.name.clone(), qualified_name: format!("{part_qname}::{}", attr.name), kind: SymbolKind::Attribute,
                parent: Some(part_qname.clone()), type_name: Some(attr.type_name.clone()),
                direction: None, is_conjugated: false, doc: attr.doc.clone(),
            });
        }
        for port in &part.ports {
            self.insert(Symbol {
                name: port.name.clone(), qualified_name: format!("{part_qname}::{}", port.name), kind: SymbolKind::Port,
                parent: Some(part_qname.clone()), type_name: Some(port.type_name.clone()),
                direction: Some(port.direction.clone()), is_conjugated: port.is_conjugated, doc: port.doc.clone(),
            });
        }
        for act in &part.actions { self.insert_scoped_symbol(&part_qname, &act.name, SymbolKind::Action, &act.doc); }
        for con in &part.constraints { self.insert_scoped_symbol(&part_qname, &con.name, SymbolKind::Constraint, &con.doc); }
        for req in &part.requirements { self.insert_scoped_symbol(&part_qname, &req.name, SymbolKind::Requirement, &req.doc); }
        for state in &part.states { self.insert_scoped_symbol(&part_qname, &state.name, SymbolKind::State, &state.doc); }
        for conn in &part.connections { self.insert_scoped_symbol(&part_qname, &conn.name, SymbolKind::Connection, &conn.doc); }
        part_qname
    }
}
