#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Unit and integration test suite for scripts/generate_wbs_suite.py.
Validates CLI parsing, repository ingestion, 5-tier WBS hierarchy synthesis,
multi-platform CSV export (Jira, Monday.com, MS Project), JSON AST integrity,
and the Zero Unicode Em Dash Invariant (\\u2014).
"""

from __future__ import annotations

import csv
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.generate_wbs_suite import (
    CSV_HEADERS,
    SystemMetadata,
    WBSAstIngestionEngine,
    WBSSuiteSynthesizer,
    main,
    parse_args,
)

EM_DASH = chr(0x2014)


class TestWBSGenerator(unittest.TestCase):
    """Unit tests for WBS Generator CLI and synthesis engine."""

    def setUp(self) -> None:
        self.temp_dir = tempfile.mkdtemp(prefix="wbs_test_")
        self.workspace = Path(self.temp_dir)
        self.output_dir = self.workspace / "docs" / "management"

    def tearDown(self) -> None:
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)

    def _populate_sample_workspace(self) -> None:
        """Populates a realistic mock workspace with all specification tiers."""
        # 1. Config / Metadata
        schema_dir = self.workspace / "schema"
        schema_dir.mkdir(parents=True, exist_ok=True)
        domain_cfg = {
            "system_identifier": "UAS-PHOENIX-01",
            "system_name": "Phoenix Autonomous Surveillance UAS",
            "mtow_kg": 45.0,
            "do178c_level": "DAL-B",
            "sora_sail": "SAIL III",
        }
        (schema_dir / "domain_config.json").write_text(json.dumps(domain_cfg), encoding="utf-8")
        (schema_dir / "model.sysml").write_text("// SysML SSOT", encoding="utf-8")

        # 2. ConOps & Mission Intent
        conops_dir = self.workspace / "docs" / "conops"
        conops_dir.mkdir(parents=True, exist_ok=True)
        (conops_dir / "CONOPS.md").write_text(
            "# Concept of Operations (ConOps): Phoenix Autonomous Surveillance UAS\n\n"
            "## 1. Scope\n- **System Identifier:** `UAS-PHOENIX-01`\n",
            encoding="utf-8",
        )
        (conops_dir / "MISSION_INTENT.md").write_text(
            "# Tactical Mission Intent & Execution Plan\n",
            encoding="utf-8",
        )

        # 3. Safety STPA
        safety_dir = self.workspace / "docs" / "safety"
        safety_dir.mkdir(parents=True, exist_ok=True)
        (safety_dir / "STPA_MATRIX.md").write_text(
            "# Level 1B Safety Matrix\n\n"
            "SAIL III classification and **SC-01**, **H-1** safety invariants.\n",
            encoding="utf-8",
        )

        # 4. ICD Interfaces
        icd_dir = self.workspace / "docs" / "interfaces"
        icd_dir.mkdir(parents=True, exist_ok=True)
        (icd_dir / "ICD_01_SYSTEM_INTERFACE_MATRIX.md").write_text("# Level 1C Interface Matrix\n", encoding="utf-8")
        (icd_dir / "ICD_02_MASTER_SIGNAL_DICTIONARY.md").write_text("# Level 1C Signal Dictionary\n", encoding="utf-8")

        # 5. Epics
        epics_dir = self.workspace / "docs" / "epics"
        epics_dir.mkdir(parents=True, exist_ok=True)
        (epics_dir / "epic-01-navigation.md").write_text(
            "| Attribute | Detail |\n| :--- | :--- |\n| **Issue ID** | #101 |\n\n"
            "# Epic: Navigation & Guidance Subsystem\n\n"
            "**Subsystem:** Navigation & Guidance\n"
            "**DAL:** DAL-B\n",
            encoding="utf-8",
        )
        (epics_dir / "epic-02-flight-control.md").write_text(
            "| Attribute | Detail |\n| :--- | :--- |\n| **Issue ID** | #102 |\n\n"
            "# Epic: Flight Control Subsystem\n\n"
            "**Subsystem:** Flight Control\n"
            "**DAL:** DAL-B\n",
            encoding="utf-8",
        )

        # 6. Features
        feat_dir = self.workspace / "docs" / "features"
        feat_dir.mkdir(parents=True, exist_ok=True)
        (feat_dir / "feat-01-state-estimation.md").write_text(
            "| Attribute | Detail |\n| :--- | :--- |\n| **Feature ID** | FEAT-01 |\n\n"
            "# Feature: Inertial State Estimation & Sensor Fusion\n\n"
            "**Subsystem:** Navigation & Guidance\n"
            "**Epic:** EPIC-01\n"
            "**DAL:** DAL-B\n"
            "SysML Anchor: `SysSSOT::NavSubsys::StateEstimator`\n"
            "Safety constraints: **SC-01**, **H-1**\n\n"
            "- Given valid sensor stream, When IMU sample arrives, Then update attitude vector.\n",
            encoding="utf-8",
        )
        (feat_dir / "feat-02-attitude-control.md").write_text(
            "| Attribute | Detail |\n| :--- | :--- |\n| **Feature ID** | FEAT-02 |\n\n"
            "# Feature: Closed-Loop Inner Attitude Control\n\n"
            "**Subsystem:** Flight Control\n"
            "**Epic:** EPIC-02\n"
            "**DAL:** DAL-B\n"
            "SysML Anchor: `SysSSOT::FlightControl::AttitudeController`\n"
            "Safety constraints: **SC-02**, **H-2**\n\n"
            "- Given commanded roll pitch yaw, When rate loop triggers at 250 Hz, Then emit actuator PWM.\n",
            encoding="utf-8",
        )

        # 7. User Stories & Use Cases
        us_dir = self.workspace / "docs" / "user-stories"
        us_dir.mkdir(parents=True, exist_ok=True)
        (us_dir / "us-01.md").write_text(
            "# User Story: Sensor Ingestion\nRealizes: FEAT-01\n",
            encoding="utf-8",
        )
        (us_dir / "us-02.md").write_text(
            "# User Story: Rate Loop Control\nRealizes: FEAT-02\n",
            encoding="utf-8",
        )

        uc_dir = self.workspace / "docs" / "use-cases"
        uc_dir.mkdir(parents=True, exist_ok=True)
        (uc_dir / "uc-01.md").write_text(
            "# Use Case: Execute Autonomous Flight\nRealizes: FEAT-01, FEAT-02\n",
            encoding="utf-8",
        )

        # 8. MBD Deliverable Files
        (self.workspace / "models" / "matlab").mkdir(parents=True, exist_ok=True)
        (self.workspace / "models" / "scripts").mkdir(parents=True, exist_ok=True)
        (self.workspace / "models" / "python").mkdir(parents=True, exist_ok=True)
        (self.workspace / "tests").mkdir(parents=True, exist_ok=True)
        (self.workspace / "docs" / "reports" / "simulink_results").mkdir(parents=True, exist_ok=True)

        (self.workspace / "models" / "matlab" / "feat_01_params.m").write_text("% MATLAB params\n", encoding="utf-8")
        (self.workspace / "models" / "scripts" / "build_feat_01_model.m").write_text("% SL Builder\n", encoding="utf-8")
        (self.workspace / "models" / "python" / "feat_01_domain.py").write_text("# Domain model\n", encoding="utf-8")
        (self.workspace / "models" / "python" / "feat_01_engine.py").write_text("# 250 Hz Engine\n", encoding="utf-8")
        (self.workspace / "tests" / "test_feat_01_simulation.py").write_text("# Pytest harness\n", encoding="utf-8")
        (self.workspace / "docs" / "reports" / "simulink_results" / "FEAT-01_results.md").write_text("# Report\n", encoding="utf-8")

    def test_parse_args_defaults(self) -> None:
        """Assert default CLI argument values."""
        args = parse_args([])
        self.assertEqual(args.workspace, ".")
        self.assertIsNone(args.output_dir)

    def test_parse_args_custom(self) -> None:
        """Assert custom CLI argument values."""
        args = parse_args(["--workspace", "/tmp/ws", "--output-dir", "/tmp/out"])
        self.assertEqual(args.workspace, "/tmp/ws")
        self.assertEqual(args.output_dir, "/tmp/out")

    def test_main_cli_execution(self) -> None:
        """Test full main() invocation on sample workspace."""
        self._populate_sample_workspace()
        exit_code = main(["--workspace", str(self.workspace), "--output-dir", str(self.output_dir)])
        self.assertEqual(exit_code, 0)

        # Assert all 3 deliverables are created
        md_file = self.output_dir / "WBS_DELIVERABLES_SUITE.md"
        csv_file = self.output_dir / "wbs_export_jira_monday_ms_project.csv"
        json_file = self.output_dir / "wbs_export.json"

        self.assertTrue(md_file.is_file(), "WBS_DELIVERABLES_SUITE.md missing")
        self.assertTrue(csv_file.is_file(), "wbs_export_jira_monday_ms_project.csv missing")
        self.assertTrue(json_file.is_file(), "wbs_export.json missing")

    def test_markdown_suite_structure_and_metadata(self) -> None:
        """Verify markdown document structure, metadata table, and sections."""
        self._populate_sample_workspace()
        engine = WBSAstIngestionEngine(workspace_path=self.workspace, output_dir=self.output_dir)
        engine.run_ingestion()
        synthesizer = WBSSuiteSynthesizer(engine)
        md_path, _, _ = synthesizer.synthesize_all()

        content = md_path.read_text(encoding="utf-8")
        lines = content.splitlines()

        # Metadata table in lines 1-10
        self.assertTrue(lines[0].startswith("| Attribute | Specification Detail |"))
        self.assertTrue(lines[1].startswith("| :--- | :--- |"))
        self.assertIn("**Title**", "\n".join(lines[:10]))
        self.assertIn("Level 4 Enterprise Realization", "\n".join(lines[:10]))

        # Check required sections
        self.assertIn("## 1. Executive Summary & Program Metrics Table", content)
        self.assertIn("## 2. System Architecture, ConOps & Safety Baseline Deliverables Table", content)
        self.assertIn("## 3. Subsystem Epics & Feature Realization Matrices", content)
        self.assertIn("### End-to-End 7-Column Traceability Matrix", content)
        self.assertIn("## 4. Master Verification & Test Execution Summary Table", content)
        self.assertIn("## 5. Multi-Platform Project Management Export & Import Guide", content)
        self.assertIn("## 6. Source References", content)

        # Traceability headers
        self.assertIn(
            "| SysML Component | Feature Spec | User Stories | MATLAB / Simulink Plant | Python 250 Hz Engine | Verification Suite | Simulation Evidence |",
            content,
        )

        # Mathematical block
        self.assertIn("\\mathbf{x}_{k+1}", content)
        self.assertIn("\\epsilon_{\\mathrm{equiv}}", content)

    def test_csv_export_format_and_columns(self) -> None:
        """Verify CSV columns, quoting, delimiter, and row count."""
        self._populate_sample_workspace()
        engine = WBSAstIngestionEngine(workspace_path=self.workspace, output_dir=self.output_dir)
        engine.run_ingestion()
        synthesizer = WBSSuiteSynthesizer(engine)
        _, csv_path, _ = synthesizer.synthesize_all()

        with open(csv_path, "r", encoding="utf-8") as f:
            reader = csv.reader(f)
            rows = list(reader)

        # Header assertion
        self.assertEqual(rows[0], CSV_HEADERS)

        # Check System root row
        system_row = rows[1]
        self.assertEqual(system_row[0], "1.0")
        self.assertEqual(system_row[1], "UAS-PHOENIX-01")
        self.assertEqual(system_row[2], "System")
        self.assertEqual(system_row[3], "Phoenix Autonomous Surveillance UAS")

        # Find Epics and Features
        item_types = [r[2] for r in rows]
        self.assertIn("Baseline Deliverable", item_types)
        self.assertIn("Epic", item_types)
        self.assertIn("Feature", item_types)
        self.assertIn("Work Package", item_types)

        # Assert 7 concrete work packages exist per feature
        wp_rows = [r for r in rows if r[2] == "Work Package"]
        self.assertEqual(len(wp_rows), 14)  # 2 features * 7 WPs

    def test_json_ast_schema_and_integrity(self) -> None:
        """Verify JSON AST structure against formal requirements."""
        self._populate_sample_workspace()
        engine = WBSAstIngestionEngine(workspace_path=self.workspace, output_dir=self.output_dir)
        engine.run_ingestion()
        synthesizer = WBSSuiteSynthesizer(engine)
        _, _, json_path = synthesizer.synthesize_all()

        with open(json_path, "r", encoding="utf-8") as f:
            data = json.load(f)

        self.assertIn("metadata", data)
        self.assertIn("wbs_tree", data)
        self.assertIn("traceability_matrix", data)
        self.assertIn("baseline_deliverables", data)

        meta = data["metadata"]
        self.assertEqual(meta["system_id"], "UAS-PHOENIX-01")
        self.assertEqual(meta["standard"], "MIL-STD-881E")
        self.assertEqual(meta["total_epics"], 2)
        self.assertEqual(meta["total_features"], 2)
        self.assertEqual(meta["total_work_packages"], 14)

        tree = data["wbs_tree"]
        self.assertEqual(tree["wbs_code"], "1.0")
        self.assertEqual(tree["level"], 1)
        self.assertEqual(len(tree["children"]), 10)  # 8 baseline + 2 epics

        trace = data["traceability_matrix"]
        self.assertEqual(len(trace), 2)
        self.assertEqual(trace[0]["sysml_component"], "SysSSOT::NavSubsys::StateEstimator")
        self.assertIn("models/python/feat_01_engine.py", trace[0]["python_250hz_engine"])

    def test_empty_workspace_graceful_handling(self) -> None:
        """Verify generator handles empty / sparse workspace gracefully without crashing."""
        # Workspace has empty directories with .gitkeep
        for sub in ("docs/epics", "docs/features", "docs/use-cases", "docs/user-stories", "schema"):
            p = self.workspace / sub
            p.mkdir(parents=True, exist_ok=True)
            (p / ".gitkeep").write_text("", encoding="utf-8")

        exit_code = main(["--workspace", str(self.workspace), "--output-dir", str(self.output_dir)])
        self.assertEqual(exit_code, 0)

        md_file = self.output_dir / "WBS_DELIVERABLES_SUITE.md"
        csv_file = self.output_dir / "wbs_export_jira_monday_ms_project.csv"
        json_file = self.output_dir / "wbs_export.json"

        self.assertTrue(md_file.is_file())
        self.assertTrue(csv_file.is_file())
        self.assertTrue(json_file.is_file())

    def test_zero_unicode_emdash_invariant(self) -> None:
        """Assert zero occurrences of Unicode em dash (\\u2014) in source files and generated outputs."""
        self._populate_sample_workspace()
        main(["--workspace", str(self.workspace), "--output-dir", str(self.output_dir)])

        # Check script source
        script_path = REPO_ROOT / "scripts" / "generate_wbs_suite.py"
        self.assertNotIn(EM_DASH, script_path.read_text(encoding="utf-8"), "Em dash in generate_wbs_suite.py")

        # Check test source
        test_path = REPO_ROOT / "tests" / "test_wbs_generator.py"
        self.assertNotIn(EM_DASH, test_path.read_text(encoding="utf-8"), "Em dash in test_wbs_generator.py")

        # Check generated outputs
        for fname in ("WBS_DELIVERABLES_SUITE.md", "wbs_export_jira_monday_ms_project.csv", "wbs_export.json"):
            gen_file = self.output_dir / fname
            self.assertTrue(gen_file.is_file())
            content = gen_file.read_text(encoding="utf-8")
            self.assertNotIn(EM_DASH, content, f"Em dash found in generated file {fname}")


if __name__ == "__main__":
    unittest.main()
