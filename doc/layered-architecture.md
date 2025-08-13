# Ratty Layered Architecture

## Overview

Ratty uses a strict layered architecture where each layer provides a clean abstraction to the layers above it. This design ensures that implementation changes in lower layers don't affect higher layers, and that data flows follow predictable patterns.

## Layer Definitions

### Layer 4: Frame Layer (Application Coordination)
**Responsibility**: Application lifecycle, global state coordination, and high-level command routing.

```rust
// frame/mod.rs
pub struct Frame {
    sashes: Vec<Sash>,
    active_sash: usize,
    global_config: Config,
    command_router: CommandRouter,
}

pub trait FrameInterface {
    fn new_window(&mut self) -> Result<SashId>;
    fn close_window(&mut self, id: SashId) -> Result<()>;
    fn route_command(&mut self, cmd: GlobalCommand) -> Result<()>;
}
```

**Data it owns**: Global application state, window collection, application-wide configuration
**Data flows down to**: Window/tab management commands to Sash layer

### Layer 3: Sash Layer (Window/Tab Management)
**Responsibility**: Managing collections of terminal panes, window layouts, and tab organization.

```rust
// sash/mod.rs
pub struct Sash {
    panes: Vec<Pane>,
    layout: Layout,
    active_pane: usize,
    window_config: WindowConfig,
}

pub trait SashInterface {
    fn new_pane(&mut self) -> Result<PaneId>;
    fn split_pane(&mut self, id: PaneId, direction: SplitDirection) -> Result<PaneId>;
    fn close_pane(&mut self, id: PaneId) -> Result<()>;
    fn set_layout(&mut self, layout: Layout) -> Result<()>;
}
```

**Data it owns**: Pane collection, layout state, window-specific configuration
**Data flows down to**: Terminal content and rendering commands to Pane layer

### Layer 2: Pane Layer (Terminal Content Management)
**Responsibility**: Terminal state, content buffering, and terminal protocol handling.

```rust
// pane/mod.rs
pub struct Pane {
    terminal_state: TerminalState,
    content_buffer: ContentBuffer,
    cursor: CursorState,
    scrollback: ScrollbackBuffer,
}

pub trait PaneInterface {
    fn write_data(&mut self, data: &[u8]) -> Result<()>;
    fn execute_command(&mut self, cmd: TerminalCommand) -> Result<()>;
    fn get_display_content(&self) -> &DisplayContent;
    fn handle_input(&mut self, input: InputEvent) -> Result<()>;
}
```

**Data it owns**: Terminal content, cursor state, scrollback buffer, terminal-specific settings
**Data flows down to**: Rendering commands and display content to Glazing layer

### Layer 1: Glazing Layer (Rendering Abstraction)
**Responsibility**: Abstract rendering interface that can be implemented by different rendering backends.

```rust
// glazing/mod.rs
pub trait GlazingInterface {
    fn render_content(&mut self, content: &DisplayContent, viewport: Viewport) -> Result<()>;
    fn clear_region(&mut self, region: Region) -> Result<()>;
    fn set_cursor(&mut self, position: Position, style: CursorStyle) -> Result<()>;
    fn present(&mut self) -> Result<()>;
}

pub struct GpuGlazing { /* GPU-specific implementation */ }
pub struct SoftwareGlazing { /* Software rendering implementation */ }
```

**Data it owns**: Rendering resources, display buffers, font/glyph caches
**Data flows down to**: Platform-specific rendering calls to Sill layer

### Layer 0: Sill Layer (Platform Interface)
**Responsibility**: OS-specific operations, window management, and hardware abstraction.

```rust
// sill/mod.rs
pub trait SillInterface {
    fn create_window(&self, config: WindowConfig) -> Result<WindowHandle>;
    fn get_display_size(&self) -> (u32, u32);
    fn poll_events(&self) -> Vec<PlatformEvent>;
    fn present_buffer(&self, handle: WindowHandle, buffer: &[u8]) -> Result<()>;
}

#[cfg(target_os = "macos")]
pub struct MacOSSill { /* macOS-specific implementation */ }
```

**Data it owns**: OS handles, platform resources, hardware state

## Data Flow Patterns

### Downward Command Flow
```
User Action → Frame → Sash → Pane → Glazing → Sill → OS/Hardware
```

Example: User types a character
1. **Hardware** captures keystroke from OS
2. **Frame** receives input event, routes to active window
3. **Sash** forwards to active pane
4. **Pane** processes character, updates terminal state
5. **Glazing** receives render command for updated content
6. **Sill** executes platform-specific drawing operations

### Upward Event Flow
```
OS/Hardware → Sill → Hardware → Frame → Sash → Pane
```

Example: Window resize event
1. **Sill** receives OS window resize event
2. **Hardware** translates to application event
3. **Frame** coordinates global response
4. **Sash** adjusts layout calculations
5. **Pane** reflows content for new dimensions

### Cross-Layer Communication Rules

1. **Direct Dependencies**: Each layer only directly depends on the layer immediately below
2. **Interface Contracts**: All communication happens through trait interfaces
3. **Event Propagation**: Events bubble up, commands flow down
4. **State Isolation**: Each layer owns its state, provides controlled access

## Abstraction Benefits

### Swappable Implementations
- **Glazing Layer**: Can swap between GPU/software rendering without affecting upper layers
- **Sill Layer**: Platform changes don't affect application logic
- **Pane Layer**: Different terminal emulation modes without changing UI

### Testing Isolation
- Each layer can be unit tested with mock implementations of lower layers
- Integration testing can be done layer by layer
- Performance profiling can isolate bottlenecks to specific layers

### Development Workflow
- Teams can work on different layers independently
- Lower-level optimizations don't require upper-level changes
- New features typically start at the appropriate layer and work up/down as needed

## Implementation Strategy

### Phase 1: Sill + Basic Glazing
- Implement platform abstraction with software rendering
- Basic window creation and event handling

### Phase 2: Pane + Enhanced Glazing  
- Add terminal state management
- Implement basic text rendering

### Phase 3: Sash Layer
- Add multi-pane and layout management
- Implement splitting and tab functionality

### Phase 4: Frame Layer
- Complete application coordination
- Add global configuration and command routing

This layered approach ensures that Ratty can evolve incrementally while maintaining clean architectural boundaries.