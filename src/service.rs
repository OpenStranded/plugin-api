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
