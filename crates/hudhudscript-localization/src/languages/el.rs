//! Greek keyword mappings
//! Ελληνικά - Greek language support

use crate::keyword_map::Keyword;
use std::collections::HashMap;

pub fn get_keywords() -> HashMap<String, Keyword> {
    let mut map = HashMap::new();

    // Core keywords
    map.insert("ας".to_string(), Keyword::Let);
    map.insert("μεταβλητή".to_string(), Keyword::Let);
    map.insert("δεδομένα".to_string(), Keyword::Data);
    map.insert("όρισε".to_string(), Keyword::Set);
    map.insert("αληθής".to_string(), Keyword::True);
    map.insert("ψευδής".to_string(), Keyword::False);

    // Control flow
    map.insert("αν".to_string(), Keyword::If);
    map.insert("αλλιώς".to_string(), Keyword::Else);
    map.insert("ενώ".to_string(), Keyword::While);
    map.insert("για".to_string(), Keyword::For);
    map.insert("επιστροφή".to_string(), Keyword::Return);
    map.insert("διακοπή".to_string(), Keyword::Break);
    map.insert("συνέχεια".to_string(), Keyword::Continue);
    map.insert("εναλλαγή".to_string(), Keyword::Switch);
    map.insert("περίπτωση".to_string(), Keyword::Case);
    map.insert("προεπιλογή".to_string(), Keyword::Default);

    // Module system
    map.insert("χρήση".to_string(), Keyword::Use);
    map.insert("ως".to_string(), Keyword::As);
    map.insert("εισαγωγή".to_string(), Keyword::Import);
    map.insert("εξαγωγή".to_string(), Keyword::Export);
    map.insert("από".to_string(), Keyword::From);

    // Agent system
    map.insert("πράκτορας".to_string(), Keyword::Agent);
    map.insert("εργασία".to_string(), Keyword::Task);
    map.insert("εργαλείο".to_string(), Keyword::Tool);
    map.insert("πόρος".to_string(), Keyword::Resource);
    map.insert("mcp".to_string(), Keyword::Mcp);
    map.insert("διακομιστής".to_string(), Keyword::Server);
    map.insert("διαμόρφωση".to_string(), Keyword::Config);
    map.insert("πάροχος".to_string(), Keyword::Provider);
    map.insert("μοντέλο".to_string(), Keyword::Model);
    map.insert("κλήση".to_string(), Keyword::Call);

    // Governance
    map.insert("σύνταγμα".to_string(), Keyword::Constitution);
    map.insert("νόμος".to_string(), Keyword::Law);
    map.insert("κανόνας".to_string(), Keyword::Rule);
    map.insert("συμβούλιο".to_string(), Keyword::Council);
    map.insert("σμήνος".to_string(), Keyword::Swarm);
    map.insert("κοινότητα".to_string(), Keyword::Community);
    map.insert("επιβολή".to_string(), Keyword::Enforcement);
    map.insert("υποχρεωτικός".to_string(), Keyword::Mandatory);
    map.insert("συμβουλευτικός".to_string(), Keyword::Advisory);
    map.insert("προαιρετικός".to_string(), Keyword::Optional);
    map.insert("ρόλος".to_string(), Keyword::Role);
    map.insert("μέλος".to_string(), Keyword::Member);
    map.insert("στρατηγική".to_string(), Keyword::Strategy);
    map.insert("ανταγωνιστικός".to_string(), Keyword::Competitive);
    map.insert("συνεργατικός".to_string(), Keyword::Collaborative);
    map.insert("κουλτούρα".to_string(), Keyword::Culture);
    map.insert("αξίες".to_string(), Keyword::Values);
    map.insert("κανόνες".to_string(), Keyword::Norms);
    map.insert("εισαγγελέας".to_string(), Keyword::Prosecutor);
    map.insert("δικαστής".to_string(), Keyword::Judge);

    // Async & Error handling
    map.insert("ασύγχρονος".to_string(), Keyword::Async);
    map.insert("αναμονή".to_string(), Keyword::Await);
    map.insert("δοκίμασε".to_string(), Keyword::Try);
    map.insert("πιάσε".to_string(), Keyword::Catch);
    map.insert("τελικά".to_string(), Keyword::Finally);
    map.insert("ρίξε".to_string(), Keyword::Throw);
    map.insert("υπόσχεση".to_string(), Keyword::Promise);
    map.insert("μέλλον".to_string(), Keyword::Future);

    // Data structures
    map.insert("νότα".to_string(), Keyword::Note);
    map.insert("συγχορδία".to_string(), Keyword::Chord);
    map.insert("μελωδία".to_string(), Keyword::Melody);
    map.insert("αρμονία".to_string(), Keyword::Harmony);
    map.insert("ρυθμός".to_string(), Keyword::Rhythm);
    map.insert("τέμπο".to_string(), Keyword::Tempo);
    map.insert("κλίμακα".to_string(), Keyword::Scale);

    // Intent & State
    map.insert("οντότητα".to_string(), Keyword::Entity);
    map.insert("κατάσταση".to_string(), Keyword::State);
    map.insert("συμβάν".to_string(), Keyword::Event);

    // Flow & Orchestration
    map.insert("ροή".to_string(), Keyword::Flow);
    map.insert("στρώμα".to_string(), Keyword::Layer);
    map.insert("δίκτυο".to_string(), Keyword::Network);
    map.insert("εξαρτάται_από".to_string(), Keyword::DependsOn);
    map.insert("μετάδοση".to_string(), Keyword::Broadcast);
    map.insert("συγχώνευση".to_string(), Keyword::Merge);
    map.insert("παράλληλα".to_string(), Keyword::Parallel);
    map.insert("διαδοχικά".to_string(), Keyword::Sequential);
    map.insert("εκτέλεση".to_string(), Keyword::Execute);
    map.insert("όταν".to_string(), Keyword::When);
    map.insert("σε".to_string(), Keyword::On);
    map.insert("σκανδάλη".to_string(), Keyword::Trigger);
    // Native Greek translations
    map.insert("ενέργεια".to_string(), Keyword::Action);
    map.insert("κανόνας_πράκτορα".to_string(), Keyword::AgentRule);
    map.insert("κατάσταση_πράκτορα".to_string(), Keyword::AgentState);
    map.insert("επίτρεψε".to_string(), Keyword::Allow);
    map.insert("στυλ_επικοινωνίας".to_string(), Keyword::CommunicationStyle);
    map.insert("σταθερά".to_string(), Keyword::Const);
    map.insert("ροή_δεδομένων".to_string(), Keyword::DataFlow);
    map.insert("αρνήσου".to_string(), Keyword::Deny);
    map.insert("εκτελεστής".to_string(), Keyword::Executor);
    map.insert("τυπικό".to_string(), Keyword::Formal);
    map.insert("συνάρτηση".to_string(), Keyword::Function);
    map.insert("ανεπίσημο".to_string(), Keyword::Informal);
    map.insert("πρόθεση".to_string(), Keyword::Intent);
    map.insert("προτεραιότητα".to_string(), Keyword::Priority);
    map.insert("αλυσίδα_κανόνων".to_string(), Keyword::RuleChain);
    map.insert("σύνολο_κανόνων".to_string(), Keyword::RuleSet);
    map.insert("μηχανή_καταστάσεων".to_string(), Keyword::StateMachine);
    map.insert("τεχνικό".to_string(), Keyword::Technical);
    map.insert("μετασχηματίζω".to_string(), Keyword::Transform);
    map.insert("μεταβλητή".to_string(), Keyword::Var);
    map.insert("θέλω".to_string(), Keyword::Want);

    map
}
