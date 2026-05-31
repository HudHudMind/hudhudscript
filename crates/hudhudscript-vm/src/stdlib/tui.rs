//! TUI module registration.

use crate::vm::VM;

pub fn register(vm: &mut VM) {
    vm.register_module(
        "tui",
        Box::new(|method, args| match method {
            "init" => hudhud_tui::tui_ops::tui_init(&args),
            "restore" => hudhud_tui::tui_ops::tui_restore(&args),
            "draw" => hudhud_tui::tui_ops::tui_draw(&args),
            "size" => hudhud_tui::tui_ops::tui_size(&args),
            "clear" => hudhud_tui::tui_ops::tui_clear(&args),
            "split" => hudhud_tui::tui_ops::tui_split(&args),
            "paragraph" => hudhud_tui::widgets::tui_paragraph(&args),
            "block" => hudhud_tui::widgets::tui_block(&args),
            "list" => hudhud_tui::widgets::tui_list(&args),
            "gauge" => hudhud_tui::widgets::tui_gauge(&args),
            "poll_event" => hudhud_tui::events::tui_poll_event(&args),
            _ => Err(hudhudscript_bytecode::error::compile_codes::runtime_error(
                format!("Unknown tui method: {}", method),
            )),
        }),
    );
}
