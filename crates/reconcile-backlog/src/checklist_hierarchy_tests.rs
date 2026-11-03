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
