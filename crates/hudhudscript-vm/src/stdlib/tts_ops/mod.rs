//! Shared Text-to-Speech builtins — espeak-ng / piper / festival.
//!
//! Single source of truth for VM and interpreter runtimes (Kural 7).

use hudhudscript_bytecode::shared_value::{runtime_error, SharedResult};
use hudhudscript_bytecode::Value16;

mod dispatch;
pub mod engine_ops;
pub mod synthesis_ops;
pub mod voice_ops;

pub use dispatch::*;
pub(crate) use engine_ops::{find_engine, is_binary_available};
pub(crate) use synthesis_ops::{
    empty_opts, error_obj, extract_options, require_string, run_command,
};

const ENGINES: &[(&str, &str)] = &[
    ("espeak-ng", "espeak-ng"),
    ("piper", "piper"),
    ("festival", "festival"),
];

pub fn call_tts_method(method: &str, args: &[Value16]) -> SharedResult<Value16> {
    match method {
        "speak" => dispatch::tts_speak(args),
        "save" => dispatch::tts_save(args),
        "voices" => voice_ops::tts_voices(args),
        "engines" => engine_ops::tts_engines(args),
        "is_available" => engine_ops::tts_is_available(args),
        "ssml" => dispatch::tts_ssml(args),
        _ => Err(runtime_error(format!("Unknown tts method: {}", method))),
    }
}
