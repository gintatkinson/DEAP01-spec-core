#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Unit tests for SysML v2 `SymbolTable` indexing, import resolution, and depth bounds.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C verification objectives.
//! ## 2. Invariants & Mathematical Properties
//! Verifies deterministic pre-order indexing, cross-package import resolution, and `MAX_AST_DEPTH` (64) enforcement.
//! ## 3. Preconditions
//! Executed under standard `cargo test` harness with bounded stack threads.
//! ## 4. Postconditions
//! Asserts exact symbol table entries, import precedence, and bounded stack usage on deep trees.
//! ## 5. Algorithmic Complexity Bounds
//! Time Complexity: O(N) over synthetic AST fixtures. Space Complexity: O(N) heap work-stack.
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface / State / Action / Resource: Zero `.unwrap()` or `.expect()` calls in test assertions.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603, REQ-SEMANTIC-SYMBOLS-MODULE]

use super::symbols::{SymbolKind, SymbolTable, MAX_AST_DEPTH};
use deap_core::sysml_ast::{ImportDef, PackageDef, PartDef, PortDef};

#[test]
fn test_symbol_table_indexing_and_lookup() -> Result<(), Box<dyn std::error::Error>> {
    let pkg = PackageDef {
        name: "Platform_SSOT".to_string(),
        part_defs: vec![
            PartDef {
                name: "ComputeNode".to_string(),
                ports: vec![PortDef {
                    name: "bus_out".to_string(),
                    type_name: "CANBus".to_string(),
                    direction: "out".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            },
            PartDef {
                name: "Actuator".to_string(),
                ports: vec![PortDef {
                    name: "bus_in".to_string(),
                    type_name: "CANBus".to_string(),
                    direction: "in".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let table = SymbolTable::build_from_package(&pkg);
    assert!(table.lookup("Platform_SSOT::ComputeNode").is_some());
    assert!(table.lookup("Platform_SSOT::ComputeNode::bus_out").is_some());

    let port_sym = table
        .lookup_port("ComputeNode.bus_out", None)
        .ok_or("expected value is absent")?;
    assert_eq!(port_sym.name, "bus_out");
    assert_eq!(port_sym.direction.as_deref(), Some("out"));
    Ok(())
}

fn build_competing_and_source_packages() -> (PackageDef, PackageDef) {
    let pkg_competing = PackageDef {
        name: "AAA_Other".to_string(),
        part_defs: vec![PartDef { name: "EngineA".to_string(), is_def: true, ..Default::default() }],
        ..Default::default()
    };
    let pkg_a = PackageDef {
        name: "Subsystem_A".to_string(),
        part_defs: vec![PartDef { name: "EngineA".to_string(), is_def: true, ..Default::default() }],
        ..Default::default()
    };
    (pkg_competing, pkg_a)
}

#[test]
fn test_symbol_table_cross_package_import_lookup() -> Result<(), Box<dyn std::error::Error>> {
    let (pkg_competing, pkg_a) = build_competing_and_source_packages();
    let pkg_b = PackageDef {
        name: "Subsystem_B".to_string(),
        imports: vec![ImportDef { path: "Subsystem_A".to_string(), is_wildcard: true, ..Default::default() }],
        part_defs: vec![PartDef {
            name: "UsageB".to_string(),
            is_def: false,
            type_name: Some("EngineA".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    };
    let root_pkg = PackageDef {
        name: String::new(),
        packages: vec![pkg_competing.clone(), pkg_a.clone(), pkg_b],
        ..Default::default()
    };

    let table = SymbolTable::build_from_package(&root_pkg);
    assert_eq!(table.package_imports.get("Subsystem_B").ok_or("absent")?.len(), 1);
    let resolved = table.lookup_in_scope("Subsystem_B", "EngineA").ok_or("absent")?;
    assert_eq!(resolved.qualified_name, "Subsystem_A::EngineA");

    let pkg_d = PackageDef {
        name: "Subsystem_D".to_string(),
        imports: vec![ImportDef { path: "Subsystem_A::EngineA".to_string(), is_wildcard: false, ..Default::default() }],
        part_defs: vec![PartDef {
            name: "UsageD".to_string(),
            is_def: false,
            type_name: Some("EngineA".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    };
    let root_with_d = PackageDef {
        name: "Root".to_string(),
        packages: vec![pkg_competing, pkg_a, pkg_d],
        ..Default::default()
    };
    let table_d = SymbolTable::build_from_package(&root_with_d);
    let resolved_d = table_d.lookup_in_scope("Root::Subsystem_D", "EngineA").ok_or("absent")?;
    assert_eq!(resolved_d.qualified_name, "Root::Subsystem_A::EngineA");
    Ok(())
}

#[test]
fn test_symbol_table_package_depth_bound_64_and_65() {
    let total_levels: usize = 65;
    let mut curr = PackageDef { name: format!("Pkg_{}", total_levels.saturating_sub(1)), ..Default::default() };
    for idx in (0..total_levels.saturating_sub(1)).rev() {
        curr = PackageDef { name: format!("Pkg_{idx}"), packages: vec![curr], ..Default::default() };
    }

    let table = SymbolTable::build_from_package(&curr);
    let mut qname = String::from("Pkg_0");
    assert!(table.lookup(&qname).is_some(), "Root Pkg_0 at depth 0 must be indexed");
    for idx in 1..64usize {
        qname = format!("{qname}::Pkg_{idx}");
        assert!(table.lookup(&qname).is_some(), "Pkg_{idx} at depth {idx} (< 64) must be indexed");
    }
    let beyond_qname = format!("{qname}::Pkg_64");
    assert!(table.lookup(&beyond_qname).is_none(), "Pkg_64 at depth 64 must not be indexed");
}

#[test]
fn test_symbol_table_part_depth_bound_64_and_65() {
    let max_part_depth: usize = 64;
    let mut curr_part = PartDef { name: format!("Part_{max_part_depth}"), ..Default::default() };
    for idx in (1..max_part_depth).rev() {
        curr_part = PartDef { name: format!("Part_{idx}"), parts: vec![curr_part], ..Default::default() };
    }
    let root_pkg = PackageDef { name: "Pkg_0".to_string(), part_defs: vec![curr_part], ..Default::default() };

    let table = SymbolTable::build_from_package(&root_pkg);
    let mut qname = String::from("Pkg_0");
    assert!(table.lookup(&qname).is_some());
    for idx in 1..64usize {
        qname = format!("{qname}::Part_{idx}");
        assert!(table.lookup(&qname).is_some(), "Part_{idx} at depth {idx} (< 64) must be indexed");
    }
    let beyond_qname = format!("{qname}::Part_64");
    assert!(table.lookup(&beyond_qname).is_none(), "Part_64 at depth 64 must not be indexed");
}

fn verify_preorder_shared_node_indexing() -> Result<(), String> {
    let branch_pkg = PackageDef {
        name: "Root".to_string(),
        packages: vec![
            PackageDef {
                name: "Alpha".to_string(),
                part_defs: vec![PartDef {
                    name: "SharedNode".to_string(),
                    parts: vec![PartDef { name: "SharedNode".to_string(), ..Default::default() }],
                    ..Default::default()
                }],
                ..Default::default()
            },
            PackageDef {
                name: "Beta".to_string(),
                part_defs: vec![PartDef { name: "SharedNode".to_string(), ..Default::default() }],
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    let branch_table = SymbolTable::build_from_package(&branch_pkg);
    let shared = branch_table.by_name.get("SharedNode").ok_or_else(|| "missing SharedNode".to_string())?;
    assert_eq!(
        shared,
        &vec![
            "Root::Alpha::SharedNode".to_string(),
            "Root::Alpha::SharedNode::SharedNode".to_string(),
            "Root::Beta::SharedNode".to_string(),
        ]
    );
    Ok(())
}

#[test]
fn test_symbol_table_deep_nesting_bounded_stack_and_preorder() -> Result<(), String> {
    assert_eq!(MAX_AST_DEPTH, 64);
    verify_preorder_shared_node_indexing()?;

    let deep_levels: usize = 2_000;
    let mut deep_pkg = PackageDef { name: format!("Pkg_{}", deep_levels.saturating_sub(1)), ..Default::default() };
    for idx in (0..deep_levels.saturating_sub(1)).rev() {
        deep_pkg = PackageDef { name: format!("Pkg_{idx}"), packages: vec![deep_pkg], ..Default::default() };
    }
    let mut deep_part = PartDef { name: format!("Part_{}", deep_levels.saturating_sub(1)), ..Default::default() };
    for idx in (1..deep_levels.saturating_sub(1)).rev() {
        deep_part = PartDef { name: format!("Part_{idx}"), parts: vec![deep_part], ..Default::default() };
    }
    let deep_part_root = PackageDef { name: "PartRoot".to_string(), part_defs: vec![deep_part], ..Default::default() };

    let (pkg_count, part_count) = std::thread::scope(|s| -> Result<(usize, usize), String> {
        let handle = std::thread::Builder::new()
            .name("bounded_symbols_2000".to_string())
            .stack_size(256 * 1024)
            .spawn_scoped(s, || {
                let t_pkg = SymbolTable::build_from_package(&deep_pkg);
                let t_part = SymbolTable::build_from_package(&deep_part_root);
                (t_pkg.symbols.len(), t_part.symbols.len())
            })
            .map_err(|e| format!("failed to spawn scoped thread: {e}"))?;
        handle.join().map_err(|_| "bounded-stack symbol thread panicked".to_string())
    })?;

    assert_eq!(pkg_count, MAX_AST_DEPTH);
    assert_eq!(part_count, MAX_AST_DEPTH);

    let mut pkg_drain = vec![deep_pkg];
    while let Some(mut node) = pkg_drain.pop() {
        pkg_drain.append(&mut node.packages);
    }
    let mut part_drain = deep_part_root.part_defs;
    while let Some(mut node) = part_drain.pop() {
        part_drain.append(&mut node.parts);
    }
    Ok(())
}

#[test]
fn test_lookup_port_split_once_and_utf8_identifiers() -> Result<(), Box<dyn std::error::Error>> {
    let pkg = PackageDef {
        name: "システム_SSOT".to_string(),
        part_defs: vec![PartDef {
            name: "センサー".to_string(),
            ports: vec![PortDef {
                name: "出力ポート".to_string(),
                type_name: "CANBus".to_string(),
                direction: "out".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    };

    let table = SymbolTable::build_from_package(&pkg);
    let resolved = table.lookup_port("センサー.出力ポート", None).ok_or("expected UTF-8 port")?;
    assert_eq!(resolved.name, "出力ポート");
    assert_eq!(resolved.kind, SymbolKind::Port);

    assert!(table.lookup_port("システム_SSOT.センサー.出力ポート", Some("センサー")).is_none());
    assert!(table.lookup_port(".出力ポート", Some("センサー")).is_none());
    assert!(table.lookup_port("センサー.", Some("センサー")).is_none());
    assert!(table.lookup_port("", Some("センサー")).is_none());
    Ok(())
}
