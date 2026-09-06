"""
Safety Integrity Quality Gate & SORA OSO-01..24 Completeness Verification Suite.
/// Realises: [SafetyIntegrityQualityGate, SORACompleteness, ASTM_F3269_RTA]
"""
import os
import sys
import re
import tempfile
import pathlib
import pytest

# Ensure scripts directory is in sys.path
repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if repo_root not in sys.path:
    sys.path.insert(0, repo_root)

from scripts.verify_downstream_baseline import (
    count_fmeca_rows,
    parse_fmeca_table,
    check_uca_categories,
    check_sora_osos,
    validate_safety_matrix_content,
    check_safety_integrity_and_sora_completeness,
)

FIXTURES_DIR = pathlib.Path(__file__).resolve().parent / "fixtures" / "safety"


def read_fixture(name: str) -> str:
    """Load a live safety specification fixture from tests/fixtures/safety/."""
    fixture_path = FIXTURES_DIR / name
    assert fixture_path.is_file(), f"Safety fixture missing: {fixture_path}"
    return fixture_path.read_text(encoding="utf-8")


def write_safety_matrix(tmpdir: str, content: str) -> str:
    """Write safety specification content into a downstream docs/safety/STPA_MATRIX.md."""
    safety_dir = os.path.join(tmpdir, "docs", "safety")
    os.makedirs(safety_dir, exist_ok=True)
    stpa_file = os.path.join(safety_dir, "STPA_MATRIX.md")
    with open(stpa_file, "w", encoding="utf-8") as f:
        f.write(content)
    return stpa_file


def _reduce_fmeca_rows(content: str, keep: int) -> str:
    """Structurally truncate the FMECA data rows of a fixture, keeping only the first `keep`."""
    retained = keep
    out = []
    for line in content.splitlines():
        if line.strip().startswith("| FM-"):
            if retained > 0:
                out.append(line)
                retained -= 1
        else:
            out.append(line)
    assert retained == 0, f"Fixture FMECA table has fewer than {keep} data rows."
    return "\n".join(out)


def test_upstream_safety_landing_zone_clean():
    """Verify that upstream distribution templates enforce clean docs/safety/ landing zone."""
    if os.path.isdir(os.path.join(repo_root, ".pipeline", "upstream")):
        safety_dir = os.path.join(repo_root, "docs", "safety")
        if os.path.isdir(safety_dir):
            allowed = {".gitkeep", "README.md"}
            for f in os.listdir(safety_dir):
                assert f in allowed, f"Upstream template contains non-template file in docs/safety/: {f}"


def test_upstream_safety_landing_zone_dirty_fails():
    """Verify check_safety_integrity_and_sora_completeness rejects dirty upstream safety landing zones."""
    with tempfile.TemporaryDirectory() as tmpdir:
        os.makedirs(os.path.join(tmpdir, ".pipeline", "upstream"), exist_ok=True)
        os.makedirs(os.path.join(tmpdir, "docs", "safety"), exist_ok=True)

        # Write allowed README.md
        with open(os.path.join(tmpdir, "docs", "safety", "README.md"), "w") as f:
            f.write("# Safety Directory\n")

        # Write concrete spec file (violation)
        with open(os.path.join(tmpdir, "docs", "safety", "STPA_MATRIX.md"), "w") as f:
            f.write("# Concrete STPA Matrix\n")

        with pytest.raises(SystemExit) as exc_info:
            check_safety_integrity_and_sora_completeness(tmpdir)
        assert exc_info.value.code == 1


def test_downstream_8_pillar_passing():
    """Verify that the complete live fixture passes validation with zero errors."""
    valid_content = read_fixture("complete_stpa_matrix.md")
    errors = validate_safety_matrix_content(valid_content)
    assert not errors, f"Expected 0 errors for complete 8-pillar STPA matrix fixture, got:\n{errors}"


def test_sora_oso_01_to_24_validation():
    """Verify all 24 SORA OSOs are validated against the live fixtures."""
    complete_content = read_fixture("complete_stpa_matrix.md")
    assert check_sora_osos(complete_content) == []

    incomplete_content = read_fixture("incomplete_osos.md")
    assert check_sora_osos(incomplete_content) == ["OSO-23", "OSO-24"]

    errors = validate_safety_matrix_content(incomplete_content)
    assert any("OSO-23" in err and "OSO-24" in err for err in errors), f"Expected missing OSOs error, got:\n{errors}"


def test_fmeca_table_parser_and_multi_mode_distribution():
    """Verify parse_fmeca_table accurately extracts components, modes, RPN, and Basis classifications."""
    complete_content = read_fixture("complete_stpa_matrix.md")
    parsed = parse_fmeca_table(complete_content)

    assert parsed["total_rows"] == 18
    assert parsed["has_rpn"] is True
    assert len(parsed["components"]) == 6
    for comp, modes in parsed["components"].items():
        assert len(modes) >= 3, f"Expected component {comp} to have at least 3 failure modes, got {len(modes)}"
    assert parsed["basis_counts"]["SSOT"] == 12
    assert parsed["basis_counts"]["Derived"] == 6
    assert len(parsed["failure_modes"]) == 18

    errors = validate_safety_matrix_content(complete_content)
    assert errors == []


def test_fmeca_missing_basis_rejected():
    """Verify FMECA matrix missing explicit Derivation Basis (SSOT / Derived) is rejected."""
    complete_content = read_fixture("complete_stpa_matrix.md")
    # Strip Basis column
    lines = []
    for line in complete_content.splitlines():
        if line.strip().startswith("|") and line.strip().endswith("|"):
            cells = [c.strip() for c in line.strip().split("|")[1:-1]]
            if len(cells) > 1 and ("Basis" in cells[-1] or cells[-1] in ("SSOT", "Derived")):
                cells = cells[:-1]
                lines.append("| " + " | ".join(cells) + " |")
            elif re.match(r"^\|(?:\s*:?-+:?\s*\|)+$", line.strip()):
                parts = line.strip().split("|")[1:-1]
                lines.append("| " + " | ".join(parts[:-1]) + " |")
            else:
                lines.append(line)
        else:
            lines.append(line)
    invalid_no_basis = "\n".join(lines)

    parsed = parse_fmeca_table(invalid_no_basis)
    assert parsed["basis_counts"]["SSOT"] == 0
    assert parsed["basis_counts"]["Derived"] == 0

    errors = validate_safety_matrix_content(invalid_no_basis)
    assert any("Pillar 7 violation: FMECA Criticality Matrix missing explicit Derivation Basis" in err for err in errors), (
        f"Expected missing Basis classification error, got:\n{errors}"
    )


def test_fmeca_missing_universal_failure_dimension_rejected():
    """Verify FMECA matrix missing a universal failure dimension is rejected."""
    complete_content = read_fixture("complete_stpa_matrix.md")
    # Replace all failure modes with Interface-only modes
    lines = []
    for line in complete_content.splitlines():
        if line.strip().startswith("| FM-"):
            cells = [c.strip() for c in line.strip().split("|")[1:-1]]
            cells[2] = f"Bus Protocol Timeout {cells[0]}"
            lines.append("| " + " | ".join(cells) + " |")
        else:
            lines.append(line)
    single_dim_content = "\n".join(lines)

    errors = validate_safety_matrix_content(single_dim_content)
    assert any("Pillar 7 violation: FMECA table missing coverage for universal failure dimension(s)" in err for err in errors), (
        f"Expected universal failure dimension error, got:\n{errors}"
    )


def test_fmeca_row_count_validation():
    """Verify FMECA matrix requires non-empty rows."""
    complete_content = read_fixture("complete_stpa_matrix.md")
    assert count_fmeca_rows(complete_content) == 18
    assert validate_safety_matrix_content(complete_content) == []

    # Structurally reduced to 0 rows must be rejected
    lines = []
    for line in complete_content.splitlines():
        if line.strip().startswith("| FM-"):
            continue
        lines.append(line)
    empty_content = "\n".join(lines)
    assert count_fmeca_rows(empty_content) == 0
    errors = validate_safety_matrix_content(empty_content)
    assert any("FMECA Criticality Matrix contains 0 rows" in err for err in errors)


def test_uca_failure_mode_categories():
    """Verify all 4 STPA UCA failure mode categories are required."""
    all_cats_text = (
        "1. Not providing causes hazard\n"
        "2. Providing causes hazard\n"
        "3. Providing too early, too late, or out of order\n"
        "4. Stopped too soon or applied too long"
    )
    assert check_uca_categories(all_cats_text) == []

    # Missing "Not providing"
    no_omission = (
        "2. Providing causes hazard\n"
        "3. Providing too early, too late, or out of order\n"
        "4. Stopped too soon or applied too long"
    )
    missing = check_uca_categories(no_omission)
    assert any("Not providing" in m for m in missing)

    # Live fixture coverage: complete matrix carries all 4 categories
    complete_content = read_fixture("complete_stpa_matrix.md")
    assert check_uca_categories(complete_content) == []


def test_astm_f3269_rta_and_commercial_toolchain_hooks():
    """Verify ASTM F3269-17 RTA and MATLAB/Simulink hooks are strictly enforced."""
    base_content = read_fixture("complete_stpa_matrix.md")

    # Strip ASTM F3269-17
    no_rta = base_content.replace("ASTM F3269-17", "").replace("ASTM F3269", "")
    errors = validate_safety_matrix_content(no_rta)
    assert any("ASTM F3269-17" in err for err in errors)

    # Strip MATLAB / Simulink
    no_matlab = base_content.replace("MATLAB", "").replace("Simulink", "").replace("Stateflow", "").replace("Embedded Coder", "").replace("SLDV", "")
    errors = validate_safety_matrix_content(no_matlab)
    assert any("MATLAB / Simulink" in err for err in errors)


def test_truncated_cartesian_matrix_rejected(tmpdir):
    """Verify a truncated UCA cartesian matrix (12 of 16 permutations) is rejected."""
    truncated_content = read_fixture("truncated_uca_matrix.md")
    write_safety_matrix(tmpdir, truncated_content)

    with pytest.raises(SystemExit) as exc_info:
        check_safety_integrity_and_sora_completeness(tmpdir)
    assert exc_info.value.code == 1


def test_missing_guideword_matrix_rejected(tmpdir, capsys):
    """Verify a matrix covering only 3 of 4 STPA guide words is rejected with the category named."""
    content = read_fixture("missing_guideword_matrix.md")
    write_safety_matrix(tmpdir, content)

    with pytest.raises(SystemExit) as exc_info:
        check_safety_integrity_and_sora_completeness(tmpdir)
    assert exc_info.value.code == 1

    captured = capsys.readouterr()
    assert "Stopped too soon" in captured.err, (
        f"Expected missing guide word category in stderr, got:\n{captured.err}"
    )


def test_incomplete_osos_matrix_rejected(tmpdir):
    """Verify a matrix missing OSO-23/OSO-24 is rejected end-to-end."""
    content = read_fixture("incomplete_osos.md")
    write_safety_matrix(tmpdir, content)

    with pytest.raises(SystemExit) as exc_info:
        check_safety_integrity_and_sora_completeness(tmpdir)
    assert exc_info.value.code == 1


def test_5_part_proof_structure_required(tmpdir):
    """Verify a proof block missing the analytical derivation part is rejected."""
    matrix_content = read_fixture("complete_stpa_matrix.md")
    proof_content = read_fixture("proof_missing_derivation.md")

    safety_dir = os.path.join(tmpdir, "docs", "safety")
    os.makedirs(safety_dir, exist_ok=True)
    with open(os.path.join(safety_dir, "STPA_MATRIX.md"), "w", encoding="utf-8") as f:
        f.write(matrix_content)
    with open(os.path.join(safety_dir, "T-01-proof.md"), "w", encoding="utf-8") as f:
        f.write(proof_content)

    with pytest.raises(SystemExit) as exc_info:
        check_safety_integrity_and_sora_completeness(tmpdir)
    assert exc_info.value.code == 1


def test_complete_proof_block_accepted(tmpdir):
    """Verify a complete 5-part proof artifact passes aggregated downstream validation."""
    matrix_content = read_fixture("complete_stpa_matrix.md")
    proof_content = read_fixture("complete_proof.md")

    safety_dir = os.path.join(tmpdir, "docs", "safety")
    os.makedirs(safety_dir, exist_ok=True)
    with open(os.path.join(safety_dir, "STPA_MATRIX.md"), "w", encoding="utf-8") as f:
        f.write(matrix_content)
    with open(os.path.join(safety_dir, "T-01-proof.md"), "w", encoding="utf-8") as f:
        f.write(proof_content)

    check_safety_integrity_and_sora_completeness(tmpdir)


def test_end_to_end_check_17_downstream_integration(tmpdir):
    """Verify end-to-end Check 17 execution on a downstream project directory."""
    valid_content = read_fixture("complete_stpa_matrix.md")
    stpa_file = write_safety_matrix(tmpdir, valid_content)
    assert os.path.isfile(stpa_file)

    # Should pass with no exception
    check_safety_integrity_and_sora_completeness(tmpdir)

    # Corrupt file with violation (drop OSO-24)
    corrupted_content = valid_content.replace("OSO-24", "INVALID-REF")
    with open(stpa_file, "w", encoding="utf-8") as f:
        f.write(corrupted_content)

    with pytest.raises(SystemExit) as exc_info:
        check_safety_integrity_and_sora_completeness(tmpdir)
    assert exc_info.value.code == 1
