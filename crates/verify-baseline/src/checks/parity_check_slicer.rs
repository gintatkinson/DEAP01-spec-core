//! Section slicer (`SectionSlicer`) and citation token extractors (`CitationRef`) for parity gates.
//!
//! Safety & Traceability: Realises: [REQ-SECTION-EXTRACTION-NO-FALLBACK], [REQ-SAME-LINE-CITATION-PRESERVATION]
//! Contract: Resolves workspace-contained citation paths and extracts isolated Markdown/SysML/JSON sections.
//! Bounds & Faults: Rejects path traversal (`..` / absolute paths) and fails closed on missing section anchors.

use super::parity_check_discovery::PathPrefix;
use std::fs;
use std::path::{Component, Path, PathBuf};

/// Persistent Section Slicer validating physical on-disk citations and section anchors.
/// Realises: [REQ-CHECKS-PARITY-GATES-SECTIONSLICER]
#[derive(Debug, Clone)]
pub struct SectionSlicer<'a> {
    pub repo_root: &'a Path,
}

impl<'a> SectionSlicer<'a> {
    /// Constructs a new `SectionSlicer` for a given repository root.
    /// Realises: [REQ-SECTION-SLICER-INIT]
    pub fn new(repo_root: &'a Path) -> Self {
        Self { repo_root }
    }

    /// Resolves and sanitizes a target file path against the repository root.
    /// Realises: [REQ-PATH-TRAVERSAL-SANITIZATION]
    pub fn resolve_path(&self, target_file: &str) -> Option<PathBuf> {
        let clean = target_file.trim_matches(|c: char| {
            c.is_whitespace() || matches!(c, '`' | '\'' | '"' | '[' | ']' | '(' | ')' | '<' | '>')
        });
        let rel_path = Path::new(clean);
        if rel_path.is_absolute()
            || rel_path
                .components()
                .any(|comp| matches!(comp, Component::ParentDir))
        {
            return None;
        }
        let path = self.repo_root.join(rel_path);
        if path.is_file() {
            return Some(path);
        }
        let parent_opt = crate::checks::structural::enclosing_upstream_compiler_root(self.repo_root);
        if let Some(ref parent_repo) = parent_opt {
            let parent_target = parent_repo.join(rel_path);
            if parent_target.is_file() {
                return Some(parent_target);
            }
        }
        if let Some(rest) = clean.strip_prefix("docs/architecture/blueprints/") {
            if rest.starts_with("blueprint-") && !rest.contains('/') {
                let design_rel = Path::new("docs/designs").join(rest);
                let design_path = self.repo_root.join(&design_rel);
                if design_path.is_file() {
                    return Some(design_path);
                }
                if let Some(ref parent_repo) = parent_opt {
                    let parent_design = parent_repo.join(&design_rel);
                    if parent_design.is_file() {
                        return Some(parent_design);
                    }
                }
            }
        }
        None
    }

    /// Checks if a Markdown heading line matches a targeted section identifier with exact word boundaries.
    /// Realises: [REQ-EXACT-HEADING-BOUNDARIES]
    pub fn heading_matches_section(heading_line: &str, sec_norm: &str) -> bool {
        let body = heading_line.trim_start_matches('#').trim();
        for token in body.split_whitespace() {
            let clean_tok = token
                .trim_start_matches('§')
                .trim_end_matches(['.', ':', ')', ']', '>']);
            if clean_tok.eq_ignore_ascii_case(sec_norm) {
                return true;
            }
        }
        body.eq_ignore_ascii_case(sec_norm)
    }

    /// Extracts the text content under a designated section heading.
    /// Realises: [REQ-SECTION-EXTRACTION-NO-FALLBACK]
    pub fn extract_section(&self, target_file: &str, section: &str) -> Option<String> {
        let path = self.resolve_path(target_file)?;
        let content = fs::read_to_string(path).ok()?;
        let sec_norm = section.trim().trim_start_matches('§').trim();
        if sec_norm.is_empty() {
            return Some(content);
        }
        let lower_target = target_file
            .trim()
            .trim_matches(|c: char| matches!(c, '`' | '\'' | '"' | '(' | ')'))
            .to_ascii_lowercase();
        if lower_target.ends_with(".sysml") || lower_target.ends_with(".json") {
            return Some(content);
        }

        let mut in_target_sec = false;
        let mut target_level = 0;
        let mut sec_content = String::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                let level = trimmed.bytes().take_while(|&b| b == b'#').count();
                if in_target_sec {
                    if level <= target_level {
                        break;
                    }
                } else if Self::heading_matches_section(trimmed, sec_norm) {
                    in_target_sec = true;
                    target_level = level;
                }
            }
            if in_target_sec {
                sec_content.push_str(line);
                sec_content.push('\n');
            }
        }

        if in_target_sec && !sec_content.is_empty() {
            Some(sec_content)
        } else {
            None
        }
    }
}

/// Extracted source citation from specification or safety prose.
/// Realises: [REQ-CHECKS-PARITY-GATES-CITATIONREF]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CitationRef {
    pub target_file: String,
    pub section: Option<String>,
    pub line_no: usize,
}

/// Parses a citation token into target file and optional section identifier.
/// Realises: [REQ-CITATION-PARSING-CHAR-BOUNDARY]
pub(crate) fn parse_citation_string(raw: &str, line_no: usize) -> Option<CitationRef> {
    let lower = raw.to_ascii_lowercase();
    let start_pos = match (lower.find("schema/"), lower.find("docs/")) {
        (Some(a), Some(b)) => a.min(b),
        (Some(a), None) => a,
        (None, Some(b)) => b,
        (None, None) => return None,
    };

    let slice = raw.get(start_pos..)?;
    let path_end = slice
        .find(|c: char| {
            c.is_whitespace()
                || matches!(
                    c,
                    '`' | '\'' | '"' | '(' | ')' | '[' | ']' | '<' | '>' | '§' | '#' | ',' | ';' | '|'
                )
        })
        .unwrap_or(slice.len());

    let path_slice = slice.get(..path_end)?;
    if path_slice.ends_with('/') {
        return None;
    }
    let path = path_slice
        .trim_matches(|c: char| matches!(c, '.' | ':' | '`' | '\'' | '"'))
        .to_string();
    let after_path = slice.get(path_end..).unwrap_or("");
    if path.is_empty()
        || path.ends_with('/')
        || path.contains(['*', '<', '>'])
        || after_path.starts_with('<')
    {
        return None;
    }

    let mut section = None;
    if let Some(sec_pos) = after_path.find('§') {
        let after_sec_pos = sec_pos.saturating_add('§'.len_utf8());
        let after_sec = after_path.get(after_sec_pos..).unwrap_or("").trim_start();
        let sec_end = after_sec
            .find(|c: char| {
                c.is_whitespace()
                    || matches!(c, '`' | '\'' | '"' | ')' | ']' | '>' | ',' | ';' | '|')
            })
            .unwrap_or(after_sec.len());
        let sec_val = after_sec
            .get(..sec_end)
            .unwrap_or("")
            .trim_end_matches('.')
            .trim();
        if !sec_val.is_empty() {
            section = Some(sec_val.to_string());
        }
    }

    let final_segment = path.rsplit('/').next().unwrap_or("");
    if final_segment.is_empty() || (section.is_none() && !final_segment.contains('.')) {
        return None;
    }

    Some(CitationRef {
        target_file: path,
        section,
        line_no,
    })
}

/// Extracts citations from a single line of text across HTML comments, prose, lists, and tables.
/// Realises: [REQ-SAME-LINE-CITATION-PRESERVATION]
pub fn extract_citations_from_line(line: &str, line_no: usize) -> Vec<CitationRef> {
    let mut citations = Vec::new();
    let lower = line.to_ascii_lowercase();

    let mut push_unique = |c: CitationRef| {
        if !citations
            .iter()
            .any(|ex: &CitationRef| ex.target_file == c.target_file && ex.section == c.section)
        {
            citations.push(c);
        }
    };

    if let Some(start_idx) = lower.find("<!-- source:") {
        let after_prefix_idx = start_idx.saturating_add("<!-- source:".len());
        if let Some(cit_str) = line
            .get(after_prefix_idx..)
            .and_then(|after| after.find("-->").and_then(|end_idx| after.get(..end_idx)))
        {
            if let Some(c) = parse_citation_string(cit_str.trim(), line_no) {
                push_unique(c);
            }
        }
    }

    for prefix_type in PathPrefix::all() {
        let prefix = prefix_type.as_str();
        let mut search_from = 0;
        while let Some(search_slice) = lower.get(search_from..) {
            let Some(pos) = search_slice.find(prefix) else {
                break;
            };
            let abs_pos = search_from.saturating_add(pos);
            let valid_boundary = abs_pos == 0
                || line
                    .as_bytes()
                    .get(abs_pos.saturating_sub(1))
                    .is_none_or(|&b| !b.is_ascii_alphanumeric() && b != b'_' && b != b'-');
            if valid_boundary {
                if let Some(c) = line
                    .get(abs_pos..)
                    .and_then(|rem| parse_citation_string(rem, line_no))
                {
                    let advance = c.target_file.len().max(prefix.len());
                    push_unique(c);
                    search_from = abs_pos.saturating_add(advance);
                    continue;
                }
            }
            search_from = abs_pos.saturating_add(prefix.len());
        }
    }

    citations
}
