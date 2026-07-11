//! # openstranded-plugin-api
//!
//! Plugin SDK for `OpenStranded` — re-exports all contract types from
//! [`openstranded-common-wasmcontract`] and adds plugin-specific utilities.
//!
//! ## Re-exports
//!
//! All core types come from [`openstranded-common-wasmcontract`]:
//!
//! - [`Value`] — dynamic type for cross-plugin arguments and return values
//! - [`ServiceError`] — typed errors for Service API calls
//! - [`Service`] — cross-plugin method call interface (trait)
//! - [`Registry`] — in-memory content pack data store
//! - [`RegistryEntry`] — a single file from content pack (DTO)
//! - [`GameAPI`] — host-side API surface provided to plugins
//! - [`Contribution`] — declarative output from WASM plugin `build()` phase
//! - [`ApiVersion`] — compile-time baked version for compatibility checks
//! - [`LogLevel`] — log severity level
//!
//! ## Plugin-specific additions
//!
//! - [`MockGameAPI`](test_utils::MockGameAPI) (behind `test-utils` feature):
//!   mock implementation of [`GameAPI`] for unit-testing plugins natively
//!
//! ## Feature flags
//!
//! - `test-utils`: enables `MockGameAPI` and test utilities
//!
//! ## WASM entry points (planned)
//!
//! Every WASM plugin must export the following `#[no_mangle] extern "C"` functions:
//!
//! | Export | Required | Called during |
//! |--------|----------|---------------|
//! | `plugin_api_version() -> ApiVersion` | Yes | Load, before anything |
//! | `plugin_build(ctx) -> Vec<Contribution>` | Yes | Registry phase |
//! | `plugin_ready(api) -> bool` | No | Discovery phase |
//! | `plugin_finish(api)` | No | Integration phase |

// Re-export everything from the shared wasmcontract crate.
pub use openstranded_common_wasmcontract::*;

#[cfg(feature = "test-utils")]
pub mod test_utils;
