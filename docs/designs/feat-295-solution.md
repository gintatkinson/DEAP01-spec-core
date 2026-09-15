---
issue_id: 295
title: "Solution Walkthrough: Raw Document Schema Ingestion"
---

# Solution Walkthrough: Raw Document Schema Ingestion (Issue #295)

## Overview
This document summarizes the changes implemented to address Work Package 4.3 (Issue #295). The system now explicitly recognizes raw document formats and throws an actionable `RuntimeError` stating that AST translation is required, rather than attempting to parse unstructured text and failing with generic schema errors.

## Code Realization Table
| Feature / Attribute | Target Platform Implementation |
|---------------------|--------------------------------|
| Raw Extension Registry | `skills/spec-orchestrator/scripts/sysmlv2_ingest.py` |
| Format Detection Logic | `detect_format` in `skills/spec-orchestrator/scripts/sysmlv2_ingest.py` |
| Exception Raising | `ingest_schema` in `skills/spec-orchestrator/scripts/sysmlv2_ingest.py` |
| Unit Tests | `tests/test_sysmlv2_ingest.py`, `tests/test_sysmlv2_ingest_format_detection.py` |

## Changes Made
1. **`sysmlv2_ingest.py`**: Added `RAW_EXTENSIONS = {".md", ".pdf", ".txt", ".doc", ".docx"}`. Updated `detect_format` to return `"raw"` for these extensions. Updated `ingest_schema` to raise a `RuntimeError` when `fmt == "raw"`.
2. **`test_sysmlv2_ingest.py`**: Added `test_raw_document_ingestion_reporting` to ensure that raw documents fail predictably with a `RuntimeError` regarding AST translation.
3. **`test_sysmlv2_ingest_format_detection.py`**: Refactored existing format detection tests to expect `RuntimeError` for raw files instead of the legacy `ValueError`.

## Verification Results
- Executed Test-Driven Development (RED-GREEN-REFACTOR).
- Ran all unit tests using `python3 -m unittest discover -s tests -p "test_sysml*.py"`.
- Results: 35 tests passed successfully, confirming both format detection logic and AST filtering functionality.
