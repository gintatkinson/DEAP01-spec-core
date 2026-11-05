//! Section 6 declaration-only contract validation helpers.

/// Rejects executable Rust items in a Section 6 declaration-only code block, recursing into inline modules.
/// Realises: [REQ-BLUEPRINT-CHECK-9-RUST-CODE-BLOCKS]
pub(crate) fn validate_declaration_only_item(item: &syn::Item, errors: &mut Vec<String>) {
    match item {
        syn::Item::Fn(item_fn) => errors.push(format!(
            "Check 9 violation (Section 6 -- Declaration-Only Contract): function body '{}' is forbidden; Section 6 must contain declarations only",
            item_fn.sig.ident
        )),
        syn::Item::Impl(_) => errors.push(
            "Check 9 violation (Section 6 -- Declaration-Only Contract): 'impl' blocks are forbidden; use bodyless trait declarations".to_string(),
        ),
        syn::Item::Macro(_) => errors.push(
            "Check 9 violation (Section 6 -- Declaration-Only Contract): macro definitions and invocations are forbidden".to_string(),
        ),
        syn::Item::Verbatim(tokens) => errors.push(format!(
            "Check 9 violation (Section 6 -- Declaration-Only Contract): unrecognized/verbatim item is forbidden: '{tokens}'"
        )),
        syn::Item::Trait(item_trait) => {
            for trait_item in &item_trait.items {
                match trait_item {
                    syn::TraitItem::Fn(trait_fn) if trait_fn.default.is_some() => errors.push(format!(
                        "Check 9 violation (Section 6 -- Declaration-Only Contract): trait method '{}' has a default body; Section 6 trait methods must be bodyless declarations",
                        trait_fn.sig.ident
                    )),
                    syn::TraitItem::Const(trait_const) if trait_const.default.is_some() => errors.push(format!(
                        "Check 9 violation (Section 6 -- Declaration-Only Contract): trait constant '{}' has a default value; Section 6 trait constants must be bodyless declarations",
                        trait_const.ident
                    )),
                    syn::TraitItem::Type(trait_type) if trait_type.default.is_some() => errors.push(format!(
                        "Check 9 violation (Section 6 -- Declaration-Only Contract): trait type '{}' has a default value; Section 6 trait types must be bodyless declarations",
                        trait_type.ident
                    )),
                    syn::TraitItem::Macro(_) => errors.push(
                        "Check 9 violation (Section 6 -- Declaration-Only Contract): trait macro items are forbidden".to_string(),
                    ),
                    syn::TraitItem::Verbatim(tokens) => errors.push(format!(
                        "Check 9 violation (Section 6 -- Declaration-Only Contract): unrecognized/verbatim trait item is forbidden: '{tokens}'"
                    )),
                    _ => {}
                }
            }
        }
        syn::Item::Mod(item_mod) => {
            if let Some((_, items)) = &item_mod.content {
                for nested in items {
                    validate_declaration_only_item(nested, errors);
                }
            }
        }
        syn::Item::Use(_)
        | syn::Item::Struct(_)
        | syn::Item::Enum(_)
        | syn::Item::Union(_)
        | syn::Item::Type(_)
        | syn::Item::Const(_)
        | syn::Item::Static(_)
        | syn::Item::ForeignMod(_) => {}
        _ => errors.push(
            "Check 9 violation (Section 6 -- Declaration-Only Contract): item is not permitted; allowed declarations are use/struct/enum/union/type/const/static/trait/external mod/foreign mod".to_string(),
        ),
    }
}
