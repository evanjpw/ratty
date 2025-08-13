use super::{PaneId, SashError, SashResult, Tab};

/// Manages tabs within a Sash
pub struct TabManager {
    tabs: Vec<Tab>,
    active_tab: Option<usize>,
    next_tab_id: u64,
    tab_config: TabConfig,
}

impl TabManager {
    /// Create a new tab manager
    pub fn new(config: TabConfig) -> Self {
        TabManager {
            tabs: Vec::new(),
            active_tab: None,
            next_tab_id: 1,
            tab_config: config,
        }
    }
    
    /// Add a new tab
    pub fn add_tab(&mut self, pane_id: PaneId, title: String) -> SashResult<usize> {
        // Check max tabs limit
        if let Some(max_tabs) = self.tab_config.max_tabs {
            if self.tabs.len() >= max_tabs {
                return Err(SashError::MaxTabsExceeded(max_tabs));
            }
        }
        
        let tab = Tab::new(pane_id, title);
        
        let new_index = match self.tab_config.new_tab_position {
            NewTabPosition::End => self.tabs.len(),
            NewTabPosition::AfterCurrent => {
                match self.active_tab {
                    Some(current) => current + 1,
                    None => self.tabs.len(),
                }
            }
            NewTabPosition::Beginning => 0,
        };
        
        self.tabs.insert(new_index, tab);
        
        // Set as active if it's the first tab or if configured to do so
        if self.active_tab.is_none() {
            self.active_tab = Some(new_index);
        } else if self.tab_config.new_tab_position == NewTabPosition::AfterCurrent {
            // Adjust active tab index if needed
            if let Some(active) = self.active_tab {
                if new_index <= active {
                    self.active_tab = Some(active + 1);
                }
            }
        }
        
        Ok(new_index)
    }
    
    /// Remove a tab at the given index
    pub fn remove_tab(&mut self, index: usize) -> SashResult<PaneId> {
        if index >= self.tabs.len() {
            return Err(SashError::TabNotFound(index));
        }
        
        // Check if it's the last tab
        if self.tabs.len() == 1 && !self.tab_config.allow_no_tabs {
            return Err(SashError::CannotCloseLastTab);
        }
        
        // Check if tab is modified and confirmation is needed
        if self.tabs[index].modified && self.tab_config.confirm_close_modified {
            // In a real implementation, this would trigger a confirmation dialog
            // For now, we'll just prevent closing
            return Err(SashError::TabOperationFailed(
                "Cannot close modified tab without confirmation".to_string()
            ));
        }
        
        let removed_tab = self.tabs.remove(index);
        
        // Update active tab
        if let Some(active) = self.active_tab {
            if index == active {
                // Was the active tab, select a new one
                if self.tabs.is_empty() {
                    self.active_tab = None;
                } else if index >= self.tabs.len() {
                    self.active_tab = Some(self.tabs.len() - 1);
                } else {
                    self.active_tab = Some(index);
                }
            } else if index < active {
                // Removed tab was before active, adjust index
                self.active_tab = Some(active - 1);
            }
        }
        
        Ok(removed_tab.pane_id)
    }
    
    /// Set the active tab
    pub fn set_active_tab(&mut self, index: usize) -> SashResult<()> {
        if index >= self.tabs.len() {
            return Err(SashError::TabNotFound(index));
        }
        
        self.active_tab = Some(index);
        Ok(())
    }
    
    /// Move a tab from one position to another
    pub fn move_tab(&mut self, from: usize, to: usize) -> SashResult<()> {
        if from >= self.tabs.len() {
            return Err(SashError::TabNotFound(from));
        }
        
        if to > self.tabs.len() {
            return Err(SashError::TabNotFound(to));
        }
        
        if from == to {
            return Ok(());
        }
        
        let tab = self.tabs.remove(from);
        
        let insert_pos = if from < to { to - 1 } else { to };
        self.tabs.insert(insert_pos, tab);
        
        // Update active tab if needed
        if let Some(active) = self.active_tab {
            if active == from {
                self.active_tab = Some(insert_pos);
            } else if from < active && insert_pos >= active {
                self.active_tab = Some(active - 1);
            } else if from > active && insert_pos <= active {
                self.active_tab = Some(active + 1);
            }
        }
        
        Ok(())
    }
    
    /// Get the active pane ID
    pub fn get_active_pane(&self) -> Option<PaneId> {
        self.active_tab
            .and_then(|idx| self.tabs.get(idx))
            .map(|tab| tab.pane_id)
    }
    
    /// Update a tab's title
    pub fn update_tab_title(&mut self, index: usize, title: String) -> SashResult<()> {
        match self.tabs.get_mut(index) {
            Some(tab) => {
                tab.title = title;
                Ok(())
            }
            None => Err(SashError::TabNotFound(index)),
        }
    }
    
    /// Set a tab's modified state
    pub fn set_tab_modified(&mut self, index: usize, modified: bool) -> SashResult<()> {
        match self.tabs.get_mut(index) {
            Some(tab) => {
                tab.modified = modified;
                Ok(())
            }
            None => Err(SashError::TabNotFound(index)),
        }
    }
    
    /// Get the number of tabs
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }
    
    /// Get the active tab index
    pub fn active_tab_index(&self) -> Option<usize> {
        self.active_tab
    }
    
    /// Get all tabs
    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }
    
    /// Navigate to the next tab
    pub fn next_tab(&mut self) -> SashResult<()> {
        if self.tabs.is_empty() {
            return Err(SashError::NoActiveTab);
        }
        
        match self.active_tab {
            Some(current) => {
                let next = (current + 1) % self.tabs.len();
                self.active_tab = Some(next);
                Ok(())
            }
            None => {
                self.active_tab = Some(0);
                Ok(())
            }
        }
    }
    
    /// Navigate to the previous tab
    pub fn previous_tab(&mut self) -> SashResult<()> {
        if self.tabs.is_empty() {
            return Err(SashError::NoActiveTab);
        }
        
        match self.active_tab {
            Some(current) => {
                let prev = if current == 0 {
                    self.tabs.len() - 1
                } else {
                    current - 1
                };
                self.active_tab = Some(prev);
                Ok(())
            }
            None => {
                self.active_tab = Some(self.tabs.len() - 1);
                Ok(())
            }
        }
    }
}

/// Configuration for tab behavior
#[derive(Debug, Clone)]
pub struct TabConfig {
    pub max_tabs: Option<usize>,
    pub allow_no_tabs: bool,
    pub auto_close_empty: bool,
    pub confirm_close_modified: bool,
    pub tab_width: TabWidth,
    pub new_tab_position: NewTabPosition,
}

impl Default for TabConfig {
    fn default() -> Self {
        TabConfig {
            max_tabs: None,
            allow_no_tabs: false,
            auto_close_empty: false,
            confirm_close_modified: true,
            tab_width: TabWidth::Adaptive { min: 100, max: 300 },
            new_tab_position: NewTabPosition::AfterCurrent,
        }
    }
}

/// Tab width configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TabWidth {
    Fixed(u16),
    Adaptive { min: u16, max: u16 },
    Flexible,
}

/// Position for new tabs
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NewTabPosition {
    End,
    AfterCurrent,
    Beginning,
}