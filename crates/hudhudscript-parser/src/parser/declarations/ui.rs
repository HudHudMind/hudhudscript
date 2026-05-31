//! UI declaration parsing (#538)
//!
//! Parses `ui App { ... }`, `screen`, `component`, widget trees, events, vars, platform blocks.

use hudhudscript_ast::{Decl, Expr, Stmt, UiComponentDecl, UiNode, UiScreenDecl};
use pest::iterators::Pair;

use crate::error::{parse_codes, ParseResult};
use crate::parser::{pair_to_span, parse_expression};
use crate::pest_parser::Rule;

/// Parse a UI app declaration: `ui MyApp { entry: Main; screen Main { ... } component Card { ... } }`
pub fn parse_ui_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected UI app name", span))?
        .as_str()
        .to_string();

    let mut entry_screen: Option<String> = None;
    let mut screens: Vec<UiScreenDecl> = Vec::new();
    let mut components: Vec<UiComponentDecl> = Vec::new();

    for member_pair in inner {
        if member_pair.as_rule() == Rule::ui_member {
            for child in member_pair.into_inner() {
                match child.as_rule() {
                    Rule::ui_entry => {
                        // entry: ScreenName
                        if let Some(id) = child.into_inner().next() {
                            entry_screen = Some(id.as_str().to_string());
                        }
                    }
                    Rule::screen_decl => {
                        screens.push(parse_screen_decl(child)?);
                    }
                    Rule::component_decl => {
                        components.push(parse_component_decl(child)?);
                    }
                    Rule::ui_field => {
                        // Generic key: value field — silently accepted as metadata
                        // (stored if needed in future; for now just validates successfully)
                        let _ = child; // consumed, no further processing needed
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(Stmt::Decl(Decl::UiApp {
        name,
        entry_screen,
        screens,
        components,
        span,
    }))
}

/// Parse a screen declaration: `screen Main(param1, param2) { ... }`
fn parse_screen_decl(pair: Pair<Rule>) -> ParseResult<UiScreenDecl> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected screen name", span))?
        .as_str()
        .to_string();

    let mut params: Vec<String> = Vec::new();
    let mut body: Vec<UiNode> = Vec::new();

    for child in inner {
        match child.as_rule() {
            Rule::identifier => {
                params.push(child.as_str().to_string());
            }
            Rule::ui_body => {
                if let Some(node) = parse_ui_body(child)? {
                    body.push(node);
                }
            }
            _ => {}
        }
    }

    Ok(UiScreenDecl {
        name,
        params,
        body,
        span,
    })
}

/// Parse a component declaration: `component Card { ... }`
fn parse_component_decl(pair: Pair<Rule>) -> ParseResult<UiComponentDecl> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected component name", span))?
        .as_str()
        .to_string();

    let mut body: Vec<UiNode> = Vec::new();
    for child in inner {
        if child.as_rule() == Rule::ui_body {
            if let Some(node) = parse_ui_body(child)? {
                body.push(node);
            }
        }
    }

    Ok(UiComponentDecl {
        name,
        props: vec![],
        body,
        span,
    })
}

/// Parse a single ui_body item → UiNode
fn parse_ui_body(pair: Pair<Rule>) -> ParseResult<Option<UiNode>> {
    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::ui_widget => return Ok(Some(parse_ui_widget(child)?)),
            Rule::ui_var => return Ok(Some(parse_ui_var(child)?)),
            Rule::ui_event => return Ok(Some(parse_ui_event(child)?)),
            Rule::ui_platform => return Ok(Some(parse_ui_platform(child)?)),
            Rule::expression => {
                let expr = parse_expression(child)?;
                return Ok(Some(UiNode::Expr(expr)));
            }
            _ => {}
        }
    }
    Ok(None)
}

/// Parse a widget: `Button "Click me" { color: "red" } { children... }`
fn parse_ui_widget(pair: Pair<Rule>) -> ParseResult<UiNode> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let widget_type = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected widget type", span))?
        .as_str()
        .to_string();

    let mut label: Option<Expr> = None;
    let mut props: Vec<(String, Expr)> = Vec::new();
    let mut events: Vec<(String, Expr)> = Vec::new();
    let mut children: Vec<UiNode> = Vec::new();
    let mut style: Vec<(String, Expr)> = Vec::new();

    for child in inner {
        match child.as_rule() {
            Rule::string | Rule::expression => {
                if label.is_none() {
                    label = Some(parse_expression(child)?);
                }
            }
            Rule::ui_widget_prop => {
                let mut pi = child.into_inner();
                if let (Some(key_pair), Some(val_pair)) = (pi.next(), pi.next()) {
                    let key = key_pair.as_str().to_string();
                    let value = parse_expression(val_pair)?;
                    // Style-related props go to style, rest to props
                    if is_style_prop(&key) {
                        style.push((key, value));
                    } else {
                        props.push((key, value));
                    }
                }
            }
            Rule::ui_body => {
                if let Some(node) = parse_ui_body(child)? {
                    // Check if child is an event node
                    match &node {
                        UiNode::Event { .. } => events.push(match node {
                            UiNode::Event {
                                event_name,
                                handler,
                                ..
                            } => (event_name, handler),
                            _ => unreachable!(),
                        }),
                        _ => children.push(node),
                    }
                }
            }
            _ => {}
        }
    }

    Ok(UiNode::Widget {
        widget_type,
        label,
        props,
        events,
        children,
        style,
        span,
    })
}

/// Parse a UI var: `var count = 0`
fn parse_ui_var(pair: Pair<Rule>) -> ParseResult<UiNode> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected variable name", span))?
        .as_str()
        .to_string();

    let value = parse_expression(
        inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected expression", span))?,
    )?;

    Ok(UiNode::Var { name, value, span })
}

/// Parse a UI event: `on_click: handler_expr`
fn parse_ui_event(pair: Pair<Rule>) -> ParseResult<UiNode> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let event_name = format!(
        "on_{}",
        inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected event name", span))?
            .as_str()
    );

    let handler = parse_expression(
        inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected event handler", span))?,
    )?;

    Ok(UiNode::Event {
        event_name,
        handler,
        span,
    })
}

/// Parse a platform block: `platform:desktop { ... }`
fn parse_ui_platform(pair: Pair<Rule>) -> ParseResult<UiNode> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let platform = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected platform name", span))?
        .as_str()
        .to_string();

    let mut body: Vec<UiNode> = Vec::new();
    for child in inner {
        if child.as_rule() == Rule::ui_body {
            if let Some(node) = parse_ui_body(child)? {
                body.push(node);
            }
        }
    }

    Ok(UiNode::PlatformBlock {
        platform,
        body,
        span,
    })
}

/// Check if a property name is a style property
fn is_style_prop(name: &str) -> bool {
    matches!(
        name,
        "size"
            | "color"
            | "background"
            | "padding"
            | "margin"
            | "border_radius"
            | "bold"
            | "align"
            | "width"
            | "height"
            | "display"
            | "flex_direction"
            | "justify_content"
            | "align_items"
            | "flex_wrap"
            | "flex_grow"
            | "flex_shrink"
            | "gap"
            | "overflow"
            | "opacity"
            | "shadow"
            | "border_width"
            | "border_color"
    )
}
