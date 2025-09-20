use super::*;
use std::collections::HashMap;

/// Configuration for a Pane
#[derive(Debug, Clone)]
pub struct PaneConfig {
    // Terminal dimensions
    pub initial_size: (u16, u16), // (width, height)
    pub min_size: (u16, u16),
    pub max_size: Option<(u16, u16)>,
    
    // Process configuration
    pub default_command: String,
    pub default_args: Vec<String>,
    pub working_directory: Option<String>,
    pub environment_variables: HashMap<String, String>,
    
    // Terminal behavior
    pub scrollback_lines: usize,
    pub auto_wrap: bool,
    pub cursor_blink: bool,
    pub cursor_style: CursorStyle,
    
    // Display settings
    pub default_title: String,
    pub title_format: String, // Template for dynamic titles
    pub show_cursor: bool,
    pub bell_action: BellAction,
    
    // Input handling
    pub alt_sends_escape: bool,
    pub application_cursor_keys: bool,
    pub application_keypad: bool,
    
    // Content management
    pub save_scrollback_to_file: bool,
    pub scrollback_file_path: Option<String>,
    pub max_memory_usage: Option<usize>, // In bytes
    
    // Performance settings
    pub render_throttle_ms: u64,
    pub max_fps: f32,
    pub lazy_rendering: bool,
    
    // Logging and debugging
    pub log_terminal_sequences: bool,
    pub debug_mode: bool,
}

impl Default for PaneConfig {
    fn default() -> Self {
        PaneConfig {
            // Default terminal size
            initial_size: (80, 24),
            min_size: (10, 3),
            max_size: None,
            
            // Default shell
            default_command: default_shell(),
            default_args: Vec::new(),
            working_directory: None,
            environment_variables: default_environment(),
            
            // Terminal behavior
            scrollback_lines: 10000,
            auto_wrap: true,
            cursor_blink: true,
            cursor_style: CursorStyle::Block,
            
            // Display
            default_title: "Terminal".to_string(),
            title_format: "{command}".to_string(),
            show_cursor: true,
            bell_action: BellAction::None,
            
            // Input
            alt_sends_escape: true,
            application_cursor_keys: false,
            application_keypad: false,
            
            // Content
            save_scrollback_to_file: false,
            scrollback_file_path: None,
            max_memory_usage: Some(256 * 1024 * 1024), // 256MB
            
            // Performance
            render_throttle_ms: 16, // ~60 FPS
            max_fps: 60.0,
            lazy_rendering: true,
            
            // Debug
            log_terminal_sequences: false,
            debug_mode: false,
        }
    }
}

impl PaneConfig {
    /// Create a new config with defaults
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the initial terminal size
    pub fn with_size(mut self, width: u16, height: u16) -> Self {
        self.initial_size = (width, height);
        self
    }
    
    /// Set the default command to run
    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.default_command = command.into();
        self
    }
    
    /// Add command arguments
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.default_args = args;
        self
    }
    
    /// Set working directory
    pub fn with_working_directory(mut self, path: impl Into<String>) -> Self {
        self.working_directory = Some(path.into());
        self
    }
    
    /// Add an environment variable
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.environment_variables.insert(key.into(), value.into());
        self
    }
    
    /// Set scrollback buffer size
    pub fn with_scrollback(mut self, lines: usize) -> Self {
        self.scrollback_lines = lines;
        self
    }
    
    /// Set title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.default_title = title.into();
        self
    }
    
    /// Enable debug mode
    pub fn with_debug(mut self, enabled: bool) -> Self {
        self.debug_mode = enabled;
        self.log_terminal_sequences = enabled;
        self
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> PaneResult<()> {
        // Check size constraints
        if self.initial_size.0 < self.min_size.0 || self.initial_size.1 < self.min_size.1 {
            return Err(PaneError::config(
                format!("Initial size {:?} is smaller than minimum size {:?}", 
                       self.initial_size, self.min_size)
            ));
        }
        
        if let Some(max_size) = self.max_size {
            if self.initial_size.0 > max_size.0 || self.initial_size.1 > max_size.1 {
                return Err(PaneError::config(
                    format!("Initial size {:?} is larger than maximum size {:?}", 
                           self.initial_size, max_size)
                ));
            }
        }
        
        // Check command is not empty
        if self.default_command.trim().is_empty() {
            return Err(PaneError::config("Default command cannot be empty"));
        }
        
        // Check scrollback size is reasonable
        if self.scrollback_lines > 1_000_000 {
            return Err(PaneError::config(
                format!("Scrollback size {} is too large (max 1,000,000)", self.scrollback_lines)
            ));
        }
        
        // Check working directory exists (if specified)
        if let Some(ref working_dir) = self.working_directory {
            if !std::path::Path::new(working_dir).exists() {
                return Err(PaneError::config(
                    format!("Working directory does not exist: {}", working_dir)
                ));
            }
        }
        
        // Check memory limit is reasonable
        if let Some(memory_limit) = self.max_memory_usage {
            if memory_limit < 1024 * 1024 { // Less than 1MB
                return Err(PaneError::config(
                    format!("Memory limit {} is too small (minimum 1MB)", memory_limit)
                ));
            }
        }
        
        // Check FPS is reasonable
        if self.max_fps <= 0.0 || self.max_fps > 1000.0 {
            return Err(PaneError::config(
                format!("Max FPS {} is not reasonable (should be 1-1000)", self.max_fps)
            ));
        }
        
        Ok(())
    }
    
    /// Get environment variables as a vector of tuples (for PTY)
    pub fn env_as_vec(&self) -> Vec<(String, String)> {
        self.environment_variables.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
    
    /// Merge another config into this one
    pub fn merge(&mut self, other: PaneConfig) {
        // Size settings
        if other.initial_size != (80, 24) { // If not default
            self.initial_size = other.initial_size;
        }
        
        // Command settings
        if other.default_command != default_shell() {
            self.default_command = other.default_command;
        }
        
        if !other.default_args.is_empty() {
            self.default_args = other.default_args;
        }
        
        if other.working_directory.is_some() {
            self.working_directory = other.working_directory;
        }
        
        // Merge environment variables
        for (key, value) in other.environment_variables {
            self.environment_variables.insert(key, value);
        }
        
        // Other settings - take non-default values
        if other.scrollback_lines != 10000 {
            self.scrollback_lines = other.scrollback_lines;
        }
        
        if other.default_title != "Terminal" {
            self.default_title = other.default_title;
        }
        
        // Always take explicit boolean settings
        self.auto_wrap = other.auto_wrap;
        self.cursor_blink = other.cursor_blink;
        self.show_cursor = other.show_cursor;
        self.alt_sends_escape = other.alt_sends_escape;
        self.application_cursor_keys = other.application_cursor_keys;
        self.application_keypad = other.application_keypad;
        self.save_scrollback_to_file = other.save_scrollback_to_file;
        self.lazy_rendering = other.lazy_rendering;
        self.log_terminal_sequences = other.log_terminal_sequences;
        self.debug_mode = other.debug_mode;
        
        // Performance settings
        self.render_throttle_ms = other.render_throttle_ms;
        self.max_fps = other.max_fps;
        
        // Optional settings
        if other.max_size.is_some() {
            self.max_size = other.max_size;
        }
        
        if other.scrollback_file_path.is_some() {
            self.scrollback_file_path = other.scrollback_file_path;
        }
        
        if other.max_memory_usage.is_some() {
            self.max_memory_usage = other.max_memory_usage;
        }
        
        // Style settings
        self.cursor_style = other.cursor_style;
        self.bell_action = other.bell_action;
    }
}

/// Bell action configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BellAction {
    None,        // Ignore bell
    Visual,      // Visual flash
    Audible,     // System beep
    Both,        // Both visual and audible
    Notification, // System notification
}

/// Get the default shell for the current platform
fn default_shell() -> String {
    #[cfg(unix)]
    {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
    }
    
    #[cfg(windows)]
    {
        std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
    }
}

/// Get default environment variables
fn default_environment() -> HashMap<String, String> {
    let mut env = HashMap::new();
    
    // Set TERM for terminal identification
    env.insert("TERM".to_string(), "xterm-256color".to_string());
    
    // Set COLORTERM for color support indication
    env.insert("COLORTERM".to_string(), "truecolor".to_string());
    
    // Copy important environment variables from parent process
    for (key, value) in std::env::vars() {
        match key.as_str() {
            "PATH" | "HOME" | "USER" | "USERNAME" | "LANG" | "LC_ALL" => {
                env.insert(key, value);
            }
            _ => {} // Ignore other variables
        }
    }
    
    env
}

/// Profile configuration for creating panes with specific settings
#[derive(Debug, Clone)]
pub struct PaneProfile {
    pub name: String,
    pub description: String,
    pub config: PaneConfig,
    pub tags: Vec<String>,
}

impl PaneProfile {
    /// Create a new profile
    pub fn new(name: impl Into<String>, config: PaneConfig) -> Self {
        PaneProfile {
            name: name.into(),
            description: String::new(),
            config,
            tags: Vec::new(),
        }
    }
    
    /// Add a description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }
    
    /// Add tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
    
    /// Add a single tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

/// Built-in pane profiles
pub struct BuiltinProfiles;

impl BuiltinProfiles {
    /// Default terminal profile
    pub fn default() -> PaneProfile {
        PaneProfile::new("default", PaneConfig::default())
            .with_description("Default terminal configuration")
    }
    
    /// Development profile with larger scrollback and debug features
    pub fn development() -> PaneProfile {
        let config = PaneConfig::default()
            .with_scrollback(50000)
            .with_debug(false) // Debug off by default
            .with_size(100, 30);
            
        PaneProfile::new("development", config)
            .with_description("Development environment with large scrollback")
            .with_tag("dev".to_string())
    }
    
    /// SSH profile for remote connections
    pub fn ssh() -> PaneProfile {
        let mut config = PaneConfig::default();
        config.title_format = "{command} - {host}".to_string();
        config.bell_action = BellAction::Notification; // Notifications for remote bells
        
        PaneProfile::new("ssh", config)
            .with_description("SSH connection profile")
            .with_tag("remote".to_string())
    }
    
    /// Monitoring profile for long-running processes
    pub fn monitoring() -> PaneProfile {
        let config = PaneConfig::default()
            .with_scrollback(100000)
            .with_size(120, 40);
            
        PaneProfile::new("monitoring", config)
            .with_description("Monitoring and logging profile with large scrollback")
            .with_tags(vec!["monitoring".to_string(), "logging".to_string()])
    }
    
    /// Get all built-in profiles
    pub fn all() -> Vec<PaneProfile> {
        vec![
            Self::default(),
            Self::development(), 
            Self::ssh(),
            Self::monitoring(),
        ]
    }
}