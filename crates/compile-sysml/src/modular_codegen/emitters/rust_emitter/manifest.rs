//! Layer 3 deterministic modular Rust source emitter, bounds auditor, and tree manifest builder.
//! Safety & Traceability: Realises: [REQ-0607]
use super::render::render_unit;
use crate::modular_codegen::{domain::bounds as bd, domain::module_graph as mg, lowering as l};
use crate::semantic::digest::write_atomic;
use bd::{ModularityBounds, ModularityDiagnostic, ModularityErrorCode as Ec};
use deap_core::{digest::compute_sha256, source_ast::parse_rust_source_ast, sysml_ast::PackageDef};
use mg::{GeneratedFileDigest, GeneratedTreeManifest, ModuleGraph};
use std::{collections::BTreeMap, collections::BTreeSet, fs, path::Path, path::PathBuf};

/// Canonical `(relative_path, rust_source, file_digest)` triple returned for each synthesized Rust unit.
/// Realises: [REQ-0607]
pub type SynthesizedRustUnit = (String, String, GeneratedFileDigest);

/// ## 1. Safety Intent & Regulatory Scope: `pub struct ModularRustOptions` bounds filesystem side effects and enforces explicit `ModularityBounds`.
/// ## 2. Invariants & Mathematical Properties: `target_dir` names destination root, `dry_run` gates disk writes, and `bounds` holds positive ceilings.
/// ## 3. Preconditions: Directs `emit_modular_rust_tree` to synthesize an in-memory manifest or persist files atomically.
/// ## 4. Postconditions: Invalid target paths or tight `bounds` surface deterministically via `ModularityDiagnostic` results.
/// ## 5. Algorithmic Complexity Bounds: Fixed-size configuration struct (`O(|target_dir|)` bytes) with zero ambient environment reads.
/// ## 6. Failure Modes & Fault Tolerance: Verified by exercising both `dry_run = true` zero-write runs and `dry_run = false` atomic persistence.
/// ## 7. Realization & Traceability: Configures Layer 3 `emit_modular_rust_tree` invocations from CLI and verification callers.
/// Realises: [REQ-0607]
#[derive(Debug, Clone)]
pub struct ModularRustOptions {
    pub target_dir: PathBuf,
    pub dry_run: bool,
    pub bounds: ModularityBounds,
}

/// ## 1. Safety Intent & Regulatory Scope: `fn digest_rust_source` audits generated file metrics.
/// ## 2. Invariants & Mathematical Properties: `rel` is a normalized workspace-relative file label and `src` holds candidate UTF-8 Rust source text.
/// ## 3. Preconditions: Parses `src` via `parse_rust_source_ast`, slices callable spans without range indexing, and computes `v(G)`.
/// ## 4. Postconditions: Unparseable Rust text maps to `Err(ModularityDiagnostic)` tagged `ModularityErrorCode::BoundsViolationE0704`.
/// ## 5. Algorithmic Complexity Bounds: Linear `O(|lines|)` slicing and hashing with `v(G) = 3` (`<= 8`) and zero slice-index panics.
/// ## 6. Failure Modes & Fault Tolerance: Verified by digesting synthesized units and asserting exact `sha256`, `line_count`, `item_count`, and peak `v(G)`.
/// ## 7. Realization & Traceability: Shared metric and cryptographic digest calculator for in-memory synthesis and on-disk tree verification.
/// Realises: [REQ-0607]
pub fn digest_rust_source(rel: &str, src: &str) -> Result<GeneratedFileDigest, ModularityDiagnostic> {
    let mk_e = |e| ModularityDiagnostic::new(Ec::BoundsViolationE0704, rel, e);
    let a = parse_rust_source_ast(src, rel).map_err(mk_e)?;
    let (ls, mut max_vg) = (src.lines().collect::<Vec<_>>(), 1);
    let fns = a.iter().filter(|i| i.kind.is_callable()).map(|i| i.line_no);
    let st: BTreeSet<_> = fns.chain([ls.len().saturating_add(1)]).collect();
    for (&s, &e) in st.iter().zip(st.iter().skip(1)) {
        let end = e.saturating_sub(1).min(ls.len());
        let sl = ls.get(s.saturating_sub(1)..end).unwrap_or(&[]);
        max_vg = max_vg.max(l::estimate_rust_fn_cyclomatic_complexity(&sl.join("\n")));
    }
    Ok(GeneratedFileDigest {
        relative_path: rel.to_string(),
        sha256: compute_sha256(src.as_bytes()),
        line_count: ls.len(),
        item_count: a.len(),
        max_cyclomatic_complexity: max_vg,
    })
}

/// ## 1. Safety Intent & Regulatory Scope: `pub fn synthesize_modular_rust_files` gates emission on ceilings.
/// ## 2. Invariants & Mathematical Properties: `graph` is a DAG-verified `ModuleGraph` and `bounds` sets line, item, and cyclomatic complexity ceilings.
/// ## 3. Preconditions: Returns canonically sorted `SynthesizedRustUnit` triples iff every generated file satisfies all ceilings.
/// ## 4. Postconditions: Accumulates all `ModularityErrorCode::BoundsViolationE0704` diagnostics across breaching files before returning `Err`.
/// ## 5. Algorithmic Complexity Bounds: Sorts output via `BTreeMap` in `O(U log U)` time with `v(G) = 7` (`<= 8`).
/// ## 6. Failure Modes & Fault Tolerance: Verified by testing compliant graphs (`Ok`) alongside tightened `ModularityBounds` negative probes (`Err(E0704)`).
/// ## 7. Realization & Traceability: Enforces post-synthesis quantitative modularity certification prior to any filesystem persistence.
/// Realises: [REQ-0607]
pub fn synthesize_modular_rust_files(
    graph: &ModuleGraph,
    bounds: &ModularityBounds,
) -> Result<Vec<SynthesizedRustUnit>, Vec<ModularityDiagnostic>> {
    let by_path: BTreeMap<_, _> = graph.units.iter().map(|u| (&u.relative_path, u)).collect();
    let (mut out, mut diags) = (Vec::with_capacity(by_path.len()), Vec::new());
    let (ml, mi) = (bounds.max_file_lines, bounds.max_items_per_file);
    let mv = bounds.max_cyclomatic_complexity;
    for (rel, u) in by_path {
        let src = render_unit(u);
        match digest_rust_source(rel, &src) {
            Ok(d) => {
                let (dl, di, dv) = (d.line_count, d.item_count, d.max_cyclomatic_complexity);
                if dl > ml || di > mi || dv > mv {
                    let s = format!("lines={dl}/{ml}, items={di}/{mi}, max_vg={dv}/{mv}");
                    let m = format!("bounds exceeded: {s}");
                    diags.push(ModularityDiagnostic::new(Ec::BoundsViolationE0704, rel, m));
                }
                out.push((rel.clone(), src, d));
            }
            Err(e) => diags.push(e),
        }
    }
    if !diags.is_empty() {
        return Err(diags);
    }
    Ok(out)
}

/// ## 1. Safety Intent & Regulatory Scope: `pub fn emit_modular_rust_tree` orchestrates modular Rust emission.
/// ## 2. Invariants & Mathematical Properties: `pkg` is a parsed SysML `PackageDef` and `options` specifies `target_dir`, `dry_run`, and `ModularityBounds`.
/// ## 3. Preconditions: Emits atomic files when `!options.dry_run` and returns `GeneratedTreeManifest` hashing `"{rel}:{sha256}\n"`.
/// ## 4. Postconditions: Propagates lowering and bounds diagnostics without partial writes, and wraps atomic write I/O errors cleanly.
/// ## 5. Algorithmic Complexity Bounds: Deterministic `O(N)` manifest hashing over `BTreeMap`-ordered units with `v(G) = 6` (`<= 8`).
/// ## 6. Failure Modes & Fault Tolerance: Verified by testing dry-run non-persistence, full emission, and stage-1 versus stage-2 manifest fixed-point parity.
/// ## 7. Realization & Traceability: Primary Layer 3 entry point compiling SysML v2 packages into modular Rust directory trees.
/// Realises: [REQ-0607]
pub fn emit_modular_rust_tree(
    pkg: &PackageDef,
    options: &ModularRustOptions,
) -> Result<GeneratedTreeManifest, Vec<ModularityDiagnostic>> {
    let graph = l::lower_package_to_module_graph(pkg, &options.bounds)?;
    let units = synthesize_modular_rust_files(&graph, &options.bounds)?;
    if !options.dry_run {
        for (rel, src, _) in &units {
            let full = options.target_dir.join(rel);
            let io = full.parent().map(fs::create_dir_all).transpose();
            if let Err(e) = io.and_then(|_| write_atomic(&full, src)) {
                let m = format!("atomic write failed: {e}");
                let d = ModularityDiagnostic::new(Ec::BoundsViolationE0704, rel, m);
                return Err(vec![d]);
            }
        }
    }
    let files: Vec<_> = units.into_iter().map(|(_, _, d)| d).collect();
    let row = |d: &GeneratedFileDigest| format!("{}:{}\n", d.relative_path, d.sha256);
    let tree_sha256 = compute_sha256(files.iter().map(row).collect::<String>().as_bytes());
    Ok(GeneratedTreeManifest { files, tree_sha256 })
}

/// ## 1. Safety Intent & Regulatory Scope: `pub fn compute_directory_tree_manifest` audits on-disk trees.
/// ## 2. Invariants & Mathematical Properties: `dir` points to a readable directory containing generated `.rs` source files in relative subdirectories.
/// ## 3. Preconditions: Traverses via explicit `Vec<PathBuf>` workstack, normalizes `/` paths, sorts via `BTreeMap`, and hashes `"{rel}:{sha256}\n"`.
/// ## 4. Postconditions: Directory, read, or AST-parse errors return `Err(ModularityDiagnostic)` without TOCTOU `exists()` checks or panics.
/// ## 5. Algorithmic Complexity Bounds: Iterative `O(F log F)` traversal over `F` files using bounded heap workstack and `v(G) = 7` (`<= 8`).
/// ## 6. Failure Modes & Fault Tolerance: Verified by comparing `emit_modular_rust_tree` output against `compute_directory_tree_manifest` via `verify_fixed_point`.
/// ## 7. Realization & Traceability: Certifies on-disk source tree integrity for closed-loop stage-2 fixed-point verification (`E0705`).
/// Realises: [REQ-0607]
pub fn compute_directory_tree_manifest(
    dir: &Path,
) -> Result<GeneratedTreeManifest, ModularityDiagnostic> {
    let err = |l: &str, m: &str, e: std::io::Error| {
        ModularityDiagnostic::new(Ec::FixedPointDriftE0705, l, format!("{m}: {e}"))
    };
    let (mut stack, mut ps) = (vec![dir.to_path_buf()], BTreeMap::new());
    while let Some(curr) = stack.pop() {
        let loc = curr.to_string_lossy().into_owned();
        let item = |r: std::io::Result<fs::DirEntry>| {
            r.and_then(|e| e.file_type().map(|ft| (e.path(), ft)))
        };
        let rd = fs::read_dir(&curr).and_then(|it| it.map(item).collect::<Result<Vec<_>, _>>());
        for (p, ft) in rd.map_err(|e| err(&loc, "directory scan failed", e))? {
            if ft.is_dir() {
                stack.push(p);
            } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
                let cs = p.strip_prefix(dir).into_iter().flat_map(|r| r.components());
                let segs: Vec<_> = cs.map(|c| c.as_os_str().to_string_lossy()).collect();
                ps.insert(segs.join("/"), p);
            }
        }
    }
    let de = |(rel, p): (String, PathBuf)| {
        let io_res = fs::read_to_string(&p).map_err(|e| err(&rel, "read_to_string failed", e));
        io_res.and_then(|src| digest_rust_source(&rel, &src))
    };
    let files = ps.into_iter().map(de).collect::<Result<Vec<_>, _>>()?;
    let row = |d: &GeneratedFileDigest| format!("{}:{}\n", d.relative_path, d.sha256);
    let tree_sha256 = compute_sha256(files.iter().map(row).collect::<String>().as_bytes());
    Ok(GeneratedTreeManifest { files, tree_sha256 })
}
