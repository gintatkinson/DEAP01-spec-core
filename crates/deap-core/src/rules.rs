//! Manifest rules and codebase configuration parsing.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// Zero-sized catalog providing iteration and membership queries over excluded workspace directories.
/// Realises: [deap_core::rules::ExcludedDirsCatalog]
#[derive(Debug, Clone, Copy, Default)]
pub struct ExcludedDirsCatalog;

/// Directories excluded from standard workspace scans.
pub const EXCLUDED_DIRS: ExcludedDirsCatalog = ExcludedDirsCatalog;

impl ExcludedDirsCatalog {
    /// Returns a by-value iterator over all 15 canonical excluded workspace directory names.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C compiler purity standards by providing
    /// deterministic iteration over excluded directory names without static string-slice arrays.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Yields all 15 canonical excluded directory names (`".git"`, `".dart_tool"`, `"build"`,
    /// `".gradle"`, `"node_modules"`, `".next"`, `"target"`, `"Pods"`, `".idea"`, `".vscode"`,
    /// `"dist"`, `"out"`, `".pytest_cache"`, `"__pycache__"`, `".agents"`) in fixed order.
    ///
    /// ## 3. Preconditions
    /// `self` is a valid `ExcludedDirsCatalog` reference.
    ///
    /// ## 4. Postconditions
    /// Returns `std::array::IntoIter<&'static str, 15>` over the canonical excluded directory names.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1) to construct iterator. Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Pure total function; never panics.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RULES-EXCLUDED-DIRS-ITER]
    pub fn iter(&self) -> std::array::IntoIter<&'static str, 15> {
        [
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
        ]
        .into_iter()
    }

    /// Checks whether a candidate directory name matches any excluded workspace directory.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Guards workspace walkers from descending
    /// into build caches, VCS metadata, or third-party vendor directories.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Evaluates membership via an exhaustive compile-time pattern match over the 15 excluded
    /// directory names; consistent with `self.iter().any(|d| d == dir)`.
    ///
    /// ## 3. Preconditions
    /// `dir` is a valid UTF-8 string slice representing a directory name.
    ///
    /// ## 4. Postconditions
    /// Returns `true` if `dir` matches one of the 15 excluded directory names, `false` otherwise.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1) bounded over the fixed 15-element match arms. Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Pure pattern match; never panics.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RULES-EXCLUDED-DIRS-CONTAINS]
    pub fn contains(&self, dir: &str) -> bool {
        matches!(
            dir,
            ".git"
                | ".dart_tool"
                | "build"
                | ".gradle"
                | "node_modules"
                | ".next"
                | "target"
                | "Pods"
                | ".idea"
                | ".vscode"
                | "dist"
                | "out"
                | ".pytest_cache"
                | "__pycache__"
                | ".agents"
        )
    }
}

/// Zero-sized catalog providing iteration and membership queries over upstream master blueprints.
/// Realises: [deap_core::rules::MasterBlueprintsCatalog]
#[derive(Debug, Clone, Copy, Default)]
pub struct MasterBlueprintsCatalog;

/// Master blueprints that reside exclusively in upstream specification repository.
pub const MASTER_BLUEPRINTS: MasterBlueprintsCatalog = MasterBlueprintsCatalog;

impl MasterBlueprintsCatalog {
    /// Returns a by-value iterator over all 3 canonical upstream master blueprint filenames.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Supports downstream single-source-of-truth
    /// verification (Check 12) without static string-slice arrays.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Yields all 3 upstream master blueprint filenames (`"DEAP_MASTER_ARCHITECTURE.md"`,
    /// `"THREE_TIER_GOVERNANCE_BLUEPRINT.md"`, `"SYSML_SSOT_BIDIRECTIONAL_SYNCHRONIZATION_ARCHITECTURE.md"`)
    /// in fixed order.
    ///
    /// ## 3. Preconditions
    /// `self` is a valid `MasterBlueprintsCatalog` reference.
    ///
    /// ## 4. Postconditions
    /// Returns `std::array::IntoIter<&'static str, 3>` over the canonical master blueprint filenames.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1) to construct iterator. Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Pure total function; never panics.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RULES-MASTER-BLUEPRINTS-ITER]
    pub fn iter(&self) -> std::array::IntoIter<&'static str, 3> {
        [
            "DEAP_MASTER_ARCHITECTURE.md",
            "THREE_TIER_GOVERNANCE_BLUEPRINT.md",
            "SYSML_SSOT_BIDIRECTIONAL_SYNCHRONIZATION_ARCHITECTURE.md",
        ]
        .into_iter()
    }

    /// Checks whether a candidate filename matches any upstream master blueprint.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Detects duplicate master blueprints in
    /// downstream customer repositories (Check 12).
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Evaluates membership via an exhaustive compile-time pattern match over the 3 master
    /// blueprint filenames; consistent with `self.iter().any(|b| b == bp)`.
    ///
    /// ## 3. Preconditions
    /// `bp` is a valid UTF-8 filename slice.
    ///
    /// ## 4. Postconditions
    /// Returns `true` if `bp` matches a canonical master blueprint filename, `false` otherwise.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1) bounded over the fixed 3-element match arms. Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Pure pattern match; never panics.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RULES-MASTER-BLUEPRINTS-CONTAINS]
    pub fn contains(&self, bp: &str) -> bool {
        matches!(
            bp,
            "DEAP_MASTER_ARCHITECTURE.md"
                | "THREE_TIER_GOVERNANCE_BLUEPRINT.md"
                | "SYSML_SSOT_BIDIRECTIONAL_SYNCHRONIZATION_ARCHITECTURE.md"
        )
    }
}

/// Top-level codebase rules representation matching `codebase_rules.json`.
/// Realises: [deap_core::rules::CodebaseRules]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct CodebaseRules {
    /// Repository metadata and governance path configuration.
    #[serde(default)]
    pub meta: Option<MetaConfig>,
    /// Issue tracker provider and label normalization configuration.
    #[serde(default)]
    pub tracker_rules: Option<TrackerRules>,
    /// Validation and section completeness configuration.
    #[serde(default)]
    pub validation_rules: Option<ValidationRules>,
    /// Mapping of specification item classes to relative backlog directories.
    #[serde(default)]
    pub backlog_directories: Option<BTreeMap<String, String>>,
    /// Mapping of target platform keys to relative application directories.
    #[serde(default)]
    pub target_directories: Option<BTreeMap<String, Option<String>>>,
    /// Additional unmapped top-level JSON configuration properties.
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Metadata configuration section within `codebase_rules.json`.
/// Realises: [deap_core::rules::MetaConfig]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct MetaConfig {
    /// Semantic version string of the rules manifest.
    #[serde(default)]
    pub version: Option<String>,
    /// Human-readable description of the rules manifest.
    #[serde(default)]
    pub description: Option<String>,
    /// Canonical upstream specification core repository slug.
    #[serde(default)]
    pub upstream_repository: Option<String>,
    /// Relative path to the project governance constitution markdown file.
    #[serde(default)]
    pub constitution_path: Option<String>,
    /// Relative path to the directory containing platform profiles.
    #[serde(default)]
    pub profiles_directory: Option<String>,
    /// Relative path to the solution walkthrough documents directory.
    #[serde(default)]
    pub walkthrough_directory: Option<String>,
    /// Filename pattern for solution walkthrough documents.
    #[serde(default)]
    pub walkthrough_pattern: Option<String>,
    /// Relative path to the backlog reconciliation utility.
    #[serde(default)]
    pub reconciliation_script_path: Option<String>,
    /// Relative path to the behavioral triggers configuration file.
    #[serde(default)]
    pub behavioral_triggers_path: Option<String>,
    /// Additional unmapped metadata properties.
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Issue tracker integration and label rules within `codebase_rules.json`.
/// Realises: [deap_core::rules::TrackerRules]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct TrackerRules {
    /// Active tracker provider identifier (e.g., `"github"` or `"gitlab"`).
    #[serde(default)]
    pub provider: Option<String>,
    /// Placeholder token for unassigned tracker issue IDs.
    #[serde(default)]
    pub issue_id_placeholder: Option<String>,
    /// Pattern string for normalizing specification item prefixes.
    #[serde(default)]
    pub prefix_normalization_regex: Option<String>,
    /// Pattern string for extracting specification item title prefixes.
    #[serde(default)]
    pub title_extraction_prefixes_regex: Option<String>,
    /// Numeric prefix format string.
    #[serde(default)]
    pub numeric_prefix: Option<String>,
    /// Canonical tracker label key-to-label-name mapping.
    #[serde(default)]
    pub labels: Option<BTreeMap<String, String>>,
    /// Additional unmapped tracker rule properties.
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Validation gate configuration within `codebase_rules.json`.
/// Realises: [deap_core::rules::ValidationRules]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ValidationRules {
    /// Flag indicating domain-agnostic baseline verification mode.
    #[serde(default)]
    pub no_domain: Option<bool>,
    /// Required markdown section headings per specification artifact category.
    #[serde(default)]
    pub required_sections: Option<BTreeMap<String, serde_json::Value>>,
    /// Additional unmapped validation rule properties.
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl CodebaseRules {
    /// Deserializes a `CodebaseRules` configuration structure from a JSON string slice.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C configuration management requirements.
    /// Parses workspace governance rules into deterministic `BTreeMap`-backed structures.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Preserves all known configuration sections and captures extension keys in `extra` via
    /// deterministic `BTreeMap` ordering.
    ///
    /// ## 3. Preconditions
    /// `s` is a valid UTF-8 string slice containing JSON text.
    ///
    /// ## 4. Postconditions
    /// Returns `Ok(CodebaseRules)` on valid JSON, or `Err(serde_json::Error)` on syntax/type mismatch.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(N log K) where N is byte length of `s` and K is key count.
    /// Space Complexity: O(N).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Fails closed by returning `Err(serde_json::Error)` on malformed JSON input without panicking.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RULES-JSON]
    pub fn from_json_str(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    /// Discovers and loads `CodebaseRules` from standard manifest locations under `repo_root`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Resolves workspace rule manifests in
    /// deterministic precedence order (`.pipeline/logical-ui/codebase_rules.json`, `codebase_rules.json`,
    /// `baseline_manifest.json`).
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Evaluates candidate paths in strict priority order and returns the first valid configuration.
    ///
    /// ## 3. Preconditions
    /// `repo_root` is a valid filesystem path reference.
    ///
    /// ## 4. Postconditions
    /// Returns `Ok(Some(CodebaseRules))` if a valid manifest is found, `Ok(None)` if no manifest exists,
    /// or `Err(std::io::Error)` if an existing file cannot be read.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(N log K) where N is manifest file size. Space Complexity: O(N).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Propagates filesystem read errors via `Result::Err`; never panics.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RULES-LOAD]
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
            rules
                .validation_rules
                .as_ref()
                .and_then(|v| v.no_domain),
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
        assert!(rules_opt.is_some(), "Should find .pipeline/logical-ui/codebase_rules.json");
        let rules = rules_opt.ok_or("Expected Some(rules)")?;
        assert_eq!(
            rules.meta.as_ref().and_then(|m| m.upstream_repository.as_deref()),
            Some("gintatkinson/DEAP01-spec-core")
        );
        Ok(())
    }
}
