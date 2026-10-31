//! Package, import, and container-scope declaration dispatchers for SysML v2 / KerML grammar parsing.
//!
//! Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-MODULE]
//! Contract: Parses top-level files, nested `package` blocks, `import` clauses, and scoped package/part container member declarations into deterministic `PackageDef` and `PartDef` AST hierarchies.
//! Bounds & Faults: Enforces `MAX_PARSE_DEPTH` (64) across recursive `package` and container scopes, failing closed with span-annotated `ParseError` diagnostics on unrecognized tokens or depth exhaustion.

use super::*;

impl<'a> SysmlParser<'a> {
    /// Parses a complete token stream representing a `package` forest or flat SysML v2 file into a root `PackageDef`.
    ///
    /// Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-PACKAGE-OR-FILE]
    /// Contract: Consumes all top-level package blocks and container member declarations up to EOF, collapsing single-child synthetic forest roots (`SysML_Model` or empty name) via `merge_package_defs`.
    /// Bounds & Faults: Bounded by `MAX_PARSE_DEPTH` (64) and token stream length `O(N)`; returns a typed `ParseError` without panicking on malformed declarations.
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
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-PACKAGE-DECL]
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
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-PACKAGE-DECL]
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
    /// Realises: [REQ-SYSML-PARSER-IMPORT]
    pub(crate) fn parse_import_decl(
        &mut self,
        doc: Option<String>,
    ) -> Result<ImportDef, ParseError> {
        self.expect(&TokenKind::Import)?;
        let mut path_segments = Vec::new();
        let mut is_wildcard = false;
        let mut is_recursive = false;

        let mut leading_colon = false;
        if self.check(&TokenKind::DoubleColon) {
            self.advance();
            leading_colon = true;
        }

        let first = self.expect_ident()?;
        path_segments.push(first);

        while self.check(&TokenKind::DoubleColon) {
            self.advance();
            if self.check(&TokenKind::Star) {
                self.advance();
                if self.check(&TokenKind::Star) {
                    self.advance();
                    is_recursive = true;
                    is_wildcard = true;
                } else {
                    is_wildcard = true;
                }
                break;
            } else {
                let seg = self.expect_ident()?;
                path_segments.push(seg);
            }
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
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-CONTAINER-ITEM]
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

    /// Dispatches a single container member declaration within an active depth frame.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-CONTAINER-ITEM]
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
            if let Some(d) = doc {
                if let Some(parent) = parent_part {
                    if parent.doc.is_none() {
                        parent.doc = Some(d);
                    }
                } else if pkg.doc.is_none() {
                    pkg.doc = Some(d);
                }
            }
            return Ok(());
        }

        match self.peek_kind() {
            TokenKind::Doc => {
                self.advance();
                self.skip_trivia();
                if self.check(&TokenKind::Eq) || self.check(&TokenKind::Colon) {
                    self.advance();
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
                if let Some(d) = inner_doc {
                    if let Some(parent) = parent_part {
                        if parent.doc.is_none() {
                            parent.doc = Some(d);
                        }
                    } else if pkg.doc.is_none() {
                        pkg.doc = Some(d);
                    }
                }
                if self.check(&TokenKind::Semi) {
                    self.advance();
                }
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
            _ => {
                self.parse_leaf_container_item(doc, pkg, parent_part)?;
            }
        }

        Ok(())
    }

    /// Parses non-container leaf member declarations inside a package or part scope, keeping
    /// leaf AST stack slots off recursive container stack frames.
    /// Realises: [REQ-PARSER-GRAMMAR-SYSMLPARSER-PARSE-CONTAINER-ITEM]
    fn parse_leaf_container_item(
        &mut self,
        doc: Option<String>,
        pkg: &mut PackageDef,
        parent_part: Option<&mut PartDef>,
    ) -> Result<(), ParseError> {
        match self.peek_kind() {
            TokenKind::Port => {
                let port = self.parse_port_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.ports.push(port);
                } else {
                    pkg.port_defs.push(port);
                }
            }
            TokenKind::In | TokenKind::Out | TokenKind::Inout if self.peek_next_is_port() => {
                let port = self.parse_port_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.ports.push(port);
                } else {
                    pkg.port_defs.push(port);
                }
            }
            TokenKind::Attribute => {
                let attr = self.parse_attribute_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.attributes.push(attr);
                } else {
                    pkg.attribute_defs.push(attr);
                }
            }
            TokenKind::Action => {
                let action = self.parse_action_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.actions.push(action);
                } else {
                    pkg.action_defs.push(action);
                }
            }
            TokenKind::Constraint | TokenKind::Assert => {
                let constraint = self.parse_constraint_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.constraints.push(constraint);
                } else {
                    pkg.constraint_defs.push(constraint);
                }
            }
            TokenKind::Requirement => {
                let req = self.parse_requirement_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.requirements.push(req);
                } else {
                    pkg.requirement_defs.push(req);
                }
            }
            TokenKind::Satisfy => {
                let req_target = self.parse_satisfy_stmt()?;
                if !req_target.is_empty() {
                    if let Some(parent) = parent_part {
                        if !parent.satisfied_requirements.contains(&req_target) {
                            parent.satisfied_requirements.push(req_target);
                        }
                    } else if !pkg.satisfied_requirements.contains(&req_target) {
                        pkg.satisfied_requirements.push(req_target);
                    }
                }
            }
            TokenKind::Import => {
                let imp = self.parse_import_decl(doc)?;
                pkg.imports.push(imp);
            }
            TokenKind::Connection => {
                let conn = self.parse_connection_decl(doc, false)?;
                if let Some(parent) = parent_part {
                    parent.connections.push(conn);
                } else {
                    pkg.connection_defs.push(conn);
                }
            }
            TokenKind::Connect => {
                let conn = self.parse_inline_connect_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.connections.push(conn);
                } else {
                    pkg.connection_defs.push(conn);
                }
            }
            TokenKind::Flow => {
                let conn = self.parse_connection_decl(doc, true)?;
                if let Some(parent) = parent_part {
                    parent.connections.push(conn);
                } else {
                    pkg.connection_defs.push(conn);
                }
            }
            TokenKind::UseCase => {
                let uc = self.parse_use_case_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.use_cases.push(uc);
                } else {
                    pkg.use_case_defs.push(uc);
                }
            }
            TokenKind::Item => {
                let item = self.parse_item_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.item_defs.push(item);
                } else {
                    pkg.item_defs.push(item);
                }
            }
            TokenKind::Hazard => {
                let hz = self.parse_hazard_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.hazards.push(hz);
                } else {
                    pkg.hazard_defs.push(hz);
                }
            }
            TokenKind::Risk => {
                let rk = self.parse_risk_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.risks.push(rk);
                } else {
                    pkg.risk_defs.push(rk);
                }
            }
            TokenKind::Test => {
                let tc = self.parse_test_case_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.test_cases.push(tc);
                } else {
                    pkg.test_case_defs.push(tc);
                }
            }
            TokenKind::Capability => {
                let cap = self.parse_capability_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.capabilities.push(cap);
                } else {
                    pkg.capability_defs.push(cap);
                }
            }
            TokenKind::Operation => {
                let op = self.parse_operation_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.operations.push(op);
                } else {
                    pkg.operation_defs.push(op);
                }
            }
            TokenKind::Interaction => {
                let inter = self.parse_interaction_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.interactions.push(inter);
                } else {
                    pkg.interaction_defs.push(inter);
                }
            }
            TokenKind::Calc => {
                let c = self.parse_calc_decl(doc)?;
                if let Some(parent) = parent_part {
                    parent.calcs.push(c);
                } else {
                    pkg.calc_defs.push(c);
                }
            }
            _ if self.peek_stereotype_name().is_some() => {
                let (stereo_name, values) = self.parse_stereotype_annotation()?;
                if let Some(parent) = parent_part {
                    if stereo_name == "TriggersHazard" {
                        for val in values {
                            if !val.is_empty() && !parent.triggers_hazards.contains(&val) {
                                parent.triggers_hazards.push(val);
                            }
                        }
                    } else if stereo_name == "SafetyRealises" {
                        for val in values {
                            if !val.is_empty() && !parent.safety_realises.contains(&val) {
                                parent.safety_realises.push(val);
                            }
                        }
                    } else if stereo_name == "Layer" {
                        if let Some(first_val) = values.into_iter().find(|v| !v.is_empty()) {
                            parent.layer = Some(first_val);
                        }
                    }
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
