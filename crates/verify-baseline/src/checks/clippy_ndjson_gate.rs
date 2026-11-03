#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! `BP-01` workspace manifest `"forbid"` gate and `cargo_metadata::Message` NDJSON diagnostic parser.
//! Realises: [REQ-0601], [REQ-0608]

pub(crate) use super::cargo_policy_gate::scan_cargo_toml_file;
use cargo_metadata::diagnostic::DiagnosticLevel;
use cargo_metadata::Message;
use std::fs;
use std::io::Cursor;
use std::path::Path;
use toml_edit::{DocumentMut, Item};

/// Mandatory Clippy lints that must be configured as `"forbid"` (`E0453`) in `[workspace.lints.clippy]` (`[REQ-0601]`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RequiredClippyForbidLint {
    /// Forbids `undocumented_unsafe_blocks` (`[REQ-0601]`).
    UndocumentedUnsafeBlocks,
    /// Forbids `multiple_unsafe_ops_per_block` (`[REQ-0601]`).
    MultipleUnsafeOpsPerBlock,
    /// Forbids `disallowed_types` (`[REQ-0601]`).
    DisallowedTypes,
    /// Forbids `unwrap_used` (`[REQ-0601]`).
    UnwrapUsed,
    /// Forbids `expect_used` (`[REQ-0601]`).
    ExpectUsed,
    /// Forbids `panic` (`[REQ-0601]`).
    Panic,
    /// Forbids `indexing_slicing` (`[REQ-0601]`).
    IndexingSlicing,
    /// Forbids `type_complexity` (`[REQ-0601]`).
    TypeComplexity,
    /// Forbids `dbg_macro` (`[REQ-0601]`).
    DbgMacro,
    /// Forbids `todo` (`[REQ-0601]`).
    Todo,
    /// Forbids `unimplemented` (`[REQ-0601]`).
    Unimplemented,
    /// Forbids `unreachable` (`[REQ-0601]`).
    Unreachable,
    /// Forbids `arithmetic_side_effects` (`[REQ-0601]`).
    ArithmeticSideEffects,
    /// Forbids `cast_possible_truncation` (`[REQ-0601]`).
    CastPossibleTruncation,
    /// Forbids `cast_sign_loss` (`[REQ-0601]`).
    CastSignLoss,
    /// Forbids `fn_params_excessive_bools` (`[REQ-0601]`).
    FnParamsExcessiveBools,
}

impl RequiredClippyForbidLint {
    /// Complete list of mandatory `[workspace.lints.clippy]` `"forbid"` lints (`[REQ-0601]`).
    pub const ALL: &'static [Self] = &[
        Self::UndocumentedUnsafeBlocks,
        Self::MultipleUnsafeOpsPerBlock,
        Self::DisallowedTypes,
        Self::UnwrapUsed,
        Self::ExpectUsed,
        Self::Panic,
        Self::IndexingSlicing,
        Self::TypeComplexity,
        Self::DbgMacro,
        Self::Todo,
        Self::Unimplemented,
        Self::Unreachable,
        Self::ArithmeticSideEffects,
        Self::CastPossibleTruncation,
        Self::CastSignLoss,
        Self::FnParamsExcessiveBools,
    ];

    /// Returns the canonical `[workspace.lints.clippy]` key for this lint (`[REQ-0601]`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UndocumentedUnsafeBlocks => "undocumented_unsafe_blocks",
            Self::MultipleUnsafeOpsPerBlock => "multiple_unsafe_ops_per_block",
            Self::DisallowedTypes => "disallowed_types",
            Self::UnwrapUsed => "unwrap_used",
            Self::ExpectUsed => "expect_used",
            Self::Panic => "panic",
            Self::IndexingSlicing => "indexing_slicing",
            Self::TypeComplexity => "type_complexity",
            Self::DbgMacro => "dbg_macro",
            Self::Todo => "todo",
            Self::Unimplemented => "unimplemented",
            Self::Unreachable => "unreachable",
            Self::ArithmeticSideEffects => "arithmetic_side_effects",
            Self::CastPossibleTruncation => "cast_possible_truncation",
            Self::CastSignLoss => "cast_sign_loss",
            Self::FnParamsExcessiveBools => "fn_params_excessive_bools",
        }
    }
}

/// Structured diagnostic extracted from a `cargo_metadata::Message` NDJSON stream (`[REQ-0601]`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClippyNdjsonDiagnostic {
    /// Optional lint or compiler code such as `clippy::unwrap_used` or `E0453` (`[REQ-0601]`).
    pub code: Option<String>,
    /// Diagnostic level (`"error"` or `"warning"`) (`[REQ-0601]`).
    pub level: String,
    /// Primary source span file name when present (`[REQ-0601]`).
    pub file: String,
    /// 1-based primary span line number when present (`[REQ-0601]`).
    pub line: usize,
    /// Rendered human-readable diagnostic summary (`[REQ-0601]`).
    pub rendered: String,
}

/// Verifies `[workspace.lints]` `"forbid"` invariants and crate `[lints] workspace = true` inheritance (`[REQ-0601]`).
pub struct WorkspaceManifestForbidGate;

impl WorkspaceManifestForbidGate {
    /// Audits `<workspace_root>/Cargo.toml` and `<workspace_root>/crates/*/Cargo.toml` (`[REQ-0601]`).
    ///
    /// # Errors
    /// Returns human-readable violation strings when any lint is not `"forbid"` (`[REQ-0601]`).
    pub fn verify(workspace_root: &Path) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let root_manifest = workspace_root.join("Cargo.toml");
        if !root_manifest.exists() {
            errors.push(format!("{}: root Cargo.toml is missing", root_manifest.display()));
            return Err(errors);
        }
        verify_root_manifest(&root_manifest, workspace_root, &mut errors);
        let crates_dir = workspace_root.join("crates");
        if crates_dir.is_dir() {
            verify_crate_manifests(&crates_dir, workspace_root, &mut errors);
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

/// Verifies root `Cargo.toml` `[workspace.lints.rust]` and `[workspace.lints.clippy]` `"forbid"` entries (`[REQ-0601]`).
fn verify_root_manifest(root_manifest: &Path, workspace_root: &Path, errors: &mut Vec<String>) {
    scan_cargo_toml_file(root_manifest, workspace_root, errors);
    let Ok(content) = fs::read_to_string(root_manifest) else {
        errors.push(String::from("Cargo.toml: failed to read root manifest"));
        return;
    };
    let Ok(doc) = content.parse::<DocumentMut>() else {
        errors.push(String::from("Cargo.toml: invalid TOML syntax"));
        return;
    };
    let rust_unsafe = doc
        .get("workspace")
        .and_then(|w| w.get("lints"))
        .and_then(|l| l.get("rust"))
        .and_then(|r| r.get("unsafe_code"));
    if !is_forbid_lint_entry(rust_unsafe) {
        errors.push(String::from("Cargo.toml: `[workspace.lints.rust] unsafe_code` must be \"forbid\""));
    }
    let clippy_table = doc.get("workspace").and_then(|w| w.get("lints")).and_then(|l| l.get("clippy"));
    for req in RequiredClippyForbidLint::ALL {
        if !is_forbid_lint_entry(clippy_table.and_then(|c| c.get(req.as_str()))) {
            errors.push(format!(
                "Cargo.toml: `[workspace.lints.clippy] {}` must be set to \"forbid\" (E0453)",
                req.as_str()
            ));
        }
    }
    if let Some(table) = clippy_table.and_then(Item::as_table_like) {
        for (key, item) in table.iter() {
            if !is_forbid_lint_entry(Some(item)) {
                errors.push(format!("Cargo.toml: `[workspace.lints.clippy] {key}` must be \"forbid\""));
            }
        }
    }
}

/// Checks that every `crates/*/Cargo.toml` inherits `[lints] workspace = true` and has zero banned deps (`[REQ-0601]`).
fn verify_crate_manifests(crates_dir: &Path, workspace_root: &Path, errors: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(crates_dir) else {
        errors.push(String::from("crates/: failed to read directory"));
        return;
    };
    let mut dirs = Vec::new();
    for entry_res in entries {
        let Ok(entry) = entry_res else {
            errors.push(String::from("crates/: failed to read directory entry"));
            return;
        };
        dirs.push(entry.path());
    }
    dirs.sort();
    for dir in dirs {
        let manifest = dir.join("Cargo.toml");
        if !dir.is_dir() || !manifest.exists() {
            continue;
        }
        let rel = manifest.strip_prefix(workspace_root).unwrap_or(&manifest).display().to_string();
        scan_cargo_toml_file(&manifest, workspace_root, errors);
        let Ok(content) = fs::read_to_string(&manifest) else {
            errors.push(format!("{rel}: failed to read crate manifest"));
            continue;
        };
        let Ok(doc) = content.parse::<DocumentMut>() else {
            errors.push(format!("{rel}: invalid TOML syntax"));
            continue;
        };
        if doc.get("lints").and_then(|l| l.get("workspace")).and_then(Item::as_bool) != Some(true) {
            errors.push(format!("{rel}: missing [lints] workspace = true inheritance"));
        }
    }
}

/// Returns `true` iff `item` is `"forbid"` or `{ level = "forbid", ... }` (`[REQ-0601]`).
fn is_forbid_lint_entry(item: Option<&Item>) -> bool {
    let Some(item) = item else { return false };
    item.as_str() == Some("forbid")
        || item.as_table_like().and_then(|t| t.get("level")).and_then(Item::as_str) == Some("forbid")
}

/// Parses a `cargo clippy --message-format=json` NDJSON stream via `cargo_metadata::Message` (`[REQ-0601]`).
///
/// # Errors
/// Fails closed if any `CompilerMessage` is an error/warning, `BuildFinished` is false, or JSON is invalid (`[REQ-0601]`).
pub fn verify_clippy_ndjson_stream(ndjson: &str) -> Result<Vec<ClippyNdjsonDiagnostic>, Vec<String>> {
    let mut diagnostics = Vec::new();
    let mut errors = Vec::new();
    let mut build_failed = false;

    for message in Message::parse_stream(Cursor::new(ndjson)) {
        match message {
            Ok(Message::CompilerMessage(msg)) => {
                let diag = &msg.message;
                if matches!(diag.level, DiagnosticLevel::Error | DiagnosticLevel::Warning | DiagnosticLevel::Ice) {
                    let level = if matches!(diag.level, DiagnosticLevel::Warning) {
                        String::from("warning")
                    } else {
                        String::from("error")
                    };
                    let code = diag.code.as_ref().map(|c| c.code.clone());
                    let primary = diag.spans.iter().find(|s| s.is_primary).or_else(|| diag.spans.first());
                    let file = primary.map_or_else(|| String::from("<unknown>"), |s| s.file_name.clone());
                    let line = primary.map_or(0, |s| s.line_start);
                    let rendered = diag.rendered.clone().unwrap_or_else(|| diag.message.clone());
                    let code_label = code.as_deref().unwrap_or("diagnostic");
                    errors.push(format!("{file}:{line}: [{level}] {code_label}: {}", diag.message));
                    diagnostics.push(ClippyNdjsonDiagnostic { code, level, file, line, rendered });
                }
            }
            Ok(Message::BuildFinished(fin)) if !fin.success => build_failed = true,
            Ok(_) => {}
            Err(e) => errors.push(format!("invalid cargo_metadata::Message NDJSON record: {e}")),
        }
    }
    if build_failed && errors.is_empty() {
        errors.push(String::from("cargo clippy build-finished reported success=false"));
    }
    if errors.is_empty() { Ok(diagnostics) } else { Err(errors) }
}

#[cfg(test)]
mod tests {
    use super::{verify_clippy_ndjson_stream, RequiredClippyForbidLint, WorkspaceManifestForbidGate};
    use std::path::Path;

    /// Verifies `RequiredClippyForbidLint::ALL` contains all 16 `BP-01` lints and workspace `Cargo.toml` enforces `"forbid"` (`[REQ-0601]`).
    #[test]
    fn workspace_manifest_enforces_forbid_on_all_bp01_lints() -> Result<(), String> {
        for required in [
            "unreachable",
            "arithmetic_side_effects",
            "cast_possible_truncation",
            "cast_sign_loss",
            "fn_params_excessive_bools",
        ] {
            if !RequiredClippyForbidLint::ALL.iter().any(|l| l.as_str() == required) {
                return Err(format!("missing required forbid lint {required}"));
            }
        }
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| String::from("missing workspace root"))?;
        WorkspaceManifestForbidGate::verify(root)
            .map_err(|errs| format!("workspace Cargo.toml failed forbid gate: {errs:#?}"))
    }

    /// Verifies `verify_clippy_ndjson_stream` parses `cargo_metadata::Message` and fails closed on diagnostics (`[REQ-0601]`).
    #[test]
    fn clippy_ndjson_gate_parses_stream_and_fails_closed_on_diagnostics() -> Result<(), String> {
        let ok_stream = r#"{"reason":"build-finished","success":true}"#;
        let ok_diags = verify_clippy_ndjson_stream(ok_stream)
            .map_err(|e| format!("expected clean NDJSON stream to pass, got {e:?}"))?;
        if !ok_diags.is_empty() {
            return Err(format!("expected zero diagnostics on clean stream, got {ok_diags:?}"));
        }
        let err_stream = concat!(
            r#"{"reason":"compiler-message","package_id":"path+file:///ws/crates/demo#0.1.0","manifest_path":"/ws/crates/demo/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"demo","src_path":"/ws/crates/demo/src/lib.rs","edition":"2021","doc":true,"doctest":false,"test":true},"message":{"rendered":"error: used unwrap()\n","children":[],"code":{"code":"clippy::unwrap_used","explanation":null},"level":"error","message":"used unwrap()","spans":[{"byte_end":10,"byte_start":0,"column_end":10,"column_start":1,"expansion":null,"file_name":"crates/demo/src/lib.rs","is_primary":true,"label":null,"line_end":4,"line_start":4,"suggested_replacement":null,"suggestion_applicability":null,"text":[]}]}}"#,
            "\n",
            r#"{"reason":"build-finished","success":false}"#
        );
        match verify_clippy_ndjson_stream(err_stream) {
            Err(errs) if errs.iter().any(|m| m.contains("clippy::unwrap_used") && m.contains("crates/demo/src/lib.rs:4")) => Ok(()),
            other => Err(format!("expected fail-closed clippy::unwrap_used diagnostic, got {other:?}")),
        }
    }
}
