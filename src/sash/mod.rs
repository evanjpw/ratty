pub mod errors;
pub mod layout;
pub mod theme;
pub mod tabs;
pub mod events;
pub mod interface;
pub mod config;

#[cfg(test)]
mod tests;

pub use config::*;
pub use errors::*;
pub use events::*;
pub use interface::*;
pub use layout::*;
pub use tabs::*;
pub use theme::*;

use crate::frame;
use crate::frame::SashId;
use crate::pane::PaneInterface;
use std::collections::HashMap;

/// Unique identifier for a Pane (terminal instance)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PaneId(pub u64);

impl PaneId {
    pub fn new(id: u64) -> Self {
        PaneId(id)
    }
    
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

// /// Placeholder trait for Pane interface - will be defined in pane layer
// #[cfg_attr(test, mockall::automock)]
// pub trait PaneInterface: Send + Sync {
//     fn id(&self) -> PaneId;
//     fn is_active(&self) -> bool;
//     fn set_active(&mut self, active: bool);
//     fn get_title(&self) -> &str;
//     fn set_title(&mut self, title: String);
//     fn is_modified(&self) -> bool;
//     fn set_modified(&mut self, modified: bool);
//     // More methods will be added as we develop the Pane layer
// }

/// The main Sash structure - represents a window containing multiple panes
pub struct Sash {
    // Identity and state
    id: SashId,
    active: bool,
    
    // Pane management
    panes: HashMap<PaneId, Box<dyn PaneInterface>>,
    active_pane_id: Option<PaneId>,
    next_pane_id: PaneId,
    
    // Layout management
    layout: Layout,
    layout_manager: LayoutManager,
    
    // Configuration
    window_config: WindowConfig,
    theme: Theme,
    
    // Event handling
    event_handler: SashEventHandler,
    
    // Tab management
    tabs: TabManager,
}

impl Sash {
    /// Create a new Sash instance
    pub fn new(id: SashId, config: WindowConfig) -> Result<Self, SashError> {
        let theme = Theme::default();
        let layout_manager = LayoutManager::new();
        let tab_config = TabConfig::default();
        let tabs = TabManager::new(tab_config);
        let event_handler = SashEventHandler::new();
        
        Ok(Sash {
            id,
            active: false,
            panes: HashMap::new(),
            active_pane_id: None,
            next_pane_id: PaneId::new(1),
            layout: Layout::Empty,
            layout_manager,
            window_config: config,
            theme,
            event_handler,
            tabs,
        })
    }
    
    /// Get the next available PaneId and increment the counter
    fn next_pane_id(&mut self) -> PaneId {
        let id = self.next_pane_id;
        self.next_pane_id = PaneId::new(self.next_pane_id.0 + 1);
        id
    }
    
    /// Check if the sash has any panes
    pub fn has_panes(&self) -> bool {
        !self.panes.is_empty()
    }
    
    /// Get the number of panes
    pub fn pane_count(&self) -> usize {
        self.panes.len()
    }
}

// Basic implementation of SashInterface for Frame compatibility
impl frame::SashInterface for Sash {
    fn id(&self) -> SashId {
        self.id
    }
    
    fn is_active(&self) -> bool {
        self.active
    }
    
    fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}

// Full implementation of our local SashInterface trait
impl SashInterface for Sash {
    // ========== Identity and State ==========
    
    fn id(&self) -> SashId {
        self.id
    }
    
    fn is_active(&self) -> bool {
        self.active
    }
    
    fn set_active(&mut self, active: bool) {
        self.active = active;
    }
    
    // ========== Pane Management ==========
    
    fn create_pane(&mut self) -> SashResult<PaneId> {
        let config = PaneConfig::default();
        self.create_pane_with_config(config)
    }
    
    fn create_pane_with_config(&mut self, _config: PaneConfig) -> SashResult<PaneId> {
        // TODO: Create actual pane when Pane layer is implemented
        // For now, just generate a new PaneId and track it
        let pane_id = self.next_pane_id();
        
        // We can't create a real pane without the Pane layer, so this is a placeholder
        // In a real implementation, we would:
        // 1. Create a new Pane instance with the config
        // 2. Add it to self.panes
        // 3. Update the layout to include the new pane
        
        // For now, return an error indicating we need the Pane layer
        Err(SashError::PaneCreationFailed(
            "Pane layer not yet implemented".to_string()
        ))
    }
    
    fn destroy_pane(&mut self, pane_id: PaneId) -> SashResult<()> {
        if !self.panes.contains_key(&pane_id) {
            return Err(SashError::PaneNotFound(pane_id));
        }
        
        // Remove from panes map
        self.panes.remove(&pane_id);
        
        // Update active pane if this was the active one
        if self.active_pane_id == Some(pane_id) {
            self.active_pane_id = None;
            // TODO: Set active to another pane if available
        }
        
        // TODO: Update layout to remove this pane
        
        Ok(())
    }
    
    fn list_panes(&self) -> Vec<PaneId> {
        self.panes.keys().copied().collect()
    }
    
    fn pane_count(&self) -> usize {
        self.panes.len()
    }
    
    // ========== Focus Management ==========
    
    fn set_active_pane(&mut self, pane_id: PaneId) -> SashResult<()> {
        if !self.panes.contains_key(&pane_id) {
            return Err(SashError::PaneNotFound(pane_id));
        }
        
        // Deactivate current pane if any
        if let Some(current_pane_id) = self.active_pane_id {
            if let Some(current_pane) = self.panes.get_mut(&current_pane_id) {
                current_pane.set_active(false);
            }
        }
        
        // Activate new pane
        if let Some(new_pane) = self.panes.get_mut(&pane_id) {
            new_pane.set_active(true);
            self.active_pane_id = Some(pane_id);
            Ok(())
        } else {
            Err(SashError::PaneNotFound(pane_id))
        }
    }
    
    fn get_active_pane(&self) -> Option<PaneId> {
        self.active_pane_id
    }
    
    fn focus_next_pane(&mut self) -> SashResult<()> {
        let pane_ids: Vec<PaneId> = self.panes.keys().copied().collect();
        if pane_ids.is_empty() {
            return Err(SashError::NoActivePane);
        }
        
        let next_pane_id = match self.active_pane_id {
            Some(current_id) => {
                let current_index = pane_ids.iter().position(|&id| id == current_id)
                    .ok_or(SashError::PaneNotFound(current_id))?;
                let next_index = (current_index + 1) % pane_ids.len();
                pane_ids[next_index]
            }
            None => pane_ids[0],
        };
        
        self.set_active_pane(next_pane_id)
    }
    
    fn focus_previous_pane(&mut self) -> SashResult<()> {
        let pane_ids: Vec<PaneId> = self.panes.keys().copied().collect();
        if pane_ids.is_empty() {
            return Err(SashError::NoActivePane);
        }
        
        let prev_pane_id = match self.active_pane_id {
            Some(current_id) => {
                let current_index = pane_ids.iter().position(|&id| id == current_id)
                    .ok_or(SashError::PaneNotFound(current_id))?;
                let prev_index = if current_index == 0 {
                    pane_ids.len() - 1
                } else {
                    current_index - 1
                };
                pane_ids[prev_index]
            }
            None => pane_ids[pane_ids.len() - 1],
        };
        
        self.set_active_pane(prev_pane_id)
    }
    
    // ========== Tab Management ==========
    
    fn new_tab(&mut self) -> SashResult<PaneId> {
        // Create a new pane first (will fail until Pane layer is implemented)
        let pane_id = self.create_pane()?;
        
        // Add tab for the pane
        let title = format!("Tab {}", self.tabs.tab_count() + 1);
        self.tabs.add_tab(pane_id, title)?;
        
        Ok(pane_id)
    }
    
    fn close_tab(&mut self, index: usize) -> SashResult<()> {
        let pane_id = self.tabs.remove_tab(index)?;
        self.destroy_pane(pane_id)?;
        Ok(())
    }
    
    fn close_current_tab(&mut self) -> SashResult<()> {
        if let Some(active_tab) = self.tabs.active_tab_index() {
            self.close_tab(active_tab)
        } else {
            Err(SashError::NoActiveTab)
        }
    }
    
    fn next_tab(&mut self) -> SashResult<()> {
        self.tabs.next_tab()
    }
    
    fn previous_tab(&mut self) -> SashResult<()> {
        self.tabs.previous_tab()
    }
    
    fn move_tab(&mut self, from: usize, to: usize) -> SashResult<()> {
        self.tabs.move_tab(from, to)
    }
    
    fn get_tab_count(&self) -> usize {
        self.tabs.tab_count()
    }
    
    fn get_active_tab(&self) -> Option<usize> {
        self.tabs.active_tab_index()
    }
    
    // ========== Split Management ==========
    
    fn split_horizontal(&mut self) -> SashResult<PaneId> {
        if let Some(active_pane_id) = self.active_pane_id {
            self.split_pane(active_pane_id, SplitDirection::Horizontal)
        } else {
            Err(SashError::NoActivePane)
        }
    }
    
    fn split_vertical(&mut self) -> SashResult<PaneId> {
        if let Some(active_pane_id) = self.active_pane_id {
            self.split_pane(active_pane_id, SplitDirection::Vertical)
        } else {
            Err(SashError::NoActivePane)
        }
    }
    
    fn split_pane(&mut self, _pane_id: PaneId, _direction: SplitDirection) -> SashResult<PaneId> {
        // TODO: Implement splitting when we have proper layout management
        // This requires:
        // 1. Creating a new pane
        // 2. Updating the layout to include the split
        // 3. Adjusting the layout tree structure
        Err(SashError::SplitFailed("Split operations not yet implemented".to_string()))
    }
    
    fn close_split(&mut self, pane_id: PaneId) -> SashResult<()> {
        // TODO: Implement when layout management is complete
        // This requires updating the layout tree to remove the split
        self.destroy_pane(pane_id)
    }
    
    fn resize_split(&mut self, _pane_id: PaneId, _ratio: f32) -> SashResult<()> {
        // TODO: Implement when layout management is complete
        Err(SashError::SplitFailed("Split resize not yet implemented".to_string()))
    }
    
    // ========== Layout Management ==========
    
    fn set_layout(&mut self, layout: Layout) -> SashResult<()> {
        self.layout_manager.set_current(layout)?;
        self.layout = self.layout_manager.current().clone();
        Ok(())
    }
    
    fn get_layout(&self) -> &Layout {
        &self.layout
    }
    
    fn save_layout(&mut self, name: String) -> SashResult<()> {
        self.layout_manager.save_layout(name)
    }
    
    fn load_layout(&mut self, name: String) -> SashResult<()> {
        self.layout_manager.load_layout(&name)?;
        self.layout = self.layout_manager.current().clone();
        Ok(())
    }
    
    fn list_saved_layouts(&self) -> Vec<String> {
        self.layout_manager.list_saved_layouts()
    }
    
    // ========== Theme and Configuration ==========
    
    fn set_theme(&mut self, theme: Theme) -> SashResult<()> {
        theme.validate()?;
        self.theme = theme;
        Ok(())
    }
    
    fn get_theme(&self) -> &Theme {
        &self.theme
    }
    
    fn update_config(&mut self, config: WindowConfig) -> SashResult<()> {
        // TODO: Validate config if needed
        self.window_config = config;
        Ok(())
    }
    
    fn get_config(&self) -> &WindowConfig {
        &self.window_config
    }
    
    // ========== Event Handling ==========
    
    fn register_event_listener(&mut self, event_type: SashEventType, listener: Box<dyn SashEventListener>) {
        self.event_handler.register_listener(event_type, listener);
    }
    
    fn emit_event(&mut self, event: SashEvent) -> SashResult<()> {
        self.event_handler.dispatch(event)
    }
    
    // ========== State and Statistics ==========
    
    fn get_statistics(&self) -> SashStatistics {
        SashStatistics {
            pane_count: self.panes.len(),
            tab_count: self.tabs.tab_count(),
            active_pane: self.active_pane_id,
            active_tab: self.tabs.active_tab_index(),
            layout_type: format!("{:?}", self.layout).split('(').next().unwrap_or("Unknown").to_string(),
            memory_usage: 0, // TODO: Calculate actual memory usage
        }
    }
    
    fn validate_state(&self) -> SashResult<()> {
        // Check that active pane exists if set
        if let Some(active_pane_id) = self.active_pane_id {
            if !self.panes.contains_key(&active_pane_id) {
                return Err(SashError::StateValidationFailed(
                    format!("Active pane {:?} does not exist", active_pane_id)
                ));
            }
        }
        
        // Check that active tab is valid
        if let Some(active_tab) = self.tabs.active_tab_index() {
            if active_tab >= self.tabs.tab_count() {
                return Err(SashError::StateValidationFailed(
                    format!("Active tab index {} is out of bounds", active_tab)
                ));
            }
        }
        
        // Validate layout
        self.layout_manager.validate_layout(&self.layout)?;
        
        // Validate theme
        self.theme.validate()?;
        
        Ok(())
    }
}