//! Markdown relative path computation and link relativization for compiled ConOps documents.

use std::path::{Component, Path, PathBuf};

/// [DOC-TIER: A] Computes a relative path from directory `base` to target `path`.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-ASSEMBLER-PATH-RELATIVE-FROM] | SysML v2: ActionDef(path_relative_from)
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

/// Checks whether `s` starts with a RFC 3986 URI scheme (`scheme://`).
/// Realises: [REQ-ASSEMBLE-CONOPS-ASSEMBLER-IS-URI-SCHEME] | SysML v2: ActionDef(is_uri_scheme)
fn is_uri_scheme(s: &str) -> bool {
    if let Some(pos) = s.find("://") {
        if pos == 0 {
            return false;
        }
        let scheme = &s[..pos];
        let mut chars = scheme.chars();
        if let Some(first) = chars.next() {
            if first.is_ascii_alphabetic()
                && chars.all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
            {
                return true;
            }
        }
    }
    false
}

/// Scans for a `[label](target)` Markdown link starting at byte index `idx` (`bytes[idx] == b'['`) (`[REQ-ASSEMBLE-CONOPS]`).
fn find_markdown_link_spans(bytes: &[u8], idx: usize) -> Option<(usize, usize, usize)> {
    let mut j = idx + 1;
    let mut label_end = None;
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
    let target_start = lend + 2;
    let mut k = target_start;
    let mut target_end = None;
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

/// [DOC-TIER: A] Normalizes markdown links `[label](target)` so they resolve relative to the directory of `output_file`.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-ASSEMBLER-RELATIVIZE-MARKDOWN-LINKS] | SysML v2: ActionDef(relativize_markdown_links)
/// Contract: Rewrites relative/workspace file links in `text: &str` relative to `output_file.parent()` while preserving `#anchor` and URI links.
/// Bounds & Faults: O(N) time and space over `text.len()`; falls back gracefully if canonicalization fails.
pub fn relativize_markdown_links(text: &str, output_file: &Path, workspace_dir: &Path) -> String {
    if text.is_empty() {
        return String::new();
    }

    let abs_ws_dir = workspace_dir
        .canonicalize()
        .unwrap_or_else(|_| workspace_dir.to_path_buf());
    let abs_output_file = if output_file.is_absolute() {
        output_file.to_path_buf()
    } else {
        abs_ws_dir.join(output_file)
    };
    let abs_output_dir = abs_output_file
        .parent()
        .unwrap_or(&abs_ws_dir)
        .to_path_buf();

    let bytes = text.as_bytes();
    let mut result = String::with_capacity(text.len());
    let mut idx = 0;

    while idx < bytes.len() {
        if bytes.get(idx).copied() == Some(b'[') {
            if let Some((lend, target_start, tend)) = find_markdown_link_spans(bytes, idx) {
                let label = &text[idx + 1..lend];
                let raw_target = &text[target_start..tend];
                if let Some(replacement) =
                    rewrite_link_target(label, raw_target, &abs_ws_dir, &abs_output_dir)
                {
                    result.push_str(&replacement);
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

/// Splits a trimmed Markdown link target into `(url_part, title_suffix, has_angle_brackets)` (`[REQ-ASSEMBLE-CONOPS]`).
fn split_link_url_and_title(target_trimmed: &str) -> (&str, String, bool) {
    if target_trimmed.starts_with('<') && target_trimmed.contains('>') {
        let end = target_trimmed.find('>').unwrap_or(1);
        let url = target_trimmed[1..end].trim();
        let title = &target_trimmed[end + 1..];
        (url, title.to_string(), true)
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

/// Resolves `path_part` to an absolute target path within `abs_ws_dir` or `abs_output_dir` (`[REQ-ASSEMBLE-CONOPS]`).
fn resolve_target_abs_path(path_part: &str, abs_ws_dir: &Path, abs_output_dir: &Path) -> PathBuf {
    let is_explicit_rel = path_part.starts_with("./")
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
    } else if is_explicit_rel {
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

/// Rewrites a single Markdown link target relative to `abs_output_dir`, or returns `None` to preserve verbatim.
/// Realises: [REQ-ASSEMBLE-CONOPS-ASSEMBLER-RELATIVIZE-MARKDOWN-LINKS] | SysML v2: ActionDef(rewrite_link_target)
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

    let (url_part, title_part, has_angle_brackets) = split_link_url_and_title(target_trimmed);
    if url_part.starts_with('#')
        || url_part.starts_with("http://")
        || url_part.starts_with("https://")
        || url_part.starts_with("mailto:")
        || url_part.starts_with("ftp://")
        || is_uri_scheme(url_part)
        || url_part.is_empty()
    {
        return None;
    }

    let (path_part, anchor_suffix) = if let Some((p, a)) = url_part.split_once('#') {
        (p, format!("#{a}"))
    } else {
        (url_part, String::new())
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
    if has_angle_brackets {
        Some(format!("[{label}](<{new_url}>{title_part})"))
    } else {
        Some(format!("[{label}]({new_url}{title_part})"))
    }
}
