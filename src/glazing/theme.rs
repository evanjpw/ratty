use super::*;
use crate::pane::CellAttributes;
use crate::sash::{Color, Theme};
use ratatui::style::{Color as RatatuiColor, Style as RatatuiStyle};

/// Theme engine for converting terminal themes to display styles
#[derive(Debug, Clone)]
pub struct ThemeEngine {
    pub(crate) current_theme: Theme,
    base_style: RatatuiStyle,
    cursor_color: Color,
    selection_color: Color,
    border_style: RatatuiStyle,
    config: ThemeConfig,
}

impl ThemeEngine {
    /// Create a new theme engine
    pub fn new(config: &ThemeConfig) -> GlazingResult<Self> {
        let default_theme = Theme::default();
        let mut engine = ThemeEngine {
            current_theme: default_theme.clone(),
            base_style: RatatuiStyle::default(),
            cursor_color: default_theme.colors.cursor,
            selection_color: default_theme.colors.selection,
            border_style: RatatuiStyle::default(),
            config: config.clone(),
        };
        
        engine.apply_theme(&default_theme)?;
        Ok(engine)
    }
    
    /// Apply a theme to the engine
    pub fn apply_theme(&mut self, theme: &Theme) -> GlazingResult<()> {
        self.current_theme = theme.clone();
        
        // Update base style
        self.base_style = RatatuiStyle::default()
            .fg(self.convert_color(theme.colors.foreground)?)
            .bg(self.convert_color(theme.colors.background)?);
        
        // Update cursor and selection colors
        self.cursor_color = theme.colors.cursor;
        self.selection_color = theme.colors.selection;
        
        // Update border style
        self.border_style = RatatuiStyle::default()
            .fg(self.convert_color(theme.colors.border)?)
            .bg(self.convert_color(theme.colors.background)?);
        
        Ok(())
    }
    
    /// Convert cell attributes and colors to display style
    pub fn convert_cell_style(
        &self,
        attributes: &CellAttributes,
        foreground: Color,
        background: Color,
    ) -> GlazingResult<super::renderer::CellStyle> {
        let mut fg = foreground;
        let mut bg = background;
        
        // Apply reverse video
        if attributes.reverse {
            std::mem::swap(&mut fg, &mut bg);
        }
        
        // Apply dim
        if attributes.dim {
            fg = self.apply_dim_color(fg);
        }
        
        Ok(super::renderer::CellStyle {
            foreground: fg,
            background: bg,
            bold: attributes.bold,
            italic: attributes.italic,
            underline: attributes.underline != crate::pane::UnderlineType::None,
            strikethrough: attributes.strikethrough,
            reverse: false, // Already applied above
            dim: attributes.dim,
        })
    }
    
    /// Get base terminal style
    pub fn get_base_style(&self) -> RatatuiStyle {
        self.base_style
    }
    
    /// Get cursor color
    pub fn get_cursor_color(&self) -> Color {
        self.cursor_color
    }
    
    /// Get selection color
    pub fn get_selection_color(&self) -> Color {
        self.selection_color
    }
    
    /// Get background color
    pub fn get_background_color(&self) -> RatatuiColor {
        self.convert_color(self.current_theme.colors.background)
            .unwrap_or(RatatuiColor::Black)
    }
    
    /// Get border style
    pub fn get_border_style(&self) -> RatatuiStyle {
        self.border_style
    }
    
    /// Get ANSI color by index
    pub fn get_ansi_color(&self, index: u8) -> Color {
        if index < 16 {
            self.current_theme.colors.ansi_colors[index as usize]
        } else if index < 232 {
            // 216-color cube
            self.calculate_color_cube_color(index - 16)
        } else {
            // Grayscale
            self.calculate_grayscale_color(index - 232)
        }
    }
    
    /// Update theme configuration
    pub fn update_config(&mut self, config: &ThemeConfig) -> GlazingResult<()> {
        self.config = config.clone();
        Ok(())
    }
    
    /// Convert Color to RatatuiColor
    fn convert_color(&self, color: Color) -> GlazingResult<RatatuiColor> {
        Ok(RatatuiColor::Rgb(color.r, color.g, color.b))
    }
    
    /// Apply dim effect to a color
    fn apply_dim_color(&self, color: Color) -> Color {
        Color::from_rgb(
            (color.r as f32 * 0.6) as u8,
            (color.g as f32 * 0.6) as u8,
            (color.b as f32 * 0.6) as u8,
        )
    }
    
    /// Calculate color for 216-color cube
    fn calculate_color_cube_color(&self, index: u8) -> Color {
        let index = index as usize;
        let r = (index / 36) % 6;
        let g = (index / 6) % 6;
        let b = index % 6;
        
        let scale = |c: usize| -> u8 {
            if c == 0 { 0 } else { (55 + c * 40) as u8 }
        };
        
        Color::from_rgb(scale(r), scale(g), scale(b))
    }
    
    /// Calculate grayscale color
    fn calculate_grayscale_color(&self, index: u8) -> Color {
        let gray = 8 + index * 10;
        Color::from_rgb(gray, gray, gray)
    }
}

/// Theme configuration
#[derive(Debug, Clone)]
pub struct ThemeConfig {
    pub default_theme_name: String,
    pub font_family: String,
    pub font_size: f32,
    pub line_height: f32,
    pub cursor_blink_rate: std::time::Duration,
    pub smooth_transitions: bool,
    pub high_contrast_mode: bool,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        ThemeConfig {
            default_theme_name: "default".to_string(),
            font_family: "monospace".to_string(),
            font_size: 12.0,
            line_height: 1.2,
            cursor_blink_rate: std::time::Duration::from_millis(500),
            smooth_transitions: true,
            high_contrast_mode: false,
        }
    }
}

/// Color palette management
#[derive(Debug, Clone)]
pub struct ColorPalette {
    pub colors: Vec<Color>,
    pub name: String,
}

impl ColorPalette {
    /// Create a new color palette
    pub fn new(name: String, colors: Vec<Color>) -> Self {
        ColorPalette { name, colors }
    }
    
    /// Get color by index
    pub fn get_color(&self, index: usize) -> Option<Color> {
        self.colors.get(index).copied()
    }
    
    /// Create standard 16-color ANSI palette
    pub fn ansi_16() -> Self {
        let colors = vec![
            // Standard colors (0-7)
            Color::from_rgb(0, 0, 0),       // Black
            Color::from_rgb(128, 0, 0),     // Dark Red
            Color::from_rgb(0, 128, 0),     // Dark Green
            Color::from_rgb(128, 128, 0),   // Dark Yellow
            Color::from_rgb(0, 0, 128),     // Dark Blue
            Color::from_rgb(128, 0, 128),   // Dark Magenta
            Color::from_rgb(0, 128, 128),   // Dark Cyan
            Color::from_rgb(192, 192, 192), // Light Gray
            
            // Bright colors (8-15)
            Color::from_rgb(128, 128, 128), // Dark Gray
            Color::from_rgb(255, 0, 0),     // Bright Red
            Color::from_rgb(0, 255, 0),     // Bright Green
            Color::from_rgb(255, 255, 0),   // Bright Yellow
            Color::from_rgb(0, 0, 255),     // Bright Blue
            Color::from_rgb(255, 0, 255),   // Bright Magenta
            Color::from_rgb(0, 255, 255),   // Bright Cyan
            Color::from_rgb(255, 255, 255), // White
        ];
        
        ColorPalette::new("ANSI 16".to_string(), colors)
    }
    
    /// Create solarized dark palette
    pub fn solarized_dark() -> Self {
        let colors = vec![
            Color::from_rgb(0x00, 0x2b, 0x36),   // base03
            Color::from_rgb(0xdc, 0x32, 0x2f),   // red
            Color::from_rgb(0x85, 0x99, 0x00),   // green
            Color::from_rgb(0xb5, 0x89, 0x00),   // yellow
            Color::from_rgb(0x26, 0x8b, 0xd2),   // blue
            Color::from_rgb(0xd3, 0x36, 0x82),   // magenta
            Color::from_rgb(0x2a, 0xa1, 0x98),   // cyan
            Color::from_rgb(0xee, 0xe8, 0xd5),   // base2
            
            Color::from_rgb(0x00, 0x2b, 0x36),   // base02
            Color::from_rgb(0xcb, 0x4b, 0x16),   // orange
            Color::from_rgb(0x58, 0x6e, 0x75),   // base01
            Color::from_rgb(0x65, 0x7b, 0x83),   // base00
            Color::from_rgb(0x83, 0x94, 0x96),   // base0
            Color::from_rgb(0x6c, 0x71, 0xc4),   // violet
            Color::from_rgb(0x93, 0xa1, 0xa1),   // base1
            Color::from_rgb(0xfd, 0xf6, 0xe3),   // base3
        ];
        
        ColorPalette::new("Solarized Dark".to_string(), colors)
    }
    
    /// Create solarized light palette
    pub fn solarized_light() -> Self {
        let colors = vec![
            Color::from_rgb(0xfd, 0xf6, 0xe3),   // base3
            Color::from_rgb(0xdc, 0x32, 0x2f),   // red
            Color::from_rgb(0x85, 0x99, 0x00),   // green
            Color::from_rgb(0xb5, 0x89, 0x00),   // yellow
            Color::from_rgb(0x26, 0x8b, 0xd2),   // blue
            Color::from_rgb(0xd3, 0x36, 0x82),   // magenta
            Color::from_rgb(0x2a, 0xa1, 0x98),   // cyan
            Color::from_rgb(0x00, 0x2b, 0x36),   // base03
            
            Color::from_rgb(0xee, 0xe8, 0xd5),   // base2
            Color::from_rgb(0xcb, 0x4b, 0x16),   // orange
            Color::from_rgb(0x93, 0xa1, 0xa1),   // base1
            Color::from_rgb(0x83, 0x94, 0x96),   // base0
            Color::from_rgb(0x65, 0x7b, 0x83),   // base00
            Color::from_rgb(0x6c, 0x71, 0xc4),   // violet
            Color::from_rgb(0x58, 0x6e, 0x75),   // base01
            Color::from_rgb(0x00, 0x2b, 0x36),   // base02
        ];
        
        ColorPalette::new("Solarized Light".to_string(), colors)
    }
}

/// Built-in theme collection
#[derive(Debug)]
pub struct ThemeCollection {
    themes: std::collections::HashMap<String, Theme>,
}

impl ThemeCollection {
    /// Create a new theme collection with built-in themes
    pub fn new() -> Self {
        let mut collection = ThemeCollection {
            themes: std::collections::HashMap::new(),
        };
        
        collection.add_builtin_themes();
        collection
    }
    
    /// Add a theme to the collection
    pub fn add_theme(&mut self, name: String, theme: Theme) {
        self.themes.insert(name, theme);
    }
    
    /// Get a theme by name
    pub fn get_theme(&self, name: &str) -> Option<&Theme> {
        self.themes.get(name)
    }
    
    /// List all available theme names
    pub fn list_themes(&self) -> Vec<String> {
        self.themes.keys().cloned().collect()
    }
    
    /// Add built-in themes
    fn add_builtin_themes(&mut self) {
        // Default theme
        self.add_theme("default".to_string(), Theme::default());
        
        // Solarized Dark
        let solarized_dark = Theme {
            name: "Solarized Dark".to_string(),
            colors: crate::sash::ColorScheme {
                foreground: Color::from_rgb(0x83, 0x94, 0x96),
                background: Color::from_rgb(0x00, 0x2b, 0x36),
                cursor: Color::from_rgb(0x83, 0x94, 0x96),
                selection: Color::from_rgb(0x07, 0x36, 0x42),
                ansi_colors: ColorPalette::solarized_dark().colors.try_into().unwrap_or([Color::default(); 16]),
                tab_active_bg: Color::from_rgb(0x58, 0x6e, 0x75),
                tab_inactive_bg: Color::from_rgb(0x00, 0x2b, 0x36),
                tab_hover_bg: Color::from_rgb(0x07, 0x36, 0x42),
                tab_border: Color::from_rgb(0x58, 0x6e, 0x75),
                split_border: Color::from_rgb(0x58, 0x6e, 0x75),
                status_bar_bg: Color::from_rgb(0x58, 0x6e, 0x75),
                status_bar_fg: Color::from_rgb(0x83, 0x94, 0x96),
                border: Color::from_rgb(0x58, 0x6e, 0x75),
            },
            fonts: crate::sash::FontScheme::default(),
            spacing: crate::sash::SpacingScheme::default(),
            decorations: crate::sash::DecorationScheme::default(),
        };
        self.add_theme("solarized-dark".to_string(), solarized_dark);
        
        // Solarized Light
        let solarized_light = Theme {
            name: "Solarized Light".to_string(),
            colors: crate::sash::ColorScheme {
                foreground: Color::from_rgb(0x65, 0x7b, 0x83),
                background: Color::from_rgb(0xfd, 0xf6, 0xe3),
                cursor: Color::from_rgb(0x65, 0x7b, 0x83),
                selection: Color::from_rgb(0xee, 0xe8, 0xd5),
                ansi_colors: ColorPalette::solarized_light().colors.try_into().unwrap_or([Color::default(); 16]),
                tab_active_bg: Color::from_rgb(0x93, 0xa1, 0xa1),
                tab_inactive_bg: Color::from_rgb(0xfd, 0xf6, 0xe3),
                tab_hover_bg: Color::from_rgb(0xee, 0xe8, 0xd5),
                tab_border: Color::from_rgb(0x93, 0xa1, 0xa1),
                split_border: Color::from_rgb(0x93, 0xa1, 0xa1),
                status_bar_bg: Color::from_rgb(0x93, 0xa1, 0xa1),
                status_bar_fg: Color::from_rgb(0x65, 0x7b, 0x83),
                border: Color::from_rgb(0x93, 0xa1, 0xa1),
            },
            fonts: crate::sash::FontScheme::default(),
            spacing: crate::sash::SpacingScheme::default(),
            decorations: crate::sash::DecorationScheme::default(),
        };
        self.add_theme("solarized-light".to_string(), solarized_light);
    }
}

impl Default for ThemeCollection {
    fn default() -> Self {
        Self::new()
    }
}