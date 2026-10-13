# Canonical SSOT Architecture Views Library

## Scope and Governance

This directory is the authoritative **Single Source of Truth (SSOT) Architecture Views Library** for the Digital Engineering Autonomous Pipeline (DEAP) framework.

In accordance with IEEE Std 1362-1998 §5.3, DoDAF v2.02 SV-1, and ISO/IEC/IEEE 15288:2023 §6.4.3, all high-level system architecture views, interface matrices, and subsystem topologies are maintained here as canonical Mermaid (`.mmd`) model views to prevent silent diagram divergence across engineering deliverables.

## Canonical Architecture Views

1. **`sv1_super_system.mmd`**: Canonical DoDAF SV-1 / IEEE 1362 §5.3 Super-System Architecture Diagram.
   - Enforces Option 3 compact subsystem blocks with embedded port declarations (`[<b>Name</b><br/>• PORT-... (DIRECTION)]`).
   - Declares vertical hierarchical tier partitioning (`direction TB`) with max 3-column horizontal clusters.
   - Declares dynamic inter-segment vertical rank spacers (`~~~`) to guarantee deterministic top-to-bottom layout.
   - Enforces Rule E5 modularized boundary edge routing: eliminates nested tier subgraphs within segments, routing external edges cleanly between segment boundary components (`CONN-01` through `CONN-08`) without cross-hierarchy collisions (refs #277).
   - Captures canonical operational connection links (`CONN-01` through `CONN-08`).

2. **`sv1_platform_subsystems.mmd`**: Canonical DoDAF SV-1 Platform Subsystems Breakdown.
   - Decomposes processing, sensing, perception, power, and actuation subsystem tiers.
   - Enforces compact block representation and traceable internal bus interconnects (`CONN-01` through `CONN-05`).

3. **`icd_system_interface_matrix.mmd`**: Canonical Level 1C System Interface Matrix Architecture Diagram.
   - Maps physical and logical interface interactions across external actors, supervisory stations, core execution nodes, and support equipment.
   - Maintains strict 1:1 bidirectional parity with ConOps SV-1 connection links (`CONN-01` through `CONN-08`).

## Divergence Prevention Invariant

All specification deliverables, including:
- Concept of Operations (`docs/conops/CONOPS.md` and `docs/conops/units/conops/`)
- Interface Control Documents (`docs/interfaces/` and `docs/icd/`)
- Executive Deliverables & Engineering Reports (`docs/reports/`)

MUST derive directly from or maintain 1:1 bidirectional parity with the canonical views in this library. Authoring independent, desynchronized inline architecture diagrams that diverge in subgraphs, nodes, ports, or connection links violates Check 25 / Check 27 and is mechanically rejected by the baseline verification compiler (refs #276).
