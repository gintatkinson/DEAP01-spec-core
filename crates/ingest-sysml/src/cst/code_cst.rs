//! Source-code CST translator lowering Rust and Dart source trees into canonical SysML v2 `PackageDef` models.
//!
//! **[DOC-TIER: A]**
//! Safety & Traceability: Realises: [REQ-FEAT-42-CODE-TRANSLATOR], [DO-330-TQL-4-CODE-TRANSLATOR]
//! Contract: Lowers Rust (`deap_core::rust_cst`) and Dart (`deap_core::dart_cst`) source CSTs into canonical SysML v2 `PackageDef` HIR/AST representations.
//! Bounds & Faults: O(N log N) time and O(N) space over source bytes; returns `Err(String)` on syntax or I/O faults.

use deap_core::dart_cst::parse_dart_items;
use deap_core::rust_cst::{list_rust_items, parse::parse_rust_source};
use deap_core::rust_cst::source_bridge::{
    classify_dart_top_level_ast, classify_rust_item_line_ast, parse_dart_source_ast,
    parse_rust_source_ast, SourceAstItem, SourceItemKind, SourceLanguage,
};
use deap_core::{ActionDef, AttributeDef, ConstraintDef, PackageDef, PartDef, PortDef, RequirementDef};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Sanitizes an arbitrary identifier string into a valid ASCII SysML v2 identifier.
/// Realises: [REQ-FEAT-42-CODE-TRANSLATOR]
fn sanitize_sysml_identifier(input: &str, fallback: &str) -> String {
    let s: String = input.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' }).collect();
    let trimmed = s.trim_matches('_');
    if trimmed.is_empty() { fallback.to_string() } else { trimmed.to_string() }
}

/// Normalizes a source-language primitive or generic wrapper type to a canonical SysML v2 type name.
/// Realises: [REQ-FEAT-42-CODE-TRANSLATOR]
fn normalize_type_for_sysml(ty: &str) -> String {
    let mut s = ty.trim().trim_end_matches('?').trim();
    while let Some(rest) = s.strip_prefix('&') {
        s = rest.trim_start().strip_prefix("mut ").unwrap_or_else(|| rest.trim_start()).trim_start();
    }
    if let Some((outer, rest)) = s.split_once('<') {
        let outer_ident = outer.trim().rsplit_once("::").map_or_else(|| outer.trim(), |(_, l)| l.trim());
        if let Some(inner) = rest.strip_suffix('>').filter(|_| matches!(outer_ident, "Result" | "Option" | "Vec" | "List" | "Set" | "Box" | "Arc" | "Rc" | "Future" | "Stream" | "ValueChanged" | "ValueSetter" | "ValueGetter")) {
            let first = inner.split(',').next().unwrap_or("").trim();
            return if !first.is_empty() && first != "()" && first != "void" { normalize_type_for_sysml(first) } else { "String".to_string() };
        }
    }
    let base = s.split_once('<').map_or(s, |(o, _)| o.trim());
    let last = base.rsplit_once("::").map_or(base, |(_, l)| l.trim());
    match sanitize_sysml_identifier(last, "String").as_str() {
        "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "int" => "Integer".to_string(),
        "f32" | "f64" | "double" | "num" => "Real".to_string(),
        "bool" => "Boolean".to_string(),
        "str" | "String" | "char" | "Path" | "PathBuf" | "Self" | "void" | "dynamic" | "Object" => "String".to_string(),
        other => other.to_string(),
    }
}

/// Extracts the primary prose summary line from collected doc comment lines.
/// Realises: [REQ-FEAT-42-CODE-TRANSLATOR]
fn summary_from_doc_lines(doc_lines: &[String]) -> Option<String> {
    doc_lines.iter().find_map(|line| {
        let t = line.trim_start_matches('/').trim_start_matches('*').trim();
        (!t.is_empty() && !t.starts_with("Realises:") && !t.starts_with('#')).then(|| t.to_string())
    })
}

/// Lowers CST-backed `SourceAstItem` records into a canonical SysML v2 `PackageDef`.
/// Realises: [REQ-FEAT-42-CODE-TRANSLATOR]
fn lower_source_ast_to_package(items: &[SourceAstItem], package_name: &str) -> PackageDef {
    let clean_pkg = sanitize_sysml_identifier(package_name, "SourceCodeModel");
    let (mut pkg, mut seen_reqs) = (PackageDef { name: clean_pkg.clone(), ..Default::default() }, BTreeSet::new());
    for item in items {
        for req_id in &item.satisfied_requirements {
            if seen_reqs.insert(req_id.clone()) {
                pkg.requirement_defs.push(RequirementDef { name: sanitize_sysml_identifier(req_id, "REQ_ITEM"), req_id: req_id.clone(), text: format!("Requirement {req_id} realized by {}", item.name), ..Default::default() });
            }
        }
        if matches!(item.kind, SourceItemKind::Struct | SourceItemKind::Class | SourceItemKind::Type | SourceItemKind::Trait | SourceItemKind::Mixin | SourceItemKind::Extension | SourceItemKind::Typedef) {
            let (mut attributes, mut ports) = (Vec::new(), Vec::new());
            for field in &item.fields {
                let (fname, ftype) = (sanitize_sysml_identifier(&field.name, "field_attr"), normalize_type_for_sysml(&field.type_name));
                if field.is_callback_or_port {
                    ports.push(PortDef { name: fname, type_name: ftype, direction: "inout".to_string(), port_category: "DataPort".to_string(), doc: field.doc.clone(), ..Default::default() });
                } else {
                    attributes.push(AttributeDef { name: fname, type_name: ftype, default_value: None, doc: field.doc.clone() });
                }
            }
            pkg.part_defs.push(PartDef { name: sanitize_sysml_identifier(&item.name, "ComponentPart"), doc: summary_from_doc_lines(&item.doc_lines), is_def: true, attributes, ports, satisfied_requirements: item.satisfied_requirements.clone(), ..Default::default() });
        }
    }
    for item in items.iter().filter(|i| i.kind.is_callable()) {
        let act_name = sanitize_sysml_identifier(&item.name, "action_op");
        let in_params: Vec<AttributeDef> = item.params.iter().filter(|p| !matches!(p.name.trim(), "self" | "&self" | "&mut self" | "mut self")).map(|p| AttributeDef { name: sanitize_sysml_identifier(&p.name, "param"), type_name: normalize_type_for_sysml(&p.type_name), default_value: None, doc: None }).collect();
        let out_params = item.return_type.as_deref().map(str::trim).filter(|rt| !rt.is_empty() && *rt != "()" && *rt != "void").map(|rt| vec![AttributeDef { name: "result".to_string(), type_name: normalize_type_for_sysml(rt), default_value: None, doc: None }]).unwrap_or_default();
        let action = ActionDef { name: act_name.clone(), doc: summary_from_doc_lines(&item.doc_lines), in_params, out_params, performer: item.owner_name.clone(), is_def: true, ..Default::default() };
        if let Some(ref owner_raw) = item.owner_name {
            if let Some(owner_part) = pkg.part_defs.iter_mut().find(|p| p.name == sanitize_sysml_identifier(owner_raw, "ComponentPart")) {
                owner_part.actions.push(action);
                for (idx, expr) in item.invariants.iter().enumerate() {
                    owner_part.constraints.push(ConstraintDef { name: format!("invariant_{act_name}_{}", idx + 1), expression: expr.clone(), is_assertion: true, ..Default::default() });
                }
            }
        } else {
            pkg.action_defs.push(action);
        }
    }
    if pkg.part_defs.is_empty() && !pkg.action_defs.is_empty() {
        pkg.part_defs.push(PartDef { name: format!("{clean_pkg}Module"), is_def: true, actions: pkg.action_defs.clone(), ..Default::default() });
    }
    pkg
}

/// Infers whether an extensionless source buffer is Dart or Rust using `dart_cst` and `rust_cst`.
/// Realises: [REQ-FEAT-42-CODE-LANG-DETECT], [DO-330-TQL-4-CODE-TRANSLATOR]
fn detect_language_from_content(content: &str) -> SourceLanguage {
    let mut dart_score = parse_dart_items(content).into_iter().filter(|i| !i.name.is_empty()).count() * 2;
    let mut rust_score = list_rust_items(&parse_rust_source("<auto>", content)).into_iter().filter(|i| !i.name.is_empty()).count() * 2;
    if let Ok(ast_items) = parse_rust_source_ast(content, "<auto>") {
        rust_score = rust_score.saturating_add(ast_items.iter().filter(|i| !i.name.is_empty()).count());
    }
    if let Ok(ast_items) = parse_dart_source_ast(content, "<auto>") {
        dart_score = dart_score.saturating_add(ast_items.iter().filter(|i| !i.name.is_empty()).count());
    }
    for line in content.lines().map(str::trim) {
        if classify_rust_item_line_ast(line).is_some_and(|(_, n)| !n.is_empty()) {
            rust_score = rust_score.saturating_add(2);
        } else if classify_dart_top_level_ast(line).is_some_and(|(_, n, _)| !n.is_empty()) {
            dart_score = dart_score.saturating_add(2);
        }
    }
    if dart_score > rust_score { SourceLanguage::Dart } else { SourceLanguage::Rust }
}

/// Deterministic translator converting Rust and Dart source code into canonical SysML v2 `PackageDef` models.
/// Realises: [REQ-FEAT-42-CODE-TRANSLATOR-STRUCT]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodeTranslator {
    /// Selected source language parser mode (`Auto`, `Rust`, or `Dart`).
    pub language: SourceLanguage,
}

impl Default for CodeTranslator {
    /// Constructs a default `CodeTranslator` initialized with `SourceLanguage::Auto`.
    /// Realises: [REQ-FEAT-42-CODE-TRANSLATOR-DEFAULT]
    fn default() -> Self { Self::new() }
}

impl CodeTranslator {
    /// Creates a new `CodeTranslator` in `SourceLanguage::Auto` mode.
    /// Realises: [REQ-FEAT-42-CODE-TRANSLATOR-NEW]
    #[must_use]
    pub const fn new() -> Self { Self { language: SourceLanguage::Auto } }

    /// Creates a `CodeTranslator` bound to an explicit `SourceLanguage` (`Auto`, `Rust`, or `Dart`).
    /// Realises: [REQ-FEAT-42-CODE-TRANSLATOR-WITH-LANG]
    #[must_use]
    pub const fn with_language(language: SourceLanguage) -> Self { Self { language } }

    /// Translates a single in-memory source string (`content`) into a canonical SysML v2 `PackageDef`.
    /// Realises: [REQ-FEAT-42-CODE-TRANSLATE-BUFFER]
    pub fn translate(&self, content: &str, file_name: &str) -> Result<PackageDef, String> {
        let resolved_lang = match self.language {
            SourceLanguage::Auto => SourceLanguage::detect_from_extension(Path::new(file_name)).unwrap_or_else(|| detect_language_from_content(content)),
            explicit => explicit,
        };
        let items = match resolved_lang {
            SourceLanguage::Rust | SourceLanguage::Auto => parse_rust_source_ast(content, file_name).map_err(String::from)?,
            SourceLanguage::Dart => parse_dart_source_ast(content, file_name).map_err(String::from)?,
        };
        let pkg_name = Path::new(file_name).file_stem().and_then(|s| s.to_str()).unwrap_or(file_name);
        Ok(lower_source_ast_to_package(&items, pkg_name))
    }

    /// Translates an explicit slice of `.rs` and/or `.dart` file paths into a consolidated `PackageDef`.
    /// Realises: [REQ-FEAT-42-CODE-TRANSLATE-FILES]
    pub fn translate_files(&self, file_paths: &[PathBuf], package_name: &str) -> Result<PackageDef, String> {
        if file_paths.is_empty() { return Err("No source files provided to CodeTranslator::translate_files".to_string()); }
        let mut all_items: Vec<SourceAstItem> = Vec::new();
        for path in file_paths {
            let (content, label) = (fs::read_to_string(path).map_err(|e| format!("Failed to read source file '{}': {}", path.display(), e))?, path.display().to_string());
            let file_lang = match self.language { SourceLanguage::Auto => SourceLanguage::detect_from_extension(path).unwrap_or_else(|| detect_language_from_content(&content)), explicit => explicit };
            let mut items = match file_lang {
                SourceLanguage::Rust | SourceLanguage::Auto => parse_rust_source_ast(&content, &label).map_err(String::from)?,
                SourceLanguage::Dart => parse_dart_source_ast(&content, &label).map_err(String::from)?,
            };
            all_items.append(&mut items);
        }
        Ok(lower_source_ast_to_package(&all_items, package_name))
    }

    /// Recursively discovers and translates all eligible source files under `source_dir` into a `PackageDef`.
    /// Realises: [REQ-FEAT-42-CODE-TRANSLATE-DIR]
    pub fn translate_directory(&self, source_dir: &Path, package_name: &str) -> Result<PackageDef, String> {
        let mut files: Vec<PathBuf> = Vec::new();
        for entry_res in walkdir::WalkDir::new(source_dir) {
            let Ok(e) = entry_res else { continue };
            let eligible = e.path().is_file() && match self.language {
                SourceLanguage::Rust => e.path().extension().is_some_and(|ext| ext == "rs"),
                SourceLanguage::Dart => e.path().extension().is_some_and(|ext| ext == "dart"),
                SourceLanguage::Auto => e.path().extension().is_some_and(|ext| ext == "rs" || ext == "dart"),
            };
            if eligible {
                files.push(e.path().to_path_buf());
            }
        }
        files.sort();
        let leaf = source_dir.file_name().and_then(|n| n.to_str()).filter(|s| !s.is_empty()).unwrap_or("SourceCodeModel");
        let effective = if !package_name.trim().is_empty() && !matches!(package_name.trim(), "src" | "lib") {
            sanitize_sysml_identifier(package_name.trim(), "SourceCodeModel")
        } else if matches!(leaf, "src" | "lib") {
            source_dir.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str()).filter(|s| !s.is_empty() && *s != "." && *s != "..").map_or_else(|| sanitize_sysml_identifier(leaf, "SourceCodeModel"), |parent| sanitize_sysml_identifier(&format!("{parent}_{leaf}_Realization"), "SourceCodeModel"))
        } else {
            sanitize_sysml_identifier(leaf, "SourceCodeModel")
        };
        if files.is_empty() { return Ok(lower_source_ast_to_package(&[], &effective)); }
        self.translate_files(&files, &effective)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate_auto_detects_extensionless_non_flutter_dart_buffer() -> Result<(), String> {
        let dart_src = "import 'dart:async';\n\nclass SampleService {\n  final int count;\n  SampleService(this.count);\n}\n";
        let pkg = CodeTranslator::new().translate(dart_src, "sample_service")?;
        pkg.part_defs.iter().any(|p| p.name == "SampleService").then_some(()).ok_or_else(|| "missing SampleService".to_string())
    }

    #[test]
    fn translate_auto_detects_extensionless_rust_buffer() -> Result<(), String> {
        let rust_src = "pub struct SampleRecord {\n    pub count: usize,\n}\n";
        let pkg = CodeTranslator::new().translate(rust_src, "sample_record")?;
        pkg.part_defs.iter().any(|p| p.name == "SampleRecord").then_some(()).ok_or_else(|| "missing SampleRecord".to_string())
    }

    #[test]
    fn translate_files_auto_detects_extensionless_non_flutter_dart_file() -> Result<(), String> {
        let temp_dir = std::env::temp_dir().join(format!("deap_code_translator_test_{}", std::process::id()));
        fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;
        let file_path = temp_dir.join("sample_service_unit");
        fs::write(&file_path, "import 'dart:convert';\n\nclass SampleWorker {\n  final String label;\n  SampleWorker(this.label);\n}\n").map_err(|e| e.to_string())?;
        let result = CodeTranslator::new().translate_files(&[file_path], "sample_pkg");
        fs::remove_dir_all(&temp_dir).ok();
        result?.part_defs.iter().any(|p| p.name == "SampleWorker").then_some(()).ok_or_else(|| "missing SampleWorker".to_string())
    }
}
