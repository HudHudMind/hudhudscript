#![allow(unused_imports)]
use super::*;

impl VM {
    #[inline(always)]
    pub(crate) fn step_actor_messaging(
        &mut self,
        instr: &Instruction,
        ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        let instructions = ctx.instructions;
        let bytecode = ctx.bytecode;
        let ip = ctx.ip;
        let ip_ref = &mut *ctx.ip_ref;
        match instr {
            Instruction::Send {
                message: msg_reg,
                target: tgt_reg,
            } => {
                let target = self.registers[*tgt_reg as usize];
                let message = self.registers[*msg_reg as usize];

                let resolved = match target.as_string() {
                    Some(name) => self.get_var_cloned(&name).unwrap_or_else(|| target.clone()),
                    None => target.clone(),
                };

                let resolved_v = resolved;
                let actor_id = if let Some(obj) = resolved_v.as_object() {
                    obj.get("__actor_id").and_then(|v| {
                        if let Some(s) = v.as_string() {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
                } else {
                    None
                };

                if let Some(id) = actor_id {
                    let actor_ref = self.actors.get(&id).ok_or_else(|| {
                        compile_codes::runtime_error(format!("Actor not found: {}", id))
                    })?;
                    actor_ref
                        .send(message.clone())
                        .map_err(|e| compile_codes::runtime_error(format!("{}", e)))?;
                } else {
                    return Err(compile_codes::runtime_error(format!(
                        "Send target must be a valid actor reference"
                    )));
                }
            }
            Instruction::Receive {
                var_sym_idx: var_name_sym,
                src,
                ..
            } => {
                let var_name = bytecode.resolve_symbol(*var_name_sym as u32);
                let source = self.registers[*src as usize];

                let resolved = match source.as_string() {
                    Some(name) => self.get_var_cloned(&name).unwrap_or_else(|| source.clone()),
                    None => source.clone(),
                };

                let resolved_v = resolved;
                let actor_id = if let Some(obj) = resolved_v.as_object() {
                    obj.get("__actor_id").and_then(|v| {
                        if let Some(s) = v.as_string() {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
                } else {
                    None
                };

                let received = match actor_id {
                    Some(id) => {
                        let mailbox = self.actor_mailboxes.get(&id).ok_or_else(|| {
                            compile_codes::runtime_error(format!("Actor not found: {}", id))
                        })?;
                        mailbox.try_recv().map(|m| m.payload).ok_or_else(|| {
                            compile_codes::runtime_error("Receive: mailbox is empty")
                        })?
                    }
                    None => {
                        return Err(compile_codes::runtime_error(format!(
                            "Receive target must be a valid actor reference"
                        )));
                    }
                };

                self.set_var(&var_name, received)?;
            }
            _ => unreachable!("instruction routed to wrong execute helper"),
        }
        Ok(StepAction::Advance)
    }
}
