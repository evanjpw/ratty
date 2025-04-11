# Ratty Development Roadmap

This document outlines the development plan for Ratty, based on the priorities in `metadata/project-goals.md`.

## Phase 1: Project Setup and Basic Functionality

- [x] Claim the "ratty" name on crates.io
- [x] Set up the basic project structure
- [x] Define the architecture
- [ ] Implement basic terminal functionality
  - [ ] Raw mode terminal setup with crossterm
  - [ ] Basic rendering with ratatui
  - [ ] Input handling
  - [ ] Command execution
- [ ] Basic cross-platform support
  - [ ] Linux
  - [ ] macOS
  - [ ] Windows

## Phase 2: High Priority Features

- [ ] Feature #1: Different color schemes/styles/fonts/themes per window
- [ ] Feature #4: Shell Integration
- [ ] Feature #5: Configuration UI
- [ ] Feature #20: Audio/visual alert
- [ ] Feature #23: Settings customization
- [ ] Feature #27: Profiles/themes
- [ ] Feature #30: Keyboard shortcuts
- [ ] Feature #40: Terminal API (VT)
- [ ] Feature #43: Xterm compatibility

## Phase 3: Medium Priority Features

- [ ] Feature #2: Multi tabs
- [ ] Feature #3: Multi windows
- [ ] Feature #7: Performance optimization
- [ ] Feature #9: Hyperlinks
- [ ] Feature #10: Split panels
- [ ] Feature #21: Scrollback configuration
- [ ] Feature #28: Window navigation
- [ ] Feature #31: SGR style mouse reporting
- [ ] Feature #36: Searchable scrollback
- [ ] Feature #38: Xterm style selection
- [ ] Feature #42: Native tabs, splits
- [ ] Feature #46: Desktop notifications
- [ ] Feature #51: 24-bit color

## Phase 4: Feature Parity and Extensions

- [ ] macOS Terminal feature parity
- [ ] iTerm2 feature parity
- [ ] Additional features from the features list

## Release Planning

### v0.1.0 (Initial Release)
- Basic terminal functionality
- Cross-platform support for core functionality

### v0.2.0
- Basic theme support
- Configuration system

### v0.3.0
- Tabs and windows
- Split panels

### v0.5.0
- Shell integration
- Advanced terminal features

### v1.0.0
- Feature complete for core functionality
- Stable API
- Comprehensive documentation