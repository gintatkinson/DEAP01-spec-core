#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Contract-bounded engineering design blueprint verification gate (`blueprint_gate`).
//! Realises: [REQ-VERIFY-BASELINE-BLUEPRINT-GATE], [DO-330-TQL-4-BLUEPRINT-GATE]

pub use crate::bp_gate_checks_a::{check_4_derivation_table, check_5_components};
pub use crate::bp_gate_checks_b::{check_6_mermaid_syntax, check_7_latex_katex};
pub use crate::bp_gate_checks_c::{check_8_bdd_and_test_target, check_9_rust_code_blocks};
pub use crate::bp_gate_publish::{
    check_11_source_references, check_12_publish_github_ssot, verify_closed_loop_github_payload,
};
pub use crate::bp_gate_sysml::check_10_compile_and_persist_sysml;
pub use crate::bp_gate_tables::{
    check_1_zero_ephemeral_path, check_2_metadata_table, is_empty_or_placeholder_cell,
    is_pascal_case_identifier, is_table_separator_row, is_valid_sysml_property_type,
    split_markdown_table_row, strip_markdown_decorations,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Canonical 7-section headers required in every engineering design blueprint.
/// Realises: [REQ-VERIFY-BASELINE-BLUEPRINT-GATE]
pub const REQUIRED_BLUEPRINT_SECTIONS: [&str; 7] = [
    "## 1. Executive Summary & Derivation-Annotation Contract",
    "## 2. Structural Component Metamodel",
    "## 3. UML & Behavioral Topology",
    "## 4. Formal Mathematical & Safety Constraints",
    "## 5. Acceptance Criteria & Verification Matrix",
    "## 6. Proposed Deterministic Implementation Contract",
    "## 7. Source References",
];

/// Parsed and validated top-of-file CommonMark metadata table for an engineering design blueprint.
/// Realises: [REQ-BLUEPRINT-METADATA-STRUCT]
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BlueprintMetadata {
    pub blueprint_id: String,
    pub requirement_id: String,
    pub title: String,
    pub status: String,
    pub target_crates: String,
    pub tracked_issues: String,
    pub generation_mode: String,
    pub specification_source: String,
}

/// Validated 5-column row from Section 1's Derivation-Annotation Contract table.
/// Realises: [REQ-BLUEPRINT-DERIVATION-ROW]
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DerivationRow {
    pub claim: String,
    pub ssot_anchor: String,
    pub methodology: String,
    pub grade: String,
    pub caveat: String,
}

/// Validated 5-column property row from a Section 2 component attribute table.
/// Realises: [REQ-BLUEPRINT-COMPONENT-PROPERTY]
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ComponentProperty {
    pub attribute: String,
    pub prop_type: String,
    pub default_val: String,
    pub unit: String,
    pub description: String,
}

/// Validated 5-column port row from a Section 2 component port table.
/// Realises: [REQ-BLUEPRINT-COMPONENT-PORT]
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ComponentPort {
    pub port: String,
    pub direction: String,
    pub port_type: String,
    pub protocol: String,
    pub description: String,
}

/// Validated structural component specification extracted from Section 2 (`### Component: <Name>`).
/// Realises: [REQ-BLUEPRINT-COMPONENT-SPEC]
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ComponentSpec {
    pub name: String,
    pub properties: Vec<ComponentProperty>,
    pub ports: Vec<ComponentPort>,
}

/// Complete verification report and artifact paths produced by the blueprint verification gate.
/// Realises: [REQ-BLUEPRINT-REPORT]
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BlueprintReport {
    pub path: PathBuf,
    pub sysml_path: PathBuf,
    pub digest_path: PathBuf,
    pub metadata: BlueprintMetadata,
    pub derivation_rows: Vec<DerivationRow>,
    pub components: Vec<ComponentSpec>,
    pub ac_count: usize,
    pub test_target: String,
    pub test_targets: Vec<String>,
    pub errors: Vec<String>,
}

/// Command-line options for the `verify-baseline check-blueprint` subcommand.
/// Realises: [REQ-BLUEPRINT-CLI-OPTIONS]
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BlueprintCliOptions {
    pub path: PathBuf,
    pub repo: Option<String>,
    pub issue: Option<u64>,
    pub title: Option<String>,
    pub publish: bool,
    pub dry_run: bool,
}

/// Check 3 (7-Section Structural Completeness Gate): Verifies exact presence and ascending line order of the 7 H2 sections.
/// Realises: [REQ-BLUEPRINT-CHECK-3-SECTIONS]
pub fn check_3_sections(content: &str) -> Result<Vec<(usize, usize, &'static str)>, Vec<String>> {
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();
    let mut in_fence = false;
    let mut found_positions: [Vec<usize>; 7] = Default::default();
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if !in_fence {
            for (sec_idx, required_header) in REQUIRED_BLUEPRINT_SECTIONS.iter().enumerate() {
                if trimmed == *required_header {
                    if let Some(pos_list) = found_positions.get_mut(sec_idx) {
                        pos_list.push(idx);
                    }
                }
            }
        }
    }
    let mut errors = Vec::new();
    let mut start_indices: Vec<(usize, &'static str)> = Vec::with_capacity(7);
    for (sec_idx, required_header) in REQUIRED_BLUEPRINT_SECTIONS.iter().enumerate() {
        let positions: &[usize] = found_positions.get(sec_idx).map_or(&[], Vec::as_slice);
        if positions.is_empty() {
            errors.push(format!("Check 3 failed: missing required section header '{required_header}'"));
        } else if positions.len() > 1 {
            errors.push(format!("Check 3 failed: duplicate section header '{required_header}' found {} times", positions.len()));
        } else if let Some(&first_pos) = positions.first() {
            start_indices.push((first_pos, *required_header));
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    let mut prev_pos: Option<(usize, &'static str)> = None;
    for &(pos, header) in &start_indices {
        if let Some((p_pos, p_header)) = prev_pos {
            if pos <= p_pos {
                errors.push(format!(
                    "Check 3 failed: section header '{header}' (line {}) appears before '{p_header}' (line {})",
                    pos.saturating_add(1),
                    p_pos.saturating_add(1)
                ));
            }
        }
        prev_pos = Some((pos, header));
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    let mut ranges = Vec::with_capacity(7);
    for (i, &(start_idx, header)) in start_indices.iter().enumerate() {
        let end_idx = start_indices.get(i.saturating_add(1)).map_or(total_lines, |&(next_start, _)| next_start);
        ranges.push((start_idx, end_idx, header));
    }
    Ok(ranges)
}

/// Evaluates section-scoped Checks 4 through 9 and 11 over the 7 section ranges from `check_3_sections`.
/// Realises: [REQ-BLUEPRINT-VERIFY-ORCHESTRATOR]
fn evaluate_blueprint_sections(
    workspace_root: &Path,
    content: &str,
    ranges: &[(usize, usize, &'static str)],
    report: &mut BlueprintReport,
    errors: &mut Vec<String>,
) {
    let lines: Vec<&str> = content.lines().collect();

    if let Some(&(s1_start, s1_end, _)) = ranges.first() {
        match check_4_derivation_table(lines.get(s1_start..s1_end).unwrap_or(&[])) {
            Ok(rows) => report.derivation_rows = rows,
            Err(errs) => errors.extend(errs),
        }
    }
    if let Some(&(s2_start, s2_end, _)) = ranges.get(1) {
        match check_5_components(lines.get(s2_start..s2_end).unwrap_or(&[])) {
            Ok(comps) => report.components = comps,
            Err(errs) => errors.extend(errs),
        }
    }
    if let Some(&(s3_start, s3_end, _)) = ranges.get(2) {
        if let Err(errs) = check_6_mermaid_syntax(lines.get(s3_start..s3_end).unwrap_or(&[])) {
            errors.extend(errs);
        }
    }
    if let Some(&(s4_start, s4_end, _)) = ranges.get(3) {
        if let Err(errs) = check_7_latex_katex(lines.get(s4_start..s4_end).unwrap_or(&[]), content) {
            errors.extend(errs);
        }
    }
    if let Some(&(s5_start, s5_end, _)) = ranges.get(4) {
        match check_8_bdd_and_test_target(lines.get(s5_start..s5_end).unwrap_or(&[])) {
            Ok((count, targets)) => {
                report.ac_count = count;
                report.test_target = targets.join(", ");
                report.test_targets = targets;
            }
            Err(errs) => errors.extend(errs),
        }
    }
    if let Some(&(s6_start, s6_end, _)) = ranges.get(5) {
        if let Err(errs) = check_9_rust_code_blocks(lines.get(s6_start..s6_end).unwrap_or(&[])) {
            errors.extend(errs);
        }
    }
    if let Some(&(s7_start, s7_end, _)) = ranges.get(6) {
        if let Err(errs) = check_11_source_references(workspace_root, lines.get(s7_start..s7_end).unwrap_or(&[])) {
            errors.extend(errs);
        }
    }
}

/// Orchestrates all 12 blueprint verification gates (`Check 1` through `Check 12`) in sequence.
/// Realises: [REQ-BLUEPRINT-VERIFY-ORCHESTRATOR]
pub fn verify_blueprint(
    workspace_root: &Path,
    options: &BlueprintCliOptions,
) -> Result<BlueprintReport, Vec<String>> {
    let resolved_path =
        check_1_zero_ephemeral_path(workspace_root, &options.path).map_err(|e| vec![e])?;
    let content = std::fs::read_to_string(&resolved_path).map_err(|e| {
        vec![format!("Check 1 failed: unable to read blueprint file '{}': {e}", resolved_path.display())]
    })?;

    let mut errors: Vec<String> = Vec::new();
    let metadata = match check_2_metadata_table(&content) {
        Ok(m) => m,
        Err(errs) => {
            errors.extend(errs);
            BlueprintMetadata::default()
        }
    };

    let mut report = BlueprintReport {
        path: resolved_path.clone(),
        metadata: metadata.clone(),
        ..Default::default()
    };

    match check_3_sections(&content) {
        Ok(ranges) => {
            evaluate_blueprint_sections(workspace_root, &content, &ranges, &mut report, &mut errors);
        }
        Err(errs) => errors.extend(errs),
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    let (sysml_path, digest_path) = check_10_compile_and_persist_sysml(&resolved_path, &content)?;
    report.sysml_path = sysml_path;
    report.digest_path = digest_path;

    if options.publish && !options.dry_run {
        let mut publish_opts = options.clone();
        publish_opts.path = resolved_path;
        check_12_publish_github_ssot(&publish_opts, &metadata, &content)?;
    }

    Ok(report)
}

/// Executes the `verify-baseline check-blueprint` CLI runner, returning `0` on pass or `1` on failure.
/// Realises: [REQ-BLUEPRINT-RUN-CLI]
pub fn run_blueprint_cli(workspace_root: &Path, options: &BlueprintCliOptions) -> i32 {
    match verify_blueprint(workspace_root, options) {
        Ok(report) => {
            println!("[PASS] Check 1: Zero-Ephemeral Path Gate ({})", report.path.display());
            println!(
                "[PASS] Check 2: CommonMark Metadata Table Gate ({} / {})",
                report.metadata.blueprint_id, report.metadata.requirement_id
            );
            println!("[PASS] Check 3: 7-Section Structural Completeness Gate (7/7 sections)");
            println!(
                "[PASS] Check 4: Derivation-Annotation Contract Gate ({} rows)",
                report.derivation_rows.len()
            );
            println!(
                "[PASS] Check 5: Component & 5-Column Property/Port Table Gate ({} components)",
                report.components.len()
            );
            println!("[PASS] Check 6: Mermaid E1-E5 Syntax Gate");
            println!("[PASS] Check 7: LaTeX / KaTeX Integrity Gate");
            println!(
                "[PASS] Check 8: BDD Acceptance Criteria & Test-Target Gate ({} ACs -> {})",
                report.ac_count, report.test_target
            );
            println!("[PASS] Check 9: Check 34 Zero Regex, Check 19 Domain Cleanliness & Clippy-Forbid Gate");
            println!("[PASS] Check 10: Deterministic In-Repo SysML v2 Compilation & Bijective Roundtrip Gate");
            println!("       SysML:  {}", report.sysml_path.display());
            println!("       Digest: {}", report.digest_path.display());
            println!("[PASS] Check 11: Source Reference Existence Gate");
            if options.publish && !options.dry_run {
                println!("[PASS] Check 12: Closed-Loop GitHub SSOT Publishing Gate [CLOSED-LOOP VERIFIED]");
            } else {
                println!("[PASS] Check 12: Closed-Loop GitHub SSOT Publishing Gate [DRY-RUN: READY TO PUBLISH]");
            }
            0
        }
        Err(errors) => {
            eprintln!("[FAIL] Blueprint verification gate rejected '{}':", options.path.display());
            for err in &errors {
                eprintln!("  - {err}");
            }
            1
        }
    }
}
