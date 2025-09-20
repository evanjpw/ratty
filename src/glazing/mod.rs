pub mod renderer;
pub mod theme;
pub mod layout;
pub mod viewport;
pub mod interface;
pub mod events;
pub mod errors;
pub mod config;

#[cfg(test)]
mod tests;

pub use config::*;
pub use errors::*;
pub use events::*;
pub use interface::*;
pub use layout::*;
pub use renderer::*;
pub use theme::*;
pub use viewport::*;

use crate::pane::{CellAttributes, Cursor, Line, ScreenBuffer, ScrollbackBuffer};
use crate::sash::{PaneId, Theme};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout as RatatuiLayout, Rect},
    style::{Color as RatatuiColor, Modifier, Style as RatatuiStyle}
    ,
    text::{Line as RatatuiLine, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame as RatatuiFrame,
};
use std::time::{Duration, Instant};

/// The main Glazing engine - coordinates all rendering operations
pub struct GlazingEngine {
    /// Core rendering components
    renderer: TerminalRenderer,
    theme_engine: ThemeEngine,
    layout_manager: LayoutManager,
    
    /// Viewport and scrolling state
    viewport: Viewport,
    
    /// Performance tracking
    performance: PerformanceTracker,
    
    /// Configuration
    config: GlazingConfig,
    
    /// Current render state
    current_frame: Option<RenderFrame>,
    last_render_time: Instant,
    
    /// Event handling
    event_handler: GlazingEventHandler,
}

impl GlazingEngine {
    /// Create a new glazing engine
    pub fn new(config: GlazingConfig) -> GlazingResult<Self> {
        Ok(GlazingEngine {
            renderer: TerminalRenderer::new(&config)?,
            theme_engine: ThemeEngine::new(&config.theme)?,
            layout_manager: LayoutManager::new(),
            viewport: Viewport::new(),
            performance: PerformanceTracker::new(),
            config,
            current_frame: None,
            last_render_time: Instant::now(),
            event_handler: GlazingEventHandler::new(),
        })
    }
    
    /// Render a single pane to the given area
    pub fn render_pane<B: Backend>(
        &mut self,
        frame: &mut RatatuiFrame,
        area: Rect,
        screen_buffer: &ScreenBuffer,
        scrollback: &ScrollbackBuffer,
        cursor: &Cursor,
        pane_id: PaneId,
        is_active: bool,
    ) -> GlazingResult<()> {
        self.performance.start_frame();
        
        // Update viewport based on area
        self.viewport.update_dimensions(area.width, area.height);
        
        // Create render frame for this pane
        let render_frame = self.create_render_frame(
            screen_buffer,
            scrollback,
            cursor,
            pane_id,
            is_active,
        )?;
        
        // Render the frame
        self.renderer.render_frame::<B>(frame, area, &render_frame, &self.theme_engine)?;
        
        // Update performance metrics
        self.performance.end_frame();
        self.last_render_time = Instant::now();
        
        // Store frame for next render comparison
        self.current_frame = Some(render_frame);
        
        Ok(())
    }
    
    /// Render multiple panes in a layout
    pub fn render_layout<B: Backend>(
        &mut self,
        frame: &mut RatatuiFrame,
        area: Rect,
        panes: &[(PaneId, &ScreenBuffer, &ScrollbackBuffer, &Cursor, bool)], // (id, screen, scrollback, cursor, is_active)
        layout: &crate::sash::Layout,
    ) -> GlazingResult<()> {
        // Calculate layout areas
        let areas = self.layout_manager.calculate_pane_areas(area, panes.len(), layout)?;
        
        // Render each pane in its designated area
        for (i, (pane_id, screen_buffer, scrollback, cursor, is_active)) in panes.iter().enumerate() {
            if let Some(pane_area) = areas.get(i) {
                self.render_pane::<B>(
                    frame,
                    *pane_area,
                    screen_buffer,
                    scrollback,
                    cursor,
                    *pane_id,
                    *is_active,
                )?;
            }
        }
        
        Ok(())
    }
    
    /// Handle scroll events
    pub fn scroll(&mut self, direction: ScrollDirection, amount: usize) -> GlazingResult<()> {
        self.viewport.scroll(direction, amount)?;
        Ok(())
    }
    
    /// Apply a new theme
    pub fn apply_theme(&mut self, theme: &Theme) -> GlazingResult<()> {
        self.theme_engine.apply_theme(theme)?;
        // Force full redraw
        if let Some(ref mut frame) = self.current_frame {
            frame.mark_all_dirty();
        }
        Ok(())
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: GlazingConfig) -> GlazingResult<()> {
        self.config = config;
        self.renderer.update_config(&self.config)?;
        self.theme_engine.update_config(&self.config.theme)?;
        Ok(())
    }
    
    /// Get current performance metrics
    pub fn get_performance_metrics(&self) -> &PerformanceTracker {
        &self.performance
    }
    
    /// Create a render frame from pane content
    fn create_render_frame(
        &self,
        screen_buffer: &ScreenBuffer,
        scrollback: &ScrollbackBuffer,
        cursor: &Cursor,
        pane_id: PaneId,
        is_active: bool,
    ) -> GlazingResult<RenderFrame> {
        let mut rendered_lines = Vec::new();
        
        // Determine which lines to render based on viewport
        let (start_line, line_count) = self.viewport.get_visible_range(
            screen_buffer.height as usize,
            scrollback.len(),
        );
        
        // Render lines from scrollback if scrolled up
        if start_line < scrollback.len() {
            let scrollback_end = (start_line + line_count).min(scrollback.len());
            for i in start_line..scrollback_end {
                if let Some(line) = scrollback.get_line(i) {
                    let rendered_line = self.renderer.render_line(
                        line,
                        i,
                        &self.theme_engine,
                        false, // scrollback lines don't have cursor
                    )?;
                    rendered_lines.push(rendered_line);
                }
            }
        }
        
        // Render lines from screen buffer
        let screen_start = if start_line >= scrollback.len() {
            start_line - scrollback.len()
        } else {
            0
        };
        
        let screen_end = (screen_start + line_count - rendered_lines.len())
            .min(screen_buffer.lines.len());
        
        for i in screen_start..screen_end {
            if let Some(line) = screen_buffer.get_line(i) {
                let has_cursor = cursor.position.row as usize == i;
                let rendered_line = self.renderer.render_line(
                    line,
                    scrollback.len() + i,
                    &self.theme_engine,
                    has_cursor,
                )?;
                rendered_lines.push(rendered_line);
            }
        }
        
        // Render cursor if active and visible
        let rendered_cursor = if is_active && cursor.should_render() {
            Some(self.renderer.render_cursor(cursor, &self.theme_engine)?)
        } else {
            None
        };
        
        Ok(RenderFrame {
            content: rendered_lines,
            cursor: rendered_cursor,
            viewport: self.viewport.clone(),
            dirty_regions: Vec::new(), // Will be populated by dirty tracking
            pane_id,
            timestamp: Instant::now(),
        })
    }
}

/// A complete rendered frame for a pane
#[derive(Debug, Clone)]
pub struct RenderFrame {
    pub content: Vec<RenderedLine>,
    pub cursor: Option<RenderedCursor>,
    pub viewport: Viewport,
    pub dirty_regions: Vec<DirtyRegion>,
    pub pane_id: PaneId,
    pub timestamp: Instant,
}

impl RenderFrame {
    /// Mark all content as dirty for full redraw
    pub fn mark_all_dirty(&mut self) {
        for line in &mut self.content {
            line.dirty = true;
        }
        self.dirty_regions.clear();
        self.dirty_regions.push(DirtyRegion::Full);
    }
    
    /// Check if any content needs redrawing
    pub fn needs_redraw(&self) -> bool {
        !self.dirty_regions.is_empty() || 
        self.content.iter().any(|line| line.dirty)
    }
}

/// Performance tracking for rendering operations
#[derive(Debug, Clone)]
pub struct PerformanceTracker {
    frame_count: u64,
    total_render_time: Duration,
    last_frame_time: Duration,
    fps: f64,
    peak_memory_usage: usize,
    current_memory_usage: usize,
    frame_start: Option<Instant>,
}

impl PerformanceTracker {
    pub fn new() -> Self {
        PerformanceTracker {
            frame_count: 0,
            total_render_time: Duration::new(0, 0),
            last_frame_time: Duration::new(0, 0),
            fps: 0.0,
            peak_memory_usage: 0,
            current_memory_usage: 0,
            frame_start: None,
        }
    }
    
    pub fn start_frame(&mut self) {
        self.frame_start = Some(Instant::now());
    }
    
    pub fn end_frame(&mut self) {
        if let Some(start) = self.frame_start {
            let frame_time = start.elapsed();
            self.last_frame_time = frame_time;
            self.total_render_time += frame_time;
            self.frame_count += 1;
            
            // Calculate FPS (over last second)
            if frame_time.as_millis() > 0 {
                self.fps = 1000.0 / frame_time.as_millis() as f64;
            }
            
            self.frame_start = None;
        }
    }
    
    pub fn average_frame_time(&self) -> Duration {
        if self.frame_count > 0 {
            self.total_render_time / self.frame_count as u32
        } else {
            Duration::new(0, 0)
        }
    }
    
    pub fn fps(&self) -> f64 {
        self.fps
    }
    
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}

impl Default for PerformanceTracker {
    fn default() -> Self {
        Self::new()
    }
}