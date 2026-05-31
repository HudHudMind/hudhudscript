//! English keyword mappings
//! English - Default language (SVO word order)

use crate::keyword_map::Keyword;
use std::collections::HashMap;

pub fn get_keywords() -> HashMap<String, Keyword> {
    let mut map = HashMap::new();

    // Module system
    map.insert("use".to_string(), Keyword::Use);
    map.insert("as".to_string(), Keyword::As);
    map.insert("import".to_string(), Keyword::Import);
    map.insert("export".to_string(), Keyword::Export);
    map.insert("from".to_string(), Keyword::From);

    // Agent system
    map.insert("agent".to_string(), Keyword::Agent);
    map.insert("task".to_string(), Keyword::Task);
    map.insert("tool".to_string(), Keyword::Tool);
    map.insert("resource".to_string(), Keyword::Resource);
    map.insert("mcp".to_string(), Keyword::Mcp);
    map.insert("server".to_string(), Keyword::Server);
    map.insert("config".to_string(), Keyword::Config);

    // Control flow
    map.insert("if".to_string(), Keyword::If);
    map.insert("then".to_string(), Keyword::Then);
    map.insert("else".to_string(), Keyword::Else);
    map.insert("while".to_string(), Keyword::While);
    map.insert("for".to_string(), Keyword::For);
    map.insert("foreach".to_string(), Keyword::ForEach);
    map.insert("in".to_string(), Keyword::In);
    map.insert("return".to_string(), Keyword::Return);
    map.insert("break".to_string(), Keyword::Break);
    map.insert("continue".to_string(), Keyword::Continue);
    map.insert("switch".to_string(), Keyword::Switch);
    map.insert("case".to_string(), Keyword::Case);
    map.insert("default".to_string(), Keyword::Default);
    map.insert("function".to_string(), Keyword::Function);

    // Logical operators
    map.insert("and".to_string(), Keyword::And);
    map.insert("or".to_string(), Keyword::Or);

    // Error handling
    map.insert("try".to_string(), Keyword::Try);
    map.insert("catch".to_string(), Keyword::Catch);
    map.insert("finally".to_string(), Keyword::Finally);
    map.insert("throw".to_string(), Keyword::Throw);

    // Async
    map.insert("async".to_string(), Keyword::Async);
    map.insert("await".to_string(), Keyword::Await);

    // Data & Variables
    map.insert("let".to_string(), Keyword::Let);
    map.insert("var".to_string(), Keyword::Var);
    map.insert("const".to_string(), Keyword::Const);
    map.insert("data".to_string(), Keyword::Data);
    map.insert("set".to_string(), Keyword::Set);

    // OOP Keywords
    map.insert("class".to_string(), Keyword::Class);
    map.insert("constructor".to_string(), Keyword::Constructor);
    map.insert("this".to_string(), Keyword::This);
    map.insert("self".to_string(), Keyword::SelfKeyword);
    map.insert("super".to_string(), Keyword::Super);
    map.insert("new".to_string(), Keyword::New);
    map.insert("extends".to_string(), Keyword::Extends);
    map.insert("public".to_string(), Keyword::Public);
    map.insert("private".to_string(), Keyword::Private);
    map.insert("protected".to_string(), Keyword::Protected);
    map.insert("static".to_string(), Keyword::Static);

    // Intent-Based Constructs
    map.insert("entity".to_string(), Keyword::Entity);
    map.insert("state".to_string(), Keyword::State);
    map.insert("statemachine".to_string(), Keyword::StateMachine);
    map.insert("agentstate".to_string(), Keyword::AgentState);
    map.insert("event".to_string(), Keyword::Event);
    map.insert("action".to_string(), Keyword::Action);
    map.insert("transform".to_string(), Keyword::Transform);

    // Governance
    map.insert("constitution".to_string(), Keyword::Constitution);
    map.insert("law".to_string(), Keyword::Law);
    map.insert("rule".to_string(), Keyword::Rule);
    map.insert("agentrule".to_string(), Keyword::AgentRule);
    map.insert("ruleset".to_string(), Keyword::RuleSet);
    map.insert("rulechain".to_string(), Keyword::RuleChain);
    map.insert("council".to_string(), Keyword::Council);
    map.insert("swarm".to_string(), Keyword::Swarm);
    map.insert("community".to_string(), Keyword::Community);
    map.insert("enforcement".to_string(), Keyword::Enforcement);
    map.insert("mandatory".to_string(), Keyword::Mandatory);
    map.insert("advisory".to_string(), Keyword::Advisory);
    map.insert("optional".to_string(), Keyword::Optional);
    map.insert("role".to_string(), Keyword::Role);
    map.insert("member".to_string(), Keyword::Member);
    map.insert("strategy".to_string(), Keyword::Strategy);
    map.insert("competitive".to_string(), Keyword::Competitive);
    map.insert("collaborative".to_string(), Keyword::Collaborative);
    map.insert("culture".to_string(), Keyword::Culture);
    map.insert("values".to_string(), Keyword::Values);
    map.insert("norms".to_string(), Keyword::Norms);
    map.insert(
        "communication_style".to_string(),
        Keyword::CommunicationStyle,
    );
    map.insert("formal".to_string(), Keyword::Formal);
    map.insert("informal".to_string(), Keyword::Informal);
    map.insert("technical".to_string(), Keyword::Technical);

    // Intent & Goals
    map.insert("intent".to_string(), Keyword::Intent);
    map.insert("want".to_string(), Keyword::Want);
    map.insert("priority".to_string(), Keyword::Priority);

    // Flow
    map.insert("flow".to_string(), Keyword::Flow);
    map.insert("dataflow".to_string(), Keyword::DataFlow);

    // Agent Roles
    map.insert("prosecutor".to_string(), Keyword::Prosecutor);
    map.insert("judge".to_string(), Keyword::Judge);
    map.insert("executor".to_string(), Keyword::Executor);

    // Music Data Structures
    map.insert("note".to_string(), Keyword::Note);
    map.insert("chord".to_string(), Keyword::Chord);
    map.insert("melody".to_string(), Keyword::Melody);
    map.insert("harmony".to_string(), Keyword::Harmony);
    map.insert("rhythm".to_string(), Keyword::Rhythm);
    map.insert("tempo".to_string(), Keyword::Tempo);
    map.insert("scale".to_string(), Keyword::Scale);

    // Constraints
    map.insert("when".to_string(), Keyword::When);
    map.insert("on".to_string(), Keyword::On);
    map.insert("trigger".to_string(), Keyword::Trigger);

    // Permissions
    map.insert("allow".to_string(), Keyword::Allow);
    map.insert("deny".to_string(), Keyword::Deny);

    // Layer-Based Orchestration
    map.insert("layer".to_string(), Keyword::Layer);
    map.insert("network".to_string(), Keyword::Network);
    map.insert("depends_on".to_string(), Keyword::DependsOn);
    map.insert("broadcast".to_string(), Keyword::Broadcast);
    map.insert("merge".to_string(), Keyword::Merge);
    map.insert("parallel".to_string(), Keyword::Parallel);
    map.insert("sequential".to_string(), Keyword::Sequential);
    map.insert("execute".to_string(), Keyword::Execute);
    map.insert(
        "communication_style".to_string(),
        Keyword::CommunicationStyle,
    );

    // Additional Keywords
    map.insert("provider".to_string(), Keyword::Provider);
    map.insert("model".to_string(), Keyword::Model);
    map.insert("call".to_string(), Keyword::Call);
    map.insert("true".to_string(), Keyword::True);
    map.insert("false".to_string(), Keyword::False);
    map.insert("promise".to_string(), Keyword::Promise);
    map.insert("future".to_string(), Keyword::Future);

    map
}
