//! Low-level emit and payload helpers for `CompileTarget`.

use super::*;

impl Compiler {

    pub fn emit(&mut self, instr: Instruction) {
        let max_reg = instr.max_register();
        if max_reg > self.current_max_register {
            self.current_max_register = max_reg;
        }
        // push_instr already keeps source_positions parallel (it
        // appends `None`). A pending statement-boundary position set
        // by `ct_mark_stmt_pos` overwrites that trailing None so the
        // DAP debug hook can resolve ip → (line, col).
        self.bytecode.push_instr(instr);
        if let Some(pos) = self.pending_source_pos.take() {
            if let Some(last) = self.bytecode.source_positions.last_mut() {
                *last = Some(pos);
            }
        }
    }
    pub fn emit_const(&mut self, val: Value16) -> u32 {
        self.bytecode.add_constant(val)
    }
    pub fn emit_num_const(&mut self, val: f64) -> u32 {
        self.bytecode.add_numeric_constant(val)
    }
    pub fn emit_int_const(&mut self, val: i64) -> u32 {
        self.bytecode.add_int_constant(val)
    }
    pub fn intern(&mut self, name: &str) -> u32 {
        hudhudscript_bytecode::interner::intern(name).0
    }
    pub fn mark_stmt_pos(&mut self, span: &Span) {
        // The NEXT `ct_emit` call will consume this and attach it to
        // the emitted instruction, keeping source_positions aligned.
        self.pending_source_pos = Some((span.start.line, span.start.column));
    }
    pub fn current_ip(&self) -> usize {
        self.bytecode.instructions.len()
    }
    pub fn patch(&mut self, ip: usize, instr: Instruction) {
        self.bytecode.instructions[ip] = instr;
    }
    pub fn add_loop_payload(&mut self, start: u32, end: u32) -> u32 {
        self.bytecode.add_loop_payload(start, end)
    }
    pub fn patch_loop_payload_end(&mut self, idx: u32, end: u32) {
        self.bytecode.patch_loop_end(idx, end);
    }
    // CROSS-2a: declaration-payload registration for the Compiler.
    pub fn add_enum_decl_payload(&mut self, payload: hudhudscript_bytecode::EnumDeclPayload) -> u32 {
        self.bytecode.add_enum_decl_payload(payload)
    }
    pub fn add_class_decl_payload(
        &mut self,
        payload: hudhudscript_bytecode::ClassDeclPayload,
    ) -> u32 {
        self.bytecode.add_class_decl_payload(payload)
    }
    pub fn add_trait_check_payload(
        &mut self,
        payload: hudhudscript_bytecode::TraitCheckPayload,
    ) -> u32 {
        self.bytecode.add_trait_check_payload(payload)
    }
    pub fn add_load_module_payload(
        &mut self,
        payload: hudhudscript_bytecode::LoadModulePayload,
    ) -> u32 {
        self.bytecode.add_load_module_payload(payload)
    }
    pub fn add_define_function_payload(
        &mut self,
        payload: hudhudscript_bytecode::DefineFunctionPayload,
    ) -> u32 {
        self.bytecode.add_define_function_payload(payload)
    }
    pub fn add_class_static_decl_payload(
        &mut self,
        payload: hudhudscript_bytecode::ClassStaticDeclPayload,
    ) -> u32 {
        self.bytecode.add_class_static_decl_payload(payload)
    }
    pub fn add_destruct_object_payload(
        &mut self,
        payload: hudhudscript_bytecode::DestructObjectPayload,
    ) -> u32 {
        self.bytecode.add_destruct_object_payload(payload)
    }
    // CROSS-2c+d: top-level Compiler forwards directly into the parent
    // `Bytecode` pools (no merge step; these are the final sink).
    pub fn add_call_payload(&mut self, sym: SymId, arg_count: u8) -> u32 {
        self.bytecode.add_call_payload(sym, arg_count)
    }
    pub fn add_two_sym_payload(&mut self, first: u32, second: u32) -> u32 {
        self.bytecode.add_two_sym_payload(first, second)
    }
    pub fn add_opt_sym_payload(&mut self, sym: Option<SymId>) -> u32 {
        self.bytecode.add_opt_sym_payload(sym)
    }
    pub fn is_known_class(&self, name: &str) -> bool {
        self.known_classes.contains(name)
    }
    pub fn is_known_generator(&self, name: &str) -> bool {
        self.known_generators.contains(name)
    }
    // Compiler allows await anywhere at the top level (no async check).
    pub fn check_await(&self, _span: &Span) -> CompileResult<()> {
        Ok(())
    }
}
