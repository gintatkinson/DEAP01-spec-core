//! Unit tests for bottom-up hierarchy synthesis (`synthesize_and_reconcile_hierarchy`).
//! Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST]

use crate::checklist::*;
use crate::lib_tests::tempdir;
use std::fs;
use std::path::Path;

fn make_spec(
    path: std::path::PathBuf,
    rel_path: &str,
    spec_type: &str,
    doc_id: &str,
    title: &str,
) -> DiscoveredSpec {
    DiscoveredSpec {
        path,
        rel_path: rel_path.to_string(),
        spec_type: spec_type.to_string(),
        document_id: Some(doc_id.to_string()),
        title: title.to_string(),
        issue_number: None,
        is_resolved: false,
        parent_epic: None,
        parent_feature: None,
    }
}

fn build_hierarchy_fixture(
    root: &Path,
) -> Result<Vec<DiscoveredSpec>, Box<dyn std::error::Error>> {
    let epics_dir = root.join("docs/epics");
    let features_dir = root.join("docs/features");
    let stories_dir = root.join("docs/user-stories");
    let ucs_dir = root.join("docs/use-cases");
    for d in [&epics_dir, &features_dir, &stories_dir, &ucs_dir] {
        fs::create_dir_all(d)?;
    }

    let epic_path = epics_dir.join("epic-001.md");
    fs::write(&epic_path, "# EPIC-001: Core System\n\n## Overview\nCore system epic.\n\n## Features\n- [ ] (To be created)\n")?;
    let feat_path = features_dir.join("feat-001.md");
    fs::write(&feat_path, "# FEAT-001: Auth Module\n\n## Overview\nAuth feature.\n")?;
    let story_path = stories_dir.join("us-001.md");
    fs::write(&story_path, "# US-001: User Login\n\n## Story\nLogin story.\n")?;
    let uc_path = ucs_dir.join("uc-001.md");
    fs::write(&uc_path, "# UC-001: Authenticate Actor\n\n## Scenario\nMain scenario.\n")?;

    let mut epic = make_spec(epic_path, "docs/epics/epic-001.md", "Epic", "EPIC-001", "EPIC-001: Core System");
    epic.issue_number = Some(1);
    let mut feat = make_spec(feat_path, "docs/features/feat-001.md", "Feature", "FEAT-001", "FEAT-001: Auth Module");
    feat.issue_number = Some(2);
    feat.parent_epic = Some("[EPIC-001: Core System](docs/epics/epic-001.md)".to_string());
    let mut story = make_spec(story_path, "docs/user-stories/us-001.md", "User Story", "US-001", "US-001: User Login");
    story.issue_number = Some(3);
    story.is_resolved = true;
    story.parent_epic = Some("EPIC-001".to_string());
    story.parent_feature = Some("#2".to_string());
    let mut uc = make_spec(uc_path, "docs/use-cases/uc-001.md", "Use Case", "UC-001", "UC-001: Authenticate Actor");
    uc.issue_number = Some(4);
    uc.parent_epic = Some("#1".to_string());

    Ok(vec![epic, feat, story, uc])
}

#[test]
fn test_synthesize_and_reconcile_hierarchy_injects_and_formats() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let specs = build_hierarchy_fixture(dir.path())?;

    let changes = synthesize_and_reconcile_hierarchy(&specs, Some("org/repo"), "main", false)?;
    assert!(changes > 0);

    let epic_spec = specs.first().ok_or("missing epic spec")?;
    let epic_content = fs::read_to_string(&epic_spec.path)?;
    assert!(!epic_content.contains("(To be created)"));
    assert!(epic_content.contains("- [ ] [FEAT-001: Auth Module](https://github.com/org/repo/issues/2) -- [docs/features/feat-001.md](https://github.com/org/repo/blob/main/docs/features/feat-001.md)"));
    assert!(epic_content.contains("## Use Cases"));
    assert!(epic_content.contains("- [ ] [UC-001: Authenticate Actor](https://github.com/org/repo/issues/4) -- [docs/use-cases/uc-001.md](https://github.com/org/repo/blob/main/docs/use-cases/uc-001.md)"));

    let feat_spec = specs.get(1).ok_or("missing feat spec")?;
    let feat_content = fs::read_to_string(&feat_spec.path)?;
    assert!(feat_content.contains("## User Stories"));
    assert!(feat_content.contains("- [x] [US-001: User Login](https://github.com/org/repo/issues/3) -- [docs/user-stories/us-001.md](https://github.com/org/repo/blob/main/docs/user-stories/us-001.md)"));
    Ok(())
}

#[test]
fn test_synthesize_and_reconcile_hierarchy_updates_existing_items_in_place() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let epics_dir = dir.path().join("docs/epics");
    let features_dir = dir.path().join("docs/features");
    fs::create_dir_all(&epics_dir)?;
    fs::create_dir_all(&features_dir)?;

    let epic_path = epics_dir.join("epic-001.md");
    fs::write(
        &epic_path,
        "# EPIC-001: Core System\n\n## Features\n- [ ] FEAT-001: Auth Module\n\n## Notes\nKeep me.\n",
    )?;

    let feat_path = features_dir.join("feat-001.md");
    fs::write(&feat_path, "# FEAT-001: Auth Module\n")?;

    let mut epic = make_spec(epic_path.clone(), "docs/epics/epic-001.md", "Epic", "EPIC-001", "EPIC-001: Core System");
    epic.issue_number = Some(10);
    let mut feat = make_spec(feat_path, "docs/features/feat-001.md", "Feature", "FEAT-001", "FEAT-001: Auth Module");
    feat.issue_number = Some(20);
    feat.is_resolved = true;
    feat.parent_epic = Some("EPIC-001".to_string());

    let changes =
        synthesize_and_reconcile_hierarchy(&[epic, feat], Some("org/repo"), "develop", false)?;
    assert_eq!(changes, 1);

    let epic_content = fs::read_to_string(&epic_path)?;
    assert!(epic_content.contains("- [x] [FEAT-001: Auth Module](https://github.com/org/repo/issues/20) -- [docs/features/feat-001.md](https://github.com/org/repo/blob/develop/docs/features/feat-001.md)"));
    assert!(epic_content.contains("## Notes\nKeep me."));
    Ok(())
}

#[test]
fn test_reconcile_or_inject_section_ignores_tilde_fenced_headings() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let epics_dir = dir.path().join("docs/epics");
    let features_dir = dir.path().join("docs/features");
    fs::create_dir_all(&epics_dir)?;
    fs::create_dir_all(&features_dir)?;

    let epic_path = epics_dir.join("epic-001.md");
    fs::write(
        &epic_path,
        "# EPIC-001: Core System\n\n~~~markdown\n## Features\n- [ ] Fake Feature Inside Tilde Block\n~~~\n\n## Overview\nReal content.\n",
    )?;

    let feat_path = features_dir.join("feat-001.md");
    fs::write(&feat_path, "# FEAT-001: Real Feature\n")?;

    let mut epic = make_spec(epic_path.clone(), "docs/epics/epic-001.md", "Epic", "EPIC-001", "EPIC-001: Core System");
    epic.issue_number = Some(1);
    let mut feat = make_spec(feat_path, "docs/features/feat-001.md", "Feature", "FEAT-001", "FEAT-001: Real Feature");
    feat.issue_number = Some(2);
    feat.parent_epic = Some("EPIC-001".to_string());

    let changes =
        synthesize_and_reconcile_hierarchy(&[epic, feat], Some("org/repo"), "main", false)?;
    assert_eq!(changes, 1);

    let epic_content = fs::read_to_string(&epic_path)?;
    assert!(epic_content.contains("~~~markdown\n## Features\n- [ ] Fake Feature Inside Tilde Block\n~~~"));
    assert!(epic_content.ends_with("## Features\n\n- [ ] [FEAT-001: Real Feature](https://github.com/org/repo/issues/2) -- [docs/features/feat-001.md](https://github.com/org/repo/blob/main/docs/features/feat-001.md)\n"));
    Ok(())
}

#[test]
fn test_compile_sysml_sections_reconciled_in_place_without_duplicate_h2(
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let epics_dir = dir.path().join("docs/epics");
    let feats_dir = dir.path().join("docs/features");
    let ucs_dir = dir.path().join("docs/use-cases");
    let stories_dir = dir.path().join("docs/user-stories");
    for d in [&epics_dir, &feats_dir, &ucs_dir, &stories_dir] {
        fs::create_dir_all(d)?;
    }

    let epic_path = epics_dir.join("EPIC-01.md");
    fs::write(
        &epic_path,
        "# EPIC-01: System\n\n## 2. Requirements & Checklist\n- [ ] [FEAT-01: Submodule](../features/FEAT-01.md)\n\n### Associated Use Cases & User Stories\n\n#### Associated Use Cases\n- [ ] [UC-01: Execute Flow](../use-cases/UC-01.md)\n\n#### Associated User Stories\n- [ ] [US-01: User Action](../user-stories/US-01.md)\n",
    )?;
    let feat_path = feats_dir.join("FEAT-01.md");
    fs::write(
        &feat_path,
        "# FEAT-01: Submodule\n\n### Associated Use Cases & User Stories\n\n#### Associated User Stories\n- [ ] [US-01: User Action](../user-stories/US-01.md)\n",
    )?;
    let uc_path = ucs_dir.join("UC-01.md");
    fs::write(&uc_path, "# UC-01: Execute Flow\n")?;
    let us_path = stories_dir.join("US-01.md");
    fs::write(&us_path, "# US-01: User Action\n")?;

    let mut epic = make_spec(epic_path.clone(), "docs/epics/EPIC-01.md", "Epic", "EPIC-01", "EPIC-01: System");
    epic.issue_number = Some(100);
    let mut feat = make_spec(feat_path.clone(), "docs/features/FEAT-01.md", "Feature", "FEAT-01", "FEAT-01: Submodule");
    feat.issue_number = Some(101);
    feat.is_resolved = true;
    feat.parent_epic = Some("EPIC-01".to_string());
    let mut uc = make_spec(uc_path, "docs/use-cases/UC-01.md", "Use Case", "UC-01", "UC-01: Execute Flow");
    uc.issue_number = Some(102);
    uc.parent_epic = Some("EPIC-01".to_string());
    let mut us = make_spec(us_path, "docs/user-stories/US-01.md", "User Story", "US-01", "US-01: User Action");
    us.issue_number = Some(103);
    us.is_resolved = true;
    us.parent_epic = Some("EPIC-01".to_string());
    us.parent_feature = Some("FEAT-01".to_string());

    let changes = synthesize_and_reconcile_hierarchy(&[epic, feat, uc, us], None, "main", false)?;
    assert!(changes > 0);

    let epic_after = fs::read_to_string(&epic_path)?;
    assert!(epic_after.contains("- [x] #101 - [FEAT-01: Submodule](../features/FEAT-01.md)"));
    assert!(epic_after.contains("- [ ] #102 - [UC-01: Execute Flow](../use-cases/UC-01.md)"));
    assert!(epic_after.contains("- [x] #103 - [US-01: User Action](../user-stories/US-01.md)"));
    assert!(!epic_after.contains("## Features"));
    assert!(!epic_after.contains("## Use Cases"));
    assert!(!epic_after.contains("## User Stories"));

    let feat_after = fs::read_to_string(&feat_path)?;
    assert!(feat_after.contains("- [x] #103 - [US-01: User Action](../user-stories/US-01.md)"));
    assert!(!feat_after.contains("## User Stories"));
    Ok(())
}

#[test]
fn test_upstream_local_purity_preserves_bare_issue_ids_and_projects_remote_blob_urls(
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::lib_tests::MockTracker;
    use crate::tracker::Issue;
    use crate::{run_reconciliation, ReconcileOptions};

    let dir = tempdir()?;
    let upstream_root = dir.path().join(".pipeline/upstream");
    let epics_dir = upstream_root.join("docs/epics");
    let feats_dir = upstream_root.join("docs/features");
    fs::create_dir_all(&epics_dir)?;
    fs::create_dir_all(&feats_dir)?;

    let epic_path = epics_dir.join("EPIC-01.md");
    fs::write(
        &epic_path,
        "| Attribute | Specification Detail |\n| :--- | :--- |\n| **Issue ID** | #201 |\n| **Title** | Epic 01: Core Compiler |\n| **Type** | epic |\n\n# Epic 01: Core Compiler\n\n## 2. Requirements & Checklist\n- [ ] [Feature 01: AST Parser](../features/FEAT-01.md)\n",
    )?;
    let feat_path = feats_dir.join("FEAT-01.md");
    fs::write(
        &feat_path,
        "| Attribute | Specification Detail |\n| :--- | :--- |\n| **Issue ID** | #202 |\n| **Title** | Feature 01: AST Parser |\n| **Type** | feature |\n| **Parent Epic** | [EPIC-01](../epics/EPIC-01.md) |\n\n# Feature 01: AST Parser\n\n## Overview\nParser spec.\n",
    )?;

    let tracker = MockTracker::new(vec![
        Issue { number: 201, title: "Epic 01: Core Compiler".to_string(), state: "OPEN".to_string(), labels: vec!["type:epic".to_string()], body: String::new() },
        Issue { number: 202, title: "Feature 01: AST Parser".to_string(), state: "CLOSED".to_string(), labels: vec!["type:feature".to_string()], body: String::new() },
    ]);

    let options = ReconcileOptions {
        workspace_root: upstream_root,
        offline: false,
        dry_run: false,
        provider: "github".to_string(),
        sync_bodies: true,
        repo: Some("org/repo".to_string()),
        default_branch: Some("main".to_string()),
    };

    run_reconciliation(&tracker, &options)?;

    let epic_local = fs::read_to_string(&epic_path)?;
    let feat_local = fs::read_to_string(&feat_path)?;
    assert!(!epic_local.contains("https://github.com/"), "local epic has github URL: {}", epic_local);
    assert!(!feat_local.contains("https://github.com/"), "local feat has github URL: {}", feat_local);
    assert!(epic_local.contains("| **Issue ID** | #201 |"));
    assert!(epic_local.contains("- [x] #202 - [Feature 01: AST Parser](../features/FEAT-01.md)"));

    let issues = tracker.issues.borrow();
    let remote_epic = issues.iter().find(|i| i.number == 201).ok_or("missing 201")?;
    assert!(remote_epic.body.contains("https://github.com/org/repo/blob/main/.pipeline/upstream/docs/features/FEAT-01.md"));
    Ok(())
}

#[test]
fn test_reconcile_syncs_stale_remote_issue_title() -> Result<(), Box<dyn std::error::Error>> {
    use crate::lib_tests::MockTracker;
    use crate::tracker::Issue;
    use crate::{run_reconciliation, ReconcileOptions};

    let dir = tempdir()?;
    let feats_dir = dir.path().join("docs/features");
    fs::create_dir_all(&feats_dir)?;

    let feat_path = feats_dir.join("FEAT-03-ClippyAttributeCstParser.md");
    fs::write(
        &feat_path,
        "| Attribute | Specification Detail |\n| :--- | :--- |\n| **Issue ID** | #529 |\n| **Title** | Feature 03: [BP 01 Rust Source Ast Gate Blueprint] ClippyAttributeCstParser |\n| **Type** | feature |\n\n# Feature 03: [BP 01 Rust Source Ast Gate Blueprint] ClippyAttributeCstParser\n\n## Overview\nCST parser.\n",
    )?;

    let tracker = MockTracker::new(vec![Issue {
        number: 529,
        title: "Feature 03: [BP 01 Rust Source Ast Gate Blueprint] ClippyAttributeAstScanner".to_string(),
        state: "OPEN".to_string(),
        labels: vec!["type:feature".to_string()],
        body: "# Feature 03: [BP 01 Rust Source Ast Gate Blueprint] ClippyAttributeCstParser\n\n## Overview\nCST parser.".to_string(),
    }]);

    let options = ReconcileOptions {
        workspace_root: dir.path().to_path_buf(),
        offline: false,
        dry_run: false,
        provider: "github".to_string(),
        sync_bodies: true,
        repo: Some("org/repo".to_string()),
        default_branch: Some("main".to_string()),
    };

    let summary = run_reconciliation(&tracker, &options)?;
    assert!(summary.remote_issues_synced >= 1);

    let issues = tracker.issues.borrow();
    let i529 = issues.iter().find(|i| i.number == 529).ok_or("missing 529")?;
    assert_eq!(
        i529.title,
        "Feature 03: [BP 01 Rust Source Ast Gate Blueprint] ClippyAttributeCstParser"
    );
    Ok(())
}
