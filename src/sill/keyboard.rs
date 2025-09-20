use super::*;
use std::collections::HashMap;
use std::time::Instant;

/// Keyboard input processor
#[derive(Debug)]
pub struct KeyboardProcessor {
    config: KeyboardConfig,
    key_mapper: KeyMapper,
    modifier_tracker: ModifierTracker,
    sequence_detector: SequenceDetector,
    input_mode_manager: InputModeManager,
    processed_count: u64,
}

impl KeyboardProcessor {
    pub fn new(config: &KeyboardConfig) -> SillResult<Self> {
        Ok(KeyboardProcessor {
            config: config.clone(),
            key_mapper: KeyMapper::new(&config.mappings)?,
            modifier_tracker: ModifierTracker::new(),
            sequence_detector: SequenceDetector::new(&config.sequences)?,
            input_mode_manager: InputModeManager::new(),
            processed_count: 0,
        })
    }
    
    /// Normalize a raw key event from platform layer
    pub fn normalize_event(&mut self, raw_event: RawKeyEvent) -> SillResult<NormalizedKeyEvent> {
        // Update modifier tracking
        self.modifier_tracker.update_from_raw(&raw_event.modifiers);
        
        // Convert platform-specific key code to normalized key
        let key = self.key_mapper.map_key_code(raw_event.key_code)?;
        
        // Get current modifier state
        let modifiers = self.modifier_tracker.get_current_state();
        
        Ok(NormalizedKeyEvent {
            key,
            character: raw_event.character,
            modifiers,
            state: raw_event.state,
            timestamp: raw_event.timestamp,
        })
    }
    
    /// Process a normalized key event
    pub fn process_event(
        &mut self,
        normalized_event: NormalizedKeyEvent,
        input_mode: &InputMode,
    ) -> SillResult<KeyEvent> {
        self.processed_count += 1;
        
        // Set input mode if changed
        self.input_mode_manager.set_mode(*input_mode);
        
        // Check for multi-key sequences
        if let Some(sequence) = self.sequence_detector.process_key(&normalized_event)? {
            return Ok(KeyEvent {
                key: Key::Sequence(sequence),
                character: None,
                modifiers: normalized_event.modifiers,
                state: normalized_event.state,
                timestamp: normalized_event.timestamp,
                input_mode: *input_mode,
            });
        }
        
        // Apply input mode transformations
        let transformed_event = self.input_mode_manager.transform_event(normalized_event, input_mode)?;
        
        // Apply key mappings
        let mapped_key = self.key_mapper.apply_mappings(&transformed_event, input_mode)?;
        
        Ok(KeyEvent {
            key: mapped_key,
            character: transformed_event.character,
            modifiers: transformed_event.modifiers,
            state: transformed_event.state,
            timestamp: transformed_event.timestamp,
            input_mode: *input_mode,
        })
    }
    
    /// Set input mode
    pub fn set_input_mode(&mut self, mode: InputMode) -> SillResult<()> {
        self.input_mode_manager.set_mode(mode);
        Ok(())
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: &KeyboardConfig) -> SillResult<()> {
        self.config = config.clone();
        self.key_mapper.update_mappings(&config.mappings)?;
        self.sequence_detector.update_sequences(&config.sequences)?;
        Ok(())
    }
    
    /// Get processed event count
    pub fn get_processed_count(&self) -> u64 {
        self.processed_count
    }
}

/// Key mapper for converting and mapping keys
#[derive(Debug)]
pub struct KeyMapper {
    platform_map: HashMap<u32, Key>,
    user_mappings: HashMap<KeyMapping, Key>,
}

impl KeyMapper {
    pub fn new(mappings: &[KeyMapping]) -> SillResult<Self> {
        let mut mapper = KeyMapper {
            platform_map: Self::create_platform_map(),
            user_mappings: HashMap::new(),
        };
        
        mapper.update_mappings(mappings)?;
        Ok(mapper)
    }
    
    /// Map platform key code to normalized key
    pub fn map_key_code(&self, key_code: u32) -> SillResult<Key> {
        self.platform_map
            .get(&key_code)
            .copied()
            .ok_or_else(|| SillError::key_processing(&format!("Unknown key code: {}", key_code)))
    }
    
    /// Apply user key mappings
    pub fn apply_mappings(
        &self,
        event: &NormalizedKeyEvent,
        input_mode: &InputMode,
    ) -> SillResult<Key> {
        // Check for user mappings first
        for (mapping, mapped_key) in &self.user_mappings {
            if mapping.matches(event, input_mode) {
                return Ok(*mapped_key);
            }
        }
        
        // Return original key if no mapping found
        Ok(event.key)
    }
    
    /// Update user mappings
    pub fn update_mappings(&mut self, mappings: &[KeyMapping]) -> SillResult<()> {
        self.user_mappings.clear();
        for mapping in mappings {
            self.user_mappings.insert(mapping.clone(), mapping.target_key);
        }
        Ok(())
    }
    
    /// Create platform-specific key code mapping
    fn create_platform_map() -> HashMap<u32, Key> {
        let mut map: HashMap<u32, Key> = HashMap::new();
        
        // ASCII characters
        for code in 32..127 {
            map.insert(code, Key::Character(code as u8 as char));
        }
        
        // Special keys (platform-specific codes would go here)
        map.insert(0x08, Key::Backspace);
        map.insert(0x09, Key::Tab);
        map.insert(0x0D, Key::Enter);
        map.insert(0x1B, Key::Escape);
        map.insert(0x20, Key::Space);
        map.insert(0x7F, Key::Delete);
        
        // Arrow keys
        map.insert(0x25, Key::ArrowLeft);
        map.insert(0x26, Key::ArrowUp);
        map.insert(0x27, Key::ArrowRight);
        map.insert(0x28, Key::ArrowDown);
        
        // Function keys
        for i in 1..=12 {
            map.insert(0x70 + i - 1, Key::Function(i as u8));
        }
        
        // Modifier keys
        map.insert(0x10, Key::Shift);
        map.insert(0x11, Key::Control);
        map.insert(0x12, Key::Alt);
        map.insert(0x5B, Key::Meta);
        
        // Navigation keys
        map.insert(0x21, Key::PageUp);
        map.insert(0x22, Key::PageDown);
        map.insert(0x23, Key::End);
        map.insert(0x24, Key::Home);
        map.insert(0x2D, Key::Insert);
        
        map
    }
}

/// Modifier key state tracking
#[derive(Debug)]
pub struct ModifierTracker {
    state: Modifiers,
    last_update: Instant,
}

impl ModifierTracker {
    pub fn new() -> Self {
        ModifierTracker {
            state: Modifiers::default(),
            last_update: Instant::now(),
        }
    }
    
    /// Update modifier state from raw platform modifiers
    pub fn update_from_raw(&mut self, raw_modifiers: &RawModifiers) {
        self.state = Modifiers {
            ctrl: raw_modifiers.ctrl,
            alt: raw_modifiers.alt,
            shift: raw_modifiers.shift,
            meta: raw_modifiers.meta,
        };
        self.last_update = Instant::now();
    }
    
    /// Get current modifier state
    pub fn get_current_state(&self) -> Modifiers {
        self.state
    }
    
    /// Check if modifiers have changed recently
    pub fn recently_changed(&self) -> bool {
        self.last_update.elapsed() < Duration::from_millis(50)
    }
}

/// Key sequence detection for multi-key combinations
#[derive(Debug)]
pub struct SequenceDetector {
    sequences: Vec<KeySequence>,
    current_sequence: Vec<Key>,
    last_key_time: Option<Instant>,
    sequence_timeout: Duration,
}

impl SequenceDetector {
    pub fn new(sequences: &[KeySequence]) -> SillResult<Self> {
        Ok(SequenceDetector {
            sequences: sequences.to_vec(),
            current_sequence: Vec::new(),
            last_key_time: None,
            sequence_timeout: Duration::from_millis(1000),
        })
    }
    
    /// Process a key and check for sequence completion
    pub fn process_key(&mut self, event: &NormalizedKeyEvent) -> SillResult<Option<KeySequence>> {
        let now = Instant::now();
        
        // Reset sequence if timeout exceeded
        if let Some(last_time) = self.last_key_time {
            if now.duration_since(last_time) > self.sequence_timeout {
                self.current_sequence.clear();
            }
        }
        
        // Only process key press events for sequences
        if event.state != KeyState::Press {
            return Ok(None);
        }
        
        self.current_sequence.push(event.key);
        self.last_key_time = Some(now);
        
        // Check for complete sequences
        for sequence in &self.sequences {
            if sequence.keys == self.current_sequence {
                let completed_sequence = sequence.clone();
                self.current_sequence.clear();
                return Ok(Some(completed_sequence));
            }
        }
        
        // Check if current sequence is a prefix of any sequence
        let is_prefix = self.sequences.iter().any(|seq| {
            seq.keys.len() > self.current_sequence.len() &&
            seq.keys[..self.current_sequence.len()] == self.current_sequence
        });
        
        if !is_prefix {
            // No sequence starts with this combination, clear and start over
            self.current_sequence.clear();
            self.current_sequence.push(event.key);
        }
        
        Ok(None)
    }
    
    /// Update sequence definitions
    pub fn update_sequences(&mut self, sequences: &[KeySequence]) -> SillResult<()> {
        self.sequences = sequences.to_vec();
        self.current_sequence.clear();
        Ok(())
    }
}

/// Input mode management
#[derive(Debug)]
pub struct InputModeManager {
    current_mode: InputMode,
    mode_stack: Vec<InputMode>,
}

impl InputModeManager {
    pub fn new() -> Self {
        InputModeManager {
            current_mode: InputMode::Normal,
            mode_stack: Vec::new(),
        }
    }
    
    /// Set current input mode
    pub fn set_mode(&mut self, mode: InputMode) {
        self.current_mode = mode;
    }
    
    /// Push current mode and set new mode
    pub fn push_mode(&mut self, mode: InputMode) {
        self.mode_stack.push(self.current_mode);
        self.current_mode = mode;
    }
    
    /// Pop previous mode
    pub fn pop_mode(&mut self) -> Option<InputMode> {
        if let Some(previous_mode) = self.mode_stack.pop() {
            self.current_mode = previous_mode;
            Some(previous_mode)
        } else {
            None
        }
    }
    
    /// Transform event based on input mode
    pub fn transform_event(
        &self,
        event: NormalizedKeyEvent,
        mode: &InputMode,
    ) -> SillResult<NormalizedKeyEvent> {
        match mode {
            InputMode::Normal => Ok(event),
            InputMode::Raw => {
                // In raw mode, pass through with minimal processing
                Ok(event)
            }
            InputMode::Application => {
                // In application mode, some keys generate different sequences
                Ok(self.transform_for_application_mode(event))
            }
            InputMode::Paste => {
                // In paste mode, disable certain key interpretations
                Ok(self.transform_for_paste_mode(event))
            }
        }
    }
    
    /// Transform event for application mode
    fn transform_for_application_mode(&self, mut event: NormalizedKeyEvent) -> NormalizedKeyEvent {
        // Application mode typically affects cursor keys and keypad
        match event.key {
            Key::ArrowUp => event.key = Key::ApplicationCursorUp,
            Key::ArrowDown => event.key = Key::ApplicationCursorDown,
            Key::ArrowLeft => event.key = Key::ApplicationCursorLeft,
            Key::ArrowRight => event.key = Key::ApplicationCursorRight,
            _ => {}
        }
        event
    }
    
    /// Transform event for paste mode
    fn transform_for_paste_mode(&self, mut event: NormalizedKeyEvent) -> NormalizedKeyEvent {
        // In paste mode, disable bracket paste escape sequences
        if matches!(event.key, Key::Escape) {
            event.key = Key::Character('\x1b');
        }
        event
    }
}

/// Normalized key event after platform conversion
#[derive(Debug, Clone)]
pub struct NormalizedKeyEvent {
    pub key: Key,
    pub character: Option<char>,
    pub modifiers: Modifiers,
    pub state: KeyState,
    pub timestamp: Instant,
}

/// Final processed key event
#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub key: Key,
    pub character: Option<char>,
    pub modifiers: Modifiers,
    pub state: KeyState,
    pub timestamp: Instant,
    pub input_mode: InputMode,
}

/// Key enumeration covering all possible keys
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum Key {
    // Character keys
    Character(char),
    
    // Special keys
    Backspace,
    Tab,
    Enter,
    Escape,
    Space,
    Delete,
    
    // Arrow keys
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    
    // Application mode cursor keys
    ApplicationCursorLeft,
    ApplicationCursorRight,
    ApplicationCursorUp,
    ApplicationCursorDown,
    
    // Function keys
    Function(u8), // F1-F12, etc.
    
    // Modifier keys
    Shift,
    Control,
    Alt,
    Meta,
    
    // Navigation keys
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    
    // Keypad keys
    KeypadEnter,
    KeypadPlus,
    KeypadMinus,
    KeypadMultiply,
    KeypadDivide,
    KeypadDecimal,
    KeypadNumber(u8), // 0-9
    
    // Media keys
    VolumeUp,
    VolumeDown,
    VolumeMute,
    
    // System keys
    PrintScreen,
    ScrollLock,
    Pause,
    Menu,
    
    // Sequences
    // Sequence(KeySequence),
    
    // Unknown key
    Unknown(u32),
}

/// Modifier state
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

impl Modifiers {
    /// Check if any modifiers are active
    pub fn any(&self) -> bool {
        self.ctrl || self.alt || self.shift || self.meta
    }
    
    /// Check if specific modifier combination is active
    pub fn matches(&self, ctrl: bool, alt: bool, shift: bool, meta: bool) -> bool {
        self.ctrl == ctrl && self.alt == alt && self.shift == shift && self.meta == meta
    }
}

/// Key state (press, release, repeat)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyState {
    Press,
    Release,
    Repeat,
}

/// Key sequence definition
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeySequence {
    pub keys: Vec<Key>,
    pub name: String,
    pub timeout: Duration,
}

// impl std::hash::Hash for KeySequence {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         self.keys.hash(state);
//         self.name.hash(state);
//         // Skip timeout for hashing as Duration doesn't implement Hash
//     }
// }

/// Key mapping configuration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyMapping {
    pub source_key: Key,
    pub modifiers: Modifiers,
    pub target_key: Key,
    pub mode: Option<InputMode>,
}

impl KeyMapping {
    /// Check if this mapping matches the given event and mode
    pub fn matches(&self, event: &NormalizedKeyEvent, mode: &InputMode) -> bool {
        event.key == self.source_key &&
        event.modifiers == self.modifiers &&
        (self.mode.is_none() || self.mode == Some(*mode))
    }
}

/// Keyboard configuration
#[derive(Debug, Clone)]
pub struct KeyboardConfig {
    pub mappings: Vec<KeyMapping>,
    pub sequences: Vec<KeySequence>,
    pub repeat_delay: Duration,
    pub repeat_rate: Duration,
    pub sequence_timeout: Duration,
    pub enable_application_mode: bool,
    pub enable_bracketed_paste: bool,
}

impl Default for KeyboardConfig {
    fn default() -> Self {
        KeyboardConfig {
            mappings: Vec::new(),
            sequences: Vec::new(),
            repeat_delay: Duration::from_millis(500),
            repeat_rate: Duration::from_millis(33), // ~30 Hz
            sequence_timeout: Duration::from_millis(1000),
            enable_application_mode: true,
            enable_bracketed_paste: true,
        }
    }
}