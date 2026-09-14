#!/usr/bin/env python3
"""
Unit test suite for SchemaRouter extensions (Objective 1 / Issue #226).
Tests constructor dependency injection, priority ordering with prepend registration,
clean handling of non-schema files (.gitkeep, .DS_Store, .gitignore, dotfiles),
logging of unhandled extensions, and regex parsing of .sysml, .kerml, .md schema files.
"""

import os
import sys
import tempfile
import unittest
from typing import Dict, Optional, Tuple, Any

TEST_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(TEST_DIR, ".."))
PARITY_AUDITOR_SRC = os.path.join(PROJECT_ROOT, "skills", "spec-orchestrator", "parity_auditor", "src")

for p in (PROJECT_ROOT, PARITY_AUDITOR_SRC):
    if p not in sys.path:
        sys.path.insert(0, p)

from parity_auditor.core.workspace import WorkspaceRepository
from parity_auditor.parsers.base import IParser
from parity_auditor.parsers.schema_router import (
    SchemaRouter,
    parse_schema_file,
    SubsystemPort,
    SubsystemPart,
    extract_subsystem_parts,
)


class DummyCustomParser(IParser):
    def __init__(self, target_ext: str, module_name: str, definitions: Dict[str, str]):
        self.target_ext = target_ext
        self.module_name = module_name
        self.definitions = definitions

    def can_parse(self, filepath: str) -> bool:
        return filepath.endswith(self.target_ext)

    def parse(self, filepath: str) -> Tuple[Optional[str], Dict[str, str]]:
        return self.module_name, self.definitions


class TestSchemaRouterExtensions(unittest.TestCase):
    def setUp(self):
        self.repo = WorkspaceRepository(PROJECT_ROOT)

    def test_constructor_dependency_injection(self):
        custom_parser = DummyCustomParser(".custom", "CustomModule", {"custom:elem": "custom"})
        router = SchemaRouter(self.repo, parsers=[custom_parser])
        self.assertEqual(router._parsers, [custom_parser])
        self.assertTrue(router.can_parse("test_file.custom"))
        self.assertFalse(router.can_parse("test_file.yang"))

        mod_name, defs = router.parse("test_file.custom")
        self.assertEqual(mod_name, "CustomModule")
        self.assertEqual(defs, {"custom:elem": "custom"})

    def test_default_constructor_uses_regex_schema_parser(self):
        router = SchemaRouter(self.repo)
        self.assertEqual(len(router._parsers), 1)
        self.assertTrue(router.can_parse("model.sysml"))
        self.assertTrue(router.can_parse("model.yang"))

    def test_priority_order_with_register_prepend(self):
        router = SchemaRouter(self.repo, parsers=[])
        parser_low = DummyCustomParser(".dup", "LowPriorityModule", {"low:node": "low"})
        parser_high = DummyCustomParser(".dup", "HighPriorityModule", {"high:node": "high"})

        router.register(parser_low)
        router.register(parser_high, prepend=True)

        self.assertEqual(router._parsers[0], parser_high)
        self.assertEqual(router._parsers[1], parser_low)

        mod_name, defs = router.parse("sample.dup")
        self.assertEqual(mod_name, "HighPriorityModule")
        self.assertEqual(defs, {"high:node": "high"})

    def test_clean_handling_of_ignored_non_schema_files(self):
        router = SchemaRouter(self.repo)
        ignored_paths = [
            ".gitkeep",
            ".DS_Store",
            ".gitignore",
            ".env",
            ".hidden_config",
            "schema/.gitkeep",
            "schema/.DS_Store",
            "docs/schema/.gitignore",
        ]

        with self.assertLogs("parity_auditor.parsers.schema_router", level="WARNING") as cm:
            # Trigger a warning with an unhandled file to ensure assertLogs works
            router.parse("unhandled.dummy")
            # All ignored files should return (None, {}) without logging warnings
            for path in ignored_paths:
                mod_name, defs = router.parse(path)
                self.assertIsNone(mod_name)
                self.assertEqual(defs, {})

        # Ensure warning was logged ONLY for unhandled.dummy and not for any ignored files
        for record in cm.records:
            for ignored in ignored_paths:
                base = os.path.basename(ignored)
                self.assertNotIn(f"in {base}", record.getMessage())
        self.assertEqual(len(cm.records), 1)
        self.assertIn("unhandled.dummy", cm.records[0].getMessage())

    def test_warning_logging_for_unhandled_extensions(self):
        router = SchemaRouter(self.repo)
        with self.assertLogs("parity_auditor.parsers.schema_router", level="WARNING") as cm:
            mod_name, defs = router.parse("/workspace/schema/unsupported_format.xyz")
            self.assertEqual(mod_name, "unsupported_format.xyz")
            self.assertEqual(defs, {})

        self.assertEqual(len(cm.records), 1)
        self.assertIn(".xyz", cm.records[0].getMessage())
        self.assertIn("unsupported_format.xyz", cm.records[0].getMessage())

    def test_parse_schema_file_with_router_dependency_injection(self):
        custom_parser = DummyCustomParser(".ext", "InjectedModule", {"injected:key": "val"})
        custom_router = SchemaRouter(self.repo, parsers=[custom_parser])

        mod_name, defs = parse_schema_file("data.ext", repo=self.repo, router=custom_router)
        self.assertEqual(mod_name, "InjectedModule")
        self.assertEqual(defs, {"injected:key": "val"})

    def test_parsing_sysml_schema_file(self):
        content = """// SysML v2 Test Schema
package FlightControlSystem {
    part def FlightComputer;
    port def NavigationPort;
    action def CalculateTrajectory;
    state def ArmedState;
    constraint def MaxVelocityConstraint;
    requirement def SafetyRequirement;
    interface def BusInterface;
    item def TelemetryPacket;
    attribute def Altitude;
}
"""
        with tempfile.NamedTemporaryFile(suffix=".sysml", mode="w+", encoding="utf-8", delete=False) as f:
            f.write(content)
            f.flush()
            temp_path = f.name

        try:
            mod_name, defs = parse_schema_file(temp_path, repo=self.repo)
            self.assertEqual(mod_name, "FlightControlSystem")
            self.assertIn("part:FlightComputer", defs)
            self.assertIn("port:NavigationPort", defs)
            self.assertIn("action:CalculateTrajectory", defs)
            self.assertIn("state:ArmedState", defs)
            self.assertIn("constraint:MaxVelocityConstraint", defs)
            self.assertIn("requirement:SafetyRequirement", defs)
            self.assertIn("interface:BusInterface", defs)
            self.assertIn("item:TelemetryPacket", defs)
            self.assertIn("attribute:Altitude", defs)
        finally:
            if os.path.exists(temp_path):
                os.unlink(temp_path)

    def test_parsing_kerml_schema_file(self):
        content = """/* KerML Test Schema */
package CoreDimensions {
    feature def MathFeature;
    function def CalculateThrust;
    type def Vector3;
    classifier def SpatialEntity;
    datatype def Timestamp;
    struct def ConfigStruct;
    behavior def FlightBehavior;
    step def InitializationStep;
    dimension LengthDimension;
    unit Meter;
}
"""
        with tempfile.NamedTemporaryFile(suffix=".kerml", mode="w+", encoding="utf-8", delete=False) as f:
            f.write(content)
            f.flush()
            temp_path = f.name

        try:
            mod_name, defs = parse_schema_file(temp_path, repo=self.repo)
            self.assertEqual(mod_name, "CoreDimensions")
            self.assertIn("feature:MathFeature", defs)
            self.assertIn("function:CalculateThrust", defs)
            self.assertIn("type:Vector3", defs)
            self.assertIn("classifier:SpatialEntity", defs)
            self.assertIn("datatype:Timestamp", defs)
            self.assertIn("struct:ConfigStruct", defs)
            self.assertIn("behavior:FlightBehavior", defs)
            self.assertIn("step:InitializationStep", defs)
            self.assertIn("dimension:LengthDimension", defs)
            self.assertIn("unit:Meter", defs)
        finally:
            if os.path.exists(temp_path):
                os.unlink(temp_path)

    def test_parsing_md_schema_file(self):
        content = """# FlightDataDictionary

Specification of data types and entities.

concept FlightPlan
entity DroneTelemetry
type Coordinate3D
field TargetAltitude
property BatteryLevel
section DiagnosticsSection
structure WaypointStruct
model GuidanceModel
"""
        with tempfile.NamedTemporaryFile(suffix=".md", mode="w+", encoding="utf-8", delete=False) as f:
            f.write(content)
            f.flush()
            temp_path = f.name

        try:
            mod_name, defs = parse_schema_file(temp_path, repo=self.repo)
            self.assertEqual(mod_name, "FlightDataDictionary")
            self.assertIn("concept:FlightPlan", defs)
            self.assertIn("entity:DroneTelemetry", defs)
            self.assertIn("type:Coordinate3D", defs)
            self.assertIn("field:TargetAltitude", defs)
            self.assertIn("property:BatteryLevel", defs)
            self.assertIn("section:DiagnosticsSection", defs)
            self.assertIn("structure:WaypointStruct", defs)
            self.assertIn("model:GuidanceModel", defs)
        finally:
            if os.path.exists(temp_path):
                os.unlink(temp_path)

    def test_canonical_dataclasses(self):
        # SubsystemPort defaults
        port_default = SubsystemPort(name="telemetry")
        self.assertEqual(port_default.name, "telemetry")
        self.assertEqual(port_default.direction, "inout")
        self.assertEqual(port_default.direction, "INOUT")
        self.assertEqual(port_default.type_name, "Port")
        self.assertEqual(port_default.doc, "")

        port_custom = SubsystemPort(name="gps_in", direction="in", type_name="GpsMsg", doc="GPS input stream")
        self.assertEqual(port_custom.direction, "IN")
        self.assertEqual(port_custom.direction, "in")
        self.assertEqual(port_custom.to_dict(), {
            "name": "gps_in",
            "direction": "in",
            "type_name": "GpsMsg",
            "doc": "GPS input stream",
        })

        # SubsystemPart defaults
        part_default = SubsystemPart(name="FlightComputer")
        self.assertEqual(part_default.name, "FlightComputer")
        self.assertEqual(part_default.doc, "")
        self.assertEqual(part_default.ports, [])
        self.assertIsNone(part_default.mass_kg)
        self.assertIsNone(part_default.power_w)
        self.assertEqual(part_default.actions, [])
        self.assertEqual(part_default.attributes, {})
        self.assertEqual(part_default.constraints, [])

        part_full = SubsystemPart(
            name="NavigationUnit",
            doc="Navigation and INS unit",
            ports=[port_custom],
            mass_kg=1.85,
            power_w=32.0,
            actions=["alignIMU", "computeState"],
            attributes={"bus": "CAN", "voltage": 28.0},
            constraints=["LatencyMax10ms"],
        )
        self.assertEqual(part_full.mass_kg, 1.85)
        self.assertEqual(part_full.power_w, 32.0)
        self.assertEqual(len(part_full.ports), 1)
        part_dict = part_full.to_dict()
        self.assertEqual(part_dict["name"], "NavigationUnit")
        self.assertEqual(part_dict["mass_kg"], 1.85)
        self.assertEqual(part_dict["power_w"], 32.0)

    def test_extract_subsystem_parts_sysml(self):
        content = """package FlightSystem {
    doc /* Flight system top-level package */

    // Primary flight computer subsystem
    part def FlightComputer {
        doc /* Dual-redundant mission and flight computer */
        attribute mass_kg : Real = 1.5;
        attribute power_w : Real = 25.0;
        in port gps_in : GpsSignal;
        out port cmd_out : ActuatorCommand;
        inout port tlm_bus : TelemetryPort;
        action executeNavigation;
        assert constraint MaxPower;
    }

    part def BatteryPack {
        mass_kg = 4.2;
        power_w = 0.0;
        out port power_bus : DCRail;
    }
}
"""
        with tempfile.NamedTemporaryFile(suffix=".sysml", mode="w+", encoding="utf-8", delete=False) as f:
            f.write(content)
            f.flush()
            temp_path = f.name

        try:
            parts = extract_subsystem_parts(temp_path)
            self.assertEqual(len(parts), 2)

            fc = parts[0]
            self.assertEqual(fc.name, "FlightComputer")
            self.assertEqual(fc.doc, "Dual-redundant mission and flight computer")
            self.assertEqual(fc.mass_kg, 1.5)
            self.assertEqual(fc.power_w, 25.0)
            self.assertEqual(len(fc.ports), 3)

            p_gps = next(p for p in fc.ports if p.name == "gps_in")
            self.assertEqual(p_gps.direction, "in")
            self.assertEqual(p_gps.direction, "IN")
            self.assertEqual(p_gps.type_name, "GpsSignal")

            p_cmd = next(p for p in fc.ports if p.name == "cmd_out")
            self.assertEqual(p_cmd.direction, "out")
            self.assertEqual(p_cmd.direction, "OUT")
            self.assertEqual(p_cmd.type_name, "ActuatorCommand")

            p_tlm = next(p for p in fc.ports if p.name == "tlm_bus")
            self.assertEqual(p_tlm.direction, "inout")
            self.assertEqual(p_tlm.direction, "INOUT")
            self.assertEqual(p_tlm.type_name, "TelemetryPort")

            self.assertIn("executeNavigation", fc.actions)
            self.assertIn("MaxPower", fc.constraints)

            bp = parts[1]
            self.assertEqual(bp.name, "BatteryPack")
            self.assertEqual(bp.mass_kg, 4.2)
            self.assertEqual(bp.power_w, 0.0)
            self.assertEqual(len(bp.ports), 1)
            self.assertEqual(bp.ports[0].name, "power_bus")
            self.assertEqual(bp.ports[0].direction, "out")
        finally:
            if os.path.exists(temp_path):
                os.unlink(temp_path)

    def test_extract_subsystem_parts_yaml(self):
        # Test A: subsystems list
        yaml_content_a = """subsystems:
  - name: NavigationUnit
    doc: Multi-constellation GNSS and IMU
    mass_kg: 0.8
    power_w: 12.0
    ports:
      - name: rf_in
        direction: in
        type_name: RFSignal
      - name: nav_out
        direction: out
        type_name: NavSolution
    actions:
      - computeNav
    constraints:
      - UpdateRate50Hz
"""
        with tempfile.NamedTemporaryFile(suffix=".yaml", mode="w+", encoding="utf-8", delete=False) as f:
            f.write(yaml_content_a)
            f.flush()
            temp_path_a = f.name

        try:
            parts = extract_subsystem_parts(temp_path_a)
            self.assertEqual(len(parts), 1)
            nav = parts[0]
            self.assertEqual(nav.name, "NavigationUnit")
            self.assertEqual(nav.mass_kg, 0.8)
            self.assertEqual(nav.power_w, 12.0)
            self.assertEqual(len(nav.ports), 2)
            self.assertEqual(nav.ports[0].name, "rf_in")
            self.assertEqual(nav.ports[0].direction, "in")
            self.assertEqual(nav.ports[1].name, "nav_out")
            self.assertEqual(nav.ports[1].direction, "out")
            self.assertIn("computeNav", nav.actions)
            self.assertIn("UpdateRate50Hz", nav.constraints)
        finally:
            if os.path.exists(temp_path_a):
                os.unlink(temp_path_a)

        # Test B: components mapping in .yml
        yaml_content_b = """components:
  ActuatorCore:
    description: Distributed servo drive assembly
    mass: 2.3 kg
    power: 90 W
    ports:
      cmd_in: in
      status_out:
        direction: out
        type: StatusPacket
"""
        with tempfile.NamedTemporaryFile(suffix=".yml", mode="w+", encoding="utf-8", delete=False) as f:
            f.write(yaml_content_b)
            f.flush()
            temp_path_b = f.name

        try:
            parts_b = extract_subsystem_parts(temp_path_b)
            self.assertEqual(len(parts_b), 1)
            act = parts_b[0]
            self.assertEqual(act.name, "ActuatorCore")
            self.assertEqual(act.mass_kg, 2.3)
            self.assertEqual(act.power_w, 90.0)
            self.assertEqual(len(act.ports), 2)
            p_cmd = next(p for p in act.ports if p.name == "cmd_in")
            self.assertEqual(p_cmd.direction, "in")
            p_stat = next(p for p in act.ports if p.name == "status_out")
            self.assertEqual(p_stat.direction, "out")
        finally:
            if os.path.exists(temp_path_b):
                os.unlink(temp_path_b)

    def test_extract_subsystem_parts_json(self):
        json_content = """{
  "subsystems": [
    {
      "name": "RadioTransceiver",
      "doc": "C2 datalink transceiver",
      "mass_kg": 0.35,
      "power_w": 15.0,
      "ports": [
        {"name": "rf_antenna", "direction": "inout", "type_name": "RFFeed"},
        {"name": "c2_data", "direction": "inout", "type_name": "Ethernet"}
      ]
    },
    {
      "name": "PayloadCamera",
      "doc": "EO/IR electro-optical gimbal",
      "mass_kg": 1.1,
      "power_w": 22.0,
      "ports": [
        {"name": "video_stream", "direction": "out", "type_name": "H264Stream"}
      ]
    }
  ]
}
"""
        with tempfile.NamedTemporaryFile(suffix=".json", mode="w+", encoding="utf-8", delete=False) as f:
            f.write(json_content)
            f.flush()
            temp_path = f.name

        try:
            parts = extract_subsystem_parts(temp_path)
            self.assertEqual(len(parts), 2)
            radio = parts[0]
            self.assertEqual(radio.name, "RadioTransceiver")
            self.assertEqual(radio.mass_kg, 0.35)
            self.assertEqual(radio.power_w, 15.0)
            self.assertEqual(len(radio.ports), 2)

            cam = parts[1]
            self.assertEqual(cam.name, "PayloadCamera")
            self.assertEqual(cam.mass_kg, 1.1)
            self.assertEqual(cam.power_w, 22.0)
            self.assertEqual(len(cam.ports), 1)
            self.assertEqual(cam.ports[0].direction, "out")
        finally:
            if os.path.exists(temp_path):
                os.unlink(temp_path)

    def test_extract_subsystem_parts_proto(self):
        proto_content = """syntax = "proto3";

// Telemetry collection subsystem (mass_kg: 0.5, power_w: 10.0)
message TelemetryUnit {
    // Current GPS position input
    string gps_pos = 1;
    // System status flags
    uint32 status_flags = 2;
}

// Guidance controller service
service GuidanceService {
    // Compute guidance trajectory
    rpc ComputeGuidance (GuidanceRequest) returns (GuidanceResponse);
    // Stream telemetry
    rpc StreamTelemetry (TelemetryRequest) returns (stream TelemetryChunk);
}
"""
        with tempfile.NamedTemporaryFile(suffix=".proto", mode="w+", encoding="utf-8", delete=False) as f:
            f.write(proto_content)
            f.flush()
            temp_path = f.name

        try:
            parts = extract_subsystem_parts(temp_path)
            self.assertEqual(len(parts), 2)

            tu = parts[0]
            self.assertEqual(tu.name, "TelemetryUnit")
            self.assertEqual(tu.mass_kg, 0.5)
            self.assertEqual(tu.power_w, 10.0)
            self.assertEqual(len(tu.ports), 2)
            self.assertEqual(tu.ports[0].name, "gps_pos")
            self.assertEqual(tu.ports[0].type_name, "string")
            self.assertEqual(tu.ports[1].name, "status_flags")

            gs = parts[1]
            self.assertEqual(gs.name, "GuidanceService")
            self.assertEqual(len(gs.ports), 2)
            self.assertEqual(gs.ports[0].name, "ComputeGuidance")
            self.assertIn("ComputeGuidance", gs.actions)
            self.assertIn("StreamTelemetry", gs.actions)
        finally:
            if os.path.exists(temp_path):
                os.unlink(temp_path)

    def test_extract_subsystem_parts_idl(self):
        idl_content = """module FlightControl {
    // Flight manager interface
    interface FlightManager {
        // Step flight control loop
        void stepFlightLoop(in StateVector state, out CommandVector cmd);
        // Emergency stop command
        boolean emergencyStop();
    };

    // Subsystem component declaration
    component PowerManagementUnit {
        provides PowerPort pwrIn;
        uses BatteryPort batOut;
        in port CommandPort cmdIn;
    };
};
"""
        with tempfile.NamedTemporaryFile(suffix=".idl", mode="w+", encoding="utf-8", delete=False) as f:
            f.write(idl_content)
            f.flush()
            temp_path = f.name

        try:
            parts = extract_subsystem_parts(temp_path)
            self.assertEqual(len(parts), 2)

            fm = parts[0]
            self.assertEqual(fm.name, "FlightManager")
            self.assertEqual(fm.doc, "Flight manager interface")
            self.assertIn("stepFlightLoop", fm.actions)
            self.assertIn("emergencyStop", fm.actions)
            self.assertEqual(len(fm.ports), 2)

            pmu = parts[1]
            self.assertEqual(pmu.name, "PowerManagementUnit")
            self.assertEqual(len(pmu.ports), 3)
            p_in = next(p for p in pmu.ports if p.name == "pwrIn")
            self.assertEqual(p_in.direction, "in")
            p_out = next(p for p in pmu.ports if p.name == "batOut")
            self.assertEqual(p_out.direction, "out")
            p_cmd = next(p for p in pmu.ports if p.name == "cmdIn")
            self.assertEqual(p_cmd.direction, "in")
        finally:
            if os.path.exists(temp_path):
                os.unlink(temp_path)

    def test_extract_subsystem_parts_arxml(self):
        arxml_content = """<?xml version="1.0" encoding="UTF-8"?>
<AUTOSAR xmlns="http://autosar.org/schema/r4.0">
  <AR-PACKAGES>
    <AR-PACKAGE>
      <SHORT-NAME>AvionicsPkg</SHORT-NAME>
      <ELEMENTS>
        <APPLICATION-SW-COMPONENT-TYPE>
          <SHORT-NAME>SensorFusionSWC</SHORT-NAME>
          <DESC><L-2 L="EN">Primary sensor fusion SWC</L-2></DESC>
          <PORTS>
            <R-PORT-PROTOTYPE>
              <SHORT-NAME>GpsIn</SHORT-NAME>
              <REQUIRED-INTERFACE-TREF>/Interfaces/GpsIf</REQUIRED-INTERFACE-TREF>
            </R-PORT-PROTOTYPE>
            <P-PORT-PROTOTYPE>
              <SHORT-NAME>FusionOut</SHORT-NAME>
              <PROVIDED-INTERFACE-TREF>/Interfaces/FusionIf</PROVIDED-INTERFACE-TREF>
            </P-PORT-PROTOTYPE>
            <PR-PORT-PROTOTYPE>
              <SHORT-NAME>DiagBus</SHORT-NAME>
              <PROVIDED-REQUIRED-INTERFACE-TREF>/Interfaces/DiagIf</PROVIDED-REQUIRED-INTERFACE-TREF>
            </PR-PORT-PROTOTYPE>
          </PORTS>
          <INTERNAL-BEHAVIORS>
            <SWC-INTERNAL-BEHAVIOR>
              <RUNNABLES>
                <RUNNABLE-ENTITY>
                  <SHORT-NAME>StepFusion</SHORT-NAME>
                </RUNNABLE-ENTITY>
              </RUNNABLES>
            </SWC-INTERNAL-BEHAVIOR>
          </INTERNAL-BEHAVIORS>
        </APPLICATION-SW-COMPONENT-TYPE>
        <SENSOR-ACTUATOR-SW-COMPONENT-TYPE>
          <SHORT-NAME>MotorActuatorSWC</SHORT-NAME>
          <DESC><L-2 L="EN">Motor actuator driver</L-2></DESC>
          <PORTS>
            <R-PORT-PROTOTYPE>
              <SHORT-NAME>MotorCmd</SHORT-NAME>
            </R-PORT-PROTOTYPE>
          </PORTS>
        </SENSOR-ACTUATOR-SW-COMPONENT-TYPE>
      </ELEMENTS>
    </AR-PACKAGE>
  </AR-PACKAGES>
</AUTOSAR>
"""
        with tempfile.NamedTemporaryFile(suffix=".arxml", mode="w+", encoding="utf-8", delete=False) as f:
            f.write(arxml_content)
            f.flush()
            temp_path = f.name

        try:
            parts = extract_subsystem_parts(temp_path)
            self.assertEqual(len(parts), 2)

            sf = parts[0]
            self.assertEqual(sf.name, "SensorFusionSWC")
            self.assertEqual(sf.doc, "Primary sensor fusion SWC")
            self.assertEqual(len(sf.ports), 3)

            p_in = next(p for p in sf.ports if p.name == "GpsIn")
            self.assertEqual(p_in.direction, "in")
            self.assertEqual(p_in.direction, "IN")

            p_out = next(p for p in sf.ports if p.name == "FusionOut")
            self.assertEqual(p_out.direction, "out")
            self.assertEqual(p_out.direction, "OUT")

            p_pr = next(p for p in sf.ports if p.name == "DiagBus")
            self.assertEqual(p_pr.direction, "inout")
            self.assertEqual(p_pr.direction, "INOUT")

            self.assertIn("StepFusion", sf.actions)

            ma = parts[1]
            self.assertEqual(ma.name, "MotorActuatorSWC")
            self.assertEqual(len(ma.ports), 1)
            self.assertEqual(ma.ports[0].direction, "in")
        finally:
            if os.path.exists(temp_path):
                os.unlink(temp_path)

    def test_extract_subsystem_parts_markdown(self):
        md_content = """# Super-System Subsystems

The primary subsystems are defined below:

| Subsystem | Description | Mass (kg) | Power (W) | Interfaces / Ports |
| :--- | :--- | :--- | :--- | :--- |
| Flight Computer | Dual-redundant core flight computer | 1.5 | 25.0 | PORT_FCS_C2 (INOUT), PORT_FCS_CMD (OUT), PORT_FCS_TLM (IN) |
| Sensor Suite | Multi-sensor IMU, barometer, and GNSS | 0.45 | 6.5 | PORT_SENSOR_OUT : ImuPacket (OUT) |
| Actuator Drive | Distributed ESC and motor control | 2.1 | 120.0 | PORT_ACT_IN (IN) |
"""
        with tempfile.NamedTemporaryFile(suffix=".md", mode="w+", encoding="utf-8", delete=False) as f:
            f.write(md_content)
            f.flush()
            temp_path = f.name

        try:
            parts = extract_subsystem_parts(temp_path)
            self.assertEqual(len(parts), 3)

            fc = parts[0]
            self.assertEqual(fc.name, "Flight Computer")
            self.assertEqual(fc.doc, "Dual-redundant core flight computer")
            self.assertEqual(fc.mass_kg, 1.5)
            self.assertEqual(fc.power_w, 25.0)
            self.assertEqual(len(fc.ports), 3)
            self.assertEqual(fc.ports[0].name, "PORT_FCS_C2")
            self.assertEqual(fc.ports[0].direction, "inout")
            self.assertEqual(fc.ports[1].name, "PORT_FCS_CMD")
            self.assertEqual(fc.ports[1].direction, "out")
            self.assertEqual(fc.ports[2].name, "PORT_FCS_TLM")
            self.assertEqual(fc.ports[2].direction, "in")

            ss = parts[1]
            self.assertEqual(ss.name, "Sensor Suite")
            self.assertEqual(ss.mass_kg, 0.45)
            self.assertEqual(ss.power_w, 6.5)
            self.assertEqual(ss.ports[0].type_name, "ImuPacket")
            self.assertEqual(ss.ports[0].direction, "out")

            ad = parts[2]
            self.assertEqual(ad.name, "Actuator Drive")
            self.assertEqual(ad.mass_kg, 2.1)
            self.assertEqual(ad.power_w, 120.0)
            self.assertEqual(ad.ports[0].direction, "in")
        finally:
            if os.path.exists(temp_path):
                os.unlink(temp_path)

    def test_extract_subsystem_parts_directory_search(self):
        with tempfile.TemporaryDirectory() as td:
            # Create .sysml
            sysml_file = os.path.join(td, "core.sysml")
            with open(sysml_file, "w", encoding="utf-8") as f:
                f.write("part def SysmlComponent { in port p1 : Sig; }")

            # Create .yaml
            yaml_file = os.path.join(td, "subsystems.yaml")
            with open(yaml_file, "w", encoding="utf-8") as f:
                f.write("subsystems:\n  - name: YamlComponent\n    mass_kg: 1.0\n")

            # Create ignored files
            with open(os.path.join(td, ".gitkeep"), "w", encoding="utf-8") as f:
                f.write("")
            with open(os.path.join(td, ".DS_Store"), "w", encoding="utf-8") as f:
                f.write("")

            parts = extract_subsystem_parts(td)
            names = [p.name for p in parts]
            self.assertIn("SysmlComponent", names)
            self.assertIn("YamlComponent", names)
            self.assertEqual(len(parts), 2)

    def test_extract_subsystem_parts_list_of_paths(self):
        content_1 = "part def UnitOne { in port p1 : Sig; }"
        content_2 = "part def UnitTwo { out port p2 : Sig; }"
        with tempfile.NamedTemporaryFile(suffix=".sysml", mode="w+", encoding="utf-8", delete=False) as f1, \
             tempfile.NamedTemporaryFile(suffix=".sysml", mode="w+", encoding="utf-8", delete=False) as f2:
            f1.write(content_1)
            f1.flush()
            f2.write(content_2)
            f2.flush()
            p1, p2 = f1.name, f2.name

        try:
            parts = extract_subsystem_parts([p1, p2])
            names = [p.name for p in parts]
            self.assertEqual(names, ["UnitOne", "UnitTwo"])
        finally:
            for p in (p1, p2):
                if os.path.exists(p):
                    os.unlink(p)

    def test_extract_subsystem_parts_via_schema_router(self):
        content = "part def RoutedUnit { inout port p_bus : Bus; }"
        with tempfile.NamedTemporaryFile(suffix=".sysml", mode="w+", encoding="utf-8", delete=False) as f:
            f.write(content)
            f.flush()
            temp_path = f.name

        try:
            router = SchemaRouter(self.repo)
            parts = router.extract_subsystem_parts(temp_path)
            self.assertEqual(len(parts), 1)
            self.assertEqual(parts[0].name, "RoutedUnit")
            self.assertEqual(parts[0].ports[0].name, "p_bus")
            self.assertEqual(parts[0].ports[0].direction, "inout")
        finally:
            if os.path.exists(temp_path):
                os.unlink(temp_path)


if __name__ == "__main__":
    unittest.main()

