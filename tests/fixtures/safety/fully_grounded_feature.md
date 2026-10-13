---
source_references:
  - schema/model.sysml
---
# Flight Control & Avionics

## Architecture Baseline
<!-- Source: schema/model.sysml -->
The flight controller interfaces with 4 actuators arranged in a cruciform configuration over RS-485.
Telemetry is streamed using MAVLink protocols.
The rail launch acceleration limit is strictly enforced at 12g.
