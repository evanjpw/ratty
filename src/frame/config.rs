use std::collections::HashMap;
use std::path::PathBuf;
use crate::frame::errors::ConfigError;

/// Key combination for shortcuts
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyCombination {
    pub modifiers: Vec<KeyModifier>,
    pub key: Key,
}

/// Key modifiers
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KeyModifier {
    Ctrl,
    Alt,
    Shift,
    Meta, // Cmd on macOS, Windows key on Windows
}

/// Individual keys
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    Char(char),
    Function(u8), // F1-F24
    Tab,
    Enter,
    Escape,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    Up,
    Down,
    Left,
    Right,
}

/// Notification settings
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationSettings {
    pub enabled: bool,
    pub show_system_notifications: bool,
    pub play_sound: bool,
    pub flash_window: bool,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        NotificationSettings {
            enabled: true,
            show_system_notifications: true,
            play_sound: false,
            flash_window: true,
        }
    }
}

/// Window configuration for new windows
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowConfig {
    pub size: (u32, u32),
    pub position: Option<(i32, i32)>,
    pub theme: Option<String>,
    pub font_family: Option<String>,
    pub font_size: Option<u16>,
    pub restore_session: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        WindowConfig {
            size: (1024, 768),
            position: None,
            theme: None,
            font_family: None,
            font_size: None,
            restore_session: false,
        }
    }
}

/// Font configuration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontConfig {
    pub family: String,
    pub size: u16,
    pub weight: FontWeight,
    pub style: FontStyle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FontWeight {
    Normal,
    Bold,
    Light,
    ExtraLight,
    SemiBold,
    ExtraBold,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
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

/// Color scheme type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorScheme {
    Light,
    Dark,
    Auto, // Follow system preference
}

/// Global application configuration
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalConfig {
    // Application-wide settings
    pub default_theme: String,
    pub default_font: FontConfig,
    
    // Window behavior
    pub default_window_config: WindowConfig,
    pub allow_multiple_windows: bool,
    pub restore_windows_on_startup: bool,
    pub confirm_before_quit: bool,
    
    // Global shortcuts and behavior
    pub global_shortcuts: HashMap<KeyCombination, crate::frame::commands::GlobalCommand>,
    
    // Platform integration
    pub shell_integration: bool,
    pub notification_settings: NotificationSettings,
    pub color_scheme: ColorScheme,
    
    // Performance settings
    pub vsync: bool,
    pub gpu_acceleration: bool,
    pub scrollback_limit: usize,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        GlobalConfig {
            default_theme: "default".to_string(),
            default_font: FontConfig::default(),
            default_window_config: WindowConfig::default(),
            allow_multiple_windows: true,
            restore_windows_on_startup: false,
            confirm_before_quit: true,
            global_shortcuts: HashMap::new(),
            shell_integration: true,
            notification_settings: NotificationSettings::default(),
            color_scheme: ColorScheme::Auto,
            vsync: true,
            gpu_acceleration: true,
            scrollback_limit: 10000,
        }
    }
}

impl GlobalConfig {
    /// Load configuration from the default location
    pub fn load() -> Result<Self, ConfigError> {
        Self::load_from_path(Self::default_config_path()?)
    }
    
    /// Load configuration from a specific path
    pub fn load_from_path(_path: PathBuf) -> Result<Self, ConfigError> {
        // TODO: Implement actual config file loading
        // For now, return default configuration
        Ok(Self::default())
    }
    
    /// Save configuration to the default location
    pub fn save(&self) -> Result<(), ConfigError> {
        self.save_to_path(Self::default_config_path()?)
    }
    
    /// Save configuration to a specific path
    pub fn save_to_path(&self, _path: PathBuf) -> Result<(), ConfigError> {
        // TODO: Implement actual config file saving
        Ok(())
    }
    
    /// Get the default configuration file path
    pub fn default_config_path() -> Result<PathBuf, ConfigError> {
        // TODO: Implement platform-specific config directory resolution
        // For now, use a simple approach
        let mut path = std::env::current_dir()
            .map_err(|e| ConfigError::Io(e))?;
        path.push("ratty.toml");
        Ok(path)
    }
    
    /// Merge user configuration overrides
    pub fn merge_user_config(&mut self, _user_config: UserConfig) -> Result<(), ConfigError> {
        // TODO: Implement configuration merging logic
        Ok(())
    }
    
    /// Validate configuration settings
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate font size
        if self.default_font.size == 0 || self.default_font.size > 200 {
            return Err(ConfigError::InvalidValue(
                "Font size must be between 1 and 200".to_string()
            ));
        }
        
        // Validate window size
        if self.default_window_config.size.0 < 100 || self.default_window_config.size.1 < 100 {
            return Err(ConfigError::InvalidValue(
                "Window size must be at least 100x100".to_string()
            ));
        }
        
        // Validate scrollback limit
        if self.scrollback_limit == 0 {
            return Err(ConfigError::InvalidValue(
                "Scrollback limit must be greater than 0".to_string()
            ));
        }
        
        Ok(())
    }
}

/// User configuration overrides (subset of GlobalConfig for user customization)
#[derive(Debug, Clone, PartialEq)]
pub struct UserConfig {
    pub theme: Option<String>,
    pub font: Option<FontConfig>,
    pub window_config: Option<WindowConfig>,
    pub shortcuts: Option<HashMap<KeyCombination, crate::frame::commands::GlobalCommand>>,
    pub color_scheme: Option<ColorScheme>,
}

impl Default for UserConfig {
    fn default() -> Self {
        UserConfig {
            theme: None,
            font: None,
            window_config: None,
            shortcuts: None,
            color_scheme: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_config_default() {
        let config = GlobalConfig::default();
        assert_eq!(config.default_theme, "default");
        assert!(config.allow_multiple_windows);
        assert!(config.confirm_before_quit);
        assert_eq!(config.scrollback_limit, 10000);
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = GlobalConfig::default();
        
        // Valid configuration should pass
        assert!(config.validate().is_ok());
        
        // Invalid font size should fail
        config.default_font.size = 0;
        assert!(config.validate().is_err());
        
        // Reset and test invalid window size
        config.default_font.size = 12;
        config.default_window_config.size = (50, 50);
        assert!(config.validate().is_err());
        
        // Reset and test invalid scrollback
        config.default_window_config.size = (800, 600);
        config.scrollback_limit = 0;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_key_combination() {
        let combo = KeyCombination {
            modifiers: vec![KeyModifier::Ctrl, KeyModifier::Shift],
            key: Key::Char('n'),
        };
        
        // Test that we can use it as a HashMap key
        let mut map = HashMap::new();
        map.insert(combo.clone(), "test");
        assert_eq!(map.get(&combo), Some(&"test"));
    }
}