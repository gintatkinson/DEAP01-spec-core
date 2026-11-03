//! `pulldown-cmark` AST metadata/section extraction and ConOps/Mission Intent document assembly (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).

use std::fs;
use std::path::{Path, PathBuf};

use deap_core::markdown_ast::{parse_markdown_ast, ParsedMarkdownAst};

use super::cmark_engine::SysMLParameterBindingEngine;
use super::cmark_links::{relativize_markdown_links, validate_unit_integrity};
use super::cmark_sanitize::sanitize_level_1b_operational_text;
use super::{CANONICAL_CONOPS_UNITS, CANONICAL_MISSION_INTENT_UNITS};
use crate::toc::{extract_headings, generate_table_of_contents, verify_markdown_links};

/// Resolves canonical unit filenames with historical alias deduplication (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn select_canonical_filenames(all_entries: &[String], whitelist: &[&str]) -> Vec<String> {
    let mut filenames: Vec<String> = Vec::new();
    let alias_groups: &[&[&str]] = &[
        &[
            "04_SYSTEM_ARCHITECTURE.md",
            "08_SYSTEM_ARCHITECTURE.md",
            "04_USER_CLASSES_AND_STAKEHOLDERS.md",
            "04_SYSTEM_CAPABILITIES_AND_FUNCTIONS.md",
        ],
        &["05_OPERATIONAL_STATE_SPACE_AND_RISK.md", "05_AIRSPACE_AND_SORA_RISK.md"],
        &["06_SAFETY_INTERLOCKS.md", "06_ROE_SAFETY_INTERLOCKS.md", "06_RULES_OF_ENGAGEMENT.md"],
        &["08_ENVIRONMENTAL_OPERATING_LIMITS.md", "08_ENVIRONMENTAL_MIL_STD_810H.md"],
    ];
    for &item in whitelist {
        if let Some(group) = alias_groups.iter().find(|g| g.contains(&item)) {
            if !filenames.iter().any(|x| group.contains(&x.as_str())) {
                if let Some(&chosen) =
                    group.iter().find(|&&cand| all_entries.iter().any(|e| e == cand))
                {
                    filenames.push(chosen.to_string());
                }
            }
        } else if all_entries.iter().any(|e| e == item) && !filenames.iter().any(|e| e == item) {
            filenames.push(item.to_string());
        }
    }
    filenames
}

/// Validates Unit 4 architecture structure using `parse_markdown_ast` (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn validate_unit_4_architecture_ast(u4_raw: &str, fname: &str) -> Option<String> {
    let ast = parse_markdown_ast(u4_raw);
    let has_super = ast.sections.iter().any(|s| {
        s.heading.starts_with("4.1")
            || s.heading.starts_with("8.1")
            || s.heading.contains("Super-System")
            || s.heading.contains("Super-system")
    });
    let has_mermaid =
        ast.code_blocks.iter().any(|cb| cb.info.starts_with("mermaid")) || u4_raw.contains("SV-1");
    let has_sub = ast.sections.iter().any(|s| {
        s.heading.starts_with("4.3")
            || s.heading.starts_with("8.3")
            || s.heading.starts_with("4.8")
            || s.heading.contains("Subsystem Architecture")
    }) && (u4_raw.contains("LRU") || u4_raw.contains("Subsystem") || u4_raw.contains("Part"));
    if !(has_super && has_mermaid && has_sub)
        && (!u4_raw.contains("{{SUPER_SYSTEM_ARCHITECTURE}}")
            || !u4_raw.contains("{{SUBSYSTEM_ARCHITECTURE_SECTION}}"))
    {
        Some(format!(
            "{fname} omits required placeholder {{{{SUPER_SYSTEM_ARCHITECTURE}}}} or {{{{SUBSYSTEM_ARCHITECTURE_SECTION}}}}."
        ))
    } else {
        None
    }
}

/// Extracts frontmatter table byte spans and updates title/version/date from AST tables (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn extract_unit_table_metadata(
    ast: &ParsedMarkdownAst,
    meta: &mut (String, String, String),
) -> Vec<std::ops::Range<usize>> {
    let mut spans = Vec::new();
    for table in &ast.tables {
        let is_fm = table.headers.first().is_some_and(|h| {
            let c = h.trim().trim_matches('*').trim();
            c.eq_ignore_ascii_case("attribute") || c.eq_ignore_ascii_case("title")
        }) || table.rows.iter().any(|r| {
            r.first().is_some_and(|c| {
                let clean = c.trim().trim_matches('*').trim();
                clean.eq_ignore_ascii_case("title")
                    || clean.eq_ignore_ascii_case("version")
                    || clean.eq_ignore_ascii_case("date")
            })
        });
        if is_fm {
            spans.push(table.start_offset..table.end_offset);
        }
        for row in std::iter::once(&table.headers).chain(table.rows.iter()) {
            if let (Some(c0), Some(c1)) = (row.first(), row.get(1)) {
                let key = c0.trim().trim_matches('*').trim();
                let val = c1.trim();
                if key.eq_ignore_ascii_case("title") && !val.is_empty() {
                    meta.0 = val.to_string();
                } else if key.eq_ignore_ascii_case("version") && !val.is_empty() {
                    meta.1 = val.to_string();
                } else if key.eq_ignore_ascii_case("date") && !val.is_empty() {
                    meta.2 = val.to_string();
                }
            }
        }
    }
    spans
}

/// Strips frontmatter table lines and demotes extra top-level H1 headings outside code blocks (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn strip_frontmatter_and_demote_h1(
    content: &str,
    ast: &ParsedMarkdownAst,
    spans: &[std::ops::Range<usize>],
    h1_found: &mut bool,
) -> String {
    let (mut filtered_lines, mut byte_cursor) = (Vec::new(), 0usize);
    for line in content.split_inclusive('\n') {
        let line_start = byte_cursor;
        byte_cursor += line.len();
        let line_clean = line.trim_end_matches(['\r', '\n']);
        if spans.iter().any(|s| line_start >= s.start && line_start < s.end) {
            continue;
        }
        let in_code = ast
            .code_blocks
            .iter()
            .any(|cb| line_start >= cb.start_offset && line_start < cb.end_offset);
        if !in_code && line_clean.trim().starts_with("# ") {
            if *h1_found {
                filtered_lines.push(format!("#{line_clean}"));
            } else {
                *h1_found = true;
                filtered_lines.push(line_clean.to_string());
            }
        } else {
            filtered_lines.push(line_clean.to_string());
        }
    }
    filtered_lines.join("\n").trim().to_string()
}

/// Discovers and filters canonical Markdown unit file paths in `units_dir` (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn discover_unit_paths(
    units_dir: &Path,
    canonical_whitelist: Option<&[&str]>,
) -> Result<Vec<PathBuf>, String> {
    if !units_dir.is_dir() {
        return Err(format!("Units directory '{}' does not exist.", units_dir.display()));
    }
    let default_wl = match units_dir.file_name().and_then(|n| n.to_str()).unwrap_or("") {
        "conops" => Some(CANONICAL_CONOPS_UNITS),
        "mission_intent" | "missionintent" => Some(CANONICAL_MISSION_INTENT_UNITS),
        _ => None,
    };
    let read_dir = fs::read_dir(units_dir)
        .map_err(|e| format!("Failed to read directory '{}': {e}", units_dir.display()))?;
    let mut all_entries = Vec::new();
    for entry_res in read_dir {
        let Ok(entry) = entry_res else { continue };
        let p = entry.path();
        if p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                all_entries.push(name.to_string());
            }
        }
    }
    all_entries.sort();
    if all_entries.is_empty() {
        return Err(format!("No markdown unit files (*.md) found in '{}'.", units_dir.display()));
    }
    let filenames = if let Some(wl) = canonical_whitelist.or(default_wl) {
        let selected = select_canonical_filenames(&all_entries, wl);
        if selected.is_empty() {
            return Err(format!(
                "No canonical unit files from whitelist found in '{}'.",
                units_dir.display()
            ));
        }
        selected
    } else {
        all_entries
    };
    Ok(filenames.iter().map(|f| units_dir.join(f)).collect())
}

/// [DOC-TIER: A] Compiles unit files located in `units_dir` into a single verified Markdown document.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-ASSEMBLER-ASSEMBLE-DOCUMENT] [FLS: fls_conops_cmark]
/// Contract: Validates unit integrity, substitutes parameters via `param_engine`, extracts frontmatter/TOC via `pulldown-cmark` AST, and checks AST part coverage.
/// Bounds & Faults: O(U * N) time and space over unit files; returns `(assembled_markdown, diagnostic_errors)`.
pub fn assemble_document(
    units_dir: &Path,
    doc_title: Option<&str>,
    doc_version_and_date: (Option<&str>, Option<&str>),
    param_engine: &mut SysMLParameterBindingEngine,
    canonical_whitelist: Option<&[&str]>,
) -> (String, Vec<String>) {
    let unit_paths = match discover_unit_paths(units_dir, canonical_whitelist) {
        Ok(paths) => paths,
        Err(e) => return (String::new(), vec![e]),
    };
    let mut errors = Vec::new();
    for path in &unit_paths {
        let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if [
            "04_SYSTEM_ARCHITECTURE.md",
            "08_SYSTEM_ARCHITECTURE.md",
            "04_USER_CLASSES_AND_STAKEHOLDERS.md",
            "04_SYSTEM_CAPABILITIES_AND_FUNCTIONS.md",
        ]
        .contains(&fname)
        {
            if let Ok(u4_raw) = fs::read_to_string(path) {
                if let Some(err) = validate_unit_4_architecture_ast(&u4_raw, fname) {
                    errors.push(err);
                }
            }
        }
    }
    let (is_valid, integrity_errors) = validate_unit_integrity(&unit_paths, Some(param_engine));
    if !is_valid {
        errors.extend(integrity_errors);
        return (String::new(), errors);
    }
    let units: Vec<(String, String)> = unit_paths
        .iter()
        .map(|path| {
            let raw_text = fs::read_to_string(path).unwrap_or_default();
            let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
            (fname, param_engine.substitute(&raw_text))
        })
        .collect();
    let (meta_title, assembled) =
        render_assembled_markdown(&units, doc_title, doc_version_and_date, param_engine);
    errors.extend(verify_markdown_links(&assembled));
    let lower = meta_title.to_lowercase();
    if (lower.contains("concept of operations") || lower.contains("conops"))
        && !param_engine.ast_part_names.is_empty()
    {
        let missing: Vec<String> = param_engine
            .ast_part_names
            .iter()
            .filter(|p| {
                let clean_p = sanitize_level_1b_operational_text(p);
                !assembled.contains(p.as_str()) && !assembled.contains(&clean_p)
            })
            .cloned()
            .collect();
        if !missing.is_empty() {
            errors.push(format!(
                "ConOps Section 4 AST Part Coverage Gate failed: Missing declared AST part def(s): {} in Section 4.",
                missing.join(", ")
            ));
        }
    }
    (assembled, errors)
}

/// Formats the final assembled Markdown document with frontmatter header table and generated TOC (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn render_assembled_markdown(
    units: &[(String, String)],
    doc_title: Option<&str>,
    (doc_version, doc_date): (Option<&str>, Option<&str>),
    param_engine: &SysMLParameterBindingEngine,
) -> (String, String) {
    let mut meta = (
        doc_title.unwrap_or("Concept of Operations").to_string(),
        doc_version
            .or_else(|| param_engine.parameter_bindings.get("DOCUMENT_VERSION").map(String::as_str))
            .unwrap_or("1.0.0")
            .to_string(),
        doc_date
            .or_else(|| param_engine.parameter_bindings.get("DOCUMENT_DATE").map(String::as_str))
            .unwrap_or("2026-10-10")
            .to_string(),
    );
    let (mut body_sections, mut h1_found) = (Vec::new(), false);
    for (_, content) in units {
        let ast = parse_markdown_ast(content);
        let spans = extract_unit_table_metadata(&ast, &mut meta);
        let cleaned = strip_frontmatter_and_demote_h1(content, &ast, &spans, &mut h1_found);
        if !cleaned.is_empty() {
            body_sections.push(cleaned);
        }
    }
    if let Some(t) = doc_title {
        meta.0 = t.to_string();
    }
    if let Some(v) = doc_version {
        meta.1 = v.to_string();
    }
    if let Some(d) = doc_date {
        meta.2 = d.to_string();
    }
    let full_body = body_sections.join("\n\n");
    let header_table = format!(
        "| Attribute | Value |\n| :--- | :--- |\n| **Title** | {} |\n| **Version** | {} |\n| **Date** | {} |\n",
        meta.0, meta.1, meta.2
    );
    let toc = generate_table_of_contents(&extract_headings(&full_body), 2);
    let assembled = if full_body.starts_with("# ") {
        if let Some((h1, rest)) = full_body.split_once('\n') {
            format!("{header_table}\n{h1}\n\n{toc}\n{}\n", rest.trim())
        } else {
            format!("{header_table}\n{full_body}\n\n{toc}\n")
        }
    } else {
        format!("{header_table}\n# {}\n\n{toc}\n{}\n", meta.0, full_body.trim())
    };
    (meta.0, assembled)
}

/// Executes top-level ConOps and Mission Intent workspace assembly (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
pub(super) fn execute_conops_assembly(
    input_dir: &Path,
    output_dir: &Path,
    verify_only: bool,
    params: Option<&Path>,
    (domain, workspace_dir): (Option<&str>, Option<&Path>),
) -> Result<bool, Box<dyn std::error::Error>> {
    let ws_path = if let Some(ws) = workspace_dir {
        ws.to_path_buf()
    } else {
        [
            output_dir.to_path_buf(),
            output_dir.join(".."),
            output_dir.join("..").join(".."),
            input_dir.to_path_buf(),
            input_dir.join(".."),
            input_dir.join("..").join(".."),
        ]
        .into_iter()
        .find(|c| c.join("schema").is_dir() || c.join(".pipeline").is_dir())
        .map(|c| c.canonicalize().unwrap_or(c))
        .unwrap_or_else(|| input_dir.to_path_buf())
    };
    let mut engine = SysMLParameterBindingEngine::new(None, params, Some(&ws_path), true, domain);
    let detected_dom = domain.map_or_else(|| engine.detected_domain.clone(), str::to_string);
    let mut all_errors = Vec::new();
    for (leaf, title, out_name, wl, tag) in [
        ("conops", "Concept of Operations (ConOps)", "CONOPS.md", CANONICAL_CONOPS_UNITS, "CONOPS"),
        (
            "mission_intent",
            "Tactical Mission Intent & Execution Plan",
            "MISSION_INTENT.md",
            CANONICAL_MISSION_INTENT_UNITS,
            "MISSION_INTENT",
        ),
    ] {
        let subdir = [
            input_dir.join(&detected_dom).join(leaf),
            input_dir.join("units").join(&detected_dom).join(leaf),
            input_dir.join(leaf),
            input_dir.join("units").join(leaf),
        ]
        .into_iter()
        .find(|c| c.is_dir());
        if let Some(u_dir) = subdir {
            let (doc, errs) = assemble_document(&u_dir, Some(title), (None, None), &mut engine, Some(wl));
            if !errs.is_empty() {
                all_errors.extend(errs.into_iter().map(|e| format!("[{tag}] {e}")));
            } else if !verify_only {
                fs::create_dir_all(output_dir)?;
                let out_file = output_dir.join(out_name);
                let relativized = relativize_markdown_links(&doc, &out_file, &ws_path);
                fs::write(&out_file, relativized)?;
            }
        }
    }
    Ok(all_errors.is_empty())
}
