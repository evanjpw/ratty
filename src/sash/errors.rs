use thiserror::Error;
use super::PaneId;

/// Main error type for the Sash layer
#[derive(Debug, Error)]
pub enum SashError {
    #[error("Pane not found: {0:?}")]
    PaneNotFound(PaneId),
    
    #[error("Tab not found at index: {0}")]
    TabNotFound(usize),
    
    #[error("Invalid layout: {0}")]
    InvalidLayout(String),
    
    #[error("Split operation failed: {0}")]
    SplitFailed(String),
    
    #[error("Theme error: {0}")]
    ThemeError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Layout constraint violation: {0}")]
    LayoutConstraintViolation(String),
    
    #[error("Maximum panes exceeded: limit is {0}")]
    MaxPanesExceeded(usize),
    
    #[error("Maximum tabs exceeded: limit is {0}")]
    MaxTabsExceeded(usize),
    
    #[error("Pane creation failed: {0}")]
    PaneCreationFailed(String),
    
    #[error("Event handling error: {0}")]
    EventError(String),
    
    #[error("No active pane")]
    NoActivePane,
    
    #[error("No active tab")]
    NoActiveTab,
    
    #[error("Invalid split ratio: {0}")]
    InvalidSplitRatio(f32),
    
    #[error("Cannot close last pane")]
    CannotCloseLastPane,
    
    #[error("Cannot close last tab")]
    CannotCloseLastTab,
    
    #[error("Layout not found: {0}")]
    LayoutNotFound(String),
    
    #[error("Invalid pane configuration: {0}")]
    InvalidPaneConfig(String),
    
    #[error("Focus operation failed: {0}")]
    FocusFailed(String),
    
    #[error("Tab operation failed: {0}")]
    TabOperationFailed(String),
    
    #[error("Theme not found: {0}")]
    ThemeNotFound(String),
    
    #[error("Invalid color specification: {0}")]
    InvalidColor(String),
    
    #[error("State validation failed: {0}")]
    StateValidationFailed(String),
}

/// Result type alias for Sash operations
pub type SashResult<T> = Result<T, SashError>;