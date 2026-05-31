//! Network module registrations.

use crate::vm::VM;

pub fn register(vm: &mut VM) {
    vm.register_method("http", "get", Box::new(hudhud_http::http_ops::get));
    vm.register_method("http", "post", Box::new(hudhud_http::http_ops::post));
    vm.register_method("http", "put", Box::new(hudhud_http::http_ops::put));
    vm.register_method("http", "delete", Box::new(hudhud_http::http_ops::delete));
    vm.register_method("http", "patch", Box::new(hudhud_http::http_ops::patch));
    vm.register_module(
        "tcp",
        Box::new(|method, args| hudhud_net::tcp_ops::dispatch(method, &args)),
    );
    vm.register_module(
        "udp",
        Box::new(|method, args| hudhud_net::udp_ops::dispatch(method, &args)),
    );
    vm.register_module(
        "ws",
        Box::new(|method, args| hudhud_net::ws_ops::dispatch(method, &args)),
    );
}
