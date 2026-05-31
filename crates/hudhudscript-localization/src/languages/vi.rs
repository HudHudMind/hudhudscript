//! Vietnamese keyword mappings
//! Tiếng Việt - Vietnamese language support

use crate::keyword_map::Keyword;
use std::collections::HashMap;

pub fn get_keywords() -> HashMap<String, Keyword> {
    let mut map = HashMap::new();

    // Core keywords
    map.insert("cho".to_string(), Keyword::Let);
    map.insert("biến".to_string(), Keyword::Let);
    map.insert("dữ_liệu".to_string(), Keyword::Data);
    map.insert("đặt".to_string(), Keyword::Set);
    map.insert("đúng".to_string(), Keyword::True);
    map.insert("sai".to_string(), Keyword::False);

    // Control flow
    map.insert("nếu".to_string(), Keyword::If);
    map.insert("nếu_không".to_string(), Keyword::Else);
    map.insert("khác".to_string(), Keyword::Else);
    map.insert("trong_khi".to_string(), Keyword::While);
    map.insert("đối_với".to_string(), Keyword::For);
    map.insert("trả_về".to_string(), Keyword::Return);
    map.insert("ngắt".to_string(), Keyword::Break);
    map.insert("tiếp_tục".to_string(), Keyword::Continue);
    map.insert("chuyển".to_string(), Keyword::Switch);
    map.insert("trường_hợp".to_string(), Keyword::Case);
    map.insert("mặc_định".to_string(), Keyword::Default);

    // Module system
    map.insert("sử_dụng".to_string(), Keyword::Use);
    map.insert("như".to_string(), Keyword::As);
    map.insert("nhập".to_string(), Keyword::Import);
    map.insert("xuất".to_string(), Keyword::Export);
    map.insert("từ".to_string(), Keyword::From);

    // Agent system
    map.insert("đại_lý".to_string(), Keyword::Agent);
    map.insert("nhiệm_vụ".to_string(), Keyword::Task);
    map.insert("công_cụ".to_string(), Keyword::Tool);
    map.insert("tài_nguyên".to_string(), Keyword::Resource);
    map.insert("mcp".to_string(), Keyword::Mcp);
    map.insert("máy_chủ".to_string(), Keyword::Server);
    map.insert("cấu_hình".to_string(), Keyword::Config);
    map.insert("nhà_cung_cấp".to_string(), Keyword::Provider);
    map.insert("mô_hình".to_string(), Keyword::Model);
    map.insert("gọi".to_string(), Keyword::Call);

    // Governance
    map.insert("hiến_pháp".to_string(), Keyword::Constitution);
    map.insert("luật".to_string(), Keyword::Law);
    map.insert("quy_tắc".to_string(), Keyword::Rule);
    map.insert("hội_đồng".to_string(), Keyword::Council);
    map.insert("đàn".to_string(), Keyword::Swarm);
    map.insert("cộng_đồng".to_string(), Keyword::Community);
    map.insert("thực_thi".to_string(), Keyword::Enforcement);
    map.insert("bắt_buộc".to_string(), Keyword::Mandatory);
    map.insert("tư_vấn".to_string(), Keyword::Advisory);
    map.insert("tùy_chọn".to_string(), Keyword::Optional);
    map.insert("vai_trò".to_string(), Keyword::Role);
    map.insert("thành_viên".to_string(), Keyword::Member);
    map.insert("chiến_lược".to_string(), Keyword::Strategy);
    map.insert("cạnh_tranh".to_string(), Keyword::Competitive);
    map.insert("hợp_tác".to_string(), Keyword::Collaborative);
    map.insert("văn_hóa".to_string(), Keyword::Culture);
    map.insert("giá_trị".to_string(), Keyword::Values);
    map.insert("chuẩn_mực".to_string(), Keyword::Norms);
    map.insert("công_tố_viên".to_string(), Keyword::Prosecutor);
    map.insert("thẩm_phán".to_string(), Keyword::Judge);

    // Async & Error handling
    map.insert("bất_đồng_bộ".to_string(), Keyword::Async);
    map.insert("chờ".to_string(), Keyword::Await);
    map.insert("thử".to_string(), Keyword::Try);
    map.insert("bắt".to_string(), Keyword::Catch);
    map.insert("cuối_cùng".to_string(), Keyword::Finally);
    map.insert("ném".to_string(), Keyword::Throw);
    map.insert("lời_hứa".to_string(), Keyword::Promise);
    map.insert("tương_lai".to_string(), Keyword::Future);

    // Data structures
    map.insert("nốt".to_string(), Keyword::Note);
    map.insert("hợp_âm".to_string(), Keyword::Chord);
    map.insert("giai_điệu".to_string(), Keyword::Melody);
    map.insert("hòa_âm".to_string(), Keyword::Harmony);
    map.insert("nhịp_điệu".to_string(), Keyword::Rhythm);
    map.insert("nhịp_độ".to_string(), Keyword::Tempo);
    map.insert("thang".to_string(), Keyword::Scale);

    // Intent & State
    map.insert("thực_thể".to_string(), Keyword::Entity);
    map.insert("trạng_thái".to_string(), Keyword::State);
    map.insert("sự_kiện".to_string(), Keyword::Event);

    // Flow & Orchestration
    map.insert("luồng".to_string(), Keyword::Flow);
    map.insert("lớp".to_string(), Keyword::Layer);
    map.insert("mạng".to_string(), Keyword::Network);
    map.insert("phụ_thuộc_vào".to_string(), Keyword::DependsOn);
    map.insert("phát_sóng".to_string(), Keyword::Broadcast);
    map.insert("hợp_nhất".to_string(), Keyword::Merge);
    map.insert("song_song".to_string(), Keyword::Parallel);
    map.insert("tuần_tự".to_string(), Keyword::Sequential);
    map.insert("thực_thi".to_string(), Keyword::Execute);
    map.insert("khi".to_string(), Keyword::When);
    map.insert("trên".to_string(), Keyword::On);
    map.insert("kích_hoạt".to_string(), Keyword::Trigger);
    // Native Vietnamese translations
    map.insert("hành_động".to_string(), Keyword::Action);
    map.insert("quy_tắc_đại_lý".to_string(), Keyword::AgentRule);
    map.insert("trạng_thái_đại_lý".to_string(), Keyword::AgentState);
    map.insert("cho_phép".to_string(), Keyword::Allow);
    map.insert(
        "phong_cách_giao_tiếp".to_string(),
        Keyword::CommunicationStyle,
    );
    map.insert("hằng_số".to_string(), Keyword::Const);
    map.insert("luồng_dữ_liệu".to_string(), Keyword::DataFlow);
    map.insert("từ_chối".to_string(), Keyword::Deny);
    map.insert("người_thực_thi".to_string(), Keyword::Executor);
    map.insert("trang_trọng".to_string(), Keyword::Formal);
    map.insert("hàm".to_string(), Keyword::Function);
    map.insert("không_trang_trọng".to_string(), Keyword::Informal);
    map.insert("mục_đích".to_string(), Keyword::Intent);
    map.insert("ưu_tiên".to_string(), Keyword::Priority);
    map.insert("chuỗi_quy_tắc".to_string(), Keyword::RuleChain);
    map.insert("tập_quy_tắc".to_string(), Keyword::RuleSet);
    map.insert("máy_trạng_thái".to_string(), Keyword::StateMachine);
    map.insert("kỹ_thuật".to_string(), Keyword::Technical);
    map.insert("biến_đổi".to_string(), Keyword::Transform);
    map.insert("biến_var".to_string(), Keyword::Var);
    map.insert("muốn".to_string(), Keyword::Want);

    map
}
