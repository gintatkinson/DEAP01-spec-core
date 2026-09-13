#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Unit tests for Mermaid Syntax Validator (Issue #200).

Validates:
1. Detection of commas inside quoted Mermaid labels in graph, flowchart, and state diagrams.
2. Detection of slashes inside quoted Mermaid labels in graph, flowchart, and state diagrams.
3. Clean compliant labels (using hyphens and spaces) passing with 0 findings.
4. Workspace scanning and Finding object properties.
"""

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
from parity_auditor.validators.mermaid_syntax_validator import (
    MermaidSyntaxValidator,
    check_mermaid_text,
    validate_mermaid_quoted_label_content,
    validate_mermaid_horizontal_flow,
    validate_mermaid_node_label_line_wrapping,
    validate_mermaid_subgraph_direction,
    validate_mermaid_option3_compact_blocks,
    validate_mermaid_layout_ergonomics,
)
from parity_auditor.core.findings import Finding


class TestMermaidSyntaxValidator(unittest.TestCase):
    def test_detects_comma_in_quoted_label_flowchart(self):
        """Verify detection of commas in quoted flowchart/graph node labels."""
        bad_md = """
```mermaid
flowchart TD
    A["ESAD powered on, safety pin in place"] --> B["Normal Mode"]
```
"""
        findings = check_mermaid_text(bad_md, source="bad_flowchart.md")
        rule_ids = [f.rule_id for f in findings]
        self.assertIn("mermaid-quoted-label-comma-forbidden", rule_ids)
        comma_findings = [f for f in findings if f.rule_id == "mermaid-quoted-label-comma-forbidden"]
        self.assertEqual(len(comma_findings), 1)
        self.assertIn("ESAD powered on, safety pin in place", str(comma_findings[0]))
        self.assertIn("GitLab's pinned Glfm renderer rejects this content", str(comma_findings[0]))

    def test_detects_slash_in_quoted_label_graph(self):
        """Verify detection of slashes in quoted graph node labels."""
        bad_md = """
```mermaid
graph TD
    Node["SitaWare HQ / ATAK / DELTA"] --> OutNode["Consumer Hub"]
```
"""
        findings = check_mermaid_text(bad_md, source="bad_graph.md")
        rule_ids = [f.rule_id for f in findings]
        self.assertIn("mermaid-quoted-label-slash-forbidden", rule_ids)
        slash_findings = [f for f in findings if f.rule_id == "mermaid-quoted-label-slash-forbidden"]
        self.assertEqual(len(slash_findings), 1)
        self.assertIn("SitaWare HQ / ATAK / DELTA", str(slash_findings[0]))

    def test_detects_comma_in_quoted_transition_statediagram(self):
        """Verify detection of commas in quoted stateDiagram-v2 transitions."""
        bad_md = """
```mermaid
stateDiagram-v2
    [*] --> Armed
    Armed --> Disabled: "ESAD powered on, safety pin in place"
    Disabled --> [*]
```
"""
        findings = check_mermaid_text(bad_md, source="bad_state.md")
        rule_ids = [f.rule_id for f in findings]
        self.assertIn("mermaid-quoted-label-comma-forbidden", rule_ids)
        comma_findings = [f for f in findings if f.rule_id == "mermaid-quoted-label-comma-forbidden"]
        self.assertEqual(len(comma_findings), 1)
        self.assertIn("ESAD powered on, safety pin in place", str(comma_findings[0]))

    def test_detects_slash_in_quoted_transition_statediagram(self):
        """Verify detection of slashes in quoted stateDiagram transitions."""
        bad_md = """
```mermaid
stateDiagram
    StateA --> StateB: "Action / Event Handler"
```
"""
        findings = check_mermaid_text(bad_md, source="bad_state_slash.md")
        rule_ids = [f.rule_id for f in findings]
        self.assertIn("mermaid-quoted-label-slash-forbidden", rule_ids)
        slash_findings = [f for f in findings if f.rule_id == "mermaid-quoted-label-slash-forbidden"]
        self.assertEqual(len(slash_findings), 1)
        self.assertIn("Action / Event Handler", str(slash_findings[0]))

    def test_detects_both_comma_and_slash_in_same_label(self):
        """Verify that a label containing both comma and slash triggers both findings."""
        bad_md = """
```mermaid
flowchart TD
    A["Primary / Secondary, Auto-Failover"] --> B["Active Node"]
```
"""
        findings = check_mermaid_text(bad_md, source="bad_combo.md")
        rule_ids = [f.rule_id for f in findings]
        self.assertIn("mermaid-quoted-label-comma-forbidden", rule_ids)
        self.assertIn("mermaid-quoted-label-slash-forbidden", rule_ids)

    def test_compliant_labels_pass_cleanly(self):
        """Verify compliant Mermaid diagrams with hyphens and spaces pass with 0 findings."""
        clean_md = """
# Compliant Mermaid Diagrams

```mermaid
flowchart TD
    A["ESAD powered on -<br/>safety pin in place"] --> B["Normal Mode"]
    Node["SitaWare HQ - ATAK - DELTA"] --> OutNode["Consumer Hub"]
```

```mermaid
stateDiagram-v2
    [*] --> Armed
    Armed --> Disabled: "ESAD powered on - safety pin in place"
    Disabled --> Ready: "Action - Event Handler"
    Ready --> [*]
```

```mermaid
graph TD
    subgraph "System Boundary"
        X["Component A (Active)"] --> Y["Component B (Standby)"]
    end
```
"""
        findings = check_mermaid_text(clean_md, source="clean_diagrams.md")
        self.assertEqual(len(findings), 0, f"Expected 0 findings but got: {findings}")

    def test_validate_mermaid_quoted_label_content_helper(self):
        """Direct test for helper function validate_mermaid_quoted_label_content."""
        line_with_comma = 'Armed --> Disabled: "ESAD powered on, safety pin in place"'
        res = validate_mermaid_quoted_label_content(line_with_comma)
        self.assertEqual(len(res), 1)
        self.assertEqual(res[0][0], '"ESAD powered on, safety pin in place"')
        self.assertEqual(res[0][1], [','])

        line_with_slash = 'Node["SitaWare HQ / ATAK / DELTA"]'
        res = validate_mermaid_quoted_label_content(line_with_slash)
        self.assertEqual(len(res), 1)
        self.assertEqual(res[0][0], '"SitaWare HQ / ATAK / DELTA"')
        self.assertEqual(res[0][1], ['/'])

        line_clean = 'Node["SitaWare HQ - ATAK - DELTA"]'
        res = validate_mermaid_quoted_label_content(line_clean)
        self.assertEqual(len(res), 0)

    def test_workspace_scanning(self):
        """Verify MermaidSyntaxValidator scans workspace documents correctly."""
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_dir = os.path.join(tmpdir, "docs")
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(docs_dir, "clean.md"), "w", encoding="utf-8") as f:
                f.write("""# Clean Document
```mermaid
flowchart TD
    A["Clean Label"] --> B["Another Clean Label"]
```
""")

            with open(os.path.join(docs_dir, "defect.md"), "w", encoding="utf-8") as f:
                f.write("""# Defect Document
```mermaid
flowchart TD
    A["Bad, Comma"] --> B["Bad / Slash"]
```
""")

            repo = WorkspaceRepository(tmpdir)
            validator = MermaidSyntaxValidator()
            findings = validator.validate(repo, search_dirs=[docs_dir])

            self.assertEqual(len(findings), 2)
            rule_ids = {f.rule_id for f in findings}
            self.assertIn("mermaid-quoted-label-comma-forbidden", rule_ids)
            self.assertIn("mermaid-quoted-label-slash-forbidden", rule_ids)
            self.assertTrue(all(isinstance(f, Finding) for f in findings))

    # -------------------------------------------------------------------------
    # Visual Ergonomics & Layout Invariant (Issue #274, Rules E1-E4)
    # -------------------------------------------------------------------------

    def test_rule_e1_horizontal_flow_rejected_when_node_count_gt_2(self):
        """Rule E1: flowchart LR with > 2 nodes is rejected with clear actionable error."""
        bad_md = """
```mermaid
flowchart LR
    A["Node A"] --> B["Node B"]
    B --> C["Node C"]
```
"""
        findings = check_mermaid_text(bad_md, source="bad_lr.md")
        rule_ids = [f.rule_id for f in findings]
        self.assertIn("mermaid-horizontal-flow-prohibited", rule_ids)
        e1_findings = [f for f in findings if f.rule_id == "mermaid-horizontal-flow-prohibited"]
        self.assertEqual(len(e1_findings), 1)
        self.assertIn("horizontal layout ('flowchart lr') is prohibited", str(e1_findings[0]))
        self.assertIn("flowchart TD", str(e1_findings[0]))

    def test_rule_e1_horizontal_flow_rejected_when_label_gt_25_chars(self):
        """Rule E1: graph LR with 2 nodes but a label > 25 chars is rejected."""
        bad_md = """
```mermaid
graph LR
    A["Start Node"] --> B["A label exceeding twenty five characters"]
```
"""
        findings = check_mermaid_text(bad_md, source="bad_lr_label.md")
        rule_ids = [f.rule_id for f in findings]
        self.assertIn("mermaid-horizontal-flow-prohibited", rule_ids)
        self.assertIn("exceeding 25 characters", str(findings[0]))

    def test_rule_e1_horizontal_flow_allowed_for_small_diagram(self):
        """Rule E1: flowchart LR with <= 2 nodes and labels <= 25 chars passes cleanly."""
        clean_md = """
```mermaid
flowchart LR
    A["Start"] --> B["End"]
```
"""
        findings = check_mermaid_text(clean_md, source="clean_lr.md")
        self.assertEqual(len(findings), 0, f"Expected 0 findings but got: {findings}")

    def test_rule_e1_vertical_flow_allowed_with_many_nodes(self):
        """Rule E1: flowchart TD with many nodes passes E1 check."""
        clean_md = """
```mermaid
flowchart TD
    A["Node A"] --> B["Node B"]
    B --> C["Node C"]
    C --> D["Node D"]
```
"""
        findings = check_mermaid_text(clean_md, source="clean_td.md")
        self.assertEqual(len(findings), 0, f"Expected 0 findings but got: {findings}")

    def test_rule_e2_unwrapped_label_gt_35_chars_rejected(self):
        """Rule E2: Node label line exceeding 35 characters without <br/> is rejected."""
        bad_md = """
```mermaid
flowchart TD
    A["Guidance and Perception Unit and Autonomous Navigation Controller"] --> B["Actuator Core"]
```
"""
        findings = check_mermaid_text(bad_md, source="bad_label_len.md")
        rule_ids = [f.rule_id for f in findings]
        self.assertIn("mermaid-node-label-line-wrapping-mandated", rule_ids)
        e2_findings = [f for f in findings if f.rule_id == "mermaid-node-label-line-wrapping-mandated"]
        self.assertEqual(len(e2_findings), 1)
        self.assertIn("node label line exceeds 35 characters without '<br/>' wrapping", str(e2_findings[0]))

    def test_rule_e2_wrapped_label_with_br_passes(self):
        """Rule E2: Node labels wrapped with <br/> into lines <= 35 characters pass cleanly."""
        clean_md = """
```mermaid
flowchart TD
    A["Guidance and Perception Unit<br/>and Autonomous Navigation<br/>Controller"] --> B["Actuator Core"]
```
"""
        findings = check_mermaid_text(clean_md, source="clean_label_wrap.md")
        self.assertEqual(len(findings), 0, f"Expected 0 findings but got: {findings}")

    def test_rule_e2_bold_tags_ignored_in_character_count(self):
        """Rule E2: <b> and </b> tags are ignored when calculating line character length."""
        clean_md = """
```mermaid
flowchart TD
    GCS["<b>Ground Control Station (GCS)</b><br/>• PORT_GCS_C2 (INOUT)"]
```
"""
        findings = check_mermaid_text(clean_md, source="clean_bold_label.md")
        self.assertEqual(len(findings), 0, f"Expected 0 findings but got: {findings}")

    def test_rule_e3_multi_subgraph_without_direction_tb_rejected(self):
        """Rule E3: Diagram with >= 2 subgraphs missing direction TB/TD is rejected."""
        bad_md = """
```mermaid
flowchart TD
    subgraph Segment_One["Command Segment"]
        A["Node A"]
    end
    subgraph Segment_Two["Platform Segment"]
        B["Node B"]
    end
```
"""
        findings = check_mermaid_text(bad_md, source="bad_subgraph.md")
        rule_ids = [f.rule_id for f in findings]
        self.assertIn("mermaid-subgraph-direction-tb-mandated", rule_ids)
        e3_findings = [f for f in findings if f.rule_id == "mermaid-subgraph-direction-tb-mandated"]
        self.assertEqual(len(e3_findings), 2)
        self.assertIn("Segment_One", str(e3_findings[0]))
        self.assertIn("Segment_Two", str(e3_findings[1]))

    def test_rule_e3_multi_subgraph_with_direction_tb_passes(self):
        """Rule E3: Diagram with >= 2 subgraphs declaring explicit direction TB passes cleanly."""
        clean_md = """
```mermaid
flowchart TD
    subgraph Segment_One["Command Segment"]
        direction TB
        A["Node A"]
    end
    subgraph Segment_Two["Platform Segment"]
        direction TB
        B["Node B"]
    end
```
"""
        findings = check_mermaid_text(clean_md, source="clean_subgraph.md")
        self.assertEqual(len(findings), 0, f"Expected 0 findings but got: {findings}")

    def test_rule_e3_subgraph_with_4_sibling_nodes_without_direction_rejected(self):
        """Rule E3: Single subgraph with >= 4 sibling nodes missing direction TB is rejected."""
        bad_md = """
```mermaid
flowchart TD
    subgraph Processing_Core["Processing Core"]
        A["Task 1"]
        B["Task 2"]
        C["Task 3"]
        D["Task 4"]
    end
```
"""
        findings = check_mermaid_text(bad_md, source="bad_4node_subgraph.md")
        rule_ids = [f.rule_id for f in findings]
        self.assertIn("mermaid-subgraph-direction-tb-mandated", rule_ids)

    def test_rule_e3_subgraph_with_3_sibling_nodes_without_direction_passes(self):
        """Rule E3: Single subgraph with < 4 sibling nodes does not require direction TB."""
        clean_md = """
```mermaid
flowchart TD
    subgraph Processing_Core["Processing Core"]
        A["Task 1"]
        B["Task 2"]
        C["Task 3"]
    end
```
"""
        findings = check_mermaid_text(clean_md, source="clean_3node_subgraph.md")
        self.assertEqual(len(findings), 0, f"Expected 0 findings but got: {findings}")

    def test_rule_e4_exploded_port_nodes_in_architecture_diagram_rejected(self):
        """Rule E4: Architecture diagram (SV-1 / ICD / STPA) with exploded standalone port nodes is rejected."""
        bad_md = """
```mermaid
flowchart TD
    %% DoDAF SV-1 System Interface Architecture
    subgraph Platform_Segment["Air Vehicle Segment"]
        direction TB
        PORT_FCS_C2["PORT_FCS_C2 (INOUT)"]
        PORT_FCS_CMD["PORT_FCS_CMD (OUT)"]
    end
```
"""
        findings = check_mermaid_text(bad_md, source="docs/conops/units/conops/04_USER_CLASSES_AND_STAKEHOLDERS.md")
        rule_ids = [f.rule_id for f in findings]
        self.assertIn("mermaid-option3-compact-block-mandated", rule_ids)
        e4_findings = [f for f in findings if f.rule_id == "mermaid-option3-compact-block-mandated"]
        self.assertTrue(len(e4_findings) >= 1)
        self.assertIn("exploded port node or subgraph detected", str(e4_findings[0]))

    def test_rule_e4_exploded_port_subgraph_in_architecture_diagram_rejected(self):
        """Rule E4: Architecture diagram with exploded port subgraph is rejected."""
        bad_md = """
```mermaid
flowchart TD
    %% ICD System Interface Matrix
    subgraph FCS_Ports["FCS Subsystem Ports"]
        direction TB
        P1["C2 Port (INOUT)"]
    end
```
"""
        findings = check_mermaid_text(bad_md, source="docs/icd/ICD_01_SYSTEM_INTERFACE_MATRIX.md")
        rule_ids = [f.rule_id for f in findings]
        self.assertIn("mermaid-option3-compact-block-mandated", rule_ids)

    def test_rule_e4_option3_compact_diagram_passes(self):
        """Rule E4: Canonical Option 3 compact blocks with embedded bulleted port attributes pass cleanly."""
        clean_md = """
# Canonical Option 3 SV-1 Architecture Diagram

```mermaid
flowchart TD
    subgraph External_Actors["External Operating Environment and Actors (IEEE 1362 §5.1)"]
        direction TB
        Operator["Human Operator and Mission<br/>Supervisor (UC-01 and UC-03)"]
        GNSS_Space["GNSS Constellation (Space Segment)"]
        Environment["Atmospheric and<br/>Environmental Dynamics"]
        RangeSafety["Range Safety Authority (UC-02)"]
    end

    subgraph Ground_Segment["Ground Command and Control Segment (IEEE 1362 §5.3)"]
        direction TB
        GCS["Ground Control Station (GCS)<br/>• PORT_GCS_C2 (INOUT)<br/>• PORT_GCS_DISP (OUT)"]
    end

    subgraph Platform_Segment["Air Vehicle and Primary Platform Segment (DoDAF SV-1)"]
        direction TB
        subgraph Tier1_Processing["Guidance & Perception Tier"]
            direction TB
            FCS["Flight and Guidance Controller<br/>• PORT_FCS_C2 (INOUT)<br/>• PORT_FCS_CMD (OUT)<br/>• PORT_FCS_TLM (IN)"]
            NavSensors["Sensor Fusion Unit<br/>• PORT_NAV_RF (IN)<br/>• PORT_NAV_DATA (OUT)"]
        end

        subgraph Tier2_Actuation["Energy - Actuation & Safety Tier"]
            direction TB
            Actuators["Distributed Actuator Core<br/>• PORT_ACT_IN (IN)"]
            Watchdog["Hardware Safety Watchdog<br/>• PORT_WD_IN (IN)<br/>• PORT_WD_TRIG (OUT)"]
        end
    end

    subgraph Support_Segment["Launch and Auxiliary Support Segment (IEEE 1362 §5.3)"]
        direction TB
        GSE["Ground Support Equipment<br/>and Staging<br/>• PORT_GSE_PWR (OUT)"]
    end

    %% External Interface Connections
    Operator -->|"CONN-01: Operator Command Input"| GCS
    GNSS_Space -->|"CONN-02: L-Band RF Navigation Signals"| NavSensors
    Environment -.->|"CONN-03: Aerodynamic Disturbance and Wind Gusts"| Actuators
    RangeSafety -->|"CONN-04: Flight Termination Consent"| GCS

    %% Segment Inter-Connects (Item Flows)
    GCS <-->|"CONN-05: PACE Bidirectional C2 Datalink [PORT_GCS_C2 <-> PORT_FCS_C2]"| FCS
    NavSensors -->|"CONN-06: Navigation State Estimates [PORT_NAV_DATA -> PORT_FCS_TLM]"| FCS
    FCS -->|"CONN-07: Real-Time Actuator Demand Vector [PORT_FCS_CMD -> PORT_ACT_IN]"| Actuators
    FCS -->|"CONN-08: Heartbeat Pulse and Safety Telemetry [PORT_FCS_CMD -> PORT_WD_IN]"| Watchdog
    GSE -.->|"CONN-09: Regulated Pre-Flight Power and Diagnostics [PORT_GSE_PWR]"| Platform_Segment
```
"""
        findings = check_mermaid_text(clean_md, source="docs/conops/units/conops/04_USER_CLASSES_AND_STAKEHOLDERS.md")
        self.assertEqual(len(findings), 0, f"Expected 0 findings but got: {findings}")

    def test_rule_e1_horizontal_sprawl_rejected_when_links_gt_4(self):
        """Rule E1: flowchart/graph LR with > 4 links and no subgraphs triggers mermaid-ergonomics-horizontal-sprawl."""
        bad_md = """
```mermaid
flowchart LR
    A --> B
    B --> C
    C --> D
    D --> E
    E --> F
```
"""
        findings = check_mermaid_text(bad_md, source="bad_sprawl_lr.md")
        rule_ids = [f.rule_id for f in findings]
        self.assertIn("mermaid-ergonomics-horizontal-sprawl", rule_ids)
        sprawl_findings = [f for f in findings if f.rule_id == "mermaid-ergonomics-horizontal-sprawl"]
        self.assertEqual(len(sprawl_findings), 1)
        self.assertIn("5 links", str(sprawl_findings[0]))

    def test_rule_e1_horizontal_sprawl_with_three_dashes_rejected_when_links_gt_4(self):
        """Rule E1: graph RL with > 4 '---' links and no subgraphs triggers mermaid-ergonomics-horizontal-sprawl."""
        bad_md = """
```mermaid
graph RL
    A --- B
    B --- C
    C --- D
    D --- E
    E --- F
```
"""
        findings = check_mermaid_text(bad_md, source="bad_sprawl_rl.md")
        rule_ids = [f.rule_id for f in findings]
        self.assertIn("mermaid-ergonomics-horizontal-sprawl", rule_ids)

    def test_rule_e2_unbroken_label_gt_35_chars_triggers_ergonomics_finding(self):
        """Rule E2: Single unbroken node label > 35 characters triggers mermaid-ergonomics-unbroken-label."""
        bad_md = """
```mermaid
flowchart TD
    A["Guidance and Perception Unit and Autonomous Navigation Controller"] --> B["Actuator Core"]
```
"""
        findings = check_mermaid_text(bad_md, source="bad_unbroken_label.md")
        rule_ids = [f.rule_id for f in findings]
        self.assertIn("mermaid-ergonomics-unbroken-label", rule_ids)
        unbroken = [f for f in findings if f.rule_id == "mermaid-ergonomics-unbroken-label"]
        self.assertEqual(len(unbroken), 1)
        self.assertIn("Guidance and Perception Unit and Autonomous Navigation Controller", str(unbroken[0]))

    def test_rule_e1_e2_well_structured_td_with_wrapped_labels_passes_cleanly(self):
        """Rule E1-E3: Well-structured TD/TB diagrams with wrapped labels pass cleanly."""
        clean_md = """
```mermaid
flowchart TD
    A["Guidance and Perception Unit<br/>and Navigation Controller"] --> B["Actuator Core"]
    B --> C["Power Unit"]
    C --> D["Telemetry Logger"]
    D --> E["Safety Watchdog"]
```
"""
        findings = check_mermaid_text(clean_md, source="clean_td_wrapped.md")
        self.assertEqual(len(findings), 0, f"Expected 0 findings but got: {findings}")

    def test_validate_mermaid_layout_ergonomics_direct_call(self):
        """Direct verification of validate_mermaid_layout_ergonomics helper function."""
        body = [
            "flowchart LR",
            "A --> B",
            "B --> C",
            "C --> D",
            "D --> E",
            "E --> F",
        ]
        findings = validate_mermaid_layout_ergonomics(start=1, body=body, kind="flowchart", source="direct_test.md")
        rule_ids = [f.rule_id for f in findings]
        self.assertIn("mermaid-ergonomics-horizontal-sprawl", rule_ids)


if __name__ == "__main__":
    unittest.main()

