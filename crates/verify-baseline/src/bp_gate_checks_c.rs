#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Checks 8, 9, and 11 (BDD Acceptance Criteria, Rust Contract Code Blocks, and Source References) for `blueprint_gate`.
//! Realises: [REQ-BLUEPRINT-CHECK-8-BDD-TEST-TARGET], [REQ-BLUEPRINT-CHECK-9-RUST-CODE-BLOCKS], [REQ-BLUEPRINT-CHECK-11-SOURCE-REFERENCES]

use crate::bp_gate_tables::strip_markdown_decorations;
use std::path::Path;

/// Checks whether `target` starts with one of the 6 canonical upstream workspace crate prefixes.
/// Realises: [REQ-BLUEPRINT-CHECK-8-CRATE-WHITELIST]
fn is_allowed_upstream_crate_test_target(target: &str) -> bool {
    target.starts_with("crates/deap-core/")
        || target.starts_with("crates/verify-baseline/")
        || target.starts_with("crates/compile-sysml/")
        || target.starts_with("crates/ingest-sysml/")
        || target.starts_with("crates/reconcile-backlog/")
        || target.starts_with("crates/assemble-conops/")
}

/// Extracts an `AC-XX` identifier if `trimmed` begins an acceptance criterion heading or bullet.
/// Realises: [REQ-BLUEPRINT-CHECK-8-AC-HEADER]
fn extract_ac_header_id(trimmed: &str) -> Option<String> {
    let is_heading_or_bullet = trimmed.starts_with('#')
        || trimmed.starts_with("- **AC-")
        || trimmed.starts_with("* **AC-")
        || trimmed.starts_with("- AC-")
        || trimmed.starts_with("* AC-")
        || trimmed.starts_with("**AC-");
    if !is_heading_or_bullet {
        return None;
    }
    let stripped = trimmed
        .trim_start_matches('#')
        .trim_start_matches('-')
        .trim_start_matches('*')
        .trim()
        .trim_start_matches('*')
        .trim_start_matches('`')
        .trim();
    let rest = stripped.strip_prefix("AC-")?;
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    (!digits.is_empty()).then(|| format!("AC-{digits}"))
}

/// Checks whether an acceptance criteria text block contains Given/When/Then and negative keywords.
/// Realises: [REQ-BLUEPRINT-CHECK-8-AC-CLAUSES]
fn validate_single_ac_block(ac_id: &str, ac_lines: &[&str], errors: &mut Vec<String>) -> bool {
    let joined = ac_lines.join("\n");
    let has_given = joined.contains("**Given**") || joined.contains("Given:") || joined.contains("Given ");
    let has_when = joined.contains("**When**") || joined.contains("When:") || joined.contains("When ");
    let has_then = joined.contains("**Then**") || joined.contains("Then:") || joined.contains("Then ");

    if !has_given || !has_when || !has_then {
        errors.push(format!(
            "Check 8 violation (Section 5 {ac_id}): Acceptance criterion '{ac_id}' must contain Given, When, and Then clauses (Given={has_given}, When={has_when}, Then={has_then})"
        ));
    }

    let lower = joined.to_ascii_lowercase();
    let neg_tokens = [
        "negative", "fail-closed", "fail closed", "reject", "invalid", "violation", "error", "non-zero",
    ];
    neg_tokens.iter().any(|tok| lower.contains(tok))
}

/// Validates that a single Section 5 test-target path is a relative `.rs` file in an upstream crate.
/// Realises: [REQ-BLUEPRINT-CHECK-8-BDD-TEST-TARGET]
fn validate_test_target_path(target: &str, errors: &mut Vec<String>) {
    let valid_crate_prefix = is_allowed_upstream_crate_test_target(target);
    let valid_rs_ext = Path::new(target)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("rs"));
    if !valid_crate_prefix || !valid_rs_ext || target.contains("..") {
        errors.push(format!(
            "Check 8 violation (Section 5): test-target '{target}' must be a relative '.rs' path inside one of the 6 upstream crates (crates/deap-core/, crates/verify-baseline/, crates/compile-sysml/, crates/ingest-sysml/, crates/reconcile-backlog/, crates/assemble-conops/)"
        ));
    }
}

/// Collects every `<!-- test-target: ... -->` comment and `- **Test-Target**: ...` bullet in Section 5.
/// Realises: [REQ-BLUEPRINT-CHECK-8-BDD-TEST-TARGET]
fn collect_section_5_test_targets(section_5_lines: &[&str]) -> (Vec<String>, Vec<String>) {
    let mut comment_targets: Vec<String> = Vec::new();
    let mut bullet_targets: Vec<String> = Vec::new();
    for raw_line in section_5_lines {
        let trimmed = raw_line.trim();
        if let Some(inner) = trimmed.strip_prefix("<!--").and_then(|s| s.strip_suffix("-->")) {
            if let Some(rest) = inner.trim().strip_prefix("test-target:") {
                comment_targets.push(rest.trim().to_string());
            }
        }
        let bullet_stripped = trimmed
            .strip_prefix('-')
            .or_else(|| trimmed.strip_prefix('*'))
            .map_or(trimmed, str::trim);
        if let Some(rest) = bullet_stripped.strip_prefix("**Test-Target**:") {
            bullet_targets.push(strip_markdown_decorations(rest));
        }
    }
    (comment_targets, bullet_targets)
}

/// Reconciles multi-target `<!-- test-target: ... -->` comments against `- **Test-Target**: ...` bullets.
/// Realises: [REQ-BLUEPRINT-CHECK-8-BDD-TEST-TARGET]
fn reconcile_test_target_bindings(
    comment_targets: &[String],
    bullet_targets: &[String],
    errors: &mut Vec<String>,
) -> Vec<String> {
    if comment_targets.is_empty() && bullet_targets.is_empty() {
        errors.push("Check 8 violation (Section 5): Missing both '<!-- test-target: <path> -->' and '- **Test-Target**: `<path>`' declarations".to_string());
        return Vec::new();
    }
    if comment_targets.is_empty() {
        errors.push("Check 8 violation (Section 5): Missing mandatory '<!-- test-target: <path> -->' HTML comment binding".to_string());
    } else if bullet_targets.is_empty() {
        errors.push("Check 8 violation (Section 5): Missing mandatory '- **Test-Target**: `<path>`' field in Section 5".to_string());
    }
    if comment_targets.len() != bullet_targets.len() {
        errors.push(format!(
            "Check 8 violation (Section 5): test-target declaration count mismatch: {} HTML comment(s) vs {} '- **Test-Target**' bullet(s); both counts must be equal and non-zero",
            comment_targets.len(),
            bullet_targets.len()
        ));
    }
    for (idx, (c_path, b_path)) in comment_targets.iter().zip(bullet_targets.iter()).enumerate() {
        if c_path != b_path {
            errors.push(format!(
                "Check 8 violation (Section 5): test-target declaration #{} HTML comment '<!-- test-target: {c_path} -->' does not match '- **Test-Target**: `{b_path}`'",
                idx.saturating_add(1)
            ));
        }
    }

    let mut validated_paths: Vec<&String> = Vec::new();
    for target in comment_targets.iter().chain(bullet_targets.iter()) {
        if !validated_paths.contains(&target) {
            validated_paths.push(target);
        }
    }
    for target in validated_paths {
        validate_test_target_path(target, errors);
    }
    comment_targets.to_vec()
}

/// Check 8 (BDD Acceptance Criteria & Test-Target Gate): Validates Section 5 Given/When/Then ACs and multi-target bindings.
/// Realises: [REQ-BLUEPRINT-CHECK-8-BDD-TEST-TARGET]
pub fn check_8_bdd_and_test_target(
    section_5_lines: &[&str],
) -> Result<(usize, Vec<String>), Vec<String>> {
    let mut errors = Vec::new();
    let mut ac_blocks: Vec<(String, Vec<&str>)> = Vec::new();

    for raw_line in section_5_lines {
        let trimmed = raw_line.trim();
        if let Some(ac_id) = extract_ac_header_id(trimmed) {
            ac_blocks.push((ac_id, vec![*raw_line]));
        } else if let Some((_, ref mut lines_vec)) = ac_blocks.last_mut() {
            lines_vec.push(*raw_line);
        }
    }

    let mut distinct_ids: Vec<String> = Vec::new();
    let mut has_negative_scenario = false;
    for (ac_id, block_lines) in &ac_blocks {
        if !distinct_ids.contains(ac_id) {
            distinct_ids.push(ac_id.clone());
        }
        if validate_single_ac_block(ac_id, block_lines, &mut errors) {
            has_negative_scenario = true;
        }
    }

    if distinct_ids.len() < 5 {
        errors.push(format!(
            "Check 8 violation (Section 5): Section 5 must contain >= 5 distinct AC-XX acceptance criteria scenarios; found {}",
            distinct_ids.len()
        ));
    }
    if !has_negative_scenario {
        errors.push("Check 8 violation (Section 5): Section 5 must include >= 1 negative / fail-closed acceptance scenario".to_string());
    }

    let (comment_targets, bullet_targets) = collect_section_5_test_targets(section_5_lines);
    let verified_targets =
        reconcile_test_target_bindings(&comment_targets, &bullet_targets, &mut errors);
    if errors.is_empty() {
        Ok((distinct_ids.len(), verified_targets))
    } else {
        Err(errors)
    }
}

/// Checks whether `haystack` contains `prefix` immediately followed by `suffix`.
/// Realises: [REQ-VERIFY-034]
pub(crate) fn has_split_token(haystack: &str, prefix: &str, suffix: &str) -> bool {
    haystack.match_indices(prefix).any(|(idx, _)| {
        haystack
            .get(idx.saturating_add(prefix.len())..)
            .is_some_and(|rest| rest.starts_with(suffix))
    })
}

/// Scans a single Section 6 Rust contract line for Check 34, Clippy-forbid, and Check 19 domain tokens.
/// Realises: [REQ-BLUEPRINT-CHECK-9-LINE-SCANNER]
pub(crate) fn scan_rust_contract_line(
    raw_line: &str,
    line_no: usize,
    lex_state: &mut deap_core::rust_cst::source_bridge::CstLexState,
    errors: &mut Vec<String>,
) {
    const FORBIDDEN_REGEX_TOKENS: [(&str, &str); 5] = [
        ("re", "gex::"),
        ("Re", "gex::new"),
        ("Re", "gexSet"),
        ("use re", "gex"),
        ("extern crate r", "egex"),
    ];
    const FORBIDDEN_PANIC_TOKENS: [(&str, &str); 5] = [
        (".un", "wrap()"),
        (".ex", "pect("),
        ("pan", "ic!("),
        ("to", "do!("),
        ("unimple", "mented!("),
    ];
    const FORBIDDEN_DOMAIN_TOKENS: [(&str, &str, &str, &str); 9] = [
        ("FLIGHT_ENVE", "LOPE_MAP", "flight_enve", "lope_map"),
        ("STANDARD_MI", "SSION_PROFILES", "standard_mi", "ssion_profiles"),
        ("Tactic", "alUAV", "tactic", "aluav"),
        ("rudde", "rvator", "rudde", "rvator"),
        ("empen", "nage", "empen", "nage"),
        ("fusel", "age", "fusel", "age"),
        ("aile", "ron", "aile", "ron"),
        ("avion", "ics", "avion", "ics"),
        ("airf", "rame", "airf", "rame"),
    ];

    let code_only =
        deap_core::rust_cst::source_bridge::scan_line_with_unified_lex_state(raw_line, lex_state);

    if FORBIDDEN_REGEX_TOKENS
        .iter()
        .any(|(prefix, suffix)| has_split_token(&code_only, prefix, suffix))
    {
        errors.push(format!(
            "Check 9 violation (Section 6 line {line_no} -- Check 34 Zero Regex): Forbidden regex usage in Rust code block: '{}'",
            raw_line.trim()
        ));
    }

    if FORBIDDEN_PANIC_TOKENS
        .iter()
        .any(|(prefix, suffix)| has_split_token(&code_only, prefix, suffix))
    {
        errors.push(format!(
            "Check 9 violation (Section 6 line {line_no} -- Clippy-Forbid): Forbidden panicking macro or unwrap/expect call in Rust code block: '{}'",
            raw_line.trim()
        ));
    }

    if deap_core::rust_cst::source_bridge::contains_direct_range_slice(&code_only) {
        errors.push(format!(
            "Check 9 violation (Section 6 line {line_no} -- Clippy indexing_slicing): Direct '[a..b]' range slice indexing is forbidden; use '.get(a..b)' instead: '{}'",
            raw_line.trim()
        ));
    }

    let lower_raw = raw_line.to_ascii_lowercase();
    for (disp_prefix, disp_suffix, low_prefix, low_suffix) in FORBIDDEN_DOMAIN_TOKENS {
        if has_split_token(raw_line, disp_prefix, disp_suffix)
            || has_split_token(&lower_raw, low_prefix, low_suffix)
        {
            errors.push(format!(
                "Check 9 violation (Section 6 line {line_no} -- Check 19 Domain Cleanliness): Forbidden hardcoded domain token '{disp_prefix}{disp_suffix}' in Rust code block"
            ));
        }
    }
}

/// Rejects executable Rust items in a Section 6 declaration-only code block, recursing into inline modules.
/// Realises: [REQ-BLUEPRINT-CHECK-9-RUST-CODE-BLOCKS]
fn validate_declaration_only_item(item: &syn::Item, errors: &mut Vec<String>) {
    match item {
        syn::Item::Fn(item_fn) => errors.push(format!(
            "Check 9 violation (Section 6 -- Declaration-Only Contract): function body '{}' is forbidden; Section 6 must contain declarations only",
            item_fn.sig.ident
        )),
        syn::Item::Impl(_) => errors.push(
            "Check 9 violation (Section 6 -- Declaration-Only Contract): 'impl' blocks are forbidden; use bodyless trait declarations".to_string(),
        ),
        syn::Item::Macro(_) => errors.push(
            "Check 9 violation (Section 6 -- Declaration-Only Contract): macro definitions and invocations are forbidden".to_string(),
        ),
        syn::Item::Verbatim(tokens) => errors.push(format!(
            "Check 9 violation (Section 6 -- Declaration-Only Contract): unrecognized/verbatim item is forbidden: '{tokens}'"
        )),
        syn::Item::Trait(item_trait) => {
            for trait_item in &item_trait.items {
                match trait_item {
                    syn::TraitItem::Fn(trait_fn) if trait_fn.default.is_some() => errors.push(format!(
                        "Check 9 violation (Section 6 -- Declaration-Only Contract): trait method '{}' has a default body; Section 6 trait methods must be bodyless declarations",
                        trait_fn.sig.ident
                    )),
                    syn::TraitItem::Macro(_) => errors.push(
                        "Check 9 violation (Section 6 -- Declaration-Only Contract): trait macro items are forbidden".to_string(),
                    ),
                    syn::TraitItem::Verbatim(tokens) => errors.push(format!(
                        "Check 9 violation (Section 6 -- Declaration-Only Contract): unrecognized/verbatim trait item is forbidden: '{tokens}'"
                    )),
                    _ => {}
                }
            }
        }
        syn::Item::Mod(item_mod) => {
            if let Some((_, items)) = &item_mod.content {
                for nested in items {
                    validate_declaration_only_item(nested, errors);
                }
            }
        }
        syn::Item::Use(_)
        | syn::Item::Struct(_)
        | syn::Item::Enum(_)
        | syn::Item::Union(_)
        | syn::Item::Type(_)
        | syn::Item::Const(_)
        | syn::Item::Static(_)
        | syn::Item::ForeignMod(_) => {}
        _ => errors.push(
            "Check 9 violation (Section 6 -- Declaration-Only Contract): item is not permitted; allowed declarations are use/struct/enum/union/type/const/static/trait/external mod/foreign mod".to_string(),
        ),
    }
}

/// Parses a Section 6 Rust block with `syn` and fails closed on parse errors or executable items.
/// Realises: [REQ-BLUEPRINT-CHECK-9-RUST-CODE-BLOCKS]
fn validate_declaration_only_block(source: &str, start_line: usize, errors: &mut Vec<String>) {
    match syn::parse_file(source) {
        Ok(file) => {
            for item in &file.items {
                validate_declaration_only_item(item, errors);
            }
        }
        Err(parse_err) => errors.push(format!(
            "Check 9 violation (Section 6 line {start_line} -- Declaration-Only Contract): Rust code block failed to parse: {parse_err}"
        )),
    }
}

/// Check 9 (Check 34 Zero Regex, Check 19 Domain Cleanliness & Clippy-Forbid Code-Block Gate).
/// Realises: [REQ-BLUEPRINT-CHECK-9-RUST-CODE-BLOCKS]
pub fn check_9_rust_code_blocks(section_6_lines: &[&str]) -> Result<usize, Vec<String>> {
    let mut errors = Vec::new();
    let mut in_rust_block = false;
    let mut in_other_fence = false;
    let mut rust_block_count = 0usize;
    let mut lex_state = deap_core::rust_cst::source_bridge::CstLexState::default();
    let mut block_source = String::new();

    for (idx, raw_line) in section_6_lines.iter().enumerate() {
        let line_no = idx.saturating_add(1);
        let trimmed = raw_line.trim();

        if !in_rust_block && !in_other_fence {
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                let lang = trimmed.trim_start_matches('`').trim_start_matches('~').trim();
                let first_token = lang.split_whitespace().next().unwrap_or("");
                if first_token.eq_ignore_ascii_case("rust") {
                    in_rust_block = true;
                    rust_block_count = rust_block_count.saturating_add(1);
                    lex_state = deap_core::rust_cst::source_bridge::CstLexState::default();
                } else {
                    in_other_fence = true;
                }
            }
            continue;
        }
        if in_other_fence {
            if trimmed == "```" || trimmed == "~~~" {
                in_other_fence = false;
            }
            continue;
        }
        if trimmed == "```" || trimmed == "~~~" {
            in_rust_block = false;
            continue;
        }
        scan_rust_contract_line(raw_line, line_no, &mut lex_state, &mut errors);
        block_source.push_str(raw_line);
        block_source.push('\n');
    }

    if !block_source.is_empty() {
        validate_declaration_only_block(&block_source, 1, &mut errors);
    }
    if in_rust_block || in_other_fence {
        errors.push("Check 9 violation (Section 6): Unclosed code fence at end of Section 6".to_string());
    }
    if rust_block_count == 0 {
        errors.push(
            "Check 9 violation (Section 6): Section 6 ('## 6. Proposed Deterministic Implementation Contract') must contain >= 1 fenced ```rust block"
                .to_string(),
        );
    }

    if errors.is_empty() {
        Ok(rust_block_count)
    } else {
        Err(errors)
    }
}
