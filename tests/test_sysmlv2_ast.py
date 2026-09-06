#!/usr/bin/env python3
"""
Unit tests for SysML v2 AST Hazard, Risk, Connection definitions, and Topology Graph queries.
Verifies parsing of hazard def, risk def, connection def, connect statements, severity ratings,
and reachable hazard graph queries across port connection topologies.
"""

import os
import sys
import unittest

PROJECT_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
SPEC_SCRIPTS_DIR = os.path.join(PROJECT_ROOT, "skills", "spec-orchestrator", "scripts")

for p in (SPEC_SCRIPTS_DIR, PROJECT_ROOT):
    if p not in sys.path:
        sys.path.insert(0, p)

from sysmlv2_ast import (
    SysMLPackage,
    PartDef,
    PortDef,
    HazardDef,
    RiskDef,
    ConnectionDef,
    SysMLHazardDef,
    SysMLRiskDef,
    SysMLConnectionDef,
    SysMLParser,
)


class TestSysMLv2ASTHazardRiskConnection(unittest.TestCase):
    """Test suite for SysML v2 AST HazardDef, RiskDef, ConnectionDef, and topology graph queries."""

    def test_dataclass_instantiations_and_aliases(self):
        """Verify dataclasses instantiations and type aliases."""
        self.assertIs(SysMLHazardDef, HazardDef)
        self.assertIs(SysMLRiskDef, RiskDef)
        self.assertIs(SysMLConnectionDef, ConnectionDef)

        h = HazardDef(name="H_Overload", doc="High load hazard", severity=8, source_port="port_a", target_port="port_b")
        self.assertEqual(h.name, "H_Overload")
        self.assertEqual(h.severity, 8)
        self.assertEqual(h.source_port, "port_a")
        self.assertEqual(h.target_port, "port_b")
        self.assertIn("hazard def H_Overload", h.to_sysml())
        self.assertIn("severity : Integer = 8", h.to_sysml())

        r = RiskDef(name="R_Degradation", doc="Component degradation", severity=6, hazard_ref="H_Overload")
        self.assertEqual(r.name, "R_Degradation")
        self.assertEqual(r.severity, 6)
        self.assertEqual(r.hazard_ref, "H_Overload")
        self.assertIn("risk def R_Degradation", r.to_sysml())

        c = ConnectionDef(name="Conn_AB", source_port="ComponentA.p_out", target_port="ComponentB.p_in", severity=3)
        self.assertEqual(c.name, "Conn_AB")
        self.assertEqual(c.source_port, "ComponentA.p_out")
        self.assertEqual(c.target_port, "ComponentB.p_in")
        self.assertIn("connection def Conn_AB", c.to_sysml())
        self.assertIn("connect ComponentA.p_out to ComponentB.p_in", c.to_sysml())

    def test_parse_hazard_def_severities(self):
        """Verify parsing of hazard definitions with various severity specifications."""
        sysml_text = """
        package SafetyPackage {
            hazard def H_ExplicitTyped {
                doc /* Explicit typed severity */
                attribute severity : Integer = 9;
                attribute source_port = "out_port";
            }

            hazard def H_Untyped {
                doc /* Untyped attribute severity */
                attribute severity = 7;
            }

            hazard def H_CommentAnnotated {
                doc /* Thermal stress condition [severity: 5] */
            }

            hazard def H_DefaultFallback {
                doc /* No severity annotation */
            }

            hazard def H_StatementTyped;
        }
        """
        pkg = SysMLParser.parse_text(sysml_text)
        self.assertEqual(len(pkg.hazard_defs), 5)

        h_map = {h.name: h for h in pkg.hazard_defs}
        self.assertEqual(h_map["H_ExplicitTyped"].severity, 9)
        self.assertEqual(h_map["H_ExplicitTyped"].attributes.get("source_port"), "out_port")
        self.assertEqual(h_map["H_Untyped"].severity, 7)
        self.assertEqual(h_map["H_CommentAnnotated"].severity, 5)
        self.assertEqual(h_map["H_DefaultFallback"].severity, 1)
        self.assertEqual(h_map["H_StatementTyped"].severity, 1)

    def test_parse_risk_def(self):
        """Verify parsing of risk definitions and hazard references."""
        sysml_text = """
        package RiskPackage {
            risk def R_Unavailability {
                doc /* Risk of node unavailability */
                attribute severity : Integer = 8;
                attribute hazard_ref = "H_Overload";
            }

            risk def R_LatencySpike {
                attribute severity = 4;
            }

            risk def R_StatementRisk;
        }
        """
        pkg = SysMLParser.parse_text(sysml_text)
        self.assertEqual(len(pkg.risk_defs), 3)

        r_map = {r.name: r for r in pkg.risk_defs}
        self.assertEqual(r_map["R_Unavailability"].severity, 8)
        self.assertEqual(r_map["R_Unavailability"].hazard_ref, "H_Overload")
        self.assertEqual(r_map["R_LatencySpike"].severity, 4)
        self.assertEqual(r_map["R_StatementRisk"].severity, 1)

    def test_parse_connection_def_and_connect_statements(self):
        """Verify parsing of connection def and connect statements."""
        sysml_text = """
        package TopologyPackage {
            part def PartAlpha {
                port out p_out : Port;
            }

            part def PartBeta {
                port in p_in : Port;
                port out p_next : Port;
            }

            part def PartGamma {
                port in p_in : Port;
            }

            connection def ConnAlphaToBeta {
                doc /* Inter-component link */
                connect PartAlpha.p_out to PartBeta.p_in;
                attribute severity : Integer = 5;
            }

            connect PartBeta.p_next to PartGamma.p_in;
        }
        """
        pkg = SysMLParser.parse_text(sysml_text)
        self.assertEqual(len(pkg.connection_defs), 2)

        c1 = pkg.connection_defs[0]
        self.assertEqual(c1.name, "ConnAlphaToBeta")
        self.assertEqual(c1.source_port, "PartAlpha.p_out")
        self.assertEqual(c1.target_port, "PartBeta.p_in")
        self.assertEqual(c1.severity, 5)

        c2 = pkg.connection_defs[1]
        self.assertEqual(c2.source_port, "PartBeta.p_next")
        self.assertEqual(c2.target_port, "PartGamma.p_in")

    def test_topology_graph_and_reachable_hazards(self):
        """Verify port connection topology graph queries and reachable hazards."""
        sysml_text = """
        package IntegratedSystem {
            part def NodeA {
                port out port1 : Port;
                hazard def H_A1 {
                    attribute severity = 9;
                }
            }

            part def NodeB {
                port in port_in : Port;
                port out port_out : Port;
                hazard def H_B1 {
                    attribute severity = 6;
                }
            }

            part def NodeC {
                port in port1 : Port;
                port out port2 : Port;
            }

            part def IsolatedNode {
                port in port_iso : Port;
                hazard def H_Isolated {
                    attribute severity = 4;
                }
            }

            connection def ConnAB {
                connect NodeA.port1 to NodeB.port_in;
            }

            connect NodeB.port_out to NodeC.port1;
        }
        """
        pkg = SysMLParser.parse_text(sysml_text)

        # Verify all parts and hazards found
        self.assertEqual(len(pkg.part_defs), 4)
        all_hazards = pkg.get_all_hazards()
        self.assertEqual(len(all_hazards), 3)

        # Connection graph inspection
        graph = pkg.get_connection_graph()
        self.assertIn("NodeA.port1", graph)
        self.assertIn("NodeB.port_in", graph["NodeA.port1"])

        # Connected parts
        connected_from_a = pkg.get_connected_parts("NodeA")
        self.assertIn("NodeB", connected_from_a)
        self.assertIn("NodeC", connected_from_a)
        self.assertNotIn("IsolatedNode", connected_from_a)

        # Reachable hazards from NodeA
        reachable_from_a = pkg.get_reachable_hazards("NodeA")
        reachable_names_a = {h.name for h in reachable_from_a}
        self.assertIn("H_A1", reachable_names_a)
        self.assertIn("H_B1", reachable_names_a)
        self.assertNotIn("H_Isolated", reachable_names_a)

        # Reachable hazards from NodeC (transitive reachability to NodeB and NodeA)
        reachable_from_c = pkg.get_reachable_hazards("NodeC")
        reachable_names_c = {h.name for h in reachable_from_c}
        self.assertIn("H_A1", reachable_names_c)
        self.assertIn("H_B1", reachable_names_c)
        self.assertNotIn("H_Isolated", reachable_names_c)

        # Reachable hazards from IsolatedNode
        reachable_iso = pkg.get_reachable_hazards("IsolatedNode")
        reachable_names_iso = {h.name for h in reachable_iso}
        self.assertEqual(reachable_names_iso, {"H_Isolated"})

        # Depth-limited query: max_depth=0 should only include local hazards
        local_a = pkg.get_reachable_hazards("NodeA", max_depth=0)
        self.assertEqual({h.name for h in local_a}, {"H_A1"})

        # SysMLParser classmethod helper query
        helper_hazards = SysMLParser.query_reachable_hazards(pkg, "NodeA")
        self.assertEqual({h.name for h in helper_hazards}, reachable_names_a)

    def test_node_counts_and_all_node_names(self):
        """Verify node counts and node names include hazards, risks, and connections."""
        sysml_text = """
        package TestCounts {
            part def PartA {
                hazard def H1 { attribute severity = 8; }
                risk def R1 { attribute severity = 5; }
                connection def C1 { connect p1 to p2; }
            }
            hazard def H2 { attribute severity = 3; }
            risk def R2 { attribute severity = 2; }
            connection def C2 { connect a to b; }
        }
        """
        pkg = SysMLParser.parse_text(sysml_text)
        counts = pkg.node_counts()
        self.assertEqual(counts["hazard_defs"], 2)
        self.assertEqual(counts["risk_defs"], 2)
        self.assertEqual(counts["connection_defs"], 2)

        names = pkg.get_all_node_names()
        self.assertIn("H1", names)
        self.assertIn("H2", names)
        self.assertIn("R1", names)
        self.assertIn("R2", names)
        self.assertIn("C1", names)
        self.assertIn("C2", names)


if __name__ == "__main__":
    unittest.main()
