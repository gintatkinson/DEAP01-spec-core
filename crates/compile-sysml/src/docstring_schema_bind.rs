#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! [DOC-TIER: A] SysML v2 schema requirement/constraint binding and unforced conflict verification.
//! Safety & Traceability: Realises: [REQ-FEAT-42-RESOLVE-SCHEMA-CONTEXT], [REQ-0605], [REQ-DEFECT-09-DOCSTRING-CLI-EXIT-CODES]
//! Contract: Binds Rust and Dart AST declarations to SysML v2 `PartDef`, `ActionDef`, `ConstraintDef`, and `RequirementDef` nodes.
//! Bounds & Faults: O(P + A + R + C) lookup time bounded by `MAX_AST_DEPTH` (`64`); fails closed on unforced conflicts.

use crate::docstring::DocstringError;
use crate::docstring_cst::extract_realises_ids_from_doc_lines;
use crate::docstring_schema_index::SchemaLookupIndex;
use deap_core::sysml_ast::{PackageDef, PartDef, RequirementDef};

/// Maximum AST nesting depth traversed when collecting SysML v2 packages and nested parts.
/// Realises: [REQ-FEAT-42-MAX-AST-DEPTH]
pub const MAX_AST_DEPTH: usize = 64;

/// [DOC-TIER: A] Collects all top-level and nested `PartDef` nodes within `pkg` into `out` up to `MAX_AST_DEPTH`.
/// Safety & Traceability: Realises: [REQ-0605], [REQ-FEAT-42-COLLECT-ALL-PARTS]
/// Contract: Visits `PartDef` nodes in pre-order without recursion and stops cleanly when `depth > MAX_AST_DEPTH`.
/// Bounds & Faults: O(P) time and space in visited `PartDef` count `P`; never panics.
pub fn collect_all_parts<'a>(pkg: &'a PackageDef, out: &mut Vec<&'a PartDef>) {
    collect_all_parts_from_depth(pkg, 0, out);
}

/// Appends `PartDef` nodes from `pkg` starting at `initial_depth` into `out`, stopping when `depth > MAX_AST_DEPTH`.
/// Realises: [REQ-FEAT-42-COLLECT-ALL-PARTS-BOUNDED]
pub(crate) fn collect_all_parts_from_depth<'a>(
    pkg: &'a PackageDef,
    initial_depth: usize,
    out: &mut Vec<&'a PartDef>,
) {
    if initial_depth > MAX_AST_DEPTH {
        return;
    }
    let mut stack: Vec<(&'a PartDef, usize)> = pkg
        .part_defs
        .iter()
        .rev()
        .map(|part| (part, initial_depth))
        .collect();
    while let Some((part, depth)) = stack.pop() {
        if depth > MAX_AST_DEPTH {
            continue;
        }
        out.push(part);
        let next_depth = depth.saturating_add(1);
        if next_depth <= MAX_AST_DEPTH {
            for child in part.parts.iter().rev() {
                stack.push((child, next_depth));
            }
        }
    }
}

/// Validates that an existing unforced `Realises:` binding does not conflict with `schema_model`.
/// Realises: [REQ-FEAT-42-VALIDATE-UNFORCED-SCHEMA-REALISES]
pub(crate) fn validate_unforced_schema_realises(
    file_rel: &str,
    item_name: &str,
    current_doc: &[String],
    schema_model: Option<&PackageDef>,
    owner_name: Option<&str>,
) -> Result<(), DocstringError> {
    let (schema_req_opt, _) = resolve_schema_context(schema_model, owner_name, item_name);
    let Some(schema_req) = schema_req_opt else {
        return Ok(());
    };
    let existing_reqs = extract_all_realises_ids(current_doc);
    if existing_reqs.is_empty() {
        return Ok(());
    }
    let all_matched = schema_req
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .all(|sr| existing_reqs.iter().any(|r| schema_identifiers_match(r, sr)));
    if !all_matched {
        return Err(DocstringError::UnforcedConflict(format!(
            "Conflicting unforced docstring in '{file_rel}' for '{item_name}': existing Realises {existing_reqs:?} conflicts with schema requirement '{schema_req}' (use --force to overwrite)"
        )));
    }
    Ok(())
}

/// Extracts all requirement IDs from `Realises: [...]` tags across `doc_lines`.
/// Realises: [REQ-FEAT-42-EXTRACT-ALL-REALISES-IDS]
#[must_use]
pub(crate) fn extract_all_realises_ids(doc_lines: &[String]) -> Vec<String> {
    let mut ids = extract_realises_ids_from_doc_lines(doc_lines);
    for line in doc_lines {
        if let Some((_, after_realises)) = line.split_once("Realises:") {
            let mut rest = after_realises;
            while let Some(open_idx) = rest.find('[') {
                let after_open = rest.get(open_idx.saturating_add(1)..).unwrap_or("");
                let Some(close_idx) = after_open.find(']') else {
                    break;
                };
                for tok in after_open.get(..close_idx).unwrap_or("").split(',') {
                    let clean = tok.trim();
                    if !clean.is_empty() && !ids.iter().any(|existing| existing == clean) {
                        ids.push(clean.to_string());
                    }
                }
                rest = after_open.get(close_idx.saturating_add(1)..).unwrap_or("");
            }
        }
    }
    ids
}

/// Checks whether a SysML `ConstraintDef` name (`c_name`) matches an `ActionDef` or callable name (`act_name`).
/// Realises: [REQ-FEAT-42-CONSTRAINT-MATCHES-ACTION]
#[must_use]
pub(crate) fn constraint_matches_action(c_name: &str, act_name: &str) -> bool {
    if schema_identifiers_match(c_name, act_name) {
        return true;
    }
    if let Some(rest) = c_name
        .strip_prefix("invariant_")
        .or_else(|| c_name.strip_prefix("INVARIANT_"))
    {
        if schema_identifiers_match(rest, act_name) {
            return true;
        }
        if let Some((prefix, _)) = rest.split_once('_') {
            if schema_identifiers_match(prefix, act_name) {
                return true;
            }
        }
        if let Some((prefix, _)) = rest.rsplit_once('_') {
            if schema_identifiers_match(prefix, act_name) {
                return true;
            }
        }
    }
    false
}

/// Formats a SysML constraint `(name, expr)` pair into a human-readable docstring annotation.
/// Realises: [REQ-FEAT-42-FORMAT-SCHEMA-CONSTRAINT]
#[must_use]
pub(crate) fn format_schema_constraint(name: &str, expr: &str) -> Option<String> {
    let (n, e) = (name.trim(), expr.trim());
    match (!n.is_empty(), !e.is_empty()) {
        (true, true) => Some(format!("{n} ({e})")),
        (false, true) => Some(e.to_string()),
        (true, false) => Some(n.to_string()),
        (false, false) => None,
    }
}

/// Normalizes an identifier to lowercase ASCII alphanumeric characters for fuzzy schema matching.
/// Realises: [REQ-FEAT-42-NORMALIZE-ASCII-ALNUM]
#[must_use]
pub(crate) fn normalize_ascii_alnum_lower(s: &str) -> String {
    s.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// Appends `token` to `reqs` if non-empty and not already matched via `schema_identifiers_match`.
/// Realises: [REQ-FEAT-42-PUSH-UNIQUE-SCHEMA-REQ]
pub(crate) fn push_unique_schema_req(reqs: &mut Vec<String>, token: String) {
    let trimmed = token.trim();
    if !trimmed.is_empty() && !reqs.iter().any(|ex| schema_identifiers_match(ex, trimmed)) {
        reqs.push(trimmed.to_string());
    }
}

/// Appends `expr` to `constraints` if non-empty and not already present verbatim.
/// Realises: [REQ-FEAT-42-PUSH-UNIQUE-SCHEMA-CONSTRAINT]
pub(crate) fn push_unique_schema_constraint(constraints: &mut Vec<String>, expr: String) {
    let trimmed = expr.trim();
    if !trimmed.is_empty() && !constraints.iter().any(|existing| existing == trimmed) {
        constraints.push(trimmed.to_string());
    }
}

/// Returns the authoritative requirement identifier (`req_id` when non-empty, otherwise `name`) of `req_def`.
/// Realises: [REQ-FEAT-42-REQ-DEF-PRIMARY-ID]
#[must_use]
pub(crate) fn req_def_primary_id(req_def: &RequirementDef) -> &str {
    if !req_def.req_id.is_empty() {
        req_def.req_id.as_str()
    } else {
        req_def.name.as_str()
    }
}

/// Checks whether `part` explicitly satisfies or owns `r`.
/// Realises: [REQ-FEAT-42-PART-SATISFIES-REQ-DEF]
#[must_use]
pub(crate) fn part_satisfies_requirement_def(part: &PartDef, r: &RequirementDef) -> bool {
    let in_satisfied = part.satisfied_requirements.iter().any(|sr| {
        schema_identifiers_match(sr, &r.name)
            || (!r.req_id.is_empty() && schema_identifiers_match(sr, &r.req_id))
    });
    let in_safety = part.safety_realises.iter().any(|sr| {
        schema_identifiers_match(sr, &r.name)
            || (!r.req_id.is_empty() && schema_identifiers_match(sr, &r.req_id))
    });
    let in_inline = part.requirements.iter().any(|pr| {
        schema_identifiers_match(&pr.name, &r.name)
            || (!r.req_id.is_empty()
                && (schema_identifiers_match(&pr.req_id, &r.req_id)
                    || schema_identifiers_match(&pr.name, &r.req_id)))
    });
    let in_satisfied_by = r
        .satisfied_by
        .iter()
        .any(|sb| schema_identifiers_match(sb, &part.name));
    in_satisfied || in_safety || in_inline || in_satisfied_by
}

/// [DOC-TIER: A] Resolves optional requirement ID and constraint expression from `schema_model`.
/// Safety & Traceability: Realises: [REQ-FEAT-42-RESOLVE-SCHEMA-CONTEXT], [REQ-0605]
/// Contract: Searches `schema_model` up to `MAX_AST_DEPTH` with strict owner part isolation.
/// Bounds & Faults: O(P + A + R + C) time and O(P) space; never panics.
#[must_use]
pub fn resolve_schema_context(
    schema_model: Option<&PackageDef>,
    owner_name: Option<&str>,
    item_name: &str,
) -> (Option<String>, Option<String>) {
    let Some(root_pkg) = schema_model else {
        return (None, None);
    };
    let index = SchemaLookupIndex::new(root_pkg, owner_name, item_name);
    let (mut matched_req, mut matched_constraint) = if let Some(owner) = owner_name {
        let Some(owned_res) = index.resolve_owned_item(owner, item_name) else {
            return (None, None);
        };
        owned_res
    } else if let Some(part) = index
        .all_parts
        .iter()
        .copied()
        .find(|p| schema_identifiers_match(&p.name, item_name))
    {
        index.extract_part_req_and_constraint(part)
    } else {
        index.resolve_pkg_action(item_name, true).unwrap_or((None, None))
    };

    let target_ident = owner_name.unwrap_or(item_name);
    let (fb_req, fb_const) = index.resolve_fallbacks(
        target_ident,
        item_name,
        matched_req.is_none(),
        matched_constraint.is_none(),
    );
    if matched_req.is_none() {
        matched_req = fb_req;
    }
    if matched_constraint.is_none() {
        matched_constraint = fb_const;
    }
    (matched_req, matched_constraint)
}

/// [DOC-TIER: A] Resolves optional requirement ID and constraint expression with strict `owner_name` isolation.
/// Safety & Traceability: Realises: [REQ-0605], [REQ-FEAT-42-RESOLVE-SCHEMA-CONTEXT]
/// Contract: Returns `(None, None)` when `owner_name` is `Some(owner)` and `owner` is unmapped in `schema_model`.
/// Bounds & Faults: O(P + A + R + C) time and O(P) space; infallible.
#[must_use]
pub fn resolve_schema_context_strict(
    schema_model: Option<&PackageDef>,
    owner_name: Option<&str>,
    item_name: &str,
) -> (Option<String>, Option<String>) {
    resolve_schema_context(schema_model, owner_name, item_name)
}

/// Compares a schema element identifier (`schema_ident`) against a source AST identifier (`source_ident`).
/// Realises: [REQ-FEAT-42-SCHEMA-IDENTIFIERS-MATCH]
#[must_use]
pub(crate) fn schema_identifiers_match(schema_ident: &str, source_ident: &str) -> bool {
    let mut a_iter = schema_ident
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase());
    let mut b_iter = source_ident
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase());
    let mut saw_any = false;
    loop {
        match (a_iter.next(), b_iter.next()) {
            (Some(ca), Some(cb)) => {
                if ca != cb {
                    return false;
                }
                saw_any = true;
            }
            (None, None) => return saw_any,
            _ => return false,
        }
    }
}
