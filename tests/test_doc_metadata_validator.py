"""
Unit tests for DocMetadataValidator.

Tests:
1. Pure Markdown header (H1 Title + Visual Table, zero YAML frontmatter) passes 100%.
2. Full 3-part header (YAML + H1 + Table) passes 100%.
3. Positive validation of valid ISO dates (YYYY-MM-DD), semver versions (v?X.Y[.Z]), and required fields.
4. Negative validation of invalid dates (e.g. "August 2026", "2026/08/31", "2026-02-30").
5. Negative validation of missing fields (missing Title, missing Version, missing Date).
6. Negative validation of invalid version strings (e.g. "draft", "1", "alpha").
7. Graceful skipping of READMEs, empty stubs, and documents marked optional.
8. Detection and rejection of concatenated title strings in H1 heading, YAML frontmatter, or tables.
9. Validation of matching YAML frontmatter, H1 heading, and visual Markdown tables.
10. Negative validation of mismatch between YAML title and H1 heading.
11. Positive validation of agile specs with prefixes (# Epic: EPIC-001 — ...).
"""

import os
import sys
import tempfile
import unittest

repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if repo_root not in sys.path:
    sys.path.insert(0, repo_root)

parity_src = os.path.join(repo_root, "skills", "spec-orchestrator", "parity_auditor", "src")
if parity_src not in sys.path:
    sys.path.insert(0, parity_src)

from parity_auditor.core.workspace import WorkspaceRepository
from parity_auditor.validators.doc_metadata_validator import (
    DocMetadataValidator,
    _is_iso_date,
    _is_semver,
    _has_concatenated_title_metadata,
    _strip_agile_prefix,
    _extract_h1_heading,
    _extract_yaml_frontmatter,
)


class TestDocMetadataValidator(unittest.TestCase):
    def setUp(self):
        self.validator = DocMetadataValidator()

    def test_helper_is_iso_date(self):
        """Verify ISO 8601 date validation helper."""
        self.assertTrue(_is_iso_date("2026-08-31"))
        self.assertTrue(_is_iso_date("2025-01-01"))
        self.assertTrue(_is_iso_date("2024-02-29"))  # leap year

        self.assertFalse(_is_iso_date("August 2026"))
        self.assertFalse(_is_iso_date("2026/08/31"))
        self.assertFalse(_is_iso_date("31-08-2026"))
        self.assertFalse(_is_iso_date("2026-02-30"))  # invalid day
        self.assertFalse(_is_iso_date("2026-13-01"))  # invalid month
        self.assertFalse(_is_iso_date(""))
        self.assertFalse(_is_iso_date(None))

    def test_helper_is_semver(self):
        """Verify semantic versioning validation helper."""
        self.assertTrue(_is_semver("1.0"))
        self.assertTrue(_is_semver("1.0.0"))
        self.assertTrue(_is_semver("v1.0"))
        self.assertTrue(_is_semver("v1.0.0"))
        self.assertTrue(_is_semver("0.1"))
        self.assertTrue(_is_semver("v0.2.1"))
        self.assertTrue(_is_semver("1.2.3-alpha.1"))
        self.assertTrue(_is_semver("v2.0.0+20260831"))

        self.assertFalse(_is_semver("draft"))
        self.assertFalse(_is_semver("1"))
        self.assertFalse(_is_semver("v1"))
        self.assertFalse(_is_semver("August 2026"))
        self.assertFalse(_is_semver("Version 1.0"))
        self.assertFalse(_is_semver(""))
        self.assertFalse(_is_semver(None))

    def test_helper_strip_agile_prefix(self):
        """Verify stripping of agile prefixes from H1 heading text."""
        self.assertEqual(_strip_agile_prefix("Epic: EPIC-001 — Core System Architecture"), "EPIC-001 — Core System Architecture")
        self.assertEqual(_strip_agile_prefix("Feature: FEAT-101 — Subsystem Mounting"), "FEAT-101 — Subsystem Mounting")
        self.assertEqual(_strip_agile_prefix("Use Case: UC-001 — Assemble Subsystem"), "UC-001 — Assemble Subsystem")
        self.assertEqual(_strip_agile_prefix("User Story: US-001 — Coupling Verification"), "US-001 — Coupling Verification")
        self.assertEqual(_strip_agile_prefix("Autonomous UAS Safety Concept"), "Autonomous UAS Safety Concept")
        self.assertEqual(_strip_agile_prefix("**Epic:** Flight Guidance"), "Flight Guidance")

    def test_helper_extract_h1_heading(self):
        """Verify extraction of first H1 heading outside frontmatter and code blocks."""
        doc_with_fm = """---
title: System Architecture
version: 1.0.0
date: 2026-08-31
---

```markdown
# Ignored Code Comment
```

# Epic: EPIC-001 — Core Architecture

## Section 1
"""
        h1_info = _extract_h1_heading(doc_with_fm)
        self.assertIsNotNone(h1_info)
        self.assertEqual(h1_info[0], "EPIC-001 — Core Architecture")
        self.assertEqual(h1_info[1], 11)

    def test_pure_markdown_header_without_yaml_passes(self):
        """Verify pure Markdown header (H1 Title + Visual Table, zero YAML frontmatter) passes 100%."""
        content = """# Autonomous UAS Infrastructure Safety Concept of Operations

| Attribute | Specification Detail |
| :--- | :--- |
| **Document ID** | DOC-CONOPS-001 |
| **Version** | 1.0.0 |
| **Date** | 2026-08-31 |
| **Status** | APPROVED |
| **Target Baseline** | v1.0.0 |

## 1. Executive Summary
Operational scope description.
"""
        # Test extract_metadata directly
        extracted, is_optional = self.validator.extract_metadata(content)
        self.assertFalse(is_optional)
        self.assertEqual(extracted["title"][0], "Autonomous UAS Infrastructure Safety Concept of Operations")
        self.assertEqual(extracted["version"][0], "1.0.0")
        self.assertEqual(extracted["date"][0], "2026-08-31")
        self.assertEqual(extracted["status"][0], "APPROVED")
        self.assertEqual(extracted["target_baseline"][0], "v1.0.0")
        self.assertEqual(extracted["document_id"][0], "DOC-CONOPS-001")

        # Test full repository validation
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_conops = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(docs_conops, exist_ok=True)
            with open(os.path.join(docs_conops, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            errors = self.validator.validate(repo)
            self.assertEqual(errors, [])

    def test_pure_markdown_header_with_title_in_table_passes(self):
        """Verify pure Markdown header with title row in visual table passes 100%."""
        content = """# Autonomous UAS Infrastructure Safety Concept of Operations

| Attribute | Specification Detail |
| :--- | :--- |
| **Title** | Autonomous UAS Infrastructure Safety Concept of Operations |
| **Document ID** | DOC-CONOPS-001 |
| **Version** | 1.0.0 |
| **Date** | 2026-08-31 |
| **Status** | APPROVED |

## 1. Executive Summary
Operational scope description.
"""
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_conops = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(docs_conops, exist_ok=True)
            with open(os.path.join(docs_conops, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            errors = self.validator.validate(repo)
            self.assertEqual(errors, [])

    def test_full_three_part_header_passes(self):
        """Verify full 3-part header (YAML frontmatter + H1 Title + Visual Table) passes 100%."""
        content = """---
title: Autonomous UAS Architecture Specification
version: 1.0.0
date: 2026-08-31
---

# Autonomous UAS Architecture Specification

| Attribute | Specification Detail |
| :--- | :--- |
| **Document ID** | DOC-ARCH-001 |
| **Title** | Autonomous UAS Architecture Specification |
| **Version** | 1.0.0 |
| **Date** | 2026-08-31 |
| **Status** | APPROVED |
| **Target Baseline** | v1.0.0 |

## 1. System Overview
Architecture description.
"""
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_arch = os.path.join(tmpdir, "docs", "architecture")
            os.makedirs(docs_arch, exist_ok=True)
            with open(os.path.join(docs_arch, "ARCH_SPEC.md"), "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            errors = self.validator.validate(repo)
            self.assertEqual(errors, [])

    def test_valid_vertical_table_passes(self):
        """Verify valid vertical frontmatter metadata table passes without findings."""
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_conops = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(docs_conops, exist_ok=True)

            content = """# Concept of Operations

| Metadata | Value |
| :--- | :--- |
| **Title** | Autonomous UAS Infrastructure Safety Concept of Operations |
| **Version** | 1.0.0 |
| **Date** | 2026-08-31 |
| **Status** | APPROVED |

## 1. Executive Summary
Operational scope description.
"""
            with open(os.path.join(docs_conops, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            errors = self.validator.validate(repo)
            self.assertEqual(errors, [])

    def test_valid_release_date_and_v_prefix_passes(self):
        """Verify valid metadata using 'Release Date' and version with 'v' prefix passes."""
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_safety = os.path.join(tmpdir, "docs", "safety")
            os.makedirs(docs_safety, exist_ok=True)

            content = """# STPA Matrix Specification

| Field | Value |
| --- | --- |
| Title | Safety Integrity & SORA Assessment Matrix |
| Version | v2.1.0 |
| Release Date | 2026-09-01 |

## 1. Losses and Hazards
"""
            with open(os.path.join(docs_safety, "STPA_MATRIX.md"), "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            errors = self.validator.validate(repo)
            self.assertEqual(errors, [])

    def test_valid_horizontal_columnar_table_passes(self):
        """Verify valid horizontal columnar frontmatter table passes."""
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_arch = os.path.join(tmpdir, "docs", "architecture")
            os.makedirs(docs_arch, exist_ok=True)

            content = """# Architectural Blueprint

| Title | Version | Date | Status |
| :--- | :--- | :--- | :--- |
| Run-Time Assurance Monitor Architecture | 0.2.0 | 2026-08-20 | Approved |

## System Overview
"""
            with open(os.path.join(docs_arch, "RTA_ARCH.md"), "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            errors = self.validator.validate(repo)
            self.assertEqual(errors, [])

    def test_missing_all_metadata_fields_when_no_h1_and_no_table(self):
        """Verify document missing H1 title and metadata table emits doc-metadata-missing-field for all fields."""
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_conops = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(docs_conops, exist_ok=True)

            content = """This document has no H1 heading and no frontmatter metadata table.
Just plain text without metadata.
"""
            with open(os.path.join(docs_conops, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            errors = self.validator.validate(repo)
            self.assertEqual(len(errors), 1)
            self.assertEqual(errors[0].rule_id, "doc-metadata-missing-field")
            self.assertIn("Title", str(errors[0]))
            self.assertIn("Version", str(errors[0]))
            self.assertIn("Date", str(errors[0]))

    def test_missing_version_and_date_with_h1_title(self):
        """Verify document with H1 title but missing Version and Date emits doc-metadata-missing-field."""
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_conops = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(docs_conops, exist_ok=True)

            content = """# Plain Document Without Metadata Table

This document does not contain a frontmatter table.
"""
            with open(os.path.join(docs_conops, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            errors = self.validator.validate(repo)
            self.assertEqual(len(errors), 1)
            self.assertEqual(errors[0].rule_id, "doc-metadata-missing-field")
            self.assertNotIn("Title", errors[0].detail["missing_fields"])
            self.assertIn("Version", errors[0].detail["missing_fields"])
            self.assertIn("Date (or Release Date)", errors[0].detail["missing_fields"])

    def test_missing_single_field_version(self):
        """Verify table missing Version field emits doc-metadata-missing-field."""
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_conops = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(docs_conops, exist_ok=True)

            content = """# Concept of Operations

| Field | Value |
| --- | --- |
| Title | UAS CONOPS |
| Date | 2026-08-31 |
"""
            with open(os.path.join(docs_conops, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            errors = self.validator.validate(repo)
            self.assertEqual(len(errors), 1)
            self.assertEqual(errors[0].rule_id, "doc-metadata-missing-field")
            self.assertIn("Version", str(errors[0]))

    def test_missing_single_field_date(self):
        """Verify table missing Date field emits doc-metadata-missing-field."""
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_conops = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(docs_conops, exist_ok=True)

            content = """# Concept of Operations

| Field | Value |
| --- | --- |
| Title | UAS CONOPS |
| Version | 1.0.0 |
"""
            with open(os.path.join(docs_conops, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            errors = self.validator.validate(repo)
            self.assertEqual(len(errors), 1)
            self.assertEqual(errors[0].rule_id, "doc-metadata-missing-field")
            self.assertIn("Date", str(errors[0]))

    def test_invalid_date_format_month_name(self):
        """Verify invalid date format like 'August 2026' emits doc-metadata-invalid-date-format."""
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_conops = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(docs_conops, exist_ok=True)

            content = """# Concept of Operations

| Field | Value |
| --- | --- |
| Title | UAS CONOPS |
| Version | 1.0.0 |
| Date | August 2026 |
"""
            with open(os.path.join(docs_conops, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            errors = self.validator.validate(repo)
            self.assertEqual(len(errors), 1)
            self.assertEqual(errors[0].rule_id, "doc-metadata-invalid-date-format")
            self.assertIn("August 2026", str(errors[0]))

    def test_invalid_date_format_slash(self):
        """Verify invalid date format with slashes emits doc-metadata-invalid-date-format."""
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_conops = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(docs_conops, exist_ok=True)

            content = """# Concept of Operations

| Field | Value |
| --- | --- |
| Title | UAS CONOPS |
| Version | 1.0.0 |
| Release Date | 2026/08/31 |
"""
            with open(os.path.join(docs_conops, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            errors = self.validator.validate(repo)
            self.assertEqual(len(errors), 1)
            self.assertEqual(errors[0].rule_id, "doc-metadata-invalid-date-format")
            self.assertIn("2026/08/31", str(errors[0]))

    def test_invalid_version_format(self):
        """Verify invalid version strings emit doc-metadata-invalid-version-format."""
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_conops = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(docs_conops, exist_ok=True)

            content = """# Concept of Operations

| Field | Value |
| --- | --- |
| Title | UAS CONOPS |
| Version | draft |
| Date | 2026-08-31 |
"""
            with open(os.path.join(docs_conops, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            errors = self.validator.validate(repo)
            self.assertEqual(len(errors), 1)
            self.assertEqual(errors[0].rule_id, "doc-metadata-invalid-version-format")
            self.assertIn("draft", str(errors[0]))

    def test_skips_readme_and_stubs_and_optional_documents(self):
        """Verify README.md files, empty stubs, and files marked optional are skipped."""
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_conops = os.path.join(tmpdir, "docs", "conops")
            docs_safety = os.path.join(tmpdir, "docs", "safety")
            os.makedirs(docs_conops, exist_ok=True)
            os.makedirs(docs_safety, exist_ok=True)

            # README.md without metadata table
            with open(os.path.join(docs_conops, "README.md"), "w", encoding="utf-8") as f:
                f.write("# CONOPS Directory\nLanding zone placeholder.\n")

            # Empty stub file
            with open(os.path.join(docs_safety, "stub.md"), "w", encoding="utf-8") as f:
                f.write("")

            # Optional document with comment
            with open(os.path.join(docs_conops, "draft_notes.md"), "w", encoding="utf-8") as f:
                f.write("<!-- optional -->\n# Draft Notes\nInformal scratchpad.\n")

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            errors = self.validator.validate(repo)
            self.assertEqual(errors, [])

    def test_helper_has_concatenated_title_metadata(self):
        """Verify concatenated title metadata helper detects versions, dates, and doc IDs."""
        self.assertFalse(_has_concatenated_title_metadata("Mission Intent — AVENGER 5 Autonomous UAS"))
        self.assertFalse(_has_concatenated_title_metadata("Autonomous UAS Infrastructure Safety Concept of Operations"))
        self.assertFalse(_has_concatenated_title_metadata("STPA Matrix Specification"))

        self.assertTrue(_has_concatenated_title_metadata("Mission Intent (DOC-MI-A5-001 v3.0.0 2026-08-31)"))
        self.assertTrue(_has_concatenated_title_metadata("Mission Intent v1.0.0"))
        self.assertTrue(_has_concatenated_title_metadata("Mission Intent v2.1"))
        self.assertTrue(_has_concatenated_title_metadata("Mission Intent 1.0.0"))
        self.assertTrue(_has_concatenated_title_metadata("Mission Intent (version: 2)"))
        self.assertTrue(_has_concatenated_title_metadata("Mission Intent 2026-08-31"))
        self.assertTrue(_has_concatenated_title_metadata("Mission Intent (DOC-MI-A5-001)"))

    def test_clean_canonical_title_passes(self):
        """Verify clean canonical document title passes without finding."""
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_conops = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(docs_conops, exist_ok=True)

            content = """# Mission Intent

| Metadata | Value |
| :--- | :--- |
| **Title** | Mission Intent — AVENGER 5 Autonomous UAS |
| **Version** | 1.0.0 |
| **Date** | 2026-08-31 |
| **Status** | APPROVED |

## 1. Executive Summary
Operational scope description.
"""
            with open(os.path.join(docs_conops, "MISSION_INTENT.md"), "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            errors = self.validator.validate(repo)
            self.assertEqual(errors, [])

    def test_concatenated_title_in_table_fails(self):
        """Verify document title in table containing concatenated metadata attributes emits doc-metadata-concatenated-title."""
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_conops = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(docs_conops, exist_ok=True)

            content = """# Mission Intent

| Metadata | Value |
| :--- | :--- |
| **Title** | Mission Intent (DOC-MI-A5-001 v3.0.0 2026-08-31) |
| **Version** | 3.0.0 |
| **Date** | 2026-08-31 |
| **Status** | DRAFT |

## 1. Executive Summary
Operational scope description.
"""
            with open(os.path.join(docs_conops, "MISSION_INTENT.md"), "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            errors = self.validator.validate(repo)
            self.assertEqual(len(errors), 1)
            self.assertEqual(errors[0].rule_id, "doc-metadata-concatenated-title")
            self.assertIn("Document title contains concatenated metadata attributes", str(errors[0]))
            self.assertIn("Mission Intent (DOC-MI-A5-001 v3.0.0 2026-08-31)", str(errors[0]))

    def test_concatenated_title_in_h1_heading_fails(self):
        """Verify document title in H1 heading containing concatenated metadata attributes emits doc-metadata-concatenated-title."""
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_conops = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(docs_conops, exist_ok=True)

            content = """# Mission Intent (DOC-MI-A5-001 v3.0.0 2026-08-31)

| Attribute | Specification Detail |
| :--- | :--- |
| **Document ID** | DOC-MI-A5-001 |
| **Version** | 3.0.0 |
| **Date** | 2026-08-31 |
| **Status** | DRAFT |

## 1. Executive Summary
Operational scope description.
"""
            with open(os.path.join(docs_conops, "MISSION_INTENT.md"), "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            errors = self.validator.validate(repo)
            self.assertEqual(len(errors), 1)
            self.assertEqual(errors[0].rule_id, "doc-metadata-concatenated-title")
            self.assertIn("Document title contains concatenated metadata attributes", str(errors[0]))

    def test_yaml_title_heading_mismatch_fails(self):
        """Verify mismatch between YAML frontmatter title and H1 heading emits doc-metadata-title-heading-mismatch."""
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_arch = os.path.join(tmpdir, "docs", "architecture")
            os.makedirs(docs_arch, exist_ok=True)

            content = """---
title: System Architecture Baseline
version: 1.0.0
date: 2026-08-31
---

# Autonomous UAS Architecture Specification

| Attribute | Specification Detail |
| :--- | :--- |
| **Title** | Autonomous UAS Architecture Specification |
| **Version** | 1.0.0 |
| **Date** | 2026-08-31 |

## 1. System Overview
Architecture description.
"""
            with open(os.path.join(docs_arch, "ARCH_SPEC.md"), "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            errors = self.validator.validate(repo)
            self.assertEqual(len(errors), 1)
            self.assertEqual(errors[0].rule_id, "doc-metadata-title-heading-mismatch")
            self.assertIn("docs/architecture/ARCH_SPEC.md: YAML title ('System Architecture Baseline') does not match H1 heading ('Autonomous UAS Architecture Specification').", str(errors[0]))

    def test_concatenated_title_in_yaml_fails(self):
        """Verify YAML title containing concatenated version/date/doc ID emits doc-metadata-concatenated-title."""
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_arch = os.path.join(tmpdir, "docs", "architecture")
            os.makedirs(docs_arch, exist_ok=True)

            content = """---
title: Flight Systems (DOC-FS-001 v2.0 2026-08-31)
version: 2.0.0
date: 2026-08-31
---

# Flight Systems (DOC-FS-001 v2.0 2026-08-31)

| Attribute | Specification Detail |
| :--- | :--- |
| **Title** | Flight Systems (DOC-FS-001 v2.0 2026-08-31) |
| **Version** | 2.0.0 |
| **Date** | 2026-08-31 |
"""
            with open(os.path.join(docs_arch, "FLIGHT_SYS.md"), "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            errors = self.validator.validate(repo)
            concatenated_findings = [e for e in errors if e.rule_id == "doc-metadata-concatenated-title"]
            self.assertTrue(len(concatenated_findings) >= 1)
            self.assertIn("Document title contains concatenated metadata attributes", str(concatenated_findings[0]))

    def test_agile_spec_with_prefixes_matches_yaml_title_passes(self):
        """Verify agile specs with prefixes (Epic:, Feature:, Use Case:, User Story:) match YAML title and pass."""
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_epics = os.path.join(tmpdir, "docs", "epics")
            docs_features = os.path.join(tmpdir, "docs", "features")
            docs_use_cases = os.path.join(tmpdir, "docs", "use-cases")
            docs_user_stories = os.path.join(tmpdir, "docs", "user-stories")
            os.makedirs(docs_epics, exist_ok=True)
            os.makedirs(docs_features, exist_ok=True)
            os.makedirs(docs_use_cases, exist_ok=True)
            os.makedirs(docs_user_stories, exist_ok=True)

            # 1. Epic
            epic_content = """---
title: EPIC-001 — Core System Architecture
version: 1.0.0
date: 2026-08-31
---

# Epic: EPIC-001 — Core System Architecture

| Attribute | Specification Detail |
| :--- | :--- |
| **Title** | EPIC-001 — Core System Architecture |
| **Version** | 1.0.0 |
| **Date** | 2026-08-31 |
"""
            with open(os.path.join(docs_epics, "EPIC-001.md"), "w", encoding="utf-8") as f:
                f.write(epic_content)

            # 2. Feature
            feat_content = """---
title: FEAT-101 — Subsystem Structural Mounting
version: 1.0.0
date: 2026-08-31
---

# Feature: FEAT-101 — Subsystem Structural Mounting

| Attribute | Specification Detail |
| :--- | :--- |
| **Title** | FEAT-101 — Subsystem Structural Mounting |
| **Version** | 1.0.0 |
| **Date** | 2026-08-31 |
"""
            with open(os.path.join(docs_features, "FEAT-101.md"), "w", encoding="utf-8") as f:
                f.write(feat_content)

            # 3. Use Case
            uc_content = """---
title: UC-001 — Assemble and Verify Subsystem
version: 1.0.0
date: 2026-08-31
---

# Use Case: UC-001 — Assemble and Verify Subsystem

| Attribute | Specification Detail |
| :--- | :--- |
| **Title** | UC-001 — Assemble and Verify Subsystem |
| **Version** | 1.0.0 |
| **Date** | 2026-08-31 |
"""
            with open(os.path.join(docs_use_cases, "UC-001.md"), "w", encoding="utf-8") as f:
                f.write(uc_content)

            # 4. User Story
            us_content = """---
title: US-001 — Subsystem Coupling Verification
version: 1.0.0
date: 2026-08-31
---

# User Story: US-001 — Subsystem Coupling Verification

| Attribute | Specification Detail |
| :--- | :--- |
| **Title** | US-001 — Subsystem Coupling Verification |
| **Version** | 1.0.0 |
| **Date** | 2026-08-31 |
"""
            with open(os.path.join(docs_user_stories, "US-001.md"), "w", encoding="utf-8") as f:
                f.write(us_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            errors = self.validator.validate(repo)
            self.assertEqual(errors, [])


if __name__ == "__main__":
    unittest.main()
