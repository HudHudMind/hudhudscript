//! Shared MPRIS D-Bus media player control (Issue #638).
//!
//! Single source of truth for VM and interpreter runtimes (Kural 7).
//! Shells out to `gdbus` or `dbus-send`.

use hudhudscript_bytecode::shared_value::{runtime_error, SharedResult};
use hudhudscript_bytecode::Value16;

mod discovery;
pub mod dispatch;
mod metadata_ops;
mod player_ops;

pub(crate) use discovery::{has_gdbus, list_mpris_players, resolve_player, short_name};
pub use dispatch::*;
pub(crate) use metadata_ops::*;
pub(crate) use player_ops::*;

const MPRIS_PREFIX: &str = "org.mpris.MediaPlayer2.";
const MPRIS_PATH: &str = "/org/mpris/MediaPlayer2";
const MPRIS_PLAYER_IFACE: &str = "org.mpris.MediaPlayer2.Player";

pub fn call_mpris_method(method: &str, args: &[Value16]) -> SharedResult<Value16> {
    match method {
        "players" => dispatch::mpris_players(args),
        "play" => dispatch::mpris_play(args),
        "pause" => dispatch::mpris_pause(args),
        "play_pause" => dispatch::mpris_play_pause(args),
        "stop" => dispatch::mpris_stop(args),
        "next" => dispatch::mpris_next(args),
        "previous" => dispatch::mpris_previous(args),
        "status" => dispatch::mpris_status(args),
        "volume" => dispatch::mpris_volume(args),
        "seek" => dispatch::mpris_seek(args),
        _ => Err(runtime_error(format!("Unknown mpris method: {}", method))),
    }
}
