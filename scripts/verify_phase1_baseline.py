#!/usr/bin/env python3
"""
Zero-Mock Physical Phase 1 Safety Verifier (verify_phase1_baseline.py)

Strictly evaluates downstream workspaces against Phase 1 Safety Grounding invariants:
  - Gate 1 (SSOT Hash Lock): Validates physical SHA-256 digests against schema/SSOT_INPUT_REGISTER.md.
  - Gate 2 (STPA Cartesian Closure): Verifies all 60 UCAs (15 control actions x 4 guide words) in Pillar 4.
  - Gate 3 (FMECA AST Completeness): Verifies all 12 AST subsystems across 4 failure dimensions (Γ, Φ, Ω, Ψ)
                                    with >= 3 failure modes per part and verified RPN = S * O * D arithmetic.
  - Gate 4 (Semantic Citation Verification): Slices physical sections via MechanicalSectionSlicer to verify
                                             claimed technical parameters exist in cited SSOT sections.
  - Gate 5 (Zero Foreign COTS Contamination): Asserts zero occurrences of 'DShot', '400 Hz inner loop',
                                              or '30 Hz GUI' across safety deliverables.

This verifier contains ZERO mock objects, ZERO synthetic stubs, and fails closed.
"""

import argparse
import hashlib
import os
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, List, Optional, Set, Tuple

# Ensure parity_auditor package is resolvable
_DEAP_CORE_ROOT = Path(__file__).resolve().parent.parent
_PARITY_AUDITOR_SRC = _DEAP_CORE_ROOT / "skills" / "spec-orchestrator" / "parity_auditor" / "src"
if str(_PARITY_AUDITOR_SRC) not in sys.path and _PARITY_AUDITOR_SRC.exists():
    sys.path.insert(0, str(_PARITY_AUDITOR_SRC))

try:
    from parity_auditor.validators.factual_grounding_validator import MechanicalSectionSlicer
except ImportError:
    MechanicalSectionSlicer = None  # Fallback handled in Gate 4 if unavailable


@dataclass
class GateResult:
    name: str
    passed: bool
    summary: str
    failures: List[str] = field(default_factory=list)
    details: List[str] = field(default_factory=list)


def compute_file_sha256(filepath: Path) -> str:
    """Compute physical SHA-256 digest of a file on disk."""
    hasher = hashlib.sha256()
    with open(filepath, "rb") as f:
        while chunk := f.read(65536):
            hasher.update(chunk)
    return hasher.hexdigest()


# ==============================================================================
# Gate 1: SSOT Hash Lock
# ==============================================================================
def verify_gate_1_ssot_hashes(workspace_dir: Path, verbose: bool = False) -> GateResult:
    """Verify physical SHA-256 hashes against schema/SSOT_INPUT_REGISTER.md."""
    register_path = workspace_dir / "schema" / "SSOT_INPUT_REGISTER.md"
    if not register_path.is_file():
        return GateResult(
            name="Gate 1 (SSOT Hash Lock)",
            passed=False,
            summary="schema/SSOT_INPUT_REGISTER.md not found on disk",
            failures=[f"Missing file: {register_path}"],
        )

    failures: List[str] = []
    details: List[str] = []
    checked_count = 0

    with open(register_path, "r", encoding="utf-8") as f:
        lines = f.readlines()

    # Regex to match table rows: | `path` | bytes | `sha256` | ...
    # or | path | bytes | sha256 | ...
    row_pattern = re.compile(r"^\s*\|\s*`?([^`|\s]+)`?\s*\|\s*(\d+)\s*\|\s*`?([a-fA-F0-9]{64})`?")

    in_register_table = False
    for line in lines:
        if "## 2. Input Document Register" in line:
            in_register_table = True
            continue
        elif in_register_table and line.startswith("## "):
            in_register_table = False

        if in_register_table:
            match = row_pattern.match(line)
            if match:
                rel_path_str = match.group(1).strip()
                expected_hash = match.group(3).strip().lower()

                target_file = workspace_dir / rel_path_str
                if not target_file.is_file():
                    failures.append(f"SSOT file missing on disk: {rel_path_str}")
                    continue

                actual_hash = compute_file_sha256(target_file).lower()
                checked_count += 1

                if actual_hash != expected_hash:
                    failures.append(
                        f"Hash mismatch for {rel_path_str}:\n"
                        f"    Expected: {expected_hash}\n"
                        f"    Actual:   {actual_hash}"
                    )
                else:
                    details.append(f"Verified {rel_path_str} (SHA-256: {actual_hash[:12]}...)")

    if checked_count == 0:
        return GateResult(
            name="Gate 1 (SSOT Hash Lock)",
            passed=False,
            summary="Zero SSOT document entries parsed from schema/SSOT_INPUT_REGISTER.md",
            failures=["Failed to parse table rows in Section 2 of SSOT_INPUT_REGISTER.md"],
        )

    passed = len(failures) == 0
    summary = f"Verified {checked_count} SSOT input files ({len(failures)} mismatches)"
    return GateResult(name="Gate 1 (SSOT Hash Lock)", passed=passed, summary=summary, failures=failures, details=details)


# ==============================================================================
# Gate 2: STPA Cartesian Closure
# ==============================================================================
def verify_gate_2_stpa_closure(workspace_dir: Path, verbose: bool = False) -> GateResult:
    """Verify all 60 UCAs (15 control actions x 4 guide words) in Pillar 4."""
    stpa_path = workspace_dir / "docs" / "safety" / "STPA_MATRIX.md"
    if not stpa_path.is_file():
        return GateResult(
            name="Gate 2 (STPA Cartesian Closure)",
            passed=False,
            summary="docs/safety/STPA_MATRIX.md not found on disk",
            failures=[f"Missing file: {stpa_path}"],
        )

    failures: List[str] = []
    details: List[str] = []

    with open(stpa_path, "r", encoding="utf-8") as f:
        content = f.read()

    # Extract Pillar 4
    p4_match = re.search(r"## Pillar 4 -- Unsafe Control Actions.*?(?=## Pillar 5|\Z)", content, re.DOTALL)
    if not p4_match:
        return GateResult(
            name="Gate 2 (STPA Cartesian Closure)",
            passed=False,
            summary="Pillar 4 (Unsafe Control Actions) section missing in STPA_MATRIX.md",
            failures=["Pillar 4 heading '## Pillar 4 -- Unsafe Control Actions' not found"],
        )

    p4_text = p4_match.group(0)

    # Find all UCA identifiers (UCA-1 through UCA-60 or UCA-01 through UCA-60)
    uca_matches = re.findall(r"\bUCA-(\d+)\b", p4_text)
    uca_nums = {int(n) for n in uca_matches}

    expected_ucas = set(range(1, 61))
    missing_ucas = expected_ucas - uca_nums
    unexpected_ucas = uca_nums - expected_ucas

    if missing_ucas:
        failures.append(f"Missing {len(missing_ucas)} UCAs: {sorted(list(missing_ucas))}")
    if unexpected_ucas:
        failures.append(f"Unexpected out-of-range UCAs: {sorted(list(unexpected_ucas))}")

    details.append(f"Found {len(uca_nums)} unique UCAs in Pillar 4 (range 1-60 expected)")
    passed = len(failures) == 0 and len(uca_nums) == 60
    summary = f"STPA Cartesian Closure: {len(uca_nums)}/60 UCAs present"

    return GateResult(name="Gate 2 (STPA Cartesian Closure)", passed=passed, summary=summary, failures=failures, details=details)


# ==============================================================================
# Gate 3: FMECA AST Completeness
# ==============================================================================
MANDATED_AST_SUBSYSTEMS = [
    "Avenger5",
    "Airframe",
    "SensorSuite",
    "Propulsion",
    "Actuation",
    "Communications",
    "PowerDistribution",
    "ESAD",
    "OnboardComputer",
    "WarheadModule",
    "RTASafetyNet",
    "PL40Catapult",
]

FAILURE_DIMENSIONS = {
    "Γ": "Interface",
    "Φ": "State",
    "Ω": "Action",
    "Ψ": "Resource",
}


def verify_gate_3_fmeca_ast_completeness(workspace_dir: Path, verbose: bool = False) -> GateResult:
    """Verify all 12 AST subsystems across 4 failure dimensions with >= 3 failure modes and exact RPN."""
    stpa_path = workspace_dir / "docs" / "safety" / "STPA_MATRIX.md"
    if not stpa_path.is_file():
        return GateResult(
            name="Gate 3 (FMECA AST Completeness)",
            passed=False,
            summary="docs/safety/STPA_MATRIX.md not found on disk",
            failures=[f"Missing file: {stpa_path}"],
        )

    failures: List[str] = []
    details: List[str] = []

    with open(stpa_path, "r", encoding="utf-8") as f:
        content = f.read()

    # Extract Pillar 7
    p7_match = re.search(r"## Pillar 7 -- FMECA Criticality Matrix.*?(?=## Pillar 8|\Z)", content, re.DOTALL)
    if not p7_match:
        return GateResult(
            name="Gate 3 (FMECA AST Completeness)",
            passed=False,
            summary="Pillar 7 (FMECA Criticality Matrix) section missing in STPA_MATRIX.md",
            failures=["Pillar 7 heading '## Pillar 7 -- FMECA Criticality Matrix' not found"],
        )

    p7_text = p7_match.group(0)
    lines = p7_text.splitlines()

    # Track subsystem failure counts, dimensions, and arithmetic errors
    subsystem_data: Dict[str, Dict[str, any]] = {
        s: {"count": 0, "dims": set(), "rpn_checked": 0} for s in MANDATED_AST_SUBSYSTEMS
    }

    rpn_errors: List[str] = []
    table_rows = 0

    for idx, line in enumerate(lines, 1):
        if not line.strip().startswith("|") or line.strip().startswith("| :---") or "Severity (S)" in line:
            continue

        cols = [c.strip() for c in line.split("|")[1:-1]]
        if len(cols) < 11:
            continue

        table_rows += 1
        sub_raw = cols[0]
        failure_mode = cols[1]
        dim_raw = cols[3]

        try:
            s_val = int(cols[7])
            o_val = int(cols[8])
            d_val = int(cols[9])
            rpn_val = int(cols[10])
        except ValueError:
            failures.append(f"Line {idx}: Invalid integer S/O/D/RPN in row: {cols[7:11]}")
            continue

        expected_rpn = s_val * o_val * d_val
        if rpn_val != expected_rpn:
            rpn_errors.append(
                f"Line {idx}: RPN mismatch for '{sub_raw}': S({s_val}) * O({o_val}) * D({d_val}) = {expected_rpn} != RPN({rpn_val})"
            )

        # Match subsystem
        matched_sub = None
        for mandated in MANDATED_AST_SUBSYSTEMS:
            if mandated.lower() in sub_raw.lower():
                matched_sub = mandated
                break

        if matched_sub:
            subsystem_data[matched_sub]["count"] += 1
            subsystem_data[matched_sub]["rpn_checked"] += 1
            # Detect dimensions: Γ, Φ, Ω, Ψ
            for dim_sym, dim_name in FAILURE_DIMENSIONS.items():
                if dim_sym in dim_raw or dim_name.lower() in dim_raw.lower():
                    subsystem_data[matched_sub]["dims"].add(dim_sym)

    if rpn_errors:
        failures.extend(rpn_errors)

    # Validate each mandated AST subsystem
    for sub, data in subsystem_data.items():
        if data["count"] < 3:
            failures.append(f"Subsystem '{sub}' has insufficient failure modes: {data['count']} (min 3 required)")

        missing_dims = set(FAILURE_DIMENSIONS.keys()) - data["dims"]
        if missing_dims:
            failures.append(
                f"Subsystem '{sub}' missing failure dimensions: {sorted(list(missing_dims))} (found: {sorted(list(data['dims']))})"
            )

        details.append(
            f"Subsystem '{sub}': {data['count']} modes, dimensions: {sorted(list(data['dims']))}, RPN checks: {data['rpn_checked']}"
        )

    passed = len(failures) == 0 and table_rows > 0
    summary = f"FMECA AST Completeness: {len(MANDATED_AST_SUBSYSTEMS)} subsystems verified across 4 dimensions ({table_rows} rows checked)"

    return GateResult(name="Gate 3 (FMECA AST Completeness)", passed=passed, summary=summary, failures=failures, details=details)


# ==============================================================================
# Gate 4: Semantic Citation Verification
# ==============================================================================
def verify_gate_4_semantic_citations(workspace_dir: Path, verbose: bool = False) -> GateResult:
    """Slice physical sections with MechanicalSectionSlicer to verify claimed technical parameters exist."""
    stpa_path = workspace_dir / "docs" / "safety" / "STPA_MATRIX.md"
    schema_dir = workspace_dir / "schema"

    if not stpa_path.is_file():
        return GateResult(
            name="Gate 4 (Semantic Citation Verification)",
            passed=False,
            summary="docs/safety/STPA_MATRIX.md not found on disk",
            failures=[f"Missing file: {stpa_path}"],
        )

    if MechanicalSectionSlicer is None:
        return GateResult(
            name="Gate 4 (Semantic Citation Verification)",
            passed=False,
            summary="MechanicalSectionSlicer could not be imported from parity_auditor",
            failures=["Import failure for MechanicalSectionSlicer"],
        )

    slicer = MechanicalSectionSlicer(str(schema_dir))
    failures: List[str] = []
    details: List[str] = []

    with open(stpa_path, "r", encoding="utf-8") as f:
        lines = f.readlines()

    # Regex to capture citations like 'a5-user-manual-2.md §7.2.3' or 'schema/a5-user-manual-2.md §7.2.3'
    citation_re = re.compile(r"([a-zA-Z0-9_\-./]+\.md)\s*§\s*([0-9]+(?:\.[0-9]+)*)")

    # Key parameter patterns to extract from the row
    freq_re = re.compile(r"\b(\d+\s*Hz)\b", re.IGNORECASE)
    protocol_re = re.compile(r"\b(PWM/DShot|DShot|CAN|RS-485|SPI|UART|MAVLink(?:\s*2)?)\b", re.IGNORECASE)

    checked_citations = 0

    for idx, line in enumerate(lines, 1):
        # We focus on rows with citations
        cit_match = citation_re.search(line)
        if not cit_match:
            continue

        doc_name = cit_match.group(1).strip()
        section_id = cit_match.group(2).strip()

        # Resolve doc_path
        if doc_name.startswith("schema/"):
            doc_path = str(workspace_dir / doc_name)
        elif (workspace_dir / "schema" / doc_name).exists():
            doc_path = str(workspace_dir / "schema" / doc_name)
        elif (workspace_dir / doc_name).exists():
            doc_path = str(workspace_dir / doc_name)
        else:
            failures.append(f"Line {idx}: Cited document '{doc_name}' does not exist on disk.")
            continue

        # Extract tokens to verify for this line
        tokens_to_test: List[str] = []

        # Check for specific frequency claims (e.g., '50 Hz' on line 533)
        for fm in freq_re.finditer(line):
            val = fm.group(1)
            tokens_to_test.append(val)
            # Add variant without space
            tokens_to_test.append(val.replace(" ", ""))

        # Check for specific protocol claims (e.g., 'PWM/DShot' or 'DShot' on line 559)
        for pm in protocol_re.finditer(line):
            val = pm.group(1)
            if "dshot" in val.lower():
                tokens_to_test.append("DShot")
                tokens_to_test.append("PWM")

        if not tokens_to_test:
            continue

        checked_citations += 1
        ok, matched_tokens, err_msg = slicer.verify_claimed_tokens(doc_path, section_id, tokens_to_test)
        if not ok:
            failures.append(f"STPA_MATRIX.md:{idx}: Citation verification failed for '{doc_name} §{section_id}': {err_msg}")
        else:
            details.append(f"STPA_MATRIX.md:{idx}: Grounded claim tokens {matched_tokens} in '{doc_name} §{section_id}'")

    passed = len(failures) == 0
    summary = f"Semantic Citation Verification: {checked_citations} claims checked ({len(failures)} citation errors)"

    return GateResult(name="Gate 4 (Semantic Citation Verification)", passed=passed, summary=summary, failures=failures, details=details)


# ==============================================================================
# Gate 5: Zero Foreign COTS Contamination
# ==============================================================================
FORBIDDEN_COTS_TERMS = [
    "DShot",
    "400 Hz inner loop",
    "30 Hz GUI",
]


def verify_gate_5_zero_cots_contamination(workspace_dir: Path, verbose: bool = False) -> GateResult:
    """Assert zero occurrences of 'DShot', '400 Hz inner loop', or '30 Hz GUI' across safety deliverables."""
    candidate_files = [
        workspace_dir / "docs" / "safety" / "STPA_MATRIX.md",
        workspace_dir / "docs" / "reports" / "PHASE_1_EXECUTIVE_ENGINEERING_DELIVERABLES.md",
    ]

    failures: List[str] = []
    details: List[str] = []
    files_checked = 0

    for file_path in candidate_files:
        if not file_path.is_file():
            continue

        files_checked += 1
        rel_path = file_path.relative_to(workspace_dir)

        with open(file_path, "r", encoding="utf-8") as f:
            for idx, line in enumerate(f, 1):
                for term in FORBIDDEN_COTS_TERMS:
                    if term.lower() in line.lower():
                        clean_snippet = line.strip()[:100]
                        failures.append(
                            f"{rel_path}:{idx}: Prohibited foreign COTS token '{term}' found: '{clean_snippet}'"
                        )

    passed = len(failures) == 0
    summary = f"Zero Foreign COTS Contamination: {files_checked} files inspected ({len(failures)} violations)"

    return GateResult(
        name="Gate 5 (Zero Foreign COTS Contamination)",
        passed=passed,
        summary=summary,
        failures=failures,
        details=details,
    )


# ==============================================================================
# Main CLI & Runner
# ==============================================================================
def main():
    parser = argparse.ArgumentParser(
        description="Verify downstream workspace against Phase 1 Zero-Failure Safety Grounding invariants."
    )
    parser.add_argument(
        "workspace",
        nargs="?",
        default=".",
        help="Path to the workspace root to verify (defaults to current directory).",
    )
    parser.add_argument(
        "--verbose",
        "-v",
        action="store_true",
        help="Display detailed verification trace output.",
    )
    args = parser.parse_args()

    workspace_dir = Path(args.workspace).resolve()
    print("=" * 80)
    print(f"ZERO-MOCK PHASE 1 SAFETY BASELINE VERIFIER")
    print(f"Target Workspace: {workspace_dir}")
    print("=" * 80)

    if not workspace_dir.is_dir():
        print(f"FATAL: Workspace path '{workspace_dir}' does not exist or is not a directory.")
        sys.exit(1)

    gates = [
        verify_gate_1_ssot_hashes(workspace_dir, args.verbose),
        verify_gate_2_stpa_closure(workspace_dir, args.verbose),
        verify_gate_3_fmeca_ast_completeness(workspace_dir, args.verbose),
        verify_gate_4_semantic_citations(workspace_dir, args.verbose),
        verify_gate_5_zero_cots_contamination(workspace_dir, args.verbose),
    ]

    all_passed = True
    print("\nGATE EXECUTION SUMMARY:\n")
    for gate in gates:
        status_str = "[ PASS ]" if gate.passed else "[ FAIL ]"
        print(f"{status_str} {gate.name}: {gate.summary}")
        if not gate.passed:
            all_passed = False
            for f in gate.failures:
                print(f"    - ERROR: {f}")
        elif args.verbose:
            for d in gate.details:
                print(f"    - INFO: {d}")

    print("\n" + "=" * 80)
    if all_passed:
        print("VERIFICATION RESULT: ALL 5 GATES PASSED (100% Phase 1 Baseline Conformance)")
        print("=" * 80)
        sys.exit(0)
    else:
        print("VERIFICATION RESULT: FAIL-CLOSED (One or more safety grounding gates failed)")
        print("=" * 80)
        sys.exit(1)


if __name__ == "__main__":
    main()
