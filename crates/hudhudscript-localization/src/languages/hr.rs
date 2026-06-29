//! Croatian keyword mappings (Latin)
//! Hrvatski - Croatian language support

use crate::keyword_map::Keyword;
use std::collections::HashMap;

pub fn get_keywords() -> HashMap<String, Keyword> {
    let mut map = HashMap::new();

    // Core keywords
    map.insert("neka".to_string(), Keyword::Let);
    map.insert("varijabla".to_string(), Keyword::Let);
    map.insert("podaci".to_string(), Keyword::Data);
    map.insert("postavi".to_string(), Keyword::Set);
    map.insert("točno".to_string(), Keyword::True);
    map.insert("netočno".to_string(), Keyword::False);

    // Control flow
    map.insert("ako".to_string(), Keyword::If);
    map.insert("inače".to_string(), Keyword::Else);
    map.insert("dok".to_string(), Keyword::While);
    map.insert("za".to_string(), Keyword::For);
    map.insert("vrati".to_string(), Keyword::Return);
    map.insert("prekini".to_string(), Keyword::Break);
    map.insert("nastavi".to_string(), Keyword::Continue);
    map.insert("prebaci".to_string(), Keyword::Switch);
    map.insert("slučaj".to_string(), Keyword::Case);
    map.insert("zadano".to_string(), Keyword::Default);

    // Module system
    map.insert("koristi".to_string(), Keyword::Use);
    map.insert("kao".to_string(), Keyword::As);
    map.insert("uvezi".to_string(), Keyword::Import);
    map.insert("izvezi".to_string(), Keyword::Export);
    map.insert("od".to_string(), Keyword::From);

    // Agent system
    map.insert("agent".to_string(), Keyword::Agent);
    map.insert("alat".to_string(), Keyword::Tool);
    map.insert("resurs".to_string(), Keyword::Resource);
    map.insert("mcp".to_string(), Keyword::Mcp);
    map.insert("server".to_string(), Keyword::Server);
    map.insert("konfiguracija".to_string(), Keyword::Config);
    map.insert("pružatelj".to_string(), Keyword::Provider);
    map.insert("model".to_string(), Keyword::Model);
    map.insert("pozovi".to_string(), Keyword::Call);

    // Governance
    map.insert("ustav".to_string(), Keyword::Constitution);
    map.insert("zakon".to_string(), Keyword::Law);
    map.insert("pravilo".to_string(), Keyword::Rule);
    map.insert("vijeće".to_string(), Keyword::Council);
    map.insert("roj".to_string(), Keyword::Swarm);
    map.insert("zajednica".to_string(), Keyword::Community);
    map.insert("provedba".to_string(), Keyword::Enforcement);
    map.insert("obavezno".to_string(), Keyword::Mandatory);
    map.insert("savjetodavno".to_string(), Keyword::Advisory);
    map.insert("opcionalno".to_string(), Keyword::Optional);
    map.insert("uloga".to_string(), Keyword::Role);
    map.insert("član".to_string(), Keyword::Member);
    map.insert("strategija".to_string(), Keyword::Strategy);
    map.insert("konkurentan".to_string(), Keyword::Competitive);
    map.insert("suradnički".to_string(), Keyword::Collaborative);
    map.insert("kultura".to_string(), Keyword::Culture);
    map.insert("vrijednosti".to_string(), Keyword::Values);
    map.insert("norme".to_string(), Keyword::Norms);
    map.insert("tužitelj".to_string(), Keyword::Prosecutor);
    map.insert("sudac".to_string(), Keyword::Judge);

    // Async & Error handling
    map.insert("asinkroni".to_string(), Keyword::Async);
    map.insert("čekaj".to_string(), Keyword::Await);
    map.insert("pokušaj".to_string(), Keyword::Try);
    map.insert("uhvati".to_string(), Keyword::Catch);
    map.insert("konačno".to_string(), Keyword::Finally);
    map.insert("baci".to_string(), Keyword::Throw);
    map.insert("obećanje".to_string(), Keyword::Promise);
    map.insert("budućnost".to_string(), Keyword::Future);

    // Data structures
    map.insert("nota".to_string(), Keyword::Note);
    map.insert("akord".to_string(), Keyword::Chord);
    map.insert("melodija".to_string(), Keyword::Melody);
    map.insert("harmonija".to_string(), Keyword::Harmony);
    map.insert("ritam".to_string(), Keyword::Rhythm);
    map.insert("tempo".to_string(), Keyword::Tempo);
    map.insert("skala".to_string(), Keyword::Scale);

    // Intent & State
    map.insert("entitet".to_string(), Keyword::Entity);
    map.insert("stanje".to_string(), Keyword::State);
    map.insert("događaj".to_string(), Keyword::Event);

    // Flow & Orchestration
    map.insert("tok".to_string(), Keyword::Flow);
    map.insert("sloj".to_string(), Keyword::Layer);
    map.insert("mreža".to_string(), Keyword::Network);
    map.insert("ovisi_o".to_string(), Keyword::DependsOn);
    map.insert("emitiraj".to_string(), Keyword::Broadcast);
    map.insert("spoji".to_string(), Keyword::Merge);
    map.insert("paralelno".to_string(), Keyword::Parallel);
    map.insert("sekvencijalno".to_string(), Keyword::Sequential);
    map.insert("izvrši".to_string(), Keyword::Execute);
    map.insert("kada".to_string(), Keyword::When);
    map.insert("na".to_string(), Keyword::On);
    map.insert("okidač".to_string(), Keyword::Trigger);
    // Native Croatian translations
    map.insert("akcija".to_string(), Keyword::Action);
    map.insert("stanje_agenta".to_string(), Keyword::AgentState);
    map.insert("dopusti".to_string(), Keyword::Allow);
    map.insert("stil_komunikacije".to_string(), Keyword::CommunicationStyle);
    map.insert("konstanta".to_string(), Keyword::Const);
    map.insert("tok_podataka".to_string(), Keyword::DataFlow);
    map.insert("zabrani".to_string(), Keyword::Deny);
    map.insert("izvršitelj".to_string(), Keyword::Executor);
    map.insert("formalno".to_string(), Keyword::Formal);
    map.insert("funkcija".to_string(), Keyword::Function);
    map.insert("neformalno".to_string(), Keyword::Informal);
    map.insert("namjera".to_string(), Keyword::Intent);
    map.insert("prioritet".to_string(), Keyword::Priority);
    map.insert("stroj_stanja".to_string(), Keyword::StateMachine);
    map.insert("tehničko".to_string(), Keyword::Technical);
    map.insert("transformirati".to_string(), Keyword::Transform);
    map.insert("varijabla".to_string(), Keyword::Var);
    map.insert("željeti".to_string(), Keyword::Want);

    map
}
