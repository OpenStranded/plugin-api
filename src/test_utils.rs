use std::cell::RefCell;
use std::collections::HashMap;

use crate::{
    GameAPI, LogLevel, RegistryEntry, Service, ServiceError,
};

/// A mock implementation of `GameAPI` for unit-testing plugins natively.
///
/// Stores all state in-memory (no WASM, no ECS). Useful for testing
/// plugin logic without spinning up the full engine.
///
/// # Example
///
/// ```rust
/// use openstranded_plugin_api::test_utils::MockGameAPI;
/// use openstranded_plugin_api::{GameAPI, Value, Service, ServiceError};
///
/// struct MyService;
/// impl Service for MyService {
///     fn call(&self, _method: &str, _args: &[Value]) -> Result<Value, ServiceError> {
///         Ok(Value::String("ok".into()))
///     }
/// }
///
/// let mut api = MockGameAPI::new();
/// api.register_service("my", Box::new(MyService)).unwrap();
/// assert!(api.has_service("my"));
///
/// // Check emitted logs
/// assert!(api.logs().is_empty());
/// ```
pub struct MockGameAPI {
    /// In-memory registry: domain → { filename → raw bytes }.
    registry: HashMap<String, HashMap<String, Vec<u8>>>,
    /// Registered services (domain → service).
    services: Vec<(String, Box<dyn Service>)>,
    /// Accumulated log messages (interior mutability for &self log access).
    logs: RefCell<Vec<(LogLevel, String)>>,
}

impl MockGameAPI {
    /// Create an empty mock API.
    #[must_use]
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
            services: Vec::new(),
            logs: RefCell::new(Vec::new()),
        }
    }

    /// Create a mock API pre-populated with a single registry file.
    ///
    /// Convenience for tests that need registry data.
    ///
    /// # Example
    ///
    /// ```rust
    /// use openstranded_plugin_api::test_utils::MockGameAPI;
    /// use openstranded_plugin_api::GameAPI;
    ///
    /// let api = MockGameAPI::with_registry_file(
    ///     "items", "items_edible.ron",
    ///     b"[{ \"id\": 1, \"name\": \"Wood\" }]"
    /// );
    /// let entries = api.registry_domain("items").unwrap();
    /// assert_eq!(entries.len(), 1);
    /// ```
    #[must_use]
    pub fn with_registry_file(domain: &str, filename: &str, data: &[u8]) -> Self {
        let mut reg: HashMap<String, HashMap<String, Vec<u8>>> = HashMap::new();
        reg.entry(domain.to_owned())
            .or_default()
            .insert(filename.to_owned(), data.to_vec());
        Self {
            registry: reg,
            services: Vec::new(),
            logs: RefCell::new(Vec::new()),
        }
    }

    /// Get all accumulated log messages (for assertions in tests).
    pub fn logs(&self) -> Vec<(LogLevel, String)> {
        self.logs.borrow().clone()
    }

    /// Check whether a specific log message exists.
    pub fn has_log(&self, level: LogLevel, msg_substring: &str) -> bool {
        self.logs
            .borrow()
            .iter()
            .any(|(l, m)| *l == level && m.contains(msg_substring))
    }

    /// Get all registered service domains.
    pub fn service_domains(&self) -> Vec<&str> {
        self.services.iter().map(|(d, _)| d.as_str()).collect()
    }
}

impl Default for MockGameAPI {
    fn default() -> Self {
        Self::new()
    }
}

impl GameAPI for MockGameAPI {
    fn registry_domain(&self, name: &str) -> Result<Vec<RegistryEntry>, ServiceError> {
        let files = self.registry.get(name).ok_or_else(|| {
            ServiceError::RegistryDomainNotFound(name.into())
        })?;
        Ok(files
            .iter()
            .map(|(filename, data)| RegistryEntry {
                filename: filename.clone(),
                data: data.clone(),
            })
            .collect())
    }

    fn registry_file(&self, domain: &str, filename: &str) -> Result<Vec<u8>, ServiceError> {
        self.registry
            .get(domain)
            .and_then(|files| files.get(filename))
            .cloned()
            .ok_or_else(|| ServiceError::FileNotFound {
                domain: domain.into(),
                filename: filename.into(),
            })
    }

    fn register_service(
        &mut self,
        domain: &str,
        service: Box<dyn Service>,
    ) -> Result<(), ServiceError> {
        if self.services.iter().any(|(d, _)| d == domain) {
            return Err(ServiceError::Internal(format!(
                "service domain '{domain}' is already registered"
            )));
        }
        self.services.push((domain.to_owned(), service));
        Ok(())
    }

    fn get_service(&self, domain: &str) -> Result<&dyn Service, ServiceError> {
        self.services
            .iter()
            .find(|(d, _)| d == domain)
            .map(|(_, s)| s.as_ref())
            .ok_or_else(|| ServiceError::DomainNotFound(domain.into()))
    }

    fn has_service(&self, domain: &str) -> bool {
        self.services.iter().any(|(d, _)| d == domain)
    }

    fn log(&self, level: LogLevel, message: &str) {
        self.logs.borrow_mut().push((level, message.to_owned()));
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Value;

    struct PingService;

    impl Service for PingService {
        fn call(&self, _method: &str, _args: &[Value]) -> Result<Value, ServiceError> {
            Ok(Value::String("pong".into()))
        }
    }

    #[test]
    fn test_mock_register_and_get_service() {
        let mut api = MockGameAPI::new();
        api.register_service("ping", Box::new(PingService)).unwrap();
        assert!(api.has_service("ping"));

        let svc = api.get_service("ping").unwrap();
        let result = svc.call("ping", &[]).unwrap();
        assert_eq!(result, Value::String("pong".into()));
    }

    #[test]
    fn test_mock_register_duplicate() {
        let mut api = MockGameAPI::new();
        api.register_service("x", Box::new(PingService)).unwrap();
        let err = api.register_service("x", Box::new(PingService)).unwrap_err();
        assert!(matches!(err, ServiceError::Internal(..)));
    }

    #[test]
    fn test_mock_get_service_not_found() {
        let api = MockGameAPI::new();
        let result = api.get_service("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_registry_domain() {
        let api = MockGameAPI::with_registry_file("test", "file.ron", b"hello");
        let entries = api.registry_domain("test").unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].filename, "file.ron");
        assert_eq!(entries[0].data, b"hello");
    }

    #[test]
    fn test_mock_registry_file() {
        let api = MockGameAPI::with_registry_file("test", "file.ron", b"hello");
        let data = api.registry_file("test", "file.ron").unwrap();
        assert_eq!(data, b"hello");
    }

    #[test]
    fn test_mock_registry_domain_not_found() {
        let api = MockGameAPI::new();
        let result = api.registry_domain("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_service_domains() {
        let mut api = MockGameAPI::new();
        api.register_service("a", Box::new(PingService)).unwrap();
        api.register_service("b", Box::new(PingService)).unwrap();
        let mut domains = api.service_domains();
        domains.sort();
        assert_eq!(domains, vec!["a", "b"]);
    }

    #[test]
    fn test_mock_log() {
        let api = MockGameAPI::new();
        api.log(LogLevel::Info, "hello world");
        api.log(LogLevel::Warn, "warning: something");

        let logs = api.logs();
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].0, LogLevel::Info);
        assert_eq!(logs[0].1, "hello world");

        assert!(api.has_log(LogLevel::Warn, "warning"));
    }
}
