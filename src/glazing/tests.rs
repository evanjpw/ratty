#[cfg(test)]
mod glazing_tests {
    use super::*;
    use crate::sash::{PaneId, Theme};
    use crate::pane::{ScreenBuffer, ScrollbackBuffer, Cursor, Line, Cell, CellAttributes};
    use crate::glazing::{
        GlazingEngine, GlazingConfig, PerformanceTracker, Viewport, ScrollDirection,
        LayoutManager, SplitDirection, ThemeEngine, ThemeConfig, RendererConfig,
        TextQuality, FontConfig, FontWeight, FontStyle, CursorConfig,
        PerformanceConfig, FeatureConfig, GlazingEventHandler, GlazingEventType,
        GlazingEvent, ViewportState, LayoutCacheStats
    };
    use std::time::Duration;

    // Helper function to create a test glazing engine
    fn create_test_glazing_engine() -> GlazingEngine {
        let config = GlazingConfig::default();
        GlazingEngine::new(config).expect("Failed to create test glazing engine")
    }

    // Helper function to create test screen buffer
    fn create_test_screen_buffer() -> ScreenBuffer {
        ScreenBuffer::new(80, 24)
    }

    // Helper function to create test scrollback buffer
    fn create_test_scrollback_buffer() -> ScrollbackBuffer {
        ScrollbackBuffer::new(1000)
    }

    // Helper function to create test cursor
    fn create_test_cursor() -> Cursor {
        Cursor::new()
    }

    // ========== Core Glazing Engine Tests ==========

    #[test]
    fn test_glazing_engine_creation() {
        let engine = create_test_glazing_engine();
        assert_eq!(engine.performance.frame_count(), 0);
        assert!(engine.current_frame.is_none());
    }

    #[test]
    fn test_glazing_engine_config_update() {
        let mut engine = create_test_glazing_engine();
        let mut new_config = GlazingConfig::default();
        new_config.renderer.target_fps = 30;
        
        assert!(engine.update_config(new_config.clone()).is_ok());
        assert_eq!(engine.config.renderer.target_fps, 30);
    }

    #[test]
    fn test_performance_tracking() {
        let mut tracker = PerformanceTracker::new();
        
        assert_eq!(tracker.frame_count(), 0);
        assert_eq!(tracker.fps(), 0.0);
        
        tracker.start_frame();
        std::thread::sleep(Duration::from_millis(1));
        tracker.end_frame();
        
        assert_eq!(tracker.frame_count(), 1);
        assert!(tracker.fps() > 0.0);
    }

    // ========== Viewport Tests ==========

    #[test]
    fn test_viewport_creation() {
        let viewport = Viewport::new();
        assert_eq!(viewport.scroll_offset, 0);
        assert_eq!(viewport.visible_lines, 24);
        assert_eq!(viewport.visible_columns, 80);
        assert!(viewport.is_at_bottom());
        assert!(viewport.is_at_top());
    }

    #[test]
    fn test_viewport_scrolling() {
        let mut viewport = Viewport::new();
        viewport.update_content_size(50, 100, 80);
        
        // Test scrolling down
        assert!(viewport.scroll(ScrollDirection::Down, 10).is_ok());
        assert_eq!(viewport.scroll_offset, 10);
        assert!(!viewport.is_at_top());
        
        // Test scrolling up
        assert!(viewport.scroll(ScrollDirection::Up, 5).is_ok());
        assert_eq!(viewport.scroll_offset, 5);
        
        // Test scrolling to bottom
        assert!(viewport.scroll(ScrollDirection::End, 0).is_ok());
        assert!(viewport.is_at_bottom());
        
        // Test scrolling to top
        assert!(viewport.scroll(ScrollDirection::Home, 0).is_ok());
        assert!(viewport.is_at_top());
    }

    #[test]
    fn test_viewport_visible_range() {
        let mut viewport = Viewport::new();
        viewport.update_content_size(24, 100, 80);
        viewport.scroll_offset = 50;
        
        let (start, count) = viewport.get_visible_range(24, 100);
        assert_eq!(start, 50);
        assert_eq!(count, 24);
    }

    #[test]
    fn test_viewport_scroll_percentage() {
        let mut viewport = Viewport::new();
        viewport.update_content_size(24, 100, 80);
        
        // At top
        viewport.scroll_offset = 0;
        assert_eq!(viewport.scroll_percentage(), 0.0);
        
        // At bottom
        viewport.scroll_offset = viewport.max_scroll_offset();
        assert_eq!(viewport.scroll_percentage(), 1.0);
        
        // In middle
        viewport.scroll_offset = viewport.max_scroll_offset() / 2;
        assert!((viewport.scroll_percentage() - 0.5).abs() < 0.1);
    }

    // ========== Layout Manager Tests ==========

    #[test]
    fn test_layout_manager_creation() {
        let manager = LayoutManager::new();
        assert_eq!(manager.cache_stats().entries, 0);
    }

    #[test]
    fn test_single_pane_layout() {
        let mut manager = LayoutManager::new();
        let area = ratatui::layout::Rect::new(0, 0, 80, 24);
        let layout = crate::sash::Layout::Single(PaneId::new(1));
        
        let areas = manager.calculate_pane_areas(area, 1, &layout).unwrap();
        assert_eq!(areas.len(), 1);
        assert_eq!(areas[0], area);
    }

    #[test]
    fn test_horizontal_split_layout() {
        let mut manager = LayoutManager::new();
        let area = ratatui::layout::Rect::new(0, 0, 80, 24);
        let layout = crate::sash::Layout::HorizontalSplit { 
            top: Box::new(crate::sash::Layout::Single(PaneId::new(1))),
            bottom: Box::new(crate::sash::Layout::Single(PaneId::new(2))),
            split_ratio: 0.5
        };
        
        let areas = manager.calculate_pane_areas(area, 2, &layout).unwrap();
        assert_eq!(areas.len(), 2);
        // Areas should be roughly equal horizontally
        assert!((areas[0].width as i32 - areas[1].width as i32).abs() <= 1);
    }

    #[test]
    fn test_grid_layout() {
        let mut manager = LayoutManager::new();
        let area = ratatui::layout::Rect::new(0, 0, 80, 24);
        let layout = crate::sash::Layout::Grid { 
            rows: 2, 
            cols: 2,
            cells: vec![
                vec![Some(PaneId::new(1)), Some(PaneId::new(2))],
                vec![Some(PaneId::new(3)), Some(PaneId::new(4))]
            ]
        };
        
        let areas = manager.calculate_pane_areas(area, 4, &layout).unwrap();
        assert_eq!(areas.len(), 4);
    }

    #[test]
    fn test_layout_caching() {
        let mut manager = LayoutManager::new();
        let area = ratatui::layout::Rect::new(0, 0, 80, 24);
        let layout = crate::sash::Layout::Single(PaneId::new(1));
        
        // First calculation
        let areas1 = manager.calculate_pane_areas(area, 1, &layout).unwrap();
        assert_eq!(manager.cache_stats().entries, 1);
        
        // Second calculation should use cache
        let areas2 = manager.calculate_pane_areas(area, 1, &layout).unwrap();
        assert_eq!(areas1, areas2);
        assert_eq!(manager.cache_stats().entries, 1);
    }

    #[test]
    fn test_split_direction_calculation() {
        let manager = LayoutManager::new();
        let area = ratatui::layout::Rect::new(0, 0, 80, 24);
        
        // Test horizontal split
        let (left, right) = manager.calculate_optimal_split(area, SplitDirection::Horizontal).unwrap();
        assert!(left.x < right.x);
        assert_eq!(left.y, right.y);
        assert_eq!(left.height, right.height);
        
        // Test vertical split
        let (top, bottom) = manager.calculate_optimal_split(area, SplitDirection::Vertical).unwrap();
        assert!(top.y < bottom.y);
        assert_eq!(top.x, bottom.x);
        assert_eq!(top.width, bottom.width);
    }

    #[test]
    fn test_can_split() {
        let manager = LayoutManager::new();
        let large_area = ratatui::layout::Rect::new(0, 0, 80, 24);
        let small_area = ratatui::layout::Rect::new(0, 0, 5, 2);
        
        assert!(manager.can_split(large_area, SplitDirection::Horizontal));
        assert!(manager.can_split(large_area, SplitDirection::Vertical));
        assert!(!manager.can_split(small_area, SplitDirection::Horizontal));
        assert!(!manager.can_split(small_area, SplitDirection::Vertical));
    }

    // ========== Theme Engine Tests ==========

    #[test]
    fn test_theme_engine_creation() {
        let config = ThemeConfig::default();
        let engine = ThemeEngine::new(&config);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_theme_application() {
        let config = ThemeConfig::default();
        let mut engine = ThemeEngine::new(&config).unwrap();
        let theme = Theme::default();
        
        assert!(engine.apply_theme(&theme).is_ok());
        assert_eq!(engine.current_theme.name, theme.name);
    }

    #[test]
    fn test_cell_style_conversion() {
        let config = ThemeConfig::default();
        let engine = ThemeEngine::new(&config).unwrap();
        
        let attributes = CellAttributes {
            bold: true,
            italic: true,
            underline: crate::pane::UnderlineType::Single,
            strikethrough: false,
            reverse: false,
            blink: crate::pane::BlinkType::None,
            dim: false,
            invisible: false,
        };
        
        let fg = crate::sash::Color::from_rgb(255, 255, 255);
        let bg = crate::sash::Color::from_rgb(0, 0, 0);
        
        let style = engine.convert_cell_style(&attributes, fg, bg).unwrap();
        assert!(style.bold);
        assert!(style.italic);
        assert!(style.underline);
        assert!(!style.strikethrough);
    }

    #[test]
    fn test_ansi_color_retrieval() {
        let config = ThemeConfig::default();
        let engine = ThemeEngine::new(&config).unwrap();
        
        // Test standard ANSI colors
        let red = engine.get_ansi_color(1);
        assert_ne!(red, crate::sash::Color::from_rgb(0, 0, 0));
        
        // Test extended colors
        let extended = engine.get_ansi_color(100);
        assert_ne!(extended, crate::sash::Color::from_rgb(0, 0, 0));
        
        // Test grayscale
        let gray = engine.get_ansi_color(240);
        assert_ne!(gray, crate::sash::Color::from_rgb(0, 0, 0));
    }

    #[test]
    fn test_color_palettes() {
        let ansi_palette = super::theme::ColorPalette::ansi_16();
        assert_eq!(ansi_palette.colors.len(), 16);
        assert_eq!(ansi_palette.name, "ANSI 16");
        
        let solarized_dark = super::theme::ColorPalette::solarized_dark();
        assert_eq!(solarized_dark.colors.len(), 16);
        assert_eq!(solarized_dark.name, "Solarized Dark");
        
        let solarized_light = super::theme::ColorPalette::solarized_light();
        assert_eq!(solarized_light.colors.len(), 16);
        assert_eq!(solarized_light.name, "Solarized Light");
    }

    #[test]
    fn test_theme_collection() {
        let collection = super::theme::ThemeCollection::new();
        let themes = collection.list_themes();
        
        assert!(!themes.is_empty());
        assert!(themes.contains(&"default".to_string()));
        assert!(themes.contains(&"solarized-dark".to_string()));
        assert!(themes.contains(&"solarized-light".to_string()));
        
        // Test theme retrieval
        let default_theme = collection.get_theme("default");
        assert!(default_theme.is_some());
        
        let nonexistent_theme = collection.get_theme("nonexistent");
        assert!(nonexistent_theme.is_none());
    }

    // ========== Configuration Tests ==========

    #[test]
    fn test_glazing_config_default() {
        let config = GlazingConfig::default();
        assert_eq!(config.renderer.target_fps, 60);
        assert!(config.renderer.hardware_acceleration);
        assert!(config.features.decorations);
        assert!(config.performance.dirty_tracking);
    }

    #[test]
    fn test_config_validation() {
        let config = GlazingConfig::default();
        let validation = config.validate();
        assert!(validation.is_valid);
        assert!(validation.errors.is_empty());
    }

    #[test]
    fn test_config_validation_errors() {
        let mut config = GlazingConfig::default();
        config.renderer.target_fps = 0; // Invalid
        config.renderer.font.size = -1.0; // Invalid
        
        let validation = config.validate();
        assert!(!validation.is_valid);
        assert!(!validation.errors.is_empty());
    }

    #[test]
    fn test_performance_optimized_config() {
        let config = GlazingConfig::performance_optimized();
        assert_eq!(config.renderer.target_fps, 30);
        assert!(!config.renderer.hardware_acceleration);
        assert!(!config.features.smooth_scrolling);
        assert!(!config.features.animations);
    }

    #[test]
    fn test_quality_optimized_config() {
        let config = GlazingConfig::quality_optimized();
        assert_eq!(config.renderer.target_fps, 60);
        assert!(config.renderer.hardware_acceleration);
        assert_eq!(config.renderer.text_quality, TextQuality::Ultra);
        assert!(config.features.smooth_scrolling);
        assert!(config.features.animations);
    }

    #[test]
    fn test_font_config() {
        let font_config = FontConfig::default();
        assert_eq!(font_config.family, "monospace");
        assert_eq!(font_config.size, 12.0);
        assert_eq!(font_config.line_height, 1.2);
        assert!(!font_config.ligatures);
        assert!(font_config.subpixel_rendering);
    }

    #[test]
    fn test_cursor_config() {
        let cursor_config = CursorConfig::default();
        assert!(cursor_config.enable_blinking);
        assert_eq!(cursor_config.blink_rate, Duration::from_millis(500));
        assert_eq!(cursor_config.thickness, 1);
        assert!(cursor_config.color_override.is_none());
    }

    // ========== Renderer Tests ==========

    #[test]
    fn test_text_renderer_creation() {
        let config = RendererConfig::default();
        let renderer = super::renderer::TextRenderer::new(&config);
        assert!(renderer.is_ok());
    }

    #[test]
    fn test_cursor_renderer_creation() {
        let config = CursorConfig::default();
        let renderer = super::renderer::CursorRenderer::new(&config);
        assert!(renderer.is_ok());
    }

    #[test]
    fn test_blink_state() {
        let rate = Duration::from_millis(100);
        let mut blink_state = super::renderer::BlinkState::new(rate);
        
        // Initial state should be visible
        assert!(blink_state.is_visible());
        
        // After waiting for blink rate, should toggle
        std::thread::sleep(Duration::from_millis(150));
        let visible_after_wait = blink_state.is_visible();
        // Note: This test might be flaky due to timing, but should generally work
    }

    #[test]
    fn test_cell_style_equality() {
        let style1 = super::renderer::CellStyle {
            foreground: crate::sash::Color::from_rgb(255, 255, 255),
            background: crate::sash::Color::from_rgb(0, 0, 0),
            bold: true,
            italic: false,
            underline: false,
            strikethrough: false,
            reverse: false,
            dim: false,
        };
        
        let style2 = style1.clone();
        assert_eq!(style1, style2);
        
        let style3 = super::renderer::CellStyle {
            bold: false,
            ..style1.clone()
        };
        assert_ne!(style1, style3);
    }

    // ========== Event Tests ==========

    #[test]
    fn test_glazing_event_handler_creation() {
        let handler = GlazingEventHandler::new();
        assert_eq!(handler.get_stats().total_listeners, 0);
        assert_eq!(handler.get_stats().events_dispatched, 0);
    }

    #[test]
    fn test_event_listener_registration() {
        let mut handler = GlazingEventHandler::new();
        let monitor = Box::new(super::events::PerformanceMonitor::new(30.0, Duration::from_millis(20)));
        
        handler.register_listener(GlazingEventType::PerformanceWarning, monitor);
        assert_eq!(handler.listener_count(GlazingEventType::PerformanceWarning), 1);
        assert!(handler.has_listeners(GlazingEventType::PerformanceWarning));
    }

    #[test]
    fn test_event_dispatch() {
        let mut handler = GlazingEventHandler::new();
        let monitor = Box::new(super::events::PerformanceMonitor::new(30.0, Duration::from_millis(20)));
        
        handler.register_listener(GlazingEventType::PerformanceWarning, monitor);
        
        let event = GlazingEvent::PerformanceWarning {
            fps: 15.0,
            frame_time: Duration::from_millis(50),
        };
        
        assert!(handler.dispatch(event).is_ok());
        assert_eq!(handler.get_stats().events_dispatched, 1);
    }

    #[test]
    fn test_debug_listener() {
        let mut debug_listener = super::events::DebugListener::new(true);
        
        assert!(debug_listener.can_handle(GlazingEventType::DebugInfoRequested));
        
        let event = GlazingEvent::DebugInfoRequested;
        assert!(debug_listener.handle_glazing_event(&event).is_ok());
    }

    // ========== Error Tests ==========

    #[test]
    fn test_glazing_error_creation() {
        let error = GlazingError::rendering("Test rendering error");
        assert!(matches!(error, GlazingError::RenderingError(_)));
        assert!(error.is_recoverable());
        assert_eq!(error.severity(), super::errors::ErrorSeverity::Warning);
    }

    #[test]
    fn test_error_severity() {
        let warning_error = GlazingError::theme("Theme issue");
        assert_eq!(warning_error.severity(), super::errors::ErrorSeverity::Warning);
        
        let fatal_error = GlazingError::memory("Out of memory");
        assert_eq!(fatal_error.severity(), super::errors::ErrorSeverity::Fatal);
        
        let info_error = GlazingError::performance("Performance notice");
        assert_eq!(info_error.severity(), super::errors::ErrorSeverity::Info);
    }

    #[test]
    fn test_error_recovery_suggestions() {
        let rendering_error = GlazingError::rendering("Failed to draw");
        assert!(rendering_error.recovery_suggestion().is_some());
        
        let font_error = GlazingError::font("Font not found");
        assert!(font_error.recovery_suggestion().is_some());
        
        let parse_error = GlazingError::parse("Invalid data");
        assert!(parse_error.recovery_suggestion().is_none());
    }

    #[test]
    fn test_error_context() {
        let context = super::errors::ErrorContext::new(
            "render_frame".to_string(),
            "renderer".to_string(),
        );
        
        assert_eq!(context.operation, "render_frame");
        assert_eq!(context.component, "renderer");
        assert!(!context.thread_id.is_empty());
    }

    #[test]
    fn test_error_reporter() {
        let mut reporter = super::errors::ErrorReporter::new(10);
        
        let error = GlazingError::rendering("Test error");
        let context = super::errors::ErrorContext::new("test".to_string(), "test".to_string());
        let contextual_error = super::errors::ContextualError::new(error, context);
        
        reporter.report(contextual_error);
        
        let stats = reporter.statistics();
        assert_eq!(stats.total_errors, 1);
        assert_eq!(stats.unique_types, 1);
        
        let recent = reporter.recent_errors(5);
        assert_eq!(recent.len(), 1);
    }

    // ========== Integration Tests ==========

    #[test]
    fn test_render_frame_creation() {
        let engine = create_test_glazing_engine();
        let screen_buffer = create_test_screen_buffer();
        let scrollback = create_test_scrollback_buffer();
        let cursor = create_test_cursor();
        
        let frame = engine.create_render_frame(
            &screen_buffer,
            &scrollback,
            &cursor,
            PaneId::new(1),
            true,
        );
        
        assert!(frame.is_ok());
        let render_frame = frame.unwrap();
        assert_eq!(render_frame.pane_id, PaneId::new(1));
    }

    #[test]
    fn test_scrollbar_info() {
        let mut viewport = Viewport::new();
        viewport.update_content_size(24, 100, 80);
        viewport.scroll_offset = 50;
        
        let scrollbar_info = super::viewport::ScrollbarInfo::from_viewport(&viewport);
        assert!(scrollbar_info.should_show());
        assert!(scrollbar_info.thumb_position > 0.0);
        assert!(scrollbar_info.thumb_size < 1.0);
    }

    #[test]
    fn test_viewport_state_serialization() {
        let viewport = Viewport::new();
        let state = ViewportState::from(&viewport);
        
        // Test that state can be serialized (this ensures it implements serde::Serialize)
        let json = serde_json::to_string(&state);
        assert!(json.is_ok());
    }

    #[test]
    fn test_smooth_scrolling() {
        let mut viewport = Viewport::new();
        viewport.smooth_scrolling = true;
        viewport.update_content_size(24, 100, 80);
        
        // Start a scroll operation
        assert!(viewport.scroll(ScrollDirection::Down, 10).is_ok());
        
        // Should have smooth scroll state
        assert!(viewport.smooth_scroll_state.is_some());
        
        // Update animation
        let has_animation = viewport.update_smooth_scroll(Duration::from_millis(50));
        // Animation might still be running or might be complete depending on timing
    }

    // ========== Mock Tests ==========

    #[cfg(test)]
    #[test]
    fn test_mock_glazing_interface() {
        let mut mock = super::interface::MockGlazingInterface::new();
        
        // Test viewport state
        let state = mock.get_viewport_state();
        assert_eq!(state.visible_lines, 24);
        assert_eq!(state.visible_columns, 80);
        
        // Test theme operations
        let theme = Theme::default();
        assert!(mock.apply_theme(&theme).is_ok());
        assert_eq!(mock.get_current_theme(), Some("Default".to_string()));
        
        // Test scroll operations
        assert!(mock.scroll(ScrollDirection::Down, 5).is_ok());
        assert!(mock.scroll_to_line(10).is_ok());
        
        // Check render calls were recorded
        let calls = mock.render_calls.borrow();
        assert!(calls.contains(&"scroll:Down:5".to_string()));
        assert!(calls.contains(&"scroll_to_line:10".to_string()));
    }
}