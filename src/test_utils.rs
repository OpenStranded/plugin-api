// openstranded-plugin-api — OpenStranded Plugin SDK — re-exports wasmcontract types + MockGameAPI
// Copyright (C) 2025  OpenStranded contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::cell::RefCell;
use std::collections::HashMap;

use crate::{
    GameAPI, LogLevel, RegistryEntry, Service, ServiceError, Value,
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
/// assert!(api.logs().is_empty());
/// ```
pub struct MockGameAPI {
    /// In-memory registry: domain → { filename → raw bytes }.
    registry: HashMap<String, HashMap<String, Vec<u8>>>,
    /// Registered services (domain → service).
    services: Vec<(String, Box<dyn Service>)>,
    /// Accumulated log messages (interior mutability for &self log access).
    logs: RefCell<Vec<(LogLevel, String)>>,
    /// Simulated config values (dotted key → Value).
    config: HashMap<String, Value>,
    /// Simulated keybinds.
    keybinds: HashMap<String, String>,
    /// In-memory save data store.
    save_data: HashMap<String, Vec<u8>>,
    /// Content files by path.
    content_files: HashMap<String, Vec<u8>>,
    /// Next entity ID for spawn_entity.
    next_entity: u64,
    /// Emitted events (for test assertions).
    events: RefCell<Vec<(String, Value)>>,
}

impl MockGameAPI {
    /// Create an empty mock API.
    #[must_use]
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
            services: Vec::new(),
            logs: RefCell::new(Vec::new()),
            config: HashMap::new(),
            keybinds: HashMap::new(),
            save_data: HashMap::new(),
            content_files: HashMap::new(),
            next_entity: 1,
            events: RefCell::new(Vec::new()),
        }
    }

    /// Create a mock API pre-populated with a single registry file.
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
            config: HashMap::new(),
            keybinds: HashMap::new(),
            save_data: HashMap::new(),
            content_files: HashMap::new(),
            next_entity: 1,
            events: RefCell::new(Vec::new()),
        }
    }

    /// Set a config value for testing (dotted key path).
    pub fn set_config(&mut self, key: &str, value: Value) {
        self.config.insert(key.to_owned(), value);
    }

    /// Set a keybinding for testing.
    pub fn set_keybind(&mut self, action: &str, key: &str) {
        self.keybinds.insert(action.to_owned(), key.to_owned());
    }

    /// Add a content file by path.
    pub fn add_content_file(&mut self, path: &str, data: Vec<u8>) {
        self.content_files.insert(path.to_owned(), data);
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

    /// Get all emitted events (for test assertions).
    pub fn events(&self) -> Vec<(String, Value)> {
        self.events.borrow().clone()
    }

    /// Check whether a specific event was emitted.
    pub fn has_event(&self, name: &str) -> bool {
        self.events.borrow().iter().any(|(n, _)| n == name)
    }
}

impl Default for MockGameAPI {
    fn default() -> Self {
        Self::new()
    }
}

impl GameAPI for MockGameAPI {
    // ── Registry ──────────────────────────────────────────────────

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

    // ── Content file access ──────────────────────────────────────

    fn read_content_file(&self, path: &str) -> Result<Vec<u8>, ServiceError> {
        self.content_files
            .get(path)
            .cloned()
            .ok_or_else(|| ServiceError::ContentFileNotFound(path.into()))
    }

    // ── Services ─────────────────────────────────────────────────

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

    fn call_service(
        &self,
        domain: &str,
        method: &str,
        args: &[Value],
    ) -> Result<Value, ServiceError> {
        let service = self.get_service(domain)?;
        service.call(method, args)
    }

    fn has_service(&self, domain: &str) -> bool {
        self.services.iter().any(|(d, _)| d == domain)
    }

    // ── ECS bridge (mock — minimal) ──────────────────────────────

    fn get_component(&self, _entity: u64, _type_name: &str) -> Result<Vec<u8>, ServiceError> {
        Err(ServiceError::Internal(
            "MockGameAPI does not support ECS operations".into(),
        ))
    }

    fn set_component(
        &mut self,
        _entity: u64,
        _type_name: &str,
        _data: &[u8],
    ) -> Result<(), ServiceError> {
        // Mock accepts silently (no-op).
        Ok(())
    }

    fn query_entities(&self, _component_name: &str) -> Result<Vec<u64>, ServiceError> {
        Ok(Vec::new()) // No entities in mock.
    }

    fn spawn_entity(&mut self, _archetype: &str) -> Result<u64, ServiceError> {
        let id = self.next_entity;
        self.next_entity += 1;
        Ok(id)
    }

    fn despawn_entity(&mut self, _entity: u64) -> Result<(), ServiceError> {
        Ok(())
    }

    // ── Configuration ────────────────────────────────────────────

    fn read_config(&self, key: &str) -> Result<Value, ServiceError> {
        Ok(self.config.get(key).cloned().unwrap_or(Value::Null))
    }

    fn read_keybinds(&self) -> Result<Value, ServiceError> {
        let map: HashMap<String, Value> = self
            .keybinds
            .iter()
            .map(|(k, v)| (k.clone(), Value::String(v.clone())))
            .collect();
        Ok(Value::Map(map))
    }

    // ── Save / Load ──────────────────────────────────────────────

    fn register_save_data(
        &mut self,
        domain: &str,
        data: Vec<u8>,
    ) -> Result<(), ServiceError> {
        self.save_data.insert(domain.to_owned(), data);
        Ok(())
    }

    fn load_save_data(&self, domain: &str) -> Result<Option<Vec<u8>>, ServiceError> {
        Ok(self.save_data.get(domain).cloned())
    }

    // ── Events ───────────────────────────────────────────────────

    fn emit_event(&self, name: &str, data: &Value) -> Result<(), ServiceError> {
        self.events.borrow_mut().push((name.to_owned(), data.clone()));
        Ok(())
    }

    // ── Logging ──────────────────────────────────────────────────

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
