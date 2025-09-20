pub mod buffer;
pub mod cursor;
pub mod terminal;
pub mod pty;
pub mod interface;
pub mod events;
pub mod errors;
pub mod config;

#[cfg(test)]
mod tests;

pub use buffer::*;
pub use config::*;
pub use cursor::*;
pub use errors::*;
pub use events::*;
pub use interface::*;
pub use pty::*;
pub use terminal::*;

use crate::sash::{PaneId, Theme};
use std::collections::VecDeque;

/// The main Pane structure - represents a single terminal instance
pub struct Pane {
    // Identity and metadata
    id: PaneId,
    title: String,
    modified: bool,
    active: bool,
    
    // Terminal emulation core
    terminal: Terminal,
    pty: Option<Box<dyn PtyInterface>>,
    
    // Text content management
    screen_buffer: ScreenBuffer,
    scrollback: ScrollbackBuffer,
    cursor: Cursor,
    
    // Terminal state
    modes: TerminalModes,
    character_sets: CharacterSets,
    tabs: TabStops,
    
    // Configuration and theming
    config: PaneConfig,
    local_theme_override: Option<Theme>,
    
    // Event handling
    event_handler: PaneEventHandler,
    
    // Statistics and diagnostics
    stats: PaneStatistics,
}

impl Pane {
    /// Create a new Pane with default configuration
    pub fn new(id: PaneId, config: PaneConfig) -> PaneResult<Self> {
        let size = config.initial_size;
        
        Ok(Pane {
            id,
            title: config.default_title.clone(),
            modified: false,
            active: false,
            
            terminal: Terminal::new()?,
            pty: None,
            
            screen_buffer: ScreenBuffer::new(size.0, size.1),
            scrollback: ScrollbackBuffer::new(config.scrollback_lines),
            cursor: Cursor::new(),
            
            modes: TerminalModes::default(),
            character_sets: CharacterSets::default(),
            tabs: TabStops::new(size.0),
            
            config,
            local_theme_override: None,
            
            event_handler: PaneEventHandler::new(),
            
            stats: PaneStatistics::default(),
        })
    }
    
    /// Create a new Pane with a specific command
    pub fn with_command(id: PaneId, config: PaneConfig, command: &str, args: &[&str]) -> PaneResult<Self> {
        let mut pane = Self::new(id, config)?;
        pane.spawn_process(command, args, &[])?;
        Ok(pane)
    }
    
    /// Get the current screen dimensions
    pub fn size(&self) -> (u16, u16) {
        (self.screen_buffer.width, self.screen_buffer.height)
    }
    
    /// Check if the pane has an active process
    pub fn has_process(&self) -> bool {
        self.pty.is_some() && self.is_process_alive()
    }
    
    /// Get the working directory if available
    pub fn working_directory(&self) -> Option<&str> {
        self.config.working_directory.as_deref()
    }
    
    /// Update the pane's activity state and emit events if needed
    fn update_activity(&mut self) {
        if !self.modified {
            self.modified = true;
            let _ = self.emit_event(PaneEvent::ContentChanged(events::ContentRegion::Screen));
        }
    }
    
    /// Process incoming data from the PTY
    fn handle_pty_data(&mut self, data: &[u8]) -> PaneResult<()> {
        self.stats.bytes_received += data.len();
        
        // Parse and process the data through the terminal emulator
        for byte in data {
            if let Some(command) = self.terminal.process_byte(*byte)? {
                self.execute_terminal_command(command)?;
            }
        }
        
        self.update_activity();
        Ok(())
    }
    
    /// Execute a terminal command (from VT parser)
    fn execute_terminal_command(&mut self, command: VtCommand) -> PaneResult<()> {
        match command {
            VtCommand::PrintChar(ch) => {
                self.screen_buffer.write_char_at_cursor(ch, &self.cursor, &self.modes)?;
                self.cursor.advance(&self.screen_buffer, &self.modes)?;
            }
            VtCommand::CursorUp(n) => {
                self.cursor.move_up(n, &self.screen_buffer)?;
            }
            VtCommand::CursorDown(n) => {
                self.cursor.move_down(n, &self.screen_buffer)?;
            }
            VtCommand::CursorForward(n) => {
                self.cursor.move_forward(n, &self.screen_buffer)?;
            }
            VtCommand::CursorBack(n) => {
                self.cursor.move_back(n)?;
            }
            VtCommand::CursorPosition(row, col) => {
                self.cursor.set_position(row, col, &self.screen_buffer)?;
            }
            VtCommand::ClearScreen(clear_type) => {
                self.screen_buffer.clear_screen(clear_type, &mut self.cursor)?;
            }
            VtCommand::ClearLine(clear_type) => {
                self.screen_buffer.clear_line(clear_type, &self.cursor)?;
            }
            VtCommand::LineFeed => {
                self.cursor.line_feed(&mut self.screen_buffer, &mut self.scrollback, &self.modes)?;
            }
            VtCommand::CarriageReturn => {
                self.cursor.carriage_return()?;
            }
            VtCommand::Tab => {
                self.cursor.tab_forward(&self.tabs, &self.screen_buffer)?;
            }
            VtCommand::Backspace => {
                self.cursor.backspace(&self.screen_buffer)?;
            }
            VtCommand::SetGraphicsRendition(params) => {
                self.modes.set_graphics_attributes(&params)?;
            }
            // TODO: Implement remaining commands
            _ => {
                // For now, just track unimplemented commands
                self.stats.unhandled_sequences += 1;
            }
        }
        
        Ok(())
    }
}

/// Terminal mode flags and state
#[derive(Debug, Clone)]
pub struct TerminalModes {
    pub insert_mode: bool,
    pub auto_wrap: bool,
    pub cursor_visible: bool,
    pub application_keypad: bool,
    pub application_cursor: bool,
    pub origin_mode: bool,
    pub current_attributes: CellAttributes,
}

impl Default for TerminalModes {
    fn default() -> Self {
        TerminalModes {
            insert_mode: false,
            auto_wrap: true,
            cursor_visible: true,
            application_keypad: false,
            application_cursor: false,
            origin_mode: false,
            current_attributes: CellAttributes::default(),
        }
    }
}

impl TerminalModes {
    /// Set graphics rendition attributes from VT sequence parameters
    pub fn set_graphics_attributes(&mut self, params: &[u8]) -> PaneResult<()> {
        for &param in params {
            match param {
                0 => self.current_attributes = CellAttributes::default(),
                1 => self.current_attributes.bold = true,
                2 => self.current_attributes.dim = true,
                3 => self.current_attributes.italic = true,
                4 => self.current_attributes.underline = UnderlineType::Single,
                5 | 6 => self.current_attributes.blink = BlinkType::Slow,
                7 => self.current_attributes.reverse = true,
                8 => self.current_attributes.invisible = true,
                9 => self.current_attributes.strikethrough = true,
                21 => self.current_attributes.underline = UnderlineType::Double,
                22 => { self.current_attributes.bold = false; self.current_attributes.dim = false; },
                23 => self.current_attributes.italic = false,
                24 => self.current_attributes.underline = UnderlineType::None,
                25 => self.current_attributes.blink = BlinkType::None,
                27 => self.current_attributes.reverse = false,
                28 => self.current_attributes.invisible = false,
                29 => self.current_attributes.strikethrough = false,
                // TODO: Implement color codes (30-37, 40-47, 90-97, 100-107)
                _ => {} // Ignore unknown parameters
            }
        }
        Ok(())
    }
}

/// Character set handling for terminal emulation
#[derive(Debug, Clone)]
pub struct CharacterSets {
    pub g0: CharacterSet,
    pub g1: CharacterSet,
    pub active: CharacterSetSlot,
}

#[derive(Debug, Clone, Copy)]
pub enum CharacterSet {
    Ascii,
    DecSpecialCharacter,
    DecAlternateCharacter,
    DecAlternateRom,
}

#[derive(Debug, Clone, Copy)]
pub enum CharacterSetSlot {
    G0,
    G1,
}

impl Default for CharacterSets {
    fn default() -> Self {
        CharacterSets {
            g0: CharacterSet::Ascii,
            g1: CharacterSet::Ascii,
            active: CharacterSetSlot::G0,
        }
    }
}

/// Tab stop management
#[derive(Debug, Clone)]
pub struct TabStops {
    stops: Vec<bool>,
    default_width: u16,
}

impl TabStops {
    pub fn new(width: u16) -> Self {
        let mut stops = vec![false; width as usize];
        // Set default tab stops every 8 columns
        for i in (0..width).step_by(8) {
            if let Some(stop) = stops.get_mut(i as usize) {
                *stop = true;
            }
        }
        
        TabStops {
            stops,
            default_width: 8,
        }
    }
    
    /// Find the next tab stop from the given column
    pub fn next_tab_stop(&self, from_col: u16) -> u16 {
        for col in (from_col + 1)..(self.stops.len() as u16) {
            if self.stops[col as usize] {
                return col;
            }
        }
        // If no tab stop found, go to the last column
        (self.stops.len() as u16).saturating_sub(1)
    }
    
    /// Set a tab stop at the given column
    pub fn set_tab_stop(&mut self, col: u16) {
        if let Some(stop) = self.stops.get_mut(col as usize) {
            *stop = true;
        }
    }
    
    /// Clear a tab stop at the given column
    pub fn clear_tab_stop(&mut self, col: u16) {
        if let Some(stop) = self.stops.get_mut(col as usize) {
            *stop = false;
        }
    }
    
    /// Clear all tab stops
    pub fn clear_all(&mut self) {
        for stop in &mut self.stops {
            *stop = false;
        }
    }
    
    /// Reset to default tab stops (every 8 columns)
    pub fn reset_defaults(&mut self) {
        self.clear_all();
        for i in (0..self.stops.len()).step_by(8) {
            self.stops[i] = true;
        }
    }
}