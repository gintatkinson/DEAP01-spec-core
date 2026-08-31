---
name: Performance Profiling Test Automation
description: Guidelines and standards for automated, repeatable 3D viewport and widget performance profiling to ensure strict 60 FPS compliance.
---

# Performance Profiling Test Automation

## 1. Overview
The purpose of this skill is to establish automated, repeatable performance profiling for 3D viewports and complex widget trees. Ensuring a smooth 60 FPS experience requires continuous verification of rendering pipelines, layout passes, and frame rasterization times to catch regressions early in both Flutter and React codebases.

## 2. Setup Standards
To configure performance profiling:
*   **Flutter**: Add `integration_test` and `flutter_driver` to your `dev_dependencies` in `pubspec.yaml`. Create a driver entry point in `test_driver/integration_test.dart` that initializes the driver (e.g., `integrationDriver()`). Then, write your performance tests in the `integration_test/` directory.
*   **React**: Utilize tools like `react-perf-devtool` or the browser's Performance API within your end-to-end testing framework (like Cypress or Playwright) to capture trace logs during simulated usage.

## 3. Zero-Mocking Persistence (Section 1.9 Compliance)
**Mandatory**: You must adhere to the Zero-Mocking Live Persistence Mandate. In-memory stubs and mocks of the data layer are strictly prohibited for performance profiling because they fail to simulate real-world database access times, serialization overhead, and memory pressures.
*   **Seeding**: At the start of the test, seed the live SQLite/local database with dense, realistic datasets (e.g., 500 ground nodes, 100 space nodes, and 200 network links).
*   **Teardown**: Ensure that the database is completely cleared and resources are released during the test teardown phase.

## 4. Interactive Animation Tracing
To accurately measure UI performance, you must simulate realistic user interactions while tracing.
*   Wrap your aggressive interaction scripts inside `WidgetTester.traceAction()`.
*   Simulate gesture events including pointer zoom scrolls, panning rotations, UI overlay expansions, and tree node iterations that trigger camera fly-to repaints.
*   Ensure that the tracing captures the full lifecycle of animations and repaints triggered by these interactions.

## 5. Frame Budget Assertions
After tracing, extract the timeline summary and enforce the following strict frame budget thresholds to guarantee a 60 FPS baseline:
*   **Average frame build time**: < 16.0ms
*   **90th percentile frame build time**: < 16.6ms
*   **Average frame rasterizer time**: < 16.0ms

Failing these thresholds should immediately fail the regression test.

## 6. Execution Commands
Run the profiling integration tests using the following CLI commands:

**For macOS desktop target:**
```bash
flutter drive \
  --driver=test_driver/integration_test.dart \
  --target=integration_test/viewport_perf_test.dart \
  -d macos \
  --profile
```

**For iOS Simulator target:**
```bash
flutter drive \
  --driver=test_driver/integration_test.dart \
  --target=integration_test/viewport_perf_test.dart \
  -d <simulator_device_id> \
  --profile
```
