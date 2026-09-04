#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
KerML / SysML v2 AST & Semantic Unit Compiler Pipeline

Provides:
- Diagnostic and data structures: DiagnosticSeverity, SourceLocation, CompilerDiagnostic, Dimension, UnitDefinition, DomainContract
- UnitRegistry: SI units and modular domain packages (aviation, marine, rail, medical, space, industrial)
- AST node hierarchy: RootNode, PackageDecl, PartDecl, AttributeDecl, MetadataDef, MetadataFieldDef, MetadataAnnotation, Binding
- Pure-Python Tokenizer & Parser matching KerML grammar
- Pass 1 (MetadataHarvesterVisitor): Harvester for @DomainMetadata annotations and domain unit loading
- Pass 2 (SemanticBindingVisitor): Semantic unit resolution and physical dimension validation
- SysMLv2CompilerDriver: Two-pass compilation driver returning diagnostics and structured AST
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass, field
from enum import Enum
import json
import math
import re
import sys
from typing import Any, Dict, List, Optional, Sequence, Set, Tuple, Union


# ============================================================================
# 1. Diagnostic and Metrological Data Structures
# ============================================================================

class DiagnosticSeverity(Enum):
    """Severity levels for compiler diagnostics."""
    INFO = "INFO"
    WARNING = "WARNING"
    ERROR = "ERROR"


@dataclass
class SourceLocation:
    """Represents a specific line and column within a source file."""
    line: int
    column: int
    source_file: str = "<stdin>"

    def __str__(self) -> str:
        return f"{self.source_file}:{self.line}:{self.column}"

    def to_dict(self) -> Dict[str, Any]:
        return {
            "line": self.line,
            "column": self.column,
            "source_file": self.source_file,
        }


@dataclass
class CompilerDiagnostic:
    """A located compiler message (error, warning, or info)."""
    severity: DiagnosticSeverity
    message: str
    location: SourceLocation

    def __str__(self) -> str:
        return f"[{self.severity.value}] {self.location}: {self.message}"

    def to_dict(self) -> Dict[str, Any]:
        return {
            "severity": self.severity.value,
            "message": self.message,
            "location": self.location.to_dict(),
        }


@dataclass(frozen=True)
class Dimension:
    """
    Physical SI base dimensions representation:
    [M^mass * L^length * T^time * I^current * Theta^temperature]
    """
    mass: int = 0
    length: int = 0
    time: int = 0
    current: int = 0
    temperature: int = 0

    def is_dimensionless(self) -> bool:
        return (
            self.mass == 0
            and self.length == 0
            and self.time == 0
            and self.current == 0
            and self.temperature == 0
        )

    def multiply(self, other: Dimension) -> Dimension:
        return Dimension(
            mass=self.mass + other.mass,
            length=self.length + other.length,
            time=self.time + other.time,
            current=self.current + other.current,
            temperature=self.temperature + other.temperature,
        )

    def divide(self, other: Dimension) -> Dimension:
        return Dimension(
            mass=self.mass - other.mass,
            length=self.length - other.length,
            time=self.time - other.time,
            current=self.current - other.current,
            temperature=self.temperature - other.temperature,
        )

    def power(self, exponent: int) -> Dimension:
        return Dimension(
            mass=self.mass * exponent,
            length=self.length * exponent,
            time=self.time * exponent,
            current=self.current * exponent,
            temperature=self.temperature * exponent,
        )

    def to_dict(self) -> Dict[str, int]:
        return {
            "mass": self.mass,
            "length": self.length,
            "time": self.time,
            "current": self.current,
            "temperature": self.temperature,
        }


@dataclass
class UnitDefinition:
    """Defines a physical unit with its base dimension and SI scale factor."""
    name: str
    dimension: Dimension
    scale_to_si: float
    description: str = ""
    domain_package: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        return {
            "name": self.name,
            "dimension": self.dimension.to_dict(),
            "scale_to_si": self.scale_to_si,
            "description": self.description,
            "domain_package": self.domain_package,
        }


@dataclass
class DomainContract:
    """Domain contract specifying regulatory frameworks and unit packages."""
    domain_id: str
    regulatory_frameworks: List[str]
    unit_packages: List[str]

    def to_dict(self) -> Dict[str, Any]:
        return {
            "domain_id": self.domain_id,
            "regulatory_frameworks": list(self.regulatory_frameworks),
            "unit_packages": list(self.unit_packages),
        }


# ============================================================================
# 2. Canonical SI Units and Modular Domain Packages
# ============================================================================

STANDARD_SI_UNITS: Dict[str, UnitDefinition] = {
    "kg": UnitDefinition(
        name="kg",
        dimension=Dimension(mass=1),
        scale_to_si=1.0,
        description="SI base unit of mass (kilogram)",
        domain_package="SI",
    ),
    "m": UnitDefinition(
        name="m",
        dimension=Dimension(length=1),
        scale_to_si=1.0,
        description="SI base unit of length (meter)",
        domain_package="SI",
    ),
    "s": UnitDefinition(
        name="s",
        dimension=Dimension(time=1),
        scale_to_si=1.0,
        description="SI base unit of time (second)",
        domain_package="SI",
    ),
    "m_s": UnitDefinition(
        name="m_s",
        dimension=Dimension(length=1, time=-1),
        scale_to_si=1.0,
        description="SI derived unit of velocity (meters per second)",
        domain_package="SI",
    ),
    "m_s2": UnitDefinition(
        name="m_s2",
        dimension=Dimension(length=1, time=-2),
        scale_to_si=1.0,
        description="SI derived unit of acceleration (meters per second squared)",
        domain_package="SI",
    ),
    "N": UnitDefinition(
        name="N",
        dimension=Dimension(mass=1, length=1, time=-2),
        scale_to_si=1.0,
        description="SI derived unit of force (Newton: kg*m/s^2)",
        domain_package="SI",
    ),
    "J": UnitDefinition(
        name="J",
        dimension=Dimension(mass=1, length=2, time=-2),
        scale_to_si=1.0,
        description="SI derived unit of energy (Joule: N*m)",
        domain_package="SI",
    ),
    "W": UnitDefinition(
        name="W",
        dimension=Dimension(mass=1, length=2, time=-3),
        scale_to_si=1.0,
        description="SI derived unit of power (Watt: J/s)",
        domain_package="SI",
    ),
    "Pa": UnitDefinition(
        name="Pa",
        dimension=Dimension(mass=1, length=-1, time=-2),
        scale_to_si=1.0,
        description="SI derived unit of pressure (Pascal: N/m^2)",
        domain_package="SI",
    ),
    "A": UnitDefinition(
        name="A",
        dimension=Dimension(current=1),
        scale_to_si=1.0,
        description="SI base unit of electric current (Ampere)",
        domain_package="SI",
    ),
    "K": UnitDefinition(
        name="K",
        dimension=Dimension(temperature=1),
        scale_to_si=1.0,
        description="SI base unit of thermodynamic temperature (Kelvin)",
        domain_package="SI",
    ),
    "rad": UnitDefinition(
        name="rad",
        dimension=Dimension(),
        scale_to_si=1.0,
        description="SI derived unit of plane angle (radian)",
        domain_package="SI",
    ),
    "deg": UnitDefinition(
        name="deg",
        dimension=Dimension(),
        scale_to_si=math.pi / 180.0,
        description="Unit of plane angle (degree)",
        domain_package="SI",
    ),
}

AVIATION_UNITS: Dict[str, UnitDefinition] = {
    "knots": UnitDefinition(
        name="knots",
        dimension=Dimension(length=1, time=-1),
        scale_to_si=0.514444,
        description="Nautical speed unit (1 knot = 0.514444 m/s)",
        domain_package="aviation_units",
    ),
    "ft": UnitDefinition(
        name="ft",
        dimension=Dimension(length=1),
        scale_to_si=0.3048,
        description="Aviation altitude/length unit (1 ft = 0.3048 m)",
        domain_package="aviation_units",
    ),
    "slug": UnitDefinition(
        name="slug",
        dimension=Dimension(mass=1),
        scale_to_si=14.5939,
        description="Imperial mass unit (1 slug = 14.5939 kg)",
        domain_package="aviation_units",
    ),
    "ft_min": UnitDefinition(
        name="ft_min",
        dimension=Dimension(length=1, time=-1),
        scale_to_si=0.00508,
        description="Rate of climb unit (feet per minute = 0.00508 m/s)",
        domain_package="aviation_units",
    ),
    "mach": UnitDefinition(
        name="mach",
        dimension=Dimension(length=1, time=-1),
        scale_to_si=340.29,
        description="Speed of sound at sea level ISA (1 mach = 340.29 m/s)",
        domain_package="aviation_units",
    ),
    "deg_s": UnitDefinition(
        name="deg_s",
        dimension=Dimension(time=-1),
        scale_to_si=math.pi / 180.0,
        description="Angular velocity (degrees per second)",
        domain_package="aviation_units",
    ),
}

MARINE_UNITS: Dict[str, UnitDefinition] = {
    "nmi": UnitDefinition(
        name="nmi",
        dimension=Dimension(length=1),
        scale_to_si=1852.0,
        description="Nautical mile (1 nmi = 1852.0 m)",
        domain_package="marine_units",
    ),
    "knots": UnitDefinition(
        name="knots",
        dimension=Dimension(length=1, time=-1),
        scale_to_si=0.514444,
        description="Marine speed unit (1 knot = 0.514444 m/s)",
        domain_package="marine_units",
    ),
    "fathom": UnitDefinition(
        name="fathom",
        dimension=Dimension(length=1),
        scale_to_si=1.8288,
        description="Marine depth unit (1 fathom = 1.8288 m)",
        domain_package="marine_units",
    ),
    "bar": UnitDefinition(
        name="bar",
        dimension=Dimension(mass=1, length=-1, time=-2),
        scale_to_si=100000.0,
        description="Hydrostatic pressure unit (1 bar = 100000.0 Pa)",
        domain_package="marine_units",
    ),
    "m_s": UnitDefinition(
        name="m_s",
        dimension=Dimension(length=1, time=-1),
        scale_to_si=1.0,
        description="Metric velocity (meters per second)",
        domain_package="marine_units",
    ),
}

RAIL_UNITS: Dict[str, UnitDefinition] = {
    "km_h": UnitDefinition(
        name="km_h",
        dimension=Dimension(length=1, time=-1),
        scale_to_si=0.277778,
        description="Rail speed unit (kilometers per hour = 0.277778 m/s)",
        domain_package="rail_units",
    ),
    "ton": UnitDefinition(
        name="ton",
        dimension=Dimension(mass=1),
        scale_to_si=1000.0,
        description="Metric ton (1 ton = 1000.0 kg)",
        domain_package="rail_units",
    ),
    "kN": UnitDefinition(
        name="kN",
        dimension=Dimension(mass=1, length=1, time=-2),
        scale_to_si=1000.0,
        description="Tractive effort unit (kilonewtons = 1000.0 N)",
        domain_package="rail_units",
    ),
    "m_s2": UnitDefinition(
        name="m_s2",
        dimension=Dimension(length=1, time=-2),
        scale_to_si=1.0,
        description="Braking deceleration (m/s^2)",
        domain_package="rail_units",
    ),
    "mm": UnitDefinition(
        name="mm",
        dimension=Dimension(length=1),
        scale_to_si=0.001,
        description="Track gauge / mechanical tolerance (millimeter = 0.001 m)",
        domain_package="rail_units",
    ),
}

MEDICAL_UNITS: Dict[str, UnitDefinition] = {
    "mmHg": UnitDefinition(
        name="mmHg",
        dimension=Dimension(mass=1, length=-1, time=-2),
        scale_to_si=133.322,
        description="Blood pressure unit (millimeter of mercury = 133.322 Pa)",
        domain_package="medical_units",
    ),
    "ml_min": UnitDefinition(
        name="ml_min",
        dimension=Dimension(length=3, time=-1),
        scale_to_si=1.66667e-8,
        description="Infusion/flow rate unit (milliliters per minute = 1.66667e-8 m^3/s)",
        domain_package="medical_units",
    ),
    "mm": UnitDefinition(
        name="mm",
        dimension=Dimension(length=1),
        scale_to_si=0.001,
        description="Surgical precision length (millimeter = 0.001 m)",
        domain_package="medical_units",
    ),
    "N_cm": UnitDefinition(
        name="N_cm",
        dimension=Dimension(mass=1, length=2, time=-2),
        scale_to_si=0.01,
        description="Surgical torque unit (Newton-centimeter = 0.01 N*m)",
        domain_package="medical_units",
    ),
    "deg": UnitDefinition(
        name="deg",
        dimension=Dimension(),
        scale_to_si=math.pi / 180.0,
        description="Joint articulation angle (degree)",
        domain_package="medical_units",
    ),
}

SPACE_UNITS: Dict[str, UnitDefinition] = {
    "km_s": UnitDefinition(
        name="km_s",
        dimension=Dimension(length=1, time=-1),
        scale_to_si=1000.0,
        description="Orbital velocity unit (kilometers per second = 1000.0 m/s)",
        domain_package="space_units",
    ),
    "AU": UnitDefinition(
        name="AU",
        dimension=Dimension(length=1),
        scale_to_si=1.495978707e11,
        description="Astronomical Unit (1 AU = 1.495978707e11 m)",
        domain_package="space_units",
    ),
    "arcsec": UnitDefinition(
        name="arcsec",
        dimension=Dimension(),
        scale_to_si=4.8481368e-6,
        description="Attitude determination precision (arcsecond = 4.8481368e-6 rad)",
        domain_package="space_units",
    ),
    "uW": UnitDefinition(
        name="uW",
        dimension=Dimension(mass=1, length=2, time=-3),
        scale_to_si=1e-6,
        description="Deep space payload power (microwatt = 1e-6 W)",
        domain_package="space_units",
    ),
    "N_s": UnitDefinition(
        name="N_s",
        dimension=Dimension(mass=1, length=1, time=-1),
        scale_to_si=1.0,
        description="Thruster total impulse (Newton-second = 1.0 N*s)",
        domain_package="space_units",
    ),
}

INDUSTRIAL_UNITS: Dict[str, UnitDefinition] = {
    "rpm": UnitDefinition(
        name="rpm",
        dimension=Dimension(time=-1),
        scale_to_si=0.10472,
        description="Rotational speed (revolutions per minute = 0.10472 rad/s)",
        domain_package="industrial_units",
    ),
    "Nm": UnitDefinition(
        name="Nm",
        dimension=Dimension(mass=1, length=2, time=-2),
        scale_to_si=1.0,
        description="Actuator torque (Newton-meter = 1.0 N*m)",
        domain_package="industrial_units",
    ),
    "m_min": UnitDefinition(
        name="m_min",
        dimension=Dimension(length=1, time=-1),
        scale_to_si=0.0166667,
        description="Conveyor / feed velocity (meters per minute = 0.0166667 m/s)",
        domain_package="industrial_units",
    ),
    "deg_s": UnitDefinition(
        name="deg_s",
        dimension=Dimension(time=-1),
        scale_to_si=math.pi / 180.0,
        description="Robotic arm angular velocity (degrees per second)",
        domain_package="industrial_units",
    ),
}

DOMAIN_PACKAGES: Dict[str, Dict[str, UnitDefinition]] = {
    "aviation_units": AVIATION_UNITS,
    "marine_units": MARINE_UNITS,
    "rail_units": RAIL_UNITS,
    "medical_units": MEDICAL_UNITS,
    "space_units": SPACE_UNITS,
    "industrial_units": INDUSTRIAL_UNITS,
}

DOMAIN_CONTRACTS: Dict[str, DomainContract] = {
    "aviation": DomainContract(
        domain_id="aviation",
        regulatory_frameworks=["DO-178C", "DO-254", "ARP4754A", "DO-365B", "STANAG 4586", "SORA v2.5"],
        unit_packages=["aviation_units"],
    ),
    "marine": DomainContract(
        domain_id="marine",
        regulatory_frameworks=["IMO", "SOLAS", "IEC 61162"],
        unit_packages=["marine_units"],
    ),
    "rail": DomainContract(
        domain_id="rail",
        regulatory_frameworks=["EN 50128", "EN 50126", "IEEE 1474"],
        unit_packages=["rail_units"],
    ),
    "medical": DomainContract(
        domain_id="medical",
        regulatory_frameworks=["IEC 62304", "ISO 14971", "FDA 21 CFR 820"],
        unit_packages=["medical_units"],
    ),
    "space": DomainContract(
        domain_id="space",
        regulatory_frameworks=["ECSS-E-ST-40C", "ECSS-Q-ST-80C", "NASA-STD-8739.8"],
        unit_packages=["space_units"],
    ),
    "industrial": DomainContract(
        domain_id="industrial",
        regulatory_frameworks=["IEC 61508", "ISO 13849", "IEC 62443"],
        unit_packages=["industrial_units"],
    ),
}


class UnitRegistry:
    """Registry maintaining available SI and modular domain physical units."""

    def __init__(self, include_standard_si: bool = True) -> None:
        self._units: Dict[str, UnitDefinition] = {}
        self._loaded_packages: Set[str] = set()
        self._loaded_domains: Set[str] = set()
        if include_standard_si:
            self.load_standard_si()

    def load_standard_si(self) -> None:
        """Loads default SI standard units."""
        for unit_def in STANDARD_SI_UNITS.values():
            self.register_unit(unit_def, package_name="SI")
        self._loaded_packages.add("SI")

    def register_unit(self, unit: UnitDefinition, package_name: Optional[str] = None) -> None:
        """Registers a unit definition under its simple name and qualified package name."""
        self._units[unit.name] = unit
        pkg = package_name or unit.domain_package
        if pkg:
            self._units[f"{pkg}::{unit.name}"] = unit
            self._loaded_packages.add(pkg)

    def load_package(self, package_name: str) -> bool:
        """Loads all unit definitions from a specified domain package."""
        if package_name in DOMAIN_PACKAGES:
            for unit_def in DOMAIN_PACKAGES[package_name].values():
                self.register_unit(unit_def, package_name=package_name)
            self._loaded_packages.add(package_name)
            return True
        return False

    def load_domain(self, domain_id: str) -> bool:
        """Resolves a domain identifier and loads all associated unit packages."""
        dom_key = domain_id.lower().strip()
        loaded = False
        if dom_key in DOMAIN_CONTRACTS:
            contract = DOMAIN_CONTRACTS[dom_key]
            for pkg in contract.unit_packages:
                if self.load_package(pkg):
                    loaded = True
            self._loaded_domains.add(dom_key)
            return loaded

        # Fallback: check if domain_id directly matches a unit package
        if dom_key in DOMAIN_PACKAGES:
            loaded = self.load_package(dom_key)
            if loaded:
                self._loaded_domains.add(dom_key)
            return loaded

        pkg_suffix = f"{dom_key}_units"
        if pkg_suffix in DOMAIN_PACKAGES:
            loaded = self.load_package(pkg_suffix)
            if loaded:
                self._loaded_domains.add(dom_key)
            return loaded

        return False

    def resolve(self, unit_ref: str) -> Optional[UnitDefinition]:
        """
        Resolves a unit reference (either simple or qualified).
        Returns the UnitDefinition or None if unresolved.
        """
        if not unit_ref:
            return None

        # Direct name lookup
        if unit_ref in self._units:
            return self._units[unit_ref]

        # Qualified name lookup e.g. "aviation_units::knots" or "SI::m" or "ISQ::m"
        if "::" in unit_ref:
            pkg, base_name = unit_ref.rsplit("::", 1)
            if base_name in self._units:
                return self._units[base_name]

            # Try loading package on demand if known
            if pkg in DOMAIN_PACKAGES:
                self.load_package(pkg)
                if unit_ref in self._units:
                    return self._units[unit_ref]
                if base_name in self._units:
                    return self._units[base_name]

        return None

    def is_registered(self, unit_ref: str) -> bool:
        return self.resolve(unit_ref) is not None

    def get_loaded_packages(self) -> List[str]:
        return sorted(list(self._loaded_packages))

    def get_loaded_domains(self) -> List[str]:
        return sorted(list(self._loaded_domains))


# ============================================================================
# 3. AST Data Models
# ============================================================================

@dataclass
class ASTNode:
    """Base AST node containing source location."""
    location: SourceLocation

    def to_dict(self) -> Dict[str, Any]:
        return {
            "type": self.__class__.__name__,
            "location": self.location.to_dict(),
        }


@dataclass
class MetadataFieldDef(ASTNode):
    """Field definition within a metadata def block (e.g., attribute domainId : String;)."""
    name: str
    type_name: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        res = super().to_dict()
        res.update({
            "name": self.name,
            "type_name": self.type_name,
        })
        return res


@dataclass
class MetadataDef(ASTNode):
    """Metadata definition block (e.g., metadata def DomainMetadata { ... })."""
    name: str
    fields: List[MetadataFieldDef] = field(default_factory=list)

    def to_dict(self) -> Dict[str, Any]:
        res = super().to_dict()
        res.update({
            "name": self.name,
            "fields": [f.to_dict() for f in self.fields],
        })
        return res


@dataclass
class Binding(ASTNode):
    """Key-value binding within an annotation (e.g., domainId = "aviation")."""
    key: str
    value: Any

    def to_dict(self) -> Dict[str, Any]:
        res = super().to_dict()
        res.update({
            "key": self.key,
            "value": self._serialize_val(self.value),
        })
        return res

    def _serialize_val(self, val: Any) -> Any:
        if isinstance(val, list):
            return [self._serialize_val(v) for v in val]
        if isinstance(val, ASTNode):
            return val.to_dict()
        return val


@dataclass
class MetadataAnnotation(ASTNode):
    """Annotation attached to an element (e.g., @DomainMetadata { domainId = "aviation"; })."""
    name: str
    bindings: List[Binding] = field(default_factory=list)

    @property
    def bindings_dict(self) -> Dict[str, Any]:
        return {b.key: b.value for b in self.bindings}

    def get_binding_value(self, key: str) -> Optional[Any]:
        for b in self.bindings:
            if b.key == key:
                return b.value
        return None

    def to_dict(self) -> Dict[str, Any]:
        res = super().to_dict()
        res.update({
            "name": self.name,
            "bindings": [b.to_dict() for b in self.bindings],
            "bindings_dict": {b.key: b._serialize_val(b.value) for b in self.bindings},
        })
        return res


@dataclass
class AttributeDecl(ASTNode):
    """Attribute declaration node (e.g., attribute airspeed : Real [knots];)."""
    name: str
    type_name: Optional[str] = None
    default_value: Optional[Any] = None
    unit_ref: Optional[str] = None
    annotations: List[MetadataAnnotation] = field(default_factory=list)
    resolved_unit: Optional[UnitDefinition] = None

    def to_dict(self) -> Dict[str, Any]:
        res = super().to_dict()
        res.update({
            "name": self.name,
            "type_name": self.type_name,
            "default_value": self.default_value,
            "unit_ref": self.unit_ref,
            "annotations": [a.to_dict() for a in self.annotations],
            "resolved_unit": self.resolved_unit.to_dict() if self.resolved_unit else None,
        })
        return res


@dataclass
class PartDecl(ASTNode):
    """Part declaration node (e.g., part def FlightControlComputer { ... })."""
    name: str
    is_def: bool = False
    elements: List[ASTNode] = field(default_factory=list)
    annotations: List[MetadataAnnotation] = field(default_factory=list)

    def to_dict(self) -> Dict[str, Any]:
        res = super().to_dict()
        res.update({
            "name": self.name,
            "is_def": self.is_def,
            "elements": [el.to_dict() for el in self.elements],
            "annotations": [a.to_dict() for a in self.annotations],
        })
        return res


@dataclass
class PackageDecl(ASTNode):
    """Package declaration node (e.g., package AircraftSystem { ... })."""
    name: str
    elements: List[ASTNode] = field(default_factory=list)
    annotations: List[MetadataAnnotation] = field(default_factory=list)

    def to_dict(self) -> Dict[str, Any]:
        res = super().to_dict()
        res.update({
            "name": self.name,
            "elements": [el.to_dict() for el in self.elements],
            "annotations": [a.to_dict() for a in self.annotations],
        })
        return res


@dataclass
class RootNode(ASTNode):
    """Root element of a KerML / SysML v2 source unit."""
    elements: List[ASTNode] = field(default_factory=list)

    def to_dict(self) -> Dict[str, Any]:
        res = super().to_dict()
        res.update({
            "elements": [el.to_dict() for el in self.elements],
        })
        return res


# ============================================================================
# 4. Pure-Python Deterministic Tokenizer & Parser
# ============================================================================

class LexerError(Exception):
    def __init__(self, message: str, line: int, column: int):
        super().__init__(f"{message} (line {line}, col {column})")
        self.message = message
        self.line = line
        self.column = column


@dataclass
class Token:
    type: str
    value: Any
    line: int
    column: int
    source_file: str = "<stdin>"

    @property
    def location(self) -> SourceLocation:
        return SourceLocation(line=self.line, column=self.column, source_file=self.source_file)


class KerMLTokenizer:
    """Deterministic, high-performance lexical analyzer for KerML."""

    KEYWORDS: Set[str] = {
        "metadata", "def", "package", "part", "attribute", "true", "false"
    }

    def __init__(self, source_code: str, source_file: str = "<stdin>") -> None:
        self.src = source_code
        self.len = len(source_code)
        self.source_file = source_file
        self.pos = 0
        self.line = 1
        self.col = 1

    def tokenize(self) -> List[Token]:
        tokens: List[Token] = []
        while self.pos < self.len:
            ch = self.src[self.pos]

            # Whitespace
            if ch in " \t\r\n":
                if ch == "\n":
                    self.line += 1
                    self.col = 1
                else:
                    self.col += 1
                self.pos += 1
                continue

            # Line comment
            if ch == "/" and self.pos + 1 < self.len and self.src[self.pos + 1] == "/":
                self._skip_line_comment()
                continue

            # Block comment
            if ch == "/" and self.pos + 1 < self.len and self.src[self.pos + 1] == "*":
                self._skip_block_comment()
                continue

            # Double colon ::
            if ch == ":" and self.pos + 1 < self.len and self.src[self.pos + 1] == ":":
                tok = Token("DOUBLE_COLON", "::", self.line, self.col, self.source_file)
                self.pos += 2
                self.col += 2
                tokens.append(tok)
                continue

            # Single-character delimiters and operators
            if ch in "@{}()[];,=:":
                type_map = {
                    "@": "AT",
                    "{": "LBRACE",
                    "}": "RBRACE",
                    "(": "LPAREN",
                    ")": "RPAREN",
                    "[": "LBRACKET",
                    "]": "RBRACKET",
                    ";": "SEMI",
                    ",": "COMMA",
                    "=": "EQUALS",
                    ":": "COLON",
                }
                tok = Token(type_map[ch], ch, self.line, self.col, self.source_file)
                self.pos += 1
                self.col += 1
                tokens.append(tok)
                continue

            # String literal
            if ch == '"':
                tokens.append(self._read_string())
                continue

            # Numeric literal (int or float)
            if ch.isdigit():
                tokens.append(self._read_number())
                continue

            # Identifiers and keywords
            if ch.isalpha() or ch == "_":
                tokens.append(self._read_identifier())
                continue

            # Unknown character
            raise LexerError(f"Unexpected character '{ch}'", self.line, self.col)

        tokens.append(Token("EOF", None, self.line, self.col, self.source_file))
        return tokens

    def _skip_line_comment(self) -> None:
        self.pos += 2
        self.col += 2
        while self.pos < self.len and self.src[self.pos] != "\n":
            self.pos += 1
            self.col += 1

    def _skip_block_comment(self) -> None:
        start_line = self.line
        start_col = self.col
        self.pos += 2
        self.col += 2
        while self.pos + 1 < self.len:
            if self.src[self.pos] == "*" and self.src[self.pos + 1] == "/":
                self.pos += 2
                self.col += 2
                return
            if self.src[self.pos] == "\n":
                self.line += 1
                self.col = 1
            else:
                self.col += 1
            self.pos += 1

        raise LexerError("Unterminated block comment", start_line, start_col)

    def _read_string(self) -> Token:
        start_line = self.line
        start_col = self.col
        self.pos += 1  # skip opening "
        self.col += 1
        chars: List[str] = []

        while self.pos < self.len:
            ch = self.src[self.pos]
            if ch == '"':
                self.pos += 1
                self.col += 1
                return Token("STRING", "".join(chars), start_line, start_col, self.source_file)
            elif ch == "\\":
                self.pos += 1
                self.col += 1
                if self.pos >= self.len:
                    break
                esc = self.src[self.pos]
                esc_map = {"n": "\n", "t": "\t", "r": "\r", "\\": "\\", '"': '"', "'": "'"}
                chars.append(esc_map.get(esc, esc))
                self.pos += 1
                self.col += 1
            elif ch == "\n":
                chars.append(ch)
                self.line += 1
                self.col = 1
                self.pos += 1
            else:
                chars.append(ch)
                self.pos += 1
                self.col += 1

        raise LexerError("Unterminated string literal", start_line, start_col)

    def _read_number(self) -> Token:
        start_line = self.line
        start_col = self.col
        start_pos = self.pos
        is_float = False

        while self.pos < self.len and self.src[self.pos].isdigit():
            self.pos += 1
            self.col += 1

        # Check for decimal part
        if (
            self.pos < self.len
            and self.src[self.pos] == "."
            and self.pos + 1 < self.len
            and self.src[self.pos + 1].isdigit()
        ):
            is_float = True
            self.pos += 1
            self.col += 1
            while self.pos < self.len and self.src[self.pos].isdigit():
                self.pos += 1
                self.col += 1

        # Check for scientific exponent [eE][+-]?[0-9]+
        if self.pos < self.len and self.src[self.pos] in "eE":
            is_float = True
            self.pos += 1
            self.col += 1
            if self.pos < self.len and self.src[self.pos] in "+-":
                self.pos += 1
                self.col += 1
            while self.pos < self.len and self.src[self.pos].isdigit():
                self.pos += 1
                self.col += 1

        num_str = self.src[start_pos:self.pos]
        val: Union[int, float] = float(num_str) if is_float else int(num_str)
        tok_type = "FLOAT" if is_float else "INT"
        return Token(tok_type, val, start_line, start_col, self.source_file)

    def _read_identifier(self) -> Token:
        start_line = self.line
        start_col = self.col
        start_pos = self.pos

        while self.pos < self.len and (self.src[self.pos].isalnum() or self.src[self.pos] == "_"):
            self.pos += 1
            self.col += 1

        text = self.src[start_pos:self.pos]
        if text == "true":
            return Token("BOOLEAN", True, start_line, start_col, self.source_file)
        elif text == "false":
            return Token("BOOLEAN", False, start_line, start_col, self.source_file)
        elif text in self.KEYWORDS:
            return Token("KEYWORD", text, start_line, start_col, self.source_file)
        else:
            return Token("ID", text, start_line, start_col, self.source_file)


class KerMLParser:
    """Recursive descent AST parser for KerML / SysML v2 grammar."""

    def __init__(self, tokens: Sequence[Token], source_file: str = "<stdin>") -> None:
        self.tokens = tokens
        self.pos = 0
        self.source_file = source_file
        self.diagnostics: List[CompilerDiagnostic] = []

    def parse(self) -> Tuple[Optional[RootNode], List[CompilerDiagnostic]]:
        root_loc = self._current_location()
        elements: List[ASTNode] = []

        while not self._is_at_end():
            el = self._parse_element()
            if el is not None:
                elements.append(el)
            else:
                if not self._is_at_end():
                    self._error(f"Unexpected token '{self._peek().value or self._peek().type}'")
                    self._advance()

        root = RootNode(location=root_loc, elements=elements)
        return root, self.diagnostics

    # ------------------------------------------------------------------------
    # Grammar Rules
    # ------------------------------------------------------------------------

    def _parse_element(self) -> Optional[ASTNode]:
        annotations = self._parse_annotations()

        if self._is_at_end():
            if annotations:
                self._error("Dangling annotation at end of file")
            return None

        tok = self._peek()

        if tok.type == "KEYWORD":
            if tok.value == "metadata":
                return self._parse_metadata_def(annotations)
            elif tok.value == "package":
                return self._parse_package_decl(annotations)
            elif tok.value == "part":
                return self._parse_part_decl(annotations)
            elif tok.value == "attribute":
                return self._parse_attribute_decl(annotations)

        # Fallback for unexpected token
        if annotations:
            self._error(f"Annotations must precede a package, part, attribute, or metadata definition, found '{tok.value or tok.type}'")
        return None

    def _parse_annotations(self) -> List[MetadataAnnotation]:
        annotations: List[MetadataAnnotation] = []
        while self._match_type("AT"):
            annot = self._parse_metadata_annotation()
            if annot is not None:
                annotations.append(annot)
        return annotations

    def _parse_metadata_annotation(self) -> Optional[MetadataAnnotation]:
        # '@' was already consumed by _parse_annotations
        loc = self._previous_location()
        name = self._parse_qualified_name()
        if not name:
            self._error("Expected qualified name after '@'")
            return None

        bindings: List[Binding] = []

        if self._match_type("LBRACE"):
            # '{' (binding ';')* '}'
            while not self._check_type("RBRACE") and not self._is_at_end():
                b = self._parse_binding()
                if b is not None:
                    bindings.append(b)
                if self._check_type("SEMI"):
                    self._advance()
            self._consume("RBRACE", "Expected '}' after annotation bindings")
        elif self._match_type("LPAREN"):
            # '(' (binding (',' binding)*)? ')'
            while not self._check_type("RPAREN") and not self._is_at_end():
                b = self._parse_binding()
                if b is not None:
                    bindings.append(b)
                if self._check_type("COMMA"):
                    self._advance()
                elif not self._check_type("RPAREN"):
                    # allow optional semicolons in parens or report
                    if self._check_type("SEMI"):
                        self._advance()
                    else:
                        break
            self._consume("RPAREN", "Expected ')' after annotation bindings")
        else:
            self._error("Expected '{' or '(' after annotation name")
            return None

        return MetadataAnnotation(location=loc, name=name, bindings=bindings)

    def _parse_binding(self) -> Optional[Binding]:
        loc = self._current_location()
        if not self._check_type("ID") and not (self._check_type("KEYWORD")):
            self._error("Expected binding identifier")
            return None

        key_tok = self._advance()
        key = str(key_tok.value)

        if not self._consume("EQUALS", f"Expected '=' after binding key '{key}'"):
            return None

        val = self._parse_expression()
        return Binding(location=loc, key=key, value=val)

    def _parse_metadata_def(self, annotations: List[MetadataAnnotation]) -> Optional[MetadataDef]:
        loc = self._current_location()
        self._consume_keyword("metadata", "Expected 'metadata'")
        self._consume_keyword("def", "Expected 'def' after 'metadata'")

        name_tok = self._consume("ID", "Expected metadata definition identifier")
        if not name_tok:
            return None
        name = str(name_tok.value)

        if not self._consume("LBRACE", "Expected '{' in metadata definition"):
            return None

        fields: List[MetadataFieldDef] = []
        while not self._check_type("RBRACE") and not self._is_at_end():
            f = self._parse_metadata_field_def()
            if f is not None:
                fields.append(f)
            else:
                self._advance()

        self._consume("RBRACE", "Expected '}' closing metadata definition")
        return MetadataDef(location=loc, name=name, fields=fields)

    def _parse_metadata_field_def(self) -> Optional[MetadataFieldDef]:
        loc = self._current_location()
        if not self._match_keyword("attribute"):
            self._error("Expected 'attribute' in metadata field definition")
            return None

        name_tok = self._consume("ID", "Expected attribute identifier")
        if not name_tok:
            return None
        name = str(name_tok.value)

        type_name: Optional[str] = None
        if self._match_type("COLON"):
            type_name = self._parse_qualified_name()

        self._consume("SEMI", "Expected ';' terminating metadata field definition")
        return MetadataFieldDef(location=loc, name=name, type_name=type_name)

    def _parse_package_decl(self, annotations: List[MetadataAnnotation]) -> Optional[PackageDecl]:
        loc = self._current_location()
        self._consume_keyword("package", "Expected 'package'")

        name = self._parse_qualified_name()
        if not name:
            self._error("Expected package name")
            return None

        if not self._consume("LBRACE", "Expected '{' after package declaration"):
            return None

        elements: List[ASTNode] = []
        while not self._check_type("RBRACE") and not self._is_at_end():
            el = self._parse_element()
            if el is not None:
                elements.append(el)
            else:
                if not self._is_at_end() and not self._check_type("RBRACE"):
                    self._advance()

        self._consume("RBRACE", "Expected '}' closing package declaration")
        return PackageDecl(location=loc, name=name, elements=elements, annotations=annotations)

    def _parse_part_decl(self, annotations: List[MetadataAnnotation]) -> Optional[PartDecl]:
        loc = self._current_location()
        self._consume_keyword("part", "Expected 'part'")

        is_def = False
        if self._match_keyword("def"):
            is_def = True

        name_tok = self._consume("ID", "Expected part identifier")
        if not name_tok:
            return None
        name = str(name_tok.value)

        if not self._consume("LBRACE", "Expected '{' after part declaration"):
            return None

        elements: List[ASTNode] = []
        while not self._check_type("RBRACE") and not self._is_at_end():
            el = self._parse_element()
            if el is not None:
                elements.append(el)
            else:
                if not self._is_at_end() and not self._check_type("RBRACE"):
                    self._advance()

        self._consume("RBRACE", "Expected '}' closing part declaration")
        return PartDecl(location=loc, name=name, is_def=is_def, elements=elements, annotations=annotations)

    def _parse_attribute_decl(self, annotations: List[MetadataAnnotation]) -> Optional[AttributeDecl]:
        loc = self._current_location()
        self._consume_keyword("attribute", "Expected 'attribute'")

        name_tok = self._consume("ID", "Expected attribute identifier")
        if not name_tok:
            return None
        name = str(name_tok.value)

        type_name: Optional[str] = None
        if self._match_type("COLON"):
            type_name = self._parse_qualified_name()

        default_value: Optional[Any] = None
        if self._match_type("EQUALS"):
            default_value = self._parse_expression()

        unit_ref: Optional[str] = None
        if self._match_type("LBRACKET"):
            unit_ref = self._parse_qualified_name()
            self._consume("RBRACKET", "Expected ']' after unit reference")

        self._consume("SEMI", "Expected ';' terminating attribute declaration")
        return AttributeDecl(
            location=loc,
            name=name,
            type_name=type_name,
            default_value=default_value,
            unit_ref=unit_ref,
            annotations=annotations,
        )

    def _parse_expression(self) -> Any:
        tok = self._peek()

        # Literals
        if tok.type in ("STRING", "INT", "FLOAT", "BOOLEAN"):
            self._advance()
            return tok.value

        # Collection literal in () or []
        if tok.type in ("LPAREN", "LBRACKET"):
            return self._parse_collection_literal()

        # Qualified name or identifier
        if tok.type == "ID":
            return self._parse_qualified_name()

        # Fallback
        self._error(f"Expected expression, found '{tok.value or tok.type}'")
        self._advance()
        return None

    def _parse_collection_literal(self) -> List[Any]:
        close_type = "RPAREN" if self._peek().type == "LPAREN" else "RBRACKET"
        self._advance()  # consume opening ( or [

        items: List[Any] = []
        while not self._check_type(close_type) and not self._is_at_end():
            expr = self._parse_expression()
            items.append(expr)
            if self._check_type("COMMA"):
                self._advance()
            elif not self._check_type(close_type):
                break

        self._consume(close_type, f"Expected '{close_type}' closing collection literal")
        return items

    def _parse_qualified_name(self) -> str:
        if not self._check_type("ID"):
            return ""

        parts: List[str] = [str(self._advance().value)]
        while self._match_type("DOUBLE_COLON"):
            if self._check_type("ID"):
                parts.append(str(self._advance().value))
            else:
                self._error("Expected identifier after '::'")
                break

        return "::".join(parts)

    # ------------------------------------------------------------------------
    # Parser Utilities
    # ------------------------------------------------------------------------

    def _peek(self) -> Token:
        if self.pos < len(self.tokens):
            return self.tokens[self.pos]
        return self.tokens[-1]

    def _previous(self) -> Token:
        return self.tokens[max(0, self.pos - 1)]

    def _is_at_end(self) -> bool:
        return self._peek().type == "EOF"

    def _check_type(self, type_: str) -> bool:
        if self._is_at_end():
            return False
        return self._peek().type == type_

    def _match_type(self, type_: str) -> bool:
        if self._check_type(type_):
            self._advance()
            return True
        return False

    def _match_keyword(self, kw: str) -> bool:
        if not self._is_at_end() and self._peek().type == "KEYWORD" and self._peek().value == kw:
            self._advance()
            return True
        return False

    def _consume(self, type_: str, message: str) -> Optional[Token]:
        if self._check_type(type_):
            return self._advance()
        self._error(message)
        return None

    def _consume_keyword(self, kw: str, message: str) -> Optional[Token]:
        if not self._is_at_end() and self._peek().type == "KEYWORD" and self._peek().value == kw:
            return self._advance()
        self._error(message)
        return None

    def _advance(self) -> Token:
        if not self._is_at_end():
            self.pos += 1
        return self._previous()

    def _current_location(self) -> SourceLocation:
        return self._peek().location

    def _previous_location(self) -> SourceLocation:
        return self._previous().location

    def _error(self, message: str) -> None:
        loc = self._current_location()
        self.diagnostics.append(
            CompilerDiagnostic(
                severity=DiagnosticSeverity.ERROR,
                message=message,
                location=loc,
            )
        )


# ============================================================================
# 5. AST Visitor Infrastructure & Two-Pass Compiler Visitors
# ============================================================================

class ASTVisitor:
    """Generic AST Visitor base class."""

    def visit(self, node: Optional[ASTNode]) -> Any:
        if node is None:
            return None
        method_name = f"visit_{node.__class__.__name__}"
        visitor = getattr(self, method_name, self.generic_visit)
        return visitor(node)

    def generic_visit(self, node: ASTNode) -> Any:
        for val in node.__dict__.values():
            if isinstance(val, ASTNode):
                self.visit(val)
            elif isinstance(val, list):
                for item in val:
                    if isinstance(item, ASTNode):
                        self.visit(item)


class MetadataHarvesterVisitor(ASTVisitor):
    """
    Pass 1: Metadata Harvester Visitor.
    Scans @DomainMetadata annotations, extracts domainId and activeFrameworks,
    and loads associated domain unit packages into the UnitRegistry.
    Emits an ERROR diagnostic if @DomainMetadata is missing mandatory domainId.
    """

    def __init__(self, unit_registry: UnitRegistry) -> None:
        self.unit_registry = unit_registry
        self.diagnostics: List[CompilerDiagnostic] = []
        self.harvested_domains: List[Dict[str, Any]] = []

    def harvest(self, root: ASTNode) -> None:
        self.visit(root)

    def visit_MetadataAnnotation(self, node: MetadataAnnotation) -> None:
        if node.name.endswith("DomainMetadata") or node.name == "DomainMetadata":
            bindings = node.bindings_dict
            domain_id_val = bindings.get("domainId") or bindings.get("domain_id")

            if domain_id_val is None or (isinstance(domain_id_val, str) and not domain_id_val.strip()):
                self.diagnostics.append(
                    CompilerDiagnostic(
                        severity=DiagnosticSeverity.ERROR,
                        message="@DomainMetadata annotation is missing mandatory 'domainId' attribute.",
                        location=node.location,
                    )
                )
            else:
                domain_id_str = str(domain_id_val).strip("\"'")
                raw_frameworks = bindings.get("activeFrameworks") or bindings.get("active_frameworks") or []
                if isinstance(raw_frameworks, str):
                    frameworks = [raw_frameworks.strip("\"'")]
                elif isinstance(raw_frameworks, (list, tuple)):
                    frameworks = [str(f).strip("\"'") for f in raw_frameworks]
                else:
                    frameworks = []

                # Load domain units
                loaded = self.unit_registry.load_domain(domain_id_str)

                harvest_entry = {
                    "domain_id": domain_id_str,
                    "active_frameworks": frameworks,
                    "loaded_successfully": loaded,
                    "location": node.location.to_dict(),
                }
                self.harvested_domains.append(harvest_entry)

        self.generic_visit(node)

    def visit_PackageDecl(self, node: PackageDecl) -> None:
        for a in node.annotations:
            self.visit(a)
        for el in node.elements:
            self.visit(el)

    def visit_PartDecl(self, node: PartDecl) -> None:
        for a in node.annotations:
            self.visit(a)
        for el in node.elements:
            self.visit(el)

    def visit_AttributeDecl(self, node: AttributeDecl) -> None:
        for a in node.annotations:
            self.visit(a)


class SemanticBindingVisitor(ASTVisitor):
    """
    Pass 2: Semantic Unit Binding Visitor.
    Visits packages, parts, and attributes. For each attribute with a unit reference,
    verifies that the unit resolves in the UnitRegistry. If not, emits a located ERROR.
    """

    def __init__(self, unit_registry: UnitRegistry) -> None:
        self.unit_registry = unit_registry
        self.diagnostics: List[CompilerDiagnostic] = []
        self.resolved_attributes: List[AttributeDecl] = []

    def bind(self, root: ASTNode) -> None:
        self.visit(root)

    def visit_AttributeDecl(self, node: AttributeDecl) -> None:
        if node.unit_ref is not None:
            unit_sym = node.unit_ref
            unit_def = self.unit_registry.resolve(unit_sym)
            if unit_def is None:
                msg = (
                    f"Unresolved physical unit '[{unit_sym}]' declared on attribute '{node.name}'. "
                    f"Verify that the appropriate domain library is imported."
                )
                self.diagnostics.append(
                    CompilerDiagnostic(
                        severity=DiagnosticSeverity.ERROR,
                        message=msg,
                        location=node.location,
                    )
                )
            else:
                node.resolved_unit = unit_def
                self.resolved_attributes.append(node)

        self.generic_visit(node)


# ============================================================================
# 6. SysML v2 Compiler Driver
# ============================================================================

class SysMLv2CompilerDriver:
    """
    Two-pass compiler driver executing full syntactic and semantic compilation
    over KerML / SysML v2 source specifications.
    """

    def __init__(self, unit_registry: Optional[UnitRegistry] = None) -> None:
        self.unit_registry = unit_registry if unit_registry is not None else UnitRegistry()

    def compile(
        self, source_code: str, source_file: str = "<stdin>"
    ) -> Tuple[List[CompilerDiagnostic], Dict[str, Any]]:
        """
        Executes complete compilation:
        1. Tokenizes and parses KerML source code into AST.
        2. Pass 1: Harvests @DomainMetadata and imports domain units.
        3. Pass 2: Performs semantic unit binding on attributes.
        Returns a tuple of (diagnostics_list, ast_dictionary).
        """
        diagnostics: List[CompilerDiagnostic] = []

        # Step 1: Lexical analysis
        try:
            tokenizer = KerMLTokenizer(source_code, source_file=source_file)
            tokens = tokenizer.tokenize()
        except LexerError as e:
            diag = CompilerDiagnostic(
                severity=DiagnosticSeverity.ERROR,
                message=str(e),
                location=SourceLocation(line=e.line, column=e.column, source_file=source_file),
            )
            return [diag], {}

        # Step 2: Parsing
        parser = KerMLParser(tokens, source_file=source_file)
        root_node, parse_diags = parser.parse()
        diagnostics.extend(parse_diags)

        if root_node is None:
            return diagnostics, {}

        # Step 3: Pass 1 - Metadata Harvesting
        harvester = MetadataHarvesterVisitor(unit_registry=self.unit_registry)
        harvester.harvest(root_node)
        diagnostics.extend(harvester.diagnostics)

        # Step 4: Pass 2 - Semantic Unit Binding
        binder = SemanticBindingVisitor(unit_registry=self.unit_registry)
        binder.bind(root_node)
        diagnostics.extend(binder.diagnostics)

        # Build output AST dictionary
        ast_dict = root_node.to_dict()
        ast_dict["metadata"] = {
            "harvested_domains": harvester.harvested_domains,
            "loaded_unit_packages": self.unit_registry.get_loaded_packages(),
            "source_file": source_file,
            "error_count": sum(1 for d in diagnostics if d.severity == DiagnosticSeverity.ERROR),
            "warning_count": sum(1 for d in diagnostics if d.severity == DiagnosticSeverity.WARNING),
        }

        return diagnostics, ast_dict

    def compile_file(self, file_path: str) -> Tuple[List[CompilerDiagnostic], Dict[str, Any]]:
        """Compiles a KerML source file from disk."""
        with open(file_path, "r", encoding="utf-8") as f:
            content = f.read()
        return self.compile(content, source_file=file_path)


# ============================================================================
# 7. Command-Line Interface
# ============================================================================

def main() -> int:
    parser = argparse.ArgumentParser(description="KerML / SysML v2 AST & Semantic Unit Compiler")
    parser.add_argument("input_file", nargs="?", default=None, help="Path to KerML/SysML v2 source file")
    parser.add_argument("--json", action="store_true", help="Output compiled AST as JSON")
    parser.add_argument("--domain", type=str, default=None, help="Explicit domain override to preload")
    args = parser.parse_args()

    registry = UnitRegistry()
    if args.domain:
        registry.load_domain(args.domain)

    driver = SysMLv2CompilerDriver(unit_registry=registry)

    if args.input_file:
        diagnostics, ast_dict = driver.compile_file(args.input_file)
    else:
        src = sys.stdin.read()
        diagnostics, ast_dict = driver.compile(src, source_file="<stdin>")

    has_errors = False
    for diag in diagnostics:
        if diag.severity == DiagnosticSeverity.ERROR:
            has_errors = True
        sys.stderr.write(f"{diag}\n")

    if args.json:
        print(json.dumps(ast_dict, indent=2))

    return 1 if has_errors else 0


if __name__ == "__main__":
    sys.exit(main())
