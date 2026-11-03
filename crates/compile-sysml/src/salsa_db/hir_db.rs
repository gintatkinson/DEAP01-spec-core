//! # Salsa HIR Lowering, Standard Library Index & Package Tree Merger (`hir_db`)
//! [DOC-TIER: A]
//! ## 1. Safety Intent & Regulatory Scope -- Provides `#[salsa::tracked]` HIR lowering (`lower_hir_tracked`), `ISQ`/`SI`/`ScalarValues` standard library resolution (`Durability::HIGH`), early-cutoff evaluation, and iterative multi-file package merging (`BP-10`).
//! ## 2. Invariants & Mathematical Properties -- Trivia-only edits preserve `HirPackageSummary::structural_hash`; iterative work-stacks bound package merging to `MAX_PARSE_DEPTH` (`64`).
//! ## 3. Preconditions -- `SalsaFileInput` is registered in `salsa::Database`; `.sysml` files are valid UTF-8.
//! ## 4. Postconditions -- Returns memoized `HirPackageSummary` and unified `PackageDef` trees.
//! ## 5. Algorithmic Complexity Bounds -- Time: O(1) on memoized hit, O(N) on miss; Space: O(N).
//! ## 6. Failure Modes & Fault Tolerance -- Zero call-stack recursion during package merging; returns typed `ParseError` on syntax or I/O errors.
//! ## 7. Realization & Traceability -- Realises: [REQ-0201], [REQ-0603], [REQ-0608] | [FLS: fls_66m4rnbssgig]

#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub use deap_core::sysml_cst::{
    evaluate_hir_early_cutoff, is_standard_library_package, lower_cst_to_hir, parse_sysml_cst,
    resolve_standard_library_symbol, DefId, HirMemberEntry, HirPackageSummary, HirSourceMap,
    PackageDef, STANDARD_LIBRARY_PACKAGES,
};

use super::cst_db::Span;
use super::hir_parser_core::{ParseError, SysmlParser, GRAMMAR_EOF_TOKEN, MAX_PARSE_DEPTH};
use super::source_db::{discover_sysml_files, SalsaFileInput};

/// Salsa tracked query lowering a `SalsaFileInput` into a memoized `HirPackageSummary`.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope -- Memoizes Rowan CST to HIR lowering with trivia-blind early cutoff (`BP-10`).
/// ## 2. Invariants & Mathematical Properties -- `HirPackageSummary::structural_hash` is invariant under whitespace and comment mutations.
/// ## 3. Preconditions -- `input` is a registered `SalsaFileInput` in `db`.
/// ## 4. Postconditions -- Returns `HirPackageSummary` containing interned `DefId` members and `structural_hash`.
/// ## 5. Algorithmic Complexity Bounds -- Time: O(1) memoized hit, O(N) miss; Space: O(N).
/// ## 6. Failure Modes & Fault Tolerance -- Total tracked function with zero panics.
/// ## 7. Realization & Traceability -- Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
#[salsa::tracked(returns(clone))]
pub fn lower_hir_tracked(
    db: &dyn salsa::Database,
    input: SalsaFileInput,
) -> HirPackageSummary {
    let parsed = parse_sysml_cst(input.text(db));
    let (summary, _source_map) = lower_cst_to_hir(&parsed);
    summary
}

/// Work-stack frame for bounded iterative child-package merging in `merge_package_defs`. Realises: [REQ-PARSER-GRAMMAR-MERGE-PACKAGE-DEFS] | [FLS: fls_66m4rnbssgig]
struct PackageMergeFrame<'a> {
    dest: &'a mut PackageDef,
    sources: Vec<PackageDef>,
    depth: usize,
}

/// Merges all direct non-package AST members from `src` into `dest` and returns `src.packages`. Realises: [REQ-PARSER-GRAMMAR-MERGE-PACKAGE-DEFS] | [FLS: fls_66m4rnbssgig]
fn merge_package_direct_members(dest: &mut PackageDef, src: PackageDef) -> Vec<PackageDef> {
    if dest.doc.is_none() && src.doc.is_some() {
        dest.doc = src.doc;
    }
    if dest.parent_package.is_none() && src.parent_package.is_some() {
        dest.parent_package = src.parent_package;
    }
    for imp in src.imports {
        if !dest.imports.iter().any(|existing| {
            existing.path == imp.path
                && existing.is_wildcard == imp.is_wildcard
                && existing.is_recursive == imp.is_recursive
        }) {
            dest.imports.push(imp);
        }
    }
    dest.part_defs.extend(src.part_defs);
    dest.port_defs.extend(src.port_defs);
    dest.attribute_defs.extend(src.attribute_defs);
    dest.action_defs.extend(src.action_defs);
    dest.operation_defs.extend(src.operation_defs);
    dest.capability_defs.extend(src.capability_defs);
    dest.interaction_defs.extend(src.interaction_defs);
    dest.constraint_defs.extend(src.constraint_defs);
    dest.test_case_defs.extend(src.test_case_defs);
    dest.requirement_defs.extend(src.requirement_defs);
    dest.state_defs.extend(src.state_defs);
    dest.use_case_defs.extend(src.use_case_defs);
    dest.item_defs.extend(src.item_defs);
    dest.hazard_defs.extend(src.hazard_defs);
    dest.risk_defs.extend(src.risk_defs);
    dest.connection_defs.extend(src.connection_defs);
    dest.calc_defs.extend(src.calc_defs);
    for sat in src.satisfied_requirements {
        if !dest.satisfied_requirements.contains(&sat) {
            dest.satisfied_requirements.push(sat);
        }
    }
    src.packages
}

/// Iteratively dismantles nested `PackageDef` trees in O(1) call-stack space. Realises: [REQ-PARSER-GRAMMAR-MERGE-PACKAGE-DEFS] | [FLS: fls_66m4rnbssgig]
pub(crate) fn drop_packages_iteratively(mut stack: Vec<PackageDef>) {
    while let Some(mut pkg) = stack.pop() {
        stack.append(&mut pkg.packages);
    }
}

/// Merges all AST definitions from `src` into `dest` using an explicit iterative work-stack bounded by `MAX_PARSE_DEPTH`.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope -- Iterative multi-file `PackageDef` merging under DO-178C Level A (`BP-10`).
/// ## 2. Invariants & Mathematical Properties -- Preserves declaration order and deduplicates imports up to `MAX_PARSE_DEPTH` (`64`).
/// ## 3. Preconditions -- `dest` and `src` are valid `PackageDef` nodes.
/// ## 4. Postconditions -- Mutates `dest` in-place with merged members and child packages.
/// ## 5. Algorithmic Complexity Bounds -- Time: O(P + M); Space: O(P).
/// ## 6. Failure Modes & Fault Tolerance -- Dismantles truncated subtrees iteratively without stack overflow.
/// ## 7. Realization & Traceability -- Realises: [REQ-PARSER-GRAMMAR-MERGE-PACKAGE-DEFS] | [FLS: fls_66m4rnbssgig]
pub fn merge_package_defs(dest: &mut PackageDef, src: PackageDef) {
    let mut stack = vec![PackageMergeFrame {
        dest,
        sources: vec![src],
        depth: 0,
    }];
    while let Some(frame) = stack.pop() {
        if frame.depth >= MAX_PARSE_DEPTH {
            for mut src_pkg in frame.sources {
                drop_packages_iteratively(std::mem::take(&mut src_pkg.packages));
            }
            continue;
        }
        let mut child_packages = Vec::new();
        for src_pkg in frame.sources {
            child_packages.extend(merge_package_direct_members(frame.dest, src_pkg));
        }
        let next_depth = frame.depth.saturating_add(1);
        if next_depth >= MAX_PARSE_DEPTH {
            drop_packages_iteratively(child_packages);
            continue;
        }
        let mut pending_by_index: BTreeMap<usize, Vec<PackageDef>> = BTreeMap::new();
        for src_child in child_packages {
            if let Some(existing_idx) = frame
                .dest
                .packages
                .iter()
                .position(|p| p.name == src_child.name)
            {
                pending_by_index.entry(existing_idx).or_default().push(src_child);
            } else {
                frame.dest.packages.push(src_child);
            }
        }
        if !pending_by_index.is_empty() {
            for (idx, dest_child) in frame.dest.packages.iter_mut().enumerate().rev() {
                if let Some(child_sources) = pending_by_index.remove(&idx) {
                    stack.push(PackageMergeFrame {
                        dest: dest_child,
                        sources: child_sources,
                        depth: next_depth,
                    });
                }
            }
        }
    }
}

/// Returns `true` when `pkg` is a synthetic forest root holding child `packages` without direct structural definitions.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope -- Identifies synthetic multi-file forest roots (`SysML_Model` or empty name).
/// ## 2. Invariants & Mathematical Properties -- Pure O(1) predicate over `PackageDef` member slices.
/// ## 3. Preconditions -- `pkg` is a valid `PackageDef` reference.
/// ## 4. Postconditions -- Returns `true` iff `pkg` holds child packages and zero direct definitions.
/// ## 5. Algorithmic Complexity Bounds -- Time: O(1); Space: O(1).
/// ## 6. Failure Modes & Fault Tolerance -- Total pure function with zero panics.
/// ## 7. Realization & Traceability -- Realises: [REQ-PARSER-GRAMMAR-IS-SYNTHETIC-FOREST-ROOT] | [FLS: fls_66m4rnbssgig]
#[must_use]
pub fn is_synthetic_forest_root(pkg: &PackageDef) -> bool {
    (pkg.name.is_empty() || pkg.name == "SysML_Model")
        && !pkg.packages.is_empty()
        && pkg.attribute_defs.is_empty()
        && pkg.port_defs.is_empty()
        && pkg.action_defs.is_empty()
        && pkg.constraint_defs.is_empty()
        && pkg.calc_defs.is_empty()
        && pkg.requirement_defs.is_empty()
        && pkg.satisfied_requirements.is_empty()
        && pkg.state_defs.is_empty()
        && pkg.connection_defs.is_empty()
        && pkg.use_case_defs.is_empty()
        && pkg.item_defs.is_empty()
        && pkg.hazard_defs.is_empty()
        && pkg.risk_defs.is_empty()
        && pkg.test_case_defs.is_empty()
        && pkg.capability_defs.is_empty()
        && pkg.operation_defs.is_empty()
        && pkg.interaction_defs.is_empty()
        && pkg.part_defs.is_empty()
}

/// Parses multiple `.sysml` files in sorted path order and merges their package definitions into a unified root `PackageDef`.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope -- Multi-file SysML v2 workspace parser (`BP-10`).
/// ## 2. Invariants & Mathematical Properties -- Sorts `files` lexicographically and merges packages via `merge_package_defs`.
/// ## 3. Preconditions -- `files` is a non-empty slice of readable `.sysml` paths.
/// ## 4. Postconditions -- Returns `Ok(PackageDef)` or `Err(ParseError)`.
/// ## 5. Algorithmic Complexity Bounds -- Time: O(B) over total bytes `B`; Space: O(B).
/// ## 6. Failure Modes & Fault Tolerance -- Fails closed on empty file list, unreadable files, or syntax errors.
/// ## 7. Realization & Traceability -- Realises: [REQ-PARSER-GRAMMAR-PARSE-SYSML-TREE] | [FLS: fls_66m4rnbssgig]
pub fn parse_sysml_tree(files: &[PathBuf]) -> Result<PackageDef, ParseError> {
    if files.is_empty() {
        return Err(ParseError {
            message: "No .sysml files provided for tree parsing".to_string(),
            span: GRAMMAR_EOF_TOKEN.span,
        });
    }
    let mut sorted_files = files.to_vec();
    sorted_files.sort();
    let mut root = PackageDef {
        name: "SysML_Model".to_string(),
        ..Default::default()
    };
    for file in &sorted_files {
        let content = fs::read_to_string(file).map_err(|e| ParseError {
            message: format!("Failed to read SysML file '{}': {}", file.display(), e),
            span: Span::default(),
        })?;
        let parsed = SysmlParser::parse_source(&content, "SysML_Model")?;
        if parsed.name == "SysML_Model" || parsed.name == root.name {
            merge_package_defs(&mut root, parsed);
        } else if let Some(existing) = root.packages.iter_mut().find(|p| p.name == parsed.name) {
            merge_package_defs(existing, parsed);
        } else {
            root.packages.push(parsed);
        }
    }
    if root.packages.len() == 1
        && (is_synthetic_forest_root(&root) || root.name == "SysML_Model" || root.name.is_empty())
    {
        if let Some(mut single) = root.packages.pop() {
            merge_package_defs(&mut single, root);
            return Ok(single);
        }
    }
    Ok(root)
}

/// Discovers all `.sysml` files iteratively in `dir` and parses them into a unified root `PackageDef`.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope -- Directory-level SysML v2 ingestion entrypoint (`BP-10`).
/// ## 2. Invariants & Mathematical Properties -- Delegates discovery to `discover_sysml_files` and merging to `parse_sysml_tree`.
/// ## 3. Preconditions -- `dir` is a directory containing one or more `.sysml` files.
/// ## 4. Postconditions -- Returns `Ok(PackageDef)` or `Err(ParseError)`.
/// ## 5. Algorithmic Complexity Bounds -- Time: O(B); Space: O(B).
/// ## 6. Failure Modes & Fault Tolerance -- Returns `Err(ParseError)` when zero `.sysml` files exist in `dir`.
/// ## 7. Realization & Traceability -- Realises: [REQ-PARSER-GRAMMAR-PARSE-SYSML-DIRECTORY] | [FLS: fls_66m4rnbssgig]
pub fn parse_sysml_directory(dir: &Path) -> Result<PackageDef, ParseError> {
    let files = discover_sysml_files(dir);
    if files.is_empty() {
        return Err(ParseError {
            message: format!("No .sysml files found in directory '{}'", dir.display()),
            span: Span::default(),
        });
    }
    parse_sysml_tree(&files)
}
