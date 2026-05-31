//! Constructor functions for type-stage errors.

use hudhudscript_ast::Span;
use hudhudscript_errors::{Error, ErrorCode, SourcePosition};

fn span_to_source(span: Span) -> SourcePosition {
    SourcePosition::new(span.start.line, span.start.column, span.start.offset)
}

pub fn mismatch(expected: impl Into<String>, found: impl Into<String>, span: Span) -> Error {
    let expected = expected.into();
    let found = found.into();
    Error::new(
        ErrorCode::TypeMismatch,
        format!("Type mismatch: expected {}, found {}", expected, found),
    )
    .at(span_to_source(span))
    .with_context("expected", expected)
    .with_context("found", found)
}

pub fn type_mismatch_in_decl(
    expected: impl Into<String>,
    found: impl Into<String>,
    variable: impl Into<String>,
    span: Span,
) -> Error {
    let expected = expected.into();
    let found = found.into();
    let variable = variable.into();
    Error::new(
        ErrorCode::TypeTypeMismatchInDecl,
        format!(
            "Type error: expected {}, got {} in declaration of '{}'",
            expected, found, variable
        ),
    )
    .at(span_to_source(span))
    .with_context("expected", expected)
    .with_context("found", found)
    .with_context("variable", variable)
}

pub fn undefined_variable(name: impl Into<String>, span: Span) -> Error {
    let name = name.into();
    Error::new(
        ErrorCode::TypeUndefinedVariable,
        format!("Undefined variable: {}", name),
    )
    .at(span_to_source(span))
    .with_context("name", name)
}

pub fn undefined_function(name: impl Into<String>, span: Span) -> Error {
    let name = name.into();
    Error::new(
        ErrorCode::TypeUndefinedFunction,
        format!("Undefined function: {}", name),
    )
    .at(span_to_source(span))
    .with_context("name", name)
}

pub fn wrong_argument_count(expected: usize, found: usize, span: Span) -> Error {
    Error::new(
        ErrorCode::TypeWrongArgumentCount,
        format!(
            "Wrong number of arguments: expected {}, found {}",
            expected, found
        ),
    )
    .at(span_to_source(span))
    .with_context("expected", expected.to_string())
    .with_context("found", found.to_string())
}

pub fn invalid_operator(op: impl Into<String>, ty: impl Into<String>, span: Span) -> Error {
    let op = op.into();
    let ty = ty.into();
    Error::new(
        ErrorCode::TypeInvalidOperator,
        format!("Cannot apply operator {} to type {}", op, ty),
    )
    .at(span_to_source(span))
    .with_context("op", op)
    .with_context("ty", ty)
}

pub fn invalid_index(ty: impl Into<String>, span: Span) -> Error {
    let ty = ty.into();
    Error::new(
        ErrorCode::TypeInvalidIndex,
        format!("Cannot index type {}", ty),
    )
    .at(span_to_source(span))
    .with_context("ty", ty)
}

pub fn invalid_member(ty: impl Into<String>, member: impl Into<String>, span: Span) -> Error {
    let ty = ty.into();
    let member = member.into();
    Error::new(
        ErrorCode::TypeInvalidMember,
        format!("Cannot access member {} on type {}", member, ty),
    )
    .at(span_to_source(span))
    .with_context("ty", ty)
    .with_context("member", member)
}

pub fn duplicate_variable(name: impl Into<String>, span: Span) -> Error {
    let name = name.into();
    Error::new(
        ErrorCode::TypeDuplicateVariable,
        format!("Duplicate variable declaration: {}", name),
    )
    .at(span_to_source(span))
    .with_context("name", name)
}

pub fn invalid_await(ty: impl Into<String>, span: Span) -> Error {
    let ty = ty.into();
    Error::new(
        ErrorCode::TypeInvalidAwait,
        format!("Cannot await non-promise type: {}", ty),
    )
    .at(span_to_source(span))
    .with_context("ty", ty)
}

pub fn await_in_atomically(span: Span) -> Error {
    Error::new(
        ErrorCode::TypeAwaitInAtomically,
        "await is not allowed inside atomically() blocks — suspending mid-transaction violates isolation",
    )
    .at(span_to_source(span))
}

pub fn non_exhaustive_match(
    union_type: impl Into<String>,
    missing: impl Into<String>,
    span: Span,
) -> Error {
    let union_type = union_type.into();
    let missing = missing.into();
    Error::new(
        ErrorCode::TypeNonExhaustiveMatch,
        format!(
            "Non-exhaustive match on union type {}: missing variants {}",
            union_type, missing
        ),
    )
    .at(span_to_source(span))
    .with_context("union_type", union_type)
    .with_context("missing", missing)
}
