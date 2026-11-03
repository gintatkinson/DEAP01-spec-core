//! Use Case diagram compliance, Epic statechart heterogeneity, and Parent Epic alignment gates.
//! Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR]

use std::collections::{BTreeMap, BTreeSet};

use crate::spec_audit::markdown_ast::MarkdownDoc;

/// Validates Use Case Mermaid diagram compliance conforming to DO-178C Level A and ISO 26262 ASIL D.
/// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-VALIDATE-USE-CASE-DIAGRAM-COMPLIANCE]
pub fn validate_use_case_diagram_compliance(doc: &MarkdownDoc, rel_path: &str) -> Vec<String> {
    let mut errors = Vec::new();
    if doc.mermaid_blocks.is_empty() {
        errors.push(format!(
            "{}: Use Case Diagram Compliance Gate violation: Use Case document contains no Mermaid diagram blocks.",
            rel_path
        ));
        return errors;
    }

    for block in &doc.mermaid_blocks {
        let lines: Vec<&str> = block.raw.lines().collect();
        if !lines.iter().any(|l| l.trim().to_lowercase().starts_with("subgraph")) {
            errors.push(format!(
                "{}: Use Case Diagram Compliance Gate violation: Diagram lacks a system boundary subgraph (expected 'subgraph \"... System Boundary\"').",
                rel_path
            ));
        }

        let actor_ids = collect_use_case_actor_ids(doc, &lines);
        let directed_arrows = ["-->", "->", "==>", "-.->", "<--", "<-"];
        for l in &lines {
            let t = l.trim();
            if t.starts_with("%%") {
                continue;
            }
            if directed_arrows.iter().any(|&arr| t.contains(arr))
                && actor_ids.iter().any(|act| t.contains(act) || t.to_lowercase().contains(act))
            {
                errors.push(format!(
                    "{}: Use Case Diagram Compliance Gate violation: Directed arrows ('-->') used to connect actor in line: '{}'. Actors must use undirected associations ('---') in UML Use Case diagrams.",
                    rel_path, t
                ));
                break;
            }
        }

        let has_non_stadium = lines.iter().any(|l| {
            let t = l.trim();
            !(t.starts_with("%%") || t.starts_with("subgraph") || t.starts_with("classDef") || t.starts_with("style"))
                && t.contains("[\"") && t.contains("\"]") && !t.contains("([\"") && !t.contains("\"])")
        });
        if has_non_stadium {
            errors.push(format!(
                "{}: Use Case Diagram Compliance Gate violation: Non-stadium nodes ('[\"... \"]') found in Use Case diagram. Use cases must use stadium-shaped nodes ('([\"... \"])').",
                rel_path
            ));
        }
        check_use_case_node_count(&lines, rel_path, &mut errors);
    }
    errors
}

/// Collects actor identifiers declared in table metadata or Mermaid actor nodes.
/// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-VALIDATE-USE-CASE-DIAGRAM-COMPLIANCE]
fn collect_use_case_actor_ids(doc: &MarkdownDoc, lines: &[&str]) -> BTreeSet<String> {
    let primary_actor = doc.table_metadata.get("primary_actor").map(|s| s.trim()).unwrap_or("");
    let mut actor_ids = BTreeSet::new();
    if !primary_actor.is_empty() {
        actor_ids.insert(primary_actor.to_string());
        actor_ids.insert(primary_actor.to_lowercase());
    }
    actor_ids.insert("actor".to_string());
    actor_ids.insert("Actor".to_string());

    for l in lines {
        let t = l.trim();
        if let Some(after_actor) = t.strip_prefix("actor ") {
            let act_id = after_actor.split_whitespace().next().unwrap_or("");
            if !act_id.is_empty() {
                actor_ids.insert(act_id.to_string());
                actor_ids.insert(act_id.to_lowercase());
            }
        } else if let Some(paren_pos) = t.find("((\"") {
            let act_id = t[..paren_pos].trim();
            if !act_id.is_empty() {
                actor_ids.insert(act_id.to_string());
                actor_ids.insert(act_id.to_lowercase());
            }
        }
    }
    actor_ids
}

/// Verifies that a Use Case diagram contains more than 2 distinct nodes.
/// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-VALIDATE-USE-CASE-DIAGRAM-COMPLIANCE]
fn check_use_case_node_count(lines: &[&str], rel_path: &str, errors: &mut Vec<String>) {
    let mut node_ids = BTreeSet::new();
    for l in lines {
        let t = l.trim();
        if t.starts_with("%%")
            || t.starts_with("subgraph")
            || t.starts_with("end")
            || t.starts_with("flowchart")
            || t.starts_with("graph")
            || t.starts_with("classDef")
            || t.starts_with("style")
        {
            continue;
        }
        let first_token = t.split(['[', '(', ' ', '-', '.']).next().unwrap_or("").trim();
        if !first_token.is_empty()
            && first_token.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            node_ids.insert(first_token.to_string());
        }
        for arr in &["-->", "---", "->", "-.->"] {
            if let Some(pos) = t.find(arr) {
                let after = t[pos + arr.len()..].trim();
                let target_token =
                    after.split(['[', '(', ' ', '-', '.']).next().unwrap_or("").trim();
                if !target_token.is_empty()
                    && target_token.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                {
                    node_ids.insert(target_token.to_string());
                }
            }
        }
    }
    if node_ids.len() <= 2 {
        errors.push(format!(
            "{}: Use Case Diagram Compliance Gate violation: Trivial 2-node flow detected (found {} distinct node(s)). Use case diagrams must model complete subsystem boundaries and scenario step inclusions.",
            rel_path,
            node_ids.len()
        ));
    }
}

/// Validates Epic statechart heterogeneity conforming to DO-178C Level A and ISO 26262 ASIL D.
/// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-VALIDATE-EPIC-STATECHART-HETEROGENEITY]
pub fn validate_epic_statechart_heterogeneity(epics: &[(&str, &MarkdownDoc)]) -> Vec<String> {
    let mut errors = Vec::new();
    let mut epic_transitions: Vec<(&str, Vec<(String, String)>)> = Vec::new();

    for &(rel_path, doc) in epics {
        for block in &doc.mermaid_blocks {
            let lower_raw = block.raw.to_lowercase();
            if block.header.starts_with("stateDiagram") || lower_raw.contains("statediagram") {
                let mut trans_list = Vec::new();
                for line in block.raw.lines() {
                    let t = line.trim();
                    if t.starts_with("%%") || t.starts_with("stateDiagram") || t.is_empty() {
                        continue;
                    }
                    if let Some(arrow_idx) = t.find("-->") {
                        let from = t[..arrow_idx].trim();
                        let rest = t[arrow_idx + 3..].trim();
                        let to = rest.split_once(':').map(|(lhs, _)| lhs.trim()).unwrap_or(rest);
                        if !from.is_empty() && !to.is_empty() {
                            trans_list.push((from.to_string(), to.to_string()));
                        }
                    }
                }
                trans_list.sort();
                epic_transitions.push((rel_path, trans_list));
                break;
            }
        }
    }

    if epic_transitions.len() < 2 {
        return errors;
    }

    let mut canned_epics = Vec::new();
    for (rel_path, trans) in &epic_transitions {
        let has_boot = trans.iter().any(|(f, t)| f == "Bootstrapping" && t == "Ingesting");
        let has_ing = trans.iter().any(|(f, t)| f == "Ingesting" && t == "Compiling");
        let has_comp = trans.iter().any(|(f, t)| f == "Compiling" && t == "Verifying");
        if has_boot && has_ing && has_comp {
            canned_epics.push(*rel_path);
        }
    }
    if canned_epics.len() > 1 {
        errors.push(format!(
            "Epic Statechart Heterogeneity Gate violation: Multiple Epics ({}) share identical canned pipeline lifecycle statecharts ('Bootstrapping -> Ingesting -> Compiling -> Verifying') without subsystem-specific states.",
            canned_epics.join(", ")
        ));
    }

    let mut checked_pairs = BTreeSet::new();
    for (i, (path1, trans1)) in epic_transitions.iter().enumerate() {
        for (path2, trans2) in epic_transitions.iter().skip(i + 1) {
            if !trans1.is_empty() && trans1 == trans2 {
                let pair_key = if path1 < path2 {
                    format!("{}|{}", path1, path2)
                } else {
                    format!("{}|{}", path2, path1)
                };
                if checked_pairs.insert(pair_key)
                    && !(canned_epics.contains(path1) && canned_epics.contains(path2))
                {
                    errors.push(format!(
                        "Epic Statechart Heterogeneity Gate violation: Epics '{}' and '{}' share identical statechart transitions ({} transitions). Epics must define distinct subsystem-specific statecharts.",
                        path1, path2, trans1.len()
                    ));
                }
            }
        }
    }

    errors
}

/// Extracts a clean epic stem from a Markdown link or identifier.
/// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-EXTRACT-EPIC-STEM]
fn extract_epic_stem(raw: &str) -> String {
    let clean = raw.trim().trim_matches(|c| c == '[' || c == ']');
    if let Some(idx) = clean.find('(') {
        let text = clean[..idx].trim().trim_matches(|c| c == ']' || c == '[');
        if !text.is_empty() {
            return text.to_string();
        }
    }
    if let Some(idx) = clean.rfind('/') {
        let fname = &clean[idx + 1..];
        return fname.strip_suffix(".md").unwrap_or(fname).to_string();
    }
    clean.to_string()
}

/// Extracts a subsystem name from a feature title (`Feature 06: [System Vision] ...`).
/// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-EXTRACT-SUBSYSTEM-FROM-TITLE]
fn extract_subsystem_from_title(title: &str) -> String {
    if let Some(open) = title.find('[') {
        if let Some(close) = title[open..].find(']') {
            return title[open + 1..open + close].trim().to_string();
        }
    }
    String::new()
}

/// Validates Parent Epic alignment between User Stories, Features, and Epics.
/// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-VALIDATE-PARENT-EPIC-ALIGNMENT]
pub fn validate_parent_epic_alignment(
    stories: &[(&str, &MarkdownDoc)],
    features: &[(&str, &MarkdownDoc)],
    epics: &[(&str, &MarkdownDoc)],
) -> Vec<String> {
    let mut errors = Vec::new();
    let mut feat_info: BTreeMap<String, (String, String)> = BTreeMap::new();
    for &(f_path, f_doc) in features {
        let f_basename = f_path.rsplit('/').next().unwrap_or(f_path);
        let f_stem = f_basename.strip_suffix(".md").unwrap_or(f_basename).to_lowercase();
        let parent_epic = f_doc.table_metadata.get("parent_epic").map(|s| s.trim()).unwrap_or("");
        let title = f_doc.table_metadata.get("title").map(|s| s.trim()).unwrap_or("");
        feat_info.insert(f_stem, (extract_subsystem_from_title(title), extract_epic_stem(parent_epic)));
    }

    let epic_associated_stories = collect_epic_associated_stories(epics);

    for &(s_path, s_doc) in stories {
        let s_basename = s_path.rsplit('/').next().unwrap_or(s_path);
        let s_stem = s_basename.strip_suffix(".md").unwrap_or(s_basename).to_lowercase();
        let declared_parent =
            s_doc.table_metadata.get("parent_epic").map(|s| s.trim()).unwrap_or("");
        let declared_stem = extract_epic_stem(declared_parent);
        let spec_source =
            s_doc.table_metadata.get("specification_source").map(|s| s.trim()).unwrap_or("");
        let is_conops_source = spec_source.to_lowercase().contains("conops");

        let linked_feature_parent_epics = find_story_linked_feature_epics(s_doc, &feat_info);
        let primary_feature_epic = linked_feature_parent_epics
            .iter()
            .find(|(_f, p)| {
                !p.to_lowercase().contains("epic-01") && !p.to_lowercase().contains("epic-02")
            })
            .map(|(_f, p)| p.as_str());

        let expected_from_epic_checklist = epic_associated_stories
            .iter()
            .find(|(k, _)| {
                k.as_str() == s_stem
                    || (k.starts_with(&s_stem) && k[s_stem.len()..].starts_with('-'))
                    || (s_stem.starts_with(k.as_str()) && s_stem[k.len()..].starts_with('-'))
            })
            .map(|(_, v)| v.as_str());
        let expected_epic = primary_feature_epic.or(expected_from_epic_checklist);

        if let Some(expected) = expected_epic {
            let declared_lower = declared_stem.to_lowercase();
            let expected_lower = expected.to_lowercase();
            let mismatch =
                if declared_lower.contains("epic-02") && !expected_lower.contains("epic-02") {
                    true
                } else if !declared_lower.contains(&expected_lower)
                    && !expected_lower.contains(&declared_lower)
                {
                    !is_conops_source
                } else {
                    false
                };
            if mismatch {
                errors.push(format!(
                    "{}: Parent Epic Alignment Gate violation: User Story declared parent Epic '{}', but its associated subsystem / required features belong to '{}'.",
                    s_path, declared_stem, expected
                ));
            }
        }
    }

    errors
}

/// Maps User Story stems mentioned in Epic prose checklists to their owning Epic stem.
/// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-VALIDATE-PARENT-EPIC-ALIGNMENT]
fn collect_epic_associated_stories(epics: &[(&str, &MarkdownDoc)]) -> BTreeMap<String, String> {
    let mut epic_associated_stories: BTreeMap<String, String> = BTreeMap::new();
    for &(ep_path, ep_doc) in epics {
        let ep_basename = ep_path.rsplit('/').next().unwrap_or(ep_path);
        let ep_stem = ep_basename.strip_suffix(".md").unwrap_or(ep_basename);
        if ep_stem.to_lowercase().contains("epic-02") {
            continue;
        }
        for segment in &ep_doc.prose {
            for raw_word in segment
                .text
                .split(|c: char| matches!(c, '(' | ')' | '[' | ']') || c.is_whitespace())
            {
                let word =
                    raw_word.trim_matches(|c: char| c.is_ascii_punctuation() && c != '-' && c != '_');
                if word.to_lowercase().contains("us-") {
                    let s_basename = word.rsplit('/').next().unwrap_or(word);
                    let s_stem =
                        s_basename.strip_suffix(".md").unwrap_or(s_basename).to_lowercase();
                    epic_associated_stories.insert(s_stem, ep_stem.to_string());
                }
            }
        }
    }
    epic_associated_stories
}

/// Finds parent Epics of features referenced in a User Story's prose.
/// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-VALIDATE-PARENT-EPIC-ALIGNMENT]
fn find_story_linked_feature_epics(
    s_doc: &MarkdownDoc,
    feat_info: &BTreeMap<String, (String, String)>,
) -> Vec<(String, String)> {
    let mut linked = Vec::new();
    for segment in &s_doc.prose {
        for raw_word in
            segment.text.split(|c: char| matches!(c, '(' | ')' | '[' | ']') || c.is_whitespace())
        {
            let word =
                raw_word.trim_matches(|c: char| c.is_ascii_punctuation() && c != '-' && c != '_');
            if word.to_lowercase().contains("feat-") {
                let f_basename = word.rsplit('/').next().unwrap_or(word);
                let f_stem = f_basename.strip_suffix(".md").unwrap_or(f_basename).to_lowercase();
                let matched_feat = feat_info.iter().find(|(k, _)| {
                    k.as_str() == f_stem
                        || (k.starts_with(&f_stem) && k[f_stem.len()..].starts_with('-'))
                        || (f_stem.starts_with(k.as_str()) && f_stem[k.len()..].starts_with('-'))
                });
                if let Some((_, (_subsys, p_epic))) = matched_feat {
                    if !p_epic.is_empty() {
                        linked.push((f_stem, p_epic.clone()));
                    }
                }
            }
        }
    }
    linked
}
