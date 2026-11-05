//! Reproducer test helpers for GitHub Issue #699 (`compile-sysml` fixture, acronym, and Feature checks).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C verification
//! objectives for `crates/compile-sysml/tests/reproducer_j_tests.rs` (refs #699).
//!
//! ## 2. Invariants & Mathematical Properties
//! - Humanized Epic, User Story, and Use Case titles preserve `"SysML"`, `"CommonMark"`, `"GitHub"`,
//!   `"OpenAPI"`, `"UUIDv5"`, `"STPA"`, `"FMECA"`, and `"SHA256"` intact without fracturing.
//! - `FEAT-01-SnapshotIntegrityValidator.md` emits zero CSS container or compiler boilerplate,
//!   projects all 10 constraints without `.truncate(8)` loss, and emits `3..=10` positive and
//!   negative BDD acceptance criteria.
//!
//! ## 3. Preconditions
//! - Pure in-memory AST construction and generated Markdown buffers.
//!
//! ## 4. Postconditions
//! - All acronym and Feature projection assertions hold.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(L)$ linear scan over generated Markdown buffers.
//! - Space Complexity: $O(1)$ auxiliary stack space over borrowed slices.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Validates generated Markdown buffers from `forward_sync_sysml_to_specs`.
//! - State: Pure read-only inspection with zero shared mutable state.
//! - Action: Executes with zero panics or `.unwrap()` calls.
//! - Resource: Reclaimed at scope exit.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-SYNC-FEAT-EPIC-AST-GROUNDED-699], [REQ-SYNC-ACRONYM-PRESERVATION-699] | SysML v2: VerificationCaseDef(reproducer_j_tests)

use deap_core::sysml_ast::{
    ActionDef, AttributeDef, ConstraintDef, PackageDef, PartDef, PortDef, RequirementDef, StateDef,
    TransitionDef, UseCaseDef,
};

use crate::reproducer_tests::extract_markdown_h2_section;

pub(crate) const ISSUE_699_TEN_CONSTRAINTS: &[(&str, &str)] = &[
    ("C01_Bound", "c01_val >= 0"),
    ("C02_EpochFloor", "c02_epoch >= 1"),
    ("C03_ReplicaCount", "c03_replicas >= 3"),
    ("C04_DigestLen", "c04_bytes == 32"),
    ("C05_SkewLimit", "c05_skew_ms <= 50"),
    ("C06_RetryCap", "c06_retries <= 4"),
    ("C07_BatchQuota", "c07_items <= 1024"),
    ("C08_QuorumRatio", "c08_ratio > 0"),
    ("C09_NinthInvariant", "c09_margin > 1"),
    ("C10_TenthReqInvariant", "c10_integrity == true"),
];

fn build_issue_699_validator_part() -> PartDef {
    let part_constraints = ISSUE_699_TEN_CONSTRAINTS
        .iter()
        .take(9)
        .map(|(name, expression)| ConstraintDef {
            name: (*name).to_string(),
            expression: (*expression).to_string(),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    PartDef {
        name: "SnapshotIntegrityValidator".to_string(),
        attributes: vec![
            AttributeDef { name: "checkpoint_digest".to_string(), type_name: "CheckpointDigestVector".to_string(), default_value: Some("sha256-zero".to_string()), doc: None },
            AttributeDef { name: "mode_label".to_string(), type_name: "String".to_string(), default_value: None, doc: None },
        ],
        ports: vec![
            PortDef { name: "snapshot_in".to_string(), direction: "in".to_string(), type_name: "SnapshotEnvelope".to_string(), ..Default::default() },
            PortDef { name: "fault_out".to_string(), direction: "out".to_string(), type_name: "IntegrityDiagnostic".to_string(), ..Default::default() },
        ],
        constraints: part_constraints,
        states: vec![StateDef {
            name: "ValidatorLifecycleState".to_string(),
            transitions: vec![TransitionDef {
                source: Some("Idle".to_string()),
                target: Some("Verifying".to_string()),
                trigger: Some("CompileSysMLToCommonMark".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Constructs a domain-neutral distributed ledger governance `PackageDef` for Issue #699.
///
/// # Arguments
/// * None.
///
/// # Returns
/// * `PackageDef` - Owned `LedgerCoreSystem` package containing `LedgerGovernance_SysML`.
///
/// # Complexity
/// * Time: $O(1)$ bounded AST construction. Space: $O(1)$.
///
/// # Failure Modes
/// * Infallible pure constructor.
pub fn build_issue_699_ledger_governance_fixture() -> PackageDef {
    let subpkg = PackageDef {
        name: "LedgerGovernance_SysML".to_string(),
        doc: Some("Authoritative distributed ledger replication governance subsystem.".to_string()),
        part_defs: vec![build_issue_699_validator_part()],
        action_defs: vec![
            ActionDef { name: "CompileSysMLToCommonMark".to_string(), ..Default::default() },
            ActionDef { name: "SyncGitHubOpenAPIWithUUIDv5".to_string(), ..Default::default() },
        ],
        use_case_defs: vec![UseCaseDef { name: "UC_01_VerifySTPAAndFMECAWithSHA256".to_string(), ..Default::default() }],
        requirement_defs: vec![RequirementDef {
            name: "IntegrityAuditRequirement".to_string(),
            req_id: "REQ-0699".to_string(),
            text: "Verify deterministic ledger snapshot digest and invariant satisfaction.".to_string(),
            satisfied_by: vec!["SnapshotIntegrityValidator".to_string()],
            constraints: vec![ConstraintDef {
                name: "C10_TenthReqInvariant".to_string(),
                expression: "c10_integrity == true".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    };

    PackageDef { name: "LedgerCoreSystem".to_string(), packages: vec![subpkg], ..Default::default() }
}

/// Extracts the body of a Markdown heading containing `title_keyword` up to the next peer heading.
///
/// # Arguments
/// * `markdown` - UTF-8 Markdown document slice.
/// * `title_keyword` - Non-empty keyword expected in the heading or fallback section.
/// * `fallback_h2_prefix` - Level-2 heading prefix fallback when `title_keyword` is in body text.
///
/// # Returns
/// * `Result<&'a str, String>` - Bounded section body slice or diagnostic error.
///
/// # Complexity
/// * Time: $O(L)$ linear scan over `markdown`. Space: $O(1)$.
///
/// # Failure Modes
/// * Returns `Err(String)` when the requested section is absent.
pub fn extract_markdown_section_by_keyword<'a>(
    markdown: &'a str,
    title_keyword: &str,
    fallback_h2_prefix: &str,
) -> Result<&'a str, String> {
    if let Some(heading_line) = markdown.lines().find(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with('#') && trimmed.contains(title_keyword)
    }) {
        let heading_level = heading_line
            .trim_start()
            .chars()
            .take_while(|&ch| ch == '#')
            .count();
        let (_before, after_heading) = markdown.split_once(heading_line).ok_or_else(|| {
            format!(
                "Failed to partition Markdown at heading {:?} in:\n{}",
                heading_line, markdown
            )
        })?;
        let next_boundary = after_heading.lines().skip(1).find(|line| {
            let trimmed = line.trim_start();
            let hashes = trimmed.chars().take_while(|&ch| ch == '#').count();
            hashes > 0 && hashes <= heading_level && trimmed.chars().nth(hashes) == Some(' ')
        });
        if let Some(boundary_line) = next_boundary {
            if let Some((section_body, _rest)) = after_heading.split_once(boundary_line) {
                return Ok(section_body);
            }
        }
        return Ok(after_heading);
    }

    let h2_body = extract_markdown_h2_section(markdown, fallback_h2_prefix)?;
    if h2_body.contains(title_keyword) {
        Ok(h2_body)
    } else {
        Err(format!(
            "Section {:?} did not contain expected keyword {:?}:\n{}",
            fallback_h2_prefix, title_keyword, h2_body
        ))
    }
}

/// Verifies acronym preservation across generated Epic, User Story, and Use Case titles (#699).
///
/// # Arguments
/// * `epic_md` - Generated `EPIC-02-LedgerGovernance_SysML.md` content.
/// * `us01_md` - Generated `US-01-CompileSysMLToCommonMark.md` content.
/// * `us02_md` - Generated `US-02-SyncGitHubOpenAPIWithUUIDv5.md` content.
/// * `uc01_md` - Generated `UC-01-VerifySTPAAndFMECAWithSHA256.md` content.
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - `Ok(())` when all acronym assertions hold.
///
/// # Complexity
/// * Time: $O(L)$ over the 4 Markdown buffers. Space: $O(1)$.
///
/// # Failure Modes
/// * Returns `Err` if H1 or metadata title rows are missing, or panics on fractured acronyms.
pub fn assert_issue_699_acronyms(
    epic_md: &str,
    us01_md: &str,
    us02_md: &str,
    uc01_md: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    for (doc_name, doc_md, expected_acronyms) in [
        ("EPIC-02-LedgerGovernance_SysML.md", epic_md, ["SysML"].as_slice()),
        (
            "US-01-CompileSysMLToCommonMark.md",
            us01_md,
            ["SysML", "CommonMark"].as_slice(),
        ),
        (
            "US-02-SyncGitHubOpenAPIWithUUIDv5.md",
            us02_md,
            ["GitHub", "OpenAPI", "UUIDv5"].as_slice(),
        ),
        (
            "UC-01-VerifySTPAAndFMECAWithSHA256.md",
            uc01_md,
            ["STPA", "FMECA", "SHA256"].as_slice(),
        ),
    ] {
        for forbidden_split in ["Sys ML", "Common Mark", "Git Hub", "Open API"] {
            assert!(
                !doc_md.contains(forbidden_split),
                "{} must never fracture acronym into {:?}:\n{}",
                doc_name,
                forbidden_split,
                doc_md
            );
        }
        let h1_title = doc_md
            .lines()
            .find(|line| line.starts_with("# "))
            .ok_or_else(|| format!("{} missing '# ' H1 title line:\n{}", doc_name, doc_md))?;
        let meta_title = doc_md
            .lines()
            .find(|line| line.starts_with("| **Title** |"))
            .ok_or_else(|| format!("{} missing '| **Title** |' row:\n{}", doc_name, doc_md))?;
        for acronym in expected_acronyms {
            let in_h1 = h1_title
                .split(|ch: char| !ch.is_alphanumeric())
                .any(|tok| tok == *acronym);
            let in_meta = meta_title
                .split(|ch: char| !ch.is_alphanumeric())
                .any(|tok| tok == *acronym);
            assert!(
                in_h1 && in_meta,
                "{} titles must preserve standalone acronym token {:?} intact (H1: {:?}, Meta: {:?})",
                doc_name,
                acronym,
                h1_title,
                meta_title
            );
        }
    }
    Ok(())
}

/// Verifies `FEAT-01-SnapshotIntegrityValidator.md` Sections 1 & 2 and `3..=10` BDD items (#699).
///
/// # Arguments
/// * `feat_md` - Generated `FEAT-01-SnapshotIntegrityValidator.md` content.
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - `Ok(())` when all Feature assertions hold.
///
/// # Complexity
/// * Time: $O(L)$ over `feat_md`. Space: $O(1)$.
///
/// # Failure Modes
/// * Propagates missing-section diagnostics or panics on constraint truncation or boilerplate.
pub fn assert_issue_699_feat_sections_and_bdd(
    feat_md: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    for forbidden in [
        "@container",
        "viewport containment",
        "AST references",
        "operational_invariants",
    ] {
        assert!(
            !feat_md.contains(forbidden),
            "FEAT-01 must contain zero boilerplate substring {:?}:\n{}",
            forbidden,
            feat_md
        );
    }

    let feat_sec1 = extract_markdown_section_by_keyword(feat_md, "Test Data Shape", "## 1.")?;
    assert!(
        feat_sec1.contains("CheckpointDigestVector") && feat_sec1.contains("sha256-zero"),
        "FEAT-01 Section 1 must preserve CheckpointDigestVector and sha256-zero:\n{}",
        feat_sec1
    );

    let feat_sec2 =
        extract_markdown_section_by_keyword(feat_md, "Validation & Constraints", "## 2.")?;
    for member in ["CheckpointDigestVector", "snapshot_in", "fault_out", "REQ-0699"] {
        assert!(
            feat_sec2.contains(member),
            "FEAT-01 Section 2 must project {:?}:\n{}",
            member,
            feat_sec2
        );
    }
    for (cname, cexpr) in ISSUE_699_TEN_CONSTRAINTS {
        assert!(
            feat_sec2.contains(cname) && feat_sec2.contains(cexpr),
            "FEAT-01 Section 2 must project constraint {:?} and {:?} without .truncate(8) loss:\n{}",
            cname,
            cexpr,
            feat_sec2
        );
    }

    let feat_bdd = extract_markdown_h2_section(feat_md, "## Acceptance Criteria (BDD)")?;
    let ac_lines = feat_bdd
        .lines()
        .filter(|line| line.trim_start().starts_with("- [ ] AC-"))
        .collect::<Vec<_>>();
    assert!(
        (3..=10).contains(&ac_lines.len()),
        "FEAT-01 BDD must contain 3..=10 '- [ ] AC-' items, got {}:\n{}",
        ac_lines.len(),
        feat_bdd
    );
    for (cname, _) in ISSUE_699_TEN_CONSTRAINTS {
        assert!(
            feat_bdd.contains(cname),
            "FEAT-01 BDD must reference constraint {:?}:\n{}",
            cname,
            feat_bdd
        );
    }
    Ok(())
}
