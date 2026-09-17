"""
Zero-mocking structural purge gate for STPA safety integrity tests.
/// Realises: [Issue071/MockGeneratorPurge]
"""
import pathlib


def _repo_root() -> pathlib.Path:
    return pathlib.Path(__file__).resolve().parents[1]


MOCK_SYMBOL = "generate_valid_" + "stpa_matrix_content"


def _text(path: pathlib.Path) -> str:
    return path.read_bytes().decode("utf-8", errors="ignore")


def test_no_mock_generator_symbol_in_tests_or_scripts():
    """No in-memory STPA mock generator symbol may exist anywhere in tests/ or scripts/."""
    offenders = []
    for scan_dir in ("tests", "scripts"):
        scan_root = _repo_root() / scan_dir
        for candidate in sorted(scan_root.rglob("*")):
            if candidate.is_file() and MOCK_SYMBOL in _text(candidate):
                offenders.append(str(candidate.relative_to(_repo_root())))
    assert not offenders, (
        f"In-memory mock generator symbol still present in tests/ or scripts/ sources: {offenders}"
    )


def test_no_synthetic_uca_stub_strings_in_test_safety_integrity():
    """No hardcoded synthetic UCA-01..UCA-04 stub strings may exist in tests/test_safety_integrity.py."""
    target = _repo_root() / "tests" / "test_safety_integrity.py"
    assert target.is_file(), f"{target} missing; cannot verify stub purge."
    content = _text(target)
    stubs = [f"UCA-{i:02d}" for i in range(1, 5)]
    found = [stub for stub in stubs if stub in content]
    assert not found, (
        f"Synthetic UCA stub strings still hardcoded in {target.name}: {found}"
    )
