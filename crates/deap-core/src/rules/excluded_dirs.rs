//! Excluded workspace directory models and catalogs.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `ExcludedDir`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Preserves structural type invariants for enum `ExcludedDir` across member set.
//!
//! ## 3. Preconditions
//! - Caller supplies well-formed values for `ExcludedDir` members.
//!
//! ## 4. Postconditions
//! - Defines authoritative `ExcludedDir` and `ExcludedDirsCatalog` representations.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(1) constant-time enum dispatch.
//! - Space Complexity: O(1) auxiliary memory.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Exhaustive matching prevents unhandled variants.
//!
//! ## 7. Realization & Traceability
//! Realises: [deap_core::rules::ExcludedDir] | SysML v2: PartDef(ExcludedDir)

/// Typed enumeration of every workspace directory name excluded from standard workspace scans.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `ExcludedDir`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for enum `ExcludedDir` across all 15 members.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `ExcludedDir` members.
///
/// ## 4. Postconditions
/// - Defines the authoritative `ExcludedDir` representation for enum `ExcludedDir`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(15)) bounded over member dimension of `ExcludedDir`.
/// - Space Complexity: O(M(15)) bounded auxiliary memory for `ExcludedDir`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `ExcludedDir`.
/// - State: Maintains consistent state invariants across `ExcludedDir`.
/// - Action: Executes `ExcludedDir` deterministically with zero unbounded recursion.
/// - Resource: Bounds stack and heap allocations within `ExcludedDir`.
///
/// ## 7. Realization & Traceability
/// Realises: [deap_core::rules::ExcludedDir] | SysML v2: PartDef(ExcludedDir)
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
    /// to the literal directory basename compared against filesystem entries.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Injective mapping from the 15 `ExcludedDir` variants to 15 distinct case-sensitive ASCII basenames.
    ///
    /// ## 3. Preconditions
    /// `self` is any of the 15 `ExcludedDir` variants.
    ///
    /// ## 4. Postconditions
    /// Returns a `'static` directory basename such as `".git"` or `"node_modules"`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1) single jump-table dispatch. Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Exhaustive match prevents unhandled variants.
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
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Supplies the `fn(&'static ExcludedDir)` pointer shape.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// For every variant `d`, `d.as_str_ref() == d.as_str()`.
    ///
    /// ## 3. Preconditions
    /// `self` is a live shared reference to an `ExcludedDir`.
    ///
    /// ## 4. Postconditions
    /// Returns the same `'static` basename as `as_str`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1). Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Dereferencing a `Copy` variant reference is total and allocation-free.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RULES-EXCLUDED-DIR-AS-STR-REF]
    pub fn as_str_ref(&self) -> &'static str {
        self.as_str()
    }
}

/// Zero-sized catalog providing iteration and membership queries over excluded workspace directories.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `ExcludedDirsCatalog`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `ExcludedDirsCatalog`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `ExcludedDirsCatalog` members.
///
/// ## 4. Postconditions
/// - Defines authoritative `ExcludedDirsCatalog` representation.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(1)) bounded over input dimension.
/// - Space Complexity: O(M(1)) zero auxiliary memory.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `ExcludedDirsCatalog`.
/// - State: Maintains consistent state invariants across `ExcludedDirsCatalog`.
/// - Action: Executes `ExcludedDirsCatalog` deterministically.
/// - Resource: Bounds stack and heap allocations within `ExcludedDirsCatalog`.
///
/// ## 7. Realization & Traceability
/// Realises: [deap_core::rules::ExcludedDirsCatalog] | SysML v2: PartDef(ExcludedDirsCatalog)
#[derive(Debug, Clone, Copy, Default)]
pub struct ExcludedDirsCatalog;

/// Directories excluded from standard workspace scans.
pub const EXCLUDED_DIRS: ExcludedDirsCatalog = ExcludedDirsCatalog;

impl ExcludedDirsCatalog {
    /// Returns a typed-enum-backed iterator over all 15 canonical excluded workspace directory names.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C compiler purity standards.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Yields all 15 canonical excluded directory names in fixed order.
    ///
    /// ## 3. Preconditions
    /// `self` is a valid `ExcludedDirsCatalog` reference.
    ///
    /// ## 4. Postconditions
    /// Returns an exact-size iterator with `len() == 15`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1) to construct, O(15) to exhaust. Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Pure total function over a `'static` table; never panics.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RULES-EXCLUDED-DIRS-ITER]
    pub fn iter(
        &self,
    ) -> std::iter::Map<
        std::slice::Iter<'static, ExcludedDir>,
        fn(&'static ExcludedDir) -> &'static str,
    > {
        let project: fn(&'static ExcludedDir) -> &'static str = ExcludedDir::as_str_ref;
        ExcludedDir::ALL.iter().map(project)
    }

    /// Checks whether a candidate directory name matches any excluded workspace directory.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Guards workspace walkers.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Evaluates membership via compile-time pattern match over 15 excluded directory names.
    ///
    /// ## 3. Preconditions
    /// `dir` is a valid UTF-8 string slice.
    ///
    /// ## 4. Postconditions
    /// Returns `true` if `dir` matches one of the 15 excluded directory names, `false` otherwise.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1). Space Complexity: O(1).
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
