use crate::frame::{
    Frame, FrameError, SashId, GlobalCommand, GlobalEvent, GlobalEventType, 
    GlobalConfig, WindowConfig, EventListener, ListenerId
};

/// Primary interface for the Frame layer
pub trait FrameInterface {
    // ========== Lifecycle Management ==========
    
    /// Initialize a new Frame instance
    fn initialize() -> Result<Self, FrameError> where Self: Sized;
    
    /// Run the main application event loop
    /// This method blocks until the application should exit
    fn run(&mut self) -> Result<(), FrameError>;
    
    /// Shutdown the application gracefully
    fn shutdown(&mut self) -> Result<(), FrameError>;
    
    /// Check if the application should continue running
    fn should_continue(&self) -> bool;
    
    // ========== Window Management ==========
    
    /// Create a new window with default configuration
    fn create_window(&mut self) -> Result<SashId, FrameError> {
        self.create_window_with_config(None)
    }
    
    /// Create a new window with specific configuration
    fn create_window_with_config(&mut self, config: Option<WindowConfig>) -> Result<SashId, FrameError>;
    
    /// Destroy a specific window
    fn destroy_window(&mut self, id: SashId) -> Result<(), FrameError>;
    
    /// Get an immutable reference to a window
    fn get_window(&self, id: SashId) -> Option<&dyn crate::frame::SashInterface>;
    
    /// Get a mutable reference to a window
    fn get_window_mut(&mut self, id: SashId) -> Option<&mut (dyn crate::frame::SashInterface + '_)>;
    
    /// List all window IDs
    fn list_windows(&self) -> Vec<SashId>;
    
    /// Get the number of open windows
    fn window_count(&self) -> usize {
        self.list_windows().len()
    }
    
    // ========== Focus Management ==========
    
    /// Set the active window
    fn set_active_window(&mut self, id: SashId) -> Result<(), FrameError>;
    
    /// Get the currently active window ID
    fn get_active_window(&self) -> Option<SashId>;
    
    /// Move focus to the next window
    fn focus_next_window(&mut self) -> Result<(), FrameError>;
    
    /// Move focus to the previous window
    fn focus_previous_window(&mut self) -> Result<(), FrameError>;
    
    // ========== Command Handling ==========
    
    /// Execute a global command
    fn execute_command(&mut self, command: GlobalCommand) -> Result<(), FrameError>;
    
    /// Check if a command can be executed in the current state
    fn can_execute_command(&self, command: &GlobalCommand) -> bool;
    
    /// Get a list of available commands in the current state
    fn available_commands(&self) -> Vec<GlobalCommand>;
    
    // ========== Configuration Management ==========
    
    /// Get the current global configuration
    fn get_global_config(&self) -> &GlobalConfig;
    
    /// Update the global configuration
    fn update_global_config(&mut self, config: GlobalConfig) -> Result<(), FrameError>;
    
    /// Reload configuration from file
    fn reload_config(&mut self) -> Result<(), FrameError>;
    
    /// Save current configuration to file
    fn save_config(&self) -> Result<(), FrameError>;
    
    // ========== Event Management ==========
    
    /// Register an event listener
    fn register_event_listener(&mut self, 
                              event_type: GlobalEventType, 
                              listener: Box<dyn EventListener>) -> ListenerId;
    
    /// Unregister an event listener
    fn unregister_event_listener(&mut self, 
                                event_type: GlobalEventType, 
                                listener_id: ListenerId) -> bool;
    
    /// Emit a global event
    fn emit_event(&mut self, event: GlobalEvent) -> Result<(), FrameError>;
    
    // ========== State Inspection ==========
    
    /// Get the current application state
    fn get_application_state(&self) -> &crate::frame::ApplicationState;
    
    /// Get application statistics
    fn get_statistics(&self) -> FrameStatistics;
    
    /// Check if the application is in a valid state
    fn validate_state(&self) -> Result<(), FrameError>;
}

/// Implementation of FrameInterface for Frame
impl FrameInterface for Frame {
    fn initialize() -> Result<Self, FrameError> {
        let mut frame = Frame::new()?;
        
        // Load configuration
        match GlobalConfig::load() {
            Ok(config) => {
                config.validate()
                    .map_err(|e| FrameError::ConfigurationError(e))?;
                frame.global_config = config;
            }
            Err(e) => {
                // Log warning but continue with default config
                eprintln!("Warning: Could not load configuration, using defaults: {}", e);
            }
        }
        
        // Initialize command router with default handlers
        frame.command_router = crate::frame::commands::CommandRouter::new();
        
        // Set application state to running
        frame.app_state = crate::frame::ApplicationState::Running;
        
        // Emit application started event
        let _ = frame.event_dispatcher.dispatch(GlobalEvent::ApplicationStarted);
        
        Ok(frame)
    }
    
    fn run(&mut self) -> Result<(), FrameError> {
        // Create initial window if none exist and configured to do so
        if self.sashes.is_empty() && self.global_config.restore_windows_on_startup {
            let _initial_window = self.create_window_with_config(None)?;
        }
        
        // Main event loop
        while self.should_continue() {
            // TODO: Process platform events
            // TODO: Handle input
            // TODO: Update windows
            // TODO: Render
            
            // For now, just break to avoid infinite loop
            // This will be replaced with actual event processing
            break;
        }
        
        Ok(())
    }
    
    fn shutdown(&mut self) -> Result<(), FrameError> {
        // Emit shutdown event
        let _ = self.event_dispatcher.dispatch(GlobalEvent::ApplicationWillTerminate);
        
        // Set application state
        self.app_state = crate::frame::ApplicationState::ShuttingDown;
        
        // Run cleanup handlers
        let mut cleanup_errors = Vec::new();
        for handler in &self.cleanup_handlers {
            if let Err(e) = handler.cleanup() {
                cleanup_errors.push(e);
            }
        }
        
        // Close all windows
        let window_ids: Vec<SashId> = self.sashes.keys().copied().collect();
        for window_id in window_ids {
            if let Err(e) = self.destroy_window(window_id) {
                cleanup_errors.push(e);
            }
        }
        
        // Save configuration if needed
        if let Err(e) = self.save_config() {
            cleanup_errors.push(e);
        }
        
        // Clear event dispatcher
        self.event_dispatcher.clear();
        
        // Return first error if any occurred during cleanup
        if let Some(first_error) = cleanup_errors.into_iter().next() {
            return Err(first_error);
        }
        
        Ok(())
    }
    
    fn should_continue(&self) -> bool {
        !self.should_shutdown() && 
        self.app_state == crate::frame::ApplicationState::Running
    }
    
    fn create_window_with_config(&mut self, _config: Option<WindowConfig>) -> Result<SashId, FrameError> {
        let window_id = self.next_sash_id();
        
        // TODO: Create actual Sash implementation
        // For now, create a mock implementation
        let sash = Box::new(MockSash::new(window_id));
        
        self.sashes.insert(window_id, sash);
        
        // Set as active window if it's the first one
        if self.active_sash_id.is_none() {
            self.active_sash_id = Some(window_id);
        }
        
        // Emit window created event
        let _ = self.event_dispatcher.dispatch(GlobalEvent::WindowCreated(window_id));
        
        Ok(window_id)
    }
    
    fn destroy_window(&mut self, id: SashId) -> Result<(), FrameError> {
        if !self.sashes.contains_key(&id) {
            return Err(FrameError::WindowNotFound(id));
        }
        
        // Remove the window
        self.sashes.remove(&id);
        
        // Update active window if necessary
        if self.active_sash_id == Some(id) {
            self.active_sash_id = self.sashes.keys().next().copied();
        }
        
        // Emit window destroyed event
        let _ = self.event_dispatcher.dispatch(GlobalEvent::WindowDestroyed(id));
        
        Ok(())
    }
    
    fn get_window(&self, id: SashId) -> Option<&dyn crate::frame::SashInterface> {
        self.sashes.get(&id).map(|s| s.as_ref())
    }
    
    fn get_window_mut(&mut self, id: SashId) -> Option<&mut (dyn crate::frame::SashInterface + '_)> {
        if let Some(sash) = self.sashes.get_mut(&id) {
            Some(sash.as_mut())
        } else {
            None
        }
    }
    
    fn list_windows(&self) -> Vec<SashId> {
        self.sashes.keys().copied().collect()
    }
    
    fn set_active_window(&mut self, id: SashId) -> Result<(), FrameError> {
        if !self.sashes.contains_key(&id) {
            return Err(FrameError::WindowNotFound(id));
        }
        
        // Update focus state
        if let Some(old_id) = self.active_sash_id {
            if let Some(old_window) = self.sashes.get_mut(&old_id) {
                old_window.set_active(false);
            }
            let _ = self.event_dispatcher.dispatch(GlobalEvent::WindowUnfocused(old_id));
        }
        
        self.active_sash_id = Some(id);
        
        if let Some(new_window) = self.sashes.get_mut(&id) {
            new_window.set_active(true);
        }
        
        let _ = self.event_dispatcher.dispatch(GlobalEvent::WindowFocused(id));
        
        Ok(())
    }
    
    fn get_active_window(&self) -> Option<SashId> {
        self.active_sash_id
    }
    
    fn focus_next_window(&mut self) -> Result<(), FrameError> {
        let window_ids: Vec<SashId> = self.list_windows();
        if window_ids.is_empty() {
            return Ok(());
        }
        
        let current_index = self.active_sash_id
            .and_then(|id| window_ids.iter().position(|&w_id| w_id == id))
            .unwrap_or(0);
        
        let next_index = (current_index + 1) % window_ids.len();
        let next_id = window_ids[next_index];
        
        self.set_active_window(next_id)
    }
    
    fn focus_previous_window(&mut self) -> Result<(), FrameError> {
        let window_ids: Vec<SashId> = self.list_windows();
        if window_ids.is_empty() {
            return Ok(());
        }
        
        let current_index = self.active_sash_id
            .and_then(|id| window_ids.iter().position(|&w_id| w_id == id))
            .unwrap_or(0);
        
        let prev_index = if current_index == 0 {
            window_ids.len() - 1
        } else {
            current_index - 1
        };
        let prev_id = window_ids[prev_index];
        
        self.set_active_window(prev_id)
    }
    
    fn execute_command(&mut self, command: GlobalCommand) -> Result<(), FrameError> {
        // Extract command router temporarily to avoid borrowing conflicts
        let mut router = std::mem::take(&mut self.command_router);
        let result = router.route(command, self)
            .map_err(|e| FrameError::CommandFailed(e.to_string()));
        self.command_router = router;
        result
    }
    
    fn can_execute_command(&self, _command: &GlobalCommand) -> bool {
        // TODO: Implement command validation logic
        true
    }
    
    fn available_commands(&self) -> Vec<GlobalCommand> {
        // TODO: Return context-appropriate commands
        vec![]
    }
    
    fn get_global_config(&self) -> &GlobalConfig {
        &self.global_config
    }
    
    fn update_global_config(&mut self, config: GlobalConfig) -> Result<(), FrameError> {
        config.validate()
            .map_err(|e| FrameError::ConfigurationError(e))?;
        
        self.global_config = config;
        
        let _ = self.event_dispatcher.dispatch(
            GlobalEvent::ConfigurationChanged(
                crate::frame::events::ConfigChange::PerformanceSettings
            )
        );
        
        Ok(())
    }
    
    fn reload_config(&mut self) -> Result<(), FrameError> {
        let config = GlobalConfig::load()
            .map_err(|e| FrameError::ConfigurationError(e))?;
        
        self.update_global_config(config)?;
        
        let _ = self.event_dispatcher.dispatch(GlobalEvent::ConfigurationReloaded);
        
        Ok(())
    }
    
    fn save_config(&self) -> Result<(), FrameError> {
        self.global_config.save()
            .map_err(|e| FrameError::ConfigurationError(e))?;
        
        Ok(())
    }
    
    fn register_event_listener(&mut self, 
                              event_type: GlobalEventType, 
                              listener: Box<dyn EventListener>) -> ListenerId {
        self.event_dispatcher.subscribe(event_type, listener)
    }
    
    fn unregister_event_listener(&mut self, 
                                event_type: GlobalEventType, 
                                listener_id: ListenerId) -> bool {
        self.event_dispatcher.unsubscribe(event_type, listener_id)
    }
    
    fn emit_event(&mut self, event: GlobalEvent) -> Result<(), FrameError> {
        self.event_dispatcher.dispatch(event)
            .map_err(|e| FrameError::EventError(e))
    }
    
    fn get_application_state(&self) -> &crate::frame::ApplicationState {
        &self.app_state
    }
    
    fn get_statistics(&self) -> FrameStatistics {
        FrameStatistics {
            window_count: self.sashes.len(),
            active_window: self.active_sash_id,
            event_listener_count: self.event_dispatcher.listener_count(GlobalEventType::ApplicationLifecycle), // Example
            uptime: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default(),
        }
    }
    
    fn validate_state(&self) -> Result<(), FrameError> {
        // Validate global configuration
        self.global_config.validate()
            .map_err(|e| FrameError::ConfigurationError(e))?;
        
        // Validate active window exists if set
        if let Some(active_id) = self.active_sash_id {
            if !self.sashes.contains_key(&active_id) {
                return Err(FrameError::StateError(
                    format!("Active window {:?} does not exist", active_id)
                ));
            }
        }
        
        // Validate application state
        match self.app_state {
            crate::frame::ApplicationState::Initializing => {
                if !self.sashes.is_empty() {
                    return Err(FrameError::StateError(
                        "Windows exist during initialization".to_string()
                    ));
                }
            }
            crate::frame::ApplicationState::ShuttingDown => {
                // Allow any state during shutdown
            }
            _ => {
                // Normal states
            }
        }
        
        Ok(())
    }
}

/// Application statistics
#[derive(Debug, Clone)]
pub struct FrameStatistics {
    pub window_count: usize,
    pub active_window: Option<SashId>,
    pub event_listener_count: usize,
    pub uptime: std::time::Duration,
}

/// Mock Sash implementation for testing
struct MockSash {
    id: SashId,
    active: bool,
}

impl MockSash {
    fn new(id: SashId) -> Self {
        MockSash { id, active: false }
    }
}

impl crate::frame::SashInterface for MockSash {
    fn id(&self) -> SashId {
        self.id
    }
    
    fn is_active(&self) -> bool {
        self.active
    }
    
    fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_initialization() {
        let frame = Frame::initialize();
        assert!(frame.is_ok());
        
        let frame = frame.unwrap();
        assert_eq!(*frame.get_application_state(), crate::frame::ApplicationState::Running);
    }
    
    #[test]
    fn test_window_creation() {
        let mut frame = Frame::initialize().unwrap();
        
        let window_id = frame.create_window().unwrap();
        assert_eq!(frame.window_count(), 1);
        assert_eq!(frame.get_active_window(), Some(window_id));
    }
    
    #[test]
    fn test_window_focus() {
        let mut frame = Frame::initialize().unwrap();
        
        let window1 = frame.create_window().unwrap();
        let window2 = frame.create_window().unwrap();
        
        assert_eq!(frame.get_active_window(), Some(window1));
        
        frame.set_active_window(window2).unwrap();
        assert_eq!(frame.get_active_window(), Some(window2));
    }
    
    #[test]
    fn test_window_destruction() {
        let mut frame = Frame::initialize().unwrap();
        
        let window_id = frame.create_window().unwrap();
        assert_eq!(frame.window_count(), 1);
        
        frame.destroy_window(window_id).unwrap();
        assert_eq!(frame.window_count(), 0);
        assert_eq!(frame.get_active_window(), None);
    }
    
    #[test]
    fn test_state_validation() {
        let frame = Frame::initialize().unwrap();
        assert!(frame.validate_state().is_ok());
    }
}