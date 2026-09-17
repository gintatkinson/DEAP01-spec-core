import os
import re
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
from parity_auditor.validators.architecture_viewpoint_validator import (
    ArchitectureViewpointValidator,
)
from parity_auditor.validators.conops_completeness_validator import (
    ConopsCompletenessValidator,
    MissionIntentCompletenessValidator,
)
from parity_auditor.validators.operational_allocation_validator import (
    OperationalAllocationValidator,
)
from scripts.verify_downstream_baseline import check_upstream_template_clean_landing_zones
from scripts.assemble_conops import assemble_document, assemble_conops


UPSTREAM_CLEAN_ALLOWLIST = {".gitkeep", "README.md", "units"}

DOWNSTREAM_CANONICAL_CONOPS_DELIVERABLES = {"MISSION_INTENT.md", "CONOPS.md"}


def _is_downstream_workspace(root):
    """Return True when the workspace at root is a downstream customer workspace.

    Upstream distribution template / compiler repos carry the
    .pipeline/upstream sentinel directory; downstream customer workspaces
    (installed via scripts/install_pipeline.sh) do not. The Clean Landing
    Zone Invariant applies strictly to upstream template repos only --
    concrete ConOps deliverables are constitutionally authorized in
    downstream workspaces (see .pipeline/constitution.md, Core System
    Boundaries & Invariants, and scripts/verify_downstream_baseline.py
    check_upstream_template_clean_landing_zones).
    """
    return not os.path.isdir(os.path.join(root, ".pipeline", "upstream"))


def _conops_landing_zone_allowlist(root):
    """Return the docs/conops landing-zone allowlist for the workspace class.

    Downstream customer workspaces may hold the canonical ConOps
    deliverables rendered from the abstract templates (MISSION_INTENT.md,
    CONOPS.md); upstream template repos must keep the strict
    clean-landing-zone rule (.gitkeep / README.md only).
    """
    if _is_downstream_workspace(root):
        return UPSTREAM_CLEAN_ALLOWLIST | DOWNSTREAM_CANONICAL_CONOPS_DELIVERABLES
    return UPSTREAM_CLEAN_ALLOWLIST


FORBIDDEN_DOMAIN_NOUNS = [
    r"\buav\b",
    r"\buas\b",
    r"\bdrone\b",
    r"\bdrones\b",
    r"\baircraft\b",
    r"\bairplane\b",
    r"\bquadcopter\b",
    r"\bmultirotor\b",
    r"\bfixed-wing\b",
    r"\bpropeller\b",
    r"\bpropellers\b",
    r"\bairframe\b",
    r"\bruddervator\b",
    r"\belevon\b",
    r"\belevons\b",
    r"\baileron\b",
    r"\bailerons\b",
    r"\bflight\b",
    r"\btakeoff\b",
    r"\btouchdown\b",
    r"\bparachute\b",
    r"\bloiter\b",
    r"\bpilot\b",
    r"\bcockpit\b",
    r"\bavenger\b",
    r"\bskyranger\b",
]


CANONICAL_CONOPS_UNIT_FILENAMES = [
    "01_METADATA_AND_OVERVIEW.md",
    "02_DEFICIENCIES_AND_MOTIVATION.md",
    "03_PROPOSED_CAPABILITIES.md",
    "04_SYSTEM_ARCHITECTURE.md",
    "04_USER_CLASSES_AND_STAKEHOLDERS.md",
    "05_OPERATIONAL_STATE_SPACE_AND_RISK.md",
    "06_UAF_OPERATIONAL_ACTIVITIES.md",
    "07_OPTX_EXCHANGES.md",
    "08_ENVIRONMENTAL_MIL_STD_810H.md",
    "09_SCENARIOS_AND_TIMELINES.md",
    "10_MAINTENANCE_AND_GSE_SUPPORT.md",
    "11_IMPACTS_AND_TRADE_STUDIES.md",
    "12_EMERGENCY_DECISION_MATRIX.md",
]

CANONICAL_MISSION_INTENT_UNIT_FILENAMES = [
    "01_COMMANDERS_INTENT.md",
    "02_MISSION_ESSENTIAL_TASK_LIST.md",
    "03_INCOSE_MOE_MOP_MATH.md",
    "04_MULTI_DOMAIN_THREAT_MATRIX.md",
    "05_PACE_C2_PLAN.md",
    "06_ROE_SAFETY_INTERLOCKS.md",
    "07_AIRSPACE_GEOZONES.md",
    "08_GO_NO_GO_MATRIX.md",
    "09_BINGO_ENERGY_MATH.md",
    "10_OPERATIONAL_ALLOCATION_TAGS.md",
]


class TestCanonicalTemplates(unittest.TestCase):
    """
    Test suite for Issue #67: Permanent Abstract Templates and Issue #145: Canonical Unit Filenames.
    Verifies that CONOPS and Mission Intent canonical templates and modular units exist,
    parse cleanly via Gate 26, contain zero domain nouns, use parameter tokens {{...}},
    are mirrored in .agents/, and that docs/conops/ is a clean landing zone with only .gitkeep
    for upstream template repos (downstream customer workspaces may hold the authorized
    MISSION_INTENT.md / CONOPS.md deliverables).
    """

    def setUp(self):
        self.conops_src = os.path.join(repo_root, "skills", "spec-orchestrator", "resources", "CONOPS_CANONICAL_TEMPLATE.md")
        self.mission_src = os.path.join(repo_root, "skills", "spec-orchestrator", "resources", "MISSION_INTENT_CANONICAL_TEMPLATE.md")
        self.conops_mirror = os.path.join(repo_root, ".agents", "skills", "spec-orchestrator", "resources", "CONOPS_CANONICAL_TEMPLATE.md")
        self.mission_mirror = os.path.join(repo_root, ".agents", "skills", "spec-orchestrator", "resources", "MISSION_INTENT_CANONICAL_TEMPLATE.md")

    def test_canonical_unit_filenames_exist_and_mirrored(self):
        """Verify that all 12 ConOps and 10 Mission Intent canonical uppercase units exist and match mirrors (Issue #145)."""
        conops_units_dir = os.path.join(repo_root, "skills", "spec-conops-engineering", "resources", "units", "conops")
        mission_units_dir = os.path.join(repo_root, "skills", "spec-conops-engineering", "resources", "units", "mission_intent")
        agents_conops_dir = os.path.join(repo_root, ".agents", "skills", "spec-conops-engineering", "resources", "units", "conops")
        agents_mission_dir = os.path.join(repo_root, ".agents", "skills", "spec-conops-engineering", "resources", "units", "mission_intent")

        for fname in CANONICAL_CONOPS_UNIT_FILENAMES:
            src = os.path.join(conops_units_dir, fname)
            mirror = os.path.join(agents_conops_dir, fname)
            self.assertTrue(os.path.isfile(src), f"Missing canonical ConOps unit: {src}")
            self.assertTrue(os.path.isfile(mirror), f"Missing mirrored ConOps unit: {mirror}")
            with open(src, "r", encoding="utf-8") as f1, open(mirror, "r", encoding="utf-8") as f2:
                self.assertEqual(f1.read(), f2.read(), f"Content mismatch for ConOps unit {fname}")

        for fname in CANONICAL_MISSION_INTENT_UNIT_FILENAMES:
            src = os.path.join(mission_units_dir, fname)
            mirror = os.path.join(agents_mission_dir, fname)
            self.assertTrue(os.path.isfile(src), f"Missing canonical Mission Intent unit: {src}")
            self.assertTrue(os.path.isfile(mirror), f"Missing mirrored Mission Intent unit: {mirror}")
            with open(src, "r", encoding="utf-8") as f1, open(mirror, "r", encoding="utf-8") as f2:
                self.assertEqual(f1.read(), f2.read(), f"Content mismatch for Mission Intent unit {fname}")

    def test_canonical_templates_exist_and_mirrored(self):
        """Verify that both canonical templates exist in resources/ and are mirrored in .agents/."""
        self.assertTrue(os.path.isfile(self.conops_src), f"Missing canonical template: {self.conops_src}")
        self.assertTrue(os.path.isfile(self.mission_src), f"Missing canonical template: {self.mission_src}")
        self.assertTrue(os.path.isfile(self.conops_mirror), f"Missing mirrored template: {self.conops_mirror}")
        self.assertTrue(os.path.isfile(self.mission_mirror), f"Missing mirrored template: {self.mission_mirror}")

        with open(self.conops_src, "r", encoding="utf-8") as f1, open(self.conops_mirror, "r", encoding="utf-8") as f2:
            self.assertEqual(f1.read(), f2.read(), "CONOPS template mirror mismatch")

        with open(self.mission_src, "r", encoding="utf-8") as f1, open(self.mission_mirror, "r", encoding="utf-8") as f2:
            self.assertEqual(f1.read(), f2.read(), "MISSION_INTENT template mirror mismatch")

    def test_conops_template_parses_cleanly_via_gate26(self):
        """Verify that CONOPS_CANONICAL_TEMPLATE.md parses with 0 errors via Gate 26 ConopsCompletenessValidator."""
        self.assertTrue(os.path.isfile(self.conops_src), f"Missing {self.conops_src}")
        with open(self.conops_src, "r", encoding="utf-8") as f:
            content = f.read()

        validator = ConopsCompletenessValidator()
        findings = validator._validate_conops_text(content, "CONOPS_CANONICAL_TEMPLATE.md")
        self.assertEqual(findings, [], f"Gate 26 findings on CONOPS template: {findings}")

    def test_mission_intent_template_parses_cleanly_via_gate26(self):
        """Verify that MISSION_INTENT_CANONICAL_TEMPLATE.md parses with 0 errors via Gate 26 MissionIntentCompletenessValidator."""
        self.assertTrue(os.path.isfile(self.mission_src), f"Missing {self.mission_src}")
        with open(self.mission_src, "r", encoding="utf-8") as f:
            content = f.read()

        validator = MissionIntentCompletenessValidator()
        findings = validator._validate_mission_text(content, "MISSION_INTENT_CANONICAL_TEMPLATE.md")
        self.assertEqual(findings, [], f"Gate 26 findings on MISSION_INTENT template: {findings}")

    def test_conops_template_zero_domain_nouns(self):
        """Verify that CONOPS_CANONICAL_TEMPLATE.md contains zero concrete domain nouns."""
        self.assertTrue(os.path.isfile(self.conops_src), f"Missing {self.conops_src}")
        with open(self.conops_src, "r", encoding="utf-8") as f:
            content = f.read()

        violations = []
        for pat in FORBIDDEN_DOMAIN_NOUNS:
            matches = re.findall(pat, content, re.IGNORECASE)
            if matches:
                violations.append(f"Found forbidden domain noun '{matches[0]}' matching pattern '{pat}'")

        self.assertEqual(violations, [], f"Forbidden domain nouns found in CONOPS template: {violations}")

    def test_mission_intent_template_zero_domain_nouns(self):
        """Verify that MISSION_INTENT_CANONICAL_TEMPLATE.md contains zero concrete domain nouns."""
        self.assertTrue(os.path.isfile(self.mission_src), f"Missing {self.mission_src}")
        with open(self.mission_src, "r", encoding="utf-8") as f:
            content = f.read()

        violations = []
        for pat in FORBIDDEN_DOMAIN_NOUNS:
            matches = re.findall(pat, content, re.IGNORECASE)
            if matches:
                violations.append(f"Found forbidden domain noun '{matches[0]}' matching pattern '{pat}'")

        self.assertEqual(violations, [], f"Forbidden domain nouns found in MISSION_INTENT template: {violations}")

    def test_canonical_units_zero_domain_nouns(self):
        """Verify that all modular unit files in CANONICAL_CONOPS_UNIT_FILENAMES contain zero forbidden domain nouns."""
        conops_units_dir = os.path.join(repo_root, "skills", "spec-conops-engineering", "resources", "units", "conops")
        violations = []
        for fname in CANONICAL_CONOPS_UNIT_FILENAMES:
            src = os.path.join(conops_units_dir, fname)
            self.assertTrue(os.path.isfile(src), f"Missing canonical ConOps unit: {src}")
            with open(src, "r", encoding="utf-8") as f:
                content = f.read()

            for pat in FORBIDDEN_DOMAIN_NOUNS:
                matches = re.findall(pat, content, re.IGNORECASE)
                if matches:
                    violations.append(f"{fname}: Found forbidden domain noun '{matches[0]}' matching pattern '{pat}'")

        self.assertEqual(violations, [], f"Forbidden domain nouns found in ConOps units: {violations}")

    def test_templates_use_parameter_tokens(self):
        """Verify that both templates use parameter tokens in format {{...}}."""
        self.assertTrue(os.path.isfile(self.conops_src), f"Missing {self.conops_src}")
        self.assertTrue(os.path.isfile(self.mission_src), f"Missing {self.mission_src}")

        with open(self.conops_src, "r", encoding="utf-8") as f:
            c_content = f.read()
        with open(self.mission_src, "r", encoding="utf-8") as f:
            m_content = f.read()

        c_tokens = re.findall(r'\{\{[A-Za-z0-9_]+(?::[^\}]*)?\}\}', c_content)
        m_tokens = re.findall(r'\{\{[A-Za-z0-9_]+(?::[^\}]*)?\}\}', m_content)

        self.assertGreaterEqual(len(c_tokens), 10, f"Expected >= 10 tokens in CONOPS template, found {len(c_tokens)}")
        self.assertGreaterEqual(len(m_tokens), 10, f"Expected >= 10 tokens in MISSION_INTENT template, found {len(m_tokens)}")

    def test_docs_conops_landing_zone_contains_only_gitkeep(self):
        """Verify that docs/conops/ contains only classification-authorized files.

        Upstream template repos (this core repo carries the .pipeline/upstream
        sentinel) must keep docs/conops/ clean with ONLY .gitkeep / README.md.
        """
        conops_dir = os.path.join(repo_root, "docs", "conops")
        self.assertTrue(os.path.isdir(conops_dir), f"docs/conops directory does not exist at {conops_dir}")

        files = os.listdir(conops_dir)
        allowlist = _conops_landing_zone_allowlist(repo_root)
        self.assertTrue(set(files).issubset(allowlist), f"docs/conops/ should contain ONLY files in {allowlist}, but found: {files}")

    def test_landing_zone_exempts_authorized_conops_deliverables_downstream(self):
        """Downstream customer workspaces permit MISSION_INTENT.md + CONOPS.md in docs/conops/.

        Regression guard for the false gate failure where a conformant
        downstream leaf carrying the constitutionally authorized ConOps
        deliverables failed the landing-zone assertion (strict upstream rule).
        """
        with tempfile.TemporaryDirectory(prefix="canonical_lz_downstream_") as tmp:
            for variant in ("no_pipeline", "pipeline_without_upstream_sentinel"):
                with self.subTest(variant=variant):
                    fake_root = os.path.join(tmp, variant)
                    conops_dir = os.path.join(fake_root, "docs", "conops")
                    os.makedirs(conops_dir)
                    if variant == "pipeline_without_upstream_sentinel":
                        os.makedirs(os.path.join(fake_root, ".pipeline"))
                    for name in [".gitkeep", "MISSION_INTENT.md", "CONOPS.md"]:
                        with open(os.path.join(conops_dir, name), "w", encoding="utf-8") as f:
                            f.write(f"# {name} downstream deliverable\n")

                    self.assertTrue(_is_downstream_workspace(fake_root))
                    files = set(os.listdir(conops_dir))
                    allowlist = _conops_landing_zone_allowlist(fake_root)
                    self.assertTrue(
                        files.issubset(allowlist),
                        f"Downstream workspaces must permit authorized ConOps deliverables; {files} outside {allowlist}",
                    )

    def test_landing_zone_rejects_stray_files_downstream(self):
        """Even downstream, docs/conops/ rejects stray files beyond the canonical deliverables."""
        with tempfile.TemporaryDirectory(prefix="canonical_lz_stray_") as tmp:
            fake_root = os.path.join(tmp, "downstream_leaf")
            conops_dir = os.path.join(fake_root, "docs", "conops")
            os.makedirs(conops_dir)
            for name in [".gitkeep", "MISSION_INTENT.md", "CONOPS.md", "STRAY_NOTES.md"]:
                with open(os.path.join(conops_dir, name), "w", encoding="utf-8") as f:
                    f.write(f"# {name}\n")

            files = set(os.listdir(conops_dir))
            allowlist = _conops_landing_zone_allowlist(fake_root)
            stray = files - allowlist
            self.assertEqual(stray, {"STRAY_NOTES.md"}, f"Downstream landing zone must reject stray files, but the assertion would clear: {files}")
            self.assertFalse(files.issubset(allowlist))

    def test_landing_zone_stays_strict_for_upstream_template_repo(self):
        """Upstream template repos keep the strict only-.gitkeep/README rule for docs/conops/."""
        with tempfile.TemporaryDirectory(prefix="canonical_lz_upstream_") as tmp:
            fake_root = os.path.join(tmp, "upstream_template")
            conops_dir = os.path.join(fake_root, "docs", "conops")
            os.makedirs(conops_dir)
            os.makedirs(os.path.join(fake_root, ".pipeline", "upstream"))
            for name in [".gitkeep", "MISSION_INTENT.md", "CONOPS.md"]:
                with open(os.path.join(conops_dir, name), "w", encoding="utf-8") as f:
                    f.write(f"# {name}\n")

            self.assertFalse(_is_downstream_workspace(fake_root))
            allowlist = _conops_landing_zone_allowlist(fake_root)
            self.assertEqual(allowlist, UPSTREAM_CLEAN_ALLOWLIST)
            files = set(os.listdir(conops_dir))
            violation = files - allowlist
            self.assertEqual(
                violation,
                {"MISSION_INTENT.md", "CONOPS.md"},
                f"Upstream template repo must reject concrete ConOps deliverables in docs/conops/: {files}",
            )

    def test_check16_clean_landing_zone_passes(self):
        """Verify that Check 16 (Upstream Template Clean Landing Zone Gate) passes."""
        # Should not raise SystemExit
        check_upstream_template_clean_landing_zones(repo_root)

    def test_ov5a_diagram_in_conops_unit_passes_gate30(self):
        """Verify that 06_UAF_OPERATIONAL_ACTIVITIES.md contains a conforming DoDAF OV-5a decomposition tree (refs #348)."""
        conops_units_dir = os.path.join(repo_root, "skills", "spec-conops-engineering", "resources", "units", "conops")
        unit_file = os.path.join(conops_units_dir, "06_UAF_OPERATIONAL_ACTIVITIES.md")
        self.assertTrue(os.path.isfile(unit_file), f"Missing {unit_file}")
        with open(unit_file, "r", encoding="utf-8") as f:
            content = f.read()

        self.assertIn("6.1.1 DoDAF OV-5a / UAF Op-Tx Operational Activity Decomposition Tree", content)
        self.assertIn("MissionOperations", content)
        for oa_idx in range(1, 9):
            self.assertIn(f"OA-{oa_idx:02d}", content)

        validator = ArchitectureViewpointValidator()
        findings = validator.validate_markdown_content(
            content,
            "skills/spec-conops-engineering/resources/units/conops/06_UAF_OPERATIONAL_ACTIVITIES.md",
            target_diagrams={"OV-5a"},
            allow_partial_snippets=True,
        )
        self.assertEqual(findings, [], f"ConOps unit 06 OV-5a failed Gate 30 validation: {findings}")

    def test_conops_unit06_sv5a_matrix_completeness(self):
        """Verify that 06_UAF_OPERATIONAL_ACTIVITIES.md contains a complete DoDAF SV-5a allocation matrix (refs #347)."""
        conops_units_dir = os.path.join(repo_root, "skills", "spec-conops-engineering", "resources", "units", "conops")
        unit_file = os.path.join(conops_units_dir, "06_UAF_OPERATIONAL_ACTIVITIES.md")
        self.assertTrue(os.path.isfile(unit_file), f"Missing {unit_file}")
        with open(unit_file, "r", encoding="utf-8") as f:
            content = f.read()

        section_heading = "### 6.2 DoDAF SV-5a / OMG UAF Op-to-Res Operational-to-Resource Allocation Matrix"
        self.assertIn(section_heading, content)

        lines = content.splitlines()
        header_idx = -1
        for idx, line in enumerate(lines):
            if section_heading in line:
                header_idx = idx
                break
        self.assertNotEqual(header_idx, -1, f"Heading '{section_heading}' not found in lines")

        # Parse the markdown table lines under Section 6.2
        table_lines = []
        for line in lines[header_idx + 1:]:
            stripped = line.strip()
            if stripped.startswith("###"):
                break
            if stripped.startswith("|") and stripped.endswith("|"):
                table_lines.append(stripped)

        self.assertGreaterEqual(len(table_lines), 3, "Expected table header, separator, and data rows under Section 6.2")

        header_cols = [c.strip() for c in table_lines[0].split("|")[1:-1]]
        required_columns = [
            "Operational Activity ID",
            "Activity Name",
            "System Performer Node",
            "Physical Subsystem / Partition",
            "Software Function / Partition",
            "Interface / Bus Binding",
            "Criticality & Safety Level",
        ]
        for col in required_columns:
            self.assertIn(col, header_cols, f"Missing required column '{col}' in SV-5a table header")

        col_indices = {col: header_cols.index(col) for col in required_columns}

        data_rows = []
        for row_str in table_lines[2:]:
            parts = [c.strip() for c in row_str.split("|")[1:-1]]
            if all(re.match(r"^:?-+:?$", p) for p in parts if p):
                continue
            row_dict = {col: parts[col_indices[col]] if col_indices[col] < len(parts) else "" for col in required_columns}
            if re.search(r"OA-\d+", row_dict["Operational Activity ID"]):
                data_rows.append(row_dict)

        expected_oas = [f"OA-{i:02d}" for i in range(1, 9)]
        found_oas = []
        for row in data_rows:
            m = re.search(r"\b(OA-\d+)\b", row["Operational Activity ID"])
            if m:
                found_oas.append(m.group(1))

        self.assertEqual(sorted(found_oas), expected_oas, f"Expected {expected_oas}, found {sorted(found_oas)}")

        for row_idx, row in enumerate(data_rows, start=1):
            for col in required_columns:
                val = row[col].strip()
                self.assertTrue(val, f"Row {row_idx} ({row.get('Operational Activity ID')}) column '{col}' is empty")

        with tempfile.TemporaryDirectory() as tmpdir:
            docs_conops = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(docs_conops, exist_ok=True)
            with open(os.path.join(docs_conops, "06_UAF_OPERATIONAL_ACTIVITIES.md"), "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = OperationalAllocationValidator()
            ops_universe = {oa: oa for oa in expected_oas}
            findings = validator._validate_conops_sv5a_matrix(repo, ops_universe)
            self.assertEqual(findings, [], f"_validate_conops_sv5a_matrix yielded unexpected findings: {findings}")

    def test_ov4_diagram_in_conops_unit_passes_gate30(self):
        """Verify that 01_METADATA_AND_OVERVIEW.md contains a conforming DoDAF OV-4 hierarchy (refs #345)."""
        conops_units_dir = os.path.join(repo_root, "skills", "spec-conops-engineering", "resources", "units", "conops")
        unit_file = os.path.join(conops_units_dir, "01_METADATA_AND_OVERVIEW.md")
        self.assertTrue(os.path.isfile(unit_file), f"Missing {unit_file}")
        with open(unit_file, "r", encoding="utf-8") as f:
            content = f.read()

        self.assertIn("1.6.1 DoDAF OV-4", content)
        validator = ArchitectureViewpointValidator()
        findings = validator.validate_markdown_content(
            content,
            "skills/spec-conops-engineering/resources/units/conops/01_METADATA_AND_OVERVIEW.md",
            target_diagrams={"OV-4"},
            allow_partial_snippets=True,
        )
        self.assertEqual(findings, [], f"ConOps unit 01 OV-4 failed Gate 30 validation: {findings}")

    def test_sv4_diagram_in_conops_unit_passes_gate30(self):
        """Verify that 04_SYSTEM_ARCHITECTURE.md contains a conforming DoDAF SV-4 diagram (refs #346)."""
        conops_units_dir = os.path.join(repo_root, "skills", "spec-conops-engineering", "resources", "units", "conops")
        unit_file = os.path.join(conops_units_dir, "04_SYSTEM_ARCHITECTURE.md")
        self.assertTrue(os.path.isfile(unit_file), f"Missing {unit_file}")
        with open(unit_file, "r", encoding="utf-8") as f:
            content = f.read()

        self.assertIn("4.6 DoDAF SV-4", content)
        validator = ArchitectureViewpointValidator()
        findings = validator.validate_markdown_content(
            content,
            "skills/spec-conops-engineering/resources/units/conops/04_SYSTEM_ARCHITECTURE.md",
            target_diagrams={"SV-4"},
            allow_partial_snippets=True,
        )
        self.assertEqual(findings, [], f"ConOps unit 04 SV-4 failed Gate 30 validation: {findings}")

    def test_spatial4d_diagram_in_conops_unit_passes_gate30(self):
        """Verify that 05_OPERATIONAL_STATE_SPACE_AND_RISK.md contains a conforming SPATIAL-4D diagram (refs #339, #351)."""
        conops_units_dir = os.path.join(repo_root, "skills", "spec-conops-engineering", "resources", "units", "conops")
        unit_file = os.path.join(conops_units_dir, "05_OPERATIONAL_STATE_SPACE_AND_RISK.md")
        self.assertTrue(os.path.isfile(unit_file), f"Missing {unit_file}")
        with open(unit_file, "r", encoding="utf-8") as f:
            content = f.read()

        self.assertIn("SPATIAL-4D", content)
        validator = ArchitectureViewpointValidator()
        findings = validator.validate_markdown_content(
            content,
            "skills/spec-conops-engineering/resources/units/conops/05_OPERATIONAL_STATE_SPACE_AND_RISK.md",
            target_diagrams={"SPATIAL-4D"},
            allow_partial_snippets=True,
        )
        self.assertEqual(findings, [], f"ConOps unit 05 SPATIAL-4D failed Gate 30 validation: {findings}")

    def test_assembled_canonical_conops_satisfies_all_architecture_viewpoints(self):
        """Verify assembled ConOps units satisfy all 11 canonical architecture diagrams without blinding (Issue #350)."""
        conops_units_dir = os.path.join(repo_root, "skills", "spec-conops-engineering", "resources", "units", "conops")
        conops_content, errors = assemble_document(conops_units_dir)
        self.assertEqual(errors, [], f"Assembled ConOps had errors: {errors}")
        validator = ArchitectureViewpointValidator()
        findings = validator.validate_markdown_content(
            conops_content, "docs/conops/CONOPS.md", target_diagrams=None
        )
        self.assertEqual(findings, [], f"Assembled ConOps failed unblinded Gate 30 validation: {findings}")

    def test_ov5b_diagram_in_conops_unit_passes_gate30(self):
        """Verify that 06_UAF_OPERATIONAL_ACTIVITIES.md contains a conforming DoDAF OV-5b diagram (refs #349)."""
        conops_units_dir = os.path.join(repo_root, "skills", "spec-conops-engineering", "resources", "units", "conops")
        unit_file = os.path.join(conops_units_dir, "06_UAF_OPERATIONAL_ACTIVITIES.md")
        self.assertTrue(os.path.isfile(unit_file), f"Missing {unit_file}")
        with open(unit_file, "r", encoding="utf-8") as f:
            content = f.read()

        self.assertIn("6.1.2 DoDAF OV-5b", content)
        validator = ArchitectureViewpointValidator()
        findings = validator.validate_markdown_content(
            content,
            "skills/spec-conops-engineering/resources/units/conops/06_UAF_OPERATIONAL_ACTIVITIES.md",
            target_diagrams={"OV-5b"},
            allow_partial_snippets=True,
        )
        self.assertEqual(findings, [], f"ConOps unit 06 OV-5b failed Gate 30 validation: {findings}")

    def test_ov6c_scenarios_in_conops_unit_passes_gate30(self):
        """Verify that 09_SCENARIOS_AND_TIMELINES.md contains conforming DoDAF OV-6c scenario sequences (refs #350)."""
        conops_units_dir = os.path.join(repo_root, "skills", "spec-conops-engineering", "resources", "units", "conops")
        unit_file = os.path.join(conops_units_dir, "09_SCENARIOS_AND_TIMELINES.md")
        self.assertTrue(os.path.isfile(unit_file), f"Missing {unit_file}")
        with open(unit_file, "r", encoding="utf-8") as f:
            content = f.read()

        validator = ArchitectureViewpointValidator()
        findings = validator.validate_markdown_content(
            content,
            "skills/spec-conops-engineering/resources/units/conops/09_SCENARIOS_AND_TIMELINES.md",
            target_diagrams={"OV-6c"},
            allow_partial_snippets=True,
        )
        self.assertEqual(findings, [], f"ConOps unit 09 OV-6c failed Gate 30 validation: {findings}")


if __name__ == "__main__":
    unittest.main()
