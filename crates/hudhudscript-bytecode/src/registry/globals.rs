use super::{BuiltinMember, MemberKind};

/// All global builtin functions (print, len, type, etc.)
pub const BUILTIN_GLOBALS: &[BuiltinMember] = &[
    BuiltinMember {
        name: "print",
        kind: MemberKind::Function,
        description: "Print values to stdout",
        params: &[("args", "...any")],
        return_type: "null",
    },
    BuiltinMember {
        name: "len",
        kind: MemberKind::Function,
        description: "Length of string, array, or object",
        params: &[("value", "any")],
        return_type: "number",
    },
    BuiltinMember {
        name: "type",
        kind: MemberKind::Function,
        description: "Type name of value",
        params: &[("value", "any")],
        return_type: "string",
    },
    BuiltinMember {
        name: "typeof",
        kind: MemberKind::Function,
        description: "Type name of value",
        params: &[("value", "any")],
        return_type: "string",
    },
    BuiltinMember {
        name: "toString",
        kind: MemberKind::Function,
        description: "Convert to string",
        params: &[("value", "any")],
        return_type: "string",
    },
    BuiltinMember {
        name: "toNumber",
        kind: MemberKind::Function,
        description: "Convert to number",
        params: &[("value", "any")],
        return_type: "number",
    },
    BuiltinMember {
        name: "toBoolean",
        kind: MemberKind::Function,
        description: "Convert to boolean",
        params: &[("value", "any")],
        return_type: "boolean",
    },
    BuiltinMember {
        name: "env",
        kind: MemberKind::Function,
        description: "Read environment variable",
        params: &[("key", "string")],
        return_type: "string",
    },
    BuiltinMember {
        name: "input",
        kind: MemberKind::Function,
        description: "Read line from stdin",
        params: &[("prompt", "string")],
        return_type: "string",
    },
    BuiltinMember {
        name: "sleep",
        kind: MemberKind::Function,
        description: "Sleep for milliseconds",
        params: &[("ms", "number")],
        return_type: "null",
    },
    // Actor primitives
    BuiltinMember {
        name: "spawn",
        kind: MemberKind::Function,
        description: "Spawn an actor from a function",
        params: &[("func", "function")],
        return_type: "object",
    },
    BuiltinMember {
        name: "send",
        kind: MemberKind::Function,
        description: "Send a message to an actor",
        params: &[("actor", "object"), ("message", "any")],
        return_type: "null",
    },
    BuiltinMember {
        name: "receive",
        kind: MemberKind::Function,
        description: "Receive a message from the actor mailbox",
        params: &[],
        return_type: "any",
    },
    // STM primitives
    BuiltinMember {
        name: "TVar",
        kind: MemberKind::Function,
        description: "Create a transactional variable",
        params: &[("initial", "any")],
        return_type: "object",
    },
    BuiltinMember {
        name: "readTVar",
        kind: MemberKind::Function,
        description: "Read a TVar inside a transaction",
        params: &[("tvar", "object")],
        return_type: "any",
    },
    BuiltinMember {
        name: "writeTVar",
        kind: MemberKind::Function,
        description: "Write to a TVar inside a transaction",
        params: &[("tvar", "object"), ("value", "any")],
        return_type: "null",
    },
    BuiltinMember {
        name: "atomically",
        kind: MemberKind::Function,
        description: "Execute a function atomically as an STM transaction",
        params: &[("func", "function")],
        return_type: "any",
    },
    // RAG primitives
    BuiltinMember {
        name: "remember",
        kind: MemberKind::Function,
        description: "Store a value in RAG memory — returns the new entry's id. \
                      Implemented by VM::rag_remember, the same code path as the \
                      `remember x in S;` statement.",
        params: &[("content", "any"), ("store", "string")],
        return_type: "string",
    },
    BuiltinMember {
        name: "recall",
        kind: MemberKind::Function,
        description: "Query RAG memory — returns ranked hits as \
                      [{ id, text, score }] (top 5). An empty query returns \
                      every stored item. Implemented by VM::rag_recall, the \
                      same code path as the `recall \"q\" from S;` statement.",
        params: &[("query", "string"), ("store", "string")],
        return_type: "array",
    },
    BuiltinMember {
        name: "forget",
        kind: MemberKind::Function,
        description: "Delete entries from RAG memory by content — returns how \
                      many were removed. An empty target clears the store. \
                      Implemented by VM::rag_forget, the same code path as the \
                      `forget x from S;` statement.",
        params: &[("target", "any"), ("store", "string")],
        return_type: "number",
    },
    BuiltinMember {
        name: "put",
        kind: MemberKind::Function,
        description: "Write to stdout without newline",
        params: &[("args", "...any")],
        return_type: "null",
    },
    BuiltinMember {
        name: "putf",
        kind: MemberKind::Function,
        description: "Printf-style formatted write (%%s, %%d, %%f)",
        params: &[("fmt", "string"), ("args", "...any")],
        return_type: "null",
    },
    BuiltinMember {
        name: "göster",
        kind: MemberKind::Function,
        description: "Write to stdout without newline (Turkish)",
        params: &[("args", "...any")],
        return_type: "null",
    },
    BuiltinMember {
        name: "fgöster",
        kind: MemberKind::Function,
        description: "Printf-style formatted write (Turkish)",
        params: &[("fmt", "string"), ("args", "...any")],
        return_type: "null",
    },
];
