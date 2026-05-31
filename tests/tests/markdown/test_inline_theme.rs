use hudhudscript_markdown::theme::*;

#[test]
fn dark_theme_creates_valid_theme() {
    let theme = dark_theme();
    assert!(!theme.h1.fg.is_empty());
    assert!(!theme.syntax.keyword.fg.is_empty());
}

#[test]
fn light_theme_creates_valid_theme() {
    let theme = light_theme();
    assert!(!theme.h1.fg.is_empty());
    assert!(!theme.syntax.keyword.fg.is_empty());
}

#[test]
fn dark_theme_all_fields_non_empty() {
    let theme = dark_theme();
    assert!(!theme.h2.fg.is_empty());
    assert!(!theme.h3.fg.is_empty());
    assert!(!theme.bold.fg.is_empty());
    assert!(!theme.italic.fg.is_empty());
    assert!(!theme.inline_code.fg.is_empty());
    assert!(!theme.code_block_border.fg.is_empty());
    assert!(!theme.blockquote.fg.is_empty());
    assert!(!theme.list_marker.fg.is_empty());
    assert!(!theme.link_url.fg.is_empty());
    assert!(!theme.link_text.fg.is_empty());
    assert!(!theme.table_border.fg.is_empty());
    assert!(!theme.horizontal_rule.fg.is_empty());
    assert!(!theme.syntax.string.fg.is_empty());
    assert!(!theme.syntax.number.fg.is_empty());
    assert!(!theme.syntax.comment.fg.is_empty());
    assert!(!theme.syntax.type_name.fg.is_empty());
    assert!(!theme.syntax.function.fg.is_empty());
    assert!(!theme.syntax.operator.fg.is_empty());
    assert!(!theme.syntax.plain.fg.is_empty());
}

#[test]
fn light_theme_all_fields_non_empty() {
    let theme = light_theme();
    assert!(!theme.h2.fg.is_empty());
    assert!(!theme.h3.fg.is_empty());
    assert!(!theme.bold.fg.is_empty());
    assert!(!theme.italic.fg.is_empty());
    assert!(!theme.inline_code.fg.is_empty());
    assert!(!theme.code_block_border.fg.is_empty());
    assert!(!theme.blockquote.fg.is_empty());
    assert!(!theme.list_marker.fg.is_empty());
    assert!(!theme.link_url.fg.is_empty());
    assert!(!theme.link_text.fg.is_empty());
    assert!(!theme.table_border.fg.is_empty());
    assert!(!theme.horizontal_rule.fg.is_empty());
    assert!(!theme.syntax.string.fg.is_empty());
    assert!(!theme.syntax.number.fg.is_empty());
    assert!(!theme.syntax.comment.fg.is_empty());
    assert!(!theme.syntax.type_name.fg.is_empty());
    assert!(!theme.syntax.function.fg.is_empty());
    assert!(!theme.syntax.operator.fg.is_empty());
    assert!(!theme.syntax.plain.fg.is_empty());
}

#[test]
fn dark_and_light_themes_differ() {
    let dark = dark_theme();
    let light = light_theme();
    assert_ne!(dark.h1.fg, light.h1.fg);
}

#[test]
fn color_new_stores_fg() {
    let c = Color::new("\x1b[31m");
    assert_eq!(c.fg, "\x1b[31m");
}

#[test]
fn ansi_constants_are_escape_sequences() {
    assert!(RESET.starts_with("\x1b["));
    assert!(BOLD.starts_with("\x1b["));
    assert!(ITALIC.starts_with("\x1b["));
    assert!(UNDERLINE.starts_with("\x1b["));
    assert!(DIM.starts_with("\x1b["));
}

#[test]
fn ansi_constants_exact_values() {
    assert_eq!(RESET, "\x1b[0m");
    assert_eq!(BOLD, "\x1b[1m");
    assert_eq!(ITALIC, "\x1b[3m");
    assert_eq!(UNDERLINE, "\x1b[4m");
    assert_eq!(DIM, "\x1b[2m");
}

#[test]
fn color_copy_and_eq() {
    let c1 = Color::new("\x1b[31m");
    let c2 = c1;
    assert_eq!(c1, c2);
}

#[test]
fn theme_clone_inline() {
    let theme = dark_theme();
    let cloned = theme.clone();
    assert_eq!(cloned.h1.fg, theme.h1.fg);
    assert_eq!(cloned.syntax.keyword.fg, theme.syntax.keyword.fg);
}

#[test]
fn light_theme_syntax_colors_differ_from_dark() {
    let dark = dark_theme();
    let light = light_theme();
    assert_ne!(dark.syntax.number.fg, light.syntax.number.fg);
    assert_ne!(dark.syntax.plain.fg, light.syntax.plain.fg);
}

#[test]
fn dark_theme_bold_color_is_white() {
    let theme = dark_theme();
    assert_eq!(theme.bold.fg, "\x1b[37m");
}

#[test]
fn light_theme_bold_color_is_black() {
    let theme = light_theme();
    assert_eq!(theme.bold.fg, "\x1b[30m");
}
