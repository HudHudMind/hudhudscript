use super::*;

mod basic;
mod function_context;
pub(crate) mod function_optimizer;
mod governance;
mod helpers;
pub(crate) mod helpers_loop;
pub mod loop_compile;
pub mod loop_engine;
pub mod loop_symbols;
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
            Decl::AgentAction {
                agent_name,
                name,
                params,
                body,
                is_async,
                ..
            } => {
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
            Decl::Ability {
                name,
                subject_type,
                params,
                body,
                ..
            } => {
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
                    .add_function(chunk_name.clone(), Arc::new(fn_chunk));
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
            Decl::Compose {
                base_subject,
                rules,
                field_rules,
                ..
            } => {
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
                event_name,
                params,
                body,
                ..
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
            // ── Loop engineering (FAZ D) ──
            Decl::Loop {
                name,
                items,
                mode,
                goal,
                ..
            } => {
                // A3: inject attached steps before compiling
                let mut augmented_items: Vec<LoopItemAst> = items.clone();
                if let Some(extra) = self.attach_step_queue.remove(name) {
                    augmented_items.extend(extra);
                }
                self.compile_decl_loop(name, &augmented_items, mode, goal.as_ref())?;
            }
            Decl::Chain {
                name, links, mode, ..
            } => {
                // A3: inject attached loops before compiling
                let mut augmented_links: Vec<ChainLinkAst> = links.clone();
                if let Some(extra) = self.attach_loop_queue.remove(name) {
                    for (loop_name, on_done, on_fail) in extra {
                        augmented_links.push(ChainLinkAst {
                            loop_name,
                            inline_loop: None,
                            on_done: on_done.unwrap_or(ChainTargetAst::Next),
                            on_fail: on_fail.unwrap_or(ChainTargetAst::ChainFail),
                        });
                    }
                }
                self.compile_decl_chain(name, &augmented_links, mode)?;
            }
            Decl::RunLoop { name, .. } => {
                self.compile_run_loop(name)?;
            }
            Decl::RunChain { name, .. } => {
                self.compile_run_chain(name)?;
            }
            Decl::Step {
                name,
                params,
                body,
                gate,
                ..
            } => {
                // A2: register standalone step for use_step / attach resolution
                if self.step_registry.contains_key(name) {
                    return Err(compile_codes::generic(format!(
                        "duplicate step: '{}'",
                        name
                    )));
                }
                self.step_registry
                    .insert(name.clone(), (params.clone(), body.clone(), gate.clone()));
            }
            Decl::Gate {
                name,
                branches,
                else_target,
                ..
            } => {
                // FAZ G: register gate for later AttachGate resolution
                self.gate_registry
                    .insert(name.clone(), (branches.clone(), else_target.clone()));
            }
            Decl::AttachStep {
                targets, loop_name, ..
            } => {
                // A3: collect attached steps for later injection during loop compilation
                for t in targets {
                    if let Some((params, body, gate)) = self.step_registry.get(&t.step).cloned() {
                        let item = LoopItemAst::InlineStep(Box::new(Decl::Step {
                            name: t.step.clone(),
                            params,
                            body,
                            gate: if let Some(ref gname) = t.gate {
                                self.gate_registry.get(gname).map(|(b, e)| StepGateAst {
                                    name: gname.clone(),
                                    branches: b.clone(),
                                    else_target: e.clone(),
                                })
                            } else {
                                gate
                            },
                            span: hudhudscript_ast::Span::default(),
                        }));
                        self.attach_step_queue
                            .entry(loop_name.clone())
                            .or_default()
                            .push(item);
                    } else {
                        return Err(compile_codes::generic(format!(
                            "attach step: unknown step '{}'",
                            t.step
                        )));
                    }
                }
            }
            Decl::AttachLoop {
                loop_name,
                chain_name,
                on_done,
                on_fail,
                ..
            } => {
                self.attach_loop_queue
                    .entry(chain_name.clone())
                    .or_default()
                    .push((loop_name.clone(), on_done.clone(), on_fail.clone()));
            }
            _ => {}
        }
        Ok(())
    }
}
