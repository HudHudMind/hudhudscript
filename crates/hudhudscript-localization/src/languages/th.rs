//! Thai keyword mappings
//! ภาษาไทย - Thai language support

use crate::keyword_map::Keyword;
use std::collections::HashMap;

pub fn get_keywords() -> HashMap<String, Keyword> {
    let mut map = HashMap::new();

    // Core keywords
    map.insert("ให้".to_string(), Keyword::Let);
    map.insert("ตัวแปร".to_string(), Keyword::Let);
    map.insert("ข้อมูล".to_string(), Keyword::Data);
    map.insert("ตั้ง".to_string(), Keyword::Set);
    map.insert("จริง".to_string(), Keyword::True);
    map.insert("เท็จ".to_string(), Keyword::False);

    // Control flow
    map.insert("ถ้า".to_string(), Keyword::If);
    map.insert("มิฉะนั้น".to_string(), Keyword::Else);
    map.insert("ขณะที่".to_string(), Keyword::While);
    map.insert("สำหรับ".to_string(), Keyword::For);
    map.insert("คืนค่า".to_string(), Keyword::Return);
    map.insert("หยุด".to_string(), Keyword::Break);
    map.insert("ดำเนินการต่อ".to_string(), Keyword::Continue);
    map.insert("สลับ".to_string(), Keyword::Switch);
    map.insert("กรณี".to_string(), Keyword::Case);
    map.insert("ค่าเริ่มต้น".to_string(), Keyword::Default);

    // Module system
    map.insert("ใช้".to_string(), Keyword::Use);
    map.insert("เป็น".to_string(), Keyword::As);
    map.insert("นำเข้า".to_string(), Keyword::Import);
    map.insert("ส่งออก".to_string(), Keyword::Export);
    map.insert("จาก".to_string(), Keyword::From);

    // Agent system
    map.insert("เอเจนต์".to_string(), Keyword::Agent);
    map.insert("งาน".to_string(), Keyword::Task);
    map.insert("เครื่องมือ".to_string(), Keyword::Tool);
    map.insert("ทรัพยากร".to_string(), Keyword::Resource);
    map.insert("เอ็มซีพี".to_string(), Keyword::Mcp);
    map.insert("เซิร์ฟเวอร์".to_string(), Keyword::Server);
    map.insert("การกำหนดค่า".to_string(), Keyword::Config);
    map.insert("ผู้ให้บริการ".to_string(), Keyword::Provider);
    map.insert("โมเดล".to_string(), Keyword::Model);
    map.insert("เรียก".to_string(), Keyword::Call);

    // Governance
    map.insert("รัฐธรรมนูญ".to_string(), Keyword::Constitution);
    map.insert("กฎหมาย".to_string(), Keyword::Law);
    map.insert("กฎ".to_string(), Keyword::Rule);
    map.insert("สภา".to_string(), Keyword::Council);
    map.insert("ฝูง".to_string(), Keyword::Swarm);
    map.insert("ชุมชน".to_string(), Keyword::Community);
    map.insert("การบังคับใช้".to_string(), Keyword::Enforcement);
    map.insert("บังคับ".to_string(), Keyword::Mandatory);
    map.insert("แนะนำ".to_string(), Keyword::Advisory);
    map.insert("ทางเลือก".to_string(), Keyword::Optional);
    map.insert("บทบาท".to_string(), Keyword::Role);
    map.insert("สมาชิก".to_string(), Keyword::Member);
    map.insert("กลยุทธ์".to_string(), Keyword::Strategy);
    map.insert("แข่งขัน".to_string(), Keyword::Competitive);
    map.insert("ร่วมมือ".to_string(), Keyword::Collaborative);
    map.insert("วัฒนธรรม".to_string(), Keyword::Culture);
    map.insert("ค่านิยม".to_string(), Keyword::Values);
    map.insert("บรรทัดฐาน".to_string(), Keyword::Norms);
    map.insert("อัยการ".to_string(), Keyword::Prosecutor);
    map.insert("ผู้พิพากษา".to_string(), Keyword::Judge);

    // Async & Error handling
    map.insert("อะซิงโครนัส".to_string(), Keyword::Async);
    map.insert("รอ".to_string(), Keyword::Await);
    map.insert("ลอง".to_string(), Keyword::Try);
    map.insert("จับ".to_string(), Keyword::Catch);
    map.insert("ในที่สุด".to_string(), Keyword::Finally);
    map.insert("โยน".to_string(), Keyword::Throw);
    map.insert("สัญญา".to_string(), Keyword::Promise);
    map.insert("อนาคต".to_string(), Keyword::Future);

    // Data structures
    map.insert("โน้ต".to_string(), Keyword::Note);
    map.insert("คอร์ด".to_string(), Keyword::Chord);
    map.insert("ทำนอง".to_string(), Keyword::Melody);
    map.insert("ความกลมกลืน".to_string(), Keyword::Harmony);
    map.insert("จังหวะ".to_string(), Keyword::Rhythm);
    map.insert("จังหวะเวลา".to_string(), Keyword::Tempo);
    map.insert("มาตราส่วน".to_string(), Keyword::Scale);

    // Intent & State
    map.insert("เอนทิตี".to_string(), Keyword::Entity);
    map.insert("สถานะ".to_string(), Keyword::State);
    map.insert("เหตุการณ์".to_string(), Keyword::Event);

    // Flow & Orchestration
    map.insert("การไหล".to_string(), Keyword::Flow);
    map.insert("ชั้น".to_string(), Keyword::Layer);
    map.insert("เครือข่าย".to_string(), Keyword::Network);
    map.insert("ขึ้นอยู่กับ".to_string(), Keyword::DependsOn);
    map.insert("ออกอากาศ".to_string(), Keyword::Broadcast);
    map.insert("ผสาน".to_string(), Keyword::Merge);
    map.insert("ขนาน".to_string(), Keyword::Parallel);
    map.insert("ตามลำดับ".to_string(), Keyword::Sequential);
    map.insert("ดำเนินการ".to_string(), Keyword::Execute);
    map.insert("เมื่อ".to_string(), Keyword::When);
    map.insert("บน".to_string(), Keyword::On);
    map.insert("ทริกเกอร์".to_string(), Keyword::Trigger);
    // Native Thai translations
    map.insert("การกระทำ".to_string(), Keyword::Action);
    map.insert("กฎของเอเจนต์".to_string(), Keyword::AgentRule);
    map.insert("สถานะเอเจนต์".to_string(), Keyword::AgentState);
    map.insert("อนุญาต".to_string(), Keyword::Allow);
    map.insert("สไตล์การสื่อสาร".to_string(), Keyword::CommunicationStyle);
    map.insert("ค่าคงที่".to_string(), Keyword::Const);
    map.insert("การไหลของข้อมูล".to_string(), Keyword::DataFlow);
    map.insert("ปฏิเสธ".to_string(), Keyword::Deny);
    map.insert("ผู้ดำเนินการ".to_string(), Keyword::Executor);
    map.insert("เป็นทางการ".to_string(), Keyword::Formal);
    map.insert("ฟังก์ชัน".to_string(), Keyword::Function);
    map.insert("ไม่เป็นทางการ".to_string(), Keyword::Informal);
    map.insert("ความตั้งใจ".to_string(), Keyword::Intent);
    map.insert("ลำดับความสำคัญ".to_string(), Keyword::Priority);
    map.insert("ลูกโซ่กฎ".to_string(), Keyword::RuleChain);
    map.insert("ชุดกฎ".to_string(), Keyword::RuleSet);
    map.insert("เครื่องสถานะ".to_string(), Keyword::StateMachine);
    map.insert("ทางเทคนิค".to_string(), Keyword::Technical);
    map.insert("แปลง".to_string(), Keyword::Transform);
    map.insert("ตัวแปรอื่น".to_string(), Keyword::Var);
    map.insert("ต้องการ".to_string(), Keyword::Want);

    map
}
