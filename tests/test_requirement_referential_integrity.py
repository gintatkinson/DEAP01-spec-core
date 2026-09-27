import unittest
import os
import sys
import tempfile
import io
from contextlib import redirect_stderr

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.dirname(SCRIPT_DIR)
SCRIPTS_DIR = os.path.join(PROJECT_ROOT, "scripts")

if SCRIPTS_DIR not in sys.path:
    sys.path.insert(0, SCRIPTS_DIR)

from compile_sysml import (
    enforce_pipeline0_compilation_gate,
    parse_sysml,
)


class TestRequirementReferentialIntegrity(unittest.TestCase):
    """
    Unit tests verifying referential integrity validation of requirement annotations
    (@SafetyRealises and @TriggersHazard) in scripts/compile_sysml.py (refs #391).
    """

    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.output_path = os.path.join(self.temp_dir.name, "schema.sysml")
        self.digest_path = os.path.join(self.temp_dir.name, "schema-digest.json")

    def tearDown(self):
        self.temp_dir.cleanup()

    def _write_schema(self, content: str) -> str:
        schema_path = os.path.join(self.temp_dir.name, "model.sysml")
        with open(schema_path, "w", encoding="utf-8") as f:
            f.write(content)
        return schema_path

    def test_valid_requirement_annotations_pass_gate(self):
        """Valid requirement annotations matching defined constraints and hazards pass compilation gate (exit code 0)."""
        content = """package ValidSafetyModel {
    constraint def UCA_10;
    hazard def H_01;

    requirement def SC_1 {
        @SafetyRealises("UCA-10")
        @TriggersHazard("H-01")
        doc /* Compliant requirement realization */
    }
}
"""
        schema_path = self._write_schema(content)
        result = enforce_pipeline0_compilation_gate(
            schema_path=schema_path,
            output_path=self.output_path,
            digest_path=self.digest_path,
        )
        self.assertEqual(result, 0)
        self.assertTrue(os.path.exists(self.output_path))
        self.assertTrue(os.path.exists(self.digest_path))

    def test_dangling_safety_realises_fails_gate(self):
        """Dangling @SafetyRealises reference fails compilation gate (exit code 1)."""
        content = """package DanglingUcaModel {
    hazard def H_01;

    requirement def SC_1 {
        @SafetyRealises("UCA_999")
        @TriggersHazard("H_01")
        doc /* Broken UCA reference */
    }
}
"""
        schema_path = self._write_schema(content)
        stderr_buf = io.StringIO()
        with redirect_stderr(stderr_buf):
            result = enforce_pipeline0_compilation_gate(
                schema_path=schema_path,
                output_path=self.output_path,
                digest_path=self.digest_path,
            )
        self.assertEqual(result, 1)
        err_msg = stderr_buf.getvalue()
        self.assertIn("Referential integrity violation", err_msg)
        self.assertIn("references undefined UCA 'UCA_999'", err_msg)

    def test_dangling_triggers_hazard_fails_gate(self):
        """Dangling @TriggersHazard reference fails compilation gate (exit code 1)."""
        content = """package DanglingHazardModel {
    constraint def UCA_01;

    requirement def SC_1 {
        @SafetyRealises("UCA_01")
        @TriggersHazard("HAZ_999")
        doc /* Broken Hazard reference */
    }
}
"""
        schema_path = self._write_schema(content)
        stderr_buf = io.StringIO()
        with redirect_stderr(stderr_buf):
            result = enforce_pipeline0_compilation_gate(
                schema_path=schema_path,
                output_path=self.output_path,
                digest_path=self.digest_path,
            )
        self.assertEqual(result, 1)
        err_msg = stderr_buf.getvalue()
        self.assertIn("Referential integrity violation", err_msg)
        self.assertIn("references undefined Hazard 'HAZ_999'", err_msg)

    def test_parse_sysml_extracts_requirement_annotations(self):
        """parse_sysml extracts requirement annotations into ast['requirement_annotations']."""
        content = """package TestModel {
    constraint def UCA_01;
    hazard def HAZ_01;

    requirement def SC_01 {
        @SafetyRealises("UCA_01")
        @TriggersHazard("HAZ_01")
    }

    requirement def SC_02 {
        @SafetyRealises(requirement="UCA_02")
        @TriggersHazard(id="HAZ_02", severity="Catastrophic")
    }
}
"""
        ast = parse_sysml(content)
        self.assertIn("requirement_annotations", ast)
        self.assertIn("hazard_defs", ast)
        self.assertIn("SC_01", ast["requirement_annotations"])
        self.assertEqual(ast["requirement_annotations"]["SC_01"]["safety_realises"], ["UCA_01"])
        self.assertEqual(ast["requirement_annotations"]["SC_01"]["triggers_hazard"], ["HAZ_01"])
        self.assertIn("SC_02", ast["requirement_annotations"])
        self.assertEqual(ast["requirement_annotations"]["SC_02"]["safety_realises"], ["UCA_02"])
        self.assertEqual(ast["requirement_annotations"]["SC_02"]["triggers_hazard"], ["HAZ_02"])

    def test_hyphen_underscore_variation_matching(self):
        """Hyphen and underscore variations resolve consistently."""
        content = """package VariationModel {
    constraint def UCA_10_CONTROLLER;
    hazard def H_15_LOSS;

    requirement def SC_10 {
        @SafetyRealises("UCA-10-CONTROLLER")
        @TriggersHazard("H-15-LOSS")
    }
}
"""
        schema_path = self._write_schema(content)
        result = enforce_pipeline0_compilation_gate(
            schema_path=schema_path,
            output_path=self.output_path,
            digest_path=self.digest_path,
        )
        self.assertEqual(result, 0)


if __name__ == "__main__":
    unittest.main()
