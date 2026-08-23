use super::{BuiltinMember, BuiltinModule, MemberKind};

pub const BUILTIN_MODULES_DATABASE: &[BuiltinModule] = &[BuiltinModule {
    name: "database",
    description: "Pooled PostgreSQL, MySQL, and SQLite connections",
    members: &[
        BuiltinMember {
            name: "open",
            kind: MemberKind::Function,
            description: "Open a reusable database pool from a configuration object",
            params: &[("config", "object")],
            return_type: "DatabaseConnection",
        },
        BuiltinMember {
            name: "connect",
            kind: MemberKind::Function,
            description: "Alias of database.open",
            params: &[("config", "object")],
            return_type: "DatabaseConnection",
        },
    ],
}];
