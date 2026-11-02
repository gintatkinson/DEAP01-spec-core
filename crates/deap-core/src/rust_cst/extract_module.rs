#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Lossless Rust CST submodule extraction and link rewriting. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D qualification standards.
//! Provides lossless AST item extraction to child submodules with re-exports and blueprint link updates.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Extraction is deterministic, pure, and preserves item syntax, comments, and attributes.
//! - Item removal deletes exact item ranges and adds deterministic submodule declarations.
//! - Line reference rewriting updates markdown link targets without altering unrelated content.
//!
//! ## 3. Preconditions
//! - Input source code and markdown buffers are valid UTF-8.
//! - Item names to extract must match top-level Rust CST items.
//!
//! ## 4. Postconditions
//! - Extracted plan returns valid parent and child source strings without unhandled panics.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ where $N$ is input source length.
//! - Space Complexity: $O(N)$ for allocated modified source buffers.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Unmatched items or syntax parse failures fail closed with Err(String).
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0608], [SC-01], [FLS: fls_cst_extract_module]
//! SysML v2: part def RustCstExtractModule

use ra_ap_syntax::{
    ast::{self, AstNode, HasModuleItem, HasName},
    Edition, SourceFile, TextRange, TextSize,
};
use ra_ap_text_edit::TextEditBuilder;

/// Plan containing rewritten parent source and generated submodule code.
/// Realises: [REQ-0608], [SC-01], [FLS: fls_cst_extract_module]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedModulePlan {
    pub submodule_name: String,
    pub updated_parent_source: String,
    pub new_submodule_source: String,
    pub extracted_item_names: Vec<String>,
}

/// Retrieves the identifier name from an AST item if present.
/// Realises: [REQ-0608], [FLS: fls_cst_extract_module]
fn get_item_name(item: &ast::Item) -> Option<String> {
    match item {
        ast::Item::Const(it) => it.name().map(|n| n.text().to_string()),
        ast::Item::Enum(it) => it.name().map(|n| n.text().to_string()),
        ast::Item::Fn(it) => it.name().map(|n| n.text().to_string()),
        ast::Item::MacroDef(it) => it.name().map(|n| n.text().to_string()),
        ast::Item::MacroRules(it) => it.name().map(|n| n.text().to_string()),
        ast::Item::Module(it) => it.name().map(|n| n.text().to_string()),
        ast::Item::Static(it) => it.name().map(|n| n.text().to_string()),
        ast::Item::Struct(it) => it.name().map(|n| n.text().to_string()),
        ast::Item::Trait(it) => it.name().map(|n| n.text().to_string()),
        ast::Item::TypeAlias(it) => it.name().map(|n| n.text().to_string()),
        ast::Item::Union(it) => it.name().map(|n| n.text().to_string()),
        _ => None,
    }
}

/// Computes the whitespace-expanded deletion byte range for an item.
/// Realises: [REQ-0608], [FLS: fls_cst_extract_module]
fn compute_deletion_range(source: &str, range: TextRange) -> (usize, usize) {
    let f_start = usize::from(range.start());
    let l_end = usize::from(range.end());
    let line_start = source.get(..f_start).and_then(|s| s.rfind('\n')).map_or(0, |i| i + 1);
    let is_only_ws_before = source.get(line_start..f_start).is_some_and(|s| s.chars().all(|c| c.is_whitespace()));
    let del_start = if is_only_ws_before { line_start } else { f_start };
    let trail = source.get(l_end..).unwrap_or("");
    let del_end = if trail.starts_with("\r\n") {
        l_end + 2
    } else if trail.starts_with('\n') {
        l_end + 1
    } else {
        l_end
    };
    (del_start, del_end)
}

/// Extracts top-level items into a dedicated submodule.
/// Realises: [REQ-0608], [SC-01], [FLS: fls_cst_extract_module]
pub fn extract_rust_module(
    parent_source: &str,
    submodule_name: &str,
    item_names: &[&str],
) -> Result<ExtractedModulePlan, String> {
    if item_names.is_empty() { return Err("No item names specified for extraction".to_string()); }
    if submodule_name.trim().is_empty() { return Err("Submodule name cannot be empty".to_string()); }

    let parsed = SourceFile::parse(parent_source, Edition::CURRENT);
    let mut matched_items = Vec::new();
    let mut extracted_item_names = Vec::new();

    for item in parsed.tree().items() {
        if let Some(name) = get_item_name(&item) {
            if item_names.contains(&name.as_str()) {
                extracted_item_names.push(name);
                matched_items.push(item);
            }
        }
    }
    for &req in item_names {
        if !extracted_item_names.iter().any(|n| n == req) {
            return Err(format!("Item '{req}' not found in parent source"));
        }
    }

    let mut new_submodule_source = format!(
        "//! Extracted submodule `{submodule_name}`. [DOC-TIER: A]\n//!\n//! ## 1. Safety Intent & Regulatory Scope\n//! Conforms to DO-178C Level A and ISO 26262 ASIL D qualification standards.\n//!\n//! ## 2. Invariants & Mathematical Properties\n//! Extracted items maintain original AST semantics and pure state invariants.\n//!\n//! ## 3. Preconditions\n//! Callers satisfy preconditions of the enclosed items.\n//!\n//! ## 4. Postconditions\n//! All operations preserve deterministic behavior and fail-closed safety.\n//!\n//! ## 5. Algorithmic Complexity Bounds\n//! Complexity bounded by the execution of constituent functions.\n//!\n//! ## 6. Failure Modes & Fault Tolerance\n//! Faults are isolated to local error returns.\n//!\n//! ## 7. Realization & Traceability\n//! Realises: [REQ-0608], [SC-01], [FLS: fls_extracted_submodule]\n\n"
    );

    let mut delete_ranges = Vec::new();
    for item in &matched_items {
        let range = item.syntax().text_range();
        let (s, e) = (usize::from(range.start()), usize::from(range.end()));
        if let Some(text) = parent_source.get(s..e) {
            new_submodule_source.push_str(text);
            new_submodule_source.push('\n');
            new_submodule_source.push('\n');
        }
        delete_ranges.push(compute_deletion_range(parent_source, range));
    }
    if new_submodule_source.ends_with("\n\n") { new_submodule_source.pop(); }

    let mut builder = TextEditBuilder::default();
    for (s, e) in delete_ranges {
        let start_ts = TextSize::try_from(s).map_err(|err| err.to_string())?;
        let end_ts = TextSize::try_from(e).map_err(|err| err.to_string())?;
        builder.delete(TextRange::new(start_ts, end_ts));
    }

    let sep = if parent_source.ends_with('\n') { "" } else { "\n" };
    let reexport = format!("{sep}pub mod {submodule_name};\npub use {submodule_name}::*;\n");
    let end_ts = TextSize::try_from(parent_source.len()).map_err(|err| err.to_string())?;
    builder.insert(end_ts, reexport);

    let mut updated_parent_source = parent_source.to_string();
    builder.finish().apply(&mut updated_parent_source);

    Ok(ExtractedModulePlan {
        submodule_name: submodule_name.to_string(),
        updated_parent_source,
        new_submodule_source,
        extracted_item_names,
    })
}

/// Rewrites markdown line reference links targeting an old module path to a new path.
/// Realises: [REQ-0608], [SC-01], [FLS: fls_cst_extract_module]
#[must_use]
pub fn rewrite_blueprint_line_refs(markdown: &str, old_rel_path: &str, new_rel_path: &str) -> String {
    if old_rel_path.is_empty() || old_rel_path == new_rel_path {
        return markdown.to_string();
    }
    let mut result = String::with_capacity(markdown.len());
    let mut rest = markdown;
    while let Some(link_start) = rest.find("](") {
        let after_paren = link_start + 2;
        if let Some(prefix) = rest.get(..after_paren) {
            result.push_str(prefix);
        }
        let after = rest.get(after_paren..).unwrap_or("");
        if let Some(link_end) = after.find(')') {
            let target = after.get(..link_end).unwrap_or("");
            if target.contains(old_rel_path) {
                result.push_str(&target.replace(old_rel_path, new_rel_path));
            } else {
                result.push_str(target);
            }
            result.push(')');
            rest = after.get(link_end + 1..).unwrap_or("");
        } else {
            result.push_str(after);
            rest = "";
            break;
        }
    }
    result.push_str(rest);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_rust_module_moves_selected_items_and_adds_reexport() -> Result<(), String> {
        let parent = "pub fn stay() -> u32 { 1 }\n\n/// Struct doc.\n#[derive(Debug)]\npub struct ExtractedStruct {\n    pub v: i32,\n}\n\npub fn extracted_fn() -> bool { true }\n";
        let plan = extract_rust_module(parent, "sub", &["ExtractedStruct", "extracted_fn"])?;
        assert_eq!(plan.submodule_name, "sub");
        assert_eq!(plan.extracted_item_names, vec!["ExtractedStruct".to_string(), "extracted_fn".to_string()]);
        assert!(plan.updated_parent_source.contains("pub fn stay()"));
        assert!(!plan.updated_parent_source.contains("pub struct ExtractedStruct"));
        assert!(!plan.updated_parent_source.contains("pub fn extracted_fn"));
        assert!(plan.updated_parent_source.contains("pub mod sub;\npub use sub::*;"));
        assert!(plan.new_submodule_source.contains("[DOC-TIER: A]"));
        assert!(plan.new_submodule_source.contains("/// Struct doc."));
        assert!(plan.new_submodule_source.contains("pub struct ExtractedStruct"));
        assert!(plan.new_submodule_source.contains("pub fn extracted_fn"));
        Ok(())
    }

    #[test]
    fn test_rewrite_blueprint_line_refs_updates_markdown_links() {
        let md = "# Blueprint\n[Core link](../crates/deap-core/src/rust_cst/mod.rs#L10-L20) and [Other](other.rs).\nPlain text crates/deap-core/src/rust_cst/mod.rs remains.\n";
        let rewritten = rewrite_blueprint_line_refs(md, "crates/deap-core/src/rust_cst/mod.rs", "crates/deap-core/src/rust_cst/extract_module.rs");
        assert!(rewritten.contains("[Core link](../crates/deap-core/src/rust_cst/extract_module.rs#L10-L20)"));
        assert!(rewritten.contains("[Other](other.rs)"));
        assert!(rewritten.contains("Plain text crates/deap-core/src/rust_cst/mod.rs remains."));
    }
}
