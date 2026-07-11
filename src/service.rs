use std::collections::HashMap;

use crate::{ServiceError, Value};

/// Unified interface for cross-plugin calls.
///
/// Each plugin registers one or more services that other plugins can call
/// through `Service::call()`.
///
/// # Example
///
/// ```rust
/// use openstranded_plugin_api::{Service, Value, ServiceError};
///
/// struct HealthService;
///
/// impl Service for HealthService {
///     fn call(&self, method: &str, args: &[Value]) -> Result<Value, ServiceError> {
///         match method {
///             "get_max" => Ok(Value::U32(100)),
///             _ => Err(ServiceError::UnknownMethod(method.into())),
///         }
///     }
/// }
/// ```
pub trait Service: Send + Sync {
    /// Call a method on this service.
    ///
    /// - `method`: method name (case-sensitive string)
    /// - `args`: call arguments
    fn call(&self, method: &str, args: &[Value]) -> Result<Value, ServiceError>;
}

/// Host-side registry of all registered services.
///
/// Stores services registered by plugins and provides access by domain name.
#[derive(Default)]
pub struct ServiceRegistry {
    services: HashMap<String, Box<dyn Service>>,
}

impl ServiceRegistry {
    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    /// Register a service under the given domain name.
    ///
    /// Returns an error if the domain is already taken.
    pub fn register(
        &mut self,
        domain: &str,
        service: Box<dyn Service>,
    ) -> Result<(), ServiceError> {
        let domain = domain.to_owned();
        if self.services.contains_key(&domain) {
            return Err(ServiceError::Internal(format!(
                "service domain '{domain}' is already registered"
            )));
        }
        self.services.insert(domain, service);
        Ok(())
    }

    /// Get a reference to a service by domain name.
    pub fn get(&self, domain: &str) -> Result<&dyn Service, ServiceError> {
        self.services
            .get(domain)
            .map(std::convert::AsRef::as_ref)
            .ok_or_else(|| ServiceError::DomainNotFound(domain.into()))
    }

    /// Check whether a domain is registered.
    #[must_use]
    pub fn has(&self, domain: &str) -> bool {
        self.services.contains_key(domain)
    }

    /// Remove a service by domain name.
    pub fn unregister(&mut self, domain: &str) -> Option<Box<dyn Service>> {
        self.services.remove(domain)
    }

    /// Number of registered services.
    #[must_use]
    pub fn len(&self) -> usize {
        self.services.len()
    }

    /// Whether the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.services.is_empty()
    }

    /// Iterate over all registered domain names.
    pub fn domains(&self) -> impl Iterator<Item = &str> {
        self.services.keys().map(std::string::String::as_str)
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    struct TestService;

    impl Service for TestService {
        fn call(&self, method: &str, _args: &[Value]) -> Result<Value, ServiceError> {
            match method {
                "ping" => Ok(Value::String("pong".into())),
                _ => Err(ServiceError::UnknownMethod(method.into())),
            }
        }
    }

    #[test]
    fn test_register_and_get() {
        let mut registry = ServiceRegistry::new();
        registry.register("test", Box::new(TestService)).unwrap();
        let svc = registry.get("test").unwrap();
        let result = svc.call("ping", &[]).unwrap();
        assert_eq!(result, Value::String("pong".into()));
    }

    #[test]
    fn test_get_unknown_domain() {
        let registry = ServiceRegistry::new();
        let result = registry.get("nonexistent");
        assert!(result.is_err());
        // ServiceError:DomainNotFound — checked by type
    }

    #[test]
    fn test_register_duplicate() {
        let mut registry = ServiceRegistry::new();
        registry.register("test", Box::new(TestService)).unwrap();
        let err = registry.register("test", Box::new(TestService)).unwrap_err();
        assert!(matches!(err, ServiceError::Internal(..)));
    }

    #[test]
    fn test_has() {
        let mut registry = ServiceRegistry::new();
        assert!(!registry.has("test"));
        registry.register("test", Box::new(TestService)).unwrap();
        assert!(registry.has("test"));
    }

    #[test]
    fn test_unregister() {
        let mut registry = ServiceRegistry::new();
        registry.register("test", Box::new(TestService)).unwrap();
        assert!(registry.unregister("test").is_some());
        assert!(!registry.has("test"));
    }

    #[test]
    fn test_domains_iter() {
        let mut registry = ServiceRegistry::new();
        registry.register("a", Box::new(TestService)).unwrap();
        registry.register("b", Box::new(TestService)).unwrap();
        let domains: Vec<&str> = registry.domains().collect();
        assert_eq!(domains.len(), 2);
        assert!(domains.contains(&"a"));
        assert!(domains.contains(&"b"));
    }
}
