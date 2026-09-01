import os
import sys
import tempfile
import unittest

repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if repo_root not in sys.path:
    sys.path.insert(0, repo_root)

parity_src = os.path.join(repo_root, "skills", "spec-orchestrator", "parity_auditor", "src")
if parity_src not in sys.path:
    sys.path.insert(0, parity_src)

from parity_auditor.core.workspace import WorkspaceRepository
from parity_auditor.aggregator import AGGREGATING_VALIDATORS
from parity_auditor.validators.standards_measurement_validator import (
    StandardsAndMeasurementValidator,
    SiDimensionVector,
    parse_si_unit,
    parse_standard_decorators,
    parse_nyquist_parameters,
    SDO_STANDARDS_TAXONOMY,
    ASSURANCE_LEVEL_LATTICES,
)


class TestStandardsAndMeasurementValidator(unittest.TestCase):
    """
    Unit test suite for Gate 25: Standards & SI 7D Parameter Metrology Validator.
    Asserts:
    (a) SI 7-dimensional exponent vector parsing in Z^7 for base and derived units,
    (b) Theorem 3 Dimensional Homogeneity (D(e_src) == D(e_dst)),
    (c) Nyquist-Shannon sampling check (f_sample >= 2*f_max),
    (d) SDO Standards Taxonomy lattice validation.
    """

    # -------------------------------------------------------------------------
    # (a) SI 7-Dimensional Exponent Vector Parsing in Z^7
    # -------------------------------------------------------------------------

    def test_si_base_dimension_parsing(self):
        """Verify SI base units map to the exact canonical Z^7 exponent vector."""
        # L (length, m)
        self.assertEqual(parse_si_unit("m").exponents, (1, 0, 0, 0, 0, 0, 0))
        self.assertEqual(parse_si_unit("meter").exponents, (1, 0, 0, 0, 0, 0, 0))
        self.assertEqual(parse_si_unit("meters").exponents, (1, 0, 0, 0, 0, 0, 0))
        self.assertEqual(parse_si_unit("km").exponents, (1, 0, 0, 0, 0, 0, 0))
        self.assertEqual(parse_si_unit("mm").exponents, (1, 0, 0, 0, 0, 0, 0))

        # M (mass, kg)
        self.assertEqual(parse_si_unit("kg").exponents, (0, 1, 0, 0, 0, 0, 0))
        self.assertEqual(parse_si_unit("kilogram").exponents, (0, 1, 0, 0, 0, 0, 0))
        self.assertEqual(parse_si_unit("g").exponents, (0, 1, 0, 0, 0, 0, 0))

        # T (time, s)
        self.assertEqual(parse_si_unit("s").exponents, (0, 0, 1, 0, 0, 0, 0))
        self.assertEqual(parse_si_unit("sec").exponents, (0, 0, 1, 0, 0, 0, 0))
        self.assertEqual(parse_si_unit("second").exponents, (0, 0, 1, 0, 0, 0, 0))
        self.assertEqual(parse_si_unit("ms").exponents, (0, 0, 1, 0, 0, 0, 0))
        self.assertEqual(parse_si_unit("hr").exponents, (0, 0, 1, 0, 0, 0, 0))

        # I (electric current, A)
        self.assertEqual(parse_si_unit("A").exponents, (0, 0, 0, 1, 0, 0, 0))
        self.assertEqual(parse_si_unit("ampere").exponents, (0, 0, 0, 1, 0, 0, 0))
        self.assertEqual(parse_si_unit("mA").exponents, (0, 0, 0, 1, 0, 0, 0))

        # Theta (thermodynamic temperature, K)
        self.assertEqual(parse_si_unit("K").exponents, (0, 0, 0, 0, 1, 0, 0))
        self.assertEqual(parse_si_unit("kelvin").exponents, (0, 0, 0, 0, 1, 0, 0))
        self.assertEqual(parse_si_unit("degK").exponents, (0, 0, 0, 0, 1, 0, 0))
        self.assertEqual(parse_si_unit("degC").exponents, (0, 0, 0, 0, 1, 0, 0))

        # N (amount of substance, mol)
        self.assertEqual(parse_si_unit("mol").exponents, (0, 0, 0, 0, 0, 1, 0))
        self.assertEqual(parse_si_unit("mole").exponents, (0, 0, 0, 0, 0, 1, 0))

        # J (luminous intensity, cd)
        self.assertEqual(parse_si_unit("cd").exponents, (0, 0, 0, 0, 0, 0, 1))
        self.assertEqual(parse_si_unit("candela").exponents, (0, 0, 0, 0, 0, 0, 1))

    def test_si_derived_and_compound_dimension_parsing(self):
        """Verify SI derived and compound units parse correctly to Z^7."""
        # Frequency (Hz = s^-1)
        self.assertEqual(parse_si_unit("Hz").exponents, (0, 0, -1, 0, 0, 0, 0))
        self.assertEqual(parse_si_unit("1/s").exponents, (0, 0, -1, 0, 0, 0, 0))

        # Velocity (m/s)
        self.assertEqual(parse_si_unit("m/s").exponents, (1, 0, -1, 0, 0, 0, 0))
        self.assertEqual(parse_si_unit("m*s^-1").exponents, (1, 0, -1, 0, 0, 0, 0))

        # Acceleration (m/s^2)
        self.assertEqual(parse_si_unit("m/s^2").exponents, (1, 0, -2, 0, 0, 0, 0))
        self.assertEqual(parse_si_unit("m/s2").exponents, (1, 0, -2, 0, 0, 0, 0))

        # Force (N = kg*m/s^2)
        self.assertEqual(parse_si_unit("N").exponents, (1, 1, -2, 0, 0, 0, 0))
        self.assertEqual(parse_si_unit("kg*m/s^2").exponents, (1, 1, -2, 0, 0, 0, 0))

        # Pressure (Pa = N/m^2 = kg/(m*s^2))
        self.assertEqual(parse_si_unit("Pa").exponents, (-1, 1, -2, 0, 0, 0, 0))
        self.assertEqual(parse_si_unit("bar").exponents, (-1, 1, -2, 0, 0, 0, 0))
        self.assertEqual(parse_si_unit("hPa").exponents, (-1, 1, -2, 0, 0, 0, 0))

        # Energy (J = N*m = kg*m^2/s^2)
        self.assertEqual(parse_si_unit("J").exponents, (2, 1, -2, 0, 0, 0, 0))
        self.assertEqual(parse_si_unit("N*m").exponents, (2, 1, -2, 0, 0, 0, 0))

        # Power (W = J/s = kg*m^2/s^3)
        self.assertEqual(parse_si_unit("W").exponents, (2, 1, -3, 0, 0, 0, 0))
        self.assertEqual(parse_si_unit("kW").exponents, (2, 1, -3, 0, 0, 0, 0))

        # Voltage (V = W/A = kg*m^2/(A*s^3))
        self.assertEqual(parse_si_unit("V").exponents, (2, 1, -3, -1, 0, 0, 0))

        # Electric Charge (C = A*s)
        self.assertEqual(parse_si_unit("C").exponents, (0, 0, 1, 1, 0, 0, 0))

        # Resistance (Ohm = V/A)
        self.assertEqual(parse_si_unit("Ohm").exponents, (2, 1, -3, -2, 0, 0, 0))
        self.assertEqual(parse_si_unit("ohm").exponents, (2, 1, -3, -2, 0, 0, 0))

        # Capacitance (F = C/V)
        self.assertEqual(parse_si_unit("F").exponents, (-2, -1, 4, 2, 0, 0, 0))

        # Magnetic Flux (Wb = V*s)
        self.assertEqual(parse_si_unit("Wb").exponents, (2, 1, -2, -1, 0, 0, 0))

        # Magnetic Flux Density (T = Wb/m^2)
        self.assertEqual(parse_si_unit("T").exponents, (0, 1, -2, -1, 0, 0, 0))

        # Dimensionless quantities (rad, deg, percent, scalar, count, boolean)
        self.assertEqual(parse_si_unit("rad").exponents, (0, 0, 0, 0, 0, 0, 0))
        self.assertEqual(parse_si_unit("deg").exponents, (0, 0, 0, 0, 0, 0, 0))
        self.assertEqual(parse_si_unit("percent").exponents, (0, 0, 0, 0, 0, 0, 0))
        self.assertEqual(parse_si_unit("%").exponents, (0, 0, 0, 0, 0, 0, 0))
        self.assertEqual(parse_si_unit("1").exponents, (0, 0, 0, 0, 0, 0, 0))
        self.assertEqual(parse_si_unit("dimensionless").exponents, (0, 0, 0, 0, 0, 0, 0))
        self.assertEqual(parse_si_unit("").exponents, (0, 0, 0, 0, 0, 0, 0))
        self.assertEqual(parse_si_unit("-").exponents, (0, 0, 0, 0, 0, 0, 0))
        self.assertEqual(parse_si_unit("count").exponents, (0, 0, 0, 0, 0, 0, 0))

    def test_si_vector_arithmetic_and_latex(self):
        """Verify vector operations and LaTeX formatting."""
        v1 = parse_si_unit("m")
        v2 = parse_si_unit("s")
        v_vel = v1 / v2
        self.assertEqual(v_vel.exponents, (1, 0, -1, 0, 0, 0, 0))
        self.assertEqual(v_vel.to_latex(), r"\text{m} \cdot \text{s}^{-1}")

        v_accel = v_vel / v2
        self.assertEqual(v_accel.exponents, (1, 0, -2, 0, 0, 0, 0))
        self.assertEqual(v_accel.to_latex(), r"\text{m} \cdot \text{s}^{-2}")

        v_dimless = parse_si_unit("rad")
        self.assertTrue(v_dimless.is_dimensionless())
        self.assertEqual(v_dimless.to_latex(), r"1")

    # -------------------------------------------------------------------------
    # (b) Theorem 3: Dimensional Homogeneity (D(e_src) == D(e_dst))
    # -------------------------------------------------------------------------

    def test_theorem_3_dimensional_homogeneity_valid(self):
        """Verify Theorem 3 passes when all connected ports share identical SI dimensions."""
        with tempfile.TemporaryDirectory() as tmpdir:
            interfaces_dir = os.path.join(tmpdir, "docs", "interfaces")
            schema_dir = os.path.join(tmpdir, "schema")
            os.makedirs(interfaces_dir, exist_ok=True)
            os.makedirs(schema_dir, exist_ok=True)

            # ICD_01 with matching port types and connections
            icd01_md = """# ICD-01 Interface Matrix
## 2. Logical Port Roster Table
| Port ID | Subsystem | Port Name | Direction | Port Type | Multiplicity | Protocol Profile |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `PORT-01` | NavigationSubsystem | nav_vel_out | OUT | VelocityPort | 1 | RealTimeStream |
| `PORT-02` | FlightControlSubsystem | fc_vel_in | IN | VelocityPort | 1 | RealTimeStream |

## 4. Connection Binding Roster Table
| Connection ID | Source Port | Dest Port | Flow Behavior | Latency Max (ms) | Reliability Req | Item Flows Conveyed |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `CONN-01` | `NavigationSubsystem.nav_vel_out` | `FlightControlSubsystem.fc_vel_in` | Stream | 5.0 | High | `PrimaryVelocity` |
"""
            with open(os.path.join(interfaces_dir, "ICD_01_SYSTEM_INTERFACE_MATRIX.md"), "w", encoding="utf-8") as f:
                f.write(icd01_md)

            # ICD_02 with homogeneous SI units (m/s)
            icd02_md = """# ICD-02 Master Signal Flow Dictionary
| Signal ID | Signal Name | Source Port | Dest Port | Data Type | SI Units | Valid Range | Update Rate | Safe Default Value | Schema Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `SIG-01` | PrimaryVelocity | `NavigationSubsystem.nav_vel_out` | `FlightControlSubsystem.fc_vel_in` | Float32 | `m/s` | `[-50.0, 50.0]` | `100 Hz` | `0.0` | `schema/model.sysml` |
"""
            with open(os.path.join(interfaces_dir, "ICD_02_MASTER_SIGNAL_DICTIONARY.md"), "w", encoding="utf-8") as f:
                f.write(icd02_md)

            # SysML model
            sysml_model = """package SystemSSOT {
    part def NavigationSubsystem {
        out port nav_vel_out : Velocity;
    }
    part def FlightControlSubsystem {
        in port fc_vel_in : Velocity;
    }
    connection conn1 connect NavigationSubsystem.nav_vel_out to FlightControlSubsystem.fc_vel_in;
}
"""
            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(sysml_model)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = StandardsAndMeasurementValidator()
            findings = validator.validate(repo)
            self.assertEqual(findings, [])

    def test_theorem_3_dimensional_inhomogeneity_detected(self):
        """Verify Theorem 3 flags when source and destination ports have incompatible SI dimensions."""
        with tempfile.TemporaryDirectory() as tmpdir:
            interfaces_dir = os.path.join(tmpdir, "docs", "interfaces")
            os.makedirs(interfaces_dir, exist_ok=True)

            # Signal dictionary connecting velocity (m/s) to acceleration input (m/s^2)
            icd01_md = """# ICD-01 Interface Matrix
## 2. Logical Port Roster Table
| Port ID | Subsystem | Port Name | Direction | Port Type | Multiplicity | Protocol Profile |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `PORT-01` | SensorSubsystem | sensor_accel_out | OUT | AccelPort | 1 | Stream |
| `PORT-02` | GuidanceSubsystem | guidance_pos_in | IN | PositionPort | 1 | Stream |

## 4. Connection Binding Roster Table
| Connection ID | Source Port | Dest Port | Flow Behavior | Latency Max (ms) | Reliability Req | Item Flows Conveyed |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `CONN-01` | `SensorSubsystem.sensor_accel_out` | `GuidanceSubsystem.guidance_pos_in` | Stream | 5.0 | High | `AccelSignal` |
"""
            with open(os.path.join(interfaces_dir, "ICD_01_SYSTEM_INTERFACE_MATRIX.md"), "w", encoding="utf-8") as f:
                f.write(icd01_md)

            # ICD_02 defines AccelSignal with m/s^2, but guidance_pos_in expects meters (m)
            icd02_md = """# ICD-02 Master Signal Flow Dictionary
| Signal ID | Signal Name | Source Port | Dest Port | Data Type | SI Units | Valid Range | Update Rate | Safe Default Value | Schema Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `SIG-01` | AccelSignal | `SensorSubsystem.sensor_accel_out` | `GuidanceSubsystem.guidance_pos_in` | Float32 | `m/s^2` | `[-20.0, 20.0]` | `100 Hz` | `0.0` | `schema/model.sysml` |
| `SIG-02` | ExpectedPosition | `SensorSubsystem.dummy_out` | `GuidanceSubsystem.guidance_pos_in` | Float32 | `m` | `[0.0, 1000.0]` | `100 Hz` | `0.0` | `schema/model.sysml` |
"""
            with open(os.path.join(interfaces_dir, "ICD_02_MASTER_SIGNAL_DICTIONARY.md"), "w", encoding="utf-8") as f:
                f.write(icd02_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = StandardsAndMeasurementValidator()
            findings = validator.validate(repo)

            inhomogeneous = [f for f in findings if getattr(f, "rule_id", "") == "metrology-dimensional-inhomogeneity"]
            self.assertTrue(len(inhomogeneous) >= 1)
            self.assertIn("Theorem 3", str(inhomogeneous[0]))
            self.assertIn("guidance_pos_in", str(inhomogeneous[0]))

    # -------------------------------------------------------------------------
    # (c) Nyquist-Shannon Sampling Frequency Check (f_sample >= 2*f_max)
    # -------------------------------------------------------------------------

    def test_nyquist_shannon_sampling_valid(self):
        """Verify Nyquist sampling passes when f_sample >= 2 * f_max."""
        # f_sample = 100 Hz, f_max = 40 Hz -> 100 >= 80 (OK)
        params = parse_nyquist_parameters("100 Hz", "40 Hz")
        self.assertIsNotNone(params)
        f_sample, f_max = params
        self.assertTrue(f_sample >= 2 * f_max)

        # Exact boundary: 100 Hz == 2 * 50 Hz (OK)
        params_bound = parse_nyquist_parameters("100 Hz", "50 Hz")
        f_s, f_m = params_bound
        self.assertTrue(f_s >= 2 * f_m)

    def test_nyquist_shannon_aliasing_detected(self):
        """Verify Nyquist check detects undersampling / aliasing risk when f_sample < 2 * f_max."""
        with tempfile.TemporaryDirectory() as tmpdir:
            interfaces_dir = os.path.join(tmpdir, "docs", "interfaces")
            os.makedirs(interfaces_dir, exist_ok=True)

            icd01_md = """# ICD-01
## 2. Logical Port Roster Table
| Port ID | Subsystem | Port Name | Direction | Port Type | Multiplicity | Protocol Profile |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `PORT-01` | Radar | radar_out | OUT | RadarPort | 1 | Stream |
| `PORT-02` | Tracker | tracker_in | IN | RadarPort | 1 | Stream |

## 4. Connection Binding Roster Table
| Connection ID | Source Port | Dest Port | Flow Behavior | Latency Max (ms) | Reliability Req | Item Flows Conveyed |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `CONN-01` | `Radar.radar_out` | `Tracker.tracker_in` | Stream | 10.0 | High | `RadarEcho` |
"""
            with open(os.path.join(interfaces_dir, "ICD_01_SYSTEM_INTERFACE_MATRIX.md"), "w", encoding="utf-8") as f:
                f.write(icd01_md)

            # Signal with 50 Hz update rate but 40 Hz bandwidth/f_max -> Nyquist requires >= 80 Hz!
            icd02_md = """# ICD-02 Master Signal Flow Dictionary
| Signal ID | Signal Name | Source Port | Dest Port | Data Type | SI Units | Valid Range | Update Rate | Max Frequency | Safe Default Value | Schema Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `SIG-01` | RadarEcho | `Radar.radar_out` | `Tracker.tracker_in` | Float32 | `V` | `[-5.0, 5.0]` | `50 Hz` | `40 Hz` | `0.0` | `schema/model.sysml` |
"""
            with open(os.path.join(interfaces_dir, "ICD_02_MASTER_SIGNAL_DICTIONARY.md"), "w", encoding="utf-8") as f:
                f.write(icd02_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = StandardsAndMeasurementValidator()
            findings = validator.validate(repo)

            aliasing_findings = [f for f in findings if getattr(f, "rule_id", "") == "metrology-nyquist-aliasing-detected"]
            self.assertTrue(len(aliasing_findings) >= 1)
            self.assertIn("SIG-01", str(aliasing_findings[0]))
            self.assertIn("Nyquist-Shannon", str(aliasing_findings[0]))

    # -------------------------------------------------------------------------
    # (d) SDO Standards Taxonomy Lattice Validation
    # -------------------------------------------------------------------------

    def test_sdo_standards_taxonomy_parsing(self):
        """Verify parsing of @standard() SysML decorators and doc tags."""
        # SysML decorator: @standard(DO_178C, "Table A-3", DAL_A)
        sysml_text = """package SystemSpecs {
    part def FlightController {
        @standard(DO_178C, "Table A-3 Objective 4", DAL_A)
        action computeGuidance;
    }
}
"""
        decs1 = parse_standard_decorators(sysml_text)
        self.assertEqual(len(decs1), 1)
        self.assertEqual(decs1[0].standard_id, "DO-178C")
        self.assertEqual(decs1[0].clause_ref, "Table A-3 Objective 4")
        self.assertEqual(decs1[0].assurance_level, "DAL-A")
        self.assertEqual(decs1[0].sdo, "RTCA")

        # Markdown comment tag: /// Standard: [ISO-26262, "Part 3 Clause 5", ASIL-D]
        md_text = """# Feature
/// Standard: [ISO-26262, "Part 3 Clause 5", ASIL-D]
/// StandardTaxonomy: [JARUS_SORA_v2.5, "Step 5", SAIL IV]
"""
        decs2 = parse_standard_decorators(md_text)
        self.assertEqual(len(decs2), 2)
        self.assertEqual(decs2[0].standard_id, "ISO-26262")
        self.assertEqual(decs2[0].assurance_level, "ASIL-D")
        self.assertEqual(decs2[0].sdo, "ISO")
        self.assertEqual(decs2[1].standard_id, "JARUS-SORA-V2.5")
        self.assertEqual(decs2[1].assurance_level, "SAIL-IV")
        self.assertEqual(decs2[1].sdo, "JARUS")

    def test_sdo_standards_taxonomy_lattice_valid(self):
        """Verify valid standards and assurance levels pass lattice validation."""
        with tempfile.TemporaryDirectory() as tmpdir:
            features_dir = os.path.join(tmpdir, "docs", "features")
            schema_dir = os.path.join(tmpdir, "schema")
            os.makedirs(features_dir, exist_ok=True)
            os.makedirs(schema_dir, exist_ok=True)

            feat_md = """# Feature: Flight Management
/// Standard: [DO-178C, "Table A-3", DAL-A]
/// Standard: [ARP4754A, "Section 5.2", DAL-B]
/// Standard: [ISO-26262, "Part 4 Clause 6", ASIL-C]
/// Standard: [IEC-62304, "Clause 5.3", Class C]
/// Standard: [JARUS-SORA-v2.5, "Step 4", SAIL III]
"""
            with open(os.path.join(features_dir, "FEAT_01.md"), "w", encoding="utf-8") as f:
                f.write(feat_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = StandardsAndMeasurementValidator()
            findings = validator.validate(repo)
            self.assertEqual(findings, [])

    def test_sdo_standards_taxonomy_invalid_standard(self):
        """Verify unrecognized standard flags standards-unrecognized-standard."""
        with tempfile.TemporaryDirectory() as tmpdir:
            features_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(features_dir, exist_ok=True)

            feat_md = """# Feature
/// Standard: [UNKNOWN-STD-999, "Clause 1", DAL-A]
"""
            with open(os.path.join(features_dir, "FEAT_01.md"), "w", encoding="utf-8") as f:
                f.write(feat_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = StandardsAndMeasurementValidator()
            findings = validator.validate(repo)

            unrec = [f for f in findings if getattr(f, "rule_id", "") == "standards-unrecognized-standard"]
            self.assertTrue(len(unrec) >= 1)
            self.assertIn("UNKNOWN-STD-999", str(unrec[0]))

    def test_sdo_standards_taxonomy_invalid_assurance_level(self):
        """Verify invalid assurance level for standard family flags standards-invalid-assurance-level."""
        with tempfile.TemporaryDirectory() as tmpdir:
            features_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(features_dir, exist_ok=True)

            # DO-178C does NOT have ASIL-D (ASIL belongs to ISO 26262)
            feat_md = """# Feature
/// Standard: [DO-178C, "Table A-3", ASIL-D]
"""
            with open(os.path.join(features_dir, "FEAT_01.md"), "w", encoding="utf-8") as f:
                f.write(feat_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = StandardsAndMeasurementValidator()
            findings = validator.validate(repo)

            inv_lvl = [f for f in findings if getattr(f, "rule_id", "") == "standards-invalid-assurance-level"]
            self.assertTrue(len(inv_lvl) >= 1)
            self.assertIn("ASIL-D", str(inv_lvl[0]))
            self.assertIn("DO-178C", str(inv_lvl[0]))

    def test_sdo_standards_taxonomy_malformed_decorator(self):
        """Verify malformed decorator flags standards-malformed-decorator."""
        with tempfile.TemporaryDirectory() as tmpdir:
            features_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(features_dir, exist_ok=True)

            feat_md = """# Feature
/// Standard: [DO-178C]
"""
            with open(os.path.join(features_dir, "FEAT_01.md"), "w", encoding="utf-8") as f:
                f.write(feat_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = StandardsAndMeasurementValidator()
            findings = validator.validate(repo)

            malformed = [f for f in findings if getattr(f, "rule_id", "") == "standards-malformed-decorator"]
            self.assertTrue(len(malformed) >= 1)

    # -------------------------------------------------------------------------
    # Artifact Synthesis & Registration Checks
    # -------------------------------------------------------------------------

    def test_clean_landing_zone_passes(self):
        """Verify empty landing zone returns zero findings."""
        with tempfile.TemporaryDirectory() as tmpdir:
            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = StandardsAndMeasurementValidator()
            findings = validator.validate(repo)
            self.assertEqual(findings, [])

    def test_matrix_and_dictionary_synthesis(self):
        """Verify automated synthesis of baseline markdown artifacts."""
        with tempfile.TemporaryDirectory() as tmpdir:
            features_dir = os.path.join(tmpdir, "docs", "features")
            interfaces_dir = os.path.join(tmpdir, "docs", "interfaces")
            os.makedirs(features_dir, exist_ok=True)
            os.makedirs(interfaces_dir, exist_ok=True)

            with open(os.path.join(features_dir, "FEAT_01.md"), "w", encoding="utf-8") as f:
                f.write("/// Standard: [DO-178C, 'Table A-3', DAL-A]\n")

            with open(os.path.join(interfaces_dir, "ICD_02_MASTER_SIGNAL_DICTIONARY.md"), "w", encoding="utf-8") as f:
                f.write("""| Signal ID | Signal Name | Source Port | Dest Port | Data Type | SI Units | Valid Range | Update Rate | Safe Default Value | Schema Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `SIG-01` | PrimaryVelocity | `Nav.p1` | `FC.p2` | Float32 | `m/s` | `[-50.0, 50.0]` | `100 Hz` | `0.0` | `schema/model.sysml` |
""")

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = StandardsAndMeasurementValidator()

            std_baseline = validator.synthesize_standards_baseline(repo)
            self.assertIn("Standards Taxonomy Baseline", std_baseline)
            self.assertIn("DO-178C", std_baseline)
            self.assertIn("DAL-A", std_baseline)

            param_dict = validator.synthesize_parameter_dictionary(repo)
            self.assertIn("Parameter & Measurement Taxonomy Dictionary", param_dict)
            self.assertIn("PrimaryVelocity", param_dict)
            self.assertIn("m/s", param_dict)
            self.assertIn(r"\text{m} \cdot \text{s}^{-1}", param_dict)

    def test_aggregating_validators_registration(self):
        """Verify StandardsAndMeasurementValidator is registered in AGGREGATING_VALIDATORS."""
        self.assertIn(StandardsAndMeasurementValidator, AGGREGATING_VALIDATORS)


if __name__ == "__main__":
    unittest.main()
