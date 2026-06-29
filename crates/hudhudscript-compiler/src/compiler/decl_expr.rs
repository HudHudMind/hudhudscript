use super::*;
impl Compiler {
    /// Delegates to the free function `root_var_name`.
    #[allow(dead_code)]
    pub(super) fn root_var_name(expr: &Expr) -> Option<String> {
        root_var_name(expr)
    }

    /// Compile a match pattern for the bytecode compiler, returning
    /// (skip_positions, register) — positions of JumpIfFalse instructions
    /// and the register they read from.
    pub(super) fn compile_match_pattern_bytecode(
        &mut self,
        pattern: &hudhudscript_ast::MatchPattern,
        match_reg: u8,
    ) -> CompileResult<Vec<usize>> {
        use hudhudscript_ast::MatchPattern;
        match pattern {
            MatchPattern::Wildcard => {
                Ok(vec![])
            }
            MatchPattern::Identifier(name) => {
                self.bytecode.push_instr(Instruction::BindVar(sym(name)));
                Ok(vec![])
            }
            MatchPattern::Literal(lit) => {
                let const_val = match lit {
                    Literal::Number(n, _) => Value16::number(*n),
                    Literal::String(s) => Value16::string(s.clone()),
                    Literal::Boolean(b) => Value16::bool_(*b),
                    Literal::Null => Value16::null(),
                    _ => Value16::null(),
                };
                let idx = self.bytecode.add_constant(const_val);
                let lit_reg = crate::compiler::regalloc::temp_reg();
                self.bytecode.push_instr(Instruction::LoadConst { dst: lit_reg, const_idx: idx as u16 });
                // Compare in temp so match_reg survives for next arm
                let cmp_reg = crate::compiler::regalloc::temp_reg();
                self.bytecode.push_instr(Instruction::IntCmp { dst: cmp_reg, src1: match_reg, src2: lit_reg, op: 4 });
                let skip = self.bytecode.instructions.len();
                self.bytecode.push_instr(Instruction::JumpIfFalse { src: cmp_reg, offset: 0 });
                Ok(vec![skip])
            }
            MatchPattern::EnumVariant {
                enum_name,
                variant,
                binding,
            } => {
                let payload_idx = self
                    .bytecode
                    .add_two_sym_payload(sym(enum_name).0, sym(variant).0);
                self.bytecode
                    .push_instr(Instruction::MatchVariant(payload_idx));
                self.bytecode.push_instr(Instruction::Move { dst: match_reg, src: 255 });
                let skip = self.bytecode.instructions.len();
                self.bytecode.push_instr(Instruction::JumpIfFalse { src: match_reg, offset: 0 });
                if let Some(bind) = binding {
                    self.bytecode.push_instr(Instruction::BindVar(sym(bind)));
                }
                Ok(vec![skip])
            }
            MatchPattern::Or(sub_patterns) => {
                let mut match_jumps: Vec<usize> = Vec::new();
                let mut fail_skip = None;
                for (i, sub) in sub_patterns.iter().enumerate() {
                    let is_last = i == sub_patterns.len() - 1;
                    let sub_skips = self.compile_match_pattern_bytecode(sub, match_reg)?;
                    if sub_skips.is_empty() {
                        break;
                    }
                    if !is_last {
                        let jmp_to_body = self.bytecode.instructions.len();
                        self.bytecode.push_instr(Instruction::Jump(0));
                        match_jumps.push(jmp_to_body);
                        let next = self.bytecode.instructions.len();
                        for s in sub_skips {
                            self.bytecode.instructions[s] =
                                Instruction::JumpIfFalse { src: match_reg, offset: jump_off(s, next) as i16 };
                        }
                    } else {
                        fail_skip = Some(sub_skips);
                    }
                }
                let body_start = self.bytecode.instructions.len();
                for jmp in match_jumps {
                    self.bytecode.instructions[jmp] = Instruction::Jump(jump_off(jmp, body_start));
                }
                Ok(fail_skip.unwrap_or_default())
            }
        }
    }

    pub(super) fn compile_expr(&mut self, expr: &Expr) -> CompileResult<()> {
        let r = crate::compiler::expr::compile_reg::compile_expr_to_reg(self, expr, &mut RegAlloc::new_with_base(self.ct_next_local_reg())?);
        self.bytecode.push_instr(Instruction::Move { dst: 255, src: r });
        Ok(())
    }

    /// Convert a `TypeAnnotation` to a lowercase type name string used by
    /// `StoreTyped` and the VM's runtime type check (Issue #452).
    pub(crate) fn type_annotation_name(annotation: &hudhudscript_ast::TypeAnnotation) -> String {
        use hudhudscript_ast::TypeAnnotation;
        match annotation {
            TypeAnnotation::Number => "number".to_string(),
            TypeAnnotation::String => "string".to_string(),
            TypeAnnotation::Boolean => "boolean".to_string(),
            TypeAnnotation::Null => "null".to_string(),
            TypeAnnotation::Any => "any".to_string(),
            TypeAnnotation::Array(_) => "array".to_string(),
            TypeAnnotation::Tool => "tool".to_string(),
            TypeAnnotation::Resource => "resource".to_string(),
            TypeAnnotation::Server => "server".to_string(),
            TypeAnnotation::Generic(name) => name.to_lowercase(),
            TypeAnnotation::Union(types) => types
                .iter()
                .map(Self::type_annotation_name)
                .collect::<Vec<_>>()
                .join("|"),
            TypeAnnotation::Parameterized { base, .. } => Self::type_annotation_name(base),
        }
    }

    /// Compile a destructuring pattern. The value to destructure is on the stack. — Issue #668
    pub(super) fn compile_destructure_pattern(
        &mut self,
        pattern: &hudhudscript_ast::Pattern,
        is_const: bool,
    ) -> CompileResult<()> {
        use hudhudscript_ast::Pattern;
        match pattern {
            Pattern::Identifier(name) => {
                self.declare_local(name, is_const)?;
                let tr = crate::compiler::regalloc::temp_reg();
                self.bytecode.push_instr(Instruction::Move { dst: tr, src: 255 });
                let sym = self.intern(name);
                if is_const {
                    self.bytecode.push_instr(Instruction::StoreConst { src: tr, sym: sym as u16 });
                } else {
                    self.bytecode.push_instr(Instruction::StoreGlobal { src: tr, sym: sym as u16 });
                }
            }
            Pattern::IdentifierDefault(name, default_expr) => {
                // Duplicate the value on stack, check if null
                // Simple approach: store to temp, check, use default if null
                let temp = format!("__destruct_temp_{}", name);
                let temp_sym = self.intern(&temp) as u16;
                let tr = crate::compiler::regalloc::temp_reg();
                self.bytecode.push_instr(Instruction::Move { dst: tr, src: 255 });
                self.bytecode.push_instr(Instruction::StoreGlobal {
                    src: tr,
                    sym: temp_sym,
                });
                let tr = crate::compiler::regalloc::temp_reg();
                self.bytecode.push_instr(Instruction::LoadGlobal {
                    dst: tr,
                    sym: temp_sym,
                });
                self.bytecode.push_instr(Instruction::Move { dst: 255, src: tr });
                let null_idx = self.bytecode.add_constant(Value16::null());
                let tr = crate::compiler::regalloc::temp_reg();
                self.bytecode.push_instr(Instruction::LoadConst { dst: tr, const_idx: null_idx as u16 });
                self.bytecode.push_instr(Instruction::Move { dst: 255, src: tr });
                self.bytecode.push_instr(Instruction::Move { dst: tr, src: 255 });
                let jump_to_default = self.bytecode.instructions.len();
                self.bytecode.push_instr(Instruction::JumpIfFalse { src: tr, offset: 0 });
                // Use default
                self.compile_expr(default_expr)?;
                let jump_to_end = self.bytecode.instructions.len();
                self.bytecode.push_instr(Instruction::Jump(0));
                // Use value
                let else_start = self.bytecode.instructions.len();
                self.bytecode.instructions[jump_to_default] =
                    Instruction::JumpIfFalse { src: tr, offset: jump_off(jump_to_default, else_start) as i16 };
                {
                    let sym = self.ct_intern(&temp);
                    let tr = crate::compiler::regalloc::temp_reg();
                    self.bytecode.push_instr(Instruction::LoadGlobal { dst: tr, sym: sym as u16 });
                    self.bytecode.push_instr(Instruction::Move { dst: 255, src: tr });
                }
                let end = self.bytecode.instructions.len();
                self.bytecode.instructions[jump_to_end] =
                    Instruction::Jump(jump_off(jump_to_end, end));
                // Store result
                self.declare_local(name, is_const)?;
                let tr = crate::compiler::regalloc::temp_reg();
                self.bytecode.push_instr(Instruction::Move { dst: tr, src: 255 });
                let sym = self.intern(name);
                if is_const {
                    self.bytecode.push_instr(Instruction::StoreConst { src: tr, sym: sym as u16 });
                } else {
                    self.bytecode.push_instr(Instruction::StoreGlobal { src: tr, sym: sym as u16 });
                }
            }
            Pattern::Array { elements, rest } => {
                // Store the array value to a temp var
                let temp = "__destruct_arr".to_string();
                let temp_sym = self.intern(&temp) as u16;
                let tr = crate::compiler::regalloc::temp_reg();
                self.bytecode.push_instr(Instruction::Move { dst: tr, src: 255 });
                self.bytecode.push_instr(Instruction::StoreGlobal {
                    src: tr,
                    sym: temp_sym,
                });
                // Extract each element by index
                for (i, elem_pattern) in elements.iter().enumerate() {
                    let tr = crate::compiler::regalloc::temp_reg();
                    self.bytecode.push_instr(Instruction::LoadGlobal {
                        dst: tr,
                        sym: temp_sym,
                    });
                    self.bytecode.push_instr(Instruction::Move { dst: 255, src: tr });
                    let idx = self.bytecode.add_int_constant(i as i64);
                    { let tr_i = crate::compiler::regalloc::temp_reg(); self.bytecode.push_instr(Instruction::LoadIntConst { dst: tr_i, const_idx: idx as u16 }); let tr_dst = crate::compiler::regalloc::temp_reg(); self.bytecode.push_instr(Instruction::Index { dst: tr_dst, obj: tr, idx: tr_i }); self.bytecode.push_instr(Instruction::Move { dst: 255, src: tr_dst }); }
                    self.compile_destructure_pattern(elem_pattern, is_const)?;
                }
                // Handle rest element
                if let Some(rest_pattern) = rest {
                    // For rest, we push the remaining elements as an array
                    // Simple approach: use DestructArray instruction
                    {
                        let sym = self.ct_intern(&temp);
                        let tr = crate::compiler::regalloc::temp_reg();
                        self.bytecode.push_instr(Instruction::LoadGlobal { dst: tr, sym: sym as u16 });
                        self.bytecode.push_instr(Instruction::Move { dst: 255, src: tr });
                    }
                    self.bytecode
                        .push_instr(Instruction::DestructArray(elements.len() as u16, true));
                    self.compile_destructure_pattern(rest_pattern, is_const)?;
                }
            }
            Pattern::Object { properties, rest } => {
                let temp = "__destruct_obj".to_string();
                let temp_sym = self.intern(&temp) as u16;
                let tr = crate::compiler::regalloc::temp_reg();
                self.bytecode.push_instr(Instruction::Move { dst: tr, src: 255 });
                self.bytecode.push_instr(Instruction::StoreGlobal {
                    src: tr,
                    sym: temp_sym,
                });
                for (key, sub_pattern) in properties {
                    let tr = crate::compiler::regalloc::temp_reg();
                    let tr2 = crate::compiler::regalloc::temp_reg();
                    self.bytecode.push_instr(Instruction::LoadGlobal {
                        dst: tr,
                        sym: temp_sym,
                    });
                    self.bytecode.push_instr(Instruction::GetProperty { dst: tr2, obj: tr, prop_sym: sym(key).0 as u16 });
                    self.bytecode.push_instr(Instruction::Move { dst: 255, src: tr2 });
                    self.compile_destructure_pattern(sub_pattern, is_const)?;
                }
                if let Some(rest_pattern) = rest {
                    let key_syms: Vec<SymId> = properties.iter().map(|(k, _)| sym(k)).collect();
                    {
                        let s = self.ct_intern(&temp);
                        let tr = crate::compiler::regalloc::temp_reg();
                        self.bytecode.push_instr(Instruction::LoadGlobal { dst: tr, sym: s as u16 });
                        self.bytecode.push_instr(Instruction::Move { dst: 255, src: tr });
                    }
                    let payload_idx = self.bytecode.add_destruct_object_payload(
                        hudhudscript_bytecode::DestructObjectPayload {
                            used_keys: key_syms,
                        },
                    );
                    self.bytecode
                        .push_instr(Instruction::DestructObject(payload_idx));
                    self.compile_destructure_pattern(rest_pattern, is_const)?;
                }
            }
        }
        Ok(())
    }
}
