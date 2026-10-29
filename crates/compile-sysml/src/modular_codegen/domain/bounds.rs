//! Quantitative modularity ceiling thresholds and deterministic diagnostic contracts for modular code generation.
//! ## 1. Safety Intent & Regulatory Scope
//! Defines the domain-layer quantitative structural envelope (`bounds.rs`) governing generated artifact modularity under DO-178C and ISO 26262 Partitioning rules.
//! ## 2. Invariants & Mathematical Properties
//! All modularity thresholds and diagnostic codes reside in pure, platform-independent value types without external I/O or mutable global state.
//! ## 3. Preconditions
//! Exposes `ModularityBounds`, `ModularityErrorCode`, and `ModularityDiagnostic` with total deterministic constructors, envelope predicates, and stable `"E0702"`..`"E0706"` identifiers.
//! ## 4. Postconditions
//! Every operation in this domain module is total and infallible, representing modularity defects explicitly as structured `ModularityDiagnostic` values.
//! ## 5. Algorithmic Complexity Bounds
//! All predicates and projections execute in constant `O(1)` time and stack space, while `ModularityDiagnostic` owns only two bounded `String` buffers.
//! ## 6. Failure Modes & Fault Tolerance
//! Verified via deterministic unit assertions covering canonical defaults, inclusive lower and upper envelope boundaries, diagnostic formatting, and error code mapping.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0607]
//! Anchors domain-tier structural complexity ceilings and `E0702` through `E0706` diagnostic reporting for the SysML v2 modular code generator.

/// Quantitative ceiling parameters constraining generated file length, AST item density, and control-flow complexity.
/// ## 1. Safety Intent & Regulatory Scope
/// `pub struct ModularityBounds` codifies auditable structural size and complexity ceilings so generated compilation units remain human-reviewable and statically verifiable.
/// ## 2. Invariants & Mathematical Properties
/// `pub struct ModularityBounds` holds five unsigned scalar limits (`max_file_lines`, `max_items_per_file`, `max_cyclomatic_complexity`, `max_action_steps_per_stage`, `max_transitions_per_handler`) validated via `ModularityBounds::is_valid_envelope`.
/// ## 3. Preconditions
/// Instances of `pub struct ModularityBounds` implement `Copy`, `Eq`, and `Default`, enabling zero-allocation propagation of immutable thresholds across codegen stages.
/// ## 4. Postconditions
/// Out-of-range field combinations in `pub struct ModularityBounds` never abort execution and are rejected cleanly through `ModularityBounds::is_valid_envelope(&self) -> bool`.
/// ## 5. Algorithmic Complexity Bounds
/// `pub struct ModularityBounds` occupies exactly five machine words (`5 * size_of::<usize>()`) on the stack with zero heap allocations and bit-exact equality semantics.
/// ## 6. Failure Modes & Fault Tolerance
/// `pub struct ModularityBounds` is proven by structural equality tests comparing `ModularityBounds::canonical()` with `ModularityBounds::default()` and boundary mutation probes on all five fields.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Encapsulates the five quantitative modularity limits enforced by the `compile-sysml` modular codegen verification gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModularityBounds {
    /// Maximum permitted physical line count per generated source file (`50..=400`).
    pub max_file_lines: usize,
    /// Maximum permitted top-level AST item declarations per generated file (`1..=12`).
    pub max_items_per_file: usize,
    /// Maximum permitted McCabe cyclomatic complexity `v(G)` per generated function (`1..=8`).
    pub max_cyclomatic_complexity: usize,
    /// Maximum permitted sequential action steps emitted within a single pipeline stage (`1..=8`).
    pub max_action_steps_per_stage: usize,
    /// Maximum permitted state transitions dispatched within a single state handler (`1..=8`).
    pub max_transitions_per_handler: usize,
}

impl ModularityBounds {
    /// Returns the canonical baseline modularity ceiling thresholds for generated compilation units.
    /// ## 1. Safety Intent & Regulatory Scope
    /// `pub const fn canonical() -> Self` establishes the normative compile-time modularity budget preventing monolithic generated artifacts in safety-critical builds.
    /// ## 2. Invariants & Mathematical Properties
    /// `pub const fn canonical() -> Self` accepts zero runtime arguments and requires no ambient configuration or initialization preconditions.
    /// ## 3. Preconditions
    /// `pub const fn canonical() -> Self` returns `400` file lines, `12` items, `8` cyclomatic complexity, `5` action steps per stage, and `6` transitions per handler.
    /// ## 4. Postconditions
    /// `pub const fn canonical() -> Self` is a pure compile-time constant constructor with zero failure paths or runtime branches.
    /// ## 5. Algorithmic Complexity Bounds
    /// `pub const fn canonical() -> Self` evaluates in `O(1)` time (`v(G) = 1`) with zero heap allocation and deterministic constant-folded output.
    /// ## 6. Failure Modes & Fault Tolerance
    /// `pub const fn canonical() -> Self` is verified by asserting exact field values on `ModularityBounds::canonical()` and confirming `is_valid_envelope()` holds.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0607]
    /// Supplies the authoritative baseline configuration for `ModularityBounds` across all modular codegen partitioning passes.
    #[must_use]
    pub const fn canonical() -> Self {
        Self {
            max_file_lines: 400,
            max_items_per_file: 12,
            max_cyclomatic_complexity: 8,
            max_action_steps_per_stage: 5,
            max_transitions_per_handler: 6,
        }
    }

    /// Checks whether all five quantitative modularity thresholds lie within their certified inclusive operational envelopes.
    /// ## 1. Safety Intent & Regulatory Scope
    /// `pub fn is_valid_envelope(&self) -> bool` guards the modular code generator against degenerate zero-sized budgets or permissive ceilings exceeding reviewable limits.
    /// ## 2. Invariants & Mathematical Properties
    /// `pub fn is_valid_envelope(&self) -> bool` borrows an initialized `ModularityBounds` value without mutating state or assuming pre-validated fields.
    /// ## 3. Preconditions
    /// `pub fn is_valid_envelope(&self) -> bool` returns `true` iff lines are in `50..=400`, items in `1..=12`, and complexity, action steps, and transitions each in `1..=8`.
    /// ## 4. Postconditions
    /// `pub fn is_valid_envelope(&self) -> bool` handles arbitrary `usize` inputs including `0` and `usize::MAX` without arithmetic overflow, returning `false` outside the envelope.
    /// ## 5. Algorithmic Complexity Bounds
    /// `pub fn is_valid_envelope(&self) -> bool` executes at most five inclusive range membership checks in `O(1)` time and stack space with cyclomatic complexity `v(G) = 5`.
    /// ## 6. Failure Modes & Fault Tolerance
    /// `pub fn is_valid_envelope(&self) -> bool` is verified via boundary-value analysis testing `49`, `50`, `400`, `401` for line limits and `0`, `1`, upper bound, and upper-bound-plus-one for remaining fields.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0607]
    /// Enforces the mathematical admissibility invariant over `ModularityBounds` configurations prior to codegen emission.
    #[must_use]
    pub fn is_valid_envelope(&self) -> bool {
        (50..=400).contains(&self.max_file_lines)
            && (1..=12).contains(&self.max_items_per_file)
            && (1..=8).contains(&self.max_cyclomatic_complexity)
            && (1..=8).contains(&self.max_action_steps_per_stage)
            && (1..=8).contains(&self.max_transitions_per_handler)
    }
}

impl Default for ModularityBounds {
    /// Constructs the default `ModularityBounds` configuration by delegating directly to the canonical ceiling contract.
    /// ## 1. Safety Intent & Regulatory Scope
    /// `fn default() -> Self` in `impl Default for ModularityBounds` guarantees default-initialized bounds always enforce the certified canonical modularity ceilings.
    /// ## 2. Invariants & Mathematical Properties
    /// `fn default() -> Self` requires zero caller arguments and preserves total equivalence with `ModularityBounds::canonical()`.
    /// ## 3. Preconditions
    /// `fn default() -> Self` yields a `ModularityBounds` value identical to `Self::canonical()` whose `is_valid_envelope()` predicate evaluates to `true`.
    /// ## 4. Postconditions
    /// `fn default() -> Self` cannot fail because delegation to `ModularityBounds::canonical()` performs no fallible computation or dynamic allocation.
    /// ## 5. Algorithmic Complexity Bounds
    /// `fn default() -> Self` completes in `O(1)` time and constant stack space with cyclomatic complexity `v(G) = 1`.
    /// ## 6. Failure Modes & Fault Tolerance
    /// `fn default() -> Self` is verified by asserting `ModularityBounds::default() == ModularityBounds::canonical()` under unit test execution.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0607]
    /// Binds Rust's standard `Default` trait on `ModularityBounds` to the normative `[REQ-0607]` canonical ceiling profile.
    fn default() -> Self {
        Self::canonical()
    }
}

/// Stable taxonomy of deterministic error codes classifying architectural modularity, layering, fixed-point, and I/O violations.
/// ## 1. Safety Intent & Regulatory Scope
/// `pub enum ModularityErrorCode` provides an exhaustive, machine-checkable classification of modularity faults (`E0702` through `E0706`) for audit and gate enforcement.
/// ## 2. Invariants & Mathematical Properties
/// `pub enum ModularityErrorCode` is a closed five-variant fieldless enumeration carrying total ordering (`Ord`) and deterministic hashing (`Hash`).
/// ## 3. Preconditions
/// Every `pub enum ModularityErrorCode` variant maps bijectively onto one stable five-character identifier via `ModularityErrorCode::code(self) -> &'static str`.
/// ## 4. Postconditions
/// Exhaustive pattern matching over `pub enum ModularityErrorCode` eliminates unrecognized diagnostic categories at compile time.
/// ## 5. Algorithmic Complexity Bounds
/// `pub enum ModularityErrorCode` is represented as a single-byte copyable discriminant with `O(1)` comparison and zero heap footprint.
/// ## 6. Failure Modes & Fault Tolerance
/// `pub enum ModularityErrorCode` is verified by exhaustive variant tests checking discriminant ordering, equality, and exact `"E0702"`..`"E0706"` string projection.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Defines the closed diagnostic code set emitted by modular codegen DAG, layering, bounds, fixed-point, and persistence checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModularityErrorCode {
    /// Directed dependency cycle detected in the generated module graph (`E0702`).
    DirectedCycleE0702,
    /// Forbidden upward architectural layer import detected (`E0703`).
    UpwardLayerImportE0703,
    /// Quantitative file, item, or complexity ceiling exceeded (`E0704`).
    BoundsViolationE0704,
    /// Regeneration output diverged from the deterministic fixed-point artifact (`E0705`).
    FixedPointDriftE0705,
    /// Filesystem read or write operation failed during modular codegen (`E0706`).
    IoFailureE0706,
}

impl ModularityErrorCode {
    /// Projects this `ModularityErrorCode` variant onto its canonical five-character static diagnostic identifier.
    /// ## 1. Safety Intent & Regulatory Scope
    /// `pub const fn code(self) -> &'static str` guarantees stable, unambiguous diagnostic identifiers across CLI output, audit logs, and verification gates.
    /// ## 2. Invariants & Mathematical Properties
    /// `pub const fn code(self) -> &'static str` accepts any valid `ModularityErrorCode` discriminant by value.
    /// ## 3. Preconditions
    /// `pub const fn code(self) -> &'static str` returns `"E0702"`, `"E0703"`, `"E0704"`, `"E0705"`, or `"E0706"` matching the variant suffix.
    /// ## 4. Postconditions
    /// `pub const fn code(self) -> &'static str` uses an exhaustive `match` expression with zero wildcard arms and zero runtime failure modes.
    /// ## 5. Algorithmic Complexity Bounds
    /// `pub const fn code(self) -> &'static str` executes in `O(1)` time (`v(G) = 5`) returning a `'static` read-only string slice without allocation.
    /// ## 6. Failure Modes & Fault Tolerance
    /// `pub const fn code(self) -> &'static str` is verified by asserting the exact five-byte static string literal returned for each of the five `ModularityErrorCode` variants.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0607]
    /// Maps `ModularityErrorCode` discriminants to normative compiler error codes `"E0702"` through `"E0706"`.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::DirectedCycleE0702 => "E0702",
            Self::UpwardLayerImportE0703 => "E0703",
            Self::BoundsViolationE0704 => "E0704",
            Self::FixedPointDriftE0705 => "E0705",
            Self::IoFailureE0706 => "E0706",
        }
    }
}

/// Structured modularity violation record pairing a typed `ModularityErrorCode` with the offending element and remediation message.
/// ## 1. Safety Intent & Regulatory Scope
/// `pub struct ModularityDiagnostic` preserves complete attribution (`code`, `element`, `message`) whenever modular code generation rejects an artifact or dependency edge.
/// ## 2. Invariants & Mathematical Properties
/// `pub struct ModularityDiagnostic` pairs one `ModularityErrorCode` with owned UTF-8 `element` and `message` strings describing the violating model or file target.
/// ## 3. Preconditions
/// `pub struct ModularityDiagnostic` implements `Clone`, `Eq`, `Display`, and `std::error::Error` for seamless integration into compiler result pipelines.
/// ## 4. Postconditions
/// `pub struct ModularityDiagnostic` is itself the reified error carrier and introduces zero unchecked indexing or fallible internal invariants.
/// ## 5. Algorithmic Complexity Bounds
/// `pub struct ModularityDiagnostic` performs deterministic value comparisons and owns solely the two `String` allocations supplied at construction.
/// ## 6. Failure Modes & Fault Tolerance
/// `pub struct ModularityDiagnostic` is verified by testing `ModularityDiagnostic::new`, `Display` rendering `"[{code}] {element}: {message}"`, and `std::error::Error` trait object compatibility.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Provides the canonical diagnostic payload returned by modular codegen architectural and quantitative verification passes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModularityDiagnostic {
    /// Classified modularity error code (`E0702`..`E0706`).
    pub code: ModularityErrorCode,
    /// Qualified SysML element, module path, or artifact identifier associated with the violation.
    pub element: String,
    /// Human-readable diagnostic explanation detailing the violated modularity invariant.
    pub message: String,
}

impl ModularityDiagnostic {
    /// Creates a new `ModularityDiagnostic` from a classified error code, target element identifier, and descriptive message.
    /// ## 1. Safety Intent & Regulatory Scope
    /// `pub fn new(code: ModularityErrorCode, element: impl Into<String>, message: impl Into<String>) -> Self` enforces uniform diagnostic construction across all modular codegen validators.
    /// ## 2. Invariants & Mathematical Properties
    /// `pub fn new(code: ModularityErrorCode, element: impl Into<String>, message: impl Into<String>) -> Self` accepts any `ModularityErrorCode` and `Into<String>` conversions for `element` and `message`.
    /// ## 3. Preconditions
    /// `pub fn new(code: ModularityErrorCode, element: impl Into<String>, message: impl Into<String>) -> Self` stores `code`, `element.into()`, and `message.into()` verbatim without truncation.
    /// ## 4. Postconditions
    /// `pub fn new(code: ModularityErrorCode, element: impl Into<String>, message: impl Into<String>) -> Self` is infallible under normal Rust string conversion semantics.
    /// ## 5. Algorithmic Complexity Bounds
    /// `pub fn new(code: ModularityErrorCode, element: impl Into<String>, message: impl Into<String>) -> Self` runs in `O(|element| + |message|)` conversion time (`O(1)` when passed owned `String`s) with `v(G) = 1`.
    /// ## 6. Failure Modes & Fault Tolerance
    /// `pub fn new(code: ModularityErrorCode, element: impl Into<String>, message: impl Into<String>) -> Self` is verified by constructing diagnostics from `&str` and `String` inputs and asserting field equality.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0607]
    /// Standardizes instantiation of `ModularityDiagnostic` records throughout the modular codegen domain and application layers.
    #[must_use]
    pub fn new(
        code: ModularityErrorCode,
        element: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            element: element.into(),
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ModularityDiagnostic {
    /// Formats this `ModularityDiagnostic` in canonical `"[E070X] element: message"` notation.
    /// ## 1. Safety Intent & Regulatory Scope
    /// `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` renders deterministic, machine-parseable diagnostic text for safety gate reporting.
    /// ## 2. Invariants & Mathematical Properties
    /// `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` borrows `self` immutably and writes exclusively into the caller-supplied `std::fmt::Formatter`.
    /// ## 3. Preconditions
    /// `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` emits `"[{}] {}: {}"` populated with `self.code.code()`, `self.element`, and `self.message`.
    /// ## 4. Postconditions
    /// `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` propagates underlying sink errors unmodified via `std::fmt::Result` without aborting.
    /// ## 5. Algorithmic Complexity Bounds
    /// `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` streams directly to the formatter in `O(|element| + |message|)` time with zero intermediate heap allocation and `v(G) = 1`.
    /// ## 6. Failure Modes & Fault Tolerance
    /// `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` is verified by calling `.to_string()` on `ModularityDiagnostic` fixtures and asserting exact output parity.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0607]
    /// Implements canonical human- and tool-readable rendering for `ModularityDiagnostic` compiler diagnostics.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}: {}", self.code.code(), self.element, self.message)
    }
}

impl std::error::Error for ModularityDiagnostic {}
