//! Filesystem and execution module registrations.

use crate::vm::VM;

pub fn register(vm: &mut VM) {
    vm.register_module(
        "file",
        Box::new(|method, args| hudhud_fs::file_ops::dispatch(method, &args)),
    );
    vm.register_module(
        "exec",
        Box::new(|method, args| hudhud_exec::exec_ops::dispatch(method, &args)),
    );
    vm.register_module(
        "fs",
        Box::new(|method, args| hudhud_fs::fs_builtins::dispatch(method, &args)),
    );
    vm.register_module(
        "Path",
        Box::new(|method, args| hudhud_fs::path::dispatch(method, &args)),
    );
    vm.register_module(
        "Temp",
        Box::new(|method, args| hudhud_fs::temp::dispatch(method, &args)),
    );
    vm.register_module(
        "Glob",
        Box::new(|method, args| hudhud_fs::glob_ops::dispatch(method, &args)),
    );
}
