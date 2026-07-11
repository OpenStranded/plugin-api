/// Declarative interface from WASM plugin to Host.
///
/// A WASM plugin's `plugin_build()` returns a list of `Contribution`s
/// describing what the plugin wants to add to the Bevy App. The Host
/// applies these contributions after collecting them from all plugins.
///
/// The plugin never receives `&mut App` directly.
#[derive(Clone, Debug)]
pub enum Contribution {
    /// Add a system to a Bevy schedule.
    System(SystemDecl),

    /// Add a resource with a default value.
    Resource(ResourceDecl),

    /// Add a known Bevy Plugin (referenced by name).
    Plugin(&'static str),

    /// Reserve a service domain (prevents conflicts).
    ServiceDomain(String),
}

/// Describes a system to be added to a Bevy schedule.
///
/// The actual system function lives in the WASM plugin and is called
/// through the host bridge at runtime.
#[derive(Clone, Debug)]
pub struct SystemDecl {
    /// Schedule label (e.g. "Update", "`FixedUpdate`", "Startup").
    pub schedule: String,
    /// Human-readable system name for logging / debugging.
    pub name: String,
}

/// Describes a resource to be registered in the Bevy World.
#[derive(Clone, Debug)]
pub struct ResourceDecl {
    /// Type name for dynamic registration (`TypeId` is unstable across WASM).
    pub type_name: String,
    /// Serialised default value (bincode / postcard).
    pub default_value: Vec<u8>,
}
