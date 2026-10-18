# YANG Compiler: User Requirements & Acceptance Tests

| Attribute | Specification Detail |
| :--- | :--- |
| **Version** | 1.0.0 |
| **Date** | 2026-08-31 |

## User Story

As a domain engineer, I want to take a YANG data model file and automatically generate a working UI layout so that I don't have to manually edit `logical-layout.json` or write any Dart/Flutter code.

## Acceptance Criteria

### AC1: Basic Compilation
**Given** a valid YANG file with `container`, `list`, `leaf`, and `enumeration` constructs
**When** I run `./target/release/ingest-sysml --schema mymodel.yang --format yang --out schema/mymodel.sysml`
**Then** the output file contains a valid SysML v2 model with:
- A tree hierarchy reflecting the YANG container/list nesting
- Attribute definitions for every leaf with correct types
- Enum options extracted from YANG enumeration statements
- Range constraints from YANG range statements

### AC2: Type Mapping
**Given** a YANG leaf with type `uint32` and a range constraint
**When** compiled
**Then** the output attribute has `type: "int"` and `minValue`/`maxValue` matching the YANG range

**Given** a YANG leaf with type `enumeration`
**When** compiled
**Then** the output attribute has `type: "enum"` and an `options` array with all enum values

### AC3: Hierarchy Preservation
**Given** a YANG model with nested containers (e.g., `system > interfaces > interface`)
**When** compiled
**Then** the output `hierarchy` preserves the nesting relationship as parent-child tree nodes

### AC4: Mandatory Fields
**Given** a YANG leaf with `mandatory true`
**When** compiled
**Then** the output attribute has `isRequired: true`

### AC5: Error Handling
**Given** an invalid YANG file (syntax error, missing file)
**When** compiled
**Then** the script exits with a non-zero exit code and prints a descriptive error message

### AC6: No Manual Editing
**Given** a generated `logical-layout.json`
**When** loaded by the Flutter app
**Then** the app renders the correct tree hierarchy, property fields with correct types, and validation constraints without any manual editing of the JSON

## Verification Commands

```bash
# AC1: Basic compilation
./target/release/ingest-sysml \
  --schema /tmp/test-model.yang \
  --format yang \
  --out /tmp/test-output.sysml

# AC5: Error on invalid file
./target/release/ingest-sysml \
  --schema /tmp/nonexistent.yang \
  --format yang \
  --out /tmp/test-output.sysml 2>&1 && echo 'FAIL: Should have errored' || echo 'AC5 PASS: Correctly errored'

# AC6: Generated file matches expected structure
./target/release/verify-baseline . --no-domain
```
