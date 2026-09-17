#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Unit tests for KerML / SysML v2 Python AST & Semantic Unit Pipeline.
Tests kerml_compiler.py components, lexer/parser, unit registry, and two-pass compiler lifecycle.
"""

import math
import os
import sys
import tempfile
import unittest

# Add skills/spec-orchestrator/scripts to sys.path
scripts_dir = os.path.abspath(
    os.path.join(os.path.dirname(__file__), "..", "skills", "spec-orchestrator", "scripts")
)
if scripts_dir not in sys.path:
    sys.path.insert(0, scripts_dir)

from kerml_compiler import (
    DiagnosticSeverity,
    SourceLocation,
    CompilerDiagnostic,
    Dimension,
    UnitDefinition,
    DomainContract,
    UnitRegistry,
    STANDARD_SI_UNITS,
    AVIATION_UNITS,
    MARINE_UNITS,
    RAIL_UNITS,
    MEDICAL_UNITS,
    SPACE_UNITS,
    INDUSTRIAL_UNITS,
    DOMAIN_PACKAGES,
    DOMAIN_CONTRACTS,
    KerMLTokenizer,
    KerMLParser,
    MetadataHarvesterVisitor,
    SemanticBindingVisitor,
    SysMLv2CompilerDriver,
)


class TestKerMLCompilerStructures(unittest.TestCase):
    """Test diagnostics, dimension mathematics, and unit definition data structures."""

    def test_diagnostic_structures(self):
        loc = SourceLocation(line=10, column=5, source_file="model.sysml")
        self.assertEqual(str(loc), "model.sysml:10:5")
        self.assertEqual(loc.to_dict(), {"line": 10, "column": 5, "source_file": "model.sysml"})

        diag = CompilerDiagnostic(
            severity=DiagnosticSeverity.ERROR,
            message="Test error",
            location=loc,
        )
        self.assertEqual(diag.severity, DiagnosticSeverity.ERROR)
        self.assertIn("ERROR", str(diag))
        self.assertIn("model.sysml:10:5", str(diag))
        self.assertIn("Test error", str(diag))

    def test_dimension_algebra(self):
        d_mass = Dimension(mass=1)
        d_length = Dimension(length=1)
        d_time = Dimension(time=1)

        # Force = mass * length * time^-2
        d_force = d_mass.multiply(d_length).divide(d_time.power(2))
        self.assertEqual(d_force, Dimension(mass=1, length=1, time=-2))

        # Energy = force * length = mass * length^2 * time^-2
        d_energy = d_force.multiply(d_length)
        self.assertEqual(d_energy, Dimension(mass=1, length=2, time=-2))

        # Power = energy / time = mass * length^2 * time^-3
        d_power = d_energy.divide(d_time)
        self.assertEqual(d_power, Dimension(mass=1, length=2, time=-3))

        # Dimensionless
        self.assertTrue(Dimension().is_dimensionless())
        self.assertFalse(d_mass.is_dimensionless())

    def test_standard_si_units(self):
        registry = UnitRegistry(include_standard_si=True)

        expected_si = [
            ("kg", Dimension(mass=1), 1.0),
            ("m", Dimension(length=1), 1.0),
            ("s", Dimension(time=1), 1.0),
            ("m_s", Dimension(length=1, time=-1), 1.0),
            ("m_s2", Dimension(length=1, time=-2), 1.0),
            ("N", Dimension(mass=1, length=1, time=-2), 1.0),
            ("J", Dimension(mass=1, length=2, time=-2), 1.0),
            ("W", Dimension(mass=1, length=2, time=-3), 1.0),
            ("Pa", Dimension(mass=1, length=-1, time=-2), 1.0),
            ("A", Dimension(current=1), 1.0),
            ("K", Dimension(temperature=1), 1.0),
            ("rad", Dimension(), 1.0),
            ("deg", Dimension(), math.pi / 180.0),
        ]

        for name, dim, scale in expected_si:
            unit = registry.resolve(name)
            self.assertIsNotNone(unit, f"SI unit '{name}' not resolved")
            self.assertEqual(unit.dimension, dim, f"Dimension mismatch for SI unit '{name}'")
            self.assertAlmostEqual(unit.scale_to_si, scale, places=5, msg=f"Scale mismatch for SI unit '{name}'")

    def test_domain_packages_definitions(self):
        # 1. Aviation units
        self.assertAlmostEqual(AVIATION_UNITS["knots"].scale_to_si, 0.514444, places=5)
        self.assertAlmostEqual(AVIATION_UNITS["ft"].scale_to_si, 0.3048, places=4)
        self.assertAlmostEqual(AVIATION_UNITS["slug"].scale_to_si, 14.5939, places=4)
        self.assertAlmostEqual(AVIATION_UNITS["ft_min"].scale_to_si, 0.00508, places=5)
        self.assertAlmostEqual(AVIATION_UNITS["mach"].scale_to_si, 340.29, places=2)
        self.assertAlmostEqual(AVIATION_UNITS["deg_s"].scale_to_si, math.pi / 180.0, places=5)

        # 2. Marine units
        self.assertAlmostEqual(MARINE_UNITS["nmi"].scale_to_si, 1852.0, places=1)
        self.assertAlmostEqual(MARINE_UNITS["knots"].scale_to_si, 0.514444, places=5)
        self.assertAlmostEqual(MARINE_UNITS["fathom"].scale_to_si, 1.8288, places=4)
        self.assertAlmostEqual(MARINE_UNITS["bar"].scale_to_si, 100000.0, places=1)
        self.assertAlmostEqual(MARINE_UNITS["m_s"].scale_to_si, 1.0, places=5)

        # 3. Rail units
        self.assertAlmostEqual(RAIL_UNITS["km_h"].scale_to_si, 0.277778, places=5)
        self.assertAlmostEqual(RAIL_UNITS["ton"].scale_to_si, 1000.0, places=1)
        self.assertAlmostEqual(RAIL_UNITS["kN"].scale_to_si, 1000.0, places=1)
        self.assertAlmostEqual(RAIL_UNITS["m_s2"].scale_to_si, 1.0, places=5)
        self.assertAlmostEqual(RAIL_UNITS["mm"].scale_to_si, 0.001, places=5)

        # 4. Medical units
        self.assertAlmostEqual(MEDICAL_UNITS["mmHg"].scale_to_si, 133.322, places=3)
        self.assertAlmostEqual(MEDICAL_UNITS["ml_min"].scale_to_si, 1.66667e-8, places=10)
        self.assertAlmostEqual(MEDICAL_UNITS["mm"].scale_to_si, 0.001, places=5)
        self.assertAlmostEqual(MEDICAL_UNITS["N_cm"].scale_to_si, 0.01, places=5)
        self.assertAlmostEqual(MEDICAL_UNITS["deg"].scale_to_si, math.pi / 180.0, places=5)

        # 5. Space units
        self.assertAlmostEqual(SPACE_UNITS["km_s"].scale_to_si, 1000.0, places=1)
        self.assertAlmostEqual(SPACE_UNITS["AU"].scale_to_si, 1.495978707e11, places=1)
        self.assertAlmostEqual(SPACE_UNITS["arcsec"].scale_to_si, 4.8481368e-6, places=10)
        self.assertAlmostEqual(SPACE_UNITS["uW"].scale_to_si, 1e-6, places=9)
        self.assertAlmostEqual(SPACE_UNITS["N_s"].scale_to_si, 1.0, places=5)

        # 6. Industrial units
        self.assertAlmostEqual(INDUSTRIAL_UNITS["rpm"].scale_to_si, 0.10472, places=5)
        self.assertAlmostEqual(INDUSTRIAL_UNITS["Nm"].scale_to_si, 1.0, places=5)
        self.assertAlmostEqual(INDUSTRIAL_UNITS["m_min"].scale_to_si, 0.0166667, places=5)
        self.assertAlmostEqual(INDUSTRIAL_UNITS["deg_s"].scale_to_si, math.pi / 180.0, places=5)


class TestKerMLTokenizerAndParser(unittest.TestCase):
    """Test lexical and syntactic parsing of KerML source text."""

    def test_lexer_tokens(self):
        src = """
        // Header comment
        /* Multi-line
           comment */
        metadata def DomainMetadata {
            attribute domainId : String;
        }
        """
        tok = KerMLTokenizer(src)
        tokens = tok.tokenize()
        types = [t.type for t in tokens]
        self.assertIn("KEYWORD", types)
        self.assertIn("ID", types)
        self.assertIn("LBRACE", types)
        self.assertIn("RBRACE", types)
        self.assertIn("SEMI", types)
        self.assertEqual(tokens[-1].type, "EOF")

    def test_parser_complete_model(self):
        src = """
        metadata def DomainMetadata {
            attribute domainId : String;
            attribute activeFrameworks : String;
        }

        @DomainMetadata {
            domainId = "aviation";
            activeFrameworks = ("DO-178C", "DO-254");
        }
        package AircraftSystem {
            part def FlightComputer {
                attribute cruiseAirspeed : Real = 250.0 [knots];
                attribute ceiling : Real = 45000.0 [ft];
                attribute mass : Real = 150.0 [kg];
            }
        }
        """
        driver = SysMLv2CompilerDriver()
        diags, ast_dict = driver.compile(src, source_file="aircraft.sysml")

        errors = [d for d in diags if d.severity == DiagnosticSeverity.ERROR]
        self.assertEqual(len(errors), 0, f"Expected 0 errors, got: {errors}")
        self.assertIn("elements", ast_dict)
        self.assertIn("loaded_unit_packages", ast_dict["metadata"])
        self.assertIn("aviation_units", ast_dict["metadata"]["loaded_unit_packages"])


class TestTwoPassCompilerLifecycle(unittest.TestCase):
    """Test Pass 1 (Metadata harvesting) and Pass 2 (Semantic unit resolution)."""

    def test_pass_1_missing_domain_id_error(self):
        src = """
        @DomainMetadata {
            activeFrameworks = ("DO-178C");
        }
        package InvalidAviationSystem {
            part def Controller {
                attribute speed : Real [knots];
            }
        }
        """
        driver = SysMLv2CompilerDriver()
        diags, _ = driver.compile(src, source_file="invalid.sysml")

        err_messages = [d.message for d in diags if d.severity == DiagnosticSeverity.ERROR]
        self.assertTrue(any("missing mandatory 'domainId'" in m for m in err_messages))
        # Since domain was not harvested, knots should also fail to resolve
        self.assertTrue(any("Unresolved physical unit '[knots]'" in m for m in err_messages))

    def test_pass_2_unresolved_physical_unit_error_message(self):
        src = """
        @DomainMetadata {
            domainId = "aviation";
            activeFrameworks = ("DO-178C");
        }
        package Aircraft {
            part def Avionics {
                attribute altitude : Real [ft];
                attribute bloodPressure : Real [mmHg];
            }
        }
        """
        driver = SysMLv2CompilerDriver()
        diags, _ = driver.compile(src, source_file="avionics.sysml")

        err_messages = [d.message for d in diags if d.severity == DiagnosticSeverity.ERROR]
        expected_err = "Unresolved physical unit '[mmHg]' declared on attribute 'bloodPressure'. Verify that the appropriate domain library is imported."
        self.assertIn(expected_err, err_messages)

    def test_all_six_domain_contracts_compilation(self):
        domains = [
            ("aviation", "knots", "ft"),
            ("marine", "nmi", "fathom"),
            ("rail", "km_h", "ton"),
            ("medical", "mmHg", "N_cm"),
            ("space", "km_s", "AU"),
            ("industrial", "rpm", "m_min"),
        ]

        for domain_id, unit1, unit2 in domains:
            src = f"""
            @DomainMetadata(
                domainId = "{domain_id}",
                activeFrameworks = ["STD-001"]
            )
            package TestPackage_{domain_id} {{
                part def SystemBlock {{
                    attribute val1 : Real = 10.0 [{unit1}];
                    attribute val2 : Real = 20.0 [{unit2}];
                    attribute standardMass : Real = 5.0 [kg];
                }}
            }}
            """
            driver = SysMLv2CompilerDriver()
            diags, ast_dict = driver.compile(src, source_file=f"{domain_id}.sysml")

            errors = [d for d in diags if d.severity == DiagnosticSeverity.ERROR]
            self.assertEqual(len(errors), 0, f"Domain {domain_id} failed compilation with errors: {errors}")
            self.assertIn(f"{domain_id}_units", ast_dict["metadata"]["loaded_unit_packages"])

    def test_file_compilation(self):
        src = """
        @DomainMetadata {
            domainId = "space";
            activeFrameworks = ("ECSS-E-ST-40C");
        }
        package Spacecraft {
            part def Thruster {
                attribute impulse : Real [N_s];
                attribute standbyPower : Real [uW];
            }
        }
        """
        with tempfile.NamedTemporaryFile("w", suffix=".sysml", delete=False) as f:
            f.write(src)
            tmp_path = f.name

        try:
            driver = SysMLv2CompilerDriver()
            diags, ast_dict = driver.compile_file(tmp_path)
            errors = [d for d in diags if d.severity == DiagnosticSeverity.ERROR]
            self.assertEqual(len(errors), 0)
            self.assertEqual(ast_dict["metadata"]["source_file"], tmp_path)
        finally:
            if os.path.exists(tmp_path):
                os.remove(tmp_path)


if __name__ == "__main__":
    unittest.main()
