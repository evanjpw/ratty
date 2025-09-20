use super::*;
use std::time::Duration;

/// Main configuration for the Glazing layer
#[derive(Debug, Clone)]
pub struct GlazingConfig {
    /// Rendering configuration
    pub renderer: RendererConfig,
    
    /// Theme configuration
    pub theme: ThemeConfig,
    
    /// Cursor configuration
    pub cursor: CursorConfig,
    
    /// Layout configuration
    pub layout: LayoutConfig,
    
    /// Performance configuration
    pub performance: PerformanceConfig,
    
    /// Feature flags
    pub features: FeatureConfig,
}

impl Default for GlazingConfig {
    fn default() -> Self {
        GlazingConfig {
            renderer: RendererConfig::default(),
            theme: ThemeConfig::default(),
            cursor: CursorConfig::default(),
            layout: LayoutConfig::default(),
            performance: PerformanceConfig::default(),
            features: FeatureConfig::default(),
        }
    }
}

/// Rendering configuration
#[derive(Debug, Clone)]
pub struct RendererConfig {
    /// Target frames per second
    pub target_fps: u32,
    
    /// Enable V-sync
    pub vsync: bool,
    
    /// Enable hardware acceleration
    pub hardware_acceleration: bool,
    
    /// Font configuration
    pub font: FontConfig,
    
    /// Anti-aliasing settings
    pub antialiasing: AntiAliasingConfig,
    
    /// Text rendering quality
    pub text_quality: TextQuality,
}

impl Default for RendererConfig {
    fn default() -> Self {
        RendererConfig {
            target_fps: 60,
            vsync: true,
            hardware_acceleration: true,
            font: FontConfig::default(),
            antialiasing: AntiAliasingConfig::default(),
            text_quality: TextQuality::High,
        }
    }
}

/// Font configuration
#[derive(Debug, Clone)]
pub struct FontConfig {
    /// Primary font family
    pub family: String,
    
    /// Font size in points
    pub size: f32,
    
    /// Line height multiplier
    pub line_height: f32,
    
    /// Font weight
    pub weight: FontWeight,
    
    /// Font style
    pub style: FontStyle,
    
    /// Fallback fonts
    pub fallbacks: Vec<String>,
    
    /// Enable font ligatures
    pub ligatures: bool,
    
    /// Subpixel rendering
    pub subpixel_rendering: bool,
}

impl Default for FontConfig {
    fn default() -> Self {
        FontConfig {
            family: "monospace".to_string(),
            size: 12.0,
            line_height: 1.2,
            weight: FontWeight::Normal,
            style: FontStyle::Normal,
            fallbacks: vec![
                "Courier New".to_string(),
                "Monaco".to_string(),
                "Consolas".to_string(),
            ],
            ligatures: false,
            subpixel_rendering: true,
        }
    }
}

/// Font weight
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
}

/// Font style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

/// Anti-aliasing configuration
#[derive(Debug, Clone)]
pub struct AntiAliasingConfig {
    /// Enable anti-aliasing
    pub enabled: bool,
    
    /// Anti-aliasing method
    pub method: AntiAliasingMethod,
    
    /// Subpixel ordering
    pub subpixel_order: SubpixelOrder,
}

impl Default for AntiAliasingConfig {
    fn default() -> Self {
        AntiAliasingConfig {
            enabled: true,
            method: AntiAliasingMethod::Subpixel,
            subpixel_order: SubpixelOrder::Rgb,
        }
    }
}

/// Anti-aliasing method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AntiAliasingMethod {
    None,
    Grayscale,
    Subpixel,
}

/// Subpixel ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubpixelOrder {
    Rgb,
    Bgr,
    Vrgb,
    Vbgr,
}

/// Text rendering quality
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextQuality {
    Low,
    Medium,
    High,
    Ultra,
}

/// Cursor configuration
#[derive(Debug, Clone)]
pub struct CursorConfig {
    /// Enable cursor blinking
    pub enable_blinking: bool,
    
    /// Blink rate
    pub blink_rate: Duration,
    
    /// Cursor thickness (for bar and underline styles)
    pub thickness: u16,
    
    /// Cursor color override (None uses theme color)
    pub color_override: Option<crate::sash::Color>,
    
    /// Smooth cursor movement
    pub smooth_movement: bool,
    
    /// Movement animation duration
    pub movement_duration: Duration,
}

impl Default for CursorConfig {
    fn default() -> Self {
        CursorConfig {
            enable_blinking: true,
            blink_rate: Duration::from_millis(500),
            thickness: 1,
            color_override: None,
            smooth_movement: false,
            movement_duration: Duration::from_millis(100),
        }
    }
}

/// Performance configuration
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Enable dirty region tracking
    pub dirty_tracking: bool,
    
    /// Enable render batching
    pub render_batching: bool,
    
    /// Maximum render time per frame (in milliseconds)
    pub max_frame_time: u32,
    
    /// Enable frame skipping when behind
    pub frame_skipping: bool,
    
    /// Memory usage limit (in MB)
    pub memory_limit: usize,
    
    /// Enable GPU acceleration
    pub gpu_acceleration: bool,
    
    /// Render cache size limit
    pub cache_size_limit: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        PerformanceConfig {
            dirty_tracking: true,
            render_batching: true,
            max_frame_time: 16, // ~60 FPS
            frame_skipping: true,
            memory_limit: 512, // 512 MB
            gpu_acceleration: true,
            cache_size_limit: 1024 * 1024 * 100, // 100 MB
        }
    }
}

/// Feature configuration flags
#[derive(Debug, Clone)]
pub struct FeatureConfig {
    /// Enable smooth scrolling
    pub smooth_scrolling: bool,
    
    /// Enable layout transitions
    pub layout_transitions: bool,
    
    /// Enable advanced text shaping
    pub text_shaping: bool,
    
    /// Enable decorations (borders, scrollbars)
    pub decorations: bool,
    
    /// Enable transparency effects
    pub transparency: bool,
    
    /// Enable blur effects
    pub blur_effects: bool,
    
    /// Enable animations
    pub animations: bool,
    
    /// Enable high DPI support
    pub high_dpi: bool,
}

impl Default for FeatureConfig {
    fn default() -> Self {
        FeatureConfig {
            smooth_scrolling: true,
            layout_transitions: false, // Disabled by default for performance
            text_shaping: false, // Disabled by default for compatibility
            decorations: true,
            transparency: false, // Disabled by default for performance
            blur_effects: false, // Disabled by default for performance
            animations: true,
            high_dpi: true,
        }
    }
}

/// Configuration validation result
#[derive(Debug)]
pub struct ConfigValidation {
    pub is_valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl GlazingConfig {
    /// Validate the configuration
    pub fn validate(&self) -> ConfigValidation {
        let mut validation = ConfigValidation {
            is_valid: true,
            warnings: Vec::new(),
            errors: Vec::new(),
        };
        
        // Validate renderer config
        if self.renderer.target_fps == 0 {
            validation.errors.push("Target FPS must be greater than 0".to_string());
            validation.is_valid = false;
        }
        
        if self.renderer.target_fps > 144 {
            validation.warnings.push("Target FPS above 144 may impact performance".to_string());
        }
        
        // Validate font config
        if self.renderer.font.size <= 0.0 {
            validation.errors.push("Font size must be greater than 0".to_string());
            validation.is_valid = false;
        }
        
        if self.renderer.font.size < 6.0 {
            validation.warnings.push("Font size below 6pt may be hard to read".to_string());
        }
        
        if self.renderer.font.line_height < 0.8 {
            validation.warnings.push("Line height below 0.8 may cause overlapping text".to_string());
        }
        
        // Validate cursor config
        if self.cursor.blink_rate.as_millis() < 100 {
            validation.warnings.push("Very fast cursor blink rate may be distracting".to_string());
        }
        
        // Validate performance config
        if self.performance.max_frame_time == 0 {
            validation.errors.push("Max frame time must be greater than 0".to_string());
            validation.is_valid = false;
        }
        
        if self.performance.memory_limit < 64 {
            validation.warnings.push("Memory limit below 64MB may cause performance issues".to_string());
        }
        
        // Validate layout config
        if self.layout.min_pane_width == 0 || self.layout.min_pane_height == 0 {
            validation.errors.push("Minimum pane dimensions must be greater than 0".to_string());
            validation.is_valid = false;
        }
        
        validation
    }
    
    /// Apply performance optimizations based on system capabilities
    pub fn optimize_for_system(&mut self) {
        // Detect system capabilities (placeholder implementation)
        let estimated_memory = self.estimate_system_memory();
        let estimated_gpu_capability = self.estimate_gpu_capability();
        
        // Adjust settings based on capabilities
        if estimated_memory < 512 {
            self.performance.memory_limit = estimated_memory / 2;
            self.performance.cache_size_limit = 1024 * 1024 * 50; // 50 MB
            self.features.animations = false;
            self.features.smooth_scrolling = false;
        }
        
        if !estimated_gpu_capability {
            self.renderer.hardware_acceleration = false;
            self.performance.gpu_acceleration = false;
            self.features.blur_effects = false;
            self.features.transparency = false;
        }
        
        // Adjust FPS based on system
        if estimated_memory < 256 {
            self.renderer.target_fps = 30;
        }
    }
    
    /// Create a configuration optimized for performance
    pub fn performance_optimized() -> Self {
        let mut config = Self::default();
        
        config.renderer.target_fps = 30;
        config.renderer.hardware_acceleration = false;
        config.performance.dirty_tracking = true;
        config.performance.render_batching = true;
        config.performance.frame_skipping = true;
        config.features.smooth_scrolling = false;
        config.features.layout_transitions = false;
        config.features.animations = false;
        config.features.transparency = false;
        config.features.blur_effects = false;
        
        config
    }
    
    /// Create a configuration optimized for quality
    pub fn quality_optimized() -> Self {
        let mut config = Self::default();
        
        config.renderer.target_fps = 60;
        config.renderer.hardware_acceleration = true;
        config.renderer.text_quality = TextQuality::Ultra;
        config.renderer.antialiasing.enabled = true;
        config.renderer.antialiasing.method = AntiAliasingMethod::Subpixel;
        config.features.smooth_scrolling = true;
        config.features.layout_transitions = true;
        config.features.animations = true;
        config.features.text_shaping = true;
        
        config
    }
    
    // Helper methods for system detection (placeholder implementations)
    fn estimate_system_memory(&self) -> usize {
        // TODO: Implement actual memory detection
        1024 // Assume 1GB
    }
    
    fn estimate_gpu_capability(&self) -> bool {
        // TODO: Implement actual GPU capability detection
        true // Assume GPU available
    }
}