#!/usr/bin/env python3
"""
Unit tests for Closed-Grammar Metamodel Allowlist Type System & Context Sandboxing (Issue #265).

Verifies:
1. Rejection of unflagged M1 domain instance dicts/tokens in upstream mode ('domain-metamodel-typing-violation').
2. Acceptance of valid M2 metamodel types and 'meta_' prefixes.
3. Behavior of sandbox_upstream_dispatch_payload in upstream mode vs downstream mode.
4. Integration with Check 19 (check_domain_agnostic_ast_cleanliness) in scripts/verify_downstream_baseline.py.
"""

import ast
import os
import sys
import tempfile
import unittest

# Ensure project root is in sys.path
TEST_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(TEST_DIR, ".."))
if PROJECT_ROOT not in sys.path:
    sys.path.insert(0, PROJECT_ROOT)

from scripts.verify_downstream_baseline import (
    ALLOWED_M2_METAMODEL_TYPES,
    ClosedGrammarMetamodelValidator,
    check_domain_agnostic_ast_cleanliness,
    is_allowed_m2_type,
)
from scripts.dispatch_subagent import (
    sandbox_upstream_dispatch_payload,
    generate_subagent_prompt,
)
from scripts.lint_subagent_prompt import lint_prompt_text


class TestClosedGrammarMetamodelGate(unittest.TestCase):
    """Test suite for Closed-Grammar Metamodel Allowlist and Upstream Context Sandboxing."""

    def test_allowed_m2_metamodel_types_completeness(self):
        """Verify that ALLOWED_M2_METAMODEL_TYPES contains all mandated M2 metamodel types."""
        mandated_types = {
            "Component",
            "Class",
            "Port",
            "Interface",
            "Statechart",
            "Constraint",
            "Signal",
            "Event",
            "AcceptanceCriterion",
            "Scenario",
            "TraceLink",
            "ActorDefinition",
            "PartDefinition",
            "PortDefinition",
            "StateDefinition",
            "ItemDefinition",
            "HumanOperator",
            "SystemController",
            "SafetyInterlock",
            "PhysicalActuator",
            "Sensor",
            "SystemUnderStudy",
            "ExternalSystem",
            "OperatorConsole",
        }
        for m_type in mandated_types:
            self.assertIn(m_type, ALLOWED_M2_METAMODEL_TYPES)
            self.assertTrue(is_allowed_m2_type(m_type), f"Failed for M2 type: {m_type}")

    def test_is_allowed_m2_type_variations(self):
        """Verify is_allowed_m2_type handles case variations and meta_ prefixes."""
        # Exact and normalized
        self.assertTrue(is_allowed_m2_type("Component"))
        self.assertTrue(is_allowed_m2_type("component"))
        self.assertTrue(is_allowed_m2_type("part_definition"))
        self.assertTrue(is_allowed_m2_type("PartDefinition"))
        self.assertTrue(is_allowed_m2_type("human_operator"))
        self.assertTrue(is_allowed_m2_type("HumanOperator"))
        self.assertTrue(is_allowed_m2_type("system_controller"))

        # meta_ prefixes
        self.assertTrue(is_allowed_m2_type("meta_custom_element"))
        self.assertTrue(is_allowed_m2_type("MetaBlock"))
        self.assertTrue(is_allowed_m2_type("meta_entity"))

        # Invalid M1 domain tokens
        self.assertFalse(is_allowed_m2_type("FlightController"))
        self.assertFalse(is_allowed_m2_type("QuadRotor"))
        self.assertFalse(is_allowed_m2_type("BatteryPack"))
        self.assertFalse(is_allowed_m2_type("AutopilotPID"))
        self.assertFalse(is_allowed_m2_type(""))
        self.assertFalse(is_allowed_m2_type("   "))

    def test_rejection_of_unflagged_m1_domain_dicts(self):
        """Verify ClosedGrammarMetamodelValidator flags M1 domain instance dicts with domain-metamodel-typing-violation."""
        m1_code_snippets = [
            'entity_spec = {"type": "FlightController", "id": "FC-001"}\n',
            'm1_entities = {"sensor": "Altimeter", "actuator": "ElevatorServo"}\n',
            'domain_instances = {"uav_1": {"mass": 12.5}}\n',
            'node_data = {"entity_type": "QuadRotorDrone", "frequency": 100}\n',
            'concrete_models = {"vehicle": "TandemRotor"}\n',
        ]

        for snippet in m1_code_snippets:
            tree = ast.parse(snippet)
            validator = ClosedGrammarMetamodelValidator("test_module.py", PROJECT_ROOT)
            validator.visit(tree)
            self.assertTrue(
                len(validator.violations) > 0,
                f"Expected violation for M1 snippet: {snippet.strip()}",
            )
            has_rule_id = any(
                "domain-metamodel-typing-violation" in v for v in validator.violations
            )
            self.assertTrue(
                has_rule_id,
                f"Expected 'domain-metamodel-typing-violation' rule ID in violations: {validator.violations}",
            )

    def test_acceptance_of_valid_m2_metamodel_ast(self):
        """Verify ClosedGrammarMetamodelValidator accepts valid M2 metamodel types and structures."""
        m2_code_snippets = [
            'element = {"type": "Component", "name": "CoreLogic"}\n',
            'port_def = {"entity_type": "PortDefinition", "direction": "in"}\n',
            'meta_node = {"metamodel_type": "meta_custom_binding", "id": "M-1"}\n',
            'role_def = {"type": "HumanOperator", "action": "Authorize"}\n',
            'interlock = {"type": "SafetyInterlock", "threshold": 0.05}\n',
        ]

        for snippet in m2_code_snippets:
            tree = ast.parse(snippet)
            validator = ClosedGrammarMetamodelValidator("test_module.py", PROJECT_ROOT)
            validator.visit(tree)
            self.assertEqual(
                validator.violations,
                [],
                f"Expected zero violations for valid M2 snippet: {snippet.strip()}",
            )

    def test_sandbox_upstream_dispatch_payload_upstream_mode(self):
        """Verify sandbox_upstream_dispatch_payload strips customer paths and enforces M2 contract in upstream mode."""
        uav_path = "/" + "jail/uav-alpha/schema/model.sysml"
        cust_path = "/" + "jail/customer-beta/docs/features/feat-1.md"
        raw_prompt = f"""You are a context-isolated subagent operating under the DEAP Engineering Framework.

Role: Spec Writer
Subagent Type: spec_writer
Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Target: {uav_path}

Mandatory Instructions:
1. Step 1: Execute `view_file` on `skills/feature-driven-implementation/SKILL.md` as your very first step.
2. Repository Scope: You are operating within classification `UPSTREAM_SPEC_CORE_COMPILER`.
3. Micro-Task Scope: Focus on `{cust_path}`.
4. Defect Reporting: Use `gh issue create` and `glab issue create`.
PROCEED
"""
        sandboxed = sandbox_upstream_dispatch_payload(
            raw_prompt,
            repo_classification="UPSTREAM_SPEC_CORE_COMPILER",
        )

        # Assert customer jail paths are stripped
        self.assertNotIn(uav_path, sandboxed)
        self.assertNotIn(cust_path, sandboxed)
        self.assertNotIn("/" + "jail/uav-", sandboxed)
        self.assertNotIn("/" + "jail/customer-", sandboxed)

        # Assert M2 metamodel contract is present
        self.assertIn("M2 Metamodel Contract", sandboxed)
        self.assertIn("ALLOWED_M2_METAMODEL_TYPES", sandboxed)

        # Assert PROCEED is maintained
        self.assertIn("PROCEED", sandboxed)

    def test_sandbox_upstream_dispatch_payload_downstream_mode(self):
        """Verify sandbox_upstream_dispatch_payload does not strip paths or inject contracts in downstream mode."""
        uav_path = "/" + "jail/uav-alpha/schema/model.sysml"
        raw_prompt = f"""You are a context-isolated subagent operating under the DEAP Engineering Framework.

Role: Downstream Worker
Repository Classification: DOWNSTREAM_APPLICATION_WORKSPACE
Target: {uav_path}

Mandatory Instructions:
1. Step 1: Execute `view_file` on `skills/feature-driven-implementation/SKILL.md` as your very first step.
2. Repository Scope: You are operating within classification `DOWNSTREAM_APPLICATION_WORKSPACE`.
PROCEED
"""
        result = sandbox_upstream_dispatch_payload(
            raw_prompt,
            repo_classification="DOWNSTREAM_APPLICATION_WORKSPACE",
        )

        # Content should remain unchanged
        self.assertEqual(result, raw_prompt)

    def test_check19_baseline_verification_integration(self):
        """Verify Check 19 integration in verify_downstream_baseline.py."""
        # 1. Clean upstream repo passes
        check_domain_agnostic_ast_cleanliness(PROJECT_ROOT)

        # 2. Upstream workspace with M1 domain violation fails with exit code 1
        with tempfile.TemporaryDirectory() as tmpdir:
            upstream_dir = os.path.join(tmpdir, ".pipeline", "upstream")
            os.makedirs(upstream_dir, exist_ok=True)

            scripts_dir = os.path.join(tmpdir, "scripts")
            os.makedirs(scripts_dir, exist_ok=True)

            bad_file = os.path.join(scripts_dir, "bad_ast_module.py")
            with open(bad_file, "w", encoding="utf-8") as f:
                f.write('M1_ENTITIES = {"flight_controller": "FC-1"}\n')

            with self.assertRaises(SystemExit) as ctx:
                check_domain_agnostic_ast_cleanliness(tmpdir)
            self.assertEqual(ctx.exception.code, 1)

        # 3. Downstream workspace skips Check 19 cleanly
        with tempfile.TemporaryDirectory() as tmpdir:
            scripts_dir = os.path.join(tmpdir, "scripts")
            os.makedirs(scripts_dir, exist_ok=True)

            downstream_file = os.path.join(scripts_dir, "downstream_module.py")
            with open(downstream_file, "w", encoding="utf-8") as f:
                f.write('M1_ENTITIES = {"flight_controller": "FC-1"}\n')

            # Should not raise SystemExit
            check_domain_agnostic_ast_cleanliness(tmpdir)


if __name__ == "__main__":
    unittest.main()
