pub mod config;
pub mod commands;
pub mod events;
pub mod errors;
pub mod interface;

#[cfg(test)]
mod tests;

pub use config::*;
pub use commands::*;
pub use events::*;
pub use errors::*;
pub use interface::*;

use std::collections::HashMap;

/// Unique identifier for a Sash (window)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SashId(pub u64);

impl SashId {
    pub fn new(id: u64) -> Self {
        SashId(id)
    }
    
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Application state tracking
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplicationState {
    Initializing,
    Running,
    Suspended,
    ShuttingDown,
}

/// Cleanup handler trait for shutdown coordination
pub trait CleanupHandler: Send + Sync {
    fn cleanup(&self) -> Result<(), FrameError>;
}

/// The main Frame structure - top-level application coordinator
pub struct Frame {
    // Window collection - owned and managed by Frame
    sashes: HashMap<SashId, Box<dyn SashInterface>>,
    active_sash_id: Option<SashId>,
    next_sash_id: SashId,
    
    // Application state
    app_state: ApplicationState,
    global_config: GlobalConfig,
    
    // Event and command coordination
    command_router: CommandRouter,
    event_dispatcher: EventDispatcher,
    
    // Shutdown coordination
    shutdown_requested: bool,
    cleanup_handlers: Vec<Box<dyn CleanupHandler>>,
}

impl Frame {
    /// Create a new Frame instance
    pub fn new() -> Result<Self, FrameError> {
        Ok(Frame {
            sashes: HashMap::new(),
            active_sash_id: None,
            next_sash_id: SashId::new(1),
            app_state: ApplicationState::Initializing,
            global_config: GlobalConfig::default(),
            command_router: CommandRouter::new(),
            event_dispatcher: EventDispatcher::new(),
            shutdown_requested: false,
            cleanup_handlers: Vec::new(),
        })
    }
    
    /// Get the next available SashId and increment the counter
    fn next_sash_id(&mut self) -> SashId {
        let id = self.next_sash_id;
        self.next_sash_id = SashId::new(self.next_sash_id.0 + 1);
        id
    }
    
    /// Check if shutdown has been requested
    pub fn should_shutdown(&self) -> bool {
        self.shutdown_requested || self.app_state == ApplicationState::ShuttingDown
    }
    
    /// Request application shutdown
    pub fn request_shutdown(&mut self) {
        self.shutdown_requested = true;
        self.app_state = ApplicationState::ShuttingDown;
    }
}

/// Placeholder trait for Sash interface - will be defined in sash layer
#[cfg_attr(test, mockall::automock)]
pub trait SashInterface: Send + Sync {
    fn id(&self) -> SashId;
    fn is_active(&self) -> bool;
    fn set_active(&mut self, active: bool);
    // More methods will be added as we develop the Sash layer
}

