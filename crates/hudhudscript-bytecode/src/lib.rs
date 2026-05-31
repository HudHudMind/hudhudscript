//! Bytecode format and instructions

pub mod actor_registry;
pub mod bytecode_builder;
pub mod bytecode_struct;
pub mod cache_utils;
pub mod captures_serde;
pub mod dynamic;
pub mod error;
pub mod instruction;
pub mod interner;
pub mod packed_instruction;
pub mod payloads;
pub mod registry;
pub mod repr;
pub mod shared_value;
pub mod sym;
pub mod value16;
pub mod value16_impl;
pub mod value16_helpers;
pub mod value16_serde;
pub mod value16_shared;
pub mod value16_utils;
pub mod value_dto;
pub mod version;

pub use bytecode_struct::Bytecode;
pub use dynamic::{DynamicData, DynamicKind, DynamicObject};
pub use error::{CompileResult as BytecodeResult, SourcePosition};
pub use instruction::Instruction;
pub use payloads::*;
pub use repr::{Repr, ReprTag};
pub use sym::SymId;
pub use value16::Value16;
pub use value_dto::ValueDto;
pub use version::{
    GeneratorState, GeneratorState16, PromiseState, PromiseState16, BYTECODE_VERSION,
};
