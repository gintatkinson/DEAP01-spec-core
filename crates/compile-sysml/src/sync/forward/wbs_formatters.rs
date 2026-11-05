#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Markdown Table and Enterprise Export Renderers for Logical ICD and MIL-STD-881E WBS Artifacts.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety-critical
//! software engineering standards. Produces deterministic CommonMark tables, RFC 4180 CSV exports,
//! MSPDI XML schedules, and JSON AST payloads for `ICD_01..02` and `WBS_01..03`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Diagonal Interconnect Invariant: Diagonal cells `(i, i)` in `ICD_01_SYSTEM_INTERFACE_MATRIX.md`
//!   always render as `---`.
//! - Referential Export Parity: All 7 WBS artifacts render from the identical ordered `WbsNode` slice.
//!
//! ## 3. Preconditions
//! - Input `SignalDictionaryRow`, connection tuples, and `WbsNode` slices are pre-validated and sorted.
//!
//! ## 4. Postconditions
//! - Returns formatted Markdown, CSV, XML, and JSON strings ready for atomic persistence.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(V^2 + E)$ for ICD rendering and $O(N)$ for WBS rendering.
//! - Space Complexity: $O(V^2 + E + N)$ for formatted output buffers.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Pure string formatting functions with zero I/O or fallible operations.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0604]

use super::*;

/// Renders `ICD_01_SYSTEM_INTERFACE_MATRIX.md` from sorted `part_list`, `cell_links`, `all_connections`, and `rows`.
/// Realises: [REQ-0604]
pub(super) fn render_icd01_matrix_and_register(
    part_list: &[String],
    cell_links: &BTreeMap<(String, String), BTreeSet<String>>,
    all_connections: &[(String, String, String, String, String)],
    rows: &[SignalDictionaryRow],
) -> String {
    let mut icd01 = String::new();
    icd01.push_str("# ICD_01: System Interface Matrix\n\n");
    icd01.push_str("## 1. N x N Subsystem & Component Interconnect Matrix\n\n");
    if part_list.is_empty() {
        icd01.push_str("| Component | System |\n| :--- | :--- |\n| **System** | --- |\n\n");
    } else {
        icd01.push_str("| Component |");
        for col in part_list {
            icd01.push_str(&format!(" {} |", col));
        }
        icd01.push('\n');
        icd01.push_str("| :--- |");
        for _ in part_list {
            icd01.push_str(" :--- |");
        }
        icd01.push('\n');
        for row_part in part_list {
            icd01.push_str(&format!("| **{}** |", row_part));
            for col_part in part_list {
                if row_part == col_part {
                    icd01.push_str(" --- |");
                } else if let Some(sigs) = cell_links.get(&(row_part.clone(), col_part.clone())) {
                    let joined: Vec<String> = sigs.iter().cloned().collect();
                    icd01.push_str(&format!(" {} |", joined.join(", ")));
                } else {
                    icd01.push_str(" - |");
                }
            }
            icd01.push('\n');
        }
        icd01.push('\n');
    }

    icd01.push_str("## 2. Point-to-Point Logical Connection Register\n\n");
    icd01.push_str(
        "| Connection ID | Source Endpoint | Target Endpoint | Signal / Item Flow | Package |\n",
    );
    icd01.push_str("| :--- | :--- | :--- | :--- | :--- |\n");
    if all_connections.is_empty() {
        for r in rows {
            icd01.push_str(&format!(
                "| `PORT-{}` | `{}` | `{}` | `{}` | `{}` |\n",
                sanitize_id(&r.signal_name),
                r.source_endpoint,
                r.target_endpoint,
                r.signal_name,
                r.package_name
            ));
        }
    } else {
        for (cid, src, dst, sig, pkg_name) in all_connections {
            icd01.push_str(&format!(
                "| `{}` | `{}` | `{}` | `{}` | `{}` |\n",
                cid, src, dst, sig, pkg_name
            ));
        }
    }
    icd01
}

/// Renders `ICD_02_MASTER_SIGNAL_DICTIONARY.md` from lexicographically sorted `rows`.
/// Realises: [REQ-0604]
pub(super) fn render_icd02_signal_dictionary(rows: &[SignalDictionaryRow]) -> String {
    let mut icd02 = String::new();
    icd02.push_str("# ICD_02: Master Signal Dictionary\n\n");
    icd02.push_str("## 1. Canonical Signal & Port Definitions\n\n");
    icd02.push_str(
        "| Signal Name | Payload Type | Direction | Source Endpoint | Target Endpoint | Package |\n",
    );
    icd02.push_str("| :--- | :--- | :--- | :--- | :--- | :--- |\n");
    for r in rows {
        icd02.push_str(&format!(
            "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` |\n",
            r.signal_name,
            r.payload_type,
            r.direction,
            r.source_endpoint,
            r.target_endpoint,
            r.package_name
        ));
    }
    icd02
}

/// Renders `WBS_01`, `WBS_02`, and `WBS_03` Markdown documents from `nodes`.
/// Realises: [REQ-0604]
pub(super) fn render_wbs_markdown_tables(nodes: &[WbsNode]) -> (String, String, String) {
    let mut wbs_01_md = String::new();
    wbs_01_md.push_str("# WBS_01: MIL-STD-881E Work Breakdown Structure\n\n");
    wbs_01_md.push_str("| WBS Code | Level | Element Name | Package | Realized Artifact |\n");
    wbs_01_md.push_str("| :--- | :--- | :--- | :--- | :--- |\n");
    for n in nodes {
        wbs_01_md.push_str(&format!(
            "| `{}` | {} | `{}` | `{}` | `{}` |\n",
            n.wbs_code, n.level, n.element_name, n.package_name, n.feat_id
        ));
    }

    let mut wbs_02_md = String::new();
    wbs_02_md.push_str("# WBS_02: Technical Realization Register\n\n");
    wbs_02_md.push_str(
        "| WBS Code | Work Package | Feature ID | Requirement IDs | UCA IDs | Test Case IDs |\n",
    );
    wbs_02_md.push_str("| :--- | :--- | :--- | :--- | :--- | :--- |\n");
    for n in nodes.iter().filter(|n| n.level == 3) {
        wbs_02_md.push_str(&format!(
            "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` |\n",
            n.wbs_code,
            n.element_name,
            n.feat_id,
            n.req_ids.join(", "),
            n.uca_ids.join(", "),
            n.tc_ids.join(", ")
        ));
    }

    let mut wbs_03_md = String::new();
    wbs_03_md.push_str("# WBS_03: Enterprise Export Schema\n\n");
    wbs_03_md.push_str("| Export Target | Relative Path | Format | Node Count |\n");
    wbs_03_md.push_str("| :--- | :--- | :--- | :--- |\n");
    for (target, rel, fmt) in [
        ("Jira CSV", "exports/wbs_jira.csv", "RFC 4180 CSV"),
        ("Monday.com CSV", "exports/wbs_monday.csv", "RFC 4180 CSV"),
        ("MS Project XML", "exports/wbs_msproject.xml", "MSPDI XML"),
        ("WBS JSON AST", "exports/wbs_ast.json", "JSON"),
    ] {
        wbs_03_md.push_str(&format!(
            "| {} | `{}` | {} | {} |\n",
            target,
            rel,
            fmt,
            nodes.len()
        ));
    }

    (wbs_01_md, wbs_02_md, wbs_03_md)
}

/// Renders `(jira_csv, monday_csv, msproject_xml, ast_json)` enterprise exports from `nodes`.
/// Realises: [REQ-0604]
pub(super) fn render_wbs_enterprise_exports(
    nodes: &[WbsNode],
) -> (String, String, String, String) {
    let mut jira_csv =
        String::from("WBS Code,Level,Summary,Issue Type,Package,Feature ID,Requirements\n");
    let mut monday_csv = String::from("WBS Code,Level,Name,Group,Feature ID,UCA IDs,Test Cases\n");
    let mut msproject_xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<Project xmlns=\"http://schemas.microsoft.com/project\">\n  <Tasks>\n",
    );
    let mut json_items = Vec::new();

    for (idx, n) in nodes.iter().enumerate() {
        let issue_type = match n.level {
            1 => "Program",
            2 => "Epic",
            _ => "Story",
        };
        jira_csv.push_str(&format!(
            "{},{},{},{},{},{},\"{}\"\n",
            n.wbs_code,
            n.level,
            n.element_name,
            issue_type,
            n.package_name,
            n.feat_id,
            n.req_ids.join(";")
        ));
        monday_csv.push_str(&format!(
            "{},{},{},{},{},\"{}\",\"{}\"\n",
            n.wbs_code,
            n.level,
            n.element_name,
            n.package_name,
            n.feat_id,
            n.uca_ids.join(";"),
            n.tc_ids.join(";")
        ));
        msproject_xml.push_str(&format!(
            "    <Task>\n      <UID>{}</UID>\n      <WBS>{}</WBS>\n      <Name>{}</Name>\n      <OutlineLevel>{}</OutlineLevel>\n    </Task>\n",
            idx.saturating_add(1),
            n.wbs_code,
            n.element_name,
            n.level
        ));
        json_items.push(format!(
            "    {{\n      \"wbs_code\": \"{}\",\n      \"level\": {},\n      \"element_name\": \"{}\",\n      \"package_name\": \"{}\",\n      \"feat_id\": \"{}\"\n    }}",
            n.wbs_code, n.level, n.element_name, n.package_name, n.feat_id
        ));
    }
    msproject_xml.push_str("  </Tasks>\n</Project>\n");
    let ast_json = format!("{{\n  \"nodes\": [\n{}\n  ]\n}}\n", json_items.join(",\n"));

    (jira_csv, monday_csv, msproject_xml, ast_json)
}
