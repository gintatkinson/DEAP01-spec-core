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
from parity_auditor.validators.icd_completeness_validator import ICDCompletenessValidator
from parity_auditor.aggregator import AGGREGATING_VALIDATORS


def _setup_base_valid_workspace(tmpdir: str) -> None:
    """Helper to populate a clean, fully-compliant baseline workspace."""
    schema_dir = os.path.join(tmpdir, "schema")
    interfaces_dir = os.path.join(tmpdir, "docs", "interfaces")
    os.makedirs(schema_dir, exist_ok=True)
    os.makedirs(interfaces_dir, exist_ok=True)

    sysml_content = """package SystemSSOT {
    port def NavTelemetryPort {
        out item PrimaryVelocity : Float32;
        out item PitchAngle : Float32;
    }

    port def FlightControlPort {
        in item PrimaryVelocity : Float32;
        in item PitchAngle : Float32;
    }

    part def NavigationSubsystem {
        out port nav_out : NavTelemetryPort;
    }

    part def FlightControlSubsystem {
        in port fcc_in : FlightControlPort;
    }

    connection Conn_Nav_FCC
        connect NavigationSubsystem.nav_out to FlightControlSubsystem.fcc_in;

    flow from NavigationSubsystem.nav_out to FlightControlSubsystem.fcc_in
        item PrimaryVelocity;

    flow from NavigationSubsystem.nav_out to FlightControlSubsystem.fcc_in
        item PitchAngle;
}
"""
    with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
        f.write(sysml_content)

    icd01_content = """| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #101 |
| **Title** | System Interface Matrix & Topological Connectivity |
| **Version** | 1.0.0 |
| **Date** | 2026-09-01 |
| **Type** | icd |
| **Interface Level** | Level 1C Logical Interface |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Level 1C: System Interface Matrix & Topological Connectivity

## 1. Executive Summary & Interface Scope
Topological connectivity matrix.

## 2. Subsystem Topological Connectivity Graph
```mermaid
flowchart TD
    subgraph NavigationSubsystem ["Navigation Subsystem"]
        P_NAV_OUT["PORT-NAV-OUT"]
    end
    subgraph FlightControlSubsystem ["Flight Control Subsystem"]
        P_FCC_IN["PORT-FCC-IN"]
    end
    P_NAV_OUT -->|"CONN-01"| P_FCC_IN
```

## 3. Canonical N² Subsystem Interface Matrix
| Subsystem | 1. NavigationSubsystem | 2. FlightControlSubsystem |
| :--- | :--- | :--- |
| **1. NavigationSubsystem** | **[ NavigationSubsystem ]** | CONN-01 (2 Signals) |
| **2. FlightControlSubsystem** | — | **[ FlightControlSubsystem ]** |

## 4. Port Definition Roster Table
| Port ID | Subsystem | Port Name | Direction | Port Type | Multiplicity | Protocol Profile |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `PORT-NAV-OUT` | NavigationSubsystem | nav_out | OUT | NavTelemetryPort | 1 | PeriodicStream (100 Hz) |
| `PORT-FCC-IN` | FlightControlSubsystem | fcc_in | IN | FlightControlPort | 1 | PeriodicStream (100 Hz) |

## 5. Connection Binding Roster Table
| Connection ID | Source Port | Dest Port | Flow Behavior | Latency Max ms | Reliability Req | Item Flows Conveyed |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `CONN-01` | `PORT-NAV-OUT` | `PORT-FCC-IN` | Continuous Stream | 10.0 | High | PrimaryVelocity, PitchAngle |
"""
    with open(os.path.join(interfaces_dir, "ICD_01_SYSTEM_INTERFACE_MATRIX.md"), "w", encoding="utf-8") as f:
        f.write(icd01_content)

    icd02_content = """| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #102 |
| **Title** | Master Signal Flow Dictionary & Safety Invariants |
| **Version** | 1.0.0 |
| **Date** | 2026-09-01 |
| **Type** | icd |
| **Interface Level** | Level 1C Logical Interface |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Level 1C: Master Signal Flow Dictionary & Safety Invariants

## 1. Executive Summary & Signal Flow Overview
Overview of signal dictionary.

## 2. Master Signal Flow Dictionary Table
| Signal ID | Signal Name | Source Port | Dest Port | Data Type | SI Units | Valid Range | Update Rate | Safe Default Value | Schema Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `SIG-NAV-FCC-001` | PrimaryVelocity | `PORT-NAV-OUT` | `PORT-FCC-IN` | Float32 | m/s | [0.0, 150.0] | 100 Hz | 0.0 | `schema/model.sysml#L3` |
| `SIG-NAV-FCC-002` | PitchAngle | `PORT-NAV-OUT` | `PORT-FCC-IN` | Float32 | rad | [-1.57, 1.57] | 100 Hz | 0.0 | `schema/model.sysml#L4` |
"""
    with open(os.path.join(interfaces_dir, "ICD_02_MASTER_SIGNAL_DICTIONARY.md"), "w", encoding="utf-8") as f:
        f.write(icd02_content)


class TestICDCompletenessValidator(unittest.TestCase):
    # -------------------------------------------------------------------------
    # Baseline Happy & Existing Negative Path Tests
    # -------------------------------------------------------------------------

    def test_valid_icd_suite_passes(self):
        """Workspace with clean SysML model and fully matching ICD_01 and ICD_02 returns 0 findings."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            errors = validator.validate(repo)
            self.assertEqual(errors, [])

    def test_dangling_port_detected(self):
        """Output port without destination connection flags finding with rule icd-dangling-port-detected."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            # Add dangling port to SysML and ICD_01
            schema_file = os.path.join(tmpdir, "schema", "model.sysml")
            with open(schema_file, "a", encoding="utf-8") as f:
                f.write("""
    part def AuxSubsystem {
        out port aux_out : Float32;
    }
""")
            icd01_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_01_SYSTEM_INTERFACE_MATRIX.md")
            with open(icd01_file, "r", encoding="utf-8") as f:
                content = f.read()
            content = content.replace(
                "| `PORT-FCC-IN` | FlightControlSubsystem | fcc_in | IN | FlightControlPort | 1 | PeriodicStream (100 Hz) |",
                "| `PORT-FCC-IN` | FlightControlSubsystem | fcc_in | IN | FlightControlPort | 1 | PeriodicStream (100 Hz) |\n| `PORT-AUX-OUT` | AuxSubsystem | aux_out | OUT | Float32 | 1 | PeriodicStream (100 Hz) |"
            )
            with open(icd01_file, "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            errors = validator.validate(repo)

            dangling_errors = [e for e in errors if getattr(e, "rule_id", "") == "icd-dangling-port-detected"]
            self.assertTrue(len(dangling_errors) >= 1)
            self.assertEqual(dangling_errors[0].rule_id, "icd-dangling-port-detected")

    def test_unmapped_signal_detected(self):
        """SysML item flow not present in ICD_02_MASTER_SIGNAL_DICTIONARY.md flags icd-unmapped-signal-detected."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            schema_file = os.path.join(tmpdir, "schema", "model.sysml")
            with open(schema_file, "a", encoding="utf-8") as f:
                f.write("""
    flow from NavigationSubsystem.nav_out to FlightControlSubsystem.fcc_in
        item UnmappedTelemetryFlow;
""")
            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            errors = validator.validate(repo)

            unmapped_errors = [e for e in errors if getattr(e, "rule_id", "") == "icd-unmapped-signal-detected"]
            self.assertTrue(len(unmapped_errors) >= 1)
            self.assertEqual(unmapped_errors[0].rule_id, "icd-unmapped-signal-detected")

    def test_missing_icd_suite_detected(self):
        """SysML model has ports/connections but docs/interfaces/ is missing flags icd-artifact-missing."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            os.makedirs(schema_dir, exist_ok=True)
            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write("""package SystemSSOT {
    part def NavigationSubsystem {
        out port nav_out : Float32;
    }
}
""")
            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            errors = validator.validate(repo)

            missing_errors = [e for e in errors if getattr(e, "rule_id", "") == "icd-artifact-missing"]
            self.assertTrue(len(missing_errors) >= 1)
            self.assertEqual(missing_errors[0].rule_id, "icd-artifact-missing")

    def test_empty_workspace_skipped(self):
        """Workspace with no ports or schemas returns 0 findings cleanly."""
        with tempfile.TemporaryDirectory() as tmpdir:
            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            errors = validator.validate(repo)
            self.assertEqual(errors, [])

    # -------------------------------------------------------------------------
    # Adversarial Fault Injection Suite: Malformed N² Matrix
    # -------------------------------------------------------------------------

    def test_adversarial_malformed_n2_matrix_missing_dest_header(self):
        """Adversarial test: N² matrix table is missing destination subsystem column header."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            icd01_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_01_SYSTEM_INTERFACE_MATRIX.md")
            with open(icd01_file, "r", encoding="utf-8") as f:
                content = f.read()

            # Malform N² matrix: drop FlightControlSubsystem destination column header
            malformed_n2 = """## 3. Canonical N² Subsystem Interface Matrix
| Subsystem | 1. NavigationSubsystem |
| :--- | :--- |
| **1. NavigationSubsystem** | **[ NavigationSubsystem ]** |
| **2. FlightControlSubsystem** | — |
"""
            content = content.replace(
                """## 3. Canonical N² Subsystem Interface Matrix
| Subsystem | 1. NavigationSubsystem | 2. FlightControlSubsystem |
| :--- | :--- | :--- |
| **1. NavigationSubsystem** | **[ NavigationSubsystem ]** | CONN-01 (2 Signals) |
| **2. FlightControlSubsystem** | — | **[ FlightControlSubsystem ]** |""",
                malformed_n2
            )
            with open(icd01_file, "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            findings = validator.validate(repo)

            n2_errors = [f for f in findings if f.rule_id == "icd-n2-matrix-malformed"]
            self.assertTrue(len(n2_errors) >= 1)
            self.assertIn("missing destination column header", str(n2_errors[0]).lower())

    def test_adversarial_malformed_n2_matrix_missing_source_header(self):
        """Adversarial test: N² matrix table is missing source subsystem row header."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            icd01_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_01_SYSTEM_INTERFACE_MATRIX.md")
            with open(icd01_file, "r", encoding="utf-8") as f:
                content = f.read()

            # Malform N² matrix: omit FlightControlSubsystem row
            malformed_n2 = """## 3. Canonical N² Subsystem Interface Matrix
| Subsystem | 1. NavigationSubsystem | 2. FlightControlSubsystem |
| :--- | :--- | :--- |
| **1. NavigationSubsystem** | **[ NavigationSubsystem ]** | CONN-01 (2 Signals) |
"""
            content = content.replace(
                """## 3. Canonical N² Subsystem Interface Matrix
| Subsystem | 1. NavigationSubsystem | 2. FlightControlSubsystem |
| :--- | :--- | :--- |
| **1. NavigationSubsystem** | **[ NavigationSubsystem ]** | CONN-01 (2 Signals) |
| **2. FlightControlSubsystem** | — | **[ FlightControlSubsystem ]** |""",
                malformed_n2
            )
            with open(icd01_file, "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            findings = validator.validate(repo)

            n2_errors = [f for f in findings if f.rule_id == "icd-n2-matrix-malformed"]
            self.assertTrue(len(n2_errors) >= 1)
            self.assertIn("missing source row header", str(n2_errors[0]).lower())


    def test_adversarial_malformed_n2_matrix_missing_entire_table(self):
        """Adversarial test: Multi-subsystem model where ICD_01 completely omits the N² matrix table."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            icd01_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_01_SYSTEM_INTERFACE_MATRIX.md")
            with open(icd01_file, "r", encoding="utf-8") as f:
                content = f.read()

            # Delete section 3 table
            target_section = """## 3. Canonical N² Subsystem Interface Matrix
| Subsystem | 1. NavigationSubsystem | 2. FlightControlSubsystem |
| :--- | :--- | :--- |
| **1. NavigationSubsystem** | **[ NavigationSubsystem ]** | CONN-01 (2 Signals) |
| **2. FlightControlSubsystem** | — | **[ FlightControlSubsystem ]** |"""
            content = content.replace(target_section, "## 3. Canonical N² Subsystem Interface Matrix\n(Omitted)")
            with open(icd01_file, "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            findings = validator.validate(repo)

            n2_errors = [f for f in findings if f.rule_id == "icd-n2-matrix-malformed"]
            self.assertTrue(len(n2_errors) >= 1)

    # -------------------------------------------------------------------------
    # Adversarial Fault Injection Suite: Signal Dictionary Missing Mandatory Columns
    # -------------------------------------------------------------------------

    def test_adversarial_signal_dict_missing_safe_default_column(self):
        """Adversarial test: Signal dictionary is missing 'Safe Default Value' / 'Fault / Safe Value' column."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            icd02_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_02_MASTER_SIGNAL_DICTIONARY.md")
            with open(icd02_file, "r", encoding="utf-8") as f:
                content = f.read()

            # Remove Safe Default Value column
            old_table = """## 2. Master Signal Flow Dictionary Table
| Signal ID | Signal Name | Source Port | Dest Port | Data Type | SI Units | Valid Range | Update Rate | Safe Default Value | Schema Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `SIG-NAV-FCC-001` | PrimaryVelocity | `PORT-NAV-OUT` | `PORT-FCC-IN` | Float32 | m/s | [0.0, 150.0] | 100 Hz | 0.0 | `schema/model.sysml#L3` |
| `SIG-NAV-FCC-002` | PitchAngle | `PORT-NAV-OUT` | `PORT-FCC-IN` | Float32 | rad | [-1.57, 1.57] | 100 Hz | 0.0 | `schema/model.sysml#L4` |"""

            new_table = """## 2. Master Signal Flow Dictionary Table
| Signal ID | Signal Name | Source Port | Dest Port | Data Type | SI Units | Valid Range | Update Rate | Schema Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `SIG-NAV-FCC-001` | PrimaryVelocity | `PORT-NAV-OUT` | `PORT-FCC-IN` | Float32 | m/s | [0.0, 150.0] | 100 Hz | `schema/model.sysml#L3` |
| `SIG-NAV-FCC-002` | PitchAngle | `PORT-NAV-OUT` | `PORT-FCC-IN` | Float32 | rad | [-1.57, 1.57] | 100 Hz | `schema/model.sysml#L4` |"""

            content = content.replace(old_table, new_table)
            with open(icd02_file, "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            findings = validator.validate(repo)

            col_errors = [f for f in findings if f.rule_id == "icd-missing-mandatory-column"]
            self.assertTrue(len(col_errors) >= 1)
            self.assertEqual(col_errors[0].detail.get("missing_column"), "Safe Default Value")

    def test_adversarial_signal_dict_missing_schema_citation_column(self):
        """Adversarial test: Signal dictionary is missing 'Schema Citation' column in table header."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            icd02_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_02_MASTER_SIGNAL_DICTIONARY.md")
            with open(icd02_file, "r", encoding="utf-8") as f:
                content = f.read()

            old_table = """## 2. Master Signal Flow Dictionary Table
| Signal ID | Signal Name | Source Port | Dest Port | Data Type | SI Units | Valid Range | Update Rate | Safe Default Value | Schema Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `SIG-NAV-FCC-001` | PrimaryVelocity | `PORT-NAV-OUT` | `PORT-FCC-IN` | Float32 | m/s | [0.0, 150.0] | 100 Hz | 0.0 | `schema/model.sysml#L3` |
| `SIG-NAV-FCC-002` | PitchAngle | `PORT-NAV-OUT` | `PORT-FCC-IN` | Float32 | rad | [-1.57, 1.57] | 100 Hz | 0.0 | `schema/model.sysml#L4` |"""

            new_table = """## 2. Master Signal Flow Dictionary Table
| Signal ID | Signal Name | Source Port | Dest Port | Data Type | SI Units | Valid Range | Update Rate | Safe Default Value |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `SIG-NAV-FCC-001` | PrimaryVelocity | `PORT-NAV-OUT` | `PORT-FCC-IN` | Float32 | m/s | [0.0, 150.0] | 100 Hz | 0.0 |
| `SIG-NAV-FCC-002` | PitchAngle | `PORT-NAV-OUT` | `PORT-FCC-IN` | Float32 | rad | [-1.57, 1.57] | 100 Hz | 0.0 |"""

            content = content.replace(old_table, new_table)
            with open(icd02_file, "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            findings = validator.validate(repo)

            col_errors = [f for f in findings if f.rule_id == "icd-missing-mandatory-column"]
            self.assertTrue(len(col_errors) >= 1)
            self.assertEqual(col_errors[0].detail.get("missing_column"), "Schema Citation")

    def test_adversarial_signal_dict_row_tbd_values(self):
        """Adversarial test: Signal dictionary rows contain TBD for safe default, units, and citation."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            icd02_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_02_MASTER_SIGNAL_DICTIONARY.md")
            with open(icd02_file, "r", encoding="utf-8") as f:
                content = f.read()

            content = content.replace("m/s", "TBD").replace("`schema/model.sysml#L3`", "TBD").replace("0.0", "TBD")
            with open(icd02_file, "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            findings = validator.validate(repo)

            rule_ids = {f.rule_id for f in findings}
            self.assertIn("icd-missing-units", rule_ids)
            self.assertIn("icd-missing-safe-default", rule_ids)
            self.assertIn("icd-missing-schema-citation", rule_ids)

    # -------------------------------------------------------------------------
    # Adversarial Fault Injection Suite: Incompatible Port Data Types
    # -------------------------------------------------------------------------

    def test_adversarial_incompatible_port_types_boolean_to_float32(self):
        """Adversarial test: Incompatible port data types (Boolean connected to Float32) in ICD_01 port roster."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            icd01_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_01_SYSTEM_INTERFACE_MATRIX.md")
            with open(icd01_file, "r", encoding="utf-8") as f:
                content = f.read()

            # Set nav_out port type to Boolean while fcc_in is Float32
            content = content.replace(
                "| `PORT-NAV-OUT` | NavigationSubsystem | nav_out | OUT | NavTelemetryPort | 1 | PeriodicStream (100 Hz) |",
                "| `PORT-NAV-OUT` | NavigationSubsystem | nav_out | OUT | Boolean | 1 | PeriodicStream (100 Hz) |"
            )
            content = content.replace(
                "| `PORT-FCC-IN` | FlightControlSubsystem | fcc_in | IN | FlightControlPort | 1 | PeriodicStream (100 Hz) |",
                "| `PORT-FCC-IN` | FlightControlSubsystem | fcc_in | IN | Float32 | 1 | PeriodicStream (100 Hz) |"
            )
            with open(icd01_file, "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            findings = validator.validate(repo)

            type_errors = [f for f in findings if f.rule_id == "icd-port-type-incompatibility"]
            self.assertTrue(len(type_errors) >= 1)
            self.assertIn("boolean", type_errors[0].detail.get("source_type", "").lower())
            self.assertIn("float32", type_errors[0].detail.get("dest_type", "").lower())

    def test_adversarial_incompatible_port_types_sysml_direct(self):
        """Adversarial test: SysML schema directly connects Boolean port to Float32 port."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            schema_file = os.path.join(tmpdir, "schema", "model.sysml")
            sysml_incompatible = """package SystemSSOT {
    part def NavigationSubsystem {
        out port nav_out : Boolean;
    }

    part def FlightControlSubsystem {
        in port fcc_in : Float32;
    }

    connection Conn_Nav_FCC
        connect NavigationSubsystem.nav_out to FlightControlSubsystem.fcc_in;

    flow from NavigationSubsystem.nav_out to FlightControlSubsystem.fcc_in
        item PrimaryVelocity;
}
"""
            with open(schema_file, "w", encoding="utf-8") as f:
                f.write(sysml_incompatible)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            findings = validator.validate(repo)

            type_errors = [f for f in findings if f.rule_id == "icd-port-type-incompatibility"]
            self.assertTrue(len(type_errors) >= 1)

    def test_adversarial_incompatible_port_types_port_def_items(self):
        """Adversarial test: SysML port definitions contain incompatible item types (Boolean vs Float32)."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            schema_file = os.path.join(tmpdir, "schema", "model.sysml")
            sysml_content = """package SystemSSOT {
    port def NavBoolPort {
        out item DiscreteFlag : Boolean;
    }

    port def FCCFloatPort {
        in item AnalogDemand : Float32;
    }

    part def NavigationSubsystem {
        out port nav_out : NavBoolPort;
    }

    part def FlightControlSubsystem {
        in port fcc_in : FCCFloatPort;
    }

    connection Conn_Nav_FCC
        connect NavigationSubsystem.nav_out to FlightControlSubsystem.fcc_in;

    flow from NavigationSubsystem.nav_out to FlightControlSubsystem.fcc_in
        item DiscreteFlag;
}
"""
            with open(schema_file, "w", encoding="utf-8") as f:
                f.write(sysml_content)

            icd01_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_01_SYSTEM_INTERFACE_MATRIX.md")
            with open(icd01_file, "r", encoding="utf-8") as f:
                c1 = f.read()
            c1 = c1.replace("NavTelemetryPort", "NavBoolPort").replace("FlightControlPort", "FCCFloatPort")
            c1 = c1.replace("PrimaryVelocity, PitchAngle", "DiscreteFlag")
            with open(icd01_file, "w", encoding="utf-8") as f:
                f.write(c1)

            icd02_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_02_MASTER_SIGNAL_DICTIONARY.md")
            with open(icd02_file, "r", encoding="utf-8") as f:
                c2 = f.read()
            c2 = c2.replace("PrimaryVelocity", "DiscreteFlag").replace("Float32", "Boolean")
            with open(icd02_file, "w", encoding="utf-8") as f:
                f.write(c2)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            findings = validator.validate(repo)

            type_errors = [f for f in findings if f.rule_id == "icd-port-type-incompatibility"]
            self.assertTrue(len(type_errors) >= 1)

    # -------------------------------------------------------------------------
    # Adversarial Fault Injection Suite: Incompatible Update Rates
    # -------------------------------------------------------------------------

    def test_adversarial_incompatible_update_rates_fast_publisher_slow_subscriber(self):
        """Adversarial test: Fast publisher (500 Hz) connected to slow subscriber (10 Hz) in ICD_01."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            icd01_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_01_SYSTEM_INTERFACE_MATRIX.md")
            with open(icd01_file, "r", encoding="utf-8") as f:
                content = f.read()

            content = content.replace(
                "| `PORT-NAV-OUT` | NavigationSubsystem | nav_out | OUT | NavTelemetryPort | 1 | PeriodicStream (100 Hz) |",
                "| `PORT-NAV-OUT` | NavigationSubsystem | nav_out | OUT | NavTelemetryPort | 1 | PeriodicStream (500 Hz) |"
            )
            content = content.replace(
                "| `PORT-FCC-IN` | FlightControlSubsystem | fcc_in | IN | FlightControlPort | 1 | PeriodicStream (100 Hz) |",
                "| `PORT-FCC-IN` | FlightControlSubsystem | fcc_in | IN | FlightControlPort | 1 | PeriodicStream (10 Hz) |"
            )
            with open(icd01_file, "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            findings = validator.validate(repo)

            rate_errors = [f for f in findings if f.rule_id == "icd-incompatible-update-rate"]
            self.assertTrue(len(rate_errors) >= 1)
            self.assertIn("500", str(rate_errors[0].detail.get("publisher_rate", "")))
            self.assertIn("10", str(rate_errors[0].detail.get("subscriber_rate", "")))

    def test_adversarial_incompatible_update_rates_signal_rate_vs_subscriber(self):
        """Adversarial test: Signal update rate (200 Hz) exceeds destination port rate (20 Hz)."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            icd01_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_01_SYSTEM_INTERFACE_MATRIX.md")
            with open(icd01_file, "r", encoding="utf-8") as f:
                c1 = f.read()
            c1 = c1.replace("PeriodicStream (100 Hz)", "PeriodicStream (20 Hz)")
            with open(icd01_file, "w", encoding="utf-8") as f:
                f.write(c1)

            icd02_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_02_MASTER_SIGNAL_DICTIONARY.md")
            with open(icd02_file, "r", encoding="utf-8") as f:
                c2 = f.read()
            c2 = c2.replace("100 Hz", "200 Hz")
            with open(icd02_file, "w", encoding="utf-8") as f:
                f.write(c2)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            findings = validator.validate(repo)

            rate_errors = [f for f in findings if f.rule_id == "icd-incompatible-update-rate"]
            self.assertTrue(len(rate_errors) >= 1)

    # -------------------------------------------------------------------------
    # Adversarial Compound Fault Injection
    # -------------------------------------------------------------------------

    def test_adversarial_compound_fault_injection_all_faults_caught(self):
        """Adversarial test: Multiple simultaneous faults injected across N² matrix, columns, types, and rates."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)

            # 1. Malform N² matrix (remove dest header)
            icd01_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_01_SYSTEM_INTERFACE_MATRIX.md")
            with open(icd01_file, "r", encoding="utf-8") as f:
                c1 = f.read()
            c1 = c1.replace(
                "| Subsystem | 1. NavigationSubsystem | 2. FlightControlSubsystem |",
                "| Subsystem | 1. NavigationSubsystem |"
            )
            # Incompatible types on ports (Boolean to Float32)
            c1 = c1.replace(
                "| `PORT-NAV-OUT` | NavigationSubsystem | nav_out | OUT | NavTelemetryPort | 1 | PeriodicStream (100 Hz) |",
                "| `PORT-NAV-OUT` | NavigationSubsystem | nav_out | OUT | Boolean | 1 | PeriodicStream (500 Hz) |"
            )
            c1 = c1.replace(
                "| `PORT-FCC-IN` | FlightControlSubsystem | fcc_in | IN | FlightControlPort | 1 | PeriodicStream (100 Hz) |",
                "| `PORT-FCC-IN` | FlightControlSubsystem | fcc_in | IN | Float32 | 1 | PeriodicStream (10 Hz) |"
            )
            with open(icd01_file, "w", encoding="utf-8") as f:
                f.write(c1)

            # 2. Missing mandatory column in ICD_02 ('Safe Default Value')
            icd02_file = os.path.join(tmpdir, "docs", "interfaces", "ICD_02_MASTER_SIGNAL_DICTIONARY.md")
            with open(icd02_file, "r", encoding="utf-8") as f:
                c2 = f.read()
            c2 = c2.replace("Safe Default Value | ", "")
            with open(icd02_file, "w", encoding="utf-8") as f:
                f.write(c2)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ICDCompletenessValidator()
            findings = validator.validate(repo)

            rule_ids = {f.rule_id for f in findings}

            # Verify 100% of injected fault categories are caught
            self.assertIn("icd-n2-matrix-malformed", rule_ids)
            self.assertIn("icd-missing-mandatory-column", rule_ids)
            self.assertIn("icd-port-type-incompatibility", rule_ids)
            self.assertIn("icd-incompatible-update-rate", rule_ids)

    # -------------------------------------------------------------------------
    # Aggregator and CLI Integration Tests
    # -------------------------------------------------------------------------

    def test_icd_completeness_validator_registered_in_aggregator(self):
        """Verify ICDCompletenessValidator is imported and registered in AGGREGATING_VALIDATORS in aggregator.py."""
        self.assertIn(ICDCompletenessValidator, AGGREGATING_VALIDATORS)

    def test_cli_integration_clean_run(self):
        """Verify parity_auditor CLI executes cleanly on valid workspace with --workspace and --schema-only."""
        with tempfile.TemporaryDirectory() as tmpdir:
            _setup_base_valid_workspace(tmpdir)
            
            pipeline_dir = os.path.join(tmpdir, ".pipeline", "logical-ui")
            os.makedirs(pipeline_dir, exist_ok=True)
            os.makedirs(os.path.join(tmpdir, "docs", "features"), exist_ok=True)
            with open(os.path.join(pipeline_dir, "codebase_rules.json"), "w", encoding="utf-8") as f:
                f.write("""{
  "meta": {
    "upstream_repository": "acme/example-project"
  },
  "backlog_directories": {
    "schemas": "schema",
    "features": "docs/features",
    "epics": "docs/epics"
  },
  "target_directories": {
    "react": "",
    "flutter": ""
  },
  "tracker_rules": {}
}""")
            with open(os.path.join(pipeline_dir, "logical-layout.json"), "w", encoding="utf-8") as f:
                f.write("{}")
            cli_py = os.path.join(repo_root, "skills", "spec-orchestrator", "parity_auditor", "src", "parity_auditor", "cli.py")
            import subprocess
            res = subprocess.run(
                [sys.executable, cli_py, "--workspace", tmpdir, "--schema-only"],
                capture_output=True,
                text=True
            )
            self.assertEqual(res.returncode, 0, f"CLI execution failed with stdout:\n{res.stdout}\nstderr:\n{res.stderr}")
            self.assertIn("ICD Completeness & Signal Flow Parity Audit", res.stdout)
            self.assertIn("Level 1C ICD port connectivity, N² matrix, and signal dictionary verified.", res.stdout)


if __name__ == "__main__":
    unittest.main()
