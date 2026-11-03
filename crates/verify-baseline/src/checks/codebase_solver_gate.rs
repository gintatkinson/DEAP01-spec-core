//! Baseline verification Check 41: Track B C-ABI Solver Bridge SSOT & 5-Table SQLite Seed Parity Gate.
//! Realises: [REQ-VERIFY-BASELINE-CODEBASE-CHECKS], [REQ-CHECK-41-SOLVER-BRIDGE-SQLITE-SEED-PARITY]

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Check 41: Track B C-ABI Solver Bridge SSOT & 5-Table SQLite Seed Parity Gate.
/// Realises: [REQ-CHECK-41-SOLVER-BRIDGE-SQLITE-SEED-PARITY]
pub fn check_41_solver_bridge_and_sqlite_seed_parity_gate(
    repo_root: &Path,
) -> Result<(), Vec<String>> {
    let app_flutter_dir = repo_root.join("app_flutter");
    if !app_flutter_dir.is_dir() {
        return Ok(());
    }

    let mut violations = Vec::new();

    check_solver_bridge_ssot_delegation(repo_root, &mut violations);
    check_bijective_sqlite_seed_parity(repo_root, &mut violations);

    if violations.is_empty() {
        Ok(())
    } else {
        violations.sort();
        Err(violations)
    }
}

/// Verifies that Dart solver bridge files are thin delegation adapters without duplicate solver math.
/// Realises: [REQ-CHECK-41-SSOT-DELEGATION]
pub fn check_no_duplicate_dart_solver_math(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();
    check_solver_bridge_ssot_delegation(repo_root, &mut violations);
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

/// Checks whether a downstream native Rust solver crate exists under `crates/`.
/// Realises: [REQ-CHECK-41-SSOT-DELEGATION]
fn has_downstream_solver_crate(repo_root: &Path) -> bool {
    let crates_dir = repo_root.join("crates");
    if !crates_dir.is_dir() {
        return false;
    }

    let upstream_crates = [
        "deap-core",
        "compile-sysml",
        "ingest-sysml",
        "verify-baseline",
        "assemble-conops",
        "reconcile-backlog",
    ];

    if let Ok(entries) = fs::read_dir(&crates_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                let name = entry.file_name().to_string_lossy().to_string();
                if !upstream_crates.contains(&name.as_str()) {
                    let src_solver = entry.path().join("src").join("solver.rs");
                    let src_lib = entry.path().join("src").join("lib.rs");
                    if src_solver.is_file() || src_lib.is_file() {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Enforces that Dart solver bridge sources delegate optimization math exclusively to the Rust solver.
/// Realises: [REQ-CHECK-41-SSOT-DELEGATION]
fn check_solver_bridge_ssot_delegation(repo_root: &Path, violations: &mut Vec<String>) {
    if !has_downstream_solver_crate(repo_root) {
        return;
    }

    let data_dir = repo_root.join("app_flutter").join("lib").join("data");
    if !data_dir.is_dir() {
        return;
    }

    let duplicate_math_tokens = [
        "_buildCandidates",
        "buildCandidates(",
        "alphaCovOffset",
        "reserveBoostPct",
        "extraDensityPenalty",
        "paretoScore",
        "violationCount: 0",
        "synthesize_candidates(",
        "synthesizeCandidates(",
        "compute_composite_score(",
        "computeCompositeScore(",
        "evaluate_constraints(",
        "evaluateConstraints(",
        "_hotlVetoActive = true",
    ];

    for entry_res in WalkDir::new(&data_dir) {
        let entry = match entry_res {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy();
        if !file_name.ends_with(".dart") || !file_name.contains("bridge") {
            continue;
        }
        if let Ok(content) = fs::read_to_string(path) {
            let rel = path
                .strip_prefix(repo_root)
                .unwrap_or(path)
                .display()
                .to_string();
            let line_count = content.lines().count();
            if line_count > 1200 {
                violations.push(format!(
                    "Check 41 violation: Dart solver bridge {} exceeds 1200-line thin-bridge ceiling ({} lines). Dart solver bridge must be a thin C-ABI delegation shim.",
                    rel, line_count
                ));
            }
            for (idx, line) in content.lines().enumerate() {
                let line_no = idx.saturating_add(1);
                let trimmed = line.trim();
                for token in &duplicate_math_tokens {
                    if trimmed.contains(token) {
                        violations.push(format!(
                            "Check 41 violation: Duplicate solver mathematics detected in {}:{}: '{}'. Dart solver bridge must be a thin C-ABI delegation shim.",
                            rel, line_no, token
                        ));
                    }
                }
            }
        }
    }
}

/// Extracts entity IDs, type names, and property keys from `app_flutter/assets/topology_data.json`.
/// Realises: [REQ-CHECK-41-BIJECTIVE-SEED-PARITY]
fn extract_topology_seed_sets(
    repo_root: &Path,
    violations: &mut Vec<String>,
) -> Option<(BTreeSet<String>, BTreeSet<String>, BTreeSet<String>)> {
    let topology_path = repo_root.join("app_flutter").join("assets").join("topology_data.json");
    if !topology_path.is_file() {
        return None;
    }
    let content = match fs::read_to_string(&topology_path) {
        Ok(c) => c,
        Err(err) => {
            violations.push(format!("Check 41 violation: Failed to read app_flutter/assets/topology_data.json: {}.", err));
            return None;
        }
    };
    let val = match serde_json::from_str::<serde_json::Value>(&content) {
        Ok(v) => v,
        Err(err) => {
            violations.push(format!("Check 41 violation: Malformed JSON in app_flutter/assets/topology_data.json: {}.", err));
            return None;
        }
    };
    let nodes_opt = val.get("nodes").and_then(|n| n.as_array()).or_else(|| val.as_array());
    let Some(nodes) = nodes_opt else {
        violations.push("Check 41 violation: app_flutter/assets/topology_data.json is missing a valid 'nodes' array.".to_string());
        return None;
    };

    let mut topology_entity_ids = BTreeSet::new();
    let mut topology_type_names = BTreeSet::new();
    let mut topology_property_keys = BTreeSet::new();

    for node in nodes {
        if let Some(id_str) = node.get("id").and_then(|i| i.as_str()) {
            let trimmed = id_str.trim();
            if !trimmed.is_empty() {
                topology_entity_ids.insert(trimmed.to_string());
            }
        }
        if let Some(type_str) = node
            .get("typeName")
            .or_else(|| node.get("type_name"))
            .or_else(|| node.get("type"))
            .and_then(|t| t.as_str())
        {
            let trimmed = type_str.trim();
            if !trimmed.is_empty() {
                topology_type_names.insert(trimmed.to_string());
            }
        }
        if let Some(props_obj) = node.get("properties").and_then(|p| p.as_object()) {
            for prop_key in props_obj.keys() {
                let trimmed = prop_key.trim();
                if !trimmed.is_empty() {
                    topology_property_keys.insert(trimmed.to_string());
                }
            }
        }
    }

    Some((topology_entity_ids, topology_type_names, topology_property_keys))
}

/// Reads all non-empty `.dart` seed files in `seeds_dir`.
/// Realises: [REQ-CHECK-41-BIJECTIVE-SEED-PARITY]
fn read_sqlite_seed_files(seeds_dir: &Path) -> (String, bool, Option<String>) {
    let mut seed_files_content = String::new();
    let mut has_generated_seed = false;
    let mut strategy_content: Option<String> = None;

    if let Ok(entries) = fs::read_dir(seeds_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                let path = entry.path();
                if path.extension().map(|ext| ext == "dart").unwrap_or(false) {
                    if let Ok(c) = fs::read_to_string(&path) {
                        if !c.trim().is_empty() {
                            let fname = entry.file_name().to_string_lossy().to_string();
                            if fname == "generated_domain_seed.g.dart" {
                                has_generated_seed = true;
                            } else if fname == "domain_seed_strategy.dart" {
                                strategy_content = Some(c.clone());
                            }
                            seed_files_content.push_str(&c);
                            seed_files_content.push('\n');
                        }
                    }
                }
            }
        }
    }

    (seed_files_content, has_generated_seed, strategy_content)
}

/// Enforces 100% bijective entity parity between topology datasets and SQLite persistence seeds.
/// Realises: [REQ-CHECK-41-BIJECTIVE-SEED-PARITY]
fn check_bijective_sqlite_seed_parity(repo_root: &Path, violations: &mut Vec<String>) {
    let Some((topology_entity_ids, topology_type_names, topology_property_keys)) =
        extract_topology_seed_sets(repo_root, violations)
    else {
        return;
    };
    let seeds_dir = repo_root.join("app_flutter").join("lib").join("data").join("seeds");
    if !seeds_dir.is_dir() {
        violations.push("Check 41 violation: SQLite seed directory app_flutter/lib/data/seeds is missing while topology_data.json is present.".to_string());
        return;
    }

    let (seed_files_content, has_generated_seed, strategy_content) = read_sqlite_seed_files(&seeds_dir);
    if seed_files_content.trim().is_empty() {
        violations.push("Check 41 violation: No non-empty .dart SQLite seed files found in app_flutter/lib/data/seeds.".to_string());
        return;
    }

    if let Some(strat) = &strategy_content {
        for placeholder in &["space_$", "field_1", "field_50"] {
            if strat.contains(placeholder) {
                violations.push(format!("Check 41 violation: Synthetic placeholder pattern '{}' detected in app_flutter/lib/data/seeds/domain_seed_strategy.dart.", placeholder));
            }
        }
        if has_generated_seed {
            let delegates_to_generated = strat.contains("seedGeneratedDomainDatabase") || strat.contains("seedDatabase");
            let contains_all_entities = !topology_entity_ids.is_empty() && topology_entity_ids.iter().all(|id| strat.contains(id));
            if !delegates_to_generated && !contains_all_entities {
                violations.push("Check 41 violation: domain_seed_strategy.dart neither delegates to seedGeneratedDomainDatabase nor seeds all topology entities directly.".to_string());
            }
        }
    }

    for entity_id in &topology_entity_ids {
        if !seed_files_content.contains(entity_id) {
            violations.push(format!("Check 41 violation: Bijective SQLite seed parity failure: entity '{}' from topology_data.json is missing from SQLite domain seed.", entity_id));
        }
    }
    for type_name in &topology_type_names {
        if !seed_files_content.contains(type_name) {
            violations.push(format!("Check 41 violation: Bijective SQLite seed parity failure: entity type '{}' from topology_data.json is missing from SQLite domain seed.", type_name));
        }
    }
    for prop_key in &topology_property_keys {
        if !seed_files_content.contains(prop_key) {
            violations.push(format!("Check 41 violation: Bijective SQLite seed parity failure: property key '{}' from topology_data.json is missing from SQLite domain seed.", prop_key));
        }
    }
    for tbl in &["type_definitions", "type_attributes", "type_relations", "properties", "instances"] {
        if !seed_files_content.contains(tbl) {
            violations.push(format!("Check 41 violation: SQLite domain seed does not populate canonical table '{}'.", tbl));
        }
    }
}
