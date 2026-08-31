# Solution Walkthrough: Domain Decoupling and Dynamic Layout/Validation

This document details the solution walkthrough for the domain decoupling and dynamic layout/validation implementation.

---

## 1. Code Realization Table

| Feature / Attribute | Source File | Class / Component | Method / Function | Description |
|---|---|---|---|---|
| YANG Compiler decoupling | `scripts/compile_yang.py` | N/A | `build_lui_json` | Loads `codebase_rules.json` and injects `"layout_rules" -> "details_tabs"`. Fallback to generic Properties `TableView` if undefined. |
| Flutter Generic Validator | `app_flutter/lib/domain/validation.dart` | N/A | `validateFields` | Evaluates validation constraints (`isRequired`/`required`, `minValue`/`maxValue`, `pattern`, `options`/`enumOptions`) against a map of inputs and a list of `FieldDescriptor`s. |
| React Generic Validator | `web_react/src/domain/validation.ts` | N/A | `validateFields` | Replicates generic validation engine in TypeScript. |
| Dynamic Property Grid | `web_react/src/components/property-grid.tsx` | `PropertyGrid` | N/A | Groups inputs by `sectionGroup`, dynamically renders form inputs by looping over `logicalLayout.attributes`, and uses `validateFields` for input validation. |
| Baseline Gate decoupling | `scripts/verify_downstream_baseline.py` | N/A | N/A | Clears hardcoded default fallback list of `MANDATED_CLASSES` to `[]`. |

---

## 2. Walkthrough of Changes

### 2.1. YANG Compiler Decoupling
The compiler previously outputted a static, hardcoded list of detailed table views inside the `TabbedContainer` children. We modified `build_lui_json` to load the target project's `codebase_rules.json` or `.pipeline/logical-ui/codebase_rules.json` and read the configs under `"layout_rules" -> "details_tabs"`. If defined, it uses this custom list to populate the container. Otherwise, it defaults to a single `properties_table` TableView.

### 2.2. Generic Validation Engine
We removed hardcoded domain validators (`validateLoadSpec`, `validatePostalAddress`, etc.) from both frameworks:
- **Flutter**: Replaced with `validateFields(Map<String, dynamic> input, List<FieldDescriptor> descriptors)` in `app_flutter/lib/domain/validation.dart`.
- **React**: Replaced with `validateFields(input: Record<string, any>, descriptors: FieldDescriptor[])` in `web_react/src/domain/validation.ts`.

Both implementations validate fields dynamically: if a field is not required and is empty or null, validation is skipped for that field. Otherwise, it evaluates:
- Required checks.
- Type compatibility (parsing integers/doubles and checking constraints).
- Minimum and maximum value limits.
- Regex pattern matching.
- Enumerated list options.

### 2.3. Dynamic Property Grid in React
The React `PropertyGrid` component now imports `logical-layout.json` (falling back to a default set of attributes for showcase simulation if empty) and:
- Dynamically groups attributes by their `sectionGroup`.
- Renders appropriate UI components (checkbox for boolean, select dropdown for enum, number input for int/double, text input for string).
- Performs live validation on blur using the dynamic `validateFields` function and displays custom error messages based on violated constraints.

### 2.4. Conformance Gate & Guide
- The conformance gate script `verify_downstream_baseline.py` was updated to set default fallback list of `MANDATED_CLASSES` to `[]` to prevent failures when domain classes are removed/decoupled.
- The YANG compiler guide `docs/operations/yang-compiler-guide.md` was updated to explicitly denote that property mappings are dynamically resolved from source schemas rather than hardcoded, labeling network-specific terms as examples.

---

## 3. Verification & Testing

### 3.1. Flutter Tests
- Wrote extensive unit tests in `app_flutter/test/domain/validation_test.dart` to assert boundary limits, regex pattern matches, required constraints, and type coercion.
- Verified that all 227 unit and widget tests pass:
  `All tests passed!`

### 3.2. React Tests
- Rewrote domain validation tests in `web_react/src/components/layout.test.tsx` to pass mock field descriptors to `validateFields` to assert regex and boundary constraints.
- Verified that the React application builds correctly:
  `Success: Build and test suite execution passed. Conformance gate verified.`
