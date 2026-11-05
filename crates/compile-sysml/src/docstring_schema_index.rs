#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! [DOC-TIER: A] Depth-bounded pre-order SysML v2 package and part lookup index (`SchemaLookupIndex`).
//! Safety & Traceability: Realises: [REQ-FEAT-42-SCHEMA-LOOKUP-INDEX], [REQ-0605]
//! Contract: Indexes `PackageDef` and `PartDef` hierarchies up to `MAX_AST_DEPTH` (`64`) for requirement and constraint resolution.
//! Bounds & Faults: O(P + R + C) traversal time and O(P) reference space; zero recursion.

use crate::docstring_schema_bind::{
    collect_all_parts_from_depth, constraint_matches_action, extract_all_realises_ids,
    format_schema_constraint, normalize_ascii_alnum_lower, part_satisfies_requirement_def,
    push_unique_schema_constraint, push_unique_schema_req, req_def_primary_id,
    schema_identifiers_match, MAX_AST_DEPTH,
};
use deap_core::sysml_ast::{ActionDef, ConstraintDef, PackageDef, PartDef, RequirementDef};

/// Depth-bounded pre-order index of SysML v2 packages and parts up to `MAX_AST_DEPTH`.
/// Realises: [REQ-FEAT-42-SCHEMA-LOOKUP-INDEX]
pub(crate) struct SchemaLookupIndex<'a> {
    pub(crate) all_pkgs: Vec<&'a PackageDef>,
    pub(crate) all_parts: Vec<&'a PartDef>,
    active_part_ident: &'a str,
    item_is_other_part: bool,
}

impl<'a> SchemaLookupIndex<'a> {
    /// Builds a `SchemaLookupIndex` over `root_pkg`, stopping when `depth > MAX_AST_DEPTH`.
    /// Realises: [REQ-FEAT-42-SCHEMA-LOOKUP-INDEX-NEW]
    pub(crate) fn new(
        root_pkg: &'a PackageDef,
        owner_name: Option<&'a str>,
        item_name: &'a str,
    ) -> Self {
        let mut all_pkgs: Vec<&'a PackageDef> = Vec::new();
        let mut all_parts: Vec<&'a PartDef> = Vec::new();
        let mut stack: Vec<(&'a PackageDef, usize)> = vec![(root_pkg, 0)];
        while let Some((curr, depth)) = stack.pop() {
            if depth > MAX_AST_DEPTH {
                continue;
            }
            all_pkgs.push(curr);
            collect_all_parts_from_depth(curr, depth, &mut all_parts);
            let next_depth = depth.saturating_add(1);
            if next_depth <= MAX_AST_DEPTH {
                for sub in curr.packages.iter().rev() {
                    stack.push((sub, next_depth));
                }
            }
        }
        let active_part_ident = owner_name.unwrap_or(item_name);
        let item_is_other_part = all_parts
            .iter()
            .any(|p| schema_identifiers_match(&p.name, item_name));
        Self {
            all_pkgs,
            all_parts,
            active_part_ident,
            item_is_other_part,
        }
    }

    /// Resolves a requirement name or ID token against all `requirement_defs` in `self.all_pkgs`.
    /// Realises: [REQ-FEAT-42-SCHEMA-RESOLVE-REQ-TOKEN]
    fn resolve_req_token(&self, token: &str) -> Option<String> {
        let trimmed = token.trim();
        if trimmed.is_empty() {
            return None;
        }
        for pkg in &self.all_pkgs {
            if let Some(req_def) = pkg.requirement_defs.iter().find(|r| {
                schema_identifiers_match(&r.name, trimmed)
                    || (!r.req_id.is_empty() && schema_identifiers_match(&r.req_id, trimmed))
            }) {
                let primary = req_def_primary_id(req_def);
                if !primary.is_empty() {
                    return Some(primary.to_string());
                }
            }
        }
        Some(trimmed.to_string())
    }

    /// Extracts requirement identifiers from an action or element `doc` comment block.
    /// Realises: [REQ-FEAT-42-SCHEMA-EXTRACT-DOC-REQ]
    fn extract_doc_req(&self, doc: &str) -> Option<String> {
        let mut collected = Vec::new();
        let doc_lines: Vec<String> = doc.lines().map(str::to_string).collect();
        for req in extract_all_realises_ids(&doc_lines) {
            if let Some(resolved) = self.resolve_req_token(&req) {
                push_unique_schema_req(&mut collected, resolved);
            }
        }
        let mut rest = doc;
        while let Some(open_idx) = rest.find('[') {
            let after_open = rest.get(open_idx.saturating_add(1)..).unwrap_or("");
            let Some(close_idx) = after_open.find(']') else {
                break;
            };
            for token in after_open.get(..close_idx).unwrap_or("").split(',') {
                let tok = token.trim();
                let upper = tok.to_ascii_uppercase();
                if upper.starts_with("REQ-") || upper.starts_with("REQ_") {
                    if let Some(resolved) = self.resolve_req_token(tok) {
                        push_unique_schema_req(&mut collected, resolved);
                    }
                }
            }
            rest = after_open.get(close_idx.saturating_add(1)..).unwrap_or("");
        }
        (!collected.is_empty()).then(|| collected.join(", "))
    }

    /// Aggregates all satisfied requirements and inline constraints declared on `part`.
    /// Realises: [REQ-FEAT-42-SCHEMA-EXTRACT-PART-CONTEXT]
    pub(crate) fn extract_part_req_and_constraint(
        &self,
        part: &PartDef,
    ) -> (Option<String>, Option<String>) {
        let mut part_reqs = Vec::new();
        for r in part.satisfied_requirements.iter().chain(part.safety_realises.iter()) {
            if let Some(resolved) = self.resolve_req_token(r) {
                push_unique_schema_req(&mut part_reqs, resolved);
            }
        }
        for r in &part.requirements {
            if let Some(resolved) = self.resolve_req_token(req_def_primary_id(r)) {
                push_unique_schema_req(&mut part_reqs, resolved);
            }
        }
        for pkg in &self.all_pkgs {
            for req in &pkg.requirement_defs {
                if req.satisfied_by.iter().any(|s| schema_identifiers_match(s, &part.name)) {
                    push_unique_schema_req(&mut part_reqs, req_def_primary_id(req).to_string());
                }
            }
        }
        let mut part_constraints = Vec::new();
        for c in &part.constraints {
            if let Some(formatted) = format_schema_constraint(&c.name, &c.expression) {
                push_unique_schema_constraint(&mut part_constraints, formatted);
            }
        }
        (
            (!part_reqs.is_empty()).then(|| part_reqs.join(", ")),
            (!part_constraints.is_empty()).then(|| part_constraints.join("; ")),
        )
    }

    /// Checks whether `r` is owned by or named after a `PartDef` distinct from `self.active_part_ident`.
    /// Realises: [REQ-FEAT-42-SCHEMA-REQ-OTHER-PART]
    fn req_belongs_to_other_part(&self, r: &RequirementDef) -> bool {
        let (nr, nreq) = (
            normalize_ascii_alnum_lower(&r.name),
            normalize_ascii_alnum_lower(&r.req_id),
        );
        self.all_parts.iter().any(|p| {
            if schema_identifiers_match(&p.name, self.active_part_ident) {
                return false;
            }
            let np = normalize_ascii_alnum_lower(&p.name);
            part_satisfies_requirement_def(p, r)
                || (np.len() >= 2 && (nr.contains(&np) || nreq.contains(&np)))
        })
    }

    /// Checks whether `c` is owned by or named after a `PartDef` distinct from `self.active_part_ident`.
    /// Realises: [REQ-FEAT-42-SCHEMA-CONSTRAINT-OTHER-PART]
    fn constraint_belongs_to_other_part(&self, c: &ConstraintDef) -> bool {
        let nc = normalize_ascii_alnum_lower(&c.name);
        self.all_parts.iter().any(|p| {
            if schema_identifiers_match(&p.name, self.active_part_ident) {
                return false;
            }
            let np = normalize_ascii_alnum_lower(&p.name);
            p.constraints.iter().any(|pc| schema_identifiers_match(&pc.name, &c.name))
                || (np.len() >= 2 && nc.contains(&np))
        })
    }

    /// Checks whether `r` binds specifically to `member_ident` without bleeding across sibling parts.
    /// Realises: [REQ-FEAT-42-SCHEMA-REQ-MATCHES-MEMBER]
    fn req_matches_member(&self, r: &RequirementDef, member_ident: &str) -> bool {
        if self.req_belongs_to_other_part(r) {
            return false;
        }
        if schema_identifiers_match(&r.name, member_ident)
            || schema_identifiers_match(&r.req_id, member_ident)
            || r.satisfied_by.iter().any(|s| schema_identifiers_match(s, member_ident))
        {
            return true;
        }
        let norm_member = normalize_ascii_alnum_lower(member_ident);
        if !self.item_is_other_part && norm_member.len() >= 4 {
            let (nr, nreq) = (
                normalize_ascii_alnum_lower(&r.name),
                normalize_ascii_alnum_lower(&r.req_id),
            );
            return nr.contains(&norm_member) || nreq.contains(&norm_member);
        }
        false
    }

    /// Aggregates action-matched constraints from `constraints` (or `act.steps` fallback) for `act`.
    /// Realises: [REQ-FEAT-42-SCHEMA-RESOLVE-ACTION-CONSTRAINTS]
    fn resolve_action_constraints(
        &self,
        act: &ActionDef,
        constraints: &[ConstraintDef],
        filter_other_parts: bool,
    ) -> Option<String> {
        let mut act_constraints = Vec::new();
        for act_c in constraints {
            let allowed = !filter_other_parts || !self.constraint_belongs_to_other_part(act_c);
            if allowed && constraint_matches_action(&act_c.name, &act.name) {
                if let Some(formatted) = format_schema_constraint(&act_c.name, &act_c.expression) {
                    push_unique_schema_constraint(&mut act_constraints, formatted);
                }
            }
        }
        if !act_constraints.is_empty() {
            return Some(act_constraints.join("; "));
        }
        for step_expr in &act.steps {
            if let Some(formatted) = format_schema_constraint(&act.name, step_expr) {
                push_unique_schema_constraint(&mut act_constraints, formatted);
            }
        }
        (!act_constraints.is_empty()).then(|| act_constraints.join("; "))
    }

    /// Searches `PackageDef::action_defs` across `self.all_pkgs` for an action matching `item_name`.
    /// Realises: [REQ-FEAT-42-SCHEMA-RESOLVE-PKG-ACTION]
    pub(crate) fn resolve_pkg_action(
        &self,
        item_name: &str,
        need_constraint: bool,
    ) -> Option<(Option<String>, Option<String>)> {
        for pkg in &self.all_pkgs {
            let Some(act) = pkg
                .action_defs
                .iter()
                .find(|a| schema_identifiers_match(&a.name, item_name))
            else {
                continue;
            };
            let mut matched_req = act.doc.as_deref().and_then(|doc| self.extract_doc_req(doc));
            if matched_req.is_none() {
                let mut act_reqs = Vec::new();
                for req_def in pkg
                    .requirement_defs
                    .iter()
                    .filter(|r| self.req_matches_member(r, &act.name))
                {
                    push_unique_schema_req(&mut act_reqs, req_def_primary_id(req_def).to_string());
                }
                matched_req = (!act_reqs.is_empty()).then(|| act_reqs.join(", "));
            }
            let matched_constraint = need_constraint
                .then(|| self.resolve_action_constraints(act, &pkg.constraint_defs, true))
                .flatten();
            return Some((matched_req, matched_constraint));
        }
        None
    }

    /// Resolves `(req, constraint)` for a member `item_name` owned by `owner`.
    /// Realises: [REQ-FEAT-42-SCHEMA-RESOLVE-OWNED-ITEM]
    pub(crate) fn resolve_owned_item(
        &self,
        owner: &str,
        item_name: &str,
    ) -> Option<(Option<String>, Option<String>)> {
        let owner_part = self
            .all_parts
            .iter()
            .copied()
            .find(|p| schema_identifiers_match(&p.name, owner))?;
        let (p_req, mut matched_constraint) = self.extract_part_req_and_constraint(owner_part);
        let mut matched_req: Option<String> = None;

        if let Some(act) = owner_part
            .actions
            .iter()
            .find(|a| schema_identifiers_match(&a.name, item_name))
        {
            matched_req = act.doc.as_deref().and_then(|doc| self.extract_doc_req(doc));
            if let Some(act_const) =
                self.resolve_action_constraints(act, &owner_part.constraints, false)
            {
                matched_constraint = Some(act_const);
            }
        }
        if matched_req.is_none() {
            let mut member_reqs = Vec::new();
            for r in owner_part
                .requirements
                .iter()
                .filter(|r| self.req_matches_member(r, item_name))
            {
                push_unique_schema_req(&mut member_reqs, req_def_primary_id(r).to_string());
            }
            matched_req = (!member_reqs.is_empty()).then(|| member_reqs.join(", "));
        }
        if matched_req.is_none() {
            let mut pkg_reqs = Vec::new();
            for pkg in &self.all_pkgs {
                for r in pkg
                    .requirement_defs
                    .iter()
                    .filter(|r| self.req_matches_member(r, item_name))
                {
                    push_unique_schema_req(&mut pkg_reqs, req_def_primary_id(r).to_string());
                }
            }
            matched_req = (!pkg_reqs.is_empty()).then(|| pkg_reqs.join(", "));
        }
        if matched_req.is_none() {
            if let Some((act_req, act_const)) =
                self.resolve_pkg_action(item_name, matched_constraint.is_none())
            {
                matched_req = act_req;
                if matched_constraint.is_none() {
                    matched_constraint = act_const;
                }
            }
        }
        Some((matched_req.or(p_req), matched_constraint))
    }

    /// Resolves package-level fallback requirements and constraints matching `target_ident` or `item_name`.
    /// Realises: [REQ-FEAT-42-SCHEMA-FALLBACK-REQS]
    pub(crate) fn resolve_fallbacks(
        &self,
        target_ident: &str,
        item_name: &str,
        need_req: bool,
        need_constraint: bool,
    ) -> (Option<String>, Option<String>) {
        let mut fallback_reqs = Vec::new();
        let mut fallback_constraints = Vec::new();
        for pkg in &self.all_pkgs {
            if need_req {
                for r in &pkg.requirement_defs {
                    if !self.req_belongs_to_other_part(r)
                        && (schema_identifiers_match(&r.name, target_ident)
                            || schema_identifiers_match(&r.req_id, target_ident)
                            || r.satisfied_by
                                .iter()
                                .any(|s| schema_identifiers_match(s, target_ident))
                            || schema_identifiers_match(&r.name, item_name)
                            || schema_identifiers_match(&r.req_id, item_name))
                    {
                        push_unique_schema_req(&mut fallback_reqs, req_def_primary_id(r).to_string());
                    }
                }
            }
            if need_constraint {
                for c_def in &pkg.constraint_defs {
                    if !self.constraint_belongs_to_other_part(c_def)
                        && (schema_identifiers_match(&c_def.name, target_ident)
                            || schema_identifiers_match(&c_def.name, item_name)
                            || constraint_matches_action(&c_def.name, item_name))
                    {
                        if let Some(f) = format_schema_constraint(&c_def.name, &c_def.expression) {
                            push_unique_schema_constraint(&mut fallback_constraints, f);
                        }
                    }
                }
            }
        }
        (
            (!fallback_reqs.is_empty()).then(|| fallback_reqs.join(", ")),
            (!fallback_constraints.is_empty()).then(|| fallback_constraints.join("; ")),
        )
    }
}
