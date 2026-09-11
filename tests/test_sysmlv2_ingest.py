# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Unit tests for SysML v2 Universal Ingestion Engine AST-scoped structural filtering
and negative invariant projection.

Realises: [SpecName/SysMLv2UniversalIngestASTFiltering]
Verifies that:
1. filter_ast_to_target_scope() filters extraneous PartDef nodes, nested subparts,
   and sub-packages to match allowed_parts.
2. Related cross-cutting AST nodes (connections, hazards, risks, capabilities, item_defs)
   referencing excluded parts are cleanly pruned.
3. Negative exclusion invariants (assert constraint assert_exclusion_* { !exists(*); })
   are projected as SysMLConstraintDef instances.
4. ingest_schema() generates filtered SysML v2 textual output and accurate schema-digest.json
   without phantom entities.
5. CLI entrypoint supports --allowed-parts and --negative-invariants flags.
"""

import json
import os
import subprocess
import sys
import tempfile
import unittest

# Ensure scripts directory is on sys.path
PROJECT_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
SPEC_SCRIPTS_DIR = os.path.join(PROJECT_ROOT, "skills", "spec-orchestrator", "scripts")
if SPEC_SCRIPTS_DIR not in sys.path:
    sys.path.insert(0, SPEC_SCRIPTS_DIR)

from sysmlv2_ast import (
    SysMLPackage,
    PartDef,
    PortDef,
    AttributeDef,
    ActionDef,
    SysMLCapabilityDef,
    SysMLConstraintDef,
    ItemDef,
    HazardDef,
    RiskDef,
    ConnectionDef,
    SysMLParser,
)
from sysmlv2_ingest import filter_ast_to_target_scope, ingest_schema

INGEST_SCRIPT_PATH = os.path.join(SPEC_SCRIPTS_DIR, "sysmlv2_ingest.py")


class TestSysMLv2IngestASTFiltering(unittest.TestCase):
    """Test suite verifying AST-scoped structural filtering and negative invariant projection."""

    def test_filter_ast_prunes_extraneous_parts(self) -> None:
        """PartDef instances not in allowed_parts must be pruned from top-level package."""
        pkg = SysMLPackage(
            name="FlightSystem",
            part_defs=[
                PartDef(name="FlightController", doc="Authoritative flight controller"),
                PartDef(name="PhantomRadar", doc="Legacy reference radar (out of scope)"),
                PartDef(name="AuxiliaryGPS", doc="External sensor (out of scope)"),
            ],
        )

        filtered = filter_ast_to_target_scope(pkg, allowed_parts=["FlightController"])

        part_names = [p.name for p in filtered.part_defs]
        self.assertEqual(part_names, ["FlightController"])
        self.assertNotIn("PhantomRadar", part_names)
        self.assertNotIn("AuxiliaryGPS", part_names)

    def test_filter_ast_handles_string_and_set_inputs(self) -> None:
        """allowed_parts accepts comma-separated string or set."""
        pkg = SysMLPackage(
            name="AvionicsPackage",
            part_defs=[
                PartDef(name="FCC"),
                PartDef(name="INS"),
                PartDef(name="LegacyTransponder"),
            ],
        )

        # Comma-separated string
        filtered_str = filter_ast_to_target_scope(pkg, allowed_parts="FCC, INS")
        self.assertEqual({p.name for p in filtered_str.part_defs}, {"FCC", "INS"})

        # Set
        filtered_set = filter_ast_to_target_scope(pkg, allowed_parts={"FCC"})
        self.assertEqual([p.name for p in filtered_set.part_defs], ["FCC"])

    def test_filter_ast_nested_subparts_and_subpackages(self) -> None:
        """Nested sub-parts and sub-packages must be recursively filtered."""
        sub_pkg = SysMLPackage(
            name="TelemetrySubsystem",
            part_defs=[
                PartDef(name="TelemetryBuffer"),
                PartDef(name="UndeclaredModem"),
            ],
        )
        parent_part = PartDef(
            name="MissionComputer",
            parts=[
                PartDef(name="CoreProcessor"),
                PartDef(name="LegacyCoProcessor"),
            ],
        )
        root_pkg = SysMLPackage(
            name="RootSystem",
            part_defs=[parent_part, PartDef(name="PhantomEntity")],
            sub_packages=[sub_pkg],
        )

        filtered = filter_ast_to_target_scope(
            root_pkg,
            allowed_parts=["MissionComputer", "CoreProcessor", "TelemetryBuffer"],
        )

        self.assertEqual(len(filtered.part_defs), 1)
        self.assertEqual(filtered.part_defs[0].name, "MissionComputer")
        subpart_names = [sub.name for sub in filtered.part_defs[0].parts]
        self.assertEqual(subpart_names, ["CoreProcessor"])

        self.assertEqual(len(filtered.sub_packages), 1)
        subpkg_part_names = [p.name for p in filtered.sub_packages[0].part_defs]
        self.assertEqual(subpkg_part_names, ["TelemetryBuffer"])

    def test_filter_ast_prunes_cross_cutting_references(self) -> None:
        """Connections, hazards, risks, and capabilities referencing excluded parts must be pruned."""
        pkg = SysMLPackage(
            name="IntegratedSystem",
            part_defs=[
                PartDef(name="SensorHub"),
                PartDef(name="LegacyActuator"),
            ],
            capability_defs=[
                SysMLCapabilityDef(name="DataLogging", subsystem="SensorHub"),
                SysMLCapabilityDef(name="LegacyControl", subsystem="LegacyActuator"),
            ],
            hazard_defs=[
                HazardDef(name="SensorLoss", part_ref="SensorHub"),
                HazardDef(name="ActuatorJam", part_ref="LegacyActuator"),
            ],
            risk_defs=[
                RiskDef(name="SensorRisk", hazard_ref="SensorLoss"),
                RiskDef(name="ActuatorRisk", hazard_ref="ActuatorJam"),
            ],
            connection_defs=[
                ConnectionDef(name="ValidBus", source_port="SensorHub.out", target_port="SensorHub.in"),
                ConnectionDef(name="DeadBus", source_port="SensorHub.out", target_port="LegacyActuator.in"),
            ],
            item_defs=[
                ItemDef(name="SensorHubPayload"),
                ItemDef(name="LegacyActuatorPacket"),
            ],
        )

        filtered = filter_ast_to_target_scope(pkg, allowed_parts=["SensorHub"])

        self.assertEqual([p.name for p in filtered.part_defs], ["SensorHub"])
        self.assertEqual([c.name for c in filtered.capability_defs], ["DataLogging"])
        self.assertEqual([h.name for h in filtered.hazard_defs], ["SensorLoss"])
        self.assertEqual([r.name for r in filtered.risk_defs], ["SensorRisk"])
        self.assertEqual([conn.name for conn in filtered.connection_defs], ["ValidBus"])
        self.assertEqual([i.name for i in filtered.item_defs], ["SensorHubPayload"])

    def test_filter_ast_projects_negative_invariants(self) -> None:
        """Negative invariants must be projected as assert constraint definitions."""
        pkg = SysMLPackage(name="CleanModel")

        filtered = filter_ast_to_target_scope(
            pkg,
            negative_invariants=["PhantomRadar", "Legacy Transponder", "Uncertified_GPS"],
        )

        constraint_names = [c.name for c in filtered.constraint_defs]
        self.assertIn("assert_exclusion_phantomradar", constraint_names)
        self.assertIn("assert_exclusion_legacy_transponder", constraint_names)
        self.assertIn("assert_exclusion_uncertified_gps", constraint_names)

        c_map = {c.name: c for c in filtered.constraint_defs}
        c1 = c_map["assert_exclusion_phantomradar"]
        self.assertTrue(c1.is_assertion)
        self.assertEqual(c1.expression, "!exists(PhantomRadar)")
        self.assertIn("PhantomRadar", c1.doc)

        sysml_output = filtered.to_sysml()
        self.assertIn("assert constraint assert_exclusion_phantomradar {", sysml_output)
        self.assertIn("!exists(PhantomRadar);", sysml_output)

    def test_filter_ast_negative_invariants_idempotency(self) -> None:
        """Projecting the same negative invariants twice must not duplicate constraint definitions."""
        pkg = SysMLPackage(name="IdempotentModel")
        filter_ast_to_target_scope(pkg, negative_invariants=["PhantomRadar"])
        filter_ast_to_target_scope(pkg, negative_invariants=["PhantomRadar"])

        constraint_names = [c.name for c in pkg.constraint_defs]
        self.assertEqual(constraint_names.count("assert_exclusion_phantomradar"), 1)

    def test_ingest_schema_end_to_end_with_filtering_and_negative_invariants(self) -> None:
        """ingest_schema must filter AST, project negative invariants, and emit accurate digest."""
        sysml_content = """package UAS_Mission_Package {
    part def AutopilotComputer {
        attribute cpuLoad : Real = 0.3;
    }
    part def LegacyAuxiliaryCamera {
        attribute res : String = "1080p";
    }
    part def UndeclaredTelemetryTransceiver {
        attribute freq : Real = 915.0;
    }
}"""
        with tempfile.TemporaryDirectory() as tmpdir:
            input_file = os.path.join(tmpdir, "input_schema.sysml")
            output_sysml = os.path.join(tmpdir, "output.sysml")
            digest_json = os.path.join(tmpdir, "schema-digest.json")

            with open(input_file, "w", encoding="utf-8") as f:
                f.write(sysml_content)

            pkg, digest = ingest_schema(
                schema_path=input_file,
                output_path=output_sysml,
                digest_path=digest_json,
                allowed_parts=["AutopilotComputer"],
                negative_invariants=["LegacyAuxiliaryCamera", "UndeclaredTelemetryTransceiver"],
            )

            # Check parsed AST package
            part_names = [p.name for p in pkg.part_defs]
            self.assertEqual(part_names, ["AutopilotComputer"])
            self.assertNotIn("LegacyAuxiliaryCamera", part_names)
            self.assertNotIn("UndeclaredTelemetryTransceiver", part_names)

            constraint_names = [c.name for c in pkg.constraint_defs]
            self.assertIn("assert_exclusion_legacyauxiliarycamera", constraint_names)
            self.assertIn("assert_exclusion_undeclaredtelemetrytransceiver", constraint_names)

            # Check written .sysml file
            with open(output_sysml, "r", encoding="utf-8") as f:
                written_sysml = f.read()

            self.assertIn("part def AutopilotComputer", written_sysml)
            self.assertNotIn("part def LegacyAuxiliaryCamera", written_sysml)
            self.assertNotIn("part def UndeclaredTelemetryTransceiver", written_sysml)
            self.assertIn("assert constraint assert_exclusion_legacyauxiliarycamera", written_sysml)
            self.assertIn("!exists(LegacyAuxiliaryCamera);", written_sysml)

            # Check digest JSON
            with open(digest_json, "r", encoding="utf-8") as f:
                digest_data = json.load(f)

            schema_nodes = digest_data["schema_nodes"]
            self.assertIn("AutopilotComputer", schema_nodes)
            self.assertNotIn("LegacyAuxiliaryCamera", schema_nodes)
            self.assertNotIn("UndeclaredTelemetryTransceiver", schema_nodes)
            self.assertIn("assert_exclusion_legacyauxiliarycamera", schema_nodes)
            self.assertIn("assert_exclusion_undeclaredtelemetrytransceiver", schema_nodes)

    def test_cli_execution_with_flags(self) -> None:
        """CLI invocation with --allowed-parts and --negative-invariants must filter and project properly."""
        idl_content = """module Surveillance {
    struct DroneCore {
        long seqId;
    };
    struct ExtraneousSensor {
        long temp;
    };
};"""
        with tempfile.TemporaryDirectory() as tmpdir:
            input_file = os.path.join(tmpdir, "interfaces.idl")
            output_sysml = os.path.join(tmpdir, "cli_output.sysml")
            digest_json = os.path.join(tmpdir, "cli_digest.json")

            with open(input_file, "w", encoding="utf-8") as f:
                f.write(idl_content)

            cmd = [
                sys.executable,
                INGEST_SCRIPT_PATH,
                "--schema",
                input_file,
                "--out",
                output_sysml,
                "--digest",
                digest_json,
                "--allowed-parts",
                "DroneCore",
                "--negative-invariants",
                "ExtraneousSensor",
            ]
            res = subprocess.run(cmd, capture_output=True, text=True)
            self.assertEqual(res.returncode, 0, f"CLI failed: {res.stderr}")

            with open(output_sysml, "r", encoding="utf-8") as f:
                sysml_out = f.read()

            self.assertIn("part def DroneCore", sysml_out)
            self.assertNotIn("part def ExtraneousSensor", sysml_out)
            self.assertIn("assert constraint assert_exclusion_extraneoussensor", sysml_out)


if __name__ == "__main__":
    unittest.main()
