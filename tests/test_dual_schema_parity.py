#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Unit and regression tests for Issue #366: Check 31 Dual-Schema SSOT Parity Gate.

Verifies that:
1. When only schema/*.sysml exists, Check 31 passes cleanly.
2. When only .pipeline/schema.sysml exists, Check 31 passes cleanly.
3. When neither exists (clean landing zone), Check 31 passes cleanly.
4. When both exist and have identical AST definitions (part def, port def, action def, item def),
   Check 31 passes cleanly.
5. When schema/*.sysml defines a part def missing in .pipeline/schema.sysml, Check 31 fails closed (exit code 1).
6. When .pipeline/schema.sysml defines a port def missing in schema/*.sysml, Check 31 fails closed (exit code 1).
7. When action def or item def definitions mismatch, Check 31 fails closed (exit code 1).
"""

import os
import shutil
import sys
import tempfile
import unittest

repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if repo_root not in sys.path:
    sys.path.insert(0, repo_root)

scripts_dir = os.path.join(repo_root, "scripts")
if scripts_dir not in sys.path:
    sys.path.insert(0, scripts_dir)

from scripts.verify_downstream_baseline import check_dual_schema_ssot_parity


class TestDualSchemaParity(unittest.TestCase):
    def setUp(self):
        self.tmpdir = tempfile.mkdtemp()
        self.schema_dir = os.path.join(self.tmpdir, "schema")
        self.pipeline_dir = os.path.join(self.tmpdir, ".pipeline")

    def tearDown(self):
        shutil.rmtree(self.tmpdir)

    def _write_schema_sysml(self, content: str, filename: str = "model.sysml"):
        os.makedirs(self.schema_dir, exist_ok=True)
        with open(os.path.join(self.schema_dir, filename), "w", encoding="utf-8") as f:
            f.write(content)

    def _write_pipeline_sysml(self, content: str):
        os.makedirs(self.pipeline_dir, exist_ok=True)
        with open(os.path.join(self.pipeline_dir, "schema.sysml"), "w", encoding="utf-8") as f:
            f.write(content)

    def test_neither_schema_exists_passes(self):
        """Clean landing zones with no schemas pass Check 31."""
        check_dual_schema_ssot_parity(self.tmpdir)

    def test_only_schema_dir_exists_passes(self):
        """Single schema in schema/*.sysml passes Check 31."""
        self._write_schema_sysml("package Model { part def Controller; }")
        check_dual_schema_ssot_parity(self.tmpdir)

    def test_only_pipeline_schema_exists_passes(self):
        """Single schema in .pipeline/schema.sysml passes Check 31."""
        self._write_pipeline_sysml("package Model { part def Controller; }")
        check_dual_schema_ssot_parity(self.tmpdir)

    def test_identical_dual_schemas_pass(self):
        """Identical AST definitions in schema/*.sysml and .pipeline/schema.sysml pass Check 31."""
        content = """
        package AutonomousSystem {
            part def FlightController;
            part def BatterySubsystem;
            port def TelemetryPort;
            action def NavigateWaypoint;
            item def TelemetryPacket;
        }
        """
        self._write_schema_sysml(content)
        self._write_pipeline_sysml(content)
        check_dual_schema_ssot_parity(self.tmpdir)

    def test_part_def_mismatch_fails_closed(self):
        """Missing part def in .pipeline/schema.sysml triggers fail-closed exit code 1."""
        schema_content = """
        package AutonomousSystem {
            part def FlightController;
            part def ActuatorBus;
            port def CommandPort;
        }
        """
        pipeline_content = """
        package AutonomousSystem {
            part def FlightController;
            port def CommandPort;
        }
        """
        self._write_schema_sysml(schema_content)
        self._write_pipeline_sysml(pipeline_content)
        with self.assertRaises(SystemExit) as ctx:
            check_dual_schema_ssot_parity(self.tmpdir)
        self.assertEqual(ctx.exception.code, 1)

    def test_port_def_mismatch_fails_closed(self):
        """Missing port def in schema/*.sysml triggers fail-closed exit code 1."""
        schema_content = """
        package AutonomousSystem {
            part def FlightController;
            port def CommandPort;
        }
        """
        pipeline_content = """
        package AutonomousSystem {
            part def FlightController;
            port def CommandPort;
            port def SensorPort;
        }
        """
        self._write_schema_sysml(schema_content)
        self._write_pipeline_sysml(pipeline_content)
        with self.assertRaises(SystemExit) as ctx:
            check_dual_schema_ssot_parity(self.tmpdir)
        self.assertEqual(ctx.exception.code, 1)

    def test_action_def_mismatch_fails_closed(self):
        """Missing action def in .pipeline/schema.sysml triggers fail-closed exit code 1."""
        schema_content = """
        package AutonomousSystem {
            part def FlightController;
            action def ExecuteEmergencyAbort;
        }
        """
        pipeline_content = """
        package AutonomousSystem {
            part def FlightController;
        }
        """
        self._write_schema_sysml(schema_content)
        self._write_pipeline_sysml(pipeline_content)
        with self.assertRaises(SystemExit) as ctx:
            check_dual_schema_ssot_parity(self.tmpdir)
        self.assertEqual(ctx.exception.code, 1)

    def test_item_def_mismatch_fails_closed(self):
        """Missing item def in schema/*.sysml triggers fail-closed exit code 1."""
        schema_content = """
        package AutonomousSystem {
            part def FlightController;
        }
        """
        pipeline_content = """
        package AutonomousSystem {
            part def FlightController;
            item def FlightPlan;
        }
        """
        self._write_schema_sysml(schema_content)
        self._write_pipeline_sysml(pipeline_content)
        with self.assertRaises(SystemExit) as ctx:
            check_dual_schema_ssot_parity(self.tmpdir)
        self.assertEqual(ctx.exception.code, 1)


if __name__ == "__main__":
    unittest.main()
