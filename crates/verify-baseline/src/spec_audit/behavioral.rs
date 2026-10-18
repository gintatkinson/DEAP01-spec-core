//! Native Rust Behavioral Triggers Validator conforming to DO-178C Level A and ISO 26262 ASIL D.
//!
//! Strongly-typed validation of `rules/behavioral_triggers.json` requirements against workspace
//! SysML schemas and specification documents (User Stories and Use Cases).
//!
//! Key capabilities:
//! - Ingestion and parsing of behavioral triggers configuration.
//! - Determination of active triggers based on workspace SysML schema symbols (via `SysmlSymbolIndex` or symbol sets).
//! - Enforcement of coverage: active trigger nodes must be referenced by at least one specification of `target_type`.
//! - Verification of trigger rules:
//!   - Required Mermaid diagram blocks of specified type (e.g. `sequenceDiagram`), containing at least one required match term.
//!   - Required body terms (and optional secondary terms) in markdown prose outside code blocks.
//!
//! Strictly zero regex, zero unwrap/expect/panic in non-test code, zero Unicode em dashes (uses ASCII -- exclusively).

use crate::spec_audit::markdown_ast::{parse_markdown, MarkdownDoc};
use crate::spec_audit::uml_validator::SysmlSymbolIndex;
use crate::spec_audit::{SpecAuditError, SpecAuditFinding};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

/// Individual rule within a behavioral trigger specifying target document type and verification constraints.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `BehavioralRule` in `crates/verify-baseline/src/spec_audit/behavioral.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `BehavioralRule` across member set `[target_type: String, requires_mermaid_block: Option<String>, match_terms_in_mermaid: Vec<String>, match_terms_in_body: Vec<String>, match_terms_in_body_secondary: Vec<String>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `BehavioralRule` members (`target_type: String, requires_mermaid_block: Option<String>, match_terms_in_mermaid: Vec<String>, match_terms_in_body: Vec<String>, match_terms_in_body_secondary: Vec<String>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `BehavioralRule` representation for struct `BehavioralRule`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `BehavioralRule`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `BehavioralRule`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `BehavioralRule` (`BehavioralRule`).
/// - State: Maintains consistent state invariants across `BehavioralRule` without partial mutation on error.
/// - Action: Executes `BehavioralRule` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `BehavioralRule` (`SPEC-AUDIT-BEHAVIORAL`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-BEHAVIORALRULE] | SysML v2: PartDef(BehavioralRule)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BehavioralRule {
    /// Target specification type ("user-story" or "use-case").
    pub target_type: String,

    /// Optional Mermaid diagram type required (e.g. "sequenceDiagram", "stateDiagram-v2").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires_mermaid_block: Option<String>,

    /// Terms that must appear in at least one required Mermaid block.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub match_terms_in_mermaid: Vec<String>,

    /// Terms that must appear in the specification prose / body (outside code blocks).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub match_terms_in_body: Vec<String>,

    /// Secondary terms that must appear in the specification prose / body (if configured).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub match_terms_in_body_secondary: Vec<String>,

    /// Human-readable diagnostic error message explaining why the trigger is required.
    pub error_message: String,
}

/// Behavioral trigger definition loaded from `rules/behavioral_triggers.json`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `BehavioralTrigger` in `crates/verify-baseline/src/spec_audit/behavioral.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `BehavioralTrigger` across member set `[name: String, trigger_nodes: Vec<String>, rules: Vec<BehavioralRule>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `BehavioralTrigger` members (`name: String, trigger_nodes: Vec<String>, rules: Vec<BehavioralRule>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `BehavioralTrigger` representation for struct `BehavioralTrigger`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(3)) bounded over input/member dimension of `BehavioralTrigger`.
/// - Space Complexity: O(M(3)) bounded auxiliary memory for `BehavioralTrigger`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `BehavioralTrigger` (`BehavioralTrigger`).
/// - State: Maintains consistent state invariants across `BehavioralTrigger` without partial mutation on error.
/// - Action: Executes `BehavioralTrigger` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `BehavioralTrigger` (`SPEC-AUDIT-BEHAVIORAL`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-BEHAVIORALTRIGGER] | SysML v2: PartDef(BehavioralTrigger)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BehavioralTrigger {
    /// Human-readable trigger name.
    pub name: String,

    /// Schema node names that arm this trigger (e.g. "stream-data", "sensor-stream").
    pub trigger_nodes: Vec<String>,

    /// Behavioral rules enforced when any trigger node is present in the schema.
    pub rules: Vec<BehavioralRule>,
}

impl BehavioralTrigger {
    /// Determines which trigger nodes are active given a pre-indexed SysML symbol table.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `BehavioralTrigger::active_nodes` in `crates/verify-baseline/src/spec_audit/behavioral.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `BehavioralTrigger::active_nodes` (declaration #3) within `SPEC-AUDIT-BEHAVIORAL`.
    ///
    /// ## 3. Preconditions
    /// - `index: &SysmlSymbolIndex` must satisfy domain and type-safety constraints prior to invoking `BehavioralTrigger::active_nodes`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Vec<String>` from `BehavioralTrigger::active_nodes` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(index)) bounded over input/member dimension of `BehavioralTrigger::active_nodes`.
    /// - Space Complexity: O(N(index)) bounded auxiliary memory for `BehavioralTrigger::active_nodes`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `BehavioralTrigger::active_nodes` (`Vec<String>`).
    /// - State: Maintains consistent state invariants across `BehavioralTrigger::active_nodes` without partial mutation on error.
    /// - Action: Executes `BehavioralTrigger::active_nodes` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `BehavioralTrigger::active_nodes` (`SPEC-AUDIT-BEHAVIORAL`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-BEHAVIORALTRIGGER-ACTIVE-NODES] | SysML v2: ActionDef(BehavioralTrigger::active_nodes)
    pub fn active_nodes(&self, index: &SysmlSymbolIndex) -> Vec<String> {
        let normalized_symbols = collect_normalized_symbols(index);
        self.active_nodes_from_normalized_set(&normalized_symbols)
    }

    /// Determines which trigger nodes are active given an already normalized set of schema symbol names.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `BehavioralTrigger::active_nodes_from_normalized_set` in `crates/verify-baseline/src/spec_audit/behavioral.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `BehavioralTrigger::active_nodes_from_normalized_set` (declaration #4) within `SPEC-AUDIT-BEHAVIORAL`.
    ///
    /// ## 3. Preconditions
    /// - `normalized_symbols: &HashSet<String>` must satisfy domain and type-safety constraints prior to invoking `BehavioralTrigger::active_nodes_from_normalized_set`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Vec<String>` from `BehavioralTrigger::active_nodes_from_normalized_set` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(normalized_symbols)) bounded over input/member dimension of `BehavioralTrigger::active_nodes_from_normalized_set`.
    /// - Space Complexity: O(N(normalized_symbols)) bounded auxiliary memory for `BehavioralTrigger::active_nodes_from_normalized_set`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `BehavioralTrigger::active_nodes_from_normalized_set` (`Vec<String>`).
    /// - State: Maintains consistent state invariants across `BehavioralTrigger::active_nodes_from_normalized_set` without partial mutation on error.
    /// - Action: Executes `BehavioralTrigger::active_nodes_from_normalized_set` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `BehavioralTrigger::active_nodes_from_normalized_set` (`SPEC-AUDIT-BEHAVIORAL`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-BEHAVIORALTRIGGER-ACTIVE-NODES-FROM-NORMALIZED-SET] | SysML v2: ActionDef(BehavioralTrigger::active_nodes_from_normalized_set)
    pub fn active_nodes_from_normalized_set(&self, normalized_symbols: &HashSet<String>) -> Vec<String> {
        self.trigger_nodes
            .iter()
            .filter(|node| normalized_symbols.contains(&normalize_symbol_name(node)))
            .cloned()
            .collect()
    }
}

/// In-memory representation of an audited specification document.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `AuditedSpec` in `crates/verify-baseline/src/spec_audit/behavioral.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `AuditedSpec` across member set `[rel_path: String, spec_type: String, doc: MarkdownDoc, content: String]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `AuditedSpec` members (`rel_path: String, spec_type: String, doc: MarkdownDoc, content: String`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `AuditedSpec` representation for struct `AuditedSpec`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(4)) bounded over input/member dimension of `AuditedSpec`.
/// - Space Complexity: O(M(4)) bounded auxiliary memory for `AuditedSpec`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `AuditedSpec` (`AuditedSpec`).
/// - State: Maintains consistent state invariants across `AuditedSpec` without partial mutation on error.
/// - Action: Executes `AuditedSpec` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `AuditedSpec` (`SPEC-AUDIT-BEHAVIORAL`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-AUDITEDSPEC] | SysML v2: PartDef(AuditedSpec)
#[derive(Debug, Clone, PartialEq)]
pub struct AuditedSpec {
    /// Relative path of the specification file (e.g. "docs/user-stories/us-01.md").
    pub rel_path: String,
    /// Document specification type (e.g. "user-story" or "use-case").
    pub spec_type: String,
    /// Parsed Markdown document AST.
    pub doc: MarkdownDoc,
    /// Raw full UTF-8 text content of the specification.
    pub content: String,
}

impl AuditedSpec {
    /// Constructs an `AuditedSpec` by parsing UTF-8 content in memory.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `AuditedSpec::from_content` in `crates/verify-baseline/src/spec_audit/behavioral.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `AuditedSpec::from_content` (declaration #6) within `SPEC-AUDIT-BEHAVIORAL`.
    ///
    /// ## 3. Preconditions
    /// - `rel_path: impl Into<String>` must satisfy domain and type-safety constraints prior to invoking `AuditedSpec::from_content`.
    /// - `spec_type: impl Into<String>` must satisfy domain and type-safety constraints prior to invoking `AuditedSpec::from_content`.
    /// - `content: impl Into<String>` must satisfy domain and type-safety constraints prior to invoking `AuditedSpec::from_content`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Result<Self, SpecAuditError>` from `AuditedSpec::from_content` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(rel_path)) bounded over input/member dimension of `AuditedSpec::from_content`.
    /// - Space Complexity: O(N(rel_path)) bounded auxiliary memory for `AuditedSpec::from_content`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `AuditedSpec::from_content` (`Result<Self, SpecAuditError>`).
    /// - State: Maintains consistent state invariants across `AuditedSpec::from_content` without partial mutation on error.
    /// - Action: Executes `AuditedSpec::from_content` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `AuditedSpec::from_content` (`SPEC-AUDIT-BEHAVIORAL`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-AUDITEDSPEC-FROM-CONTENT] | SysML v2: ActionDef(AuditedSpec::from_content)
    pub fn from_content(
        rel_path: impl Into<String>,
        spec_type: impl Into<String>,
        content: impl Into<String>,
    ) -> Result<Self, SpecAuditError> {
        let rel_path = rel_path.into();
        let spec_type = spec_type.into();
        let content = content.into();
        let doc = parse_markdown(&content)?;
        Ok(Self {
            rel_path,
            spec_type,
            doc,
            content,
        })
    }

    /// Loads and parses an audited specification from a file on disk.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `AuditedSpec::from_file` in `crates/verify-baseline/src/spec_audit/behavioral.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `AuditedSpec::from_file` (declaration #7) within `SPEC-AUDIT-BEHAVIORAL`.
    ///
    /// ## 3. Preconditions
    /// - `path: &Path` must satisfy domain and type-safety constraints prior to invoking `AuditedSpec::from_file`.
    /// - `rel_path: impl Into<String>` must satisfy domain and type-safety constraints prior to invoking `AuditedSpec::from_file`.
    /// - `spec_type: impl Into<String>` must satisfy domain and type-safety constraints prior to invoking `AuditedSpec::from_file`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Result<Self, SpecAuditError>` from `AuditedSpec::from_file` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(path)) bounded over input/member dimension of `AuditedSpec::from_file`.
    /// - Space Complexity: O(N(path)) bounded auxiliary memory for `AuditedSpec::from_file`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `AuditedSpec::from_file` (`Result<Self, SpecAuditError>`).
    /// - State: Maintains consistent state invariants across `AuditedSpec::from_file` without partial mutation on error.
    /// - Action: Executes `AuditedSpec::from_file` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `AuditedSpec::from_file` (`SPEC-AUDIT-BEHAVIORAL`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-AUDITEDSPEC-FROM-FILE] | SysML v2: ActionDef(AuditedSpec::from_file)
    pub fn from_file(
        path: &Path,
        rel_path: impl Into<String>,
        spec_type: impl Into<String>,
    ) -> Result<Self, SpecAuditError> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            SpecAuditError::IoError(format!(
                "Failed to read specification file at '{}': {}",
                path.display(),
                e
            ))
        })?;
        Self::from_content(rel_path, spec_type, content)
    }
}

/// Normalizes a symbol or schema node name into canonical lowercased comparison form.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `normalize_symbol_name` in `crates/verify-baseline/src/spec_audit/behavioral.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `normalize_symbol_name` (declaration #8) within `SPEC-AUDIT-BEHAVIORAL`.
///
/// ## 3. Preconditions
/// - `name: &str` must satisfy domain and type-safety constraints prior to invoking `normalize_symbol_name`.
///
/// ## 4. Postconditions
/// - Returns `String` from `normalize_symbol_name` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(name)) bounded over input/member dimension of `normalize_symbol_name`.
/// - Space Complexity: O(N(name)) bounded auxiliary memory for `normalize_symbol_name`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `normalize_symbol_name` (`String`).
/// - State: Maintains consistent state invariants across `normalize_symbol_name` without partial mutation on error.
/// - Action: Executes `normalize_symbol_name` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `normalize_symbol_name` (`SPEC-AUDIT-BEHAVIORAL`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-NORMALIZE-SYMBOL-NAME] | SysML v2: ActionDef(normalize_symbol_name)
pub fn normalize_symbol_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for ch in name.chars() {
        if ch != '-' && ch != '_' && !ch.is_whitespace() {
            out.extend(ch.to_lowercase());
        }
    }
    out
}

/// Collects all declared symbol identifiers across a `SysmlSymbolIndex` into a normalized lookup set.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `collect_normalized_symbols` in `crates/verify-baseline/src/spec_audit/behavioral.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `collect_normalized_symbols` (declaration #9) within `SPEC-AUDIT-BEHAVIORAL`.
///
/// ## 3. Preconditions
/// - `index: &SysmlSymbolIndex` must satisfy domain and type-safety constraints prior to invoking `collect_normalized_symbols`.
///
/// ## 4. Postconditions
/// - Returns `HashSet<String>` from `collect_normalized_symbols` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(index)) bounded over input/member dimension of `collect_normalized_symbols`.
/// - Space Complexity: O(N(index)) bounded auxiliary memory for `collect_normalized_symbols`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `collect_normalized_symbols` (`HashSet<String>`).
/// - State: Maintains consistent state invariants across `collect_normalized_symbols` without partial mutation on error.
/// - Action: Executes `collect_normalized_symbols` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `collect_normalized_symbols` (`SPEC-AUDIT-BEHAVIORAL`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-COLLECT-NORMALIZED-SYMBOLS] | SysML v2: ActionDef(collect_normalized_symbols)
pub fn collect_normalized_symbols(index: &SysmlSymbolIndex) -> HashSet<String> {
    let mut set = HashSet::with_capacity(
        index.part_names.len()
            + index.port_names.len()
            + index.action_names.len()
            + index.state_names.len()
            + index.attribute_names.len()
            + index.classifiers.len(),
    );
    for name in &index.part_names {
        set.insert(normalize_symbol_name(name));
    }
    for name in &index.port_names {
        set.insert(normalize_symbol_name(name));
    }
    for name in &index.action_names {
        set.insert(normalize_symbol_name(name));
    }
    for name in &index.state_names {
        set.insert(normalize_symbol_name(name));
    }
    for name in &index.attribute_names {
        set.insert(normalize_symbol_name(name));
    }
    for name in index.classifiers.keys() {
        set.insert(normalize_symbol_name(name));
    }
    set
}

/// Extracts all lines of a Markdown document that reside outside fenced code blocks.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `extract_body_outside_code_blocks` in `crates/verify-baseline/src/spec_audit/behavioral.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `extract_body_outside_code_blocks` (declaration #10) within `SPEC-AUDIT-BEHAVIORAL`.
///
/// ## 3. Preconditions
/// - `content: &str` must satisfy domain and type-safety constraints prior to invoking `extract_body_outside_code_blocks`.
///
/// ## 4. Postconditions
/// - Returns `String` from `extract_body_outside_code_blocks` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(content)) bounded over input/member dimension of `extract_body_outside_code_blocks`.
/// - Space Complexity: O(N(content)) bounded auxiliary memory for `extract_body_outside_code_blocks`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `extract_body_outside_code_blocks` (`String`).
/// - State: Maintains consistent state invariants across `extract_body_outside_code_blocks` without partial mutation on error.
/// - Action: Executes `extract_body_outside_code_blocks` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `extract_body_outside_code_blocks` (`SPEC-AUDIT-BEHAVIORAL`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-EXTRACT-BODY-OUTSIDE-CODE-BLOCKS] | SysML v2: ActionDef(extract_body_outside_code_blocks)
pub fn extract_body_outside_code_blocks(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut in_code_block = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code_block = !in_code_block;
            continue;
        }

        if !in_code_block {
            out.push_str(line);
            out.push('\n');
        }
    }

    out
}

/// Checks whether `content` contains `target` as a distinct token with word boundaries.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `contains_token_with_word_boundary` in `crates/verify-baseline/src/spec_audit/behavioral.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `contains_token_with_word_boundary` (declaration #11) within `SPEC-AUDIT-BEHAVIORAL`.
///
/// ## 3. Preconditions
/// - `content: &str` must satisfy domain and type-safety constraints prior to invoking `contains_token_with_word_boundary`.
/// - `target: &str` must satisfy domain and type-safety constraints prior to invoking `contains_token_with_word_boundary`.
///
/// ## 4. Postconditions
/// - Returns `bool` from `contains_token_with_word_boundary` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(content)) bounded over input/member dimension of `contains_token_with_word_boundary`.
/// - Space Complexity: O(N(content)) bounded auxiliary memory for `contains_token_with_word_boundary`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `contains_token_with_word_boundary` (`bool`).
/// - State: Maintains consistent state invariants across `contains_token_with_word_boundary` without partial mutation on error.
/// - Action: Executes `contains_token_with_word_boundary` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `contains_token_with_word_boundary` (`SPEC-AUDIT-BEHAVIORAL`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-CONTAINS-TOKEN-WITH-WORD-BOUNDARY] | SysML v2: ActionDef(contains_token_with_word_boundary)
pub fn contains_token_with_word_boundary(content: &str, target: &str) -> bool {
    if target.is_empty() || content.is_empty() {
        return false;
    }
    let lower_content = content.to_ascii_lowercase();
    let lower_target = target.to_ascii_lowercase();

    let mut start = 0;
    while let Some(slice) = lower_content.get(start..) {
        let Some(pos) = slice.find(&lower_target) else {
            break;
        };
        let abs_pos = start + pos;
        let end_pos = abs_pos + lower_target.len();

        let before_ok = if abs_pos == 0 {
            true
        } else {
            let prev_byte = lower_content.as_bytes()[abs_pos - 1];
            !prev_byte.is_ascii_alphanumeric() && prev_byte != b'_' && prev_byte != b'-'
        };

        let after_ok = if end_pos >= lower_content.len() {
            true
        } else {
            let next_byte = lower_content.as_bytes()[end_pos];
            !next_byte.is_ascii_alphanumeric() && next_byte != b'_' && next_byte != b'-'
        };

        if before_ok && after_ok {
            return true;
        }

        let step = lower_content
            .get(abs_pos..)
            .and_then(|s| s.chars().next())
            .map(|c| c.len_utf8())
            .unwrap_or(1);
        start = abs_pos + step;
        if start >= lower_content.len() {
            break;
        }
    }

    false
}

/// Checks whether a specification references a trigger node by name.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `spec_references_node` in `crates/verify-baseline/src/spec_audit/behavioral.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `spec_references_node` (declaration #12) within `SPEC-AUDIT-BEHAVIORAL`.
///
/// ## 3. Preconditions
/// - `content: &str` must satisfy domain and type-safety constraints prior to invoking `spec_references_node`.
/// - `node: &str` must satisfy domain and type-safety constraints prior to invoking `spec_references_node`.
///
/// ## 4. Postconditions
/// - Returns `bool` from `spec_references_node` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(content)) bounded over input/member dimension of `spec_references_node`.
/// - Space Complexity: O(N(content)) bounded auxiliary memory for `spec_references_node`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `spec_references_node` (`bool`).
/// - State: Maintains consistent state invariants across `spec_references_node` without partial mutation on error.
/// - Action: Executes `spec_references_node` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `spec_references_node` (`SPEC-AUDIT-BEHAVIORAL`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-SPEC-REFERENCES-NODE] | SysML v2: ActionDef(spec_references_node)
pub fn spec_references_node(content: &str, node: &str) -> bool {
    if node.is_empty() || content.is_empty() {
        return false;
    }

    // 1. Direct word-boundary match (e.g. "stream-data")
    if contains_token_with_word_boundary(content, node) {
        return true;
    }

    // 2. Underscore-separated match (e.g. "stream_data")
    if node.contains('-') {
        let with_underscores = node.replace('-', "_");
        if contains_token_with_word_boundary(content, &with_underscores) {
            return true;
        }
    }

    // 3. Space-separated match (e.g. "stream data")
    if node.contains('-') || node.contains('_') {
        let with_spaces = node.replace(['-', '_'], " ");
        if contains_token_with_word_boundary(content, &with_spaces) {
            return true;
        }
    }

    // 4. Collapsed alphanumeric match (e.g. "streamdata" matching "StreamData")
    let collapsed: String = node.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    if !collapsed.is_empty() && collapsed != node && contains_token_with_word_boundary(content, &collapsed) {
        return true;
    }

    false
}

/// Checks whether a specification document matches the target specification type.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `matches_target_type` in `crates/verify-baseline/src/spec_audit/behavioral.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `matches_target_type` (declaration #13) within `SPEC-AUDIT-BEHAVIORAL`.
///
/// ## 3. Preconditions
/// - `spec_type: &str` must satisfy domain and type-safety constraints prior to invoking `matches_target_type`.
/// - `rel_path: &str` must satisfy domain and type-safety constraints prior to invoking `matches_target_type`.
/// - `target_type: &str` must satisfy domain and type-safety constraints prior to invoking `matches_target_type`.
///
/// ## 4. Postconditions
/// - Returns `bool` from `matches_target_type` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(spec_type)) bounded over input/member dimension of `matches_target_type`.
/// - Space Complexity: O(N(spec_type)) bounded auxiliary memory for `matches_target_type`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `matches_target_type` (`bool`).
/// - State: Maintains consistent state invariants across `matches_target_type` without partial mutation on error.
/// - Action: Executes `matches_target_type` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `matches_target_type` (`SPEC-AUDIT-BEHAVIORAL`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-MATCHES-TARGET-TYPE] | SysML v2: ActionDef(matches_target_type)
pub fn matches_target_type(spec_type: &str, rel_path: &str, target_type: &str) -> bool {
    let norm_target = target_type.to_lowercase().replace('_', "-");
    let norm_spec = spec_type.to_lowercase().replace('_', "-");
    let norm_path = rel_path.to_lowercase().replace('_', "-");

    if norm_target == "user-story" {
        norm_spec == "user-story"
            || norm_spec == "user-stories"
            || norm_spec == "story"
            || norm_path.contains("/user-stories/")
            || norm_path.contains("/user-story-")
            || norm_path.contains("/us-")
    } else if norm_target == "use-case" {
        norm_spec == "use-case"
            || norm_spec == "use-cases"
            || norm_path.contains("/use-cases/")
            || norm_path.contains("/use-case-")
            || norm_path.contains("/uc-")
    } else {
        norm_spec == norm_target || norm_path.contains(&norm_target)
    }
}

/// Checks if Markdown body text contains at least one of the specified match terms.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `body_contains_any_term` in `crates/verify-baseline/src/spec_audit/behavioral.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `body_contains_any_term` (declaration #14) within `SPEC-AUDIT-BEHAVIORAL`.
///
/// ## 3. Preconditions
/// - `body: &str` must satisfy domain and type-safety constraints prior to invoking `body_contains_any_term`.
/// - `terms: &[String]` must satisfy domain and type-safety constraints prior to invoking `body_contains_any_term`.
///
/// ## 4. Postconditions
/// - Returns `bool` from `body_contains_any_term` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(body)) bounded over input/member dimension of `body_contains_any_term`.
/// - Space Complexity: O(N(body)) bounded auxiliary memory for `body_contains_any_term`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `body_contains_any_term` (`bool`).
/// - State: Maintains consistent state invariants across `body_contains_any_term` without partial mutation on error.
/// - Action: Executes `body_contains_any_term` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `body_contains_any_term` (`SPEC-AUDIT-BEHAVIORAL`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-BODY-CONTAINS-ANY-TERM] | SysML v2: ActionDef(body_contains_any_term)
fn body_contains_any_term(body: &str, terms: &[String]) -> bool {
    if terms.is_empty() {
        return true;
    }
    let lower_body = body.to_lowercase();
    for term in terms {
        let lower_term = term.to_lowercase();
        if lower_body.contains(&lower_term) {
            return true;
        }
        if term.contains('-') {
            let with_space = lower_term.replace('-', " ");
            if lower_body.contains(&with_space) {
                return true;
            }
            let with_underscore = lower_term.replace('-', "_");
            if lower_body.contains(&with_underscore) {
                return true;
            }
        }
        if term.contains('_') {
            let with_space = lower_term.replace('_', " ");
            if lower_body.contains(&with_space) {
                return true;
            }
            let with_hyphen = lower_term.replace('_', "-");
            if lower_body.contains(&with_hyphen) {
                return true;
            }
        }
    }
    false
}

/// Parses behavioral triggers JSON configuration into strongly-typed structures.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `parse_behavioral_triggers` in `crates/verify-baseline/src/spec_audit/behavioral.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `parse_behavioral_triggers` (declaration #15) within `SPEC-AUDIT-BEHAVIORAL`.
///
/// ## 3. Preconditions
/// - `json_content: &str` must satisfy domain and type-safety constraints prior to invoking `parse_behavioral_triggers`.
///
/// ## 4. Postconditions
/// - Returns `Result<Vec<BehavioralTrigger>, SpecAuditError>` from `parse_behavioral_triggers` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(json_content)) bounded over input/member dimension of `parse_behavioral_triggers`.
/// - Space Complexity: O(N(json_content)) bounded auxiliary memory for `parse_behavioral_triggers`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `parse_behavioral_triggers` (`Result<Vec<BehavioralTrigger>, SpecAuditError>`).
/// - State: Maintains consistent state invariants across `parse_behavioral_triggers` without partial mutation on error.
/// - Action: Executes `parse_behavioral_triggers` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `parse_behavioral_triggers` (`SPEC-AUDIT-BEHAVIORAL`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-PARSE-BEHAVIORAL-TRIGGERS] | SysML v2: ActionDef(parse_behavioral_triggers)
pub fn parse_behavioral_triggers(json_content: &str) -> Result<Vec<BehavioralTrigger>, SpecAuditError> {
    serde_json::from_str(json_content)
        .map_err(|e| SpecAuditError::RuleError(format!("Failed to parse behavioral triggers JSON: {}", e)))
}

/// Loads and parses behavioral triggers JSON configuration from a file path.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `load_behavioral_triggers` in `crates/verify-baseline/src/spec_audit/behavioral.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `load_behavioral_triggers` (declaration #16) within `SPEC-AUDIT-BEHAVIORAL`.
///
/// ## 3. Preconditions
/// - `path: &Path` must satisfy domain and type-safety constraints prior to invoking `load_behavioral_triggers`.
///
/// ## 4. Postconditions
/// - Returns `Result<Vec<BehavioralTrigger>, SpecAuditError>` from `load_behavioral_triggers` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(path)) bounded over input/member dimension of `load_behavioral_triggers`.
/// - Space Complexity: O(N(path)) bounded auxiliary memory for `load_behavioral_triggers`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `load_behavioral_triggers` (`Result<Vec<BehavioralTrigger>, SpecAuditError>`).
/// - State: Maintains consistent state invariants across `load_behavioral_triggers` without partial mutation on error.
/// - Action: Executes `load_behavioral_triggers` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `load_behavioral_triggers` (`SPEC-AUDIT-BEHAVIORAL`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-LOAD-BEHAVIORAL-TRIGGERS] | SysML v2: ActionDef(load_behavioral_triggers)
pub fn load_behavioral_triggers(path: &Path) -> Result<Vec<BehavioralTrigger>, SpecAuditError> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        SpecAuditError::IoError(format!(
            "Failed to read behavioral triggers file at '{}': {}",
            path.display(),
            e
        ))
    })?;
    parse_behavioral_triggers(&content)
}

/// Validates specifications against armed behavioral triggers.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `validate_behavioral_triggers` in `crates/verify-baseline/src/spec_audit/behavioral.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `validate_behavioral_triggers` (declaration #17) within `SPEC-AUDIT-BEHAVIORAL`.
///
/// ## 3. Preconditions
/// - `triggers: &[BehavioralTrigger]` must satisfy domain and type-safety constraints prior to invoking `validate_behavioral_triggers`.
/// - `symbol_index: &SysmlSymbolIndex` must satisfy domain and type-safety constraints prior to invoking `validate_behavioral_triggers`.
/// - `specs: &[AuditedSpec]` must satisfy domain and type-safety constraints prior to invoking `validate_behavioral_triggers`.
///
/// ## 4. Postconditions
/// - Returns `Result<Vec<SpecAuditFinding>, SpecAuditError>` from `validate_behavioral_triggers` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(triggers)) bounded over input/member dimension of `validate_behavioral_triggers`.
/// - Space Complexity: O(N(triggers)) bounded auxiliary memory for `validate_behavioral_triggers`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `validate_behavioral_triggers` (`Result<Vec<SpecAuditFinding>, SpecAuditError>`).
/// - State: Maintains consistent state invariants across `validate_behavioral_triggers` without partial mutation on error.
/// - Action: Executes `validate_behavioral_triggers` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `validate_behavioral_triggers` (`SPEC-AUDIT-BEHAVIORAL`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-VALIDATE-BEHAVIORAL-TRIGGERS] | SysML v2: ActionDef(validate_behavioral_triggers)
pub fn validate_behavioral_triggers(
    triggers: &[BehavioralTrigger],
    symbol_index: &SysmlSymbolIndex,
    specs: &[AuditedSpec],
) -> Result<Vec<SpecAuditFinding>, SpecAuditError> {
    let mut findings = Vec::new();
    let normalized_symbols = collect_normalized_symbols(symbol_index);

    for trigger in triggers {
        let active_nodes = trigger.active_nodes_from_normalized_set(&normalized_symbols);
        if active_nodes.is_empty() {
            // An inactive trigger imposes nothing
            continue;
        }

        for rule in &trigger.rules {
            // Filter candidate specifications matching target_type
            let candidate_specs: Vec<&AuditedSpec> = specs
                .iter()
                .filter(|s| matches_target_type(&s.spec_type, &s.rel_path, &rule.target_type))
                .collect();

            for active_node in &active_nodes {
                // Find all candidate specs referencing this active trigger node
                let referencing_specs: Vec<&AuditedSpec> = candidate_specs
                    .iter()
                    .copied()
                    .filter(|s| spec_references_node(&s.content, active_node))
                    .collect();

                if referencing_specs.is_empty() {
                    findings.push(SpecAuditFinding::new(
                        "behavioral-trigger-node-must-be-covered-by-a-specification",
                        format!(
                            "Validation failed: No {} specifications found referencing active trigger node '{}'. {}",
                            rule.target_type, active_node, rule.error_message
                        ),
                        active_node.clone(),
                        None,
                    ));
                    continue;
                }

                // Check that every specification referencing this active trigger node satisfies the rule
                for spec in referencing_specs {
                    let mut file_valid = true;

                    // 1. Mermaid diagram block check
                    if let Some(ref req_mermaid) = rule.requires_mermaid_block {
                        let matching_blocks: Vec<_> = spec
                            .doc
                            .mermaid_blocks
                            .iter()
                            .filter(|b| {
                                b.header.eq_ignore_ascii_case(req_mermaid)
                                    || b.raw.lines().any(|l| l.trim().starts_with(req_mermaid.as_str()))
                            })
                            .collect();

                        if matching_blocks.is_empty() {
                            file_valid = false;
                        } else if !rule.match_terms_in_mermaid.is_empty() {
                            let term_found = matching_blocks.iter().any(|block| {
                                rule.match_terms_in_mermaid.iter().any(|term| {
                                    block.raw.contains(term.as_str())
                                        || block.raw.to_lowercase().contains(&term.to_lowercase())
                                })
                            });
                            if !term_found {
                                file_valid = false;
                            }
                        }
                    }

                    // 2. Markdown prose body check (outside code blocks)
                    if file_valid && !rule.match_terms_in_body.is_empty() {
                        let body_text = extract_body_outside_code_blocks(&spec.content);
                        if !body_contains_any_term(&body_text, &rule.match_terms_in_body) {
                            file_valid = false;
                        }
                    }

                    // 3. Secondary Markdown prose body check (if configured)
                    if file_valid && !rule.match_terms_in_body_secondary.is_empty() {
                        let body_text = extract_body_outside_code_blocks(&spec.content);
                        if !body_contains_any_term(&body_text, &rule.match_terms_in_body_secondary) {
                            file_valid = false;
                        }
                    }

                    if !file_valid {
                        let filename = Path::new(&spec.rel_path)
                            .file_name()
                            .and_then(|f| f.to_str())
                            .unwrap_or(&spec.rel_path);

                        findings.push(SpecAuditFinding::new(
                            "behavioral-trigger-rule-must-be-satisfied-by-the-specification",
                            format!("In {}: {}", filename, rule.error_message),
                            spec.rel_path.clone(),
                            None,
                        ));
                    }
                }
            }
        }
    }

    Ok(findings)
}

/// Discovers specification files in workspace directories and runs behavioral trigger validation.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `validate_behavioral_triggers_workspace` in `crates/verify-baseline/src/spec_audit/behavioral.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `validate_behavioral_triggers_workspace` (declaration #18) within `SPEC-AUDIT-BEHAVIORAL`.
///
/// ## 3. Preconditions
/// - `workspace_root: &Path` must satisfy domain and type-safety constraints prior to invoking `validate_behavioral_triggers_workspace`.
/// - `symbol_index: &SysmlSymbolIndex` must satisfy domain and type-safety constraints prior to invoking `validate_behavioral_triggers_workspace`.
///
/// ## 4. Postconditions
/// - Returns `Result<Vec<SpecAuditFinding>, SpecAuditError>` from `validate_behavioral_triggers_workspace` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(workspace_root)) bounded over input/member dimension of `validate_behavioral_triggers_workspace`.
/// - Space Complexity: O(N(workspace_root)) bounded auxiliary memory for `validate_behavioral_triggers_workspace`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `validate_behavioral_triggers_workspace` (`Result<Vec<SpecAuditFinding>, SpecAuditError>`).
/// - State: Maintains consistent state invariants across `validate_behavioral_triggers_workspace` without partial mutation on error.
/// - Action: Executes `validate_behavioral_triggers_workspace` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `validate_behavioral_triggers_workspace` (`SPEC-AUDIT-BEHAVIORAL`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-VALIDATE-BEHAVIORAL-TRIGGERS-WORKSPACE] | SysML v2: ActionDef(validate_behavioral_triggers_workspace)
pub fn validate_behavioral_triggers_workspace(
    workspace_root: &Path,
    symbol_index: &SysmlSymbolIndex,
) -> Result<Vec<SpecAuditFinding>, SpecAuditError> {
    let triggers_path = workspace_root.join("rules").join("behavioral_triggers.json");
    if !triggers_path.exists() {
        return Ok(Vec::new());
    }

    let triggers = load_behavioral_triggers(&triggers_path)?;
    let mut specs = Vec::new();

    let target_dirs = [
        ("user-story", workspace_root.join("docs").join("user-stories")),
        ("use-case", workspace_root.join("docs").join("use-cases")),
    ];

    for (spec_type, dir_path) in target_dirs {
        if !dir_path.is_dir() {
            continue;
        }

        let read_dir = std::fs::read_dir(&dir_path).map_err(|e| {
            SpecAuditError::IoError(format!(
                "Failed to read specification directory '{}': {}",
                dir_path.display(),
                e
            ))
        })?;

        for entry_res in read_dir {
            let entry = entry_res.map_err(|e| {
                SpecAuditError::IoError(format!("Failed to read directory entry: {}", e))
            })?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                let rel_path = path
                    .strip_prefix(workspace_root)
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| path.to_string_lossy().to_string());

                let spec = AuditedSpec::from_file(&path, rel_path, spec_type)?;
                specs.push(spec);
            }
        }
    }

    validate_behavioral_triggers(&triggers, symbol_index, &specs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_trigger() -> BehavioralTrigger {
        BehavioralTrigger {
            name: "High-Frequency Stream Data Trigger".to_string(),
            trigger_nodes: vec![
                "stream-data".to_string(),
                "sensor-stream".to_string(),
            ],
            rules: vec![
                BehavioralRule {
                    target_type: "user-story".to_string(),
                    requires_mermaid_block: Some("sequenceDiagram".to_string()),
                    match_terms_in_mermaid: vec![
                        "OffThreadHandler".to_string(),
                        "ConcurrencyGuard".to_string(),
                    ],
                    match_terms_in_body: Vec::new(),
                    match_terms_in_body_secondary: Vec::new(),
                    error_message: "User Story diagrams must illustrate off-thread computations (OffThreadHandler).".to_string(),
                },
                BehavioralRule {
                    target_type: "use-case".to_string(),
                    requires_mermaid_block: None,
                    match_terms_in_mermaid: Vec::new(),
                    match_terms_in_body: vec![
                        "temporal-context".to_string(),
                        "playback".to_string(),
                    ],
                    match_terms_in_body_secondary: Vec::new(),
                    error_message: "Use Cases must detail the temporal context state machine.".to_string(),
                },
            ],
        }
    }

    #[test]
    fn test_inactive_trigger_imposes_nothing() {
        let trigger = create_test_trigger();
        let index = SysmlSymbolIndex::new(); // empty index -> inactive trigger
        let specs = Vec::new();

        let findings = validate_behavioral_triggers(&[trigger], &index, &specs).unwrap();
        assert!(findings.is_empty(), "Inactive trigger should impose nothing");
    }

    #[test]
    fn test_active_trigger_missing_coverage_fails() {
        let trigger = create_test_trigger();
        let mut index = SysmlSymbolIndex::new();
        index.part_names.insert("stream-data".to_string()); // arms the trigger!

        let specs = Vec::new(); // no specifications present

        let findings = validate_behavioral_triggers(&[trigger], &index, &specs).unwrap();
        assert_eq!(findings.len(), 2, "Expected 2 findings (one per rule for stream-data)");
        assert_eq!(
            findings[0].rule_id,
            "behavioral-trigger-node-must-be-covered-by-a-specification"
        );
        assert_eq!(
            findings[1].rule_id,
            "behavioral-trigger-node-must-be-covered-by-a-specification"
        );
    }

    #[test]
    fn test_active_trigger_user_story_passes_with_required_mermaid() {
        let trigger = create_test_trigger();
        let mut index = SysmlSymbolIndex::new();
        index.part_names.insert("stream-data".to_string());

        let user_story_content = r#"# US-01 Ingest Sensor Data

The system ingests high-frequency stream-data from telemetry sensors.

```mermaid
sequenceDiagram
    participant S as Sensor
    participant O as OffThreadHandler
    S->>O: transmit(stream_data)
```
"#;

        let use_case_content = r#"# UC-01 Stream Data Playback

The operator reviews recorded stream-data.

## 4. Main Success Scenario
1. Operator requests temporal-context playback.
"#;

        let specs = vec![
            AuditedSpec::from_content("docs/user-stories/us-01.md", "user-story", user_story_content).unwrap(),
            AuditedSpec::from_content("docs/use-cases/uc-01.md", "use-case", use_case_content).unwrap(),
        ];

        let findings = validate_behavioral_triggers(&[trigger], &index, &specs).unwrap();
        assert!(
            findings.is_empty(),
            "Expected 0 findings for passing specs, got: {:?}",
            findings
        );
    }

    #[test]
    fn test_active_trigger_fails_when_mermaid_lacks_match_terms() {
        let trigger = create_test_trigger();
        let mut index = SysmlSymbolIndex::new();
        index.part_names.insert("stream-data".to_string());

        let invalid_user_story = r#"# US-01 Ingest Sensor Data

The system ingests high-frequency stream-data from telemetry sensors.

```mermaid
sequenceDiagram
    participant S as Sensor
    participant B as Buffer
    S->>B: transmit(stream_data)
```
"#;

        let valid_use_case = r#"# UC-01 Stream Data Playback

The operator reviews recorded stream-data.

## 4. Main Success Scenario
1. Operator requests playback mode.
"#;

        let specs = vec![
            AuditedSpec::from_content("docs/user-stories/us-01.md", "user-story", invalid_user_story).unwrap(),
            AuditedSpec::from_content("docs/use-cases/uc-01.md", "use-case", valid_use_case).unwrap(),
        ];

        let findings = validate_behavioral_triggers(&[trigger], &index, &specs).unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(
            findings[0].rule_id,
            "behavioral-trigger-rule-must-be-satisfied-by-the-specification"
        );
        assert!(findings[0].message.contains("OffThreadHandler"));
    }

    #[test]
    fn test_active_trigger_fails_when_use_case_lacks_body_terms() {
        let trigger = create_test_trigger();
        let mut index = SysmlSymbolIndex::new();
        index.part_names.insert("stream-data".to_string());

        let valid_user_story = r#"# US-01 Ingest Sensor Data

The system ingests high-frequency stream-data from telemetry sensors.

```mermaid
sequenceDiagram
    participant S as Sensor
    participant O as OffThreadHandler
    S->>O: transmit(stream_data)
```
"#;

        let invalid_use_case = r#"# UC-01 Stream Data Monitor

The operator reviews recorded stream-data.

## 4. Main Success Scenario
1. Operator requests direct feed view without any temporal mechanisms.
"#;

        let specs = vec![
            AuditedSpec::from_content("docs/user-stories/us-01.md", "user-story", valid_user_story).unwrap(),
            AuditedSpec::from_content("docs/use-cases/uc-01.md", "use-case", invalid_use_case).unwrap(),
        ];

        let findings = validate_behavioral_triggers(&[trigger], &index, &specs).unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(
            findings[0].rule_id,
            "behavioral-trigger-rule-must-be-satisfied-by-the-specification"
        );
        assert!(findings[0].message.contains("temporal context state machine"));
    }

    #[test]
    fn test_contains_token_with_word_boundary() {
        assert!(contains_token_with_word_boundary("stream-data payload", "stream-data"));
        assert!(contains_token_with_word_boundary("The `stream-data` node", "stream-data"));
        assert!(!contains_token_with_word_boundary("mainstream-database", "stream-data"));
        assert!(!contains_token_with_word_boundary("stream-dataset", "stream-data"));
    }

    #[test]
    fn test_extract_body_outside_code_blocks() {
        let content = "Heading 1\n```mermaid\nsequenceDiagram\n```\nParagraph text\n";
        let body = extract_body_outside_code_blocks(content);
        assert!(body.contains("Heading 1"));
        assert!(body.contains("Paragraph text"));
        assert!(!body.contains("sequenceDiagram"));
    }
}
