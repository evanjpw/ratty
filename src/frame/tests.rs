#[cfg(test)]
mod frame_tests {
    use super::*;
    use crate::frame::{
        Frame, FrameInterface, SashId, GlobalCommand, GlobalEvent, GlobalEventType,
        ApplicationState, GlobalConfig, WindowConfig, EventListener, ListenerId,
        EventError
    };

    // Test helper functions
    fn create_test_config() -> GlobalConfig {
        GlobalConfig::default()
    }

    fn create_test_window_config() -> WindowConfig {
        WindowConfig {
            size: (800, 600),
            position: Some((100, 100)),
            theme: Some("test_theme".to_string()),
            font_family: Some("test_font".to_string()),
            font_size: Some(14),
            restore_session: false,
        }
    }

    // Mock implementations for testing
    struct TestEventListener {
        id: ListenerId,
        handled_events: std::sync::Arc<std::sync::Mutex<Vec<GlobalEvent>>>,
    }

    impl TestEventListener {
        fn new(id: u64) -> Self {
            Self {
                id: ListenerId(id),
                handled_events: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            }
        }

        fn get_handled_events(&self) -> Vec<GlobalEvent> {
            self.handled_events.lock().unwrap().clone()
        }
    }

    impl EventListener for TestEventListener {
        fn handle_event(&mut self, event: &GlobalEvent) -> Result<(), EventError> {
            self.handled_events.lock().unwrap().push(event.clone());
            Ok(())
        }

        fn can_handle(&self, _event_type: GlobalEventType) -> bool {
            true
        }

        fn listener_id(&self) -> ListenerId {
            self.id
        }
    }

    // Frame lifecycle tests
    #[test]
    fn test_frame_initialization() {
        let frame = Frame::initialize();
        assert!(frame.is_ok(), "Frame initialization should succeed");
        
        let frame = frame.unwrap();
        assert_eq!(*frame.get_application_state(), ApplicationState::Running);
        assert_eq!(frame.window_count(), 0);
        assert_eq!(frame.get_active_window(), None);
    }

    #[test]
    fn test_frame_shutdown() {
        let mut frame = Frame::initialize().unwrap();
        
        // Create a window to test cleanup
        let _window_id = frame.create_window().unwrap();
        assert_eq!(frame.window_count(), 1);
        
        let result = frame.shutdown();
        assert!(result.is_ok(), "Frame shutdown should succeed");
        
        // After shutdown, should not continue
        assert!(!frame.should_continue());
    }

    #[test]
    fn test_frame_should_continue() {
        let frame = Frame::initialize().unwrap();
        assert!(frame.should_continue(), "New frame should continue running");
    }

    // Window management tests
    #[test]
    fn test_create_single_window() {
        let mut frame = Frame::initialize().unwrap();
        
        let window_id = frame.create_window().unwrap();
        assert_eq!(frame.window_count(), 1);
        assert_eq!(frame.get_active_window(), Some(window_id));
        
        let windows = frame.list_windows();
        assert_eq!(windows.len(), 1);
        assert!(windows.contains(&window_id));
    }

    #[test]
    fn test_create_multiple_windows() {
        let mut frame = Frame::initialize().unwrap();
        
        let window1 = frame.create_window().unwrap();
        let window2 = frame.create_window().unwrap();
        let window3 = frame.create_window().unwrap();
        
        assert_eq!(frame.window_count(), 3);
        assert_eq!(frame.get_active_window(), Some(window1)); // First window stays active
        
        let windows = frame.list_windows();
        assert!(windows.contains(&window1));
        assert!(windows.contains(&window2));
        assert!(windows.contains(&window3));
    }

    #[test]
    fn test_create_window_with_config() {
        let mut frame = Frame::initialize().unwrap();
        let config = create_test_window_config();
        
        let window_id = frame.create_window_with_config(Some(config)).unwrap();
        assert_eq!(frame.window_count(), 1);
        assert_eq!(frame.get_active_window(), Some(window_id));
    }

    #[test]
    fn test_destroy_window() {
        let mut frame = Frame::initialize().unwrap();
        
        let window1 = frame.create_window().unwrap();
        let window2 = frame.create_window().unwrap();
        
        assert_eq!(frame.window_count(), 2);
        
        let result = frame.destroy_window(window2);
        assert!(result.is_ok());
        assert_eq!(frame.window_count(), 1);
        
        let windows = frame.list_windows();
        assert!(windows.contains(&window1));
        assert!(!windows.contains(&window2));
    }

    #[test]
    fn test_destroy_nonexistent_window() {
        let mut frame = Frame::initialize().unwrap();
        
        let nonexistent_id = SashId::new(999);
        let result = frame.destroy_window(nonexistent_id);
        
        assert!(result.is_err());
        assert_eq!(frame.window_count(), 0);
    }

    #[test]
    fn test_destroy_active_window_updates_focus() {
        let mut frame = Frame::initialize().unwrap();
        
        let window1 = frame.create_window().unwrap();
        let window2 = frame.create_window().unwrap();
        
        // Set window2 as active
        frame.set_active_window(window2).unwrap();
        assert_eq!(frame.get_active_window(), Some(window2));
        
        // Destroy active window
        frame.destroy_window(window2).unwrap();
        
        // Active window should update to remaining window
        assert_eq!(frame.get_active_window(), Some(window1));
    }

    #[test]
    fn test_destroy_last_window() {
        let mut frame = Frame::initialize().unwrap();
        
        let window_id = frame.create_window().unwrap();
        assert_eq!(frame.get_active_window(), Some(window_id));
        
        frame.destroy_window(window_id).unwrap();
        
        assert_eq!(frame.window_count(), 0);
        assert_eq!(frame.get_active_window(), None);
    }

    // Focus management tests
    #[test]
    fn test_set_active_window() {
        let mut frame = Frame::initialize().unwrap();
        
        let window1 = frame.create_window().unwrap();
        let window2 = frame.create_window().unwrap();
        
        assert_eq!(frame.get_active_window(), Some(window1));
        
        let result = frame.set_active_window(window2);
        assert!(result.is_ok());
        assert_eq!(frame.get_active_window(), Some(window2));
    }

    #[test]
    fn test_set_active_nonexistent_window() {
        let mut frame = Frame::initialize().unwrap();
        frame.create_window().unwrap();
        
        let nonexistent_id = SashId::new(999);
        let result = frame.set_active_window(nonexistent_id);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_focus_next_window() {
        let mut frame = Frame::initialize().unwrap();
        
        let window1 = frame.create_window().unwrap();
        let window2 = frame.create_window().unwrap();
        let window3 = frame.create_window().unwrap();
        
        assert_eq!(frame.get_active_window(), Some(window1));
        
        frame.focus_next_window().unwrap();
        assert_eq!(frame.get_active_window(), Some(window2));
        
        frame.focus_next_window().unwrap();
        assert_eq!(frame.get_active_window(), Some(window3));
        
        // Should wrap around
        frame.focus_next_window().unwrap();
        assert_eq!(frame.get_active_window(), Some(window1));
    }

    #[test]
    fn test_focus_previous_window() {
        let mut frame = Frame::initialize().unwrap();
        
        let window1 = frame.create_window().unwrap();
        let window2 = frame.create_window().unwrap();
        let window3 = frame.create_window().unwrap();
        
        assert_eq!(frame.get_active_window(), Some(window1));
        
        frame.focus_previous_window().unwrap();
        assert_eq!(frame.get_active_window(), Some(window3)); // Should wrap to last
        
        frame.focus_previous_window().unwrap();
        assert_eq!(frame.get_active_window(), Some(window2));
        
        frame.focus_previous_window().unwrap();
        assert_eq!(frame.get_active_window(), Some(window1));
    }

    #[test]
    fn test_focus_navigation_empty_frame() {
        let mut frame = Frame::initialize().unwrap();
        
        // Should not fail with no windows
        let result1 = frame.focus_next_window();
        let result2 = frame.focus_previous_window();
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert_eq!(frame.get_active_window(), None);
    }

    #[test]
    fn test_focus_navigation_single_window() {
        let mut frame = Frame::initialize().unwrap();
        let window_id = frame.create_window().unwrap();
        
        frame.focus_next_window().unwrap();
        assert_eq!(frame.get_active_window(), Some(window_id));
        
        frame.focus_previous_window().unwrap();
        assert_eq!(frame.get_active_window(), Some(window_id));
    }

    // Event system tests
    #[test]
    fn test_register_event_listener() {
        let mut frame = Frame::initialize().unwrap();
        let listener = Box::new(TestEventListener::new(1));
        
        let listener_id = frame.register_event_listener(
            GlobalEventType::ApplicationLifecycle, 
            listener
        );
        
        assert_eq!(listener_id.0, 1);
    }

    #[test]
    fn test_emit_event() {
        let mut frame = Frame::initialize().unwrap();
        
        let test_listener = TestEventListener::new(1);
        let events_ref = test_listener.handled_events.clone();
        let listener = Box::new(test_listener);
        
        frame.register_event_listener(
            GlobalEventType::ApplicationLifecycle,
            listener
        );
        
        let test_event = GlobalEvent::ApplicationStarted;
        let result = frame.emit_event(test_event.clone());
        
        assert!(result.is_ok());
        
        let collected_events = events_ref.lock().unwrap().clone();
        assert_eq!(collected_events.len(), 1);
        assert_eq!(collected_events[0], test_event);
    }

    #[test]
    fn test_unregister_event_listener() {
        let mut frame = Frame::initialize().unwrap();
        let listener = Box::new(TestEventListener::new(1));
        
        let listener_id = frame.register_event_listener(
            GlobalEventType::ApplicationLifecycle,
            listener
        );
        
        let removed = frame.unregister_event_listener(
            GlobalEventType::ApplicationLifecycle,
            listener_id
        );
        
        assert!(removed);
    }

    // Command system tests
    #[test]
    fn test_execute_command_with_no_handlers() {
        let mut frame = Frame::initialize().unwrap();
        
        let command = GlobalCommand::NewWindow;
        let result = frame.execute_command(command);
        
        // Should fail because no command handlers are registered
        assert!(result.is_err());
    }

    #[test]
    fn test_can_execute_command() {
        let frame = Frame::initialize().unwrap();
        
        let command = GlobalCommand::NewWindow;
        let can_execute = frame.can_execute_command(&command);
        
        // With current implementation, should return true
        assert!(can_execute);
    }

    #[test]
    fn test_available_commands() {
        let frame = Frame::initialize().unwrap();
        
        let commands = frame.available_commands();
        
        // Current implementation returns empty vec
        assert_eq!(commands.len(), 0);
    }

    // Configuration tests
    #[test]
    fn test_get_global_config() {
        let frame = Frame::initialize().unwrap();
        
        let config = frame.get_global_config();
        
        // Should return default configuration
        assert_eq!(config.default_theme, "default");
        assert!(config.allow_multiple_windows);
    }

    #[test]
    fn test_update_global_config() {
        let mut frame = Frame::initialize().unwrap();
        
        let mut new_config = GlobalConfig::default();
        new_config.default_theme = "dark".to_string();
        new_config.allow_multiple_windows = false;
        
        let result = frame.update_global_config(new_config.clone());
        assert!(result.is_ok());
        
        let updated_config = frame.get_global_config();
        assert_eq!(updated_config.default_theme, "dark");
        assert!(!updated_config.allow_multiple_windows);
    }

    #[test]
    fn test_update_invalid_config() {
        let mut frame = Frame::initialize().unwrap();
        
        let mut invalid_config = GlobalConfig::default();
        invalid_config.default_font.size = 0; // Invalid font size
        
        let result = frame.update_global_config(invalid_config);
        assert!(result.is_err());
    }

    #[test]
    fn test_save_config() {
        let frame = Frame::initialize().unwrap();
        
        let result = frame.save_config();
        
        // Current implementation is a no-op, so should succeed
        assert!(result.is_ok());
    }

    // State validation tests
    #[test]
    fn test_validate_state() {
        let frame = Frame::initialize().unwrap();
        
        let result = frame.validate_state();
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_statistics() {
        let mut frame = Frame::initialize().unwrap();
        
        let stats = frame.get_statistics();
        assert_eq!(stats.window_count, 0);
        assert_eq!(stats.active_window, None);
        
        // Create a window and check stats update
        let window_id = frame.create_window().unwrap();
        let stats = frame.get_statistics();
        assert_eq!(stats.window_count, 1);
        assert_eq!(stats.active_window, Some(window_id));
    }

    #[test]
    fn test_get_application_state() {
        let frame = Frame::initialize().unwrap();
        
        let state = frame.get_application_state();
        assert_eq!(*state, ApplicationState::Running);
    }

    // Window lifecycle tests
    #[test]
    fn test_window_creation_emits_events() {
        let mut frame = Frame::initialize().unwrap();
        
        let test_listener = TestEventListener::new(1);
        let events_ref = test_listener.handled_events.clone();
        let listener = Box::new(test_listener);
        
        frame.register_event_listener(
            GlobalEventType::WindowManagement,
            listener
        );
        
        let window_id = frame.create_window().unwrap();
        
        let events = events_ref.lock().unwrap().clone();
        
        // Should have at least the WindowCreated event
        let has_created_event = events.iter().any(|e| {
            matches!(e, GlobalEvent::WindowCreated(id) if *id == window_id)
        });
        
        assert!(has_created_event);
    }

    #[test]
    fn test_window_destruction_emits_events() {
        let mut frame = Frame::initialize().unwrap();
        
        let window_id = frame.create_window().unwrap();
        
        let test_listener = TestEventListener::new(1);
        let events_ref = test_listener.handled_events.clone();
        let listener = Box::new(test_listener);
        
        frame.register_event_listener(
            GlobalEventType::WindowManagement,
            listener
        );
        
        frame.destroy_window(window_id).unwrap();
        
        let events = events_ref.lock().unwrap().clone();
        
        // Should have the WindowDestroyed event
        let has_destroyed_event = events.iter().any(|e| {
            matches!(e, GlobalEvent::WindowDestroyed(id) if *id == window_id)
        });
        
        assert!(has_destroyed_event);
    }
}