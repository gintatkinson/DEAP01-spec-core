---
title: "Implementation Profile — ROS2 C++"
project: "Digital Engineering Agent Platform (DEAP)"
tier: implementation
platform: ros2_cpp
disallowed_technologies:
  - CUDA
  - PyTorch
created: "2026-08-29"
last_updated: "2026-08-29"
---

# Implementation Profile: ROS2 C++

> This document governs feature implementation on ROS2 C++ Real-Time nodes.
> Read alongside `.pipeline/constitution.md` (functional layer).

## Platform & Stack
- Framework: ROS 2 (Humble / Iron / Jazzy) with `rclcpp`
- Language: C++17 / C++20 (GCC / Clang)
- Memory Management: Zero dynamic allocations inside active real-time execution loops (`rttest` verification).
- Communication Safety: Hardened Quality of Service (QoS) profiles (`RELIABILITY_RELIABLE`, `TRANSIENT_LOCAL`).

## Coding Standards
- Lifecycle Nodes: All domain controllers MUST inherit from `rclcpp_lifecycle::LifecycleNode`.
- Naming Conventions: Files — `snake_case.cpp` / `snake_case.hpp`. Classes — `PascalCase`. Topics/Services — `/snake_case`.
- Commercial Toolchain Context: MATLAB / Simulink / Embedded Coder synthesis integration hooks for control law algorithms.

## Testing Mandates
- Unit Tests: `ament_cmake_gtest` / `ament_cmake_pytest`.
- Linting: `ament_lint_auto`, `ament_clang_format`, `ament_cpplint`.
- Real-Time Profiling: Memory lock checks (`mlockall`) and jitter benchmarking.

## Build & Deployment
- Build Command: `colcon build --symlink-install --cmake-args -DCMAKE_BUILD_TYPE=Release`
- Test Command: `colcon test --event-handlers console_direct+`
