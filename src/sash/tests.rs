#[cfg(test)]
mod sash_tests {
    use super::*;
    use crate::sash::{
        Sash, SashInterface, PaneId, SashError, WindowConfig, Theme,
        Layout, Tab, TabManager, TabConfig, LayoutManager, Color, NewTabPosition
    };
    use crate::frame::SashId;
    
    // Helper function to create a test Sash
    fn create_test_sash() -> Sash {
        let config = WindowConfig::default();
        Sash::new(SashId::new(1), config).expect("Failed to create test sash")
    }
    
    // Basic Sash tests
    #[test]
    fn test_sash_creation() {
        let sash = create_test_sash();
        // Use local SashInterface trait methods
        use crate::sash::SashInterface as LocalSashInterface;
        assert_eq!(LocalSashInterface::id(&sash), SashId::new(1));
        assert!(!LocalSashInterface::is_active(&sash));
        assert_eq!(sash.pane_count(), 0);
        assert!(!sash.has_panes());
    }
    
    #[test]
    fn test_sash_active_state() {
        let mut sash = create_test_sash();
        // Use local SashInterface trait methods
        use crate::sash::SashInterface as LocalSashInterface;
        assert!(!LocalSashInterface::is_active(&sash));
        
        LocalSashInterface::set_active(&mut sash, true);
        assert!(LocalSashInterface::is_active(&sash));
        
        LocalSashInterface::set_active(&mut sash, false);
        assert!(!LocalSashInterface::is_active(&sash));
    }
    
    // PaneId tests
    #[test]
    fn test_pane_id() {
        let id1 = PaneId::new(42);
        let id2 = PaneId::new(42);
        let id3 = PaneId::new(43);
        
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        assert_eq!(id1.as_u64(), 42);
    }
    
    // Layout tests
    #[test]
    fn test_layout_empty() {
        let layout = Layout::Empty;
        assert!(layout.is_empty());
        assert_eq!(layout.get_pane_ids().len(), 0);
    }
    
    #[test]
    fn test_layout_single() {
        let pane_id = PaneId::new(1);
        let layout = Layout::Single(pane_id);
        
        assert!(!layout.is_empty());
        assert_eq!(layout.get_pane_ids(), vec![pane_id]);
        assert!(layout.find_pane(pane_id).is_some());
        assert!(layout.find_pane(PaneId::new(2)).is_none());
    }
    
    #[test]
    fn test_layout_tabs() {
        let tab1 = Tab::new(PaneId::new(1), "Tab 1".to_string());
        let tab2 = Tab::new(PaneId::new(2), "Tab 2".to_string());
        
        let layout = Layout::Tabs {
            tabs: vec![tab1, tab2],
            active_tab: 0,
        };
        
        let pane_ids = layout.get_pane_ids();
        assert_eq!(pane_ids.len(), 2);
        assert!(pane_ids.contains(&PaneId::new(1)));
        assert!(pane_ids.contains(&PaneId::new(2)));
    }
    
    #[test]
    fn test_layout_horizontal_split() {
        let layout = Layout::HorizontalSplit {
            top: Box::new(Layout::Single(PaneId::new(1))),
            bottom: Box::new(Layout::Single(PaneId::new(2))),
            split_ratio: 0.5,
        };
        
        let pane_ids = layout.get_pane_ids();
        assert_eq!(pane_ids.len(), 2);
        
        let path1 = layout.find_pane(PaneId::new(1));
        assert!(path1.is_some());
        assert_eq!(path1.unwrap(), vec![crate::sash::layout::LayoutPath::Top]);
        
        let path2 = layout.find_pane(PaneId::new(2));
        assert!(path2.is_some());
        assert_eq!(path2.unwrap(), vec![crate::sash::layout::LayoutPath::Bottom]);
    }
    
    #[test]
    fn test_layout_manager() {
        let mut manager = LayoutManager::new();
        
        // Initial state
        assert!(manager.current().is_empty());
        assert_eq!(manager.list_saved_layouts().len(), 0);
        
        // Set a layout
        let layout = Layout::Single(PaneId::new(1));
        assert!(manager.set_current(layout.clone()).is_ok());
        
        // Save layout
        assert!(manager.save_layout("test".to_string()).is_ok());
        assert_eq!(manager.list_saved_layouts().len(), 1);
        
        // Load layout
        assert!(manager.load_layout("test").is_ok());
        assert!(manager.load_layout("nonexistent").is_err());
    }
    
    #[test]
    fn test_layout_validation() {
        let manager = LayoutManager::new();
        
        // Valid split ratio
        let valid_layout = Layout::HorizontalSplit {
            top: Box::new(Layout::Empty),
            bottom: Box::new(Layout::Empty),
            split_ratio: 0.5,
        };
        assert!(manager.validate_layout(&valid_layout).is_ok());
        
        // Invalid split ratio
        let invalid_layout = Layout::HorizontalSplit {
            top: Box::new(Layout::Empty),
            bottom: Box::new(Layout::Empty),
            split_ratio: 1.5,
        };
        assert!(manager.validate_layout(&invalid_layout).is_err());
    }
    
    // Tab tests
    #[test]
    fn test_tab_creation() {
        let tab = Tab::new(PaneId::new(1), "Test Tab".to_string());
        assert_eq!(tab.pane_id, PaneId::new(1));
        assert_eq!(tab.title, "Test Tab");
        assert!(tab.closable);
        assert!(!tab.modified);
    }
    
    #[test]
    fn test_tab_manager() {
        let mut manager = TabManager::new(TabConfig::default());
        
        // Initial state
        assert_eq!(manager.tab_count(), 0);
        assert!(manager.active_tab_index().is_none());
        assert!(manager.get_active_pane().is_none());
        
        // Add tabs
        let idx1 = manager.add_tab(PaneId::new(1), "Tab 1".to_string()).unwrap();
        assert_eq!(idx1, 0);
        assert_eq!(manager.tab_count(), 1);
        assert_eq!(manager.active_tab_index(), Some(0));
        
        let _idx2 = manager.add_tab(PaneId::new(2), "Tab 2".to_string()).unwrap();
        assert_eq!(manager.tab_count(), 2);
        
        // Navigate tabs
        assert!(manager.next_tab().is_ok());
        assert_eq!(manager.active_tab_index(), Some(1));
        
        assert!(manager.previous_tab().is_ok());
        assert_eq!(manager.active_tab_index(), Some(0));
    }
    
    #[test]
    fn test_tab_manager_remove() {
        let mut config = TabConfig::default();
        config.allow_no_tabs = true;
        config.new_tab_position = NewTabPosition::End; // Use End for predictable order
        let mut manager = TabManager::new(config);
        
        manager.add_tab(PaneId::new(1), "Tab 1".to_string()).unwrap();
        manager.add_tab(PaneId::new(2), "Tab 2".to_string()).unwrap();
        manager.add_tab(PaneId::new(3), "Tab 3".to_string()).unwrap();
        
        // Verify initial order: [1, 2, 3]
        let tabs = manager.tabs();
        assert_eq!(tabs[0].pane_id, PaneId::new(1));
        assert_eq!(tabs[1].pane_id, PaneId::new(2));
        assert_eq!(tabs[2].pane_id, PaneId::new(3));
        
        // Remove middle tab (index 1, which is PaneId(2))
        let removed = manager.remove_tab(1).unwrap();
        assert_eq!(removed, PaneId::new(2));
        assert_eq!(manager.tab_count(), 2);
        
        // Remove last tab (now index 1, which is PaneId(3))
        let removed = manager.remove_tab(1).unwrap();
        assert_eq!(removed, PaneId::new(3));
        assert_eq!(manager.tab_count(), 1);
    }
    
    #[test]
    fn test_tab_manager_move() {
        let mut config = TabConfig::default();
        config.new_tab_position = NewTabPosition::End; // Use End for predictable order
        let mut manager = TabManager::new(config);
        
        manager.add_tab(PaneId::new(1), "Tab 1".to_string()).unwrap();
        manager.add_tab(PaneId::new(2), "Tab 2".to_string()).unwrap();
        manager.add_tab(PaneId::new(3), "Tab 3".to_string()).unwrap();
        
        // Verify initial order: [1, 2, 3]
        let tabs = manager.tabs();
        assert_eq!(tabs[0].pane_id, PaneId::new(1));
        assert_eq!(tabs[1].pane_id, PaneId::new(2));
        assert_eq!(tabs[2].pane_id, PaneId::new(3));
        
        // Move tab from index 0 to index 2 (move PaneId(1) to position 2)
        assert!(manager.move_tab(0, 2).is_ok());
        
        // Verify new order: [2, 1, 3] (inserted at position 2-1=1 after removal)
        let tabs = manager.tabs();
        assert_eq!(tabs[0].pane_id, PaneId::new(2));
        assert_eq!(tabs[1].pane_id, PaneId::new(1));
        assert_eq!(tabs[2].pane_id, PaneId::new(3));
    }
    
    // Theme tests
    #[test]
    fn test_theme_creation() {
        let theme = Theme::new("test".to_string());
        assert_eq!(theme.name, "test");
        assert!(theme.validate().is_ok());
    }
    
    #[test]
    fn test_theme_validation() {
        let mut theme = Theme::default();
        
        // Valid theme
        assert!(theme.validate().is_ok());
        
        // Invalid font size
        theme.fonts.size = 0;
        assert!(theme.validate().is_err());
        
        // Invalid spacing
        theme.fonts.size = 12;
        theme.spacing.tab_height = 0;
        assert!(theme.validate().is_err());
    }
    
    #[test]
    fn test_color_creation() {
        let color1 = Color::from_rgb(255, 128, 0);
        assert_eq!(color1.r, 255);
        assert_eq!(color1.g, 128);
        assert_eq!(color1.b, 0);
        assert_eq!(color1.a, 255);
        
        let color2 = Color::from_rgba(255, 128, 0, 128);
        assert_eq!(color2.a, 128);
    }
    
    #[test]
    fn test_color_hex() {
        // RGB hex
        let color = Color::from_hex("#FF8000").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 0);
        assert_eq!(color.a, 255);
        
        // RGBA hex
        let color = Color::from_hex("#FF800080").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 0);
        assert_eq!(color.a, 128);
        
        // To hex
        let color = Color::from_rgb(255, 128, 0);
        assert_eq!(color.to_hex(), "#FF8000");
        
        let color = Color::from_rgba(255, 128, 0, 128);
        assert_eq!(color.to_hex(), "#FF800080");
        
        // Invalid hex
        assert!(Color::from_hex("invalid").is_err());
        assert!(Color::from_hex("#12345").is_err());
    }
}