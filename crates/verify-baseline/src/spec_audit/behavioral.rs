//! Native Rust Behavioral Triggers Validator conforming to DO-178C Level A and ISO 26262 ASIL D.
//!
//! Strongly-typed validation of `rules/behavioral_triggers.json` requirements against workspace
//! SysML schemas and specification documents (User Stories and Use Cases).

use crate::spec_audit::behavioral_matchers::spec_satisfies_behavioral_rule;
pub use crate::spec_audit::behavioral_matchers::{
    collect_normalized_symbols, contains_token_with_word_boundary, extract_body_outside_code_blocks,
    matches_target_type, normalize_symbol_name, spec_references_node,
};
use crate::spec_audit::markdown_ast::{parse_markdown, MarkdownDoc};
use crate::spec_audit::uml_validator::SysmlSymbolIndex;
use crate::spec_audit::{SpecAuditError, SpecAuditFinding};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::Path;

/// Individual rule within a behavioral trigger specifying target document type and verification constraints.
/// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-BEHAVIORALRULE]
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
/// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-BEHAVIORALTRIGGER]
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
    /// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-BEHAVIORALTRIGGER-ACTIVE-NODES]
    pub fn active_nodes(&self, index: &SysmlSymbolIndex) -> Vec<String> {
        let normalized_symbols = collect_normalized_symbols(index);
        self.active_nodes_from_normalized_set(&normalized_symbols)
    }

    /// Determines which trigger nodes are active given an already normalized set of schema symbol names.
    /// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-BEHAVIORALTRIGGER-ACTIVE-NODES-FROM-NORMALIZED-SET]
    pub fn active_nodes_from_normalized_set(
        &self,
        normalized_symbols: &BTreeSet<String>,
    ) -> Vec<String> {
        self.trigger_nodes
            .iter()
            .filter(|node| normalized_symbols.contains(&normalize_symbol_name(node)))
            .cloned()
            .collect()
    }
}

/// In-memory representation of an audited specification document.
/// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-AUDITEDSPEC]
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
    /// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-AUDITEDSPEC-FROM-CONTENT]
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
    /// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-AUDITEDSPEC-FROM-FILE]
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

/// Parses behavioral triggers JSON configuration into strongly-typed structures.
/// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-PARSE-BEHAVIORAL-TRIGGERS]
pub fn parse_behavioral_triggers(
    json_content: &str,
) -> Result<Vec<BehavioralTrigger>, SpecAuditError> {
    serde_json::from_str(json_content).map_err(|e| {
        SpecAuditError::RuleError(format!("Failed to parse behavioral triggers JSON: {}", e))
    })
}

/// Loads and parses behavioral triggers JSON configuration from a file path.
/// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-LOAD-BEHAVIORAL-TRIGGERS]
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
/// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-VALIDATE-BEHAVIORAL-TRIGGERS]
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
            continue;
        }

        for rule in &trigger.rules {
            let candidate_specs: Vec<&AuditedSpec> = specs
                .iter()
                .filter(|s| matches_target_type(&s.spec_type, &s.rel_path, &rule.target_type))
                .collect();

            for active_node in &active_nodes {
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

                for spec in referencing_specs {
                    if !spec_satisfies_behavioral_rule(spec, rule) {
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
/// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-VALIDATE-BEHAVIORAL-TRIGGERS-WORKSPACE]
pub fn validate_behavioral_triggers_workspace(
    workspace_root: &Path,
    symbol_index: &SysmlSymbolIndex,
) -> Result<Vec<SpecAuditFinding>, SpecAuditError> {
    let triggers_path = workspace_root
        .join("rules")
        .join("behavioral_triggers.json");
    if !triggers_path.exists() {
        return Ok(Vec::new());
    }

    let triggers = load_behavioral_triggers(&triggers_path)?;
    let mut specs = Vec::new();
    let target_dirs = [
        (
            "user-story",
            workspace_root.join("docs").join("user-stories"),
        ),
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
#[path = "behavioral_tests.rs"]
mod tests;
