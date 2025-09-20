use super::*;
use crate::pane::{ScreenBuffer, ScrollbackBuffer, Cursor};
use crate::sash::{PaneId, Theme};
use ratatui::{backend::Backend, layout::Rect, Frame as RatatuiFrame};

/// Primary interface for Glazing operations
pub trait GlazingInterface: Send + Sync {
    // ========== Core Rendering ==========
    
    /// Render a single pane to the given area
    fn render_pane<B: Backend>(
        &mut self,
        frame: &mut RatatuiFrame,
        area: Rect,
        screen_buffer: &ScreenBuffer,
        scrollback: &ScrollbackBuffer,
        cursor: &Cursor,
        pane_id: PaneId,
        is_active: bool,
    ) -> GlazingResult<()>;
    
    /// Render multiple panes in a layout
    fn render_layout<B: Backend>(
        &mut self,
        frame: &mut RatatuiFrame,
        area: Rect,
        panes: &[(PaneId, &ScreenBuffer, &ScrollbackBuffer, &Cursor, bool)],
        layout: &crate::sash::Layout,
    ) -> GlazingResult<()>;
    
    /// Force a complete redraw of all content
    fn force_redraw(&mut self) -> GlazingResult<()>;
    
    // ========== Viewport Management ==========
    
    /// Scroll the viewport
    fn scroll(&mut self, direction: ScrollDirection, amount: usize) -> GlazingResult<()>;
    
    /// Scroll to a specific line
    fn scroll_to_line(&mut self, line: usize) -> GlazingResult<()>;
    
    /// Get current viewport state
    fn get_viewport_state(&self) -> ViewportState;
    
    /// Check if viewport is at bottom (showing current content)
    fn is_at_bottom(&self) -> bool;
    
    /// Check if viewport is at top (showing oldest scrollback)
    fn is_at_top(&self) -> bool;
    
    // ========== Theme and Styling ==========
    
    /// Apply a theme to the rendering engine
    fn apply_theme(&mut self, theme: &Theme) -> GlazingResult<()>;
    
    /// Get current theme name
    fn get_current_theme(&self) -> Option<String>;
    
    /// List available themes
    fn list_themes(&self) -> Vec<String>;
    
    /// Set font configuration
    fn set_font(&mut self, config: FontConfig) -> GlazingResult<()>;
    
    /// Get current font configuration
    fn get_font_config(&self) -> &FontConfig;
    
    // ========== Configuration ==========
    
    /// Update rendering configuration
    fn update_config(&mut self, config: GlazingConfig) -> GlazingResult<()>;
    
    /// Get current configuration
    fn get_config(&self) -> &GlazingConfig;
    
    /// Validate configuration
    fn validate_config(&self, config: &GlazingConfig) -> super::config::ConfigValidation;
    
    // ========== Performance ==========
    
    /// Get performance metrics
    fn get_performance_metrics(&self) -> &PerformanceTracker;
    
    /// Get rendering statistics
    fn get_render_stats(&self) -> RenderStatistics;
    
    /// Enable or disable performance monitoring
    fn set_performance_monitoring(&mut self, enabled: bool);
    
    /// Clear performance statistics
    fn clear_performance_stats(&mut self);
    
    // ========== Event Handling ==========
    
    /// Register an event listener
    fn register_event_listener(&mut self, event_type: GlazingEventType, listener: Box<dyn GlazingEventListener>);
    
    /// Emit an event
    fn emit_event(&mut self, event: GlazingEvent) -> GlazingResult<()>;
    
    /// Check if there are listeners for an event type
    fn has_event_listeners(&self, event_type: GlazingEventType) -> bool;
    
    // ========== Debug and Diagnostics ==========
    
    /// Enable or disable debug mode
    fn set_debug_mode(&mut self, enabled: bool);
    
    /// Get debug information
    fn get_debug_info(&self) -> DebugInfo;
    
    /// Dump internal state for debugging
    fn dump_state(&self) -> String;
    
    /// Validate internal state
    fn validate_state(&self) -> GlazingResult<()>;
}

/// Implementation of GlazingInterface for GlazingEngine
impl GlazingInterface for GlazingEngine {
    // ========== Core Rendering ==========
    
    fn render_pane<B: Backend>(
        &mut self,
        frame: &mut RatatuiFrame,
        area: Rect,
        screen_buffer: &ScreenBuffer,
        scrollback: &ScrollbackBuffer,
        cursor: &Cursor,
        pane_id: PaneId,
        is_active: bool,
    ) -> GlazingResult<()> {
        self.render_pane::<B>(frame, area, screen_buffer, scrollback, cursor, pane_id, is_active)
    }
    
    fn render_layout<B: Backend>(
        &mut self,
        frame: &mut RatatuiFrame,
        area: Rect,
        panes: &[(PaneId, &ScreenBuffer, &ScrollbackBuffer, &Cursor, bool)],
        layout: &crate::sash::Layout,
    ) -> GlazingResult<()> {
        self.render_layout::<B>(frame, area, panes, layout)
    }
    
    fn force_redraw(&mut self) -> GlazingResult<()> {
        if let Some(ref mut frame) = self.current_frame {
            frame.mark_all_dirty();
        }
        Ok(())
    }
    
    // ========== Viewport Management ==========
    
    fn scroll(&mut self, direction: ScrollDirection, amount: usize) -> GlazingResult<()> {
        self.scroll(direction, amount)
    }
    
    fn scroll_to_line(&mut self, line: usize) -> GlazingResult<()> {
        self.viewport.scroll_to_line(line)
    }
    
    fn get_viewport_state(&self) -> ViewportState {
        ViewportState::from(&self.viewport)
    }
    
    fn is_at_bottom(&self) -> bool {
        self.viewport.is_at_bottom()
    }
    
    fn is_at_top(&self) -> bool {
        self.viewport.is_at_top()
    }
    
    // ========== Theme and Styling ==========
    
    fn apply_theme(&mut self, theme: &Theme) -> GlazingResult<()> {
        self.apply_theme(theme)
    }
    
    fn get_current_theme(&self) -> Option<String> {
        Some(self.theme_engine.current_theme.name.clone())
    }
    
    fn list_themes(&self) -> Vec<String> {
        let collection = super::theme::ThemeCollection::new();
        collection.list_themes()
    }
    
    fn set_font(&mut self, config: FontConfig) -> GlazingResult<()> {
        self.config.renderer.font = config;
        self.renderer.update_config(&self.config)?;
        Ok(())
    }
    
    fn get_font_config(&self) -> &FontConfig {
        &self.config.renderer.font
    }
    
    // ========== Configuration ==========
    
    fn update_config(&mut self, config: GlazingConfig) -> GlazingResult<()> {
        self.update_config(config)
    }
    
    fn get_config(&self) -> &GlazingConfig {
        &self.config
    }
    
    fn validate_config(&self, config: &GlazingConfig) -> super::config::ConfigValidation {
        config.validate()
    }
    
    // ========== Performance ==========
    
    fn get_performance_metrics(&self) -> &PerformanceTracker {
        self.get_performance_metrics()
    }
    
    fn get_render_stats(&self) -> RenderStatistics {
        RenderStatistics {
            frames_rendered: self.performance.frame_count(),
            average_frame_time: self.performance.average_frame_time(),
            current_fps: self.performance.fps(),
            memory_usage: self.performance.current_memory_usage,
            peak_memory_usage: self.performance.peak_memory_usage,
            cache_hit_rate: 0.85, // TODO: Implement actual cache metrics
            gpu_acceleration_enabled: self.config.renderer.hardware_acceleration,
        }
    }
    
    fn set_performance_monitoring(&mut self, _enabled: bool) {
        // TODO: Implement performance monitoring toggle
    }
    
    fn clear_performance_stats(&mut self) {
        self.performance = PerformanceTracker::new();
    }
    
    // ========== Event Handling ==========
    
    fn register_event_listener(&mut self, event_type: GlazingEventType, listener: Box<dyn GlazingEventListener>) {
        self.event_handler.register_listener(event_type, listener);
    }
    
    fn emit_event(&mut self, event: GlazingEvent) -> GlazingResult<()> {
        self.event_handler.dispatch(event)
    }
    
    fn has_event_listeners(&self, event_type: GlazingEventType) -> bool {
        self.event_handler.has_listeners(event_type)
    }
    
    // ========== Debug and Diagnostics ==========
    
    fn set_debug_mode(&mut self, enabled: bool) {
        let _ = self.emit_event(GlazingEvent::DebugModeToggled { enabled });
    }
    
    fn get_debug_info(&self) -> DebugInfo {
        DebugInfo {
            engine_state: "running".to_string(),
            renderer_info: self.renderer.get_debug_info(),
            theme_info: self.theme_engine.get_debug_info(),
            viewport_info: (&self.viewport.clone()).into(),
            performance_info: self.performance.clone(),
            config_info: format!("{:?}", self.config),
            cache_stats: self.layout_manager.cache_stats(),
            event_stats: self.event_handler.get_stats().clone(),
        }
    }
    
    fn dump_state(&self) -> String {
        format!(
            "GlazingEngine State:\n\
             - Current FPS: {:.1}\n\
             - Frame Count: {}\n\
             - Viewport Offset: {}\n\
             - Theme: {}\n\
             - Memory Usage: {} bytes\n\
             - Cache Entries: {}\n\
             - Event Listeners: {}",
            self.performance.fps(),
            self.performance.frame_count(),
            self.viewport.scroll_offset,
            self.theme_engine.current_theme.name,
            self.performance.current_memory_usage,
            self.layout_manager.cache_stats().entries,
            self.event_handler.get_stats().total_listeners
        )
    }
    
    fn validate_state(&self) -> GlazingResult<()> {
        // Validate viewport bounds
        if self.viewport.scroll_offset > self.viewport.total_lines {
            return Err(GlazingError::invalid_state(
                format!("Scroll offset {} exceeds total lines {}", 
                       self.viewport.scroll_offset, self.viewport.total_lines)
            ));
        }
        
        // Validate configuration
        let validation = self.config.validate();
        if !validation.is_valid {
            return Err(GlazingError::config(
                format!("Invalid configuration: {:?}", validation.errors)
            ));
        }
        
        Ok(())
    }
}

/// Rendering statistics
#[derive(Debug, Clone)]
pub struct RenderStatistics {
    pub frames_rendered: u64,
    pub average_frame_time: std::time::Duration,
    pub current_fps: f64,
    pub memory_usage: usize,
    pub peak_memory_usage: usize,
    pub cache_hit_rate: f64,
    pub gpu_acceleration_enabled: bool,
}

/// Debug information structure
#[derive(Debug, Clone)]
pub struct DebugInfo {
    pub engine_state: String,
    pub renderer_info: RendererDebugInfo,
    pub theme_info: ThemeDebugInfo,
    pub viewport_info: ViewportState,
    pub performance_info: PerformanceTracker,
    pub config_info: String,
    pub cache_stats: LayoutCacheStats,
    pub event_stats: EventHandlerStats,
}

/// Renderer debug information
#[derive(Debug, Clone)]
pub struct RendererDebugInfo {
    pub backend_type: String,
    pub hardware_acceleration: bool,
    pub text_quality: String,
    pub font_family: String,
    pub font_size: f32,
}

impl super::renderer::TerminalRenderer {
    pub fn get_debug_info(&self) -> RendererDebugInfo {
        RendererDebugInfo {
            backend_type: "ratatui".to_string(),
            hardware_acceleration: self.config.hardware_acceleration,
            text_quality: format!("{:?}", self.config.text_quality),
            font_family: self.config.font.family.clone(),
            font_size: self.config.font.size,
        }
    }
}

/// Theme debug information
#[derive(Debug, Clone)]
pub struct ThemeDebugInfo {
    pub current_theme: String,
    pub available_themes: Vec<String>,
    pub base_colors: String,
}

impl super::theme::ThemeEngine {
    pub fn get_debug_info(&self) -> ThemeDebugInfo {
        let collection = super::theme::ThemeCollection::new();
        ThemeDebugInfo {
            current_theme: self.current_theme.name.clone(),
            available_themes: collection.list_themes(),
            base_colors: format!("fg: {:?}, bg: {:?}", 
                               self.current_theme.colors.foreground,
                               self.current_theme.colors.background),
        }
    }
}

// /// Trait for objects that provide glazing services
// pub trait GlazingProvider {
//     /// Get the glazing interface
//     fn glazing(&mut self) -> &mut dyn GlazingInterface;
//
//     /// Check if glazing is available
//     fn has_glazing(&self) -> bool;
//
//     /// Initialize glazing with configuration
//     fn init_glazing(&mut self, config: GlazingConfig) -> GlazingResult<()>;
//
//     /// Shutdown glazing
//     fn shutdown_glazing(&mut self) -> GlazingResult<()>;
// }

/// Mock implementation for testing
#[cfg(test)]
pub struct MockGlazingInterface {
    pub render_calls: std::cell::RefCell<Vec<String>>,
    pub theme_name: std::cell::RefCell<Option<String>>,
    pub viewport_state: std::cell::RefCell<ViewportState>,
    pub performance_metrics: PerformanceTracker,
}

#[cfg(test)]
impl MockGlazingInterface {
    pub fn new() -> Self {
        MockGlazingInterface {
            render_calls: std::cell::RefCell::new(Vec::new()),
            theme_name: std::cell::RefCell::new(None),
            viewport_state: std::cell::RefCell::new(ViewportState {
                scroll_offset: 0,
                visible_lines: 24,
                total_lines: 100,
                horizontal_offset: 0,
                visible_columns: 80,
                total_columns: 80,
                is_at_bottom: true,
                is_at_top: false,
                scroll_percentage: 0.0,
            }),
            performance_metrics: PerformanceTracker::new(),
        }
    }
}

#[cfg(test)]
impl GlazingInterface for MockGlazingInterface {
    fn render_pane<B: Backend>(
        &mut self,
        _frame: &mut RatatuiFrame,
        _area: Rect,
        _screen_buffer: &ScreenBuffer,
        _scrollback: &ScrollbackBuffer,
        _cursor: &Cursor,
        pane_id: PaneId,
        _is_active: bool,
    ) -> GlazingResult<()> {
        self.render_calls.borrow_mut().push(format!("render_pane:{:?}", pane_id));
        Ok(())
    }
    
    fn render_layout<B: Backend>(
        &mut self,
        _frame: &mut RatatuiFrame,
        _area: Rect,
        panes: &[(PaneId, &ScreenBuffer, &ScrollbackBuffer, &Cursor, bool)],
        _layout: &crate::sash::Layout,
    ) -> GlazingResult<()> {
        self.render_calls.borrow_mut().push(format!("render_layout:{}", panes.len()));
        Ok(())
    }
    
    fn force_redraw(&mut self) -> GlazingResult<()> {
        self.render_calls.borrow_mut().push("force_redraw".to_string());
        Ok(())
    }
    
    fn scroll(&mut self, direction: ScrollDirection, amount: usize) -> GlazingResult<()> {
        self.render_calls.borrow_mut().push(format!("scroll:{:?}:{}", direction, amount));
        Ok(())
    }
    
    fn scroll_to_line(&mut self, line: usize) -> GlazingResult<()> {
        self.render_calls.borrow_mut().push(format!("scroll_to_line:{}", line));
        Ok(())
    }
    
    fn get_viewport_state(&self) -> ViewportState {
        self.viewport_state.borrow().clone()
    }
    
    fn is_at_bottom(&self) -> bool {
        self.viewport_state.borrow().is_at_bottom
    }
    
    fn is_at_top(&self) -> bool {
        self.viewport_state.borrow().is_at_top
    }
    
    fn apply_theme(&mut self, theme: &Theme) -> GlazingResult<()> {
        *self.theme_name.borrow_mut() = Some(theme.name.clone());
        Ok(())
    }
    
    fn get_current_theme(&self) -> Option<String> {
        self.theme_name.borrow().clone()
    }
    
    fn list_themes(&self) -> Vec<String> {
        vec!["default".to_string(), "dark".to_string(), "light".to_string()]
    }
    
    fn set_font(&mut self, _config: FontConfig) -> GlazingResult<()> {
        Ok(())
    }
    
    fn get_font_config(&self) -> &FontConfig {
        // This is a bit awkward for the mock, but we'll return a static reference
        static DEFAULT_FONT: FontConfig = FontConfig {
            family: String::new(),
            size: 12.0,
            line_height: 1.2,
            weight: FontWeight::Normal,
            style: FontStyle::Normal,
            fallbacks: Vec::new(),
            ligatures: false,
            subpixel_rendering: true,
        };
        &DEFAULT_FONT
    }
    
    fn update_config(&mut self, _config: GlazingConfig) -> GlazingResult<()> {
        Ok(())
    }
    
    fn get_config(&self) -> &GlazingConfig {
        static DEFAULT_CONFIG: GlazingConfig = GlazingConfig {
            renderer: RendererConfig {
                target_fps: 60,
                vsync: true,
                hardware_acceleration: true,
                font: FontConfig {
                    family: String::new(),
                    size: 12.0,
                    line_height: 1.2,
                    weight: FontWeight::Normal,
                    style: FontStyle::Normal,
                    fallbacks: Vec::new(),
                    ligatures: false,
                    subpixel_rendering: true,
                },
                antialiasing: AntiAliasingConfig {
                    enabled: true,
                    method: AntiAliasingMethod::Subpixel,
                    subpixel_order: SubpixelOrder::Rgb,
                },
                text_quality: TextQuality::High,
            },
            theme: ThemeConfig {
                default_theme_name: String::new(),
                font_family: String::new(),
                font_size: 12.0,
                line_height: 1.2,
                cursor_blink_rate: std::time::Duration::from_millis(500),
                smooth_transitions: true,
                high_contrast_mode: false,
            },
            cursor: CursorConfig {
                enable_blinking: true,
                blink_rate: std::time::Duration::from_millis(500),
                thickness: 1,
                color_override: None,
                smooth_movement: false,
                movement_duration: std::time::Duration::from_millis(100),
            },
            layout: LayoutConfig {
                pane_margin: 1,
                min_pane_width: 10,
                min_pane_height: 3,
                max_cache_entries: 100,
                smooth_transitions: false,
                transition_duration: std::time::Duration::from_millis(200),
            },
            performance: PerformanceConfig {
                dirty_tracking: true,
                render_batching: true,
                max_frame_time: 16,
                frame_skipping: true,
                memory_limit: 512,
                gpu_acceleration: true,
                cache_size_limit: 104857600,
            },
            features: FeatureConfig {
                smooth_scrolling: true,
                layout_transitions: false,
                text_shaping: false,
                decorations: true,
                transparency: false,
                blur_effects: false,
                animations: true,
                high_dpi: true,
            },
        };
        &DEFAULT_CONFIG
    }
    
    fn validate_config(&self, config: &GlazingConfig) -> super::config::ConfigValidation {
        config.validate()
    }
    
    fn get_performance_metrics(&self) -> &PerformanceTracker {
        &self.performance_metrics
    }
    
    fn get_render_stats(&self) -> RenderStatistics {
        RenderStatistics {
            frames_rendered: 0,
            average_frame_time: std::time::Duration::new(0, 0),
            current_fps: 60.0,
            memory_usage: 1024,
            peak_memory_usage: 2048,
            cache_hit_rate: 0.9,
            gpu_acceleration_enabled: true,
        }
    }
    
    fn set_performance_monitoring(&mut self, _enabled: bool) {}
    
    fn clear_performance_stats(&mut self) {}
    
    fn register_event_listener(&mut self, _event_type: GlazingEventType, _listener: Box<dyn GlazingEventListener>) {}
    
    fn emit_event(&mut self, _event: GlazingEvent) -> GlazingResult<()> {
        Ok(())
    }
    
    fn has_event_listeners(&self, _event_type: GlazingEventType) -> bool {
        false
    }
    
    fn set_debug_mode(&mut self, _enabled: bool) {}
    
    fn get_debug_info(&self) -> DebugInfo {
        DebugInfo {
            engine_state: "mock".to_string(),
            renderer_info: RendererDebugInfo {
                backend_type: "mock".to_string(),
                hardware_acceleration: false,
                text_quality: "mock".to_string(),
                font_family: "mock".to_string(),
                font_size: 12.0,
            },
            theme_info: ThemeDebugInfo {
                current_theme: "mock".to_string(),
                available_themes: vec!["mock".to_string()],
                base_colors: "mock".to_string(),
            },
            viewport_info: self.get_viewport_state(),
            performance_info: self.performance_metrics.clone(),
            config_info: "mock".to_string(),
            cache_stats: LayoutCacheStats { entries: 0, max_entries: 100 },
            event_stats: EventHandlerStats {
                total_listeners: 0,
                events_dispatched: 0,
                listener_errors: 0,
                slow_dispatches: 0,
                total_dispatch_time: std::time::Duration::new(0, 0),
            },
        }
    }
    
    fn dump_state(&self) -> String {
        "Mock Glazing State".to_string()
    }
    
    fn validate_state(&self) -> GlazingResult<()> {
        Ok(())
    }
}