# Glazing Layer Architecture

## Overview

The Glazing layer is responsible for rendering and display in the Ratty terminal emulator. Following the window metaphor, the "glazing" is the transparent surface through which we see the terminal content - it handles all visual presentation, layout, and rendering.

## Core Responsibilities

### 1. Terminal Content Rendering
- Render terminal text with proper formatting (bold, italic, colors, etc.)
- Display cursor in various styles (block, underline, bar) with blinking support
- Handle Unicode and extended character sets
- Support for background and foreground colors (256-color, true color)

### 2. Layout and Viewport Management
- Manage scrollable viewport for terminal content
- Handle scrollback buffer display
- Coordinate pane splitting and tabbed interfaces
- Responsive layout adjustments

### 3. Theme and Styling Engine
- Apply color schemes and themes to terminal content
- Support for custom fonts and font rendering
- Handle high-DPI displays and scaling
- Dark/light mode support

### 4. Performance Optimization
- Dirty region tracking for efficient redraws
- Frame rate limiting and vsync
- Memory-efficient rendering pipelines
- GPU acceleration where available

## Architecture Components

```
┌─────────────────────────────────────────────────────────────────┐
│                        Glazing Layer                            │
├─────────────────────────────────────────────────────────────────┤
│  ┌───────────────┐  ┌─────────────────┐  ┌─────────────────┐   │
│  │    Renderer   │  │  Theme Engine   │  │  Layout Manager │   │
│  │               │  │                 │  │                 │   │
│  │ - Text        │  │ - Color         │  │ - Viewport      │   │
│  │ - Cursor      │  │ - Fonts         │  │ - Scrolling     │   │
│  │ - Decorations │  │ - Styling       │  │ - Positioning   │   │
│  └───────────────┘  └─────────────────┘  └─────────────────┘   │
│                                                                 │
│  ┌───────────────┐  ┌─────────────────┐  ┌─────────────────┐   │
│  │  Performance  │  │   Integration   │  │   Interface     │   │
│  │               │  │                 │  │                 │   │
│  │ - Dirty       │  │ - Ratatui       │  │ - GlazingTrait  │   │
│  │ - Batching    │  │ - Backend       │  │ - Events        │   │
│  │ - Caching     │  │ - Platform      │  │ - Configuration │   │
│  └───────────────┘  └─────────────────┘  └─────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## Data Structures

### Core Types

```rust
pub struct GlazingEngine {
    renderer: TerminalRenderer,
    theme_engine: ThemeEngine,
    layout_manager: LayoutManager,
    performance_tracker: PerformanceTracker,
    config: GlazingConfig,
}

pub struct Viewport {
    scroll_offset: usize,
    visible_lines: usize,
    total_lines: usize,
    horizontal_offset: usize,
}

pub struct RenderFrame {
    content: Vec<RenderedLine>,
    cursor: Option<RenderedCursor>,
    viewport: Viewport,
    dirty_regions: Vec<DirtyRegion>,
}
```

### Rendering Components

```rust
pub struct TerminalRenderer {
    text_renderer: TextRenderer,
    cursor_renderer: CursorRenderer,
    decoration_renderer: DecorationRenderer,
    batching_engine: RenderBatcher,
}

pub struct RenderedLine {
    cells: Vec<RenderedCell>,
    line_number: usize,
    is_wrapped: bool,
    dirty: bool,
}

pub struct RenderedCell {
    character: char,
    style: CellStyle,
    position: CellPosition,
}
```

## Integration Points

### With Pane Layer
- Receives terminal content from ScreenBuffer and ScrollbackBuffer
- Monitors cursor state and position changes
- Responds to resize events and configuration updates

### With Sash Layer
- Handles multi-pane layout and tab rendering
- Coordinates focus indicators and active pane highlighting
- Manages theme application across panes

### With Ratatui
- Uses ratatui as the primary rendering backend
- Leverages ratatui's widget system for UI components
- Integrates with various terminal backends (crossterm, termion, etc.)

## Performance Considerations

### Dirty Region Tracking
- Only redraw changed areas of the terminal
- Batch similar drawing operations
- Cache rendered content where possible

### Memory Management
- Efficient allocation patterns for render frames
- Reuse of render buffers
- Smart garbage collection of cached content

### Frame Rate Management
- Target 60 FPS for smooth cursor blinking and animations
- Adaptive frame rates based on content changes
- Vsync support where available

## Configuration

### Rendering Options
```rust
pub struct GlazingConfig {
    pub target_fps: u32,
    pub enable_gpu_acceleration: bool,
    pub cursor_blink_rate: Duration,
    pub smooth_scrolling: bool,
    pub font_config: FontConfig,
    pub color_config: ColorConfig,
}
```

### Theme Configuration
```rust
pub struct ThemeConfig {
    pub color_scheme: ColorScheme,
    pub font_family: String,
    pub font_size: f32,
    pub line_height: f32,
    pub cursor_style: CursorStyle,
    pub selection_color: Color,
}
```

## Event Handling

### Rendering Events
- Content changed events from Pane layer
- Theme change events from configuration
- Resize events from terminal backend
- Focus change events for cursor display

### Performance Events
- Frame rate monitoring
- Memory usage tracking
- Render time profiling
- GPU utilization metrics

## Error Handling

### Rendering Errors
- Failed draw operations
- Font loading failures
- Color format errors
- Backend communication issues

### Recovery Strategies
- Fallback rendering modes
- Graceful degradation of features
- Error reporting and logging
- Automatic retry mechanisms

## Testing Strategy

### Unit Tests
- Individual renderer component testing
- Theme engine validation
- Layout calculation verification
- Performance metric tracking

### Integration Tests
- End-to-end rendering pipeline testing
- Theme application across components
- Resize handling and layout updates
- Multi-pane rendering coordination

### Performance Tests
- Frame rate benchmarking
- Memory usage profiling
- Rendering latency measurement
- Scalability testing with large content

## Future Considerations

### Advanced Features
- Hardware acceleration support
- Custom shader integration
- Advanced text shaping (ligatures, RTL)
- Animation and transition effects

### Accessibility
- High contrast mode support
- Screen reader integration
- Keyboard navigation aids
- Color blind friendly themes

### Multi-platform Support
- Platform-specific optimizations
- Native font rendering integration
- DPI awareness and scaling
- Window system integration