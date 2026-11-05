//! Markdown requirement metadata, acceptance criteria, formal invariant, and cross-reference AST extraction.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use crate::table::sanitize_identifier;
use deap_core::markdown_ast::parse_markdown_ast;
use deap_core::sysml_ast::ConstraintDef;

use super::md_common::{
    clean_bdd_text, clean_meta_val, clean_normative_text, collapse_whitespace, find_req_ids,
    parse_frontmatter, strip_latex_math_commands, Frontmatter,
};

/// Filters `content` lines to exclude fenced code blocks (preserving `math` blocks when `keep_math` is `true`). Realises: [REQ-INGEST-MD-REQUIREMENTS]
fn non_fenced_lines(content: &str, keep_math: bool) -> Vec<&str> {
    let ast = parse_markdown_ast(content);
    let base = content.as_ptr() as usize;
    content
        .lines()
        .filter(|l| {
            let off = (l.as_ptr() as usize).saturating_sub(base);
            !ast.code_blocks.iter().any(|cb| {
                !(keep_math && cb.info.trim().eq_ignore_ascii_case("math"))
                    && (cb.start_offset..cb.end_offset).contains(&off)
            })
        })
        .collect()
}

/// Extracts the Section 1 normative requirement statement outside fenced code blocks.
/// Realises: [REQ-TRANSLATORS-MARKDOWN-EXTRACT-NORMATIVE-STATEMENT]
pub fn extract_normative_statement(content: &str) -> String {
    let lines: Vec<&str> = non_fenced_lines(content, false)
        .into_iter()
        .map(str::trim)
        .skip_while(|t| !t.starts_with("## 1."))
        .skip(1)
        .take_while(|t| !t.starts_with("##") && !t.starts_with("---"))
        .skip_while(|t| t.is_empty() || t.starts_with('|'))
        .take_while(|t| !t.is_empty() && !t.starts_with('|'))
        .collect();
    if lines.is_empty() {
        "Normative statement specified in schema.".to_string()
    } else {
        clean_normative_text(&lines.join(" "))
    }
}

/// Returns `true` when `byte` can continue a diagnostic-code token.
/// Realises: [REQ-TRANSLATORS-MARKDOWN-EXTRACT-DIAGNOSTIC-CODES]
fn is_code_token_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// Extracts declared diagnostic codes from `content` in source order, deduplicated.
/// Accepts `DEAP-E` codes and plain `E` codes (2-4 digits, optional `xx`/`XX` suffix) at token boundaries.
/// Realises: [REQ-TRANSLATORS-MARKDOWN-EXTRACT-DIAGNOSTIC-CODES]
pub fn extract_diagnostic_codes(content: &str) -> Vec<String> {
    let declared_lines: Vec<&str> = non_fenced_lines(content, false)
        .into_iter()
        .filter(|line| {
            let lower = line.to_ascii_lowercase();
            lower.contains("diagnostic") || lower.contains("binding")
        })
        .collect();
    let text = declared_lines.join("\n");
    let bytes = text.as_bytes();
    let (mut codes, mut index) = (Vec::new(), 0usize);
    while index < bytes.len() {
        let prefix_len = if bytes.get(index..).is_some_and(|rest| rest.starts_with(b"DEAP-E")) { 6 } else if bytes.get(index) == Some(&b'E') { 1 } else { index += 1; continue; };
        let left_ok = index == 0 || !bytes.get(index - 1).is_some_and(|&b| is_code_token_byte(b));
        let in_bad_prefix = index >= 5 && bytes.get(index - 5..index).is_some_and(|p| p == b"DEAP-");
        if !left_ok || in_bad_prefix {
            index += 1;
            continue;
        }
        let mut cursor = index + prefix_len;
        let digits_start = cursor;
        while cursor < bytes.len() && cursor - digits_start < 4 && bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
            cursor += 1;
        }
        if cursor - digits_start < 2 || bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
            index += 1;
            continue;
        }
        let is_x = |i: usize| bytes.get(i).is_some_and(|b| *b == b'x' || *b == b'X');
        if is_x(cursor) && is_x(cursor + 1) {
            cursor += 2;
        }
        if !bytes.get(cursor).is_none_or(|&b| !is_code_token_byte(b)) {
            index += 1;
            continue;
        }
        if let Some(code) = text.get(index..cursor) {
            if !code.is_empty() && !codes.iter().any(|existing: &String| existing == code) {
                codes.push(code.to_string());
            }
        }
        index = cursor.max(index + 1);
    }
    codes
}

/// Extracts `(uuid, complexity, standard, diagnostics)` from frontmatter, pre-section tables, and
/// declared document text. Returns all declared diagnostic codes joined by `, ` with no fallback.
/// Realises: [REQ-TRANSLATORS-MARKDOWN-EXTRACT-REQ-METADATA]
pub fn extract_req_metadata(content: &str, frontmatter: &Frontmatter) -> (String, String, String, String) {
    let mut uuid = frontmatter.uuidv5.clone().unwrap_or_default();
    let (mut comp, mut std_str) = (String::new(), String::new());
    for t in non_fenced_lines(content, false).into_iter().map(str::trim).take_while(|t| !t.starts_with("## ")) {
        let cells: Vec<&str> = t.split('|').map(str::trim).filter(|c| !c.is_empty()).collect();
        if let (true, Some(c0), Some(c1)) = (t.starts_with('|'), cells.first(), cells.get(1)) {
            let (k, v) = (c0.replace(['*', '`', '_'], "").to_ascii_lowercase(), clean_meta_val(c1));
            if (k.contains("complexity") || k.contains("class")) && comp.is_empty() {
                comp = v;
            } else if (k.contains("standard") || k.contains("governing")) && std_str.is_empty() {
                std_str = v;
            } else if (k.contains("uuid") || k.contains("anchor")) && uuid.is_empty() {
                uuid = v;
            }
        }
    }
    if comp.is_empty() { comp = "Class P".to_string(); }
    if std_str.is_empty() { std_str = "IEEE 29148-2018 / RFC 2119".to_string(); }
    let diag = extract_diagnostic_codes(content).join(", ");
    (uuid, comp, std_str, diag)
}

/// Parses an `### AC-XX: <Title>` heading line into `(ac_number, title)`. Realises: [REQ-INGEST-MD-REQUIREMENTS]
fn parse_ac_header(line: &str) -> Option<(usize, String)> {
    let r = line.trim().strip_prefix("###")?.trim_start();
    let r = r.strip_prefix("Acceptance Criteria:").or_else(|| r.strip_prefix("Acceptance Criteria")).unwrap_or(r).trim_start();
    let r = ["AC-", "AC", "ac-", "ac"].iter().find_map(|p| r.strip_prefix(p))?;
    let n_len = r.chars().take_while(|c| c.is_ascii_digit()).count();
    let num: usize = r.get(..n_len).filter(|s| !s.is_empty())?.parse().ok()?;
    let after = r[n_len..].trim_start();
    Some((num, after.strip_prefix([':', '-']).unwrap_or(after).trim().to_string()))
}

/// Extracts and cleans the BDD prose following `tag` in `line`. Realises: [REQ-INGEST-MD-REQUIREMENTS]
fn extract_bullet_content(line: &str, tag: &str) -> String {
    line.to_ascii_lowercase().find(tag).map_or_else(
        || clean_bdd_text(line),
        |pos| clean_bdd_text(line[pos + tag.len()..].trim_start_matches(|c: char| c == '*' || c == '`' || c.is_whitespace())),
    )
}

/// Extracts `(ac_num_str, clean_ac_slug, bdd_text)` tuples from `### AC-XX` sections.
/// Realises: [REQ-TRANSLATORS-MARKDOWN-EXTRACT-ACCEPTANCE-CRITERIA]
pub fn extract_acceptance_criteria(content: &str) -> Vec<(String, String, String)> {
    let lines = non_fenced_lines(content, false);
    let (mut ac_list, mut i, n) = (Vec::new(), 0, lines.len());
    while i < n {
        let Some((num, raw_title)) = parse_ac_header(lines.get(i).map_or("", |l| l.trim())) else {
            i += 1;
            continue;
        };
        let (ac_num_str, clean_ac_slug) = (format!("{:02}", num), sanitize_identifier(&raw_title, "Criteria"));
        i += 1;
        let (mut given, mut when, mut then, mut diag, mut fb) = (String::new(), String::new(), String::new(), String::new(), Vec::new());
        while let Some(&cur_raw) = lines.get(i) {
            let cur = cur_raw.trim();
            if cur.starts_with('#') || cur.starts_with("---") {
                break;
            }
            if !cur.is_empty() {
                let low = cur.to_ascii_lowercase();
                let mut matched = false;
                for (tag, dst) in [("given:", &mut given), ("when:", &mut when), ("then:", &mut then), ("diagnostic:", &mut diag)] {
                    if low.contains(tag) {
                        *dst = extract_bullet_content(cur, tag);
                        matched = true;
                        break;
                    }
                }
                if !matched {
                    fb.push(cur);
                }
            }
            i += 1;
        }
        let parts: Vec<String> = [("Given", &given), ("When", &when), ("Then", &then), ("Diagnostic", &diag)]
            .into_iter()
            .filter(|(_, v)| !v.is_empty())
            .map(|(k, v)| format!("{k}: {}", v.trim().trim_end_matches('.')))
            .collect();
        let bdd = if !parts.is_empty() {
            format!("{}.", parts.join(". "))
        } else if !fb.is_empty() {
            clean_bdd_text(&fb.join(" "))
        } else {
            format!("AC-{ac_num_str} verification criteria.")
        };
        ac_list.push((ac_num_str, clean_ac_slug, bdd));
    }
    ac_list
}

/// Normalizes an invariant heading/label line by stripping inline math, LaTeX commands, and trailing qualifiers. Realises: [REQ-INGEST-MD-REQUIREMENTS]
fn clean_invariant_title(raw: &str) -> String {
    let text = raw.trim().split(':').next().unwrap_or("").replace(['*', '`', '_', '#'], " ");
    let (mut no_dollar, mut in_d) = (String::with_capacity(text.len()), false);
    for c in text.chars() {
        if c == '$' {
            in_d = !in_d;
        } else if !in_d {
            no_dollar.push(c);
        }
    }
    let no_latex = strip_latex_math_commands(&no_dollar, Some(&[]));
    let trimmed = no_latex.trim_start();
    let mut s = trimmed
        .split_once(char::is_whitespace)
        .filter(|(t, _)| !t.is_empty() && t.chars().any(|c| c.is_ascii_digit()) && t.chars().all(|c| c.is_ascii_digit() || c == '.'))
        .map_or_else(|| trimmed.to_string(), |(_, r)| r.trim_start().to_string());
    for delim in [" over ", " across ", " for ", " where ", " on ", " satisfying ", " satisfies ", " under "] {
        if let Some(pos) = s.to_ascii_lowercase().find(delim) {
            s = s[..pos].to_string();
        }
    }
    collapse_whitespace(&s).replace('\u{2014}', "--")
}

/// Extracts formal invariant constraints from Section 2 (`## 2. Formal Invariant`).
/// Realises: [REQ-SYSML-INGEST-MD/extract_formal_invariants]
pub fn extract_formal_invariants(content: &str, req_num: &str) -> Vec<ConstraintDef> {
    let all_lines = non_fenced_lines(content, true);
    let mut invariants = Vec::new();
    for prefixes in [
        &["## 2. Formal Invariant", "## 2."][..],
        &["## 4. Formal", "## 4. Mathematical", "## 4. Safety"][..],
    ] {
        let lines: Vec<&str> = all_lines
            .iter()
            .copied()
            .skip_while(|l| !prefixes.iter().any(|p| l.trim().starts_with(p)))
            .skip(1)
            .take_while(|l| !l.trim().starts_with("## ") && !l.trim().starts_with("---"))
            .collect();
        let (mut i, mut pending_title) = (0, None);
        while let Some(&raw_line) = lines.get(i) {
            let (line, low) = (raw_line.trim(), raw_line.trim().to_ascii_lowercase());
            let is_cand = !line.is_empty()
                && (["invariant", "contract", "gate", "ordering", "zero diff churn", "determinism", "equivalence", "bound"]
                    .iter()
                    .any(|k| low.contains(k))
                    || line.ends_with(':')
                    || line.starts_with("### "))
                && !["let ", "where ", "for each", "each "].iter().any(|p| low.starts_with(p))
                && !line.starts_with(['|', '`', '$']);
            if is_cand {
                let ct = clean_invariant_title(line);
                if !ct.is_empty() {
                    pending_title = Some(ct);
                }
            }
            if line.starts_with("```math") || line == "$$" {
                let close = if line == "$$" { "$$" } else { "```" };
                let mut math_lines = Vec::new();
                i += 1;
                while let Some(&m_raw) = lines.get(i) {
                    let m = m_raw.trim();
                    if (close == "$$" && m == "$$") || (close == "```" && m.starts_with("```")) {
                        break;
                    }
                    if !m.is_empty() {
                        math_lines.push(m);
                    }
                    i += 1;
                }
                let math_doc = collapse_whitespace(&math_lines.join(" "))
                    .replace('\u{2014}', "--")
                    .replace("*/", "* /")
                    .replace("/*", "/ *");
                let slug = pending_title
                    .take()
                    .map(|t| sanitize_identifier(&t, "FormalInvariant"))
                    .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("invariant"))
                    .unwrap_or_else(|| "FormalInvariant".to_string());
                let base = format!("Invariant_REQ_{req_num}_{slug}");
                let (mut inv_name, mut sfx) = (base.clone(), 2);
                while invariants.iter().any(|inv: &ConstraintDef| inv.name == inv_name) {
                    inv_name = format!("{base}_{sfx}");
                    sfx += 1;
                }
                invariants.push(ConstraintDef {
                    name: inv_name,
                    doc: (!math_doc.is_empty()).then_some(math_doc),
                    is_assertion: false,
                    ..Default::default()
                });
            }
            i += 1;
        }
    }
    if invariants.is_empty() {
        invariants.push(ConstraintDef {
            name: format!("Invariant_REQ_{req_num}_FormalInvariant"),
            doc: Some(format!("Formal invariant constraint for REQ-{req_num}.")),
            is_assertion: false,
            ..Default::default()
        });
    }
    invariants
}

static REQ_TITLE_MAP: OnceLock<BTreeMap<String, String>> = OnceLock::new();

/// Resolves a requirement reference into a canonical SysML v2 identifier.
/// Realises: [REQ-SYSML-INGEST-MD/resolve_requirement_reference]
pub fn resolve_requirement_reference(req_id: &str) -> String {
    let num_str: String = req_id.chars().filter(|c| c.is_ascii_digit()).collect();
    let map = REQ_TITLE_MAP.get_or_init(|| {
        let mut m = BTreeMap::new();
        let m_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        for dir in [Path::new("schema").to_path_buf(), Path::new(&m_dir).join("../../schema"), Path::new(&m_dir).join("schema")] {
            if let Ok(entries) = fs::read_dir(&dir) {
                for p in entries.flatten().map(|e| e.path()).filter(|p| {
                    p.extension().and_then(|s| s.to_str()) == Some("md")
                        && p.file_stem().and_then(|s| s.to_str()).is_some_and(|f| f.starts_with("REQ-"))
                }) {
                    if let Ok(c) = fs::read_to_string(&p) {
                        let (fm, _) = parse_frontmatter(&c);
                        if let (Some(id), Some(title)) = (fm.id, fm.title) {
                            m.insert(id, title);
                        }
                    }
                }
                if !m.is_empty() {
                    break;
                }
            }
        }
        m
    });
    if let Some(title) = map.get(&format!("REQ-{num_str}")) {
        let primary = title.split('&').next().unwrap_or(title).split(':').next().unwrap_or(title).trim();
        format!("REQ_{num_str}_{}", sanitize_identifier(primary, "Requirement"))
    } else {
        format!("REQ_{num_str}")
    }
}

/// Extracts cross-referenced requirement identifiers (`REQ-XXXX`) from Section 1 and Section 2.
/// Realises: [REQ-TRANSLATORS-MARKDOWN-EXTRACT-REQUIREMENT-CROSS-REFERENCES]
pub fn extract_requirement_cross_references(content: &str, current_req_id: &str) -> Vec<String> {
    let relevant = content.find("## 1.").map_or(content, |pos| {
        let after = &content[pos..];
        after.find("## 3.").map_or(after, |end| &after[..end])
    });
    let cur_digits: String = current_req_id.chars().filter(|c| c.is_ascii_digit()).collect();
    let mut refs = Vec::new();
    for r_str in find_req_ids(relevant) {
        if r_str.chars().filter(|c| c.is_ascii_digit()).collect::<String>() != cur_digits {
            let canonical = resolve_requirement_reference(r_str);
            if !refs.contains(&canonical) {
                refs.push(canonical);
            }
        }
    }
    refs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_formal_invariants_and_cross_references() {
        let content = "## 1. Normative Statement\nSort symbols (REQ-0031, REQ-0044, REQ-0019).\n\n## 2. Formal Invariant\nStrict Total Ordering Invariant:\n```math\nx < y\n```\nZero Diff Churn & Determinism:\n```math\nE(A1) = E(A2)\n```\n";
        let invs = extract_formal_invariants(content, "0032");
        assert_eq!(invs.len(), 2);
        assert_eq!(
            invs.first().map(|i| i.name.as_str()),
            Some("Invariant_REQ_0032_Strict_Total_Ordering_Invariant")
        );
        assert_eq!(
            invs.get(1).map(|i| i.name.as_str()),
            Some("Invariant_REQ_0032_Zero_Diff_Churn_Determinism")
        );
        let refs = extract_requirement_cross_references(content, "REQ-0032");
        assert_eq!(refs.len(), 3);
    }
}
