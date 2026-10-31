//! Feature specification (`docs/features/FEAT-*.md`) synthesis from indexed SysML v2 `PartDef` nodes.
//!
//! Safety & Traceability: Realises: [REQ-SYNC-FORWARD-FEATURE-EMITTER]
//! Contract: Projects each indexed `PartContext` into a canonical Feature Markdown specification containing a Mermaid `classDiagram`, JSON Schema test data shape, validation constraints, 3-layer Definition of Done sections, and constraint-derived BDD acceptance criteria.
//! Bounds & Faults: Uses deterministic `BTreeSet` deduplication, saturating index arithmetic (`saturating_add`), and non-panicking tracker issue ID resolution.

use super::*;

/// Collects deduplicated `RequirementDef` nodes explicitly satisfied by `part_name` or unscoped within `pkg_name`.
/// Realises: [REQ-SYNC-FORWARD-FEATURE-EMITTER]
fn collect_feature_matching_requirements<'a>(
    part_name: &str,
    pkg_name: &str,
    index: &'a ForwardSyncIndex<'a>,
) -> Vec<&'a RequirementDef> {
    let owning_pkg = index
        .all_packages
        .iter()
        .copied()
        .find(|p| p.name == pkg_name);
    let mut matching_reqs: Vec<&'a RequirementDef> = Vec::new();
    let mut seen_matching_reqs: BTreeSet<String> = BTreeSet::new();
    for r in index
        .all_requirements
        .iter()
        .filter(|r| r.satisfied_by.iter().any(|s| s == part_name))
        .chain(
            owning_pkg
                .into_iter()
                .flat_map(|p| p.requirement_defs.iter())
                .filter(|r| r.satisfied_by.is_empty()),
        )
    {
        let key = format_requirement_label(r);
        if !key.is_empty() && seen_matching_reqs.insert(key) {
            matching_reqs.push(r);
        }
    }
    matching_reqs
}

/// Renders `### 4. Interactive Flow & States` operation and parameterized action bullets for `part`.
/// Realises: [REQ-SYNC-FORWARD-FEATURE-EMITTER]
fn render_feature_interactive_flow(part: &PartDef) -> String {
    let mut emitted_part_ops = BTreeSet::new();
    let mut op_action_bullets = Vec::new();
    for op in &part.operations {
        let ret = op.return_type.as_deref().unwrap_or("void");
        if emitted_part_ops.insert(op.name.as_str()) {
            op_action_bullets.push(format!("- `+{}() : {}`", op.name, ret));
        }
    }
    for act in &part.actions {
        if emitted_part_ops.insert(act.name.as_str()) {
            op_action_bullets.push(format_markdown_operation_bullet(act));
        }
    }
    if op_action_bullets.is_empty() {
        format!(
            "Operational lifecycle events and discrete state transitions for {}.",
            part.name
        )
    } else {
        format!(
            "Operational lifecycle events and discrete state transitions for {}.\n{}",
            part.name,
            op_action_bullets.join("\n")
        )
    }
}

/// Resolves `(spec_source, source_refs)` Markdown links for a Feature based on modular vs single-file schema presence.
/// Realises: [REQ-SYNC-FORWARD-FEATURE-EMITTER]
fn resolve_feature_sources(has_modular_schema: bool, subsys_dir: &str) -> (String, String) {
    let source_refs = if !has_modular_schema {
        r#"- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 15288:2023 §6.4.3 Architecture Definition Process"#
            .to_string()
    } else if subsys_dir == "conops" {
        r#"- ConOps Operational Actors: `schema/conops/actors.sysml`
- ConOps Operational Activities: `schema/conops/activities.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations"#
            .to_string()
    } else if subsys_dir == "architecture" {
        r#"- System Architecture Model: `schema/model.sysml`
- ConOps Operational Activities: `schema/conops/activities.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 15288:2023 §6.4.3 Architecture Definition Process"#
            .to_string()
    } else {
        format!(
            r#"- Subsystem Requirements: `schema/subsystems/{subsys_dir}/requirements.sysml`
- Subsystem Architecture: `schema/subsystems/{subsys_dir}/architecture.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 15288:2023 §6.4.3 Architecture Definition Process"#
        )
    };

    let spec_source = if !has_modular_schema || subsys_dir == "architecture" {
        "[schema/model.sysml](../../schema/model.sysml)".to_string()
    } else if subsys_dir == "conops" {
        "[schema/conops/actors.sysml](../../schema/conops/actors.sysml)".to_string()
    } else {
        format!(
            "[schema/subsystems/{subsys_dir}/architecture.sysml](../../schema/subsystems/{subsys_dir}/architecture.sysml)"
        )
    };

    (spec_source, source_refs)
}

/// Synthesizes all `features/FEAT-XX-*.md` specifications from `index.part_contexts` into `generated_files`.
/// Realises: [REQ-SYNC-FORWARD-FEATURE-EMITTER]
pub(crate) fn emit_feature_specs(
    pkg: &PackageDef,
    index: &ForwardSyncIndex<'_>,
    generated_files: &mut BTreeMap<String, String>,
) {
    for (idx, ctx) in index.part_contexts.iter().enumerate() {
        let f_num = idx.saturating_add(1);
        let part_name = &ctx.part.name;
        let rel_path = format!("features/FEAT-{:02}-{}.md", f_num, part_name);
        let _feat_date = resolve_date(&index.target_out_dir, &rel_path, index.today_iso);
        let feat_issue_id =
            format_issue_id_cell(resolve_existing_issue_id(&index.target_out_dir, &rel_path));
        let title = format!(
            "Feature {:02}: [{}] {}",
            f_num, ctx.subsys_display, part_name
        );

        let class_diagram_block = generate_class_diagram(ctx.part);
        let json_schema = generate_json_schema(ctx.part);
        let matching_reqs = collect_feature_matching_requirements(part_name, &ctx.pkg_name, index);

        let bdd_criteria = generate_feature_bdd(ctx.part, &matching_reqs);
        let validation_section =
            render_feature_validation_and_constraints(ctx.part, &matching_reqs);
        let arrangement_section = render_feature_logical_arrangement(ctx.part);
        let layer1_section = render_feature_layer1_state_model(ctx.part);
        let layer2_section = render_feature_layer2_logic_management(ctx.part, &matching_reqs);
        let layer3_section = render_feature_layer3_interface_binding(ctx.part);
        let interactive_flow_section = render_feature_interactive_flow(ctx.part);
        let (spec_source, source_refs) =
            resolve_feature_sources(index.has_modular_schema, &ctx.subsys_dir);

        let parent_epic_link = if ctx.pkg_name == pkg.name {
            format!(
                "[EPIC-01-{p_name}](../epics/EPIC-01-{p_name}.md)",
                p_name = pkg.name
            )
        } else {
            resolve_parent_epic_link(&index.ordered_packages, &ctx.pkg_name)
        };
        let interface_type = "Physical / Structural / Internal Bus";
        let schema_container = format!("{}::{}", ctx.pkg_name, part_name);

        let content = format!(
            r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | {feat_issue_id} |
| **Title** | {title} |
| **Type** | feature |
| **Parent Epic** | {parent_epic_link} |
| **Interface Type** | {interface_type} |
| **Schema Containers** | `{schema_container}` |
| **Generation Mode** | subagent |
| **Specification Source** | {spec_source} |

# {title}

## UML Class Diagram

```mermaid
{class_diagram_block}
```

## Interface Requirements
### 1. Test Data Shape
```json
{json_schema}
```

### 2. Validation & Constraints
{validation_section}

### 3. Visual Layout & Arrangement
{arrangement_section}

### 4. Interactive Flow & States
{interactive_flow_section}

### Layer 1: Domain State & Signal Model
{layer1_section}

### Layer 2: Logic & Safety State Management
{layer2_section}

### Layer 3: Presentation & Actuator Interface Binding
{layer3_section}

## Acceptance Criteria (BDD)
{bdd_criteria}

## Source References
Formal component specifications and normative systems engineering requirements:
{source_refs}

Component behavior and interface definitions derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 15288.
"#
        );

        generated_files.insert(rel_path, content);
    }
}
