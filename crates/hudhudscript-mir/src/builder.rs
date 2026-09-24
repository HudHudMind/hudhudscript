//! MIR function builder (linear SSA within blocks).
//!
//! AŞAMA-0 scope: values are created in monotonically increasing order
//! inside a block; block parameters (phi) arrive with full-CFG support.
//! The builder refuses structural misuse by panicking with clear
//! messages — it is a compiler-internal tool, not a trust boundary
//! (untrusted input is caught by the verifier before codegen).

use std::sync::Arc;

use crate::mir::{
    BlockId, CmpOp, FunctionId, MirBlock, MirFunction, MirInst, MirTerminator, MirType,
    RuntimeHelperId, TrapKind, ValueId,
};

pub struct MirFunctionBuilder {
    name: Arc<str>,
    param_tys: Vec<MirType>,
    return_ty: MirType,
    blocks: Vec<MirBlock>,
    next_value: u32,
    terminated: Vec<bool>,
    function_names: Vec<Arc<str>>,
    /// CallStatic'de kullanılan function id'leri → isim (dışarıdan verilir)
    pub known_function_names: Vec<Arc<str>>,
    /// String sabitleri: ConstString index'leri buraya bakar.
    pub string_table: Vec<Arc<str>>,
}

impl MirFunctionBuilder {
    pub fn new(name: &str, param_tys: Vec<MirType>, return_ty: MirType) -> Self {
        let entry = MirBlock { id: BlockId(0), params: Vec::new(), insts: Vec::new(), terminator: None };
        Self {
            name: Arc::from(name),
            param_tys,
            return_ty,
            blocks: vec![entry],
            next_value: 0,
            terminated: vec![false],
            function_names: vec![],
            known_function_names: vec![],
            string_table: vec![],
        }
    }

    /// Harici bir fonksiyon adını kaydet; FunctionId döner.
    /// CallStatic'de callee olarak kullanılır; finish() sırasında
    /// function_names'e yazılır.
    pub fn register_external_function(&mut self, name: &str) -> crate::mir::FunctionId {
        self.known_function_names.push(Arc::from(name));
        crate::mir::FunctionId((self.known_function_names.len() - 1) as u32)
    }

    pub fn entry(&self) -> BlockId {
        BlockId(0)
    }

    /// Parametreli blok oluştur; her param için benzersiz ValueId döner.
    pub fn create_block_with_params(&mut self, params: Vec<MirType>) -> (BlockId, Vec<ValueId>) {
        let id = BlockId(self.blocks.len() as u32);
        let mut param_values = Vec::with_capacity(params.len());
        let mut block_params = Vec::with_capacity(params.len());
        for ty in params {
            let v = self.fresh();
            param_values.push(v);
            block_params.push((ty, v));
        }
        self.blocks.push(MirBlock {
            id,
            params: block_params,
            insts: Vec::new(),
            terminator: None,
        });
        self.terminated.push(false);
        (id, param_values)
    }

    pub fn create_block(&mut self) -> BlockId {
        let id = BlockId(self.blocks.len() as u32);
        self.blocks.push(MirBlock { id, params: Vec::new(), insts: Vec::new(), terminator: None });
        self.terminated.push(false);
        id
    }

    fn fresh(&mut self) -> ValueId {
        let v = ValueId(self.next_value);
        self.next_value += 1;
        v
    }

    fn push(&mut self, block: BlockId, inst: MirInst) {
        assert!(!self.terminated[block.as_usize()], "block {block} already terminated");
        self.blocks[block.as_usize()].insts.push(inst);
    }

    // ── constants ──
    pub fn const_i64(&mut self, b: BlockId, value: i64) -> ValueId {
        let d = self.fresh();
        self.push(b, MirInst::ConstInt { dst: d, ty: MirType::I64, value });
        d
    }

    pub fn const_f64(&mut self, b: BlockId, value: f64) -> ValueId {
        let d = self.fresh();
        self.push(
            b,
            MirInst::ConstFloat { dst: d, ty: MirType::F64, bits: value.to_bits() },
        );
        d
    }

    pub fn const_bool(&mut self, b: BlockId, value: bool) -> ValueId {
        let d = self.fresh();
        self.push(b, MirInst::ConstBool { dst: d, value });
        d
    }

    pub fn const_null(&mut self, b: BlockId) -> ValueId {
        let d = self.fresh();
        self.push(b, MirInst::ConstNull { dst: d });
        d
    }

    /// String constant: fonksiyonun string_table'ına ekler ve ConstString üretir.
    pub fn const_string(&mut self, b: BlockId, value: &str) -> ValueId {
        let d = self.fresh();
        let index = self.string_table.len() as u32;
        self.string_table.push(Arc::from(value));
        self.push(b, MirInst::ConstString { dst: d, index });
        d
    }

    // ── string operations ──
    pub fn string_concat(&mut self, b: BlockId, lhs: ValueId, rhs: ValueId) -> ValueId {
        let d = self.fresh();
        self.push(b, MirInst::StringConcat { dst: d, lhs, rhs });
        d
    }

    pub fn string_len(&mut self, b: BlockId, src: ValueId) -> ValueId {
        let d = self.fresh();
        self.push(b, MirInst::StringLen { dst: d, src });
        d
    }

    pub fn string_eq(&mut self, b: BlockId, lhs: ValueId, rhs: ValueId) -> ValueId {
        let d = self.fresh();
        self.push(b, MirInst::StringEq { dst: d, lhs, rhs });
        d
    }

    pub fn int_to_string(&mut self, b: BlockId, src: ValueId) -> ValueId {
        let d = self.fresh();
        self.push(b, MirInst::IntToString { dst: d, src });
        d
    }

    pub fn float_to_string(&mut self, b: BlockId, src: ValueId) -> ValueId {
        let d = self.fresh();
        self.push(b, MirInst::FloatToString { dst: d, src });
        d
    }

    pub fn string_substring(&mut self, b: BlockId, s: ValueId, start: ValueId, end: ValueId) -> ValueId {
        let d = self.fresh();
        self.push(b, MirInst::StringSubstring { dst: d, s, start, end });
        d
    }

    pub fn int_to_float(&mut self, b: BlockId, src: ValueId) -> ValueId {
        let d = self.fresh();
        self.push(b, MirInst::IntToFloat { dst: d, src });
        d
    }

    // ── array operations ──
    pub fn array_new(&mut self, b: BlockId, capacity: ValueId) -> ValueId {
        let d = self.fresh();
        self.push(b, MirInst::ArrayNew { dst: d, capacity });
        d
    }

    pub fn array_push(&mut self, b: BlockId, arr: ValueId, value: ValueId) {
        self.push(b, MirInst::ArrayPush { arr, value });
    }

    pub fn array_get(&mut self, b: BlockId, ty: MirType, arr: ValueId, index: ValueId) -> ValueId {
        let d = self.fresh();
        self.push(b, MirInst::ArrayGet { dst: d, ty, arr, index });
        d
    }

    pub fn array_set(&mut self, b: BlockId, arr: ValueId, index: ValueId, value: ValueId) {
        self.push(b, MirInst::ArraySet { arr, index, value });
    }

    pub fn array_pop(&mut self, b: BlockId, arr: ValueId) -> ValueId {
        let d = self.fresh();
        self.push(b, MirInst::ArrayPop { dst: d, arr });
        d
    }

    pub fn string_char_at(&mut self, b: BlockId, s: ValueId, index: ValueId) -> ValueId {
        let d = self.fresh();
        self.push(b, MirInst::StringCharAt { dst: d, s, index });
        d
    }

    pub fn array_len(&mut self, b: BlockId, arr: ValueId) -> ValueId {
        let d = self.fresh();
        self.push(b, MirInst::ArrayLen { dst: d, arr });
        d
    }

    pub fn array_join(&mut self, b: BlockId, arr: ValueId, sep: ValueId) -> ValueId {
        let d = self.fresh();
        self.push(b, MirInst::ArrayJoin { dst: d, arr, sep });
        d
    }

    pub fn call_native_f2(&mut self, b: BlockId, helper: crate::mir::RuntimeHelperId, a: ValueId, bb: ValueId) -> ValueId {
        let d = self.fresh();
        self.push(b, MirInst::CallNative { dst: d, ty: MirType::F64, helper, args: vec![a, bb] });
        d
    }

    pub fn logical_and(&mut self, b: BlockId, lhs: ValueId, rhs: ValueId) -> ValueId {
        let d = self.fresh();
        self.push(b, MirInst::LogicalAnd { dst: d, lhs, rhs });
        d
    }

    pub fn logical_or(&mut self, b: BlockId, lhs: ValueId, rhs: ValueId) -> ValueId {
        let d = self.fresh();
        self.push(b, MirInst::LogicalOr { dst: d, lhs, rhs });
        d
    }

    // ── object operations ──
    pub fn object_new(&mut self, b: BlockId) -> ValueId {
        let d = self.fresh();
        self.push(b, MirInst::ObjectNew { dst: d });
        d
    }

    pub fn object_set(&mut self, b: BlockId, obj: ValueId, key: ValueId, value: ValueId) {
        self.push(b, MirInst::ObjectSet { obj, key, value });
    }

    pub fn object_get(&mut self, b: BlockId, ty: MirType, obj: ValueId, key: ValueId) -> ValueId {
        let d = self.fresh();
        self.push(b, MirInst::ObjectGet { dst: d, ty, obj, key });
        d
    }

    pub fn object_has(&mut self, b: BlockId, obj: ValueId, key: ValueId) -> ValueId {
        let d = self.fresh();
        self.push(b, MirInst::ObjectHas { dst: d, obj, key });
        d
    }

    pub fn object_len(&mut self, b: BlockId, obj: ValueId) -> ValueId {
        let d = self.fresh();
        self.push(b, MirInst::ObjectLen { dst: d, obj });
        d
    }

    /// i. parametreyi tanımlar (fonksiyonun param_tys[index] ile aynı tip).
    pub fn param(&mut self, b: BlockId, index: u32) -> ValueId {
        let ty = self.param_tys[index as usize];
        let d = self.fresh();
        self.push(b, MirInst::Param { dst: d, ty, index });
        d
    }

    // ── arithmetic ──
    pub fn bin(&mut self, b: BlockId, op: BinOp, ty: MirType, lhs: ValueId, rhs: ValueId) -> ValueId {
        let d = self.fresh();
        let inst = match op {
            BinOp::Add => MirInst::Add { dst: d, ty, lhs, rhs },
            BinOp::Sub => MirInst::Sub { dst: d, ty, lhs, rhs },
            BinOp::Mul => MirInst::Mul { dst: d, ty, lhs, rhs },
            BinOp::Div => MirInst::Div { dst: d, ty, lhs, rhs },
            BinOp::Rem => MirInst::Rem { dst: d, ty, lhs, rhs },
        };
        self.push(b, inst);
        d
    }

    pub fn cmp(&mut self, b: BlockId, op: CmpOp, ty: MirType, lhs: ValueId, rhs: ValueId) -> ValueId {
        let d = self.fresh();
        self.push(b, MirInst::Cmp { dst: d, op, ty, lhs, rhs });
        d
    }

    // ── calls ──
    /// Declare a CallStatic target by name; returns its FunctionId.
    pub fn declare_function(&mut self, name: &str) -> FunctionId {
        self.function_names.push(Arc::from(name));
        FunctionId((self.function_names.len() - 1) as u32)
    }

    pub fn call_static(&mut self, b: BlockId, ty: MirType, callee: FunctionId, args: Vec<ValueId>) -> ValueId {
        let d = self.fresh();
        self.push(b, MirInst::CallStatic { dst: d, ty, callee, args });
        d
    }

    pub fn call_native(&mut self, b: BlockId, ty: MirType, helper: RuntimeHelperId, args: Vec<ValueId>) -> ValueId {
        let d = self.fresh();
        self.push(b, MirInst::CallNative { dst: d, ty, helper, args });
        d
    }

    // ── runtime/GC ──
    pub fn gc_safepoint(&mut self, b: BlockId) {
        self.push(b, MirInst::GcSafepoint);
    }

    pub fn trap(&mut self, b: BlockId, kind: TrapKind) {
        self.push(b, MirInst::Trap { kind });
    }

    // ── terminators ──
    pub fn ret(&mut self, b: BlockId, value: ValueId) {
        self.terminate(b, MirTerminator::Return(value));
    }

    pub fn ret_void(&mut self, b: BlockId) {
        self.terminate(b, MirTerminator::ReturnVoid);
    }

    pub fn branch(&mut self, b: BlockId, target: BlockId) {
        self.terminate(b, MirTerminator::Branch { target, args: Vec::new() });
    }

    pub fn branch_with_args(&mut self, b: BlockId, target: BlockId, args: Vec<ValueId>) {
        self.terminate(b, MirTerminator::Branch { target, args });
    }

    pub fn cond_branch(&mut self, b: BlockId, cond: ValueId, then_block: BlockId, else_block: BlockId) {
        self.terminate(b, MirTerminator::CondBranch { cond, then_block, then_args: Vec::new(), else_block, else_args: Vec::new() });
    }

    pub fn cond_branch_with_args(
        &mut self,
        b: BlockId,
        cond: ValueId,
        then_block: BlockId,
        then_args: Vec<ValueId>,
        else_block: BlockId,
        else_args: Vec<ValueId>,
    ) {
        self.terminate(b, MirTerminator::CondBranch { cond, then_block, then_args, else_block, else_args });
    }

    fn terminate(&mut self, b: BlockId, t: MirTerminator) {
        assert!(!self.terminated[b.as_usize()], "block {b} already terminated");
        self.blocks[b.as_usize()].terminator = Some(t);
        self.terminated[b.as_usize()] = true;
    }

    pub fn finish(self) -> MirFunction {
        MirFunction {
            name: self.name,
            param_tys: self.param_tys,
            return_ty: self.return_ty,
            entry: BlockId(0),
            blocks: self.blocks,
            function_names: Arc::new(self.known_function_names),
            string_table: Arc::new(self.string_table),
        }
    }
}

pub use crate::types::BinOp;

