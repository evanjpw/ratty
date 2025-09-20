use super::*;
use std::collections::HashMap;
use std::time::Instant;

/// Sill-specific event types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SillEventType {
    KeyProcessed,
    MouseProcessed,
    ClipboardOperation,
    SelectionChanged,
    FocusChanged,
    InputModeChanged,
    PerformanceWarning,
    ConfigurationChanged,
    ErrorOccurred,
}

/// Events emitted by the Sill layer
#[derive(Debug, Clone)]
pub enum SillEvent {
    KeyProcessed {
        event: KeyEvent,
        commands_generated: usize,
        processing_time: Duration,
    },
    MouseProcessed {
        event: MouseEvent,
        commands_generated: usize,
        processing_time: Duration,
    },
    ClipboardOperation {
        operation: ClipboardOperation,
        text_length: usize,
        success: bool,
    },
    SelectionChanged {
        selection: Option<Selection>,
        mode: SelectionMode,
        pane_id: Option<PaneId>,
    },
    FocusChanged {
        old_focus: Option<PaneId>,
        new_focus: Option<PaneId>,
    },
    InputModeChanged {
        old_mode: InputMode,
        new_mode: InputMode,
    },
    PerformanceWarning {
        metric: PerformanceMetric,
        value: f64,
        threshold: f64,
    },
    ConfigurationChanged {
        component: String,
        changes: Vec<String>,
    },
    ErrorOccurred {
        error: SillError,
        context: ErrorContext,
    },
}

/// Clipboard operations for event tracking
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClipboardOperation {
    Copy,
    Paste,
    Cut,
    Clear,
}

/// Performance metrics for monitoring
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PerformanceMetric {
    InputLatency,
    ProcessingTime,
    EventQueueSize,
    MemoryUsage,
    CommandGenerationRate,
}

/// Event listener trait for Sill events
pub trait SillEventListener: Send + Sync {
    /// Handle a Sill event
    fn handle_sill_event(&mut self, event: &SillEvent) -> SillResult<()>;
    
    /// Check if this listener can handle a specific event type
    fn can_handle(&self, event_type: SillEventType) -> bool;
    
    /// Get listener priority (higher priority listeners are called first)
    fn priority(&self) -> u8 {
        0
    }
}

/// Event handler for managing Sill events
pub struct SillEventHandler {
    listeners: HashMap<SillEventType, Vec<Box<dyn SillEventListener>>>,
    event_stats: EventStatistics,
}

impl SillEventHandler {
    pub fn new() -> Self {
        SillEventHandler {
            listeners: HashMap::new(),
            event_stats: EventStatistics::new(),
        }
    }
    
    /// Register a listener for specific event types
    pub fn register_listener(
        &mut self,
        event_type: SillEventType,
        listener: Box<dyn SillEventListener>,
    ) {
        self.listeners
            .entry(event_type.clone())
            .or_insert_with(Vec::new)
            .push(listener);
        
        // Sort listeners by priority (highest first)
        if let Some(listeners) = self.listeners.get_mut(&event_type) {
            listeners.sort_by(|a, b| b.priority().cmp(&a.priority()));
        }
    }
    
    /// Emit a Sill event to all registered listeners
    pub fn emit(&mut self, event: SillEvent) -> SillResult<()> {
        let event_type = self.get_event_type(&event);
        self.event_stats.record_event(&event_type);
        
        if let Some(listeners) = self.listeners.get_mut(&event_type) {
            for listener in listeners.iter_mut() {
                if let Err(e) = listener.handle_sill_event(&event) {
                    // Log error but continue with other listeners
                    eprintln!("Event listener error: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Emit key processed event
    pub fn emit_key_processed(&mut self, key_event: &KeyEvent) -> SillResult<()> {
        self.emit(SillEvent::KeyProcessed {
            event: key_event.clone(),
            commands_generated: 1, // TODO: Track actual count
            processing_time: Duration::from_micros(100), // TODO: Track actual time
        })
    }
    
    /// Emit mouse processed event
    pub fn emit_mouse_processed(&mut self, mouse_event: &MouseEvent) -> SillResult<()> {
        self.emit(SillEvent::MouseProcessed {
            event: mouse_event.clone(),
            commands_generated: 1, // TODO: Track actual count
            processing_time: Duration::from_micros(50), // TODO: Track actual time
        })
    }
    
    /// Emit clipboard operation event
    pub fn emit_clipboard_operation(&mut self, operation: ClipboardOperation, text: &str) -> SillResult<()> {
        self.emit(SillEvent::ClipboardOperation {
            operation,
            text_length: text.len(),
            success: true, // TODO: Track actual success
        })
    }
    
    /// Emit selection changed event
    pub fn emit_selection_changed(
        &mut self,
        selection: Option<Selection>,
        mode: SelectionMode,
        pane_id: Option<PaneId>,
    ) -> SillResult<()> {
        self.emit(SillEvent::SelectionChanged {
            selection,
            mode,
            pane_id,
        })
    }
    
    /// Emit focus changed event
    pub fn emit_focus_changed(&mut self, new_focus: Option<PaneId>) -> SillResult<()> {
        self.emit(SillEvent::FocusChanged {
            old_focus: None, // TODO: Track previous focus
            new_focus,
        })
    }
    
    /// Emit input mode changed event
    pub fn emit_input_mode_changed(&mut self, old_mode: InputMode, new_mode: InputMode) -> SillResult<()> {
        self.emit(SillEvent::InputModeChanged { old_mode, new_mode })
    }
    
    /// Emit performance warning event
    pub fn emit_performance_warning(
        &mut self,
        metric: PerformanceMetric,
        value: f64,
        threshold: f64,
    ) -> SillResult<()> {
        self.emit(SillEvent::PerformanceWarning {
            metric,
            value,
            threshold,
        })
    }
    
    /// Emit configuration changed event
    pub fn emit_configuration_changed(&mut self, component: String, changes: Vec<String>) -> SillResult<()> {
        self.emit(SillEvent::ConfigurationChanged { component, changes })
    }
    
    /// Emit error occurred event
    pub fn emit_error(&mut self, error: SillError, context: ErrorContext) -> SillResult<()> {
        self.emit(SillEvent::ErrorOccurred { error, context })
    }
    
    /// Get number of listeners for an event type
    pub fn listener_count(&self, event_type: SillEventType) -> usize {
        self.listeners
            .get(&event_type)
            .map(|listeners| listeners.len())
            .unwrap_or(0)
    }
    
    /// Check if there are listeners for an event type
    pub fn has_listeners(&self, event_type: SillEventType) -> bool {
        self.listener_count(event_type) > 0
    }
    
    /// Get event statistics
    pub fn get_stats(&self) -> &EventStatistics {
        &self.event_stats
    }
    
    /// Clear all listeners
    pub fn clear_listeners(&mut self) {
        self.listeners.clear();
    }
    
    /// Get event type from event instance
    fn get_event_type(&self, event: &SillEvent) -> SillEventType {
        match event {
            SillEvent::KeyProcessed { .. } => SillEventType::KeyProcessed,
            SillEvent::MouseProcessed { .. } => SillEventType::MouseProcessed,
            SillEvent::ClipboardOperation { .. } => SillEventType::ClipboardOperation,
            SillEvent::SelectionChanged { .. } => SillEventType::SelectionChanged,
            SillEvent::FocusChanged { .. } => SillEventType::FocusChanged,
            SillEvent::InputModeChanged { .. } => SillEventType::InputModeChanged,
            SillEvent::PerformanceWarning { .. } => SillEventType::PerformanceWarning,
            SillEvent::ConfigurationChanged { .. } => SillEventType::ConfigurationChanged,
            SillEvent::ErrorOccurred { .. } => SillEventType::ErrorOccurred,
        }
    }
}

/// Event statistics for monitoring
#[derive(Debug, Clone)]
pub struct EventStatistics {
    total_events: u64,
    events_by_type: HashMap<SillEventType, u64>,
    last_event_time: Option<Instant>,
}

impl EventStatistics {
    pub fn new() -> Self {
        EventStatistics {
            total_events: 0,
            events_by_type: HashMap::new(),
            last_event_time: None,
        }
    }
    
    /// Record an event occurrence*
    pub fn record_event(&mut self, event_type: &SillEventType) {
        self.total_events += 1;
        *self.events_by_type.entry(event_type.to_owned()).or_insert(0) += 1;
        self.last_event_time = Some(Instant::now());
    }
    
    /// Get total number of events
    pub fn total_events(&self) -> u64 {
        self.total_events
    }
    
    /// Get event count for a specific type
    pub fn event_count(&self, event_type: SillEventType) -> u64 {
        self.events_by_type.get(&event_type).copied().unwrap_or(0)
    }
    
    /// Get time since last event
    pub fn time_since_last_event(&self) -> Option<Duration> {
        self.last_event_time.map(|time| time.elapsed())
    }
}

/// Debug event listener for development
#[derive(Debug)]
pub struct DebugEventListener {
    enabled: bool,
    verbose: bool,
}

impl DebugEventListener {
    pub fn new(enabled: bool) -> Self {
        DebugEventListener {
            enabled,
            verbose: false,
        }
    }
    
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

impl SillEventListener for DebugEventListener {
    fn handle_sill_event(&mut self, event: &SillEvent) -> SillResult<()> {
        if !self.enabled {
            return Ok(());
        }
        
        match event {
            SillEvent::KeyProcessed { event, commands_generated, processing_time } => {
                if self.verbose {
                    println!("Key processed: {:?} -> {} commands in {:?}", event, commands_generated, processing_time);
                } else {
                    println!("Key: {:?}", event.key);
                }
            }
            SillEvent::MouseProcessed { event, commands_generated, processing_time } => {
                if self.verbose {
                    println!("Mouse processed: {:?} -> {} commands in {:?}", event, commands_generated, processing_time);
                } else {
                    println!("Mouse: {:?} at {:?}", event.event_type, event.position);
                }
            }
            SillEvent::ClipboardOperation { operation, text_length, success } => {
                println!("Clipboard {:?}: {} chars, success: {}", operation, text_length, success);
            }
            SillEvent::SelectionChanged { selection, mode, pane_id } => {
                println!("Selection changed: {:?} mode in pane {:?}", mode, pane_id);
                if self.verbose {
                    if let Some(sel) = selection {
                        println!("  Selection: {:?} to {:?}", sel.start, sel.end);
                    }
                }
            }
            SillEvent::FocusChanged { old_focus, new_focus } => {
                println!("Focus: {:?} -> {:?}", old_focus, new_focus);
            }
            SillEvent::InputModeChanged { old_mode, new_mode } => {
                println!("Input mode: {:?} -> {:?}", old_mode, new_mode);
            }
            SillEvent::PerformanceWarning { metric, value, threshold } => {
                println!("Performance warning: {:?} = {} (threshold: {})", metric, value, threshold);
            }
            SillEvent::ConfigurationChanged { component, changes } => {
                println!("Config changed in {}: {:?}", component, changes);
            }
            SillEvent::ErrorOccurred { error, context } => {
                println!("Error in {}: {}", context.component, error);
            }
        }
        
        Ok(())
    }
    
    fn can_handle(&self, _event_type: SillEventType) -> bool {
        self.enabled
    }
    
    fn priority(&self) -> u8 {
        255 // Highest priority for debugging
    }
}

/// Performance monitoring event listener
#[derive(Debug)]
pub struct PerformanceMonitor {
    latency_threshold: Duration,
    processing_threshold: Duration,
    warnings_sent: u64,
}

impl PerformanceMonitor {
    pub fn new(latency_threshold_ms: f64, processing_threshold_ms: f64) -> Self {
        PerformanceMonitor {
            latency_threshold: Duration::from_millis(latency_threshold_ms as u64),
            processing_threshold: Duration::from_millis(processing_threshold_ms as u64),
            warnings_sent: 0,
        }
    }
}

impl SillEventListener for PerformanceMonitor {
    fn handle_sill_event(&mut self, event: &SillEvent) -> SillResult<()> {
        match event {
            SillEvent::KeyProcessed { processing_time, .. } => {
                if *processing_time > self.processing_threshold {
                    self.warnings_sent += 1;
                    println!("Performance warning: Key processing took {:?}", processing_time);
                }
            }
            SillEvent::MouseProcessed { processing_time, .. } => {
                if *processing_time > self.processing_threshold {
                    self.warnings_sent += 1;
                    println!("Performance warning: Mouse processing took {:?}", processing_time);
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    fn can_handle(&self, event_type: SillEventType) -> bool {
        matches!(event_type, 
            SillEventType::KeyProcessed | 
            SillEventType::MouseProcessed |
            SillEventType::PerformanceWarning
        )
    }
}

impl Default for SillEventHandler {
    fn default() -> Self {
        Self::new()
    }
}