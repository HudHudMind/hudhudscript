//! Arabic keyword mappings
//! العربية - Arabic language support (VSO - Verb-Subject-Object, RTL)

use crate::keyword_map::Keyword;
use std::collections::HashMap;

pub fn get_keywords() -> HashMap<String, Keyword> {
    let mut map = HashMap::new();

    // Module system
    map.insert("استخدم".to_string(), Keyword::Use);
    map.insert("كـ".to_string(), Keyword::As);
    map.insert("استورد".to_string(), Keyword::Import);
    map.insert("صدّر".to_string(), Keyword::Export);
    map.insert("من".to_string(), Keyword::From);

    // Agent system
    map.insert("وكيل".to_string(), Keyword::Agent);
    map.insert("أداة".to_string(), Keyword::Tool);
    map.insert("مورد".to_string(), Keyword::Resource);
    map.insert("mcp".to_string(), Keyword::Mcp);
    map.insert("خادم".to_string(), Keyword::Server);
    map.insert("إعداد".to_string(), Keyword::Config);

    // Control flow
    map.insert("إذا".to_string(), Keyword::If);
    map.insert("وإلا".to_string(), Keyword::Else);
    map.insert("بينما".to_string(), Keyword::While);
    map.insert("لكل".to_string(), Keyword::For);
    map.insert("ارجع".to_string(), Keyword::Return);
    map.insert("اكسر".to_string(), Keyword::Break);
    map.insert("استمر".to_string(), Keyword::Continue);
    map.insert("اختر".to_string(), Keyword::Switch);
    map.insert("تبديل".to_string(), Keyword::Switch); // Alternative
    map.insert("حالة".to_string(), Keyword::Case);
    map.insert("افتراضي".to_string(), Keyword::Default);

    // Functions
    map.insert("دالة".to_string(), Keyword::Function);
    map.insert("د".to_string(), Keyword::Function);

    // Error handling
    map.insert("حاول".to_string(), Keyword::Try);
    map.insert("امسك".to_string(), Keyword::Catch);
    map.insert("اصطد".to_string(), Keyword::Catch); // Alternative
    map.insert("أخيرا".to_string(), Keyword::Finally);
    map.insert("ارمي".to_string(), Keyword::Throw);

    // Async
    map.insert("غير_متزامن".to_string(), Keyword::Async);
    map.insert("انتظر".to_string(), Keyword::Await);

    // Data & Variables
    map.insert("متغير".to_string(), Keyword::Let);
    map.insert("بيانات".to_string(), Keyword::Data);
    map.insert("قيمة".to_string(), Keyword::Set);

    // Governance
    map.insert("دستور".to_string(), Keyword::Constitution);
    map.insert("قانون".to_string(), Keyword::Law);
    map.insert("قاعدة".to_string(), Keyword::Rule);
    map.insert("مجلس".to_string(), Keyword::Council);
    map.insert("سرب".to_string(), Keyword::Swarm);
    map.insert("مجتمع".to_string(), Keyword::Community);
    map.insert("تنفيذ".to_string(), Keyword::Enforcement);
    map.insert("إلزامي".to_string(), Keyword::Mandatory);
    map.insert("استشاري".to_string(), Keyword::Advisory);
    map.insert("اختياري".to_string(), Keyword::Optional);
    map.insert("دور".to_string(), Keyword::Role);
    map.insert("عضو".to_string(), Keyword::Member);
    map.insert("استراتيجية".to_string(), Keyword::Strategy);
    map.insert("تنافسي".to_string(), Keyword::Competitive);
    map.insert("تعاوني".to_string(), Keyword::Collaborative);
    map.insert("متوازي".to_string(), Keyword::Parallel);
    map.insert("تسلسلي".to_string(), Keyword::Sequential);
    map.insert("نفذ".to_string(), Keyword::Execute);
    map.insert("مزود".to_string(), Keyword::Provider);

    // Agent System (additional)
    map.insert("نموذج".to_string(), Keyword::Model);
    map.insert("استدعاء".to_string(), Keyword::Call);

    // Async (additional)
    map.insert("وعد".to_string(), Keyword::Promise);
    map.insert("مستقبل".to_string(), Keyword::Future);

    // Intent & Entity
    map.insert("نية".to_string(), Keyword::Intent);
    map.insert("يريد".to_string(), Keyword::Want);
    map.insert("أولوية".to_string(), Keyword::Priority);
    map.insert("إجراء".to_string(), Keyword::Action);
    map.insert("تحويل".to_string(), Keyword::Transform);

    // Events (additional)
    map.insert("متى".to_string(), Keyword::When);

    // Permissions
    map.insert("اسمح".to_string(), Keyword::Allow);
    map.insert("امنع".to_string(), Keyword::Deny);

    // Governance (additional)

    // Roles (additional)
    map.insert("مدعي".to_string(), Keyword::Prosecutor);
    map.insert("قاضي".to_string(), Keyword::Judge);
    map.insert("منفذ".to_string(), Keyword::Executor);

    // Culture
    map.insert("ثقافة".to_string(), Keyword::Culture);
    map.insert("قيم".to_string(), Keyword::Values);
    map.insert("معايير".to_string(), Keyword::Norms);
    map.insert("أسلوب_التواصل".to_string(), Keyword::CommunicationStyle);
    map.insert("رسمي".to_string(), Keyword::Formal);
    map.insert("غير_رسمي".to_string(), Keyword::Informal);
    map.insert("تقني".to_string(), Keyword::Technical);

    // Flow & Orchestration (additional)
    map.insert("تدفق".to_string(), Keyword::Flow);
    map.insert("تدفق_البيانات".to_string(), Keyword::DataFlow);
    map.insert("طبقة".to_string(), Keyword::Layer);
    map.insert("شبكة".to_string(), Keyword::Network);
    map.insert("يعتمد_على".to_string(), Keyword::DependsOn);
    map.insert("بث".to_string(), Keyword::Broadcast);
    map.insert("دمج".to_string(), Keyword::Merge);

    // Music Data Structures
    map.insert("نوتة".to_string(), Keyword::Note);
    map.insert("وتر".to_string(), Keyword::Chord);
    map.insert("لحن".to_string(), Keyword::Melody);
    map.insert("انسجام".to_string(), Keyword::Harmony);
    map.insert("إيقاع".to_string(), Keyword::Rhythm);
    map.insert("سرعة".to_string(), Keyword::Tempo);
    map.insert("سلم".to_string(), Keyword::Scale);

    map
}
