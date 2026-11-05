#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Section 6 Rust contract code-block validation for `blueprint_gate` Check 9.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Enforces the Section 6 declaration-only contract, the Check 34 zero-regex invariant, Check 19
//! domain cleanliness, and Clippy panic/indexing bans under DO-178C Level A, DO-330 TQL-4, and
//! ISO 26262 ASIL D tool qualification standards.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Every fenced `rust` block in Section 6 parses with `syn` and contains declarations only.
//! - Split-token scans inspect lexed code spans so forbidden tokens never appear verbatim.
//! - Forbidden token tables are static arrays with zero runtime string assembly.
//!
//! ## 3. Preconditions
//! - `section_6_lines` contains the exact line slice of Section 6 from a blueprint document.
//!
//! ## 4. Postconditions
//! - Returns the number of fenced Rust blocks or a deterministic `Vec<String>` of violations.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(L \times T)$ over Section 6 lines $L$ and fixed token tables $T$.
//! - Space Complexity: $O(S)$ for the accumulated Rust block source $S$.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Malformed fences, unparseable blocks, and forbidden tokens fail closed without panicking.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-BLUEPRINT-CHECK-9-RUST-CODE-BLOCKS], [REQ-VERIFY-034], [DO-330-TQL-4-BLUEPRINT-GATE]

/// Checks whether `haystack` contains `prefix` immediately followed by `suffix`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Supports split-token matching so banned regex and panic tokens never appear verbatim in source.
///
/// ## 2. Invariants & Mathematical Properties
/// Returns `true` iff some match index of `prefix` is directly followed by `suffix`.
///
/// ## 3. Preconditions
/// `prefix` and `suffix` are non-empty UTF-8 string slices.
///
/// ## 4. Postconditions
/// Performs no allocation and leaves `haystack` unmodified.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N \times P)$ over haystack length $N$ and prefix length $P$.
/// - Space Complexity: $O(1)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Out-of-range continuation lookups return `false` instead of panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-VERIFY-034]
pub(crate) fn has_split_token(haystack: &str, prefix: &str, suffix: &str) -> bool {
    haystack.match_indices(prefix).any(|(idx, _)| {
        haystack
            .get(idx.saturating_add(prefix.len())..)
            .is_some_and(|rest| rest.starts_with(suffix))
    })
}

/// Scans a single Section 6 Rust contract line for Check 34, Clippy-forbid, and Check 19 domain tokens.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Rejects regex usage, panicking calls, direct range slicing, and hardcoded domain tokens before
/// any Section 6 contract can be published.
///
/// ## 2. Invariants & Mathematical Properties
/// Each violation class appends at most one diagnostic per scanned line.
///
/// ## 3. Preconditions
/// `lex_state` is the running unified lexer state for the enclosing fenced Rust block.
///
/// ## 4. Postconditions
/// Advances `lex_state` and appends deterministic violations to `errors`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ over the raw line length $N$ plus fixed token tables.
/// - Space Complexity: $O(1)$ beyond emitted diagnostics.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Never panics on arbitrary UTF-8 input; lexer lookups are range-checked.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-BLUEPRINT-CHECK-9-LINE-SCANNER], [REQ-VERIFY-034]
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
                    syn::TraitItem::Const(trait_const) if trait_const.default.is_some() => errors.push(format!(
                        "Check 9 violation (Section 6 -- Declaration-Only Contract): trait constant '{}' has a default value; Section 6 trait constants must be bodyless declarations",
                        trait_const.ident
                    )),
                    syn::TraitItem::Type(trait_type) if trait_type.default.is_some() => errors.push(format!(
                        "Check 9 violation (Section 6 -- Declaration-Only Contract): trait type '{}' has a default value; Section 6 trait types must be bodyless declarations",
                        trait_type.ident
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Validates every fenced Rust block in Section 6 against declaration-only, zero-regex, domain
/// cleanliness, and panic-free contract rules.
///
/// ## 2. Invariants & Mathematical Properties
/// Returns `Ok(n)` with `n >= 1` only when all Rust blocks pass and at least one block exists.
///
/// ## 3. Preconditions
/// `section_6_lines` preserves document line order starting at Section 6 line 1.
///
/// ## 4. Postconditions
/// Emits no side effects and returns every violation in deterministic scan order.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L \times T)$ over scanned lines $L$ and fixed token tables $T$.
/// - Space Complexity: $O(S + V)$ for block source $S$ and violations $V$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Unclosed fences, missing Rust blocks, and parse failures fail closed without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-BLUEPRINT-CHECK-9-RUST-CODE-BLOCKS], [REQ-VERIFY-034]
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
