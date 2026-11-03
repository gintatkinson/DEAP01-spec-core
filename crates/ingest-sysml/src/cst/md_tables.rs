//! Markdown table AST processors for BOM, Use Cases, and Logical Interface / Port tables.
//!
//! **[DOC-TIER: A]**
//! Realises: [REQ-INGEST-MD-TABLES], [REQ-SYSML-INGEST-MD]

use std::collections::BTreeMap;

use crate::table::{
    extract_provenance_citation, parse_numeric_with_unit, sanitize_identifier, MarkdownTable,
    PROVENANCE_COLUMNS,
};
use deap_core::markdown_ast::parse_markdown_ast;
use deap_core::{AttributeDef, PackageDef, PartDef, PortDef, TestCaseDef, UseCaseDef};

use super::md_common::{clean_bdd_text, compose_sanitized_doc, strip_leading_section_number};
use super::md_parametric::resolve_typed_default;

/// Constructs a `PortDef` with an empty `port_category` for canonical interface modeling.
/// Realises: [REQ-SYSML-INGEST-MD/make_port]
pub fn make_port(name: &str, type_name: &str, direction: &str) -> PortDef {
    PortDef { name: name.to_string(), type_name: type_name.to_string(), direction: direction.to_string(), ..Default::default() }
}

/// Checks whether a normalized table header list represents a Use Case specification table.
/// Realises: [REQ-SYSML-INGEST-MD/is_use_case_table]
pub fn is_use_case_table(headers: &[String]) -> bool {
    headers.iter().any(|h| {
        matches!(
            h.as_str(),
            "use_case_id" | "uc_id" | "use_case" | "included_use_cases" | "verification_test_cases"
        )
    })
}

/// Splits a comma- or semicolon-delimited cell string into cleaned non-empty prose items.
/// Realises: [REQ-SYSML-INGEST-MD/split_trimmed_items]
pub fn split_trimmed_items(raw: &str) -> Vec<String> {
    raw.split([',', ';'])
        .map(|s| clean_bdd_text(s.trim().trim_matches(['`', '*', ' '])))
        .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("none") && !s.eq_ignore_ascii_case("n/a") && s != "-")
        .collect()
}

/// Splits a comma- or semicolon-delimited cell string into sanitized SysML identifiers.
/// Realises: [REQ-SYSML-INGEST-MD/split_ident_list]
pub fn split_ident_list(raw: &str, fallback: &str) -> Vec<String> {
    raw.split([',', ';'])
        .map(|s| s.trim().trim_matches(['`', '*', ' ']))
        .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("none") && !s.eq_ignore_ascii_case("n/a") && *s != "-")
        .map(|s| sanitize_identifier(s, fallback))
        .collect()
}

/// Parses an H2-H4 or bold `UC-XX` heading line into a sanitized `UseCaseDef` identifier. Realises: [REQ-INGEST-MD-TABLES]
fn parse_prose_uc_header(line: &str) -> Option<String> {
    let t = line.trim();
    let raw_body = if t.starts_with('#') {
        let hc = t.chars().take_while(|&c| c == '#').count();
        t.get(hc..).filter(|_| (2..=4).contains(&hc))?.trim()
    } else {
        t.strip_prefix("**")?.strip_suffix("**")?.trim()
    };
    let clean = raw_body.trim_matches(['*', '`', ' ']);
    let low = clean.to_ascii_lowercase();
    let stripped = strip_leading_section_number(&low);
    if !["uc-", "uc_", "uc "].iter().any(|p| stripped.starts_with(p)) {
        return None;
    }
    let off = low.len().saturating_sub(stripped.len());
    Some(sanitize_identifier(clean.get(off..).unwrap_or(clean).trim(), "UseCase"))
}

/// Parses a single body line inside a prose `UC-XX` section into `uc` and `cur_tcs`.
/// Realises: [REQ-SYSML-INGEST-MD/extract_prose_use_cases]
fn ingest_prose_uc_line(t: &str, uc: &mut UseCaseDef, cur_tcs: &mut Vec<String>, in_scen: &mut bool) {
    let n_len = t.chars().take_while(|c| c.is_ascii_digit()).count();
    if let Some(step) = (n_len > 0).then(|| t.get(n_len..)).flatten().and_then(|s| s.strip_prefix(". ").or_else(|| s.strip_prefix(") "))) {
        *in_scen = true;
        let c = clean_bdd_text(step.trim());
        if !c.is_empty() {
            uc.steps.push(c);
        }
    } else if let Some((k_raw, v_raw)) = t.strip_prefix("- ").or_else(|| t.strip_prefix("* ")).map(|b| b.replace('*', "")).and_then(|s| s.split_once(':').map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().to_string()))) {
        *in_scen = k_raw.contains("scenario") || k_raw.contains("steps");
        if matches!(k_raw.as_str(), "primary actor" | "actor" | "actors") || k_raw.contains("secondary actor") || k_raw.contains("supporting actor") {
            for a in split_ident_list(&v_raw, "Actor") {
                if uc.actor.is_none() && !k_raw.contains("secondary") && !k_raw.contains("supporting") {
                    uc.actor = Some(a.clone());
                }
                if !uc.actors.contains(&a) {
                    uc.actors.push(a);
                }
            }
        } else if matches!(k_raw.as_str(), "objective" | "goal") {
            let obj = clean_bdd_text(&v_raw);
            if !obj.is_empty() {
                uc.objective = Some(obj);
            }
        } else if k_raw.starts_with("precondition") {
            uc.preconditions.extend(split_trimmed_items(&v_raw));
        } else if k_raw.starts_with("postcondition") {
            uc.postconditions.extend(split_trimmed_items(&v_raw));
        } else if k_raw.contains("included use case") || k_raw == "includes" {
            for inc in split_ident_list(&v_raw, "UseCase") {
                if !uc.includes.contains(&inc) {
                    uc.includes.push(inc);
                }
            }
        } else if k_raw.contains("test case") || k_raw.contains("verification") {
            for tc in split_ident_list(&v_raw, "TestCase") {
                if !cur_tcs.contains(&tc) {
                    cur_tcs.push(tc);
                }
            }
        }
    } else if *in_scen && !t.starts_with(['|', '-', '*']) {
        let cont = clean_bdd_text(t);
        if let Some(last) = (!cont.is_empty()).then(|| uc.steps.last_mut()).flatten() {
            last.push(' ');
            last.push_str(&cont);
        }
    }
}

/// Extracts prose `UC-XX` subsections and linked `TestCaseDef` nodes into `pkg`.
/// Realises: [REQ-SYSML-INGEST-MD/extract_prose_use_cases]
pub fn extract_prose_use_cases(content: &str, pkg: &mut PackageDef) {
    let ast = parse_markdown_ast(content);
    let base = content.as_ptr() as usize;
    let subject = pkg.part_defs.first().map(|p| p.name.clone());
    let (mut cur_uc, mut cur_tcs, mut in_scen) = (None::<UseCaseDef>, Vec::<String>::new(), false);
    let flush = |uc_opt: &mut Option<UseCaseDef>, tcs: &mut Vec<String>, out: &mut PackageDef| {
        if let Some(mut uc) = uc_opt.take() {
            if uc.doc.is_none() {
                uc.doc = uc.objective.clone();
            }
            for tc in tcs.drain(..) {
                if !out.test_case_defs.iter().any(|t| t.name == tc) {
                    out.test_case_defs.push(TestCaseDef { name: tc, doc: Some(format!("Verification test case for {}", uc.name)), ..Default::default() });
                }
            }
            if !out.use_case_defs.iter().any(|u| u.name == uc.name) {
                out.use_case_defs.push(uc);
            }
        }
    };
    for raw_line in content.lines() {
        let off = (raw_line.as_ptr() as usize).saturating_sub(base);
        if ast.code_blocks.iter().any(|cb| (cb.start_offset..cb.end_offset).contains(&off)) {
            continue;
        }
        let t = raw_line.trim();
        if let Some(uc_name) = parse_prose_uc_header(t) {
            flush(&mut cur_uc, &mut cur_tcs, pkg);
            in_scen = false;
            cur_uc = Some(UseCaseDef { name: uc_name, subject: subject.clone(), ..Default::default() });
        } else if t.starts_with('#') || t.starts_with("---") {
            flush(&mut cur_uc, &mut cur_tcs, pkg);
            in_scen = false;
        } else if let Some(uc) = cur_uc.as_mut().filter(|_| !t.is_empty()) {
            ingest_prose_uc_line(t, uc, &mut cur_tcs, &mut in_scen);
        }
    }
    flush(&mut cur_uc, &mut cur_tcs, pkg);
}

/// Processes Use Case specification tables into `UseCaseDef` and `TestCaseDef` AST nodes.
/// Realises: [REQ-SYSML-INGEST-MD/process_use_case_table]
pub fn process_use_case_table(table: &MarkdownTable, pkg: &mut PackageDef) {
    let h = &table.normalized_headers;
    let find_col = |keys: &[&str]| h.iter().find(|c| keys.contains(&c.as_str()));
    let id_col = find_col(&["use_case_id", "uc_id", "id"]);
    let title_col = find_col(&["use_case", "title", "name", "use_case_name"]);
    let actor_col = find_col(&["primary_actor", "actor", "actors"]);
    let sec_actor_col = find_col(&["secondary_actor", "secondary_actors", "supporting_actors"]);
    let obj_col = find_col(&["objective", "goal", "description", "summary"]);
    let pre_col = find_col(&["preconditions", "precondition"]);
    let post_col = find_col(&["postconditions", "postcondition"]);
    let inc_col = find_col(&["included_use_cases", "includes", "include"]);
    let tc_col = find_col(&["verification_test_cases", "test_cases", "test_case", "verification"]);
    let steps_col = find_col(&["steps", "main_success_scenario", "scenario"]);
    let subject_name = pkg.part_defs.first().map(|p| p.name.clone());

    for row in &table.rows {
        let raw_id = id_col.and_then(|c| row.get(c)).map_or("", |v| v.trim().trim_matches(['`', '*', ' ']));
        let raw_title = title_col.and_then(|c| row.get(c)).map_or("", |v| v.trim().trim_matches(['`', '*', ' ']));
        if raw_id.is_empty() && raw_title.is_empty() {
            continue;
        }
        let uc_name = if !raw_id.is_empty() && !raw_title.is_empty() && raw_id != raw_title {
            sanitize_identifier(&format!("{raw_id}_{raw_title}"), "UseCase")
        } else if !raw_id.is_empty() {
            sanitize_identifier(raw_id, "UseCase")
        } else {
            sanitize_identifier(raw_title, "UseCase")
        };
        let mut actors = actor_col.and_then(|c| row.get(c)).map(|v| split_ident_list(v, "Actor")).unwrap_or_default();
        let actor = actors.first().cloned();
        if let Some(sec_list) = sec_actor_col.and_then(|c| row.get(c)).map(|v| split_ident_list(v, "Actor")) {
            for sa in sec_list {
                if !actors.contains(&sa) {
                    actors.push(sa);
                }
            }
        }
        let objective = obj_col
            .and_then(|c| row.get(c))
            .map(|v| clean_bdd_text(v.trim()))
            .filter(|s| !s.is_empty())
            .or_else(|| (!raw_title.is_empty()).then(|| clean_bdd_text(raw_title)));
        if let Some(tcs) = tc_col.and_then(|c| row.get(c)).map(|v| split_ident_list(v, "TestCase")) {
            for tc_name in tcs {
                if !pkg.test_case_defs.iter().any(|t| t.name == tc_name) {
                    pkg.test_case_defs.push(TestCaseDef {
                        name: tc_name,
                        doc: Some(format!("Verification test case for {uc_name}")),
                        ..Default::default()
                    });
                }
            }
        }
        if !pkg.use_case_defs.iter().any(|u| u.name == uc_name) {
            pkg.use_case_defs.push(UseCaseDef {
                name: uc_name,
                doc: objective.clone(),
                actor,
                actors,
                subject: subject_name.clone(),
                objective,
                preconditions: pre_col.and_then(|c| row.get(c)).map(|v| split_trimmed_items(v)).unwrap_or_default(),
                postconditions: post_col.and_then(|c| row.get(c)).map(|v| split_trimmed_items(v)).unwrap_or_default(),
                includes: inc_col.and_then(|c| row.get(c)).map(|v| split_ident_list(v, "UseCase")).unwrap_or_default(),
                steps: steps_col.and_then(|c| row.get(c)).map(|v| split_trimmed_items(v)).unwrap_or_default(),
                ..Default::default()
            });
        }
    }
}

/// Processes BOM table into `PartDef` AST entities with typed `AttributeDef` items.
/// Realises: [REQ-SYSML-INGEST-MD/process_bom_table]
pub fn process_bom_table(table: &MarkdownTable, part_registry: &mut BTreeMap<String, PartDef>) {
    let headers = &table.normalized_headers;
    let unit_map: BTreeMap<String, String> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| (h.clone(), table.units.get(i).cloned().unwrap_or_default()))
        .collect();
    let comp_col = headers
        .iter()
        .find(|h| ["component", "component_name", "part", "part_name", "subsystem", "item", "module", "assembly"].contains(&h.as_str()))
        .or_else(|| headers.first())
        .cloned()
        .unwrap_or_else(|| "component".to_string());
    let attr_name_col = headers
        .iter()
        .find(|h| {
            h.as_str() != comp_col.as_str()
                && ["attribute", "attribute_name", "attribute_item", "property", "property_name", "field", "field_name", "parameter", "parameter_name"]
                    .contains(&h.as_str())
        })
        .cloned();
    for row in &table.rows {
        let comp_raw = row.get(&comp_col).map_or("", |v| v.trim());
        if comp_raw.is_empty()
            || comp_raw.chars().all(|c| matches!(c, '-' | '=' | '_' | ':' | '|') || c.is_whitespace())
            || !row.iter().any(|(k, v)| k.as_str() != comp_col.as_str() && !v.trim().is_empty())
        {
            continue;
        }
        let comp_clean = comp_raw.chars().filter(|c| !matches!(c, '*' | '`' | '_' | '#')).collect::<String>().trim().to_ascii_lowercase();
        if ["total", "subtotal", "sum", "summary", "aggregate", "overall"]
            .iter()
            .any(|&kw| comp_clean == kw || comp_clean.starts_with(&format!("{kw} ")))
        {
            continue;
        }
        let comp_name = sanitize_identifier(comp_raw, "Component");
        let part = part_registry.entry(comp_name.clone()).or_insert_with(|| PartDef { name: comp_name, ..Default::default() });
        let row_citation = extract_provenance_citation(row, headers);
        if let Some(ref attr_col) = attr_name_col {
            process_row_per_attribute_bom_row(row, headers, &unit_map, attr_col, part);
            continue;
        }
        let mut desc_found = false;
        for (col, val) in row {
            if col == &comp_col || val.trim().is_empty() || PROVENANCE_COLUMNS.contains(&col.as_str()) {
                continue;
            }
            if ["description", "doc", "function", "notes"].contains(&col.as_str()) {
                desc_found = true;
                if part.doc.is_none() {
                    part.doc = Some(compose_sanitized_doc(val.trim(), &row_citation, ""));
                }
                continue;
            }
            let attr_name = sanitize_identifier(col, "attr");
            let (num_val, val_unit, int_val) = parse_numeric_with_unit(val);
            let col_u = unit_map.get(col).cloned().unwrap_or_default();
            let eff_unit = if !col_u.is_empty() { col_u } else { val_unit.unwrap_or_default() };
            let (type_name, def_opt) = resolve_typed_default("", val, int_val, num_val, None);
            let doc_str = compose_sanitized_doc("", &row_citation, &eff_unit);
            if !part.attributes.iter().any(|a| a.name == attr_name) {
                part.attributes.push(AttributeDef {
                    name: attr_name,
                    type_name,
                    default_value: def_opt,
                    doc: (!doc_str.is_empty()).then_some(doc_str),
                });
            }
        }
        if !desc_found && !row_citation.is_empty() && part.doc.is_none() {
            part.doc = Some(compose_sanitized_doc("", &row_citation, ""));
        }
    }
}

/// Processes a row-per-attribute BOM table row into a typed `AttributeDef` on `part`. Realises: [REQ-INGEST-MD-TABLES]
fn process_row_per_attribute_bom_row(
    row: &BTreeMap<String, String>,
    headers: &[String],
    unit_map: &BTreeMap<String, String>,
    attr_col: &str,
    part: &mut PartDef,
) {
    let raw_attr = row.get(attr_col).map_or("", |v| v.trim().trim_matches(['`', '*', ' ']));
    if raw_attr.is_empty() {
        return;
    }
    let row_citation = extract_provenance_citation(row, headers);
    let attr_name = sanitize_identifier(raw_attr, "attr");
    let find_col = |keys: &[&str]| headers.iter().find(|h| keys.contains(&h.as_str()));
    let type_col = find_col(&["type", "data_type", "attribute_type", "value_type"]);
    let def_col = find_col(&["default", "default_bound", "default_value", "bound", "bounds", "value", "nominal", "initial_value"]);
    let unit_col = find_col(&["unit", "units", "engineering_units"]);
    let doc_col = find_col(&["description", "doc", "function", "notes", "comment"]);
    let raw_type = type_col.and_then(|c| row.get(c)).map_or("", |v| v.trim().trim_matches(['`', '*', ' ']));
    let raw_def = def_col.and_then(|c| row.get(c)).map_or("", |v| v.trim().trim_matches(['`', '*', ' ']));
    let raw_desc = doc_col.and_then(|c| row.get(c)).map_or("", |v| v.trim());
    let row_unit = unit_col.and_then(|c| row.get(c)).map_or("", |v| v.trim().trim_matches(['`', '*', ' ']));
    let (num_val, val_unit, int_val) = parse_numeric_with_unit(raw_def);
    let eff_unit = if !row_unit.is_empty() {
        row_unit.to_string()
    } else {
        def_col.and_then(|c| unit_map.get(c)).filter(|u| !u.is_empty()).cloned().or(val_unit).unwrap_or_default()
    };
    let (type_name, default_value) = resolve_typed_default(raw_type, raw_def, int_val, num_val, None);
    let doc_str = compose_sanitized_doc(raw_desc, &row_citation, &eff_unit);
    if !part.attributes.iter().any(|a| a.name == attr_name) {
        part.attributes.push(AttributeDef {
            name: attr_name,
            type_name,
            default_value,
            doc: (!doc_str.is_empty()).then_some(doc_str),
        });
    }
}
