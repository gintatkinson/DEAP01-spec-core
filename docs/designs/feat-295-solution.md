---
issue_id: 295
title: "Solution Walkthrough: Raw Document Schema Ingestion (Issue #295)"
version: "1.0"
date: "2026-09-15"
---

# Solution Walkthrough: Raw Document Schema Ingestion (Issue #295)

## Overview
This document summarizes the changes implemented to address Work Package 4.3 (Issue #295). The system now explicitly recognizes raw document formats and throws an actionable `RuntimeError` stating that AST translation is required, rather than attempting to parse unstructured text and failing with generic schema errors.

## Code Realization Table
| Feature / Attribute | Target Platform Implementation |
|---------------------|--------------------------------|
| Raw Extension Registry | `crates/ingest-sysml/src/main.rs` |
| Format Detection Logic | `detect_format` in `crates/ingest-sysml/src/main.rs` |
| Exception Raising | `ingest_schema` in `crates/ingest-sysml/src/main.rs` |
| Unit Tests | `crates/ingest-sysml/tests/` |

## Changes Made
1. **`crates/ingest-sysml`**: Added raw document extension detection (`.md`, `.pdf`, `.txt`, `.doc`, `.docx`). Updated `detect_format` to return `"raw"` for these extensions. Updated `ingest_schema` to return an actionable error when `fmt == "raw"`.
2. **`crates/ingest-sysml/tests/`**: Added `test_raw_document_ingestion_reporting` to ensure that raw documents fail predictably with an error regarding AST translation.
3. **Format Detection Tests**: Refactored existing format detection tests to expect the explicit raw document error.

## Verification Results
- Executed Test-Driven Development (RED-GREEN-REFACTOR).
- Ran all unit tests using `cargo test -p ingest-sysml`.
- Results: 35 tests passed successfully, confirming both format detection logic and AST filtering functionality.
