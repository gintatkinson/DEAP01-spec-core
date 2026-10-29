//! Architectural layer role taxonomy and dependency direction rules for modular code generation.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Module `modular_codegen::domain::layer_role` enforces unidirectional architectural stratification
//! across generated software partitions under DO-178C and ISO 26262 modular decomposition rules.
//! ## 2. Invariants & Mathematical Properties
//! Callers provide borrowed UTF-8 annotation strings and ordered `SysML` v2 package path slices.
//! ## 3. Preconditions
//! Every classified software partition resolves to one of three ordered strata with downward-only coupling.
//! ## 4. Postconditions
//! Unknown lexical layer annotations return `Option::None`, while component resolution defaults to `LayerRole::Domain`.
//! ## 5. Algorithmic Complexity Bounds
//! Operations run deterministically in bounded linear time over token bytes and package depth via fixed stack tables.
//! ## 6. Failure Modes & Fault Tolerance
//! Unit and mutation probes validate tier ordinals, labels, quote stripping, suffix matching, and coupling gates.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0607]
//! Module anchor: `crates/compile-sysml/src/modular_codegen/domain/layer_role.rs`.

use std::fmt;

/// Canonical three-tier architectural role assigned to each generated `SysML` v2 module partition.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Type `pub enum LayerRole` segregates generated units into `Domain`, `Application`, and `Presentation` tiers.
/// ## 2. Invariants & Mathematical Properties
/// Instances of `pub enum LayerRole` originate via explicit variants, `from_annotation`, or `resolve_part_layer`.
/// ## 3. Preconditions
/// Declaration order on `pub enum LayerRole` guarantees `Domain < Application < Presentation` matching `tier()`.
/// ## 4. Postconditions
/// Closed three-variant sum `pub enum LayerRole` makes invalid strata unrepresentable at compile time.
/// ## 5. Algorithmic Complexity Bounds
/// Copyable single-byte discriminant `pub enum LayerRole` guarantees `O(1)` stack transfer and zero heap use.
/// ## 6. Failure Modes & Fault Tolerance
/// Structural assertions on `pub enum LayerRole` verify total ordering, hashing, and label round-tripping.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Type anchor: `pub enum LayerRole { Domain, Application, Presentation }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LayerRole {
    Domain,
    Application,
    Presentation,
}

impl LayerRole {
    /// Maps a `LayerRole` variant to its strictly positive 1-based architectural tier ordinal.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// `pub const fn tier(self) -> u8` assigns monotonic numeric ranks gating cross-layer dependency checks.
    /// ## 2. Invariants & Mathematical Properties
    /// `pub const fn tier(self) -> u8` accepts any initialized by-value `LayerRole` without ambient state.
    /// ## 3. Preconditions
    /// `pub const fn tier(self) -> u8` returns `1` for `Domain`, `2` for `Application`, and `3` for `Presentation`.
    /// ## 4. Postconditions
    /// `pub const fn tier(self) -> u8` is total and infallible across all three closed enum variants.
    /// ## 5. Algorithmic Complexity Bounds
    /// `pub const fn tier(self) -> u8` evaluates deterministically in `const` or runtime `O(1)` time and space.
    /// ## 6. Failure Modes & Fault Tolerance
    /// Exhaustive checks confirm `pub const fn tier(self) -> u8` yields `[1, 2, 3]` aligned with `Ord`.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0607]
    /// Method anchor: `LayerRole::tier` numeric stratum projection.
    pub const fn tier(self) -> u8 {
        match self {
            Self::Domain => 1,
            Self::Application => 2,
            Self::Presentation => 3,
        }
    }

    /// Returns the canonical PascalCase static string identifier for a `LayerRole` variant.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// `pub const fn as_str(self) -> &'static str` supplies stable layer labels for audit diagnostics and manifests.
    /// ## 2. Invariants & Mathematical Properties
    /// `pub const fn as_str(self) -> &'static str` takes a copied `LayerRole` value with zero buffer lifetimes.
    /// ## 3. Preconditions
    /// `pub const fn as_str(self) -> &'static str` yields `"Domain"`, `"Application"`, or `"Presentation"`.
    /// ## 4. Postconditions
    /// `pub const fn as_str(self) -> &'static str` cannot fail because every match arm binds a static literal.
    /// ## 5. Algorithmic Complexity Bounds
    /// `pub const fn as_str(self) -> &'static str` executes in constant `O(1)` time with zero dynamic allocation.
    /// ## 6. Failure Modes & Fault Tolerance
    /// Bijection tests verify `pub const fn as_str(self) -> &'static str` round-trips via `from_annotation`.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0607]
    /// Method anchor: `LayerRole::as_str` static textual serialization.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Domain => "Domain",
            Self::Application => "Application",
            Self::Presentation => "Presentation",
        }
    }

    /// Parses a raw `SysML` v2 layer annotation or package segment into an optional `LayerRole`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// `pub fn from_annotation(raw: &str) -> Option<Self>` sanitizes raw model strings into canonical layer roles.
    /// ## 2. Invariants & Mathematical Properties
    /// `pub fn from_annotation(raw: &str) -> Option<Self>` accepts any UTF-8 slice with optional whitespace or `"`.
    /// ## 3. Preconditions
    /// `pub fn from_annotation(raw: &str) -> Option<Self>` lowercases input and matches exact keywords or `_` suffixes.
    /// ## 4. Postconditions
    /// `pub fn from_annotation(raw: &str) -> Option<Self>` returns `None` for empty, quote-only, or unknown tokens.
    /// ## 5. Algorithmic Complexity Bounds
    /// `pub fn from_annotation(raw: &str) -> Option<Self>` performs `O(n)` ASCII lowering and 11-rule bounded scans.
    /// ## 6. Failure Modes & Fault Tolerance
    /// Lexical tests on `pub fn from_annotation(raw: &str) -> Option<Self>` probe quotes, casing, suffixes, and `"ui"`.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0607]
    /// Method anchor: `LayerRole::from_annotation` lexical role classifier.
    pub fn from_annotation(raw: &str) -> Option<Self> {
        let normalized = raw.trim().trim_matches('"').trim().to_ascii_lowercase();
        for (token, role) in [
            ("domain", Self::Domain),
            ("application", Self::Application),
            ("lowering", Self::Application),
            ("presentation", Self::Presentation),
            ("emitters", Self::Presentation),
            ("ui", Self::Presentation),
        ] {
            if normalized == token {
                return Some(role);
            }
        }
        for (suffix, role) in [
            ("_domain", Self::Domain),
            ("_application", Self::Application),
            ("_lowering", Self::Application),
            ("_presentation", Self::Presentation),
            ("_emitters", Self::Presentation),
        ] {
            if normalized.ends_with(suffix) {
                return Some(role);
            }
        }
        None
    }

    /// Resolves a component's effective `LayerRole` from explicit part metadata or package ancestry.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// `pub fn resolve_part_layer(part_layer: Option<&str>, package_path: &[String]) -> Self` assigns deterministic strata.
    /// ## 2. Invariants & Mathematical Properties
    /// `pub fn resolve_part_layer(part_layer: Option<&str>, package_path: &[String]) -> Self` reads optional tag and path.
    /// ## 3. Preconditions
    /// `pub fn resolve_part_layer(part_layer: Option<&str>, package_path: &[String]) -> Self` prefers part, inner pkg, then `Domain`.
    /// ## 4. Postconditions
    /// `pub fn resolve_part_layer(part_layer: Option<&str>, package_path: &[String]) -> Self` falls back safely to `Self::Domain`.
    /// ## 5. Algorithmic Complexity Bounds
    /// `pub fn resolve_part_layer(part_layer: Option<&str>, package_path: &[String]) -> Self` runs in `O(d * m)` reverse scan time.
    /// ## 6. Failure Modes & Fault Tolerance
    /// Hierarchy tests on `pub fn resolve_part_layer(part_layer: Option<&str>, package_path: &[String]) -> Self` check precedence.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0607]
    /// Method anchor: `LayerRole::resolve_part_layer` hierarchical layer resolver.
    pub fn resolve_part_layer(part_layer: Option<&str>, package_path: &[String]) -> Self {
        if let Some(role) = part_layer.and_then(Self::from_annotation) {
            return role;
        }
        for segment in package_path.iter().rev() {
            if let Some(role) = Self::from_annotation(segment) {
                return role;
            }
        }
        Self::Domain
    }

    /// Evaluates whether a partition in `self` is architecturally permitted to depend on `target`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// `pub const fn permits_dependency_on(self, target: Self) -> bool` blocks illegal lower-to-upper layer imports.
    /// ## 2. Invariants & Mathematical Properties
    /// `pub const fn permits_dependency_on(self, target: Self) -> bool` takes source `self` and dependency `target`.
    /// ## 3. Preconditions
    /// `pub const fn permits_dependency_on(self, target: Self) -> bool` holds iff `self.tier() >= target.tier()`.
    /// ## 4. Postconditions
    /// `pub const fn permits_dependency_on(self, target: Self) -> bool` returns `false` on upward tier references.
    /// ## 5. Algorithmic Complexity Bounds
    /// `pub const fn permits_dependency_on(self, target: Self) -> bool` executes two `O(1)` tier projections in `const`.
    /// ## 6. Failure Modes & Fault Tolerance
    /// Full `3x3` matrix tests on `pub const fn permits_dependency_on(self, target: Self) -> bool` verify all nine pairs.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0607]
    /// Method anchor: `LayerRole::permits_dependency_on` directional coupling predicate.
    pub const fn permits_dependency_on(self, target: Self) -> bool {
        self.tier() >= target.tier()
    }
}

impl fmt::Display for LayerRole {
    /// Formats `LayerRole` into character output streams using its canonical PascalCase tier label.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// `fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result` renders unambiguous layer names in compiler diagnostics.
    /// ## 2. Invariants & Mathematical Properties
    /// `fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result` borrows `&self` and a mutable `fmt::Formatter` sink.
    /// ## 3. Preconditions
    /// `fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result` delegates directly to `f.write_str(self.as_str())`.
    /// ## 4. Postconditions
    /// `fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result` propagates stream `fmt::Error` results without aborting.
    /// ## 5. Algorithmic Complexity Bounds
    /// `fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result` writes one bounded `'static` slice with zero buffering.
    /// ## 6. Failure Modes & Fault Tolerance
    /// Formatting assertions confirm `fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result` equals `LayerRole::as_str`.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0607]
    /// Trait anchor: `<LayerRole as std::fmt::Display>::fmt` stream formatter.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
