"""
Unit tests for Phase 2 Compiler & Domain Template Architecture Remediation.
Addresses Issues #174, #175, #177, #178, #179, #180, #206.
"""

import math
import os
import tempfile
import unittest

from scripts.assemble_conops import (
    SysMLParameterBindingEngine,
    assemble_conops,
    LifecycleType,
    ContainmentActionType,
    LifecycleContract,
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
        # Verify formula parity: v = sqrt(2mg / (rho * S * Cd)) and Ek = 0.5 * m * v^2
        expected_v_air = round(math.sqrt((2.0 * 50.0 * 9.80665) / (1.225 * 0.18 * 0.45)), 2)
        expected_ek_air = round(0.5 * 50.0 * (expected_v_air ** 2), 1)
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
            for dom in ("aviation", "medical", "rail", "marine", "space", "industrial"):
                out_dir = os.path.join(tmpdir, f"docs_conops_{dom}")
                success = assemble_conops(
                    input_dir=canonical_units_dir,
                    output_dir=out_dir,
                    verify_only=False,
                    params={"SYSTEM_IDENTIFIER": f"TestSystem_{dom.capitalize()}"},
                    domain=dom,
                )
                self.assertTrue(success, f"assemble_conops failed for domain: {dom}")

                for doc_name in ("CONOPS.md", "MISSION_INTENT.md"):
                    doc_path = os.path.join(out_dir, doc_name)
                    self.assertTrue(os.path.isfile(doc_path), f"Missing {doc_name} for domain {dom}")
                    with open(doc_path, "r", encoding="utf-8") as f:
                        doc_text = f.read()

                    # Zero residual archetype
                    self.assertNotIn("the Abstract Cyber-Physical System Archetype", doc_text)
                    self.assertNotIn("The Abstract Cyber-Physical System Archetype", doc_text)

                conops_file = os.path.join(out_dir, "CONOPS.md")
                with open(conops_file, "r", encoding="utf-8") as f:
                    conops_text = f.read()

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

    def test_cross_domain_state_vectors_and_anti_plagiarism(self):
        """Verify cross-domain state space parameterization and anti-plagiarism isolation (Fixes #165, #174, #175, #176)."""
        domains = ("aviation", "medical", "rail", "marine", "space", "industrial")

        state_vector_min_set = set()
        state_vector_max_set = set()
        state_space_standards_set = set()
        safety_bounds_standards_set = set()

        canonical_units_dir = os.path.join(
            os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
            "skills",
            "spec-conops-engineering",
            "resources",
            "units",
        )

        domain_expected_tokens = {
            "aviation": {
                "STATE_SPACE_STANDARD": "ISO/IEC/IEEE 29148:2018 §6.4.2",
                "SAFETY_BOUNDS_STANDARD": "ASTM F3269-17 §6.2",
                "STATE_VECTOR_MIN_EXPRESSION": "[phi_min, lambda_min, h_min, u_min, v_min, w_min]^T",
                "STATE_VECTOR_MAX_EXPRESSION": "[phi_max, lambda_max, h_max, u_max, v_max, w_max]^T",
                "STATE_VECTOR_MIN_UNITS": "rad, rad, m, m/s, m/s, m/s",
                "STATE_VECTOR_MAX_UNITS": "rad, rad, m, m/s, m/s, m/s",
                "CONTAINMENT_BUFFER_UNIT": "m",
            },
            "medical": {
                "STATE_SPACE_STANDARD": "IEC 62304:2006+AMD1:2015 §5.2",
                "SAFETY_BOUNDS_STANDARD": "IEC 60601-1-8:2020 §6.9",
                "STATE_VECTOR_MIN_EXPRESSION": "[x_min, y_min, z_min, vx_min, vy_min, vz_min]^T",
                "STATE_VECTOR_MAX_EXPRESSION": "[x_max, y_max, z_max, vx_max, vy_max, vz_max]^T",
                "STATE_VECTOR_MIN_UNITS": "mm, mm, mm, mm/s, mm/s, mm/s",
                "STATE_VECTOR_MAX_UNITS": "mm, mm, mm, mm/s, mm/s, mm/s",
                "CONTAINMENT_BUFFER_UNIT": "mm",
            },
            "rail": {
                "STATE_SPACE_STANDARD": "EN 50126:2017 §6.2",
                "SAFETY_BOUNDS_STANDARD": "EN 50128:2011/A2:2020 SIL 4 §6.3",
                "STATE_VECTOR_MIN_EXPRESSION": "[s_min, v_min, a_min, p_brake_min]^T",
                "STATE_VECTOR_MAX_EXPRESSION": "[s_max, v_max, a_max, p_brake_max]^T",
                "STATE_VECTOR_MIN_UNITS": "m, m/s, m/s^2, bar",
                "STATE_VECTOR_MAX_UNITS": "m, m/s, m/s^2, bar",
                "CONTAINMENT_BUFFER_UNIT": "m",
            },
            "marine": {
                "STATE_SPACE_STANDARD": "DNV-GL-ST-E403 §3.2",
                "SAFETY_BOUNDS_STANDARD": "ISO 13628-6 §6.3",
                "STATE_VECTOR_MIN_EXPRESSION": "[x_north_min, y_east_min, z_depth_min, u_surge_min, v_sway_min, w_heave_min]^T",
                "STATE_VECTOR_MAX_EXPRESSION": "[x_north_max, y_east_max, z_depth_max, u_surge_max, v_sway_max, w_heave_max]^T",
                "STATE_VECTOR_MIN_UNITS": "m, m, m Depth, m/s, m/s, m/s",
                "STATE_VECTOR_MAX_UNITS": "m, m, m Depth, m/s, m/s, m/s",
                "CONTAINMENT_BUFFER_UNIT": "m Depth",
            },
            "space": {
                "STATE_SPACE_STANDARD": "ECSS-E-ST-10C §5.2",
                "SAFETY_BOUNDS_STANDARD": "ECSS-E-ST-40C §6.3",
                "STATE_VECTOR_MIN_EXPRESSION": "[r_x_min, r_y_min, r_z_min, v_x_min, v_y_min, v_z_min]^T",
                "STATE_VECTOR_MAX_EXPRESSION": "[r_x_max, r_y_max, r_z_max, v_x_max, v_y_max, v_z_max]^T",
                "STATE_VECTOR_MIN_UNITS": "km, km, km, km/s, km/s, km/s",
                "STATE_VECTOR_MAX_UNITS": "km, km, km, km/s, km/s, km/s",
                "CONTAINMENT_BUFFER_UNIT": "km Orbital Altitude",
            },
            "industrial": {
                "STATE_SPACE_STANDARD": "ISO 3691-4:2023 §4.2",
                "SAFETY_BOUNDS_STANDARD": "IEC 61508 SIL 3 Part 2 §7.4",
                "STATE_VECTOR_MIN_EXPRESSION": "[x_grid_min, y_grid_min, theta_yaw_min, v_trans_min, omega_rot_min, h_fork_min]^T",
                "STATE_VECTOR_MAX_EXPRESSION": "[x_grid_max, y_grid_max, theta_yaw_max, v_trans_max, omega_rot_max, h_fork_max]^T",
                "STATE_VECTOR_MIN_UNITS": "m, m, rad, m/s, rad/s, m",
                "STATE_VECTOR_MAX_UNITS": "m, m, rad, m/s, rad/s, m",
                "CONTAINMENT_BUFFER_UNIT": "m",
            },
        }

        with tempfile.TemporaryDirectory() as tmpdir:
            for dom in domains:
                engine = SysMLParameterBindingEngine(domain=dom, auto_detect=False)

                # Verify each expected token is present and matches
                expected = domain_expected_tokens[dom]
                for key, val in expected.items():
                    resolved_val = engine.resolve_token(key)
                    self.assertTrue(resolved_val, f"Empty token {key} for domain {dom}")
                    self.assertEqual(resolved_val, val, f"Mismatch in {key} for domain {dom}")

                state_vector_min_set.add(engine.resolve_token("STATE_VECTOR_MIN_EXPRESSION"))
                state_vector_max_set.add(engine.resolve_token("STATE_VECTOR_MAX_EXPRESSION"))
                state_space_standards_set.add(engine.resolve_token("STATE_SPACE_STANDARD"))
                safety_bounds_standards_set.add(engine.resolve_token("SAFETY_BOUNDS_STANDARD"))

                # Test assembly from canonical units
                out_dir = os.path.join(tmpdir, f"out_{dom}")
                success = assemble_conops(
                    input_dir=canonical_units_dir,
                    output_dir=out_dir,
                    verify_only=False,
                    domain=dom,
                )
                self.assertTrue(success, f"assemble_conops failed for {dom}")

                conops_path = os.path.join(out_dir, "CONOPS.md")
                with open(conops_path, "r", encoding="utf-8") as f:
                    conops_content = f.read()

                # Verify Table 1.3 in assembled document contains the domain-specific standards and state vectors
                self.assertIn(expected["STATE_SPACE_STANDARD"], conops_content)
                self.assertIn(expected["SAFETY_BOUNDS_STANDARD"], conops_content)
                self.assertIn(expected["STATE_VECTOR_MIN_EXPRESSION"], conops_content)
                self.assertIn(expected["STATE_VECTOR_MAX_EXPRESSION"], conops_content)
                self.assertIn(expected["STATE_VECTOR_MIN_UNITS"], conops_content)
                self.assertIn(expected["STATE_VECTOR_MAX_UNITS"], conops_content)

                # Anti-plagiarism isolation: Verify other domains' unique standards are not present in this domain
                for other_dom, other_expected in domain_expected_tokens.items():
                    if other_dom != dom:
                        self.assertNotIn(
                            other_expected["STATE_VECTOR_MIN_EXPRESSION"],
                            conops_content,
                            f"Domain {dom} contains state vector min expression from {other_dom}",
                        )

        # Mutually distinct state vector expressions across all 6 domains
        self.assertEqual(len(state_vector_min_set), 6, f"Expected 6 distinct state vector min expressions, got {len(state_vector_min_set)}")
        self.assertEqual(len(state_vector_max_set), 6, f"Expected 6 distinct state vector max expressions, got {len(state_vector_max_set)}")
        # Mutually distinct standards across all 6 domains
        self.assertEqual(len(state_space_standards_set), 6, f"Expected 6 distinct state space standards, got {len(state_space_standards_set)}")
        self.assertEqual(len(safety_bounds_standards_set), 6, f"Expected 6 distinct safety bounds standards, got {len(safety_bounds_standards_set)}")

    def test_lifecycle_archetype_expendable_kinetic_effector(self):
        """Verify EXPENDABLE_KINETIC_EFFECTOR archetype: 0 RTB, 0 runway landing; enforces terminal intercept / ditching zeroization."""
        canonical_units_dir = os.path.join(
            os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
            "skills",
            "spec-conops-engineering",
            "resources",
            "units",
        )
        params = {
            "SYSTEM_IDENTIFIER": "SkyShield_Kinetic_Interceptor",
            "PLATFORM_TYPE": "Delta-Wing High-G Solid-Rocket Interceptor UAS",
            "OPERATIONAL_DOMAIN": "Defensive Counter-UAS Kinetic Intercept",
            "IS_EXPENDABLE": "true",
            "PAYLOAD_TYPE": "Kinetic Warhead Effector",
            "TOTAL_MTOW_KG": "12.0",
        }
        engine = SysMLParameterBindingEngine(parameter_values=params, auto_detect=False)
        contract = engine._derive_lifecycle_contract()

        self.assertEqual(contract.lifecycle_type, LifecycleType.EXPENDABLE_KINETIC_EFFECTOR)
        self.assertEqual(contract.containment_action, ContainmentActionType.SAFE_IMPACT_ZEROIZATION)
        self.assertEqual(engine.resolve_token("LIFECYCLE_TYPE"), "EXPENDABLE_KINETIC_EFFECTOR")
        self.assertEqual(engine.resolve_token("LIFECYCLE_TRANSIT_MODE"), "Terminal_Engagement_Transit")
        self.assertIn("zeroization", engine.resolve_token("LIFECYCLE_BINGO_SAFETY_ACTION").lower())
        self.assertIn("zeroization", engine.resolve_token("LIFECYCLE_END_STATE").lower())

        with tempfile.TemporaryDirectory() as tmpdir:
            out_dir = os.path.join(tmpdir, "out_expendable")
            success = assemble_conops(
                input_dir=canonical_units_dir,
                output_dir=out_dir,
                verify_only=False,
                params=params,
            )
            self.assertTrue(success)

            with open(os.path.join(out_dir, "CONOPS.md"), "r", encoding="utf-8") as f:
                conops_txt = f.read()
            with open(os.path.join(out_dir, "MISSION_INTENT.md"), "r", encoding="utf-8") as f:
                intent_txt = f.read()

            combined = conops_txt + "\n" + intent_txt

            # Verify 0 RTB / return-to-base in key lifecycle fields
            self.assertNotIn("autonomous return-to-base (rtb)", combined.lower())
            self.assertNotIn("autonomous return-to-base sequence", combined.lower())
            self.assertNotIn("civilian runway landing", combined.lower())
            self.assertIn("zeroization", combined.lower())
            self.assertIn("terminal intercept", combined.lower())
            self.assertIn("safe containment ditching", combined.lower())

    def test_lifecycle_archetype_continuous_stationary_medical(self):
        """Verify CONTINUOUS_STATIONARY (Medical) archetype: 0 RTB, 0 flight corridors; enforces electromechanical joint brake lock & sterile field preservation."""
        canonical_units_dir = os.path.join(
            os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
            "skills",
            "spec-conops-engineering",
            "resources",
            "units",
        )
        params = {
            "SYSTEM_IDENTIFIER": "Surgical_Robotic_Console_X1",
            "PLATFORM_TYPE": "Surgical Robotic Master-Slave Console",
            "OPERATIONAL_DOMAIN": "Hospital Clinical Operating Room",
            "DOMAIN_TYPE": "medical",
        }
        engine = SysMLParameterBindingEngine(domain="medical", parameter_values=params, auto_detect=False)
        contract = engine._derive_lifecycle_contract()

        self.assertEqual(contract.lifecycle_type, LifecycleType.CONTINUOUS_STATIONARY)
        self.assertEqual(contract.containment_action, ContainmentActionType.ELECTROMECHANICAL_BRAKE_LOCK)
        self.assertEqual(engine.resolve_token("LIFECYCLE_TYPE"), "CONTINUOUS_STATIONARY")
        self.assertEqual(engine.resolve_token("PRIMARY_TERMINAL_TARGET"), "Primary Sterile Field Docking Station")
        self.assertEqual(engine.resolve_token("LIFECYCLE_TRANSIT_MODE"), "Autonomous_Clinical_Safing")

        with tempfile.TemporaryDirectory() as tmpdir:
            out_dir = os.path.join(tmpdir, "out_medical")
            success = assemble_conops(
                input_dir=canonical_units_dir,
                output_dir=out_dir,
                verify_only=False,
                domain="medical",
                params=params,
            )
            self.assertTrue(success)

            with open(os.path.join(out_dir, "CONOPS.md"), "r", encoding="utf-8") as f:
                conops_txt = f.read()
            with open(os.path.join(out_dir, "MISSION_INTENT.md"), "r", encoding="utf-8") as f:
                intent_txt = f.read()

            combined = conops_txt + "\n" + intent_txt

            # 0 RTB and 0 flight corridors
            self.assertNotIn("autonomous return-to-base (rtb)", combined.lower())
            self.assertNotIn("autonomous return-to-base sequence", combined.lower())
            self.assertNotIn("flight corridor", combined.lower())
            self.assertIn("sterile field", combined.lower())
            self.assertIn("joint brake", combined.lower())

    def test_lifecycle_archetype_track_bound_guided_rail(self):
        """Verify TRACK_BOUND_GUIDED (Rail) archetype: 0 RTB, 0 flight corridors; enforces track deceleration & maintenance siding brake."""
        canonical_units_dir = os.path.join(
            os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
            "skills",
            "spec-conops-engineering",
            "resources",
            "units",
        )
        params = {
            "SYSTEM_IDENTIFIER": "Autonomous_Shunting_Locomotive_9000",
            "PLATFORM_TYPE": "Heavy Freight Shunting Locomotive",
            "OPERATIONAL_DOMAIN": "Railway Classification Yard Operations",
            "DOMAIN_TYPE": "rail",
        }
        engine = SysMLParameterBindingEngine(domain="rail", parameter_values=params, auto_detect=False)
        contract = engine._derive_lifecycle_contract()

        self.assertEqual(contract.lifecycle_type, LifecycleType.TRACK_BOUND_GUIDED)
        self.assertEqual(contract.containment_action, ContainmentActionType.TRACK_SIDING_BRAKE)
        self.assertEqual(engine.resolve_token("LIFECYCLE_TYPE"), "TRACK_BOUND_GUIDED")
        self.assertEqual(engine.resolve_token("PRIMARY_TERMINAL_TARGET"), "Primary Rail Maintenance Siding")
        self.assertEqual(engine.resolve_token("LIFECYCLE_TRANSIT_MODE"), "Autonomous_Track_Deceleration")

        with tempfile.TemporaryDirectory() as tmpdir:
            out_dir = os.path.join(tmpdir, "out_rail")
            success = assemble_conops(
                input_dir=canonical_units_dir,
                output_dir=out_dir,
                verify_only=False,
                domain="rail",
                params=params,
            )
            self.assertTrue(success)

            with open(os.path.join(out_dir, "CONOPS.md"), "r", encoding="utf-8") as f:
                conops_txt = f.read()
            with open(os.path.join(out_dir, "MISSION_INTENT.md"), "r", encoding="utf-8") as f:
                intent_txt = f.read()

            combined = conops_txt + "\n" + intent_txt

            # 0 RTB and 0 flight corridors
            self.assertNotIn("autonomous return-to-base (rtb)", combined.lower())
            self.assertNotIn("autonomous return-to-base sequence", combined.lower())
            self.assertNotIn("flight corridor", combined.lower())
            self.assertIn("track deceleration", combined.lower())
            self.assertIn("siding", combined.lower())

    def test_lifecycle_archetype_persistent_orbital_space(self):
        """Verify PERSISTENT_ORBITAL (Space) archetype: 0 atmospheric landing; enforces de-orbit / graveyard disposal."""
        canonical_units_dir = os.path.join(
            os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
            "skills",
            "spec-conops-engineering",
            "resources",
            "units",
        )
        params = {
            "SYSTEM_IDENTIFIER": "LEO_CubeSat_Constellation_Node",
            "PLATFORM_TYPE": "12U LEO Earth Observation CubeSat",
            "OPERATIONAL_DOMAIN": "Low Earth Orbit Space Operations",
            "DOMAIN_TYPE": "space",
        }
        engine = SysMLParameterBindingEngine(domain="space", parameter_values=params, auto_detect=False)
        contract = engine._derive_lifecycle_contract()

        self.assertEqual(contract.lifecycle_type, LifecycleType.PERSISTENT_ORBITAL)
        self.assertEqual(contract.containment_action, ContainmentActionType.DEORBIT_DISPOSAL_BURN)
        self.assertEqual(engine.resolve_token("LIFECYCLE_TYPE"), "PERSISTENT_ORBITAL")
        self.assertEqual(engine.resolve_token("PRIMARY_TERMINAL_TARGET"), "Designated De-Orbit Reentry Corridor / Graveyard Orbit")
        self.assertEqual(engine.resolve_token("LIFECYCLE_TRANSIT_MODE"), "Autonomous_Disposal_Burn_Transit")

        with tempfile.TemporaryDirectory() as tmpdir:
            out_dir = os.path.join(tmpdir, "out_space")
            success = assemble_conops(
                input_dir=canonical_units_dir,
                output_dir=out_dir,
                verify_only=False,
                domain="space",
                params=params,
            )
            self.assertTrue(success)

            with open(os.path.join(out_dir, "CONOPS.md"), "r", encoding="utf-8") as f:
                conops_txt = f.read()
            with open(os.path.join(out_dir, "MISSION_INTENT.md"), "r", encoding="utf-8") as f:
                intent_txt = f.read()

            combined = conops_txt + "\n" + intent_txt

            # 0 atmospheric runway landing
            self.assertNotIn("autonomous return-to-base (rtb)", combined.lower())
            self.assertNotIn("civilian runway landing", combined.lower())
            self.assertIn("de-orbit", combined.lower())
            self.assertIn("graveyard", combined.lower())

    def test_lifecycle_archetype_reusable_recovery(self):
        """Verify REUSABLE_RECOVERY (Aviation/Maritime/Ground) archetype: enforces nominal RTB / vertiport / dock arrival."""
        canonical_units_dir = os.path.join(
            os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
            "skills",
            "spec-conops-engineering",
            "resources",
            "units",
        )
        params = {
            "SYSTEM_IDENTIFIER": "Tactical_ISR_FixedWing_UAV",
            "PLATFORM_TYPE": "Tactical ISR Fixed-Wing UAV",
            "OPERATIONAL_DOMAIN": "Airspace Operations",
            "DOMAIN_TYPE": "aviation",
            "TOTAL_MTOW_KG": "25.0",
        }
        engine = SysMLParameterBindingEngine(domain="aviation", parameter_values=params, auto_detect=False)
        contract = engine._derive_lifecycle_contract()

        self.assertEqual(contract.lifecycle_type, LifecycleType.REUSABLE_RECOVERY)
        self.assertEqual(contract.containment_action, ContainmentActionType.CONTROLLED_RECOVERY_LANDING)
        self.assertEqual(engine.resolve_token("LIFECYCLE_TYPE"), "REUSABLE_RECOVERY")
        self.assertEqual(engine.resolve_token("PRIMARY_TERMINAL_TARGET"), "Primary Recovery Base")
        self.assertEqual(engine.resolve_token("LIFECYCLE_TRANSIT_MODE"), "Autonomous_RTB_Transit")

        with tempfile.TemporaryDirectory() as tmpdir:
            out_dir = os.path.join(tmpdir, "out_aviation")
            success = assemble_conops(
                input_dir=canonical_units_dir,
                output_dir=out_dir,
                verify_only=False,
                domain="aviation",
                params=params,
            )
            self.assertTrue(success)

            with open(os.path.join(out_dir, "CONOPS.md"), "r", encoding="utf-8") as f:
                conops_txt = f.read()
            with open(os.path.join(out_dir, "MISSION_INTENT.md"), "r", encoding="utf-8") as f:
                intent_txt = f.read()

            combined = conops_txt + "\n" + intent_txt

            self.assertIn("primary recovery base", combined.lower())
            self.assertIn("return-to-base", combined.lower())

    def test_explicit_bingo_energy_parameters_preservation(self):
        """Verify explicit Bingo energy partitions are not overwritten by _derive_energy_budgets (Issue #206)."""
        explicit_bingo_params = {
            "TOTAL_POWER_NOMINAL_W": "500.0",
            "ENDURANCE_HOURS": "2.0",
            "BATTERY_CAPACITY_JOULES": "4000000.0",
            "E_RETURN_JOULES": "1800000.0",
            "E_DIVERT_JOULES": "700000.0",
            "E_RESERVE_JOULES": "950000.0",
            "E_CONTINGENCY_JOULES": "550000.0",
            "E_BINGO_JOULES": "4000000.0",
            "E_BINGO_THRESHOLD_JOULES": "4000000.0",
        }
        engine = SysMLParameterBindingEngine(
            parameter_values=explicit_bingo_params,
            auto_detect=False,
        )

        for key, val in explicit_bingo_params.items():
            self.assertEqual(
                engine.resolve_token(key),
                val,
                f"Explicit parameter {key} was clobbered: expected {val}, got {engine.resolve_token(key)}",
            )

    def test_state_vector_unit_cardinality_remediation_issue_205(self):
        """
        Reproduction and regression test for Issue #205:
        Verify that STATE_VECTOR_MIN_UNITS and STATE_VECTOR_MAX_UNITS in _derive_domain_ontology
        have exact dimensional cardinality matching state vector coordinate dimensions across
        all supported domains (aviation, medical, marine, space, rail, industrial).
        Specifically:
        - aviation: 6 elements ("rad, rad, m, m/s, m/s, m/s")
        - medical: 6 elements ("mm, mm, mm, mm/s, mm/s, mm/s")
        - marine: 6 elements ("m, m, m Depth, m/s, m/s, m/s")
        - space: 6 elements ("km, km, km, km/s, km/s, km/s")
        - industrial: 6 elements ("m, m, rad, m/s, rad/s, m")
        - rail: 4 elements ("m, m/s, m/s^2, bar")
        """
        expected_units = {
            "aviation": {
                "STATE_VECTOR_MIN_UNITS": "rad, rad, m, m/s, m/s, m/s",
                "STATE_VECTOR_MAX_UNITS": "rad, rad, m, m/s, m/s, m/s",
                "expected_dim": 6,
            },
            "medical": {
                "STATE_VECTOR_MIN_UNITS": "mm, mm, mm, mm/s, mm/s, mm/s",
                "STATE_VECTOR_MAX_UNITS": "mm, mm, mm, mm/s, mm/s, mm/s",
                "expected_dim": 6,
            },
            "marine": {
                "STATE_VECTOR_MIN_UNITS": "m, m, m Depth, m/s, m/s, m/s",
                "STATE_VECTOR_MAX_UNITS": "m, m, m Depth, m/s, m/s, m/s",
                "expected_dim": 6,
            },
            "space": {
                "STATE_VECTOR_MIN_UNITS": "km, km, km, km/s, km/s, km/s",
                "STATE_VECTOR_MAX_UNITS": "km, km, km, km/s, km/s, km/s",
                "expected_dim": 6,
            },
            "industrial": {
                "STATE_VECTOR_MIN_UNITS": "m, m, rad, m/s, rad/s, m",
                "STATE_VECTOR_MAX_UNITS": "m, m, rad, m/s, rad/s, m",
                "expected_dim": 6,
            },
            "rail": {
                "STATE_VECTOR_MIN_UNITS": "m, m/s, m/s^2, bar",
                "STATE_VECTOR_MAX_UNITS": "m, m/s, m/s^2, bar",
                "expected_dim": 4,
            },
        }

        for dom, spec in expected_units.items():
            with self.subTest(domain=dom):
                engine = SysMLParameterBindingEngine(domain=dom, auto_detect=False)
                min_units = engine.resolve_token("STATE_VECTOR_MIN_UNITS")
                max_units = engine.resolve_token("STATE_VECTOR_MAX_UNITS")
                min_expr = engine.resolve_token("STATE_VECTOR_MIN_EXPRESSION")
                max_expr = engine.resolve_token("STATE_VECTOR_MAX_EXPRESSION")

                # Parse comma-separated elements
                min_expr_elems = [e.strip() for e in min_expr.strip("[]^T").split(",")]
                max_expr_elems = [e.strip() for e in max_expr.strip("[]^T").split(",")]
                min_unit_elems = [u.strip() for u in min_units.split(",")]
                max_unit_elems = [u.strip() for u in max_units.split(",")]

                self.assertEqual(
                    len(min_expr_elems),
                    spec["expected_dim"],
                    f"Domain {dom} min expression dimension mismatch",
                )
                self.assertEqual(
                    len(max_expr_elems),
                    spec["expected_dim"],
                    f"Domain {dom} max expression dimension mismatch",
                )
                self.assertEqual(
                    len(min_unit_elems),
                    spec["expected_dim"],
                    f"Domain {dom} min units cardinality mismatch: {min_units}",
                )
                self.assertEqual(
                    len(max_unit_elems),
                    spec["expected_dim"],
                    f"Domain {dom} max units cardinality mismatch: {max_units}",
                )
                self.assertEqual(min_units, spec["STATE_VECTOR_MIN_UNITS"])
                self.assertEqual(max_units, spec["STATE_VECTOR_MAX_UNITS"])

    def test_lifecycle_derivation_purely_schema_driven_issue_209(self):
        """
        Reproduction and regression test for Issue #209:
        Verify that _derive_lifecycle_contract derives lifecycle archetypes
        (EXPENDABLE_KINETIC_EFFECTOR, CONTINUOUS_STATIONARY, TRACK_BOUND_GUIDED,
        PERSISTENT_ORBITAL, REUSABLE_RECOVERY) purely from domain schema parameters
        and domain keywords, NOT from sniffing test harness folder strings
        ('run_10', 'run_07', 'run_08', 'run_06').
        """
        # 1. Verify schema-driven derivation without any run_XX strings
        schema_cases = [
            (
                {"DOMAIN_TYPE": "medical", "SYSTEM_IDENTIFIER": "SurgicalConsole"},
                "medical",
                LifecycleType.CONTINUOUS_STATIONARY,
            ),
            (
                {"DOMAIN_TYPE": "rail", "SYSTEM_IDENTIFIER": "LocomotiveUnit"},
                "rail",
                LifecycleType.TRACK_BOUND_GUIDED,
            ),
            (
                {"DOMAIN_TYPE": "space", "SYSTEM_IDENTIFIER": "OrbitalCubeSat"},
                "space",
                LifecycleType.PERSISTENT_ORBITAL,
            ),
            (
                {"PLATFORM_TYPE": "Kinetic Interceptor UAV", "IS_EXPENDABLE": "true"},
                "aviation",
                LifecycleType.EXPENDABLE_KINETIC_EFFECTOR,
            ),
            (
                {"PLATFORM_TYPE": "Tactical ISR Fixed-Wing UAV"},
                "aviation",
                LifecycleType.REUSABLE_RECOVERY,
            ),
        ]

        for params, dom, expected_type in schema_cases:
            engine = SysMLParameterBindingEngine(domain=dom, parameter_values=params, auto_detect=False)
            contract = engine._derive_lifecycle_contract()
            self.assertEqual(
                contract.lifecycle_type,
                expected_type,
                f"Failed pure schema derivation for {dom} ({params}): got {contract.lifecycle_type}, expected {expected_type}",
            )

        # 2. Verify that test runner strings ('run_10', 'run_07', 'run_08', 'run_06') in identifiers
        # do NOT erroneously force incorrect archetype classification for standard systems.
        anti_sniffing_cases = [
            (
                {"SYSTEM_IDENTIFIER": "run_07_tactical_surveillance_drone", "PLATFORM_TYPE": "Fixed-Wing UAV"},
                "aviation",
                LifecycleType.REUSABLE_RECOVERY,
            ),
            (
                {"SYSTEM_IDENTIFIER": "run_08_maritime_patrol_vessel", "PLATFORM_TYPE": "Surface Patrol Craft"},
                "marine",
                LifecycleType.REUSABLE_RECOVERY,
            ),
            (
                {"SYSTEM_IDENTIFIER": "run_06_warehouse_forklift", "PLATFORM_TYPE": "Autonomous AGV"},
                "industrial",
                LifecycleType.REUSABLE_RECOVERY,
            ),
            (
                {"SYSTEM_IDENTIFIER": "run_10_cargo_transport_uav", "PLATFORM_TYPE": "Cargo Transport UAV", "IS_EXPENDABLE": "false"},
                "aviation",
                LifecycleType.REUSABLE_RECOVERY,
            ),
        ]

        for params, dom, expected_type in anti_sniffing_cases:
            engine = SysMLParameterBindingEngine(domain=dom, parameter_values=params, auto_detect=False)
            contract = engine._derive_lifecycle_contract()
            self.assertEqual(
                contract.lifecycle_type,
                expected_type,
                f"Test runner string sniffing poisoned classification for {params}: got {contract.lifecycle_type}, expected {expected_type}",
            )


if __name__ == "__main__":
    unittest.main()



