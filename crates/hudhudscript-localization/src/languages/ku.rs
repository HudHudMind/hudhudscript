//! Kurdish keyword mappings (Kurmanji)
//! Kurdî - Kurdish language support

use crate::keyword_map::Keyword;
use std::collections::HashMap;

pub fn get_keywords() -> HashMap<String, Keyword> {
    let mut map = HashMap::new();

    // Core keywords
    map.insert("bila".to_string(), Keyword::Let);
    map.insert("guherbar".to_string(), Keyword::Let);
    map.insert("dane".to_string(), Keyword::Data);
    map.insert("saz_bike".to_string(), Keyword::Set);
    map.insert("rast".to_string(), Keyword::True);
    map.insert("ne_rast".to_string(), Keyword::False);

    // Control flow
    map.insert("eger".to_string(), Keyword::If);
    map.insert("wekî_din".to_string(), Keyword::Else);
    map.insert("dema_ku".to_string(), Keyword::While);
    map.insert("ji_bo".to_string(), Keyword::For);
    map.insert("vegere".to_string(), Keyword::Return);
    map.insert("bişkîne".to_string(), Keyword::Break);
    map.insert("bidomîne".to_string(), Keyword::Continue);
    map.insert("guherîne".to_string(), Keyword::Switch);
    map.insert("rewş".to_string(), Keyword::Case);
    map.insert("standard".to_string(), Keyword::Default);

    // Module system
    map.insert("bikar_bîne".to_string(), Keyword::Use);
    map.insert("wek".to_string(), Keyword::As);
    map.insert("têxe".to_string(), Keyword::Import);
    map.insert("derxîne".to_string(), Keyword::Export);
    map.insert("ji".to_string(), Keyword::From);

    // Agent system
    map.insert("ajans".to_string(), Keyword::Agent);
    map.insert("kar".to_string(), Keyword::Task);
    map.insert("amûr".to_string(), Keyword::Tool);
    map.insert("çavkanî".to_string(), Keyword::Resource);
    map.insert("mcp".to_string(), Keyword::Mcp);
    map.insert("server".to_string(), Keyword::Server);
    map.insert("veavakirinî".to_string(), Keyword::Config);
    map.insert("pêşkêşkar".to_string(), Keyword::Provider);
    map.insert("model".to_string(), Keyword::Model);
    map.insert("bang_bike".to_string(), Keyword::Call);

    // Governance
    map.insert("destûr".to_string(), Keyword::Constitution);
    map.insert("qanûn".to_string(), Keyword::Law);
    map.insert("rêbaz".to_string(), Keyword::Rule);
    map.insert("encûmen".to_string(), Keyword::Council);
    map.insert("kom".to_string(), Keyword::Swarm);
    map.insert("civak".to_string(), Keyword::Community);
    map.insert("bicîhbikirin".to_string(), Keyword::Enforcement);
    map.insert("mecbûrî".to_string(), Keyword::Mandatory);
    map.insert("şîretî".to_string(), Keyword::Advisory);
    map.insert("vebijarkî".to_string(), Keyword::Optional);
    map.insert("rol".to_string(), Keyword::Role);
    map.insert("endam".to_string(), Keyword::Member);
    map.insert("stratejî".to_string(), Keyword::Strategy);
    map.insert("pêşbazî".to_string(), Keyword::Competitive);
    map.insert("hevkarî".to_string(), Keyword::Collaborative);
    map.insert("çand".to_string(), Keyword::Culture);
    map.insert("nirx".to_string(), Keyword::Values);
    map.insert("norm".to_string(), Keyword::Norms);
    map.insert("dadger".to_string(), Keyword::Prosecutor);
    map.insert("hakim".to_string(), Keyword::Judge);

    // Async & Error handling
    map.insert("asenkron".to_string(), Keyword::Async);
    map.insert("li_benda".to_string(), Keyword::Await);
    map.insert("biceribîne".to_string(), Keyword::Try);
    map.insert("bigire".to_string(), Keyword::Catch);
    map.insert("dawî".to_string(), Keyword::Finally);
    map.insert("biavêje".to_string(), Keyword::Throw);
    map.insert("soz".to_string(), Keyword::Promise);
    map.insert("pêşerojî".to_string(), Keyword::Future);

    // Data structures
    map.insert("not".to_string(), Keyword::Note);
    map.insert("akord".to_string(), Keyword::Chord);
    map.insert("melodî".to_string(), Keyword::Melody);
    map.insert("harmonî".to_string(), Keyword::Harmony);
    map.insert("rîtm".to_string(), Keyword::Rhythm);
    map.insert("tempo".to_string(), Keyword::Tempo);
    map.insert("pîvan".to_string(), Keyword::Scale);

    // Intent & State
    map.insert("entîte".to_string(), Keyword::Entity);
    map.insert("rewşa".to_string(), Keyword::State);
    map.insert("bûyer".to_string(), Keyword::Event);

    // Flow & Orchestration
    map.insert("herikîn".to_string(), Keyword::Flow);
    map.insert("qat".to_string(), Keyword::Layer);
    map.insert("tor".to_string(), Keyword::Network);
    map.insert("girêdayî".to_string(), Keyword::DependsOn);
    map.insert("belav_bike".to_string(), Keyword::Broadcast);
    map.insert("yek_bike".to_string(), Keyword::Merge);
    map.insert("paralel".to_string(), Keyword::Parallel);
    map.insert("rêzik".to_string(), Keyword::Sequential);
    map.insert("bicîh_bîne".to_string(), Keyword::Execute);
    map.insert("dema".to_string(), Keyword::When);
    map.insert("li_ser".to_string(), Keyword::On);
    map.insert("vekirin".to_string(), Keyword::Trigger);
    // Native Kurdish (Kurmanji) translations
    map.insert("çalakî".to_string(), Keyword::Action);
    map.insert("rêbaza_ajansê".to_string(), Keyword::AgentRule);
    map.insert("rewşa_ajansê".to_string(), Keyword::AgentState);
    map.insert("destûr_bide".to_string(), Keyword::Allow);
    map.insert(
        "şêweya_ragihandinê".to_string(),
        Keyword::CommunicationStyle,
    );
    map.insert("sabit".to_string(), Keyword::Const);
    map.insert("herikîna_daneyê".to_string(), Keyword::DataFlow);
    map.insert("qedexe_bike".to_string(), Keyword::Deny);
    map.insert("bicîhker".to_string(), Keyword::Executor);
    map.insert("fermî".to_string(), Keyword::Formal);
    map.insert("fonksiyon".to_string(), Keyword::Function);
    map.insert("ne_fermî".to_string(), Keyword::Informal);
    map.insert("mebestî".to_string(), Keyword::Intent);
    map.insert("pêşînî".to_string(), Keyword::Priority);
    map.insert("zincîra_rêbazan".to_string(), Keyword::RuleChain);
    map.insert("komela_rêbazan".to_string(), Keyword::RuleSet);
    map.insert("makîneya_rewşan".to_string(), Keyword::StateMachine);
    map.insert("teknîk".to_string(), Keyword::Technical);
    map.insert("veguherîne".to_string(), Keyword::Transform);
    map.insert("guherbar".to_string(), Keyword::Var);
    map.insert("xwestin".to_string(), Keyword::Want);

    map
}
