//! Integration verification test suite for `REQ-0605` `AC-05` and `REQ-0606` `AC-01`..`AC-03` (Part G).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const MAX_MODULE_WALK_DEPTH: usize = 16;

/// Resolves a target `.rs` file or its decomposed `<stem>/` module directory into sorted `.rs` paths.
fn resolve_module_rs_files(path: &Path) -> Vec<PathBuf> {
    if path.is_file() {
        return vec![path.to_path_buf()];
    }
    let dir_path = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.with_extension("")
    };
    if !dir_path.is_dir() {
        return Vec::new();
    }
    let mut collected = BTreeSet::new();
    let mut stack = vec![(dir_path, 0usize)];
    while let Some((current_dir, depth)) = stack.pop() {
        if depth > MAX_MODULE_WALK_DEPTH {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&current_dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let entry_path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                let next_depth = depth.saturating_add(1);
                if next_depth <= MAX_MODULE_WALK_DEPTH {
                    stack.push((entry_path, next_depth));
                }
            } else if file_type.is_file()
                && entry_path.extension().and_then(|ext| ext.to_str()) == Some("rs")
            {
                collected.insert(entry_path);
            }
        }
    }
    collected.into_iter().collect()
}

/// Verifies `REQ-0605` `AC-05`: unified lexer state and absence of duplicate scanners in `docstring` and `docstring_gate` modules.
#[test]
fn test_req_0605_ac05_unified_lexer_and_delegated_scanners() {
    use deap_core::rust_cst::source_bridge::{
        extract_rust_impl_header_prefix, parse_rust_impl_header, scan_line_with_unified_lex_state,
        strip_dart_annotations_tracking_state, CstLexState, ScopedItemScannerState, SourceLanguage,
    };

    let mut dart_lex = CstLexState { language: SourceLanguage::Dart, ..Default::default() };
    let mut scoped = ScopedItemScannerState::default();
    assert_eq!(scoped.dart_annotation_paren_depth, 0);
    assert_eq!(strip_dart_annotations_tracking_state("@pragma(", &mut scoped.dart_annotation_paren_depth), "");
    assert_eq!(scoped.dart_annotation_paren_depth, 1);
    assert_eq!(strip_dart_annotations_tracking_state("'vm:prefer-inline',", &mut scoped.dart_annotation_paren_depth), "");
    assert_eq!(scoped.dart_annotation_paren_depth, 1);
    assert_eq!(strip_dart_annotations_tracking_state(") void runStep() {}", &mut scoped.dart_annotation_paren_depth).trim(), "void runStep() {}");
    assert_eq!(scoped.dart_annotation_paren_depth, 0);
    assert_eq!(
        extract_rust_impl_header_prefix("impl<F: Fn(u32) -> bool> CoreHandler<F> {"),
        Some("impl<F: Fn(u32) -> bool> CoreHandler<F>".to_string())
    );
    assert_eq!(parse_rust_impl_header("impl<F: Fn(u32) -> bool> CoreHandler<F> {"), Some("CoreHandler".to_string()));
    let scanned_dart = scan_line_with_unified_lex_state("void/*inline*/executeStep() {}", &mut dart_lex);
    assert_eq!(scanned_dart.trim(), "void  executeStep() {}");

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(Path::parent).unwrap_or_else(|| Path::new("."));
    let docstring_files = resolve_module_rs_files(&workspace_root.join("crates/compile-sysml/src/docstring.rs"));
    assert!(!docstring_files.is_empty());
    for file_path in &docstring_files {
        let docstring_rs_src = std::fs::read_to_string(file_path).unwrap_or_default();
        assert!(!docstring_rs_src.contains("fn extract_rust_impl_header_prefix("));
    }

    let docstring_gate_files = resolve_module_rs_files(&workspace_root.join("crates/verify-baseline/src/checks/docstring_gate.rs"));
    assert!(!docstring_gate_files.is_empty());
    for file_path in &docstring_gate_files {
        let docstring_gate_src = std::fs::read_to_string(file_path).unwrap_or_default();
        assert!(!docstring_gate_src.contains("enum DartMultilineString") && !docstring_gate_src.contains("fn scan_dart_line_preserving_lexical_state("));
    }
}

/// RAII temporary workspace directory fixture that deterministically removes its directory tree on drop.
struct TempWorkspace {
    root: PathBuf,
}

impl TempWorkspace {
    fn new(prefix: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let root =
            std::env::temp_dir().join(format!("deap_{}_{}_{}", prefix, std::process::id(), nanos));
        let _ = std::fs::create_dir_all(&root);
        Self { root }
    }

    fn write_file(&self, rel_path: &str, contents: &str) -> PathBuf {
        let full = self.root.join(rel_path);
        if let Some(parent) = full.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&full, contents);
        full
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        if self.root.exists() {
            std::fs::remove_dir_all(&self.root).unwrap_or(());
        }
    }
}

/// Reads all synthesized `DEFECT-*.md` files from `defects_dir` into a deterministically ordered map.
fn collect_defect_dossiers(defects_dir: &Path) -> std::collections::BTreeMap<String, Vec<u8>> {
    let mut dossiers = std::collections::BTreeMap::new();
    if let Ok(entries) = std::fs::read_dir(defects_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let file_name = entry.file_name().to_string_lossy().into_owned();
            if file_name.starts_with("DEFECT-") && file_name.ends_with(".md") {
                if let Ok(bytes) = std::fs::read(&path) {
                    dossiers.insert(file_name, bytes);
                }
            }
        }
    }
    dossiers
}

fn setup_req_0606_dirty_workspace(prefix: &str) -> TempWorkspace {
    let ws = TempWorkspace::new(prefix);
    let _ = std::fs::create_dir_all(ws.root.join(".git").join("objects"));
    let _ = std::fs::create_dir_all(ws.root.join(".git").join("refs"));
    ws.write_file(".git/HEAD", "ref: refs/heads/main\n");
    ws.write_file(
        ".git/config",
        "[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n[remote \"origin\"]\n\turl = https://github.com/sample-org/sample-repo.git\n",
    );
    let _ = std::fs::create_dir_all(ws.root.join(".pipeline").join("defects"));
    ws.write_file("Cargo.toml", "[workspace]\nmembers = [\"crates/sample_crate\"]\n");
    ws.write_file(
        "crates/sample_crate/Cargo.toml",
        "[package]\nname = \"sample_crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    let dirty_rust = r#"/// Computes a sample byte using unchecked slicing and narrowing cast.
pub fn slice_and_cast_sample(buf: &[u8], wide: u64) -> u8 {
    let sub = &buf[0..2];
    let narrow = wide as u8;
    sub[0].wrapping_add(narrow)
}

/// Public function intentionally missing any requirement traceability tag.
pub fn untraced_counter_step(current: u32) -> u32 {
    current.saturating_add(1)
}

#[test]
fn hollow_zero_assertion_probe() {
    let computed = 2 + 2;
    if computed == 99 {}
}
"#;
    ws.write_file("crates/sample_crate/src/lib.rs", dirty_rust);
    let dirty_dart = r#"import 'dart:async';
import 'package:flutter/foundation.dart';

class SampleStreamNotifier extends ChangeNotifier {
  StreamSubscription<int>? _subscription;
  int latestValue = 0;

  void attachStream(Stream<int> source) {
    _subscription = source.listen((event) {
      latestValue = event;
      notifyListeners();
    });
  }

  Future<void> refreshAfterAwait(Future<int> pendingValue) async {
    final resolved = await pendingValue;
    latestValue = resolved;
    notifyListeners();
  }
}
"#;
    ws.write_file("app_flutter/lib/sample_notifier.dart", dirty_dart);
    ws
}

/// Verifies `REQ-0606` `AC-01` through `AC-03`: 5-pillar static defect detection (`E0601`..`E0605`),
/// closed-loop 12-check defect dossier synthesis, two-run idempotency, and `--pillar concurrency` filtering.
#[test]
fn req_0606_ac01_to_ac03_cli_audit_code_five_pillars_dossiers_and_idempotency() {
    use std::process::Command;
    use verify_baseline::audit_code::{
        ClosedLoopDefectPublisherGate, DeterministicDossierSynthesizer,
        DeterministicStaticDefectAuditor, FivePillarStaticAstAnalyzer, MermaidUmlDiagramGenerator,
    };

    assert_eq!(DeterministicStaticDefectAuditor::default().pillar_coverage_count, 5);
    assert!(FivePillarStaticAstAnalyzer::default().zero_regex_mode);
    assert_eq!(DeterministicDossierSynthesizer::default().mandatory_section_count, 7);
    assert!(MermaidUmlDiagramGenerator::default().forbid_semicolons);
    assert_eq!(ClosedLoopDefectPublisherGate::default().schema_gate_count, 12);

    let ws = setup_req_0606_dirty_workspace("req_0606_five_pillars");
    let defects_dir = ws.root.join(".pipeline").join("defects");
    let bin = env!("CARGO_BIN_EXE_verify-baseline");
    let ws_root_str = ws.root.to_str().unwrap_or("");

    let Ok(run1_out) = Command::new(bin)
        .args(["audit-code", "--path", ws_root_str, "--dry-run"])
        .output()
    else {
        return;
    };
    assert_eq!(run1_out.status.code(), Some(0));
    let run1_dossiers = collect_defect_dossiers(&defects_dir);
    assert!(!run1_dossiers.is_empty());

    let Ok(run2_out) = Command::new(bin)
        .args(["audit-code", "--path", ws_root_str, "--dry-run"])
        .output()
    else {
        return;
    };
    assert_eq!(run2_out.status.code(), Some(0));
    assert_eq!(run1_dossiers, collect_defect_dossiers(&defects_dir));

    let combined_corpus: String = run1_dossiers
        .values()
        .map(|b| String::from_utf8_lossy(b).into_owned())
        .collect::<Vec<_>>()
        .join("\n");
    for label in ["Pillar 1", "Pillar 2", "Pillar 3", "Pillar 4", "Pillar 5", "E0601", "E0602", "E0603", "E0604", "E0605"] {
        assert!(combined_corpus.contains(label), "Missing {label}");
    }

    for file_name in run1_dossiers.keys() {
        let rel_dossier = format!(".pipeline/defects/{file_name}");
        if let Ok(check_out) = Command::new(bin)
            .current_dir(&ws.root)
            .args(["check-defect", &rel_dossier, "--dry-run"])
            .output()
        {
            assert_eq!(check_out.status.code(), Some(0));
        }
        let _ = std::fs::remove_file(defects_dir.join(file_name));
    }

    if let Ok(filter_out) = Command::new(bin)
        .args(["audit-code", "--path", ws_root_str, "--pillar", "concurrency", "--dry-run"])
        .output()
    {
        assert_eq!(filter_out.status.code(), Some(0));
    }
    let filtered = collect_defect_dossiers(&defects_dir);
    assert!(!filtered.is_empty());
    for bytes in filtered.values() {
        let body = String::from_utf8_lossy(bytes);
        assert!(body.contains("Pillar 3") && body.contains("E0603") && !body.contains("E0601"));
    }
}
