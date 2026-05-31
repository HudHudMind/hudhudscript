use hudhudscript_ui_core::theme::{
    BorderRadius, ColorPalette, Shadows, Spacing, Theme, ThemeMode, Typography,
};

#[test]
fn test_default_theme() {
    let theme = Theme::default();
    assert_eq!(theme.name, "default");
    assert_eq!(theme.mode, ThemeMode::Light);
    assert_eq!(theme.spacing.md, 16.0);
}

#[test]
fn test_dark_mode_colors() {
    let mut theme = Theme::default();
    theme.mode = ThemeMode::Dark;
    let colors = theme.active_colors();
    assert_eq!(colors.background, "#121212");
}

#[test]
fn test_theme_serialization() {
    let theme = Theme::default();
    let json = serde_json::to_string(&theme).unwrap();
    assert!(json.contains("default"));
    assert!(json.contains("1976D2"));
}

#[test]
fn test_light_mode_colors() {
    let theme = Theme::default();
    let colors = theme.active_colors();
    assert_eq!(colors.primary, "#1976D2");
    assert_eq!(colors.background, "#FFFFFF");
}

#[test]
fn test_system_mode_uses_light_colors() {
    let mut theme = Theme::default();
    theme.mode = ThemeMode::System;
    // System mode defaults to light colors (falls to _ arm)
    let colors = theme.active_colors();
    assert_eq!(colors.background, "#FFFFFF");
}

#[test]
fn test_dark_mode_no_dark_colors_falls_back() {
    let mut theme = Theme::default();
    theme.mode = ThemeMode::Dark;
    theme.dark_colors = None;
    // When dark_colors is None, falls back to light colors
    let colors = theme.active_colors();
    assert_eq!(colors.background, "#FFFFFF");
}

#[test]
fn test_color_palette_default_all_fields() {
    let palette = ColorPalette::default();
    assert_eq!(palette.error, "#D32F2F");
    assert_eq!(palette.warning, "#FFA000");
    assert_eq!(palette.success, "#388E3C");
    assert_eq!(palette.info, "#1976D2");
    assert_eq!(palette.text_disabled, "#BDBDBD");
    assert_eq!(palette.divider, "#E0E0E0");
}

#[test]
fn test_typography_defaults() {
    let typo = Typography::default();
    assert_eq!(typo.font_size_xs, 10.0);
    assert_eq!(typo.font_size_xxl, 32.0);
    assert_eq!(typo.font_weight_bold, 700);
    assert_eq!(typo.line_height, 1.5);
}

#[test]
fn test_spacing_defaults() {
    let spacing = Spacing::default();
    assert_eq!(spacing.xs, 4.0);
    assert_eq!(spacing.xxl, 48.0);
}

#[test]
fn test_border_radius_defaults() {
    let br = BorderRadius::default();
    assert_eq!(br.none, 0.0);
    assert_eq!(br.full, 9999.0);
}

#[test]
fn test_shadows_defaults() {
    let shadows = Shadows::default();
    assert_eq!(shadows.none, "none");
    assert!(shadows.sm.contains("rgba"));
}

#[test]
fn test_theme_mode_default() {
    let mode = ThemeMode::default();
    assert_eq!(mode, ThemeMode::Light);
}
