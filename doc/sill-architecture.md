# Sill Layer Architecture

## Overview

The Sill layer is the input handling foundation in the Ratty terminal emulator, following our window metaphor where the sill is the bottom horizontal member of a window frame that supports the structure and provides interaction. Just as a window sill is where you might rest your hands to open or interact with a window, the Sill layer is where all user input enters the terminal system.

## Core Responsibilities

1. **Keyboard Input Processing**
   - Raw key event capture and normalization
   - Key mapping and translation
   - Modifier key handling (Ctrl, Alt, Shift, Meta)
   - Key sequence detection and buffering
   - Input mode management (normal, raw, application)

2. **Mouse Input Handling**
   - Mouse event capture and translation
   - Click, drag, and scroll event processing
   - Selection management and highlighting
   - Mouse mode support (normal, button tracking, any-event tracking)
   - Coordinate mapping to terminal cells

3. **Clipboard Integration**
   - Copy/paste operations
   - Selection buffer management
   - Platform-specific clipboard access
   - Format conversion (plain text, RTF, HTML)

4. **Input Event Routing**
   - Event prioritization and queuing
   - Event dispatch to appropriate handlers
   - Command pattern implementation
   - Input focus management
   - Event filtering and transformation

5. **Text Selection**
   - Selection modes (character, word, line, block)
   - Selection rendering coordination
   - Multi-click selection behavior
   - Selection history and persistence

## Architecture Design

### Core Components

```
SillEngine
├── KeyboardProcessor
│   ├── KeyMapper
│   ├── ModifierTracker
│   ├── SequenceDetector
│   └── InputModeManager
├── MouseProcessor
│   ├── EventTranslator
│   ├── SelectionManager
│   ├── CoordinateMapper
│   └── MouseModeManager
├── ClipboardManager
│   ├── SystemClipboard
│   ├── SelectionBuffer
│   └── FormatConverter
├── InputRouter
│   ├── EventQueue
│   ├── CommandDispatcher
│   ├── FocusManager
│   └── EventFilter
└── SelectionEngine
    ├── SelectionState
    ├── SelectionRenderer
    ├── SelectionHistory
    └── SelectionModes
```

### Key Design Patterns

1. **Event-Driven Architecture**: All input is processed through events
2. **Command Pattern**: Input events are translated to commands
3. **Strategy Pattern**: Different input modes use different processing strategies
4. **Observer Pattern**: Components subscribe to input events
5. **State Machine**: For complex input sequences and modes

### Input Flow

```
Raw Input → Platform Layer → Sill Engine → Event Processing → Command Generation → Pane/Frame
                                         ↓
                                   Selection/Clipboard
```

## Implementation Strategy

### Phase 1: Keyboard Foundation
- Basic key event structure
- ASCII character input
- Simple modifier support
- Direct character insertion

### Phase 2: Advanced Keyboard
- Complex key sequences (F-keys, arrows, etc.)
- Full modifier combinations
- Input modes (raw, cooked, application)
- Key mapping configuration

### Phase 3: Mouse Support
- Basic click and drag
- Text selection
- Scroll wheel support
- Mouse reporting modes

### Phase 4: Clipboard & Selection
- Basic copy/paste
- Selection highlighting
- Multi-click selection
- Clipboard format support

### Phase 5: Advanced Features
- IME (Input Method Editor) support
- Accessibility features
- Custom key bindings
- Gesture support

## Interface Design

### SillInterface Trait
```rust
pub trait SillInterface: Send + Sync {
    /// Process a keyboard event
    fn process_key_event(&mut self, event: KeyEvent) -> SillResult<Vec<Command>>;
    
    /// Process a mouse event
    fn process_mouse_event(&mut self, event: MouseEvent) -> SillResult<Vec<Command>>;
    
    /// Handle clipboard operations
    fn clipboard_copy(&self) -> SillResult<String>;
    fn clipboard_paste(&mut self) -> SillResult<String>;
    
    /// Get current selection
    fn get_selection(&self) -> Option<Selection>;
    
    /// Set input mode
    fn set_input_mode(&mut self, mode: InputMode) -> SillResult<()>;
    
    /// Configure key mappings
    fn set_key_mapping(&mut self, mapping: KeyMapping) -> SillResult<()>;
}
```

## Event Types

### KeyEvent
- Key code (physical key)
- Character (logical character)
- Modifiers (Ctrl, Alt, Shift, Meta)
- Key state (press, release, repeat)
- Timestamp

### MouseEvent
- Position (x, y coordinates)
- Button state
- Event type (click, drag, scroll)
- Modifiers
- Click count (for multi-click)

### Command
- Command type (Insert, Delete, Move, etc.)
- Parameters
- Target (active pane, specific pane, global)
- Priority

## Platform Considerations

### Cross-Platform Input
- Abstract platform-specific key codes
- Normalize modifier keys across platforms
- Handle platform clipboard differences
- Support platform-specific features (e.g., macOS trackpad gestures)

### Terminal Compatibility
- Support all standard terminal input sequences
- Emulate legacy terminal behavior
- Handle terminfo capabilities
- Support modern extended features

## Performance Considerations

1. **Input Latency**: Minimize processing delay
2. **Event Batching**: Group related events
3. **Buffering**: Efficient input buffering
4. **Throttling**: Handle rapid input gracefully

## Security Considerations

1. **Input Validation**: Sanitize all input
2. **Injection Prevention**: Prevent control sequence injection
3. **Clipboard Security**: Validate clipboard content
4. **Resource Limits**: Prevent input-based DoS

## Testing Strategy

1. **Unit Tests**: Individual component testing
2. **Integration Tests**: Cross-component interaction
3. **Platform Tests**: Platform-specific behavior
4. **Performance Tests**: Latency and throughput
5. **Fuzzing**: Input validation testing

## Configuration

The Sill layer supports extensive configuration:
- Key mappings and bindings
- Mouse behavior settings
- Selection modes and behavior
- Clipboard integration options
- Input filtering rules

## Error Handling

All errors are categorized:
- Input errors (invalid sequences)
- Platform errors (clipboard access)
- Configuration errors (invalid mappings)
- State errors (invalid mode transitions)

## Future Enhancements

1. **Gesture Support**: Touch and trackpad gestures
2. **Voice Input**: Accessibility enhancement
3. **Macro Recording**: Input automation
4. **AI Assistance**: Predictive input
5. **Remote Input**: Network input protocols