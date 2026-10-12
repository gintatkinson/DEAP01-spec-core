//! deap-core: Shared library crate across DEAP pipeline utilities.

pub mod diagnostics;
pub mod markdown;
pub mod rules;
pub mod sysml_ast;
pub mod workspace;

pub use diagnostics::{DefectDossier, Diagnostic, DiagnosticSeverity, WhyEntry};
pub use markdown::{
    check_latex_katex_syntax, check_mermaid_syntax, strip_code_and_glfm, strip_fenced_code_blocks,
    strip_glfm_inline_math, strip_inline_code, ALLOWED_ALIGNMENT_ENVS, VALID_MERMAID_HEADERS,
};
pub use rules::{
    CodebaseRules, MetaConfig, TrackerRules, ValidationRules, EXCLUDED_DIRS, MASTER_BLUEPRINTS,
};
pub use sysml_ast::{
    parse_action_defs, parse_attribute_defs, parse_connection_defs, parse_constraint_defs,
    parse_part_defs, parse_port_defs, parse_requirement_defs, parse_state_defs, parse_sysml,
    ActionDef, AttributeDef, CapabilityDef, ConnectionDef, ConstraintDef, FlowDef, HazardDef,
    ImportDef, InteractionDef, ItemDef, OperationDef, PackageDef, PartDef, PortDef, RequirementDef, RiskDef,
    StateDef, SysmlModel, TestCaseDef, TransitionDef, UseCaseDef,
};
pub use workspace::{
    check_downstream_instructions_exist, check_gitignore_exists, check_no_ds_store_files,
    check_no_duplicate_master_blueprints, check_reconcile_backlog_tooling_exists,
    check_upstream_template_clean_landing_zones, is_upstream_compiler,
};
