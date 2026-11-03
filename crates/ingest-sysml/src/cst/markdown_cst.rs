//! Normative Markdown CST (`pulldown-cmark` / `deap_core::markdown_ast`) and Blueprint to SysML v2 HIR/AST Translator.
//!
//! **[DOC-TIER: A]**
//! Realises: [REQ-0021] [REQ-0032] [REQ-SYSML-INGEST-MD]

use crate::table::{
    classify_table, clean_cell_value, extract_unit_from_header_or_val, is_table_delimiter_cell,
    normalize_col, sanitize_identifier, MarkdownTable, TableType, NON_COMPONENT_SECTION_KEYWORDS,
};
use deap_core::{ConnectionDef, PackageDef, PartDef};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub use super::{
    md_behavioral::*, md_common::*, md_parametric::*, md_requirements::*, md_tables::*,
};

/// Appends elements from `src` into `dst` when no existing element matches `eq`.
/// Realises: [REQ-SYSML-INGEST-MD/extend_unique]
fn extend_unique<T>(dst: &mut Vec<T>, src: Vec<T>, eq: impl Fn(&T, &T) -> bool) {
    for item in src {
        if !dst.iter().any(|existing| eq(existing, &item)) {
            dst.push(item);
        }
    }
}

/// Merges behavioral definitions (`action_defs`, `interaction_defs`, `state_defs`, `use_case_defs`, `test_case_defs`) from `src` into `dst`.
/// Realises: [REQ-SYSML-INGEST-MD/merge_behavioral_defs]
fn merge_behavioral_defs(dst: &mut PackageDef, src: PackageDef) {
    extend_unique(&mut dst.action_defs, src.action_defs, |a, b| a.name == b.name);
    extend_unique(&mut dst.interaction_defs, src.interaction_defs, |a, b| a.name == b.name);
    extend_unique(&mut dst.state_defs, src.state_defs, |a, b| a.name == b.name);
    extend_unique(&mut dst.use_case_defs, src.use_case_defs, |a, b| a.name == b.name);
    extend_unique(&mut dst.test_case_defs, src.test_case_defs, |a, b| a.name == b.name);
}

/// Consolidates requirement-bearing sub-packages into a hierarchical subsystem `PackageDef` (`[REQ-INGEST-MARKDOWN]`).
fn consolidate_requirement_packages(sub_packages: Vec<PackageDef>, default_name: &str) -> PackageDef {
    let root_name = sanitize_identifier(default_name, "System_Model");
    let mut subsys_map: BTreeMap<String, PackageDef> = BTreeMap::new();
    let mut merged_connections: Vec<ConnectionDef> = Vec::new();
    for mut sub_pkg in sub_packages {
        let sub_name = sub_pkg.parent_package.clone().unwrap_or_else(|| sub_pkg.name.clone());
        let entry = subsys_map.entry(sub_name.clone()).or_insert_with(|| PackageDef {
            name: sub_name.clone(), doc: Some(format!("Subsystem package for {sub_name}")), parent_package: Some(root_name.clone()), ..Default::default()
        });
        extend_unique(&mut entry.requirement_defs, std::mem::take(&mut sub_pkg.requirement_defs), |a, b| a.req_id == b.req_id);
        let parts: Vec<PartDef> = std::mem::take(&mut sub_pkg.part_defs).into_iter().filter(|p| !p.name.starts_with("AC_") && !p.name.starts_with("ac_")).collect();
        extend_unique(&mut entry.part_defs, parts, |a, b| a.name == b.name);
        extend_unique(&mut merged_connections, std::mem::take(&mut sub_pkg.connection_defs), |a, b| a.name == b.name);
        merge_behavioral_defs(entry, sub_pkg);
    }
    let mut packages: Vec<PackageDef> = subsys_map.into_values().collect();
    for p in &mut packages {
        p.requirement_defs.sort_by(|a, b| a.req_id.cmp(&b.req_id));
        if p.part_defs.is_empty() {
            p.part_defs.push(PartDef { name: format!("{}Controller", p.name), doc: Some(format!("Primary controller for {}", p.name)), is_def: true, ..Default::default() });
        }
    }
    PackageDef { name: root_name, doc: Some("Consolidated System Architecture".to_string()), packages, connection_defs: merged_connections, ..Default::default() }
}

/// Consolidates OEM specification sub-packages into a flat merged `PackageDef` (`[REQ-INGEST-MARKDOWN]`).
fn consolidate_oem_packages(sub_packages: Vec<PackageDef>, default_name: &str) -> PackageDef {
    let mut combined_pkg = PackageDef {
        name: sanitize_identifier(default_name, "OEM_System_Model"), doc: Some("Consolidated Level 0 OEM Specifications".to_string()), ..Default::default()
    };
    let mut part_registry: BTreeMap<String, PartDef> = BTreeMap::new();
    for mut sub_pkg in sub_packages {
        for p in std::mem::take(&mut sub_pkg.part_defs) {
            if let Some(existing) = part_registry.get_mut(&p.name) {
                if let Some(p_doc) = p.doc {
                    match existing.doc {
                        Some(ref ex_doc) if !ex_doc.contains(&p_doc) => existing.doc = Some(format!("{ex_doc}\n{p_doc}")),
                        None => existing.doc = Some(p_doc),
                        _ => {}
                    }
                }
                extend_unique(&mut existing.attributes, p.attributes, |a, b| a.name == b.name);
                extend_unique(&mut existing.ports, p.ports, |a, b| a.name == b.name);
                extend_unique(&mut existing.constraints, p.constraints, |a, b| a.name == b.name);
            } else {
                part_registry.insert(p.name.clone(), p);
            }
        }
        extend_unique(&mut combined_pkg.attribute_defs, std::mem::take(&mut sub_pkg.attribute_defs), |a, b| a.name == b.name);
        extend_unique(&mut combined_pkg.port_defs, std::mem::take(&mut sub_pkg.port_defs), |a, b| a.name == b.name);
        extend_unique(&mut combined_pkg.constraint_defs, std::mem::take(&mut sub_pkg.constraint_defs), |a, b| a.name == b.name);
        extend_unique(&mut combined_pkg.connection_defs, std::mem::take(&mut sub_pkg.connection_defs), |a, b| a.name == b.name);
        merge_behavioral_defs(&mut combined_pkg, sub_pkg);
    }
    combined_pkg.part_defs = part_registry.into_values().collect();
    combined_pkg
}

/// Resolves the component target for an H2-H4 Markdown heading (`[REQ-INGEST-MARKDOWN]`).
fn resolve_h2_to_h4_target(raw_header: &str, hash_count: usize, explicit_comps: bool, prev: Option<String>) -> Option<String> {
    let raw_norm: String = raw_header.chars().filter(|&c| c != '*' && c != '`').collect::<String>().trim().to_ascii_lowercase();
    let norm_header = strip_leading_section_number(&raw_norm);
    let clean_key: String = norm_header.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { ' ' }).collect::<String>().split_whitespace().collect::<Vec<_>>().join(" ");
    let is_ac = ["ac-", "ac ", "acceptance criteria"].iter().any(|p| norm_header.starts_with(p) || clean_key.starts_with(p));
    let is_uc = ["uc-", "uc_", "uc ", "use case"].iter().any(|p| norm_header.starts_with(p) || clean_key.starts_with(p));
    let (explicit_comp, comp_cand) = parse_comp_heading(raw_header);
    let is_num_non_comp = !explicit_comp && raw_header.split_once(char::is_whitespace).is_some_and(|(t, _)| !t.is_empty() && t.chars().any(|c| c.is_ascii_digit()) && t.chars().all(|c| c.is_ascii_digit() || c == '.'));
    let padded_key = format!(" {clean_key} ");
    let is_func = !explicit_comp && (is_ac || is_uc || is_num_non_comp
        || NON_COMPONENT_SECTION_KEYWORDS.iter().any(|&kw| padded_key.contains(&format!(" {kw} ")))
        || ["operational context", "executive summary", "command and control", "use cases", "operational use cases", "mathematical formulations", "normative model", "acceptance criteria"].iter().any(|&kw| clean_key.contains(kw)));
    if is_ac || is_func { None } else if explicit_comp || !explicit_comps { Some(comp_cand) } else if hash_count <= 2 { None } else { prev }
}

/// Accumulates Markdown section prose and tables during structural parsing. Realises: [REQ-INGEST-MARKDOWN]
#[derive(Default)]
struct SectionBuilder {
    sec_header: String,
    component_target: Option<String>,
    sec_doc: Vec<String>,
    tables: Vec<MarkdownTable>,
    pending_table: Option<MarkdownTable>,
    sections: Vec<ParsedSection>,
}

impl SectionBuilder {
    /// Initializes a `MarkdownTable` with normalized headers and extracted header units. Realises: [REQ-INGEST-MARKDOWN]
    fn new_table(raw_h: Vec<String>) -> MarkdownTable {
        MarkdownTable {
            normalized_headers: raw_h.iter().map(|c| normalize_col(c)).collect(),
            units: raw_h.iter().map(|c| extract_unit_from_header_or_val(c).1).collect(),
            raw_headers: raw_h, rows: Vec::new(),
        }
    }

    /// Commits a non-empty pending table into the current section's table list. Realises: [REQ-INGEST-MARKDOWN]
    fn flush_table(&mut self) {
        if let Some(t) = self.pending_table.take().filter(|t| !t.raw_headers.is_empty() && !t.rows.is_empty()) {
            self.tables.push(t);
        }
    }

    /// Commits the current accumulated section prose and tables into `self.sections`. Realises: [REQ-INGEST-MARKDOWN]
    fn flush_section(&mut self) {
        self.flush_table();
        if !self.sec_header.is_empty() || !self.tables.is_empty() || !self.sec_doc.is_empty() {
            self.sections.push(ParsedSection {
                component_target: self.component_target.clone(),
                section_doc: self.sec_doc.join(" ").trim().replace("*/", "* /").replace("/*", "/ *"),
                tables: std::mem::take(&mut self.tables),
            });
        }
        self.sec_doc.clear();
    }

    /// Processes an H1-H4 Markdown heading line and updates section/component context. Realises: [REQ-INGEST-MARKDOWN]
    fn handle_heading(&mut self, line_str: &str, pkg_name: &mut String, explicit_comps: bool) -> bool {
        let hash_count = line_str.chars().take_while(|&c| c == '#').count();
        let after_hashes = line_str.get(hash_count..).unwrap_or("");
        if !after_hashes.starts_with(char::is_whitespace) || after_hashes.trim().is_empty() {
            return false;
        }
        let raw_hdr = after_hashes.trim();
        if hash_count == 1 {
            self.flush_section();
            let clean = strip_pkg_words(raw_hdr);
            let cand = if clean.is_empty() { raw_hdr } else { &clean };
            if !pkg_name.starts_with("BP_") {
                *pkg_name = sanitize_identifier(cand, pkg_name);
            }
            self.sec_header = raw_hdr.to_string();
            self.component_target = None;
            return true;
        }
        if (2..=4).contains(&hash_count) {
            self.flush_section();
            self.sec_header = raw_hdr.to_string();
            self.component_target = resolve_h2_to_h4_target(raw_hdr, hash_count, explicit_comps, self.component_target.take());
            return true;
        }
        false
    }

    /// Parses a pipe-delimited Markdown table header, delimiter, or data row. Realises: [REQ-INGEST-MARKDOWN]
    fn process_table_line(&mut self, line_str: &str, next_line: Option<&str>) -> usize {
        if !line_str.contains('|') {
            return 0;
        }
        let cells: Vec<String> = line_str.trim_matches('|').split('|').map(|c| c.trim().to_string()).collect();
        let is_delim = !cells.is_empty() && cells.iter().all(|c| is_table_delimiter_cell(c));
        if is_delim && self.pending_table.is_none() {
            if let Some(hdr) = self.sec_doc.pop() {
                self.pending_table = Some(Self::new_table(hdr.trim_matches('|').split('|').map(|c| c.trim().to_string()).collect()));
                return 1;
            }
        } else if is_delim && self.pending_table.is_some() {
            return 1;
        } else if let Some(ref mut tbl) = self.pending_table {
            let row = tbl.normalized_headers.iter().enumerate().map(|(idx, col)| (col.clone(), cells.get(idx).map_or_else(String::new, |c| clean_cell_value(c)))).collect();
            tbl.rows.push(row);
            return 1;
        } else if next_line.is_some_and(|nl| nl.contains('|')) {
            let next_cells: Vec<&str> = next_line.unwrap_or("").trim().trim_matches('|').split('|').map(str::trim).collect();
            if !next_cells.is_empty() && next_cells.iter().all(|c| is_table_delimiter_cell(c)) {
                self.flush_table();
                self.pending_table = Some(Self::new_table(cells));
                return 2;
            }
        }
        0
    }
}

/// Translates Level 0 OEM Markdown and Blueprint specifications into SysML v2 `PackageDef` ASTs.
/// Realises: [REQ-0021] [REQ-SYSML-INGEST-MD/MarkdownTranslator]
pub struct MarkdownTranslator;

impl Default for MarkdownTranslator {
    /// Returns a default `MarkdownTranslator` instance. Realises: [REQ-INGEST-MARKDOWN]
    fn default() -> Self { Self::new() }
}

impl MarkdownTranslator {
    /// Creates a new `MarkdownTranslator`. Realises: [REQ-SYSML-INGEST-MD/new]
    #[must_use]
    pub const fn new() -> Self { Self }

    /// Extracts YAML frontmatter or Markdown header metadata table and returns `(Frontmatter, remaining_markdown)`. Realises: [REQ-SYSML-INGEST-MD/parse_frontmatter]
    #[must_use]
    pub fn parse_frontmatter(content: &str) -> (Frontmatter, &str) { parse_frontmatter(content) }

    /// Translates a single Markdown document string into a canonical SysML v2 `PackageDef`.
    /// Realises: [REQ-SYSML-INGEST-MD/translate]
    pub fn translate(&self, content: &str, default_name: &str) -> Result<PackageDef, String> {
        let (frontmatter, body_content) = Self::parse_frontmatter(content);
        let (pkg_name, sections) = self.parse_markdown_structure(body_content, default_name, &frontmatter);
        let mut pkg = PackageDef {
            name: pkg_name.clone(),
            doc: Some("Translated from Level 0 OEM Markdown specification".to_string()),
            ..Default::default()
        };
        let mut part_registry: BTreeMap<String, PartDef> = BTreeMap::new();

        for sec in &sections {
            let current_part_name = sec.component_target.as_ref().map(|target| {
                let part_name = sanitize_identifier(target, "Component");
                let entry = part_registry.entry(part_name.clone()).or_insert_with(|| PartDef {
                    name: part_name.clone(),
                    doc: (!sec.section_doc.is_empty()).then(|| sec.section_doc.clone()),
                    ..Default::default()
                });
                if entry.doc.is_none() && !sec.section_doc.is_empty() {
                    entry.doc = Some(sec.section_doc.clone());
                }
                part_name
            });
            for tbl in &sec.tables {
                if is_use_case_table(&tbl.normalized_headers) {
                    process_use_case_table(tbl, &mut pkg);
                    continue;
                }
                let tbl_type = classify_table(&tbl.normalized_headers);
                if matches!(tbl_type, TableType::Bom) {
                    process_bom_table(tbl, &mut part_registry);
                    continue;
                }
                let target_part = current_part_name.as_deref().and_then(|n| part_registry.get_mut(n));
                match tbl_type {
                    TableType::Ports => process_ports_table(tbl, target_part, &mut pkg),
                    TableType::Constraints => process_constraints_table(tbl, target_part, &mut pkg),
                    TableType::Properties => process_properties_table(tbl, target_part, &mut pkg),
                    TableType::Generic => process_generic_table(tbl, target_part, &mut pkg),
                    TableType::Bom => {}
                }
            }
        }
        for (p_name, p_def) in part_registry {
            if !pkg.part_defs.iter().any(|p| p.name == p_name) {
                pkg.part_defs.push(p_def);
            }
        }
        extract_bom_bullet_parts(body_content, &mut pkg);
        if pkg.part_defs.is_empty() && (!pkg.port_defs.is_empty() || !pkg.attribute_defs.is_empty() || !pkg.constraint_defs.is_empty()) {
            pkg.part_defs.push(PartDef {
                name: format!("{pkg_name}_System"),
                doc: Some(format!("Synthesized system component for {pkg_name}")),
                attributes: pkg.attribute_defs.clone(),
                ports: pkg.port_defs.clone(),
                constraints: pkg.constraint_defs.clone(),
                ..Default::default()
            });
        }
        extract_behavioral_ast_nodes(body_content, &mut pkg);
        populate_package_requirement(content, &frontmatter, &mut pkg);
        Ok(pkg)
    }

    /// Translates multiple Markdown specification files in parallel and consolidates them into a single `PackageDef`.
    /// Realises: [REQ-SYSML-INGEST-MD/translate_files]
    pub fn translate_files(&self, file_paths: &[String], default_name: &str) -> Result<PackageDef, String> {
        use rayon::prelude::*;
        if file_paths.is_empty() {
            return Err("Zero Markdown specification files provided to translate_files".to_string());
        }
        let sub_packages: Vec<PackageDef> = file_paths
            .par_iter()
            .map(|fpath| -> Result<PackageDef, String> {
                let path = Path::new(fpath);
                if !path.exists() {
                    return Err(format!("Markdown specification file does not exist: '{}'", path.display()));
                }
                let content = fs::read_to_string(path)
                    .map_err(|e| format!("Failed to read Markdown specification file '{}': {e}", path.display()))?;
                let f_basename = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Module");
                self.translate(&content, f_basename)
                    .map_err(|e| format!("Failed to translate Markdown specification file '{}': {e}", path.display()))
            })
            .collect::<Result<Vec<PackageDef>, String>>()?;

        if sub_packages.iter().any(|p| !p.requirement_defs.is_empty()) {
            Ok(consolidate_requirement_packages(sub_packages, default_name))
        } else {
            Ok(consolidate_oem_packages(sub_packages, default_name))
        }
    }

    /// Internal structure parser decomposing Markdown headings, prose, and tables via `deap_core::markdown_ast`.
    /// Realises: [REQ-SYSML-INGEST-MD/parse_markdown_structure]
    fn parse_markdown_structure(
        &self,
        content: &str,
        default_name: &str,
        frontmatter: &Frontmatter,
    ) -> (String, Vec<ParsedSection>) {
        let ast = deap_core::markdown_ast::parse_markdown_ast(content);
        let lines: Vec<&str> = content.lines().collect();
        let mut pkg_name = frontmatter.title.as_deref().map_or_else(
            || sanitize_identifier(default_name, "Markdown_Package"),
            |t| sanitize_identifier(t, default_name),
        );
        let mut sb = SectionBuilder {
            sections: Vec::with_capacity(ast.sections.len().max(1)),
            ..Default::default()
        };
        let explicit_comps = ast.sections.iter().any(|s| (2..=4).contains(&usize::from(s.level)) && parse_comp_heading(&s.heading).0);

        let (mut i, mut in_code_block) = (0, false);
        while i < lines.len() {
            let line_str = lines.get(i).copied().unwrap_or("").trim();
            if line_str.starts_with("```") || line_str.starts_with("~~~") {
                sb.flush_table();
                in_code_block = !in_code_block;
                i += 1;
                continue;
            }
            if in_code_block || sb.handle_heading(line_str, &mut pkg_name, explicit_comps) {
                i += 1;
                continue;
            }
            let step = sb.process_table_line(line_str, lines.get(i + 1).copied());
            if step > 0 {
                i += step;
                continue;
            }
            if sb.pending_table.is_some() && (!line_str.contains('|') || line_str.is_empty()) {
                sb.flush_table();
            }
            if !line_str.is_empty() && !line_str.starts_with("<!--") && !line_str.starts_with("```") {
                sb.sec_doc.push(line_str.to_string());
            }
            i += 1;
        }
        sb.flush_section();
        (pkg_name, sb.sections)
    }
}
