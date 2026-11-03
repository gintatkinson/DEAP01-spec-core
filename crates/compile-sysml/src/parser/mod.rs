//! Parser module for SysML v2 / KerML syntax.
//! [DOC-TIER: A]
//!
//! Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-MODULE], [REQ-PARSER-EXPRESSIONS-EXPR]
//! Contract: Re-exports `SysmlParser` and mounts the `expressions` and `grammar` parser subsystems.
//! Bounds & Faults: All parsing operations are bounded by recursion depth 64 (`MAX_PARSE_DEPTH` / `MAX_EXPR_DEPTH`).

pub mod expressions;
pub mod grammar;

pub use grammar::SysmlParser;

#[cfg(test)]
mod tests {
    use super::expressions::*;
    use crate::lexer::scanner::Scanner;

    #[test]
    fn test_parse_inequality_constraint() -> Result<(), Box<dyn std::error::Error>> {
        let tokens = Scanner::new("v <= 60.0").scan_all()?;
        let expr = parse_expression(&tokens)?;
        assert_eq!(expr.to_string_repr(), "v <= 60.0");
        Ok(())
    }

    #[test]
    fn test_parse_nested_logical_expression() -> Result<(), Box<dyn std::error::Error>> {
        let tokens = Scanner::new("alt >= 10.0 && alt <= 120.0").scan_all()?;
        let expr = parse_expression(&tokens)?;
        assert_eq!(expr.to_string_repr(), "alt >= 10.0 and alt <= 120.0");
        Ok(())
    }

    #[test]
    fn test_parse_member_and_function_call() -> Result<(), Box<dyn std::error::Error>> {
        let tokens = Scanner::new("abs(fcc.speed) <= limits.max").scan_all()?;
        let expr = parse_expression(&tokens)?;
        assert_eq!(expr.to_string_repr(), "abs(fcc.speed) <= limits.max");
        Ok(())
    }

    #[test]
    fn test_parse_parenthesized_depth_bounds() -> Result<(), Box<dyn std::error::Error>> {
        let max_valid = MAX_EXPR_DEPTH - 1;
        let valid_src = format!("{}x{}", "(".repeat(max_valid), ")".repeat(max_valid));
        assert_eq!(
            parse_expression(&Scanner::new(&valid_src).scan_all()?)?.to_string_repr(),
            "x"
        );
        let exceeded = format!(
            "{}x{}",
            "(".repeat(MAX_EXPR_DEPTH),
            ")".repeat(MAX_EXPR_DEPTH)
        );
        assert!(parse_expression(&Scanner::new(&exceeded).scan_all()?).is_err());
        Ok(())
    }

    #[test]
    fn test_parse_unary_call_binary_and_member_depth_bounds(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let max_ops = MAX_EXPR_DEPTH - 1;
        let valid_unary = format!("{}x", "-".repeat(max_ops));
        assert!(parse_expression(&Scanner::new(&valid_unary).scan_all()?).is_ok());
        let deep_unary = format!("{}x", "-".repeat(MAX_EXPR_DEPTH));
        assert!(parse_expression(&Scanner::new(&deep_unary).scan_all()?).is_err());
        let valid_calls = format!("{}x{}", "f(".repeat(max_ops), ")".repeat(max_ops));
        assert!(parse_expression(&Scanner::new(&valid_calls).scan_all()?).is_ok());
        let deep_calls = format!(
            "{}x{}",
            "f(".repeat(MAX_EXPR_DEPTH),
            ")".repeat(MAX_EXPR_DEPTH)
        );
        assert!(parse_expression(&Scanner::new(&deep_calls).scan_all()?).is_err());
        let valid_pow = std::iter::repeat_n("x", MAX_EXPR_DEPTH)
            .collect::<Vec<_>>()
            .join(" ^ ");
        assert!(parse_expression(&Scanner::new(&valid_pow).scan_all()?).is_ok());
        let deep_pow = std::iter::repeat_n("x", MAX_EXPR_DEPTH + 1)
            .collect::<Vec<_>>()
            .join(" ^ ");
        assert!(parse_expression(&Scanner::new(&deep_pow).scan_all()?).is_err());
        let valid_member = format!("x{}", ".a".repeat(max_ops));
        assert!(parse_expression(&Scanner::new(&valid_member).scan_all()?).is_ok());
        let deep_member = format!("x{}", ".a".repeat(MAX_EXPR_DEPTH));
        assert!(parse_expression(&Scanner::new(&deep_member).scan_all()?).is_err());
        Ok(())
    }

    #[test]
    fn test_to_string_repr_depth_bounds() {
        let mut at_bound = Expr::Ident("x".to_string());
        for _ in 0..(MAX_EXPR_DEPTH - 1) {
            at_bound = Expr::Unary {
                op: UnaryOp::Neg,
                expr: Box::new(at_bound),
            };
        }
        assert_eq!(
            at_bound.to_string_repr(),
            format!("{}x", "-".repeat(MAX_EXPR_DEPTH - 1))
        );
        let mut beyond_bound = Expr::Ident("x".to_string());
        for _ in 0..(MAX_EXPR_DEPTH + 8) {
            beyond_bound = Expr::Unary {
                op: UnaryOp::Neg,
                expr: Box::new(beyond_bound),
            };
        }
        assert_eq!(
            beyond_bound.to_string_repr(),
            format!("{}...", "-".repeat(MAX_EXPR_DEPTH))
        );
    }
}
