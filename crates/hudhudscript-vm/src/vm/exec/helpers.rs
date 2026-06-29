use crate::vm::prepack::prepack_instructions;
use crate::vm::VM;
use crate::vm::{numeric_slot, GenStep, NumericSlot};
use hudhudscript_bytecode::error::compile_codes;
use hudhudscript_bytecode::error::{CompileError, CompileResult};
use hudhudscript_bytecode::FunctionData;
use hudhudscript_bytecode::Value16;
use hudhudscript_bytecode::{Bytecode, FunctionChunk};
use std::sync::Arc;

impl VM {
    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    pub(crate) fn pop_number(&mut self, reg: u8) -> CompileResult<f64> {
        let v = self.registers[reg as usize];
        if let Some(n) = v.as_number() {
            Ok(n)
        } else if let Some(i) = v.as_int() {
            Ok(i as f64)
        } else {
            Err(compile_codes::runtime_error(format!(
                "Expected number, got {:?}",
                v
            )))
        }
    }

    /// #P2-7 helper: pull one value from the top generator frame and bind
    /// it to the iterator variable. Kept out of the IterNext dispatch body
    /// so the frame stays small in debug builds (Value enum bloats frames).
    pub(crate) fn step_generator_iter(&mut self) -> GenStep {
        let state = match self.iterator_generators.last() {
            Some(Some(s)) => s.clone(),
            _ => return GenStep::Exhausted,
        };
        let next = crate::vm::exec::helpers::generator_advance(self, &state);
        match next {
            Some(val) => {
                let var_name = self
                    .iterators
                    .last()
                    .map(|(_, n, _)| n.clone())
                    .unwrap_or_default();
                let sym_id = hudhudscript_bytecode::interner::intern(var_name.as_str()).0;
                let mut stored = false;
                if let Some(local_syms_ptr) = self.call_stack_local_syms.last() {
                    let local_syms = unsafe { &**local_syms_ptr };
                    if let Ok(idx) = local_syms.binary_search_by_key(&sym_id, |(s, _, _)| *s) {
                        let slot = local_syms[idx].1 as i32;
                        if slot >= 0 {
                            let local_idx = slot as usize;
                            if local_idx < self.registers.len() {
                                self.registers[local_idx] = val;
                                stored = true;
                            }
                        }
                    }
                }
                if !stored {
                    let sym = hudhudscript_bytecode::interner::intern(&var_name);
                    self.globals.insert(sym, val);
                }
                GenStep::Advanced
            }
            None => GenStep::Exhausted,
        }
    }

    pub(crate) fn values_equal(&self, left: &Value16, right: &Value16) -> bool {
        if let (Some(a), Some(b)) = (left.as_bool(), right.as_bool()) {
            a == b
        } else if let (Some(a), Some(b)) = (left.as_int(), right.as_int()) {
            a == b
        } else if let (Some(a), Some(b)) = (left.as_number(), right.as_number()) {
            a == b
        } else if let (Some(a), Some(b)) = (left.as_int(), right.as_number()) {
            (a as f64) == b
        } else if let (Some(a), Some(b)) = (left.as_number(), right.as_int()) {
            a == (b as f64)
        } else if let (Some(a), Some(b)) = (left.as_string(), right.as_string()) {
            a == b
        } else if let (Some(a), Some(b)) = (left.as_array(), right.as_array()) {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| self.values_equal(x, y))
        } else if let (Some(a), Some(b)) = (left.as_object(), right.as_object()) {
            a.len() == b.len()
                && a.iter()
                    .all(|(k, v)| b.get(&k).is_some_and(|v2| self.values_equal(v, v2)))
        } else if let (Some(a), Some(b)) = (left.as_promise_state(), right.as_promise_state()) {
            match (a, b) {
                (
                    hudhudscript_bytecode::PromiseState::Resolved(va),
                    hudhudscript_bytecode::PromiseState::Resolved(vb),
                ) => self.values_equal(va.as_ref(), vb.as_ref()),
                (
                    hudhudscript_bytecode::PromiseState::Pending,
                    hudhudscript_bytecode::PromiseState::Pending,
                ) => true,
                (
                    hudhudscript_bytecode::PromiseState::Rejected(a),
                    hudhudscript_bytecode::PromiseState::Rejected(b),
                ) => a == b,
                _ => false,
            }
        } else if let (Some(a), Some(b)) = (left.as_option(), right.as_option()) {
            match (a, b) {
                (None, None) => true,
                (Some(va), Some(vb)) => self.values_equal(va, vb),
                _ => false,
            }
        } else if let (Some(a), Some(b)) = (left.as_result(), right.as_result()) {
            match (a, b) {
                (Ok(va), Ok(vb)) => self.values_equal(va, vb),
                (Err(ea), Err(eb)) => ea == eb,
                _ => false,
            }
        } else if let (Some(a), Some(b)) = (left.as_set(), right.as_set()) {
            a.len() == b.len()
                && a.iter()
                    .all(|va| b.iter().any(|vb| self.values_equal(va, vb)))
        } else if let (Some(a), Some(b)) = (left.as_map_pairs(), right.as_map_pairs()) {
            a.len() == b.len()
                && a.iter().all(|(ka, va)| {
                    b.iter()
                        .any(|(kb, vb)| self.values_equal(ka, kb) && self.values_equal(va, vb))
                })
        } else if left.is_null() && right.is_null() {
            true
        } else {
            false
        }
    }

    #[inline(always)]
    pub(crate) fn is_truthy(&self, val: &Value16) -> bool {
        val.is_truthy()
    }
}

// ── V2-B: Generator yield — TEK tüketim noktası (Kural 7) ──

use hudhudscript_bytecode::GeneratorState16;
use parking_lot::Mutex;

/// Advance a generator by one step. Yield-tabanlı ise VM receiver'dan
/// gc::attach ile ana heap'e kopyalanır. Precomputed ise eski advance().
pub(crate) fn generator_advance(
    vm: &mut crate::vm::VM,
    state: &Arc<Mutex<GeneratorState16>>,
) -> Option<hudhudscript_bytecode::Value16> {
    let id = state.lock().yield_id;
    match id {
        Some(id) => {
            let tree = match vm.yield_receivers.get(&id)?.recv() {
                Ok(tree) => tree,
                Err(_) => {
                    // Generator thread finished — cleanup receiver
                    vm.yield_receivers.remove(&id);
                    return None;
                }
            };
            let val = hudhudscript_bytecode::gc_detach::attach(&tree);
            state.lock().buffered.push(val);
            Some(val)
        }
        None => state.lock().advance(),
    }
}

#[cfg(test)]
mod generator_tests {
    use super::*;
    use crate::vm::VM;
    use hudhudscript_bytecode::{gc_detach, Value16};

    #[test]
    fn advance_precomputed_uses_original_advance() {
        let mut vm = VM::new();
        let state = Arc::new(Mutex::new(GeneratorState16::from(vec![
            Value16::int(10),
            Value16::int(20),
        ])));
        // yield_id=None → uses advance()
        let v1 = generator_advance(&mut vm, &state);
        let v2 = generator_advance(&mut vm, &state);
        let v3 = generator_advance(&mut vm, &state);
        assert_eq!(v1.and_then(|v| v.as_int()), Some(10));
        assert_eq!(v2.and_then(|v| v.as_int()), Some(20));
        assert!(v3.is_none());
        // Check buffered
        let st = state.lock();
        assert_eq!(st.buffered.len(), 2);
    }

    #[test]
    fn advance_yield_based_attaches_to_caller_heap() {
        let mut vm = VM::new();
        let (tx, rx) = std::sync::mpsc::channel();
        let tree = gc_detach::detach(Value16::string("yielded-long-string")).unwrap();
        tx.send(tree).unwrap();
        drop(tx); // close channel

        let state = Arc::new(Mutex::new({
            let mut s = GeneratorState16::new(std::sync::mpsc::sync_channel::<Value16>(0).1);
            s.yield_id = Some(0);
            s
        }));
        vm.yield_receivers.insert(0, rx);

        let val = generator_advance(&mut vm, &state);
        assert!(val.is_some());
        let v = val.unwrap();
        assert_eq!(v.as_str(), Some("yielded-long-string"));

        // Check buffered was populated
        let st = state.lock();
        assert_eq!(st.buffered.len(), 1);
        assert_eq!(st.buffered[0].as_str(), Some("yielded-long-string"));
    }
}
