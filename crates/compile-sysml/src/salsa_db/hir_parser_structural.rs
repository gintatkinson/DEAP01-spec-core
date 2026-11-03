#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Package, import, and container-scope declaration dispatchers for SysML v2 / KerML grammar parsing.
//!
//! Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-MODULE] | [FLS: fls_66m4rnbssgig]
//! Contract: Parses top-level files, nested `package` blocks, `import` clauses, and scoped package/part container member declarations into deterministic `PackageDef` and `PartDef` AST hierarchies.
//! Bounds & Faults: Enforces `MAX_PARSE_DEPTH` (64) across recursive `package` and container scopes, failing closed with span-annotated `ParseError` diagnostics on unrecognized tokens or depth exhaustion.

use super::cst_db::TokenKind;
use super::hir_db::{is_synthetic_forest_root, merge_package_defs};
use super::hir_parser_core::{ParseError, SysmlParser, MAX_PARSE_DEPTH};
use deap_core::sysml_cst::{ImportDef, PackageDef, PartDef};

impl<'a> SysmlParser<'a> {
    /// Parses a complete token stream representing a `package` forest or flat SysML v2 file into a root `PackageDef`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-PACKAGE-OR-FILE] | [FLS: fls_66m4rnbssgig]
    pub fn parse_package_or_file(
        &mut self,
        default_pkg_name: &str,
    ) -> Result<PackageDef, ParseError> {
        self.skip_trivia();
        let mut pending_doc = self.take_doc_comment();
        self.skip_trivia();
        if self.check(&TokenKind::Package) {
            let first_pkg = self.parse_package_decl(pending_doc)?;
            self.skip_trivia();
            if self.is_at_end() {
                return Ok(first_pkg);
            }
            let mut root = PackageDef {
                name: default_pkg_name.to_string(),
                packages: vec![first_pkg],
                ..Default::default()
            };
            while !self.is_at_end() {
                self.parse_container_item(&mut root, None)?;
                self.skip_trivia();
            }
            if root.packages.len() == 1
                && (is_synthetic_forest_root(&root)
                    || root.name == "SysML_Model"
                    || root.name.is_empty())
            {
                if let Some(mut single) = root.packages.pop() {
                    merge_package_defs(&mut single, root);
                    return Ok(single);
                }
            }
            return Ok(root);
        }

        let mut pkg = PackageDef {
            name: default_pkg_name.to_string(),
            doc: pending_doc.take(),
            ..Default::default()
        };
        while !self.is_at_end() {
            self.parse_container_item(&mut pkg, None)?;
            self.skip_trivia();
        }
        if pkg.packages.len() == 1 && is_synthetic_forest_root(&pkg) {
            if let Some(mut single) = pkg.packages.pop() {
                merge_package_defs(&mut single, pkg);
                return Ok(single);
            }
        }
        Ok(pkg)
    }

    /// Parses a `package <Name> { ... }` declaration bounded by `MAX_PARSE_DEPTH`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-PACKAGE-DECL] | [FLS: fls_66m4rnbssgig]
    pub(crate) fn parse_package_decl(
        &mut self,
        doc: Option<String>,
    ) -> Result<PackageDef, ParseError> {
        if self.depth >= MAX_PARSE_DEPTH {
            return Err(self.max_depth_error());
        }
        self.depth = self.depth.saturating_add(1);
        let result = self.parse_package_decl_inner(doc);
        self.depth = self.depth.saturating_sub(1);
        result
    }

    /// Parses the tokens and body of a `package <Name> { ... }` declaration within an active depth frame.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-PACKAGE-DECL] | [FLS: fls_66m4rnbssgig]
    fn parse_package_decl_inner(
        &mut self,
        mut doc: Option<String>,
    ) -> Result<PackageDef, ParseError> {
        self.expect(&TokenKind::Package)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::OpenBrace)?;
        self.skip_trivia();
        if doc.is_none() {
            doc = self.take_doc_comment();
        }
        let mut pkg = Box::new(PackageDef {
            name,
            doc,
            ..Default::default()
        });
        while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
            self.parse_container_item(&mut pkg, None)?;
            self.skip_trivia();
        }
        self.expect(&TokenKind::CloseBrace)?;
        Ok(*pkg)
    }

    /// Parses a qualified, wildcard (`::*`), or recursive (`::**`) `import` statement into an `ImportDef`.
    /// Realises: [REQ-SYSML-PARSER-IMPORT] | [FLS: fls_66m4rnbssgig]
    pub(crate) fn parse_import_decl(
        &mut self,
        doc: Option<String>,
    ) -> Result<ImportDef, ParseError> {
        self.expect(&TokenKind::Import)?;
        let mut path_segments = Vec::new();
        let mut is_wildcard = false;
        let mut is_recursive = false;
        let leading_colon = self.eat(&TokenKind::DoubleColon);
        path_segments.push(self.expect_ident()?);

        while self.eat(&TokenKind::DoubleColon) {
            if self.eat(&TokenKind::Star) {
                is_wildcard = true;
                is_recursive = self.eat(&TokenKind::Star);
                break;
            }
            path_segments.push(self.expect_ident()?);
        }
        self.expect_semi()?;
        let mut path = path_segments.join("::");
        if leading_colon {
            path = format!("::{}", path);
        }
        Ok(ImportDef {
            path,
            is_wildcard,
            is_recursive,
            doc,
        })
    }

    /// Parses a single member declaration inside a package or part scope bounded by `MAX_PARSE_DEPTH`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-CONTAINER-ITEM] | [FLS: fls_66m4rnbssgig]
    pub(crate) fn parse_container_item(
        &mut self,
        pkg: &mut PackageDef,
        parent_part: Option<&mut PartDef>,
    ) -> Result<(), ParseError> {
        if self.depth >= MAX_PARSE_DEPTH {
            return Err(self.max_depth_error());
        }
        self.depth = self.depth.saturating_add(1);
        let result = self.parse_container_item_inner(pkg, parent_part);
        self.depth = self.depth.saturating_sub(1);
        result
    }

    /// Attaches a container documentation string to `parent_part` or `pkg` if none is set.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-CONTAINER-ITEM] | [FLS: fls_66m4rnbssgig]
    fn assign_container_doc(
        pkg: &mut PackageDef,
        parent_part: Option<&mut PartDef>,
        doc: Option<String>,
    ) {
        if let Some(d) = doc {
            if let Some(parent) = parent_part {
                if parent.doc.is_none() {
                    parent.doc = Some(d);
                }
            } else if pkg.doc.is_none() {
                pkg.doc = Some(d);
            }
        }
    }

    /// Dispatches a single container member declaration within an active depth frame.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-CONTAINER-ITEM] | [FLS: fls_66m4rnbssgig]
    fn parse_container_item_inner(
        &mut self,
        pkg: &mut PackageDef,
        parent_part: Option<&mut PartDef>,
    ) -> Result<(), ParseError> {
        self.skip_trivia();
        let mut doc = self.take_doc_comment();
        self.skip_trivia();
        while let Some(extra_doc) = self.take_doc_comment() {
            doc = Some(extra_doc);
            self.skip_trivia();
        }
        if self.is_at_end() || self.check(&TokenKind::CloseBrace) {
            Self::assign_container_doc(pkg, parent_part, doc);
            return Ok(());
        }
        match self.peek_kind() {
            TokenKind::Doc => {
                self.advance();
                self.skip_trivia();
                if self.eat(&TokenKind::Eq) || self.eat(&TokenKind::Colon) {
                    self.skip_trivia();
                }
                let inner_doc = if let Some(d) = self.take_doc_comment() {
                    Some(d)
                } else if let TokenKind::StringLit(s) = self.peek_kind().clone() {
                    self.advance();
                    Some(s)
                } else {
                    None
                };
                Self::assign_container_doc(pkg, parent_part, inner_doc);
                self.eat(&TokenKind::Semi);
            }
            TokenKind::Package => {
                let subpkg = self.parse_package_decl(doc)?;
                if let Some(existing) = pkg.packages.iter_mut().find(|p| p.name == subpkg.name) {
                    merge_package_defs(existing, subpkg);
                } else {
                    pkg.packages.push(subpkg);
                }
            }
            TokenKind::Part => {
                let part = self.parse_part_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.parts.push(part);
                } else {
                    pkg.part_defs.push(part);
                }
            }
            TokenKind::State => {
                let state = self.parse_state_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.states.push(state);
                } else {
                    pkg.state_defs.push(state);
                }
            }
            _ => self.parse_leaf_container_item(doc, pkg, parent_part)?,
        }
        Ok(())
    }

    /// Applies a parsed `@TriggersHazard`, `@SafetyRealises`, or `@Layer` stereotype to `parent`.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-CONTAINER-ITEM] | [FLS: fls_66m4rnbssgig]
    fn apply_part_stereotype(parent: &mut PartDef, stereo_name: &str, values: Vec<String>) {
        let non_empty = values.into_iter().filter(|v| !v.is_empty());
        match stereo_name {
            "TriggersHazard" => {
                for val in non_empty {
                    if !parent.triggers_hazards.contains(&val) {
                        parent.triggers_hazards.push(val);
                    }
                }
            }
            "SafetyRealises" => {
                for val in non_empty {
                    if !parent.safety_realises.contains(&val) {
                        parent.safety_realises.push(val);
                    }
                }
            }
            "Layer" => {
                if let Some(first_val) = non_empty.into_iter().next() {
                    parent.layer = Some(first_val);
                }
            }
            _ => {}
        }
    }

    /// Parses non-container leaf member declarations inside a package or part scope.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-CONTAINER-ITEM] | [FLS: fls_66m4rnbssgig]
    fn parse_leaf_container_item(
        &mut self,
        doc: Option<String>,
        pkg: &mut PackageDef,
        parent_part: Option<&mut PartDef>,
    ) -> Result<(), ParseError> {
        macro_rules! push_member {
            ($val:expr, $part_field:ident, $pkg_field:ident) => {
                if let Some(parent) = parent_part {
                    parent.$part_field.push($val);
                } else {
                    pkg.$pkg_field.push($val);
                }
            };
        }
        match self.peek_kind() {
            TokenKind::Port => push_member!(self.parse_port_decl(doc)?, ports, port_defs),
            TokenKind::In | TokenKind::Out | TokenKind::Inout if self.peek_next_is_port() => {
                push_member!(self.parse_port_decl(doc)?, ports, port_defs)
            }
            TokenKind::Attribute => push_member!(self.parse_attribute_decl(doc)?, attributes, attribute_defs),
            TokenKind::Action => push_member!(self.parse_action_decl(doc)?, actions, action_defs),
            TokenKind::Constraint | TokenKind::Assert => {
                push_member!(self.parse_constraint_decl(doc)?, constraints, constraint_defs)
            }
            TokenKind::Requirement => {
                push_member!(self.parse_requirement_decl(doc)?, requirements, requirement_defs)
            }
            TokenKind::Satisfy => {
                let req_target = self.parse_satisfy_stmt()?;
                let list = match parent_part {
                    Some(p) => &mut p.satisfied_requirements,
                    None => &mut pkg.satisfied_requirements,
                };
                if !req_target.is_empty() && !list.contains(&req_target) {
                    list.push(req_target);
                }
            }
            TokenKind::Import => pkg.imports.push(self.parse_import_decl(doc)?),
            TokenKind::Connection => push_member!(self.parse_connection_decl(doc, false)?, connections, connection_defs),
            TokenKind::Connect => push_member!(self.parse_inline_connect_decl(doc)?, connections, connection_defs),
            TokenKind::Flow => push_member!(self.parse_connection_decl(doc, true)?, connections, connection_defs),
            TokenKind::UseCase => push_member!(self.parse_use_case_decl(doc)?, use_cases, use_case_defs),
            TokenKind::Item => push_member!(self.parse_item_decl(doc)?, item_defs, item_defs),
            TokenKind::Hazard => push_member!(self.parse_hazard_decl(doc)?, hazards, hazard_defs),
            TokenKind::Risk => push_member!(self.parse_risk_decl(doc)?, risks, risk_defs),
            TokenKind::Test => push_member!(self.parse_test_case_decl(doc)?, test_cases, test_case_defs),
            TokenKind::Capability => push_member!(self.parse_capability_decl(doc)?, capabilities, capability_defs),
            TokenKind::Operation => push_member!(self.parse_operation_decl(doc)?, operations, operation_defs),
            TokenKind::Interaction => push_member!(self.parse_interaction_decl(doc)?, interactions, interaction_defs),
            TokenKind::Calc => push_member!(self.parse_calc_decl(doc)?, calcs, calc_defs),
            _ if self.peek_stereotype_name().is_some() => {
                let (stereo_name, values) = self.parse_stereotype_annotation()?;
                if let Some(parent) = parent_part {
                    Self::apply_part_stereotype(parent, &stereo_name, values);
                }
            }
            _ => {
                let tok = self.peek();
                return Err(ParseError {
                    message: format!(
                        "Unexpected token '{:?}' in container scope; expected a valid SysML v2 declaration",
                        tok.kind
                    ),
                    span: tok.span,
                });
            }
        }
        Ok(())
    }
}
