---
title: "Implementation Profile — PX4 Autopilot Module"
project: "Digital Engineering Agent Platform (DEAP)"
tier: implementation
platform: px4_module
disallowed_technologies:
  - CUDA
  - PyTorch
created: "2026-08-29"
last_updated: "2026-08-29"
---

# Implementation Profile: PX4 Autopilot Module

> This document governs feature implementation on PX4 Autopilot Flight Modules.
> Read alongside `.pipeline/constitution.md` (functional layer).

## Platform & Stack
- Framework: PX4 Autopilot Firmware (v1.14+)
- Messaging: uORB publish/subscribe middleware
- Fail-Safe Management: PX4 Flight Mode Safety Gates (Geofence, Battery, Data Link Loss, Fail-Safe RTL)
- Interface Protocol: MAVLink v2.0 with microRTPS / XRCE-DDS bridge.

## Coding Standards
- Module Structure: Inherit from `ModuleBase<T>` with standard `task_spawn`, `custom_command`, and `print_usage`.
- Safety Net Integration: ASTM F3269-17 Run-Time Assurance (RTA) monitors and failsafe statechart transitions.
- Commercial Toolchain Context: MATLAB / Simulink / Stateflow / Embedded Coder code generation target.

## Testing Mandates
- Unit Tests: PX4 GTest suite (`make tests`).
- SITL Simulation: `make px4_sitl jmavsim` / `make px4_sitl gazebo-classic`.

## Build & Deployment
- Build Command: `make px4_sitl default` (or hardware target `make px4_fmu-v5_default`)
