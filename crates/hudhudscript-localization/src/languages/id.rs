//! Indonesian keyword mappings
//! Bahasa Indonesia - Indonesian language support

use crate::keyword_map::Keyword;
use std::collections::HashMap;

pub fn get_keywords() -> HashMap<String, Keyword> {
    let mut map = HashMap::new();

    // Core keywords
    map.insert("biarkan".to_string(), Keyword::Let);
    map.insert("variabel".to_string(), Keyword::Let);
    map.insert("data".to_string(), Keyword::Data);
    map.insert("atur".to_string(), Keyword::Set);
    map.insert("benar".to_string(), Keyword::True);
    map.insert("salah".to_string(), Keyword::False);

    // Control flow
    map.insert("jika".to_string(), Keyword::If);
    map.insert("jika_tidak".to_string(), Keyword::Else);
    map.insert("lainnya".to_string(), Keyword::Else);
    map.insert("selama".to_string(), Keyword::While);
    map.insert("untuk".to_string(), Keyword::For);
    map.insert("kembali".to_string(), Keyword::Return);
    map.insert("hentikan".to_string(), Keyword::Break);
    map.insert("lanjutkan".to_string(), Keyword::Continue);
    map.insert("ganti".to_string(), Keyword::Switch);
    map.insert("kasus".to_string(), Keyword::Case);
    map.insert("bawaan".to_string(), Keyword::Default);

    // Module system
    map.insert("gunakan".to_string(), Keyword::Use);
    map.insert("sebagai".to_string(), Keyword::As);
    map.insert("impor".to_string(), Keyword::Import);
    map.insert("ekspor".to_string(), Keyword::Export);
    map.insert("dari".to_string(), Keyword::From);

    // Agent system
    map.insert("agen".to_string(), Keyword::Agent);
    map.insert("tugas".to_string(), Keyword::Task);
    map.insert("alat".to_string(), Keyword::Tool);
    map.insert("sumber_daya".to_string(), Keyword::Resource);
    map.insert("mcp".to_string(), Keyword::Mcp);
    map.insert("server".to_string(), Keyword::Server);
    map.insert("konfigurasi".to_string(), Keyword::Config);
    map.insert("penyedia".to_string(), Keyword::Provider);
    map.insert("model".to_string(), Keyword::Model);
    map.insert("panggil".to_string(), Keyword::Call);

    // Governance
    map.insert("konstitusi".to_string(), Keyword::Constitution);
    map.insert("hukum".to_string(), Keyword::Law);
    map.insert("aturan".to_string(), Keyword::Rule);
    map.insert("dewan".to_string(), Keyword::Council);
    map.insert("kawanan".to_string(), Keyword::Swarm);
    map.insert("komunitas".to_string(), Keyword::Community);
    map.insert("penegakan".to_string(), Keyword::Enforcement);
    map.insert("wajib".to_string(), Keyword::Mandatory);
    map.insert("nasihat".to_string(), Keyword::Advisory);
    map.insert("opsional".to_string(), Keyword::Optional);
    map.insert("peran".to_string(), Keyword::Role);
    map.insert("anggota".to_string(), Keyword::Member);
    map.insert("strategi".to_string(), Keyword::Strategy);
    map.insert("kompetitif".to_string(), Keyword::Competitive);
    map.insert("kolaboratif".to_string(), Keyword::Collaborative);
    map.insert("budaya".to_string(), Keyword::Culture);
    map.insert("nilai".to_string(), Keyword::Values);
    map.insert("norma".to_string(), Keyword::Norms);
    map.insert("jaksa".to_string(), Keyword::Prosecutor);
    map.insert("hakim".to_string(), Keyword::Judge);

    // Async & Error handling
    map.insert("asinkron".to_string(), Keyword::Async);
    map.insert("tunggu".to_string(), Keyword::Await);
    map.insert("coba".to_string(), Keyword::Try);
    map.insert("tangkap".to_string(), Keyword::Catch);
    map.insert("akhirnya".to_string(), Keyword::Finally);
    map.insert("lempar".to_string(), Keyword::Throw);
    map.insert("janji".to_string(), Keyword::Promise);
    map.insert("masa_depan".to_string(), Keyword::Future);

    // Data structures
    map.insert("catatan".to_string(), Keyword::Note);
    map.insert("akord".to_string(), Keyword::Chord);
    map.insert("melodi".to_string(), Keyword::Melody);
    map.insert("harmoni".to_string(), Keyword::Harmony);
    map.insert("ritme".to_string(), Keyword::Rhythm);
    map.insert("tempo".to_string(), Keyword::Tempo);
    map.insert("skala".to_string(), Keyword::Scale);

    // Intent & State
    map.insert("entitas".to_string(), Keyword::Entity);
    map.insert("keadaan".to_string(), Keyword::State);
    map.insert("peristiwa".to_string(), Keyword::Event);

    // Flow & Orchestration
    map.insert("aliran".to_string(), Keyword::Flow);
    map.insert("lapisan".to_string(), Keyword::Layer);
    map.insert("jaringan".to_string(), Keyword::Network);
    map.insert("tergantung_pada".to_string(), Keyword::DependsOn);
    map.insert("siarkan".to_string(), Keyword::Broadcast);
    map.insert("gabung".to_string(), Keyword::Merge);
    map.insert("paralel".to_string(), Keyword::Parallel);
    map.insert("berurutan".to_string(), Keyword::Sequential);
    map.insert("jalankan".to_string(), Keyword::Execute);
    map.insert("ketika".to_string(), Keyword::When);
    map.insert("pada".to_string(), Keyword::On);
    map.insert("pemicu".to_string(), Keyword::Trigger);

    // Additional keywords (Indonesian translations)
    map.insert("aksi".to_string(), Keyword::Action);
    map.insert("aturan_agen".to_string(), Keyword::AgentRule);
    map.insert("keadaan_agen".to_string(), Keyword::AgentState);
    map.insert("izinkan".to_string(), Keyword::Allow);
    map.insert("gaya_komunikasi".to_string(), Keyword::CommunicationStyle);
    map.insert("konstan".to_string(), Keyword::Const);
    map.insert("aliran_data".to_string(), Keyword::DataFlow);
    map.insert("tolak".to_string(), Keyword::Deny);
    map.insert("pelaksana".to_string(), Keyword::Executor);
    map.insert("resmi".to_string(), Keyword::Formal);
    map.insert("fungsi".to_string(), Keyword::Function);
    map.insert("tidak_resmi".to_string(), Keyword::Informal);
    map.insert("niat".to_string(), Keyword::Intent);
    map.insert("prioritas".to_string(), Keyword::Priority);
    map.insert("rantai_aturan".to_string(), Keyword::RuleChain);
    map.insert("set_aturan".to_string(), Keyword::RuleSet);
    map.insert("mesin_keadaan".to_string(), Keyword::StateMachine);
    map.insert("teknis".to_string(), Keyword::Technical);
    map.insert("transformasi".to_string(), Keyword::Transform);
    map.insert("variabel_var".to_string(), Keyword::Var);
    map.insert("ingin".to_string(), Keyword::Want);

    map
}
