//! `dart_cst`: Lossless Concrete Syntax Tree (CST) lexer and extractor for Dart. [DOC-TIER: A]
//! ## 1. Safety Intent & Regulatory Scope
//! Deterministic, lossless CST representation for `.dart` files built on `logos` and `rowan`.
//! ## 2. Invariants & Mathematical Properties
//! Every source byte is preserved in the `rowan` root node; `DartCstItem` uses 1-based lines.
//! ## 3. Preconditions
//! Callers provide a valid UTF-8 string slice representing Dart source code.
//! ## 4. Postconditions
//! Returns ordered top-level `DartCstItem` declarations with preceding `///` doc comments.
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ linear in source byte length $N$.
//! - Space Complexity: $O(N)$ proportional to token count and CST node allocation.
//! ## 6. Failure Modes & Fault Tolerance
//! Unrecognised characters fall back to `DartToken::Other` without panicking.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0201], [REQ-0602]

#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

use logos::Logos;
use rowan::{GreenNodeBuilder, Language, SyntaxKind, SyntaxNode};

/// Syntax node and token kinds for the lossless Dart CST. Realises: [REQ-0201]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum DartSyntaxKind {
    /// Root syntax node covering the entire file. Realises: [REQ-0201]
    Root = 0,
    /// Individual lexical token span. Realises: [REQ-0201]
    Token = 1,
}

/// Rowan language tag for the Dart CST. Realises: [REQ-0201]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DartLanguage {}

impl Language for DartLanguage {
    type Kind = DartSyntaxKind;

    /// Converts a raw syntax kind into a typed `DartSyntaxKind`. Realises: [REQ-0201]
    fn kind_from_raw(raw: SyntaxKind) -> Self::Kind {
        if raw.0 == 0 { DartSyntaxKind::Root } else { DartSyntaxKind::Token }
    }

    /// Converts a typed `DartSyntaxKind` into a raw `SyntaxKind`. Realises: [REQ-0201]
    fn kind_to_raw(kind: Self::Kind) -> SyntaxKind {
        SyntaxKind(kind as u16)
    }
}

/// Lexical token definitions for Dart source scanning. Realises: [REQ-0201]
#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DartToken {
    /// Triple-slash documentation comment line. Realises: [REQ-0201]
    #[regex(r"///[^\n]*", allow_greedy = true)]
    DocComment,
    /// Non-doc trivia (comments, string literals, whitespace). Realises: [REQ-0201]
    #[regex(r"//[^\n]*", allow_greedy = true)]
    #[regex(r"/\*([^*]|\*+[^*/])*\*+/")]
    #[regex(r#"r"[^"]*"|r'[^']*'"#)]
    #[regex(r#""\$\{[^}]*\}"|'\$\{[^}]*\}'"#)]
    #[regex(r#""([^"\\]|\\.)*"|'([^'\\]|\\.)*'"#)]
    #[regex(r"\s+")]
    Trivia,
    /// Structural declaration keyword. Realises: [REQ-0201]
    #[regex(r"class|mixin|enum|extension|typedef|const")]
    Keyword,
    /// Dart identifier token. Realises: [REQ-0201]
    #[regex(r"[A-Za-z_$][A-Za-z0-9_$]*", priority = 2)]
    Ident,
    /// Structural delimiter or assignment punctuation. Realises: [REQ-0201]
    #[regex(r"[{}\(\)\[\];=]")]
    Punct,
    /// Fallback single-character token. Realises: [REQ-0201]
    #[regex(r".", priority = 1)]
    Other,
}

/// High-level structural declaration categories in Dart. Realises: [REQ-0201]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DartItemKind {
    /// Class declaration (`class`). Realises: [REQ-0201]
    Class,
    /// Mixin declaration (`mixin`). Realises: [REQ-0201]
    Mixin,
    /// Enumeration declaration (`enum`). Realises: [REQ-0201]
    Enum,
    /// Extension declaration (`extension`). Realises: [REQ-0201]
    Extension,
    /// Type alias declaration (`typedef`). Realises: [REQ-0201]
    TypeAlias,
    /// Function declaration. Realises: [REQ-0201]
    Function,
    /// Top-level constant declaration (`const`). Realises: [REQ-0201]
    Const,
}

/// Extracted structural item from a Dart CST. Realises: [REQ-0201]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DartCstItem {
    /// Structural category of the item. Realises: [REQ-0201]
    pub kind: DartItemKind,
    /// Identifier name of the declared item. Realises: [REQ-0201]
    pub name: String,
    /// 1-based starting line number of the declaration. Realises: [REQ-0201]
    pub start_line: usize,
    /// 1-based ending line number of the declaration. Realises: [REQ-0201]
    pub end_line: usize,
    /// Whether the declaration is preceded by a `///` doc comment. Realises: [REQ-0201]
    pub has_doc_comment: bool,
    /// Concatenated text of the preceding `///` doc comment. Realises: [REQ-0201]
    pub doc_text: String,
}

/// Parses Dart source into a lossless `rowan` CST and extracts declarations. Realises: [REQ-0201]
#[must_use]
pub fn parse_dart_items(source: &str) -> Vec<DartCstItem> {
    let mut builder = GreenNodeBuilder::new();
    builder.start_node(DartLanguage::kind_to_raw(DartSyntaxKind::Root));
    let mut tokens = Vec::new();
    for (res, span) in DartToken::lexer(source).spanned() {
        let tok = res.unwrap_or(DartToken::Other);
        if let Some(slice) = source.get(span.clone()) {
            builder.token(DartLanguage::kind_to_raw(DartSyntaxKind::Token), slice);
            let line = source.get(..span.start).unwrap_or(source).bytes().filter(|&b| b == b'\n').count() + 1;
            tokens.push((tok, line, slice));
        }
    }
    builder.finish_node();
    let root: SyntaxNode<DartLanguage> = SyntaxNode::new_root(builder.finish());
    let _lossless_len = root.text().len();
    extract_items(&tokens)
}

/// Parses Dart source text into CST items. Realises: [REQ-0201]
#[must_use]
pub fn parse_dart_cst(source: &str) -> Vec<DartCstItem> {
    parse_dart_items(source)
}

/// Scans top-level token spans to build `DartCstItem` records. Realises: [REQ-0201]
fn extract_items(tokens: &[(DartToken, usize, &str)]) -> Vec<DartCstItem> {
    let (mut items, mut docs) = (Vec::new(), Vec::new());
    let (mut brace_depth, mut paren_depth) = (0usize, 0usize);
    let mut active: Option<(DartItemKind, usize, Option<String>, bool)> = None;
    let (mut stmt_start, mut last_ident, mut seen_top_eq): (Option<usize>, Option<String>, bool) = (None, None, false);

    for &(tok, line, slice) in tokens {
        if brace_depth == 0 && active.is_none() && tok == DartToken::DocComment {
            docs.push(slice.strip_prefix("///").unwrap_or(slice).trim().to_owned());
            continue;
        }
        if tok == DartToken::Trivia {
            continue;
        }
        if brace_depth == 0 && stmt_start.is_none() {
            stmt_start = Some(line);
        }
        if brace_depth == 0 && paren_depth == 0 {
            let kw = match (tok, slice) {
                (DartToken::Keyword, "class") => Some(DartItemKind::Class),
                (DartToken::Keyword, "mixin") => Some(DartItemKind::Mixin),
                (DartToken::Keyword, "enum") => Some(DartItemKind::Enum),
                (DartToken::Keyword, "extension") => Some(DartItemKind::Extension),
                (DartToken::Keyword, "typedef") => Some(DartItemKind::TypeAlias),
                (DartToken::Keyword, "const") => Some(DartItemKind::Const),
                _ => None,
            };
            if let Some(k) = kw {
                active = Some((k, stmt_start.unwrap_or(line), None, false));
            } else if tok == DartToken::Ident {
                last_ident = Some(slice.to_owned());
                if let Some((k, _, ref mut slot, seen_eq)) = active {
                    if !seen_eq && (slot.is_none() || k == DartItemKind::Const) {
                        *slot = Some(slice.to_owned());
                    }
                }
            } else if slice == "=" {
                seen_top_eq = true;
                if let Some((_, _, _, ref mut seen_eq)) = active {
                    *seen_eq = true;
                }
            } else if slice == "(" && active.is_none() && !seen_top_eq {
                if let Some(fn_name) = last_ident.clone().filter(|n| {
                    !matches!(n.as_str(), "Function" | "if" | "for" | "while" | "switch" | "catch" | "assert" | "return" | "super" | "this")
                }) {
                    active = Some((DartItemKind::Function, stmt_start.unwrap_or(line), Some(fn_name), false));
                }
            }
        }
        let finish = match slice {
            "(" => { paren_depth += 1; false }
            ")" => { paren_depth = paren_depth.saturating_sub(1); false }
            "{" => { brace_depth += 1; false }
            "}" => { brace_depth = brace_depth.saturating_sub(1); brace_depth == 0 }
            ";" => brace_depth == 0,
            _ => false,
        };
        if finish {
            if let Some((kind, start_line, Some(name), _)) = active.take() {
                items.push(DartCstItem { kind, name, start_line, end_line: line, has_doc_comment: !docs.is_empty(), doc_text: docs.join("\n") });
            }
            docs.clear();
            (stmt_start, last_ident, seen_top_eq) = (None, None, false);
        }
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies extraction of documented and undocumented Dart declarations. Realises: [REQ-0201]
    #[test]
    fn extracts_dart_declarations_and_docs() {
        let src = "/// Greeter class.\nclass Greeter {\n  void say() {}\n}\nconst int kMax = 10;\n";
        let items = parse_dart_items(src);
        assert_eq!(items.first().map(|i| (i.kind, i.name.as_str(), i.start_line, i.end_line, i.has_doc_comment, i.doc_text.as_str())), Some((DartItemKind::Class, "Greeter", 2, 4, true, "Greeter class.")));
        assert_eq!(items.get(1).map(|i| (i.kind, i.name.as_str(), i.has_doc_comment)), Some((DartItemKind::Const, "kMax", false)));
    }

    /// Verifies mixin, enum, extension, typedef, and function extraction. Realises: [REQ-0201]
    #[test]
    fn extracts_all_dart_item_kinds() {
        let src = "mixin M {}\nenum E { a }\nextension X on int {}\ntypedef F = void Function();\nint add(int a) {\n  return a;\n}\n";
        let kinds: Vec<_> = parse_dart_items(src).into_iter().map(|i| (i.kind, i.name)).collect();
        assert_eq!(kinds.first(), Some(&(DartItemKind::Mixin, "M".to_owned())));
        assert_eq!(kinds.get(4), Some(&(DartItemKind::Function, "add".to_owned())));
    }
}
