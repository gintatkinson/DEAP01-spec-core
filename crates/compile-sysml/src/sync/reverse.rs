#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Markdown Specifications to SysML v2 Reverse Synchronization Engine.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety-critical
//! software engineering standards. Mitigates dual-schema AST drift hazards (HAZ-SCHEMA-DRIFT-GATE31)
//! and specification-model desynchronization (HAZ-SPEC-MODEL-DESYNC).
//!
//! ## 2. Invariants & Mathematical Properties
//! - KerML Identifier Invariant: All emitted classifier identifiers conform to KerML grammar
//!   rules (starting with an alphabetic character or underscore, containing only alphanumeric/underscores).
//! - Non-Destructive AST Parity: Existing schema AST structures and subsystem part definitions
//!   are preserved, merging markdown-extracted operations into existing parts without creating
//!   duplicate top-level entities.
//! - Deterministic Serialization & Digest Integrity: Emitted `.pipeline/schema.sysml` and
//!   `.pipeline/schema-digest.json` produce bit-for-bit identical outputs for identical inputs.
//!
//! ## 3. Preconditions
//! - `opts.docs_dir` must exist and be readable.
//! - Parsed markdown frontmatters must contain valid type discriminators.
//!
//! ## 4. Postconditions
//! - Returns `(PackageDef, SchemaDigest)` representing the unified AST and cryptographic digest.
//! - Atomically persists the updated schema and digest when configured.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N_{docs} \cdot L_{doc} + V_{ast} + E_{ast})$ where $N_{docs}$ is the number
//!   of specification files, $L_{doc}$ is average file length, and $(V, E)$ represent AST nodes and edges.
//! - Space Complexity: $O(V_{ast} + E_{ast})$ memory for AST and extracted elements.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Returns `Err(String)` if `docs_dir` is missing, in-place overwrite is unauthorized, or filesystem writes fail.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0604] [REQ-SYSML-REVERSE-SYNC-ENGINE]

pub mod extractor;
pub mod frontmatter;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_manifest_and_utf8;

use std::fs;
use std::path::PathBuf;

use deap_core::sysml_ast::PackageDef;
pub use frontmatter::*;

use crate::semantic::digest::{generate_digest, write_atomic, write_digest_atomic, SchemaDigest};
use crate::semantic::serializer::to_sysml;

/// Options for reverse synchronization.
///
/// Safety & Traceability: Realises: [REQ-0604] [REQ-SYNC-REVERSE-REVERSESYNCOPTIONS]
/// Contract: Specifies input `docs_dir`, optional base `schema_path`, target `output_path` and `digest_path`, and `allow_overwrite` guard.
/// Bounds & Faults: Pure value struct holding owned `PathBuf` paths and boolean overwrite permission flag.
#[derive(Debug, Clone)]
pub struct ReverseSyncOptions {
    pub docs_dir: PathBuf,
    pub schema_path: Option<PathBuf>,
    pub output_path: PathBuf,
    pub digest_path: PathBuf,
    pub allow_overwrite: bool,
}

/// Discovers `.sysml` files under `schema_path` and `docs/designs/`, refreshing sidecar `.digest.json` files when `!dry_run`.
/// Realises: [REQ-0604] [REQ-SYNC-REVERSE-REVERSE-SYNC-SPECS-TO-SYSML]
fn collect_and_refresh_manifest_files(
    opts: &ReverseSyncOptions,
    dry_run: bool,
) -> Result<Vec<PathBuf>, String> {
    let mut manifest_files: Vec<PathBuf> = Vec::new();
    let schema_dir = opts
        .schema_path
        .as_deref()
        .unwrap_or_else(|| std::path::Path::new("schema"));
    if schema_dir.is_dir() {
        manifest_files.extend(crate::parser::grammar::discover_sysml_files(schema_dir));
    } else if schema_dir.is_file() {
        manifest_files.push(schema_dir.to_path_buf());
    }

    let designs_dir = opts.docs_dir.join("designs");
    if designs_dir.is_dir() {
        for design_sysml in crate::parser::grammar::discover_sysml_files(&designs_dir) {
            if !dry_run {
                let design_src = fs::read_to_string(&design_sysml).map_err(|e| {
                    format!(
                        "Failed to read design SysML file '{}': {}",
                        design_sysml.display(),
                        e
                    )
                })?;
                let design_pkg = crate::parse_sysml(&design_src).map_err(|e| e.to_string())?;
                let single_manifest = crate::semantic::digest::compute_file_manifest(
                    std::slice::from_ref(&design_sysml),
                );
                let bp_digest =
                    generate_digest(&design_pkg, &design_src).with_file_manifest(single_manifest);
                if let Some(stem) = design_sysml.file_stem().and_then(|s| s.to_str()) {
                    let sidecar_path = designs_dir.join(format!("{stem}.digest.json"));
                    write_digest_atomic(&sidecar_path, &bp_digest).map_err(|e| e.to_string())?;
                }
            }
            manifest_files.push(design_sysml);
        }
    }

    manifest_files.sort();
    manifest_files.dedup();
    Ok(manifest_files)
}

/// Reverse synchronizes markdown documents under `docs_dir` into the SysML AST SSOT.
///
/// Safety & Traceability: Realises: [REQ-0604] [REQ-SYSML-SYNC-REVERSE/ENGINE-SYNC]
/// Contract: Extracts Markdown specifications under `opts.docs_dir` into `base_pkg` and atomically writes `.pipeline/schema.sysml` and `.pipeline/schema-digest.json`.
/// Bounds & Faults: Delegates to `reverse_sync_specs_to_sysml_with_dry_run(..., false)`.
pub fn reverse_sync_specs_to_sysml(
    base_pkg: Option<&PackageDef>,
    opts: &ReverseSyncOptions,
) -> Result<(PackageDef, SchemaDigest), String> {
    reverse_sync_specs_to_sysml_with_dry_run(base_pkg, opts, false)
}

/// Synchronizes Markdown specifications into a SysML v2 AST package and cryptographic digest,
/// optionally suppressing disk writes when `dry_run` is `true`.
///
/// Safety & Traceability: Realises: [REQ-0604] [REQ-SYSML-SYNC-REVERSE/ENGINE-SYNC]
/// Contract: Updates `PartDef` operations in-place via `pkg.find_part_mut`, computes the unified SHA-256 `SchemaDigest` and `file_manifest`, and performs zero disk writes when `dry_run` is `true`.
/// Bounds & Faults: Validates directory existence and overwrite permissions fail-closed, returning `Err(String)` on I/O or parse errors.
pub fn reverse_sync_specs_to_sysml_with_dry_run(
    base_pkg: Option<&PackageDef>,
    opts: &ReverseSyncOptions,
    dry_run: bool,
) -> Result<(PackageDef, SchemaDigest), String> {
    if !opts.docs_dir.exists() {
        return Err(format!(
            "Docs directory does not exist: {}",
            opts.docs_dir.display()
        ));
    }
    if !opts.allow_overwrite && opts.schema_path.as_ref() == Some(&opts.output_path) {
        return Err(format!(
            "In-place overwrite of base input schema '{}' is prohibited when allow_overwrite=false.",
            opts.output_path.display()
        ));
    }

    let mut pkg = base_pkg.cloned().unwrap_or_else(|| PackageDef {
        name: "System_SSOT".to_string(),
        doc: Some("Single Source of Truth for System Architecture and Safety Model".to_string()),
        ..Default::default()
    });

    extractor::extract_markdown_specs_into_package(&mut pkg, &opts.docs_dir)?;
    let sysml_code = to_sysml(&pkg);

    if !dry_run {
        if let Some(parent) = opts.output_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory '{}': {}", parent.display(), e))?;
        }
        write_atomic(&opts.output_path, &sysml_code).map_err(|e| e.to_string())?;
    }

    let mut digest = generate_digest(&pkg, &sysml_code);
    let manifest_files = collect_and_refresh_manifest_files(opts, dry_run)?;
    if !manifest_files.is_empty() {
        let manifest = crate::semantic::digest::compute_file_manifest(&manifest_files);
        digest = digest.with_file_manifest(manifest);
    }

    if dry_run {
        println!(
            "[SysML v2 Reverse-Sync] [DRY RUN] Would merge specifications from '{}' -> '{}' (SHA-256: {})",
            opts.docs_dir.display(),
            opts.output_path.display(),
            digest.sha256
        );
        return Ok((pkg, digest));
    }

    write_digest_atomic(&opts.digest_path, &digest).map_err(|e| e.to_string())?;
    println!(
        "[SysML v2 Reverse-Sync] Merged specifications from '{}' -> '{}' (SHA-256: {})",
        opts.docs_dir.display(),
        opts.output_path.display(),
        digest.sha256
    );
    Ok((pkg, digest))
}
