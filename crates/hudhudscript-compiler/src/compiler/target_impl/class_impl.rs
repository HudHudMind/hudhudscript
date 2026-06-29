//! Class compilation for `CompileTarget`.

use super::*;

impl Compiler {

    pub fn compile_class_impl(&mut self, class_decl: &hudhudscript_ast::ClassDecl) -> CompileResult<()> {
        use hudhudscript_ast::ClassMember;
        let mut method_names: Vec<String> = Vec::new();
        let mut static_method_names: Vec<String> = Vec::new();
        let mut method_access_list: Vec<u8> = Vec::new();
        let mut static_field_names: Vec<String> = Vec::new();

        // If extending a parent, copy parent methods into child class metadata
        if let Some(parent_name) = &class_decl.parent {
            let parent_prefix = format!("{}::", parent_name);
            let inherited: Vec<(String, String)> = self
                .bytecode
                .function_keys()
                .into_iter()
                .filter(|k| k.starts_with(&parent_prefix))
                .map(|k| {
                    let method = k[parent_prefix.len()..].to_string();
                    (k.clone(), method)
                })
                .collect();
            for (parent_chunk_name, method) in inherited {
                let child_chunk_name = format!("{}::{}", class_decl.name, method);
                let child_overrides = class_decl.members.iter().any(|m| match m {
                    ClassMember::Method { name, .. } => *name == method,
                    ClassMember::Constructor { .. } => method == "constructor",
                    _ => false,
                });
                if !child_overrides {
                    let chunk = self.bytecode.get_function(&parent_chunk_name);
                    if let Some(chunk) = chunk {
                        self.bytecode.add_function(child_chunk_name, chunk);
                        if method != "constructor" {
                            method_names.push(method);
                        }
                    }
                }
            }
        }

        for member in &class_decl.members {
            match member {
                ClassMember::Method {
                    name,
                    params,
                    body,
                    is_static,
                    access,
                    ..
                } => {
                    let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                    let chunk_name = format!("{}::{}", class_decl.name, name);
                    let chunk = self.compile_function_body(param_names, body)?;
                    self.bytecode.add_function(chunk_name, Arc::new(chunk));
                    let access_byte: u8 = match access {
                        hudhudscript_ast::AccessModifier::Public => 0,
                        hudhudscript_ast::AccessModifier::Private => 1,
                        hudhudscript_ast::AccessModifier::Protected => 2,
                    };
                    if *is_static {
                        static_method_names.push(name.clone());
                        method_access_list.push(access_byte);
                    } else {
                        method_names.push(name.clone());
                        method_access_list.push(access_byte);
                    }
                }
                ClassMember::Constructor { params, body, .. } => {
                    let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                    let chunk_name = format!("{}::constructor", class_decl.name);
                    let chunk = self.compile_function_body(param_names, body)?;
                    self.bytecode.add_function(chunk_name, Arc::new(chunk));
                    method_names.push("constructor".to_string());
                }
                ClassMember::Field {
                    name,
                    initializer,
                    is_static,
                    ..
                } => {
                    if let Some(val) = initializer {
                        self.compile_expr(val)?;
                    } else {
                        let idx = self.bytecode.add_constant(Value16::null());
                        let tr = crate::compiler::regalloc::temp_reg();
                        self.bytecode.push_instr(Instruction::LoadConst { dst: tr, const_idx: idx as u16 });
                        self.bytecode.push_instr(Instruction::Move { dst: 255, src: tr });
                    }
                    let tr = crate::compiler::regalloc::temp_reg();
                    self.bytecode.push_instr(Instruction::Move { dst: tr, src: 255 });
                    if *is_static {
                        static_field_names.push(name.clone());
                    }
                    self.bytecode.push_instr(Instruction::StoreGlobal {
                        src: tr,
                        sym: hudhudscript_bytecode::interner::intern(&format!(
                            "{}::{}",
                            class_decl.name, name
                        ))
                        .0 as u16,
                    });
                }
            }
        }
        let parent = class_decl.parent.clone();
        // Collect non-constructor, non-static method names for trait checking
        let all_class_methods: Vec<String> = method_names
            .iter()
            .filter(|m| *m != "constructor")
            .cloned()
            .collect();
        let name_sym = sym(&class_decl.name);
        let parent_sym = parent.as_ref().map(|p| sym(p));
        let method_syms: Vec<SymId> = method_names.iter().map(|m| sym(m)).collect();
        let class_idx =
            self.bytecode
                .add_class_decl_payload(hudhudscript_bytecode::ClassDeclPayload {
                    name: name_sym,
                    parent: parent_sym,
                    methods: method_syms,
                    method_access: method_access_list,
                    is_abstract: class_decl.is_abstract,
                });
        self.bytecode.push_instr(Instruction::ClassDecl(class_idx));
        self.known_classes.insert(class_decl.name.clone());
        if !static_method_names.is_empty() || !static_field_names.is_empty() {
            let static_syms: Vec<SymId> = static_method_names.iter().map(|m| sym(m)).collect();
            let static_field_syms: Vec<SymId> = static_field_names.iter().map(|f| sym(f)).collect();
            let static_idx = self.bytecode.add_class_static_decl_payload(
                hudhudscript_bytecode::ClassStaticDeclPayload {
                    class_name: name_sym,
                    static_methods: static_syms,
                    static_fields: static_field_syms,
                },
            );
            self.bytecode
                .push_instr(Instruction::ClassStaticDecl(static_idx));
        }

        // Issue #982: SOP trait/protocol enforcement — emit TraitCheck for each `implements` clause
        for trait_name in &class_decl.implements {
            if let Some(required_methods) = self.known_traits.get(trait_name) {
                let req_syms: Vec<SymId> = required_methods.iter().map(|m| sym(m)).collect();
                let cls_syms: Vec<SymId> = all_class_methods.iter().map(|m| sym(m)).collect();
                let trait_idx = self.bytecode.add_trait_check_payload(
                    hudhudscript_bytecode::TraitCheckPayload {
                        class_name: name_sym,
                        trait_name: sym(trait_name),
                        required_methods: req_syms,
                        class_methods: cls_syms,
                    },
                );
                self.bytecode.push_instr(Instruction::TraitCheck(trait_idx));
            }
            // If the trait is not yet known at compile time, the check will be
            // skipped (forward-declared or imported traits can't be verified statically).
        }

        Ok(())
    }
}
