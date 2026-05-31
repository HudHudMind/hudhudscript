//! Math and datetime module registrations.

use crate::vm::VM;

pub fn register(vm: &mut VM) {
    vm.register_module(
        "linalg",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_linalg::linalg::LinAlgMethodId>()?;
            hudhud_linalg::linalg::dispatch(id, &args)
        }),
    );
    vm.register_module(
        "stats",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_stats::stats::StatsMethodId>()?;
            hudhud_stats::stats::dispatch(id, &args)
        }),
    );
    vm.register_module(
        "Date",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_datetime::date::DateMethodId>()?;
            hudhud_datetime::date::dispatch(id, &args)
        }),
    );
    vm.register_module(
        "duration",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_datetime::duration::DurationMethodId>()?;
            hudhud_datetime::duration::dispatch(id, &args)
        }),
    );
}
