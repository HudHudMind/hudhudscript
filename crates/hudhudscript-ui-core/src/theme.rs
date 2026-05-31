//! Theme and design token system (#548)
//!
//! Provides consistent design tokens for colors, typography, spacing, and more.
//! Supports light/dark mode variants.

use serde::{Deserialize, Serialize};

/// Color palette tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    pub primary: String,
    pub secondary: String,
    pub background: String,
    pub surface: String,
    pub error: String,
    pub warning: String,
    pub success: String,
    pub info: String,
    pub text_primary: String,
    pub text_secondary: String,
    pub text_disabled: String,
    pub divider: String,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            primary: "#1976D2".to_string(),
            secondary: "#424242".to_string(),
            background: "#FFFFFF".to_string(),
            surface: "#FFFFFF".to_string(),
            error: "#D32F2F".to_string(),
            warning: "#FFA000".to_string(),
            success: "#388E3C".to_string(),
            info: "#1976D2".to_string(),
            text_primary: "#212121".to_string(),
            text_secondary: "#757575".to_string(),
            text_disabled: "#BDBDBD".to_string(),
            divider: "#E0E0E0".to_string(),
        }
    }
}

/// Typography tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Typography {
    pub font_family: String,
    pub font_size_xs: f64,
    pub font_size_sm: f64,
    pub font_size_md: f64,
    pub font_size_lg: f64,
    pub font_size_xl: f64,
    pub font_size_xxl: f64,
    pub font_weight_light: u32,
    pub font_weight_regular: u32,
    pub font_weight_medium: u32,
    pub font_weight_bold: u32,
    pub line_height: f64,
}

impl Default for Typography {
    fn default() -> Self {
        Self {
            font_family: "system-ui, -apple-system, sans-serif".to_string(),
            font_size_xs: 10.0,
            font_size_sm: 12.0,
            font_size_md: 14.0,
            font_size_lg: 18.0,
            font_size_xl: 24.0,
            font_size_xxl: 32.0,
            font_weight_light: 300,
            font_weight_regular: 400,
            font_weight_medium: 500,
            font_weight_bold: 700,
            line_height: 1.5,
        }
    }
}

/// Spacing scale tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spacing {
    pub xs: f64,
    pub sm: f64,
    pub md: f64,
    pub lg: f64,
    pub xl: f64,
    pub xxl: f64,
}

impl Default for Spacing {
    fn default() -> Self {
        Self {
            xs: 4.0,
            sm: 8.0,
            md: 16.0,
            lg: 24.0,
            xl: 32.0,
            xxl: 48.0,
        }
    }
}

/// Border radius tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorderRadius {
    pub none: f64,
    pub sm: f64,
    pub md: f64,
    pub lg: f64,
    pub full: f64,
}

impl Default for BorderRadius {
    fn default() -> Self {
        Self {
            none: 0.0,
            sm: 4.0,
            md: 8.0,
            lg: 16.0,
            full: 9999.0,
        }
    }
}

/// Shadow tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shadows {
    pub none: String,
    pub sm: String,
    pub md: String,
    pub lg: String,
}

impl Default for Shadows {
    fn default() -> Self {
        Self {
            none: "none".to_string(),
            sm: "0 1px 2px rgba(0,0,0,0.1)".to_string(),
            md: "0 4px 6px rgba(0,0,0,0.1)".to_string(),
            lg: "0 10px 15px rgba(0,0,0,0.1)".to_string(),
        }
    }
}

/// Theme mode
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
    System,
}

/// Complete theme definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub mode: ThemeMode,
    pub colors: ColorPalette,
    pub dark_colors: Option<ColorPalette>,
    pub typography: Typography,
    pub spacing: Spacing,
    pub border_radius: BorderRadius,
    pub shadows: Shadows,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            mode: ThemeMode::Light,
            colors: ColorPalette::default(),
            dark_colors: Some(ColorPalette {
                primary: "#90CAF9".to_string(),
                secondary: "#B0BEC5".to_string(),
                background: "#121212".to_string(),
                surface: "#1E1E1E".to_string(),
                error: "#EF5350".to_string(),
                warning: "#FFB74D".to_string(),
                success: "#66BB6A".to_string(),
                info: "#42A5F5".to_string(),
                text_primary: "#FFFFFF".to_string(),
                text_secondary: "#B0BEC5".to_string(),
                text_disabled: "#616161".to_string(),
                divider: "#424242".to_string(),
            }),
            typography: Typography::default(),
            spacing: Spacing::default(),
            border_radius: BorderRadius::default(),
            shadows: Shadows::default(),
        }
    }
}

impl Theme {
    /// Get the active color palette based on mode
    pub fn active_colors(&self) -> &ColorPalette {
        match self.mode {
            ThemeMode::Dark => self.dark_colors.as_ref().unwrap_or(&self.colors),
            _ => &self.colors,
        }
    }
}
