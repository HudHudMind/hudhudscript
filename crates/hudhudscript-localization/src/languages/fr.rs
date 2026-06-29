//! French keyword mappings
//! Français - French language support (SVO)

use crate::keyword_map::Keyword;
use std::collections::HashMap;

pub fn get_keywords() -> HashMap<String, Keyword> {
    let mut map = HashMap::new();

    // Module system
    map.insert("utiliser".to_string(), Keyword::Use);
    map.insert("comme".to_string(), Keyword::As);
    map.insert("importer".to_string(), Keyword::Import);
    map.insert("exporter".to_string(), Keyword::Export);
    map.insert("depuis".to_string(), Keyword::From);

    // Agent system
    map.insert("agent".to_string(), Keyword::Agent);
    map.insert("outil".to_string(), Keyword::Tool);
    map.insert("ressource".to_string(), Keyword::Resource);
    map.insert("mcp".to_string(), Keyword::Mcp);
    map.insert("serveur".to_string(), Keyword::Server);
    map.insert("configuration".to_string(), Keyword::Config);

    // Control flow
    map.insert("si".to_string(), Keyword::If);
    map.insert("sinon".to_string(), Keyword::Else);
    map.insert("pendant".to_string(), Keyword::While);
    map.insert("pour".to_string(), Keyword::For);
    map.insert("retourner".to_string(), Keyword::Return);
    map.insert("casser".to_string(), Keyword::Break);
    map.insert("continuer".to_string(), Keyword::Continue);
    map.insert("changer".to_string(), Keyword::Switch);
    map.insert("cas".to_string(), Keyword::Case);
    map.insert("défaut".to_string(), Keyword::Default);

    // Error handling
    map.insert("essayer".to_string(), Keyword::Try);
    map.insert("attraper".to_string(), Keyword::Catch);
    map.insert("finalement".to_string(), Keyword::Finally);
    map.insert("lancer".to_string(), Keyword::Throw);

    // Async
    map.insert("asynchrone".to_string(), Keyword::Async);
    map.insert("attendre".to_string(), Keyword::Await);

    // Data & Variables
    map.insert("variable".to_string(), Keyword::Let);
    map.insert("données".to_string(), Keyword::Data);
    map.insert("valeur".to_string(), Keyword::Set);

    // Governance
    map.insert("constitution".to_string(), Keyword::Constitution);
    map.insert("loi".to_string(), Keyword::Law);
    map.insert("règle".to_string(), Keyword::Rule);
    map.insert("conseil".to_string(), Keyword::Council);
    map.insert("essaim".to_string(), Keyword::Swarm);
    map.insert("communauté".to_string(), Keyword::Community);
    map.insert("obligatoire".to_string(), Keyword::Mandatory);
    map.insert("consultatif".to_string(), Keyword::Advisory);
    map.insert("optionnel".to_string(), Keyword::Optional);
    map.insert("rôle".to_string(), Keyword::Role);
    map.insert("membre".to_string(), Keyword::Member);
    map.insert("stratégie".to_string(), Keyword::Strategy);
    map.insert("compétitif".to_string(), Keyword::Competitive);
    map.insert("collaboratif".to_string(), Keyword::Collaborative);
    map.insert("parallèle".to_string(), Keyword::Parallel);
    map.insert("séquentiel".to_string(), Keyword::Sequential);
    map.insert("exécuter".to_string(), Keyword::Execute);
    // Native French translations for governance/intent/flow keywords
    map.insert("action".to_string(), Keyword::Action);
    map.insert("état_agent".to_string(), Keyword::AgentState);
    map.insert("autoriser".to_string(), Keyword::Allow);
    map.insert("diffuser".to_string(), Keyword::Broadcast);
    map.insert("appeler".to_string(), Keyword::Call);
    map.insert("accord".to_string(), Keyword::Chord);
    map.insert(
        "style_communication".to_string(),
        Keyword::CommunicationStyle,
    );
    map.insert("constante".to_string(), Keyword::Const);
    map.insert("culture".to_string(), Keyword::Culture);
    map.insert("flux_données".to_string(), Keyword::DataFlow);
    map.insert("refuser".to_string(), Keyword::Deny);
    map.insert("dépend_de".to_string(), Keyword::DependsOn);
    map.insert("application".to_string(), Keyword::Enforcement);
    map.insert("entité".to_string(), Keyword::Entity);
    map.insert("événement".to_string(), Keyword::Event);
    map.insert("exécuteur".to_string(), Keyword::Executor);
    map.insert("flux".to_string(), Keyword::Flow);
    map.insert("formel".to_string(), Keyword::Formal);
    map.insert("fonction".to_string(), Keyword::Function);
    map.insert("futur".to_string(), Keyword::Future);
    map.insert("harmonie".to_string(), Keyword::Harmony);
    map.insert("informel".to_string(), Keyword::Informal);
    map.insert("intention".to_string(), Keyword::Intent);
    map.insert("juge".to_string(), Keyword::Judge);
    map.insert("couche".to_string(), Keyword::Layer);
    map.insert("mélodie".to_string(), Keyword::Melody);
    map.insert("fusionner".to_string(), Keyword::Merge);
    map.insert("modèle".to_string(), Keyword::Model);
    map.insert("réseau".to_string(), Keyword::Network);
    map.insert("normes".to_string(), Keyword::Norms);
    map.insert("note".to_string(), Keyword::Note);
    map.insert("sur".to_string(), Keyword::On);
    map.insert("priorité".to_string(), Keyword::Priority);
    map.insert("promesse".to_string(), Keyword::Promise);
    map.insert("procureur".to_string(), Keyword::Prosecutor);
    map.insert("fournisseur".to_string(), Keyword::Provider);
    map.insert("rythme".to_string(), Keyword::Rhythm);
    map.insert("gamme".to_string(), Keyword::Scale);
    map.insert("état".to_string(), Keyword::State);
    map.insert("machine_état".to_string(), Keyword::StateMachine);
    map.insert("technique".to_string(), Keyword::Technical);
    map.insert("tempo".to_string(), Keyword::Tempo);
    map.insert("transformer".to_string(), Keyword::Transform);
    map.insert("déclencheur".to_string(), Keyword::Trigger);
    map.insert("valeurs".to_string(), Keyword::Values);
    map.insert("variable".to_string(), Keyword::Var);
    map.insert("vouloir".to_string(), Keyword::Want);
    map.insert("quand".to_string(), Keyword::When);

    map
}
