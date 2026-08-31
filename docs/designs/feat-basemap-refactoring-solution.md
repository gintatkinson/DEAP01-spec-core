# Feature Solution Walkthrough: Basemap Configuration Refactoring

This document describes the design, implementation, and verification for the basemap configuration refactoring, which dynamically configures imagery provider endpoints in the application.

## 1. Description & Rationale
Previously, the basemap imagery URLs (OpenStreetMap, ArcGIS, CartoDB) were hardcoded directly in `TileFetcher.urlFor` inside `tile_fetcher.dart`. To allow runtime updates, alternative deployments, and config flexibility, these endpoints have been moved to `app_flutter/assets/persistence-config.json` and are now read at startup.

---

## 2. Code Realization Table

| Feature / Attribute | Source File | Class / Component | Method / Function / Member |
| :--- | :--- | :--- | :--- |
| Basemap URL Templates JSON config | [persistence-config.json](app_flutter/assets/persistence-config.json) | N/A | `basemaps` |
| Dynamic URL loading and mapping | [tile_fetcher.dart](app_flutter/lib/domain/cesium_3d/tile_fetcher.dart) | `TileFetcher` | `_basemapTemplates`, `configure`, `urlFor` |
| Startup configuration hook | [repository_resolver.dart](app_flutter/lib/domain/repository_resolver.dart) | `RepositoryResolver` | `resolve` |
| Test suite updates | [tile_fetcher_test.dart](app_flutter/test/cesium_3d/tile_fetcher_test.dart) | `TileFetcher` Tests | `setUp` |

---

## 3. Verification Details

### Automated Unit Tests
The unit tests in `tile_fetcher_test.dart` were updated to load the configuration from `persistence-config.json` in their setup block (with a fallback mapping if the assets file cannot be located by the test runner).

All tests execute successfully:
```bash
00:00 +0: loading app_flutter/test/cesium_3d/tile_fetcher_test.dart
00:00 +0: TileFetcher is enabled by default when MAP_IMAGERY_ENABLED is unset
00:00 +1: TileFetcher disable() causes fetchTile to return null
00:00 +2: TileFetcher urlFor produces HTTPS URLs for every provider
00:00 +3: TileFetcher cacheLength reports the number of cached entries
00:00 +4: TileFetcher clearCache empties stored entries
00:00 +5: TileFetcher supports file:// scheme via urlOverride
00:00 +6: TileCache stores and retrieves entries
00:00 +7: TileCache get returns null for missing keys
00:00 +8: TileCache clear removes all entries
00:00 +9: TileCache LRU eviction removes least-recently-used entry when full
00:00 +10: TileCache maxSize is correctly reported
00:00 +11: All tests passed!
```

All 246 tests in the broader `app_flutter` test suite passed successfully without regression.
