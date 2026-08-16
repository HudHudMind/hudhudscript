//! V2-B0: Instruction impl methods extracted from instruction.rs.

use crate::instruction::Instruction;
use crate::Value16;

impl Instruction {
    /// Returns the highest register index referenced by this instruction.
    /// Used by the compiler to track per-function max register usage.
    pub fn max_register(&self) -> u8 {
        use Instruction::*;
        let mr = match *self {
            JumpIfFalse { src, .. } | JumpIfTrue { src, .. } => src,
            TailCall {
                func_reg,
                first_arg_reg,
                arg_count,
            } => func_reg.max(first_arg_reg.saturating_add(arg_count).saturating_sub(1)),
            ForIn { iter_reg, .. } | IterNext { iter_reg, .. } => iter_reg,
            Throw { src } | Require { src } | Perform { src } | Yield { src } => src,
            CharDispatch { src, .. } => src,
            Await { src, dst } => dst.max(src),
            Spawn {
                first_arg,
                arg_count,
                ..
            }
            | Send {
                message: first_arg,
                target: arg_count,
            } => first_arg.max(arg_count),
            Despawn { reg } => reg,
            ViewAs { obj, .. } => obj,
            Receive { src, .. } => src,
            MethodCall {
                dst,
                obj,
                first_arg,
                arg_count,
                ..
            } => {
                let last_arg = first_arg.saturating_add(arg_count).saturating_sub(1);
                dst.max(obj).max(last_arg)
            }
            NewInstance {
                first_arg,
                arg_count,
                ..
            }
            | MakeGenerator {
                first_arg,
                arg_count,
                ..
            } => first_arg.saturating_add(arg_count).saturating_sub(1),
            SuperCall {
                dst,
                first_arg,
                arg_count,
                ..
            } => {
                let last_arg = first_arg.saturating_add(arg_count).saturating_sub(1);
                dst.max(last_arg)
            }
            Call {
                dst,
                first_arg,
                arg_count,
                ..
            } => {
                let last_arg = first_arg.saturating_add(arg_count).saturating_sub(1);
                dst.max(last_arg)
            }
            GetProperty { dst, obj, .. } => dst.max(obj),
            DeclStore { src, .. }
            | StoreConst { src, .. }
            | StoreGlobal { src, .. }
            | DeclGlobal { src, .. } => src,
            StoreGlobalConst { .. } => 0,
            LoadClosureSlot { dst, .. } => dst,
            StoreClosureSlot { src, .. } => src,
            LoadGlobal { dst, .. }
            | LoadNumConst { dst, .. }
            | LoadConst { dst, .. }
            | LoadIntConst { dst, .. } => dst,
            StringConcat {
                regs_start,
                count,
                dst,
            } => dst.max(regs_start.saturating_add(count).saturating_sub(1)),
            ArrayPush { dst, arr, val }
            | SetProperty {
                dst, obj: arr, val, ..
            } => dst.max(arr).max(val),
            ObjLitSet { obj: arr, val, .. } => arr.max(val),
            SpreadIntoArray { dst, src } | SpreadIntoObject { dst, src } => dst.max(src),
            Index { dst, obj, idx } => dst.max(obj).max(idx),
            IndexArray { dst, obj, idx } => dst.max(obj).max(idx),
            IndexStringAscii { dst, obj, idx } => dst.max(obj).max(idx),
            IndexAssign { obj, idx, val } => obj.max(idx).max(val),
            IndexAssignArray { obj, idx, val } => obj.max(idx).max(val),
            StrCat { dst, src1, src2 }
            | StringIndexOf {
                dst,
                haystack: src1,
                needle: src2,
            }
            | StringContains {
                dst,
                haystack: src1,
                needle: src2,
            } => dst.max(src1).max(src2),
            StrCatMut { dst, src2 } => dst.max(src2),
            NumMulAddAssign { dst, mul, add } => dst.max(mul).max(add),
            NumMulAddIndexed { acc, mul, arr, idx } => acc.max(mul).max(arr).max(idx),
            FloatMulAdd {
                dst,
                mul1,
                mul2,
                add,
            } => dst.max(mul1).max(mul2).max(add),
            FloatAdd { dst, src1, src2 } => dst.max(src1).max(src2),
            FloatMul { dst, src1, src2 } => dst.max(src1).max(src2),
            IntMulMod {
                dst,
                src1,
                src2,
                src3,
            } => dst.max(src1).max(src2).max(src3),
            IntMulModI {
                dst, src1, src2, ..
            } => dst.max(src1).max(src2),
            StrCharEqRR {
                dst,
                src_s,
                src_i,
                src_j,
            } => dst.max(src_s).max(src_i).max(src_j),
            IntAdd { dst, src1, src2 }
            | IntSub { dst, src1, src2 }
            | IntMul { dst, src1, src2 }
            | IntCmp {
                dst, src1, src2, ..
            }
            | NumAdd { dst, src1, src2 }
            | NumSub { dst, src1, src2 }
            | NumMul { dst, src1, src2 }
            | NumDiv { dst, src1, src2 }
            | NumMod { dst, src1, src2 } => dst.max(src1).max(src2),
            IntAddI { dst, src, .. }
            | IntSubI { dst, src, .. }
            | IntMulI { dst, src, .. }
            | IntDivI { dst, src, .. }
            | IntModI { dst, src, .. }
            | IntCmpI { dst, src, .. }
            | IntModCmpI { dst, src, .. } => dst.max(src),
            IntCmpIJumpIfFalse { src, .. } => src,
            IntCmpIJumpIfTrue { src, .. } => src,
            IntCmpRRJumpIfFalse { src1, src2, .. } => src1.max(src2),
            IntAddIJump { reg, .. } => reg,
            LoopEndIntAddIJump { reg, .. } => reg,
            IntSubIJump { reg, .. } => reg,
            ReturnConst { .. } => 0,
            NumAddI { dst, src, .. }
            | NumSubI { dst, src, .. }
            | NumMulI { dst, src, .. }
            | NumDivI { dst, src, .. } => dst.max(src),
            IntDiv { dst, src1, src2 } | IntMod { dst, src1, src2 } => dst.max(src1).max(src2),
            IntLeRRJumpIfFalse { src1, src2, .. } | IntLtRRJumpIfFalse { src1, src2, .. } => {
                src1.max(src2)
            }
            IntLtRRJumpPacked(_) | IntLeRRJumpPacked(_) | IntCmpRRJumpPacked { .. } => 255,
            IntAddReturn { src1, src2 } | IntSubReturn { src1, src2 } => src1.max(src2).max(255),
            IntMulReturn { src1, src2 } | IntDivReturn { src1, src2 } => src1.max(src2).max(255),
            IntCmpIReturn { src, .. } => src,
            ArrayPushIntConst { arr, .. } => arr,
            ArrayPushConst { arr, .. } => arr,
            Index2D {
                dst,
                obj,
                idx1,
                idx2,
            } => dst.max(obj).max(idx1).max(idx2),
            IndexAssign2D {
                obj,
                idx1,
                idx2,
                val,
            } => obj.max(idx1).max(idx2).max(val),
            IntMulAddAssign { acc, src1, src2 } => acc.max(src1).max(src2),
            PropertySubAssign { obj, src, .. } => obj.max(src),
            StrCat3 { dst, a, b, c } => dst.max(a).max(b).max(c),
            Neg { dst, src } | Not { dst, src } | Move { dst, src } => dst.max(src),
            Return { src } => src,
            // Instructions with no registers
            Jump(..)
            | Break
            | Continue
            | TryBegin(..)
            | TryEnd
            | FinallyBegin(..)
            | FinallyEnd
            | FinallyExit(..)
            | LoopBegin(..)
            | LoopEnd
            | EnumDecl(..)
            | MatchVariant(..)
            | BindVar(..)
            | ClassDecl(..)
            | TraitCheck(..)
            | LoadModule(..)
            | DefineFunction(..)
            | GetStatic(..)
            | ClassStaticDecl(..)
            | DestructArray(..)
            | DestructObject(..)
            | CallSpread(..)
            | MethodCallSpread { .. } => 0,
            Remember { src, .. } => src,
            Recall { dst, src, .. } => dst.max(src),
            Forget { src, .. } => src,
            // Instructions with no registers
            IntLeJumpIfFalse(..) | IntLtJumpIfFalse(..) | IntSubCall1(..) | IntAddCall1(..) => 0,
            MakeArray { dst, .. } => dst,
            // P8: 2-element array
            MakeArray2 { dst, a, b } => dst.max(a).max(b),
            MakeObject { dst, .. } => dst,
            // P2: length/pop fast path
            ArrayLen { dst, obj } | StringLen { dst, obj } | ArrayPop { dst, obj } => dst.max(obj),
            // P5: sqrt intrinsic
            NumSqrt { dst, src } | NumSin { dst, src } | NumCos { dst, src } => dst.max(src),
            // G12: F-op'lar register dosyasına yalnız yükle/sakla uçlarından dokunur.
            FLoadNum { src, .. } => src,
            FStoreNum { dst, .. } => dst,
            FAdd { .. }
            | FSub { .. }
            | FMul { .. }
            | FDiv { .. }
            | FSin { .. }
            | FCos { .. }
            | FSqrt { .. }
            | FConst { .. }
            | FMove { .. } => 0,
        };
        // Register 255 is reserved for bridge/return-value; it does not count
        // toward per-function max register usage.
        if mr == 255 {
            0
        } else {
            mr
        }
    }
}

/// G4/G5 temeli — KESİN register-etki modeli (Kural 7: TEK yer).
///
/// `reads`: komutun okuduğu tekil register'lar; `read_range`: ardışık okunan
/// aralık (çağrı argümanları); `writes`: önceki değeri OKUNMADAN üzerine
/// yazılan register (read-modify-write'lar `writes` DEĞİL, `reads`'e girer —
/// canlılık için accumulate bir OKUMADIR); `barrier`: register davranışı
/// payload-dolaylı / tam modellenmemiş — canlılık analizi bunu "her şeyi
/// okur" saymak ZORUNDADIR.
#[derive(Clone, Copy, Debug, Default)]
pub struct RegisterEffects {
    pub reads: [Option<u8>; 4],
    pub read_range: Option<(u8, u8)>,
    pub writes: Option<u8>,
    pub barrier: bool,
}

impl RegisterEffects {
    #[inline]
    pub fn reads_reg(&self, reg: u8) -> bool {
        if self.barrier {
            return true;
        }
        if self.reads.iter().flatten().any(|&r| r == reg) {
            return true;
        }
        if let Some((first, count)) = self.read_range {
            let f = first as u16;
            let r = reg as u16;
            return r >= f && r < f + count as u16;
        }
        false
    }
}

impl Instruction {
    /// Bkz. [`RegisterEffects`]. `_` JOKERİ YOK — yeni varyant eklendiğinde
    /// derleme hatası bu modeli güncellemeye zorlar (opcode_spec deseni).
    pub fn register_effects(&self) -> RegisterEffects {
        use Instruction::*;
        fn rw(reads: &[u8], writes: Option<u8>) -> RegisterEffects {
            let mut r = [None; 4];
            for (i, &x) in reads.iter().take(4).enumerate() {
                r[i] = Some(x);
            }
            RegisterEffects {
                reads: r,
                read_range: None,
                writes,
                barrier: false,
            }
        }
        fn barrier() -> RegisterEffects {
            RegisterEffects {
                barrier: true,
                ..Default::default()
            }
        }
        match *self {
            // ── yükler (yalnız yazar) ─────────────────────────────
            LoadConst { dst, .. }
            | LoadNumConst { dst, .. }
            | LoadIntConst { dst, .. }
            | LoadGlobal { dst, .. }
            | LoadClosureSlot { dst, .. } => rw(&[], Some(dst)),
            // ── saklamalar (yalnız okur; hedef register değil) ────
            StoreGlobal { src, .. }
            | DeclGlobal { src, .. }
            | DeclStore { src, .. }
            | StoreConst { src, .. }
            | StoreClosureSlot { src, .. } => rw(&[src], None),
            StoreGlobalConst { .. } => rw(&[], None),
            // ── veri akışı ────────────────────────────────────────
            Move { dst, src }
            | Neg { dst, src }
            | Not { dst, src }
            | NumSqrt { dst, src }
            | NumSin { dst, src }
            | NumCos { dst, src } => rw(&[src], Some(dst)),
            // G12: FLoadNum reg okur (slot'a yazar — reg-uzayı dışı);
            // FStoreNum reg'e yazar; slot-içi op'lar reg'e dokunmaz.
            FLoadNum { src, .. } => rw(&[src], None),
            FStoreNum { dst, .. } => rw(&[], Some(dst)),
            FAdd { .. }
            | FSub { .. }
            | FMul { .. }
            | FDiv { .. }
            | FSin { .. }
            | FCos { .. }
            | FSqrt { .. }
            | FConst { .. }
            | FMove { .. } => rw(&[], None),
            // ── aritmetik RR ──────────────────────────────────────
            IntAdd { dst, src1, src2 }
            | IntSub { dst, src1, src2 }
            | IntMul { dst, src1, src2 }
            | IntDiv { dst, src1, src2 }
            | IntMod { dst, src1, src2 }
            | NumAdd { dst, src1, src2 }
            | NumSub { dst, src1, src2 }
            | NumMul { dst, src1, src2 }
            | NumDiv { dst, src1, src2 }
            | NumMod { dst, src1, src2 }
            | FloatAdd { dst, src1, src2 }
            | FloatMul { dst, src1, src2 }
            | IntCmp {
                dst, src1, src2, ..
            }
            | StrCat { dst, src1, src2 } => rw(&[src1, src2], Some(dst)),
            // ── aritmetik imm ─────────────────────────────────────
            IntAddI { dst, src, .. }
            | IntSubI { dst, src, .. }
            | IntMulI { dst, src, .. }
            | IntDivI { dst, src, .. }
            | IntModI { dst, src, .. }
            | IntCmpI { dst, src, .. }
            | IntModCmpI { dst, src, .. }
            | NumAddI { dst, src, .. }
            | NumSubI { dst, src, .. }
            | NumMulI { dst, src, .. }
            | NumDivI { dst, src, .. } => rw(&[src], Some(dst)),
            // ── compound (read-modify-write → hepsi READ) ─────────
            IntMulAddAssign { acc, src1, src2 } => rw(&[acc, src1, src2], None),
            NumMulAddAssign { dst, mul, add } => rw(&[dst, mul, add], None),
            NumMulAddIndexed { acc, mul, arr, idx } => rw(&[acc, mul, arr, idx], None),
            FloatMulAdd {
                dst,
                mul1,
                mul2,
                add,
            } => rw(&[mul1, mul2, add], Some(dst)),
            IntMulMod {
                dst,
                src1,
                src2,
                src3,
            } => rw(&[src1, src2, src3], Some(dst)),
            IntMulModI {
                dst, src1, src2, ..
            } => rw(&[src1, src2], Some(dst)),
            PropertySubAssign { obj, src, .. } => rw(&[obj, src], None),
            StrCatMut { dst, src2 } => rw(&[dst, src2], None),
            StrCat3 { dst, a, b, c } => rw(&[a, b, c], Some(dst)),
            StrCharEqRR {
                dst,
                src_s,
                src_i,
                src_j,
            } => rw(&[src_s, src_i, src_j], Some(dst)),
            StringIndexOf {
                dst,
                haystack,
                needle,
            }
            | StringContains {
                dst,
                haystack,
                needle,
            } => rw(&[haystack, needle], Some(dst)),
            StringConcat {
                regs_start,
                count,
                dst,
            } => RegisterEffects {
                reads: [None; 4],
                read_range: Some((regs_start, count)),
                writes: Some(dst),
                barrier: false,
            },
            // ── koleksiyon ────────────────────────────────────────
            Index { dst, obj, idx }
            | IndexArray { dst, obj, idx }
            | IndexStringAscii { dst, obj, idx } => rw(&[obj, idx], Some(dst)),
            IndexAssign { obj, idx, val } | IndexAssignArray { obj, idx, val } => {
                rw(&[obj, idx, val], None)
            }
            Index2D {
                dst,
                obj,
                idx1,
                idx2,
            } => rw(&[obj, idx1, idx2], Some(dst)),
            IndexAssign2D {
                obj,
                idx1,
                idx2,
                val,
            } => rw(&[obj, idx1, idx2, val], None),
            ArrayPush { dst, arr, val } => rw(&[arr, val], Some(dst)),
            ArrayPushConst { arr, .. } | ArrayPushIntConst { arr, .. } => rw(&[arr], None),
            ArrayLen { dst, obj } | StringLen { dst, obj } | ArrayPop { dst, obj } => {
                rw(&[obj], Some(dst))
            }
            ObjLitSet { obj, val, .. } => rw(&[obj, val], None),
            MakeArray2 { dst, a, b } => rw(&[a, b], Some(dst)),
            // MakeArray/MakeObject count>0 formu bridge kullanır → barrier.
            MakeArray { dst, count } => {
                if count == 0 {
                    rw(&[], Some(dst))
                } else {
                    barrier()
                }
            }
            MakeObject { dst, count } => {
                if count == 0 {
                    rw(&[], Some(dst))
                } else {
                    barrier()
                }
            }
            GetProperty { dst, obj, .. } => rw(&[obj], Some(dst)),
            SetProperty { dst, obj, val, .. } => rw(&[obj, val], Some(dst)),
            SpreadIntoArray { .. } | SpreadIntoObject { .. } => barrier(), // r255 bridge
            // ── kontrol ───────────────────────────────────────────
            Jump(..) | Break | Continue | TryBegin(..) | TryEnd | FinallyBegin(..) | FinallyEnd
            | FinallyExit(..) | LoopBegin(..) | LoopEnd => rw(&[], None),
            JumpIfFalse { src, .. } | JumpIfTrue { src, .. } => rw(&[src], None),
            IntCmpIJumpIfFalse { src, .. } | IntCmpIJumpIfTrue { src, .. } => rw(&[src], None),
            IntCmpRRJumpIfFalse { src1, src2, .. }
            | IntLtRRJumpIfFalse { src1, src2, .. }
            | IntLeRRJumpIfFalse { src1, src2, .. } => rw(&[src1, src2], None),
            IntAddIJump { reg, .. } | IntSubIJump { reg, .. } | LoopEndIntAddIJump { reg, .. } => {
                rw(&[reg], None) // rmw sayaç
            }
            Return { src } | Throw { src } => rw(&[src], None),
            ReturnConst { .. } => rw(&[], Some(255)),
            IntAddReturn { src1, src2 }
            | IntSubReturn { src1, src2 }
            | IntMulReturn { src1, src2 }
            | IntDivReturn { src1, src2 } => rw(&[src1, src2], Some(255)),
            IntCmpIReturn { src, .. } => rw(&[src], Some(255)),
            // ── çağrılar ──────────────────────────────────────────
            Call {
                dst,
                first_arg,
                arg_count,
                ..
            } => RegisterEffects {
                reads: [None; 4],
                read_range: Some((first_arg, arg_count)),
                writes: Some(dst),
                barrier: false,
            },
            MethodCall {
                dst,
                obj,
                first_arg,
                arg_count,
                ..
            } => RegisterEffects {
                reads: [Some(obj), None, None, None],
                read_range: Some((first_arg, arg_count)),
                writes: Some(dst),
                barrier: false,
            },
            SuperCall {
                dst,
                first_arg,
                arg_count,
                ..
            } => RegisterEffects {
                reads: [None; 4],
                read_range: Some((first_arg, arg_count)),
                writes: Some(dst),
                barrier: false,
            },
            TailCall {
                func_reg,
                first_arg_reg,
                arg_count,
            } => RegisterEffects {
                reads: [Some(func_reg), None, None, None],
                read_range: Some((first_arg_reg, arg_count)),
                writes: None,
                barrier: false,
            },
            // ── payload-dolaylı / bridge / agent — MODELLENMEDİ ───
            CallSpread(..)
            | MethodCallSpread { .. }
            | IntSubCall1(..)
            | IntAddCall1(..)
            | IntLeJumpIfFalse(..)
            | IntLtJumpIfFalse(..)
            | IntLtRRJumpPacked(..)
            | IntLeRRJumpPacked(..)
            | IntCmpRRJumpPacked { .. }
            | CharDispatch { .. }
            | ForIn { .. }
            | IterNext { .. }
            | EnumDecl(..)
            | MatchVariant(..)
            | BindVar(..)
            | ClassDecl(..)
            | TraitCheck(..)
            | LoadModule(..)
            | DefineFunction(..)
            | GetStatic(..)
            | ClassStaticDecl(..)
            | DestructArray(..)
            | DestructObject(..)
            | NewInstance { .. }
            | MakeGenerator { .. }
            | Spawn { .. }
            | Despawn { .. }
            | ViewAs { .. }
            | Send { .. }
            | Receive { .. }
            | Require { .. }
            | Perform { .. }
            | Yield { .. }
            | Await { .. }
            | Remember { .. }
            | Recall { .. }
            | Forget { .. } => barrier(),
        }
    }

    /// Dallanma hedefi: `Some(mutlak_ip)` — hedefi bilinen göreli atlamalar.
    /// `barrier`'lı kontrol komutları (CharDispatch, payload-jump'lar,
    /// ForIn/IterNext) None döner ama register_effects().barrier=true
    /// olduğundan canlılık analizi zaten muhafazakâr davranır.
    pub fn branch_target(&self, ip: usize) -> Option<usize> {
        use Instruction::*;
        let rel: i64 = match *self {
            Jump(off) => off as i64,
            JumpIfFalse { offset, .. } | JumpIfTrue { offset, .. } => offset as i64,
            IntCmpIJumpIfFalse { offset, .. } | IntCmpIJumpIfTrue { offset, .. } => offset as i64,
            IntCmpRRJumpIfFalse { offset, .. } => offset as i64,
            IntLtRRJumpIfFalse { offset, .. } | IntLeRRJumpIfFalse { offset, .. } => offset as i64,
            IntAddIJump { offset, .. }
            | IntSubIJump { offset, .. }
            | LoopEndIntAddIJump { offset, .. } => offset as i64,
            TryBegin(off) => off as i64,
            FinallyBegin(off) | FinallyExit(off) => off as i64,
            _ => return None,
        };
        let abs = ip as i64 + rel;
        if abs >= 0 {
            Some(abs as usize)
        } else {
            None
        }
    }
}
