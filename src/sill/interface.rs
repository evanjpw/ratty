use super::*;
use crate::sash::PaneId;

/// Main interface trait for the Sill layer
pub trait SillInterface: Send + Sync {
    /// Process a keyboard event
    fn process_key_event(&mut self, event: RawKeyEvent) -> SillResult<Vec<InputCommand>>;
    
    /// Process a mouse event
    fn process_mouse_event(&mut self, event: RawMouseEvent) -> SillResult<Vec<InputCommand>>;
    
    /// Handle clipboard operations
    fn clipboard_copy(&mut self) -> SillResult<String>;
    fn clipboard_paste(&mut self) -> SillResult<String>;
    
    /// Get current selection
    fn get_selection(&self) -> Option<&Selection>;
    
    /// Set input mode
    fn set_input_mode(&mut self, mode: InputMode) -> SillResult<()>;
    
    /// Set focus to a specific pane
    fn set_focus(&mut self, pane_id: Option<PaneId>) -> SillResult<()>;
    
    /// Start text selection
    fn start_selection(&mut self, position: SelectionPosition, mode: SelectionMode) -> SillResult<()>;
    
    /// Update selection
    fn update_selection(&mut self, position: SelectionPosition) -> SillResult<()>;
    
    /// End selection
    fn end_selection(&mut self) -> SillResult<Option<Selection>>;
    
    /// Clear selection
    fn clear_selection(&mut self) -> SillResult<()>;
    
    /// Configure key mappings
    fn set_key_mapping(&mut self, mapping: KeyMapping) -> SillResult<()>;
    
    /// Get performance metrics
    fn get_performance_metrics(&self) -> &InputPerformanceTracker;
    
    /// Get input statistics
    fn get_input_statistics(&self) -> InputStatistics;
}

impl SillInterface for SillEngine {
    fn process_key_event(&mut self, event: RawKeyEvent) -> SillResult<Vec<InputCommand>> {
        SillEngine::process_key_event(self, event)
    }
    
    fn process_mouse_event(&mut self, event: RawMouseEvent) -> SillResult<Vec<InputCommand>> {
        SillEngine::process_mouse_event(self, event)
    }
    
    fn clipboard_copy(&mut self) -> SillResult<String> {
        SillEngine::clipboard_copy(self)
    }
    
    fn clipboard_paste(&mut self) -> SillResult<String> {
        SillEngine::clipboard_paste(self)
    }
    
    fn get_selection(&self) -> Option<&Selection> {
        SillEngine::get_selection(self)
    }
    
    fn set_input_mode(&mut self, mode: InputMode) -> SillResult<()> {
        SillEngine::set_input_mode(self, mode)
    }
    
    fn set_focus(&mut self, pane_id: Option<PaneId>) -> SillResult<()> {
        SillEngine::set_focus(self, pane_id)
    }
    
    fn start_selection(&mut self, position: SelectionPosition, mode: SelectionMode) -> SillResult<()> {
        SillEngine::start_selection(self, position, mode)
    }
    
    fn update_selection(&mut self, position: SelectionPosition) -> SillResult<()> {
        SillEngine::update_selection(self, position)
    }
    
    fn end_selection(&mut self) -> SillResult<Option<Selection>> {
        SillEngine::end_selection(self)
    }
    
    fn clear_selection(&mut self) -> SillResult<()> {
        SillEngine::clear_selection(self)
    }
    
    fn set_key_mapping(&mut self, mapping: KeyMapping) -> SillResult<()> {
        // Add the mapping to the keyboard processor
        let mut mappings = self.config.keyboard.mappings.clone();
        mappings.push(mapping);
        self.config.keyboard.mappings = mappings;
        self.keyboard_processor.update_config(&self.config.keyboard)
    }
    
    fn get_performance_metrics(&self) -> &InputPerformanceTracker {
        SillEngine::get_performance_metrics(self)
    }
    
    fn get_input_statistics(&self) -> InputStatistics {
        SillEngine::get_input_statistics(self)
    }
}

/// Mock implementation for testing
#[cfg(test)]
pub struct MockSillInterface {
    pub key_events: Vec<RawKeyEvent>,
    pub mouse_events: Vec<RawMouseEvent>,
    pub commands_generated: Vec<InputCommand>,
    pub current_selection: Option<Selection>,
    pub input_mode: InputMode,
    pub focused_pane: Option<PaneId>,
    pub clipboard_content: String,
    pub key_mappings: Vec<KeyMapping>,
    pub performance_tracker: InputPerformanceTracker,
}

#[cfg(test)]
impl MockSillInterface {
    pub fn new() -> Self {
        MockSillInterface {
            key_events: Vec::new(),
            mouse_events: Vec::new(),
            commands_generated: Vec::new(),
            current_selection: None,
            input_mode: InputMode::Normal,
            focused_pane: None,
            clipboard_content: String::new(),
            key_mappings: Vec::new(),
            performance_tracker: InputPerformanceTracker::new(),
        }
    }
    
    pub fn with_clipboard_content(mut self, content: String) -> Self {
        self.clipboard_content = content;
        self
    }
    
    pub fn with_selection(mut self, selection: Selection) -> Self {
        self.current_selection = Some(selection);
        self
    }
    
    pub fn with_focus(mut self, pane_id: PaneId) -> Self {
        self.focused_pane = Some(pane_id);
        self
    }
}

#[cfg(test)]
impl SillInterface for MockSillInterface {
    fn process_key_event(&mut self, event: RawKeyEvent) -> SillResult<Vec<InputCommand>> {
        self.key_events.push(event.clone());
        
        // Generate simple commands based on key
        let mut commands = Vec::new();
        
        if let Some(character) = event.character {
            commands.push(InputCommand::InsertText {
                text: character.to_string(),
                target: CommandTarget::ActivePane,
            });
        }
        
        self.commands_generated.extend(commands.clone());
        Ok(commands)
    }
    
    fn process_mouse_event(&mut self, event: RawMouseEvent) -> SillResult<Vec<InputCommand>> {
        self.mouse_events.push(event.clone());
        
        let mut commands = Vec::new();
        
        match event.event_type {
            MouseEventType::Press => {
                let position = SelectionPosition {
                    row: (event.position.1 / 16) as u16,
                    col: (event.position.0 / 8) as u16,
                };
                
                commands.push(InputCommand::StartSelection {
                    position,
                    mode: SelectionMode::Character,
                    target: CommandTarget::ActivePane,
                });
            }
            _ => {}
        }
        
        self.commands_generated.extend(commands.clone());
        Ok(commands)
    }
    
    fn clipboard_copy(&mut self) -> SillResult<String> {
        if let Some(ref selection) = self.current_selection {
            let text = selection.get_text()?;
            self.clipboard_content = text.clone();
            Ok(text)
        } else {
            Err(SillError::no_selection("No text selected"))
        }
    }
    
    fn clipboard_paste(&mut self) -> SillResult<String> {
        Ok(self.clipboard_content.clone())
    }
    
    fn get_selection(&self) -> Option<&Selection> {
        self.current_selection.as_ref()
    }
    
    fn set_input_mode(&mut self, mode: InputMode) -> SillResult<()> {
        self.input_mode = mode;
        Ok(())
    }
    
    fn set_focus(&mut self, pane_id: Option<PaneId>) -> SillResult<()> {
        self.focused_pane = pane_id;
        Ok(())
    }
    
    fn start_selection(&mut self, position: SelectionPosition, mode: SelectionMode) -> SillResult<()> {
        self.current_selection = Some(Selection {
            start: position,
            end: position,
            mode,
            pane_id: self.focused_pane,
            timestamp: std::time::Instant::now(),
            active: true,
        });
        Ok(())
    }
    
    fn update_selection(&mut self, position: SelectionPosition) -> SillResult<()> {
        if let Some(ref mut selection) = self.current_selection {
            selection.end = position;
        } else {
            return Err(SillError::no_selection("No active selection"));
        }
        Ok(())
    }
    
    fn end_selection(&mut self) -> SillResult<Option<Selection>> {
        if let Some(mut selection) = self.current_selection.take() {
            selection.active = false;
            Ok(Some(selection))
        } else {
            Ok(None)
        }
    }
    
    fn clear_selection(&mut self) -> SillResult<()> {
        self.current_selection = None;
        Ok(())
    }
    
    fn set_key_mapping(&mut self, mapping: KeyMapping) -> SillResult<()> {
        self.key_mappings.push(mapping);
        Ok(())
    }
    
    fn get_performance_metrics(&self) -> &InputPerformanceTracker {
        &self.performance_tracker
    }
    
    fn get_input_statistics(&self) -> InputStatistics {
        InputStatistics {
            keys_processed: self.key_events.len() as u64,
            mouse_events_processed: self.mouse_events.len() as u64,
            clipboard_operations: if self.clipboard_content.is_empty() { 0 } else { 1 },
            selections_made: if self.current_selection.is_some() { 1 } else { 0 },
            commands_generated: self.commands_generated.len() as u64,
            average_processing_time: Duration::from_micros(100),
            peak_processing_time: Duration::from_micros(500),
        }
    }
}

/// Utility functions for creating test events
#[cfg(test)]
pub mod test_utils {
    use super::*;
    use std::time::Instant;
    
    /// Create a test key event
    pub fn create_key_event(key_code: u32, character: Option<char>) -> RawKeyEvent {
        RawKeyEvent {
            key_code,
            scan_code: 0,
            modifiers: RawModifiers {
                ctrl: false,
                alt: false,
                shift: false,
                meta: false,
                caps_lock: false,
                num_lock: false,
            },
            character,
            state: KeyState::Press,
            timestamp: Instant::now(),
        }
    }
    
    /// Create a test key event with modifiers
    pub fn create_key_event_with_modifiers(
        key_code: u32,
        character: Option<char>,
        ctrl: bool,
        alt: bool,
        shift: bool,
        meta: bool,
    ) -> RawKeyEvent {
        RawKeyEvent {
            key_code,
            scan_code: 0,
            modifiers: RawModifiers {
                ctrl,
                alt,
                shift,
                meta,
                caps_lock: false,
                num_lock: false,
            },
            character,
            state: KeyState::Press,
            timestamp: Instant::now(),
        }
    }
    
    /// Create a test mouse event
    pub fn create_mouse_event(x: i32, y: i32, button: MouseButton, event_type: MouseEventType) -> RawMouseEvent {
        RawMouseEvent {
            position: (x, y),
            button,
            event_type,
            modifiers: RawModifiers {
                ctrl: false,
                alt: false,
                shift: false,
                meta: false,
                caps_lock: false,
                num_lock: false,
            },
            click_count: 1,
            scroll_delta: (0.0, 0.0),
            timestamp: Instant::now(),
        }
    }
    
    /// Create a test selection
    pub fn create_test_selection(start_row: u16, start_col: u16, end_row: u16, end_col: u16) -> Selection {
        Selection {
            start: SelectionPosition { row: start_row, col: start_col },
            end: SelectionPosition { row: end_row, col: end_col },
            mode: SelectionMode::Character,
            pane_id: Some(PaneId::new(1)),
            timestamp: Instant::now(),
            active: true,
        }
    }
    
    /// Create a test key mapping
    pub fn create_key_mapping(source: Key, target: Key) -> KeyMapping {
        KeyMapping {
            source_key: source,
            modifiers: Modifiers::default(),
            target_key: target,
            mode: None,
        }
    }
}