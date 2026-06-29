#![allow(unused_imports)]

use super::*;

impl VM {
    #[inline]
    pub(crate) fn step_rag(
        &mut self,
        instr: &Instruction,
        ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        let bytecode = ctx.bytecode;

        match instr {
            Instruction::Remember {
                store_idx: payload_idx,
                src,
                ..
            } => {
                // #892: Delegate to hudhudscript_rag::VectorStore
                // CROSS-2d: opt-sym payload carries the optional store name.
                let payload = bytecode.get_opt_sym_payload(*payload_idx as u32);
                let store_name_sym = payload.sym;
                let content = self.registers[*src as usize];
                let store_resolved = store_name_sym
                    .as_ref()
                    .map(|s| bytecode.resolve_symbol(s.0));
                let store_key = store_resolved.as_deref().unwrap_or("default");
                let text = self.value_to_string(&content);

                // Lazily create a VectorStore for this store name
                let dims = self.rag_embedder.dimensions();
                if !self.rag_stores.contains_key(store_key) {
                    let new_store = VectorStore::new(VectorStoreConfig {
                        name: store_key.to_string(),
                        dimensions: dims,
                        distance_metric: DistanceMetric::Cosine,
                        persist_path: None,
                    })
                    .map_err(|e| {
                        compile_codes::runtime_error(format!("Failed to create VectorStore: {e}"))
                    })?;
                    // Issue #979: evict if cache exceeds limit before inserting
                    enforce_cache_limit(&mut self.rag_stores, MAX_RAG_STORE_CACHE);
                    self.rag_stores.insert(store_key.to_string(), new_store);
                }
                let store = self.rag_stores.get_mut(store_key).ok_or_else(|| {
                    compile_codes::runtime_error("VectorStore disappeared after insertion")
                })?;

                let embedding = self
                    .rag_embedder
                    .embed(&text)
                    .unwrap_or_else(|_| vec![0.0f32; dims]);
                let entry_id = store
                    .insert(&text, embedding, serde_json::json!({}))
                    // Default to empty string ID on insert failure
                    .unwrap_or_default();

                // Track text→id for forget-by-content
                self.rag_text_to_ids
                    .entry(store_key.to_string())
                    .or_default()
                    .entry(text)
                    .or_default()
                    .push(entry_id);

                // Also maintain __rag_store: variable for empty-query recall
                let var_key = format!("__rag_store:{}", store_key);
                let existing = self
                    .get_var_cloned(&var_key)
                    .unwrap_or(Value16::array(vec![]));
                if let Some(arr) = existing.as_array() {
                    let mut new_arr = arr.clone();
                    new_arr.push(content);
                    self.set_global(&var_key, Value16::array(new_arr));
                }
            }
            Instruction::Recall {
                store_idx: payload_idx,
                src,
                dst,
            } => {
                // CROSS-2d: opt-sym payload carries the optional store name.
                let payload = bytecode.get_opt_sym_payload(*payload_idx as u32);
                let store_name_sym = payload.sym;
                // #892 / Kural 7 parity: cosine-similarity recall via
                // hudhudscript_rag::VectorStore. Matches the interpreter's
                // `Stmt::Recall` semantics (see interpreter/statement.rs:266):
                //
                //   - top-K = 5 (the interpreter's constant)
                //   - no score-based filter (ranked list is returned
                //     verbatim; callers can slice/threshold in script land)
                //   - each hit is a `{ id, text, score }` object, not a
                //     bare string
                //
                // Previously the VM used top-K = 10 and filtered results
                // where `r.score >= 1.0`, which with SimpleEmbedding's
                // near-orthogonal vectors silently dropped all results in
                // the `vm_remember_*` regression tests. Empty-query recall
                // still short-circuits through the `__rag_store:<name>`
                // variable so `recall("")` returns every stored item
                // (matches the interpreter's "return all" shortcut).
                let query = self.registers[*src as usize];
                let query_str = self.value_to_string(&query);
                let store_resolved = store_name_sym
                    .as_ref()
                    .map(|s| bytecode.resolve_symbol(s.0));
                let store_key = store_resolved.as_deref().unwrap_or("default");

                let results = if query_str.trim().is_empty() {
                    let key = format!("__rag_store:{}", store_key);
                    let items = self.get_var_cloned(&key).unwrap_or(Value16::array(vec![]));
                    let items_v = items;
                    if let Some(arr) = items_v.as_array() {
                        arr.into_iter().map(|v| v.clone()).collect()
                    } else {
                        vec![]
                    }
                } else if let Some(store) = self.rag_stores.get(store_key) {
                    if store.is_empty() {
                        vec![]
                    } else {
                        let dims = self.rag_embedder.dimensions();
                        let query_emb = self
                            .rag_embedder
                            .embed(&query_str)
                            .unwrap_or_else(|_| vec![0.0f32; dims]);

                        const TOP_K: usize = 5;
                        match store.query(&query_emb, TOP_K) {
                            Ok(search_results) => search_results
                                .into_iter()
                                .map(|r| {
                                    let mut obj = hudhudscript_bytecode::ObjMap::default();
                                    obj.insert("id".to_string(), Value16::string(r.id));
                                    obj.insert("text".to_string(), Value16::string(r.text));
                                    obj.insert(
                                        "score".to_string(),
                                        Value16::number(r.score as f64),
                                    );
                                    Value16::object(obj)
                                })
                                .collect(),
                            Err(_) => vec![],
                        }
                    }
                } else {
                    vec![]
                };

                self.registers[*dst as usize] = Value16::array(results);
            }
            Instruction::Forget {
                store_idx: payload_idx,
                src,
                ..
            } => {
                // #892: Real targeted forget — delegated to VectorStore
                // CROSS-2d: opt-sym payload carries the optional store name.
                let payload = bytecode.get_opt_sym_payload(*payload_idx as u32);
                let store_name_sym = payload.sym;
                let target = self.registers[*src as usize];
                let target_str = self.value_to_string(&target);
                let store_resolved = store_name_sym
                    .as_ref()
                    .map(|s| bytecode.resolve_symbol(s.0));
                let store_key = store_resolved.as_deref().unwrap_or("default");
                let var_key = format!("__rag_store:{}", store_key);

                if target_str.is_empty() {
                    // Empty target = clear all
                    self.set_global(&var_key, Value16::array(vec![]));
                    self.rag_stores.remove(store_key);
                    self.rag_text_to_ids.remove(store_key);
                } else {
                    // Remove items matching the target text from the backing variable
                    if let Some(var_val) = self.get_var_cloned(&var_key) {
                        if let Some(arr) = var_val.as_array() {
                            let filtered: Vec<Value16> = arr
                                .iter()
                                .filter(|item| self.value_to_string(*item) != target_str)
                                .cloned()
                                .collect();
                            self.set_global(&var_key, Value16::array(filtered));
                        }
                    }
                    // Delete matching entries from VectorStore by looking up tracked IDs
                    // Default to empty vec if text was never inserted
                    let ids_to_delete: Vec<String> = self
                        .rag_text_to_ids
                        .get_mut(store_key)
                        .and_then(|text_map| text_map.remove(&target_str))
                        .unwrap_or_default();
                    if let Some(store) = self.rag_stores.get_mut(store_key) {
                        for id in &ids_to_delete {
                            store.delete(id);
                        }
                    }
                }
            }

            _ => unreachable!("instruction routed to wrong execute helper"),
        }

        Ok(StepAction::Advance)
    }
}
