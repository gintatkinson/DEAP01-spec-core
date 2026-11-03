#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Unit tests for `audit_code` self-hosting cleanliness, directory walk, and atomic write snapshot isolation.
//! Realises: [REQ-0606-SELF-HOSTING-UNIT-TEST]

use crate::audit_code::{run_audit_code, AuditCodeArgs, AuditPillar, FivePillarStaticAstAnalyzer};
use crate::audit_code_scanner::collect_target_source_files;
use std::fs;
use std::path::{Path, PathBuf};

/// Verifies self-hosting 5-pillar static cleanliness of `audit_code.rs` and CLI pillar parsing.
#[test]
fn test_audit_code_self_hosting_cleanliness_and_pillar_tokens() {
    let src = include_str!("audit_code.rs");
    let analyzer = FivePillarStaticAstAnalyzer::default();
    let findings = analyzer.evaluate_source_file_pillars(
        Path::new("."),
        "crates/verify-baseline/src/audit_code.rs",
        src,
        None,
    );
    assert!(
        findings.is_empty(),
        "Expected audit_code.rs to have 0 self-hosting findings, got: {findings:?}"
    );
    assert_eq!(AuditPillar::from_cli_token("all"), Ok(None));
    assert_eq!(
        AuditPillar::from_cli_token("memory-safety"),
        Ok(Some(AuditPillar::MemorySafety))
    );
    assert_eq!(
        AuditPillar::from_cli_token("resource-lifecycle"),
        Ok(Some(AuditPillar::ResourceLifecycle))
    );
    assert_eq!(
        AuditPillar::from_cli_token("concurrency"),
        Ok(Some(AuditPillar::Concurrency))
    );
    assert_eq!(
        AuditPillar::from_cli_token("test-integrity"),
        Ok(Some(AuditPillar::TestIntegrity))
    );
    assert_eq!(
        AuditPillar::from_cli_token("semantic-traceability"),
        Ok(Some(AuditPillar::SemanticTraceability))
    );
    assert!(AuditPillar::from_cli_token("unknown-pillar").is_err());
}

struct ScopedTempDir {
    path: PathBuf,
}

impl ScopedTempDir {
    fn new(prefix: &str) -> Result<Self, std::io::Error> {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let seq = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "audit_code_walk_{}_{}_{}_{}",
            prefix,
            std::process::id(),
            nanos,
            seq
        ));
        fs::create_dir_all(&dir)?;
        Ok(Self { path: dir })
    }
}

impl Drop for ScopedTempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Verifies iterative pre-order lexicographical directory traversal and cache/hidden exclusions.
#[test]
fn test_collect_target_source_files_iterative_preorder_and_exclusions(
) -> Result<(), Box<dyn std::error::Error>> {
    let scratch = ScopedTempDir::new("preorder")?;
    let root = &scratch.path;
    for d in ["b_dir/sub_dir", "d_dir", ".hidden", "target", "build", "node_modules"] {
        fs::create_dir_all(root.join(d))?;
    }
    let (f_a, f_b1, f_b_deep, f_bz, f_c, f_d) = (
        root.join("a_first.rs"),
        root.join("b_dir/nested_1.rs"),
        root.join("b_dir/sub_dir/deep.dart"),
        root.join("b_dir/z_after_sub.rs"),
        root.join("c_middle.dart"),
        root.join("d_dir/leaf.rs"),
    );
    for file in [&f_a, &f_b1, &f_b_deep, &f_bz, &f_c, &f_d] {
        fs::write(file, "// fixture")?;
    }
    for ignored in [
        "notes.md",
        ".hidden/ignored.rs",
        "target/ignored.rs",
        "build/ignored.dart",
        "node_modules/ignored.rs",
    ] {
        fs::write(root.join(ignored), "// ignored")?;
    }
    let mut collected = Vec::new();
    assert_eq!(collect_target_source_files(root, &mut collected), Ok(()));
    assert_eq!(collected, vec![f_a.clone(), f_b1, f_b_deep, f_bz, f_c, f_d]);
    let mut single_out = Vec::new();
    assert_eq!(collect_target_source_files(&f_a, &mut single_out), Ok(()));
    assert_eq!(single_out, vec![f_a]);
    Ok(())
}

/// Verifies that `run_audit_code` and `runner::write_defect_dossier` persist defect dossiers
/// atomically via sibling temp-file rename without mutating pre-opened reader descriptors.
#[test]
fn test_audit_code_and_runner_atomic_write_snapshot_isolation(
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Read;

    let scratch = ScopedTempDir::new("atomic_persist")?;
    let root = &scratch.path;
    let src_dir = root.join("src");
    let defects_dir = root.join(".pipeline").join("defects");
    fs::create_dir_all(&src_dir)?;
    fs::create_dir_all(&defects_dir)?;

    let sample_rs = src_dir.join("sample.rs");
    fs::write(
        &sample_rs,
        "pub fn untraced_demo_fn(x: u32) -> u32 {\n    x.saturating_add(1)\n}\n",
    )?;
    let expected_dossier = defects_dir.join("DEFECT-src-sample-rs-e0605-trace-l1.md");
    fs::write(&expected_dossier, "STALE_PRE_EXISTING_SNAPSHOT\n")?;
    let mut pre_open_reader = fs::File::open(&expected_dossier)?;

    let args = AuditCodeArgs {
        path: PathBuf::from("src/sample.rs"),
        pillar: Some(AuditPillar::SemanticTraceability),
        issue: None,
        repo: None,
        publish: false,
        dry_run: true,
    };
    assert_eq!(run_audit_code(root, &args), Ok(()));

    let mut old_fd_snapshot = String::new();
    pre_open_reader.read_to_string(&mut old_fd_snapshot)?;
    assert_eq!(old_fd_snapshot, "STALE_PRE_EXISTING_SNAPSHOT\n");
    assert!(fs::read_to_string(&expected_dossier)?.starts_with("## 1. Context and References"));

    crate::runner::write_defect_dossier(
        root,
        "Check 10 (.gitignore exists)",
        &["synthetic atomic runner probe".to_string()],
    );
    let mut saw_runner_json = false;
    let mut saw_runner_md = false;
    for entry_res in fs::read_dir(&defects_dir)? {
        let fname = entry_res?.file_name().to_string_lossy().into_owned();
        assert!(!fname.contains(".tmp."), "left temp file: {fname}");
        saw_runner_json |= fname.starts_with("defect_") && fname.ends_with(".json");
        saw_runner_md |= fname.starts_with("defect_") && fname.ends_with(".md");
    }
    assert!(saw_runner_json && saw_runner_md);
    Ok(())
}
