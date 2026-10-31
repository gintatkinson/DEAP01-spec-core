//! Multi-file SysML v2 directory discovery, iterative depth-bounded package merging, and forest assembly.
//!
//! Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-MODULE]
//! Contract: Discovers `.sysml` files iteratively under workspace directories, merges multi-file `PackageDef` hierarchies in `O(1)` call-stack space up to `MAX_PARSE_DEPTH` (64), and collapses single-package synthetic forest roots deterministically.
//! Bounds & Faults: Uses explicit `Vec<PathBuf>` and `Vec<PackageMergeFrame>` work-stacks with lexicographical `.sort()` ordering, zero call-stack recursion, and typed `ParseError` diagnostics on empty or unreadable file sets.

use super::*;

/// Returns `true` when directory basename `name` is a build, dependency, or generated artifact directory excluded from SysML discovery.
/// Realises: [REQ-PARSER-GRAMMAR-DISCOVER-SYSML-FILES]
fn is_excluded_sysml_dir(name: &str) -> bool {
    matches!(
        name,
        "target"
            | "node_modules"
            | "build"
            | "Pods"
            | "vendor"
            | "dist"
            | "ephemeral"
            | "generated"
    )
}

/// Discovers all `.sysml` files under `dir` iteratively using an explicit `Vec<PathBuf>` work-stack, filtering out hidden, temporary, and build-cache paths.
///
/// Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-DISCOVER-SYSML-FILES]
/// Contract: Accepts a root directory `dir` (including `.`-prefixed roots at depth 0), prunes child entries prefixed with `.` or `#` and excluded build directories, and returns a lexicographically sorted `Vec<PathBuf>` of `.sysml` files.
/// Bounds & Faults: Executes iteratively in `O(1)` call-stack space via an explicit `Vec<PathBuf>` directory stack without following directory symlinks; returns an empty vector when `dir` is not an accessible directory.
pub fn discover_sysml_files(dir: &Path) -> Vec<PathBuf> {
    if !dir.is_dir() {
        return Vec::new();
    }

    let mut dir_stack: Vec<PathBuf> = vec![dir.to_path_buf()];
    let mut files = Vec::new();

    while let Some(current_dir) = dir_stack.pop() {
        let Ok(entries) = fs::read_dir(&current_dir) else {
            continue;
        };
        let mut child_dirs = Vec::new();
        for entry_res in entries {
            let Ok(entry) = entry_res else {
                continue;
            };
            let raw_name = entry.file_name();
            let file_name = raw_name.to_string_lossy();
            if file_name.starts_with('.') || file_name.starts_with('#') {
                continue;
            }
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            let path = entry.path();
            if file_type.is_dir() {
                if !is_excluded_sysml_dir(file_name.as_ref()) {
                    child_dirs.push(path);
                }
            } else if file_type.is_file() && path.extension().is_some_and(|ext| ext == "sysml") {
                files.push(path);
            }
        }
        child_dirs.sort();
        dir_stack.extend(child_dirs);
    }

    files.sort();
    files
}

/// Work-stack frame for bounded iterative child-package merging in `merge_package_defs`.
/// Realises: [REQ-PARSER-GRAMMAR-MERGE-PACKAGE-DEFS]
struct PackageMergeFrame<'a> {
    dest: &'a mut PackageDef,
    sources: Vec<PackageDef>,
    depth: usize,
}

/// Merges all direct non-package AST members from `src` into `dest` and returns `src.packages`.
/// Realises: [REQ-PARSER-GRAMMAR-MERGE-PACKAGE-DEFS]
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

/// Iteratively dismantles nested `PackageDef` trees in O(1) call-stack space so dropping
/// truncated subtrees beyond `MAX_PARSE_DEPTH` cannot trigger recursive `Drop` stack overflow.
/// Realises: [REQ-PARSER-GRAMMAR-MERGE-PACKAGE-DEFS]
pub(crate) fn drop_packages_iteratively(mut stack: Vec<PackageDef>) {
    while let Some(mut pkg) = stack.pop() {
        stack.append(&mut pkg.packages);
    }
}

/// Merges all AST definitions from `src` into `dest` using an explicit iterative work-stack bounded by `MAX_PARSE_DEPTH`.
///
/// Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-MERGE-PACKAGE-DEFS]
/// Contract: Preserves deterministic declaration ordering and import/satisfaction deduplication across direct definitions and nested child packages up to `MAX_PARSE_DEPTH` (64) levels.
/// Bounds & Faults: Executes in `O(1)` call-stack space via `Vec<PackageMergeFrame>`, halting descent at `depth >= MAX_PARSE_DEPTH` and dismantling truncated subtrees iteratively with zero panics.
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
                pending_by_index
                    .entry(existing_idx)
                    .or_default()
                    .push(src_child);
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

/// Returns `true` when `pkg` is a synthetic multi-package or single-package forest root (`SysML_Model` or empty name) holding child `packages` without direct structural definitions.
///
/// Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-IS-SYNTHETIC-FOREST-ROOT]
/// Contract: Inspects `pkg.name`, `pkg.packages`, and all direct member slices without mutating `pkg`.
/// Bounds & Faults: Executes in `O(1)` time and `O(1)` space as a total pure predicate.
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
///
/// Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-PARSE-SYSML-TREE]
/// Contract: Sorts `files` lexicographically, parses each file via `SysmlParser::parse_source`, merges matching root/child packages via `merge_package_defs`, and collapses single-child synthetic roots.
/// Bounds & Faults: Fails closed with a typed `ParseError` when `files` is empty, any file cannot be read, or any file contains invalid SysML v2 syntax.
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
///
/// Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-PARSE-SYSML-DIRECTORY]
/// Contract: Delegates file discovery to `discover_sysml_files` and AST assembly to `parse_sysml_tree`.
/// Bounds & Faults: Returns a typed `ParseError` when `dir` contains zero `.sysml` files or any discovered file fails lexical/syntactic validation.
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
