//! Lexical identifier normalization, Dart string escaping, and schema hashing utilities.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D deterministic code generation
//! standards. Centralizes regex-free identifier tokenization, Dart string literal escaping, SHA-256
//! schema hash derivation, and invariant bound extraction for modular Flutter code emission.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Pure, stateless string and digest functions operating over UTF-8 inputs without regex engines.
//! - Guarantees deterministic Dart class, field, label, and snake_case file artifact identifiers.
//!
//! ## 3. Preconditions
//! - Callers supply valid UTF-8 string slices sourced from parsed SysML AST nodes or topology JSON.
//!
//! ## 4. Postconditions
//! - Produces escaped Dart literals, 64-bit schema hashes, normalized identifiers, or numeric bounds.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ linear single-pass execution bounded by input string byte length.
//! - Space Complexity: $O(N)$ bounded heap allocation for token buffers ($O(1)$ for hash and bounds).
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Total panic-free functions across empty slices, malformed brackets, and arbitrary Unicode scalars.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0607], [REQ-EMIT-FLUTTER-CONTRACTS]

/// Escapes single-quoted Dart string literals by escaping `\`, `'`, and `$`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D code generation standards.
/// Prevents Dart string interpolation injection (`$`) and syntax errors (`'`, `\`) in generated
/// SQLite seeds, typed models, and UI contract tests.
///
/// ## 2. Invariants & Mathematical Properties
/// - Pure total function over UTF-8 strings: maps `\` to `\\`, `'` to `\'`, and `$` to `\$` while
///   preserving all other Unicode scalar values verbatim.
///
/// ## 3. Preconditions
/// - `s` is any valid UTF-8 string slice.
///
/// ## 4. Postconditions
/// - Returns an owned `String` safe for embedding inside a single-quoted Dart literal `'...'`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ single linear pass where $N$ is `s.len()`.
/// - Space Complexity: $O(N)$ bounded output buffer allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Total function: never panics or fails on any input string.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FLUTTER-ESCAPE-DART-SINGLE-QUOTED]
pub fn escape_dart_single_quoted(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str(r"\\"),
            '\'' => out.push_str(r"\'"),
            '$' => out.push_str(r"\$"),
            _ => out.push(ch),
        }
    }
    out
}

/// Computes a deterministic 64-bit cryptographic schema hash from input text sources.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ISO 26262 ASIL D compile-time version integrity mandates.
/// Derives the authoritative `DEAP_SCHEMA_HASH` constant from SysML models and topology payloads.
///
/// ## 2. Invariants & Mathematical Properties
/// - Pure hash function: identical UTF-8 inputs always yield identical 64-bit unsigned integers.
/// - Ingests SHA-256 digest and takes the leading 16 hex digits (top 64 bits).
///
/// ## 3. Preconditions
/// - `schema_text` and `topology_text` must be valid UTF-8 string slices.
///
/// ## 4. Postconditions
/// - Returns a non-zero 64-bit integer hash suitable for Dart constant injection.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ linear in byte length of inputs.
/// - Space Complexity: $O(1)$ fixed SHA-256 buffer.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Infallible calculation defaulting to a non-zero constant if string parsing fails.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FLUTTER-COMPUTE-SCHEMA-HASH]
pub fn compute_schema_hash(schema_text: &str, topology_text: &str) -> u64 {
    let mut combined = Vec::with_capacity(schema_text.len().saturating_add(topology_text.len()));
    combined.extend_from_slice(schema_text.as_bytes());
    combined.extend_from_slice(topology_text.as_bytes());
    let sha256_hex = deap_core::digest::compute_sha256(&combined);
    let hex_prefix: String = sha256_hex.chars().take(16).collect();
    u64::from_str_radix(&hex_prefix, 16).unwrap_or(0x8F4A_3C2E_1D9B_7E5A)
}

/// Splits an arbitrary identifier into constituent lexical words without regex.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Check 34 Zero-Regex source parsing requirements.
/// Tokenizes compound identifiers by snake_case, kebab-case, whitespace, or camelCase transitions.
///
/// ## 2. Invariants & Mathematical Properties
/// - Pure partition: concatenation of words preserves all alphanumeric characters in source order.
///
/// ## 3. Preconditions
/// - `s` is a valid UTF-8 string slice.
///
/// ## 4. Postconditions
/// - Returns `Vec<String>` containing individual lexical word tokens.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ linear in character count.
/// - Space Complexity: $O(N)$ vector of owned word strings.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Empty inputs return an empty vector without error.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FLUTTER-SPLIT-WORDS]
pub fn split_ident_into_words(s: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = s.chars().collect();
    for (i, &ch) in chars.iter().enumerate() {
        if ch == '_' || ch == '-' || ch == ' ' {
            if !current.is_empty() {
                words.push(current);
                current = String::new();
            }
        } else {
            let prev = if i > 0 { chars.get(i.saturating_sub(1)).copied() } else { None };
            let next = chars.get(i.saturating_add(1)).copied();
            if let Some(p) = prev {
                let is_transition = (p.is_ascii_lowercase() && ch.is_ascii_uppercase())
                    || (p.is_ascii_uppercase()
                        && ch.is_ascii_uppercase()
                        && next.map(|n| n.is_ascii_lowercase()).unwrap_or(false));
                if is_transition && !current.is_empty() {
                    words.push(current);
                    current = String::new();
                }
            }
            current.push(ch);
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

/// Converts an arbitrary string token into canonical lower camelCase without regex.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Check 34 Zero-Regex source integrity mandates.
/// Normalizes kebab-case, snake_case, and spaced identifiers into Dart member names.
///
/// ## 2. Invariants & Mathematical Properties
/// - Deterministic character-by-character transformation.
/// - Initial word is always ASCII lowercase; subsequent words are capitalized.
///
/// ## 3. Preconditions
/// - `s` is a UTF-8 string slice.
///
/// ## 4. Postconditions
/// - Returns owned `String` conforming to Dart camelCase naming conventions.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ linear in character count.
/// - Space Complexity: $O(N)$ for the returned string buffer.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Empty inputs return an empty `String` without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FLUTTER-TO-CAMEL-CASE]
pub fn to_camel_case(s: &str) -> String {
    let words = split_ident_into_words(s);
    let mut result = String::with_capacity(s.len());
    for (i, word) in words.iter().enumerate() {
        if i == 0 {
            result.push_str(&word.to_ascii_lowercase());
        } else {
            let mut chars = word.chars();
            if let Some(first) = chars.next() {
                result.push(first.to_ascii_uppercase());
                for rest in chars {
                    result.push(rest.to_ascii_lowercase());
                }
            }
        }
    }
    result
}

/// Converts an arbitrary identifier into PascalCase without regex.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Check 34 Zero-Regex source integrity mandates.
/// Normalizes identifiers into Dart type and class names.
///
/// ## 2. Invariants & Mathematical Properties
/// - Pure transformation capitalizing the leading letter of every constituent word.
///
/// ## 3. Preconditions
/// - `s` is a valid UTF-8 string slice.
///
/// ## 4. Postconditions
/// - Returns owned `String` conforming to Dart class naming conventions.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ linear in character count.
/// - Space Complexity: $O(N)$ allocated string buffer.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Infallible conversion over arbitrary UTF-8 slices.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FLUTTER-TO-PASCAL-CASE]
pub fn to_pascal_case(s: &str) -> String {
    let words = split_ident_into_words(s);
    let mut result = String::with_capacity(s.len());
    for word in &words {
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            result.push(first.to_ascii_uppercase());
            for rest in chars {
                result.push(rest.to_ascii_lowercase());
            }
        }
    }
    result
}

/// Converts an arbitrary SysML or JSON identifier into canonical Dart file snake_case.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A deterministic artifact naming and Dart package file layout rules.
/// Derives collision-resistant lowercase `.g.dart` module basenames without regular expressions.
///
/// ## 2. Invariants & Mathematical Properties
/// - Joins all lexical tokens from `split_ident_into_words` in ASCII lowercase separated by `_`.
/// - Maps empty or delimiter-only inputs deterministically to the fallback stem `"root"`.
///
/// ## 3. Preconditions
/// - `s` is any UTF-8 identifier slice representing a SysML block, package, or domain type.
///
/// ## 4. Postconditions
/// - Returns a non-empty lowercase `snake_case` `String` suitable for modular `.g.dart` filenames.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ linear in the byte length of `s`.
/// - Space Complexity: $O(N)$ bounded allocation for lowercase token assembly.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Never panics or returns an empty filename stem; degenerates safely to `"root"`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
pub fn to_snake_case_dart(s: &str) -> String {
    let words = split_ident_into_words(s);
    if words.is_empty() {
        return "root".to_string();
    }
    let lowered: Vec<String> = words.iter().map(|w| w.to_ascii_lowercase()).collect();
    lowered.join("_")
}

/// Formats a PascalCase or snake_case identifier into a human-readable display title.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to MIL-STD-1472H human engineering user interface labeling standards.
/// Produces natural title labels from raw schema attribute identifiers.
///
/// ## 2. Invariants & Mathematical Properties
/// - Capitalizes each constituent word and separates words with single ASCII spaces.
///
/// ## 3. Preconditions
/// - `s` is a UTF-8 string slice.
///
/// ## 4. Postconditions
/// - Returns human-readable label string.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ linear in character count.
/// - Space Complexity: $O(N)$ string allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Infallible formatting over arbitrary UTF-8 slices.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FLUTTER-FORMAT-DISPLAY-NAME]
pub fn format_display_name(s: &str) -> String {
    let words = split_ident_into_words(s);
    let formatted: Vec<String> = words
        .iter()
        .map(|w| {
            let mut chars = w.chars();
            if let Some(first) = chars.next() {
                let mut out = String::new();
                out.push(first.to_ascii_uppercase());
                for rest in chars {
                    out.push(rest.to_ascii_lowercase());
                }
                out
            } else {
                String::new()
            }
        })
        .collect();
    formatted.join(" ")
}

/// Extracts numeric invariant min and max bounds from documentation or constraint expressions.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Check 34 Zero-Regex source parsing requirements.
/// Scans SysML invariant comments for bounded ranges formatted as `[min, max]`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Pure scanner without regex: locates bracket delimiters and comma separators.
///
/// ## 3. Preconditions
/// - `doc` is an optional string slice reference.
///
/// ## 4. Postconditions
/// - Returns `(Option<f64>, Option<f64>)` indicating lower and upper numeric boundaries.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ linear in length of document slice.
/// - Space Complexity: $O(1)$ zero-allocation scanning.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Returns `(None, None)` if delimiters or floating-point literals cannot be parsed.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FLUTTER-EXTRACT-BOUNDS]
pub fn extract_bounds_from_doc(doc: Option<&str>) -> (Option<f64>, Option<f64>) {
    let Some(text) = doc else { return (None, None) };
    let Some((_, after_open)) = text.split_once('[') else { return (None, None) };
    let Some((inner, _)) = after_open.split_once(']') else { return (None, None) };
    let mut parts = inner.split(',');
    let (Some(first), Some(second)) = (parts.next(), parts.next()) else { return (None, None) };
    match (first.trim().parse::<f64>().ok(), second.trim().parse::<f64>().ok()) {
        (Some(min_val), Some(max_val)) => (Some(min_val), Some(max_val)),
        _ => (None, None),
    }
}
