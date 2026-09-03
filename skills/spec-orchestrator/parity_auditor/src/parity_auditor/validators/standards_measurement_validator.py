r"""
Standards & SI 7-Dimensional Parameter Metrology Quality Gate (Gate 25).

Enforces:
1. SI 7-Dimensional Exponent Vector Profiles in \mathbb{Z}^7:
   D(Q) = [d_L, d_M, d_T, d_I, d_\Theta, d_N, d_J] \in \mathbb{Z}^7
   Where:
     L = length (m)
     M = mass (kg)
     T = time (s)
     I = electric current (A)
     \Theta = thermodynamic temperature (K)
     N = amount of substance (mol)
     J = luminous intensity (cd)
2. Theorem 3 (Dimensional Homogeneity):
   For every connector c = (p_{src}, p_{dst}), assert D(e_{src}) = D(e_{dst}).
3. Nyquist-Shannon Sampling Frequency Integrity:
   f_{sample} \ge 2 \cdot f_{max}
4. OMG UAF Std-Tx (Standards Taxonomy) Lattice Validation:
   - SDO issuing bodies (RTCA, SAE, ISO, IEC, JARUS, ASTM, IEEE)
   - Standard baselines (DO-178C, DO-254, ARP4754A, JARUS_SORA_v2.5, ISO_26262, IEC_62304)
   - Monotonic assurance level hierarchies (DAL-A..DAL-E, ASIL-A..ASIL-D, SAIL I..SAIL VI, SIL 1..SIL 4)
   - Decorator syntax: '@standard(StandardID, "ClauseRef", AssuranceLevel)' and doc tags.
5. Automated synthesis of STANDARDS_TAXONOMY_BASELINE.md and PARAMETER_MEASUREMENT_DICTIONARY.md.
"""

import os
import re
import sys
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Set, Tuple, Any, Union

try:
    from .base import IValidator
    from ..core.findings import Finding
    from ..core.workspace import WorkspaceRepository
except (ImportError, ValueError):
    _src_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
    if _src_dir not in sys.path:
        sys.path.insert(0, _src_dir)
    from parity_auditor.validators.base import IValidator
    from parity_auditor.core.findings import Finding
    from parity_auditor.core.workspace import WorkspaceRepository


@dataclass(frozen=True)
class SiDimensionVector:
    """
    7-dimensional SI base exponent vector in \mathbb{Z}^7.
    Indices:
      0: L (length, m)
      1: M (mass, kg)
      2: T (time, s)
      3: I (electric current, A)
      4: \Theta (temperature, K)
      5: N (amount of substance, mol)
      6: J (luminous intensity, cd)
    """
    d_l: int = 0
    d_m: int = 0
    d_t: int = 0
    d_i: int = 0
    d_theta: int = 0
    d_n: int = 0
    d_j: int = 0

    @property
    def exponents(self) -> Tuple[int, int, int, int, int, int, int]:
        return (self.d_l, self.d_m, self.d_t, self.d_i, self.d_theta, self.d_n, self.d_j)

    def is_dimensionless(self) -> bool:
        return all(x == 0 for x in self.exponents)

    def __add__(self, other: "SiDimensionVector") -> "SiDimensionVector":
        return SiDimensionVector(
            self.d_l + other.d_l,
            self.d_m + other.d_m,
            self.d_t + other.d_t,
            self.d_i + other.d_i,
            self.d_theta + other.d_theta,
            self.d_n + other.d_n,
            self.d_j + other.d_j,
        )

    def __sub__(self, other: "SiDimensionVector") -> "SiDimensionVector":
        return SiDimensionVector(
            self.d_l - other.d_l,
            self.d_m - other.d_m,
            self.d_t - other.d_t,
            self.d_i - other.d_i,
            self.d_theta - other.d_theta,
            self.d_n - other.d_n,
            self.d_j - other.d_j,
        )

    def __truediv__(self, other: "SiDimensionVector") -> "SiDimensionVector":
        return self - other

    def __mul__(self, factor: Union[int, "SiDimensionVector"]) -> "SiDimensionVector":
        if isinstance(factor, int):
            return SiDimensionVector(
                self.d_l * factor,
                self.d_m * factor,
                self.d_t * factor,
                self.d_i * factor,
                self.d_theta * factor,
                self.d_n * factor,
                self.d_j * factor,
            )
        elif isinstance(factor, SiDimensionVector):
            return self + factor
        return NotImplemented

    def __rmul__(self, factor: int) -> "SiDimensionVector":
        return self * factor

    def to_list(self) -> List[int]:
        return list(self.exponents)

    def to_latex(self) -> str:
        """Render KaTeX mathematical representation of the SI exponent vector."""
        if self.is_dimensionless():
            return r"1"

        symbols = [
            (self.d_l, r"\text{m}"),
            (self.d_m, r"\text{kg}"),
            (self.d_t, r"\text{s}"),
            (self.d_i, r"\text{A}"),
            (self.d_theta, r"\text{K}"),
            (self.d_n, r"\text{mol}"),
            (self.d_j, r"\text{cd}"),
        ]

        pos_parts = []
        neg_parts = []

        for exp, sym in symbols:
            if exp == 1:
                pos_parts.append(sym)
            elif exp > 1:
                pos_parts.append(f"{sym}^{{{exp}}}")
            elif exp == -1:
                neg_parts.append(f"{sym}^{{-1}}")
            elif exp < -1:
                neg_parts.append(f"{sym}^{{{exp}}}")

        all_parts = pos_parts + neg_parts
        return r" \cdot ".join(all_parts) if all_parts else r"1"

    def __str__(self) -> str:
        return f"[L={self.d_l}, M={self.d_m}, T={self.d_t}, I={self.d_i}, Θ={self.d_theta}, N={self.d_n}, J={self.d_j}]"


# Base unit lookup mapping
BASE_UNIT_MAP: Dict[str, SiDimensionVector] = {
    # Length
    "m": SiDimensionVector(d_l=1),
    "meter": SiDimensionVector(d_l=1),
    "meters": SiDimensionVector(d_l=1),
    "km": SiDimensionVector(d_l=1),
    "cm": SiDimensionVector(d_l=1),
    "mm": SiDimensionVector(d_l=1),
    "um": SiDimensionVector(d_l=1),
    "nm": SiDimensionVector(d_l=1),

    # Mass
    "kg": SiDimensionVector(d_m=1),
    "kilogram": SiDimensionVector(d_m=1),
    "kilograms": SiDimensionVector(d_m=1),
    "g": SiDimensionVector(d_m=1),
    "gram": SiDimensionVector(d_m=1),
    "grams": SiDimensionVector(d_m=1),
    "mg": SiDimensionVector(d_m=1),
    "ug": SiDimensionVector(d_m=1),

    # Time
    "s": SiDimensionVector(d_t=1),
    "sec": SiDimensionVector(d_t=1),
    "second": SiDimensionVector(d_t=1),
    "seconds": SiDimensionVector(d_t=1),
    "ms": SiDimensionVector(d_t=1),
    "us": SiDimensionVector(d_t=1),
    "ns": SiDimensionVector(d_t=1),
    "min": SiDimensionVector(d_t=1),
    "minute": SiDimensionVector(d_t=1),
    "minutes": SiDimensionVector(d_t=1),
    "hr": SiDimensionVector(d_t=1),
    "hour": SiDimensionVector(d_t=1),
    "hours": SiDimensionVector(d_t=1),

    # Current
    "a": SiDimensionVector(d_i=1),
    "amp": SiDimensionVector(d_i=1),
    "ampere": SiDimensionVector(d_i=1),
    "amperes": SiDimensionVector(d_i=1),
    "ma": SiDimensionVector(d_i=1),
    "ka": SiDimensionVector(d_i=1),

    # Temperature
    "k": SiDimensionVector(d_theta=1),
    "kelvin": SiDimensionVector(d_theta=1),
    "degk": SiDimensionVector(d_theta=1),
    "deg_k": SiDimensionVector(d_theta=1),
    "degc": SiDimensionVector(d_theta=1),
    "deg_c": SiDimensionVector(d_theta=1),
    "celsius": SiDimensionVector(d_theta=1),

    # Amount of substance
    "mol": SiDimensionVector(d_n=1),
    "mole": SiDimensionVector(d_n=1),
    "moles": SiDimensionVector(d_n=1),

    # Luminous intensity
    "cd": SiDimensionVector(d_j=1),
    "candela": SiDimensionVector(d_j=1),

    # Dimensionless
    "1": SiDimensionVector(),
    "dimensionless": SiDimensionVector(),
    "rad": SiDimensionVector(),
    "radian": SiDimensionVector(),
    "radians": SiDimensionVector(),
    "deg": SiDimensionVector(),
    "degree": SiDimensionVector(),
    "degrees": SiDimensionVector(),
    "deg_ang": SiDimensionVector(),
    "ratio": SiDimensionVector(),
    "percent": SiDimensionVector(),
    "%": SiDimensionVector(),
    "scalar": SiDimensionVector(),
    "unitless": SiDimensionVector(),
    "count": SiDimensionVector(),
    "boolean": SiDimensionVector(),
    "bool": SiDimensionVector(),
    "enum": SiDimensionVector(),
    "-": SiDimensionVector(),
    "none": SiDimensionVector(),
    "n/a": SiDimensionVector(),
    "": SiDimensionVector(),
}

# Derived unit shortcuts
DERIVED_UNIT_MAP: Dict[str, SiDimensionVector] = {
    # Frequency
    "hz": SiDimensionVector(d_t=-1),
    "1/s": SiDimensionVector(d_t=-1),
    "s^-1": SiDimensionVector(d_t=-1),
    "khz": SiDimensionVector(d_t=-1),
    "mhz": SiDimensionVector(d_t=-1),

    # Kinematics
    "m/s": SiDimensionVector(d_l=1, d_t=-1),
    "m*s^-1": SiDimensionVector(d_l=1, d_t=-1),
    "m s^-1": SiDimensionVector(d_l=1, d_t=-1),
    "mps": SiDimensionVector(d_l=1, d_t=-1),
    "km/h": SiDimensionVector(d_l=1, d_t=-1),
    "knot": SiDimensionVector(d_l=1, d_t=-1),
    "knots": SiDimensionVector(d_l=1, d_t=-1),
    "m/s^2": SiDimensionVector(d_l=1, d_t=-2),
    "m/s2": SiDimensionVector(d_l=1, d_t=-2),
    "m*s^-2": SiDimensionVector(d_l=1, d_t=-2),
    "mps2": SiDimensionVector(d_l=1, d_t=-2),
    "g_accel": SiDimensionVector(d_l=1, d_t=-2),

    # Geometry & Fluid
    "m^2": SiDimensionVector(d_l=2),
    "m2": SiDimensionVector(d_l=2),
    "cm^2": SiDimensionVector(d_l=2),
    "mm^2": SiDimensionVector(d_l=2),
    "m^3": SiDimensionVector(d_l=3),
    "m3": SiDimensionVector(d_l=3),
    "liter": SiDimensionVector(d_l=3),
    "l": SiDimensionVector(d_l=3),
    "ml": SiDimensionVector(d_l=3),
    "m^3/s": SiDimensionVector(d_l=3, d_t=-1),
    "l/s": SiDimensionVector(d_l=3, d_t=-1),
    "l/min": SiDimensionVector(d_l=3, d_t=-1),
    "kg/s": SiDimensionVector(d_m=1, d_t=-1),
    "g/s": SiDimensionVector(d_m=1, d_t=-1),
    "kg/m^3": SiDimensionVector(d_l=-3, d_m=1),
    "kg*m^-3": SiDimensionVector(d_l=-3, d_m=1),
    "g/cm^3": SiDimensionVector(d_l=-3, d_m=1),

    # Dynamics & Energy
    "n": SiDimensionVector(d_l=1, d_m=1, d_t=-2),
    "newton": SiDimensionVector(d_l=1, d_m=1, d_t=-2),
    "kn": SiDimensionVector(d_l=1, d_m=1, d_t=-2),
    "kg*m/s^2": SiDimensionVector(d_l=1, d_m=1, d_t=-2),
    "kg*m*s^-2": SiDimensionVector(d_l=1, d_m=1, d_t=-2),
    "n*m": SiDimensionVector(d_l=2, d_m=1, d_t=-2),
    "nm": SiDimensionVector(d_l=2, d_m=1, d_t=-2),
    "pa": SiDimensionVector(d_l=-1, d_m=1, d_t=-2),
    "pascal": SiDimensionVector(d_l=-1, d_m=1, d_t=-2),
    "kpa": SiDimensionVector(d_l=-1, d_m=1, d_t=-2),
    "hpa": SiDimensionVector(d_l=-1, d_m=1, d_t=-2),
    "bar": SiDimensionVector(d_l=-1, d_m=1, d_t=-2),
    "mbar": SiDimensionVector(d_l=-1, d_m=1, d_t=-2),
    "psi": SiDimensionVector(d_l=-1, d_m=1, d_t=-2),
    "n/m^2": SiDimensionVector(d_l=-1, d_m=1, d_t=-2),
    "j": SiDimensionVector(d_l=2, d_m=1, d_t=-2),
    "joule": SiDimensionVector(d_l=2, d_m=1, d_t=-2),
    "kj": SiDimensionVector(d_l=2, d_m=1, d_t=-2),
    "wh": SiDimensionVector(d_l=2, d_m=1, d_t=-2),
    "kwh": SiDimensionVector(d_l=2, d_m=1, d_t=-2),
    "w": SiDimensionVector(d_l=2, d_m=1, d_t=-3),
    "watt": SiDimensionVector(d_l=2, d_m=1, d_t=-3),
    "kw": SiDimensionVector(d_l=2, d_m=1, d_t=-3),
    "mw": SiDimensionVector(d_l=2, d_m=1, d_t=-3),
    "j/s": SiDimensionVector(d_l=2, d_m=1, d_t=-3),
    "j/kg": SiDimensionVector(d_l=2, d_t=-2),

    # Electromagnetism
    "c": SiDimensionVector(d_t=1, d_i=1),
    "coulomb": SiDimensionVector(d_t=1, d_i=1),
    "ah": SiDimensionVector(d_t=1, d_i=1),
    "mah": SiDimensionVector(d_t=1, d_i=1),
    "v": SiDimensionVector(d_l=2, d_m=1, d_t=-3, d_i=-1),
    "volt": SiDimensionVector(d_l=2, d_m=1, d_t=-3, d_i=-1),
    "mv": SiDimensionVector(d_l=2, d_m=1, d_t=-3, d_i=-1),
    "kv": SiDimensionVector(d_l=2, d_m=1, d_t=-3, d_i=-1),
    "f": SiDimensionVector(d_l=-2, d_m=-1, d_t=4, d_i=2),
    "farad": SiDimensionVector(d_l=-2, d_m=-1, d_t=4, d_i=2),
    "uf": SiDimensionVector(d_l=-2, d_m=-1, d_t=4, d_i=2),
    "nf": SiDimensionVector(d_l=-2, d_m=-1, d_t=4, d_i=2),
    "pf": SiDimensionVector(d_l=-2, d_m=-1, d_t=4, d_i=2),
    "ohm": SiDimensionVector(d_l=2, d_m=1, d_t=-3, d_i=-2),
    "v/a": SiDimensionVector(d_l=2, d_m=1, d_t=-3, d_i=-2),
    "kohm": SiDimensionVector(d_l=2, d_m=1, d_t=-3, d_i=-2),
    "mohm": SiDimensionVector(d_l=2, d_m=1, d_t=-3, d_i=-2),
    "wb": SiDimensionVector(d_l=2, d_m=1, d_t=-2, d_i=-1),
    "weber": SiDimensionVector(d_l=2, d_m=1, d_t=-2, d_i=-1),
    "t": SiDimensionVector(d_m=1, d_t=-2, d_i=-1),
    "tesla": SiDimensionVector(d_m=1, d_t=-2, d_i=-1),
    "h": SiDimensionVector(d_l=2, d_m=1, d_t=-2, d_i=-2),
    "henry": SiDimensionVector(d_l=2, d_m=1, d_t=-2, d_i=-2),
    "v/m": SiDimensionVector(d_l=1, d_m=1, d_t=-3, d_i=-1),

    # Optics / Radiation / Chemistry
    "lm": SiDimensionVector(d_j=1),
    "lumen": SiDimensionVector(d_j=1),
    "lx": SiDimensionVector(d_l=-2, d_j=1),
    "lux": SiDimensionVector(d_l=-2, d_j=1),
    "bq": SiDimensionVector(d_t=-1),
    "becquerel": SiDimensionVector(d_t=-1),
    "gy": SiDimensionVector(d_l=2, d_t=-2),
    "gray": SiDimensionVector(d_l=2, d_t=-2),
    "sv": SiDimensionVector(d_l=2, d_t=-2),
    "sievert": SiDimensionVector(d_l=2, d_t=-2),
    "kat": SiDimensionVector(d_t=-1, d_n=1),
    "katal": SiDimensionVector(d_t=-1, d_n=1),
}


def _parse_single_term(term: str) -> SiDimensionVector:
    """Parses a single unit term, potentially with exponent (e.g. 'm^2', 's^-1', 'kg')."""
    term = term.strip()
    if not term:
        return SiDimensionVector()

    # Match base with optional exponent: 'm^2', 's^-1', 's2'
    m_exp = re.match(r'^([a-zA-Z%Ω]+)(?:\^?(-?\d+))?$', term)
    if m_exp:
        base_sym = m_exp.group(1).lower()
        exp_val = int(m_exp.group(2)) if m_exp.group(2) else 1

        # Check in base or derived maps
        if base_sym in BASE_UNIT_MAP:
            return BASE_UNIT_MAP[base_sym] * exp_val
        if base_sym in DERIVED_UNIT_MAP:
            return DERIVED_UNIT_MAP[base_sym] * exp_val

    # Case-sensitive check for symbols like V, A, K, C, W, N, J, F, T, H
    if term in ("V", "A", "K", "C", "W", "N", "J", "F", "T", "H", "Ohm", "Hz", "Pa", "Wb"):
        term_lower = term.lower()
        if term_lower in DERIVED_UNIT_MAP:
            return DERIVED_UNIT_MAP[term_lower]
        if term_lower in BASE_UNIT_MAP:
            return BASE_UNIT_MAP[term_lower]

    cleaned = term.lower()
    if cleaned in DERIVED_UNIT_MAP:
        return DERIVED_UNIT_MAP[cleaned]
    if cleaned in BASE_UNIT_MAP:
        return BASE_UNIT_MAP[cleaned]

    return SiDimensionVector()


def parse_si_unit(unit_str: str) -> SiDimensionVector:
    """
    Parses an SI unit expression into its canonical 7-dimensional exponent vector.

    Examples:
        'm/s' -> (1, 0, -1, 0, 0, 0, 0)
        'kg*m/s^2' -> (1, 1, -2, 0, 0, 0, 0)
        'N' -> (1, 1, -2, 0, 0, 0, 0)
        'rad' -> (0, 0, 0, 0, 0, 0, 0)
    """
    if not unit_str:
        return SiDimensionVector()

    raw = unit_str.strip().strip('`[]_ \t\n\r')
    if not raw:
        return SiDimensionVector()

    # Exact dictionary lookup first
    raw_lower = raw.lower()
    if raw_lower in DERIVED_UNIT_MAP:
        return DERIVED_UNIT_MAP[raw_lower]
    if raw_lower in BASE_UNIT_MAP:
        return BASE_UNIT_MAP[raw_lower]

    # Handle division: numerator / denominator
    if "/" in raw:
        parts = raw.split("/", 1)
        num_str = parts[0].strip()
        den_str = parts[1].strip()

        # Handle parentheses around denominator: /(m*s^2)
        den_str = re.sub(r'^[()]+|[()]+$', '', den_str)

        num_vec = parse_si_unit(num_str) if num_str else SiDimensionVector()
        den_vec = parse_si_unit(den_str) if den_str else SiDimensionVector()
        return num_vec - den_vec

    # Handle multiplication: split by '*' or whitespace or '·'
    sub_terms = re.split(r'[*·\s]+', raw)
    total_vec = SiDimensionVector()
    for st in sub_terms:
        st = st.strip()
        if st:
            total_vec = total_vec + _parse_single_term(st)

    return total_vec


# =============================================================================
# SDO Standards Taxonomy & Lattice Formalisms
# =============================================================================

SDO_STANDARDS_TAXONOMY: Dict[str, List[str]] = {
    "RTCA": ["DO-178C", "DO-254", "DO-331", "DO-330", "DO-160G", "DO-385", "DO-365"],
    "SAE": ["ARP4754A", "ARP4754B", "ARP4761", "ARP4761A", "AS5506", "AS6983"],
    "ISO": ["ISO-26262", "ISO-15288", "ISO-80000", "ISO-21448", "ISO-21434"],
    "IEC": ["IEC-62304", "IEC-61508", "IEC-62443"],
    "JARUS": ["JARUS-SORA-V2.5", "JARUS-SORA-V2.0", "JARUS-SORA"],
    "ASTM": ["ASTM-F3298", "ASTM-F3005", "ASTM-F3178", "ASTM-F3269"],
    "IEEE": ["IEEE-15288", "IEEE-1362", "IEEE-1012"],
}

# Monotonic assurance level lattice definitions
ASSURANCE_LEVEL_LATTICES: Dict[str, Dict[str, int]] = {
    # RTCA & SAE: Design Assurance Levels (DAL)
    "RTCA": {
        "DAL-A": 5, "DAL-B": 4, "DAL-C": 3, "DAL-D": 2, "DAL-E": 1,
        "DAL_A": 5, "DAL_B": 4, "DAL_C": 3, "DAL_D": 2, "DAL_E": 1,
        "DALA": 5, "DALB": 4, "DALC": 3, "DALD": 2, "DALE": 1,
    },
    "SAE": {
        "DAL-A": 5, "DAL-B": 4, "DAL-C": 3, "DAL-D": 2, "DAL-E": 1,
        "DAL_A": 5, "DAL_B": 4, "DAL_C": 3, "DAL_D": 2, "DAL_E": 1,
        "DALA": 5, "DALB": 4, "DALC": 3, "DALD": 2, "DALE": 1,
    },
    # ISO: Automotive Safety Integrity Levels (ASIL)
    "ISO": {
        "ASIL-D": 4, "ASIL-C": 3, "ASIL-B": 2, "ASIL-A": 1, "QM": 0,
        "ASIL_D": 4, "ASIL_C": 3, "ASIL_B": 2, "ASIL_A": 1,
        "ASILD": 4, "ASILC": 3, "ASILB": 2, "ASILA": 1,
    },
    # JARUS SORA: Specific Assurance and Integrity Levels (SAIL)
    "JARUS": {
        "SAIL-VI": 6, "SAIL-V": 5, "SAIL-IV": 4, "SAIL-III": 3, "SAIL-II": 2, "SAIL-I": 1,
        "SAIL_VI": 6, "SAIL_V": 5, "SAIL_IV": 4, "SAIL_III": 3, "SAIL_II": 2, "SAIL_I": 1,
        "SAIL VI": 6, "SAIL V": 5, "SAIL IV": 4, "SAIL III": 3, "SAIL II": 2, "SAIL I": 1,
        "SAIL-6": 6, "SAIL-5": 5, "SAIL-4": 4, "SAIL-3": 3, "SAIL-2": 2, "SAIL-1": 1,
        "SAIL6": 6, "SAIL5": 5, "SAIL4": 4, "SAIL3": 3, "SAIL2": 2, "SAIL1": 1,
    },
    # IEC: Safety Integrity Levels (SIL) & Software Safety Classes
    "IEC": {
        "SIL-4": 4, "SIL-3": 3, "SIL-2": 2, "SIL-1": 1,
        "SIL_4": 4, "SIL_3": 3, "SIL_2": 2, "SIL_1": 1,
        "SIL 4": 4, "SIL 3": 3, "SIL 2": 2, "SIL 1": 1,
        "SIL4": 4, "SIL3": 3, "SIL2": 2, "SIL1": 1,
        "CLASS-C": 3, "CLASS-B": 2, "CLASS-A": 1,
        "CLASS_C": 3, "CLASS_B": 2, "CLASS_A": 1,
        "CLASS C": 3, "CLASS B": 2, "CLASS A": 1,
    },
    # ASTM & IEEE: General Assurance Rankings
    "ASTM": {
        "HIGH": 3, "MEDIUM": 2, "LOW": 1,
        "LEVEL-A": 3, "LEVEL-B": 2, "LEVEL-C": 1,
        "LEVEL A": 3, "LEVEL B": 2, "LEVEL C": 1,
    },
    "IEEE": {
        "HIGH": 3, "MEDIUM": 2, "LOW": 1,
        "LEVEL-A": 3, "LEVEL-B": 2, "LEVEL-C": 1,
        "LEVEL A": 3, "LEVEL B": 2, "LEVEL C": 1,
    },
}


def _normalize_standard_id(raw_id: str) -> str:
    """Normalizes standard identifier into canonical uppercase hyphenated format."""
    s = raw_id.strip().upper()
    s = re.sub(r'[*`_\[\]]', '', s).strip()
    s = s.replace("_", "-")

    # Match DO-178C, DO-254
    m_do = re.match(r'^DO-?(\d+[A-Z]?)$', s)
    if m_do:
        return f"DO-{m_do.group(1)}"

    # Match ISO-XXXXX
    m_iso = re.match(r'^ISO-?(\d+.*)$', s)
    if m_iso:
        return f"ISO-{m_iso.group(1)}"

    # Match IEC-XXXXX
    m_iec = re.match(r'^IEC-?(\d+.*)$', s)
    if m_iec:
        return f"IEC-{m_iec.group(1)}"

    # Match ARPXXXX
    m_arp = re.match(r'^ARP-?(\d+[A-Z]?)$', s)
    if m_arp:
        return f"ARP{m_arp.group(1)}"

    # Match JARUS-SORA
    if "SORA" in s:
        m_sora = re.search(r'V?(\d+(?:\.\d+)?)$', s)
        ver = f"-V{m_sora.group(1)}" if m_sora else ""
        return f"JARUS-SORA{ver}"

    return s


SAFETY_CRITICAL_STANDARDS: Set[str] = {
    "JARUS-SORA-V2.5", "JARUS-SORA-V2.0", "JARUS-SORA",
    "ISO-26262", "ISO-21448",
    "DO-178C", "DO-254", "DO-331", "DO-385", "DO-365",
    "IEC-62304", "IEC-61508", "IEC-62443",
    "ARP4754A", "ARP4754B", "ARP4761", "ARP4761A",
    "MIL-STD-882E", "MIL-STD-1629A",
    "ASTM-F3269", "ASTM-F3298",
}


def is_safety_critical_standard(standard_id: str) -> bool:
    """Checks whether a canonical standard identifier is a safety-critical standard."""
    norm = _normalize_standard_id(standard_id)
    if norm in SAFETY_CRITICAL_STANDARDS:
        return True
    for s in SAFETY_CRITICAL_STANDARDS:
        if norm.startswith(s) or s.startswith(norm):
            return True
    if any(k in norm for k in ("SORA", "26262", "62304", "61508", "178C", "254", "882E", "4761", "4754", "3269")):
        return True
    return False


def _parse_sora_table_rows(text: str) -> Set[str]:
    """
    Parses SORA OSO IDs from markdown table data rows in text.
    Returns set of canonical OSO IDs (e.g. {'OSO-01', ..., 'OSO-24'}).
    """
    oso_ids: Set[str] = set()
    lines = text.splitlines()
    header_seen = False

    for line in lines:
        stripped = line.strip()
        if stripped.startswith("|") and stripped.endswith("|"):
            cells = [c.strip() for c in stripped.split("|")[1:-1]]
            # Check for separator row: |:---|:---| or |---|---|
            if all(re.fullmatch(r":?-{1,}:?", c) for c in cells if c):
                header_seen = True
                continue
            if not header_seen:
                # Header row
                continue
            # Data row
            for cell in cells:
                m = re.search(r'\bOSO-?0*(\d+)\b', cell, re.IGNORECASE)
                if m:
                    idx = int(m.group(1))
                    if 1 <= idx <= 24:
                        oso_ids.add(f"OSO-{idx:02d}")
        else:
            header_seen = False
    return oso_ids


def _resolve_sdo(standard_id: str) -> Optional[str]:
    """Resolves the SDO issuing body for a given canonical standard ID."""
    norm = _normalize_standard_id(standard_id)
    for sdo, std_list in SDO_STANDARDS_TAXONOMY.items():
        for std in std_list:
            if norm == std or norm.replace("-", "") == std.replace("-", "") or norm.startswith(std):
                return sdo
    return None


@dataclass
class StandardDecorator:
    standard_id: str
    clause_ref: str
    assurance_level: str
    sdo: Optional[str] = None
    location: str = ""
    raw_text: str = ""

    def __post_init__(self):
        if not self.sdo:
            object.__setattr__(self, "sdo", _resolve_sdo(self.standard_id))


def parse_standard_decorators(text: str, location: str = "") -> List[StandardDecorator]:
    """
    Extracts all @standard(...) SysML decorators and doc tags from content.

    Supported patterns:
        @standard(DO_178C, "Table A-3", DAL_A)
        @standard(RTCA::DO-178C, "Table A-3", DAL-A)
        /// Standard: [DO-178C, "Table A-3", DAL-A]
        /// StandardTaxonomy: [ISO-26262, "Part 3 Clause 5", ASIL-D]
        doc /* /// Standard: [DO-254, "Section 5.1", DAL-B] */
    """
    decorators: List[StandardDecorator] = []

    # 1. SysML decorator: @standard(StandardID, "ClauseRef", AssuranceLevel)
    sysml_pattern = re.compile(
        r'@standard\s*\(\s*([A-Za-z0-9_:\-]+)\s*,\s*["\']([^"\']+)["\']\s*,\s*([A-Za-z0-9_\-\s]+)\s*\)',
        re.IGNORECASE
    )
    for match in sysml_pattern.finditer(text):
        raw_std = match.group(1).strip()
        clause = match.group(2).strip()
        raw_lvl = match.group(3).strip()

        # Strip SDO prefix if present: RTCA::DO-178C
        if "::" in raw_std:
            _, raw_std = raw_std.split("::", 1)

        norm_std = _normalize_standard_id(raw_std)
        norm_lvl = re.sub(r'[\s_]+', '-', raw_lvl.strip().upper())
        decorators.append(StandardDecorator(
            standard_id=norm_std,
            clause_ref=clause,
            assurance_level=norm_lvl,
            location=location,
            raw_text=match.group(0)
        ))

    # 2. Markdown / doc comment tag: /// Standard: [Std, Clause, Level]
    doc_pattern = re.compile(
        r'///\s*(?:Standard|StandardTaxonomy)\s*:\s*\[([^\]]+)\]',
        re.IGNORECASE
    )
    for match in doc_pattern.finditer(text):
        parts = [p.strip().strip('"\'') for p in match.group(1).split(",")]
        if len(parts) >= 3:
            raw_std = parts[0]
            clause = parts[1]
            raw_lvl = parts[2]
            if "/" in raw_std:
                _, raw_std = raw_std.split("/", 1)
            norm_std = _normalize_standard_id(raw_std)
            norm_lvl = re.sub(r'[\s_]+', '-', raw_lvl.strip().upper())
            decorators.append(StandardDecorator(
                standard_id=norm_std,
                clause_ref=clause,
                assurance_level=norm_lvl,
                location=location,
                raw_text=match.group(0)
            ))
        elif len(parts) < 3:
            # Malformed tag: capture with empty parts to allow validator to flag
            decorators.append(StandardDecorator(
                standard_id=parts[0] if len(parts) > 0 else "",
                clause_ref=parts[1] if len(parts) > 1 else "",
                assurance_level="",
                location=location,
                raw_text=match.group(0)
            ))

    return decorators


# =============================================================================
# Nyquist-Shannon Sampling Frequency Helpers
# =============================================================================

def parse_rate_hz(val: str) -> Optional[float]:
    """Parse update frequency in Hz from a string representation."""
    if not val:
        return None
    val_clean = val.strip()
    m_mhz = re.search(r'([0-9]+(?:\.[0-9]+)?)\s*mhz\b', val_clean, re.IGNORECASE)
    if m_mhz:
        return float(m_mhz.group(1)) * 1_000_000.0
    m_khz = re.search(r'([0-9]+(?:\.[0-9]+)?)\s*khz\b', val_clean, re.IGNORECASE)
    if m_khz:
        return float(m_khz.group(1)) * 1000.0
    m_hz = re.search(r'([0-9]+(?:\.[0-9]+)?)\s*hz\b', val_clean, re.IGNORECASE)
    if m_hz:
        return float(m_hz.group(1))
    m_num = re.search(r'^([0-9]+(?:\.[0-9]+)?)$', val_clean)
    if m_num:
        return float(m_num.group(1))
    return None


def parse_nyquist_parameters(update_rate_str: str, max_freq_str: str) -> Optional[Tuple[float, float]]:
    """
    Parses sampling frequency (f_sample) and signal bandwidth (f_max) in Hz.
    Returns (f_sample, f_max) or None if either cannot be parsed.
    """
    f_sample = parse_rate_hz(update_rate_str)
    f_max = parse_rate_hz(max_freq_str)
    if f_sample is not None and f_max is not None:
        return (f_sample, f_max)
    return None


# =============================================================================
# Gate 25: StandardsAndMeasurementValidator
# =============================================================================

def _infer_dimension_from_type_or_name(name_or_type: str) -> Optional[SiDimensionVector]:
    """Infers the physical SI dimension vector from a port or signal type name."""
    if not name_or_type:
        return None
    clean = re.sub(r'[*`_\[\]]', '', name_or_type).strip().lower()

    # Exact unit match
    if clean in DERIVED_UNIT_MAP:
        return DERIVED_UNIT_MAP[clean]
    if clean in BASE_UNIT_MAP:
        return BASE_UNIT_MAP[clean]

    # Semantic keyword patterns
    if "position" in clean or "distance" in clean or "altitude" in clean or clean.startswith("length"):
        return SiDimensionVector(d_l=1)
    if "accel" in clean:
        return SiDimensionVector(d_l=1, d_t=-2)
    if "velocity" in clean or "speed" in clean or "rateofclimb" in clean:
        return SiDimensionVector(d_l=1, d_t=-1)
    if "pressure" in clean or "baro" in clean:
        return SiDimensionVector(d_l=-1, d_m=1, d_t=-2)
    if "force" in clean or "thrust" in clean:
        return SiDimensionVector(d_l=1, d_m=1, d_t=-2)
    if "torque" in clean or "moment" in clean:
        return SiDimensionVector(d_l=2, d_m=1, d_t=-2)
    if "power" in clean:
        return SiDimensionVector(d_l=2, d_m=1, d_t=-3)
    if "energy" in clean or "work" in clean:
        return SiDimensionVector(d_l=2, d_m=1, d_t=-2)
    if "voltage" in clean or "potential" in clean:
        return SiDimensionVector(d_l=2, d_m=1, d_t=-3, d_i=-1)
    if "current" in clean and "concurrent" not in clean and "recurrent" not in clean:
        return SiDimensionVector(d_i=1)
    if "temp" in clean:
        return SiDimensionVector(d_theta=1)
    if "freq" in clean or "frequency" in clean:
        return SiDimensionVector(d_t=-1)
    if "angle" in clean or "heading" in clean or "pitch" in clean or "roll" in clean or "yaw" in clean:
        return SiDimensionVector()
    return None


class StandardsAndMeasurementValidator(IValidator):
    """
    Gate 25: Standards & SI 7-Dimensional Parameter Metrology Validator.

    Enforces:
    - SI 7-dimensional exponent vector parsing in \mathbb{Z}^7
    - Theorem 3 Dimensional Homogeneity: D(e_{src}) == D(e_{dst})
    - Nyquist-Shannon sampling frequency check: f_{sample} >= 2 * f_{max}
    - SDO Standards Taxonomy lattice validation
    """

    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[Finding]:
        findings: List[Finding] = []
        workspace_dir = repo.workspace_dir

        # 1. Collect all standard decorators across specifications and SysML models
        scan_dirs = ["docs/features", "docs/epics", "docs/user-stories", "docs/use-cases", "docs/interfaces", "schema"]
        target_files: List[Tuple[str, str]] = []

        for sdir in scan_dirs:
            full_dir = os.path.join(workspace_dir, sdir)
            if os.path.isdir(full_dir):
                for root, _, files in os.walk(full_dir):
                    for f in sorted(files):
                        if (f.endswith(".md") or f.endswith(".sysml")) and f != "README.md":
                            full_p = os.path.join(root, f)
                            rel_p = os.path.relpath(full_p, workspace_dir)
                            target_files.append((full_p, rel_p))

        pipeline_sysml = os.path.join(workspace_dir, ".pipeline", "schema.sysml")
        if os.path.isfile(pipeline_sysml):
            target_files.append((pipeline_sysml, ".pipeline/schema.sysml"))

        decorators: List[StandardDecorator] = []
        for full_p, rel_p in target_files:
            try:
                with open(full_p, "r", encoding="utf-8", errors="ignore") as f:
                    content = f.read()
                decs = parse_standard_decorators(content, location=rel_p)
                decorators.extend(decs)
            except Exception:
                pass

        # 2. Validate SDO Standards Taxonomy Lattice
        for dec in decorators:
            loc = dec.location or "unknown"
            if not dec.standard_id or not dec.clause_ref or not dec.assurance_level:
                findings.append(Finding(
                    "standards-malformed-decorator",
                    f"Malformed standard decorator '{dec.raw_text}' at {loc}: missing required standard ID, clause reference, or assurance level.",
                    location=loc,
                    detail={"raw_text": dec.raw_text, "location": loc}
                ))
                continue

            sdo = dec.sdo
            if not sdo:
                findings.append(Finding(
                    "standards-unrecognized-standard",
                    f"Standard identifier '{dec.standard_id}' at {loc} is not recognized in the authoritative SDO standards taxonomy catalog (RTCA, SAE, ISO, IEC, JARUS, ASTM, IEEE).",
                    location=loc,
                    detail={"standard_id": dec.standard_id, "location": loc}
                ))
                continue

            # Validate assurance level in standard family lattice
            lattice = ASSURANCE_LEVEL_LATTICES.get(sdo, {})
            norm_lvl = dec.assurance_level.upper().replace("_", "-")
            if norm_lvl not in lattice:
                valid_levels = sorted(list({k for k in lattice.keys() if "_" not in k and " " not in k}))
                findings.append(Finding(
                    "standards-invalid-assurance-level",
                    f"Assurance level '{dec.assurance_level}' is invalid for standard '{dec.standard_id}' (SDO: {sdo}) at {loc}. Valid assurance levels in this taxonomy: {', '.join(valid_levels)}.",
                    location=loc,
                    detail={"standard_id": dec.standard_id, "assurance_level": dec.assurance_level, "sdo": sdo, "location": loc}
                ))

        # 2.5 Validate Phase-0 Safety Obligations & Mechanical Witnesses (Issue #93)
        declared_safety_stds: Set[str] = set()
        for dec in decorators:
            if dec.standard_id and (is_safety_critical_standard(dec.standard_id) or dec.sdo in ("JARUS", "RTCA", "SAE")):
                declared_safety_stds.add(dec.standard_id)

        # Also inspect docs/research/RESEARCH_INVENTORY.md if present
        inventory_file = os.path.join(workspace_dir, "docs", "research", "RESEARCH_INVENTORY.md")
        if os.path.isfile(inventory_file):
            try:
                with open(inventory_file, "r", encoding="utf-8", errors="ignore") as inv_f:
                    inv_text = inv_f.read()
                for line in inv_text.splitlines():
                    if "|" in line:
                        for part in line.split("|"):
                            clean_part = part.strip().strip("`* ")
                            if clean_part and is_safety_critical_standard(clean_part):
                                declared_safety_stds.add(_normalize_standard_id(clean_part))
            except Exception:
                pass

        if declared_safety_stds:
            is_upstream = repo.is_upstream_compiler_repo() if hasattr(repo, "is_upstream_compiler_repo") else False
            safety_dir = os.path.join(workspace_dir, "docs", "safety")
            safety_files: List[Tuple[str, str]] = []
            if os.path.isdir(safety_dir):
                for root, _, files in os.walk(safety_dir):
                    for f in sorted(files):
                        if f.endswith(".md") and f != "README.md":
                            full_p = os.path.join(root, f)
                            rel_p = os.path.relpath(full_p, workspace_dir)
                            safety_files.append((full_p, rel_p))

            if is_upstream and not safety_files and not decorators:
                # Upstream distribution template clean landing zone passes cleanly
                pass
            elif not os.path.isdir(safety_dir) or not safety_files:
                findings.append(Finding(
                    "standards-missing-safety-baseline",
                    f"Safety-critical standard(s) ({', '.join(sorted(declared_safety_stds))}) declared in normative baseline, but docs/safety/ directory is absent or contains zero safety specifications.",
                    location="docs/safety",
                    detail={"declared_safety_standards": sorted(declared_safety_stds)}
                ))
            else:
                # Read all safety files
                safety_content_parts = []
                for full_p, _ in safety_files:
                    try:
                        with open(full_p, "r", encoding="utf-8", errors="ignore") as sf:
                            safety_content_parts.append(sf.read())
                    except Exception:
                        pass
                safety_content = "\n\n".join(safety_content_parts)

                # SORA OSO mechanical witness validation
                is_sora_declared = any("SORA" in s.upper() for s in declared_safety_stds)
                if is_sora_declared:
                    oso_rows = _parse_sora_table_rows(safety_content)
                    if len(oso_rows) == 0:
                        findings.append(Finding(
                            "standards-empty-oso-table",
                            "SORA safety standard declared in normative baseline, but docs/safety/ contains 0 OSO evaluation table rows (mechanical witness failure).",
                            location="docs/safety",
                            detail={"declared_standards": sorted(declared_safety_stds), "oso_rows_found": 0}
                        ))
                    else:
                        expected_osos = {f"OSO-{i:02d}" for i in range(1, 25)}
                        missing_osos = sorted(expected_osos - oso_rows)
                        if missing_osos:
                            findings.append(Finding(
                                "standards-missing-sora-oso-witness",
                                f"SORA OSO evaluation table in docs/safety/ is missing mechanical witnesses for {len(missing_osos)} OSO(s): {', '.join(missing_osos)}.",
                                location="docs/safety",
                                detail={"missing_osos": missing_osos, "found_count": len(oso_rows)}
                            ))

        # 3. Read Level 1C ICD tables for Theorem 3 and Nyquist validation
        icd01_path = os.path.join(workspace_dir, "docs", "interfaces", "ICD_01_SYSTEM_INTERFACE_MATRIX.md")
        icd02_path = os.path.join(workspace_dir, "docs", "interfaces", "ICD_02_MASTER_SIGNAL_DICTIONARY.md")

        signals: List[Dict[str, str]] = []
        connections: List[Dict[str, str]] = []
        port_types: Dict[str, str] = {}

        if os.path.isfile(icd01_path):
            try:
                with open(icd01_path, "r", encoding="utf-8", errors="ignore") as f:
                    icd01_content = f.read()
                from .icd_completeness_validator import _parse_detailed_markdown_tables
                tables01 = _parse_detailed_markdown_tables(icd01_content)
                for tbl in tables01:
                    for row in tbl.rows:
                        if "port_id" in row and "port_name" in row:
                            pid = row.get("port_id", "").strip("`").strip()
                            subsys = row.get("subsystem", "").strip("`").strip()
                            pname = row.get("port_name", "").strip("`").strip()
                            ptype = row.get("port_type", "").strip("`").strip()
                            if pid and ptype:
                                port_types[pid] = ptype
                            if subsys and pname and ptype:
                                port_types[f"{subsys}.{pname}"] = ptype
                            if pname and ptype:
                                port_types[pname] = ptype
                        elif "connection_id" in row and "source_port" in row and "dest_port" in row:
                            connections.append(row)
            except Exception:
                pass

        if os.path.isfile(icd02_path):
            try:
                with open(icd02_path, "r", encoding="utf-8", errors="ignore") as f:
                    icd02_content = f.read()
                from .icd_completeness_validator import _parse_detailed_markdown_tables
                tables02 = _parse_detailed_markdown_tables(icd02_content)
                for tbl in tables02:
                    for row in tbl.rows:
                        if "signal_id" in row or "signal_name" in row:
                            signals.append(row)
            except Exception:
                pass

        # 4. Theorem 3 (Dimensional Homogeneity): Assert D(e_src) == D(e_dst)
        for sig in signals:
            sig_id = sig.get("signal_id", "SIG-UNKNOWN")
            sig_name = sig.get("signal_name", sig_id)
            src_port = (sig.get("source_port", "") or sig.get("src_port", "") or sig.get("source", "")).strip("`").strip()
            dst_port = (sig.get("dest_port", "") or sig.get("dst_port", "") or sig.get("dest", "")).strip("`").strip()
            sig_unit = (sig.get("si_units", "") or sig.get("units", "") or sig.get("unit", "")).strip("`").strip()

            src_dim = parse_si_unit(sig_unit)

            # Check destination port type expected dimension
            dst_ptype = port_types.get(dst_port) or port_types.get(dst_port.split(".")[-1], "")
            if dst_ptype:
                dst_expected_dim = _infer_dimension_from_type_or_name(dst_ptype)
                if dst_expected_dim is not None and dst_expected_dim.exponents != src_dim.exponents:
                    findings.append(Finding(
                        "metrology-dimensional-inhomogeneity",
                        f"Theorem 3 (Dimensional Homogeneity) violation on signal '{sig_id}' ({sig_name}): source port '{src_port}' provides unit '{sig_unit}' ({src_dim}) but destination port '{dst_port}' (type '{dst_ptype}') expects dimension {dst_expected_dim}: D(e_src) != D(e_dst).",
                        location=icd02_path,
                        detail={
                            "signal_id": sig_id,
                            "source_port": src_port,
                            "dest_port": dst_port,
                            "source_dimension": src_dim.to_list(),
                            "dest_dimension": dst_expected_dim.to_list()
                        }
                    ))

            # Check source port type expected dimension
            src_ptype = port_types.get(src_port) or port_types.get(src_port.split(".")[-1], "")
            if src_ptype:
                src_expected_dim = _infer_dimension_from_type_or_name(src_ptype)
                if src_expected_dim is not None and src_expected_dim.exponents != src_dim.exponents:
                    findings.append(Finding(
                        "metrology-dimensional-inhomogeneity",
                        f"Theorem 3 (Dimensional Homogeneity) violation on signal '{sig_id}' ({sig_name}): source port '{src_port}' (type '{src_ptype}') expects dimension {src_expected_dim} but signal provides unit '{sig_unit}' ({src_dim}): D(e_src) != D(e_dst).",
                        location=icd02_path,
                        detail={
                            "signal_id": sig_id,
                            "source_port": src_port,
                            "dest_port": dst_port,
                            "source_dimension": src_expected_dim.to_list(),
                            "signal_dimension": src_dim.to_list()
                        }
                    ))

        # 5. Nyquist-Shannon Sampling Check: f_sample >= 2 * f_max
        for sig in signals:
            sig_id = sig.get("signal_id", "SIG-UNKNOWN")
            sig_name = sig.get("signal_name", sig_id)
            update_rate_str = sig.get("update_rate", "") or sig.get("rate", "") or sig.get("frequency", "")
            max_freq_str = (
                sig.get("max_frequency", "")
                or sig.get("max_freq", "")
                or sig.get("bandwidth", "")
                or sig.get("f_max", "")
                or sig.get("signal_bandwidth", "")
            )

            # If max frequency not explicitly in table column, check valid range or comments
            if not max_freq_str:
                valid_range = sig.get("valid_range", "") or ""
                m_bw = re.search(r'bandwidth\s*[:=]\s*([0-9]+(?:\.[0-9]+)?\s*[kKmM]?Hz)', valid_range, re.IGNORECASE)
                if m_bw:
                    max_freq_str = m_bw.group(1)

            if update_rate_str and max_freq_str:
                nyq = parse_nyquist_parameters(update_rate_str, max_freq_str)
                if nyq is not None:
                    f_sample, f_max = nyq
                    if f_sample < 2.0 * f_max:
                        findings.append(Finding(
                            "metrology-nyquist-aliasing-detected",
                            f"Nyquist-Shannon sampling violation on signal '{sig_id}' ({sig_name}): update rate f_sample={f_sample:.1f} Hz is less than 2 * f_max = 2 * {f_max:.1f} Hz = {2.0 * f_max:.1f} Hz (aliasing and state estimation distortion risk).",
                            location=icd02_path,
                            detail={
                                "signal_id": sig_id,
                                "signal_name": sig_name,
                                "f_sample_hz": f_sample,
                                "f_max_hz": f_max,
                                "nyquist_threshold_hz": 2.0 * f_max
                            }
                        ))

        return findings

    def synthesize_standards_baseline(self, repo: WorkspaceRepository) -> str:
        """
        Generates markdown content for STANDARDS_TAXONOMY_BASELINE.md.
        """
        findings = self.validate(repo)
        workspace_dir = repo.workspace_dir

        scan_dirs = ["docs/features", "docs/epics", "docs/user-stories", "docs/use-cases", "docs/interfaces", "schema"]
        decorators: List[StandardDecorator] = []

        for sdir in scan_dirs:
            full_dir = os.path.join(workspace_dir, sdir)
            if os.path.isdir(full_dir):
                for root, _, files in os.walk(full_dir):
                    for f in sorted(files):
                        if (f.endswith(".md") or f.endswith(".sysml")) and f != "README.md":
                            full_p = os.path.join(root, f)
                            rel_p = os.path.relpath(full_p, workspace_dir)
                            try:
                                with open(full_p, "r", encoding="utf-8", errors="ignore") as file_obj:
                                    decorators.extend(parse_standard_decorators(file_obj.read(), location=rel_p))
                            except Exception:
                                pass

        lines: List[str] = [
            "| **Attribute** | **Value** |",
            "| :--- | :--- |",
            "| **Document Title** | OMG UAF Standards Taxonomy Baseline (Std-Tx) |",
            "| **Document ID** | UAF-STD-TAXONOMY-001 |",
            "| **Standard Alignment** | OMG UAF v1.2 / v2.0 (Std-Tx) & ISO/IEC/IEEE 15288:2023 |",
            "| **Quality Gate** | Gate 25 (StandardsAndMeasurementValidator) |",
            f"| **Active Standard Citations** | {len(decorators)} |",
            "",
            "# Standards Taxonomy Baseline (Std-Tx)",
            "",
            "## 1. Executive Summary & SDO Governance Lattice",
            "",
            "This document formalizes the authoritative baseline of Standards Developing Organizations (SDOs), standard baselines, clause citations, and monotonic assurance level allocations across the system.",
            "",
            "## 2. Active Standards Citation Matrix",
            "",
            "| SDO Issuing Body | Standard ID | Clause Reference | Assurance Level | Realizing Artifact Location |",
            "| :--- | :--- | :--- | :--- | :--- |",
        ]

        for dec in sorted(decorators, key=lambda d: (d.sdo or "", d.standard_id, d.location)):
            sdo_str = dec.sdo or "UNKNOWN"
            lines.append(f"| {sdo_str} | `{dec.standard_id}` | {dec.clause_ref} | `{dec.assurance_level}` | `{dec.location}` |")

        if not decorators:
            lines.append("| *None* | *None* | *None* | *None* | *None* |")

        lines.append("")
        return "\n".join(lines)

    def synthesize_parameter_dictionary(self, repo: WorkspaceRepository) -> str:
        """
        Generates markdown content for PARAMETER_MEASUREMENT_DICTIONARY.md.
        """
        workspace_dir = repo.workspace_dir
        icd02_path = os.path.join(workspace_dir, "docs", "interfaces", "ICD_02_MASTER_SIGNAL_DICTIONARY.md")

        signals: List[Dict[str, str]] = []
        if os.path.isfile(icd02_path):
            try:
                with open(icd02_path, "r", encoding="utf-8", errors="ignore") as f:
                    icd02_content = f.read()
                from .icd_completeness_validator import _parse_detailed_markdown_tables
                tables02 = _parse_detailed_markdown_tables(icd02_content)
                for tbl in tables02:
                    for row in tbl.rows:
                        if "signal_id" in row or "signal_name" in row:
                            signals.append(row)
            except Exception:
                pass

        lines: List[str] = [
            "| **Attribute** | **Value** |",
            "| :--- | :--- |",
            "| **Document Title** | OMG UAF Parameter & Measurement Taxonomy Dictionary (Param-Tx) |",
            "| **Document ID** | UAF-PARAM-MEASUREMENT-001 |",
            "| **Standard Alignment** | OMG UAF v1.2 / v2.0 (Param-Tx), ISO 80000-1 & BIPM SI Units |",
            "| **Quality Gate** | Gate 25 (StandardsAndMeasurementValidator) |",
            f"| **Cataloged Signals** | {len(signals)} |",
            "",
            "# Parameter & Measurement Taxonomy Dictionary (Param-Tx)",
            "",
            "## 1. Executive Summary & 7D SI Metrology Formalism",
            "",
            "Formalizes physical parameters and item flow signals in the 7-dimensional SI base exponent space:",
            r"$$D(Q) = [d_L, d_M, d_T, d_I, d_\Theta, d_N, d_J] \in \mathbb{Z}^7$$",
            "",
            "## 2. Master Parameter Metrology Catalog",
            "",
            "| Signal ID | Signal Name | Data Type | SI Units | SI 7D Vector $[d_L, d_M, d_T, d_I, d_\\Theta, d_N, d_J]$ | KaTeX Dimensional Expression | Update Rate |",
            "| :--- | :--- | :--- | :--- | :--- | :--- | :--- |",
        ]

        for sig in sorted(signals, key=lambda s: s.get("signal_id", "")):
            sig_id = sig.get("signal_id", "")
            sig_name = sig.get("signal_name", "")
            dtype = sig.get("data_type", "")
            unit = sig.get("si_units", "") or sig.get("units", "") or sig.get("unit", "")
            rate = sig.get("update_rate", "") or sig.get("rate", "") or "—"

            dim_vec = parse_si_unit(unit)
            vec_str = str(dim_vec.to_list())
            latex_str = f"${dim_vec.to_latex()}$"

            lines.append(f"| `{sig_id}` | {sig_name} | `{dtype}` | `{unit}` | `{vec_str}` | {latex_str} | `{rate}` |")

        if not signals:
            lines.append("| *None* | *None* | *None* | *None* | *None* | *None* | *None* |")

        lines.append("")
        return "\n".join(lines)


# Backwards compatibility alias
StandardsMeasurementValidator = StandardsAndMeasurementValidator


if __name__ == "__main__":
    repo = WorkspaceRepository()
    validator = StandardsAndMeasurementValidator()
    errors = validator.validate(repo)
    if errors:
        for err in errors:
            print(f"[{getattr(err, 'rule_id', 'ERROR')}] {err}")
        sys.exit(1)
    else:
        print("[OK] Gate 25 (StandardsAndMeasurementValidator): All standards and measurement metrology checks passed.")
        sys.exit(0)
