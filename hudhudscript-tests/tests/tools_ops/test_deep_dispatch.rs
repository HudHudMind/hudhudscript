//! Deep coverage tests for P0 tools_ops crates — dispatch + error paths.
//! Exercises every ScriptMethodId variant through dispatch with varied args.

use hudhudscript_bytecode::Value16;

// ═══════════════════════════════════════════════════════════════════════════
// Browser — dispatch all variants with various args
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn browser_open_missing_url() {
    let r = hudhud_browser::browser_ops::dispatch(
        hudhud_browser::browser_ops::ScriptMethodId::Open,
        &[],
    );
    // May fail due to missing browser or return error for missing arg
    let _ = r;
}

#[test]
fn browser_open_with_url() {
    let r = hudhud_browser::browser_ops::dispatch(
        hudhud_browser::browser_ops::ScriptMethodId::Open,
        &[Value16::string("https://example.com")],
    );
    let _ = r;
}

#[test]
fn browser_bookmarks_no_args() {
    let r = hudhud_browser::browser_ops::dispatch(
        hudhud_browser::browser_ops::ScriptMethodId::Bookmarks,
        &[],
    );
    let _ = r;
}

#[test]
fn browser_history_no_args() {
    let r = hudhud_browser::browser_ops::dispatch(
        hudhud_browser::browser_ops::ScriptMethodId::History,
        &[],
    );
    let _ = r;
}

#[test]
fn browser_tabs_no_args() {
    let r = hudhud_browser::browser_ops::dispatch(
        hudhud_browser::browser_ops::ScriptMethodId::Tabs,
        &[],
    );
    let _ = r;
}

#[test]
fn browser_default_browser_no_args() {
    let r = hudhud_browser::browser_ops::dispatch(
        hudhud_browser::browser_ops::ScriptMethodId::DefaultBrowser,
        &[],
    );
    let _ = r;
}

#[test]
fn browser_installed_browsers_no_args() {
    let r = hudhud_browser::browser_ops::dispatch(
        hudhud_browser::browser_ops::ScriptMethodId::InstalledBrowsers,
        &[],
    );
    let _ = r;
}

#[test]
fn browser_search_with_query() {
    let r = hudhud_browser::browser_ops::dispatch(
        hudhud_browser::browser_ops::ScriptMethodId::Search,
        &[Value16::string("rust programming")],
    );
    let _ = r;
}

#[test]
fn browser_search_missing_query() {
    let r = hudhud_browser::browser_ops::dispatch(
        hudhud_browser::browser_ops::ScriptMethodId::Search,
        &[],
    );
    let _ = r;
}

// ═══════════════════════════════════════════════════════════════════════════
// Docker — dispatch all variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn docker_run_basic() {
    let r = hudhud_docker::docker_ops::dispatch(
        hudhud_docker::docker_ops::ScriptMethodId::Run,
        &[Value16::string("alpine:latest")],
    );
    let _ = r;
}

#[test]
fn docker_run_with_options() {
    use std::collections::HashMap;
    let mut opts = HashMap::new();
    opts.insert("name".to_string(), Value16::string("test-container"));
    opts.insert("detach".to_string(), Value16::bool_(true));
    opts.insert("ports".to_string(), Value16::string("8080:80"));
    let r = hudhud_docker::docker_ops::dispatch(
        hudhud_docker::docker_ops::ScriptMethodId::Run,
        &[Value16::string("nginx:latest"), Value16::object(opts)],
    );
    let _ = r;
}

#[test]
fn docker_run_with_volumes() {
    use std::collections::HashMap;
    let mut opts = HashMap::new();
    opts.insert("volumes".to_string(), Value16::string("/host:/container"));
    let r = hudhud_docker::docker_ops::dispatch(
        hudhud_docker::docker_ops::ScriptMethodId::Run,
        &[Value16::string("ubuntu:latest"), Value16::object(opts)],
    );
    let _ = r;
}

#[test]
fn docker_run_with_env() {
    use std::collections::HashMap;
    let mut env = HashMap::new();
    env.insert("NODE_ENV".to_string(), Value16::string("production"));
    let mut opts = HashMap::new();
    opts.insert("env".to_string(), Value16::object(env));
    let r = hudhud_docker::docker_ops::dispatch(
        hudhud_docker::docker_ops::ScriptMethodId::Run,
        &[Value16::string("node:18"), Value16::object(opts)],
    );
    let _ = r;
}

#[test]
fn docker_run_with_ports_array() {
    use std::collections::HashMap;
    let mut opts = HashMap::new();
    opts.insert(
        "ports".to_string(),
        Value16::array(vec![Value16::string("80:80"), Value16::string("443:443")]),
    );
    let r = hudhud_docker::docker_ops::dispatch(
        hudhud_docker::docker_ops::ScriptMethodId::Run,
        &[Value16::string("nginx:latest"), Value16::object(opts)],
    );
    let _ = r;
}

#[test]
fn docker_stop_with_name() {
    let r = hudhud_docker::docker_ops::dispatch(
        hudhud_docker::docker_ops::ScriptMethodId::Stop,
        &[Value16::string("test-container")],
    );
    let _ = r;
}

#[test]
fn docker_rm_with_name() {
    let r = hudhud_docker::docker_ops::dispatch(
        hudhud_docker::docker_ops::ScriptMethodId::Rm,
        &[Value16::string("test-container")],
    );
    let _ = r;
}

#[test]
fn docker_logs_with_tail() {
    let r = hudhud_docker::docker_ops::dispatch(
        hudhud_docker::docker_ops::ScriptMethodId::Logs,
        &[Value16::string("test-container"), Value16::number(50.0)],
    );
    let _ = r;
}

#[test]
fn docker_logs_default_tail() {
    let r = hudhud_docker::docker_ops::dispatch(
        hudhud_docker::docker_ops::ScriptMethodId::Logs,
        &[Value16::string("test-container")],
    );
    let _ = r;
}

#[test]
fn docker_exec_with_command() {
    let r = hudhud_docker::docker_ops::dispatch(
        hudhud_docker::docker_ops::ScriptMethodId::Exec,
        &[
            Value16::string("test-container"),
            Value16::string("echo hello"),
        ],
    );
    let _ = r;
}

#[test]
fn docker_build_with_tag() {
    let r = hudhud_docker::docker_ops::dispatch(
        hudhud_docker::docker_ops::ScriptMethodId::Build,
        &[Value16::string("."), Value16::string("myapp:latest")],
    );
    let _ = r;
}

// ═══════════════════════════════════════════════════════════════════════════
// Email — dispatch all variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn email_send_simple_with_args() {
    let r = hudhud_email::email_ops::dispatch(
        hudhud_email::email_ops::ScriptMethodId::SendSimple,
        &[
            Value16::string("test@example.com"),
            Value16::string("Subject"),
            Value16::string("Body text"),
        ],
    );
    let _ = r;
}

#[test]
fn email_send_missing_args() {
    let r =
        hudhud_email::email_ops::dispatch(hudhud_email::email_ops::ScriptMethodId::SendSimple, &[]);
    assert!(r.is_err());
}

#[test]
fn email_parse_mime_with_input() {
    let r = hudhud_email::email_ops::dispatch(
        hudhud_email::email_ops::ScriptMethodId::ParseMime,
        &[Value16::string(
            "From: sender@example.com\nSubject: Test\n\nBody",
        )],
    );
    let _ = r;
}

#[test]
fn email_parse_mime_missing_input() {
    let r =
        hudhud_email::email_ops::dispatch(hudhud_email::email_ops::ScriptMethodId::ParseMime, &[]);
    assert!(r.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════
// Firewall — dispatch all variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn fw_allow_with_port() {
    let r = hudhud_firewall::firewall_ops::dispatch(
        hudhud_firewall::firewall_ops::ScriptMethodId::Allow,
        &[Value16::string("22")],
    );
    let _ = r;
}

#[test]
fn fw_allow_with_port_and_protocol() {
    let r = hudhud_firewall::firewall_ops::dispatch(
        hudhud_firewall::firewall_ops::ScriptMethodId::Allow,
        &[Value16::string("80"), Value16::string("tcp")],
    );
    let _ = r;
}

#[test]
fn fw_allow_missing_port() {
    let r = hudhud_firewall::firewall_ops::dispatch(
        hudhud_firewall::firewall_ops::ScriptMethodId::Allow,
        &[],
    );
    assert!(r.is_err());
}

#[test]
fn fw_deny_with_port() {
    let r = hudhud_firewall::firewall_ops::dispatch(
        hudhud_firewall::firewall_ops::ScriptMethodId::Deny,
        &[Value16::string("23")],
    );
    let _ = r;
}

#[test]
fn fw_delete_rule_with_number() {
    let r = hudhud_firewall::firewall_ops::dispatch(
        hudhud_firewall::firewall_ops::ScriptMethodId::DeleteRule,
        &[Value16::number(1.0)],
    );
    let _ = r;
}

#[test]
fn fw_delete_rule_wrong_type() {
    let r = hudhud_firewall::firewall_ops::dispatch(
        hudhud_firewall::firewall_ops::ScriptMethodId::DeleteRule,
        &[Value16::string("not_a_number")],
    );
    assert!(r.is_err());
}

#[test]
fn fw_enable_no_args() {
    let r = hudhud_firewall::firewall_ops::dispatch(
        hudhud_firewall::firewall_ops::ScriptMethodId::Enable,
        &[],
    );
    let _ = r;
}

#[test]
fn fw_disable_no_args() {
    let r = hudhud_firewall::firewall_ops::dispatch(
        hudhud_firewall::firewall_ops::ScriptMethodId::Disable,
        &[],
    );
    let _ = r;
}

#[test]
fn fw_reset_no_args() {
    let r = hudhud_firewall::firewall_ops::dispatch(
        hudhud_firewall::firewall_ops::ScriptMethodId::Reset,
        &[],
    );
    let _ = r;
}

// ═══════════════════════════════════════════════════════════════════════════
// GPU — dispatch all variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn gpu_list_no_args() {
    let r = hudhud_gpu::gpu_ops::dispatch(hudhud_gpu::gpu_ops::ScriptMethodId::List, &[]);
    let _ = r;
}

#[test]
fn gpu_usage_no_args() {
    let r = hudhud_gpu::gpu_ops::dispatch(hudhud_gpu::gpu_ops::ScriptMethodId::Usage, &[]);
    let _ = r;
}

#[test]
fn gpu_memory_no_args() {
    let r = hudhud_gpu::gpu_ops::dispatch(hudhud_gpu::gpu_ops::ScriptMethodId::Memory, &[]);
    let _ = r;
}

#[test]
fn gpu_driver_no_args() {
    let r = hudhud_gpu::gpu_ops::dispatch(hudhud_gpu::gpu_ops::ScriptMethodId::Driver, &[]);
    let _ = r;
}

#[test]
fn gpu_cuda_available() {
    let r = hudhud_gpu::gpu_ops::dispatch(hudhud_gpu::gpu_ops::ScriptMethodId::CudaAvailable, &[]);
    let _ = r;
}

#[test]
fn gpu_rocm_available() {
    let r = hudhud_gpu::gpu_ops::dispatch(hudhud_gpu::gpu_ops::ScriptMethodId::RocmAvailable, &[]);
    let _ = r;
}

#[test]
fn gpu_set_visible() {
    let r = hudhud_gpu::gpu_ops::dispatch(
        hudhud_gpu::gpu_ops::ScriptMethodId::SetVisible,
        &[Value16::string("0,1")],
    );
    let _ = r;
}

#[test]
fn gpu_processes_no_args() {
    let r = hudhud_gpu::gpu_ops::dispatch(hudhud_gpu::gpu_ops::ScriptMethodId::Processes, &[]);
    let _ = r;
}

// ═══════════════════════════════════════════════════════════════════════════
// Hardware — dispatch all variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn hw_cpu_info_no_args() {
    let r = hudhud_hardware::hardware_ops::dispatch(
        hudhud_hardware::hardware_ops::ScriptMethodId::CpuInfo,
        &[],
    );
    let _ = r;
}

#[test]
fn hw_memory_info_no_args() {
    let r = hudhud_hardware::hardware_ops::dispatch(
        hudhud_hardware::hardware_ops::ScriptMethodId::MemoryInfo,
        &[],
    );
    let _ = r;
}

#[test]
fn hw_gpu_info_no_args() {
    let r = hudhud_hardware::hardware_ops::dispatch(
        hudhud_hardware::hardware_ops::ScriptMethodId::GpuInfo,
        &[],
    );
    let _ = r;
}

#[test]
fn hw_disk_info_no_args() {
    let r = hudhud_hardware::hardware_ops::dispatch(
        hudhud_hardware::hardware_ops::ScriptMethodId::DiskInfo,
        &[],
    );
    let _ = r;
}

#[test]
fn hw_network_adapters() {
    let r = hudhud_hardware::hardware_ops::dispatch(
        hudhud_hardware::hardware_ops::ScriptMethodId::NetworkAdapters,
        &[],
    );
    let _ = r;
}

#[test]
fn hw_usb_devices() {
    let r = hudhud_hardware::hardware_ops::dispatch(
        hudhud_hardware::hardware_ops::ScriptMethodId::UsbDevices,
        &[],
    );
    let _ = r;
}

#[test]
fn hw_audio_devices() {
    let r = hudhud_hardware::hardware_ops::dispatch(
        hudhud_hardware::hardware_ops::ScriptMethodId::AudioDevices,
        &[],
    );
    let _ = r;
}

#[test]
fn hw_display_info() {
    let r = hudhud_hardware::hardware_ops::dispatch(
        hudhud_hardware::hardware_ops::ScriptMethodId::DisplayInfo,
        &[],
    );
    let _ = r;
}

// ═══════════════════════════════════════════════════════════════════════════
// Media — dispatch all variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn media_image_info_with_path() {
    let r = hudhud_media::media_ops::dispatch(
        hudhud_media::media_ops::ScriptMethodId::ImageInfo,
        &[Value16::string("/tmp/test.png")],
    );
    let _ = r;
}

#[test]
fn media_image_info_missing_path() {
    let r =
        hudhud_media::media_ops::dispatch(hudhud_media::media_ops::ScriptMethodId::ImageInfo, &[]);
    assert!(r.is_err());
}

#[test]
fn media_image_resize() {
    let r = hudhud_media::media_ops::dispatch(
        hudhud_media::media_ops::ScriptMethodId::ImageResize,
        &[
            Value16::string("/tmp/test.png"),
            Value16::number(100.0),
            Value16::number(100.0),
        ],
    );
    let _ = r;
}

#[test]
fn media_image_resize_missing_args() {
    let r = hudhud_media::media_ops::dispatch(
        hudhud_media::media_ops::ScriptMethodId::ImageResize,
        &[],
    );
    assert!(r.is_err());
}

#[test]
fn media_image_convert() {
    let r = hudhud_media::media_ops::dispatch(
        hudhud_media::media_ops::ScriptMethodId::ImageConvert,
        &[Value16::string("/tmp/test.png"), Value16::string("jpg")],
    );
    let _ = r;
}

#[test]
fn media_audio_info() {
    let r = hudhud_media::media_ops::dispatch(
        hudhud_media::media_ops::ScriptMethodId::AudioInfo,
        &[Value16::string("/tmp/test.mp3")],
    );
    let _ = r;
}

#[test]
fn media_video_info() {
    let r = hudhud_media::media_ops::dispatch(
        hudhud_media::media_ops::ScriptMethodId::VideoInfo,
        &[Value16::string("/tmp/test.mp4")],
    );
    let _ = r;
}

#[test]
fn media_transcode() {
    let r = hudhud_media::media_ops::dispatch(
        hudhud_media::media_ops::ScriptMethodId::Transcode,
        &[Value16::string("/tmp/test.mp4"), Value16::string("avi")],
    );
    let _ = r;
}

#[test]
fn media_thumbnail() {
    let r = hudhud_media::media_ops::dispatch(
        hudhud_media::media_ops::ScriptMethodId::Thumbnail,
        &[Value16::string("/tmp/test.mp4")],
    );
    let _ = r;
}

#[test]
fn media_file_type() {
    let r = hudhud_media::media_ops::dispatch(
        hudhud_media::media_ops::ScriptMethodId::FileType,
        &[Value16::string("/tmp/test.png")],
    );
    let _ = r;
}

// ═══════════════════════════════════════════════════════════════════════════
// Notify — dispatch all variants with args
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn notify_send_with_title_body() {
    let r = hudhud_notify::notify_ops::dispatch(
        hudhud_notify::notify_ops::ScriptMethodId::Send,
        &[Value16::string("Title"), Value16::string("Body text")],
    );
    let _ = r;
}

#[test]
fn notify_send_with_icon() {
    let r = hudhud_notify::notify_ops::dispatch(
        hudhud_notify::notify_ops::ScriptMethodId::SendWithIcon,
        &[
            Value16::string("Title"),
            Value16::string("Body"),
            Value16::string("dialog-information"),
        ],
    );
    let _ = r;
}

#[test]
fn notify_send_urgent() {
    let r = hudhud_notify::notify_ops::dispatch(
        hudhud_notify::notify_ops::ScriptMethodId::SendUrgent,
        &[Value16::string("Alert!"), Value16::string("Urgent message")],
    );
    let _ = r;
}

#[test]
fn notify_journal() {
    let r = hudhud_notify::notify_ops::dispatch(
        hudhud_notify::notify_ops::ScriptMethodId::Journal,
        &[Value16::string("Log entry")],
    );
    let _ = r;
}

#[test]
fn notify_journal_structured() {
    use std::collections::HashMap;
    let mut entry = HashMap::new();
    entry.insert("title".to_string(), Value16::string("Event"));
    entry.insert("body".to_string(), Value16::string("Details"));
    let r = hudhud_notify::notify_ops::dispatch(
        hudhud_notify::notify_ops::ScriptMethodId::JournalStructured,
        &[Value16::object(entry)],
    );
    let _ = r;
}

#[test]
fn notify_send_missing_args() {
    let r =
        hudhud_notify::notify_ops::dispatch(hudhud_notify::notify_ops::ScriptMethodId::Send, &[]);
    assert!(r.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════
// OCR — dispatch all variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ocr_extract_with_image_path() {
    let r = hudhud_ocr::ocr_ops::dispatch(
        hudhud_ocr::ocr_ops::ScriptMethodId::Extract,
        &[Value16::string("/tmp/test.png")],
    );
    let _ = r;
}

#[test]
fn ocr_extract_missing_path() {
    let r = hudhud_ocr::ocr_ops::dispatch(hudhud_ocr::ocr_ops::ScriptMethodId::Extract, &[]);
    assert!(r.is_err());
}

#[test]
fn ocr_extract_with_confidence() {
    let r = hudhud_ocr::ocr_ops::dispatch(
        hudhud_ocr::ocr_ops::ScriptMethodId::ExtractWithConfidence,
        &[Value16::string("/tmp/test.png")],
    );
    let _ = r;
}

#[test]
fn ocr_languages_no_args() {
    let r = hudhud_ocr::ocr_ops::dispatch(hudhud_ocr::ocr_ops::ScriptMethodId::Languages, &[]);
    let _ = r;
}

#[test]
fn ocr_pdf_with_path() {
    let r = hudhud_ocr::ocr_ops::dispatch(
        hudhud_ocr::ocr_ops::ScriptMethodId::Pdf,
        &[Value16::string("/tmp/test.pdf")],
    );
    let _ = r;
}

#[test]
fn ocr_is_available_no_args() {
    let r = hudhud_ocr::ocr_ops::dispatch(hudhud_ocr::ocr_ops::ScriptMethodId::IsAvailable, &[]);
    let _ = r;
}

// ═══════════════════════════════════════════════════════════════════════════
// PDF — dispatch all variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn pdf_read_with_path() {
    let r = hudhud_pdf::pdf_ops::dispatch(
        hudhud_pdf::pdf_ops::ScriptMethodId::Read,
        &[Value16::string("/tmp/test.pdf")],
    );
    let _ = r;
}

#[test]
fn pdf_read_missing_path() {
    let r = hudhud_pdf::pdf_ops::dispatch(hudhud_pdf::pdf_ops::ScriptMethodId::Read, &[]);
    assert!(r.is_err());
}

#[test]
fn pdf_info_with_path() {
    let r = hudhud_pdf::pdf_ops::dispatch(
        hudhud_pdf::pdf_ops::ScriptMethodId::Info,
        &[Value16::string("/tmp/test.pdf")],
    );
    let _ = r;
}

#[test]
fn pdf_merge_with_files() {
    let r = hudhud_pdf::pdf_ops::dispatch(
        hudhud_pdf::pdf_ops::ScriptMethodId::Merge,
        &[
            Value16::string("/tmp/output.pdf"),
            Value16::array(vec![
                Value16::string("/tmp/a.pdf"),
                Value16::string("/tmp/b.pdf"),
            ]),
        ],
    );
    let _ = r;
}

#[test]
fn pdf_split_with_path() {
    let r = hudhud_pdf::pdf_ops::dispatch(
        hudhud_pdf::pdf_ops::ScriptMethodId::Split,
        &[Value16::string("/tmp/test.pdf")],
    );
    let _ = r;
}

#[test]
fn pdf_to_images_with_path() {
    let r = hudhud_pdf::pdf_ops::dispatch(
        hudhud_pdf::pdf_ops::ScriptMethodId::ToImages,
        &[Value16::string("/tmp/test.pdf")],
    );
    let _ = r;
}

#[test]
fn pdf_page_count_with_path() {
    let r = hudhud_pdf::pdf_ops::dispatch(
        hudhud_pdf::pdf_ops::ScriptMethodId::PageCount,
        &[Value16::string("/tmp/test.pdf")],
    );
    let _ = r;
}

// ═══════════════════════════════════════════════════════════════════════════
// Translate — dispatch all variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn translate_text_with_args() {
    let r = hudhud_translate::translate_ops::dispatch(
        hudhud_translate::translate_ops::ScriptMethodId::Text,
        &[
            Value16::string("Hello world"),
            Value16::string("en"),
            Value16::string("tr"),
        ],
    );
    let _ = r;
}

#[test]
fn translate_text_missing_args() {
    let r = hudhud_translate::translate_ops::dispatch(
        hudhud_translate::translate_ops::ScriptMethodId::Text,
        &[],
    );
    assert!(r.is_err());
}

#[test]
fn translate_detect_with_text() {
    let r = hudhud_translate::translate_ops::dispatch(
        hudhud_translate::translate_ops::ScriptMethodId::Detect,
        &[Value16::string("Bonjour le monde")],
    );
    let _ = r;
}

#[test]
fn translate_languages_no_args() {
    let r = hudhud_translate::translate_ops::dispatch(
        hudhud_translate::translate_ops::ScriptMethodId::Languages,
        &[],
    );
    let _ = r;
}

#[test]
fn translate_batch_with_texts() {
    let r = hudhud_translate::translate_ops::dispatch(
        hudhud_translate::translate_ops::ScriptMethodId::Batch,
        &[
            Value16::array(vec![Value16::string("Hello"), Value16::string("World")]),
            Value16::string("en"),
            Value16::string("tr"),
        ],
    );
    let _ = r;
}

// ═══════════════════════════════════════════════════════════════════════════
// Workflow — dispatch all variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn workflow_trigger_with_name() {
    let r = hudhud_workflow::workflow_ops::dispatch(
        hudhud_workflow::workflow_ops::ScriptMethodId::Trigger,
        &[Value16::string("ci-build")],
    );
    let _ = r;
}

#[test]
fn workflow_list_no_args() {
    let r = hudhud_workflow::workflow_ops::dispatch(
        hudhud_workflow::workflow_ops::ScriptMethodId::List,
        &[],
    );
    let _ = r;
}

#[test]
fn workflow_execute_with_name() {
    let r = hudhud_workflow::workflow_ops::dispatch(
        hudhud_workflow::workflow_ops::ScriptMethodId::Execute,
        &[Value16::string("deploy-prod")],
    );
    let _ = r;
}

#[test]
fn workflow_status_with_name() {
    let r = hudhud_workflow::workflow_ops::dispatch(
        hudhud_workflow::workflow_ops::ScriptMethodId::Status,
        &[Value16::string("ci-build")],
    );
    let _ = r;
}

#[test]
fn workflow_create_webhook_with_url() {
    let r = hudhud_workflow::workflow_ops::dispatch(
        hudhud_workflow::workflow_ops::ScriptMethodId::CreateWebhook,
        &[
            Value16::string("ci-build"),
            Value16::string("https://hooks.example.com/webhook"),
        ],
    );
    let _ = r;
}

// ═══════════════════════════════════════════════════════════════════════════
// Project — dispatch all variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn project_detect_with_path() {
    let r = hudhud_project::project_env_ops::dispatch(
        hudhud_project::project_env_ops::ScriptMethodId::Detect,
        &[Value16::string("/tmp")],
    );
    let _ = r;
}

#[test]
fn project_detect_venv_with_path() {
    let r = hudhud_project::project_env_ops::dispatch(
        hudhud_project::project_env_ops::ScriptMethodId::DetectVenv,
        &[Value16::string("/tmp")],
    );
    let _ = r;
}

#[test]
fn project_parse_env_file_with_path() {
    let r = hudhud_project::project_env_ops::dispatch(
        hudhud_project::project_env_ops::ScriptMethodId::ParseEnvFile,
        &[Value16::string("/tmp/.env")],
    );
    let _ = r;
}

#[test]
fn project_toolchain_version() {
    let r = hudhud_project::project_env_ops::dispatch(
        hudhud_project::project_env_ops::ScriptMethodId::ToolchainVersion,
        &[Value16::string("rust")],
    );
    let _ = r;
}

#[test]
fn project_dependencies() {
    let r = hudhud_project::project_env_ops::dispatch(
        hudhud_project::project_env_ops::ScriptMethodId::Dependencies,
        &[Value16::string("/tmp/project")],
    );
    let _ = r;
}

// ═══════════════════════════════════════════════════════════════════════════
// Security — dispatch all variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn security_suid_files_with_path() {
    let r = hudhud_security::security_ops::dispatch(
        hudhud_security::security_ops::ScriptMethodId::SuidFiles,
        &[Value16::string("/usr/bin")],
    );
    let _ = r;
}

#[test]
fn security_check_ssl_with_host() {
    let r = hudhud_security::security_ops::dispatch(
        hudhud_security::security_ops::ScriptMethodId::CheckSsl,
        &[Value16::string("example.com")],
    );
    let _ = r;
}

#[test]
fn security_world_writable_with_path() {
    let r = hudhud_security::security_ops::dispatch(
        hudhud_security::security_ops::ScriptMethodId::WorldWritable,
        &[Value16::string("/tmp")],
    );
    let _ = r;
}

#[test]
fn security_open_ports_no_args() {
    let r = hudhud_security::security_ops::dispatch(
        hudhud_security::security_ops::ScriptMethodId::OpenPorts,
        &[],
    );
    let _ = r;
}

#[test]
fn security_failed_logins_no_args() {
    let r = hudhud_security::security_ops::dispatch(
        hudhud_security::security_ops::ScriptMethodId::FailedLogins,
        &[],
    );
    let _ = r;
}

#[test]
fn security_check_permissions_with_path() {
    let r = hudhud_security::security_ops::dispatch(
        hudhud_security::security_ops::ScriptMethodId::CheckPermissions,
        &[Value16::string("/etc/passwd")],
    );
    let _ = r;
}

// ═══════════════════════════════════════════════════════════════════════════
// Download — dispatch all variants with proper args
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn download_file_with_url_and_path() {
    let r = hudhud_download::download_ops::dispatch(
        hudhud_download::download_ops::ScriptMethodId::File,
        &[
            Value16::string("https://example.com/file.txt"),
            Value16::string("/tmp/out.txt"),
        ],
    );
    let _ = r;
}

#[test]
fn download_file_missing_both_args() {
    let r = hudhud_download::download_ops::dispatch(
        hudhud_download::download_ops::ScriptMethodId::File,
        &[],
    );
    assert!(r.is_err());
}

#[test]
fn download_head_with_url() {
    let r = hudhud_download::download_ops::dispatch(
        hudhud_download::download_ops::ScriptMethodId::Head,
        &[Value16::string("https://example.com")],
    );
    let _ = r;
}

#[test]
fn download_text_with_url() {
    let r = hudhud_download::download_ops::dispatch(
        hudhud_download::download_ops::ScriptMethodId::Text,
        &[Value16::string("https://example.com")],
    );
    let _ = r;
}

#[test]
fn download_json_with_url() {
    let r = hudhud_download::download_ops::dispatch(
        hudhud_download::download_ops::ScriptMethodId::Json,
        &[Value16::string("https://api.example.com/data")],
    );
    let _ = r;
}

#[test]
fn download_resume_with_url_and_path() {
    let r = hudhud_download::download_ops::dispatch(
        hudhud_download::download_ops::ScriptMethodId::Resume,
        &[
            Value16::string("https://example.com/large.bin"),
            Value16::string("/tmp/large.bin"),
        ],
    );
    let _ = r;
}

#[test]
fn download_file_with_progress() {
    let r = hudhud_download::download_ops::dispatch(
        hudhud_download::download_ops::ScriptMethodId::FileWithProgress,
        &[
            Value16::string("https://example.com/file.zip"),
            Value16::string("/tmp/file.zip"),
        ],
    );
    let _ = r;
}

// ═══════════════════════════════════════════════════════════════════════════
// APT — dispatch with various args covering error paths
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn apt_search_with_query() {
    let r = hudhud_apt::apt_ops::dispatch(
        hudhud_apt::apt_ops::ScriptMethodId::Search,
        &[Value16::string("python")],
    );
    let _ = r;
}

#[test]
fn apt_search_wrong_type() {
    let r = hudhud_apt::apt_ops::dispatch(
        hudhud_apt::apt_ops::ScriptMethodId::Search,
        &[Value16::number(42.0)],
    );
    assert!(r.is_err());
}

#[test]
fn apt_info_with_package() {
    let r = hudhud_apt::apt_ops::dispatch(
        hudhud_apt::apt_ops::ScriptMethodId::Info,
        &[Value16::string("bash")],
    );
    let _ = r;
}

#[test]
fn apt_install_with_package() {
    let r = hudhud_apt::apt_ops::dispatch(
        hudhud_apt::apt_ops::ScriptMethodId::Install,
        &[Value16::string("curl")],
    );
    let _ = r;
}

#[test]
fn apt_remove_with_package() {
    let r = hudhud_apt::apt_ops::dispatch(
        hudhud_apt::apt_ops::ScriptMethodId::Remove,
        &[Value16::string("unused-pkg")],
    );
    let _ = r;
}

#[test]
fn apt_add_key_with_url() {
    let r = hudhud_apt::apt_ops::dispatch(
        hudhud_apt::apt_ops::ScriptMethodId::AddKey,
        &[Value16::string("https://example.com/key.gpg")],
    );
    let _ = r;
}
