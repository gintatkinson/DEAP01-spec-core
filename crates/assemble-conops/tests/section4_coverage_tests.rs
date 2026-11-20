//! Integration tests for Issue #867: ConOps Section 4 AST Part Coverage Gate, SysML SSOT precedence, and CLI error reporting.

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use assemble_conops::{
    assemble_document, extract_subsystem_parts_from_sysml, SysMLParameterBindingEngine,
};

struct TempWorkspaceGuard {
    root: PathBuf,
}

impl TempWorkspaceGuard {
    fn new(label: &str) -> Result<Self, Box<dyn Error>> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!(
            "deap_issue_867_{label}_{}_{}",
            std::process::id(),
            nanos
        ));
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    fn path(&self) -> &Path {
        &self.root
    }
}

impl Drop for TempWorkspaceGuard {
    fn drop(&mut self) {
        if self.root.exists() {
            if let Err(err) = fs::remove_dir_all(&self.root) {
                eprintln!("warning: failed to clean temp dir {}: {err}", self.root.display());
            }
        }
    }
}

#[test]
fn test_issue_867_sysml_ssot_ignores_research_and_markdown_tables_and_part_usages(
) -> Result<(), Box<dyn Error>> {
    let ws = TempWorkspaceGuard::new("ssot")?;
    let root = ws.path();
    fs::create_dir_all(root.join(".pipeline"))?;
    fs::create_dir_all(root.join("schema/extracted"))?;
    fs::create_dir_all(root.join("docs/research"))?;

    fs::write(
        root.join(".pipeline/schema.sysml"),
        concat!(
            "package SystemModel {\n",
            "    part def AlphaController {\n",
            "        part def InnerActuator {\n",
            "            out port cmd_out : ActPort;\n",
            "        }\n",
            "        part actuator_inst : InnerActuator;\n",
            "    }\n",
            "    part def BetaSensor :> BasePkg::BaseSensor {\n",
            "        in port data_in : DataPort;\n",
            "    }\n",
            "    part alpha_usage : AlphaController;\n",
            "}\n",
        ),
    )?;

    fs::write(
        root.join("docs/research/FAILURE_MODE_REGISTRY.md"),
        "| # | AST Part (`part def`) | Scope |\n| :--- | :--- | :--- |\n| 1 | `RetiredLegacyPart` | Retired |\n",
    )?;
    fs::write(
        root.join("schema/extracted/specs.md"),
        concat!(
            "| Param ID | Subsystem / Scope | Value | Units |\n",
            "| :--- | :--- | :--- | :--- |\n",
            "| P-1 | Airframe | 28.0 | V |\n\n",
            "| Interface | Connector / Part No | Spec |\n",
            "| :--- | :--- | :--- |\n",
            "| Output | Custom 8-pin | Pins |\n",
        ),
    )?;
    fs::write(
        root.join("schema/README.md"),
        "| Subsystem | Role |\n| :--- | :--- |\n| ReadmeGhostPart | Docs |\n",
    )?;

    let engine = SysMLParameterBindingEngine::new(None, None, Some(root), true, None);
    let expected: Vec<String> = ["AlphaController", "BetaSensor", "InnerActuator"]
        .into_iter()
        .map(String::from)
        .collect();
    let actual: Vec<String> = engine.ast_part_names.iter().cloned().collect();
    assert_eq!(actual, expected);
    assert!(engine
        .ast_parts
        .iter()
        .all(|p| p.source_file != "schema/DEAP_MODEL.sysml"));
    Ok(())
}

#[test]
fn test_issue_867_section4_gate_checks_only_section4_with_word_boundaries(
) -> Result<(), Box<dyn Error>> {
    let ws = TempWorkspaceGuard::new("sec4_scope")?;
    let root = ws.path();
    let units_dir = root.join("docs/conops/units/conops");
    fs::create_dir_all(root.join("schema"))?;
    fs::create_dir_all(&units_dir)?;

    fs::write(
        root.join("schema/model.sysml"),
        "package TestSystem {\n    part def Switch;\n    part def Radio;\n    part def AlphaNode;\n}\n",
    )?;
    fs::write(
        units_dir.join("01_METADATA_AND_OVERVIEW.md"),
        concat!(
            "| Attribute | Value |\n| :--- | :--- |\n",
            "| **Title** | Concept of Operations (ConOps) |\n",
            "| **Version** | 1.0.0 |\n",
            "| **Date** | 2026-11-20 |\n\n",
            "# Concept of Operations\n\n",
            "## 1. Scope\n\n",
            "Overview mentions `Switch`, `Radio`, and `AlphaNode` in Section 1 only.\n",
        ),
    )?;
    fs::write(
        units_dir.join("04_SYSTEM_ARCHITECTURE.md"),
        concat!(
            "## 4. System Operational Architecture\n\n",
            "### 4.1 Super-System Architecture\n\n",
            "```mermaid\nflowchart TD\n    EthSwitch[\"EthSwitch PORT-A\"] -->|CONN-01| RadioLink[\"RadioLink\"]\n```\n\n",
            "### 4.3 Subsystem Architecture\n\n",
            "Covers `EthSwitch` and `RadioLink` LRU subsystems only.\n",
        ),
    )?;

    let mut engine = SysMLParameterBindingEngine::new(None, None, Some(root), true, None);
    let (_doc_bad, errs_bad) = assemble_document(
        &units_dir,
        Some("Concept of Operations (ConOps)"),
        (None, None),
        &mut engine,
        None,
    );
    assert!(
        errs_bad
            .iter()
            .any(|e| e.contains("ConOps Section 4 AST Part Coverage Gate failed")),
        "Expected Section 4 gate failure when parts are only in Section 1 or substrings, got: {errs_bad:?}"
    );

    fs::write(
        units_dir.join("04_SYSTEM_ARCHITECTURE.md"),
        concat!(
            "## 4. System Operational Architecture\n\n",
            "### 4.1 Super-System Architecture\n\n",
            "```mermaid\nflowchart TD\n    Switch[\"Switch PORT-A\"] -->|CONN-01| Radio[\"Radio\"]\n```\n\n",
            "### 4.3 Subsystem Architecture\n\n",
            "Covers `Switch`, `Radio`, and `AlphaNode` LRU subsystems.\n",
        ),
    )?;
    let (_doc_ok, errs_ok) = assemble_document(
        &units_dir,
        Some("Concept of Operations (ConOps)"),
        (None, None),
        &mut engine,
        None,
    );
    assert!(
        errs_ok.is_empty(),
        "Expected Section 4 gate to pass when all part defs appear in Section 4, got: {errs_ok:?}"
    );
    Ok(())
}

#[test]
fn test_issue_867_cli_prints_section4_errors_to_stderr_on_failure() -> Result<(), Box<dyn Error>> {
    let ws = TempWorkspaceGuard::new("cli_err")?;
    let root = ws.path();
    let units_dir = root.join("docs/conops/units/conops");
    fs::create_dir_all(root.join("schema"))?;
    fs::create_dir_all(&units_dir)?;

    fs::write(
        root.join("schema/model.sysml"),
        "package TestSystem {\n    part def UncoveredModule;\n}\n",
    )?;
    fs::write(
        units_dir.join("01_METADATA_AND_OVERVIEW.md"),
        "# Overview\n\n## 1. Scope\n\nSystem scope.\n",
    )?;
    fs::write(
        units_dir.join("04_SYSTEM_ARCHITECTURE.md"),
        concat!(
            "## 4. System Operational Architecture\n\n",
            "### 4.1 Super-System Architecture\n\n",
            "```mermaid\nflowchart TD\n    A[\"NodeA\"]\n```\n\n",
            "### 4.3 Subsystem Architecture\n\n",
            "Empty LRU coverage.\n",
        ),
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_assemble-conops"))
        .arg("--verify")
        .arg(root)
        .output()?;
    assert!(!output.status.success(), "CLI should fail on missing part def");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("ConOps Section 4 AST Part Coverage Gate failed")
            && stderr.contains("UncoveredModule"),
        "Expected gate failure diagnostic on stderr, got: '{stderr}'"
    );
    Ok(())
}

#[test]
fn test_issue_867_extract_subsystem_parts_from_sysml_requires_def_handles_qualified_and_nested(
) -> Result<(), Box<dyn Error>> {
    let sysml = concat!(
        "package P {\n",
        "    part def Outer :> Pkg::Base {\n",
        "        part def NestedChild {\n",
        "            out port sig_out : Pkg::SigPort;\n",
        "        }\n",
        "        part nestedUsage : NestedChild;\n",
        "    }\n",
        "    part topUsage : Outer;\n",
        "}\n",
    );
    let parts = extract_subsystem_parts_from_sysml(sysml, "schema/model.sysml");
    let names: Vec<&str> = parts.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, vec!["Outer", "NestedChild"]);
    let outer = parts.first().ok_or("missing Outer")?;
    let nested = parts.get(1).ok_or("missing NestedChild")?;
    assert_eq!(outer.source_line, Some(2));
    assert!(
        outer.ports.is_empty(),
        "Outer should not steal NestedChild ports, got: {:?}",
        outer.ports
    );
    assert_eq!(nested.source_line, Some(3));
    assert_eq!(nested.ports.len(), 1);
    assert_eq!(nested.ports.first().map(|p| p.name.as_str()), Some("sig_out"));
    assert_eq!(
        nested.ports.first().map(|p| p.type_name.as_str()),
        Some("Pkg::SigPort")
    );
    Ok(())
}

#[test]
fn test_issue_867_auto_discover_stops_at_workspace_root_boundary() -> Result<(), Box<dyn Error>> {
    let ws = TempWorkspaceGuard::new("ws_boundary")?;
    let parent_root = ws.path();
    let child_ws = parent_root.join("nested_project");
    fs::create_dir_all(parent_root.join("schema"))?;
    fs::create_dir_all(child_ws.join("schema"))?;
    fs::write(
        parent_root.join("schema/parent.sysml"),
        "package ParentPkg { part def ParentLeakPart; }\n",
    )?;
    fs::write(
        child_ws.join("schema/child.sysml"),
        "package ChildPkg { part def ChildOnlyPart; }\n",
    )?;

    let engine = SysMLParameterBindingEngine::new(None, None, Some(&child_ws), true, None);
    let actual: Vec<String> = engine.ast_part_names.iter().cloned().collect();
    assert_eq!(actual, vec!["ChildOnlyPart".to_string()]);
    Ok(())
}
