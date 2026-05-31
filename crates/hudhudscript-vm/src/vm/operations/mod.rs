//! Generic value operations shared between the VM and the interpreter.
//!
//! This module is the single-source-of-truth counterpart to
//! `hudhudscript-builtins/src/operations/`.  The interpreter-era module
//! operated on the interpreter's concrete `Value` type; these modules are
//! generic over any `V: SharedValue`, so the same evaluator code is used by
//! both runtimes (Kural 7).
//!
//! Sub-modules:
//! - [`arithmetic`] — `+`, `-`, `*`, `/`, `%` and relational operators.
//! - [`helpers`]    — literals, unary ops, member/index access,
//!                    `Array`/`Object` constructors.
//! - [`validators`] — argument count and type validators used by builtin
//!                    methods.
//!
//! The interpreter-specific string-method dispatcher (`call_string_method`)
//! is *not* re-exported here because the original file was already a pure
//! delegation to `crate::vm::string::call_string_method`.

pub mod arithmetic;
pub mod helpers;
pub mod validators;
