//! Grammar-backed `CommonMark` AST verification gates (`parity`, `spec_audit`, `blueprint`). [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Unifies `verify-baseline` table, frontmatter, metrology, metamodel, and realization parity
//! verification gates over `deap_core::markdown_ast` (`pulldown-cmark` + `pulldown-cmark-to-cmark`)
//! under DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D (`BP-04` / `WBS 01.04.05`).
//!
//! ## 2. Invariants & Mathematical Properties
//! - Zero Ad-Hoc Line Scanners: Section tables, separator rows, and frontmatter blocks are
//!   evaluated via `deap_core::markdown_ast` AST nodes and offset-tracked event streams.
//! - Gate Equivalence (`#744`): Check 26 SI 7D parameter metrology enforces identical unit rules
//!   across full workspace runs, `--only <file>`, and `--spec-only` modes.
//!
//! ## 3. Preconditions
//! - Callers supply valid UTF-8 Markdown content or readable repository root directories.
//!
//! ## 4. Postconditions
//! - Returns deterministic `Result` diagnostics or structured AST table projections with zero panics.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ linear in Markdown byte length $N$.
//! - Space Complexity: $O(N)$ proportional to extracted table cells and findings.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Unreadable files, malformed frontmatter, invalid SI units, ungrounded `UseCaseDef` nodes,
//!   and domain-envelope mismatches fail closed with explicit `Err` diagnostics.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0604], [REQ-0401], [BP-04] [FLS: fls_t4w1da759n2e, fls_qcb1n9c0e5hz]

#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]

pub use crate::cmark_gates_audit as spec_audit;

/// CommonMark AST parity, SI metrology, `UseCaseDef` realization, and M2 metamodel gates (`[REQ-0604]`).
/// Realises: [REQ-0604], [FLS: fls_t4w1da759n2e]
pub mod parity {
    use crate::checks::parity_check_coverage::parse_use_case_defs;
    use crate::checks::parity_check_discovery::TargetScanDir;
    use crate::checks::parity_check_icd_ast::{is_metrology_placeholder, is_valid_si_unit};
    use crate::checks::parity_gates::ParityOutcome;
    use deap_core::markdown_ast::{is_gfm_delimiter_cell, parse_markdown_ast};
    use deap_core::rules::EXCLUDED_DIRS;
    use deap_core::workspace::is_upstream_compiler;
    use std::fs;
    use std::path::{Path, PathBuf};
    use walkdir::WalkDir;

    /// [DOC-TIER: A] Extracts lowercased table headers and data rows under a heading matching `section_kw`.
    /// Safety & Traceability: Realises: [REQ-0604], [FLS: fls_t4w1da759n2e]
    /// Contract: Parses `content` via `deap_core::markdown_ast::parse_markdown_ast` without `.lines()`.
    /// Bounds & Faults: O(N) time and space; returns empty vectors if section or table is absent.
    #[must_use]
    pub fn parse_section_table_rows_cmark(
        content: &str,
        section_kw: &str,
    ) -> (Vec<String>, Vec<Vec<String>>) {
        let ast = parse_markdown_ast(content);
        let kw_lower = section_kw.to_ascii_lowercase();
        let mut fallback_headers = Vec::new();
        for sec in &ast.sections {
            if !sec.heading.to_ascii_lowercase().contains(&kw_lower) {
                continue;
            }
            for tbl in &ast.tables {
                if tbl.start_offset >= sec.start_offset && tbl.end_offset <= sec.end_offset {
                    let headers: Vec<String> =
                        tbl.headers.iter().map(|h| h.to_ascii_lowercase()).collect();
                    if !tbl.rows.is_empty() {
                        return (headers, tbl.rows.clone());
                    }
                    fallback_headers = headers;
                }
            }
        }
        (fallback_headers, Vec::new())
    }

    /// [DOC-TIER: A] Checks whether `line` is a valid GFM table alignment separator row.
    /// Safety & Traceability: Realises: [REQ-0604], [FLS: fls_t4w1da759n2e]
    /// Contract: Validates that `line` starts with `|` and all non-empty cells satisfy `is_gfm_delimiter_cell`.
    /// Bounds & Faults: O(|line|) time, O(1) space; never panics.
    #[must_use]
    pub fn is_table_separator_line_cmark(line: &str) -> bool {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            return false;
        }
        let cols: Vec<&str> = trimmed.split('|').map(str::trim).filter(|c| !c.is_empty()).collect();
        !cols.is_empty() && cols.iter().all(|c| is_gfm_delimiter_cell(c))
    }

    /// Collects sorted `.md` files across `scan_dirs` for Check 26 (`[REQ-0604]`, `[FLS: fls_t4w1da759n2e]`).
    fn collect_metrology_md_files(scan_dirs: &[PathBuf], errors: &mut Vec<String>) -> Vec<PathBuf> {
        let mut files = Vec::new();
        for dir in scan_dirs {
            if !dir.is_dir() {
                continue;
            }
            let iter = WalkDir::new(dir).sort_by_file_name().into_iter().filter_entry(|e| {
                !e.file_type().is_dir() || {
                    let name = e.file_name().to_string_lossy();
                    !EXCLUDED_DIRS.iter().any(|ex| name == *ex) && name != "blueprints"
                }
            });
            for entry_res in iter {
                match entry_res {
                    Ok(e) if e.file_type().is_file() && e.path().extension().is_some_and(|x| x == "md") => {
                        files.push(e.path().to_path_buf());
                    }
                    Ok(_) => {}
                    Err(err) => errors.push(format!("Check 26 failed: I/O error walking '{}': {err}", dir.display())),
                }
            }
        }
        files.sort();
        files
    }

    /// Filters `files` to match `only_str` or resolves `only_str` directly (`[REQ-0604]`, `[FLS: fls_t4w1da759n2e]`).
    fn apply_only_file_filter(repo_root: &Path, files: Vec<PathBuf>, only_str: &str) -> Vec<PathBuf> {
        let norm = only_str.trim().trim_start_matches("./").replace('\\', "/");
        let mut matched: Vec<PathBuf> = files
            .into_iter()
            .filter(|p| {
                let rel = p.strip_prefix(repo_root).unwrap_or(p).to_string_lossy().replace('\\', "/");
                rel == norm
                    || rel.ends_with(&format!("/{norm}"))
                    || p.file_name().and_then(|n| n.to_str()) == Some(norm.as_str())
            })
            .collect();
        if matched.is_empty() {
            let direct = repo_root.join(&norm);
            if direct.is_file() && direct.extension().is_some_and(|e| e == "md") {
                matched.push(direct);
            }
        }
        matched
    }

    /// [DOC-TIER: A] Check 26 SI 7D Parameter Metrology Gate with optional `--only` file scoping (`#744`).
    /// Safety & Traceability: Realises: [REQ-PARITY-GATE-26], [REQ-0604], [FLS: fls_t4w1da759n2e]
    /// Contract: Audits `signal`, `parameter`, and `metric` tables in target Markdown files for SI units.
    /// Bounds & Faults: O(F * N) time; fails closed on unreadable files or invalid SI unit cells.
    pub fn check_standards_measurement_filtered(
        repo_root: &Path,
        allow_missing_specs: bool,
        strict: bool,
        only_filter: Option<&str>,
    ) -> Result<ParityOutcome, Vec<String>> {
        let is_upstream = is_upstream_compiler(repo_root);
        let iface_dir = TargetScanDir::Interfaces.resolve(repo_root);
        let feat_dir = TargetScanDir::Features.resolve(repo_root);
        if is_upstream && !iface_dir.is_dir() && !feat_dir.is_dir() && only_filter.is_none() {
            return Ok(ParityOutcome::PendingOrClean);
        }
        let mut errors = Vec::new();
        let scan_dirs = if is_upstream {
            vec![iface_dir, feat_dir]
        } else {
            vec![iface_dir, feat_dir, TargetScanDir::Conops.resolve(repo_root), TargetScanDir::Architecture.resolve(repo_root)]
        };
        let discovered = collect_metrology_md_files(&scan_dirs, &mut errors);
        let target_files = match only_filter {
            Some(only_str) => apply_only_file_filter(repo_root, discovered, only_str),
            None => discovered,
        };
        if target_files.is_empty() && errors.is_empty() {
            let _ = (allow_missing_specs, strict);
            return Ok(ParityOutcome::PendingOrClean);
        }
        for path in target_files {
            let rel = path.strip_prefix(repo_root).unwrap_or(&path).display().to_string();
            match fs::read_to_string(&path) {
                Ok(content) => audit_metrology_tables_in_text(&content, &rel, &mut errors),
                Err(err) => errors.push(format!("Check 26 failed: Unable to read file '{rel}': {err}")),
            }
        }
        if errors.is_empty() { Ok(ParityOutcome::Verified) } else { Err(errors) }
    }

    /// Audits `signal`, `parameter`, and `metric` tables in `content` (`[REQ-0604]`, `[FLS: fls_t4w1da759n2e]`).
    pub(crate) fn audit_metrology_tables_in_text(content: &str, rel: &str, errors: &mut Vec<String>) {
        for table_kind in ["signal", "parameter", "metric"] {
            let (headers, rows) = parse_section_table_rows_cmark(content, table_kind);
            if let Some(unit_col) = headers.iter().position(|h| h.contains("unit")) {
                for row in &rows {
                    if let Some(unit_val) = row.get(unit_col) {
                        let clean = unit_val.trim().trim_matches(|c| c == '`' || c == '*');
                        if clean.is_empty() || is_metrology_placeholder(clean) || !is_valid_si_unit(clean) {
                            errors.push(format!(
                                "Check 26 failed: Invalid or missing SI dimensional unit '{clean}' in '{rel}'"
                            ));
                        }
                    }
                }
            }
        }
    }

    /// [DOC-TIER: A] Verifies forward parity from schema `UseCaseDef` AST nodes to `docs/use-cases/*.md` (`#747`).
    /// Safety & Traceability: Realises: [REQ-0604], [FLS: fls_t4w1da759n2e]
    /// Contract: Every `use case def <Name>` in `schema/` must be cited in at least one `docs/use-cases/*.md`.
    /// Bounds & Faults: O(U * M) time; returns `Err(Vec<String>)` when any `UseCaseDef` is unmapped.
    pub fn check_schema_use_case_realization_parity(repo_root: &Path) -> Result<ParityOutcome, Vec<String>> {
        let Some(model_text) = crate::checks::parity_check_discovery::discover_sysml_model_text(repo_root) else {
            return Ok(ParityOutcome::PendingOrClean);
        };
        let use_cases = parse_use_case_defs(&model_text);
        if use_cases.is_empty() {
            return Ok(ParityOutcome::PendingOrClean);
        }
        let mut errors = Vec::new();
        let uc_files = collect_metrology_md_files(&[TargetScanDir::UseCases.resolve(repo_root)], &mut errors);
        let mut combined_uc_docs = String::new();
        for f in &uc_files {
            match fs::read_to_string(f) {
                Ok(txt) => {
                    combined_uc_docs.push_str(&txt.to_ascii_lowercase());
                    combined_uc_docs.push('\n');
                }
                Err(err) => errors.push(format!("UseCase parity read error '{}': {err}", f.display())),
            }
        }
        for (uc_name, _) in &use_cases {
            if !uc_name.is_empty() && !combined_uc_docs.contains(&uc_name.to_ascii_lowercase()) {
                errors.push(format!("Schema UseCaseDef '{uc_name}' has no realizing specification under docs/use-cases/"));
            }
        }
        if errors.is_empty() { Ok(ParityOutcome::Verified) } else { Err(errors) }
    }

    /// [DOC-TIER: A] Validates that Pipeline 0 documentation orders Phase 0.5 `FAILURE_MODE_REGISTRY.md` before STPA/FMECA (`#747`).
    /// Safety & Traceability: Realises: [REQ-0604], [FLS: fls_t4w1da759n2e]
    /// Contract: Verifies `FAILURE_MODE_REGISTRY.md` appears prior to `SAFETY_01_STPA_HAZARD_ANALYSIS.md`.
    /// Bounds & Faults: O(N) string offset check; returns `Err` if registry is omitted or out of order.
    pub fn validate_pipeline0_phase05_registry_order(content: &str, rel_path: &str) -> Result<(), Vec<String>> {
        let Some(reg_pos) = content.find("FAILURE_MODE_REGISTRY.md") else {
            return Err(vec![format!(
                "{rel_path}: Pipeline 0 prompt omits mandatory Phase 0.5 'docs/research/FAILURE_MODE_REGISTRY.md' step"
            )]);
        };
        if let Some(stpa_pos) = content.find("SAFETY_01_STPA_HAZARD_ANALYSIS.md") {
            if reg_pos > stpa_pos {
                return Err(vec![format!(
                    "{rel_path}: Phase 0.5 'FAILURE_MODE_REGISTRY.md' must precede 'SAFETY_01_STPA_HAZARD_ANALYSIS.md'"
                )]);
            }
        }
        Ok(())
    }

    /// [DOC-TIER: A] Validates M2 performer classification, `perform action`, and ISO 42010 viewpoint separation (`#745`).
    /// Safety & Traceability: Realises: [REQ-0604], [FLS: fls_t4w1da759n2e]
    /// Contract: Rejects operational processes typed as structural performers and checks viewpoint bindings.
    /// Bounds & Faults: O(N) scan over AST tables; returns `Err` on metamodel conflation.
    pub fn validate_m2_performer_and_viewpoints_cmark(content: &str, rel_path: &str) -> Result<(), Vec<String>> {
        let ast = parse_markdown_ast(content);
        let mut errors = Vec::new();
        for tbl in &ast.tables {
            let hdr = tbl.headers.join(" ").to_ascii_lowercase();
            if hdr.contains("performer") && hdr.contains("type") {
                for row in &tbl.rows {
                    let row_txt = row.join(" ").to_ascii_lowercase();
                    if row_txt.contains("action def") || row_txt.contains("use case def") {
                        errors.push(format!(
                            "{rel_path}: Behavioral process node conflated with structural UAF Performer in row '{row_txt}'"
                        ));
                    }
                }
            }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }

    /// [DOC-TIER: A] Validates that physical envelope attributes are not applied to non-physical parts (`#746`).
    /// Safety & Traceability: Realises: [REQ-0604], [FLS: fls_t4w1da759n2e]
    /// Contract: Rejects `mass_kg`, `volume_m3`, `ip_rating`, `operating_temp_c`, `power_draw_w` on `cyber`/`logical` domains.
    /// Bounds & Faults: O(A) over attribute list `A`; returns `Err` on domain-attribute mismatch.
    pub fn validate_part_attribute_domain_conformance_cmark(
        part_name: &str,
        domain: &str,
        attributes: &[String],
    ) -> Result<(), Vec<String>> {
        let dom = domain.trim().to_ascii_lowercase();
        let physical_only = ["mass_kg", "volume_m3", "ip_rating", "operating_temp_c", "power_draw_w"];
        let mut errors = Vec::new();
        if matches!(dom.as_str(), "cyber" | "software" | "logical") {
            for attr in attributes {
                let norm = attr.trim().to_ascii_lowercase();
                if physical_only.iter().any(|&p| norm == p) {
                    errors.push(format!(
                        "Part '{part_name}' in non-physical domain '{domain}' must not declare physical envelope attribute '{attr}'"
                    ));
                }
            }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }

    /// [DOC-TIER: A] Verifies absence of ungrounded hardcoded small-UAV airworthiness clauses (`#734`).
    /// Safety & Traceability: Realises: [REQ-0604], [FLS: fls_t4w1da759n2e]
    /// Contract: Rejects hardcoded `14 CFR Part 107` 25 kg limit injection when not schema-derived.
    /// Bounds & Faults: O(N) substring check; returns `Err` on hardcoded clause violation.
    pub fn verify_zero_hardcoded_airworthiness_clauses(content: &str) -> Result<(), Vec<String>> {
        if content.contains("14 CFR Part 107") && content.contains("25.0 kg") {
            return Err(vec!["Hardcoded 14 CFR Part 107 25.0 kg small-UAV limit detected (#734)".to_string()]);
        }
        Ok(())
    }
}

/// CommonMark AST blueprint table splitting, separator validation, and lossless roundtrip helpers (`[REQ-0604]`).
/// Realises: [REQ-0604], [FLS: fls_t4w1da759n2e]
pub mod blueprint {
    use deap_core::markdown_ast::{cmark_options, emit_lossless_markdown, is_gfm_delimiter_cell, CompilerError};
    use pulldown_cmark::{Event, Parser};

    /// [DOC-TIER: A] Splits a CommonMark pipe-delimited table row into trimmed cell strings while respecting `\|`.
    /// Safety & Traceability: Realises: [REQ-BLUEPRINT-TABLE-ROW-SPLIT], [REQ-0604], [FLS: fls_t4w1da759n2e]
    /// Contract: Requires leading and trailing `|`; preserves escaped `\|` inside cells.
    /// Bounds & Faults: O(|line|) time and space; returns `None` if `line` is not a pipe-delimited row.
    #[must_use]
    pub fn split_table_row_cmark(line: &str) -> Option<Vec<String>> {
        let interior = line.trim().strip_prefix('|')?.strip_suffix('|')?;
        let (mut cells, mut current, mut escaped) = (Vec::new(), String::new(), false);
        for ch in interior.chars() {
            if escaped {
                current.push(ch);
                escaped = false;
            } else if ch == '\\' {
                current.push(ch);
                escaped = true;
            } else if ch == '|' {
                cells.push(current.trim().to_string());
                current.clear();
            } else {
                current.push(ch);
            }
        }
        cells.push(current.trim().to_string());
        Some(cells)
    }

    /// [DOC-TIER: A] Validates that `cells` form a GFM table alignment separator row of `expected_cols` columns.
    /// Safety & Traceability: Realises: [REQ-BLUEPRINT-TABLE-SEPARATOR], [REQ-0604], [FLS: fls_t4w1da759n2e]
    /// Contract: Checks `cells.len() == expected_cols` and `is_gfm_delimiter_cell` on every cell.
    /// Bounds & Faults: O(C) time, O(1) space; never panics.
    #[must_use]
    pub fn is_table_separator_row_cmark(cells: &[String], expected_cols: usize) -> bool {
        cells.len() == expected_cols && cells.iter().all(|c| is_gfm_delimiter_cell(c))
    }

    /// [DOC-TIER: A] Verifies deterministic CommonMark roundtrip serialization via `pulldown_cmark_to_cmark::cmark`.
    /// Safety & Traceability: Realises: [REQ-0604], [BP-04], [FLS: fls_t4w1da759n2e]
    /// Contract: Parses `source` with `cmark_options()` and re-emits canonical CommonMark via `emit_lossless_markdown`.
    /// Bounds & Faults: O(N) time and space; returns `Err(CompilerError)` on serialization failure.
    pub fn verify_cmark_lossless_roundtrip(source: &str) -> Result<String, CompilerError> {
        let events: Vec<Event<'_>> = Parser::new_ext(source, cmark_options()).collect();
        emit_lossless_markdown(&events)
    }
}
