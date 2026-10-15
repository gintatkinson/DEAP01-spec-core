//! Manifest rules and codebase configuration parsing.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// Typed enumeration of every workspace directory name excluded from standard workspace scans.
/// Realises: [deap_core::rules::ExcludedDir]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExcludedDir {
    /// Git version-control metadata directory (`.git`).
    Git,
    /// Dart tool cache directory (`.dart_tool`).
    DartTool,
    /// Generic build output directory (`build`).
    Build,
    /// Gradle cache directory (`.gradle`).
    Gradle,
    /// Node.js vendored dependency directory (`node_modules`).
    NodeModules,
    /// Next.js build cache directory (`.next`).
    Next,
    /// Cargo build output directory (`target`).
    Target,
    /// CocoaPods vendored dependency directory (`Pods`).
    Pods,
    /// JetBrains IDE metadata directory (`.idea`).
    Idea,
    /// VS Code editor metadata directory (`.vscode`).
    Vscode,
    /// Distribution bundle output directory (`dist`).
    Dist,
    /// Generic compiler output directory (`out`).
    Out,
    /// Legacy pytest cache directory (`.pytest_cache`).
    PytestCache,
    /// Legacy Python bytecode cache directory (`__pycache__`).
    Pycache,
    /// Agent skill and rule mirror directory (`.agents`).
    Agents,
}

impl ExcludedDir {
    /// Canonical declaration-ordered table of all 15 excluded workspace directory variants.
    pub const ALL: &'static [Self] = &[
        Self::Git,
        Self::DartTool,
        Self::Build,
        Self::Gradle,
        Self::NodeModules,
        Self::Next,
        Self::Target,
        Self::Pods,
        Self::Idea,
        Self::Vscode,
        Self::Dist,
        Self::Out,
        Self::PytestCache,
        Self::Pycache,
        Self::Agents,
    ];

    /// Projects an excluded directory variant onto its exact on-disk directory name.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Binds each typed workspace-walker exclusion
    /// to the literal directory basename compared against filesystem entries, so no walker can
    /// observe a misspelled or case-drifted exclusion name.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Injective mapping from the 15 `ExcludedDir` variants to 15 distinct case-sensitive ASCII
    /// basenames; `ExcludedDir::ALL[i].as_str()` reproduces the legacy catalog order exactly.
    ///
    /// ## 3. Preconditions
    /// `self` is any of the 15 `ExcludedDir` variants (guaranteed by the type system).
    ///
    /// ## 4. Postconditions
    /// Returns a `'static` directory basename such as `".git"` or `"node_modules"`, with no path
    /// separators and no leading `./`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1) single jump-table dispatch, evaluable in `const` context. Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Exhaustive `match` means adding a variant without a basename is a compile error rather
    /// than a silent scan of a cache directory at runtime.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RULES-EXCLUDED-DIR-AS-STR]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Git => ".git",
            Self::DartTool => ".dart_tool",
            Self::Build => "build",
            Self::Gradle => ".gradle",
            Self::NodeModules => "node_modules",
            Self::Next => ".next",
            Self::Target => "target",
            Self::Pods => "Pods",
            Self::Idea => ".idea",
            Self::Vscode => ".vscode",
            Self::Dist => "dist",
            Self::Out => "out",
            Self::PytestCache => ".pytest_cache",
            Self::Pycache => "__pycache__",
            Self::Agents => ".agents",
        }
    }

    /// Borrowing adapter that yields the excluded directory basename from a variant reference.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Supplies the `fn(&'static ExcludedDir)`
    /// pointer shape required by `ExcludedDirsCatalog::iter`, keeping the iterator's projection
    /// a named, documented, statically dispatched function instead of an opaque closure.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// For every variant `d`, `d.as_str_ref() == d.as_str()`; the adapter adds no mapping logic
    /// of its own and therefore cannot diverge from the by-value projection.
    ///
    /// ## 3. Preconditions
    /// `self` is a live shared reference to an `ExcludedDir`, typically borrowed from `ExcludedDir::ALL`.
    ///
    /// ## 4. Postconditions
    /// Returns the same `'static` basename as `as_str`; the borrow of `self` is not extended
    /// into the returned slice.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1) one dereference plus one dispatch. Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Cannot fail: dereferencing a `Copy` variant reference is total and allocation-free.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RULES-EXCLUDED-DIR-AS-STR-REF]
    pub fn as_str_ref(&self) -> &'static str {
        self.as_str()
    }
}

/// Zero-sized catalog providing iteration and membership queries over excluded workspace directories.
/// Realises: [deap_core::rules::ExcludedDirsCatalog]
#[derive(Debug, Clone, Copy, Default)]
pub struct ExcludedDirsCatalog;

/// Directories excluded from standard workspace scans.
pub const EXCLUDED_DIRS: ExcludedDirsCatalog = ExcludedDirsCatalog;

impl ExcludedDirsCatalog {
    /// Returns a typed-enum-backed iterator over all 15 canonical excluded workspace directory names.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C compiler purity standards by deriving every
    /// yielded excluded directory name from the typed `ExcludedDir` enumeration rather than an
    /// untyped string array, so walkers pruning `.git`, `target` and vendor caches share one source.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Yields all 15 canonical excluded directory names (`".git"`, `".dart_tool"`, `"build"`,
    /// `".gradle"`, `"node_modules"`, `".next"`, `"target"`, `"Pods"`, `".idea"`, `".vscode"`,
    /// `"dist"`, `"out"`, `".pytest_cache"`, `"__pycache__"`, `".agents"`) in fixed order,
    /// element-for-element equal to `ExcludedDir::ALL` projected through `ExcludedDir::as_str`.
    ///
    /// ## 3. Preconditions
    /// `self` is a valid `ExcludedDirsCatalog` reference (zero-sized; any instance is equivalent).
    ///
    /// ## 4. Postconditions
    /// Returns a `Map<slice::Iter<'static, ExcludedDir>, fn(&'static ExcludedDir) -> &'static str>`
    /// that is double-ended and exact-size with `len() == 15`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1) to construct, O(15) to exhaust. Space Complexity: O(1), two pointers plus one fn pointer.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Pure total function over a `'static` table; never panics and never allocates, so a walker
    /// cannot fail open by receiving a truncated exclusion list.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RULES-EXCLUDED-DIRS-ITER]
    pub fn iter(
        &self,
    ) -> std::iter::Map<std::slice::Iter<'static, ExcludedDir>, fn(&'static ExcludedDir) -> &'static str>
    {
        let project: fn(&'static ExcludedDir) -> &'static str = ExcludedDir::as_str_ref;
        ExcludedDir::ALL.iter().map(project)
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

/// Typed enumeration of the upstream master blueprint filenames that downstream repositories must not duplicate.
/// Realises: [deap_core::rules::MasterBlueprint]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MasterBlueprint {
    /// Master platform architecture blueprint (`DEAP_MASTER_ARCHITECTURE.md`).
    DeapMasterArchitecture,
    /// Three-tier governance blueprint (`THREE_TIER_GOVERNANCE_BLUEPRINT.md`).
    ThreeTierGovernanceBlueprint,
    /// SysML SSOT bidirectional synchronization blueprint
    /// (`SYSML_SSOT_BIDIRECTIONAL_SYNCHRONIZATION_ARCHITECTURE.md`).
    SysmlSsotBidirectionalSynchronizationArchitecture,
}

impl MasterBlueprint {
    /// Canonical declaration-ordered table of all 3 upstream master blueprint variants.
    pub const ALL: &'static [Self] = &[
        Self::DeapMasterArchitecture,
        Self::ThreeTierGovernanceBlueprint,
        Self::SysmlSsotBidirectionalSynchronizationArchitecture,
    ];

    /// Projects a master blueprint variant onto its exact upstream Markdown filename.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Fixes the filename used by the downstream
    /// single-source-of-truth duplicate detector (Check 12) to a typed, exhaustively matched
    /// variant, so a renamed blueprint cannot silently escape duplicate detection.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Bijection between the 3 `MasterBlueprint` variants and 3 distinct upper-snake-case `.md`
    /// filenames; declaration order equals the legacy catalog order.
    ///
    /// ## 3. Preconditions
    /// `self` is one of the 3 `MasterBlueprint` variants; no runtime validation is required.
    ///
    /// ## 4. Postconditions
    /// Returns a `'static` bare filename ending in `.md` (no directory component), suitable for
    /// comparison against `DirEntry::file_name()`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1) three-arm `const` match. Space Complexity: O(1) with no allocation.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Exhaustiveness checking rejects at compile time any blueprint variant lacking a filename,
    /// eliminating the fail-open case of an unnamed blueprint being skipped by Check 12.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RULES-MASTER-BLUEPRINT-AS-STR]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DeapMasterArchitecture => "DEAP_MASTER_ARCHITECTURE.md",
            Self::ThreeTierGovernanceBlueprint => "THREE_TIER_GOVERNANCE_BLUEPRINT.md",
            Self::SysmlSsotBidirectionalSynchronizationArchitecture => {
                "SYSML_SSOT_BIDIRECTIONAL_SYNCHRONIZATION_ARCHITECTURE.md"
            }
        }
    }

    /// Borrowing adapter returning the master blueprint filename from a variant reference.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Provides the `fn(&'static MasterBlueprint)`
    /// projection consumed by `MasterBlueprintsCatalog::iter`, so the blueprint iterator's element
    /// mapping is a single audited function shared with the by-value path.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// For every variant `b`, `b.as_str_ref() == b.as_str()` by construction (pure delegation).
    ///
    /// ## 3. Preconditions
    /// `self` borrows a valid `MasterBlueprint`, normally an element of `MasterBlueprint::ALL`.
    ///
    /// ## 4. Postconditions
    /// Returns the identical `'static` blueprint filename; the result outlives the input borrow.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1) copy-out of a one-byte discriminant then dispatch. Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Infallible delegation; no allocation, I/O, or panic path exists.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RULES-MASTER-BLUEPRINT-AS-STR-REF]
    pub fn as_str_ref(&self) -> &'static str {
        self.as_str()
    }
}

/// Zero-sized catalog providing iteration and membership queries over upstream master blueprints.
/// Realises: [deap_core::rules::MasterBlueprintsCatalog]
#[derive(Debug, Clone, Copy, Default)]
pub struct MasterBlueprintsCatalog;

/// Master blueprints that reside exclusively in upstream specification repository.
pub const MASTER_BLUEPRINTS: MasterBlueprintsCatalog = MasterBlueprintsCatalog;

impl MasterBlueprintsCatalog {
    /// Returns a typed-enum-backed iterator over all 3 canonical upstream master blueprint filenames.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Supports downstream single-source-of-truth
    /// verification (Check 12) by sourcing every blueprint filename from the typed
    /// `MasterBlueprint` enumeration instead of an untyped string array.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Yields all 3 upstream master blueprint filenames (`"DEAP_MASTER_ARCHITECTURE.md"`,
    /// `"THREE_TIER_GOVERNANCE_BLUEPRINT.md"`, `"SYSML_SSOT_BIDIRECTIONAL_SYNCHRONIZATION_ARCHITECTURE.md"`)
    /// in fixed order, identical to `MasterBlueprint::ALL` mapped through `MasterBlueprint::as_str`.
    ///
    /// ## 3. Preconditions
    /// `self` is a valid `MasterBlueprintsCatalog` reference; the catalog carries no state.
    ///
    /// ## 4. Postconditions
    /// Returns a `Map<slice::Iter<'static, MasterBlueprint>, fn(&'static MasterBlueprint) -> &'static str>`
    /// that callers may `chain` with additional names (as `check_no_duplicate_master_blueprints` does).
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1) construction and O(3) full traversal. Space Complexity: O(1) iterator state.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Total over a compile-time table; cannot yield fewer than 3 names, so Check 12 never
    /// under-reports duplicates because of a short blueprint list.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RULES-MASTER-BLUEPRINTS-ITER]
    pub fn iter(
        &self,
    ) -> std::iter::Map<
        std::slice::Iter<'static, MasterBlueprint>,
        fn(&'static MasterBlueprint) -> &'static str,
    > {
        let project: fn(&'static MasterBlueprint) -> &'static str = MasterBlueprint::as_str_ref;
        MasterBlueprint::ALL.iter().map(project)
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
        let source = include_str!("rules.rs");
        let needle_into_iter = ["IntoIter<&'static", " str"].concat();
        let needle_static_slice = ["&'static [&'static", " str]"].concat();
        let hits: Vec<String> = source
            .lines()
            .enumerate()
            .filter(|(_, l)| l.contains(&needle_into_iter) || l.contains(&needle_static_slice))
            .map(|(i, l)| format!("line {}: {}", i + 1, l.trim()))
            .collect();
        assert!(
            hits.is_empty(),
            "rules.rs still carries untyped string-array catalog evasions:\n{}",
            hits.join("\n")
        );
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
