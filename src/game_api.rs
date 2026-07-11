use crate::{RegistryEntry, Service, ServiceError};

/// Log severity level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// Host-side API surface provided to WASM plugins.
///
/// During the build/ready/finish lifecycle phases, the plugin receives
/// a `&mut dyn GameAPI` to interact with the engine.
///
/// # Implementations
///
/// - **Host (engine):** real implementation backed by ECS + wasmtime
/// - **Test:** `MockGameAPI` (behind `test-utils` feature) for unit-testing plugins natively
///
/// # Example
///
/// ```rust
/// use openstranded_plugin_api::{GameAPI, RegistryEntry, Service, ServiceError, Value, LogLevel};
///
/// fn plugin_build(api: &mut dyn GameAPI) {
///     let entries = api.registry_domain("items").unwrap();
///     for entry in &entries {
///         api.log(LogLevel::Info, &format!("Loaded {}", entry.filename));
///     }
/// }
/// ```
pub trait GameAPI {
    // ── Registry ──────────────────────────────────────────────────

    /// Get all file entries for a domain.
    fn registry_domain(&self, name: &str) -> Result<Vec<RegistryEntry>, ServiceError>;

    /// Get raw bytes of a specific file within a domain.
    fn registry_file(&self, domain: &str, filename: &str) -> Result<Vec<u8>, ServiceError>;

    // ── Services ─────────────────────────────────────────────────

    /// Register a service under the given domain name.
    fn register_service(
        &mut self,
        domain: &str,
        service: Box<dyn Service>,
    ) -> Result<(), ServiceError>;

    /// Get a reference to a registered service.
    fn get_service(&self, domain: &str) -> Result<&dyn Service, ServiceError>;

    /// Check whether a service domain is registered.
    fn has_service(&self, domain: &str) -> bool;

    // ── Logging ──────────────────────────────────────────────────

    /// Log a message at the given level.
    fn log(&self, level: LogLevel, message: &str);
}
