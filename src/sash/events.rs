use super::{Layout, PaneId, SashError, SashResult};
use crate::sash::config::FontConfig;
use crate::sash::layout::SplitDirection;
use crate::sash::theme::ColorScheme;
use std::collections::HashMap;

/// Events that can occur within a Sash
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
    TabMoved { from: usize, to: usize },
    TabTitleChanged(usize, String),
    
    // Layout events
    LayoutChanged(Layout),
    SplitCreated(SplitDirection, PaneId),
    SplitRemoved(PaneId),
    SplitResized { pane_id: PaneId, ratio: f32 },
    
    // Theme events
    ThemeChanged(String),
    ColorsChanged(ColorScheme),
    FontChanged(FontConfig),
    
    // Window events
    WindowResized { width: u16, height: u16 },
    WindowFocusChanged(bool),
}

/// Types of events for categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SashEventType {
    Pane,
    Tab,
    Layout,
    Theme,
    Window,
}

impl From<&SashEvent> for SashEventType {
    fn from(event: &SashEvent) -> Self {
        match event {
            SashEvent::PaneCreated(_) | SashEvent::PaneDestroyed(_) |
            SashEvent::PaneFocused(_) | SashEvent::PaneUnfocused(_) |
            SashEvent::PaneModified(_, _) | SashEvent::PaneTitleChanged(_, _) => SashEventType::Pane,
            
            SashEvent::TabAdded(_, _) | SashEvent::TabRemoved(_, _) |
            SashEvent::TabActivated(_) | SashEvent::TabMoved { .. } |
            SashEvent::TabTitleChanged(_, _) => SashEventType::Tab,
            
            SashEvent::LayoutChanged(_) | SashEvent::SplitCreated(_, _) |
            SashEvent::SplitRemoved(_) | SashEvent::SplitResized { .. } => SashEventType::Layout,
            
            SashEvent::ThemeChanged(_) | SashEvent::ColorsChanged(_) |
            SashEvent::FontChanged(_) => SashEventType::Theme,
            
            SashEvent::WindowResized { .. } | SashEvent::WindowFocusChanged(_) => SashEventType::Window,
        }
    }
}

/// Trait for handling sash events
#[cfg_attr(test, mockall::automock)]
pub trait SashEventListener: Send + Sync {
    fn handle_sash_event(&mut self, event: &SashEvent) -> SashResult<()>;
    fn can_handle(&self, event_type: SashEventType) -> bool;
}

/// Manages event listeners and dispatching
pub struct SashEventHandler {
    listeners: HashMap<SashEventType, Vec<Box<dyn SashEventListener>>>,
    event_queue: Vec<SashEvent>,
    dispatching: bool,
}

impl SashEventHandler {
    /// Create a new event handler
    pub fn new() -> Self {
        SashEventHandler {
            listeners: HashMap::new(),
            event_queue: Vec::new(),
            dispatching: false,
        }
    }
    
    /// Register an event listener
    pub fn register_listener(&mut self, event_type: SashEventType, listener: Box<dyn SashEventListener>) {
        self.listeners
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(listener);
    }
    
    /// Dispatch an event to registered listeners
    pub fn dispatch(&mut self, event: SashEvent) -> SashResult<()> {
        // Queue event if already dispatching to prevent recursion
        if self.dispatching {
            self.event_queue.push(event);
            return Ok(());
        }
        
        self.dispatching = true;
        let result = self.dispatch_immediate(&event);
        
        // Process queued events
        while let Some(queued_event) = self.event_queue.pop() {
            if let Err(e) = self.dispatch_immediate(&queued_event) {
                eprintln!("Error dispatching queued event: {}", e);
            }
        }
        
        self.dispatching = false;
        result
    }
    
    /// Dispatch an event immediately
    fn dispatch_immediate(&mut self, event: &SashEvent) -> SashResult<()> {
        let event_type = SashEventType::from(event);
        
        if let Some(listeners) = self.listeners.get_mut(&event_type) {
            let mut errors = Vec::new();
            
            for listener in listeners {
                if listener.can_handle(event_type) {
                    if let Err(e) = listener.handle_sash_event(event) {
                        errors.push(e);
                    }
                }
            }
            
            // Return first error if any occurred
            if let Some(first_error) = errors.into_iter().next() {
                return Err(first_error);
            }
        }
        
        Ok(())
    }
    
    /// Get the number of listeners for an event type
    pub fn listener_count(&self, event_type: SashEventType) -> usize {
        self.listeners.get(&event_type).map_or(0, |l| l.len())
    }
    
    /// Check if there are any listeners for an event type
    pub fn has_listeners(&self, event_type: SashEventType) -> bool {
        self.listener_count(event_type) > 0
    }
    
    /// Clear all listeners
    pub fn clear(&mut self) {
        self.listeners.clear();
        self.event_queue.clear();
    }
}

impl Default for SashEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper trait for objects that can emit sash events
pub trait SashEventEmitter {
    fn emit_event(&mut self, event: SashEvent) -> SashResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_from_event() {
        let pane_event = SashEvent::PaneCreated(PaneId::new(1));
        assert_eq!(SashEventType::from(&pane_event), SashEventType::Pane);
        
        let tab_event = SashEvent::TabActivated(0);
        assert_eq!(SashEventType::from(&tab_event), SashEventType::Tab);
        
        let layout_event = SashEvent::LayoutChanged(Layout::Empty);
        assert_eq!(SashEventType::from(&layout_event), SashEventType::Layout);
        
        let theme_event = SashEvent::ThemeChanged("dark".to_string());
        assert_eq!(SashEventType::from(&theme_event), SashEventType::Theme);
        
        let window_event = SashEvent::WindowResized { width: 800, height: 600 };
        assert_eq!(SashEventType::from(&window_event), SashEventType::Window);
    }
}