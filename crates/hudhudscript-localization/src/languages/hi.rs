//! Hindi keyword mappings
//! हिन्दी - Hindi language support

use crate::keyword_map::Keyword;
use std::collections::HashMap;

pub fn get_keywords() -> HashMap<String, Keyword> {
    let mut map = HashMap::new();

    // Core keywords
    map.insert("मान_लो".to_string(), Keyword::Let);
    map.insert("चर".to_string(), Keyword::Let);
    map.insert("डेटा".to_string(), Keyword::Data);
    map.insert("सेट_करो".to_string(), Keyword::Set);
    map.insert("सत्य".to_string(), Keyword::True);
    map.insert("असत्य".to_string(), Keyword::False);

    // Control flow
    map.insert("अगर".to_string(), Keyword::If);
    map.insert("नहीं_तो".to_string(), Keyword::Else);
    map.insert("जब_तक".to_string(), Keyword::While);
    map.insert("के_लिए".to_string(), Keyword::For);
    map.insert("वापस_करो".to_string(), Keyword::Return);
    map.insert("तोड़ो".to_string(), Keyword::Break);
    map.insert("जारी_रखो".to_string(), Keyword::Continue);
    map.insert("स्विच".to_string(), Keyword::Switch);
    map.insert("केस".to_string(), Keyword::Case);
    map.insert("डिफ़ॉल्ट".to_string(), Keyword::Default);

    // Module system
    map.insert("उपयोग_करो".to_string(), Keyword::Use);
    map.insert("के_रूप_में".to_string(), Keyword::As);
    map.insert("आयात_करो".to_string(), Keyword::Import);
    map.insert("निर्यात_करो".to_string(), Keyword::Export);
    map.insert("से".to_string(), Keyword::From);

    // Agent system
    map.insert("एजेंट".to_string(), Keyword::Agent);
    map.insert("उपकरण".to_string(), Keyword::Tool);
    map.insert("संसाधन".to_string(), Keyword::Resource);
    map.insert("एमसीपी".to_string(), Keyword::Mcp);
    map.insert("सर्वर".to_string(), Keyword::Server);
    map.insert("कॉन्फ़िग".to_string(), Keyword::Config);
    map.insert("प्रदाता".to_string(), Keyword::Provider);
    map.insert("मॉडल".to_string(), Keyword::Model);
    map.insert("कॉल_करो".to_string(), Keyword::Call);

    // Governance
    map.insert("संविधान".to_string(), Keyword::Constitution);
    map.insert("कानून".to_string(), Keyword::Law);
    map.insert("नियम".to_string(), Keyword::Rule);
    map.insert("परिषद".to_string(), Keyword::Council);
    map.insert("झुंड".to_string(), Keyword::Swarm);
    map.insert("समुदाय".to_string(), Keyword::Community);
    map.insert("प्रवर्तन".to_string(), Keyword::Enforcement);
    map.insert("अनिवार्य".to_string(), Keyword::Mandatory);
    map.insert("सलाहकार".to_string(), Keyword::Advisory);
    map.insert("वैकल्पिक".to_string(), Keyword::Optional);
    map.insert("भूमिका".to_string(), Keyword::Role);
    map.insert("सदस्य".to_string(), Keyword::Member);
    map.insert("रणनीति".to_string(), Keyword::Strategy);
    map.insert("प्रतिस्पर्धी".to_string(), Keyword::Competitive);
    map.insert("सहयोगी".to_string(), Keyword::Collaborative);
    map.insert("संस्कृति".to_string(), Keyword::Culture);
    map.insert("मूल्य".to_string(), Keyword::Values);
    map.insert("मानदंड".to_string(), Keyword::Norms);
    map.insert("अभियोजक".to_string(), Keyword::Prosecutor);
    map.insert("न्यायाधीश".to_string(), Keyword::Judge);

    // Async & Error handling
    map.insert("असिंक".to_string(), Keyword::Async);
    map.insert("प्रतीक्षा_करो".to_string(), Keyword::Await);
    map.insert("कोशिश_करो".to_string(), Keyword::Try);
    map.insert("पकड़ो".to_string(), Keyword::Catch);
    map.insert("अंत_में".to_string(), Keyword::Finally);
    map.insert("फेंको".to_string(), Keyword::Throw);
    map.insert("वादा".to_string(), Keyword::Promise);
    map.insert("भविष्य".to_string(), Keyword::Future);

    // Data structures
    map.insert("नोट".to_string(), Keyword::Note);
    map.insert("कॉर्ड".to_string(), Keyword::Chord);
    map.insert("धुन".to_string(), Keyword::Melody);
    map.insert("सामंजस्य".to_string(), Keyword::Harmony);
    map.insert("लय".to_string(), Keyword::Rhythm);
    map.insert("गति".to_string(), Keyword::Tempo);
    map.insert("पैमाना".to_string(), Keyword::Scale);

    // Intent & State
    map.insert("इकाई".to_string(), Keyword::Entity);
    map.insert("स्थिति".to_string(), Keyword::State);
    map.insert("घटना".to_string(), Keyword::Event);

    // Flow & Orchestration
    map.insert("प्रवाह".to_string(), Keyword::Flow);
    map.insert("परत".to_string(), Keyword::Layer);
    map.insert("नेटवर्क".to_string(), Keyword::Network);
    map.insert("निर्भर_करता_है".to_string(), Keyword::DependsOn);
    map.insert("प्रसारण".to_string(), Keyword::Broadcast);
    map.insert("मर्ज_करो".to_string(), Keyword::Merge);
    map.insert("समानांतर".to_string(), Keyword::Parallel);
    map.insert("क्रमिक".to_string(), Keyword::Sequential);
    map.insert("निष्पादित_करो".to_string(), Keyword::Execute);
    map.insert("जब".to_string(), Keyword::When);
    map.insert("पर".to_string(), Keyword::On);
    map.insert("ट्रिगर".to_string(), Keyword::Trigger);
    // Native Hindi translations
    map.insert("क्रिया".to_string(), Keyword::Action);
    map.insert("एजेंट_अवस्था".to_string(), Keyword::AgentState);
    map.insert("अनुमति_दो".to_string(), Keyword::Allow);
    map.insert("संचार_शैली".to_string(), Keyword::CommunicationStyle);
    map.insert("स्थिरांक".to_string(), Keyword::Const);
    map.insert("डेटा_प्रवाह".to_string(), Keyword::DataFlow);
    map.insert("अस्वीकार_करो".to_string(), Keyword::Deny);
    map.insert("निष्पादक".to_string(), Keyword::Executor);
    map.insert("औपचारिक".to_string(), Keyword::Formal);
    map.insert("फ़ंक्शन".to_string(), Keyword::Function);
    map.insert("अनौपचारिक".to_string(), Keyword::Informal);
    map.insert("इरादा".to_string(), Keyword::Intent);
    map.insert("प्राथमिकता".to_string(), Keyword::Priority);
    map.insert("स्थिति_मशीन".to_string(), Keyword::StateMachine);
    map.insert("तकनीकी".to_string(), Keyword::Technical);
    map.insert("रूपांतरित_करो".to_string(), Keyword::Transform);
    map.insert("चर_अन्य".to_string(), Keyword::Var);
    map.insert("चाहना".to_string(), Keyword::Want);

    map
}
