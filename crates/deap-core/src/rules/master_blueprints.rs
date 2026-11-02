//! Upstream master blueprint models and catalogs.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `MasterBlueprint`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Preserves structural type invariants for enum `MasterBlueprint` across all 3 members.
//!
//! ## 3. Preconditions
//! - Caller supplies well-formed values for `MasterBlueprint` members.
//!
//! ## 4. Postconditions
//! - Defines authoritative `MasterBlueprint` and `MasterBlueprintsCatalog` representations.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(1) constant-time enum dispatch.
//! - Space Complexity: O(1) auxiliary memory.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Exhaustive matching prevents unhandled variants.
//!
//! ## 7. Realization & Traceability
//! Realises: [deap_core::rules::MasterBlueprint] | SysML v2: PartDef(MasterBlueprint)

/// Typed enumeration of the upstream master blueprint filenames that downstream repositories must not duplicate.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `MasterBlueprint`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants across all 3 members.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `MasterBlueprint` members.
///
/// ## 4. Postconditions
/// - Defines authoritative `MasterBlueprint` representation.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(3)) bounded over member dimension of `MasterBlueprint`.
/// - Space Complexity: O(M(3)) bounded auxiliary memory for `MasterBlueprint`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `MasterBlueprint`.
/// - State: Maintains consistent state invariants across `MasterBlueprint`.
/// - Action: Executes `MasterBlueprint` deterministically.
/// - Resource: Bounds allocations within `MasterBlueprint`.
///
/// ## 7. Realization & Traceability
/// Realises: [deap_core::rules::MasterBlueprint] | SysML v2: PartDef(MasterBlueprint)
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
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Fixes filename used by duplicate detector.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Bijection between the 3 `MasterBlueprint` variants and 3 distinct upper-snake-case `.md` filenames.
    ///
    /// ## 3. Preconditions
    /// `self` is one of the 3 `MasterBlueprint` variants.
    ///
    /// ## 4. Postconditions
    /// Returns a `'static` bare filename ending in `.md`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1) three-arm `const` match. Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Exhaustiveness checking rejects unhandled variants at compile time.
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

    /// Borrowing adapter that yields the master blueprint filename from a variant reference.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Supplies `fn(&'static MasterBlueprint)` pointer shape.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// For every variant `b`, `b.as_str_ref() == b.as_str()`.
    ///
    /// ## 3. Preconditions
    /// `self` is a live shared reference to a `MasterBlueprint`.
    ///
    /// ## 4. Postconditions
    /// Returns the same `'static` filename as `as_str`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1). Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Total and allocation-free.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RULES-MASTER-BLUEPRINT-AS-STR-REF]
    pub fn as_str_ref(&self) -> &'static str {
        self.as_str()
    }
}

/// Zero-sized catalog providing iteration and membership queries over master blueprints.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `MasterBlueprintsCatalog`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `MasterBlueprintsCatalog`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `MasterBlueprintsCatalog` members.
///
/// ## 4. Postconditions
/// - Defines authoritative `MasterBlueprintsCatalog` representation.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(1)) bounded over input dimension.
/// - Space Complexity: O(M(1)) zero auxiliary memory.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `MasterBlueprintsCatalog`.
/// - State: Maintains consistent state invariants across `MasterBlueprintsCatalog`.
/// - Action: Executes `MasterBlueprintsCatalog` deterministically.
/// - Resource: Bounds allocations within `MasterBlueprintsCatalog`.
///
/// ## 7. Realization & Traceability
/// Realises: [deap_core::rules::MasterBlueprintsCatalog] | SysML v2: PartDef(MasterBlueprintsCatalog)
#[derive(Debug, Clone, Copy, Default)]
pub struct MasterBlueprintsCatalog;

/// Master blueprints that reside exclusively in upstream specification repository.
pub const MASTER_BLUEPRINTS: MasterBlueprintsCatalog = MasterBlueprintsCatalog;

impl MasterBlueprintsCatalog {
    /// Returns a typed-enum-backed iterator over all 3 canonical upstream master blueprint filenames.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Supports downstream SSOT verification.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Yields all 3 upstream master blueprint filenames in fixed order.
    ///
    /// ## 3. Preconditions
    /// `self` is a valid `MasterBlueprintsCatalog` reference.
    ///
    /// ## 4. Postconditions
    /// Returns an iterator yielding exactly 3 static strings.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1) construction, O(3) traversal. Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Total over a compile-time table; cannot yield fewer than 3 names.
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
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Detects duplicate master blueprints.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Evaluates membership via compile-time pattern match over 3 master blueprint filenames.
    ///
    /// ## 3. Preconditions
    /// `bp` is a valid UTF-8 filename slice.
    ///
    /// ## 4. Postconditions
    /// Returns `true` if `bp` matches a canonical master blueprint filename, `false` otherwise.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1). Space Complexity: O(1).
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
