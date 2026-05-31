//! Bengali keyword mappings
//! বাংলা - Bengali language support

use crate::keyword_map::Keyword;
use std::collections::HashMap;

pub fn get_keywords() -> HashMap<String, Keyword> {
    let mut map = HashMap::new();

    // Core keywords
    map.insert("ধরো".to_string(), Keyword::Let);
    map.insert("চলক".to_string(), Keyword::Let);
    map.insert("ডেটা".to_string(), Keyword::Data);
    map.insert("সেট_করো".to_string(), Keyword::Set);
    map.insert("সত্য".to_string(), Keyword::True);
    map.insert("মিথ্যা".to_string(), Keyword::False);

    // Control flow
    map.insert("যদি".to_string(), Keyword::If);
    map.insert("নাহলে".to_string(), Keyword::Else);
    map.insert("যতক্ষণ".to_string(), Keyword::While);
    map.insert("জন্য".to_string(), Keyword::For);
    map.insert("ফেরত_দাও".to_string(), Keyword::Return);
    map.insert("ভাঙো".to_string(), Keyword::Break);
    map.insert("চালিয়ে_যাও".to_string(), Keyword::Continue);
    map.insert("সুইচ".to_string(), Keyword::Switch);
    map.insert("কেস".to_string(), Keyword::Case);
    map.insert("ডিফল্ট".to_string(), Keyword::Default);

    // Module system
    map.insert("ব্যবহার_করো".to_string(), Keyword::Use);
    map.insert("হিসেবে".to_string(), Keyword::As);
    map.insert("আমদানি_করো".to_string(), Keyword::Import);
    map.insert("রপ্তানি_করো".to_string(), Keyword::Export);
    map.insert("থেকে".to_string(), Keyword::From);

    // Agent system
    map.insert("এজেন্ট".to_string(), Keyword::Agent);
    map.insert("কাজ".to_string(), Keyword::Task);
    map.insert("টুল".to_string(), Keyword::Tool);
    map.insert("সম্পদ".to_string(), Keyword::Resource);
    map.insert("এমসিপি".to_string(), Keyword::Mcp);
    map.insert("সার্ভার".to_string(), Keyword::Server);
    map.insert("কনফিগ".to_string(), Keyword::Config);
    map.insert("প্রদানকারী".to_string(), Keyword::Provider);
    map.insert("মডেল".to_string(), Keyword::Model);
    map.insert("কল_করো".to_string(), Keyword::Call);

    // Governance
    map.insert("সংবিধান".to_string(), Keyword::Constitution);
    map.insert("আইন".to_string(), Keyword::Law);
    map.insert("নিয়ম".to_string(), Keyword::Rule);
    map.insert("পরিষদ".to_string(), Keyword::Council);
    map.insert("ঝাঁক".to_string(), Keyword::Swarm);
    map.insert("সম্প্রদায়".to_string(), Keyword::Community);
    map.insert("প্রয়োগ".to_string(), Keyword::Enforcement);
    map.insert("বাধ্যতামূলক".to_string(), Keyword::Mandatory);
    map.insert("পরামর্শমূলক".to_string(), Keyword::Advisory);
    map.insert("ঐচ্ছিক".to_string(), Keyword::Optional);
    map.insert("ভূমিকা".to_string(), Keyword::Role);
    map.insert("সদস্য".to_string(), Keyword::Member);
    map.insert("কৌশল".to_string(), Keyword::Strategy);
    map.insert("প্রতিযোগিতামূলক".to_string(), Keyword::Competitive);
    map.insert("সহযোগিতামূলক".to_string(), Keyword::Collaborative);
    map.insert("সংস্কৃতি".to_string(), Keyword::Culture);
    map.insert("মূল্যবোধ".to_string(), Keyword::Values);
    map.insert("নিয়মকানুন".to_string(), Keyword::Norms);
    map.insert("প্রসিকিউটর".to_string(), Keyword::Prosecutor);
    map.insert("বিচারক".to_string(), Keyword::Judge);

    // Async & Error handling
    map.insert("অ্যাসিঙ্ক".to_string(), Keyword::Async);
    map.insert("অপেক্ষা_করো".to_string(), Keyword::Await);
    map.insert("চেষ্টা_করো".to_string(), Keyword::Try);
    map.insert("ধরে_ফেলো".to_string(), Keyword::Catch);
    map.insert("শেষে".to_string(), Keyword::Finally);
    map.insert("ছুঁড়ে_দাও".to_string(), Keyword::Throw);
    map.insert("প্রতিশ্রুতি".to_string(), Keyword::Promise);
    map.insert("ভবিষ্যৎ".to_string(), Keyword::Future);

    // Data structures
    map.insert("নোট".to_string(), Keyword::Note);
    map.insert("কর্ড".to_string(), Keyword::Chord);
    map.insert("সুর".to_string(), Keyword::Melody);
    map.insert("সামঞ্জস্য".to_string(), Keyword::Harmony);
    map.insert("ছন্দ".to_string(), Keyword::Rhythm);
    map.insert("গতি".to_string(), Keyword::Tempo);
    map.insert("স্কেল".to_string(), Keyword::Scale);

    // Intent & State
    map.insert("সত্তা".to_string(), Keyword::Entity);
    map.insert("অবস্থা".to_string(), Keyword::State);
    map.insert("ঘটনা".to_string(), Keyword::Event);

    // Flow & Orchestration
    map.insert("প্রবাহ".to_string(), Keyword::Flow);
    map.insert("স্তর".to_string(), Keyword::Layer);
    map.insert("নেটওয়ার্ক".to_string(), Keyword::Network);
    map.insert("নির্ভর_করে".to_string(), Keyword::DependsOn);
    map.insert("সম্প্রচার".to_string(), Keyword::Broadcast);
    map.insert("মার্জ_করো".to_string(), Keyword::Merge);
    map.insert("সমান্তরাল".to_string(), Keyword::Parallel);
    map.insert("ক্রমিক".to_string(), Keyword::Sequential);
    map.insert("চালাও".to_string(), Keyword::Execute);
    map.insert("যখন".to_string(), Keyword::When);
    map.insert("উপর".to_string(), Keyword::On);
    map.insert("ট্রিগার".to_string(), Keyword::Trigger);
    // Native Bengali translations
    map.insert("কার্য".to_string(), Keyword::Action);
    map.insert("এজেন্ট_নিয়ম".to_string(), Keyword::AgentRule);
    map.insert("এজেন্ট_অবস্থা".to_string(), Keyword::AgentState);
    map.insert("অনুমতি_দাও".to_string(), Keyword::Allow);
    map.insert("যোগাযোগ_শৈলী".to_string(), Keyword::CommunicationStyle);
    map.insert("ধ্রুবক".to_string(), Keyword::Const);
    map.insert("ডেটা_প্রবাহ".to_string(), Keyword::DataFlow);
    map.insert("অস্বীকার_করো".to_string(), Keyword::Deny);
    map.insert("নির্বাহক".to_string(), Keyword::Executor);
    map.insert("আনুষ্ঠানিক".to_string(), Keyword::Formal);
    map.insert("ফাংশন".to_string(), Keyword::Function);
    map.insert("অনানুষ্ঠানিক".to_string(), Keyword::Informal);
    map.insert("উদ্দেশ্য".to_string(), Keyword::Intent);
    map.insert("অগ্রাধিকার".to_string(), Keyword::Priority);
    map.insert("নিয়ম_শৃঙ্খল".to_string(), Keyword::RuleChain);
    map.insert("নিয়ম_সেট".to_string(), Keyword::RuleSet);
    map.insert("অবস্থা_যন্ত্র".to_string(), Keyword::StateMachine);
    map.insert("প্রযুক্তিগত".to_string(), Keyword::Technical);
    map.insert("রূপান্তর_করো".to_string(), Keyword::Transform);
    map.insert("চলক_ভেরিয়েবল".to_string(), Keyword::Var);
    map.insert("চাওয়া".to_string(), Keyword::Want);

    map
}
