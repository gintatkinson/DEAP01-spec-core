//! Lexer module for SysML v2 / KerML tokenization.
//! [DOC-TIER: A]
//!
//! Safety & Traceability: Realises: [REQ-LEXER-SCANNER-MODULE], [REQ-LEXER-TOKEN-TOKEN]
//! Contract: Re-exports `Scanner`, `Span`, `Token`, and `TokenKind` backed by `deap_core::sysml_cst::lexer::SysmlToken`.
//! Bounds & Faults: Linear-time `O(N)` lexical scanning with total bounds-safe UTF-8 cursor primitives.

pub mod scanner;
pub mod token;

pub use scanner::Scanner;
pub use token::{Span, Token, TokenKind};

#[cfg(test)]
mod tests {
    use super::*;

    fn kind_at(tokens: &[Token], idx: usize) -> Result<TokenKind, String> {
        tokens
            .get(idx)
            .map(|t| t.kind.clone())
            .ok_or_else(|| format!("missing token at index {idx}"))
    }

    #[test]
    fn test_scan_keywords_and_symbols() -> Result<(), Box<dyn std::error::Error>> {
        let src = "package Foo { part def Bar :> Base { port in p : RS485; } }";
        let mut scanner = Scanner::new(src);
        let tokens = scanner.scan_all()?;

        assert_eq!(kind_at(&tokens, 0)?, TokenKind::Package);
        assert_eq!(kind_at(&tokens, 1)?, TokenKind::Ident("Foo".to_string()));
        assert_eq!(kind_at(&tokens, 2)?, TokenKind::OpenBrace);
        assert_eq!(kind_at(&tokens, 3)?, TokenKind::Part);
        assert_eq!(kind_at(&tokens, 4)?, TokenKind::Def);
        assert_eq!(kind_at(&tokens, 5)?, TokenKind::Ident("Bar".to_string()));
        assert_eq!(kind_at(&tokens, 6)?, TokenKind::ColonGt);
        assert_eq!(kind_at(&tokens, 7)?, TokenKind::Ident("Base".to_string()));
        Ok(())
    }

    #[test]
    fn test_scan_doc_comment() -> Result<(), Box<dyn std::error::Error>> {
        let src = "doc /* SSOT for Autonomous Vehicle */ part def FCC;";
        let mut scanner = Scanner::new(src);
        let tokens = scanner.scan_all()?;

        assert_eq!(
            kind_at(&tokens, 0)?,
            TokenKind::DocComment("SSOT for Autonomous Vehicle".to_string())
        );
        assert_eq!(kind_at(&tokens, 1)?, TokenKind::Part);
        assert_eq!(kind_at(&tokens, 2)?, TokenKind::Def);
        assert_eq!(kind_at(&tokens, 3)?, TokenKind::Ident("FCC".to_string()));
        assert_eq!(kind_at(&tokens, 4)?, TokenKind::Semi);
        Ok(())
    }

    #[test]
    fn test_scan_numbers_and_operators() -> Result<(), Box<dyn std::error::Error>> {
        let src = "v <= 60.0; count := 42; is_valid == true;";
        let mut scanner = Scanner::new(src);
        let tokens = scanner.scan_all()?;

        assert_eq!(kind_at(&tokens, 0)?, TokenKind::Ident("v".to_string()));
        assert_eq!(kind_at(&tokens, 1)?, TokenKind::LtEq);
        assert_eq!(kind_at(&tokens, 2)?, TokenKind::RealLit(60.0));
        assert_eq!(kind_at(&tokens, 3)?, TokenKind::Semi);

        assert_eq!(kind_at(&tokens, 4)?, TokenKind::Ident("count".to_string()));
        assert_eq!(kind_at(&tokens, 5)?, TokenKind::ColonEq);
        assert_eq!(kind_at(&tokens, 6)?, TokenKind::IntLit(42));
        assert_eq!(kind_at(&tokens, 7)?, TokenKind::Semi);

        assert_eq!(
            kind_at(&tokens, 8)?,
            TokenKind::Ident("is_valid".to_string())
        );
        assert_eq!(kind_at(&tokens, 9)?, TokenKind::EqEq);
        assert_eq!(kind_at(&tokens, 10)?, TokenKind::BoolLit(true));
        assert_eq!(kind_at(&tokens, 11)?, TokenKind::Semi);
        Ok(())
    }

    #[test]
    fn test_lexer_tokenizes_import_statement() -> Result<(), Box<dyn std::error::Error>> {
        let src = "import Foo::*; import Bar::Baz; import Sys::**;";
        let mut scanner = Scanner::new(src);
        let tokens = scanner.scan_all()?;

        assert_eq!(kind_at(&tokens, 0)?, TokenKind::Import);
        assert_eq!(kind_at(&tokens, 1)?, TokenKind::Ident("Foo".to_string()));
        assert_eq!(kind_at(&tokens, 2)?, TokenKind::DoubleColon);
        assert_eq!(kind_at(&tokens, 3)?, TokenKind::Star);
        assert_eq!(kind_at(&tokens, 4)?, TokenKind::Semi);

        assert_eq!(kind_at(&tokens, 5)?, TokenKind::Import);
        assert_eq!(kind_at(&tokens, 6)?, TokenKind::Ident("Bar".to_string()));
        assert_eq!(kind_at(&tokens, 7)?, TokenKind::DoubleColon);
        assert_eq!(kind_at(&tokens, 8)?, TokenKind::Ident("Baz".to_string()));
        assert_eq!(kind_at(&tokens, 9)?, TokenKind::Semi);

        assert_eq!(kind_at(&tokens, 10)?, TokenKind::Import);
        assert_eq!(kind_at(&tokens, 11)?, TokenKind::Ident("Sys".to_string()));
        assert_eq!(kind_at(&tokens, 12)?, TokenKind::DoubleColon);
        assert_eq!(kind_at(&tokens, 13)?, TokenKind::Star);
        assert_eq!(kind_at(&tokens, 14)?, TokenKind::Star);
        assert_eq!(kind_at(&tokens, 15)?, TokenKind::Semi);
        Ok(())
    }

    #[test]
    fn test_lexer_nested_block_comments() -> Result<(), Box<dyn std::error::Error>> {
        let src = "/* outer /* nested */ still outer */ part def FCC;";
        let mut scanner = Scanner::new(src);
        let tokens = scanner.scan_all()?;

        assert_eq!(
            kind_at(&tokens, 0)?,
            TokenKind::DocComment("outer /* nested */ still outer".to_string())
        );
        assert_eq!(kind_at(&tokens, 1)?, TokenKind::Part);
        assert_eq!(kind_at(&tokens, 2)?, TokenKind::Def);
        assert_eq!(kind_at(&tokens, 3)?, TokenKind::Ident("FCC".to_string()));
        assert_eq!(kind_at(&tokens, 4)?, TokenKind::Semi);
        Ok(())
    }

    #[test]
    fn test_lexer_unrestricted_single_quoted_identifier() -> Result<(), Box<dyn std::error::Error>>
    {
        let src = "part def 'Flight Control Computer' :> 'Avionics System';";
        let mut scanner = Scanner::new(src);
        let tokens = scanner.scan_all()?;

        assert_eq!(kind_at(&tokens, 0)?, TokenKind::Part);
        assert_eq!(kind_at(&tokens, 1)?, TokenKind::Def);
        assert_eq!(
            kind_at(&tokens, 2)?,
            TokenKind::Ident("Flight Control Computer".to_string())
        );
        assert_eq!(kind_at(&tokens, 3)?, TokenKind::ColonGt);
        assert_eq!(
            kind_at(&tokens, 4)?,
            TokenKind::Ident("Avionics System".to_string())
        );
        assert_eq!(kind_at(&tokens, 5)?, TokenKind::Semi);
        Ok(())
    }

    #[test]
    fn test_scanner_utf8_boundaries_and_eof_advance_safety(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let src = concat!(
            "/* doc: Δv <= 60.0 m/s 🛰 */\n",
            "// 航空安全コメント Ω\n",
            "part def 'センサー_01' {\n",
            "    doc /* Ω温度 >= -40.0 */\n",
            "    use case '軌道制御_UC' { attribute msg = \"ΔΩ🛰\"; }\n",
            "}"
        );
        let mut scanner = Scanner::new(src);
        let tokens = scanner.scan_all()?;
        assert_eq!(
            kind_at(&tokens, 0)?,
            TokenKind::DocComment("Δv <= 60.0 m/s 🛰".to_string())
        );
        assert_eq!(kind_at(&tokens, 1)?, TokenKind::Part);
        assert_eq!(kind_at(&tokens, 2)?, TokenKind::Def);
        assert_eq!(
            kind_at(&tokens, 3)?,
            TokenKind::Ident("センサー_01".to_string())
        );

        assert_eq!(scanner.advance(), '\0');
        assert_eq!(scanner.peek(), '\0');
        assert!(!scanner.starts_with("/*"));
        assert_eq!(scanner.next_token()?.kind, TokenKind::Eof);

        let mut mid_utf8 = Scanner {
            source: "/* Δ */",
            cst_tokens: Vec::new(),
            cst_idx: 0,
            cursor: 4,
            line: 1,
            col: 1,
        };
        assert_eq!(mid_utf8.peek(), '\0');
        assert!(!mid_utf8.starts_with("*/"));
        assert_eq!(mid_utf8.advance(), '\0');
        Ok(())
    }
}
