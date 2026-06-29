//! Polish keyword mappings
//! Polski - Polish language support

use crate::keyword_map::Keyword;
use std::collections::HashMap;

pub fn get_keywords() -> HashMap<String, Keyword> {
    let mut map = HashMap::new();

    // Core keywords - Variables & Data
    map.insert("niech".to_string(), Keyword::Let);
    map.insert("zmienna".to_string(), Keyword::Let);
    map.insert("dane".to_string(), Keyword::Data);
    map.insert("ustaw".to_string(), Keyword::Set);
    map.insert("prawda".to_string(), Keyword::True);
    map.insert("fałsz".to_string(), Keyword::False);

    // Control flow
    map.insert("jeśli".to_string(), Keyword::If);
    map.insert("jeżeli".to_string(), Keyword::If);
    map.insert("w_przeciwnym_razie".to_string(), Keyword::Else);
    map.insert("inaczej".to_string(), Keyword::Else);
    map.insert("podczas".to_string(), Keyword::While);
    map.insert("dopóki".to_string(), Keyword::While);
    map.insert("dla".to_string(), Keyword::For);
    map.insert("zwróć".to_string(), Keyword::Return);
    map.insert("powrót".to_string(), Keyword::Return);
    map.insert("przerwij".to_string(), Keyword::Break);
    map.insert("kontynuuj".to_string(), Keyword::Continue);
    map.insert("przełącz".to_string(), Keyword::Switch);
    map.insert("przypadek".to_string(), Keyword::Case);
    map.insert("domyślny".to_string(), Keyword::Default);

    // Module system
    map.insert("użyj".to_string(), Keyword::Use);
    map.insert("jako".to_string(), Keyword::As);
    map.insert("importuj".to_string(), Keyword::Import);
    map.insert("eksportuj".to_string(), Keyword::Export);
    map.insert("z".to_string(), Keyword::From);
    map.insert("od".to_string(), Keyword::From);

    // Agent system
    map.insert("agent".to_string(), Keyword::Agent);
    map.insert("narzędzie".to_string(), Keyword::Tool);
    map.insert("zasób".to_string(), Keyword::Resource);
    map.insert("mcp".to_string(), Keyword::Mcp);
    map.insert("serwer".to_string(), Keyword::Server);
    map.insert("konfiguracja".to_string(), Keyword::Config);
    map.insert("dostawca".to_string(), Keyword::Provider);
    map.insert("model".to_string(), Keyword::Model);
    map.insert("wywołaj".to_string(), Keyword::Call);

    // Governance
    map.insert("konstytucja".to_string(), Keyword::Constitution);
    map.insert("prawo".to_string(), Keyword::Law);
    map.insert("reguła".to_string(), Keyword::Rule);
    map.insert("zasada".to_string(), Keyword::Rule);
    map.insert("rada".to_string(), Keyword::Council);
    map.insert("rój".to_string(), Keyword::Swarm);
    map.insert("społeczność".to_string(), Keyword::Community);
    map.insert("egzekwowanie".to_string(), Keyword::Enforcement);
    map.insert("obowiązkowy".to_string(), Keyword::Mandatory);
    map.insert("doradczy".to_string(), Keyword::Advisory);
    map.insert("opcjonalny".to_string(), Keyword::Optional);
    map.insert("rola".to_string(), Keyword::Role);
    map.insert("członek".to_string(), Keyword::Member);
    map.insert("strategia".to_string(), Keyword::Strategy);
    map.insert("konkurencyjny".to_string(), Keyword::Competitive);
    map.insert("współpracujący".to_string(), Keyword::Collaborative);
    map.insert("kultura".to_string(), Keyword::Culture);
    map.insert("wartości".to_string(), Keyword::Values);
    map.insert("normy".to_string(), Keyword::Norms);
    map.insert("prokurator".to_string(), Keyword::Prosecutor);
    map.insert("sędzia".to_string(), Keyword::Judge);

    // Async & Error handling
    map.insert("asynchroniczny".to_string(), Keyword::Async);
    map.insert("czekaj".to_string(), Keyword::Await);
    map.insert("oczekuj".to_string(), Keyword::Await);
    map.insert("spróbuj".to_string(), Keyword::Try);
    map.insert("złap".to_string(), Keyword::Catch);
    map.insert("przechwyt".to_string(), Keyword::Catch);
    map.insert("w_końcu".to_string(), Keyword::Finally);
    map.insert("ostatecznie".to_string(), Keyword::Finally);
    map.insert("rzuć".to_string(), Keyword::Throw);
    map.insert("obietnica".to_string(), Keyword::Promise);
    map.insert("przyszłość".to_string(), Keyword::Future);

    // Data structures (Music metaphor)
    map.insert("nuta".to_string(), Keyword::Note);
    map.insert("akord".to_string(), Keyword::Chord);
    map.insert("melodia".to_string(), Keyword::Melody);
    map.insert("harmonia".to_string(), Keyword::Harmony);
    map.insert("rytm".to_string(), Keyword::Rhythm);
    map.insert("tempo".to_string(), Keyword::Tempo);
    map.insert("skala".to_string(), Keyword::Scale);

    // Intent & State
    map.insert("encja".to_string(), Keyword::Entity);
    map.insert("stan".to_string(), Keyword::State);
    map.insert("zdarzenie".to_string(), Keyword::Event);

    // Flow & Orchestration
    map.insert("przepływ".to_string(), Keyword::Flow);
    map.insert("warstwa".to_string(), Keyword::Layer);
    map.insert("sieć".to_string(), Keyword::Network);
    map.insert("zależy_od".to_string(), Keyword::DependsOn);
    map.insert("rozgłoś".to_string(), Keyword::Broadcast);
    map.insert("scal".to_string(), Keyword::Merge);
    map.insert("połącz".to_string(), Keyword::Merge);
    map.insert("równolegle".to_string(), Keyword::Parallel);
    map.insert("sekwencyjnie".to_string(), Keyword::Sequential);
    map.insert("wykonaj".to_string(), Keyword::Execute);
    map.insert("kiedy".to_string(), Keyword::When);
    map.insert("na".to_string(), Keyword::On);
    map.insert("wyzwalacz".to_string(), Keyword::Trigger);
    // Native Polish translations
    map.insert("akcja".to_string(), Keyword::Action);
    map.insert("stan_agenta".to_string(), Keyword::AgentState);
    map.insert("zezwól".to_string(), Keyword::Allow);
    map.insert("styl_komunikacji".to_string(), Keyword::CommunicationStyle);
    map.insert("stała".to_string(), Keyword::Const);
    map.insert("przepływ_danych".to_string(), Keyword::DataFlow);
    map.insert("odmów".to_string(), Keyword::Deny);
    map.insert("wykonawca".to_string(), Keyword::Executor);
    map.insert("formalny".to_string(), Keyword::Formal);
    map.insert("funkcja".to_string(), Keyword::Function);
    map.insert("nieformalny".to_string(), Keyword::Informal);
    map.insert("zamiar".to_string(), Keyword::Intent);
    map.insert("priorytet".to_string(), Keyword::Priority);
    map.insert("maszyna_stanów".to_string(), Keyword::StateMachine);
    map.insert("techniczny".to_string(), Keyword::Technical);
    map.insert("przekształć".to_string(), Keyword::Transform);
    map.insert("zmienna_var".to_string(), Keyword::Var);
    map.insert("chcieć".to_string(), Keyword::Want);

    map
}
