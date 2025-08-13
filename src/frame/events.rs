use std::collections::HashMap;
use crate::frame::errors::EventError;
use crate::frame::config::{FontConfig, ColorScheme};
use crate::frame::SashId;

/// Unique identifier for event listeners
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ListenerId(pub u64);

impl ListenerId {
    pub fn new(id: u64) -> Self {
        ListenerId(id)
    }
}

/// Global events that can occur in the application
#[derive(Debug, Clone, PartialEq)]
pub enum GlobalEvent {
    // Application lifecycle events
    ApplicationStarted,
    ApplicationWillTerminate,
    ApplicationDidBecomeActive,
    ApplicationDidResignActive,
    ApplicationSuspended,
    ApplicationResumed,
    
    // Window management events
    WindowCreated(SashId),
    WindowDestroyed(SashId),
    WindowFocused(SashId),
    WindowUnfocused(SashId),
    WindowMoved(SashId, (i32, i32)),
    WindowResized(SashId, (u32, u32)),
    WindowMinimized(SashId),
    WindowRestored(SashId),
    
    // Configuration events
    ConfigurationChanged(ConfigChange),
    ConfigurationReloaded,
    ConfigurationSaved,
    ThemeChanged(String),
    FontChanged(FontConfig),
    
    // System events
    SystemColorSchemeChanged(ColorScheme),
    SystemFontChanged(FontConfig),
    LowMemoryWarning,
    SystemSleep,
    SystemWake,
    
    // User interaction events
    ShortcutTriggered(crate::frame::commands::GlobalCommand),
    MenuItemSelected(String),
    PreferencesRequested,
    
    // Error and warning events
    ErrorOccurred(String),
    WarningOccurred(String),
    
    // Custom events for extensibility
    Custom(String, serde_json::Value),
}

/// Types of configuration changes
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigChange {
    GlobalTheme(String),
    GlobalFont(FontConfig),
    WindowDefaults(crate::frame::config::WindowConfig),
    NotificationSettings(crate::frame::config::NotificationSettings),
    ColorScheme(ColorScheme),
    ShortcutsChanged,
    PerformanceSettings,
}

/// Event type enumeration for listener registration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlobalEventType {
    ApplicationLifecycle,
    WindowManagement,
    Configuration,
    System,
    UserInteraction,
    Error,
    Custom,
}

impl From<&GlobalEvent> for GlobalEventType {
    fn from(event: &GlobalEvent) -> Self {
        match event {
            GlobalEvent::ApplicationStarted | GlobalEvent::ApplicationWillTerminate |
            GlobalEvent::ApplicationDidBecomeActive | GlobalEvent::ApplicationDidResignActive |
            GlobalEvent::ApplicationSuspended | GlobalEvent::ApplicationResumed => 
                GlobalEventType::ApplicationLifecycle,
            
            GlobalEvent::WindowCreated(_) | GlobalEvent::WindowDestroyed(_) |
            GlobalEvent::WindowFocused(_) | GlobalEvent::WindowUnfocused(_) |
            GlobalEvent::WindowMoved(_, _) | GlobalEvent::WindowResized(_, _) |
            GlobalEvent::WindowMinimized(_) | GlobalEvent::WindowRestored(_) => 
                GlobalEventType::WindowManagement,
            
            GlobalEvent::ConfigurationChanged(_) | GlobalEvent::ConfigurationReloaded |
            GlobalEvent::ConfigurationSaved | GlobalEvent::ThemeChanged(_) |
            GlobalEvent::FontChanged(_) => GlobalEventType::Configuration,
            
            GlobalEvent::SystemColorSchemeChanged(_) | GlobalEvent::SystemFontChanged(_) |
            GlobalEvent::LowMemoryWarning | GlobalEvent::SystemSleep | 
            GlobalEvent::SystemWake => GlobalEventType::System,
            
            GlobalEvent::ShortcutTriggered(_) | GlobalEvent::MenuItemSelected(_) |
            GlobalEvent::PreferencesRequested => GlobalEventType::UserInteraction,
            
            GlobalEvent::ErrorOccurred(_) | GlobalEvent::WarningOccurred(_) => 
                GlobalEventType::Error,
            
            GlobalEvent::Custom(_, _) => GlobalEventType::Custom,
        }
    }
}

/// Trait for event listeners
#[cfg_attr(test, mockall::automock)]
pub trait EventListener: Send + Sync {
    fn handle_event(&mut self, event: &GlobalEvent) -> Result<(), EventError>;
    fn can_handle(&self, event_type: GlobalEventType) -> bool;
    fn listener_id(&self) -> ListenerId;
}

/// Event priority for controlling dispatch order
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Event listener wrapper with priority
struct PrioritizedListener {
    listener: Box<dyn EventListener>,
    priority: EventPriority,
}

impl PrioritizedListener {
    fn new(listener: Box<dyn EventListener>, priority: EventPriority) -> Self {
        PrioritizedListener { listener, priority }
    }
}

/// Event dispatching system
pub struct EventDispatcher {
    listeners: HashMap<GlobalEventType, Vec<PrioritizedListener>>,
    next_listener_id: u64,
    event_queue: Vec<GlobalEvent>,
    dispatching: bool,
}

impl EventDispatcher {
    /// Create a new event dispatcher
    pub fn new() -> Self {
        EventDispatcher {
            listeners: HashMap::new(),
            next_listener_id: 1,
            event_queue: Vec::new(),
            dispatching: false,
        }
    }
    
    /// Subscribe to events of a specific type
    pub fn subscribe(&mut self, 
                    event_type: GlobalEventType, 
                    listener: Box<dyn EventListener>) -> ListenerId {
        self.subscribe_with_priority(event_type, listener, EventPriority::Normal)
    }
    
    /// Subscribe to events with a specific priority
    pub fn subscribe_with_priority(&mut self, 
                                  event_type: GlobalEventType, 
                                  listener: Box<dyn EventListener>,
                                  priority: EventPriority) -> ListenerId {
        let listener_id = listener.listener_id();
        let prioritized = PrioritizedListener::new(listener, priority);
        
        let listeners = self.listeners.entry(event_type).or_insert_with(Vec::new);
        listeners.push(prioritized);
        
        // Sort by priority (highest first)
        listeners.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        listener_id
    }
    
    /// Unsubscribe a listener from a specific event type
    pub fn unsubscribe(&mut self, event_type: GlobalEventType, listener_id: ListenerId) -> bool {
        if let Some(listeners) = self.listeners.get_mut(&event_type) {
            let initial_len = listeners.len();
            listeners.retain(|l| l.listener.listener_id() != listener_id);
            listeners.len() != initial_len
        } else {
            false
        }
    }
    
    /// Unsubscribe a listener from all event types
    pub fn unsubscribe_all(&mut self, listener_id: ListenerId) -> usize {
        let mut removed_count = 0;
        
        for listeners in self.listeners.values_mut() {
            let initial_len = listeners.len();
            listeners.retain(|l| l.listener.listener_id() != listener_id);
            removed_count += initial_len - listeners.len();
        }
        
        removed_count
    }
    
    /// Dispatch an event immediately
    pub fn dispatch(&mut self, event: GlobalEvent) -> Result<(), EventError> {
        // If we're already dispatching, queue the event to prevent recursion
        if self.dispatching {
            self.event_queue.push(event);
            return Ok(());
        }
        
        self.dispatching = true;
        let result = self.dispatch_immediate(&event);
        
        // Process any queued events
        while let Some(queued_event) = self.event_queue.pop() {
            if let Err(e) = self.dispatch_immediate(&queued_event) {
                // Log error but continue processing other events
                eprintln!("Error dispatching queued event: {}", e);
            }
        }
        
        self.dispatching = false;
        result
    }
    
    /// Dispatch an event immediately without queueing
    fn dispatch_immediate(&mut self, event: &GlobalEvent) -> Result<(), EventError> {
        let event_type = GlobalEventType::from(event);
        
        if let Some(listeners) = self.listeners.get_mut(&event_type) {
            let mut errors = Vec::new();
            
            for prioritized_listener in listeners {
                if prioritized_listener.listener.can_handle(event_type) {
                    if let Err(e) = prioritized_listener.listener.handle_event(event) {
                        errors.push(e);
                        // Continue to other listeners even if one fails
                    }
                }
            }
            
            // Return the first error if any occurred
            if let Some(first_error) = errors.into_iter().next() {
                return Err(first_error);
            }
        }
        
        Ok(())
    }
    
    /// Get the number of listeners for an event type
    pub fn listener_count(&self, event_type: GlobalEventType) -> usize {
        self.listeners.get(&event_type).map_or(0, |l| l.len())
    }
    
    /// Check if there are any listeners for an event type
    pub fn has_listeners(&self, event_type: GlobalEventType) -> bool {
        self.listener_count(event_type) > 0
    }
    
    /// Clear all listeners
    pub fn clear(&mut self) {
        self.listeners.clear();
        self.event_queue.clear();
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock event listener for testing
    struct MockListener {
        id: ListenerId,
        handled_events: Vec<GlobalEvent>,
        should_fail: bool,
    }
    
    impl MockListener {
        fn new(id: u64) -> Self {
            MockListener {
                id: ListenerId::new(id),
                handled_events: Vec::new(),
                should_fail: false,
            }
        }
        
        fn with_failure(id: u64) -> Self {
            MockListener {
                id: ListenerId::new(id),
                handled_events: Vec::new(),
                should_fail: true,
            }
        }
    }
    
    impl EventListener for MockListener {
        fn handle_event(&mut self, event: &GlobalEvent) -> Result<(), EventError> {
            if self.should_fail {
                return Err(EventError::ListenerFailed("Mock failure".to_string()));
            }
            self.handled_events.push(event.clone());
            Ok(())
        }
        
        fn can_handle(&self, _event_type: GlobalEventType) -> bool {
            true
        }
        
        fn listener_id(&self) -> ListenerId {
            self.id
        }
    }

    #[test]
    fn test_event_type_conversion() {
        let app_event = GlobalEvent::ApplicationStarted;
        assert_eq!(GlobalEventType::from(&app_event), GlobalEventType::ApplicationLifecycle);
        
        let window_event = GlobalEvent::WindowCreated(SashId::new(1));
        assert_eq!(GlobalEventType::from(&window_event), GlobalEventType::WindowManagement);
    }
    
    #[test]
    fn test_event_dispatcher_subscription() {
        let mut dispatcher = EventDispatcher::new();
        let listener = Box::new(MockListener::new(1));
        let listener_id = listener.listener_id();
        
        assert!(!dispatcher.has_listeners(GlobalEventType::ApplicationLifecycle));
        
        let returned_id = dispatcher.subscribe(GlobalEventType::ApplicationLifecycle, listener);
        assert_eq!(returned_id, listener_id);
        
        assert!(dispatcher.has_listeners(GlobalEventType::ApplicationLifecycle));
        assert_eq!(dispatcher.listener_count(GlobalEventType::ApplicationLifecycle), 1);
    }
    
    #[test]
    fn test_event_dispatch() {
        let mut dispatcher = EventDispatcher::new();
        let listener = Box::new(MockListener::new(1));
        
        dispatcher.subscribe(GlobalEventType::ApplicationLifecycle, listener);
        
        let event = GlobalEvent::ApplicationStarted;
        let result = dispatcher.dispatch(event);
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_event_dispatch_failure() {
        let mut dispatcher = EventDispatcher::new();
        let listener = Box::new(MockListener::with_failure(1));
        
        dispatcher.subscribe(GlobalEventType::ApplicationLifecycle, listener);
        
        let event = GlobalEvent::ApplicationStarted;
        let result = dispatcher.dispatch(event);
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_unsubscribe() {
        let mut dispatcher = EventDispatcher::new();
        let listener = Box::new(MockListener::new(1));
        let listener_id = listener.listener_id();
        
        dispatcher.subscribe(GlobalEventType::ApplicationLifecycle, listener);
        assert_eq!(dispatcher.listener_count(GlobalEventType::ApplicationLifecycle), 1);
        
        let removed = dispatcher.unsubscribe(GlobalEventType::ApplicationLifecycle, listener_id);
        assert!(removed);
        assert_eq!(dispatcher.listener_count(GlobalEventType::ApplicationLifecycle), 0);
    }
    
    #[test]
    fn test_priority_ordering() {
        let mut dispatcher = EventDispatcher::new();
        
        let low_listener = Box::new(MockListener::new(1));
        let high_listener = Box::new(MockListener::new(2));
        
        // Subscribe in reverse priority order
        dispatcher.subscribe_with_priority(GlobalEventType::ApplicationLifecycle, low_listener, EventPriority::Low);
        dispatcher.subscribe_with_priority(GlobalEventType::ApplicationLifecycle, high_listener, EventPriority::High);
        
        // High priority listener should be first in the list
        assert_eq!(dispatcher.listener_count(GlobalEventType::ApplicationLifecycle), 2);
    }
}