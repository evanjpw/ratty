use super::*;
use std::time::Duration;

/// Complete Sill layer configuration
#[derive(Debug, Clone)]
pub struct SillConfig {
    pub keyboard: KeyboardConfig,
    pub mouse: MouseConfig,
    pub clipboard: ClipboardConfig,
    pub selection: SelectionConfig,
    pub routing: RoutingConfig,
    pub performance: PerformanceConfig,
    pub debug: DebugConfig,
}

impl Default for SillConfig {
    fn default() -> Self {
        SillConfig {
            keyboard: KeyboardConfig::default(),
            mouse: MouseConfig::default(),
            clipboard: ClipboardConfig::default(),
            selection: SelectionConfig::default(),
            routing: RoutingConfig::default(),
            performance: PerformanceConfig::default(),
            debug: DebugConfig::default(),
        }
    }
}

impl SillConfig {
    /// Create a performance-optimized configuration
    pub fn performance_optimized() -> Self {
        SillConfig {
            keyboard: KeyboardConfig {
                repeat_delay: Duration::from_millis(300),
                repeat_rate: Duration::from_millis(25),
                sequence_timeout: Duration::from_millis(500),
                ..KeyboardConfig::default()
            },
            mouse: MouseConfig {
                double_click_time: Duration::from_millis(300),
                scroll_speed: 2.0,
                ..MouseConfig::default()
            },
            clipboard: ClipboardConfig {
                buffer_size: 25,
                max_text_size: 512 * 1024, // 512KB
                ..ClipboardConfig::default()
            },
            selection: SelectionConfig {
                history_size: 50,
                ..SelectionConfig::default()
            },
            routing: RoutingConfig {
                command_timeout: Duration::from_millis(50),
                ..RoutingConfig::default()
            },
            performance: PerformanceConfig {
                max_input_rate: 1000.0,
                enable_throttling: true,
                ..PerformanceConfig::default()
            },
            debug: DebugConfig {
                enable_logging: false,
                ..DebugConfig::default()
            },
        }
    }
    
    /// Create a developer/debug configuration
    pub fn debug_optimized() -> Self {
        SillConfig {
            debug: DebugConfig {
                enable_logging: true,
                log_all_events: true,
                enable_profiling: true,
                profile_detailed: true,
                ..DebugConfig::default()
            },
            performance: PerformanceConfig {
                enable_metrics: true,
                detailed_metrics: true,
                ..PerformanceConfig::default()
            },
            ..SillConfig::default()
        }
    }
    
    /// Create an accessibility-focused configuration
    pub fn accessibility_optimized() -> Self {
        SillConfig {
            keyboard: KeyboardConfig {
                repeat_delay: Duration::from_millis(750), // Longer delay
                repeat_rate: Duration::from_millis(50),  // Slower repeat
                sequence_timeout: Duration::from_millis(2000), // Longer timeout
                ..KeyboardConfig::default()
            },
            mouse: MouseConfig {
                double_click_time: Duration::from_millis(750), // Longer double-click
                double_click_distance: 4, // Larger tolerance
                scroll_speed: 0.5, // Slower scrolling
                ..MouseConfig::default()
            },
            selection: SelectionConfig {
                auto_copy: true,
                trim_whitespace: true,
                ..SelectionConfig::default()
            },
            ..SillConfig::default()
        }
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> ConfigValidation {
        let mut validation = ConfigValidation {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        };
        
        // Validate keyboard configuration
        if self.keyboard.repeat_delay.as_millis() == 0 {
            validation.errors.push("Keyboard repeat delay cannot be zero".to_string());
            validation.is_valid = false;
        }
        
        if self.keyboard.repeat_rate.as_millis() == 0 {
            validation.errors.push("Keyboard repeat rate cannot be zero".to_string());
            validation.is_valid = false;
        }
        
        // Validate mouse configuration
        if self.mouse.double_click_time.as_millis() == 0 {
            validation.errors.push("Mouse double-click time cannot be zero".to_string());
            validation.is_valid = false;
        }
        
        if self.mouse.scroll_speed <= 0.0 {
            validation.errors.push("Mouse scroll speed must be positive".to_string());
            validation.is_valid = false;
        }
        
        // Validate clipboard configuration
        if self.clipboard.buffer_size == 0 {
            validation.warnings.push("Clipboard buffer size is zero - clipboard history disabled".to_string());
        }
        
        if self.clipboard.max_text_size == 0 {
            validation.errors.push("Clipboard max text size cannot be zero".to_string());
            validation.is_valid = false;
        }
        
        // Validate selection configuration
        if self.selection.history_size == 0 {
            validation.warnings.push("Selection history size is zero - selection history disabled".to_string());
        }
        
        // Validate performance configuration
        if self.performance.max_input_rate <= 0.0 {
            validation.errors.push("Performance max input rate must be positive".to_string());
            validation.is_valid = false;
        }
        
        // Cross-component validation
        if self.keyboard.sequence_timeout < self.keyboard.repeat_rate * 2 {
            validation.warnings.push("Keyboard sequence timeout may be too short for repeat rate".to_string());
        }
        
        validation
    }
    
    /// Load configuration from file (placeholder)
    pub fn load_from_file(_path: &str) -> SillResult<Self> {
        // This would load from a TOML/JSON file
        // For now, return default configuration
        Ok(SillConfig::default())
    }
    
    /// Save configuration to file (placeholder)
    pub fn save_to_file(&self, _path: &str) -> SillResult<()> {
        // This would save to a TOML/JSON file
        // For now, just return success
        Ok(())
    }
    
    /// Merge with another configuration (other takes precedence)
    pub fn merge_with(&mut self, other: SillConfig) {
        // This would intelligently merge configurations
        // For now, just replace
        *self = other;
    }
}

/// Performance configuration
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    pub max_input_rate: f64,       // Events per second
    pub enable_throttling: bool,
    pub throttle_threshold: f64,   // Input rate that triggers throttling
    pub enable_metrics: bool,
    pub detailed_metrics: bool,
    pub metrics_interval: Duration,
    pub max_queue_size: usize,
    pub queue_warning_threshold: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        PerformanceConfig {
            max_input_rate: 2000.0,
            enable_throttling: false,
            throttle_threshold: 1500.0,
            enable_metrics: false,
            detailed_metrics: false,
            metrics_interval: Duration::from_secs(1),
            max_queue_size: 1000,
            queue_warning_threshold: 100,
        }
    }
}

/// Debug configuration
#[derive(Debug, Clone)]
pub struct DebugConfig {
    pub enable_logging: bool,
    pub log_level: LogLevel,
    pub log_all_events: bool,
    pub log_performance: bool,
    pub enable_profiling: bool,
    pub profile_detailed: bool,
    pub profile_interval: Duration,
    pub dump_events_on_error: bool,
    pub max_debug_history: usize,
}

impl Default for DebugConfig {
    fn default() -> Self {
        DebugConfig {
            enable_logging: false,
            log_level: LogLevel::Info,
            log_all_events: false,
            log_performance: false,
            enable_profiling: false,
            profile_detailed: false,
            profile_interval: Duration::from_secs(5),
            dump_events_on_error: true,
            max_debug_history: 1000,
        }
    }
}

/// Logging levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Configuration validation result
#[derive(Debug, Clone)]
pub struct ConfigValidation {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ConfigValidation {
    /// Check if configuration is valid
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }
    
    /// Get all validation messages
    pub fn get_messages(&self) -> Vec<String> {
        let mut messages = Vec::new();
        
        for error in &self.errors {
            messages.push(format!("ERROR: {}", error));
        }
        
        for warning in &self.warnings {
            messages.push(format!("WARNING: {}", warning));
        }
        
        messages
    }
    
    /// Get error count
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }
    
    /// Get warning count
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }
}

/// Configuration builder for fluent configuration creation
#[derive(Debug)]
pub struct SillConfigBuilder {
    config: SillConfig,
}

impl SillConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        SillConfigBuilder {
            config: SillConfig::default(),
        }
    }
    
    /// Set keyboard configuration
    pub fn keyboard(mut self, config: KeyboardConfig) -> Self {
        self.config.keyboard = config;
        self
    }
    
    /// Set mouse configuration
    pub fn mouse(mut self, config: MouseConfig) -> Self {
        self.config.mouse = config;
        self
    }
    
    /// Set clipboard configuration
    pub fn clipboard(mut self, config: ClipboardConfig) -> Self {
        self.config.clipboard = config;
        self
    }
    
    /// Set selection configuration
    pub fn selection(mut self, config: SelectionConfig) -> Self {
        self.config.selection = config;
        self
    }
    
    /// Set routing configuration
    pub fn routing(mut self, config: RoutingConfig) -> Self {
        self.config.routing = config;
        self
    }
    
    /// Set performance configuration
    pub fn performance(mut self, config: PerformanceConfig) -> Self {
        self.config.performance = config;
        self
    }
    
    /// Set debug configuration
    pub fn debug(mut self, config: DebugConfig) -> Self {
        self.config.debug = config;
        self
    }
    
    /// Enable performance optimization
    pub fn enable_performance_optimization(mut self) -> Self {
        self.config = SillConfig::performance_optimized();
        self
    }
    
    /// Enable debug mode
    pub fn enable_debug_mode(mut self) -> Self {
        self.config.debug = DebugConfig {
            enable_logging: true,
            log_level: LogLevel::Debug,
            log_all_events: true,
            enable_profiling: true,
            ..DebugConfig::default()
        };
        self
    }
    
    /// Enable accessibility features
    pub fn enable_accessibility(mut self) -> Self {
        self.config = SillConfig::accessibility_optimized();
        self
    }
    
    /// Build the configuration
    pub fn build(self) -> SillResult<SillConfig> {
        let validation = self.config.validate();
        
        if validation.is_valid() {
            Ok(self.config)
        } else {
            Err(SillError::configuration(&format!(
                "Invalid configuration: {}",
                validation.get_messages().join(", ")
            )))
        }
    }
    
    /// Build without validation
    pub fn build_unchecked(self) -> SillConfig {
        self.config
    }
}

impl Default for SillConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration presets for common use cases
pub struct ConfigPresets;

impl ConfigPresets {
    /// Default configuration for general use
    pub fn default() -> SillConfig {
        SillConfig::default()
    }
    
    /// Configuration optimized for gaming/low-latency applications
    pub fn gaming() -> SillConfig {
        SillConfig {
            keyboard: KeyboardConfig {
                repeat_delay: Duration::from_millis(200),
                repeat_rate: Duration::from_millis(16), // ~60 Hz
                ..KeyboardConfig::default()
            },
            mouse: MouseConfig {
                double_click_time: Duration::from_millis(200),
                scroll_speed: 3.0,
                ..MouseConfig::default()
            },
            performance: PerformanceConfig {
                max_input_rate: 5000.0,
                enable_throttling: false,
                enable_metrics: true,
                ..PerformanceConfig::default()
            },
            ..SillConfig::default()
        }
    }
    
    /// Configuration for terminal servers/remote access
    pub fn server() -> SillConfig {
        SillConfig {
            performance: PerformanceConfig {
                max_input_rate: 500.0,
                enable_throttling: true,
                throttle_threshold: 400.0,
                ..PerformanceConfig::default()
            },
            clipboard: ClipboardConfig {
                use_system_clipboard: false, // May not be available on servers
                buffer_size: 10,
                ..ClipboardConfig::default()
            },
            debug: DebugConfig {
                enable_logging: true,
                log_level: LogLevel::Warn,
                ..DebugConfig::default()
            },
            ..SillConfig::default()
        }
    }
    
    /// Configuration for embedded/resource-constrained systems
    pub fn embedded() -> SillConfig {
        SillConfig {
            clipboard: ClipboardConfig {
                buffer_size: 5,
                max_text_size: 64 * 1024, // 64KB
                ..ClipboardConfig::default()
            },
            selection: SelectionConfig {
                history_size: 10,
                ..SelectionConfig::default()
            },
            performance: PerformanceConfig {
                max_input_rate: 200.0,
                enable_throttling: true,
                max_queue_size: 100,
                ..PerformanceConfig::default()
            },
            debug: DebugConfig {
                enable_logging: false,
                enable_profiling: false,
                max_debug_history: 100,
                ..DebugConfig::default()
            },
            ..SillConfig::default()
        }
    }
}