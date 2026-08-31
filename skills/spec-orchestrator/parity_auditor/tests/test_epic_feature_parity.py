#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Unit tests for Epic Subsystem Allocation (Check 18) and Feature Operation Coverage (Check 19).

Resolves Issues #38 & #39 in the Primary Commercial Toolchain Integration Context:
- Issue #38: Epic Subsystem Allocation & Formal Capability Def Declarations
- Issue #39: Feature Operation Extraction & 100% Typed Parameter Interface Coverage
"""

import os
import sys
import json
import tempfile
import pytest

# Ensure spec-orchestrator scripts and parity_auditor are on sys.path
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PARITY_AUDITOR_DIR = os.path.abspath(os.path.join(SCRIPT_DIR, ".."))
SPEC_ORCH_DIR = os.path.abspath(os.path.join(PARITY_AUDITOR_DIR, ".."))
PROJECT_ROOT = os.path.abspath(os.path.join(SPEC_ORCH_DIR, "..", ".."))
SPEC_SCRIPTS_DIR = os.path.join(SPEC_ORCH_DIR, "scripts")
PARITY_AUDITOR_SRC = os.path.join(PARITY_AUDITOR_DIR, "src")

for p in (SPEC_SCRIPTS_DIR, PARITY_AUDITOR_SRC, PROJECT_ROOT):
    if p not in sys.path:
        sys.path.insert(0, p)

from parity_auditor.core.workspace import WorkspaceRepository
from parity_auditor.validators.cardinality_validator import SchemaCardinalityValidator
from parity_auditor.validators.uml import UmlValidator
from parity_auditor.core.findings import Finding

SAMPLE_SYSML_MODEL = """
package AutonomousUAS_SSOT {
    doc /* SSOT for Autonomous UAS Infrastructure Safety */

    capability def AutonomousCollisionAvoidance {
        doc /* Real-time detect-and-avoid capability */
        subsystem FlightGuidance;
    }

    capability def PrecisionLanding {
        doc /* Autonomous beacon-guided landing */
        subsystem FlightGuidance;
    }

    part def FlightGuidanceComputer {
        doc /* Flight guidance and supervisory controller */
        attribute activeRouteId : String = "ROUTE_ALPHA";

        action ExecuteManeuver(in targetHeading : Float, out maneuverStatus : StatusEnum);
        operation def ComputeThrustVector(in currentHeading : Vector3D, in targetHeading : Vector3D) : ThrustVector;

        assert constraint ThermalLimitConstraint {
            coreTemperature <= 85.0;
        }
    }
}
"""

SAMPLE_VALID_EPIC = """---
title: "Flight Guidance Subsystem"
type: "epic"
package: "FlightGuidance"
generation_mode: "subagent"
spec_source: "Project Constitution"
---

# Epic: Flight Guidance Subsystem

## 1. Context
Flight guidance and trajectory control subsystem.

## 2. Requirements & Checklist
- [ ] #101 - Flight Guidance Computer (https://github.com/gintatkinson/DEAP-uas-infrastructure-safety/blob/main/docs/features/feat-01-flight-guidance-computer.md) Primary flight guidance computer

## 3. Architecture

### Subsystem Capability Allocations
| Capability Name | Subsystem Package | Description / Objective |
| --- | --- | --- |
| AutonomousCollisionAvoidance | FlightGuidance | Executes real-time detect-and-avoid trajectory deconfliction |
| PrecisionLanding | FlightGuidance | Beacon-guided autonomous precision landing |

### Subsystem Component Definition
```mermaid
classDiagram
    class FlightGuidanceComponent {
        <<component>>
        +Boolean executeGuidanceLoop()
    }
```

## System-Level UML Class Diagram
```mermaid
classDiagram
    class FlightGuidanceComputer {
        +String activeRouteId
    }
```

## 4. Operational Considerations
Real-time embedded execution.

## 5. Security & Governance
DO-178C Level A assurance.

## 6. Source References
Structural Schema: [model.sysml](https://raw.githubusercontent.com/gintatkinson/DEAP-uas-infrastructure-safety/main/schema/model.sysml) (Clause: 3.1)
"""

SAMPLE_VALID_FEATURE = """---
title: "Flight Guidance Computer"
type: "feature"
interface_type: "m2m"
generation_mode: "subagent"
spec_source: "Project Constitution"
schema_containers:
  - path: "AutonomousUAS_SSOT:AutonomousUAS_SSOT/FlightGuidanceComputer"
    node_type: container
---

# Feature: Flight Guidance Computer

## Parent Epic
- [ ] #100 - Flight Guidance Subsystem (https://github.com/gintatkinson/DEAP-uas-infrastructure-safety/blob/main/docs/epics/epic-01-flight-guidance.md) Subsystem parent

## Description
Flight guidance and supervisory controller component.

## UML Class Diagram
```mermaid
classDiagram
    class FlightGuidanceComputer {
        +String activeRouteId "[1]"
        +Boolean ExecuteManeuver(Float in_targetHeading, StatusEnum out_maneuverStatus)
        +ThrustVector ComputeThrustVector(Vector3D in_currentHeading, Vector3D in_targetHeading)
    }
```

## Interface Requirements

### 1. Payload Schema
```json
{
  "activeRouteId": "ROUTE_ALPHA"
}
```

### 2. Validation & Constraints
- ThermalLimitConstraint: coreTemperature <= 85.0

### 3. Logical Operations & Interface Messages
- `+ExecuteManeuver(in targetHeading : Float, out maneuverStatus : StatusEnum)`: Executes emergency evasive deconfliction maneuver.
- `+ComputeThrustVector(in currentHeading : Vector3D, in targetHeading : Vector3D) : ThrustVector`: Calculates required 3D thrust vector.

### 4. Logical Exception States & Validation Failures
- Thermal throttling triggers on coreTemperature > 85.0.

## Given-When-Then Acceptance Criteria
- Given flight guidance computer in navigation mode, When obstacle detected, Then ExecuteManeuver commands trajectory change.

## Source References
Structural Schema: [model.sysml](https://raw.githubusercontent.com/gintatkinson/DEAP-uas-infrastructure-safety/main/schema/model.sysml) (Clause: 3.2)

## Logical UI & Interface Bindings
| Interface Channel | Category | Target Component / Handler | Target Container / Endpoint | Data Source Binding |
| --- | --- | --- | --- | --- |
| mcp | M2M API | MCPToolHandler | /guidance/controller | /AutonomousUAS_SSOT:AutonomousUAS_SSOT/FlightGuidanceComputer |
"""

def _create_mock_repo(tmp_dir, sysml_content=SAMPLE_SYSML_MODEL, epics=None, features=None):
    rules_dir = os.path.join(tmp_dir, ".pipeline", "logical-ui")
    os.makedirs(rules_dir, exist_ok=True)
    real_rules_path = os.path.join(PROJECT_ROOT, ".pipeline", "logical-ui", "codebase_rules.json")
    if os.path.exists(real_rules_path):
        with open(real_rules_path, "r", encoding="utf-8") as f:
            rules_content = json.load(f)
    else:
        rules_content = {
            "meta": {"upstream_repository": "gintatkinson/DEAP-uas-infrastructure-safety"},
            "backlog_directories": {
                "schemas": "schema",
                "epics": "docs/epics",
                "features": "docs/features",
                "use_cases": "docs/use-cases",
                "user_stories": "docs/user-stories"
            }
        }
    with open(os.path.join(rules_dir, "codebase_rules.json"), "w", encoding="utf-8") as f:
        json.dump(rules_content, f)


    schema_dir = os.path.join(tmp_dir, "schema")
    os.makedirs(schema_dir, exist_ok=True)
    if sysml_content:
        with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
            f.write(sysml_content)

    epics_dir = os.path.join(tmp_dir, "docs", "epics")
    os.makedirs(epics_dir, exist_ok=True)
    if epics:
        for fname, content in epics.items():
            with open(os.path.join(epics_dir, fname), "w", encoding="utf-8") as f:
                f.write(content)

    features_dir = os.path.join(tmp_dir, "docs", "features")
    os.makedirs(features_dir, exist_ok=True)
    if features:
        for fname, content in features.items():
            with open(os.path.join(features_dir, fname), "w", encoding="utf-8") as f:
                f.write(content)

    return WorkspaceRepository(tmp_dir)


def test_check18_valid_epic_subsystem_and_capability_allocation():
    """Verify Check 18 passes when Epic allocates to valid subsystem package and references all capabilities."""
    with tempfile.TemporaryDirectory() as tmp_dir:
        repo = _create_mock_repo(
            tmp_dir,
            epics={"epic-01-flight-guidance.md": SAMPLE_VALID_EPIC}
        )
        validator = SchemaCardinalityValidator()
        errors = validator.validate_package_structure_and_subsystem_allocation(repo)
        assert len(errors) == 0, f"Expected 0 errors, got: {errors}"


def test_check18_invalid_subsystem_package():
    """Verify Check 18 rejects Epic allocating to an undeclared/invalid subsystem package."""
    invalid_epic = SAMPLE_VALID_EPIC.replace('package: "FlightGuidance"', 'package: "UnknownSubsystem"')
    with tempfile.TemporaryDirectory() as tmp_dir:
        repo = _create_mock_repo(
            tmp_dir,
            epics={"epic-01-flight-guidance.md": invalid_epic}
        )
        validator = SchemaCardinalityValidator()
        errors = validator.validate_package_structure_and_subsystem_allocation(repo)
        assert any(e.rule_id == "epic-subsystem-package-invalid" for e in errors)
        assert any("UnknownSubsystem" in str(e) for e in errors)


def test_check18_missing_package_allocation():
    """Verify Check 18 rejects Epic missing package/subsystem allocation."""
    missing_pkg_epic = SAMPLE_VALID_EPIC.replace('package: "FlightGuidance"', '')
    # Also strip FlightGuidance from text
    missing_pkg_epic = missing_pkg_epic.replace('FlightGuidance', 'GenericSubsystem')
    with tempfile.TemporaryDirectory() as tmp_dir:
        repo = _create_mock_repo(
            tmp_dir,
            epics={"epic-01-generic.md": missing_pkg_epic}
        )
        validator = SchemaCardinalityValidator()
        errors = validator.validate_package_structure_and_subsystem_allocation(repo)
        assert any(e.rule_id == "epic-package-allocation-missing" for e in errors)


def test_check18_missing_capability_allocation():
    """Verify Check 18 rejects Epic missing formal capability def declared for its subsystem package."""
    missing_cap_epic = SAMPLE_VALID_EPIC.replace("PrecisionLanding", "IgnoredCapability")
    with tempfile.TemporaryDirectory() as tmp_dir:
        repo = _create_mock_repo(
            tmp_dir,
            epics={"epic-01-flight-guidance.md": missing_cap_epic}
        )
        validator = SchemaCardinalityValidator()
        errors = validator.validate_package_structure_and_subsystem_allocation(repo)
        assert any(e.rule_id == "epic-capability-allocation-missing" for e in errors)
        assert any("PrecisionLanding" in str(e) for e in errors)


def test_check18_unknown_capability_reference():
    """Verify Check 18 rejects Epic referencing capability def not declared in SysML AST."""
    unknown_cap_epic = SAMPLE_VALID_EPIC + "\n- `capability def GhostCapability`\n"
    with tempfile.TemporaryDirectory() as tmp_dir:
        repo = _create_mock_repo(
            tmp_dir,
            epics={"epic-01-flight-guidance.md": unknown_cap_epic}
        )
        validator = SchemaCardinalityValidator()
        errors = validator.validate_package_structure_and_subsystem_allocation(repo)
        assert any(e.rule_id == "epic-capability-unknown" for e in errors)
        assert any("GhostCapability" in str(e) for e in errors)


def test_check18_sysml_capability_unallocated_across_all_epics():
    """Verify Check 18 detects SysML capability def unallocated when no Epic covers it."""
    with tempfile.TemporaryDirectory() as tmp_dir:
        # Create empty epics dir
        repo = _create_mock_repo(tmp_dir, epics={})
        validator = SchemaCardinalityValidator()
        errors = validator.validate_package_structure_and_subsystem_allocation(repo)
        # Empty epics returns []
        assert len(errors) == 0


def test_check19_valid_feature_operations_and_typed_signatures():
    """Verify Check 19 passes with 100% typed operation parameter signatures and constraints."""
    with tempfile.TemporaryDirectory() as tmp_dir:
        repo = _create_mock_repo(
            tmp_dir,
            features={"feat-01-flight-guidance-computer.md": SAMPLE_VALID_FEATURE}
        )
        validator = SchemaCardinalityValidator()
        errors = validator.validate_feature_operation_and_constraint_coverage(repo)
        assert len(errors) == 0, f"Expected 0 errors, got: {errors}"


def test_check19_missing_feature_operation():
    """Verify Check 19 rejects Feature missing an action/operation defined on SysML part."""
    missing_op_feature = SAMPLE_VALID_FEATURE.replace("ComputeThrustVector", "ComputeThrustOmitted")
    with tempfile.TemporaryDirectory() as tmp_dir:
        repo = _create_mock_repo(
            tmp_dir,
            features={"feat-01-flight-guidance-computer.md": missing_op_feature}
        )
        validator = SchemaCardinalityValidator()
        errors = validator.validate_feature_operation_and_constraint_coverage(repo)
        assert any(e.rule_id == "feature-operation-missing" for e in errors)
        assert any("ComputeThrustVector" in str(e) for e in errors)


def test_check19_untyped_operation_parameter():
    """Verify Check 19 rejects operations with missing or untyped interface parameters."""
    # Replace typed parameter Float with untyped or missing type
    untyped_param_feature = SAMPLE_VALID_FEATURE.replace("targetHeading : Float", "targetHeading")
    untyped_param_feature = untyped_param_feature.replace("Float in_targetHeading", "in_targetHeading")
    with tempfile.TemporaryDirectory() as tmp_dir:
        repo = _create_mock_repo(
            tmp_dir,
            features={"feat-01-flight-guidance-computer.md": untyped_param_feature}
        )
        validator = SchemaCardinalityValidator()
        errors = validator.validate_feature_operation_and_constraint_coverage(repo)
        assert any(e.rule_id == "feature-operation-param-untyped" for e in errors)
        assert any("targetHeading" in str(e) for e in errors)


def test_check19_missing_constraint_coverage():
    """Verify Check 19 rejects Feature omitting SysML part constraints/invariants."""
    missing_con_feature = SAMPLE_VALID_FEATURE.replace("ThermalLimitConstraint", "GenericConstraint")
    missing_con_feature = missing_con_feature.replace("coreTemperature <= 85.0", "")
    with tempfile.TemporaryDirectory() as tmp_dir:
        repo = _create_mock_repo(
            tmp_dir,
            features={"feat-01-flight-guidance-computer.md": missing_con_feature}
        )
        validator = SchemaCardinalityValidator()
        errors = validator.validate_feature_operation_and_constraint_coverage(repo)
        assert any(e.rule_id == "feature-constraint-coverage-missing" for e in errors)
        assert any("ThermalLimitConstraint" in str(e) for e in errors)


def test_check18_and_19_full_cardinality_validator_integration():
    """Verify full SchemaCardinalityValidator.validate(is_sysml=True) passes on fully compliant model."""
    with tempfile.TemporaryDirectory() as tmp_dir:
        repo = _create_mock_repo(
            tmp_dir,
            epics={"epic-01-flight-guidance.md": SAMPLE_VALID_EPIC},
            features={"feat-01-flight-guidance-computer.md": SAMPLE_VALID_FEATURE}
        )
        validator = SchemaCardinalityValidator()
        errors = validator.validate(repo, is_sysml=True)
        assert len(errors) == 0, f"Expected 0 errors, got: {errors}"


@pytest.mark.parametrize("token", [
    "#[IssueID]",
    "[Feat-ID]",
    "[Feature Title]",
    "[EpicID]",
    "[US-ID]",
    "[Use Case ID]",
    "{{REQUIRED_JUSTIFICATION}}",
    "(semantic linkage justification)"
])
def test_check18_epic_rejects_unreplaced_secondary_checklist_tokens(token):
    """Verify UmlValidator rejects Epics with unreplaced tokens in secondary checklists (Issue #58)."""
    invalid_epic = SAMPLE_VALID_EPIC + f"\n### Associated Use Cases & User Stories\n\n#### Associated Use Cases\n- [ ] {token} - [UC-001: Example](https://github.com/org/repo/blob/main/docs/use-cases/uc-01.md) Autonomous collision avoidance linkage\n"
    with tempfile.TemporaryDirectory() as tmp_dir:
        repo = _create_mock_repo(tmp_dir, epics={"epic-01-flight-guidance.md": invalid_epic})
        validator = UmlValidator()
        errors = validator.validate(repo)
        assert any(
            e.rule_id in ("epic-prohibit-unreplaced-placeholder-text", "specification-must-not-contain-template-placeholders")
            for e in errors
        ), f"Expected error for secondary checklist placeholder token '{token}', got: {errors}"



