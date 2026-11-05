//! Behavioral AST extraction (`ActionDef`, `InteractionDef`, `StateDef`, `UseCaseDef`) from Markdown AST and Mermaid blocks.

use crate::table::sanitize_identifier;
use deap_core::markdown_ast::parse_markdown_ast;
use deap_core::sysml_ast::{ActionDef, AttributeDef, InteractionDef, PackageDef, PartDef, PortDef, RequirementDef, StateDef, TransitionDef, UseCaseDef};
use super::md_common::{clean_bdd_text, Frontmatter};
use super::md_requirements::{extract_acceptance_criteria, extract_formal_invariants, extract_normative_statement, extract_req_metadata, extract_requirement_cross_references};
use super::md_tables::extract_prose_use_cases;

/// Extracts BOM bullet subcomponent items (`- <Ident> (Quantity <N>): <Desc>`) into `PartDef` AST nodes.
/// Realises: [REQ-SYSML-INGEST-MD/extract_bom_bullet_parts]
pub fn extract_bom_bullet_parts(content: &str, pkg: &mut PackageDef) {
    let ast = parse_markdown_ast(content);
    let base = content.as_ptr() as usize;
    for line in content.lines() {
        let off = (line.as_ptr() as usize).saturating_sub(base);
        if ast.code_blocks.iter().any(|cb| (cb.start_offset..cb.end_offset).contains(&off)) {
            continue;
        }
        let trimmed = line.trim();
        let Some(bullet) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) else { continue };
        let Some((raw_name, after_qty)) = bullet.split_once("(Quantity ").or_else(|| bullet.split_once("(quantity ")) else { continue };
        let Some((qty_raw, after_paren)) = after_qty.split_once(')') else { continue };
        let clean_name = raw_name.trim().trim_matches(['*', '`', '_', ' ']);
        let qty_digits: String = qty_raw.chars().filter(|c| c.is_ascii_digit()).collect();
        if clean_name.is_empty() || qty_digits.is_empty() {
            continue;
        }
        let part_name = sanitize_identifier(clean_name, "Subcomponent");
        let raw_desc = after_paren.trim_start().strip_prefix([':', '-']).unwrap_or(after_paren).trim();
        let desc_clean = clean_bdd_text(raw_desc);
        if !pkg.part_defs.iter().any(|p| p.name == part_name) {
            pkg.part_defs.push(PartDef {
                name: part_name,
                doc: (!desc_clean.is_empty()).then_some(desc_clean),
                is_def: true,
                attributes: vec![AttributeDef { name: "quantity".to_string(), type_name: "Integer".to_string(), default_value: Some(qty_digits), doc: None }],
                ports: [("cmd_in", "in"), ("status_out", "out")]
                    .into_iter()
                    .map(|(n, d)| PortDef { name: n.to_string(), type_name: "String".to_string(), direction: d.to_string(), port_category: "DataPort".to_string(), ..Default::default() })
                    .collect(),
                ..Default::default()
            });
        }
    }
}

/// Strips ingested user-story action prefixes (`US_AC_<N>_` or `US_<N>_`) from `raw`. Realises: [REQ-INGEST-MD-BEHAVIORAL]
fn strip_ingested_action_prefix(raw: &str) -> &str {
    raw.strip_prefix("US_AC_")
        .or_else(|| raw.strip_prefix("US_"))
        .and_then(|after| after.split_once('_'))
        .filter(|(d, r)| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()) && !r.is_empty())
        .map_or(raw, |(_, rest)| rest)
}

/// Splits `s` on the first single `:` character that is not part of a `::` path separator. Realises: [REQ-INGEST-MD-BEHAVIORAL]
fn split_once_single_colon(s: &str) -> Option<(&str, &str)> {
    let bytes = s.as_bytes();
    for (idx, &b) in bytes.iter().enumerate() {
        if b == b':'
            && !idx.checked_sub(1).and_then(|p| bytes.get(p)).is_some_and(|&p| p == b':')
            && !bytes.get(idx + 1).is_some_and(|&n| n == b':')
        {
            return Some((s.get(..idx)?, s.get(idx + 1..)?));
        }
    }
    None
}

/// Matches a Mermaid sequence arrow in `line` and returns `(sender, rhs, is_reply)`. Realises: [REQ-INGEST-MD-BEHAVIORAL]
fn match_sequence_arrow(line: &str) -> Option<(&str, &str, bool)> {
    const ARROWS: [(&str, bool); 8] = [("-->>", true), ("->>", false), ("-->", true), ("->", false), ("--x", true), ("-x", false), ("--)", true), ("-)", false)];
    let header = split_once_single_colon(line).map_or(line, |(hdr, _)| hdr);
    for idx in 0..header.len() {
        let Some(tail) = header.get(idx..) else { continue };
        for (arrow, is_reply) in ARROWS {
            if tail.starts_with(arrow) {
                return Some((line.get(..idx)?, line.get(idx + arrow.len()..)?, is_reply));
            }
        }
    }
    None
}

/// Synthesizes a sanitized PascalCase `ActionDef` identifier from a Mermaid sequence message label. Realises: [REQ-INGEST-MD-BEHAVIORAL]
fn synthesize_sequence_action_name(msg_clean: &str) -> String {
    const DELIMS: [char; 7] = ['`', '"', '\'', '*', '[', ']', ','];
    let before_p = msg_clean.split_once('(').map_or(msg_clean, |(p, _)| p);
    let before_c = split_once_single_colon(before_p).map_or(before_p, |(p, _)| p);
    let head = strip_ingested_action_prefix(before_c.trim().trim_matches(DELIMS).trim());
    if head.is_empty() {
        return "ExecuteAction".to_string();
    }
    if !head.contains(char::is_whitespace) {
        return sanitize_identifier(&head.replace("::", "_"), "ExecuteAction");
    }
    let (mut pascal, mut count) = (String::new(), 0usize);
    'outer: for word in head.split_whitespace() {
        let stripped = word.trim_matches(DELIMS).trim_start_matches('-').trim_matches(DELIMS);
        for seg in stripped.split(|c: char| !c.is_ascii_alphanumeric()).filter(|s| !s.is_empty()) {
            let low_tail = seg.chars().any(|c| c.is_ascii_uppercase()) && !seg.chars().any(|c| c.is_ascii_lowercase());
            let mut chars = seg.chars();
            if let Some(first) = chars.next() {
                pascal.push(first.to_ascii_uppercase());
                for ch in chars {
                    pascal.push(if low_tail { ch.to_ascii_lowercase() } else { ch });
                }
            }
            count += 1;
            if count >= 6 {
                break 'outer;
            }
        }
    }
    sanitize_identifier(&pascal, "ExecuteAction")
}

/// Extracts a single Mermaid `sequenceDiagram` participant or arrow statement into `lifelines`, `messages`, and `pkg.action_defs`.
/// Realises: [REQ-0603]
fn parse_mermaid_sequence_line(trimmed: &str, lifelines: &mut Vec<String>, messages: &mut Vec<String>, pkg: &mut PackageDef) {
    if let Some(rest) = trimmed.strip_prefix("participant ").or_else(|| trimmed.strip_prefix("actor ")) {
        let (a, b) = rest.split_once(" as ").unwrap_or((rest, rest));
        let lf = format!("{} : {}", sanitize_identifier(a.trim(), "Participant"), sanitize_identifier(b.trim(), "Actor"));
        if !lifelines.contains(&lf) {
            lifelines.push(lf);
        }
        return;
    }
    if ["Note ", "note ", "loop ", "alt ", "else", "opt ", "par ", "rect ", "activate ", "deactivate ", "autonumber", "end"].iter().any(|kw| trimmed.starts_with(kw)) {
        return;
    }
    if let Some((lhs, rhs, is_reply)) = match_sequence_arrow(trimmed) {
        let sender = sanitize_identifier(lhs.trim().trim_end_matches(['+', '-']).trim(), "Sender");
        let (r_raw, m_raw) = split_once_single_colon(rhs).unwrap_or((rhs, "SendMessage"));
        let receiver = sanitize_identifier(r_raw.trim().trim_start_matches(['+', '-']).trim(), "Receiver");
        let msg_clean = m_raw.trim().trim_matches(['"', '`']).trim();
        let act = strip_ingested_action_prefix(&synthesize_sequence_action_name(msg_clean)).to_string();
        let stmt = if is_reply {
            format!("{} from {sender} to {receiver}", sanitize_identifier(&format!("reply_{act}"), "reply_payload"))
        } else {
            let doc = clean_bdd_text(msg_clean);
            if !pkg.action_defs.iter().any(|a| a.name == act) {
                pkg.action_defs.push(ActionDef { name: act.clone(), doc: (!doc.is_empty()).then_some(doc), is_def: true, ..Default::default() });
            }
            format!("{act} from {sender} to {receiver}")
        };
        if !messages.contains(&stmt) {
            messages.push(stmt);
        }
    }
}

/// Extracts a single Mermaid `stateDiagram` transition line (`A --> B : trigger`) into `transitions`.
/// Realises: [REQ-0603]
fn parse_mermaid_state_line(trimmed: &str, transitions: &mut Vec<TransitionDef>) {
    if let Some((lhs, rhs)) = trimmed.split_once("-->") {
        let (tgt_raw, trig_raw) = rhs.split_once(':').map_or((rhs.trim(), None), |(t, tr)| (t.trim(), Some(tr.trim())));
        let src_id = if lhs.trim() == "[*]" { "Initial".to_string() } else { sanitize_identifier(lhs.trim(), "State") };
        let tgt_id = if tgt_raw == "[*]" { "Terminal".to_string() } else { sanitize_identifier(tgt_raw, "State") };
        let t_name = trig_raw.filter(|s| !s.is_empty()).map_or_else(
            || sanitize_identifier(&format!("{src_id}_to_{tgt_id}"), "Transition"),
            |tr| sanitize_identifier(&format!("{src_id}_to_{tgt_id}_on_{}", sanitize_identifier(tr, "Trigger")), "Transition"),
        );
        if !transitions.iter().any(|t: &TransitionDef| t.name.as_deref() == Some(t_name.as_str())) {
            transitions.push(TransitionDef { name: Some(t_name), source: Some(src_id), target: Some(tgt_id), trigger: trig_raw.map(|s| sanitize_identifier(s, "Trigger")), ..Default::default() });
        }
    }
}

/// Parses one fenced `mermaid` block and appends extracted `InteractionDef` or `StateDef` nodes to `pkg`.
/// Realises: [REQ-0603]
fn ingest_mermaid_block(code: &str, seq_counter: &mut usize, state_counter: &mut usize, pkg: &mut PackageDef) {
    let mut mode = "";
    let (mut lifelines, mut messages, mut transitions) = (Vec::new(), Vec::new(), Vec::new());
    for line in code.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("%%") {
            continue;
        }
        if mode.is_empty() {
            mode = if trimmed.starts_with("sequenceDiagram") { "sequence" } else if trimmed.starts_with("stateDiagram") { "state" } else { "other" };
            continue;
        }
        if mode == "sequence" {
            parse_mermaid_sequence_line(trimmed, &mut lifelines, &mut messages, pkg);
        } else if mode == "state" {
            parse_mermaid_state_line(trimmed, &mut transitions);
        }
    }
    if mode == "sequence" && (!lifelines.is_empty() || !messages.is_empty()) {
        *seq_counter += 1;
        let raw = if *seq_counter == 1 { format!("{}_SequenceInteraction", pkg.name) } else { format!("{}_SequenceInteraction_{seq_counter}", pkg.name) };
        let name = sanitize_identifier(&raw, "SequenceInteraction");
        if !pkg.interaction_defs.iter().any(|i| i.name == name) {
            pkg.interaction_defs.push(InteractionDef { name, lifelines, messages, triggers: Vec::new(), doc: Some("Extracted from Mermaid sequenceDiagram specification.".to_string()) });
        }
    } else if mode == "state" && !transitions.is_empty() {
        *state_counter += 1;
        let raw = if *state_counter == 1 { format!("{}_LifecycleState", pkg.name) } else { format!("{}_LifecycleState_{state_counter}", pkg.name) };
        let name = sanitize_identifier(&raw, "LifecycleState");
        if !pkg.state_defs.iter().any(|s| s.name == name) {
            let mut sub_states = Vec::new();
            for tr in &transitions {
                for st_id in [tr.source.as_deref(), tr.target.as_deref()].into_iter().flatten() {
                    if !matches!(st_id, "Initial" | "Terminal" | "") && !sub_states.iter().any(|s: &StateDef| s.name == st_id) {
                        sub_states.push(StateDef { name: st_id.to_string(), ..Default::default() });
                    }
                }
            }
            pkg.state_defs.push(StateDef { name, doc: Some("Extracted from Mermaid stateDiagram specification.".to_string()), sub_states, transitions, ..Default::default() });
        }
    }
}

/// Synthesizes a deterministic package lifecycle `StateDef` when no Mermaid `stateDiagram` is present.
/// Realises: [REQ-0603]
fn ensure_default_lifecycle_state(pkg: &mut PackageDef) {
    if !pkg.state_defs.is_empty() {
        return;
    }
    let mut child_names: Vec<String> = pkg.part_defs.iter().take(3).map(|p| sanitize_identifier(&format!("{}_State", p.name), "State")).collect();
    for sfx in ["InitState", "ActiveState", "VerifiedState"] {
        if child_names.len() < 3 {
            let cand = sanitize_identifier(&format!("{}_{sfx}", pkg.name), "State");
            if !child_names.contains(&cand) {
                child_names.push(cand);
            }
        }
    }
    let s1 = child_names.first().cloned().unwrap_or_else(|| "InitState".to_string());
    let s2 = child_names.get(1).cloned().unwrap_or_else(|| "ActiveState".to_string());
    let s3 = child_names.get(2).cloned().unwrap_or_else(|| "VerifiedState".to_string());
    let trig1 = pkg.action_defs.first().map_or_else(|| sanitize_identifier(&format!("Initialize_{}", pkg.name), "Action"), |a| a.name.clone());
    let trig2 = pkg.action_defs.get(1).or_else(|| pkg.action_defs.first()).map_or_else(|| sanitize_identifier(&format!("Verify_{}", pkg.name), "Action"), |a| a.name.clone());
    pkg.state_defs.push(StateDef {
        name: sanitize_identifier(&format!("{}_LifecycleState", pkg.name), "LifecycleState"),
        doc: Some("Synthesized deterministic package lifecycle state machine.".to_string()),
        sub_states: [s1.clone(), s2.clone(), s3.clone()].into_iter().map(|name| StateDef { name, ..Default::default() }).collect(),
        transitions: vec![
            TransitionDef { name: Some(sanitize_identifier(&format!("Initial_to_{s1}"), "Transition")), source: Some("Initial".to_string()), target: Some(s1.clone()), ..Default::default() },
            TransitionDef { name: Some(sanitize_identifier(&format!("{s1}_to_{s2}_on_{trig1}"), "Transition")), source: Some(s1), target: Some(s2.clone()), trigger: Some(trig1), ..Default::default() },
            TransitionDef { name: Some(sanitize_identifier(&format!("{s2}_to_{s3}_on_{trig2}"), "Transition")), source: Some(s2), target: Some(s3.clone()), trigger: Some(trig2), ..Default::default() },
            TransitionDef { name: Some(sanitize_identifier(&format!("{s3}_to_Terminal"), "Transition")), source: Some(s3), target: Some("Terminal".to_string()), ..Default::default() },
        ],
        ..Default::default()
    });
}

/// Extracts behavioral AST nodes (`ActionDef`, `InteractionDef`, `StateDef`, `UseCaseDef`) from Mermaid diagrams and `### AC-XX` sections.
/// Realises: [REQ-0603] [REQ-SYSML-INGEST-MD/extract_behavioral_ast_nodes]
pub fn extract_behavioral_ast_nodes(content: &str, pkg: &mut PackageDef) {
    let ast = parse_markdown_ast(content);
    let (mut seq_counter, mut state_counter) = (0usize, 0usize);
    for cb in ast.code_blocks.iter().filter(|cb| cb.info.trim().eq_ignore_ascii_case("mermaid")) {
        ingest_mermaid_block(&cb.code, &mut seq_counter, &mut state_counter, pkg);
    }
    ensure_default_lifecycle_state(pkg);
    let subject_name = pkg.part_defs.first().map(|p| p.name.clone());
    for (ac_num, slug, bdd) in extract_acceptance_criteria(content) {
        let uc_name = sanitize_identifier(&format!("UC_AC_{ac_num}_{slug}"), "UseCase");
        let obj = slug.replace('_', " ");
        if !pkg.use_case_defs.iter().any(|u| u.name == uc_name) {
            pkg.use_case_defs.push(UseCaseDef { name: uc_name, doc: (!bdd.is_empty()).then_some(bdd), subject: subject_name.clone(), objective: (!obj.is_empty()).then_some(obj), ..Default::default() });
        }
    }
    extract_prose_use_cases(content, pkg);
}

/// Populates formal `RequirementDef` AST nodes on `pkg` when `frontmatter.id` declares a requirement or blueprint ID.
/// Realises: [REQ-SYSML-INGEST-MD/populate_package_requirement]
pub fn populate_package_requirement(content: &str, frontmatter: &Frontmatter, pkg: &mut PackageDef) {
    let Some(req_id) = frontmatter.id.as_deref().filter(|id| {
        id.starts_with("REQ-") || id.starts_with("REQ_") || id.starts_with("BP-") || id.starts_with("BP_")
    }) else {
        return;
    };
    let is_bp = req_id.starts_with("BP-") || req_id.starts_with("BP_") || pkg.name.starts_with("BP_");
    let sub_str = frontmatter.subsystem.as_deref().unwrap_or("").trim();
    let parent_pkg_name = if !is_bp && !sub_str.is_empty() {
        sanitize_identifier(sub_str, &pkg.name)
    } else {
        pkg.name.clone()
    };
    pkg.parent_package = Some(parent_pkg_name.clone());
    let (uuid, complexity, standard, diagnostics) = extract_req_metadata(content, frontmatter);
    let num_str: String = req_id.chars().filter(|c| c.is_ascii_digit()).collect();
    let title_clean = frontmatter.title.as_deref().map_or_else(
        || "Requirement".to_string(),
        |t| sanitize_identifier(t.split(['&', ':']).next().unwrap_or(t).trim(), "Requirement"),
    );
    let mut req_attributes: Vec<AttributeDef> = [("uuidv5", &uuid), ("complexity_class", &complexity), ("governing_standard", &standard), ("diagnostic_codes", &diagnostics)]
        .into_iter()
        .map(|(k, v)| AttributeDef { name: k.to_string(), type_name: "String".to_string(), default_value: Some(format!("\"{v}\"")), doc: None })
        .collect();
    let mut verified_by = Vec::new();
    for (ac_num_str, slug, bdd) in extract_acceptance_criteria(content) {
        req_attributes.push(AttributeDef { name: format!("ac_{ac_num_str}_{}", slug.to_ascii_lowercase()), type_name: "String".to_string(), default_value: Some(format!("\"{bdd}\"")), doc: None });
        verified_by.push(format!("AC_{ac_num_str}_{slug}"));
    }
    let formal_invariants = extract_formal_invariants(content, &num_str);
    let mut requires = Vec::new();
    for inv in &formal_invariants {
        if !requires.contains(&inv.name) {
            requires.push(inv.name.clone());
        }
    }
    let satisfied_by = pkg.part_defs.first().map_or_else(|| format!("{parent_pkg_name}_Component"), |p| p.name.clone());
    pkg.requirement_defs.push(RequirementDef {
        name: format!("REQ_{num_str}_{title_clean}"),
        req_id: req_id.to_string(),
        text: extract_normative_statement(content),
        doc: Some(format!("UUIDv5: {uuid} | Complexity: {complexity} | Standard: {standard} | Diagnostics: {diagnostics}")),
        attributes: req_attributes,
        constraints: formal_invariants,
        requires,
        derived_from: extract_requirement_cross_references(content, req_id),
        verified_by,
        satisfied_by: vec![satisfied_by],
        ..Default::default()
    });
}
