#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! [DOC-TIER: A] Layer 1 CLI Presentation & Algebraic Mode Resolution for `compile-sysml`.
//! ## 1. Safety Intent & Regulatory Scope
//! Safety & Traceability: Realises: [REQ-COMPILER-SYSML-CLI], [REQ-0608], [DO-330-TQL-4-CLI]
//! ## 2. Invariants & Mathematical Properties
//! Contract: Provides the declarative CLI contract (`Cli`), diagnostic format (`MessageFormat`), and mutually exclusive operational mode sum type (`CommandMode`).
//! ## 3. Preconditions
//! ## 4. Postconditions
//! ## 5. Algorithmic Complexity Bounds
//! ## 6. Failure Modes & Fault Tolerance
//! Bounds & Faults: O(A) time and space in CLI argument token count `A`; rejects conflicting mode flags with `Err(String)`.
//! ## 7. Realization & Traceability

use crate::cli_command_builder::{build_cli_command, cli_from_matches};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Diagnostic output format for `compile-sysml` (`--message-format <human|json>`).
/// ## 1. Safety Intent & Regulatory Scope
/// Safety & Traceability: Realises: [REQ-0608], [SC-01], [FLS: fls_compile_cst_cli]
/// ## 2. Invariants & Mathematical Properties
/// Contract: Selects between human-readable (`Human`) and single-line NDJSON (`Json`) output.
/// ## 3. Preconditions
/// ## 4. Postconditions
/// ## 5. Algorithmic Complexity Bounds
/// ## 6. Failure Modes & Fault Tolerance
/// Bounds & Faults: O(1) time and space; total value semantics.
/// ## 7. Realization & Traceability
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MessageFormat {
    #[default]
    Human,
    Json,
}

impl MessageFormat {
    /// Parses `"json"` (case-insensitive) as `MessageFormat::Json` and defaults other tokens to `Human`.
    /// Safety & Traceability: Realises: [REQ-0608]
    /// Contract: Maps `"json"` to `Self::Json` and `"human"` (or fallback) to `Self::Human`.
    /// Bounds & Faults: O(1) time and space; infallible.
    #[must_use]
    pub fn from_flag_lossless(raw: &str) -> Self {
        if raw.trim().eq_ignore_ascii_case("json") {
            Self::Json
        } else {
            Self::Human
        }
    }
}

/// Command-line argument definition for the `compile-sysml` compiler and synchronization engine.
/// Safety & Traceability: Realises: [REQ-COMPILER-SYSML-CLI-ARGS], [REQ-0608], [DO-330-TQL-4-CLI]
/// Contract: Binds Pipeline 0 compilation, STPA, sync, docstring synthesis, MBD/Flutter/Rust codegen, and Track-1 CST refactoring flags.
/// Bounds & Faults: O(A) time and space in argument token count `A`; validated via `Cli::into_mode`.
#[derive(Debug, Clone)]
pub struct Cli {
    pub file: Option<PathBuf>,
    pub schema: Option<PathBuf>,
    pub compile: bool,
    pub stpa_transpile: bool,
    pub stpa: Option<PathBuf>,
    pub reverse_sync: bool,
    pub forward_sync: bool,
    pub inject_safety_docstrings: Option<PathBuf>,
    pub extract_source: Option<PathBuf>,
    pub export_mbd: Option<PathBuf>,
    pub slice_package: Option<String>,
    pub emit_flutter_contracts: Option<PathBuf>,
    pub emit_modular_rust: Option<PathBuf>,
    pub list_rust_items: Option<PathBuf>,
    pub extract_rust_module: Option<PathBuf>,
    pub items: Option<String>,
    pub dest: Option<PathBuf>,
    pub compact_tier_bc: Option<PathBuf>,
    pub message_format: MessageFormat,
    pub apply_machine_fixes: bool,
    pub docs: PathBuf,
    pub out: PathBuf,
    pub out_dir: Option<PathBuf>,
    pub digest: PathBuf,
    pub force: bool,
    pub dry_run: bool,
    pub scoring_config: Option<PathBuf>,
}

/// Strongly-typed algebraic sum type representing the mutually exclusive operational modes of `compile-sysml`.
/// Safety & Traceability: Realises: [REQ-COMPILER-SYSML-COMMAND-MODE], [REQ-0608], [DO-330-TQL-4-CLI]
/// Contract: Total partitioning of valid `Cli` invocations into isolated SESE dispatch variants.
/// Bounds & Faults: O(1) variant discrimination; cannot represent conflicting operational modes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandMode {
    InjectDocstrings { target_dir: PathBuf, schema: Option<PathBuf>, dry_run: bool, force: bool },
    ExtractSource { extract_dir: PathBuf, out: PathBuf, digest: PathBuf, dry_run: bool },
    ExportMbd { mbd_dir: PathBuf, schema: Option<PathBuf>, dry_run: bool },
    SlicePackage { target_pkg: String, schema: Option<PathBuf> },
    EmitFlutterContracts { target_dir: PathBuf, schema: Option<PathBuf>, dry_run: bool },
    EmitModularRust { target_dir: PathBuf, schema: Option<PathBuf>, dry_run: bool },
    ListRustItems { target_file: PathBuf },
    ExtractRustModule { source_file: PathBuf, dest_file: PathBuf, items_csv: String, dry_run: bool },
    CompactTierBc { target_path: PathBuf, dry_run: bool },
    ApplyMachineFixes { target_path: PathBuf, schema: Option<PathBuf>, dry_run: bool, force: bool },
    StpaTranspile { schema: Option<PathBuf>, out_dir: PathBuf, scoring_config: Option<PathBuf>, dry_run: bool },
    StpaToConstraints { stpa_file: PathBuf, schema: Option<PathBuf>, out: PathBuf, dry_run: bool },
    ForwardSync { schema: Option<PathBuf>, docs: PathBuf, out_dir: Option<PathBuf>, dry_run: bool, force: bool },
    ReverseSync { schema: Option<PathBuf>, docs: PathBuf, out: PathBuf, digest: PathBuf, force: bool, dry_run: bool },
    Compile { schema: Option<PathBuf>, out: PathBuf, digest: PathBuf, dry_run: bool },
    ShowUsage,
}

impl Cli {
    /// Counts the number of primary operational mode flags active on `self`.
    /// Realises: [REQ-COMPILER-SYSML-CLI-MODE-COUNT]
    fn count_active_modes(&self) -> usize {
        usize::from(self.inject_safety_docstrings.is_some())
            + usize::from(self.extract_source.is_some())
            + usize::from(self.export_mbd.is_some())
            + usize::from(self.slice_package.is_some())
            + usize::from(self.emit_flutter_contracts.is_some())
            + usize::from(self.emit_modular_rust.is_some())
            + usize::from(self.list_rust_items.is_some())
            + usize::from(self.extract_rust_module.is_some())
            + usize::from(self.compact_tier_bc.is_some())
            + usize::from(self.stpa_transpile)
            + usize::from(self.stpa.is_some())
            + usize::from(self.forward_sync)
            + usize::from(self.reverse_sync)
            + usize::from(self.compile)
    }

    /// Detects whether `--schema` or `FILE` points to a self-contained sub-workspace root directory.
    /// Realises: [REQ-COMPILER-SYSML-SUBWORKSPACE-DETECT]
    fn resolve_subworkspace_root(&self) -> Option<PathBuf> {
        let candidate = self.schema.as_deref().or(self.file.as_deref())?;
        let is_non_root_dir = candidate.is_dir()
            && candidate != Path::new(".")
            && candidate.file_name().is_none_or(|n| n != "schema");
        let has_workspace_markers = candidate.join("schema").is_dir()
            || candidate.join(".pipeline").is_dir()
            || candidate.ends_with(".pipeline/upstream");
        (is_non_root_dir && has_workspace_markers).then(|| candidate.to_path_buf())
    }

    /// Scopes default `out`, `digest`, and `docs` paths into a detected sub-workspace root.
    /// Realises: [REQ-COMPILER-SYSML-SUBWORKSPACE-NORMALIZE]
    fn normalize_subworkspace_paths(&mut self) {
        if let Some(ws) = self.resolve_subworkspace_root() {
            if self.out == Path::new(".pipeline/schema.sysml") {
                self.out = ws.join(".pipeline").join("schema.sysml");
            }
            if self.digest == Path::new(".pipeline/schema-digest.json") {
                self.digest = ws.join(".pipeline").join("schema-digest.json");
            }
            if self.docs == Path::new("docs/") || self.docs == Path::new("docs") {
                self.docs = ws.join("docs");
            }
        }
    }

    /// Extracts Track-1 CST refactoring modes (`--list-rust-items`, `--extract-rust-module`, `--compact-tier-bc`, `--apply-machine-fixes`).
    /// Realises: [REQ-0608], [SC-01]
    fn take_track1_cst_mode(&mut self) -> Result<Option<CommandMode>, String> {
        if let Some(target_file) = self.list_rust_items.take() {
            return Ok(Some(CommandMode::ListRustItems { target_file }));
        }
        if let Some(source_file) = self.extract_rust_module.take() {
            let items_csv = self.items.take().ok_or_else(|| {
                "Error: --extract-rust-module requires --items <CSV> and --dest <DEST>.".to_string()
            })?;
            let dest_file = self.dest.take().ok_or_else(|| {
                "Error: --extract-rust-module requires --items <CSV> and --dest <DEST>.".to_string()
            })?;
            return Ok(Some(CommandMode::ExtractRustModule {
                source_file,
                dest_file,
                items_csv,
                dry_run: self.dry_run,
            }));
        }
        if let Some(target_path) = self.compact_tier_bc.take() {
            return Ok(Some(CommandMode::CompactTierBc {
                target_path,
                dry_run: self.dry_run,
            }));
        }
        if self.apply_machine_fixes && self.count_active_modes() == 0 {
            let target_path = self
                .inject_safety_docstrings
                .take()
                .or_else(|| self.file.take())
                .unwrap_or_else(|| PathBuf::from("."));
            return Ok(Some(CommandMode::ApplyMachineFixes {
                target_path,
                schema: self.schema.take(),
                dry_run: self.dry_run,
                force: self.force,
            }));
        }
        Ok(None)
    }

    /// Extracts codegen, docstring, source extraction, MBD export, or package slicing modes if active.
    /// Realises: [REQ-COMPILER-SYSML-INTO-CODEGEN-MODE]
    fn take_codegen_mode(&mut self) -> Option<CommandMode> {
        let schema = self.schema.clone().or_else(|| self.file.clone());
        if let Some(target_dir) = self.inject_safety_docstrings.take() {
            if self.apply_machine_fixes {
                return Some(CommandMode::ApplyMachineFixes {
                    target_path: target_dir,
                    schema,
                    dry_run: self.dry_run,
                    force: self.force,
                });
            }
            return Some(CommandMode::InjectDocstrings {
                target_dir,
                schema,
                dry_run: self.dry_run,
                force: self.force,
            });
        }
        if let Some(extract_dir) = self.extract_source.take() {
            return Some(CommandMode::ExtractSource {
                extract_dir,
                out: self.out.clone(),
                digest: self.digest.clone(),
                dry_run: self.dry_run,
            });
        }
        if let Some(mbd_dir) = self.export_mbd.take() {
            return Some(CommandMode::ExportMbd { mbd_dir, schema, dry_run: self.dry_run });
        }
        if let Some(target_pkg) = self.slice_package.take() {
            return Some(CommandMode::SlicePackage { target_pkg, schema });
        }
        if let Some(target_dir) = self.emit_flutter_contracts.take() {
            return Some(CommandMode::EmitFlutterContracts { target_dir, schema, dry_run: self.dry_run });
        }
        if let Some(target_dir) = self.emit_modular_rust.take() {
            return Some(CommandMode::EmitModularRust { target_dir, schema, dry_run: self.dry_run });
        }
        None
    }

    /// Resolves STPA safety transpilation, bidirectional synchronization, compilation, or usage modes.
    /// Realises: [REQ-COMPILER-SYSML-INTO-PIPELINE-MODE]
    fn into_pipeline_mode(self) -> CommandMode {
        let schema = self.schema.or(self.file);
        if self.stpa_transpile {
            return CommandMode::StpaTranspile {
                schema,
                out_dir: self.out_dir.unwrap_or_else(|| self.docs.join("safety")),
                scoring_config: self.scoring_config,
                dry_run: self.dry_run,
            };
        }
        if let Some(stpa_file) = self.stpa {
            return CommandMode::StpaToConstraints { stpa_file, schema, out: self.out, dry_run: self.dry_run };
        }
        if self.forward_sync {
            return CommandMode::ForwardSync {
                schema,
                docs: self.docs,
                out_dir: self.out_dir,
                dry_run: self.dry_run,
                force: self.force,
            };
        }
        if self.reverse_sync {
            return CommandMode::ReverseSync {
                schema,
                docs: self.docs,
                out: self.out,
                digest: self.digest,
                force: self.force,
                dry_run: self.dry_run,
            };
        }
        if self.compile || schema.is_some() {
            return CommandMode::Compile { schema, out: self.out, digest: self.digest, dry_run: self.dry_run };
        }
        CommandMode::ShowUsage
    }

    /// Converts parsed CLI flags into a validated, mutually exclusive `CommandMode`.
    /// Safety & Traceability: Realises: [REQ-COMPILER-SYSML-CLI-INTO-MODE], [REQ-0608], [DO-330-TQL-4-CLI]
    /// Contract: Rejects multiple primary operational mode flags (`count_active_modes() > 1`) with `Err(String)`.
    /// Bounds & Faults: O(1) time and space; never panics.
    pub fn into_mode(mut self) -> Result<CommandMode, String> {
        if self.count_active_modes() > 1 {
            return Err(
                "Error: Multiple operational mode flags specified. Supply exactly one primary mode flag per invocation."
                    .to_string(),
            );
        }
        self.normalize_subworkspace_paths();
        if let Some(cst_mode) = self.take_track1_cst_mode()? {
            return Ok(cst_mode);
        }
        if let Some(mode) = self.take_codegen_mode() {
            return Ok(mode);
        }
        Ok(self.into_pipeline_mode())
    }

    /// Parses process command-line arguments from `std::env::args_os` into `Cli`.
    /// Safety & Traceability: Realises: [REQ-COMPILER-SYSML-CLI-PARSE], [DO-330-TQL-4-CLI]
    /// Contract: Parses OS process argument tokens via `build_cli_command()`.
    /// Bounds & Faults: O(A) time and space in CLI argument token count `A`.
    #[must_use]
    pub fn parse() -> Self {
        if let Ok(matches) = build_cli_command().try_get_matches() {
            return cli_from_matches(&matches);
        }
        crate::cst_cli::handle_cst_cli_if_present();
        let matches = build_cli_command().get_matches();
        cli_from_matches(&matches)
    }

    /// Parses an arbitrary iterator of argument tokens into `Cli`.
    /// Safety & Traceability: Realises: [REQ-COMPILER-SYSML-CLI-PARSE-FROM], [DO-330-TQL-4-CLI]
    /// Contract: Produces identical `Cli` bindings as `Self::parse()` for equivalent argument sequences.
    /// Bounds & Faults: O(A) time and space in token count `A`.
    #[must_use]
    pub fn parse_from<I, T>(itr: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let matches = build_cli_command().get_matches_from(itr);
        cli_from_matches(&matches)
    }

    /// Fallibly parses an arbitrary iterator of argument tokens into `Cli` without terminating the process.
    /// Safety & Traceability: Realises: [REQ-0607], [REQ-COMPILER-SYSML-CLI-PARSE-FROM]
    /// Contract: Returns `Ok(Cli)` on valid syntax or `Err(clap::Error)` on invalid syntax.
    /// Bounds & Faults: O(A) time and space in token count `A`; never calls `process::exit`.
    pub fn try_parse_from<I, T>(itr: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let matches = build_cli_command().try_get_matches_from(itr)?;
        Ok(cli_from_matches(&matches))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies `MessageFormat::from_flag_lossless` and bare `Cli::into_mode` smoke invariants.
    #[test]
    fn test_cli_smoke_invariants() {
        assert_eq!(MessageFormat::from_flag_lossless("json"), MessageFormat::Json);
        assert_eq!(MessageFormat::from_flag_lossless("human"), MessageFormat::Human);
        let cli = Cli::parse_from(["compile-sysml"]);
        assert_eq!(cli.into_mode(), Ok(CommandMode::ShowUsage));
    }
}
