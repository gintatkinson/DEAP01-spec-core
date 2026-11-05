//! Check 27: Cross-Document Diagram Parity Gate (`check_cross_document_diagram_parity`) and `CONN-*` collectors.
//!
//! Safety & Traceability: Realises: [REQ-PARITY-GATE-27], [REQ-CONN-ID-SET-EXTRACTION]
//! Contract: Enforces 1:1 bidirectional `CONN-*` parity across ConOps SV-1, `ICD_01`, executive reports, and SSOT Architecture Views.
//! Bounds & Faults: Deterministic sorted path scanning and BTreeSet difference evaluation; fails closed on mismatches.

use super::parity_check_discovery::TargetScanDir;
use super::parity_gates::ParityOutcome;
use deap_core::workspace::is_upstream_compiler_workspace;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// Extracts all connection identifiers (`CONN-01`..`CONN-N`) from text without regex.
/// Realises: [REQ-CONN-ID-SET-EXTRACTION]
pub fn extract_connection_ids(text: &str) -> BTreeSet<String> {
    let mut conns = BTreeSet::new();
    let bytes = text.as_bytes();
    let mut i: usize = 0;
    while i.saturating_add(5) <= bytes.len() {
        let Some(prefix) = bytes.get(i..i.saturating_add(5)) else {
            i += 1;
            continue;
        };
        let valid_leading = i == 0
            || bytes
                .get(i.saturating_sub(1))
                .is_none_or(|&b| !b.is_ascii_alphanumeric() && b != b'_' && b != b'-');
        if prefix.eq_ignore_ascii_case(b"CONN-") && valid_leading {
            let mut end = i.saturating_add(5);
            if bytes
                .get(end)
                .copied()
                .is_some_and(|b| b.is_ascii_alphanumeric())
            {
                while let Some(&b) = bytes.get(end) {
                    if b.is_ascii_alphanumeric() || b == b'_' || b == b'-' {
                        end += 1;
                    } else {
                        break;
                    }
                }
                if let Some(slice) = text.get(i..end) {
                    conns.insert(slice.to_ascii_uppercase());
                }
                i = end;
                continue;
            }
        }
        i += 1;
    }
    conns
}

/// Recursively collects connection identifiers from Markdown and Mermaid files in a directory.
/// Realises: [REQ-DETERMINISTIC-CONNECTION-COLLECTION]
pub(crate) fn collect_connections_from_path(dir: &Path, conns: &mut BTreeSet<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        let mut entry_list: Vec<_> = entries.flatten().collect();
        entry_list.sort_by_key(|e| e.file_name());
        for entry in entry_list {
            let path = entry.path();
            if path.is_dir() {
                collect_connections_from_path(&path, conns);
            } else if path.is_file()
                && path.extension().is_some_and(|e| e == "md" || e == "mmd")
            {
                if let Ok(content) = fs::read_to_string(&path) {
                    conns.extend(extract_connection_ids(&content));
                }
            }
        }
    }
}

/// Check 27: Cross-Document Diagram Parity Gate (Gate 25B).
/// Realises: [REQ-PARITY-GATE-27]
pub fn check_cross_document_diagram_parity(repo_root: &Path) -> Result<ParityOutcome, Vec<String>> {
    let arch_views_dir = TargetScanDir::Architecture.resolve(repo_root).join("views");
    if is_upstream_compiler_workspace(repo_root) {
        if !arch_views_dir.is_dir() {
            return Err(vec![
                "Canonical SSOT Architecture Views Library (docs/architecture/views/) is missing in upstream compiler workspace (mandating canonical architecture views library per refs #276)".to_string(),
            ]);
        }
        if !arch_views_dir.join("sv1_super_system.mmd").is_file() {
            return Err(vec![
                "Canonical SSOT Architecture Views Library (docs/architecture/views/) is missing required canonical view sv1_super_system.mmd (refs #276)".to_string(),
            ]);
        }
    }

    let mut conns_arch = BTreeSet::new();
    if arch_views_dir.is_dir() {
        collect_connections_from_path(&arch_views_dir, &mut conns_arch);
    }

    let mut conns_conops = BTreeSet::new();
    let conops_dir = TargetScanDir::Conops.resolve(repo_root);
    if conops_dir.is_dir() {
        collect_connections_from_path(&conops_dir, &mut conns_conops);
    }
    let conops_file = repo_root.join("docs").join("CONOPS.md");
    if conops_file.is_file() {
        if let Ok(c) = fs::read_to_string(&conops_file) {
            conns_conops.extend(extract_connection_ids(&c));
        }
    }

    let mut conns_icd = BTreeSet::new();
    let interfaces_dir = TargetScanDir::Interfaces.resolve(repo_root);
    if interfaces_dir.is_dir() {
        collect_connections_from_path(&interfaces_dir, &mut conns_icd);
    }
    let icd_dir = TargetScanDir::Icd.resolve(repo_root);
    if icd_dir.is_dir() {
        collect_connections_from_path(&icd_dir, &mut conns_icd);
    }

    let mut conns_reports = BTreeSet::new();
    let reports_dir = TargetScanDir::Reports.resolve(repo_root);
    if reports_dir.is_dir() {
        collect_connections_from_path(&reports_dir, &mut conns_reports);
    }

    let mut mismatches = Vec::new();
    if conns_conops.is_empty() && !conns_icd.is_empty() && conops_dir.is_dir() {
        mismatches.push(format!(
            "Asymmetric connection declarations: ConOps declares 0 connections, while Level 1C System Interface Matrix declares {} connections.",
            conns_icd.len()
        ));
    } else if !conns_conops.is_empty() && conns_icd.is_empty() && interfaces_dir.is_dir() {
        mismatches.push(format!(
            "Asymmetric connection declarations: ConOps declares {} connections, while Level 1C System Interface Matrix declares 0 connections.",
            conns_conops.len()
        ));
    }

    compare_connection_sets(
        (&conns_arch, &conns_conops, &conns_icd, &conns_reports),
        &mut mismatches,
    );

    if mismatches.is_empty() {
        Ok(ParityOutcome::Verified)
    } else {
        mismatches.sort();
        mismatches.dedup();
        Err(mismatches)
    }
}

/// Compares pairwise connection sets across Architecture Views, ConOps, ICD, and Executive Reports.
/// Realises: [REQ-PARITY-GATE-27]
fn compare_connection_sets(
    sets: (
        &BTreeSet<String>,
        &BTreeSet<String>,
        &BTreeSet<String>,
        &BTreeSet<String>,
    ),
    mismatches: &mut Vec<String>,
) {
    let (conns_arch, conns_conops, conns_icd, conns_reports) = sets;
    let arch_label = "Canonical SSOT Architecture Library (docs/architecture/views/)";
    let conops_label = "ConOps SV-1 diagram";
    let icd_label = "Level 1C System Interface Matrix";
    let report_label = "Executive engineering report";
    let report_lower = "executive engineering report";

    if !conns_arch.is_empty() {
        push_conn_diffs(
            (conns_conops, conops_label, "declares"),
            (conns_arch, arch_label),
            "refs #276",
            mismatches,
        );
        push_conn_diffs(
            (conns_icd, icd_label, "registers"),
            (conns_arch, arch_label),
            "refs #276",
            mismatches,
        );
        if !conns_reports.is_empty() {
            for conn in conns_reports.difference(conns_arch) {
                mismatches.push(format!(
                    "{} declares connection '{}', but it is missing from {} (mandating 1:1 bidirectional parity per refs #276)",
                    report_label, conn, arch_label
                ));
            }
            for conn in conns_arch.difference(conns_reports) {
                mismatches.push(format!(
                    "{} registers connection '{}', but it is missing from {} (mandating 1:1 bidirectional parity per refs #276)",
                    arch_label, conn, report_lower
                ));
            }
        }
    }
    push_conn_diffs(
        (conns_conops, conops_label, "declares"),
        (conns_icd, icd_label),
        "refs #271",
        mismatches,
    );
    if !conns_reports.is_empty() {
        for (other, other_label, verb) in [
            (conns_conops, conops_label, "declares"),
            (conns_icd, icd_label, "registers"),
        ] {
            if !other.is_empty() {
                for conn in conns_reports.difference(other) {
                    mismatches.push(format!(
                        "{} declares connection '{}', but it is missing from {} (mandating 1:1 bidirectional parity per refs #276)",
                        report_label, conn, other_label
                    ));
                }
                for conn in other.difference(conns_reports) {
                    mismatches.push(format!(
                        "{} {} connection '{}', but it is missing from {} (mandating 1:1 bidirectional parity per refs #276)",
                        other_label, verb, conn, report_lower
                    ));
                }
            }
        }
    }
}

/// Appends bidirectional set-difference diagnostics between two non-empty connection sets.
/// Realises: [REQ-PARITY-GATE-27]
fn push_conn_diffs(
    left: (&BTreeSet<String>, &str, &str),
    right: (&BTreeSet<String>, &str),
    issue_ref: &str,
    mismatches: &mut Vec<String>,
) {
    let (lhs, lhs_label, lhs_verb) = left;
    let (rhs, rhs_label) = right;
    if lhs.is_empty() || rhs.is_empty() {
        return;
    }
    for conn in lhs.difference(rhs) {
        mismatches.push(format!(
            "{} {} connection '{}', but it is missing from {} (mandating 1:1 bidirectional parity per {})",
            lhs_label, lhs_verb, conn, rhs_label, issue_ref
        ));
    }
    for conn in rhs.difference(lhs) {
        mismatches.push(format!(
            "{} registers connection '{}', but it is missing from {} (mandating 1:1 bidirectional parity per {})",
            rhs_label, conn, lhs_label, issue_ref
        ));
    }
}
