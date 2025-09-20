#[cfg(test)]
mod sill_tests {
    use super::*;
    use crate::sash::PaneId;
    use std::time::{Duration, Instant};

    // Helper function to create a test sill engine
    fn create_test_sill_engine() -> SillEngine {
        let config = SillConfig::default();
        SillEngine::new(config).expect("Failed to create test sill engine")
    }

    // Helper function to create test raw key event
    fn create_test_key_event(key_code: u32, character: Option<char>) -> RawKeyEvent {
        RawKeyEvent {
            key_code,
            scan_code: 0,
            modifiers: RawModifiers {
                ctrl: false,
                alt: false,
                shift: false,
                meta: false,
                caps_lock: false,
                num_lock: false,
            },
            character,
            state: KeyState::Press,
            timestamp: Instant::now(),
        }
    }

    // Helper function to create test raw mouse event
    fn create_test_mouse_event(x: i32, y: i32, button: MouseButton, event_type: MouseEventType) -> RawMouseEvent {
        RawMouseEvent {
            position: (x, y),
            button,
            event_type,
            modifiers: RawModifiers {
                ctrl: false,
                alt: false,
                shift: false,
                meta: false,
                caps_lock: false,
                num_lock: false,
            },
            click_count: 1,
            scroll_delta: (0.0, 0.0),
            timestamp: Instant::now(),
        }
    }

    // ========== Core Sill Engine Tests ==========

    #[test]
    fn test_sill_engine_creation() {
        let engine = create_test_sill_engine();
        assert_eq!(engine.current_focus, None);
        assert_eq!(engine.input_mode, InputMode::Normal);
    }

    #[test]
    fn test_sill_engine_focus_management() {
        let mut engine = create_test_sill_engine();
        let pane_id = PaneId::new(42);
        
        assert!(engine.set_focus(Some(pane_id)).is_ok());
        assert_eq!(engine.current_focus, Some(pane_id));
        
        assert!(engine.set_focus(None).is_ok());
        assert_eq!(engine.current_focus, None);
    }

    #[test]
    fn test_input_mode_changes() {
        let mut engine = create_test_sill_engine();
        
        assert_eq!(engine.input_mode, InputMode::Normal);
        
        assert!(engine.set_input_mode(InputMode::Raw).is_ok());
        assert_eq!(engine.input_mode, InputMode::Raw);
        
        assert!(engine.set_input_mode(InputMode::Application).is_ok());
        assert_eq!(engine.input_mode, InputMode::Application);
    }

    // ========== Keyboard Processing Tests ==========

    #[test]
    fn test_keyboard_character_input() {
        let mut engine = create_test_sill_engine();
        let event = create_test_key_event(65, Some('A')); // 'A' key
        
        let commands = engine.process_key_event(event).unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            InputCommand::InsertText { text, target } => {
                assert_eq!(text, "A");
                assert_eq!(*target, CommandTarget::ActivePane);
            }
            _ => panic!("Expected InsertText command"),
        }
    }

    #[test]
    fn test_keyboard_arrow_keys() {
        let mut engine = create_test_sill_engine();
        
        // Test up arrow
        let up_event = create_test_key_event(0x26, None); // Up arrow
        let commands = engine.process_key_event(up_event).unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            InputCommand::MoveCursor { direction, amount, .. } => {
                assert_eq!(*direction, CursorDirection::Up);
                assert_eq!(*amount, 1);
            }
            _ => panic!("Expected MoveCursor command"),
        }
    }

    #[test]
    fn test_keyboard_special_keys() {
        let mut engine = create_test_sill_engine();
        
        // Test Enter key
        let enter_event = create_test_key_event(0x0D, Some('\r'));
        let commands = engine.process_key_event(enter_event).unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            InputCommand::InsertText { text, .. } => {
                assert_eq!(text, "\n");
            }
            _ => panic!("Expected InsertText command for Enter"),
        }
    }

    #[test]
    fn test_keyboard_with_modifiers() {
        let mut engine = create_test_sill_engine();
        
        // Test Ctrl+C
        let mut ctrl_c_event = create_test_key_event(67, Some('C'));
        ctrl_c_event.modifiers.ctrl = true;
        
        let commands = engine.process_key_event(ctrl_c_event).unwrap();
        
        // Should generate copy command
        assert!(commands.iter().any(|cmd| matches!(cmd, InputCommand::Copy { .. })));
    }

    // ========== Mouse Processing Tests ==========

    #[test]
    fn test_mouse_click_selection() {
        let mut engine = create_test_sill_engine();
        let mouse_event = create_test_mouse_event(100, 50, MouseButton::Left, MouseEventType::Press);
        
        let commands = engine.process_mouse_event(mouse_event).unwrap();
        
        // Should generate start selection command
        assert!(commands.iter().any(|cmd| matches!(cmd, InputCommand::StartSelection { .. })));
    }

    #[test]
    fn test_mouse_drag_selection() {
        let mut engine = create_test_sill_engine();
        
        // Start selection
        let press_event = create_test_mouse_event(100, 50, MouseButton::Left, MouseEventType::Press);
        engine.process_mouse_event(press_event).unwrap();
        
        // Drag to extend selection
        let drag_event = create_test_mouse_event(200, 100, MouseButton::Left, MouseEventType::Drag);
        let commands = engine.process_mouse_event(drag_event).unwrap();
        
        // Should generate update selection command
        assert!(commands.iter().any(|cmd| matches!(cmd, InputCommand::UpdateSelection { .. })));
    }

    #[test]
    fn test_mouse_scroll() {
        let mut engine = create_test_sill_engine();
        let mut scroll_event = create_test_mouse_event(100, 50, MouseButton::WheelUp, MouseEventType::Scroll);
        scroll_event.scroll_delta = (0.0, 3.0);
        
        let commands = engine.process_mouse_event(scroll_event).unwrap();
        
        // Should generate scroll command
        assert!(commands.iter().any(|cmd| matches!(cmd, InputCommand::Scroll { .. })));
    }

    #[test]
    fn test_mouse_double_click() {
        let mut engine = create_test_sill_engine();
        let mut double_click_event = create_test_mouse_event(100, 50, MouseButton::Left, MouseEventType::Press);
        double_click_event.click_count = 2;
        
        let commands = engine.process_mouse_event(double_click_event).unwrap();
        
        // Should generate word selection
        if let Some(InputCommand::StartSelection { mode, .. }) = commands.first() {
            assert_eq!(*mode, SelectionMode::Word);
        } else {
            panic!("Expected StartSelection command");
        }
    }

    // ========== Selection Tests ==========

    #[test]
    fn test_selection_lifecycle() {
        let mut engine = create_test_sill_engine();
        let start_pos = SelectionPosition { row: 5, col: 10 };
        let end_pos = SelectionPosition { row: 5, col: 20 };
        
        // Start selection
        assert!(engine.start_selection(start_pos, SelectionMode::Character).is_ok());
        assert!(engine.get_selection().is_some());
        
        // Update selection
        assert!(engine.update_selection(end_pos).is_ok());
        let selection = engine.get_selection().unwrap();
        assert_eq!(selection.end, end_pos);
        
        // End selection
        let ended_selection = engine.end_selection().unwrap();
        assert!(ended_selection.is_some());
        
        // Clear selection
        assert!(engine.clear_selection().is_ok());
        assert!(engine.get_selection().is_none());
    }

    #[test]
    fn test_selection_bounds() {
        let selection = Selection {
            start: SelectionPosition { row: 10, col: 5 },
            end: SelectionPosition { row: 8, col: 15 },
            mode: SelectionMode::Character,
            pane_id: Some(PaneId::new(1)),
            timestamp: Instant::now(),
            active: true,
        };
        
        let (start, end) = selection.bounds();
        assert_eq!(start, SelectionPosition { row: 8, col: 15 });
        assert_eq!(end, SelectionPosition { row: 10, col: 5 });
    }

    #[test]
    fn test_selection_contains() {
        let selection = Selection {
            start: SelectionPosition { row: 5, col: 10 },
            end: SelectionPosition { row: 5, col: 20 },
            mode: SelectionMode::Character,
            pane_id: Some(PaneId::new(1)),
            timestamp: Instant::now(),
            active: true,
        };
        
        assert!(selection.contains(SelectionPosition { row: 5, col: 15 }));
        assert!(!selection.contains(SelectionPosition { row: 5, col: 25 }));
        assert!(!selection.contains(SelectionPosition { row: 6, col: 15 }));
    }

    // ========== Clipboard Tests ==========

    #[test]
    fn test_clipboard_operations() {
        let mut engine = create_test_sill_engine();
        
        // Create a selection
        let start_pos = SelectionPosition { row: 0, col: 0 };
        let end_pos = SelectionPosition { row: 0, col: 10 };
        assert!(engine.start_selection(start_pos, SelectionMode::Character).is_ok());
        assert!(engine.update_selection(end_pos).is_ok());
        
        // Copy should work with selection
        let copy_result = engine.clipboard_copy();
        assert!(copy_result.is_ok());
        
        // Paste should return the copied text
        let paste_result = engine.clipboard_paste();
        assert!(paste_result.is_ok());
    }

    #[test]
    fn test_clipboard_without_selection() {
        let mut engine = create_test_sill_engine();
        
        // Copy without selection should fail
        let copy_result = engine.clipboard_copy();
        assert!(copy_result.is_err());
        assert!(matches!(copy_result.unwrap_err(), SillError::NoSelection(_)));
    }

    // ========== Configuration Tests ==========

    #[test]
    fn test_config_validation() {
        let config = SillConfig::default();
        let validation = config.validate();
        assert!(validation.is_valid());
        assert!(validation.errors.is_empty());
    }

    #[test]
    fn test_invalid_config_validation() {
        let mut config = SillConfig::default();
        config.keyboard.repeat_delay = Duration::from_millis(0); // Invalid
        config.mouse.scroll_speed = -1.0; // Invalid
        
        let validation = config.validate();
        assert!(!validation.is_valid());
        assert!(!validation.errors.is_empty());
    }

    #[test]
    fn test_config_presets() {
        let gaming_config = ConfigPresets::gaming();
        assert!(gaming_config.validate().is_valid());
        assert!(gaming_config.performance.max_input_rate > 1000.0);
        
        let server_config = ConfigPresets::server();
        assert!(server_config.validate().is_valid());
        assert!(!server_config.clipboard.use_system_clipboard);
        
        let embedded_config = ConfigPresets::embedded();
        assert!(embedded_config.validate().is_valid());
        assert!(embedded_config.clipboard.buffer_size < 10);
    }

    #[test]
    fn test_config_builder() {
        let config = SillConfigBuilder::new()
            .enable_debug_mode()
            .enable_performance_optimization()
            .build()
            .unwrap();
        
        assert!(config.debug.enable_logging);
        assert!(config.performance.max_input_rate > 1000.0);
    }

    // ========== Performance Tests ==========

    #[test]
    fn test_performance_tracking() {
        let mut engine = create_test_sill_engine();
        let event = create_test_key_event(65, Some('A'));
        
        // Process some events
        for _ in 0..10 {
            let _ = engine.process_key_event(event.clone());
        }
        
        let stats = engine.get_input_statistics();
        assert_eq!(stats.keys_processed, 10);
        assert!(stats.commands_generated > 0);
    }

    #[test]
    fn test_performance_metrics() {
        let mut tracker = InputPerformanceTracker::new();
        
        tracker.start_input_processing();
        std::thread::sleep(Duration::from_micros(100));
        tracker.end_input_processing();
        
        assert_eq!(tracker.total_events(), 1);
        assert!(tracker.peak_processing_time() >= Duration::from_micros(100));
    }

    // ========== Error Handling Tests ==========

    #[test]
    fn test_error_creation() {
        let error = SillError::key_processing("Test error");
        assert!(matches!(error, SillError::KeyProcessing(_)));
        assert!(error.is_recoverable());
        assert_eq!(error.severity(), ErrorSeverity::Warning);
    }

    #[test]
    fn test_error_recovery_suggestions() {
        let error = SillError::clipboard_access("Permission denied");
        assert!(error.recovery_suggestion().is_some());
        
        let internal_error = SillError::internal("Critical failure");
        assert!(internal_error.recovery_suggestion().is_none());
    }

    #[test]
    fn test_error_reporter() {
        let mut reporter = ErrorReporter::new(5);
        let error = SillError::key_processing("Test error");
        let context = ErrorContext::new("test".to_string(), "keyboard".to_string());
        let contextual_error = ContextualError::new(error, context);
        
        reporter.report(contextual_error);
        
        let stats = reporter.statistics();
        assert_eq!(stats.total_errors, 1);
        assert_eq!(stats.unique_types, 1);
        
        let recent = reporter.recent_errors(10);
        assert_eq!(recent.len(), 1);
    }

    // ========== Event System Tests ==========

    #[test]
    fn test_event_handler() {
        let mut handler = SillEventHandler::new();
        let debug_listener = Box::new(DebugEventListener::new(true));
        
        handler.register_listener(SillEventType::KeyProcessed, debug_listener);
        assert_eq!(handler.listener_count(SillEventType::KeyProcessed), 1);
        
        let event = SillEvent::KeyProcessed {
            event: KeyEvent {
                key: Key::Character('A'),
                character: Some('A'),
                modifiers: Modifiers::default(),
                state: KeyState::Press,
                timestamp: Instant::now(),
                input_mode: InputMode::Normal,
            },
            commands_generated: 1,
            processing_time: Duration::from_micros(100),
        };
        
        assert!(handler.emit(event).is_ok());
    }

    #[test]
    fn test_event_statistics() {
        let mut stats = EventStatistics::new();
        
        stats.record_event(&SillEventType::KeyProcessed);
        stats.record_event(&SillEventType::MouseProcessed);
        stats.record_event(&SillEventType::KeyProcessed);
        
        assert_eq!(stats.total_events(), 3);
        assert_eq!(stats.event_count(SillEventType::KeyProcessed), 2);
        assert_eq!(stats.event_count(SillEventType::MouseProcessed), 1);
    }

    // ========== Integration Tests ==========

    #[test]
    fn test_full_input_processing_pipeline() {
        let mut engine = create_test_sill_engine();
        engine.set_focus(Some(PaneId::new(1))).unwrap();
        
        // Process a complete input sequence
        let key_event = create_test_key_event(72, Some('H'));
        let commands = engine.process_key_event(key_event).unwrap();
        
        assert!(!commands.is_empty());
        assert!(commands.iter().any(|cmd| matches!(cmd, InputCommand::InsertText { .. })));
        
        // Check that metrics were updated
        let stats = engine.get_input_statistics();
        assert!(stats.keys_processed > 0);
        assert!(stats.commands_generated > 0);
    }

    #[test]
    fn test_mouse_and_keyboard_interaction() {
        let mut engine = create_test_sill_engine();
        
        // Start mouse selection
        let mouse_press = create_test_mouse_event(50, 25, MouseButton::Left, MouseEventType::Press);
        let mouse_commands = engine.process_mouse_event(mouse_press).unwrap();
        assert!(mouse_commands.iter().any(|cmd| matches!(cmd, InputCommand::StartSelection { .. })));
        
        // Process keyboard input while selection is active
        let key_event = create_test_key_event(67, Some('C'));
        let key_commands = engine.process_key_event(key_event).unwrap();
        assert!(key_commands.iter().any(|cmd| matches!(cmd, InputCommand::InsertText { .. })));
        
        // Verify selection is still active
        assert!(engine.get_selection().is_some());
    }

    // ========== Mock Interface Tests ==========

    #[cfg(test)]
    #[test]
    fn test_mock_sill_interface() {
        let mut mock = interface::MockSillInterface::new()
            .with_clipboard_content("test content".to_string())
            .with_focus(PaneId::new(1));
        
        // Test key processing
        let key_event = interface::test_utils::create_key_event(65, Some('A'));
        let commands = mock.process_key_event(key_event).unwrap();
        assert!(!commands.is_empty());
        
        // Test mouse processing
        let mouse_event = interface::test_utils::create_mouse_event(50, 25, MouseButton::Left, MouseEventType::Press);
        let mouse_commands = mock.process_mouse_event(mouse_event).unwrap();
        assert!(!mouse_commands.is_empty());
        
        // Test clipboard
        let paste_result = mock.clipboard_paste().unwrap();
        assert_eq!(paste_result, "test content");
        
        // Test selection
        assert!(mock.start_selection(SelectionPosition { row: 0, col: 0 }, SelectionMode::Character).is_ok());
        assert!(mock.get_selection().is_some());
        
        // Test statistics
        let stats = mock.get_input_statistics();
        assert_eq!(stats.keys_processed, 1);
        assert_eq!(stats.mouse_events_processed, 1);
    }
}