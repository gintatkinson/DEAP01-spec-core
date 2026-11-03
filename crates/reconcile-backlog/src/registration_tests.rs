//! Unit tests for bottom-up registration, publish integrity gate, and zero-clippy/unwrap source invariants.
//! Realises: [REQ-RECONCILE-BACKLOG-CORE]

use super::lib_tests::{tempdir, MockTracker};
use super::*;
use crate::checklist::DiscoveredSpec;
use crate::tracker::{IssueIndex, OfflineTracker};
use std::fs;
use std::path::Path;

#[test]
fn test_resolve_canonical_spec_tier() {
    assert_eq!(
        resolve_canonical_spec_tier("docs/use-cases/uc-001.md", None, "UC-001"),
        (0, "use_case", "type:use-case")
    );
    assert_eq!(
        resolve_canonical_spec_tier("docs/user-stories/us-001.md", None, "US-001"),
        (1, "user_story", "type:user-story")
    );
    assert_eq!(
        resolve_canonical_spec_tier("docs/features/feat-001.md", None, "FEAT-001"),
        (2, "feature", "type:feature")
    );
    assert_eq!(
        resolve_canonical_spec_tier("docs/epics/epic-001.md", None, "EPIC-001"),
        (3, "epic", "type:epic")
    );
}

fn setup_registration_fixture(
    root: &Path,
) -> Result<Vec<DiscoveredSpec>, Box<dyn std::error::Error>> {
    let epics_dir = root.join("docs/epics");
    let feats_dir = root.join("docs/features");
    let stories_dir = root.join("docs/user-stories");
    let ucs_dir = root.join("docs/use-cases");
    for d in [&epics_dir, &feats_dir, &stories_dir, &ucs_dir] {
        fs::create_dir_all(d)?;
    }

    let uc_path = ucs_dir.join("uc-001.md");
    fs::write(&uc_path, "# UC-001: Auth Flow\n\n| Field | Value |\n|---|---|\n| **Type** | `Use Case` |\n| **Parent Epic** | EPIC-001 |\n| **GitHub Issue** | Pending |\n\n## Flow\n1. Actor logs in.\n")?;
    let us_path = stories_dir.join("us-001.md");
    fs::write(&us_path, "# US-001: Login Button\n\n| Field | Value |\n|---|---|\n| **Type** | `User Story` |\n| **Parent Epic** | EPIC-001 |\n| **Parent Feature** | FEAT-001 |\n| **GitHub Issue** | Pending |\n\n## Story\nAs a user...\n")?;
    let feat_path = feats_dir.join("feat-001.md");
    fs::write(&feat_path, "# FEAT-001: Auth Module\n\n| Field | Value |\n|---|---|\n| **Type** | `Feature` |\n| **Parent Epic** | EPIC-001 |\n| **GitHub Issue** | Pending |\n\n## Overview\nAuth module.\n")?;
    let epic_path = epics_dir.join("epic-001.md");
    fs::write(&epic_path, "# EPIC-001: Security\n\n| Field | Value |\n|---|---|\n| **Type** | `Epic` |\n| **GitHub Issue** | Pending |\n\n## Overview\nSecurity epic.\n")?;

    Ok(vec![
        DiscoveredSpec {
            path: epic_path,
            rel_path: "docs/epics/epic-001.md".to_string(),
            spec_type: "Epic".to_string(),
            document_id: Some("EPIC-001".to_string()),
            title: "EPIC-001: Security".to_string(),
            issue_number: None,
            is_resolved: false,
            parent_epic: None,
            parent_feature: None,
        },
        DiscoveredSpec {
            path: feat_path,
            rel_path: "docs/features/feat-001.md".to_string(),
            spec_type: "Feature".to_string(),
            document_id: Some("FEAT-001".to_string()),
            title: "FEAT-001: Auth Module".to_string(),
            issue_number: None,
            is_resolved: false,
            parent_epic: Some("EPIC-001".to_string()),
            parent_feature: None,
        },
        DiscoveredSpec {
            path: us_path,
            rel_path: "docs/user-stories/us-001.md".to_string(),
            spec_type: "User Story".to_string(),
            document_id: Some("US-001".to_string()),
            title: "US-001: Login Button".to_string(),
            issue_number: None,
            is_resolved: false,
            parent_epic: Some("EPIC-001".to_string()),
            parent_feature: Some("FEAT-001".to_string()),
        },
        DiscoveredSpec {
            path: uc_path,
            rel_path: "docs/use-cases/uc-001.md".to_string(),
            spec_type: "Use Case".to_string(),
            document_id: Some("UC-001".to_string()),
            title: "UC-001: Auth Flow".to_string(),
            issue_number: None,
            is_resolved: false,
            parent_epic: Some("EPIC-001".to_string()),
            parent_feature: None,
        },
    ])
}

#[test]
fn test_register_unallocated_specs_bottom_up_and_backpatch() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let mut specs = setup_registration_fixture(dir.path())?;
    let tracker = MockTracker::new(vec![]);
    let mut index = IssueIndex::default();

    let created = register_unallocated_specs_bottom_up(
        dir.path(),
        &mut specs,
        &mut index,
        &tracker,
        ("org/repo", "main", None),
    )?;

    assert_eq!(created, 4);
    assert_eq!(specs.get(3).and_then(|s| s.issue_number), Some(1));
    assert_eq!(specs.get(2).and_then(|s| s.issue_number), Some(2));
    assert_eq!(specs.get(1).and_then(|s| s.issue_number), Some(3));
    assert_eq!(specs.first().and_then(|s| s.issue_number), Some(4));

    let created_issues = tracker.issues.borrow();
    let i0 = created_issues.first().ok_or("missing i0")?;
    let i1 = created_issues.get(1).ok_or("missing i1")?;
    let i2 = created_issues.get(2).ok_or("missing i2")?;
    let i3 = created_issues.get(3).ok_or("missing i3")?;
    assert_eq!(i0.title, "UC-001: Auth Flow");
    assert_eq!(i1.title, "US-001: Login Button");
    assert_eq!(i2.title, "FEAT-001: Auth Module");
    assert_eq!(i3.title, "EPIC-001: Security");
    assert!(i2.body.contains("https://github.com/org/repo/issues/2"));
    assert!(i3.body.contains("https://github.com/org/repo/issues/3"));
    assert!(i3.body.contains("https://github.com/org/repo/issues/1"));
    Ok(())
}

#[test]
fn test_validate_checklist_publish_integrity_catches_unlinked_child(
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let epic_path = dir.path().join("epic-001.md");
    fs::write(
        &epic_path,
        "# EPIC-001: Core\n\n## Features\n- [ ] FEAT-001: Unlinked Feature\n",
    )?;

    let specs = vec![DiscoveredSpec {
        path: epic_path,
        rel_path: "docs/epics/epic-001.md".to_string(),
        spec_type: "Epic".to_string(),
        document_id: Some("EPIC-001".to_string()),
        title: "EPIC-001: Core".to_string(),
        issue_number: Some(1),
        is_resolved: false,
        parent_epic: None,
        parent_feature: None,
    }];

    assert!(matches!(
        validate_checklist_publish_integrity(&specs),
        Err(ReconcileError::PublishGateViolation(_))
    ));
    Ok(())
}

#[test]
fn test_run_reconciliation_offline_mode() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let epics_dir = dir.path().join("docs/epics");
    fs::create_dir_all(&epics_dir)?;
    fs::write(
        epics_dir.join("epic-001.md"),
        "# EPIC-001: Offline\n\n## Features\n- [ ] (To be created)\n",
    )?;

    let tracker = OfflineTracker::new();
    let options = ReconcileOptions {
        workspace_root: dir.path().to_path_buf(),
        offline: true,
        dry_run: false,
        ..Default::default()
    };

    let summary = run_reconciliation(&tracker, &options)?;
    assert_eq!(summary.total_specs_scanned, 1);
    assert_eq!(summary.placeholders_purged, 1);
    Ok(())
}

fn production_source_files() -> [(&'static str, &'static str); 15] {
    [
        ("lib.rs", include_str!("lib.rs")),
        ("main.rs", include_str!("main.rs")),
        ("checklist.rs", include_str!("checklist.rs")),
        ("checklist_hierarchy.rs", include_str!("checklist_hierarchy.rs")),
        ("checklist_placeholders.rs", include_str!("checklist_placeholders.rs")),
        ("create_issue.rs", include_str!("create_issue.rs")),
        ("error.rs", include_str!("error.rs")),
        ("frontmatter.rs", include_str!("frontmatter.rs")),
        ("frontmatter_update.rs", include_str!("frontmatter_update.rs")),
        ("git_env.rs", include_str!("git_env.rs")),
        ("projection.rs", include_str!("projection.rs")),
        ("registration.rs", include_str!("registration.rs")),
        ("tracker.rs", include_str!("tracker.rs")),
        ("tracker_github.rs", include_str!("tracker_github.rs")),
        ("tracker_normalize.rs", include_str!("tracker_normalize.rs")),
    ]
}

#[test]
fn test_issue_506_zero_clippy_allow_overrides_and_production_unwraps() {
    let allow_pat = concat!("allow(", "clippy::");
    let unwrap_pat = concat!(".", "unwrap(");
    for (name, src) in production_source_files() {
        assert!(
            !src.contains(allow_pat),
            "{} must not contain clippy allow overrides",
            name
        );
        let prod_src = src.split("#[cfg(test)]").next().unwrap_or(src);
        for (line_idx, line) in prod_src.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            assert!(
                !trimmed.contains(unwrap_pat),
                "{}:{} contains production unwrap",
                name,
                line_idx + 1
            );
        }
    }
}

#[test]
fn test_defect_08_zero_blanket_clippy_allow_suppressions() {
    let allow_pat = concat!("allow(", "clippy::");
    for (name, src) in production_source_files() {
        assert!(!src.contains(allow_pat), "{} has clippy allow", name);
    }
}

#[test]
fn test_wave_1c_module_sloc_fn_span_and_ast_compliance() {
    let char_indices_pat = concat!(".", "char_indices()");
    for (name, src) in production_source_files() {
        let line_count = src.lines().count();
        assert!(
            line_count <= 380,
            "{} has {} lines (must be <= 380)",
            name,
            line_count
        );
        assert!(
            !src.contains(char_indices_pat),
            "{} must not use char_indices",
            name
        );
        let parsed = deap_core::rust_cst::ParsedRustFile::parse_rust_source(src);
        let items = deap_core::rust_cst::list_rust_items(&parsed);
        for item in &items {
            if item.kind == deap_core::rust_cst::RustItemKind::Fn {
                let span = item.end_line.saturating_sub(item.start_line) + 1;
                assert!(
                    span <= 80,
                    "{}:{} fn '{}' spans {} lines (must be <= 80)",
                    name,
                    item.start_line,
                    item.name,
                    span
                );
            }
        }
    }
}
