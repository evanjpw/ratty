# Ratty Design Principles

## Window Metaphor Naming Scheme

Ratty uses the metaphor of physical windows (wood, metal, glass) for naming structs, modules, and architectural components. This creates an intuitive and cohesive naming system that reflects the layered nature of the application.

### Core Components

- **Frame** - The main application structure that holds everything together
  - `Frame` struct - Main application coordinator
  - `frame` module - Application lifecycle management

- **Sash** - The movable window parts that contain panes
  - `Sash` struct - Tab/window container that can hold multiple panes
  - `sash` module - Multi-window and tab management

- **Pane** - Individual glass sections, each containing terminal content
  - `Pane` struct - Single terminal instance
  - `pane` module - Terminal state and content management

- **Mullion** - Vertical/horizontal dividers between panes
  - `Mullion` struct - Split panel dividers
  - `mullion` module - Layout and splitting logic

- **Sill** - The foundation that interfaces with the building (OS)
  - `Sill` trait - Platform abstraction interface
  - `sill` module - Platform-specific implementations

- **Glazing** - The actual visible content (text, graphics)
  - `Glazing` trait - Rendering abstraction
  - `glazing` module - Rendering implementations (GPU, software, etc.)

- **Hardware** - The mechanical parts that handle interaction
  - `Hardware` struct - Input handling and event processing
  - `hardware` module - Keyboard, mouse, and event management

### Extended Components

- **Trim** - Decorative elements and UI chrome
  - `Trim` struct - Status bars, borders, decorative elements
  - `trim` module - UI decoration and styling

- **Casing** - The outer boundary and window frame
  - `Casing` struct - Window boundaries and chrome
  - `casing` module - Window management and decoration

- **Glazing Bar** - Small dividers within panes
  - `GlazingBar` struct - Fine-grained content dividers
  - Used for sub-pane organization

## Rust Design Principles

Following the four core principles for robust Rust architecture:

### 1. Data Flows Downward

```
Frame (Application Commands)
  ↓
Sash (Window/Tab Commands)  
  ↓
Pane (Terminal Commands)
  ↓
Glazing (Render Commands)
  ↓
Sill (Platform Commands)
```

- Commands and state changes flow from higher-level components to lower-level ones
- Each layer processes and translates commands for the layer below
- No upward data mutation - only event notifications flow upward

### 2. Mutability Stays Contained

- Each component owns its mutable state
- State is accessed through immutable borrows or controlled mutation methods
- Cross-layer communication uses immutable data structures
- State changes are coordinated through message passing, not shared mutation

### 3. Modules/Crates Become Interfaces

- Each layer exposes a clean, minimal interface
- Implementation details are hidden behind trait boundaries
- Dependencies flow downward through the architecture
- Upper layers depend on traits, not concrete implementations

### 4. Pack Lightly

- Structs contain only the data they directly need
- Complex operations are broken into smaller, focused components
- Data is passed as lightweight references rather than heavy clones
- Each component has a single, clear responsibility

## Architecture Implications

### Clear Abstraction Boundaries

The **Glazing Layer** serves as the key abstraction boundary:
- Upper layers work with logical rendering concepts ("display text at row/col")
- Lower layers handle concrete rendering ("call GPU texture update")
- The glazing interface can be swapped without affecting upper layers

### Event Flow Pattern

```
Hardware (Input Events) → Frame → Sash → Pane → Terminal Logic
                                    ↓
Platform Events ← Sill ← Glazing ← Rendering Updates
```

Events flow upward for processing, commands flow downward for execution.

### State Ownership

- **Frame**: Owns application-wide state and coordinates major operations
- **Sash**: Owns window/tab state and layout information  
- **Pane**: Owns terminal content, cursor position, and terminal-specific state
- **Glazing**: Owns rendering resources and display buffers
- **Sill**: Owns platform handles and OS-specific resources

This design ensures that each component has clear ownership boundaries while maintaining loose coupling through well-defined interfaces.