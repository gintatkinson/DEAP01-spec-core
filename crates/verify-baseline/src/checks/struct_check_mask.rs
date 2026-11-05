//! Comment/literal line-preserving masker, Tier A module docblock auditor, and range-index detector.
//!
//! Safety & Traceability: Realises: [REQ-CHECK-34-MASK-COMMENTS-AND-LITERALS]
//! Contract: Masks comments, string literals, raw strings, and character literals while preserving 1-based line numbers.
//! Bounds & Faults: Single-pass O(N) character scan; never panics on malformed literals or EOF.

use super::struct_check_purity::{contains_bounded_split_token, has_split_token};

/// Cursor state for line-preserving comment and literal masking.
/// Realises: [REQ-CHECK-34-MASK-COMMENTS-AND-LITERALS]
struct MaskCursor<'a> {
    chars: &'a [char],
    i: usize,
    lines: Vec<String>,
    current_line: String,
    only_ws_on_line: bool,
}

impl<'a> MaskCursor<'a> {
    /// Consumes a `//` line comment or preserves a leading `///` / `//!` doc comment line.
    /// Realises: [REQ-CHECK-34-MASK-COMMENTS-AND-LITERALS]
    fn consume_line_or_doc_comment(&mut self) {
        let third = self.chars.get(self.i.saturating_add(2)).copied();
        if self.only_ws_on_line && (third == Some('/') || third == Some('!')) {
            while let Some(ch) = self.chars.get(self.i).copied() {
                if ch == '\n' {
                    break;
                }
                if ch != '\r' {
                    self.current_line.push(ch);
                }
                self.i = self.i.saturating_add(1);
            }
            return;
        }
        self.i = self.i.saturating_add(2);
        while let Some(ch) = self.chars.get(self.i).copied() {
            if ch == '\n' {
                break;
            }
            self.i = self.i.saturating_add(1);
        }
    }

    /// Consumes a nested `/* ... */` block comment while preserving newline boundaries.
    /// Realises: [REQ-CHECK-34-MASK-COMMENTS-AND-LITERALS]
    fn consume_block_comment(&mut self) {
        self.current_line.push(' ');
        let mut depth: usize = 1;
        self.i = self.i.saturating_add(2);
        while depth > 0 {
            match self.chars.get(self.i).copied() {
                None => break,
                Some('\n') => {
                    self.lines.push(std::mem::take(&mut self.current_line));
                    self.only_ws_on_line = true;
                    self.i = self.i.saturating_add(1);
                }
                Some('/') if self.chars.get(self.i.saturating_add(1)).copied() == Some('*') => {
                    depth = depth.saturating_add(1);
                    self.i = self.i.saturating_add(2);
                }
                Some('*') if self.chars.get(self.i.saturating_add(1)).copied() == Some('/') => {
                    depth = depth.saturating_sub(1);
                    self.i = self.i.saturating_add(2);
                }
                Some(_) => {
                    self.i = self.i.saturating_add(1);
                }
            }
        }
    }

    /// Attempts to consume a raw string literal (`r"..."`, `r#"..."#`, `br"..."`, `br#"..."#`).
    /// Realises: [REQ-CHECK-34-MASK-COMMENTS-AND-LITERALS]
    fn try_consume_raw_string(&mut self, c: char) -> bool {
        let prev_is_ident = if self.i > 0 {
            self.chars
                .get(self.i.saturating_sub(1))
                .copied()
                .is_some_and(|p| p.is_ascii_alphanumeric() || p == '_' || p == '\'')
        } else {
            false
        };
        if prev_is_ident {
            return false;
        }
        let raw_r_pos = if c == 'r' {
            Some(self.i)
        } else if c == 'b' && self.chars.get(self.i.saturating_add(1)).copied() == Some('r') {
            Some(self.i.saturating_add(1))
        } else {
            None
        };
        let Some(r_pos) = raw_r_pos else {
            return false;
        };
        let mut scan = r_pos.saturating_add(1);
        let mut hashes: usize = 0;
        while self.chars.get(scan).copied() == Some('#') {
            hashes = hashes.saturating_add(1);
            scan = scan.saturating_add(1);
        }
        if self.chars.get(scan).copied() != Some('"') {
            return false;
        }
        self.only_ws_on_line = false;
        self.current_line.push_str("\"\"");
        self.i = scan.saturating_add(1);
        while let Some(ch) = self.chars.get(self.i).copied() {
            if ch == '\n' {
                self.lines.push(std::mem::take(&mut self.current_line));
                self.only_ws_on_line = true;
                self.i = self.i.saturating_add(1);
            } else if ch == '"' {
                self.i = self.i.saturating_add(1);
                let mut matched: usize = 0;
                while matched < hashes
                    && self.chars.get(self.i.saturating_add(matched)).copied() == Some('#')
                {
                    matched = matched.saturating_add(1);
                }
                if matched == hashes {
                    self.i = self.i.saturating_add(hashes);
                    break;
                }
            } else {
                self.i = self.i.saturating_add(1);
            }
        }
        true
    }

    /// Consumes a standard double-quoted string literal `"..."` while preserving newlines.
    /// Realises: [REQ-CHECK-34-MASK-COMMENTS-AND-LITERALS]
    fn consume_quoted_string(&mut self) {
        self.only_ws_on_line = false;
        self.current_line.push_str("\"\"");
        self.i = self.i.saturating_add(1);
        while let Some(ch) = self.chars.get(self.i).copied() {
            if ch == '\\' {
                if self.chars.get(self.i.saturating_add(1)).copied() == Some('\n') {
                    self.lines.push(std::mem::take(&mut self.current_line));
                    self.only_ws_on_line = true;
                }
                self.i = self.i.saturating_add(2);
            } else if ch == '"' {
                self.i = self.i.saturating_add(1);
                break;
            } else {
                if ch == '\n' {
                    self.lines.push(std::mem::take(&mut self.current_line));
                    self.only_ws_on_line = true;
                }
                self.i = self.i.saturating_add(1);
            }
        }
    }

    /// Consumes a character literal `'x'` / `'\\.'` or lifetime token `'a`.
    /// Realises: [REQ-CHECK-34-MASK-COMMENTS-AND-LITERALS]
    fn consume_char_or_lifetime(&mut self) {
        if self.chars.get(self.i.saturating_add(1)).copied() == Some('\\') {
            self.only_ws_on_line = false;
            self.current_line.push_str("' '");
            self.i = self.i.saturating_add(2);
            while let Some(ch) = self.chars.get(self.i).copied() {
                self.i = self.i.saturating_add(1);
                if ch == '\'' || ch == '\n' {
                    if ch == '\n' {
                        self.lines.push(std::mem::take(&mut self.current_line));
                        self.only_ws_on_line = true;
                    }
                    break;
                }
            }
        } else if self.chars.get(self.i.saturating_add(2)).copied() == Some('\'')
            && self.chars.get(self.i.saturating_add(1)).copied() != Some('\n')
        {
            self.only_ws_on_line = false;
            self.current_line.push_str("' '");
            self.i = self.i.saturating_add(3);
        } else {
            self.only_ws_on_line = false;
            self.current_line.push('\'');
            self.i = self.i.saturating_add(1);
        }
    }
}

/// Masks non-doc comments, string literals, raw strings, and character literals while preserving 1-based line numbers.
/// Realises: [REQ-CHECK-34-MASK-COMMENTS-AND-LITERALS]
pub(crate) fn mask_comments_and_literals_preserving_lines(source: &str) -> Vec<String> {
    let chars: Vec<char> = source.chars().collect();
    let len = chars.len();
    let mut cursor = MaskCursor {
        chars: &chars,
        i: 0,
        lines: Vec::new(),
        current_line: String::new(),
        only_ws_on_line: true,
    };

    while cursor.i < len {
        let Some(c) = cursor.chars.get(cursor.i).copied() else {
            break;
        };
        if c == '\r' {
            cursor.i = cursor.i.saturating_add(1);
            continue;
        }
        if c == '\n' {
            cursor
                .lines
                .push(std::mem::take(&mut cursor.current_line));
            cursor.only_ws_on_line = true;
            cursor.i = cursor.i.saturating_add(1);
            continue;
        }
        if c == '/' && cursor.chars.get(cursor.i.saturating_add(1)).copied() == Some('/') {
            cursor.consume_line_or_doc_comment();
            continue;
        }
        if c == '/' && cursor.chars.get(cursor.i.saturating_add(1)).copied() == Some('*') {
            cursor.consume_block_comment();
            continue;
        }
        if cursor.try_consume_raw_string(c) {
            continue;
        }
        if c == '"' {
            cursor.consume_quoted_string();
            continue;
        }
        if c == '\'' {
            cursor.consume_char_or_lifetime();
            continue;
        }
        if !c.is_whitespace() {
            cursor.only_ws_on_line = false;
        }
        cursor.current_line.push(c);
        cursor.i = cursor.i.saturating_add(1);
    }

    if (!cursor.current_line.is_empty() || !source.ends_with('\n')) && !source.is_empty() {
        cursor.lines.push(cursor.current_line);
    }
    cursor.lines
}

/// Audits top-level `//!` module header docblocks in `masked_lines` against the Tier A contract.
/// Realises: [REQ-CHECK-34-TIER-A-MODULE-DOCBLOCK]
pub(crate) fn audit_tier_a_module_docblocks(
    masked_lines: &[String],
    file_rel: &str,
    violations: &mut Vec<String>,
) {
    let mut module_doc: Vec<String> = Vec::new();
    let mut start_line: usize = 1;

    for (idx, line) in masked_lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//!") {
            if module_doc.is_empty() {
                start_line = idx.saturating_add(1);
            }
            module_doc.push(trimmed.to_string());
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with("#![") {
            continue;
        }
        flush_tier_a_module_docblock(&mut module_doc, file_rel, start_line, violations);
        if trimmed.contains("cfg(test)") || contains_bounded_split_token(trimmed, "mod", " tests") {
            break;
        }
    }
    flush_tier_a_module_docblock(&mut module_doc, file_rel, start_line, violations);
}

/// Validates and clears an accumulated `//!` module docblock against the Tier A contract.
/// Realises: [REQ-CHECK-34-FLUSH-TIER-A-MODULE-DOCBLOCK]
fn flush_tier_a_module_docblock(
    module_doc: &mut Vec<String>,
    file_rel: &str,
    start_line: usize,
    violations: &mut Vec<String>,
) {
    if module_doc.is_empty() {
        return;
    }
    if !is_legacy_single_line_module_title(module_doc)
        && !super::docstring_gate::is_compliant_rust_docblock(module_doc, true)
    {
        violations.push(format!(
            "{}:{}: Tier A module header missing valid 7-section Rustdoc, 4-Block Contract, or non-empty 'Realises: [...]' tag",
            file_rel, start_line
        ));
    }
    module_doc.clear();
}

/// Identifies a legacy single-line non-empty `//!` module title that carries no partial contract or `Realises:` claim.
/// Realises: [REQ-CHECK-34-LEGACY-SINGLE-LINE-MODULE-TITLE]
fn is_legacy_single_line_module_title(module_doc: &[String]) -> bool {
    if module_doc.len() != 1 {
        return false;
    }
    let Some(first) = module_doc.first() else {
        return false;
    };
    let body = first.trim().strip_prefix("//!").unwrap_or("").trim();
    body.chars().any(char::is_alphanumeric)
        && !body.starts_with('#')
        && !body.contains("Realises:")
        && !body.contains("Safety & Traceability")
        && !body.contains("Contract:")
        && !body.contains("Bounds & Faults:")
}

/// Detects direct panicking range-slice indexing (`expr[a..b]`) on one masked code line.
/// Realises: [REQ-CHECK-34-DIRECT-RANGE-INDEX-SCAN]
pub(crate) fn contains_direct_range_index(code: &str) -> bool {
    let bytes = code.as_bytes();
    let mut prev_byte: Option<u8> = None;
    for (open_idx, &byte) in bytes.iter().enumerate() {
        let is_index_operator = byte == b'['
            && prev_byte
                .is_some_and(|p| p.is_ascii_alphanumeric() || p == b'_' || p == b')' || p == b']');
        if is_index_operator {
            let mut depth: usize = 0;
            let mut close_idx = bytes.len();
            for (scan_idx, &scan_byte) in bytes.iter().enumerate().skip(open_idx) {
                if scan_byte == b'[' {
                    depth = depth.saturating_add(1);
                } else if scan_byte == b']' {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        close_idx = scan_idx;
                        break;
                    }
                }
            }
            let inner = code
                .get(open_idx.saturating_add(1)..close_idx)
                .unwrap_or(code);
            if has_split_token(inner, ".", ".") {
                return true;
            }
        }
        prev_byte = Some(byte);
    }
    false
}
