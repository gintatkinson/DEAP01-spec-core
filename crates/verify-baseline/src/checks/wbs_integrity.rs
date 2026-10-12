//! Check 20: WBS & Enterprise Deliverables Suite Validation.

use regex::Regex;
use serde_json::Value;
use std::fs;
use std::path::Path;

/// Outcome of the WBS check.
#[derive(Debug, PartialEq, Eq)]
pub enum WbsCheckOutcome {
    PendingOrNotPresent,
    Verified,
}

pub const REQUIRED_CSV_HEADERS: &[&str] = &[
    "WBS Code",
    "Name",
    "Level",
    "Type",
    "Parent WBS",
    "Duration (Days)",
    "Start Date",
    "Finish Date",
    "Predecessors",
    "Assigned Resources",
    "Jira Epic / Issue Key",
    "Monday / MS Project Mapping",
];

/// Check 20: WBS & Enterprise Deliverables Suite Validation.
pub fn check_wbs_suite_integrity(
    repo_root: &Path,
    allow_missing_specs: bool,
    strict: bool,
) -> Result<WbsCheckOutcome, Vec<String>> {
    let wbs_md = repo_root.join("docs").join("management").join("WBS_DELIVERABLES_SUITE.md");
    let wbs_dir = repo_root.join("docs").join("wbs");

    if !wbs_md.is_file() && !wbs_dir.is_dir() {
        if allow_missing_specs && !strict {
            return Ok(WbsCheckOutcome::PendingOrNotPresent);
        } else if strict {
            return Err(vec![
                "Check 20 failed: WBS & Enterprise Deliverables Suite ('docs/management/WBS_DELIVERABLES_SUITE.md') is missing in downstream customer mode.".to_string()
            ]);
        } else {
            return Ok(WbsCheckOutcome::PendingOrNotPresent);
        }
    }

    let mut errors = Vec::new();

    let target_md = if wbs_md.is_file() {
        wbs_md.clone()
    } else {
        wbs_dir.join("WBS_DELIVERABLES_SUITE.md")
    };

    let wbs_csv = repo_root.join("docs").join("management").join("wbs_export_jira_monday_ms_project.csv");
    let wbs_json = repo_root.join("docs").join("management").join("wbs_export.json");

    if !wbs_csv.is_file() {
        errors.push(format!(
            "Missing enterprise export deliverable: {}",
            wbs_csv.strip_prefix(repo_root).unwrap_or(&wbs_csv).display()
        ));
    }
    if !wbs_json.is_file() {
        errors.push(format!(
            "Missing enterprise export deliverable: {}",
            wbs_json.strip_prefix(repo_root).unwrap_or(&wbs_json).display()
        ));
    }

    // Validate WBS_DELIVERABLES_SUITE.md content
    if target_md.is_file() {
        match fs::read_to_string(&target_md) {
            Ok(content) => {
                // Check 2-column Metadata Table
                let has_meta_table = content.contains("| Attribute | Specification Detail |")
                    || content.contains("|Attribute|Specification Detail|");
                if !has_meta_table {
                    errors.push("WBS_DELIVERABLES_SUITE.md missing 2-column Metadata Table (| Attribute | Specification Detail |).".to_string());
                }

                // Check 7-Column Traceability Matrix
                let has_traceability = content.contains("WBS Code")
                    && content.contains("Deliverable / Task Name")
                    && content.contains("Spec Reference")
                    && content.contains("MBD Package / Model File")
                    && content.contains("Verification Artifact")
                    && content.contains("Target Issue Key")
                    && content.contains("Owner / Role");
                if !has_traceability {
                    errors.push("WBS_DELIVERABLES_SUITE.md missing 7-Column Traceability Matrix header.".to_string());
                }

                // Check internal markdown hyperlinks resolve
                let link_re = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();
                let parent_dir = target_md.parent().unwrap_or(repo_root);
                for cap in link_re.captures_iter(&content) {
                    let target = &cap[2];
                    if !target.starts_with("http://")
                        && !target.starts_with("https://")
                        && !target.starts_with('#')
                        && !target.starts_with("mailto:")
                    {
                        let clean_target = target.split('#').next().unwrap_or(target);
                        if !clean_target.is_empty() {
                            let resolved = parent_dir.join(clean_target);
                            if !resolved.exists() {
                                errors.push(format!(
                                    "Broken markdown link in WBS_DELIVERABLES_SUITE.md: '{}' (resolved to: {})",
                                    target,
                                    resolved.display()
                                ));
                            }
                        }
                    }
                }
            }
            Err(e) => {
                errors.push(format!("Failed to read {}: {}", target_md.display(), e));
            }
        }
    }

    // Validate CSV headers
    if wbs_csv.is_file() {
        if let Ok(content) = fs::read_to_string(&wbs_csv) {
            if let Some(first_line) = content.lines().next() {
                let headers: Vec<&str> = first_line.split(',').map(|s| s.trim().trim_matches('"')).collect();
                for req in REQUIRED_CSV_HEADERS {
                    if !headers.iter().any(|&h| h == *req) {
                        errors.push(format!("wbs_export_jira_monday_ms_project.csv missing required header: '{}'", req));
                    }
                }
            } else {
                errors.push("wbs_export_jira_monday_ms_project.csv is empty.".to_string());
            }
        }
    }

    // Validate JSON structure
    if wbs_json.is_file() {
        match fs::read_to_string(&wbs_json) {
            Ok(content) => match serde_json::from_str::<Value>(&content) {
                Ok(json_data) => {
                    for req_key in &["metadata", "wbs_tree", "traceability_matrix"] {
                        if json_data.get(req_key).is_none() {
                            errors.push(format!("wbs_export.json missing required top-level key: '{}'", req_key));
                        }
                    }
                    if let Some(wbs_tree) = json_data.get("wbs_tree") {
                        if !wbs_tree.is_object() {
                            errors.push("wbs_export.json 'wbs_tree' must be a JSON object.".to_string());
                        } else {
                            for tree_key in &["wbs_code", "name", "level", "children"] {
                                if wbs_tree.get(tree_key).is_none() {
                                    errors.push(format!("wbs_export.json 'wbs_tree' missing required key: '{}'", tree_key));
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!("wbs_export.json invalid JSON: {}", e));
                }
            },
            Err(e) => {
                errors.push(format!("Failed to read {}: {}", wbs_json.display(), e));
            }
        }
    }

    // Validate Zero Unicode em dashes
    for target_path in [&target_md, &wbs_csv, &wbs_json] {
        if target_path.is_file() {
            if let Ok(content) = fs::read_to_string(target_path) {
                if content.contains('\u{2014}') {
                    let rel = target_path.strip_prefix(repo_root).unwrap_or(target_path);
                    errors.push(format!(
                        "Zero em dash violation in {}: contains forbidden Unicode em dash (\u{2014}).",
                        rel.display()
                    ));
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(WbsCheckOutcome::Verified)
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;

    struct TempDir {
        path: std::path::PathBuf,
    }

    impl TempDir {
        fn new(name: &str) -> Self {
            static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let count = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!("verify_wbs_{}_{}_{}", name, nanos, count));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn test_wbs_pending_when_missing() {
        let tmp = TempDir::new("missing");
        let res = check_wbs_suite_integrity(&tmp.path, true, false);
        assert_eq!(res, Ok(WbsCheckOutcome::PendingOrNotPresent));
    }

    #[test]
    fn test_wbs_strict_fails_when_missing() {
        let tmp = TempDir::new("strict");
        let res = check_wbs_suite_integrity(&tmp.path, true, true);
        assert!(res.is_err());
    }

    #[test]
    fn test_wbs_valid_suite() {
        let tmp = TempDir::new("valid");
        let mgmt = tmp.path.join("docs").join("management");
        fs::create_dir_all(&mgmt).unwrap();

        let md_file = mgmt.join("WBS_DELIVERABLES_SUITE.md");
        let mut f_md = File::create(md_file).unwrap();
        writeln!(f_md, "| Attribute | Specification Detail |\n|---|---|\n\n| WBS Code | Deliverable / Task Name | Spec Reference | MBD Package / Model File | Verification Artifact | Target Issue Key | Owner / Role |").unwrap();

        let csv_file = mgmt.join("wbs_export_jira_monday_ms_project.csv");
        let mut f_csv = File::create(csv_file).unwrap();
        writeln!(f_csv, "{}", REQUIRED_CSV_HEADERS.join(",")).unwrap();

        let json_file = mgmt.join("wbs_export.json");
        let mut f_json = File::create(json_file).unwrap();
        writeln!(f_json, r#"{{"metadata": {{}}, "wbs_tree": {{"wbs_code": "1", "name": "Root", "level": 1, "children": []}}, "traceability_matrix": []}}"#).unwrap();

        let res = check_wbs_suite_integrity(&tmp.path, false, false);
        assert_eq!(res, Ok(WbsCheckOutcome::Verified));
    }
}
