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

/// Cryptographic fixed-point receipt proving Stage-1 and Stage-2 AST formatting idempotence.
/// Realises: [REQ-0701]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynCodegenFixedPointReceipt {
    pub stage1_sha256: String,
    pub stage2_sha256: String,
    pub is_idempotent: bool,
    pub formatted_source: String,
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
}
