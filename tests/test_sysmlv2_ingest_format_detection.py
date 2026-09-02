# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Unit tests for SysML v2 Universal Ingestion Engine format detection.

Realises: [SpecName/SysMLv2UniversalIngestFormatDetection]
Verifies fail-closed behavior on unsupported schema extensions (e.g. .pdf, .docx, .exe)
and ensures proper format recognition across all supported schema types.
"""

import os
import subprocess
import sys
import tempfile
import unittest

# Add skills/spec-orchestrator/scripts to sys.path
SCRIPTS_DIR = os.path.abspath(
    os.path.join(os.path.dirname(__file__), "..", "skills", "spec-orchestrator", "scripts")
)
if SCRIPTS_DIR not in sys.path:
    sys.path.insert(0, SCRIPTS_DIR)

from sysmlv2_ingest import detect_format, ingest_schema

INGEST_SCRIPT_PATH = os.path.join(SCRIPTS_DIR, "sysmlv2_ingest.py")


class TestSysMLv2IngestFormatDetection(unittest.TestCase):
    """Test suite verifying fail-closed format detection in sysmlv2_ingest.py.

    Realises: [SpecName/SysMLv2UniversalIngestFormatDetection]
    """

    def test_detect_format_unsupported_extensions_raise_value_error(self) -> None:
        """Calling detect_format with unsupported extensions must raise ValueError instead of returning 'idl'."""
        unsupported_files = [
            ("document.pdf", "%PDF-1.4 binary stream data"),
            ("notes.docx", "PK\x03\x04 archive data"),
            ("program.exe", "MZ executable binary"),
            ("archive.zip", "PK\x03\x04 compressed payload"),
            ("data.bin", "\x00\x01\x02\x03 raw binary"),
            ("readme.txt", "This is plain unstructured documentation text."),
        ]
        for filename, content in unsupported_files:
            with self.subTest(filename=filename):
                with self.assertRaises(ValueError) as ctx:
                    detect_format(filename, content)
                self.assertIn("Unsupported schema format", str(ctx.exception))
                self.assertIn(filename, str(ctx.exception))

    def test_detect_format_supported_extensions(self) -> None:
        """Calling detect_format with supported extensions must return the canonical format name."""
        cases = [
            ("model.sysml", "sysml"),
            ("MODEL.SYSML", "sysml"),
            ("interfaces.idl", "idl"),
            ("INTERFACES.IDL", "idl"),
            ("component.arxml", "autosar"),
            ("system.xml", "autosar"),
            ("telemetry.proto", "protobuf"),
            ("api.json", "openapi"),
            ("api.yaml", "openapi"),
            ("api.yml", "openapi"),
        ]
        for filename, expected_fmt in cases:
            with self.subTest(filename=filename, expected_fmt=expected_fmt):
                result = detect_format(filename, "")
                self.assertEqual(result, expected_fmt)

    def test_detect_format_content_fallback_supported(self) -> None:
        """Calling detect_format without a recognized extension but with valid schema content must detect format."""
        content_cases = [
            ("unknown_ext.txt", "package MyPackage { part def Sensor {}; }", "sysml"),
            ("unknown_ext.txt", "module Spaceflight { struct Telemetry { long id; }; };", "idl"),
            ("unknown_ext.txt", "<AUTOSAR><AR-PACKAGES></AR-PACKAGES></AUTOSAR>", "autosar"),
            ("unknown_ext.txt", 'syntax = "proto3"; message Coordinate { double lat = 1; }', "protobuf"),
            ("unknown_ext.txt", '{"openapi": "3.0.0", "paths": {}}', "openapi"),
        ]
        for filename, content, expected_fmt in content_cases:
            with self.subTest(content=content[:20], expected_fmt=expected_fmt):
                result = detect_format(filename, content)
                self.assertEqual(result, expected_fmt)

    def test_ingest_schema_unsupported_extension_raises_value_error(self) -> None:
        """ingest_schema must raise ValueError when given a file with an unsupported extension in auto mode."""
        with tempfile.TemporaryDirectory() as tmpdir:
            pdf_path = os.path.join(tmpdir, "architecture_spec.pdf")
            out_sysml = os.path.join(tmpdir, "schema.sysml")
            out_digest = os.path.join(tmpdir, "schema-digest.json")
            with open(pdf_path, "wb") as f:
                f.write(b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\n%%EOF")

            with self.assertRaises(ValueError) as ctx:
                ingest_schema(pdf_path, format_type="auto", output_path=out_sysml, digest_path=out_digest)
            self.assertIn("Unsupported schema format", str(ctx.exception))
            self.assertIn("architecture_spec.pdf", str(ctx.exception))

    def test_ingest_schema_unsupported_explicit_format_raises_value_error(self) -> None:
        """ingest_schema must raise ValueError when given an unsupported explicit format string."""
        with tempfile.TemporaryDirectory() as tmpdir:
            dummy_path = os.path.join(tmpdir, "schema.raw")
            out_sysml = os.path.join(tmpdir, "schema.sysml")
            out_digest = os.path.join(tmpdir, "schema-digest.json")
            with open(dummy_path, "w", encoding="utf-8") as f:
                f.write("arbitrary text")

            with self.assertRaises(ValueError) as ctx:
                ingest_schema(dummy_path, format_type="unsupported_custom_format", output_path=out_sysml, digest_path=out_digest)
            self.assertIn("Unsupported schema format", str(ctx.exception))

    def test_cli_unsupported_pdf_exits_nonzero(self) -> None:
        """Running the CLI with an unsupported .pdf schema must fail closed with a non-zero exit code."""
        with tempfile.TemporaryDirectory() as tmpdir:
            pdf_path = os.path.join(tmpdir, "flight_envelope.pdf")
            out_sysml = os.path.join(tmpdir, "out.sysml")
            out_digest = os.path.join(tmpdir, "out.json")
            with open(pdf_path, "wb") as f:
                f.write(b"%PDF-1.4 dummy binary PDF content")

            result = subprocess.run(
                [sys.executable, INGEST_SCRIPT_PATH, "--schema", pdf_path, "--out", out_sysml, "--digest", out_digest],
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(result.returncode, 0)
            combined_output = result.stdout + result.stderr
            self.assertIn("Unsupported schema format", combined_output)

    def test_cli_positional_unsupported_pdf_exits_nonzero(self) -> None:
        """Running the CLI with a positional unsupported .pdf schema must fail closed with non-zero exit code."""
        with tempfile.TemporaryDirectory() as tmpdir:
            pdf_path = os.path.join(tmpdir, "mission_plan.pdf")
            out_sysml = os.path.join(tmpdir, "out.sysml")
            out_digest = os.path.join(tmpdir, "out.json")
            with open(pdf_path, "wb") as f:
                f.write(b"%PDF-1.4 dummy binary PDF content")

            result = subprocess.run(
                [sys.executable, INGEST_SCRIPT_PATH, pdf_path, "--out", out_sysml, "--digest", out_digest],
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(result.returncode, 0)
            combined_output = result.stdout + result.stderr
            self.assertIn("Unsupported schema format", combined_output)


if __name__ == "__main__":
    unittest.main()
