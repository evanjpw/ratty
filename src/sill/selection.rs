use super::*;
use crate::pane::CursorPosition;
use crate::sash::PaneId;
use std::time::Instant;

/// Selection engine for text selection management
#[derive(Debug)]
pub struct SelectionEngine {
    config: SelectionConfig,
    current_selection: Option<Selection>,
    selection_history: SelectionHistory,
    focused_pane: Option<PaneId>,
    selection_count: u64,
}

impl SelectionEngine {
    pub fn new(config: &SelectionConfig) -> SillResult<Self> {
        Ok(SelectionEngine {
            config: config.clone(),
            current_selection: None,
            selection_history: SelectionHistory::new(config.history_size),
            focused_pane: None,
            selection_count: 0,
        })
    }
    
    /// Start a new text selection
    pub fn start_selection(
        &mut self,
        position: SelectionPosition,
        mode: SelectionMode,
        pane_id: Option<PaneId>,
    ) -> SillResult<()> {
        // Clear any existing selection
        self.clear_selection()?;
        
        let selection = Selection {
            start: position,
            end: position,
            mode,
            pane_id,
            timestamp: Instant::now(),
            active: true,
        };
        
        self.current_selection = Some(selection);
        self.focused_pane = pane_id;
        self.selection_count += 1;
        
        Ok(())
    }
    
    /// Update the current selection endpoint
    pub fn update_selection(&mut self, position: SelectionPosition) -> SillResult<()> {
        if let Some(ref mut selection) = self.current_selection {
            selection.end = position;
            selection.timestamp = Instant::now();
        } else {
            return Err(SillError::no_selection("No active selection to update"));
        }
        
        Ok(())
    }
    
    /// End the current selection
    pub fn end_selection(&mut self) -> SillResult<Option<Selection>> {
        if let Some(mut selection) = self.current_selection.take() {
            selection.active = false;
            
            // Add to history if it's a valid selection
            if !selection.is_empty() {
                self.selection_history.add_selection(selection.clone());
            }
            
            Ok(Some(selection))
        } else {
            Ok(None)
        }
    }
    
    /// Clear the current selection
    pub fn clear_selection(&mut self) -> SillResult<()> {
        self.current_selection = None;
        Ok(())
    }
    
    /// Get the current selection
    pub fn get_current_selection(&self) -> Option<&Selection> {
        self.current_selection.as_ref()
    }
    
    /// Handle mouse event for selection
    pub fn handle_mouse_event(&mut self, event: &NormalizedMouseEvent) -> SillResult<()> {
        let position = SelectionPosition {
            row: event.position.row,
            col: event.position.col,
        };
        
        match event.event_type {
            MouseEventType::Press => {
                let mode = match event.click_count {
                    1 => SelectionMode::Character,
                    2 => SelectionMode::Word,
                    3 => SelectionMode::Line,
                    _ => SelectionMode::Character,
                };
                
                self.start_selection(position, mode, self.focused_pane)?;
            }
            MouseEventType::Drag => {
                if self.current_selection.is_some() {
                    self.update_selection(position)?;
                }
            }
            MouseEventType::Release => {
                if self.current_selection.is_some() {
                    self.end_selection()?;
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Select all text in the current pane
    pub fn select_all(&mut self, pane_id: Option<PaneId>) -> SillResult<()> {
        // This would need to get the actual pane dimensions
        let start = SelectionPosition { row: 0, col: 0 };
        let end = SelectionPosition { row: u16::MAX, col: u16::MAX };
        
        let selection = Selection {
            start,
            end,
            mode: SelectionMode::All,
            pane_id,
            timestamp: Instant::now(),
            active: false,
        };
        
        self.current_selection = Some(selection);
        self.selection_count += 1;
        
        Ok(())
    }
    
    /// Select word at position
    pub fn select_word_at(&mut self, position: SelectionPosition, pane_id: Option<PaneId>) -> SillResult<()> {
        // This would need to access the pane content to find word boundaries
        let start = position; // TODO: Find word start
        let end = position;   // TODO: Find word end
        
        let selection = Selection {
            start,
            end,
            mode: SelectionMode::Word,
            pane_id,
            timestamp: Instant::now(),
            active: false,
        };
        
        self.current_selection = Some(selection);
        self.selection_count += 1;
        
        Ok(())
    }
    
    /// Select line at position
    pub fn select_line_at(&mut self, position: SelectionPosition, pane_id: Option<PaneId>) -> SillResult<()> {
        let start = SelectionPosition { row: position.row, col: 0 };
        let end = SelectionPosition { row: position.row, col: u16::MAX };
        
        let selection = Selection {
            start,
            end,
            mode: SelectionMode::Line,
            pane_id,
            timestamp: Instant::now(),
            active: false,
        };
        
        self.current_selection = Some(selection);
        self.selection_count += 1;
        
        Ok(())
    }
    
    /// Get selection history
    pub fn get_selection_history(&self) -> &SelectionHistory {
        &self.selection_history
    }
    
    /// Get focused pane
    pub fn get_focused_pane(&self) -> Option<PaneId> {
        self.focused_pane
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: &SelectionConfig) -> SillResult<()> {
        self.config = config.clone();
        self.selection_history.set_max_size(config.history_size);
        Ok(())
    }
    
    /// Get selection count
    pub fn get_selection_count(&self) -> u64 {
        self.selection_count
    }
}

/// Text selection representation
#[derive(Debug, Clone)]
pub struct Selection {
    pub start: SelectionPosition,
    pub end: SelectionPosition,
    pub mode: SelectionMode,
    pub pane_id: Option<PaneId>,
    pub timestamp: Instant,
    pub active: bool,
}

impl Selection {
    /// Check if selection is empty
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
    
    /// Get the bounds of the selection (normalized start/end)
    pub fn bounds(&self) -> (SelectionPosition, SelectionPosition) {
        if self.start <= self.end {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }
    
    /// Check if a position is within the selection
    pub fn contains(&self, position: SelectionPosition) -> bool {
        let (start, end) = self.bounds();
        position >= start && position <= end
    }
    
    /// Get selected text (placeholder - would need pane access)
    pub fn get_text(&self) -> SillResult<String> {
        // This would need to access the actual pane content
        // For now, return a placeholder
        Ok(format!("Selected text from {:?} to {:?}", self.start, self.end))
    }
    
    /// Get selection length in characters
    pub fn length(&self) -> usize {
        let (start, end) = self.bounds();
        
        match self.mode {
            SelectionMode::Character => {
                if start.row == end.row {
                    (end.col - start.col) as usize
                } else {
                    // Multi-line selection - approximate
                    ((end.row - start.row) as usize * 80) + (end.col as usize)
                }
            }
            SelectionMode::Word => {
                // Word count - approximate
                self.length() / 5
            }
            SelectionMode::Line => {
                (end.row - start.row + 1) as usize
            }
            SelectionMode::Block => {
                ((end.row - start.row + 1) * (end.col - start.col + 1)) as usize
            }
            SelectionMode::All => {
                // Entire pane - approximate
                80 * 24
            }
        }
    }
}

/// Position within a selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SelectionPosition {
    pub row: u16,
    pub col: u16,
}

impl From<CursorPosition> for SelectionPosition {
    fn from(cursor_pos: CursorPosition) -> Self {
        SelectionPosition {
            row: cursor_pos.row,
            col: cursor_pos.col,
        }
    }
}

impl From<MousePosition> for SelectionPosition {
    fn from(mouse_pos: MousePosition) -> Self {
        SelectionPosition {
            row: mouse_pos.row,
            col: mouse_pos.col,
        }
    }
}

/// Selection modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelectionMode {
    /// Character-by-character selection
    Character,
    /// Word-by-word selection
    Word,
    /// Line-by-line selection
    Line,
    /// Block/rectangular selection
    Block,
    /// Select all content
    All,
}

impl Default for SelectionMode {
    fn default() -> Self {
        SelectionMode::Character
    }
}

/// Selection history management
#[derive(Debug)]
pub struct SelectionHistory {
    selections: Vec<Selection>,
    max_size: usize,
    current_index: usize,
}

impl SelectionHistory {
    pub fn new(max_size: usize) -> Self {
        SelectionHistory {
            selections: Vec::new(),
            max_size,
            current_index: 0,
        }
    }
    
    /// Add a selection to history
    pub fn add_selection(&mut self, selection: Selection) {
        // Don't add empty selections to history
        if selection.is_empty() {
            return;
        }
        
        self.selections.push(selection);
        
        // Keep only the most recent selections
        if self.selections.len() > self.max_size {
            self.selections.remove(0);
        }
        
        self.current_index = self.selections.len();
    }
    
    /// Get the most recent selection
    pub fn get_latest(&self) -> Option<&Selection> {
        self.selections.last()
    }
    
    /// Get selection by index (0 = oldest)
    pub fn get(&self, index: usize) -> Option<&Selection> {
        self.selections.get(index)
    }
    
    /// Get all selections
    pub fn get_all(&self) -> &[Selection] {
        &self.selections
    }
    
    /// Clear selection history
    pub fn clear(&mut self) {
        self.selections.clear();
        self.current_index = 0;
    }
    
    /// Set maximum history size
    pub fn set_max_size(&mut self, max_size: usize) {
        self.max_size = max_size;
        
        // Trim if necessary
        if self.selections.len() > max_size {
            let excess = self.selections.len() - max_size;
            self.selections.drain(0..excess);
            self.current_index = self.current_index.saturating_sub(excess);
        }
    }
    
    /// Get history size
    pub fn len(&self) -> usize {
        self.selections.len()
    }
    
    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.selections.is_empty()
    }
}

/// Selection configuration
#[derive(Debug, Clone)]
pub struct SelectionConfig {
    pub enabled: bool,
    pub history_size: usize,
    pub word_separators: String,
    pub line_separators: String,
    pub auto_copy: bool,
    pub trim_whitespace: bool,
    pub block_selection_key: Option<Key>,
    pub extend_selection_key: Option<Key>,
}

impl Default for SelectionConfig {
    fn default() -> Self {
        SelectionConfig {
            enabled: true,
            history_size: 100,
            word_separators: " \t\n\r.,;:!?()[]{}\"'".to_string(),
            line_separators: "\n\r".to_string(),
            auto_copy: false,
            trim_whitespace: true,
            block_selection_key: Some(Key::Alt),
            extend_selection_key: Some(Key::Shift),
        }
    }
}

/// Selection renderer for highlighting selected text
#[derive(Debug)]
pub struct SelectionRenderer {
    config: SelectionRenderConfig,
}

impl SelectionRenderer {
    pub fn new(config: SelectionRenderConfig) -> Self {
        SelectionRenderer { config }
    }
    
    /// Check if a position should be highlighted as selected
    pub fn is_position_selected(&self, position: SelectionPosition, selection: &Selection) -> bool {
        if !selection.active && !self.config.show_inactive_selections {
            return false;
        }
        
        selection.contains(position)
    }
    
    /// Get selection highlight style
    pub fn get_selection_style(&self) -> SelectionStyle {
        self.config.style.clone()
    }
    
    /// Update renderer configuration
    pub fn update_config(&mut self, config: SelectionRenderConfig) {
        self.config = config;
    }
}

/// Selection rendering configuration
#[derive(Debug, Clone)]
pub struct SelectionRenderConfig {
    pub style: SelectionStyle,
    pub show_inactive_selections: bool,
    pub blink_inactive: bool,
    pub fade_after: Option<Duration>,
}

impl Default for SelectionRenderConfig {
    fn default() -> Self {
        SelectionRenderConfig {
            style: SelectionStyle::default(),
            show_inactive_selections: true,
            blink_inactive: false,
            fade_after: Some(Duration::from_secs(5)),
        }
    }
}

/// Selection visual style
#[derive(Debug, Clone)]
pub struct SelectionStyle {
    pub background_color: crate::sash::Color,
    pub foreground_color: Option<crate::sash::Color>,
    pub opacity: f32,
    pub border: bool,
    pub border_color: Option<crate::sash::Color>,
}

impl Default for SelectionStyle {
    fn default() -> Self {
        SelectionStyle {
            background_color: crate::sash::Color::from_rgb(100, 150, 200),
            foreground_color: None, // Use default text color
            opacity: 0.3,
            border: false,
            border_color: None,
        }
    }
}