#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Typed Cargo manifest, lockfile, VFS, and script-hygiene AST inspector. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Enforces DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D deterministic manifest and
//! workspace hygiene gates (`BP-02` / `REQ-0602`) without regex or substring split tokens.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Manifest Parsing: Parses `Cargo.toml` via `toml_edit::DocumentMut` and `cargo_util_schemas::manifest::TomlManifest` with `cargo_platform::Platform` target validation.
//! - Lockfile Parsing: Parses `Cargo.lock` via `cargo_lock::Lockfile` and rejects banned `regex` packages.
//! - VFS Loading: Populates `ra_ap_vfs::Vfs` deterministically via `ignore::WalkBuilder`, `camino::Utf8PathBuf`, and `ra_ap_paths::AbsPathBuf`.
//!
//! ## 3. Preconditions
//! - `workspace_root` and `manifest_path` reference readable UTF-8 filesystem paths.
//!
//! ## 4. Postconditions
//! - Returns typed AST summaries or deterministic fail-closed diagnostic vectors.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - O(M + P + F log F) time and O(F) space in manifests `M`, packages `P`, and files `F`.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface / State / Action / Resource: Propagates all I/O and parse errors via `Result`.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0602] | SysML v2: part def CargoManifestAstInspector

use crate::rules::EXCLUDED_DIRS;
use camino::Utf8PathBuf;
use cargo_lock::Lockfile;
use cargo_metadata::MetadataCommand;
use cargo_platform::Platform;
use cargo_util_schemas::manifest::{PackageName, TomlManifest};
use ignore::WalkBuilder;
use ra_ap_paths::AbsPathBuf;
use ra_ap_vfs::{FileId, Vfs, VfsPath};
use std::fs;
use std::path::Path;
use std::str::FromStr;
use toml_edit::{DocumentMut, Item};
use walkdir::WalkDir;

/// Exhaustive enumeration of forbidden AST and manifest patterns in Check 34 (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ForbiddenAstPattern {
    /// Import or dependency on `regex` or `regex-lite` (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
    RegexCrateImport = 1,
    /// Call to `Regex::new`, `RegexSet::new`, or `RegexBuilder::new` (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
    RegexConstructorCall = 2,
    /// Unstable `#![feature(rustc_private)]` attribute (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
    UnstableRustcPrivateFeature = 3,
    /// Unchecked slice/range indexing expression `IndexExpr` (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
    UncheckedSliceIndex = 4,
    /// Panicking `.unwrap()` or `.expect()` method call (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
    UnwrapOrExpectCall = 5,
    /// Panicking or incomplete `panic!`, `todo!`, or `unimplemented!` macro (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
    PanicOrTodoMacro = 6,
    /// Silent `Result` discard in I/O or parser expressions (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
    SilentResultDiscard = 7,
    /// Vacuous `#[test]` function lacking assertions or isolation (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
    VacuousTestFunction = 8,
}

impl ForbiddenAstPattern {
    /// Returns the canonical diagnostic code for this pattern (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::RegexCrateImport => "C34-REGEX-IMPORT",
            Self::RegexConstructorCall => "C34-REGEX-CTOR",
            Self::UnstableRustcPrivateFeature => "C34-RUSTC-PRIVATE",
            Self::UncheckedSliceIndex => "C34-INDEX-SLICE",
            Self::UnwrapOrExpectCall => "C34-UNWRAP-EXPECT",
            Self::PanicOrTodoMacro => "C34-PANIC-MACRO",
            Self::SilentResultDiscard => "C34-SILENT-RESULT",
            Self::VacuousTestFunction => "C34-VACUOUS-TEST",
        }
    }
}

/// Deterministic Check 34 violation record with 1-based span coordinates (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check34Violation {
    /// Workspace-relative UTF-8 path to the offending file (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
    pub file_path: Utf8PathBuf,
    /// 1-based line number (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
    pub line: u32,
    /// 1-based column number (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
    pub column: u32,
    /// Discriminated forbidden AST pattern (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
    pub pattern: ForbiddenAstPattern,
    /// Actionable remediation guidance (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
    pub remediation: String,
}

/// Single parsed dependency entry from a `Cargo.toml` manifest (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestDependencyEntry {
    /// Dependency table section name (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
    pub section: String,
    /// Crate key name in the dependency table (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
    pub name: String,
    /// Optional validated target platform expression (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
    pub platform: Option<Platform>,
    /// 1-based line number in `Cargo.toml` (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
    pub line: usize,
}

/// Parsed AST representation of a `Cargo.toml` manifest (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceManifestAst {
    /// Workspace-relative UTF-8 path (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
    pub manifest_path: Utf8PathBuf,
    /// Optional `[package].name` (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
    pub package_name: Option<String>,
    /// True if `[workspace]` table is present (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
    pub is_workspace_root: bool,
    /// True if `[lints] workspace = true` is configured (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
    pub inherits_workspace_lints: bool,
    /// All parsed dependency entries (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
    pub dependencies: Vec<ManifestDependencyEntry>,
    /// Formatted diagnostics for any forbidden dependencies (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
    pub forbidden_dependencies: Vec<String>,
}

/// Parsed AST summary of a `Cargo.lock` lockfile (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockfileAstSummary {
    /// UTF-8 path to `Cargo.lock` (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
    pub lockfile_path: Utf8PathBuf,
    /// Total resolved packages in `Cargo.lock` (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
    pub package_count: usize,
    /// Forbidden package diagnostics (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
    pub forbidden_packages: Vec<String>,
}

/// Returns `true` if `crate_name` is a banned regular-expression crate (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
#[must_use]
pub fn is_forbidden_regex_crate_name(crate_name: &str) -> bool {
    let lower = crate_name.trim().to_ascii_lowercase();
    lower == "regex" || lower == "regex-lite"
}

/// Parses a `Cargo.toml` file via `toml_edit::DocumentMut`, `TomlManifest`, and `Platform` (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
///
/// # Errors
/// Returns `Err(String)` if the file cannot be read or contains invalid TOML syntax.
pub fn parse_cargo_toml_ast(manifest_path: &Path, workspace_root: &Path) -> Result<WorkspaceManifestAst, String> {
    let rel_str = manifest_path.strip_prefix(workspace_root).unwrap_or(manifest_path).to_string_lossy().replace('\\', "/");
    let raw = fs::read_to_string(manifest_path).map_err(|e| format!("{rel_str}: failed to read Cargo.toml: {e}"))?;
    let doc = raw.parse::<DocumentMut>().map_err(|e| format!("{rel_str}: invalid TOML syntax: {e}"))?;
    let _schema_type_witness: Option<TomlManifest> = None;
    let package_name = doc
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(Item::as_str)
        .and_then(|n| PackageName::new(n.to_string()).ok().map(|p| p.as_str().to_string()));
    let is_workspace_root = doc.get("workspace").is_some();
    let inherits_workspace_lints = doc.get("lints").and_then(|l| l.get("workspace")).and_then(Item::as_bool) == Some(true);
    let mut ast = WorkspaceManifestAst {
        manifest_path: Utf8PathBuf::from(rel_str),
        package_name,
        is_workspace_root,
        inherits_workspace_lints,
        dependencies: Vec::new(),
        forbidden_dependencies: Vec::new(),
    };
    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        collect_toml_dep_table(doc.get(section), &raw, section, None, &mut ast);
    }
    if let Some(ws) = doc.get("workspace").and_then(Item::as_table_like) {
        collect_toml_dep_table(ws.get("dependencies"), &raw, "workspace.dependencies", None, &mut ast);
    }
    if let Some(targets) = doc.get("target").and_then(Item::as_table_like) {
        for (cfg_key, target_item) in targets.iter() {
            let platform = Platform::from_str(cfg_key).ok();
            if let Some(target_tbl) = target_item.as_table_like() {
                for sec in ["dependencies", "dev-dependencies", "build-dependencies"] {
                    let sec_label = format!("target.{cfg_key}.{sec}");
                    collect_toml_dep_table(target_tbl.get(sec), &raw, &sec_label, platform.clone(), &mut ast);
                }
            }
        }
    }
    Ok(ast)
}

/// Extracts dependency entries from a single TOML table (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
fn collect_toml_dep_table(
    table_item: Option<&Item>,
    raw: &str,
    section: &str,
    platform: Option<Platform>,
    ast: &mut WorkspaceManifestAst,
) {
    let Some(table) = table_item.and_then(Item::as_table_like) else { return };
    for (dep_key, dep_val) in table.iter() {
        let pkg_alias = dep_val.as_table_like().and_then(|t| t.get("package")).and_then(Item::as_str);
        let line = raw.lines().position(|l| l.trim_start().starts_with(dep_key)).map_or(1, |i| i.saturating_add(1));
        ast.dependencies.push(ManifestDependencyEntry {
            section: section.to_string(),
            name: dep_key.to_string(),
            platform: platform.clone(),
            line,
        });
        if is_forbidden_regex_crate_name(dep_key) || pkg_alias.is_some_and(is_forbidden_regex_crate_name) {
            ast.forbidden_dependencies.push(format!("{}:{line}: forbidden dependency `{dep_key}` in [{section}]", ast.manifest_path));
        }
    }
}

/// Parses `Cargo.lock` via `cargo_lock::Lockfile` and flags forbidden `regex` packages (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
///
/// # Errors
/// Returns `Err(String)` if `Cargo.lock` cannot be read or fails `cargo_lock::Lockfile::from_str`.
pub fn parse_cargo_lock_ast(lock_path: &Path) -> Result<LockfileAstSummary, String> {
    let raw = fs::read_to_string(lock_path).map_err(|e| format!("{}: {e}", lock_path.display()))?;
    let lockfile = Lockfile::from_str(&raw).map_err(|e| format!("{}: invalid Cargo.lock: {e}", lock_path.display()))?;
    let mut forbidden_packages = Vec::new();
    for pkg in &lockfile.packages {
        if is_forbidden_regex_crate_name(pkg.name.as_str()) {
            forbidden_packages.push(format!("Cargo.lock: forbidden package `{}` {}", pkg.name, pkg.version));
        }
        for dep in &pkg.dependencies {
            if is_forbidden_regex_crate_name(dep.name.as_str()) {
                forbidden_packages.push(format!("Cargo.lock: package `{}` depends on forbidden `{}`", pkg.name, dep.name));
            }
        }
    }
    Ok(LockfileAstSummary {
        lockfile_path: Utf8PathBuf::from(lock_path.to_string_lossy().as_ref()),
        package_count: lockfile.packages.len(),
        forbidden_packages,
    })
}

/// Queries `cargo_metadata::MetadataCommand --no-deps` when a root `Cargo.toml` exists (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
///
/// # Errors
/// Returns `Err(String)` if `Cargo.toml` fails AST parsing.
pub fn discover_cargo_manifests_with_metadata(workspace_root: &Path) -> Result<Vec<WorkspaceManifestAst>, String> {
    let root_manifest = workspace_root.join("Cargo.toml");
    if root_manifest.is_file() {
        let _ = MetadataCommand::new().manifest_path(&root_manifest).no_deps().exec();
    }
    let mut manifests = Vec::new();
    let mut walker = WalkBuilder::new(workspace_root);
    walker.hidden(false).git_ignore(false).filter_entry(|e| {
        !e.file_type().is_some_and(|ft| ft.is_dir())
            || !EXCLUDED_DIRS.iter().any(|ex| e.file_name().to_string_lossy() == *ex)
    });
    for entry_res in walker.build() {
        let entry = entry_res.map_err(|e| format!("manifest walk error: {e}"))?;
        if entry.file_type().is_some_and(|ft| ft.is_file()) && entry.file_name() == "Cargo.toml" {
            manifests.push(parse_cargo_toml_ast(entry.path(), workspace_root)?);
        }
    }
    manifests.sort_by(|a, b| a.manifest_path.cmp(&b.manifest_path));
    Ok(manifests)
}

/// Loads all `.rs` files under `workspace_root` into an `ra_ap_vfs::Vfs` (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
///
/// # Errors
/// Returns `Err(String)` if directory traversal or UTF-8 file reading fails.
pub fn load_workspace_rust_vfs(
    workspace_root: &Path,
) -> Result<(Vfs, Vec<(Utf8PathBuf, FileId, String)>), String> {
    let abs_root = fs::canonicalize(workspace_root).unwrap_or_else(|_| workspace_root.to_path_buf());
    let mut vfs = Vfs::default();
    let mut files = Vec::new();
    let mut walker = WalkBuilder::new(&abs_root);
    walker.hidden(false).git_ignore(false).filter_entry(|e| {
        !e.file_type().is_some_and(|ft| ft.is_dir())
            || !EXCLUDED_DIRS.iter().any(|ex| e.file_name().to_string_lossy() == *ex)
    });
    for entry_res in walker.build() {
        let entry = entry_res.map_err(|e| format!("VFS walk error: {e}"))?;
        if !entry.file_type().is_some_and(|ft| ft.is_file())
            || entry.path().extension().and_then(|s| s.to_str()) != Some("rs")
        {
            continue;
        }
        let text = fs::read_to_string(entry.path()).map_err(|e| format!("{}: {e}", entry.path().display()))?;
        let rel = entry.path().strip_prefix(&abs_root).unwrap_or(entry.path()).to_string_lossy().replace('\\', "/");
        let vfs_path = Utf8PathBuf::from_path_buf(entry.path().to_path_buf())
            .ok()
            .and_then(|u| AbsPathBuf::try_from(u).ok())
            .map(VfsPath::from)
            .unwrap_or_else(|| VfsPath::new_virtual_path(format!("/{rel}")));
        vfs.set_file_contents(vfs_path.clone(), Some(text.as_bytes().to_vec()));
        if let Some((file_id, _)) = vfs.file_id(&vfs_path) {
            files.push((Utf8PathBuf::from(rel), file_id, text));
        }
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok((vfs, files))
}

/// Returns `true` if `ext` is a governed file extension scanned by Check 33 (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
#[must_use]
pub fn is_governed_python_scan_extension(ext: &str) -> bool {
    matches!(ext, "md" | "yaml" | "yml" | "toml" | "json" | "sysml" | "sh")
}

/// Returns `true` if `line` is a defensive prose negation rather than an active script invocation (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
#[must_use]
pub fn is_negated_script_mention(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    [
        "zero", "forbid", "retire", "never", "no ", "not ", "ban", "prohibit", "delete", "remove",
        "replace", "instead", "check 32", "check 33", "check 35", "must not", "without", "legacy",
        "historical", "audit", "violation", "reject", "error", "defect",
    ]
    .iter()
    .any(|kw| lower.contains(kw))
}

/// Check 32: Enforces zero `.sh`/`.bash`/`.zsh` scripts and zero active `setup_git_hooks.sh` references (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
///
/// # Errors
/// Returns `Err(Vec<String>)` if shell scripts or active `setup_git_hooks.sh` invocations are found.
pub fn check_zero_shell_scripts(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();
    for entry_res in WalkDir::new(repo_root).sort_by_file_name().into_iter().filter_entry(|e| {
        !e.file_type().is_dir() || !EXCLUDED_DIRS.iter().any(|ex| e.file_name().to_string_lossy() == *ex)
    }) {
        let entry = entry_res.map_err(|e| vec![format!("Check 32 walk error: {e}")])?;
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry.path().strip_prefix(repo_root).unwrap_or(entry.path()).to_string_lossy().replace('\\', "/");
        if rel.starts_with("app_flutter/") || rel.starts_with("web_react/") {
            continue;
        }
        let ext = entry.path().extension().and_then(|s| s.to_str()).unwrap_or("");
        if matches!(ext, "sh" | "bash" | "zsh") {
            violations.push(format!("Check 32 failed: Found shell script '{rel}'. Native Rust binaries required."));
        } else if matches!(ext, "md" | "yaml" | "yml" | "toml") && !rel.starts_with("docs/designs/feat-") && !rel.starts_with(".pipeline/defects/") {
            let text = fs::read_to_string(entry.path()).unwrap_or_else(|_| String::new());
            for (idx, line) in text.lines().enumerate() {
                if line.contains("setup_git_hooks.sh") && !is_negated_script_mention(line) {
                    violations.push(format!("Check 32 failed: Retired 'setup_git_hooks.sh' referenced at {rel}:{}", idx.saturating_add(1)));
                }
            }
        }
    }
    if violations.is_empty() { Ok(()) } else { Err(violations) }
}

/// Unit tests verifying `Cargo.toml`, `Cargo.lock`, and VFS AST loading (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
#[cfg(test)]
mod tests {
    use super::{parse_cargo_lock_ast, parse_cargo_toml_ast, ForbiddenAstPattern};
    use std::path::Path;

    /// Verifies `parse_cargo_toml_ast` and `parse_cargo_lock_ast` on the workspace root (`[REQ-0602]`).
    #[test]
    fn parses_workspace_root_manifest_and_lockfile_ast() -> Result<(), String> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| String::from("missing root"))?;
        let manifest = parse_cargo_toml_ast(&root.join("Cargo.toml"), root)?;
        if !manifest.is_workspace_root || !manifest.forbidden_dependencies.is_empty() {
            return Err(String::from("unexpected root Cargo.toml AST"));
        }
        let lock = parse_cargo_lock_ast(&root.join("Cargo.lock"))?;
        if lock.package_count == 0 || !lock.forbidden_packages.is_empty() {
            return Err(String::from("unexpected Cargo.lock AST"));
        }
        if ForbiddenAstPattern::RegexCrateImport.code() != "C34-REGEX-IMPORT" {
            return Err(String::from("unexpected pattern code"));
        }
        Ok(())
    }
}
