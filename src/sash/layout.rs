use super::{PaneId, SashError, SashResult};
use std::collections::HashMap;

/// Represents the layout of panes within a Sash
#[derive(Debug, Clone, PartialEq)]
pub enum Layout {
    /// No panes (empty window)
    Empty,
    
    /// Single pane fills the entire window
    Single(PaneId),
    
    /// Tabbed interface with active tab
    Tabs {
        tabs: Vec<Tab>,
        active_tab: usize,
    },
    
    /// Horizontal split (top/bottom)
    HorizontalSplit {
        top: Box<Layout>,
        bottom: Box<Layout>,
        split_ratio: f32, // 0.0 to 1.0, percentage for top
    },
    
    /// Vertical split (left/right)
    VerticalSplit {
        left: Box<Layout>,
        right: Box<Layout>,
        split_ratio: f32, // 0.0 to 1.0, percentage for left
    },
    
    /// Grid layout with fixed dimensions
    Grid {
        rows: usize,
        cols: usize,
        cells: Vec<Vec<Option<PaneId>>>,
    },
    
    /// Custom named layout (saved configurations)
    Custom {
        name: String,
        layout: Box<Layout>,
    },
}

impl Layout {
    /// Check if the layout is empty
    pub fn is_empty(&self) -> bool {
        matches!(self, Layout::Empty)
    }
    
    /// Get all pane IDs in the layout
    pub fn get_pane_ids(&self) -> Vec<PaneId> {
        match self {
            Layout::Empty => vec![],
            Layout::Single(id) => vec![*id],
            Layout::Tabs { tabs, .. } => tabs.iter().map(|t| t.pane_id).collect(),
            Layout::HorizontalSplit { top, bottom, .. } => {
                let mut ids = top.get_pane_ids();
                ids.extend(bottom.get_pane_ids());
                ids
            }
            Layout::VerticalSplit { left, right, .. } => {
                let mut ids = left.get_pane_ids();
                ids.extend(right.get_pane_ids());
                ids
            }
            Layout::Grid { cells, .. } => {
                cells.iter()
                    .flat_map(|row| row.iter())
                    .filter_map(|cell| *cell)
                    .collect()
            }
            Layout::Custom { layout, .. } => layout.get_pane_ids(),
        }
    }
    
    /// Find a pane in the layout and return its path
    pub fn find_pane(&self, pane_id: PaneId) -> Option<Vec<LayoutPath>> {
        self.find_pane_recursive(pane_id, vec![])
    }
    
    fn find_pane_recursive(&self, pane_id: PaneId, mut path: Vec<LayoutPath>) -> Option<Vec<LayoutPath>> {
        match self {
            Layout::Empty => None,
            Layout::Single(id) => {
                if *id == pane_id {
                    Some(path)
                } else {
                    None
                }
            }
            Layout::Tabs { tabs, active_tab: _ } => {
                for (index, tab) in tabs.iter().enumerate() {
                    if tab.pane_id == pane_id {
                        path.push(LayoutPath::Tab(index));
                        return Some(path);
                    }
                }
                None
            }
            Layout::HorizontalSplit { top, bottom, .. } => {
                path.push(LayoutPath::Top);
                if let Some(result) = top.find_pane_recursive(pane_id, path.clone()) {
                    return Some(result);
                }
                path.pop();
                path.push(LayoutPath::Bottom);
                bottom.find_pane_recursive(pane_id, path)
            }
            Layout::VerticalSplit { left, right, .. } => {
                path.push(LayoutPath::Left);
                if let Some(result) = left.find_pane_recursive(pane_id, path.clone()) {
                    return Some(result);
                }
                path.pop();
                path.push(LayoutPath::Right);
                right.find_pane_recursive(pane_id, path)
            }
            Layout::Grid { cells, .. } => {
                for (row_idx, row) in cells.iter().enumerate() {
                    for (col_idx, cell) in row.iter().enumerate() {
                        if let Some(id) = cell {
                            if *id == pane_id {
                                path.push(LayoutPath::GridCell(row_idx, col_idx));
                                return Some(path);
                            }
                        }
                    }
                }
                None
            }
            Layout::Custom { layout, .. } => layout.find_pane_recursive(pane_id, path),
        }
    }
}

/// Path component for navigating layouts
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutPath {
    Tab(usize),
    Top,
    Bottom,
    Left,
    Right,
    GridCell(usize, usize),
}

/// Represents a single tab in a tabbed layout
#[derive(Debug, Clone, PartialEq)]
pub struct Tab {
    pub pane_id: PaneId,
    pub title: String,
    pub closable: bool,
    pub modified: bool,
}

impl Tab {
    pub fn new(pane_id: PaneId, title: String) -> Self {
        Tab {
            pane_id,
            title,
            closable: true,
            modified: false,
        }
    }
}

/// Manages layout operations and constraints
pub struct LayoutManager {
    current_layout: Layout,
    saved_layouts: HashMap<String, Layout>,
    split_constraints: SplitConstraints,
}

impl LayoutManager {
    pub fn new() -> Self {
        LayoutManager {
            current_layout: Layout::Empty,
            saved_layouts: HashMap::new(),
            split_constraints: SplitConstraints::default(),
        }
    }
    
    /// Get the current layout
    pub fn current(&self) -> &Layout {
        &self.current_layout
    }
    
    /// Set the current layout
    pub fn set_current(&mut self, layout: Layout) -> SashResult<()> {
        self.validate_layout(&layout)?;
        self.current_layout = layout;
        Ok(())
    }
    
    /// Save the current layout with a name
    pub fn save_layout(&mut self, name: String) -> SashResult<()> {
        if self.current_layout.is_empty() {
            return Err(SashError::InvalidLayout("Cannot save empty layout".to_string()));
        }
        self.saved_layouts.insert(name, self.current_layout.clone());
        Ok(())
    }
    
    /// Load a saved layout
    pub fn load_layout(&mut self, name: &str) -> SashResult<()> {
        match self.saved_layouts.get(name) {
            Some(layout) => {
                self.current_layout = layout.clone();
                Ok(())
            }
            None => Err(SashError::LayoutNotFound(name.to_string())),
        }
    }
    
    /// List all saved layout names
    pub fn list_saved_layouts(&self) -> Vec<String> {
        self.saved_layouts.keys().cloned().collect()
    }
    
    /// Validate a layout against constraints
    pub fn validate_layout(&self, layout: &Layout) -> SashResult<()> {
        match layout {
            Layout::HorizontalSplit { split_ratio, .. } |
            Layout::VerticalSplit { split_ratio, .. } => {
                if *split_ratio < 0.0 || *split_ratio > 1.0 {
                    return Err(SashError::InvalidSplitRatio(*split_ratio));
                }
            }
            Layout::Grid { rows, cols, cells } => {
                if *rows == 0 || *cols == 0 {
                    return Err(SashError::InvalidLayout(
                        "Grid must have at least 1 row and 1 column".to_string()
                    ));
                }
                if cells.len() != *rows || cells.iter().any(|row| row.len() != *cols) {
                    return Err(SashError::InvalidLayout(
                        "Grid dimensions don't match cell array".to_string()
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Check if a split is valid given current constraints
    pub fn can_split(&self, direction: SplitDirection, current_size: (u16, u16)) -> bool {
        match direction {
            SplitDirection::Horizontal => {
                current_size.1 >= self.split_constraints.min_pane_height * 2
            }
            SplitDirection::Vertical => {
                current_size.0 >= self.split_constraints.min_pane_width * 2
            }
        }
    }
}

/// Split direction for layout operations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

/// Constraints for split operations
#[derive(Debug, Clone)]
pub struct SplitConstraints {
    pub min_pane_width: u16,
    pub min_pane_height: u16,
    pub max_split_depth: usize,
}

impl Default for SplitConstraints {
    fn default() -> Self {
        SplitConstraints {
            min_pane_width: 20,
            min_pane_height: 5,
            max_split_depth: 10,
        }
    }
}