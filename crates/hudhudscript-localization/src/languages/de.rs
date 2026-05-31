//! German keyword mappings
//! Deutsch - German language support (V2 - Verb-second)

use crate::keyword_map::Keyword;
use std::collections::HashMap;

pub fn get_keywords() -> HashMap<String, Keyword> {
    let mut map = HashMap::new();

    // Module system
    map.insert("verwenden".to_string(), Keyword::Use);
    map.insert("als".to_string(), Keyword::As);
    map.insert("importieren".to_string(), Keyword::Import);
    map.insert("exportieren".to_string(), Keyword::Export);
    map.insert("von".to_string(), Keyword::From);

    // Agent system
    map.insert("agent".to_string(), Keyword::Agent);
    map.insert("aufgabe".to_string(), Keyword::Task);
    map.insert("werkzeug".to_string(), Keyword::Tool);
    map.insert("ressource".to_string(), Keyword::Resource);
    map.insert("mcp".to_string(), Keyword::Mcp);
    map.insert("server".to_string(), Keyword::Server);
    map.insert("konfiguration".to_string(), Keyword::Config);

    // Control flow
    map.insert("wenn".to_string(), Keyword::If);
    map.insert("sonst".to_string(), Keyword::Else);
    map.insert("während".to_string(), Keyword::While);
    map.insert("für".to_string(), Keyword::For);
    map.insert("zurück".to_string(), Keyword::Return);
    map.insert("abbrechen".to_string(), Keyword::Break);
    map.insert("fortsetzen".to_string(), Keyword::Continue);
    map.insert("wechseln".to_string(), Keyword::Switch);
    map.insert("fall".to_string(), Keyword::Case);
    map.insert("standard".to_string(), Keyword::Default);

    // Error handling
    map.insert("versuchen".to_string(), Keyword::Try);
    map.insert("fangen".to_string(), Keyword::Catch);
    map.insert("schließlich".to_string(), Keyword::Finally);
    map.insert("werfen".to_string(), Keyword::Throw);

    // Async
    map.insert("asynchron".to_string(), Keyword::Async);
    map.insert("warten".to_string(), Keyword::Await);

    // Data & Variables
    map.insert("sei".to_string(), Keyword::Let);
    map.insert("daten".to_string(), Keyword::Data);
    map.insert("wert".to_string(), Keyword::Set);

    // Governance
    map.insert("verfassung".to_string(), Keyword::Constitution);
    map.insert("gesetz".to_string(), Keyword::Law);
    map.insert("regel".to_string(), Keyword::Rule);
    map.insert("rat".to_string(), Keyword::Council);
    map.insert("schwarm".to_string(), Keyword::Swarm);
    map.insert("gemeinschaft".to_string(), Keyword::Community);
    map.insert("obligatorisch".to_string(), Keyword::Mandatory);
    map.insert("beratend".to_string(), Keyword::Advisory);
    map.insert("optional".to_string(), Keyword::Optional);
    map.insert("rolle".to_string(), Keyword::Role);
    map.insert("mitglied".to_string(), Keyword::Member);
    map.insert("strategie".to_string(), Keyword::Strategy);
    map.insert("wettbewerbsfähig".to_string(), Keyword::Competitive);
    map.insert("kollaborativ".to_string(), Keyword::Collaborative);
    map.insert("parallel".to_string(), Keyword::Parallel);
    map.insert("sequenziell".to_string(), Keyword::Sequential);
    map.insert("ausführen".to_string(), Keyword::Execute);
    // Additional keywords
    map.insert("aktion".to_string(), Keyword::Action);
    map.insert("agentenregel".to_string(), Keyword::AgentRule);
    map.insert("agentenzustand".to_string(), Keyword::AgentState);
    map.insert("erlauben".to_string(), Keyword::Allow);
    map.insert("übertragen".to_string(), Keyword::Broadcast);
    map.insert("aufrufen".to_string(), Keyword::Call);
    map.insert("akkord".to_string(), Keyword::Chord);
    map.insert(
        "kommunikationsstil".to_string(),
        Keyword::CommunicationStyle,
    );
    map.insert("konstante".to_string(), Keyword::Const);
    map.insert("kultur".to_string(), Keyword::Culture);
    map.insert("datenfluss".to_string(), Keyword::DataFlow);
    map.insert("verweigern".to_string(), Keyword::Deny);
    map.insert("hängt_ab_von".to_string(), Keyword::DependsOn);
    map.insert("durchsetzung".to_string(), Keyword::Enforcement);
    map.insert("entität".to_string(), Keyword::Entity);
    map.insert("ereignis".to_string(), Keyword::Event);
    map.insert("vollstrecker".to_string(), Keyword::Executor);
    map.insert("fluss".to_string(), Keyword::Flow);
    map.insert("formell".to_string(), Keyword::Formal);
    map.insert("function".to_string(), Keyword::Function);
    map.insert("zukunft".to_string(), Keyword::Future);
    map.insert("harmonie".to_string(), Keyword::Harmony);
    map.insert("informell".to_string(), Keyword::Informal);
    map.insert("absicht".to_string(), Keyword::Intent);
    map.insert("richter".to_string(), Keyword::Judge);
    map.insert("schicht".to_string(), Keyword::Layer);
    map.insert("melodie".to_string(), Keyword::Melody);
    map.insert("zusammenführen".to_string(), Keyword::Merge);
    map.insert("modell".to_string(), Keyword::Model);
    map.insert("netzwerk".to_string(), Keyword::Network);
    map.insert("normen".to_string(), Keyword::Norms);
    map.insert("note".to_string(), Keyword::Note);
    map.insert("auf".to_string(), Keyword::On);
    map.insert("priorität".to_string(), Keyword::Priority);
    map.insert("versprechen".to_string(), Keyword::Promise);
    map.insert("staatsanwalt".to_string(), Keyword::Prosecutor);
    map.insert("anbieter".to_string(), Keyword::Provider);
    map.insert("rhythmus".to_string(), Keyword::Rhythm);
    map.insert("regelkette".to_string(), Keyword::RuleChain);
    map.insert("regelsatz".to_string(), Keyword::RuleSet);
    map.insert("tonleiter".to_string(), Keyword::Scale);
    map.insert("zustand".to_string(), Keyword::State);
    map.insert("zustandsmaschine".to_string(), Keyword::StateMachine);
    map.insert("technisch".to_string(), Keyword::Technical);
    map.insert("tempo".to_string(), Keyword::Tempo);
    map.insert("transformieren".to_string(), Keyword::Transform);
    map.insert("auslösen".to_string(), Keyword::Trigger);
    map.insert("werte".to_string(), Keyword::Values);
    map.insert("variable".to_string(), Keyword::Let);
    map.insert("wollen".to_string(), Keyword::Want);
    map.insert("wann".to_string(), Keyword::When);

    map
}
