use std::collections::HashMap;
use crate::frame::errors::CommandError;
use crate::frame::config::FontConfig;
use crate::frame::SashId;

/// Global commands that operate at the Frame level
#[derive(Debug, Clone, PartialEq)]
pub enum GlobalCommand {
    // Application lifecycle
    Quit,
    QuitForce, // Quit without confirmation
    Suspend,
    Resume,
    
    // Window management
    NewWindow,
    NewWindowWithConfig(crate::frame::config::WindowConfig),
    CloseWindow(SashId),
    CloseCurrentWindow,
    FocusWindow(SashId),
    NextWindow,
    PreviousWindow,
    ListWindows,
    
    // Global settings
    ChangeGlobalTheme(String),
    ChangeGlobalFont(FontConfig),
    ReloadConfig,
    ShowPreferences,
    SaveConfig,
    
    // Cross-window operations
    BroadcastToAllWindows(SashCommand),
    MoveTabBetweenWindows { 
        from: SashId, 
        to: SashId, 
        tab_id: PaneId 
    },
    
    // Global shortcuts and behavior
    ToggleFullscreen,
    MinimizeAllWindows,
    RestoreAllWindows,
    
    // Debug and development
    ShowDebugInfo,
    ReloadThemes,
    DumpState,
}

/// Commands that operate at the Sash (window) level
#[derive(Debug, Clone, PartialEq)]
pub enum SashCommand {
    // Tab management
    NewTab,
    CloseTab(PaneId),
    CloseCurrentTab,
    NextTab,
    PreviousTab,
    MoveTab(PaneId, usize), // Move tab to position
    
    // Pane management
    SplitHorizontal,
    SplitVertical,
    ClosePane(PaneId),
    FocusPane(PaneId),
    NextPane,
    PreviousPane,
    
    // Layout management
    SetLayout(Layout),
    SaveLayout(String),
    LoadLayout(String),
    
    // Window-specific settings
    SetTheme(String),
    SetFont(FontConfig),
    ToggleTabBar,
    ToggleStatusBar,
}

/// Commands that operate at the Pane (terminal) level
#[derive(Debug, Clone, PartialEq)]
pub enum PaneCommand {
    // Terminal operations
    SendInput(Vec<u8>),
    SendText(String),
    ExecuteCommand(String),
    
    // Terminal state
    Clear,
    Reset,
    SetTitle(String),
    
    // Scrolling
    ScrollUp(usize),
    ScrollDown(usize),
    ScrollToTop,
    ScrollToBottom,
    PageUp,
    PageDown,
    
    // Selection and clipboard
    SelectAll,
    Copy,
    Paste,
    Search(String),
    FindNext,
    FindPrevious,
}

/// Placeholder types - will be defined in their respective layers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PaneId(pub u64);

#[derive(Debug, Clone, PartialEq)]
pub enum Layout {
    Single,
    HorizontalSplit,
    VerticalSplit,
    Grid { rows: usize, cols: usize },
    Custom(String), // Named custom layout
}

/// Command type enumeration for router registration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlobalCommandType {
    Quit,
    NewWindow,
    CloseWindow,
    FocusWindow,
    WindowNavigation,
    GlobalSettings,
    CrossWindow,
    Fullscreen,
    Debug,
}

impl From<&GlobalCommand> for GlobalCommandType {
    fn from(command: &GlobalCommand) -> Self {
        match command {
            GlobalCommand::Quit | GlobalCommand::QuitForce | 
            GlobalCommand::Suspend | GlobalCommand::Resume => GlobalCommandType::Quit,
            
            GlobalCommand::NewWindow | GlobalCommand::NewWindowWithConfig(_) => 
                GlobalCommandType::NewWindow,
            
            GlobalCommand::CloseWindow(_) | GlobalCommand::CloseCurrentWindow => 
                GlobalCommandType::CloseWindow,
            
            GlobalCommand::FocusWindow(_) => GlobalCommandType::FocusWindow,
            
            GlobalCommand::NextWindow | GlobalCommand::PreviousWindow | 
            GlobalCommand::ListWindows => GlobalCommandType::WindowNavigation,
            
            GlobalCommand::ChangeGlobalTheme(_) | GlobalCommand::ChangeGlobalFont(_) |
            GlobalCommand::ReloadConfig | GlobalCommand::ShowPreferences | 
            GlobalCommand::SaveConfig => GlobalCommandType::GlobalSettings,
            
            GlobalCommand::BroadcastToAllWindows(_) | 
            GlobalCommand::MoveTabBetweenWindows { .. } => GlobalCommandType::CrossWindow,
            
            GlobalCommand::ToggleFullscreen | GlobalCommand::MinimizeAllWindows |
            GlobalCommand::RestoreAllWindows => GlobalCommandType::Fullscreen,
            
            GlobalCommand::ShowDebugInfo | GlobalCommand::ReloadThemes | 
            GlobalCommand::DumpState => GlobalCommandType::Debug,
        }
    }
}

/// Trait for command handlers
#[cfg_attr(test, mockall::automock)]
pub trait CommandHandler: Send + Sync {
    fn handle(&mut self, command: GlobalCommand, frame: &mut crate::frame::Frame) -> Result<(), CommandError>;
    fn can_handle(&self, command_type: GlobalCommandType) -> bool;
}

/// Command routing system
pub struct CommandRouter {
    handlers: HashMap<GlobalCommandType, Vec<Box<dyn CommandHandler>>>,
    default_handler: Option<Box<dyn CommandHandler>>,
}

impl CommandRouter {
    /// Create a new command router
    pub fn new() -> Self {
        CommandRouter {
            handlers: HashMap::new(),
            default_handler: None,
        }
    }
    
    /// Register a handler for a specific command type
    pub fn register_handler(&mut self, cmd_type: GlobalCommandType, handler: Box<dyn CommandHandler>) {
        self.handlers.entry(cmd_type).or_insert_with(Vec::new).push(handler);
    }
    
    /// Set the default handler for unregistered command types
    pub fn set_default_handler(&mut self, handler: Box<dyn CommandHandler>) {
        self.default_handler = Some(handler);
    }
    
    /// Route a command to the appropriate handler(s)
    pub fn route(&mut self, command: GlobalCommand, frame: &mut crate::frame::Frame) -> Result<(), CommandError> {
        let command_type = GlobalCommandType::from(&command);
        
        // Try registered handlers first
        if let Some(handlers) = self.handlers.get_mut(&command_type) {
            let mut last_error = None;
            let mut handled = false;
            
            for handler in handlers {
                if handler.can_handle(command_type) {
                    match handler.handle(command.clone(), frame) {
                        Ok(()) => {
                            handled = true;
                            break; // Command successfully handled
                        }
                        Err(e) => {
                            last_error = Some(e);
                            // Continue to next handler
                        }
                    }
                }
            }
            
            if handled {
                return Ok(());
            } else if let Some(error) = last_error {
                return Err(error);
            }
        }
        
        // Try default handler if no specific handler succeeded
        if let Some(ref mut default_handler) = self.default_handler {
            if default_handler.can_handle(command_type) {
                return default_handler.handle(command, frame);
            }
        }
        
        // No handler found
        Err(CommandError::NoHandler(format!("No handler found for command type: {:?}", command_type)))
    }
    
    /// Check if a command type has registered handlers
    pub fn has_handler(&self, command_type: GlobalCommandType) -> bool {
        self.handlers.contains_key(&command_type) || 
        self.default_handler.as_ref().map_or(false, |h| h.can_handle(command_type))
    }
    
    /// Get the number of handlers for a command type
    pub fn handler_count(&self, command_type: GlobalCommandType) -> usize {
        self.handlers.get(&command_type).map_or(0, |h| h.len()) +
        if self.default_handler.as_ref().map_or(false, |h| h.can_handle(command_type)) { 1 } else { 0 }
    }
}

impl Default for CommandRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::Frame;

    // Mock command handler for testing
    struct MockHandler {
        handled_commands: Vec<GlobalCommand>,
        should_fail: bool,
    }
    
    impl MockHandler {
        fn new() -> Self {
            MockHandler {
                handled_commands: Vec::new(),
                should_fail: false,
            }
        }
        
        fn with_failure() -> Self {
            MockHandler {
                handled_commands: Vec::new(),
                should_fail: true,
            }
        }
    }
    
    impl CommandHandler for MockHandler {
        fn handle(&mut self, command: GlobalCommand, _frame: &mut Frame) -> Result<(), CommandError> {
            if self.should_fail {
                return Err(CommandError::ExecutionFailed("Mock failure".to_string()));
            }
            self.handled_commands.push(command);
            Ok(())
        }
        
        fn can_handle(&self, _command_type: GlobalCommandType) -> bool {
            true
        }
    }

    #[test]
    fn test_command_type_conversion() {
        let quit_cmd = GlobalCommand::Quit;
        assert_eq!(GlobalCommandType::from(&quit_cmd), GlobalCommandType::Quit);
        
        let new_window_cmd = GlobalCommand::NewWindow;
        assert_eq!(GlobalCommandType::from(&new_window_cmd), GlobalCommandType::NewWindow);
    }
    
    #[test]
    fn test_command_router_registration() {
        let mut router = CommandRouter::new();
        let handler = Box::new(MockHandler::new());
        
        assert!(!router.has_handler(GlobalCommandType::NewWindow));
        
        router.register_handler(GlobalCommandType::NewWindow, handler);
        
        assert!(router.has_handler(GlobalCommandType::NewWindow));
        assert_eq!(router.handler_count(GlobalCommandType::NewWindow), 1);
    }
    
    #[test]
    fn test_command_routing() {
        let mut router = CommandRouter::new();
        let handler = Box::new(MockHandler::new());
        router.register_handler(GlobalCommandType::NewWindow, handler);
        
        let mut frame = Frame::new().unwrap();
        let command = GlobalCommand::NewWindow;
        
        let result = router.route(command, &mut frame);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_command_routing_failure() {
        let mut router = CommandRouter::new();
        let handler = Box::new(MockHandler::with_failure());
        router.register_handler(GlobalCommandType::NewWindow, handler);
        
        let mut frame = Frame::new().unwrap();
        let command = GlobalCommand::NewWindow;
        
        let result = router.route(command, &mut frame);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_no_handler() {
        let mut router = CommandRouter::new();
        let mut frame = Frame::new().unwrap();
        let command = GlobalCommand::NewWindow;
        
        let result = router.route(command, &mut frame);
        assert!(result.is_err());
        
        if let Err(CommandError::NoHandler(_)) = result {
            // Expected error type
        } else {
            panic!("Expected NoHandler error");
        }
    }
}