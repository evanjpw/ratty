use super::*;
use crate::sash::{PaneId, Theme};

/// Primary interface for Pane operations
pub trait PaneInterface: Send + Sync {
    // ========== Identity and Basic State ==========
    
    /// Get the pane ID
    fn id(&self) -> PaneId;
    
    /// Check if this pane is active
    fn is_active(&self) -> bool;
    
    /// Set the active state
    fn set_active(&mut self, active: bool);
    
    /// Get the pane title
    fn get_title(&self) -> &str;
    
    /// Set the pane title
    fn set_title(&mut self, title: String);
    
    /// Check if the pane has been modified
    fn is_modified(&self) -> bool;
    
    /// Set the modified state
    fn set_modified(&mut self, modified: bool);
    
    // ========== Process and PTY Management ==========
    
    /// Spawn a new process in this pane
    fn spawn_process(&mut self, command: &str, args: &[&str], env: &[(String, String)]) -> PaneResult<()>;
    
    /// Kill the current process
    fn kill_process(&mut self) -> PaneResult<()>;
    
    /// Check if the process is still alive
    fn is_process_alive(&self) -> bool;
    
    /// Get the process ID
    fn get_process_id(&self) -> Option<u32>;
    
    // ========== Input/Output ==========
    
    /// Write input data to the PTY
    fn write_input(&mut self, data: &[u8]) -> PaneResult<usize>;
    
    /// Read output data from the PTY
    fn read_output(&mut self) -> PaneResult<Vec<u8>>;
    
    /// Process output data through the terminal emulator
    fn process_output(&mut self, data: &[u8]) -> PaneResult<()>;
    
    // ========== Terminal State ==========
    
    /// Resize the terminal
    fn resize(&mut self, rows: u16, cols: u16) -> PaneResult<()>;
    
    /// Get the current terminal size
    fn get_size(&self) -> (u16, u16);
    
    /// Get the current cursor position
    fn get_cursor_position(&self) -> (u16, u16);
    
    /// Get the current terminal mode
    fn get_terminal_mode(&self) -> TerminalMode;
    
    // ========== Content Access ==========
    
    /// Get a specific line from the screen buffer
    fn get_line(&self, index: usize) -> Option<&Line>;
    
    /// Get the entire screen content
    fn get_screen_content(&self) -> &ScreenBuffer;
    
    /// Get the scrollback buffer
    fn get_scrollback(&self) -> &ScrollbackBuffer;
    
    /// Get the current cursor state
    fn get_cursor(&self) -> &Cursor;
    
    // ========== Search and Navigation ==========
    
    /// Search for text in the terminal content
    fn search(&self, pattern: &str, direction: SearchDirection) -> Vec<SearchMatch>;
    
    /// Scroll to a specific position
    fn scroll_to(&mut self, position: ScrollPosition) -> PaneResult<()>;
    
    /// Clear the screen
    fn clear_screen(&mut self) -> PaneResult<()>;
    
    /// Clear the scrollback buffer
    fn clear_scrollback(&mut self) -> PaneResult<()>;
    
    // ========== Configuration ==========
    
    /// Update the pane configuration
    fn update_config(&mut self, config: PaneConfig) -> PaneResult<()>;
    
    /// Get the current configuration
    fn get_config(&self) -> &PaneConfig;
    
    /// Apply a theme to the pane
    fn apply_theme(&mut self, theme: &Theme) -> PaneResult<()>;
    
    // ========== Event Handling ==========
    
    /// Register an event listener
    fn register_event_listener(&mut self, event_type: PaneEventType, listener: Box<dyn PaneEventListener>);
    
    /// Emit an event
    fn emit_event(&mut self, event: PaneEvent) -> PaneResult<()>;
    
    // ========== Statistics and Diagnostics ==========
    
    /// Get pane statistics
    fn get_statistics(&self) -> PaneStatistics;
    
    /// Validate the internal state
    fn validate_state(&self) -> PaneResult<()>;
    
    /// Force a refresh of the display
    fn refresh(&mut self) -> PaneResult<()>;
    
    /// Check if the pane needs rendering
    fn needs_render(&self) -> bool;
}

/// Implementation of PaneInterface for the Pane struct
impl PaneInterface for Pane {
    // ========== Identity and Basic State ==========
    
    fn id(&self) -> PaneId {
        self.id
    }
    
    fn is_active(&self) -> bool {
        self.active
    }
    
    fn set_active(&mut self, active: bool) {
        if self.active != active {
            self.active = active;
            let _ = self.emit_event(PaneEvent::ActiveStateChanged(active));
            
            // Update cursor visibility based on active state
            if active {
                self.cursor.blink_state.set_visible();
            }
        }
    }
    
    fn get_title(&self) -> &str {
        &self.title
    }
    
    fn set_title(&mut self, title: String) {
        if self.title != title {
            self.title = title.clone();
            let _ = self.emit_event(PaneEvent::TitleChanged(title));
        }
    }
    
    fn is_modified(&self) -> bool {
        self.modified
    }
    
    fn set_modified(&mut self, modified: bool) {
        if self.modified != modified {
            self.modified = modified;
            let _ = self.emit_event(PaneEvent::ModifiedStateChanged(modified));
        }
    }
    
    // ========== Process and PTY Management ==========
    
    fn spawn_process(&mut self, command: &str, args: &[&str], env: &[(String, String)]) -> PaneResult<()> {
        // Create PTY if not exists
        if self.pty.is_none() {
            self.pty = Some(PtyFactory::create());
        }
        
        if let Some(ref mut pty) = self.pty {
            pty.spawn(command, args, env)?;
            
            if let Some(pid) = pty.pid() {
                self.stats.record_spawn();
                let _ = self.emit_event(PaneEvent::ProcessSpawned(pid));
            }
            
            // Update title if it's still the default
            if self.title == self.config.default_title {
                let new_title = if args.is_empty() {
                    command.to_string()
                } else {
                    format!("{} {}", command, args.join(" "))
                };
                self.set_title(new_title);
            }
        }
        
        Ok(())
    }
    
    fn kill_process(&mut self) -> PaneResult<()> {
        if let Some(ref mut pty) = self.pty {
            pty.kill()?;
            let _ = self.emit_event(PaneEvent::ProcessKilled);
        }
        Ok(())
    }
    
    fn is_process_alive(&self) -> bool {
        self.pty.as_ref().map_or(false, |pty| pty.is_alive())
    }
    
    fn get_process_id(&self) -> Option<u32> {
        self.pty.as_ref().and_then(|pty| pty.pid())
    }
    
    // ========== Input/Output ==========
    
    fn write_input(&mut self, data: &[u8]) -> PaneResult<usize> {
        if let Some(ref mut pty) = self.pty {
            let bytes_written = pty.write(data)?;
            self.stats.bytes_sent += bytes_written;
            Ok(bytes_written)
        } else {
            Err(PaneError::ProcessError("No PTY available".to_string()))
        }
    }
    
    fn read_output(&mut self) -> PaneResult<Vec<u8>> {
        if let Some(ref mut pty) = self.pty {
            let data = pty.read()?;
            if !data.is_empty() {
                self.process_output(&data)?;
            }
            Ok(data)
        } else {
            Ok(Vec::new())
        }
    }
    
    fn process_output(&mut self, data: &[u8]) -> PaneResult<()> {
        self.handle_pty_data(data)?;
        Ok(())
    }
    
    // ========== Terminal State ==========
    
    fn resize(&mut self, rows: u16, cols: u16) -> PaneResult<()> {
        if rows == 0 || cols == 0 {
            return Err(PaneError::validation(
                format!("Invalid size: {}x{}", cols, rows)
            ));
        }
        
        // Resize PTY
        if let Some(ref mut pty) = self.pty {
            pty.resize(rows, cols)?;
        }
        
        // Resize screen buffer
        self.screen_buffer.resize(cols, rows)?;
        
        // Update tab stops for new width
        self.tabs = TabStops::new(cols);
        
        // Emit resize event
        let _ = self.emit_event(PaneEvent::Resized(cols, rows));
        
        Ok(())
    }
    
    fn get_size(&self) -> (u16, u16) {
        (self.screen_buffer.width, self.screen_buffer.height)
    }
    
    fn get_cursor_position(&self) -> (u16, u16) {
        (self.cursor.position.col, self.cursor.position.row)
    }
    
    fn get_terminal_mode(&self) -> TerminalMode {
        self.terminal.current_mode()
    }
    
    // ========== Content Access ==========
    
    fn get_line(&self, index: usize) -> Option<&Line> {
        self.screen_buffer.get_line(index)
    }
    
    fn get_screen_content(&self) -> &ScreenBuffer {
        &self.screen_buffer
    }
    
    fn get_scrollback(&self) -> &ScrollbackBuffer {
        &self.scrollback
    }
    
    fn get_cursor(&self) -> &Cursor {
        &self.cursor
    }
    
    // ========== Search and Navigation ==========
    
    fn search(&self, pattern: &str, direction: SearchDirection) -> Vec<SearchMatch> {
        let mut matches = Vec::new();
        
        // Search in scrollback
        let scrollback_matches = self.scrollback.search(pattern, true); // Case sensitive for now
        matches.extend(scrollback_matches);
        
        // Search in screen buffer
        for (line_idx, line) in self.screen_buffer.lines.iter().enumerate() {
            let text = line.text();
            let mut start = 0;
            
            while let Some(pos) = text[start..].find(pattern) {
                let match_start = start + pos;
                matches.push(SearchMatch {
                    buffer_type: BufferType::Screen,
                    line: line_idx,
                    start_col: match_start,
                    end_col: match_start + pattern.len(),
                    text: pattern.to_string(),
                });
                start = match_start + 1;
            }
        }
        
        // Sort matches by position
        matches.sort_by_key(|m| (m.buffer_type as u8, m.line, m.start_col));
        
        // Reverse for backward search
        if let SearchDirection::Backward = direction {
            matches.reverse();
        }
        
        matches
    }
    
    fn scroll_to(&mut self, position: ScrollPosition) -> PaneResult<()> {
        // TODO: Implement scrolling in the display/glazing layer
        // For now, this is a placeholder
        match position {
            ScrollPosition::Top => {
                // Scroll to top of scrollback
                Ok(())
            }
            ScrollPosition::Bottom => {
                // Scroll to bottom (current screen)
                Ok(())
            }
            ScrollPosition::Line(_line) => {
                // Scroll to specific line
                Ok(())
            }
            ScrollPosition::Relative(_offset) => {
                // Relative scroll
                Ok(())
            }
        }
    }
    
    fn clear_screen(&mut self) -> PaneResult<()> {
        self.screen_buffer.clear_screen(ClearType::All, &mut self.cursor)?;
        Ok(())
    }
    
    fn clear_scrollback(&mut self) -> PaneResult<()> {
        self.scrollback.clear();
        Ok(())
    }
    
    // ========== Configuration ==========
    
    fn update_config(&mut self, config: PaneConfig) -> PaneResult<()> {
        config.validate()?;
        
        // Apply size changes
        if config.initial_size != self.config.initial_size {
            self.resize(config.initial_size.1, config.initial_size.0)?;
        }
        
        // Apply scrollback changes
        if config.scrollback_lines != self.config.scrollback_lines {
            self.scrollback = ScrollbackBuffer::new(config.scrollback_lines);
        }
        
        // Apply cursor style changes
        if config.cursor_style != self.config.cursor_style {
            self.cursor.set_style(config.cursor_style);
        }
        
        // Apply visibility changes
        if config.show_cursor != self.config.show_cursor {
            let visibility = if config.show_cursor {
                if config.cursor_blink {
                    match config.cursor_style {
                        CursorStyle::Block => CursorVisibility::BlinkingBlock,
                        CursorStyle::Underline => CursorVisibility::BlinkingUnderline,
                        CursorStyle::Bar => CursorVisibility::BlinkingBar,
                    }
                } else {
                    CursorVisibility::Visible
                }
            } else {
                CursorVisibility::Hidden
            };
            self.cursor.set_visibility(visibility);
        }
        
        // Apply terminal mode changes
        self.modes.auto_wrap = config.auto_wrap;
        self.modes.cursor_visible = config.show_cursor;
        
        self.config = config;
        Ok(())
    }
    
    fn get_config(&self) -> &PaneConfig {
        &self.config
    }
    
    fn apply_theme(&mut self, theme: &Theme) -> PaneResult<()> {
        // Apply theme to all cells in screen buffer
        for line in &mut self.screen_buffer.lines {
            for cell in &mut line.cells {
                cell.apply_theme(theme);
            }
            line.dirty = true; // Mark line as needing re-render
        }
        
        // Apply theme to scrollback
        for line in &mut self.scrollback.lines {
            for cell in &mut line.cells {
                cell.apply_theme(theme);
            }
        }
        
        self.screen_buffer.mark_all_dirty();
        Ok(())
    }
    
    // ========== Event Handling ==========
    
    fn register_event_listener(&mut self, event_type: PaneEventType, listener: Box<dyn PaneEventListener>) {
        self.event_handler.register_listener(event_type, listener);
    }
    
    fn emit_event(&mut self, event: PaneEvent) -> PaneResult<()> {
        self.stats.record_activity();
        self.event_handler.dispatch(event)?;
        Ok(())
    }
    
    // ========== Statistics and Diagnostics ==========
    
    fn get_statistics(&self) -> PaneStatistics {
        let mut stats = self.stats.clone();
        
        // Update memory usage estimate
        stats.memory_usage = self.estimate_memory_usage();
        
        stats
    }
    
    fn validate_state(&self) -> PaneResult<()> {
        // Check cursor position is valid
        let (width, height) = self.get_size();
        if self.cursor.position.row >= height || self.cursor.position.col >= width {
            return Err(PaneError::state(
                format!("Cursor position ({}, {}) is invalid for size {}x{}",
                       self.cursor.position.col, self.cursor.position.row, width, height)
            ));
        }
        
        // Check screen buffer integrity
        if self.screen_buffer.lines.len() != height as usize {
            return Err(PaneError::state(
                format!("Screen buffer has {} lines but height is {}",
                       self.screen_buffer.lines.len(), height)
            ));
        }
        
        for (i, line) in self.screen_buffer.lines.iter().enumerate() {
            if line.cells.len() != width as usize {
                return Err(PaneError::state(
                    format!("Line {} has {} cells but width is {}",
                           i, line.cells.len(), width)
                ));
            }
        }
        
        Ok(())
    }
    
    fn refresh(&mut self) -> PaneResult<()> {
        self.screen_buffer.mark_all_dirty();
        Ok(())
    }
    
    fn needs_render(&self) -> bool {
        !self.screen_buffer.dirty_regions.dirty_lines.iter().all(|&dirty| !dirty) ||
        self.screen_buffer.dirty_regions.all_dirty
    }
}

impl Pane {
    /// Estimate memory usage of this pane
    fn estimate_memory_usage(&self) -> usize {
        let mut size = std::mem::size_of::<Pane>();
        
        // Screen buffer
        size += self.screen_buffer.lines.len() * std::mem::size_of::<Line>();
        for line in &self.screen_buffer.lines {
            size += line.cells.len() * std::mem::size_of::<Cell>();
        }
        
        // Scrollback buffer
        size += self.scrollback.lines.len() * std::mem::size_of::<Line>();
        for line in &self.scrollback.lines {
            size += line.cells.len() * std::mem::size_of::<Cell>();
        }
        
        // Other allocations
        size += self.title.len();
        size += self.config.environment_variables.len() * 64; // Estimate for env vars
        
        size
    }
}