//! Built-in keyword hover information.

use crate::hover::HoverInfo;

/// Provide hover info for built-in keywords and well-known identifiers.
pub fn builtin_info(word: &str) -> Option<HoverInfo> {
    let (sig, doc) = match word {
        // Keywords
        "agent" => (
            "agent <name>: { ... }",
            "Declares an AI agent with model, instructions, and tools.",
        ),
        "task" => (
            "task <name>: { ... }",
            "Declares a task (deprecated, use `action` instead).",
        ),
        "action" => (
            "action <name>: { ... }",
            "Declares an action that agents can perform.",
        ),
        "subject" => (
            "subject <name> [has <role>] { ... }",
            "Declares a Subject-Oriented subject with state, capabilities, and intents.",
        ),
        "role" => (
            "role <name> { ... }",
            "Declares a role that subjects can assume.",
        ),
        "tool" => ("tool <name>: { ... }", "Declares an MCP tool binding."),
        "resource" => (
            "resource <name>: { ... }",
            "Declares an MCP resource binding.",
        ),
        "let" => ("let <name> = <value>", "Declares a mutable variable."),
        "const" => ("const <name> = <value>", "Declares an immutable constant."),
        "function" => (
            "function <name>(<params>) { ... }",
            "Declares a named function.",
        ),
        "class" => (
            "class <name> [<- <parent>] { ... }",
            "Declares a class with optional inheritance.",
        ),
        "enum" => (
            "enum <name> { Variant1, Variant2(fields) }",
            "Declares an algebraic data type.",
        ),
        "match" => (
            "match <expr> { Pattern => { ... } }",
            "Pattern matching statement.",
        ),
        "if" => (
            "if (<condition>) { ... } [else { ... }]",
            "Conditional branching.",
        ),
        "while" => (
            "while (<condition>) { ... }",
            "Loop while condition is true.",
        ),
        "for" => (
            "for (<var> in <iterable>) { ... }",
            "Iterate over a collection.",
        ),
        "return" => ("return [<value>]", "Return a value from a function."),
        "import" => (
            "import { <names> } from \"<module>\"",
            "ES-module style import.",
        ),
        "export" => (
            "export <declaration>",
            "Export a declaration from this module.",
        ),
        "async" => (
            "async function|task ...",
            "Marks a function or task as asynchronous.",
        ),
        "await" => ("await <promise>", "Waits for a promise to resolve."),
        "spawn" => ("spawn <Subject>(<args>)", "Spawns a subject instance."),
        "send" => (
            "send <message> to <target>",
            "Sends a message to a subject.",
        ),
        "receive" => (
            "receive <var> from <source>",
            "Receives a message from a subject.",
        ),
        "constitution" => (
            "constitution <name>: { ... }",
            "Declares a governance constitution.",
        ),
        "governance" => (
            "governance <name>: <type> { ... }",
            "Declares a governance model.",
        ),
        "protocol" => (
            "protocol <name>: { ... }",
            "Declares an execution protocol.",
        ),
        "strategy" => (
            "strategy <name>: { ... }",
            "Declares an execution strategy.",
        ),
        "store" => ("store <name> { ... }", "Declares a RAG vector store."),
        "remember" => (
            "remember <content> [in <store>]",
            "Stores content in a RAG vector store.",
        ),
        "recall" => (
            "recall <query> [from <store>]",
            "Retrieves content from a RAG vector store.",
        ),

        // Built-in globals
        "print" | "println" => ("print(...args)", "Prints values to standard output."),
        "Math" => (
            "Math",
            "Built-in math module with PI, E, abs, floor, ceil, sqrt, pow, min, max, random.",
        ),
        "true" => ("true: Boolean", "Boolean literal true."),
        "false" => ("false: Boolean", "Boolean literal false."),
        "null" => (
            "null: Null",
            "Null literal representing the absence of a value.",
        ),

        _ => return None,
    };

    Some(HoverInfo {
        signature: sig.to_string(),
        docs: Some(doc.to_string()),
    })
}
