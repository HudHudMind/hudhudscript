//! URL, regex, and events module registrations.

use crate::vm::VM;

pub fn register(vm: &mut VM) {
    vm.register_module(
        "URLParser",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_url::url_parser::UrlParserMethodId>()?;
            hudhud_url::url_parser::dispatch(id, &args)
        }),
    );
    vm.register_module(
        "Regex",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_regex::regex_ops::RegexMethodId>()?;
            hudhud_regex::regex_ops::dispatch(id, &args)
        }),
    );
    vm.register_module(
        "schedule",
        Box::new(|method, args| {
            let id = method.parse::<hudhud_scheduler::schedule_ops::ScriptMethodId>()?;
            hudhud_scheduler::schedule_ops::dispatch(id, &args)
        }),
    );
}
