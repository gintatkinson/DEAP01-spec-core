#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! CST-based attribute gate (`BP-01`) using `ra_ap_syntax::SourceFile::parse` (`Edition2024`).
//! Realises: [REQ-0601], [REQ-0608]

use ra_ap_syntax::{ast, AstNode, Edition, SourceFile, SyntaxKind, SyntaxToken};
use std::fs;
use std::path::{Path, PathBuf};

/// Represents a forbidden lint-suppression or compiler-internal attribute node (`[REQ-0601]`, `[REQ-0608]`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClippySuppressionViolation {
    /// Workspace-relative file path containing the attribute (`[REQ-0601]`).
    pub file: String,
    /// 1-based line number of the attribute start (`[REQ-0601]`).
    pub line: usize,
    /// Normalized non-trivia token representation of the attribute (`[REQ-0601]`).
    pub attribute: String,
}

/// Scans `content` with `ra_ap_syntax::SourceFile::parse` (`Edition2024`) for forbidden attributes (`[REQ-0601]`).
#[must_use]
pub fn scan_clippy_allow_tokens(content: &str, file_label: &str) -> Vec<ClippySuppressionViolation> {
    let parse = SourceFile::parse(content, Edition::Edition2024);
    let mut violations = Vec::new();
    for attr in parse.tree().syntax().descendants().filter_map(ast::Attr::cast) {
        let non_trivia: Vec<SyntaxToken> = attr
            .syntax()
            .descendants_with_tokens()
            .filter_map(ra_ap_syntax::NodeOrToken::into_token)
            .filter(|tok| !tok.kind().is_trivia())
            .collect();
        if is_forbidden_attribute_tokens(&non_trivia) {
            let offset: usize = attr.syntax().text_range().start().into();
            let line = content
                .get(..offset)
                .map_or(1, |prefix| prefix.bytes().filter(|&b| b == b'\n').count().saturating_add(1));
            let attribute: String = non_trivia.iter().map(SyntaxToken::text).collect();
            violations.push(ClippySuppressionViolation {
                file: file_label.to_string(),
                line,
                attribute,
            });
        }
    }
    violations
}

/// Checks whether non-trivia attribute tokens represent `allow(...)`, `expect(...)`, or `feature(rustc_private)` (`[REQ-0601]`).
fn is_forbidden_attribute_tokens(tokens: &[SyntaxToken]) -> bool {
    let code_tokens: Vec<&SyntaxToken> = tokens
        .iter()
        .filter(|tok| {
            !matches!(
                tok.kind(),
                SyntaxKind::STRING
                    | SyntaxKind::BYTE_STRING
                    | SyntaxKind::C_STRING
                    | SyntaxKind::CHAR
                    | SyntaxKind::BYTE
            )
        })
        .collect();
    for (idx, tok) in code_tokens.iter().enumerate() {
        if tok.kind() != SyntaxKind::IDENT {
            continue;
        }
        let next_is_lparen = code_tokens
            .get(idx.saturating_add(1))
            .is_some_and(|next| next.kind() == SyntaxKind::L_PAREN);
        let text = tok.text();
        if (text == "allow" || text == "expect") && next_is_lparen {
            return true;
        }
        if text == "feature"
            && next_is_lparen
            && code_tokens
                .iter()
                .skip(idx.saturating_add(2))
                .any(|arg| arg.kind() == SyntaxKind::IDENT && arg.text() == "rustc_private")
        {
            return true;
        }
    }
    false
}

/// Audits all Rust sources under `<workspace_root>/crates` for forbidden `#[allow]`/`#[expect]` attributes (`[REQ-0601]`).
///
/// # Errors
/// Returns formatted diagnostic strings when any forbidden attribute CST node is found (`[REQ-0608]`).
pub fn check_cst_attr_gate(workspace_root: &Path) -> Result<(), Vec<String>> {
    let crates_dir = workspace_root.join("crates");
    if !crates_dir.exists() {
        return Ok(());
    }
    let mut rs_files = Vec::new();
    let mut errors = Vec::new();
    collect_crate_rs_files(&crates_dir, &mut rs_files, &mut errors);
    rs_files.sort();
    for path in &rs_files {
        let rel = path
            .strip_prefix(workspace_root)
            .unwrap_or(path.as_path())
            .to_string_lossy()
            .replace('\\', "/");
        let source = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                errors.push(format!("{rel}: failed to read Rust source: {e}"));
                continue;
            }
        };
        for v in scan_clippy_allow_tokens(&source, &rel) {
            errors.push(format!(
                "{}:L{}: forbidden attribute `{}` detected (BP-01)",
                v.file, v.line, v.attribute
            ));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Recursively collects `.rs` files under `dir`, skipping `target` and hidden directories (`[REQ-0601]`).
fn collect_crate_rs_files(dir: &Path, out: &mut Vec<PathBuf>, errors: &mut Vec<String>) {
    let entries = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(e) => {
            errors.push(format!("{}: failed to read directory: {e}", dir.display()));
            return;
        }
    };
    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in entries {
        match entry {
            Ok(ent) => paths.push(ent.path()),
            Err(e) => errors.push(format!("{}: directory entry error: {e}", dir.display())),
        }
    }
    paths.sort();
    for path in paths {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name == "target" || name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            collect_crate_rs_files(&path, out, errors);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{check_cst_attr_gate, scan_clippy_allow_tokens};
    use std::path::Path;

    /// Verifies `scan_clippy_allow_tokens` ignores comments/doc-strings and rejects `allow`/`expect`/`rustc_private` (`[REQ-0601]`).
    #[test]
    fn cst_attr_gate_rejects_allow_expect_and_ignores_doc_literals() -> Result<(), String> {
        let kw_allow = format!("{}{}", "allow(", "clippy::");
        let kw_expect = format!("{}{}", "expect(", "clippy::");
        let clean = format!(
            "// #[{kw_allow}unwrap_used)]\n\
             #[doc = \"{kw_allow}unwrap_used) and {kw_expect}panic)\"]\n\
             pub fn clean_fn() -> &'static str {{\n\
                 \"#[{kw_allow}indexing_slicing)]\"\n\
             }}\n"
        );
        let clean_hits = scan_clippy_allow_tokens(&clean, "crates/demo/src/lib.rs");
        if !clean_hits.is_empty() {
            return Err(format!("expected zero violations on doc/string literals, got {clean_hits:?}"));
        }

        let dirty = format!(
            "#![feature(rustc_private)]\n\
             #![{kw_allow}indexing_slicing)]\n\
             #[{kw_expect}unwrap_used)]\n\
             #[cfg_attr(test, {kw_allow}panic))]\n\
             #[allow(non_camel_case_types)]\n\
             pub struct Bad;\n"
        );
        let hits = scan_clippy_allow_tokens(&dirty, "crates/demo/src/lib.rs");
        if hits.len() != 5 {
            return Err(format!("expected 5 CST attribute violations, got {}: {hits:?}", hits.len()));
        }
        Ok(())
    }

    /// Verifies zero `#[allow(...)]` or `#[expect(...)]` attribute nodes exist across `crates/` (`[REQ-0601]`, `[REQ-0608]`).
    #[test]
    fn workspace_crates_have_zero_allow_or_expect_attributes() -> Result<(), String> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| String::from("missing workspace root"))?;
        check_cst_attr_gate(root).map_err(|errs| format!("workspace CST attribute gate failed: {errs:#?}"))
    }
}
