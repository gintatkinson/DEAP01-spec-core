"""
Unit tests for Phase 2 Compiler & Domain Template Architecture Remediation.
Addresses Issues #174, #175, #177, #178, #179, #180.
"""

import json
import math
import os
import tempfile
import unittest

from scripts.assemble_conops import (
    CANONICAL_CONOPS_UNITS,
    CANONICAL_MISSION_INTENT_UNITS,
    SysMLParameterBindingEngine,
    assemble_conops,
    assemble_document,
    bind_parameters,
)


class TestDomainCompilerRemediation(unittest.TestCase):
    """Verifies domain routing, mass conservation, closed-form physics, energy conservation, and zero-domain invariants."""

    def test_domain_auto_detection(self):
        """Test domain auto-detection across all 6 domains (Issue #175)."""
        domain_cases = {
            "aviation": {"PLATFORM_TYPE": "Tactical ISR UAV", "OPERATIONAL_DOMAIN": "Airspace Operations"},
            "medical": {"PLATFORM_TYPE": "Surgical Robotic Console", "OPERATIONAL_DOMAIN": "Hospital Operating Room"},
            "rail": {"PLATFORM_TYPE": "Freight Shunting Locomotive", "OPERATIONAL_DOMAIN": "Railway Classification Yard"},
            "marine": {"PLATFORM_TYPE": "Subsea Autonomous Underwater Vehicle", "OPERATIONAL_DOMAIN": "Maritime Deep Sea"},
            "space": {"PLATFORM_TYPE": "LEO CubeSat Spacecraft", "OPERATIONAL_DOMAIN": "Orbital Space"},
            "industrial": {"PLATFORM_TYPE": "Autonomous Forklift AGV", "OPERATIONAL_DOMAIN": "Warehouse Logistics Floor"},
        }

        for expected_dom, params in domain_cases.items():
            engine = SysMLParameterBindingEngine(parameter_values=params, auto_detect=False)
            self.assertEqual(engine.detected_domain, expected_dom, f"Failed detecting {expected_dom} from {params}")

    def test_explicit_domain_parameter(self):
        """Test explicit domain override via SysMLParameterBindingEngine (Issue #175)."""
        for dom in ("aviation", "medical", "rail", "marine", "space", "industrial"):
            engine = SysMLParameterBindingEngine(domain=dom, auto_detect=False)
            self.assertEqual(engine.detected_domain, dom)
            self.assertEqual(engine.resolve_token("DOMAIN_TYPE"), dom)
            self.assertEqual(engine.resolve_token("DETECTED_DOMAIN"), dom)

    def test_mass_conservation_table_1_3_2(self):
        """Verify Table 1.3.2 Mass Conservation arithmetic parity (Issue #177)."""
        test_masses = [5.0, 25.0, 50.0, 150.0, 1200.0, 45000.0]

        for m in test_masses:
            engine = SysMLParameterBindingEngine(
                parameter_values={"TOTAL_MTOW_KG": str(m)},
                auto_detect=False,
            )

            # Check percentage fractions sum to 100.0%
            pct_airframe = float(engine.resolve_token("MASS_FRACTION_AIRFRAME_PCT"))
            pct_avionics = float(engine.resolve_token("MASS_FRACTION_AVIONICS_PCT"))
            pct_propulsion = float(engine.resolve_token("MASS_FRACTION_PROPULSION_PCT"))
            pct_energy = float(engine.resolve_token("MASS_FRACTION_ENERGY_PCT"))
            pct_payload = float(engine.resolve_token("MASS_FRACTION_PAYLOAD_PCT"))
            pct_containment = float(engine.resolve_token("MASS_FRACTION_CONTAINMENT_PCT"))

            pct_sum = round(pct_airframe + pct_avionics + pct_propulsion + pct_energy + pct_payload + pct_containment, 1)
            self.assertEqual(pct_sum, 100.0, f"Mass percentage fractions do not sum to 100.0%: {pct_sum}")

            # Check mass budget sum strictly equals MTOW
            kg_airframe = float(engine.resolve_token("MASS_BUDGET_AIRFRAME_KG"))
            kg_avionics = float(engine.resolve_token("MASS_BUDGET_AVIONICS_KG"))
            kg_propulsion = float(engine.resolve_token("MASS_BUDGET_PROPULSION_KG"))
            kg_energy = float(engine.resolve_token("MASS_BUDGET_ENERGY_KG"))
            kg_payload = float(engine.resolve_token("MASS_BUDGET_PAYLOAD_KG"))
            kg_containment = float(engine.resolve_token("MASS_BUDGET_CONTAINMENT_KG"))

            kg_sum = round(kg_airframe + kg_avionics + kg_propulsion + kg_energy + kg_payload + kg_containment, 2)
            self.assertAlmostEqual(kg_sum, m, places=2, msg=f"Subsystem masses {kg_sum} != MTOW {m}")

    def test_closed_form_quadratic_physics_solver(self):
        """Verify Section 5.2 closed-form quadratic physics solver for medium densities (Issue #178)."""
        # Aviation (air ISA rho = 1.225)
        eng_air = SysMLParameterBindingEngine(domain="aviation", parameter_values={"TOTAL_MTOW_KG": "50.0"}, auto_detect=False)
        self.assertAlmostEqual(float(eng_air.resolve_token("AIR_DENSITY_KGM3")), 1.225, places=3)
        v_unmit_air = float(eng_air.resolve_token("V_TERMINAL_UNMITIGATED_MPS"))
        ek_unmit_air = float(eng_air.resolve_token("E_K_UNMITIGATED_JOULES"))
        # Verify formula parity: v = sqrt(2mg / (rho * S * Cd))
        expected_v_air = round(math.sqrt((2.0 * 50.0 * 9.80665) / (1.225 * 0.18 * 0.45)), 2)
        expected_ek_air = round(0.5 * 50.0 * (v_unmit_air ** 2), 1)
        self.assertAlmostEqual(v_unmit_air, expected_v_air, places=1)
        self.assertAlmostEqual(ek_unmit_air, expected_ek_air, places=1)

        # Marine (seawater rho = 1025.0)
        eng_marine = SysMLParameterBindingEngine(domain="marine", parameter_values={"TOTAL_MTOW_KG": "250.0"}, auto_detect=False)
        self.assertAlmostEqual(float(eng_marine.resolve_token("FLUID_DENSITY_KGM3")), 1025.0, places=1)
        v_unmit_mar = float(eng_marine.resolve_token("V_TERMINAL_UNMITIGATED_MPS"))
        expected_v_mar = round(math.sqrt((2.0 * 250.0 * 9.80665) / (1025.0 * 0.18 * 0.45)), 2)
        self.assertAlmostEqual(v_unmit_mar, expected_v_mar, places=1)

        # Space (vacuum rho = 1e-12)
        eng_space = SysMLParameterBindingEngine(domain="space", parameter_values={"TOTAL_MTOW_KG": "12.0"}, auto_detect=False)
        self.assertEqual(float(eng_space.resolve_token("RHO_MEDIUM")), 1.0e-12)

    def test_first_law_energy_conservation(self):
        """Verify First-Law Energy Conservation: E_capacity >= P_nominal * t_endurance (Issue #179)."""
        test_endurances = [1.0, 2.5, 5.0, 10.0]

        for t_h in test_endurances:
            engine = SysMLParameterBindingEngine(
                parameter_values={
                    "TOTAL_POWER_NOMINAL_W": "400.0",
                    "ENDURANCE_HOURS": str(t_h),
                },
                auto_detect=False,
            )

            e_joules = float(engine.resolve_token("BATTERY_CAPACITY_JOULES"))
            p_nom = float(engine.resolve_token("TOTAL_POWER_NOMINAL_W"))
            t_sec = t_h * 3600.0

            # First Law: Total energy must equal or exceed nominal power integrated over endurance
            self.assertGreaterEqual(
                e_joules,
                p_nom * t_sec,
                f"Battery energy {e_joules} J is less than nominal power requirement {p_nom * t_sec} J",
            )

            # Statutory reserve ratio must be >= 20%
            e_reserve = float(engine.resolve_token("E_RESERVE_JOULES"))
            reserve_ratio = e_reserve / e_joules
            self.assertAlmostEqual(reserve_ratio, 0.20, places=2)

            # Power partitions must sum to nominal power
            p_prop = float(engine.resolve_token("POWER_NOMINAL_PROPULSION_W"))
            p_avionics = float(engine.resolve_token("POWER_NOMINAL_AVIONICS_W"))
            p_payload = float(engine.resolve_token("POWER_NOMINAL_PAYLOAD_W"))
            p_containment = float(engine.resolve_token("POWER_NOMINAL_CONTAINMENT_W"))
            p_sum = round(p_prop + p_avionics + p_payload + p_containment, 1)
            self.assertAlmostEqual(p_sum, p_nom, places=1)

    def test_zero_domain_invariants_medical(self):
        """Verify Medical domain has ZERO Remote ID, ZERO landing zones, ZERO parachutes, ZERO combat ROEs (Issues #174, #175)."""
        engine = SysMLParameterBindingEngine(
            domain="medical",
            parameter_values={"SYSTEM_IDENTIFIER": "SurgicalConsoleX1"},
            auto_detect=False,
        )
        sample_text = (
            "The system deploys a parachute in the emergency landing zone following Remote ID verification. "
            "Under ROE-01, the system authorizes weapons release with minimal collateral damage based on PID."
        )
        bound = engine.substitute(sample_text)

        self.assertNotIn("parachute", bound.lower())
        self.assertNotIn("remote id", bound.lower())
        self.assertNotIn("landing zone", bound.lower())
        self.assertNotIn("roe-01", bound.lower())
        self.assertNotIn("weapons release", bound.lower())
        self.assertNotIn("collateral damage", bound.lower())
        self.assertIn("SAF-01", bound)
        self.assertIn("PCV", bound)
        self.assertIn("sterile field docking zone", bound.lower())
        self.assertIn("failsafe joint brake", bound.lower())

    def test_zero_domain_invariants_rail(self):
        """Verify Rail domain has ZERO Remote ID, ZERO landing zones, ZERO parachutes, ZERO flight plans (Issues #174, #175)."""
        engine = SysMLParameterBindingEngine(
            domain="rail",
            parameter_values={"SYSTEM_IDENTIFIER": "ShuntingLocomotive9000"},
            auto_detect=False,
        )
        sample_text = (
            "The system activates a parachute at the landing zone and checks the flight plan with Remote ID broadcast."
        )
        bound = engine.substitute(sample_text)

        self.assertNotIn("parachute", bound.lower())
        self.assertNotIn("landing zone", bound.lower())
        self.assertNotIn("flight plan", bound.lower())
        self.assertNotIn("remote id", bound.lower())
        self.assertIn("pneumatic emergency brake", bound.lower())

    def test_zero_domain_invariants_marine(self):
        """Verify Marine domain has ZERO air parachutes, ZERO 5.8 GHz WiFi underwater (Issues #174, #175)."""
        engine = SysMLParameterBindingEngine(
            domain="marine",
            parameter_values={"SYSTEM_IDENTIFIER": "SubseaAUVExplorer"},
            auto_detect=False,
        )
        sample_text = (
            "Under emergency descent, the vehicle deploys a parachute and establishes 5.8 GHz Wi-Fi communications at 100 m AGL."
        )
        bound = engine.substitute(sample_text)

        self.assertNotIn("parachute", bound.lower())
        self.assertNotIn("5.8 ghz wi-fi", bound.lower())
        self.assertNotIn("m agl", bound.lower())
        self.assertIn("drop-weight", bound.lower())
        self.assertIn("acoustic modem", bound.lower())
        self.assertIn("m depth", bound.lower())

    def test_zero_domain_invariants_space(self):
        """Verify Space domain has ZERO blowing rain, ZERO parachutes in vacuum (Issues #174, #175)."""
        engine = SysMLParameterBindingEngine(
            domain="space",
            parameter_values={"SYSTEM_IDENTIFIER": "LEOCubeSatSat1"},
            auto_detect=False,
        )
        sample_text = (
            "In safe mode, the spacecraft deploys a parachute to descend from 500 m AGL."
        )
        bound = engine.substitute(sample_text)

        self.assertNotIn("parachute", bound.lower())
        self.assertNotIn("m agl", bound.lower())
        self.assertIn("de-orbit", bound.lower())

    def test_zero_domain_invariants_industrial(self):
        """Verify Industrial domain has ZERO flight plans, ZERO combat ROEs (Issues #174, #175)."""
        engine = SysMLParameterBindingEngine(
            domain="industrial",
            parameter_values={"SYSTEM_IDENTIFIER": "WarehouseForkliftAGV"},
            auto_detect=False,
        )
        sample_text = (
            "The AGV follows the flight plan under ROE-02 rules of engagement."
        )
        bound = engine.substitute(sample_text)

        self.assertNotIn("flight plan", bound.lower())
        self.assertNotIn("roe-02", bound.lower())
        self.assertIn("SAF-02", bound)
        self.assertIn("warehouse route order", bound.lower())

    def test_residual_archetype_string_elimination(self):
        """Verify complete elimination of residual archetype strings (Issue #180)."""
        engine = SysMLParameterBindingEngine(
            parameter_values={"SYSTEM_IDENTIFIER": "TacticalScoutUAV"},
            auto_detect=False,
        )
        sample_text = (
            "This document defines the requirements for the Abstract Cyber-Physical System Archetype. "
            "The Abstract Cyber-Physical System Archetype is designed for high tempo operations."
        )
        bound = engine.substitute(sample_text)

        self.assertNotIn("Abstract Cyber-Physical System Archetype", bound)
        self.assertIn("the TacticalScoutUAV", bound)
    def test_end_to_end_assembly_with_domain_routing(self):
        """Verify assemble_conops end-to-end assembly across domains with domain parameter routing."""
        canonical_units_dir = os.path.join(
            os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
            "skills",
            "spec-conops-engineering",
            "resources",
            "units",
        )

        with tempfile.TemporaryDirectory() as tmpdir:
            out_dir = os.path.join(tmpdir, "docs_conops")
            for dom in ("aviation", "medical", "rail", "marine", "space", "industrial"):
                success = assemble_conops(
                    input_dir=canonical_units_dir,
                    output_dir=out_dir,
                    verify_only=False,
                    params={"SYSTEM_IDENTIFIER": f"TestSystem_{dom.capitalize()}"},
                    domain=dom,
                )
                self.assertTrue(success, f"assemble_conops failed for domain: {dom}")

                conops_file = os.path.join(out_dir, "CONOPS.md")
                self.assertTrue(os.path.isfile(conops_file))
                with open(conops_file, "r", encoding="utf-8") as f:
                    conops_text = f.read()

                # Zero residual archetype
                self.assertNotIn("the Abstract Cyber-Physical System Archetype", conops_text)
                self.assertNotIn("The Abstract Cyber-Physical System Archetype", conops_text)

                # Domain checks
                if dom == "medical":
                    self.assertNotIn("Remote ID", conops_text)
                    self.assertNotIn("parachute", conops_text.lower())
                elif dom == "rail":
                    self.assertNotIn("Remote ID", conops_text)
                    self.assertNotIn("parachute", conops_text.lower())
                elif dom == "marine":
                    self.assertNotIn("5.8 GHz Wi-Fi", conops_text)
                    self.assertNotIn("parachute", conops_text.lower())
                elif dom == "space":
                    self.assertNotIn("parachute", conops_text.lower())
                elif dom == "industrial":
                    self.assertNotIn("flight plan", conops_text.lower())


if __name__ == "__main__":
    unittest.main()

