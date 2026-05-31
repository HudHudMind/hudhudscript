use crate::vm::VM;

pub fn register_platform_modules(vm: &mut VM) {
    // Hardware detection
    vm.register_module(
        "hardware",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_hardware::hardware_ops::ScriptMethodId>()?;
            hudhud_hardware::hardware_ops::dispatch(id, &args)
        }),
    );
    // Project environment detection
    vm.register_module(
        "project",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_project::project_env_ops::ScriptMethodId>()?;
            hudhud_project::project_env_ops::dispatch(id, &args)
        }),
    );
    // Media
    vm.register_module(
        "media",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_media::media_ops::ScriptMethodId>()?;
            hudhud_media::media_ops::dispatch(id, &args)
        }),
    );
    // Transmission RPC
    vm.register_module(
        "torrent",
        Box::new(|method, args| crate::stdlib::torrent_ops::call_torrent_method(method, &args)),
    );
    // MPRIS media player control
    vm.register_module(
        "mpris",
        Box::new(|method, args| crate::stdlib::mpris_ops::call_mpris_method(method, &args)),
    );
    // Text-to-Speech
    vm.register_module(
        "tts",
        Box::new(|method, args| crate::stdlib::tts_ops::call_tts_method(method, &args)),
    );
    // Browser integration
    vm.register_module(
        "browser",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_browser::browser_ops::ScriptMethodId>()?;
            hudhud_browser::browser_ops::dispatch(id, &args)
        }),
    );
}
