use std::fmt;

/// Result type for Glazing layer operations
pub type GlazingResult<T> = Result<T, GlazingError>;

/// Errors that can occur in the Glazing layer
#[derive(Debug, thiserror::Error)]
pub enum GlazingError {
    #[error("Rendering error: {0}")]
    RenderingError(String),
    
    #[error("Theme error: {0}")]
    ThemeError(String),
    
    #[error("Layout error: {0}")]
    LayoutError(String),
    
    #[error("Font error: {0}")]
    FontError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Performance error: {0}")]
    PerformanceError(String),
    
    #[error("Backend error: {0}")]
    BackendError(String),
    
    #[error("Resource error: {0}")]
    ResourceError(String),
    
    #[error("Viewport error: {0}")]
    ViewportError(String),
    
    #[error("Color conversion error: {0}")]
    ColorError(String),
    
    #[error("Invalid state: {0}")]
    InvalidState(String),
    
    #[error("Not supported: {0}")]
    NotSupported(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    
    #[error("Memory error: {0}")]
    MemoryError(String),
    
    #[error("GPU error: {0}")]
    GpuError(String),
    
    #[error("Thread error: {0}")]
    ThreadError(String),
    
    #[error("Initialization error: {0}")]
    InitializationError(String),
    
    #[error("Shutdown error: {0}")]
    ShutdownError(String),
}

impl GlazingError {
    /// Create a rendering error
    pub fn rendering<S: Into<String>>(msg: S) -> Self {
        GlazingError::RenderingError(msg.into())
    }
    
    /// Create a theme error
    pub fn theme<S: Into<String>>(msg: S) -> Self {
        GlazingError::ThemeError(msg.into())
    }
    
    /// Create a layout error
    pub fn layout<S: Into<String>>(msg: S) -> Self {
        GlazingError::LayoutError(msg.into())
    }
    
    /// Create a font error
    pub fn font<S: Into<String>>(msg: S) -> Self {
        GlazingError::FontError(msg.into())
    }
    
    /// Create a configuration error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        GlazingError::ConfigError(msg.into())
    }
    
    /// Create a performance error
    pub fn performance<S: Into<String>>(msg: S) -> Self {
        GlazingError::PerformanceError(msg.into())
    }
    
    /// Create a backend error
    pub fn backend<S: Into<String>>(msg: S) -> Self {
        GlazingError::BackendError(msg.into())
    }
    
    /// Create a resource error
    pub fn resource<S: Into<String>>(msg: S) -> Self {
        GlazingError::ResourceError(msg.into())
    }
    
    /// Create a viewport error
    pub fn viewport<S: Into<String>>(msg: S) -> Self {
        GlazingError::ViewportError(msg.into())
    }
    
    /// Create a color error
    pub fn color<S: Into<String>>(msg: S) -> Self {
        GlazingError::ColorError(msg.into())
    }
    
    /// Create an invalid state error
    pub fn invalid_state<S: Into<String>>(msg: S) -> Self {
        GlazingError::InvalidState(msg.into())
    }
    
    /// Create a not supported error
    pub fn not_supported<S: Into<String>>(msg: S) -> Self {
        GlazingError::NotSupported(msg.into())
    }
    
    /// Create a parse error
    pub fn parse<S: Into<String>>(msg: S) -> Self {
        GlazingError::ParseError(msg.into())
    }
    
    /// Create a timeout error
    pub fn timeout<S: Into<String>>(msg: S) -> Self {
        GlazingError::TimeoutError(msg.into())
    }
    
    /// Create a memory error
    pub fn memory<S: Into<String>>(msg: S) -> Self {
        GlazingError::MemoryError(msg.into())
    }
    
    /// Create a GPU error
    pub fn gpu<S: Into<String>>(msg: S) -> Self {
        GlazingError::GpuError(msg.into())
    }
    
    /// Create a thread error
    pub fn thread<S: Into<String>>(msg: S) -> Self {
        GlazingError::ThreadError(msg.into())
    }
    
    /// Create an initialization error
    pub fn initialization<S: Into<String>>(msg: S) -> Self {
        GlazingError::InitializationError(msg.into())
    }
    
    /// Create a shutdown error
    pub fn shutdown<S: Into<String>>(msg: S) -> Self {
        GlazingError::ShutdownError(msg.into())
    }
    
    /// Check if this is a recoverable error
    pub fn is_recoverable(&self) -> bool {
        match self {
            GlazingError::RenderingError(_) => true,
            GlazingError::ThemeError(_) => true,
            GlazingError::LayoutError(_) => true,
            GlazingError::FontError(_) => false, // Font errors usually require restart
            GlazingError::ConfigError(_) => true,
            GlazingError::PerformanceError(_) => true,
            GlazingError::BackendError(_) => false, // Backend errors usually fatal
            GlazingError::ResourceError(_) => true,
            GlazingError::ViewportError(_) => true,
            GlazingError::ColorError(_) => true,
            GlazingError::InvalidState(_) => false, // State errors usually require reset
            GlazingError::NotSupported(_) => false,
            GlazingError::IoError(_) => false,
            GlazingError::ParseError(_) => true,
            GlazingError::TimeoutError(_) => true,
            GlazingError::MemoryError(_) => false, // Memory errors usually fatal
            GlazingError::GpuError(_) => true, // Can fall back to software rendering
            GlazingError::ThreadError(_) => false,
            GlazingError::InitializationError(_) => false,
            GlazingError::ShutdownError(_) => false,
        }
    }
    
    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            GlazingError::RenderingError(_) => ErrorSeverity::Warning,
            GlazingError::ThemeError(_) => ErrorSeverity::Warning,
            GlazingError::LayoutError(_) => ErrorSeverity::Warning,
            GlazingError::FontError(_) => ErrorSeverity::Error,
            GlazingError::ConfigError(_) => ErrorSeverity::Warning,
            GlazingError::PerformanceError(_) => ErrorSeverity::Info,
            GlazingError::BackendError(_) => ErrorSeverity::Fatal,
            GlazingError::ResourceError(_) => ErrorSeverity::Warning,
            GlazingError::ViewportError(_) => ErrorSeverity::Warning,
            GlazingError::ColorError(_) => ErrorSeverity::Warning,
            GlazingError::InvalidState(_) => ErrorSeverity::Error,
            GlazingError::NotSupported(_) => ErrorSeverity::Error,
            GlazingError::IoError(_) => ErrorSeverity::Error,
            GlazingError::ParseError(_) => ErrorSeverity::Warning,
            GlazingError::TimeoutError(_) => ErrorSeverity::Warning,
            GlazingError::MemoryError(_) => ErrorSeverity::Fatal,
            GlazingError::GpuError(_) => ErrorSeverity::Warning,
            GlazingError::ThreadError(_) => ErrorSeverity::Error,
            GlazingError::InitializationError(_) => ErrorSeverity::Fatal,
            GlazingError::ShutdownError(_) => ErrorSeverity::Error,
        }
    }
    
    /// Get suggested recovery action
    pub fn recovery_suggestion(&self) -> Option<String> {
        match self {
            GlazingError::RenderingError(_) => {
                Some("Try refreshing the display or reducing rendering quality".to_string())
            }
            GlazingError::ThemeError(_) => {
                Some("Try switching to the default theme".to_string())
            }
            GlazingError::LayoutError(_) => {
                Some("Try resizing the window or changing the layout".to_string())
            }
            GlazingError::FontError(_) => {
                Some("Check font installation and try using a different font".to_string())
            }
            GlazingError::ConfigError(_) => {
                Some("Check configuration settings and reset to defaults if needed".to_string())
            }
            GlazingError::PerformanceError(_) => {
                Some("Try reducing rendering quality or closing other applications".to_string())
            }
            GlazingError::GpuError(_) => {
                Some("Try disabling hardware acceleration".to_string())
            }
            GlazingError::MemoryError(_) => {
                Some("Close other applications to free memory".to_string())
            }
            _ => None,
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorSeverity {
    Info = 0,
    Warning = 1,
    Error = 2,
    Fatal = 3,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Info => write!(f, "INFO"),
            ErrorSeverity::Warning => write!(f, "WARNING"),
            ErrorSeverity::Error => write!(f, "ERROR"),
            ErrorSeverity::Fatal => write!(f, "FATAL"),
        }
    }
}

/// Error context for debugging and logging
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub component: String,
    pub timestamp: std::time::SystemTime,
    pub thread_id: String,
    pub additional_info: std::collections::HashMap<String, String>,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(operation: String, component: String) -> Self {
        ErrorContext {
            operation,
            component,
            timestamp: std::time::SystemTime::now(),
            thread_id: format!("{:?}", std::thread::current().id()),
            additional_info: std::collections::HashMap::new(),
        }
    }
    
    /// Add additional information to the context
    pub fn with_info<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.additional_info.insert(key.into(), value.into());
        self
    }
}

/// Enhanced error with context
#[derive(Debug)]//, Clone
pub struct ContextualError {
    pub error: GlazingError,
    pub context: ErrorContext,
}

impl ContextualError {
    /// Create a new contextual error
    pub fn new(error: GlazingError, context: ErrorContext) -> Self {
        ContextualError { error, context }
    }
}

impl fmt::Display for ContextualError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}:{}] {} - {}",
            self.context.component,
            self.context.operation,
            self.error.severity(),
            self.error
        )
    }
}

impl std::error::Error for ContextualError {}

/// Error recovery strategies
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Retry the operation
    Retry { max_attempts: u32, delay: std::time::Duration },
    
    /// Fall back to a simpler approach
    Fallback { description: String },
    
    /// Reset component state
    Reset { component: String },
    
    /// Switch to safe mode
    SafeMode,
    
    /// Abort operation gracefully
    Abort,
}

/// Error reporter for collecting and managing errors
#[derive(Debug)]
pub struct ErrorReporter {
    errors: Vec<ContextualError>,
    max_errors: usize,
    error_counts: std::collections::HashMap<String, usize>,
}

impl ErrorReporter {
    /// Create a new error reporter
    pub fn new(max_errors: usize) -> Self {
        ErrorReporter {
            errors: Vec::new(),
            max_errors,
            error_counts: std::collections::HashMap::new(),
        }
    }
    
    /// Report an error
    pub fn report(&mut self, error: ContextualError) {
        // Count error types
        let error_type = format!("{:?}", std::mem::discriminant(&error.error));
        *self.error_counts.entry(error_type).or_insert(0) += 1;
        
        // Add to error list
        self.errors.push(error);
        
        // Limit error history
        if self.errors.len() > self.max_errors {
            self.errors.remove(0);
        }
    }
    
    /// Get recent errors
    pub fn recent_errors(&self, count: usize) -> &[ContextualError] {
        let start = self.errors.len().saturating_sub(count);
        &self.errors[start..]
    }
    
    /// Get error statistics
    pub fn statistics(&self) -> ErrorStatistics {
        let total_errors = self.errors.len();
        let unique_types = self.error_counts.len();
        let most_common = self.error_counts
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(error_type, &count)| (error_type.clone(), count));
        
        ErrorStatistics {
            total_errors,
            unique_types,
            most_common,
            severity_distribution: self.calculate_severity_distribution(),
        }
    }
    
    /// Clear error history
    pub fn clear(&mut self) {
        self.errors.clear();
        self.error_counts.clear();
    }
    
    /// Calculate severity distribution
    fn calculate_severity_distribution(&self) -> std::collections::HashMap<ErrorSeverity, usize> {
        let mut distribution = std::collections::HashMap::new();
        
        for error in &self.errors {
            let severity = error.error.severity();
            *distribution.entry(severity).or_insert(0) += 1;
        }
        
        distribution
    }
}

/// Error statistics for monitoring
#[derive(Debug, Clone)]
pub struct ErrorStatistics {
    pub total_errors: usize,
    pub unique_types: usize,
    pub most_common: Option<(String, usize)>,
    pub severity_distribution: std::collections::HashMap<ErrorSeverity, usize>,
}

impl Default for ErrorReporter {
    fn default() -> Self {
        Self::new(1000) // Keep last 1000 errors
    }
}