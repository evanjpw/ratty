use thiserror::Error;

/// Main error type for the Frame layer
#[derive(Debug, Error)]
pub enum FrameError {
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("Window not found: {0:?}")]
    WindowNotFound(crate::frame::SashId),
    
    #[error("Invalid window state: {0}")]
    InvalidWindowState(String),
    
    #[error("Command execution failed: {0}")]
    CommandFailed(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(#[from] ConfigError),
    
    #[error("Event dispatch error: {0}")]
    EventError(#[from] EventError),
    
    #[error("Shutdown error: {0}")]
    ShutdownError(String),
    
    #[error("Resource error: {0}")]
    ResourceError(String),
    
    #[error("State error: {0}")]
    StateError(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Thread error: {0}")]
    ThreadError(String),
    
    #[error("Timeout error: {0}")]
    TimeoutError(String),
}

/// Configuration-related errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to load configuration: {0}")]
    LoadFailed(String),
    
    #[error("Failed to save configuration: {0}")]
    SaveFailed(String),
    
    #[error("Configuration validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),
    
    #[error("Missing required configuration: {0}")]
    MissingRequired(String),
    
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),
    
    #[error("Configuration parse error: {0}")]
    ParseError(String),
    
    #[error("Configuration permission error: {0}")]
    PermissionError(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

/// Command execution errors
#[derive(Debug, Error)]
pub enum CommandError {
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    
    #[error("Command not supported: {0}")]
    NotSupported(String),
    
    #[error("No handler found for command: {0}")]
    NoHandler(String),
    
    #[error("Handler registration failed: {0}")]
    HandlerRegistrationFailed(String),
    
    #[error("Command timeout: {0}")]
    Timeout(String),
    
    #[error("Command validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Insufficient permissions: {0}")]
    PermissionDenied(String),
    
    #[error("Resource not available: {0}")]
    ResourceNotAvailable(String),
    
    #[error("State error: {0}")]
    StateError(String),
    
    #[error("Frame error: {0}")]
    FrameError(#[from] FrameError),
}

/// Event system errors
#[derive(Debug, Error)]
pub enum EventError {
    #[error("Event listener failed: {0}")]
    ListenerFailed(String),
    
    #[error("Event dispatch failed: {0}")]
    DispatchFailed(String),
    
    #[error("Invalid event: {0}")]
    InvalidEvent(String),
    
    #[error("Event serialization failed: {0}")]
    SerializationFailed(String),
    
    #[error("Event deserialization failed: {0}")]
    DeserializationFailed(String),
    
    #[error("Listener registration failed: {0}")]
    RegistrationFailed(String),
    
    #[error("Listener not found: {0:?}")]
    ListenerNotFound(crate::frame::events::ListenerId),
    
    #[error("Event queue overflow: {0}")]
    QueueOverflow(String),
    
    #[error("Event processing timeout: {0}")]
    ProcessingTimeout(String),
    
    #[error("Recursive event dispatch detected: {0}")]
    RecursiveDispatch(String),
    
    #[error("Thread error: {0}")]
    ThreadError(String),
}

/// Window management errors
#[derive(Debug, Error)]
pub enum WindowError {
    #[error("Window creation failed: {0}")]
    CreationFailed(String),
    
    #[error("Window destruction failed: {0}")]
    DestructionFailed(String),
    
    #[error("Window not found: {0:?}")]
    WindowNotFound(crate::frame::SashId),
    
    #[error("Window already exists: {0:?}")]
    WindowAlreadyExists(crate::frame::SashId),
    
    #[error("Invalid window configuration: {0}")]
    InvalidConfiguration(String),
    
    #[error("Window operation not allowed: {0}")]
    OperationNotAllowed(String),
    
    #[error("Window state error: {0}")]
    StateError(String),
    
    #[error("Window focus error: {0}")]
    FocusError(String),
    
    #[error("Window resize error: {0}")]
    ResizeError(String),
    
    #[error("Window move error: {0}")]
    MoveError(String),
}

/// Resource management errors
#[derive(Debug, Error)]
pub enum ResourceError {
    #[error("Resource allocation failed: {0}")]
    AllocationFailed(String),
    
    #[error("Resource deallocation failed: {0}")]
    DeallocationFailed(String),
    
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Resource already exists: {0}")]
    AlreadyExists(String),
    
    #[error("Resource access denied: {0}")]
    AccessDenied(String),
    
    #[error("Resource exhausted: {0}")]
    Exhausted(String),
    
    #[error("Resource lock failed: {0}")]
    LockFailed(String),
    
    #[error("Resource corruption detected: {0}")]
    Corrupted(String),
    
    #[error("Resource version mismatch: {0}")]
    VersionMismatch(String),
    
    #[error("Resource dependency error: {0}")]
    DependencyError(String),
}

/// Utility trait for converting errors to user-friendly messages
pub trait UserFriendlyError {
    fn user_message(&self) -> String;
    fn error_code(&self) -> Option<u32>;
    fn severity(&self) -> ErrorSeverity;
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl UserFriendlyError for FrameError {
    fn user_message(&self) -> String {
        match self {
            FrameError::InitializationFailed(_) => 
                "Failed to start the application. Please try again.".to_string(),
            FrameError::WindowNotFound(_) => 
                "The requested window could not be found.".to_string(),
            FrameError::InvalidWindowState(_) => 
                "The window is in an invalid state.".to_string(),
            FrameError::CommandFailed(_) => 
                "The requested operation could not be completed.".to_string(),
            FrameError::ConfigurationError(_) => 
                "There was a problem with the application configuration.".to_string(),
            FrameError::EventError(_) => 
                "An internal communication error occurred.".to_string(),
            FrameError::ShutdownError(_) => 
                "There was a problem shutting down the application.".to_string(),
            FrameError::ResourceError(_) => 
                "A system resource could not be accessed.".to_string(),
            FrameError::StateError(_) => 
                "The application is in an unexpected state.".to_string(),
            FrameError::Io(_) => 
                "A file or network operation failed.".to_string(),
            FrameError::ThreadError(_) => 
                "A background operation failed.".to_string(),
            FrameError::TimeoutError(_) => 
                "The operation timed out.".to_string(),
        }
    }
    
    fn error_code(&self) -> Option<u32> {
        match self {
            FrameError::InitializationFailed(_) => Some(1001),
            FrameError::WindowNotFound(_) => Some(1002),
            FrameError::InvalidWindowState(_) => Some(1003),
            FrameError::CommandFailed(_) => Some(1004),
            FrameError::ConfigurationError(_) => Some(1005),
            FrameError::EventError(_) => Some(1006),
            FrameError::ShutdownError(_) => Some(1007),
            FrameError::ResourceError(_) => Some(1008),
            FrameError::StateError(_) => Some(1009),
            FrameError::Io(_) => Some(1010),
            FrameError::ThreadError(_) => Some(1011),
            FrameError::TimeoutError(_) => Some(1012),
        }
    }
    
    fn severity(&self) -> ErrorSeverity {
        match self {
            FrameError::InitializationFailed(_) | FrameError::ShutdownError(_) => 
                ErrorSeverity::Critical,
            FrameError::WindowNotFound(_) | FrameError::InvalidWindowState(_) |
            FrameError::CommandFailed(_) => ErrorSeverity::Error,
            FrameError::ConfigurationError(_) | FrameError::EventError(_) |
            FrameError::ResourceError(_) | FrameError::StateError(_) => ErrorSeverity::Warning,
            FrameError::Io(_) | FrameError::ThreadError(_) | FrameError::TimeoutError(_) => 
                ErrorSeverity::Error,
        }
    }
}

/// Error context for better debugging
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub component: String,
    pub timestamp: std::time::SystemTime,
    pub stack_trace: Option<String>,
    pub additional_info: std::collections::HashMap<String, String>,
}

impl ErrorContext {
    pub fn new(operation: &str, component: &str) -> Self {
        ErrorContext {
            operation: operation.to_string(),
            component: component.to_string(),
            timestamp: std::time::SystemTime::now(),
            stack_trace: None,
            additional_info: std::collections::HashMap::new(),
        }
    }
    
    pub fn with_info(mut self, key: &str, value: &str) -> Self {
        self.additional_info.insert(key.to_string(), value.to_string());
        self
    }
    
    pub fn with_stack_trace(mut self, trace: String) -> Self {
        self.stack_trace = Some(trace);
        self
    }
}

/// Enhanced error type with context
#[derive(Debug)]
pub struct ContextualError {
    pub error: FrameError,
    pub context: ErrorContext,
}

impl std::fmt::Display for ContextualError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (in {} during {})", 
               self.error, 
               self.context.component, 
               self.context.operation)
    }
}

impl std::error::Error for ContextualError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion() {
        let config_error = ConfigError::LoadFailed("test".to_string());
        let frame_error: FrameError = config_error.into();
        
        match frame_error {
            FrameError::ConfigurationError(_) => {}, // Expected
            _ => panic!("Unexpected error type"),
        }
    }
    
    #[test]
    fn test_user_friendly_error() {
        let error = FrameError::InitializationFailed("test".to_string());
        let message = error.user_message();
        
        assert!(!message.is_empty());
        assert!(error.error_code().is_some());
        assert_eq!(error.severity(), ErrorSeverity::Critical);
    }
    
    #[test]
    fn test_error_context() {
        let context = ErrorContext::new("window_creation", "frame")
            .with_info("window_id", "123")
            .with_info("config", "default");
        
        assert_eq!(context.operation, "window_creation");
        assert_eq!(context.component, "frame");
        assert_eq!(context.additional_info.len(), 2);
    }
    
    #[test]
    fn test_contextual_error() {
        let error = FrameError::WindowNotFound(crate::frame::SashId::new(1));
        let context = ErrorContext::new("focus_window", "frame");
        let contextual = ContextualError { error, context };
        
        let error_string = format!("{}", contextual);
        assert!(error_string.contains("focus_window"));
        assert!(error_string.contains("frame"));
    }
}