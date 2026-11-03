//! ConOps architecture validators (Super-System, Rule E5 Subgraph Routing, Subsystem, and Interface Topology).

use crate::spec_audit::SpecAuditFinding;
use std::collections::{BTreeMap, BTreeSet};

/// Checks whether a document is a ConOps or architecture unit specification.
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-VALIDATE-SUPER-SYSTEM-ARCHITECTURE]
fn is_conops_or_arch_unit(lower_doc: &str, rel_path: &str) -> bool {
    let lower_path = rel_path.to_lowercase();
    let in_conops_dir =
        lower_path.contains("docs/conops/") || lower_path.starts_with("docs/conops");
    let is_conops_spec =
        lower_path.ends_with("/conops.md") || lower_path == "conops.md" || lower_doc.contains("# concept of operations");
    let is_full_conops = is_conops_spec || (in_conops_dir && lower_path.ends_with("conops.md"));
    let is_arch_unit =
        in_conops_dir && (lower_path.contains("04_") || lower_path.contains("08_") || lower_path.contains("architecture"));
    is_full_conops || is_arch_unit
}

/// Extracts the first fenced Mermaid block slice within `arch_region`.
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-VALIDATE-SUPER-SYSTEM-ARCHITECTURE]
fn extract_first_mermaid_slice(arch_region: &str) -> Option<&str> {
    let m_idx = arch_region.find("```mermaid")?;
    let m_slice = &arch_region[m_idx..];
    let end_idx = m_slice[10..].find("```").map(|i| m_idx + 10 + i).unwrap_or(arch_region.len());
    Some(&arch_region[m_idx..end_idx])
}

/// Audits the Mermaid diagram inside Section 4.1 / 8.1 for ports, connections, and Rule E5 routing.
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-VALIDATE-SUPER-SYSTEM-ARCHITECTURE]
fn validate_super_system_diagram_region(
    content: &str,
    rel_path: &str,
    is_template_placeholder: bool,
    findings: &mut Vec<SpecAuditFinding>,
) {
    let arch_start = content
        .find("### 4.1")
        .or_else(|| content.find("### 8.1"))
        .or_else(|| content.find("### 4.7"))
        .unwrap_or(0);
    let arch_region = content.get(arch_start..).unwrap_or(content);
    let diag_slice = extract_first_mermaid_slice(arch_region);

    let has_diagram_ports = is_template_placeholder
        || diag_slice.is_some_and(|d| {
            d.contains("PORT-") || d.contains("PORT_") || d.to_lowercase().contains("port")
        });
    if !has_diagram_ports {
        findings.push(SpecAuditFinding::new(
            "conops-super-system-ports-missing",
            format!(
                "Section 4 / 8 Super-System Architecture diagram in '{}' omits discrete port definitions (PORT-<SUBSYS>-<NAME>) mandated by DoDAF SV-1 and SysML v2 Port Taxonomy.",
                rel_path
            ),
            rel_path,
            None,
        ));
    }

    let has_diagram_conns = is_template_placeholder
        || diag_slice.is_some_and(|d| d.contains("CONN-") || d.contains("conn-"));
    if !has_diagram_conns {
        findings.push(SpecAuditFinding::new(
            "conops-super-system-connections-missing",
            format!(
                "Section 4 / 8 Super-System Architecture diagram in '{}' omits traceable connection identifiers (CONN-01..CONN-N) mandated by ISO/IEC/IEEE 29148:2018 §6.4.3 and IEEE 1362 §5.3.",
                rel_path
            ),
            rel_path,
            None,
        ));
    }

    if !is_template_placeholder {
        if let Some(diag_text) = diag_slice {
            if let Some(err_msg) = audit_modular_subgraph_edge_routing(diag_text) {
                findings.push(SpecAuditFinding::new(
                    "conops-super-system-modular-edge-routing-missing",
                    format!(
                        "Section 4 / 8 Super-System Architecture diagram in '{}' {}. Decouple super-system segment context from internal subsystem breakdown views and route external edges cleanly to boundary ports (Rule E5, refs #277).",
                        rel_path, err_msg
                    ),
                    rel_path,
                    None,
                ));
            }
        }
    }
}

/// Audits Section 4.1 / 8.1 Super-System Operational Architecture and Segment Boundaries.
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-VALIDATE-SUPER-SYSTEM-ARCHITECTURE]
pub fn validate_super_system_architecture(content: &str, rel_path: &str) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();
    let lower_doc = content.to_lowercase();
    if !is_conops_or_arch_unit(&lower_doc, rel_path) {
        return findings;
    }

    let has_super_arch = (lower_doc.contains("super-system") || lower_doc.contains("super system"))
        && (lower_doc.contains("operational architecture")
            || lower_doc.contains("segment boundaries")
            || lower_doc.contains("### 4.1")
            || lower_doc.contains("### 8.1")
            || lower_doc.contains("### 4.7"));

    if !has_super_arch {
        findings.push(SpecAuditFinding::new(
            "conops-super-system-architecture-missing",
            format!("ConOps specification in '{}' omits required Section 4.1 / 8.1 / 4.7 Super-System Operational Architecture and Segment Boundaries (ISO/IEC/IEEE 29148:2018 §6.4.2 / IEEE Std 1362-1998 §5.3).", rel_path),
            rel_path,
            None,
        ));
        return findings;
    }

    let is_template_placeholder = content.contains("{{SUPER_SYSTEM_ARCHITECTURE}}");
    let has_diagram = is_template_placeholder
        || (content.contains("```mermaid") && (lower_doc.contains("flowchart") || lower_doc.contains("graph ")));
    if !has_diagram {
        findings.push(SpecAuditFinding::new(
            "conops-super-system-boundary-diagram-missing",
            format!("Section 4.1 / 8.1 / 4.7 in '{}' omits required Mermaid boundary diagram formalizing system operational segment boundaries.", rel_path),
            rel_path,
            None,
        ));
    }

    if !lower_doc.contains("segment") {
        findings.push(SpecAuditFinding::new(
            "conops-super-system-segments-missing",
            format!("Section 4.1 / 8.1 / 4.7 in '{}' omits required operational segment definitions and segment allocation matrix.", rel_path),
            rel_path,
            None,
        ));
    }

    let has_ieee1362 = lower_doc.contains("ieee 1362")
        || lower_doc.contains("ieee std 1362")
        || lower_doc.contains("ieee:1362");
    let has_dodaf_sv1 = (lower_doc.contains("dodaf") && (lower_doc.contains("sv-1") || lower_doc.contains("sv1")))
        || lower_doc.contains("dodaf 2.02 sv-1")
        || lower_doc.contains("dodaf v2.02 sv-1");
    if !has_ieee1362 || !has_dodaf_sv1 {
        findings.push(SpecAuditFinding::new(
            "conops-super-system-standards-governance-missing",
            format!("ConOps Section 4 / 8 Super-System Architecture in '{}' lacks explicit IEEE 1362 (§5.3) and DoDAF SV-1 standards governance.", rel_path),
            rel_path,
            None,
        ));
    }

    validate_super_system_diagram_region(content, rel_path, is_template_placeholder, &mut findings);
    findings
}

/// Parses an edge line in a Mermaid flowchart into `(from_id, to_id)`.
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-AUDIT-MODULAR-SUBGRAPH-EDGE-ROUTING]
fn parse_mermaid_edge_endpoints(line: &str) -> Option<(String, String)> {
    const ARROWS: &[&str] = &[
        "<-->", "<-.->", "<==>", "-->", "-.->", "==>", "---", "-.-", "===",
    ];
    for &arrow in ARROWS {
        if let Some(pos) = line.find(arrow) {
            let from_part = line[..pos].trim();
            let after_arrow = line[pos + arrow.len()..].trim();
            let to_part = if let Some(after_pipe) = after_arrow.strip_prefix('|') {
                if let Some(second_pipe) = after_pipe.find('|') {
                    after_pipe[second_pipe + 1..].trim()
                } else {
                    after_arrow
                }
            } else {
                after_arrow
            };
            let from_id = extract_node_id_local(from_part);
            let to_id = extract_node_id_local(to_part);
            return Some((from_id, to_id));
        }
    }
    None
}

/// Audits Mermaid diagram for Rule E5 modularized boundary edge routing.
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-AUDIT-MODULAR-SUBGRAPH-EDGE-ROUTING]
pub fn audit_modular_subgraph_edge_routing(diag_text: &str) -> Option<String> {
    let mut subgraph_stack: Vec<String> = Vec::new();
    let mut max_nesting_depth = 0usize;
    let mut nested_nodes: BTreeSet<String> = BTreeSet::new();
    let mut node_subgraph: BTreeMap<String, String> = BTreeMap::new();
    let mut cross_edges: Vec<(String, String, String)> = Vec::new();

    for raw_line in diag_text.lines() {
        let line = strip_mermaid_comment_local(raw_line).trim();
        if line.is_empty() {
            continue;
        }

        if let Some(after_raw) = line.strip_prefix("subgraph") {
            let after = after_raw.trim();
            let sub_id = if let Some(bracket_pos) = after.find('[') {
                after[..bracket_pos].trim().to_string()
            } else if let Some(space_pos) = after.find(' ') {
                after[..space_pos].trim().to_string()
            } else {
                after.trim_matches('"').to_string()
            };
            subgraph_stack.push(sub_id);
            max_nesting_depth = max_nesting_depth.max(subgraph_stack.len());
            continue;
        }

        if line == "end" || line.starts_with("end ") {
            subgraph_stack.pop();
            continue;
        }

        if let Some((from_id, to_id)) = parse_mermaid_edge_endpoints(line) {
            if !from_id.is_empty() && !to_id.is_empty() {
                cross_edges.push((from_id, to_id, line.to_string()));
            }
        } else if !subgraph_stack.is_empty() {
            let node_id = extract_node_id_local(line);
            if !node_id.is_empty()
                && !matches!(node_id.as_str(), "direction" | "style" | "class" | "classDef")
                && !node_id.contains("~~~")
            {
                if let Some(current_sub) = subgraph_stack.last() {
                    node_subgraph.insert(node_id.clone(), current_sub.clone());
                }
                if subgraph_stack.len() >= 2 {
                    nested_nodes.insert(node_id);
                }
            }
        }
    }

    if max_nesting_depth >= 3 {
        return Some(format!(
            "contains monolithic deeply nested subgraphs (nesting depth {} >= 3), violating Rule E5 (modularized boundary edge routing)",
            max_nesting_depth
        ));
    }

    for (from, to, edge_line) in &cross_edges {
        if nested_nodes.contains(to) || nested_nodes.contains(from) {
            let from_sub = node_subgraph.get(from).map(|s| s.as_str()).unwrap_or("");
            let to_sub = node_subgraph.get(to).map(|s| s.as_str()).unwrap_or("");
            if from_sub != to_sub {
                return Some(format!(
                    "routes cross-hierarchy edge '{}' across nested subgraph boundaries, violating Rule E5 (modularized boundary edge routing)",
                    edge_line
                ));
            }
        }
    }

    None
}

/// Extracts a clean node identifier prefix before shape delimiters.
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-EXTRACT-NODE-ID-LOCAL]
fn extract_node_id_local(s: &str) -> String {
    let delimiters = ['[', '(', '{', '>', '-', '.', '='];
    if let Some(idx) = s.find(|c| delimiters.contains(&c)) {
        s[..idx].trim().to_string()
    } else {
        s.trim().to_string()
    }
}

/// Strips inline `%%` Mermaid comments.
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-STRIP-MERMAID-COMMENT-LOCAL]
fn strip_mermaid_comment_local(line: &str) -> &str {
    if let Some(pos) = line.find("%%") {
        &line[..pos]
    } else {
        line
    }
}

/// Audits Section 4.3 / 8.3 Subsystem Architecture (100% AST part definition coverage).
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-VALIDATE-SUBSYSTEM-ARCHITECTURE]
pub fn validate_subsystem_architecture(content: &str, rel_path: &str) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();
    let lower_doc = content.to_lowercase();
    if !is_conops_or_arch_unit(&lower_doc, rel_path) {
        return findings;
    }

    let has_subsys = lower_doc.contains("subsystem architecture")
        || lower_doc.contains("physical subsystem decomposition")
        || lower_doc.contains("### 4.3")
        || lower_doc.contains("### 8.3")
        || lower_doc.contains("### 4.8");
    if !has_subsys {
        findings.push(SpecAuditFinding::new(
            "conops-subsystem-architecture-missing",
            format!(
                "ConOps specification in '{}' omits required Section 4.3 / 8.3 Subsystem Architecture (100% AST Part Coverage) in accordance with ISO/IEC/IEEE 29148:2018 §6.4.3.",
                rel_path
            ),
            rel_path,
            None,
        ));
        return findings;
    }

    let has_ast_binding = lower_doc.contains("part def")
        || lower_doc.contains("ast part")
        || lower_doc.contains("ast_part_ref")
        || lower_doc.contains("ssot ground truth")
        || lower_doc.contains("subsystem part")
        || lower_doc.contains("functional purpose")
        || lower_doc.contains("schema/");
    if !has_ast_binding {
        findings.push(SpecAuditFinding::new(
            "conops-subsystem-ast-coverage-missing",
            format!(
                "Section 4.3 / 8.3 in '{}' omits deterministic AST part def coverage and SSOT ground truth citations.",
                rel_path
            ),
            rel_path,
            None,
        ));
    }

    findings
}

/// Audits Section 4.4 / 8.4 System Interface Topology and Port Taxonomy.
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-VALIDATE-SYSTEM-INTERFACE-TOPOLOGY]
pub fn validate_system_interface_topology(content: &str, rel_path: &str) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();
    let lower_doc = content.to_lowercase();
    if !is_conops_or_arch_unit(&lower_doc, rel_path) {
        return findings;
    }

    let has_topo = lower_doc.contains("port taxonomy")
        || lower_doc.contains("interface interconnect")
        || lower_doc.contains("system interface topology")
        || lower_doc.contains("### 4.4")
        || lower_doc.contains("### 8.4");
    if !has_topo {
        findings.push(SpecAuditFinding::new(
            "conops-system-interface-topology-missing",
            format!(
                "ConOps specification in '{}' omits required Section 4.4 / 8.4 System Interface Topology and Port Taxonomy in accordance with ISO/IEC/IEEE 29148:2018 §6.4.3.",
                rel_path
            ),
            rel_path,
            None,
        ));
    }

    findings
}
