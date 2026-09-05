#!/usr/bin/env python3
"""
Unit tests for Defect Dossier Schema Validator & Issue Filer (`scripts/file_defect.py`).
Verifies that `file_defect.py` enforces the 7-section Adversarial Audit schema,
5 Whys structure, offline Mermaid validation, balanced fences, and provider resolution.
"""

import os
import subprocess
import sys
import tempfile
import unittest

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if REPO_ROOT not in sys.path:
    sys.path.insert(0, REPO_ROOT)

from scripts.file_defect import (
    validate_defect_body,
    resolve_label,
)


SAMPLE_COMPLIANT_CRITICAL = """## 1. Context and References

- **File**: `cesium_native_bridge/src/bridge.cpp:56-61`
- **Pillar**: Memory Safety
- **Symptom**: Dart FFI caller reads garbage or crashes after calling bridge_get_last_error when another thread concurrently calls bridge_shutdown on the same handle.

## 2. Root Cause Analysis (5 Whys)

1. **Why does the Dart VM crash?** Because it dereferences a pointer whose backing memory was freed.
2. **Why was the memory freed?** Because bridge_get_last_error returns c_str() and releases the mutex; a concurrent bridge_shutdown erases the BridgeState.
3. **Why return a raw pointer to internal state?** Because the API was designed for zero-copy convenience.
4. **Why is that assumption violated?** Because no ownership protocol or lifetime contract exists across the FFI boundary.
5. **Why was no contract designed?** Because the C FFI pattern chose raw C string returns without ownership semantics.

## 3. Correctness Analysis

Thread T1 calls bridge_get_last_error, releases mutex. Thread T2 calls bridge_shutdown, freeing buffer.

## 4. UML Diagrams

```mermaid
sequenceDiagram
    participant T1 as Thread 1 (Get Error)
    participant T2 as Thread 2 (Shutdown)
    T1->>T2: Concurrent execution
    Note over T1: Acquires mutex, extracts c_str pointer p, releases mutex
    Note over T2: Acquires mutex, erases BridgeState, deallocates string backing p
    T1-->>T2: Dereferences p (Use-After-Free)
```

## 5. Affected Callers / Downstream Impact

Dart FFI caller getLastError() receives dangling pointer after concurrent shutdown.

## 6. Proposed Correction

```cpp
int32_t bridge_get_last_error(bridge_handle_t handle, char* out, int32_t size) {
    return BRIDGE_OK;
}
```

## 7. Relationship to Existing Issues

Discovered in audit — new finding.

## Audit Source

Adversarial Memory Safety Audit
SEVERITY: Critical
FILE_LOCATION: cesium_native_bridge/src/bridge.cpp:56-61
"""


SAMPLE_COMPLIANT_SUGGESTION = """## 1. Context and References

- **File**: `src/allocator.cpp:10-20`
- **Pillar**: Resource Lifecycle
- **Symptom**: Potential unbounded cache growth in long-running headless simulations.

## 2. Root Cause Analysis (5 Whys)

1. **Why could memory grow?** Because cache eviction threshold is fixed rather than memory-adaptive.
2. **Why is it fixed?** Because default settings assume desktop hardware with ample memory.
3. **Why assume desktop?** Because initial deployment target was single-workstation testing.
4. **Why not parametrize?** Because embedded deployment requirements were not yet scoped.
5. **Why were they not scoped?** Because initial architectural baseline prioritized functional correctness.

## 3. Correctness Analysis

Analysis shows no immediate leak under normal test loads.

## 4. UML Diagrams

N/A — Suggestion severity.

## 5. Affected Callers / Downstream Impact

Future long-running batch workers.

## 6. Proposed Correction

```cpp
void set_cache_limit(size_t limit);
```

## 7. Relationship to Existing Issues

Discovered in audit — new finding.

## Audit Source

Adversarial Resource Lifecycle Audit
SEVERITY: Suggestion
FILE_LOCATION: src/allocator.cpp:10-20
"""


class TestFileDefectIntegrity(unittest.TestCase):
    """Test suite for defect dossier validation and CLI functionality."""

    def test_compliant_critical_dossier_passes(self):
        """Verify that a compliant Critical defect dossier passes validation with 0 errors."""
        errors = validate_defect_body(SAMPLE_COMPLIANT_CRITICAL, title="[AUDIT] bridge.cpp: UAF")
        self.assertEqual(errors, [])

    def test_compliant_suggestion_dossier_passes(self):
        """Verify that a compliant Suggestion defect dossier with N/A diagram passes with 0 errors."""
        errors = validate_defect_body(SAMPLE_COMPLIANT_SUGGESTION, title="[AUDIT] allocator.cpp: cache limit")
        self.assertEqual(errors, [])

    def test_missing_section_header_rejected(self):
        """Verify that omitting a numbered section header fails validation."""
        broken = SAMPLE_COMPLIANT_CRITICAL.replace("## 5. Affected Callers", "### Affected Callers")
        errors = validate_defect_body(broken)
        self.assertTrue(any("Section headers error" in e for e in errors))

    def test_out_of_order_sections_rejected(self):
        """Verify that misnumbered sections are rejected."""
        broken = SAMPLE_COMPLIANT_CRITICAL.replace("## 4. UML Diagrams", "## 8. UML Diagrams")
        errors = validate_defect_body(broken)
        self.assertTrue(any("Section headers error" in e for e in errors))

    def test_missing_audit_source_header_rejected(self):
        """Verify that omitting ## Audit Source is rejected."""
        broken = SAMPLE_COMPLIANT_CRITICAL.replace("## Audit Source", "## Audit Summary")
        errors = validate_defect_body(broken)
        self.assertTrue(any("Missing mandatory '## Audit Source'" in e for e in errors))

    def test_missing_or_invalid_severity_rejected(self):
        """Verify that invalid or missing SEVERITY line is rejected."""
        broken_invalid = SAMPLE_COMPLIANT_CRITICAL.replace("SEVERITY: Critical", "SEVERITY: High")
        errors_invalid = validate_defect_body(broken_invalid)
        self.assertTrue(any("Invalid SEVERITY" in e for e in errors_invalid))

        broken_missing = SAMPLE_COMPLIANT_CRITICAL.replace("SEVERITY: Critical", "")
        errors_missing = validate_defect_body(broken_missing)
        self.assertTrue(any("Missing mandatory 'SEVERITY:'" in e for e in errors_missing))

    def test_missing_file_location_rejected(self):
        """Verify that missing FILE_LOCATION line is rejected."""
        broken = SAMPLE_COMPLIANT_CRITICAL.replace("FILE_LOCATION: cesium_native_bridge/src/bridge.cpp:56-61", "")
        errors = validate_defect_body(broken)
        self.assertTrue(any("FILE_LOCATION" in e for e in errors))

    def test_incomplete_5_whys_rejected(self):
        """Verify that Section 2 with fewer than 5 Whys is rejected."""
        broken = SAMPLE_COMPLIANT_CRITICAL.replace(
            "5. **Why was no contract designed?** Because the C FFI pattern chose raw C string returns without ownership semantics.",
            ""
        )
        errors = validate_defect_body(broken)
        self.assertTrue(any("Section 2 must contain exactly 5 'Why ...? Because ...'" in e for e in errors))

    def test_critical_missing_mermaid_block_rejected(self):
        """Verify that Critical finding without ```mermaid block in Section 4 is rejected."""
        broken = SAMPLE_COMPLIANT_CRITICAL.replace("```mermaid", "```text")
        errors = validate_defect_body(broken)
        self.assertTrue(any("Section 4 must contain a ```mermaid code block" in e for e in errors))

    def test_critical_invalid_mermaid_syntax_rejected(self):
        """Verify that Critical finding with invalid Mermaid diagram syntax (e.g. semicolon in Note) is rejected."""
        broken = SAMPLE_COMPLIANT_CRITICAL.replace(
            "Note over T1: Acquires mutex, extracts c_str pointer p, releases mutex",
            "Note over T1: Acquires mutex; extracts c_str pointer p"
        )
        errors = validate_defect_body(broken)
        self.assertTrue(any("Mermaid syntax error in Section 4" in e for e in errors))

    def test_suggestion_missing_na_rejected(self):
        """Verify that Suggestion finding without N/A in Section 4 is rejected."""
        broken = SAMPLE_COMPLIANT_SUGGESTION.replace("N/A — Suggestion severity.", "No diagram provided.")
        errors = validate_defect_body(broken)
        self.assertTrue(any("Section 4 must declare 'N/A — Suggestion severity.'" in e for e in errors))

    def test_unbalanced_code_blocks_rejected(self):
        """Verify that odd number of code block fences is rejected."""
        broken = SAMPLE_COMPLIANT_CRITICAL + "\n```\n"
        errors = validate_defect_body(broken)
        self.assertTrue(any("Unbalanced code blocks" in e for e in errors))

    def test_ascii_art_uml_arrows_in_prose_rejected(self):
        """Verify that ASCII art arrows in prose outside code blocks are rejected."""
        broken = SAMPLE_COMPLIANT_CRITICAL.replace(
            "Dart FFI caller getLastError() receives dangling pointer after concurrent shutdown.",
            "Dart FFI caller ->> getLastError() receives dangling pointer ->> crash."
        )
        errors = validate_defect_body(broken)
        self.assertTrue(any("ASCII art arrow" in e for e in errors))

    def test_missing_section1_bullets_rejected(self):
        """Verify that Section 1 missing File, Pillar, or Symptom bullet is rejected."""
        broken = SAMPLE_COMPLIANT_CRITICAL.replace("- **Pillar**: Memory Safety", "")
        errors = validate_defect_body(broken)
        self.assertTrue(any("Section 1 missing mandatory bullet point" in e for e in errors))

    def test_resolve_label_mappings(self):
        """Verify that resolve_label maps severities to provider labels properly."""
        self.assertEqual(resolve_label("Critical", provider="github"), "bug")
        self.assertEqual(resolve_label("Important", provider="github"), "bug")
        self.assertEqual(resolve_label("Suggestion", provider="github"), "enhancement")
        self.assertEqual(resolve_label("Nitpick", provider="github"), "enhancement")

        self.assertEqual(resolve_label("Critical", provider="gitlab"), "type::bug")
        self.assertEqual(resolve_label("Important", provider="gitlab"), "type::bug")
        self.assertEqual(resolve_label("Suggestion", provider="gitlab"), "type::feature")
        self.assertEqual(resolve_label("Nitpick", provider="gitlab"), "type::feature")

        # Explicit label override
        self.assertEqual(resolve_label("Critical", provider="github", explicit_label="custom-label"), "custom-label")

    def test_cli_dry_run_compliant_and_malformed(self):
        """Verify CLI dry run exit codes on compliant and malformed files."""
        with tempfile.NamedTemporaryFile("w", suffix=".md", delete=False) as tf_good:
            tf_good.write(SAMPLE_COMPLIANT_CRITICAL)
            good_path = tf_good.name

        with tempfile.NamedTemporaryFile("w", suffix=".md", delete=False) as tf_bad:
            tf_bad.write("Malformed dossier content")
            bad_path = tf_bad.name

        try:
            # Good file dry-run
            cmd_good = [
                sys.executable,
                os.path.join(REPO_ROOT, "scripts", "file_defect.py"),
                "--body-file", good_path,
                "--title", "[AUDIT] test: pass",
                "--dry-run",
            ]
            res_good = subprocess.run(cmd_good, capture_output=True, text=True)
            self.assertEqual(res_good.returncode, 0, f"Expected 0, got {res_good.returncode}. stderr: {res_good.stderr}")
            self.assertIn("validation PASSED", res_good.stdout)

            # Bad file dry-run
            cmd_bad = [
                sys.executable,
                os.path.join(REPO_ROOT, "scripts", "file_defect.py"),
                "--body-file", bad_path,
                "--title", "[AUDIT] test: fail",
                "--dry-run",
            ]
            res_bad = subprocess.run(cmd_bad, capture_output=True, text=True)
            self.assertEqual(res_bad.returncode, 1)
            self.assertIn("Defect dossier validation FAILED", res_bad.stderr)
        finally:
            if os.path.exists(good_path):
                os.remove(good_path)
            if os.path.exists(bad_path):
                os.remove(bad_path)


if __name__ == "__main__":
    unittest.main()
