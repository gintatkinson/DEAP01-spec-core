#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Deterministic `syn` + `quote` + `prettyplease` AST code generation and fixed-point verifier. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D tool qualification standards.
//! Provides typed Rust token synthesis (`proc_macro2`, `quote`, `syn`) and two-stage
//! `prettyplease` unparse idempotence verification backed by `ra_ap_syntax` CST validation.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Stage-1 and Stage-2 formatted AST projections satisfy `SHA-256(Stage 1) == SHA-256(Stage 2)`.
//! - Every emitted Rust module contains zero `ra_ap_syntax` CST parse errors and uses LF newlines.
//!
//! ## 3. Preconditions
//! - Callers supply valid Rust identifiers, type expressions, or raw UTF-8 Rust module source text.
//!
//! ## 4. Postconditions
//! - Returns a verified `SynCodegenFixedPointReceipt` proving fixed-point formatting idempotence.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ linear in AST token count and source byte length $N$.
//! - Space Complexity: $O(N)$ bounded by in-memory `syn::File` and CST syntax nodes.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Malformed Rust syntax, invalid identifiers, or non-idempotent AST unparse results fail closed.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0701], [BP-07]

use deap_core::digest::compute_sha256;
use deap_core::rust_cst::{normalize_lf, parse::parse_rust_source};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::collections::{BTreeMap, BTreeSet};

/// Cryptographic fixed-point receipt proving Stage-1 and Stage-2 AST formatting idempotence.
/// Realises: [REQ-0701]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynCodegenFixedPointReceipt {
    pub stage1_sha256: String,
    pub stage2_sha256: String,
    pub is_idempotent: bool,
    pub formatted_source: String,
}

/// Cryptographic manifest tracking generated compilation units and stage digests for fixed-point verification.
/// Realises: [REQ-0607], [REQ-0608], [REQ-0701], [BP-07], [BP-08]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StageTreeDigestManifest {
    pub file_digests: BTreeMap<String, String>,
    pub aggregate_sha256: String,
}

impl StageTreeDigestManifest {
    /// Constructs a new manifest computing aggregate SHA-256 over canonical sorted relative paths.
    /// Realises: [REQ-0607], [BP-07]
    #[must_use]
    pub fn new(file_digests: BTreeMap<String, String>) -> Self {
        let mut hasher_input = String::new();
        for (path, digest) in &file_digests {
            hasher_input.push_str(path);
            hasher_input.push(':');
            hasher_input.push_str(digest);
            hasher_input.push('\n');
        }
        let aggregate_sha256 = compute_sha256(hasher_input.as_bytes());
        Self {
            file_digests,
            aggregate_sha256,
        }
    }

    /// Constructs a manifest from slice of `(relative_path, content_or_digest)` pairs.
    /// Realises: [REQ-0607], [BP-07]
    #[must_use]
    pub fn from_entries(entries: &[(&str, &str)]) -> Self {
        let mut map = BTreeMap::new();
        for &(path, digest) in entries {
            map.insert(path.to_string(), digest.to_string());
        }
        Self::new(map)
    }

    /// Verifies fixed-point equivalence against another stage manifest snapshot.
    /// # Errors
    /// Returns an error if any file count or digest drifts between stage snapshots.
    /// Realises: [REQ-0607], [BP-07]
    pub fn verify_fixed_point(&self, other: &Self) -> Result<(), String> {
        if self.file_digests.len() != other.file_digests.len() {
            return Err(format!(
                "DEAP-E0705: Stage fixed-point file count drift: stage1={} vs stage2={}",
                self.file_digests.len(),
                other.file_digests.len()
            ));
        }
        for (path, s1_hash) in &self.file_digests {
            match other.file_digests.get(path) {
                Some(s2_hash) if s1_hash == s2_hash => {}
                Some(s2_hash) => {
                    return Err(format!(
                        "DEAP-E0705: Stage fixed-point hash mismatch on '{path}': stage1={s1_hash} != stage2={s2_hash}"
                    ));
                }
                None => {
                    return Err(format!(
                        "DEAP-E0705: Stage fixed-point missing file in stage2: '{path}'"
                    ));
                }
            }
        }
        if self.aggregate_sha256 != other.aggregate_sha256 {
            return Err(format!(
                "DEAP-E0705: Stage fixed-point aggregate hash drift: stage1={} != stage2={}",
                self.aggregate_sha256, other.aggregate_sha256
            ));
        }
        Ok(())
    }
}

/// Synthesizes a Rust compilation unit AST from `TokenStream` tokens and verifies fixed-point idempotence.
/// # Errors
/// Returns an error if tokens fail `syn` AST parsing, `ra_ap_syntax` CST errors occur, or Stage-1 != Stage-2.
/// Realises: [REQ-0607], [REQ-0701], [BP-07]
pub fn synthesize_prettyplease_unit(
    module_name: &str,
    req_id: &str,
    tokens: TokenStream,
) -> Result<SynCodegenFixedPointReceipt, String> {
    let _ = parse_safe_ident(module_name)?;
    let parsed_file = syn::parse2::<syn::File>(tokens)
        .map_err(|err| format!("syn::parse2 failed for unit '{module_name}' [{req_id}]: {err}"))?;
    let unparsed = prettyplease::unparse(&parsed_file);
    format_and_verify_rust_module_ast(&unparsed)
}

/// Compares two `StageTreeDigestManifest` snapshots, verifying Stage-1 == Stage-2 SHA-256 parity for all files.
/// # Errors
/// Returns `Err` with `DEAP-E0705` if file count, individual file hashes, or aggregate SHA-256 diverge.
/// Realises: [REQ-0607], [REQ-0608], [BP-07], [BP-08]
pub fn verify_stage_fixed_point(
    stage1: &StageTreeDigestManifest,
    stage2: &StageTreeDigestManifest,
    stage_label: &str,
) -> Result<bool, String> {
    if stage1.aggregate_sha256.is_empty() || stage2.aggregate_sha256.is_empty() {
        return Err(format!(
            "DEAP-E0705: empty fixed-point digest in stage '{stage_label}'"
        ));
    }
    let all_paths: BTreeSet<&String> = stage1
        .file_digests
        .keys()
        .chain(stage2.file_digests.keys())
        .collect();
    let mut mismatch_count = 0usize;
    let mut first_mismatch = String::new();

    for path in all_paths {
        let h1 = stage1.file_digests.get(path);
        let h2 = stage2.file_digests.get(path);
        if h1 != h2 {
            mismatch_count += 1;
            if first_mismatch.is_empty() {
                first_mismatch = format!("mismatch in '{path}': stage1={h1:?} != stage2={h2:?}");
            }
        }
    }

    if mismatch_count > 0 || stage1.aggregate_sha256 != stage2.aggregate_sha256 {
        return Err(format!(
            "DEAP-E0705: Stage fixed-point drift in '{stage_label}': {mismatch_count} mismatched files ({first_mismatch})"
        ));
    }

    Ok(true)
}

/// Parses, formats via `prettyplease`, and verifies Stage-1 == Stage-2 SHA-256 and CST validity.
/// # Errors
/// Returns an error if `syn` parsing fails, `ra_ap_syntax` reports CST errors, or hashes diverge.
/// Realises: [REQ-0701]
pub fn format_and_verify_rust_module_ast(
    raw_rust_source: &str,
) -> Result<SynCodegenFixedPointReceipt, String> {
    let normalized_input = normalize_lf(raw_rust_source);
    let ast1 = syn::parse_file(&normalized_input)
        .map_err(| err| format!("Stage-1 syn::parse_file failed: {err}"))?;
    let stage1 = normalize_lf(&prettyplease::unparse(&ast1));

    let ast2 = syn::parse_file(&stage1)
        .map_err(|err| format!("Stage-2 syn::parse_file failed: {err}"))?;
    let stage2 = normalize_lf(&prettyplease::unparse(&ast2));

    let cst_errors = parse_rust_source("<codegen>", &stage1).syntax_errors();
    if !cst_errors.is_empty() {
        return Err(format!(
            "ra_ap_syntax CST validation failed on formatted output: {}",
            cst_errors.join("; ")
        ));
    }

    let stage1_sha256 = compute_sha256(stage1.as_bytes());
    let stage2_sha256 = compute_sha256(stage2.as_bytes());
    let is_idempotent = stage1_sha256 == stage2_sha256;
    if !is_idempotent {
        return Err(format!(
            "Non-idempotent AST unparse: stage1={stage1_sha256} != stage2={stage2_sha256}"
        ));
    }

    Ok(SynCodegenFixedPointReceipt {
        stage1_sha256,
        stage2_sha256,
        is_idempotent,
        formatted_source: stage1,
    })
}

/// Validates and constructs a `proc_macro2::Ident` at `Span::call_site()` without panicking.
/// # Errors
/// Returns an error when `raw_ident` is not a valid Rust identifier token.
/// Realises: [REQ-0701]
fn parse_safe_ident(raw_ident: &str) -> Result<Ident, String> {
    syn::parse_str::<syn::Ident>(raw_ident)
        .map_err(|err| format!("Invalid Rust identifier '{raw_ident}': {err}"))?;
    Ok(Ident::new(raw_ident, Span::call_site()))
}

/// Synthesizes a typed domain `pub struct` via `quote!` and verifies AST fixed-point idempotence.
/// # Errors
/// Returns an error if any identifier or field type is invalid or AST verification fails.
/// Realises: [REQ-0701]
pub fn synthesize_typed_domain_struct_tokens(
    struct_ident: &str,
    req_id: &str,
    fields: &[(&str, &str)],
) -> Result<SynCodegenFixedPointReceipt, String> {
    let ident = parse_safe_ident(struct_ident)?;
    let struct_doc = format!(" Generated domain struct `{struct_ident}`.");
    let req_doc = format!(" Realises: [{req_id}]");
    let mut field_tokens: Vec<TokenStream> = Vec::with_capacity(fields.len());
    for &(field_name, field_ty_str) in fields {
        let field_ident = parse_safe_ident(field_name)?;
        let field_ty = syn::parse_str::<syn::Type>(field_ty_str)
            .map_err(|err| format!("Invalid field type '{field_ty_str}' for '{field_name}': {err}"))?;
        let field_doc = format!(" Field `{field_name}`. Realises: [{req_id}]");
        field_tokens.push(quote! {
            #[doc = #field_doc]
            pub #field_ident: #field_ty
        });
    }

    let tokens = quote! {
        #[doc = #struct_doc]
        #[doc = #req_doc]
        #[derive(Debug, Clone, PartialEq)]
        pub struct #ident {
            #(#field_tokens,)*
        }
    };
    let parsed_file = syn::parse2::<syn::File>(tokens)
        .map_err(|err| format!("syn::parse2::<syn::File> failed for '{struct_ident}': {err}"))?;
    let unparsed = prettyplease::unparse(&parsed_file);
    format_and_verify_rust_module_ast(&unparsed)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// Verifies `synthesize_typed_domain_struct_tokens` achieves Stage-1 == Stage-2 idempotence.
    /// Realises: [REQ-0701]
    #[test]
    fn test_synthesize_typed_domain_struct_tokens_idempotent_and_cst_clean() -> Result<(), String> {
        let receipt = synthesize_typed_domain_struct_tokens(
            "TelemetrySample",
            "REQ-0701",
            &[
                ("sequence_id", "u64"),
                ("attitude_rad", "f64"),
                ("channel_label", "String"),
            ],
        )?;
        assert!(receipt.is_idempotent);
        assert_eq!(receipt.stage1_sha256, receipt.stage2_sha256);
        assert!(receipt.formatted_source.contains("pub struct TelemetrySample"));
        assert!(receipt.formatted_source.contains("pub sequence_id: u64"));
        assert!(!receipt.formatted_source.contains('\r'));
        let cst = parse_rust_source("<test>", &receipt.formatted_source);
        assert!(cst.syntax_errors().is_empty());
        Ok(())
    }

    /// Verifies `format_and_verify_rust_module_ast` normalizes CRLF and rejects invalid syntax.
    /// Realises: [REQ-0701]
    #[test]
    fn test_format_and_verify_rust_module_ast_crlf_and_malformed_rejection() -> Result<(), String> {
        let crlf_src = "pub struct Frame {\r\n    pub id: u32,\r\n}\r\n";
        let receipt = format_and_verify_rust_module_ast(crlf_src)?;
        assert!(receipt.is_idempotent);
        assert_eq!(receipt.stage1_sha256, receipt.stage2_sha256);
        assert!(!receipt.formatted_source.contains("\r\n"));

        let malformed = "pub struct Broken { pub count: ";
        let err_res = format_and_verify_rust_module_ast(malformed);
        assert!(err_res.is_err());

        let bad_ident_res =
            synthesize_typed_domain_struct_tokens("123Invalid", "REQ-0701", &[("ok", "u32")]);
        assert!(bad_ident_res.is_err());
        Ok(())
    }

    /// Verifies `StageTreeDigestManifest` and `verify_stage_fixed_point` succeed on matching trees.
    /// Realises: [REQ-0607], [REQ-0608], [BP-07]
    #[test]
    fn test_stage_tree_digest_manifest_and_verify_stage_fixed_point() -> Result<(), String> {
        let manifest1 = StageTreeDigestManifest::from_entries(&[
            ("model.rs", "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"),
            ("lib.rs", "ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb"),
        ]);
        let manifest2 = manifest1.clone();
        assert!(manifest1.verify_fixed_point(&manifest2).is_ok());
        let verified = verify_stage_fixed_point(&manifest1, &manifest2, "test_stage")?;
        assert!(verified);
        Ok(())
    }

    /// Verifies `verify_stage_fixed_point` fails closed with DEAP-E0705 on drift.
    /// Realises: [REQ-0607], [REQ-0608], [BP-07]
    #[test]
    fn test_verify_stage_fixed_point_drift_fails_closed() {
        let manifest1 = StageTreeDigestManifest::from_entries(&[
            ("model.rs", "hash_a"),
        ]);
        let manifest2 = StageTreeDigestManifest::from_entries(&[
            ("model.rs", "hash_b"),
        ]);
        let res = verify_stage_fixed_point(&manifest1, &manifest2, "drift_stage");
        assert!(res.is_err());
        let err_msg = res.unwrap_err();
        assert!(err_msg.contains("DEAP-E0705"));
        assert!(err_msg.contains("drift_stage"));
    }

    /// Verifies `synthesize_prettyplease_unit` generates clean, idempotent Rust AST.
    /// Realises: [REQ-0607], [REQ-0701], [BP-07]
    #[test]
    fn test_synthesize_prettyplease_unit_idempotent() -> Result<(), String> {
        let tokens = quote! {
            pub struct EngineState {
                pub rpm: u32,
            }
        };
        let receipt = synthesize_prettyplease_unit("engine_state", "REQ-0607", tokens)?;
        assert!(receipt.is_idempotent);
        assert!(receipt.formatted_source.contains("pub struct EngineState"));
        Ok(())
    }
}
