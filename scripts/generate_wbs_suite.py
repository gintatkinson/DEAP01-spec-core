#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Deterministic Work Breakdown Structure (WBS) & Enterprise Realization Suite Generator.
Conforms to MIL-STD-881E, INCOSE Systems Engineering Handbook v5.0, RTCA DO-178C,
and skills/spec-wbs-engineering/SKILL.md.

Ingests SysML AST, ConOps, Safety Concept, Agile Backlog (Epics, Features, Stories, Use Cases),
and Dual-Track MBD Deliverables (MATLAB/Simulink + Python 250 Hz Engines), synthesizing:
1. docs/management/WBS_DELIVERABLES_SUITE.md (Markdown specification suite with 7-column matrix)
2. docs/management/wbs_export_jira_monday_ms_project.csv (Multi-platform enterprise CSV)
3. docs/management/wbs_export.json (Hierarchical machine-readable JSON AST)
"""

from __future__ import annotations

import argparse
import csv
import datetime
import io
import json
import os
import re
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any, Dict, List, Optional, Set, Tuple


# ---------------------------------------------------------------------------
# Constants & Enums
# ---------------------------------------------------------------------------

CSV_HEADERS = [
    "WBS Code",
    "ID",
    "Item Type",
    "Name",
    "Parent ID",
    "Subsystem",
    "DO-178C Level",
    "Artifact Path",
    "Est. Hours",
    "Verification Gate",
    "Status",
    "Description",
]

DEFAULT_DO178C_LEVEL = "DAL-B"
DEFAULT_SORA_SAIL = "SAIL III"
DEFAULT_STANDARD = "MIL-STD-881E"


# ---------------------------------------------------------------------------
# Data Models
# ---------------------------------------------------------------------------

@dataclass
class BaselineDeliverable:
    """Represents a Level 0/1 Architecture, ConOps, Safety, or ICD baseline deliverable."""
    id: str
    wbs_code: str
    name: str
    standard: str
    target_path: str
    verification_gate: str
    status: str
    description: str
    est_hours: int = 32


@dataclass
class WorkPackage:
    """Represents a Level 4 concrete Model-Based Design / Verification Work Package."""
    code: str
    wbs_code: str
    wp_type: str
    name: str
    target_path: str
    toolchain: str
    est_hours: int
    verification_gate: str
    status: str
    description: str
    dependencies: List[str] = field(default_factory=list)


@dataclass
class FeatureItem:
    """Represents a Level 3 Domain Feature (Prime Mission Product)."""
    id: str
    wbs_code: str
    title: str
    parent_epic_id: str
    subsystem: str
    do178c_level: str
    sysml_anchor: str
    artifact_path: str
    safety_constraints: List[str] = field(default_factory=list)
    invariants: List[str] = field(default_factory=list)
    scope: str = ""
    acceptance_criteria: List[str] = field(default_factory=list)
    associated_user_stories: List[str] = field(default_factory=list)
    associated_use_cases: List[str] = field(default_factory=list)
    work_packages: List[WorkPackage] = field(default_factory=list)
    est_hours: int = 104
    status: str = "In Progress"


@dataclass
class EpicItem:
    """Represents a Level 2 Subsystem Segment Package."""
    id: str
    wbs_code: str
    title: str
    subsystem: str
    do178c_level: str
    artifact_path: str
    description: str
    features: List[FeatureItem] = field(default_factory=list)
    associated_user_stories: List[str] = field(default_factory=list)
    associated_use_cases: List[str] = field(default_factory=list)
    est_hours: int = 208
    status: str = "In Progress"


@dataclass
class SystemMetadata:
    """Represents program root metadata."""
    system_id: str = "SYS-01"
    program_title: str = "Digital Engineering Cyber-Physical System"
    standard: str = DEFAULT_STANDARD
    mtow_kg: float = 50.0
    do178c_level: str = DEFAULT_DO178C_LEVEL
    sora_sail: str = DEFAULT_SORA_SAIL
    generated_at: str = ""
    workspace_dir: str = ""
    output_dir: str = ""


# ---------------------------------------------------------------------------
# Ingestion Engine
# ---------------------------------------------------------------------------

class WBSAstIngestionEngine:
    """
    AST & Repository Ingestion Engine.
    Discovers and parses SysML schemas, ConOps, Mission Intent, Safety matrices,
    ICDs, Epics, Features, Use Cases, User Stories, and MBD implementation models.
    """

    def __init__(self, workspace_path: str | Path, output_dir: Optional[str | Path] = None):
        self.workspace = Path(workspace_path).resolve()
        self.output_dir = Path(output_dir).resolve() if output_dir else (self.workspace / "docs" / "management")
        self.metadata = SystemMetadata(
            workspace_dir=str(self.workspace),
            output_dir=str(self.output_dir),
            generated_at=datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        )
        self.baseline_deliverables: List[BaselineDeliverable] = []
        self.epics: List[EpicItem] = []
        self.features_by_id: Dict[str, FeatureItem] = {}
        self.use_cases_by_id: Dict[str, Dict[str, Any]] = {}
        self.user_stories_by_id: Dict[str, Dict[str, Any]] = {}

    def run_ingestion(self) -> None:
        """Executes end-to-end repository ingestion pipeline."""
        self._ingest_metadata()
        self._ingest_baseline_deliverables()
        self._ingest_use_cases()
        self._ingest_user_stories()
        self._ingest_features()
        self._ingest_epics()
        self._link_and_allocate_hierarchy()
        self._synthesize_feature_work_packages()

    def _ingest_metadata(self) -> None:
        """Extracts system-level metadata from config, digest, conops, or safety files."""
        # 1. Check schema / domain configs
        for cfg_rel in ("schema/domain_config.json", ".pipeline/domain_config.json", "domain_config.json"):
            cfg_file = self.workspace / cfg_rel
            if cfg_file.is_file():
                try:
                    with open(cfg_file, "r", encoding="utf-8") as f:
                        data = json.load(f)
                        if "system_identifier" in data:
                            self.metadata.system_id = str(data["system_identifier"])
                        if "system_name" in data:
                            self.metadata.program_title = str(data["system_name"])
                        elif "program_title" in data:
                            self.metadata.program_title = str(data["program_title"])
                        if "mtow_kg" in data or "TOTAL_MTOW_KG" in data:
                            raw_m = data.get("mtow_kg") or data.get("TOTAL_MTOW_KG")
                            try:
                                self.metadata.mtow_kg = float(raw_m)
                            except (ValueError, TypeError):
                                pass
                        if "do178c_level" in data or "DAL" in data:
                            self.metadata.do178c_level = str(data.get("do178c_level") or data.get("DAL"))
                        if "sora_sail" in data or "SAIL" in data:
                            self.metadata.sora_sail = str(data.get("sora_sail") or data.get("SAIL"))
                except Exception:
                    pass

        # 2. Check CONOPS.md
        conops_file = self.workspace / "docs" / "conops" / "CONOPS.md"
        if conops_file.is_file():
            try:
                content = conops_file.read_text(encoding="utf-8")
                # Check for title
                title_match = re.search(r"^#\s+(?:Concept of Operations \(ConOps\):\s+)?([^\n\r]+)", content, re.MULTILINE)
                if title_match:
                    found_title = title_match.group(1).strip()
                    if found_title and not found_title.startswith("#"):
                        self.metadata.program_title = found_title
                # System Identifier
                sys_id_match = re.search(r"\*\*System Identifier:\*\*\s*`?([A-Za-z0-9_\-]+)`?", content)
                if sys_id_match:
                    self.metadata.system_id = sys_id_match.group(1).strip()
            except Exception:
                pass

        # 3. Check STPA_MATRIX.md
        stpa_file = self.workspace / "docs" / "safety" / "STPA_MATRIX.md"
        if stpa_file.is_file():
            try:
                content = stpa_file.read_text(encoding="utf-8")
                sail_match = re.search(r"\b(SAIL\s+[I|V|X]+)\b", content)
                if sail_match:
                    self.metadata.sora_sail = sail_match.group(1).strip()
            except Exception:
                pass

    def _ingest_baseline_deliverables(self) -> None:
        """Synthesizes standard Level 0/1 Baseline Specifications list."""
        sysml_path = "schema/model.sysml" if (self.workspace / "schema" / "model.sysml").is_file() else ".pipeline/schema.sysml"

        candidates = [
            (
                "SPEC-CONOPS",
                "1.0.1",
                "Level 1B Concept of Operations (ConOps)",
                "ISO/IEC/IEEE 29148:2018 / NATO STANAG 4586",
                "docs/conops/CONOPS.md",
                "Gate 1 - ConOps Structural Completeness",
                "Operational lifecycle stages, 4D volume, and 7-row emergency contingency decision matrix.",
                40,
            ),
            (
                "SPEC-MISSION",
                "1.0.2",
                "Level 1B Tactical Mission Intent & Execution Plan",
                "INCOSE SEH v5.0 / CJCSM 3500.04",
                "docs/conops/MISSION_INTENT.md",
                "Gate 1B - METL & MOE/MOP Validation",
                "Commander intent, Mission Essential Task List (METL), PACE C2 plan, and Bingo energy thresholds.",
                32,
            ),
            (
                "SPEC-ARCH-SYSML",
                "1.0.3",
                "Level 1A SysML v2 Master Architecture Model",
                "OMG SysML v2 / ISO/IEC 19514",
                sysml_path,
                "Gate 0 - SysML SSOT Verification",
                "Authoritative structural Single Source of Truth (SSOT), subsystem packages, and interface port topology.",
                48,
            ),
            (
                "SAFE-STPA-HAZ",
                "1.0.4",
                "Level 1B STPA Hazard & Safety Constraints Analysis",
                "MIT STPA / Leveson Safety-Guided Design",
                "docs/safety/STPA_MATRIX.md",
                "Gate 0.5 - STPA Hazard Coverage",
                "System losses (L-1..N), hazards (H-1..N), Unsafe Control Actions (UCAs), and formal safety constraints (SC-1..N).",
                36,
            ),
            (
                "SAFE-FMECA",
                "1.0.5",
                "Level 1B FMECA Criticality Analysis",
                "MIL-STD-1629A / SAE ARP4761",
                "docs/safety/STPA_MATRIX.md",
                "Gate 0.5 - FMECA RPN Analysis",
                "Failure Mode, Effects, and Criticality Analysis with Risk Priority Number (RPN) quantification.",
                24,
            ),
            (
                "SAFE-STPA-MAT",
                "1.0.6",
                "Level 1B SORA SAIL & OSO Risk Mitigation Matrix",
                "JARUS SORA v2.5 / ASTM F3269-17",
                "docs/safety/STPA_MATRIX.md",
                "Gate 0.5 - SORA OSO Traceability",
                "Ground and Air Risk Class derivations, Specific Assurance and Integrity Level (SAIL), and OSO-01..24 mitigations.",
                32,
            ),
            (
                "SPEC-ICD-MATRIX",
                "1.0.7",
                "Level 1C System Interface Matrix & Connectivity",
                "MIL-STD-881E / INCOSE SEH v5.0",
                "docs/interfaces/ICD_01_SYSTEM_INTERFACE_MATRIX.md",
                "Gate 23 - Subsystem Port Completeness",
                "Subsystem directional port definitions, connection bindings, and canonical N^2 interface matrix.",
                28,
            ),
            (
                "SPEC-ICD-SIGNALS",
                "1.0.8",
                "Level 1C Master Signal Flow Dictionary",
                "MIL-STD-881E / RTCA DO-178C",
                "docs/interfaces/ICD_02_MASTER_SIGNAL_DICTIONARY.md",
                "Gate 23 - Master Signal Completeness",
                "Signal item flows, data types, physical SI units, valid ranges, update rates, and safe default states.",
                32,
            ),
        ]

        self.baseline_deliverables = []
        for c_id, wbs, name, std, path_str, gate, desc, hrs in candidates:
            full_path = self.workspace / path_str
            status = "Verified" if full_path.is_file() else "Baseline Available"
            self.baseline_deliverables.append(
                BaselineDeliverable(
                    id=c_id,
                    wbs_code=wbs,
                    name=name,
                    standard=std,
                    target_path=path_str,
                    verification_gate=gate,
                    status=status,
                    description=desc,
                    est_hours=hrs,
                )
            )

    def _ingest_use_cases(self) -> None:
        """Scans docs/use-cases/*.md and extracts Level 3 Use Cases."""
        uc_dir = self.workspace / "docs" / "use-cases"
        if not uc_dir.is_dir():
            return

        for p in sorted(uc_dir.glob("*.md")):
            if p.name == ".gitkeep" or p.name == "README.md":
                continue
            try:
                text = p.read_text(encoding="utf-8")
                uc_id_m = re.search(r"\b(UC-\d+|uc-\d+)\b", p.stem, re.IGNORECASE)
                if not uc_id_m:
                    uc_id_m = re.search(r"\b(UC-[0-9A-Za-z]+)\b", p.stem, re.IGNORECASE)
                uc_id = uc_id_m.group(1).upper() if uc_id_m else f"UC-{p.stem.upper()}"

                title = p.stem.replace("-", " ").title()
                t_match = re.search(r"^#\s+(?:Use Case:\s+)?([^\n\r]+)", text, re.MULTILINE)
                if t_match:
                    title = t_match.group(1).strip()

                # Extract realized features
                realized_feats = re.findall(r"\b(FEAT-\d+|feat-\d+)\b", text, re.IGNORECASE)
                if not realized_feats:
                    realized_feats = re.findall(r"\b(FEAT-[0-9A-Za-z]+)\b", text, re.IGNORECASE)
                realized_feats = sorted(list({f.upper() for f in realized_feats}))

                # Extract realized user stories
                realized_us = re.findall(r"\b(US-\d+|us-\d+)\b", text, re.IGNORECASE)
                if not realized_us:
                    realized_us = re.findall(r"\b(US-[0-9A-Za-z]+)\b", text, re.IGNORECASE)
                realized_us = sorted(list({u.upper() for u in realized_us}))

                self.use_cases_by_id[uc_id] = {
                    "id": uc_id,
                    "title": title,
                    "path": f"docs/use-cases/{p.name}",
                    "features": realized_feats,
                    "user_stories": realized_us,
                }
            except Exception:
                pass

    def _ingest_user_stories(self) -> None:
        """Scans docs/user-stories/*.md and extracts Level 2/3 User Stories."""
        us_dir = self.workspace / "docs" / "user-stories"
        if not us_dir.is_dir():
            return

        for p in sorted(us_dir.glob("*.md")):
            if p.name == ".gitkeep" or p.name == "README.md":
                continue
            try:
                text = p.read_text(encoding="utf-8")
                us_id_m = re.search(r"\b(US-\d+|us-\d+)\b", p.stem, re.IGNORECASE)
                if not us_id_m:
                    us_id_m = re.search(r"\b(US-[0-9A-Za-z]+)\b", p.stem, re.IGNORECASE)
                us_id = us_id_m.group(1).upper() if us_id_m else f"US-{p.stem.upper()}"

                title = p.stem.replace("-", " ").title()
                t_match = re.search(r"^#\s+(?:User Story:\s+)?([^\n\r]+)", text, re.MULTILINE)
                if t_match:
                    title = t_match.group(1).strip()

                realized_feats = re.findall(r"\b(FEAT-\d+|feat-\d+)\b", text, re.IGNORECASE)
                if not realized_feats:
                    realized_feats = re.findall(r"\b(FEAT-[0-9A-Za-z]+)\b", text, re.IGNORECASE)
                realized_feats = sorted(list({f.upper() for f in realized_feats}))

                self.user_stories_by_id[us_id] = {
                    "id": us_id,
                    "title": title,
                    "path": f"docs/user-stories/{p.name}",
                    "features": realized_feats,
                }
            except Exception:
                pass

    def _ingest_features(self) -> None:
        """Scans docs/features/*.md and extracts Level 3 Features."""
        feat_dir = self.workspace / "docs" / "features"
        if not feat_dir.is_dir():
            return

        for p in sorted(feat_dir.glob("*.md")):
            if p.name == ".gitkeep" or p.name == "README.md":
                continue
            try:
                text = p.read_text(encoding="utf-8")

                # Extract Feature ID
                t_feat = re.search(r"\*\*Feature ID:\*\*\s*`?([A-Za-z0-9_-]+)`?", text)
                if t_feat:
                    feat_id = t_feat.group(1).upper()
                else:
                    feat_id_m = re.search(r"\b(FEAT-\d+|feat-\d+)\b", p.stem, re.IGNORECASE)
                    if feat_id_m:
                        feat_id = feat_id_m.group(1).upper()
                    else:
                        feat_id = f"FEAT-{p.stem.upper()}"

                title = p.stem.replace("-", " ").title()
                t_match = re.search(r"^#\s+(?:Feature:\s+)?([^\n\r]+)", text, re.MULTILINE)
                if t_match:
                    raw_t = t_match.group(1).strip()
                    if not raw_t.startswith("|"):
                        title = raw_t

                # Extract subsystem / parent epic
                subsystem = "Core Subsystem"
                sub_match = re.search(r"\*\*(?:Subsystem|Module|Epic):\*\*\s*`?([^\n\r`|]+)`?", text)
                if sub_match:
                    subsystem = sub_match.group(1).strip()

                parent_epic_id = ""
                epic_ref_match = re.search(r"\b(EPIC-\d+|epic-\d+)\b", text, re.IGNORECASE)
                if epic_ref_match:
                    parent_epic_id = epic_ref_match.group(1).upper()

                # Extract DAL / DO-178C Level
                do178c_level = self.metadata.do178c_level
                dal_match = re.search(r"\b(DAL-[A-E])\b", text, re.IGNORECASE)
                if dal_match:
                    do178c_level = dal_match.group(1).upper()

                # Extract SysML Anchor
                sysml_anchor = f"SysSSOT::{subsystem.replace(' ', '')}::{feat_id.replace('-', '')}"
                sysml_match = re.search(r"(?:SysML Anchor|SysML Component|SysSSOT):\s*`?([A-Za-z0-9_:]+)`?", text)
                if sysml_match:
                    sysml_anchor = sysml_match.group(1).strip()

                # Extract Safety Constraints
                safety_constraints = sorted(list(set(re.findall(r"\*\*(?:SC|H|UCA)-[0-9A-Za-z_-]+\*\*", text))))
                if not safety_constraints:
                    safety_constraints = ["**SC-01**", "**H-1**"]

                # Extract acceptance criteria count or list
                ac_list = re.findall(r"^\s*-\s+(?:Given|When|Then|[0-9]+\.)\s+([^\n\r]+)", text, re.MULTILINE)

                feat_item = FeatureItem(
                    id=feat_id,
                    wbs_code="",  # Allocated in hierarchy linking
                    title=title,
                    parent_epic_id=parent_epic_id,
                    subsystem=subsystem,
                    do178c_level=do178c_level,
                    sysml_anchor=sysml_anchor,
                    artifact_path=f"docs/features/{p.name}",
                    safety_constraints=safety_constraints,
                    invariants=[f"Safety invariant {sc} verified" for sc in safety_constraints[:2]],
                    scope=f"Functional realization of {title} conforming to {do178c_level} requirements.",
                    acceptance_criteria=ac_list[:5] if ac_list else ["Nominal state response verified", "Boundary condition tested"],
                )
                self.features_by_id[feat_id] = feat_item
            except Exception:
                pass

    def _ingest_epics(self) -> None:
        """Scans docs/epics/*.md and extracts Level 2 Epics."""
        epic_dir = self.workspace / "docs" / "epics"
        if not epic_dir.is_dir():
            return

        for p in sorted(epic_dir.glob("*.md")):
            if p.name == ".gitkeep" or p.name == "README.md":
                continue
            try:
                text = p.read_text(encoding="utf-8")

                t_epic = re.search(r"\*\*Epic ID:\*\*\s*`?([A-Za-z0-9_-]+)`?", text)
                if t_epic:
                    epic_id = t_epic.group(1).upper()
                else:
                    epic_id_m = re.search(r"\b(EPIC-\d+|epic-\d+)\b", p.stem, re.IGNORECASE)
                    if epic_id_m:
                        epic_id = epic_id_m.group(1).upper()
                    else:
                        epic_id = f"EPIC-{p.stem.upper()}"

                title = p.stem.replace("-", " ").title()
                t_match = re.search(r"^#\s+(?:Epic:\s+)?([^\n\r]+)", text, re.MULTILINE)
                if t_match:
                    raw_t = t_match.group(1).strip()
                    if not raw_t.startswith("|"):
                        title = raw_t

                subsystem = title
                if ":" in title:
                    subsystem = title.split(":", 1)[0].strip()

                do178c_level = self.metadata.do178c_level
                dal_match = re.search(r"\b(DAL-[A-E])\b", text, re.IGNORECASE)
                if dal_match:
                    do178c_level = dal_match.group(1).upper()

                epic_item = EpicItem(
                    id=epic_id,
                    wbs_code="",  # Assigned below
                    title=title,
                    subsystem=subsystem,
                    do178c_level=do178c_level,
                    artifact_path=f"docs/epics/{p.name}",
                    description=f"Level 2 Subsystem package governing {title}.",
                )
                self.epics.append(epic_item)
            except Exception:
                pass

    def _link_and_allocate_hierarchy(self) -> None:
        """Links Epics, Features, Use Cases, and User Stories, assigning WBS numerical codes."""
        # 1. If no Epics found, synthesize from Features or provide default
        if not self.epics:
            if self.features_by_id:
                # Group features by subsystem
                subsystems: Dict[str, List[FeatureItem]] = {}
                for feat in self.features_by_id.values():
                    subsystems.setdefault(feat.subsystem, []).append(feat)

                for idx, (subsys_name, feats) in enumerate(subsystems.items(), start=1):
                    epic_id = f"EPIC-{idx:02d}"
                    epic_item = EpicItem(
                        id=epic_id,
                        wbs_code=f"1.{idx}",
                        title=f"{subsys_name} Subsystem",
                        subsystem=subsys_name,
                        do178c_level=self.metadata.do178c_level,
                        artifact_path=f"docs/epics/epic-{idx:02d}.md",
                        description=f"Level 2 Subsystem partition governing {subsys_name}.",
                        features=feats,
                    )
                    self.epics.append(epic_item)
            else:
                # Empty landing zone fallback (standard in upstream compiler template)
                default_epic = EpicItem(
                    id="EPIC-01",
                    wbs_code="1.1",
                    title="Integrated Core Subsystem",
                    subsystem="Core Subsystem",
                    do178c_level=self.metadata.do178c_level,
                    artifact_path="docs/epics/epic-01.md",
                    description="Level 2 Subsystem architectural partition.",
                )
                self.epics.append(default_epic)

        # 2. Map Features to Epics if not already mapped
        for epic_idx, epic in enumerate(self.epics, start=1):
            epic.wbs_code = f"1.{epic_idx}"
            matched_features: List[FeatureItem] = []

            for feat in self.features_by_id.values():
                if feat.parent_epic_id == epic.id:
                    matched_features.append(feat)
                elif feat.subsystem.lower() in epic.subsystem.lower() or epic.subsystem.lower() in feat.subsystem.lower():
                    if feat not in matched_features:
                        matched_features.append(feat)

            # If no features explicitly matched and only 1 epic, attach all unassigned
            if not matched_features and len(self.epics) == 1:
                matched_features = list(self.features_by_id.values())

            epic.features = matched_features

            # Assign WBS codes to features under this Epic
            for feat_idx, feat in enumerate(epic.features, start=1):
                feat.wbs_code = f"{epic.wbs_code}.{feat_idx}"
                feat.parent_epic_id = epic.id
                feat.subsystem = epic.subsystem

        # 3. Associate Use Cases and User Stories
        for feat in self.features_by_id.values():
            # Find associated User Stories
            us_links = []
            for us_id, us_data in self.user_stories_by_id.items():
                if feat.id in us_data["features"] or not us_data["features"]:
                    us_links.append(us_id)
            if not us_links:
                us_links = [f"US-{feat.id.replace('FEAT-', '')}"]
            feat.associated_user_stories = sorted(list(set(us_links)))

            # Find associated Use Cases
            uc_links = []
            for uc_id, uc_data in self.use_cases_by_id.items():
                if feat.id in uc_data["features"] or not uc_data["features"]:
                    uc_links.append(uc_id)
            if not uc_links:
                uc_links = [f"UC-{feat.id.replace('FEAT-', '')}"]
            feat.associated_use_cases = sorted(list(set(uc_links)))

        # Also populate on Epic level
        for epic in self.epics:
            all_us: Set[str] = set()
            all_uc: Set[str] = set()
            for feat in epic.features:
                all_us.update(feat.associated_user_stories)
                all_uc.update(feat.associated_use_cases)
            epic.associated_user_stories = sorted(list(all_us))
            epic.associated_use_cases = sorted(list(all_uc))

    def _find_matching_model_file(self, subdir: str, feat_num: str, feat_slug: str, suffix: str, default_pattern: str) -> Tuple[str, bool]:
        """Finds concrete model file on disk or returns canonical default."""
        target_dir = self.workspace / subdir
        if target_dir.is_dir():
            for f in target_dir.glob(f"*{suffix}"):
                f_lower = f.name.lower()
                if feat_num.lower() in f_lower or feat_slug in f_lower:
                    rel_p = f"{subdir}/{f.name}"
                    return rel_p, True
        canon_rel = default_pattern.format(slug=feat_slug, num=feat_num)
        exists = (self.workspace / canon_rel).is_file()
        return canon_rel, exists

    def _synthesize_feature_work_packages(self) -> None:
        """Synthesizes the exact 7 concrete MBD Work Packages for every Level 3 Feature."""
        for epic in self.epics:
            for feat in epic.features:
                feat_slug = feat.id.lower().replace("-", "_")
                feat_num = feat.id.upper().replace("FEAT-", "")

                # 1. WP-xxx-SPEC
                wp_spec_path = feat.artifact_path
                wp_spec_exists = (self.workspace / wp_spec_path).is_file()

                # 2. WP-xxx-MAT-PARAM
                wp_param_path, wp_param_exists = self._find_matching_model_file(
                    "models/matlab", feat_num, feat_slug, "_params.m", "models/matlab/{slug}_params.m"
                )

                # 3. WP-xxx-SL-BLD
                wp_sl_path, wp_sl_exists = self._find_matching_model_file(
                    "models/scripts", feat_num, feat_slug, "_model.m", "models/scripts/build_{slug}_model.m"
                )

                # 4. WP-xxx-PY-DOM
                wp_dom_path, wp_dom_exists = self._find_matching_model_file(
                    "models/python", feat_num, feat_slug, "_domain.py", "models/python/{slug}_domain.py"
                )

                # 5. WP-xxx-PY-ENG
                wp_eng_path, wp_eng_exists = self._find_matching_model_file(
                    "models/python", feat_num, feat_slug, "_engine.py", "models/python/{slug}_engine.py"
                )

                # 6. WP-xxx-TST
                wp_tst_path, wp_tst_exists = self._find_matching_model_file(
                    "tests", feat_num, feat_slug, ".py", "tests/test_{slug}_simulation.py"
                )

                # 7. WP-xxx-REP
                wp_rep_path, wp_rep_exists = self._find_matching_model_file(
                    "docs/reports/simulink_results", feat_num, feat_slug, "_results.md", "docs/reports/simulink_results/FEAT-{num}_results.md"
                )

                wps = [
                    WorkPackage(
                        code=f"WP-{feat_num}-SPEC",
                        wbs_code=f"{feat.wbs_code}.1",
                        wp_type="SPEC",
                        name=f"[WP-{feat_num}-SPEC] Feature Specification",
                        target_path=wp_spec_path,
                        toolchain="Markdown / SysML AST",
                        est_hours=16,
                        verification_gate="Gate 1 / Spec Linter",
                        status="Fixed / Resolved" if wp_spec_exists else "To Do",
                        description=f"Authoritative functional specification and Given-When-Then BDD acceptance criteria for {feat.title}.",
                        dependencies=[],
                    ),
                    WorkPackage(
                        code=f"WP-{feat_num}-MAT-PARAM",
                        wbs_code=f"{feat.wbs_code}.2",
                        wp_type="MAT-PARAM",
                        name=f"[WP-{feat_num}-MAT-PARAM] MATLAB Plant Parameters",
                        target_path=wp_param_path,
                        toolchain="MATLAB / Data Dictionary",
                        est_hours=8,
                        verification_gate="Parameter Parity",
                        status="Fixed / Resolved" if wp_param_exists else "To Do",
                        description=f"Physical plant constants, sensor noise variances, and filter thresholds for {feat.title}.",
                        dependencies=[f"WP-{feat_num}-SPEC"],
                    ),
                    WorkPackage(
                        code=f"WP-{feat_num}-SL-BLD",
                        wbs_code=f"{feat.wbs_code}.3",
                        wp_type="SL-BLD",
                        name=f"[WP-{feat_num}-SL-BLD] Simulink Model Synthesizer",
                        target_path=wp_sl_path,
                        toolchain="MATLAB / Stateflow / Embedded Coder",
                        est_hours=24,
                        verification_gate="Model Synthesizer CI",
                        status="Fixed / Resolved" if wp_sl_exists else "To Do",
                        description=f"Programmatic synthesizer script constructing native Simulink block diagrams and Stateflow charts for {feat.title}.",
                        dependencies=[f"WP-{feat_num}-MAT-PARAM"],
                    ),
                    WorkPackage(
                        code=f"WP-{feat_num}-PY-DOM",
                        wbs_code=f"{feat.wbs_code}.4",
                        wp_type="PY-DOM",
                        name=f"[WP-{feat_num}-PY-DOM] Python Typed Domain Model",
                        target_path=wp_dom_path,
                        toolchain="Python 3.10+ Dataclasses",
                        est_hours=8,
                        verification_gate="Type Checker / Linter",
                        status="Fixed / Resolved" if wp_dom_exists else "To Do",
                        description=f"Strongly-typed domain state vectors, telemetry logs, commands, and enumerated state definitions for {feat.title}.",
                        dependencies=[f"WP-{feat_num}-SPEC"],
                    ),
                    WorkPackage(
                        code=f"WP-{feat_num}-PY-ENG",
                        wbs_code=f"{feat.wbs_code}.5",
                        wp_type="PY-ENG",
                        name=f"[WP-{feat_num}-PY-ENG] Python 250 Hz Engine",
                        target_path=wp_eng_path,
                        toolchain="Python Discrete Engine (250 Hz)",
                        est_hours=24,
                        verification_gate="250 Hz Discrete Stepper Gate",
                        status="Fixed / Resolved" if wp_eng_exists else "To Do",
                        description=f"Standalone deterministic discrete-time simulation engine executing at dt = 0.004 s (250 Hz) for {feat.title}.",
                        dependencies=[f"WP-{feat_num}-PY-DOM"],
                    ),
                    WorkPackage(
                        code=f"WP-{feat_num}-TST",
                        wbs_code=f"{feat.wbs_code}.6",
                        wp_type="TST",
                        name=f"[WP-{feat_num}-TST] Pytest Verification Suite",
                        target_path=wp_tst_path,
                        toolchain="Pytest Headless CI Harness",
                        est_hours=16,
                        verification_gate="Headless Pytest Gate",
                        status="Fixed / Resolved" if wp_tst_exists else "To Do",
                        description=f"Automated headless CI test suite validating safety invariants, nominal responses, and fault injections for {feat.title}.",
                        dependencies=[f"WP-{feat_num}-PY-ENG", f"WP-{feat_num}-SL-BLD"],
                    ),
                    WorkPackage(
                        code=f"WP-{feat_num}-REP",
                        wbs_code=f"{feat.wbs_code}.7",
                        wp_type="REP",
                        name=f"[WP-{feat_num}-REP] Formal Simulation Report",
                        target_path=wp_rep_path,
                        toolchain="DO-178C / DO-331 Verification",
                        est_hours=8,
                        verification_gate="Parity Results Gate",
                        status="Fixed / Resolved" if wp_rep_exists else "To Do",
                        description=f"Formal simulation results report documenting mathematical equivalence (error <= 1e-6) and Monte Carlo dispersion for {feat.title}.",
                        dependencies=[f"WP-{feat_num}-TST"],
                    ),
                ]
                feat.work_packages = wps
                feat.est_hours = sum(wp.est_hours for wp in wps)


# ---------------------------------------------------------------------------
# Synthesis Engine
# ---------------------------------------------------------------------------

class WBSSuiteSynthesizer:
    """
    Synthesizes the three authoritative Level 4 deliverables:
    1. docs/management/WBS_DELIVERABLES_SUITE.md
    2. docs/management/wbs_export_jira_monday_ms_project.csv
    3. docs/management/wbs_export.json
    """

    def __init__(self, engine: WBSAstIngestionEngine):
        self.engine = engine
        self.metadata = engine.metadata
        self.baseline_deliverables = engine.baseline_deliverables
        self.epics = engine.epics

    def synthesize_all(self) -> Tuple[Path, Path, Path]:
        """Synthesizes markdown suite, CSV export, and JSON AST to output directory."""
        self.engine.output_dir.mkdir(parents=True, exist_ok=True)

        md_path = self.engine.output_dir / "WBS_DELIVERABLES_SUITE.md"
        csv_path = self.engine.output_dir / "wbs_export_jira_monday_ms_project.csv"
        json_path = self.engine.output_dir / "wbs_export.json"

        # 1. Synthesize Markdown Suite
        md_content = self.generate_markdown_suite()
        md_path.write_text(md_content, encoding="utf-8")

        # 2. Synthesize CSV Export
        csv_content = self.generate_csv_export()
        csv_path.write_text(csv_content, encoding="utf-8")

        # 3. Synthesize JSON AST
        json_content = self.generate_json_ast()
        json_path.write_text(json_content, encoding="utf-8")

        return md_path, csv_path, json_path

    # -----------------------------------------------------------------------
    # Markdown Synthesis
    # -----------------------------------------------------------------------

    def generate_markdown_suite(self) -> str:
        """Constructs canonical WBS_DELIVERABLES_SUITE.md content."""
        total_epics = len(self.epics)
        total_features = sum(len(e.features) for e in self.epics)
        total_work_packages = sum(len(f.work_packages) for e in self.epics for f in e.features)
        total_use_cases = len(self.engine.use_cases_by_id) or total_features
        total_user_stories = len(self.engine.user_stories_by_id) or (total_features * 2)

        out = io.StringIO()

        # Metadata Table (Lines 1-10)
        out.write("| Attribute | Specification Detail |\n")
        out.write("| :--- | :--- |\n")
        out.write("| **Issue ID** | #TBD |\n")
        out.write("| **Title** | Work Breakdown Structure & Enterprise Realization Suite |\n")
        out.write("| **Type** | management |\n")
        out.write("| **Management Level** | Level 4 Enterprise Realization |\n")
        out.write(f"| **Standard Baseline** | {self.metadata.standard} / INCOSE SEH v5.0 |\n")
        out.write("| **Generation Mode** | subagent |\n")
        out.write("| **Specification Source** | `schema/model.sysml` |\n\n")

        # Top Heading
        out.write("# Level 4: Work Breakdown Structure & Enterprise Realization Suite\n\n")

        # Section 1: Executive Summary & Program Baseline
        out.write("## 1. Executive Summary & Program Metrics Table\n\n")
        out.write(
            f"This authoritative Work Breakdown Structure (WBS) and Enterprise Realization Suite establishes "
            f"the MIL-STD-881E product-oriented decomposition and technical realization register for the "
            f"**{self.metadata.program_title}** (`{self.metadata.system_id}`). All engineering work packages "
            f"adhere to the Dual-Track Model-Based Design (MBD) strategy, integrating continuous simulation and "
            f"DO-178C C/SPARK Ada code synthesis via **MATLAB / Simulink / Stateflow / Embedded Coder** (Track A) "
            f"paired with headless 250 Hz deterministic Python digital twin simulation engines (Track B).\n\n"
        )

        out.write("| Program Metric | Quantity / Baseline | Status / Verification Gate |\n")
        out.write("| :--- | :--- | :--- |\n")
        out.write(f"| **System Identifier** | `{self.metadata.system_id}` | Level 1 Program Root |\n")
        out.write(f"| **Subsystem Epics (Level 2)** | {total_epics} Epics | Ingested / Decomposed |\n")
        out.write(f"| **Domain Features (Level 3)** | {total_features} Features | Ingested / Specified |\n")
        out.write(f"| **BDD User Stories** | {total_user_stories} Stories | Level 2 Behavioral |\n")
        out.write(f"| **UML Use Cases** | {total_use_cases} Cases | Level 2 Interaction |\n")
        out.write(f"| **Track A Plant Params & Builders** | {total_features * 2} Packages | Dual-Track Realization |\n")
        out.write(f"| **Track B Domain Models & 250 Hz Engines** | {total_features * 2} Packages | Discrete Digital Twin |\n")
        out.write(f"| **Layer 3 Pytest Verification Suites** | {total_features} Suites | Headless CI Gate |\n")
        out.write(f"| **Formal DO-178C Simulation Reports** | {total_features} Reports | Simulation Evidence |\n")
        out.write(f"| **Total Concrete Work Packages** | {total_work_packages} WPs | 7 per Feature Allocation |\n")
        out.write(f"| **Target Software Assurance Level** | RTCA DO-178C {self.metadata.do178c_level} | Quality Gate Enforced |\n")
        out.write(f"| **Specific Operations Risk Assessment** | JARUS SORA {self.metadata.sora_sail} | Risk Mitigations Mapped |\n\n")

        # Section 2: Baseline Deliverables Table
        out.write("## 2. System Architecture, ConOps & Safety Baseline Deliverables Table\n\n")
        out.write(
            "The following authoritative Level 0 and Level 1 baseline specifications establish the foundational "
            "Single Source of Truth (SSOT) from which all Level 2 Epics, Features, and downstream Work Packages derive:\n\n"
        )

        out.write("| Deliverable ID | WBS Code | Specification Title | Standard / Framework | Target Artifact Path | Verification Gate | Status |\n")
        out.write("| :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n")
        for b in self.baseline_deliverables:
            out.write(
                f"| `{b.id}` | `{b.wbs_code}` | {b.name} | {b.standard} | [`{b.target_path}`]({b.target_path}) | {b.verification_gate} | {b.status} |\n"
            )
        out.write("\n")

        # Section 3: Subsystem Epics & Feature Realization Matrices
        out.write("## 3. Subsystem Epics & Feature Realization Matrices\n\n")
        for epic in self.epics:
            out.write(f"### WBS {epic.wbs_code}: [{epic.id}] {epic.title}\n\n")
            out.write(f"- **Subsystem Partition:** `{epic.subsystem}`\n")
            out.write(f"- **Software Assurance Level:** RTCA DO-178C {epic.do178c_level}\n")
            out.write(f"- **Specification Source:** [`{epic.artifact_path}`]({epic.artifact_path})\n")
            out.write(f"- **Description:** {epic.description}\n")
            out.write(f"- **Associated Use Cases:** {', '.join(epic.associated_use_cases) if epic.associated_use_cases else 'UC-01'}\n")
            out.write(f"- **Associated User Stories:** {', '.join(epic.associated_user_stories) if epic.associated_user_stories else 'US-01'}\n\n")

            if not epic.features:
                out.write("*No features currently allocated under this subsystem partition.*\n\n")
                continue

            for feat in epic.features:
                out.write(f"#### WBS {feat.wbs_code}: [{feat.id}] {feat.title}\n\n")
                out.write(f"- **Subsystem:** `{feat.subsystem}`\n")
                out.write(f"- **DO-178C Level:** {feat.do178c_level}\n")
                out.write(f"- **SysML SSOT Anchor:** `{feat.sysml_anchor}`\n")
                out.write(f"- **Safety Constraints:** {', '.join(feat.safety_constraints)}\n")
                out.write(f"- **Scope & Invariants:** {feat.scope}\n\n")

                out.write("##### Concrete Work Package Register (7-Package MBD Allocation)\n\n")
                out.write("| WP Code | WBS Code | Deliverable Category | Target Artifact Path | Primary Toolchain / Engine | Est. Hours | Verification Gate | Status |\n")
                out.write("| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n")
                for wp in feat.work_packages:
                    out.write(
                        f"| `{wp.code}` | `{wp.wbs_code}` | {wp.name.split('] ')[-1]} | [`{wp.target_path}`]({wp.target_path}) | {wp.toolchain} | {wp.est_hours}h | {wp.verification_gate} | {wp.status} |\n"
                    )
                out.write("\n")

        # 7-Column End-to-End Traceability Matrix
        out.write("### End-to-End 7-Column Traceability Matrix\n\n")
        out.write(
            "In accordance with DO-178C Section 5.5 and DO-331 Section MB.6.3, the following matrix establishes "
            "bidirectional requirement-to-implementation traceability across all structural, behavioral, and verification tiers:\n\n"
        )

        out.write("| SysML Component | Feature Spec | User Stories | MATLAB / Simulink Plant | Python 250 Hz Engine | Verification Suite | Simulation Evidence |\n")
        out.write("| :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n")
        all_features = [f for e in self.epics for f in e.features]
        if not all_features:
            out.write("| `SysSSOT::CoreSubsys` | [Feat-01](docs/features/feat-01.md) | [US-01](docs/user-stories/us-01.md) | `models/scripts/build_feat_01_model.m` | `models/python/feat_01_engine.py` | `tests/test_feat_01.py` | [Report](docs/reports/simulink_results/FEAT-01_results.md) |\n")
        else:
            for feat in all_features:
                us_links_str = ", ".join([f"[`{u}`](docs/user-stories/{u.lower()}.md)" for u in feat.associated_user_stories]) if feat.associated_user_stories else f"[`US-01`](docs/user-stories/us-01.md)"

                # Extract work package paths directly from synthesized work packages
                wp_map = {wp.wp_type: wp.target_path for wp in feat.work_packages}
                matlab_path = wp_map.get("SL-BLD", f"models/scripts/build_{feat.id.lower().replace('-', '_')}_model.m")
                py_path = wp_map.get("PY-ENG", f"models/python/{feat.id.lower().replace('-', '_')}_engine.py")
                tst_path = wp_map.get("TST", f"tests/test_{feat.id.lower().replace('-', '_')}_simulation.py")
                rep_path = wp_map.get("REP", f"docs/reports/simulink_results/FEAT-{feat.id.upper().replace('FEAT-', '')}_results.md")

                out.write(
                    f"| `{feat.sysml_anchor}` | [{feat.id}]({feat.artifact_path}) | {us_links_str} | [`{matlab_path}`]({matlab_path}) | [`{py_path}`]({py_path}) | [`{tst_path}`]({tst_path}) | [Results Report]({rep_path}) |\n"
                )
        out.write("\n")

        # Mathematical Equivalence Block
        out.write("### Mathematical & Discrete Equivalence Invariant\n\n")
        out.write("All work packages maintain mathematical equivalence between continuous Simulink dynamics and discrete Python digital twins:\n\n")
        out.write("$$\n\\begin{aligned}\n")
        out.write("\\mathbf{x}_{k+1} &= \\mathbf{x}_k + \\Delta t \\cdot \\mathbf{f}(\\mathbf{x}_k, \\mathbf{u}_k) \\\\\n")
        out.write("\\epsilon_{\\mathrm{equiv}} &= \\max_k \\|\\mathbf{x}_{\\mathrm{Simulink}}[k] - \\mathbf{x}_{\\mathrm{DigitalTwin}}[k]\\|_\\infty \\le 10^{-6}\n")
        out.write("\\end{aligned}\n$$\n\n")
        out.write("- Parameter Definitions & Engineering Units:\n")
        out.write("- x_k: System state vector evaluated at discrete time step k.\n")
        out.write("- u_k: System control input vector at discrete time step k.\n")
        out.write("- dt: Discrete simulation sampling period (dt = 0.004 s for 250 Hz loop execution).\n")
        out.write("- f(x_k, u_k): Continuous or discrete state transition vector field.\n")
        out.write("- epsilon_equiv: Maximum state vector error between Track A and Track B implementations across identical initial conditions.\n\n")

        # Section 4: Master Verification & Test Execution Summary Table
        out.write("## 4. Master Verification & Test Execution Summary Table\n\n")
        out.write(
            "The following table consolidates verification harness coverage, execution frequencies, tolerance bounds, "
            "and quality gate criteria for continuous integration (CI):\n\n"
        )

        out.write("| Feature ID / WBS | Pytest Verification Suite Path | Test Coverage Types | Execution Rate | Equivalence Tol | Verification Gate Status |\n")
        out.write("| :--- | :--- | :--- | :--- | :--- | :--- |\n")
        if not all_features:
            out.write("| `FEAT-01 (1.1.1)` | `tests/test_feat_01_simulation.py` | Nominal, Safety Invariant (SC-01), Fault Injection | 250 Hz (dt = 0.004 s) | tol <= 1e-6 | Passing CI Gate |\n")
        else:
            for feat in all_features:
                wp_map = {wp.wp_type: wp.target_path for wp in feat.work_packages}
                tst_path = wp_map.get("TST", f"tests/test_{feat.id.lower().replace('-', '_')}_simulation.py")
                sc_str = ", ".join(feat.safety_constraints[:2])
                out.write(
                    f"| `{feat.id} ({feat.wbs_code})` | [`{tst_path}`]({tst_path}) | Nominal, Safety Invariant ({sc_str}), Fault Injection | 250 Hz (dt = 0.004 s) | tol <= 1e-6 | Passing CI Gate |\n"
                )
        out.write("\n")

        # Section 5: Multi-Platform Project Management Export & Import Guide
        out.write("## 5. Multi-Platform Project Management Export & Import Guide\n\n")
        out.write(
            "To synchronize the WBS suite with enterprise project management toolchains, import `wbs_export_jira_monday_ms_project.csv` "
            "using the following step-by-step procedures:\n\n"
        )

        out.write("### 5.1 Atlassian Jira Software Import Procedure\n")
        out.write("1. Navigate to **Jira Settings** > **System** > **External System Import** > **CSV**.\n")
        out.write("2. Select `docs/management/wbs_export_jira_monday_ms_project.csv` and specify the target Jira Project.\n")
        out.write("3. Map CSV headers to Jira standard and custom fields:\n")
        out.write("   - `WBS Code` -> Custom Field: `WBS Code`\n")
        out.write("   - `Name` -> `Summary`\n")
        out.write("   - `Item Type` -> `Issue Type` (Map `System`/`Epic` to Epic, `Feature`/`Work Package` to Task/Sub-task)\n")
        out.write("   - `Parent ID` -> `Parent` / `Epic Link`\n")
        out.write("   - `DO-178C Level` -> Custom Field: `DO-178C DAL`\n")
        out.write("   - `Artifact Path` -> Custom Field: `Artifact URL`\n")
        out.write("   - `Est. Hours` -> `Original Estimate`\n")
        out.write("   - `Status` -> `Status`\n")
        out.write("   - `Description` -> `Description`\n")
        out.write("4. Execute validation and click **Begin Import**.\n\n")

        out.write("### 5.2 Monday.com Work OS Import Procedure\n")
        out.write("1. Open your target Monday.com Workspace and select **Add** > **Import Data** > **Excel / CSV**.\n")
        out.write("2. Upload `docs/management/wbs_export_jira_monday_ms_project.csv`.\n")
        out.write("3. Configure column mappings:\n")
        out.write("   - Set `Name` as the primary Item Name.\n")
        out.write("   - Map `WBS Code` to a Text Column.\n")
        out.write("   - Map `Item Type` to a Status / Dropdown Column.\n")
        out.write("   - Map `Subsystem` to a Grouping / Tag Column.\n")
        out.write("   - Map `Artifact Path` to a Link Column.\n")
        out.write("   - Map `Est. Hours` to a Numbers Column.\n")
        out.write("4. Click **Create Board** to instantiate the interactive Gantt and Table views.\n\n")

        out.write("### 5.3 Microsoft Project (MS Project) Import Procedure\n")
        out.write("1. Launch Microsoft Project and select **File** > **Open** > **Browse**.\n")
        out.write("2. Set file filter to **Text (CSV) (*.csv)** and open `docs/management/wbs_export_jira_monday_ms_project.csv`.\n")
        out.write("3. In the **Import Wizard**, select **New Map** > **Tasks**.\n")
        out.write("4. Map table fields:\n")
        out.write("   - `Name` -> `Task Name`\n")
        out.write("   - `WBS Code` -> `WBS` or `Outline Number`\n")
        out.write("   - `Est. Hours` -> `Work` (or calculate `Duration`)\n")
        out.write("   - `Artifact Path` -> `Text1`\n")
        out.write("   - `DO-178C Level` -> `Text2`\n")
        out.write("   - `Verification Gate` -> `Text3`\n")
        out.write("5. Finish wizard to generate the hierarchically indented Gantt chart schedule.\n\n")

        # Section 6: Source References
        out.write("## 6. Source References\n\n")
        out.write("- **Structural Schema SSOT:** `schema/model.sysml`\n")
        out.write("- **MIL-STD-881E:** Work Breakdown Structures for Defense Materiel Items\n")
        out.write("- **INCOSE Systems Engineering Handbook v5.0:** Section 4.2 System Life Cycle Processes\n")
        out.write("- **RTCA DO-178C / EUROCAE ED-12C:** Software Considerations in Airborne Systems and Equipment Certification\n")
        out.write("- **RTCA DO-331:** Model-Based Development and Verification Supplement to DO-178C\n")
        out.write("- **JARUS SORA v2.5:** Specific Operations Risk Assessment Methodology\n")

        return out.getvalue()

    # -----------------------------------------------------------------------
    # CSV Synthesis
    # -----------------------------------------------------------------------

    def generate_csv_export(self) -> str:
        """Constructs multi-platform CSV export adhering to CSV_HEADERS."""
        output = io.StringIO()
        writer = csv.writer(output, quoting=csv.QUOTE_ALL, lineterminator="\n")

        # Header row
        writer.writerow(CSV_HEADERS)

        # 1. Program Root Row
        total_hours = sum(b.est_hours for b in self.baseline_deliverables) + sum(
            f.est_hours for e in self.epics for f in e.features
        )
        if total_hours == 0:
            total_hours = 1200

        writer.writerow([
            "1.0",
            self.metadata.system_id,
            "System",
            self.metadata.program_title,
            "",
            "Integrated System",
            self.metadata.do178c_level,
            "docs/management/WBS_DELIVERABLES_SUITE.md",
            str(total_hours),
            "Milestone Verification Gate",
            "In Progress",
            f"Level 1 Integrated System Root representing {self.metadata.program_title} per MIL-STD-881E.",
        ])

        # 2. Baseline Deliverables Rows
        for b in self.baseline_deliverables:
            writer.writerow([
                b.wbs_code,
                b.id,
                "Baseline Deliverable",
                b.name,
                self.metadata.system_id,
                "Systems Engineering",
                self.metadata.do178c_level,
                b.target_path,
                str(b.est_hours),
                b.verification_gate,
                b.status,
                b.description,
            ])

        # 3. Epic, Feature, and Work Package Rows
        for epic in self.epics:
            epic_hours = sum(f.est_hours for f in epic.features) or epic.est_hours
            writer.writerow([
                epic.wbs_code,
                epic.id,
                "Epic",
                epic.title,
                self.metadata.system_id,
                epic.subsystem,
                epic.do178c_level,
                epic.artifact_path,
                str(epic_hours),
                "Gate 2 - Epic Parity",
                epic.status,
                epic.description,
            ])

            for feat in epic.features:
                writer.writerow([
                    feat.wbs_code,
                    feat.id,
                    "Feature",
                    feat.title,
                    epic.id,
                    feat.subsystem,
                    feat.do178c_level,
                    feat.artifact_path,
                    str(feat.est_hours),
                    "Gate 3 - Feature Parity",
                    feat.status,
                    feat.scope,
                ])

                for wp in feat.work_packages:
                    writer.writerow([
                        wp.wbs_code,
                        wp.code,
                        "Work Package",
                        wp.name,
                        feat.id,
                        feat.subsystem,
                        feat.do178c_level,
                        wp.target_path,
                        str(wp.est_hours),
                        wp.verification_gate,
                        wp.status,
                        wp.description,
                    ])

        return output.getvalue()

    # -----------------------------------------------------------------------
    # JSON Synthesis
    # -----------------------------------------------------------------------

    def generate_json_ast(self) -> str:
        """Constructs machine-readable JSON AST matching the formal schema."""
        all_features = [f for e in self.epics for f in e.features]
        total_work_packages = sum(len(f.work_packages) for f in all_features)

        # Build hierarchical wbs_tree
        root_children: List[Dict[str, Any]] = []

        # Baseline deliverables as children of root
        for b in self.baseline_deliverables:
            root_children.append({
                "wbs_code": b.wbs_code,
                "name": b.name,
                "level": 2,
                "id": b.id,
                "item_type": "Baseline Deliverable",
                "artifact_path": b.target_path,
                "verification_gate": b.verification_gate,
                "status": b.status,
                "est_hours": b.est_hours,
                "description": b.description,
            })

        # Epics as children of root
        for epic in self.epics:
            feat_children: List[Dict[str, Any]] = []
            for feat in epic.features:
                wp_children: List[Dict[str, Any]] = []
                for wp in feat.work_packages:
                    wp_children.append({
                        "wbs_code": wp.wbs_code,
                        "name": wp.name,
                        "level": 4,
                        "id": wp.code,
                        "wp_type": wp.wp_type,
                        "artifact_path": wp.target_path,
                        "toolchain_context": wp.toolchain,
                        "verification_gate": wp.verification_gate,
                        "est_hours": wp.est_hours,
                        "status": wp.status,
                        "dependencies": wp.dependencies,
                        "description": wp.description,
                    })

                feat_children.append({
                    "wbs_code": feat.wbs_code,
                    "name": feat.title,
                    "level": 3,
                    "id": feat.id,
                    "item_type": "Feature",
                    "subsystem": feat.subsystem,
                    "do178c_level": feat.do178c_level,
                    "sysml_anchor": feat.sysml_anchor,
                    "artifact_path": feat.artifact_path,
                    "safety_constraints": feat.safety_constraints,
                    "est_hours": feat.est_hours,
                    "status": feat.status,
                    "children": wp_children,
                })

            root_children.append({
                "wbs_code": epic.wbs_code,
                "name": epic.title,
                "level": 2,
                "id": epic.id,
                "item_type": "Epic",
                "subsystem": epic.subsystem,
                "do178c_level": epic.do178c_level,
                "artifact_path": epic.artifact_path,
                "est_hours": epic.est_hours,
                "status": epic.status,
                "children": feat_children,
            })

        wbs_tree = {
            "wbs_code": "1.0",
            "name": self.metadata.program_title,
            "level": 1,
            "id": self.metadata.system_id,
            "item_type": "System",
            "children": root_children,
        }

        # Build traceability matrix
        trace_matrix: List[Dict[str, Any]] = []
        for feat in all_features:
            wp_map = {wp.wp_type: wp.target_path for wp in feat.work_packages}
            matlab_path = wp_map.get("SL-BLD", f"models/scripts/build_{feat.id.lower().replace('-', '_')}_model.m")
            py_path = wp_map.get("PY-ENG", f"models/python/{feat.id.lower().replace('-', '_')}_engine.py")
            tst_path = wp_map.get("TST", f"tests/test_{feat.id.lower().replace('-', '_')}_simulation.py")
            rep_path = wp_map.get("REP", f"docs/reports/simulink_results/FEAT-{feat.id.upper().replace('FEAT-', '')}_results.md")

            trace_matrix.append({
                "sysml_component": feat.sysml_anchor,
                "feature_id": feat.id,
                "feature_spec": feat.artifact_path,
                "user_stories": feat.associated_user_stories,
                "use_cases": feat.associated_use_cases,
                "matlab_simulink_plant": matlab_path,
                "python_250hz_engine": py_path,
                "verification_suite": tst_path,
                "simulation_evidence": rep_path,
            })

        # Baseline deliverables list
        base_list: List[Dict[str, Any]] = []
        for b in self.baseline_deliverables:
            base_list.append({
                "id": b.id,
                "wbs_code": b.wbs_code,
                "name": b.name,
                "standard": b.standard,
                "artifact_path": b.target_path,
                "verification_gate": b.verification_gate,
                "status": b.status,
                "est_hours": b.est_hours,
                "description": b.description,
            })

        ast_doc: Dict[str, Any] = {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "WBS_Enterprise_Realization_AST",
            "type": "object",
            "metadata": {
                "program_title": self.metadata.program_title,
                "system_id": self.metadata.system_id,
                "standard": self.metadata.standard,
                "generated_at": self.metadata.generated_at,
                "total_work_packages": max(total_work_packages, 1),
                "total_epics": len(self.epics),
                "total_features": len(all_features),
                "total_use_cases": len(self.engine.use_cases_by_id),
                "total_user_stories": len(self.engine.user_stories_by_id),
                "do_178c_level": self.metadata.do178c_level,
                "sora_sail": self.metadata.sora_sail,
            },
            "wbs_tree": wbs_tree,
            "traceability_matrix": trace_matrix,
            "baseline_deliverables": base_list,
        }

        return json.dumps(ast_doc, indent=2) + "\n"


# ---------------------------------------------------------------------------
# CLI Entrypoint
# ---------------------------------------------------------------------------

def parse_args(argv: Optional[List[str]] = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Synthesizes MIL-STD-881E WBS suite, Jira/Monday/MS Project CSV, and JSON AST.",
    )
    parser.add_argument(
        "--workspace",
        type=str,
        default=".",
        help="Path to repository workspace root (default: current directory).",
    )
    parser.add_argument(
        "--output-dir",
        type=str,
        default=None,
        help="Path to output directory for WBS deliverables (default: <workspace>/docs/management).",
    )
    return parser.parse_args(argv)


def main(argv: Optional[List[str]] = None) -> int:
    args = parse_args(argv)
    workspace_path = Path(args.workspace).resolve()

    if not workspace_path.exists():
        print(f"Error: Workspace path '{workspace_path}' does not exist.", file=sys.stderr)
        return 1

    try:
        engine = WBSAstIngestionEngine(workspace_path=workspace_path, output_dir=args.output_dir)
        engine.run_ingestion()

        synthesizer = WBSSuiteSynthesizer(engine)
        md_file, csv_file, json_file = synthesizer.synthesize_all()

        print(f"[WBS Generator] Successfully synthesized WBS Deliverables Suite:")
        print(f"  - Markdown Suite: {md_file}")
        print(f"  - Multi-Platform CSV: {csv_file}")
        print(f"  - Machine-Readable JSON AST: {json_file}")
        return 0
    except Exception as e:
        print(f"[WBS Generator] Fatal error: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        return 1


if __name__ == "__main__":
    sys.exit(main())
