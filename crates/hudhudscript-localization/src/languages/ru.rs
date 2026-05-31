//! Russian keyword mappings
//! Русский - Russian language support (SVO, flexible word order)

use crate::keyword_map::Keyword;
use std::collections::HashMap;

pub fn get_keywords() -> HashMap<String, Keyword> {
    let mut map = HashMap::new();

    // Module system
    map.insert("использовать".to_string(), Keyword::Use);
    map.insert("как".to_string(), Keyword::As);
    map.insert("импорт".to_string(), Keyword::Import);
    map.insert("экспорт".to_string(), Keyword::Export);
    map.insert("из".to_string(), Keyword::From);

    // Agent system
    map.insert("агент".to_string(), Keyword::Agent);
    map.insert("задача".to_string(), Keyword::Task);
    map.insert("инструмент".to_string(), Keyword::Tool);
    map.insert("ресурс".to_string(), Keyword::Resource);
    map.insert("mcp".to_string(), Keyword::Mcp);
    map.insert("сервер".to_string(), Keyword::Server);
    map.insert("конфиг".to_string(), Keyword::Config);
    map.insert("провайдер".to_string(), Keyword::Provider);
    map.insert("модель".to_string(), Keyword::Model);
    map.insert("вызвать".to_string(), Keyword::Call);

    // Control flow
    map.insert("если".to_string(), Keyword::If);
    map.insert("иначе".to_string(), Keyword::Else);
    map.insert("пока".to_string(), Keyword::While);
    map.insert("для".to_string(), Keyword::For);
    map.insert("вернуть".to_string(), Keyword::Return);
    map.insert("прервать".to_string(), Keyword::Break);
    map.insert("продолжить".to_string(), Keyword::Continue);
    map.insert("выбрать".to_string(), Keyword::Switch);
    map.insert("случай".to_string(), Keyword::Case);
    map.insert("по_умолчанию".to_string(), Keyword::Default);

    // Functions
    map.insert("функция".to_string(), Keyword::Function);
    map.insert("фн".to_string(), Keyword::Function);

    // Async
    map.insert("асинхронный".to_string(), Keyword::Async);
    map.insert("ждать".to_string(), Keyword::Await);
    map.insert("обещание".to_string(), Keyword::Promise);
    map.insert("будущее".to_string(), Keyword::Future);

    // Error handling
    map.insert("попробовать".to_string(), Keyword::Try);
    map.insert("поймать".to_string(), Keyword::Catch);
    map.insert("наконец".to_string(), Keyword::Finally);
    map.insert("бросить".to_string(), Keyword::Throw);

    // Async
    map.insert("асинхронный".to_string(), Keyword::Async);
    map.insert("ждать".to_string(), Keyword::Await);

    // Data & Variables
    map.insert("переменная".to_string(), Keyword::Let);
    map.insert("константа".to_string(), Keyword::Const);
    map.insert("данные".to_string(), Keyword::Data);
    map.insert("значение".to_string(), Keyword::Set);

    // State
    map.insert("состояние".to_string(), Keyword::State);
    map.insert("машина_состояний".to_string(), Keyword::StateMachine);

    // Entity
    map.insert("сущность".to_string(), Keyword::Entity);
    map.insert("состояние_агента".to_string(), Keyword::AgentState);

    // Intent & Entity
    map.insert("намерение".to_string(), Keyword::Intent);
    map.insert("хотеть".to_string(), Keyword::Want);
    map.insert("приоритет".to_string(), Keyword::Priority);
    map.insert("действие".to_string(), Keyword::Action);
    map.insert("преобразовать".to_string(), Keyword::Transform);

    // Events
    map.insert("событие".to_string(), Keyword::Event);
    map.insert("на".to_string(), Keyword::On);
    map.insert("триггер".to_string(), Keyword::Trigger);
    map.insert("когда".to_string(), Keyword::When);

    // Permissions
    map.insert("разрешить".to_string(), Keyword::Allow);
    map.insert("запретить".to_string(), Keyword::Deny);

    // Governance
    map.insert("конституция".to_string(), Keyword::Constitution);
    map.insert("закон".to_string(), Keyword::Law);
    map.insert("правило".to_string(), Keyword::Rule);
    map.insert("совет".to_string(), Keyword::Council);
    map.insert("рой".to_string(), Keyword::Swarm);
    map.insert("сообщество".to_string(), Keyword::Community);
    map.insert("исполнение".to_string(), Keyword::Enforcement);
    map.insert("обязательный".to_string(), Keyword::Mandatory);
    map.insert("рекомендательный".to_string(), Keyword::Advisory);
    map.insert("необязательный".to_string(), Keyword::Optional);
    map.insert("правило_агента".to_string(), Keyword::AgentRule);
    map.insert("набор_правил".to_string(), Keyword::RuleSet);
    map.insert("цепочка_правил".to_string(), Keyword::RuleChain);
    map.insert("роль".to_string(), Keyword::Role);
    map.insert("член".to_string(), Keyword::Member);
    map.insert("стратегия".to_string(), Keyword::Strategy);
    map.insert("конкурентный".to_string(), Keyword::Competitive);
    map.insert("совместный".to_string(), Keyword::Collaborative);
    map.insert("прокурор".to_string(), Keyword::Prosecutor);
    map.insert("судья".to_string(), Keyword::Judge);
    map.insert("исполнитель".to_string(), Keyword::Executor);

    // Culture
    map.insert("культура".to_string(), Keyword::Culture);
    map.insert("ценности".to_string(), Keyword::Values);
    map.insert("нормы".to_string(), Keyword::Norms);
    map.insert("стиль_общения".to_string(), Keyword::CommunicationStyle);
    map.insert("формальный".to_string(), Keyword::Formal);
    map.insert("неформальный".to_string(), Keyword::Informal);
    map.insert("технический".to_string(), Keyword::Technical);

    // Flow & Orchestration
    map.insert("поток".to_string(), Keyword::Flow);
    map.insert("поток_данных".to_string(), Keyword::DataFlow);
    map.insert("слой".to_string(), Keyword::Layer);
    map.insert("сеть".to_string(), Keyword::Network);
    map.insert("зависит_от".to_string(), Keyword::DependsOn);
    map.insert("транслировать".to_string(), Keyword::Broadcast);
    map.insert("объединить".to_string(), Keyword::Merge);
    map.insert("параллельный".to_string(), Keyword::Parallel);
    map.insert("последовательный".to_string(), Keyword::Sequential);
    map.insert("выполнить".to_string(), Keyword::Execute);

    // Music Data Structures
    map.insert("нота".to_string(), Keyword::Note);
    map.insert("аккорд".to_string(), Keyword::Chord);
    map.insert("мелодия".to_string(), Keyword::Melody);
    map.insert("гармония".to_string(), Keyword::Harmony);
    map.insert("ритм".to_string(), Keyword::Rhythm);
    map.insert("темп".to_string(), Keyword::Tempo);
    map.insert("гамма".to_string(), Keyword::Scale);

    map
}
