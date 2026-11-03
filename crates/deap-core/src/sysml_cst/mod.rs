//! [DOC-TIER: A]
//!
//! ### 1. Architectural Purpose
//! Exposes the lossless SysML v2 and KerML Concrete Syntax Tree (CST) subsystem,
//! unifying the `logos` DFA lexer (`lexer`), `rowan` syntax kinds (`syntax_kind`),
//! lossless CST / construct parser (`parser`), and incremental Salsa-backed query
//! database (`salsa_db`).
//!
//! ### 2. Mathematical / Formal / Standard Formulation
//! For any UTF-8 SysML v2 specification string $S \in \Sigma^*$:
//! $$
//! \begin{aligned}
//! \text{unparse}(\text{parse\_sysml\_cst}(S).\text{green}) &= S
//! \end{aligned}
//! $$
//! And for any two inputs $S_1, S_2$ equivalent modulo whitespace trivia:
//! $$
//! \begin{aligned}
//! D_{\text{canon}}(S_1) &= D_{\text{canon}}(S_2)
//! \end{aligned}
//! $$
//!
//! ### 3. Step-by-Step Execution Flow
//! 1. Lex raw UTF-8 source text into byte-accurate `LexedSysmlToken` slices (`lexer`).
//! 2. Map token kinds to `SysmlSyntaxKind` (`syntax_kind`).
//! 3. Construct the lossless Rowan `GreenNode` and extract `SysmlConstructSummary` records (`parser`).
//! 4. Memoize CST, HIR constructs, and canonical SHA-256 digests in `SysmlCompilerDb` (`salsa_db`).
//!
//! ### 4. Key Data Structures & Invariants
//! - `ParsedSysmlCst`: Root Rowan green node, construct list, diagnostics, and roundtrip proof.
//! - `SysmlCompilerDb`: Deterministic incremental query cache keyed by `SourceProgram`.
//!
//! ### 5. Error Handling & Edge Cases
//! - Lexical anomalies and unbalanced braces emit `SysmlCstDiagnostic` records without panicking.
//!
//! ### 6. Performance & Complexity
//! - Time and space complexity: $\mathcal{O}(|S|)$ linear in byte length of the source buffer.
//!
//! ### 7. Safety & Concurrency
//! Pure safe Rust (`#![forbid(unsafe_code)]` compatible) across all submodules.

#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]

pub mod lexer;
pub mod parser;
pub mod salsa_db;
pub mod syntax_kind;

pub use lexer::{tokenize_sysml, LexedSysmlToken, SysmlToken};
pub use parser::{
    classify_keyword_kind, parse_sysml_cst, ParsedSysmlCst, SysmlConstructSummary,
    SysmlCstDiagnostic,
};
pub use salsa_db::{
    compute_canonical_digest, lower_sysml_hir, parse_sysml_green, SourceProgram, SysmlCompilerDb,
};
pub use syntax_kind::{
    token_to_syntax_kind, SysmlLanguage, SysmlSyntaxKind, SysmlSyntaxNode, SysmlSyntaxToken,
};

#[cfg(test)]
mod tests {
    use super::*;

    const NORMATIVE_SYSML_SNIPPET: &str = r"library package AerospaceCore {
    metadata def HazardMeta;
    metadata def RiskMeta;
    metadata def CapabilityMeta;
    part def ActuatorSubsys {
        port cmdPort : ~CommandPort [1..*] {ordered};
    }
    part primaryActuator :> ActuatorSubsys {
        port activePort :>> cmdPort ::> AerospaceCore::ActuatorSubsys::cmdPort;
    }
    port def CommandPort;
    action def ComputeControlLaw;
    action runControl : ComputeControlLaw;
    state def FlightModeState;
    state activeMode : FlightModeState;
    constraint def TorqueLimitConstraint;
    constraint torqueCheck : TorqueLimitConstraint;
    requirement def <'REQ-0301'> DeterminismReq {
        doc /* Deterministic CST roundtrip */
    }
    requirement <'REQ-0302'> runtimeReq : DeterminismReq;
    use case def StabilizeVehicle;
    use case stabilizeNominal;
    verification def VerifyDeterminism;
    verification verifyRun : VerifyDeterminism;
    #hazard part actuatorRunaway;
    #risk part elevatedRisk;
    #capability part faultTolerance;
    hazard def LegacyThermalHazard;
    variation part def RedundancyMode {
        variant part dualChannel;
    }
}
";

    #[test]
    fn test_lossless_roundtrip_and_normative_constructs() {
        let parsed = parse_sysml_cst(NORMATIVE_SYSML_SNIPPET);
        assert!(parsed.lossless_roundtrip_eq);
        assert!(parsed.diagnostics.is_empty());
        assert_eq!(parsed.syntax().text().to_string(), NORMATIVE_SYSML_SNIPPET);

        let kinds: Vec<SysmlSyntaxKind> = parsed.constructs.iter().map(|c| c.kind).collect();
        assert!(kinds.contains(&SysmlSyntaxKind::PackageDecl));
        assert!(kinds.contains(&SysmlSyntaxKind::MetadataDef));
        assert!(kinds.contains(&SysmlSyntaxKind::PartDef));
        assert!(kinds.contains(&SysmlSyntaxKind::PartUsage));
        assert!(kinds.contains(&SysmlSyntaxKind::PortDef));
        assert!(kinds.contains(&SysmlSyntaxKind::PortUsage));
        assert!(kinds.contains(&SysmlSyntaxKind::ActionDef));
        assert!(kinds.contains(&SysmlSyntaxKind::ActionUsage));
        assert!(kinds.contains(&SysmlSyntaxKind::StateDef));
        assert!(kinds.contains(&SysmlSyntaxKind::StateUsage));
        assert!(kinds.contains(&SysmlSyntaxKind::ConstraintDef));
        assert!(kinds.contains(&SysmlSyntaxKind::ConstraintUsage));
        assert!(kinds.contains(&SysmlSyntaxKind::RequirementDef));
        assert!(kinds.contains(&SysmlSyntaxKind::RequirementUsage));
        assert!(kinds.contains(&SysmlSyntaxKind::UseCaseDef));
        assert!(kinds.contains(&SysmlSyntaxKind::UseCaseUsage));
        assert!(kinds.contains(&SysmlSyntaxKind::VerificationDef));
        assert!(kinds.contains(&SysmlSyntaxKind::VerificationUsage));
        assert!(kinds.contains(&SysmlSyntaxKind::VariationDecl));
        assert!(kinds.contains(&SysmlSyntaxKind::VariantUsage));
    }

    #[test]
    fn test_salsa_db_incremental_digest_stability() {
        let mut db = SysmlCompilerDb::new();
        let prog_v1 = SourceProgram::new("schema/model.sysml", NORMATIVE_SYSML_SNIPPET);
        let digest_v1 = compute_canonical_digest(&mut db, &prog_v1);
        let digest_v1_repeat = compute_canonical_digest(&mut db, &prog_v1);
        assert_eq!(digest_v1, digest_v1_repeat);
        assert_eq!(db.parse_count(), 1);
        assert_eq!(digest_v1.len(), 64);

        let whitespace_edited = NORMATIVE_SYSML_SNIPPET.replace("    ", "        \n  ");
        let prog_v2 = prog_v1.with_text(whitespace_edited);
        let cst_v2 = parse_sysml_green(&mut db, &prog_v2);
        assert!(cst_v2.lossless_roundtrip_eq);
        let digest_v2 = compute_canonical_digest(&mut db, &prog_v2);
        assert_eq!(digest_v1, digest_v2);
        assert_eq!(db.parse_count(), 2);

        let hir = lower_sysml_hir(&mut db, &prog_v2);
        assert!(!hir.is_empty());
    }
}
