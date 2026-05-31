//! Persian/Farsi keyword mappings
//! فارسی - Persian language support

use crate::keyword_map::Keyword;
use std::collections::HashMap;

pub fn get_keywords() -> HashMap<String, Keyword> {
    let mut map = HashMap::new();

    // Core keywords
    map.insert("بگذار".to_string(), Keyword::Let);
    map.insert("متغیر".to_string(), Keyword::Let);
    map.insert("داده".to_string(), Keyword::Data);
    map.insert("تنظیم".to_string(), Keyword::Set);
    map.insert("درست".to_string(), Keyword::True);
    map.insert("نادرست".to_string(), Keyword::False);

    // Control flow
    map.insert("اگر".to_string(), Keyword::If);
    map.insert("وگرنه".to_string(), Keyword::Else);
    map.insert("تا_وقتی".to_string(), Keyword::While);
    map.insert("برای".to_string(), Keyword::For);
    map.insert("برگردان".to_string(), Keyword::Return);
    map.insert("بشکن".to_string(), Keyword::Break);
    map.insert("ادامه".to_string(), Keyword::Continue);
    map.insert("تعویض".to_string(), Keyword::Switch);
    map.insert("حالت".to_string(), Keyword::Case);
    map.insert("پیش_فرض".to_string(), Keyword::Default);

    // Module system
    map.insert("استفاده".to_string(), Keyword::Use);
    map.insert("به_عنوان".to_string(), Keyword::As);
    map.insert("وارد_کن".to_string(), Keyword::Import);
    map.insert("صادر_کن".to_string(), Keyword::Export);
    map.insert("از".to_string(), Keyword::From);

    // Agent system
    map.insert("عامل".to_string(), Keyword::Agent);
    map.insert("وظیفه".to_string(), Keyword::Task);
    map.insert("ابزار".to_string(), Keyword::Tool);
    map.insert("منبع".to_string(), Keyword::Resource);
    map.insert("ام_سی_پی".to_string(), Keyword::Mcp);
    map.insert("سرور".to_string(), Keyword::Server);
    map.insert("پیکربندی".to_string(), Keyword::Config);
    map.insert("ارائه_دهنده".to_string(), Keyword::Provider);
    map.insert("مدل".to_string(), Keyword::Model);
    map.insert("فراخوانی".to_string(), Keyword::Call);

    // Governance
    map.insert("قانون_اساسی".to_string(), Keyword::Constitution);
    map.insert("قانون".to_string(), Keyword::Law);
    map.insert("قاعده".to_string(), Keyword::Rule);
    map.insert("شورا".to_string(), Keyword::Council);
    map.insert("ازدحام".to_string(), Keyword::Swarm);
    map.insert("جامعه".to_string(), Keyword::Community);
    map.insert("اجرا".to_string(), Keyword::Enforcement);
    map.insert("اجباری".to_string(), Keyword::Mandatory);
    map.insert("مشاوره_ای".to_string(), Keyword::Advisory);
    map.insert("اختیاری".to_string(), Keyword::Optional);
    map.insert("نقش".to_string(), Keyword::Role);
    map.insert("عضو".to_string(), Keyword::Member);
    map.insert("استراتژی".to_string(), Keyword::Strategy);
    map.insert("رقابتی".to_string(), Keyword::Competitive);
    map.insert("همکاری".to_string(), Keyword::Collaborative);
    map.insert("فرهنگ".to_string(), Keyword::Culture);
    map.insert("ارزش_ها".to_string(), Keyword::Values);
    map.insert("هنجارها".to_string(), Keyword::Norms);
    map.insert("دادستان".to_string(), Keyword::Prosecutor);
    map.insert("قاضی".to_string(), Keyword::Judge);

    // Async & Error handling
    map.insert("ناهمزمان".to_string(), Keyword::Async);
    map.insert("منتظر_باش".to_string(), Keyword::Await);
    map.insert("امتحان_کن".to_string(), Keyword::Try);
    map.insert("بگیر".to_string(), Keyword::Catch);
    map.insert("نهایتا".to_string(), Keyword::Finally);
    map.insert("پرتاب".to_string(), Keyword::Throw);
    map.insert("وعده".to_string(), Keyword::Promise);
    map.insert("آینده".to_string(), Keyword::Future);

    // Data structures
    map.insert("نت".to_string(), Keyword::Note);
    map.insert("آکورد".to_string(), Keyword::Chord);
    map.insert("ملودی".to_string(), Keyword::Melody);
    map.insert("هارمونی".to_string(), Keyword::Harmony);
    map.insert("ریتم".to_string(), Keyword::Rhythm);
    map.insert("تمپو".to_string(), Keyword::Tempo);
    map.insert("مقیاس".to_string(), Keyword::Scale);

    // Intent & State
    map.insert("موجودیت".to_string(), Keyword::Entity);
    map.insert("وضعیت".to_string(), Keyword::State);
    map.insert("رویداد".to_string(), Keyword::Event);

    // Flow & Orchestration
    map.insert("جریان".to_string(), Keyword::Flow);
    map.insert("لایه".to_string(), Keyword::Layer);
    map.insert("شبکه".to_string(), Keyword::Network);
    map.insert("وابسته_به".to_string(), Keyword::DependsOn);
    map.insert("پخش".to_string(), Keyword::Broadcast);
    map.insert("ادغام".to_string(), Keyword::Merge);
    map.insert("موازی".to_string(), Keyword::Parallel);
    map.insert("ترتیبی".to_string(), Keyword::Sequential);
    map.insert("اجرا_کن".to_string(), Keyword::Execute);
    map.insert("وقتی".to_string(), Keyword::When);
    map.insert("روی".to_string(), Keyword::On);
    map.insert("ماشه".to_string(), Keyword::Trigger);
    // Native Persian/Farsi translations
    map.insert("عمل".to_string(), Keyword::Action);
    map.insert("قانون_عامل".to_string(), Keyword::AgentRule);
    map.insert("وضعیت_عامل".to_string(), Keyword::AgentState);
    map.insert("اجازه_بده".to_string(), Keyword::Allow);
    map.insert("سبک_ارتباطی".to_string(), Keyword::CommunicationStyle);
    map.insert("ثابت".to_string(), Keyword::Const);
    map.insert("جریان_داده".to_string(), Keyword::DataFlow);
    map.insert("رد_کن".to_string(), Keyword::Deny);
    map.insert("مجری".to_string(), Keyword::Executor);
    map.insert("رسمی".to_string(), Keyword::Formal);
    map.insert("تابع".to_string(), Keyword::Function);
    map.insert("غیررسمی".to_string(), Keyword::Informal);
    map.insert("هدف".to_string(), Keyword::Intent);
    map.insert("اولویت".to_string(), Keyword::Priority);
    map.insert("زنجیره_قوانین".to_string(), Keyword::RuleChain);
    map.insert("مجموعه_قوانین".to_string(), Keyword::RuleSet);
    map.insert("ماشین_حالت".to_string(), Keyword::StateMachine);
    map.insert("فنی".to_string(), Keyword::Technical);
    map.insert("تبدیل_کن".to_string(), Keyword::Transform);
    map.insert("متغیر_دیگر".to_string(), Keyword::Var);
    map.insert("خواستن".to_string(), Keyword::Want);

    map
}
