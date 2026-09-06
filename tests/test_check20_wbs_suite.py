#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Unit test suite for Check 20 (WBS & Enterprise Deliverables Suite Validation)
in scripts/verify_downstream_baseline.py.
"""

import json
import os
import re
import sys
import tempfile
import unittest
from pathlib import Path

repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if repo_root not in sys.path:
    sys.path.insert(0, repo_root)

from scripts.generate_wbs_suite import (
    WBSAstIngestionEngine,
    WBSSuiteSynthesizer,
)
from scripts.verify_downstream_baseline import (
    check_wbs_suite_integrity,
    _check_wbs_suite_integrity,
    run_all_checks,
)


def _create_valid_wbs_suite(tmpdir: str) -> None:
    mgmt_dir = os.path.join(tmpdir, "docs", "management")
    os.makedirs(mgmt_dir, exist_ok=True)

    md_path = os.path.join(mgmt_dir, "WBS_DELIVERABLES_SUITE.md")
    csv_path = os.path.join(mgmt_dir, "wbs_export_jira_monday_ms_project.csv")
    json_path = os.path.join(mgmt_dir, "wbs_export.json")

    md_content = """| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #100 |
| **Title** | Work Breakdown Structure & Enterprise Realization Suite |
| **Type** | management |
| **Management Level** | Level 4 Enterprise Realization |
| **Standard Baseline** | MIL-STD-881E / INCOSE SEH v5.0 |
| **Generation Mode** | subagent |
| **Specification Source** | `schema/model.sysml` |

# Level 4: Work Breakdown Structure & Enterprise Realization Suite

## 1. Executive Summary & Program Baseline
Overview of the program.

## 2. Baseline Deliverables Table
| Deliverable ID | WBS Code | Specification Title | Standard / Framework | Target Artifact Path | Verification Gate | Status |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `SPEC-CONOPS` | `1.0.1` | Level 1B Concept of Operations | ISO 29148 | docs/conops/CONOPS.md | Gate 1 | Verified |

## 3. Subsystem Epics & Feature Realization Matrices
### WBS 1.1: [EPIC-01] Navigation Subsystem

### End-to-End 7-Column Traceability Matrix
| SysML Component | Feature Spec | User Stories | MATLAB / Simulink Plant | Python 250 Hz Engine | Verification Suite | Simulation Evidence |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `SysSSOT::Nav` | [FEAT-01](docs/features/feat-01.md) | [US-01](docs/user-stories/us-01.md) | models/scripts/build_nav_model.m | models/python/nav_engine.py | tests/test_nav.py | [Report](docs/reports/simulink_results/FEAT-01_results.md) |

## 4. Master Verification & Test Execution Summary Table
| Feature ID / WBS | Pytest Verification Suite Path | Test Coverage Types | Execution Rate | Equivalence Tol | Verification Gate Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `FEAT-01 (1.1.1)` | tests/test_nav.py | Nominal, Safety Invariant | 250 Hz (dt = 0.004 s) | tol <= 1e-6 | Passing CI Gate |

## 5. Multi-Platform Project Management Export & Import Guide
Step-by-step import guide for Jira, Monday.com, and MS Project.

## 6. Source References
- MIL-STD-881E
"""
    with open(md_path, "w", encoding="utf-8") as f:
        f.write(md_content)

    csv_content = (
        '"WBS Code","ID","Item Type","Name","Parent ID","Subsystem","DO-178C Level","Artifact Path","Est. Hours","Verification Gate","Status","Description"\n'
        '"1.0","SYS-01","System","Digital Engineering Platform","","Integrated System","DAL-B","docs/management/WBS_DELIVERABLES_SUITE.md","1200","Milestone Verification Gate","In Progress","Level 1 Integrated System Root per MIL-STD-881E."\n'
    )
    with open(csv_path, "w", encoding="utf-8") as f:
        f.write(csv_content)

    json_ast = {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "WBS_Enterprise_Realization_AST",
        "type": "object",
        "metadata": {
            "program_title": "Digital Engineering Platform",
            "system_id": "SYS-01",
            "standard": "MIL-STD-881E",
            "generated_at": "2026-09-05T00:00:00Z",
            "total_work_packages": 7,
        },
        "wbs_tree": {
            "wbs_code": "1.0",
            "name": "Digital Engineering Platform",
            "level": 1,
            "children": [],
        },
        "traceability_matrix": [],
    }
    with open(json_path, "w", encoding="utf-8") as f:
        json.dump(json_ast, f, indent=2)


class TestCheck20WBSSuiteIntegrity(unittest.TestCase):
    """Test suite validating Check 20 behavior and edge case handling."""

    def test_clean_repo_pending_passes(self):
        """When docs/management/WBS_DELIVERABLES_SUITE.md does not exist, Check 20 passes."""
        with tempfile.TemporaryDirectory() as tmpdir:
            # Should return cleanly without sys.exit
            check_wbs_suite_integrity(tmpdir)

    def test_valid_suite_passes(self):
        """When valid WBS suite exists, Check 20 passes."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _create_valid_wbs_suite(tmpdir)
            check_wbs_suite_integrity(tmpdir)

    def test_missing_csv_export_fails(self):
        """When CSV export is missing, Check 20 raises SystemExit."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _create_valid_wbs_suite(tmpdir)
            csv_path = os.path.join(tmpdir, "docs", "management", "wbs_export_jira_monday_ms_project.csv")
            os.remove(csv_path)

            with self.assertRaises(SystemExit) as cm:
                check_wbs_suite_integrity(tmpdir)
            self.assertEqual(cm.exception.code, 1)

    def test_missing_json_ast_fails(self):
        """When JSON AST export is missing, Check 20 raises SystemExit."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _create_valid_wbs_suite(tmpdir)
            json_path = os.path.join(tmpdir, "docs", "management", "wbs_export.json")
            os.remove(json_path)

            with self.assertRaises(SystemExit) as cm:
                check_wbs_suite_integrity(tmpdir)
            self.assertEqual(cm.exception.code, 1)

    def test_missing_section_header_fails(self):
        """When a required section header is missing from WBS markdown, Check 20 raises SystemExit."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _create_valid_wbs_suite(tmpdir)
            md_path = os.path.join(tmpdir, "docs", "management", "WBS_DELIVERABLES_SUITE.md")
            with open(md_path, "r", encoding="utf-8") as f:
                content = f.read()

            # Remove Executive Summary header
            corrupted = content.replace("## 1. Executive Summary", "## 1. Introduction")
            with open(md_path, "w", encoding="utf-8") as f:
                f.write(corrupted)

            with self.assertRaises(SystemExit) as cm:
                check_wbs_suite_integrity(tmpdir)
            self.assertEqual(cm.exception.code, 1)

    def test_missing_metadata_table_fails(self):
        """When 2-column metadata table is missing, Check 20 raises SystemExit."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _create_valid_wbs_suite(tmpdir)
            md_path = os.path.join(tmpdir, "docs", "management", "WBS_DELIVERABLES_SUITE.md")
            with open(md_path, "r", encoding="utf-8") as f:
                lines = f.readlines()

            # Remove first 10 lines
            corrupted = "".join(lines[10:])
            with open(md_path, "w", encoding="utf-8") as f:
                f.write(corrupted)

            with self.assertRaises(SystemExit) as cm:
                check_wbs_suite_integrity(tmpdir)
            self.assertEqual(cm.exception.code, 1)

    def test_missing_traceability_matrix_fails(self):
        """When 7-column traceability matrix is missing, Check 20 raises SystemExit."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _create_valid_wbs_suite(tmpdir)
            md_path = os.path.join(tmpdir, "docs", "management", "WBS_DELIVERABLES_SUITE.md")
            with open(md_path, "r", encoding="utf-8") as f:
                content = f.read()

            # Remove 7-column header
            corrupted = content.replace("| SysML Component |", "| Component Name |")
            with open(md_path, "w", encoding="utf-8") as f:
                f.write(corrupted)

            with self.assertRaises(SystemExit) as cm:
                check_wbs_suite_integrity(tmpdir)
            self.assertEqual(cm.exception.code, 1)

    def test_csv_header_mismatch_fails(self):
        """When CSV header does not match 12 expected columns, Check 20 raises SystemExit."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _create_valid_wbs_suite(tmpdir)
            csv_path = os.path.join(tmpdir, "docs", "management", "wbs_export_jira_monday_ms_project.csv")
            with open(csv_path, "w", encoding="utf-8") as f:
                f.write('"Col1","Col2"\n"Val1","Val2"\n')

            with self.assertRaises(SystemExit) as cm:
                check_wbs_suite_integrity(tmpdir)
            self.assertEqual(cm.exception.code, 1)

    def test_csv_empty_data_rows_fails(self):
        """When CSV contains only headers and no data rows, Check 20 raises SystemExit."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _create_valid_wbs_suite(tmpdir)
            csv_path = os.path.join(tmpdir, "docs", "management", "wbs_export_jira_monday_ms_project.csv")
            with open(csv_path, "w", encoding="utf-8") as f:
                f.write(
                    '"WBS Code","ID","Item Type","Name","Parent ID","Subsystem","DO-178C Level","Artifact Path","Est. Hours","Verification Gate","Status","Description"\n'
                )

            with self.assertRaises(SystemExit) as cm:
                check_wbs_suite_integrity(tmpdir)
            self.assertEqual(cm.exception.code, 1)

    def test_json_missing_keys_fails(self):
        """When JSON AST export is missing required keys, Check 20 raises SystemExit."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _create_valid_wbs_suite(tmpdir)
            json_path = os.path.join(tmpdir, "docs", "management", "wbs_export.json")
            with open(json_path, "w", encoding="utf-8") as f:
                json.dump({"metadata": {}}, f)

            with self.assertRaises(SystemExit) as cm:
                check_wbs_suite_integrity(tmpdir)
            self.assertEqual(cm.exception.code, 1)

    def test_em_dash_detected_fails(self):
        """When Unicode em dash is present in any deliverable, Check 20 raises SystemExit."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _create_valid_wbs_suite(tmpdir)
            md_path = os.path.join(tmpdir, "docs", "management", "WBS_DELIVERABLES_SUITE.md")
            with open(md_path, "a", encoding="utf-8") as f:
                f.write("\nEm dash violation: \u2014\n")

            with self.assertRaises(SystemExit) as cm:
                check_wbs_suite_integrity(tmpdir)
            self.assertEqual(cm.exception.code, 1)

    def test_run_all_checks_passes_on_repo(self):
        """Verify run_all_checks passes on the active repository."""
        run_all_checks(repo_root)

    def _setup_mock_spec_workspace(self, workspace_path: Path) -> None:
        """Helper to scaffold a complete mock specification workspace."""
        (workspace_path / "schema").mkdir(parents=True, exist_ok=True)
        (workspace_path / "schema" / "model.sysml").write_text("// SysML SSOT", encoding="utf-8")

        conops_dir = workspace_path / "docs" / "conops"
        conops_dir.mkdir(parents=True, exist_ok=True)
        (conops_dir / "CONOPS.md").write_text("# Concept of Operations (ConOps)\n", encoding="utf-8")
        (conops_dir / "MISSION_INTENT.md").write_text("# Tactical Mission Intent\n", encoding="utf-8")

        safety_dir = workspace_path / "docs" / "safety"
        safety_dir.mkdir(parents=True, exist_ok=True)
        (safety_dir / "STPA_MATRIX.md").write_text("# Level 1B Safety Matrix\n", encoding="utf-8")

        icd_dir = workspace_path / "docs" / "interfaces"
        icd_dir.mkdir(parents=True, exist_ok=True)
        (icd_dir / "ICD_01_SYSTEM_INTERFACE_MATRIX.md").write_text("# System Interface Matrix\n", encoding="utf-8")
        (icd_dir / "ICD_02_MASTER_SIGNAL_DICTIONARY.md").write_text("# Signal Flow Dictionary\n", encoding="utf-8")

        epics_dir = workspace_path / "docs" / "epics"
        epics_dir.mkdir(parents=True, exist_ok=True)
        (epics_dir / "epic-01-navigation.md").write_text(
            "| Attribute | Detail |\n| :--- | :--- |\n| **Epic ID** | EPIC-01 |\n\n# Epic: Navigation\n\n**Subsystem:** Navigation\n",
            encoding="utf-8",
        )

        feat_dir = workspace_path / "docs" / "features"
        feat_dir.mkdir(parents=True, exist_ok=True)
        (feat_dir / "feat-01-state-estimation.md").write_text(
            "| Attribute | Detail |\n| :--- | :--- |\n| **Feature ID** | FEAT-01 |\n\n"
            "# Feature: State Estimation\n\n**Subsystem:** Navigation\n**Epic:** EPIC-01\n"
            "SysML Anchor: `SysSSOT::Nav::StateEstimator`\nSafety constraints: **SC-01**\n\n"
            "- Given valid sensor data, When sample arrives, Then update state.\n",
            encoding="utf-8",
        )

        us_dir = workspace_path / "docs" / "user-stories"
        us_dir.mkdir(parents=True, exist_ok=True)
        (us_dir / "us-01.md").write_text("# User Story: Ingestion\nRealizes: FEAT-01\n", encoding="utf-8")

        uc_dir = workspace_path / "docs" / "use-cases"
        uc_dir.mkdir(parents=True, exist_ok=True)
        (uc_dir / "uc-01.md").write_text("# Use Case: Flight\nRealizes: FEAT-01\n", encoding="utf-8")

    def test_wbs_generator_emits_document_relative_markdown_links(self):
        """Verify all markdown links in WBS_DELIVERABLES_SUITE.md are document-relative (Issue #240)."""
        with tempfile.TemporaryDirectory() as tmpdir:
            ws = Path(tmpdir)
            self._setup_mock_spec_workspace(ws)

            engine = WBSAstIngestionEngine(workspace_path=ws)
            engine.run_ingestion()
            synthesizer = WBSSuiteSynthesizer(engine)
            md_path, _, _ = synthesizer.synthesize_all()

            content = md_path.read_text(encoding="utf-8")

            # Find all markdown links [text](href)
            md_links = re.findall(r"\[([^\]]+)\]\(([^)]+)\)", content)
            self.assertTrue(len(md_links) > 0, "No markdown links found in generated WBS suite")

            for text, href in md_links:
                # No link should start with repo-root prefixes docs/, schema/, .pipeline/
                self.assertFalse(
                    href.startswith("docs/") or href.startswith("schema/") or href.startswith(".pipeline/"),
                    f"Found repo-root relative link target '{href}' (text: '{text}'). Must be document-relative.",
                )
                self.assertTrue(
                    href.startswith("../") or href.startswith("./") or href.startswith("#"),
                    f"Link target '{href}' is not document-relative (should start with '../' or './').",
                )

            # Check specific baseline & spec links
            hrefs = [h for _, h in md_links]
            self.assertIn("../conops/CONOPS.md", hrefs)
            self.assertIn("../../schema/model.sysml", hrefs)
            self.assertIn("../safety/STPA_MATRIX.md", hrefs)
            self.assertIn("../interfaces/ICD_01_SYSTEM_INTERFACE_MATRIX.md", hrefs)
            self.assertIn("../interfaces/ICD_02_MASTER_SIGNAL_DICTIONARY.md", hrefs)
            self.assertIn("../epics/epic-01-navigation.md", hrefs)
            self.assertIn("../features/feat-01-state-estimation.md", hrefs)
            self.assertIn("../user-stories/us-01.md", hrefs)

    def test_wbs_generator_formats_uncreated_phase3_artifacts_as_code_spans(self):
        """Verify uncreated Phase 3 code/report artifacts are rendered as code spans, not dead links (Issue #241)."""
        with tempfile.TemporaryDirectory() as tmpdir:
            ws = Path(tmpdir)
            self._setup_mock_spec_workspace(ws)

            # Ensure Phase 3 files DO NOT exist on disk
            self.assertFalse((ws / "models" / "matlab" / "feat_01_params.m").exists())
            self.assertFalse((ws / "models" / "scripts" / "build_feat_01_model.m").exists())
            self.assertFalse((ws / "models" / "python" / "feat_01_domain.py").exists())
            self.assertFalse((ws / "models" / "python" / "feat_01_engine.py").exists())
            self.assertFalse((ws / "tests" / "test_feat_01_simulation.py").exists())
            self.assertFalse((ws / "docs" / "reports" / "simulink_results" / "FEAT-01_results.md").exists())

            engine = WBSAstIngestionEngine(workspace_path=ws)
            engine.run_ingestion()
            synthesizer = WBSSuiteSynthesizer(engine)
            md_path, _, _ = synthesizer.synthesize_all()

            content = md_path.read_text(encoding="utf-8")

            # Verify uncreated artifacts are formatted as plain backticked code spans
            self.assertIn("`models/matlab/feat_01_params.m`", content)
            self.assertIn("`models/scripts/build_feat_01_model.m`", content)
            self.assertIn("`models/python/feat_01_domain.py`", content)
            self.assertIn("`models/python/feat_01_engine.py`", content)
            self.assertIn("`tests/test_feat_01_simulation.py`", content)
            self.assertIn("`docs/reports/simulink_results/FEAT-01_results.md`", content)

            # Ensure they are NOT rendered as markdown links
            md_links = re.findall(r"\[([^\]]+)\]\(([^)]+)\)", content)
            link_hrefs = [h for _, h in md_links]
            for href in link_hrefs:
                self.assertNotIn("build_feat_01_model.m", href)
                self.assertNotIn("feat_01_engine.py", href)
                self.assertNotIn("test_feat_01_simulation.py", href)
                self.assertNotIn("FEAT-01_results.md", href)

    def test_wbs_generator_links_existing_phase3_artifacts_when_present(self):
        """Verify Phase 3 artifacts are rendered as document-relative links when they exist on disk (Issue #241)."""
        with tempfile.TemporaryDirectory() as tmpdir:
            ws = Path(tmpdir)
            self._setup_mock_spec_workspace(ws)

            # Create Phase 3 deliverable files
            (ws / "models" / "matlab").mkdir(parents=True, exist_ok=True)
            (ws / "models" / "scripts").mkdir(parents=True, exist_ok=True)
            (ws / "models" / "python").mkdir(parents=True, exist_ok=True)
            (ws / "tests").mkdir(parents=True, exist_ok=True)
            (ws / "docs" / "reports" / "simulink_results").mkdir(parents=True, exist_ok=True)

            (ws / "models" / "matlab" / "feat_01_params.m").write_text("% params", encoding="utf-8")
            (ws / "models" / "scripts" / "build_feat_01_model.m").write_text("% builder", encoding="utf-8")
            (ws / "models" / "python" / "feat_01_domain.py").write_text("# domain", encoding="utf-8")
            (ws / "models" / "python" / "feat_01_engine.py").write_text("# engine", encoding="utf-8")
            (ws / "tests" / "test_feat_01_simulation.py").write_text("# test", encoding="utf-8")
            (ws / "docs" / "reports" / "simulink_results" / "FEAT-01_results.md").write_text("# results", encoding="utf-8")

            engine = WBSAstIngestionEngine(workspace_path=ws)
            engine.run_ingestion()
            synthesizer = WBSSuiteSynthesizer(engine)
            md_path, _, _ = synthesizer.synthesize_all()

            content = md_path.read_text(encoding="utf-8")

            # Verify that existing artifacts are rendered as document-relative links
            self.assertIn("[`models/matlab/feat_01_params.m`](../../models/matlab/feat_01_params.m)", content)
            self.assertIn("[`models/scripts/build_feat_01_model.m`](../../models/scripts/build_feat_01_model.m)", content)
            self.assertIn("[`models/python/feat_01_domain.py`](../../models/python/feat_01_domain.py)", content)
            self.assertIn("[`models/python/feat_01_engine.py`](../../models/python/feat_01_engine.py)", content)
            self.assertIn("[`tests/test_feat_01_simulation.py`](../../tests/test_feat_01_simulation.py)", content)
            self.assertIn("[Results Report](../reports/simulink_results/FEAT-01_results.md)", content)


if __name__ == "__main__":
    unittest.main()
