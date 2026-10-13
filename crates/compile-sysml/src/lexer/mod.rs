//! Lexer module for SysML v2 / KerML tokenization.

pub mod scanner;
pub mod token;

pub use scanner::Scanner;
pub use token::{Span, Token, TokenKind};
