use super::{
    Layout, PaneConfig, PaneId, SashEvent, SashEventListener, SashEventType,
    SashResult, SplitDirection, Theme, WindowConfig
};
use crate::frame::SashId;

/// Statistics about a Sash
#[derive(Debug, Clone)]
pub struct SashStatistics {
    pub pane_count: usize,
    pub tab_count: usize,
    pub active_pane: Option<PaneId>,
    pub active_tab: Option<usize>,
    pub layout_type: String,
    pub memory_usage: usize,
}

/// Primary interface for the Sash layer
pub trait SashInterface: Send + Sync {
    // ========== Identity and State ==========
    
    /// Get the Sash ID
    fn id(&self) -> SashId;
    
    /// Check if this Sash is active
    fn is_active(&self) -> bool;
    
    /// Set the active state
    fn set_active(&mut self, active: bool);
    
    // ========== Pane Management ==========
    
    /// Create a new pane with default configuration
    fn create_pane(&mut self) -> SashResult<PaneId>;
    
    /// Create a new pane with specific configuration
    fn create_pane_with_config(&mut self, config: PaneConfig) -> SashResult<PaneId>;
    
    /// Destroy a pane
    fn destroy_pane(&mut self, pane_id: PaneId) -> SashResult<()>;
    
    // TODO: Implement pane access methods later
    // fn get_pane(&self, pane_id: PaneId) -> Option<&dyn super::PaneInterface>;
    // fn get_pane_mut(&mut self, pane_id: PaneId) -> Option<&mut dyn super::PaneInterface>;
    
    /// List all pane IDs
    fn list_panes(&self) -> Vec<PaneId>;
    
    /// Get the number of panes
    fn pane_count(&self) -> usize;
    
    // ========== Focus Management ==========
    
    /// Set the active pane
    fn set_active_pane(&mut self, pane_id: PaneId) -> SashResult<()>;
    
    /// Get the currently active pane
    fn get_active_pane(&self) -> Option<PaneId>;
    
    /// Focus the next pane
    fn focus_next_pane(&mut self) -> SashResult<()>;
    
    /// Focus the previous pane
    fn focus_previous_pane(&mut self) -> SashResult<()>;
    
    // ========== Tab Management ==========
    
    /// Create a new tab
    fn new_tab(&mut self) -> SashResult<PaneId>;
    
    /// Close a tab by index
    fn close_tab(&mut self, index: usize) -> SashResult<()>;
    
    /// Close the current tab
    fn close_current_tab(&mut self) -> SashResult<()>;
    
    /// Navigate to the next tab
    fn next_tab(&mut self) -> SashResult<()>;
    
    /// Navigate to the previous tab
    fn previous_tab(&mut self) -> SashResult<()>;
    
    /// Move a tab from one position to another
    fn move_tab(&mut self, from: usize, to: usize) -> SashResult<()>;
    
    /// Get the number of tabs
    fn get_tab_count(&self) -> usize;
    
    /// Get the active tab index
    fn get_active_tab(&self) -> Option<usize>;
    
    // ========== Split Management ==========
    
    /// Split the current pane horizontally
    fn split_horizontal(&mut self) -> SashResult<PaneId>;
    
    /// Split the current pane vertically
    fn split_vertical(&mut self) -> SashResult<PaneId>;
    
    /// Split a specific pane
    fn split_pane(&mut self, pane_id: PaneId, direction: SplitDirection) -> SashResult<PaneId>;
    
    /// Close a split (remove pane and adjust layout)
    fn close_split(&mut self, pane_id: PaneId) -> SashResult<()>;
    
    /// Resize a split
    fn resize_split(&mut self, pane_id: PaneId, ratio: f32) -> SashResult<()>;
    
    // ========== Layout Management ==========
    
    /// Set the layout
    fn set_layout(&mut self, layout: Layout) -> SashResult<()>;
    
    /// Get the current layout
    fn get_layout(&self) -> &Layout;
    
    /// Save the current layout with a name
    fn save_layout(&mut self, name: String) -> SashResult<()>;
    
    /// Load a saved layout
    fn load_layout(&mut self, name: String) -> SashResult<()>;
    
    /// List all saved layout names
    fn list_saved_layouts(&self) -> Vec<String>;
    
    // ========== Theme and Configuration ==========
    
    /// Set the theme
    fn set_theme(&mut self, theme: Theme) -> SashResult<()>;
    
    /// Get the current theme
    fn get_theme(&self) -> &Theme;
    
    /// Update the window configuration
    fn update_config(&mut self, config: WindowConfig) -> SashResult<()>;
    
    /// Get the window configuration
    fn get_config(&self) -> &WindowConfig;
    
    // ========== Event Handling ==========
    
    /// Register an event listener
    fn register_event_listener(&mut self, event_type: SashEventType, listener: Box<dyn SashEventListener>);
    
    /// Emit an event
    fn emit_event(&mut self, event: SashEvent) -> SashResult<()>;
    
    // ========== State and Statistics ==========
    
    /// Get statistics about the Sash
    fn get_statistics(&self) -> SashStatistics;
    
    /// Validate the internal state
    fn validate_state(&self) -> SashResult<()>;
}