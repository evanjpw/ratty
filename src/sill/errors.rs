use std::fmt;

/// Result type for Sill layer operations
pub type SillResult<T> = Result<T, SillError>;

/// Errors that can occur in the Sill layer
#[derive(Debug, Clone)]
pub enum SillError {
    /// Input processing errors
    KeyProcessing(String),
    MouseProcessing(String),
    InputNormalization(String),
    InvalidSequence(String),
    
    /// Event routing errors
    RoutingError(String),
    InvalidTarget(String),
    FocusError(String),
    
    /// Selection errors
    SelectionError(String),
    NoSelection(String),
    InvalidSelection(String),
    
    /// Clipboard errors
    ClipboardAccess(String),
    ClipboardFormat(String),
    ClipboardUnavailable(String),
    
    /// Configuration errors
    ConfigurationError(String),
    InvalidMapping(String),
    InvalidMode(String),
    
    /// Platform errors
    PlatformError(String),
    PermissionDenied(String),
    ResourceUnavailable(String),
    
    /// Performance errors
    InputOverload(String),
    ProcessingTimeout(String),
    
    /// Generic errors
    Internal(String),
    NotImplemented(String),
}

impl SillError {
    /// Create a key processing error
    pub fn key_processing(msg: &str) -> Self {
        SillError::KeyProcessing(msg.to_string())
    }
    
    /// Create a mouse processing error
    pub fn mouse_processing(msg: &str) -> Self {
        SillError::MouseProcessing(msg.to_string())
    }
    
    /// Create an input normalization error
    pub fn input_normalization(msg: &str) -> Self {
        SillError::InputNormalization(msg.to_string())
    }
    
    /// Create an invalid sequence error
    pub fn invalid_sequence(msg: &str) -> Self {
        SillError::InvalidSequence(msg.to_string())
    }
    
    /// Create a routing error
    pub fn routing(msg: &str) -> Self {
        SillError::RoutingError(msg.to_string())
    }
    
    /// Create an invalid target error
    pub fn invalid_target(msg: &str) -> Self {
        SillError::InvalidTarget(msg.to_string())
    }
    
    /// Create a focus error
    pub fn focus(msg: &str) -> Self {
        SillError::FocusError(msg.to_string())
    }
    
    /// Create a selection error
    pub fn selection(msg: &str) -> Self {
        SillError::SelectionError(msg.to_string())
    }
    
    /// Create a no selection error
    pub fn no_selection(msg: &str) -> Self {
        SillError::NoSelection(msg.to_string())
    }
    
    /// Create an invalid selection error
    pub fn invalid_selection(msg: &str) -> Self {
        SillError::InvalidSelection(msg.to_string())
    }
    
    /// Create a clipboard access error
    pub fn clipboard_access(msg: &str) -> Self {
        SillError::ClipboardAccess(msg.to_string())
    }
    
    /// Create a clipboard format error
    pub fn clipboard_format(msg: &str) -> Self {
        SillError::ClipboardFormat(msg.to_string())
    }
    
    /// Create a clipboard unavailable error
    pub fn clipboard_unavailable(msg: &str) -> Self {
        SillError::ClipboardUnavailable(msg.to_string())
    }
    
    /// Create a configuration error
    pub fn configuration(msg: &str) -> Self {
        SillError::ConfigurationError(msg.to_string())
    }
    
    /// Create an invalid mapping error
    pub fn invalid_mapping(msg: &str) -> Self {
        SillError::InvalidMapping(msg.to_string())
    }
    
    /// Create an invalid mode error
    pub fn invalid_mode(msg: &str) -> Self {
        SillError::InvalidMode(msg.to_string())
    }
    
    /// Create a platform error
    pub fn platform(msg: &str) -> Self {
        SillError::PlatformError(msg.to_string())
    }
    
    /// Create a permission denied error
    pub fn permission_denied(msg: &str) -> Self {
        SillError::PermissionDenied(msg.to_string())
    }
    
    /// Create a resource unavailable error
    pub fn resource_unavailable(msg: &str) -> Self {
        SillError::ResourceUnavailable(msg.to_string())
    }
    
    /// Create an input overload error
    pub fn input_overload(msg: &str) -> Self {
        SillError::InputOverload(msg.to_string())
    }
    
    /// Create a processing timeout error
    pub fn processing_timeout(msg: &str) -> Self {
        SillError::ProcessingTimeout(msg.to_string())
    }
    
    /// Create an internal error
    pub fn internal(msg: &str) -> Self {
        SillError::Internal(msg.to_string())
    }
    
    /// Create a not implemented error
    pub fn not_implemented(msg: &str) -> Self {
        SillError::NotImplemented(msg.to_string())
    }
    
    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            SillError::PermissionDenied(_) => false,
            SillError::ResourceUnavailable(_) => false,
            SillError::PlatformError(_) => false,
            SillError::Internal(_) => false,
            _ => true,
        }
    }
    
    /// Get error severity
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            SillError::KeyProcessing(_) => ErrorSeverity::Warning,
            SillError::MouseProcessing(_) => ErrorSeverity::Warning,
            SillError::InputNormalization(_) => ErrorSeverity::Warning,
            SillError::InvalidSequence(_) => ErrorSeverity::Info,
            SillError::RoutingError(_) => ErrorSeverity::Error,
            SillError::InvalidTarget(_) => ErrorSeverity::Warning,
            SillError::FocusError(_) => ErrorSeverity::Warning,
            SillError::SelectionError(_) => ErrorSeverity::Warning,
            SillError::NoSelection(_) => ErrorSeverity::Info,
            SillError::InvalidSelection(_) => ErrorSeverity::Warning,
            SillError::ClipboardAccess(_) => ErrorSeverity::Error,
            SillError::ClipboardFormat(_) => ErrorSeverity::Warning,
            SillError::ClipboardUnavailable(_) => ErrorSeverity::Warning,
            SillError::ConfigurationError(_) => ErrorSeverity::Error,
            SillError::InvalidMapping(_) => ErrorSeverity::Warning,
            SillError::InvalidMode(_) => ErrorSeverity::Warning,
            SillError::PlatformError(_) => ErrorSeverity::Fatal,
            SillError::PermissionDenied(_) => ErrorSeverity::Fatal,
            SillError::ResourceUnavailable(_) => ErrorSeverity::Error,
            SillError::InputOverload(_) => ErrorSeverity::Warning,
            SillError::ProcessingTimeout(_) => ErrorSeverity::Warning,
            SillError::Internal(_) => ErrorSeverity::Fatal,
            SillError::NotImplemented(_) => ErrorSeverity::Error,
        }
    }
    
    /// Get recovery suggestion for the error
    pub fn recovery_suggestion(&self) -> Option<&'static str> {
        match self {
            SillError::KeyProcessing(_) => Some("Check keyboard input mode and mappings"),
            SillError::MouseProcessing(_) => Some("Verify mouse mode settings"),
            SillError::InputNormalization(_) => Some("Check input encoding and platform settings"),
            SillError::InvalidSequence(_) => Some("Review input sequence configuration"),
            SillError::RoutingError(_) => Some("Check event routing configuration"),
            SillError::InvalidTarget(_) => Some("Verify target pane exists and is accessible"),
            SillError::FocusError(_) => Some("Check focus management state"),
            SillError::SelectionError(_) => Some("Clear selection and try again"),
            SillError::NoSelection(_) => Some("Make a text selection first"),
            SillError::InvalidSelection(_) => Some("Check selection boundaries"),
            SillError::ClipboardAccess(_) => Some("Check clipboard permissions and system state"),
            SillError::ClipboardFormat(_) => Some("Try copying as plain text"),
            SillError::ClipboardUnavailable(_) => Some("Wait and retry clipboard operation"),
            SillError::ConfigurationError(_) => Some("Reset to default configuration"),
            SillError::InvalidMapping(_) => Some("Check key mapping syntax"),
            SillError::InvalidMode(_) => Some("Switch to a supported input mode"),
            SillError::PlatformError(_) => None,
            SillError::PermissionDenied(_) => Some("Check application permissions"),
            SillError::ResourceUnavailable(_) => Some("Free up system resources and retry"),
            SillError::InputOverload(_) => Some("Reduce input rate or enable throttling"),
            SillError::ProcessingTimeout(_) => Some("Increase processing timeout limits"),
            SillError::Internal(_) => None,
            SillError::NotImplemented(_) => Some("Feature not yet available"),
        }
    }
}

impl fmt::Display for SillError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SillError::KeyProcessing(msg) => write!(f, "Key processing error: {}", msg),
            SillError::MouseProcessing(msg) => write!(f, "Mouse processing error: {}", msg),
            SillError::InputNormalization(msg) => write!(f, "Input normalization error: {}", msg),
            SillError::InvalidSequence(msg) => write!(f, "Invalid input sequence: {}", msg),
            SillError::RoutingError(msg) => write!(f, "Event routing error: {}", msg),
            SillError::InvalidTarget(msg) => write!(f, "Invalid target: {}", msg),
            SillError::FocusError(msg) => write!(f, "Focus error: {}", msg),
            SillError::SelectionError(msg) => write!(f, "Selection error: {}", msg),
            SillError::NoSelection(msg) => write!(f, "No selection: {}", msg),
            SillError::InvalidSelection(msg) => write!(f, "Invalid selection: {}", msg),
            SillError::ClipboardAccess(msg) => write!(f, "Clipboard access error: {}", msg),
            SillError::ClipboardFormat(msg) => write!(f, "Clipboard format error: {}", msg),
            SillError::ClipboardUnavailable(msg) => write!(f, "Clipboard unavailable: {}", msg),
            SillError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            SillError::InvalidMapping(msg) => write!(f, "Invalid mapping: {}", msg),
            SillError::InvalidMode(msg) => write!(f, "Invalid mode: {}", msg),
            SillError::PlatformError(msg) => write!(f, "Platform error: {}", msg),
            SillError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            SillError::ResourceUnavailable(msg) => write!(f, "Resource unavailable: {}", msg),
            SillError::InputOverload(msg) => write!(f, "Input overload: {}", msg),
            SillError::ProcessingTimeout(msg) => write!(f, "Processing timeout: {}", msg),
            SillError::Internal(msg) => write!(f, "Internal error: {}", msg),
            SillError::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
        }
    }
}

impl std::error::Error for SillError {}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Fatal,
}

/// Error context for debugging and logging
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub component: String,
    pub timestamp: std::time::Instant,
    pub thread_id: String,
}

impl ErrorContext {
    pub fn new(operation: String, component: String) -> Self {
        ErrorContext {
            operation,
            component,
            timestamp: std::time::Instant::now(),
            thread_id: format!("{:?}", std::thread::current().id()),
        }
    }
}

/// Contextual error with additional debugging information
#[derive(Debug, Clone)]
pub struct ContextualError {
    pub error: SillError,
    pub context: ErrorContext,
}

impl ContextualError {
    pub fn new(error: SillError, context: ErrorContext) -> Self {
        ContextualError { error, context }
    }
}

/// Error reporter for collecting and analyzing errors
#[derive(Debug)]
pub struct ErrorReporter {
    errors: Vec<ContextualError>,
    max_errors: usize,
    error_counts: std::collections::HashMap<String, u64>,
}

impl ErrorReporter {
    pub fn new(max_errors: usize) -> Self {
        ErrorReporter {
            errors: Vec::new(),
            max_errors,
            error_counts: std::collections::HashMap::new(),
        }
    }
    
    /// Report a new error
    pub fn report(&mut self, error: ContextualError) {
        let error_type = format!("{:?}", error.error);
        *self.error_counts.entry(error_type).or_insert(0) += 1;
        
        self.errors.push(error);
        
        // Keep only the most recent errors
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
        ErrorStatistics {
            total_errors: self.error_counts.values().sum(),
            unique_types: self.error_counts.len(),
            most_frequent: self.error_counts
                .iter()
                .max_by_key(|(_, count)| *count)
                .map(|(error_type, count)| (error_type.clone(), *count)),
        }
    }
}

/// Error statistics for monitoring
#[derive(Debug, Clone)]
pub struct ErrorStatistics {
    pub total_errors: u64,
    pub unique_types: usize,
    pub most_frequent: Option<(String, u64)>,
}