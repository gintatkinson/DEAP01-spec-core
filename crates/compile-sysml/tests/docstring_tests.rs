#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! [DOC-TIER: A] Unit & Integration Tests for `compile-sysml` CST Docstring Injection & Source Extraction (`BP-05`).
//! ## 1. Safety Intent & Regulatory Scope
//! Verifies lossless CST docstring extraction, surgical `TextEdit` splicing, 4-block and 7-section
//! compliance, schema traceability binding, and Tier B/C compaction under DO-178C Level A and DO-330 TQL-4.
//! ## 2. Invariants & Mathematical Properties
//! - Second-pass docstring injection is a strict fixed point (`second_count == 0`).
//! - Unforced `Realises` conflicts fail closed with `DocstringExitCode::IdempotencyOrConflict`.
//! ## 3. Preconditions
//! - Writable OS temporary directory for isolated test fixtures.
//! ## 4. Postconditions
//! - Temporary directories are cleaned up prior to test completion.
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ over source fixture bytes; Space Complexity: $O(N)$.
//! ## 6. Failure Modes & Fault Tolerance
//! - Propagates parse and I/O errors explicitly via `Result`.
//! ## 7. Realization & Traceability
//! Safety & Traceability: Realises: [REQ-COMPILER-SYSML-DOCSTRING], [REQ-0608], [DO-330-TQL-4-TEST]
//! Contract: Verifies lossless CST docstring extraction, surgical `TextEdit` splicing, and schema binding.
//! Bounds & Faults: Self-cleaning temporary directories in `std::env::temp_dir()`; deterministic assertions.

use crate::docstring::{
    collect_all_parts, derive_module_req_tag, inject_dart_file_docstrings,
    inject_rust_file_docstrings, probe_single_char_literal, resolve_schema_context_strict,
    run_docstring_injection, run_extract_source, DocstringExitCode, DocstringInjectionOptions,
    MAX_AST_DEPTH,
};
use crate::docstring_compliance_check::{
    has_all_seven_sections, has_valid_four_block_contract, is_compliant_docblock,
    is_compliant_docblock_with_mode,
};
use crate::docstring_cst::{
    apply_cst_docstring_edits, compact_tier_bc_tree, extract_cst_item_doc_lines,
    has_cst_inner_module_doc, list_rust_items, plan_cst_docstring_edits, ParsedRustFile,
};
use crate::docstring_schema_index::SchemaLookupIndex;
use crate::parse_sysml;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static DOCSTRING_TEST_SEQ: AtomicU64 = AtomicU64::new(1);

/// Creates a unique self-contained temporary directory for docstring test fixtures.
/// Realises: [REQ-COMPILER-SYSML-DOCSTRING]
fn unique_doc_temp_dir(prefix: &str) -> PathBuf {
    let seq = DOCSTRING_TEST_SEQ.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!(
        ".deap_docstring_test_{}_{}_{}",
        prefix,
        std::process::id(),
        seq
    ));
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::create_dir_all(&dir);
    dir
}

#[test]
fn test_cst_extract_leading_docblock_and_surgical_text_edits() -> Result<(), String> {
    let orig_src = "//! [DOC-TIER: A] Module header.\n/// Existing summary.\n/// Realises: [REQ-01]\npub fn alpha() -> i32 {\n    1\n}\n";
    assert!(has_cst_inner_module_doc(orig_src));
    let parsed = ParsedRustFile::parse_rust_source(orig_src);
    let items = list_rust_items(&parsed);
    let first_item = items.first().ok_or("expected alpha item")?;
    let docs = extract_cst_item_doc_lines(first_item);
    assert_eq!(docs.len(), 2);

    let mut replacements = BTreeMap::new();
    replacements.insert(
        first_item.start_line,
        "/// Extra contract line.\n".to_string(),
    );
    let edits = plan_cst_docstring_edits(orig_src, &replacements);
    assert_eq!(edits.len(), 1);
    let spliced = apply_cst_docstring_edits(orig_src, &edits);
    assert!(spliced.contains("/// Extra contract line."));
    assert!(spliced.contains("pub fn alpha() -> i32"));
    Ok(())
}

#[test]
fn test_compliance_check_recognizes_7_section_and_4_block_contracts() {
    let four_block = vec![
        "/// Summary line.".to_string(),
        "/// Safety & Traceability: Realises: [REQ-01]".to_string(),
        "/// Contract: Deterministic contract.".to_string(),
        "/// Bounds & Faults: O(1) time and space.".to_string(),
    ];
    assert!(has_valid_four_block_contract(&four_block));
    assert!(is_compliant_docblock(&four_block, false));

    let seven_sec = vec![
        "/// Summary.".to_string(),
        "/// ## 1. Safety Intent & Regulatory Scope".to_string(),
        "/// Conforms to DO-178C Level A.".to_string(),
        "/// ## 2. Invariants & Mathematical Properties".to_string(),
        "/// Deterministic.".to_string(),
        "/// ## 3. Preconditions".to_string(),
        "/// Valid input.".to_string(),
        "/// ## 4. Postconditions".to_string(),
        "/// Valid output.".to_string(),
        "/// ## 5. Algorithmic Complexity Bounds".to_string(),
        "/// - Time Complexity: O(1).".to_string(),
        "/// - Space Complexity: O(1).".to_string(),
        "/// ## 6. Failure Modes & Fault Tolerance".to_string(),
        "/// Infallible.".to_string(),
        "/// ## 7. Realization & Traceability".to_string(),
        "/// Realises: [REQ-01], [DO-330-TQL-4-01]".to_string(),
    ];
    assert!(has_all_seven_sections(&seven_sec.join("\n")));
    assert!(is_compliant_docblock_with_mode(&seven_sec, true, false));

    let placeholder_doc = vec![
        "/// TODO: placeholder".to_string(),
        "/// Realises: [REQ-01]".to_string(),
    ];
    assert!(!is_compliant_docblock(&placeholder_doc, false));
}

#[test]
fn test_inject_rust_file_docstrings_synthesizes_and_converges() -> Result<(), String> {
    let source = "\
pub struct TelemetryPacket {
    pub seq: u64,
}

impl TelemetryPacket {
    pub fn next_seq(&self) -> u64 {
        self.seq.saturating_add(1)
    }

    fn internal_helper(flag: bool) -> bool {
        !flag
    }
}
";
    let (injected, count) =
        inject_rust_file_docstrings(source, "TELEMETRY", None, false).map_err(|e| e.to_string())?;
    assert_eq!(
        count, 3,
        "expected struct + 2 methods enriched, got {count}"
    );
    assert!(injected.contains("/// ## 1. Safety Intent & Regulatory Scope"));
    assert!(injected.contains("REQ-TELEMETRY"));

    let (second_pass, second_count) =
        inject_rust_file_docstrings(&injected, "TELEMETRY", None, false)
            .map_err(|e| e.to_string())?;
    assert_eq!(second_count, 0, "second pass must be a fixed point");
    assert_eq!(second_pass, injected);
    Ok(())
}

#[test]
fn test_inject_dart_file_docstrings_synthesizes_and_converges() -> Result<(), String> {
    let dart_src = "\
class FlightState {
  final double altitude;
  FlightState(this.altitude);

  double climb(double delta) {
    assert(delta >= 0.0);
    return altitude + delta;
  }
}
";
    let (injected, count) = inject_dart_file_docstrings(dart_src, "FLIGHT-DART", None, false)
        .map_err(|e| e.to_string())?;
    assert!(count >= 2);
    assert!(injected.contains("/// ## 1. Safety Intent & Regulatory Scope"));
    assert!(injected.contains("REQ-FLIGHT-DART"));

    let (_, second_count) = inject_dart_file_docstrings(&injected, "FLIGHT-DART", None, false)
        .map_err(|e| e.to_string())?;
    assert_eq!(second_count, 0);
    Ok(())
}

#[test]
fn test_schema_lookup_index_and_resolve_schema_context_strict() -> Result<(), String> {
    let sysml = r#"
        package AvionicsPkg {
            requirement def ReqNav {
                id = "REQ-NAV-001";
                text = "Navigation controller shall bound drift.";
            }
            part def NavController {
                satisfy ReqNav;
                attribute drift_rad : Real;
                action def ComputeCorrection;
            }
        }
    "#;
    let pkg = parse_sysml(sysml).map_err(|e| format!("{e:?}"))?;
    let mut parts = Vec::new();
    collect_all_parts(&pkg, &mut parts);
    assert_eq!(parts.len(), 1);
    assert_eq!(MAX_AST_DEPTH, 64);

    let index = SchemaLookupIndex::new(&pkg, None, "NavController");
    assert_eq!(index.all_parts.len(), 1);
    let (realises_opt, _doc_opt) = resolve_schema_context_strict(Some(&pkg), None, "NavController");
    let realises = realises_opt.ok_or("expected resolved schema realises string")?;
    assert!(realises.contains("REQ-NAV-001"));
    Ok(())
}

#[test]
fn test_run_docstring_injection_and_unforced_conflict_detection() -> Result<(), String> {
    let dir = unique_doc_temp_dir("run_inject");
    let rs_file = dir.join("nav_controller.rs");
    fs::write(
        &rs_file,
        "pub struct NavController {\n    pub drift: f64,\n}\n",
    )
    .map_err(|e| e.to_string())?;

    let opts = DocstringInjectionOptions {
        target_path: dir.clone(),
        dry_run: false,
        force: false,
        schema_model: None,
    };
    let first = run_docstring_injection(&opts).map_err(|(_, m)| m)?;
    assert_eq!(first.files_modified, 1);
    assert_eq!(first.items_enriched, 1);
    let second = run_docstring_injection(&opts).map_err(|(_, m)| m)?;
    assert_eq!(second.files_modified, 0);

    let sysml = r#"
        package ConflictPkg {
            requirement def ReqOther {
                id = "REQ-OTHER-999";
                text = "Other req.";
            }
            part def NavController {
                satisfy ReqOther;
            }
        }
    "#;
    let pkg = parse_sysml(sysml).map_err(|e| format!("{e:?}"))?;
    let conflict_opts = DocstringInjectionOptions {
        target_path: dir.clone(),
        dry_run: true,
        force: false,
        schema_model: Some(pkg),
    };
    let conflict_res = run_docstring_injection(&conflict_opts);
    let _ = fs::remove_dir_all(&dir);
    assert!(
        matches!(conflict_res, Err((DocstringExitCode::IdempotencyOrConflict, _))),
        "mismatched Realises ID without --force must return IdempotencyOrConflict: {conflict_res:?}"
    );
    Ok(())
}

#[test]
fn test_run_extract_source_and_compact_tier_bc_tree() -> Result<(), String> {
    let dir = unique_doc_temp_dir("extract_and_compact");
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| e.to_string())?;
    let rs_file = src_dir.join("sensor.rs");
    fs::write(
        &rs_file,
        "/// Verbose private doc line 1.\n/// Verbose line 2.\n/// Verbose line 3.\n/// Verbose line 4.\n/// Realises: [REQ-SENSOR-01]\nfn read_raw() -> u32 {\n    42\n}\n\npub struct SensorUnit {\n    pub reading: u32,\n}\n",
    )
    .map_err(|e| e.to_string())?;

    let compacted_files = compact_tier_bc_tree(&src_dir)?;
    assert_eq!(compacted_files, 1);
    let after_compact = fs::read_to_string(&rs_file).map_err(|e| e.to_string())?;
    assert!(after_compact.contains("/// Verbose private doc line 1."));
    assert!(after_compact.contains("/// Realises: [REQ-SENSOR-01]"));
    assert!(!after_compact.contains("Verbose line 3."));

    let out_sysml = dir.join("extracted.sysml");
    let out_digest = dir.join("extracted-digest.json");
    let pkg = run_extract_source(&src_dir, &out_sysml, &out_digest, false).map_err(|(_, m)| m)?;
    let _ = fs::remove_dir_all(&dir);
    assert!(!pkg.part_defs.is_empty());
    assert_eq!(probe_single_char_literal("'x'"), Some(3));
    assert_eq!(
        derive_module_req_tag("src/sensor_unit.rs"),
        "SENSOR-UNIT"
    );
    Ok(())
}
