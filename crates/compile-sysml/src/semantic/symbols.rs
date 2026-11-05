#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Symbol table and scope resolution for SysML v2 / KerML models.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for SysML v2 symbol resolution.
//! ## 2. Invariants & Mathematical Properties
//! Indexes hierarchical packages, parts, ports, attributes, requirements, and states into deterministic `BTreeMap` lookup tables.
//! ## 3. Preconditions
//! AST inputs conform to `deap_core::sysml_ast` structural invariants within `MAX_AST_DEPTH` (64).
//! ## 4. Postconditions
//! Provides deterministic qualified, scoped, imported, and port-level symbol resolution with zero panicking operations.
//! ## 5. Algorithmic Complexity Bounds
//! Time Complexity: O(log N) qualified lookup; O(I * K) scoped import resolution. Space Complexity: O(N) symbol storage.
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface / State / Action / Resource: Caps AST traversal depth at `MAX_AST_DEPTH` (64) using explicit iterative work-stacks.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603, REQ-SEMANTIC-SYMBOLS-MODULE]

use deap_core::sysml_ast::*;
use std::collections::BTreeMap;

/// Maximum AST nesting depth permitted during SysML v2 symbol indexing and serialization.
///
/// Safety & Traceability: Realises: [REQ-SEMANTIC-SYMBOLS-MAX-AST-DEPTH]
/// Contract: Bounds nested package, part, and state hierarchy traversal to 64 levels (`0..64`).
/// Bounds & Faults: Terminates traversal safely without stack overflow when `depth >= MAX_AST_DEPTH`.
pub const MAX_AST_DEPTH: usize = 64;

/// Classification of SysML v2 entities in the symbol table.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `SymbolKind`.
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for enum `SymbolKind` across all SysML v2 AST definition kinds.
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `SymbolKind` variants.
/// ## 4. Postconditions
/// - Defines the authoritative `SymbolKind` representation.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1). Space Complexity: O(1).
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface / State / Action / Resource: Copyable scalar discriminant with zero allocation.
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLKIND] | SysML v2: PartDef(SymbolKind)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Package,
    Part,
    Port,
    Attribute,
    Action,
    Operation,
    Constraint,
    Requirement,
    State,
    Transition,
    Connection,
    Item,
    Hazard,
    Risk,
    UseCase,
    TestCase,
    Capability,
    Interaction,
}

/// A resolved symbol entry in the symbol table.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `Symbol`.
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `Symbol` across `name`, `qualified_name`, `kind`, and metadata fields.
/// ## 3. Preconditions
/// - Caller supplies well-formed identifier and scope strings.
/// ## 4. Postconditions
/// - Defines the authoritative `Symbol` record stored in `SymbolTable`.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) field access. Space Complexity: O(L) over identifier string lengths.
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface / State / Action / Resource: Value struct with bounded string fields.
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOL] | SysML v2: PartDef(Symbol)
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

/// Symbol table providing scope resolution and entity lookup.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `SymbolTable`.
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic ordering across `symbols`, `by_name`, and `package_imports` `BTreeMap` indices.
/// ## 3. Preconditions
/// - Populated via `SymbolTable::build_from_package` or `SymbolTable::new`.
/// ## 4. Postconditions
/// - Defines the authoritative `SymbolTable` representation for semantic resolution.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N log N) construction. Space Complexity: O(N) symbol entries.
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface / State / Action / Resource: Deterministic `BTreeMap` storage without hash randomization.
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE] | SysML v2: PartDef(SymbolTable)
#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    /// Qualified name -> Symbol (e.g., "Package::Part::Port")
    pub symbols: BTreeMap<String, Symbol>,
    /// Unqualified name -> List of qualified names
    pub by_name: BTreeMap<String, Vec<String>>,
    /// Package qualified name -> List of imports declared in that package
    pub package_imports: BTreeMap<String, Vec<ImportDef>>,
}

impl SymbolTable {
    /// Constructs an empty `SymbolTable`.
    /// Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-NEW] | SysML v2: ActionDef(SymbolTable::new)
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a comprehensive symbol table from a `PackageDef` hierarchy.
    /// Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-BUILD-FROM-PACKAGE] | SysML v2: ActionDef(SymbolTable::build_from_package)
    pub fn build_from_package(pkg: &PackageDef) -> Self {
        let mut table = Self::new();
        table.index_package(pkg, "");
        table
    }

    /// Look up a symbol by exact qualified name.
    /// Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-LOOKUP] | SysML v2: ActionDef(SymbolTable::lookup)
    pub fn lookup(&self, qualified_name: &str) -> Option<&Symbol> {
        self.symbols.get(qualified_name)
    }

    /// Look up a symbol within a given parent scope or fall back to global scope.
    /// Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-LOOKUP-IN-SCOPE] | SysML v2: ActionDef(SymbolTable::lookup_in_scope)
    pub fn lookup_in_scope(&self, scope: &str, name: &str) -> Option<&Symbol> {
        if name.contains("::") {
            if let Some(sym) = self.lookup(name) {
                return Some(sym);
            }
            if !scope.is_empty() {
                let scoped_name = format!("{}::{}", scope, name);
                if let Some(sym) = self.lookup(&scoped_name) {
                    return Some(sym);
                }
            }
        }
        if !scope.is_empty() {
            let scoped_name = format!("{}::{}", scope, name);
            if let Some(sym) = self.symbols.get(&scoped_name) {
                return Some(sym);
            }
        }
        if let Some(sym) = self.lookup_via_imports(scope, name) {
            return Some(sym);
        }
        if let Some(sym) = self.symbols.get(name) {
            return Some(sym);
        }
        self.by_name
            .get(name)
            .and_then(|qnames| qnames.first())
            .and_then(|first| self.symbols.get(first))
    }

    /// Resolves `name` by traversing `package_imports` from `scope` up to enclosing parent scopes.
    /// Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-LOOKUP-IN-SCOPE]
    fn lookup_via_imports(&self, scope: &str, name: &str) -> Option<&Symbol> {
        let mut curr_scope = Some(scope);
        while let Some(s) = curr_scope {
            if let Some(imports) = self.package_imports.get(s) {
                for import in imports {
                    if let Some(sym) = self.resolve_single_import(s, import, name) {
                        return Some(sym);
                    }
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

    /// Evaluates a single wildcard or explicit `ImportDef` within `scope_str` for `name`.
    /// Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-LOOKUP-IN-SCOPE]
    fn resolve_single_import(
        &self,
        scope_str: &str,
        import: &ImportDef,
        name: &str,
    ) -> Option<&Symbol> {
        if import.is_wildcard || import.path.ends_with("::*") || import.path.ends_with("::**") {
            let base = import.path.trim_end_matches("::**").trim_end_matches("::*");
            let candidate_qname = format!("{}::{}", base, name);
            if let Some(sym) = self.symbols.get(&candidate_qname) {
                return Some(sym);
            }
            if import.is_recursive || import.path.ends_with("::**") {
                let prefix = format!("{}::", base);
                if let Some(qnames) = self.by_name.get(name) {
                    for q in qnames {
                        if q.starts_with(&prefix) {
                            if let Some(sym) = self.symbols.get(q) {
                                return Some(sym);
                            }
                        }
                    }
                }
            }
            if let Some((root_scope, _)) = scope_str.split_once("::") {
                let candidate_rooted = format!("{}::{}::{}", root_scope, base, name);
                if let Some(sym) = self.symbols.get(&candidate_rooted) {
                    return Some(sym);
                }
            }
        } else {
            let target_elem = import.path.rsplit("::").next().unwrap_or(&import.path);
            if target_elem == name {
                if let Some(sym) = self.symbols.get(&import.path) {
                    return Some(sym);
                }
                if let Some((root_scope, _)) = scope_str.split_once("::") {
                    let candidate_rooted = format!("{}::{}", root_scope, import.path);
                    if let Some(sym) = self.symbols.get(&candidate_rooted) {
                        return Some(sym);
                    }
                }
            }
        }
        None
    }

    /// Look up a port on a part given `"PartName.portName"` or in a specific part scope.
    /// Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-LOOKUP-PORT] | SysML v2: ActionDef(SymbolTable::lookup_port)
    pub fn lookup_port(&self, port_ref: &str, current_part: Option<&str>) -> Option<&Symbol> {
        if let Some((part_name, port_name)) = port_ref.split_once('.') {
            if !port_name.contains('.') {
                if let Some(sym) = self.lookup_dotted_port(part_name, port_name) {
                    return Some(sym);
                }
            }
        } else if let Some(part) = current_part {
            let candidate_q = format!("{}::{}", part, port_ref);
            if let Some(sym) = self
                .symbols
                .get(&candidate_q)
                .filter(|s| s.kind == SymbolKind::Port)
            {
                return Some(sym);
            }
            if let Some(part_qnames) = self.by_name.get(part) {
                for part_q in part_qnames {
                    let candidate_scoped_q = format!("{}::{}", part_q, port_ref);
                    if let Some(sym) = self
                        .symbols
                        .get(&candidate_scoped_q)
                        .filter(|s| s.kind == SymbolKind::Port)
                    {
                        return Some(sym);
                    }
                }
            }
        }
        if let Some(port_qnames) = self.by_name.get(port_ref) {
            for q in port_qnames {
                if let Some(sym) = self.symbols.get(q).filter(|s| s.kind == SymbolKind::Port) {
                    return Some(sym);
                }
            }
        }
        None
    }

    /// Resolves `"part_name.port_name"` across direct part symbols and typed part definitions.
    /// Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-LOOKUP-PORT]
    fn lookup_dotted_port(&self, part_name: &str, port_name: &str) -> Option<&Symbol> {
        let part_qnames = self.by_name.get(part_name)?;
        for part_q in part_qnames {
            let candidate_port_q = format!("{}::{}", part_q, port_name);
            if let Some(sym) = self
                .symbols
                .get(&candidate_port_q)
                .filter(|s| s.kind == SymbolKind::Port)
            {
                return Some(sym);
            }
            if let Some(type_name) = self
                .symbols
                .get(part_q)
                .and_then(|part_sym| part_sym.type_name.as_ref())
            {
                if let Some(type_qnames) = self.by_name.get(type_name) {
                    for type_q in type_qnames {
                        let candidate_type_port_q = format!("{}::{}", type_q, port_name);
                        if let Some(sym) = self
                            .symbols
                            .get(&candidate_type_port_q)
                            .filter(|s| s.kind == SymbolKind::Port)
                        {
                            return Some(sym);
                        }
                    }
                }
            }
        }
        None
    }

    /// Get all symbols of a specific kind.
    /// Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-ALL-OF-KIND] | SysML v2: ActionDef(SymbolTable::all_of_kind)
    pub fn all_of_kind(&self, kind: SymbolKind) -> Vec<&Symbol> {
        self.symbols.values().filter(|s| s.kind == kind).collect()
    }

    /// Inserts `symbol` into both the qualified `symbols` map and unqualified `by_name` index.
    /// Realises: [REQ-SEMANTIC-SYMBOLS-SYMBOLTABLE-INSERT]
    pub(crate) fn insert(&mut self, symbol: Symbol) {
        self.by_name
            .entry(symbol.name.clone())
            .or_default()
            .push(symbol.qualified_name.clone());
        self.symbols.insert(symbol.qualified_name.clone(), symbol);
    }
}
