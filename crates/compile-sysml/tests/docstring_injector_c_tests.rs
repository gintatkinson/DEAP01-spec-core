//! Integration test suite C for Tier A/B/C docstring synthesis, compaction, and generated skip isolation.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Verifies DO-178C Level A and DO-330 TQL-4 Tier A 7-section synthesis, Tier B/C concise 2-line
//! synthesis, legacy Tier B/C 7-section compaction, `#[test]` immunity, and `modular_codegen/generated`
//! directory skip isolation.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Tier A (`pub`) items receive full 7-section docstrings; Tier B/C items receive concise 2-line docstrings.
//! - Legacy 7-section docstrings on Tier B/C items compact to 2 lines while preserving `Realises` tags.
//! - Files under `modular_codegen/generated/` remain byte-for-byte untouched.
//!
//! ## 3. Preconditions
//! - Writable OS temporary directory for directory compaction and skip isolation tests.
//!
//! ## 4. Postconditions
//! - Temporary directories are cleaned up prior to function return.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ over fixture source strings; Space Complexity: $O(N)$.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Propagates docstring injection and I/O errors explicitly via `Result<(), String>`.
//!
//! ## 7. Realization & Traceability
//! Safety & Traceability: Realises: [REQ-FEAT-42-DOCSTRING-INJECTOR-INTEGRATION-TESTS], [REQ-FEAT-42-TEST-DOCBLOCK-EXTRACTOR]
//! Contract: Verifies tiered Rust docstring synthesis, compaction, and `modular_codegen/generated` skip isolation.
//! Bounds & Faults: Isolated temporary directories under `std::env::temp_dir()`.

use crate::docstring_injector_tests::unique_test_dir;
use compile_sysml::docstring::{
    inject_rust_file_docstrings, run_docstring_injection, DocstringInjectionOptions,
};
use std::fs;

/// Extracts the contiguous `///` docblock lines immediately preceding `decl_needle` (skipping intervening `#[...]` attributes).
/// Realises: [REQ-FEAT-42-TEST-DOCBLOCK-EXTRACTOR]
pub(crate) fn extract_docblock_before_decl<'a>(
    source: &'a str,
    decl_needle: &str,
) -> Result<Vec<&'a str>, String> {
    let lines: Vec<&'a str> = source.lines().collect();
    let decl_idx = lines
        .iter()
        .position(|line| line.trim().starts_with(decl_needle))
        .ok_or_else(|| format!("Declaration needle '{decl_needle}' not found in:\n{source}"))?;

    let mut cursor = decl_idx;
    while cursor > 0 {
        let prev = lines.get(cursor.saturating_sub(1)).copied().unwrap_or("").trim();
        if prev.starts_with("#[") {
            cursor = cursor.saturating_sub(1);
        } else {
            break;
        }
    }

    let mut doc_lines = Vec::new();
    while cursor > 0 {
        let prev = lines.get(cursor.saturating_sub(1)).copied().unwrap_or("").trim();
        if prev.starts_with("///") {
            doc_lines.push(prev);
            cursor = cursor.saturating_sub(1);
        } else {
            break;
        }
    }
    doc_lines.reverse();
    Ok(doc_lines)
}

/// Asserts that every Tier B/C item in `synthesized` has a concise 2-line docstring with `Realises`.
/// Realises: [REQ-FEAT-42-DOCSTRING-INJECTOR-INTEGRATION-TESTS]
fn assert_tier_bc_two_line_docblocks(synthesized: &str) -> Result<(), String> {
    for tier_bc_decl in [
        "fn encode_slot(&self) -> u32;",
        "pub(crate) fn internal_slot(&self)",
        "fn private_slot(&self)",
        "fn encode_slot(&self) -> u32 {",
        "fn default() -> Self",
        "struct PrivateScratch",
        "enum PrivateCursor",
        "const MAX_SLOT_WINDOW: usize = 32;",
        "type SlotWindowIndex = usize;",
        "pub(crate) fn crate_transform(val: u32)",
        "pub(super) fn super_transform(val: u32)",
        "pub(in crate) fn path_transform(val: u32)",
        "fn private_transform(val: u32)",
    ] {
        let doc = extract_docblock_before_decl(synthesized, tier_bc_decl)?;
        assert_eq!(doc.len(), 2, "Tier B/C item {tier_bc_decl} must have 2 lines");
        assert!(!doc.join("\n").contains("## 1."));
        assert!(doc.get(1).is_some_and(|line| line.starts_with("/// Realises: [REQ-")));
    }
    Ok(())
}

/// Verifies undocumented Tier A vs Tier B/C synthesis, `#[test]` immunity, and idempotency.
/// Realises: [REQ-FEAT-42-DOCSTRING-INJECTOR-INTEGRATION-TESTS]
fn verify_undocumented_tiered_synthesis() -> Result<(), String> {
    let undocumented_tiered_src = "\
pub struct RecordFrame { pub slot_id: u32 }
pub enum StageMode { Idle, Active }
pub trait FrameCodec {
    fn encode_slot(&self) -> u32;
}
impl RecordFrame {
    pub fn new(slot_id: u32) -> Self { Self { slot_id } }
    pub(crate) fn internal_slot(&self) -> u32 { self.slot_id }
    fn private_slot(&self) -> u32 { self.slot_id }
}
impl FrameCodec for RecordFrame {
    fn encode_slot(&self) -> u32 { self.slot_id }
}
impl Default for RecordFrame {
    fn default() -> Self { Self { slot_id: 0 } }
}
struct PrivateScratch { offset: usize }
enum PrivateCursor { Head, Tail }
const MAX_SLOT_WINDOW: usize = 32;
type SlotWindowIndex = usize;
pub fn exported_transform(val: u32) -> u32 { val.saturating_add(1) }
pub(crate) fn crate_transform(val: u32) -> u32 { val.saturating_add(2) }
pub(super) fn super_transform(val: u32) -> u32 { val.saturating_add(3) }
pub(in crate) fn path_transform(val: u32) -> u32 { val.saturating_add(4) }
fn private_transform(val: u32) -> u32 { val.saturating_add(5) }
#[test]
fn standalone_unit_probe() { assert_eq!(1 + 1, 2); }
";
    let (synthesized, synth_count) = inject_rust_file_docstrings(
        undocumented_tiered_src,
        "crates/core_module/src/frame_codec.rs",
        None,
        false,
    )
    .map_err(|e| e.into_message())?;
    assert!(synth_count >= 16, "Expected >= 16 items enriched, got {synth_count}");

    for tier_a_decl in [
        "pub struct RecordFrame",
        "pub enum StageMode",
        "pub trait FrameCodec",
        "pub fn new(slot_id: u32)",
        "pub fn exported_transform(val: u32)",
    ] {
        let joined = extract_docblock_before_decl(&synthesized, tier_a_decl)?.join("\n");
        assert!(
            joined.contains("## 1. Safety Intent & Regulatory Scope")
                && joined.contains("## 7. Realization & Traceability")
                && joined.contains("Realises: [REQ-")
        );
    }
    assert_tier_bc_two_line_docblocks(&synthesized)?;
    assert!(extract_docblock_before_decl(&synthesized, "fn standalone_unit_probe()")?.is_empty());
    let (synth_second, synth_second_count) = inject_rust_file_docstrings(
        &synthesized,
        "crates/core_module/src/frame_codec.rs",
        None,
        false,
    )
    .map_err(|e| e.into_message())?;
    assert_eq!(synth_second_count, 0);
    assert_eq!(synthesized, synth_second);
    Ok(())
}

/// Builds the legacy mixed Tier A + Tier B/C Rust source fixture.
/// Realises: [REQ-FEAT-42-DOCSTRING-INJECTOR-INTEGRATION-TESTS]
fn build_legacy_mixed_src() -> String {
    let sec2_to_6 = "/// ## 2. Invariants & Mathematical Properties\n/// - Deterministic.\n/// ## 3. Preconditions\n/// - Valid input.\n/// ## 4. Postconditions\n/// - Valid output.\n/// ## 5. Algorithmic Complexity Bounds\n/// - Time Complexity: O(1); Space Complexity: O(1).\n/// ## 6. Failure Modes & Fault Tolerance\n/// - Infallible.";
    format!(
        "/// Public signal router state.\n/// ## 1. Safety Intent & Regulatory Scope\n/// Conforms to deterministic routing rules for `SignalRouter`.\n{sec2_to_6}\n/// ## 7. Realization & Traceability\n/// Realises: [REQ-ROUTER-PUBLIC-00]\npub struct SignalRouter {{ pub channel_count: usize }}\n\n\
         /// Clamps raw ratio within normalized unit interval.\n/// ## 1. Safety Intent & Regulatory Scope\n/// Conforms to DO-178C Level A safety objectives for fn `clamp_ratio`.\n{sec2_to_6}\n/// ## 7. Realization & Traceability\n/// Realises: [REQ-ROUTER-CLAMP-01] | SysML v2: ActionDef(clamp_ratio)\nfn clamp_ratio(raw: f64) -> f64 {{ raw.clamp(0.0, 1.0) }}\n\n\
         /// ## 1. Safety Intent & Regulatory Scope\n/// Advances discrete step counter with saturating increment.\n{sec2_to_6}\n/// ## 7. Realization & Traceability\n/// Realises: [REQ-ROUTER-STEP-02]\npub(crate) fn step_counter(tick: u64) -> u64 {{ tick.saturating_add(1) }}\n\n\
         impl Default for SignalRouter {{\n    /// Constructs a zero-channel default `SignalRouter`.\n    /// ## 1. Safety Intent & Regulatory Scope\n    /// Conforms to DO-178C Level A safety objectives for `SignalRouter::default`.\n    /// ## 2. Invariants & Mathematical Properties\n    /// - Initializes `channel_count` to `0`.\n    /// ## 3. Preconditions\n    /// - None.\n    /// ## 4. Postconditions\n    /// - Returns default `SignalRouter`.\n    /// ## 5. Algorithmic Complexity Bounds\n    /// - Time Complexity: O(1); Space Complexity: O(1).\n    /// ## 6. Failure Modes & Fault Tolerance\n    /// - Infallible.\n    /// ## 7. Realization & Traceability\n    fn default() -> Self {{ Self {{ channel_count: 0 }} }}\n}}\n\n\
         /// Internal cursor tracking active router offset.\n/// ## 1. Safety Intent & Regulatory Scope\n/// Conforms to DO-178C Level A safety objectives for struct `RouterCursor`.\n{sec2_to_6}\n/// ## 7. Realization & Traceability\n/// Realises: [REQ-ROUTER-CURSOR-03]\nstruct RouterCursor {{ index: usize }}\n"
    )
}

/// Verifies legacy Tier B/C compaction and idempotence.
/// Realises: [REQ-FEAT-42-DOCSTRING-INJECTOR-INTEGRATION-TESTS]
fn verify_tier_bc_compaction(legacy_mixed_src: &str) -> Result<(), String> {
    let (compacted, compact_count) = inject_rust_file_docstrings(
        legacy_mixed_src,
        "crates/core_module/src/signal_router.rs",
        None,
        false,
    )
    .map_err(|e| e.into_message())?;
    assert_eq!(compact_count, 4);
    assert_eq!(compacted.matches("## 1. Safety Intent & Regulatory Scope").count(), 1);

    assert_eq!(
        extract_docblock_before_decl(&compacted, "fn clamp_ratio(raw: f64)")?,
        vec![
            "/// Clamps raw ratio within normalized unit interval.",
            "/// Realises: [REQ-ROUTER-CLAMP-01]",
        ]
    );
    assert_eq!(
        extract_docblock_before_decl(&compacted, "pub(crate) fn step_counter(tick: u64)")?,
        vec![
            "/// Advances discrete step counter with saturating increment.",
            "/// Realises: [REQ-ROUTER-STEP-02]",
        ]
    );
    let default_doc = extract_docblock_before_decl(&compacted, "fn default() -> Self")?;
    assert_eq!(default_doc.len(), 2);
    assert_eq!(default_doc.first().copied(), Some("/// Constructs a zero-channel default `SignalRouter`."));
    assert!(default_doc.get(1).is_some_and(|line| line.starts_with("/// Realises: [REQ-")));
    assert_eq!(
        extract_docblock_before_decl(&compacted, "struct RouterCursor")?,
        vec![
            "/// Internal cursor tracking active router offset.",
            "/// Realises: [REQ-ROUTER-CURSOR-03]",
        ]
    );

    let (compacted_again, compact_again_count) = inject_rust_file_docstrings(
        &compacted,
        "crates/core_module/src/signal_router.rs",
        None,
        false,
    )
    .map_err(|e| e.into_message())?;
    assert_eq!(compact_again_count, 0);
    assert_eq!(compacted, compacted_again);
    Ok(())
}

/// Verifies `modular_codegen/generated` skip isolation.
/// Realises: [REQ-FEAT-42-DOCSTRING-INJECTOR-INTEGRATION-TESTS]
fn verify_generated_skip_isolation(legacy_mixed_src: &str) -> Result<(), String> {
    let generated_legacy_src = "/// Generated stage helper.\n/// ## 1. Safety Intent & Regulatory Scope\n/// Preserves Stage-1 == Stage-2 metacircular SHA-256 parity.\n/// ## 2. Invariants & Mathematical Properties\n/// - Bit-for-bit stable.\n/// ## 3. Preconditions\n/// - Valid token.\n/// ## 4. Postconditions\n/// - Returns token.\n/// ## 5. Algorithmic Complexity Bounds\n/// - O(1).\n/// ## 6. Failure Modes & Fault Tolerance\n/// - Total.\n/// ## 7. Realization & Traceability\n/// Realises: [REQ-METACIRCULAR-STAGE-01]\nfn generated_stage_helper(tok: u32) -> u32 { tok }\n";
    let (gen_out, gen_count) = inject_rust_file_docstrings(
        generated_legacy_src,
        "crates/compile-sysml/src/modular_codegen/generated/stage_emit.rs",
        None,
        false,
    )
    .map_err(|e| e.into_message())?;
    assert_eq!(gen_count, 0);
    assert_eq!(gen_out, generated_legacy_src);

    let dir = unique_test_dir("feat42_tiered_compaction_gen_skip")?;
    let src_dir = dir.join("src");
    let gen_dir = src_dir.join("modular_codegen").join("generated");
    fs::create_dir_all(&gen_dir).map_err(|e| e.to_string())?;
    fs::write(src_dir.join("signal_router.rs"), legacy_mixed_src).map_err(|e| e.to_string())?;
    let generated_file = gen_dir.join("stage_emit.rs");
    fs::write(&generated_file, generated_legacy_src).map_err(|e| e.to_string())?;

    let report = run_docstring_injection(&DocstringInjectionOptions {
        target_path: src_dir,
        dry_run: false,
        force: false,
        schema_model: None,
    })
    .map_err(|(c, m)| format!("dir compaction ({c:?}): {m}"))?;
    assert_eq!(report.files_scanned, 1);
    assert_eq!(report.files_modified, 1);
    assert_eq!(fs::read_to_string(&generated_file).map_err(|e| e.to_string())?, generated_legacy_src);

    let direct_gen_report = run_docstring_injection(&DocstringInjectionOptions {
        target_path: gen_dir,
        dry_run: false,
        force: false,
        schema_model: None,
    })
    .map_err(|(c, m)| format!("gen dir scan ({c:?}): {m}"))?;
    assert_eq!(direct_gen_report.files_scanned, 0);
    assert_eq!(direct_gen_report.files_modified, 0);
    fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(())
}

/// Runs all Tier A/B/C docstring synthesis, compaction, and generated skip isolation checks.
/// Safety & Traceability: Realises: [REQ-FEAT-42-DOCSTRING-INJECTOR-INTEGRATION-TESTS]
/// Contract: Executes `verify_undocumented_tiered_synthesis` and compaction/skip checks.
/// Bounds & Faults: Cleans up temporary directories before returning.
pub fn run_tiered_v2_synthesis_and_compaction_checks() -> Result<(), String> {
    verify_undocumented_tiered_synthesis()?;
    let legacy_mixed_src = build_legacy_mixed_src();
    verify_tier_bc_compaction(&legacy_mixed_src)?;
    verify_generated_skip_isolation(&legacy_mixed_src)
}
