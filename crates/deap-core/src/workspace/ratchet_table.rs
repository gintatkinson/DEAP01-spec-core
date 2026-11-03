#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Baseline line-count ratchet table locking legacy `.rs` files at exact HEAD ceilings.
//!
//! Safety & Traceability: Realises: [REQ-0601], [REQ-0608]
//! Contract: Provides `LEGACY_RATCHET_MAX_LINES`, `active_greater_than_500_ratchet_count`, and `lookup_ratchet_ceiling` for exact legacy file line-count verification.
//! Bounds & Faults: O(log N) binary search over sorted static entries with zero panicking operations.

use std::borrow::Cow;

/// Canonical `(rel_path, max_lines)` entry in `LEGACY_RATCHET_MAX_LINES`.
/// Realises: [REQ-0601], [REQ-0608]
pub type RatchetPair = (&'static str, usize);

/// Lexicographically sorted `(rel_path, max_lines)` ceilings for legacy workspace `.rs` files.
/// Any growth above the recorded HEAD line count fails verification (`[REQ-0601]`, `[REQ-0608]`).
/// Realises: [REQ-0601], [REQ-0608]
pub const LEGACY_RATCHET_MAX_LINES: &[RatchetPair] = &[];

/// Indicates that zero new unratcheted `> 500`-line modules are permitted in the workspace (`[REQ-0601]`, `[REQ-0801]`).
pub const ZERO_NEW_RATCHET_ENFORCED: bool = true;

/// Canonical forbidden dummy CST pattern entry type (`[REQ-0601]`, `[REQ-0608]`).
/// Realises: [REQ-0601], [REQ-0608]
pub type ForbiddenDummyPattern = &'static str;

/// Forbidden hand-rolled scanner and dummy CST discard patterns (`[REQ-0601]`, `[REQ-0608]`).
/// Realises: [REQ-0601], [REQ-0608]
pub const FORBIDDEN_DUMMY_CST_PATTERNS: &[ForbiddenDummyPattern] = &[
    "let _ = parse_",
    "let _cst =",
    "scan_quoted_string",
    "tokenize_yang",
];

/// Counts entries in [`LEGACY_RATCHET_MAX_LINES`] whose line ceiling exceeds `500` (`[REQ-0601]`, `[REQ-0801]`).
#[must_use]
pub const fn active_greater_than_500_ratchet_count() -> usize {
    let mut rest = LEGACY_RATCHET_MAX_LINES;
    let mut count = 0;
    while let Some((&(_, ceiling), tail)) = rest.split_first() {
        if ceiling > 500 {
            count += 1;
        }
        rest = tail;
    }
    count
}

/// Returns the maximum allowed line count for `rel_path` in `table` via `O(log N)` binary search.
/// Normalizes Windows `\` separators and bare `crates/...` prefixes (`[REQ-0601]`, `[REQ-0608]`).
#[must_use]
pub fn lookup_ratchet_ceiling_in(table: &[RatchetPair], rel_path: &str) -> Option<usize> {
    let slash_norm: Cow<'_, str> = if rel_path.contains('\\') {
        Cow::Owned(rel_path.replace('\\', "/"))
    } else {
        Cow::Borrowed(rel_path)
    };
    let canonical: Cow<'_, str> = if slash_norm.starts_with("crates/") {
        Cow::Owned(format!("./{slash_norm}"))
    } else {
        slash_norm
    };
    let target = canonical.as_ref();
    table
        .binary_search_by_key(&target, |&(path, _)| path)
        .ok()
        .and_then(|idx| table.get(idx).map(|&(_, ceiling)| ceiling))
}

/// Returns the maximum allowed line count for `rel_path` via `O(log N)` binary search.
/// Normalizes Windows `\` separators and bare `crates/...` prefixes (`[REQ-0601]`, `[REQ-0608]`).
#[must_use]
pub fn lookup_ratchet_ceiling(rel_path: &str) -> Option<usize> {
    lookup_ratchet_ceiling_in(LEGACY_RATCHET_MAX_LINES, rel_path)
}
