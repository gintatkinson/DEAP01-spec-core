//! Unit tests covering codebase rules, catalogs, and blueprint verification.
#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
use super::super::*;
use std::path::Path;

#[test]
fn test_excluded_dirs_coverage() {
    let required = [
        ".git",
        ".dart_tool",
        "build",
        ".gradle",
        "node_modules",
        ".next",
        "target",
        "Pods",
        ".idea",
        ".vscode",
        "dist",
        "out",
        ".pytest_cache",
        "__pycache__",
        ".agents",
    ];
    for dir in &required {
        assert!(
            EXCLUDED_DIRS.contains(dir),
            "Missing expected excluded dir: {}",
            dir
        );
    }
}

#[test]
fn test_master_blueprints_coverage() {
    let expected = [
        "DEAP_MASTER_ARCHITECTURE.md",
        "THREE_TIER_GOVERNANCE_BLUEPRINT.md",
        "SYSML_SSOT_BIDIRECTIONAL_SYNCHRONIZATION_ARCHITECTURE.md",
    ];
    for bp in &expected {
        assert!(
            MASTER_BLUEPRINTS.contains(bp),
            "Missing expected master blueprint: {}",
            bp
        );
    }
}

#[test]
fn test_excluded_dir_enum_cardinality_order_and_catalog_parity() {
    let expected = vec![
        ".git",
        ".dart_tool",
        "build",
        ".gradle",
        "node_modules",
        ".next",
        "target",
        "Pods",
        ".idea",
        ".vscode",
        "dist",
        "out",
        ".pytest_cache",
        "__pycache__",
        ".agents",
    ];
    assert_eq!(ExcludedDir::ALL.len(), 15);
    let via_enum: Vec<&'static str> = ExcludedDir::ALL.iter().map(|d| d.as_str()).collect();
    assert_eq!(via_enum, expected);
    let via_catalog: Vec<&'static str> = EXCLUDED_DIRS.iter().collect();
    assert_eq!(via_catalog, expected);
    assert_eq!(EXCLUDED_DIRS.iter().len(), 15);
    for d in ExcludedDir::ALL {
        assert_eq!(d.as_str_ref(), d.as_str());
        assert!(EXCLUDED_DIRS.contains(d.as_str()));
    }
    assert!(!EXCLUDED_DIRS.contains("src"));
    assert!(!EXCLUDED_DIRS.contains(".GIT"));
    assert!(!EXCLUDED_DIRS.contains(""));
    assert!(!EXCLUDED_DIRS.iter().any(|d| d == "crates"));
}

#[test]
fn test_master_blueprint_enum_cardinality_order_and_catalog_parity() {
    let expected = vec![
        "DEAP_MASTER_ARCHITECTURE.md",
        "THREE_TIER_GOVERNANCE_BLUEPRINT.md",
        "SYSML_SSOT_BIDIRECTIONAL_SYNCHRONIZATION_ARCHITECTURE.md",
    ];
    assert_eq!(MasterBlueprint::ALL.len(), 3);
    let via_enum: Vec<&'static str> = MasterBlueprint::ALL.iter().map(|b| b.as_str()).collect();
    assert_eq!(via_enum, expected);
    let via_catalog: Vec<&'static str> = MASTER_BLUEPRINTS.iter().collect();
    assert_eq!(via_catalog, expected);
    for b in MasterBlueprint::ALL {
        assert_eq!(b.as_str_ref(), b.as_str());
        assert!(MASTER_BLUEPRINTS.contains(b.as_str()));
    }
    assert!(!MASTER_BLUEPRINTS.contains("README.md"));
    assert!(!MASTER_BLUEPRINTS.contains("deap_master_architecture.md"));
}

#[test]
fn test_rules_catalogs_free_of_static_str_iter_evasion() {
    let sources = [
        include_str!("excluded_dirs.rs"),
        include_str!("master_blueprints.rs"),
        include_str!("canonical_blueprints.rs"),
    ];
    let needle_into_iter = ["IntoIter<&'static", " str"].concat();
    let needle_static_slice = ["&'static [&'static", " str]"].concat();

    for (idx, source) in sources.iter().enumerate() {
        let hits: Vec<String> = source
            .lines()
            .enumerate()
            .filter(|(_, l)| l.contains(&needle_into_iter) || l.contains(&needle_static_slice))
            .map(|(i, l)| format!("file {} line {}: {}", idx, i + 1, l.trim()))
            .collect();
        assert!(
            hits.is_empty(),
            "rules submodule carries untyped string-array catalog evasions:\n{}",
            hits.join("\n")
        );
    }
}

#[test]
fn test_parse_codebase_rules_json() -> Result<(), Box<dyn std::error::Error>> {
    let json_data = r#"{
        "meta": {
            "version": "1.0.0",
            "upstream_repository": "gintatkinson/DEAP01-spec-core"
        },
        "tracker_rules": {
            "provider": "github",
            "labels": {
                "epic": "epic",
                "feature": "feature"
            }
        },
        "validation_rules": {
            "no_domain": true
        }
    }"#;
    let rules = CodebaseRules::from_json_str(json_data)?;
    assert_eq!(
        rules.meta.as_ref().and_then(|m| m.version.as_deref()),
        Some("1.0.0")
    );
    assert_eq!(
        rules
            .meta
            .as_ref()
            .and_then(|m| m.upstream_repository.as_deref()),
        Some("gintatkinson/DEAP01-spec-core")
    );
    assert_eq!(
        rules.validation_rules.as_ref().and_then(|v| v.no_domain),
        Some(true)
    );
    Ok(())
}

#[test]
fn test_load_from_actual_repo_dir() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root")?;
    let rules_opt = CodebaseRules::load_from_dir(repo_root)?;
    assert!(
        rules_opt.is_some(),
        "Should find .pipeline/logical-ui/codebase_rules.json"
    );
    let rules = rules_opt.ok_or("Expected Some(rules)")?;
    assert_eq!(
        rules
            .meta
            .as_ref()
            .and_then(|m| m.upstream_repository.as_deref()),
        Some("gintatkinson/DEAP01-spec-core")
    );
    Ok(())
}

#[test]
fn test_canonical_architecture_blueprint_enum_cardinality_and_catalog() {
    let expected = vec![
        "DEAP_SYSML_V2_INGESTION_ENGINE_BLUEPRINT.md",
        "SYSML_SSOT_BIDIRECTIONAL_SYNCHRONIZATION_ARCHITECTURE.md",
        "DEAP_DETERMINISTIC_SAFETY_SPECIFICATION_COMPILER_BLUEPRINT.md",
        "DEAP_MULTI_TOOLCHAIN_SYNTHESIS_ARCHITECTURE.md",
        "DEAP_LOGICAL_INTERFACE_SPECIFICATION_BLUEPRINT.md",
        "MULTI_PROVIDER_GITLAB_INFRASTRUCTURE_ARCHITECTURE.md",
        "PERSISTENCE_ARCHITECTURE.md",
        "SAFETY_CRITICAL_REALTIME_UI_FRAMEWORK.md",
        "RUNTIME_METADATA_ENGINE.md",
        "SPECKIT_NATIVE_INTEGRATION.md",
        "DEAP_LOCAL_AIRGAPPED_DEEPSEEK_WORKSTATION_BLUEPRINT.md",
        "DEAP_DEEPSEEK_HARNESS_INTEGRATION_BLUEPRINT.md",
        "DEAP_POLY_REPO_BINARY_DISTRIBUTION_ARCHITECTURE.md",
    ];
    assert_eq!(CanonicalArchitectureBlueprint::ALL.len(), 13);
    let via_enum: Vec<&'static str> = CanonicalArchitectureBlueprint::ALL
        .iter()
        .map(|b| b.as_str())
        .collect();
    assert_eq!(via_enum, expected);
    let via_catalog: Vec<&'static str> = CANONICAL_ARCHITECTURE_BLUEPRINTS.iter().collect();
    assert_eq!(via_catalog, expected);
    for b in CanonicalArchitectureBlueprint::ALL {
        assert_eq!(b.as_str_ref(), b.as_str());
        assert!(CANONICAL_ARCHITECTURE_BLUEPRINTS.contains(b.as_str()));
    }
    assert!(!CANONICAL_ARCHITECTURE_BLUEPRINTS.contains("NON_EXISTENT_BLUEPRINT.md"));
    assert!(!CANONICAL_ARCHITECTURE_BLUEPRINTS
        .contains("deap_sysml_v2_ingestion_engine_blueprint.md"));
}

fn verify_canonical_architecture_blueprints_on_disk(repo_root: &Path) -> Result<(), String> {
    if !crate::workspace::is_upstream_compiler(repo_root) {
        return Ok(());
    }
    let blueprints_dir = repo_root
        .join("docs")
        .join("architecture")
        .join("blueprints");
    for blueprint_name in CANONICAL_ARCHITECTURE_BLUEPRINTS.iter() {
        let path = blueprints_dir.join(blueprint_name);
        if !path.is_file() {
            return Err(format!(
                "Expected canonical architecture blueprint file to exist on disk: {}",
                path.display()
            ));
        }
    }
    Ok(())
}

#[test]
fn test_canonical_architecture_blueprints_exist_on_disk() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root")?;
    verify_canonical_architecture_blueprints_on_disk(repo_root)?;
    Ok(())
}

struct ScopedTempDir(std::path::PathBuf);

impl ScopedTempDir {
    fn new(suffix: &str) -> std::io::Result<Self> {
        let path = std::env::temp_dir().join(format!(
            "deap_core_rules_{}_{}",
            std::process::id(),
            suffix
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path)?;
        Ok(Self(path))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for ScopedTempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn test_canonical_architecture_blueprints_skipped_in_downstream_workspace() -> Result<(), Box<dyn std::error::Error>> {
    let downstream_dir = ScopedTempDir::new("downstream_473")?;
    let res = verify_canonical_architecture_blueprints_on_disk(downstream_dir.path());
    assert!(
        res.is_ok(),
        "Downstream workspace without .pipeline/upstream must skip canonical blueprint check, got: {:?}",
        res
    );

    let upstream_dir = ScopedTempDir::new("upstream_473")?;
    std::fs::create_dir_all(upstream_dir.path().join(".pipeline").join("upstream"))?;
    let upstream_res = verify_canonical_architecture_blueprints_on_disk(upstream_dir.path());
    assert!(
        upstream_res.is_err(),
        "Upstream workspace missing canonical blueprints on disk must fail"
    );
    Ok(())
}
}
