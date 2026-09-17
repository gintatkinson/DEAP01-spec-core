"""
Unit tests for the abstract dynamic Cartesian STPA transpiler and 10-proof
generator in scripts/compile_sysml.py (feat(compiler): DEAP-spec-core#72).

All tests drive the public CLI entrypoint:

    python3 scripts/compile_sysml.py --stpa-transpile --schema <fixture.sysml> --out-dir <dir>

and assert over the emitted 10-pillar safety artifact suite. The fixture is a
neutral, domain-agnostic SysML v2 model: two part defs (ControllerA with 3
action defs, ControllerB with 2 action defs), typed in/out parameters, and
constraint/attribute defs carrying symbolic numeric thresholds. Every numeric
constant asserted in generated output MUST derive from fixture schema tokens
or from deterministic structural identifiers derived from model cardinality.
"""

import json
import os
import re
import subprocess
import sys
import time

import pytest

TEST_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(TEST_DIR, ".."))
SCRIPT = os.path.join(PROJECT_ROOT, "scripts", "compile_sysml.py")
PENDING_PARAMETER = "PENDING_PARAMETER"

NEUTRAL_FIXTURE = """\
package NeutralModel {
    doc /* Neutral domain-agnostic schema for STPA transpiler tests */
    part def ControllerA {
        action EngagePrimaryControl(in requestId : String, out status : Boolean);
        action AdjustOperatingParameter(in targetValue : Real, out appliedValue : Real);
        action TerminateActiveOperation(in reasonCode : String);
    }
    part def ControllerB {
        action MonitorSystemState(in sampleRate : Real, out stateVector : Real);
        action IssuePeriodicCommand(in interval : Real, out commandIssued : Boolean);
    }
    constraint def SafeThreshold_OperatingAltitude {
        OperatingAltitude <= 3000.0;
    }
    constraint def SafeThreshold_MaxOperatingSpeed {
        MaxOperatingSpeed <= 60.0;
    }
    constraint def SafeThreshold_MinOperatingSpeed {
        MinOperatingSpeed >= 12.5;
    }
    attribute MaxOperatingSpeed : Real = 60.0;
    attribute MinOperatingSpeed : Real = 12.5;
    attribute ParameterMass : Real = 40.0;
    attribute GravityAcceleration : Real = 9.80665;
    attribute MediumDensity : Real = 1.225;
    attribute DragCoefficient : Real = 1.75;
    attribute DecelerationArea : Real = 12.5;
    attribute FrontalArea : Real = 210.0e-4;
    attribute EnergyDensityLimit : Real = 28.5e4;
    attribute KineticFactor : Real = 2.0;
    attribute InitialAltitude : Real = 1000.0;
    attribute LiftToDragRatio : Real = 14.2;
    attribute BestGlideSpeed : Real = 24.5;
    attribute SinkRate : Real = 1.725;
    attribute WindDriftSpeed : Real = 10.0;
    attribute ContainmentRadius : Real = 25000.0;
    attribute BufferRadius : Real = 2000.0;
    attribute BarrierRadius : Real = 5000.0;
    attribute PositionOffset : Real = 4850.0;
    attribute GroundSpeed : Real = 35.0;
    attribute AccelerationLimit : Real = 24.5166;
    attribute BarrierGain : Real = 2.0;
    attribute InitialPotential : Real = 1200.0;
    attribute SafePotential : Real = 50.0;
    attribute BleedResistance : Real = 100.0e3;
    attribute StorageCapacitance : Real = 10.0e-6;
    attribute StrokeLength : Real = 4.2;
    attribute PistonArea : Real = 0.007854;
    attribute RailPressure : Real = 6.5e5;
    attribute FrictionCoefficient : Real = 0.045;
    attribute InclineAngle : Real = 12.0;
    attribute StallSpeed : Real = 18.0;
    attribute TransmitPower : Real = 40.0;
    attribute TransmitGain : Real = 18.0;
    attribute ReceiveGain : Real = 3.5;
    attribute CarrierFrequency : Real = 5.03e9;
    attribute StandoffDistance : Real = 50000.0;
    attribute InsertionLoss : Real = 4.5;
    attribute ReceiveSensitivity : Real = -102.0;
    attribute MinLinkMargin : Real = 12.0;
    attribute TotalEnergy : Real = 3.24e6;
    attribute PropulsionPower : Real = 650.0;
    attribute AvionicsPower : Real = 95.0;
    attribute CruiseSpeed : Real = 35.0;
    attribute ReserveDistance : Real = 30000.0;
    attribute AbortReserve : Real = 4.86e5;
    attribute DischargeCurrent : Real = 26.6;
    attribute InternalResistance : Real = 0.038;
    attribute DissipationProduct : Real = 1.85;
    attribute AmbientTemperature : Real = 45.0;
    attribute ThermalLimit : Real = 60.0;
    attribute WellClearRadius : Real = 1200.0;
    attribute VerticalClearance : Real = 137.0;
    attribute WarnTime : Real = 35.0;
    attribute RelativeVelocity : Real = 110.0;
    attribute EvadeAcceleration : Real = 3.9226;
    attribute TerminalMass : Real = 38.5;
    attribute DescentAngle : Real = 22.0;
    attribute DescentDragCoefficient : Real = 0.145;
    attribute ReferenceArea : Real = 1.15;
    attribute SeaLevelDensity : Real = 1.205;
    attribute DynamicPressureLimit : Real = 1850.0;
    attribute FieldOfViewHalf : Real = 22.0;
    attribute ChannelFailureRate1 : Real = 1.2e-4;
    attribute ChannelFailureRate2 : Real = 1.5e-5;
    attribute SwitchRate : Real = 7.2e3;
    attribute MissionDuration : Real = 1.0;
    attribute FailureCeiling : Real = 1.0e-7;
}
"""

CANONICAL_GUIDE_WORDS = (
    "Not providing",
    "Providing",
    "Too early / Too late / Out of order",
    "Stopped too soon / Applied too long",
)

EXPECTED_ARTIFACTS = (
    "01_LOSSES_HAZARDS_TOPOLOGY.md",
    "02_UCA_COMBINATORIAL_MATRIX.md",
    "03_LOSS_SCENARIOS.md",
    "04_SAFETY_CONSTRAINTS.md",
    "05_FMECA_MATRIX.md",
    "06_SORA_SAIL_ASSESSMENT.md",
    "07_RTA_ARCHITECTURE.md",
    "STPA_MATRIX.md",
    "HAZARD_LOG.md",
    "SLDV_FORMAL_PROOFS.m",
)

NUMERIC_LITERAL_RE = re.compile(r"\d+(?:\.\d+)?(?:[eE][+-]?\d+)?")
STRUCTURAL_ID_RE = re.compile(r"\b(?:UCA|FMECA|OSO|SC|LS|L|H|T)-\d+\b")

CONTROLLER_TOTAL_ACTION_COUNT = 3 + 2


@pytest.fixture()
def neutral_schema(tmp_path):
    """Writes the neutral SysML v2 fixture schema into a tmp_path file."""
    schema_path = tmp_path / "neutral_fixture.sysml"
    schema_path.write_text(NEUTRAL_FIXTURE, encoding="utf-8")
    return schema_path


def _run_cli(schema_path, out_dir, extra_args=()):
    """Invokes the documented --stpa-transpile CLI with bounded output capture."""
    cmd = [
        sys.executable,
        SCRIPT,
        "--stpa-transpile",
        "--schema",
        str(schema_path),
        "--out-dir",
        str(out_dir),
    ]
    cmd.extend(str(a) for a in extra_args)
    before = time.perf_counter()
    proc = subprocess.run(cmd, capture_output=True, text=True, cwd=PROJECT_ROOT)
    elapsed = time.perf_counter() - before
    return proc, elapsed


def _read_artifact(out_dir, artifact_name):
    artifact_path = os.path.join(str(out_dir), artifact_name)
    with open(artifact_path, "r", encoding="utf-8") as f:
        return f.read()


def _parse_markdown_table_rows(text):
    rows = []
    for line in text.splitlines():
        line = line.strip()
        if not line.startswith("|"):
            continue
        inner = line.replace("|", "").strip()
        if inner and set(inner.replace(":", "").replace("-", "")) <= {" "}:
            continue
        if "---" in line:
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        rows.append(cells)
    return rows


def test_cartesian_expansion_cardinality(neutral_schema, tmp_path):
    """Transpile yields (3+2)*4 = 20 UCA rows, each with controller, action, guide word."""
    out_dir = tmp_path / "out_cardinality"
    proc, _ = _run_cli(neutral_schema, out_dir)
    assert proc.returncode == 0, f"CLI failed: {proc.stderr}"

    uca_text = _read_artifact(out_dir, "02_UCA_COMBINATORIAL_MATRIX.md")
    uca_rows = [
        row for row in _parse_markdown_table_rows(uca_text)
        if row and re.match(r"UCA-\d+", row[0])
    ]

    assert len(uca_rows) == CONTROLLER_TOTAL_ACTION_COUNT * 4

    controller_a_rows = [row for row in uca_rows if row[1] == "ControllerA"]
    controller_b_rows = [row for row in uca_rows if row[1] == "ControllerB"]
    assert len(controller_a_rows) == 3 * 4
    assert len(controller_b_rows) == 2 * 4

    for row in uca_rows:
        assert len(row) >= 4
        assert re.match(r"UCA-\d+", row[0])
        assert row[1] in ("ControllerA", "ControllerB")
        assert row[3] in CANONICAL_GUIDE_WORDS

    present_guide_words = {row[3] for row in uca_rows}
    assert present_guide_words == set(CANONICAL_GUIDE_WORDS)


def test_artifact_suite_emission(neutral_schema, tmp_path):
    """All artifacts (uca matrix, fmeca matrix, sora oso roster, proofs, hazard log skeletons) emitted."""
    out_dir = tmp_path / "out_suite"
    proc, _ = _run_cli(neutral_schema, out_dir)
    assert proc.returncode == 0, f"CLI failed: {proc.stderr}"

    for artifact in EXPECTED_ARTIFACTS:
        path = os.path.join(str(out_dir), artifact)
        assert os.path.isfile(path), f"Missing artifact: {artifact}"
        assert os.path.getsize(path) > 0, f"Empty artifact: {artifact}"

    uca_text = _read_artifact(out_dir, "02_UCA_COMBINATORIAL_MATRIX.md")
    assert "UCA-" in uca_text

    fmeca_text = _read_artifact(out_dir, "05_FMECA_MATRIX.md")
    assert "FMECA-" in fmeca_text
    assert PENDING_PARAMETER in fmeca_text

    sora_text = _read_artifact(out_dir, "06_SORA_SAIL_ASSESSMENT.md")
    for oso_id in ("OSO-01", "OSO-12", "OSO-24"):
        assert oso_id in sora_text

    rta_text = _read_artifact(out_dir, "07_RTA_ARCHITECTURE.md")
    for theorem_id in ("T-01", "T-05", "T-10"):
        assert theorem_id in rta_text

    sldv_text = _read_artifact(out_dir, "SLDV_FORMAL_PROOFS.m")
    assert "sldv.assert" in sldv_text

    hazard_log = _read_artifact(out_dir, "HAZARD_LOG.md")
    assert "H-" in hazard_log
    assert "L-" in hazard_log

    topology = _read_artifact(out_dir, "01_LOSSES_HAZARDS_TOPOLOGY.md")
    assert "```mermaid" in topology
    assert "graph TD" in topology
    assert topology.count("```") % 2 == 0


def test_fmeca_rpn_from_scoring_configuration(neutral_schema, tmp_path):
    """FMECA RPN = S*O*D computed from configured categorical scoring (no domain defaults)."""
    scoring_config = {
        "severity_scale": [
            {"label": "Highest", "score": 10},
            {"label": "Elevated", "score": 6},
            {"label": "Baseline", "score": 2},
        ],
        "occurrence_scale": [
            {"label": "Recurrent", "score": 9},
            {"label": "Occasional", "score": 5},
            {"label": "Isolated", "score": 1},
        ],
        "detection_scale": [
            {"label": "Undetected", "score": 10},
            {"label": "Detectable", "score": 4},
            {"label": "Evident", "score": 1},
        ],
    }
    config_path = tmp_path / "scoring_config.json"
    config_path.write_text(json.dumps(scoring_config), encoding="utf-8")

    out_dir = tmp_path / "out_rpn"
    proc, _ = _run_cli(neutral_schema, out_dir, extra_args=("--fmeca-scoring-config", config_path))
    assert proc.returncode == 0, f"CLI failed: {proc.stderr}"

    fmeca_text = _read_artifact(out_dir, "05_FMECA_MATRIX.md")
    fmeca_rows = [
        row for row in _parse_markdown_table_rows(fmeca_text)
        if row and re.match(r"FMECA-\d+", row[0])
    ]
    assert fmeca_rows

    def _score(value_cell, scale):
        for entry in scale:
            if entry["label"] in value_cell:
                return entry["score"]
        return None

    for row in fmeca_rows:
        s = _score(row[4], scoring_config["severity_scale"])
        o = _score(row[5], scoring_config["occurrence_scale"])
        d = _score(row[6], scoring_config["detection_scale"])
        assert s is not None and o is not None and d is not None
        rpn_text = row[7]
        assert str(s * o * d) in rpn_text


def test_zero_hardcoded_numeric_defaults(neutral_schema, tmp_path):
    """Generated output contains no numeric constant unless it appears in the schema text."""
    out_dir = tmp_path / "out_zero_defaults"
    proc, _ = _run_cli(neutral_schema, out_dir)
    assert proc.returncode == 0, f"CLI failed: {proc.stderr}"

    schema_literals = set(NUMERIC_LITERAL_RE.findall(NEUTRAL_FIXTURE))
    assert schema_literals

    offenders = {}
    for artifact in EXPECTED_ARTIFACTS:
        text = _read_artifact(out_dir, artifact)
        scrubbed = STRUCTURAL_ID_RE.sub(" ", text)
        found = set(NUMERIC_LITERAL_RE.findall(scrubbed))
        unknown = sorted(found - schema_literals)
        if unknown:
            offenders[artifact] = unknown

    assert not offenders, (
        f"Generated artifacts contain numeric constants not present in schema text: {offenders}"
    )


def test_cli_stpa_transpile_exit_zero(neutral_schema, tmp_path):
    """CLI --stpa-transpile invocation exits 0 and emits the artifact directory."""
    out_dir = tmp_path / "out_cli"
    proc, elapsed = _run_cli(neutral_schema, out_dir)

    assert proc.returncode == 0, f"CLI failed: {proc.stderr}"
    assert os.path.isdir(str(out_dir))
    assert any(name.endswith(".md") or name.endswith(".m") for name in os.listdir(str(out_dir)))
    assert "Traceback" not in proc.stderr


def test_transpiler_execution_time_bound(neutral_schema, tmp_path):
    """End-to-end transpilation completes under 5.0 s wall clock (CI-safe bound)."""
    out_dir = tmp_path / "out_perf"
    proc, elapsed = _run_cli(neutral_schema, out_dir)

    assert proc.returncode == 0, f"CLI failed: {proc.stderr}"
    assert elapsed < 5.0, f"Transpilation took {elapsed:.3f}s, exceeding 5.0s bound"
