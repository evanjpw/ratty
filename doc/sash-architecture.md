# Sash Layer Architecture

## Overview

The Sash layer (Layer 3) manages window content organization, handling collections of Panes (terminal instances), layout management, and window-specific functionality. Following the window metaphor, a Sash represents the movable window frame that can contain multiple Panes arranged in various layouts.

## Core Responsibilities

1. **Pane Collection Management**: Owning and organizing multiple terminal Panes
2. **Layout Management**: Handling tabs, splits, grids, and custom arrangements
3. **Focus Management**: Managing which Pane is active within the window
4. **Tab Functionality**: Supporting tabbed interface with multiple terminal sessions
5. **Split Management**: Enabling horizontal/vertical splits and complex layouts
6. **Window-Specific Configuration**: Managing Sash-level settings and themes
7. **Event Coordination**: Routing events between Frame and Pane layers

## Data Structures

### Core Sash Structure

```rust
// sash/mod.rs
pub struct Sash {
    // Identity and state
    id: SashId,
    active: bool,
    
    // Pane management
    panes: HashMap<PaneId, Box<dyn PaneInterface>>,
    active_pane_id: Option<PaneId>,
    next_pane_id: PaneId,
    
    // Layout management
    layout: Layout,
    layout_manager: LayoutManager,
    
    // Configuration
    window_config: WindowConfig,
    theme: Theme,
    
    // Event handling
    event_handler: SashEventHandler,
    
    // Tab management
    tabs: TabManager,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PaneId(pub u64);

impl PaneId {
    pub fn new(id: u64) -> Self {
        PaneId(id)
    }
}
```

### Layout Management

```rust
// sash/layout.rs
#[derive(Debug, Clone, PartialEq)]
pub enum Layout {
    // Single pane fills the entire window
    Single(PaneId),
    
    // Tabbed interface with active tab
    Tabs {
        tabs: Vec<Tab>,
        active_tab: usize,
    },
    
    // Horizontal split (top/bottom)
    HorizontalSplit {
        top: Box<Layout>,
        bottom: Box<Layout>,
        split_ratio: f32, // 0.0 to 1.0
    },
    
    // Vertical split (left/right)
    VerticalSplit {
        left: Box<Layout>,
        right: Box<Layout>,
        split_ratio: f32,
    },
    
    // Grid layout with fixed dimensions
    Grid {
        rows: usize,
        cols: usize,
        cells: Vec<Vec<Option<PaneId>>>,
    },
    
    // Custom named layout (saved configurations)
    Custom {
        name: String,
        layout: Box<Layout>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tab {
    pub pane_id: PaneId,
    pub title: String,
    pub closable: bool,
    pub modified: bool, // Has unsaved content
}

pub struct LayoutManager {
    current_layout: Layout,
    saved_layouts: HashMap<String, Layout>,
    split_constraints: SplitConstraints,
}

#[derive(Debug, Clone)]
pub struct SplitConstraints {
    pub min_pane_width: u16,
    pub min_pane_height: u16,
    pub max_split_depth: usize,
}
```

### Theme Management

```rust
// sash/theme.rs
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    pub name: String,
    pub colors: ColorScheme,
    pub fonts: FontScheme,
    pub spacing: SpacingScheme,
    pub decorations: DecorationScheme,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorScheme {
    // Terminal colors
    pub foreground: Color,
    pub background: Color,
    pub cursor: Color,
    pub selection: Color,
    
    // ANSI colors (16 standard colors)
    pub ansi_colors: [Color; 16],
    
    // UI elements
    pub tab_active_bg: Color,
    pub tab_inactive_bg: Color,
    pub tab_border: Color,
    pub split_border: Color,
    pub status_bar_bg: Color,
    pub status_bar_fg: Color,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FontScheme {
    pub family: String,
    pub size: u16,
    pub weight: FontWeight,
    pub style: FontStyle,
    pub ligatures: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpacingScheme {
    pub tab_height: u16,
    pub tab_padding: u16,
    pub split_border_width: u16,
    pub status_bar_height: u16,
    pub content_padding: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DecorationScheme {
    pub show_tabs: bool,
    pub show_status_bar: bool,
    pub show_split_borders: bool,
    pub tab_style: TabStyle,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TabStyle {
    Classic,    // Traditional rectangular tabs
    Rounded,    // Rounded corner tabs
    Underline,  // Underlined active tab
    Minimal,    // Text-only tabs
}
```

### Tab Management

```rust
// sash/tabs.rs
pub struct TabManager {
    tabs: Vec<Tab>,
    active_tab: usize,
    next_tab_id: u64,
    tab_config: TabConfig,
}

#[derive(Debug, Clone)]
pub struct TabConfig {
    pub max_tabs: Option<usize>,
    pub auto_close_empty: bool,
    pub confirm_close_modified: bool,
    pub tab_width: TabWidth,
    pub new_tab_position: NewTabPosition,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TabWidth {
    Fixed(u16),
    Adaptive { min: u16, max: u16 },
    Flexible,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NewTabPosition {
    End,
    AfterCurrent,
    Beginning,
}

impl TabManager {
    pub fn new(config: TabConfig) -> Self;
    pub fn add_tab(&mut self, pane_id: PaneId, title: String) -> Result<(), SashError>;
    pub fn remove_tab(&mut self, index: usize) -> Result<PaneId, SashError>;
    pub fn set_active_tab(&mut self, index: usize) -> Result<(), SashError>;
    pub fn move_tab(&mut self, from: usize, to: usize) -> Result<(), SashError>;
    pub fn get_active_pane(&self) -> Option<PaneId>;
    pub fn update_tab_title(&mut self, index: usize, title: String) -> Result<(), SashError>;
    pub fn set_tab_modified(&mut self, index: usize, modified: bool) -> Result<(), SashError>;
}
```

### Event Handling

```rust
// sash/events.rs
pub struct SashEventHandler {
    listeners: HashMap<SashEventType, Vec<Box<dyn SashEventListener>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SashEvent {
    // Pane events
    PaneCreated(PaneId),
    PaneDestroyed(PaneId),
    PaneFocused(PaneId),
    PaneUnfocused(PaneId),
    PaneModified(PaneId, bool),
    PaneTitleChanged(PaneId, String),
    
    // Tab events
    TabAdded(usize, PaneId),
    TabRemoved(usize, PaneId),
    TabActivated(usize),
    TabMoved(usize, usize),
    
    // Layout events
    LayoutChanged(Layout),
    SplitCreated(SplitDirection, PaneId),
    SplitRemoved(PaneId),
    SplitResized(f32),
    
    // Theme events
    ThemeChanged(String),
    ColorsChanged(ColorScheme),
    FontChanged(FontScheme),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SashEventType {
    Pane,
    Tab,
    Layout,
    Theme,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

pub trait SashEventListener: Send + Sync {
    fn handle_sash_event(&mut self, event: &SashEvent) -> Result<(), SashError>;
    fn can_handle(&self, event_type: SashEventType) -> bool;
}
```

## Sash Interface

### Primary Interface

```rust
// sash/interface.rs
pub trait SashInterface: Send + Sync {
    // Identity and state
    fn id(&self) -> SashId;
    fn is_active(&self) -> bool;
    fn set_active(&mut self, active: bool);
    
    // Pane management
    fn create_pane(&mut self) -> Result<PaneId, SashError>;
    fn create_pane_with_config(&mut self, config: PaneConfig) -> Result<PaneId, SashError>;
    fn destroy_pane(&mut self, pane_id: PaneId) -> Result<(), SashError>;
    fn get_pane(&self, pane_id: PaneId) -> Option<&dyn PaneInterface>;
    fn get_pane_mut(&mut self, pane_id: PaneId) -> Option<&mut (dyn PaneInterface + '_)>;
    fn list_panes(&self) -> Vec<PaneId>;
    fn pane_count(&self) -> usize;
    
    // Focus management
    fn set_active_pane(&mut self, pane_id: PaneId) -> Result<(), SashError>;
    fn get_active_pane(&self) -> Option<PaneId>;
    fn focus_next_pane(&mut self) -> Result<(), SashError>;
    fn focus_previous_pane(&mut self) -> Result<(), SashError>;
    
    // Tab management
    fn new_tab(&mut self) -> Result<PaneId, SashError>;
    fn close_tab(&mut self, index: usize) -> Result<(), SashError>;
    fn close_current_tab(&mut self) -> Result<(), SashError>;
    fn next_tab(&mut self) -> Result<(), SashError>;
    fn previous_tab(&mut self) -> Result<(), SashError>;
    fn move_tab(&mut self, from: usize, to: usize) -> Result<(), SashError>;
    fn get_tab_count(&self) -> usize;
    fn get_active_tab(&self) -> Option<usize>;
    
    // Split management
    fn split_horizontal(&mut self) -> Result<PaneId, SashError>;
    fn split_vertical(&mut self) -> Result<PaneId, SashError>;
    fn split_pane(&mut self, pane_id: PaneId, direction: SplitDirection) -> Result<PaneId, SashError>;
    fn close_split(&mut self, pane_id: PaneId) -> Result<(), SashError>;
    fn resize_split(&mut self, pane_id: PaneId, ratio: f32) -> Result<(), SashError>;
    
    // Layout management
    fn set_layout(&mut self, layout: Layout) -> Result<(), SashError>;
    fn get_layout(&self) -> &Layout;
    fn save_layout(&mut self, name: String) -> Result<(), SashError>;
    fn load_layout(&mut self, name: String) -> Result<(), SashError>;
    fn list_saved_layouts(&self) -> Vec<String>;
    
    // Theme and configuration
    fn set_theme(&mut self, theme: Theme) -> Result<(), SashError>;
    fn get_theme(&self) -> &Theme;
    fn update_config(&mut self, config: WindowConfig) -> Result<(), SashError>;
    fn get_config(&self) -> &WindowConfig;
    
    // Event handling
    fn register_event_listener(&mut self, event_type: SashEventType, listener: Box<dyn SashEventListener>);
    fn emit_event(&mut self, event: SashEvent) -> Result<(), SashError>;
    
    // State and statistics
    fn get_statistics(&self) -> SashStatistics;
    fn validate_state(&self) -> Result<(), SashError>;
}

#[derive(Debug, Clone)]
pub struct SashStatistics {
    pub pane_count: usize,
    pub tab_count: usize,
    pub active_pane: Option<PaneId>,
    pub active_tab: Option<usize>,
    pub layout_type: String,
    pub memory_usage: usize,
}
```

### Error Handling

```rust
// sash/errors.rs
#[derive(Debug, thiserror::Error)]
pub enum SashError {
    #[error("Pane not found: {0:?}")]
    PaneNotFound(PaneId),
    
    #[error("Tab not found: {0}")]
    TabNotFound(usize),
    
    #[error("Invalid layout: {0}")]
    InvalidLayout(String),
    
    #[error("Split operation failed: {0}")]
    SplitFailed(String),
    
    #[error("Theme error: {0}")]
    ThemeError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Layout constraint violation: {0}")]
    LayoutConstraintViolation(String),
    
    #[error("Maximum panes exceeded: {0}")]
    MaxPanesExceeded(usize),
    
    #[error("Pane creation failed: {0}")]
    PaneCreationFailed(String),
    
    #[error("Event handling error: {0}")]
    EventError(String),
}
```

## Data Flow Patterns

### Downward Command Flow (Frame → Sash → Pane)

```
Frame sends SashCommand → Sash processes → Routes to appropriate Pane(s)
```

### Upward Event Flow (Pane → Sash → Frame)

```
Pane generates event → Sash processes/aggregates → Forwards to Frame if needed
```

### Internal Event Flow

```
User Action → Input Handler → Layout Manager → Pane Focus → Event Emission
```

## Implementation Strategy

### Phase 1: Basic Structure
- Core Sash struct and SashInterface
- Basic pane management (create, destroy, focus)
- Simple single-pane layout

### Phase 2: Tab Management
- Tab creation and destruction
- Tab navigation and focus
- Tab titles and state

### Phase 3: Split Management
- Horizontal and vertical splits
- Split resizing and constraints
- Complex nested layouts

### Phase 4: Theme and Configuration
- Theme loading and application
- Dynamic theme switching
- Window-specific configuration

### Phase 5: Advanced Layouts
- Grid layouts
- Custom saved layouts
- Layout serialization/deserialization

## Testing Strategy

### Unit Testing
- Test each component in isolation with mocked dependencies
- Test layout calculations and transformations
- Test theme application and validation

### Integration Testing
- Test Sash with mock Panes
- Test event flow between components
- Test complex layout scenarios

### Layout Testing
- Test split calculations and constraints
- Test tab ordering and navigation
- Test theme rendering with different layouts

This architecture provides a solid foundation for the Sash layer that follows the window metaphor while maintaining clean separation of concerns and supporting all the planned terminal emulator features.