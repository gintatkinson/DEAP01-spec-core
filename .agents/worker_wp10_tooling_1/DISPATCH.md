# Dispatch Record

## 2026-09-27T08:14:15Z
Execute view_file on skills/debug-protocol/SKILL.md as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Target Workspace: /Users/perkunas/jail/DEAP01-spec-core
Working directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_wp10_tooling_1
Primary Commercial Toolchain Integration Context: MATLAB / Simulink / Stateflow / Embedded Coder
Parent Orchestrator ID: d0acf8cb-0b5f-428f-bbe8-5c9e60d7dedc

Task: Execute WP-10 from approved implementation_plan.md: Tooling Fix for Check 23 Balanced Brace AST Extraction & Component Scoping in Factual Grounding.

Problem Description:
Independent Victory Auditor 6 found that running baseline verification on customer workspace uav-009:
`python3 scripts/verify_downstream_baseline.py /Users/perkunas/jail/uav-009`
Terminated with exit code 1 at Check 23, reporting 421 factual grounding errors (e.g., `04_SYSTEM_ARCHITECTURE.md:413: Fabricated numeric quantity '12.0 W' exceeds schema ground truth limit (2.0w)...`).

Root Cause Analysis:
In `skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/factual_grounding_validator.py:1642`:
The method `_extract_from_sysml(self, text: str, gt: SchemaGroundTruth)` used:
`part_pattern = re.compile(r'\bpart\s+(?:def\s+)?([a-zA-Z0-9_]+)\s*\{([^}]*)\}', re.DOTALL)`
The non-nesting regex `[^}]*` terminates at the FIRST closing brace `}` inside nested `part def` or nested blocks (for example, inside `Avenger5AirVehicle`, `FlightController` ends at line 180 with `}`).
Consequently, `Avenger5AirVehicle` is prematurely cut off at line 180!
All subsequent subsystem declarations between lines 180 and 430—such as `radioOutputPowerW: Real = 2.0;`—fall outside any matched `part def` span and are dumped into `top_level_text` with `owner=None`!
As a global un-scoped metric with unit `W`, `2.0 W` was matched against every component in the aircraft mentioning "power", falsely flagging 421 legitimate physical attributes across 75 documents.

Requirements for WP-10:
1. Replace the naive `[^}]*` regex in `_extract_from_sysml` with a balanced-brace block scanner that correctly extracts the full body of each `part def` and `item def` block, respecting nested braces (following the canonical pattern used in `schema_router.py:608-615`).
2. Recursively or hierarchically extract nested `part def` blocks so child subsystem attributes receive their proper `owner` (e.g., `owner=pname_norm`), preventing inner attributes from spilling into top-level un-scoped attributes with `owner=None`.
3. In `_evaluate_numeric_claims_in_line`:
   Ensure that when a metric has a specific owner or specific property tokens (e.g. `radioOutputPowerW` having meaningful tokens `['radio', 'output', 'power']`), it is NEVER matched against an unrelated component (e.g. `Gimbal`, `Battery`, `Motor`) simply because the generic token `power` appears on the line. When `metric.owner` is not present on the line or in the heading, require strong token matching before binding the metric as a candidate.
4. Adhere strictly to Failure Mode 11: Under NO circumstances should customer SysML models (`/Users/perkunas/jail/uav-009/schema/avenger5_system.sysml`) or customer specifications in `uav-009/docs/` be modified or deleted. The fix is strictly in `factual_grounding_validator.py`.

Testing & Verification:
- Run `python3 -m unittest tests/test_check23_factual_grounding_gate.py` to verify unit tests pass.
- Test Check 23 on `uav-009`:
  `python3 -c "import sys; sys.path.insert(0, 'scripts'); from verify_downstream_baseline import check_factual_grounding; check_factual_grounding('/Users/perkunas/jail/uav-009')"`
  Verify it passes with exit code 0 and reports `Success: Check 23 verified`.
- Report all changes, diffs, and verification results in `.agents/worker_wp10_tooling_1/handoff.md`.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

PROCEED

## 2026-09-27T08:50:30Z
**Context**: WP-10 Tooling Fix for Check 23 Balanced Brace AST Extraction & Component Scoping
**Content**: Heartbeat check-in: how is progress on integrating the balanced-brace parser into factual_grounding_validator.py and running Check 23 on uav-009?
**Action**: Please report your current status or completion handoff.

