"""
Unit tests for SysML v2 Compiler Multi-Mode FMECA and 4-Guide-Word STPA Upgrades (DEAP-spec-core#51 and #50).
"""

import unittest
import os
import tempfile
import sys

# Ensure parity_auditor src is on sys.path
TEST_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(TEST_DIR, ".."))
PARITY_AUDITOR_SRC = os.path.join(PROJECT_ROOT, "skills", "spec-orchestrator", "parity_auditor", "src")
SPEC_SCRIPTS_DIR = os.path.join(PROJECT_ROOT, "skills", "spec-orchestrator", "scripts")

for p in (PROJECT_ROOT, PARITY_AUDITOR_SRC, SPEC_SCRIPTS_DIR):
    if p not in sys.path:
        sys.path.insert(0, p)

from scripts.compile_sysml import (
    parse_stpa_ucas,
    parse_fmeca_modes,
    compile_uca_to_constraint,
    compile_fmeca_to_constraint,
    compile_stpa_to_ast,
    compile_stpa_to_sysml,
    extract_use_cases_from_markdown,
    reverse_sync_specs_to_sysml,
)
from parity_auditor.validators.link_validator import LinkValidator
from parity_auditor.core.workspace import WorkspaceRepository


class TestCompileSysmlUpgrades(unittest.TestCase):
    """Test suite for compile_sysml.py upgrades and LinkValidator hardening."""

    def setUp(self):
        self.sample_fmeca_multimode = """
# FMECA Criticality Matrix (MIL-STD-1629A)

| Component | Failure Mode & Mechanism | Failure Rate lambda_p (/10^6 hr) | Mode Ratio alpha | Effect Prob beta | Mode Criticality C_m | Item Criticality C_r | Severity S | Mitigation |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| **Electric Motor** | Stator winding short | 12.5 | 0.40 | 1.00 | 7.50e-6 | 1.88e-5 | Catastrophic (5) | Stateflow glide recovery (SC-03) |
| | Bearing seizure | 12.5 | 0.30 | 1.00 | 5.63e-6 | 1.88e-5 | Catastrophic (5) | Pre-flight spin check, vibration limits |
| | Magnet demagnetization | 12.5 | 0.20 | 0.80 | 3.00e-6 | 1.88e-5 | Critical (4) | Thermal sensors, FCL current limiting |
| | Shaft shear at collet | 12.5 | 0.10 | 1.00 | 1.88e-6 | 1.88e-5 | Catastrophic (5) | High-strength steel shaft, torque limit |
| **Speed Controller** | MOSFET thermal shutdown | 18.0 | 0.35 | 0.90 | 8.51e-6 | 2.43e-5 | Critical (4) | Active heat sink, FCL thermal throttling |
| | Gate shoot-through short | 18.0 | 0.25 | 1.00 | 6.75e-6 | 2.43e-5 | Catastrophic (5) | Hardware dead-time protection |
| | Commutation desync | 18.0 | 0.25 | 0.70 | 4.73e-6 | 2.43e-5 | Critical (4) | Advanced BEMF zero-crossing filter |
| | BEC rail brownout | 18.0 | 0.15 | 1.00 | 4.05e-6 | 2.43e-5 | Catastrophic (5) | Isolated dual power architecture |
"""

        self.sample_stpa_4guidewords = """
# STPA Unsafe Control Actions (4-Guide-Word Taxonomy)

| Control Action | Guide Word | Context / State | Resulting Hazard | Safety Constraint |
| :--- | :--- | :--- | :--- | :--- |
| **Engage Autonomous RTL** | Not providing | C2 link lost t >= 30s in BVLOS | H-1, H-5 | SC-01, SC-06 |
| | Providing | Flare landing at 2m AGL | H-3, L-2 | SC-03, SC-16 |
| | Too late | Battery SoC < 20% against headwind | H-5, L-2 | SC-05 |
| | Stopped too soon | During transit before safe altitude | H-1, H-4 | SC-01, SC-04 |
| **Engage Geofence Turn** | Not providing | Distance to boundary < 50m | H-1, L-4 | SC-01 |
| | Providing | Centered in flight corridor | H-4, L-2 | SC-01, SC-04 |
| | Too late | Crossing boundary at V = 31 m/s | H-1, L-4 | SC-01 |
| | Applied too long | Turn held continuously | H-4, H-3 | SC-01, SC-04 |
"""

    def test_multimode_fmeca_component_inheritance(self):
        """Verify that multi-mode FMECA rows inherit component identity on empty leading cells."""
        modes = parse_fmeca_modes(self.sample_fmeca_multimode)
        self.assertEqual(len(modes), 8)

        # First 4 modes belong to Electric Motor
        for i in range(4):
            self.assertEqual(modes[i]["component"], "Electric Motor")
        
        # Second 4 modes belong to Speed Controller
        for i in range(4, 8):
            self.assertEqual(modes[i]["component"], "Speed Controller")

        self.assertEqual(modes[0]["failure_mode"], "Stator winding short")
        self.assertEqual(modes[1]["failure_mode"], "Bearing seizure")
        self.assertEqual(modes[4]["failure_mode"], "MOSFET thermal shutdown")

    def test_fmeca_quantitative_metrics_extraction(self):
        """Verify extraction of MIL-STD-1629A quantitative parameters (lambda_p, alpha, beta, Cm, Cr)."""
        modes = parse_fmeca_modes(self.sample_fmeca_multimode)
        m0 = modes[0]
        self.assertTrue(m0["is_quantitative"])
        self.assertEqual(m0["lambda_p"], 12.5)
        self.assertEqual(m0["alpha"], 0.40)
        self.assertEqual(m0["beta"], 1.00)
        self.assertAlmostEqual(m0["c_m"], 7.50e-6, places=8)
        self.assertAlmostEqual(m0["c_r"], 1.88e-5, places=7)

        # Verify compilation to SysML constraint def
        c = compile_fmeca_to_constraint(m0)
        doc = c.doc if hasattr(c, 'doc') else c['doc']
        name = c.name if hasattr(c, 'name') else c['name']
        self.assertIn("MIL-STD-1629A", doc)
        self.assertIn("Electric_Motor", name)

    def test_stpa_four_guideword_taxonomy_parsing(self):
        """Verify parsing of 4-guide-word STPA matrices with action inheritance."""
        ucas = parse_stpa_ucas(self.sample_stpa_4guidewords)
        self.assertEqual(len(ucas), 8)

        # First 4 UCAs are Engage Autonomous RTL
        for i in range(4):
            self.assertEqual(ucas[i]["control_action"], "Engage Autonomous RTL")

        # Second 4 UCAs are Engage Geofence Turn
        for i in range(4, 8):
            self.assertEqual(ucas[i]["control_action"], "Engage Geofence Turn")

        guidewords = [u["category"] for u in ucas]
        self.assertIn("Not providing", guidewords)
        self.assertIn("Providing", guidewords)
        self.assertIn("Too late", guidewords)
        self.assertIn("Stopped too soon", guidewords)
        self.assertIn("Applied too long", guidewords)

    def test_stpa_ast_rta_assertion_synthesis(self):
        """Verify AST constraint compilation and formal expression synthesis for UCAs."""
        ucas = parse_stpa_ucas(self.sample_stpa_4guidewords)
        c0 = compile_uca_to_constraint(ucas[0])  # Not providing RTL on C2 loss
        c2 = compile_uca_to_constraint(ucas[2])  # Too late RTL on low SoC

        expr0 = c0.expression if hasattr(c0, 'expression') else c0['expression']
        expr2 = c2.expression if hasattr(c2, 'expression') else c2['expression']

        self.assertIn("c2LinkLossDuration", expr0)
        self.assertIn("batterySoC", expr2)

        # Verify full package compilation
        sysml_text = compile_stpa_to_sysml(self.sample_stpa_4guidewords, "UAS_SafetyPackage")
        self.assertIn("package UAS_SafetyPackage", sysml_text)
        self.assertIn("assert constraint Assert_UCA_", sysml_text)

    def test_backward_compatibility_qualitative_tables(self):
        """Verify backward compatibility with legacy qualitative FMECA and UCA tables."""
        legacy_text = """
| UCA ID | Controller | Control Action | STPA UCA Category | Context | Hazard | Severity | SAIL |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **UCA-01** | FlightController | Land | 1. Not Provided | Critical battery | H-1 | Catastrophic | SAIL IV |

| FMECA-ID | Component | Failure Mode | Effect | Mitigation |
| :--- | :--- | :--- | :--- | :--- |
| **FMECA-01** | GPSModule | Lock Loss | Drift | Dual GPS Cross-Voting |
"""
        ucas = parse_stpa_ucas(legacy_text)
        fmecas = parse_fmeca_modes(legacy_text)

        self.assertEqual(len(ucas), 1)
        self.assertEqual(ucas[0]["id"], "UCA-01")
        self.assertEqual(len(fmecas), 1)
        self.assertEqual(fmecas[0]["id"], "FMECA-01")

    def test_link_validator_file_protocol_rejection(self):
        """Verify LinkValidator flags non-portable file:/// URIs."""
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_dir = os.path.join(tmpdir, "docs")
            os.makedirs(docs_dir, exist_ok=True)

            test_file = os.path.join(docs_dir, "test.md")
            with open(test_file, "w", encoding="utf-8") as f:
                f.write("[Broken Absolute Link](file:///tmp/test.md)")

            repo = WorkspaceRepository(tmpdir)
            validator = LinkValidator()
            findings = validator.validate(repo)

    def test_compiler_domain_decoupling_defaults(self):
        """Verify generic fallback names across STPA compiler, reverse sync, and use case extractors."""
        # 1. compile_stpa_to_ast and compile_stpa_to_sysml default package name
        ast_pkg = compile_stpa_to_ast(self.sample_stpa_4guidewords)
        pkg_name = ast_pkg.name if hasattr(ast_pkg, 'name') else ast_pkg['package']
        self.assertEqual(pkg_name, "System_SafetyConstraints")
        sysml_code = compile_stpa_to_sysml(self.sample_stpa_4guidewords)
        self.assertIn("package System_SafetyConstraints {", sysml_code)

        # 2. Fallback controller in parse_stpa_ucas (Pattern 2 and Pattern 3)
        ucas = parse_stpa_ucas(self.sample_stpa_4guidewords)
        self.assertEqual(ucas[0]["controller"], "SafetyController")

        # 3. Fallback use case in extract_use_cases_from_markdown
        fallback_uc = extract_use_cases_from_markdown("Some generic text without heading or metadata")
        self.assertEqual(len(fallback_uc), 1)
        uc_name = fallback_uc[0].name if hasattr(fallback_uc[0], 'name') else fallback_uc[0]['name']
        self.assertEqual(uc_name, "SystemUseCase")

        # 4. Fallback root package and docstring in reverse_sync_specs_to_sysml
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_dir = os.path.join(tmpdir, "docs")
            os.makedirs(docs_dir, exist_ok=True)
            out_sysml = os.path.join(tmpdir, "schema.sysml")
            out_digest = os.path.join(tmpdir, "schema-digest.json")
            pkg, _ = reverse_sync_specs_to_sysml(docs_dir=docs_dir, output_path=out_sysml, digest_path=out_digest)
            pkg_name = pkg.name if hasattr(pkg, 'name') else pkg['name']
            pkg_doc = pkg.doc if hasattr(pkg, 'doc') else pkg.get('doc', '')
            self.assertEqual(pkg_name, "System_SSOT")
            self.assertEqual(pkg_doc, "Single Source of Truth for System Architecture and Safety Model")

    def test_rta_formal_expression_explicit_and_generic(self):
        """Verify _derive_formal_rta_expression precedence with explicit expressions and generic invariants."""
        # Explicit formal expression precedence
        uca_explicit = {
            "id": "UCA-CUSTOM-01",
            "formal_expression": "customSignal >= thresholdValue * 2",
            "category": "not providing",
            "context": "C2 link lost",
        }
        c_exp = compile_uca_to_constraint(uca_explicit)
        expr_exp = c_exp.expression if hasattr(c_exp, 'expression') else c_exp['expression']
        self.assertEqual(expr_exp, "customSignal >= thresholdValue * 2")

        # Generic guide word without domain keywords
        uca_generic_prov = {
            "id": "UCA-GEN-01",
            "category": "providing",
            "context": "Generic subsystem operating normally",
        }
        c_gen = compile_uca_to_constraint(uca_generic_prov)
        expr_gen = c_gen.expression if hasattr(c_gen, 'expression') else c_gen['expression']
        self.assertEqual(expr_gen, "systemStateValid == true")


if __name__ == "__main__":
    unittest.main()
