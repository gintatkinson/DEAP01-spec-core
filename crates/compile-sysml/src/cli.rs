//! Layer 1 CLI Presentation & Algebraic Mode Resolution for `compile-sysml`.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides the declarative command-line interface contract (`Cli`) and strongly-typed algebraic
//! operational mode sum type (`CommandMode`) for `compile-sysml` under DO-178C Level A, DO-330 TQL-4,
//! and ISO 26262 ASIL D tool qualification standards.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Mutual exclusivity invariant: `Cli::into_mode` rejects any CLI invocation specifying more than
//!   one primary operational mode flag (`active_modes > 1`) with a deterministic error diagnostic.
//! - Zero stamp coupling: downstream orchestration handlers consume only the fields belonging to their
//!   resolved `CommandMode` variant rather than a 19-field mutable `Cli` record.
//!
//! ## 3. Preconditions
//! - Constructed from OS argument tokens via `Cli::parse()` or `Cli::parse_from(...)`.
//!
//! ## 4. Postconditions
//! - Produces a validated, immutable `CommandMode` value or a descriptive mutual-exclusivity `Err(String)`.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time complexity: $O(A)$ where $A$ is the number of command-line argument tokens.
//! - Space complexity: $O(A)$ for owned `PathBuf` and `String` parameters.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Conflicting mode flags return `Err(String)` cleanly without panicking or invoking `process::exit`.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-COMPILER-SYSML-CLI], [DO-330-TQL-4-CLI]

use clap::Parser;
use std::path::{Path, PathBuf};

/// Command-line argument definition for the `compile-sysml` compiler and synchronization engine.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Defines the deterministic CLI contract for `compile-sysml` under DO-178C Level A and DO-330 TQL-4,
/// covering Pipeline 0 compilation, STPA transpilation, specification synchronization, source AST
/// extraction, MBD export, package slicing, Flutter contract emission, and 7-section safety docstring synthesis.
///
/// ## 2. Invariants & Mathematical Properties
/// - Strongly typed argument binding via `clap::Parser`.
/// - Defaults `docs` to `"docs/"`, `out` to `".pipeline/schema.sysml"`, and `digest` to
///   `".pipeline/schema-digest.json"`.
///
/// ## 3. Preconditions
/// - Constructed from process arguments via `Cli::parse()` or `Cli::parse_from()`.
///
/// ## 4. Postconditions
/// - Populates validated `PathBuf` and boolean mode flags for algebraic conversion via `Cli::into_mode`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(A)$ where $A$ is the number of CLI argument tokens.
/// - Space Complexity: $O(A)$ for owned path buffers.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Invalid or unrecognized CLI flags are rejected by `clap` prior to entering pipeline execution.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-CLI-ARGS], [DO-330-TQL-4-CLI]
#[derive(Parser, Debug, Clone)]
#[command(name = "compile-sysml")]
#[command(about = "SysML v2 / KerML compiler and specification synchronization engine")]
pub struct Cli {
    /// Path to input .sysml file
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,

    /// Path to schema file or directory (default searches schema/*.sysml)
    #[arg(long, value_name = "SCHEMA_FILE")]
    pub schema: Option<PathBuf>,

    /// Run the Pipeline 0 compilation gate: parse, validate semantics, serialize, and emit digest
    #[arg(long)]
    pub compile: bool,

    /// Transpile SysML AST into the 10-pillar STPA safety artifact suite
    #[arg(long)]
    pub stpa_transpile: bool,

    /// Path to STPA markdown file to compile into SysML constraint notation
    #[arg(long, value_name = "STPA_FILE")]
    pub stpa: Option<PathBuf>,

    /// Execute reverse synchronization from markdown specs into SysML SSOT
    #[arg(long)]
    pub reverse_sync: bool,

    /// Execute forward synchronization from SysML SSOT into markdown specs
    #[arg(long)]
    pub forward_sync: bool,

    /// Synthesize and inject 7-section safety docstrings into source files under DIR
    #[arg(long, value_name = "DIR")]
    pub inject_safety_docstrings: Option<PathBuf>,

    /// Extract structural source AST elements from DIR into SysML v2 notation
    #[arg(long, value_name = "DIR")]
    pub extract_source: Option<PathBuf>,

    /// Export grounded MBD verification artifacts (.slreqx, .sfx, .m, layer_contracts.json) to DIR
    #[arg(long, value_name = "DIR")]
    pub export_mbd: Option<PathBuf>,

    /// Slice a target package subsystem AST within a deterministic token budget
    #[arg(long, value_name = "PACKAGE")]
    pub slice_package: Option<String>,

    /// Emit deterministic Flutter and C-ABI contracts (seeds, models, solver bridge, and UI tests)
    #[arg(long, value_name = "FLUTTER_DIR")]
    pub emit_flutter_contracts: Option<PathBuf>,

    /// Path to documentation directory
    #[arg(long, default_value = "docs/")]
    pub docs: PathBuf,

    /// Path to output schema model file
    #[arg(long, default_value = ".pipeline/schema.sysml")]
    pub out: PathBuf,

    /// Path to explicit output directory (for --stpa-transpile or --forward-sync)
    #[arg(long, value_name = "OUT_DIR")]
    pub out_dir: Option<PathBuf>,

    /// Path to output schema digest JSON
    #[arg(long, default_value = ".pipeline/schema-digest.json")]
    pub digest: PathBuf,

    /// Force overwrite of existing files / bypass landing zone guard
    #[arg(long)]
    pub force: bool,

    /// Simulate synchronization without writing changes to disk
    #[arg(long)]
    pub dry_run: bool,

    /// Path to optional FMECA scoring configuration JSON
    #[arg(long, value_name = "SCORING_CONFIG")]
    pub scoring_config: Option<PathBuf>,
}

/// Strongly-typed algebraic sum type representing the mutually exclusive operational modes of `compile-sysml`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Eliminates primitive obsession, dead boolean expressions, and 19-field stamp coupling across
/// `compile-sysml` pipeline stages under DO-178C Level A and DO-330 TQL-4 architectural modularity rules.
///
/// ## 2. Invariants & Mathematical Properties
/// - Total partitioning of valid CLI invocations: every valid `Cli` maps to exactly one `CommandMode` variant.
/// - Each variant carries only the parameters relevant to its specific pipeline stage.
///
/// ## 3. Preconditions
/// - Constructed exclusively via `Cli::into_mode` after verifying `count_active_modes() <= 1`.
///
/// ## 4. Postconditions
/// - Enables Single-Entry/Single-Exit (SESE) dispatch in `driver::execute` with zero flag re-probing.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$ variant discrimination.
/// - Space Complexity: $O(1)$ fixed enum size over owned `PathBuf` and `String` fields.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Algebraic sum type cannot represent conflicting simultaneous operational modes.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-COMMAND-MODE], [DO-330-TQL-4-CLI]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandMode {
    /// Synthesize and inject 7-section safety docstrings into Rust/Dart source files.
    InjectDocstrings {
        target_dir: PathBuf,
        schema: Option<PathBuf>,
        dry_run: bool,
        force: bool,
    },
    /// Extract structural source AST elements from a directory into SysML v2 notation.
    ExtractSource {
        extract_dir: PathBuf,
        out: PathBuf,
        digest: PathBuf,
        dry_run: bool,
    },
    /// Export grounded Model-Based Design (MBD) verification artifacts to a target directory.
    ExportMbd {
        mbd_dir: PathBuf,
        schema: Option<PathBuf>,
        dry_run: bool,
    },
    /// Slice a target package subsystem AST within a deterministic token budget.
    SlicePackage {
        target_pkg: String,
        schema: Option<PathBuf>,
    },
    /// Emit deterministic Flutter and C-ABI contracts for a target application workspace.
    EmitFlutterContracts {
        target_dir: PathBuf,
        schema: Option<PathBuf>,
        dry_run: bool,
    },
    /// Transpile a SysML v2 AST model into the 10-pillar STPA safety artifact suite.
    StpaTranspile {
        schema: Option<PathBuf>,
        out_dir: PathBuf,
        scoring_config: Option<PathBuf>,
        dry_run: bool,
    },
    /// Compile an STPA Markdown table into formal SysML v2 constraint definitions.
    StpaToConstraints {
        stpa_file: PathBuf,
        schema: Option<PathBuf>,
        out: PathBuf,
        dry_run: bool,
    },
    /// Forward-synchronize a SysML v2 SSOT model into Agile Markdown specifications.
    ForwardSync {
        schema: Option<PathBuf>,
        docs: PathBuf,
        out_dir: Option<PathBuf>,
        dry_run: bool,
        force: bool,
    },
    /// Reverse-synchronize Agile Markdown specifications into the SysML v2 SSOT model.
    ReverseSync {
        schema: Option<PathBuf>,
        docs: PathBuf,
        out: PathBuf,
        digest: PathBuf,
        force: bool,
        dry_run: bool,
    },
    /// Execute the Pipeline 0 SysML v2 compilation, semantic validation, and SHA-256 digest gate.
    Compile {
        schema: Option<PathBuf>,
        out: PathBuf,
        digest: PathBuf,
        dry_run: bool,
    },
    /// Display CLI usage guidance when no operational mode or input file is supplied.
    ShowUsage,
}

impl Cli {
    /// Counts the number of primary operational mode flags active on `self`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Provides branch-free cardinality counting of primary mode flags to enforce strict mutual
    /// exclusivity under DO-178C Level A and DO-330 TQL-4 deterministic CLI parsing rules.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Branchless sum over the 10 primary mode flags ($v(G) = 1$).
    /// - Returns a value in the closed interval $[0, 10]$.
    ///
    /// ## 3. Preconditions
    /// - `self` is a populated `Cli` instance.
    ///
    /// ## 4. Postconditions
    /// - Returns the exact count of active primary operational mode flags.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$.
    /// - Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Pure total function; never fails or panics.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-COMPILER-SYSML-CLI-MODE-COUNT]
    fn count_active_modes(&self) -> usize {
        usize::from(self.inject_safety_docstrings.is_some())
            + usize::from(self.extract_source.is_some())
            + usize::from(self.export_mbd.is_some())
            + usize::from(self.slice_package.is_some())
            + usize::from(self.emit_flutter_contracts.is_some())
            + usize::from(self.stpa_transpile)
            + usize::from(self.stpa.is_some())
            + usize::from(self.forward_sync)
            + usize::from(self.reverse_sync)
            + usize::from(self.compile)
    }

    /// Detects whether `--schema` or `FILE` points to a self-contained sub-workspace root directory.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Ensures sub-workspace targets (such as `.pipeline/upstream`) scope default `out`, `digest`,
    /// and `docs` paths into the target sub-workspace rather than overwriting parent workspace files
    /// under DO-330 TQL-4 workspace isolation rules.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Returns `Some(ws)` only when the candidate path is a directory distinct from `"."` and `"schema"`
    ///   that contains `schema/`, `.pipeline/`, or ends with `.pipeline/upstream`.
    ///
    /// ## 3. Preconditions
    /// - `self` is an initialized `Cli` struct.
    ///
    /// ## 4. Postconditions
    /// - Returns `Option<PathBuf>` identifying the sub-workspace directory if applicable.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$ metadata checks.
    /// - Space Complexity: $O(1)$ path allocation.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Non-existent paths or plain files cleanly evaluate to `None`.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-COMPILER-SYSML-SUBWORKSPACE-DETECT]
    fn resolve_subworkspace_root(&self) -> Option<PathBuf> {
        let candidate = self.schema.as_deref().or(self.file.as_deref())?;
        let is_non_root_dir = candidate.is_dir()
            && candidate != Path::new(".")
            && candidate.file_name().is_none_or(|n| n != "schema");
        let has_workspace_markers = candidate.join("schema").is_dir()
            || candidate.join(".pipeline").is_dir()
            || candidate.ends_with(".pipeline/upstream");
        if is_non_root_dir && has_workspace_markers {
            Some(candidate.to_path_buf())
        } else {
            None
        }
    }

    /// Scopes default `out`, `digest`, and `docs` paths into a detected sub-workspace root.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Preserves default path isolation when operating against nested sub-workspaces under DO-178C
    /// Level A and DO-330 TQL-4 configuration management rules.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Rebaselining occurs only when `out`, `digest`, or `docs` still hold their unmodified default values.
    /// - Explicit user-supplied `--out`, `--digest`, or `--docs` paths are never overwritten.
    ///
    /// ## 3. Preconditions
    /// - Invoked on a mutable `Cli` prior to constructing `CommandMode`.
    ///
    /// ## 4. Postconditions
    /// - Default output, digest, and docs paths are rooted under `ws` when `resolve_subworkspace_root()` is `Some(ws)`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$.
    /// - Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - No-op when not targeting a sub-workspace root.
    ///
    /// ## 7. Realization & Traceability
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

    /// Extracts codegen, docstring, source extraction, MBD export, or package slicing modes if active.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Bounds per-function cyclomatic complexity ($v(G) \le 6$) by partitioning auxiliary codegen
    /// and inspection modes from core pipeline compilation and synchronization modes under DO-178C Level A.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Consumes at most one mode option from `self` when `count_active_modes() <= 1`.
    ///
    /// ## 3. Preconditions
    /// - `self.count_active_modes() <= 1` and `self.normalize_subworkspace_paths()` have executed.
    ///
    /// ## 4. Postconditions
    /// - Returns `Some(CommandMode)` if a codegen/inspection flag is active, or `None` otherwise.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$.
    /// - Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Infallible total mapping over `Option` fields.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-COMPILER-SYSML-INTO-CODEGEN-MODE]
    fn take_codegen_mode(&mut self) -> Option<CommandMode> {
        let schema = self.schema.clone().or_else(|| self.file.clone());
        if let Some(target_dir) = self.inject_safety_docstrings.take() {
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
            return Some(CommandMode::ExportMbd {
                mbd_dir,
                schema,
                dry_run: self.dry_run,
            });
        }
        if let Some(target_pkg) = self.slice_package.take() {
            return Some(CommandMode::SlicePackage { target_pkg, schema });
        }
        if let Some(target_dir) = self.emit_flutter_contracts.take() {
            return Some(CommandMode::EmitFlutterContracts {
                target_dir,
                schema,
                dry_run: self.dry_run,
            });
        }
        None
    }

    /// Resolves STPA safety transpilation, bidirectional synchronization, compilation, or usage modes.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Completes the total partitioning of `Cli` into `CommandMode` while maintaining $v(G) \le 6$
    /// per function under DO-178C Level A and DO-330 TQL-4 structural complexity limits.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Maps `--compile` or positional/explicit schema arguments without another mode flag to `CommandMode::Compile`.
    /// - Maps bare invocations with no flags and no schema inputs to `CommandMode::ShowUsage`.
    ///
    /// ## 3. Preconditions
    /// - `self.count_active_modes() <= 1` and `self.into_codegen_mode()` returned `None`.
    ///
    /// ## 4. Postconditions
    /// - Returns the resolved `CommandMode` variant.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$.
    /// - Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Total function over `self`; never panics.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-COMPILER-SYSML-INTO-PIPELINE-MODE]
    fn into_pipeline_mode(self) -> CommandMode {
        let schema = self.schema.or(self.file);
        if self.stpa_transpile {
            return CommandMode::StpaTranspile {
                schema,
                out_dir: self.out_dir.unwrap_or_else(|| PathBuf::from("docs/safety")),
                scoring_config: self.scoring_config,
                dry_run: self.dry_run,
            };
        }
        if let Some(stpa_file) = self.stpa {
            return CommandMode::StpaToConstraints {
                stpa_file,
                schema,
                out: self.out,
                dry_run: self.dry_run,
            };
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
            return CommandMode::Compile {
                schema,
                out: self.out,
                digest: self.digest,
                dry_run: self.dry_run,
            };
        }
        CommandMode::ShowUsage
    }

    /// Converts parsed CLI flags into a validated, mutually exclusive `CommandMode`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Enforces strict mutual exclusivity across primary operational mode flags (`ARCH-05`) under
    /// DO-178C Level A and DO-330 TQL-4 tool operational requirements, preventing silent flag shadowing.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - If `count_active_modes() > 1`, returns `Err` immediately without executing any pipeline stage.
    /// - Otherwise normalizes sub-workspace paths and maps `self` into a unique `CommandMode` variant ($v(G) \le 6$).
    ///
    /// ## 3. Preconditions
    /// - `self` is a valid `Cli` struct produced by `clap` or constructed in tests.
    ///
    /// ## 4. Postconditions
    /// - Returns `Ok(CommandMode)` when at most one primary mode flag is active, or `Err(String)` when
    ///   multiple conflicting mode flags are supplied.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$.
    /// - Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Returns `Err("Error: Multiple operational mode flags specified. Supply exactly one primary mode flag per invocation.".to_string())` when `active_modes > 1`.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-COMPILER-SYSML-CLI-INTO-MODE], [DO-330-TQL-4-CLI]
    pub fn into_mode(mut self) -> Result<CommandMode, String> {
        if self.count_active_modes() > 1 {
            return Err(
                "Error: Multiple operational mode flags specified. Supply exactly one primary mode flag per invocation."
                    .to_string(),
            );
        }
        self.normalize_subworkspace_paths();
        if let Some(mode) = self.take_codegen_mode() {
            return Ok(mode);
        }
        Ok(self.into_pipeline_mode())
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]
mod tests {
    use super::*;
    use crate::{flatten_package_to_model, parse_sysml};

    #[test]
    fn test_cli_export_mbd_and_slice_package_flags_and_flatten() {
        let cli = Cli::parse_from(vec![
            "compile-sysml",
            "--export-mbd",
            "out/mbd",
            "--slice-package",
            "FlightPkg",
        ]);
        assert_eq!(cli.export_mbd, Some(PathBuf::from("out/mbd")));
        assert_eq!(cli.slice_package.as_deref(), Some("FlightPkg"));

        // Verify that combining two primary operational flags is rejected by into_mode()
        assert!(cli.into_mode().is_err());

        let pkg = parse_sysml(
            "package RootPkg { package FlightPkg { part def Autopilot { attribute gain : Real; } } }",
        )
        .expect("parse nested pkg");
        let model = flatten_package_to_model(pkg, Some("FlightPkg"));
        assert_eq!(model.package_name.as_deref(), Some("FlightPkg"));
        let slice = model
            .slice_package_subsystem("FlightPkg", 4096)
            .expect("slice should succeed");
        assert_eq!(slice.parts.len(), 1);
    }

    #[test]
    fn test_cli_emit_flutter_contracts_flag() {
        let cli = Cli::parse_from(vec![
            "compile-sysml",
            "--emit-flutter-contracts",
            "app_flutter",
        ]);
        assert_eq!(
            cli.emit_flutter_contracts,
            Some(PathBuf::from("app_flutter"))
        );
        assert_eq!(
            cli.into_mode(),
            Ok(CommandMode::EmitFlutterContracts {
                target_dir: PathBuf::from("app_flutter"),
                schema: None,
                dry_run: false,
            })
        );
    }

    /// Verifies `ARCH-05`: `Cli::into_mode` rejects mutually exclusive primary mode flags and maps single modes deterministically.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `test_arch_05_cli_into_mode_mutual_exclusivity_and_mapping`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Any invocation with `active_modes > 1` returns `Err`.
    /// - Single mode invocations map 1-to-1 to their corresponding `CommandMode` variant.
    ///
    /// ## 3. Preconditions
    /// - Synthetic CLI argument vectors passed to `Cli::parse_from`.
    ///
    /// ## 4. Postconditions
    /// - Asserts `--compile --reverse-sync` returns `Err` and single flags return `Ok(CommandMode::...)`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Pure in-memory unit test.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-COMPILER-SYSML-CLI-INTO-MODE]
    #[test]
    fn test_arch_05_cli_into_mode_mutual_exclusivity_and_mapping() {
        let conflicting = Cli::parse_from(vec!["compile-sysml", "--compile", "--reverse-sync"]);
        let err = conflicting
            .into_mode()
            .expect_err("conflicting --compile and --reverse-sync must fail");
        assert!(err.contains("Multiple operational mode flags specified"));

        let usage_cli = Cli::parse_from(vec!["compile-sysml"]);
        assert_eq!(usage_cli.into_mode(), Ok(CommandMode::ShowUsage));

        let implicit_compile = Cli::parse_from(vec!["compile-sysml", "schema/model.sysml", "--dry-run"]);
        assert_eq!(
            implicit_compile.into_mode(),
            Ok(CommandMode::Compile {
                schema: Some(PathBuf::from("schema/model.sysml")),
                out: PathBuf::from(".pipeline/schema.sysml"),
                digest: PathBuf::from(".pipeline/schema-digest.json"),
                dry_run: true,
            })
        );

        let explicit_compile = Cli::parse_from(vec![
            "compile-sysml",
            "--compile",
            "--schema",
            "schema/model.sysml",
        ]);
        assert_eq!(
            explicit_compile.into_mode(),
            Ok(CommandMode::Compile {
                schema: Some(PathBuf::from("schema/model.sysml")),
                out: PathBuf::from(".pipeline/schema.sysml"),
                digest: PathBuf::from(".pipeline/schema-digest.json"),
                dry_run: false,
            })
        );
    }
}
