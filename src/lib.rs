//! # openstranded-plugin-api
//!
//! Plugin SDK for `OpenStranded` — the core types and traits that all
//! WASM game plugins use to communicate with the engine and each other.
//!
//! ## Core types
//!
//! - [`Value`]: dynamic type for cross-plugin arguments and return values
//! - [`ServiceError`]: typed errors for Service API calls
//! - [`Service`] + [`ServiceRegistry`]: cross-plugin method call interface
//! - [`Registry`] + [`RegistryEntry`]: in-memory content pack data
//! - [`GameAPI`]: host-side API surface provided to plugins
//! - [`Contribution`]: declarative output from WASM plugin `build()` phase
//! - [`ApiVersion`]: compile-time baked version for compatibility checks
//!
//! ## Feature flags
//!
//! - `parse` (default): enables [`parse_registry_data`] and [`parse_registry_list`]
//! - `test-utils`: enables `MockGameAPI` for testing
//!
//! ## WASM entry points
//!
//! Every WASM plugin must export the following `#[no_mangle] extern "C"` functions:
//!
//! | Export | Required | Called during |
//! |--------|----------|---------------|
//! | `plugin_api_version() -> ApiVersion` | Yes | Load, before anything |
//! | `plugin_build(ctx) -> Vec<Contribution>` | Yes | Registry phase |
//! | `plugin_ready(api) -> bool` | No | Discovery phase |
//! | `plugin_finish(api)` | No | Integration phase |

mod value;
mod error;
mod service;
mod registry;
mod game_api;
mod contributions;
mod version;

#[cfg(feature = "test-utils")]
pub mod test_utils;

// ── Re-exports ─────────────────────────────────────────────────────

pub use value::Value;
pub use error::ServiceError;
pub use service::{Service, ServiceRegistry};
pub use registry::{Registry, RegistryEntry};
#[cfg(feature = "parse")]
pub use registry::{parse_registry_data, parse_registry_list};
pub use game_api::{GameAPI, LogLevel};
pub use contributions::{Contribution, SystemDecl, ResourceDecl};
pub use version::{ApiVersion, VersionMismatch};
