//! # SysML v2 Universal Ingestion Engine Core
//!
//! ## 1. Safety Intent & Regulatory Scope
//! This library provides the authoritative Ground 0.0 ingestion framework for transforming
//! heterogeneous Level 0 OEM requirements, technical interface control documents (ICDs),
//! and domain specifications into canonical SysML v2 textual Abstract Syntax Tree (AST)
//! models (`deap_core::sysml_ast::PackageDef`).
//!
//! In high-integrity software engineering (DO-178C Level A, ISO 26262 ASIL D, ECSS-E-ST-40C, DO-331),
//! specification ingestion establishes the foundational formal baseline for all downstream verification,
//! formal invariant checks, hazard mitigations, and code generation. Corruptions or unverified
//! omissions at this layer propagate silently throughout the lifecycle.
//!
//! ## 2. Invariants & Mathematical Properties
//! - **Pure Schema-Driven Operation**: All AST derivations occur deterministically from input
//!   schemas without hardcoded domain vocabulary or speculative heuristic assumptions.
//! - **Heterogeneous Specification Support**:
//!   - Level 0 OEM Markdown with YAML frontmatter, BDD acceptance criteria, and GFM tables ([`translators::markdown`]).
//!   - OMG IDL 3.x / 4.x specifications for interface and payload definitions ([`translators::idl`]).
//!   - OpenAPI 3.0 / 3.1 RESTful interface specifications ([`translators::openapi`]).
//!   - IETF RFC 6020 / RFC 7950 YANG data models ([`translators::yang`]).
//! - **Scope Filtering & Negative Invariant Enforcement**:
//!   [`filter_ast_to_target_scope`] strictly prunes phantom entities and projects negative exclusion
//!   assertions (`assert_exclusion_<entity>`) to ensure strict closed-world semantics.
//! - **Cryptographic Fingerprinting & Truncation Gates**:
//!   [`digest`] enforces FIPS 180-4 SHA-256 fingerprinting and subagent context truncation verification.
//!
//! ## 3. Preconditions
//! - Input payloads provided to ingestion translators must be valid UTF-8 text streams.
//! - Target package identifiers and scope boundary filter sets must be well-formed strings.
//!
//! ## 4. Postconditions
//! - Successful ingestion produces a deterministic, structurally valid `deap_core::sysml_ast::PackageDef`.
//! - When scope filtering is applied, out-of-scope definitions and dangling references are pruned and
//!   formal negative exclusion constraints are injected into `PackageDef::constraint_defs`.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - **Time Complexity**: $O(N + P \cdot C)$ where $N$ is the input specification byte length, $P$ is the
//!   number of AST part definitions, and $C$ is the number of cross-referenced items/connections.
//! - **Space Complexity**: $O(N)$ auxiliary heap memory proportional to the parsed AST node count.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Malformed syntax or structural violations in upstream documents return explicit `Err(String)`
//!   diagnostics without panicking or emitting partial corrupt AST packages.
//! - Unrecognized or out-of-scope entities are deterministically excluded via closed-world pruning.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-SYSML-INGEST-LIB], [DO-330-TQL-4-INGEST-LIB]
#![allow(
    clippy::collapsible_if,
    clippy::collapsible_match,
    clippy::collapsible_str_replace,
    clippy::disallowed_types,
    clippy::double_ended_iterator_last,
    clippy::expect_used,
    clippy::if_same_then_else,
    clippy::indexing_slicing,
    clippy::manual_contains,
    clippy::manual_flatten,
    clippy::manual_ignore_case_cmp,
    clippy::manual_pattern_char_comparison,
    clippy::manual_range_contains,
    clippy::manual_strip,
    clippy::manual_unwrap_or,
    clippy::manual_unwrap_or_default,
    clippy::needless_bool,
    clippy::needless_bool_assign,
    clippy::needless_lifetimes,
    clippy::panic,
    clippy::question_mark,
    clippy::result_unit_err,
    clippy::trim_split_whitespace,
    clippy::unnecessary_map_or,
    clippy::unnecessary_unwrap,
    clippy::unwrap_used,
    clippy::vec_init_then_push
)]

pub mod detectors;
pub mod digest;
pub mod table;
pub mod translators;

use std::collections::HashSet;
use deap_core::sysml_ast::{ConstraintDef, PackageDef, PartDef};

/// Fail-safe process exit code classification for the `ingest-sysml` CLI engine.
///
/// # Summary
/// Partitions `ingest-sysml` execution outcomes into deterministic exit statuses:
/// `Clean` (`0`), `ParseOrIoError` (`1`), and `ScopeOrCryptoError` (`2`).
///
/// # Preconditions
/// - Constructed by `run_ingestion_with_code` upon completing or halting an ingestion pass.
///
/// # Postconditions
/// - Maps bijectively to OS process exit codes `0`, `1`, and `2` via [`IngestExitCode::code`].
///
/// # Invariants
/// - `IngestExitCode::Clean as i32 == 0`.
/// - `IngestExitCode::ParseOrIoError as i32 == 1`.
/// - `IngestExitCode::ScopeOrCryptoError as i32 == 2`.
///
/// # Failure Modes
/// - Exhaustive `#[repr(i32)]`-compatible unit enum; cannot represent unmapped status codes.
///
/// # Side Effects
/// - Pure value enumeration with zero I/O or heap allocation.
///
/// # Determinism
/// - Deterministic constant discriminant mapping across all target architectures.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C fail-safe
/// tool qualification requirements for `IngestSysml::SourceCodeAstIngestion`, distinguishing
/// lexical/syntactic/I/O failures (`1`) from architectural scope, negative-invariant, and
/// post-write cryptographic verification failures (`2`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Total Partition: Every ingestion outcome resolves to exactly one variant in `{0, 1, 2}`.
///
/// ## 3. Preconditions
/// - None; value type usable across library and CLI boundaries.
///
/// ## 4. Postconditions
/// - Provides strongly typed error categorization prior to `std::process::exit`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - **Time Complexity**: $O(1)$.
/// - **Space Complexity**: $O(1)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Infallible enum representation with zero runtime failure states.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-EXIT-CODE/IngestExitCode] | SysML v2: part def SourceCodeAstIngestion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngestExitCode {
    /// Clean ingestion, scope verification, and cryptographic digest verification (`0`).
    Clean = 0,
    /// Lexical, syntactic, format classification, or filesystem I/O error (`1`).
    ParseOrIoError = 1,
    /// Architectural scope starvation, negative-invariant violation, or SHA-256 verification failure (`2`).
    ScopeOrCryptoError = 2,
}

impl IngestExitCode {
    /// Converts this [`IngestExitCode`] into its canonical `i32` OS process exit status.
    ///
    /// # Summary
    /// Returns `0` for `Clean`, `1` for `ParseOrIoError`, and `2` for `ScopeOrCryptoError`.
    ///
    /// # Preconditions
    /// - `self` is a valid [`IngestExitCode`] variant.
    ///
    /// # Postconditions
    /// - Returns an `i32` in the closed interval `[0, 2]`.
    ///
    /// # Invariants
    /// - Referentially transparent `const fn` conversion preserving enum discriminant values.
    ///
    /// # Failure Modes
    /// - Infallible total conversion; never panics or overflows.
    ///
    /// # Side Effects
    /// - None.
    ///
    /// # Determinism
    /// - Compile-time and runtime deterministic evaluation.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Enforces unambiguous exit code reporting to orchestrators and CI gates under DO-330 TQL-4.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - `self.code() == self as i32` for all variants of [`IngestExitCode`].
    ///
    /// ## 3. Preconditions
    /// - `self: IngestExitCode` is initialized.
    ///
    /// ## 4. Postconditions
    /// - Returns `0`, `1`, or `2`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - **Time Complexity**: $O(1)$.
    /// - **Space Complexity**: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Total `const fn` with zero failure paths.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-INGEST-EXIT-CODE/code] | SysML v2: part def SourceCodeAstIngestion
    pub const fn code(self) -> i32 {
        self as i32
    }
}

/// Translates a single Markdown requirement document into a canonical SysML v2 `PackageDef`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Provides the primary convenience entrypoint for ingesting Level 0 OEM Markdown documents,
/// invoking [`translators::markdown::MarkdownTranslator::translate`] in a bounded execution context.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves total deterministic mapping from Markdown AST headings, YAML frontmatter, and tables
///   into `PackageDef` nodes without speculative domain inference.
///
/// ## 3. Preconditions
/// - `content` is a valid UTF-8 string slice containing Markdown text.
/// - `file_name` is a non-empty fallback package identifier.
///
/// ## 4. Postconditions
/// - Returns `Ok(PackageDef)` containing all extracted SysML v2 AST nodes on success, or an
///   explanatory `Err(String)` diagnostic on parsing failure.
///
/// ## 5. Algorithmic Complexity Bounds
/// - **Time Complexity**: $O(N)$ where $N$ is the byte length of `content`.
/// - **Space Complexity**: $O(N)$ to hold the synthesized `PackageDef` AST.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Propagates translator diagnostic errors cleanly if `content` violates structural expectations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-CORE/translate_markdown]
pub fn translate_markdown(content: &str, file_name: &str) -> Result<PackageDef, String> {
    translators::markdown::MarkdownTranslator::new().translate(content, file_name)
}

/// Translates a single YANG model into a canonical SysML v2 `PackageDef`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Provides the primary convenience entrypoint for ingesting RFC 6020/7950 YANG models,
/// invoking [`translators::yang::YangTranslator::translate`] in a bounded execution context.
///
/// ## 2. Invariants & Mathematical Properties
/// - Deterministic projection of YANG containers, groupings, typedefs, and leaves into canonical
///   SysML v2 `PartDef`, `ItemDef`, and `AttributeDef` structures.
///
/// ## 3. Preconditions
/// - `content` is a valid UTF-8 string slice containing YANG specification text.
/// - `file_name` is a non-empty fallback package identifier.
///
/// ## 4. Postconditions
/// - Returns `Ok(PackageDef)` on success, or an explanatory `Err(String)` on parsing failure.
///
/// ## 5. Algorithmic Complexity Bounds
/// - **Time Complexity**: $O(N)$ where $N$ is the byte length of `content`.
/// - **Space Complexity**: $O(N)$ proportional to the number of YANG statements and AST elements.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Unbalanced braces or malformed YANG constructs yield deterministic `Err(String)` diagnostics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-CORE/translate_yang]
pub fn translate_yang(content: &str, file_name: &str) -> Result<PackageDef, String> {
    translators::yang::YangTranslator::new().translate(content, file_name)
}

/// Translates a single Rust or Dart source file into a canonical SysML v2 `PackageDef`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 source-code AST ingestion (`translate_code`).
/// Invokes [`translators::code::CodeTranslator::translate`] to parse source declarations
/// without regular expressions and lower them into a `PackageDef`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Maps classes/structs to `PartDef`, fields to `AttributeDef`, callbacks to `PortDef`,
///   and functions/methods to `ActionDef`.
///
/// ## 3. Preconditions
/// - `content: &str` is valid UTF-8 Rust or Dart source code.
/// - `file_name: &str` is the source filename or package identifier.
///
/// ## 4. Postconditions
/// - Returns `Ok(PackageDef)` on clean parse and lowering, or `Err(String)` on syntax error.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `content.len()`.
/// - Space Complexity: O(N) proportional to extracted AST elements.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Propagates syntax/delimiter errors with `line:col` coordinates.
/// - State: Pure functional delegation.
/// - Action: Never panics on malformed source code.
/// - Resource: Bounded by single file buffer size.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-CORE/translate_code]
pub fn translate_code(content: &str, file_name: &str) -> Result<PackageDef, String> {
    translators::code::CodeTranslator::new().translate(content, file_name)
}

/// Filters AST nodes to target metamodel scope and projects negative invariants.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Enforces a strict closed-world assumption by pruning extraneous external reference entities
/// that are not part of the active system boundary. Furthermore, projects formal negative
/// exclusion invariants (`assert_exclusion_<name>`) to mathematically prove that phantom
/// entities do not exist in the ingested model.
///
/// ## 2. Invariants & Mathematical Properties
/// - **Closed-World Pruning**: Every surviving `PartDef` in `pkg.part_defs` (and nested sub-packages)
///   is guaranteed to be a member of `allowed_parts` when a whitelist is supplied.
/// - **Idempotent Constraint Injection**: Duplicate negative invariant entries do not produce
///   duplicate `ConstraintDef` assertions in `pkg.constraint_defs`.
///
/// ## 3. Preconditions
/// - `pkg` is a mutable reference to an initialized `PackageDef`.
/// - `allowed_parts` is an optional set of whitelisted part names. If `None`, no parts are pruned.
/// - `negative_invariants` is an optional slice of entity names whose absence must be formally asserted.
///
/// ## 4. Postconditions
/// - Any `PartDef` whose name is not in `allowed_parts` is recursively pruned from `pkg.part_defs`.
/// - Cascading references in items, capabilities, hazards, risks, and connections are cleaned.
/// - For each entry in `negative_invariants`, a formal `ConstraintDef` is appended asserting `!exists(<entity>)`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - **Time Complexity**: $O(P \cdot I + H + R + K + M \cdot C)$ where $P$ is part count, $I$ is item count,
///   $H$ is hazard count, $R$ is risk count, $K$ is connection count, $M$ is negative invariant count,
///   and $C$ is existing constraint count.
/// - **Space Complexity**: $O(H + D)$ where $H$ is surviving hazard set size and $D$ is package recursion depth.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Empty `allowed_parts` prunes all parts while preserving structural validity of `pkg`.
/// - Non-alphanumeric characters in `negative_invariants` are sanitized into valid identifier tokens.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-SCOPE/filter_ast_to_target_scope]
pub fn filter_ast_to_target_scope(
    pkg: &mut PackageDef,
    allowed_parts: Option<&HashSet<String>>,
    negative_invariants: Option<&[String]>,
) {
    // Step 1: Prune parts outside allowed scope if a whitelist is supplied
    if let Some(allowed) = allowed_parts {
        /// Recursively filters a `PartDef` and its nested child parts against `allowed`.
        ///
        /// ## 1. Safety Intent & Regulatory Scope
        /// Ensures hierarchical containment integrity during scope pruning so no disallowed child
        /// part remains attached to an allowed parent part.
        ///
        /// ## 2. Invariants & Mathematical Properties
        /// - Returns `true` if and only if `part.name` is present in `allowed`.
        /// - All retained descendants of `part.parts` also satisfy membership in `allowed`.
        ///
        /// ## 3. Preconditions
        /// - `part` is a valid mutable `PartDef` reference.
        /// - `allowed` is an initialized `HashSet<String>` of permitted part names.
        ///
        /// ## 4. Postconditions
        /// - If `part.name` is in `allowed`, `part.parts` is pruned in-place to retain only allowed descendants.
        ///
        /// ## 5. Algorithmic Complexity Bounds
        /// - **Time Complexity**: $O(T)$ where $T$ is the number of nodes in the `part` subtree.
        /// - **Space Complexity**: $O(D)$ call-stack frames where $D$ is the maximum part nesting depth.
        ///
        /// ## 6. Failure Modes & Fault Tolerance
        /// - Immediately short-circuits and returns `false` when the root `part.name` is excluded.
        ///
        /// ## 7. Realization & Traceability
        /// Realises: [REQ-SYSML-INGEST-SCOPE/filter_part]
        fn filter_part(part: &mut PartDef, allowed: &HashSet<String>) -> bool {
            if !allowed.contains(&part.name) {
                return false;
            }
            part.parts.retain_mut(|sub| filter_part(sub, allowed));
            true
        }

        // Retain only whitelisted top-level part definitions
        pkg.part_defs.retain_mut(|p| filter_part(p, allowed));

        if !allowed.is_empty() {
            // Prune item defs not associated with surviving parts
            pkg.item_defs.retain(|item| {
                allowed.contains(&item.name)
                    || pkg.part_defs.iter().any(|p| p.name.contains(&item.name) || item.name.contains(&p.name))
            });

            // Prune capabilities whose subsystem is not allowed
            pkg.capability_defs.retain(|c| {
                c.subsystem.as_ref().map_or(true, |sub| allowed.contains(sub)) || allowed.contains(&c.name)
            });

            // Prune hazards referencing excluded parts
            pkg.hazard_defs.retain(|h| {
                h.part_ref.as_ref().map_or(true, |pr| allowed.contains(pr)) || allowed.contains(&h.name)
            });

            // Prune risks referencing pruned hazards
            let surviving_hazards: HashSet<String> = pkg.hazard_defs.iter().map(|h| h.name.clone()).collect();
            pkg.risk_defs.retain(|r| {
                r.hazard_ref.as_ref().map_or(true, |hr| surviving_hazards.contains(hr)) || allowed.contains(&r.name)
            });

            // Prune connection defs where source or target part is excluded
            pkg.connection_defs.retain(|conn| {
                let src_part = conn.source_port.split('.').next();
                let tgt_part = conn.target_port.split('.').next();
                if let Some(sp) = src_part {
                    if conn.source_port.contains('.') && !allowed.contains(sp) {
                        return false;
                    }
                }
                if let Some(tp) = tgt_part {
                    if conn.target_port.contains('.') && !allowed.contains(tp) {
                        return false;
                    }
                }
                true
            });
        }

        // Recursively filter child sub-packages
        for sub_pkg in &mut pkg.packages {
            filter_ast_to_target_scope(sub_pkg, Some(allowed), None);
        }
    }

    // Step 2: Project negative exclusion invariants into formal constraint assertions
    if let Some(neg_list) = negative_invariants {
        for inv in neg_list {
            let clean = inv
                .replace(|c: char| !c.is_ascii_alphanumeric() && c != '_', "_")
                .to_ascii_lowercase();
            let clean_name = clean.trim_matches('_');
            let constraint_name = format!("assert_exclusion_{}", clean_name);
            if !pkg.constraint_defs.iter().any(|c| c.name == constraint_name) {
                pkg.constraint_defs.push(ConstraintDef {
                    name: constraint_name,
                    expression: format!("!exists({})", inv),
                    parameters: Vec::new(),
                    is_assertion: true,
                    doc: Some(format!("Negative invariant asserting exclusion of phantom entity: {}", inv)),
                    pre_conditions: Vec::new(),
                    post_conditions: Vec::new(),
                });
            }
        }
    }
}

/// Checks whether a `PackageDef` (or any nested sub-package) contains at least one `PartDef`.
///
/// # Summary
/// Performs an iterative work-stack traversal over `pkg` and `pkg.packages` to determine whether
/// at least one `PartDef` survived scope filtering.
///
/// # Preconditions
/// - `pkg` is a valid `PackageDef` reference.
///
/// # Postconditions
/// - Returns `true` iff `pkg.part_defs` or any nested `sub_pkg.part_defs` is non-empty.
///
/// # Invariants
/// - Bounded iterative traversal with zero recursion, zero `.unwrap()`, and zero slice indexing.
///
/// # Failure Modes
/// - Infallible read-only predicate; returns `false` on empty packages.
///
/// # Side Effects
/// - Allocates a transient traversal stack `Vec<&PackageDef>`; does not mutate `pkg`.
///
/// # Determinism
/// - Referentially transparent predicate over `pkg`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Supports DO-178C Level A / DO-330 TQL-4 architectural scope verification (`IngestExitCode::ScopeOrCryptoError`)
/// by detecting when `--allowed-parts` matches zero parts in the extracted model.
///
/// ## 2. Invariants & Mathematical Properties
/// - $\text{package\_has\_any\_part}(P) \iff \exists Q \in \text{packages}(P) : Q.\text{part\_defs} \neq \emptyset$.
///
/// ## 3. Preconditions
/// - `pkg: &PackageDef` is an initialized package tree.
///
/// ## 4. Postconditions
/// - Returns `bool` indicating whether at least one `PartDef` exists in the package hierarchy.
///
/// ## 5. Algorithmic Complexity Bounds
/// - **Time Complexity**: $O(K)$ where $K$ is the number of nested packages in `pkg`.
/// - **Space Complexity**: $O(K)$ for the explicit package work stack.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Total function; never panics or fails.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-SCOPE/package_has_any_part] | SysML v2: part def SourceCodeAstIngestion
fn package_has_any_part(pkg: &PackageDef) -> bool {
    let mut stack: Vec<&PackageDef> = vec![pkg];
    while let Some(curr) = stack.pop() {
        if !curr.part_defs.is_empty() {
            return true;
        }
        for sub in &curr.packages {
            stack.push(sub);
        }
    }
    false
}

/// Checks whether any declared entity in `pkg` (or nested packages/parts) matches `entity_name`.
///
/// # Summary
/// Traverses `pkg`, nested `pkg.packages`, and nested `PartDef::parts` to test whether `entity_name`
/// exists as a declared `PartDef`, `ItemDef`, `ActionDef`, `PortDef`, or `StateDef`.
///
/// # Preconditions
/// - `pkg` is a valid `PackageDef` reference; `entity_name` is a non-empty trimmed identifier.
///
/// # Postconditions
/// - Returns `true` iff any `PartDef`, `ItemDef`, `ActionDef`, `PortDef`, or `StateDef` in `pkg`
///   has `name == entity_name`.
///
/// # Invariants
/// - Iterative stack-based traversal with zero unbounded recursion, zero `.unwrap()`, and zero slice indexing.
///
/// # Failure Modes
/// - Infallible read-only predicate; returns `false` when `entity_name` is absent.
///
/// # Side Effects
/// - Allocates transient traversal stacks `Vec<&PackageDef>` and `Vec<&PartDef>`; does not mutate `pkg`.
///
/// # Determinism
/// - Referentially transparent over `pkg` and `entity_name`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Enforces closed-world negative exclusion invariants (`--negative-invariants`) under DO-178C Level A
/// and DO-330 TQL-4 by detecting forbidden phantom or out-of-boundary entities remaining in the AST.
///
/// ## 2. Invariants & Mathematical Properties
/// - Inspects package-level `item_defs`, `action_defs`, `port_defs`, `state_defs`, and all reachable
///   `PartDef` nodes along with their nested `actions`, `ports`, and `states`.
///
/// ## 3. Preconditions
/// - `pkg: &PackageDef` is an initialized package tree; `entity_name: &str` is the forbidden identifier.
///
/// ## 4. Postconditions
/// - Returns `true` immediately upon encountering a matching entity declaration, or `false` after exhaustive scan.
///
/// ## 5. Algorithmic Complexity Bounds
/// - **Time Complexity**: $O(K + P + I + A + R + S)$ across all packages $K$, parts $P$, items $I$,
///   actions $A$, ports $R$, and states $S$.
/// - **Space Complexity**: $O(K + P)$ for the explicit traversal stacks.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Total function; never panics or fails.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-SCOPE/package_contains_entity] | SysML v2: part def SourceCodeAstIngestion
fn package_contains_entity(pkg: &PackageDef, entity_name: &str) -> bool {
    let mut pkg_stack: Vec<&PackageDef> = vec![pkg];
    while let Some(curr_pkg) = pkg_stack.pop() {
        if curr_pkg.item_defs.iter().any(|i| i.name == entity_name)
            || curr_pkg.action_defs.iter().any(|a| a.name == entity_name)
            || curr_pkg.port_defs.iter().any(|port| port.name == entity_name)
            || curr_pkg.state_defs.iter().any(|s| s.name == entity_name)
        {
            return true;
        }
        let mut part_stack: Vec<&PartDef> = curr_pkg.part_defs.iter().collect();
        while let Some(part) = part_stack.pop() {
            if part.name == entity_name
                || part.actions.iter().any(|a| a.name == entity_name)
                || part.ports.iter().any(|port| port.name == entity_name)
                || part.states.iter().any(|s| s.name == entity_name)
            {
                return true;
            }
            for child in &part.parts {
                part_stack.push(child);
            }
        }
        for sub_pkg in &curr_pkg.packages {
            pkg_stack.push(sub_pkg);
        }
    }
    false
}

/// Verifies post-filtering architectural scope survival and negative exclusion invariants on `pkg`.
///
/// # Summary
/// Validates that a non-empty `allowed_parts` filter leaves at least one surviving `PartDef` in `pkg`,
/// and that no forbidden entity listed in `negative_invariants` exists in `pkg` as a `PartDef`,
/// `ItemDef`, `ActionDef`, `PortDef`, or `StateDef`.
///
/// # Preconditions
/// - `pkg` has already been processed by [`filter_ast_to_target_scope`].
/// - `allowed_parts` and `negative_invariants` reflect the caller's architectural boundary configuration.
///
/// # Postconditions
/// - Returns `Ok(())` iff all scope survival and negative exclusion invariants hold.
/// - Returns `Err(String)` describing the exact scope starvation or negative-invariant violation otherwise.
///
/// # Invariants
/// - Read-only verification gate; never mutates `pkg` or panics.
///
/// # Failure Modes
/// - Returns `Err` when `allowed_parts` is `Some(allowed)` with `!allowed.is_empty()` and zero `PartDef`
///   nodes survived in `pkg`.
/// - Returns `Err` when any non-empty trimmed entry in `negative_invariants` matches a declared entity in `pkg`.
///
/// # Side Effects
/// - None; pure verification function.
///
/// # Determinism
/// - Deterministic validation outcome for identical `(pkg, allowed_parts, negative_invariants)` inputs.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Enforces the DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D architectural boundary contract
/// for `IngestSysml::SourceCodeAstIngestion`, driving exit code `2` (`IngestExitCode::ScopeOrCryptoError`)
/// whenever scope filters starve the model or forbidden entities violate negative exclusion invariants.
///
/// ## 2. Invariants & Mathematical Properties
/// - **Non-Empty Scope Survival**: $\text{allowed} \neq \emptyset \implies \exists p \in \text{Parts}(pkg)$.
/// - **Disjoint Negative Invariants**: $\forall \text{inv} \in \text{neg\_list} : \neg \text{exists}(pkg, \text{inv})$.
///
/// ## 3. Preconditions
/// - `pkg: &PackageDef` is the post-filtered SysML v2 package tree.
/// - `allowed_parts: Option<&HashSet<String>>` is the optional part whitelist.
/// - `negative_invariants: Option<&[String]>` is the optional forbidden entity list.
///
/// ## 4. Postconditions
/// - Returns `Ok(())` on full compliance or `Err(String)` mapped to `IngestExitCode::ScopeOrCryptoError`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - **Time Complexity**: $O(M \cdot (K + P + I + A + R + S))$ where $M$ is `negative_invariants` length.
/// - **Space Complexity**: $O(K + P)$ for bounded iterative traversal stacks.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Emits explicit diagnostic messages naming the offending package and entity/filter set.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-SCOPE/verify_scope_and_negative_invariants] | SysML v2: part def SourceCodeAstIngestion
pub fn verify_scope_and_negative_invariants(
    pkg: &PackageDef,
    allowed_parts: Option<&HashSet<String>>,
    negative_invariants: Option<&[String]>,
) -> Result<(), String> {
    if let Some(allowed) = allowed_parts {
        if !allowed.is_empty() && !package_has_any_part(pkg) {
            return Err(format!(
                "Architectural scope verification failed: none of the --allowed-parts {:?} matched any extracted PartDef in package '{}'",
                allowed, pkg.name
            ));
        }
    }

    if let Some(neg_list) = negative_invariants {
        for raw_inv in neg_list {
            let inv = raw_inv.trim();
            if !inv.is_empty() && package_contains_entity(pkg, inv) {
                return Err(format!(
                    "Architectural negative-invariant violation: forbidden entity '{}' exists in package '{}'",
                    inv, pkg.name
                ));
            }
        }
    }

    Ok(())
}

/// Collects all declared `PartDef` names across a `PackageDef` hierarchy into `out`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, and Issue #472 architectural realization scoping
/// (`collect_declared_part_names`). Extracts the authoritative set of architectural `part def`
/// identifiers from a domain reference schema so code-ingestion passes can prune unannotated
/// internal implementation helpers automatically.
///
/// ## 2. Invariants & Mathematical Properties
/// - Traverses `pkg.part_defs`, nested `part.parts`, and nested `pkg.packages` using bounded
///   explicit work stacks without unbounded recursion.
/// - Monotonically inserts non-empty `PartDef::name` strings into `out`.
///
/// ## 3. Preconditions
/// - `pkg: &PackageDef` is a parsed domain reference schema package tree.
/// - `out: &mut HashSet<String>` is an initialized hash set receiving discovered part names.
///
/// ## 4. Postconditions
/// - `out` contains every `PartDef::name` reachable from `pkg`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - **Time Complexity**: $O(P + K)$ where $P$ is total `PartDef` count and $K$ is nested `PackageDef` count.
/// - **Space Complexity**: $O(P + K)$ for the traversal work stacks and hash set entries.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Empty packages or part trees leave `out` unmodified without error or panic.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-SCOPE/collect_declared_part_names], [REQ-ISSUE-472-SCHEMA-SCOPED-CODE-INGESTION]
pub fn collect_declared_part_names(pkg: &PackageDef, out: &mut HashSet<String>) {
    let mut pkg_stack: Vec<&PackageDef> = vec![pkg];
    while let Some(curr_pkg) = pkg_stack.pop() {
        let mut part_stack: Vec<&PartDef> = curr_pkg.part_defs.iter().collect();
        while let Some(curr_part) = part_stack.pop() {
            if !curr_part.name.trim().is_empty() {
                out.insert(curr_part.name.clone());
            }
            for child in &curr_part.parts {
                part_stack.push(child);
            }
        }
        for sub_pkg in &curr_pkg.packages {
            pkg_stack.push(sub_pkg);
        }
    }
}

/// Filters a code-extracted `PackageDef` to retain only architectural `part def` components
/// present in `schema_parts` or source constructs carrying explicit `Realises:` bindings.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, ISO 26262 ASIL D, and Issue #472 realization boundary control
/// (`filter_ast_to_realized_or_allowed`). Prevents internal UI widgets, `State<T>` classes,
/// scratch buffers, and helper enums lacking architectural realization tags from polluting
/// system-level `part def` namespaces and breaking Check 17 Pillar 7 FMECA closure.
///
/// ## 2. Invariants & Mathematical Properties
/// - A `PartDef` $P$ is retained iff $P.\text{name} \in \text{schema\_parts} \lor P.\text{satisfied\_requirements} \neq \emptyset$
///   (or $P$ encloses a retained child `PartDef`).
/// - Applies identically across nested `pkg.packages`.
///
/// ## 3. Preconditions
/// - `pkg: &mut PackageDef` is a mutable code-extracted SysML v2 package.
/// - `schema_parts: &HashSet<String>` contains architectural `part def` names from the reference schema.
///
/// ## 4. Postconditions
/// - Unannotated implementation helper `PartDef` nodes absent from `schema_parts` are pruned in-place.
/// - All architectural `PartDef` nodes and explicitly realized (`!satisfied_requirements.is_empty()`)
///   `PartDef` nodes are preserved.
///
/// ## 5. Algorithmic Complexity Bounds
/// - **Time Complexity**: $O(P)$ where $P$ is total `PartDef` node count across `pkg`.
/// - **Space Complexity**: $O(D)$ where $D$ is maximum nested package/part depth.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Safely retains realized parts even when `schema_parts` is empty.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-SCOPE/filter_ast_to_realized_or_allowed], [REQ-ISSUE-472-SCHEMA-SCOPED-CODE-INGESTION]
pub fn filter_ast_to_realized_or_allowed(pkg: &mut PackageDef, schema_parts: &HashSet<String>) {
    /// Recursively prunes `part.parts` and determines whether `part` is retained under realization scoping.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Preserves hierarchical part containment during realization filtering so a parent container
    /// is retained whenever either the parent itself or any nested child realizes a requirement
    /// or matches an architectural `part def` in `schema_parts`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Evaluates child `part.parts` bottom-up via `retain_mut` before deciding parent retention.
    ///
    /// ## 3. Preconditions
    /// - `part: &mut PartDef` is a candidate code-extracted part node.
    /// - `schema_parts: &HashSet<String>` is the reference architectural part name set.
    ///
    /// ## 4. Postconditions
    /// - Returns `true` iff `part` or a surviving child is in `schema_parts` or has non-empty `satisfied_requirements`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - **Time Complexity**: $O(T)$ where $T$ is the subtree size rooted at `part`.
    /// - **Space Complexity**: $O(H)$ where $H$ is the part nesting height.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Pure in-place vector retention with zero panics.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-INGEST-SCOPE/retain_realized_part]
    fn retain_realized_part(part: &mut PartDef, schema_parts: &HashSet<String>) -> bool {
        part.parts
            .retain_mut(|sub| retain_realized_part(sub, schema_parts));
        let has_realises_doc = part
            .doc
            .as_deref()
            .is_some_and(|d| d.contains("Realises:") && !d.contains("Realises: [UNMAPPED]"));
        schema_parts.contains(&part.name)
            || !part.satisfied_requirements.is_empty()
            || has_realises_doc
            || !part.parts.is_empty()
    }

    pkg.part_defs
        .retain_mut(|p| retain_realized_part(p, schema_parts));

    for sub_pkg in &mut pkg.packages {
        filter_ast_to_realized_or_allowed(sub_pkg, schema_parts);
    }
    pkg.packages.retain(|sub_pkg| {
        !sub_pkg.part_defs.is_empty()
            || !sub_pkg.action_defs.is_empty()
            || !sub_pkg.requirement_defs.is_empty()
            || !sub_pkg.packages.is_empty()
    });
}

/// Loads or parses and caches an ingested `SysmlModel` using `IncrementalAstCache`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Provides SHA-256 content-addressed incremental AST caching for Level 0 SysML v2 ingestion
/// under DO-330 TQL-4 determinism and reproducibility mandates.
///
/// ## 2. Invariants & Mathematical Properties
/// - Cache hit occurs if and only if the persisted `.pipeline/cache/sysml_ast_cache.json` entry
///   matches the SHA-256 digest of `sysml_source`.
/// - Returns `(SysmlModel, true)` on a verified cache hit and `(SysmlModel, false)` on a fresh
///   parse-and-persist cycle.
///
/// ## 3. Preconditions
/// - `repo_root` is a writable workspace directory; `sysml_source` is valid UTF-8 SysML v2 text.
///
/// ## 4. Postconditions
/// - Returns `Ok((SysmlModel, bool))` with the parsed or cached `SysmlModel` and cache-hit flag,
///   or `Err(String)` if parsing or cache persistence fails.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time complexity: $O(N)$ where $N$ is `sysml_source.len()` for hashing and serialization.
/// - Space complexity: $O(N)$ proportional to AST node count.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Corrupted cache files are automatically invalidated and recomputed by `IncrementalAstCache`;
///   filesystem write errors propagate as `Err(String)`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-CACHE-MODEL]
pub fn cache_ingested_sysml_model(
    repo_root: &std::path::Path,
    sysml_source: &str,
) -> Result<(deap_core::sysml_ast::SysmlModel, bool), String> {
    let cache = deap_core::sysml_ast::IncrementalAstCache::new(repo_root);
    cache.load_or_parse(sysml_source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_translate_markdown_api() {
        let content = "# Package_0\n\n## Classifier_Alpha\nSynthetic element.";
        let res = translate_markdown(content, "Package_0");
        assert!(res.is_ok());
        let pkg = res.unwrap();
        assert_eq!(pkg.name, "Package_0");
        assert_eq!(pkg.part_defs.len(), 1);
        assert_eq!(pkg.part_defs[0].name, "Classifier_Alpha");
    }

    #[test]
    fn test_filter_ast_to_target_scope_synthetic() {
        let mut pkg = PackageDef {
            name: "Package_0".to_string(),
            part_defs: vec![
                PartDef {
                    name: "Classifier_Alpha".to_string(),
                    ..Default::default()
                },
                PartDef {
                    name: "Classifier_Extraneous".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let mut allowed = HashSet::new();
        allowed.insert("Classifier_Alpha".to_string());
        let neg = vec!["Phantom_Entity".to_string()];

        filter_ast_to_target_scope(&mut pkg, Some(&allowed), Some(&neg));

        assert_eq!(pkg.part_defs.len(), 1);
        assert_eq!(pkg.part_defs[0].name, "Classifier_Alpha");
        assert_eq!(pkg.constraint_defs.len(), 1);
        assert_eq!(pkg.constraint_defs[0].name, "assert_exclusion_phantom_entity");
        assert_eq!(pkg.constraint_defs[0].expression, "!exists(Phantom_Entity)");
    }

    struct TempTestDir(std::path::PathBuf);

    impl TempTestDir {
        fn new(label: &str) -> Self {
            use std::sync::atomic::{AtomicUsize, Ordering};
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            let id = COUNTER.fetch_add(1, Ordering::SeqCst);
            let dir = std::env::temp_dir().join(format!(
                "deap_ingest_test_{}_{}_{}",
                label,
                std::process::id(),
                id
            ));
            std::fs::remove_dir_all(&dir).unwrap_or(());
            std::fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
    }

    impl Drop for TempTestDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn test_batch_translate_multiple_files_synthetic() {
        use std::fs;
        let content1 = "# Package_0\n\n## Classifier_Alpha\nAlpha doc.";
        let content2 = "# Package_0\n\n## Classifier_Beta\nBeta doc.";

        let tmp = TempTestDir::new("batch_synthetic");
        let f1 = tmp.0.join("spec1.md");
        let f2 = tmp.0.join("spec2.md");
        fs::write(&f1, content1).unwrap();
        fs::write(&f2, content2).unwrap();

        let translator = translators::markdown::MarkdownTranslator::new();
        let file_paths = vec![f1.to_string_lossy().to_string(), f2.to_string_lossy().to_string()];
        let pkg = translator.translate_files(&file_paths, "Consolidated_Package").expect("Failed batch translation");

        assert_eq!(pkg.name, "Consolidated_Package");
        let p_names: Vec<&str> = pkg.part_defs.iter().map(|p| p.name.as_str()).collect();
        assert!(p_names.contains(&"Classifier_Alpha"));
        assert!(p_names.contains(&"Classifier_Beta"));
    }

    #[test]
    fn test_batch_translators_fail_closed_on_empty_and_mixed_missing_inputs() {
        use std::fs;
        let tmp = TempTestDir::new("batch_fail_closed");

        let valid_md_path = tmp.0.join("valid.md");
        fs::write(&valid_md_path, "# Package_0\n\n## Classifier_Alpha\nAlpha doc.").unwrap();
        let valid_md = valid_md_path.to_string_lossy().to_string();
        let missing_md = tmp.0.join("missing.md").to_string_lossy().to_string();

        let md_translator = translators::markdown::MarkdownTranslator::new();
        assert!(
            md_translator.translate_files(&[], "Pkg").is_err(),
            "MarkdownTranslator::translate_files must fail closed on empty slice"
        );
        assert!(
            md_translator
                .translate_files(&[valid_md, missing_md], "Pkg")
                .is_err(),
            "MarkdownTranslator::translate_files must fail closed on mixed valid + missing batch"
        );

        let valid_yang_path = tmp.0.join("valid.yang");
        fs::write(
            &valid_yang_path,
            "module valid-mod { namespace \"urn:test:valid\"; prefix vm; container root { leaf id { type string; } } }",
        )
        .unwrap();
        let valid_yang = valid_yang_path.to_string_lossy().to_string();
        let missing_yang = tmp.0.join("missing.yang").to_string_lossy().to_string();

        let yang_translator = translators::yang::YangTranslator::new();
        assert!(
            yang_translator.translate_files(&[], "Pkg").is_err(),
            "YangTranslator::translate_files must fail closed on empty slice"
        );
        assert!(
            yang_translator
                .translate_files(std::slice::from_ref(&missing_yang), "Pkg")
                .is_err(),
            "YangTranslator::translate_files must fail closed on single missing file"
        );
        assert!(
            yang_translator
                .translate_files(&[valid_yang, missing_yang], "Pkg")
                .is_err(),
            "YangTranslator::translate_files must fail closed on mixed valid + missing batch"
        );
    }

    #[test]
    fn test_ingest_sysml_lib_7_section_rustdoc() {
        let src = include_str!("lib.rs");
        assert!(src.contains("//! ## 1. Safety Intent & Regulatory Scope"));
        assert!(src.contains("//! ## 2. Invariants & Mathematical Properties"));
        assert!(src.contains("//! ## 3. Preconditions"));
        assert!(src.contains("//! ## 4. Postconditions"));
        assert!(src.contains("//! ## 5. Algorithmic Complexity Bounds"));
        assert!(src.contains("//! ## 6. Failure Modes & Fault Tolerance"));
        assert!(src.contains("//! ## 7. Realization & Traceability"));
        assert!(src.contains("//! Realises: [REQ-SYSML-INGEST-LIB], [DO-330-TQL-4-INGEST-LIB]"));
    }

    #[test]
    fn test_cache_ingested_sysml_model_hit_and_invalidation() {
        let tmp_dir = std::env::temp_dir().join(format!(
            "deap_ingest_cache_test_{}",
            std::process::id()
        ));
        std::fs::remove_dir_all(&tmp_dir).unwrap_or(());
        std::fs::create_dir_all(&tmp_dir).expect("temp dir creation");

        let sysml_v1 = "package IngestPkg { part def SensorUnit { attribute rate_hz : Real; } }";
        let (model_first, hit_first) =
            cache_ingested_sysml_model(&tmp_dir, sysml_v1).expect("first cache call");
        assert!(!hit_first);
        assert_eq!(model_first.parts.len(), 1);

        let (model_second, hit_second) =
            cache_ingested_sysml_model(&tmp_dir, sysml_v1).expect("second cache call");
        assert!(hit_second);
        assert_eq!(model_second, model_first);

        let sysml_v2 = "package IngestPkg { part def SensorUnit { attribute rate_hz : Real; } part def ActuatorUnit {} }";
        let (model_third, hit_third) =
            cache_ingested_sysml_model(&tmp_dir, sysml_v2).expect("third cache call after mutation");
        assert!(!hit_third);
        assert_eq!(model_third.parts.len(), 2);

        std::fs::remove_dir_all(&tmp_dir).unwrap_or(());
    }
}

