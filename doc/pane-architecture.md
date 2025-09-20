# Pane Layer Architecture

## Overview

The Pane layer represents individual terminal instances within the Ratty terminal emulator. Each Pane contains a terminal session with its own process, text buffer, cursor state, and terminal emulation. Following the window metaphor, Panes are the individual "glass sections" that contain the actual terminal content.

## Core Responsibilities

1. **Terminal Emulation**: VT100/xterm protocol implementation
2. **Text Buffer Management**: Screen buffer, scrollback, and history
3. **Process Management**: PTY creation and process lifecycle
4. **Cursor Management**: Position, visibility, and styling
5. **Terminal State**: Mode flags, character sets, and attributes
6. **Content Rendering**: Preparing text data for the Glazing layer

## Data Structures

### Primary Pane Structure

```rust
// pane/mod.rs
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PaneId(pub u64);

impl PaneId {
    pub fn new(id: u64) -> Self { PaneId(id) }
    pub fn as_u64(&self) -> u64 { self.0 }
}
```

### Terminal Buffer System

```rust
// pane/buffer.rs
pub struct ScreenBuffer {
    lines: Vec<Line>,
    width: u16,
    height: u16,
    dirty_regions: DirtyTracker,
}

pub struct ScrollbackBuffer {
    lines: VecDeque<Line>,
    max_lines: usize,
    current_size: usize,
}

pub struct Line {
    cells: Vec<Cell>,
    wrapped: bool,
    dirty: bool,
    timestamp: Option<std::time::Instant>,
}

pub struct Cell {
    character: char,
    attributes: CellAttributes,
    foreground: Color,
    background: Color,
}

#[derive(Debug, Clone)]
pub struct CellAttributes {
    bold: bool,
    dim: bool,
    italic: bool,
    underline: UnderlineType,
    strikethrough: bool,
    reverse: bool,
    blink: BlinkType,
    invisible: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum UnderlineType {
    None,
    Single,
    Double,
    Curly,
    Dotted,
    Dashed,
}
```

### Cursor Management

```rust
// pane/cursor.rs
pub struct Cursor {
    position: CursorPosition,
    style: CursorStyle,
    visibility: CursorVisibility,
    blink_state: BlinkState,
    saved_positions: Vec<CursorPosition>,
}

#[derive(Debug, Clone, Copy)]
pub struct CursorPosition {
    pub row: u16,
    pub col: u16,
    pub origin_mode: bool, // Affects row calculation
}

#[derive(Debug, Clone, Copy)]
pub enum CursorStyle {
    Block,
    Underline,
    Bar,
}

#[derive(Debug, Clone, Copy)]
pub enum CursorVisibility {
    Visible,
    Hidden,
    BlinkingBlock,
    BlinkingUnderline,
    BlinkingBar,
}
```

### Terminal Emulation Core

```rust
// pane/terminal.rs
pub struct Terminal {
    parser: VtParser,
    interpreter: VtInterpreter,
    current_mode: TerminalMode,
}

pub struct VtParser {
    state_machine: ParserStateMachine,
    current_sequence: Vec<u8>,
    params: Vec<i32>,
}

pub struct VtInterpreter {
    handler_map: HashMap<VtCommand, Box<dyn VtCommandHandler>>,
}

#[derive(Debug, Clone)]
pub enum VtCommand {
    // Cursor movement
    CursorUp(u16),
    CursorDown(u16),
    CursorForward(u16),
    CursorBack(u16),
    CursorPosition(u16, u16),
    
    // Screen manipulation
    ClearScreen(ClearType),
    ClearLine(ClearType),
    InsertLines(u16),
    DeleteLines(u16),
    
    // Character handling
    PrintChar(char),
    Backspace,
    Tab,
    LineFeed,
    CarriageReturn,
    
    // Attributes
    SetGraphicsRendition(Vec<u8>),
    SetMode(Vec<u16>),
    ResetMode(Vec<u16>),
    
    // Advanced features
    DeviceStatusReport,
    DecPrivateModeSet(u16),
    DecPrivateModeReset(u16),
}
```

### PTY Interface

```rust
// pane/pty.rs
pub trait PtyInterface: Send + Sync {
    /// Spawn a new process with PTY
    fn spawn(&mut self, command: &str, args: &[&str], env: &[(String, String)]) -> PtyResult<()>;
    
    /// Read data from the PTY
    fn read(&mut self) -> PtyResult<Vec<u8>>;
    
    /// Write data to the PTY
    fn write(&mut self, data: &[u8]) -> PtyResult<usize>;
    
    /// Resize the PTY
    fn resize(&mut self, rows: u16, cols: u16) -> PtyResult<()>;
    
    /// Get process ID
    fn pid(&self) -> Option<u32>;
    
    /// Check if process is alive
    fn is_alive(&self) -> bool;
    
    /// Kill the process
    fn kill(&mut self) -> PtyResult<()>;
}

#[derive(Debug, thiserror::Error)]
pub enum PtyError {
    #[error("Failed to spawn process: {0}")]
    SpawnFailed(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Process not running")]
    ProcessNotRunning,
    #[error("Invalid size: {rows}x{cols}")]
    InvalidSize { rows: u16, cols: u16 },
}

pub type PtyResult<T> = Result<T, PtyError>;
```

## Pane Interface

### Primary Interface Implementation

```rust
// pane/interface.rs
pub trait PaneInterface: Send + Sync {
    // Identity and basic state
    fn id(&self) -> PaneId;
    fn is_active(&self) -> bool;
    fn set_active(&mut self, active: bool);
    fn get_title(&self) -> &str;
    fn set_title(&mut self, title: String);
    fn is_modified(&self) -> bool;
    fn set_modified(&mut self, modified: bool);
    
    // Process and PTY management
    fn spawn_process(&mut self, command: &str, args: &[&str], env: &[(String, String)]) -> PaneResult<()>;
    fn kill_process(&mut self) -> PaneResult<()>;
    fn is_process_alive(&self) -> bool;
    fn get_process_id(&self) -> Option<u32>;
    
    // Input/Output
    fn write_input(&mut self, data: &[u8]) -> PaneResult<usize>;
    fn read_output(&mut self) -> PaneResult<Vec<u8>>;
    fn process_output(&mut self, data: &[u8]) -> PaneResult<()>;
    
    // Terminal state
    fn resize(&mut self, rows: u16, cols: u16) -> PaneResult<()>;
    fn get_size(&self) -> (u16, u16);
    fn get_cursor_position(&self) -> (u16, u16);
    
    // Content access
    fn get_line(&self, index: usize) -> Option<&Line>;
    fn get_screen_content(&self) -> &ScreenBuffer;
    fn get_scrollback(&self) -> &ScrollbackBuffer;
    
    // Search and navigation
    fn search(&self, pattern: &str, direction: SearchDirection) -> Vec<SearchMatch>;
    fn scroll_to(&mut self, position: ScrollPosition) -> PaneResult<()>;
    
    // Configuration
    fn update_config(&mut self, config: PaneConfig) -> PaneResult<()>;
    fn get_config(&self) -> &PaneConfig;
    fn apply_theme(&mut self, theme: &Theme) -> PaneResult<()>;
    
    // Event handling
    fn register_event_listener(&mut self, event_type: PaneEventType, listener: Box<dyn PaneEventListener>);
    fn emit_event(&mut self, event: PaneEvent) -> PaneResult<()>;
    
    // Statistics and diagnostics
    fn get_statistics(&self) -> PaneStatistics;
    fn validate_state(&self) -> PaneResult<()>;
}
```

### Events and Error Handling

```rust
// pane/events.rs
#[derive(Debug, Clone)]
pub enum PaneEvent {
    // Process events
    ProcessSpawned(u32),
    ProcessExited(i32),
    ProcessKilled,
    
    // Content events
    ContentChanged(ContentRegion),
    TitleChanged(String),
    CursorMoved(u16, u16),
    
    // Terminal events
    Resized(u16, u16),
    ModeChanged(TerminalMode),
    BellRung,
    
    // User interaction events
    TextSelected(Selection),
    SearchResultsChanged(Vec<SearchMatch>),
    
    // Error events
    PtyError(PtyError),
    TerminalError(String),
}

// pane/errors.rs
#[derive(Debug, thiserror::Error)]
pub enum PaneError {
    #[error("PTY error: {0}")]
    PtyError(#[from] PtyError),
    
    #[error("Terminal parse error: {0}")]
    ParseError(String),
    
    #[error("Invalid cursor position: row={0}, col={1}")]
    InvalidCursorPosition(u16, u16),
    
    #[error("Buffer overflow: attempted to write {0} bytes to {1} capacity")]
    BufferOverflow(usize, usize),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Process operation failed: {0}")]
    ProcessError(String),
    
    #[error("Search failed: {0}")]
    SearchError(String),
}

pub type PaneResult<T> = Result<T, PaneError>;
```

## Implementation Strategy

### Phase 1: Basic Structure
1. Create module structure and basic types
2. Implement PaneInterface trait stub
3. Create basic buffer management
4. Add simple cursor handling

### Phase 2: Terminal Emulation Core
1. Implement VT parser for basic sequences
2. Add character rendering to buffer
3. Implement cursor movement commands
4. Add basic screen manipulation

### Phase 3: PTY Integration
1. Create PTY interface abstraction
2. Implement platform-specific PTY handling
3. Add process spawning and management
4. Integrate input/output processing

### Phase 4: Advanced Features
1. Add scrollback buffer management
2. Implement search functionality
3. Add selection and clipboard integration
4. Implement advanced VT sequences

## Testing Strategy

### Unit Testing
- Mock PTY interface for terminal emulation testing
- Test buffer management independently
- Test VT sequence parsing and interpretation
- Test cursor movement and positioning

### Integration Testing
- Test PTY + terminal emulation integration
- Test event propagation to Sash layer
- Test configuration and theme application
- Test process lifecycle management

### Terminal Compatibility Testing
- Test against known VT100/xterm sequences
- Test with common applications (vim, emacs, htop)
- Test edge cases and error conditions
- Test performance with large outputs

This Pane layer design provides the foundation for full terminal emulation while maintaining clean interfaces with the Sash layer above and preparing for the Glazing layer below.