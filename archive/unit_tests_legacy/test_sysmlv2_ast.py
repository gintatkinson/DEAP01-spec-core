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
    ItemFlowDef,
    SysMLHazardDef,
    SysMLRiskDef,
    SysMLConnectionDef,
    SysMLItemFlowDef,
    SysMLPortDef,
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

    def test_item_flow_def(self):
        """Verify ItemFlowDef defaults, custom instantiation, dictionary export, and SysML serialization."""
        self.assertIs(SysMLItemFlowDef, ItemFlowDef)
        self.assertIs(SysMLPortDef, PortDef)

        # Default values
        flow_default = ItemFlowDef(name="default_flow")
        self.assertEqual(flow_default.name, "default_flow")
        self.assertEqual(flow_default.direction, "out")
        self.assertEqual(flow_default.item_type, "Item")
        self.assertEqual(flow_default.doc, "")
        self.assertIsNone(flow_default.rate_hz)
        self.assertEqual(flow_default.unit, "")
        self.assertEqual(flow_default.valid_range, "")
        self.assertIsNone(flow_default.default_value)
        self.assertEqual(flow_default.to_sysml().strip(), "out flow default_flow : Item;")

        # Custom values
        flow_custom = ItemFlowDef(
            name="nav_telemetry",
            direction="in",
            item_type="NavigationData",
            doc="Navigation sensor stream",
            rate_hz=50.0,
            unit="m/s",
            valid_range="[-500.0, 500.0]",
            default_value="0.0",
        )
        self.assertEqual(flow_custom.to_dict(), {
            "name": "nav_telemetry",
            "direction": "in",
            "item_type": "NavigationData",
            "doc": "Navigation sensor stream",
            "rate_hz": 50.0,
            "unit": "m/s",
            "valid_range": "[-500.0, 500.0]",
            "default_value": "0.0",
        })
        self.assertIn("in flow nav_telemetry : NavigationData;", flow_custom.to_sysml())
        self.assertIn("doc /* Navigation sensor stream */", flow_custom.to_sysml())

    def test_enhanced_port_def(self):
        """Verify PortDef conjugation, category, protocol family, electrical attributes, and item flows."""
        # Conjugated port by explicit flag
        p_conj1 = PortDef(name="sensor_port", type_name="SensorPort", is_conjugated=True)
        self.assertTrue(p_conj1.is_conjugated)
        self.assertEqual(p_conj1.type_name, "SensorPort")
        self.assertIn("port sensor_port : ~SensorPort;", p_conj1.to_sysml())

        # Conjugated port auto-detected via type_name prefix
        p_conj2 = PortDef(name="client_port", type_name="~ServicePort")
        self.assertTrue(p_conj2.is_conjugated)
        self.assertEqual(p_conj2.type_name, "ServicePort")
        self.assertIn("port client_port : ~ServicePort;", p_conj2.to_sysml())

        # Enhanced PortDef with all fields
        f1 = ItemFlowDef(name="command_stream", direction="in", item_type="CommandMsg", rate_hz=20.0)
        p_full = PortDef(
            name="cmd_in",
            direction="in",
            type_name="ActuatorCommandPort",
            doc="Actuator command input",
            port_category="CommandPort",
            protocol_family="MIL-STD-1553",
            electrical_attributes={"baud_rate": 1000000, "voltage_domain": "28V"},
            item_flows=[f1],
        )
        p_dict = p_full.to_dict()
        self.assertEqual(p_dict["name"], "cmd_in")
        self.assertEqual(p_dict["direction"], "in")
        self.assertEqual(p_dict["port_category"], "CommandPort")
        self.assertEqual(p_dict["protocol_family"], "MIL-STD-1553")
        self.assertEqual(p_dict["electrical_attributes"]["baud_rate"], 1000000)
        self.assertEqual(p_dict["electrical_attributes"]["voltage_domain"], "28V")
        self.assertEqual(len(p_dict["item_flows"]), 1)
        self.assertEqual(p_dict["item_flows"][0]["name"], "command_stream")

        sysml_repr = p_full.to_sysml()
        self.assertIn("in port cmd_in : ActuatorCommandPort {", sysml_repr)
        self.assertIn("attribute protocol_family : String = \"MIL-STD-1553\";", sysml_repr)
        self.assertIn("attribute port_category : String = \"CommandPort\";", sysml_repr)
        self.assertIn("attribute baud_rate : Integer = 1000000;", sysml_repr)
        self.assertIn("in flow command_stream : CommandMsg;", sysml_repr)

    def test_enhanced_connection_def(self):
        """Verify ConnectionDef part extraction, item flow reference, protocol, and latency."""
        # Auto-population of source_part and target_part from port names
        c = ConnectionDef(
            name="Conn_FCS_Actuator",
            source_port="FlightController.act_cmd_out",
            target_port="ActuatorDriver.act_cmd_in",
            severity=4,
            protocol="CAN",
            latency_ms=2.5,
            item_flow_ref="ActuatorCommandFlow",
        )
        self.assertEqual(c.source_part, "FlightController")
        self.assertEqual(c.target_part, "ActuatorDriver")
        self.assertEqual(c.protocol, "CAN")
        self.assertEqual(c.latency_ms, 2.5)
        self.assertEqual(c.item_flow_ref, "ActuatorCommandFlow")

        c_dict = c.to_dict()
        self.assertEqual(c_dict["source_part"], "FlightController")
        self.assertEqual(c_dict["target_part"], "ActuatorDriver")
        self.assertEqual(c_dict["protocol"], "CAN")
        self.assertEqual(c_dict["latency_ms"], 2.5)
        self.assertEqual(c_dict["item_flow_ref"], "ActuatorCommandFlow")

        c_sysml = c.to_sysml()
        self.assertIn("connection def Conn_FCS_Actuator {", c_sysml)
        self.assertIn("connect FlightController.act_cmd_out to ActuatorDriver.act_cmd_in;", c_sysml)
        self.assertIn("attribute severity : Integer = 4;", c_sysml)
        self.assertIn("attribute protocol : String = \"CAN\";", c_sysml)
        self.assertIn("attribute latency_ms : Real = 2.5;", c_sysml)
        self.assertIn("attribute item_flow_ref : String = \"ActuatorCommandFlow\";", c_sysml)

        # Explicitly set parts are preserved
        c_explicit = ConnectionDef(
            name="Conn_Explicit",
            source_part="CustomSource",
            target_part="CustomTarget",
            source_port="SubA.port1",
            target_port="SubB.port2",
        )
        self.assertEqual(c_explicit.source_part, "CustomSource")
        self.assertEqual(c_explicit.target_part, "CustomTarget")

    def test_parse_port_conjugation_and_typing(self):
        """Verify SysMLParser parses conjugated ports, port categories, protocols, and port blocks."""
        sysml_text = """
        package AvionicsPackage {
            part def FlightComputer {
                ~port sensor_in : SensorPort;
                out port nav_out : ~NavPort;
                in port cmd_in : CommandPort;
                port telem_out : TelemetryPort [protocol: ARINC 429] [baud: 100000];
            }

            port def GPSInterfacePort {
                doc /* GPS Interface Port Definition */
                attribute protocol_family = "RS-485";
                attribute baud_rate : Integer = 115200;
                attribute voltage_domain = "12V";
                out flow gps_fix : GPSFixMsg [rate: 10Hz, unit: m];
            }
        }
        """
        pkg = SysMLParser.parse_text(sysml_text)
        self.assertEqual(len(pkg.part_defs), 1)
        fc = pkg.part_defs[0]
        self.assertEqual(len(fc.ports), 4)

        p_map = {p.name: p for p in fc.ports}
        # sensor_in: ~port
        self.assertTrue(p_map["sensor_in"].is_conjugated)
        self.assertEqual(p_map["sensor_in"].type_name, "SensorPort")

        # nav_out: port : ~NavPort
        self.assertTrue(p_map["nav_out"].is_conjugated)
        self.assertEqual(p_map["nav_out"].type_name, "NavPort")
        self.assertEqual(p_map["nav_out"].direction, "out")

        # cmd_in: CommandPort category
        self.assertEqual(p_map["cmd_in"].port_category, "CommandPort")
        self.assertEqual(p_map["cmd_in"].direction, "in")

        # telem_out: TelemetryPort, protocol, baud_rate
        self.assertEqual(p_map["telem_out"].port_category, "TelemetryPort")
        self.assertEqual(p_map["telem_out"].protocol_family, "ARINC 429")
        self.assertEqual(p_map["telem_out"].electrical_attributes.get("baud_rate"), 100000)

        # GPSInterfacePort in package port_defs
        self.assertEqual(len(pkg.port_defs), 1)
        gps_port = pkg.port_defs[0]
        self.assertEqual(gps_port.name, "GPSInterfacePort")
        self.assertEqual(gps_port.protocol_family, "RS-485")
        self.assertEqual(gps_port.electrical_attributes.get("baud_rate"), 115200)
        self.assertEqual(gps_port.electrical_attributes.get("voltage_domain"), "12V")
        self.assertEqual(len(gps_port.item_flows), 1)
        fix_flow = gps_port.item_flows[0]
        self.assertEqual(fix_flow.name, "gps_fix")
        self.assertEqual(fix_flow.direction, "out")
        self.assertEqual(fix_flow.item_type, "GPSFixMsg")
        self.assertEqual(fix_flow.rate_hz, 10.0)
        self.assertEqual(fix_flow.unit, "m")

    def test_parse_enhanced_connections_and_flows(self):
        """Verify SysMLParser parses connection blocks and connect statements with protocol, latency, and item flows."""
        sysml_text = """
        package InterconnectPackage {
            connection def ConnEthernet {
                doc /* High-speed link */
                connect MissionComputer.eth_out to DataRecorder.eth_in;
                attribute protocol : String = "Ethernet";
                attribute latency_ms : Real = 0.5;
                attribute item_flow_ref : String = "SensorPayloadStream";
            }

            connect SensorHub.can_out to FlightController.can_in [protocol: CAN] [latency: 2.0] flow of SensorTelemetry;

            flow from GuidanceUnit.pos_out to NavigationFilter.pos_in item PositionFlow;
        }
        """
        pkg = SysMLParser.parse_text(sysml_text)
        self.assertEqual(len(pkg.connection_defs), 3)

        c_map = {c.name: c for c in pkg.connection_defs}

        c_eth = c_map["ConnEthernet"]
        self.assertEqual(c_eth.source_part, "MissionComputer")
        self.assertEqual(c_eth.source_port, "MissionComputer.eth_out")
        self.assertEqual(c_eth.target_part, "DataRecorder")
        self.assertEqual(c_eth.target_port, "DataRecorder.eth_in")
        self.assertEqual(c_eth.protocol, "Ethernet")
        self.assertEqual(c_eth.latency_ms, 0.5)
        self.assertEqual(c_eth.item_flow_ref, "SensorPayloadStream")

        c_can = next(c for c in pkg.connection_defs if "SensorHub" in c.source_port)
        self.assertEqual(c_can.source_part, "SensorHub")
        self.assertEqual(c_can.target_part, "FlightController")
        self.assertEqual(c_can.protocol, "CAN")
        self.assertEqual(c_can.latency_ms, 2.0)
        self.assertEqual(c_can.item_flow_ref, "SensorTelemetry")

        c_pos = next(c for c in pkg.connection_defs if "GuidanceUnit" in c.source_port)
        self.assertEqual(c_pos.source_part, "GuidanceUnit")
        self.assertEqual(c_pos.target_part, "NavigationFilter")
        self.assertEqual(c_pos.item_flow_ref, "PositionFlow")


if __name__ == "__main__":
    unittest.main()
