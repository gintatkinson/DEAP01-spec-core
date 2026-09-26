import unittest
import os
import sys
import json
import subprocess
import tempfile
import shutil

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.dirname(SCRIPT_DIR)
SCRIPTS_DIR = os.path.join(PROJECT_ROOT, "scripts")

if SCRIPTS_DIR not in sys.path:
    sys.path.insert(0, SCRIPTS_DIR)

import compile_sysml
from compile_sysml import (
    enforce_pipeline0_compilation_gate,
    forward_sync_sysml_to_specs,
    reverse_sync_specs_to_sysml,
    is_phase0_verified,
    extract_epics_from_markdown,
    SysMLParser,
    SysMLCapabilityDef,
)


class TestCompileSysmlGate(unittest.TestCase):
    """
    Test cases for enforce_pipeline0_compilation_gate, forward_sync_sysml_to_specs Phase Gate Guard,
    and SysMLCapabilityDef compilation to Epics in scripts/compile_sysml.py (Issue #360).
    """

    def setUp(self):
        """Set up temporary directory and files for testing."""
        self.temp_dir = tempfile.TemporaryDirectory()
        self.output_path = os.path.join(self.temp_dir.name, "schema.sysml")
        self.digest_path = os.path.join(self.temp_dir.name, "schema-digest.json")
        self.mock_sysml = os.path.join(self.temp_dir.name, "mock.sysml")
        self.empty_sysml = os.path.join(self.temp_dir.name, "empty.sysml")
        
        with open(self.mock_sysml, "w", encoding="utf-8") as f:
            f.write("package MockPackage { part def MockPart; }")
            
        with open(self.empty_sysml, "w", encoding="utf-8") as f:
            f.write("")

    def tearDown(self):
        """Clean up temporary resources."""
        self.temp_dir.cleanup()

    def test_fail_closed_non_existent_schema(self):
        """Test fail-closed on non-existent schema."""
        result = compile_sysml.enforce_pipeline0_compilation_gate(
            schema_path="non_existent_file.sysml",
            output_path=self.output_path,
            digest_path=self.digest_path
        )
        self.assertEqual(result, 1)

    def test_fail_closed_empty_schema(self):
        """Test fail-closed on empty schema."""
        result = compile_sysml.enforce_pipeline0_compilation_gate(
            schema_path=self.empty_sysml,
            output_path=self.output_path,
            digest_path=self.digest_path
        )
        self.assertEqual(result, 1)

    def test_success_valid_schema(self):
        """Test success on valid SysML model (verifying output files are generated with valid SHA-256)."""
        result = compile_sysml.enforce_pipeline0_compilation_gate(
            schema_path=self.mock_sysml,
            output_path=self.output_path,
            digest_path=self.digest_path
        )
        self.assertEqual(result, 0)
        self.assertTrue(os.path.exists(self.output_path))
        self.assertTrue(os.path.exists(self.digest_path))
        
        with open(self.digest_path, "r", encoding="utf-8") as f:
            digest = json.load(f)
            self.assertIn("sha256", digest)
            self.assertIn("node_counts", digest)

    def test_cli_compile_flag(self):
        """Test CLI --compile via subprocess."""
        script_path = os.path.join(SCRIPTS_DIR, "compile_sysml.py")
        cmd = [
            sys.executable, script_path,
            "--compile",
            "--schema", self.mock_sysml,
            "--out", self.output_path,
            "--digest", self.digest_path
        ]
        result = subprocess.run(cmd, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0)
        self.assertTrue(os.path.exists(self.output_path))
        self.assertTrue(os.path.exists(self.digest_path))
        
        # Test missing file
        cmd_missing = [
            sys.executable, script_path,
            "--compile",
            "--schema", "missing_file.sysml",
            "--out", self.output_path,
            "--digest", self.digest_path
        ]
        result_missing = subprocess.run(cmd_missing, capture_output=True, text=True)
        self.assertEqual(result_missing.returncode, 1)

    def test_phase_gate_guard_blocks_unverified_phase0(self):
        """
        Verify forward_sync_sysml_to_specs rejects writing to downstream landing zones
        when Phase 0 has not been compiled/verified and force=False.
        """
        workspace = os.path.join(self.temp_dir.name, "downstream_ws")
        os.makedirs(workspace, exist_ok=True)
        schema_file = os.path.join(workspace, "model.sysml")
        with open(schema_file, "w", encoding="utf-8") as f:
            f.write("package TestPkg { part def CoreController; }")

        docs_target = os.path.join(workspace, "docs")
        os.makedirs(docs_target, exist_ok=True)

        with self.assertRaises(RuntimeError) as ctx:
            forward_sync_sysml_to_specs(
                schema_path=schema_file,
                docs_dir=docs_target,
                force=False,
                project_root=workspace,
            )
        self.assertIn("Phase Gate Guard", str(ctx.exception))
        self.assertIn("Phase 0 compilation is not verified", str(ctx.exception))

    def test_phase_gate_guard_allows_with_force(self):
        """Verify forward_sync_sysml_to_specs succeeds when force=True even if Phase 0 is unverified."""
        workspace = os.path.join(self.temp_dir.name, "downstream_ws_force")
        os.makedirs(workspace, exist_ok=True)
        schema_file = os.path.join(workspace, "model.sysml")
        with open(schema_file, "w", encoding="utf-8") as f:
            f.write("package TestPkg { part def CoreController; }")

        docs_target = os.path.join(workspace, "docs")
        os.makedirs(docs_target, exist_ok=True)

        generated = forward_sync_sysml_to_specs(
            schema_path=schema_file,
            docs_dir=docs_target,
            force=True,
            project_root=workspace,
        )
        self.assertTrue(len(generated) > 0)
        self.assertTrue(os.path.exists(os.path.join(docs_target, "features")))

    def test_phase_gate_guard_allows_when_phase0_verified(self):
        """Verify forward_sync_sysml_to_specs succeeds without force when Phase 0 is properly verified."""
        workspace = os.path.join(self.temp_dir.name, "downstream_ws_verified")
        pipeline_dir = os.path.join(workspace, ".pipeline")
        os.makedirs(pipeline_dir, exist_ok=True)
        schema_file = os.path.join(workspace, "model.sysml")
        with open(schema_file, "w", encoding="utf-8") as f:
            f.write("package TestPkg { part def CoreController; }")

        out_sysml = os.path.join(pipeline_dir, "schema.sysml")
        out_digest = os.path.join(pipeline_dir, "schema-digest.json")

        gate_res = enforce_pipeline0_compilation_gate(
            schema_path=schema_file,
            output_path=out_sysml,
            digest_path=out_digest,
        )
        self.assertEqual(gate_res, 0)
        self.assertTrue(is_phase0_verified(project_root=workspace))

        docs_target = os.path.join(workspace, "docs")
        os.makedirs(docs_target, exist_ok=True)

        generated = forward_sync_sysml_to_specs(
            schema_path=out_sysml,
            docs_dir=docs_target,
            force=False,
            project_root=workspace,
        )
        self.assertTrue(len(generated) > 0)
        self.assertTrue(os.path.exists(os.path.join(docs_target, "epics")))

    def test_sysml_capability_def_compiles_to_epics(self):
        """
        Verify formal SysMLCapabilityDef parsing and forward compilation into Epics (Issue #360),
        and reverse sync back to SysMLCapabilityDef.
        """
        workspace = os.path.join(self.temp_dir.name, "epic_comp_ws")
        docs_target = os.path.join(workspace, "docs")
        os.makedirs(docs_target, exist_ok=True)

        schema_content = """package NavigationSystem {
    doc /* Navigation & Trajectory Management */

    part def FlightComputer;

    capability def TrajectoryGuidance {
        doc /* Autonomous Waypoint Navigation & Path Following */
        subsystem FlightComputer;
    }

    capability def ObstacleAvoidance {
        doc /* Real-Time Reactive Obstacle Avoidance */
        subsystem FlightComputer;
    }
}
"""
        schema_file = os.path.join(workspace, "model.sysml")
        with open(schema_file, "w", encoding="utf-8") as f:
            f.write(schema_content)

        generated = forward_sync_sysml_to_specs(
            schema_path=schema_file,
            docs_dir=docs_target,
            force=True,
            project_root=workspace,
        )

        epic_keys = [k for k in generated.keys() if k.startswith("epics")]
        self.assertGreaterEqual(len(epic_keys), 2)

        traj_key = [k for k in epic_keys if "TrajectoryGuidance" in k][0]
        traj_content = generated[traj_key]

        # Verify Frontmatter
        self.assertIn('type: epic', traj_content)
        self.assertIn('generation_mode: subagent', traj_content)
        self.assertIn('subsystem: "FlightComputer"', traj_content)

        # Verify Capability table
        self.assertIn('| **TrajectoryGuidance** | FlightComputer |', traj_content)

        # Verify Architectural Context Mermaid
        self.assertIn('class FlightComputer {', traj_content)
        self.assertIn('+performTrajectoryGuidance()', traj_content)

        # Verify Reverse Sync roundtrip extracts SysMLCapabilityDef
        extracted_caps = extract_epics_from_markdown(traj_content, traj_key)
        self.assertEqual(len(extracted_caps), 1)
        cap = extracted_caps[0]
        self.assertEqual(getattr(cap, "name", ""), "TrajectoryGuidance")
        self.assertEqual(getattr(cap, "subsystem", ""), "FlightComputer")

    def test_cli_forward_sync_force_flag(self):
        """Test CLI --forward-sync with --force flag."""
        script_path = os.path.join(SCRIPTS_DIR, "compile_sysml.py")
        workspace = os.path.join(self.temp_dir.name, "cli_force_ws")
        docs_target = os.path.join(workspace, "docs")
        os.makedirs(docs_target, exist_ok=True)
        schema_file = os.path.join(workspace, "model.sysml")
        with open(schema_file, "w", encoding="utf-8") as f:
            f.write("package CliPkg { part def WorkerNode; }")

        cmd = [
            sys.executable, script_path,
            "--forward-sync",
            "--force",
            "--schema", schema_file,
            "--docs", docs_target,
        ]
        result = subprocess.run(cmd, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0)
        self.assertTrue(os.path.exists(os.path.join(docs_target, "epics")))
        self.assertTrue(os.path.exists(os.path.join(docs_target, "features")))


if __name__ == "__main__":
    unittest.main()
