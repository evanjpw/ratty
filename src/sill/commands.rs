use super::*;
use crate::sash::PaneId;
use std::time::Instant;

/// Input commands generated from processed input events
#[derive(Debug, Clone)]
pub enum InputCommand {
    /// Insert character(s) at cursor
    InsertText {
        text: String,
        target: CommandTarget,
    },
    
    /// Movement commands
    MoveCursor {
        direction: CursorDirection,
        amount: u16,
        target: CommandTarget,
    },
    
    /// Scroll commands
    Scroll {
        direction: ScrollDirection,
        amount: usize,
        target: CommandTarget,
    },
    
    /// Selection commands
    StartSelection {
        position: SelectionPosition,
        mode: SelectionMode,
        target: CommandTarget,
    },
    UpdateSelection {
        position: SelectionPosition,
        target: CommandTarget,
    },
    EndSelection {
        target: CommandTarget,
    },
    ClearSelection {
        target: CommandTarget,
    },
    
    /// Clipboard commands
    Copy {
        target: CommandTarget,
    },
    Paste {
        target: CommandTarget,
    },
    Cut {
        target: CommandTarget,
    },
    
    /// Terminal control commands
    SendSequence {
        sequence: String,
        target: CommandTarget,
    },
    SetMode {
        mode: InputMode,
        target: CommandTarget,
    },
    
    /// Window management commands
    SplitPane {
        direction: SplitDirection,
        target: CommandTarget,
    },
    ClosePane {
        target: CommandTarget,
    },
    FocusPane {
        target: CommandTarget,
    },
    ResizePane {
        direction: ResizeDirection,
        amount: i16,
        target: CommandTarget,
    },
    
    /// Application commands
    Quit,
    NewWindow,
    ToggleFullscreen,
    ShowHelp,
    
    /// Configuration commands
    ReloadConfig,
    ChangeTheme {
        theme_name: String,
    },
    
    /// Debug commands
    DebugDump {
        component: String,
    },
    
    /// Custom commands
    Custom {
        name: String,
        args: Vec<String>,
        target: CommandTarget,
    },
}

impl InputCommand {
    /// Get the target of this command
    pub fn target(&self) -> &CommandTarget {
        match self {
            InputCommand::InsertText { target, .. } => target,
            InputCommand::MoveCursor { target, .. } => target,
            InputCommand::Scroll { target, .. } => target,
            InputCommand::StartSelection { target, .. } => target,
            InputCommand::UpdateSelection { target, .. } => target,
            InputCommand::EndSelection { target } => target,
            InputCommand::ClearSelection { target } => target,
            InputCommand::Copy { target } => target,
            InputCommand::Paste { target } => target,
            InputCommand::Cut { target } => target,
            InputCommand::SendSequence { target, .. } => target,
            InputCommand::SetMode { target, .. } => target,
            InputCommand::SplitPane { target, .. } => target,
            InputCommand::ClosePane { target } => target,
            InputCommand::FocusPane { target } => target,
            InputCommand::ResizePane { target, .. } => target,
            InputCommand::Custom { target, .. } => target,
            // Global commands have no specific target
            _ => &CommandTarget::Global,
        }
    }
    
    /// Get command priority (higher = more important)
    pub fn priority(&self) -> u8 {
        match self {
            InputCommand::Quit => 255,
            InputCommand::InsertText { .. } => 200,
            InputCommand::MoveCursor { .. } => 180,
            InputCommand::SendSequence { .. } => 180,
            InputCommand::Copy { .. } | InputCommand::Paste { .. } | InputCommand::Cut { .. } => 150,
            InputCommand::StartSelection { .. } | InputCommand::UpdateSelection { .. } | InputCommand::EndSelection { .. } => 140,
            InputCommand::Scroll { .. } => 120,
            InputCommand::SetMode { .. } => 100,
            InputCommand::FocusPane { .. } => 90,
            InputCommand::SplitPane { .. } | InputCommand::ClosePane { .. } | InputCommand::ResizePane { .. } => 80,
            InputCommand::ChangeTheme { .. } => 70,
            InputCommand::ReloadConfig => 60,
            InputCommand::NewWindow | InputCommand::ToggleFullscreen => 50,
            InputCommand::ShowHelp => 40,
            InputCommand::DebugDump { .. } => 30,
            InputCommand::Custom { .. } => 20,
            InputCommand::ClearSelection { .. } => 10,
        }
    }
    
    /// Check if this command requires a focused pane
    pub fn requires_focus(&self) -> bool {
        matches!(self,
            InputCommand::InsertText { .. } |
            InputCommand::MoveCursor { .. } |
            InputCommand::SendSequence { .. } |
            InputCommand::StartSelection { .. } |
            InputCommand::UpdateSelection { .. } |
            InputCommand::EndSelection { .. } |
            InputCommand::Copy { .. } |
            InputCommand::Paste { .. } |
            InputCommand::Cut { .. } |
            InputCommand::Scroll { .. }
        )
    }
    
    /// Get estimated execution time
    pub fn estimated_duration(&self) -> Duration {
        match self {
            InputCommand::InsertText { text, .. } => {
                // Estimate based on text length
                Duration::from_micros(text.len() as u64 * 10)
            }
            InputCommand::Paste { .. } => Duration::from_millis(5),
            InputCommand::Scroll { .. } => Duration::from_millis(2),
            InputCommand::ChangeTheme { .. } => Duration::from_millis(50),
            InputCommand::ReloadConfig => Duration::from_millis(100),
            _ => Duration::from_micros(100),
        }
    }
}

/// Command target specification
#[derive(Debug, Clone, PartialEq)]
pub enum CommandTarget {
    /// Target the currently focused pane
    ActivePane,
    /// Target a specific pane
    Pane(PaneId),
    /// Target all panes
    AllPanes,
    /// Global command (no specific target)
    Global,
}

impl Default for CommandTarget {
    fn default() -> Self {
        CommandTarget::ActivePane
    }
}

/// Cursor movement directions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CursorDirection {
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    WordLeft,
    WordRight,
    LineStart,
    LineEnd,
    PageUp,
    PageDown,
    DocumentStart,
    DocumentEnd,
}

/// Scroll directions (reusing from other modules)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    Home,
    End,
}

/// Split directions for pane splitting
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

/// Resize directions for pane resizing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResizeDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Command dispatcher for routing commands to appropriate handlers
pub struct CommandDispatcher {
    command_queue: Vec<QueuedCommand>,
    handlers: Vec<Box<dyn CommandHandler>>,
    processing_stats: CommandStats,
}

impl CommandDispatcher {
    pub fn new() -> Self {
        CommandDispatcher {
            command_queue: Vec::new(),
            handlers: Vec::new(),
            processing_stats: CommandStats::new(),
        }
    }
    
    /// Add a command to the queue
    pub fn queue_command(&mut self, command: InputCommand) {
        let priority = command.priority();
        let queued_command = QueuedCommand {
            command,
            timestamp: Instant::now(),
            priority,
        };
        
        self.command_queue.push(queued_command);
        
        // Sort by priority (highest first)
        self.command_queue.sort_by(|a, b| b.priority.cmp(&a.priority));
    }
    
    /// Process all queued commands
    pub fn process_commands(&mut self) -> SillResult<Vec<CommandResult>> {
        let mut results = Vec::new();
        
        while let Some(queued_command) = self.command_queue.pop() {
            self.processing_stats.commands_processed += 1;
            
            let start_time = Instant::now();
            let result = self.dispatch_command(queued_command.command)?;
            let processing_time = start_time.elapsed();
            
            self.processing_stats.total_processing_time += processing_time;
            
            if processing_time > self.processing_stats.max_processing_time {
                self.processing_stats.max_processing_time = processing_time;
            }
            
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// Dispatch a single command to appropriate handler
    fn dispatch_command(&mut self, command: InputCommand) -> SillResult<CommandResult> {
        for handler in &mut self.handlers {
            if handler.can_handle(&command) {
                return handler.handle_command(command);
            }
        }
        
        // No handler found
        Err(SillError::routing(&format!("No handler found for command: {:?}", command)))
    }
    
    /// Register a command handler
    pub fn register_handler(&mut self, handler: Box<dyn CommandHandler>) {
        self.handlers.push(handler);
    }
    
    /// Get command processing statistics
    pub fn get_stats(&self) -> &CommandStats {
        &self.processing_stats
    }
    
    /// Clear command queue
    pub fn clear_queue(&mut self) {
        self.command_queue.clear();
    }
    
    /// Get queue size
    pub fn queue_size(&self) -> usize {
        self.command_queue.len()
    }
}

/// Queued command with metadata
#[derive(Debug)]
struct QueuedCommand {
    command: InputCommand,
    timestamp: Instant,
    priority: u8,
}

/// Command handler trait
pub trait CommandHandler: Send + Sync {
    /// Check if this handler can process the given command
    fn can_handle(&self, command: &InputCommand) -> bool;
    
    /// Handle the command and return result
    fn handle_command(&mut self, command: InputCommand) -> SillResult<CommandResult>;
    
    /// Get handler name for debugging
    fn name(&self) -> &str;
    
    /// Get handler priority (higher priority handlers are checked first)
    fn priority(&self) -> u8 {
        0
    }
}

/// Result of command execution
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub success: bool,
    pub message: Option<String>,
    pub duration: Duration,
    pub side_effects: Vec<SideEffect>,
}

impl CommandResult {
    pub fn success() -> Self {
        CommandResult {
            success: true,
            message: None,
            duration: Duration::from_micros(0),
            side_effects: Vec::new(),
        }
    }
    
    pub fn success_with_message(message: String) -> Self {
        CommandResult {
            success: true,
            message: Some(message),
            duration: Duration::from_micros(0),
            side_effects: Vec::new(),
        }
    }
    
    pub fn failure(message: String) -> Self {
        CommandResult {
            success: false,
            message: Some(message),
            duration: Duration::from_micros(0),
            side_effects: Vec::new(),
        }
    }
    
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
    
    pub fn with_side_effect(mut self, effect: SideEffect) -> Self {
        self.side_effects.push(effect);
        self
    }
}

/// Side effects that can result from command execution
#[derive(Debug, Clone)]
pub enum SideEffect {
    /// Cursor position changed
    CursorMoved {
        old_position: CursorPosition,
        new_position: CursorPosition,
    },
    /// Selection changed
    SelectionChanged {
        old_selection: Option<Selection>,
        new_selection: Option<Selection>,
    },
    /// Screen content changed
    ScreenChanged {
        pane_id: PaneId,
        regions: Vec<String>, // Simplified - would be actual dirty regions
    },
    /// Focus changed
    FocusChanged {
        old_focus: Option<PaneId>,
        new_focus: Option<PaneId>,
    },
    /// Mode changed
    ModeChanged {
        old_mode: InputMode,
        new_mode: InputMode,
    },
    /// Configuration changed
    ConfigChanged {
        component: String,
    },
    /// New command generated
    CommandGenerated {
        command: InputCommand,
    },
}

/// Command processing statistics
#[derive(Debug, Clone)]
pub struct CommandStats {
    pub commands_processed: u64,
    pub total_processing_time: Duration,
    pub max_processing_time: Duration,
    pub commands_by_type: std::collections::HashMap<String, u64>,
}

impl CommandStats {
    pub fn new() -> Self {
        CommandStats {
            commands_processed: 0,
            total_processing_time: Duration::new(0, 0),
            max_processing_time: Duration::new(0, 0),
            commands_by_type: std::collections::HashMap::new(),
        }
    }
    
    pub fn average_processing_time(&self) -> Duration {
        if self.commands_processed > 0 {
            self.total_processing_time / self.commands_processed as u32
        } else {
            Duration::new(0, 0)
        }
    }
}

/// Input router for routing events to commands
pub struct InputRouter {
    config: RoutingConfig,
    command_dispatcher: CommandDispatcher,
    event_filter: EventFilter,
    focus_manager: FocusManager,
    command_count: u64,
}

impl InputRouter {
    pub fn new(config: &RoutingConfig) -> SillResult<Self> {
        Ok(InputRouter {
            config: config.clone(),
            command_dispatcher: CommandDispatcher::new(),
            event_filter: EventFilter::new(&config.filters),
            focus_manager: FocusManager::new(),
            command_count: 0,
        })
    }
    
    /// Route a key event to commands
    pub fn route_key_event(
        &mut self,
        event: KeyEvent,
        focus: Option<PaneId>,
    ) -> SillResult<Vec<InputCommand>> {
        // Apply event filtering
        if !self.event_filter.should_process_key(&event) {
            return Ok(Vec::new());
        }
        
        let target = self.determine_target(focus);
        let mut commands = Vec::new();

        let target_clone = target.clone();
        
        match event.key {
            Key::Character(c) => {
                commands.push(InputCommand::InsertText {
                    text: c.to_string(),
                    target,
                });
            }
            Key::ArrowUp => {
                commands.push(InputCommand::MoveCursor {
                    direction: CursorDirection::Up,
                    amount: 1,
                    target,
                });
            }
            Key::ArrowDown => {
                commands.push(InputCommand::MoveCursor {
                    direction: CursorDirection::Down,
                    amount: 1,
                    target,
                });
            }
            Key::ArrowLeft => {
                commands.push(InputCommand::MoveCursor {
                    direction: CursorDirection::Left,
                    amount: 1,
                    target,
                });
            }
            Key::ArrowRight => {
                commands.push(InputCommand::MoveCursor {
                    direction: CursorDirection::Right,
                    amount: 1,
                    target,
                });
            }
            Key::PageUp => {
                commands.push(InputCommand::Scroll {
                    direction: ScrollDirection::PageUp,
                    amount: 1,
                    target,
                });
            }
            Key::PageDown => {
                commands.push(InputCommand::Scroll {
                    direction: ScrollDirection::PageDown,
                    amount: 1,
                    target,
                });
            }
            Key::Home => {
                commands.push(InputCommand::MoveCursor {
                    direction: CursorDirection::LineStart,
                    amount: 1,
                    target,
                });
            }
            Key::End => {
                commands.push(InputCommand::MoveCursor {
                    direction: CursorDirection::LineEnd,
                    amount: 1,
                    target,
                });
            }
            Key::Enter => {
                commands.push(InputCommand::InsertText {
                    text: "\n".to_string(),
                    target,
                });
            }
            Key::Tab => {
                commands.push(InputCommand::InsertText {
                    text: "\t".to_string(),
                    target,
                });
            }
            Key::Backspace => {
                commands.push(InputCommand::SendSequence {
                    sequence: "\x08".to_string(),
                    target,
                });
            }
            Key::Delete => {
                commands.push(InputCommand::SendSequence {
                    sequence: "\x7f".to_string(),
                    target,
                });
            }
            Key::Escape => {
                commands.push(InputCommand::SendSequence {
                    sequence: "\x1b".to_string(),
                    target,
                });
            }
            _ => {
                // Handle other keys or key combinations
                if let Some(sequence) = self.key_to_sequence(&event.key) {
                    commands.push(InputCommand::SendSequence {
                        sequence,
                        target,
                    });
                }
            }
        }
        
        // Apply modifier-based commands
        if event.modifiers.ctrl {
            commands.extend(self.handle_ctrl_combinations(&event, target_clone)?);
        }
        
        self.command_count += commands.len() as u64;
        Ok(commands)
    }
    
    /// Route a mouse event to commands
    pub fn route_mouse_event(
        &mut self,
        event: MouseEvent,
        focus: Option<PaneId>,
    ) -> SillResult<Vec<InputCommand>> {
        if !self.event_filter.should_process_mouse(&event) {
            return Ok(Vec::new());
        }
        
        let target = self.determine_target(focus);
        let mut commands = Vec::new();
        
        match event.event_type {
            MouseEventType::Press => {
                let position = SelectionPosition {
                    row: event.position.row,
                    col: event.position.col,
                };
                
                let mode = match event.click_count {
                    1 => SelectionMode::Character,
                    2 => SelectionMode::Word,
                    3 => SelectionMode::Line,
                    _ => SelectionMode::Character,
                };
                
                commands.push(InputCommand::StartSelection {
                    position,
                    mode,
                    target: target.clone(),
                });
                
                // Focus the pane under the mouse
                if let CommandTarget::Pane(pane_id) = target {
                    commands.push(InputCommand::FocusPane {
                        target: CommandTarget::Pane(pane_id),
                    });
                }
            }
            MouseEventType::Drag => {
                let position = SelectionPosition {
                    row: event.position.row,
                    col: event.position.col,
                };
                
                commands.push(InputCommand::UpdateSelection {
                    position,
                    target,
                });
            }
            MouseEventType::Release => {
                commands.push(InputCommand::EndSelection { target });
            }
            MouseEventType::Scroll => {
                let direction = if event.scroll_delta.1 > 0.0 {
                    ScrollDirection::Up
                } else {
                    ScrollDirection::Down
                };
                
                commands.push(InputCommand::Scroll {
                    direction,
                    amount: event.scroll_delta.1.abs() as usize,
                    target,
                });
            }
            MouseEventType::Move => {
                // Mouse move events typically don't generate commands
                // unless we're in a special mode
            }
        }
        
        self.command_count += commands.len() as u64;
        Ok(commands)
    }
    
    /// Update focus information
    pub fn update_focus(&mut self, focus: Option<PaneId>) -> SillResult<()> {
        self.focus_manager.set_focus(focus);
        Ok(())
    }
    
    /// Sanitize input text
    pub fn sanitize_input(&self, text: &str) -> SillResult<String> {
        let mut sanitized = text.to_string();
        
        // Remove null characters
        sanitized = sanitized.replace('\0', "");
        
        // Limit length
        if sanitized.len() > self.config.max_input_length {
            sanitized.truncate(self.config.max_input_length);
        }
        
        Ok(sanitized)
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: &RoutingConfig) -> SillResult<()> {
        self.config = config.clone();
        self.event_filter.update_config(&config.filters)?;
        Ok(())
    }
    
    /// Get command count
    pub fn get_command_count(&self) -> u64 {
        self.command_count
    }
    
    /// Determine command target from focus
    fn determine_target(&self, focus: Option<PaneId>) -> CommandTarget {
        match focus {
            Some(pane_id) => CommandTarget::Pane(pane_id),
            None => CommandTarget::ActivePane,
        }
    }
    
    /// Convert key to terminal sequence
    fn key_to_sequence(&self, key: &Key) -> Option<String> {
        match key {
            Key::Function(n) => Some(format!("\x1b[{}~", n + 10)),
            Key::ApplicationCursorUp => Some("\x1bOA".to_string()),
            Key::ApplicationCursorDown => Some("\x1bOB".to_string()),
            Key::ApplicationCursorRight => Some("\x1bOC".to_string()),
            Key::ApplicationCursorLeft => Some("\x1bOD".to_string()),
            _ => None,
        }
    }
    
    /// Handle Ctrl key combinations
    fn handle_ctrl_combinations(
        &self,
        event: &KeyEvent,
        target: CommandTarget,
    ) -> SillResult<Vec<InputCommand>> {
        let mut commands = Vec::new();
        
        if let Key::Character(c) = event.key {
            match c.to_ascii_lowercase() {
                'c' => commands.push(InputCommand::Copy { target }),
                'v' => commands.push(InputCommand::Paste { target }),
                'x' => commands.push(InputCommand::Cut { target }),
                'a' => commands.push(InputCommand::StartSelection {
                    position: SelectionPosition { row: 0, col: 0 },
                    mode: SelectionMode::All,
                    target,
                }),
                _ => {}
            }
        }
        
        Ok(commands)
    }
}

/// Event filter for controlling which events get processed
#[derive(Debug)]
pub struct EventFilter {
    filters: Vec<FilterRule>,
}

impl EventFilter {
    pub fn new(filters: &[FilterRule]) -> Self {
        EventFilter {
            filters: filters.to_vec(),
        }
    }
    
    /// Check if a key event should be processed
    pub fn should_process_key(&self, event: &KeyEvent) -> bool {
        for filter in &self.filters {
            if !filter.allows_key(event) {
                return false;
            }
        }
        true
    }
    
    /// Check if a mouse event should be processed
    pub fn should_process_mouse(&self, event: &MouseEvent) -> bool {
        for filter in &self.filters {
            if !filter.allows_mouse(event) {
                return false;
            }
        }
        true
    }
    
    /// Update filter configuration
    pub fn update_config(&mut self, filters: &[FilterRule]) -> SillResult<()> {
        self.filters = filters.to_vec();
        Ok(())
    }
}

/// Focus manager for tracking input focus
#[derive(Debug)]
pub struct FocusManager {
    current_focus: Option<PaneId>,
    focus_history: Vec<PaneId>,
}

impl FocusManager {
    pub fn new() -> Self {
        FocusManager {
            current_focus: None,
            focus_history: Vec::new(),
        }
    }
    
    /// Set current focus
    pub fn set_focus(&mut self, pane_id: Option<PaneId>) {
        if let Some(id) = pane_id {
            self.focus_history.retain(|&x| x != id);
            self.focus_history.push(id);
            
            // Keep history limited
            if self.focus_history.len() > 10 {
                self.focus_history.remove(0);
            }
        }
        
        self.current_focus = pane_id;
    }
    
    /// Get current focus
    pub fn get_focus(&self) -> Option<PaneId> {
        self.current_focus
    }
    
    /// Get previous focus
    pub fn get_previous_focus(&self) -> Option<PaneId> {
        if self.focus_history.len() >= 2 {
            Some(self.focus_history[self.focus_history.len() - 2])
        } else {
            None
        }
    }
}

/// Filter rule for event filtering
#[derive(Debug, Clone)]
pub struct FilterRule {
    pub name: String,
    pub enabled: bool,
    pub key_filter: Option<KeyFilter>,
    pub mouse_filter: Option<MouseFilter>,
}

impl FilterRule {
    /// Check if this rule allows the key event
    pub fn allows_key(&self, event: &KeyEvent) -> bool {
        if !self.enabled {
            return true;
        }
        
        if let Some(ref filter) = self.key_filter {
            filter.allows(event)
        } else {
            true
        }
    }
    
    /// Check if this rule allows the mouse event
    pub fn allows_mouse(&self, event: &MouseEvent) -> bool {
        if !self.enabled {
            return true;
        }
        
        if let Some(ref filter) = self.mouse_filter {
            filter.allows(event)
        } else {
            true
        }
    }
}

/// Key event filter
#[derive(Debug, Clone)]
pub struct KeyFilter {
    pub allowed_keys: Option<Vec<Key>>,
    pub blocked_keys: Option<Vec<Key>>,
    pub allowed_modifiers: Option<Modifiers>,
}

impl KeyFilter {
    pub fn allows(&self, event: &KeyEvent) -> bool {
        // Check blocked keys first
        if let Some(ref blocked) = self.blocked_keys {
            if blocked.contains(&event.key) {
                return false;
            }
        }
        
        // Check allowed keys
        if let Some(ref allowed) = self.allowed_keys {
            if !allowed.contains(&event.key) {
                return false;
            }
        }
        
        // Check modifiers
        if let Some(ref allowed_mods) = self.allowed_modifiers {
            if event.modifiers != *allowed_mods {
                return false;
            }
        }
        
        true
    }
}

/// Mouse event filter
#[derive(Debug, Clone)]
pub struct MouseFilter {
    pub allowed_buttons: Option<Vec<MouseButton>>,
    pub blocked_buttons: Option<Vec<MouseButton>>,
    pub min_click_count: Option<u8>,
    pub max_click_count: Option<u8>,
}

impl MouseFilter {
    pub fn allows(&self, event: &MouseEvent) -> bool {
        // Check blocked buttons
        if let Some(ref blocked) = self.blocked_buttons {
            if blocked.contains(&event.button) {
                return false;
            }
        }
        
        // Check allowed buttons
        if let Some(ref allowed) = self.allowed_buttons {
            if !allowed.contains(&event.button) {
                return false;
            }
        }
        
        // Check click count range
        if let Some(min) = self.min_click_count {
            if event.click_count < min {
                return false;
            }
        }
        
        if let Some(max) = self.max_click_count {
            if event.click_count > max {
                return false;
            }
        }
        
        true
    }
}

/// Routing configuration
#[derive(Debug, Clone)]
pub struct RoutingConfig {
    pub filters: Vec<FilterRule>,
    pub max_input_length: usize,
    pub enable_command_queuing: bool,
    pub command_timeout: Duration,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        RoutingConfig {
            filters: Vec::new(),
            max_input_length: 1024,
            enable_command_queuing: true,
            command_timeout: Duration::from_millis(100),
        }
    }
}