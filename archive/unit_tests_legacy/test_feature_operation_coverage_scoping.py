#!/usr/bin/env python3
"""
Regression test suite for Check 19 Feature Operation Coverage Scoping (Issue #231).
Verifies that prose mentions of foreign parts do not trigger false missing operation errors
and that owning features are validated properly.
"""

import json
import os
import sys
import tempfile
import unittest

# Setup path
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(SCRIPT_DIR, ".."))
PARITY_AUDITOR_SRC = os.path.join(PROJECT_ROOT, "skills", "spec-orchestrator", "parity_auditor", "src")
SPEC_SCRIPTS_DIR = os.path.join(PROJECT_ROOT, "skills", "spec-orchestrator", "scripts")

for p in (SPEC_SCRIPTS_DIR, PARITY_AUDITOR_SRC, PROJECT_ROOT):
    if p not in sys.path:
        sys.path.insert(0, p)

from parity_auditor.core.workspace import WorkspaceRepository
from parity_auditor.validators.cardinality_validator import SchemaCardinalityValidator


SYSML_SCHEMA = """
package AutonomousDefenseSystem {
    part def Avenger5 {
        action ArmWeaponSystem(in armCode : String, out status : Boolean);
        action FireWeapon(in triggerId : Integer, out success : Boolean);
    }

    part def NavigationSubsystem {
        action ComputeWaypoint(in targetLat : Real, in targetLon : Real, out waypointId : Integer);
    }
}
"""

FEAT_01_AVENGER5_VALID = """---
issue_id: 101
title: "Arm and Fire Weapon System"
schema_containers:
  - path: "subsystem/Avenger5"
    node_type: "container"
---

# Feat-101: Arm and Fire Weapon System

## UML Class Diagram
```mermaid
classDiagram
    class Avenger5 {
        +ArmWeaponSystem(String armCode) Boolean
        +FireWeapon(Integer triggerId) Boolean
    }
```

## Formal Parameter Specification
| Parameter | Direction | Type |
| armCode | in | String |
| status | out | Boolean |
| triggerId | in | Integer |
| success | out | Boolean |

## Acceptance Criteria
- Given armed state, when ArmWeaponSystem is invoked with armCode: String, then status: Boolean is True.
- Given target locked, when FireWeapon is invoked with triggerId: Integer, then success: Boolean is True.
"""

FEAT_02_NAV_PROSE_MENTIONING_AVENGER5 = """---
issue_id: 102
title: "Compute Navigation Waypoints"
schema_containers:
  - path: "subsystem/NavigationSubsystem"
    node_type: "container"
---

# Feat-102: Compute Navigation Waypoints

## Description
This subsystem is nested under part def Avenger5 and interfaces with the ESAD arm-state machine.
It coordinates navigation parameters alongside the parent platform.

## UML Class Diagram
```mermaid
classDiagram
    class NavigationSubsystem {
        +ComputeWaypoint(Real targetLat, Real targetLon) Integer
    }
```

## Formal Parameter Specification
| Parameter | Direction | Type |
| targetLat | in | Real |
| targetLon | in | Real |
| waypointId | out | Integer |

## Acceptance Criteria
- Given coordinates, when ComputeWaypoint is invoked with targetLat: Real and targetLon: Real, then waypointId: Integer is returned.
"""


def _create_workspace(tmpdir, schema_content, feature_files_dict):
    pipeline_dir = os.path.join(tmpdir, ".pipeline")
    os.makedirs(pipeline_dir, exist_ok=True)
    with open(os.path.join(pipeline_dir, "schema.sysml"), "w", encoding="utf-8") as f:
        f.write(schema_content)

    rules_json = os.path.join(pipeline_dir, "codebase_rules.json")
    with open(rules_json, "w", encoding="utf-8") as f:
        json.dump({
            "meta": {"workspace": "test_workspace"},
            "backlog_directories": {
                "features": "docs/features",
                "epics": "docs/epics",
                "user_stories": "docs/user-stories",
                "use_cases": "docs/use-cases",
                "schemas": ".pipeline"
            }
        }, f)

    features_dir = os.path.join(tmpdir, "docs", "features")
    os.makedirs(features_dir, exist_ok=True)
    for fname, fcontent in feature_files_dict.items():
        with open(os.path.join(features_dir, fname), "w", encoding="utf-8") as f:
            f.write(fcontent)

    return WorkspaceRepository(tmpdir)


class TestFeatureOperationCoverageScoping(unittest.TestCase):
    """Test suite verifying scoping of feature operation coverage without false prose triggers."""

    def test_prose_mention_of_peer_part_does_not_trigger_missing_operation_error(self):
        """Feature mentioning parent/peer part in prose should not trigger false missing operation findings."""
        with tempfile.TemporaryDirectory() as tmpdir:
            repo = _create_workspace(tmpdir, SYSML_SCHEMA, {
                "feat-101-avenger5.md": FEAT_01_AVENGER5_VALID,
                "feat-102-navigation.md": FEAT_02_NAV_PROSE_MENTIONING_AVENGER5
            })
            validator = SchemaCardinalityValidator()
            errors = validator.validate_feature_operation_and_constraint_coverage(repo)
            self.assertEqual(len(errors), 0, f"Expected 0 errors, got: {[str(e) for e in errors]}")

    def test_owning_feature_missing_operation_is_correctly_flagged(self):
        """Owning feature that fails to cover all operations of its bound part is flagged."""
        incomplete_avenger5 = FEAT_01_AVENGER5_VALID.replace("FireWeapon", "OmittedOperation")
        with tempfile.TemporaryDirectory() as tmpdir:
            repo = _create_workspace(tmpdir, SYSML_SCHEMA, {
                "feat-101-avenger5.md": incomplete_avenger5,
                "feat-102-navigation.md": FEAT_02_NAV_PROSE_MENTIONING_AVENGER5
            })
            validator = SchemaCardinalityValidator()
            errors = validator.validate_feature_operation_and_constraint_coverage(repo)
            
            # Should flag FireWeapon on feat-101 (owning feature)
            missing_ops = [e for e in errors if e.rule_id == "feature-operation-missing"]
            self.assertTrue(any("FireWeapon" in str(e) and "feat-101" in str(e) for e in missing_ops))
            # feat-102 must NOT be blamed for FireWeapon
            self.assertFalse(any("feat-102" in str(e) for e in missing_ops))

    def test_explicit_mermaid_class_diagram_part_matching_without_schema_containers(self):
        """Feature declaring part in Mermaid classDiagram is matched when schema_containers is generic."""
        feat_generic_container = """---
issue_id: 103
title: "Avenger5 Direct Definition"
schema_containers:
  - path: "generic/subsystem"
    node_type: "container"
---

# Feat-103: Avenger5 Direct Definition

## UML Class Diagram
```mermaid
classDiagram
    class Avenger5 {
        +ArmWeaponSystem(String armCode) Boolean
        +FireWeapon(Integer triggerId) Boolean
    }
```

## Formal Parameter Specification
| Parameter | Direction | Type |
| armCode | in | String |
| status | out | Boolean |
| triggerId | in | Integer |
| success | out | Boolean |

## Acceptance Criteria
- Given armed state, when ArmWeaponSystem is invoked with armCode: String, then status: Boolean is True.
- Given target locked, when FireWeapon is invoked with triggerId: Integer, then success: Boolean is True.
"""
        with tempfile.TemporaryDirectory() as tmpdir:
            repo = _create_workspace(tmpdir, SYSML_SCHEMA, {
                "feat-103-avenger5.md": feat_generic_container,
                "feat-102-navigation.md": FEAT_02_NAV_PROSE_MENTIONING_AVENGER5
            })
            validator = SchemaCardinalityValidator()
            errors = validator.validate_feature_operation_and_constraint_coverage(repo)
            self.assertEqual(len(errors), 0, f"Expected 0 errors, got: {[str(e) for e in errors]}")


if __name__ == "__main__":
    unittest.main()
