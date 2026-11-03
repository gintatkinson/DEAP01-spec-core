//! Unit tests for Check 41 (C-ABI solver bridge and SQLite seed parity) and JSON entity extraction.
//! Realises: [REQ-VERIFY-BASELINE-CODEBASE-CHECKS], [REQ-CHECK-41-SOLVER-BRIDGE-SQLITE-SEED-PARITY]

use std::collections::BTreeSet;
use std::fs;

use super::codebase::*;
use super::codebase_tests_a::ScopedTempDir;

#[test]
fn test_check_41_positive_c_abi_and_seed_parity() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = ScopedTempDir::new("check_41_pos")?;
    let assets_dir = tmp.path.join("app_flutter/assets");
    let seeds_dir = tmp.path.join("app_flutter/lib/data/seeds");
    let data_dir = tmp.path.join("app_flutter/lib/data");
    fs::create_dir_all(&assets_dir)?;
    fs::create_dir_all(&seeds_dir)?;
    fs::create_dir_all(&data_dir)?;

    let topology_json = r#"{
  "nodes": [
    {"id": "SENSOR_01", "name": "Primary Sensor"},
    {"id": "CONTROLLER_02", "name": "Main Controller"}
  ]
}"#;
    fs::write(assets_dir.join("topology_data.json"), topology_json)?;

    let seed_dart = r#"
// Generated domain seed
Future<void> seedDatabase(Database db) async {
  // 5 canonical tables:
  // type_definitions, type_attributes, type_relations, properties, instances
  await db.execute("INSERT INTO type_definitions VALUES (...)");
  await db.execute("INSERT INTO type_attributes VALUES (...)");
  await db.execute("INSERT INTO type_relations VALUES (...)");
  await db.execute("INSERT INTO instances (id) VALUES ('SENSOR_01'), ('CONTROLLER_02')");
  await db.execute("INSERT INTO properties VALUES (...)");
}
"#;
    fs::write(seeds_dir.join("generated_domain_seed.g.dart"), seed_dart)?;

    let bridge_dart = r#"
class SolverBridge {
  final DynamicLibrary _dylib;
  SolverBridge(this._dylib);

  Uint8List evaluate(Uint8List payload) {
    // Pure thin C-ABI delegation shim calling native cdylib
    return _callNativeFfi(payload);
  }
}
"#;
    fs::write(data_dir.join("solver_bridge.dart"), bridge_dart)?;

    let res = check_41_solver_bridge_and_sqlite_seed_parity_gate(&tmp.path);
    assert!(
        res.is_ok(),
        "Positive C-ABI and seed parity must pass Check 41: {:?}",
        res.err()
    );
    Ok(())
}

#[test]
fn test_check_41_negative_duplicate_math_and_missing_seeds(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = ScopedTempDir::new("check_41_neg")?;
    let solver_crate_dir = tmp.path.join("crates/domain-solver/src");
    let assets_dir = tmp.path.join("app_flutter/assets");
    let seeds_dir = tmp.path.join("app_flutter/lib/data/seeds");
    let data_dir = tmp.path.join("app_flutter/lib/data");
    fs::create_dir_all(&solver_crate_dir)?;
    fs::create_dir_all(&assets_dir)?;
    fs::create_dir_all(&seeds_dir)?;
    fs::create_dir_all(&data_dir)?;

    fs::write(solver_crate_dir.join("lib.rs"), "// solver crate")?;
    fs::write(solver_crate_dir.join("solver.rs"), "// solver arithmetic")?;

    let topology_json = r#"{
  "nodes": [
    {"id": "SENSOR_01", "name": "Primary Sensor"},
    {"id": "SENSOR_02_MISSING", "name": "Missing Sensor"}
  ]
}"#;
    fs::write(assets_dir.join("topology_data.json"), topology_json)?;

    // Seed file misses SENSOR_02_MISSING and misses type_relations table
    let defective_seed_dart = r#"
Future<void> seedDatabase(Database db) async {
  await db.execute("INSERT INTO type_definitions VALUES (...)");
  await db.execute("INSERT INTO type_attributes VALUES (...)");
  await db.execute("INSERT INTO instances (id) VALUES ('SENSOR_01')");
  await db.execute("INSERT INTO properties VALUES (...)");
}
"#;
    fs::write(
        seeds_dir.join("generated_domain_seed.g.dart"),
        defective_seed_dart,
    )?;

    // Bridge file has duplicate candidate generator and Pareto scoring
    let defective_bridge_dart = r#"
class SolverBridge {
  List<Candidate> _buildCandidates() {
    final paretoScore = alphaCovOffset * 0.8 + reserveBoostPct * 0.2;
    return [Candidate(score: paretoScore, violationCount: 0)];
  }
}
"#;
    fs::write(
        data_dir.join("domain_solver_bridge.dart"),
        defective_bridge_dart,
    )?;

    let res = check_41_solver_bridge_and_sqlite_seed_parity_gate(&tmp.path);
    assert!(
        res.is_err(),
        "Duplicate math and missing seed entities must fail Check 41"
    );
    let errs = res.err().ok_or("expected Err")?;
    assert!(errs.iter().any(|e| e.contains("SENSOR_02_MISSING")));
    assert!(errs.iter().any(|e| e.contains("type_relations")));
    assert!(errs.iter().any(|e| e.contains("_buildCandidates")));
    Ok(())
}

#[test]
fn test_extract_strings_from_json_value_iterative_deep_and_order_parity(
) -> Result<(), Box<dyn std::error::Error>> {
    let sample = serde_json::from_str::<serde_json::Value>(
        r#"{
            "nodes": [
                {
                    "id": "ZONE-NORTH-01",
                    "label": "Unit-Alpha (P1)",
                    "parentId": "SECTOR_MAIN",
                    "properties": {
                        "bus_tag": "LINK-BUS-07",
                        "plain": "ignoredplain"
                    }
                },
                {
                    "sourceId": "SRC_NODE_A",
                    "targetId": "TGT_NODE_B"
                }
            ]
        }"#,
    )?;
    let mut extracted = BTreeSet::new();
    extract_strings_from_json_value(&sample, &mut extracted);
    assert!(extracted.contains("ZONE-NORTH-01"));
    assert!(extracted.contains("NORTH-01"));
    assert!(extracted.contains("Unit-Alpha (P1)"));
    assert!(extracted.contains("Unit-Alpha"));
    assert!(extracted.contains("SECTOR_MAIN"));
    assert!(extracted.contains("LINK-BUS-07"));
    assert!(extracted.contains("BUS-07"));
    assert!(extracted.contains("SRC_NODE_A"));
    assert!(extracted.contains("TGT_NODE_B"));
    assert!(!extracted.contains("ignoredplain"));

    let mut leaf_map = serde_json::Map::new();
    leaf_map.insert(
        "id".to_string(),
        serde_json::Value::String("DEEP-ENTITY-42".to_string()),
    );
    let mut deep_val = serde_json::Value::Object(leaf_map);
    for _ in 0..3000 {
        deep_val = serde_json::Value::Array(vec![deep_val]);
    }

    let deep_extracted = std::thread::scope(|scope| -> Result<BTreeSet<String>, String> {
        let handle = std::thread::Builder::new()
            .name("bounded_json_walk".to_string())
            .stack_size(128 * 1024)
            .spawn_scoped(scope, || {
                let mut set = BTreeSet::new();
                extract_strings_from_json_value(&deep_val, &mut set);
                set
            })
            .map_err(|e| format!("spawn failed: {e}"))?;
        handle
            .join()
            .map_err(|_| "thread panicked during JSON traversal".to_string())
    })?;

    while let serde_json::Value::Array(mut arr) = deep_val {
        deep_val = arr.pop().unwrap_or(serde_json::Value::Null);
    }

    assert!(deep_extracted.contains("DEEP-ENTITY-42"));
    assert!(deep_extracted.contains("ENTITY-42"));
    Ok(())
}
