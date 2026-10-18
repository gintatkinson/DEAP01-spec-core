//! Integration tests for FEAT-42 Source-Code AST Ingestion (`ingest-sysml --source <DIR> --format code --language <auto|dart|rust>`).

use compile_sysml::{parse_sysml, to_sysml, SemanticValidator};
use deap_core::source_ast::SourceLanguage;
use ingest_sysml::translators::code::CodeTranslator;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

struct ScopedTempDir {
    root: PathBuf,
}

impl ScopedTempDir {
    fn new(label: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!(
            "deap_code_ingest_{}_{}_{}",
            label,
            std::process::id(),
            nanos
        ));
        fs::create_dir_all(&root).expect("create temp dir");
        Self { root }
    }
}

impl Drop for ScopedTempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn test_ac01_dart_stateful_widget_extraction_into_canonical_packagedef_ast() {
    let ws = ScopedTempDir::new("ac01_dart_widget");
    let lib_dir = ws.root.join("lib");
    fs::create_dir_all(&lib_dir).expect("create lib/");

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
    fs::write(lib_dir.join("flight_control_widget.dart"), dart_source)
        .expect("write dart source file");

    let translator = CodeTranslator::with_language(SourceLanguage::Dart);
    let pkg = translator
        .translate_directory(&lib_dir, "FlightUiPackage")
        .expect("translate Dart directory into PackageDef");

    let widget_part = pkg
        .part_defs
        .iter()
        .find(|p| p.name == "FlightControlWidget")
        .expect("Expected PartDef for FlightControlWidget");

    // Verify fields mapped to AttributeDef
    assert!(
        widget_part
            .attributes
            .iter()
            .any(|a| a.name == "vehicleCallsign"),
        "Expected AttributeDef 'vehicleCallsign' on FlightControlWidget"
    );
    assert!(
        widget_part
            .attributes
            .iter()
            .any(|a| a.name == "targetAltitudeMeters"),
        "Expected AttributeDef 'targetAltitudeMeters' on FlightControlWidget"
    );

    // Verify callbacks mapped to PortDef
    assert!(
        widget_part
            .ports
            .iter()
            .any(|p| p.name == "onEmergencyStop"),
        "Expected PortDef 'onEmergencyStop' on FlightControlWidget"
    );
    assert!(
        widget_part
            .ports
            .iter()
            .any(|p| p.name == "onAltitudeChanged"),
        "Expected PortDef 'onAltitudeChanged' on FlightControlWidget"
    );

    // Verify requirement realization binding
    assert!(
        widget_part
            .satisfied_requirements
            .contains(&"REQ-UI-01".to_string()),
        "Expected REQ-UI-01 in satisfied_requirements"
    );

    // Verify canonical SysML v2 serialization and round-trip parsing
    let sysml_text = to_sysml(&pkg);
    assert!(
        sysml_text.contains("satisfy requirement REQ-UI-01;"),
        "Expected 'satisfy requirement REQ-UI-01;' in canonical SysML v2 output:\n{}",
        sysml_text
    );
    let reparsed = parse_sysml(&sysml_text).expect("re-parse serialized SysML v2 model");
    let mut validator = SemanticValidator::new(&reparsed);
    validator.validate();
    assert!(
        !validator.has_errors(),
        "Expected zero semantic validation errors on lowered Dart model, got: {:?}",
        validator.errors()
    );
}

#[test]
fn test_ac02_rust_struct_and_impl_block_ast_ingestion() {
    let ws = ScopedTempDir::new("ac02_rust_struct_impl");
    let src_dir = ws.root.join("crates").join("my-crate").join("src");
    fs::create_dir_all(&src_dir).expect("create crates/my-crate/src");

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
    fs::write(src_dir.join("lib.rs"), rust_source).expect("write lib.rs");

    let translator = CodeTranslator::with_language(SourceLanguage::Rust);
    let pkg = translator
        .translate_directory(&src_dir, "MyCratePkg")
        .expect("translate Rust directory into PackageDef");

    let buf_part = pkg
        .part_defs
        .iter()
        .find(|p| p.name == "ComponentBuffer")
        .expect("Expected PartDef for ComponentBuffer");

    // Verify struct fields lowered to AttributeDef
    assert_eq!(buf_part.attributes.len(), 3);
    assert!(buf_part.attributes.iter().any(|a| a.name == "capacity"));
    assert!(buf_part.attributes.iter().any(|a| a.name == "write_index"));
    assert!(buf_part.attributes.iter().any(|a| a.name == "frame_tag"));

    // Verify public methods lowered to typed ActionDef with formal in and out parameter signatures
    let enqueue_act = buf_part
        .actions
        .iter()
        .find(|a| a.name == "enqueue_frame")
        .expect("Expected ActionDef 'enqueue_frame' in ComponentBuffer");
    assert_eq!(enqueue_act.in_params.len(), 2);
    assert_eq!(enqueue_act.in_params[0].name, "payload_len");
    assert_eq!(enqueue_act.in_params[1].name, "tag");
    assert_eq!(enqueue_act.out_params.len(), 1);
    assert_eq!(enqueue_act.out_params[0].name, "result");

    let rem_act = buf_part
        .actions
        .iter()
        .find(|a| a.name == "remaining_capacity")
        .expect("Expected ActionDef 'remaining_capacity' in ComponentBuffer");
    assert!(rem_act.in_params.is_empty());
    assert_eq!(rem_act.out_params.len(), 1);

    // Verify assert! lowered to ConstraintDef assertion
    assert!(
        buf_part
            .constraints
            .iter()
            .any(|c| c.is_assertion && c.expression.contains("self.write_index <= self.capacity")),
        "Expected assert! lowered to ConstraintDef on ComponentBuffer"
    );

    let sysml_text = to_sysml(&pkg);
    let reparsed = parse_sysml(&sysml_text).expect("re-parse serialized Rust SysML v2 model");
    let mut validator = SemanticValidator::new(&reparsed);
    validator.validate();
    assert!(
        !validator.has_errors(),
        "Expected zero semantic validation errors on lowered Rust model, got: {:?}",
        validator.errors()
    );
}

#[test]
fn test_ac05_zero_regex_lexer_error_recovery_halts_closed_leaving_outputs_unmodified() {
    let ws = ScopedTempDir::new("ac05_syntax_error");
    let src_dir = ws.root.join("src");
    fs::create_dir_all(&src_dir).expect("create src/");

    // Write malformed Rust file with unclosed brace
    let broken_rust = "pub struct UnclosedComponent {\n    pub id: u64,\n";
    fs::write(src_dir.join("broken.rs"), broken_rust).expect("write broken.rs");

    let out_sysml = ws.root.join("schema.sysml");
    let sentinel = "package ExistingBaseline {}\n";
    fs::write(&out_sysml, sentinel).expect("write sentinel schema.sysml");

    let translator = CodeTranslator::with_language(SourceLanguage::Rust);
    let res = translator.translate_directory(&src_dir, "BrokenPkg");
    assert!(
        res.is_err(),
        "Expected CodeTranslator to fail closed on unclosed brace"
    );
    let err_msg = res.unwrap_err();
    assert!(
        err_msg.contains("syntax error") && err_msg.contains("line 1, column 30"),
        "Expected structured line/col syntax error diagnostic, got: {}",
        err_msg
    );

    // Verify existing output file was left unmodified
    let after_content = fs::read_to_string(&out_sysml).expect("read sentinel schema.sysml");
    assert_eq!(
        after_content, sentinel,
        "Existing output file must remain untouched on ingestion failure"
    );
}
