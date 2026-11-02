//! Codebase rules configuration and manifest models.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `CodebaseRules`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Preserves structural type invariants for codebase configuration and governance rules.
//!
//! ## 3. Preconditions
//! - Caller supplies well-formed configuration structures and JSON strings.
//!
//! ## 4. Postconditions
//! - Returns initialized `CodebaseRules` representation.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(N log K) bounded by JSON payload size.
//! - Space Complexity: O(N) auxiliary memory for configuration trees.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Deserialization errors fail closed via `Result::Err(serde_json::Error)`.
//!
//! ## 7. Realization & Traceability
//! Realises: [deap_core::rules::CodebaseRules] | SysML v2: PartDef(CodebaseRules)

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// Top-level codebase rules representation matching `codebase_rules.json`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `CodebaseRules`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants across all member fields.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `CodebaseRules` members.
///
/// ## 4. Postconditions
/// - Defines authoritative `CodebaseRules` representation.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over member dimension of `CodebaseRules`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `CodebaseRules`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `CodebaseRules`.
/// - State: Maintains consistent state invariants across `CodebaseRules`.
/// - Action: Executes `CodebaseRules` deterministically.
/// - Resource: Bounds stack and heap allocations within `CodebaseRules`.
///
/// ## 7. Realization & Traceability
/// Realises: [deap_core::rules::CodebaseRules] | SysML v2: PartDef(CodebaseRules)
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `MetaConfig`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants across all member fields.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `MetaConfig` members.
///
/// ## 4. Postconditions
/// - Defines authoritative `MetaConfig` representation.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over member dimension of `MetaConfig`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `MetaConfig`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `MetaConfig`.
/// - State: Maintains consistent state invariants across `MetaConfig`.
/// - Action: Executes `MetaConfig` deterministically.
/// - Resource: Bounds stack and heap allocations within `MetaConfig`.
///
/// ## 7. Realization & Traceability
/// Realises: [deap_core::rules::MetaConfig] | SysML v2: PartDef(MetaConfig)
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `TrackerRules`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants across all member fields.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `TrackerRules` members.
///
/// ## 4. Postconditions
/// - Defines authoritative `TrackerRules` representation.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over member dimension of `TrackerRules`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `TrackerRules`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `TrackerRules`.
/// - State: Maintains consistent state invariants across `TrackerRules`.
/// - Action: Executes `TrackerRules` deterministically.
/// - Resource: Bounds stack and heap allocations within `TrackerRules`.
///
/// ## 7. Realization & Traceability
/// Realises: [deap_core::rules::TrackerRules] | SysML v2: PartDef(TrackerRules)
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `ValidationRules`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants across all member fields.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `ValidationRules` members.
///
/// ## 4. Postconditions
/// - Defines authoritative `ValidationRules` representation.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(3)) bounded over member dimension of `ValidationRules`.
/// - Space Complexity: O(M(3)) bounded auxiliary memory for `ValidationRules`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `ValidationRules`.
/// - State: Maintains consistent state invariants across `ValidationRules`.
/// - Action: Executes `ValidationRules` deterministically.
/// - Resource: Bounds stack and heap allocations within `ValidationRules`.
///
/// ## 7. Realization & Traceability
/// Realises: [deap_core::rules::ValidationRules] | SysML v2: PartDef(ValidationRules)
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
    /// Preserves all known configuration sections and captures extension keys in `extra`.
    ///
    /// ## 3. Preconditions
    /// `s` is a valid UTF-8 string slice containing JSON text.
    ///
    /// ## 4. Postconditions
    /// Returns `Ok(CodebaseRules)` on valid JSON, or `Err(serde_json::Error)` on syntax/type mismatch.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(N log K) where N is byte length of `s` and K is key count. Space Complexity: O(N).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Fails closed by returning `Err(serde_json::Error)` on malformed JSON without panicking.
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
    /// deterministic precedence order.
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
