//! Shared builtins facade — re-exports domain crate modules for test compatibility.

// Domain crate re-exports
pub use hudhud_apt::apt_ops;
pub use hudhud_archive::archive_ops;
pub use hudhud_browser::browser_ops;
pub use hudhud_codesign::codesign_ops;
pub use hudhud_crypto::crypto_ops;
pub use hudhud_datetime::{date, duration};
pub use hudhud_docker::docker_ops;
pub use hudhud_download::download_ops;
pub use hudhud_email::email_ops;
pub use hudhud_encoding;
pub use hudhud_env::env_ops;
pub use hudhud_eventbus::event_bus_ops;
pub use hudhud_firewall::firewall_ops;
pub use hudhud_fs::{file_ops, fs_builtins, glob_ops, path, temp};
pub use hudhud_gpu::gpu_ops;
pub use hudhud_hardware::hardware_ops;
pub use hudhud_http::{http_ops, json};
pub use hudhud_linalg::linalg;
pub use hudhud_math::math;
pub use hudhud_media::media_ops;
pub use hudhud_metrics::system_metrics_ops;
pub use hudhud_net::{tcp_ops, udp_ops, ws_ops};
pub use hudhud_notify::notify_ops;
pub use hudhud_ocr::ocr_ops;
pub use hudhud_os::os_ops;
pub use hudhud_pdf::pdf_ops;
pub use hudhud_plugin::plugin_ops;
pub use hudhud_pluginconfig::plugin_config_ops;
pub use hudhud_print::print_ops;
pub use hudhud_project::project_env_ops;
pub use hudhud_regex::regex_ops;
pub use hudhud_scheduler::schedule_ops;
pub use hudhud_security::security_ops;
pub use hudhud_sop::sop_ops;
pub use hudhud_stats::stats;
pub use hudhud_term::{log_ops, stdin_ops, terminal_ops};
pub use hudhud_timer::timer_ops;
pub use hudhud_translate::translate_ops;
pub use hudhud_typeops::types_ops;
pub use hudhud_unix::unix_socket_ops;
pub use hudhud_url::url_parser;
pub use hudhud_workflow::workflow_ops;
pub use hudhud_xdg::xdg_ops;
pub use hudhudscript_exception::exception;

// Serial ops (csv, toml, yaml, ini) are in hudhud-serial
pub use hudhud_serial::{csv_ops, ini_ops, toml_ops, yaml_ops};

// Daemon ops is in hudhud-exec
pub use hudhud_exec::daemon_ops;

// VM modules re-exported for test compatibility
pub use hudhudscript_vm::vm::array;
pub use hudhudscript_vm::vm::json as vm_json;
pub use hudhudscript_vm::vm::map;
pub use hudhudscript_vm::vm::operations;
pub use hudhudscript_vm::vm::set;
pub use hudhudscript_vm::vm::string;

// Bytecode re-exports
pub use hudhudscript_bytecode::interner;
