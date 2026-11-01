#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Check 36 docstring tier classification, traceability anchor, and anti-padding rules.
//! Enforces explicit `[DOC-TIER: A]` opt-in and `<= 4`-line Tier-B/C docblocks on non-ratcheted modules.
//! Realises: [REQ-0605]

/// Explicit docblock marker opting an item into multi-section Tier-A validation (`[REQ-0605]`).
pub const EXPLICIT_TIER_A_MARKER: &str = "[DOC-TIER: A]";

/// Maximum number of doc-comment lines permitted on a non-ratcheted Tier-B/C item (`[REQ-0605]`).
pub const MAX_TIER_BC_DOC_LINES: usize = 4;

/// Returns `true` iff `doc_lines` explicitly opts into multi-section Tier-A via unbackticked `[DOC-TIER: A]` (`[REQ-0605]`).
#[must_use]
pub fn has_explicit_tier_a_marker(doc_lines: &[String]) -> bool {
    doc_lines
        .iter()
        .any(|line| line_has_unbackticked_tier_a_marker(line))
}

/// Checks a single doc line for `[DOC-TIER: A]` outside balanced backtick code spans (`[REQ-0605]`).
fn line_has_unbackticked_tier_a_marker(line: &str) -> bool {
    let backtick_count = line.bytes().filter(|&b| b == b'`').count();
    if backtick_count % 2 != 0 {
        return line.contains(EXPLICIT_TIER_A_MARKER);
    }
    line.split('`')
        .enumerate()
        .any(|(idx, seg)| idx % 2 == 0 && seg.contains(EXPLICIT_TIER_A_MARKER))
}

/// Returns `true` when `doc_block` carries a non-empty single-line `Realises: [<ID>]` tag (`[REQ-0605]`).
#[must_use]
pub fn has_non_empty_realises_tag(doc_block: &str) -> bool {
    let mut rest = doc_block;
    while let Some((_, after_prefix)) = rest.split_once("Realises: [") {
        let same_line = after_prefix
            .split_once('\n')
            .map_or(after_prefix, |(line, _)| line);
        if let Some((inside_brackets, _)) = same_line.split_once(']') {
            if !inside_brackets.trim().is_empty() {
                return true;
            }
        }
        rest = after_prefix;
    }
    false
}

/// Returns `true` when `doc_block` carries a valid `[REQ-...]`, `[FLS: fls_...]`, or `Realises: [...]` anchor (`[REQ-0605]`).
#[must_use]
pub fn has_traceability_anchor(doc_block: &str) -> bool {
    has_bracketed_prefix_anchor(doc_block, "[REQ-")
        || has_bracketed_prefix_anchor(doc_block, "[FLS: fls_")
        || has_bracketed_prefix_anchor(doc_block, "Realises: [")
}

/// Checks whether `doc_block` contains `prefix` closed on the same line by `']'` with alphanumeric ID text (`[REQ-0605]`).
fn has_bracketed_prefix_anchor(doc_block: &str, prefix: &str) -> bool {
    let mut rest = doc_block;
    while let Some((_, after_prefix)) = rest.split_once(prefix) {
        let same_line = after_prefix
            .split_once('\n')
            .map_or(after_prefix, |(line, _)| line);
        if let Some((inside, _)) = same_line.split_once(']') {
            if !inside.contains('[') && inside.bytes().any(|b| b.is_ascii_alphanumeric()) {
                return true;
            }
        }
        rest = after_prefix;
    }
    false
}

/// Strips bracketed `[REQ-...]`, `[FLS: ...]`, and `[DOC-TIER: ...]` tags so tag-only lines do not count as prose (`[REQ-0605]`).
#[must_use]
pub fn strip_tier_and_trace_tags(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some((before, after_open)) = rest.split_once('[') {
        out.push_str(before);
        let is_tag = after_open.starts_with("REQ-")
            || after_open.starts_with("FLS:")
            || after_open.starts_with("DOC-TIER:");
        if is_tag {
            if let Some((_, after_close)) = after_open.split_once(']') {
                rest = after_close;
                continue;
            }
        }
        out.push('[');
        rest = after_open;
    }
    out.push_str(rest);
    out
}

/// Normalizes `file_rel` separators and leading `./` prefix for `lookup_ratchet_ceiling` (`[REQ-0605]`).
#[must_use]
pub fn normalize_rel_path_for_ratchet(file_rel: &str) -> String {
    let slash_norm = file_rel.replace('\\', "/");
    let mut trimmed = slash_norm.trim();
    while let Some(rest) = trimmed.strip_prefix("./") {
        trimmed = rest;
    }
    if trimmed.starts_with("crates/") {
        format!("./{trimmed}")
    } else {
        trimmed.to_string()
    }
}

/// Returns `true` when a Tier-B/C docblock has `> 4` doc lines and `file_rel` is not in `ratchet_table` (`[REQ-0605]`).
#[must_use]
pub fn exceeds_unratcheted_tier_bc_line_cap(doc_line_count: usize, file_rel: &str) -> bool {
    if doc_line_count <= MAX_TIER_BC_DOC_LINES {
        return false;
    }
    let normalized_rel_path = normalize_rel_path_for_ratchet(file_rel);
    crate::checks::ratchet_table::lookup_ratchet_ceiling(&normalized_rel_path).is_none()
}

/// Unit tests covering Check 36 tier classification, traceability anchors, and anti-padding rejections (`[REQ-0605]`).
#[cfg(test)]
mod tests {
    use super::{
        exceeds_unratcheted_tier_bc_line_cap, has_explicit_tier_a_marker,
        has_non_empty_realises_tag, has_traceability_anchor, normalize_rel_path_for_ratchet,
        strip_tier_and_trace_tags,
    };
    use crate::checks::docstring_gate::audit_rust_source_docstrings;
    use crate::checks::dod_gates::count_executable_sloc;

    /// Verifies explicit `[DOC-TIER: A]` detection ignores backticked prose mentions (`[REQ-0605]`).
    #[test]
    fn explicit_tier_a_marker_requires_unbackticked_tag() {
        let tier_a = vec![String::from("/// [DOC-TIER: A] Public API contract.")];
        assert!(has_explicit_tier_a_marker(&tier_a));

        let backticked = vec![String::from("/// Mentions `[DOC-TIER: A]` in prose (`[REQ-0605]`).")];
        assert!(!has_explicit_tier_a_marker(&backticked));

        let ordinary_pub = vec![String::from("/// Concise summary (`[REQ-0605]`).")];
        assert!(!has_explicit_tier_a_marker(&ordinary_pub));
    }

    /// Verifies `[REQ-...]`, `[FLS: fls_...]`, `Realises: [...]`, and vacuous anchor rejection (`[REQ-0605]`).
    #[test]
    fn traceability_anchor_accepts_req_and_fls_and_rejects_vacuous_tags() {
        assert!(has_traceability_anchor("/// Concise summary (`[REQ-0605]`)."));
        assert!(has_traceability_anchor("/// Safety anchor [FLS: fls_unsafety_01]."));
        assert!(has_traceability_anchor("/// Summary.\n/// Realises: [REQ-CHECK-36]"));
        assert!(has_non_empty_realises_tag("/// Summary.\n/// Realises: [REQ-CHECK-36]"));
        assert!(!has_non_empty_realises_tag("/// Realises: [   ]"));
        assert!(!has_traceability_anchor("/// Summary without traceability tag."));
        assert!(!has_traceability_anchor("/// Empty tags [REQ-] [FLS: fls_] Realises: []."));
        assert!(!has_traceability_anchor("/// [DOC-TIER: A] Missing requirement anchor."));
    }

    /// Verifies tag stripping, `<= 4`-line Tier-B/C cap, ratchet exemption, and `< 300` SLOC ceiling (`[REQ-0605]`).
    #[test]
    fn tier_bc_line_cap_ratchet_lookup_and_module_sloc_bounds() -> Result<(), String> {
        let stripped = strip_tier_and_trace_tags("(`[REQ-0605]`, `[FLS: fls_01]`) [DOC-TIER: A]");
        assert!(!stripped.chars().any(char::is_alphanumeric));
        let with_prose = strip_tier_and_trace_tags("Valid summary (`[REQ-0605]`).");
        assert!(with_prose.chars().any(char::is_alphanumeric));

        assert_eq!(
            normalize_rel_path_for_ratchet("crates\\verify-baseline\\src\\runner.rs"),
            "./crates/verify-baseline/src/runner.rs"
        );
        assert!(!exceeds_unratcheted_tier_bc_line_cap(4, "crates/demo/src/lib.rs"));
        assert!(exceeds_unratcheted_tier_bc_line_cap(5, "crates/demo/src/lib.rs"));
        assert!(exceeds_unratcheted_tier_bc_line_cap(
            5,
            "crates/verify-baseline/src/checks/dod_gates.rs"
        ));
        assert!(!exceeds_unratcheted_tier_bc_line_cap(
            20,
            "crates/verify-baseline/src/checks/docstring_gate.rs"
        ));

        let sloc = count_executable_sloc(include_str!("docstring_tier_rules.rs"))?;
        if sloc >= 300 {
            return Err(format!("docstring_tier_rules.rs SLOC {sloc} exceeds < 300 bound"));
        }
        Ok(())
    }

    /// Verifies Check 36 rejects downgraded `[DOC-TIER: A]` items and `> 4`-line padded Tier-B/C docblocks (`[REQ-0605]`).
    #[test]
    fn rejects_downgraded_tier_a_and_padded_tier_bc_docstrings() {
        let downgraded_tier_a = concat!(
            "/// [DOC-TIER: A] Concise summary without 4-Block Contract or 7-Section headers.\n",
            "/// Realises: [REQ-DOWNGRADED-PUB-FN]\n",
            "pub fn downgraded_public_fn(v: usize) -> usize { v }\n\n",
            "/// [DOC-TIER: A] Generic holder documented with valid 4-Block Contract.\n",
            "/// Safety & Traceability: Realises: [REQ-GENERIC-HOLDER]\n",
            "/// Contract: Wraps `v`.\n",
            "/// Bounds & Faults: O(1) stack footprint.\n",
            "pub struct GenericHolder<T> { pub v: T }\n\n",
            "impl<T> GenericHolder<T> where for<'a> &'a T: Copy {\n",
            "    /// [DOC-TIER: A] Two-line docblock is insufficient for explicit `[DOC-TIER: A]`.\n",
            "    /// Realises: [REQ-HRTB-INHERENT-PUB]\n",
            "    pub fn borrow_ref(&self) -> &T { &self.v }\n",
            "}\n\n",
            "/// Padded Tier-B/C line 1 without explicit Tier-A opt-in.\n",
            "/// Padded Tier-B/C line 2.\n",
            "/// Padded Tier-B/C line 3.\n",
            "/// Padded Tier-B/C line 4.\n",
            "/// Padded Tier-B/C line 5 (`[REQ-PADDED-BC]`).\n",
            "pub fn padded_five_line_tier_bc() {}\n",
        );
        let errs = audit_rust_source_docstrings(downgraded_tier_a, "crates/demo/src/downgraded.rs")
            .err()
            .unwrap_or_default()
            .join("\n");
        assert!(
            errs.contains("downgraded_public_fn")
                && errs.contains("borrow_ref")
                && errs.contains("padded_five_line_tier_bc")
                && !errs.contains("GenericHolder"),
            "Unexpected Tier-A/cap diagnostics:\n{errs}"
        );
    }

    /// Verifies Check 36 rejects missing traceability tags, missing summaries, and placeholder markers (`[REQ-0605]`).
    #[test]
    fn rejects_missing_traceability_missing_summary_and_placeholders() {
        let missing_traceability = concat!(
            "/// Public 4-Block item missing Realises bracketed ID.\n",
            "/// Safety & Traceability: Verified manually without requirement tag.\n",
            "/// Contract: Deterministic execution.\n",
            "/// Bounds & Faults: O(1) time.\n",
            "pub fn pub_missing_realises() {}\n\n",
            "/// Private helper missing Realises tag.\n",
            "fn priv_missing_realises() {}\n\n",
            "/// Private helper with vacuous empty Realises brackets.\n",
            "/// Realises: []\n",
            "pub(crate) fn crate_empty_realises() {}\n\n",
            "/// Realises: [REQ-NO-SUMMARY]\n",
            "fn priv_missing_summary() {}\n",
        );
        let trace_errs =
            audit_rust_source_docstrings(missing_traceability, "crates/demo/src/trace.rs")
                .err()
                .unwrap_or_default()
                .join("\n");
        for sym in [
            "pub_missing_realises",
            "priv_missing_realises",
            "crate_empty_realises",
            "priv_missing_summary",
        ] {
            assert!(trace_errs.contains(sym), "Missing violation for '{sym}':\n{trace_errs}");
        }
        for marker in ["TODO", "TBD", "FIXME", "XXX"] {
            let item_src = format!(
                "/// Helper summary with {marker} marker.\n/// Realises: [REQ-MARKER-01]\nfn helper_under_test() {{}}\n"
            );
            let item_errs =
                audit_rust_source_docstrings(&item_src, "crates/demo/src/placeholder.rs")
                    .err()
                    .unwrap_or_default()
                    .join("\n");
            assert!(item_errs.contains("placeholder docstring"), "{marker}: {item_errs}");
            let mod_src = format!(
                "//! Module summary with {marker} marker.\n/// Valid helper summary.\n/// Realises: [REQ-MARKER-02]\nfn clean_helper() {{}}\n"
            );
            let mod_errs =
                audit_rust_source_docstrings(&mod_src, "crates/demo/src/mod_placeholder.rs")
                    .err()
                    .unwrap_or_default()
                    .join("\n");
            assert!(mod_errs.contains("placeholder module docstring") && !mod_errs.contains("clean_helper"));
        }
    }
}
