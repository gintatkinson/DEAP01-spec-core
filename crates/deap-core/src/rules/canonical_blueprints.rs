//! Canonical architecture blueprint models and catalogs.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `CanonicalArchitectureBlueprint`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Preserves structural type invariants for enum `CanonicalArchitectureBlueprint` across all 13 members.
//!
//! ## 3. Preconditions
//! - Caller supplies well-formed values for `CanonicalArchitectureBlueprint` members.
//!
//! ## 4. Postconditions
//! - Defines authoritative `CanonicalArchitectureBlueprint` and `CanonicalArchitectureBlueprintsCatalog` representations.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(1) constant-time enum dispatch.
//! - Space Complexity: O(1) auxiliary memory.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Exhaustive matching prevents unhandled variants.
//!
//! ## 7. Realization & Traceability
//! Realises: [deap_core::rules::CanonicalArchitectureBlueprint] | SysML v2: PartDef(CanonicalArchitectureBlueprint)

/// Typed enumeration of the 13 canonical architecture blueprint filenames under `docs/architecture/blueprints/`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `CanonicalArchitectureBlueprint`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Bijective 13-variant mapping between each variant and its canonical markdown filename.
///
/// ## 3. Preconditions
/// - Caller selects or iterates over one of the 13 compile-time variants.
///
/// ## 4. Postconditions
/// - Projects each variant to its exact `.md` blueprint filename.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) per projection and O(13) across `ALL`.
/// - Space Complexity: O(1) static compile-time storage.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Exhaustive match arms enforce compile-time coverage.
/// - State: Stateless value enum.
/// - Action: Deterministic projection with zero panics.
/// - Resource: Zero heap allocation.
///
/// ## 7. Realization & Traceability
/// Realises: [deap_core::rules::CanonicalArchitectureBlueprint] | SysML v2: PartDef(CanonicalArchitectureBlueprint)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CanonicalArchitectureBlueprint {
    /// `"DEAP_SYSML_V2_INGESTION_ENGINE_BLUEPRINT.md"` -- SysML v2 ingestion engine blueprint.
    DeapSysmlV2IngestionEngineBlueprint,
    /// `"SYSML_SSOT_BIDIRECTIONAL_SYNCHRONIZATION_ARCHITECTURE.md"` -- SysML v2 bidirectional SSOT synchronization blueprint.
    SysmlSsotBidirectionalSynchronizationArchitecture,
    /// `"DEAP_DETERMINISTIC_SAFETY_SPECIFICATION_COMPILER_BLUEPRINT.md"` -- deterministic safety specification compiler blueprint.
    DeapDeterministicSafetySpecificationCompilerBlueprint,
    /// `"DEAP_MULTI_TOOLCHAIN_SYNTHESIS_ARCHITECTURE.md"` -- multi-toolchain synthesis architecture blueprint.
    DeapMultiToolchainSynthesisArchitecture,
    /// `"DEAP_LOGICAL_INTERFACE_SPECIFICATION_BLUEPRINT.md"` -- logical interface specification (ICD) blueprint.
    DeapLogicalInterfaceSpecificationBlueprint,
    /// `"MULTI_PROVIDER_GITLAB_INFRASTRUCTURE_ARCHITECTURE.md"` -- multi-provider GitHub/GitLab tracker infrastructure blueprint.
    MultiProviderGitlabInfrastructureArchitecture,
    /// `"PERSISTENCE_ARCHITECTURE.md"` -- zero-mocking persistence and AST storage blueprint.
    PersistenceArchitecture,
    /// `"SAFETY_CRITICAL_REALTIME_UI_FRAMEWORK.md"` -- safety-critical real-time UI framework blueprint.
    SafetyCriticalRealtimeUiFramework,
    /// `"RUNTIME_METADATA_ENGINE.md"` -- runtime metadata reflection and type descriptor engine blueprint.
    RuntimeMetadataEngine,
    /// `"SPECKIT_NATIVE_INTEGRATION.md"` -- SpecKit native integration blueprint.
    SpeckitNativeIntegration,
    /// `"DEAP_LOCAL_AIRGAPPED_DEEPSEEK_WORKSTATION_BLUEPRINT.md"` -- local air-gapped DeepSeek workstation blueprint.
    DeapLocalAirgappedDeepseekWorkstationBlueprint,
    /// `"DEAP_DEEPSEEK_HARNESS_INTEGRATION_BLUEPRINT.md"` -- DeepSeek harness and CoT audit integration blueprint.
    DeapDeepseekHarnessIntegrationBlueprint,
    /// `"DEAP_POLY_REPO_BINARY_DISTRIBUTION_ARCHITECTURE.md"` -- poly-repo binary-only distribution and tier isolation architecture blueprint.
    DeapPolyRepoBinaryDistributionArchitecture,
}

impl CanonicalArchitectureBlueprint {
    /// Complete ordered slice of all 13 `CanonicalArchitectureBlueprint` variants.
    pub const ALL: &'static [Self] = &[
        Self::DeapSysmlV2IngestionEngineBlueprint,
        Self::SysmlSsotBidirectionalSynchronizationArchitecture,
        Self::DeapDeterministicSafetySpecificationCompilerBlueprint,
        Self::DeapMultiToolchainSynthesisArchitecture,
        Self::DeapLogicalInterfaceSpecificationBlueprint,
        Self::MultiProviderGitlabInfrastructureArchitecture,
        Self::PersistenceArchitecture,
        Self::SafetyCriticalRealtimeUiFramework,
        Self::RuntimeMetadataEngine,
        Self::SpeckitNativeIntegration,
        Self::DeapLocalAirgappedDeepseekWorkstationBlueprint,
        Self::DeapDeepseekHarnessIntegrationBlueprint,
        Self::DeapPolyRepoBinaryDistributionArchitecture,
    ];

    /// Projects this `CanonicalArchitectureBlueprint` variant to its canonical markdown filename under `docs/architecture/blueprints/`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and Check 34 compile-time catalog rules.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Injective across all 13 variants; every returned `&'static str` ends with `".md"`.
    ///
    /// ## 3. Preconditions
    /// `self` is a valid `CanonicalArchitectureBlueprint` enum value.
    ///
    /// ## 4. Postconditions
    /// Returns the exact static filename string corresponding to `self`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1). Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Total function over a closed 13-variant enum.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RULES-CANONICAL-BLUEPRINT-AS-STR]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DeapSysmlV2IngestionEngineBlueprint => {
                "DEAP_SYSML_V2_INGESTION_ENGINE_BLUEPRINT.md"
            }
            Self::SysmlSsotBidirectionalSynchronizationArchitecture => {
                "SYSML_SSOT_BIDIRECTIONAL_SYNCHRONIZATION_ARCHITECTURE.md"
            }
            Self::DeapDeterministicSafetySpecificationCompilerBlueprint => {
                "DEAP_DETERMINISTIC_SAFETY_SPECIFICATION_COMPILER_BLUEPRINT.md"
            }
            Self::DeapMultiToolchainSynthesisArchitecture => {
                "DEAP_MULTI_TOOLCHAIN_SYNTHESIS_ARCHITECTURE.md"
            }
            Self::DeapLogicalInterfaceSpecificationBlueprint => {
                "DEAP_LOGICAL_INTERFACE_SPECIFICATION_BLUEPRINT.md"
            }
            Self::MultiProviderGitlabInfrastructureArchitecture => {
                "MULTI_PROVIDER_GITLAB_INFRASTRUCTURE_ARCHITECTURE.md"
            }
            Self::PersistenceArchitecture => "PERSISTENCE_ARCHITECTURE.md",
            Self::SafetyCriticalRealtimeUiFramework => "SAFETY_CRITICAL_REALTIME_UI_FRAMEWORK.md",
            Self::RuntimeMetadataEngine => "RUNTIME_METADATA_ENGINE.md",
            Self::SpeckitNativeIntegration => "SPECKIT_NATIVE_INTEGRATION.md",
            Self::DeapLocalAirgappedDeepseekWorkstationBlueprint => {
                "DEAP_LOCAL_AIRGAPPED_DEEPSEEK_WORKSTATION_BLUEPRINT.md"
            }
            Self::DeapDeepseekHarnessIntegrationBlueprint => {
                "DEAP_DEEPSEEK_HARNESS_INTEGRATION_BLUEPRINT.md"
            }
            Self::DeapPolyRepoBinaryDistributionArchitecture => {
                "DEAP_POLY_REPO_BINARY_DISTRIBUTION_ARCHITECTURE.md"
            }
        }
    }

    /// Reference-accepting adapter around [`CanonicalArchitectureBlueprint::as_str`] for iterator mapping.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and Check 34 `(b)`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// For every `v`, `v.as_str_ref() == v.as_str()`.
    ///
    /// ## 3. Preconditions
    /// `self` is a valid shared reference.
    ///
    /// ## 4. Postconditions
    /// Returns the identical `&'static str` filename produced by `(*self).as_str()`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1). Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Infallible reference projection.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RULES-CANONICAL-BLUEPRINT-AS-STR-REF]
    pub const fn as_str_ref(&self) -> &'static str {
        (*self).as_str()
    }
}

/// Zero-sized catalog type providing typed iteration and membership queries over all 13 canonical architecture blueprints.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `CanonicalArchitectureBlueprintsCatalog`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Zero-sized structural unit type backed exclusively by `CanonicalArchitectureBlueprint::ALL`.
///
/// ## 3. Preconditions
/// - Constructed at compile time via `CANONICAL_ARCHITECTURE_BLUEPRINTS`.
///
/// ## 4. Postconditions
/// - Exposes deterministic `iter()` and `contains()` operations.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1). Space Complexity: O(1) zero-sized type.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Type-safe catalog methods with no mutable state.
///
/// ## 7. Realization & Traceability
/// Realises: [deap_core::rules::CanonicalArchitectureBlueprintsCatalog] | SysML v2: PartDef(CanonicalArchitectureBlueprintsCatalog)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanonicalArchitectureBlueprintsCatalog;

/// Authoritative catalog of the 13 canonical architecture blueprints in `docs/architecture/blueprints/`.
pub const CANONICAL_ARCHITECTURE_BLUEPRINTS: CanonicalArchitectureBlueprintsCatalog =
    CanonicalArchitectureBlueprintsCatalog;

impl CanonicalArchitectureBlueprintsCatalog {
    /// Returns a deterministic iterator yielding all 13 canonical architecture blueprint filenames.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and Check 34 `(b)`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Yields exactly 13 `&'static str` filenames in declaration order.
    ///
    /// ## 3. Preconditions
    /// `self` is a valid catalog reference.
    ///
    /// ## 4. Postconditions
    /// Returns a `std::iter::Map` over `CanonicalArchitectureBlueprint::ALL.iter()`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1) iterator creation; O(13) traversal. Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Pure iterator projection; never panics.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RULES-CANONICAL-BLUEPRINTS-ITER]
    pub fn iter(
        &self,
    ) -> std::iter::Map<
        std::slice::Iter<'static, CanonicalArchitectureBlueprint>,
        fn(&'static CanonicalArchitectureBlueprint) -> &'static str,
    > {
        let project: fn(&'static CanonicalArchitectureBlueprint) -> &'static str =
            CanonicalArchitectureBlueprint::as_str_ref;
        CanonicalArchitectureBlueprint::ALL.iter().map(project)
    }

    /// Checks whether `candidate` matches one of the 13 canonical architecture blueprint filenames.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C architecture governance verification.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Returns `true` if and only if `candidate` equals `v.as_str()` for some `v` in `CanonicalArchitectureBlueprint::ALL`.
    ///
    /// ## 3. Preconditions
    /// `candidate` is a valid UTF-8 filename slice.
    ///
    /// ## 4. Postconditions
    /// Returns `true` on exact match, or `false` otherwise.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1) pattern match across 13 literal arms. Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Handles empty strings and arbitrary relative paths safely.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RULES-CANONICAL-BLUEPRINTS-CONTAINS]
    pub fn contains(&self, candidate: &str) -> bool {
        matches!(
            candidate,
            "DEAP_SYSML_V2_INGESTION_ENGINE_BLUEPRINT.md"
                | "SYSML_SSOT_BIDIRECTIONAL_SYNCHRONIZATION_ARCHITECTURE.md"
                | "DEAP_DETERMINISTIC_SAFETY_SPECIFICATION_COMPILER_BLUEPRINT.md"
                | "DEAP_MULTI_TOOLCHAIN_SYNTHESIS_ARCHITECTURE.md"
                | "DEAP_LOGICAL_INTERFACE_SPECIFICATION_BLUEPRINT.md"
                | "MULTI_PROVIDER_GITLAB_INFRASTRUCTURE_ARCHITECTURE.md"
                | "PERSISTENCE_ARCHITECTURE.md"
                | "SAFETY_CRITICAL_REALTIME_UI_FRAMEWORK.md"
                | "RUNTIME_METADATA_ENGINE.md"
                | "SPECKIT_NATIVE_INTEGRATION.md"
                | "DEAP_LOCAL_AIRGAPPED_DEEPSEEK_WORKSTATION_BLUEPRINT.md"
                | "DEAP_DEEPSEEK_HARNESS_INTEGRATION_BLUEPRINT.md"
                | "DEAP_POLY_REPO_BINARY_DISTRIBUTION_ARCHITECTURE.md"
        )
    }
}
