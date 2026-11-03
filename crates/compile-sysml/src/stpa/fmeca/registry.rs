#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! FMECA Failure Mode Registry Markdown Parser and Workspace Loader.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, IEC 61508, and ECSS-E-ST-40C
//! safety objectives for module `crates/compile-sysml/src/stpa/fmeca/registry.rs`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Linear single-pass Markdown table parser with zero `regex` dependencies and zero direct slice indexing.
//! - Preserves source row ordering, clamps all parsed `(S, O, D)` scores to `[1, 10]`, and tracks explicit SSOT provenance.
//!
//! ## 3. Preconditions
//! - Caller supplies UTF-8 Markdown content or a valid safety output directory path.
//!
//! ## 4. Postconditions
//! - Returns deterministic `Vec<RegistryFailureEntry>` records covering all valid component failure rows.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(N) over input Markdown byte length.
//! - Space Complexity: O(R) over parsed `RegistryFailureEntry` record count.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Gracefully skips malformed lines, separator lines, and incomplete rows.
//! - State: Resets column bindings whenever contiguous table blocks end.
//! - Action: Executes deterministically with zero panics or `.unwrap()`.
//! - Resource: Bounded heap allocations proportional to valid failure rows.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603], [REQ-STPA-FMECA-PARSE-REGISTRY] | SysML v2: PackageDef(fmeca_registry)

use std::fs;
use std::path::Path;

use super::{template_for_dimension, FailureDimension, RegistryFailureEntry};

/// Column index mapping resolved from a Markdown table header in `FAILURE_MODE_REGISTRY.md`.
/// Realises: [REQ-STPA-FMECA-REGISTRY-TABLE-COLUMNS]
#[derive(Debug, Clone, Default)]
struct RegistryTableColumns {
    component_col: Option<usize>,
    dimension_col: Option<usize>,
    failure_mode_col: Option<usize>,
    effect_col: Option<usize>,
    severity_col: Option<usize>,
    occurrence_col: Option<usize>,
    detection_col: Option<usize>,
    mitigation_col: Option<usize>,
    basis_col: Option<usize>,
}

/// Parses Markdown tables from `FAILURE_MODE_REGISTRY.md` into deterministic `RegistryFailureEntry` records.
/// Realises: [REQ-STPA-FMECA-PARSE-REGISTRY]
pub fn parse_failure_mode_registry(markdown: &str) -> Vec<RegistryFailureEntry> {
    let mut entries = Vec::new();
    let mut active_cols: Option<RegistryTableColumns> = None;
    let mut heading_component: Option<String> = None;

    for line in markdown.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix('#') {
            active_cols = None;
            let title = rest.trim_start_matches('#').trim();
            let cleaned = title
                .strip_prefix("Component:")
                .or_else(|| title.strip_prefix("component:"))
                .unwrap_or(title)
                .trim()
                .trim_matches('`')
                .trim();
            if !cleaned.is_empty() {
                heading_component = Some(cleaned.to_string());
            }
            continue;
        }

        let Some(cells) = split_markdown_table_cells(trimmed) else {
            if !trimmed.is_empty() {
                active_cols = None;
            }
            continue;
        };

        if is_markdown_separator_row(&cells) {
            continue;
        }

        if let Some(cols) = detect_registry_header_columns(&cells) {
            active_cols = Some(cols);
            continue;
        }

        if let Some(cols) = active_cols.as_ref() {
            if let Some(entry) = parse_registry_data_row(&cells, cols, heading_component.as_deref())
            {
                entries.push(entry);
            }
        }
    }

    entries
}

/// Loads and parses `FAILURE_MODE_REGISTRY.md` relative to the target safety `out_dir`.
/// Realises: [REQ-STPA-FMECA-LOAD-REGISTRY-OUTDIR]
pub fn load_failure_mode_registry_for_out_dir(out_dir: &Path) -> Vec<RegistryFailureEntry> {
    let parent = out_dir.parent().unwrap_or_else(|| Path::new("."));
    let candidates = [
        parent.join("research").join("FAILURE_MODE_REGISTRY.md"),
        parent
            .join("docs")
            .join("research")
            .join("FAILURE_MODE_REGISTRY.md"),
        out_dir
            .join("..")
            .join("research")
            .join("FAILURE_MODE_REGISTRY.md"),
        out_dir
            .join("..")
            .join("docs")
            .join("research")
            .join("FAILURE_MODE_REGISTRY.md"),
    ];

    for candidate in &candidates {
        if let Ok(content) = fs::read_to_string(candidate) {
            return parse_failure_mode_registry(&content);
        }
    }

    Vec::new()
}

/// Splits a single Markdown pipe-table line into trimmed, unquoted cell strings.
/// Realises: [REQ-STPA-FMECA-SPLIT-MD-CELLS]
fn split_markdown_table_cells(trimmed: &str) -> Option<Vec<String>> {
    let without_leading = trimmed.strip_prefix('|')?;
    let body = without_leading.strip_suffix('|').unwrap_or(without_leading);
    let cells: Vec<String> = body
        .split('|')
        .map(|cell| {
            cell.trim()
                .trim_matches('`')
                .trim_matches('*')
                .trim()
                .to_string()
        })
        .collect();
    if cells.is_empty() {
        None
    } else {
        Some(cells)
    }
}

/// Determines whether a parsed Markdown row is a pipe-table alignment separator (`| :--- | :---: |`).
/// Realises: [REQ-STPA-FMECA-MD-SEPARATOR-PREDICATE]
fn is_markdown_separator_row(cells: &[String]) -> bool {
    !cells.is_empty()
        && cells.iter().all(|cell| {
            cell.contains('-')
                && cell
                    .chars()
                    .all(|ch| ch == '-' || ch == ':' || ch.is_whitespace())
        })
}

/// Detects a `FAILURE_MODE_REGISTRY.md` table header row and resolves its column coordinates.
/// Realises: [REQ-STPA-FMECA-DETECT-REGISTRY-HEADER]
fn detect_registry_header_columns(cells: &[String]) -> Option<RegistryTableColumns> {
    let mut cols = RegistryTableColumns::default();
    for (idx, cell) in cells.iter().enumerate() {
        let lower = cell.to_ascii_lowercase();
        classify_registry_header_cell(idx, &lower, &mut cols);
    }

    let dim_idx = cols.dimension_col?;
    let _mode_idx = cols.failure_mode_col?;
    let dim_header = cells.get(dim_idx)?;
    if FailureDimension::from_label(dim_header).is_some() {
        return None;
    }
    Some(cols)
}

/// Maps a single lowercased header token onto its corresponding `RegistryTableColumns` slot.
/// Realises: [REQ-STPA-FMECA-CLASSIFY-HEADER-CELL]
fn classify_registry_header_cell(idx: usize, lower: &str, cols: &mut RegistryTableColumns) {
    if cols.component_col.is_none()
        && (lower.contains("component") || lower.contains("part") || lower.contains("subsystem"))
    {
        cols.component_col = Some(idx);
    } else if cols.dimension_col.is_none() && lower.contains("dimension") {
        cols.dimension_col = Some(idx);
    } else if cols.failure_mode_col.is_none()
        && (lower.contains("failure mode") || lower.contains("failure_mode") || lower == "mode")
    {
        cols.failure_mode_col = Some(idx);
    } else if cols.effect_col.is_none()
        && (lower.contains("effect") || lower.contains("consequence") || lower.contains("impact"))
    {
        cols.effect_col = Some(idx);
    } else if cols.severity_col.is_none()
        && (lower == "s" || lower == "sev" || lower.contains("severity"))
    {
        cols.severity_col = Some(idx);
    } else if cols.occurrence_col.is_none()
        && (lower == "o"
            || lower == "occ"
            || lower.contains("occurrence")
            || lower.contains("likelihood"))
    {
        cols.occurrence_col = Some(idx);
    } else if cols.detection_col.is_none()
        && (lower == "d" || lower == "det" || lower.contains("detection"))
    {
        cols.detection_col = Some(idx);
    } else if cols.mitigation_col.is_none()
        && (lower.contains("mitigation")
            || lower.contains("control")
            || lower.contains("countermeasure"))
    {
        cols.mitigation_col = Some(idx);
    } else if cols.basis_col.is_none()
        && (lower.contains("basis")
            || lower.contains("provenance")
            || lower.contains("evidence")
            || lower.contains("source")
            || lower.contains("citation"))
    {
        cols.basis_col = Some(idx);
    }
}

/// Converts a single parsed Markdown data row into a validated `RegistryFailureEntry`.
/// Realises: [REQ-STPA-FMECA-PARSE-REGISTRY-ROW]
fn parse_registry_data_row(
    cells: &[String],
    cols: &RegistryTableColumns,
    fallback_component: Option<&str>,
) -> Option<RegistryFailureEntry> {
    let dim_raw = cols.dimension_col.and_then(|i| cells.get(i))?;
    let dimension = FailureDimension::from_label(dim_raw)?;
    let failure_mode = cols
        .failure_mode_col
        .and_then(|i| cells.get(i))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())?
        .to_string();
    let component = cols
        .component_col
        .and_then(|i| cells.get(i))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .or(fallback_component)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())?
        .to_string();

    let tmpl = template_for_dimension(dimension);
    let effect = cols
        .effect_col
        .and_then(|i| cells.get(i))
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    let mitigation = cols
        .mitigation_col
        .and_then(|i| cells.get(i))
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    let severity = cols
        .severity_col
        .and_then(|i| cells.get(i))
        .map_or(tmpl.default_s, |s| parse_clamped_score(s, tmpl.default_s));
    let occurrence = cols
        .occurrence_col
        .and_then(|i| cells.get(i))
        .map_or(tmpl.default_o, |s| parse_clamped_score(s, tmpl.default_o));
    let detection = cols
        .detection_col
        .and_then(|i| cells.get(i))
        .map_or(tmpl.default_d, |s| parse_clamped_score(s, tmpl.default_d));
    let basis_cell = cols
        .basis_col
        .and_then(|i| cells.get(i))
        .map(String::as_str);

    Some(RegistryFailureEntry {
        component,
        dimension,
        failure_mode,
        effect,
        severity,
        occurrence,
        detection,
        mitigation,
        is_ssot_evidenced: parse_ssot_evidence_flag(basis_cell),
    })
}

/// Extracts and clamps an integer FMECA score (`1..=10`) from a Markdown table cell.
/// Realises: [REQ-STPA-FMECA-PARSE-CLAMPED-SCORE]
fn parse_clamped_score(raw: &str, default_score: u32) -> u32 {
    let digits: String = raw
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits
        .parse::<u32>()
        .map(|n| n.clamp(1, 10))
        .unwrap_or_else(|_| default_score.clamp(1, 10))
}

/// Evaluates whether a registry row's basis/evidence cell qualifies for `"SSOT"` provenance.
/// Realises: [REQ-STPA-FMECA-SSOT-EVIDENCE-FLAG]
fn parse_ssot_evidence_flag(basis_cell: Option<&str>) -> bool {
    let Some(raw) = basis_cell else {
        return true;
    };
    let lower = raw.trim().to_ascii_lowercase();
    !(lower.is_empty()
        || lower == "-"
        || lower == "n/a"
        || lower == "none"
        || lower == "tbd"
        || lower == "pending"
        || lower == "unverified"
        || lower.contains("derived"))
}
