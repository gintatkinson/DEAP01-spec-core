//! ConOps & Mission Intent modular assembly, link relativization, and verification engine.

mod document;
mod integrity;
mod links;

#[cfg(test)]
mod tests_integrity;
#[cfg(test)]
mod tests_units_arch;
#[cfg(test)]
mod tests_units_env;
#[cfg(test)]
mod tests_units_ops;

use std::fs;
use std::path::{Path, PathBuf};

pub use document::assemble_document;
pub use integrity::{find_raw_tokens, validate_unit_integrity};
pub use links::{path_relative_from, relativize_markdown_links};

use crate::params::SysMLParameterBindingEngine;

/// Canonical unit file whitelists for deterministic ConOps & Mission Intent assembly.
pub static CANONICAL_CONOPS_UNITS: &[&str] = &[
    "01_METADATA_AND_OVERVIEW.md",
    "02_DEFICIENCIES_AND_MOTIVATION.md",
    "03_PROPOSED_CAPABILITIES.md",
    "04_SYSTEM_ARCHITECTURE.md",
    "04_USER_CLASSES_AND_STAKEHOLDERS.md",
    "05_OPERATIONAL_STATE_SPACE_AND_RISK.md",
    "06_UAF_OPERATIONAL_ACTIVITIES.md",
    "07_OPTX_EXCHANGES.md",
    "08_SYSTEM_ARCHITECTURE.md",
    "08_ENVIRONMENTAL_OPERATING_LIMITS.md",
    "08_ENVIRONMENTAL_MIL_STD_810H.md",
    "09_SCENARIOS_AND_TIMELINES.md",
    "10_MAINTENANCE_AND_GSE_SUPPORT.md",
    "11_IMPACTS_AND_TRADE_STUDIES.md",
    "12_EMERGENCY_DECISION_MATRIX.md",
];

pub static CANONICAL_MISSION_INTENT_UNITS: &[&str] = &[
    "01_COMMANDERS_INTENT.md",
    "02_MISSION_ESSENTIAL_TASK_LIST.md",
    "03_INCOSE_MOE_MOP_MATH.md",
    "04_MULTI_DOMAIN_THREAT_MATRIX.md",
    "05_PACE_C2_PLAN.md",
    "06_SAFETY_INTERLOCKS.md",
    "06_ROE_SAFETY_INTERLOCKS.md",
    "06_RULES_OF_ENGAGEMENT.md",
    "07_AIRSPACE_GEOZONES.md",
    "08_GO_NO_GO_MATRIX.md",
    "09_ENERGY_AND_RESERVE_BOUNDS.md",
    "09_BINGO_ENERGY_MATH.md",
    "10_OPERATIONAL_ALLOCATION_TAGS.md",
];

/// Resolves the workspace root directory from explicit override or ancestor heuristics (`[REQ-ASSEMBLE-CONOPS]`).
fn resolve_workspace_root(
    input_dir: &Path,
    output_dir: &Path,
    workspace_dir: Option<&Path>,
) -> PathBuf {
    if let Some(ws) = workspace_dir {
        return ws.to_path_buf();
    }
    for cand in &[
        output_dir.to_path_buf(),
        output_dir.join(".."),
        output_dir.join("..").join(".."),
        input_dir.to_path_buf(),
        input_dir.join(".."),
        input_dir.join("..").join(".."),
    ] {
        if cand.join("schema").is_dir() || cand.join(".pipeline").is_dir() {
            return cand.canonicalize().unwrap_or_else(|_| cand.clone());
        }
    }
    input_dir.to_path_buf()
}

/// Discovers a unit directory (`conops` or `mission_intent`) under `input_dir` (`[REQ-ASSEMBLE-CONOPS]`).
fn find_unit_subdir(input_dir: &Path, detected_dom: &str, leaf: &str) -> Option<PathBuf> {
    [
        input_dir.join(detected_dom).join(leaf),
        input_dir.join("units").join(detected_dom).join(leaf),
        input_dir.join(leaf),
        input_dir.join("units").join(leaf),
    ]
    .into_iter()
    .find(|cand| cand.is_dir())
}

/// Writes a compiled document to `output_dir/filename` after relativizing markdown links (`[REQ-ASSEMBLE-CONOPS]`).
fn write_compiled_doc(
    doc: String,
    output_dir: &Path,
    ws_path: &Path,
    filename: &str,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(output_dir)?;
    let out_file = output_dir.join(filename);
    let relativized = relativize_markdown_links(&doc, &out_file, ws_path);
    fs::write(&out_file, relativized)?;
    println!("[+] Successfully wrote compiled {label} to '{}'.", out_file.display());
    Ok(())
}

/// [DOC-TIER: A] Orchestrates the assembly, parameter binding, and validation of both `CONOPS.md` and `MISSION_INTENT.md`.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-ASSEMBLER-ASSEMBLE-CONOPS] | SysML v2: ActionDef(assemble_conops)
/// Contract: Discovers ConOps and Mission Intent unit directories under `input_dir`, binds SysML parameters, and writes/verifies outputs in `output_dir`.
/// Bounds & Faults: O(U * N) time and space over unit files; returns `Ok(false)` when validation gates fail without partial output mutation.
pub fn assemble_conops(
    input_dir: &Path,
    output_dir: &Path,
    verify_only: bool,
    params: Option<&Path>,
    (domain, workspace_dir): (Option<&str>, Option<&Path>),
) -> Result<bool, Box<dyn std::error::Error>> {
    println!(
        "[*] ConOps Assembly Engine starting: input='{}', output='{}', verify_only={}, domain={}",
        input_dir.display(), output_dir.display(), verify_only, domain.unwrap_or("None")
    );
    let ws_path = resolve_workspace_root(input_dir, output_dir, workspace_dir);
    let mut engine = SysMLParameterBindingEngine::new(None, params, Some(&ws_path), true, domain);
    let detected_dom = domain.map_or_else(|| engine.detected_domain.clone(), str::to_string);
    let mut all_errors = Vec::new();

    if let Some(c_dir) = find_unit_subdir(input_dir, &detected_dom, "conops") {
        println!("[*] Assembling Concept of Operations from '{}' [domain={detected_dom}]...", c_dir.display());
        let (doc, errs) = assemble_document(&c_dir, Some("Concept of Operations (ConOps)"), (None, None), &mut engine, Some(CANONICAL_CONOPS_UNITS));
        if !errs.is_empty() {
            all_errors.extend(errs.into_iter().map(|e| format!("[CONOPS] {e}")));
        } else if !verify_only {
            write_compiled_doc(doc, output_dir, &ws_path, "CONOPS.md", "ConOps")?;
        }
    } else {
        println!("[-] ConOps units directory not found in '{}'. Skipping CONOPS.md assembly.", input_dir.display());
    }

    if let Some(m_dir) = find_unit_subdir(input_dir, &detected_dom, "mission_intent") {
        println!("[*] Assembling Tactical Mission Intent from '{}' [domain={detected_dom}]...", m_dir.display());
        let (doc, errs) = assemble_document(&m_dir, Some("Tactical Mission Intent & Execution Plan"), (None, None), &mut engine, Some(CANONICAL_MISSION_INTENT_UNITS));
        if !errs.is_empty() {
            all_errors.extend(errs.into_iter().map(|e| format!("[MISSION_INTENT] {e}")));
        } else if !verify_only {
            write_compiled_doc(doc, output_dir, &ws_path, "MISSION_INTENT.md", "Mission Intent")?;
        }
    } else {
        println!("[-] Mission Intent units directory not found in '{}'. Skipping MISSION_INTENT.md assembly.", input_dir.display());
    }

    if !all_errors.is_empty() {
        eprintln!("\n[!] ConOps Assembly Errors encountered:");
        for err in &all_errors {
            eprintln!("    - {err}");
        }
        return Ok(false);
    }
    println!("[+] All ConOps assembly and verification checks passed cleanly.");
    Ok(true)
}
