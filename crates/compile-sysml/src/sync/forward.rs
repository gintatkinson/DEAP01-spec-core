//! SysML v2 to Markdown Forward Synchronization Engine.
//!
//! Generates canonical Agile markdown specifications dynamically from the SysML v2 SSOT AST:
//! - Epics (docs/epics/EPIC-*.md) from PackageDef & CapabilityDef
//! - Features (docs/features/FEAT-*.md) from structural PartDef
//! - User Stories (docs/user-stories/US-*.md) from InteractionDef & ActionDef
//! - Use Cases (docs/use-cases/UC-*.md) from UseCaseDef
//! - Safety Matrix (docs/safety/STPA_MATRIX.md) from ConstraintDef & STPA Generator

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use deap_core::sysml_ast::{PackageDef, PartDef, RequirementDef, UseCaseDef};
use walkdir::WalkDir;

use crate::semantic::digest::write_atomic;
use crate::stpa::uca::expand_cartesian_stpa;

/// Forward sync options.
#[derive(Debug, Clone)]
pub struct ForwardSyncOptions {
    pub schema_path: PathBuf,
    pub docs_dir: PathBuf,
    pub out_dir: Option<PathBuf>,
    pub dry_run: bool,
    pub force: bool,
}

/// Sanitizes an identifier for use in file names and symbol names.
fn sanitize_id(id: &str) -> String {
    let clean: String = id
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect();
    if clean.is_empty() {
        "Default".to_string()
    } else {
        clean
    }
}

/// Converts PascalCase or camelCase to snake_case.
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 && !result.ends_with('_') {
                result.push('_');
            }
            result.extend(ch.to_lowercase());
        } else {
            result.push(ch);
        }
    }
    result
}

/// Converts PascalCase or snake_case identifier into a space-separated title display string.
fn to_display_name(s: &str) -> String {
    let clean = s.replace('_', " ");
    let mut result = String::new();
    let mut prev_is_lower = false;
    for ch in clean.chars() {
        if ch.is_uppercase() && prev_is_lower {
            result.push(' ');
        }
        prev_is_lower = ch.is_lowercase();
        result.push(ch);
    }
    result.trim().to_string()
}

/// Strips redundant leading prefixes (e.g., `UC_01_`, `UC-01-`, `UC01_`) from a Use Case name.
fn strip_use_case_prefix(name: &str) -> &str {
    let s = name.trim();
    if s.len() >= 2 && s[..2].eq_ignore_ascii_case("uc") {
        let rest = &s[2..];
        let rest = rest.strip_prefix('_').or_else(|| rest.strip_prefix('-')).unwrap_or(rest);
        let digit_count = rest.chars().take_while(|c| c.is_ascii_digit()).count();
        if digit_count > 0 {
            let after_digits = &rest[digit_count..];
            let clean = after_digits
                .strip_prefix('_')
                .or_else(|| after_digits.strip_prefix('-'))
                .unwrap_or(after_digits);
            if !clean.is_empty() {
                return clean;
            }
        }
    }
    s
}

/// Extracts the date from an existing file if present, preserving historical spec dates,
/// otherwise falls back to the current ISO date.
fn resolve_date(target_out_dir: &Path, rel_path: &str, fallback_iso: &str) -> String {
    let full = target_out_dir.join(rel_path);
    if let Ok(content) = fs::read_to_string(&full) {
        for line in content.lines().take(25) {
            let trimmed = line.trim();
            if trimmed.starts_with("date:") {
                let d = trimmed
                    .trim_start_matches("date:")
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if !d.is_empty() {
                    return d.to_string();
                }
            } else if trimmed.starts_with('|') && trimmed.ends_with('|') {
                let inner = trimmed.trim_matches('|');
                let parts: Vec<&str> = inner.split('|').map(|s| s.trim()).collect();
                if parts.len() >= 2 {
                    let mut key = String::new();
                    for c in parts[0].chars() {
                        if !matches!(c, '*' | '`' | '_' | ':' | '#') {
                            key.push(c);
                        }
                    }
                    let key = key.trim().to_lowercase().replace(' ', "_");
                    if key == "date" {
                        let d = parts[1]
                            .trim()
                            .trim_matches('*')
                            .trim_matches('`')
                            .trim_matches('"')
                            .trim_matches('\'');
                        if !d.is_empty() {
                            return d.to_string();
                        }
                    }
                }
            }
        }
    }
    fallback_iso.to_string()
}

/// Extracts issue ID from raw markdown content if present.
fn parse_issue_id_from_content(content: &str) -> Option<usize> {
    for line in content.lines().take(25) {
        let trimmed = line.trim();
        if trimmed.starts_with("issue_id:") {
            let d = trimmed
                .trim_start_matches("issue_id:")
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .trim_start_matches('#');
            if let Ok(parsed) = d.parse::<usize>() {
                if parsed > 0 {
                    return Some(parsed);
                }
            }
        } else if trimmed.starts_with('|') && trimmed.ends_with('|') {
            let inner = trimmed.trim_matches('|');
            let parts: Vec<&str> = inner.split('|').map(|s| s.trim()).collect();
            if parts.len() >= 2 {
                let mut key = String::new();
                for c in parts[0].chars() {
                    if !matches!(c, '*' | '`' | '_' | ':' | '#') {
                        key.push(c);
                    }
                }
                let key = key.trim().to_lowercase().replace(' ', "_").replace('-', "_");
                if key == "issue_id" || key == "issue" || key == "id" {
                    let d = parts[1]
                        .trim()
                        .trim_matches('*')
                        .trim_matches('`')
                        .trim_matches('"')
                        .trim_matches('\'')
                        .trim_start_matches('#')
                        .trim();
                    if let Ok(parsed) = d.parse::<usize>() {
                        if parsed > 0 {
                            return Some(parsed);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Extracts issue ID from an existing file if present, preserving historical issue links.
/// If rel_path does not exist on disk, attempts prefix lookup for renamed files (e.g., `UC-01-*.md`).
fn resolve_issue_id(target_out_dir: &Path, rel_path: &str, fallback_id: usize) -> usize {
    let full = target_out_dir.join(rel_path);
    if let Ok(content) = fs::read_to_string(&full) {
        if let Some(id) = parse_issue_id_from_content(&content) {
            return id;
        }
    }
    // Prefix lookup for renamed files (e.g., UC-01-*.md)
    if let Some(parent) = Path::new(rel_path).parent() {
        if let Some(file_name) = Path::new(rel_path).file_name().and_then(|f| f.to_str()) {
            let prefix_end = file_name.find('-').and_then(|first_dash| {
                file_name[first_dash + 1..].find('-').map(|second_dash| first_dash + 1 + second_dash + 1)
            });
            if let Some(prefix_len) = prefix_end {
                let prefix = &file_name[..prefix_len];
                let dir_path = target_out_dir.join(parent);
                if let Ok(entries) = fs::read_dir(&dir_path) {
                    for entry in entries.flatten() {
                        let name = entry.file_name();
                        let name_str = name.to_string_lossy();
                        if name_str.starts_with(prefix) && name_str.ends_with(".md") {
                            if let Ok(content) = fs::read_to_string(entry.path()) {
                                if let Some(id) = parse_issue_id_from_content(&content) {
                                    return id;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    fallback_id
}

/// Generates a valid Draft 2020-12 JSON Schema payload from PartDef ports and attributes.
fn generate_json_schema(part: &PartDef) -> String {
    let mut props = Vec::new();
    let mut required = Vec::new();

    props.push(format!(
        "    \"part\": {{\n      \"type\": \"string\",\n      \"const\": \"{}\"\n    }}",
        part.name
    ));
    required.push("\"part\"".to_string());

    props.push(
        "    \"status\": {\n      \"type\": \"string\",\n      \"enum\": [\"nominal\", \"degraded\", \"fault\"]\n    }".to_string()
    );
    required.push("\"status\"".to_string());

    for attr in &part.attributes {
        let json_type = match attr.type_name.as_str() {
            "Integer" | "int" => "integer",
            "Real" | "Float" | "float" | "double" => "number",
            "Boolean" | "bool" => "boolean",
            _ => "string",
        };
        props.push(format!(
            "    \"{}\": {{\n      \"type\": \"{}\",\n      \"description\": \"Subsystem attribute {}\"\n    }}",
            attr.name, json_type, attr.name
        ));
        required.push(format!("\"{}\"", attr.name));
    }

    for port in &part.ports {
        props.push(format!(
            "    \"{}\": {{\n      \"type\": \"object\",\n      \"properties\": {{\n        \"direction\": {{ \"type\": \"string\", \"const\": \"{}\" }},\n        \"port_type\": {{ \"type\": \"string\", \"const\": \"{}\" }}\n      }},\n      \"required\": [\"direction\", \"port_type\"]\n    }}",
            port.name, port.direction, port.type_name
        ));
        required.push(format!("\"{}\"", port.name));
    }

    format!(
        r#"{{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "{name}PayloadSchema",
  "type": "object",
  "properties": {{
{props}
  }},
  "required": [
    {reqs}
  ],
  "additionalProperties": false
}}"#,
        name = part.name,
        props = props.join(",\n"),
        reqs = required.join(",\n    ")
    )
}

/// Generates Acceptance Criteria (BDD) dynamically from PartDef constraints and matching RequirementDef nodes.
fn generate_feature_bdd(part: &PartDef, reqs: &[&RequirementDef]) -> String {
    let mut criteria = Vec::new();

    // Pattern A: Nominal domain state / signal handling
    criteria.push(format!(
        "- [ ] AC-01: Given nominal {} domain state, When command or input signals are received, Then updates internal state and signal outputs within declared bounds.",
        part.name
    ));

    // Pattern B: Invariant / Constraint verification
    let mut constraint_names: Vec<String> = Vec::new();
    let mut seen_names = HashSet::new();

    for c in &part.constraints {
        let name = c.name.trim();
        if !name.is_empty() && seen_names.insert(name.to_string()) {
            constraint_names.push(name.to_string());
        }
    }

    for req in reqs {
        if !req.constraints.is_empty() {
            for c in &req.constraints {
                let name = c.name.trim();
                if !name.is_empty() && seen_names.insert(name.to_string()) {
                    constraint_names.push(name.to_string());
                }
            }
        } else {
            let name = req.name.trim();
            if !name.is_empty() && seen_names.insert(name.to_string()) {
                constraint_names.push(name.to_string());
            } else if !req.req_id.trim().is_empty() && seen_names.insert(req.req_id.trim().to_string()) {
                constraint_names.push(req.req_id.trim().to_string());
            }
        }
    }

    for req in &part.requirements {
        if !req.constraints.is_empty() {
            for c in &req.constraints {
                let name = c.name.trim();
                if !name.is_empty() && seen_names.insert(name.to_string()) {
                    constraint_names.push(name.to_string());
                }
            }
        } else {
            let name = req.name.trim();
            if !name.is_empty() && seen_names.insert(name.to_string()) {
                constraint_names.push(name.to_string());
            }
        }
    }

    // Ensure granularity bounds: between 3 and 10 acceptance criteria total per feature.
    // With Pattern A (1) and Pattern C (1), Pattern B must have between 1 and 8 criteria.
    if constraint_names.is_empty() {
        constraint_names.push("operational_invariants".to_string());
    } else if constraint_names.len() > 8 {
        constraint_names.truncate(8);
    }

    for c_name in &constraint_names {
        let ac_num = criteria.len() + 1;
        criteria.push(format!(
            "- [ ] AC-{:02}: Given operational state, When {} evaluates constraint '{}', Then asserts invariant satisfaction without violation.",
            ac_num, part.name, c_name
        ));
    }

    // Pattern C: Failsafe / negative error recovery
    let ac_num = criteria.len() + 1;
    criteria.push(format!(
        "- [ ] AC-{:02}: Given malformed input signals or boundary condition violation, When {} error handling is invoked, Then transitions safely to failsafe fallback state without corruption.",
        ac_num, part.name
    ));

    criteria.join("\n")
}

/// Generates a valid Mermaid class diagram dynamically lowered from PartDef members and subparts.
fn generate_class_diagram(part: &PartDef) -> String {
    let mut lines = Vec::new();
    lines.push("classDiagram".to_string());

    let has_members = !part.ports.is_empty() || !part.attributes.is_empty() || !part.operations.is_empty();

    if has_members {
        lines.push(format!("    class {} {{", part.name));

        for port in &part.ports {
            let dir_prefix = if port.direction == "in" {
                "in"
            } else if port.direction == "out" {
                "out"
            } else {
                "inout"
            };
            let p_type = if port.type_name.is_empty() { "Port" } else { &port.type_name };
            lines.push(format!("        +{} {} ({})", p_type, port.name, dir_prefix));
        }

        for attr in &part.attributes {
            let a_type = if attr.type_name.is_empty() { "String" } else { &attr.type_name };
            lines.push(format!("        +{} {}", a_type, attr.name));
        }

        for op in &part.operations {
            let ret = op.return_type.as_deref().unwrap_or("void");
            lines.push(format!("        +{}() {}", op.name, ret));
        }

        lines.push("    }".to_string());
    } else {
        lines.push(format!("    class {}", part.name));
    }

    for subpart in &part.parts {
        let target_type = subpart.type_name.as_deref().unwrap_or(&subpart.name);
        lines.push(format!("    {} --> {} : {}", part.name, target_type, subpart.name));
    }

    lines.join("\n")
}

/// Generates a compliant UML Use Case diagram with system boundary, stadium-shaped use case nodes,
/// external actors outside boundary, and undirected associations.
fn generate_use_case_diagram(
    primary_actor: &str,
    subject: &str,
    clean_title: &str,
    main_scenario: &[String],
) -> String {
    let mut lines = Vec::new();
    lines.push("graph TD".to_string());
    lines.push(format!("    subgraph \"{} System Boundary\"", subject));
    lines.push(format!("        UC_Main([\"{}\"])", clean_title));

    let mut step_ids = Vec::new();
    for (idx, step) in main_scenario.iter().enumerate().take(6) {
        let step_id = format!("UC_Step{:02}", idx + 1);
        let mut clean_step = step.trim().to_string();
        if let Some(dot_idx) = clean_step.find(". ") {
            if clean_step[..dot_idx].chars().all(|c| c.is_ascii_digit()) {
                clean_step = clean_step[dot_idx + 2..].trim().to_string();
            }
        }
        if clean_step.len() > 60 {
            clean_step.truncate(57);
            clean_step.push_str("...");
        }
        let clean_step = clean_step.replace('"', "'");
        lines.push(format!("        {}([\"{}\"])", step_id, clean_step));
        step_ids.push(step_id);
    }
    lines.push("    end".to_string());

    let actor_id = sanitize_id(primary_actor);
    lines.push(format!("    {}((\"{}\")) --- UC_Main", actor_id, primary_actor));
    for step_id in step_ids {
        lines.push(format!("    UC_Main -. \"&lt;&lt;include&gt;&gt;\" .-> {}", step_id));
    }

    lines.join("\n")
}

/// Generates a Mermaid state machine diagram lowered from PackageDef state_defs or falls back to system lifecycle modes.
fn generate_epic_statechart(p: &PackageDef) -> String {
    if let Some(st_def) = p.state_defs.first() {
        let mut lines = Vec::new();
        lines.push("stateDiagram-v2".to_string());
        let mut source_states = HashSet::new();
        let mut target_states = HashSet::new();

        for trans in &st_def.transitions {
            let t_name = trans.name.as_deref().unwrap_or("");
            if let Some(first_idx) = t_name.find("first ") {
                let after_first = &t_name[first_idx + 6..];
                let src_candidate = after_first.split_whitespace().next().unwrap_or("");
                if let Some(then_idx) = after_first.find("then ") {
                    let after_then = &after_first[then_idx + 5..];
                    let tgt_candidate = after_then.split_whitespace().next().unwrap_or("");
                    if !src_candidate.is_empty() && !tgt_candidate.is_empty() {
                        let src = if src_candidate == "entry" { "[*]" } else { src_candidate };
                        let tgt = tgt_candidate;
                        lines.push(format!("    {} --> {}", src, tgt));
                        if src != "[*]" {
                            source_states.insert(src.to_string());
                        }
                        target_states.insert(tgt.to_string());
                    }
                }
            }
        }

        // Add terminal transition for any target state that is not a source state
        for tgt in &target_states {
            if !source_states.contains(tgt) {
                lines.push(format!("    {} --> [*]", tgt));
            }
        }

        if lines.len() > 1 {
            return lines.join("\n");
        }
    }

    if p.name == "ConOps" {
        return r#"stateDiagram-v2
    [*] --> IngestOEMArtifacts
    IngestOEMArtifacts --> ParseSysMLAST : tokens_streamed
    ParseSysMLAST --> ValidateModelSemantics : ast_built
    ValidateModelSemantics --> SynthesizeICDInterconnects : semantics_verified
    SynthesizeICDInterconnects --> EmitDownstreamArtifacts : icd_synthesized
    EmitDownstreamArtifacts --> [*]"#.to_string();
    }

    // Default canonical lifecycle state machine for root / system
    r#"stateDiagram-v2
    [*] --> SystemColdStart
    SystemColdStart --> SubsystemDiscovery : initialize_compiler
    SubsystemDiscovery --> PipelineOrchestration : subsystems_bound
    PipelineOrchestration --> HealthDiagnosticsEvaluation : pipeline_completed
    HealthDiagnosticsEvaluation --> SystemReady : diagnostics_clear
    HealthDiagnosticsEvaluation --> DegradedFailure : errors_reported
    SystemReady --> [*]
    DegradedFailure --> [*]"#.to_string()
}

/// Generates an expanded Mermaid class diagram for an Epic including constituent part defs,
/// port defs, attributes, and composition relationships.
fn generate_epic_class_diagram(p: &PackageDef, primary_engine: &str, is_root: bool) -> String {
    let mut lines = Vec::new();
    lines.push("classDiagram".to_string());

    if is_root {
        if let Some(root_part) = p.part_defs.iter().find(|part| part.name == "DEAPCompilerSystem") {
            let has_members = !root_part.ports.is_empty() || !root_part.attributes.is_empty() || !root_part.operations.is_empty();
            if has_members {
                lines.push("    class DEAPCompilerSystem {".to_string());
                for port in &root_part.ports {
                    let dir_prefix = if port.direction == "in" {
                        "in"
                    } else if port.direction == "out" {
                        "out"
                    } else {
                        "inout"
                    };
                    let p_type = if port.type_name.is_empty() { "Port" } else { &port.type_name };
                    lines.push(format!("        +{} {} ({})", p_type, port.name, dir_prefix));
                }
                for attr in &root_part.attributes {
                    let a_type = if attr.type_name.is_empty() { "String" } else { &attr.type_name };
                    lines.push(format!("        +{} {}", a_type, attr.name));
                }
                lines.push("    }".to_string());
            } else {
                lines.push("    class DEAPCompilerSystem".to_string());
            }
        } else {
            lines.push("    class DEAPCompilerSystem".to_string());
        }
        for subpart in &p.part_defs {
            if subpart.name != "DEAPCompilerSystem" {
                lines.push(format!("    class {}", subpart.name));
                lines.push(format!("    DEAPCompilerSystem --> {} : contains", subpart.name));
            }
        }
        return lines.join("\n");
    }

    lines.push("    class DEAPCompilerSystem".to_string());

    for part in &p.part_defs {
        let has_members = !part.ports.is_empty() || !part.attributes.is_empty() || !part.operations.is_empty();
        if has_members {
            lines.push(format!("    class {} {{", part.name));
            for port in &part.ports {
                let dir_prefix = if port.direction == "in" {
                    "in"
                } else if port.direction == "out" {
                    "out"
                } else {
                    "inout"
                };
                let p_type = if port.type_name.is_empty() { "Port" } else { &port.type_name };
                lines.push(format!("        +{} {} ({})", p_type, port.name, dir_prefix));
            }
            for attr in &part.attributes {
                let a_type = if attr.type_name.is_empty() { "String" } else { &attr.type_name };
                lines.push(format!("        +{} {}", a_type, attr.name));
            }
            for op in &part.operations {
                let ret = op.return_type.as_deref().unwrap_or("void");
                lines.push(format!("        +{}() {}", op.name, ret));
            }
            lines.push("    }".to_string());
        } else {
            lines.push(format!("    class {}", part.name));
        }
    }

    lines.push(format!("    DEAPCompilerSystem --> {} : contains", primary_engine));

    for part in &p.part_defs {
        if part.name != primary_engine {
            lines.push(format!("    {} --> {} : coordinates", primary_engine, part.name));
        }
        for subpart in &part.parts {
            let target_type = subpart.type_name.as_deref().unwrap_or(&subpart.name);
            lines.push(format!("    {} --> {} : {}", part.name, target_type, subpart.name));
        }
    }

    lines.join("\n")
}


/// Structured use case sections parsed from doc comments.
#[derive(Debug, Default)]
struct ParsedUseCaseDoc {
    actors: Vec<String>,
    subject: Option<String>,
    preconditions: Vec<String>,
    trigger: String,
    main_scenario: Vec<String>,
    alternate_flows: Vec<String>,
    postconditions: Vec<String>,
}

/// Parses the structured doc comment of a UseCaseDef.
fn parse_use_case_doc(doc_opt: Option<&str>) -> ParsedUseCaseDoc {
    let mut parsed = ParsedUseCaseDoc::default();
    let doc_str = match doc_opt {
        Some(s) if !s.trim().is_empty() => s.trim(),
        _ => return parsed,
    };

    let mut current_section = "";
    for line in doc_str.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Actors:") {
            let act_str = trimmed.trim_start_matches("Actors:").trim();
            parsed.actors = act_str
                .split(',')
                .map(|a| a.trim().to_string())
                .filter(|a| !a.is_empty())
                .collect();
            current_section = "actors";
        } else if trimmed.starts_with("Subject:") {
            let subj = trimmed.trim_start_matches("Subject:").trim();
            if !subj.is_empty() {
                parsed.subject = Some(subj.to_string());
            }
            current_section = "subject";
        } else if trimmed.starts_with("Preconditions:") {
            let pre = trimmed.trim_start_matches("Preconditions:").trim();
            if !pre.is_empty() {
                parsed.preconditions.push(pre.to_string());
            }
            current_section = "preconditions";
        } else if trimmed.starts_with("Trigger:") {
            parsed.trigger = trimmed.trim_start_matches("Trigger:").trim().to_string();
            current_section = "trigger";
        } else if trimmed.starts_with("Main Success Scenario:") {
            current_section = "main";
        } else if trimmed.starts_with("Alternate and Exception Flows:") {
            current_section = "alt";
        } else if trimmed.starts_with("Postconditions:") {
            let post = trimmed.trim_start_matches("Postconditions:").trim();
            if !post.is_empty() {
                parsed.postconditions.push(post.to_string());
            }
            current_section = "postconditions";
        } else if !trimmed.is_empty() {
            match current_section {
                "main" => parsed.main_scenario.push(trimmed.to_string()),
                "alt" => parsed.alternate_flows.push(trimmed.to_string()),
                "preconditions" => parsed.preconditions.push(trimmed.to_string()),
                "postconditions" => parsed.postconditions.push(trimmed.to_string()),
                _ => {}
            }
        }
    }

    parsed
}

/// Metadata associating a PartDef with its owning package context.
struct PartContext<'a> {
    part: &'a PartDef,
    pkg_name: String,
    subsys_display: String,
    subsys_dir: String,
}

/// Executes forward synchronization from the SysML model to markdown specifications.
pub fn forward_sync_sysml_to_specs(
    pkg: &PackageDef,
    opts: &ForwardSyncOptions,
) -> Result<HashMap<String, String>, String> {
    let target_out_dir = opts
        .out_dir
        .as_ref()
        .cloned()
        .unwrap_or_else(|| opts.docs_dir.clone());

    let mut generated_files = HashMap::new();
    let today_iso = Utc::now().format("%Y-%m-%d").to_string();

    let all_packages = pkg.get_all_packages();
    let mut all_requirements = Vec::new();
    for p in &all_packages {
        all_requirements.extend(p.requirement_defs.clone());
    }

    // Map each package to its parts, use cases, interactions, and actions
    let mut part_contexts: Vec<PartContext> = Vec::new();
    let mut ordered_packages: Vec<&PackageDef> = Vec::new();

    // Canonical ordering of subsystems
    const SUBSYSTEM_ORDER: &[&str] = &[
        "SystemVision",
        "UniversalSchemaIngestion",
        "CoreMetamodelNodeArena",
        "SysmlGrammarLowering",
        "PhysicalMetrologyFlows",
        "SpatioTemporalStateSolvers",
        "FormalSafetyTraceability",
        "InterconnectContracts",
        "DownstreamAgileProjections",
        "MultiTargetCodegen",
        "DiagnosticErrorCatalog",
        "CompilerAssurance",
    ];

    // Collect ordered packages: Root first (DEAP_Compiler_System), then ConOps, then 12 subsystems
    let root_pkg = all_packages
        .iter()
        .find(|p| p.name == "DEAP_Compiler_System")
        .or_else(|| all_packages.iter().find(|p| p.name != "SysML_Model" && p.name == pkg.name))
        .or_else(|| all_packages.iter().find(|p| p.name == "SysML_Model"));

    let conops_pkg = all_packages.iter().find(|p| p.name == "ConOps");

    if let Some(rp) = root_pkg {
        ordered_packages.push(rp);
    }
    if let Some(cp) = conops_pkg {
        if !ordered_packages.contains(&cp) {
            ordered_packages.push(cp);
        }
    }
    for sub_name in SUBSYSTEM_ORDER {
        if let Some(sp) = all_packages.iter().find(|p| p.name == *sub_name) {
            if !ordered_packages.contains(&sp) {
                ordered_packages.push(sp);
            }
        }
    }
    for p in &all_packages {
        if p.name != "SysML_Model" && !ordered_packages.contains(&p) {
            ordered_packages.push(p);
        }
    }

    // Map each package to its parts: ConOps first (FEAT-01..04), then DEAPCompilerSystem (FEAT-05), then 12 subsystems (FEAT-06..53)
    let mut part_packages: Vec<&PackageDef> = Vec::new();
    if let Some(cp) = conops_pkg {
        part_packages.push(cp);
    }
    if let Some(rp) = root_pkg {
        if !part_packages.contains(&rp) {
            part_packages.push(rp);
        }
    }
    for sub_name in SUBSYSTEM_ORDER {
        if let Some(sp) = all_packages.iter().find(|p| p.name == *sub_name) {
            if !part_packages.contains(&sp) {
                part_packages.push(sp);
            }
        }
    }
    for p in &all_packages {
        if p.name != "SysML_Model" && !part_packages.contains(&p) {
            part_packages.push(p);
        }
    }

    for p in &part_packages {
        let subsys_dir = if p.name == "DEAP_Compiler_System" || p.name == "SysML_Model" {
            "architecture".to_string()
        } else if p.name == "ConOps" {
            "conops".to_string()
        } else {
            to_snake_case(&p.name)
        };
        let subsys_display = if p.name == "DEAP_Compiler_System" || p.name == "SysML_Model" {
            "System Architecture".to_string()
        } else {
            to_display_name(&p.name)
        };

        for part in &p.part_defs {
            part_contexts.push(PartContext {
                part,
                pkg_name: p.name.clone(),
                subsys_display: subsys_display.clone(),
                subsys_dir: subsys_dir.clone(),
            });
        }
    }

    // Pre-calculate user story and use case allocations per package (excluding root and conops)
    struct StoryMeta {
        action_name: String,
        engine_name: String,
        caller_engine: String,
        reply_msg: String,
        interaction_name: String,
        subsys_display: String,
        pkg_name: String,
        subsys_dir: String,
        engine_feat_num: usize,
        caller_feat_num: Option<usize>,
        s_num: usize,
    }

    let mut story_metas: Vec<StoryMeta> = Vec::new();
    for p in &ordered_packages {
        if p.name == "DEAP_Compiler_System" || p.name == "ConOps" || p.name == "SysML_Model" || p.name == pkg.name {
            continue;
        }
        let action_name = match p.action_defs.first() {
            Some(a) => a.name.clone(),
            None => continue,
        };
        let subsys_display = to_display_name(&p.name);
        let subsys_dir = to_snake_case(&p.name);
        let default_engine_name = p
            .part_defs
            .iter()
            .find(|part| part.name.ends_with("Engine"))
            .map(|part| part.name.clone())
            .unwrap_or_else(|| {
                p.part_defs
                    .first()
                    .map(|part| part.name.clone())
                    .unwrap_or_else(|| "DEAPCompilerSystem".to_string())
            });

        let mut caller_engine = "DEAPCompilerSystem".to_string();
        let mut target_engine = default_engine_name.clone();
        let mut reply_msg = "OperationComplete".to_string();
        let mut interaction_name = action_name.clone();

        if let Some(inter) = p.interaction_defs.first() {
            interaction_name = inter.name.clone();
            if inter.lifelines.len() >= 2 {
                let l0 = &inter.lifelines[0];
                if let Some((_, type_name)) = l0.split_once(':') {
                    caller_engine = type_name.trim().to_string();
                }
                let l1 = &inter.lifelines[1];
                if let Some((_, type_name)) = l1.split_once(':') {
                    target_engine = type_name.trim().to_string();
                }
            }
            if inter.messages.len() >= 2 {
                let m1 = &inter.messages[1];
                if let Some(first_word) = m1.split_whitespace().next() {
                    reply_msg = first_word.to_string();
                }
            }
        }

        let default_engine_feat_num = part_contexts
            .iter()
            .position(|ctx| ctx.pkg_name == p.name && ctx.part.name == default_engine_name)
            .or_else(|| part_contexts.iter().position(|ctx| ctx.part.name == default_engine_name))
            .map(|pos| pos + 1)
            .unwrap_or(1);

        let engine_feat_num = part_contexts
            .iter()
            .position(|ctx| ctx.pkg_name == p.name && ctx.part.name == target_engine)
            .or_else(|| part_contexts.iter().position(|ctx| ctx.part.name == target_engine))
            .map(|pos| pos + 1)
            .unwrap_or(default_engine_feat_num);

        let caller_feat_num = part_contexts
            .iter()
            .position(|ctx| ctx.part.name == caller_engine)
            .map(|pos| pos + 1);

        let s_num = story_metas.len() + 1;
        story_metas.push(StoryMeta {
            action_name,
            engine_name: target_engine,
            caller_engine,
            reply_msg,
            interaction_name,
            subsys_display,
            pkg_name: p.name.clone(),
            subsys_dir,
            engine_feat_num,
            caller_feat_num,
            s_num,
        });
    }

    struct UseCaseMeta<'a> {
        uc: &'a UseCaseDef,
        pkg_name: String,
        subsys_dir: String,
        engine_name: String,
        engine_feat_num: usize,
        u_num: usize,
        story_s_num: Option<usize>,
    }

    let mut uc_metas: Vec<UseCaseMeta> = Vec::new();
    for p in &ordered_packages {
        if p.name == "DEAP_Compiler_System" || p.name == "ConOps" || p.name == "SysML_Model" || p.name == pkg.name {
            continue;
        }
        let subsys_dir = if p.name == "DEAP_Compiler_System" || p.name == "SysML_Model" {
            "architecture".to_string()
        } else if p.name == "ConOps" {
            "conops".to_string()
        } else {
            to_snake_case(&p.name)
        };
        let default_engine_name = p
            .part_defs
            .iter()
            .find(|part| part.name.ends_with("Engine"))
            .map(|part| part.name.clone())
            .unwrap_or_else(|| {
                p.part_defs
                    .first()
                    .map(|part| part.name.clone())
                    .unwrap_or_else(|| "DEAPCompilerSystem".to_string())
            });

        let default_engine_feat_num = part_contexts
            .iter()
            .position(|ctx| ctx.pkg_name == p.name && ctx.part.name == default_engine_name)
            .or_else(|| part_contexts.iter().position(|ctx| ctx.part.name == default_engine_name))
            .map(|pos| pos + 1)
            .unwrap_or(1);

        let story_s_num = story_metas
            .iter()
            .find(|s| s.pkg_name == p.name)
            .map(|s| s.s_num);

        for uc in &p.use_case_defs {
            let parsed_doc = parse_use_case_doc(uc.doc.as_deref());
            let subject_candidate = uc.subject.as_ref().or(parsed_doc.subject.as_ref());
            let engine_name = if let Some(subj) = subject_candidate {
                if p.part_defs.iter().any(|part| &part.name == subj)
                    || part_contexts.iter().any(|ctx| &ctx.part.name == subj && ctx.pkg_name == p.name)
                {
                    subj.clone()
                } else {
                    default_engine_name.clone()
                }
            } else {
                default_engine_name.clone()
            };

            let engine_feat_num = part_contexts
                .iter()
                .position(|ctx| ctx.pkg_name == p.name && ctx.part.name == engine_name)
                .or_else(|| part_contexts.iter().position(|ctx| ctx.part.name == engine_name))
                .map(|pos| pos + 1)
                .unwrap_or(default_engine_feat_num);

            let u_num = uc_metas.len() + 1;
            uc_metas.push(UseCaseMeta {
                uc,
                pkg_name: p.name.clone(),
                subsys_dir: subsys_dir.clone(),
                engine_name,
                engine_feat_num,
                u_num,
                story_s_num,
            });
        }
    }

    // 1. Epics (EPIC-*.md) dynamically lowered from PackageDef nodes
    for (idx, p) in ordered_packages.iter().enumerate() {
        let epic_num = idx + 1;
        let is_root = p.name == "DEAP_Compiler_System" || p.name == "SysML_Model" || p.name == pkg.name;
        let is_conops = p.name == "ConOps";

        let (title, rel_path, default_issue_id) = if is_root {
            (
                "Epic 01: System Architecture and Compiler Primacy".to_string(),
                "epics/EPIC-01-SysML_Model.md".to_string(),
                101,
            )
        } else if is_conops {
            (
                "Epic 02: Concept of Operations Operational Activities".to_string(),
                "epics/EPIC-02-ConOps.md".to_string(),
                102,
            )
        } else {
            (
                format!("Epic {:02}: {}", epic_num, to_display_name(&p.name)),
                format!("epics/EPIC-{:02}-{}.md", epic_num, p.name),
                epic_num + 100,
            )
        };

        let _epic_date = resolve_date(&target_out_dir, &rel_path, &today_iso);
        let epic_issue_id = resolve_issue_id(&target_out_dir, &rel_path, default_issue_id);

        let subsys_display = if is_root {
            "System Architecture".to_string()
        } else {
            to_display_name(&p.name)
        };
        let subsys_dir = if is_root {
            "architecture".to_string()
        } else if is_conops {
            "conops".to_string()
        } else {
            to_snake_case(&p.name)
        };

        let primary_engine = p
            .part_defs
            .iter()
            .find(|part| part.name.ends_with("Engine"))
            .map(|part| part.name.clone())
            .unwrap_or_else(|| {
                p.part_defs
                    .first()
                    .map(|part| part.name.clone())
                    .unwrap_or_else(|| "DEAPCompilerSystem".to_string())
            });

        // Collect feature checklist for this epic
        let mut feat_checklist = Vec::new();
        for (f_idx, ctx) in part_contexts.iter().enumerate() {
            let f_num = f_idx + 1;
            let f_path = format!("features/FEAT-{:02}-{}.md", f_num, ctx.part.name);
            let f_issue = resolve_issue_id(&target_out_dir, &f_path, f_num + 9);
            let is_conops_or_root_part = ctx.pkg_name == "ConOps" || ctx.part.name == "DEAPCompilerSystem";
            if (is_root && is_conops_or_root_part)
                || (is_conops && is_conops_or_root_part)
                || (!is_root && !is_conops && ctx.pkg_name == p.name)
            {
                feat_checklist.push(format!(
                    "- [ ] #{} - Feature {:02}: [{}] {}",
                    f_issue, f_num, ctx.subsys_display, ctx.part.name
                ));
            }
        }
        if feat_checklist.is_empty() {
            feat_checklist.push(format!("- [ ] Feature 01: Subsystem capability implementation for {}.", subsys_display));
        }
        let feat_checklist_text = feat_checklist.join("\n");

        // Associated use cases and user stories
        let mut assoc_uc = Vec::new();
        for uc_m in uc_metas.iter().filter(|m| m.pkg_name == p.name || is_conops) {
            let clean_name = strip_use_case_prefix(&uc_m.uc.name);
            let uc_slug = sanitize_id(clean_name);
            let uc_path = format!("use-cases/UC-{:02}-{}.md", uc_m.u_num, uc_slug);
            let uc_issue = resolve_issue_id(&target_out_dir, &uc_path, 94 + uc_m.u_num);
            assoc_uc.push(format!(
                "- [ ] #{} - [Use Case {:02}: {}](../use-cases/UC-{:02}-{}.md)",
                uc_issue, uc_m.u_num, to_display_name(clean_name), uc_m.u_num, uc_slug
            ));
        }

        let mut assoc_us = Vec::new();
        for s_m in story_metas.iter().filter(|m| m.pkg_name == p.name || is_conops) {
            let us_slug = sanitize_id(&s_m.action_name);
            let us_path = format!("user-stories/US-{:02}-{}.md", s_m.s_num, us_slug);
            let us_issue = resolve_issue_id(&target_out_dir, &us_path, 88 + s_m.s_num);
            assoc_us.push(format!(
                "- [ ] #{} - [User Story {:02}: {}](../user-stories/US-{:02}-{}.md)",
                us_issue, s_m.s_num, to_display_name(&s_m.action_name), s_m.s_num, us_slug
            ));
        }

        let assoc_uc_text = if assoc_uc.is_empty() {
            "- None directly allocated (operational behavior allocated at system ConOps level in Epic 02)".to_string()
        } else {
            assoc_uc.join("\n")
        };
        let assoc_us_text = if assoc_us.is_empty() {
            "- None directly allocated (operational behavior allocated at system ConOps level in Epic 02)".to_string()
        } else {
            assoc_us.join("\n")
        };

        let context_text = p
            .doc
            .clone()
            .unwrap_or_else(|| format!("Subsystem specification for {} establishing architectural layout, mathematical invariants, and verification criteria.", subsys_display));

        let source_refs = if is_root {
            r#"- Schema SSOT Architecture: `schema/model.sysml`
- ConOps Operational Activities: `schema/conops/activities.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 15288:2023 §6.4.3 Architecture Definition Process

The system architecture and formal verification invariants derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 15288."#.to_string()
        } else if is_conops {
            r#"- ConOps Operational Activities: `schema/conops/activities.sysml`
- ConOps External Actors: `schema/conops/actors.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational activity definitions and actor models derive from `schema/conops/activities.sysml` pursuant to ISO/IEC/IEEE 29148."#.to_string()
        } else {
            format!(
                r#"- System Architecture Model: `schema/model.sysml`
- Subsystem Specification Model: `schema/subsystems/{subsys_dir}/architecture.sysml`
- Subsystem Requirements Model: `schema/subsystems/{subsys_dir}/requirements.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 15288:2023 §6.4.3 Architecture Definition Process

Subsystem architectural composition and formal invariants derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 15288."#
            )
        };

        let class_diagram_block = generate_epic_class_diagram(p, &primary_engine, is_root);
        let state_diagram_block = generate_epic_statechart(p);

        let content = format!(
            r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #{epic_issue_id} |
| **Title** | {title} |
| **Type** | epic |
| **Package** | {pkg_name} |
| **Subsystem** | {subsys_display} |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# {title}

## 1. Context
{context_text}

## 2. Requirements & Checklist
{feat_checklist_text}

### Associated Use Cases & User Stories

#### Associated Use Cases
{assoc_uc_text}

#### Associated User Stories
{assoc_us_text}

## 3. Architecture
Subsystem structural composition, port allocations, and directional data connectors realized by {primary_engine}.

## 4. Operational Considerations
Deterministic compilation passes, error containment, and provable polynomial complexity execution.

## 5. Security & Governance
Safety-critical invariant satisfaction, formal trace matrix closure, and zero hardcoded domain semantics.

## 6. Source References
{source_refs}

## System-Level UML Class Diagram

```mermaid
{class_diagram_block}
```

## System State Machine Diagram

```mermaid
{state_diagram_block}
```
"#,
            title = title,
            pkg_name = p.name,
            subsys_display = subsys_display,
            epic_issue_id = epic_issue_id,
            context_text = context_text,
            feat_checklist_text = feat_checklist_text,
            assoc_uc_text = assoc_uc_text,
            assoc_us_text = assoc_us_text,
            primary_engine = primary_engine,
            source_refs = source_refs,
            class_diagram_block = class_diagram_block,
            state_diagram_block = state_diagram_block,
        );

        generated_files.insert(rel_path, content);
    }

    // 2. Features (FEAT-*.md) dynamically lowered from PartDef nodes
    for (idx, ctx) in part_contexts.iter().enumerate() {
        let f_num = idx + 1;
        let part_name = &ctx.part.name;
        let rel_path = format!("features/FEAT-{:02}-{}.md", f_num, part_name);
        let _feat_date = resolve_date(&target_out_dir, &rel_path, &today_iso);
        let feat_issue_id = resolve_issue_id(&target_out_dir, &rel_path, f_num + 9);
        let title = format!("Feature {:02}: [{}] {}", f_num, ctx.subsys_display, part_name);

        let class_diagram_block = generate_class_diagram(ctx.part);

        let json_schema = generate_json_schema(ctx.part);

        // Find matching requirements
        let matching_reqs: Vec<&RequirementDef> = all_requirements
            .iter()
            .filter(|r| {
                r.satisfied_by.iter().any(|s| s == part_name)
                    || r.name.to_lowercase().contains(&ctx.subsys_dir)
            })
            .collect();

        let bdd_criteria = generate_feature_bdd(ctx.part, &matching_reqs);

        let source_refs = if ctx.subsys_dir == "conops" {
            r#"- ConOps Operational Actors: `schema/conops/actors.sysml`
- ConOps Operational Activities: `schema/conops/activities.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations"#
                .to_string()
        } else if ctx.subsys_dir == "architecture" {
            r#"- System Architecture Model: `schema/model.sysml`
- ConOps Operational Activities: `schema/conops/activities.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 15288:2023 §6.4.3 Architecture Definition Process"#
                .to_string()
        } else {
            format!(
                r#"- Subsystem Requirements: `schema/subsystems/{subsys_dir}/requirements.sysml`
- Subsystem Architecture: `schema/subsystems/{subsys_dir}/architecture.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 15288:2023 §6.4.3 Architecture Definition Process"#,
                subsys_dir = ctx.subsys_dir
            )
        };

        let parent_epic_link = if ctx.pkg_name == "DEAP_Compiler_System" || ctx.pkg_name == "SysML_Model" || ctx.pkg_name == pkg.name {
            "[EPIC-01-SysML_Model](../epics/EPIC-01-SysML_Model.md)".to_string()
        } else if ctx.pkg_name == "ConOps" {
            "[EPIC-02-ConOps](../epics/EPIC-02-ConOps.md)".to_string()
        } else {
            let ep_idx = ordered_packages
                .iter()
                .position(|p| p.name == ctx.pkg_name)
                .map(|idx| idx + 1)
                .unwrap_or(1);
            format!(
                "[EPIC-{ep_idx:02}-{pkg_name}](../epics/EPIC-{ep_idx:02}-{pkg_name}.md)",
                ep_idx = ep_idx,
                pkg_name = ctx.pkg_name
            )
        };

        let interface_type = "Physical / Structural / Internal Bus";
        let schema_container = format!("{}::{}", ctx.pkg_name, part_name);
        let spec_source = if ctx.subsys_dir == "conops" {
            "[schema/conops/actors.sysml](../../schema/conops/actors.sysml)".to_string()
        } else if ctx.subsys_dir == "architecture" {
            "[schema/model.sysml](../../schema/model.sysml)".to_string()
        } else {
            format!("[schema/subsystems/{subsys_dir}/architecture.sysml](../../schema/subsystems/{subsys_dir}/architecture.sysml)", subsys_dir = ctx.subsys_dir)
        };

        let content = format!(
            r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #{feat_issue_id} |
| **Title** | {title} |
| **Type** | feature |
| **Parent Epic** | {parent_epic_link} |
| **Interface Type** | {interface_type} |
| **Schema Containers** | `{schema_container}` |
| **Generation Mode** | subagent |
| **Specification Source** | {spec_source} |

# {title}

## UML Class Diagram

```mermaid
{class_diagram_block}
```

## Interface Requirements
### 1. Test Data Shape
```json
{json_schema}
```

### 2. Validation & Constraints
Formal constraints and invariant guarantees enforced by {part_name}.

### 3. Visual Layout & Arrangement
Logical layout, viewport containment, and container structure for {part_name} conforming to platform design tokens. Container boundaries enforce @container isolation rules.

### 4. Interactive Flow & States
Operational lifecycle events and discrete state transitions for {part_name}.

### Layer 1: Domain State & Signal Model
Domain state vector representing {part_name} status, AST references, and typed signal models.

### Layer 2: Logic & Safety State Management
Deterministic state machine solver and safety monitor logic for {part_name}.

### Layer 3: Presentation & Actuator Interface Binding
Presentation interface binding and diagnostic event routing for {part_name}.

## Acceptance Criteria (BDD)
{bdd_criteria}

## Source References
Formal component specifications and normative systems engineering requirements:
{source_refs}

Component behavior and interface definitions derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 15288.
"#,
            title = title,
            feat_issue_id = feat_issue_id,
            parent_epic_link = parent_epic_link,
            interface_type = interface_type,
            schema_container = schema_container,
            spec_source = spec_source,
            part_name = part_name,
            class_diagram_block = class_diagram_block,
            json_schema = json_schema,
            bdd_criteria = bdd_criteria,
            source_refs = source_refs,
        );

        generated_files.insert(rel_path, content);
    }

    // 3. User Stories (US-*.md) dynamically lowered from subsystem actions and engines
    for s_meta in &story_metas {
        let s_num = s_meta.s_num;
        let story_slug = sanitize_id(&s_meta.action_name);
        let rel_path = format!("user-stories/US-{:02}-{}.md", s_num, story_slug);
        let _us_date = resolve_date(&target_out_dir, &rel_path, &today_iso);
        let us_issue_id = resolve_issue_id(&target_out_dir, &rel_path, s_num + 88);
        let title = format!("User Story {:02}: {}", s_num, to_display_name(&s_meta.action_name));

        let action_name = &s_meta.action_name;
        let engine_name = &s_meta.engine_name;
        let caller_engine = &s_meta.caller_engine;
        let reply_msg = &s_meta.reply_msg;
        let interaction_name = &s_meta.interaction_name;
        let engine_feat_num = s_meta.engine_feat_num;
        let subsys_display = &s_meta.subsys_display;
        let subsys_dir = &s_meta.subsys_dir;

        let seq_diagram = format!(
            r#"sequenceDiagram
    autonumber
    actor HumanEngineer
    participant {caller_engine}
    participant {engine_name}
    HumanEngineer->>{caller_engine}: {action_name}()
    {caller_engine}->>{engine_name}: {action_name}()
    {engine_name}-->>{caller_engine}: {reply_msg}
    {caller_engine}-->>HumanEngineer: StatusReport"#
        );

        let mut req_features_list = Vec::new();
        req_features_list.push("- [ ] [Feature 01: [ConOps] Human Engineer Interface](../features/FEAT-01-HumanEngineer.md)".to_string());
        if let Some(c_num) = s_meta.caller_feat_num {
            let c_subsys = part_contexts.get(c_num - 1).map(|c| c.subsys_display.as_str()).unwrap_or("Architecture");
            req_features_list.push(format!(
                "- [ ] [Feature {:02}: [{}] {}](../features/FEAT-{:02}-{}.md)",
                c_num, c_subsys, caller_engine, c_num, caller_engine
            ));
        }
        if caller_engine != engine_name {
            req_features_list.push(format!(
                "- [ ] [Feature {:02}: [{}] {}](../features/FEAT-{:02}-{}.md)",
                engine_feat_num, subsys_display, engine_name, engine_feat_num, engine_name
            ));
        }
        let req_features = req_features_list.join("\n");

        let parent_epic_link = {
            let ep_idx = ordered_packages
                .iter()
                .position(|p| p.name == s_meta.pkg_name)
                .map(|idx| idx + 1)
                .unwrap_or(2);
            if ep_idx == 2 {
                "[EPIC-02-ConOps](../epics/EPIC-02-ConOps.md)".to_string()
            } else {
                format!(
                    "[EPIC-{:02}-{}](../epics/EPIC-{:02}-{}.md)",
                    ep_idx, s_meta.pkg_name, ep_idx, s_meta.pkg_name
                )
            }
        };

        let spec_source = if subsys_dir == "conops" {
            "[schema/conops/activities.sysml](../../schema/conops/activities.sysml)".to_string()
        } else {
            format!(
                "[schema/subsystems/{subsys_dir}/behavior.sysml](../../schema/subsystems/{subsys_dir}/behavior.sysml)",
                subsys_dir = subsys_dir
            )
        };

        let source_refs = if subsys_dir == "conops" {
            r#"- ConOps Operational Activities: `schema/conops/activities.sysml`
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Authoritative user story flows and interaction lifelines derive from `schema/conops/activities.sysml` pursuant to ISO/IEC/IEEE 29148."#.to_string()
        } else {
            format!(
                r#"- Subsystem Behavior Specification: `schema/subsystems/{subsys_dir}/behavior.sysml`
- ConOps Operational Activities: `schema/conops/activities.sysml`
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Authoritative user story flows and interaction lifelines derive from `schema/subsystems/{subsys_dir}/behavior.sysml` pursuant to ISO/IEC/IEEE 29148."#,
                subsys_dir = subsys_dir
            )
        };

        let content = format!(
            r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #{us_issue_id} |
| **Title** | {title} |
| **Type** | user-story |
| **Parent Epic** | {parent_epic_link} |
| **SysML Interaction** | `{interaction_name}` |
| **SysML Test Case** | `Test_{action_name}` |
| **Generation Mode** | subagent |
| **Specification Source** | {spec_source} |

# {title}

## UML Sequence Diagram

```mermaid
{seq_diagram}
```

## Acceptance Criteria (BDD)
- [ ] AC-US-{s_num:02}-01: Given operational environment initialized, When {action_name} executes, Then {engine_name} processes the AST within defined latency envelope.
- [ ] AC-US-{s_num:02}-02: Given formal constraints, When semantic validator executes, Then asserts invariant satisfaction.
- [ ] AC-US-{s_num:02}-03: Given exceptional conditions, When error recovery activates, Then graceful recovery occurs without data loss.

## Required Features
{req_features}

## Source References
Operational concept definitions and system sequence interaction models:
{source_refs}
"#,
            title = title,
            action_name = action_name,
            engine_name = engine_name,
            us_issue_id = us_issue_id,
            parent_epic_link = parent_epic_link,
            interaction_name = interaction_name,
            spec_source = spec_source,
            seq_diagram = seq_diagram,
            s_num = s_num,
            req_features = req_features,
            source_refs = source_refs,
        );

        generated_files.insert(rel_path, content);
    }

    // 4. Use Cases (UC-*.md) dynamically lowered from UseCaseDef nodes
    for uc_m in &uc_metas {
        let u_num = uc_m.u_num;
        let clean_name = strip_use_case_prefix(&uc_m.uc.name);
        let uc_slug = sanitize_id(clean_name);
        let rel_path = format!("use-cases/UC-{:02}-{}.md", u_num, uc_slug);
        let _uc_date = resolve_date(&target_out_dir, &rel_path, &today_iso);
        let uc_issue_id = resolve_issue_id(&target_out_dir, &rel_path, u_num + 94);
        let title = format!("Use Case {:02}: {}", u_num, to_display_name(clean_name));

        let parsed_doc = parse_use_case_doc(uc_m.uc.doc.as_deref());

        let actors: Vec<String> = if !parsed_doc.actors.is_empty() {
            parsed_doc.actors
        } else if !uc_m.uc.actors.is_empty() {
            uc_m.uc.actors.clone()
        } else if let Some(ref a) = uc_m.uc.actor {
            vec![a.clone()]
        } else {
            vec!["HumanEngineer".to_string()]
        };

        let primary_actor = actors.first().cloned().unwrap_or_else(|| "HumanEngineer".to_string());
        let _actors_yaml = actors
            .iter()
            .map(|a| format!("  - \"{}\"", a))
            .collect::<Vec<_>>()
            .join("\n");

        let subject = &uc_m.engine_name;

        let preconditions = if !parsed_doc.preconditions.is_empty() {
            parsed_doc
                .preconditions
                .iter()
                .map(|p| format!("- {}", p))
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            "- The compiler execution environment is initialized in nominal operational state.\n- Authoritative input schema files exist with read access."
                .to_string()
        };

        let trigger = if !parsed_doc.trigger.is_empty() {
            format!("- {}", parsed_doc.trigger)
        } else {
            format!("- Operational workflow command dispatched for {}.", to_display_name(clean_name))
        };

        let main_scenario = if !parsed_doc.main_scenario.is_empty() {
            parsed_doc.main_scenario.join("\n")
        } else {
            format!(
                "1. Actor `{}` initiates workflow execution.\n2. System `DEAPCompilerSystem` parses input model definitions and dispatches task to `{}`.\n3. Component `{}` processes structural elements and validates semantic constraints.\n4. System verifies invariant satisfaction and completes workflow execution.",
                primary_actor, subject, subject
            )
        };

        let alternate_flows = if !parsed_doc.alternate_flows.is_empty() {
            parsed_doc
                .alternate_flows
                .iter()
                .map(|f| if f.starts_with('-') { f.clone() } else { format!("- {}", f) })
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            "- 5a. Syntax or semantic parsing error detected:\n  - System captures error location and emits structured diagnostic error catalog entries.\n  - Execution halts gracefully without crashing or corrupting working tree state."
                .to_string()
        };

        let postconditions = if !parsed_doc.postconditions.is_empty() {
            parsed_doc
                .postconditions
                .iter()
                .map(|p| format!("- {}", p))
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            "- All semantic models and generated artifacts satisfy formal invariant requirements.\n- Output artifacts are deterministically emitted with bitwise reproducibility."
                .to_string()
        };

        // Realization Matrix links
        let feat_link_num = uc_m.engine_feat_num;
        let feat_part = &uc_m.engine_name;
        let feat_subsys = to_display_name(&uc_m.pkg_name);
        let feat_issue = resolve_issue_id(
            &target_out_dir,
            &format!("features/FEAT-{:02}-{}.md", feat_link_num, feat_part),
            feat_link_num + 9,
        );

        let matching_story = uc_m
            .story_s_num
            .and_then(|s_num| story_metas.iter().find(|s| s.s_num == s_num && s.pkg_name == uc_m.pkg_name));

        let us_row = if let Some(story) = matching_story {
            let us_link_num = story.s_num;
            let us_slug = sanitize_id(&story.action_name);
            let us_display = to_display_name(&story.action_name);
            let us_issue = resolve_issue_id(
                &target_out_dir,
                &format!("user-stories/US-{:02}-{}.md", us_link_num, us_slug),
                us_link_num + 88,
            );
            format!(
                "| **#{}** | User Story | [User Story {:02}: {}](../user-stories/US-{:02}-{}.md) |\n",
                us_issue, us_link_num, us_display, us_link_num, us_slug
            )
        } else {
            String::new()
        };

        let feat_row = format!(
            "| **#{}** | Feature | [Feature {:02}: [{}] {}](../features/FEAT-{:02}-{}.md) |\n",
            feat_issue, feat_link_num, feat_subsys, feat_part, feat_link_num, feat_part
        );

        let realization_matrix_table = format!(
            "| Specification Item | Type | Link / Reference |\n\
| :--- | :--- | :--- |\n\
{}{}",
            us_row, feat_row
        );

        let clean_title_display = to_display_name(clean_name);
        let uc_diagram = generate_use_case_diagram(&primary_actor, subject, &clean_title_display, &parsed_doc.main_scenario);

        let spec_source = if uc_m.subsys_dir == "conops" {
            "[schema/conops/activities.sysml](../../schema/conops/activities.sysml)".to_string()
        } else {
            format!(
                "[schema/subsystems/{subsys_dir}/behavior.sysml](../../schema/subsystems/{subsys_dir}/behavior.sysml)",
                subsys_dir = uc_m.subsys_dir
            )
        };

        let source_refs = if uc_m.subsys_dir == "conops" {
            r#"- ConOps Operational Activities: `schema/conops/activities.sysml`
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational use case realization and traceability links derive from `schema/conops/activities.sysml` pursuant to ISO/IEC/IEEE 29148."#.to_string()
        } else {
            format!(
                r#"- Subsystem Behavior Specification: `schema/subsystems/{subsys_dir}/behavior.sysml`
- ConOps Operational Activities: `schema/conops/activities.sysml`
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational use case realization and traceability links derive from `schema/subsystems/{subsys_dir}/behavior.sysml` pursuant to ISO/IEC/IEEE 29148."#,
                subsys_dir = uc_m.subsys_dir
            )
        };

        let content = format!(
            r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #{uc_issue_id} |
| **Title** | {title} |
| **Type** | use-case |
| **Primary Actor** | {primary_actor} |
| **Scope** | {subject} |
| **Level** | User Goal |
| **Generation Mode** | subagent |
| **Specification Source** | {spec_source} |

# {title}

## UML Diagrams

```mermaid
{uc_diagram}
```

## 1. Actors
- `{primary_actor}`

## 2. Preconditions
{preconditions}

## 3. Trigger
{trigger}

## 4. Main Success Scenario
{main_scenario}

## 5. Alternate and Exception Flows
{alternate_flows}

## 6. Postconditions
{postconditions}

## 8. Realization Matrix
{realization_matrix_table}
## Source References
Use case operational flows and lifecycle activity references:
{source_refs}
"#,
            title = title,
            subject = subject,
            uc_issue_id = uc_issue_id,
            primary_actor = primary_actor,
            spec_source = spec_source,
            uc_diagram = uc_diagram,
            preconditions = preconditions,
            trigger = trigger,
            main_scenario = main_scenario,
            alternate_flows = alternate_flows,
            postconditions = postconditions,
            realization_matrix_table = realization_matrix_table,
            source_refs = source_refs,
        );

        generated_files.insert(rel_path, content);
    }

    // 5. STPA Matrix (safety/STPA_MATRIX.md)
    let ucas = expand_cartesian_stpa(pkg);
    let all_constraints = pkg.get_all_constraints();

    let mut sc_table = String::new();
    sc_table.push_str("| SC ID | Constraint Statement / Description | Controller / Subsystem | Traceability / UCA |\n");
    sc_table.push_str("| :--- | :--- | :--- | :--- |\n");

    for (idx, c) in all_constraints.iter().enumerate() {
        let sc_id = format!("SC-{:02}", idx + 1);
        let desc = c.doc.as_deref().filter(|d| !d.is_empty()).unwrap_or(&c.expression);
        let desc_clean = if desc.is_empty() { &c.name } else { desc }.replace('|', "\\|");
        let uca_trace = ucas
            .iter()
            .find(|u| u.constraint == sc_id || u.constraint == c.name)
            .map(|u| u.id.as_str())
            .unwrap_or("-");
        let controller = ucas
            .iter()
            .find(|u| u.constraint == sc_id || u.constraint == c.name)
            .map(|u| u.controller.as_str())
            .unwrap_or("-");
        sc_table.push_str(&format!(
            "| **{}** | {} | {} | {} |\n",
            sc_id, desc_clean, controller, uca_trace
        ));
    }

    for u in &ucas {
        if !all_constraints.iter().any(|c| c.name == u.constraint) {
            sc_table.push_str(&format!(
                "| **{}** | System shall prevent {} during {} | {} | {} |\n",
                u.constraint, u.guide_word, u.control_action, u.controller, u.id
            ));
        }
    }

    let mut uca_table = String::new();
    uca_table.push_str("| UCA ID | Controller | Control Action | Guide Word | Hazard | Safety Constraint |\n");
    uca_table.push_str("| :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n");
    for u in &ucas {
        uca_table.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            u.id, u.controller, u.control_action, u.guide_word, u.hazard, u.constraint
        ));
    }

    let stpa_content = format!(
        r#"# STPA Safety & Failure Mode Tracking Matrix

## 1. Formal Safety Constraints
{sc_table}
## 2. STPA Unsafe Control Actions (UCA) Matrix
{uca_table}
"#
    );
    generated_files.insert("safety/STPA_MATRIX.md".to_string(), stpa_content);

    // If not dry-run, remove stale files when force is enabled, then persist generated files
    if !opts.dry_run {
        if opts.force {
            clean_stale_specifications(&target_out_dir, &generated_files)?;
        }

        for (rel_path, text) in &generated_files {
            let full_dest = target_out_dir.join(rel_path);
            if let Some(parent) = full_dest.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    format!("Failed to create directory '{}': {}", parent.display(), e)
                })?;
            }
            write_atomic(&full_dest, text).map_err(|e| e.to_string())?;
        }
        println!(
            "[SysML v2 Forward-Sync] Successfully synchronized {} specifications to '{}'",
            generated_files.len(),
            target_out_dir.display()
        );
    } else {
        println!(
            "[SysML v2 Forward-Sync] [DRY RUN] Would generate {} specifications in '{}'",
            generated_files.len(),
            target_out_dir.display()
        );
    }

    Ok(generated_files)
}

/// Removes stale or obsolete generated specification files that are no longer part of the AST.
fn clean_stale_specifications(
    target_out_dir: &Path,
    active_specs: &HashMap<String, String>,
) -> Result<(), String> {
    let subdirs = ["epics", "features", "user-stories", "use-cases"];
    let active_keys: HashSet<String> = active_specs.keys().cloned().collect();

    for subdir in &subdirs {
        let dir_path = target_out_dir.join(subdir);
        if !dir_path.is_dir() {
            continue;
        }

        for entry in WalkDir::new(&dir_path).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            let file_path = entry.path();
            if file_path.extension().map(|e| e == "md").unwrap_or(false) {
                if let Ok(rel) = file_path.strip_prefix(target_out_dir) {
                    let rel_str = rel.to_string_lossy().replace('\\', "/");
                    if !active_keys.contains(&rel_str) {
                        let _ = fs::remove_file(file_path);
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use deap_core::sysml_ast::PartDef;

    #[test]
    fn test_forward_sync_generates_expected_files() {
        let mut pkg = PackageDef::default();
        pkg.name = "TestSystem".to_string();

        let mut p1 = PartDef::default();
        p1.name = "Airframe".to_string();
        pkg.part_defs.push(p1);

        let opts = ForwardSyncOptions {
            schema_path: PathBuf::from("schema/test.sysml"),
            docs_dir: PathBuf::from("docs"),
            out_dir: None,
            dry_run: true,
            force: true,
        };

        let generated = forward_sync_sysml_to_specs(&pkg, &opts).unwrap();
        assert!(generated.keys().any(|k| k.starts_with("epics/")));
        assert!(generated.keys().any(|k| k.starts_with("features/")));
        assert!(generated.contains_key("safety/STPA_MATRIX.md"));
    }

    #[test]
    fn test_recursive_package_traversal_picks_up_subpackage_elements() {
        let mut root = PackageDef {
            name: "DEAP_Compiler_System".to_string(),
            ..Default::default()
        };
        let mut sub = PackageDef {
            name: "Subsystem_Alpha".to_string(),
            ..Default::default()
        };
        sub.part_defs.push(PartDef {
            name: "AlphaPart".to_string(),
            ..Default::default()
        });
        sub.use_case_defs.push(deap_core::sysml_ast::UseCaseDef {
            name: "AlphaUseCase".to_string(),
            subject: Some("AlphaPart".to_string()),
            ..Default::default()
        });
        sub.action_defs.push(deap_core::sysml_ast::ActionDef {
            name: "AlphaAction".to_string(),
            ..Default::default()
        });
        root.packages.push(sub);

        let opts = ForwardSyncOptions {
            schema_path: PathBuf::from("schema/test.sysml"),
            docs_dir: PathBuf::from("docs"),
            out_dir: None,
            dry_run: true,
            force: true,
        };

        let generated = forward_sync_sysml_to_specs(&root, &opts).unwrap();
        assert!(generated.keys().any(|k| k.contains("Subsystem_Alpha")), "Subpackage must generate Epic");
        assert!(generated.keys().any(|k| k.contains("AlphaPart")), "Subpackage part must generate Feature");
        assert!(generated.keys().any(|k| k.contains("AlphaUseCase")), "Subpackage use case must generate Use Case");
        assert!(generated.keys().any(|k| k.contains("AlphaAction")), "Subpackage action must generate User Story");
    }

    #[test]
    fn test_zero_hardcoded_domain_strings_exist_in_output_specifications() {
        let mut pkg = PackageDef {
            name: "GenericCompiler".to_string(),
            ..Default::default()
        };
        let mut sub = PackageDef {
            name: "ParserSubsystem".to_string(),
            ..Default::default()
        };
        sub.part_defs.push(PartDef {
            name: "LexerEngine".to_string(),
            ..Default::default()
        });
        pkg.packages.push(sub);

        let opts = ForwardSyncOptions {
            schema_path: PathBuf::from("schema/test.sysml"),
            docs_dir: PathBuf::from("docs"),
            out_dir: None,
            dry_run: true,
            force: true,
        };

        let generated = forward_sync_sysml_to_specs(&pkg, &opts).unwrap();
        let forbidden = [
            "CoreController",
            "TelemetrySync",
            "ExecuteAutonomousMission",
            "HandleSafetyFailsafe",
            "SensorSuite",
            "OperatorConsole",
            "SafetyWatchdog",
            "flight parameters",
        ];

        for (path, content) in &generated {
            for bad in &forbidden {
                assert!(
                    !content.contains(bad),
                    "File '{}' contains forbidden domain string '{}'",
                    path,
                    bad
                );
            }
        }
    }

    #[test]
    fn test_empty_collections_do_not_emit_phantom_specifications() {
        let mut pkg = PackageDef {
            name: "CleanSystem".to_string(),
            ..Default::default()
        };
        pkg.part_defs.push(PartDef {
            name: "CleanPart".to_string(),
            ..Default::default()
        });

        let opts = ForwardSyncOptions {
            schema_path: PathBuf::from("schema/test.sysml"),
            docs_dir: PathBuf::from("docs"),
            out_dir: None,
            dry_run: true,
            force: true,
        };

        let generated = forward_sync_sysml_to_specs(&pkg, &opts).unwrap();
        assert!(
            !generated.keys().any(|k| k.starts_with("use-cases/")),
            "Empty use cases must not emit phantom use-case files"
        );
        assert!(
            !generated.keys().any(|k| k.starts_with("user-stories/")),
            "Empty interactions must not emit phantom user-story files"
        );
    }

    #[test]
    fn test_use_case_realization_matrix_traceability_alignment() {
        let mut root = PackageDef {
            name: "DEAP_Compiler_System".to_string(),
            ..Default::default()
        };

        // Subsystem Alpha has Engine, Action, and Use Case
        let mut sub_alpha = PackageDef {
            name: "Subsystem_Alpha".to_string(),
            ..Default::default()
        };
        sub_alpha.part_defs.push(PartDef {
            name: "AlphaEngine".to_string(),
            ..Default::default()
        });
        sub_alpha.action_defs.push(deap_core::sysml_ast::ActionDef {
            name: "AlphaAction".to_string(),
            ..Default::default()
        });
        sub_alpha.use_case_defs.push(UseCaseDef {
            name: "UC_Alpha_Workflow".to_string(),
            ..Default::default()
        });

        // Subsystem Beta has Engine, Action, and Use Case
        let mut sub_beta = PackageDef {
            name: "Subsystem_Beta".to_string(),
            ..Default::default()
        };
        sub_beta.part_defs.push(PartDef {
            name: "BetaEngine".to_string(),
            ..Default::default()
        });
        sub_beta.action_defs.push(deap_core::sysml_ast::ActionDef {
            name: "BetaAction".to_string(),
            ..Default::default()
        });
        sub_beta.use_case_defs.push(UseCaseDef {
            name: "UC_Beta_Workflow".to_string(),
            ..Default::default()
        });

        // Subsystem Gamma has Engine and Use Case, but NO Action (tests zero cross-wiring when subsystem has no user story)
        let mut sub_gamma = PackageDef {
            name: "Subsystem_Gamma".to_string(),
            ..Default::default()
        };
        sub_gamma.part_defs.push(PartDef {
            name: "GammaEngine".to_string(),
            ..Default::default()
        });
        sub_gamma.use_case_defs.push(UseCaseDef {
            name: "UC_Gamma_Workflow".to_string(),
            ..Default::default()
        });

        root.packages.push(sub_alpha);
        root.packages.push(sub_beta);
        root.packages.push(sub_gamma);

        let opts = ForwardSyncOptions {
            schema_path: PathBuf::from("schema/test.sysml"),
            docs_dir: PathBuf::from("docs"),
            out_dir: None,
            dry_run: true,
            force: true,
        };

        let generated = forward_sync_sysml_to_specs(&root, &opts).unwrap();

        let alpha_uc = generated
            .iter()
            .find(|(k, _)| k.contains("UC_Alpha_Workflow"))
            .map(|(_, v)| v)
            .expect("Alpha UC must be generated");
        let beta_uc = generated
            .iter()
            .find(|(k, _)| k.contains("UC_Beta_Workflow"))
            .map(|(_, v)| v)
            .expect("Beta UC must be generated");
        let gamma_uc = generated
            .iter()
            .find(|(k, _)| k.contains("UC_Gamma_Workflow"))
            .map(|(_, v)| v)
            .expect("Gamma UC must be generated");

        // Alpha UC checks
        assert!(alpha_uc.contains("AlphaAction"), "Alpha UC must link to AlphaAction user story");
        assert!(alpha_uc.contains("AlphaEngine"), "Alpha UC must link to AlphaEngine feature");
        assert!(!alpha_uc.contains("BetaAction"), "Alpha UC must NOT link to BetaAction");
        assert!(!alpha_uc.contains("BetaEngine"), "Alpha UC must NOT link to BetaEngine");

        // Beta UC checks
        assert!(beta_uc.contains("BetaAction"), "Beta UC must link to BetaAction user story");
        assert!(beta_uc.contains("BetaEngine"), "Beta UC must link to BetaEngine feature");
        assert!(!beta_uc.contains("AlphaAction"), "Beta UC must NOT link to AlphaAction");
        assert!(!beta_uc.contains("AlphaEngine"), "Beta UC must NOT link to AlphaEngine");

        // Gamma UC checks: Gamma has no ActionDef, so its Realization Matrix MUST NOT contain AlphaAction or BetaAction
        assert!(gamma_uc.contains("GammaEngine"), "Gamma UC must link to GammaEngine feature");
        assert!(!gamma_uc.contains("AlphaAction"), "Gamma UC must NOT cross-wire to AlphaAction");
        assert!(!gamma_uc.contains("BetaAction"), "Gamma UC must NOT cross-wire to BetaAction");
    }

    #[test]
    fn test_feature_bdd_derived_from_constraints_and_requirements() {
        use deap_core::sysml_ast::{ConstraintDef, RequirementDef};

        let mut part = PartDef::default();
        part.name = "TelemetryProcessor".to_string();
        part.constraints.push(ConstraintDef {
            name: "FrameRateLimit".to_string(),
            expression: "frame_rate <= 60".to_string(),
            ..Default::default()
        });

        let req = RequirementDef {
            name: "LatencyBoundRequirement".to_string(),
            req_id: "REQ-0100".to_string(),
            constraints: vec![ConstraintDef {
                name: "MaxLatencyInvariant".to_string(),
                expression: "latency_ms < 5".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };

        let bdd = generate_feature_bdd(&part, &[&req]);

        // Pattern A: Nominal domain state / signal handling
        assert!(
            bdd.contains("Given nominal TelemetryProcessor domain state, When command or input signals are received, Then updates internal state and signal outputs within declared bounds."),
            "Must contain Pattern A nominal domain state criterion, got:\n{}",
            bdd
        );

        // Pattern B: Invariant / constraint verification
        assert!(
            bdd.contains("Given operational state, When TelemetryProcessor evaluates constraint 'FrameRateLimit', Then asserts invariant satisfaction without violation."),
            "Must contain part constraint FrameRateLimit in Pattern B, got:\n{}",
            bdd
        );
        assert!(
            bdd.contains("Given operational state, When TelemetryProcessor evaluates constraint 'MaxLatencyInvariant', Then asserts invariant satisfaction without violation."),
            "Must contain requirement constraint MaxLatencyInvariant in Pattern B, got:\n{}",
            bdd
        );

        // Pattern C: Failsafe / negative error recovery
        assert!(
            bdd.contains("Given malformed input signals or boundary condition violation, When TelemetryProcessor error handling is invoked, Then transitions safely to failsafe fallback state without corruption."),
            "Must contain Pattern C failsafe criterion, got:\n{}",
            bdd
        );

        // Granularity bounds check (3 to 10 criteria)
        let lines: Vec<&str> = bdd.lines().filter(|l| l.trim().starts_with("- [ ] AC-")).collect();
        assert!(
            lines.len() >= 3 && lines.len() <= 10,
            "Criteria count must be between 3 and 10, got: {}",
            lines.len()
        );
    }

    #[test]
    fn test_strip_use_case_prefix_variations() {
        assert_eq!(
            strip_use_case_prefix("UC_01_System_Vision_and_Foundational_Invariants"),
            "System_Vision_and_Foundational_Invariants"
        );
        assert_eq!(
            strip_use_case_prefix("UC-02-Universal_Schema_Ingestion_Workflow"),
            "Universal_Schema_Ingestion_Workflow"
        );
        assert_eq!(
            strip_use_case_prefix("UC03_Node_Arena_Memory_Management_Workflow"),
            "Node_Arena_Memory_Management_Workflow"
        );
        assert_eq!(
            strip_use_case_prefix("UC_Alpha_Workflow"),
            "UC_Alpha_Workflow"
        );
    }

    #[test]
    fn test_generate_use_case_diagram_model_integrity() {
        let scenario = vec![
            "1. Actor initiates task".to_string(),
            "2. Engine validates model".to_string(),
        ];
        let diag = generate_use_case_diagram(
            "CIContinuousIntegrationRunner",
            "StateMachineSolverEngine",
            "State Machine Solver Workflow",
            &scenario,
        );

        assert!(diag.contains("graph TD"));
        assert!(diag.contains("subgraph \"StateMachineSolverEngine System Boundary\""));
        assert!(diag.contains("UC_Main([\"State Machine Solver Workflow\"])"));
        assert!(diag.contains("UC_Step01([\"Actor initiates task\"])"));
        assert!(diag.contains("UC_Step02([\"Engine validates model\"])"));
        assert!(diag.contains("end"));
        assert!(diag.contains("CIContinuousIntegrationRunner((\"CIContinuousIntegrationRunner\")) --- UC_Main"));
        assert!(diag.contains("UC_Main -. \"&lt;&lt;include&gt;&gt;\" .-> UC_Step01"));
    }

    #[test]
    fn test_epic_statechart_and_class_diagram_generation() {
        use deap_core::sysml_ast::{AttributeDef, PortDef, StateDef, TransitionDef};

        let mut pkg = PackageDef {
            name: "SpatioTemporalStateSolvers".to_string(),
            ..Default::default()
        };
        let mut st_def = StateDef {
            name: "StateSolverExecutionStatechart".to_string(),
            ..Default::default()
        };
        st_def.transitions.push(TransitionDef {
            name: Some("InitialToIngest first entry then IngestingStates".to_string()),
            ..Default::default()
        });
        st_def.transitions.push(TransitionDef {
            name: Some("IngestToSolve first IngestingStates then Solved".to_string()),
            ..Default::default()
        });
        pkg.state_defs.push(st_def);

        let mut engine = PartDef {
            name: "StateMachineSolverEngine".to_string(),
            ..Default::default()
        };
        engine.ports.push(PortDef {
            name: "stream_in".to_string(),
            direction: "in".to_string(),
            type_name: "ASTStreamPort".to_string(),
            ..Default::default()
        });
        engine.attributes.push(AttributeDef {
            name: "recursion_depth".to_string(),
            type_name: "Integer".to_string(),
            ..Default::default()
        });
        pkg.part_defs.push(engine);

        let helper = PartDef {
            name: "StateHierarchyFlattener".to_string(),
            ..Default::default()
        };
        pkg.part_defs.push(helper);

        let statechart = generate_epic_statechart(&pkg);
        assert!(statechart.contains("stateDiagram-v2"));
        assert!(statechart.contains("[*] --> IngestingStates"));
        assert!(statechart.contains("IngestingStates --> Solved"));
        assert!(statechart.contains("Solved --> [*]"));

        let class_diag = generate_epic_class_diagram(&pkg, "StateMachineSolverEngine", false);
        assert!(class_diag.contains("classDiagram"));
        assert!(class_diag.contains("class StateMachineSolverEngine {"));
        assert!(class_diag.contains("+ASTStreamPort stream_in (in)"));
        assert!(class_diag.contains("+Integer recursion_depth"));
        assert!(class_diag.contains("DEAPCompilerSystem --> StateMachineSolverEngine : contains"));
        assert!(class_diag.contains("StateMachineSolverEngine --> StateHierarchyFlattener : coordinates"));
    }

    #[test]
    fn test_user_story_subsystem_parent_epic_and_dynamic_spec_source() {
        use deap_core::sysml_ast::{ActionDef, InteractionDef};

        let mut root = PackageDef {
            name: "DEAP_Compiler_System".to_string(),
            ..Default::default()
        };
        let mut sub = PackageDef {
            name: "SpatioTemporalStateSolvers".to_string(),
            ..Default::default()
        };
        sub.part_defs.push(PartDef {
            name: "StateMachineSolverEngine".to_string(),
            ..Default::default()
        });
        sub.action_defs.push(ActionDef {
            name: "SolveStateDynamicsAction".to_string(),
            ..Default::default()
        });
        sub.interaction_defs.push(InteractionDef {
            name: "StateSolverInteraction".to_string(),
            lifelines: vec![
                "grammar : GrammarLoweringEngine".to_string(),
                "solver : StateMachineSolverEngine".to_string(),
            ],
            messages: vec![
                "SubmitModel from grammar to solver".to_string(),
                "StateOutput from solver to grammar".to_string(),
            ],
            ..Default::default()
        });
        root.packages.push(sub);

        let opts = ForwardSyncOptions {
            schema_path: PathBuf::from("schema/test.sysml"),
            docs_dir: PathBuf::from("docs"),
            out_dir: None,
            dry_run: true,
            force: true,
        };

        let generated = forward_sync_sysml_to_specs(&root, &opts).unwrap();
        let us = generated
            .iter()
            .find(|(k, _)| k.contains("US-") && k.contains("SolveStateDynamicsAction"))
            .map(|(_, v)| v)
            .expect("User story must be generated");

        assert!(
            us.contains("[schema/subsystems/spatio_temporal_state_solvers/behavior.sysml]"),
            "Specification Source must bind to behavior.sysml"
        );
        assert!(
            us.contains("GrammarLoweringEngine->>StateMachineSolverEngine: SolveStateDynamicsAction()"),
            "Sequence diagram must use interaction lifelines"
        );
        assert!(
            us.contains("StateMachineSolverEngine-->>GrammarLoweringEngine: StateOutput"),
            "Sequence diagram must return reply message from interaction def"
        );
        assert!(
            us.contains("`StateSolverInteraction`"),
            "SysML Interaction must match interaction name"
        );
    }
}


