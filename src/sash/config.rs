use crate::frame::config::{FontWeight, FontStyle};

/// Configuration for a window (Sash)
#[derive(Debug, Clone, PartialEq)]
pub struct WindowConfig {
    pub size: (u32, u32),
    pub position: Option<(i32, i32)>,
    pub theme: Option<String>,
    pub font_config: Option<FontConfig>,
    pub restore_session: bool,
    pub max_panes: Option<usize>,
    pub default_shell: Option<String>,
    pub working_directory: Option<String>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        WindowConfig {
            size: (1024, 768),
            position: None,
            theme: None,
            font_config: None,
            restore_session: false,
            max_panes: Some(100),
            default_shell: None,
            working_directory: None,
        }
    }
}

/// Configuration for a pane (terminal instance)
#[derive(Debug, Clone, PartialEq)]
pub struct PaneConfig {
    pub shell: Option<String>,
    pub working_directory: Option<String>,
    pub environment: Vec<(String, String)>,
    pub title: Option<String>,
    pub closable: bool,
    pub scrollback_lines: usize,
}

impl Default for PaneConfig {
    fn default() -> Self {
        PaneConfig {
            shell: None,
            working_directory: None,
            environment: Vec::new(),
            title: None,
            closable: true,
            scrollback_lines: 10000,
        }
    }
}

/// Font configuration (re-exported for convenience)
#[derive(Debug, Clone, PartialEq)]
pub struct FontConfig {
    pub family: String,
    pub size: u16,
    pub weight: FontWeight,
    pub style: FontStyle,
}

impl Default for FontConfig {
    fn default() -> Self {
        FontConfig {
            family: "monospace".to_string(),
            size: 12,
            weight: FontWeight::Normal,
            style: FontStyle::Normal,
        }
    }
}