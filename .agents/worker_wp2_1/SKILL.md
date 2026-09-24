<!-- Copyright Gint Atkinson, gint.atkinson@gmail.com -->

---
name: feature-driven-implementation
description: "Implements Agile features using serial, subagent-driven, TDD-disciplined execution with two-stage review gates. Use when you need to implement a feature from a GitHub backlog with micro-task decomposition, RED-GREEN-REFACTOR cycles, and automated Epic resolution."
compatibility: "Requires git and configured issue tracker. Works with major agent orchestrators."
metadata:
  risk: low
  source: custom
  version: "2.0"
---

# Feature-Driven Autonomous Delivery & Closure

Use this skill to execute the end-to-end implementation lifecycle for prioritized Agile features and ensure complete automated resolution of feature issues, walkthrough updates, and parent Epics. Resolution means `Fixed / Resolved`; an agent never reaches `Closed`, which requires Product Owner validation (`.pipeline/constitution.md:161`).

This skill integrates subagent-driven development, TDD execution discipline, two-stage review gates, micro-task decomposition, systematic debugging, and verification-before-completion -- ensuring that agents cannot drift, falsely report success, or skip quality gates.
