//! Semantic validation engine for SysML v2 / KerML models.

use super::symbols::{Symbol, SymbolKind, SymbolTable};
use deap_core::sysml_ast::*;
use std::collections::BTreeSet;

/// Severity level of semantic diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

/// Structured diagnostic emitted during semantic validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticDiagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub element: String,
}

impl SemanticDiagnostic {
    pub fn error(element: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            element: element.into(),
        }
    }

    pub fn warning(element: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
            element: element.into(),
        }
    }
}

/// Semantic validator verifying topology, types, and referential integrity.
pub struct SemanticValidator<'a> {
    pkg: &'a PackageDef,
    symbols: SymbolTable,
    diagnostics: Vec<SemanticDiagnostic>,
}

impl<'a> SemanticValidator<'a> {
    pub fn new(pkg: &'a PackageDef) -> Self {
        let symbols = SymbolTable::build_from_package(pkg);
        Self {
            pkg,
            symbols,
            diagnostics: Vec::new(),
        }
    }

    /// Execute complete semantic validation suite.
    pub fn validate(&mut self) -> &[SemanticDiagnostic] {
        self.diagnostics.clear();
        self.validate_part_hierarchy();
        self.validate_connections();
        self.validate_referential_integrity();
        self.validate_part_defs();
        self.validate_requirements();
        &self.diagnostics
    }

    /// Check if any error-level diagnostics were recorded.
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == DiagnosticSeverity::Error)
    }

    /// Filter error diagnostics.
    pub fn errors(&self) -> Vec<&SemanticDiagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .collect()
    }

    // --- Validation Rules ---

    fn validate_part_hierarchy(&mut self) {
        let mut seen_parts = BTreeSet::new();
        self.validate_pkg_part_hierarchy(self.pkg, "", &mut seen_parts);
    }

    fn validate_pkg_part_hierarchy(
        &mut self,
        pkg: &PackageDef,
        parent_scope: &str,
        seen: &mut BTreeSet<String>,
    ) {
        let pkg_scope = if parent_scope.is_empty() {
            if std::ptr::eq(pkg, self.pkg) {
                String::new()
            } else {
                pkg.name.clone()
            }
        } else if pkg.name.is_empty() {
            parent_scope.to_string()
        } else {
            format!("{}::{}", parent_scope, pkg.name)
        };

        self.check_parts_recursive(&pkg.part_defs, &pkg_scope, seen);

        for subpkg in &pkg.packages {
            self.validate_pkg_part_hierarchy(subpkg, &pkg_scope, seen);
        }
    }

    fn check_parts_recursive(
        &mut self,
        parts: &[PartDef],
        scope: &str,
        seen: &mut BTreeSet<String>,
    ) {
        for part in parts {
            let part_key = if scope.is_empty() {
                part.name.clone()
            } else {
                format!("{}::{}", scope, part.name)
            };

            if !seen.insert(part_key.clone()) {
                self.diagnostics.push(SemanticDiagnostic::error(
                    &part.name,
                    format!("Duplicate part declaration '{}' in scope '{}'", part.name, scope),
                ));
            }

            // Check duplicate ports on this part
            let mut seen_ports = BTreeSet::new();
            for port in &part.ports {
                if !seen_ports.insert(port.name.clone()) {
                    self.diagnostics.push(SemanticDiagnostic::error(
                        format!("{}.{}", part.name, port.name),
                        format!(
                            "Duplicate port declaration '{}' on part '{}'",
                            port.name, part.name
                        ),
                    ));
                }
            }

            self.check_parts_recursive(&part.parts, &part_key, seen);
        }
    }

    fn validate_connections(&mut self) {
        self.validate_package_connections(self.pkg);
    }

    fn validate_package_connections(&mut self, pkg: &PackageDef) {
        self.validate_connection_list(&pkg.connection_defs, None);

        for part in &pkg.part_defs {
            self.validate_part_connections(part);
        }

        for subpkg in &pkg.packages {
            self.validate_package_connections(subpkg);
        }
    }

    fn validate_part_connections(&mut self, part: &PartDef) {
        self.validate_connection_list(&part.connections, Some(&part.name));
        for subpart in &part.parts {
            self.validate_part_connections(subpart);
        }
    }

    fn validate_connection_list(&mut self, conns: &[ConnectionDef], part_scope: Option<&str>) {
        for conn in conns {
            let src_sym = self.symbols.lookup_port(&conn.source_port, part_scope);
            let tgt_sym = self.symbols.lookup_port(&conn.target_port, part_scope);

            if src_sym.is_none() {
                self.diagnostics.push(SemanticDiagnostic::error(
                    &conn.name,
                    format!(
                        "Connection '{}' references undefined source port '{}'",
                        conn.name, conn.source_port
                    ),
                ));
            }

            if tgt_sym.is_none() {
                self.diagnostics.push(SemanticDiagnostic::error(
                    &conn.name,
                    format!(
                        "Connection '{}' references undefined target port '{}'",
                        conn.name, conn.target_port
                    ),
                ));
            }

            if let (Some(src), Some(tgt)) = (src_sym, tgt_sym) {
                let src_eff_dir = get_effective_direction(src);
                let tgt_eff_dir = get_effective_direction(tgt);

                // Boundary delegation:
                // Inside a part definition, connecting an input boundary port of the enclosing part
                // inward to a subpart's input port (in -> in), or an internal subpart's output port
                // outward to an output boundary port (out -> out), represents valid delegation in SysML v2.
                let is_src_boundary = part_scope.is_some() && !conn.source_port.contains('.');
                let is_tgt_boundary = part_scope.is_some() && !conn.target_port.contains('.');

                if is_src_boundary && !is_tgt_boundary && src_eff_dir == "in" && tgt_eff_dir == "in" {
                    // Valid inward delegation: boundary in -> subpart in
                } else if !is_src_boundary && is_tgt_boundary && src_eff_dir == "out" && tgt_eff_dir == "out" {
                    // Valid outward delegation: subpart out -> boundary out
                } else if is_src_boundary && is_tgt_boundary && src_eff_dir == "in" && tgt_eff_dir == "out" {
                    // Valid boundary pass-through delegation: boundary in -> boundary out
                } else {
                    match (src_eff_dir.as_str(), tgt_eff_dir.as_str()) {
                        ("out", "in") | ("inout", "in") | ("out", "inout") | ("inout", "inout") => {
                            // Compatible flow direction
                        }
                        ("in", "out") => {
                            self.diagnostics.push(SemanticDiagnostic::error(
                                &conn.name,
                                format!(
                                    "Inverted flow direction in connection '{}': source port '{}' ({}) cannot feed target port '{}' ({})",
                                    conn.name, conn.source_port, src_eff_dir, conn.target_port, tgt_eff_dir
                                ),
                            ));
                        }
                        ("in", "in") => {
                            self.diagnostics.push(SemanticDiagnostic::error(
                                &conn.name,
                                format!(
                                    "Invalid connection '{}': cannot connect input port '{}' to input port '{}'",
                                    conn.name, conn.source_port, conn.target_port
                                ),
                            ));
                        }
                        ("out", "out") => {
                            self.diagnostics.push(SemanticDiagnostic::error(
                                &conn.name,
                                format!(
                                    "Invalid connection '{}': cannot connect output port '{}' to output port '{}'",
                                    conn.name, conn.source_port, conn.target_port
                                ),
                            ));
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn validate_referential_integrity(&mut self) {
        let known_constraints: BTreeSet<String> = self
            .symbols
            .all_of_kind(SymbolKind::Constraint)
            .into_iter()
            .map(|s| normalize_id(&s.name))
            .collect();

        let known_hazards: BTreeSet<String> = self
            .symbols
            .all_of_kind(SymbolKind::Hazard)
            .into_iter()
            .map(|s| normalize_id(&s.name))
            .collect();

        let extract_annotation_targets = |text: &str, tag: &str| -> Vec<Vec<String>> {
            let mut results = Vec::new();
            let lower_text = text.to_ascii_lowercase();
            let lower_tag = tag.to_ascii_lowercase();
            let mut search_idx = 0;
            while let Some(pos) = lower_text[search_idx..].find(&lower_tag) {
                let tag_start = search_idx + pos;
                let idx = tag_start + tag.len();
                let rest = &text[idx..];
                let (ws_end, next_char) = rest
                    .char_indices()
                    .find(|(_, c)| !c.is_whitespace())
                    .map_or((rest.len(), None), |(c_idx, c)| (c_idx, Some(c)));
                match next_char {
                    Some('(') => {
                        let inside_start = idx + ws_end + 1;
                        if let Some(close_pos) = text[inside_start..].find(')') {
                            let inside = &text[inside_start..inside_start + close_pos];
                            let targets: Vec<String> = inside
                                .split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect();
                            results.push(targets);
                            search_idx = inside_start + close_pos + 1;
                            continue;
                        }
                    }
                    Some(':') => {
                        let after_colon = &rest[ws_end + 1..];
                        let id_start = after_colon
                            .char_indices()
                            .find(|(_, c)| !c.is_whitespace())
                            .map_or(after_colon.len(), |(c_idx, _)| c_idx);
                        let id_rest = &after_colon[id_start..];
                        let id_len = id_rest
                            .chars()
                            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                            .map(|c| c.len_utf8())
                            .sum::<usize>();
                        if id_len > 0 {
                            let id = id_rest[..id_len].trim().to_string();
                            results.push(vec![id]);
                            let global_end = idx + ws_end + 1 + id_start + id_len;
                            search_idx = global_end.max(tag_start + 1);
                            continue;
                        }
                    }
                    _ => {}
                }
                search_idx = tag_start + 1;
            }
            results
        };

        let mut req_texts = Vec::new();
        for p in self.pkg.get_all_packages() {
            self.collect_requirement_texts(&p.requirement_defs, &mut req_texts);
            for part in &p.part_defs {
                self.collect_part_requirement_texts(part, &mut req_texts);
            }
        }

        for (req_name, text_corpus) in req_texts {
            // Check @SafetyRealises
            for targets in extract_annotation_targets(&text_corpus, "@SafetyRealises") {
                for target in &targets {
                    let clean_target = normalize_id(target);
                    if !known_constraints.is_empty()
                        && !known_constraints.iter().any(|c| c == &clean_target || c.contains(&clean_target))
                    {
                        self.diagnostics.push(SemanticDiagnostic::error(
                            &req_name,
                            format!(
                                "Requirement '{}' references undefined constraint/UCA '{}' in @SafetyRealises",
                                req_name, target
                            ),
                        ));
                    }
                }
            }

            // Check @TriggersHazard
            for targets in extract_annotation_targets(&text_corpus, "@TriggersHazard") {
                for target in &targets {
                    let clean_target = normalize_id(target);
                    if !known_hazards.is_empty()
                        && !known_hazards.iter().any(|h| h == &clean_target || h.contains(&clean_target))
                    {
                        self.diagnostics.push(SemanticDiagnostic::error(
                            &req_name,
                            format!(
                                "Requirement '{}' references undefined hazard '{}' in @TriggersHazard",
                                req_name, target
                            ),
                        ));
                    }
                }
            }
        }
    }

    fn collect_requirement_texts(&self, reqs: &[RequirementDef], out: &mut Vec<(String, String)>) {
        for req in reqs {
            let corpus = format!(
                "{} {} {}",
                req.doc.as_deref().unwrap_or(""),
                req.text,
                req.assumes.join(" ")
            );
            out.push((req.name.clone(), corpus));
        }
    }

    fn collect_part_requirement_texts(&self, part: &PartDef, out: &mut Vec<(String, String)>) {
        self.collect_requirement_texts(&part.requirements, out);
        for subpart in &part.parts {
            self.collect_part_requirement_texts(subpart, out);
        }
    }

    /// Validate part definitions, ensuring all referenced types resolve within local scope, imported namespaces, or global symbols.
    fn validate_part_defs(&mut self) {
        self.validate_part_defs_in_pkg(self.pkg, "");
    }

    fn validate_part_defs_in_pkg(&mut self, pkg: &PackageDef, parent_scope: &str) {
        let pkg_scope = if parent_scope.is_empty() {
            pkg.name.clone()
        } else if pkg.name.is_empty() {
            parent_scope.to_string()
        } else {
            format!("{}::{}", parent_scope, pkg.name)
        };

        for part in &pkg.part_defs {
            self.validate_single_part_def(part, &pkg_scope);
        }

        for subpkg in &pkg.packages {
            self.validate_part_defs_in_pkg(subpkg, &pkg_scope);
        }
    }

    fn validate_single_part_def(&mut self, part: &PartDef, scope: &str) {
        if let Some(ref ty) = part.type_name {
            if !self.is_resolvable_type(scope, ty) {
                self.diagnostics.push(SemanticDiagnostic::error(
                    &part.name,
                    format!("Part '{}' references undefined type '{}'", part.name, ty),
                ));
            }
        }

        let part_scope = if scope.is_empty() {
            part.name.clone()
        } else {
            format!("{}::{}", scope, part.name)
        };

        for subpart in &part.parts {
            self.validate_single_part_def(subpart, &part_scope);
        }
    }

    /// Validate requirement definitions, ensuring satisfying elements and attribute types resolve in scope.
    fn validate_requirements(&mut self) {
        self.validate_requirements_in_pkg(self.pkg, "");
    }

    fn validate_requirements_in_pkg(&mut self, pkg: &PackageDef, parent_scope: &str) {
        let pkg_scope = if parent_scope.is_empty() {
            pkg.name.clone()
        } else if pkg.name.is_empty() {
            parent_scope.to_string()
        } else {
            format!("{}::{}", parent_scope, pkg.name)
        };

        for req in &pkg.requirement_defs {
            self.validate_single_requirement_def(req, &pkg_scope);
        }

        for part in &pkg.part_defs {
            self.validate_part_requirements(part, &pkg_scope);
        }

        for subpkg in &pkg.packages {
            self.validate_requirements_in_pkg(subpkg, &pkg_scope);
        }
    }

    fn validate_single_requirement_def(&mut self, req: &RequirementDef, scope: &str) {
        for attr in &req.attributes {
            if !self.is_resolvable_type(scope, &attr.type_name) {
                self.diagnostics.push(SemanticDiagnostic::error(
                    &req.name,
                    format!(
                        "Requirement '{}' attribute '{}' references undefined type '{}'",
                        req.name, attr.name, attr.type_name
                    ),
                ));
            }
        }

        for target in &req.satisfied_by {
            if !self.is_resolvable_type(scope, target) {
                self.diagnostics.push(SemanticDiagnostic::error(
                    &req.name,
                    format!(
                        "Requirement '{}' references undefined satisfying element '{}'",
                        req.name, target
                    ),
                ));
            }
        }
    }

    fn validate_part_requirements(&mut self, part: &PartDef, scope: &str) {
        let part_scope = if scope.is_empty() {
            part.name.clone()
        } else {
            format!("{}::{}", scope, part.name)
        };

        for req in &part.requirements {
            self.validate_single_requirement_def(req, &part_scope);
        }

        for subpart in &part.parts {
            self.validate_part_requirements(subpart, &part_scope);
        }
    }

    /// Checks whether a type expression is resolvable in the given scope.
    ///
    /// Resolves primitive and builtin types, types defined in the symbol table
    /// (including imported packages and outer scopes), and recursively resolves
    /// generic container parameters (e.g., `Vec<CustomPart>`, `Option<SensorNode>`,
    /// `HashMap<String, Vec<SensorNode>>`).
    ///
    /// Enforces strict fail-closed semantics: if an outer container is recognized
    /// but any inner type argument is undefined, or if an outer container is undefined,
    /// this function returns `false`.
    pub fn is_resolvable_type(&self, scope: &str, type_name: &str) -> bool {
        let clean = normalize_source_type(type_name);
        if clean.is_empty() {
            return false;
        }
        if clean == "()" {
            return true;
        }

        // Handle tuple types FIRST: (T1, T2, ...)
        if clean.starts_with('(') && clean.ends_with(')') {
            let mut depth = 0usize;
            let mut closed_early = false;
            let last_idx = clean.len() - 1;
            for (idx, ch) in clean.char_indices() {
                match ch {
                    '(' => depth += 1,
                    ')' => {
                        depth = depth.saturating_sub(1);
                        if depth == 0 && idx < last_idx {
                            closed_early = true;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if depth == 0 && !closed_early {
                let inner = clean[1..last_idx].trim();
                if inner.is_empty() {
                    return true;
                }
                let args = split_top_level_type_args(inner);
                return !args.is_empty() && args.into_iter().all(|arg| self.is_resolvable_type(scope, arg));
            }
        }

        // Handle generic containers: Outer<Inner1, Inner2, ...>
        if let Some(lt_idx) = clean.find('<') {
            if let Some(gt_idx) = clean.rfind('>') {
                if gt_idx > lt_idx {
                    if !clean[gt_idx + 1..].trim().is_empty() {
                        // Trailing tokens after '>' (e.g. Option<String> UndefinedType) must fail closed
                        return false;
                    }
                    let outer = clean[..lt_idx].trim();
                    let inner_str = &clean[lt_idx + 1..gt_idx];

                    // Outer container must be a recognized builtin or a defined type in scope
                    let outer_ok = is_primitive_or_builtin_type(outer)
                        || self.symbols.lookup_in_scope(scope, outer).is_some();

                    if !outer_ok {
                        return false;
                    }

                    // Decompose top-level type arguments
                    let args = split_top_level_type_args(inner_str);
                    if args.is_empty() {
                        // Empty generic parameters (e.g. `Vec<>`) are invalid
                        return false;
                    }

                    // Recursively verify EVERY inner type argument
                    return args.into_iter().all(|arg| self.is_resolvable_type(scope, arg));
                }
            }
        }

        // Check primitive / builtin types
        if is_primitive_or_builtin_type(clean) {
            return true;
        }

        // Check user-defined symbols in scope (including imports and global fallback)
        self.symbols.lookup_in_scope(scope, clean).is_some()
    }
}

/// Deterministically normalizes a source-language type expression by stripping reference/pointer
/// prefixes (`&`, `&mut`, `&const`, `*const`, `*mut`), lifetime specifiers (`'a`, `'static`),
/// array wrappers (`[T; N]`, `[T]`), multiplicity suffixes (`[1..*]`, `[]`), and nullable markers (`?`).
///
/// Complies with Check 34 Zero Regex invariant using character/string slicing only.
pub fn normalize_source_type<'a>(raw: &'a str) -> &'a str {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return "";
    }
    let mut s = trimmed;
    loop {
        let prev = s;

        // 1. Strip trailing nullable markers '?'
        while let Some(stripped) = s.strip_suffix('?') {
            s = stripped.trim_end();
        }

        // 2. Strip reference and pointer prefixes
        if let Some(rest) = s.strip_prefix("*const") {
            if rest.is_empty() || rest.starts_with(|c: char| c.is_whitespace() || c == '*' || c == '&' || c == '[' || c == '(') {
                s = rest.trim_start();
            }
        } else if let Some(rest) = s.strip_prefix("*mut") {
            if rest.is_empty() || rest.starts_with(|c: char| c.is_whitespace() || c == '*' || c == '&' || c == '[' || c == '(') {
                s = rest.trim_start();
            }
        } else if let Some(rest) = s.strip_prefix("&mut") {
            if rest.is_empty() || rest.starts_with(|c: char| c.is_whitespace() || c == '*' || c == '&' || c == '[' || c == '(') {
                s = rest.trim_start();
            }
        } else if let Some(rest) = s.strip_prefix("&const") {
            if rest.is_empty() || rest.starts_with(|c: char| c.is_whitespace() || c == '*' || c == '&' || c == '[' || c == '(') {
                s = rest.trim_start();
            }
        } else if let Some(rest) = s.strip_prefix('&') {
            let mut rest = rest.trim_start();
            // Check for lifetime specifier e.g. 'a or 'static
            if rest.starts_with('\'') {
                let lifetime_end = rest
                    .char_indices()
                    .skip(1)
                    .find(|(_, c)| !c.is_ascii_alphanumeric() && *c != '_')
                    .map_or(rest.len(), |(idx, _)| idx);
                rest = rest[lifetime_end..].trim_start();
                if let Some(after_mut) = rest.strip_prefix("mut") {
                    if after_mut.is_empty() || after_mut.starts_with(|c: char| c.is_whitespace() || c == '*' || c == '&' || c == '[' || c == '(') {
                        rest = after_mut.trim_start();
                    }
                } else if let Some(after_const) = rest.strip_prefix("const") {
                    if after_const.is_empty() || after_const.starts_with(|c: char| c.is_whitespace() || c == '*' || c == '&' || c == '[' || c == '(') {
                        rest = after_const.trim_start();
                    }
                }
            }
            s = rest.trim_start();
        }

        // 3. Strip trait object prefix `dyn `
        if let Some(rest) = s.strip_prefix("dyn") {
            if rest.starts_with(|c: char| c.is_whitespace()) {
                s = rest.trim_start();
            }
        }

        // 4. Bracket handling: multiplicity suffixes and array wrappers.
        // Multiplicity suffixes (`Type[1..*]`, `Type[]`, `Type[0..1]`) are stripped BEFORE
        // array wrappers (`[T; N]`, `[T]`).
        // We find the matching opening '[' for the trailing ']' by walking backwards.
        if s.ends_with(']') {
            let mut depth = 0usize;
            let mut matching_open = None;
            for (idx, ch) in s.char_indices().rev() {
                match ch {
                    ']' => depth += 1,
                    '[' => {
                        depth -= 1;
                        if depth == 0 {
                            matching_open = Some(idx);
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if let Some(open_idx) = matching_open {
                if open_idx > 0 {
                    // Multiplicity suffix of preceding type: Type[1..*], Type[], Type[0..1], [u8][0..1]
                    s = s[..open_idx].trim_end();
                } else {
                    // Enclosing array wrapper: [T; N] or [T]
                    let inner = s[1..s.len() - 1].trim();
                    let mut inner_depth = 0usize;
                    let mut semi_pos = None;
                    for (idx, ch) in inner.char_indices() {
                        match ch {
                            '[' | '<' | '(' => inner_depth += 1,
                            ']' | '>' | ')' => inner_depth = inner_depth.saturating_sub(1),
                            ';' if inner_depth == 0 => {
                                semi_pos = Some(idx);
                                break;
                            }
                            _ => {}
                        }
                    }
                    s = if let Some(semi_idx) = semi_pos {
                        inner[..semi_idx].trim()
                    } else {
                        inner
                    };
                }
            }
        }

        if s == prev {
            break;
        }
    }

    // Detect when raw was non-empty but normalized to empty (e.g. bare modifiers like "&", "*const", "[]", "?").
    // Preserves the raw trimmed token so fail-closed validation can reject it with proper diagnostics.
    if s.is_empty() {
        return trimmed;
    }

    s
}

/// Deterministically splits comma-separated top-level type arguments inside generic angle brackets `<...>`.
/// Tracks angle bracket (`<` / `>`), parenthesis (`(` / `)`), and square bracket (`[` / `]`) nesting depth
/// so nested generics (e.g. `HashMap<String, Vec<u8>>` or `Result<Option<T>, E>`) are split only at depth 0.
///
/// Complies with Check 34 Zero Regex invariant using character/string scanning only.
pub fn split_top_level_type_args<'a>(args: &'a str) -> Vec<&'a str> {
    let mut s = args.trim();
    if s.starts_with('<') && s.ends_with('>') {
        let mut depth = 0usize;
        let mut closed_early = false;
        let last_idx = s.len() - 1;
        for (idx, ch) in s.char_indices() {
            match ch {
                '<' => depth += 1,
                '>' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 && idx < last_idx {
                        closed_early = true;
                        break;
                    }
                }
                _ => {}
            }
        }
        if depth == 0 && !closed_early {
            s = s[1..last_idx].trim();
        }
    }
    if s.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut angle_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut start = 0usize;

    for (idx, ch) in s.char_indices() {
        match ch {
            '<' => angle_depth += 1,
            '>' => angle_depth = angle_depth.saturating_sub(1),
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            ',' if angle_depth == 0 && paren_depth == 0 && bracket_depth == 0 => {
                let slice = s[start..idx].trim();
                if !slice.is_empty() {
                    result.push(slice);
                }
                start = idx + ch.len_utf8();
            }
            _ => {}
        }
    }

    let tail = s[start..].trim();
    if !tail.is_empty() {
        result.push(tail);
    }

    result
}

pub fn is_primitive_or_builtin_type(type_name: &str) -> bool {
    let clean = normalize_source_type(type_name);
    if clean.is_empty() {
        return false;
    }
    if clean == "()" {
        return true;
    }

    // Tuple unrolling: (T1, T2, ...)
    if clean.starts_with('(') && clean.ends_with(')') {
        let mut depth = 0usize;
        let mut closed_early = false;
        let last_idx = clean.len() - 1;
        for (idx, ch) in clean.char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 && idx < last_idx {
                        closed_early = true;
                        break;
                    }
                }
                _ => {}
            }
        }
        if depth == 0 && !closed_early {
            let inner = clean[1..last_idx].trim();
            if inner.is_empty() {
                return true;
            }
            let args = split_top_level_type_args(inner);
            return !args.is_empty() && args.into_iter().all(is_primitive_or_builtin_type);
        }
    }

    // Generic decomposition
    if let Some(lt_idx) = clean.find('<') {
        if let Some(gt_idx) = clean.rfind('>') {
            if gt_idx > lt_idx {
                if !clean[gt_idx + 1..].trim().is_empty() {
                    return false;
                }
                let outer = clean[..lt_idx].trim();
                let inner_str = &clean[lt_idx + 1..gt_idx];
                if !is_primitive_or_builtin_type(outer) {
                    return false;
                }
                let args = split_top_level_type_args(inner_str);
                if args.is_empty() {
                    return false;
                }
                return args.into_iter().all(is_primitive_or_builtin_type);
            }
        }
    }

    // Standard module prefixes
    if clean.starts_with("ScalarValues::")
        || clean.starts_with("ISQ::")
        || clean.starts_with("SI::")
        || clean.starts_with("std::")
        || clean.starts_with("core::")
        || clean.starts_with("alloc::")
    {
        return true;
    }

    let unqualified = clean.rsplit("::").next().unwrap_or(clean).trim();

    matches!(
        unqualified,
        "Boolean"
            | "bool"
            | "boolean"
            | "Integer"
            | "int"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "Natural"
            | "natural"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "Real"
            | "real"
            | "Float"
            | "float"
            | "double"
            | "num"
            | "f32"
            | "f64"
            | "String"
            | "string"
            | "str"
            | "char"
            | "void"
            | "unit"
            | "dynamic"
            | "Object"
            | "ScalarValues"
            | "Vec"
            | "Option"
            | "Result"
            | "Box"
            | "Arc"
            | "Rc"
            | "Cow"
            | "Mutex"
            | "RwLock"
            | "BTreeMap"
            | "BTreeSet"
            | "HashMap"
            | "HashSet"
            | "Path"
            | "PathBuf"
            | "Duration"
            | "Instant"
            | "SystemTime"
            | "DateTime"
            | "Error"
            | "List"
            | "Map"
            | "Set"
            | "Iterable"
            | "Future"
            | "Stream"
            | "Widget"
            | "StatefulWidget"
            | "StatelessWidget"
            | "State"
            | "BuildContext"
            | "Key"
            | "GlobalKey"
            | "LocalKey"
            | "ValueKey"
            | "VoidCallback"
            | "ValueChanged"
            | "ValueGetter"
            | "ValueSetter"
            | "WidgetBuilder"
            | "IndexedWidgetBuilder"
            | "TransitionBuilder"
            | "Color"
            | "IconData"
            | "EdgeInsets"
            | "EdgeInsetsGeometry"
            | "BoxConstraints"
            | "Size"
            | "Offset"
            | "Rect"
            | "TextStyle"
            | "ThemeData"
            | "TextEditingController"
            | "ScrollController"
            | "FocusNode"
            | "AnimationController"
    )
}

fn get_effective_direction(port_sym: &Symbol) -> String {
    let dir = port_sym.direction.as_deref().unwrap_or("inout");
    if port_sym.is_conjugated {
        match dir {
            "in" => "out".to_string(),
            "out" => "in".to_string(),
            other => other.to_string(),
        }
    } else {
        dir.to_string()
    }
}

fn normalize_id(id: &str) -> String {
    id.trim().replace('-', "_").to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_connection_passes() -> Result<(), Box<dyn std::error::Error>> {
        let pkg = PackageDef {
            name: "TestSys".to_string(),
            part_defs: vec![
                PartDef {
                    name: "Sensor".to_string(),
                    ports: vec![
                        PortDef {
                            name: "out_data".to_string(),
                            direction: "out".to_string(),
                            type_name: "RawData".to_string(),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                },
                PartDef {
                    name: "Computer".to_string(),
                    ports: vec![
                        PortDef {
                            name: "in_data".to_string(),
                            direction: "in".to_string(),
                            type_name: "RawData".to_string(),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                },
            ],
            connection_defs: vec![
                ConnectionDef {
                    name: "SensorToComputer".to_string(),
                    source_port: "Sensor.out_data".to_string(),
                    target_port: "Computer.in_data".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let mut validator = SemanticValidator::new(&pkg);
        validator.validate();
        assert!(!validator.has_errors(), "Expected no errors, got: {:?}", validator.errors());
        Ok(())
    }

    #[test]
    fn test_mismatched_connection_direction_fails() -> Result<(), Box<dyn std::error::Error>> {
        let pkg = PackageDef {
            name: "TestSys".to_string(),
            part_defs: vec![
                PartDef {
                    name: "Sensor".to_string(),
                    ports: vec![
                        PortDef {
                            name: "in_data".to_string(),
                            direction: "in".to_string(),
                            type_name: "RawData".to_string(),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                },
                PartDef {
                    name: "Computer".to_string(),
                    ports: vec![
                        PortDef {
                            name: "out_data".to_string(),
                            direction: "out".to_string(),
                            type_name: "RawData".to_string(),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                },
            ],
            connection_defs: vec![
                ConnectionDef {
                    name: "InvertedFlow".to_string(),
                    source_port: "Sensor.in_data".to_string(),
                    target_port: "Computer.out_data".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let mut validator = SemanticValidator::new(&pkg);
        validator.validate();
        assert!(validator.has_errors());
        let errors = validator.errors();
        assert_eq!(errors.len(), 1);
        assert!(errors.first().ok_or("missing first element of errors")?.message.contains("Inverted flow direction"));
        Ok(())
    }

    #[test]
    fn test_dangling_safety_realises_fails() -> Result<(), Box<dyn std::error::Error>> {
        let pkg = PackageDef {
            name: "SafetySys".to_string(),
            requirement_defs: vec![
                RequirementDef {
                    name: "Req_Safety_1".to_string(),
                    text: "Shall maintain velocity limit. @SafetyRealises(UCA-99)".to_string(),
                    ..Default::default()
                },
            ],
            constraint_defs: vec![
                ConstraintDef {
                    name: "UCA_01_Overspeed".to_string(),
                    expression: "v <= 60.0".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let mut validator = SemanticValidator::new(&pkg);
        validator.validate();
        assert!(validator.has_errors());
        let errors = validator.errors();
        assert!(errors.first().ok_or("missing first element of errors")?.message.contains("references undefined constraint/UCA 'UCA-99'"));
        Ok(())
    }

    #[test]
    fn test_validate_part_defs_with_cross_package_import() -> Result<(), Box<dyn std::error::Error>> {
        let pkg_a = PackageDef {
            name: "Subsystem_1".to_string(),
            part_defs: vec![
                PartDef {
                    name: "SystemVisionEngine".to_string(),
                    is_def: true,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let pkg_b = PackageDef {
            name: "Subsystem_2".to_string(),
            imports: vec![
                ImportDef {
                    path: "Subsystem_1".to_string(),
                    is_wildcard: true,
                    ..Default::default()
                },
            ],
            part_defs: vec![
                PartDef {
                    name: "v".to_string(),
                    is_def: false,
                    type_name: Some("SystemVisionEngine".to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let root = PackageDef {
            name: "".to_string(),
            packages: vec![pkg_a, pkg_b],
            ..Default::default()
        };

        let mut validator = SemanticValidator::new(&root);
        validator.validate();
        assert!(
            !validator.has_errors(),
            "Expected no validation errors with imported type, got: {:?}",
            validator.errors()
        );
        Ok(())
    }

    #[test]
    fn test_validate_part_defs_missing_type_fails() -> Result<(), Box<dyn std::error::Error>> {
        let pkg = PackageDef {
            name: "Subsystem_2".to_string(),
            part_defs: vec![
                PartDef {
                    name: "v".to_string(),
                    is_def: false,
                    type_name: Some("SystemVisionEngine".to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let mut validator = SemanticValidator::new(&pkg);
        validator.validate();
        assert!(validator.has_errors());
        let errors = validator.errors();
        assert!(errors
            .iter()
            .any(|e| e.message.contains("references undefined type 'SystemVisionEngine'")));
        Ok(())
    }

    #[test]
    fn test_validate_requirements_with_cross_package_import() -> Result<(), Box<dyn std::error::Error>> {
        let pkg_a = PackageDef {
            name: "Subsystem_1".to_string(),
            part_defs: vec![
                PartDef {
                    name: "SafetyController".to_string(),
                    is_def: true,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let pkg_b = PackageDef {
            name: "Subsystem_2".to_string(),
            imports: vec![
                ImportDef {
                    path: "Subsystem_1".to_string(),
                    is_wildcard: true,
                    ..Default::default()
                },
            ],
            requirement_defs: vec![
                RequirementDef {
                    name: "Req_Ctrl".to_string(),
                    text: "Shall engage safety controller.".to_string(),
                    satisfied_by: vec!["SafetyController".to_string()],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let root = PackageDef {
            name: "".to_string(),
            packages: vec![pkg_a, pkg_b],
            ..Default::default()
        };

        let mut validator = SemanticValidator::new(&root);
        validator.validate();
        assert!(
            !validator.has_errors(),
            "Expected no validation errors, got: {:?}",
            validator.errors()
        );
        Ok(())
    }

    #[test]
    fn test_subpackage_connections_validated_for_dangling_ports() -> Result<(), Box<dyn std::error::Error>> {
        let subpkg = PackageDef {
            name: "NestedSubsystem".to_string(),
            part_defs: vec![
                PartDef {
                    name: "Worker".to_string(),
                    ports: vec![
                        PortDef {
                            name: "out_stream".to_string(),
                            direction: "out".to_string(),
                            type_name: "StreamData".to_string(),
                            ..Default::default()
                        },
                    ],
                    connections: vec![
                        ConnectionDef {
                            name: "DanglingSubpartConn".to_string(),
                            source_port: "out_stream".to_string(),
                            target_port: "Worker.undefined_target_port".to_string(),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                },
            ],
            connection_defs: vec![
                ConnectionDef {
                    name: "DanglingPkgConn".to_string(),
                    source_port: "Worker.out_stream".to_string(),
                    target_port: "UndefinedPart.in_stream".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let root = PackageDef {
            name: "RootSystem".to_string(),
            packages: vec![subpkg],
            ..Default::default()
        };

        let mut validator = SemanticValidator::new(&root);
        validator.validate();
        assert!(
            validator.has_errors(),
            "Expected validation errors for dangling ports inside nested subpackage, but validator reported no errors"
        );
        let errors = validator.errors();
        assert!(
            errors.iter().any(|e| e.message.contains("references undefined target port 'Worker.undefined_target_port'")),
            "Expected error for Worker.undefined_target_port, got: {:?}",
            errors
        );
        assert!(
            errors.iter().any(|e| e.message.contains("references undefined target port 'UndefinedPart.in_stream'")),
            "Expected error for UndefinedPart.in_stream, got: {:?}",
            errors
        );
        Ok(())
    }

    #[test]
    fn test_issue_461_normalize_source_type() {
        assert_eq!(normalize_source_type("&str"), "str");
        assert_eq!(normalize_source_type("&mut Vec<u8>"), "Vec<u8>");
        assert_eq!(normalize_source_type("&const i32"), "i32");
        assert_eq!(normalize_source_type("&&str"), "str");
        assert_eq!(normalize_source_type("&'a str"), "str");
        assert_eq!(normalize_source_type("&'static str"), "str");
        assert_eq!(normalize_source_type("&'a mut [u8]"), "u8");
        assert_eq!(normalize_source_type("*const u8"), "u8");
        assert_eq!(normalize_source_type("*mut u8"), "u8");
        assert_eq!(normalize_source_type("[u8; 32]"), "u8");
        assert_eq!(normalize_source_type("[u8]"), "u8");
        assert_eq!(normalize_source_type("[[u8; 4]; 8]"), "u8");
        assert_eq!(normalize_source_type("Sensor[1..*]"), "Sensor");
        assert_eq!(normalize_source_type("u8[]"), "u8");
        assert_eq!(normalize_source_type("Sensor[0..1]"), "Sensor");
        assert_eq!(normalize_source_type("String?"), "String");
        assert_eq!(normalize_source_type("Widget?"), "Widget");
        assert_eq!(normalize_source_type("BuildContext?"), "BuildContext");
        assert_eq!(normalize_source_type("&'a mut [Option<String>?; 10]"), "Option<String>");
    }

    #[test]
    fn test_issue_461_split_top_level_type_args() {
        assert_eq!(split_top_level_type_args("u8"), vec!["u8"]);
        assert_eq!(split_top_level_type_args("String, i32"), vec!["String", "i32"]);
        assert_eq!(split_top_level_type_args("<String, i32>"), vec!["String", "i32"]);
        assert_eq!(
            split_top_level_type_args("HashMap<String, Vec<u8>>, Result<Option<T>, E>"),
            vec!["HashMap<String, Vec<u8>>", "Result<Option<T>, E>"]
        );
        assert_eq!(
            split_top_level_type_args("(A, B), [C; 2]"),
            vec!["(A, B)", "[C; 2]"]
        );
        assert!(split_top_level_type_args("").is_empty());
        assert!(split_top_level_type_args("< >").is_empty());
    }

    #[test]
    fn test_issue_461_is_primitive_or_builtin_type_scalars() {
        let scalars = vec![
            "void", "unit", "()", "double", "num", "char", "dynamic", "Object",
            "bool", "Boolean", "int", "Integer", "i8", "i16", "i32", "i64", "i128", "isize",
            "u8", "u16", "u32", "u64", "u128", "usize", "float", "Float", "f32", "f64",
            "String", "string", "str", "ScalarValues",
        ];
        for ty in scalars {
            assert!(
                is_primitive_or_builtin_type(ty),
                "Expected scalar '{}' to be recognized as primitive or builtin",
                ty
            );
        }
    }

    #[test]
    fn test_issue_461_is_primitive_or_builtin_type_rust_containers() {
        let containers = vec![
            "Vec", "Option", "Result", "Box", "Arc", "Rc", "Cow",
            "Mutex", "RwLock", "BTreeMap", "BTreeSet", "HashMap", "HashSet",
            "Path", "PathBuf", "Duration", "Instant", "SystemTime", "DateTime", "Error",
        ];
        for ty in containers {
            assert!(
                is_primitive_or_builtin_type(ty),
                "Expected Rust container '{}' to be recognized as builtin",
                ty
            );
        }
    }

    #[test]
    fn test_issue_461_is_primitive_or_builtin_type_dart_collections_and_async() {
        let types = vec!["List", "Map", "Set", "Iterable", "Future", "Stream"];
        for ty in types {
            assert!(
                is_primitive_or_builtin_type(ty),
                "Expected Dart/Async type '{}' to be recognized as builtin",
                ty
            );
        }
    }

    #[test]
    fn test_issue_461_is_primitive_or_builtin_type_flutter_framework() {
        let flutter_types = vec![
            "Widget", "StatefulWidget", "StatelessWidget", "State", "BuildContext",
            "Key", "GlobalKey", "LocalKey", "ValueKey", "VoidCallback", "ValueChanged",
            "ValueGetter", "ValueSetter", "WidgetBuilder", "IndexedWidgetBuilder",
            "TransitionBuilder", "Color", "IconData", "EdgeInsets", "EdgeInsetsGeometry",
            "BoxConstraints", "Size", "Offset", "Rect", "TextStyle", "ThemeData",
            "TextEditingController", "ScrollController", "FocusNode", "AnimationController",
        ];
        for ty in flutter_types {
            assert!(
                is_primitive_or_builtin_type(ty),
                "Expected Flutter UI type '{}' to be recognized as builtin",
                ty
            );
        }
    }

    #[test]
    fn test_issue_461_is_primitive_or_builtin_type_module_prefixes() {
        let prefixed_types = vec![
            "ScalarValues::Boolean",
            "ISQ::LengthValue",
            "SI::m",
            "std::path::PathBuf",
            "std::collections::HashMap",
            "std::sync::Arc",
            "std::time::Duration",
            "core::option::Option",
            "core::result::Result",
            "alloc::vec::Vec",
            "alloc::string::String",
        ];
        for ty in prefixed_types {
            assert!(
                is_primitive_or_builtin_type(ty),
                "Expected prefixed type '{}' to be recognized as builtin",
                ty
            );
        }
    }

    #[test]
    fn test_issue_461_is_primitive_or_builtin_type_composite_generics() {
        assert!(is_primitive_or_builtin_type("Vec<u8>"));
        assert!(is_primitive_or_builtin_type("Option<String>"));
        assert!(is_primitive_or_builtin_type("Result<i32, Error>"));
        assert!(is_primitive_or_builtin_type("HashMap<String, Vec<u8>>"));
        assert!(is_primitive_or_builtin_type("List<Widget>"));
        assert!(is_primitive_or_builtin_type("Future<void>"));
        assert!(is_primitive_or_builtin_type("Stream<String>"));
    }

    #[test]
    fn test_issue_461_is_primitive_or_builtin_type_negative_unknown_types() {
        assert!(!is_primitive_or_builtin_type("UnknownCustomSensor"));
        assert!(!is_primitive_or_builtin_type("Vec<UnknownCustomSensor>"));
        assert!(!is_primitive_or_builtin_type("HashMap<String, UnknownCustomSensor>"));
        assert!(!is_primitive_or_builtin_type("Future<UnknownCustomSensor>"));
    }

    #[test]
    fn test_issue_461_is_resolvable_type_composite_and_generic_builtins() {
        let pkg = PackageDef::default();
        let validator = SemanticValidator::new(&pkg);

        // Plain scalars and framework types
        assert!(validator.is_resolvable_type("", "i32"));
        assert!(validator.is_resolvable_type("", "f64"));
        assert!(validator.is_resolvable_type("", "String"));
        assert!(validator.is_resolvable_type("", "Widget"));
        assert!(validator.is_resolvable_type("", "BuildContext"));
        assert!(validator.is_resolvable_type("", "StatefulWidget"));
        assert!(validator.is_resolvable_type("", "()"));

        // Wrapped types
        assert!(validator.is_resolvable_type("", "&Widget"));
        assert!(validator.is_resolvable_type("", "&mut String"));
        assert!(validator.is_resolvable_type("", "*const i32"));
        assert!(validator.is_resolvable_type("", "Widget?"));
        assert!(validator.is_resolvable_type("", "[u8; 32]"));
        assert!(validator.is_resolvable_type("", "&'a str"));

        // Generic builtins wrapping builtins
        assert!(validator.is_resolvable_type("", "Option<String>"));
        assert!(validator.is_resolvable_type("", "Vec<u8>"));
        assert!(validator.is_resolvable_type("", "Result<i32, String>"));
        assert!(validator.is_resolvable_type("", "HashMap<String, Vec<u8>>"));
        assert!(validator.is_resolvable_type("", "Option<Widget>?"));
    }

    #[test]
    fn test_issue_461_is_resolvable_type_user_defined_symbols_in_scope() {
        let pkg = PackageDef {
            name: "Subsystem".to_string(),
            part_defs: vec![
                PartDef {
                    name: "SensorNode".to_string(),
                    is_def: true,
                    ..Default::default()
                },
                PartDef {
                    name: "SafetyController".to_string(),
                    is_def: true,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let validator = SemanticValidator::new(&pkg);

        // Direct user-defined symbol
        assert!(validator.is_resolvable_type("Subsystem", "SensorNode"));
        assert!(validator.is_resolvable_type("Subsystem", "Subsystem::SensorNode"));

        // Generic container wrapping user-defined symbol
        assert!(validator.is_resolvable_type("Subsystem", "Vec<SensorNode>"));
        assert!(validator.is_resolvable_type("Subsystem", "Option<SafetyController>"));
        assert!(validator.is_resolvable_type("Subsystem", "Result<SensorNode, String>"));
        assert!(validator.is_resolvable_type("Subsystem", "HashMap<String, Vec<SensorNode>>"));

        // Wrappers with user-defined symbol
        assert!(validator.is_resolvable_type("Subsystem", "&Vec<SensorNode>"));
        assert!(validator.is_resolvable_type("Subsystem", "Option<SensorNode>?"));
        assert!(validator.is_resolvable_type("Subsystem", "[SensorNode; 4]"));
    }

    #[test]
    fn test_issue_461_is_resolvable_type_fail_closed_negative_mutations() {
        let pkg = PackageDef {
            name: "Subsystem".to_string(),
            part_defs: vec![
                PartDef {
                    name: "SensorNode".to_string(),
                    is_def: true,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let validator = SemanticValidator::new(&pkg);

        // Undefined atomic types
        assert!(!validator.is_resolvable_type("Subsystem", "UndefinedType"));
        assert!(!validator.is_resolvable_type("Subsystem", "&UndefinedType"));
        assert!(!validator.is_resolvable_type("Subsystem", "UndefinedType?"));

        // Generic container with undefined inner argument
        assert!(!validator.is_resolvable_type("Subsystem", "Vec<UndefinedType>"));
        assert!(!validator.is_resolvable_type("Subsystem", "Option<UndefinedType>"));
        assert!(!validator.is_resolvable_type("Subsystem", "Result<SensorNode, UndefinedError>"));
        assert!(!validator.is_resolvable_type("Subsystem", "Result<UndefinedNode, String>"));
        assert!(!validator.is_resolvable_type("Subsystem", "HashMap<String, Vec<UndefinedType>>"));

        // Undefined outer container with valid inner argument
        assert!(!validator.is_resolvable_type("Subsystem", "UndefinedContainer<String>"));
        assert!(!validator.is_resolvable_type("Subsystem", "UndefinedContainer<SensorNode>"));

        // Empty generic parameter
        assert!(!validator.is_resolvable_type("Subsystem", "Vec<>"));
    }

    #[test]
    fn test_issue_461_validate_part_defs_with_composite_generic_types_passes() {
        let pkg = PackageDef {
            name: "Platform".to_string(),
            part_defs: vec![
                PartDef {
                    name: "SensorNode".to_string(),
                    is_def: true,
                    ..Default::default()
                },
                PartDef {
                    name: "nodes".to_string(),
                    is_def: false,
                    type_name: Some("Vec<SensorNode>".to_string()),
                    ..Default::default()
                },
                PartDef {
                    name: "active_node".to_string(),
                    is_def: false,
                    type_name: Some("Option<SensorNode>".to_string()),
                    ..Default::default()
                },
                PartDef {
                    name: "view".to_string(),
                    is_def: false,
                    type_name: Some("Widget".to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let mut validator = SemanticValidator::new(&pkg);
        validator.validate();
        assert!(
            !validator.has_errors(),
            "Expected zero validation errors for valid composite types, got: {:?}",
            validator.errors()
        );
    }

    #[test]
    fn test_issue_461_validate_part_defs_with_undefined_generic_inner_fails() {
        let pkg = PackageDef {
            name: "Platform".to_string(),
            part_defs: vec![
                PartDef {
                    name: "bad_nodes".to_string(),
                    is_def: false,
                    type_name: Some("Vec<GhostType>".to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let mut validator = SemanticValidator::new(&pkg);
        validator.validate();
        assert!(
            validator.has_errors(),
            "Expected validation error for undefined generic inner type"
        );
        let errors = validator.errors();
        assert!(
            errors.iter().any(|e| e.message.contains("Part 'bad_nodes' references undefined type 'Vec<GhostType>'")),
            "Expected error message referencing undefined type, got: {:?}",
            errors
        );
    }

    #[test]
    fn test_issue_461_validate_requirements_with_composite_generic_attributes_passes() {
        let pkg = PackageDef {
            name: "Safety".to_string(),
            part_defs: vec![
                PartDef {
                    name: "Guard".to_string(),
                    is_def: true,
                    ..Default::default()
                },
            ],
            requirement_defs: vec![
                RequirementDef {
                    name: "Req_Safety".to_string(),
                    text: "Shall maintain logs.".to_string(),
                    attributes: vec![
                        AttributeDef {
                            name: "log_entries".to_string(),
                            type_name: "Vec<String>".to_string(),
                            ..Default::default()
                        },
                        AttributeDef {
                            name: "guard_ref".to_string(),
                            type_name: "Option<Guard>".to_string(),
                            ..Default::default()
                        },
                    ],
                    satisfied_by: vec!["Guard".to_string()],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let mut validator = SemanticValidator::new(&pkg);
        validator.validate();
        assert!(
            !validator.has_errors(),
            "Expected zero validation errors for requirement with generic attributes, got: {:?}",
            validator.errors()
        );
    }

    // =========================================================================
    // Issue #461 Iteration 2: Comprehensive Edge Case Regression Test Suite
    // Covers all 8 edge cases discovered during Iteration 1 adversarial review.
    // =========================================================================

    #[test]
    fn test_issue_461_iter2_edge_1_trailing_tokens_after_generics_fail_closed() {
        let pkg = PackageDef {
            name: "TestScope".to_string(),
            part_defs: vec![
                PartDef {
                    name: "ValidPart".to_string(),
                    is_def: true,
                    ..Default::default()
                },
                PartDef {
                    name: "bad_part_trailing_undefined".to_string(),
                    is_def: false,
                    type_name: Some("Option<String> UndefinedType".to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let mut validator = SemanticValidator::new(&pkg);
        validator.validate();

        // 1. Full model validation MUST fail closed
        assert!(
            validator.has_errors(),
            "Expected validation error for part referencing 'Option<String> UndefinedType'"
        );
        let errors = validator.errors();
        assert!(
            errors.iter().any(|e| e.message.contains("references undefined type 'Option<String> UndefinedType'")),
            "Expected diagnostic error message for trailing undefined token, got: {:?}",
            errors
        );

        // 2. Direct is_resolvable_type probes MUST fail closed
        let validator = SemanticValidator::new(&pkg);
        assert!(!validator.is_resolvable_type("TestScope", "Option<String> UndefinedType"));
        assert!(!validator.is_resolvable_type("TestScope", "Vec<u8> junk"));
        assert!(!validator.is_resolvable_type("TestScope", "HashMap<String, i32> trailing"));
        assert!(!validator.is_resolvable_type("TestScope", "Vec<ValidPart> GhostPart"));

        // 3. Direct is_primitive_or_builtin_type probes MUST fail closed
        assert!(!is_primitive_or_builtin_type("Option<String> UndefinedType"));
        assert!(!is_primitive_or_builtin_type("Vec<u8> junk"));
        assert!(!is_primitive_or_builtin_type("HashMap<String, i32> trailing"));
    }

    #[test]
    fn test_issue_461_edge_1_trailing_tokens_after_generics_fail_closed() {
        test_issue_461_iter2_edge_1_trailing_tokens_after_generics_fail_closed();
    }

    #[test]
    fn test_issue_461_iter2_edge_2_tuples_with_generics_resolve() {
        let pkg = PackageDef {
            name: "TestScope".to_string(),
            part_defs: vec![
                PartDef {
                    name: "ValidPart".to_string(),
                    is_def: true,
                    ..Default::default()
                },
                PartDef {
                    name: "tuple_part".to_string(),
                    is_def: false,
                    type_name: Some("(ValidPart, Vec<String>)".to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        // 1. Full model validation MUST pass for valid generic tuples
        let mut validator = SemanticValidator::new(&pkg);
        validator.validate();
        assert!(
            !validator.has_errors(),
            "Expected zero validation errors for valid tuple with generics, got: {:?}",
            validator.errors()
        );

        // 2. Direct is_resolvable_type probes
        let validator = SemanticValidator::new(&pkg);
        assert!(validator.is_resolvable_type("TestScope", "(ValidPart, Vec<String>)"));
        assert!(validator.is_resolvable_type("TestScope", "(i32, Option<ValidPart>)"));
        assert!(validator.is_resolvable_type("TestScope", "(Vec<ValidPart>, HashMap<String, Option<i32>>)"));

        // 3. Negative mutation: tuples containing undefined types MUST fail closed
        assert!(!validator.is_resolvable_type("TestScope", "(ValidPart, Vec<UndefinedPart>)"));
        assert!(!validator.is_resolvable_type("TestScope", "(UndefinedPart, Vec<String>)"));
        assert!(!validator.is_resolvable_type("TestScope", "(i32, Option<GhostSymbol>)"));
    }

    #[test]
    fn test_issue_461_edge_2_tuples_with_generics_resolve() {
        test_issue_461_iter2_edge_2_tuples_with_generics_resolve();
    }

    #[test]
    fn test_issue_461_iter2_edge_3_bare_modifiers_and_empty_types_fail_closed() {
        let pkg = PackageDef::default();
        let validator = SemanticValidator::new(&pkg);

        // 1. Bare modifiers in is_resolvable_type MUST return false
        assert!(!validator.is_resolvable_type("", "&"));
        assert!(!validator.is_resolvable_type("", "*const"));
        assert!(!validator.is_resolvable_type("", "*mut"));
        assert!(!validator.is_resolvable_type("", "&mut"));
        assert!(!validator.is_resolvable_type("", "&const"));
        assert!(!validator.is_resolvable_type("", "[]"));
        assert!(!validator.is_resolvable_type("", "?"));
        assert!(!validator.is_resolvable_type("", ""));
        assert!(!validator.is_resolvable_type("", "   "));

        // 2. Bare modifiers in is_primitive_or_builtin_type MUST return false
        assert!(!is_primitive_or_builtin_type("&"));
        assert!(!is_primitive_or_builtin_type("*const"));
        assert!(!is_primitive_or_builtin_type("*mut"));
        assert!(!is_primitive_or_builtin_type("&mut"));
        assert!(!is_primitive_or_builtin_type("&const"));
        assert!(!is_primitive_or_builtin_type("[]"));
        assert!(!is_primitive_or_builtin_type("?"));
        assert!(!is_primitive_or_builtin_type(""));
        assert!(!is_primitive_or_builtin_type("   "));

        // 3. Generics wrapping bare modifiers MUST fail closed
        assert!(!validator.is_resolvable_type("", "Option<&>"));
        assert!(!validator.is_resolvable_type("", "Option<[]>"));
        assert!(!validator.is_resolvable_type("", "Vec<?>"));
        assert!(!validator.is_resolvable_type("", "Result<&, []>"));

        // 4. Model validation probe with bare modifier part
        let invalid_pkg = PackageDef {
            name: "InvalidPkg".to_string(),
            part_defs: vec![
                PartDef {
                    name: "bad_mod_part".to_string(),
                    is_def: false,
                    type_name: Some("&".to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let mut invalid_validator = SemanticValidator::new(&invalid_pkg);
        invalid_validator.validate();
        assert!(
            invalid_validator.has_errors(),
            "Expected validation error for part with bare modifier type '&'"
        );
    }

    #[test]
    fn test_issue_461_edge_3_bare_modifiers_and_empty_types_fail_closed() {
        test_issue_461_iter2_edge_3_bare_modifiers_and_empty_types_fail_closed();
    }

    #[test]
    fn test_issue_461_iter2_edge_4_pointer_array_wrappers_normalize_and_resolve() {
        let pkg = PackageDef {
            name: "TestScope".to_string(),
            part_defs: vec![
                PartDef {
                    name: "ValidPart".to_string(),
                    is_def: true,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        // 1. Normalization probes for pointer array wrappers
        assert_eq!(normalize_source_type("*const[ValidPart; 10]"), "ValidPart");
        assert_eq!(normalize_source_type("*mut[u8; 16]"), "u8");
        assert_eq!(normalize_source_type("&mut[ValidPart; 4]"), "ValidPart");
        assert_eq!(normalize_source_type("&const[i32; 8]"), "i32");
        assert_eq!(normalize_source_type("*const[TotallyUndefined; 10]"), "TotallyUndefined");

        // 2. Resolution for valid pointer-wrapped arrays
        let validator = SemanticValidator::new(&pkg);
        assert!(validator.is_resolvable_type("TestScope", "*const[ValidPart; 10]"));
        assert!(validator.is_resolvable_type("TestScope", "*mut[ValidPart; 10]"));
        assert!(validator.is_resolvable_type("TestScope", "&mut[ValidPart; 10]"));
        assert!(validator.is_resolvable_type("TestScope", "&const[ValidPart; 10]"));
        assert!(validator.is_resolvable_type("TestScope", "*const[u8; 32]"));

        // 3. Negative mutations: pointer-wrapped arrays with undefined types MUST fail closed
        assert!(!validator.is_resolvable_type("TestScope", "*const[TotallyUndefined; 10]"));
        assert!(!validator.is_resolvable_type("TestScope", "*mut[TotallyUndefined; 10]"));
        assert!(!validator.is_resolvable_type("TestScope", "&mut[TotallyUndefined; 10]"));
        assert!(!validator.is_resolvable_type("TestScope", "&const[TotallyUndefined; 10]"));
    }

    #[test]
    fn test_issue_461_edge_4_pointer_array_wrappers_normalize_and_resolve() {
        test_issue_461_iter2_edge_4_pointer_array_wrappers_normalize_and_resolve();
    }

    #[test]
    fn test_issue_461_iter2_edge_5_slice_multiplicity_normalization_and_resolution() {
        let pkg = PackageDef {
            name: "TestScope".to_string(),
            part_defs: vec![
                PartDef {
                    name: "SensorNode".to_string(),
                    is_def: true,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        // 1. Normalization of slices with multiplicity suffixes
        assert_eq!(normalize_source_type("[u8][0..1]"), "u8");
        assert_eq!(normalize_source_type("[u8][1..*]"), "u8");
        assert_eq!(normalize_source_type("[u8][]"), "u8");
        assert_eq!(normalize_source_type("[SensorNode][0..1]"), "SensorNode");
        assert_eq!(normalize_source_type("[SensorNode][1..*]"), "SensorNode");
        assert_eq!(normalize_source_type("&[u8][0..1]"), "u8");

        // 2. Resolution of valid slices with multiplicity
        let validator = SemanticValidator::new(&pkg);
        assert!(validator.is_resolvable_type("TestScope", "[u8][0..1]"));
        assert!(validator.is_resolvable_type("TestScope", "[u8][1..*]"));
        assert!(validator.is_resolvable_type("TestScope", "[SensorNode][0..1]"));
        assert!(validator.is_resolvable_type("TestScope", "[SensorNode][1..*]"));

        // 3. Negative mutation: slices with undefined inner type MUST fail closed
        assert!(!validator.is_resolvable_type("TestScope", "[UndefinedPart][0..1]"));
        assert!(!validator.is_resolvable_type("TestScope", "[UndefinedPart][1..*]"));
    }

    #[test]
    fn test_issue_461_edge_5_slice_multiplicity_normalization_and_resolution() {
        test_issue_461_iter2_edge_5_slice_multiplicity_normalization_and_resolution();
    }

    #[test]
    fn test_issue_461_iter2_edge_6_split_top_level_type_args_multiple_bracket_groups() {
        // 1. Multiple top-level angle bracket groups split cleanly
        assert_eq!(split_top_level_type_args("<A>, <B>"), vec!["<A>", "<B>"]);
        assert_eq!(split_top_level_type_args("<A, B>, <C, D>"), vec!["<A, B>", "<C, D>"]);
        assert_eq!(
            split_top_level_type_args("<A>, <B, <C, D>>, <E>"),
            vec!["<A>", "<B, <C, D>>", "<E>"]
        );

        // 2. Single enclosing angle brackets are unwrapped cleanly
        assert_eq!(split_top_level_type_args("<A, B>"), vec!["A", "B"]);
        assert_eq!(split_top_level_type_args("<<A, B>>"), vec!["<A, B>"]);

        // 3. Standard comma-separated arguments remain intact
        assert_eq!(split_top_level_type_args("A, B, C"), vec!["A", "B", "C"]);
        assert_eq!(
            split_top_level_type_args("HashMap<String, Vec<u8>>, Result<Option<T>, E>"),
            vec!["HashMap<String, Vec<u8>>", "Result<Option<T>, E>"]
        );
    }

    #[test]
    fn test_issue_461_edge_6_split_top_level_type_args_multiple_bracket_groups() {
        test_issue_461_iter2_edge_6_split_top_level_type_args_multiple_bracket_groups();
    }

    #[test]
    fn test_issue_461_iter2_edge_7_trait_objects_with_dyn_resolve() {
        let pkg = PackageDef {
            name: "TestScope".to_string(),
            part_defs: vec![
                PartDef {
                    name: "ValidPart".to_string(),
                    is_def: true,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        // 1. Normalization of trait objects with dyn prefix
        assert_eq!(normalize_source_type("dyn Error"), "Error");
        assert_eq!(normalize_source_type("&dyn Error"), "Error");
        assert_eq!(normalize_source_type("Box<dyn Error>"), "Box<dyn Error>");
        assert_eq!(
            normalize_source_type("Result<Option<HashMap<String, Vec<u8>>>, Box<dyn Error>>"),
            "Result<Option<HashMap<String, Vec<u8>>>, Box<dyn Error>>"
        );

        // 2. Resolution of standard trait objects
        let validator = SemanticValidator::new(&pkg);
        assert!(validator.is_resolvable_type("", "Box<dyn Error>"));
        assert!(validator.is_resolvable_type("", "Arc<dyn Error>"));
        assert!(validator.is_resolvable_type("", "Result<Option<HashMap<String, Vec<u8>>>, Box<dyn Error>>"));
        assert!(validator.is_resolvable_type("TestScope", "Result<ValidPart, Box<dyn Error>>"));

        // 3. Negative mutation: trait objects with undefined trait MUST fail closed
        assert!(!validator.is_resolvable_type("", "Box<dyn UndefinedTrait>"));
        assert!(!validator.is_resolvable_type("TestScope", "Result<ValidPart, Box<dyn UndefinedTrait>>"));
    }

    #[test]
    fn test_issue_461_edge_7_trait_objects_with_dyn_resolve() {
        test_issue_461_iter2_edge_7_trait_objects_with_dyn_resolve();
    }

    #[test]
    fn test_issue_461_iter2_edge_8_tuples_in_primitive_and_generic_containers() {
        let pkg = PackageDef {
            name: "TestScope".to_string(),
            part_defs: vec![
                PartDef {
                    name: "ValidPart".to_string(),
                    is_def: true,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        // 1. Direct tuple resolution in is_primitive_or_builtin_type
        assert!(is_primitive_or_builtin_type("(u32, bool)"));
        assert!(is_primitive_or_builtin_type("(i32, f64, String)"));
        assert!(is_primitive_or_builtin_type("Vec<(u32, bool)>"));
        assert!(is_primitive_or_builtin_type("Option<(i32, f64)>"));
        assert!(is_primitive_or_builtin_type("HashMap<String, (u8, u16)>"));

        // 2. Negative mutations in is_primitive_or_builtin_type
        assert!(!is_primitive_or_builtin_type("(u32, UnknownType)"));
        assert!(!is_primitive_or_builtin_type("Vec<(u32, UnknownType)>"));

        // 3. Direct resolution in is_resolvable_type with mixed scope symbols
        let validator = SemanticValidator::new(&pkg);
        assert!(validator.is_resolvable_type("", "Vec<(u32, bool)>"));
        assert!(validator.is_resolvable_type("TestScope", "Vec<(ValidPart, bool)>"));
        assert!(validator.is_resolvable_type("TestScope", "Option<(ValidPart, String)>"));
        assert!(!validator.is_resolvable_type("TestScope", "Vec<(UndefinedPart, bool)>"));
    }

    #[test]
    fn test_issue_461_edge_8_tuples_in_primitive_and_generic_containers() {
        test_issue_461_iter2_edge_8_tuples_in_primitive_and_generic_containers();
    }

    #[test]
    fn test_issue_461_iter2_all_iteration_1_edge_cases_comprehensive_matrix() {
        let pkg = PackageDef {
            name: "MatrixScope".to_string(),
            part_defs: vec![
                PartDef {
                    name: "DeclaredSensor".to_string(),
                    is_def: true,
                    ..Default::default()
                },
                PartDef {
                    name: "DeclaredActuator".to_string(),
                    is_def: true,
                    ..Default::default()
                },
                // Edge Case 2: Tuple with generic
                PartDef {
                    name: "sensor_tuple".to_string(),
                    is_def: false,
                    type_name: Some("(DeclaredSensor, Vec<String>)".to_string()),
                    ..Default::default()
                },
                // Edge Case 4: Pointer array wrapper
                PartDef {
                    name: "pointer_array".to_string(),
                    is_def: false,
                    type_name: Some("*const[DeclaredActuator; 8]".to_string()),
                    ..Default::default()
                },
                // Edge Case 5: Slice multiplicity
                PartDef {
                    name: "slice_multiplicity".to_string(),
                    is_def: false,
                    type_name: Some("[DeclaredSensor][1..*]".to_string()),
                    ..Default::default()
                },
                // Edge Case 7: Trait object
                PartDef {
                    name: "error_handler".to_string(),
                    is_def: false,
                    type_name: Some("Box<dyn Error>".to_string()),
                    ..Default::default()
                },
                // Edge Case 8: Generic container wrapping primitive tuple
                PartDef {
                    name: "coordinate_stream".to_string(),
                    is_def: false,
                    type_name: Some("Stream<(f64, f64)>".to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        // All valid edge cases in the matrix MUST validate cleanly
        let mut validator = SemanticValidator::new(&pkg);
        validator.validate();
        assert!(
            !validator.has_errors(),
            "Expected zero validation errors for comprehensive edge cases matrix, got: {:?}",
            validator.errors()
        );
    }

    #[test]
    fn test_issue_461_all_iteration_1_edge_cases_comprehensive_matrix() {
        test_issue_461_iter2_all_iteration_1_edge_cases_comprehensive_matrix();
    }
}



