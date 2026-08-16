//! Statement parsing

use hudhudscript_ast::{
    AccessModifier, CatchClause, ClassDecl, ClassMember, Decl, EnumVariant, Expr, GenericParam,
    Literal, MatchArm, MatchPattern, Param, Pattern, Stmt, SwitchCase, TraitMethodSig,
    TypeAnnotation,
};
use pest::iterators::Pair;

use crate::error::{parse_codes, ParseResult};
use crate::pest_parser::Rule;

use super::{pair_to_span, parse_expression};

/// Resolve a type name string to its TypeAnnotation variant.
fn resolve_type_name(name: &str) -> TypeAnnotation {
    match name {
        "String" | "string" => TypeAnnotation::String,
        "Number" | "number" => TypeAnnotation::Number,
        "Boolean" | "boolean" | "bool" => TypeAnnotation::Boolean,
        "Null" | "null" => TypeAnnotation::Null,
        "Any" | "any" => TypeAnnotation::Any,
        "Tool" | "tool" => TypeAnnotation::Tool,
        "Resource" | "resource" => TypeAnnotation::Resource,
        "Server" | "server" => TypeAnnotation::Server,
        _ => TypeAnnotation::Generic(name.to_string()),
    }
}

/// Parse a type_single rule into a TypeAnnotation.
///
/// Grammar: `type_single = { identifier ~ ("<" ~ type_annotation ~ ... ~ ">")? ~ question? }`
///
/// Because `question` is a silent rule in pest, it is consumed but does not appear
/// as a child node. We detect it by checking whether the full matched text of the
/// `type_single` pair ends with a question-mark character.
fn parse_type_single(pair: Pair<Rule>) -> TypeAnnotation {
    let full_text = pair.as_str().trim();
    let is_optional =
        full_text.ends_with('?') || full_text.ends_with('؟') || full_text.ends_with('？');

    let mut inner = pair.into_inner();

    // First child is the identifier (base type name)
    let ident_pair = inner.next().expect("type_single must have an identifier");
    let type_name = ident_pair.as_str();

    // Collect generic type arguments: <T, U, ...>
    let type_args: Vec<TypeAnnotation> = inner
        .filter(|child| child.as_rule() == Rule::type_annotation)
        .map(parse_type_annotation_rule)
        .collect();

    let mut base = resolve_type_name(type_name);

    if !type_args.is_empty() {
        // Special case: Array<T> → Array(Box<T>)
        if matches!(base, TypeAnnotation::Generic(ref s) if s == "Array") && type_args.len() == 1 {
            base = TypeAnnotation::Array(Box::new(type_args.into_iter().next().unwrap()));
        } else {
            base = TypeAnnotation::Parameterized {
                base: Box::new(base),
                args: type_args,
            };
        }
    }

    // T? → Union(T, Null)
    if is_optional {
        base = TypeAnnotation::Union(vec![base, TypeAnnotation::Null]);
    }

    base
}

/// Parse a type_annotation rule: `type_single ("|" type_single)*`
///
/// Returns a single type if there is one alternative, or a Union if multiple.
fn parse_type_annotation_rule(pair: Pair<Rule>) -> TypeAnnotation {
    let children: Vec<Pair<Rule>> = pair.into_inner().collect();

    if children.len() == 1 {
        parse_type_single(children.into_iter().next().unwrap())
    } else {
        let types: Vec<TypeAnnotation> = children.into_iter().map(parse_type_single).collect();
        TypeAnnotation::Union(types)
    }
}

/// Parse a type_annotation rule from a Pair — public interface for other modules.
pub fn parse_type_annotation(pair: Pair<Rule>) -> TypeAnnotation {
    parse_type_annotation_rule(pair)
}

/// Parse a func_param into a Param with optional type annotation.
///
fn parse_func_param(pair: Pair<Rule>) -> Param {
    let mut children = pair.into_inner();
    let mut first = children
        .next()
        .expect("func_param must have at least one child");

    // Skip optional let/var keyword before parameter name (Turkish değişken → let)
    if first.as_rule() == Rule::let_kw || first.as_rule() == Rule::var_kw {
        first = children.next().expect("Expected identifier after let/var");
    }

    match first.as_rule() {
        Rule::rest_param => {
            let rest_name = first.into_inner().next().unwrap();
            let span = pair_to_span(&rest_name);
            Param {
                name: format!("...{}", rest_name.as_str()),
                type_annotation: None,
                span,
            }
        }
        Rule::identifier => {
            let span = pair_to_span(&first);
            let name = first.as_str().to_string();
            let type_ann = children
                .find(|c| c.as_rule() == Rule::type_annotation)
                .map(parse_type_annotation);
            Param {
                name,
                type_annotation: type_ann,
                span,
            }
        }
        // Destructuring patterns get a synthetic name; actual destructuring handled by callers
        Rule::destruct_object_pattern | Rule::destruct_array_pattern => {
            let span = pair_to_span(&first);
            Param {
                name: "__destruct_param".to_string(),
                type_annotation: None,
                span,
            }
        }
        _ => {
            let span = pair_to_span(&first);
            Param {
                name: first.as_str().to_string(),
                type_annotation: None,
                span,
            }
        }
    }
}

/// Parse a func_param and return the param name plus an optional destructuring pattern.
///
/// For simple params (identifier or rest), returns (name, None).
/// For destructuring patterns ({a, b} or [x, y]), returns a synthetic name and the pattern
/// for desugaring into a `Stmt::Destructure` at the start of the function body.
///
/// Issue #1015: Function parameter destructuring support.
pub(crate) fn parse_func_param_with_pattern(
    pair: Pair<Rule>,
    destr_index: &mut usize,
) -> ParseResult<(String, Option<Pattern>)> {
    let mut children = pair.into_inner();
    let mut first = children
        .next()
        .expect("func_param must have at least one child");

    if first.as_rule() == Rule::let_kw || first.as_rule() == Rule::var_kw {
        first = children.next().expect("Expected identifier after let/var");
    }

    match first.as_rule() {
        Rule::rest_param => {
            let rest_name = first.into_inner().next().unwrap();
            Ok((format!("...{}", rest_name.as_str()), None))
        }
        Rule::identifier => {
            let name = first.as_str().to_string();
            Ok((name, None))
        }
        Rule::destruct_object_pattern | Rule::destruct_array_pattern => {
            let pattern = parse_destruct_pattern(first)?;
            let synthetic_name = format!("__destruct_{}", *destr_index);
            *destr_index += 1;
            Ok((synthetic_name, Some(pattern)))
        }
        _ => Ok((first.as_str().to_string(), None)),
    }
}

/// Parse a statement
pub fn parse_statement(pair: Pair<Rule>) -> ParseResult<Option<Stmt>> {
    let _span = pair_to_span(&pair);

    if let Some(inner_pair) = pair.into_inner().next() {
        return Ok(Some(match inner_pair.as_rule() {
            Rule::decorated_subject_decl => super::parse_decorated_subject_decl(inner_pair)?,
            Rule::subject_decl => super::parse_subject_decl(inner_pair)?,
            Rule::relation_decl => super::parse_relation_decl(inner_pair)?,
            Rule::effect_decl => super::parse_effect_decl(inner_pair)?,
            Rule::loop_decl => super::declarations::loop_engine::parse_loop_decl(inner_pair)?,
            Rule::step_decl => super::declarations::loop_engine::parse_step_decl(inner_pair)?,
            Rule::gate_decl => super::declarations::loop_engine::parse_gate_decl(inner_pair)?,
            Rule::chain_decl => super::declarations::loop_engine::parse_chain_decl(inner_pair)?,
            Rule::run_stmt => super::declarations::loop_engine::parse_run_stmt(inner_pair)?,
            Rule::run_chain_stmt => {
                super::declarations::loop_engine::parse_run_chain_stmt(inner_pair)?
            }
            Rule::attach_step_decl => {
                super::declarations::loop_engine::parse_attach_step_decl(inner_pair)?
            }
            Rule::compose_decl => super::parse_compose_decl(inner_pair)?,
            Rule::ability_decl => super::parse_ability_decl(inner_pair)?,
            Rule::trait_decl => parse_trait_decl(inner_pair)?,
            Rule::class_decl => parse_class_decl(inner_pair)?,
            Rule::entity_decl => parse_entity_decl(inner_pair)?,
            Rule::statemachine_decl => parse_statemachine_decl(inner_pair)?,
            Rule::event_decl => parse_event_decl(inner_pair)?,
            Rule::music_decl => parse_music_decl(inner_pair)?,
            Rule::contract_decl => parse_contract_decl(inner_pair)?,
            Rule::treaty_decl => parse_treaty_decl(inner_pair)?,
            Rule::constitution_decl => super::parse_constitution_decl(inner_pair)?,
            Rule::law_decl => super::parse_law_decl(inner_pair)?,
            Rule::council_decl => super::parse_council_decl(inner_pair)?,
            Rule::role_decl => super::parse_role_decl(inner_pair)?,
            Rule::protocol_decl => super::parse_protocol_decl(inner_pair)?,
            Rule::governance_decl => super::parse_governance_decl(inner_pair)?,
            Rule::rule_decl => super::parse_rule_decl(inner_pair)?,
            Rule::swarm_decl => super::parse_swarm_decl(inner_pair)?,
            Rule::community_decl => super::parse_community_decl(inner_pair)?,
            Rule::provider_decl => super::parse_provider_decl(inner_pair)?,
            Rule::store_decl => parse_store_decl(inner_pair)?,
            Rule::remember_stmt => parse_remember_stmt(inner_pair)?,
            Rule::recall_stmt => parse_recall_stmt(inner_pair)?,
            Rule::forget_stmt => parse_forget_stmt(inner_pair)?,
            Rule::agent_decl => super::parse_agent_decl(inner_pair)?,
            Rule::mcp_decl => super::parse_mcp_decl(inner_pair)?,
            Rule::action_decl => super::parse_action_decl(inner_pair)?,
            Rule::data_decl => parse_data_decl(inner_pair)?,
            Rule::ui_decl => super::parse_ui_decl(inner_pair)?,
            Rule::deploy_decl => super::parse_deploy_decl(inner_pair)?,
            Rule::resource_decl => super::parse_resource_decl(inner_pair)?,
            Rule::async_function_decl => parse_async_function_decl(inner_pair)?,
            Rule::generator_decl => parse_generator_decl(inner_pair)?,
            Rule::function_decl => parse_function_decl(inner_pair)?,
            Rule::export_stmt => parse_export_stmt(inner_pair)?,
            Rule::import_stmt => super::parse_import_stmt(inner_pair)?,
            Rule::destructure_var_stmt => parse_destructure_stmt(inner_pair, false)?,
            Rule::destructure_let_stmt => parse_destructure_stmt(inner_pair, false)?,
            Rule::destructure_const_stmt => parse_destructure_stmt(inner_pair, true)?,
            Rule::var_stmt => parse_var_stmt(inner_pair)?,
            Rule::let_stmt => parse_let_stmt(inner_pair)?,
            Rule::const_stmt => parse_const_stmt(inner_pair)?,
            Rule::assignment_stmt => parse_assignment_stmt(inner_pair)?,
            Rule::return_stmt => parse_return_stmt(inner_pair)?,
            Rule::break_stmt => parse_break_stmt(inner_pair)?,
            Rule::continue_stmt => parse_continue_stmt(inner_pair)?,
            Rule::switch_stmt => parse_switch_stmt(inner_pair)?,
            Rule::try_stmt => parse_try_stmt(inner_pair)?,
            Rule::throw_stmt => parse_throw_stmt(inner_pair)?,
            Rule::spawn_stmt => parse_spawn_stmt(inner_pair)?,
            Rule::despawn_stmt => parse_despawn_stmt(inner_pair)?,
            Rule::send_stmt => parse_send_stmt(inner_pair)?,
            Rule::receive_stmt => parse_receive_stmt(inner_pair)?,
            Rule::require_stmt => parse_require_stmt(inner_pair)?,
            Rule::perform_stmt => parse_perform_stmt(inner_pair)?,
            Rule::match_stmt => parse_match_stmt(inner_pair)?,
            Rule::enum_decl => parse_enum_decl(inner_pair)?,
            Rule::if_stmt => parse_if_stmt(inner_pair)?,
            Rule::while_stmt => parse_while_stmt(inner_pair)?,
            Rule::for_stmt => parse_for_stmt(inner_pair)?,
            Rule::for_c_style_stmt => parse_for_c_style_stmt(inner_pair)?,
            Rule::for_range_stmt => parse_for_range_stmt(inner_pair)?,
            Rule::herbir_stmt => parse_herbir_stmt(inner_pair)?,
            Rule::expr_stmt => parse_expr_stmt(inner_pair)?,
            Rule::block => parse_block(inner_pair)?,
            _ => return Ok(None),
        }));
    }

    Ok(None)
}

pub fn parse_var_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    use hudhudscript_ast::{OwnershipMode, VarDecl};

    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected variable name", span))?
        .as_str()
        .to_string();

    // Next child is either type_annotation or expression
    let next = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected expression", span))?;

    let (type_ann, value) = if next.as_rule() == Rule::type_annotation {
        let ta = Some(parse_type_annotation(next));
        let expr_pair = inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected expression after type", span))?;
        (ta, parse_expression(expr_pair)?)
    } else {
        (None, parse_expression(next)?)
    };

    // #744: `var` produces VarDecl (mutable, no TDZ), `let` produces Stmt::Let (with TDZ for const)
    Ok(Stmt::VarDecl(VarDecl {
        name,
        initializer: Some(value),
        type_annotation: type_ann,
        is_const: false,
        ownership: OwnershipMode::Owned,
        span,
    }))
}

pub fn parse_let_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    use hudhudscript_ast::{OwnershipMode, VarDecl};

    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected variable name", span))?
        .as_str()
        .to_string();

    let next = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected expression", span))?;

    let (type_ann, value) = if next.as_rule() == Rule::type_annotation {
        let ta = Some(parse_type_annotation(next));
        let expr_pair = inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected expression after type", span))?;
        (ta, parse_expression(expr_pair)?)
    } else {
        (None, parse_expression(next)?)
    };

    // If there's a type annotation, use VarDecl (richer form). Otherwise keep Stmt::Let
    // for backward compatibility with the rest of the pipeline.
    if type_ann.is_some() {
        Ok(Stmt::VarDecl(VarDecl {
            name,
            initializer: Some(value),
            type_annotation: type_ann,
            is_const: false,
            ownership: OwnershipMode::Owned,
            span,
        }))
    } else {
        Ok(Stmt::Let { name, value, span })
    }
}

pub fn parse_const_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    use hudhudscript_ast::{OwnershipMode, VarDecl};

    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected constant name", span))?
        .as_str()
        .to_string();

    let next = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected expression", span))?;

    let (type_ann, value) = if next.as_rule() == Rule::type_annotation {
        let ta = Some(parse_type_annotation(next));
        let expr_pair = inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected expression after type", span))?;
        (ta, parse_expression(expr_pair)?)
    } else {
        (None, parse_expression(next)?)
    };

    if type_ann.is_some() {
        Ok(Stmt::VarDecl(VarDecl {
            name,
            initializer: Some(value),
            type_annotation: type_ann,
            is_const: true,
            ownership: OwnershipMode::Owned,
            span,
        }))
    } else {
        Ok(Stmt::Const { name, value, span })
    }
}

pub mod control_flow;
pub mod declarations;
pub mod for_c_style;
pub mod generators;
pub mod pattern;
pub mod special;
pub mod types;

pub use control_flow::*;
pub use declarations::*;
pub use for_c_style::*;
pub use generators::*;
pub use pattern::*;
pub use special::*;
pub use types::*;
