use super::*;
use crate::sash::PaneId;
use std::collections::HashMap;

/// Glazing-specific events
#[derive(Debug, Clone)]
pub enum GlazingEvent {
    // Rendering events
    FrameRendered { duration: std::time::Duration, frame_count: u64 },
    RenderError { error: String, recoverable: bool },
    PerformanceWarning { fps: f64, frame_time: std::time::Duration },
    
    // Theme events
    ThemeChanged { theme_name: String },
    ThemeLoadError { theme_name: String, error: String },
    ColorSchemeChanged { scheme: String },
    
    // Layout events
    LayoutChanged { pane_count: usize, layout_type: String },
    PaneResized { pane_id: PaneId, new_size: (u16, u16) },
    ViewportChanged { scroll_offset: usize, visible_lines: usize },
    
    // Font events
    FontChanged { family: String, size: f32 },
    FontLoadError { font_name: String, error: String },
    FontFallback { original: String, fallback: String },
    
    // Cursor events
    CursorMoved { pane_id: PaneId, position: (u16, u16) },
    CursorStyleChanged { style: crate::pane::CursorStyle },
    CursorBlinkStateChanged { visible: bool },
    
    // Configuration events
    ConfigChanged { component: String },
    ConfigValidationFailed { errors: Vec<String> },
    
    // Performance events
    MemoryUsageChanged { current: usize, peak: usize },
    CacheStatsChanged { hit_rate: f64, size: usize },
    GpuStatusChanged { available: bool, vendor: String },
    
    // User interaction events
    ScrollEvent { direction: ScrollDirection, amount: usize },
    ZoomEvent { factor: f32 },
    
    // Backend events
    BackendChanged { backend_type: String },
    BackendError { error: String },
    
    // Debug events
    DebugInfoRequested,
    DebugModeToggled { enabled: bool },
}

/// Event type enumeration for subscription
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlazingEventType {
    FrameRendered,
    RenderError,
    PerformanceWarning,
    ThemeChanged,
    ThemeLoadError,
    ColorSchemeChanged,
    LayoutChanged,
    PaneResized,
    ViewportChanged,
    FontChanged,
    FontLoadError,
    FontFallback,
    CursorMoved,
    CursorStyleChanged,
    CursorBlinkStateChanged,
    ConfigChanged,
    ConfigValidationFailed,
    MemoryUsageChanged,
    CacheStatsChanged,
    GpuStatusChanged,
    ScrollEvent,
    ZoomEvent,
    BackendChanged,
    BackendError,
    DebugInfoRequested,
    DebugModeToggled,
    All, // Subscribe to all events
}

impl From<&GlazingEvent> for GlazingEventType {
    fn from(event: &GlazingEvent) -> Self {
        match event {
            GlazingEvent::FrameRendered { .. } => GlazingEventType::FrameRendered,
            GlazingEvent::RenderError { .. } => GlazingEventType::RenderError,
            GlazingEvent::PerformanceWarning { .. } => GlazingEventType::PerformanceWarning,
            GlazingEvent::ThemeChanged { .. } => GlazingEventType::ThemeChanged,
            GlazingEvent::ThemeLoadError { .. } => GlazingEventType::ThemeLoadError,
            GlazingEvent::ColorSchemeChanged { .. } => GlazingEventType::ColorSchemeChanged,
            GlazingEvent::LayoutChanged { .. } => GlazingEventType::LayoutChanged,
            GlazingEvent::PaneResized { .. } => GlazingEventType::PaneResized,
            GlazingEvent::ViewportChanged { .. } => GlazingEventType::ViewportChanged,
            GlazingEvent::FontChanged { .. } => GlazingEventType::FontChanged,
            GlazingEvent::FontLoadError { .. } => GlazingEventType::FontLoadError,
            GlazingEvent::FontFallback { .. } => GlazingEventType::FontFallback,
            GlazingEvent::CursorMoved { .. } => GlazingEventType::CursorMoved,
            GlazingEvent::CursorStyleChanged { .. } => GlazingEventType::CursorStyleChanged,
            GlazingEvent::CursorBlinkStateChanged { .. } => GlazingEventType::CursorBlinkStateChanged,
            GlazingEvent::ConfigChanged { .. } => GlazingEventType::ConfigChanged,
            GlazingEvent::ConfigValidationFailed { .. } => GlazingEventType::ConfigValidationFailed,
            GlazingEvent::MemoryUsageChanged { .. } => GlazingEventType::MemoryUsageChanged,
            GlazingEvent::CacheStatsChanged { .. } => GlazingEventType::CacheStatsChanged,
            GlazingEvent::GpuStatusChanged { .. } => GlazingEventType::GpuStatusChanged,
            GlazingEvent::ScrollEvent { .. } => GlazingEventType::ScrollEvent,
            GlazingEvent::ZoomEvent { .. } => GlazingEventType::ZoomEvent,
            GlazingEvent::BackendChanged { .. } => GlazingEventType::BackendChanged,
            GlazingEvent::BackendError { .. } => GlazingEventType::BackendError,
            GlazingEvent::DebugInfoRequested => GlazingEventType::DebugInfoRequested,
            GlazingEvent::DebugModeToggled { .. } => GlazingEventType::DebugModeToggled,
        }
    }
}

/// Event listener trait for glazing events
pub trait GlazingEventListener: Send + Sync {
    /// Handle a glazing event
    fn handle_glazing_event(&mut self, event: &GlazingEvent) -> GlazingResult<()>;
    
    /// Check if this listener can handle a specific event type
    fn can_handle(&self, event_type: GlazingEventType) -> bool;
    
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

/// Glazing event handler
pub struct GlazingEventHandler {
    listeners: HashMap<GlazingEventType, Vec<PrioritizedGlazingListener>>,
    event_queue: Vec<GlazingEvent>,
    dispatching: bool,
    stats: EventHandlerStats,
}

struct PrioritizedGlazingListener {
    listener: Box<dyn GlazingEventListener>,
    priority: EventPriority,
}

impl GlazingEventHandler {
    /// Create a new event handler
    pub fn new() -> Self {
        GlazingEventHandler {
            listeners: HashMap::new(),
            event_queue: Vec::new(),
            dispatching: false,
            stats: EventHandlerStats::new(),
        }
    }
    
    /// Register an event listener
    pub fn register_listener(&mut self, event_type: GlazingEventType, listener: Box<dyn GlazingEventListener>) {
        let priority = listener.priority();
        let prioritized = PrioritizedGlazingListener { listener, priority };
        
        self.listeners
            .entry(event_type)
            .or_default()
            .push(prioritized);
        
        // Sort by priority (high to low)
        if let Some(listeners) = self.listeners.get_mut(&event_type) {
            listeners.sort_by(|a, b| b.priority.cmp(&a.priority));
        }
        
        self.stats.total_listeners += 1;
    }
    
    /// Dispatch an event to all registered listeners
    pub fn dispatch(&mut self, event: GlazingEvent) -> GlazingResult<()> {
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
    fn dispatch_immediate(&mut self, event: &GlazingEvent) -> GlazingResult<()> {
        let event_type = GlazingEventType::from(event);
        self.stats.events_dispatched += 1;
        
        let start_time = std::time::Instant::now();
        
        // Dispatch to specific event type listeners
        if let Some(listeners) = self.listeners.get_mut(&event_type) {
            for prioritized in listeners {
                if prioritized.listener.can_handle(event_type) {
                    if let Err(e) = prioritized.listener.handle_glazing_event(event) {
                        self.stats.listener_errors += 1;
                        eprintln!("Glazing event listener error: {}", e);
                    }
                }
            }
        }
        
        // Dispatch to "All" event listeners
        if let Some(all_listeners) = self.listeners.get_mut(&GlazingEventType::All) {
            for prioritized in all_listeners {
                if prioritized.listener.can_handle(event_type) {
                    if let Err(e) = prioritized.listener.handle_glazing_event(event) {
                        self.stats.listener_errors += 1;
                        eprintln!("Glazing event listener error: {}", e);
                    }
                }
            }
        }
        
        let dispatch_time = start_time.elapsed();
        self.stats.total_dispatch_time += dispatch_time;
        
        if dispatch_time > std::time::Duration::from_millis(10) {
            self.stats.slow_dispatches += 1;
        }
        
        Ok(())
    }
    
    /// Get the number of listeners for an event type
    pub fn listener_count(&self, event_type: GlazingEventType) -> usize {
        self.listeners.get(&event_type).map_or(0, |listeners| listeners.len())
    }
    
    /// Check if there are listeners for an event type
    pub fn has_listeners(&self, event_type: GlazingEventType) -> bool {
        self.listener_count(event_type) > 0 || self.listener_count(GlazingEventType::All) > 0
    }
    
    /// Clear all listeners
    pub fn clear(&mut self) {
        self.listeners.clear();
        self.event_queue.clear();
        self.stats = EventHandlerStats::new();
    }
    
    /// Remove all listeners for a specific event type
    pub fn clear_listeners(&mut self, event_type: GlazingEventType) {
        if let Some(listeners) = self.listeners.remove(&event_type) {
            self.stats.total_listeners -= listeners.len();
        }
    }
    
    /// Get event handler statistics
    pub fn get_stats(&self) -> &EventHandlerStats {
        &self.stats
    }
}

impl Default for GlazingEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Event handler statistics
#[derive(Debug, Clone)]
pub struct EventHandlerStats {
    pub total_listeners: usize,
    pub events_dispatched: u64,
    pub listener_errors: u64,
    pub slow_dispatches: u64,
    pub total_dispatch_time: std::time::Duration,
}

impl EventHandlerStats {
    fn new() -> Self {
        EventHandlerStats {
            total_listeners: 0,
            events_dispatched: 0,
            listener_errors: 0,
            slow_dispatches: 0,
            total_dispatch_time: std::time::Duration::new(0, 0),
        }
    }
    
    /// Get average dispatch time per event
    pub fn average_dispatch_time(&self) -> std::time::Duration {
        if self.events_dispatched > 0 {
            self.total_dispatch_time / self.events_dispatched as u32
        } else {
            std::time::Duration::new(0, 0)
        }
    }
    
    /// Get error rate as percentage
    pub fn error_rate(&self) -> f64 {
        if self.events_dispatched > 0 {
            (self.listener_errors as f64 / self.events_dispatched as f64) * 100.0
        } else {
            0.0
        }
    }
}

/// Trait for objects that can emit glazing events
pub trait GlazingEventEmitter {
    /// Emit a glazing event
    fn emit_glazing_event(&mut self, event: GlazingEvent) -> GlazingResult<()>;
    
    /// Register a glazing event listener
    fn register_glazing_listener(&mut self, event_type: GlazingEventType, listener: Box<dyn GlazingEventListener>);
}

/// Built-in event listeners

/// Performance monitoring listener
pub struct PerformanceMonitor {
    fps_threshold: f64,
    frame_time_threshold: std::time::Duration,
}

impl PerformanceMonitor {
    pub fn new(fps_threshold: f64, frame_time_threshold: std::time::Duration) -> Self {
        PerformanceMonitor {
            fps_threshold,
            frame_time_threshold,
        }
    }
}

impl GlazingEventListener for PerformanceMonitor {
    fn handle_glazing_event(&mut self, event: &GlazingEvent) -> GlazingResult<()> {
        match event {
            GlazingEvent::FrameRendered { duration, .. } => {
                if *duration > self.frame_time_threshold {
                    eprintln!("Warning: Slow frame render time: {:?}", duration);
                }
            }
            GlazingEvent::PerformanceWarning { fps, frame_time } => {
                if *fps < self.fps_threshold {
                    eprintln!("Warning: Low FPS: {:.1}", fps);
                }
                if *frame_time > self.frame_time_threshold {
                    eprintln!("Warning: High frame time: {:?}", frame_time);
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    fn can_handle(&self, event_type: GlazingEventType) -> bool {
        matches!(event_type, 
            GlazingEventType::FrameRendered | 
            GlazingEventType::PerformanceWarning
        )
    }
    
    fn priority(&self) -> EventPriority {
        EventPriority::High
    }
}

/// Debug information listener
pub struct DebugListener {
    enabled: bool,
}

impl DebugListener {
    pub fn new(enabled: bool) -> Self {
        DebugListener { enabled }
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl GlazingEventListener for DebugListener {
    fn handle_glazing_event(&mut self, event: &GlazingEvent) -> GlazingResult<()> {
        if !self.enabled {
            return Ok(());
        }
        
        match event {
            GlazingEvent::DebugInfoRequested => {
                println!("=== Glazing Debug Information ===");
                // TODO: Print debug information
            }
            GlazingEvent::DebugModeToggled { enabled } => {
                self.enabled = *enabled;
                println!("Debug mode {}", if *enabled { "enabled" } else { "disabled" });
            }
            _ => {
                if self.enabled {
                    println!("DEBUG: {:?}", event);
                }
            }
        }
        Ok(())
    }
    
    fn can_handle(&self, _event_type: GlazingEventType) -> bool {
        true // Handle all events when debug mode is enabled
    }
    
    fn priority(&self) -> EventPriority {
        EventPriority::Low // Debug listener has low priority
    }
}