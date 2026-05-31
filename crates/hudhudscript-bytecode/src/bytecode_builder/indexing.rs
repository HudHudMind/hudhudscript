use crate::Bytecode;

impl Bytecode {
    /// Rebuild the reverse indices from the forward tables.  Called
    /// lazily after deserialization when the skip-serialized indices
    /// are empty but the forward tables are not.
    pub(crate) fn rebuild_indices_if_stale(&mut self) {
        if self.symbol_index.is_empty() && !self.symbols.is_empty() {
            self.symbol_index.reserve(self.symbols.len());
            for (i, s) in self.symbols.iter().enumerate() {
                self.symbol_index.insert(s.clone(), i as u32);
            }
        }
        if self.numeric_index.is_empty() && !self.numeric_constants.is_empty() {
            self.numeric_index.reserve(self.numeric_constants.len());
            for (i, bits) in self.numeric_constants.iter().enumerate() {
                self.numeric_index.insert(*bits, i as u32);
            }
        }
        if self.int_index.is_empty() && !self.int_constants.is_empty() {
            self.int_index.reserve(self.int_constants.len());
            for (i, v) in self.int_constants.iter().enumerate() {
                self.int_index.insert(*v, i as u32);
            }
        }
        if self.symbol_list_index.is_empty() && !self.symbol_lists.is_empty() {
            self.symbol_list_index.reserve(self.symbol_lists.len());
            for (i, list) in self.symbol_lists.iter().enumerate() {
                self.symbol_list_index.insert(list.clone(), i as u32);
            }
        }
    }
}
