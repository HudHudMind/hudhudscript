//! Italian keyword mappings
//! Italiano - Italian language support (SVO)

use crate::keyword_map::Keyword;
use std::collections::HashMap;

pub fn get_keywords() -> HashMap<String, Keyword> {
    let mut map = HashMap::new();

    // Module system
    map.insert("usare".to_string(), Keyword::Use);
    map.insert("come".to_string(), Keyword::As);
    map.insert("importare".to_string(), Keyword::Import);
    map.insert("esportare".to_string(), Keyword::Export);
    map.insert("da".to_string(), Keyword::From);

    // Agent system
    map.insert("agente".to_string(), Keyword::Agent);
    map.insert("strumento".to_string(), Keyword::Tool);
    map.insert("risorsa".to_string(), Keyword::Resource);
    map.insert("mcp".to_string(), Keyword::Mcp);
    map.insert("server".to_string(), Keyword::Server);
    map.insert("configurazione".to_string(), Keyword::Config);

    // Control flow
    map.insert("se".to_string(), Keyword::If);
    map.insert("altrimenti".to_string(), Keyword::Else);
    map.insert("mentre".to_string(), Keyword::While);
    map.insert("per".to_string(), Keyword::For);
    map.insert("ritornare".to_string(), Keyword::Return);
    map.insert("rompere".to_string(), Keyword::Break);
    map.insert("continuare".to_string(), Keyword::Continue);
    map.insert("cambiare".to_string(), Keyword::Switch);
    map.insert("caso".to_string(), Keyword::Case);
    map.insert("predefinito".to_string(), Keyword::Default);

    // Error handling
    map.insert("provare".to_string(), Keyword::Try);
    map.insert("catturare".to_string(), Keyword::Catch);
    map.insert("finalmente".to_string(), Keyword::Finally);
    map.insert("lanciare".to_string(), Keyword::Throw);

    // Async
    map.insert("asincrono".to_string(), Keyword::Async);
    map.insert("attendere".to_string(), Keyword::Await);

    // Data & Variables
    map.insert("variabile".to_string(), Keyword::Let);
    map.insert("dati".to_string(), Keyword::Data);
    map.insert("valore".to_string(), Keyword::Set);

    // Governance
    map.insert("costituzione".to_string(), Keyword::Constitution);
    map.insert("legge".to_string(), Keyword::Law);
    map.insert("regola".to_string(), Keyword::Rule);
    map.insert("consiglio".to_string(), Keyword::Council);
    map.insert("sciame".to_string(), Keyword::Swarm);
    map.insert("comunità".to_string(), Keyword::Community);
    map.insert("obbligatorio".to_string(), Keyword::Mandatory);
    map.insert("consultivo".to_string(), Keyword::Advisory);
    map.insert("opzionale".to_string(), Keyword::Optional);
    map.insert("ruolo".to_string(), Keyword::Role);
    map.insert("membro".to_string(), Keyword::Member);
    map.insert("strategia".to_string(), Keyword::Strategy);
    map.insert("competitivo".to_string(), Keyword::Competitive);
    map.insert("collaborativo".to_string(), Keyword::Collaborative);
    map.insert("parallelo".to_string(), Keyword::Parallel);
    map.insert("sequenziale".to_string(), Keyword::Sequential);
    map.insert("eseguire".to_string(), Keyword::Execute);
    // Native Italian translations for governance/intent/flow keywords
    map.insert("azione".to_string(), Keyword::Action);
    map.insert("stato_agente".to_string(), Keyword::AgentState);
    map.insert("consentire".to_string(), Keyword::Allow);
    map.insert("trasmettere".to_string(), Keyword::Broadcast);
    map.insert("chiamare".to_string(), Keyword::Call);
    map.insert("accordo".to_string(), Keyword::Chord);
    map.insert(
        "stile_comunicazione".to_string(),
        Keyword::CommunicationStyle,
    );
    map.insert("costante".to_string(), Keyword::Const);
    map.insert("cultura".to_string(), Keyword::Culture);
    map.insert("flusso_dati".to_string(), Keyword::DataFlow);
    map.insert("negare".to_string(), Keyword::Deny);
    map.insert("dipende_da".to_string(), Keyword::DependsOn);
    map.insert("applicazione".to_string(), Keyword::Enforcement);
    map.insert("entità".to_string(), Keyword::Entity);
    map.insert("evento".to_string(), Keyword::Event);
    map.insert("esecutore".to_string(), Keyword::Executor);
    map.insert("flusso".to_string(), Keyword::Flow);
    map.insert("formale".to_string(), Keyword::Formal);
    map.insert("funzione".to_string(), Keyword::Function);
    map.insert("futuro".to_string(), Keyword::Future);
    map.insert("armonia".to_string(), Keyword::Harmony);
    map.insert("informale".to_string(), Keyword::Informal);
    map.insert("intenzione".to_string(), Keyword::Intent);
    map.insert("giudice".to_string(), Keyword::Judge);
    map.insert("strato".to_string(), Keyword::Layer);
    map.insert("melodia".to_string(), Keyword::Melody);
    map.insert("unire".to_string(), Keyword::Merge);
    map.insert("modello".to_string(), Keyword::Model);
    map.insert("rete".to_string(), Keyword::Network);
    map.insert("norme".to_string(), Keyword::Norms);
    map.insert("nota".to_string(), Keyword::Note);
    map.insert("su".to_string(), Keyword::On);
    map.insert("priorità".to_string(), Keyword::Priority);
    map.insert("promessa".to_string(), Keyword::Promise);
    map.insert("procuratore".to_string(), Keyword::Prosecutor);
    map.insert("fornitore".to_string(), Keyword::Provider);
    map.insert("ritmo".to_string(), Keyword::Rhythm);
    map.insert("scala".to_string(), Keyword::Scale);
    map.insert("stato".to_string(), Keyword::State);
    map.insert("macchina_stati".to_string(), Keyword::StateMachine);
    map.insert("tecnico".to_string(), Keyword::Technical);
    map.insert("tempo".to_string(), Keyword::Tempo);
    map.insert("trasformare".to_string(), Keyword::Transform);
    map.insert("innesco".to_string(), Keyword::Trigger);
    map.insert("valori".to_string(), Keyword::Values);
    map.insert("variabile".to_string(), Keyword::Var);
    map.insert("volere".to_string(), Keyword::Want);
    map.insert("quando".to_string(), Keyword::When);

    map
}
