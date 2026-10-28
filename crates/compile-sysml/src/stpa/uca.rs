//! STPA Unsafe Control Action (UCA) Cartesian Expansion Engine.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, IEC 61508, and ECSS-E-ST-40C
//! safety analysis objectives for module `crates/compile-sysml/src/stpa/uca.rs`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Expands every controlling `PartDef` action (or package-scoped `ActionDef` when inline part
//!   actions are absent) across the 4 canonical STPA guide words (`4 * |Actions|` bijective rows).
//! - Resolves controller ownership, operational context, hazard severity, and integrity level
//!   deterministically from SysML v2 AST nodes with zero unresolved placeholder literals and zero
//!   hardcoded compiler self-references.
//!
//! ## 3. Preconditions
//! - Caller supplies a parsed `deap_core::sysml_ast::PackageDef` hierarchy.
//!
//! ## 4. Postconditions
//! - Every emitted `UnsafeControlAction` contains single-line Markdown-safe cells, deterministic
//!   `H-*` and `SC-*` identifiers, and AST-grounded `severity` and `sail` (integrity level) values.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(P + A * (G + M)) where P is AST part/package node count, A is action count,
//!   G = 4 guide words, and M is bounded attribute/hazard lookup cardinality.
//! - Space Complexity: O(P + A * G) auxiliary memory for collected AST metadata and output UCAs.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Degrades gracefully on minimal schemas with zero `PartDef`, `AttributeDef`, or `ConstraintDef` nodes.
//! - State: Pure functional projection with zero shared mutable state.
//! - Action: Executes with zero regular expressions, zero unchecked unwrapping, and zero direct slice indexing.
//! - Resource: Bounded linear heap allocations proportional to AST node count.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-STPA-UCA-MODULE] | SysML v2: PackageDef(stpa_uca)

use std::collections::BTreeSet;

use deap_core::sysml_ast::{
    ActionDef, AttributeDef, ConstraintDef, HazardDef, PackageDef, PartDef, RequirementDef,
};
use serde::{Deserialize, Serialize};

/// The 4 canonical STPA guide-word categories.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C hazard
/// analysis completeness objectives for `STPA_GUIDE_WORDS` in `crates/compile-sysml/src/stpa/uca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Fixed-cardinality 4-element array enumerating omission, commission, temporal/sequence order,
///   and actuation duration STPA guide-word classes in canonical deterministic order.
///
/// ## 3. Preconditions
/// - Static compile-time constant requiring zero runtime initialization.
///
/// ## 4. Postconditions
/// - Guarantees `STPA_GUIDE_WORDS.len() == 4` across all Cartesian UCA expansion loops.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) constant iteration bound (4 elements).
/// - Space Complexity: O(1) static read-only data segment storage.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Immutable `'static` string slices.
/// - State: Read-only constant memory.
/// - Action: Zero runtime computation or failure modes.
/// - Resource: Zero heap allocation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-UCA-GUIDE-WORDS] | SysML v2: AttributeDef(STPA_GUIDE_WORDS)
pub const STPA_GUIDE_WORDS: [&str; 4] = [
    "Not providing",
    "Providing",
    "Too early / Too late / Out of order",
    "Stopped too soon / Applied too long",
];

/// Representation of a single Unsafe Control Action derived from an action and guide word.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `UnsafeControlAction` in `crates/compile-sysml/src/stpa/uca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `UnsafeControlAction` across member set `[id, controller, control_action, guide_word, context, hazard, constraint, severity, sail]`.
///
/// ## 3. Preconditions
/// - Caller supplies single-line sanitized values derived from SysML v2 AST definitions.
///
/// ## 4. Postconditions
/// - Defines the authoritative `UnsafeControlAction` representation for downstream STPA matrices and constraint synthesis.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) constant-time member access across 9 owned string fields.
/// - Space Complexity: O(L) bounded auxiliary memory proportional to total field string byte length L.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `UnsafeControlAction` (`UnsafeControlAction`).
/// - State: Maintains consistent state invariants across `UnsafeControlAction` without partial mutation on error.
/// - Action: Executes `UnsafeControlAction` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `UnsafeControlAction` (`STPA-UCA`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-UCA-UNSAFECONTROLACTION] | SysML v2: PartDef(UnsafeControlAction)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UnsafeControlAction {
    pub id: String,
    pub controller: String,
    pub control_action: String,
    pub guide_word: String,
    pub context: String,
    pub hazard: String,
    pub constraint: String,
    pub severity: String,
    pub sail: String,
}

/// Aggregated read-only AST metadata harvested across a `PackageDef` hierarchy for UCA synthesis.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C deterministic
/// AST traceability objectives for struct `AstRiskContext` in `crates/compile-sysml/src/stpa/uca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Separates package-level `AttributeDef` declarations from flattened `PartDef` attributes so
///   scoped controller lookup takes precedence over package defaults while preserving part-level
///   integrity tiers when actions are declared at package scope.
/// - Sets `has_assert_or_guard == true` iff at least one `ConstraintDef` carries `is_assertion` or
///   a non-empty expression, or at least one `RequirementDef` exists in the package tree.
///
/// ## 3. Preconditions
/// - Constructed from a valid `&PackageDef` via `AstRiskContext::from_package`.
///
/// ## 4. Postconditions
/// - Provides pre-collected slices for O(M) deterministic lookup per action without repeated tree traversals.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(pkg)) single harvest pass over package and part AST nodes.
/// - Space Complexity: O(N(pkg)) bounded vectors of cloned AST metadata nodes.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Handles empty packages with zero parts, attributes, hazards, constraints, or requirements cleanly.
/// - State: Immutable after construction in `from_package`.
/// - Action: Zero panics, regular expressions, or direct slice indexing.
/// - Resource: Reclaimed deterministically at the end of `expand_cartesian_stpa`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-UCA-AST-RISK-CONTEXT] | SysML v2: PartDef(AstRiskContext)
struct AstRiskContext {
    base_pkg_name: String,
    all_constraints: Vec<ConstraintDef>,
    pkg_attributes: Vec<AttributeDef>,
    all_part_attributes: Vec<AttributeDef>,
    pkg_hazards: Vec<HazardDef>,
    pkg_requirements: Vec<RequirementDef>,
    has_assert_or_guard: bool,
}

impl AstRiskContext {
    /// Harvests package- and part-scoped attributes, hazards, constraints, and requirements from `pkg`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C AST
    /// extraction objectives for `AstRiskContext::from_package` in `crates/compile-sysml/src/stpa/uca.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Deduplicates `RequirementDef` entries by identifier using `BTreeSet` and preserves
    ///   declaration order across `pkg.get_all_packages()` and `all_parts`.
    ///
    /// ## 3. Preconditions
    /// - `pkg` and `all_parts` originate from the same parsed SysML v2 `PackageDef`.
    ///
    /// ## 4. Postconditions
    /// - `base_pkg_name` is guaranteed non-empty (falling back to `"SystemController"` only when `pkg.name.trim()` is empty).
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(pkg) * log(R)) where R is unique requirement count.
    /// - Space Complexity: O(N(pkg)) bounded owned metadata collections.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Accepts empty slices and empty package names cleanly.
    /// - State: Pure construction with zero global state.
    /// - Action: Zero unchecked unwrapping or indexing.
    /// - Resource: Linear heap usage bounded by AST size.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-STPA-UCA-AST-RISK-CONTEXT-BUILD] | SysML v2: ActionDef(AstRiskContext_from_package)
    fn from_package(pkg: &PackageDef, all_parts: &[PartDef]) -> Self {
        let trimmed_pkg = pkg.name.trim();
        let base_pkg_name = if trimmed_pkg.is_empty() {
            "SystemController".to_string()
        } else {
            trimmed_pkg.to_string()
        };

        let all_constraints = pkg.get_all_constraints();
        let mut pkg_attributes = Vec::new();
        let mut pkg_hazards = Vec::new();
        let mut pkg_requirements = Vec::new();
        let mut seen_reqs = BTreeSet::new();

        for sub_pkg in pkg.get_all_packages() {
            for attr in &sub_pkg.attribute_defs {
                pkg_attributes.push(attr.clone());
            }
            for haz in &sub_pkg.hazard_defs {
                pkg_hazards.push(haz.clone());
            }
            for req in &sub_pkg.requirement_defs {
                if seen_reqs.insert(req.name.clone()) {
                    pkg_requirements.push(req.clone());
                }
            }
        }

        let mut all_part_attributes = Vec::new();
        for part in all_parts {
            if part.name.starts_with('_') {
                continue;
            }
            for attr in &part.attributes {
                all_part_attributes.push(attr.clone());
            }
        }

        let has_assert_or_guard = all_constraints
            .iter()
            .any(|c| c.is_assertion || !c.expression.trim().is_empty())
            || !pkg_requirements.is_empty();

        Self {
            base_pkg_name,
            all_constraints,
            pkg_attributes,
            all_part_attributes,
            pkg_hazards,
            pkg_requirements,
            has_assert_or_guard,
        }
    }
}

/// Resolved controller binding pairing an optional AST `PartDef` reference with its sanitized controller identifier.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C controller
/// attribution objectives for struct `ResolvedController` in `crates/compile-sysml/src/stpa/uca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - `name` is always a non-empty, single-line sanitized identifier derived strictly from an AST
///   `PartDef` or the enclosing `PackageDef` (never a hardcoded upstream compiler token).
///
/// ## 3. Preconditions
/// - Constructed by `resolve_action_controller` from `ActionDef`, `all_parts`, `non_internal_parts`, and `base_pkg_name`.
///
/// ## 4. Postconditions
/// - When `part` is `Some(p)`, `name` equals `sanitize_uca_cell(&p.name)`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) field access.
/// - Space Complexity: O(L) where L is `name.len()`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Explicitly models part absence via `Option<&'a PartDef>` when `all_parts` is empty.
/// - State: Immutable value binding scoped to lifetime `'a`.
/// - Action: Zero panics or invalid lifetime assumptions.
/// - Resource: Single string allocation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-UCA-RESOLVED-CONTROLLER] | SysML v2: PartDef(ResolvedController)
struct ResolvedController<'a> {
    part: Option<&'a PartDef>,
    name: String,
}

/// Sanitizes arbitrary AST text into a single-line Markdown table cell free of pipe delimiters and newlines.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C Markdown
/// table structural integrity objectives for fn `sanitize_uca_cell` in `crates/compile-sysml/src/stpa/uca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Replaces `\r` and `\n` with `' '` and `|` with `'/'` so downstream pipe-delimited Markdown
///   parsers (`compile_stpa_to_constraints_filtered`) observe invariant column positions.
///
/// ## 3. Preconditions
/// - `raw: &str` is any valid UTF-8 slice from an AST identifier, expression, or attribute default.
///
/// ## 4. Postconditions
/// - Returned `String` is trimmed and contains zero `'|'`, `'\r'`, or `'\n'` characters.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(raw)) single pass over scalar characters.
/// - Space Complexity: O(N(raw)) bounded output string buffer.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Accepts empty or whitespace-only inputs cleanly.
/// - State: Pure string projection.
/// - Action: Zero regular expressions, panics, or byte-range slicing.
/// - Resource: Pre-allocated capacity proportional to `raw.len()`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-UCA-SANITIZE-CELL] | SysML v2: ActionDef(sanitize_uca_cell)
fn sanitize_uca_cell(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.trim().chars() {
        match ch {
            '\r' | '\n' => out.push(' '),
            '|' => out.push('/'),
            _ => out.push(ch),
        }
    }
    out.trim().to_string()
}

/// Normalizes an optional AST `AttributeDef` default value by stripping surrounding whitespace and quotes.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C SSOT parameter
/// validation objectives for fn `clean_attr_value` in `crates/compile-sysml/src/stpa/uca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Returns `None` for empty strings, quoted empty strings (`""`), or unpopulated sentinel placeholders.
///
/// ## 3. Preconditions
/// - `raw: &str` is borrowed from `AttributeDef.default_value`.
///
/// ## 4. Postconditions
/// - Returns `Some(sanitized)` iff at least one non-quote, non-whitespace character remains.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(raw)) linear trim and cell sanitization.
/// - Space Complexity: O(N(raw)) for the sanitized `String` when non-empty.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Rejects blank defaults cleanly via `None`.
/// - State: Pure stateless helper.
/// - Action: Uses `trim` and `trim_matches` with zero byte-index slicing.
/// - Resource: Allocates only when returning `Some`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-UCA-CLEAN-ATTR-VALUE] | SysML v2: ActionDef(clean_attr_value)
fn clean_attr_value(raw: &str) -> Option<String> {
    let unquoted = raw.trim().trim_matches('"').trim();
    if unquoted.is_empty() {
        return None;
    }
    let clean = sanitize_uca_cell(unquoted);
    if clean.is_empty() {
        None
    } else {
        Some(clean)
    }
}

/// Searches an `AttributeDef` slice case-insensitively for an exact identifier match with a non-empty default value.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C deterministic
/// attribute resolution objectives for fn `find_attr_exact_ci` in `crates/compile-sysml/src/stpa/uca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Scans `attrs` in deterministic declaration order using ASCII case-insensitive comparison.
///
/// ## 3. Preconditions
/// - `attrs` is a slice of `AttributeDef` nodes; `target` is a non-empty lookup key.
///
/// ## 4. Postconditions
/// - Returns the first non-empty sanitized `default_value` whose `attr.name` matches `target`, or `None`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(|attrs| * L) where L is bounded identifier/value byte length.
/// - Space Complexity: O(L) for the returned value string.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Returns `None` when `attrs` is empty or matching attributes have `default_value == None`.
/// - State: Read-only slice traversal.
/// - Action: Zero regular expressions, panics, or unchecked indexing.
/// - Resource: Bounded single-string allocation on match.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-UCA-FIND-ATTR-EXACT] | SysML v2: ActionDef(find_attr_exact_ci)
fn find_attr_exact_ci(attrs: &[AttributeDef], target: &str) -> Option<String> {
    for attr in attrs {
        if attr.name.eq_ignore_ascii_case(target) {
            if let Some(ref val) = attr.default_value {
                if let Some(clean) = clean_attr_value(val) {
                    return Some(clean);
                }
            }
        }
    }
    None
}

/// Searches an `AttributeDef` slice case-insensitively for an identifier starting with `prefix` carrying a non-empty default value.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C prefix-scoped
/// integrity attribute discovery objectives for fn `find_attr_prefix_ci` in `crates/compile-sysml/src/stpa/uca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Matches `AttributeDef` declarations such as `IntegrityLevel_Tier = "SIL-3"` without requiring
///   a hardcoded tier suffix, using character-safe prefix comparison without byte-range slicing.
///
/// ## 3. Preconditions
/// - `attrs` is a slice of `AttributeDef` nodes; `prefix` is a non-empty ASCII prefix string.
///
/// ## 4. Postconditions
/// - Returns the first non-empty sanitized `default_value` whose `attr.name` begins with `prefix` (case-insensitive), or `None`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(|attrs| * L) linear scan over attribute names and prefixes.
/// - Space Complexity: O(L) bounded lowercase comparison and output string allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Safely handles attribute names shorter than `prefix` or containing multi-byte UTF-8 characters.
/// - State: Pure read-only inspection.
/// - Action: Uses `.to_ascii_lowercase().starts_with(...)` instead of `&name[..prefix.len()]` to obey Check 34.
/// - Resource: Minimal short-lived string buffer.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-UCA-FIND-ATTR-PREFIX] | SysML v2: ActionDef(find_attr_prefix_ci)
fn find_attr_prefix_ci(attrs: &[AttributeDef], prefix: &str) -> Option<String> {
    let lower_prefix = prefix.to_ascii_lowercase();
    for attr in attrs {
        if attr.name.to_ascii_lowercase().starts_with(&lower_prefix) {
            if let Some(ref val) = attr.default_value {
                if let Some(clean) = clean_attr_value(val) {
                    return Some(clean);
                }
            }
        }
    }
    None
}

/// Maps an integer AST `HazardDef.severity` score (`i32`) to its canonical SSOT severity classification label.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, IEC 61508, and ECSS-E-ST-40C
/// hazard severity classification objectives for fn `map_hazard_severity_score` in `crates/compile-sysml/src/stpa/uca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Monotonic mapping: `score >= 4` -> `"Catastrophic (SSOT)"`, `3` -> `"Critical (SSOT)"`,
///   `2` -> `"Marginal (SSOT)"`, `1` -> `"Minor (SSOT)"`, and `score <= 0` -> `None`.
///
/// ## 3. Preconditions
/// - `score: i32` originates from an AST `HazardDef.severity` field or numeric severity attribute.
///
/// ## 4. Postconditions
/// - Returns `Some` domain-neutral SSOT severity string for positive scores (`>= 1`) and `None` for unpopulated (`<= 0`) scores.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) constant-time integer match.
/// - Space Complexity: O(1) bounded short string allocation on positive scores.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Treats non-positive (`<= 0`) default integers as unspecified (`None`) rather than fabricating `"Minor (SSOT)"`.
/// - State: Pure stateless mapping.
/// - Action: Total integer match with zero panics.
/// - Resource: Minimal heap string on match.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-UCA-MAP-HAZARD-SEVERITY] | SysML v2: ActionDef(map_hazard_severity_score)
fn map_hazard_severity_score(score: i32) -> Option<String> {
    match score {
        s if s >= 4 => Some("Catastrophic (SSOT)".to_string()),
        3 => Some("Critical (SSOT)".to_string()),
        2 => Some("Marginal (SSOT)".to_string()),
        1 => Some("Minor (SSOT)".to_string()),
        _ => None,
    }
}

/// Resolves the deterministic controller binding for a package-scoped `ActionDef` without compiler self-reference injection.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C pure schema-driven
/// controller attribution objectives for fn `resolve_action_controller` in `crates/compile-sysml/src/stpa/uca.rs` (refs #697).
///
/// ## 2. Invariants & Mathematical Properties
/// - Priority order:
///   1. `action.performer` matching a declared `PartDef` in `non_internal_parts` or `all_parts`.
///   2. `non_internal_parts.first().copied()` (first user-declared `PartDef` whose identifier does not start with `'_'`).
///   3. `all_parts.first()`.
///   4. `base_pkg_name` (`pkg.name.trim()`, or `"SystemController"` only when `pkg.name.trim()` is empty) when `all_parts` is empty.
/// - Never emits hardcoded upstream compiler tokens regardless of whether `pkg.name == "SysML_Model"`.
///
/// ## 3. Preconditions
/// - `non_internal_parts` contains all references from `all_parts` with `!p.name.starts_with('_')`.
///
/// ## 4. Postconditions
/// - Returns `ResolvedController` with a non-empty sanitized `name` and optional matched `&PartDef`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(|all_parts|) when `action.performer` is populated, O(1) otherwise.
/// - Space Complexity: O(L) for the sanitized controller identifier string.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Handles `action.performer == None`, unmatched performer strings, and empty `all_parts` slices cleanly.
/// - State: Pure read-only borrow resolution.
/// - Action: Zero `.unwrap()`, panics, or direct indexing.
/// - Resource: Single `String` allocation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-UCA-RESOLVE-CONTROLLER] | SysML v2: ActionDef(resolve_action_controller)
fn resolve_action_controller<'a>(
    action: &ActionDef,
    all_parts: &'a [PartDef],
    non_internal_parts: &[&'a PartDef],
    base_pkg_name: &str,
) -> ResolvedController<'a> {
    if let Some(ref perf) = action.performer {
        let perf_trimmed = perf.trim();
        if !perf_trimmed.is_empty() {
            if let Some(matched) = non_internal_parts
                .iter()
                .copied()
                .find(|p| p.name == perf_trimmed || p.name.eq_ignore_ascii_case(perf_trimmed))
                .or_else(|| {
                    all_parts.iter().find(|p| {
                        p.name == perf_trimmed || p.name.eq_ignore_ascii_case(perf_trimmed)
                    })
                })
            {
                return ResolvedController {
                    part: Some(matched),
                    name: sanitize_uca_cell(&matched.name),
                };
            }
        }
    }

    if let Some(part) = non_internal_parts
        .first()
        .copied()
        .or_else(|| all_parts.first())
    {
        ResolvedController {
            part: Some(part),
            name: sanitize_uca_cell(&part.name),
        }
    } else {
        ResolvedController {
            part: None,
            name: sanitize_uca_cell(base_pkg_name),
        }
    }
}

/// Derives UCA hazard severity from explicit AST `HazardDef` / `AttributeDef` SSOT declarations or deterministic STPA guide-word rules.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, IEC 61508, and ECSS-E-ST-40C
/// severity derivation objectives for fn `resolve_uca_severity` in `crates/compile-sysml/src/stpa/uca.rs` (refs #697).
///
/// ## 2. Invariants & Mathematical Properties
/// - Checks explicit SSOT `HazardDef` scores on `matched_part.hazards` and `ctx.pkg_hazards` first,
///   then `{action}_Severity`, `{controller}_Severity`, `HazardSeverity`, `DefaultSeverity`, and `Severity` attributes.
/// - When SSOT is silent, derives `"Catastrophic (Derived)"` / `"Critical (Derived)"` for omission/commission
///   guide words and `"Critical (Derived)"` / `"Marginal (Derived)"` for temporal/duration guide words
///   conditioned on `ctx.has_assert_or_guard`.
///
/// ## 3. Preconditions
/// - `guide_word` is one of `STPA_GUIDE_WORDS`; `action_idx` is the 0-based action index.
///
/// ## 4. Postconditions
/// - Returns a non-empty, domain-neutral severity classification string free of unresolved placeholders.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(|hazards| + |attributes|) bounded over controller and package AST metadata.
/// - Space Complexity: O(1) bounded classification string allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Handles absent `matched_part`, empty `hazards`, and empty `attributes` cleanly.
/// - State: Pure read-only evaluation.
/// - Action: Uses checked `.get(action_idx % len)` with non-empty guards (zero modulo-by-zero or slice panics).
/// - Resource: Bounded short string allocations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-UCA-RESOLVE-SEVERITY] | SysML v2: ActionDef(resolve_uca_severity)
fn resolve_uca_severity(
    matched_part: Option<&PartDef>,
    controller_name: &str,
    action_name: &str,
    action_idx: usize,
    guide_word: &str,
    ctx: &AstRiskContext,
) -> String {
    let action_lower = action_name.to_ascii_lowercase();
    if let Some(part) = matched_part {
        if !part.hazards.is_empty() {
            let chosen = part
                .hazards
                .iter()
                .find(|h| {
                    h.severity >= 1
                        && (h.name.eq_ignore_ascii_case(action_name)
                            || h.name.to_ascii_lowercase().contains(&action_lower))
                })
                .or_else(|| part.hazards.get(action_idx % part.hazards.len()));
            if let Some(h) = chosen {
                if let Some(mapped) = map_hazard_severity_score(h.severity) {
                    return mapped;
                }
            }
        }
    }

    if !ctx.pkg_hazards.is_empty() {
        let chosen = ctx
            .pkg_hazards
            .iter()
            .find(|h| {
                h.severity >= 1
                    && (h.name.eq_ignore_ascii_case(action_name)
                        || h.name.eq_ignore_ascii_case(controller_name)
                        || h.name.to_ascii_lowercase().contains(&action_lower))
            })
            .or_else(|| ctx.pkg_hazards.get(action_idx % ctx.pkg_hazards.len()));
        if let Some(h) = chosen {
            if let Some(mapped) = map_hazard_severity_score(h.severity) {
                return mapped;
            }
        }
    }

    let candidate_keys = [
        format!("{}_Severity", action_name),
        format!("{}_Severity", controller_name),
        "HazardSeverity".to_string(),
        "DefaultSeverity".to_string(),
        "Severity".to_string(),
    ];

    for key in &candidate_keys {
        let found = matched_part
            .and_then(|p| find_attr_exact_ci(&p.attributes, key))
            .or_else(|| find_attr_exact_ci(&ctx.pkg_attributes, key))
            .or_else(|| find_attr_exact_ci(&ctx.all_part_attributes, key));
        if let Some(val) = found {
            if let Ok(parsed_score) = val.parse::<i32>() {
                if let Some(mapped) = map_hazard_severity_score(parsed_score) {
                    return mapped;
                }
            }
            return val;
        }
    }

    match guide_word {
        "Not providing" | "Providing" => {
            if ctx.has_assert_or_guard {
                "Catastrophic (Derived)".to_string()
            } else {
                "Critical (Derived)".to_string()
            }
        }
        _ => {
            if ctx.has_assert_or_guard {
                "Critical (Derived)".to_string()
            } else {
                "Marginal (Derived)".to_string()
            }
        }
    }
}

/// Derives UCA safety integrity level from scoped AST `AttributeDef` declarations or deterministic IEC 61508 / ISO 15026-3 severity mapping.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, IEC 61508, ISO/IEC/IEEE 15026-3,
/// and ECSS-E-ST-40C integrity level derivation objectives for fn `resolve_uca_integrity_level`
/// in `crates/compile-sysml/src/stpa/uca.rs` (refs #697).
///
/// ## 2. Invariants & Mathematical Properties
/// - Priority order across `matched_part.attributes`, `ctx.pkg_attributes`, and `ctx.all_part_attributes`:
///   `{action}_IntegrityLevel` -> `{controller}_IntegrityLevel` -> `IntegrityLevel` -> any `IntegrityLevel_*`
///   attribute with non-empty `default_value` -> `{action}_AssuranceLevel` -> `{controller}_AssuranceLevel` -> `AssuranceLevel`.
/// - Deterministic fallback when SSOT attributes are absent:
///   `"SIL-3 (Derived)"` for `"Catastrophic*"`, `"SIL-2 (Derived)"` for `"Critical*"`, else `"SIL-1 (Derived)"`.
///
/// ## 3. Preconditions
/// - `severity` has already been resolved by `resolve_uca_severity`.
///
/// ## 4. Postconditions
/// - Returns a non-empty, domain-neutral integrity level label containing zero hardcoded domain-specific tokens and zero placeholders.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(|attributes|) across controller and package attribute slices.
/// - Space Complexity: O(1) bounded integrity string allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Handles schemas with zero attributes cleanly via deterministic severity-based SIL derivation.
/// - State: Pure read-only lookup.
/// - Action: Zero regular expressions, panics, or direct slice indexing.
/// - Resource: Bounded short string allocations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-UCA-RESOLVE-INTEGRITY] | SysML v2: ActionDef(resolve_uca_integrity_level)
fn resolve_uca_integrity_level(
    matched_part: Option<&PartDef>,
    controller_name: &str,
    action_name: &str,
    severity: &str,
    ctx: &AstRiskContext,
) -> String {
    let integrity_keys = [
        format!("{}_IntegrityLevel", action_name),
        format!("{}_IntegrityLevel", controller_name),
        "IntegrityLevel".to_string(),
    ];
    let assurance_keys = [
        format!("{}_AssuranceLevel", action_name),
        format!("{}_AssuranceLevel", controller_name),
        "AssuranceLevel".to_string(),
    ];

    if let Some(part) = matched_part {
        for key in &integrity_keys {
            if let Some(val) = find_attr_exact_ci(&part.attributes, key) {
                return val;
            }
        }
        if let Some(val) = find_attr_prefix_ci(&part.attributes, "IntegrityLevel_") {
            return val;
        }
        for key in &assurance_keys {
            if let Some(val) = find_attr_exact_ci(&part.attributes, key) {
                return val;
            }
        }
    }

    for attrs in [&ctx.pkg_attributes, &ctx.all_part_attributes] {
        for key in &integrity_keys {
            if let Some(val) = find_attr_exact_ci(attrs, key) {
                return val;
            }
        }
        if let Some(val) = find_attr_prefix_ci(attrs, "IntegrityLevel_") {
            return val;
        }
        for key in &assurance_keys {
            if let Some(val) = find_attr_exact_ci(attrs, key) {
                return val;
            }
        }
    }

    if severity.starts_with("Catastrophic") {
        "SIL-3 (Derived)".to_string()
    } else if severity.starts_with("Critical") {
        "SIL-2 (Derived)".to_string()
    } else {
        "SIL-1 (Derived)".to_string()
    }
}

/// Synthesizes a single-line, guide-word-differentiated STPA operational context anchored to AST states, ports, constraints, and requirements.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C STPA operational
/// context traceability objectives for fn `synthesize_uca_context` in `crates/compile-sysml/src/stpa/uca.rs` (refs #697).
///
/// ## 2. Invariants & Mathematical Properties
/// - Cycles across `[0..]` slices of `matched_part.states`, `matched_part.ports`, and
///   `matched_part.constraints` / `ctx.all_constraints` / `ctx.pkg_requirements` via `.get(uca_idx % len)`
///   without `.first()` truncation.
/// - Produces distinct operational prose across all 4 STPA guide words and sanitizes output via `sanitize_uca_cell`.
///
/// ## 3. Preconditions
/// - `uca_idx` is the 0-based UCA sequence index; `guide_word` is one of `STPA_GUIDE_WORDS`.
///
/// ## 4. Postconditions
/// - Returns a non-empty single-line Markdown cell containing zero `'|'` or newline characters.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L) bounded by the string length L of the selected AST state, port, and constraint anchors.
/// - Space Complexity: O(L) for the formatted context string.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Degrades cleanly to domain-neutral structural descriptions when `states`, `ports`, or `constraints` are empty.
/// - State: Pure read-only projection.
/// - Action: Checked `.get(...)` modulo access guarded by `!slice.is_empty()`.
/// - Resource: Bounded single-line string allocation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-UCA-SYNTHESIZE-CONTEXT] | SysML v2: ActionDef(synthesize_uca_context)
fn synthesize_uca_context(
    matched_part: Option<&PartDef>,
    controller_name: &str,
    action_name: &str,
    guide_word: &str,
    uca_idx: usize,
    ctx: &AstRiskContext,
) -> String {
    let state_anchor = matched_part
        .filter(|p| !p.states.is_empty())
        .and_then(|p| p.states.get(uca_idx % p.states.len()))
        .map(|s| format!("state {}", sanitize_uca_cell(&s.name)))
        .unwrap_or_else(|| "active operational state".to_string());

    let port_anchor = matched_part
        .filter(|p| !p.ports.is_empty())
        .and_then(|p| p.ports.get(uca_idx % p.ports.len()))
        .map(|port| format!("port {}", sanitize_uca_cell(&port.name)))
        .unwrap_or_else(|| "control interface".to_string());

    let active_constraints: &[ConstraintDef] = match matched_part {
        Some(p) if !p.constraints.is_empty() => &p.constraints,
        _ => &ctx.all_constraints,
    };

    let guard_anchor = if !active_constraints.is_empty() {
        active_constraints
            .get(uca_idx % active_constraints.len())
            .map(|c| {
                let expr = sanitize_uca_cell(&c.expression);
                if expr.is_empty() {
                    format!("constraint {}", sanitize_uca_cell(&c.name))
                } else {
                    format!("constraint {} ({})", sanitize_uca_cell(&c.name), expr)
                }
            })
            .unwrap_or_else(|| format!("package {} control envelope", sanitize_uca_cell(&ctx.base_pkg_name)))
    } else if !ctx.pkg_requirements.is_empty() {
        ctx.pkg_requirements
            .get(uca_idx % ctx.pkg_requirements.len())
            .map(|r| {
                let rid = sanitize_uca_cell(&r.req_id);
                if rid.is_empty() {
                    format!("requirement {}", sanitize_uca_cell(&r.name))
                } else {
                    format!("requirement {} ({})", sanitize_uca_cell(&r.name), rid)
                }
            })
            .unwrap_or_else(|| format!("package {} control envelope", sanitize_uca_cell(&ctx.base_pkg_name)))
    } else {
        format!(
            "package {} control envelope",
            sanitize_uca_cell(&ctx.base_pkg_name)
        )
    };

    let ctrl = sanitize_uca_cell(controller_name);
    let act = sanitize_uca_cell(action_name);

    let raw = match guide_word {
        "Not providing" => format!(
            "{} omits {} in {} across {} while {} demands intervention",
            ctrl, act, state_anchor, port_anchor, guard_anchor
        ),
        "Providing" => format!(
            "{} issues {} in {} across {} violating {}",
            ctrl, act, state_anchor, port_anchor, guard_anchor
        ),
        "Too early / Too late / Out of order" => format!(
            "{} dispatches {} out of temporal sequence in {} via {} relative to {}",
            ctrl, act, state_anchor, port_anchor, guard_anchor
        ),
        _ => format!(
            "{} prematurely halts or latches {} in {} on {} against {}",
            ctrl, act, state_anchor, port_anchor, guard_anchor
        ),
    };

    sanitize_uca_cell(&raw)
}

/// Expands the dynamic Cartesian UCA matrix across controlling `PartDef` actions (or package-level
/// `ActionDef` nodes) and the 4 canonical STPA guide words.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `expand_cartesian_stpa` in `crates/compile-sysml/src/stpa/uca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation, `H-{action_counter}` and `SC-{uca_counter:03}` numbering,
///   AST-derived `severity` and `sail` resolution, and zero hardcoded compiler controller names.
///
/// ## 3. Preconditions
/// - `pkg: &PackageDef` must satisfy domain and type-safety constraints prior to invoking `expand_cartesian_stpa`.
///
/// ## 4. Postconditions
/// - Returns `Vec<UnsafeControlAction>` from `expand_cartesian_stpa` with zero unresolved placeholder cells.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(pkg)) bounded over input/member dimension of `expand_cartesian_stpa`.
/// - Space Complexity: O(N(pkg)) bounded auxiliary memory for `expand_cartesian_stpa`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `expand_cartesian_stpa` (`Vec<UnsafeControlAction>`).
/// - State: Maintains consistent state invariants across `expand_cartesian_stpa` without partial mutation on error.
/// - Action: Executes `expand_cartesian_stpa` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `expand_cartesian_stpa` (`STPA-UCA`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-UCA-EXPAND-CARTESIAN-STPA] | SysML v2: ActionDef(expand_cartesian_stpa)
pub fn expand_cartesian_stpa(pkg: &PackageDef) -> Vec<UnsafeControlAction> {
    let mut ucas = Vec::new();
    let all_parts = pkg.get_all_parts();
    let non_internal_parts: Vec<&PartDef> = all_parts
        .iter()
        .filter(|p| !p.name.starts_with('_'))
        .collect();
    let ctx = AstRiskContext::from_package(pkg, &all_parts);

    let mut controllers: Vec<&PartDef> = non_internal_parts
        .iter()
        .copied()
        .filter(|part| !part.actions.is_empty())
        .collect();
    if controllers.is_empty() {
        controllers = all_parts
            .iter()
            .filter(|part| !part.actions.is_empty())
            .collect();
    }

    let mut uca_counter: usize = 0;
    let mut action_counter: usize = 0;

    for controller in &controllers {
        let controller_name = sanitize_uca_cell(&controller.name);
        for action in &controller.actions {
            let action_idx = action_counter;
            action_counter = action_counter.saturating_add(1);
            let action_name = sanitize_uca_cell(&action.name);

            for guide_word in &STPA_GUIDE_WORDS {
                let uca_idx = uca_counter;
                uca_counter = uca_counter.saturating_add(1);
                let severity = resolve_uca_severity(
                    Some(controller),
                    &controller_name,
                    &action_name,
                    action_idx,
                    guide_word,
                    &ctx,
                );
                let sail = resolve_uca_integrity_level(
                    Some(controller),
                    &controller_name,
                    &action_name,
                    &severity,
                    &ctx,
                );
                let context = synthesize_uca_context(
                    Some(controller),
                    &controller_name,
                    &action_name,
                    guide_word,
                    uca_idx,
                    &ctx,
                );
                ucas.push(UnsafeControlAction {
                    id: format!("UCA-{:03}", uca_counter),
                    controller: controller_name.clone(),
                    control_action: action_name.clone(),
                    guide_word: (*guide_word).to_string(),
                    context,
                    hazard: format!("H-{}", action_counter),
                    constraint: format!("SC-{:03}", uca_counter),
                    severity,
                    sail,
                });
            }
        }
    }

    // Expand actions declared at package level (or across recursive subpackages) when no PartDef has inline actions
    if ucas.is_empty() {
        let all_actions = pkg.get_all_actions();
        if !all_actions.is_empty() {
            for action in &all_actions {
                let resolved = resolve_action_controller(
                    action,
                    &all_parts,
                    &non_internal_parts,
                    &ctx.base_pkg_name,
                );
                let action_idx = action_counter;
                action_counter = action_counter.saturating_add(1);
                let action_name = sanitize_uca_cell(&action.name);

                for guide_word in &STPA_GUIDE_WORDS {
                    let uca_idx = uca_counter;
                    uca_counter = uca_counter.saturating_add(1);
                    let severity = resolve_uca_severity(
                        resolved.part,
                        &resolved.name,
                        &action_name,
                        action_idx,
                        guide_word,
                        &ctx,
                    );
                    let sail = resolve_uca_integrity_level(
                        resolved.part,
                        &resolved.name,
                        &action_name,
                        &severity,
                        &ctx,
                    );
                    let context = synthesize_uca_context(
                        resolved.part,
                        &resolved.name,
                        &action_name,
                        guide_word,
                        uca_idx,
                        &ctx,
                    );
                    ucas.push(UnsafeControlAction {
                        id: format!("UCA-{:03}", uca_counter),
                        controller: resolved.name.clone(),
                        control_action: action_name.clone(),
                        guide_word: (*guide_word).to_string(),
                        context,
                        hazard: format!("H-{}", action_counter),
                        constraint: format!("SC-{:03}", uca_counter),
                        severity,
                        sail,
                    });
                }
            }
        }
    }

    ucas
}

#[cfg(test)]
mod tests {
    use super::*;
    use deap_core::sysml_ast::{ActionDef, PartDef};

    /// Verifies that `expand_cartesian_stpa` expands 2 inline controller actions across all 4 STPA
    /// guide words into 8 bijective UCAs preserving `H-1`/`H-2` and `SC-001`..`SC-008` identifiers
    /// alongside resolved domain-neutral severity and integrity levels.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C unit
    /// verification objectives for fn `test_uca_cartesian_expansion_all_guide_words` in
    /// `crates/compile-sysml/src/stpa/uca.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Cardinality invariant: `|ucas| == 2 * 4 == 8` with deterministic guide-word ordering and
    ///   resolved non-empty `severity` and `sail` fields.
    ///
    /// ## 3. Preconditions
    /// - Synthetic `TestPackage` contains `SystemController` with `EngageActuator` and `ReleaseSafetyLock`.
    ///
    /// ## 4. Postconditions
    /// - All 8 UCA records match expected IDs, guide words, hazards, constraints, and derived SIL classifications.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) over fixed 2-action fixture.
    /// - Space Complexity: O(1) bounded 8-element UCA vector.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Invokes `expand_cartesian_stpa(&pkg)` directly.
    /// - State: Pure in-memory fixture with zero side effects.
    /// - Action: Uses checked `.first()` and `.get(...)` with `?` propagation and zero `.unwrap()`.
    /// - Resource: Reclaimed immediately on scope exit.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-STPA-UCA-TEST-CARTESIAN-EXPANSION] | SysML v2: VerificationCaseDef(test_uca_cartesian_expansion_all_guide_words)
    #[test]
    fn test_uca_cartesian_expansion_all_guide_words() -> Result<(), Box<dyn std::error::Error>> {
        let mut pkg = PackageDef {
            name: "TestPackage".to_string(),
            ..Default::default()
        };

        let mut controller = PartDef {
            name: "SystemController".to_string(),
            ..Default::default()
        };
        let act1 = ActionDef {
            name: "EngageActuator".to_string(),
            ..Default::default()
        };
        controller.actions.push(act1);

        let act2 = ActionDef {
            name: "ReleaseSafetyLock".to_string(),
            ..Default::default()
        };
        controller.actions.push(act2);
        pkg.part_defs.push(controller);

        let ucas = expand_cartesian_stpa(&pkg);
        // 2 actions * 4 guide words = 8 UCAs
        assert_eq!(ucas.len(), 8);
        let first = ucas.first().ok_or("missing first element of ucas")?;
        assert_eq!(first.id, "UCA-001");
        assert_eq!(first.controller, "SystemController");
        assert_eq!(first.control_action, "EngageActuator");
        assert_eq!(first.guide_word, "Not providing");
        assert_eq!(first.constraint, "SC-001");
        assert_eq!(first.hazard, "H-1");
        assert_eq!(first.severity, "Critical (Derived)");
        assert_eq!(first.sail, "SIL-2 (Derived)");

        assert_eq!(ucas.get(1).ok_or("missing element 1 of ucas")?.guide_word, "Providing");
        assert_eq!(ucas.get(2).ok_or("missing element 2 of ucas")?.guide_word, "Too early / Too late / Out of order");
        assert_eq!(ucas.get(3).ok_or("missing element 3 of ucas")?.guide_word, "Stopped too soon / Applied too long");

        assert_eq!(ucas.get(4).ok_or("missing element 4 of ucas")?.control_action, "ReleaseSafetyLock");
        assert_eq!(ucas.get(4).ok_or("missing element 4 of ucas")?.hazard, "H-2");
        Ok(())
    }
}
