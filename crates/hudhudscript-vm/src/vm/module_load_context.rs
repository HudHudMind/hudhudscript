//! G08: circular module import guard.
//!
//! `ModuleLoadGuard` is an RAII marker: it holds no lock while the module
//! body executes (the shared context mutex is only taken on enter/drop), so
//! nested loads never deadlock. A repeated active identity is reported as a
//! cycle with the full load chain.

use hudhudscript_bytecode::error::compile_codes;
use parking_lot::Mutex;
use rustc_hash::FxHashSet;
use std::sync::Arc;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub(crate) struct ModuleIdentity(pub(crate) String);

#[derive(Default)]
pub(crate) struct ModuleLoadContext {
    pub(crate) active: FxHashSet<ModuleIdentity>,
    pub(crate) chain: Vec<ModuleIdentity>,
}

pub(crate) struct ModuleLoadGuard {
    context: Arc<Mutex<ModuleLoadContext>>,
    identity: ModuleIdentity,
}

impl ModuleLoadGuard {
    pub(crate) fn enter(
        context: Arc<Mutex<ModuleLoadContext>>,
        identity: ModuleIdentity,
    ) -> Result<Self, hudhudscript_errors::Error> {
        {
            let mut state = context.lock();
            if state.active.contains(&identity) {
                let mut chain: Vec<String> =
                    state.chain.iter().map(|item| item.0.clone()).collect();
                chain.push(identity.0.clone());
                return Err(compile_codes::runtime_error(format!(
                    "Circular module import: {}",
                    chain.join(" -> ")
                )));
            }
            state.active.insert(identity.clone());
            state.chain.push(identity.clone());
        }
        Ok(Self { context, identity })
    }
}

impl Drop for ModuleLoadGuard {
    fn drop(&mut self) {
        let mut state = self.context.lock();
        state.active.remove(&self.identity);
        if state.chain.last() == Some(&self.identity) {
            state.chain.pop();
        } else if let Some(index) = state.chain.iter().rposition(|v| v == &self.identity) {
            state.chain.remove(index);
        }
    }
}
