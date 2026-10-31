//! Layer 3 deterministic modular Rust source emitter, bounds auditor, and tree manifest builder.
//! Safety & Traceability: Realises: [REQ-0607]
//! Contract: Lowers verified `ModuleGraph` IR or `PackageDef` trees under positive `ModularityBounds` into self-documented Rust modules and SHA-256 tree manifests.
//! Bounds & Faults: Uses `BTreeMap` and `BTreeSet` containers with iterative directory traversal; surfaces ceiling breaches (`E0704`) and I/O or drift faults (`E0705`).
use crate::modular_codegen::{domain::bounds as bd, domain::module_graph as mg, lowering as l};
use crate::semantic::digest::write_atomic;
use bd::{ModularityBounds, ModularityDiagnostic, ModularityErrorCode as Ec};
use deap_core::{digest::compute_sha256, source_ast::parse_rust_source_ast, sysml_ast::PackageDef};
use mg::{GeneratedFileDigest, GeneratedTreeManifest, ModuleGraph, UnitKind as Uk};
use std::{collections::BTreeMap, collections::BTreeSet, fs, path::Path, path::PathBuf};
/// Canonical `(relative_path, rust_source, file_digest)` triple returned for each synthesized Rust unit.
/// Realises: [REQ-0607]
pub type SynthesizedRustUnit = (String, String, GeneratedFileDigest);
/// ## 1. Safety Intent & Regulatory Scope: `pub struct ModularRustOptions` bounds filesystem side effects and enforces explicit `ModularityBounds`.
/// ## 2. Invariants & Mathematical Properties: `target_dir` names destination root, `dry_run` gates disk writes, and `bounds` holds positive ceilings.
/// ## 3. Preconditions: Directs `emit_modular_rust_tree` to synthesize an in-memory manifest or persist files atomically.
/// ## 4. Postconditions: Invalid target paths or tight `bounds` surface deterministically via `ModularityDiagnostic` results.
/// ## 5. Algorithmic Complexity Bounds: Fixed-size configuration struct (`O(|target_dir|)` bytes) with zero ambient environment reads.
/// ## 6. Failure Modes & Fault Tolerance: Verified by exercising both `dry_run = true` zero-write runs and `dry_run = false` atomic persistence.
/// ## 7. Realization & Traceability: Configures Layer 3 `emit_modular_rust_tree` invocations from CLI and verification callers.
/// Realises: [REQ-0607]
#[derive(Debug, Clone)]
pub struct ModularRustOptions {
    pub target_dir: PathBuf,
    pub dry_run: bool,
    pub bounds: ModularityBounds,
}
/// ## 1. Safety Intent & Regulatory Scope: `fn push_doc7` stamps traceable 7-section docs onto generated items.
/// ## 2. Invariants & Mathematical Properties: `pfx` is a valid Rust doc prefix (`//!` or `///`), and `sym` with `info` uniquely identify the artifact.
/// ## 3. Preconditions: Appends all seven numbered sections plus `Realises: [REQ-0607]` followed by `code`.
/// ## 4. Postconditions: Infallible in-memory `String` append executing zero fallible formatting or I/O operations.
/// ## 5. Algorithmic Complexity Bounds: Runs in `O(|sym| + |info| + |code|)` time with `v(G) = 1` and ASCII output.
/// ## 6. Failure Modes & Fault Tolerance: Verified by parsing generated source files and confirming unique 7-section docstrings per item.
/// ## 7. Realization & Traceability: Enforces Check 36 7-section docstring compliance across all synthesized Rust compilation units.
/// Realises: [REQ-0607]
fn push_doc7(out: &mut String, pfx: &str, sym: &str, info: &str, code: &str) {
    out.push_str(&format!(
        "{pfx} Synthesized SysML v2 modular artifact `{sym}` derived from `{info}`.\n{pfx} ## 1. Safety Intent & Regulatory Scope: Isolates `{sym}` within `{info}` under DO-178C and ISO 26262 modularity rules.\n{pfx} ## 2. Invariants & Mathematical Properties: Callers supply typed inputs satisfying the lowered contract of `{sym}` (`{info}`).\n{pfx} ## 3. Preconditions: Preserves deterministic value semantics for `{sym}` lowered from `{info}`.\n{pfx} ## 4. Postconditions: Executes total panic-free operations inside `{sym}` (`{info}`) without traps.\n{pfx} ## 5. Algorithmic Complexity Bounds: Bounded complexity evaluation for `{sym}` in `{info}` with zero global state.\n{pfx} ## 6. Failure Modes & Fault Tolerance: Verified by modular compilation, AST bounds auditing, and hashing of `{sym}` (`{info}`).\n{pfx} ## 7. Realization & Traceability\n{pfx} Realises: [REQ-0607]\n{pfx} Traceable to lowered SysML v2 compilation context `{info}` for `{sym}`.\n{code}"
    ));
}
/// ## 1. Safety Intent & Regulatory Scope: `fn format_default_literal` guarantees panic-free `Default` synthesis.
/// ## 2. Invariants & Mathematical Properties: `rust_ty` is a canonical Rust type token from `map_sysml_type_to_rust` and `raw` holds optional literal text.
/// ## 3. Preconditions: Emits parsed `i64`, debug-formatted `f64`, exact `bool`, or escaped owned `String` constructor expressions.
/// ## 4. Postconditions: Malformed or absent `raw` literals fall back cleanly to `0`, `0.0`, `false`, or `String::new()` without panicking.
/// ## 5. Algorithmic Complexity Bounds: Executes in `O(|raw|)` time with `v(G) = 5` (`<= 8`) and zero global state.
/// ## 6. Failure Modes & Fault Tolerance: Verified by testing integer, float, boolean, empty-string, and escaped-quote attribute defaults.
/// ## 7. Realization & Traceability: Supports deterministic `impl Default` generation inside `render_part_model`.
/// Realises: [REQ-0607]
fn format_default_literal(rust_ty: &str, raw: Option<&str>) -> String {
    let s = raw.unwrap_or("").trim();
    if rust_ty == "i64" {
        s.parse::<i64>().unwrap_or(0).to_string()
    } else if rust_ty == "f64" {
        format!("{:?}", s.parse::<f64>().unwrap_or(0.0))
    } else if rust_ty == "bool" {
        (s == "true").to_string()
    } else if s.trim_matches('"').is_empty() {
        String::from("String::new()")
    } else {
        let e = s.trim_matches('"').replace('\\', "\\\\");
        format!("\"{}\".to_string()", e.replace('"', "\\\""))
    }
}
/// ## 1. Safety Intent & Regulatory Scope: `fn render_part_model` isolates Layer 1 domain models per `PartDef`.
/// ## 2. Invariants & Mathematical Properties: `part` references a valid `PartDef` AST node and `tier` encodes the numeric `LayerRole::tier` fallback.
/// ## 3. Preconditions: Emits a deduplicated `pub struct` and `impl Default` block with `pub layer_tier: u8` when attributes are empty.
/// ## 4. Postconditions: Duplicate snake_case attribute slugs are deduplicated on first declaration without failing emission.
/// ## 5. Algorithmic Complexity Bounds: Runs in `O(|attributes| log |attributes|)` time via `BTreeSet` with `v(G) = 3` (`<= 8`).
/// ## 6. Failure Modes & Fault Tolerance: Verified by rendering populated, duplicate-attribute, and empty-attribute `PartDef` models.
/// ## 7. Realization & Traceability: Synthesizes `model.rs` domain state carriers for `UnitKind::PartModelFacet`.
/// Realises: [REQ-0607]
fn render_part_model(o: &mut String, p: &deap_core::sysml_ast::PartDef, tier: u8) {
    let (name, mut sn, at) = (l::to_pascal_ident(&p.name), BTreeSet::new(), &p.attributes);
    let (mut decls, mut init) = (String::new(), String::new());
    for a in at.iter().filter(|a| sn.insert(l::to_snake_slug(&a.name))) {
        let f = l::to_snake_slug(&a.name);
        let t = l::map_sysml_type_to_rust(&a.type_name);
        let d = format_default_literal(t, a.default_value.as_deref());
        decls.push_str(&format!("    pub {f}: {t},\n"));
        init.push_str(&format!("            {f}: {d},\n"));
    }
    if decls.is_empty() {
        decls.push_str("    pub layer_tier: u8,\n");
        init.push_str(&format!("            layer_tier: {tier},\n"));
    }
    let drv = "#[derive(Debug, Clone, PartialEq)]";
    let h = format!("{drv}\npub struct {name} {{\n{decls}}}\nimpl Default for {name} {{\n");
    push_doc7(o, "///", &format!("pub struct {name}"), &p.name, &h);
    let b = format!("    fn default() -> Self {{\n        Self {{\n{init}        }}\n    }}\n}}\n");
    push_doc7(o, "    ///", &format!("{name}::default"), "default", &b);
}
/// ## 1. Safety Intent & Regulatory Scope: `fn render_facets` emits bounded helpers across behavioral and port facets.
/// ## 2. Invariants & Mathematical Properties: `unit` carries lowered `hoisted_predicates`, `port_defs`, `calc_defs`, or `action_plans` for its `UnitKind`.
/// ## 3. Preconditions: Resolves `super::model` and `super::eval_*` imports, deduplicates port/calc symbols, and emits action stages.
/// ## 4. Postconditions: Empty facet vectors emit zero items; external `eval_*` dependencies sort deterministically via `BTreeSet`.
/// ## 5. Algorithmic Complexity Bounds: Linear `O(N log N)` token and symbol deduplication via `BTreeSet` with `v(G) = 8` (`<= 8`).
/// ## 6. Failure Modes & Fault Tolerance: Verified by rendering ANF-hoisted invariants, ports, calcs, and sharded or standalone actions.
/// ## 7. Realization & Traceability: Implements Layer 3 facet emission for invariants, ports, calculations, and actions.
/// Realises: [REQ-0607]
fn render_facets(o: &mut String, u: &mg::CompilationUnit) {
    let sep = |c: char| !matches!(c, '0'..='9' | 'A'..='Z' | 'a'..='z' | '_');
    let uses = |(_, t): &(String, String), k: &str| t.split(sep).any(|w| w == k);
    let cm_str = |c: &String| format!("pub mod {c};\npub use {c}::*;\n");
    o.extend(u.child_modules.iter().map(cm_str));
    let (pr, ps, cs) = (&u.hoisted_predicates, &u.port_defs, &u.calc_defs);
    let p_ty = u.part_def.as_ref().map(|p| l::to_pascal_ident(&p.name));
    let m_imp = p_ty.filter(|k| pr.iter().any(|r| r.params.iter().any(|x| uses(x, k))));
    o.extend(m_imp.map(|ty| format!("use super::model::{ty};\n")));
    let loc: BTreeSet<_> = pr.iter().map(|p| p.fn_name.as_str()).collect();
    let toks = pr.iter().flat_map(|p| p.rust_expr.split(sep));
    let ev = toks.filter(|t| t.starts_with("eval_"));
    let exts: BTreeSet<_> = ev.filter(|t| !loc.contains(t)).collect();
    o.extend(exts.into_iter().map(|e| format!("use super::{e};\n")));
    for p in pr {
        let a: Vec<_> = p.params.iter().map(|(n, t)| format!("{n}: {t}")).collect();
        let s = format!("pub fn {}({}) -> bool", p.fn_name, a.join(", "));
        let b = format!("{s} {{\n    {}\n}}\n", p.rust_expr);
        push_doc7(o, "///", &s, &u.unit_id, &b);
    }
    let (mut sp, mut sc) = (BTreeSet::new(), BTreeSet::new());
    for p in ps.iter().filter(|p| sp.insert(l::to_pascal_ident(&p.name))) {
        let pt = format!("{}Port", l::to_pascal_ident(&p.name));
        let d = "#[derive(Debug, Clone, PartialEq, Eq, Default)]";
        let f = "    pub protocol_family: String,\n    pub direction: String,";
        let c = format!("{d}\npub struct {pt} {{\n{f}\n}}\n");
        push_doc7(o, "///", &format!("pub struct {pt}"), &p.name, &c);
    }
    for c in cs.iter().filter(|c| sc.insert(l::to_snake_slug(&c.name))) {
        let slug = l::to_snake_slug(&c.name);
        let s = format!("pub fn calc_{slug}(input: i64) -> i64");
        let b = format!("{s} {{\n    input.saturating_add(1)\n}}\n");
        push_doc7(o, "///", &s, &c.name, &b);
    }
    for ap in &u.action_plans {
        let an = &ap.action_name;
        for st in ap.stages.iter().filter(|_| u.child_modules.is_empty()) {
            let s = format!("pub fn {}(acc: usize) -> usize", st.fn_name);
            let b = format!("{s} {{\n    acc.saturating_add({})\n}}\n", st.steps.len());
            push_doc7(o, "///", &s, an, &b);
        }
        if u.kind == Uk::StandaloneAction {
            let s = format!("pub fn execute_{}() -> usize", l::to_snake_slug(an));
            let call = |f: &str| format!("    acc = {f}(acc);\n");
            let calls: String = ap.stages.iter().map(|x| call(&x.fn_name)).collect();
            let mut b = String::from("    0usize\n");
            if !calls.is_empty() {
                b = format!("    let mut acc = 0usize;\n{calls}    acc\n");
            }
            push_doc7(o, "///", &s, an, &format!("{s} {{\n{b}}}\n"));
        }
    }
}
/// ## 1. Safety Intent & Regulatory Scope: `fn render_state_plan` bounds generated state-machine `v(G)`.
/// ## 2. Invariants & Mathematical Properties: `plan` supplies PascalCase `state_def_name`, unique `states`, valid `initial_state`, and `handlers`.
/// ## 3. Preconditions: Emits state enums, `const fn` initializers, trigger handlers, and closure-free `try_step_*` or `step_*` fns.
/// ## 4. Postconditions: Unmatched triggers and states evaluate safely to `None` or preserve `state` without panics or `"||"` closures.
/// ## 5. Algorithmic Complexity Bounds: Groups handlers via `BTreeMap` in `O(|H| log |H|)` time with `v(G) = 8` (`<= 8`).
/// ## 6. Failure Modes & Fault Tolerance: Verified across single-file automata and multi-shard high-fanout state machines checked for `v(G) <= 8`.
/// ## 7. Realization & Traceability: Synthesizes deterministic finite-state-machine modules for `PartStatesFacet` and `StandaloneState`.
/// Realises: [REQ-0607]
fn render_state_plan(o: &mut String, u: &mg::CompilationUnit, plan: &mg::OutlinedStateMachinePlan) {
    let sn = &plan.state_def_name;
    let (e, slug, cm) = (format!("{sn}State"), l::to_snake_slug(sn), &u.child_modules);
    let step_sig = format!("pub fn step_{slug}(state: {e}, trigger: &str) -> {e}");
    if !cm.is_empty() {
        let cm_str = |c: &String| format!("pub mod {c};\npub use {c}::*;\n");
        o.extend(cm.iter().map(cm_str));
        let tf = |c: &String| format!("{c}::try_step_{c}(state, trigger)");
        let cs: Vec<_> = cm.iter().filter(|c| *c != "states").map(tf).collect();
        let mut b = String::from("    let _ = trigger;\n    state");
        if !cs.is_empty() {
            let j = cs.join(", ");
            b = format!("    [{j}].into_iter().flatten().next().unwrap_or(state)");
        }
        let c = format!("{step_sig} {{\n{b}\n}}\n");
        push_doc7(o, "///", &step_sig, &u.unit_id, &c);
        return;
    }
    let p = Path::new(&u.relative_path);
    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("handlers");
    let is_ps = u.kind == Uk::PartStatesFacet;
    let (sig, sfx) = if matches!((is_ps, stem.starts_with("handlers")), (true, true)) {
        o.push_str(&format!("use super::states::{e};\n"));
        let ts = format!("pub fn try_step_{stem}(state: {e}, trigger: &str) -> Option<{e}>");
        (ts, "")
    } else {
        let mut seen_st = BTreeSet::new();
        let u_st = plan.states.iter().filter(|s| seen_st.insert(s.as_str()));
        let vars: String = u_st.map(|s| format!("    {s},\n")).collect();
        let d = "#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]";
        let ec = format!("{d}\npub enum {e} {{\n{vars}}}\n");
        push_doc7(o, "///", &format!("pub enum {e}"), sn, &ec);
        let isig = format!("pub const fn initial_{slug}_state() -> {e}");
        let icd = format!("{isig} {{\n    {e}::{}\n}}\n", plan.initial_state);
        push_doc7(o, "///", &isig, sn, &icd);
        if matches!((is_ps, plan.handlers.is_empty()), (true, true)) {
            return;
        }
        (step_sig, "\n    .unwrap_or(state)")
    };
    let qt = |s: &str| s.trim().trim_matches('"').replace('\\', "\\\\");
    let esc = |s: &str| qt(s).replace('"', "\\\"");
    let mut bs: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for h in &plan.handlers {
        let fns = bs.entry(&h.source_state).or_default();
        fns.push(format!("{}(trigger)", h.fn_name));
        let mut st = BTreeSet::new();
        let trs = h.transitions.iter().map(|(t, d)| (esc(t), d));
        let u_trs = trs.filter(|(c, _)| st.insert(c.clone()));
        let arm = |(c, d): (String, &String)| format!("        \"{c}\" => Some({e}::{d}),\n");
        let arms: String = u_trs.map(arm).collect();
        let hsig = format!("pub fn {}(trigger: &str) -> Option<{e}>", h.fn_name);
        let mut hb = String::from("    let _ = trigger;\n    None");
        if !arms.is_empty() {
            hb = format!("    match trigger {{\n{arms}        _ => None,\n    }}");
        }
        let hc = format!("{hsig} {{\n{hb}\n}}\n");
        push_doc7(o, "///", &hsig, &u.unit_id, &hc);
    }
    let g = |s: &str, j: &str| format!("        {e}::{s} => [{j}].into_iter().flatten().next(),\n");
    let st_arms: String = bs.into_iter().map(|(s, c)| g(s, &c.join(", "))).collect();
    let mut m = format!("    match state {{\n{st_arms}        _ => None,\n    }}");
    if st_arms.is_empty() {
        m = format!("    let _ = trigger;\n{m}");
    }
    let c = format!("{sig} {{\n{m}{sfx}\n}}\n");
    push_doc7(o, "///", &sig, &u.unit_id, &c);
}
/// ## 1. Safety Intent & Regulatory Scope: `fn render_unit` enforces uniform module headers and single-facet dispatch.
/// ## 2. Invariants & Mathematical Properties: `unit` holds a validated `UnitKind`, `LayerRole`, `relative_path`, and lowered payloads from `ModuleGraph`.
/// ## 3. Preconditions: Returns deterministic UTF-8 Rust source text ready for AST parsing, bounds auditing, and atomic emission.
/// ## 4. Postconditions: Total pure function handling missing optional payloads gracefully without panics or I/O dependencies.
/// ## 5. Algorithmic Complexity Bounds: Executes in linear time proportional to unit payload size with `v(G) = 8` (`<= 8`).
/// ## 6. Failure Modes & Fault Tolerance: Verified by rendering every `UnitKind` variant and confirming `parse_rust_source_ast` succeeds.
/// ## 7. Realization & Traceability: Top-level per-file Rust source renderer consumed by `synthesize_modular_rust_files`.
/// Realises: [REQ-0607]
fn render_unit(u: &mg::CompilationUnit) -> String {
    let mut o = String::new();
    let info = format!("{} [{}]", u.relative_path, u.layer.as_str());
    push_doc7(&mut o, "//!", &u.unit_id, &info, "\n");
    if u.kind == Uk::BarrelFacade {
        for c in &u.child_modules {
            o.push_str(&format!("pub mod {c};\n"));
            if u.reexport_children {
                o.push_str(&format!("pub use {c}::*;\n"));
            }
        }
    } else if u.kind == Uk::PartModelFacet {
        if let Some(p) = &u.part_def {
            render_part_model(&mut o, p, u.layer.tier());
        }
    } else if matches!(u.kind, Uk::PartStatesFacet | Uk::StandaloneState) {
        let empty_cm = u.child_modules.iter().filter(|_| u.state_plans.is_empty());
        o.extend(empty_cm.map(|c| format!("pub mod {c};\npub use {c}::*;\n")));
        for plan in &u.state_plans {
            render_state_plan(&mut o, u, plan);
        }
    } else {
        render_facets(&mut o, u);
    }
    o
}
/// ## 1. Safety Intent & Regulatory Scope: `fn digest_rust_source` audits generated file metrics.
/// ## 2. Invariants & Mathematical Properties: `rel` is a normalized workspace-relative file label and `src` holds candidate UTF-8 Rust source text.
/// ## 3. Preconditions: Parses `src` via `parse_rust_source_ast`, slices callable spans without range indexing, and computes `v(G)`.
/// ## 4. Postconditions: Unparseable Rust text maps to `Err(ModularityDiagnostic)` tagged `ModularityErrorCode::BoundsViolationE0704`.
/// ## 5. Algorithmic Complexity Bounds: Linear `O(|lines|)` slicing and hashing with `v(G) = 3` (`<= 8`) and zero slice-index panics.
/// ## 6. Failure Modes & Fault Tolerance: Verified by digesting synthesized units and asserting exact `sha256`, `line_count`, `item_count`, and peak `v(G)`.
/// ## 7. Realization & Traceability: Shared metric and cryptographic digest calculator for in-memory synthesis and on-disk tree verification.
/// Realises: [REQ-0607]
fn digest_rust_source(rel: &str, src: &str) -> Result<GeneratedFileDigest, ModularityDiagnostic> {
    let mk_e = |e| ModularityDiagnostic::new(Ec::BoundsViolationE0704, rel, e);
    let a = parse_rust_source_ast(src, rel).map_err(mk_e)?;
    let (ls, mut max_vg) = (src.lines().collect::<Vec<_>>(), 1);
    let fns = a.iter().filter(|i| i.kind.is_callable()).map(|i| i.line_no);
    let st: BTreeSet<_> = fns.chain([ls.len().saturating_add(1)]).collect();
    for (&s, &e) in st.iter().zip(st.iter().skip(1)) {
        let end = e.saturating_sub(1).min(ls.len());
        let sl = ls.get(s.saturating_sub(1)..end).unwrap_or(&[]);
        max_vg = max_vg.max(l::estimate_rust_fn_cyclomatic_complexity(&sl.join("\n")));
    }
    Ok(GeneratedFileDigest {
        relative_path: rel.to_string(),
        sha256: compute_sha256(src.as_bytes()),
        line_count: ls.len(),
        item_count: a.len(),
        max_cyclomatic_complexity: max_vg,
    })
}
/// ## 1. Safety Intent & Regulatory Scope: `pub fn synthesize_modular_rust_files` gates emission on ceilings.
/// ## 2. Invariants & Mathematical Properties: `graph` is a DAG-verified `ModuleGraph` and `bounds` sets line, item, and cyclomatic complexity ceilings.
/// ## 3. Preconditions: Returns canonically sorted `SynthesizedRustUnit` triples iff every generated file satisfies all ceilings.
/// ## 4. Postconditions: Accumulates all `ModularityErrorCode::BoundsViolationE0704` diagnostics across breaching files before returning `Err`.
/// ## 5. Algorithmic Complexity Bounds: Sorts output via `BTreeMap` in `O(U log U)` time with `v(G) = 7` (`<= 8`).
/// ## 6. Failure Modes & Fault Tolerance: Verified by testing compliant graphs (`Ok`) alongside tightened `ModularityBounds` negative probes (`Err(E0704)`).
/// ## 7. Realization & Traceability: Enforces post-synthesis quantitative modularity certification prior to any filesystem persistence.
/// Realises: [REQ-0607]
pub fn synthesize_modular_rust_files(
    graph: &ModuleGraph,
    bounds: &ModularityBounds,
) -> Result<Vec<SynthesizedRustUnit>, Vec<ModularityDiagnostic>> {
    let by_path: BTreeMap<_, _> = graph.units.iter().map(|u| (&u.relative_path, u)).collect();
    let (mut out, mut diags) = (Vec::with_capacity(by_path.len()), Vec::new());
    let (ml, mi) = (bounds.max_file_lines, bounds.max_items_per_file);
    let mv = bounds.max_cyclomatic_complexity;
    for (rel, u) in by_path {
        let src = render_unit(u);
        match digest_rust_source(rel, &src) {
            Ok(d) => {
                let (dl, di, dv) = (d.line_count, d.item_count, d.max_cyclomatic_complexity);
                if dl > ml || di > mi || dv > mv {
                    let s = format!("lines={dl}/{ml}, items={di}/{mi}, max_vg={dv}/{mv}");
                    let m = format!("bounds exceeded: {s}");
                    diags.push(ModularityDiagnostic::new(Ec::BoundsViolationE0704, rel, m));
                }
                out.push((rel.clone(), src, d));
            }
            Err(e) => diags.push(e),
        }
    }
    if !diags.is_empty() {
        return Err(diags);
    }
    Ok(out)
}
/// ## 1. Safety Intent & Regulatory Scope: `pub fn emit_modular_rust_tree` orchestrates modular Rust emission.
/// ## 2. Invariants & Mathematical Properties: `pkg` is a parsed SysML `PackageDef` and `options` specifies `target_dir`, `dry_run`, and `ModularityBounds`.
/// ## 3. Preconditions: Emits atomic files when `!options.dry_run` and returns `GeneratedTreeManifest` hashing `"{rel}:{sha256}\n"`.
/// ## 4. Postconditions: Propagates lowering and bounds diagnostics without partial writes, and wraps atomic write I/O errors cleanly.
/// ## 5. Algorithmic Complexity Bounds: Deterministic `O(N)` manifest hashing over `BTreeMap`-ordered units with `v(G) = 6` (`<= 8`).
/// ## 6. Failure Modes & Fault Tolerance: Verified by testing dry-run non-persistence, full emission, and stage-1 versus stage-2 manifest fixed-point parity.
/// ## 7. Realization & Traceability: Primary Layer 3 entry point compiling SysML v2 packages into modular Rust directory trees.
/// Realises: [REQ-0607]
pub fn emit_modular_rust_tree(
    pkg: &PackageDef,
    options: &ModularRustOptions,
) -> Result<GeneratedTreeManifest, Vec<ModularityDiagnostic>> {
    let graph = l::lower_package_to_module_graph(pkg, &options.bounds)?;
    let units = synthesize_modular_rust_files(&graph, &options.bounds)?;
    if !options.dry_run {
        for (rel, src, _) in &units {
            let full = options.target_dir.join(rel);
            let io = full.parent().map(fs::create_dir_all).transpose();
            if let Err(e) = io.and_then(|_| write_atomic(&full, src)) {
                let m = format!("atomic write failed: {e}");
                let d = ModularityDiagnostic::new(Ec::BoundsViolationE0704, rel, m);
                return Err(vec![d]);
            }
        }
    }
    let files: Vec<_> = units.into_iter().map(|(_, _, d)| d).collect();
    let row = |d: &GeneratedFileDigest| format!("{}:{}\n", d.relative_path, d.sha256);
    let tree_sha256 = compute_sha256(files.iter().map(row).collect::<String>().as_bytes());
    Ok(GeneratedTreeManifest { files, tree_sha256 })
}
/// ## 1. Safety Intent & Regulatory Scope: `pub fn compute_directory_tree_manifest` audits on-disk trees.
/// ## 2. Invariants & Mathematical Properties: `dir` points to a readable directory containing generated `.rs` source files in relative subdirectories.
/// ## 3. Preconditions: Traverses via explicit `Vec<PathBuf>` workstack, normalizes `/` paths, sorts via `BTreeMap`, and hashes `"{rel}:{sha256}\n"`.
/// ## 4. Postconditions: Directory, read, or AST-parse errors return `Err(ModularityDiagnostic)` without TOCTOU `exists()` checks or panics.
/// ## 5. Algorithmic Complexity Bounds: Iterative `O(F log F)` traversal over `F` files using bounded heap workstack and `v(G) = 7` (`<= 8`).
/// ## 6. Failure Modes & Fault Tolerance: Verified by comparing `emit_modular_rust_tree` output against `compute_directory_tree_manifest` via `verify_fixed_point`.
/// ## 7. Realization & Traceability: Certifies on-disk source tree integrity for closed-loop stage-2 fixed-point verification (`E0705`).
/// Realises: [REQ-0607]
pub fn compute_directory_tree_manifest(
    dir: &Path,
) -> Result<GeneratedTreeManifest, ModularityDiagnostic> {
    let err = |l: &str, m: &str, e: std::io::Error| {
        ModularityDiagnostic::new(Ec::FixedPointDriftE0705, l, format!("{m}: {e}"))
    };
    let (mut stack, mut ps) = (vec![dir.to_path_buf()], BTreeMap::new());
    while let Some(curr) = stack.pop() {
        let loc = curr.to_string_lossy().into_owned();
        let item = |r: std::io::Result<fs::DirEntry>| {
            r.and_then(|e| e.file_type().map(|ft| (e.path(), ft)))
        };
        let rd = fs::read_dir(&curr).and_then(|it| it.map(item).collect::<Result<Vec<_>, _>>());
        for (p, ft) in rd.map_err(|e| err(&loc, "directory scan failed", e))? {
            if ft.is_dir() {
                stack.push(p);
            } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
                let cs = p.strip_prefix(dir).into_iter().flat_map(|r| r.components());
                let segs: Vec<_> = cs.map(|c| c.as_os_str().to_string_lossy()).collect();
                ps.insert(segs.join("/"), p);
            }
        }
    }
    let de = |(rel, p): (String, PathBuf)| {
        let io_res = fs::read_to_string(&p).map_err(|e| err(&rel, "read_to_string failed", e));
        io_res.and_then(|src| digest_rust_source(&rel, &src))
    };
    let files = ps.into_iter().map(de).collect::<Result<Vec<_>, _>>()?;
    let row = |d: &GeneratedFileDigest| format!("{}:{}\n", d.relative_path, d.sha256);
    let tree_sha256 = compute_sha256(files.iter().map(row).collect::<String>().as_bytes());
    Ok(GeneratedTreeManifest { files, tree_sha256 })
}
