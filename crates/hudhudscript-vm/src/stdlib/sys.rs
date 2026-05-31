//! System and environment module registrations.

use crate::vm::VM;

pub fn register(vm: &mut VM) {
    vm.register_module(
        "daemon",
        Box::new(|method, args| hudhud_exec::daemon_ops::dispatch(method, &args)),
    );
    vm.register_method("env", "get", Box::new(hudhud_env::env_ops::get));
    vm.register_method("env", "set", Box::new(hudhud_env::env_ops::set));
    vm.register_method("env", "remove", Box::new(hudhud_env::env_ops::remove));
    vm.register_method("env", "has", Box::new(hudhud_env::env_ops::has));
    vm.register_method("env", "all", Box::new(hudhud_env::env_ops::all));
    vm.register_method(
        "env",
        "all_unfiltered",
        Box::new(hudhud_env::env_ops::all_unfiltered),
    );
    vm.register_method("os", "name", Box::new(hudhud_os::os_ops::name));
    vm.register_method("os", "arch", Box::new(hudhud_os::os_ops::arch));
    vm.register_method("os", "version", Box::new(hudhud_os::os_ops::version));
    vm.register_method("os", "hostname", Box::new(hudhud_os::os_ops::hostname));
    vm.register_method("os", "username", Box::new(hudhud_os::os_ops::username));
    vm.register_method("os", "homedir", Box::new(hudhud_os::os_ops::homedir));
    vm.register_method("os", "tmpdir", Box::new(hudhud_os::os_ops::tmpdir));
    vm.register_method("os", "cpus", Box::new(hudhud_os::os_ops::cpus));
    vm.register_method("os", "uptime", Box::new(hudhud_os::os_ops::uptime));
    vm.register_method("os", "pid", Box::new(hudhud_os::os_ops::pid));
    vm.register_module(
        "stdin",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_term::stdin_ops::StdinMethodId>()?;
            hudhud_term::stdin_ops::dispatch(id, &args)
        }),
    );
    vm.register_module(
        "Terminal",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_term::terminal_ops::TerminalMethodId>()?;
            hudhud_term::terminal_ops::dispatch(id, &args)
        }),
    );
    vm.register_module(
        "log",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_term::log_ops::LogMethodId>()?;
            hudhud_term::log_ops::dispatch(id, &args)
        }),
    );
}
