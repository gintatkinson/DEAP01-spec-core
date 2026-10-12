//! Manifest rules and codebase configuration parsing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Directories excluded from standard workspace scans.
pub const EXCLUDED_DIRS: &[&str] = &[
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

/// Master blueprints that reside exclusively in upstream specification repository.
pub const MASTER_BLUEPRINTS: &[&str] = &[
    "DEAP_MASTER_ARCHITECTURE.md",
    "THREE_TIER_GOVERNANCE_BLUEPRINT.md",
    "SYSML_SSOT_BIDIRECTIONAL_SYNCHRONIZATION_ARCHITECTURE.md",
];

/// Top-level codebase rules representation matching codebase_rules.json.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct CodebaseRules {
    #[serde(default)]
    pub meta: Option<MetaConfig>,
    #[serde(default)]
    pub tracker_rules: Option<TrackerRules>,
    #[serde(default)]
    pub validation_rules: Option<ValidationRules>,
    #[serde(default)]
    pub backlog_directories: Option<HashMap<String, String>>,
    #[serde(default)]
    pub target_directories: Option<HashMap<String, Option<String>>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct MetaConfig {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub upstream_repository: Option<String>,
    #[serde(default)]
    pub constitution_path: Option<String>,
    #[serde(default)]
    pub profiles_directory: Option<String>,
    #[serde(default)]
    pub walkthrough_directory: Option<String>,
    #[serde(default)]
    pub walkthrough_pattern: Option<String>,
    #[serde(default)]
    pub reconciliation_script_path: Option<String>,
    #[serde(default)]
    pub behavioral_triggers_path: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct TrackerRules {
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub issue_id_placeholder: Option<String>,
    #[serde(default)]
    pub prefix_normalization_regex: Option<String>,
    #[serde(default)]
    pub title_extraction_prefixes_regex: Option<String>,
    #[serde(default)]
    pub numeric_prefix: Option<String>,
    #[serde(default)]
    pub labels: Option<HashMap<String, String>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ValidationRules {
    #[serde(default)]
    pub no_domain: Option<bool>,
    #[serde(default)]
    pub required_sections: Option<HashMap<String, serde_json::Value>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl CodebaseRules {
    pub fn from_json_str(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    pub fn load_from_dir(repo_root: &Path) -> Result<Option<Self>, std::io::Error> {
        let candidates = [
            repo_root.join(".pipeline/logical-ui/codebase_rules.json"),
            repo_root.join("codebase_rules.json"),
            repo_root.join("baseline_manifest.json"),
        ];

        for path in &candidates {
            if path.is_file() {
                let content = std::fs::read_to_string(path)?;
                if let Ok(rules) = Self::from_json_str(&content) {
                    return Ok(Some(rules));
                }
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_parse_codebase_rules_json() {
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

        let rules = CodebaseRules::from_json_str(json_data).expect("Failed to parse rules");
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
            rules
                .validation_rules
                .as_ref()
                .and_then(|v| v.no_domain),
            Some(true)
        );
    }

    #[test]
    fn test_load_from_actual_repo_dir() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("repo root");
        let rules_opt = CodebaseRules::load_from_dir(repo_root).expect("io error");
        assert!(rules_opt.is_some(), "Should find .pipeline/logical-ui/codebase_rules.json");
        let rules = rules_opt.unwrap();
        assert_eq!(
            rules.meta.as_ref().and_then(|m| m.upstream_repository.as_deref()),
            Some("gintatkinson/DEAP01-spec-core")
        );
    }
}
