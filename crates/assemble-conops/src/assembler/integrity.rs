//! Unit integrity and unresolved parameter token validation for ConOps and Mission Intent units.

use std::fs;
use std::path::PathBuf;

use crate::params::SysMLParameterBindingEngine;

/// [DOC-TIER: A] Scans text for unresolved parameter tokens matching `{{KEY}}` or `{{KEY:default}}`.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-ASSEMBLER-FIND-RAW-TOKENS] | SysML v2: ActionDef(find_raw_tokens)
/// Contract: Returns `Vec<String>` of all single-line `{{...}}` tokens whose key consists of ASCII alphanumeric/underscore characters.
/// Bounds & Faults: O(N) time and space over `text.len()`; ignores multi-line braces and LaTeX math braces.
pub fn find_raw_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i + 4 <= bytes.len() {
        if bytes.get(i).copied() == Some(b'{') && bytes.get(i + 1).copied() == Some(b'{') {
            let start = i;
            let mut j = i + 2;
            let mut found_end = None;
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
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-ASSEMBLER-VALIDATE-UNIT-INTEGRITY] | SysML v2: ActionDef(validate_unit_integrity)
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
                errors.push(format!(
                    "Failed to read unit file '{}': {e}",
                    path.display()
                ));
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

        let placeholders: Vec<String> = find_raw_tokens(&processed);
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
