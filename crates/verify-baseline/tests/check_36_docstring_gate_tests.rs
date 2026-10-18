//! Integration tests for Check 36: Per-Item 7-Section Safety Docstring Completeness Gate
//! across Rust (`crates/**/src/**/*.rs`) and Dart (`app_flutter/lib/**/*.dart`) sources.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use verify_baseline::checks::{
    audit_dart_source_docstrings, audit_rust_source_docstrings, audit_source_purity,
    check_docstring_completeness,
};
use walkdir::WalkDir;

struct ScopedTempWorkspace {
    root: PathBuf,
}

impl ScopedTempWorkspace {
    fn new(prefix: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!(
            "deap_check36_{}_{}_{}",
            prefix,
            std::process::id(),
            nanos
        ));
        fs::create_dir_all(&root).expect("create temp workspace");
        Self { root }
    }
}

impl Drop for ScopedTempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn test_check_36_rejects_placeholder_and_missing_docstrings() {
    // 1. Reproduce exact Tier 2 defect from DEAP01-cuas-coa-engine:
    // app_flutter/lib/app/app.dart:10-14 where `const MyApp({super.key});` had
    // `/// Member documentation.` and `class MyApp` lacked the 7-section docstring.
    let defective_dart = r#"import 'package:flutter/material.dart';

class MyApp extends StatelessWidget {
  /// Member documentation.
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return const MaterialApp(
      home: Scaffold(),
    );
  }
}
"#;

    let dart_res = audit_dart_source_docstrings(defective_dart, "app_flutter/lib/app/app.dart");
    assert!(
        dart_res.is_err(),
        "Check 36 must reject Dart file with '/// Member documentation.' and missing 7-section docstrings"
    );
    let dart_errs = dart_res.unwrap_err().join("\n");
    assert!(
        dart_errs.contains("Member documentation."),
        "Expected placeholder docstring diagnostic for '/// Member documentation.', got:\n{}",
        dart_errs
    );
    assert!(
        dart_errs.contains("MyApp"),
        "Expected missing 7-section docstring diagnostic on class/constructor MyApp, got:\n{}",
        dart_errs
    );

    // 2. Test Rust source with `/// Member documentation.` and missing 7-section docstrings
    let defective_rust = r#"/// Member documentation.
pub struct SensorPacket {
    pub seq: u64,
}

pub fn decode_packet(raw: &[u8]) -> Option<SensorPacket> {
    if raw.is_empty() {
        return None;
    }
    Some(SensorPacket { seq: 1 })
}
"#;

    let rust_res = audit_rust_source_docstrings(defective_rust, "crates/demo/src/lib.rs");
    assert!(
        rust_res.is_err(),
        "Check 36 must reject Rust file with '/// Member documentation.' and missing 7-section docstrings"
    );
    let rust_errs = rust_res.unwrap_err().join("\n");
    assert!(
        rust_errs.contains("Member documentation."),
        "Expected placeholder docstring diagnostic in Rust source, got:\n{}",
        rust_errs
    );
    assert!(
        rust_errs.contains("SensorPacket") && rust_errs.contains("decode_packet"),
        "Expected diagnostics for both SensorPacket and decode_packet, got:\n{}",
        rust_errs
    );

    // 3. Verify end-to-end workspace check_docstring_completeness fails closed on temp workspace
    let ws = ScopedTempWorkspace::new("placeholder_and_missing");
    let dart_dir = ws.root.join("app_flutter").join("lib").join("app");
    fs::create_dir_all(&dart_dir).expect("create app_flutter/lib/app");
    fs::write(dart_dir.join("app.dart"), defective_dart).expect("write app.dart");

    let ws_res = check_docstring_completeness(&ws.root);
    assert!(
        ws_res.is_err(),
        "check_docstring_completeness must fail closed on workspace with defective app.dart"
    );
}

#[test]
fn test_check_36_rejects_undocumented_rust_items_even_when_lib_rs_has_module_doc() {
    let ws = ScopedTempWorkspace::new("no_module_exemption");
    let src_dir = ws.root.join("crates").join("sample-crate").join("src");
    fs::create_dir_all(&src_dir).expect("create crates/sample-crate/src");

    // lib.rs has a complete 7-section `//!` module header, followed immediately by an
    // undocumented struct and function, plus a submodule `submod.rs` with undocumented items.
    let lib_rs_with_module_doc = r#"//! Sample crate module documentation.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Module-level safety intent under DO-178C Level A.
//!
//! ## 2. Invariants & Mathematical Properties
//! Module-level invariants hold across exported types.
//!
//! ## 3. Preconditions
//! Caller supplies valid UTF-8 inputs.
//!
//! ## 4. Postconditions
//! Module operations return deterministic results.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(N)
//! - Space Complexity: O(N)
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Rejects malformed inputs.
//! - State: Preserves immutable state.
//! - Action: Deterministic execution.
//! - Resource: Bounded memory.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-SAMPLE-MOD]

pub mod submod;

pub struct UndocumentedInLib {
    pub value: u32,
}

pub fn undocumented_lib_fn(x: u32) -> u32 {
    x.saturating_add(1)
}
"#;

    let submod_rs = r#"//! Submodule header with 7 sections that must never exempt items.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Submodule safety intent.
//!
//! ## 2. Invariants & Mathematical Properties
//! Submodule invariants.
//!
//! ## 3. Preconditions
//! Submodule preconditions.
//!
//! ## 4. Postconditions
//! Submodule postconditions.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(1)
//! - Space Complexity: O(1)
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: None.
//! - State: Pure.
//! - Action: Infallible.
//! - Resource: O(1).
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-SUBMOD]

pub enum UndocumentedEnum {
    Alpha,
    Beta,
}

pub trait UndocumentedTrait {
    fn compute(&self) -> u32;
}

struct PrivateUndocumentedStruct {
    count: usize,
}

impl PrivateUndocumentedStruct {
    fn increment(&mut self) {
        self.count = self.count.saturating_add(1);
    }
}
"#;

    fs::write(src_dir.join("lib.rs"), lib_rs_with_module_doc).expect("write lib.rs");
    fs::write(src_dir.join("submod.rs"), submod_rs).expect("write submod.rs");

    let res = check_docstring_completeness(&ws.root);
    assert!(
        res.is_err(),
        "Check 36 must NEVER exempt structs, enums, traits, or functions when lib.rs or submodule has //! module doc"
    );
    let errs = res.unwrap_err().join("\n");
    for expected_symbol in [
        "UndocumentedInLib",
        "undocumented_lib_fn",
        "UndocumentedEnum",
        "UndocumentedTrait",
        "PrivateUndocumentedStruct",
        "increment",
    ] {
        assert!(
            errs.contains(expected_symbol),
            "Expected Check 36 to flag undocumented symbol '{}' despite //! module doc, got:\n{}",
            expected_symbol,
            errs
        );
    }
}

#[test]
fn test_live_workspace_crates_have_100_percent_per_item_7_section_docstrings() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("repo root");

    // 1. Verify docstring_gate.rs, source_ast.rs, docstring.rs, and code.rs satisfy Check 34 source purity and Check 36 7-section Rustdoc
    for rel_mod in [
        "crates/verify-baseline/src/checks/docstring_gate.rs",
        "crates/deap-core/src/source_ast.rs",
        "crates/compile-sysml/src/docstring.rs",
        "crates/ingest-sysml/src/translators/code.rs",
    ] {
        let mod_path = repo_root.join(rel_mod);
        let mod_src = fs::read_to_string(&mod_path).expect("read module source");
        let purity_res = audit_source_purity(&mod_src, rel_mod, true);
        assert!(
            purity_res.is_ok(),
            "{} failed Check 34 source purity: {:?}",
            rel_mod,
            purity_res.err()
        );
        let doc_res = audit_rust_source_docstrings(&mod_src, rel_mod);
        assert!(
            doc_res.is_ok(),
            "{} failed Check 36 per-item docstring gate: {:?}",
            rel_mod,
            doc_res.err()
        );
    }

    let runner_path = repo_root.join("crates/verify-baseline/src/runner.rs");
    let runner_src = fs::read_to_string(&runner_path).expect("read runner.rs");
    let runner_doc_res =
        audit_rust_source_docstrings(&runner_src, "crates/verify-baseline/src/runner.rs");
    assert!(
        runner_doc_res.is_ok(),
        "runner.rs failed Check 36 per-item docstring gate: {:?}",
        runner_doc_res.err()
    );

    // 2. Verify 100% of non-test .rs files across all 6 workspace crates pass Check 36
    let crates_dir = repo_root.join("crates");
    let mut total_files = 0usize;
    let mut total_items = 0usize;
    let mut all_violations = Vec::new();

    for entry_res in WalkDir::new(&crates_dir).into_iter().filter_entry(|e| {
        if e.file_type().is_dir() {
            let name = e.file_name().to_string_lossy();
            name != "target"
                && name != ".git"
                && name != "tests"
                && name != "benches"
                && name != "examples"
        } else {
            true
        }
    }) {
        let entry = entry_res.expect("walkdir entry");
        if entry.file_type().is_file()
            && entry
                .path()
                .extension()
                .map(|ext| ext == "rs")
                .unwrap_or(false)
        {
            total_files += 1;
            let rel = entry
                .path()
                .strip_prefix(repo_root)
                .unwrap_or(entry.path())
                .display()
                .to_string();
            let content = fs::read_to_string(entry.path()).expect("read rs file");
            match audit_rust_source_docstrings(&content, &rel) {
                Ok(count) => {
                    total_items += count;
                }
                Err(errs) => {
                    all_violations.extend(errs);
                }
            }
        }
    }

    assert!(
        total_files >= 40,
        "Expected at least 40 non-test .rs files across crates/, scanned {}",
        total_files
    );
    assert!(
        all_violations.is_empty(),
        "Live workspace crates have {} Check 36 per-item 7-section docstring violation(s) (first 25 shown):\n{}",
        all_violations.len(),
        all_violations
            .iter()
            .take(25)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(
        total_items >= 200,
        "Expected at least 200 documented Rust items across workspace crates, verified {}",
        total_items
    );

    let full_gate = check_docstring_completeness(repo_root);
    assert!(
        full_gate.is_ok(),
        "check_docstring_completeness failed on live workspace: {:?}",
        full_gate.err()
    );
}
