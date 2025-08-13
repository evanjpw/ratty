# Frame Layer Architecture

## Overview

The Frame layer serves as the top-level application coordinator, responsible for application lifecycle, global state management, and high-level command routing. It owns and coordinates all Sash instances (windows) while maintaining clean separation from lower-level concerns.

## Core Responsibilities

1. **Application Lifecycle**: Startup, shutdown, and main event loop coordination
2. **Global State Coordination**: Managing the collection of windows and application-wide state
3. **Command Routing**: Processing high-level commands and routing them to appropriate components
4. **Configuration Management**: Global application configuration and settings
5. **Event Coordination**: Managing cross-window events and global application events

## Data Structures

### Primary Frame Structure

```rust
// frame/mod.rs
pub struct Frame {
    // Window collection - owned and managed by Frame
    sashes: Vec<Sash>,
    active_sash_id: SashId,
    next_sash_id: SashId,
    
    // Application state
    app_state: ApplicationState,
    global_config: GlobalConfig,
    
    // Event and command coordination
    command_router: CommandRouter,
    event_dispatcher: EventDispatcher,
    
    // Shutdown coordination
    shutdown_requested: bool,
    cleanup_handlers: Vec<Box<dyn CleanupHandler>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SashId(u64);

pub enum ApplicationState {
    Initializing,
    Running,
    Suspended,
    ShuttingDown,
}
```

### Configuration Management

```rust
// frame/config.rs
pub struct GlobalConfig {
    // Application-wide settings
    pub default_theme: String,
    pub default_font_family: String,
    pub default_font_size: u16,
    
    // Window behavior
    pub default_window_size: (u32, u32),
    pub allow_multiple_windows: bool,
    pub restore_windows_on_startup: bool,
    
    // Global shortcuts and behavior
    pub global_shortcuts: HashMap<KeyCombination, GlobalCommand>,
    pub quit_confirmation: bool,
    
    // Platform integration
    pub shell_integration: bool,
    pub notification_settings: NotificationSettings,
}

impl GlobalConfig {
    pub fn load() -> Result<Self, ConfigError>;
    pub fn save(&self) -> Result<(), ConfigError>;
    pub fn merge_user_config(&mut self, user_config: UserConfig) -> Result<(), ConfigError>;
}
```

### Command System

```rust
// frame/commands.rs
pub enum GlobalCommand {
    // Application lifecycle
    Quit,
    Suspend,
    Resume,
    
    // Window management
    NewWindow,
    CloseWindow(SashId),
    FocusWindow(SashId),
    NextWindow,
    PreviousWindow,
    
    // Global settings
    ChangeGlobalTheme(String),
    ChangeGlobalFont(FontConfig),
    ReloadConfig,
    ShowPreferences,
    
    // Cross-window operations
    BroadcastToAllWindows(SashCommand),
    MoveTabBetweenWindows { from: SashId, to: SashId, tab_id: PaneId },
}

pub struct CommandRouter {
    handlers: HashMap<GlobalCommandType, Box<dyn CommandHandler>>,
}

impl CommandRouter {
    pub fn route(&mut self, command: GlobalCommand, frame: &mut Frame) -> Result<(), CommandError>;
    pub fn register_handler(&mut self, cmd_type: GlobalCommandType, handler: Box<dyn CommandHandler>);
}
```

### Event System

```rust
// frame/events.rs
pub enum GlobalEvent {
    // Application events
    ApplicationStarted,
    ApplicationWillTerminate,
    ApplicationDidBecomeActive,
    ApplicationDidResignActive,
    
    // Window events
    WindowCreated(SashId),
    WindowDestroyed(SashId),
    WindowFocused(SashId),
    WindowUnfocused(SashId),
    
    // Configuration events
    ConfigurationChanged(ConfigChange),
    ThemeChanged(String),
    
    // System events
    SystemColorSchemeChanged(ColorScheme),
    SystemFontChanged(FontConfig),
    LowMemoryWarning,
}

pub struct EventDispatcher {
    listeners: HashMap<GlobalEventType, Vec<Box<dyn EventListener>>>,
}

impl EventDispatcher {
    pub fn dispatch(&mut self, event: GlobalEvent) -> Result<(), EventError>;
    pub fn subscribe(&mut self, event_type: GlobalEventType, listener: Box<dyn EventListener>);
    pub fn unsubscribe(&mut self, event_type: GlobalEventType, listener_id: ListenerId);
}
```

## Frame Interface

### Primary Interface

```rust
// frame/interface.rs
pub trait FrameInterface {
    // Lifecycle management
    fn initialize() -> Result<Self, FrameError> where Self: Sized;
    fn run(&mut self) -> Result<(), FrameError>;
    fn shutdown(&mut self) -> Result<(), FrameError>;
    
    // Window management
    fn create_window(&mut self, config: Option<WindowConfig>) -> Result<SashId, FrameError>;
    fn destroy_window(&mut self, id: SashId) -> Result<(), FrameError>;
    fn get_window(&self, id: SashId) -> Option<&Sash>;
    fn get_window_mut(&mut self, id: SashId) -> Option<&mut Sash>;
    fn list_windows(&self) -> Vec<SashId>;
    
    // Focus management
    fn set_active_window(&mut self, id: SashId) -> Result<(), FrameError>;
    fn get_active_window(&self) -> Option<SashId>;
    
    // Command handling
    fn execute_command(&mut self, command: GlobalCommand) -> Result<(), FrameError>;
    
    // Configuration
    fn get_global_config(&self) -> &GlobalConfig;
    fn update_global_config(&mut self, config: GlobalConfig) -> Result<(), FrameError>;
    
    // Event handling
    fn register_event_listener(&mut self, event_type: GlobalEventType, listener: Box<dyn EventListener>);
}
```

### Error Handling

```rust
// frame/errors.rs
#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("Window not found: {0:?}")]
    WindowNotFound(SashId),
    
    #[error("Invalid window state: {0}")]
    InvalidWindowState(String),
    
    #[error("Command execution failed: {0}")]
    CommandFailed(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(#[from] ConfigError),
    
    #[error("Event dispatch error: {0}")]
    EventError(#[from] EventError),
    
    #[error("Shutdown error: {0}")]
    ShutdownError(String),
}
```

## Data Flow Patterns

### Startup Sequence

```
1. Frame::initialize()
   ↓
2. Load GlobalConfig
   ↓  
3. Initialize CommandRouter & EventDispatcher
   ↓
4. Create initial Sash (if configured)
   ↓
5. Dispatch ApplicationStarted event
   ↓
6. Enter main event loop (Frame::run())
```

### Command Processing

```
External Command → Frame::execute_command() → CommandRouter::route() → 
Target Component (Sash/Pane) → State Update → Event Dispatch (if needed)
```

### Event Flow

```
Lower Layer Event → EventDispatcher::dispatch() → 
Registered Listeners → State Updates → Command Generation (if needed)
```

## Implementation Principles

### Data Flows Downward
- Frame sends commands to Sashes
- Sashes never directly modify Frame state
- Configuration changes flow from Frame to all components

### Mutability Stays Contained
- Frame owns all Sash instances
- Configuration is accessed through controlled methods
- State changes happen through explicit interfaces

### Modules as Interfaces
- Frame exposes FrameInterface trait
- All external interaction goes through well-defined interfaces
- Implementation details are hidden

### Pack Lightly
- Frame only contains what it directly needs
- Complex operations are delegated to specialized components
- State is organized by responsibility

## Testing Strategy

### Unit Testing
- Mock Sash implementations for testing Frame logic
- Test command routing without actual window creation
- Test configuration management in isolation

### Integration Testing
- Test Frame + Sash interaction
- Test complete command flows
- Test event propagation

### Lifecycle Testing
- Test startup/shutdown sequences
- Test error recovery
- Test resource cleanup

This Frame architecture provides a solid foundation that can coordinate the entire application while maintaining clean separation of concerns and following Rust best practices.