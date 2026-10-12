//! # RFC 6020 / RFC 7950 YANG to SysML v2 AST Translator Engine
//!
//! ## 1. Safety Intent & Regulatory Scope
//! This module provides deterministic translation of Internet Engineering Task Force (IETF)
//! RFC 6020 and RFC 7950 YANG data modeling language specifications into canonical SysML v2 textual
//! AST models (`deap_core::sysml_ast::PackageDef`).
//!
//! In high-integrity distributed systems and telecommunications architectures, YANG models define
//! authoritative structural network topologies, configuration constraints, state data trees, and
//! remote procedure call (RPC) payloads.
//!
//! ## 2. Core Safety Invariants
//! - **REQ-0015 (Zero-Regex Monotonic Scanning)**: Tokenization and parsing execute strictly via
//!   deterministic character iterators and finite-state token parsing without regular expression engines.
//! - **Zero Em Dash Invariant**: Exclusively ASCII characters (`--` or `-`) are utilized across all docstrings.
//! - **Structural Parity**: YANG `module` nodes map to `PackageDef` namespaces, `container` and `grouping`
//!   nodes map to `PartDef` components, `leaf` and `leaf-list` nodes map to `AttributeDef` entities, and
//!   `typedef` declarations map to formal SysML data types with boundary constraints (`ConstraintDef`).

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use deap_core::sysml_ast::{
    ActionDef, AttributeDef, ConstraintDef, ImportDef, PackageDef, PartDef, UseCaseDef,
};

/// SysML v2 reserved keywords that must not be used as bare identifiers in AST generation.
const RESERVED_SYSML_KEYWORDS: &[&str] = &[
    "package", "part", "def", "port", "attribute", "action", "flow",
    "item", "connect", "connection", "interface", "doc", "assert",
    "constraint", "state", "requirement", "use", "case", "test",
    "in", "out", "inout", "import", "public", "private", "alias",
    "perform", "subsystem", "first", "then", "step", "entry", "exit",
    "do", "transition", "assume", "require", "verify", "satisfy", "text",
];

fn is_reserved_keyword(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    RESERVED_SYSML_KEYWORDS.contains(&lower.as_str())
}

/// Converts arbitrary text into a valid PascalCase SysML v2 identifier.
pub fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for ch in s.chars() {
        if ch == '-' || ch == '_' || ch == '.' || ch == ':' || ch == '@' || ch.is_whitespace() {
            capitalize_next = true;
        } else if ch.is_alphanumeric() {
            if result.is_empty() && ch.is_ascii_digit() {
                result.push('_');
            }
            if capitalize_next {
                result.extend(ch.to_uppercase());
                capitalize_next = false;
            } else {
                result.push(ch);
            }
        }
    }

    if result.is_empty() {
        return "Item".to_string();
    }

    if is_reserved_keyword(&result) {
        result.push_str("_item");
    }

    result
}

/// Converts arbitrary text into a valid camelCase SysML v2 identifier.
pub fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    let mut is_first = true;

    for ch in s.chars() {
        if ch == '-' || ch == '_' || ch == '.' || ch == ':' || ch == '@' || ch.is_whitespace() {
            if !is_first {
                capitalize_next = true;
            }
        } else if ch.is_alphanumeric() {
            if result.is_empty() && ch.is_ascii_digit() {
                result.push('_');
            }
            if is_first {
                result.extend(ch.to_lowercase());
                is_first = false;
                capitalize_next = false;
            } else if capitalize_next {
                result.extend(ch.to_uppercase());
                capitalize_next = false;
            } else {
                result.push(ch);
            }
        }
    }

    if result.is_empty() {
        return "item".to_string();
    }

    if is_reserved_keyword(&result) {
        result.push_str("_val");
    }

    result
}

/// Converts arbitrary text into a valid snake_case SysML identifier.
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_is_sep = false;

    for ch in s.chars() {
        if ch == '-' || ch == '_' || ch == '.' || ch == ':' || ch == '@' || ch.is_whitespace() {
            if !prev_is_sep && !result.is_empty() {
                result.push('_');
                prev_is_sep = true;
            }
        } else if ch.is_alphanumeric() {
            if result.is_empty() && ch.is_ascii_digit() {
                result.push('_');
            }
            result.extend(ch.to_lowercase());
            prev_is_sep = false;
        }
    }

    if result.ends_with('_') {
        result.pop();
    }

    if result.is_empty() {
        return "item".to_string();
    }

    if is_reserved_keyword(&result) {
        result.push_str("_item");
    }

    result
}

/// Token types recognized by the pure lexical YANG scanner.
#[derive(Debug, Clone, PartialEq)]
enum YangToken {
    Word(String),
    OpenBrace,
    CloseBrace,
    Semicolon,
}

/// Skips whitespace and both single-line (`//`) and block (`/* ... */`) comments.
fn skip_whitespace_and_comments(chars: &[char], pos: &mut usize) {
    while *pos < chars.len() {
        let ch = chars[*pos];
        if ch.is_whitespace() {
            *pos += 1;
        } else if ch == '/' && *pos + 1 < chars.len() {
            if chars[*pos + 1] == '/' {
                *pos += 2;
                while *pos < chars.len() && chars[*pos] != '\n' {
                    *pos += 1;
                }
            } else if chars[*pos + 1] == '*' {
                *pos += 2;
                while *pos + 1 < chars.len() {
                    if chars[*pos] == '*' && chars[*pos + 1] == '/' {
                        *pos += 2;
                        break;
                    }
                    *pos += 1;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }
}

/// Scans a single quoted string slice (double or single quoted).
fn scan_quoted_string(chars: &[char], pos: &mut usize, quote_char: char) -> Result<String, String> {
    *pos += 1; // Consume opening quote
    let mut s = String::new();

    while *pos < chars.len() {
        let ch = chars[*pos];
        if quote_char == '"' && ch == '\\' {
            *pos += 1;
            if *pos >= chars.len() {
                return Err("Unterminated escape sequence in double-quoted string".to_string());
            }
            match chars[*pos] {
                'n' => s.push('\n'),
                't' => s.push('\t'),
                'r' => s.push('\r'),
                '"' => s.push('"'),
                '\\' => s.push('\\'),
                other => s.push(other),
            }
            *pos += 1;
        } else if ch == quote_char {
            *pos += 1; // Consume closing quote
            return Ok(s);
        } else {
            s.push(ch);
            *pos += 1;
        }
    }

    Err(format!("Unterminated quoted string starting with {}", quote_char))
}

/// Scans a quoted string literal including any RFC 6020 / RFC 7950 string concatenation with `+`.
fn scan_string_literal(chars: &[char], pos: &mut usize) -> Result<String, String> {
    let quote = chars[*pos];
    let mut full = scan_quoted_string(chars, pos, quote)?;

    loop {
        let saved_pos = *pos;
        skip_whitespace_and_comments(chars, pos);
        if *pos < chars.len() && chars[*pos] == '+' {
            *pos += 1; // Consume '+'
            skip_whitespace_and_comments(chars, pos);
            if *pos < chars.len() && (chars[*pos] == '"' || chars[*pos] == '\'') {
                let next_quote = chars[*pos];
                let next_part = scan_quoted_string(chars, pos, next_quote)?;
                full.push_str(&next_part);
                continue;
            } else {
                return Err("Expected string after '+' concatenation operator in YANG".to_string());
            }
        } else {
            *pos = saved_pos;
            break;
        }
    }

    Ok(full)
}

/// Pure lexical tokenizer for YANG source conforming to REQ-0015 (zero regex).
fn tokenize_yang(source: &str) -> Result<Vec<YangToken>, String> {
    let chars: Vec<char> = source.chars().collect();
    let mut pos = 0;
    let mut tokens = Vec::new();

    while pos < chars.len() {
        skip_whitespace_and_comments(&chars, &mut pos);
        if pos >= chars.len() {
            break;
        }

        let ch = chars[pos];
        match ch {
            '{' => {
                tokens.push(YangToken::OpenBrace);
                pos += 1;
            }
            '}' => {
                tokens.push(YangToken::CloseBrace);
                pos += 1;
            }
            ';' => {
                tokens.push(YangToken::Semicolon);
                pos += 1;
            }
            '"' | '\'' => {
                let s = scan_string_literal(&chars, &mut pos)?;
                tokens.push(YangToken::Word(s));
            }
            _ => {
                let mut word = String::new();
                while pos < chars.len() {
                    let c = chars[pos];
                    if c.is_whitespace() || c == '{' || c == '}' || c == ';' || c == '"' || c == '\'' {
                        break;
                    }
                    if c == '/' && pos + 1 < chars.len() && (chars[pos + 1] == '/' || chars[pos + 1] == '*') {
                        break;
                    }
                    word.push(c);
                    pos += 1;
                }
                if !word.is_empty() {
                    tokens.push(YangToken::Word(word));
                }
            }
        }
    }

    Ok(tokens)
}

/// Intermediate structural AST representation of a parsed YANG statement.
#[derive(Debug, Clone)]
pub struct YangStmt {
    pub keyword: String,
    pub argument: Option<String>,
    pub substatements: Vec<YangStmt>,
}

impl YangStmt {
    /// Returns the argument of the first substatement matching `kw`.
    pub fn get_arg(&self, kw: &str) -> Option<&str> {
        self.substatements
            .iter()
            .find(|s| s.keyword == kw)
            .and_then(|s| s.argument.as_deref())
    }

    /// Returns the arguments of all substatements matching `kw`.
    pub fn get_all_args(&self, kw: &str) -> Vec<&str> {
        self.substatements
            .iter()
            .filter(|s| s.keyword == kw)
            .filter_map(|s| s.argument.as_deref())
            .collect()
    }

    /// Finds the first substatement with keyword `kw`.
    pub fn find_first(&self, kw: &str) -> Option<&YangStmt> {
        self.substatements.iter().find(|s| s.keyword == kw)
    }

    /// Finds all substatements with keyword `kw`.
    pub fn find_all(&self, kw: &str) -> Vec<&YangStmt> {
        self.substatements.iter().filter(|s| s.keyword == kw).collect()
    }
}

/// Recursively parses a single YANG statement from the token stream.
fn parse_statement(tokens: &[YangToken], pos: &mut usize) -> Result<YangStmt, String> {
    let keyword = match tokens.get(*pos) {
        Some(YangToken::Word(kw)) => {
            *pos += 1;
            kw.clone()
        }
        Some(tok) => return Err(format!("Expected keyword, found {:?}", tok)),
        None => return Err("Unexpected end of tokens while expecting statement keyword".to_string()),
    };

    let mut argument = None;
    if let Some(tok) = tokens.get(*pos) {
        match tok {
            YangToken::Semicolon => {
                *pos += 1;
                return Ok(YangStmt {
                    keyword,
                    argument: None,
                    substatements: Vec::new(),
                });
            }
            YangToken::OpenBrace => {
                *pos += 1;
                let subs = parse_substatements(tokens, pos)?;
                return Ok(YangStmt {
                    keyword,
                    argument: None,
                    substatements: subs,
                });
            }
            YangToken::Word(arg) => {
                argument = Some(arg.clone());
                *pos += 1;
            }
            YangToken::CloseBrace => {
                return Err(format!("Unexpected '}}' directly after keyword '{}'", keyword));
            }
        }
    }

    match tokens.get(*pos) {
        Some(YangToken::Semicolon) => {
            *pos += 1;
            Ok(YangStmt {
                keyword,
                argument,
                substatements: Vec::new(),
            })
        }
        Some(YangToken::OpenBrace) => {
            *pos += 1;
            let subs = parse_substatements(tokens, pos)?;
            Ok(YangStmt {
                keyword,
                argument,
                substatements: subs,
            })
        }
        Some(tok) => Err(format!(
            "Expected ';' or '{{' after argument for '{}', found {:?}",
            keyword, tok
        )),
        None => Err(format!("Unexpected end of tokens after argument for '{}'", keyword)),
    }
}

/// Recursively parses substatements enclosed in braces (`{ ... }`).
fn parse_substatements(tokens: &[YangToken], pos: &mut usize) -> Result<Vec<YangStmt>, String> {
    let mut stmts = Vec::new();
    while *pos < tokens.len() {
        if let Some(YangToken::CloseBrace) = tokens.get(*pos) {
            *pos += 1;
            return Ok(stmts);
        }
        stmts.push(parse_statement(tokens, pos)?);
    }
    Err("Unclosed YANG block: missing '}'".to_string())
}

/// Parses the full token stream into top-level statements.
fn parse_yang_statements(tokens: &[YangToken]) -> Result<Vec<YangStmt>, String> {
    let mut pos = 0;
    let mut stmts = Vec::new();
    while pos < tokens.len() {
        stmts.push(parse_statement(tokens, &mut pos)?);
    }
    Ok(stmts)
}

/// Maps a YANG data type identifier to a canonical SysML v2 primitive or resolved type.
fn map_yang_type(raw_type: &str, typedef_map: &HashMap<String, String>) -> String {
    let trimmed = raw_type.trim();
    let unaliased = if let Some(idx) = trimmed.find(':') {
        &trimmed[idx + 1..]
    } else {
        trimmed
    };

    if let Some(mapped) = typedef_map.get(unaliased) {
        return mapped.clone();
    }

    match unaliased {
        "int8" | "int16" | "int32" | "int64"
        | "uint8" | "uint16" | "uint32" | "uint64"
        | "counter32" | "counter64" | "zero-based-counter32" | "zero-based-counter64"
        | "gauge32" | "gauge64" => "Integer".to_string(),
        "decimal64" | "float" => "Real".to_string(),
        "boolean" | "empty" => "Boolean".to_string(),
        "string" | "binary" | "enumeration" | "identityref"
        | "leafref" | "instance-identifier" | "union" | "bits" => "String".to_string(),
        other => to_pascal_case(other),
    }
}

/// Deterministic translator converting RFC 6020 / RFC 7950 YANG specifications into SysML v2 ASTs.
#[derive(Debug, Default, Clone)]
pub struct YangTranslator;

impl YangTranslator {
    /// Creates a fresh instance of the YANG translator.
    pub fn new() -> Self {
        Self
    }

    /// Translates a single YANG schema file content into a canonical SysML v2 `PackageDef`.
    pub fn translate(&self, content: &str, default_name: &str) -> Result<PackageDef, String> {
        let tokens = tokenize_yang(content)?;
        let stmts = parse_yang_statements(&tokens)?;

        let module_stmt = stmts
            .iter()
            .find(|s| s.keyword == "module" || s.keyword == "submodule");

        let mod_name = module_stmt
            .and_then(|s| s.argument.as_deref())
            .unwrap_or(default_name);

        let mod_subs: &[YangStmt] = if let Some(m) = module_stmt {
            &m.substatements
        } else {
            &stmts
        };

        let pkg_name = to_pascal_case(mod_name);
        let doc = module_stmt
            .and_then(|s| s.get_arg("description"))
            .map(|d| d.to_string());

        let mut pkg = PackageDef {
            name: pkg_name.clone(),
            doc,
            ..Default::default()
        };

        // 1. Process imports
        for stmt in mod_subs {
            if stmt.keyword == "import" {
                if let Some(target) = stmt.argument.as_deref() {
                    pkg.imports.push(ImportDef {
                        path: to_pascal_case(target),
                        is_wildcard: true,
                        is_recursive: false,
                        doc: stmt.get_arg("reference").map(|r| r.to_string()),
                    });
                }
            }
        }

        // 2. Build local typedef resolution map
        let mut typedef_map: HashMap<String, String> = HashMap::new();
        for stmt in mod_subs {
            if stmt.keyword == "typedef" {
                if let Some(td_name) = stmt.argument.as_deref() {
                    let base_type = stmt
                        .find_first("type")
                        .and_then(|t| t.argument.as_deref())
                        .unwrap_or("string");
                    let resolved = map_yang_type(base_type, &typedef_map);
                    typedef_map.insert(td_name.to_string(), resolved);
                }
            }
        }

        // 3. Translate typedefs into package attributes and structural PartDefs
        for stmt in mod_subs {
            if stmt.keyword == "typedef" {
                if let Some(td_name) = stmt.argument.as_deref() {
                    let base_type = typedef_map
                        .get(td_name)
                        .cloned()
                        .unwrap_or_else(|| "String".to_string());
                    let td_doc = stmt.get_arg("description").map(|d| d.to_string());
                    let default_val = stmt.get_arg("default").map(|d| d.to_string());

                    // Attribute alias on package
                    pkg.attribute_defs.push(AttributeDef {
                        name: to_camel_case(td_name),
                        type_name: base_type.clone(),
                        default_value: default_val.clone(),
                        doc: td_doc.clone(),
                    });

                    // Build formal constraints
                    let mut constraints = Vec::new();
                    let type_stmt = stmt.find_first("type");

                    if let Some(t_stmt) = type_stmt {
                        if let Some(range_val) = t_stmt.get_arg("range") {
                            constraints.push(ConstraintDef {
                                name: format!("{}_range", to_snake_case(td_name)),
                                expression: format!("value in {}", range_val),
                                is_assertion: true,
                                doc: Some(format!("Range constraint: {}", range_val)),
                                ..Default::default()
                            });
                        }
                        if let Some(len_val) = t_stmt.get_arg("length") {
                            constraints.push(ConstraintDef {
                                name: format!("{}_length", to_snake_case(td_name)),
                                expression: format!("length(value) in {}", len_val),
                                is_assertion: true,
                                doc: Some(format!("Length constraint: {}", len_val)),
                                ..Default::default()
                            });
                        }
                        for (idx, pat_val) in t_stmt.get_all_args("pattern").into_iter().enumerate() {
                            let clean_pat = pat_val.replace('"', "\\\"");
                            constraints.push(ConstraintDef {
                                name: format!("{}_pattern_{}", to_snake_case(td_name), idx + 1),
                                expression: format!("matches(value, \"{}\")", clean_pat),
                                is_assertion: true,
                                doc: Some(format!("Pattern constraint: {}", pat_val)),
                                ..Default::default()
                            });
                        }
                        if let Some(fd) = t_stmt.get_arg("fraction-digits") {
                            constraints.push(ConstraintDef {
                                name: format!("{}_fraction_digits", to_snake_case(td_name)),
                                expression: format!("fraction_digits <= {}", fd),
                                is_assertion: true,
                                doc: Some(format!("Fraction digits bound: {}", fd)),
                                ..Default::default()
                            });
                        }
                        let enums = t_stmt.find_all("enum");
                        if !enums.is_empty() {
                            let enum_names: Vec<&str> = enums
                                .iter()
                                .filter_map(|e| e.argument.as_deref())
                                .collect();
                            let enum_conds: Vec<String> = enum_names
                                .iter()
                                .map(|n| format!("value == \"{}\"", n))
                                .collect();
                            constraints.push(ConstraintDef {
                                name: format!("{}_enum_values", to_snake_case(td_name)),
                                expression: enum_conds.join(" || "),
                                is_assertion: true,
                                doc: Some("Enumeration value constraints".to_string()),
                                ..Default::default()
                            });
                        }
                    }

                    // Structural PartDef for the type definition
                    pkg.part_defs.push(PartDef {
                        name: to_pascal_case(td_name),
                        doc: td_doc,
                        attributes: vec![AttributeDef {
                            name: "value".to_string(),
                            type_name: base_type,
                            default_value: default_val,
                            doc: None,
                        }],
                        constraints,
                        is_def: true,
                        ..Default::default()
                    });
                }
            }
        }

        // 4. Translate feature flags into PartDefs
        for stmt in mod_subs {
            if stmt.keyword == "feature" {
                if let Some(feat_name) = stmt.argument.as_deref() {
                    let feat_doc = stmt.get_arg("description").map(|d| d.to_string());
                    pkg.part_defs.push(PartDef {
                        name: to_pascal_case(feat_name),
                        doc: feat_doc,
                        is_def: true,
                        ..Default::default()
                    });
                }
            }
        }

        // 5. Translate groupings and containers
        let mut extra_parts = Vec::new();
        for stmt in mod_subs {
            if stmt.keyword == "grouping" || stmt.keyword == "container" {
                let part = translate_container_or_grouping(stmt, &typedef_map, &mut extra_parts);
                pkg.part_defs.push(part);
            }
        }
        pkg.part_defs.extend(extra_parts);

        // 6. Translate RPC / action statements
        for stmt in mod_subs {
            if stmt.keyword == "rpc" || stmt.keyword == "action" {
                if let Some(rpc_name) = stmt.argument.as_deref() {
                    let rpc_doc = stmt.get_arg("description").map(|d| d.to_string());
                    let mut in_params = Vec::new();
                    let mut out_params = Vec::new();

                    if let Some(input_stmt) = stmt.find_first("input") {
                        for leaf in input_stmt.find_all("leaf") {
                            if let Some(l_name) = leaf.argument.as_deref() {
                                let l_type = leaf
                                    .find_first("type")
                                    .and_then(|t| t.argument.as_deref())
                                    .unwrap_or("string");
                                in_params.push(AttributeDef {
                                    name: to_camel_case(l_name),
                                    type_name: map_yang_type(l_type, &typedef_map),
                                    default_value: leaf.get_arg("default").map(|d| d.to_string()),
                                    doc: leaf.get_arg("description").map(|d| d.to_string()),
                                });
                            }
                        }
                    }

                    if let Some(output_stmt) = stmt.find_first("output") {
                        for leaf in output_stmt.find_all("leaf") {
                            if let Some(l_name) = leaf.argument.as_deref() {
                                let l_type = leaf
                                    .find_first("type")
                                    .and_then(|t| t.argument.as_deref())
                                    .unwrap_or("string");
                                out_params.push(AttributeDef {
                                    name: to_camel_case(l_name),
                                    type_name: map_yang_type(l_type, &typedef_map),
                                    default_value: leaf.get_arg("default").map(|d| d.to_string()),
                                    doc: leaf.get_arg("description").map(|d| d.to_string()),
                                });
                            }
                        }
                    }

                    pkg.action_defs.push(ActionDef {
                        name: to_pascal_case(rpc_name),
                        doc: rpc_doc,
                        in_params,
                        out_params,
                        is_def: true,
                        ..Default::default()
                    });
                }
            }
        }

        // Add standard operational lifecycle action and use case for downstream projection
        pkg.action_defs.push(ActionDef {
            name: format!("Validate{}", pkg.name),
            doc: Some(format!("Validates {} data contracts and invariants", pkg.name)),
            is_def: true,
            ..Default::default()
        });

        pkg.use_case_defs.push(UseCaseDef {
            name: format!("Manage{}", pkg.name),
            subject: Some(pkg.name.clone()),
            doc: Some(format!(
                "Subject: {}\nMain Success Scenario:\n1. Initialize {}\n2. Validate and enforce operational constraints",
                pkg.name, pkg.name
            )),
            ..Default::default()
        });

        // 7. Deduplicate part_defs by name
        let mut dedup_parts: Vec<PartDef> = Vec::new();
        for p in pkg.part_defs {
            if let Some(existing) = dedup_parts.iter_mut().find(|existing| existing.name == p.name) {
                // Merge attributes and child parts
                for attr in p.attributes {
                    if !existing.attributes.iter().any(|a| a.name == attr.name) {
                        existing.attributes.push(attr);
                    }
                }
                for sub in p.parts {
                    if !existing.parts.iter().any(|s| s.name == sub.name) {
                        existing.parts.push(sub);
                    }
                }
                for c in p.constraints {
                    if !existing.constraints.iter().any(|ec| ec.name == c.name) {
                        existing.constraints.push(c);
                    }
                }
            } else {
                dedup_parts.push(p);
            }
        }
        pkg.part_defs = dedup_parts;

        Ok(pkg)
    }

    /// Batch-translates multiple YANG schema files into a unified multi-package `PackageDef`.
    pub fn translate_files(&self, file_paths: &[String], default_name: &str) -> Result<PackageDef, String> {
        let mut sub_packages = Vec::new();

        for fpath in file_paths {
            let path = Path::new(fpath);
            if !path.exists() {
                continue;
            }
            let content = fs::read_to_string(path)
                .map_err(|e| format!("Failed to read schema file '{}': {}", fpath, e))?;
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("YangModule");
            let sub_pkg = self.translate(&content, stem)?;
            sub_packages.push((stem.to_string(), sub_pkg));
        }

        if sub_packages.is_empty() {
            return Err("Zero valid YANG schema files were loaded".to_string());
        }

        if sub_packages.len() == 1 {
            return Ok(sub_packages.remove(0).1);
        }

        // Detect name collisions among translated packages (e.g. multiple revisions of the same module)
        let mut name_counts: HashMap<String, usize> = HashMap::new();
        for (_, pkg) in &sub_packages {
            *name_counts.entry(pkg.name.clone()).or_insert(0) += 1;
        }

        let mut finalized_packages = Vec::new();
        for (stem, mut pkg) in sub_packages {
            if name_counts.get(&pkg.name).copied().unwrap_or(0) > 1 {
                // Disambiguate with revision from stem (e.g. ietf-inet-types@2013-07-15 -> IetfInetTypes_2013_07_15)
                if let Some(rev_idx) = stem.find('@') {
                    let rev_suffix = to_pascal_case(&stem[rev_idx + 1..]);
                    pkg.name = format!("{}_{}", pkg.name, rev_suffix);
                } else {
                    let stem_clean = to_pascal_case(&stem);
                    pkg.name = stem_clean;
                }
            }
            finalized_packages.push(pkg);
        }

        let root_name = if default_name == "schema" || default_name == "Model" || default_name.is_empty() {
            "SysML_Model".to_string()
        } else {
            to_pascal_case(default_name)
        };

        let root_pkg = PackageDef {
            name: root_name,
            doc: Some("Consolidated YANG model suite".to_string()),
            packages: finalized_packages,
            ..Default::default()
        };

        Ok(root_pkg)
    }
}

/// Recursively translates a YANG container or grouping statement into a structural `PartDef`.
fn translate_container_or_grouping(
    stmt: &YangStmt,
    typedef_map: &HashMap<String, String>,
    extra_parts: &mut Vec<PartDef>,
) -> PartDef {
    let name = stmt.argument.as_deref().unwrap_or("Anonymous");
    let part_name = to_pascal_case(name);
    let doc = stmt.get_arg("description").map(|d| d.to_string());

    let mut attributes = Vec::new();
    let mut parts = Vec::new();
    let mut constraints = Vec::new();

    // 1. Process leaves
    for leaf in stmt.find_all("leaf") {
        if let Some(l_name) = leaf.argument.as_deref() {
            let l_type = leaf
                .find_first("type")
                .and_then(|t| t.argument.as_deref())
                .unwrap_or("string");
            let mapped_type = map_yang_type(l_type, typedef_map);
            let default_val = leaf.get_arg("default").map(|d| d.to_string());
            let l_doc = leaf.get_arg("description").map(|d| d.to_string());

            attributes.push(AttributeDef {
                name: to_camel_case(l_name),
                type_name: mapped_type,
                default_value: default_val,
                doc: l_doc,
            });

            // Extract leaf constraints
            if let Some(t_stmt) = leaf.find_first("type") {
                if let Some(range_val) = t_stmt.get_arg("range") {
                    constraints.push(ConstraintDef {
                        name: format!("{}_{}_range", to_snake_case(name), to_snake_case(l_name)),
                        expression: format!("{} in {}", to_camel_case(l_name), range_val),
                        is_assertion: true,
                        doc: Some(format!("Range constraint: {}", range_val)),
                        ..Default::default()
                    });
                }
                if let Some(len_val) = t_stmt.get_arg("length") {
                    constraints.push(ConstraintDef {
                        name: format!("{}_{}_length", to_snake_case(name), to_snake_case(l_name)),
                        expression: format!("length({}) in {}", to_camel_case(l_name), len_val),
                        is_assertion: true,
                        doc: Some(format!("Length constraint: {}", len_val)),
                        ..Default::default()
                    });
                }
                for (idx, pat_val) in t_stmt.get_all_args("pattern").into_iter().enumerate() {
                    let clean_pat = pat_val.replace('"', "\\\"");
                    constraints.push(ConstraintDef {
                        name: format!("{}_{}_pattern_{}", to_snake_case(name), to_snake_case(l_name), idx + 1),
                        expression: format!("matches({}, \"{}\")", to_camel_case(l_name), clean_pat),
                        is_assertion: true,
                        doc: Some(format!("Pattern constraint: {}", pat_val)),
                        ..Default::default()
                    });
                }
            }
        }
    }

    // 2. Process leaf-lists
    for llist in stmt.find_all("leaf-list") {
        if let Some(ll_name) = llist.argument.as_deref() {
            let ll_type = llist
                .find_first("type")
                .and_then(|t| t.argument.as_deref())
                .unwrap_or("string");
            let mapped_type = format!("{}[]", map_yang_type(ll_type, typedef_map));

            attributes.push(AttributeDef {
                name: to_camel_case(ll_name),
                type_name: mapped_type,
                default_value: None,
                doc: llist.get_arg("description").map(|d| d.to_string()),
            });
        }
    }

    // 3. Process nested containers and lists
    for child in &stmt.substatements {
        if child.keyword == "container" || child.keyword == "list" {
            if let Some(c_name) = child.argument.as_deref() {
                let child_part = translate_container_or_grouping(child, typedef_map, extra_parts);
                parts.push(PartDef {
                    name: to_camel_case(c_name),
                    type_name: Some(child_part.name.clone()),
                    is_def: false,
                    ..Default::default()
                });
                extra_parts.push(child_part);
            }
        }
    }

    // 4. Process choices and cases
    for choice in stmt.find_all("choice") {
        if let Some(choice_name) = choice.argument.as_deref() {
            let mut case_names = Vec::new();

            for case_stmt in choice.find_all("case") {
                if let Some(cs_name) = case_stmt.argument.as_deref() {
                    case_names.push(to_camel_case(cs_name));
                    let case_part = translate_container_or_grouping(case_stmt, typedef_map, extra_parts);
                    parts.push(PartDef {
                        name: to_camel_case(cs_name),
                        type_name: Some(case_part.name.clone()),
                        is_def: false,
                        ..Default::default()
                    });
                    extra_parts.push(case_part);
                }
            }

            // Handle direct leaf shorthand within choice
            for direct_leaf in choice.find_all("leaf") {
                if let Some(dl_name) = direct_leaf.argument.as_deref() {
                    case_names.push(to_camel_case(dl_name));
                    let l_type = direct_leaf
                        .find_first("type")
                        .and_then(|t| t.argument.as_deref())
                        .unwrap_or("string");
                    attributes.push(AttributeDef {
                        name: to_camel_case(dl_name),
                        type_name: map_yang_type(l_type, typedef_map),
                        default_value: direct_leaf.get_arg("default").map(|d| d.to_string()),
                        doc: direct_leaf.get_arg("description").map(|d| d.to_string()),
                    });
                }
            }

            // Mutual exclusion invariant
            if !case_names.is_empty() {
                constraints.push(ConstraintDef {
                    name: format!("{}_mutex", to_snake_case(choice_name)),
                    expression: format!("xor({})", case_names.join(", ")),
                    is_assertion: true,
                    doc: Some(format!("Choice {} mutual exclusion invariant", choice_name)),
                    ..Default::default()
                });
            }
        }
    }

    // 5. Process must / when expressions
    for (idx, must_stmt) in stmt.find_all("must").into_iter().enumerate() {
        if let Some(expr) = must_stmt.argument.as_deref() {
            constraints.push(ConstraintDef {
                name: format!("{}_must_{}", to_snake_case(name), idx + 1),
                expression: expr.to_string(),
                is_assertion: true,
                doc: must_stmt.get_arg("description").map(|d| d.to_string()),
                ..Default::default()
            });
        }
    }

    PartDef {
        name: part_name,
        doc,
        attributes,
        parts,
        constraints,
        is_def: true,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yang_scanner_and_statement_parsing() {
        let yang_src = r#"
            module test-module {
                yang-version 1.1;
                namespace "urn:ietf:params:xml:ns:yang:test-module";
                prefix tm;

                description "Test module description " + "with concatenation.";

                typedef test-type {
                    type string {
                        length "1..255";
                        pattern '[a-z]+';
                    }
                }

                container test-container {
                    leaf id {
                        type uint32;
                    }
                    choice opt {
                        case a {
                            leaf val-a { type string; }
                        }
                        case b {
                            leaf val-b { type boolean; }
                        }
                    }
                }
            }
        "#;

        let translator = YangTranslator::new();
        let pkg = translator.translate(yang_src, "TestModule").unwrap();
        assert_eq!(pkg.name, "TestModule");
        assert!(pkg.doc.as_deref().unwrap().contains("Test module description with concatenation."));

        // Check typedef
        assert!(pkg.attribute_defs.iter().any(|a| a.name == "testType"));
        assert!(pkg.part_defs.iter().any(|p| p.name == "TestType"));

        // Check container
        let container = pkg.part_defs.iter().find(|p| p.name == "TestContainer").unwrap();
        assert!(container.attributes.iter().any(|a| a.name == "id"));
        assert!(container.parts.iter().any(|p| p.name == "a"));
        assert!(container.parts.iter().any(|p| p.name == "b"));
        assert!(container.constraints.iter().any(|c| c.name == "opt_mutex"));
    }
}
