//! Epic specification (`docs/epics/EPIC-*.md`) synthesis from ordered SysML v2 `PackageDef` nodes.
//!
//! Safety & Traceability: Realises: [REQ-SYNC-FORWARD-EPIC-EMITTER]
//! Contract: Emits canonical Epic Markdown specifications for root, `ConOps`, and subsystem packages, assembling linked Feature checklists, associated Use Case and User Story links, 5-section architectural prose, and system-level Mermaid class/statechart diagrams.
//! Bounds & Faults: Iterates deterministically over pre-indexed package and part slices using saturating index arithmetic (`saturating_add`) and non-panicking tracker issue ID lookups.

use super::*;

/// Scoped package metadata for synthesizing a single `EPIC-XX-*.md` specification.
/// Realises: [REQ-SYNC-FORWARD-EPIC-EMITTER]
struct EpicRenderCtx<'a> {
    root_pkg_name: &'a str,
    pkg: &'a PackageDef,
    epic_num: usize,
    is_root: bool,
    is_conops: bool,
    subsys_display: String,
    subsys_dir: String,
}

/// Resolves the Epic title and workspace-relative `epics/EPIC-XX-*.md` destination path.
/// Realises: [REQ-SYNC-FORWARD-EPIC-EMITTER]
fn resolve_epic_title_and_path(ctx: &EpicRenderCtx<'_>) -> (String, String) {
    let p = ctx.pkg;
    let epic_num = ctx.epic_num;
    if ctx.is_root {
        let root_title = if p.name == "DEAP_Compiler_System" {
            "Epic 01: System Architecture and Compiler Primacy".to_string()
        } else {
            format!(
                "Epic {:02}: {} System Architecture",
                epic_num,
                to_display_name(&p.name)
            )
        };
        (
            root_title,
            format!("epics/EPIC-{:02}-{}.md", epic_num, p.name),
        )
    } else if ctx.is_conops {
        (
            "Epic 02: Concept of Operations Operational Activities".to_string(),
            "epics/EPIC-02-ConOps.md".to_string(),
        )
    } else {
        (
            format!("Epic {:02}: {}", epic_num, to_display_name(&p.name)),
            format!("epics/EPIC-{:02}-{}.md", epic_num, p.name),
        )
    }
}

/// Assembles the Markdown Feature checklist (`- [ ] #<id> - [Feature XX: ...]`) allocated to `ctx.pkg`.
/// Realises: [REQ-SYNC-FORWARD-EPIC-EMITTER]
fn build_epic_feature_checklist(ctx: &EpicRenderCtx<'_>, index: &ForwardSyncIndex<'_>) -> String {
    let mut feat_checklist = Vec::new();
    for (f_idx, part_ctx) in index.part_contexts.iter().enumerate() {
        let f_num = f_idx.saturating_add(1);
        let f_path = format!("features/FEAT-{:02}-{}.md", f_num, part_ctx.part.name);
        let f_issue =
            format_issue_id_cell(resolve_existing_issue_id(&index.target_out_dir, &f_path));
        let is_match = part_ctx.pkg_name == ctx.pkg.name
            || (ctx.is_conops && part_ctx.pkg_name == "ConOps")
            || (ctx.is_root
                && (part_ctx.pkg_name == ctx.root_pkg_name || part_ctx.pkg_name == "SysML_Model"));
        if is_match {
            let feat_title = format!(
                "Feature {:02}: [{}] {}",
                f_num, part_ctx.subsys_display, part_ctx.part.name
            );
            let feat_url = format!("../features/FEAT-{:02}-{}.md", f_num, part_ctx.part.name);
            let mut mapped = Vec::new();
            for req in &part_ctx.part.requirements {
                mapped.push(req.name.clone());
            }
            for c in &part_ctx.part.constraints {
                mapped.push(c.name.clone());
            }
            let mapped_clauses = if mapped.is_empty() {
                format!("{}.sysml", part_ctx.subsys_dir)
            } else {
                mapped.join(", ")
            };
            feat_checklist.push(format!(
                "- [ ] {} - [{}]({}) ({})",
                f_issue, feat_title, feat_url, mapped_clauses
            ));
        }
    }
    if feat_checklist.is_empty() {
        feat_checklist.push(format!(
            "- [ ] Feature 01: Subsystem capability implementation for {}.",
            ctx.subsys_display
        ));
    }
    feat_checklist.join("\n")
}

/// Builds the associated Use Case and User Story checklist blocks for `ctx.pkg`.
/// Realises: [REQ-SYNC-FORWARD-EPIC-EMITTER]
fn build_epic_associated_backlog(
    ctx: &EpicRenderCtx<'_>,
    index: &ForwardSyncIndex<'_>,
) -> (String, String) {
    let mut assoc_uc = Vec::new();
    for uc_m in index
        .uc_metas
        .iter()
        .filter(|m| m.pkg_name == ctx.pkg.name || (ctx.is_conops && !index.has_conops_uc))
    {
        let clean_name = strip_use_case_prefix(&uc_m.uc.name);
        let uc_slug = sanitize_id(clean_name);
        let uc_path = format!("use-cases/UC-{:02}-{}.md", uc_m.u_num, uc_slug);
        let uc_issue =
            format_issue_id_cell(resolve_existing_issue_id(&index.target_out_dir, &uc_path));
        assoc_uc.push(format!(
            "- [ ] {} - [Use Case {:02}: {}](../use-cases/UC-{:02}-{}.md)",
            uc_issue,
            uc_m.u_num,
            to_display_name(clean_name),
            uc_m.u_num,
            uc_slug
        ));
    }

    let mut assoc_us = Vec::new();
    for s_m in index
        .story_metas
        .iter()
        .filter(|m| m.pkg_name == ctx.pkg.name || (ctx.is_conops && !index.has_conops_us))
    {
        let us_slug = sanitize_id(&s_m.action_name);
        let us_path = format!("user-stories/US-{:02}-{}.md", s_m.s_num, us_slug);
        let us_issue =
            format_issue_id_cell(resolve_existing_issue_id(&index.target_out_dir, &us_path));
        let us_display_title = index
            .story_display_titles
            .get(&s_m.s_num)
            .cloned()
            .unwrap_or_else(|| to_display_name(&s_m.action_name));
        assoc_us.push(format!(
            "- [ ] {} - [User Story {:02}: {}](../user-stories/US-{:02}-{}.md)",
            us_issue, s_m.s_num, us_display_title, s_m.s_num, us_slug
        ));
    }

    let fallback = "- None directly allocated (operational behavior allocated at system ConOps level in Epic 02)";
    let assoc_uc_text = if assoc_uc.is_empty() {
        fallback.to_string()
    } else {
        assoc_uc.join("\n")
    };
    let assoc_us_text = if assoc_us.is_empty() {
        fallback.to_string()
    } else {
        assoc_us.join("\n")
    };
    (assoc_uc_text, assoc_us_text)
}

/// Formats normative `## 6. Source References` citations for an Epic based on modular vs single-file schema layout.
/// Realises: [REQ-SYNC-FORWARD-EPIC-EMITTER]
fn resolve_epic_source_refs(ctx: &EpicRenderCtx<'_>, has_modular_schema: bool) -> String {
    if !has_modular_schema {
        r#"- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 15288:2023 §6.4.3 Architecture Definition Process

Subsystem architectural composition and formal invariants derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 15288."#
            .to_string()
    } else if ctx.is_root {
        r#"- Schema SSOT Architecture: `schema/model.sysml`
- ConOps Operational Activities: `schema/conops/activities.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 15288:2023 §6.4.3 Architecture Definition Process

The system architecture and formal verification invariants derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 15288."#
            .to_string()
    } else if ctx.is_conops {
        r#"- ConOps Operational Activities: `schema/conops/activities.sysml`
- ConOps External Actors: `schema/conops/actors.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational activity definitions and actor models derive from `schema/conops/activities.sysml` pursuant to ISO/IEC/IEEE 29148."#
            .to_string()
    } else {
        let subsys_dir = &ctx.subsys_dir;
        format!(
            r#"- System Architecture Model: `schema/model.sysml`
- Subsystem Specification Model: `schema/subsystems/{subsys_dir}/architecture.sysml`
- Subsystem Requirements Model: `schema/subsystems/{subsys_dir}/requirements.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 15288:2023 §6.4.3 Architecture Definition Process

Subsystem architectural composition and formal invariants derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 15288."#
        )
    }
}

/// Synthesizes all `epics/EPIC-XX-*.md` specifications from `index.ordered_packages` into `generated_files`.
/// Realises: [REQ-SYNC-FORWARD-EPIC-EMITTER]
pub(crate) fn emit_epic_specs(
    pkg: &PackageDef,
    index: &ForwardSyncIndex<'_>,
    generated_files: &mut BTreeMap<String, String>,
) {
    for (idx, p) in index.ordered_packages.iter().enumerate() {
        let epic_num = idx.saturating_add(1);
        let is_root = p.name == "SysML_Model" || p.name == pkg.name;
        let is_conops = p.name == "ConOps";
        let (subsys_display, subsys_dir) = if is_root {
            (
                "System Architecture".to_string(),
                "architecture".to_string(),
            )
        } else if is_conops {
            (to_display_name(&p.name), "conops".to_string())
        } else {
            (to_display_name(&p.name), to_snake_case(&p.name))
        };

        let ctx = EpicRenderCtx {
            root_pkg_name: &pkg.name,
            pkg: p,
            epic_num,
            is_root,
            is_conops,
            subsys_display,
            subsys_dir,
        };
        let (title, rel_path) = resolve_epic_title_and_path(&ctx);
        let _epic_date = resolve_date(&index.target_out_dir, &rel_path, index.today_iso);
        let epic_issue_id =
            format_issue_id_cell(resolve_existing_issue_id(&index.target_out_dir, &rel_path));
        let primary_engine = select_default_package_engine(p);

        let feat_checklist_text = build_epic_feature_checklist(&ctx, index);
        let (assoc_uc_text, assoc_us_text) = build_epic_associated_backlog(&ctx, index);
        let context_text = render_epic_context(p, &ctx.subsys_display);
        let architecture_text = render_epic_architecture(p);
        let operational_text = render_epic_operational_considerations(p);
        let governance_text = render_epic_security_and_governance(p, &index.all_requirements);
        let source_refs = resolve_epic_source_refs(&ctx, index.has_modular_schema);
        let class_diagram_block = generate_epic_class_diagram(p, &primary_engine, is_root);
        let state_diagram_block = generate_epic_statechart(p);

        let content = format!(
            r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | {epic_issue_id} |
| **Title** | {title} |
| **Type** | epic |
| **Package** | {pkg_name} |
| **Subsystem** | {subsys_display} |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# {title}

## 1. Context
{context_text}

## 2. Requirements & Checklist
{feat_checklist_text}

### Associated Use Cases & User Stories

#### Associated Use Cases
{assoc_uc_text}

#### Associated User Stories
{assoc_us_text}

## 3. Architecture
{architecture_text}

## 4. Operational Considerations
{operational_text}

## 5. Security & Governance
{governance_text}

## 6. Source References
{source_refs}

## System-Level UML Class Diagram

```mermaid
{class_diagram_block}
```

## System State Machine Diagram

```mermaid
{state_diagram_block}
```
"#,
            pkg_name = p.name,
            subsys_display = ctx.subsys_display,
        );

        generated_files.insert(rel_path, content);
    }
}
