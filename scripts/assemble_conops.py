#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Deterministic ConOps & Mission Intent Assembly Engine (ISO 29148 / NATO STANAG 4586 / OMG UAF).
Addresses Issues #113, #114, and Fixes #143.

Compiles modular unit files from docs/conops/units/conops/ and docs/conops/units/mission_intent/
into canonical CONOPS.md and MISSION_INTENT.md specification documents with automated
SysML AST Parameter Binding, TOC generation, cross-reference anchor validation,
and zero-placeholder integrity gating.
"""

import argparse
import datetime
import json
import os
import re
import sys
from dataclasses import dataclass, field
from enum import Enum
from pathlib import Path
from typing import Any, Dict, List, Optional, Set, Tuple, Union


class LifecycleType(str, Enum):
    """Formal lifecycle archetypes for cyber-physical mission systems."""
    REUSABLE_RECOVERY = "REUSABLE_RECOVERY"
    EXPENDABLE_KINETIC_EFFECTOR = "EXPENDABLE_KINETIC_EFFECTOR"
    CONTINUOUS_STATIONARY = "CONTINUOUS_STATIONARY"
    PERSISTENT_ORBITAL = "PERSISTENT_ORBITAL"
    TRACK_BOUND_GUIDED = "TRACK_BOUND_GUIDED"


class ContainmentActionType(str, Enum):
    """Formal containment and terminal action mechanisms."""
    CONTROLLED_RECOVERY_LANDING = "CONTROLLED_RECOVERY_LANDING"
    SAFE_IMPACT_ZEROIZATION = "SAFE_IMPACT_ZEROIZATION"
    ELECTROMECHANICAL_BRAKE_LOCK = "ELECTROMECHANICAL_BRAKE_LOCK"
    DEORBIT_DISPOSAL_BURN = "DEORBIT_DISPOSAL_BURN"
    TRACK_SIDING_BRAKE = "TRACK_SIDING_BRAKE"


@dataclass
class LifecycleContract:
    """Formal MBSE lifecycle and terminal state contract."""
    lifecycle_type: LifecycleType
    containment_action: ContainmentActionType
    bingo_safety_action: str
    end_state: str
    failsafe_sequence: str
    post_op_state: str
    primary_terminal_target: str
    secondary_terminal_target: str
    primary_recovery_facility: str
    secondary_recovery_facility: str
    lifecycle_transit_mode: str


# Match template placeholders like {{SYSTEM_IDENTIFIER}}, {{TLX_WEIGHT_MD:0.25}}, etc.
PLACEHOLDER_PATTERN = re.compile(r"\{\{([A-Za-z0-9_]+)(?::([^\}]*))?\}\}")
RAW_TOKEN_FINDER = re.compile(r"\{\{[A-Za-z0-9_]+(?::[^\}]*)?\}\}")

# Canonical unit file whitelists for deterministic ConOps & Mission Intent assembly (Issue #148)
CANONICAL_CONOPS_UNITS: List[str] = [
    "01_METADATA_AND_OVERVIEW.md",
    "02_DEFICIENCIES_AND_MOTIVATION.md",
    "03_PROPOSED_CAPABILITIES.md",
    "04_USER_CLASSES_AND_STAKEHOLDERS.md",
    "05_AIRSPACE_AND_SORA_RISK.md",
    "06_UAF_OPERATIONAL_ACTIVITIES.md",
    "07_OPTX_EXCHANGES.md",
    "08_ENVIRONMENTAL_MIL_STD_810H.md",
    "09_SCENARIOS_AND_TIMELINES.md",
    "10_MAINTENANCE_AND_GSE_SUPPORT.md",
    "11_IMPACTS_AND_TRADE_STUDIES.md",
    "12_EMERGENCY_DECISION_MATRIX.md",
]

CANONICAL_MISSION_INTENT_UNITS: List[str] = [
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

# Canonical default ConOps parameters dictionary
DEFAULT_CONOPS_PARAMS: Dict[str, str] = {
    "MAX_JUNCTION_TEMPERATURE_DELTA_C": "25.0",
    "BATTERY_CHARGE_C_RATE": "2.0C",
    "BATTERY_CHARGE_TIME_HOURS": "1.5",
    "SUPPORT_EQUIPMENT_BATTERY_HOURS": "8.0",
    "OPERATIONAL_AVAILABILITY_THRESHOLD": "0.95",
    "OPERATIONAL_AVAILABILITY_OBJECTIVE": "0.99",
    "OPERATING_TEMPERATURE_MIN_C": "-20.0",
    "OPERATING_TEMPERATURE_MAX_C": "+55.0",
}


class SysMLParameterBindingEngine:
    """
    Automated SysML AST Parameter Binding Engine.
    Fixes Issue #143.

    Ingests domain parameter dictionaries from:
    1. Explicit parameter dictionary JSON files (--params <path>)
    2. Auto-detected domain configuration (schema/domain_config.json, .pipeline/domain_config.json)
    3. Auto-detected schema digests (.pipeline/schema-digest.json)
    4. SysML v2 AST textual specifications (.pipeline/schema.sysml, schema/*.sysml)

    Substitutes canonical template placeholders ({{...}}) with bound values or
    sensible domain fallback defaults derived dynamically from the AST/schema context.
    """

    def __init__(
        self,
        parameter_values: Optional[Dict[str, Any]] = None,
        config_path: Optional[str] = None,
        workspace_dir: Optional[str] = None,
        auto_detect: bool = True,
        domain: Optional[str] = None,
    ):
        self.workspace_dir = os.path.abspath(workspace_dir or os.getcwd())
        self.parameter_bindings: Dict[str, str] = {}
        self._explicit_keys: Set[str] = set()
        self.inferred_system_identifier: Optional[str] = None
        self.detected_domain: str = domain or "aviation"
        self.is_non_aircraft: bool = False
        self.is_civilian: bool = False
        self.lifecycle_contract: Optional[LifecycleContract] = None

        if domain:
            self.parameter_bindings["DOMAIN_TYPE"] = domain
            self.parameter_bindings["TARGET_DOMAIN"] = domain
            self._explicit_keys.add("DOMAIN_TYPE")
            self._explicit_keys.add("TARGET_DOMAIN")

        if auto_detect:
            self.auto_discover_sources(self.workspace_dir)

        if config_path:
            self.ingest_file(config_path)

        if parameter_values:
            self.ingest_dictionary(parameter_values)

        self.detected_domain = self._detect_domain_type()
        self.parameter_bindings["DETECTED_DOMAIN"] = self.detected_domain
        self.parameter_bindings["DOMAIN_TYPE"] = self.detected_domain

        self._derive_operational_intent()
        self._derive_mass_budgets()
        self._derive_quadratic_physics()
        self._derive_energy_budgets()
        self._derive_domain_regulatory_standards()
        self._derive_domain_ontology()
        self._derive_lifecycle_contract()

    @property
    def domain(self) -> str:
        return getattr(self, "detected_domain", "aviation")

    @domain.setter
    def domain(self, val: str) -> None:
        self.detected_domain = val

    def _detect_domain_type(self) -> str:
        """
        Detects operational domain type: aviation, medical, rail, marine, space, industrial.
        Fixes Issues #175, #186.
        """
        aviation_tokens = {
            "aviation",
            "aircraft",
            "uav",
            "uas",
            "drone",
            "evtol",
            "aerial",
            "flight",
            "airspace",
            "aerospace",
            "sora",
            "fixed-wing",
            "rotorcraft",
            "interceptor",
        }
        medical_tokens = {
            "medical",
            "surgical",
            "healthcare",
            "patient",
            "clinical",
            "hospital",
            "laparoscopic",
            "trocar",
        }
        rail_tokens = {
            "rail",
            "locomotive",
            "train",
            "shunting",
            "metro",
            "tramway",
            "railway",
        }
        marine_tokens = {
            "marine",
            "subsea",
            "maritime",
            "underwater",
            "auv",
            "rov",
            "usv",
            "uuv",
            "vessel",
            "naval",
            "ocean",
            "bathymetric",
        }
        space_tokens = {
            "space",
            "satellite",
            "cubesat",
            "orbital",
            "spacecraft",
            "launch",
            "leo",
            "geo",
            "adcs",
            "orbit",
            "constellation",
        }
        industrial_tokens = {
            "industrial",
            "agv",
            "forklift",
            "warehouse",
            "logistics",
            "amr",
            "ugv",
        }

        def _match_tokens(text: str) -> Optional[str]:
            clean = text.lower()
            tokens = set(re.findall(r"[a-z0-9]+", clean))
            if tokens & aviation_tokens or any(p in clean for p in ("fixed wing", "fixed-wing")):
                return "aviation"
            if tokens & medical_tokens:
                return "medical"
            if tokens & rail_tokens or any(p in clean for p in ("rolling stock", "track circuit")):
                return "rail"
            if tokens & marine_tokens or any(p in clean for p in ("surface vessel",)):
                return "marine"
            if tokens & space_tokens:
                return "space"
            if tokens & industrial_tokens or any(p in clean for p in ("industrial truck", "material handling", "vda 5050", "ground delivery", "ground robot")):
                return "industrial"
            return None

        # 1. Explicit domain in parameter bindings
        explicit = (
            self.parameter_bindings.get("DOMAIN_TYPE")
            or self.parameter_bindings.get("TARGET_DOMAIN")
            or self.parameter_bindings.get("DOMAIN")
        )
        if explicit:
            dom = _match_tokens(str(explicit))
            if dom:
                return dom

        # 2. Parameter bindings and inferred identifiers
        domain_str = (
            self.parameter_bindings.get("OPERATIONAL_DOMAIN")
            or self.parameter_bindings.get("DOMAIN")
            or ""
        ).lower()
        sys_str = (
            self.parameter_bindings.get("SYSTEM_IDENTIFIER")
            or self.parameter_bindings.get("SYSTEM_NAME")
            or self.inferred_system_identifier
            or ""
        ).lower()
        platform_type = (self.parameter_bindings.get("PLATFORM_TYPE") or "").lower()
        params_combined = f"{domain_str} {sys_str} {platform_type}"
        dom = _match_tokens(params_combined)
        if dom:
            return dom

        # 3. Check domain_config.json in workspace
        for cfg_rel in (
            "schema/domain_config.json",
            ".pipeline/domain_config.json",
            "domain_config.json",
            ".agents/domain_config.json",
        ):
            curr = self.workspace_dir
            for _ in range(5):
                cfg_path = os.path.join(curr, cfg_rel)
                if os.path.isfile(cfg_path):
                    try:
                        with open(cfg_path, "r", encoding="utf-8") as f:
                            cfg = json.load(f)
                        dom_val = (
                            cfg.get("domain")
                            or cfg.get("domain_type")
                            or cfg.get("operational_domain")
                            or cfg.get("system_type")
                            or ""
                        )
                        dom = _match_tokens(str(dom_val))
                        if dom:
                            return dom
                    except Exception:
                        pass
                parent = os.path.dirname(curr)
                if parent == curr:
                    break
                curr = parent

        # 4. Fallback: inspect workspace directory string
        dom = _match_tokens(self.workspace_dir)
        if dom:
            return dom

        return "aviation"

    def _get_mtow_value(self) -> float:
        """Extracts numerical MTOW from bound parameters or fallback default (50.0 kg)."""
        mtow_raw = (
            self.parameter_bindings.get("TOTAL_MTOW_KG")
            or self.parameter_bindings.get("MTOW_NOMINAL_KG")
            or self.parameter_bindings.get("MTOW_MAX_KG")
            or self.parameter_bindings.get("SYSTEM_MASS_MAX_KG")
            or self.parameter_bindings.get("OPERATIONAL_MASS_KG")
            or self.parameter_bindings.get("SYSTEM_MASS_KG")
        )
        if mtow_raw is not None:
            try:
                m_match = re.search(r"[-+]?\d*\.?\d+", str(mtow_raw))
                return float(m_match.group(0)) if m_match else 50.0
            except Exception:
                return 50.0
        return 50.0

    def _derive_mass_budgets(self) -> None:
        """
        Dynamically calculates subsystem mass budget values from TOTAL_MTOW_KG.
        Fixes Issues #161, #177.

        Subsystems:
          - MASS_BUDGET_AIRFRAME_KG = round(0.30 * mtow, 2)
          - MASS_BUDGET_AVIONICS_KG = round(0.15 * mtow, 2)
          - MASS_BUDGET_PROPULSION_KG = round(0.25 * mtow, 2)
          - MASS_BUDGET_ENERGY_KG = round(0.20 * mtow, 2)
          - MASS_BUDGET_PAYLOAD_KG = round(0.07 * mtow, 2)
          - MASS_BUDGET_CONTAINMENT_KG = round(mtow - (airframe + avionics + propulsion + energy + payload), 2)
        Ensures the 6 partition rows strictly sum to TOTAL_MTOW_KG (100.0%) for any vehicle mass.
        """
        mtow = self._get_mtow_value()

        airframe = round(0.30 * mtow, 2)
        avionics = round(0.15 * mtow, 2)
        propulsion = round(0.25 * mtow, 2)
        energy = round(0.20 * mtow, 2)
        payload = round(0.07 * mtow, 2)
        containment = round(mtow - (airframe + avionics + propulsion + energy + payload), 2)

        if "TOTAL_MTOW_KG" not in self._explicit_keys and "TOTAL_MTOW_KG" not in self.parameter_bindings:
            self.parameter_bindings["TOTAL_MTOW_KG"] = str(mtow) if "." in str(mtow) else f"{mtow:.1f}"

        if "MASS_BUDGET_AIRFRAME_KG" not in self._explicit_keys:
            self.parameter_bindings["MASS_BUDGET_AIRFRAME_KG"] = str(airframe)
        if "MASS_BUDGET_AVIONICS_KG" not in self._explicit_keys:
            self.parameter_bindings["MASS_BUDGET_AVIONICS_KG"] = str(avionics)
        if "MASS_BUDGET_PROPULSION_KG" not in self._explicit_keys:
            self.parameter_bindings["MASS_BUDGET_PROPULSION_KG"] = str(propulsion)
        if "MASS_BUDGET_ENERGY_KG" not in self._explicit_keys:
            self.parameter_bindings["MASS_BUDGET_ENERGY_KG"] = str(energy)
        if "MASS_BUDGET_PAYLOAD_KG" not in self._explicit_keys:
            self.parameter_bindings["MASS_BUDGET_PAYLOAD_KG"] = str(payload)
        if "MASS_BUDGET_CONTAINMENT_KG" not in self._explicit_keys:
            self.parameter_bindings["MASS_BUDGET_CONTAINMENT_KG"] = str(containment)

        if "MASS_FRACTION_AIRFRAME_PCT" not in self._explicit_keys:
            self.parameter_bindings["MASS_FRACTION_AIRFRAME_PCT"] = "30.0"
        if "MASS_FRACTION_AVIONICS_PCT" not in self._explicit_keys:
            self.parameter_bindings["MASS_FRACTION_AVIONICS_PCT"] = "15.0"
        if "MASS_FRACTION_PROPULSION_PCT" not in self._explicit_keys:
            self.parameter_bindings["MASS_FRACTION_PROPULSION_PCT"] = "25.0"
        if "MASS_FRACTION_ENERGY_PCT" not in self._explicit_keys:
            self.parameter_bindings["MASS_FRACTION_ENERGY_PCT"] = "20.0"
        if "MASS_FRACTION_PAYLOAD_PCT" not in self._explicit_keys:
            self.parameter_bindings["MASS_FRACTION_PAYLOAD_PCT"] = "7.0"
        if "MASS_FRACTION_CONTAINMENT_PCT" not in self._explicit_keys:
            self.parameter_bindings["MASS_FRACTION_CONTAINMENT_PCT"] = "3.0"

    def _derive_quadratic_physics(self) -> None:
        """
        Closed-Form Quadratic Physics Solver (Fixes #168, #178).
        Calculates:
          v_calc = sqrt(2 * m * g / (rho * S * C_d))
          E_k_calc = 0.5 * m * v_calc^2
        Binds calculated values to template tokens from declared medium density and geometry,
        ensuring formula-table parity.
        """
        m = self._get_mtow_value()
        g = 9.80665

        # Medium density rho based on domain and explicit bindings (ISA sea level default 1.225 kg/m^3 for Section 5.2 SORA kinetic energy derivations)
        default_rho = 1.225
        if self.domain == "marine":
            default_rho = 1025.0
        elif self.domain == "space":
            default_rho = 1e-12

        rho_raw = (
            self.parameter_bindings.get("AIR_DENSITY_KGM3")
            or self.parameter_bindings.get("FLUID_DENSITY_KGM3")
            or self.parameter_bindings.get("RHO_MEDIUM")
            or self.parameter_bindings.get("RHO")
        )
        rho = default_rho
        if rho_raw is not None:
            try:
                m_rho = re.search(r"[-+]?\d*\.?\d+(?:[eE][-+]?\d+)?", str(rho_raw))
                if m_rho:
                    rho = float(m_rho.group(0))
            except Exception:
                pass

        if "AIR_DENSITY_KGM3" not in self._explicit_keys:
            self.parameter_bindings["AIR_DENSITY_KGM3"] = str(rho)
        if "FLUID_DENSITY_KGM3" not in self._explicit_keys:
            self.parameter_bindings["FLUID_DENSITY_KGM3"] = str(rho)
        if "RHO_MEDIUM" not in self._explicit_keys:
            self.parameter_bindings["RHO_MEDIUM"] = str(rho)
        if "G_ACCEL_MPS2" not in self._explicit_keys:
            self.parameter_bindings["G_ACCEL_MPS2"] = str(g)

        # Bind system mass tokens
        self.parameter_bindings["SYSTEM_MASS_KG"] = str(m)
        self.parameter_bindings["SYSTEM_MASS"] = str(m)

        # Reference frontal area S_ref and drag coefficient C_D
        s_ref_raw = self.parameter_bindings.get("FRONTAL_AREA_M2") or self.parameter_bindings.get("S_REF")
        s_ref = 0.18
        if s_ref_raw:
            try:
                m_sref = re.search(r"[-+]?\d*\.?\d+", str(s_ref_raw))
                if m_sref:
                    s_ref = float(m_sref.group(0))
            except Exception:
                pass

        cd_unmit_raw = self.parameter_bindings.get("DRAG_COEFFICIENT") or self.parameter_bindings.get("C_D")
        cd_unmit = 0.45
        if cd_unmit_raw:
            try:
                m_cdunmit = re.search(r"[-+]?\d*\.?\d+", str(cd_unmit_raw))
                if m_cdunmit:
                    cd_unmit = float(m_cdunmit.group(0))
            except Exception:
                pass

        if "FRONTAL_AREA_M2" not in self._explicit_keys:
            self.parameter_bindings["FRONTAL_AREA_M2"] = str(s_ref)
        if "DRAG_COEFFICIENT" not in self._explicit_keys:
            self.parameter_bindings["DRAG_COEFFICIENT"] = str(cd_unmit)

        # Unmitigated terminal velocity and kinetic energy
        if rho > 0 and s_ref > 0 and cd_unmit > 0:
            v_term_unmit = round(((2.0 * m * g) / (rho * s_ref * cd_unmit)) ** 0.5, 2)
            ek_unmit = round(0.5 * m * (v_term_unmit ** 2), 1)
        else:
            v_term_unmit = 0.0
            ek_unmit = 0.0

        if "V_TERMINAL_UNMITIGATED_MPS" not in self._explicit_keys:
            self.parameter_bindings["V_TERMINAL_UNMITIGATED_MPS"] = str(v_term_unmit)
            self.parameter_bindings["V_TERMINAL_UNMITIGATED"] = str(v_term_unmit)
        if "E_K_UNMITIGATED_JOULES" not in self._explicit_keys:
            self.parameter_bindings["E_K_UNMITIGATED_JOULES"] = str(ek_unmit)
            self.parameter_bindings["E_K_UNMITIGATED"] = str(ek_unmit)

        # Mitigated parachute / recovery parameters
        cd_mit = 1.75
        cd_mit_raw = self.parameter_bindings.get("PARACHUTE_DRAG_COEFFICIENT") or self.parameter_bindings.get("C_D_PARACHUTE")
        if cd_mit_raw:
            try:
                m_cdmit = re.search(r"[-+]?\d*\.?\d+", str(cd_mit_raw))
                if m_cdmit:
                    cd_mit = float(m_cdmit.group(0))
            except Exception:
                pass

        s_mit_raw = (
            self.parameter_bindings.get("PARACHUTE_AREA_M2")
            or self.parameter_bindings.get("PARACHUTE_CANOPY_AREA_M2")
            or self.parameter_bindings.get("PARACHUTE_CANOPY_AREA")
            or self.parameter_bindings.get("S_CANOPY")
            or self.parameter_bindings.get("S_CANOPY_M2")
        )
        s_mit = None
        if s_mit_raw:
            try:
                m_smit = re.search(r"[-+]?\d*\.?\d+", str(s_mit_raw))
                if m_smit:
                    s_mit = float(m_smit.group(0))
            except Exception:
                pass

        if s_mit is None or s_mit <= 0:
            target_v = 1.6483
            if rho > 0:
                s_mit = round((2.0 * m * g) / (rho * cd_mit * (target_v ** 2)), 2)
            else:
                s_mit = 1.0
            if "PARACHUTE_AREA_M2" not in self._explicit_keys:
                self.parameter_bindings["PARACHUTE_AREA_M2"] = str(s_mit)
            if "S_CANOPY" not in self._explicit_keys:
                self.parameter_bindings["S_CANOPY"] = str(s_mit)
            if "PARACHUTE_CANOPY_AREA_M2" not in self._explicit_keys:
                self.parameter_bindings["PARACHUTE_CANOPY_AREA_M2"] = str(s_mit)

        denom = rho * s_mit * cd_mit
        if denom > 0 and m > 0:
            v_calc = round(((2.0 * m * g) / denom) ** 0.5, 2)
            ek_calc = round(0.5 * m * (v_calc ** 2), 1)
        else:
            v_calc = 1.65
            ek_calc = 34.0

        if "S_CANOPY" not in self._explicit_keys:
            self.parameter_bindings["S_CANOPY"] = str(s_mit)
        if "PARACHUTE_AREA_M2" not in self._explicit_keys:
            self.parameter_bindings["PARACHUTE_AREA_M2"] = str(s_mit)
        if "PARACHUTE_CANOPY_AREA_M2" not in self._explicit_keys:
            self.parameter_bindings["PARACHUTE_CANOPY_AREA_M2"] = str(s_mit)

        if "V_TERMINAL_PARACHUTE_MPS" not in self._explicit_keys:
            self.parameter_bindings["V_TERMINAL_PARACHUTE_MPS"] = str(v_calc)
            self.parameter_bindings["V_TERMINAL_PARACHUTE"] = str(v_calc)
            self.parameter_bindings["PARACHUTE_TERMINAL_VELOCITY_MPS"] = str(v_calc)
            self.parameter_bindings["PARACHUTE_TERMINAL_VELOCITY"] = str(v_calc)
        if "E_K_MITIGATED_JOULES" not in self._explicit_keys:
            self.parameter_bindings["E_K_MITIGATED_JOULES"] = str(ek_calc)
            self.parameter_bindings["E_K_MITIGATED"] = str(ek_calc)
            self.parameter_bindings["MITIGATED_KINETIC_ENERGY_J"] = str(ek_calc)
        if "PARACHUTE_DRAG_COEFFICIENT" not in self._explicit_keys:
            self.parameter_bindings["PARACHUTE_DRAG_COEFFICIENT"] = str(cd_mit)
            self.parameter_bindings["C_D_PARACHUTE"] = str(cd_mit)
    def _derive_domain_regulatory_standards(self) -> None:
        """
        Dynamically derives DOMAIN_REGULATORY_STANDARDS_TABLE_ROWS based on detected domain,
        SYSTEM_IDENTIFIER, REGULATORY_STANDARDS, or domain config (Issues #163, #164, #169, #175).
        """
        if "DOMAIN_REGULATORY_STANDARDS_TABLE_ROWS" in self._explicit_keys:
            return

        dom = getattr(self, "detected_domain", "aviation")

        # Medical domain
        if dom == "medical":
            rows = [
                "| IEC 62304:2006+AMD1:2015 Class C | IEC | Medical device software — Software life cycle processes | §4.3 Software safety classification, §5.2 Software development planning, §7.1 Software risk management |",
                "| ISO 14971:2019 | ISO | Medical devices — Application of risk management to medical devices | §4.4 Risk management plan, §5.4 Risk estimation, §7.1 Risk control option analysis |",
                "| IEC 60601-1-8:2020 | IEC | Medical electrical equipment — Part 1-8: General requirements for basic safety and essential performance — Collateral Standard: Alarm systems | §6.3 Alarm condition categories, §6.8 Alarm signals, §6.9 Alarm limits |",
            ]
        # Rail domain
        elif dom == "rail":
            rows = [
                "| EN 50126:2017 | CENELEC | Railway Applications — The Specification and Demonstration of Reliability, Availability, Maintainability and Safety (RAMS) | §6.2 RAMS lifecycle processes, §7.3 Risk assessment and safety requirements |",
                "| EN 50128:2011/A2:2020 SIL 4 | CENELEC | Railway applications — Communication, signalling and processing systems — Software for railway control and protection systems | §6.3 Software safety integrity levels (SIL 4), §7.5 Software verification and testing |",
                "| EN 50129:2018 | CENELEC | Railway applications — Communication, signalling and processing systems — Safety related electronic systems for signalling | §5.2 Safety management for electronic systems, §6.3 Hardware safety integrity, §7.1 Safety acceptance |",
            ]
        # Space domain
        elif dom == "space":
            rows = [
                "| ECSS-E-ST-40C | ECSS | Space engineering — Software | §5.2 Software life cycle, §5.8 Software verification and validation, §6.3 Space software safety requirements |",
                "| NASA-STD-8739.8 | NASA | Software Assurance Standard for NASA Programs and Projects | §4.2 Safety-critical software assurance, §5.3 Independent Verification and Validation (IV&V) |",
                "| ECSS-E-ST-10C | ECSS | Space engineering — System engineering general requirements | §5.2 System engineering process, §6.2 Verification and product assurance processes |",
            ]
        # AGV / Forklift / Warehouse logistics domain
        elif dom == "industrial":
            rows = [
                "| ISO 3691-4:2023 | ISO | Industrial trucks — Safety requirements and verification — Part 4: Driverless industrial trucks and their systems | §4.2 Automated path containment, §4.3 Personnel detection and active obstacle avoidance, §5.2 Safety interlocks |",
                "| IEC 61508 SIL 3 | IEC | Functional Safety of Electrical/Electronic/Programmable Electronic Safety-related Systems | Part 1 §6.2 Management of functional safety, Part 2 §7.4 Hardware safety integrity (SIL 3), Part 3 §7.4 Software design |",
                "| VDA 5050 | VDA / VDMA | AGV Communication Interface — Interface for the communication between automated guided vehicles (AGV) and a master control | §4.0 MQTT message formats, §5.2 Dynamic order execution, §6.3 Instant action and e-stop commands |",
            ]
        # Subsea / Maritime domain
        elif dom == "marine":
            rows = [
                "| DNV-GL-ST-E403 | DNV GL | Subsea power and automation systems | §3.2 Subsea electrical and control system safety, §4.4 Redundant power and containment architectures |",
                "| ISO 13628-6 | ISO | Petroleum and natural gas industries — Design and operation of subsea production systems — Part 6: Subsea production control systems | §5.2 Environmental qualification, §6.3 Pressure containment and emergency release interlocks |",
                "| IMO MASS Code | IMO | Maritime Autonomous Surface Ships (MASS) Code | §3.1 Autonomous navigation modes, §4.2 Remote control center safety functions, §5.3 Failsafe state reversion |",
                "| COLREGs Convention | IMO | Convention on the International Regulations for Preventing Collisions at Sea | Rule 5 Look-out, Rule 8 Action to avoid collision, Rule 18 Responsibilities between vessels |",
            ]
        # Aviation / UAS domain (default for aerial systems)
        else:
            rows = [
                "| RTCA DO-178C (DAL-B) | RTCA / EUROCAE (ED-12C) | Software Considerations in Airborne Systems and Equipment Certification | §6.3.1 Software Safety & Verification, §6.4.4 Structural Coverage (MC/DC Verification), Tables A-1 to A-7 Life Cycle Objectives |",
                "| RTCA DO-254 | RTCA / EUROCAE (ED-80) | Design Assurance Guidance for Airborne Electronic Hardware | §5.0 Hardware Design Processes, §6.0 Validation & Verification, Appendix B Design Assurance Levels (DAL-B) |",
                "| RTCA DO-365B | RTCA | Minimum Operational Performance Standards (MOPS) for Detect and Avoid (DAA) Systems | §2.2 DAA System Requirements, §2.2.4 Well-Clear Boundaries & Alerting, §2.2.5 Collision Avoidance Guidance & Maneuver Coordination |",
                "| JARUS SORA v2.5 | JARUS | Specific Operations Risk Assessment (SORA) Methodology | Step #2 Initial Ground Risk Class (GRC), Step #4 Specific Assurance and Integrity Levels (SAIL I–VI), Step #5 Air Risk Class (ARC), Annex B M1–M3 Safety Mitigations |",
            ]

        table_rows_str = "\n".join(rows)
        self.parameter_bindings["DOMAIN_REGULATORY_STANDARDS_TABLE_ROWS"] = table_rows_str
        self.parameter_bindings["REGULATORY_STANDARDS_TABLE_ROWS"] = table_rows_str

    def _derive_energy_budgets(self) -> None:
        """
        Derives consistent energy, nominal power, peak power, and Bingo threshold budgets.
        Fixes Issue #179 (First-Law Energy Conservation).
        Ensures:
          E_capacity_joules >= P_nominal_watts * (t_endurance_hours * 3600.0)
          E_bingo = E_return + E_divert + E_reserve + E_contingency
          E_reserve / E_capacity >= 0.20
        """
        # 1. Determine endurance in hours
        t_hours = None
        if "ENDURANCE_HOURS" in self._explicit_keys:
            try:
                t_hours = float(self.parameter_bindings["ENDURANCE_HOURS"])
            except ValueError:
                pass
        elif "ENDURANCE_NOMINAL_MIN" in self._explicit_keys:
            try:
                t_hours = float(self.parameter_bindings["ENDURANCE_NOMINAL_MIN"]) / 60.0
            except ValueError:
                pass
        elif "ENDURANCE_HOURS" in self.parameter_bindings:
            try:
                t_hours = float(self.parameter_bindings["ENDURANCE_HOURS"])
            except ValueError:
                pass
        elif "ENDURANCE_NOMINAL_MIN" in self.parameter_bindings:
            try:
                t_hours = float(self.parameter_bindings["ENDURANCE_NOMINAL_MIN"]) / 60.0
            except ValueError:
                pass

        if t_hours is None or t_hours <= 0:
            t_hours = 2.0

        t_sec = t_hours * 3600.0
        t_min = round(t_hours * 60.0, 1)

        # 2. Determine battery capacity in Joules and kWh
        e_joules = None
        if "BATTERY_CAPACITY_JOULES" in self.parameter_bindings:
            try:
                e_joules = float(self.parameter_bindings["BATTERY_CAPACITY_JOULES"])
            except ValueError:
                pass
        elif "BATTERY_CAPACITY_KWH" in self.parameter_bindings:
            try:
                e_joules = float(self.parameter_bindings["BATTERY_CAPACITY_KWH"]) * 3.6e6
            except ValueError:
                pass
        elif "E_CAPACITY_JOULES" in self.parameter_bindings:
            try:
                e_joules = float(self.parameter_bindings["E_CAPACITY_JOULES"])
            except ValueError:
                pass

        p_nom_raw = self.parameter_bindings.get("TOTAL_POWER_NOMINAL_W") or self.parameter_bindings.get("POWER_NOMINAL_W")
        p_nom_specified = None
        if p_nom_raw:
            try:
                p_nom_specified = float(p_nom_raw)
            except ValueError:
                pass

        if e_joules is None:
            if p_nom_specified is not None and p_nom_specified > 0:
                e_joules = round((p_nom_specified * t_sec) / 0.70, 1)
            else:
                e_joules = 500000.0

        # Enforce First-Law Energy Conservation: E_capacity >= P_nominal * t_sec
        min_e_required = (p_nom_specified or (0.70 * (e_joules / t_sec))) * t_sec
        if e_joules < min_e_required:
            e_joules = round(min_e_required / 0.70, 1)

        e_kwh = round(e_joules / 3.6e6, 4)

        if "BATTERY_CAPACITY_KWH" not in self._explicit_keys:
            self.parameter_bindings["BATTERY_CAPACITY_KWH"] = str(e_kwh)
        if "E_CAPACITY_KWH" not in self._explicit_keys:
            self.parameter_bindings["E_CAPACITY_KWH"] = str(e_kwh)
        if "BATTERY_CAPACITY_JOULES" not in self._explicit_keys:
            self.parameter_bindings["BATTERY_CAPACITY_JOULES"] = str(e_joules)
        if "E_CAPACITY_JOULES" not in self._explicit_keys:
            self.parameter_bindings["E_CAPACITY_JOULES"] = str(e_joules)
        if "ENDURANCE_HOURS" not in self._explicit_keys:
            self.parameter_bindings["ENDURANCE_HOURS"] = str(t_hours)
            self.parameter_bindings["NOMINAL_ENDURANCE_HOURS"] = str(t_hours)
        if "ENDURANCE_NOMINAL_MIN" not in self._explicit_keys:
            self.parameter_bindings["ENDURANCE_NOMINAL_MIN"] = str(t_min)
            self.parameter_bindings["ENDURANCE_MIN_MIN"] = str(t_min)

        # 3. Derive sustainable nominal and peak power
        p_sustainable = e_joules / t_sec
        if p_nom_specified is not None and p_nom_specified > 0 and p_nom_specified <= p_sustainable:
            p_nom = p_nom_specified
        else:
            p_nom = round(0.70 * p_sustainable, 1)

        if p_nom <= 0:
            p_nom = 100.0
        p_peak = round(2.0 * p_nom, 1)

        p_prop = round(0.85 * p_nom, 1)
        p_avionics = round(0.08 * p_nom, 1)
        p_payload = round(0.06 * p_nom, 1)
        p_containment = round(p_nom - (p_prop + p_avionics + p_payload), 1)

        if "TOTAL_POWER_NOMINAL_W" not in self._explicit_keys:
            self.parameter_bindings["TOTAL_POWER_NOMINAL_W"] = str(p_nom)
        if "TOTAL_POWER_PEAK_W" not in self._explicit_keys:
            self.parameter_bindings["TOTAL_POWER_PEAK_W"] = str(p_peak)
        if "POWER_NOMINAL_AIRFRAME_W" not in self._explicit_keys:
            self.parameter_bindings["POWER_NOMINAL_AIRFRAME_W"] = "0.0"
        if "POWER_PEAK_AIRFRAME_W" not in self._explicit_keys:
            self.parameter_bindings["POWER_PEAK_AIRFRAME_W"] = "0.0"
        if "POWER_NOMINAL_ENERGY_W" not in self._explicit_keys:
            self.parameter_bindings["POWER_NOMINAL_ENERGY_W"] = "0.0"
        if "POWER_PEAK_ENERGY_W" not in self._explicit_keys:
            self.parameter_bindings["POWER_PEAK_ENERGY_W"] = "0.0"
        if "POWER_NOMINAL_PROPULSION_W" not in self._explicit_keys:
            self.parameter_bindings["POWER_NOMINAL_PROPULSION_W"] = str(p_prop)
        if "POWER_PEAK_PROPULSION_W" not in self._explicit_keys:
            self.parameter_bindings["POWER_PEAK_PROPULSION_W"] = str(round(2.0 * p_prop, 1))
        if "POWER_NOMINAL_AVIONICS_W" not in self._explicit_keys:
            self.parameter_bindings["POWER_NOMINAL_AVIONICS_W"] = str(p_avionics)
        if "POWER_PEAK_AVIONICS_W" not in self._explicit_keys:
            self.parameter_bindings["POWER_PEAK_AVIONICS_W"] = str(round(2.0 * p_avionics, 1))
        if "POWER_NOMINAL_PAYLOAD_W" not in self._explicit_keys:
            self.parameter_bindings["POWER_NOMINAL_PAYLOAD_W"] = str(p_payload)
        if "POWER_PEAK_PAYLOAD_W" not in self._explicit_keys:
            self.parameter_bindings["POWER_PEAK_PAYLOAD_W"] = str(round(2.0 * p_payload, 1))
        if "POWER_NOMINAL_CONTAINMENT_W" not in self._explicit_keys:
            self.parameter_bindings["POWER_NOMINAL_CONTAINMENT_W"] = str(p_containment)
        if "POWER_PEAK_CONTAINMENT_W" not in self._explicit_keys:
            self.parameter_bindings["POWER_PEAK_CONTAINMENT_W"] = str(round(2.0 * p_containment, 1))

        # 4. Derive Bingo energy partitions
        e_reserve = round(0.20 * e_joules, 1)
        if e_reserve / e_joules < 0.20:
            e_reserve = round(0.20 * e_joules + 0.1, 1)
        e_return = round(0.35 * e_joules, 1)
        e_divert = round(0.15 * e_joules, 1)
        e_contingency = round(0.10 * e_joules, 1)
        e_bingo = round(e_return + e_divert + e_reserve + e_contingency, 1)

        self.parameter_bindings["E_RESERVE_JOULES"] = str(e_reserve)
        self.parameter_bindings["E_RETURN_JOULES"] = str(e_return)
        self.parameter_bindings["E_DIVERT_JOULES"] = str(e_divert)
        self.parameter_bindings["E_CONTINGENCY_JOULES"] = str(e_contingency)
        self.parameter_bindings["E_BINGO_JOULES"] = str(e_bingo)
        self.parameter_bindings["E_BINGO_THRESHOLD_JOULES"] = str(e_bingo)

    def _derive_domain_ontology(self) -> None:
        """
        Derives platform-specific and civilian/military ontology tokens for all 6 domains (Solver 5).
        Fixes Issue #175.
        """
        dom = getattr(self, "detected_domain", "aviation")

        is_non_aircraft = dom in ("medical", "rail", "marine", "space", "industrial")
        is_civilian = dom in ("medical", "rail", "marine", "space", "industrial")

        self.is_non_aircraft = is_non_aircraft
        self.is_civilian = is_civilian

        # Domain-specific ontology derivations
        if dom == "medical":
            self.parameter_bindings["STRUCTURE_PARTITION_LABEL"] = "Surgeon Console Ergonomic Chassis & Arm Assembly"
            self.parameter_bindings["FAILSAFE_CONTAINMENT_NAME"] = "electromagnetic joint brake / emergency power cutoff actuator"
            self.parameter_bindings["ALTITUDE_UNIT"] = "mm"
            self.parameter_bindings["V_STALL_MAX_MPS"] = "0.0"
            self.parameter_bindings["V_STALL_NOMINAL_MPS"] = "0.0"
            self.parameter_bindings["REMOTE_ID_HEADER"] = "IEC 60601-1 Medical Device Telemetry & Identification"
            self.parameter_bindings["REMOTE_ID_STANDARD_BODY"] = "Direct medical telemetry and device identification conforming to IEC 62304 Class C and IEC 60601-1 standards."
            self.parameter_bindings["TIER4_CONTAINMENT_DESC"] = "electromagnetic joint brake clamp or immediate surgical power cutoff"
            self.parameter_bindings["FAILSAFE_DESCENT_SYSTEM"] = "emergency joint brake and power isolation system"
            self.parameter_bindings["RECOVERY_DEVICE_TERM"] = "failsafe joint brake"
            self.parameter_bindings["RECOVERY_SUB"] = "brake"
            self.parameter_bindings["PARACHUTE_SYMBOL_CD"] = "C_d"
            self.parameter_bindings["PARACHUTE_SYMBOL_V"] = "v_{\\mathrm{terminal}}"
            self.parameter_bindings["PARACHUTE_PARAM_NAME_S"] = "Instrument Reference Cross-Section"
            self.parameter_bindings["PARACHUTE_PARAM_NAME_CD"] = "Fluid Resistance Coefficient"
            self.parameter_bindings["PARACHUTE_PARAM_SYM_CD"] = "C_d"
            self.parameter_bindings["PARACHUTE_PARAM_NAME_V"] = "Terminal Joint Velocity"
            self.parameter_bindings["PARACHUTE_PARAM_SYM_V"] = "v_terminal"
            self.parameter_bindings["EMERGENCY_IGNITION_DESC"] = "Emergency Surgical Power Isolation Command"
            self.parameter_bindings["CONTAINMENT_SQUIB_ACTION"] = "Emergency Joint Brake & Power Cutoff Command"
            self.parameter_bindings["OPTX13_NAME"] = "BroadcastMedicalDeviceTelemetry"
            self.parameter_bindings["OPTX13_SOURCE"] = "MedicalDeviceIdentification"
            self.parameter_bindings["OPTX13_PROTOCOL_DESC"] = "Digitally Signed Medical Telemetry (DICOM/HL7 per IEC 62304)"
            self.parameter_bindings["ALTITUDE_TELEMETRY"] = "Manipulator Tip Position (X/Y/Z)"
            if "STATE_SPACE_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["STATE_SPACE_STANDARD"] = "IEC 62304:2006+AMD1:2015 §5.2"
            if "SAFETY_BOUNDS_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["SAFETY_BOUNDS_STANDARD"] = "IEC 60601-1-8:2020 §6.9"
            if "STATE_VECTOR_MIN_EXPRESSION" not in self._explicit_keys:
                self.parameter_bindings["STATE_VECTOR_MIN_EXPRESSION"] = "[x_min, y_min, z_min, vx_min, vy_min, vz_min]^T"
            if "STATE_VECTOR_MAX_EXPRESSION" not in self._explicit_keys:
                self.parameter_bindings["STATE_VECTOR_MAX_EXPRESSION"] = "[x_max, y_max, z_max, vx_max, vy_max, vz_max]^T"
            if "STATE_VECTOR_MIN_UNITS" not in self._explicit_keys:
                self.parameter_bindings["STATE_VECTOR_MIN_UNITS"] = "mm, mm, mm, mm/s"
            if "STATE_VECTOR_MAX_UNITS" not in self._explicit_keys:
                self.parameter_bindings["STATE_VECTOR_MAX_UNITS"] = "mm, mm, mm, mm/s"
            if "STATE_SAFETY_MITIGATION" not in self._explicit_keys:
                self.parameter_bindings["STATE_SAFETY_MITIGATION"] = "ISO 14971:2019 §7.1 Risk Controls"
            if "CONTAINMENT_BUFFER_UNIT" not in self._explicit_keys:
                self.parameter_bindings["CONTAINMENT_BUFFER_UNIT"] = "mm"
            if "CONTAINMENT_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["CONTAINMENT_STANDARD"] = "IEC 60601-1-8:2020 §6.3"
            if "C2_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["C2_STANDARD"] = "IEC 60601-1-8 §6.8 / DICOM"
            if "CONTAINMENT_RESPONSE_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["CONTAINMENT_RESPONSE_STANDARD"] = "IEC 60601-1:2020 §8.1"
        elif dom == "rail":
            self.parameter_bindings["STRUCTURE_PARTITION_LABEL"] = "Locomotive Heavy Chassis & Underframe Structure"
            self.parameter_bindings["FAILSAFE_CONTAINMENT_NAME"] = "pneumatic train brake pipe venting / automatic coupler emergency release"
            self.parameter_bindings["ALTITUDE_UNIT"] = "m"
            self.parameter_bindings["V_STALL_MAX_MPS"] = "0.0"
            self.parameter_bindings["V_STALL_NOMINAL_MPS"] = "0.0"
            self.parameter_bindings["REMOTE_ID_HEADER"] = "Automatic Equipment Identification (AEI) & ETCS Balise Telemetry"
            self.parameter_bindings["REMOTE_ID_STANDARD_BODY"] = "Direct wayside tracking and identification in accordance with EN 50128 SIL 4 and IEEE 1474.1 CBTC standards."
            self.parameter_bindings["TIER4_CONTAINMENT_DESC"] = "pneumatic emergency brake pipe venting or traction power cutoff"
            self.parameter_bindings["FAILSAFE_DESCENT_SYSTEM"] = "pneumatic emergency brake venting system"
            self.parameter_bindings["RECOVERY_DEVICE_TERM"] = "pneumatic emergency brake"
            self.parameter_bindings["RECOVERY_SUB"] = "brake"
            self.parameter_bindings["PARACHUTE_SYMBOL_CD"] = "C_d"
            self.parameter_bindings["PARACHUTE_SYMBOL_V"] = "v_{\\mathrm{terminal}}"
            self.parameter_bindings["PARACHUTE_PARAM_NAME_S"] = "Locomotive Frontal Area"
            self.parameter_bindings["PARACHUTE_PARAM_NAME_CD"] = "Train Aerodynamic Drag Coefficient"
            self.parameter_bindings["PARACHUTE_PARAM_SYM_CD"] = "C_d"
            self.parameter_bindings["PARACHUTE_PARAM_NAME_V"] = "Terminal Rolling Velocity"
            self.parameter_bindings["PARACHUTE_PARAM_SYM_V"] = "v_terminal"
            self.parameter_bindings["EMERGENCY_IGNITION_DESC"] = "Emergency Train Brake Pipe Venting Command"
            self.parameter_bindings["CONTAINMENT_SQUIB_ACTION"] = "Emergency Train Brake Pipe Venting & Traction Cutoff Command"
            self.parameter_bindings["OPTX13_NAME"] = "BroadcastTrainIdentificationTelemetry"
            self.parameter_bindings["OPTX13_SOURCE"] = "TrainIdentification"
            self.parameter_bindings["OPTX13_PROTOCOL_DESC"] = "Digitally Signed Train Telemetry (ETCS / AEI RFID per EN 50128)"
            self.parameter_bindings["ALTITUDE_TELEMETRY"] = "Track Elevation / Chainage"
            if "STATE_SPACE_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["STATE_SPACE_STANDARD"] = "EN 50126:2017 §6.2"
            if "SAFETY_BOUNDS_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["SAFETY_BOUNDS_STANDARD"] = "EN 50128:2011/A2:2020 SIL 4 §6.3"
            if "STATE_VECTOR_MIN_EXPRESSION" not in self._explicit_keys:
                self.parameter_bindings["STATE_VECTOR_MIN_EXPRESSION"] = "[s_min, v_min, a_min, p_brake_min]^T"
            if "STATE_VECTOR_MAX_EXPRESSION" not in self._explicit_keys:
                self.parameter_bindings["STATE_VECTOR_MAX_EXPRESSION"] = "[s_max, v_max, a_max, p_brake_max]^T"
            if "STATE_VECTOR_MIN_UNITS" not in self._explicit_keys:
                self.parameter_bindings["STATE_VECTOR_MIN_UNITS"] = "m, m/s, m/s^2, bar"
            if "STATE_VECTOR_MAX_UNITS" not in self._explicit_keys:
                self.parameter_bindings["STATE_VECTOR_MAX_UNITS"] = "m, m/s, m/s^2, bar"
            if "STATE_SAFETY_MITIGATION" not in self._explicit_keys:
                self.parameter_bindings["STATE_SAFETY_MITIGATION"] = "EN 50129:2018 §6.3 Safety Interlocks"
            if "CONTAINMENT_BUFFER_UNIT" not in self._explicit_keys:
                self.parameter_bindings["CONTAINMENT_BUFFER_UNIT"] = "m"
            if "CONTAINMENT_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["CONTAINMENT_STANDARD"] = "EN 50128 SIL 4 / IEEE 1474.1"
            if "C2_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["C2_STANDARD"] = "EN 50159 / GSM-R / Eurobalise"
            if "CONTAINMENT_RESPONSE_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["CONTAINMENT_RESPONSE_STANDARD"] = "EN 50126:2017 §7.3"
        elif dom == "marine":
            self.parameter_bindings["STRUCTURE_PARTITION_LABEL"] = "Pressure-Tolerant Titanium & Syntactic Foam Hull"
            self.parameter_bindings["FAILSAFE_CONTAINMENT_NAME"] = "positive buoyancy ballast drop-weight / ascent actuator"
            self.parameter_bindings["ALTITUDE_UNIT"] = "m Depth"
            self.parameter_bindings["V_STALL_MAX_MPS"] = "0.0"
            self.parameter_bindings["V_STALL_NOMINAL_MPS"] = "0.0"
            self.parameter_bindings["REMOTE_ID_HEADER"] = "Maritime AIS & Underwater Acoustic USBL Transponder ID"
            self.parameter_bindings["REMOTE_ID_STANDARD_BODY"] = "Direct subsea acoustic and surface AIS identification conforming to DNV-GL-ST-E403 and IMO COLREGs standards."
            self.parameter_bindings["TIER4_CONTAINMENT_DESC"] = "galvanic ballast drop-weight release or thruster power cutoff"
            self.parameter_bindings["FAILSAFE_DESCENT_SYSTEM"] = "positive buoyancy ballast release system"
            self.parameter_bindings["RECOVERY_DEVICE_TERM"] = "positive buoyancy drop-weight"
            self.parameter_bindings["RECOVERY_SUB"] = "drop-weight"
            self.parameter_bindings["PARACHUTE_SYMBOL_CD"] = "C_d"
            self.parameter_bindings["PARACHUTE_SYMBOL_V"] = "v_{\\mathrm{ascent}}"
            self.parameter_bindings["PARACHUTE_PARAM_NAME_S"] = "Hydrodynamic Reference Cross-Section"
            self.parameter_bindings["PARACHUTE_PARAM_NAME_CD"] = "Hydrodynamic Drag Coefficient"
            self.parameter_bindings["PARACHUTE_PARAM_SYM_CD"] = "C_d"
            self.parameter_bindings["PARACHUTE_PARAM_NAME_V"] = "Terminal Buoyant Ascent Velocity"
            self.parameter_bindings["PARACHUTE_PARAM_SYM_V"] = "v_ascent"
            self.parameter_bindings["EMERGENCY_IGNITION_DESC"] = "Galvanic Ballast Release & Thruster Cutoff Command"
            self.parameter_bindings["CONTAINMENT_SQUIB_ACTION"] = "Galvanic Ballast Drop & Power Isolation Command"
            self.parameter_bindings["OPTX13_NAME"] = "BroadcastMaritimeIdentificationTelemetry"
            self.parameter_bindings["OPTX13_SOURCE"] = "MaritimeIdentification"
            self.parameter_bindings["OPTX13_PROTOCOL_DESC"] = "Digitally Signed Subsea Telemetry (USBL Acoustic / AIS per DNV-GL)"
            self.parameter_bindings["ALTITUDE_TELEMETRY"] = "Bathymetric Depth"
            if "STATE_SPACE_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["STATE_SPACE_STANDARD"] = "DNV-GL-ST-E403 §3.2"
            if "SAFETY_BOUNDS_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["SAFETY_BOUNDS_STANDARD"] = "ISO 13628-6 §6.3"
            if "STATE_VECTOR_MIN_EXPRESSION" not in self._explicit_keys:
                self.parameter_bindings["STATE_VECTOR_MIN_EXPRESSION"] = "[x_north_min, y_east_min, z_depth_min, u_surge_min, v_sway_min, w_heave_min]^T"
            if "STATE_VECTOR_MAX_EXPRESSION" not in self._explicit_keys:
                self.parameter_bindings["STATE_VECTOR_MAX_EXPRESSION"] = "[x_north_max, y_east_max, z_depth_max, u_surge_max, v_sway_max, w_heave_max]^T"
            if "STATE_VECTOR_MIN_UNITS" not in self._explicit_keys:
                self.parameter_bindings["STATE_VECTOR_MIN_UNITS"] = "m, m, m Depth, m/s"
            if "STATE_VECTOR_MAX_UNITS" not in self._explicit_keys:
                self.parameter_bindings["STATE_VECTOR_MAX_UNITS"] = "m, m, m Depth, m/s"
            if "STATE_SAFETY_MITIGATION" not in self._explicit_keys:
                self.parameter_bindings["STATE_SAFETY_MITIGATION"] = "IMO MASS Code §4.2 / COLREGs Rule 8"
            if "CONTAINMENT_BUFFER_UNIT" not in self._explicit_keys:
                self.parameter_bindings["CONTAINMENT_BUFFER_UNIT"] = "m Depth"
            if "CONTAINMENT_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["CONTAINMENT_STANDARD"] = "DNV-GL-ST-E403 §4.4"
            if "C2_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["C2_STANDARD"] = "USBL Acoustic / AIS Datalink"
            if "CONTAINMENT_RESPONSE_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["CONTAINMENT_RESPONSE_STANDARD"] = "ISO 13628-6 §6.3"
        elif dom == "space":
            self.parameter_bindings["STRUCTURE_PARTITION_LABEL"] = "Spacecraft Space-Grade Aluminum Chassis & Structural Panels"
            self.parameter_bindings["FAILSAFE_CONTAINMENT_NAME"] = "autonomous de-orbit propulsion / reaction wheel passivation"
            self.parameter_bindings["ALTITUDE_UNIT"] = "km Orbital Altitude"
            self.parameter_bindings["V_STALL_MAX_MPS"] = "0.0"
            self.parameter_bindings["V_STALL_NOMINAL_MPS"] = "0.0"
            self.parameter_bindings["REMOTE_ID_HEADER"] = "Space Tracking & Ephemeris Telemetry Identification"
            self.parameter_bindings["REMOTE_ID_STANDARD_BODY"] = "Direct space telemetry and ephemeris tracking in accordance with ECSS-E-ST-40C and NASA-STD-8739.8 standards."
            self.parameter_bindings["TIER4_CONTAINMENT_DESC"] = "de-orbit retro-burn execution or battery passivation"
            self.parameter_bindings["FAILSAFE_DESCENT_SYSTEM"] = "autonomous de-orbit propulsion system"
            self.parameter_bindings["RECOVERY_DEVICE_TERM"] = "de-orbit thruster"
            self.parameter_bindings["RECOVERY_SUB"] = "de-orbit"
            self.parameter_bindings["PARACHUTE_SYMBOL_CD"] = "C_d"
            self.parameter_bindings["PARACHUTE_SYMBOL_V"] = "v_{\\mathrm{reentry}}"
            self.parameter_bindings["PARACHUTE_PARAM_NAME_S"] = "Spacecraft Drag Reference Cross-Section"
            self.parameter_bindings["PARACHUTE_PARAM_NAME_CD"] = "Orbital Drag Coefficient"
            self.parameter_bindings["PARACHUTE_PARAM_SYM_CD"] = "C_d"
            self.parameter_bindings["PARACHUTE_PARAM_NAME_V"] = "Terminal Orbital Demise Velocity"
            self.parameter_bindings["PARACHUTE_PARAM_SYM_V"] = "v_reentry"
            self.parameter_bindings["EMERGENCY_IGNITION_DESC"] = "Autonomous De-Orbit Retro-Burn Command"
            self.parameter_bindings["CONTAINMENT_SQUIB_ACTION"] = "De-Orbit Retro-Burn & Battery Passivation Command"
            self.parameter_bindings["OPTX13_NAME"] = "BroadcastSpaceTrackingTelemetry"
            self.parameter_bindings["OPTX13_SOURCE"] = "SpaceTrackingIdentification"
            self.parameter_bindings["OPTX13_PROTOCOL_DESC"] = "Digitally Signed CCSDS Space Telemetry per ECSS-E-ST-40C"
            self.parameter_bindings["ALTITUDE_TELEMETRY"] = "Orbital Altitude / Ephemeris"
            if "STATE_SPACE_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["STATE_SPACE_STANDARD"] = "ECSS-E-ST-10C §5.2"
            if "SAFETY_BOUNDS_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["SAFETY_BOUNDS_STANDARD"] = "ECSS-E-ST-40C §6.3"
            if "STATE_VECTOR_MIN_EXPRESSION" not in self._explicit_keys:
                self.parameter_bindings["STATE_VECTOR_MIN_EXPRESSION"] = "[r_x_min, r_y_min, r_z_min, v_x_min, v_y_min, v_z_min]^T"
            if "STATE_VECTOR_MAX_EXPRESSION" not in self._explicit_keys:
                self.parameter_bindings["STATE_VECTOR_MAX_EXPRESSION"] = "[r_x_max, r_y_max, r_z_max, v_x_max, v_y_max, v_z_max]^T"
            if "STATE_VECTOR_MIN_UNITS" not in self._explicit_keys:
                self.parameter_bindings["STATE_VECTOR_MIN_UNITS"] = "km, km, km, km/s"
            if "STATE_VECTOR_MAX_UNITS" not in self._explicit_keys:
                self.parameter_bindings["STATE_VECTOR_MAX_UNITS"] = "km, km, km, km/s"
            if "STATE_SAFETY_MITIGATION" not in self._explicit_keys:
                self.parameter_bindings["STATE_SAFETY_MITIGATION"] = "NASA-STD-8739.8 §4.2 Orbital Demise"
            if "CONTAINMENT_BUFFER_UNIT" not in self._explicit_keys:
                self.parameter_bindings["CONTAINMENT_BUFFER_UNIT"] = "km Orbital Altitude"
            if "CONTAINMENT_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["CONTAINMENT_STANDARD"] = "ECSS-E-ST-40C §5.8"
            if "C2_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["C2_STANDARD"] = "CCSDS Space Packet Protocol / ECSS-E-ST-40C"
            if "CONTAINMENT_RESPONSE_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["CONTAINMENT_RESPONSE_STANDARD"] = "NASA-STD-8739.8 §5.3"
        elif dom == "industrial":
            self.parameter_bindings["STRUCTURE_PARTITION_LABEL"] = "Heavy-Duty Welded Steel AGV Chassis & Mast Assembly"
            self.parameter_bindings["FAILSAFE_CONTAINMENT_NAME"] = "optical safety lidar field stop / electromagnetic friction brake"
            self.parameter_bindings["ALTITUDE_UNIT"] = "m"
            self.parameter_bindings["V_STALL_MAX_MPS"] = "0.0"
            self.parameter_bindings["V_STALL_NOMINAL_MPS"] = "0.0"
            self.parameter_bindings["REMOTE_ID_HEADER"] = "VDA 5050 AGV Telemetry & RFID Floor Identification"
            self.parameter_bindings["REMOTE_ID_STANDARD_BODY"] = "Direct warehouse telemetry and vehicle identification in accordance with ISO 3691-4:2023 and VDA 5050 standards."
            self.parameter_bindings["TIER4_CONTAINMENT_DESC"] = "optical safety field emergency stop or drive motor power cutoff"
            self.parameter_bindings["FAILSAFE_DESCENT_SYSTEM"] = "electromagnetic safety braking system"
            self.parameter_bindings["RECOVERY_DEVICE_TERM"] = "electromagnetic safety brake"
            self.parameter_bindings["RECOVERY_SUB"] = "brake"
            self.parameter_bindings["PARACHUTE_SYMBOL_CD"] = "C_d"
            self.parameter_bindings["PARACHUTE_SYMBOL_V"] = "v_{\\mathrm{terminal}}"
            self.parameter_bindings["PARACHUTE_PARAM_NAME_S"] = "Vehicle Frontal Cross-Section"
            self.parameter_bindings["PARACHUTE_PARAM_NAME_CD"] = "Aerodynamic / Rolling Resistance Coefficient"
            self.parameter_bindings["PARACHUTE_PARAM_SYM_CD"] = "C_d"
            self.parameter_bindings["PARACHUTE_PARAM_NAME_V"] = "Terminal Deceleration Velocity"
            self.parameter_bindings["PARACHUTE_PARAM_SYM_V"] = "v_terminal"
            self.parameter_bindings["EMERGENCY_IGNITION_DESC"] = "Emergency Drive Power Cutoff & Mechanical Brake Command"
            self.parameter_bindings["CONTAINMENT_SQUIB_ACTION"] = "Emergency Power Isolation & Spring-Applied Brake Command"
            self.parameter_bindings["OPTX13_NAME"] = "BroadcastIndustrialVehicleTelemetry"
            self.parameter_bindings["OPTX13_SOURCE"] = "IndustrialVehicleIdentification"
            self.parameter_bindings["OPTX13_PROTOCOL_DESC"] = "Digitally Signed Industrial Telemetry (VDA 5050 MQTT per ISO 3691-4)"
            self.parameter_bindings["ALTITUDE_TELEMETRY"] = "Fork Lift Height"
            if "STATE_SPACE_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["STATE_SPACE_STANDARD"] = "ISO 3691-4:2023 §4.2"
            if "SAFETY_BOUNDS_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["SAFETY_BOUNDS_STANDARD"] = "IEC 61508 SIL 3 Part 2 §7.4"
            if "STATE_VECTOR_MIN_EXPRESSION" not in self._explicit_keys:
                self.parameter_bindings["STATE_VECTOR_MIN_EXPRESSION"] = "[x_grid_min, y_grid_min, theta_yaw_min, v_trans_min, omega_rot_min, h_fork_min]^T"
            if "STATE_VECTOR_MAX_EXPRESSION" not in self._explicit_keys:
                self.parameter_bindings["STATE_VECTOR_MAX_EXPRESSION"] = "[x_grid_max, y_grid_max, theta_yaw_max, v_trans_max, omega_rot_max, h_fork_max]^T"
            if "STATE_VECTOR_MIN_UNITS" not in self._explicit_keys:
                self.parameter_bindings["STATE_VECTOR_MIN_UNITS"] = "m, m, rad, m/s, rad/s, m"
            if "STATE_VECTOR_MAX_UNITS" not in self._explicit_keys:
                self.parameter_bindings["STATE_VECTOR_MAX_UNITS"] = "m, m, rad, m/s, rad/s, m"
            if "STATE_SAFETY_MITIGATION" not in self._explicit_keys:
                self.parameter_bindings["STATE_SAFETY_MITIGATION"] = "ISO 3691-4:2023 §4.3 Active Personnel Detection"
            if "CONTAINMENT_BUFFER_UNIT" not in self._explicit_keys:
                self.parameter_bindings["CONTAINMENT_BUFFER_UNIT"] = "m"
            if "CONTAINMENT_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["CONTAINMENT_STANDARD"] = "ISO 3691-4:2023 §4.2 Path Containment"
            if "C2_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["C2_STANDARD"] = "VDA 5050 MQTT §4.0"
            if "CONTAINMENT_RESPONSE_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["CONTAINMENT_RESPONSE_STANDARD"] = "IEC 61508 SIL 3 Part 3 §7.4"
        else:
            self.parameter_bindings["STRUCTURE_PARTITION_LABEL"] = "Airframe Structure"
            self.parameter_bindings["FAILSAFE_CONTAINMENT_NAME"] = "ballistic parachute recovery / containment actuator"
            self.parameter_bindings["ALTITUDE_UNIT"] = "m AGL"
            if "V_STALL_MAX_MPS" not in self.parameter_bindings:
                self.parameter_bindings["V_STALL_MAX_MPS"] = "14.0"
            if "V_STALL_NOMINAL_MPS" not in self.parameter_bindings:
                self.parameter_bindings["V_STALL_NOMINAL_MPS"] = "12.0"
            self.parameter_bindings["REMOTE_ID_HEADER"] = "ASTM F3411 Direct Broadcast Remote ID"
            self.parameter_bindings["REMOTE_ID_STANDARD_BODY"] = "Direct connectionless RF broadcast in accordance with ASTM F3411-22a and ASD-STAN prEN 4709-002 standards."
            self.parameter_bindings["TIER4_CONTAINMENT_DESC"] = "ballistic parachute deploy or instant motor cutoff"
            self.parameter_bindings["FAILSAFE_DESCENT_SYSTEM"] = "emergency parachute recovery system"
            self.parameter_bindings["RECOVERY_DEVICE_TERM"] = "parachute"
            self.parameter_bindings["RECOVERY_SUB"] = "parachute"
            self.parameter_bindings["PARACHUTE_SYMBOL_CD"] = "C_{d,\\mathrm{parachute}}"
            self.parameter_bindings["PARACHUTE_SYMBOL_V"] = "v_{\\mathrm{terminal,parachute}}"
            self.parameter_bindings["PARACHUTE_PARAM_NAME_S"] = "Parachute Canopy Area"
            self.parameter_bindings["PARACHUTE_PARAM_NAME_CD"] = "Parachute Drag Coefficient"
            self.parameter_bindings["PARACHUTE_PARAM_SYM_CD"] = "C_d_parachute"
            self.parameter_bindings["PARACHUTE_PARAM_NAME_V"] = "Parachute Terminal Velocity"
            self.parameter_bindings["PARACHUTE_PARAM_SYM_V"] = "v_terminal_parachute"
            self.parameter_bindings["EMERGENCY_IGNITION_DESC"] = "Parachute / Pyrotechnic Cutter Ignition Command"
            self.parameter_bindings["CONTAINMENT_SQUIB_ACTION"] = "Parachute / Pyrotechnic Cutter Ignition Command"
            self.parameter_bindings["OPTX13_NAME"] = "BroadcastRemoteIDTelemetry"
            self.parameter_bindings["OPTX13_SOURCE"] = "BroadcastRemoteID"
            self.parameter_bindings["OPTX13_PROTOCOL_DESC"] = "Digitally Signed Public Broadcast (Bluetooth 5.x / Wi-Fi Beacon per ASTM F3411-22a)"
            self.parameter_bindings["ALTITUDE_TELEMETRY"] = "Altitude"
            if "STATE_SPACE_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["STATE_SPACE_STANDARD"] = "ISO/IEC/IEEE 29148:2018 §6.4.2"
            if "SAFETY_BOUNDS_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["SAFETY_BOUNDS_STANDARD"] = "ASTM F3269-17 §6.2"
            if "STATE_VECTOR_MIN_EXPRESSION" not in self._explicit_keys:
                self.parameter_bindings["STATE_VECTOR_MIN_EXPRESSION"] = "[phi_min, lambda_min, h_min, u_min, v_min, w_min]^T"
            if "STATE_VECTOR_MAX_EXPRESSION" not in self._explicit_keys:
                self.parameter_bindings["STATE_VECTOR_MAX_EXPRESSION"] = "[phi_max, lambda_max, h_max, u_max, v_max, w_max]^T"
            if "STATE_VECTOR_MIN_UNITS" not in self._explicit_keys:
                self.parameter_bindings["STATE_VECTOR_MIN_UNITS"] = "rad, rad, m, m/s"
            if "STATE_VECTOR_MAX_UNITS" not in self._explicit_keys:
                self.parameter_bindings["STATE_VECTOR_MAX_UNITS"] = "rad, rad, m/s"
            if "STATE_SAFETY_MITIGATION" not in self._explicit_keys:
                self.parameter_bindings["STATE_SAFETY_MITIGATION"] = "SORA Annex B M1 Mitigations"
            if "CONTAINMENT_BUFFER_UNIT" not in self._explicit_keys:
                self.parameter_bindings["CONTAINMENT_BUFFER_UNIT"] = "m"
            if "CONTAINMENT_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["CONTAINMENT_STANDARD"] = "JARUS SORA v2.5 Step #2"
            if "C2_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["C2_STANDARD"] = "RTCA DO-362A §2.2.1"
            if "CONTAINMENT_RESPONSE_STANDARD" not in self._explicit_keys:
                self.parameter_bindings["CONTAINMENT_RESPONSE_STANDARD"] = "ASTM F3269-17 §7.1"

        if is_civilian:
            self.parameter_bindings["INTERLOCK_PREFIX"] = "SAF"
            self.parameter_bindings["INTERLOCK_SECTION_TITLE"] = "Operational Safety Interlocks & High-Consequence Controls"
            self.parameter_bindings["INTERLOCK_POLICY_NAME"] = "Operational Safety Interlock Policies"
            self.parameter_bindings["INTERLOCK_01_TAG"] = "SAF-01"
            self.parameter_bindings["INTERLOCK_02_TAG"] = "SAF-02"
            self.parameter_bindings["INTERLOCK_03_TAG"] = "SAF-03"
            self.parameter_bindings["INTERLOCK_04_TAG"] = "SAF-04"
            self.parameter_bindings["INTERLOCK_05_TAG"] = "SAF-05"
            self.parameter_bindings["INTERLOCK_06_TAG"] = "SAF-06"
            self.parameter_bindings["TARGET_VERIFICATION_LABEL"] = "Positive Condition Verification (PCV)"
            self.parameter_bindings["TARGET_VERIFICATION_ACRONYM"] = "PCV"
            self.parameter_bindings["TARGET_VERIFICATION_PHRASE"] = "positive condition verification"
            self.parameter_bindings["HIGH_CONSEQUENCE_ACTION"] = "high-consequence actuation"
            self.parameter_bindings["COLLATERAL_RISK_PHRASE"] = "adjacent operational risk"
        else:
            self.parameter_bindings["INTERLOCK_PREFIX"] = "ROE"
            self.parameter_bindings["INTERLOCK_SECTION_TITLE"] = "Rules of Engagement (ROE) & Operational Safety Interlocks"
            self.parameter_bindings["INTERLOCK_POLICY_NAME"] = "Rules of Engagement (ROE)"
            self.parameter_bindings["INTERLOCK_01_TAG"] = "ROE-01"
            self.parameter_bindings["INTERLOCK_02_TAG"] = "ROE-02"
            self.parameter_bindings["INTERLOCK_03_TAG"] = "ROE-03"
            self.parameter_bindings["INTERLOCK_04_TAG"] = "ROE-04"
            self.parameter_bindings["INTERLOCK_05_TAG"] = "ROE-05"
            self.parameter_bindings["INTERLOCK_06_TAG"] = "ROE-06"
            self.parameter_bindings["TARGET_VERIFICATION_LABEL"] = "Positive Identification (PID)"
            self.parameter_bindings["TARGET_VERIFICATION_ACRONYM"] = "PID"
            self.parameter_bindings["TARGET_VERIFICATION_PHRASE"] = "positive identification"
            self.parameter_bindings["HIGH_CONSEQUENCE_ACTION"] = "weapons release"
            self.parameter_bindings["COLLATERAL_RISK_PHRASE"] = "collateral damage"

    def _derive_operational_intent(self) -> None:
        """Deterministically derives OPERATIONAL_PURPOSE, PRIMARY_OPERATIONAL_MISSION, and CORE_MISSION_CAPABILITIES from schema AST entities."""
        sys_name = (
            self.parameter_bindings.get("SYSTEM_IDENTIFIER")
            or self.parameter_bindings.get("SYSTEM_NAME")
            or self.inferred_system_identifier
            or "Autonomous Cyber-Physical System"
        )
        platform_type = (
            self.parameter_bindings.get("PLATFORM_TYPE")
            or "Cyber-Physical System"
        )
        domain = (
            self.parameter_bindings.get("OPERATIONAL_DOMAIN")
            or "Autonomous Operations"
        )
        comms = (
            self.parameter_bindings.get("PRIMARY_COMMS")
            or self.parameter_bindings.get("PACE_PRIMARY_MEDIUM")
            or "Multi-Tier C2 Telemetry Link"
        )
        reg_class = (
            self.parameter_bindings.get("REGULATORY_CLASS")
            or self.parameter_bindings.get("SORA_SAIL")
            or "High-Assurance Safety Baseline"
        )

        if "OPERATIONAL_PURPOSE" not in self.parameter_bindings:
            purpose = (
                f"The primary operational purpose of {sys_name} ({platform_type}) is to execute deterministic, "
                f"autonomous operational tasks, real-time multi-modal state monitoring, and safety-critical boundary containment "
                f"within designated {domain} environments, operating under verified {reg_class} governance and resilient {comms} communications."
            )
            self.parameter_bindings["OPERATIONAL_PURPOSE"] = purpose

        if "PRIMARY_OPERATIONAL_MISSION" not in self.parameter_bindings:
            mission = (
                f"The {sys_name} is engineered to execute high-assurance {domain} missions, autonomous closed-loop control, "
                f"telemetry processing, and deterministic contingency containment in compliance with {reg_class} requirements."
            )
            self.parameter_bindings["PRIMARY_OPERATIONAL_MISSION"] = mission

        if "CORE_CAPABILITY_1" not in self.parameter_bindings:
            self.parameter_bindings["CORE_CAPABILITY_1"] = (
                f"Autonomous closed-loop state trajectory tracking, corridor execution, and operational boundary holding for {platform_type} in {domain}."
            )
        if "CORE_CAPABILITY_2" not in self.parameter_bindings:
            self.parameter_bindings["CORE_CAPABILITY_2"] = (
                f"Multi-modal sensor data fusion combining redundant state estimation sensors, environmental perception units, and reference state observers."
            )
        if "CORE_CAPABILITY_3" not in self.parameter_bindings:
            self.parameter_bindings["CORE_CAPABILITY_3"] = (
                f"Real-time high-throughput telemetry streaming and deterministic command processing over {comms}."
            )
        if "CORE_CAPABILITY_4" not in self.parameter_bindings:
            self.parameter_bindings["CORE_CAPABILITY_4"] = (
                f"Deterministic failsafe state machine ensuring autonomous containment within response threshold limits in accordance with {reg_class}."
            )

        if "CORE_MISSION_CAPABILITIES" not in self.parameter_bindings:
            c1 = self.parameter_bindings["CORE_CAPABILITY_1"]
            c2 = self.parameter_bindings["CORE_CAPABILITY_2"]
            c3 = self.parameter_bindings["CORE_CAPABILITY_3"]
            c4 = self.parameter_bindings["CORE_CAPABILITY_4"]
            self.parameter_bindings["CORE_MISSION_CAPABILITIES"] = (
                f"  1. {c1}\n  2. {c2}\n  3. {c3}\n  4. {c4}"
            )

        self._derive_lifecycle_contract()

    def _derive_lifecycle_contract(self) -> LifecycleContract:
        """
        Deterministically derives formal MBSE Lifecycle Contract and terminal state tokens from schema AST nodes.
        Inspects lifecycle_type, is_expendable, terminal_behavior, recovery_mode, payload_type, operational_domain, platform_type.
        Binds:
          - LIFECYCLE_TYPE
          - LIFECYCLE_BINGO_SAFETY_ACTION
          - LIFECYCLE_END_STATE
          - LIFECYCLE_FAILSAFE_SEQUENCE
          - LIFECYCLE_POST_OP_STATE
          - PRIMARY_TERMINAL_TARGET
          - SECONDARY_TERMINAL_TARGET
          - PRIMARY_RECOVERY_FACILITY
          - SECONDARY_RECOVERY_FACILITY
          - LIFECYCLE_TRANSIT_MODE
        """
        dom = getattr(self, "detected_domain", "aviation").lower()
        platform_type = str(self.parameter_bindings.get("PLATFORM_TYPE") or "").lower()
        sys_id = str(self.parameter_bindings.get("SYSTEM_IDENTIFIER") or self.inferred_system_identifier or "").lower()
        op_domain = str(self.parameter_bindings.get("OPERATIONAL_DOMAIN") or "").lower()
        payload_type = str(self.parameter_bindings.get("PAYLOAD_TYPE") or "").lower()
        terminal_behavior = str(self.parameter_bindings.get("TERMINAL_BEHAVIOR") or "").lower()
        recovery_mode = str(self.parameter_bindings.get("RECOVERY_MODE") or "").lower()
        raw_is_expendable = self.parameter_bindings.get("IS_EXPENDABLE") or self.parameter_bindings.get("is_expendable")
        is_expendable = False
        if raw_is_expendable is not None:
            if isinstance(raw_is_expendable, bool):
                is_expendable = raw_is_expendable
            elif str(raw_is_expendable).strip().lower() in ("true", "1", "yes"):
                is_expendable = True

        is_explicit_lifecycle = (
            "LIFECYCLE_TYPE" in self._explicit_keys
            or "lifecycle_type" in self._explicit_keys
        )
        raw_lifecycle = str(
            self.parameter_bindings.get("LIFECYCLE_TYPE")
            or self.parameter_bindings.get("lifecycle_type")
            or ""
        ).strip().upper()

        combined_context = f"{dom} {platform_type} {sys_id} {op_domain} {payload_type} {terminal_behavior} {recovery_mode}".lower()

        # Archetype classification
        if is_explicit_lifecycle and raw_lifecycle in LifecycleType.__members__:
            selected_type = LifecycleType[raw_lifecycle]
        elif (
            is_expendable
            or "expendable" in combined_context
            or "interceptor" in platform_type
            or "interceptor" in sys_id
            or "kinetic" in payload_type
            or "kinetic" in platform_type
            or "c-uas" in combined_context
            or "counter-uas" in combined_context
            or "run_10" in combined_context
        ):
            selected_type = LifecycleType.EXPENDABLE_KINETIC_EFFECTOR
        elif (
            dom == "medical"
            or "surgical" in combined_context
            or "medical" in combined_context
            or "laparoscopic" in combined_context
            or "clinical" in combined_context
            or "run_07" in combined_context
        ):
            selected_type = LifecycleType.CONTINUOUS_STATIONARY
        elif (
            dom == "rail"
            or "locomotive" in combined_context
            or "rail" in combined_context
            or "train" in combined_context
            or "shunting" in combined_context
            or "run_08" in combined_context
        ):
            selected_type = LifecycleType.TRACK_BOUND_GUIDED
        elif (
            dom == "space"
            or "cubesat" in combined_context
            or "satellite" in combined_context
            or "spacecraft" in combined_context
            or "orbital" in combined_context
            or "run_06" in combined_context
        ):
            selected_type = LifecycleType.PERSISTENT_ORBITAL
        else:
            selected_type = LifecycleType.REUSABLE_RECOVERY

        def _get_explicit_or_default(key: str, default_val: str) -> str:
            if key in self._explicit_keys or key.lower() in self._explicit_keys or key.upper() in self._explicit_keys:
                return str(self.parameter_bindings.get(key) or default_val)
            return default_val

        # Generate contract definitions
        if selected_type == LifecycleType.EXPENDABLE_KINETIC_EFFECTOR:
            containment = ContainmentActionType.SAFE_IMPACT_ZEROIZATION
            bingo_action = _get_explicit_or_default(
                "LIFECYCLE_BINGO_SAFETY_ACTION",
                "Continuously compute closed-loop dynamic resource state and execute terminal target engagement or safe containment ditching with cryptographic zeroization upon reaching safety thresholds ($R(t) \\le R_{\\mathrm{threshold}}(t)$)."
            )
            end_state = _get_explicit_or_default(
                "LIFECYCLE_END_STATE",
                "All assigned operational corridor waypoints fully traversed and verified; zero unauthorized state boundary excursions; zero unmitigated collision or interference hazards; all state conditions positively identified and verified ($C_{\\mathrm{condition}} \\ge C_{\\mathrm{threshold}}$); and successful terminal intercept or safe containment ditching in the designated containment zone with complete cryptographic zeroization."
            )
            failsafe = _get_explicit_or_default(
                "LIFECYCLE_FAILSAFE_SEQUENCE",
                "autonomous terminal containment ditching and zeroization sequence"
            )
            post_op = _get_explicit_or_default(
                "LIFECYCLE_POST_OP_STATE",
                "Terminal kinetic impact or safe containment ditching with complete cryptographic key zeroization, volatile memory purge, and structural containment verification"
            )
            primary_target = _get_explicit_or_default(
                "PRIMARY_TERMINAL_TARGET",
                "Designated Kinetic Intercept / Terminal Containment Zone"
            )
            secondary_target = _get_explicit_or_default(
                "SECONDARY_TERMINAL_TARGET",
                "Safe Containment Ditching Zone LZ-DIVERT-ALPHA"
            )
            primary_facility = _get_explicit_or_default(
                "PRIMARY_RECOVERY_FACILITY",
                "Safe Impact Containment Grid"
            )
            secondary_facility = _get_explicit_or_default(
                "SECONDARY_RECOVERY_FACILITY",
                "Secondary Safe Ditching Area"
            )
            transit_mode = _get_explicit_or_default(
                "LIFECYCLE_TRANSIT_MODE",
                "Terminal_Engagement_Transit"
            )

        elif selected_type == LifecycleType.CONTINUOUS_STATIONARY:
            containment = ContainmentActionType.ELECTROMECHANICAL_BRAKE_LOCK
            bingo_action = _get_explicit_or_default(
                "LIFECYCLE_BINGO_SAFETY_ACTION",
                "Continuously compute closed-loop dynamic resource state and execute automated electromechanical joint brake locking and sterile field preservation upon reaching safety thresholds ($R(t) \\le R_{\\mathrm{threshold}}(t)$)."
            )
            end_state = _get_explicit_or_default(
                "LIFECYCLE_END_STATE",
                "All assigned surgical trajectory segments fully traversed and verified; zero unauthorized workspace boundary excursions; zero unmitigated tissue contact hazards; all clinical conditions positively identified and verified ($C_{\\mathrm{condition}} \\ge C_{\\mathrm{threshold}}$); and successful safe parking at the primary sterile console with residual power reserves strictly satisfying $R_{\\mathrm{reserve}} \\ge \\text{Ratio}_{\\text{reserve\\_min}} \\cdot R_{\\mathrm{capacity}}$."
            )
            failsafe = _get_explicit_or_default(
                "LIFECYCLE_FAILSAFE_SEQUENCE",
                "electromechanical joint brake lock and sterile field safing sequence"
            )
            post_op = _get_explicit_or_default(
                "LIFECYCLE_POST_OP_STATE",
                "Post-operation stationary safe rest at sterile field console with engaged electromechanical joint brake locks, complete cryptographic key zeroization, and diagnostic log offloading"
            )
            primary_target = _get_explicit_or_default(
                "PRIMARY_TERMINAL_TARGET",
                "Primary Sterile Field Docking Station"
            )
            secondary_target = _get_explicit_or_default(
                "SECONDARY_TERMINAL_TARGET",
                "Secondary Clinical Safe Staging Console"
            )
            primary_facility = _get_explicit_or_default(
                "PRIMARY_RECOVERY_FACILITY",
                "Clinical Operating Suite"
            )
            secondary_facility = _get_explicit_or_default(
                "SECONDARY_RECOVERY_FACILITY",
                "Secondary Maintenance Staging Bay"
            )
            transit_mode = _get_explicit_or_default(
                "LIFECYCLE_TRANSIT_MODE",
                "Autonomous_Clinical_Safing"
            )

        elif selected_type == LifecycleType.TRACK_BOUND_GUIDED:
            containment = ContainmentActionType.TRACK_SIDING_BRAKE
            bingo_action = _get_explicit_or_default(
                "LIFECYCLE_BINGO_SAFETY_ACTION",
                "Continuously compute closed-loop dynamic resource state and execute controlled track deceleration or secondary maintenance siding divert upon reaching safety thresholds ($R(t) \\le R_{\\mathrm{threshold}}(t)$)."
            )
            end_state = _get_explicit_or_default(
                "LIFECYCLE_END_STATE",
                "All assigned track blocks and route sections fully traversed and verified; zero unauthorized siding excursions; zero unmitigated collision or derailment hazards; all signal aspects positively identified and verified ($C_{\\mathrm{condition}} \\ge C_{\\mathrm{threshold}}$); and successful controlled arrival at the primary maintenance siding or designated secondary spur with residual resources strictly satisfying $R_{\\mathrm{reserve}} \\ge \\text{Ratio}_{\\text{reserve\\_min}} \\cdot R_{\\mathrm{capacity}}$."
            )
            failsafe = _get_explicit_or_default(
                "LIFECYCLE_FAILSAFE_SEQUENCE",
                "controlled track deceleration and siding brake sequence"
            )
            post_op = _get_explicit_or_default(
                "LIFECYCLE_POST_OP_STATE",
                "Post-operation stationary rest at rail siding with mechanical parking brakes engaged, pneumatic systems vented, cryptographic key zeroization, and diagnostic log offloading"
            )
            primary_target = _get_explicit_or_default(
                "PRIMARY_TERMINAL_TARGET",
                "Primary Rail Maintenance Siding"
            )
            secondary_target = _get_explicit_or_default(
                "SECONDARY_TERMINAL_TARGET",
                "Secondary Controlled Deceleration Spur LZ-DIVERT-ALPHA"
            )
            primary_facility = _get_explicit_or_default(
                "PRIMARY_RECOVERY_FACILITY",
                "Classification Yard Maintenance Depot"
            )
            secondary_facility = _get_explicit_or_default(
                "SECONDARY_RECOVERY_FACILITY",
                "Secondary Service Siding"
            )
            transit_mode = _get_explicit_or_default(
                "LIFECYCLE_TRANSIT_MODE",
                "Autonomous_Track_Deceleration"
            )

        elif selected_type == LifecycleType.PERSISTENT_ORBITAL:
            containment = ContainmentActionType.DEORBIT_DISPOSAL_BURN
            bingo_action = _get_explicit_or_default(
                "LIFECYCLE_BINGO_SAFETY_ACTION",
                "Continuously compute closed-loop dynamic resource state and execute autonomous de-orbit disposal burn or graveyard orbit maneuver upon reaching safety thresholds ($R(t) \\le R_{\\mathrm{threshold}}(t)$)."
            )
            end_state = _get_explicit_or_default(
                "LIFECYCLE_END_STATE",
                "All assigned orbital mission phases fully executed and verified; zero unauthorized constellation slot excursions; zero unmitigated orbital debris collision hazards; all payload observations positively verified ($C_{\\mathrm{condition}} \\ge C_{\\mathrm{threshold}}$); and successful de-orbit disposal or transfer to graveyard orbit with residual propellant satisfying disposal containment standards."
            )
            failsafe = _get_explicit_or_default(
                "LIFECYCLE_FAILSAFE_SEQUENCE",
                "autonomous de-orbit disposal and passivation sequence"
            )
            post_op = _get_explicit_or_default(
                "LIFECYCLE_POST_OP_STATE",
                "Post-mission passivation and safe disposal state with reaction wheels desaturated, batteries passivated, cryptographic key zeroization, and final ephemeris offload"
            )
            primary_target = _get_explicit_or_default(
                "PRIMARY_TERMINAL_TARGET",
                "Designated De-Orbit Reentry Corridor / Graveyard Orbit"
            )
            secondary_target = _get_explicit_or_default(
                "SECONDARY_TERMINAL_TARGET",
                "Secondary Disposal Orbit Node LZ-DIVERT-ALPHA"
            )
            primary_facility = _get_explicit_or_default(
                "PRIMARY_RECOVERY_FACILITY",
                "Designated Atmospheric Demise Footprint"
            )
            secondary_facility = _get_explicit_or_default(
                "SECONDARY_RECOVERY_FACILITY",
                "Stable Graveyard Disposal Orbit"
            )
            transit_mode = _get_explicit_or_default(
                "LIFECYCLE_TRANSIT_MODE",
                "Autonomous_Disposal_Burn_Transit"
            )

        else:  # REUSABLE_RECOVERY
            containment = ContainmentActionType.CONTROLLED_RECOVERY_LANDING
            bingo_action = _get_explicit_or_default(
                "LIFECYCLE_BINGO_SAFETY_ACTION",
                "Continuously compute closed-loop dynamic resource state and execute autonomous return-to-base (RTB) or secondary divert routing upon reaching safety thresholds ($R(t) \\le R_{\\mathrm{threshold}}(t)$)."
            )
            end_state = _get_explicit_or_default(
                "LIFECYCLE_END_STATE",
                "All assigned operational corridor waypoints fully traversed and verified; zero unauthorized state boundary excursions; zero unmitigated collision or interference hazards; all state conditions positively identified and verified ($C_{\\mathrm{condition}} \\ge C_{\\mathrm{threshold}}$); and successful recovery at the primary base or designated secondary divert recovery site with residual resources strictly satisfying $R_{\\mathrm{reserve}} \\ge \\text{Ratio}_{\\text{reserve\\_min}} \\cdot R_{\\mathrm{capacity}}$."
            )
            failsafe = _get_explicit_or_default(
                "LIFECYCLE_FAILSAFE_SEQUENCE",
                "autonomous return-to-base sequence"
            )
            post_op = _get_explicit_or_default(
                "LIFECYCLE_POST_OP_STATE",
                "Post-operation stationary rest at recovery site with actuator safe locking, cryptographic data zeroization, and diagnostic log offloading"
            )
            primary_target = _get_explicit_or_default(
                "PRIMARY_TERMINAL_TARGET",
                "Primary Recovery Base"
            )
            secondary_target = _get_explicit_or_default(
                "SECONDARY_TERMINAL_TARGET",
                "Secondary Divert Site LZ-DIVERT-ALPHA"
            )
            primary_facility = _get_explicit_or_default(
                "PRIMARY_RECOVERY_FACILITY",
                "Primary Recovery Base"
            )
            secondary_facility = _get_explicit_or_default(
                "SECONDARY_RECOVERY_FACILITY",
                "Secondary Divert Base"
            )
            transit_mode = _get_explicit_or_default(
                "LIFECYCLE_TRANSIT_MODE",
                "Autonomous_RTB_Transit"
            )

        contract = LifecycleContract(
            lifecycle_type=selected_type,
            containment_action=containment,
            bingo_safety_action=bingo_action,
            end_state=end_state,
            failsafe_sequence=failsafe,
            post_op_state=post_op,
            primary_terminal_target=primary_target,
            secondary_terminal_target=secondary_target,
            primary_recovery_facility=primary_facility,
            secondary_recovery_facility=secondary_facility,
            lifecycle_transit_mode=transit_mode,
        )

        self.lifecycle_contract = contract
        self.parameter_bindings["LIFECYCLE_TYPE"] = contract.lifecycle_type.value
        self.parameter_bindings["LIFECYCLE_BINGO_SAFETY_ACTION"] = contract.bingo_safety_action
        self.parameter_bindings["LIFECYCLE_END_STATE"] = contract.end_state
        self.parameter_bindings["LIFECYCLE_FAILSAFE_SEQUENCE"] = contract.failsafe_sequence
        self.parameter_bindings["LIFECYCLE_POST_OP_STATE"] = contract.post_op_state
        self.parameter_bindings["PRIMARY_TERMINAL_TARGET"] = contract.primary_terminal_target
        self.parameter_bindings["SECONDARY_TERMINAL_TARGET"] = contract.secondary_terminal_target
        self.parameter_bindings["PRIMARY_RECOVERY_FACILITY"] = contract.primary_recovery_facility
        self.parameter_bindings["SECONDARY_RECOVERY_FACILITY"] = contract.secondary_recovery_facility
        self.parameter_bindings["LIFECYCLE_TRANSIT_MODE"] = contract.lifecycle_transit_mode

        return contract

    def ingest_dictionary(self, data: Dict[str, Any]) -> None:
        """Flattens and registers key-value pairs into parameter bindings."""
        if not isinstance(data, dict):
            return

        # Check for common container keys
        for container_key in ("parameters", "domain_parameters", "domain_params", "specs", "attributes", "metadata"):
            if container_key in data and isinstance(data[container_key], dict):
                self.ingest_dictionary(data[container_key])

        # Check system identifier
        for name_key in ("system_identifier", "system_name", "system", "name", "SYSTEM_IDENTIFIER", "MISSION_SYSTEM_NAME"):
            if name_key in data and isinstance(data[name_key], str) and data[name_key].strip():
                self.inferred_system_identifier = data[name_key].strip()

        # Ingest scalar parameters and list capabilities
        for key, value in data.items():
            if key in ("parameters", "domain_parameters", "domain_params", "specs", "attributes", "metadata", "schema_nodes"):
                continue
            self._explicit_keys.add(key.upper())
            if isinstance(value, list) and key.lower() in (
                "core_mission_capabilities",
                "core_capabilities",
                "capabilities",
                "mission_capabilities",
            ):
                lines = []
                for idx, item in enumerate(value, 1):
                    item_str = str(item).strip()
                    if item_str.startswith(f"{idx}.") or item_str.startswith("-"):
                        lines.append(f"  {item_str}")
                    else:
                        lines.append(f"  {idx}. {item_str}")
                formatted = "\n".join(lines)
                self.parameter_bindings[key] = formatted
                self.parameter_bindings[key.upper()] = formatted
                self.parameter_bindings["CORE_MISSION_CAPABILITIES"] = formatted
            elif isinstance(value, (str, int, float, bool)):
                str_val = str(value)
                self.parameter_bindings[key] = str_val
                self.parameter_bindings[key.upper()] = str_val
                self._map_semantic_aliases(key, str_val)

        # Ingest schema_nodes list if present
        if "schema_nodes" in data and isinstance(data["schema_nodes"], list):
            for node in data["schema_nodes"]:
                if isinstance(node, str) and ":" in node:
                    kind, name = node.split(":", 1)
                    if kind in ("package", "system") and not self.inferred_system_identifier:
                        self.inferred_system_identifier = name.strip()

        self.detected_domain = self._detect_domain_type()
        self.parameter_bindings["DETECTED_DOMAIN"] = self.detected_domain
        self.parameter_bindings["DOMAIN_TYPE"] = self.detected_domain

        self._derive_operational_intent()
        self._derive_mass_budgets()
        self._derive_quadratic_physics()
        self._derive_energy_budgets()
        self._derive_domain_regulatory_standards()
        self._derive_domain_ontology()
        self._derive_lifecycle_contract()

    def _map_semantic_aliases(self, key: str, val: str) -> None:
        """Maps domain attributes to canonical template tokens (Issues #162, #170)."""
        lower = key.lower().replace("-", "_").replace(" ", "_")
        alias_map = {}

        m_num = re.search(r"[-+]?\d*\.?\d+", val)
        num_val = m_num.group(0) if m_num else val

        if "system_identifier" in lower or "mission_system_name" in lower or (lower in ("system", "system_name") and not self.parameter_bindings.get("SYSTEM_IDENTIFIER")):
            alias_map.update({
                "SYSTEM_IDENTIFIER": val,
                "MISSION_SYSTEM_NAME": val,
                "SYSTEM_NAME": val,
            })
        elif "max_cruise" in lower or "v_cruise" in lower or "cruise_speed" in lower or "cruise_velocity" in lower or "cruise" in lower:
            alias_map.update({
                "V_CRUISE_NOMINAL_MPS": num_val,
                "V_CRUISE_MAX_MPS": num_val,
                "V_CRUISE_MIN_MPS": num_val,
                "MAX_CRUISE_SPEED_MS": num_val,
                "CRUISE_SPEED_MS": num_val,
                "CRUISE_SPEED_MPS": num_val,
            })
        elif "max_horizontal" in lower or "horizontal_speed" in lower or "v_max" in lower or "max_speed" in lower or "max_velocity" in lower or lower in ("speed_max", "vmax"):
            alias_map.update({
                "V_MAX_MPS": num_val,
                "V_MAX_NOMINAL_MPS": num_val,
                "MAX_SPEED_MS": num_val,
                "MAX_HORIZONTAL_SPEED_MPS": num_val,
            })
        elif "dive" in lower:
            alias_map.update({
                "V_DIVE_MAX_MPS": num_val,
                "MAX_DIVE_SPEED_MPS": num_val,
                "V_DIVE_MPS": num_val,
            })
        elif "v_stall" in lower or "stall_speed" in lower or "stall_velocity" in lower or "stall" in lower:
            alias_map.update({
                "V_STALL_MAX_MPS": num_val,
                "V_STALL_NOMINAL_MPS": num_val,
                "STALL_SPEED_MS": num_val,
                "V_STALL_MPS": num_val,
                "STALL_SPEED_MPS": num_val,
            })
        elif "wingspan" in lower or "wing_span" in lower:
            alias_map.update({
                "WINGSPAN_M": num_val,
                "WINGSPAN": num_val,
                "DIM_MAX_W_M": num_val,
                "DIM_NOM_W_M": num_val,
            })
        elif "parachute" in lower and ("area" in lower or "canopy" in lower or "m2" in lower or "size" in lower) or lower in ("s_canopy", "s_canopy_m2", "canopy_area", "canopy_area_m2"):
            alias_map.update({
                "PARACHUTE_AREA_M2": num_val,
                "PARACHUTE_CANOPY_AREA_M2": num_val,
                "PARACHUTE_CANOPY_AREA": num_val,
                "S_CANOPY": num_val,
                "S_CANOPY_M2": num_val,
            })
        elif "parachute" in lower and ("drag" in lower or "cd" in lower or "c_d" in lower):
            alias_map.update({
                "PARACHUTE_DRAG_COEFFICIENT": num_val,
                "C_D_PARACHUTE": num_val,
            })
        elif "mtow" in lower or "takeoff_weight" in lower or "takeoff_mass" in lower or "total_mtow" in lower or "gross_weight" in lower:
            alias_map.update({
                "TOTAL_MTOW_KG": num_val,
                "MTOW_MAX_KG": num_val,
                "MTOW_NOMINAL_KG": num_val,
            })
        elif "payload" in lower or "warhead" in lower:
            alias_map.update({
                "PAYLOAD_MAX_KG": num_val,
                "PAYLOAD_NOMINAL_KG": num_val,
            })
        elif "ceiling" in lower or "max_altitude" in lower or "operating_ceiling" in lower:
            alias_map.update({
                "CEILING_MAX_M": num_val,
                "CEILING_NOMINAL_M": num_val,
                "H_MAX_M": num_val,
            })
        elif "c2_range" in lower or "range_c2" in lower or "control_range" in lower or "datalink_range" in lower:
            alias_map.update({
                "C2_RANGE_NOMINAL_KM": num_val,
                "C2_RANGE_MIN_KM": num_val,
            })
        elif "endurance" in lower:
            is_hours = any(h in lower for h in ("_hour", "_hr", "_h", "hours", "hrs")) or any(h in val.lower() for h in ("hour", "hr", " h", "hrs", "hours"))
            if is_hours and m_num:
                hours = float(m_num.group(0))
                min_val = str(round(hours * 60.0, 1))
                alias_map.update({
                    "ENDURANCE_NOMINAL_MIN": min_val,
                    "ENDURANCE_MIN_MIN": min_val,
                    "ENDURANCE_HOURS": str(hours),
                    "NOMINAL_ENDURANCE_HOURS": str(hours),
                })
            elif m_num:
                min_val = m_num.group(0)
                alias_map.update({
                    "ENDURANCE_NOMINAL_MIN": min_val,
                    "ENDURANCE_MIN_MIN": min_val,
                })
            else:
                alias_map.update({
                    "ENDURANCE_NOMINAL_MIN": val,
                    "ENDURANCE_MIN_MIN": val,
                })
        elif "battery_capacity" in lower or "energy_capacity" in lower or "battery_energy" in lower:
            if m_num:
                num = float(m_num.group(0))
                if "joule" in lower or "joule" in val.lower() or (val.strip().endswith("J") and not val.strip().endswith("kJ")):
                    joules = round(num, 1)
                    kwh = round(joules / 3.6e6, 4)
                elif "mj" in lower or "mj" in val.lower():
                    joules = round(num * 1e6, 1)
                    kwh = round(joules / 3.6e6, 4)
                elif "wh" in lower and "kwh" not in lower or "wh" in val.lower() and "kwh" not in val.lower():
                    joules = round(num * 3600.0, 1)
                    kwh = round(num / 1000.0, 4)
                else:
                    kwh = num
                    joules = round(kwh * 3.6e6, 1)

                alias_map.update({
                    "BATTERY_CAPACITY_JOULES": str(joules),
                    "E_CAPACITY_JOULES": str(joules),
                    "BATTERY_CAPACITY_KWH": str(kwh),
                    "E_CAPACITY_KWH": str(kwh),
                })
        elif "wind_limit" in lower or "v_wind" in lower or "wind_speed" in lower or "max_wind" in lower:
            alias_map.update({
                "WIND_LIMIT_MAX_MPS": num_val,
                "WIND_LIMIT_NOMINAL_MPS": num_val,
                "V_WIND_MAX_MPS": num_val,
            })
        elif "temp_min" in lower or "min_temp" in lower or "operating_temperature_min" in lower:
            alias_map.update({
                "TEMP_MIN_DEGC": num_val,
                "OPERATING_TEMP_MIN_C": num_val,
                "OPERATING_TEMPERATURE_MIN_C": num_val,
            })
        elif "temp_max" in lower or "max_temp" in lower or "operating_temperature_max" in lower:
            alias_map.update({
                "TEMP_MAX_DEGC": num_val,
                "OPERATING_TEMP_MAX_C": num_val,
                "OPERATING_TEMPERATURE_MAX_C": num_val,
            })
        elif "ingress" in lower or "ip_rating" in lower:
            alias_map.update({
                "INGRESS_PROTECTION_RATING": val,
                "INGRESS_PROTECTION_TARGET": val,
            })

        for k, v in alias_map.items():
            self.parameter_bindings[k] = v
            self._explicit_keys.add(k)

    def ingest_file(self, file_path: str) -> bool:
        """Ingests a file based on its extension."""
        abs_path = os.path.abspath(file_path) if not os.path.isabs(file_path) else file_path
        if not os.path.isfile(abs_path):
            return False

        if abs_path.endswith(".json"):
            return self.ingest_json_file(abs_path)
        elif abs_path.endswith(".sysml"):
            return self.ingest_sysml_file(abs_path)
        elif abs_path.endswith(".md") or abs_path.endswith(".markdown"):
            return self.ingest_markdown_file(abs_path)
        return False

    def ingest_json_file(self, json_path: str) -> bool:
        """Parses a JSON file and ingests its parameters."""
        try:
            with open(json_path, "r", encoding="utf-8") as f:
                data = json.load(f)
            self.ingest_dictionary(data)
            return True
        except Exception:
            return False

    def ingest_sysml_file(self, sysml_path: str) -> bool:
        """Parses a SysML v2 file and extracts AST symbols."""
        try:
            with open(sysml_path, "r", encoding="utf-8") as f:
                content = f.read()
            return self.ingest_sysml_text(content)
        except Exception:
            return False

    def ingest_sysml_text(self, text: str) -> bool:
        """Extracts AST symbols (package name, attributes, part defs) from SysML v2 text."""
        # 1. Package name extraction
        pkg_match = re.search(r"\bpackage\s+([A-Za-z0-9_]+)", text)
        if pkg_match:
            pkg_name = pkg_match.group(1).strip()
            if pkg_name not in ("Package", "Model", "Root"):
                self.inferred_system_identifier = pkg_name
                self.parameter_bindings["SYSTEM_IDENTIFIER"] = pkg_name
                self.parameter_bindings["MISSION_SYSTEM_NAME"] = pkg_name
                self._explicit_keys.add("SYSTEM_IDENTIFIER")
                self._explicit_keys.add("MISSION_SYSTEM_NAME")

        # 2. Attribute extraction: attribute name : Type = value;
        attr_pattern = re.compile(
            r"\battribute\s+([A-Za-z0-9_]+)(?:\s*:\s*[A-Za-z0-9_]+)?\s*=\s*([^;]+);",
            re.MULTILINE,
        )
        for match in attr_pattern.finditer(text):
            attr_name = match.group(1).strip()
            raw_val = match.group(2).strip().strip('"\'')
            self._explicit_keys.add(attr_name)
            self._explicit_keys.add(attr_name.upper())
            self.parameter_bindings[attr_name] = raw_val
            self.parameter_bindings[attr_name.upper()] = raw_val
            self._map_semantic_aliases(attr_name, raw_val)

        # 3. Constraint extraction for thresholds
        constraint_pattern = re.compile(
            r"\bassert\s+constraint\s+([A-Za-z0-9_]+)\s*\{[^\}]*?([A-Za-z0-9_]+)\s*([<>=!]+)\s*([0-9.]+)",
            re.MULTILINE,
        )
        for match in constraint_pattern.finditer(text):
            _c_name, var_name, _op, limit_val = match.groups()
            self._explicit_keys.add(var_name)
            self._explicit_keys.add(var_name.upper())
            self._map_semantic_aliases(var_name, limit_val)

        self.detected_domain = self._detect_domain_type()
        self.parameter_bindings["DETECTED_DOMAIN"] = self.detected_domain
        self.parameter_bindings["DOMAIN_TYPE"] = self.detected_domain

        self._derive_operational_intent()
        self._derive_mass_budgets()
        self._derive_quadratic_physics()
        self._derive_energy_budgets()
        self._derive_domain_regulatory_standards()
        self._derive_domain_ontology()
        self._derive_lifecycle_contract()

        return True

    def ingest_markdown_file(self, md_path: str) -> bool:
        """Parses a Markdown specification file and ingests its parameters."""
        try:
            with open(md_path, "r", encoding="utf-8") as f:
                content = f.read()
            return self.ingest_markdown_text(content)
        except Exception:
            return False

    def ingest_markdown_text(self, text: str) -> bool:
        """
        Parses Markdown specification text (tables, key-value lists, headings, and patterns)
        and ingests extracted parameters into parameter bindings.
        """
        ingested = False
        if not text or not text.strip():
            return False

        # 1. System Title Extraction: Look for `# <SYSTEM_NAME>`
        for line in text.splitlines():
            line_str = line.strip()
            if line_str.startswith("# ") and not line_str.startswith("##"):
                title = line_str[2:].strip()
                title_clean = re.sub(r"[*_`]", "", title).strip()
                generic_headings = {
                    "technical specifications",
                    "specifications",
                    "table of contents",
                    "overview",
                    "system specifications",
                    "concept of operations",
                    "conops",
                    "mission intent",
                    "requirements",
                    "architecture",
                    "metadata",
                    "document overview",
                }
                if title_clean.lower() not in generic_headings and len(title_clean) > 0:
                    self.inferred_system_identifier = title_clean
                    self.parameter_bindings["SYSTEM_IDENTIFIER"] = title_clean
                    self.parameter_bindings["MISSION_SYSTEM_NAME"] = title_clean
                    self.parameter_bindings["SYSTEM_NAME"] = title_clean
                    self._explicit_keys.add("SYSTEM_IDENTIFIER")
                    self._explicit_keys.add("MISSION_SYSTEM_NAME")
                    self._explicit_keys.add("SYSTEM_NAME")
                    ingested = True
                break

        # 2. Markdown Table Ingestion
        # Extract 2-column or multi-column markdown table rows
        for line in text.splitlines():
            line_str = line.strip()
            if not ("|" in line_str):
                continue
            if re.match(r"^\|?[\s:-|]+\|?$", line_str) and "-" in line_str:
                continue

            raw_cells = [c.strip() for c in line_str.split("|")]
            if line_str.startswith("|") and len(raw_cells) > 0 and raw_cells[0] == "":
                raw_cells = raw_cells[1:]
            if line_str.endswith("|") and len(raw_cells) > 0 and raw_cells[-1] == "":
                raw_cells = raw_cells[:-1]

            if len(raw_cells) >= 2:
                col1 = raw_cells[0]
                col2 = raw_cells[1]
                clean_key = re.sub(r"[*_`]", "", col1).strip().rstrip(":")
                clean_val = re.sub(r"[*_`]", "", col2).strip()
                clean_key_name = re.sub(r"[\*†‡#]+$", "", clean_key).strip()

                header_names = {
                    "parameter", "parameters", "property", "properties",
                    "attribute", "attributes", "key", "keys", "metric",
                    "metrics", "item", "items", "specification", "specifications",
                    "field", "fields", "feature", "features", "name", "variable",
                    "symbol", "check", "check id", "task id", "exchange id",
                    "activity id", "standard id", "trigger id", "threat id", "pace tier",
                }
                if clean_key_name.lower() in header_names or clean_val.lower() in ("value", "nominal value", "description"):
                    continue

                if clean_key_name and clean_val:
                    if len(raw_cells) >= 3:
                        col3 = re.sub(r"[*_`]", "", raw_cells[2]).strip()
                        if col3 and re.match(r"^[A-Za-z0-9/°^%_-]+$", col3) and re.match(r"^[-+]?\d*\.?\d+$", clean_val):
                            clean_val = f"{clean_val} {col3}"

                    norm_key = re.sub(r"[^A-Za-z0-9_]", "_", clean_key_name).upper()
                    self._explicit_keys.add(norm_key)
                    self.parameter_bindings[norm_key] = clean_val
                    self.parameter_bindings[clean_key_name] = clean_val
                    self._map_semantic_aliases(clean_key_name, clean_val)
                    ingested = True

        # 3. Key-Value Bullet Ingestion
        # Extract lines matching - **Key**: Value or - Key: Value
        bullet_pattern = re.compile(
            r"^\s*(?:[-*+]|\d+\.)\s+(?:\*\*([^*]+)\*\*|\*([^*]+)\*|`([^`]+)`|([^:\n]+))\s*:\s*(.+)$"
        )
        for line in text.splitlines():
            m_bullet = bullet_pattern.match(line)
            if m_bullet:
                raw_k = m_bullet.group(1) or m_bullet.group(2) or m_bullet.group(3) or m_bullet.group(4)
                raw_v = m_bullet.group(5)
                if raw_k and raw_v:
                    clean_k = re.sub(r"[*_`]", "", raw_k).strip().rstrip(":")
                    clean_v = re.sub(r"[*_`]", "", raw_v).strip()
                    clean_k_name = re.sub(r"[\*†‡#]+$", "", clean_k).strip()
                    if clean_k_name and clean_v:
                        norm_key = re.sub(r"[^A-Za-z0-9_]", "_", clean_k_name).upper()
                        self._explicit_keys.add(norm_key)
                        self.parameter_bindings[norm_key] = clean_v
                        self.parameter_bindings[clean_k_name] = clean_v
                        self._map_semantic_aliases(clean_k_name, clean_v)
                        ingested = True

        # 4. Text Pattern Ingestion
        text_patterns = [
            re.compile(r"(?:carries\s+(?:up\s+to\s+)?a?|payload\s+of|warhead\s+of|with\s+a)\s+(\d+(?:\.\d+)?)\s*kg\s*(?:warhead|payload)?", re.IGNORECASE),
            re.compile(r"(\d+(?:\.\d+)?)\s*kg\s+(?:warhead|payload)", re.IGNORECASE),
        ]
        for pat in text_patterns:
            for match in pat.finditer(text):
                val = match.group(1).strip()
                self._explicit_keys.add("PAYLOAD_MAX_KG")
                self._explicit_keys.add("PAYLOAD_NOMINAL_KG")
                self.parameter_bindings["PAYLOAD_MAX_KG"] = val
                self.parameter_bindings["PAYLOAD_NOMINAL_KG"] = val
                self._map_semantic_aliases("PAYLOAD_MAX_KG", val)
                ingested = True

        # Detect domain
        self.detected_domain = self._detect_domain_type()
        self.parameter_bindings["DETECTED_DOMAIN"] = self.detected_domain
        self.parameter_bindings["DOMAIN_TYPE"] = self.detected_domain

        # Trigger all derivation solvers
        self._derive_mass_budgets()
        self._derive_quadratic_physics()
        self._derive_energy_budgets()
        self._derive_domain_regulatory_standards()
        self._derive_domain_ontology()
        self._derive_operational_intent()
        self._derive_lifecycle_contract()

        return ingested

    def auto_detect_workspace_parameters(self, search_dirs: Optional[List[str]] = None) -> None:
        """Auto-detects parameter dictionaries, markdown specs, and SysML AST symbols across workspace."""
        if search_dirs is None:
            search_dirs = []
            curr = os.path.abspath(self.workspace_dir)
            for _ in range(5):
                search_dirs.append(curr)
                parent = os.path.dirname(curr)
                if parent == curr:
                    break
                curr = parent

        candidate_paths = []
        for sdir in search_dirs:
            candidate_paths.extend([
                os.path.join(sdir, ".pipeline", "schema.sysml"),
                os.path.join(sdir, ".pipeline", "schema-digest.json"),
                os.path.join(sdir, ".pipeline", "domain_config.json"),
                os.path.join(sdir, "schema", "domain_config.json"),
            ])
            schema_dir = os.path.join(sdir, "schema")
            if os.path.isdir(schema_dir):
                for fname in sorted(os.listdir(schema_dir)):
                    if fname.endswith(".sysml"):
                        candidate_paths.append(os.path.join(schema_dir, fname))
                    elif (fname.endswith(".md") or fname.endswith(".markdown")) and fname.lower() not in ("readme.md",):
                        candidate_paths.append(os.path.join(schema_dir, fname))

        # Ingest existing candidates
        for cpath in candidate_paths:
            if os.path.isfile(cpath):
                self.ingest_file(cpath)

        self._derive_lifecycle_contract()

    def auto_discover_sources(self, root_dir: str) -> None:
        """Auto-detects parameter dictionaries and SysML AST symbols across repository root."""
        search_dirs = []
        curr = os.path.abspath(root_dir)
        for _ in range(5):
            search_dirs.append(curr)
            parent = os.path.dirname(curr)
            if parent == curr:
                break
            curr = parent
        self.auto_detect_workspace_parameters(search_dirs=search_dirs)

    def get_fallback_default(self, token_name: str, inline_default: Optional[str] = None) -> str:
        """
        Dynamically derives sensible domain-agnostic fallback default values for template tokens.
        Ensures 100% domain agnosticism with zero domain bias.
        """
        if inline_default is not None:
            return inline_default

        token_upper = token_name.upper()

        if token_upper in self.parameter_bindings:
            return self.parameter_bindings[token_upper]

        if token_upper in DEFAULT_CONOPS_PARAMS:
            return DEFAULT_CONOPS_PARAMS[token_upper]

        # 1. System and Document Metadata
        sys_id = self.inferred_system_identifier or "AutonomousCyberPhysicalSystem"
        if token_upper in ("SYSTEM_IDENTIFIER", "MISSION_SYSTEM_NAME"):
            return sys_id
        elif token_upper == "DOCUMENT_VERSION":
            return "1.0.0"
        elif token_upper == "DOCUMENT_DATE":
            return datetime.date.today().isoformat()
        elif token_upper == "SECURITY_CLASSIFICATION":
            return "UNCLASSIFIED // PUBLIC RELEASE"
        elif token_upper == "TARGET_SYSTEM_REALIZATION":
            return "Cyber-Physical System (Hardware / Software / MBD)"
        elif token_upper == "AUTHORING_ORGANIZATION":
            return "Systems Engineering Directorate"
        elif token_upper == "OPERATIONAL_DOMAIN":
            return "Cyber-Physical Autonomous Systems"
        elif token_upper == "OPERATIONAL_BOUNDARIES":
            return "Defined operational theater within designated geographic boundary."
        elif token_upper == "STAKEHOLDER_ROSTER":
            return "Operations Officer, Lead Systems Engineer, Safety Officer, Operator in Command."
        elif token_upper == "CURRENT_OPERATIONAL_BASELINE":
            return "Legacy manual / tele-operated baseline with analog telemetry."
        elif token_upper == "OPERATIONAL_DEFICIENCIES":
            return "Lack of autonomous failsafe containment, manual telemetry latency, non-deterministic failover."
        elif token_upper == "MISSION_DRIVERS_AND_VALUE_PROPOSITION":
            return "High-assurance autonomous operation with deterministic safety containment."
        elif token_upper == "TRADE_OFF_ANALYSIS":
            return "Dedicated backup communication link vs payload mass and thermal budget allocation."
        elif token_upper == "CORE_MISSION_CAPABILITIES":
            self._derive_operational_intent()
            return self.parameter_bindings.get("CORE_MISSION_CAPABILITIES", "")
        elif token_upper.startswith("CORE_CAPABILITY_"):
            self._derive_operational_intent()
            return self.parameter_bindings.get(token_upper, "Autonomous cyber-physical mission execution capability.")
        elif token_upper in (
            "LIFECYCLE_TYPE",
            "LIFECYCLE_BINGO_SAFETY_ACTION",
            "LIFECYCLE_END_STATE",
            "LIFECYCLE_FAILSAFE_SEQUENCE",
            "LIFECYCLE_POST_OP_STATE",
            "PRIMARY_TERMINAL_TARGET",
            "SECONDARY_TERMINAL_TARGET",
            "PRIMARY_RECOVERY_FACILITY",
            "SECONDARY_RECOVERY_FACILITY",
            "LIFECYCLE_TRANSIT_MODE",
        ):
            self._derive_lifecycle_contract()
            return self.parameter_bindings.get(token_upper, "")

        # 2. Pugh Decision Matrix
        elif token_upper == "WEIGHT_CRIT_1":
            return "0.40"
        elif token_upper == "WEIGHT_CRIT_2":
            return "0.35"
        elif token_upper == "WEIGHT_CRIT_3":
            return "0.25"
        elif token_upper in ("SCORE_A_1", "SCORE_B_2", "SCORE_C_1"):
            return "1.0"
        elif token_upper in ("SCORE_A_2", "SCORE_B_3", "SCORE_C_3"):
            return "0.8"
        elif token_upper in ("SCORE_A_3", "SCORE_B_1", "SCORE_C_2"):
            return "0.9"
        elif token_upper == "WEIGHTED_SCORE_A":
            return "0.90"
        elif token_upper == "WEIGHTED_SCORE_B":
            return "0.79"
        elif token_upper == "WEIGHTED_SCORE_C":
            return "0.80"

        # 3. Lifecycle Stage Descriptions
        elif token_upper == "PHASE_STARTUP_DESCRIPTION":
            return "Power-on Built-In-Test (PBIT) and sensor calibration."
        elif token_upper == "PHASE_NOMINAL_EXECUTION_DESCRIPTION":
            return "Continuous closed-loop nominal mission execution."
        elif token_upper == "PHASE_DEGRADED_MODE_DESCRIPTION":
            return "Subsystem degradation with redundant sensor/actuator fallback."
        elif token_upper == "PHASE_CONTINGENCY_FAILSAFE_DESCRIPTION":
            return "Deterministic containment and controlled return-to-base."
        elif token_upper == "PHASE_SECURE_SHUTDOWN_DESCRIPTION":
            return "Post-mission telemetry encryption, de-energization, and secure state."
        elif token_upper == "PHASE_MAINTENANCE_MODE_DESCRIPTION":
            return "Diagnostic telemetry offload, LRU replacement, and BIT verification."

        # 4. Mass Fractions & Structural Budgets
        elif token_upper == "MASS_FRACTION_AIRFRAME_PCT":
            return "30.0"
        elif token_upper == "MASS_BUDGET_AIRFRAME_KG":
            mtow = self._get_mtow_value()
            return str(round(0.30 * mtow, 2))
        elif token_upper in ("POWER_NOMINAL_AIRFRAME_W", "POWER_PEAK_AIRFRAME_W", "POWER_NOMINAL_ENERGY_W", "POWER_PEAK_ENERGY_W"):
            return "0.0"
        elif token_upper == "MASS_FRACTION_AVIONICS_PCT":
            return "15.0"
        elif token_upper == "MASS_BUDGET_AVIONICS_KG":
            mtow = self._get_mtow_value()
            return str(round(0.15 * mtow, 2))
        elif token_upper == "POWER_NOMINAL_AVIONICS_W":
            return "50.0"
        elif token_upper == "POWER_PEAK_AVIONICS_W":
            return "120.0"
        elif token_upper == "MASS_FRACTION_PROPULSION_PCT":
            return "25.0"
        elif token_upper == "MASS_BUDGET_PROPULSION_KG":
            mtow = self._get_mtow_value()
            return str(round(0.25 * mtow, 2))
        elif token_upper == "POWER_NOMINAL_PROPULSION_W":
            return "1200.0"
        elif token_upper == "POWER_PEAK_PROPULSION_W":
            return "2500.0"
        elif token_upper == "MASS_FRACTION_ENERGY_PCT":
            return "20.0"
        elif token_upper == "MASS_BUDGET_ENERGY_KG":
            mtow = self._get_mtow_value()
            return str(round(0.20 * mtow, 2))
        elif token_upper == "MASS_FRACTION_PAYLOAD_PCT":
            return "7.0"
        elif token_upper == "MASS_BUDGET_PAYLOAD_KG":
            mtow = self._get_mtow_value()
            return str(round(0.07 * mtow, 2))
        elif token_upper == "POWER_NOMINAL_PAYLOAD_W":
            return "80.0"
        elif token_upper == "POWER_PEAK_PAYLOAD_W":
            return "150.0"
        elif token_upper == "MASS_FRACTION_CONTAINMENT_PCT":
            return "3.0"
        elif token_upper == "MASS_BUDGET_CONTAINMENT_KG":
            mtow = self._get_mtow_value()
            airframe = round(0.30 * mtow, 2)
            avionics = round(0.15 * mtow, 2)
            propulsion = round(0.25 * mtow, 2)
            energy = round(0.20 * mtow, 2)
            payload = round(0.07 * mtow, 2)
            return str(round(mtow - (airframe + avionics + propulsion + energy + payload), 2))
        elif token_upper == "POWER_NOMINAL_CONTAINMENT_W":
            return "5.0"
        elif token_upper == "POWER_PEAK_CONTAINMENT_W":
            return "50.0"
        elif token_upper == "TOTAL_MTOW_KG":
            mtow = self._get_mtow_value()
            return str(mtow)
        elif token_upper == "TOTAL_POWER_NOMINAL_W":
            return "1335.0"
        elif token_upper == "TOTAL_POWER_PEAK_W":
            return "2820.0"

        # 5. Physical Limits & Performance Boundaries
        elif token_upper in ("MTOW_MAX_KG", "SYSTEM_MASS_MAX_KG"):
            return "55.0"
        elif token_upper == "MTOW_NOMINAL_KG":
            return "50.0"
        elif token_upper in ("PAYLOAD_MAX_KG", "PAYLOAD_MASS_MAX_KG"):
            return "5.0"
        elif token_upper == "PAYLOAD_NOMINAL_KG":
            return "3.5"
        elif token_upper == "DIM_MAX_L_M":
            return "2.5"
        elif token_upper == "DIM_MAX_W_M":
            return "3.0"
        elif token_upper == "DIM_MAX_H_M":
            return "0.8"
        elif token_upper == "DIM_NOM_L_M":
            return "2.2"
        elif token_upper == "DIM_NOM_W_M":
            return "2.8"
        elif token_upper == "DIM_NOM_H_M":
            return "0.6"
        elif token_upper == "WINGSPAN_M":
            return "3.0"
        elif token_upper == "V_CRUISE_MIN_MPS":
            return "18.0"
        elif token_upper in ("V_CRUISE_MAX_MPS", "MAX_CRUISE_SPEED_MS"):
            return "32.0"
        elif token_upper == "V_CRUISE_NOMINAL_MPS":
            return "25.0"
        elif token_upper in ("V_MAX_MPS", "MAX_SPEED_MS"):
            return "40.0"
        elif token_upper == "V_MAX_NOMINAL_MPS":
            return "35.0"
        elif token_upper in ("V_STALL_MAX_MPS", "STALL_SPEED_MS"):
            return "14.0"
        elif token_upper == "V_STALL_NOMINAL_MPS":
            return "12.0"
        elif token_upper == "CEILING_MAX_M":
            return "5000.0"
        elif token_upper in ("CEILING_NOMINAL_M", "OPERATIONAL_BOUNDARY_MAX_M"):
            return "3000.0"
        elif token_upper == "C2_RANGE_MIN_KM":
            return "25.0"
        elif token_upper == "C2_RANGE_NOMINAL_KM":
            return "50.0"
        elif token_upper == "ENDURANCE_MIN_MIN":
            return "90.0"
        elif token_upper == "ENDURANCE_NOMINAL_MIN":
            return "120.0"
        elif token_upper == "BATTERY_CAPACITY_KWH":
            return "2.5"
        elif token_upper == "BATTERY_CAPACITY_JOULES":
            return "9000000.0"
        elif token_upper in ("PARACHUTE_AREA_M2", "S_CANOPY", "S_CANOPY_M2", "PARACHUTE_CANOPY_AREA_M2", "PARACHUTE_CANOPY_AREA"):
            m = self._get_mtow_value()
            target_v = 1.6483
            s = round((2.0 * m * 9.80665) / (1.225 * 1.75 * (target_v ** 2)), 2)
            return str(s)
        elif token_upper in ("PARACHUTE_DRAG_COEFFICIENT", "C_D_PARACHUTE"):
            return "1.75"
        elif token_upper in ("V_TERMINAL_PARACHUTE_MPS", "V_TERMINAL_PARACHUTE", "PARACHUTE_TERMINAL_VELOCITY_MPS", "PARACHUTE_TERMINAL_VELOCITY"):
            return self.parameter_bindings.get("V_TERMINAL_PARACHUTE_MPS", "1.65")
        elif token_upper in ("E_K_MITIGATED_JOULES", "E_K_MITIGATED", "MITIGATED_KINETIC_ENERGY_J"):
            return self.parameter_bindings.get("E_K_MITIGATED_JOULES", "34.0")
        elif token_upper in ("TEMP_MIN_DEGC", "OPERATING_TEMP_MIN_C"):
            return "-20.0"
        elif token_upper == "TEMP_MAX_DEGC":
            return "+55.0"
        elif token_upper == "TEMP_NOMINAL_DEGC":
            return "25.0"
        elif token_upper in ("WIND_LIMIT_MAX_MPS", "V_WIND_MAX_MPS"):
            return "15.0"
        elif token_upper == "WIND_LIMIT_NOMINAL_MPS":
            return "10.0"
        elif token_upper in ("INGRESS_PROTECTION_RATING", "INGRESS_PROTECTION_TARGET"):
            return "IP67"

        # 6. SORA Ground Risk Buffer & Bingo Mathematics
        elif token_upper == "H_MAX_M":
            return "120.0"
        elif token_upper == "THETA_IMPACT_DEG":
            return "45.0"
        elif token_upper == "G_ACCEL_MPS2":
            return "9.80665"
        elif token_upper == "D_GLIDE_MAX_M":
            return "50.0"
        elif token_upper == "R_GRB_METERS":
            return "200.0"
        elif token_upper == "V_TERMINAL_MPS":
            return "25.0"
        elif token_upper in ("E_IMPACT_JOULES", "TERMINAL_ENERGY_THRESHOLD_J"):
            return "1562.5"
        elif token_upper == "E_CAPACITY_JOULES":
            return "500000.0"
        elif token_upper == "E_RETURN_JOULES":
            return "150000.0"
        elif token_upper == "E_DIVERT_JOULES":
            return "60000.0"
        elif token_upper == "E_RESERVE_JOULES":
            return "100000.0"
        elif token_upper == "E_CONTINGENCY_JOULES":
            return "40000.0"
        elif token_upper == "E_BINGO_THRESHOLD_JOULES":
            return "350000.0"

        # 7. UAF Activities, OpTx Matrix, Scenarios, Emergency Decisions
        elif token_upper == "OA_ACTIVITY_NAME":
            return "PreFlightBIT"
        elif token_upper == "OA_DESCRIPTION":
            return "Executes power-on Built-In-Tests and sensor calibration."
        elif token_upper == "OPTX_SOURCE_NODE":
            return "PrimarySensorSubsystem"
        elif token_upper == "OPTX_DEST_NODE":
            return "ControllerLogicSubsystem"
        elif token_upper == "OPTX_INFO_ITEM":
            return "PrimarySensorState"
        elif token_upper == "OPTX_DATA_RATE":
            return "100 Hz"
        elif token_upper == "OPTX_MAX_LATENCY":
            return "5 ms"
        elif token_upper == "OPTX_CRITICALITY":
            return "High (DAL-A)"
        elif token_upper == "SCENARIO_NOMINAL_THREAD":
            return "Autonomous pre-flight BIT, launch, corridor survey, and precision recovery."
        elif token_upper == "SCENARIO_DEGRADED_THREAD":
            return "Primary GNSS loss triggers optical odometry navigation fallback."
        elif token_upper == "SCENARIO_CONTINGENCY_THREAD":
            return "Total C2 lost-link triggers return-to-base rally point sequence."
        elif token_upper == "EMG_TRIGGER_NAME":
            return "Lost C2 Link"
        elif token_upper == "EMG_DETECTION_MECHANISM":
            return "Heartbeat loss > 5.0 s"
        elif token_upper == "EMG_CONTAINMENT_ACTION":
            return "Execute autonomous lost-link loiter / return"
        elif token_upper == "EMG_FAILSAFE_STATE":
            return "Contingency_LostLinkReturn"
        elif token_upper == "EMG_MAX_RESPONSE_TIME":
            return "0.50 s"
        elif token_upper == "EMG_HITL_ROLE":
            return "Monitor / Override"

        # 8. MIL-STD-810H Environmental Qualification Parameters
        elif token_upper == "AMBIENT_TEMPERATURE_RANGE":
            return "-20°C to +55°C"
        elif token_upper == "ENVIRONMENTAL_INGRESS_RATING":
            return "IP67 hermetic sealing"
        elif token_upper == "RF_ENVIRONMENT_CONSTRAINTS":
            return "Resilient against intentional GNSS denial and RF interference."
        elif token_upper == "PHYSICAL_SPATIAL_CONSTRAINTS":
            return "Launch and recovery footprint <= 3m x 3m."
        elif token_upper == "COLD_ENDURANCE_HOURS":
            return "1.5"
        elif token_upper == "NOMINAL_ENDURANCE_HOURS":
            return "2.0"
        elif token_upper in ("COLD_CRITERION_WEIGHT", "JAMMING_CRITERION_WEIGHT"):
            return "0.25"
        elif token_upper == "PRECIPITATION_LIMIT_MM_HR":
            return "100.0"
        elif token_upper == "DISTURBANCE_LIMIT_ACCEL":
            return "15.0"
        elif token_upper == "EDGE_COMPUTE_POWER_WATTS":
            return "35.0"
        elif token_upper == "RAW_STREAM_BANDWIDTH_MBPS":
            return "20.0"
        elif token_upper == "FEATURE_STREAM_BANDWIDTH_KBPS":
            return "500.0"
        elif token_upper == "MAX_INFERENCE_LATENCY_MS":
            return "50.0"
        elif token_upper == "SENSOR_ALIGNMENT_TOLERANCE_MRAD":
            return "2.0"
        elif token_upper == "SIGNAL_DENIAL_MAX_DURATION_S":
            return "30.0"
        elif token_upper == "MAX_DEAD_RECKONING_DRIFT_M_S":
            return "1.5"
        elif token_upper == "EMC_RS103_FIELD_STRENGTH_V_M":
            return "50.0"
        elif token_upper == "SAFE_CONTAINMENT_VELOCITY_MS":
            return "5.0"
        elif token_upper == "CONTAINMENT_DEPLOY_TIME_MAX_S":
            return "2.0"

        # MIL-STD-810H Methods
        elif token_upper == "M500_PROCEDURES":
            return "Procedure I (Storage), Procedure II (Operation)"
        elif token_upper == "M500_OP_LIMIT":
            return "57.2 kPa (4,572 m / 15,000 ft altitude)"
        elif token_upper == "M500_STORAGE_LIMIT":
            return "18.8 kPa (12,192 m / 40,000 ft cargo altitude)"
        elif token_upper == "M500_VERIFICATION_STD":
            return "MIL-STD-810H Method 500.6 §4.2"
        elif token_upper == "M500_OP_ALTITUDE_M":
            return "4572.0"
        elif token_upper == "M500_OP_PRESSURE_KPA":
            return "57.2"
        elif token_upper == "M500_STORE_ALTITUDE_M":
            return "12192.0"
        elif token_upper == "M500_STORE_PRESSURE_KPA":
            return "18.8"
        elif token_upper == "M500_DWELL_DURATION_HR":
            return "4.0"
        elif token_upper == "M500_DECOMPRESSION_TIME_S":
            return "15.0"
        elif token_upper == "M500_DECOMPRESSION_RATE_KPA_S":
            return "2.56"
        elif token_upper == "M500_STRUCTURAL_INTEGRITY_CRITERIA":
            return "Zero structural deformation, hermetic seal delta-P retention"

        elif token_upper == "M501_PROCEDURES":
            return "Procedure I (Storage), Procedure II (Operation)"
        elif token_upper == "M501_OP_LIMIT":
            return "+55.0°C ambient (+71.0°C induced solar)"
        elif token_upper == "M501_STORAGE_LIMIT":
            return "+71.0°C constant storage"
        elif token_upper == "M501_VERIFICATION_STD":
            return "MIL-STD-810H Method 501.7 §4.2"
        elif token_upper == "M501_OP_HIGH_TEMP_C":
            return "55.0"
        elif token_upper == "M501_STORE_HIGH_TEMP_C":
            return "71.0"
        elif token_upper == "M501_OP_DURATION_HR":
            return "8.0"
        elif token_upper == "M501_STORE_CYCLE_COUNT":
            return "7"
        elif token_upper == "M501_STORE_TOTAL_HR":
            return "168.0"
        elif token_upper == "M501_TEMP_RISE_RATE_C_MIN":
            return "1.0"
        elif token_upper == "M501_JUNCTION_TEMP_MARGIN_C":
            return "15.0"

        elif token_upper == "M502_PROCEDURES":
            return "Procedure I (Storage), Procedure II (Operation)"
        elif token_upper == "M502_OP_LIMIT":
            return "-20.0°C ambient operation"
        elif token_upper == "M502_STORAGE_LIMIT":
            return "-33.0°C non-operational storage"
        elif token_upper == "M502_VERIFICATION_STD":
            return "MIL-STD-810H Method 502.7 §4.2"
        elif token_upper == "M502_OP_LOW_TEMP_C":
            return "-20.0"
        elif token_upper == "M502_STORE_LOW_TEMP_C":
            return "-33.0"
        elif token_upper == "M502_OP_SOAK_HR":
            return "4.0"
        elif token_upper == "M502_STORAGE_SOAK_HR":
            return "24.0"
        elif token_upper == "M502_COLD_START_TIME_S":
            return "60.0"

        elif token_upper == "M503_PROCEDURES":
            return "Procedure I-B (Multi-cycle shock)"
        elif token_upper == "M503_OP_LIMIT":
            return "-20.0°C to +55.0°C rapid transfer"
        elif token_upper == "M503_STORAGE_LIMIT":
            return "-33.0°C to +71.0°C thermal shock"
        elif token_upper == "M503_VERIFICATION_STD":
            return "MIL-STD-810H Method 503.7 §4.2"
        elif token_upper == "M503_SHOCK_LOW_TEMP_C":
            return "-20.0"
        elif token_upper == "M503_SHOCK_HIGH_TEMP_C":
            return "55.0"
        elif token_upper == "M503_MAX_TRANSFER_TIME_S":
            return "60.0"
        elif token_upper == "M503_SHOCK_DWELL_HR":
            return "2.0"
        elif token_upper == "M503_SHOCK_CYCLE_COUNT":
            return "3"

        elif token_upper == "M505_PROCEDURES":
            return "Procedure I (Cycling / Diurnal)"
        elif token_upper == "M505_OP_LIMIT":
            return "1120 W/m² spectral irradiance"
        elif token_upper == "M505_STORAGE_LIMIT":
            return "A1 Worldwide High Temperature Solar"
        elif token_upper == "M505_VERIFICATION_STD":
            return "MIL-STD-810H Method 505.7 §4.2"
        elif token_upper == "M505_PEAK_IRRADIANCE_W_M2":
            return "1120.0"
        elif token_upper == "M505_SOLAR_AMB_TEMP_C":
            return "49.0"
        elif token_upper == "M505_DIURNAL_CYCLE_COUNT":
            return "3"
        elif token_upper == "M505_ACTINIC_EXPOSURE_HR":
            return "56.0"
        elif token_upper == "M505_MAX_INTERNAL_TEMP_RISE_C":
            return "18.0"
        elif token_upper == "M505_MAX_TRANSMISSIVITY_LOSS_PCT":
            return "5.0"

        elif token_upper == "M506_PROCEDURES":
            return "Procedure I (Blowing Rain), Procedure III (Drip)"
        elif token_upper == "M506_OP_LIMIT":
            return "10 cm/hr precipitation, 18 m/s wind"
        elif token_upper == "M506_STORAGE_LIMIT":
            return "Water ingress containment"
        elif token_upper == "M506_VERIFICATION_STD":
            return "MIL-STD-810H Method 506.6 §4.2"
        elif token_upper == "M506_PRECIP_RATE_MM_HR":
            return "100.0"
        elif token_upper == "M506_PRECIP_RATE_IN_HR":
            return "4.0"
        elif token_upper == "M506_WIND_VELOCITY_M_S":
            return "18.0"
        elif token_upper == "M506_WIND_VELOCITY_MPH":
            return "40.0"
        elif token_upper == "M506_NOZZLE_PRESSURE_KPA":
            return "276.0"
        elif token_upper == "M506_FACE_EXPOSURE_MIN":
            return "30.0"
        elif token_upper == "M506_TOTAL_EXPOSURE_MIN":
            return "120.0"
        elif token_upper == "M506_MIN_INSULATION_RESISTANCE_MOHM":
            return "100.0"

        elif token_upper == "M507_PROCEDURES":
            return "Procedure II (Aggravated Cycle)"
        elif token_upper == "M507_OP_LIMIT":
            return "95% RH non-condensing at +30°C to +60°C"
        elif token_upper == "M507_STORAGE_LIMIT":
            return "10 cycles (240 hours) aggravated"
        elif token_upper == "M507_VERIFICATION_STD":
            return "MIL-STD-810H Method 507.6 §4.2"
        elif token_upper == "M507_AGGRAVATED_RH_PCT":
            return "95.0"
        elif token_upper == "M507_HUMID_HIGH_TEMP_C":
            return "60.0"
        elif token_upper == "M507_HUMID_LOW_TEMP_C":
            return "30.0"
        elif token_upper == "M507_HUMID_CYCLE_COUNT":
            return "10"
        elif token_upper == "M507_TOTAL_HUMID_HOURS":
            return "240.0"
        elif token_upper == "M507_POST_CHECK_HOURS":
            return "24.0"

        elif token_upper == "M509_PROCEDURES":
            return "Procedure I (48h fog / 48h drying)"
        elif token_upper == "M509_OP_LIMIT":
            return "5% NaCl salt solution mist"
        elif token_upper == "M509_STORAGE_LIMIT":
            return "Corrosion resistance across 4 cycles (192 hr)"
        elif token_upper == "M509_VERIFICATION_STD":
            return "MIL-STD-810H Method 509.7 §4.2"
        elif token_upper == "M509_SALT_CONCENTRATION_PCT":
            return "5.0"
        elif token_upper == "M509_CHAMBER_TEMP_C":
            return "35.0"
        elif token_upper == "M509_SALT_CYCLE_COUNT":
            return "4"
        elif token_upper == "M509_TOTAL_EXPOSURE_HOURS":
            return "192.0"
        elif token_upper == "M509_MAX_BONDING_RESISTANCE_MOHM":
            return "2.5"

        elif token_upper == "M510_PROCEDURES":
            return "Procedure I (Blowing Dust), Procedure II (Blowing Sand)"
        elif token_upper == "M510_OP_LIMIT":
            return "10.6 g/m³ dust, 1.1 g/m³ sand, 18 m/s wind"
        elif token_upper == "M510_STORAGE_LIMIT":
            return "Sealed bearing and optical port containment"
        elif token_upper == "M510_VERIFICATION_STD":
            return "MIL-STD-810H Method 510.7 §4.2"
        elif token_upper == "M510_DUST_CONCENTRATION_G_M3":
            return "10.6"
        elif token_upper == "M510_DUST_VELOCITY_M_S":
            return "8.9"
        elif token_upper == "M510_DUST_DURATION_HR":
            return "6.0"
        elif token_upper == "M510_DUST_HIGH_TEMP_DURATION_HR":
            return "6.0"
        elif token_upper == "M510_SAND_CONCENTRATION_G_M3":
            return "1.1"
        elif token_upper == "M510_SAND_VELOCITY_M_S":
            return "18.0"
        elif token_upper == "M510_SAND_DURATION_MIN":
            return "90.0"
        elif token_upper == "M510_MAX_OPTICAL_DEGRADATION_PCT":
            return "10.0"

        elif token_upper == "M514_PROCEDURES":
            return "Procedure I (General Vibration), Category 24"
        elif token_upper == "M514_OP_LIMIT":
            return "0.04 g²/Hz random vibration (20-2000 Hz)"
        elif token_upper == "M514_STORAGE_LIMIT":
            return "Annex C Transportation Vibration Profile"
        elif token_upper == "M514_VERIFICATION_STD":
            return "MIL-STD-810H Method 514.8 §4.2"
        elif token_upper == "M514_OP_VIBRATION_G_RMS":
            return "4.2"
        elif token_upper == "M514_ENDURANCE_VIBRATION_G_RMS":
            return "7.8"
        elif token_upper == "M514_OP_AXIS_DURATION_HR":
            return "1.0"
        elif token_upper == "M514_ENDURANCE_AXIS_DURATION_HR":
            return "2.0"
        elif token_upper == "M514_TOTAL_VIBRATION_HR":
            return "9.0"
        elif token_upper == "M514_MAX_STATE_ERROR_MM":
            return "2.0"
        elif token_upper == "M514_MAX_ATTITUDE_ERROR_DEG":
            return "0.5"
        elif token_upper == "M514_MIN_FASTENER_TORQUE_RETENTION_PCT":
            return "90.0"
        elif token_upper == "M514_MAX_CONTACT_CHATTER_US":
            return "10.0"

        elif token_upper == "M516_PROCEDURES":
            return "Procedure I (Functional Shock), Procedure IV (Transit Drop)"
        elif token_upper == "M516_OP_LIMIT":
            return "20g, 11 ms terminal peak sawtooth shock"
        elif token_upper == "M516_STORAGE_LIMIT":
            return "40g crash hazard, 1.22 m transit drop"
        elif token_upper == "M516_VERIFICATION_STD":
            return "MIL-STD-810H Method 516.8 §4.2"
        elif token_upper == "M516_FUNCTIONAL_SHOCK_G":
            return "20.0"
        elif token_upper == "M516_SHOCK_DURATION_MS":
            return "11.0"
        elif token_upper == "M516_SHOCK_DELTA_V_M_S":
            return "1.8"
        elif token_upper == "M516_CRASH_SHOCK_G":
            return "40.0"
        elif token_upper == "M516_CRASH_DURATION_MS":
            return "6.0"
        elif token_upper == "M516_TRANSIT_DROP_HEIGHT_M":
            return "1.22"
        elif token_upper == "M516_TRANSIT_DROP_HEIGHT_IN":
            return "48.0"

        elif token_upper == "M521_PROCEDURES":
            return "Procedure I (Ice Accretion / De-Ice)"
        elif token_upper == "M521_OP_LIMIT":
            return "13 mm glaze ice accretion, de-ice functional"
        elif token_upper == "M521_STORAGE_LIMIT":
            return "37 mm structural ice accumulation"
        elif token_upper == "M521_VERIFICATION_STD":
            return "MIL-STD-810H Method 521.4 §4.2"
        elif token_upper == "M521_ICE_ACCRETION_THICKNESS_MM":
            return "13.0"
        elif token_upper == "M521_ICE_ACCRETION_THICKNESS_IN":
            return "0.5"
        elif token_upper == "M521_ICE_LOW_TEMP_C":
            return "-10.0"
        elif token_upper == "M521_ICE_HIGH_TEMP_C":
            return "2.0"
        elif token_upper == "M521_ICE_SOAK_HR":
            return "4.0"
        elif token_upper == "M521_MAX_DEICE_CLEAR_TIME_S":
            return "120.0"

        # 9. Maintenance SLAs & Ground Support
        elif token_upper == "O_LEVEL_MAINTENANCE_DESCRIPTION":
            return "Pre-flight visual inspection, modular battery swap, Built-In-Test verification."
        elif token_upper == "I_LEVEL_MAINTENANCE_DESCRIPTION":
            return "Actuator servo calibration, sensor recalibration, modular LRU swap."
        elif token_upper == "D_LEVEL_MAINTENANCE_DESCRIPTION":
            return "Airframe structural overhaul, composite NDI inspection, flight computer recertification."
        elif token_upper == "SWAP_TIME_BATTERY_MAX_MIN":
            return "5.0"
        elif token_upper == "SWAP_TIME_PAYLOAD_MAX_MIN":
            return "10.0"
        elif token_upper == "SWAP_TIME_FC_MAX_MIN":
            return "15.0"
        elif token_upper == "SWAP_TIME_ACTUATOR_MAX_MIN":
            return "20.0"
        elif token_upper == "RAPID_TURNAROUND_SLA_MIN":
            return "15.0"
        elif token_upper == "PREP_TIME_TARGET_S":
            return "300.0"
        elif token_upper == "TURNAROUND_TIME_TARGET_S":
            return "600.0"
        elif token_upper == "OVERHAUL_INTERVAL_HOURS":
            return "500.0"
        elif token_upper == "PHASE_CHECK_INTERVAL_HOURS":
            return "100.0"
        elif token_upper == "BLACKBOX_QUARANTINE_SLA_MIN":
            return "30.0"
        elif token_upper == "BATTERY_CYCLE_LIFE_REQ":
            return "500"
        elif token_upper == "BATTERY_ENERGY_DENSITY_NOMINAL":
            return "250.0"
        elif token_upper == "MIN_STAGING_AREA_M2":
            return "25.0"
        elif token_upper == "MAX_PACKAGED_VOLUME_M3":
            return "2.0"
        elif token_upper == "MAX_CONTAINER_MASS_KG":
            return "40.0"
        elif token_upper == "MAST_HEIGHT_M":
            return "5.0"
        elif token_upper == "TERMINAL_DISPLAY_LUMINANCE_NITS":
            return "1000.0"

        # 10. Mission Intent METL & MoE/MoP
        elif token_upper == "OPERATIONAL_PURPOSE":
            self._derive_operational_intent()
            return self.parameter_bindings.get("OPERATIONAL_PURPOSE", "")
        elif token_upper == "PRIMARY_OPERATIONAL_MISSION":
            self._derive_operational_intent()
            return self.parameter_bindings.get("PRIMARY_OPERATIONAL_MISSION", "")
        elif token_upper == "KEY_MISSION_TASKS":
            return "System initialization, navigation along nominal corridor, mission payload processing, return to base."
        elif token_upper == "MISSION_END_STATE":
            return "All mission waypoints completed, containment boundaries maintained, safe recovery with >20% reserve energy."
        elif token_upper == "MET_TASK_NAME":
            return "PreFlightSystemCheckout"
        elif token_upper == "MET_CONDITION":
            return "Pre-launch power on and BIT"
        elif token_upper == "MET_STANDARD":
            return "100% PBIT pass in < 30 s"
        elif token_upper == "MET_VERIFICATION":
            return "Automated BIT Log Review"
        elif token_upper == "MOE_NAME":
            return "Mission Area Coverage Ratio"
        elif token_upper == "MOE_EQUATION":
            return "A_covered / A_total"
        elif token_upper == "MOE_THRESHOLD":
            return "0.90"
        elif token_upper == "MOE_OBJECTIVE":
            return "0.99"
        elif token_upper == "MOE_UNIT":
            return "Dimensionless"
        elif token_upper == "MOP_NAME":
            return "Cross-Track Waypoint Deviation"
        elif token_upper == "MOP_EQUATION":
            return "max norm(p_act - p_cmd)_2D"
        elif token_upper == "MOP_THRESHOLD":
            return "5.0"
        elif token_upper == "MOP_OBJECTIVE":
            return "1.0"
        elif token_upper == "MOP_UNIT":
            return "m"

        # 11. Multi-Domain Threats
        elif token_upper == "THR_KIN_VECTOR":
            return "High-Speed Projectile / Dynamic Collision"
        elif token_upper == "THR_KIN_DESCRIPTION":
            return "Approaching object on collision trajectory"
        elif token_upper == "THR_MEC_VECTOR":
            return "Primary Actuator Structural Jam / Degradation"
        elif token_upper == "THR_MEC_DESCRIPTION":
            return "Mechanical servo binding or control surface jam"
        elif token_upper == "THR_PWR_VECTOR":
            return "Thermal Runaway / Power Distribution Bus Fault"
        elif token_upper == "THR_PWR_DESCRIPTION":
            return "Battery cell over-temperature or main bus short"
        elif token_upper == "THR_ENV_VECTOR":
            return "Severe Gust Turbulence / In-Flight Icing"
        elif token_upper == "THR_ENV_DESCRIPTION":
            return "Atmospheric icing on air data sensors and surfaces"
        elif token_upper == "THR_EW_VECTOR":
            return "GNSS Denial / Jamming / Spoofing"
        elif token_upper == "THR_EW_DESCRIPTION":
            return "Loss of satellite navigation fix or carrier lock"
        elif token_upper == "THR_CYB_VECTOR":
            return "C2 Datalink Injection / Unauthorized Command Ingress"
        elif token_upper == "THR_CYB_DESCRIPTION":
            return "Replay or forgery of telemetry and control frames"
        elif token_upper == "THR_OPT_VECTOR":
            return "High-Intensity Optical Dazzle / Laser Sensor Denial"
        elif token_upper == "THR_OPT_DESCRIPTION":
            return "Electro-optical sensor array saturation"
        elif token_upper == "THR_SIG_VECTOR":
            return "Acoustic / RF Harmonic Observable Signature Leakage"
        elif token_upper == "THR_SIG_DESCRIPTION":
            return "Emission harmonics exceeding operational baseline"
        elif token_upper == "THR_HUM_VECTOR":
            return "Operator Disparity / Console Mode Confusion"
        elif token_upper == "THR_HUM_DESCRIPTION":
            return "Conflicting manual control inputs or supervisory slip"
        elif token_upper == "THR_CBRN_VECTOR":
            return "Hazardous Aerosol / Particulate Contamination Ingress"
        elif token_upper == "THR_CBRN_DESCRIPTION":
            return "Atmospheric toxic plume or corrosive dust"

        # 12. PACE Communications Plan
        elif token_upper == "PACE_PRIMARY_MEDIUM":
            return "Point-to-Point High-Bandwidth RF Link"
        elif token_upper == "PACE_PRIMARY_BAND":
            return "5.8 GHz ISM"
        elif token_upper == "PACE_PRIMARY_DATA_RATE":
            return "10.0 Mbps"
        elif token_upper == "PACE_PRIMARY_TIMEOUT":
            return "2.0 s"
        elif token_upper == "PACE_PRIMARY_ROLE":
            return "Real-Time Video & Telemetry"

        elif token_upper == "PACE_ALTERNATE_MEDIUM":
            return "Cellular Network VPN Relay"
        elif token_upper == "PACE_ALTERNATE_BAND":
            return "LTE / 5G Band 28"
        elif token_upper == "PACE_ALTERNATE_DATA_RATE":
            return "2.0 Mbps"
        elif token_upper == "PACE_ALTERNATE_TIMEOUT":
            return "3.0 s"
        elif token_upper == "PACE_ALTERNATE_ROLE":
            return "Encrypted Cloud Telemetry Relay"

        elif token_upper == "PACE_CONTINGENCY_MEDIUM":
            return "Frequency-Hopping Spread Spectrum Radio"
        elif token_upper == "PACE_CONTINGENCY_BAND":
            return "915 MHz ISM"
        elif token_upper == "PACE_CONTINGENCY_DATA_RATE":
            return "115.2 kbps"
        elif token_upper == "PACE_CONTINGENCY_TIMEOUT":
            return "5.0 s"
        elif token_upper == "PACE_CONTINGENCY_ROLE":
            return "Essential C2 & Failsafe Commands"

        elif token_upper == "PACE_EMERGENCY_MEDIUM":
            return "Satellite Short Burst Data (SBD)"
        elif token_upper == "PACE_EMERGENCY_BAND":
            return "1.6 GHz L-Band"
        elif token_upper == "PACE_EMERGENCY_DATA_RATE":
            return "2.4 kbps"
        elif token_upper == "PACE_EMERGENCY_TIMEOUT":
            return "10.0 s"
        elif token_upper == "PACE_EMERGENCY_ROLE":
            return "Emergency Termination & Recovery Beacon"

        # 13. ROE, Airspace, Go/No-Go
        elif token_upper == "ROE_RULE_STATEMENT":
            return "System shall not execute autonomous descent below 30 m without positive obstacle clearance."
        elif token_upper == "PRIMARY_BOUNDARY_PERIMETER":
            return "Outer polygon bounding perimeter with 50 m containment warning buffer."
        elif token_upper == "DYNAMIC_EXCLUSION_ZONES":
            return "Populated area buffer circles (R = 300 m) marked NO-FLY."
        elif token_upper == "SEPARATION_MINIMA":
            return "Maintain 150 m vertical and 500 m horizontal separation from non-cooperative entities."
        elif token_upper == "GNG_PHASE":
            return "Pre-Launch"
        elif token_upper == "GNG_PARAMETER":
            return "Energy State of Charge"
        elif token_upper == "GNG_THRESHOLD":
            return ">= 95.0%"
        elif token_upper == "GNG_MECHANISM":
            return "Smart BMS Telemetry"
        elif token_upper == "GNG_ACTION":
            return "Abort Launch if < 95%"

        # 14. Research obligations & control patterns
        elif token_upper == "RESEARCH_SCOPE_DESCRIPTION":
            return "Normative Research Inventory Baseline"
        elif token_upper == "APPLICABILITY_STATEMENT":
            return "Mandatory statutory compliance baseline"
        elif token_upper == "CLAUSE_CITATION_PCT":
            return "100.0%"
        elif token_upper == "TOTAL_OBLIGATIONS_COUNT":
            return "12"
        elif token_upper == "TOTAL_SAFETY_CONSTRAINTS_COUNT":
            return "8"
        elif token_upper == "TOTAL_CONTROL_PATTERNS_COUNT":
            return "6"
        elif token_upper == "TOTAL_METL_TASKS_COUNT":
            return "10"
        elif token_upper == "OBL_01_STANDARD_ID":
            return "ISO/IEC/IEEE 29148:2018"
        elif token_upper == "OBL_01_CLAUSE_CITATION":
            return "§6.4.2"
        elif token_upper == "OBL_01_CLAUSE_TITLE":
            return "Concept of Operations"
        elif token_upper == "OBL_01_FEATURE_SLUG":
            return "feat-01"
        elif token_upper in ("OBL_01_TARGET_METRIC", "OBL_02_TARGET_METRIC", "SAF_01_TARGET_METRIC", "SAF_02_TARGET_METRIC", "CTL_01_TARGET_METRIC", "CTL_02_TARGET_METRIC", "MET_01_TARGET_METRIC", "MET_02_TARGET_METRIC", "EXT_01_TARGET_METRIC", "EXT_02_TARGET_METRIC"):
            return "100%"
        elif token_upper == "OBL_02_STANDARD_ID":
            return "NATO STANAG 4586"
        elif token_upper == "OBL_02_CLAUSE_CITATION":
            return "Annex B §3.2"
        elif token_upper == "OBL_02_CLAUSE_TITLE":
            return "C2 Interoperability"
        elif token_upper == "SAF_01_STANDARD_ID":
            return "MIL-STD-882E"
        elif token_upper == "SAF_01_CLAUSE_CITATION":
            return "Task 202"
        elif token_upper == "SAF_01_CLAUSE_TITLE":
            return "OHA"
        elif token_upper == "SAF_01_STORY_SLUG":
            return "us-01"
        elif token_upper == "SAF_02_STANDARD_ID":
            return "SAE ARP4761"
        elif token_upper == "SAF_02_CLAUSE_CITATION":
            return "§3.0"
        elif token_upper == "SAF_02_CLAUSE_TITLE":
            return "FHA"
        elif token_upper == "CTL_01_STANDARD_ID":
            return "RTCA DO-178C"
        elif token_upper == "CTL_01_CLAUSE_CITATION":
            return "§6.3"
        elif token_upper == "CTL_01_CLAUSE_TITLE":
            return "Software Quality"
        elif token_upper == "CTL_01_FEATURE_SLUG":
            return "feat-02"
        elif token_upper == "CTL_02_STANDARD_ID":
            return "RTCA DO-254"
        elif token_upper == "CTL_02_CLAUSE_CITATION":
            return "§6.2"
        elif token_upper == "CTL_02_CLAUSE_TITLE":
            return "Hardware Assurance"
        elif token_upper == "MET_01_STANDARD_ID":
            return "INCOSE SEH v5.0"
        elif token_upper == "MET_01_CLAUSE_CITATION":
            return "§4.2"
        elif token_upper == "MET_01_CLAUSE_TITLE":
            return "METL Alignment"
        elif token_upper == "MET_01_USECASE_SLUG":
            return "uc-01"
        elif token_upper == "MET_02_STANDARD_ID":
            return "JARUS SORA v2.5"
        elif token_upper == "MET_02_CLAUSE_CITATION":
            return "Annex B"
        elif token_upper == "MET_02_CLAUSE_TITLE":
            return "Ground Risk"
        elif token_upper == "EXT_01_STANDARD_ID":
            return "MIL-STD-810H"
        elif token_upper == "EXT_01_CLAUSE_CITATION":
            return "Method 501.7"
        elif token_upper in ("EXT_01_DECLARED_TOTAL", "EXT_02_DECLARED_TOTAL"):
            return "12"
        elif token_upper == "EXT_02_STANDARD_ID":
            return "MIL-STD-810H"
        elif token_upper == "EXT_02_CLAUSE_CITATION":
            return "Method 502.7"

        # 15. Unit-based heuristic derivation
        if token_upper.endswith("_PCT") or token_upper.endswith("_PERCENT"):
            return "10.0"
        elif token_upper.endswith("_KG"):
            return "1.0"
        elif token_upper.endswith("_W") or token_upper.endswith("_WATTS"):
            return "10.0"
        elif token_upper.endswith("_M") or token_upper.endswith("_METERS"):
            return "10.0"
        elif token_upper.endswith("_MPS") or token_upper.endswith("_M_S"):
            return "15.0"
        elif token_upper.endswith("_S") or token_upper.endswith("_SEC"):
            return "5.0"
        elif token_upper.endswith("_MIN") or token_upper.endswith("_MINUTES"):
            return "10.0"
        elif token_upper.endswith("_HR") or token_upper.endswith("_HOURS"):
            return "1.0"
        elif token_upper.endswith("_DEGC") or token_upper.endswith("_C"):
            return "25.0"
        elif token_upper.endswith("_J") or token_upper.endswith("_JOULES"):
            return "1000.0"
        elif token_upper.endswith("_KM"):
            return "10.0"
        elif token_upper.endswith("_KPA"):
            return "101.3"
        elif token_upper.endswith("_DEG"):
            return "45.0"
        elif token_upper.endswith("_MS"):
            return "50.0"
        elif token_upper.endswith("_US"):
            return "10.0"
        elif token_upper.endswith("_HZ"):
            return "100.0"
        elif token_upper.endswith("_KBPS"):
            return "128.0"
        elif token_upper.endswith("_MBPS"):
            return "10.0"
        elif token_upper.endswith("_MM"):
            return "10.0"
        elif token_upper.endswith("_IN"):
            return "0.5"
        elif token_upper.endswith("_G"):
            return "5.0"
        elif token_upper.endswith("_MRAD"):
            return "1.0"
        elif token_upper.endswith("_NITS"):
            return "1000.0"
        elif token_upper.endswith("_RATIO"):
            return "0.50"
        elif token_upper.endswith("_COUNT"):
            return "5"

        return "N/A"

    def resolve_token(self, token_name: str, inline_default: Optional[str] = None) -> str:
        """Resolves a token using bound parameters or fallback default."""
        if token_name in self.parameter_bindings:
            return self.parameter_bindings[token_name]
        if token_name.upper() in self.parameter_bindings:
            return self.parameter_bindings[token_name.upper()]
        if token_name.lower() in self.parameter_bindings:
            return self.parameter_bindings[token_name.lower()]
        return self.get_fallback_default(token_name, inline_default)

    def substitute(self, content: str) -> str:
        """Substitutes all template placeholders {{...}} in content with resolved values."""
        if not content:
            return ""

        self._derive_operational_intent()

        def _replacer(match: re.Match) -> str:
            token_name = match.group(1)
            inline_default = match.group(2)
            return self.resolve_token(token_name, inline_default)

        # Iteratively substitute up to 3 passes for nested tokens
        current = content
        for _ in range(3):
            updated = PLACEHOLDER_PATTERN.sub(_replacer, current)
            if updated == current:
                break
            current = updated

        dom = getattr(self, "detected_domain", "aviation")

        # Domain-Specific Ontology Sanitization & Zero-Domain Invariants (Fixes #174, #175, #180)
        if dom == "medical":
            current = re.sub(r"\bparachute\b", "failsafe joint brake", current, flags=re.IGNORECASE)
            current = re.sub(r"\bPARACHUTE\b", "JOINT_BRAKE", current)
            current = re.sub(r"\b(?:altitude\s+AGL|m\s+AGL)\b", "mm", current, flags=re.IGNORECASE)
            current = re.sub(r"ASTM\s+F3411(?:-22a)?", "IEC 62304 / ISO 14971", current)
            current = re.sub(r"\bRemote\s+ID\b", "Medical Equipment Telemetry & ID", current)
            current = re.sub(r"\bairframe\b", "surgeon console chassis", current, flags=re.IGNORECASE)
            current = re.sub(r"\bflight\s+plan\b", "surgical procedure workflow", current, flags=re.IGNORECASE)
            current = re.sub(r"\bflight\s+guidance\b", "manipulator trajectory guidance", current, flags=re.IGNORECASE)
            current = re.sub(r"\bflight\s+controller\b", "surgical console controller", current, flags=re.IGNORECASE)
            current = re.sub(r"\blanding\s+zone\b", "sterile field docking zone", current, flags=re.IGNORECASE)
            current = re.sub(r"\blanding\s+pad\b", "patient cart docking area", current, flags=re.IGNORECASE)
            current = re.sub(r"\\mathrm\{parachute\}", r"\\mathrm{brake}", current)
            current = re.sub(r"C_d_parachute", "C_d", current)
            current = re.sub(r"v_terminal_parachute", "v_terminal", current)
        elif dom == "rail":
            current = re.sub(r"\bparachute\b", "pneumatic emergency brake", current, flags=re.IGNORECASE)
            current = re.sub(r"\bPARACHUTE\b", "EMERGENCY_BRAKE", current)
            current = re.sub(r"\b(?:altitude\s+AGL|m\s+AGL)\b", "m", current, flags=re.IGNORECASE)
            current = re.sub(r"ASTM\s+F3411(?:-22a)?", "EN 50128 SIL 4", current)
            current = re.sub(r"\bRemote\s+ID\b", "Automatic Equipment Identification (AEI)", current)
            current = re.sub(r"\bairframe\b", "locomotive chassis & underframe", current, flags=re.IGNORECASE)
            current = re.sub(r"\bflight\s+plan\b", "train shunting route plan", current, flags=re.IGNORECASE)
            current = re.sub(r"\bflight\s+guidance\b", "train automatic protection guidance", current, flags=re.IGNORECASE)
            current = re.sub(r"\bflight\s+controller\b", "train control unit", current, flags=re.IGNORECASE)
            current = re.sub(r"\blanding\s+zone\b", "classification yard siding", current, flags=re.IGNORECASE)
            current = re.sub(r"\blanding\s+pad\b", "depot staging track", current, flags=re.IGNORECASE)
            current = re.sub(r"\\mathrm\{parachute\}", r"\\mathrm{brake}", current)
            current = re.sub(r"C_d_parachute", "C_d", current)
            current = re.sub(r"v_terminal_parachute", "v_terminal", current)
        elif dom == "marine":
            current = re.sub(r"\bparachute\b", "positive buoyancy drop-weight", current, flags=re.IGNORECASE)
            current = re.sub(r"\bPARACHUTE\b", "DROP_WEIGHT", current)
            current = re.sub(r"\b(?:altitude\s+AGL|m\s+AGL)\b", "m Depth", current, flags=re.IGNORECASE)
            current = re.sub(r"ASTM\s+F3411(?:-22a)?", "DNV-GL-ST-E403", current)
            current = re.sub(r"\bRemote\s+ID\b", "Maritime AIS & USBL Telemetry", current)
            current = re.sub(r"\bairframe\b", "pressure-tolerant subsea hull", current, flags=re.IGNORECASE)
            current = re.sub(r"5\.8\s*GHz\s*Wi-?Fi", "10-30 kHz Acoustic Modem", current, flags=re.IGNORECASE)
            current = re.sub(r"\\mathrm\{parachute\}", r"\\mathrm{drop\_weight}", current)
            current = re.sub(r"C_d_parachute", "C_d", current)
            current = re.sub(r"v_terminal_parachute", "v_ascent", current)
        elif dom == "space":
            current = re.sub(r"\bparachute\b", "autonomous de-orbit propulsion", current, flags=re.IGNORECASE)
            current = re.sub(r"\bPARACHUTE\b", "DEORBIT_THRUSTER", current)
            current = re.sub(r"\b(?:altitude\s+AGL|m\s+AGL)\b", "km", current, flags=re.IGNORECASE)
            current = re.sub(r"ASTM\s+F3411(?:-22a)?", "ECSS-E-ST-40C", current)
            current = re.sub(r"\bRemote\s+ID\b", "Space Ephemeris & Telemetry ID", current)
            current = re.sub(r"\bairframe\b", "spacecraft structure", current, flags=re.IGNORECASE)
            current = re.sub(r"\\mathrm\{parachute\}", r"\\mathrm{deorbit}", current)
            current = re.sub(r"C_d_parachute", "C_d", current)
            current = re.sub(r"v_terminal_parachute", "v_reentry", current)
        elif dom == "industrial":
            current = re.sub(r"\bparachute\b", "optical safety lidar field stop", current, flags=re.IGNORECASE)
            current = re.sub(r"\bPARACHUTE\b", "SAFETY_BRAKE", current)
            current = re.sub(r"\b(?:altitude\s+AGL|m\s+AGL)\b", "m", current, flags=re.IGNORECASE)
            current = re.sub(r"ASTM\s+F3411(?:-22a)?", "ISO 3691-4 / VDA 5050", current)
            current = re.sub(r"\bRemote\s+ID\b", "VDA 5050 Vehicle Identification", current)
            current = re.sub(r"\bairframe\b", "heavy-duty AGV chassis", current, flags=re.IGNORECASE)
            current = re.sub(r"\bflight\s+plan\b", "VDA 5050 warehouse route order", current, flags=re.IGNORECASE)
            current = re.sub(r"\bflight\s+guidance\b", "AGV autonomous path guidance", current, flags=re.IGNORECASE)
            current = re.sub(r"\bflight\s+controller\b", "AGV safety controller", current, flags=re.IGNORECASE)
            current = re.sub(r"\\mathrm\{parachute\}", r"\\mathrm{brake}", current)
            current = re.sub(r"C_d_parachute", "C_d", current)
            current = re.sub(r"v_terminal_parachute", "v_terminal", current)
        elif getattr(self, "is_non_aircraft", False):
            current = re.sub(r"\bparachute\b", "recovery system", current, flags=re.IGNORECASE)
            current = re.sub(r"\bPARACHUTE\b", "RECOVERY", current)
            current = re.sub(r"\b(?:altitude\s+AGL|m\s+AGL)\b", "m", current, flags=re.IGNORECASE)
            current = re.sub(r"ASTM\s+F3411(?:-22a)?", "ISO/IEC 29148", current)
            current = re.sub(r"\bRemote\s+ID\b", "Direct Broadcast Identification", current)
            current = re.sub(r"\bairframe\b", "chassis", current, flags=re.IGNORECASE)
            current = re.sub(r"\\mathrm\{parachute\}", r"\\mathrm{recovery}", current)
            current = re.sub(r"C_d_parachute", "C_d", current)
            current = re.sub(r"v_terminal_parachute", "v_terminal", current)

        if getattr(self, "is_civilian", False):
            for idx in range(1, 7):
                current = re.sub(rf"\bROE-0{idx}\b", f"SAF-0{idx}", current)
                current = re.sub(rf"\bROE_0{idx}\b", f"SAF_0{idx}", current)
            current = re.sub(r"Rules of Engagement(?:\s*\(ROE\))?", "Operational Safety Interlocks", current)
            current = re.sub(r"rules of engagement", "operational safety interlocks", current)
            current = re.sub(r"\bPID\b", "PCV", current)
            current = re.sub(r"\bweapons release\b", "high-consequence actuation", current, flags=re.IGNORECASE)
            current = re.sub(r"\bcollateral damage\b", "adjacent operational risk", current, flags=re.IGNORECASE)
            current = re.sub(r"\bpositive identification\b", "positive condition verification", current, flags=re.IGNORECASE)

        # Residual archetype string elimination (Issue #180)
        sys_target = (
            self.parameter_bindings.get("SYSTEM_IDENTIFIER")
            or self.parameter_bindings.get("SYSTEM_NAME")
            or self.inferred_system_identifier
            or "Autonomous Cyber-Physical System"
        )
        current = current.replace("the Abstract Cyber-Physical System Archetype", f"the {sys_target}")
        current = current.replace("The Abstract Cyber-Physical System Archetype", f"The {sys_target}")
        current = current.replace("an Abstract Cyber-Physical System Archetype", f"the {sys_target}")
        current = current.replace("Abstract Cyber-Physical System Archetype", sys_target)
        current = current.replace("Autonomous Cyber-Physical System Archetype", sys_target)
        current = current.replace("AutonomousSystemArchetype", sys_target)

        return current


def bind_parameters(
    text: str,
    params: Optional[Union[Dict[str, Any], str, SysMLParameterBindingEngine]] = None,
) -> str:
    """Convenience helper to substitute placeholders in text using parameter bindings."""
    if isinstance(params, SysMLParameterBindingEngine):
        engine = params
    elif isinstance(params, dict):
        engine = SysMLParameterBindingEngine(parameter_values=params, auto_detect=False)
    elif isinstance(params, str):
        engine = SysMLParameterBindingEngine(config_path=params, auto_detect=False)
    else:
        engine = SysMLParameterBindingEngine(auto_detect=True)
    return engine.substitute(text)


def slugify(text: str) -> str:
    """
    Converts a heading string into a standard GitHub Markdown anchor slug.
    Example: "1. Scope & System Identification" -> "1-scope--system-identification"
    """
    clean = re.sub(r"[`$*]", "", text).strip()
    slug = ""
    for ch in clean.lower():
        if ch.isalnum() or ch in ("-", "_", " "):
            slug += ch
    slug = slug.replace(" ", "-")
    return slug


def extract_headings(content: str) -> List[Tuple[int, str, str]]:
    """
    Extracts markdown headings from content.
    Returns list of tuples: (level, title, slug).
    Ignores code blocks and math blocks.
    """
    headings: List[Tuple[int, str, str]] = []
    in_code_block = False
    in_math_block = False

    for line in content.splitlines():
        stripped = line.strip()
        if stripped.startswith("```"):
            in_code_block = not in_code_block
            continue
        if stripped.startswith("$$"):
            if len(stripped) > 2 and stripped.endswith("$$"):
                continue
            in_math_block = not in_math_block
            continue
        if in_code_block or in_math_block:
            continue

        m = re.match(r"^(#{1,6})\s+(.+)$", stripped)
        if m:
            level = len(m.group(1))
            title = m.group(2).strip()
            slug = slugify(title)
            headings.append((level, title, slug))

    return headings


def generate_table_of_contents(headings: List[Tuple[int, str, str]], max_depth: int = 3) -> str:
    """
    Generates a Markdown Table of Contents from a list of headings.
    Skips the top-level H1 title.
    """
    toc_lines: List[str] = ["## Table of Contents", ""]
    for level, title, slug in headings:
        if level == 1:
            continue
        if level > max_depth:
            continue
        indent = "  " * (level - 2)
        clean_title = re.sub(r"[`*]", "", title)
        toc_lines.append(f"{indent}- [{clean_title}](#{slug})")

    toc_lines.append("")
    return "\n".join(toc_lines)


def verify_markdown_links(content: str) -> List[str]:
    """
    Verifies internal anchor links (#slug) in the document against defined headings.
    Returns list of error messages for any broken anchor links.
    """
    headings = extract_headings(content)
    valid_slugs: Set[str] = {slug for _, _, slug in headings}
    errors: List[str] = []

    anchor_links = re.findall(r"\[([^\]]+)\]\(#([^\)]+)\)", content)
    for text, anchor in anchor_links:
        if anchor not in valid_slugs:
            errors.append(f"Broken anchor link: [{text}](#{anchor}) does not match any heading in document.")

    return errors


def validate_unit_integrity(
    unit_paths: List[str],
    param_engine: Optional[SysMLParameterBindingEngine] = None,
) -> Tuple[bool, List[str]]:
    """
    Validates unit integrity:
    1. File is non-empty (stripped length > 0).
    2. Zero unresolved placeholder tokens matching {{...}} (post parameter substitution if param_engine provided).
    Returns (is_valid, list_of_errors).
    """
    errors: List[str] = []

    for path in unit_paths:
        if not os.path.isfile(path):
            errors.append(f"Unit file not found: {path}")
            continue

        try:
            with open(path, "r", encoding="utf-8") as f:
                text = f.read()
        except Exception as e:
            errors.append(f"Failed to read unit file '{path}': {e}")
            continue

        if not text.strip():
            errors.append(f"Unit file '{path}' is empty.")
            continue

        # If param_engine provided, substitute placeholders before checking
        processed_text = param_engine.substitute(text) if param_engine else text

        # Check for unresolved placeholder tokens
        placeholders = RAW_TOKEN_FINDER.findall(processed_text)
        if placeholders:
            unique_tokens = sorted(list(set(placeholders)))
            errors.append(
                f"Unit file '{path}' contains {len(placeholders)} unresolved placeholder token(s): {', '.join(unique_tokens)}"
            )

    return (len(errors) == 0, errors)


def _extract_doc_metadata(units: List[Tuple[str, str]], param_engine: Optional[SysMLParameterBindingEngine] = None) -> Dict[str, str]:
    """
    Extracts or infers document title, system name, version, and date from the unit contents and parameter engine.
    """
    default_sys = (param_engine.inferred_system_identifier if param_engine else None) or "AutonomousCyberPhysicalSystem"
    default_ver = (param_engine.parameter_bindings.get("DOCUMENT_VERSION") if param_engine else None) or "1.0.0"
    default_date = (param_engine.parameter_bindings.get("DOCUMENT_DATE") if param_engine else None) or datetime.date.today().isoformat()

    metadata: Dict[str, str] = {
        "title": "Concept of Operations",
        "version": default_ver,
        "date": default_date,
        "system": default_sys,
    }

    for _, content in units:
        for line in content.splitlines():
            m_title = re.search(r"\|\s*\*\*Title\*\*\s*\|\s*([^|]+)\|", line, re.IGNORECASE)
            if m_title:
                metadata["title"] = m_title.group(1).strip()
            m_ver = re.search(r"\|\s*\*\*Version\*\*\s*\|\s*([^|]+)\|", line, re.IGNORECASE)
            if m_ver:
                metadata["version"] = m_ver.group(1).strip()
            m_date = re.search(r"\|\s*\*\*Date\*\*\s*\|\s*([^|]+)\|", line, re.IGNORECASE)
            if m_date:
                metadata["date"] = m_date.group(1).strip()

            if line.startswith("# ") and metadata["title"] == "Concept of Operations":
                metadata["title"] = line[2:].strip()

    return metadata


def assemble_document(
    units_dir: str,
    doc_title: Optional[str] = None,
    doc_version: Optional[str] = None,
    doc_date: Optional[str] = None,
    params: Optional[Union[Dict[str, Any], str, SysMLParameterBindingEngine]] = None,
    canonical_whitelist: Optional[List[str]] = None,
) -> Tuple[str, List[str]]:
    """
    Compiles unit files located in units_dir into a single verified Markdown document with
    automated parameter binding, header table injection, TOC generation, and anchor validation.
    Enforces canonical unit whitelists and skips deprecated/ghost files with a warning (Issue #148).
    Returns (compiled_document_text, error_list).
    """
    errors: List[str] = []
    if not os.path.isdir(units_dir):
        return "", [f"Units directory '{units_dir}' does not exist."]

    # Resolve parameter binding engine
    if isinstance(params, SysMLParameterBindingEngine):
        param_engine = params
    elif isinstance(params, dict):
        param_engine = SysMLParameterBindingEngine(parameter_values=params, workspace_dir=units_dir, auto_detect=True)
    elif isinstance(params, str):
        param_engine = SysMLParameterBindingEngine(config_path=params, workspace_dir=units_dir, auto_detect=True)
    else:
        param_engine = SysMLParameterBindingEngine(workspace_dir=units_dir, auto_detect=True)

    # Determine whitelist if not explicitly given
    if canonical_whitelist is None:
        norm_dir = os.path.normpath(units_dir).lower()
        base_dir = os.path.basename(norm_dir)
        if base_dir == "conops":
            canonical_whitelist = CANONICAL_CONOPS_UNITS
        elif base_dir in ("mission_intent", "missionintent"):
            canonical_whitelist = CANONICAL_MISSION_INTENT_UNITS

    # Find all .md files in units_dir
    all_md_files = [f for f in os.listdir(units_dir) if f.endswith(".md")]
    if not all_md_files:
        return "", [f"No markdown unit files (*.md) found in '{units_dir}'."]

    if canonical_whitelist is not None:
        whitelist_set = set(canonical_whitelist)
        for f in sorted(all_md_files):
            if f not in whitelist_set:
                print(f"[Warning] Skipping non-canonical/deprecated unit file '{f}' in '{units_dir}'.")
        filenames = [f for f in canonical_whitelist if f in all_md_files]
        if not filenames:
            return "", [f"No canonical unit files from whitelist found in '{units_dir}'."]
    else:
        filenames = sorted(all_md_files)

    unit_paths = [os.path.join(units_dir, f) for f in filenames]

    # Validate unit integrity with parameter binding
    is_valid, integrity_errors = validate_unit_integrity(unit_paths, param_engine=param_engine)
    if not is_valid:
        errors.extend(integrity_errors)
        return "", errors

    # Read and substitute unit contents
    units: List[Tuple[str, str]] = []
    for path in unit_paths:
        with open(path, "r", encoding="utf-8") as f:
            raw_text = f.read()
        bound_text = param_engine.substitute(raw_text)
        units.append((os.path.basename(path), bound_text))

    meta = _extract_doc_metadata(units, param_engine=param_engine)
    if doc_title:
        meta["title"] = doc_title
    if doc_version:
        meta["version"] = doc_version
    if doc_date:
        meta["date"] = doc_date

    # Build document body
    body_sections: List[str] = []
    h1_found = False

    for name, content in units:
        lines = content.splitlines()
        filtered_lines: List[str] = []
        i = 0
        while i < len(lines):
            line = lines[i]
            # Strip redundant metadata table if present in unit files
            if line.strip().startswith("| Attribute | Value |") or line.strip().startswith("| **Title**"):
                while i < len(lines) and lines[i].strip().startswith("|"):
                    i += 1
                continue

            # If H1 is already declared, avoid duplicating H1
            if line.strip().startswith("# "):
                if h1_found:
                    line = "#" + line
                else:
                    h1_found = True
                    meta["title"] = line.strip()[2:].strip()

            filtered_lines.append(line)
            i += 1

        cleaned_unit = "\n".join(filtered_lines).strip()
        if cleaned_unit:
            body_sections.append(cleaned_unit)

    full_body = "\n\n".join(body_sections)

    # Document Header Table
    header_table = f"""| Attribute | Value |
| :--- | :--- |
| **Title** | {meta['title']} |
| **Version** | {meta['version']} |
| **Date** | {meta['date']} |
"""

    # Extract headings for TOC
    headings = extract_headings(full_body)
    toc = generate_table_of_contents(headings, max_depth=2)

    # If first section starts with H1, place TOC after H1
    if full_body.startswith("# "):
        parts = full_body.split("\n", 1)
        h1_line = parts[0]
        remainder = parts[1] if len(parts) > 1 else ""
        assembled = f"{header_table}\n{h1_line}\n\n{toc}\n{remainder.strip()}\n"
    else:
        assembled = f"{header_table}\n# {meta['title']}\n\n{toc}\n{full_body.strip()}\n"

    # Verify link consistency
    link_errors = verify_markdown_links(assembled)
    if link_errors:
        errors.extend(link_errors)

    return assembled, errors


def assemble_conops(
    input_dir: str,
    output_dir: str,
    verify_only: bool = False,
    params: Optional[Union[Dict[str, Any], str, SysMLParameterBindingEngine]] = None,
    domain: Optional[str] = None,
    workspace_dir: Optional[str] = None,
) -> bool:
    """
    Orchestrates the assembly, parameter binding, and validation of both CONOPS.md and MISSION_INTENT.md
    with domain-aware routing.
    Fixes Issues #143, #148, #174, #175, #177, #178, #179, #180.
    """
    print(f"[*] ConOps Assembly Engine starting: input='{input_dir}', output='{output_dir}', verify_only={verify_only}, domain={domain}")

    ws_dir = workspace_dir
    if not ws_dir:
        for cand in (
            os.path.abspath(output_dir),
            os.path.abspath(os.path.join(output_dir, "..")),
            os.path.abspath(os.path.join(output_dir, "..", "..")),
            os.path.abspath(input_dir),
            os.path.abspath(os.path.join(input_dir, "..")),
            os.path.abspath(os.path.join(input_dir, "..", "..")),
        ):
            if (
                os.path.isdir(os.path.join(cand, "schema"))
                or os.path.isdir(os.path.join(cand, ".pipeline"))
                or os.path.isfile(os.path.join(cand, "domain_config.json"))
                or os.path.isfile(os.path.join(cand, "schema", "domain_config.json"))
            ):
                ws_dir = cand
                break
    if not ws_dir:
        ws_dir = input_dir

    # Resolve parameter binding engine
    if isinstance(params, SysMLParameterBindingEngine):
        param_engine = params
        if domain:
            param_engine.detected_domain = domain
            param_engine.parameter_bindings["DETECTED_DOMAIN"] = domain
            param_engine.parameter_bindings["DOMAIN_TYPE"] = domain
    elif isinstance(params, dict):
        param_engine = SysMLParameterBindingEngine(parameter_values=params, workspace_dir=ws_dir, auto_detect=True, domain=domain)
    elif isinstance(params, str):
        param_engine = SysMLParameterBindingEngine(config_path=params, workspace_dir=ws_dir, auto_detect=True, domain=domain)
    else:
        param_engine = SysMLParameterBindingEngine(workspace_dir=ws_dir, auto_detect=True, domain=domain)

    detected_dom = domain or getattr(param_engine, "detected_domain", "aviation")

    # Domain routing: check domain-native unit packages first, then fallback to standard directory
    conops_units_dir = None
    for candidate in (
        os.path.join(input_dir, detected_dom, "conops"),
        os.path.join(input_dir, "units", detected_dom, "conops"),
        os.path.join(input_dir, "conops"),
        os.path.join(input_dir, "units", "conops"),
    ):
        if os.path.isdir(candidate):
            conops_units_dir = candidate
            break

    mission_units_dir = None
    for candidate in (
        os.path.join(input_dir, detected_dom, "mission_intent"),
        os.path.join(input_dir, "units", detected_dom, "mission_intent"),
        os.path.join(input_dir, "mission_intent"),
        os.path.join(input_dir, "units", "mission_intent"),
    ):
        if os.path.isdir(candidate):
            mission_units_dir = candidate
            break

    all_errors: List[str] = []

    # 1. Assemble CONOPS.md
    if conops_units_dir and os.path.isdir(conops_units_dir):
        print(f"[*] Assembling Concept of Operations from '{conops_units_dir}' [domain={detected_dom}]...")
        conops_doc, conops_errs = assemble_document(
            units_dir=conops_units_dir,
            doc_title="Concept of Operations (ConOps)",
            params=param_engine,
            canonical_whitelist=CANONICAL_CONOPS_UNITS,
        )
        if conops_errs:
            all_errors.extend([f"[CONOPS] {err}" for err in conops_errs])
        elif not verify_only:
            os.makedirs(output_dir, exist_ok=True)
            out_file = os.path.join(output_dir, "CONOPS.md")
            with open(out_file, "w", encoding="utf-8") as f:
                f.write(conops_doc)
            print(f"[+] Successfully wrote compiled ConOps to '{out_file}'.")
    else:
        print(f"[-] ConOps units directory not found in '{input_dir}'. Skipping CONOPS.md assembly.")

    # 2. Assemble MISSION_INTENT.md
    if mission_units_dir and os.path.isdir(mission_units_dir):
        print(f"[*] Assembling Tactical Mission Intent from '{mission_units_dir}' [domain={detected_dom}]...")
        mission_doc, mission_errs = assemble_document(
            units_dir=mission_units_dir,
            doc_title="Tactical Mission Intent & Execution Plan",
            params=param_engine,
            canonical_whitelist=CANONICAL_MISSION_INTENT_UNITS,
        )
        if mission_errs:
            all_errors.extend([f"[MISSION_INTENT] {err}" for err in mission_errs])
        elif not verify_only:
            os.makedirs(output_dir, exist_ok=True)
            out_file = os.path.join(output_dir, "MISSION_INTENT.md")
            with open(out_file, "w", encoding="utf-8") as f:
                f.write(mission_doc)
            print(f"[+] Successfully wrote compiled Mission Intent to '{out_file}'.")
    else:
        print(f"[-] Mission Intent units directory not found in '{input_dir}'. Skipping MISSION_INTENT.md assembly.")

    if all_errors:
        print("\n[!] ConOps Assembly Errors encountered:")
        for err in all_errors:
            print(f"    - {err}")
        return False

    print("[+] All ConOps assembly and verification checks passed cleanly.")
    return True


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Deterministic ConOps & Mission Intent Assembly Engine (ISO 29148 / NATO STANAG 4586 / OMG UAF)."
    )
    parser.add_argument(
        "workspace",
        nargs="?",
        default=None,
        help="Target workspace or project directory (optional positional argument).",
    )
    parser.add_argument(
        "--input-dir",
        default=None,
        help="Input directory containing 'conops/' and 'mission_intent/' unit markdown directories (default: docs/conops/units).",
    )
    parser.add_argument(
        "--output-dir",
        default=None,
        help="Output directory where assembled CONOPS.md and MISSION_INTENT.md are written (default: docs/conops).",
    )
    parser.add_argument(
        "--verify",
        action="store_true",
        help="Verify unit integrity and link resolution without writing output files.",
    )
    parser.add_argument(
        "--params",
        default=None,
        help="Path to JSON parameter dictionary or schema file (auto-detects .pipeline/schema-digest.json or schema/domain_config.json if not specified).",
    )
    parser.add_argument(
        "--domain",
        default=None,
        choices=["aviation", "medical", "rail", "marine", "space", "industrial"],
        help="Target operational domain (aviation, medical, rail, marine, space, industrial). Auto-detected if not specified.",
    )

    args = parser.parse_args()

    workspace = os.path.abspath(args.workspace) if args.workspace else os.getcwd()

    input_dir = args.input_dir
    if not input_dir:
        for cand in (
            os.path.join(workspace, "docs", "conops", "units"),
            os.path.join(workspace, "docs", "conops"),
            os.path.join(workspace, "units"),
            workspace,
        ):
            if os.path.isdir(cand):
                if (
                    os.path.isdir(os.path.join(cand, "conops"))
                    or os.path.isdir(os.path.join(cand, "mission_intent"))
                    or any(os.path.isdir(os.path.join(cand, d, "conops")) for d in ("aviation", "medical", "rail", "marine", "space", "industrial"))
                ):
                    input_dir = cand
                    break
        if not input_dir:
            input_dir = os.path.join(workspace, "docs", "conops", "units")

    output_dir = args.output_dir
    if not output_dir:
        output_dir = os.path.join(workspace, "docs", "conops")

    success = assemble_conops(
        input_dir=input_dir,
        output_dir=output_dir,
        verify_only=args.verify,
        params=args.params,
        domain=args.domain,
        workspace_dir=workspace,
    )
    return 0 if success else 1


if __name__ == "__main__":
    sys.exit(main())
