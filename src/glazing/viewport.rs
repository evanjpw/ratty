use super::*;
use std::cmp::{max, min};

/// Viewport management for scrollable terminal content
#[derive(Debug, Clone)]
pub struct Viewport {
    /// Current scroll offset (lines from top)
    pub scroll_offset: usize,
    
    /// Number of visible lines in the viewport
    pub visible_lines: usize,
    
    /// Total number of lines available (screen + scrollback)
    pub total_lines: usize,
    
    /// Horizontal scroll offset (columns from left)
    pub horizontal_offset: usize,
    
    /// Number of visible columns in the viewport
    pub visible_columns: usize,
    
    /// Total number of columns available
    pub total_columns: usize,
    
    /// Smooth scrolling configuration
    pub smooth_scrolling: bool,
    
    /// Current smooth scroll animation state
    smooth_scroll_state: Option<SmoothScrollState>,
}

impl Viewport {
    /// Create a new viewport
    pub fn new() -> Self {
        Viewport {
            scroll_offset: 0,
            visible_lines: 24,
            total_lines: 24,
            horizontal_offset: 0,
            visible_columns: 80,
            total_columns: 80,
            smooth_scrolling: true,
            smooth_scroll_state: None,
        }
    }
    
    /// Update viewport dimensions
    pub fn update_dimensions(&mut self, width: u16, height: u16) {
        self.visible_columns = width as usize;
        self.visible_lines = height as usize;
        
        // Clamp scroll offsets to valid ranges
        self.clamp_scroll_offsets();
    }
    
    /// Update total content size
    pub fn update_content_size(&mut self, screen_lines: usize, scrollback_lines: usize, columns: usize) {
        self.total_lines = screen_lines + scrollback_lines;
        self.total_columns = columns;
        
        // Clamp scroll offsets to valid ranges
        self.clamp_scroll_offsets();
    }
    
    /// Scroll in the given direction
    pub fn scroll(&mut self, direction: ScrollDirection, amount: usize) -> GlazingResult<()> {
        let old_offset = self.scroll_offset;
        
        match direction {
            ScrollDirection::Up => {
                self.scroll_offset = self.scroll_offset.saturating_sub(amount);
            }
            ScrollDirection::Down => {
                let max_scroll = self.max_scroll_offset();
                self.scroll_offset = min(self.scroll_offset + amount, max_scroll);
            }
            ScrollDirection::Left => {
                self.horizontal_offset = self.horizontal_offset.saturating_sub(amount);
            }
            ScrollDirection::Right => {
                let max_horizontal = self.max_horizontal_offset();
                self.horizontal_offset = min(self.horizontal_offset + amount, max_horizontal);
            }
            ScrollDirection::PageUp => {
                let page_size = self.visible_lines.saturating_sub(1);
                self.scroll_offset = self.scroll_offset.saturating_sub(page_size);
            }
            ScrollDirection::PageDown => {
                let page_size = self.visible_lines.saturating_sub(1);
                let max_scroll = self.max_scroll_offset();
                self.scroll_offset = min(self.scroll_offset + page_size, max_scroll);
            }
            ScrollDirection::Home => {
                self.scroll_offset = 0;
            }
            ScrollDirection::End => {
                self.scroll_offset = self.max_scroll_offset();
            }
        }
        
        // Start smooth scrolling animation if enabled and offset changed
        if self.smooth_scrolling && old_offset != self.scroll_offset {
            self.start_smooth_scroll(old_offset, self.scroll_offset);
        }
        
        Ok(())
    }
    
    /// Scroll to a specific line
    pub fn scroll_to_line(&mut self, line: usize) -> GlazingResult<()> {
        let old_offset = self.scroll_offset;
        self.scroll_offset = min(line, self.max_scroll_offset());
        
        if self.smooth_scrolling && old_offset != self.scroll_offset {
            self.start_smooth_scroll(old_offset, self.scroll_offset);
        }
        
        Ok(())
    }
    
    /// Get the range of visible lines (start_line, count)
    pub fn get_visible_range(&self, screen_lines: usize, scrollback_lines: usize) -> (usize, usize) {
        let total_lines = screen_lines + scrollback_lines;
        let start_line = min(self.scroll_offset, total_lines.saturating_sub(1));
        let end_line = min(start_line + self.visible_lines, total_lines);
        let count = end_line - start_line;
        
        (start_line, count)
    }
    
    /// Check if we're at the bottom (showing current screen)
    pub fn is_at_bottom(&self) -> bool {
        self.scroll_offset >= self.max_scroll_offset()
    }
    
    /// Check if we're at the top (showing oldest scrollback)
    pub fn is_at_top(&self) -> bool {
        self.scroll_offset == 0
    }
    
    /// Get scroll percentage (0.0 to 1.0)
    pub fn scroll_percentage(&self) -> f32 {
        let max_scroll = self.max_scroll_offset();
        if max_scroll == 0 {
            1.0
        } else {
            self.scroll_offset as f32 / max_scroll as f32
        }
    }
    
    /// Update smooth scrolling animation
    pub fn update_smooth_scroll(&mut self, delta_time: std::time::Duration) -> bool {
        if let Some(ref mut state) = self.smooth_scroll_state {
            state.update(delta_time);
            
            if state.is_complete() {
                self.smooth_scroll_state = None;
                false
            } else {
                true
            }
        } else {
            false
        }
    }
    
    /// Get current smooth scroll offset
    pub fn get_smooth_scroll_offset(&self) -> f32 {
        if let Some(ref state) = self.smooth_scroll_state {
            state.current_offset()
        } else {
            self.scroll_offset as f32
        }
    }
    
    /// Calculate maximum scroll offset
    fn max_scroll_offset(&self) -> usize {
        if self.total_lines > self.visible_lines {
            self.total_lines - self.visible_lines
        } else {
            0
        }
    }
    
    /// Calculate maximum horizontal offset
    fn max_horizontal_offset(&self) -> usize {
        if self.total_columns > self.visible_columns {
            self.total_columns - self.visible_columns
        } else {
            0
        }
    }
    
    /// Clamp scroll offsets to valid ranges
    fn clamp_scroll_offsets(&mut self) {
        self.scroll_offset = min(self.scroll_offset, self.max_scroll_offset());
        self.horizontal_offset = min(self.horizontal_offset, self.max_horizontal_offset());
    }
    
    /// Start smooth scrolling animation
    fn start_smooth_scroll(&mut self, from: usize, to: usize) {
        self.smooth_scroll_state = Some(SmoothScrollState::new(
            from as f32,
            to as f32,
            std::time::Duration::from_millis(150), // Animation duration
        ));
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new()
    }
}

/// Scroll direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    Home,
    End,
}

/// Smooth scrolling animation state
#[derive(Debug, Clone)]
struct SmoothScrollState {
    start_offset: f32,
    end_offset: f32,
    current_time: std::time::Duration,
    duration: std::time::Duration,
}

impl SmoothScrollState {
    fn new(start: f32, end: f32, duration: std::time::Duration) -> Self {
        SmoothScrollState {
            start_offset: start,
            end_offset: end,
            current_time: std::time::Duration::new(0, 0),
            duration,
        }
    }
    
    fn update(&mut self, delta_time: std::time::Duration) {
        self.current_time += delta_time;
    }
    
    fn is_complete(&self) -> bool {
        self.current_time >= self.duration
    }
    
    fn current_offset(&self) -> f32 {
        if self.is_complete() {
            self.end_offset
        } else {
            let progress = self.current_time.as_secs_f32() / self.duration.as_secs_f32();
            let eased_progress = self.ease_in_out_cubic(progress);
            self.start_offset + (self.end_offset - self.start_offset) * eased_progress
        }
    }
    
    // Smooth easing function
    fn ease_in_out_cubic(&self, t: f32) -> f32 {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
        }
    }
}

/// Scrollbar rendering helper
#[derive(Debug, Clone)]
pub struct ScrollbarInfo {
    pub total_lines: usize,
    pub visible_lines: usize,
    pub scroll_offset: usize,
    pub thumb_position: f32,
    pub thumb_size: f32,
}

impl ScrollbarInfo {
    /// Create scrollbar info from viewport
    pub fn from_viewport(viewport: &Viewport) -> Self {
        let total = viewport.total_lines.max(1);
        let visible = viewport.visible_lines.min(total);
        let thumb_size = visible as f32 / total as f32;
        let thumb_position = if total > visible {
            viewport.scroll_offset as f32 / (total - visible) as f32
        } else {
            0.0
        };
        
        ScrollbarInfo {
            total_lines: total,
            visible_lines: visible,
            scroll_offset: viewport.scroll_offset,
            thumb_position: thumb_position.clamp(0.0, 1.0),
            thumb_size: thumb_size.clamp(0.1, 1.0), // Minimum thumb size for usability
        }
    }
    
    /// Check if scrollbar should be visible
    pub fn should_show(&self) -> bool {
        self.total_lines > self.visible_lines
    }
}

/// Viewport state for serialization/debugging
#[derive(Debug, Clone, serde::Serialize)]
pub struct ViewportState {
    pub scroll_offset: usize,
    pub visible_lines: usize,
    pub total_lines: usize,
    pub horizontal_offset: usize,
    pub visible_columns: usize,
    pub total_columns: usize,
    pub is_at_bottom: bool,
    pub is_at_top: bool,
    pub scroll_percentage: f32,
}

impl From<&Viewport> for ViewportState {
    fn from(viewport: &Viewport) -> Self {
        ViewportState {
            scroll_offset: viewport.scroll_offset,
            visible_lines: viewport.visible_lines,
            total_lines: viewport.total_lines,
            horizontal_offset: viewport.horizontal_offset,
            visible_columns: viewport.visible_columns,
            total_columns: viewport.total_columns,
            is_at_bottom: viewport.is_at_bottom(),
            is_at_top: viewport.is_at_top(),
            scroll_percentage: viewport.scroll_percentage(),
        }
    }
}