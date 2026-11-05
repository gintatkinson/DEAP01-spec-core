//! Integration tests for FEAT-42 Source-Code AST Ingestion (`ingest-sysml --source <DIR> --format code --language <auto|dart|rust>`).
//!
//! Realises: [REQ-SYSML-INGEST-CODE-TESTS]

use compile_sysml::{parse_sysml, to_sysml, SemanticValidator};
use deap_core::source_ast::SourceLanguage;
use ingest_sysml::translators::code::CodeTranslator;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// RAII temporary workspace directory for isolated `ingest-sysml` integration tests. Realises: [REQ-SYSML-INGEST-CODE-TESTS/ScopedTempDir]
struct ScopedTempDir {
    root: PathBuf,
}

impl ScopedTempDir {
    /// Creates a new isolated temporary directory labeled with `label`. Realises: [REQ-SYSML-INGEST-CODE-TESTS/ScopedTempDir_new]
    fn new(label: &str) -> Result<Self, String> {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        let root = std::env::temp_dir().join(format!("deap_code_ingest_{label}_{}_{nanos}", std::process::id()));
        fs::create_dir_all(&root).map_err(|e| format!("Failed to create temp dir '{}': {e}", root.display()))?;
        Ok(Self { root })
    }
}

impl Drop for ScopedTempDir {
    /// Removes the temporary test directory when `ScopedTempDir` goes out of scope. Realises: [REQ-SYSML-INGEST-CODE-TESTS/ScopedTempDir_drop]
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Verifies AC-01 Dart `StatefulWidget` AST extraction into a canonical `PackageDef`. Realises: [REQ-SYSML-INGEST-CODE-TESTS/ac01_dart_widget]
#[test]
fn test_ac01_dart_stateful_widget_extraction_into_canonical_packagedef_ast() -> Result<(), String> {
    let ws = ScopedTempDir::new("ac01_dart_widget")?;
    let lib_dir = ws.root.join("lib");
    fs::create_dir_all(&lib_dir).map_err(|e| format!("create lib/: {e}"))?;

    let dart_source = r#"import 'package:flutter/material.dart';

/// Primary telemetry control widget.
/// Realises: [REQ-UI-01]
class FlightControlWidget extends StatefulWidget {
  final String vehicleCallsign;
  final int targetAltitudeMeters;
  final VoidCallback onEmergencyStop;
  final ValueChanged<int> onAltitudeChanged;

  const FlightControlWidget({
    super.key,
    required this.vehicleCallsign,
    required this.targetAltitudeMeters,
    required this.onEmergencyStop,
    required this.onAltitudeChanged,
  });

  @override
  State<FlightControlWidget> createState() => _FlightControlWidgetState();
}

class _FlightControlWidgetState extends State<FlightControlWidget> {
  bool isArmed = false;

  void setArmedState(bool armed) {
    assert(armed == true || armed == false);
    isArmed = armed;
  }
}
"#;
    fs::write(lib_dir.join("flight_control_widget.dart"), dart_source).map_err(|e| format!("write dart source file: {e}"))?;

    let pkg = CodeTranslator::with_language(SourceLanguage::Dart).translate_directory(&lib_dir, "FlightUiPackage")?;
    let widget_part = pkg
        .part_defs
        .iter()
        .find(|p| p.name == "FlightControlWidget")
        .ok_or_else(|| "Expected PartDef for FlightControlWidget".to_string())?;

    if !widget_part.attributes.iter().any(|a| a.name == "vehicleCallsign") {
        return Err("Expected AttributeDef 'vehicleCallsign' on FlightControlWidget".to_string());
    }
    if !widget_part.attributes.iter().any(|a| a.name == "targetAltitudeMeters") {
        return Err("Expected AttributeDef 'targetAltitudeMeters' on FlightControlWidget".to_string());
    }
    if !widget_part.ports.iter().any(|p| p.name == "onEmergencyStop") {
        return Err("Expected PortDef 'onEmergencyStop' on FlightControlWidget".to_string());
    }
    if !widget_part.ports.iter().any(|p| p.name == "onAltitudeChanged") {
        return Err("Expected PortDef 'onAltitudeChanged' on FlightControlWidget".to_string());
    }
    if !widget_part.satisfied_requirements.contains(&"REQ-UI-01".to_string()) {
        return Err("Expected REQ-UI-01 in satisfied_requirements".to_string());
    }

    let sysml_text = to_sysml(&pkg);
    if !sysml_text.contains("satisfy requirement REQ-UI-01;") {
        return Err(format!("Expected 'satisfy requirement REQ-UI-01;' in canonical SysML v2 output:\n{sysml_text}"));
    }
    let reparsed = parse_sysml(&sysml_text).map_err(|e| format!("re-parse serialized SysML v2 model: {e}"))?;
    let mut validator = SemanticValidator::new(&reparsed);
    validator.validate();
    if validator.has_errors() {
        return Err(format!("Expected zero semantic validation errors on lowered Dart model, got: {:?}", validator.errors()));
    }
    Ok(())
}

/// Verifies AC-02 Rust `struct` and `impl` block AST ingestion into a canonical `PackageDef`. Realises: [REQ-SYSML-INGEST-CODE-TESTS/ac02_rust_struct_impl]
#[test]
fn test_ac02_rust_struct_and_impl_block_ast_ingestion() -> Result<(), String> {
    let ws = ScopedTempDir::new("ac02_rust_struct_impl")?;
    let src_dir = ws.root.join("crates").join("my-crate").join("src");
    fs::create_dir_all(&src_dir).map_err(|e| format!("create crates/my-crate/src: {e}"))?;

    let rust_source = r#"/// Bounded ring buffer for telemetry frames.
/// Realises: [REQ-BUF-01]
pub struct ComponentBuffer {
    pub capacity: usize,
    pub write_index: usize,
    pub frame_tag: String,
}

impl ComponentBuffer {
    pub fn enqueue_frame(&mut self, payload_len: usize, tag: &str) -> Result<usize, String> {
        assert!(self.write_index <= self.capacity);
        let _unused = (payload_len, tag);
        Ok(self.write_index)
    }

    pub fn remaining_capacity(&self) -> usize {
        self.capacity.saturating_sub(self.write_index)
    }
}
"#;
    fs::write(src_dir.join("lib.rs"), rust_source).map_err(|e| format!("write lib.rs: {e}"))?;

    let pkg = CodeTranslator::with_language(SourceLanguage::Rust).translate_directory(&src_dir, "MyCratePkg")?;
    let buf_part = pkg
        .part_defs
        .iter()
        .find(|p| p.name == "ComponentBuffer")
        .ok_or_else(|| "Expected PartDef for ComponentBuffer".to_string())?;

    if buf_part.attributes.len() != 3
        || !buf_part.attributes.iter().any(|a| a.name == "capacity")
        || !buf_part.attributes.iter().any(|a| a.name == "write_index")
        || !buf_part.attributes.iter().any(|a| a.name == "frame_tag")
    {
        return Err("Expected capacity, write_index, frame_tag attributes on ComponentBuffer".to_string());
    }

    let enqueue_act = buf_part
        .actions
        .iter()
        .find(|a| a.name == "enqueue_frame")
        .ok_or_else(|| "Expected ActionDef 'enqueue_frame' in ComponentBuffer".to_string())?;
    if enqueue_act.in_params.len() != 2
        || enqueue_act.in_params.first().map(|p| p.name.as_str()) != Some("payload_len")
        || enqueue_act.in_params.get(1).map(|p| p.name.as_str()) != Some("tag")
        || enqueue_act.out_params.first().map(|p| p.name.as_str()) != Some("result")
    {
        return Err("Unexpected enqueue_frame parameters".to_string());
    }

    let rem_act = buf_part
        .actions
        .iter()
        .find(|a| a.name == "remaining_capacity")
        .ok_or_else(|| "Expected ActionDef 'remaining_capacity' in ComponentBuffer".to_string())?;
    if !rem_act.in_params.is_empty() || rem_act.out_params.len() != 1 {
        return Err("Expected 0 in_params and 1 out_param on remaining_capacity".to_string());
    }
    if !buf_part.constraints.iter().any(|c| c.is_assertion && c.expression.contains("self.write_index <= self.capacity")) {
        return Err("Expected assert! lowered to ConstraintDef on ComponentBuffer".to_string());
    }

    let sysml_text = to_sysml(&pkg);
    let reparsed = parse_sysml(&sysml_text).map_err(|e| format!("re-parse serialized Rust SysML v2 model: {e}"))?;
    let mut validator = SemanticValidator::new(&reparsed);
    validator.validate();
    if validator.has_errors() {
        return Err(format!("Expected zero semantic validation errors on lowered Rust model, got: {:?}", validator.errors()));
    }
    Ok(())
}

/// Verifies AC-05 zero-regex lexer error recovery halts closed leaving outputs unmodified. Realises: [REQ-SYSML-INGEST-CODE-TESTS/ac05_syntax_error]
#[test]
fn test_ac05_zero_regex_lexer_error_recovery_halts_closed_leaving_outputs_unmodified() -> Result<(), String> {
    let ws = ScopedTempDir::new("ac05_syntax_error")?;
    let src_dir = ws.root.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| format!("create src/: {e}"))?;
    fs::write(src_dir.join("broken.rs"), "pub struct UnclosedComponent {\n    pub id: u64,\n").map_err(|e| format!("write broken.rs: {e}"))?;

    let out_sysml = ws.root.join("schema.sysml");
    let sentinel = "package ExistingBaseline {}\n";
    fs::write(&out_sysml, sentinel).map_err(|e| format!("write sentinel schema.sysml: {e}"))?;

    let err_msg = match CodeTranslator::with_language(SourceLanguage::Rust).translate_directory(&src_dir, "BrokenPkg") {
        Ok(_) => return Err("Expected CodeTranslator to fail closed on unclosed brace".to_string()),
        Err(e) => e,
    };
    if !err_msg.contains("syntax error") || !err_msg.contains("line 1, column 30") {
        return Err(format!("Expected structured line/col syntax error diagnostic, got: {err_msg}"));
    }

    let after_content = fs::read_to_string(&out_sysml).map_err(|e| format!("read sentinel schema.sysml: {e}"))?;
    if after_content != sentinel {
        return Err("Existing output file must remain untouched on ingestion failure".to_string());
    }
    Ok(())
}

/// Verifies the `ingest-sysml` CLI exit code `0 / 1 / 2` contract end-to-end. Realises: [REQ-SYSML-INGEST-CLI-EXIT-CODES]
#[test]
fn test_ingest_sysml_cli_exit_codes_0_1_and_2() -> Result<(), String> {
    let ws = ScopedTempDir::new("cli_exit_codes")?;
    let bin = env!("CARGO_BIN_EXE_ingest-sysml");

    let valid_dir = ws.root.join("valid_src");
    fs::create_dir_all(&valid_dir).map_err(|e| format!("create valid_src: {e}"))?;
    fs::write(
        valid_dir.join("lib.rs"),
        "/// Telemetry ring buffer component.\n/// Realises: [REQ-BUF-01]\npub struct ComponentBuffer {\n    pub capacity: usize,\n}\n",
    )
    .map_err(|e| format!("write valid lib.rs: {e}"))?;

    let output_0 = Command::new(bin)
        .args([
            "--source", &valid_dir.to_string_lossy(),
            "--out", &ws.root.join("out_ok.sysml").to_string_lossy(),
            "--digest", &ws.root.join("digest_ok.json").to_string_lossy(),
        ])
        .output()
        .map_err(|e| format!("spawn ingest-sysml (case 0): {e}"))?;
    if output_0.status.code() != Some(0) {
        return Err(format!("Expected exit code 0, got {:?}", output_0.status.code()));
    }

    let broken_dir = ws.root.join("broken_src");
    fs::create_dir_all(&broken_dir).map_err(|e| format!("create broken_src: {e}"))?;
    fs::write(broken_dir.join("broken.rs"), "pub struct UnclosedComponent {\n    pub id: u64,\n").map_err(|e| format!("write broken.rs: {e}"))?;
    let output_1 = Command::new(bin)
        .args([
            "--source", &broken_dir.to_string_lossy(),
            "--out", &ws.root.join("out_err1.sysml").to_string_lossy(),
            "--digest", &ws.root.join("digest_err1.json").to_string_lossy(),
        ])
        .output()
        .map_err(|e| format!("spawn ingest-sysml (case 1): {e}"))?;
    if output_1.status.code() != Some(1) {
        return Err(format!("Expected exit code 1, got {:?}", output_1.status.code()));
    }

    let output_2a = Command::new(bin)
        .args([
            "--source", &valid_dir.to_string_lossy(),
            "--negative-invariants", "ComponentBuffer",
            "--out", &ws.root.join("out_err2a.sysml").to_string_lossy(),
            "--digest", &ws.root.join("digest_err2a.json").to_string_lossy(),
        ])
        .output()
        .map_err(|e| format!("spawn ingest-sysml (case 2a): {e}"))?;
    if output_2a.status.code() != Some(2) {
        return Err(format!("Expected exit code 2, got {:?}", output_2a.status.code()));
    }

    let output_2b = Command::new(bin)
        .args([
            "--source", &valid_dir.to_string_lossy(),
            "--allowed-parts", "NonExistentPart",
            "--out", &ws.root.join("out_err2b.sysml").to_string_lossy(),
            "--digest", &ws.root.join("digest_err2b.json").to_string_lossy(),
        ])
        .output()
        .map_err(|e| format!("spawn ingest-sysml (case 2b): {e}"))?;
    if output_2b.status.code() != Some(2) {
        return Err(format!("Expected exit code 2, got {:?}", output_2b.status.code()));
    }
    Ok(())
}
