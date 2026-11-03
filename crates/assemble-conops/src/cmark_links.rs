//! Unit integrity validation and Markdown link relativization (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).

use std::fs;
use std::path::{Component, Path, PathBuf};

use super::cmark_engine::SysMLParameterBindingEngine;

/// [DOC-TIER: A] Scans text for unresolved parameter tokens matching `{{KEY}}` or `{{KEY:default}}`.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-ASSEMBLER-FIND-RAW-TOKENS] [FLS: fls_conops_cmark]
/// Contract: Returns `Vec<String>` of all single-line `{{...}}` tokens whose key consists of ASCII alphanumeric/underscore characters.
/// Bounds & Faults: O(N) time and space over `text.len()`; ignores multi-line braces and LaTeX math braces.
pub fn find_raw_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i + 4 <= bytes.len() {
        if bytes.get(i).copied() == Some(b'{') && bytes.get(i + 1).copied() == Some(b'{') {
            let (start, mut j, mut found_end) = (i, i + 2, None);
            while j + 1 < bytes.len() {
                if bytes.get(j).copied() == Some(b'}') && bytes.get(j + 1).copied() == Some(b'}') {
                    found_end = Some(j);
                    break;
                }
                if bytes.get(j).copied() == Some(b'\n') {
                    break;
                }
                j += 1;
            }
            if let Some(end_pos) = found_end {
                let inner = &text[start + 2..end_pos];
                let (key, _default) = match inner.split_once(':') {
                    Some((k, d)) => (k, Some(d)),
                    None => (inner, None),
                };
                if !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    tokens.push(text[start..end_pos + 2].to_string());
                    i = end_pos + 2;
                    continue;
                }
            }
        }
        i += 1;
    }
    tokens
}

/// [DOC-TIER: A] Validates unit files for existence, non-emptiness, and zero unresolved `{{TOKEN}}` placeholders.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-ASSEMBLER-VALIDATE-UNIT-INTEGRITY] [FLS: fls_conops_cmark]
/// Contract: Substitutes each file in `unit_paths` with `param_engine` (when provided) and returns `(is_valid, errors)`.
/// Bounds & Faults: O(U * N) time and space over unit count and file length; reports deterministic diagnostics on I/O or token errors.
pub fn validate_unit_integrity(
    unit_paths: &[PathBuf],
    param_engine: Option<&SysMLParameterBindingEngine>,
) -> (bool, Vec<String>) {
    let mut errors = Vec::new();
    for path in unit_paths {
        if !path.is_file() {
            errors.push(format!("Unit file not found: {}", path.display()));
            continue;
        }
        let text = match fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) => {
                errors.push(format!("Failed to read unit file '{}': {e}", path.display()));
                continue;
            }
        };
        if text.trim().is_empty() {
            errors.push(format!("Unit file '{}' is empty.", path.display()));
            continue;
        }
        let processed = if let Some(eng) = param_engine {
            eng.substitute(&text)
        } else {
            text
        };
        let placeholders = find_raw_tokens(&processed);
        if !placeholders.is_empty() {
            let mut unique = placeholders;
            unique.sort();
            unique.dedup();
            errors.push(format!(
                "Unit file '{}' contains {} unresolved placeholder token(s): {}",
                path.display(),
                unique.len(),
                unique.join(", ")
            ));
        }
    }
    (errors.is_empty(), errors)
}

/// [DOC-TIER: A] Computes a relative path from directory `base` to target `path`.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-ASSEMBLER-PATH-RELATIVE-FROM] [FLS: fls_conops_cmark]
/// Contract: Accepts normalized `path: &Path` and `base: &Path` and returns relative `PathBuf` traversal using `..` components.
/// Bounds & Faults: O(N) time and space over path component count; infallible component traversal.
pub fn path_relative_from(path: &Path, base: &Path) -> PathBuf {
    let mut path_comps = path.components().peekable();
    let mut base_comps = base.components().peekable();
    while path_comps.peek() == base_comps.peek() && path_comps.peek().is_some() {
        path_comps.next();
        base_comps.next();
    }
    let mut result = PathBuf::new();
    for comp in base_comps {
        if matches!(comp, Component::Normal(_)) {
            result.push("..");
        }
    }
    for comp in path_comps {
        result.push(comp.as_os_str());
    }
    result
}

/// Checks whether `s` starts with an RFC 3986 URI scheme (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn is_uri_scheme(s: &str) -> bool {
    if let Some(pos) = s.find("://") {
        if pos == 0 {
            return false;
        }
        let mut chars = s[..pos].chars();
        if let Some(first) = chars.next() {
            return first.is_ascii_alphabetic()
                && chars.all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.');
        }
    }
    false
}

/// Scans for a `[label](target)` Markdown link starting at byte index `idx` (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn find_markdown_link_spans(bytes: &[u8], idx: usize) -> Option<(usize, usize, usize)> {
    let (mut j, mut label_end) = (idx + 1, None);
    while j < bytes.len() {
        if bytes.get(j).copied() == Some(b'\\') && j + 1 < bytes.len() {
            j += 2;
            continue;
        }
        if bytes.get(j).copied() == Some(b']') {
            label_end = Some(j);
            break;
        }
        j += 1;
    }
    let lend = label_end?;
    if lend <= idx + 1 || bytes.get(lend + 1).copied() != Some(b'(') {
        return None;
    }
    let (target_start, mut k, mut target_end) = (lend + 2, lend + 2, None);
    while k < bytes.len() {
        if bytes.get(k).copied() == Some(b')') {
            target_end = Some(k);
            break;
        }
        if bytes.get(k).copied() == Some(b'\n') {
            break;
        }
        k += 1;
    }
    let tend = target_end?;
    (tend > target_start).then_some((lend, target_start, tend))
}

/// Splits a trimmed Markdown link target into `(url_part, title_suffix, has_angle_brackets)` (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn split_link_url_and_title(target_trimmed: &str) -> (&str, String, bool) {
    if target_trimmed.starts_with('<') && target_trimmed.contains('>') {
        let end = target_trimmed.find('>').unwrap_or(1);
        (
            target_trimmed[1..end].trim(),
            target_trimmed[end + 1..].to_string(),
            true,
        )
    } else if let Some((p0, p1)) = target_trimmed.split_once(' ') {
        if p1.starts_with('"') || p1.starts_with('\'') || p1.starts_with('(') {
            (p0, format!(" {p1}"), false)
        } else {
            (target_trimmed, String::new(), false)
        }
    } else {
        (target_trimmed, String::new(), false)
    }
}

/// Resolves `path_part` to an absolute target path within `abs_ws_dir` or `abs_output_dir` (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn resolve_target_abs_path(path_part: &str, abs_ws_dir: &Path, abs_output_dir: &Path) -> PathBuf {
    let is_rel = path_part.starts_with("./")
        || path_part.starts_with("../")
        || path_part == "."
        || path_part == "..";
    let is_abs = path_part.starts_with('/')
        || (path_part.len() > 2 && path_part.as_bytes().get(1).copied() == Some(b':'));
    if is_abs {
        let p = Path::new(path_part);
        if p.starts_with(abs_ws_dir) {
            p.to_path_buf()
        } else {
            abs_ws_dir.join(path_part.trim_start_matches(['/', '\\']))
        }
    } else if is_rel {
        let candidate = abs_output_dir.join(path_part);
        if candidate.exists() {
            candidate
        } else {
            let trimmed = path_part.trim_start_matches(['.', '/', '\\']);
            let cand_ws = abs_ws_dir.join(trimmed);
            let cand_docs = abs_ws_dir.join("docs").join(trimmed);
            if cand_ws.exists() {
                cand_ws
            } else if cand_docs.exists() {
                cand_docs
            } else {
                candidate
            }
        }
    } else {
        abs_ws_dir.join(path_part)
    }
}

/// Rewrites a single Markdown link target relative to `abs_output_dir` (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn rewrite_link_target(
    label: &str,
    raw_target: &str,
    abs_ws_dir: &Path,
    abs_output_dir: &Path,
) -> Option<String> {
    let target_trimmed = raw_target.trim();
    if target_trimmed.is_empty() {
        return None;
    }
    let (url_part, title_part, has_angle) = split_link_url_and_title(target_trimmed);
    if url_part.starts_with('#')
        || is_uri_scheme(url_part)
        || url_part.starts_with("mailto:")
        || url_part.is_empty()
    {
        return None;
    }
    let (path_part, anchor_suffix) = match url_part.split_once('#') {
        Some((p, a)) => (p, format!("#{a}")),
        None => (url_part, String::new()),
    };
    if path_part.is_empty() {
        return None;
    }
    let target_abs = resolve_target_abs_path(path_part, abs_ws_dir, abs_output_dir);
    let rel = path_relative_from(&target_abs, abs_output_dir);
    let mut rel_str = rel.to_string_lossy().replace('\\', "/");
    if path_part.starts_with("./") && !rel_str.starts_with('.') && !rel_str.is_empty() {
        rel_str = format!("./{rel_str}");
    }
    let new_url = format!("{rel_str}{anchor_suffix}");
    if has_angle {
        Some(format!("[{label}](<{new_url}>{title_part})"))
    } else {
        Some(format!("[{label}]({new_url}{title_part})"))
    }
}

/// [DOC-TIER: A] Normalizes markdown links `[label](target)` so they resolve relative to `output_file`.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-ASSEMBLER-RELATIVIZE-MARKDOWN-LINKS] [FLS: fls_conops_cmark]
/// Contract: Rewrites relative/workspace file links in `text: &str` relative to `output_file.parent()` while preserving `#anchor` and URI links.
/// Bounds & Faults: O(N) time and space over `text.len()`; falls back gracefully if canonicalization fails.
pub fn relativize_markdown_links(text: &str, output_file: &Path, workspace_dir: &Path) -> String {
    if text.is_empty() {
        return String::new();
    }
    let abs_ws_dir = workspace_dir
        .canonicalize()
        .unwrap_or_else(|_| workspace_dir.to_path_buf());
    let abs_out = if output_file.is_absolute() {
        output_file.to_path_buf()
    } else {
        abs_ws_dir.join(output_file)
    };
    let abs_out_dir = abs_out.parent().unwrap_or(&abs_ws_dir).to_path_buf();
    let bytes = text.as_bytes();
    let (mut result, mut idx) = (String::with_capacity(text.len()), 0);
    while idx < bytes.len() {
        if bytes.get(idx).copied() == Some(b'[') {
            if let Some((lend, tstart, tend)) = find_markdown_link_spans(bytes, idx) {
                let label = &text[idx + 1..lend];
                let raw_target = &text[tstart..tend];
                if let Some(repl) = rewrite_link_target(label, raw_target, &abs_ws_dir, &abs_out_dir)
                {
                    result.push_str(&repl);
                } else {
                    result.push_str(&text[idx..tend + 1]);
                }
                idx = tend + 1;
                continue;
            }
        }
        if let Some(ch) = text[idx..].chars().next() {
            result.push(ch);
            idx += ch.len_utf8();
        } else {
            break;
        }
    }
    result
}
