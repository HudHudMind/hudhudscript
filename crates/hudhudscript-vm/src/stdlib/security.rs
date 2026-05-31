//! Security, crypto, archive, and download module registrations.

use crate::vm::VM;

pub fn register(vm: &mut VM) {
    vm.register_module(
        "crypto",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_crypto::crypto_ops::CryptoMethodId>()?;
            hudhud_crypto::crypto_ops::dispatch(id, &args)
        }),
    );
    vm.register_module(
        "archive",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_archive::archive_ops::ScriptMethodId>()?;
            hudhud_archive::archive_ops::dispatch(id, &args)
        }),
    );
    vm.register_module(
        "system",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_metrics::system_metrics_ops::ScriptMethodId>()?;
            hudhud_metrics::system_metrics_ops::dispatch(id, &args)
        }),
    );
    vm.register_module(
        "download",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_download::download_ops::ScriptMethodId>()?;
            hudhud_download::download_ops::dispatch(id, &args)
        }),
    );
    vm.register_module(
        "docker",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_docker::docker_ops::ScriptMethodId>()?;
            hudhud_docker::docker_ops::dispatch(id, &args)
        }),
    );
    vm.register_module(
        "email",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_email::email_ops::ScriptMethodId>()?;
            hudhud_email::email_ops::dispatch(id, &args)
        }),
    );
    vm.register_module(
        "security",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_security::security_ops::ScriptMethodId>()?;
            hudhud_security::security_ops::dispatch(id, &args)
        }),
    );
    vm.register_module(
        "notify",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_notify::notify_ops::ScriptMethodId>()?;
            hudhud_notify::notify_ops::dispatch(id, &args)
        }),
    );
    vm.register_module(
        "firewall",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_firewall::firewall_ops::ScriptMethodId>()?;
            hudhud_firewall::firewall_ops::dispatch(id, &args)
        }),
    );
    vm.register_module(
        "unix",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_unix::unix_socket_ops::ScriptMethodId>()?;
            hudhud_unix::unix_socket_ops::dispatch(id, &args)
        }),
    );
    vm.register_module(
        "apt",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_apt::apt_ops::ScriptMethodId>()?;
            hudhud_apt::apt_ops::dispatch(id, &args)
        }),
    );
    vm.register_module(
        "xdg",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_xdg::xdg_ops::ScriptMethodId>()?;
            hudhud_xdg::xdg_ops::dispatch(id, &args)
        }),
    );
    vm.register_module(
        "codesign",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_codesign::codesign_ops::ScriptMethodId>()?;
            hudhud_codesign::codesign_ops::dispatch(id, &args)
        }),
    );
    vm.register_module(
        "PluginConfig",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_pluginconfig::plugin_config_ops::ScriptMethodId>()?;
            hudhud_pluginconfig::plugin_config_ops::dispatch(id, &args)
        }),
    );
    vm.register_module(
        "ocr",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_ocr::ocr_ops::ScriptMethodId>()?;
            hudhud_ocr::ocr_ops::dispatch(id, &args)
        }),
    );
    vm.register_module(
        "workflow",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_workflow::workflow_ops::ScriptMethodId>()?;
            hudhud_workflow::workflow_ops::dispatch(id, &args)
        }),
    );
    vm.register_module(
        "pdf",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_pdf::pdf_ops::ScriptMethodId>()?;
            hudhud_pdf::pdf_ops::dispatch(id, &args)
        }),
    );
    vm.register_module(
        "dbus",
        Box::new(|method, args| crate::stdlib::dbus_ops::call_dbus_method(method, &args)),
    );
    vm.register_module(
        "gpu",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_gpu::gpu_ops::ScriptMethodId>()?;
            hudhud_gpu::gpu_ops::dispatch(id, &args)
        }),
    );
    vm.register_module(
        "translate",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_translate::translate_ops::ScriptMethodId>()?;
            hudhud_translate::translate_ops::dispatch(id, &args)
        }),
    );
    vm.register_module(
        "e2e",
        Box::new(|method, args| crate::stdlib::e2e_ops::call_e2e_method(method, &args)),
    );
}
