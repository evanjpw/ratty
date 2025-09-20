use super::*;
use ratatui::layout::{Constraint, Direction, Layout as RatatuiLayout, Rect};

/// Layout manager for organizing panes within the display area
#[derive(Debug)]
pub struct LayoutManager {
    config: LayoutConfig,
    cached_layouts: std::collections::HashMap<LayoutCacheKey, Vec<Rect>>,
}

impl LayoutManager {
    /// Create a new layout manager
    pub fn new() -> Self {
        LayoutManager {
            config: LayoutConfig::default(),
            cached_layouts: std::collections::HashMap::new(),
        }
    }
    
    /// Calculate areas for panes based on layout
    pub fn calculate_pane_areas(
        &mut self,
        area: Rect,
        pane_count: usize,
        layout: &crate::sash::Layout,
    ) -> GlazingResult<Vec<Rect>> {
        if pane_count == 0 {
            return Ok(Vec::new());
        }
        
        // Check cache first
        let cache_key = LayoutCacheKey {
            area,
            pane_count,
            layout: layout.clone(),
        };
        
        if let Some(cached) = self.cached_layouts.get(&cache_key) {
            return Ok(cached.clone());
        }
        
        // Calculate new layout
        let areas = match layout {
            crate::sash::Layout::Single(_) => {
                vec![area]
            }
            crate::sash::Layout::HorizontalSplit { split_ratio, .. } => {
                self.calculate_horizontal_split(area, pane_count, &[*split_ratio, 1.0 - split_ratio])?
            }
            crate::sash::Layout::VerticalSplit { split_ratio, .. } => {
                self.calculate_vertical_split(area, pane_count, &[*split_ratio, 1.0 - split_ratio])?
            }
            crate::sash::Layout::Grid { rows, cols, .. } => {
                self.calculate_grid_layout(area, *rows, *cols)?
            }
            crate::sash::Layout::Custom { layout, .. } => {
                // Recursively handle the nested layout
                self.calculate_pane_areas(area, pane_count, layout)?
            }
            crate::sash::Layout::Empty => {
                Vec::new()
            }
            crate::sash::Layout::Tabs { tabs, .. } => {
                // For tabs, just use the full area
                vec![area]
            }
        };
        
        // Cache the result
        self.cached_layouts.insert(cache_key, areas.clone());
        
        // Limit cache size
        if self.cached_layouts.len() > self.config.max_cache_entries {
            self.cached_layouts.clear();
        }
        
        Ok(areas)
    }
    
    /// Calculate horizontal split layout
    fn calculate_horizontal_split(
        &self,
        area: Rect,
        pane_count: usize,
        ratios: &[f32],
    ) -> GlazingResult<Vec<Rect>> {
        if pane_count == 0 {
            return Ok(Vec::new());
        }
        
        let constraints = if ratios.len() >= pane_count {
            // Use provided ratios
            ratios[..pane_count]
                .iter()
                .map(|&ratio| Constraint::Percentage((ratio * 100.0) as u16))
                .collect()
        } else {
            // Equal distribution
            let percentage = 100 / pane_count as u16;
            vec![Constraint::Percentage(percentage); pane_count]
        };
        
        let layout = RatatuiLayout::default()
            .direction(Direction::Horizontal)
            .margin(self.config.pane_margin)
            .constraints(constraints);

        // TODO: This was originally `layout.split(area)`. Verify that it is OK to use `to_vec()`.
        Ok(layout.split(area).to_vec())
    }
    
    /// Calculate vertical split layout
    fn calculate_vertical_split(
        &self,
        area: Rect,
        pane_count: usize,
        ratios: &[f32],
    ) -> GlazingResult<Vec<Rect>> {
        if pane_count == 0 {
            return Ok(Vec::new());
        }
        
        let constraints = if ratios.len() >= pane_count {
            // Use provided ratios
            ratios[..pane_count]
                .iter()
                .map(|&ratio| Constraint::Percentage((ratio * 100.0) as u16))
                .collect()
        } else {
            // Equal distribution
            let percentage = 100 / pane_count as u16;
            vec![Constraint::Percentage(percentage); pane_count]
        };
        
        let layout = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .margin(self.config.pane_margin)
            .constraints(constraints);

        // TODO: This was originally `layout.split(area)`. Verify that it is OK to use `to_vec()`.
        Ok(layout.split(area).to_vec())
    }
    
    /// Calculate grid layout
    fn calculate_grid_layout(
        &self,
        area: Rect,
        rows: usize,
        cols: usize,
    ) -> GlazingResult<Vec<Rect>> {
        if rows == 0 || cols == 0 {
            return Ok(Vec::new());
        }
        
        let row_height = area.height / rows as u16;
        let col_width = area.width / cols as u16;
        
        let mut areas = Vec::new();
        
        for row in 0..rows {
            for col in 0..cols {
                let x = area.x + col as u16 * col_width;
                let y = area.y + row as u16 * row_height;
                
                // Adjust width and height for last column/row to fill remaining space
                let width = if col == cols - 1 {
                    area.width - (col as u16 * col_width)
                } else {
                    col_width
                };
                
                let height = if row == rows - 1 {
                    area.height - (row as u16 * row_height)
                } else {
                    row_height
                };
                
                areas.push(Rect { x, y, width, height });
            }
        }
        
        Ok(areas)
    }
    
    /// Calculate custom layout (placeholder implementation)
    fn _calculate_custom_layout(
        &self,
        area: Rect,
        pane_count: usize,
        _config: &str,
    ) -> GlazingResult<Vec<Rect>> {
        // Parse custom layout configuration
        // For now, fall back to equal horizontal split
        // TODO: Implement custom layout parsing
        self.calculate_horizontal_split(area, pane_count, &[])
    }
    
    /// Calculate optimal split for adding a new pane
    pub fn calculate_optimal_split(
        &self,
        current_area: Rect,
        split_direction: SplitDirection,
    ) -> GlazingResult<(Rect, Rect)> {
        match split_direction {
            SplitDirection::Horizontal => {
                let split_x = current_area.x + current_area.width / 2;
                let left_area = Rect {
                    x: current_area.x,
                    y: current_area.y,
                    width: split_x - current_area.x,
                    height: current_area.height,
                };
                let right_area = Rect {
                    x: split_x,
                    y: current_area.y,
                    width: current_area.x + current_area.width - split_x,
                    height: current_area.height,
                };
                Ok((left_area, right_area))
            }
            SplitDirection::Vertical => {
                let split_y = current_area.y + current_area.height / 2;
                let top_area = Rect {
                    x: current_area.x,
                    y: current_area.y,
                    width: current_area.width,
                    height: split_y - current_area.y,
                };
                let bottom_area = Rect {
                    x: current_area.x,
                    y: split_y,
                    width: current_area.width,
                    height: current_area.y + current_area.height - split_y,
                };
                Ok((top_area, bottom_area))
            }
        }
    }
    
    /// Check if area is large enough for splitting
    pub fn can_split(&self, area: Rect, direction: SplitDirection) -> bool {
        match direction {
            SplitDirection::Horizontal => {
                area.width >= self.config.min_pane_width * 2 + self.config.pane_margin * 2
            }
            SplitDirection::Vertical => {
                area.height >= self.config.min_pane_height * 2 + self.config.pane_margin * 2
            }
        }
    }
    
    /// Update layout configuration
    pub fn update_config(&mut self, config: LayoutConfig) {
        self.config = config;
        self.cached_layouts.clear(); // Clear cache when config changes
    }
    
    /// Clear layout cache
    pub fn clear_cache(&mut self) {
        self.cached_layouts.clear();
    }
    
    /// Get cache statistics
    pub fn cache_stats(&self) -> LayoutCacheStats {
        LayoutCacheStats {
            entries: self.cached_layouts.len(),
            max_entries: self.config.max_cache_entries,
        }
    }
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Split direction for layout calculations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

/// Layout configuration
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    /// Margin between panes
    pub pane_margin: u16,
    
    /// Minimum pane width
    pub min_pane_width: u16,
    
    /// Minimum pane height
    pub min_pane_height: u16,
    
    /// Maximum number of cached layout calculations
    pub max_cache_entries: usize,
    
    /// Whether to use smooth layout transitions
    pub smooth_transitions: bool,
    
    /// Transition duration for layout changes
    pub transition_duration: std::time::Duration,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        LayoutConfig {
            pane_margin: 1,
            min_pane_width: 10,
            min_pane_height: 3,
            max_cache_entries: 100,
            smooth_transitions: false, // Disabled for now
            transition_duration: std::time::Duration::from_millis(200),
        }
    }
}

/// Cache key for layout calculations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct LayoutCacheKey {
    area: Rect,
    pane_count: usize,
    layout: crate::sash::Layout,
}

/// Layout cache statistics
#[derive(Debug, Clone)]
pub struct LayoutCacheStats {
    pub entries: usize,
    pub max_entries: usize,
}

/// Layout animation state for smooth transitions
#[derive(Debug, Clone)]
pub struct LayoutTransition {
    from_areas: Vec<Rect>,
    to_areas: Vec<Rect>,
    start_time: std::time::Instant,
    duration: std::time::Duration,
}

impl LayoutTransition {
    /// Create a new layout transition
    pub fn new(
        from_areas: Vec<Rect>,
        to_areas: Vec<Rect>,
        duration: std::time::Duration,
    ) -> Self {
        LayoutTransition {
            from_areas,
            to_areas,
            start_time: std::time::Instant::now(),
            duration,
        }
    }
    
    /// Get current interpolated areas
    pub fn current_areas(&self) -> Vec<Rect> {
        let elapsed = self.start_time.elapsed();
        let progress = (elapsed.as_secs_f32() / self.duration.as_secs_f32()).clamp(0.0, 1.0);
        
        if progress >= 1.0 {
            return self.to_areas.clone();
        }
        
        let mut current_areas = Vec::new();
        let max_len = self.from_areas.len().max(self.to_areas.len());
        
        for i in 0..max_len {
            let from_rect = self.from_areas.get(i).copied().unwrap_or_default();
            let to_rect = self.to_areas.get(i).copied().unwrap_or_default();
            
            let interpolated = Rect {
                x: interpolate_u16(from_rect.x, to_rect.x, progress),
                y: interpolate_u16(from_rect.y, to_rect.y, progress),
                width: interpolate_u16(from_rect.width, to_rect.width, progress),
                height: interpolate_u16(from_rect.height, to_rect.height, progress),
            };
            
            current_areas.push(interpolated);
        }
        
        current_areas
    }
    
    /// Check if transition is complete
    pub fn is_complete(&self) -> bool {
        self.start_time.elapsed() >= self.duration
    }
}

/// Interpolate between two u16 values
fn interpolate_u16(from: u16, to: u16, progress: f32) -> u16 {
    (from as f32 + (to as i32 - from as i32) as f32 * progress) as u16
}