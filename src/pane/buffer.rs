use super::*;
use std::collections::VecDeque;
use crate::sash::{Color, Theme};

/// Screen buffer containing the currently visible terminal content
#[derive(Debug, Clone)]
pub struct ScreenBuffer {
    pub lines: Vec<Line>,
    pub width: u16,
    pub height: u16,
    pub dirty_regions: DirtyTracker,
}

impl ScreenBuffer {
    /// Create a new screen buffer with the given dimensions
    pub fn new(width: u16, height: u16) -> Self {
        let mut lines = Vec::with_capacity(height as usize);
        for _ in 0..height {
            lines.push(Line::new(width));
        }
        
        ScreenBuffer {
            lines,
            width,
            height,
            dirty_regions: DirtyTracker::new(width, height),
        }
    }
    
    /// Resize the screen buffer
    pub fn resize(&mut self, new_width: u16, new_height: u16) -> PaneResult<()> {
        let old_height = self.height;
        
        // Adjust line count
        if new_height > old_height {
            // Add new lines at the bottom
            for _ in old_height..new_height {
                self.lines.push(Line::new(new_width));
            }
        } else if new_height < old_height {
            // Remove lines from the bottom
            self.lines.truncate(new_height as usize);
        }
        
        // Resize all existing lines
        for line in &mut self.lines {
            line.resize(new_width);
        }
        
        self.width = new_width;
        self.height = new_height;
        self.dirty_regions = DirtyTracker::new(new_width, new_height);
        self.mark_all_dirty();
        
        Ok(())
    }
    
    /// Write a character at the cursor position
    pub fn write_char_at_cursor(&mut self, ch: char, cursor: &Cursor, modes: &TerminalModes) -> PaneResult<()> {
        let pos = cursor.position;
        self.write_char_at(pos.row, pos.col, ch, &modes.current_attributes)?;
        Ok(())
    }
    
    /// Write a character at a specific position
    pub fn write_char_at(&mut self, row: u16, col: u16, ch: char, attrs: &CellAttributes) -> PaneResult<()> {
        if row >= self.height {
            return Err(PaneError::InvalidCursorPosition(row, col));
        }
        
        let line = &mut self.lines[row as usize];
        if col < self.width {
            line.write_char(col, ch, attrs.clone());
            self.dirty_regions.mark_cell_dirty(row, col);
        }
        
        Ok(())
    }
    
    /// Clear the entire screen
    pub fn clear_screen(&mut self, clear_type: ClearType, cursor: &mut Cursor) -> PaneResult<()> {
        match clear_type {
            ClearType::All => {
                for line in &mut self.lines {
                    line.clear();
                }
                cursor.set_position(0, 0, self)?;
            }
            ClearType::ToEnd => {
                // Clear from cursor to end of screen
                let cursor_row = cursor.position.row as usize;
                let cursor_col = cursor.position.col;
                
                // Clear rest of current line
                if cursor_row < self.lines.len() {
                    self.lines[cursor_row].clear_from(cursor_col);
                }
                
                // Clear all lines below
                for line in &mut self.lines[(cursor_row + 1)..] {
                    line.clear();
                }
            }
            ClearType::ToBeginning => {
                // Clear from beginning to cursor
                let cursor_row = cursor.position.row as usize;
                let cursor_col = cursor.position.col;
                
                // Clear all lines above
                for line in &mut self.lines[..cursor_row] {
                    line.clear();
                }
                
                // Clear beginning of current line
                if cursor_row < self.lines.len() {
                    self.lines[cursor_row].clear_to(cursor_col);
                }
            }
        }
        
        self.mark_all_dirty();
        Ok(())
    }
    
    /// Clear a line
    pub fn clear_line(&mut self, clear_type: ClearType, cursor: &Cursor) -> PaneResult<()> {
        let row = cursor.position.row as usize;
        if row >= self.lines.len() {
            return Ok(()); // Out of bounds, nothing to clear
        }
        
        let line = &mut self.lines[row];
        match clear_type {
            ClearType::All => line.clear(),
            ClearType::ToEnd => line.clear_from(cursor.position.col),
            ClearType::ToBeginning => line.clear_to(cursor.position.col),
        }
        
        self.dirty_regions.mark_line_dirty(cursor.position.row);
        Ok(())
    }
    
    /// Insert blank lines at the cursor position
    pub fn insert_lines(&mut self, count: u16, cursor: &Cursor) -> PaneResult<()> {
        let start_row = cursor.position.row as usize;
        if start_row >= self.lines.len() {
            return Ok(());
        }
        
        // Remove lines from the bottom and insert blank lines at cursor
        for _ in 0..count.min(self.height - cursor.position.row) {
            if self.lines.len() > start_row {
                self.lines.remove(self.lines.len() - 1);
                self.lines.insert(start_row, Line::new(self.width));
            }
        }
        
        self.mark_all_dirty();
        Ok(())
    }
    
    /// Delete lines at the cursor position
    pub fn delete_lines(&mut self, count: u16, cursor: &Cursor) -> PaneResult<()> {
        let start_row = cursor.position.row as usize;
        if start_row >= self.lines.len() {
            return Ok(());
        }
        
        // Remove lines at cursor position and add blank lines at bottom
        for _ in 0..count.min(self.height - cursor.position.row) {
            if start_row < self.lines.len() {
                self.lines.remove(start_row);
                self.lines.push(Line::new(self.width));
            }
        }
        
        self.mark_all_dirty();
        Ok(())
    }
    
    /// Get a line by index
    pub fn get_line(&self, index: usize) -> Option<&Line> {
        self.lines.get(index)
    }
    
    /// Get a mutable line by index
    pub fn get_line_mut(&mut self, index: usize) -> Option<&mut Line> {
        self.lines.get_mut(index)
    }
    
    /// Mark the entire screen as dirty
    pub fn mark_all_dirty(&mut self) {
        self.dirty_regions.mark_all_dirty();
    }
    
    /// Get dirty regions for rendering
    pub fn get_dirty_regions(&self) -> &DirtyTracker {
        &self.dirty_regions
    }
    
    /// Clear dirty regions (called after rendering)
    pub fn clear_dirty(&mut self) {
        self.dirty_regions.clear();
    }
}

/// A single line of terminal content
#[derive(Debug, Clone)]
pub struct Line {
    pub cells: Vec<Cell>,
    pub wrapped: bool,
    pub dirty: bool,
    pub timestamp: Option<std::time::Instant>,
}

impl Line {
    /// Create a new line with the given width
    pub fn new(width: u16) -> Self {
        Line {
            cells: vec![Cell::default(); width as usize],
            wrapped: false,
            dirty: true,
            timestamp: Some(std::time::Instant::now()),
        }
    }
    
    /// Resize the line
    pub fn resize(&mut self, new_width: u16) {
        let old_len = self.cells.len();
        self.cells.resize(new_width as usize, Cell::default());
        
        // Mark as dirty if size changed
        if new_width as usize != old_len {
            self.dirty = true;
            self.timestamp = Some(std::time::Instant::now());
        }
    }
    
    /// Write a character at the specified column
    pub fn write_char(&mut self, col: u16, ch: char, attrs: CellAttributes) {
        if let Some(cell) = self.cells.get_mut(col as usize) {
            cell.character = ch;
            cell.attributes = attrs;
            // TODO: Set colors from current attributes/theme
            self.dirty = true;
            self.timestamp = Some(std::time::Instant::now());
        }
    }
    
    /// Clear the entire line
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            *cell = Cell::default();
        }
        self.dirty = true;
        self.timestamp = Some(std::time::Instant::now());
    }
    
    /// Clear from the specified column to the end of the line
    pub fn clear_from(&mut self, from_col: u16) {
        if let Some(cells_to_clear) = self.cells.get_mut(from_col as usize..) {
            for cell in cells_to_clear {
                *cell = Cell::default();
            }
            self.dirty = true;
            self.timestamp = Some(std::time::Instant::now());
        }
    }
    
    /// Clear from the beginning of the line to the specified column (inclusive)
    pub fn clear_to(&mut self, to_col: u16) {
        let end_idx = (to_col + 1).min(self.cells.len() as u16) as usize;
        if let Some(cells_to_clear) = self.cells.get_mut(..end_idx) {
            for cell in cells_to_clear {
                *cell = Cell::default();
            }
            self.dirty = true;
            self.timestamp = Some(std::time::Instant::now());
        }
    }
    
    /// Get the text content of the line
    pub fn text(&self) -> String {
        self.cells.iter().map(|cell| cell.character).collect()
    }
    
    /// Check if the line is empty (all default cells)
    pub fn is_empty(&self) -> bool {
        self.cells.iter().all(|cell| cell.is_default())
    }
}

/// A single character cell in the terminal
#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    pub character: char,
    pub attributes: CellAttributes,
    pub foreground: Color,
    pub background: Color,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            character: ' ',
            attributes: CellAttributes::default(),
            foreground: Color::from_rgb(255, 255, 255), // Default white
            background: Color::from_rgb(0, 0, 0),       // Default black
        }
    }
}

impl Cell {
    /// Check if this cell is in default state
    pub fn is_default(&self) -> bool {
        self.character == ' ' && 
        self.attributes == CellAttributes::default() &&
        self.foreground == Color::from_rgb(255, 255, 255) &&
        self.background == Color::from_rgb(0, 0, 0)
    }
    
    /// Apply a theme to this cell's colors
    pub fn apply_theme(&mut self, theme: &Theme) {
        if self.foreground == Color::from_rgb(255, 255, 255) {
            self.foreground = theme.colors.foreground;
        }
        if self.background == Color::from_rgb(0, 0, 0) {
            self.background = theme.colors.background;
        }
    }
}

/// Cell attributes for text formatting
#[derive(Debug, Clone, PartialEq)]
pub struct CellAttributes {
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: UnderlineType,
    pub strikethrough: bool,
    pub reverse: bool,
    pub blink: BlinkType,
    pub invisible: bool,
}

impl Default for CellAttributes {
    fn default() -> Self {
        CellAttributes {
            bold: false,
            dim: false,
            italic: false,
            underline: UnderlineType::None,
            strikethrough: false,
            reverse: false,
            blink: BlinkType::None,
            invisible: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnderlineType {
    None,
    Single,
    Double,
    Curly,
    Dotted,
    Dashed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlinkType {
    None,
    Slow,
    Fast,
}

/// Scrollback buffer for storing terminal history
#[derive(Debug, Clone)]
pub struct ScrollbackBuffer {
    pub lines: VecDeque<Line>,
    pub max_lines: usize,
    pub current_size: usize,
}

impl ScrollbackBuffer {
    /// Create a new scrollback buffer
    pub fn new(max_lines: usize) -> Self {
        ScrollbackBuffer {
            lines: VecDeque::with_capacity(max_lines),
            max_lines,
            current_size: 0,
        }
    }
    
    /// Add a line to the scrollback buffer
    pub fn push_line(&mut self, line: Line) {
        if self.current_size >= self.max_lines {
            self.lines.pop_front();
        } else {
            self.current_size += 1;
        }
        self.lines.push_back(line);
    }
    
    /// Get a line from the scrollback buffer
    pub fn get_line(&self, index: usize) -> Option<&Line> {
        self.lines.get(index)
    }
    
    /// Get the total number of lines in scrollback
    pub fn len(&self) -> usize {
        self.lines.len()
    }
    
    /// Check if scrollback is empty
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
    
    /// Clear the scrollback buffer
    pub fn clear(&mut self) {
        self.lines.clear();
        self.current_size = 0;
    }
    
    /// Search for a pattern in the scrollback buffer
    pub fn search(&self, pattern: &str, case_sensitive: bool) -> Vec<SearchMatch> {
        let mut matches = Vec::new();
        
        for (line_idx, line) in self.lines.iter().enumerate() {
            let text = line.text();
            let search_text = if case_sensitive { text } else { text.to_lowercase() };
            let search_pattern = if case_sensitive { pattern.to_string() } else { pattern.to_lowercase() };
            
            let mut start = 0;
            while let Some(pos) = search_text[start..].find(&search_pattern) {
                let match_start = start + pos;
                matches.push(SearchMatch {
                    buffer_type: BufferType::Scrollback,
                    line: line_idx,
                    start_col: match_start,
                    end_col: match_start + pattern.len(),
                    text: pattern.to_string(),
                });
                start = match_start + 1;
            }
        }
        
        matches
    }
}

/// Clear type for screen and line clearing operations
#[derive(Debug, Clone, Copy)]
pub enum ClearType {
    All,
    ToEnd,
    ToBeginning,
}

/// Track dirty regions for efficient rendering
#[derive(Debug, Clone)]
pub struct DirtyTracker {
    pub width: u16,
    pub height: u16,
    pub dirty_lines: Vec<bool>,
    pub all_dirty: bool,
}

impl DirtyTracker {
    pub fn new(width: u16, height: u16) -> Self {
        DirtyTracker {
            width,
            height,
            dirty_lines: vec![false; height as usize],
            all_dirty: true, // Initially everything is dirty
        }
    }
    
    pub fn mark_cell_dirty(&mut self, row: u16, _col: u16) {
        if let Some(line_dirty) = self.dirty_lines.get_mut(row as usize) {
            *line_dirty = true;
        }
    }
    
    pub fn mark_line_dirty(&mut self, row: u16) {
        if let Some(line_dirty) = self.dirty_lines.get_mut(row as usize) {
            *line_dirty = true;
        }
    }
    
    pub fn mark_all_dirty(&mut self) {
        self.all_dirty = true;
        for dirty in &mut self.dirty_lines {
            *dirty = true;
        }
    }
    
    pub fn clear(&mut self) {
        self.all_dirty = false;
        for dirty in &mut self.dirty_lines {
            *dirty = false;
        }
    }
    
    pub fn is_line_dirty(&self, row: u16) -> bool {
        self.all_dirty || self.dirty_lines.get(row as usize).copied().unwrap_or(false)
    }
    
    pub fn is_all_dirty(&self) -> bool {
        self.all_dirty
    }
}

/// Search match result
#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub buffer_type: BufferType,
    pub line: usize,
    pub start_col: usize,
    pub end_col: usize,
    pub text: String,
}

#[derive(Debug, Clone, Copy)]
pub enum BufferType {
    Screen,
    Scrollback,
}

// ContentRegion moved to events.rs to avoid duplication