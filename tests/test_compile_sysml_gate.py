import unittest
import os
import sys
import json
import subprocess
import tempfile

# Assuming scripts/compile_sysml.py is in the parent directory
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.dirname(SCRIPT_DIR)
SCRIPTS_DIR = os.path.join(PROJECT_ROOT, "scripts")

if SCRIPTS_DIR not in sys.path:
    sys.path.insert(0, SCRIPTS_DIR)

import compile_sysml


class TestCompileSysmlGate(unittest.TestCase):
    """
    Test cases for enforce_pipeline0_compilation_gate in scripts/compile_sysml.py
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
        """
        Test fail-closed on non-existent schema.
        """
        result = compile_sysml.enforce_pipeline0_compilation_gate(
            schema_path="non_existent_file.sysml",
            output_path=self.output_path,
            digest_path=self.digest_path
        )
        self.assertEqual(result, 1)

    def test_fail_closed_empty_schema(self):
        """
        Test fail-closed on empty schema.
        """
        result = compile_sysml.enforce_pipeline0_compilation_gate(
            schema_path=self.empty_sysml,
            output_path=self.output_path,
            digest_path=self.digest_path
        )
        self.assertEqual(result, 1)

    def test_success_valid_schema(self):
        """
        Test success on valid SysML model (verifying output files are generated with valid SHA-256).
        """
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
        """
        Test CLI --compile via subprocess.
        """
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

if __name__ == "__main__":
    unittest.main()
