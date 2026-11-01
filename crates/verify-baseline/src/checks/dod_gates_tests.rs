#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Unit tests for `dod_gates` covering ratchet overflow, SLOC, fn bounds, lexer bans, and literal portability.
//! Realises: [REQ-0601], [REQ-0608]

use super::dod_gates::{check_dod_gates, count_executable_sloc};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Auto-cleaned temporary workspace directory under `std::env::temp_dir()` (`[REQ-0601]`).
struct ScopedTempDir {
    path: PathBuf,
}

impl ScopedTempDir {
    /// Creates an isolated PID- and sequence-tagged directory in `std::env::temp_dir()` (`[REQ-0601]`).
    fn new(label: &str) -> Result<Self, String> {
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let id = SEQ.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("dod_gates_{label}_{}_{id}", std::process::id()));
        fs::create_dir_all(&path).map_err(|e| e.to_string())?;
        Ok(Self { path })
    }

    /// Writes `source` to a `/` relative path inside this temporary workspace (`[REQ-0601]`).
    fn write_rel(&self, rel: &str, source: &str) -> Result<(), String> {
        let target = rel.split('/').fold(self.path.clone(), |acc, seg| acc.join(seg));
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(&target, source).map_err(|e| e.to_string())
    }
}

impl Drop for ScopedTempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Resolves the repository workspace root from `CARGO_MANIFEST_DIR` (`[REQ-0601]`).
fn workspace_root() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| String::from("unable to resolve workspace root"))
}

/// Verifies RED/GREEN `+1` line ratchet ceiling enforcement on ratcheted paths (`[REQ-0601]`, `[REQ-0608]`).
#[test]
fn ratchet_ceiling_exact_passes_and_plus_one_fails() -> Result<(), String> {
    let ws = ScopedTempDir::new("ratchet")?;
    let rel = "crates/compile-sysml/src/stpa/safety_suite/tokens.rs";
    let exact_175 = "// line\n".repeat(175);
    ws.write_rel(rel, &exact_175)?;
    check_dod_gates(&ws.path).map_err(|e| format!("expected 175 lines GREEN, got {e:?}"))?;

    let overflow_176 = "// line\n".repeat(176);
    ws.write_rel(rel, &overflow_176)?;
    match check_dod_gates(&ws.path) {
        Err(errs) if errs.iter().any(|m| m.contains("176") && m.contains("175")) => Ok(()),
        other => Err(format!("expected +1 ratchet overflow error, got {other:?}")),
    }
}

/// Verifies RED/GREEN `> 500` executable SLOC enforcement while excluding `///` doc attributes (`[REQ-0601]`).
#[test]
fn module_sloc_excludes_docs_and_rejects_over_500() -> Result<(), String> {
    let ws = ScopedTempDir::new("sloc")?;
    let mut green_src = String::new();
    for idx in 0..90_u32 {
        let _ = writeln!(green_src, "/// Doc line {idx}");
    }
    for idx in 0..490_u32 {
        let _ = writeln!(green_src, "pub const K_{idx}: u32 = {idx};");
    }
    ws.write_rel("crates/demo/src/lib.rs", &green_src)?;
    check_dod_gates(&ws.path).map_err(|e| format!("expected 490 exec SLOC GREEN, got {e:?}"))?;

    let mut red_src = String::new();
    for idx in 0..501_u32 {
        let _ = writeln!(red_src, "pub const K_{idx}: u32 = {idx};");
    }
    ws.write_rel("crates/demo/src/lib.rs", &red_src)?;
    match check_dod_gates(&ws.path) {
        Err(errs) if errs.iter().any(|m| m.contains("501") && m.contains("500")) => Ok(()),
        other => Err(format!("expected >500 SLOC error, got {other:?}")),
    }
}

/// Verifies RED/GREEN `<= 80`-line function span and `<= 5`-parameter bounds (`[REQ-0601]`).
#[test]
fn function_span_and_parameter_bounds_red_green() -> Result<(), String> {
    let ws = ScopedTempDir::new("fn_bounds")?;
    let body_78 = "    let x = 1_i32;\n".repeat(78);
    let fn_80 = format!("pub fn ok_span() {{\n{body_78}}}\n");
    ws.write_rel("crates/demo/src/span.rs", &fn_80)?;
    check_dod_gates(&ws.path).map_err(|e| format!("expected 80-line fn GREEN, got {e:?}"))?;

    let body_79 = "    let x = 1_i32;\n".repeat(79);
    let fn_81 = format!("pub fn over_span() {{\n{body_79}}}\n");
    ws.write_rel("crates/demo/src/span.rs", &fn_81)?;
    if !matches!(check_dod_gates(&ws.path), Err(ref e) if e.iter().any(|m| m.contains("over_span") && m.contains("81"))) {
        return Err(String::from("expected 81-line fn RED failure"));
    }
    ws.write_rel("crates/demo/src/span.rs", "pub fn ok_params(a: i32, b: i32, c: i32, d: i32, e: i32) -> i32 { a + b + c + d + e }\n")?;
    check_dod_gates(&ws.path).map_err(|e| format!("expected 5-param fn GREEN, got {e:?}"))?;

    ws.write_rel("crates/demo/src/span.rs", "pub fn bad_params(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32) -> i32 { a + b + c + d + e + f }\n")?;
    match check_dod_gates(&ws.path) {
        Err(errs) if errs.iter().any(|m| m.contains("bad_params") && m.contains("6 parameters")) => Ok(()),
        other => Err(format!("expected >5-param fn error, got {other:?}")),
    }
}

/// Verifies RED/GREEN hand-rolled lexer ban for `.chars().peekable()` and `.char_indices()` (`[REQ-0608]`).
#[test]
fn hand_rolled_lexer_calls_fail_without_logos_and_pass_with_logos() -> Result<(), String> {
    let ws = ScopedTempDir::new("lexer")?;
    ws.write_rel("crates/demo/src/peek.rs", "pub fn scan(s: &str) -> usize { s.chars().peekable().count() }\n")?;
    if !matches!(check_dod_gates(&ws.path), Err(ref e) if e.iter().any(|m| m.contains(".chars().peekable()"))) {
        return Err(String::from("expected .chars().peekable() RED failure"));
    }
    ws.write_rel("crates/demo/src/peek.rs", "pub fn scan(s: &str) -> usize { s.char_indices().count() }\n")?;
    if !matches!(check_dod_gates(&ws.path), Err(ref e) if e.iter().any(|m| m.contains(".char_indices()"))) {
        return Err(String::from("expected .char_indices() RED failure"));
    }
    let logos_ok = "#[derive(Logos)]\npub enum Tok { A }\npub fn lex(s: &str) -> usize { s.chars().peekable().count() + s.char_indices().count() }\n";
    ws.write_rel("crates/demo/src/peek.rs", logos_ok)?;
    check_dod_gates(&ws.path).map_err(|e| format!("expected Logos-derived module GREEN, got {e:?}"))
}

/// Verifies RED/GREEN hardcoded host paths and OS library suffixes, including `#[cfg(test)]` exemption (`[REQ-0608]`).
#[test]
fn hardcoded_paths_and_os_lib_suffixes_red_green() -> Result<(), String> {
    let ws = ScopedTempDir::new("literals")?;
    for bad in ["/Users/alice/bin", "/home/bob/x", "C:\\Users\\eve\\x", "libfoo.dylib", "foo.dll", "libfoo.so", "libfoo.so.6"] {
        let src = format!("pub fn bad() -> &'static str {{ {bad:?} }}\n");
        ws.write_rel("crates/demo/src/p.rs", &src)?;
        if check_dod_gates(&ws.path).is_ok() {
            return Err(format!("expected literal {bad:?} to fail outside #[cfg(test)]"));
        }
    }
    let test_exempt = concat!(
        "pub fn clean() -> &'static str { \"./crates/demo\" }\n",
        "#[cfg(test)]\nmod tests {\n",
        "    fn fixture() -> (&'static str, &'static str) { (\"/Users/alice/tmp\", \"libfoo.dylib\") }\n",
        "}\n"
    );
    ws.write_rel("crates/demo/src/p.rs", test_exempt)?;
    check_dod_gates(&ws.path).map_err(|e| format!("expected #[cfg(test)] literals GREEN, got {e:?}"))
}

/// Verifies clean `< 380`-SLOC module GREEN, absent `crates/` GREEN, syntax error RED, and own SLOC ceilings (`[REQ-0601]`).
#[test]
fn clean_module_and_self_sloc_bounds_pass() -> Result<(), String> {
    let empty_ws = ScopedTempDir::new("empty")?;
    check_dod_gates(&empty_ws.path).map_err(|e| format!("missing crates/ must pass: {e:?}"))?;

    let ws = ScopedTempDir::new("clean")?;
    let mut clean_src = String::from("//! Clean portable module.\n");
    for idx in 0..180_u32 {
        let _ = writeln!(clean_src, "pub fn step_{idx}(v: u32) -> u32 {{ v.saturating_add({idx}) }}");
    }
    ws.write_rel("crates/demo/src/clean.rs", &clean_src)?;
    ws.write_rel("crates/demo/target/generated_bad.rs", "pub fn broken( {")?;
    check_dod_gates(&ws.path).map_err(|e| format!("expected clean <380-SLOC module GREEN, got {e:?}"))?;

    ws.write_rel("crates/demo/src/broken.rs", "pub fn broken( {")?;
    if check_dod_gates(&ws.path).is_ok() {
        return Err(String::from("expected syntax error to fail closed"));
    }
    let gate_sloc = count_executable_sloc(include_str!("dod_gates.rs"))?;
    let test_sloc = count_executable_sloc(include_str!("dod_gates_tests.rs"))?;
    if gate_sloc >= 250 || test_sloc >= 220 {
        return Err(format!("SLOC bound exceeded: dod_gates={gate_sloc}/250, tests={test_sloc}/220"));
    }
    Ok(())
}

/// Verifies zero DoD gate findings across the current repository workspace `.` (`[REQ-0601]`, `[REQ-0608]`).
#[test]
fn current_repository_passes_dod_gates_with_zero_findings() -> Result<(), String> {
    let root = workspace_root()?;
    check_dod_gates(&root).map_err(|errs| format!("current repo failed dod_gates ({} findings): {errs:#?}", errs.len()))
}
