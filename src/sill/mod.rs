pub mod events;
pub mod keyboard;
pub mod mouse;
pub mod clipboard;
pub mod selection;
pub mod commands;
pub mod interface;
pub mod errors;
pub mod config;

#[cfg(test)]
mod tests;

pub use events::*;
pub use keyboard::*;
pub use mouse::*;
pub use clipboard::*;
pub use selection::*;
pub use commands::*;
pub use interface::*;
pub use errors::*;
pub use config::*;

use crate::pane::CursorPosition;
use crate::sash::PaneId;
use std::time::{Duration, Instant};

/// The main Sill engine - coordinates all input handling operations
pub struct SillEngine {
    /// Input processing components
    keyboard_processor: KeyboardProcessor,
    mouse_processor: MouseProcessor,
    clipboard_manager: ClipboardManager,
    
    /// Event routing and management
    input_router: InputRouter,
    selection_engine: SelectionEngine,
    
    /// Configuration
    config: SillConfig,
    
    /// Current state
    current_focus: Option<PaneId>,
    input_mode: InputMode,
    
    /// Event handling
    event_handler: SillEventHandler,
    
    /// Performance tracking
    performance: InputPerformanceTracker,
}

impl SillEngine {
    /// Create a new sill engine
    pub fn new(config: SillConfig) -> SillResult<Self> {
        Ok(SillEngine {
            keyboard_processor: KeyboardProcessor::new(&config.keyboard)?,
            mouse_processor: MouseProcessor::new(&config.mouse)?,
            clipboard_manager: ClipboardManager::new(&config.clipboard)?,
            input_router: InputRouter::new(&config.routing)?,
            selection_engine: SelectionEngine::new(&config.selection)?,
            config,
            current_focus: None,
            input_mode: InputMode::Normal,
            event_handler: SillEventHandler::new(),
            performance: InputPerformanceTracker::new(),
        })
    }
    
    /// Process a raw key event
    pub fn process_key_event(&mut self, event: RawKeyEvent) -> SillResult<Vec<InputCommand>> {
        self.performance.start_input_processing();
        
        // Normalize the key event
        let normalized_event = self.keyboard_processor.normalize_event(event)?;
        
        // Process through keyboard processor
        let key_event = self.keyboard_processor.process_event(normalized_event, &self.input_mode)?;
        
        // Route through input router
        let commands = self.input_router.route_key_event(key_event.clone(), self.current_focus)?;
        
        // Track performance
        self.performance.end_input_processing();
        
        // Emit events for debugging/monitoring
        self.event_handler.emit_key_processed(&key_event)?;
        
        Ok(commands)
    }
    
    /// Process a raw mouse event
    pub fn process_mouse_event(&mut self, event: RawMouseEvent) -> SillResult<Vec<InputCommand>> {
        self.performance.start_input_processing();
        
        // Normalize the mouse event
        let normalized_event = self.mouse_processor.normalize_event(event)?;
        
        // Check if this affects selection
        if self.mouse_processor.affects_selection(&normalized_event) {
            self.selection_engine.handle_mouse_event(&normalized_event)?;
        }
        
        // Process through mouse processor
        let mouse_event = self.mouse_processor.process_event(normalized_event)?;
        
        // Route through input router
        let commands = self.input_router.route_mouse_event(mouse_event.clone(), self.current_focus)?;
        
        // Track performance
        self.performance.end_input_processing();
        
        // Emit events for debugging/monitoring
        self.event_handler.emit_mouse_processed(&mouse_event)?;
        
        Ok(commands)
    }
    
    /// Handle clipboard copy operation
    pub fn clipboard_copy(&mut self) -> SillResult<String> {
        if let Some(selection) = self.selection_engine.get_current_selection() {
            let text = selection.get_text()?;
            self.clipboard_manager.copy_text(&text)?;
            
            // Emit clipboard event
            self.event_handler.emit_clipboard_operation(ClipboardOperation::Copy, &text)?;
            
            Ok(text)
        } else {
            Err(SillError::no_selection("No text selected for copying"))
        }
    }
    
    /// Handle clipboard paste operation
    pub fn clipboard_paste(&mut self) -> SillResult<String> {
        let text = self.clipboard_manager.get_text()?;
        
        // Validate and sanitize clipboard content
        let sanitized_text = self.input_router.sanitize_input(&text)?;
        
        // Emit clipboard event
        self.event_handler.emit_clipboard_operation(ClipboardOperation::Paste, &sanitized_text)?;
        
        Ok(sanitized_text)
    }
    
    /// Set input focus to a specific pane
    pub fn set_focus(&mut self, pane_id: Option<PaneId>) -> SillResult<()> {
        self.current_focus = pane_id;
        self.input_router.update_focus(pane_id)?;
        
        // Clear selection when focus changes
        if pane_id != self.selection_engine.get_focused_pane() {
            self.selection_engine.clear_selection()?;
        }
        
        self.event_handler.emit_focus_changed(pane_id)?;
        Ok(())
    }
    
    /// Set input mode
    pub fn set_input_mode(&mut self, mode: InputMode) -> SillResult<()> {
        let old_mode = self.input_mode;
        self.input_mode = mode;
        
        // Update processors with new mode
        self.keyboard_processor.set_input_mode(mode)?;
        self.mouse_processor.set_input_mode(mode)?;
        
        self.event_handler.emit_input_mode_changed(old_mode, mode)?;
        Ok(())
    }
    
    /// Start text selection at a position
    pub fn start_selection(&mut self, position: SelectionPosition, mode: SelectionMode) -> SillResult<()> {
        self.selection_engine.start_selection(position, mode, self.current_focus)?;
        Ok(())
    }
    
    /// Update selection to a new position
    pub fn update_selection(&mut self, position: SelectionPosition) -> SillResult<()> {
        self.selection_engine.update_selection(position)?;
        Ok(())
    }
    
    /// End current selection
    pub fn end_selection(&mut self) -> SillResult<Option<Selection>> {
        self.selection_engine.end_selection()
    }
    
    /// Clear current selection
    pub fn clear_selection(&mut self) -> SillResult<()> {
        self.selection_engine.clear_selection()
    }
    
    /// Get current selection
    pub fn get_selection(&self) -> Option<&Selection> {
        self.selection_engine.get_current_selection()
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: SillConfig) -> SillResult<()> {
        self.config = config;
        self.keyboard_processor.update_config(&self.config.keyboard)?;
        self.mouse_processor.update_config(&self.config.mouse)?;
        self.clipboard_manager.update_config(&self.config.clipboard)?;
        self.input_router.update_config(&self.config.routing)?;
        self.selection_engine.update_config(&self.config.selection)?;
        Ok(())
    }
    
    /// Get performance metrics
    pub fn get_performance_metrics(&self) -> &InputPerformanceTracker {
        &self.performance
    }
    
    /// Get input statistics
    pub fn get_input_statistics(&self) -> InputStatistics {
        InputStatistics {
            keys_processed: self.keyboard_processor.get_processed_count(),
            mouse_events_processed: self.mouse_processor.get_processed_count(),
            clipboard_operations: self.clipboard_manager.get_operation_count(),
            selections_made: self.selection_engine.get_selection_count(),
            commands_generated: self.input_router.get_command_count(),
            average_processing_time: self.performance.average_processing_time(),
            peak_processing_time: self.performance.peak_processing_time(),
        }
    }
}

/// Raw key event from the platform layer
#[derive(Debug, Clone)]
pub struct RawKeyEvent {
    pub key_code: u32,
    pub scan_code: u32,
    pub modifiers: RawModifiers,
    pub character: Option<char>,
    pub state: KeyState,
    pub timestamp: Instant,
}

/// Raw mouse event from the platform layer
#[derive(Debug, Clone)]
pub struct RawMouseEvent {
    pub position: (i32, i32),
    pub button: MouseButton,
    pub event_type: MouseEventType,
    pub modifiers: RawModifiers,
    pub click_count: u8,
    pub scroll_delta: (f32, f32),
    pub timestamp: Instant,
}

/// Raw modifier state from platform
#[derive(Debug, Clone, Copy)]
pub struct RawModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
    pub caps_lock: bool,
    pub num_lock: bool,
}

/// Input performance tracking
#[derive(Debug, Clone)]
pub struct InputPerformanceTracker {
    total_events: u64,
    total_processing_time: Duration,
    peak_processing_time: Duration,
    last_processing_time: Duration,
    processing_start: Option<Instant>,
}

impl InputPerformanceTracker {
    pub fn new() -> Self {
        InputPerformanceTracker {
            total_events: 0,
            total_processing_time: Duration::new(0, 0),
            peak_processing_time: Duration::new(0, 0),
            last_processing_time: Duration::new(0, 0),
            processing_start: None,
        }
    }
    
    pub fn start_input_processing(&mut self) {
        self.processing_start = Some(Instant::now());
    }
    
    pub fn end_input_processing(&mut self) {
        if let Some(start) = self.processing_start.take() {
            let processing_time = start.elapsed();
            self.last_processing_time = processing_time;
            self.total_processing_time += processing_time;
            self.total_events += 1;
            
            if processing_time > self.peak_processing_time {
                self.peak_processing_time = processing_time;
            }
        }
    }
    
    pub fn average_processing_time(&self) -> Duration {
        if self.total_events > 0 {
            self.total_processing_time / self.total_events as u32
        } else {
            Duration::new(0, 0)
        }
    }
    
    pub fn peak_processing_time(&self) -> Duration {
        self.peak_processing_time
    }
    
    pub fn total_events(&self) -> u64 {
        self.total_events
    }
}

/// Input statistics for monitoring
#[derive(Debug, Clone)]
pub struct InputStatistics {
    pub keys_processed: u64,
    pub mouse_events_processed: u64,
    pub clipboard_operations: u64,
    pub selections_made: u64,
    pub commands_generated: u64,
    pub average_processing_time: Duration,
    pub peak_processing_time: Duration,
}

/// Input modes that affect processing behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputMode {
    /// Normal terminal input mode
    Normal,
    /// Raw mode - minimal processing
    Raw,
    /// Application mode - application-specific sequences
    Application,
    /// Paste mode - special handling for pasted content
    Paste,
}

impl Default for InputMode {
    fn default() -> Self {
        InputMode::Normal
    }
}