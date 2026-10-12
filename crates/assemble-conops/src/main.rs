//! CLI binary for assemble-conops.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;

use assemble_conops::assemble_conops;

#[derive(Parser, Debug)]
#[command(
    name = "assemble-conops",
    author = "Gint Atkinson",
    version = "0.1.0",
    about = "Deterministic ConOps & Mission Intent Assembly Engine (ISO 29148 / NATO STANAG 4586 / OMG UAF)."
)]
struct Args {
    /// Target workspace or project directory (optional positional argument).
    #[arg(value_name = "WORKSPACE")]
    workspace: Option<PathBuf>,

    /// Target workspace or project directory.
    #[arg(long = "workspace")]
    workspace_flag: Option<PathBuf>,

    /// Input directory containing 'conops/' and 'mission_intent/' unit markdown directories (default: docs/conops/units).
    #[arg(long = "input-dir")]
    input_dir: Option<PathBuf>,

    /// Output directory where assembled CONOPS.md and MISSION_INTENT.md are written (default: docs/conops).
    #[arg(long = "output-dir")]
    output_dir: Option<PathBuf>,

    /// Verify unit integrity and link resolution without writing output files.
    #[arg(long = "verify", action = clap::ArgAction::SetTrue)]
    verify: bool,

    /// Path to JSON parameter dictionary or schema file (auto-detects .pipeline/schema-digest.json if not specified).
    #[arg(long = "params")]
    params: Option<PathBuf>,

    /// Target operational domain. Auto-detected if not specified.
    #[arg(long = "domain")]
    domain: Option<String>,
}

fn check_candidate_input_dir(cand: &Path) -> bool {
    if !cand.is_dir() {
        return false;
    }
    if cand.join("conops").is_dir() || cand.join("mission_intent").is_dir() {
        return true;
    }
    if let Ok(entries) = fs::read_dir(cand) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() && path.join("conops").is_dir() {
                return true;
            }
        }
    }
    false
}

fn main() -> ExitCode {
    let args = Args::parse();

    let target_ws = args.workspace_flag.or(args.workspace);
    let explicit_io = args.input_dir.is_some() || args.output_dir.is_some();

    let workspace: Option<PathBuf> = if let Some(ws) = target_ws {
        Some(ws.canonicalize().unwrap_or(ws))
    } else if !explicit_io {
        std::env::current_dir().ok()
    } else {
        None
    };

    let ws_ref = workspace
        .clone()
        .unwrap_or_else(|| PathBuf::from("."));

    let input_dir = if let Some(in_dir) = args.input_dir {
        in_dir
    } else {
        let mut resolved = None;
        for cand in &[
            ws_ref.join("docs").join("conops").join("units"),
            ws_ref.join("docs").join("conops"),
            ws_ref.join("units"),
            ws_ref.clone(),
        ] {
            if check_candidate_input_dir(cand) {
                resolved = Some(cand.clone());
                break;
            }
        }
        resolved.unwrap_or_else(|| ws_ref.join("docs").join("conops").join("units"))
    };

    let output_dir = if let Some(out_dir) = args.output_dir {
        out_dir
    } else {
        ws_ref.join("docs").join("conops")
    };

    match assemble_conops(
        &input_dir,
        &output_dir,
        args.verify,
        args.params.as_deref(),
        args.domain.as_deref(),
        workspace.as_deref(),
    ) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(err) => {
            eprintln!("[!] Assembly execution failed: {err}");
            ExitCode::FAILURE
        }
    }
}
