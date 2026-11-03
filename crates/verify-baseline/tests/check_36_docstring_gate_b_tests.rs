//! Integration tests for Check 36: Live workspace per-item docstring gate and module resolver (Part B).

use std::collections::BTreeSet;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use verify_baseline::checks::{
    audit_rust_source_docstrings, audit_source_purity, check_docstring_completeness,
};
use walkdir::WalkDir;

const MAX_MODULE_WALK_DEPTH: usize = 16;

/// Resolves a target `.rs` file or its decomposed `<stem>/` module directory into sorted `.rs` paths.
fn resolve_module_rs_files(path: &Path) -> Vec<PathBuf> {
    if path.is_file() {
        return vec![path.to_path_buf()];
    }
    let dir_path = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.with_extension("")
    };
    if !dir_path.is_dir() {
        return Vec::new();
    }
    let mut collected = BTreeSet::new();
    let mut stack = vec![(dir_path, 0usize)];
    while let Some((current_dir, depth)) = stack.pop() {
        if depth > MAX_MODULE_WALK_DEPTH {
            continue;
        }
        let Ok(entries) = fs::read_dir(&current_dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let entry_path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                let next_depth = depth.saturating_add(1);
                if next_depth <= MAX_MODULE_WALK_DEPTH {
                    stack.push((entry_path, next_depth));
                }
            } else if file_type.is_file()
                && entry_path.extension().and_then(|ext| ext.to_str()) == Some("rs")
            {
                collected.insert(entry_path);
            }
        }
    }
    collected.into_iter().collect()
}

struct ScopedTempWorkspace {
    root: PathBuf,
}

impl ScopedTempWorkspace {
    fn new(prefix: &str) -> Result<Self, Box<dyn Error>> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!(
            "deap_check36_b_{}_{}_{}",
            prefix,
            std::process::id(),
            nanos
        ));
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }
}

impl Drop for ScopedTempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Verifies key target modules satisfy Check 34 source purity and Check 36 Rustdoc completeness.
fn assert_target_modules_pure_and_documented(repo_root: &Path) -> Result<(), Box<dyn Error>> {
    for rel_mod in [
        "crates/verify-baseline/src/checks/docstring_gate.rs",
        "crates/deap-core/src/source_ast.rs",
        "crates/compile-sysml/src/docstring.rs",
        "crates/ingest-sysml/src/lib.rs",
    ] {
        let resolved_files = resolve_module_rs_files(&repo_root.join(rel_mod));
        assert!(
            !resolved_files.is_empty(),
            "Expected {} (or decomposed module directory) to resolve to at least one .rs file",
            rel_mod
        );
        for file_path in resolved_files {
            let rel_file = file_path
                .strip_prefix(repo_root)
                .unwrap_or(&file_path)
                .display()
                .to_string();
            let mod_src = fs::read_to_string(&file_path)?;
            let purity_res = audit_source_purity(&mod_src, &rel_file, true);
            assert!(
                purity_res.is_ok(),
                "{} failed Check 34 source purity: {:?}",
                rel_file,
                purity_res.err()
            );
            let doc_res = audit_rust_source_docstrings(&mod_src, &rel_file);
            assert!(
                doc_res.is_ok(),
                "{} failed Check 36 per-item docstring gate: {:?}",
                rel_file,
                doc_res.err()
            );
        }
    }

    let runner_files =
        resolve_module_rs_files(&repo_root.join("crates/verify-baseline/src/runner.rs"));
    assert!(
        !runner_files.is_empty(),
        "Expected crates/verify-baseline/src/runner.rs (or runner/ directory) to resolve to at least one .rs file"
    );
    for runner_file in runner_files {
        let rel_runner = runner_file
            .strip_prefix(repo_root)
            .unwrap_or(&runner_file)
            .display()
            .to_string();
        let runner_src = fs::read_to_string(&runner_file)?;
        let runner_doc_res = audit_rust_source_docstrings(&runner_src, &rel_runner);
        assert!(
            runner_doc_res.is_ok(),
            "{} failed Check 36 per-item docstring gate: {:?}",
            rel_runner,
            runner_doc_res.err()
        );
    }
    Ok(())
}

#[test]
fn test_live_workspace_crates_have_100_percent_per_item_7_section_docstrings() -> Result<(), Box<dyn Error>> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .ok_or("failed to resolve repo_root from CARGO_MANIFEST_DIR")?;

    assert_target_modules_pure_and_documented(repo_root)?;

    let crates_dir = repo_root.join("crates");
    let mut total_files = 0usize;
    let mut total_items = 0usize;
    let mut all_violations = Vec::new();

    for entry_res in WalkDir::new(&crates_dir).into_iter().filter_entry(|e| {
        if e.file_type().is_dir() {
            let name = e.file_name().to_string_lossy();
            !matches!(name.as_ref(), "target" | ".git" | "tests" | "benches" | "examples")
        } else {
            true
        }
    }) {
        let entry = entry_res?;
        if entry.file_type().is_file() && entry.path().extension().is_some_and(|ext| ext == "rs") {
            total_files += 1;
            let rel = entry.path().strip_prefix(repo_root).unwrap_or(entry.path()).display().to_string();
            let content = fs::read_to_string(entry.path())?;
            match audit_rust_source_docstrings(&content, &rel) {
                Ok(count) => total_items += count,
                Err(errs) => all_violations.extend(errs),
            }
        }
    }

    assert!(total_files >= 40, "Expected at least 40 non-test .rs files across crates/, scanned {}", total_files);
    assert!(
        all_violations.is_empty(),
        "Live workspace crates have {} Check 36 per-item 7-section docstring violation(s) (first 25 shown):\n{}",
        all_violations.len(),
        all_violations.iter().take(25).cloned().collect::<Vec<_>>().join("\n")
    );
    assert!(total_items >= 200, "Expected at least 200 documented Rust items across workspace crates, verified {}", total_items);

    let full_gate = check_docstring_completeness(repo_root);
    assert!(full_gate.is_ok(), "check_docstring_completeness failed on live workspace: {:?}", full_gate.err());
    Ok(())
}

/// Verifies `resolve_module_rs_files` resolves both a single `foo.rs` file and a decomposed `foo/` module directory within bounded depth.
#[test]
fn test_resolve_module_rs_files_single_file_and_decomposed_directory() -> Result<(), Box<dyn Error>>
{
    let ws = ScopedTempWorkspace::new("resolve_module_rs")?;
    let src_dir = ws.root.join("src");
    fs::create_dir_all(&src_dir)?;

    let single_foo = src_dir.join("foo.rs");
    fs::write(&single_foo, "// single module file\n")?;
    assert_eq!(
        resolve_module_rs_files(&single_foo),
        vec![single_foo.clone()],
        "Single foo.rs file must resolve to vec![foo.rs]"
    );

    fs::remove_file(&single_foo)?;
    let foo_dir = src_dir.join("foo");
    let nested_dir = foo_dir.join("nested");
    fs::create_dir_all(&nested_dir)?;
    let mod_rs = foo_dir.join("mod.rs");
    let sub_rs = foo_dir.join("sub.rs");
    let leaf_rs = nested_dir.join("leaf.rs");
    fs::write(&sub_rs, "// sub\n")?;
    fs::write(&mod_rs, "// mod\n")?;
    fs::write(&leaf_rs, "// leaf\n")?;
    fs::write(foo_dir.join("notes.md"), "ignored non-rust file\n")?;

    assert_eq!(
        resolve_module_rs_files(&single_foo),
        vec![mod_rs, leaf_rs, sub_rs],
        "Decomposed foo/ directory must resolve all .rs submodules in deterministic sorted order"
    );

    let bar_rs = src_dir.join("bar.rs");
    let mut cur_dir = src_dir.join("bar");
    for level in 1..=16usize {
        cur_dir = cur_dir.join(format!("d{level:02}"));
    }
    fs::create_dir_all(&cur_dir)?;
    let depth_16_file = cur_dir.join("at_max_depth.rs");
    fs::write(&depth_16_file, "// depth 16\n")?;

    let depth_17_dir = cur_dir.join("d17");
    fs::create_dir_all(&depth_17_dir)?;
    let depth_17_file = depth_17_dir.join("beyond_max_depth.rs");
    fs::write(&depth_17_file, "// depth 17\n")?;

    let bar_resolved = resolve_module_rs_files(&bar_rs);
    assert_eq!(
        bar_resolved,
        vec![depth_16_file],
        "Resolver must include depth <= 16 and exclude depth 17"
    );

    assert!(
        resolve_module_rs_files(&src_dir.join("missing.rs")).is_empty(),
        "Missing module path must return empty Vec"
    );
    Ok(())
}
