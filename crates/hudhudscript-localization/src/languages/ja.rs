//! Japanese keyword mappings — 日本語
//! Kural 8: Romanize YASAK. Tüm keyword'ler Kanji/Hiragana/Katakana ile yazılır.

use crate::keyword_map::Keyword;
use std::collections::HashMap;

pub fn get_keywords() -> HashMap<String, Keyword> {
    let mut map = HashMap::new();

    // モジュール (Module system)
    map.insert("使う".to_string(), Keyword::Use);
    map.insert("として".to_string(), Keyword::As);
    map.insert("輸入".to_string(), Keyword::Import);
    map.insert("輸出".to_string(), Keyword::Export);
    map.insert("から".to_string(), Keyword::From);

    // エージェント (Agent system)
    map.insert("エージェント".to_string(), Keyword::Agent);
    map.insert("ツール".to_string(), Keyword::Tool);
    map.insert("資源".to_string(), Keyword::Resource);
    map.insert("mcp".to_string(), Keyword::Mcp);
    map.insert("サーバー".to_string(), Keyword::Server);
    map.insert("設定".to_string(), Keyword::Config);

    // 制御構文 (Control flow)
    map.insert("もし".to_string(), Keyword::If);
    map.insert("でなければ".to_string(), Keyword::Else);
    map.insert("間".to_string(), Keyword::While);
    map.insert("ため".to_string(), Keyword::For);
    map.insert("戻る".to_string(), Keyword::Return);
    map.insert("破る".to_string(), Keyword::Break);
    map.insert("続ける".to_string(), Keyword::Continue);

    // 非同期 (Async)
    map.insert("非同期".to_string(), Keyword::Async);
    map.insert("待つ".to_string(), Keyword::Await);

    // 変数 (Variables)
    map.insert("変数".to_string(), Keyword::Let);
    map.insert("変数".to_string(), Keyword::Var);
    map.insert("定数".to_string(), Keyword::Const);
    map.insert("データ".to_string(), Keyword::Data);
    map.insert("設定値".to_string(), Keyword::Set);

    // 関数・クラス (Functions & Classes)
    map.insert("関数".to_string(), Keyword::Function);
    map.insert("クラス".to_string(), Keyword::Class);
    map.insert("新しい".to_string(), Keyword::New);
    map.insert("これ".to_string(), Keyword::This);

    // 真偽値 (Booleans)
    map.insert("真".to_string(), Keyword::True);
    map.insert("偽".to_string(), Keyword::False);
    // Null is handled as a literal, not a keyword

    // 例外処理 (Exception handling)
    map.insert("試みる".to_string(), Keyword::Try);
    map.insert("捕まえる".to_string(), Keyword::Catch);
    map.insert("投げる".to_string(), Keyword::Throw);
    map.insert("最後".to_string(), Keyword::Finally);

    // 切り替え (Switch)
    map.insert("切り替える".to_string(), Keyword::Switch);
    map.insert("場合".to_string(), Keyword::Case);
    map.insert("基本".to_string(), Keyword::Default);

    // 意図 (Intent)
    map.insert("実体".to_string(), Keyword::Entity);
    map.insert("状態".to_string(), Keyword::State);
    map.insert("イベント".to_string(), Keyword::Event);
    map.insert("アクション".to_string(), Keyword::Action);
    map.insert("変換".to_string(), Keyword::Transform);

    // 統治 (Governance)
    map.insert("憲法".to_string(), Keyword::Constitution);
    map.insert("法律".to_string(), Keyword::Law);
    map.insert("規則".to_string(), Keyword::Rule);
    map.insert("会議".to_string(), Keyword::Council);
    map.insert("群れ".to_string(), Keyword::Swarm);
    map.insert("共同体".to_string(), Keyword::Community);
    map.insert("執行".to_string(), Keyword::Enforcement);
    map.insert("必須".to_string(), Keyword::Mandatory);
    map.insert("助言".to_string(), Keyword::Advisory);
    map.insert("任意".to_string(), Keyword::Optional);
    map.insert("役割".to_string(), Keyword::Role);
    map.insert("メンバー".to_string(), Keyword::Member);
    map.insert("戦略".to_string(), Keyword::Strategy);
    map.insert("競争".to_string(), Keyword::Competitive);
    map.insert("協同".to_string(), Keyword::Collaborative);
    map.insert("文化".to_string(), Keyword::Culture);
    map.insert("価値".to_string(), Keyword::Values);
    map.insert("規範".to_string(), Keyword::Norms);
    map.insert("公式".to_string(), Keyword::Formal);
    map.insert("非公式".to_string(), Keyword::Informal);
    map.insert("技術".to_string(), Keyword::Technical);

    // 意図 (Intent & Goals)
    map.insert("意図".to_string(), Keyword::Intent);
    map.insert("欲しい".to_string(), Keyword::Want);
    map.insert("優先度".to_string(), Keyword::Priority);

    // フロー (Flow)
    map.insert("フロー".to_string(), Keyword::Flow);
    map.insert("データフロー".to_string(), Keyword::DataFlow);

    // 役割 (Agent Roles)
    map.insert("検察官".to_string(), Keyword::Prosecutor);
    map.insert("裁判官".to_string(), Keyword::Judge);
    map.insert("実行者".to_string(), Keyword::Executor);

    // 音楽 (Music)
    map.insert("音符".to_string(), Keyword::Note);
    map.insert("和音".to_string(), Keyword::Chord);
    map.insert("旋律".to_string(), Keyword::Melody);
    map.insert("ハーモニー".to_string(), Keyword::Harmony);
    map.insert("リズム".to_string(), Keyword::Rhythm);
    map.insert("テンポ".to_string(), Keyword::Tempo);
    map.insert("音階".to_string(), Keyword::Scale);

    // 制約 (Constraints)
    map.insert("時".to_string(), Keyword::When);
    map.insert("上".to_string(), Keyword::On);
    map.insert("トリガー".to_string(), Keyword::Trigger);

    // 権限 (Permissions)
    map.insert("許可".to_string(), Keyword::Allow);
    map.insert("禁止".to_string(), Keyword::Deny);

    // オーケストレーション (Orchestration)
    map.insert("層".to_string(), Keyword::Layer);
    map.insert("ネットワーク".to_string(), Keyword::Network);
    map.insert("依存".to_string(), Keyword::DependsOn);
    map.insert("放送".to_string(), Keyword::Broadcast);
    map.insert("合併".to_string(), Keyword::Merge);
    map.insert("並行".to_string(), Keyword::Parallel);
    map.insert("順序".to_string(), Keyword::Sequential);
    map.insert("実行".to_string(), Keyword::Execute);

    // その他
    map.insert("ステートマシン".to_string(), Keyword::StateMachine);
    map.insert("プロミス".to_string(), Keyword::Promise);
    map.insert("プロバイダ".to_string(), Keyword::Provider);
    map.insert("モデル".to_string(), Keyword::Model);
    map.insert("呼ぶ".to_string(), Keyword::Call);

    map
}
