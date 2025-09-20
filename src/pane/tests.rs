#[cfg(test)]
mod pane_tests {
    use super::*;
    use crate::sash::{PaneId, Theme};
    use crate::pane::{
        Pane, PaneConfig, ScreenBuffer, Line, CellAttributes, BuiltinProfiles, PaneProfile,
        ScrollbackBuffer, Cursor, CursorStyle, CursorVisibility, Terminal, TerminalMode,
        VtCommand, PtyFactory, PtyConfig, PaneInterface
    };
    
    // Helper function to create a test pane
    fn create_test_pane() -> Pane {
        let config = PaneConfig::default();
        Pane::new(PaneId::new(1), config).expect("Failed to create test pane")
    }
    
    // Helper function to create a test pane with custom config
    fn create_test_pane_with_config(config: PaneConfig) -> Pane {
        Pane::new(PaneId::new(1), config).expect("Failed to create test pane")
    }
    
    // ========== Basic Pane Tests ==========
    
    #[test]
    fn test_pane_creation() {
        let pane = create_test_pane();
        assert_eq!(pane.id(), PaneId::new(1));
        assert!(!pane.is_active());
        assert!(!pane.is_modified());
        assert_eq!(pane.get_title(), "Terminal");
        assert_eq!(pane.get_size(), (80, 24));
    }
    
    #[test]
    fn test_pane_active_state() {
        let mut pane = create_test_pane();
        assert!(!pane.is_active());
        
        pane.set_active(true);
        assert!(pane.is_active());
        
        pane.set_active(false);
        assert!(!pane.is_active());
    }
    
    #[test]
    fn test_pane_title() {
        let mut pane = create_test_pane();
        assert_eq!(pane.get_title(), "Terminal");
        
        pane.set_title("Test Pane".to_string());
        assert_eq!(pane.get_title(), "Test Pane");
    }
    
    #[test]
    fn test_pane_modified_state() {
        let mut pane = create_test_pane();
        assert!(!pane.is_modified());
        
        pane.set_modified(true);
        assert!(pane.is_modified());
        
        pane.set_modified(false);
        assert!(!pane.is_modified());
    }
    
    #[test]
    fn test_pane_size() {
        let pane = create_test_pane();
        assert_eq!(pane.get_size(), (80, 24));
        assert_eq!(pane.size(), (80, 24));
    }
    
    #[test]
    fn test_pane_resize() {
        let mut pane = create_test_pane();
        assert_eq!(pane.get_size(), (80, 24));
        
        pane.resize(30, 100).expect("Resize should succeed");
        assert_eq!(pane.get_size(), (100, 30));
    }
    
    #[test]
    fn test_pane_resize_invalid() {
        let mut pane = create_test_pane();
        
        // Test zero width
        assert!(pane.resize(24, 0).is_err());
        
        // Test zero height  
        assert!(pane.resize(0, 80).is_err());
    }
    
    // ========== Configuration Tests ==========
    
    #[test]
    fn test_pane_config_default() {
        let config = PaneConfig::default();
        assert_eq!(config.initial_size, (80, 24));
        assert_eq!(config.scrollback_lines, 10000);
        assert!(config.auto_wrap);
        assert!(config.cursor_blink);
        assert_eq!(config.default_title, "Terminal");
    }
    
    #[test]
    fn test_pane_config_validation() {
        let mut config = PaneConfig::default();
        
        // Valid config should pass
        assert!(config.validate().is_ok());
        
        // Invalid size should fail
        config.initial_size = (5, 2); // Smaller than min_size (10, 3)
        assert!(config.validate().is_err());
        
        // Empty command should fail
        config.initial_size = (80, 24); // Reset to valid
        config.default_command = "".to_string();
        assert!(config.validate().is_err());
        
        // Large scrollback should fail
        config.default_command = "/bin/sh".to_string(); // Reset to valid
        config.scrollback_lines = 2_000_000; // Too large
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_pane_config_builder() {
        let config = PaneConfig::new()
            .with_size(100, 30)
            .with_command("zsh")
            .with_scrollback(20000)
            .with_title("Custom Terminal");
            
        assert_eq!(config.initial_size, (100, 30));
        assert_eq!(config.default_command, "zsh");
        assert_eq!(config.scrollback_lines, 20000);
        assert_eq!(config.default_title, "Custom Terminal");
    }
    
    #[test]
    fn test_pane_update_config() {
        let mut pane = create_test_pane();
        assert_eq!(pane.get_size(), (80, 24));
        
        let new_config = PaneConfig::default().with_size(100, 30);
        pane.update_config(new_config).expect("Config update should succeed");
        
        assert_eq!(pane.get_size(), (100, 30));
    }
    
    // ========== Buffer Tests ==========
    
    #[test]
    fn test_screen_buffer_creation() {
        let buffer = ScreenBuffer::new(80, 24);
        assert_eq!(buffer.width, 80);
        assert_eq!(buffer.height, 24);
        assert_eq!(buffer.lines.len(), 24);
        
        for line in &buffer.lines {
            assert_eq!(line.cells.len(), 80);
        }
    }
    
    #[test]
    fn test_screen_buffer_resize() {
        let mut buffer = ScreenBuffer::new(80, 24);
        
        buffer.resize(100, 30).expect("Resize should succeed");
        assert_eq!(buffer.width, 100);
        assert_eq!(buffer.height, 30);
        assert_eq!(buffer.lines.len(), 30);
        
        for line in &buffer.lines {
            assert_eq!(line.cells.len(), 100);
        }
    }
    
    #[test]
    fn test_line_operations() {
        let mut line = Line::new(80);
        assert_eq!(line.cells.len(), 80);
        assert!(line.is_empty());
        
        // Write a character
        let attrs = CellAttributes::default();
        line.write_char(0, 'H', attrs.clone());
        line.write_char(1, 'i', attrs.clone());
        
        assert!(!line.is_empty());
        assert_eq!(line.text().chars().take(2).collect::<String>(), "Hi");
    }
    
    #[test]
    fn test_scrollback_buffer() {
        let mut scrollback = ScrollbackBuffer::new(5); // Small buffer for testing
        assert_eq!(scrollback.len(), 0);
        assert!(scrollback.is_empty());
        
        // Add lines
        for i in 0..3 {
            let mut line = Line::new(10);
            line.write_char(0, char::from_digit(i, 10).unwrap(), CellAttributes::default());
            scrollback.push_line(line);
        }
        
        assert_eq!(scrollback.len(), 3);
        assert!(!scrollback.is_empty());
        
        // Add more lines to exceed capacity
        for i in 3..8 {
            let mut line = Line::new(10);
            line.write_char(0, char::from_digit(i, 10).unwrap(), CellAttributes::default());
            scrollback.push_line(line);
        }
        
        // Should be capped at max_lines (5)
        assert_eq!(scrollback.len(), 5);
        
        // First lines should have been removed
        if let Some(first_line) = scrollback.get_line(0) {
            assert_eq!(first_line.cells[0].character, '3'); // First preserved line
        }
    }
    
    // ========== Cursor Tests ==========
    
    #[test]
    fn test_cursor_creation() {
        let cursor = Cursor::new();
        assert_eq!(cursor.position.row, 0);
        assert_eq!(cursor.position.col, 0);
        assert_eq!(cursor.style, CursorStyle::Block);
        assert_eq!(cursor.visibility, CursorVisibility::Visible);
    }
    
    #[test]
    fn test_cursor_movement() {
        let mut cursor = Cursor::new();
        let screen = ScreenBuffer::new(80, 24);
        
        // Test forward movement
        cursor.move_forward(5, &screen).expect("Move should succeed");
        assert_eq!(cursor.position.col, 5);
        
        // Test backward movement
        cursor.move_back(2).expect("Move should succeed");
        assert_eq!(cursor.position.col, 3);
        
        // Test down movement
        cursor.move_down(3, &screen).expect("Move should succeed");
        assert_eq!(cursor.position.row, 3);
        
        // Test up movement
        cursor.move_up(1, &screen).expect("Move should succeed");
        assert_eq!(cursor.position.row, 2);
    }
    
    #[test]
    fn test_cursor_position_clamping() {
        let mut cursor = Cursor::new();
        let screen = ScreenBuffer::new(80, 24);
        
        // Test movement beyond boundaries
        cursor.move_forward(100, &screen).expect("Move should succeed");
        assert_eq!(cursor.position.col, 79); // Should be clamped to screen width - 1
        
        cursor.move_down(100, &screen).expect("Move should succeed");
        assert_eq!(cursor.position.row, 23); // Should be clamped to screen height - 1
    }
    
    #[test]
    fn test_cursor_save_restore() {
        let mut cursor = Cursor::new();
        let screen = ScreenBuffer::new(80, 24);
        
        // Move cursor and save position
        cursor.move_forward(10, &screen).expect("Move should succeed");
        cursor.move_down(5, &screen).expect("Move should succeed");
        cursor.save_position();
        
        // Move cursor elsewhere
        cursor.move_forward(20, &screen).expect("Move should succeed");
        cursor.move_down(10, &screen).expect("Move should succeed");
        assert_eq!(cursor.position.col, 30);
        assert_eq!(cursor.position.row, 15);
        
        // Restore position
        cursor.restore_position(&screen).expect("Restore should succeed");
        assert_eq!(cursor.position.col, 10);
        assert_eq!(cursor.position.row, 5);
    }
    
    // ========== Terminal Tests ==========
    
    #[test]
    fn test_terminal_creation() {
        let terminal = Terminal::new().expect("Terminal creation should succeed");
        assert_eq!(terminal.current_mode(), TerminalMode::Normal);
    }
    
    #[test] 
    fn test_terminal_character_processing() {
        let mut terminal = Terminal::new().expect("Terminal creation should succeed");
        
        // Test printable character
        let command = terminal.process_byte(b'A').expect("Processing should succeed");
        assert!(command.is_some());
        
        if let Some(VtCommand::PrintChar(ch)) = command {
            assert_eq!(ch, 'A');
        } else {
            panic!("Expected PrintChar command");
        }
    }
    
    #[test]
    fn test_terminal_control_sequences() {
        let mut terminal = Terminal::new().expect("Terminal creation should succeed");
        
        // Test bell (BEL)
        let command = terminal.process_byte(0x07).expect("Processing should succeed");
        assert!(matches!(command, Some(VtCommand::Bell)));
        
        // Test backspace
        let command = terminal.process_byte(0x08).expect("Processing should succeed");
        assert!(matches!(command, Some(VtCommand::Backspace)));
        
        // Test tab
        let command = terminal.process_byte(0x09).expect("Processing should succeed");
        assert!(matches!(command, Some(VtCommand::Tab)));
        
        // Test line feed
        let command = terminal.process_byte(0x0A).expect("Processing should succeed");
        assert!(matches!(command, Some(VtCommand::LineFeed)));
        
        // Test carriage return
        let command = terminal.process_byte(0x0D).expect("Processing should succeed");
        assert!(matches!(command, Some(VtCommand::CarriageReturn)));
    }
    
    // ========== Statistics Tests ==========
    
    #[test]
    fn test_pane_statistics() {
        let pane = create_test_pane();
        let stats = pane.get_statistics();
        
        assert_eq!(stats.bytes_received, 0);
        assert_eq!(stats.bytes_sent, 0);
        assert_eq!(stats.lines_processed, 0);
        assert!(stats.creation_time.elapsed().as_millis() < 100); // Should be recent
    }
    
    #[test]
    fn test_pane_validation() {
        let pane = create_test_pane();
        assert!(pane.validate_state().is_ok());
    }
    
    // ========== PTY Tests ==========
    
    #[test]
    fn test_pty_factory() {
        let pty = PtyFactory::create();
        assert_eq!(pty.size(), (80, 24));
        assert!(!pty.is_alive());
        assert!(pty.pid().is_none());
    }
    
    #[test]
    fn test_pty_config() {
        let config = PtyConfig::default();
        assert_eq!(config.initial_size, (80, 24));
        assert_eq!(config.scroll_buffer_size, 10000);
        
        let pty = PtyFactory::create_with_config(config);
        assert_eq!(pty.size(), (80, 24));
    }
    
    // ========== Search Tests ==========
    
    #[test]
    fn test_scrollback_search() {
        let mut scrollback = ScrollbackBuffer::new(100);
        
        // Add some test content
        let test_lines = vec!["hello world", "foo bar", "hello again"];
        for text in test_lines {
            let mut line = Line::new(text.len() as u16);
            for (i, ch) in text.chars().enumerate() {
                line.write_char(i as u16, ch, CellAttributes::default());
            }
            scrollback.push_line(line);
        }
        
        // Search for "hello"
        let matches = scrollback.search("hello", true);
        assert_eq!(matches.len(), 2);
        
        // Check match details
        assert_eq!(matches[0].line, 0);
        assert_eq!(matches[0].start_col, 0);
        assert_eq!(matches[0].end_col, 5);
        assert_eq!(matches[0].text, "hello");
        
        assert_eq!(matches[1].line, 2);
        assert_eq!(matches[1].start_col, 0);
        assert_eq!(matches[1].end_col, 5);
    }
    
    // ========== Theme Tests ==========
    
    #[test]
    fn test_theme_application() {
        let mut pane = create_test_pane();
        let theme = Theme::default();
        
        // Apply theme should not fail
        assert!(pane.apply_theme(&theme).is_ok());
    }
    
    // ========== Profile Tests ==========
    
    #[test]
    fn test_builtin_profiles() {
        let profiles = BuiltinProfiles::all();
        assert!(!profiles.is_empty());
        
        let default_profile = BuiltinProfiles::default();
        assert_eq!(default_profile.name, "default");
        assert_eq!(default_profile.config.initial_size, (80, 24));
        
        let dev_profile = BuiltinProfiles::development();
        assert_eq!(dev_profile.name, "development");
        assert_eq!(dev_profile.config.initial_size, (100, 30));
        assert_eq!(dev_profile.config.scrollback_lines, 50000);
    }
    
    #[test]
    fn test_profile_creation() {
        let config = PaneConfig::default().with_size(120, 40);
        let profile = PaneProfile::new("test", config)
            .with_description("Test profile")
            .with_tag("test".to_string());
            
        assert_eq!(profile.name, "test");
        assert_eq!(profile.description, "Test profile");
        assert_eq!(profile.tags, vec!["test"]);
        assert_eq!(profile.config.initial_size, (120, 40));
    }
}