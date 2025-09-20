use super::*;

/// Cursor position and state management
#[derive(Debug, Clone)]
pub struct Cursor {
    pub position: CursorPosition,
    pub style: CursorStyle,
    pub visibility: CursorVisibility,
    pub blink_state: BlinkState,
    pub saved_positions: Vec<CursorPosition>,
}

impl Cursor {
    /// Create a new cursor at origin
    pub fn new() -> Self {
        Cursor {
            position: CursorPosition::default(),
            style: CursorStyle::Block,
            visibility: CursorVisibility::Visible,
            blink_state: BlinkState::default(),
            saved_positions: Vec::new(),
        }
    }
    
    /// Set cursor position (1-based coordinates from VT sequences)
    pub fn set_position(&mut self, row: u16, col: u16, screen: &ScreenBuffer) -> PaneResult<()> {
        // Convert from 1-based to 0-based coordinates
        let zero_row = row.saturating_sub(1);
        let zero_col = col.saturating_sub(1);
        
        // Apply origin mode if enabled
        let final_row = if self.position.origin_mode {
            // TODO: In origin mode, row is relative to scroll region
            zero_row
        } else {
            zero_row
        };
        
        // Clamp to screen boundaries
        self.position.row = final_row.min(screen.height.saturating_sub(1));
        self.position.col = zero_col.min(screen.width.saturating_sub(1));
        
        Ok(())
    }
    
    /// Move cursor up by n rows
    pub fn move_up(&mut self, n: u16, screen: &ScreenBuffer) -> PaneResult<()> {
        let new_row = self.position.row.saturating_sub(n);
        self.set_position(new_row + 1, self.position.col + 1, screen)
    }
    
    /// Move cursor down by n rows
    pub fn move_down(&mut self, n: u16, screen: &ScreenBuffer) -> PaneResult<()> {
        let new_row = (self.position.row + n).min(screen.height.saturating_sub(1));
        self.set_position(new_row + 1, self.position.col + 1, screen)
    }
    
    /// Move cursor forward by n columns
    pub fn move_forward(&mut self, n: u16, screen: &ScreenBuffer) -> PaneResult<()> {
        let new_col = (self.position.col + n).min(screen.width.saturating_sub(1));
        self.set_position(self.position.row + 1, new_col + 1, screen)
    }
    
    /// Move cursor back by n columns
    pub fn move_back(&mut self, n: u16) -> PaneResult<()> {
        self.position.col = self.position.col.saturating_sub(n);
        Ok(())
    }
    
    /// Advance cursor after character input
    pub fn advance(&mut self, screen: &ScreenBuffer, modes: &TerminalModes) -> PaneResult<()> {
        self.position.col += 1;
        
        // Handle line wrapping
        if self.position.col >= screen.width {
            if modes.auto_wrap {
                self.position.col = 0;
                self.position.row += 1;
                
                // Handle scrolling if we've gone past the bottom
                if self.position.row >= screen.height {
                    self.position.row = screen.height.saturating_sub(1);
                    // TODO: Scroll screen up
                }
            } else {
                // Stay at the last column
                self.position.col = screen.width.saturating_sub(1);
            }
        }
        
        Ok(())
    }
    
    /// Handle line feed (move down, potentially scroll)
    pub fn line_feed(
        &mut self, 
        screen: &mut ScreenBuffer, 
        scrollback: &mut ScrollbackBuffer, 
        _modes: &TerminalModes
    ) -> PaneResult<()> {
        self.position.row += 1;
        
        // Check if we need to scroll
        if self.position.row >= screen.height {
            self.position.row = screen.height.saturating_sub(1);
            
            // Move the top line to scrollback and add a new line at bottom
            if let Some(top_line) = screen.lines.first().cloned() {
                scrollback.push_line(top_line);
            }
            
            // Shift all lines up
            for i in 1..screen.lines.len() {
                if let (Some(src), Some(dst)) = (screen.lines.get(i).cloned(), screen.lines.get_mut(i - 1)) {
                    *dst = src;
                }
            }
            
            // Add new blank line at bottom
            if let Some(last_line) = screen.lines.last_mut() {
                *last_line = Line::new(screen.width);
            }
            
            screen.mark_all_dirty();
        }
        
        Ok(())
    }
    
    /// Handle carriage return (move to beginning of line)
    pub fn carriage_return(&mut self) -> PaneResult<()> {
        self.position.col = 0;
        Ok(())
    }
    
    /// Handle tab forward
    pub fn tab_forward(&mut self, tabs: &TabStops, screen: &ScreenBuffer) -> PaneResult<()> {
        let next_tab = tabs.next_tab_stop(self.position.col);
        self.position.col = next_tab.min(screen.width.saturating_sub(1));
        Ok(())
    }
    
    /// Handle backspace
    pub fn backspace(&mut self, screen: &ScreenBuffer) -> PaneResult<()> {
        if self.position.col > 0 {
            self.position.col -= 1;
        } else if self.position.row > 0 {
            // Move to end of previous line
            self.position.row -= 1;
            self.position.col = screen.width.saturating_sub(1);
        }
        Ok(())
    }
    
    /// Save current cursor position
    pub fn save_position(&mut self) {
        self.saved_positions.push(self.position);
    }
    
    /// Restore last saved cursor position
    pub fn restore_position(&mut self, screen: &ScreenBuffer) -> PaneResult<()> {
        if let Some(saved_pos) = self.saved_positions.pop() {
            self.position = saved_pos;
            // Ensure position is still valid for current screen size
            self.position.row = self.position.row.min(screen.height.saturating_sub(1));
            self.position.col = self.position.col.min(screen.width.saturating_sub(1));
        }
        Ok(())
    }
    
    /// Set cursor visibility
    pub fn set_visibility(&mut self, visibility: CursorVisibility) {
        self.visibility = visibility;
    }
    
    /// Set cursor style
    pub fn set_style(&mut self, style: CursorStyle) {
        self.style = style;
    }
    
    /// Update blink state (called periodically for blinking cursors)
    pub fn update_blink(&mut self, elapsed: std::time::Duration) {
        self.blink_state.update(elapsed);
    }
    
    /// Check if cursor should be rendered (considering blink state)
    pub fn should_render(&self) -> bool {
        match self.visibility {
            CursorVisibility::Hidden => false,
            CursorVisibility::Visible => true,
            CursorVisibility::BlinkingBlock | 
            CursorVisibility::BlinkingUnderline | 
            CursorVisibility::BlinkingBar => self.blink_state.is_visible(),
        }
    }
}

/// Cursor position with coordinate and mode information
#[derive(Debug, Clone, Copy)]
pub struct CursorPosition {
    pub row: u16,
    pub col: u16,
    pub origin_mode: bool, // Affects row calculation with scroll regions
}

impl Default for CursorPosition {
    fn default() -> Self {
        CursorPosition {
            row: 0,
            col: 0,
            origin_mode: false,
        }
    }
}

/// Cursor visual style
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CursorStyle {
    Block,
    Underline,
    Bar,
}

/// Cursor visibility state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CursorVisibility {
    Visible,
    Hidden,
    BlinkingBlock,
    BlinkingUnderline,
    BlinkingBar,
}

/// Cursor blink state management
#[derive(Debug, Clone)]
pub struct BlinkState {
    is_visible: bool,
    last_toggle: std::time::Instant,
    blink_interval: std::time::Duration,
}

impl Default for BlinkState {
    fn default() -> Self {
        BlinkState {
            is_visible: true,
            last_toggle: std::time::Instant::now(),
            blink_interval: std::time::Duration::from_millis(500), // 500ms blink interval
        }
    }
}

impl BlinkState {
    /// Update blink state based on elapsed time
    pub fn update(&mut self, elapsed: std::time::Duration) {
        if elapsed >= self.blink_interval {
            self.is_visible = !self.is_visible;
            self.last_toggle = std::time::Instant::now();
        }
    }
    
    /// Check if cursor should be visible in current blink state
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }
    
    /// Force cursor to visible state
    pub fn set_visible(&mut self) {
        self.is_visible = true;
        self.last_toggle = std::time::Instant::now();
    }
    
    /// Set blink interval
    pub fn set_interval(&mut self, interval: std::time::Duration) {
        self.blink_interval = interval;
    }
}