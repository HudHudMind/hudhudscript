//! Chinese keyword mappings
//! 中文 - Chinese language support (SVO, topic-prominent)

use crate::keyword_map::Keyword;
use std::collections::HashMap;

pub fn get_keywords() -> HashMap<String, Keyword> {
    let mut map = HashMap::new();

    // Module system
    map.insert("使用".to_string(), Keyword::Use);
    map.insert("作为".to_string(), Keyword::As);
    map.insert("导入".to_string(), Keyword::Import);
    map.insert("导出".to_string(), Keyword::Export);
    map.insert("从".to_string(), Keyword::From);

    // Agent system
    map.insert("代理".to_string(), Keyword::Agent);
    map.insert("任务".to_string(), Keyword::Task);
    map.insert("工具".to_string(), Keyword::Tool);
    map.insert("资源".to_string(), Keyword::Resource);
    map.insert("mcp".to_string(), Keyword::Mcp);
    map.insert("服务器".to_string(), Keyword::Server);
    map.insert("配置".to_string(), Keyword::Config);

    // Control flow
    map.insert("如果".to_string(), Keyword::If);
    map.insert("否则".to_string(), Keyword::Else);
    map.insert("当".to_string(), Keyword::While);
    map.insert("对于".to_string(), Keyword::For);
    map.insert("返回".to_string(), Keyword::Return);
    map.insert("中断".to_string(), Keyword::Break);
    map.insert("继续".to_string(), Keyword::Continue);
    map.insert("选择".to_string(), Keyword::Switch);
    map.insert("情况".to_string(), Keyword::Case);
    map.insert("默认".to_string(), Keyword::Default);

    // Error handling
    map.insert("尝试".to_string(), Keyword::Try);
    map.insert("捕获".to_string(), Keyword::Catch);
    map.insert("最后".to_string(), Keyword::Finally);
    map.insert("抛出".to_string(), Keyword::Throw);

    // Async
    map.insert("异步".to_string(), Keyword::Async);
    map.insert("等待".to_string(), Keyword::Await);

    // Data & Variables
    map.insert("变量".to_string(), Keyword::Let);
    map.insert("数据".to_string(), Keyword::Data);
    map.insert("值".to_string(), Keyword::Set);

    // Governance
    map.insert("宪法".to_string(), Keyword::Constitution);
    map.insert("法律".to_string(), Keyword::Law);
    map.insert("规则".to_string(), Keyword::Rule);
    map.insert("议会".to_string(), Keyword::Council);
    map.insert("群".to_string(), Keyword::Swarm);
    map.insert("社区".to_string(), Keyword::Community);
    map.insert("强制".to_string(), Keyword::Mandatory);
    map.insert("建议".to_string(), Keyword::Advisory);
    map.insert("可选".to_string(), Keyword::Optional);
    map.insert("角色".to_string(), Keyword::Role);
    map.insert("成员".to_string(), Keyword::Member);
    map.insert("策略".to_string(), Keyword::Strategy);
    map.insert("竞争".to_string(), Keyword::Competitive);
    map.insert("协作".to_string(), Keyword::Collaborative);
    map.insert("并行".to_string(), Keyword::Parallel);
    map.insert("顺序".to_string(), Keyword::Sequential);
    map.insert("执行".to_string(), Keyword::Execute);
    // Native Chinese translations for governance/intent/flow keywords
    map.insert("动作".to_string(), Keyword::Action);
    map.insert("代理规则".to_string(), Keyword::AgentRule);
    map.insert("代理状态".to_string(), Keyword::AgentState);
    map.insert("允许".to_string(), Keyword::Allow);
    map.insert("广播".to_string(), Keyword::Broadcast);
    map.insert("调用".to_string(), Keyword::Call);
    map.insert("和弦".to_string(), Keyword::Chord);
    map.insert("通信风格".to_string(), Keyword::CommunicationStyle);
    map.insert("常量".to_string(), Keyword::Const);
    map.insert("文化".to_string(), Keyword::Culture);
    map.insert("数据流".to_string(), Keyword::DataFlow);
    map.insert("拒绝".to_string(), Keyword::Deny);
    map.insert("依赖于".to_string(), Keyword::DependsOn);
    map.insert("执法".to_string(), Keyword::Enforcement);
    map.insert("实体".to_string(), Keyword::Entity);
    map.insert("事件".to_string(), Keyword::Event);
    map.insert("执行者".to_string(), Keyword::Executor);
    map.insert("流程".to_string(), Keyword::Flow);
    map.insert("正式".to_string(), Keyword::Formal);
    map.insert("函数".to_string(), Keyword::Function);
    map.insert("未来".to_string(), Keyword::Future);
    map.insert("和声".to_string(), Keyword::Harmony);
    map.insert("非正式".to_string(), Keyword::Informal);
    map.insert("意图".to_string(), Keyword::Intent);
    map.insert("法官".to_string(), Keyword::Judge);
    map.insert("层次".to_string(), Keyword::Layer);
    map.insert("旋律".to_string(), Keyword::Melody);
    map.insert("合并".to_string(), Keyword::Merge);
    map.insert("模型".to_string(), Keyword::Model);
    map.insert("网络".to_string(), Keyword::Network);
    map.insert("规范".to_string(), Keyword::Norms);
    map.insert("音符".to_string(), Keyword::Note);
    map.insert("在".to_string(), Keyword::On);
    map.insert("优先级".to_string(), Keyword::Priority);
    map.insert("承诺".to_string(), Keyword::Promise);
    map.insert("检察官".to_string(), Keyword::Prosecutor);
    map.insert("提供者".to_string(), Keyword::Provider);
    map.insert("节奏".to_string(), Keyword::Rhythm);
    map.insert("规则链".to_string(), Keyword::RuleChain);
    map.insert("规则集".to_string(), Keyword::RuleSet);
    map.insert("音阶".to_string(), Keyword::Scale);
    map.insert("状态".to_string(), Keyword::State);
    map.insert("状态机".to_string(), Keyword::StateMachine);
    map.insert("技术".to_string(), Keyword::Technical);
    map.insert("节拍".to_string(), Keyword::Tempo);
    map.insert("转换".to_string(), Keyword::Transform);
    map.insert("触发器".to_string(), Keyword::Trigger);
    map.insert("价值观".to_string(), Keyword::Values);
    map.insert("变量".to_string(), Keyword::Var);
    map.insert("想要".to_string(), Keyword::Want);
    map.insert("当".to_string(), Keyword::When);

    map
}
