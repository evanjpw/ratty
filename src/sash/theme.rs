use std::str::FromStr;
use super::{SashError, SashResult};
use crate::frame::config::{FontWeight, FontStyle};
use ratatui::style::Color as RatColor;

/// Complete theme configuration for a Sash
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    pub name: String,
    pub colors: ColorScheme,
    pub fonts: FontScheme,
    pub spacing: SpacingScheme,
    pub decorations: DecorationScheme,
}

impl Theme {
    /// Create a new theme with the given name
    pub fn new(name: String) -> Self {
        Theme {
            name,
            colors: ColorScheme::default(),
            fonts: FontScheme::default(),
            spacing: SpacingScheme::default(),
            decorations: DecorationScheme::default(),
        }
    }
    
    /// Validate theme settings
    pub fn validate(&self) -> SashResult<()> {
        // Validate font size
        if self.fonts.size == 0 || self.fonts.size > 200 {
            return Err(SashError::ThemeError(
                "Font size must be between 1 and 200".to_string()
            ));
        }
        
        // Validate spacing
        if self.spacing.tab_height == 0 || self.spacing.status_bar_height == 0 {
            return Err(SashError::ThemeError(
                "Tab and status bar heights must be greater than 0".to_string()
            ));
        }
        
        Ok(())
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            name: "default".to_string(),
            colors: ColorScheme::default(),
            fonts: FontScheme::default(),
            spacing: SpacingScheme::default(),
            decorations: DecorationScheme::default(),
        }
    }
}

/// Color scheme for terminal and UI elements
#[derive(Debug, Clone, PartialEq)]
pub struct ColorScheme {
    // Terminal colors
    pub foreground: Color,
    pub background: Color,
    pub cursor: Color,
    pub selection: Color,
    
    // ANSI colors (16 standard colors)
    pub ansi_colors: [Color; 16],
    
    // UI elements
    pub tab_active_bg: Color,
    pub tab_inactive_bg: Color,
    pub tab_hover_bg: Color,
    pub tab_border: Color,
    pub split_border: Color,
    pub status_bar_bg: Color,
    pub status_bar_fg: Color,
    // TODO: This was added to fix a compilation error & may actually be harmful
    pub border: Color,
}

impl Default for ColorScheme {
    fn default() -> Self {
        ColorScheme {
            // Terminal defaults (dark theme)
            foreground: Color::from_rgb(204, 204, 204),
            background: Color::from_rgb(30, 30, 30),
            cursor: Color::from_rgb(255, 255, 255),
            selection: Color::from_rgba(100, 100, 100, 128),
            
            // Default ANSI colors
            ansi_colors: [
                Color::from_rgb(0, 0, 0),       // Black
                Color::from_rgb(205, 49, 49),   // Red
                Color::from_rgb(13, 188, 121),  // Green
                Color::from_rgb(229, 229, 16),  // Yellow
                Color::from_rgb(36, 114, 200),  // Blue
                Color::from_rgb(188, 63, 188),  // Magenta
                Color::from_rgb(17, 168, 205),  // Cyan
                Color::from_rgb(229, 229, 229), // White
                // Bright colors
                Color::from_rgb(102, 102, 102), // Bright Black
                Color::from_rgb(241, 76, 76),   // Bright Red
                Color::from_rgb(35, 209, 139),  // Bright Green
                Color::from_rgb(245, 245, 67),  // Bright Yellow
                Color::from_rgb(59, 142, 234),  // Bright Blue
                Color::from_rgb(214, 112, 214), // Bright Magenta
                Color::from_rgb(41, 184, 219),  // Bright Cyan
                Color::from_rgb(255, 255, 255), // Bright White
            ],
            
            // UI defaults
            tab_active_bg: Color::from_rgb(60, 60, 60),
            tab_inactive_bg: Color::from_rgb(40, 40, 40),
            tab_hover_bg: Color::from_rgb(50, 50, 50),
            tab_border: Color::from_rgb(80, 80, 80),
            split_border: Color::from_rgb(80, 80, 80),
            status_bar_bg: Color::from_rgb(40, 40, 40),
            status_bar_fg: Color::from_rgb(180, 180, 180),
            border: Color::from_rgb(80, 80, 80),
        }
    }
}

/// RGBA color representation
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Create a color from RGB values (alpha = 255)
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b, a: 255 }
    }
    
    /// Create a color from RGBA values
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }
    
    /// Create a color from a hex string
    pub fn from_hex(hex: &str) -> SashResult<Self> {
        let hex = hex.trim_start_matches('#');
        
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16)
                    .map_err(|_| SashError::InvalidColor(format!("Invalid hex color: {}", hex)))?;
                let g = u8::from_str_radix(&hex[2..4], 16)
                    .map_err(|_| SashError::InvalidColor(format!("Invalid hex color: {}", hex)))?;
                let b = u8::from_str_radix(&hex[4..6], 16)
                    .map_err(|_| SashError::InvalidColor(format!("Invalid hex color: {}", hex)))?;
                Ok(Color::from_rgb(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16)
                    .map_err(|_| SashError::InvalidColor(format!("Invalid hex color: {}", hex)))?;
                let g = u8::from_str_radix(&hex[2..4], 16)
                    .map_err(|_| SashError::InvalidColor(format!("Invalid hex color: {}", hex)))?;
                let b = u8::from_str_radix(&hex[4..6], 16)
                    .map_err(|_| SashError::InvalidColor(format!("Invalid hex color: {}", hex)))?;
                let a = u8::from_str_radix(&hex[6..8], 16)
                    .map_err(|_| SashError::InvalidColor(format!("Invalid hex color: {}", hex)))?;
                Ok(Color::from_rgba(r, g, b, a))
            }
            _ => Err(SashError::InvalidColor(format!("Invalid hex color length: {}", hex))),
        }
    }
    
    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        if self.a == 255 {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        }
    }
}

impl From<Color> for  RatColor {
    fn from(value: Color) -> Self {
        RatColor::from_str(value.to_hex().as_str()).unwrap()
    }
}

/// Font configuration for the terminal
#[derive(Debug, Clone, PartialEq)]
pub struct FontScheme {
    pub family: String,
    pub size: u16,
    pub weight: FontWeight,
    pub style: FontStyle,
    pub ligatures: bool,
}

impl Default for FontScheme {
    fn default() -> Self {
        FontScheme {
            family: "monospace".to_string(),
            size: 12,
            weight: FontWeight::Normal,
            style: FontStyle::Normal,
            ligatures: true,
        }
    }
}

/// Spacing configuration for UI elements
#[derive(Debug, Clone, PartialEq)]
pub struct SpacingScheme {
    pub tab_height: u16,
    pub tab_padding: u16,
    pub split_border_width: u16,
    pub status_bar_height: u16,
    pub content_padding: u16,
}

impl Default for SpacingScheme {
    fn default() -> Self {
        SpacingScheme {
            tab_height: 30,
            tab_padding: 10,
            split_border_width: 2,
            status_bar_height: 25,
            content_padding: 2,
        }
    }
}

/// Decoration configuration for UI elements
#[derive(Debug, Clone, PartialEq)]
pub struct DecorationScheme {
    pub show_tabs: bool,
    pub show_status_bar: bool,
    pub show_split_borders: bool,
    pub tab_style: TabStyle,
    pub border_style: BorderStyle,
}

impl Default for DecorationScheme {
    fn default() -> Self {
        DecorationScheme {
            show_tabs: true,
            show_status_bar: true,
            show_split_borders: true,
            tab_style: TabStyle::Rounded,
            border_style: BorderStyle::Solid,
        }
    }
}

/// Style for tab rendering
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TabStyle {
    Classic,    // Traditional rectangular tabs
    Rounded,    // Rounded corner tabs
    Underline,  // Underlined active tab
    Minimal,    // Text-only tabs
}

/// Style for border rendering
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BorderStyle {
    None,
    Solid,
    Dashed,
    Dotted,
    Double,
}