use super::pty::PtyError;

/// Pane-specific error types
#[derive(Debug, thiserror::Error)]
pub enum PaneError {
    #[error("PTY error: {0}")]
    PtyError(#[from] PtyError),
    
    #[error("Terminal parse error: {0}")]
    ParseError(String),
    
    #[error("Invalid cursor position: row={0}, col={1}")]
    InvalidCursorPosition(u16, u16),
    
    #[error("Buffer overflow: attempted to write {0} bytes to {1} capacity")]
    BufferOverflow(usize, usize),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Process operation failed: {0}")]
    ProcessError(String),
    
    #[error("Search failed: {0}")]
    SearchError(String),
    
    #[error("Terminal state error: {0}")]
    StateError(String),
    
    #[error("Resize operation failed: {0}")]
    ResizeError(String),
    
    #[error("Input/Output error: {0}")]
    IoError(String),
    
    #[error("Theme application failed: {0}")]
    ThemeError(String),
    
    #[error("Event handling error: {0}")]
    EventError(String),
    
    #[error("Validation failed: {0}")]
    ValidationError(String),
    
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl PaneError {
    /// Create a parse error
    pub fn parse(msg: impl Into<String>) -> Self {
        PaneError::ParseError(msg.into())
    }
    
    /// Create a configuration error
    pub fn config(msg: impl Into<String>) -> Self {
        PaneError::ConfigError(msg.into())
    }
    
    /// Create a process error
    pub fn process(msg: impl Into<String>) -> Self {
        PaneError::ProcessError(msg.into())
    }
    
    /// Create a state error
    pub fn state(msg: impl Into<String>) -> Self {
        PaneError::StateError(msg.into())
    }
    
    /// Create an I/O error
    pub fn io(msg: impl Into<String>) -> Self {
        PaneError::IoError(msg.into())
    }
    
    /// Create a validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        PaneError::ValidationError(msg.into())
    }
    
    /// Create a not implemented error
    pub fn not_implemented(feature: impl Into<String>) -> Self {
        PaneError::NotImplemented(feature.into())
    }
    
    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            PaneError::PtyError(pty_err) => match pty_err {
                PtyError::ProcessNotRunning => true,
                PtyError::InvalidSize { .. } => true,
                _ => false,
            },
            PaneError::ParseError(_) => true,
            PaneError::InvalidCursorPosition(_, _) => true,
            PaneError::ConfigError(_) => true,
            PaneError::SearchError(_) => true,
            PaneError::ThemeError(_) => true,
            PaneError::ValidationError(_) => true,
            PaneError::NotImplemented(_) => true,
            _ => false,
        }
    }
    
    /// Get error category for reporting
    pub fn category(&self) -> ErrorCategory {
        match self {
            PaneError::PtyError(_) => ErrorCategory::Process,
            PaneError::ParseError(_) => ErrorCategory::Terminal,
            PaneError::InvalidCursorPosition(_, _) => ErrorCategory::Terminal,
            PaneError::BufferOverflow(_, _) => ErrorCategory::Memory,
            PaneError::ConfigError(_) => ErrorCategory::Configuration,
            PaneError::ProcessError(_) => ErrorCategory::Process,
            PaneError::SearchError(_) => ErrorCategory::Content,
            PaneError::StateError(_) => ErrorCategory::State,
            PaneError::ResizeError(_) => ErrorCategory::Display,
            PaneError::IoError(_) => ErrorCategory::IO,
            PaneError::ThemeError(_) => ErrorCategory::Display,
            PaneError::EventError(_) => ErrorCategory::Event,
            PaneError::ValidationError(_) => ErrorCategory::Validation,
            PaneError::NotImplemented(_) => ErrorCategory::Implementation,
            PaneError::InternalError(_) => ErrorCategory::Internal,
        }
    }
}

/// Error categories for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    Process,
    Terminal,
    Memory,
    Configuration,
    Content,
    State,
    Display,
    IO,
    Event,
    Validation,
    Implementation,
    Internal,
}

impl ErrorCategory {
    /// Get human-readable category name
    pub fn name(&self) -> &'static str {
        match self {
            ErrorCategory::Process => "Process Management",
            ErrorCategory::Terminal => "Terminal Emulation",
            ErrorCategory::Memory => "Memory Management",
            ErrorCategory::Configuration => "Configuration",
            ErrorCategory::Content => "Content Management",
            ErrorCategory::State => "State Management",
            ErrorCategory::Display => "Display/Rendering",
            ErrorCategory::IO => "Input/Output",
            ErrorCategory::Event => "Event Handling",
            ErrorCategory::Validation => "Validation",
            ErrorCategory::Implementation => "Implementation",
            ErrorCategory::Internal => "Internal Error",
        }
    }
    
    /// Get suggested user action for this category
    pub fn user_action(&self) -> &'static str {
        match self {
            ErrorCategory::Process => "Check process permissions and availability",
            ErrorCategory::Terminal => "Verify terminal compatibility",
            ErrorCategory::Memory => "Free up system memory",
            ErrorCategory::Configuration => "Check configuration settings",
            ErrorCategory::Content => "Verify content format",
            ErrorCategory::State => "Restart the pane",
            ErrorCategory::Display => "Check display settings",
            ErrorCategory::IO => "Check file permissions and network connectivity",
            ErrorCategory::Event => "Report this issue",
            ErrorCategory::Validation => "Check input parameters",
            ErrorCategory::Implementation => "Feature not yet available",
            ErrorCategory::Internal => "Report this issue",
        }
    }
}

pub type PaneResult<T> = Result<T, PaneError>;

/// Convert std::io::Error to PaneError
impl From<std::io::Error> for PaneError {
    fn from(err: std::io::Error) -> Self {
        PaneError::IoError(err.to_string())
    }
}

/// Convert string errors to PaneError
impl From<String> for PaneError {
    fn from(err: String) -> Self {
        PaneError::InternalError(err)
    }
}

impl From<&str> for PaneError {
    fn from(err: &str) -> Self {
        PaneError::InternalError(err.to_string())
    }
}