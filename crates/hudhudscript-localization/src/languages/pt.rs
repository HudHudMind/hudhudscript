//! Portuguese keyword mappings
//! Português - Portuguese language support (SVO)

use crate::keyword_map::Keyword;
use std::collections::HashMap;

pub fn get_keywords() -> HashMap<String, Keyword> {
    let mut map = HashMap::new();

    // Module system
    map.insert("usar".to_string(), Keyword::Use);
    map.insert("como".to_string(), Keyword::As);
    map.insert("importar".to_string(), Keyword::Import);
    map.insert("exportar".to_string(), Keyword::Export);
    map.insert("de".to_string(), Keyword::From);

    // Agent system
    map.insert("agente".to_string(), Keyword::Agent);
    map.insert("ferramenta".to_string(), Keyword::Tool);
    map.insert("recurso".to_string(), Keyword::Resource);
    map.insert("mcp".to_string(), Keyword::Mcp);
    map.insert("servidor".to_string(), Keyword::Server);
    map.insert("configuração".to_string(), Keyword::Config);

    // Control flow
    map.insert("se".to_string(), Keyword::If);
    map.insert("senão".to_string(), Keyword::Else);
    map.insert("enquanto".to_string(), Keyword::While);
    map.insert("para".to_string(), Keyword::For);
    map.insert("retornar".to_string(), Keyword::Return);
    map.insert("quebrar".to_string(), Keyword::Break);
    map.insert("continuar".to_string(), Keyword::Continue);
    map.insert("mudar".to_string(), Keyword::Switch);
    map.insert("caso".to_string(), Keyword::Case);
    map.insert("padrão".to_string(), Keyword::Default);

    // Error handling
    map.insert("tentar".to_string(), Keyword::Try);
    map.insert("capturar".to_string(), Keyword::Catch);
    map.insert("finalmente".to_string(), Keyword::Finally);
    map.insert("lançar".to_string(), Keyword::Throw);

    // Async
    map.insert("assíncrono".to_string(), Keyword::Async);
    map.insert("aguardar".to_string(), Keyword::Await);

    // Data & Variables
    map.insert("variável".to_string(), Keyword::Let);
    map.insert("dados".to_string(), Keyword::Data);
    map.insert("valor".to_string(), Keyword::Set);

    // Governance
    map.insert("constituição".to_string(), Keyword::Constitution);
    map.insert("lei".to_string(), Keyword::Law);
    map.insert("regra".to_string(), Keyword::Rule);
    map.insert("conselho".to_string(), Keyword::Council);
    map.insert("enxame".to_string(), Keyword::Swarm);
    map.insert("comunidade".to_string(), Keyword::Community);
    map.insert("obrigatório".to_string(), Keyword::Mandatory);
    map.insert("consultivo".to_string(), Keyword::Advisory);
    map.insert("opcional".to_string(), Keyword::Optional);
    map.insert("papel".to_string(), Keyword::Role);
    map.insert("membro".to_string(), Keyword::Member);
    map.insert("estratégia".to_string(), Keyword::Strategy);
    map.insert("competitivo".to_string(), Keyword::Competitive);
    map.insert("colaborativo".to_string(), Keyword::Collaborative);
    map.insert("paralelo".to_string(), Keyword::Parallel);
    map.insert("sequencial".to_string(), Keyword::Sequential);
    map.insert("executar".to_string(), Keyword::Execute);
    // Function keyword
    map.insert("função".to_string(), Keyword::Function);

    // Native Portuguese translations for governance/intent/flow keywords
    map.insert("ação".to_string(), Keyword::Action);
    map.insert("estado_agente".to_string(), Keyword::AgentState);
    map.insert("permitir".to_string(), Keyword::Allow);
    map.insert("transmitir".to_string(), Keyword::Broadcast);
    map.insert("chamar".to_string(), Keyword::Call);
    map.insert("acorde".to_string(), Keyword::Chord);
    map.insert(
        "estilo_comunicação".to_string(),
        Keyword::CommunicationStyle,
    );
    map.insert("constante".to_string(), Keyword::Const);
    map.insert("cultura".to_string(), Keyword::Culture);
    map.insert("fluxo_dados".to_string(), Keyword::DataFlow);
    map.insert("negar".to_string(), Keyword::Deny);
    map.insert("depende_de".to_string(), Keyword::DependsOn);
    map.insert("aplicação".to_string(), Keyword::Enforcement);
    map.insert("entidade".to_string(), Keyword::Entity);
    map.insert("evento".to_string(), Keyword::Event);
    map.insert("executor".to_string(), Keyword::Executor);
    map.insert("fluxo".to_string(), Keyword::Flow);
    map.insert("formal".to_string(), Keyword::Formal);
    map.insert("futuro".to_string(), Keyword::Future);
    map.insert("harmonia".to_string(), Keyword::Harmony);
    map.insert("informal".to_string(), Keyword::Informal);
    map.insert("intenção".to_string(), Keyword::Intent);
    map.insert("juiz".to_string(), Keyword::Judge);
    map.insert("camada".to_string(), Keyword::Layer);
    map.insert("melodia".to_string(), Keyword::Melody);
    map.insert("mesclar".to_string(), Keyword::Merge);
    map.insert("modelo".to_string(), Keyword::Model);
    map.insert("rede".to_string(), Keyword::Network);
    map.insert("normas".to_string(), Keyword::Norms);
    map.insert("nota".to_string(), Keyword::Note);
    map.insert("em".to_string(), Keyword::On);
    map.insert("prioridade".to_string(), Keyword::Priority);
    map.insert("promessa".to_string(), Keyword::Promise);
    map.insert("promotor".to_string(), Keyword::Prosecutor);
    map.insert("provedor".to_string(), Keyword::Provider);
    map.insert("ritmo".to_string(), Keyword::Rhythm);
    map.insert("escala".to_string(), Keyword::Scale);
    map.insert("estado".to_string(), Keyword::State);
    map.insert("máquina_estados".to_string(), Keyword::StateMachine);
    map.insert("técnico".to_string(), Keyword::Technical);
    map.insert("tempo".to_string(), Keyword::Tempo);
    map.insert("transformar".to_string(), Keyword::Transform);
    map.insert("gatilho".to_string(), Keyword::Trigger);
    map.insert("valores".to_string(), Keyword::Values);
    map.insert("variável".to_string(), Keyword::Var);
    map.insert("querer".to_string(), Keyword::Want);
    map.insert("quando".to_string(), Keyword::When);

    map
}
