mod globals;
mod indexing;
mod modules_core;
mod modules_io;
mod modules_system;
mod types;

pub use globals::BUILTIN_GLOBALS;
pub use indexing::{get_global, get_module};
pub use types::{BuiltinMember, BuiltinModule, MemberKind};

use std::sync::LazyLock;

/// Combined view of all builtin modules.
pub static BUILTIN_MODULES: LazyLock<Vec<&'static BuiltinModule>> = LazyLock::new(|| {
    let mut v = Vec::new();
    v.extend(modules_core::BUILTIN_MODULES_CORE.iter());
    v.extend(modules_io::BUILTIN_MODULES_IO.iter());
    v.extend(modules_system::BUILTIN_MODULES_SYSTEM.iter());
    v
});
