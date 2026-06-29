//! Spanish keyword mappings
//! Español - Spanish language support (SVO)

use crate::keyword_map::Keyword;
use std::collections::HashMap;

pub fn get_keywords() -> HashMap<String, Keyword> {
    let mut map = HashMap::new();

    // Module system
    map.insert("usar".to_string(), Keyword::Use);
    map.insert("como".to_string(), Keyword::As);
    map.insert("importar".to_string(), Keyword::Import);
    map.insert("exportar".to_string(), Keyword::Export);
    map.insert("desde".to_string(), Keyword::From);

    // Agent system
    map.insert("agente".to_string(), Keyword::Agent);
    map.insert("herramienta".to_string(), Keyword::Tool);
    map.insert("recurso".to_string(), Keyword::Resource);
    map.insert("mcp".to_string(), Keyword::Mcp);
    map.insert("servidor".to_string(), Keyword::Server);
    map.insert("configuración".to_string(), Keyword::Config);
    map.insert("proveedor".to_string(), Keyword::Provider);
    map.insert("modelo".to_string(), Keyword::Model);
    map.insert("llamar".to_string(), Keyword::Call);

    // Control flow
    map.insert("si".to_string(), Keyword::If);
    map.insert("sino".to_string(), Keyword::Else);
    map.insert("mientras".to_string(), Keyword::While);
    map.insert("para".to_string(), Keyword::For);
    map.insert("retornar".to_string(), Keyword::Return);
    map.insert("romper".to_string(), Keyword::Break);
    map.insert("continuar".to_string(), Keyword::Continue);
    map.insert("cambiar".to_string(), Keyword::Switch);
    map.insert("caso".to_string(), Keyword::Case);
    map.insert("predeterminado".to_string(), Keyword::Default);

    // Functions
    map.insert("función".to_string(), Keyword::Function);
    map.insert("fn".to_string(), Keyword::Function);

    // Async
    map.insert("asíncrono".to_string(), Keyword::Async);
    map.insert("esperar".to_string(), Keyword::Await);
    map.insert("promesa".to_string(), Keyword::Promise);
    map.insert("futuro".to_string(), Keyword::Future);

    // Error handling
    map.insert("intentar".to_string(), Keyword::Try);
    map.insert("capturar".to_string(), Keyword::Catch);
    map.insert("finalmente".to_string(), Keyword::Finally);
    map.insert("lanzar".to_string(), Keyword::Throw);

    // Data & Variables
    map.insert("variable".to_string(), Keyword::Let);
    map.insert("constante".to_string(), Keyword::Const);
    map.insert("datos".to_string(), Keyword::Data);
    map.insert("valor".to_string(), Keyword::Set);

    // State
    map.insert("estado".to_string(), Keyword::State);
    map.insert("máquina_de_estados".to_string(), Keyword::StateMachine);

    // Entity
    map.insert("entidad".to_string(), Keyword::Entity);
    map.insert("estado_agente".to_string(), Keyword::AgentState);

    // Intent & Entity
    map.insert("intención".to_string(), Keyword::Intent);
    map.insert("querer".to_string(), Keyword::Want);
    map.insert("prioridad".to_string(), Keyword::Priority);
    map.insert("acción".to_string(), Keyword::Action);
    map.insert("transformar".to_string(), Keyword::Transform);

    // Events
    map.insert("evento".to_string(), Keyword::Event);
    map.insert("en".to_string(), Keyword::On);
    map.insert("disparar".to_string(), Keyword::Trigger);
    map.insert("cuando".to_string(), Keyword::When);

    // Permissions
    map.insert("permitir".to_string(), Keyword::Allow);
    map.insert("denegar".to_string(), Keyword::Deny);

    // Governance
    map.insert("constitución".to_string(), Keyword::Constitution);
    map.insert("ley".to_string(), Keyword::Law);
    map.insert("regla".to_string(), Keyword::Rule);
    map.insert("consejo".to_string(), Keyword::Council);
    map.insert("enjambre".to_string(), Keyword::Swarm);
    map.insert("comunidad".to_string(), Keyword::Community);
    map.insert("aplicación".to_string(), Keyword::Enforcement);
    map.insert("obligatorio".to_string(), Keyword::Mandatory);
    map.insert("consultivo".to_string(), Keyword::Advisory);
    map.insert("opcional".to_string(), Keyword::Optional);
    map.insert("rol".to_string(), Keyword::Role);
    map.insert("miembro".to_string(), Keyword::Member);
    map.insert("estrategia".to_string(), Keyword::Strategy);
    map.insert("competitivo".to_string(), Keyword::Competitive);
    map.insert("colaborativo".to_string(), Keyword::Collaborative);
    map.insert("fiscal".to_string(), Keyword::Prosecutor);
    map.insert("juez".to_string(), Keyword::Judge);
    map.insert("ejecutor".to_string(), Keyword::Executor);

    // Culture
    map.insert("cultura".to_string(), Keyword::Culture);
    map.insert("valores".to_string(), Keyword::Values);
    map.insert("normas".to_string(), Keyword::Norms);
    map.insert(
        "estilo_comunicación".to_string(),
        Keyword::CommunicationStyle,
    );
    map.insert("formal".to_string(), Keyword::Formal);
    map.insert("informal".to_string(), Keyword::Informal);
    map.insert("técnico".to_string(), Keyword::Technical);

    // Flow & Orchestration
    map.insert("flujo".to_string(), Keyword::Flow);
    map.insert("flujo_datos".to_string(), Keyword::DataFlow);
    map.insert("capa".to_string(), Keyword::Layer);
    map.insert("red".to_string(), Keyword::Network);
    map.insert("depende_de".to_string(), Keyword::DependsOn);
    map.insert("difundir".to_string(), Keyword::Broadcast);
    map.insert("fusionar".to_string(), Keyword::Merge);
    map.insert("paralelo".to_string(), Keyword::Parallel);
    map.insert("secuencial".to_string(), Keyword::Sequential);
    map.insert("ejecutar".to_string(), Keyword::Execute);

    // Music Data Structures
    map.insert("nota".to_string(), Keyword::Note);
    map.insert("acorde".to_string(), Keyword::Chord);
    map.insert("melodía".to_string(), Keyword::Melody);
    map.insert("armonía".to_string(), Keyword::Harmony);
    map.insert("ritmo".to_string(), Keyword::Rhythm);
    map.insert("tempo".to_string(), Keyword::Tempo);
    map.insert("escala".to_string(), Keyword::Scale);

    map
}
