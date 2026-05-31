//! Serbian keyword mappings (Cyrillic)
//! Српски - Serbian language support

use crate::keyword_map::Keyword;
use std::collections::HashMap;

pub fn get_keywords() -> HashMap<String, Keyword> {
    let mut map = HashMap::new();

    // Core keywords
    map.insert("нека".to_string(), Keyword::Let);
    map.insert("променљива".to_string(), Keyword::Let);
    map.insert("подаци".to_string(), Keyword::Data);
    map.insert("постави".to_string(), Keyword::Set);
    map.insert("тачно".to_string(), Keyword::True);
    map.insert("нетачно".to_string(), Keyword::False);

    // Control flow
    map.insert("ако".to_string(), Keyword::If);
    map.insert("иначе".to_string(), Keyword::Else);
    map.insert("док".to_string(), Keyword::While);
    map.insert("за".to_string(), Keyword::For);
    map.insert("врати".to_string(), Keyword::Return);
    map.insert("прекини".to_string(), Keyword::Break);
    map.insert("настави".to_string(), Keyword::Continue);
    map.insert("пребаци".to_string(), Keyword::Switch);
    map.insert("случај".to_string(), Keyword::Case);
    map.insert("подразумевано".to_string(), Keyword::Default);

    // Module system
    map.insert("користи".to_string(), Keyword::Use);
    map.insert("као".to_string(), Keyword::As);
    map.insert("увези".to_string(), Keyword::Import);
    map.insert("извези".to_string(), Keyword::Export);
    map.insert("од".to_string(), Keyword::From);

    // Agent system
    map.insert("агент".to_string(), Keyword::Agent);
    map.insert("задатак".to_string(), Keyword::Task);
    map.insert("алат".to_string(), Keyword::Tool);
    map.insert("ресурс".to_string(), Keyword::Resource);
    map.insert("мцп".to_string(), Keyword::Mcp);
    map.insert("сервер".to_string(), Keyword::Server);
    map.insert("конфигурација".to_string(), Keyword::Config);
    map.insert("провајдер".to_string(), Keyword::Provider);
    map.insert("модел".to_string(), Keyword::Model);
    map.insert("позови".to_string(), Keyword::Call);

    // Governance
    map.insert("устав".to_string(), Keyword::Constitution);
    map.insert("закон".to_string(), Keyword::Law);
    map.insert("правило".to_string(), Keyword::Rule);
    map.insert("савет".to_string(), Keyword::Council);
    map.insert("рој".to_string(), Keyword::Swarm);
    map.insert("заједница".to_string(), Keyword::Community);
    map.insert("спровођење".to_string(), Keyword::Enforcement);
    map.insert("обавезно".to_string(), Keyword::Mandatory);
    map.insert("саветодавно".to_string(), Keyword::Advisory);
    map.insert("опционо".to_string(), Keyword::Optional);
    map.insert("улога".to_string(), Keyword::Role);
    map.insert("члан".to_string(), Keyword::Member);
    map.insert("стратегија".to_string(), Keyword::Strategy);
    map.insert("конкурентан".to_string(), Keyword::Competitive);
    map.insert("сарадљив".to_string(), Keyword::Collaborative);
    map.insert("култура".to_string(), Keyword::Culture);
    map.insert("вредности".to_string(), Keyword::Values);
    map.insert("норме".to_string(), Keyword::Norms);
    map.insert("тужилац".to_string(), Keyword::Prosecutor);
    map.insert("судија".to_string(), Keyword::Judge);

    // Async & Error handling
    map.insert("асинхрони".to_string(), Keyword::Async);
    map.insert("чекај".to_string(), Keyword::Await);
    map.insert("покушај".to_string(), Keyword::Try);
    map.insert("ухвати".to_string(), Keyword::Catch);
    map.insert("коначно".to_string(), Keyword::Finally);
    map.insert("баци".to_string(), Keyword::Throw);
    map.insert("обећање".to_string(), Keyword::Promise);
    map.insert("будућност".to_string(), Keyword::Future);

    // Data structures
    map.insert("нота".to_string(), Keyword::Note);
    map.insert("акорд".to_string(), Keyword::Chord);
    map.insert("мелодија".to_string(), Keyword::Melody);
    map.insert("хармонија".to_string(), Keyword::Harmony);
    map.insert("ритам".to_string(), Keyword::Rhythm);
    map.insert("темпо".to_string(), Keyword::Tempo);
    map.insert("скала".to_string(), Keyword::Scale);

    // Intent & State
    map.insert("ентитет".to_string(), Keyword::Entity);
    map.insert("стање".to_string(), Keyword::State);
    map.insert("догађај".to_string(), Keyword::Event);

    // Flow & Orchestration
    map.insert("ток".to_string(), Keyword::Flow);
    map.insert("слој".to_string(), Keyword::Layer);
    map.insert("мрежа".to_string(), Keyword::Network);
    map.insert("зависи_од".to_string(), Keyword::DependsOn);
    map.insert("емитуј".to_string(), Keyword::Broadcast);
    map.insert("споји".to_string(), Keyword::Merge);
    map.insert("паралелно".to_string(), Keyword::Parallel);
    map.insert("секвенцијално".to_string(), Keyword::Sequential);
    map.insert("изврши".to_string(), Keyword::Execute);
    map.insert("када".to_string(), Keyword::When);
    map.insert("на".to_string(), Keyword::On);
    map.insert("окидач".to_string(), Keyword::Trigger);
    // Native Serbian translations (Cyrillic)
    map.insert("акција".to_string(), Keyword::Action);
    map.insert("правило_агента".to_string(), Keyword::AgentRule);
    map.insert("стање_агента".to_string(), Keyword::AgentState);
    map.insert("дозволи".to_string(), Keyword::Allow);
    map.insert("стил_комуникације".to_string(), Keyword::CommunicationStyle);
    map.insert("константа".to_string(), Keyword::Const);
    map.insert("ток_података".to_string(), Keyword::DataFlow);
    map.insert("забрани".to_string(), Keyword::Deny);
    map.insert("извршилац".to_string(), Keyword::Executor);
    map.insert("формално".to_string(), Keyword::Formal);
    map.insert("функција".to_string(), Keyword::Function);
    map.insert("неформално".to_string(), Keyword::Informal);
    map.insert("намера".to_string(), Keyword::Intent);
    map.insert("приоритет".to_string(), Keyword::Priority);
    map.insert("ланац_правила".to_string(), Keyword::RuleChain);
    map.insert("скуп_правила".to_string(), Keyword::RuleSet);
    map.insert("машина_стања".to_string(), Keyword::StateMachine);
    map.insert("техничко".to_string(), Keyword::Technical);
    map.insert("трансформисати".to_string(), Keyword::Transform);
    map.insert("промењљива".to_string(), Keyword::Var);
    map.insert("желети".to_string(), Keyword::Want);

    map
}
