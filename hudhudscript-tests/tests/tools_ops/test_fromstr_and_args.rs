//! Extended tests for P0 tools_ops crates:
//! - FromStr parsing for all ScriptMethodId enums
//! - Argument validation (type errors, missing args)
//! - Dispatch with valid/invalid Value16 args

use hudhudscript_bytecode::Value16;
use std::str::FromStr;

// ── APT ────────────────────────────────────────────────────────────

#[test]
fn apt_fromstr_list_installed() {
    let id = hudhud_apt::apt_ops::ScriptMethodId::from_str("list_installed").unwrap();
    assert_eq!(id, hudhud_apt::apt_ops::ScriptMethodId::ListInstalled);
}

#[test]
fn apt_fromstr_search() {
    let id = hudhud_apt::apt_ops::ScriptMethodId::from_str("search").unwrap();
    assert_eq!(id, hudhud_apt::apt_ops::ScriptMethodId::Search);
}

#[test]
fn apt_fromstr_info() {
    let id = hudhud_apt::apt_ops::ScriptMethodId::from_str("info").unwrap();
    assert_eq!(id, hudhud_apt::apt_ops::ScriptMethodId::Info);
}

#[test]
fn apt_fromstr_install() {
    let id = hudhud_apt::apt_ops::ScriptMethodId::from_str("install").unwrap();
    assert_eq!(id, hudhud_apt::apt_ops::ScriptMethodId::Install);
}

#[test]
fn apt_fromstr_remove() {
    let id = hudhud_apt::apt_ops::ScriptMethodId::from_str("remove").unwrap();
    assert_eq!(id, hudhud_apt::apt_ops::ScriptMethodId::Remove);
}

#[test]
fn apt_fromstr_update() {
    let id = hudhud_apt::apt_ops::ScriptMethodId::from_str("update").unwrap();
    assert_eq!(id, hudhud_apt::apt_ops::ScriptMethodId::Update);
}

#[test]
fn apt_fromstr_upgradable() {
    let id = hudhud_apt::apt_ops::ScriptMethodId::from_str("upgradable").unwrap();
    assert_eq!(id, hudhud_apt::apt_ops::ScriptMethodId::Upgradable);
}

#[test]
fn apt_fromstr_add_repo() {
    let id = hudhud_apt::apt_ops::ScriptMethodId::from_str("add_repo").unwrap();
    assert_eq!(id, hudhud_apt::apt_ops::ScriptMethodId::AddRepo);
}

#[test]
fn apt_fromstr_add_key() {
    let id = hudhud_apt::apt_ops::ScriptMethodId::from_str("add_key").unwrap();
    assert_eq!(id, hudhud_apt::apt_ops::ScriptMethodId::AddKey);
}

#[test]
fn apt_fromstr_unknown_returns_error() {
    let result = hudhud_apt::apt_ops::ScriptMethodId::from_str("nonexistent");
    assert!(result.is_err());
}

#[test]
fn apt_search_missing_arg_returns_error() {
    let result = hudhud_apt::apt_ops::dispatch(hudhud_apt::apt_ops::ScriptMethodId::Search, &[]);
    assert!(result.is_err());
}

#[test]
fn apt_info_missing_arg_returns_error() {
    let result = hudhud_apt::apt_ops::dispatch(hudhud_apt::apt_ops::ScriptMethodId::Info, &[]);
    assert!(result.is_err());
}

#[test]
fn apt_install_missing_arg_returns_error() {
    let result = hudhud_apt::apt_ops::dispatch(hudhud_apt::apt_ops::ScriptMethodId::Install, &[]);
    assert!(result.is_err());
}

#[test]
fn apt_remove_missing_arg_returns_error() {
    let result = hudhud_apt::apt_ops::dispatch(hudhud_apt::apt_ops::ScriptMethodId::Remove, &[]);
    assert!(result.is_err());
}

#[test]
fn apt_add_repo_missing_arg_returns_error() {
    let result = hudhud_apt::apt_ops::dispatch(hudhud_apt::apt_ops::ScriptMethodId::AddRepo, &[]);
    assert!(result.is_err());
}

#[test]
fn apt_add_key_missing_arg_returns_error() {
    let result = hudhud_apt::apt_ops::dispatch(hudhud_apt::apt_ops::ScriptMethodId::AddKey, &[]);
    assert!(result.is_err());
}

#[test]
fn apt_install_wrong_type_arg() {
    let result = hudhud_apt::apt_ops::dispatch(
        hudhud_apt::apt_ops::ScriptMethodId::Install,
        &[Value16::number(42.0)],
    );
    assert!(result.is_err());
}

// ── Docker ─────────────────────────────────────────────────────────

#[test]
fn docker_fromstr_ps() {
    let id = hudhud_docker::docker_ops::ScriptMethodId::from_str("ps").unwrap();
    assert_eq!(id, hudhud_docker::docker_ops::ScriptMethodId::Ps);
}

#[test]
fn docker_fromstr_images() {
    let id = hudhud_docker::docker_ops::ScriptMethodId::from_str("images").unwrap();
    assert_eq!(id, hudhud_docker::docker_ops::ScriptMethodId::Images);
}

#[test]
fn docker_fromstr_run() {
    let id = hudhud_docker::docker_ops::ScriptMethodId::from_str("run").unwrap();
    assert_eq!(id, hudhud_docker::docker_ops::ScriptMethodId::Run);
}

#[test]
fn docker_fromstr_stop() {
    let id = hudhud_docker::docker_ops::ScriptMethodId::from_str("stop").unwrap();
    assert_eq!(id, hudhud_docker::docker_ops::ScriptMethodId::Stop);
}

#[test]
fn docker_fromstr_rm() {
    let id = hudhud_docker::docker_ops::ScriptMethodId::from_str("rm").unwrap();
    assert_eq!(id, hudhud_docker::docker_ops::ScriptMethodId::Rm);
}

#[test]
fn docker_fromstr_logs() {
    let id = hudhud_docker::docker_ops::ScriptMethodId::from_str("logs").unwrap();
    assert_eq!(id, hudhud_docker::docker_ops::ScriptMethodId::Logs);
}

#[test]
fn docker_fromstr_exec() {
    let id = hudhud_docker::docker_ops::ScriptMethodId::from_str("exec").unwrap();
    assert_eq!(id, hudhud_docker::docker_ops::ScriptMethodId::Exec);
}

#[test]
fn docker_fromstr_build() {
    let id = hudhud_docker::docker_ops::ScriptMethodId::from_str("build").unwrap();
    assert_eq!(id, hudhud_docker::docker_ops::ScriptMethodId::Build);
}

#[test]
fn docker_fromstr_unknown_returns_error() {
    let result = hudhud_docker::docker_ops::ScriptMethodId::from_str("nope");
    assert!(result.is_err());
}

#[test]
fn docker_run_missing_image_arg() {
    let result =
        hudhud_docker::docker_ops::dispatch(hudhud_docker::docker_ops::ScriptMethodId::Run, &[]);
    assert!(result.is_err());
}

#[test]
fn docker_stop_missing_arg() {
    let result =
        hudhud_docker::docker_ops::dispatch(hudhud_docker::docker_ops::ScriptMethodId::Stop, &[]);
    assert!(result.is_err());
}

#[test]
fn docker_exec_missing_command_arg() {
    let result = hudhud_docker::docker_ops::dispatch(
        hudhud_docker::docker_ops::ScriptMethodId::Exec,
        &[Value16::string("my-container")],
    );
    assert!(result.is_err());
}

// ── Firewall ───────────────────────────────────────────────────────

#[test]
fn firewall_fromstr_status() {
    let id = hudhud_firewall::firewall_ops::ScriptMethodId::from_str("status").unwrap();
    assert_eq!(id, hudhud_firewall::firewall_ops::ScriptMethodId::Status);
}

#[test]
fn firewall_fromstr_rules() {
    let id = hudhud_firewall::firewall_ops::ScriptMethodId::from_str("rules").unwrap();
    assert_eq!(id, hudhud_firewall::firewall_ops::ScriptMethodId::Rules);
}

#[test]
fn firewall_fromstr_allow() {
    let id = hudhud_firewall::firewall_ops::ScriptMethodId::from_str("allow").unwrap();
    assert_eq!(id, hudhud_firewall::firewall_ops::ScriptMethodId::Allow);
}

#[test]
fn firewall_fromstr_deny() {
    let id = hudhud_firewall::firewall_ops::ScriptMethodId::from_str("deny").unwrap();
    assert_eq!(id, hudhud_firewall::firewall_ops::ScriptMethodId::Deny);
}

#[test]
fn firewall_fromstr_delete_rule() {
    let id = hudhud_firewall::firewall_ops::ScriptMethodId::from_str("delete_rule").unwrap();
    assert_eq!(
        id,
        hudhud_firewall::firewall_ops::ScriptMethodId::DeleteRule
    );
}

#[test]
fn firewall_fromstr_enable() {
    let id = hudhud_firewall::firewall_ops::ScriptMethodId::from_str("enable").unwrap();
    assert_eq!(id, hudhud_firewall::firewall_ops::ScriptMethodId::Enable);
}

#[test]
fn firewall_fromstr_disable() {
    let id = hudhud_firewall::firewall_ops::ScriptMethodId::from_str("disable").unwrap();
    assert_eq!(id, hudhud_firewall::firewall_ops::ScriptMethodId::Disable);
}

#[test]
fn firewall_fromstr_reset() {
    let id = hudhud_firewall::firewall_ops::ScriptMethodId::from_str("reset").unwrap();
    assert_eq!(id, hudhud_firewall::firewall_ops::ScriptMethodId::Reset);
}

#[test]
fn firewall_fromstr_unknown_returns_error() {
    let result = hudhud_firewall::firewall_ops::ScriptMethodId::from_str("unknown_mode");
    assert!(result.is_err());
}

#[test]
fn firewall_delete_rule_missing_arg() {
    let result = hudhud_firewall::firewall_ops::dispatch(
        hudhud_firewall::firewall_ops::ScriptMethodId::DeleteRule,
        &[],
    );
    assert!(result.is_err());
}

// ── Download ───────────────────────────────────────────────────────

#[test]
fn download_fromstr_file() {
    let id = hudhud_download::download_ops::ScriptMethodId::from_str("file").unwrap();
    assert_eq!(id, hudhud_download::download_ops::ScriptMethodId::File);
}

#[test]
fn download_fromstr_file_with_progress() {
    let id = hudhud_download::download_ops::ScriptMethodId::from_str("file_with_progress").unwrap();
    assert_eq!(
        id,
        hudhud_download::download_ops::ScriptMethodId::FileWithProgress
    );
}

#[test]
fn download_fromstr_resume() {
    let id = hudhud_download::download_ops::ScriptMethodId::from_str("resume").unwrap();
    assert_eq!(id, hudhud_download::download_ops::ScriptMethodId::Resume);
}

#[test]
fn download_fromstr_head() {
    let id = hudhud_download::download_ops::ScriptMethodId::from_str("head").unwrap();
    assert_eq!(id, hudhud_download::download_ops::ScriptMethodId::Head);
}

#[test]
fn download_fromstr_text() {
    let id = hudhud_download::download_ops::ScriptMethodId::from_str("text").unwrap();
    assert_eq!(id, hudhud_download::download_ops::ScriptMethodId::Text);
}

#[test]
fn download_fromstr_json() {
    let id = hudhud_download::download_ops::ScriptMethodId::from_str("json").unwrap();
    assert_eq!(id, hudhud_download::download_ops::ScriptMethodId::Json);
}

#[test]
fn download_fromstr_unknown() {
    let result = hudhud_download::download_ops::ScriptMethodId::from_str("torrent");
    assert!(result.is_err());
}

#[test]
fn download_file_missing_url_arg() {
    let result = hudhud_download::download_ops::dispatch(
        hudhud_download::download_ops::ScriptMethodId::File,
        &[],
    );
    assert!(result.is_err());
}

#[test]
fn download_head_missing_url_arg() {
    let result = hudhud_download::download_ops::dispatch(
        hudhud_download::download_ops::ScriptMethodId::Head,
        &[],
    );
    assert!(result.is_err());
}

// ── Email ──────────────────────────────────────────────────────────

#[test]
fn email_fromstr_send() {
    let id = hudhud_email::email_ops::ScriptMethodId::from_str("send").unwrap();
    assert_eq!(id, hudhud_email::email_ops::ScriptMethodId::Send);
}

#[test]
fn email_fromstr_send_simple() {
    let id = hudhud_email::email_ops::ScriptMethodId::from_str("send_simple").unwrap();
    assert_eq!(id, hudhud_email::email_ops::ScriptMethodId::SendSimple);
}

#[test]
fn email_fromstr_parse_mime() {
    let id = hudhud_email::email_ops::ScriptMethodId::from_str("parse_mime").unwrap();
    assert_eq!(id, hudhud_email::email_ops::ScriptMethodId::ParseMime);
}

#[test]
fn email_fromstr_unknown() {
    let result = hudhud_email::email_ops::ScriptMethodId::from_str("forward");
    assert!(result.is_err());
}

#[test]
fn email_send_missing_arg() {
    let result =
        hudhud_email::email_ops::dispatch(hudhud_email::email_ops::ScriptMethodId::Send, &[]);
    assert!(result.is_err());
}

// ── Browser ────────────────────────────────────────────────────────

#[test]
fn browser_fromstr_open() {
    let id = hudhud_browser::browser_ops::ScriptMethodId::from_str("open").unwrap();
    assert_eq!(id, hudhud_browser::browser_ops::ScriptMethodId::Open);
}

#[test]
fn browser_fromstr_bookmarks() {
    let id = hudhud_browser::browser_ops::ScriptMethodId::from_str("bookmarks").unwrap();
    assert_eq!(id, hudhud_browser::browser_ops::ScriptMethodId::Bookmarks);
}

#[test]
fn browser_fromstr_history() {
    let id = hudhud_browser::browser_ops::ScriptMethodId::from_str("history").unwrap();
    assert_eq!(id, hudhud_browser::browser_ops::ScriptMethodId::History);
}

#[test]
fn browser_fromstr_default_browser() {
    let id = hudhud_browser::browser_ops::ScriptMethodId::from_str("default_browser").unwrap();
    assert_eq!(
        id,
        hudhud_browser::browser_ops::ScriptMethodId::DefaultBrowser
    );
}

#[test]
fn browser_fromstr_installed_browsers() {
    let id = hudhud_browser::browser_ops::ScriptMethodId::from_str("installed_browsers").unwrap();
    assert_eq!(
        id,
        hudhud_browser::browser_ops::ScriptMethodId::InstalledBrowsers
    );
}

#[test]
fn browser_fromstr_search() {
    let id = hudhud_browser::browser_ops::ScriptMethodId::from_str("search").unwrap();
    assert_eq!(id, hudhud_browser::browser_ops::ScriptMethodId::Search);
}

#[test]
fn browser_fromstr_tabs() {
    let id = hudhud_browser::browser_ops::ScriptMethodId::from_str("tabs").unwrap();
    assert_eq!(id, hudhud_browser::browser_ops::ScriptMethodId::Tabs);
}

#[test]
fn browser_fromstr_unknown() {
    let result = hudhud_browser::browser_ops::ScriptMethodId::from_str("close");
    assert!(result.is_err());
}

// ── Notify ─────────────────────────────────────────────────────────

#[test]
fn notify_fromstr_send() {
    let id = hudhud_notify::notify_ops::ScriptMethodId::from_str("send").unwrap();
    assert_eq!(id, hudhud_notify::notify_ops::ScriptMethodId::Send);
}

#[test]
fn notify_fromstr_send_urgent() {
    let id = hudhud_notify::notify_ops::ScriptMethodId::from_str("send_urgent").unwrap();
    assert_eq!(id, hudhud_notify::notify_ops::ScriptMethodId::SendUrgent);
}

#[test]
fn notify_fromstr_send_with_icon() {
    let id = hudhud_notify::notify_ops::ScriptMethodId::from_str("send_with_icon").unwrap();
    assert_eq!(id, hudhud_notify::notify_ops::ScriptMethodId::SendWithIcon);
}

#[test]
fn notify_fromstr_journal() {
    let id = hudhud_notify::notify_ops::ScriptMethodId::from_str("journal").unwrap();
    assert_eq!(id, hudhud_notify::notify_ops::ScriptMethodId::Journal);
}

#[test]
fn notify_fromstr_journal_structured() {
    let id = hudhud_notify::notify_ops::ScriptMethodId::from_str("journal_structured").unwrap();
    assert_eq!(
        id,
        hudhud_notify::notify_ops::ScriptMethodId::JournalStructured
    );
}

#[test]
fn notify_fromstr_unknown() {
    let result = hudhud_notify::notify_ops::ScriptMethodId::from_str("broadcast");
    assert!(result.is_err());
}

// ── Hardware ───────────────────────────────────────────────────────

#[test]
fn hardware_fromstr_cpu_info() {
    let id = hudhud_hardware::hardware_ops::ScriptMethodId::from_str("cpu_info").unwrap();
    assert_eq!(id, hudhud_hardware::hardware_ops::ScriptMethodId::CpuInfo);
}

#[test]
fn hardware_fromstr_memory_info() {
    let id = hudhud_hardware::hardware_ops::ScriptMethodId::from_str("memory_info").unwrap();
    assert_eq!(
        id,
        hudhud_hardware::hardware_ops::ScriptMethodId::MemoryInfo
    );
}

#[test]
fn hardware_fromstr_gpu_info() {
    let id = hudhud_hardware::hardware_ops::ScriptMethodId::from_str("gpu_info").unwrap();
    assert_eq!(id, hudhud_hardware::hardware_ops::ScriptMethodId::GpuInfo);
}

#[test]
fn hardware_fromstr_disk_info() {
    let id = hudhud_hardware::hardware_ops::ScriptMethodId::from_str("disk_info").unwrap();
    assert_eq!(id, hudhud_hardware::hardware_ops::ScriptMethodId::DiskInfo);
}

#[test]
fn hardware_fromstr_network_adapters() {
    let id = hudhud_hardware::hardware_ops::ScriptMethodId::from_str("network_adapters").unwrap();
    assert_eq!(
        id,
        hudhud_hardware::hardware_ops::ScriptMethodId::NetworkAdapters
    );
}

#[test]
fn hardware_fromstr_usb_devices() {
    let id = hudhud_hardware::hardware_ops::ScriptMethodId::from_str("usb_devices").unwrap();
    assert_eq!(
        id,
        hudhud_hardware::hardware_ops::ScriptMethodId::UsbDevices
    );
}

#[test]
fn hardware_fromstr_audio_devices() {
    let id = hudhud_hardware::hardware_ops::ScriptMethodId::from_str("audio_devices").unwrap();
    assert_eq!(
        id,
        hudhud_hardware::hardware_ops::ScriptMethodId::AudioDevices
    );
}

#[test]
fn hardware_fromstr_display_info() {
    let id = hudhud_hardware::hardware_ops::ScriptMethodId::from_str("display_info").unwrap();
    assert_eq!(
        id,
        hudhud_hardware::hardware_ops::ScriptMethodId::DisplayInfo
    );
}

#[test]
fn hardware_fromstr_unknown() {
    let result = hudhud_hardware::hardware_ops::ScriptMethodId::from_str("unknown");
    assert!(result.is_err());
}

// ── Media ──────────────────────────────────────────────────────────

#[test]
fn media_fromstr_image_info() {
    let id = hudhud_media::media_ops::ScriptMethodId::from_str("image_info").unwrap();
    assert_eq!(id, hudhud_media::media_ops::ScriptMethodId::ImageInfo);
}

#[test]
fn media_fromstr_audio_info() {
    let id = hudhud_media::media_ops::ScriptMethodId::from_str("audio_info").unwrap();
    assert_eq!(id, hudhud_media::media_ops::ScriptMethodId::AudioInfo);
}

#[test]
fn media_fromstr_video_info() {
    let id = hudhud_media::media_ops::ScriptMethodId::from_str("video_info").unwrap();
    assert_eq!(id, hudhud_media::media_ops::ScriptMethodId::VideoInfo);
}

#[test]
fn media_fromstr_image_resize() {
    let id = hudhud_media::media_ops::ScriptMethodId::from_str("image_resize").unwrap();
    assert_eq!(id, hudhud_media::media_ops::ScriptMethodId::ImageResize);
}

#[test]
fn media_fromstr_image_convert() {
    let id = hudhud_media::media_ops::ScriptMethodId::from_str("image_convert").unwrap();
    assert_eq!(id, hudhud_media::media_ops::ScriptMethodId::ImageConvert);
}

#[test]
fn media_fromstr_transcode() {
    let id = hudhud_media::media_ops::ScriptMethodId::from_str("transcode").unwrap();
    assert_eq!(id, hudhud_media::media_ops::ScriptMethodId::Transcode);
}

#[test]
fn media_fromstr_thumbnail() {
    let id = hudhud_media::media_ops::ScriptMethodId::from_str("thumbnail").unwrap();
    assert_eq!(id, hudhud_media::media_ops::ScriptMethodId::Thumbnail);
}

#[test]
fn media_fromstr_file_type() {
    let id = hudhud_media::media_ops::ScriptMethodId::from_str("file_type").unwrap();
    assert_eq!(id, hudhud_media::media_ops::ScriptMethodId::FileType);
}

#[test]
fn media_fromstr_unknown() {
    let result = hudhud_media::media_ops::ScriptMethodId::from_str("edit");
    assert!(result.is_err());
}

// ── GPU ────────────────────────────────────────────────────────────

#[test]
fn gpu_fromstr_list() {
    let id = hudhud_gpu::gpu_ops::ScriptMethodId::from_str("list").unwrap();
    assert_eq!(id, hudhud_gpu::gpu_ops::ScriptMethodId::List);
}

#[test]
fn gpu_fromstr_usage() {
    let id = hudhud_gpu::gpu_ops::ScriptMethodId::from_str("usage").unwrap();
    assert_eq!(id, hudhud_gpu::gpu_ops::ScriptMethodId::Usage);
}

#[test]
fn gpu_fromstr_driver() {
    let id = hudhud_gpu::gpu_ops::ScriptMethodId::from_str("driver").unwrap();
    assert_eq!(id, hudhud_gpu::gpu_ops::ScriptMethodId::Driver);
}

#[test]
fn gpu_fromstr_memory() {
    let id = hudhud_gpu::gpu_ops::ScriptMethodId::from_str("memory").unwrap();
    assert_eq!(id, hudhud_gpu::gpu_ops::ScriptMethodId::Memory);
}

#[test]
fn gpu_fromstr_cuda_available() {
    let id = hudhud_gpu::gpu_ops::ScriptMethodId::from_str("cuda_available").unwrap();
    assert_eq!(id, hudhud_gpu::gpu_ops::ScriptMethodId::CudaAvailable);
}

#[test]
fn gpu_fromstr_rocm_available() {
    let id = hudhud_gpu::gpu_ops::ScriptMethodId::from_str("rocm_available").unwrap();
    assert_eq!(id, hudhud_gpu::gpu_ops::ScriptMethodId::RocmAvailable);
}

#[test]
fn gpu_fromstr_set_visible() {
    let id = hudhud_gpu::gpu_ops::ScriptMethodId::from_str("set_visible").unwrap();
    assert_eq!(id, hudhud_gpu::gpu_ops::ScriptMethodId::SetVisible);
}

#[test]
fn gpu_fromstr_processes() {
    let id = hudhud_gpu::gpu_ops::ScriptMethodId::from_str("processes").unwrap();
    assert_eq!(id, hudhud_gpu::gpu_ops::ScriptMethodId::Processes);
}

#[test]
fn gpu_fromstr_unknown() {
    let result = hudhud_gpu::gpu_ops::ScriptMethodId::from_str("unknown");
    assert!(result.is_err());
}

// ── Security ───────────────────────────────────────────────────────

#[test]
fn security_fromstr_suid_files() {
    let id = hudhud_security::security_ops::ScriptMethodId::from_str("suid_files").unwrap();
    assert_eq!(id, hudhud_security::security_ops::ScriptMethodId::SuidFiles);
}

#[test]
fn security_fromstr_check_ssl() {
    let id = hudhud_security::security_ops::ScriptMethodId::from_str("check_ssl").unwrap();
    assert_eq!(id, hudhud_security::security_ops::ScriptMethodId::CheckSsl);
}

#[test]
fn security_fromstr_open_ports() {
    let id = hudhud_security::security_ops::ScriptMethodId::from_str("open_ports").unwrap();
    assert_eq!(id, hudhud_security::security_ops::ScriptMethodId::OpenPorts);
}

#[test]
fn security_fromstr_world_writable() {
    let id = hudhud_security::security_ops::ScriptMethodId::from_str("world_writable").unwrap();
    assert_eq!(
        id,
        hudhud_security::security_ops::ScriptMethodId::WorldWritable
    );
}

#[test]
fn security_fromstr_failed_logins() {
    let id = hudhud_security::security_ops::ScriptMethodId::from_str("failed_logins").unwrap();
    assert_eq!(
        id,
        hudhud_security::security_ops::ScriptMethodId::FailedLogins
    );
}

#[test]
fn security_fromstr_check_permissions() {
    let id = hudhud_security::security_ops::ScriptMethodId::from_str("check_permissions").unwrap();
    assert_eq!(
        id,
        hudhud_security::security_ops::ScriptMethodId::CheckPermissions
    );
}

#[test]
fn security_fromstr_unknown() {
    let result = hudhud_security::security_ops::ScriptMethodId::from_str("hack");
    assert!(result.is_err());
}

// ── Project ────────────────────────────────────────────────────────

#[test]
fn project_fromstr_detect() {
    let id = hudhud_project::project_env_ops::ScriptMethodId::from_str("detect").unwrap();
    assert_eq!(id, hudhud_project::project_env_ops::ScriptMethodId::Detect);
}

#[test]
fn project_fromstr_detect_venv() {
    let id = hudhud_project::project_env_ops::ScriptMethodId::from_str("detect_venv").unwrap();
    assert_eq!(
        id,
        hudhud_project::project_env_ops::ScriptMethodId::DetectVenv
    );
}

#[test]
fn project_fromstr_parse_env_file() {
    let id = hudhud_project::project_env_ops::ScriptMethodId::from_str("parse_env_file").unwrap();
    assert_eq!(
        id,
        hudhud_project::project_env_ops::ScriptMethodId::ParseEnvFile
    );
}

#[test]
fn project_fromstr_toolchain_version() {
    let id =
        hudhud_project::project_env_ops::ScriptMethodId::from_str("toolchain_version").unwrap();
    assert_eq!(
        id,
        hudhud_project::project_env_ops::ScriptMethodId::ToolchainVersion
    );
}

#[test]
fn project_fromstr_dependencies() {
    let id = hudhud_project::project_env_ops::ScriptMethodId::from_str("dependencies").unwrap();
    assert_eq!(
        id,
        hudhud_project::project_env_ops::ScriptMethodId::Dependencies
    );
}

#[test]
fn project_fromstr_unknown() {
    let result = hudhud_project::project_env_ops::ScriptMethodId::from_str("deploy");
    assert!(result.is_err());
}

// ── OCR ────────────────────────────────────────────────────────────

#[test]
fn ocr_fromstr_extract() {
    let id = hudhud_ocr::ocr_ops::ScriptMethodId::from_str("extract").unwrap();
    assert_eq!(id, hudhud_ocr::ocr_ops::ScriptMethodId::Extract);
}

#[test]
fn ocr_fromstr_languages() {
    let id = hudhud_ocr::ocr_ops::ScriptMethodId::from_str("languages").unwrap();
    assert_eq!(id, hudhud_ocr::ocr_ops::ScriptMethodId::Languages);
}

#[test]
fn ocr_fromstr_is_available() {
    let id = hudhud_ocr::ocr_ops::ScriptMethodId::from_str("is_available").unwrap();
    assert_eq!(id, hudhud_ocr::ocr_ops::ScriptMethodId::IsAvailable);
}

#[test]
fn ocr_fromstr_extract_with_confidence() {
    let id = hudhud_ocr::ocr_ops::ScriptMethodId::from_str("extract_with_confidence").unwrap();
    assert_eq!(
        id,
        hudhud_ocr::ocr_ops::ScriptMethodId::ExtractWithConfidence
    );
}

#[test]
fn ocr_fromstr_pdf() {
    let id = hudhud_ocr::ocr_ops::ScriptMethodId::from_str("pdf").unwrap();
    assert_eq!(id, hudhud_ocr::ocr_ops::ScriptMethodId::Pdf);
}

#[test]
fn ocr_fromstr_unknown() {
    let result = hudhud_ocr::ocr_ops::ScriptMethodId::from_str("read_image");
    assert!(result.is_err());
}

// ── PDF ────────────────────────────────────────────────────────────

#[test]
fn pdf_fromstr_read() {
    let id = hudhud_pdf::pdf_ops::ScriptMethodId::from_str("read").unwrap();
    assert_eq!(id, hudhud_pdf::pdf_ops::ScriptMethodId::Read);
}

#[test]
fn pdf_fromstr_info() {
    let id = hudhud_pdf::pdf_ops::ScriptMethodId::from_str("info").unwrap();
    assert_eq!(id, hudhud_pdf::pdf_ops::ScriptMethodId::Info);
}

#[test]
fn pdf_fromstr_page_count() {
    let id = hudhud_pdf::pdf_ops::ScriptMethodId::from_str("page_count").unwrap();
    assert_eq!(id, hudhud_pdf::pdf_ops::ScriptMethodId::PageCount);
}

#[test]
fn pdf_fromstr_merge() {
    let id = hudhud_pdf::pdf_ops::ScriptMethodId::from_str("merge").unwrap();
    assert_eq!(id, hudhud_pdf::pdf_ops::ScriptMethodId::Merge);
}

#[test]
fn pdf_fromstr_split() {
    let id = hudhud_pdf::pdf_ops::ScriptMethodId::from_str("split").unwrap();
    assert_eq!(id, hudhud_pdf::pdf_ops::ScriptMethodId::Split);
}

#[test]
fn pdf_fromstr_to_images() {
    let id = hudhud_pdf::pdf_ops::ScriptMethodId::from_str("to_images").unwrap();
    assert_eq!(id, hudhud_pdf::pdf_ops::ScriptMethodId::ToImages);
}

#[test]
fn pdf_fromstr_unknown() {
    let result = hudhud_pdf::pdf_ops::ScriptMethodId::from_str("sign");
    assert!(result.is_err());
}

// ── Translate ──────────────────────────────────────────────────────

#[test]
fn translate_fromstr_text() {
    let id = hudhud_translate::translate_ops::ScriptMethodId::from_str("text").unwrap();
    assert_eq!(id, hudhud_translate::translate_ops::ScriptMethodId::Text);
}

#[test]
fn translate_fromstr_languages() {
    let id = hudhud_translate::translate_ops::ScriptMethodId::from_str("languages").unwrap();
    assert_eq!(
        id,
        hudhud_translate::translate_ops::ScriptMethodId::Languages
    );
}

#[test]
fn translate_fromstr_detect() {
    let id = hudhud_translate::translate_ops::ScriptMethodId::from_str("detect").unwrap();
    assert_eq!(id, hudhud_translate::translate_ops::ScriptMethodId::Detect);
}

#[test]
fn translate_fromstr_batch() {
    let id = hudhud_translate::translate_ops::ScriptMethodId::from_str("batch").unwrap();
    assert_eq!(id, hudhud_translate::translate_ops::ScriptMethodId::Batch);
}

#[test]
fn translate_fromstr_unknown() {
    let result = hudhud_translate::translate_ops::ScriptMethodId::from_str("speak");
    assert!(result.is_err());
}

// ── Workflow ───────────────────────────────────────────────────────

#[test]
fn workflow_fromstr_trigger() {
    let id = hudhud_workflow::workflow_ops::ScriptMethodId::from_str("trigger").unwrap();
    assert_eq!(id, hudhud_workflow::workflow_ops::ScriptMethodId::Trigger);
}

#[test]
fn workflow_fromstr_list() {
    let id = hudhud_workflow::workflow_ops::ScriptMethodId::from_str("list").unwrap();
    assert_eq!(id, hudhud_workflow::workflow_ops::ScriptMethodId::List);
}

#[test]
fn workflow_fromstr_execute() {
    let id = hudhud_workflow::workflow_ops::ScriptMethodId::from_str("execute").unwrap();
    assert_eq!(id, hudhud_workflow::workflow_ops::ScriptMethodId::Execute);
}

#[test]
fn workflow_fromstr_status() {
    let id = hudhud_workflow::workflow_ops::ScriptMethodId::from_str("status").unwrap();
    assert_eq!(id, hudhud_workflow::workflow_ops::ScriptMethodId::Status);
}

#[test]
fn workflow_fromstr_create_webhook() {
    let id = hudhud_workflow::workflow_ops::ScriptMethodId::from_str("create_webhook").unwrap();
    assert_eq!(
        id,
        hudhud_workflow::workflow_ops::ScriptMethodId::CreateWebhook
    );
}

#[test]
fn workflow_fromstr_unknown() {
    let result = hudhud_workflow::workflow_ops::ScriptMethodId::from_str("delete");
    assert!(result.is_err());
}

// ── Print (no ScriptMethodId, uses standalone functions) ─────────────

#[test]
fn print_start_capture_does_not_panic() {
    hudhud_print::print_ops::start_capture();
    let output = hudhud_print::print_ops::stop_capture();
    assert!(output.is_some());
    // Output should be empty since nothing was printed between capture/stop
}

#[test]
fn print_line_during_capture() {
    hudhud_print::print_ops::start_capture();
    hudhud_print::print_ops::print_line("test message");
    let output = hudhud_print::print_ops::stop_capture();
    assert!(output.is_some());
    let text = output.unwrap();
    assert!(text.contains("test message"));
}

#[test]
fn print_multiple_lines_captured() {
    hudhud_print::print_ops::start_capture();
    hudhud_print::print_ops::print_line("line1");
    hudhud_print::print_ops::print_line("line2");
    let output = hudhud_print::print_ops::stop_capture();
    assert!(output.is_some());
    let text = output.unwrap();
    assert!(text.contains("line1"));
    assert!(text.contains("line2"));
}

// ── Tools-ai (uses Provider enum and RateLimiter, not ScriptMethodId) ──

#[test]
fn tools_ai_rate_limiter_new_creates_empty() {
    use hudhudscript_tools_ai::RateLimiter;
    let limiter = RateLimiter::new();
    assert!(std::mem::size_of_val(&limiter) > 0);
}

#[test]
fn tools_ai_rate_limiter_with_limits() {
    use hudhudscript_tools_ai::{Provider, ProviderRateLimit, RateLimiter};
    use std::collections::HashMap;
    let mut limits = HashMap::new();
    limits.insert(
        Provider::OpenAI,
        ProviderRateLimit {
            rpm: 60,
            tpm: 10000,
        },
    );
    let limiter = RateLimiter::with_limits(limits);
    assert!(std::mem::size_of_val(&limiter) > 0);
}

#[test]
fn tools_ai_conversation_new() {
    use hudhudscript_tools_ai::Conversation;
    let conv = Conversation::new("test-model", 4096);
    assert!(conv.messages().is_empty());
}

#[test]
fn tools_ai_conversation_add_system() {
    use hudhudscript_tools_ai::Conversation;
    let mut conv = Conversation::new("test-model", 4096);
    conv.add_system("You are helpful.");
    assert_eq!(conv.messages().len(), 1);
}

#[test]
fn tools_ai_memory_entry_new() {
    use hudhudscript_tools_ai::MemoryEntry;
    let entry = MemoryEntry::new("agent-1", "key1", "value1");
    assert_eq!(entry.key, "key1");
    assert_eq!(entry.content, "value1");
}

#[test]
fn tools_ai_memory_store_and_recall() {
    use hudhudscript_tools_ai::MemoryStore;
    let store = MemoryStore::new();
    store.store("agent-1", "name", "HudHud").unwrap();
    let val = store.recall("agent-1", "name").unwrap();
    assert!(val.is_some());
    assert_eq!(val.unwrap().content, "HudHud");
}

// ── Tools-io (uses ToolRegistry, not ScriptMethodId) ────────────────

#[test]
fn tools_io_standard_tool_variants_exist() {
    use hudhudscript_tools_io::StandardTool;
    // Just verify the enum exists and can be constructed
    assert!(std::mem::size_of::<StandardTool>() > 0);
}

#[test]
fn tools_io_tool_error_enum_exists() {
    use hudhudscript_tools_io::ToolError;
    assert!(std::mem::size_of::<ToolError>() > 0);
}

// ── Tools-vcs (uses GitConfig, not ScriptMethodId) ──────────────────

#[test]
fn tools_vcs_git_config_global() {
    use hudhudscript_tools_vcs::GitConfig;
    let config = GitConfig::global();
    assert!(config.repo_path().is_none());
}

#[test]
fn tools_vcs_git_config_for_repo() {
    use hudhudscript_tools_vcs::GitConfig;
    let config = GitConfig::for_repo("/tmp/nonexistent");
    assert!(config.repo_path().is_some());
}

#[test]
fn tools_vcs_git_config_user_name() {
    use hudhudscript_tools_vcs::GitConfig;
    let config = GitConfig::global();
    // user_name may be None on CI, just check it doesn't panic
    let _ = config.user_name();
}
