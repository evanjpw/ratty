use super::*;
use std::time::Instant;

/// Mouse input processor
#[derive(Debug)]
pub struct MouseProcessor {
    config: MouseConfig,
    event_translator: EventTranslator,
    coordinate_mapper: CoordinateMapper,
    mouse_mode_manager: MouseModeManager,
    processed_count: u64,
    last_click_time: Option<Instant>,
    last_click_position: Option<MousePosition>,
    click_count: u8,
}

impl MouseProcessor {
    pub fn new(config: &MouseConfig) -> SillResult<Self> {
        Ok(MouseProcessor {
            config: config.clone(),
            event_translator: EventTranslator::new(),
            coordinate_mapper: CoordinateMapper::new(),
            mouse_mode_manager: MouseModeManager::new(),
            processed_count: 0,
            last_click_time: None,
            last_click_position: None,
            click_count: 0,
        })
    }
    
    /// Normalize a raw mouse event from platform layer
    pub fn normalize_event(&mut self, raw_event: RawMouseEvent) -> SillResult<NormalizedMouseEvent> {
        // Map screen coordinates to terminal cell coordinates
        let position = self.coordinate_mapper.screen_to_terminal(
            raw_event.position.0,
            raw_event.position.1,
        )?;
        
        // Detect multi-click sequences
        let click_count = self.detect_multi_click(&raw_event)?;
        
        // Convert platform button codes
        let button = self.event_translator.translate_button(raw_event.button)?;
        
        // Convert platform event type
        let event_type = self.event_translator.translate_event_type(raw_event.event_type)?;
        
        // Convert platform modifiers
        let modifiers = Modifiers {
            ctrl: raw_event.modifiers.ctrl,
            alt: raw_event.modifiers.alt,
            shift: raw_event.modifiers.shift,
            meta: raw_event.modifiers.meta,
        };
        
        Ok(NormalizedMouseEvent {
            position,
            button,
            event_type,
            modifiers,
            click_count,
            scroll_delta: raw_event.scroll_delta,
            timestamp: raw_event.timestamp,
        })
    }
    
    /// Process a normalized mouse event
    pub fn process_event(&mut self, normalized_event: NormalizedMouseEvent) -> SillResult<MouseEvent> {
        self.processed_count += 1;
        
        // Apply mouse mode transformations
        let transformed_event = self.mouse_mode_manager.transform_event(normalized_event)?;
        
        // Generate appropriate terminal mouse sequences if needed
        let terminal_sequence = if self.mouse_mode_manager.should_report_to_terminal() {
            Some(self.generate_terminal_sequence(&transformed_event)?)
        } else {
            None
        };
        
        Ok(MouseEvent {
            position: transformed_event.position,
            button: transformed_event.button,
            event_type: transformed_event.event_type,
            modifiers: transformed_event.modifiers,
            click_count: transformed_event.click_count,
            scroll_delta: transformed_event.scroll_delta,
            timestamp: transformed_event.timestamp,
            terminal_sequence,
        })
    }
    
    /// Check if this mouse event affects text selection
    pub fn affects_selection(&self, event: &NormalizedMouseEvent) -> bool {
        match event.event_type {
            MouseEventType::Press | MouseEventType::Drag | MouseEventType::Release => {
                // Only left button affects selection by default
                event.button == MouseButton::Left
            }
            MouseEventType::Scroll => false,
            MouseEventType::Move => false,
        }
    }
    
    /// Set input mode
    pub fn set_input_mode(&mut self, mode: InputMode) -> SillResult<()> {
        self.mouse_mode_manager.set_input_mode(mode);
        Ok(())
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: &MouseConfig) -> SillResult<()> {
        self.config = config.clone();
        self.coordinate_mapper.update_config(config)?;
        self.mouse_mode_manager.update_config(config)?;
        Ok(())
    }
    
    /// Get processed event count
    pub fn get_processed_count(&self) -> u64 {
        self.processed_count
    }
    
    /// Detect multi-click sequences
    fn detect_multi_click(&mut self, raw_event: &RawMouseEvent) -> SillResult<u8> {
        if raw_event.event_type != MouseEventType::Press {
            return Ok(1);
        }
        
        let now = raw_event.timestamp;
        let position = MousePosition {
            row: (raw_event.position.1 / self.config.cell_height as i32) as u16,
            col: (raw_event.position.0 / self.config.cell_width as i32) as u16,
        };
        
        let mut click_count = 1;
        
        if let (Some(last_time), Some(last_pos)) = (self.last_click_time, self.last_click_position) {
            let time_delta = now.duration_since(last_time);
            let position_delta = ((position.row as i32 - last_pos.row as i32).abs(),
                                  (position.col as i32 - last_pos.col as i32).abs());
            
            if time_delta <= self.config.double_click_time &&
               position_delta.0 <= self.config.double_click_distance as i32 &&
               position_delta.1 <= self.config.double_click_distance as i32 {
                click_count = (self.click_count + 1).min(3); // Max triple-click
            }
        }
        
        self.last_click_time = Some(now);
        self.last_click_position = Some(position);
        self.click_count = click_count;
        
        Ok(click_count)
    }
    
    /// Generate terminal mouse sequence for reporting to application
    fn generate_terminal_sequence(&self, event: &NormalizedMouseEvent) -> SillResult<String> {
        // Generate SGR (1006) mode mouse sequence
        let button_code = match (event.button, event.event_type) {
            (MouseButton::Left, MouseEventType::Press) => 0,
            (MouseButton::Middle, MouseEventType::Press) => 1,
            (MouseButton::Right, MouseEventType::Press) => 2,
            (MouseButton::Left, MouseEventType::Release) => 0,
            (MouseButton::Middle, MouseEventType::Release) => 1,
            (MouseButton::Right, MouseEventType::Release) => 2,
            (MouseButton::WheelUp, MouseEventType::Scroll) => 64,
            (MouseButton::WheelDown, MouseEventType::Scroll) => 65,
            _ => 0,
        };
        
        let mut modifiers = 0;
        if event.modifiers.shift { modifiers += 4; }
        if event.modifiers.alt { modifiers += 8; }
        if event.modifiers.ctrl { modifiers += 16; }
        
        let final_code = button_code + modifiers;
        let release_char = if event.event_type == MouseEventType::Release { 'm' } else { 'M' };
        
        Ok(format!("\x1b[<{};{};{}{}", 
                  final_code, 
                  event.position.col + 1, 
                  event.position.row + 1, 
                  release_char))
    }
}

/// Event translator for platform-specific mouse events
#[derive(Debug)]
pub struct EventTranslator;

impl EventTranslator {
    pub fn new() -> Self {
        EventTranslator
    }
    
    /// Translate platform mouse button to normalized button
    pub fn translate_button(&self, button: MouseButton) -> SillResult<MouseButton> {
        // Platform button codes would be converted here
        Ok(button)
    }
    
    /// Translate platform event type to normalized event type
    pub fn translate_event_type(&self, event_type: MouseEventType) -> SillResult<MouseEventType> {
        // Platform event type codes would be converted here
        Ok(event_type)
    }
}

/// Coordinate mapper for converting between screen and terminal coordinates
#[derive(Debug)]
pub struct CoordinateMapper {
    cell_width: u16,
    cell_height: u16,
    terminal_width: u16,
    terminal_height: u16,
}

impl CoordinateMapper {
    pub fn new() -> Self {
        CoordinateMapper {
            cell_width: 8,   // Default monospace character width
            cell_height: 16, // Default line height
            terminal_width: 80,
            terminal_height: 24,
        }
    }
    
    /// Convert screen pixel coordinates to terminal cell coordinates
    pub fn screen_to_terminal(&self, x: i32, y: i32) -> SillResult<MousePosition> {
        if x < 0 || y < 0 {
            return Err(SillError::mouse_processing("Negative coordinates"));
        }
        
        let col = (x as u16 / self.cell_width).min(self.terminal_width.saturating_sub(1));
        let row = (y as u16 / self.cell_height).min(self.terminal_height.saturating_sub(1));
        
        Ok(MousePosition { row, col })
    }
    
    /// Convert terminal cell coordinates to screen pixel coordinates
    pub fn terminal_to_screen(&self, position: MousePosition) -> (i32, i32) {
        let x = position.col as i32 * self.cell_width as i32;
        let y = position.row as i32 * self.cell_height as i32;
        (x, y)
    }
    
    /// Update coordinate mapper configuration
    pub fn update_config(&mut self, config: &MouseConfig) -> SillResult<()> {
        self.cell_width = config.cell_width;
        self.cell_height = config.cell_height;
        Ok(())
    }
    
    /// Set terminal dimensions
    pub fn set_terminal_size(&mut self, width: u16, height: u16) {
        self.terminal_width = width;
        self.terminal_height = height;
    }
}

/// Mouse mode manager for handling different mouse reporting modes
#[derive(Debug)]
pub struct MouseModeManager {
    current_mode: MouseMode,
    input_mode: InputMode,
    tracking_enabled: bool,
}

impl MouseModeManager {
    pub fn new() -> Self {
        MouseModeManager {
            current_mode: MouseMode::Normal,
            input_mode: InputMode::Normal,
            tracking_enabled: false,
        }
    }
    
    /// Set mouse mode
    pub fn set_mouse_mode(&mut self, mode: MouseMode) {
        self.current_mode = mode;
        self.tracking_enabled = matches!(mode, 
            MouseMode::ButtonTracking | 
            MouseMode::AnyEventTracking |
            MouseMode::FocusTracking
        );
    }
    
    /// Set input mode
    pub fn set_input_mode(&mut self, mode: InputMode) {
        self.input_mode = mode;
    }
    
    /// Transform mouse event based on current mode
    pub fn transform_event(&self, event: NormalizedMouseEvent) -> SillResult<NormalizedMouseEvent> {
        // Mouse mode transformations would be applied here
        Ok(event)
    }
    
    /// Check if mouse events should be reported to terminal application
    pub fn should_report_to_terminal(&self) -> bool {
        self.tracking_enabled && self.input_mode == InputMode::Application
    }
    
    /// Update mouse mode manager configuration
    pub fn update_config(&mut self, _config: &MouseConfig) -> SillResult<()> {
        Ok(())
    }
}

/// Normalized mouse event after platform conversion
#[derive(Debug, Clone)]
pub struct NormalizedMouseEvent {
    pub position: MousePosition,
    pub button: MouseButton,
    pub event_type: MouseEventType,
    pub modifiers: Modifiers,
    pub click_count: u8,
    pub scroll_delta: (f32, f32),
    pub timestamp: Instant,
}

/// Final processed mouse event
#[derive(Debug, Clone)]
pub struct MouseEvent {
    pub position: MousePosition,
    pub button: MouseButton,
    pub event_type: MouseEventType,
    pub modifiers: Modifiers,
    pub click_count: u8,
    pub scroll_delta: (f32, f32),
    pub timestamp: Instant,
    pub terminal_sequence: Option<String>,
}

/// Mouse position in terminal coordinates
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MousePosition {
    pub row: u16,
    pub col: u16,
}

/// Mouse button enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    WheelUp,
    WheelDown,
    WheelLeft,
    WheelRight,
    Other(u8),
}

/// Mouse event types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseEventType {
    Press,
    Release,
    Move,
    Drag,
    Scroll,
}

/// Mouse modes for terminal compatibility
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseMode {
    /// Normal mode - no reporting to terminal
    Normal,
    /// X10 compatibility mode - button press only
    X10,
    /// Button tracking mode - press and release
    ButtonTracking,
    /// Any event tracking mode - all mouse events
    AnyEventTracking,
    /// Focus tracking mode - focus in/out events
    FocusTracking,
}

/// Mouse configuration
#[derive(Debug, Clone)]
pub struct MouseConfig {
    pub enabled: bool,
    pub cell_width: u16,
    pub cell_height: u16,
    pub double_click_time: Duration,
    pub double_click_distance: u16,
    pub scroll_speed: f32,
    pub invert_scroll: bool,
    pub enable_tracking: bool,
    pub tracking_mode: MouseMode,
    pub focus_follows_mouse: bool,
}

impl Default for MouseConfig {
    fn default() -> Self {
        MouseConfig {
            enabled: true,
            cell_width: 8,
            cell_height: 16,
            double_click_time: Duration::from_millis(500),
            double_click_distance: 2,
            scroll_speed: 1.0,
            invert_scroll: false,
            enable_tracking: true,
            tracking_mode: MouseMode::Normal,
            focus_follows_mouse: false,
        }
    }
}