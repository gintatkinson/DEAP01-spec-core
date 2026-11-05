//! Check 31: Dual-Schema SSOT Parity Gate (`check_dual_schema_ssot_parity`).
//!
//! Safety & Traceability: Realises: [REQ-DUAL-SCHEMA-SSOT-PARITY], [REQ-DUAL-SCHEMA-SSOT-PARITY/PACKAGE-SCOPING]
//! Contract: Enforces bidirectional AST parity across 11 SysML v2 node kinds and design `.digest.json` sidecar presence.
//! Bounds & Faults: Package-qualified BTreeMap comparison; fails closed on any structural divergence or missing sidecar.

use super::parity_check_coverage::parse_use_case_defs;
use super::parity_gates::ParityOutcome;
use deap_core::sysml_ast::{
    find_matching_brace, mask_string_literals, parse_action_defs, parse_attribute_defs,
    parse_calc_defs, parse_connection_defs, parse_constraint_defs, parse_part_defs,
    parse_port_defs, parse_requirement_defs, parse_state_defs, parse_test_case_defs,
    strip_sysml_comments, ActionDef, AttributeDef, CalcDef, ConnectionDef, ConstraintDef, PartDef,
    PortDef, RequirementDef, StateDef, TestCaseDef,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Collects package-qualified slices `(package_path, shallow_package_body)` from SysML v2 source text.
/// Realises: [REQ-DUAL-SCHEMA-SSOT-PARITY/PACKAGE-SCOPING]
pub(crate) fn collect_qualified_package_bodies(
    content: &str,
    parent_pkg: &str,
    out: &mut Vec<(String, String)>,
) {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut remainder = clean.as_bytes().to_vec();
    let mut idx = 0;

    while let Some(pos) = masked.get(idx..).and_then(|s| s.find("package")) {
        let pkg_pos = idx + pos;
        let before_ok = pkg_pos == 0
            || !masked.as_bytes().get(pkg_pos - 1).copied().is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_');
        let after_ok = !masked.as_bytes().get(pkg_pos + 7).copied().is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_');
        if !before_ok || !after_ok {
            idx = pkg_pos + 7;
            continue;
        }
        let mut curr = pkg_pos + 7;
        curr += masked.get(curr..).unwrap_or("").bytes().take_while(|b| b.is_ascii_whitespace()).count();
        let name_len = masked.get(curr..).unwrap_or("").bytes().take_while(|b| b.is_ascii_alphanumeric() || *b == b'_').count();
        let name = masked.get(curr..curr + name_len).unwrap_or("");
        if name.is_empty() {
            idx = pkg_pos + 7;
            continue;
        }
        curr += name_len;
        curr += masked.get(curr..).unwrap_or("").bytes().take_while(|b| b.is_ascii_whitespace()).count();
        if masked.get(curr..).is_some_and(|s| s.starts_with('{')) {
            if let Some(close_pos) = find_matching_brace(&clean, curr) {
                let body = clean.get(curr + 1..close_pos).unwrap_or("");
                let qualified_pkg = if parent_pkg.is_empty() {
                    if name == "SysML_Model" { String::new() } else { name.to_string() }
                } else {
                    format!("{}::{}", parent_pkg, name)
                };
                collect_qualified_package_bodies(body, &qualified_pkg, out);
                if let Some(slice) = remainder.get_mut(pkg_pos..=close_pos) {
                    for b in slice {
                        if *b != b'\n' { *b = b' '; }
                    }
                }
                idx = close_pos + 1;
                continue;
            }
        }
        idx = pkg_pos + 7;
    }

    let shallow = String::from_utf8(remainder).unwrap_or_default();
    if !shallow.trim().is_empty() {
        out.push((parent_pkg.to_string(), shallow));
    }
}

/// Compares scope-qualified AST definitions between schema directory and pipeline schema.
/// Realises: [REQ-DUAL-SCHEMA-SSOT-PARITY/SCOPED-COMPARISON]
pub(crate) fn compare_scoped_defs<T: PartialEq>(
    kind_label: &str,
    dir_map: &BTreeMap<String, Vec<T>>,
    pipe_map: &BTreeMap<String, Vec<T>>,
    mismatches: &mut Vec<String>,
) {
    for (qual_name, dir_items) in dir_map {
        match pipe_map.get(qual_name) {
            Some(pipe_items) if dir_items != pipe_items => {
                if kind_label == "part def" {
                    mismatches.push(format!(
                        "part def '{}' has structural divergence between schema/*.sysml and .pipeline/schema.sysml (ports, attributes, or body differ)",
                        qual_name
                    ));
                } else {
                    mismatches.push(format!(
                        "{} '{}' has structural divergence between schema/*.sysml and .pipeline/schema.sysml",
                        kind_label, qual_name
                    ));
                }
            }
            None => mismatches.push(format!(
                "{} defined in schema/*.sysml but missing in .pipeline/schema.sysml: [\"{}\"]",
                kind_label, qual_name
            )),
            Some(_) => {}
        }
    }
    for qual_name in pipe_map.keys() {
        if !dir_map.contains_key(qual_name) {
            mismatches.push(format!(
                "{} defined in .pipeline/schema.sysml but missing in schema/*.sysml: [\"{}\"]",
                kind_label, qual_name
            ));
        }
    }
}

/// Package-scoped AST node maps across all 11 SysML v2 definition kinds for Check 31 comparison.
/// Realises: [REQ-DUAL-SCHEMA-SSOT-PARITY]
#[derive(Default)]
struct ScopedAstDefs {
    parts: BTreeMap<String, Vec<PartDef>>,
    ports: BTreeMap<String, Vec<PortDef>>,
    actions: BTreeMap<String, Vec<ActionDef>>,
    reqs: BTreeMap<String, Vec<RequirementDef>>,
    constraints: BTreeMap<String, Vec<ConstraintDef>>,
    states: BTreeMap<String, Vec<StateDef>>,
    calcs: BTreeMap<String, Vec<CalcDef>>,
    conns: BTreeMap<String, Vec<ConnectionDef>>,
    tests: BTreeMap<String, Vec<TestCaseDef>>,
    attrs: BTreeMap<String, Vec<AttributeDef>>,
    use_cases: BTreeMap<String, Vec<(String, String)>>,
}

impl ScopedAstDefs {
    /// Formats a package-qualified AST node name (`pkg::name` or `name`).
    /// Realises: [REQ-DUAL-SCHEMA-SSOT-PARITY/PACKAGE-SCOPING]
    fn qualify(pkg_path: &str, name: &str) -> String {
        if pkg_path.is_empty() {
            name.to_string()
        } else {
            format!("{}::{}", pkg_path, name)
        }
    }

    /// Parses all 11 SysML v2 AST definition kinds from package-qualified body slices.
    /// Realises: [REQ-DUAL-SCHEMA-SSOT-PARITY]
    fn from_bodies(bodies: &[(String, String)]) -> Self {
        let mut defs = Self::default();
        for (pkg, body) in bodies {
            for p in parse_part_defs(body) {
                let v = defs.parts.entry(Self::qualify(pkg, &p.name)).or_default();
                if !v.contains(&p) { v.push(p); }
            }
            for p in parse_port_defs(body) {
                let v = defs.ports.entry(Self::qualify(pkg, &p.name)).or_default();
                if !v.contains(&p) { v.push(p); }
            }
            for a in parse_action_defs(body) {
                let v = defs.actions.entry(Self::qualify(pkg, &a.name)).or_default();
                if !v.contains(&a) { v.push(a); }
            }
            for r in parse_requirement_defs(body) {
                let v = defs.reqs.entry(Self::qualify(pkg, &r.name)).or_default();
                if !v.contains(&r) { v.push(r); }
            }
            for c in parse_constraint_defs(body) {
                let v = defs.constraints.entry(Self::qualify(pkg, &c.name)).or_default();
                if !v.contains(&c) { v.push(c); }
            }
            for s in parse_state_defs(body) {
                let v = defs.states.entry(Self::qualify(pkg, &s.name)).or_default();
                if !v.contains(&s) { v.push(s); }
            }
            for c in parse_calc_defs(body) {
                let v = defs.calcs.entry(Self::qualify(pkg, &c.name)).or_default();
                if !v.contains(&c) { v.push(c); }
            }
            for c in parse_connection_defs(body) {
                let v = defs.conns.entry(Self::qualify(pkg, &c.name)).or_default();
                if !v.contains(&c) { v.push(c); }
            }
            for t in parse_test_case_defs(body) {
                let v = defs.tests.entry(Self::qualify(pkg, &t.name)).or_default();
                if !v.contains(&t) { v.push(t); }
            }
            for a in parse_attribute_defs(body) {
                let v = defs.attrs.entry(Self::qualify(pkg, &a.name)).or_default();
                if !v.contains(&a) { v.push(a); }
            }
            for (uc_name, uc_body) in parse_use_case_defs(body) {
                let v = defs.use_cases.entry(Self::qualify(pkg, &uc_name)).or_default();
                let item = (uc_name, uc_body);
                if !v.contains(&item) { v.push(item); }
            }
        }
        defs
    }
}

/// Check 31: Dual-Schema SSOT Parity Gate.
/// Realises: [REQ-DUAL-SCHEMA-SSOT-PARITY]
pub fn check_dual_schema_ssot_parity(repo_root: &Path) -> Result<ParityOutcome, Vec<String>> {
    let mut schema_sysml_files = collect_dual_schema_files(repo_root)?;
    let pipeline_schema = repo_root.join(".pipeline").join("schema.sysml");
    let has_pipeline_schema = pipeline_schema.is_file()
        && fs::metadata(&pipeline_schema)
            .map(|m| m.len() > 0)
            .unwrap_or(false);

    if schema_sysml_files.is_empty() || !has_pipeline_schema {
        return Ok(ParityOutcome::DualSchemaSingleOrClean);
    }

    let mut schema_dir_content = String::new();
    schema_sysml_files.sort();
    for fpath in &schema_sysml_files {
        let c = fs::read_to_string(fpath).map_err(|e| {
            vec![format!(
                "Check 31 failed: Unable to read {}: {}",
                fpath.display(),
                e
            )]
        })?;
        schema_dir_content.push_str(&c);
        schema_dir_content.push_str("\n\n");
    }

    let pipeline_schema_content = fs::read_to_string(&pipeline_schema).map_err(|e| {
        vec![format!(
            "Check 31 failed: Unable to read {}: {}",
            pipeline_schema.display(),
            e
        )]
    })?;

    let mut dir_bodies = Vec::new();
    collect_qualified_package_bodies(&schema_dir_content, "", &mut dir_bodies);
    let mut pipe_bodies = Vec::new();
    collect_qualified_package_bodies(&pipeline_schema_content, "", &mut pipe_bodies);

    let dir_defs = ScopedAstDefs::from_bodies(&dir_bodies);
    let pipe_defs = ScopedAstDefs::from_bodies(&pipe_bodies);

    let mut mismatches = Vec::new();
    compare_scoped_defs("part def", &dir_defs.parts, &pipe_defs.parts, &mut mismatches);
    compare_scoped_defs("port def", &dir_defs.ports, &pipe_defs.ports, &mut mismatches);
    compare_scoped_defs("action def", &dir_defs.actions, &pipe_defs.actions, &mut mismatches);
    compare_scoped_defs("requirement def", &dir_defs.reqs, &pipe_defs.reqs, &mut mismatches);
    compare_scoped_defs(
        "constraint def",
        &dir_defs.constraints,
        &pipe_defs.constraints,
        &mut mismatches,
    );
    compare_scoped_defs("state def", &dir_defs.states, &pipe_defs.states, &mut mismatches);
    compare_scoped_defs("calc def", &dir_defs.calcs, &pipe_defs.calcs, &mut mismatches);
    compare_scoped_defs("connection def", &dir_defs.conns, &pipe_defs.conns, &mut mismatches);
    compare_scoped_defs("test case def", &dir_defs.tests, &pipe_defs.tests, &mut mismatches);
    compare_scoped_defs("attribute def", &dir_defs.attrs, &pipe_defs.attrs, &mut mismatches);
    compare_scoped_defs(
        "use case def",
        &dir_defs.use_cases,
        &pipe_defs.use_cases,
        &mut mismatches,
    );
    if schema_sysml_files.len() > 1 {
        if let Ok(m_text) = fs::read_to_string(repo_root.join("schema/model.sysml")) {
            crate::spec_audit::contract_and_schema_parity::compare_model_sysml_against_combined(
                &m_text,
                &schema_dir_content,
                &mut mismatches,
            );
        }
    }

    if mismatches.is_empty() {
        Ok(ParityOutcome::DualSchemaIdentical)
    } else {
        Err(mismatches)
    }
}

/// Discovers non-empty `.sysml` files in `schema/` and `docs/designs/`, verifying `.digest.json` sidecars.
/// Realises: [REQ-DUAL-SCHEMA-SSOT-PARITY]
fn collect_dual_schema_files(repo_root: &Path) -> Result<Vec<PathBuf>, Vec<String>> {
    let schema_dir = repo_root.join("schema");
    let designs_dir = repo_root.join("docs").join("designs");
    let mut schema_sysml_files = Vec::new();

    if schema_dir.is_dir() {
        schema_sysml_files =
            deap_core::sysml_ast::UnifiedSchemaModelExtractor::discover_schema_files(&[
                &schema_dir,
            ])
            .into_iter()
            .filter(|p| {
                p.extension().is_some_and(|ext| ext == "sysml")
                    && fs::metadata(p).map(|m| m.len() > 0).unwrap_or(false)
            })
            .collect();
    }

    if designs_dir.is_dir() {
        let mut design_sysml_files: Vec<PathBuf> =
            deap_core::sysml_ast::UnifiedSchemaModelExtractor::discover_schema_files(&[
                &designs_dir,
            ])
            .into_iter()
            .filter(|p| {
                p.extension().is_some_and(|ext| ext == "sysml")
                    && fs::metadata(p).map(|m| m.len() > 0).unwrap_or(false)
            })
            .collect();
        design_sysml_files.sort();

        let mut sidecar_errors = Vec::new();
        for design_sysml in &design_sysml_files {
            let digest_path = design_sysml.with_extension("digest.json");
            let has_valid_digest = digest_path.is_file()
                && fs::metadata(&digest_path)
                    .map(|m| m.len() > 0)
                    .unwrap_or(false);
            if !has_valid_digest {
                sidecar_errors.push(format!(
                    "Check 31 failed: Missing or empty companion digest sidecar {} for design schema {}",
                    digest_path.display(),
                    design_sysml.display()
                ));
            }
        }
        if !sidecar_errors.is_empty() {
            return Err(sidecar_errors);
        }
        schema_sysml_files.extend(design_sysml_files);
    }

    Ok(schema_sysml_files)
}
