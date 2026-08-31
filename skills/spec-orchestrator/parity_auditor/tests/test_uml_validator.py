#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Unit tests for UmlValidator placeholder detection in specification files.

Verifies that unreplaced template placeholder tokens (e.g. [Feat-ID], [Feature Title],
[EpicID], [US-ID], [Use Case ID], {{REQUIRED_JUSTIFICATION}}, etc.) trigger parity auditor
validation failures across Epics, Features, User Stories, and Use Cases.
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
from parity_auditor.validators.uml import UmlValidator

SAMPLE_SYSML_MODEL = """
package AutonomousUAS_SSOT {
    doc /* SSOT for Autonomous UAS Infrastructure Safety */
    part def FlightGuidanceComputer {
        doc /* Flight guidance and supervisory controller */
        attribute activeRouteId : String;
    }
}
"""

VALID_EPIC = """---
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

### Associated Use Cases & User Stories

#### Associated Use Cases
- [ ] #102 - Trajectory Deconfliction (https://github.com/gintatkinson/DEAP-uas-infrastructure-safety/blob/main/docs/use-cases/uc-01-trajectory-deconfliction.md) Flight trajectory deconfliction

#### Associated User Stories
- [ ] #103 - Avoid Obstacle (https://github.com/gintatkinson/DEAP-uas-infrastructure-safety/blob/main/docs/user-stories/us-01-avoid-obstacle.md) Emergency avoidance maneuver

## 3. Architecture

### Subsystem Capability Allocations
| Capability Name | Subsystem Package | Description / Objective |
| --- | --- | --- |
| AutonomousCollisionAvoidance | FlightGuidance | Executes real-time detect-and-avoid trajectory deconfliction |

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

## State Machine Definitions

## System State Machine Diagram
```mermaid
stateDiagram-v2
    [*] --> Standby
```

## 4. Operational Considerations
Real-time embedded execution.

## 5. Security & Governance
DO-178C Level A assurance.

## Specification Context
Normative schema definition.

## 6. Source References
Structural Schema: [model.sysml](https://raw.githubusercontent.com/gintatkinson/DEAP-uas-infrastructure-safety/main/schema/model.sysml) (Clause: 3.1)
"""

VALID_FEATURE = """---
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
Flight guidance controller component.

## UML Class Diagram
```mermaid
classDiagram
    class FlightGuidanceComputer {
        +String activeRouteId "[1]"
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
- ThermalLimit: coreTemperature <= 85.0

### 3. Logical Operations & Interface Messages
- `+ExecuteManeuver(in targetHeading : Float)`: Executes maneuver.

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


def _create_mock_repo(tmp_dir, epics=None, features=None, user_stories=None, use_cases=None):
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
    with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
        f.write(SAMPLE_SYSML_MODEL)

    if epics:
        epics_dir = os.path.join(tmp_dir, "docs", "epics")
        os.makedirs(epics_dir, exist_ok=True)
        for fname, content in epics.items():
            with open(os.path.join(epics_dir, fname), "w", encoding="utf-8") as f:
                f.write(content)

    if features:
        features_dir = os.path.join(tmp_dir, "docs", "features")
        os.makedirs(features_dir, exist_ok=True)
        for fname, content in features.items():
            with open(os.path.join(features_dir, fname), "w", encoding="utf-8") as f:
                f.write(content)

    if user_stories:
        stories_dir = os.path.join(tmp_dir, "docs", "user-stories")
        os.makedirs(stories_dir, exist_ok=True)
        for fname, content in user_stories.items():
            with open(os.path.join(stories_dir, fname), "w", encoding="utf-8") as f:
                f.write(content)

    if use_cases:
        uc_dir = os.path.join(tmp_dir, "docs", "use-cases")
        os.makedirs(uc_dir, exist_ok=True)
        for fname, content in use_cases.items():
            with open(os.path.join(uc_dir, fname), "w", encoding="utf-8") as f:
                f.write(content)

    return WorkspaceRepository(tmp_dir)


@pytest.mark.parametrize("placeholder_token", [
    "[Feat-ID]",
    "[Feature Title]",
    "[EpicID]",
    "[US-ID]",
    "[Use Case ID]",
    "#[IssueID]",
    "#[EpicIssueID]",
    "[Epic Title]",
    "[User Story Title]",
    "[Use Case Title]",
    "{{REQUIRED_JUSTIFICATION}}",
    "{{REQUIRED_SOURCE_REF}}",
    "[Repository Base URL]",
    "[Branch Name]",
    "<blob_path>",
])
def test_epic_rejects_unreplaced_placeholder_tokens(placeholder_token):
    """Assert that UmlValidator rejects Epics containing unreplaced placeholder tokens."""
    invalid_epic = VALID_EPIC.replace(
        "Flight guidance and trajectory control subsystem.",
        f"Flight guidance context with placeholder {placeholder_token}."
    )
    with tempfile.TemporaryDirectory() as tmp_dir:
        repo = _create_mock_repo(tmp_dir, epics={"epic-01-flight-guidance.md": invalid_epic})
        validator = UmlValidator()
        errors = validator.validate(repo)
        assert any(
            e.rule_id in ("specification-must-not-contain-template-placeholders", "epic-prohibit-unreplaced-placeholder-text")
            for e in errors
        ), f"Expected validation failure for token '{placeholder_token}', but got errors: {errors}"


@pytest.mark.parametrize("placeholder_token", [
    "[Feat-ID]",
    "[Feature Title]",
    "[EpicID]",
    "[US-ID]",
    "[Use Case ID]",
    "#[IssueID]",
    "#[EpicIssueID]",
    "{{REQUIRED_JUSTIFICATION}}",
    "{{REQUIRED_SOURCE_REF}}",
])
def test_feature_rejects_unreplaced_placeholder_tokens(placeholder_token):
    """Assert that UmlValidator rejects Features containing unreplaced placeholder tokens."""
    invalid_feature = VALID_FEATURE.replace(
        "Flight guidance controller component.",
        f"Flight guidance controller component with {placeholder_token} placeholder."
    )
    with tempfile.TemporaryDirectory() as tmp_dir:
        repo = _create_mock_repo(tmp_dir, features={"feat-01-flight-guidance-computer.md": invalid_feature})
        validator = UmlValidator()
        errors = validator.validate(repo)
        assert any(
            e.rule_id == "specification-must-not-contain-template-placeholders"
            for e in errors
        ), f"Expected validation failure for token '{placeholder_token}', but got errors: {errors}"


def test_check18_epic_rejects_unreplaced_secondary_checklist_tokens():
    """Verify UmlValidator rejects Epics with unreplaced tokens in secondary checklists (Issue #58)."""
    invalid_epic = VALID_EPIC + "\n#### Associated Use Cases\n- [ ] #[IssueID] - [UC-001: Example](https://github.com/org/repo/blob/main/docs/use-cases/uc-01.md) (semantic linkage justification)\n"
    with tempfile.TemporaryDirectory() as tmp_dir:
        repo = _create_mock_repo(tmp_dir, epics={"epic-01-flight-guidance.md": invalid_epic})
        validator = UmlValidator()
        errors = validator.validate(repo)
        assert any(
            e.rule_id in ("epic-prohibit-unreplaced-placeholder-text", "specification-must-not-contain-template-placeholders")
            for e in errors
        ), f"Expected error for secondary checklist placeholder tokens, got: {errors}"
