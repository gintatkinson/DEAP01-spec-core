#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Unit test suite for Check 20 (WBS & Enterprise Deliverables Suite Validation)
in scripts/verify_downstream_baseline.py.
"""

import json
import os
import sys
import tempfile
import unittest

repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if repo_root not in sys.path:
    sys.path.insert(0, repo_root)

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


if __name__ == "__main__":
    unittest.main()
