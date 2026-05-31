//! Turkish keyword mappings
//! Türkçe - Turkish language support (SOV word order)

use crate::keyword_map::Keyword;
use std::collections::HashMap;

pub fn get_keywords() -> HashMap<String, Keyword> {
    let mut map = HashMap::new();

    // Module system
    map.insert("kullan".to_string(), Keyword::Use);
    map.insert("olarak".to_string(), Keyword::As);
    map.insert("içe_aktar".to_string(), Keyword::Import);
    map.insert("dışa_aktar".to_string(), Keyword::Export);
    map.insert("den".to_string(), Keyword::From);

    // Agent system
    map.insert("ajani".to_string(), Keyword::Agent);
    map.insert("ajan".to_string(), Keyword::Agent);
    map.insert("görev".to_string(), Keyword::Task);
    map.insert("araç".to_string(), Keyword::Tool);
    map.insert("kaynak".to_string(), Keyword::Resource);
    map.insert("mcp".to_string(), Keyword::Mcp);
    map.insert("sunucu".to_string(), Keyword::Server);
    map.insert("yapılandırma".to_string(), Keyword::Config);

    // Control flow
    map.insert("eğer".to_string(), Keyword::If);
    map.insert("ise".to_string(), Keyword::Then); // eğer ... ise
    map.insert("değilse".to_string(), Keyword::Else);
    map.insert("iken".to_string(), Keyword::While);
    map.insert("için".to_string(), Keyword::For);
    map.insert("herbir".to_string(), Keyword::ForEach);
    map.insert("içinde".to_string(), Keyword::In);
    map.insert("dön".to_string(), Keyword::Return);
    map.insert("kır".to_string(), Keyword::Break);
    map.insert("çık".to_string(), Keyword::Break);
    map.insert("devam".to_string(), Keyword::Continue);
    map.insert("seçim".to_string(), Keyword::Switch);
    map.insert("hal".to_string(), Keyword::Case); // Changed from "durum" to avoid conflict
    map.insert("varsayılan".to_string(), Keyword::Default);

    // Logical operators
    map.insert("ve".to_string(), Keyword::And); // Logical AND
    map.insert("veya".to_string(), Keyword::Or); // Logical OR

    // Error handling
    map.insert("dene".to_string(), Keyword::Try);
    map.insert("yakala".to_string(), Keyword::Catch);
    map.insert("sonunda".to_string(), Keyword::Finally);
    map.insert("fırlat".to_string(), Keyword::Throw);

    // Async
    map.insert("eşzamansız".to_string(), Keyword::Async);
    map.insert("bekle".to_string(), Keyword::Await);

    // Functions
    map.insert("işlev".to_string(), Keyword::Function);
    map.insert("iş".to_string(), Keyword::Function);

    // OOP Keywords
    map.insert("sınıf".to_string(), Keyword::Class);
    map.insert("kurucu".to_string(), Keyword::Constructor);
    map.insert("bu".to_string(), Keyword::This);
    map.insert("kendi".to_string(), Keyword::SelfKeyword);
    map.insert("üst".to_string(), Keyword::Super);
    map.insert("yeni".to_string(), Keyword::New);
    map.insert("açık".to_string(), Keyword::Public);
    map.insert("gizli".to_string(), Keyword::Private);
    map.insert("saklı".to_string(), Keyword::Protected);
    map.insert("statik".to_string(), Keyword::Static);

    // Data & Variables
    map.insert("tanım".to_string(), Keyword::Let);
    map.insert("değişken".to_string(), Keyword::Let);
    map.insert("veri".to_string(), Keyword::Data);
    map.insert("değer".to_string(), Keyword::Set);

    // Intent-Based Constructs
    map.insert("varlık".to_string(), Keyword::Entity);
    map.insert("durum".to_string(), Keyword::State);
    map.insert("ajan_durumu".to_string(), Keyword::AgentState);
    map.insert("olay".to_string(), Keyword::Event);
    map.insert("eylem".to_string(), Keyword::Action);
    map.insert("dönüşüm".to_string(), Keyword::Transform);

    // Governance
    map.insert("anayasa".to_string(), Keyword::Constitution);
    map.insert("yasa".to_string(), Keyword::Law);
    map.insert("kural".to_string(), Keyword::Rule);
    map.insert("ajan_kuralı".to_string(), Keyword::AgentRule);
    map.insert("kural_seti".to_string(), Keyword::RuleSet);
    map.insert("kural_zinciri".to_string(), Keyword::RuleChain);
    map.insert("konsey".to_string(), Keyword::Council);
    map.insert("sürü".to_string(), Keyword::Swarm);
    map.insert("topluluk".to_string(), Keyword::Community);
    map.insert("uygulama".to_string(), Keyword::Enforcement);
    map.insert("zorunlu".to_string(), Keyword::Mandatory);
    map.insert("tavsiye".to_string(), Keyword::Advisory);
    map.insert("isteğe_bağlı".to_string(), Keyword::Optional);
    map.insert("rol".to_string(), Keyword::Role);
    map.insert("üye".to_string(), Keyword::Member);
    map.insert("strateji".to_string(), Keyword::Strategy);
    map.insert("rekabetçi".to_string(), Keyword::Competitive);
    map.insert("işbirlikçi".to_string(), Keyword::Collaborative);
    map.insert("kültür".to_string(), Keyword::Culture);
    map.insert("değerler".to_string(), Keyword::Values);
    map.insert("normlar".to_string(), Keyword::Norms);
    map.insert("iletişim_tarzı".to_string(), Keyword::CommunicationStyle);
    map.insert("resmi".to_string(), Keyword::Formal);
    map.insert("gayri_resmi".to_string(), Keyword::Informal);
    map.insert("teknik".to_string(), Keyword::Technical);

    // Intent & Goals
    map.insert("niyet".to_string(), Keyword::Intent);
    map.insert("iste".to_string(), Keyword::Want);
    map.insert("öncelik".to_string(), Keyword::Priority);

    // Flow
    map.insert("akış".to_string(), Keyword::Flow);
    map.insert("veri_akışı".to_string(), Keyword::DataFlow);

    // Agent Roles
    map.insert("savcı".to_string(), Keyword::Prosecutor);
    map.insert("hakim".to_string(), Keyword::Judge);
    map.insert("yürütücü".to_string(), Keyword::Executor);

    // Music Data Structures
    map.insert("nota".to_string(), Keyword::Note);
    map.insert("akor".to_string(), Keyword::Chord);
    map.insert("melodi".to_string(), Keyword::Melody);
    map.insert("armoni".to_string(), Keyword::Harmony);
    map.insert("ritim".to_string(), Keyword::Rhythm);
    map.insert("tempo".to_string(), Keyword::Tempo);
    map.insert("skala".to_string(), Keyword::Scale);

    // Constraints
    map.insert("ne_zaman".to_string(), Keyword::When);
    map.insert("üzerinde".to_string(), Keyword::On);
    map.insert("tetikle".to_string(), Keyword::Trigger);

    // Permissions
    map.insert("izin_ver".to_string(), Keyword::Allow);
    map.insert("yasakla".to_string(), Keyword::Deny);

    // Layer-Based Orchestration
    map.insert("katman".to_string(), Keyword::Layer);
    map.insert("ağ".to_string(), Keyword::Network);
    map.insert("bağlı".to_string(), Keyword::DependsOn);
    map.insert("yayınla".to_string(), Keyword::Broadcast);
    map.insert("birleştir".to_string(), Keyword::Merge);
    map.insert("paralel".to_string(), Keyword::Parallel);
    map.insert("sıralı".to_string(), Keyword::Sequential);
    map.insert("çalıştır".to_string(), Keyword::Execute);

    // Additional Keywords (missing from original)
    map.insert("sağlayıcı".to_string(), Keyword::Provider);
    map.insert("model".to_string(), Keyword::Model);
    map.insert("çağır".to_string(), Keyword::Call);
    map.insert("doğru".to_string(), Keyword::True);
    map.insert("yanlış".to_string(), Keyword::False);
    map.insert("söz".to_string(), Keyword::Promise);
    map.insert("gelecek".to_string(), Keyword::Future);

    map
}
