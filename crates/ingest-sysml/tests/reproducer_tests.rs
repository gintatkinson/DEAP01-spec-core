//! Regression and reproduction test suite for `ingest-sysml` Markdown and SysML translators.
//!
//! Realises: [REQ-SYSML-INGEST-MD/reproducer_tests]

use ingest_sysml::cst::idl::{parse_idl_cst, IDLTranslator};
use ingest_sysml::cst::markdown::MarkdownTranslator;
use ingest_sysml::cst::yang::{parse_yang_cst, YangTranslator};
use std::time::{SystemTime, UNIX_EPOCH};

/// Verifies `MarkdownTranslator::translate_files` preserves distinct top-level packages for distinct blueprints (`BP-01`, `BP-02`) sharing `Target Crates`. Realises: [REQ-SYSML-INGEST-MD/test_issue_548_blueprint_target_crates_do_not_collapse_packages]
#[test]
fn test_issue_548_blueprint_target_crates_do_not_collapse_packages() -> Result<(), String> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let temp_dir = std::env::temp_dir().join(format!("deap_issue_548_reproducer_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).map_err(|e| format!("failed to create temp dir: {e}"))?;

    let bp1_path = temp_dir.join("blueprint-01-alpha.md");
    let bp2_path = temp_dir.join("blueprint-02-beta.md");
    let bp1_content = "# Alpha Architecture Blueprint\n\n| Attribute | Specification Detail |\n| :--- | :--- |\n| **Blueprint ID** | `BP-01` |\n| **Title** | Alpha Architecture Blueprint |\n| **Target Crates** | `crates/a, crates/b` |\n\n## 1. Normative Statement\n\nThe system shall enforce alpha architectural boundaries across workspace crates.\n";
    let bp2_content = "# Beta Architecture Blueprint\n\n| Attribute | Specification Detail |\n| :--- | :--- |\n| **Blueprint ID** | `BP-02` |\n| **Title** | Beta Architecture Blueprint |\n| **Target Crates** | `crates/a, crates/b` |\n\n## 1. Normative Statement\n\nThe system shall enforce beta architectural boundaries across workspace crates.\n";

    std::fs::write(&bp1_path, bp1_content).map_err(|e| format!("failed to write bp1: {e}"))?;
    std::fs::write(&bp2_path, bp2_content).map_err(|e| format!("failed to write bp2: {e}"))?;

    let files = vec![bp1_path.to_string_lossy().to_string(), bp2_path.to_string_lossy().to_string()];
    let result = MarkdownTranslator::new().translate_files(&files, "Blueprint_Suite");
    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir).map_err(|e| format!("failed to remove temp dir: {e}"))?;
    }

    let pkg = result?;
    assert_eq!(pkg.packages.len(), 2);
    assert!(pkg.packages.iter().any(|p| p.name.starts_with("BP_01")));
    assert!(pkg.packages.iter().any(|p| p.name.starts_with("BP_02")));
    assert!(!pkg.packages.iter().any(|p| p.name.contains("crates_a")));
    Ok(())
}

const ISSUE_550_MD: &str = "\
# Blueprint 01: Subsystem Verification Gate

| Attribute | Specification Detail |
| :--- | :--- |
| **Blueprint ID** | `BP-01` |
| **Title** | Subsystem Verification Gate |

## Part 1: Purpose & Architectural Scope
Defines the subsystem verification gate and its BOM subcomponents.

## Part 2: Normative Requirements

### REQ-SUBSYS-001: Mandatory Subcomponent Gate
The subsystem shall enforce all BOM subcomponent gates.

## Part 3: Structural Decomposition (SysML v2 Alignment)

### Component: SubsystemRoot

Bill of Materials (BOM):
- SubcomponentAlpha (Quantity 1): First BOM component enforcing alpha boundary invariants.
- **SubcomponentBeta** (Quantity 2): Second BOM component enforcing beta state transitions.
- `SubcomponentGamma` (Quantity 1): Third backtick-quoted BOM component enforcing gamma telemetry.

| Port Name | Direction | Type | Description |
| :--- | :--- | :--- | :--- |
| `cmd_in` | `in` | `CommandPayload` | Input command port |
| `status_out` | `out` | `StatusPayload` | Output status port |

## Part 4: Behavioral & Data Modeling

```mermaid
sequenceDiagram
    participant Op as Operator
    participant Gate as SubsystemRoot
    Op->>Gate: DispatchGateCheck(cmd_in)
    Gate-->>Op: EmitGateStatus(status_out)
```

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Verifying : DispatchGateCheck
    Verifying --> Verified : ValidationPassed
    Verified --> [*]
```

## Part 8: Verification & Acceptance Criteria

### AC-01: Deterministic Verification Scenario
- **Given**: A configured `SubsystemRoot` instance in `Idle` state.
- **When**: `DispatchGateCheck` is received on `cmd_in`.
- **Then**: `SubsystemRoot` transitions to `Verified` and emits `status_out`.
";

/// Verifies BOM bullet subcomponents, Mermaid sequence/state blocks, and AC-XX scenarios are extracted and preserved. Realises: [REQ-SYSML-INGEST-MD/test_issue_550_bom_bullets_and_behavioral_blocks_extracted]
#[test]
fn test_issue_550_bom_bullets_and_behavioral_blocks_extracted() -> Result<(), String> {
    let translator = MarkdownTranslator::new();
    let pkg = translator.translate(ISSUE_550_MD, "SubsystemPkg").map_err(|e| format!("translate failed: {e}"))?;
    let part_names: Vec<&str> = pkg.part_defs.iter().map(|p| p.name.as_str()).collect();
    for expected in ["SubsystemRoot", "SubcomponentAlpha", "SubcomponentBeta", "SubcomponentGamma"] {
        assert!(part_names.contains(&expected), "Missing {expected} in {part_names:?}");
    }
    for bom_name in ["SubcomponentAlpha", "SubcomponentBeta", "SubcomponentGamma"] {
        let bom_part = pkg.part_defs.iter().find(|p| p.name == bom_name).ok_or_else(|| format!("missing {bom_name}"))?;
        assert!(!bom_part.ports.is_empty() && !bom_part.attributes.is_empty());
    }
    assert!(!pkg.action_defs.is_empty() && !pkg.interaction_defs.is_empty() && !pkg.state_defs.is_empty() && !pkg.use_case_defs.is_empty());

    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let temp_dir = std::env::temp_dir().join(format!("deap_issue_550_reproducer_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).map_err(|e| format!("failed to create temp dir: {e}"))?;
    let bp_path = temp_dir.join("blueprint-01-subsystem.md");
    std::fs::write(&bp_path, ISSUE_550_MD).map_err(|e| format!("failed to write bp_path: {e}"))?;

    let consolidated_res = translator.translate_files(&[bp_path.to_string_lossy().to_string()], "ConsolidatedPkg");
    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir).map_err(|e| format!("failed to remove temp dir: {e}"))?;
    }

    let consolidated = consolidated_res.map_err(|e| format!("translate_files failed: {e}"))?;
    let child_pkg = consolidated.packages.first().unwrap_or(&consolidated);
    assert!(!child_pkg.action_defs.is_empty() && !child_pkg.interaction_defs.is_empty() && !child_pkg.state_defs.is_empty() && !child_pkg.use_case_defs.is_empty());
    Ok(())
}

const ISSUE_692_MD: &str = "\
# Blueprint 692: Sequence Diagram Compound Action and Reply Filtering

| Attribute | Specification Detail |
| :--- | :--- |
| **Blueprint ID** | `BP-692` |
| **Title** | Sequence Diagram Compound Action and Reply Filtering |

## Part 1: Purpose & Architectural Scope
Verifies deterministic SysML v2 action and interaction extraction from Mermaid sequence diagrams.

## Part 2: Normative Requirements

### REQ-0603: Deterministic Sequence Action Extraction
The translator shall extract compound solid-arrow commands into `ActionDef` nodes and exclude dashed reply arrows from `ActionDef` and synthesized `StateDef` transition triggers.

## Part 3: Structural Decomposition (SysML v2 Alignment)

### Component: Coord

| Port Name | Direction | Type | Description |
| :--- | :--- | :--- | :--- |
| `cmd_out` | `out` | `CommandPayload` | Outbound command port |

## Part 4: Behavioral & Data Modeling

```mermaid
sequenceDiagram
    participant Coord
    participant Parser
    participant Ast
    participant Tracker
    participant Compiler
    Coord->>Parser: Parse deap_core::source_ast(schema_path)
    Parser->>Ast: deap_core::source_ast::lower_unit(ast)
    Ast-->>Parser: WBS_01_03
    Parser-->>Coord: Ok(Vec<PackageDef>)
    Coord->>Tracker: Execute gh issue edit --body-file
    Tracker-->Coord: [CLOSED-LOOP VERIFIED]
    Coord->>Compiler: compile-sysml --forward-sync docs
    Compiler--xCoord: Exit 0
    Coord->>Compiler: compile-sysml --reverse-sync docs
    Compiler--)Coord: C-PURITY-03
```
";

/// Verifies compound solid-arrow commands and dashed reply arrow exclusion from `action_defs` and `state_defs`. Realises: [REQ-0603] [REQ-SYSML-INGEST-MD/test_issue_692_sequence_diagram_excludes_reply_arrows_and_preserves_compound_actions]
#[test]
fn test_issue_692_sequence_diagram_excludes_reply_arrows_and_preserves_compound_actions() -> Result<(), String> {
    let pkg = MarkdownTranslator::new().translate(ISSUE_692_MD, "Issue692Pkg").map_err(|e| format!("translate failed: {e}"))?;
    let action_names: Vec<&str> = pkg.action_defs.iter().map(|a| a.name.as_str()).collect();
    let forbidden_reply_tokens = ["Ok", "Vec", "CLOSED_LOOP", "CLOSED_LOOP_VERIFIED", "ClosedLoopVerified", "Exit_item", "Exit", "Exit0", "C_PURITY_03", "CPurity03", "WBS_01_03"];
    for reply_tok in forbidden_reply_tokens {
        assert!(!action_names.contains(&reply_tok));
    }
    assert!(!action_names.iter().any(|name| name.starts_with("reply_")));
    for truncated in ["Parse", "deap_core", "Execute", "gh", "compile_sysml", "CompileSysml"] {
        assert!(!action_names.contains(&truncated));
    }

    let expected_actions = ["ParseDeapCoreSourceAst", "deap_core_source_ast_lower_unit", "ExecuteGhIssueEditBodyFile", "CompileSysmlForwardSyncDocs", "CompileSysmlReverseSyncDocs"];
    for expected in expected_actions {
        assert!(action_names.contains(&expected));
    }
    assert_eq!(action_names.len(), expected_actions.len());
    assert_eq!(pkg.interaction_defs.len(), 1);
    let interaction = pkg.interaction_defs.first().ok_or_else(|| "missing interaction_def".to_string())?;

    for msg in &interaction.messages {
        let (msg_token, route_tail) = msg.split_once(" from ").ok_or_else(|| format!("missing ' from ': '{msg}'"))?;
        let is_valid = !msg_token.is_empty()
            && msg_token.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && msg_token.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
        assert!(is_valid, "Invalid ident '{msg_token}' in '{msg}'");
        let (sender, receiver) = route_tail.split_once(" to ").ok_or_else(|| format!("missing ' to ': '{msg}'"))?;
        assert!(!sender.is_empty() && !receiver.is_empty());
    }

    for expected_msg in [
        "ParseDeapCoreSourceAst from Coord to Parser",
        "deap_core_source_ast_lower_unit from Parser to Ast",
        "ExecuteGhIssueEditBodyFile from Coord to Tracker",
        "CompileSysmlForwardSyncDocs from Coord to Compiler",
        "CompileSysmlReverseSyncDocs from Coord to Compiler",
    ] {
        assert!(interaction.messages.iter().any(|m| m == expected_msg));
    }

    let reply_messages: Vec<&str> = interaction.messages.iter().map(String::as_str).filter(|m| m.starts_with("reply_")).collect();
    assert_eq!(reply_messages.len(), 5);
    assert_eq!(interaction.messages.len(), 10);

    for state_def in &pkg.state_defs {
        for transition in &state_def.transitions {
            if let Some(trigger_str) = transition.trigger.as_deref() {
                assert!(!forbidden_reply_tokens.contains(&trigger_str) && !trigger_str.starts_with("reply_"));
            }
        }
    }
    Ok(())
}

/// Verifies lossless CST parsing and HIR lowering for OMG IDL and IETF YANG translators (`crates/ingest-sysml/src/cst/{idl_cst,yang_cst,yang_lower}.rs`). Realises: [REQ-0201]
#[test]
fn test_idl_and_yang_cst_translators_lossless_roundtrip_and_hir_lowering() -> Result<(), String> {
    let idl_src = "module TelemetryMod {\n  typedef unsigned long SeqId;\n  struct NavPacket { double lat_deg; double lon_deg; };\n  interface FlightLink { void send_nav(in NavPacket pkt, out SeqId ack); };\n};\n";
    let idl_cst = parse_idl_cst(idl_src);
    assert!(idl_cst.lossless_roundtrip_eq);
    let idl_pkg = IDLTranslator::new().translate(idl_src, "DefaultIdl")?;
    assert_eq!(idl_pkg.name, "TelemetryMod");
    assert!(idl_pkg.part_defs.iter().any(|p| p.name == "NavPacket" && p.attributes.len() == 2));
    assert!(idl_pkg.part_defs.iter().any(|p| p.name == "FlightLink" && p.actions.len() == 1));
    assert!(idl_pkg.attribute_defs.iter().any(|a| a.name == "SeqId"));

    let yang_src = "module ietf-nav-link {\n  namespace \"urn:ietf:params:xml:ns:yang:ietf-nav-link\";\n  prefix nav;\n  container nav-state {\n    leaf altitude-m {\n      type int32;\n      units \"meters\";\n    }\n  }\n}\n";
    let yang_cst = parse_yang_cst(yang_src);
    assert!(yang_cst.lossless_roundtrip_eq);
    let yang_pkg = YangTranslator::new().translate(yang_src, "DefaultYang")?;
    assert_eq!(yang_pkg.name, "IetfNavLink");
    assert!(yang_pkg.part_defs.iter().any(|p| p.name == "NavState"));
    Ok(())
}
