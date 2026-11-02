//! Source-code AST translator lowering Rust and Dart source trees into canonical SysML v2 `PackageDef` models.
//!
//! Safety & Traceability: Realises: [REQ-FEAT-42-CODE-TRANSLATOR], [DO-330-TQL-4-CODE-TRANSLATOR]
//! Contract: Lowers Rust and Dart source trees into canonical SysML v2 `PackageDef` AST representations.
//! Bounds & Faults: O(N log N) time and O(N) space over source bytes; returns `Err(String)` on syntax or I/O faults.

use deap_core::dart_cst::parse_dart_cst;
use deap_core::rust_cst::{list_rust_items, ParsedRustFile};
use deap_core::source_ast::{
    classify_dart_top_level_ast, classify_rust_item_line_ast, ingest_source_directory,
    lower_source_ast_to_package, parse_dart_source_ast, parse_rust_source_ast, SourceAstItem,
    SourceLanguage,
};
use deap_core::sysml_ast::PackageDef;
use std::fs;
use std::path::{Path, PathBuf};

/// Infers whether an extensionless source buffer is Dart or Rust using `dart_cst` and `rust_cst`.
/// Realises: [REQ-FEAT-42-CODE-LANG-DETECT], [DO-330-TQL-4-CODE-TRANSLATOR]
fn detect_language_from_content(content: &str) -> SourceLanguage {
    let dart_cst_items = parse_dart_cst(content);
    let rust_parsed = ParsedRustFile::parse_rust_source(content);
    let rust_cst_items = list_rust_items(&rust_parsed);
    let mut dart_score: usize = dart_cst_items.len();
    let mut rust_score: usize = rust_cst_items.len();

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty()
            || line.starts_with("//")
            || line.starts_with("/*")
            || line.starts_with('*')
        {
            continue;
        }
        if line.starts_with("import '")
            || line.starts_with("import \"")
            || line.starts_with("export '")
            || line.starts_with("export \"")
            || line.starts_with("part '")
            || line.starts_with("part \"")
            || line.starts_with("part of ")
            || line.starts_with("library ")
            || line.starts_with("final ")
            || line.starts_with("late ")
        {
            dart_score = dart_score.saturating_add(2);
        }
        if line.starts_with("use ")
            || line.starts_with("pub use ")
            || line.starts_with("mod ")
            || line.starts_with("pub mod ")
            || line.starts_with("impl ")
            || line.starts_with("impl<")
            || line.starts_with("#[")
            || line.starts_with("#![")
        {
            rust_score = rust_score.saturating_add(2);
        }
        if classify_rust_item_line_ast(line).is_some() {
            rust_score = rust_score.saturating_add(2);
        } else if classify_dart_top_level_ast(line).is_some() {
            dart_score = dart_score.saturating_add(2);
        }
    }

    if dart_score > rust_score {
        SourceLanguage::Dart
    } else {
        SourceLanguage::Rust
    }
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
    fn default() -> Self {
        Self::new()
    }
}

impl CodeTranslator {
    /// Creates a new `CodeTranslator` in `SourceLanguage::Auto` mode.
    /// Realises: [REQ-FEAT-42-CODE-TRANSLATOR-NEW]
    pub const fn new() -> Self {
        Self {
            language: SourceLanguage::Auto,
        }
    }

    /// Creates a `CodeTranslator` bound to an explicit `SourceLanguage` (`Auto`, `Rust`, or `Dart`).
    /// Realises: [REQ-FEAT-42-CODE-TRANSLATOR-WITH-LANG]
    pub const fn with_language(language: SourceLanguage) -> Self {
        Self { language }
    }

    /// Translates a single in-memory source string (`content`) into a canonical SysML v2 `PackageDef`.
    /// Realises: [REQ-FEAT-42-CODE-TRANSLATE-BUFFER]
    pub fn translate(&self, content: &str, file_name: &str) -> Result<PackageDef, String> {
        let resolved_lang = match self.language {
            SourceLanguage::Auto => SourceLanguage::detect_from_extension(Path::new(file_name))
                .unwrap_or_else(|| detect_language_from_content(content)),
            explicit => explicit,
        };

        let items = match resolved_lang {
            SourceLanguage::Rust | SourceLanguage::Auto => {
                parse_rust_source_ast(content, file_name).map_err(String::from)?
            }
            SourceLanguage::Dart => {
                parse_dart_source_ast(content, file_name).map_err(String::from)?
            }
        };

        let pkg_name = Path::new(file_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(file_name);

        Ok(lower_source_ast_to_package(&items, pkg_name))
    }

    /// Translates an explicit slice of `.rs` and/or `.dart` file paths into a consolidated `PackageDef`.
    /// Realises: [REQ-FEAT-42-CODE-TRANSLATE-FILES]
    pub fn translate_files(
        &self,
        file_paths: &[PathBuf],
        package_name: &str,
    ) -> Result<PackageDef, String> {
        if file_paths.is_empty() {
            return Err("No source files provided to CodeTranslator::translate_files".to_string());
        }
        let mut all_items: Vec<SourceAstItem> = Vec::new();
        for path in file_paths {
            let content = fs::read_to_string(path)
                .map_err(|e| format!("Failed to read source file '{}': {}", path.display(), e))?;
            let label = path.display().to_string();
            let file_lang = match self.language {
                SourceLanguage::Auto => SourceLanguage::detect_from_extension(path)
                    .unwrap_or_else(|| detect_language_from_content(&content)),
                explicit => explicit,
            };
            let mut items = match file_lang {
                SourceLanguage::Rust | SourceLanguage::Auto => {
                    parse_rust_source_ast(&content, &label).map_err(String::from)?
                }
                SourceLanguage::Dart => {
                    parse_dart_source_ast(&content, &label).map_err(String::from)?
                }
            };
            all_items.append(&mut items);
        }
        Ok(lower_source_ast_to_package(&all_items, package_name))
    }

    /// Recursively discovers and translates all eligible source files under `source_dir` into a `PackageDef`.
    /// Realises: [REQ-FEAT-42-CODE-TRANSLATE-DIR]
    pub fn translate_directory(
        &self,
        source_dir: &Path,
        package_name: &str,
    ) -> Result<PackageDef, String> {
        ingest_source_directory(source_dir, self.language, package_name).map_err(String::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate_auto_detects_extensionless_non_flutter_dart_buffer() -> Result<(), String> {
        let dart_src = "import 'dart:async';\n\nclass SampleService {\n  final int count;\n  SampleService(this.count);\n}\n";
        let translator = CodeTranslator::new();
        let pkg = translator.translate(dart_src, "sample_service")?;
        if pkg.part_defs.iter().any(|p| p.name == "SampleService") {
            Ok(())
        } else {
            Err(format!(
                "Expected PartDef 'SampleService' from extensionless Dart buffer, got part_defs: {:?}",
                pkg.part_defs.iter().map(|p| &p.name).collect::<Vec<_>>()
            ))
        }
    }

    #[test]
    fn translate_auto_detects_extensionless_rust_buffer() -> Result<(), String> {
        let rust_src = "pub struct SampleRecord {\n    pub count: usize,\n}\n";
        let translator = CodeTranslator::new();
        let pkg = translator.translate(rust_src, "sample_record")?;
        if pkg.part_defs.iter().any(|p| p.name == "SampleRecord") {
            Ok(())
        } else {
            Err(format!(
                "Expected PartDef 'SampleRecord' from extensionless Rust buffer, got part_defs: {:?}",
                pkg.part_defs.iter().map(|p| &p.name).collect::<Vec<_>>()
            ))
        }
    }

    #[test]
    fn translate_files_auto_detects_extensionless_non_flutter_dart_file() -> Result<(), String> {
        let mut temp_dir = std::env::temp_dir();
        temp_dir.push(format!("deap_code_translator_test_{}", std::process::id()));
        fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;
        let file_path = temp_dir.join("sample_service_unit");
        let dart_src = "import 'dart:convert';\n\nclass SampleWorker {\n  final String label;\n  SampleWorker(this.label);\n}\n";
        fs::write(&file_path, dart_src).map_err(|e| e.to_string())?;

        let translator = CodeTranslator::new();
        let result = translator.translate_files(&[file_path], "sample_pkg");
        let _ = fs::remove_dir_all(&temp_dir);

        let pkg = result?;
        if pkg.part_defs.iter().any(|p| p.name == "SampleWorker") {
            Ok(())
        } else {
            Err(format!(
                "Expected PartDef 'SampleWorker' from extensionless Dart file, got part_defs: {:?}",
                pkg.part_defs.iter().map(|p| &p.name).collect::<Vec<_>>()
            ))
        }
    }
}
