//! Korean keyword mappings
//! 한국어 - Korean language support (SOV)

use crate::keyword_map::Keyword;
use std::collections::HashMap;

pub fn get_keywords() -> HashMap<String, Keyword> {
    let mut map = HashMap::new();

    // Variables & Data
    map.insert("변수".to_string(), Keyword::Let);
    map.insert("상수".to_string(), Keyword::Const);
    map.insert("데이터".to_string(), Keyword::Data);
    map.insert("설정".to_string(), Keyword::Set);

    // Control flow
    map.insert("만약".to_string(), Keyword::If);
    map.insert("아니면".to_string(), Keyword::Else);
    map.insert("동안".to_string(), Keyword::While);
    map.insert("반복".to_string(), Keyword::For);
    map.insert("반환".to_string(), Keyword::Return);
    map.insert("중단".to_string(), Keyword::Break);
    map.insert("계속".to_string(), Keyword::Continue);
    map.insert("선택".to_string(), Keyword::Switch);
    map.insert("경우".to_string(), Keyword::Case);
    map.insert("기본".to_string(), Keyword::Default);

    // Functions & OOP
    map.insert("함수".to_string(), Keyword::Function);
    map.insert("클래스".to_string(), Keyword::Class);
    map.insert("새".to_string(), Keyword::New);
    map.insert("이것".to_string(), Keyword::This);

    // Booleans (Null is handled by the keyword normalizer as a literal)
    map.insert("참".to_string(), Keyword::True);
    map.insert("거짓".to_string(), Keyword::False);

    // Async
    map.insert("비동기".to_string(), Keyword::Async);
    map.insert("기다리기".to_string(), Keyword::Await);

    // Error handling
    map.insert("시도".to_string(), Keyword::Try);
    map.insert("잡기".to_string(), Keyword::Catch);
    map.insert("마지막".to_string(), Keyword::Finally);
    map.insert("던지기".to_string(), Keyword::Throw);

    // Module system
    map.insert("사용".to_string(), Keyword::Use);
    map.insert("가져오기".to_string(), Keyword::Import);
    map.insert("내보내기".to_string(), Keyword::Export);
    map.insert("에서".to_string(), Keyword::From);

    // Agent system
    map.insert("에이전트".to_string(), Keyword::Agent);
    map.insert("도구".to_string(), Keyword::Tool);
    map.insert("자원".to_string(), Keyword::Resource);

    map
}
