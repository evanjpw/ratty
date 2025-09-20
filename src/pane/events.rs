use super::*;
use std::collections::HashMap;

/// Pane-specific events
#[derive(Debug, Clone)]
pub enum PaneEvent {
    // Process events
    ProcessSpawned(u32),
    ProcessExited(i32),
    ProcessKilled,
    
    // Content events
    ContentChanged(ContentRegion),
    TitleChanged(String),
    CursorMoved(u16, u16),
    
    // Terminal events
    Resized(u16, u16),
    ModeChanged(TerminalMode),
    BellRung,
    
    // User interaction events
    TextSelected(Selection),
    SearchResultsChanged(Vec<SearchMatch>),
    
    // Error events
    PtyError(String), // Convert to String to make it cloneable
    TerminalError(String),
    
    // State events
    ActiveStateChanged(bool),
    ModifiedStateChanged(bool),
    
    // Performance events
    PerformanceWarning(PerformanceIssue),
}

/// Content region for change notifications
#[derive(Debug, Clone)]
pub enum ContentRegion {
    Screen,
    Scrollback,
    Line(usize),
    Cell(usize, usize),
    Range { start_line: usize, end_line: usize },
}

/// Text selection information
#[derive(Debug, Clone)]
pub struct Selection {
    pub start: SelectionPoint,
    pub end: SelectionPoint,
    pub text: String,
    pub selection_type: SelectionType,
}

#[derive(Debug, Clone)]
pub struct SelectionPoint {
    pub line: usize,
    pub column: usize,
    pub buffer_type: BufferType,
}

#[derive(Debug, Clone, Copy)]
pub enum SelectionType {
    Character,
    Word,
    Line,
    Block,
}

/// Performance issue types
#[derive(Debug, Clone)]
pub enum PerformanceIssue {
    SlowRendering { duration: std::time::Duration },
    HighMemoryUsage { bytes: usize },
    LargeOutput { bytes: usize },
    FrequentUpdates { updates_per_second: f32 },
}

/// Event type enumeration for subscription
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaneEventType {
    ProcessSpawned,
    ProcessExited,
    ProcessKilled,
    ContentChanged,
    TitleChanged,
    CursorMoved,
    Resized,
    ModeChanged,
    BellRung,
    TextSelected,
    SearchResultsChanged,
    PtyError,
    TerminalError,
    ActiveStateChanged,
    ModifiedStateChanged,
    PerformanceWarning,
    All, // Subscribe to all events
}

impl From<&PaneEvent> for PaneEventType {
    fn from(event: &PaneEvent) -> Self {
        match event {
            PaneEvent::ProcessSpawned(_) => PaneEventType::ProcessSpawned,
            PaneEvent::ProcessExited(_) => PaneEventType::ProcessExited,
            PaneEvent::ProcessKilled => PaneEventType::ProcessKilled,
            PaneEvent::ContentChanged(_) => PaneEventType::ContentChanged,
            PaneEvent::TitleChanged(_) => PaneEventType::TitleChanged,
            PaneEvent::CursorMoved(_, _) => PaneEventType::CursorMoved,
            PaneEvent::Resized(_, _) => PaneEventType::Resized,
            PaneEvent::ModeChanged(_) => PaneEventType::ModeChanged,
            PaneEvent::BellRung => PaneEventType::BellRung,
            PaneEvent::TextSelected(_) => PaneEventType::TextSelected,
            PaneEvent::SearchResultsChanged(_) => PaneEventType::SearchResultsChanged,
            PaneEvent::PtyError(_) => PaneEventType::PtyError,
            PaneEvent::TerminalError(_) => PaneEventType::TerminalError,
            PaneEvent::ActiveStateChanged(_) => PaneEventType::ActiveStateChanged,
            PaneEvent::ModifiedStateChanged(_) => PaneEventType::ModifiedStateChanged,
            PaneEvent::PerformanceWarning(_) => PaneEventType::PerformanceWarning,
        }
    }
}

/// Event listener trait for pane events
pub trait PaneEventListener: Send + Sync {
    /// Handle a pane event
    fn handle_pane_event(&mut self, event: &PaneEvent) -> PaneResult<()>;
    
    /// Check if this listener can handle a specific event type
    fn can_handle(&self, event_type: PaneEventType) -> bool;
    
    /// Get listener priority (higher priority listeners are called first)
    fn priority(&self) -> EventPriority { EventPriority::Normal }
}

/// Event priority for listener ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Pane event handler
pub struct PaneEventHandler {
    listeners: HashMap<PaneEventType, Vec<PrioritizedListener>>,
    event_queue: Vec<PaneEvent>,
    dispatching: bool,
}

struct PrioritizedListener {
    listener: Box<dyn PaneEventListener>,
    priority: EventPriority,
}

impl PaneEventHandler {
    /// Create a new event handler
    pub fn new() -> Self {
        PaneEventHandler {
            listeners: HashMap::new(),
            event_queue: Vec::new(),
            dispatching: false,
        }
    }
    
    /// Register an event listener
    pub fn register_listener(&mut self, event_type: PaneEventType, listener: Box<dyn PaneEventListener>) {
        let priority = listener.priority();
        let prioritized = PrioritizedListener { listener, priority };
        
        self.listeners
            .entry(event_type)
            .or_default()
            .push(prioritized);
        
        // Sort by priority (high to low)
        if let Some(listeners) = self.listeners.get_mut(&event_type) {
            listeners.sort_by(|a, b| b.priority.cmp(&a.priority));
        }
    }
    
    /// Dispatch an event to all registered listeners
    pub fn dispatch(&mut self, event: PaneEvent) -> PaneResult<()> {
        if self.dispatching {
            // Avoid recursion by queuing events during dispatch
            self.event_queue.push(event);
            return Ok(());
        }
        
        self.dispatching = true;
        let result = self.dispatch_immediate(&event);
        
        // Process any queued events
        while let Some(queued_event) = self.event_queue.pop() {
            let _ = self.dispatch_immediate(&queued_event);
        }
        
        self.dispatching = false;
        result
    }
    
    /// Dispatch event immediately to listeners
    fn dispatch_immediate(&mut self, event: &PaneEvent) -> PaneResult<()> {
        let event_type = PaneEventType::from(event);
        
        // Dispatch to specific event type listeners
        if let Some(listeners) = self.listeners.get_mut(&event_type) {
            for prioritized in listeners {
                if prioritized.listener.can_handle(event_type) {
                    if let Err(e) = prioritized.listener.handle_pane_event(event) {
                        // Log error but continue with other listeners
                        eprintln!("Event listener error: {}", e);
                    }
                }
            }
        }
        
        // Dispatch to "All" event listeners
        if let Some(all_listeners) = self.listeners.get_mut(&PaneEventType::All) {
            for prioritized in all_listeners {
                if prioritized.listener.can_handle(event_type) {
                    if let Err(e) = prioritized.listener.handle_pane_event(event) {
                        eprintln!("Event listener error: {}", e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Get the number of listeners for an event type
    pub fn listener_count(&self, event_type: PaneEventType) -> usize {
        self.listeners.get(&event_type).map_or(0, |listeners| listeners.len())
    }
    
    /// Check if there are listeners for an event type
    pub fn has_listeners(&self, event_type: PaneEventType) -> bool {
        self.listener_count(event_type) > 0 || self.listener_count(PaneEventType::All) > 0
    }
    
    /// Clear all listeners
    pub fn clear(&mut self) {
        self.listeners.clear();
        self.event_queue.clear();
    }
    
    /// Remove all listeners for a specific event type
    pub fn clear_listeners(&mut self, event_type: PaneEventType) {
        self.listeners.remove(&event_type);
    }
}

impl Default for PaneEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for objects that can emit pane events
pub trait PaneEventEmitter {
    /// Emit a pane event
    fn emit_pane_event(&mut self, event: PaneEvent) -> PaneResult<()>;
    
    /// Register a pane event listener
    fn register_pane_listener(&mut self, event_type: PaneEventType, listener: Box<dyn PaneEventListener>);
}

/// Search direction for content search
#[derive(Debug, Clone, Copy)]
pub enum SearchDirection {
    Forward,
    Backward,
}

/// Scroll position for navigation
#[derive(Debug, Clone, Copy)]
pub enum ScrollPosition {
    Top,
    Bottom,
    Line(usize),
    Relative(i32), // Relative to current position
}

/// Statistics about a pane for monitoring and debugging
#[derive(Debug, Clone)]
pub struct PaneStatistics {
    pub bytes_received: usize,
    pub bytes_sent: usize,
    pub lines_processed: usize,
    pub commands_executed: usize,
    pub unhandled_sequences: usize,
    pub render_count: usize,
    pub last_activity: Option<std::time::Instant>,
    pub creation_time: std::time::Instant,
    pub process_spawn_time: Option<std::time::Instant>,
    pub memory_usage: usize, // Estimated memory usage in bytes
}

impl Default for PaneStatistics {
    fn default() -> Self {
        PaneStatistics {
            bytes_received: 0,
            bytes_sent: 0,
            lines_processed: 0,
            commands_executed: 0,
            unhandled_sequences: 0,
            render_count: 0,
            last_activity: None,
            creation_time: std::time::Instant::now(),
            process_spawn_time: None,
            memory_usage: 0,
        }
    }
}

impl PaneStatistics {
    /// Create new statistics with current timestamp
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Update activity timestamp
    pub fn record_activity(&mut self) {
        self.last_activity = Some(std::time::Instant::now());
    }
    
    /// Record process spawn
    pub fn record_spawn(&mut self) {
        self.process_spawn_time = Some(std::time::Instant::now());
    }
    
    /// Get uptime duration
    pub fn uptime(&self) -> std::time::Duration {
        self.creation_time.elapsed()
    }
    
    /// Get time since last activity
    pub fn idle_time(&self) -> Option<std::time::Duration> {
        self.last_activity.map(|last| last.elapsed())
    }
    
    /// Calculate throughput (bytes per second)
    pub fn throughput(&self) -> f64 {
        let uptime_secs = self.uptime().as_secs_f64();
        if uptime_secs > 0.0 {
            self.bytes_received as f64 / uptime_secs
        } else {
            0.0
        }
    }
    
    /// Check if pane appears to be active
    pub fn is_active(&self) -> bool {
        self.idle_time()
            .map(|idle| idle < std::time::Duration::from_secs(60))
            .unwrap_or(false)
    }
}