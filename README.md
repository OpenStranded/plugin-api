# openstranded-plugin-api

Plugin SDK for OpenStranded — core types and traits that all WASM game
plugins use to communicate with the engine and each other.

## Core types

- `Value` — dynamic type for cross-plugin arguments and return values
- `ServiceError` — typed errors for Service API calls
- `Service` + `ServiceRegistry` — cross-plugin method call interface
- `Registry` + `RegistryEntry` — in-memory content pack data
- `GameAPI` — host-side API surface provided to plugins
- `Contribution` — declarative output from WASM plugin build phase
- `ApiVersion` — compile-time baked version for compatibility checks

## Feature flags

- `ron` (default) — enables `parse_registry_data` and `parse_registry_list` helpers
- `test-utils` — enables `MockGameAPI` for unit-testing plugins natively

## WASM entry points

Every WASM game plugin must export these `#[no_mangle] extern "C"` functions:

| Export | Required | Called during |
|--------|----------|---------------|
| `plugin_api_version() -> ApiVersion` | Yes | Load, before anything |
| `plugin_build(ctx) -> Vec<Contribution>` | Yes | Registry phase |
| `plugin_ready(api) -> bool` | No | Discovery phase |
| `plugin_finish(api)` | No | Integration phase |

## License

GPL-3.0-or-later
