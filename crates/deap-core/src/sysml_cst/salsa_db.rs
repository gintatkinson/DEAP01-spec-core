//! [DOC-TIER: A]
//!
//! ### 1. Architectural Purpose
//! Provides the Salsa-backed deterministic incremental query database (`SysmlCompilerDb`)
//! for SysML v2 and KerML Concrete Syntax Tree (CST) parsing, High-Level Intermediate
//! Representation (HIR) construct lowering, and canonical SHA-256 digest memoization.
//!
//! ### 2. Mathematical / Formal / Standard Formulation
//! Let $S_r$ be a source text at revision $r$ with token stream $T(S_r)$ and non-trivia
//! subsequence $T_{\neg\text{ws}}(S_r) = \langle t \in T(S_r) \mid \text{kind}(t) \neq \text{WHITESPACE} \rangle$:
//! $$
//! \begin{aligned}
//! D_{\text{canon}}(S_r) &= \text{SHA256}\!\left(\text{encode}(T_{\neg\text{ws}}(S_r)) \;\|\; \text{encode}(\text{HIR}(S_r))\right)
//! \end{aligned}
//! $$
//! By construction, if $S_{r+1}$ differs from $S_r$ solely by whitespace trivia, then
//! $D_{\text{canon}}(S_{r+1}) = D_{\text{canon}}(S_r)$ (early-cutoff stability).
//!
//! ### 3. Step-by-Step Execution Flow
//! 1. Accept a `SourceProgram` keyed by workspace-relative URI and UTF-8 text.
//! 2. Check the memoized revision table in `SysmlCompilerDb`; return cached artifacts if text matches.
//! 3. Otherwise increment the database revision counter, invoke `parse_sysml_cst`, and extract HIR constructs.
//! 4. Compute the canonical SHA-256 hex digest over non-whitespace CST tokens and HIR construct headers.
//! 5. Store and return the `Arc<ParsedSysmlCst>`, `Vec<SysmlConstructSummary>`, or canonical digest.
//!
//! ### 4. Key Data Structures & Invariants
//! - `SourceProgram`: Input key holding `(uri, text, revision)`.
//! - `SysmlCompilerDb`: Deterministic incremental query store tracking revisions and parse counts.
//!
//! ### 5. Error Handling & Edge Cases
//! - Empty inputs and inputs with lexical errors still produce deterministic CSTs and digests without panicking.
//!
//! ### 6. Performance & Complexity
//! - Cache hit: $\mathcal{O}(|S|)$ byte comparison and $\mathcal{O}(1)$ `Arc` clone.
//! - Cache miss: $\mathcal{O}(|S|)$ single-pass lexing, CST construction, and SHA-256 hashing.
//!
//! ### 7. Safety & Concurrency
//! Pure safe Rust (`#![forbid(unsafe_code)]` compatible) using `Arc` for shared immutable trees.

#![deny(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::doc_markdown, clippy::missing_const_for_fn)]

use std::collections::BTreeMap;
use std::sync::Arc;

use rowan::NodeOrToken;
use salsa as _;

use crate::digest::compute_sha256;
use super::parser::{parse_sysml_cst, ParsedSysmlCst, SysmlConstructSummary};
use super::syntax_kind::SysmlSyntaxKind;

/// Incremental source input handle for the SysML v2 query database. Realises: [REQ-0201]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceProgram {
    /// Canonical workspace-relative path or logical URI of the SysML v2 source unit.
    pub uri: String,
    /// Raw UTF-8 source text of the SysML v2 specification unit.
    pub text: String,
    /// Monotonic revision counter updated on input mutation.
    pub revision: u64,
}

impl SourceProgram {
    /// Creates a new `SourceProgram` input at revision `1`. Realises: [REQ-0201]
    #[must_use]
    pub fn new(uri: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            text: text.into(),
            revision: 1,
        }
    }

    /// Returns an updated `SourceProgram` with new source text and incremented revision. Realises: [REQ-0201]
    #[must_use]
    pub fn with_text(&self, text: impl Into<String>) -> Self {
        Self {
            uri: self.uri.clone(),
            text: text.into(),
            revision: self.revision.saturating_add(1),
        }
    }
}

/// Cached program entry in incremental database. Realises: [REQ-0201]
#[derive(Debug, Clone)]
struct CachedProgramEntry {
    text: String,
    cst: Arc<ParsedSysmlCst>,
    hir: Arc<Vec<SysmlConstructSummary>>,
    canonical_digest: String,
}

/// Deterministic incremental query database for SysML v2 CST and HIR compilation. Realises: [REQ-0201]
#[derive(Debug, Default, Clone)]
pub struct SysmlCompilerDb {
    entries: BTreeMap<String, CachedProgramEntry>,
    revision: u64,
    parse_count: usize,
}

impl SysmlCompilerDb {
    /// Creates an empty `SysmlCompilerDb` instance. Realises: [REQ-0201]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the current monotonic database revision counter. Realises: [REQ-0201]
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Returns the total number of uncached CST parse executions performed. Realises: [REQ-0201]
    #[must_use]
    pub fn parse_count(&self) -> usize {
        self.parse_count
    }

    /// Memoizes and returns the lossless Rowan CST for `program`. Realises: [REQ-0201]
    pub fn parse_sysml_green(&mut self, program: &SourceProgram) -> Arc<ParsedSysmlCst> {
        Arc::clone(&self.ensure_cached(program).cst)
    }

    /// Memoizes and returns the lowered HIR construct summaries for `program`. Realises: [REQ-0201]
    pub fn lower_sysml_hir(&mut self, program: &SourceProgram) -> Vec<SysmlConstructSummary> {
        (*self.ensure_cached(program).hir).clone()
    }

    /// Memoizes and returns the canonical whitespace-invariant SHA-256 hex digest for `program`. Realises: [REQ-0201]
    pub fn compute_canonical_digest(&mut self, program: &SourceProgram) -> String {
        self.ensure_cached(program).canonical_digest.clone()
    }

    /// Ensures cached entry is up to date. Realises: [REQ-0201]
    fn ensure_cached(&mut self, program: &SourceProgram) -> &CachedProgramEntry {
        let stale = self
            .entries
            .get(&program.uri)
            .is_some_and(|entry| entry.text != program.text);
        if stale {
            self.entries.remove(&program.uri);
        }

        let revision_ref = &mut self.revision;
        let parse_count_ref = &mut self.parse_count;
        self.entries.entry(program.uri.clone()).or_insert_with(|| {
            *revision_ref = revision_ref.saturating_add(1);
            *parse_count_ref = parse_count_ref.saturating_add(1);
            let parsed = parse_sysml_cst(&program.text);
            let canonical_digest = hash_canonical_cst(&parsed);
            let hir = Arc::new(parsed.constructs.clone());
            let cst = Arc::new(parsed);
            CachedProgramEntry {
                text: program.text.clone(),
                cst,
                hir,
                canonical_digest,
            }
        })
    }
}

/// Query function returning the memoized `Arc<ParsedSysmlCst>` for `program`. Realises: [REQ-0201]
pub fn parse_sysml_green(
    db: &mut SysmlCompilerDb,
    program: &SourceProgram,
) -> Arc<ParsedSysmlCst> {
    db.parse_sysml_green(program)
}

/// Query function returning the lowered `Vec<SysmlConstructSummary>` HIR for `program`. Realises: [REQ-0201]
pub fn lower_sysml_hir(
    db: &mut SysmlCompilerDb,
    program: &SourceProgram,
) -> Vec<SysmlConstructSummary> {
    db.lower_sysml_hir(program)
}

/// Query function returning the canonical whitespace-invariant SHA-256 hex digest for `program`. Realises: [REQ-0201]
pub fn compute_canonical_digest(db: &mut SysmlCompilerDb, program: &SourceProgram) -> String {
    db.compute_canonical_digest(program)
}

/// Hashes canonical CST tokens into SHA-256 digest. Realises: [REQ-0201]
fn hash_canonical_cst(parsed: &ParsedSysmlCst) -> String {
    let mut buffer = Vec::new();
    for element in parsed.syntax().descendants_with_tokens() {
        if let NodeOrToken::Token(token) = element {
            if token.kind() != SysmlSyntaxKind::WHITESPACE {
                buffer.extend_from_slice(&u16::from(token.kind()).to_le_bytes());
                buffer.extend_from_slice(token.text().as_bytes());
                buffer.push(0xff);
            }
        }
    }
    for construct in &parsed.constructs {
        buffer.extend_from_slice(&u16::from(construct.kind).to_le_bytes());
        buffer.extend_from_slice(construct.keyword_head.as_bytes());
        buffer.push(0xfe);
        if let Some(name) = &construct.name {
            buffer.extend_from_slice(name.as_bytes());
        }
        buffer.push(u8::from(construct.is_definition));
    }
    compute_sha256(&buffer)
}
