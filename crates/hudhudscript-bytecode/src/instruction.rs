use crate::{FunctionChunk, SymId, Value16, BYTECODE_VERSION};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instruction {
    Jump(i32),
    JumpIfFalse {
        src: u8,
        offset: i16,
    },
    JumpIfTrue {
        src: u8,
        offset: i16,
    },
    TailCall {
        func_reg: u8,
        first_arg_reg: u8,
        arg_count: u8,
    },
    IntLeJumpIfFalse(u32),
    IntLtJumpIfFalse(u32),
    IntSubCall1(u32),
    IntAddCall1(u32),
    EnumDecl(u32),
    MatchVariant(u32),
    BindVar(SymId),
    Break,
    Continue,
    ForIn {
        iter_reg: u8,
        var_sym_idx: u16,
        end_offset: i16,
    },
    IterNext {
        iter_reg: u8,
        var_sym_idx: u16,
        end_offset: i16,
    },
    TryBegin(i32),
    TryEnd,
    Throw {
        src: u8,
    },
    FinallyBegin(i32),
    FinallyEnd,
    FinallyExit(i32),
    LoopBegin(u32),
    LoopEnd,
    Spawn {
        /// B3: subject name SymId (not payload index — immune to table shifts)
        name_sym: u32,
        first_arg: u8,
        arg_count: u8,
    },
    Despawn {
        reg: u8,
    },
    ViewAs {
        obj: u8,
        view_sym: u16,
    },
    Send {
        message: u8,
        target: u8,
    },
    Receive {
        var_sym_idx: u16,
        src: u8,
    },
    Require {
        src: u8,
    },
    Perform {
        src: u8,
    },
    Await {
        src: u8,
        dst: u8,
    },
    Yield {
        src: u8,
    },
    ClassDecl(u32),
    NewInstance {
        payload_idx: u16,
        first_arg: u8,
        arg_count: u8,
    },
    TraitCheck(u32),
    LoadModule(u32),
    DefineFunction(u32),
    MethodCall {
        dst: u8,
        obj: u8,
        payload_idx: u16,
        first_arg: u8,
        arg_count: u8,
    },
    SuperCall {
        dst: u8,
        payload_idx: u16,
        first_arg: u8,
        arg_count: u8,
    },
    MakeGenerator {
        payload_idx: u16,
        first_arg: u8,
        arg_count: u8,
    },
    CallSpread(SymId),
    MethodCallSpread(SymId),
    GetProperty {
        dst: u8,
        obj: u8,
        prop_sym: u16,
    },
    GetStatic(u32),
    ClassStaticDecl(u32),
    DeclStore {
        payload_idx: u16,
        src: u8,
    },
    DestructArray(u16, bool),
    DestructObject(u32),
    Remember {
        store_idx: u16,
        src: u8,
    },
    Recall {
        store_idx: u16,
        src: u8,
        dst: u8,
    },
    Forget {
        store_idx: u16,
        src: u8,
    },
    StringConcat {
        regs_start: u8,
        count: u8,
        dst: u8,
    },
    ArrayPush {
        dst: u8,
        arr: u8,
        val: u8,
    },
    SpreadIntoArray {
        dst: u8,
        src: u8,
    },
    SpreadIntoObject {
        dst: u8,
        src: u8,
    },
    IntAdd {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    IntSub {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    IntMul {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    IntAddI {
        dst: u8,
        src: u8,
        imm: i16,
    },
    IntSubI {
        dst: u8,
        src: u8,
        imm: i16,
    },
    IntMulI { dst: u8, src: u8, imm: i16 },
    IntDivI { dst: u8, src: u8, imm: i16 },
    IntModI { dst: u8, src: u8, imm: i16 },
    /// Fused IntModI + IntCmpI: `(src % mod_imm) op cmp_imm` → dst (bool).
    /// Unpacked only — never enters dense/packed encoding.
    IntModCmpI {
        dst: u8,
        src: u8,
        mod_imm: i16,
        cmp_imm: i16,
        op: u8,
    },
    IntCmp {
        dst: u8,
        src1: u8,
        src2: u8,
        op: u8,
    },
    IntCmpI {
        dst: u8,
        src: u8,
        imm: i16,
        op: u8,
    },
    /// Branch fusion: compare src with immediate, jump if false.
    /// Replaces IntCmpI { dst, .. } + JumpIfFalse { src=dst, offset }.
    IntCmpIJumpIfFalse {
        src: u8,
        imm: i16,
        op: u8,
        offset: i16,
    },
    /// Branch fusion: compare src with immediate, jump if TRUE.
    IntCmpIJumpIfTrue {
        src: u8,
        imm: i16,
        op: u8,
        offset: i16,
    },
    /// Branch fusion: compare two registers, jump if false.
    /// Replaces IntCmp { dst, .. } + JumpIfFalse { src=dst, offset }.
    IntCmpRRJumpIfFalse {
        src1: u8,
        src2: u8,
        op: u8,
        offset: i16,
    },
    /// Loop tail fusion: `reg += imm` then jump.
    /// Replaces IntAddI { dst=reg, src=reg, imm } + Jump(offset).
    IntAddIJump {
        reg: u8,
        imm: i16,
        offset: i16,
    },
    /// LoopEnd + IntAddI + Jump fusion: pops loop frame, adds imm to reg, jumps.
    LoopEndIntAddIJump {
        reg: u8,
        imm: i16,
        offset: i16,
    },
    /// IntSubI (self-update) + Jump → IntSubIJump
    IntSubIJump {
        reg: u8,
        imm: i16,
        offset: i16,
    },
    /// C6: byte-indexed character dispatch for if-else-if chains of
    /// single-character string equality comparisons.
    CharDispatch {
        src: u8,
        table_idx: u16,
    },
    /// Fast return from constant: LoadConst + Return fusion.
    ReturnConst {
        const_idx: u16,
    },
    NumAdd {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    NumAddI {
        dst: u8,
        src: u8,
        imm: i16,
    },
    NumSub {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    NumSubI {
        dst: u8,
        src: u8,
        imm: i16,
    },
    NumMulI {
        dst: u8,
        src: u8,
        imm: i16,
    },
    NumDivI {
        dst: u8,
        src: u8,
        imm: i16,
    },
    NumMul {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    NumDiv {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    NumMod {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    IntDiv {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    IntMod {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    IntLeRRJumpIfFalse {
        src1: u8,
        src2: u8,
        offset: i16,
    },
    /// Packed compare+jump (side-table): payload_idx into cmp_jump_payloads
    IntLtRRJumpPacked(u32),
    IntLeRRJumpPacked(u32),
    IntLtRRJumpIfFalse {
        src1: u8,
        src2: u8,
        offset: i16,
    },
    IntAddReturn {
        src1: u8,
        src2: u8,
    },
    IntSubReturn {
        src1: u8,
        src2: u8,
    },
    IntMulReturn {
        src1: u8,
        src2: u8,
    },
    IntDivReturn {
        src1: u8,
        src2: u8,
    },
    IntCmpIReturn {
        src: u8,
        imm: i16,
        op: u8,
    },
    /// LoadIntConst + ArrayPush → ArrayPushIntConst
    ArrayPushIntConst {
        arr: u8,
        const_idx: u16,
    },
    /// LoadConst + ArrayPush → ArrayPushConst (non-int constants)
    ArrayPushConst {
        arr: u8,
        const_idx: u16,
    },
    /// Index + Index → Index2D (matrix[i][j] read)
    Index2D {
        dst: u8,
        obj: u8,
        idx1: u8,
        idx2: u8,
    },
    /// Index + IndexAssign → IndexAssign2D (matrix[i][j] = val)
    IndexAssign2D {
        obj: u8,
        idx1: u8,
        idx2: u8,
        val: u8,
    },
    /// IntMul+IntAdd → IntMulAddAssign (acc += src1*src2, matrix çarpım kalbi)
    IntMulAddAssign {
        acc: u8,
        src1: u8,
        src2: u8,
    },
    /// GetProperty + IntSub + SetProperty → PropertySubAssign
    PropertySubAssign {
        obj: u8,
        prop_sym: u16,
        src: u8,
    },
    /// StrCat(a, b) + StrCat(tmp, c) → StrCat3
    StrCat3 {
        dst: u8,
        a: u8,
        b: u8,
        c: u8,
    },
    LoadIntConst {
        dst: u8,
        const_idx: u16,
    },
    LoadConst {
        dst: u8,
        const_idx: u16,
    },
    LoadNumConst {
        dst: u8,
        const_idx: u16,
    },
    Return {
        src: u8,
    },
    Index {
        dst: u8,
        obj: u8,
        idx: u8,
    },
    /// P2: array-only index — no object/string/type dispatch overhead.
    IndexArray {
        dst: u8,
        obj: u8,
        idx: u8,
    },
    /// P2: string ASCII fast-path index.
    IndexStringAscii {
        dst: u8,
        obj: u8,
        idx: u8,
    },
    MakeArray {
        dst: u8,
        count: u16,
    },
    MakeObject {
        dst: u8,
        count: u16,
    },
    Call {
        dst: u8,
        payload_idx: u16,
        first_arg: u8,
        arg_count: u8,
    },
    LoadGlobal {
        dst: u8,
        sym: u16,
    },
    StoreGlobal {
        src: u8,
        sym: u16,
    },
    /// OPT 2: Store a chunk constant directly to a global slot.
    /// Replaces LoadConst + StoreGlobal.
    StoreGlobalConst {
        sym: u16,
        const_idx: u16,
    },
    /// G5-slotvec: load closure cell by slot index (no sym lookup, no hash).
    LoadClosureSlot {
        dst: u8,
        slot: u8,
    },
    /// G5-slotvec: store to closure cell by slot index.
    StoreClosureSlot {
        src: u8,
        slot: u8,
    },
    DeclGlobal {
        src: u8,
        sym: u16,
    },
    StoreConst {
        src: u8,
        sym: u16,
    },
    StrCat {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    /// In-place string append (dst == src1 implied), self-assignment only.
    NumMulAddAssign { dst: u8, mul: u8, add: u8 },
    /// Horner polynomial fusion: acc = acc * mul + arr[idx] (2 ops → 1)
    NumMulAddIndexed { acc: u8, mul: u8, arr: u8, idx: u8 },
    /// Float-only fused multiply-add: dst = mul1 * mul2 + add (single FMA instruction).
    FloatMulAdd { dst: u8, mul1: u8, mul2: u8, add: u8 },
    /// Float-only add: dst = src1 + src2 (no type check).
    FloatAdd { dst: u8, src1: u8, src2: u8 },
    /// Float-only mul: dst = src1 * src2 (no type check).
    FloatMul { dst: u8, src1: u8, src2: u8 },
    /// P4: fused integer multiply-modulo — dst = (src1 * src2) % src3.
    /// Used by `modular_exp` and other modular arithmetic loops to keep
    /// the hot path in Int tag without intermediate Number widening.
    IntMulMod { dst: u8, src1: u8, src2: u8, src3: u8 },
    /// P4: fused integer multiply-modulo with constant modulus —
    /// dst = (src1 * src2) % imm.  Common when the modulus is a loop
    /// invariant literal (e.g. `result = (result * base) % 1000000007`).
    IntMulModI { dst: u8, src1: u8, src2: u8, imm: i16 },
    /// Palindrome fusion: s[i] == s[j] (2 Index + Cmp → 1 op)
    StrCharEqRR { dst: u8, src_s: u8, src_i: u8, src_j: u8 },
    StrCatMut {
        dst: u8,
        src2: u8,
    },
    StringIndexOf {
        dst: u8,
        haystack: u8,
        needle: u8,
    },
    StringContains {
        dst: u8,
        haystack: u8,
        needle: u8,
    },
    SetProperty {
        dst: u8,
        obj: u8,
        val: u8,
        prop_sym: u16,
    },
    /// T6.3: In-place insert for object literals — no clone, alias-safe.
    ObjLitSet {
        obj: u8,
        val: u8,
        prop_sym: u16,
    },
    IndexAssign {
        obj: u8,
        idx: u8,
        val: u8,
    },
    /// P2: array-only index assignment — no object dispatch overhead.
    IndexAssignArray {
        obj: u8,
        idx: u8,
        val: u8,
    },
    Neg {
        dst: u8,
        src: u8,
    },
    Not {
        dst: u8,
        src: u8,
    },
    Move {
        dst: u8,
        src: u8,
    },
    /// P2: array length fast path — avoids GetProperty dispatch.
    ArrayLen {
        dst: u8,
        obj: u8,
    },
    /// P2: string length fast path — avoids GetProperty dispatch.
    StringLen {
        dst: u8,
        obj: u8,
    },
    /// P2: array pop fast path — avoids MethodCall dispatch.
    ArrayPop {
        dst: u8,
        obj: u8,
    },
    /// P5: Math.sqrt(number) intrinsic — direct numeric sqrt, no MethodCall.
    NumSqrt {
        dst: u8,
        src: u8,
    },
    /// P8: 2-element array literal — avoids MakeArray(0) + 2×ArrayPush.
    MakeArray2 {
        dst: u8,
        a: u8,
        b: u8,
    },
    /// G4: genel cmp+branch packed formu — IntCmpRRJumpIfFalse'un
    /// payload-tablolu hâli (alanlar u32 packed'e sığmadığı için src1/src2/
    /// target `CmpJumpPayload`'da yaşar; `op` packed arg1'e girer).
    /// Optimizer sonu dönüşümüyle üretilir; Lt/Le'ye özel eski
    /// IntLt/LeRRJumpPacked ikilisinin 6-op'lu tek genellemesi.
    /// BYTECODE_VERSION 22'de eklendi (enum kuyruğu — eski .hudb'ler
    /// etkilenmez).
    IntCmpRRJumpPacked {
        op: u8,
        payload_idx: u16,
    },
    /// G8: Math.sin(x) intrinsic — MethodCall yerine tek komut (P5/NumSqrt
    /// deseninin sin eşi). Math gölgelenmediyse derleyici emit eder.
    /// BYTECODE_VERSION 22 (aynı yayın döngüsü, enum kuyruğu).
    NumSin {
        dst: u8,
        src: u8,
    },
    /// G8: Math.cos(x) intrinsic — NumSin'in eşi.
    NumCos {
        dst: u8,
        src: u8,
    },
    // ── G12: unboxed float slot ailesi (exp/unboxed-float) ──────────
    // Sıcak döngüde float-KANITLI yereller Value16 register'ı yerine
    // VM'in `f_slots: [f64; 64]` dosyasında yaşar; aradaki tüm aritmetik
    // tag'siz f64 üstünde koşar. Döngü sınırlarında FLoadNum/FStoreNum
    // kutu-aç/kutula. BYTECODE_VERSION 22 (aynı yayın döngüsü, kuyruk).
    /// Value16 reg → f64 slot (Int/Number kabul; değilse runtime hata —
    /// derleyici tip-kanıtı olmadan emit ETMEZ, hata=derleyici bug'ı).
    FLoadNum { fslot: u8, src: u8 },
    /// f64 slot → Value16 reg (Number olarak kutula).
    FStoreNum { dst: u8, fslot: u8 },
    /// f_slots[d] = f_slots[a] ⊕ f_slots[b] — tag'siz.
    FAdd { d: u8, a: u8, b: u8 },
    FSub { d: u8, a: u8, b: u8 },
    FMul { d: u8, a: u8, b: u8 },
    FDiv { d: u8, a: u8, b: u8 },
    /// f_slots[d] = f_slots[s].sin() vb.
    FSin { d: u8, s: u8 },
    FCos { d: u8, s: u8 },
    FSqrt { d: u8, s: u8 },
    /// f_slots[d] = f_slots[s] — slot-içi kopya (let x = y deseni).
    FMove { d: u8, s: u8 },
    /// f_slots[d] = sabit (numeric_constants havuzundan).
    FConst { d: u8, const_idx: u16 },
}
