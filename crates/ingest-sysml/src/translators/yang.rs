//! # RFC 6020 / RFC 7950 YANG to SysML v2 AST Translator Engine
//!
//! Safety & Traceability: Realises: [REQ-0015], [REQ-0201]
//! Contract: Deterministic translation of IETF RFC 6020 and RFC 7950 YANG data modeling specifications into canonical SysML v2 `PackageDef` ASTs via `yang_cst` (`logos` + `rowan`).
//! Bounds & Faults: O(N) time and O(N) space over YANG source bytes; returns `Err(String)` on lexical, syntactic, or I/O faults.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use deap_core::sysml_ast::{ActionDef, AttributeDef, ConstraintDef, ImportDef, PackageDef, PartDef, UseCaseDef};
use rowan::{NodeOrToken, SyntaxNode, SyntaxToken};

use crate::translators::yang_cst::{parse_yang_cst, YangLanguage, YangSyntaxKind};

/// SysML v2 reserved keywords that must not be used as bare identifiers in AST generation.
const RESERVED_SYSML_KEYWORDS: &[&str] = &[
    "package", "part", "def", "port", "attribute", "action", "flow", "item", "connect", "connection",
    "interface", "doc", "assert", "constraint", "state", "requirement", "use", "case", "test", "in",
    "out", "inout", "import", "public", "private", "alias", "perform", "subsystem", "first", "then",
    "step", "entry", "exit", "do", "transition", "assume", "require", "verify", "satisfy", "text",
];

/// Checks whether `s` is a reserved SysML v2 keyword (case-insensitive). Realises: [REQ-TRANSLATORS-YANG-IS-RESERVED-KEYWORD]
fn is_reserved_keyword(s: &str) -> bool { RESERVED_SYSML_KEYWORDS.contains(&s.to_ascii_lowercase().as_str()) }

/// Converts arbitrary text into a valid PascalCase SysML v2 identifier. Realises: [REQ-TRANSLATORS-YANG-TO-PASCAL-CASE]
pub fn to_pascal_case(s: &str) -> String {
    let (mut result, mut cap) = (String::new(), true);
    for ch in s.chars() {
        if matches!(ch, '-' | '_' | '.' | ':' | '@') || ch.is_whitespace() { cap = true; }
        else if ch.is_alphanumeric() {
            if result.is_empty() && ch.is_ascii_digit() { result.push('_'); }
            if cap { result.extend(ch.to_uppercase()); cap = false; } else { result.push(ch); }
        }
    }
    if result.is_empty() { return "Item".to_string(); }
    if is_reserved_keyword(&result) { result.push_str("_item"); }
    result
}

/// Converts arbitrary text into a valid camelCase SysML v2 identifier. Realises: [REQ-TRANSLATORS-YANG-TO-CAMEL-CASE]
pub fn to_camel_case(s: &str) -> String {
    let (mut result, mut cap, mut is_first) = (String::new(), false, true);
    for ch in s.chars() {
        if matches!(ch, '-' | '_' | '.' | ':' | '@') || ch.is_whitespace() { if !is_first { cap = true; } }
        else if ch.is_alphanumeric() {
            if result.is_empty() && ch.is_ascii_digit() { result.push('_'); }
            if is_first { result.extend(ch.to_lowercase()); is_first = false; cap = false; }
            else if cap { result.extend(ch.to_uppercase()); cap = false; } else { result.push(ch); }
        }
    }
    if result.is_empty() { return "item".to_string(); }
    if is_reserved_keyword(&result) { result.push_str("_val"); }
    result
}

/// Converts arbitrary text into a valid snake_case SysML identifier. Realises: [REQ-TRANSLATORS-YANG-TO-SNAKE-CASE]
pub fn to_snake_case(s: &str) -> String {
    let (mut result, mut prev_is_sep) = (String::new(), false);
    for ch in s.chars() {
        if matches!(ch, '-' | '_' | '.' | ':' | '@') || ch.is_whitespace() {
            if !prev_is_sep && !result.is_empty() { result.push('_'); prev_is_sep = true; }
        } else if ch.is_alphanumeric() {
            if result.is_empty() && ch.is_ascii_digit() { result.push('_'); }
            result.extend(ch.to_lowercase());
            prev_is_sep = false;
        }
    }
    if result.ends_with('_') { result.pop(); }
    if result.is_empty() { return "item".to_string(); }
    if is_reserved_keyword(&result) { result.push_str("_item"); }
    result
}

/// Intermediate structural AST representation of a parsed YANG statement. Realises: [REQ-TRANSLATORS-YANG-YANGSTMT]
#[derive(Debug, Clone)]
pub struct YangStmt {
    pub keyword: String,
    pub argument: Option<String>,
    pub substatements: Vec<YangStmt>,
}

impl YangStmt {
    /// Returns the argument of the first substatement matching `kw`. Realises: [REQ-TRANSLATORS-YANG-YANGSTMT-GET-ARG]
    pub fn get_arg(&self, kw: &str) -> Option<&str> { self.substatements.iter().find(|s| s.keyword == kw).and_then(|s| s.argument.as_deref()) }
    /// Returns the arguments of all substatements matching `kw`. Realises: [REQ-TRANSLATORS-YANG-YANGSTMT-GET-ALL-ARGS]
    pub fn get_all_args(&self, kw: &str) -> Vec<&str> { self.substatements.iter().filter(|s| s.keyword == kw).filter_map(|s| s.argument.as_deref()).collect() }
    /// Finds the first substatement with keyword `kw`. Realises: [REQ-TRANSLATORS-YANG-YANGSTMT-FIND-FIRST]
    pub fn find_first(&self, kw: &str) -> Option<&YangStmt> { self.substatements.iter().find(|s| s.keyword == kw) }
    /// Finds all substatements with keyword `kw`. Realises: [REQ-TRANSLATORS-YANG-YANGSTMT-FIND-ALL]
    pub fn find_all(&self, kw: &str) -> Vec<&YangStmt> { self.substatements.iter().filter(|s| s.keyword == kw).collect() }
}

/// Unescapes a single-quoted or double-quoted YANG string literal token from the CST. Realises: [REQ-0015], [REQ-0201]
fn unescape_yang_string(raw: &str) -> String {
    if let Some(inner) = raw.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')) { return inner.to_string(); }
    let inner = raw.strip_prefix('"').and_then(|s| s.strip_suffix('"')).unwrap_or(raw);
    let (mut out, mut chars) = (String::with_capacity(inner.len()), inner.chars());
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some(c) => out.push(c),
                None => out.push('\\'),
            }
        } else { out.push(ch); }
    }
    out
}

/// Coalesces non-trivia CST header tokens (before `;` or `{`) into `[keyword, optional_arg]`. Realises: [REQ-0015], [REQ-0201]
fn coalesce_header_words(tokens: &[SyntaxToken<YangLanguage>]) -> Result<Vec<String>, String> {
    let (mut words, mut prev_end, mut idx) = (Vec::<String>::new(), None, 0usize);
    while let Some(tok) = tokens.get(idx) {
        idx += 1;
        if tok.kind() == YangSyntaxKind::Error && (tok.text().starts_with('"') || tok.text().starts_with('\'')) {
            return Err("Unterminated quoted string in YANG statement".to_string());
        }
        if tok.kind() == YangSyntaxKind::StringLit {
            let mut s = unescape_yang_string(tok.text());
            while tokens.get(idx).is_some_and(|t| t.kind() == YangSyntaxKind::Punct && t.text() == "+") {
                idx += 1;
                let Some(next_str) = tokens.get(idx).filter(|t| t.kind() == YangSyntaxKind::StringLit) else {
                    return Err("Expected string after '+' concatenation operator in YANG".to_string());
                };
                s.push_str(&unescape_yang_string(next_str.text()));
                idx += 1;
            }
            words.push(s);
            prev_end = None;
        } else {
            let (start, end) = (usize::from(tok.text_range().start()), usize::from(tok.text_range().end()));
            if prev_end == Some(start) {
                if let Some(last) = words.last_mut() { last.push_str(tok.text()); }
            } else {
                words.push(tok.text().to_string());
            }
            prev_end = Some(end);
        }
    }
    Ok(words)
}

/// Lowers a single `rowan` CST `Statement` node into a recursive `YangStmt` AST node. Realises: [REQ-0015], [REQ-0201]
fn lower_cst_statement(node: &SyntaxNode<YangLanguage>) -> Result<YangStmt, String> {
    let direct_tokens: Vec<SyntaxToken<YangLanguage>> = node.children_with_tokens().filter_map(NodeOrToken::into_token)
        .filter(|t| !matches!(t.kind(), YangSyntaxKind::Whitespace | YangSyntaxKind::LineComment | YangSyntaxKind::BlockComment)).collect();
    let header_slice = match direct_tokens.as_slice() {
        [head @ .., last] if last.kind() == YangSyntaxKind::Punct && last.text() == ";" => head,
        [head @ .., open, close] if open.kind() == YangSyntaxKind::Punct && open.text() == "{" && close.kind() == YangSyntaxKind::Punct && close.text() == "}" => head,
        _ if direct_tokens.iter().any(|t| t.kind() == YangSyntaxKind::Punct && t.text() == "{") => return Err("Unclosed YANG block: missing '}'".to_string()),
        _ => return Err("Unexpected end of tokens in YANG statement".to_string()),
    };
    let mut words = coalesce_header_words(header_slice)?;
    let (keyword, argument) = match words.len() {
        1 => (words.remove(0), None),
        2 => { let arg = words.pop(); (words.remove(0), arg) }
        0 => return Err("Expected statement keyword in YANG statement".to_string()),
        _ => return Err(format!("Expected ';' or '{{' after argument for '{}'", words.first().map_or("", String::as_str))),
    };
    let substatements = node.children().filter(|c| c.kind() == YangSyntaxKind::Statement).map(|c| lower_cst_statement(&c)).collect::<Result<Vec<_>, _>>()?;
    Ok(YangStmt { keyword, argument, substatements })
}

/// Parses a YANG source string into top-level `YangStmt` trees via `yang_cst::parse_yang_cst`. Realises: [REQ-TRANSLATORS-YANG-PARSE-YANG-STATEMENTS]
fn parse_yang_statements(source: &str) -> Result<Vec<YangStmt>, String> {
    let cst = parse_yang_cst(source);
    let root: SyntaxNode<YangLanguage> = SyntaxNode::new_root(cst.green);
    if root.children_with_tokens().filter_map(NodeOrToken::into_token).any(|t| !matches!(t.kind(), YangSyntaxKind::Whitespace | YangSyntaxKind::LineComment | YangSyntaxKind::BlockComment)) {
        return Err("Unexpected top-level token outside YANG statement".to_string());
    }
    root.children().filter(|n| n.kind() == YangSyntaxKind::Statement).map(|n| lower_cst_statement(&n)).collect()
}

/// Maps a YANG data type identifier to a canonical SysML v2 primitive or resolved type. Realises: [REQ-TRANSLATORS-YANG-MAP-YANG-TYPE]
fn map_yang_type(raw_type: &str, typedef_map: &BTreeMap<String, String>) -> String {
    let trimmed = raw_type.trim();
    let unaliased = trimmed.split_once(':').map_or(trimmed, |(_, rest)| rest);
    if let Some(mapped) = typedef_map.get(unaliased) { return mapped.clone(); }
    match unaliased {
        "int8" | "int16" | "int32" | "int64" | "uint8" | "uint16" | "uint32" | "uint64"
        | "counter32" | "counter64" | "zero-based-counter32" | "zero-based-counter64"
        | "gauge32" | "gauge64" => "Integer".to_string(),
        "decimal64" | "float" => "Real".to_string(),
        "boolean" | "empty" => "Boolean".to_string(),
        "string" | "binary" | "enumeration" | "identityref" | "leafref" | "instance-identifier" | "union" | "bits" => "String".to_string(),
        other => to_pascal_case(other),
    }
}

/// Renders a raw YANG `default` argument as a SysML v2 literal compatible with the mapped type. Realises: [REQ-0015/YANG-DEFAULT-VALUE-SYSML-LITERAL]
fn format_yang_default_value(raw_def: &str, sysml_type: &str) -> String {
    if ((sysml_type == "Integer" || sysml_type == "Real") && raw_def.parse::<f64>().is_ok()) || (sysml_type == "Boolean" && matches!(raw_def, "true" | "false")) {
        return raw_def.to_string();
    }
    format!("\"{}\"", raw_def.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Constructs an assertion `ConstraintDef` node. Realises: [REQ-0015], [REQ-0201]
fn make_constraint(name: String, expression: String, doc: Option<String>) -> ConstraintDef {
    ConstraintDef { name, expression, is_assertion: true, doc, ..Default::default() }
}

/// Extracts `range`, `length`, `pattern`, `fraction-digits`, and `enum` constraints from a YANG `type` substatement. Realises: [REQ-0015], [REQ-0201]
fn extract_type_constraints(t_stmt: &YangStmt, prefix: &str, target: &str, is_typedef: bool) -> Vec<ConstraintDef> {
    let mut constraints = Vec::new();
    if let Some(r) = t_stmt.get_arg("range") {
        constraints.push(make_constraint(format!("{prefix}_range"), format!("{target} in {r}"), Some(format!("Range constraint: {r}"))));
    }
    if let Some(l) = t_stmt.get_arg("length") {
        constraints.push(make_constraint(format!("{prefix}_length"), format!("length({target}) in {l}"), Some(format!("Length constraint: {l}"))));
    }
    for (idx, p) in t_stmt.get_all_args("pattern").into_iter().enumerate() {
        let clean = p.replace('"', "\\\"");
        constraints.push(make_constraint(format!("{prefix}_pattern_{}", idx + 1), format!("matches({target}, \"{clean}\")"), Some(format!("Pattern constraint: {p}"))));
    }
    if is_typedef {
        if let Some(fd) = t_stmt.get_arg("fraction-digits") {
            constraints.push(make_constraint(format!("{prefix}_fraction_digits"), format!("fraction_digits <= {fd}"), Some(format!("Fraction digits bound: {fd}"))));
        }
        let enums: Vec<String> = t_stmt.find_all("enum").iter().filter_map(|e| e.argument.as_deref()).map(|n| format!("{target} == \"{n}\"")).collect();
        if !enums.is_empty() {
            constraints.push(make_constraint(format!("{prefix}_enum_values"), enums.join(" || "), Some("Enumeration value constraints".to_string())));
        }
    }
    constraints
}

/// Lowers a single YANG `leaf` statement into a SysML `AttributeDef`. Realises: [REQ-0015], [REQ-0201]
fn translate_leaf_attr(leaf: &YangStmt, typedef_map: &BTreeMap<String, String>) -> Option<AttributeDef> {
    let l_name = leaf.argument.as_deref()?;
    let raw_type = leaf.find_first("type").and_then(|t| t.argument.as_deref()).unwrap_or("string");
    let mapped_type = map_yang_type(raw_type, typedef_map);
    let default_value = leaf.get_arg("default").map(|d| format_yang_default_value(d, &mapped_type));
    Some(AttributeDef { name: to_camel_case(l_name), type_name: mapped_type, default_value, doc: leaf.get_arg("description").map(str::to_string) })
}

/// Lowers a YANG `typedef` statement into a package `AttributeDef` alias and constrained `PartDef`. Realises: [REQ-0015], [REQ-0201]
fn translate_typedef(stmt: &YangStmt, typedef_map: &BTreeMap<String, String>, pkg: &mut PackageDef) {
    let Some(td_name) = stmt.argument.as_deref() else { return };
    let base_type = typedef_map.get(td_name).cloned().unwrap_or_else(|| "String".to_string());
    let td_doc = stmt.get_arg("description").map(str::to_string);
    let default_val = stmt.get_arg("default").map(|d| format_yang_default_value(d, &base_type));
    pkg.attribute_defs.push(AttributeDef { name: to_camel_case(td_name), type_name: base_type.clone(), default_value: default_val.clone(), doc: td_doc.clone() });
    let constraints = stmt.find_first("type").map(|t| extract_type_constraints(t, &to_snake_case(td_name), "value", true)).unwrap_or_default();
    let value_attr = AttributeDef { name: "value".to_string(), type_name: base_type, default_value: default_val, doc: None };
    pkg.part_defs.push(PartDef { name: to_pascal_case(td_name), doc: td_doc, attributes: vec![value_attr], constraints, is_def: true, ..Default::default() });
}

/// Lowers a YANG `rpc` or `action` statement into a SysML `ActionDef`. Realises: [REQ-0015], [REQ-0201]
fn translate_rpc(stmt: &YangStmt, typedef_map: &BTreeMap<String, String>) -> Option<ActionDef> {
    let rpc_name = stmt.argument.as_deref()?;
    let in_params = stmt.find_first("input").map(|i| i.find_all("leaf").into_iter().filter_map(|l| translate_leaf_attr(l, typedef_map)).collect()).unwrap_or_default();
    let out_params = stmt.find_first("output").map(|o| o.find_all("leaf").into_iter().filter_map(|l| translate_leaf_attr(l, typedef_map)).collect()).unwrap_or_default();
    Some(ActionDef { name: to_pascal_case(rpc_name), doc: stmt.get_arg("description").map(str::to_string), in_params, out_params, is_def: true, ..Default::default() })
}

/// Deduplicates `PartDef` records by name while merging attributes, child parts, and constraints. Realises: [REQ-0015], [REQ-0201]
fn deduplicate_part_defs(parts: Vec<PartDef>) -> Vec<PartDef> {
    let mut dedup: Vec<PartDef> = Vec::new();
    for p in parts {
        if let Some(existing) = dedup.iter_mut().find(|e| e.name == p.name) {
            for attr in p.attributes { if !existing.attributes.iter().any(|a| a.name == attr.name) { existing.attributes.push(attr); } }
            for sub in p.parts { if !existing.parts.iter().any(|s| s.name == sub.name) { existing.parts.push(sub); } }
            for c in p.constraints { if !existing.constraints.iter().any(|ec| ec.name == c.name) { existing.constraints.push(c); } }
        } else {
            dedup.push(p);
        }
    }
    dedup
}

/// Deterministic translator converting RFC 6020 / RFC 7950 YANG specifications into SysML v2 ASTs. Realises: [REQ-TRANSLATORS-YANG-YANGTRANSLATOR]
#[derive(Debug, Default, Clone)]
pub struct YangTranslator;

impl YangTranslator {
    /// Creates a fresh instance of the YANG translator. Realises: [REQ-TRANSLATORS-YANG-YANGTRANSLATOR-NEW]
    pub fn new() -> Self { Self }

    /// Translates a single YANG schema file content into a canonical SysML v2 `PackageDef`. Realises: [REQ-TRANSLATORS-YANG-YANGTRANSLATOR-TRANSLATE]
    pub fn translate(&self, content: &str, default_name: &str) -> Result<PackageDef, String> {
        let stmts = parse_yang_statements(content)?;
        let module_stmt = stmts.iter().find(|s| s.keyword == "module" || s.keyword == "submodule");
        let mod_name = module_stmt.and_then(|s| s.argument.as_deref()).unwrap_or(default_name);
        let mod_subs: &[YangStmt] = module_stmt.map_or(&stmts, |m| &m.substatements);
        let mut pkg = PackageDef { name: to_pascal_case(mod_name), doc: module_stmt.and_then(|s| s.get_arg("description")).map(str::to_string), ..Default::default() };
        let mut typedef_map: BTreeMap<String, String> = BTreeMap::new();
        for stmt in mod_subs {
            if let ("typedef", Some(td_name)) = (stmt.keyword.as_str(), stmt.argument.as_deref()) {
                let base_type = stmt.find_first("type").and_then(|t| t.argument.as_deref()).unwrap_or("string");
                typedef_map.insert(td_name.to_string(), map_yang_type(base_type, &typedef_map));
            }
        }
        let mut extra_parts = Vec::new();
        for stmt in mod_subs {
            match stmt.keyword.as_str() {
                "import" => if let Some(target) = stmt.argument.as_deref() {
                    pkg.imports.push(ImportDef { path: to_pascal_case(target), is_wildcard: true, is_recursive: false, doc: stmt.get_arg("reference").map(str::to_string) });
                },
                "typedef" => translate_typedef(stmt, &typedef_map, &mut pkg),
                "feature" => if let Some(feat_name) = stmt.argument.as_deref() {
                    pkg.part_defs.push(PartDef { name: to_pascal_case(feat_name), doc: stmt.get_arg("description").map(str::to_string), is_def: true, ..Default::default() });
                },
                "grouping" | "container" => pkg.part_defs.push(translate_container_or_grouping(stmt, &typedef_map, &mut extra_parts)),
                "rpc" | "action" => if let Some(act) = translate_rpc(stmt, &typedef_map) { pkg.action_defs.push(act); },
                _ => {}
            }
        }
        pkg.part_defs.extend(extra_parts);
        pkg.action_defs.push(ActionDef { name: format!("Validate{}", pkg.name), doc: Some(format!("Validates {} data contracts and invariants", pkg.name)), is_def: true, ..Default::default() });
        let uc_doc = format!("Subject: {}\nMain Success Scenario:\n1. Initialize {}\n2. Validate and enforce operational constraints", pkg.name, pkg.name);
        pkg.use_case_defs.push(UseCaseDef { name: format!("Manage{}", pkg.name), subject: Some(pkg.name.clone()), doc: Some(uc_doc), ..Default::default() });
        pkg.part_defs = deduplicate_part_defs(pkg.part_defs);
        Ok(pkg)
    }

    /// Batch-translates multiple YANG schema files into a unified multi-package `PackageDef`. Realises: [REQ-TRANSLATORS-YANG-YANGTRANSLATOR-TRANSLATE-FILES]
    pub fn translate_files(&self, file_paths: &[String], default_name: &str) -> Result<PackageDef, String> {
        if file_paths.is_empty() { return Err("Zero valid YANG schema files were loaded".to_string()); }
        let mut sub_packages = Vec::new();
        for fpath in file_paths {
            let path = Path::new(fpath);
            if !path.exists() { return Err(format!("YANG schema file does not exist: '{}'", path.display())); }
            let content = fs::read_to_string(path).map_err(|e| format!("Failed to read schema file '{fpath}': {e}"))?;
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("YangModule");
            sub_packages.push((stem.to_string(), self.translate(&content, stem)?));
        }
        if sub_packages.len() == 1 { return Ok(sub_packages.remove(0).1); }
        let mut name_counts: BTreeMap<String, usize> = BTreeMap::new();
        for (_, pkg) in &sub_packages { *name_counts.entry(pkg.name.clone()).or_insert(0) += 1; }
        let mut finalized_packages = Vec::new();
        for (stem, mut pkg) in sub_packages {
            if name_counts.get(&pkg.name).copied().unwrap_or(0) > 1 {
                pkg.name = stem.split_once('@').map_or_else(|| to_pascal_case(&stem), |(_, rev)| format!("{}_{}", pkg.name, to_pascal_case(rev)));
            }
            finalized_packages.push(pkg);
        }
        let root_name = if matches!(default_name, "schema" | "Model" | "") { "SysML_Model".to_string() } else { to_pascal_case(default_name) };
        Ok(PackageDef { name: root_name, doc: Some("Consolidated YANG model suite".to_string()), packages: finalized_packages, ..Default::default() })
    }
}

/// Lowers a YANG `choice` statement into case `PartDef`s, direct `AttributeDef`s, and a mutex `ConstraintDef`. Realises: [REQ-0015], [REQ-0201]
fn translate_choice(choice: &YangStmt, typedef_map: &BTreeMap<String, String>, attributes: &mut Vec<AttributeDef>, parts: &mut Vec<PartDef>, extra_parts: &mut Vec<PartDef>) -> Option<ConstraintDef> {
    let choice_name = choice.argument.as_deref()?;
    let mut case_names = Vec::new();
    for case_stmt in choice.find_all("case") {
        if let Some(cs_name) = case_stmt.argument.as_deref() {
            case_names.push(to_camel_case(cs_name));
            let case_part = translate_container_or_grouping(case_stmt, typedef_map, extra_parts);
            parts.push(PartDef { name: to_camel_case(cs_name), type_name: Some(case_part.name.clone()), is_def: false, ..Default::default() });
            extra_parts.push(case_part);
        }
    }
    for direct_leaf in choice.find_all("leaf") {
        if let Some(attr) = translate_leaf_attr(direct_leaf, typedef_map) {
            case_names.push(attr.name.clone());
            attributes.push(attr);
        }
    }
    (!case_names.is_empty()).then(|| make_constraint(format!("{}_mutex", to_snake_case(choice_name)), format!("xor({})", case_names.join(", ")), Some(format!("Choice {choice_name} mutual exclusion invariant"))))
}

/// Recursively translates a YANG container or grouping statement into a structural `PartDef`. Realises: [REQ-TRANSLATORS-YANG-TRANSLATE-CONTAINER-OR-GROUPING]
fn translate_container_or_grouping(stmt: &YangStmt, typedef_map: &BTreeMap<String, String>, extra_parts: &mut Vec<PartDef>) -> PartDef {
    let name = stmt.argument.as_deref().unwrap_or("Anonymous");
    let (mut attributes, mut parts, mut constraints) = (Vec::new(), Vec::new(), Vec::new());
    for leaf in stmt.find_all("leaf") {
        if let Some(attr) = translate_leaf_attr(leaf, typedef_map) {
            let l_name = leaf.argument.as_deref().unwrap_or("leaf");
            if let Some(t_stmt) = leaf.find_first("type") {
                constraints.extend(extract_type_constraints(t_stmt, &format!("{}_{}", to_snake_case(name), to_snake_case(l_name)), &attr.name, false));
            }
            attributes.push(attr);
        }
    }
    for llist in stmt.find_all("leaf-list") {
        if let Some(ll_name) = llist.argument.as_deref() {
            let ll_type = llist.find_first("type").and_then(|t| t.argument.as_deref()).unwrap_or("string");
            attributes.push(AttributeDef { name: to_camel_case(ll_name), type_name: format!("{}[]", map_yang_type(ll_type, typedef_map)), default_value: None, doc: llist.get_arg("description").map(str::to_string) });
        }
    }
    for child in &stmt.substatements {
        if let ("container" | "list", Some(c_name)) = (child.keyword.as_str(), child.argument.as_deref()) {
            let child_part = translate_container_or_grouping(child, typedef_map, extra_parts);
            parts.push(PartDef { name: to_camel_case(c_name), type_name: Some(child_part.name.clone()), is_def: false, ..Default::default() });
            extra_parts.push(child_part);
        }
    }
    for choice in stmt.find_all("choice") {
        if let Some(mutex) = translate_choice(choice, typedef_map, &mut attributes, &mut parts, extra_parts) { constraints.push(mutex); }
    }
    for (idx, must_stmt) in stmt.find_all("must").into_iter().enumerate() {
        if let Some(expr) = must_stmt.argument.as_deref() {
            constraints.push(make_constraint(format!("{}_must_{}", to_snake_case(name), idx + 1), expr.to_string(), must_stmt.get_arg("description").map(str::to_string)));
        }
    }
    PartDef { name: to_pascal_case(name), doc: stmt.get_arg("description").map(str::to_string), attributes, parts, constraints, is_def: true, ..Default::default() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yang_scanner_and_statement_parsing() -> Result<(), String> {
        let yang_src = "module test-module { yang-version 1.1; namespace \"urn:ietf:params:xml:ns:yang:test-module\"; prefix tm; description \"Test module description \" + \"with concatenation.\"; typedef test-type { type string { length \"1..255\"; pattern '[a-z]+'; } } container test-container { leaf id { type uint32; } choice opt { case a { leaf val-a { type string; } } case b { leaf val-b { type boolean; } } } } }";
        let pkg = YangTranslator::new().translate(yang_src, "TestModule")?;
        assert_eq!(pkg.name, "TestModule");
        assert!(pkg.doc.as_deref().is_some_and(|d| d.contains("Test module description with concatenation.")));
        assert!(pkg.attribute_defs.iter().any(|a| a.name == "testType") && pkg.part_defs.iter().any(|p| p.name == "TestType"));
        let container = pkg.part_defs.iter().find(|p| p.name == "TestContainer").ok_or("missing TestContainer")?;
        assert!(container.attributes.iter().any(|a| a.name == "id"));
        assert!(container.parts.iter().any(|p| p.name == "a") && container.parts.iter().any(|p| p.name == "b"));
        assert!(container.constraints.iter().any(|c| c.name == "opt_mutex"));
        Ok(())
    }

    const ROUND_TRIP_YANG: &str = "module sample-module { yang-version 1.1; namespace \"urn:example:sample-module\"; prefix sm; description \"Neutral sample module for round-trip verification.\"; typedef time-text { type string; default \"00:00:00\"; description \"Clock text value.\"; } typedef ratio-value { type decimal64 { fraction-digits 2; } default \"0.5\"; } container sample-root { description \"Root container.\"; leaf start-time { type time-text; default \"00:00:00\"; } leaf family { type string; default \"ipv4\"; } leaf retries { type uint8; default \"3\"; } leaf enabled { type boolean; default \"true\"; } leaf quoted-text { type string; default \"say \\\"hi\\\" \\\\ bye\"; } leaf-list tags { type string; } leaf-list counts { type uint32; } list entry { key \"label\"; leaf label { type string; } leaf weight { type decimal64 { fraction-digits 1; } default \"1.5\"; } } choice mode { case first-mode { leaf first-value { type string; default \"alpha\"; } } case second-mode { leaf second-value { type uint16; } } } } rpc reset-item { description \"Resets an item.\"; input { leaf target { type string; default \"all\"; } leaf delay { type uint32; default \"10\"; } } output { leaf outcome { type string; default \"ok\"; } leaf done { type boolean; default \"false\"; } } } }";

    #[test]
    fn test_format_yang_default_value_literal_rules() {
        for (raw, ty, expected) in [
            ("42", "Integer", "42"), ("-7", "Integer", "-7"), ("0.25", "Real", "0.25"), ("true", "Boolean", "true"), ("false", "Boolean", "false"),
            ("00:00:00", "String", "\"00:00:00\""), ("ipv4", "String", "\"ipv4\""), ("42", "String", "\"42\""), ("unbounded", "Integer", "\"unbounded\""),
            ("TRUE", "Boolean", "\"TRUE\""), ("true", "String", "\"true\""), ("", "String", "\"\""), ("a\"b\\c", "String", "\"a\\\"b\\\\c\""), ("3", "TimeText", "\"3\""),
        ] {
            assert_eq!(format_yang_default_value(raw, ty), expected);
        }
    }

    #[test]
    fn test_yang_string_defaults_are_quoted_sysml_literals() -> Result<(), String> {
        let pkg = YangTranslator::new().translate(ROUND_TRIP_YANG, "SampleModule")?;
        assert_eq!(pkg.attribute_defs.iter().find(|a| a.name == "timeText").and_then(|a| a.default_value.as_deref()), Some("\"00:00:00\""));
        assert_eq!(pkg.attribute_defs.iter().find(|a| a.name == "ratioValue").and_then(|a| a.default_value.as_deref()), Some("0.5"));
        let td_value = pkg.part_defs.iter().find(|p| p.name == "TimeText").and_then(|p| p.attributes.iter().find(|a| a.name == "value"));
        assert_eq!(td_value.and_then(|a| a.default_value.as_deref()), Some("\"00:00:00\""));
        let root = pkg.part_defs.iter().find(|p| p.name == "SampleRoot").ok_or("missing SampleRoot")?;
        let default_of = |name: &str| root.attributes.iter().find(|a| a.name == name).and_then(|a| a.default_value.clone());
        for (k, v) in [("startTime", Some("\"00:00:00\"")), ("family", Some("\"ipv4\"")), ("retries", Some("3")), ("enabled", Some("true")), ("quotedText", Some("\"say \\\"hi\\\" \\\\ bye\"")), ("tags", None)] {
            assert_eq!(default_of(k).as_deref(), v);
        }
        assert_eq!(root.attributes.iter().find(|a| a.name == "tags").map(|a| a.type_name.as_str()), Some("String[]"));
        let first_mode = pkg.part_defs.iter().find(|p| p.name == "FirstMode").ok_or("missing FirstMode")?;
        assert_eq!(first_mode.attributes.iter().find(|a| a.name == "firstValue").and_then(|a| a.default_value.as_deref()), Some("\"alpha\""));
        let rpc = pkg.action_defs.iter().find(|a| a.name == "ResetItem").ok_or("missing ResetItem")?;
        let in_default = |name: &str| rpc.in_params.iter().find(|a| a.name == name).and_then(|a| a.default_value.clone());
        let out_default = |name: &str| rpc.out_params.iter().find(|a| a.name == name).and_then(|a| a.default_value.clone());
        assert_eq!(in_default("target").as_deref(), Some("\"all\""));
        assert_eq!(in_default("delay").as_deref(), Some("10"));
        assert_eq!(out_default("outcome").as_deref(), Some("\"ok\""));
        assert_eq!(out_default("done").as_deref(), Some("false"));
        Ok(())
    }

    #[test]
    fn test_yang_to_sysml_round_trip_parses_and_validates_with_zero_errors() -> Result<(), String> {
        let pkg = YangTranslator::new().translate(ROUND_TRIP_YANG, "SampleModule")?;
        let sysml_text = compile_sysml::to_sysml(&pkg);
        assert!(sysml_text.contains("String[]") && sysml_text.contains("\"00:00:00\""));
        let reparsed = compile_sysml::parse_sysml(&sysml_text).map_err(|e| format!("round-trip parse failed: {e:?}\n{sysml_text}"))?;
        let mut validator = compile_sysml::SemanticValidator::new(&reparsed);
        let error_count = validator.validate().iter().filter(|d| d.severity == compile_sysml::DiagnosticSeverity::Error).count();
        assert_eq!(error_count, 0, "round-trip semantic errors: {:?}\n{sysml_text}", validator.errors());
        let root = reparsed.part_defs.iter().chain(reparsed.packages.iter().flat_map(|p| p.part_defs.iter())).find(|p| p.name == "SampleRoot").ok_or("SampleRoot survives round trip")?;
        assert_eq!(root.attributes.iter().find(|a| a.name == "tags").map(|a| a.type_name.as_str()), Some("String[]"));
        assert_eq!(root.attributes.iter().find(|a| a.name == "startTime").and_then(|a| a.default_value.as_deref()), Some("\"00:00:00\""));
        Ok(())
    }
}
