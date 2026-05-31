use super::*;

mod basic;
mod governance;
mod helpers;
mod protocol;
mod sop;
mod store_domain;
mod swarm;
mod ui_deploy;

impl Compiler {
    pub(super) fn compile_decl(&mut self, decl: &Decl) -> CompileResult<()> {
        match decl {
            Decl::Import { module, alias, .. } => {
                self.compile_decl_import(module, alias.as_ref())?;
            }

            Decl::Agent { name, fields, .. } => {
                self.compile_decl_agent(name, fields)?;
            }
            Decl::AgentAction { agent_name, name, params, body, is_async, .. } => {
                let qualified_name = format!("{}.{}", agent_name, name);
                let fn_chunk = self.compile_function_body_named_async(
                    params.clone(),
                    Some(qualified_name.clone()),
                    body,
                    *is_async,
                )?;
                self.bytecode
                    .action_registry
                    .borrow_mut()
                    .insert(qualified_name, Arc::new(fn_chunk));
            }
            Decl::Ability { name, subject_type, params, body, .. } => {
                // P2.3: on ability — composites effects with context. Should use `self`.
                if params.first().map(|s| s.as_str()) != Some("self") {
                    eprintln!(
                        "[SOP] Warning: ability '{}' missing `self` as first parameter. \
                         Abilities operate on subject context; add `self` parameter.",
                        name
                    );
                }

                let chunk_name = if let Some(ref subj) = subject_type {
                    format!("ability::{}::{}", subj, name)
                } else {
                    format!("ability::{}", name)
                };
                let fn_chunk = self.compile_function_body(params.clone(), body)?;
                self.bytecode
                    .functions
                    .borrow_mut()
                    .insert(chunk_name.clone(), Arc::new(fn_chunk));
            }
            Decl::Action { name, fields, .. } => {
                self.compile_decl_action(name, fields)?;
            }
            Decl::Tool { name, fields, .. } => {
                self.compile_decl_tool(name, fields)?;
            }
            Decl::Resource { name, fields, .. } => {
                self.compile_decl_resource(name, fields)?;
            }
            Decl::Provider { name, fields, .. } => {
                self.compile_decl_provider(name, fields)?;
            }

            Decl::Constitution {
                name,
                description,
                laws,
                ..
            } => {
                self.compile_decl_constitution(name, description.as_ref(), laws)?;
            }
            Decl::Law {
                name,
                description,
                enforcement_level,
                rules,
                ..
            } => {
                self.compile_decl_law(name, description, enforcement_level, rules)?;
            }
            Decl::Council {
                name,
                constitution,
                members,
                rules,
                ..
            } => {
                self.compile_decl_council(name, constitution, members, rules)?;
            }
            Decl::Rule {
                name,
                conditions,
                actions,
                priority,
                ..
            } => {
                self.compile_decl_rule(name, conditions, actions, *priority)?;
            }
            Decl::Governance {
                name,
                base_type,
                fields,
                ..
            } => {
                self.compile_decl_governance(name, base_type, fields)?;
            }

            Decl::Swarm {
                name,
                agents,
                strategy,
                ..
            } => {
                self.compile_decl_swarm(name, agents, strategy)?;
            }
            Decl::Community {
                name,
                members,
                councils,
                culture,
                ..
            } => {
                self.compile_decl_community(name, members, councils, culture)?;
            }

            Decl::Protocol {
                name,
                execution,
                governance,
                timeout,
                session,
                ..
            } => {
                self.compile_decl_protocol(
                    name,
                    execution.as_ref(),
                    governance.as_ref(),
                    timeout.as_ref(),
                    session,
                )?;
            }
            Decl::Strategy {
                name,
                execution,
                governance,
                timeout,
                permissions,
                realm,
                session,
                ..
            } => {
                self.compile_decl_strategy(
                    name,
                    execution.as_ref(),
                    governance.as_ref(),
                    timeout.as_ref(),
                    permissions,
                    realm.as_ref(),
                    session,
                )?;
            }

            Decl::Subject {
                name,
                decorators,
                of_subject,
                roles,
                states,
                capabilities,
                ability_defs,
                intents,
                uses,
                memory,
                perception,
                fields,
                ..
            } => {
                self.compile_decl_subject(
                    name,
                    decorators,
                    of_subject.clone(),
                    roles,
                    states,
                    capabilities,
                    ability_defs,
                    intents,
                    uses,
                    memory,
                    perception,
                    fields,
                )?;
            }
            Decl::Compose { base_subject, rules, field_rules, .. } => {
                self.compile_decl_compose(base_subject, rules, field_rules)?;
            }
            Decl::Role {
                name,
                capabilities,
                fields,
                ..
            } => {
                self.compile_decl_role(name, capabilities, fields)?;
            }
            Decl::Relation {
                subject_a,
                subject_b,
                fields,
                ..
            } => {
                self.compile_decl_relation(subject_a, subject_b, fields)?;
            }
            Decl::Effect {
                event_name, params, body, ..
            } => {
                self.compile_decl_effect(event_name, params, body)?;
            }

            Decl::Store { name, fields, .. } => {
                self.compile_decl_store(name, fields)?;
            }

            Decl::Entity { name, fields, .. } => {
                self.compile_decl_entity(name, fields)?;
            }
            Decl::StateMachine { name, fields, .. } => {
                self.compile_decl_statemachine(name, fields)?;
            }
            Decl::Event { name, fields, .. } => {
                self.compile_decl_event(name, fields)?;
            }
            Decl::Contract { name, fields, .. } => {
                self.compile_decl_contract(name, fields)?;
            }
            Decl::Treaty { name, fields, .. } => {
                self.compile_decl_treaty(name, fields)?;
            }
            Decl::Music {
                kind, name, fields, ..
            } => {
                self.compile_decl_music(kind, name, fields)?;
            }

            Decl::UiApp {
                name,
                entry_screen,
                screens,
                components,
                ..
            } => {
                self.compile_decl_uiapp(name, entry_screen.as_ref(), screens, components)?;
            }
            Decl::Deploy { name, fields, .. } => {
                self.compile_decl_deploy(name, fields)?;
            }
        }
        Ok(())
    }
}
