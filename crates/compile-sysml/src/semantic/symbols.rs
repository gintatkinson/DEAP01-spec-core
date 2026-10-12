//! Symbol table and scope resolution for SysML v2 / KerML models.

use deap_core::sysml_ast::*;
use std::collections::BTreeMap;

/// Classification of SysML v2 entities in the symbol table.
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
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a comprehensive symbol table from a PackageDef hierarchy.
    pub fn build_from_package(pkg: &PackageDef) -> Self {
        let mut table = Self::new();
        table.index_package(pkg, "");
        table
    }

    /// Look up a symbol by exact qualified name.
    pub fn lookup(&self, qualified_name: &str) -> Option<&Symbol> {
        self.symbols.get(qualified_name)
    }

    /// Look up a symbol within a given parent scope or fall back to global scope.
    pub fn lookup_in_scope(&self, scope: &str, name: &str) -> Option<&Symbol> {
        // 1. If `name` contains `::`, call `self.lookup(name)`.
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

        // 2. If `!scope.is_empty()`, check direct scoped name `format!("{}::{}", scope, name)`.
        if !scope.is_empty() {
            let scoped_name = format!("{}::{}", scope, name);
            if let Some(sym) = self.symbols.get(&scoped_name) {
                return Some(sym);
            }
        }

        // 3. Check imported namespaces in `scope` (or traversing enclosing parent scopes):
        let mut curr_scope = Some(scope);
        while let Some(s) = curr_scope {
            if let Some(imports) = self.package_imports.get(s) {
                for import in imports {
                    // Wildcard imports (e.g. import Subsystem_1::*; where path is "Subsystem_1::*" or "Subsystem_1")
                    if import.is_wildcard
                        || import.path.ends_with("::*")
                        || import.path.ends_with("::**")
                    {
                        let base = import
                            .path
                            .trim_end_matches("::**")
                            .trim_end_matches("::*");

                        // Check direct qualified name under base prefix
                        let candidate_qname = format!("{}::{}", base, name);
                        if let Some(sym) = self.symbols.get(&candidate_qname) {
                            return Some(sym);
                        }

                        // Also check recursive wildcard imports if applicable
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

                        // Also check if base is a relative package inside the enclosing root scope
                        if let Some(idx) = s.find("::") {
                            let root_scope = &s[..idx];
                            let candidate_rooted = format!("{}::{}::{}", root_scope, base, name);
                            if let Some(sym) = self.symbols.get(&candidate_rooted) {
                                return Some(sym);
                            }
                        }
                    } else {
                        // Explicit element import (e.g. import Subsystem_1::SystemVisionEngine;)
                        let target_elem = import.path.rsplit("::").next().unwrap_or(&import.path);
                        if target_elem == name {
                            if let Some(sym) = self.symbols.get(&import.path) {
                                return Some(sym);
                            }
                            // Also check relative to root scope if applicable
                            if let Some(idx) = s.find("::") {
                                let root_scope = &s[..idx];
                                let candidate_rooted = format!("{}::{}", root_scope, import.path);
                                if let Some(sym) = self.symbols.get(&candidate_rooted) {
                                    return Some(sym);
                                }
                            }
                        }
                    }
                }
            }

            // Traverse to enclosing parent scope
            curr_scope = if let Some(idx) = s.rfind("::") {
                Some(&s[..idx])
            } else if !s.is_empty() {
                Some("")
            } else {
                None
            };
        }

        // 4. Fall back to direct name in global scope
        if let Some(sym) = self.symbols.get(name) {
            return Some(sym);
        }

        // 5. Fall back to unqualified search in self.by_name
        if let Some(qnames) = self.by_name.get(name) {
            if let Some(first) = qnames.first() {
                return self.symbols.get(first);
            }
        }

        None
    }

    /// Look up a port on a part given "PartName.portName" or in a specific part scope.
    pub fn lookup_port(&self, port_ref: &str, current_part: Option<&str>) -> Option<&Symbol> {
        if port_ref.contains('.') {
            let parts: Vec<&str> = port_ref.split('.').collect();
            if parts.len() == 2 {
                let part_name = parts[0];
                let port_name = parts[1];

                // Find candidate part
                if let Some(part_qnames) = self.by_name.get(part_name) {
                    for part_q in part_qnames {
                        let candidate_port_q = format!("{}::{}", part_q, port_name);
                        if let Some(sym) = self.symbols.get(&candidate_port_q) {
                            if sym.kind == SymbolKind::Port {
                                return Some(sym);
                            }
                        }

                        // Also check via the part's type definition if the part is a typed instance
                        if let Some(part_sym) = self.symbols.get(part_q) {
                            if let Some(ref type_name) = part_sym.type_name {
                                if let Some(type_qnames) = self.by_name.get(type_name) {
                                    for type_q in type_qnames {
                                        let candidate_type_port_q = format!("{}::{}", type_q, port_name);
                                        if let Some(sym) = self.symbols.get(&candidate_type_port_q) {
                                            if sym.kind == SymbolKind::Port {
                                                return Some(sym);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else if let Some(part) = current_part {
            let candidate_q = format!("{}::{}", part, port_ref);
            if let Some(sym) = self.symbols.get(&candidate_q) {
                if sym.kind == SymbolKind::Port {
                    return Some(sym);
                }
            }

            if let Some(part_qnames) = self.by_name.get(part) {
                for part_q in part_qnames {
                    let candidate_scoped_q = format!("{}::{}", part_q, port_ref);
                    if let Some(sym) = self.symbols.get(&candidate_scoped_q) {
                        if sym.kind == SymbolKind::Port {
                            return Some(sym);
                        }
                    }
                }
            }
        }

        // Fall back to unqualified search
        if let Some(port_qnames) = self.by_name.get(port_ref) {
            for q in port_qnames {
                if let Some(sym) = self.symbols.get(q) {
                    if sym.kind == SymbolKind::Port {
                        return Some(sym);
                    }
                }
            }
        }

        None
    }

    /// Get all symbols of a specific kind.
    pub fn all_of_kind(&self, kind: SymbolKind) -> Vec<&Symbol> {
        self.symbols.values().filter(|s| s.kind == kind).collect()
    }

    fn insert(&mut self, symbol: Symbol) {
        self.by_name
            .entry(symbol.name.clone())
            .or_default()
            .push(symbol.qualified_name.clone());
        self.symbols.insert(symbol.qualified_name.clone(), symbol);
    }

    fn index_package(&mut self, pkg: &PackageDef, parent_scope: &str) {
        let pkg_qname = if parent_scope.is_empty() {
            pkg.name.clone()
        } else if pkg.name.is_empty() {
            parent_scope.to_string()
        } else {
            format!("{}::{}", parent_scope, pkg.name)
        };

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
            parent: if parent_scope.is_empty() {
                None
            } else {
                Some(parent_scope.to_string())
            },
            type_name: None,
            direction: None,
            is_conjugated: false,
            doc: pkg.doc.clone(),
        });

        for attr in &pkg.attribute_defs {
            self.insert(Symbol {
                name: attr.name.clone(),
                qualified_name: format!("{}::{}", pkg_qname, attr.name),
                kind: SymbolKind::Attribute,
                parent: Some(pkg_qname.clone()),
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
                parent: Some(pkg_qname.clone()),
                type_name: Some(port.type_name.clone()),
                direction: Some(port.direction.clone()),
                is_conjugated: port.is_conjugated,
                doc: port.doc.clone(),
            });
        }

        for act in &pkg.action_defs {
            self.insert(Symbol {
                name: act.name.clone(),
                qualified_name: format!("{}::{}", pkg_qname, act.name),
                kind: SymbolKind::Action,
                parent: Some(pkg_qname.clone()),
                type_name: None,
                direction: None,
                is_conjugated: false,
                doc: act.doc.clone(),
            });
        }

        for con in &pkg.constraint_defs {
            self.insert(Symbol {
                name: con.name.clone(),
                qualified_name: format!("{}::{}", pkg_qname, con.name),
                kind: SymbolKind::Constraint,
                parent: Some(pkg_qname.clone()),
                type_name: None,
                direction: None,
                is_conjugated: false,
                doc: con.doc.clone(),
            });
        }

        for req in &pkg.requirement_defs {
            self.insert(Symbol {
                name: req.name.clone(),
                qualified_name: format!("{}::{}", pkg_qname, req.name),
                kind: SymbolKind::Requirement,
                parent: Some(pkg_qname.clone()),
                type_name: None,
                direction: None,
                is_conjugated: false,
                doc: req.doc.clone(),
            });
        }

        for state in &pkg.state_defs {
            self.insert(Symbol {
                name: state.name.clone(),
                qualified_name: format!("{}::{}", pkg_qname, state.name),
                kind: SymbolKind::State,
                parent: Some(pkg_qname.clone()),
                type_name: None,
                direction: None,
                is_conjugated: false,
                doc: state.doc.clone(),
            });
        }

        for conn in &pkg.connection_defs {
            self.insert(Symbol {
                name: conn.name.clone(),
                qualified_name: format!("{}::{}", pkg_qname, conn.name),
                kind: SymbolKind::Connection,
                parent: Some(pkg_qname.clone()),
                type_name: None,
                direction: None,
                is_conjugated: false,
                doc: conn.doc.clone(),
            });
        }

        for hz in &pkg.hazard_defs {
            self.insert(Symbol {
                name: hz.name.clone(),
                qualified_name: format!("{}::{}", pkg_qname, hz.name),
                kind: SymbolKind::Hazard,
                parent: Some(pkg_qname.clone()),
                type_name: None,
                direction: None,
                is_conjugated: false,
                doc: hz.doc.clone(),
            });
        }

        for rk in &pkg.risk_defs {
            self.insert(Symbol {
                name: rk.name.clone(),
                qualified_name: format!("{}::{}", pkg_qname, rk.name),
                kind: SymbolKind::Risk,
                parent: Some(pkg_qname.clone()),
                type_name: None,
                direction: None,
                is_conjugated: false,
                doc: rk.doc.clone(),
            });
        }

        for part in &pkg.part_defs {
            self.index_part(part, &pkg_qname);
        }

        for subpkg in &pkg.packages {
            self.index_package(subpkg, &pkg_qname);
        }
    }

    fn index_part(&mut self, part: &PartDef, parent_scope: &str) {
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
            self.insert(Symbol {
                name: act.name.clone(),
                qualified_name: format!("{}::{}", part_qname, act.name),
                kind: SymbolKind::Action,
                parent: Some(part_qname.clone()),
                type_name: None,
                direction: None,
                is_conjugated: false,
                doc: act.doc.clone(),
            });
        }

        for con in &part.constraints {
            self.insert(Symbol {
                name: con.name.clone(),
                qualified_name: format!("{}::{}", part_qname, con.name),
                kind: SymbolKind::Constraint,
                parent: Some(part_qname.clone()),
                type_name: None,
                direction: None,
                is_conjugated: false,
                doc: con.doc.clone(),
            });
        }

        for req in &part.requirements {
            self.insert(Symbol {
                name: req.name.clone(),
                qualified_name: format!("{}::{}", part_qname, req.name),
                kind: SymbolKind::Requirement,
                parent: Some(part_qname.clone()),
                type_name: None,
                direction: None,
                is_conjugated: false,
                doc: req.doc.clone(),
            });
        }

        for state in &part.states {
            self.insert(Symbol {
                name: state.name.clone(),
                qualified_name: format!("{}::{}", part_qname, state.name),
                kind: SymbolKind::State,
                parent: Some(part_qname.clone()),
                type_name: None,
                direction: None,
                is_conjugated: false,
                doc: state.doc.clone(),
            });
        }

        for conn in &part.connections {
            self.insert(Symbol {
                name: conn.name.clone(),
                qualified_name: format!("{}::{}", part_qname, conn.name),
                kind: SymbolKind::Connection,
                parent: Some(part_qname.clone()),
                type_name: None,
                direction: None,
                is_conjugated: false,
                doc: conn.doc.clone(),
            });
        }

        for subpart in &part.parts {
            self.index_part(subpart, &part_qname);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table_indexing_and_lookup() {
        let pkg = PackageDef {
            name: "Vehicle_SSOT".to_string(),
            part_defs: vec![
                PartDef {
                    name: "FlightComputer".to_string(),
                    ports: vec![
                        PortDef {
                            name: "bus_out".to_string(),
                            type_name: "CANBus".to_string(),
                            direction: "out".to_string(),
                            is_conjugated: false,
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                },
                PartDef {
                    name: "Actuator".to_string(),
                    ports: vec![
                        PortDef {
                            name: "bus_in".to_string(),
                            type_name: "CANBus".to_string(),
                            direction: "in".to_string(),
                            is_conjugated: false,
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let table = SymbolTable::build_from_package(&pkg);
        assert!(table.lookup("Vehicle_SSOT::FlightComputer").is_some());
        assert!(table.lookup("Vehicle_SSOT::FlightComputer::bus_out").is_some());

        let port_sym = table.lookup_port("FlightComputer.bus_out", None).unwrap();
        assert_eq!(port_sym.name, "bus_out");
        assert_eq!(port_sym.direction.as_deref(), Some("out"));
    }

    #[test]
    fn test_symbol_table_cross_package_import_lookup() {
        let pkg_competing = PackageDef {
            name: "AAA_Other".to_string(),
            part_defs: vec![
                PartDef {
                    name: "EngineA".to_string(),
                    is_def: true,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let pkg_a = PackageDef {
            name: "Subsystem_A".to_string(),
            part_defs: vec![
                PartDef {
                    name: "EngineA".to_string(),
                    is_def: true,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let pkg_b = PackageDef {
            name: "Subsystem_B".to_string(),
            imports: vec![
                ImportDef {
                    path: "Subsystem_A".to_string(),
                    is_wildcard: true,
                    ..Default::default()
                },
            ],
            part_defs: vec![
                PartDef {
                    name: "UsageB".to_string(),
                    is_def: false,
                    type_name: Some("EngineA".to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let root_pkg = PackageDef {
            name: "".to_string(),
            packages: vec![pkg_competing.clone(), pkg_a.clone(), pkg_b],
            ..Default::default()
        };

        let table = SymbolTable::build_from_package(&root_pkg);

        // Verify package_imports records the imports for Subsystem_B
        assert!(table.package_imports.contains_key("Subsystem_B"));
        assert_eq!(table.package_imports.get("Subsystem_B").unwrap().len(), 1);

        // Verify that lookup_in_scope("Subsystem_B", "EngineA") resolves to Subsystem_A::EngineA
        // rather than AAA_Other::EngineA or failing.
        let resolved = table.lookup_in_scope("Subsystem_B", "EngineA");
        assert!(resolved.is_some(), "EngineA should resolve via import in Subsystem_B");
        assert_eq!(
            resolved.unwrap().qualified_name,
            "Subsystem_A::EngineA",
            "Import scoping should take precedence over unqualified fallback"
        );

        // Explicit element import test
        let pkg_d = PackageDef {
            name: "Subsystem_D".to_string(),
            imports: vec![
                ImportDef {
                    path: "Subsystem_A::EngineA".to_string(),
                    is_wildcard: false,
                    ..Default::default()
                },
            ],
            part_defs: vec![
                PartDef {
                    name: "UsageD".to_string(),
                    is_def: false,
                    type_name: Some("EngineA".to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let root_with_d = PackageDef {
            name: "Root".to_string(),
            packages: vec![pkg_competing, pkg_a, pkg_d],
            ..Default::default()
        };
        let table_d = SymbolTable::build_from_package(&root_with_d);
        let resolved_d = table_d.lookup_in_scope("Root::Subsystem_D", "EngineA");
        assert!(resolved_d.is_some(), "EngineA should resolve via explicit import in Root::Subsystem_D");
        assert_eq!(resolved_d.unwrap().qualified_name, "Root::Subsystem_A::EngineA");
    }
}
