//! P4c: BuiltinMethod enum + SymId lookup table.
//! Converts hot-path string method comparisons to integer enum matching.

use hudhudscript_bytecode::SymId;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Every method name dispatched via string matching in `call_method_on_value`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum BuiltinMethod {
    AddAgent,
    Append,
    Build,
    Call,
    Catch,
    Channel,
    Daemon,
    Decide,
    Exec,
    Execute,
    Fs,
    Keys,
    Length,
    Log,
    New,
    Next,
    Os,
    Regex,
    RemoveAgent,
    Run,
    Schedule,
    Stdin,
    Stream,
    Tcp,
    Then,
    Tokenomics,
    Udp,
    Unix,
    Uuid,
    Values,
    Vote,
    Ws,
}

static METHOD_TABLE: OnceLock<HashMap<SymId, BuiltinMethod>> = OnceLock::new();

pub(crate) fn lookup_method(sym: SymId) -> Option<BuiltinMethod> {
    let table = METHOD_TABLE.get_or_init(|| {
        let mut m = HashMap::default();
        macro_rules! insert {
            ($name:expr, $variant:ident) => {
                m.insert(
                    SymId(hudhudscript_bytecode::interner::intern($name).0),
                    BuiltinMethod::$variant,
                );
            };
        }
        insert!("add_agent", AddAgent);
        insert!("append", Append);
        insert!("build", Build);
        insert!("call", Call);
        insert!("catch", Catch);
        insert!("channel", Channel);
        insert!("daemon", Daemon);
        insert!("decide", Decide);
        insert!("exec", Exec);
        insert!("execute", Execute);
        insert!("fs", Fs);
        insert!("keys", Keys);
        insert!("length", Length);
        insert!("log", Log);
        insert!("new", New);
        insert!("next", Next);
        insert!("os", Os);
        insert!("regex", Regex);
        insert!("remove_agent", RemoveAgent);
        insert!("run", Run);
        insert!("schedule", Schedule);
        insert!("stdin", Stdin);
        insert!("stream", Stream);
        insert!("tcp", Tcp);
        insert!("then", Then);
        insert!("tokenomics", Tokenomics);
        insert!("udp", Udp);
        insert!("unix", Unix);
        insert!("uuid", Uuid);
        insert!("values", Values);
        insert!("vote", Vote);
        insert!("ws", Ws);
        m
    });
    table.get(&sym).copied()
}
