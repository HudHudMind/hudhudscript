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

        // Subject-Oriented Programming
        "compose" => (
            "compose <Subject> with <OtherSubject>",
            "Composes two subjects together.",
        ),
        "can" => ("can <capability>", "Declares a subject capability."),
        "has" => ("has <role>", "Declares a subject role."),
        "intends" => ("intends <goal>", "Declares a subject intent."),
        "uses" => ("uses <provider>", "Declares provider usage."),
        "via" => ("via <transport>", "Declares transport medium."),
        "relation" => ("relation <name> { ... }", "Declares a relation."),
        "effect" => ("effect <name> { ... }", "Declares an effect."),
        "memory" => ("memory", "Subject memory field."),
        "perception" => ("perception", "Subject perception field."),
        "context" => ("context", "Subject context field."),

        // Governance
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
        "law" => ("law <name> { ... }", "Declares a governance law."),
        "rule" => ("rule <name> { ... }", "Declares a governance rule."),
        "council" => ("council <name>: { ... }", "Declares a governance council."),
        "swarm" => ("swarm <name> { ... }", "Declares a swarm."),
        "community" => (
            "community <name> { ... }",
            "Declares a governance community.",
        ),
        "contract" => ("contract <name> { ... }", "Declares a governance contract."),
        "treaty" => ("treaty <name> { ... }", "Declares a governance treaty."),
        "enforcement" => (
            "enforcement <name> { ... }",
            "Declares a governance enforcement mechanism.",
        ),
        "mandatory" => ("mandatory", "Mandatory governance level."),
        "advisory" => ("advisory", "Advisory governance level."),
        "optional" => ("optional", "Optional governance level."),

        // Governance models
        "democracy" => ("democracy", "Democratic governance model."),
        "monarchy" => ("monarchy", "Monarchic governance model."),
        "technocracy" => ("technocracy", "Technocratic governance model."),
        "theocracy" => ("theocracy", "Theocratic governance model."),
        "parliamentary" => ("parliamentary", "Parliamentary governance model."),
        "meritocracy" => ("meritocracy", "Meritocratic governance model."),
        "anarchy" => ("anarchy", "Anarchic governance model."),
        "oligarchy" => ("oligarchy", "Oligarchic governance model."),
        "consensus" => ("consensus", "Consensus-based governance model."),
        "autocracy" => ("autocracy", "Autocratic governance model."),

        // Roles & Strategy
        "member" => ("member", "Governance member."),
        "prosecutor" => ("prosecutor", "Prosecutor role."),
        "judge" => ("judge", "Judge role."),
        "executor" => ("executor", "Executor role."),
        "competitive" => ("competitive", "Competitive strategy."),
        "collaborative" => ("collaborative", "Collaborative strategy."),

        // Voting / execution types
        "majority" => ("majority", "Majority voting rule."),
        "unanimous" => ("unanimous", "Unanimous voting rule."),
        "weighted" => ("weighted", "Weighted voting rule."),
        "firstWins" => ("firstWins", "First-wins voting rule."),
        "roundRobin" => ("roundRobin", "Round-robin execution strategy."),

        // Strategy session hooks
        "onStart" => ("onStart { ... }", "Hook triggered when strategy starts."),
        "onMemberStart" => (
            "onMemberStart { ... }",
            "Hook triggered when a member starts.",
        ),
        "onMemberComplete" => (
            "onMemberComplete { ... }",
            "Hook triggered when a member completes.",
        ),
        "onVote" => ("onVote { ... }", "Hook triggered on vote."),
        "onComplete" => (
            "onComplete { ... }",
            "Hook triggered when strategy completes.",
        ),
        "onError" => ("onError { ... }", "Hook triggered on strategy error."),

        // Culture
        "culture" => ("culture <name> { ... }", "Declares a culture profile."),
        "values" => ("values", "Defines cultural values."),
        "norms" => ("norms", "Defines cultural norms."),
        "communication_style" => (
            "communication_style: formal | informal | technical",
            "Defines communication style.",
        ),
        "formal" => ("formal", "Formal communication style."),
        "informal" => ("informal", "Informal communication style."),
        "technical" => ("technical", "Technical communication style."),

        // Loop engineering
        "loop" => ("loop <name> { ... }", "Declares a loop-engineering block."),
        "step" => ("step <name> { ... }", "Declares a loop-engineering step."),
        "gate" => ("gate <name> { ... }", "Declares a loop-engineering gate."),
        "chain" => (
            "chain <name> { attach <step>; ... }",
            "Declares a loop-engineering chain of steps.",
        ),
        "attach" => ("attach <stepName>", "Attaches a step to the current chain."),

        // Flow & orchestration
        "flow" => ("flow <name> { ... }", "Declares a flow."),
        "dataflow" => ("dataflow <name> { ... }", "Declares a dataflow."),
        "layer" => ("layer <name> { ... }", "Declares a layer."),
        "network" => ("network <name> { ... }", "Declares a network."),
        "depends_on" => ("depends_on <node>", "Declares a dependency between nodes."),
        "broadcast" => ("broadcast <message>", "Broadcasts a message."),
        "merge" => ("merge <a>, <b>", "Merges two flows."),
        "parallel" => ("parallel", "Parallel execution mode."),
        "sequential" => ("sequential", "Sequential execution mode."),
        "execute" => ("execute <action>", "Executes an action."),

        // RAG
        "store" => ("store <name> { ... }", "Declares a RAG vector store."),
        "remember" => (
            "remember <content> [in <store>]",
            "Stores content in a RAG vector store.",
        ),
        "recall" => (
            "recall <query> [from <store>]",
            "Retrieves content from a RAG vector store.",
        ),
        "forget" => (
            "forget <content> [in <store>]",
            "Forgets content from a RAG vector store.",
        ),
        "embed" => ("embed <content>", "Creates an embedding."),

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
