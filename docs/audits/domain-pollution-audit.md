# Domain Pollution Audit Report

| File Path | Line Number | Snippet | Decontamination / Refactoring Actions |
|---|---|---|---|
| `implementation_plan.md` | 11 | `1.  All files across the entire repository (including `app_flutter/`, `web_react/`, `.pipeline/`, `....` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/features/feat-002-alternate-systems.md` | 20 | `class ReferenceFrame {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/features/feat-002-alternate-systems.md` | 24 | `ReferenceFrame "1" --> "0..1" AlternateSystem : usesAlternateSystem` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/features/feat-12-yang-compiler.md` | 34 | `"yangFile": "ietf-geo-location.yang"` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/feat-hardware-decoupled-persistence-design.md` | 51 | `To avoid the resource overhead of floating-point units (FPUs) in FPGA fabric, dim_0, dim_1, a...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/feat-hardware-decoupled-persistence-design.md` | 61 | `\| `0x04` \| `GEODETIC_SYSTEM` \| R/W \| Bits 1-0: Coordinate Choice (00=Unconfigured, 01=Geometry,...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/feat-hardware-decoupled-persistence-design.md` | 62 | `\| `0x08` \| `COORD_LAT_X` \| R/W \| Dim_0 or Cartesian X (32-bit Q16.16 format) \|` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/feat-hardware-decoupled-persistence-design.md` | 63 | `\| `0x0C` \| `COORD_LON_Y` \| R/W \| Dim_1 or Cartesian Y (32-bit Q16.16 format) \|` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/features/feat-44-downstream-baseline.md` | 63 | `"dim_0": 35.6762,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/features/feat-44-downstream-baseline.md` | 64 | `"dim_1": 139.6503,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/features/feat-44-downstream-baseline.md` | 106 | `- **When** the seeding manager attempts to write a record with a dim_0 of 95.0 (exceeding the sta...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `web_react/src/components/property-grid.tsx` | 11 | `dim_0: 37.7749,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `web_react/src/components/property-grid.tsx` | 12 | `dim_1: -122.4194,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `web_react/src/components/property-grid.tsx` | 24 | `{ key: 'dim_0', label: 'Dim_0', type: 'double', sectionGroup: 'Geometry Coordinate Frame', isR...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `web_react/src/components/property-grid.tsx` | 25 | `{ key: 'dim_1', label: 'Dim_1', type: 'double', sectionGroup: 'Geometry Coordinate Frame', i...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/feat-65-solution.md` | 56 | `Loaded mandated classes dynamically from tmp/test-verify-react/.pipeline/logical-ui/codebase_rules.j...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/feat-65-solution.md` | 57 | `['SlotContainerLocation']` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/feat-g1-g12-solution-definition.md` | 51 | `- Child `EllipsoidCoordinates` containing actual `dim_0`, `dim_1`, and `dim_2`.` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/feat-g1-g12-solution-definition.md` | 52 | `- Child `VelocityVector` containing motion components `vNorth`, `vEast`, `vUp` (enabling dynamic pos...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/feat-g1-g12-solution-definition.md` | 55 | `- Child `ReferenceFrame` (defining the local spatial anchor).` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/feat-g1-g12-solution-definition.md` | 56 | `- Child `EllipsoidCoordinates` mapped to Japan geometry coordinates.` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/feat-g1-g12-solution-definition.md` | 57 | `3.  **Landing Stations (`cable_landing_0` to `cable_landing_X`)**:` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/feat-g1-g12-solution-definition.md` | 65 | ``CoreExchange` \rightarrow `SlotContainerEntity` (with slotContainer dimensions) \rightarrow `SlotContainerPlacement` (slot...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/feat-g1-g12-solution-definition.md` | 79 | `- **Feature #4**: Geometry Coordinates (fetching dim_0, dim_1, dim_2 coordinates)` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/feat-g1-g12-solution-definition.md` | 98 | `- `Position computeCurrentPosition(EllipsoidCoordinates coords, VelocityVector rateOfChange, DateTime ti...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/feat-g1-g12-solution-definition.md` | 99 | `- `double computeSpeed(VelocityVector rateOfChange)`: Calculates rateOfChange magnitude in m/s.` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/feat-g1-g12-solution-definition.md` | 100 | `- `double computeHeading(VelocityVector rateOfChange)`: Calculates azimuth heading in degrees.` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/uml_frontend_alignment_audit.md` | 99 | `+Real dim_0 [1]` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/uml_frontend_alignment_audit.md` | 100 | `+Real dim_1 [1]` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/use-cases/uc-02-local-firebase-emulator.md` | 39 | `1. [SeedingManager](https://github.com/gintatkinson/DEAP-spec-core/blob/main/docs/features/fe...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/feat-epic-template-mandate-plan.md` | 12 | `3. Download the official `ietf-geo-location@2022-02-11.yang` schema from the standard YangModels Git...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/feat-epic-template-mandate-plan.md` | 28 | `+ReferenceFrame referenceFrame` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/feat-epic-template-mandate-plan.md` | 33 | `class ReferenceFrame {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/feat-epic-template-mandate-plan.md` | 46 | `class EllipsoidLocation {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/feat-epic-template-mandate-plan.md` | 47 | `+Dim_0 dim_0` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/feat-epic-template-mandate-plan.md` | 48 | `+Dim_1 dim_1` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/feat-epic-template-mandate-plan.md` | 61 | `GeoLocation *-- ReferenceFrame` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/feat-epic-template-mandate-plan.md` | 62 | `ReferenceFrame *-- GeometrySystem` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/feat-epic-template-mandate-plan.md` | 63 | `LocationChoice <\|-- EllipsoidLocation` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `web_react/src/types.ts` | 124 | `export interface SlotContainerLocation {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `web_react/src/types.ts` | 134 | `location: SlotContainerLocation;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `web_react/src/types.ts` | 137 | `export interface ContainedSlotContainer {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `web_react/src/types.ts` | 141 | `validateSlotOverlap(other: ContainedSlotContainer): boolean;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `web_react/src/types.ts` | 144 | `export interface SlotContainerContainmentSubsystem {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `web_react/src/types.ts` | 145 | `slotContainer: ContainedSlotContainer[];` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/upstream_decontamination_baseline_report.md` | 147 | `-            "dim_0", "dim_1", "trajectory", "orbit",` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/upstream_decontamination_baseline_report.md` | 167 | `-                            forbidden_nodes = {"cartesian", "geometry", "location-choice"}` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/audits/astrodynamics_geodesy_critique.md` | 75 | `Bilinear interpolation is only \mathcal{C}^0 continuous. At grid boundaries, the first derivative ...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/audits/astrodynamics_geodesy_critique.md` | 106 | `The geoid undulation formula h_{geometric} = H_{MSL} + N is applied blindly.` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/audits/astrodynamics_geodesy_critique.md` | 111 | `* GPS / GNSS receivers on aircraft or spacecraft report geometric dim_2 (h) directly.` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/persistence-architecture-blueprint.md` | 100 | `* **Action:** Delete the remaining hardcoded Dart/TS dummy classes (e.g., `RateOfChange`, `PhysicalAddre...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/adversarial_audit_synthesis.md` | 18 | `\| 🟡 **Major**    \| [UML-SEM-01]  \| UML Semantics \| `ReferenceFrame` composition (`*--`) destroys...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/adversarial_audit_synthesis.md` | 20 | `\| 🟡 **Major**    \| [UML-GEO-02]  \| Geometry Model \| `coordAccuracy` and `dim_2Accuracy` static ...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/adversarial_audit_synthesis.md` | 43 | `class ReferenceFrame {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/adversarial_audit_synthesis.md` | 59 | `class Geometry {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/adversarial_audit_synthesis.md` | 60 | `+Decimal64 dim_0 {fractionDigits = 16, range = "-90.0..90.0", units = "degrees"}` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/adversarial_audit_synthesis.md` | 61 | `+Decimal64 dim_1 {fractionDigits = 16, range = "-180.0..180.0", units = "degrees"}` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/adversarial_audit_synthesis.md` | 86 | `GeoLocation "1" --> "1" ReferenceFrame : referenceFrame` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/adversarial_audit_synthesis.md` | 87 | `ReferenceFrame "1" *-- "1" GeometrySystem : geometrySystem` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/adversarial_audit_synthesis.md` | 92 | `Location <\|-- Geometry` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/adversarial_audit_synthesis.md` | 95 | `ReferenceFrame --> AstronomicalBody : astronomicalBody` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/adversarial_audit_synthesis.md` | 100 | `2. **Corrected Lifecycles**: `ReferenceFrame` is associated via direct reference (`-->`) to prevent ...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/adversarial_audit_synthesis.md` | 101 | `3. **No Double-Declaration Redundancy**: Removed object-typed properties (`referenceFrame`, `locatio...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/incident_retrospective.md` | 12 | `- **Mock Use Case Leftover:** A mock Use Case file `docs/use-cases/uc-03-handle-location-expiration....` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/audits/communications_rf_laser_critique.md` | 9 | `## 1. Ray-Sphere & Ray-Geometry Intersection Math` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/audits/communications_rf_laser_critique.md` | 14 | `While this is computationally simple, it is highly inaccurate for geometric bodies like Earth (WGS...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/audits/communications_rf_laser_critique.md` | 21 | `### 1.2. Exact Ray-Geometry Intersection Formulation` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/audits/communications_rf_laser_critique.md` | 22 | `For any planet or moon modeled as a triaxial or biaxial geometry centered at \mathbf{c}_k with se...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/audits/communications_rf_laser_critique.md` | 31 | `Substituting the ray into the geometry equation yields:` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/audits/communications_rf_laser_critique.md` | 44 | `Because A > 0, t_1 \le t_2. The line segment [0, 1] intersects the geometry if and only if:` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/audits/communications_rf_laser_critique.md` | 179 | `### 6.2. GPU (WGSL) Ray-Geometry Occlusion Code` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/audits/communications_rf_laser_critique.md` | 180 | `Integrate this exact, branch-optimized ray-geometry occlusion algorithm into the compute shader:` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/audits/communications_rf_laser_critique.md` | 183 | `struct Geometry {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/audits/communications_rf_laser_critique.md` | 188 | `// Returns true if ray from start to end is occluded by the geometry` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/audits/communications_rf_laser_critique.md` | 189 | `fn check_ellipsoid_occlusion(start: vec3<f32>, end: vec3<f32>, body: Geometry) -> bool {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/feat-44-solution.md` | 72 | `\| `SlotContainerLocation` \| `@realizes UML::SlotContainerLocation` \| [types.ts](web_react/src/types.ts)...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/feat-44-solution.md` | 73 | `\| \| `@realizes UML::SlotContainerLocation` \| [types.dart](app_flutter/lib/domain/types.dart)...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/feat-44-solution.md` | 76 | `\| `ContainedSlotContainer` \| `@realizes UML::ContainedSlotContainer` \| [types.ts](web_react/src/types.ts)...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/feat-44-solution.md` | 77 | `\| \| `@realizes UML::ContainedSlotContainer` \| [types.dart](app_flutter/lib/domain/types.dart)...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/feat-44-solution.md` | 78 | `\| `SlotContainerContainmentSubsystem` \| `@realizes UML::SlotContainerContainmentSubsystem` \| [types.ts](web_react/src/types.ts)...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/feat-44-solution.md` | 79 | `\| \| `@realizes UML::SlotContainerContainmentSubsystem` \| [types.dart](app_flutter/lib/domain/types.dart)...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/audits/pipeline_integration_critique.md` | 125 | `- Therefore, if the schema defines multiple trigger nodes (e.g., `dim_0`, `dim_1`, `dim_2`...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/audits/pipeline_integration_critique.md` | 131 | `B --> C{Contains any of Dim_0/Dim_1/Dim_2/etc.?}` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/feat-backprop-flutter-source-changes.md` | 14 | `- [camera_controller.dart](app_flutter/lib/domain/camera_controller.dart)...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/designs/feat-backprop-flutter-source-changes.md` | 24 | `- [globe_camera_drag_test.dart](app_flutter/integration_test/globe_camera_drag_test.dart)...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 5 | `"dim_0": 40.8232978,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 6 | `"dim_1": 140.7503634` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 11 | `"dim_0": 35.8531756,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 12 | `"dim_1": 139.3298997` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 17 | `"dim_0": 35.6181937,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 18 | `"dim_1": 139.626029` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 23 | `"dim_0": 35.6608225,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 24 | `"dim_1": 138.5724577` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 29 | `"dim_0": 24.3424146,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 30 | `"dim_1": 124.1543772` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 35 | `"dim_0": 33.5444481,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 36 | `"dim_1": 130.4619739` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 41 | `"dim_0": 35.6516172,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 42 | `"dim_1": 139.7041546` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 47 | `"dim_0": 36.6814747,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 48 | `"dim_1": 137.2366087` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 53 | `"dim_0": 35.7048503,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 54 | `"dim_1": 139.5798226` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 59 | `"dim_0": 35.7033569,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 60 | `"dim_1": 139.5788071` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 65 | `"dim_0": 35.4618644,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 66 | `"dim_1": 139.5114218` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 71 | `"dim_0": 36.6743842,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 72 | `"dim_1": 136.8681352` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 77 | `"dim_0": 35.6194779,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 78 | `"dim_1": 138.4648227` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 83 | `"dim_0": 43.0558424,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 84 | `"dim_1": 141.3344281` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 89 | `"dim_0": 35.961876,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 90 | `"dim_1": 140.635629` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 95 | `"dim_0": 36.5374547,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 96 | `"dim_1": 140.5294505` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 101 | `"dim_0": 33.9653364,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 102 | `"dim_1": 132.1107071` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 107 | `"dim_0": 32.019024,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 108 | `"dim_1": 130.194179` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 113 | `"dim_0": 35.631017,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 114 | `"dim_1": 139.725482` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 119 | `"dim_0": 35.683423,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 120 | `"dim_1": 139.687889` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 125 | `"dim_0": 35.671099,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 126 | `"dim_1": 139.757656` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 131 | `"dim_0": 35.090013,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 132 | `"dim_1": 138.9554277` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 137 | `"dim_0": 34.397632,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 138 | `"dim_1": 132.456619` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 143 | `"dim_0": 33.843701,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 144 | `"dim_1": 132.773435` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 149 | `"dim_0": 33.887351,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 150 | `"dim_1": 130.901483` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 155 | `"dim_0": 32.801067,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 156 | `"dim_1": 130.718274` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 161 | `"dim_0": 35.6666239,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 162 | `"dim_1": 138.5691456` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 167 | `"dim_0": 34.9883169,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 168 | `"dim_1": 133.4605143` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 173 | `"dim_0": 36.3186401,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 174 | `"dim_1": 139.1978382` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 179 | `"dim_0": 35.9383964,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 180 | `"dim_1": 140.5466896` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 185 | `"dim_0": 35.9856102,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 186 | `"dim_1": 140.4895069` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 191 | `"dim_0": 33.8911307,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 192 | `"dim_1": 130.767468` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 197 | `"dim_0": 35.4318097,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 198 | `"dim_1": 139.4106549` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 203 | `"dim_0": 35.431918,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 204 | `"dim_1": 139.4102092` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 209 | `"dim_0": 35.43209,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 210 | `"dim_1": 139.409513` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 215 | `"dim_0": 35.9973876,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 216 | `"dim_1": 138.1426388` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 221 | `"dim_0": 35.9986399,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 222 | `"dim_1": 138.1431585` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 227 | `"dim_0": 35.703706,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 228 | `"dim_1": 139.5606279` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 233 | `"dim_0": 35.4993051,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 234 | `"dim_1": 135.7461386` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 239 | `"dim_0": 34.9502825,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 240 | `"dim_1": 138.3548617` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 245 | `"dim_0": 34.9585307,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 246 | `"dim_1": 138.3532068` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 251 | `"dim_0": 39.2662115,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 252 | `"dim_1": 141.8482356` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 257 | `"dim_0": 35.065998,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 258 | `"dim_1": 138.2874892` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 263 | `"dim_0": 35.7354408,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 264 | `"dim_1": 139.787435` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 269 | `"dim_0": 35.7378366,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 270 | `"dim_1": 139.327122` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 275 | `"dim_0": 26.438728,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 276 | `"dim_1": 127.8021718` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 281 | `"dim_0": 35.1447925,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 282 | `"dim_1": 136.9000811` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 287 | `"dim_0": 35.6872668,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 288 | `"dim_1": 139.5971731` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 293 | `"dim_0": 34.6874147,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 294 | `"dim_1": 135.5507858` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 299 | `"dim_0": 35.669079,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 300 | `"dim_1": 139.725426` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 305 | `"dim_0": 35.5872215,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 306 | `"dim_1": 139.7315776` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 311 | `"dim_0": 36.131513,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 312 | `"dim_1": 140.0838561` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 317 | `"dim_0": 35.9024038,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 318 | `"dim_1": 139.5185908` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 323 | `"dim_0": 35.4549513,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 324 | `"dim_1": 139.6301061` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 329 | `"dim_0": 35.7026905,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 330 | `"dim_1": 139.7761707` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 335 | `"dim_0": 35.465288,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 336 | `"dim_1": 139.6200662` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 341 | `"dim_0": 35.6905033,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 342 | `"dim_1": 139.7041382` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 347 | `"dim_0": 34.703049,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 348 | `"dim_1": 135.546082` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 353 | `"dim_0": 34.909868,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 354 | `"dim_1": 137.4208282` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 359 | `"dim_0": 38.3176959,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 360 | `"dim_1": 140.6327578` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 365 | `"dim_0": 35.5865433,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 366 | `"dim_1": 139.7319506` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 371 | `"dim_0": 35.6780082,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 372 | `"dim_1": 138.5544683` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 377 | `"dim_0": 35.61449,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 378 | `"dim_1": 139.627532` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 383 | `"dim_0": 43.2011119,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 384 | `"dim_1": 141.7664742` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 389 | `"dim_0": 35.6978016,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 390 | `"dim_1": 139.7603133` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 395 | `"dim_0": 35.6836682,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 396 | `"dim_1": 139.5596113` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 401 | `"dim_0": 38.2698868,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 402 | `"dim_1": 140.8739382` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 407 | `"dim_0": 34.6720542,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 408 | `"dim_1": 133.9113646` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 413 | `"dim_0": 35.6885077,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 414 | `"dim_1": 139.5669273` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 419 | `"dim_0": 34.681893,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 420 | `"dim_1": 135.824998` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 425 | `"dim_0": 34.7229371,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 426 | `"dim_1": 135.5497665` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 431 | `"dim_0": 34.6982087,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 432 | `"dim_1": 135.5033866` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 437 | `"dim_0": 41.8759359,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 438 | `"dim_1": 140.9465037` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 443 | `"dim_0": 41.7733707,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 444 | `"dim_1": 140.7395509` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 449 | `"dim_0": 41.7889742,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 450 | `"dim_1": 140.7620157` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 455 | `"dim_0": 37.4788242,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 456 | `"dim_1": 139.9827588` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 461 | `"dim_0": 35.687556,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 462 | `"dim_1": 139.5687762` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 467 | `"dim_0": 35.6878418,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 468 | `"dim_1": 139.5689773` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 473 | `"dim_0": 35.6318985,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 474 | `"dim_1": 139.7254729` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 479 | `"dim_0": 34.9707804,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 480 | `"dim_1": 134.8091935` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 485 | `"dim_0": 35.583326,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 486 | `"dim_1": 139.658096` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 491 | `"dim_0": 35.6943215,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 492 | `"dim_1": 139.5613669` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 497 | `"dim_0": 33.6366968,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 498 | `"dim_1": 130.4441284` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 503 | `"dim_0": 33.6487306,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 504 | `"dim_1": 130.4254193` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 509 | `"dim_0": 35.3692143,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 510 | `"dim_1": 139.5639146` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 515 | `"dim_0": 34.665264,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 516 | `"dim_1": 135.496332` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 521 | `"dim_0": 43.081131,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 522 | `"dim_1": 141.307631` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 527 | `"dim_0": 43.081304,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 528 | `"dim_1": 141.306837` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 533 | `"dim_0": 42.3444724,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 534 | `"dim_1": 141.0301402` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 539 | `"dim_0": 36.2373376,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 540 | `"dim_1": 137.9699587` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 545 | `"dim_0": 35.6495671,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 546 | `"dim_1": 138.7212006` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 551 | `"dim_0": 33.6896934,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 552 | `"dim_1": 130.4079061` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 557 | `"dim_0": 39.069846,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 558 | `"dim_1": 141.7195817` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 563 | `"dim_0": 35.7128536,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 564 | `"dim_1": 139.7920868` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 569 | `"dim_0": 35.6498834,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 570 | `"dim_1": 139.9036642` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 575 | `"dim_0": 35.6508004,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 576 | `"dim_1": 139.5884369` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 581 | `"dim_0": 34.2033698,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 582 | `"dim_1": 133.110778` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 587 | `"dim_0": 33.8364233,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 588 | `"dim_1": 132.7377185` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 593 | `"dim_0": 34.816145,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 594 | `"dim_1": 135.648006` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 599 | `"dim_0": 35.4435519,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 600 | `"dim_1": 139.6427383` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 605 | `"dim_0": 43.161399,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 606 | `"dim_1": 141.413348` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 611 | `"dim_0": 43.0662656,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 612 | `"dim_1": 141.347764` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 617 | `"dim_0": 35.6505907,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 618 | `"dim_1": 139.5885565` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 623 | `"dim_0": 43.074893,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 624 | `"dim_1": 141.296016` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 629 | `"dim_0": 43.067643,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 630 | `"dim_1": 141.274229` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 635 | `"dim_0": 34.4649273,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 636 | `"dim_1": 135.737724` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 641 | `"dim_0": 36.3774128,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 642 | `"dim_1": 140.4694164` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 647 | `"dim_0": 35.741877,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 648 | `"dim_1": 136.947447` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 653 | `"dim_0": 35.445958,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 654 | `"dim_1": 137.019196` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 659 | `"dim_0": 37.422941,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 660 | `"dim_1": 140.3516437` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 665 | `"dim_0": 35.707605,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 666 | `"dim_1": 139.772868` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 671 | `"dim_0": 35.9528489,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 672 | `"dim_1": 139.6664989` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 677 | `"dim_0": 34.6611737,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 678 | `"dim_1": 135.5576237` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 683 | `"dim_0": 34.6895639,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 684 | `"dim_1": 135.526578` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 689 | `"dim_0": 38.6015143,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 690 | `"dim_1": 141.0223522` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 695 | `"dim_0": 35.7837052,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 696 | `"dim_1": 139.0299334` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 701 | `"dim_0": 39.4003926,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 702 | `"dim_1": 141.9177101` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 707 | `"dim_0": 34.3143649,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 708 | `"dim_1": 135.609275` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 713 | `"dim_0": 34.3131599,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 714 | `"dim_1": 135.6087609` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 719 | `"dim_0": 34.702675,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 720 | `"dim_1": 135.5624` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 725 | `"dim_0": 34.702892,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 726 | `"dim_1": 135.564727` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 731 | `"dim_0": 34.70305,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 732 | `"dim_1": 135.569342` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 737 | `"dim_0": 34.6632082,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 738 | `"dim_1": 133.9264179` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 743 | `"dim_0": 34.8912452,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 744 | `"dim_1": 139.036261` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 749 | `"dim_0": 34.6629743,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 750 | `"dim_1": 133.9260612` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 755 | `"dim_0": 34.6632333,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 756 | `"dim_1": 133.9260638` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 761 | `"dim_0": 35.5420691,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 762 | `"dim_1": 134.8171682` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 767 | `"dim_0": 34.6983542,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 768 | `"dim_1": 135.5600928` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 773 | `"dim_0": 35.022963,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 774 | `"dim_1": 137.0883736` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 779 | `"dim_0": 35.627857,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 780 | `"dim_1": 139.448368` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 785 | `"dim_0": 35.6231045,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 786 | `"dim_1": 139.4443515` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 791 | `"dim_0": 35.62203,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 792 | `"dim_1": 139.448439` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 797 | `"dim_0": 35.621301,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 798 | `"dim_1": 139.447919` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 803 | `"dim_0": 35.621736,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 804 | `"dim_1": 139.454109` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 809 | `"dim_0": 35.6207542,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 810 | `"dim_1": 139.453825` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 815 | `"dim_0": 36.3177156,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 816 | `"dim_1": 139.8075803` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 821 | `"dim_0": 35.0355248,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 822 | `"dim_1": 137.0793849` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 827 | `"dim_0": 34.6394724,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 828 | `"dim_1": 135.538722` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 833 | `"dim_0": 35.5873824,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 834 | `"dim_1": 139.7318651` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 839 | `"dim_0": 36.8326542,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 840 | `"dim_1": 139.7165299` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 845 | `"dim_0": 36.8324524,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 846 | `"dim_1": 139.716589` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 851 | `"dim_0": 34.706748,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 852 | `"dim_1": 135.565527` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 857 | `"dim_0": 40.587143,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 858 | `"dim_1": 140.399528` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 863 | `"dim_0": 34.8835811,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 864 | `"dim_1": 136.5848024` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 869 | `"dim_0": 34.8809094,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 870 | `"dim_1": 136.5851254` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 875 | `"dim_0": 34.7705973,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 876 | `"dim_1": 138.0142349` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 881 | `"dim_0": 35.635247,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 882 | `"dim_1": 139.4434658` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 887 | `"dim_0": 32.5817171,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 888 | `"dim_1": 131.6670691` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 893 | `"dim_0": 34.9571806,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 894 | `"dim_1": 137.1671518` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 899 | `"dim_0": 36.3777276,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 900 | `"dim_1": 139.7341856` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 905 | `"dim_0": 36.3778023,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 906 | `"dim_1": 139.7341629` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 911 | `"dim_0": 43.2302432,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 912 | `"dim_1": 143.2931095` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 917 | `"dim_0": 34.7030391,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 918 | `"dim_1": 135.6347653` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 923 | `"dim_0": 35.7223011,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 924 | `"dim_1": 139.6742332` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 929 | `"dim_0": 33.748058,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 930 | `"dim_1": 129.689912` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 935 | `"dim_0": 38.2426035,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 936 | `"dim_1": 140.9089403` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 941 | `"dim_0": 38.2759322,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 942 | `"dim_1": 140.8678459` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 947 | `"dim_0": 35.6312558,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 948 | `"dim_1": 139.7256288` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 953 | `"dim_0": 38.2542392,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 954 | `"dim_1": 140.8974716` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 959 | `"dim_0": 38.249536,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 960 | `"dim_1": 140.9025306` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 965 | `"dim_0": 38.249186,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 966 | `"dim_1": 140.9096622` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 971 | `"dim_0": 38.2455736,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 972 | `"dim_1": 140.9095308` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 977 | `"dim_0": 38.3081053,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 978 | `"dim_1": 140.8309344` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 983 | `"dim_0": 38.046232,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 984 | `"dim_1": 140.7185234` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 989 | `"dim_0": 38.2517041,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 990 | `"dim_1": 140.9186167` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 995 | `"dim_0": 38.1946404,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 996 | `"dim_1": 140.8826319` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1001 | `"dim_0": 37.8649754,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1002 | `"dim_1": 139.1103323` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1007 | `"dim_0": 37.8646959,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1008 | `"dim_1": 139.1107749` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1013 | `"dim_0": 34.8665155,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1014 | `"dim_1": 137.0968657` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1019 | `"dim_0": 35.7551652,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1020 | `"dim_1": 139.6494327` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1025 | `"dim_0": 38.2538806,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1026 | `"dim_1": 140.9029521` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1031 | `"dim_0": 38.2398608,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1032 | `"dim_1": 140.869198` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1037 | `"dim_0": 34.6853434,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1038 | `"dim_1": 135.506705` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1043 | `"dim_0": 32.4631506,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1044 | `"dim_1": 139.7605581` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1049 | `"dim_0": 34.8881105,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1050 | `"dim_1": 135.8030678` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1055 | `"dim_0": 43.059672,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1056 | `"dim_1": 141.335903` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1061 | `"dim_0": 38.2610967,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1062 | `"dim_1": 140.8960947` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1067 | `"dim_0": 38.2536407,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1068 | `"dim_1": 140.881987` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1073 | `"dim_0": 38.2509821,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1074 | `"dim_1": 140.3362594` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1079 | `"dim_0": 38.2500152,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1080 | `"dim_1": 140.3096648` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1085 | `"dim_0": 38.7594893,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1086 | `"dim_1": 140.3030128` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1091 | `"dim_0": 34.6992466,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1092 | `"dim_1": 135.4979477` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1097 | `"dim_0": 34.6999283,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1098 | `"dim_1": 135.4982139` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1103 | `"dim_0": 34.6994549,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1104 | `"dim_1": 135.4990761` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1109 | `"dim_0": 34.6998568,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1110 | `"dim_1": 135.4995045` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1115 | `"dim_0": 35.692197,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1116 | `"dim_1": 139.7402319` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1121 | `"dim_0": 35.6931016,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1122 | `"dim_1": 139.7432137` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1127 | `"dim_0": 35.6868165,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1128 | `"dim_1": 139.7417417` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1133 | `"dim_0": 35.7033848,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1134 | `"dim_1": 139.747336` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1139 | `"dim_0": 35.0915594,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1140 | `"dim_1": 138.9556263` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1145 | `"dim_0": 35.0432895,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1146 | `"dim_1": 137.0383507` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1151 | `"dim_0": 34.989451,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1152 | `"dim_1": 136.9998657` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1157 | `"dim_0": 34.6987675,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1158 | `"dim_1": 135.4975385` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1163 | `"dim_0": 38.0491958,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1164 | `"dim_1": 140.1633021` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1169 | `"dim_0": 35.733025,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1170 | `"dim_1": 139.716576` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1175 | `"dim_0": 35.6911426,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1176 | `"dim_1": 139.743325` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1181 | `"dim_0": 35.7274913,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1182 | `"dim_1": 139.7164829` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1187 | `"dim_0": 35.7068932,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1188 | `"dim_1": 139.7728638` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1193 | `"dim_0": 35.6734397,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1194 | `"dim_1": 139.7269513` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1199 | `"dim_0": 43.7541794,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1200 | `"dim_1": 142.3987172` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1205 | `"dim_0": 38.2606643,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1206 | `"dim_1": 140.9258492` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1211 | `"dim_0": 35.6927208,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1212 | `"dim_1": 139.7413779` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1217 | `"dim_0": 35.0975631,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1218 | `"dim_1": 137.0522347` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1223 | `"dim_0": 35.2254068,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1224 | `"dim_1": 139.6633476` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1229 | `"dim_0": 38.6171219,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1230 | `"dim_1": 139.601807` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1235 | `"dim_0": 38.6171696,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1236 | `"dim_1": 139.6024045` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1241 | `"dim_0": 34.9796985,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1242 | `"dim_1": 138.9452187` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1247 | `"dim_0": 34.7047733,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1248 | `"dim_1": 135.4989987` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1253 | `"dim_0": 35.8304856,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1254 | `"dim_1": 139.5620996` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1259 | `"dim_0": 34.7398302,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1260 | `"dim_1": 136.8712352` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1265 | `"dim_0": 24.348551,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1266 | `"dim_1": 124.1577972` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1271 | `"dim_0": 38.2694404,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1272 | `"dim_1": 140.893314` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1277 | `"dim_0": 35.6908802,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1278 | `"dim_1": 139.7842408` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1283 | `"dim_0": 34.6528424,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1284 | `"dim_1": 134.0272067` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1289 | `"dim_0": 34.780384,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1290 | `"dim_1": 137.7386209` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1295 | `"dim_0": 35.8120991,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1296 | `"dim_1": 139.3620811` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1301 | `"dim_0": 36.9560219,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1302 | `"dim_1": 137.5584725` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1307 | `"dim_0": 35.7050543,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1308 | `"dim_1": 139.7544671` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1313 | `"dim_0": 34.7420517,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1314 | `"dim_1": 135.5464796` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1319 | `"dim_0": 34.7443685,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1320 | `"dim_1": 135.5429958` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1325 | `"dim_0": 34.5305377,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1326 | `"dim_1": 135.4658581` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1331 | `"dim_0": 36.0951704,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1332 | `"dim_1": 133.0950308` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1337 | `"dim_0": 36.6896894,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1338 | `"dim_1": 137.2125044` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1343 | `"dim_0": 35.6973638,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1344 | `"dim_1": 139.813326` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1349 | `"dim_0": 35.7934278,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1350 | `"dim_1": 139.7970189` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1355 | `"dim_0": 35.7510811,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1356 | `"dim_1": 139.5942139` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1361 | `"dim_0": 35.5178063,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1362 | `"dim_1": 139.4731849` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1367 | `"dim_0": 37.1649062,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1368 | `"dim_1": 138.2377186` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1373 | `"dim_0": 35.3785803,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1374 | `"dim_1": 139.918189` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1379 | `"dim_0": 35.4418378,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1380 | `"dim_1": 136.7602386` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1385 | `"dim_0": 35.65787,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1386 | `"dim_1": 139.7495878` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1391 | `"dim_0": 36.6990026,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1392 | `"dim_1": 137.8642206` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1397 | `"dim_0": 36.7120457,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1398 | `"dim_1": 137.1024221` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1403 | `"dim_0": 37.8481981,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1404 | `"dim_1": 136.9162818` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1409 | `"dim_0": 35.6896966,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1410 | `"dim_1": 139.785016` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1415 | `"dim_0": 35.7331255,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1416 | `"dim_1": 139.7101992` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1421 | `"dim_0": 35.6946225,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1422 | `"dim_1": 139.7518292` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1427 | `"dim_0": 35.6966853,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1428 | `"dim_1": 139.8945566` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1433 | `"dim_0": 34.7376965,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1434 | `"dim_1": 136.5177219` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1439 | `"dim_0": 34.500848,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1440 | `"dim_1": 135.5989229` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1445 | `"dim_0": 36.7451291,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1446 | `"dim_1": 137.1895178` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1451 | `"dim_0": 34.7046967,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1452 | `"dim_1": 135.5030816` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1457 | `"dim_0": 36.3687273,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1458 | `"dim_1": 140.3589946` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1463 | `"dim_0": 38.2209819,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1464 | `"dim_1": 139.4738401` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1469 | `"dim_0": 34.3970515,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1470 | `"dim_1": 133.2007559` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1475 | `"dim_0": 40.5146357,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1476 | `"dim_1": 141.4996037` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1481 | `"dim_0": 40.5146819,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1482 | `"dim_1": 141.4999586` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1487 | `"dim_0": 35.4135253,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1488 | `"dim_1": 136.737327` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1493 | `"dim_0": 34.8446152,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1494 | `"dim_1": 135.5813454` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1499 | `"dim_0": 34.7332293,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1500 | `"dim_1": 136.5159589` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1505 | `"dim_0": 34.5477244,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1506 | `"dim_1": 136.9780742` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1511 | `"dim_0": 35.429548,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1512 | `"dim_1": 139.6445276` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1517 | `"dim_0": 35.6857047,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1518 | `"dim_1": 139.7769959` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1523 | `"dim_0": 34.8929481,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1524 | `"dim_1": 133.6825804` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1529 | `"dim_0": 35.6897399,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1530 | `"dim_1": 139.7665734` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1535 | `"dim_0": 35.6828186,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1536 | `"dim_1": 139.7725546` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1541 | `"dim_0": 36.3792662,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1542 | `"dim_1": 140.4683509` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1547 | `"dim_0": 36.3792856,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1548 | `"dim_1": 140.4685548` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1553 | `"dim_0": 35.3781095,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1554 | `"dim_1": 139.9185303` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1559 | `"dim_0": 35.3780592,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1560 | `"dim_1": 139.9186456` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1565 | `"dim_0": 35.6557646,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1566 | `"dim_1": 139.680962` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1571 | `"dim_0": 35.615105,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1572 | `"dim_1": 139.6759493` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1577 | `"dim_0": 33.5822012,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1578 | `"dim_1": 130.2644689` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1583 | `"dim_0": 33.5443271,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1584 | `"dim_1": 130.3146924` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1589 | `"dim_0": 38.2529837,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1590 | `"dim_1": 140.8817666` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1595 | `"dim_0": 35.3558576,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1596 | `"dim_1": 137.0926063` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1601 | `"dim_0": 34.5498165,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1602 | `"dim_1": 135.513344` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1607 | `"dim_0": 34.5475185,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1608 | `"dim_1": 135.5143447` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1613 | `"dim_0": 38.2582319,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1614 | `"dim_1": 140.8708747` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1619 | `"dim_0": 38.258302,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1620 | `"dim_1": 140.8710161` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1625 | `"dim_0": 34.5659446,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1626 | `"dim_1": 135.5236751` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1631 | `"dim_0": 35.6836647,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1632 | `"dim_1": 139.5589869` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1637 | `"dim_0": 35.6836673,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1638 | `"dim_1": 139.5588956` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1643 | `"dim_0": 35.7021128,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1644 | `"dim_1": 139.5767357` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1649 | `"dim_0": 35.6494055,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1650 | `"dim_1": 139.9047687` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1655 | `"dim_0": 35.6500921,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1656 | `"dim_1": 139.9037571` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1661 | `"dim_0": 35.7021209,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1662 | `"dim_1": 139.5611212` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1667 | `"dim_0": 35.681596,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1668 | `"dim_1": 139.5535984` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1673 | `"dim_0": 34.5714557,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1674 | `"dim_1": 135.6178166` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1679 | `"dim_0": 35.6835115,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1680 | `"dim_1": 139.7841021` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1685 | `"dim_0": 35.6835038,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1686 | `"dim_1": 139.78356` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1691 | `"dim_0": 38.239925,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1692 | `"dim_1": 140.851627` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1697 | `"dim_0": 35.7088262,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1698 | `"dim_1": 139.7955202` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1703 | `"dim_0": 35.7094115,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1704 | `"dim_1": 139.791463` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1709 | `"dim_0": 35.6849511,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1710 | `"dim_1": 139.5832065` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1715 | `"dim_0": 35.6832426,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1716 | `"dim_1": 139.5633488` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1721 | `"dim_0": 35.6995616,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1722 | `"dim_1": 139.5742849` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1727 | `"dim_0": 35.699267,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1728 | `"dim_1": 139.5768383` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1733 | `"dim_0": 35.6850459,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1734 | `"dim_1": 139.5583257` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1739 | `"dim_0": 38.2407413,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1740 | `"dim_1": 140.8597844` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1745 | `"dim_0": 35.7226754,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1746 | `"dim_1": 139.7031563` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1751 | `"dim_0": 38.0583552,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1752 | `"dim_1": 140.7728404` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1757 | `"dim_0": 34.6697826,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1758 | `"dim_1": 133.9119539` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1763 | `"dim_0": 32.460665,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1764 | `"dim_1": 131.1519286` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1769 | `"dim_0": 35.736589,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1770 | `"dim_1": 139.8800715` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1775 | `"dim_0": 35.6935142,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1776 | `"dim_1": 139.5613895` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1781 | `"dim_0": 35.6695405,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1782 | `"dim_1": 139.5558111` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1787 | `"dim_0": 36.394964,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1788 | `"dim_1": 140.5321634` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1793 | `"dim_0": 36.3947351,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1794 | `"dim_1": 140.5321205` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1799 | `"dim_0": 35.7179341,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1800 | `"dim_1": 139.5667519` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1805 | `"dim_0": 35.4129092,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1806 | `"dim_1": 134.2536703` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1811 | `"dim_0": 34.4007242,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1812 | `"dim_1": 132.4615317` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1817 | `"dim_0": 38.227668,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1818 | `"dim_1": 140.8951263` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1823 | `"dim_0": 43.056442,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1824 | `"dim_1": 141.33425` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1829 | `"dim_0": 34.5468595,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1830 | `"dim_1": 135.5183754` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1835 | `"dim_0": 34.5594958,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1836 | `"dim_1": 135.5153317` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1841 | `"dim_0": 34.5413345,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1842 | `"dim_1": 133.7738168` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1847 | `"dim_0": 34.9169703,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1848 | `"dim_1": 135.6872047` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1853 | `"dim_0": 34.5653541,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1854 | `"dim_1": 135.5173854` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1859 | `"dim_0": 34.5638341,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1860 | `"dim_1": 135.5244332` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1865 | `"dim_0": 34.5774892,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1866 | `"dim_1": 135.4764334` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1871 | `"dim_0": 34.5785612,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1872 | `"dim_1": 135.4744773` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1877 | `"dim_0": 34.5873672,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1878 | `"dim_1": 135.4823384` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1883 | `"dim_0": 34.5849995,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1884 | `"dim_1": 135.4801314` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1889 | `"dim_0": 34.583037,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1890 | `"dim_1": 135.4783066` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1895 | `"dim_0": 38.2400306,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1896 | `"dim_1": 140.851439` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1901 | `"dim_0": 38.2540529,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1902 | `"dim_1": 140.8817363` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1907 | `"dim_0": 37.4781371,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1908 | `"dim_1": 138.9948782` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1913 | `"dim_0": 38.1114885,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1914 | `"dim_1": 140.8703928` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1919 | `"dim_0": 35.4382323,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1920 | `"dim_1": 139.3078558` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1925 | `"dim_0": 35.6239759,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1926 | `"dim_1": 135.0627154` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1931 | `"dim_0": 35.917722,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1932 | `"dim_1": 139.7820863` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1937 | `"dim_0": 35.9179131,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1938 | `"dim_1": 139.7822741` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1943 | `"dim_0": 38.1953698,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1944 | `"dim_1": 140.9203161` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1949 | `"dim_0": 34.5734445,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1950 | `"dim_1": 135.4720626` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1955 | `"dim_0": 34.5746532,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1956 | `"dim_1": 135.4729912` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1961 | `"dim_0": 34.5808325,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1962 | `"dim_1": 135.4787052` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1967 | `"dim_0": 35.605239,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1968 | `"dim_1": 139.5068262` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1973 | `"dim_0": 43.0454044,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1974 | `"dim_1": 141.3800364` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1979 | `"dim_0": 38.2206846,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1980 | `"dim_1": 140.8106292` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1985 | `"dim_0": 35.9370958,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1986 | `"dim_1": 139.8189479` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1991 | `"dim_0": 35.9378971,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1992 | `"dim_1": 139.8186207` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1997 | `"dim_0": 36.7306227,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 1998 | `"dim_1": 137.1836122` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2003 | `"dim_0": 38.2738924,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2004 | `"dim_1": 140.7642756` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2009 | `"dim_0": 38.2754543,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2010 | `"dim_1": 140.7499637` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2015 | `"dim_0": 35.7183325,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2016 | `"dim_1": 139.5625404` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2021 | `"dim_0": 35.7189486,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2022 | `"dim_1": 139.562563` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2027 | `"dim_0": 39.717334,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2028 | `"dim_1": 141.1424017` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2033 | `"dim_0": 38.0492491,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2034 | `"dim_1": 140.7342146` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2039 | `"dim_0": 34.5407545,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2040 | `"dim_1": 135.5206748` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2045 | `"dim_0": 38.0048378,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2046 | `"dim_1": 140.6206456` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2051 | `"dim_0": 37.9951812,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2052 | `"dim_1": 140.4416469` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2057 | `"dim_0": 38.1002033,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2058 | `"dim_1": 140.8562953` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2063 | `"dim_0": 38.1191978,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2064 | `"dim_1": 140.8734462` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2069 | `"dim_0": 38.1028108,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2070 | `"dim_1": 140.9121636` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2075 | `"dim_0": 38.1240575,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2076 | `"dim_1": 140.9021693` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2081 | `"dim_0": 35.6269208,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2082 | `"dim_1": 139.5731826` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2087 | `"dim_0": 38.3570085,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2088 | `"dim_1": 140.8547299` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2093 | `"dim_0": 34.5818373,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2094 | `"dim_1": 135.5089142` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2099 | `"dim_0": 38.6881385,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2100 | `"dim_1": 141.194612` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2105 | `"dim_0": 34.3252161,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2106 | `"dim_1": 134.0446629` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2111 | `"dim_0": 34.5597242,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2112 | `"dim_1": 135.4723619` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2117 | `"dim_0": 38.2577154,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2118 | `"dim_1": 140.8703943` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2123 | `"dim_0": 38.2521045,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2124 | `"dim_1": 140.8814242` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2129 | `"dim_0": 34.5805485,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2130 | `"dim_1": 135.463541` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2135 | `"dim_0": 33.3595692,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2136 | `"dim_1": 130.7818495` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2141 | `"dim_0": 34.5551746,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2142 | `"dim_1": 135.5059118` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2147 | `"dim_0": 38.2389564,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2148 | `"dim_1": 140.9027237` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2153 | `"dim_0": 33.3567745,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2154 | `"dim_1": 130.7544662` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2159 | `"dim_0": 33.3397478,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2160 | `"dim_1": 130.7601558` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2165 | `"dim_0": 33.3534135,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2166 | `"dim_1": 130.7347649` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2171 | `"dim_0": 38.8425202,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2172 | `"dim_1": 141.5799934` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2177 | `"dim_0": 38.2716127,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2178 | `"dim_1": 140.7672424` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2183 | `"dim_0": 40.8235329,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2184 | `"dim_1": 140.7513155` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2189 | `"dim_0": 40.8232884,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2190 | `"dim_1": 140.7511592` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2195 | `"dim_0": 40.8233582,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2196 | `"dim_1": 140.7511645` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2201 | `"dim_0": 40.8234864,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2202 | `"dim_1": 140.7513315` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2207 | `"dim_0": 40.8234165,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2208 | `"dim_1": 140.7503626` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2213 | `"dim_0": 34.8364834,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2214 | `"dim_1": 137.4025072` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2219 | `"dim_0": 38.2494991,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2220 | `"dim_1": 140.8976276` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2225 | `"dim_0": 43.0639876,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2226 | `"dim_1": 141.3307515` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2231 | `"dim_0": 38.2442535,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2232 | `"dim_1": 140.8953922` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2237 | `"dim_0": 35.6868409,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2238 | `"dim_1": 139.7666525` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2243 | `"dim_0": 35.6877341,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2244 | `"dim_1": 139.7667901` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2249 | `"dim_0": 35.6873579,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2250 | `"dim_1": 139.7674357` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2255 | `"dim_0": 35.6858213,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2256 | `"dim_1": 139.7678451` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2261 | `"dim_0": 35.685894,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2262 | `"dim_1": 139.767112` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2267 | `"dim_0": 35.6856511,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2268 | `"dim_1": 139.7637584` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2273 | `"dim_0": 38.2487409,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2274 | `"dim_1": 140.9152589` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2279 | `"dim_0": 38.29825,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2280 | `"dim_1": 140.6813373` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2285 | `"dim_0": 38.292916,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2286 | `"dim_1": 140.6867566` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2291 | `"dim_0": 38.4436423,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2292 | `"dim_1": 141.2681052` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2297 | `"dim_0": 35.6309711,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2298 | `"dim_1": 139.7263178` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2303 | `"dim_0": 34.9888072,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2304 | `"dim_1": 139.8623046` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2309 | `"dim_0": 34.9892969,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2310 | `"dim_1": 139.8624902` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2315 | `"dim_0": 35.7769657,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2316 | `"dim_1": 140.2876411` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2321 | `"dim_0": 35.7770767,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2322 | `"dim_1": 140.2878557` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2327 | `"dim_0": 34.1673104,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2328 | `"dim_1": 131.4615667` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2333 | `"dim_0": 34.1678937,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2334 | `"dim_1": 131.4622166` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2339 | `"dim_0": 34.6791577,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2340 | `"dim_1": 132.5321542` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2345 | `"dim_0": 34.679041,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2346 | `"dim_1": 132.5323042` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2351 | `"dim_0": 34.8050152,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2352 | `"dim_1": 132.8552967` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2357 | `"dim_0": 34.8050147,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2358 | `"dim_1": 132.8553118` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2363 | `"dim_0": 34.8053998,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2364 | `"dim_1": 132.8543216` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2369 | `"dim_0": 34.8053665,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2370 | `"dim_1": 132.8543549` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2375 | `"dim_0": 34.8050832,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2376 | `"dim_1": 132.8550549` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2381 | `"dim_0": 35.7338406,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2382 | `"dim_1": 140.8323455` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2387 | `"dim_0": 35.7337056,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2388 | `"dim_1": 140.832198` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2393 | `"dim_0": 35.8643775,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2394 | `"dim_1": 139.6700817` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2399 | `"dim_0": 33.9868212,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2400 | `"dim_1": 131.4397614` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2405 | `"dim_0": 33.9950036,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2406 | `"dim_1": 131.432467` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2411 | `"dim_0": 35.5104942,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2412 | `"dim_1": 137.8359448` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2417 | `"dim_0": 34.1663896,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2418 | `"dim_1": 131.4358035` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2423 | `"dim_0": 35.0761774,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2424 | `"dim_1": 138.9422966` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2429 | `"dim_0": 33.5234784,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2430 | `"dim_1": 130.3839788` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2435 | `"dim_0": 35.7104001,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2436 | `"dim_1": 139.8739761` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2441 | `"dim_0": 35.7100995,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2442 | `"dim_1": 139.8740512` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2447 | `"dim_0": 36.6461075,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2448 | `"dim_1": 138.1783701` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2453 | `"dim_0": 38.2768881,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2454 | `"dim_1": 140.9239191` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2459 | `"dim_0": 34.5581037,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2460 | `"dim_1": 135.4712263` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2465 | `"dim_0": 35.5584106,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2466 | `"dim_1": 140.4072261` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2471 | `"dim_0": 34.5580939,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2472 | `"dim_1": 135.5085701` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2477 | `"dim_0": 34.9893072,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2478 | `"dim_1": 139.8624936` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2483 | `"dim_0": 34.9888011,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2484 | `"dim_1": 139.8623041` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2489 | `"dim_0": 35.39857,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2490 | `"dim_1": 139.5325889` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2495 | `"dim_0": 35.461693,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2496 | `"dim_1": 139.512063` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2501 | `"dim_0": 35.4757596,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2502 | `"dim_1": 139.5723384` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2507 | `"dim_0": 35.530251,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2508 | `"dim_1": 139.500037` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2513 | `"dim_0": 34.6103409,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2514 | `"dim_1": 133.863027` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2519 | `"dim_0": 34.6615307,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2520 | `"dim_1": 133.926725` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2525 | `"dim_0": 34.6040588,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2526 | `"dim_1": 133.8274584` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2531 | `"dim_0": 38.1726046,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2532 | `"dim_1": 140.8907109` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2537 | `"dim_0": 33.5163729,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2538 | `"dim_1": 130.3760366` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2543 | `"dim_0": 40.8224419,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2544 | `"dim_1": 140.7504602` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2549 | `"dim_0": 40.8225566,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2550 | `"dim_1": 140.7502502` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2555 | `"dim_0": 40.8224499,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2556 | `"dim_1": 140.7503595` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2561 | `"dim_0": 40.8225681,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2562 | `"dim_1": 140.7503588` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2567 | `"dim_0": 38.2616682,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2568 | `"dim_1": 140.9004403` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2573 | `"dim_0": 34.6988676,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2574 | `"dim_1": 135.530052` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2579 | `"dim_0": 26.692359,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2580 | `"dim_1": 127.9281397` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2585 | `"dim_0": 26.7103791,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2586 | `"dim_1": 127.8025288` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2591 | `"dim_0": 34.6933776,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2592 | `"dim_1": 134.2047203` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2597 | `"dim_0": 34.6467613,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2598 | `"dim_1": 135.5109779` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2603 | `"dim_0": 34.7387087,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2604 | `"dim_1": 135.5415884` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2609 | `"dim_0": 34.7377114,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2610 | `"dim_1": 135.5418679` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2615 | `"dim_0": 34.7390559,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2616 | `"dim_1": 135.5401227` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2621 | `"dim_0": 43.7701559,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2622 | `"dim_1": 142.3630012` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2627 | `"dim_0": 34.7432518,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2628 | `"dim_1": 135.5360203` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2633 | `"dim_0": 34.7499347,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2634 | `"dim_1": 135.5354197` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2639 | `"dim_0": 34.7553046,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2640 | `"dim_1": 135.5511005` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2645 | `"dim_0": 34.7516624,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2646 | `"dim_1": 135.5487072` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2651 | `"dim_0": 34.7442983,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2652 | `"dim_1": 135.5265008` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2657 | `"dim_0": 34.7478291,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2658 | `"dim_1": 135.5278998` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2663 | `"dim_0": 34.7491146,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2664 | `"dim_1": 135.5295123` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2669 | `"dim_0": 34.756636,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2670 | `"dim_1": 135.545477` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2675 | `"dim_0": 34.7574622,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2676 | `"dim_1": 135.534742` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2681 | `"dim_0": 34.7284964,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2682 | `"dim_1": 135.5422947` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2687 | `"dim_0": 34.7280686,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2688 | `"dim_1": 135.5382856` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2693 | `"dim_0": 34.7262528,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2694 | `"dim_1": 135.5350415` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2699 | `"dim_0": 34.722409,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2700 | `"dim_1": 135.538451` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2705 | `"dim_0": 34.7236766,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2706 | `"dim_1": 135.5384653` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2711 | `"dim_0": 34.7259575,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2712 | `"dim_1": 135.5447811` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2717 | `"dim_0": 36.6490069,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2718 | `"dim_1": 138.1855847` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2723 | `"dim_0": 34.7175628,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2724 | `"dim_1": 135.5381126` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2729 | `"dim_0": 34.7167117,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2730 | `"dim_1": 135.5380577` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2735 | `"dim_0": 34.7183847,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2736 | `"dim_1": 135.538203` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2741 | `"dim_0": 34.715705,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2742 | `"dim_1": 135.5402157` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2747 | `"dim_0": 34.7195279,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2748 | `"dim_1": 135.5406151` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2753 | `"dim_0": 34.719338,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2754 | `"dim_1": 135.5441877` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2759 | `"dim_0": 34.7181421,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2760 | `"dim_1": 135.5487036` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2765 | `"dim_0": 34.1689587,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2766 | `"dim_1": 131.0314372` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2771 | `"dim_0": 34.2305028,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2772 | `"dim_1": 131.366975` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2777 | `"dim_0": 34.7166811,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2778 | `"dim_1": 135.5560988` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2783 | `"dim_0": 35.4027637,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2784 | `"dim_1": 134.7706338` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2789 | `"dim_0": 35.4026347,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2790 | `"dim_1": 134.7703213` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2795 | `"dim_0": 34.7202969,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2796 | `"dim_1": 135.5542305` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2801 | `"dim_0": 34.724684,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2802 | `"dim_1": 135.5543193` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2807 | `"dim_0": 34.7257563,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2808 | `"dim_1": 135.5518045` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2813 | `"dim_0": 34.7300845,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2814 | `"dim_1": 135.5509174` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2819 | `"dim_0": 34.7158112,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2820 | `"dim_1": 135.5852483` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2825 | `"dim_0": 34.7159506,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2826 | `"dim_1": 135.5838402` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2831 | `"dim_0": 38.1995876,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2832 | `"dim_1": 140.8684704` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2837 | `"dim_0": 34.7274062,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2838 | `"dim_1": 135.4227322` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2843 | `"dim_0": 35.4670892,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2844 | `"dim_1": 139.3139116` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2849 | `"dim_0": 34.7075533,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2850 | `"dim_1": 135.5889856` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2855 | `"dim_0": 36.0564009,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2856 | `"dim_1": 136.4922345` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2861 | `"dim_0": 34.5755275,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2862 | `"dim_1": 135.4795518` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2867 | `"dim_0": 34.7025607,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2868 | `"dim_1": 135.5602395` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2873 | `"dim_0": 34.5733578,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2874 | `"dim_1": 135.4755037` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2879 | `"dim_0": 38.2825509,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2880 | `"dim_1": 140.8396569` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2885 | `"dim_0": 34.6986472,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2886 | `"dim_1": 135.5702616` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2891 | `"dim_0": 34.69939,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2892 | `"dim_1": 135.5638191` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2897 | `"dim_0": 35.6562033,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2898 | `"dim_1": 140.3168343` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2903 | `"dim_0": 35.6561706,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2904 | `"dim_1": 140.316318` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2909 | `"dim_0": 35.6563777,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2910 | `"dim_1": 140.3169537` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2915 | `"dim_0": 34.8781474,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2916 | `"dim_1": 135.6960909` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2921 | `"dim_0": 34.692519,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2922 | `"dim_1": 135.5791081` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2927 | `"dim_0": 34.6942477,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2928 | `"dim_1": 135.5694786` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2933 | `"dim_0": 32.8428378,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2934 | `"dim_1": 130.1804595` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2939 | `"dim_0": 33.9924329,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2940 | `"dim_1": 130.9668734` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2945 | `"dim_0": 35.7203459,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2946 | `"dim_1": 140.6486517` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2951 | `"dim_0": 34.6924023,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2952 | `"dim_1": 135.5610288` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2957 | `"dim_0": 34.6400347,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2958 | `"dim_1": 135.5340784` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2963 | `"dim_0": 34.639326,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2964 | `"dim_1": 135.5339852` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2969 | `"dim_0": 34.6316257,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2970 | `"dim_1": 135.5435252` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2975 | `"dim_0": 34.6361616,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2976 | `"dim_1": 135.5444247` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2981 | `"dim_0": 35.6410463,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2982 | `"dim_1": 139.444454` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2987 | `"dim_0": 34.0105306,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2988 | `"dim_1": 132.1964415` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2993 | `"dim_0": 34.0083505,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2994 | `"dim_1": 131.5804415` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 2999 | `"dim_0": 34.0080236,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3000 | `"dim_1": 131.5803886` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3005 | `"dim_0": 35.1694,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3006 | `"dim_1": 136.912338` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3011 | `"dim_0": 35.5191363,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3012 | `"dim_1": 140.3234553` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3017 | `"dim_0": 35.6730767,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3018 | `"dim_1": 139.6842336` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3023 | `"dim_0": 35.0216179,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3024 | `"dim_1": 135.7787301` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3029 | `"dim_0": 34.5303244,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3030 | `"dim_1": 135.498836` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3035 | `"dim_0": 38.2732326,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3036 | `"dim_1": 140.9681033` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3041 | `"dim_0": 38.1685501,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3042 | `"dim_1": 140.8676066` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3047 | `"dim_0": 38.7090103,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3048 | `"dim_1": 140.8354198` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3053 | `"dim_0": 34.62729,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3054 | `"dim_1": 135.4766464` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3059 | `"dim_0": 34.8832071,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3060 | `"dim_1": 135.7356656` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3065 | `"dim_0": 34.3362927,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3066 | `"dim_1": 134.0514719` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3071 | `"dim_0": 33.0931419,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3072 | `"dim_1": 139.8024415` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3077 | `"dim_0": 31.5573982,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3078 | `"dim_1": 130.4941045` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3083 | `"dim_0": 38.233522,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3084 | `"dim_1": 140.9065244` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3089 | `"dim_0": 38.2494896,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3090 | `"dim_1": 140.9248316` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3095 | `"dim_0": 40.8234535,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3096 | `"dim_1": 140.7412015` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3101 | `"dim_0": 34.4280439,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3102 | `"dim_1": 135.8218724` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3107 | `"dim_0": 35.0950436,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3108 | `"dim_1": 137.0107666` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3113 | `"dim_0": 35.949423,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3114 | `"dim_1": 139.696362` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3119 | `"dim_0": 35.3336873,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3120 | `"dim_1": 137.1289014` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3125 | `"dim_0": 36.8479765,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3126 | `"dim_1": 138.3638093` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3131 | `"dim_0": 39.7207617,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3132 | `"dim_1": 140.1404318` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3137 | `"dim_0": 38.5743742,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3138 | `"dim_1": 140.965339` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3143 | `"dim_0": 35.7498186,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3144 | `"dim_1": 139.7361491` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3149 | `"dim_0": 34.9503421,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3150 | `"dim_1": 135.7470279` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3155 | `"dim_0": 34.4356433,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3156 | `"dim_1": 135.2443968` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3161 | `"dim_0": 41.7831712,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3162 | `"dim_1": 140.7973542` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3167 | `"dim_0": 34.5377149,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3168 | `"dim_1": 135.534388` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3173 | `"dim_0": 34.5478437,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3174 | `"dim_1": 135.5084699` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3179 | `"dim_0": 34.7417586,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3180 | `"dim_1": 135.7648797` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3185 | `"dim_0": 36.3183398,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3186 | `"dim_1": 139.1981434` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3191 | `"dim_0": 38.3040222,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3192 | `"dim_1": 140.8702589` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3197 | `"dim_0": 34.39634,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3198 | `"dim_1": 132.457382` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3203 | `"dim_0": 34.405043,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3204 | `"dim_1": 132.464869` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3209 | `"dim_0": 34.658772,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3210 | `"dim_1": 135.518323` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3215 | `"dim_0": 34.617818,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3216 | `"dim_1": 135.546783` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3221 | `"dim_0": 34.717031,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3222 | `"dim_1": 135.48248` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3227 | `"dim_0": 34.70305,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3228 | `"dim_1": 135.545723` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3233 | `"dim_0": 34.710384,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3234 | `"dim_1": 135.49789` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3239 | `"dim_0": 34.685452,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3240 | `"dim_1": 135.466332` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3245 | `"dim_0": 34.701302,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3246 | `"dim_1": 135.516047` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3251 | `"dim_0": 34.66347,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3252 | `"dim_1": 135.454534` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3257 | `"dim_0": 34.709081,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3258 | `"dim_1": 135.457914` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3263 | `"dim_0": 34.676765,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3264 | `"dim_1": 135.544177` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3269 | `"dim_0": 34.691038,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3270 | `"dim_1": 135.49418` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3275 | `"dim_0": 34.672183,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3276 | `"dim_1": 135.514117` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3281 | `"dim_0": 36.593627,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3282 | `"dim_1": 136.627882` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3287 | `"dim_0": 37.9126,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3288 | `"dim_1": 139.0531` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3293 | `"dim_0": 35.175612,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3294 | `"dim_1": 136.921726` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3299 | `"dim_0": 35.174412,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3300 | `"dim_1": 136.924261` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3305 | `"dim_0": 35.164907,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3306 | `"dim_1": 136.935845` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3311 | `"dim_0": 35.184746,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3312 | `"dim_1": 136.942009` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3317 | `"dim_0": 35.172234,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3318 | `"dim_1": 136.87632` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3323 | `"dim_0": 35.166873,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3324 | `"dim_1": 136.863019` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3329 | `"dim_0": 35.133835,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3330 | `"dim_1": 136.934014` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3335 | `"dim_0": 35.159334,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3336 | `"dim_1": 136.907351` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3341 | `"dim_0": 35.166407,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3342 | `"dim_1": 136.899142` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3347 | `"dim_0": 35.175875,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3348 | `"dim_1": 136.883808` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3353 | `"dim_0": 35.18021,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3354 | `"dim_1": 136.886248` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3359 | `"dim_0": 35.478933,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3360 | `"dim_1": 139.638926` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3365 | `"dim_0": 35.529519,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3366 | `"dim_1": 139.704845` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3371 | `"dim_0": 35.627373,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3372 | `"dim_1": 139.692044` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3377 | `"dim_0": 35.652549,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3378 | `"dim_1": 139.747789` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3383 | `"dim_0": 35.630151,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3384 | `"dim_1": 139.725215` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3389 | `"dim_0": 35.632191,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3390 | `"dim_1": 139.725002` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3395 | `"dim_0": 35.528801,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3396 | `"dim_1": 139.694661` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3401 | `"dim_0": 35.513644,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3402 | `"dim_1": 139.712594` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3407 | `"dim_0": 35.588106,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3408 | `"dim_1": 139.718852` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3413 | `"dim_0": 35.666075,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3414 | `"dim_1": 139.748008` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3419 | `"dim_0": 35.647091,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3420 | `"dim_1": 139.817333` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3425 | `"dim_0": 35.598601,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3426 | `"dim_1": 139.684747` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3431 | `"dim_0": 35.435227,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3432 | `"dim_1": 139.663295` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3437 | `"dim_0": 35.444357,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3438 | `"dim_1": 139.643649` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3443 | `"dim_0": 35.648021,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3444 | `"dim_1": 139.606529` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3449 | `"dim_0": 35.63534,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3450 | `"dim_1": 139.601602` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3455 | `"dim_0": 35.618357,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3456 | `"dim_1": 139.626191` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3461 | `"dim_0": 35.46555,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3462 | `"dim_1": 139.481385` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3467 | `"dim_0": 35.374791,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3468 | `"dim_1": 139.508775` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3473 | `"dim_0": 35.391529,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3474 | `"dim_1": 139.521473` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3479 | `"dim_0": 35.484182,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3480 | `"dim_1": 139.626632` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3485 | `"dim_0": 35.670674,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3486 | `"dim_1": 139.775993` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3491 | `"dim_0": 35.6915,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3492 | `"dim_1": 139.75685` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3497 | `"dim_0": 35.710285,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3498 | `"dim_1": 139.66173` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3503 | `"dim_0": 35.699312,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3504 | `"dim_1": 139.696468` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3509 | `"dim_0": 35.683102,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3510 | `"dim_1": 139.688852` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3515 | `"dim_0": 35.697898,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3516 | `"dim_1": 139.795345` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3521 | `"dim_0": 35.714044,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3522 | `"dim_1": 139.61688` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3527 | `"dim_0": 35.714077,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3528 | `"dim_1": 139.61694` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3533 | `"dim_0": 35.714151,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3534 | `"dim_1": 139.61693` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3539 | `"dim_0": 35.691188,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3540 | `"dim_1": 139.712079` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3545 | `"dim_0": 35.726188,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3546 | `"dim_1": 139.741375` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3551 | `"dim_0": 35.684451,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3552 | `"dim_1": 139.702884` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3557 | `"dim_0": 35.750761,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3558 | `"dim_1": 139.594457` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3563 | `"dim_0": 35.669285,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3564 | `"dim_1": 139.597931` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3569 | `"dim_0": 35.5980045,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3570 | `"dim_1": 139.3456893` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3575 | `"dim_0": 36.5334845,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3576 | `"dim_1": 138.0979204` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3581 | `"dim_0": 35.6664956,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3582 | `"dim_1": 139.8161512` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3587 | `"dim_0": 36.6531478,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3588 | `"dim_1": 138.1853954` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3593 | `"dim_0": 33.3131944,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3594 | `"dim_1": 130.5555555` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3599 | `"dim_0": 35.5760567,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3600 | `"dim_1": 139.4204416` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3605 | `"dim_0": 34.5426787,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3606 | `"dim_1": 135.5265591` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3611 | `"dim_0": 34.5493122,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3612 | `"dim_1": 132.0346855` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3617 | `"dim_0": 35.3917065,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3618 | `"dim_1": 139.5212936` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3623 | `"dim_0": 36.3903134,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3624 | `"dim_1": 139.0679192` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3629 | `"dim_0": 34.9148122,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3630 | `"dim_1": 135.764223` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3635 | `"dim_0": 35.3329274,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3636 | `"dim_1": 136.8701424` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3641 | `"dim_0": 33.2264514,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3642 | `"dim_1": 132.5637461` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3647 | `"dim_0": 34.2281328,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3648 | `"dim_1": 133.7809295` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3653 | `"dim_0": 34.2280279,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3654 | `"dim_1": 133.7810087` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3659 | `"dim_0": 34.2280806,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3660 | `"dim_1": 133.7808387` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3665 | `"dim_0": 35.5865706,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3666 | `"dim_1": 139.7318657` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3671 | `"dim_0": 35.5873161,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3672 | `"dim_1": 139.7318748` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3677 | `"dim_0": 35.5873028,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3678 | `"dim_1": 139.7315732` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3683 | `"dim_0": 34.6493846,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3684 | `"dim_1": 134.164342` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3689 | `"dim_0": 34.6446117,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3690 | `"dim_1": 133.8982713` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3695 | `"dim_0": 34.6718964,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3696 | `"dim_1": 134.171604` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3701 | `"dim_0": 34.960233,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3702 | `"dim_1": 135.7463457` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3707 | `"dim_0": 34.9604522,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3708 | `"dim_1": 135.7470023` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3713 | `"dim_0": 34.9904136,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3714 | `"dim_1": 135.8401961` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3719 | `"dim_0": 35.6691104,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3720 | `"dim_1": 139.740475` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3725 | `"dim_0": 34.7813504,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3726 | `"dim_1": 134.3011586` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3731 | `"dim_0": 34.7813635,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3732 | `"dim_1": 134.3011905` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3737 | `"dim_0": 34.7799295,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3738 | `"dim_1": 134.30356` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3743 | `"dim_0": 34.7799236,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3744 | `"dim_1": 134.3035021` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3749 | `"dim_0": 34.7682895,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3750 | `"dim_1": 134.0735353` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3755 | `"dim_0": 34.694112,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3756 | `"dim_1": 135.1994615` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3761 | `"dim_0": 34.694515,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3762 | `"dim_1": 135.1993625` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3767 | `"dim_0": 35.6905308,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3768 | `"dim_1": 140.0387739` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3773 | `"dim_0": 35.6314801,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3774 | `"dim_1": 139.7419959` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3779 | `"dim_0": 35.6845537,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3780 | `"dim_1": 139.703223` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3785 | `"dim_0": 34.6445001,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3786 | `"dim_1": 133.8983802` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3791 | `"dim_0": 34.6524241,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3792 | `"dim_1": 134.0354868` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3797 | `"dim_0": 33.5883632,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3798 | `"dim_1": 130.3974953` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3803 | `"dim_0": 34.666715,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3804 | `"dim_1": 134.0927982` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3809 | `"dim_0": 34.33559,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3810 | `"dim_1": 134.0513778` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3815 | `"dim_0": 34.5021264,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3816 | `"dim_1": 133.7905368` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3821 | `"dim_0": 31.3981188,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3822 | `"dim_1": 131.3051436` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3827 | `"dim_0": 34.4329415,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3828 | `"dim_1": 135.243249` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3833 | `"dim_0": 34.9316292,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3834 | `"dim_1": 133.5162218` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3839 | `"dim_0": 35.8575092,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3840 | `"dim_1": 139.9706429` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3845 | `"dim_0": 35.8574592,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3846 | `"dim_1": 139.9704337` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3851 | `"dim_0": 35.8296299,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3852 | `"dim_1": 139.7380052` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3857 | `"dim_0": 35.8649785,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3858 | `"dim_1": 139.6473284` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3863 | `"dim_0": 35.858109,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3864 | `"dim_1": 139.513642` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3869 | `"dim_0": 35.6511846,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3870 | `"dim_1": 139.7035092` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3875 | `"dim_0": 34.38074,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3876 | `"dim_1": 132.47201` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3881 | `"dim_0": 34.39834,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3882 | `"dim_1": 132.44387` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3887 | `"dim_0": 34.39783,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3888 | `"dim_1": 132.45677` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3893 | `"dim_0": 34.960634,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3894 | `"dim_1": 135.745871` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3899 | `"dim_0": 34.695109,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3900 | `"dim_1": 135.49217` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3905 | `"dim_0": 35.6845109,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3906 | `"dim_1": 139.7029373` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3911 | `"dim_0": 34.6807323,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3912 | `"dim_1": 135.5148743` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3917 | `"dim_0": 34.6806843,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3918 | `"dim_1": 135.5149747` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3923 | `"dim_0": 34.6482127,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3924 | `"dim_1": 133.9181453` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3929 | `"dim_0": 35.6845616,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3930 | `"dim_1": 139.7030736` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3935 | `"dim_0": 34.3384866,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3936 | `"dim_1": 134.046918` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3941 | `"dim_0": 34.3384724,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3942 | `"dim_1": 134.0469764` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3947 | `"dim_0": 34.9168695,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3948 | `"dim_1": 135.6872521` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3953 | `"dim_0": 34.6984578,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3954 | `"dim_1": 135.5031802` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3959 | `"dim_0": 34.6985569,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3960 | `"dim_1": 135.5031982` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3965 | `"dim_0": 35.6777578,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3966 | `"dim_1": 139.7124984` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3971 | `"dim_0": 34.9758503,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3972 | `"dim_1": 135.7472096` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3977 | `"dim_0": 34.9758054,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3978 | `"dim_1": 135.7472233` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3983 | `"dim_0": 34.6955151,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3984 | `"dim_1": 135.4910672` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3989 | `"dim_0": 34.6954351,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3990 | `"dim_1": 135.491135` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3995 | `"dim_0": 34.9847255,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 3996 | `"dim_1": 135.7596386` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4001 | `"dim_0": 35.9634504,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4002 | `"dim_1": 140.6379047` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4007 | `"dim_0": 35.7946153,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4008 | `"dim_1": 139.3193164` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4013 | `"dim_0": 35.7957047,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4014 | `"dim_1": 139.3183651` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4019 | `"dim_0": 33.1600131,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4020 | `"dim_1": 130.4034064` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4025 | `"dim_0": 35.4755861,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4026 | `"dim_1": 139.5726537` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4031 | `"dim_0": 35.408151,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4032 | `"dim_1": 139.5914251` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4037 | `"dim_0": 35.4082369,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4038 | `"dim_1": 139.591803` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4043 | `"dim_0": 33.5885397,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4044 | `"dim_1": 130.3983144` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4049 | `"dim_0": 25.8678259,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4050 | `"dim_1": 131.2366597` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4055 | `"dim_0": 33.4229296,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4056 | `"dim_1": 130.6600368` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4061 | `"dim_0": 35.1036519,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4062 | `"dim_1": 138.859842` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4067 | `"dim_0": 35.6475749,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4068 | `"dim_1": 139.7221663` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4073 | `"dim_0": 38.5853416,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4074 | `"dim_1": 140.9674834` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4079 | `"dim_0": 38.3368705,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4080 | `"dim_1": 140.6109013` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4085 | `"dim_0": 36.1012852,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4086 | `"dim_1": 139.4587355` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4091 | `"dim_0": 26.403175,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4092 | `"dim_1": 127.7376447` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4097 | `"dim_0": 35.7588879,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4098 | `"dim_1": 139.4669257` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4103 | `"dim_0": 34.167827,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4104 | `"dim_1": 131.4622642` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4109 | `"dim_0": 34.1673674,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4110 | `"dim_1": 131.461511` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4115 | `"dim_0": 35.6463183,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4116 | `"dim_1": 139.7106279` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4121 | `"dim_0": 35.7867332,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4122 | `"dim_1": 139.4781241` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4127 | `"dim_0": 35.6491322,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4128 | `"dim_1": 139.7109839` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4133 | `"dim_0": 34.6265999,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4134 | `"dim_1": 133.807649` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4139 | `"dim_0": 34.6263582,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4140 | `"dim_1": 133.8082078` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4145 | `"dim_0": 38.2681207,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4146 | `"dim_1": 140.7907272` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4151 | `"dim_0": 34.66324,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4152 | `"dim_1": 133.92644` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4157 | `"dim_0": 35.7328151,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4158 | `"dim_1": 139.7490576` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4163 | `"dim_0": 35.7402144,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4164 | `"dim_1": 139.7468151` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4169 | `"dim_0": 35.7283993,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4170 | `"dim_1": 139.729445` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4175 | `"dim_0": 35.7320395,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4176 | `"dim_1": 139.7284922` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4181 | `"dim_0": 34.9879632,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4182 | `"dim_1": 133.4608384` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4187 | `"dim_0": 35.1566104,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4188 | `"dim_1": 133.6149005` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4193 | `"dim_0": 34.3967472,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4194 | `"dim_1": 132.4568769` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4199 | `"dim_0": 35.7276013,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4200 | `"dim_1": 139.7270895` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4205 | `"dim_0": 35.7277914,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4206 | `"dim_1": 139.7270991` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4211 | `"dim_0": 35.7300971,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4212 | `"dim_1": 139.7129205` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4217 | `"dim_0": 35.484182,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4218 | `"dim_1": 139.626632` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4223 | `"dim_0": 35.475736,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4224 | `"dim_1": 139.572285` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4229 | `"dim_0": 35.478933,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4230 | `"dim_1": 139.638926` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4235 | `"dim_0": 35.435227,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4236 | `"dim_1": 139.663295` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4241 | `"dim_0": 34.648302,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4242 | `"dim_1": 135.781885` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4247 | `"dim_0": 34.336224,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4248 | `"dim_1": 134.051376` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4253 | `"dim_0": 34.336302,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4254 | `"dim_1": 134.051724` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4259 | `"dim_0": 34.336319,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4260 | `"dim_1": 134.051566` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4265 | `"dim_0": 34.687213,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4266 | `"dim_1": 135.189872` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4271 | `"dim_0": 34.38807,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4272 | `"dim_1": 132.49186` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4277 | `"dim_0": 35.042311,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4278 | `"dim_1": 135.779432` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4283 | `"dim_0": 34.990339,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4284 | `"dim_1": 135.762865` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4289 | `"dim_0": 35.004365,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4290 | `"dim_1": 135.73605` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4295 | `"dim_0": 35.008707,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4296 | `"dim_1": 135.760415` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4301 | `"dim_0": 35.611878,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4302 | `"dim_1": 139.747725` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4307 | `"dim_0": 32.938996,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4308 | `"dim_1": 129.6399428` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4313 | `"dim_0": 35.6731077,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4314 | `"dim_1": 139.7408665` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4319 | `"dim_0": 35.673166,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4320 | `"dim_1": 139.7405892` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4325 | `"dim_0": 38.7904478,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4326 | `"dim_1": 140.0208576` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4331 | `"dim_0": 34.7946172,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4332 | `"dim_1": 135.5551767` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4337 | `"dim_0": 34.741135,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4338 | `"dim_1": 135.7644462` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4343 | `"dim_0": 34.6879521,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4344 | `"dim_1": 133.9499438` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4349 | `"dim_0": 32.979742,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4350 | `"dim_1": 130.8087376` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4355 | `"dim_0": 35.7289827,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4356 | `"dim_1": 139.4779867` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4361 | `"dim_0": 34.5434058,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4362 | `"dim_1": 133.6698381` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4367 | `"dim_0": 33.7481052,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4368 | `"dim_1": 129.6899016` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4373 | `"dim_0": 34.7200772,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4374 | `"dim_1": 134.1921101` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4379 | `"dim_0": 35.693183,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4380 | `"dim_1": 139.8267889` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4385 | `"dim_0": 34.6364685,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4386 | `"dim_1": 135.5882383` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4391 | `"dim_0": 34.6336861,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4392 | `"dim_1": 135.6099056` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4397 | `"dim_0": 34.4198563,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4398 | `"dim_1": 135.3308274` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4403 | `"dim_0": 34.6361514,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4404 | `"dim_1": 135.6395299` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4409 | `"dim_0": 35.374791,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4410 | `"dim_1": 139.508775` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4415 | `"dim_0": 35.398605,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4416 | `"dim_1": 139.532613` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4421 | `"dim_0": 35.46555,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4422 | `"dim_1": 139.481385` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4427 | `"dim_0": 35.530251,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4428 | `"dim_1": 139.500037` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4433 | `"dim_0": 35.461693,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4434 | `"dim_1": 139.512063` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4439 | `"dim_0": 34.681915,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4440 | `"dim_1": 135.825142` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4445 | `"dim_0": 34.661529,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4446 | `"dim_1": 133.926462` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4451 | `"dim_0": 35.650306,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4452 | `"dim_1": 139.589518` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4457 | `"dim_0": 35.650831,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4458 | `"dim_1": 139.587835` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4463 | `"dim_0": 24.2742639,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4464 | `"dim_1": 123.8789127` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4469 | `"dim_0": 34.587764,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4470 | `"dim_1": 135.4822215` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4475 | `"dim_0": 35.7501479,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4476 | `"dim_1": 139.4212207` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4481 | `"dim_0": 35.7029723,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4482 | `"dim_1": 139.4215632` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4487 | `"dim_0": 35.7143739,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4488 | `"dim_1": 139.5184868` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4493 | `"dim_0": 35.6423993,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4494 | `"dim_1": 139.5378706` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4499 | `"dim_0": 34.6596471,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4500 | `"dim_1": 133.9399539` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4505 | `"dim_0": 38.333938,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4506 | `"dim_1": 140.6130516` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4511 | `"dim_0": 34.6952416,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4512 | `"dim_1": 133.87163` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4517 | `"dim_0": 35.5821308,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4518 | `"dim_1": 139.6505493` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4523 | `"dim_0": 34.6632984,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4524 | `"dim_1": 133.9234126` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4529 | `"dim_0": 34.6661204,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4530 | `"dim_1": 134.091651` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4535 | `"dim_0": 35.0820728,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4536 | `"dim_1": 137.0808394` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4541 | `"dim_0": 35.0813339,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4542 | `"dim_1": 137.0796717` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4547 | `"dim_0": 35.0806889,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4548 | `"dim_1": 137.06542` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4553 | `"dim_0": 43.0306065,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4554 | `"dim_1": 141.3592772` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4559 | `"dim_0": 43.0319934,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4560 | `"dim_1": 141.3597677` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4565 | `"dim_0": 36.575038,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4566 | `"dim_1": 136.6658401` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4571 | `"dim_0": 35.593852,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4572 | `"dim_1": 138.5215527` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4577 | `"dim_0": 35.571092,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/ntt_exchanges_japan_763.json` | 4578 | `"dim_1": 139.6873495` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `tests/test_linter_reliability.py` | 426 | `We track dim_0 and dim_1 coordinates.` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/integration_test/globe_camera_drag_test.dart` | 23 | `testWidgets('Globe camera drag: dim_1 increases after leftward pan gesture', (WidgetTester teste...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/integration_test/globe_camera_drag_test.dart` | 96 | `final double initialLongitude = controller.current.dim_1;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/integration_test/globe_camera_drag_test.dart` | 99 | `expect(initialLongitude, greaterThan(0), reason: 'Initial dim_1 should be positive');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/integration_test/globe_camera_drag_test.dart` | 108 | `final double newLongitude = controller.current.dim_1;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/integration_test/globe_camera_drag_test.dart` | 111 | `expect(newLongitude, greaterThan(initialLongitude),` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/integration_test/globe_camera_drag_test.dart` | 112 | `reason: 'Dim_1 should increase after leftward drag. '` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/integration_test/globe_camera_drag_test.dart` | 113 | `'Initial: initialLongitude, New: newLongitude');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/consolidated_decision_making_report.md` | 16 | `3. **Exact Geometric Occlusion & Physical Jamming Models**: Upgrade line-of-sight checks from ray-...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/consolidated_decision_making_report.md` | 157 | `### 4.1 Exact Ray-Geometry Line-of-Sight (LOS) Occlusion` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/consolidated_decision_making_report.md` | 158 | `Instead of checking spherical approximations (which introduce up to 21 km of geometry error at the p...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/consolidated_decision_making_report.md` | 161 | `An geometry is defined by \frac{x^2}{a^2} + \frac{y^2}{b^2} + \frac{z^2}{c^2} = 1. Let \mathbf{M...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/decisions/consolidated_decision_making_report.md` | 301 | `* **Math Proofs**: Verify that the ray-geometry quadratic formula in WebGPU produces collision coor...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/feat-firestore-persistence-adapter-design.md` | 60 | `"referenceFrame": {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/feat-firestore-persistence-adapter-design.md` | 68 | `"dim_0": 35.6762,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/feat-firestore-persistence-adapter-design.md` | 69 | `"dim_1": 139.6503,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/feat-firestore-persistence-adapter-design.md` | 109 | `dim_0: data.dim_0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/feat-firestore-persistence-adapter-design.md` | 110 | `dim_1: data.dim_1,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/feat-firestore-persistence-adapter-design.md` | 127 | `dim_0: loc.dim_0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/feat-firestore-persistence-adapter-design.md` | 128 | `dim_1: loc.dim_1,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/feat-firestore-persistence-adapter-design.md` | 172 | `dim_0: 35.6762,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/feat-firestore-persistence-adapter-design.md` | 173 | `dim_1: 139.6503,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/integration_test/globe_camera_reset_test.dart` | 109 | `final initialLat = _parseHudValue('Dim_0', tester);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/integration_test/globe_camera_reset_test.dart` | 110 | `final initialLng = _parseHudValue('Dim_1', tester);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/integration_test/globe_camera_reset_test.dart` | 124 | `final afterLat = _parseHudValue('Dim_0', tester);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/integration_test/globe_camera_reset_test.dart` | 125 | `final afterLng = _parseHudValue('Dim_1', tester);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/integration_test/globe_camera_reset_test.dart` | 129 | `reason: 'Dim_0 must be identical after tree node tap. '` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/integration_test/globe_camera_reset_test.dart` | 132 | `reason: 'Dim_1 must be identical after tree node tap. '` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 6 | `"dim_0": 34.9767161,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 7 | `"dim_1": 139.9546792` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 13 | `"dim_0": 35.0387486,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 14 | `"dim_1": 139.8371399` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 20 | `"dim_0": 35.0377356,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 21 | `"dim_1": 140.0172905` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 27 | `"dim_0": 35.062414,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 28 | `"dim_1": 140.0613872` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 34 | `"dim_0": 35.1140584,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 35 | `"dim_1": 140.098692` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 41 | `"dim_0": 35.0782692,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 42 | `"dim_1": 139.9664886` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 48 | `"dim_0": 34.3578919,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 49 | `"dim_1": 136.8949592` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 55 | `"dim_0": 34.3411841,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 56 | `"dim_1": 136.8196451` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 62 | `"dim_0": 34.6891047,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 63 | `"dim_1": 137.4643919` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 69 | `"dim_0": 35.2938695,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 70 | `"dim_1": 139.2460216` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 76 | `"dim_0": 35.1441984,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 77 | `"dim_1": 139.6207589` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 83 | `"dim_0": 36.7199765,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 84 | `"dim_1": 140.7158414` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 90 | `"dim_0": 36.8018507,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 91 | `"dim_1": 140.7513188` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 97 | `"dim_0": 36.3836175,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 98 | `"dim_1": 140.6123681` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 104 | `"dim_0": 43.171677,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 105 | `"dim_1": 141.3159605` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 111 | `"dim_0": 42.6341039,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 112 | `"dim_1": 141.6054899` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 118 | `"dim_0": 37.170264,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 119 | `"dim_1": 138.2422616` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 125 | `"dim_0": 33.5571816,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 126 | `"dim_1": 130.196231` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 132 | `"dim_0": 32.097681,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 133 | `"dim_1": 131.294542` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 139 | `"dim_0": 33.6251241,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 140 | `"dim_1": 130.6180016` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 146 | `"dim_0": 33.8829996,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 147 | `"dim_1": 130.8749015` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 153 | `"dim_0": 26.5707754,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/assets/cable_landing_stations_japan.json` | 154 | `"dim_1": 128.0255901` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/requirements/dynamic-geolocation-motion-blueprint.md` | 18 | `The following class diagram defines the logical schema for geolocation, reference frames, and motion...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/requirements/dynamic-geolocation-motion-blueprint.md` | 27 | `class Geometry {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/requirements/dynamic-geolocation-motion-blueprint.md` | 28 | `+Real dim_0 "[1]"` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/requirements/dynamic-geolocation-motion-blueprint.md` | 29 | `+Real dim_1 "[1]"` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/requirements/dynamic-geolocation-motion-blueprint.md` | 40 | `class ReferenceFrame {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/requirements/dynamic-geolocation-motion-blueprint.md` | 57 | `GeoLocation *-- ReferenceFrame : "has referenceFrame"` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/requirements/dynamic-geolocation-motion-blueprint.md` | 61 | `Location <\|-- Geometry : "inherits geometry coordinates"` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/requirements/dynamic-geolocation-motion-blueprint.md` | 71 | `Rail transportation utilizes 1D Linear Referencing Systems (LRS) to track assets along physically co...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/requirements/dynamic-geolocation-motion-blueprint.md` | 84 | `- Dim_0, Dim_1, Dim_2` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/requirements/dynamic-geolocation-motion-blueprint.md` | 94 | `- Geometry position (Dim_0, Dim_1)` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/requirements/dynamic-geolocation-motion-blueprint.md` | 104 | `- Geometry position (Dim_0, Dim_1, GNSS Dim_2)` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/requirements/dynamic-geolocation-motion-blueprint.md` | 126 | `* **When** a 3D GPS telemetry update is received with geometry coordinates (Dim_0: 35.6895, Longi...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/requirements/dynamic-geolocation-motion-blueprint.md` | 171 | `- `GeoLocation.referenceFrame` mapped to `properties_view.reference_system`` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/integration_test/visual_rendering_defect_test.dart` | 124 | `dim_0: 35.6074,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/integration_test/visual_rendering_defect_test.dart` | 125 | `dim_1: 140.1063,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/domain/validation.dart` | 5 | `class ReferenceFrameValidation {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/domain/validation.dart` | 10 | `const ReferenceFrameValidation({` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/domain/validation.dart` | 26 | `ReferenceFrameValidation validateReferenceFrame(` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/domain/validation.dart` | 34 | `return ReferenceFrameValidation(` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/domain/validation.dart` | 44 | `return ReferenceFrameValidation(` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/domain/validation.dart` | 51 | `return ReferenceFrameValidation(` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/integration_test/camera_gestures_navigation_test.dart` | 194 | `final double initialLat = controller.current.dim_0;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/integration_test/camera_gestures_navigation_test.dart` | 195 | `final double initialLng = controller.current.dim_1;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/integration_test/camera_gestures_navigation_test.dart` | 206 | `final double postFlyLat = controller.current.dim_0;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/integration_test/camera_gestures_navigation_test.dart` | 207 | `final double postFlyLng = controller.current.dim_1;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/integration_test/camera_gestures_navigation_test.dart` | 208 | `expect(postFlyLat, isNot(equals(initialLat)), reason: 'Dim_0 should update after fly-to');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/integration_test/camera_gestures_navigation_test.dart` | 209 | `expect(postFlyLng, isNot(equals(initialLng)), reason: 'Dim_1 should update after fly-to');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/integration_test/camera_gestures_navigation_test.dart` | 218 | `final double postDragLng = controller.current.dim_1;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/integration_test/camera_gestures_navigation_test.dart` | 219 | `expect(postDragLng, isNot(equals(postFlyLng)), reason: 'Dim_1 should change after pan drag gestu...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/ntt_exchanges_report.md` | 20 | `\| Name \| Operator/Brand \| Dim_0 \| Dim_1 \|` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/data/data_sources/sqlite_data_source.dart` | 465 | `final latPath = _findPathToKey(decoded, 'dim_0');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/data/data_sources/sqlite_data_source.dart` | 466 | `final lngPath = _findPathToKey(decoded, 'dim_1');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/consolidated_logical_ui_design_report.md` | 192 | `2. **Dynamic Trajectory Projection**: The `TopographicalView` rendering engine maps dim_0, longit...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/cable_landing_stations_report.md` | 1 | `# Submarine InterfaceNodes in Japan` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/cable_landing_stations_report.md` | 3 | `This report contains the geocoded dataset of **22 submarine interfaceNodes** across Japan, i...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/cable_landing_stations_report.md` | 11 | `## InterfaceNodes List` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `docs/cable_landing_stations_report.md` | 13 | `\| Station Name (English) \| Station Name (Japanese) \| Location \| Dim_0 \| Dim_1 \|` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/data/data_sources/firebase_data_source.dart` | 397 | `final ellip = loc['geometry'] ?? loc;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/data/data_sources/firebase_data_source.dart` | 399 | `latVal = double.tryParse(ellip['dim_0']?.toString() ?? '');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/data/data_sources/firebase_data_source.dart` | 400 | `lngVal = double.tryParse(ellip['dim_1']?.toString() ?? '');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/data/seeds/domain_seed_strategy.dart` | 10 | `/// interfaceNodes, and their interconnectivity links.` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/data/seeds/domain_seed_strategy.dart` | 79 | `'lat': (item['dim_0'] as num).toDouble(),` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/data/seeds/domain_seed_strategy.dart` | 80 | `'lon': (item['dim_1'] as num).toDouble(),` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/data/seeds/domain_seed_strategy.dart` | 82 | `_addNodeToBatch(batch, id, null, nttDetails, lat: (item['dim_0'] as num).toDouble(), lon: (item['...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/data/seeds/domain_seed_strategy.dart` | 85 | `// 4. Load and parse interfaceNodes data from assets` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/data/seeds/domain_seed_strategy.dart` | 101 | `'lat': (item['dim_0'] as num).toDouble(),` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/data/seeds/domain_seed_strategy.dart` | 102 | `'lon': (item['dim_1'] as num).toDouble(),` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/data/seeds/domain_seed_strategy.dart` | 104 | `_addNodeToBatch(batch, id, null, landingDetails, lat: (item['dim_0'] as num).toDouble(), lon: (it...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/data/seeds/domain_seed_strategy.dart` | 221 | `'geometry': {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/data/seeds/domain_seed_strategy.dart` | 222 | `'dim_0': lat,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/data/seeds/domain_seed_strategy.dart` | 223 | `'dim_1': lon,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/visual_test_spec.md` | 13 | `*   **Far Zoom**: `VirtualCamera(dim_0: 35.6074, dim_1: 140.1063, dim_2: 6378137.0 + 20960...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/visual_test_spec.md` | 14 | `*   **Close Zoom**: `VirtualCamera(dim_0: 35.6074, dim_1: 140.1063, dim_2: 6378137.0 + 500...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/spec_validator.py` | 62 | `f"Expected format e.g. 'ietf-geo-location:geo-location/reference-frame'."` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/cesium_3d/viewport_math_test.dart` | 11 | `dim_0: 0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/cesium_3d/viewport_math_test.dart` | 12 | `dim_1: 0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/cesium_3d/viewport_math_test.dart` | 19 | `expect(absoluteCamera.dim_2, Geometry.wgs84EquatorialRadius + 500.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/cesium_3d/viewport_math_test.dart` | 24 | `dim_0: 0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/cesium_3d/viewport_math_test.dart` | 25 | `dim_1: 0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/cesium_3d/viewport_math_test.dart` | 58 | `dim_0: 0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/cesium_3d/viewport_math_test.dart` | 59 | `dim_1: 0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/cesium_3d/viewport_math_test.dart` | 80 | `dim_0: 0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/cesium_3d/viewport_math_test.dart` | 81 | `dim_1: 0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/cesium_3d/viewport_math_test.dart` | 107 | `dim_0: 0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/cesium_3d/viewport_math_test.dart` | 108 | `dim_1: 0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport_classes.dart` | 27 | `if (dim_2 >= Geometry.wgs84EquatorialRadius) {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport_classes.dart` | 31 | `dim_0: dim_0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport_classes.dart` | 32 | `dim_1: dim_1,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport_classes.dart` | 33 | `dim_2: Geometry.wgs84EquatorialRadius + dim_2,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport_classes.dart` | 106 | `_r2 = Geometry.wgs84EquatorialRadius * Geometry.wgs84EquatorialRadius;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport_classes.dart` | 135 | `final double R = Geometry.wgs84EquatorialRadius;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport_classes.dart` | 249 | `final double R = Geometry.wgs84EquatorialRadius;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport_classes.dart` | 413 | `final double baseRotation = -(camera.dim_1 * math.pi / 180.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport_classes.dart` | 414 | `final double baseTilt = -(camera.dim_0 * math.pi / 180.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport_classes.dart` | 427 | `earthCenterProj = transformer.projectWgs84ToScreen(latRad: 0.0, lngRad: 0.0, heightMeters: -Ellipsoi...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport_classes.dart` | 432 | `final double radDiff = cRad * cRad - Geometry.wgs84EquatorialRadius * Geometry.wgs84EquatorialRadi...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport_classes.dart` | 433 | `projectedRadius = Geometry.wgs84EquatorialRadius * f / math.sqrt(radDiff <= 0.0 ? 1.0 : radDiff);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/validation_test.dart` | 43 | `group('validateReferenceFrame', () {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/validation_test.dart` | 46 | `final result = validateReferenceFrame(frame);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/validation_test.dart` | 53 | `final result = validateReferenceFrame(frame, alternateSystemEnabled: true);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/validation_test.dart` | 61 | `final result = validateReferenceFrame(frame, alternateSystemEnabled: false);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/validation_test.dart` | 67 | `final result = validateReferenceFrame(frame);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/validation_test.dart` | 75 | `final result = validateReferenceFrame(frame, frameName: 'test\x00name');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/validation_test.dart` | 81 | `final result = validateReferenceFrame(frame, frameName: 'test\nname');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/validation_test.dart` | 87 | `final result = validateReferenceFrame(frame, frameName: 'test\tname');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/validation_test.dart` | 93 | `final result = validateReferenceFrame(frame, frameName: 'test\x7fname');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/validation_test.dart` | 99 | `final result = validateReferenceFrame(` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/validation_test.dart` | 111 | `final result = validateReferenceFrame(frame, frameName: 'mars');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/validation_test.dart` | 117 | `final result = validateReferenceFrame(frame, frameName: 'MarsMoon');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/validation_test.dart` | 125 | `final result = validateReferenceFrame(frame, frameName: 'the-mars');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/validation_test.dart` | 131 | `final result = validateReferenceFrame(frame, frameName: 'THE-MARS');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/validation_test.dart` | 137 | `final result = validateReferenceFrame(frame, frameName: '  the-moon  ');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/validation_test.dart` | 145 | `final result = validateReferenceFrame(frame);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/validation_test.dart` | 153 | `final result = validateReferenceFrame(frame);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/validation_test.dart` | 159 | `final resultEnabled = validateReferenceFrame(frame, alternateSystemEnabled: true);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/domain/validation_test.dart` | 160 | `final resultDisabled = validateReferenceFrame(frame, alternateSystemEnabled: false);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/topographical_view.dart` | 119 | `double dim_0;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/topographical_view.dart` | 120 | `double dim_1;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/topographical_view.dart` | 136 | `dim_0 = 35.6074;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/topographical_view.dart` | 137 | `dim_1 = 140.1063;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/topographical_view.dart` | 139 | `dim_0 = latVal;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/topographical_view.dart` | 140 | `dim_1 = lngVal;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/topographical_view.dart` | 144 | `dim_0 = 35.6074;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/topographical_view.dart` | 145 | `dim_1 = 140.1063;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/topographical_view.dart` | 148 | `dim_0 = dim_0.clamp(-90.0, 90.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/topographical_view.dart` | 149 | `dim_1 = dim_1.clamp(-180.0, 180.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/topographical_view.dart` | 151 | `print("TopographicalView: final camera lat=dim_0, lng=dim_1");` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/topographical_view.dart` | 153 | `dim_0: dim_0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/topographical_view.dart` | 154 | `dim_1: dim_1,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-orchestrator/parity_auditor/tests/test_spec_validator_issue237.py` | 109 | `"  - path: \"ietf-geo-location/reference-frame\"\n"` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-orchestrator/parity_auditor/tests/test_spec_validator_issue237.py` | 120 | `"  - path: \"ietf-geo-location:geo-location/reference-frame\"\n"` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-orchestrator/parity_auditor/tests/test_spec_validator_issue237.py` | 131 | `assert any("feat-01-unqualified.md" in err and "ietf-geo-location/reference-frame" in err and "unqua...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport.dart` | 130 | `const double earthRadius = Geometry.wgs84EquatorialRadius;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport.dart` | 245 | `Geometry.wgs84EquatorialRadius,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport.dart` | 340 | `final double surfaceHeight = Geometry.wgs84EquatorialRadius + elev * state.verticalExaggeration;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport.dart` | 415 | `final double R = Geometry.wgs84EquatorialRadius;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport.dart` | 568 | `cam = VirtualCamera.raw(dim_0: -tilt * 180 / math.pi, dim_1: -rotationAngle * 180 / math.pi, ...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport.dart` | 579 | `heightMeters: heightMeters - Geometry.wgs84EquatorialRadius,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport.dart` | 632 | `Text('Dim_0: {cam.dim_0.toStringAsFixed(6)}', style: const TextStyle(color: Color(0xFFE0E0E0)...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport.dart` | 633 | `Text('Dim_1: {cam.dim_1.toStringAsFixed(6)}', style: const TextStyle(color: Color(0xFFE0E0E...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport.dart` | 634 | `Text('Dim_2: {(cam.dim_2 - Geometry.wgs84EquatorialRadius).toStringAsFixed(2)} meters', styl...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport.dart` | 842 | `double dim_0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport.dart` | 843 | `double dim_1, {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport.dart` | 851 | `final camera = rawCamera.dim_2 < Geometry.wgs84EquatorialRadius` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport.dart` | 853 | `dim_0: rawCamera.dim_0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport.dart` | 854 | `dim_1: rawCamera.dim_1,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport.dart` | 855 | `dim_2: Geometry.wgs84EquatorialRadius + rawCamera.dim_2,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport.dart` | 866 | `rotationAngle: -(camera.dim_1 * math.pi / 180.0),` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport.dart` | 867 | `tilt: -(camera.dim_0 * math.pi / 180.0),` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport.dart` | 885 | `final double terrainElev = Scene3DViewportPainter.getElevationStatic(dim_0, dim_1, _elevation...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport.dart` | 901 | `latRad: dim_0 * math.pi / 180.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport.dart` | 902 | `lngRad: dim_1 * math.pi / 180.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport.dart` | 1021 | `return VirtualCamera.raw(dim_0: 35.6074, dim_1: 140.1063, dim_2: 500.0, heading: 0.0, pitc...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport.dart` | 1023 | `return VirtualCamera.raw(dim_0: latVal, dim_1: lngVal, dim_2: 500.0, heading: 0.0, pitch: ...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport.dart` | 1072 | `final surfaceAlt = current.dim_2 - Geometry.wgs84EquatorialRadius;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport.dart` | 1078 | `dim_0: current.dim_0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport.dart` | 1079 | `dim_1: current.dim_1,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/topology/scene_3d_viewport.dart` | 1080 | `dim_2: targetAlt + Geometry.wgs84EquatorialRadius,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_focus_test.dart` | 11 | `dim_0: 35.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_focus_test.dart` | 12 | `dim_1: 135.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_focus_test.dart` | 61 | `expect(controller.current.dim_1, lessThan(135.0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_focus_test.dart` | 66 | `expect(controller.current.dim_1, equals(135.0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_focus_test.dart` | 68 | `// Hold Shift and press Arrow Left key (should rotate heading, dim_1 stays 135.0)` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_focus_test.dart` | 73 | `expect(controller.current.dim_1, equals(135.0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_focus_test.dart` | 81 | `expect(controller.current.dim_1, equals(135.0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test_output.txt` | 48 | `Dim_0 should update after fly-to` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/collapse_hud_test.dart` | 9 | `dim_0: 35.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/collapse_hud_test.dart` | 10 | `dim_1: 135.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 10 | `class Geometry {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 21 | `final double dim_0;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 22 | `final double dim_1;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 30 | `required double dim_0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 31 | `required double dim_1,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 37 | `if (dim_0.isNaN \|\| dim_0.isInfinite \|\|` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 38 | `dim_1.isNaN \|\| dim_1.isInfinite \|\|` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 45 | `if (dim_0 < -90.0 \|\| dim_0 > 90.0) {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 46 | `throw CoordinateValidationException('Dim_0 must be in the range [-90.0, 90.0].');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 48 | `if (dim_1 < -180.0 \|\| dim_1 > 180.0) {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 49 | `throw CoordinateValidationException('Dim_1 must be in the range [-180.0, 180.0].');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 55 | `dim_0: dim_0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 56 | `dim_1: dim_1,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 66 | `required this.dim_0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 67 | `required this.dim_1,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 76 | `dim_0: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 77 | `dim_1: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 85 | `/// Clamps dim_2 to at least -100.0, dim_0 to [-90, 90], and dim_1 to [-180, 180].` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 87 | `required double dim_0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 88 | `required double dim_1,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 94 | `final double lat = (dim_0.isNaN \|\| dim_0.isInfinite) ? 0.0 : dim_0;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 95 | `final double lng = (dim_1.isNaN \|\| dim_1.isInfinite) ? 0.0 : dim_1;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 105 | `dim_0: clampedLat,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 106 | `dim_1: clampedLng,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 118 | `return other.dim_0 == dim_0 &&` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 119 | `other.dim_1 == dim_1 &&` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 133 | `return (dim_0 - other.dim_0).abs() <= epsilonCoordinate &&` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 134 | `(dim_1 - other.dim_1).abs() <= epsilonCoordinate &&` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 142 | `int get hashCode => Object.hash(dim_0, dim_1, dim_2, heading, pitch, roll);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/virtual_camera.dart` | 146 | `return 'VirtualCamera(dim_0: dim_0, dim_1: dim_1, dim_2: dim_2, heading: hea...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-orchestrator/parity_auditor/tests/test_cli.py` | 161 | `class ReferenceFrame {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-orchestrator/parity_auditor/tests/test_cli.py` | 164 | `note for ReferenceFrame "alternateSystem guarded by <<feature_guard>> alternate-systems"` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-orchestrator/parity_auditor/tests/test_cli.py` | 167 | `assert "ReferenceFrame" in parsed.classes` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-orchestrator/parity_auditor/tests/test_cli.py` | 168 | `cls_info = parsed.classes["ReferenceFrame"]` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_angle_wrapping_test.dart` | 25 | `dim_0: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_angle_wrapping_test.dart` | 26 | `dim_1: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_angle_wrapping_test.dart` | 56 | `dim_0: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_angle_wrapping_test.dart` | 57 | `dim_1: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-orchestrator/parity_auditor/tests/test_cli_coverage_choice_case.py` | 38 | `{"path": "ietf-geo-location:geo-location/location/geometry"},` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-orchestrator/parity_auditor/tests/test_cli_coverage_choice_case.py` | 39 | `{"path": "ietf-geo-location:geo-location/location/cartesian"}` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-orchestrator/parity_auditor/tests/test_cli_coverage_choice_case.py` | 45 | `"  - path: ietf-geo-location:geo-location/location/geometry\n"` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-orchestrator/parity_auditor/tests/test_cli_coverage_choice_case.py` | 46 | `"  - path: ietf-geo-location:geo-location/location/cartesian\n"` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-orchestrator/parity_auditor/tests/test_cli_coverage_choice_case.py` | 60 | `"ietf-geo-location",` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-orchestrator/parity_auditor/tests/test_cli_coverage_choice_case.py` | 62 | `"ietf-geo-location:geo-location/location/geometry": {"type": "case"},` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-orchestrator/parity_auditor/tests/test_cli_coverage_choice_case.py` | 63 | `"ietf-geo-location:geo-location/location/cartesian": {"type": "case"}` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_drag_test.dart` | 10 | `dim_0: 35.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_drag_test.dart` | 11 | `dim_1: 135.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_drag_test.dart` | 30 | `expect(find.textContaining('Dim_0: 35.000000'), findsOneWidget);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_drag_test.dart` | 31 | `expect(find.textContaining('Dim_1: 135.000000'), findsOneWidget);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_drag_test.dart` | 42 | `expect(controller.current.dim_1, isNot(135.0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_drag_test.dart` | 43 | `expect(controller.current.dim_0, isNot(35.0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_drag_test.dart` | 46 | `expect(find.textContaining('Dim_0: 35.000000'), findsNothing);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_drag_test.dart` | 47 | `expect(find.textContaining('Dim_1: 135.000000'), findsNothing);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/globe_tile_renderer.dart` | 136 | `/// Converts dim_0/dim_1 (degrees) to a tile coordinate at the` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/globe_tile_renderer.dart` | 152 | `/// Dim_1 of the *western* edge of tile column [x] at zoom [z].` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/globe_tile_renderer.dart` | 156 | `/// Dim_0 of the *northern* edge of tile row [y] at zoom [z].` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/globe_tile_renderer.dart` | 173 | `final double R = Geometry.wgs84EquatorialRadius;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/globe_tile_renderer.dart` | 179 | `final center = _latLngToTile(camera.dim_0, camera.dim_1, zoom);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/globe_tile_renderer.dart` | 182 | `// Horizon angle theta = acos(R / (R + h)) where R = Geometry.wgs84EquatorialRadius` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/globe_tile_renderer.dart` | 196 | `final midCenter = _latLngToTile(camera.dim_0, camera.dim_1, midZoom);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-orchestrator/parity_auditor/tests/test_compile_yang_issue240.py` | 30 | `lat_leaf = MockNode("leaf", "dim_0", children=[type_stmt])` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-orchestrator/parity_auditor/tests/test_compile_yang_issue240.py` | 31 | `long_leaf = MockNode("leaf", "dim_1", children=[type_stmt])` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-orchestrator/parity_auditor/tests/test_compile_yang_issue240.py` | 42 | `assert "location/dim_0" in attr_keys` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-orchestrator/parity_auditor/tests/test_compile_yang_issue240.py` | 43 | `assert "location/dim_1" in attr_keys` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-orchestrator/parity_auditor/tests/test_compile_yang_issue240.py` | 149 | `leaf dim_0 {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-orchestrator/parity_auditor/tests/test_compile_yang_issue240.py` | 152 | `leaf dim_1 {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-orchestrator/parity_auditor/tests/test_compile_yang_issue240.py` | 171 | `assert lui_json["attributes"][0]["key"] == "location/dim_0"` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-orchestrator/parity_auditor/tests/test_compile_yang_issue240.py` | 172 | `assert lui_json["attributes"][1]["key"] == "location/dim_1"` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 33 | `final double minAlt = Geometry.wgs84EquatorialRadius + terrainH + minAltitude;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 37 | `CameraController(VirtualCamera camera) : _camera = camera.dim_2 < Geometry.wgs84EquatorialRadius...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 38 | `dim_0: camera.dim_0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 39 | `dim_1: camera.dim_1,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 40 | `dim_2: Geometry.wgs84EquatorialRadius + camera.dim_2,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 51 | `final absoluteCamera = camera.dim_2 < Geometry.wgs84EquatorialRadius ? VirtualCamera.clamped(` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 52 | `dim_0: camera.dim_0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 53 | `dim_1: camera.dim_1,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 54 | `dim_2: Geometry.wgs84EquatorialRadius + camera.dim_2,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 59 | `final double targetAlt = _clampAltitudeToTerrain(absoluteCamera.dim_0, absoluteCamera.dim_1, ...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 61 | `dim_0: absoluteCamera.dim_0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 62 | `dim_1: absoluteCamera.dim_1,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 83 | `final double lat1 = a.dim_0 * math.pi / 180.0;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 84 | `final double lat2 = b.dim_0 * math.pi / 180.0;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 85 | `final double lon1 = a.dim_1 * math.pi / 180.0;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 86 | `final double lon2 = b.dim_1 * math.pi / 180.0;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 136 | `dim_0: a.dim_0 + (b.dim_0 - a.dim_0) * t,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 137 | `dim_1: _interpolateCircular(a.dim_1, b.dim_1, t, _wrapLngStatic),` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 174 | `final double factor = (_camera.dim_2 - Geometry.wgs84EquatorialRadius + 500000.0) * 2.8074e-5 / ...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 180 | `final newLat = (_camera.dim_0 - dyAligned * factor).clamp(-90.0, 90.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 181 | `final newLng = _wrapLng(_camera.dim_1 - dxAligned * factor);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 184 | `dim_0: newLat,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 185 | `dim_1: newLng,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 196 | `dim_0: _camera.dim_0, dim_1: _camera.dim_1,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 207 | `dim_0: _camera.dim_0, dim_1: _camera.dim_1,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 216 | `final double terrainH = _getTerrainHeight(_camera.dim_0, _camera.dim_1);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 217 | `final double currentHeightAGL = _camera.dim_2 - (Geometry.wgs84EquatorialRadius + terrainH);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 220 | `final double newAlt = Geometry.wgs84EquatorialRadius + clampedHeightAGL + terrainH;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 222 | `dim_0: _camera.dim_0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 223 | `dim_1: _camera.dim_1,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 235 | `final double terrainH = _getTerrainHeight(_camera.dim_0, _camera.dim_1);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 236 | `final double currentHeightAGL = _camera.dim_2 - (Geometry.wgs84EquatorialRadius + terrainH);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 239 | `final double newAlt = Geometry.wgs84EquatorialRadius + clampedHeightAGL + terrainH;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 241 | `dim_0: _camera.dim_0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 242 | `dim_1: _camera.dim_1,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 253 | `dim_0: _camera.dim_0, dim_1: _wrapLng(_camera.dim_1 + degrees),` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 262 | `dim_0: _camera.dim_0, dim_1: _camera.dim_1,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/camera_controller.dart` | 272 | `dim_0: _camera.dim_0, dim_1: _camera.dim_1,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/hud_update_test.dart` | 51 | `dim_0: 35.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/hud_update_test.dart` | 52 | `dim_1: 135.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/hud_update_test.dart` | 63 | `expect(find.textContaining('Dim_0: 35.000000'), findsOneWidget);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/hud_update_test.dart` | 64 | `expect(find.textContaining('Dim_1: 135.000000'), findsOneWidget);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/hud_update_test.dart` | 72 | `final double newLat = controller.current.dim_0;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/hud_update_test.dart` | 73 | `final double newLng = controller.current.dim_1;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/hud_update_test.dart` | 78 | `expect(find.textContaining('Dim_0: 35.000000'), findsNothing);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/hud_update_test.dart` | 79 | `expect(find.textContaining('Dim_1: 135.000000'), findsNothing);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/hud_update_test.dart` | 89 | `expect(controllerAfter.current.dim_0, equals(newLat));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/hud_update_test.dart` | 90 | `expect(controllerAfter.current.dim_1, equals(newLng));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/hud_update_test.dart` | 91 | `expect(find.textContaining('Dim_0: 35.000000'), findsNothing);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/hud_update_test.dart` | 92 | `expect(find.textContaining('Dim_1: 135.000000'), findsNothing);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/cesium_engine.dart` | 63 | `native.ref.dim_0 = camera.dim_0;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/cesium_engine.dart` | 64 | `native.ref.dim_1 = camera.dim_1;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_collision_test.dart` | 30 | `test('Nadir Zoom-in Clamps at Geometry Base Over Ocean (Flat Terrain)', () {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_collision_test.dart` | 32 | `dim_0: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_collision_test.dart` | 33 | `dim_1: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_collision_test.dart` | 52 | `dim_0: 35.3606,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_collision_test.dart` | 53 | `dim_1: 138.7274,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_collision_test.dart` | 74 | `dim_0: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_collision_test.dart` | 75 | `dim_1: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_collision_test.dart` | 86 | `dim_0: 35.3606,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_collision_test.dart` | 87 | `dim_1: 138.7274,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/cesium_3d_native.dart` | 26 | `dim_0: camera.dim_0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/cesium_3d_native.dart` | 27 | `dim_1: camera.dim_1,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/logical_ui_validator.py` | 166 | `FORBIDDEN_CHOICE_NODES = {"location-choice", "cartesian", "geometry", "choice", "case"}` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/logical_ui_validator.py` | 227 | `GEODETIC_REGEX = re.compile(r"\b(?:location\|rateOfChange\|geo-location\|geometry\|dim_0\|dim_1\|...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/scroll_zoom_test.dart` | 11 | `dim_0: 35.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/scroll_zoom_test.dart` | 12 | `dim_1: 135.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/native/bridge_bindings.dart` | 12 | `external double dim_0;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/lib/features/map_viewport/cesium_3d/native/bridge_bindings.dart` | 15 | `external double dim_1;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/double_click_fly_test.dart` | 10 | `dim_0: 35.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/double_click_fly_test.dart` | 11 | `dim_1: 135.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-orchestrator/parity_auditor/tests/test_logical_ui_validator_issue222.py` | 374 | `Contains dim_0 and dim_1 coordinates.` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-orchestrator/parity_auditor/tests/test_logical_ui_validator_issue222.py` | 388 | `Contains dim_0 and dim_1 coordinates.` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-orchestrator/parity_auditor/tests/test_logical_ui_validator_issue222.py` | 622 | `- **Data Source Binding:** /ietf-hardware:hardware/component/location-choice, /ietf-hardware:hardwar...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-orchestrator/parity_auditor/tests/test_logical_ui_validator_issue222.py` | 630 | `forbidden_nodes = ["location-choice", "cartesian", "geometry", "my-choice", "my-case"]` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/ctrl_drag_test.dart` | 11 | `dim_0: 35.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/ctrl_drag_test.dart` | 12 | `dim_1: 135.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/ctrl_drag_test.dart` | 48 | `expect(controller.current.dim_0, equals(35.0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/ctrl_drag_test.dart` | 49 | `expect(controller.current.dim_1, equals(135.0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/virtual_camera_test.dart` | 7 | `const camera = VirtualCamera.raw(dim_0: 10, dim_1: 20, dim_2: 30, heading: 40, pitch: 50, ...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/virtual_camera_test.dart` | 12 | `const camera1 = VirtualCamera.raw(dim_0: 10, dim_1: 20, dim_2: 30, heading: 40, pitch: 50,...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/virtual_camera_test.dart` | 13 | `const camera2 = VirtualCamera.raw(dim_0: 10.00000001, dim_1: 20.00000001, dim_2: 30.0001, ...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/virtual_camera_test.dart` | 18 | `const camera1 = VirtualCamera.raw(dim_0: 10, dim_1: 20, dim_2: 30, heading: 40, pitch: 50,...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/virtual_camera_test.dart` | 19 | `const camera2 = VirtualCamera.raw(dim_0: 10.1, dim_1: 20.0, dim_2: 30.0, heading: 40.0, pi...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/schema-specification-engineering/SKILL.md` | 57 | `- **Container Traceability**: Every Feature MUST declare exactly one schema container in its YAML fr...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/schema-specification-engineering/SKILL.md` | 118 | `> **Container Traceability:** Every Feature MUST declare its schema container in `schema_containers`...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/schema-specification-engineering/SKILL.md` | 305 | `- Geolocation and geometry attributes (such as reference-frame, geometry-system, coordinates, veloci...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 34 | `dim_0: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 35 | `dim_1: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 46 | `final centerTile = renderer.latLngToTileForTesting(camera.dim_0, camera.dim_1, 8);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 91 | `dim_0: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 92 | `dim_1: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 103 | `// Now call renderTiles and capture the latitudes passed to projectFn` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 104 | `final latitudes = <double>[];` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 114 | `latitudes.add(lat);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 119 | `// Helper to compute unclamped dim_0 at zoom 2, y=0 and y=4` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 127 | `// Verify that the captured latitudes contain exactly 90.0 and -90.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 128 | `// and do NOT contain unclamped boundary latitudes (~85.0511 or ~-85.0511)` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 129 | `expect(latitudes, contains(90.0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 130 | `expect(latitudes, contains(-90.0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 131 | `expect(latitudes, isNot(contains(unclampedNorth)));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 132 | `expect(latitudes, isNot(contains(unclampedSouth)));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 214 | `dim_0: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 215 | `dim_1: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 261 | `dim_0: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 262 | `dim_1: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 274 | `dim_0: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 275 | `dim_1: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 291 | `dim_0: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 292 | `dim_1: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 314 | `dim_0: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 315 | `dim_1: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 395 | `dim_0: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 396 | `dim_1: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 436 | `final latitudes = [-35.0, 0.0, 35.3606];` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 437 | `final longitudes = [-135.0, 0.0, 138.7274];` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 443 | `for (final lat in latitudes) {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 444 | `for (final lng in longitudes) {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 448 | `dim_0: lat,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 449 | `dim_1: lng,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 541 | `final double baseRotation = -(camera.dim_1 * math.pi / 180.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 542 | `final double baseTilt = -(camera.dim_0 * math.pi / 180.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 566 | `dim_0: 35.3606,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 567 | `dim_1: 138.7274,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 598 | `final double rotationY = -(camera.dim_1 * math.pi / 180.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/globe_tile_renderer_test.dart` | 599 | `final double tilt = -(camera.dim_0 * math.pi / 180.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/shift_drag_test.dart` | 11 | `dim_0: 35.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/shift_drag_test.dart` | 12 | `dim_1: 135.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/shift_drag_test.dart` | 48 | `expect(controller.current.dim_0, equals(35.0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/shift_drag_test.dart` | 49 | `expect(controller.current.dim_1, equals(135.0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `skills/spec-usecase-engineering/SKILL.md` | 149 | `> **Container Traceability:** Every Use Case MUST declare its schema container in `schema_containers...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/tile_imagery_repaint_test.dart` | 94 | `dim_0: 35.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/tile_imagery_repaint_test.dart` | 95 | `dim_1: 135.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/right_click_drag_test.dart` | 11 | `dim_0: 35.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/right_click_drag_test.dart` | 12 | `dim_1: 135.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/right_click_drag_test.dart` | 48 | `expect(controller.current.dim_0, equals(35.0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/right_click_drag_test.dart` | 49 | `expect(controller.current.dim_1, equals(135.0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 17 | `dim_0: lat,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 18 | `dim_1: lng,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 30 | `expect(cam.dim_1, lessThan(135.0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 31 | `expect(cam.dim_0, lessThan(35.0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 34 | `test('pan left (negative dx) increases dim_1', () {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 36 | `final before = c.current.dim_1;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 39 | `expect(after.dim_1, greaterThan(before));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 40 | `expect(after.dim_0, equals(35.0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 46 | `test('pan up (negative dy) increases dim_0', () {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 48 | `final before = c.current.dim_0;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 51 | `expect(after.dim_0, greaterThan(before));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 52 | `expect(after.dim_1, equals(135.0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 59 | `expect(c.current.dim_1, closeTo(-1.75638, 0.0001));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 60 | `expect(c.current.dim_0, closeTo(-1.75638, 0.0001));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 63 | `test('pan clamps dim_0 to [-90, 90]', () {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 66 | `expect(c.current.dim_0, equals(90.0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 69 | `test('pan wraps dim_1 past 180', () {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 72 | `expect(c.current.dim_1, lessThan(-160.0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 81 | `expect(after.dim_0, equals(before.dim_0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 82 | `expect(after.dim_1, equals(before.dim_1));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 90 | `expect(after.dim_0, equals(35.0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 101 | `expect(after.dim_0, equals(before.dim_0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 102 | `expect(after.dim_1, equals(before.dim_1));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 111 | `expect(after.dim_0, equals(before.dim_0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 112 | `expect(after.dim_1, equals(before.dim_1));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 129 | `test('dim_1 wraps around -180/+180 boundary', () {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 132 | `expect(c.current.dim_1, lessThan(180));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 133 | `expect(c.current.dim_1, greaterThan(155));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 136 | `test('keyboardRotate changes dim_1 only', () {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 139 | `expect(c.current.dim_1, equals(145.0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 140 | `expect(c.current.dim_0, equals(35.0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 147 | `expect(c.current.dim_1, equals(135.0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 148 | `expect(c.current.dim_0, equals(35.0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 195 | `expect(after.dim_0, equals(before.dim_0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 196 | `expect(after.dim_1, equals(before.dim_1));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 236 | `dim_0: 35.6,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 237 | `dim_1: 135.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 244 | `dim_0: 40.7,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 245 | `dim_1: -74.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 271 | `expect(controller.current.dim_0, closeTo(40.7, 0.001));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 272 | `expect(controller.current.dim_1, closeTo(-74.0, 0.001));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 280 | `final a = VirtualCamera(dim_0: 35, dim_1: 135, dim_2: 500, heading: 0, pitch: -45, roll: 0...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 281 | `final b = VirtualCamera(dim_0: 35, dim_1: 135, dim_2: 500, heading: 0, pitch: -45, roll: 0...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 285 | `final a = VirtualCamera(dim_0: 35, dim_1: 135, dim_2: 500, heading: 0, pitch: -45, roll: 0...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/camera_controller_test.dart` | 286 | `final b = VirtualCamera(dim_0: 36, dim_1: 135, dim_2: 500, heading: 0, pitch: -45, roll: 0...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_widget_test.dart` | 13 | `dim_0: 0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_widget_test.dart` | 14 | `dim_1: 0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_test.dart` | 19 | `dim_0: 35.0, dim_1: 138.0, dim_2: 2000000.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_test.dart` | 67 | `dim_0: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_test.dart` | 68 | `dim_1: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_test.dart` | 107 | `math.pi, // opposite dim_1` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_test.dart` | 121 | `dim_0: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_test.dart` | 122 | `dim_1: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_test.dart` | 162 | `dim_0: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_test.dart` | 163 | `dim_1: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_test.dart` | 201 | `math.pi, // opposite dim_1` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_test.dart` | 230 | `dim_0: 35.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_test.dart` | 231 | `dim_1: 135.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_test.dart` | 258 | `0.5, // 30 degrees dim_0` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_test.dart` | 259 | `2.3, // 131 degrees dim_1` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_test.dart` | 275 | `final camera = VirtualCamera.clamped(dim_0: 35.0, dim_1: 138.0, dim_2: 2000000.0, heading:...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_test.dart` | 323 | `dim_0: 35.0, dim_1: 138.0, dim_2: 2000000.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_test.dart` | 412 | `final camera = VirtualCamera.clamped(dim_0: 35.0, dim_1: 138.0, dim_2: 2000000.0, heading:...` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 35 | `// dim0 = dim_1 (x), dim1 = dim_0 (y) per resolveCoordinate` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 43 | `dim0: 140.0, // dim_1 (x)` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 44 | `dim1: 35.0,  // dim_0 (y)` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 55 | `dim0: -75.0, // dim_1 (x)` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 56 | `dim1: 50.0,   // dim_0 (y)` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 131 | `dim_0: lat,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 132 | `dim_1: lng,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 177 | `expect(controller.current.dim_0, 35.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 178 | `expect(controller.current.dim_1, 140.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 182 | `final double pannedLongitude = controller.current.dim_1;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 183 | `expect(pannedLongitude, greaterThan(140.0),` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 194 | `expect(afterController.current.dim_0, 35.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 195 | `reason: 'Camera dim_0 should remain at ViewA coordinate since we decoupled single-click');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 196 | `expect(afterController.current.dim_1, pannedLongitude,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 197 | `reason: 'Camera dim_1 should remain at panned coordinate since we decoupled single-click');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 208 | `final double pannedLat = controller.current.dim_0;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 209 | `final double pannedLng = controller.current.dim_1;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 217 | `expect(afterController.current.dim_0, pannedLat,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 218 | `reason: 'Camera dim_0 should be preserved when view is unchanged');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 219 | `expect(afterController.current.dim_1, pannedLng,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 220 | `reason: 'Camera dim_1 should be preserved when view is unchanged');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 250 | `expect(controller.current.dim_1, isNot(135.0));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 269 | `expect(afterController.current.dim_0, 50.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 270 | `expect(afterController.current.dim_1, -75.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 284 | `dim_0: 35.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 285 | `dim_1: 135.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 305 | `final double pannedLng = controller.current.dim_1;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 313 | `dim_0: 35.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 314 | `dim_1: 135.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 328 | `expect(afterController.current.dim_1, pannedLng,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 338 | `expect(controller.current.dim_0, 50.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 339 | `expect(controller.current.dim_1, -75.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 342 | `final double pannedLat = controller.current.dim_0;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 343 | `final double pannedLng = controller.current.dim_1;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 349 | `expect(afterController.current.dim_0, pannedLat,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 350 | `reason: 'Camera dim_0 preserved after tree notification');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 351 | `expect(afterController.current.dim_1, pannedLng,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 352 | `reason: 'Camera dim_1 preserved after tree notification');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 366 | `expect(afterNavController.current.dim_0, 35.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 367 | `expect(afterNavController.current.dim_1, 140.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 371 | `final double pannedLat = ctrl.current.dim_0;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 372 | `final double pannedLng = ctrl.current.dim_1;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 378 | `expect(afterController.current.dim_0, pannedLat,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 380 | `expect(afterController.current.dim_1, pannedLng,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 390 | `expect(controller.current.dim_0, 50.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 391 | `reason: 'Initial camera should be at ViewB dim_0');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 392 | `expect(controller.current.dim_1, -75.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 393 | `reason: 'Initial camera should be at ViewB dim_1');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 394 | `expect(controller.current.dim_0, isNot(35.0),` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 404 | `expect(controller.current.dim_0, 35.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 405 | `expect(controller.current.dim_1, 140.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 412 | `expect(afterSwitchCtrl.current.dim_0, 35.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 413 | `expect(afterSwitchCtrl.current.dim_1, 140.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 416 | `final double pannedLng = afterSwitchCtrl.current.dim_1;` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 422 | `expect(backCtrl.current.dim_0, 35.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 423 | `expect(backCtrl.current.dim_1, pannedLng);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 435 | `expect(bCtrl.current.dim_0, 35.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 437 | `expect(bCtrl.current.dim_1, 140.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 443 | `expect(aCtrl.current.dim_0, 35.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 445 | `expect(aCtrl.current.dim_1, 140.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 454 | `dim_0: lat,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 455 | `dim_1: lng,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 498 | `expect(controller.current.dim_0, closeTo(35.0, 0.1));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 499 | `expect(controller.current.dim_1, closeTo(140.0, 0.1));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 537 | `expect(controller.current.dim_0, isNot(closeTo(50.0, 0.1)),` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 538 | `reason: 'Camera should not jump to B dim_0');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 539 | `expect(controller.current.dim_1, isNot(closeTo(-75.0, 0.1)),` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 540 | `reason: 'Camera should not jump to B dim_1');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 542 | `expect(controller.current.dim_0, closeTo(35.0, 0.1),` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 543 | `reason: 'Camera should be on the flight path near 35.0 dim_0');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 544 | `expect(controller.current.dim_1, closeTo(140.0, 1.0),` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 545 | `reason: 'Camera should be on the flight path near 140.0 dim_1');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 550 | `expect(controller.current.dim_0, isNot(closeTo(50.0, 0.1)),` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 551 | `reason: 'Camera should NOT jump to B dim_0 at frame i');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 552 | `expect(controller.current.dim_1, isNot(closeTo(-75.0, 0.1)),` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 553 | `reason: 'Camera should NOT jump to B dim_1 at frame i');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 607 | `expect(controller.current.dim_0, -75.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/camera_reset_reproduction_test.dart` | 608 | `expect(controller.current.dim_1, 50.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/adversarial_fuzzer_test.dart` | 63 | `dim_0: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/adversarial_fuzzer_test.dart` | 64 | `dim_1: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/adversarial_fuzzer_test.dart` | 89 | `dim_0: lat,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/adversarial_fuzzer_test.dart` | 90 | `dim_1: lng,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/adversarial_fuzzer_test.dart` | 179 | `-(camera.dim_1 * math.pi / 180.0),` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d/adversarial_fuzzer_test.dart` | 180 | `-(camera.dim_0 * math.pi / 180.0),` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/double_click_fly_acceptance_test.dart` | 161 | `dim0: 139.7, // dim_1 (x)` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/double_click_fly_acceptance_test.dart` | 162 | `dim1: 35.6,  // dim_0 (y)` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/double_click_fly_acceptance_test.dart` | 173 | `dim0: -74.0, // dim_1 (x)` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/double_click_fly_acceptance_test.dart` | 174 | `dim1: 40.7,  // dim_0 (y)` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/double_click_fly_acceptance_test.dart` | 286 | `expect(controller.current.dim_0, 35.6,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/double_click_fly_acceptance_test.dart` | 287 | `reason: 'Initial camera should be centered on Node A dim_0');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/double_click_fly_acceptance_test.dart` | 288 | `expect(controller.current.dim_1, 139.7,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/double_click_fly_acceptance_test.dart` | 289 | `reason: 'Initial camera should be centered on Node A dim_1');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/double_click_fly_acceptance_test.dart` | 306 | `expect(controller.current.dim_0, 35.6,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/double_click_fly_acceptance_test.dart` | 307 | `reason: 'ACCEPTANCE CRITERIA: Camera dim_0 must NOT jump/move on single-click');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/double_click_fly_acceptance_test.dart` | 308 | `expect(controller.current.dim_1, 139.7,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/double_click_fly_acceptance_test.dart` | 309 | `reason: 'ACCEPTANCE CRITERIA: Camera dim_1 must NOT jump/move on single-click');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/double_click_fly_acceptance_test.dart` | 330 | `expect(controller.current.dim_0, greaterThan(35.6),` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/double_click_fly_acceptance_test.dart` | 331 | `reason: 'Camera dim_0 should have started moving towards Node B coordinates');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/double_click_fly_acceptance_test.dart` | 332 | `expect(controller.current.dim_1, isNot(139.7),` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/double_click_fly_acceptance_test.dart` | 333 | `reason: 'Camera dim_1 should have started moving towards Node B coordinates');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/double_click_fly_acceptance_test.dart` | 339 | `expect(controller.current.dim_0, 40.7,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/double_click_fly_acceptance_test.dart` | 340 | `reason: 'Camera should have arrived at Node B dim_0');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/double_click_fly_acceptance_test.dart` | 341 | `expect(controller.current.dim_1, -74.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/double_click_fly_acceptance_test.dart` | 342 | `reason: 'Camera should have arrived at Node B dim_1');` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/double_click_fly_acceptance_test.dart` | 380 | `expect(controller.current.dim_0, 35.6);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/double_click_fly_acceptance_test.dart` | 381 | `expect(controller.current.dim_1, 139.7);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/double_click_fly_acceptance_test.dart` | 408 | `expect(controller.current.dim_0, greaterThan(35.6));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/double_click_fly_acceptance_test.dart` | 409 | `expect(controller.current.dim_1, isNot(139.7));` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/double_click_fly_acceptance_test.dart` | 413 | `expect(controller.current.dim_0, 40.7);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/double_click_fly_acceptance_test.dart` | 414 | `expect(controller.current.dim_1, -74.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 13 | `dim_0: 37.7749,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 14 | `dim_1: -122.4194,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 21 | `expect(camera.dim_0, 37.7749);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 22 | `expect(camera.dim_1, -122.4194);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 30 | `test('Throws validation exception for invalid dim_0', () {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 33 | `dim_0: 95.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 34 | `dim_1: -122.4194,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 44 | `test('Throws validation exception for invalid dim_1', () {` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 47 | `dim_0: 37.7749,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 48 | `dim_1: -185.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 61 | `dim_0: 37.7749,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 62 | `dim_1: -122.4194,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 74 | `dim_0: 120.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 75 | `dim_1: -200.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 82 | `expect(camera.dim_0, 90.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 83 | `expect(camera.dim_1, -180.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 90 | `dim_0: double.nan,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 91 | `dim_1: -122.4194,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 101 | `dim_0: 37.7749,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 102 | `dim_1: double.infinity,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 114 | `dim_0: double.nan,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 115 | `dim_1: double.infinity,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 122 | `expect(camera.dim_0, 0.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 123 | `expect(camera.dim_1, 0.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 167 | `dim_0: 37.7749,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 168 | `dim_1: -122.4194,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 177 | `dim_0: 37.7749,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 178 | `dim_1: -122.4194,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 200 | `dim_0: 37.7749,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/cesium_3d_test.dart` | 201 | `dim_1: -122.4194,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scroll_zoom_test.dart` | 18 | `dim_0: 35.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scroll_zoom_test.dart` | 19 | `dim_1: 140.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/features/topology/globe_rendering_benchmark_test.dart` | 70 | `dim_0: 35.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/features/topology/globe_rendering_benchmark_test.dart` | 71 | `dim_1: 135.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/features/topology/globe_rendering_benchmark_test.dart` | 88 | `dim_0: 35.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/features/topology/globe_rendering_benchmark_test.dart` | 89 | `dim_1: 135.0 + (f * 0.1),` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 40 | `dim_0: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 41 | `dim_1: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 82 | `dim_0: 35.3606,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 83 | `dim_1: 138.7274,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 97 | `dim0: 138.7274, // dim_1` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 98 | `dim1: 35.3606,  // dim_0` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 166 | `dim_0: lat,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 167 | `dim_1: lng,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 189 | `final double rotationAngle = - (camera.dim_1 * math.pi / 180.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 190 | `final double tilt = - (camera.dim_0 * math.pi / 180.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 278 | `dim_0: 35.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 279 | `dim_1: 135.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 320 | `dim_0: 35.18,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 321 | `dim_1: 136.90,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 400 | `final double rotationAngle = - (camera.dim_1 * math.pi / 180.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 401 | `final double tilt = - (camera.dim_0 * math.pi / 180.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 425 | `dim_0: 35.18,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 426 | `dim_1: 136.90,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 466 | `dim_0: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 467 | `dim_1: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 506 | `dim_0: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 507 | `dim_1: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 545 | `dim_0: 35.3606,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 546 | `dim_1: 138.7274,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 602 | `final double rotationAngle = -(camera.dim_1 * math.pi / 180.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 603 | `final double tilt = -(camera.dim_0 * math.pi / 180.0);` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 629 | `dim_0: 35.3606,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 630 | `dim_1: 138.7274,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 673 | `dim_0: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 674 | `dim_1: 0.0,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 697 | `final double lng = 2.0; // Off-axis culled dim_1 to ensure non-zero perpendicular component` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 729 | `dim_0: 35.3606,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
| `app_flutter/test/topology/scene_3d_viewport_golden_test.dart` | 730 | `dim_1: 138.7274,` | Rename to domain-agnostic terms (e.g. x/y/z or physical dimensions) |
