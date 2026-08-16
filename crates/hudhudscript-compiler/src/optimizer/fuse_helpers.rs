use crate::optimizer::utils::adjust_jumps_after_remove_full;
use hudhudscript_bytecode::{Instruction, LoopPayload};

/// Fuse IntArith+Return patterns: IntAdd/Sub/Mul/Div+Return → IntXReturn,
/// IntCmpI+Return → IntCmpIReturn. Returns true if a fusion was applied.
pub(crate) fn try_fuse_arith_return(
    instructions: &mut Vec<Instruction>,
    loop_payloads: &mut [LoopPayload],
    source_positions: &mut Vec<Option<(usize, usize)>>,
    i: usize,
) -> bool {
    use Instruction::*;
    let new_instr = match (&instructions[i], &instructions[i + 1]) {
        (IntAdd { dst, src1, src2 }, Return { src }) if *dst == *src => Some(IntAddReturn {
            src1: *src1,
            src2: *src2,
        }),
        (IntSub { dst, src1, src2 }, Return { src }) if *dst == *src => Some(IntSubReturn {
            src1: *src1,
            src2: *src2,
        }),
        (IntMul { dst, src1, src2 }, Return { src }) if *dst == *src => Some(IntMulReturn {
            src1: *src1,
            src2: *src2,
        }),
        (IntDiv { dst, src1, src2 }, Return { src }) if *dst == *src => Some(IntDivReturn {
            src1: *src1,
            src2: *src2,
        }),
        (IntCmpI { dst, src, imm, op }, Return { src: ret_src }) if *dst == *ret_src => {
            Some(IntCmpIReturn {
                src: *src,
                imm: *imm,
                op: *op,
            })
        }
        (LoadConst { dst, const_idx }, Return { src }) if *dst == *src => Some(ReturnConst {
            const_idx: *const_idx,
        }),
        // NOT (G3 denetimi): `return 42` şekli LoadIntConst+Return üretir ve
        // FÜZLENEMEZ — LoadIntConst `int_constants` havuzundan, ReturnConst
        // `constants` havuzundan okur (indeks uzayları FARKLI). Diriltme ya
        // ReturnIntConst opcode'u (G4) ya da optimizer'a havuz erişimi ister.
        _ => None,
    };
    if let Some(fused) = new_instr {
        instructions[i] = fused;
        remove_fused_pair(instructions, loop_payloads, source_positions, i);
        true
    } else {
        false
    }
}

/// Common tail for 2-instruction fusion: remove `instructions[i+1]`,
/// fix jumps, and drop the corresponding source position.
pub(crate) fn remove_fused_pair(
    instructions: &mut Vec<Instruction>,
    loop_payloads: &mut [LoopPayload],
    source_positions: &mut Vec<Option<(usize, usize)>>,
    i: usize,
) {
    adjust_jumps_after_remove_full(instructions, loop_payloads, &mut [], &mut [], i + 1);
    instructions.remove(i + 1);
    if i + 1 < source_positions.len() {
        source_positions.remove(i + 1);
    }
}

/// G4: füzyonun tükettiği cmp-temp'in `from`'dan sonra ÖLÜ olduğunu kanıtlar.
/// Eski `reg_used_as_source` yazım-kill bilmiyordu: temp register bir sonraki
/// statement'ta YENİDEN YAZILIP okununca "canlı" sanıp `if (a < b)` ailesinin
/// tamamında füzyonu iptal ediyordu (IntCmpRRJumpIfFalse'un ölü kalma nedeni).
///
/// Yöntem (muhafazakâr, sound): tarama başlangıçları = lineer devam (`from`)
/// + chunk'taki TÜM bilinen dal hedefleri. Her taramada: modellenmemiş komut
/// (`barrier`) ya da `reg` okuması → CANLI (füzyon iptali); `reg`'e kill-yazım
/// → o yol güvenli. Register modeli TEK kaynaktan: `Instruction::
/// register_effects()` + `branch_target()` (bytecode crate, jokersiz match).
pub(crate) fn reg_dead_after(
    instructions: &[Instruction],
    loop_payloads: &[LoopPayload],
    from: usize,
    reg: u8,
) -> bool {
    let mut starts = vec![from];
    for (ip, instr) in instructions.iter().enumerate() {
        if let Some(t) = instr.branch_target(ip) {
            if t < instructions.len() {
                starts.push(t);
            }
        }
        // G5 ön-şartı: Break/Continue hedefleri runtime'da loop_headers'tan
        // gelir; derleme-zamanı karşılığı LoopBegin'in payload'ındaki
        // start/end IP'leridir. Bunlar da akış giriş noktasıdır — ilk
        // deneme bu deliği görmemişti (iki miscompile testlerde yakalandı).
        if let Instruction::LoopBegin(idx) = instr {
            if let Some(p) = loop_payloads.get(*idx as usize) {
                if (p.start as usize) < instructions.len() {
                    starts.push(p.start as usize);
                }
                if (p.end as usize) < instructions.len() {
                    starts.push(p.end as usize);
                }
            }
        }
    }
    starts.sort_unstable();
    starts.dedup();
    for &s in &starts {
        for instr in &instructions[s..] {
            let e = instr.register_effects();
            if e.reads_reg(reg) {
                return false; // canlı — füzyon güvensiz
            }
            if e.writes == Some(reg) {
                break; // kill — bu yol güvenli
            }
        }
    }
    true
}

/// BUG2 (eski API): register herhangi bir komutta kaynak olarak geçiyor mu.
/// Yalnız `instr_reads_reg`'in modellediği komutlar için kesin — kill
/// bilmez, bilinmeyen komutu "okumuyor" sayar. Yeni kod `reg_dead_after`
/// kullanmalı; bu yalnız eski çağıranlar için duruyor.
#[allow(dead_code)]
pub(crate) fn reg_used_as_source(instructions: &[Instruction], reg: u8) -> bool {
    for instr in instructions {
        if instr_reads_reg(instr, reg) {
            return true;
        }
    }
    false
}

pub(crate) fn instr_reads_reg(instr: &Instruction, reg: u8) -> bool {
    use Instruction::*;
    match instr {
        Move { src, .. } => *src == reg,
        JumpIfFalse { src, .. } | JumpIfTrue { src, .. } => *src == reg,
        Throw { src } | Return { src } => *src == reg,
        IntCmp { src1, src2, .. } => *src1 == reg || *src2 == reg,
        IntCmpI { src, .. } => *src == reg,
        IntAdd { src1, src2, .. }
        | IntSub { src1, src2, .. }
        | IntMul { src1, src2, .. }
        | IntDiv { src1, src2, .. }
        | IntMod { src1, src2, .. } => *src1 == reg || *src2 == reg,
        IntAddI { src, .. } | IntSubI { src, .. } | IntMulI { src, .. } | IntModI { src, .. } => {
            *src == reg
        }
        NumAdd { src1, src2, .. }
        | NumSub { src1, src2, .. }
        | NumMul { src1, src2, .. }
        | NumDiv { src1, src2, .. }
        | NumMod { src1, src2, .. } => *src1 == reg || *src2 == reg,
        Call {
            first_arg,
            arg_count,
            ..
        } => {
            let f = *first_arg as usize;
            (f..f + *arg_count as usize).any(|r| r as u8 == reg)
        }
        Index { obj, idx, .. } | IndexArray { obj, idx, .. } => *obj == reg || *idx == reg,
        StrCat { src1, src2, .. } => *src1 == reg || *src2 == reg,
        GetProperty { obj, .. } => *obj == reg,
        ArrayPush { val, .. } => *val == reg,
        DeclGlobal { src, .. } | StoreGlobal { src, .. } => *src == reg,
        _ => false,
    }
}
/// G4: `IntCmpRRJumpIfFalse` → `IntCmpRRJumpPacked` dönüşümü — alanlar u32
/// packed'e sığmadığı için src1/src2/target `CmpJumpPayload`'a taşınır
/// (op packed arg1'de). Optimizasyonun SONUNDA koşar; sonrasında komut
/// silinirse `adjust_jumps_after_remove_full` payload target'larını
/// güncellemeyi zaten bilir. u16 payload kapasitesi dolarsa kalan komutlar
/// UNPACKED kalır — kapasite sınırı, fallback değil (tek semantik:
/// `cmp_rr_generic`; iki temsil aynı çekirdeği çağırır).
pub(crate) fn pack_cmp_jumps(
    instructions: &mut [Instruction],
    cmp_jump_payloads: &mut Vec<hudhudscript_bytecode::CmpJumpPayload>,
) {
    for i in 0..instructions.len() {
        if let Instruction::IntCmpRRJumpIfFalse {
            src1,
            src2,
            op,
            offset,
        } = instructions[i]
        {
            let target = (i as i64).wrapping_add(offset as i64);
            if target < 0 {
                continue;
            }
            let idx = cmp_jump_payloads.len();
            if idx > u16::MAX as usize {
                return;
            }
            cmp_jump_payloads.push(hudhudscript_bytecode::CmpJumpPayload {
                src1,
                src2,
                target: target as u32,
            });
            instructions[i] = Instruction::IntCmpRRJumpPacked {
                op,
                payload_idx: idx as u16,
            };
        }
    }
}

/// G5 — MOVE birleştirme (copy coalescing): `Üretici{dst:t} + Move{d,t}`
/// çiftinde `t` sonrasında ÖLÜyse üreticinin hedefi `d` yapılır, Move silinir.
/// MOVE_RR global #1 opcode (252M; k_nucleotide'de tek başına 1.4M).
///
/// Güvenlik kapıları (ilk denemenin iki miscompile dersi işlenmiş):
/// - İZOLASYON İÇİNDE çağrılır (BUG4b: adjust iç-içe payload'ları bozmasın).
/// - `reg_dead_after` artık LoopPayload start/end hedeflerini de tarar
///   (Break/Continue akış-girişleri — ilk denemenin kör noktası).
/// - CharDispatch (mutlak-IP tabloları), ForIn/IterNext (görünmez hedef),
///   payload-atlama varyantları → fonksiyon KOMPLE atlanır.
/// - Hiçbir dal Move'un indeksine atlamamalı.
/// - Yalnız `set_dst`'in tanıdığı basit üreticiler.
pub(crate) fn coalesce_moves(
    instructions: &mut Vec<Instruction>,
    loop_payloads: &mut [LoopPayload],
    source_positions: &mut Vec<Option<(usize, usize)>>,
    protected_below: u8,
) {
    use Instruction as I;
    if instructions.iter().any(|i| {
        matches!(
            i,
            I::CharDispatch { .. }
                | I::ForIn { .. }
                | I::IterNext { .. }
                | I::IntLtRRJumpPacked(..)
                | I::IntLeRRJumpPacked(..)
                | I::IntLeJumpIfFalse(..)
                | I::IntLtJumpIfFalse(..)
        )
    }) {
        return;
    }
    let mut i = 0;
    while i + 1 < instructions.len() {
        let I::Move { dst: d, src: t } = instructions[i + 1] else {
            i += 1;
            continue;
        };
        if d == t || t == 255 || d == 255 {
            i += 1;
            continue;
        }
        // İkinci denemenin kök-neden dersi (test_array_push): DEĞİŞKEN EVİ
        // registerları (locals bölgesi, 0..protected_below) chunk bittikten
        // SONRA da canlıdır (get_variable / main-slot / generator resume
        // okur) — chunk-içi tarama bunu göremez. Korumalı register ASLA
        // elenmez; temp'ler RegAlloc'ta bu tabanın üstünde tahsis edilir.
        if t < protected_below {
            i += 1;
            continue;
        }
        let eff = instructions[i].register_effects();
        if eff.barrier || eff.writes != Some(t) {
            i += 1;
            continue;
        }
        if !reg_dead_after(instructions, loop_payloads, i + 2, t) {
            i += 1;
            continue;
        }
        let mut move_targeted = false;
        for (ip, instr) in instructions.iter().enumerate() {
            if let Some(tg) = instr.branch_target(ip) {
                if tg == i + 1 {
                    move_targeted = true;
                    break;
                }
            }
            if let I::LoopBegin(idx) = instr {
                if let Some(p) = loop_payloads.get(*idx as usize) {
                    if p.start as usize == i + 1 || p.end as usize == i + 1 {
                        move_targeted = true;
                        break;
                    }
                }
            }
        }
        if move_targeted {
            i += 1;
            continue;
        }
        if !set_dst(&mut instructions[i], d) {
            i += 1;
            continue;
        }
        remove_fused_pair(instructions, loop_payloads, source_positions, i);
        // Aynı i'de yeni çift oluşabilir — i ilerletilmez.
    }
}

/// G5 — hedef register yeniden-yazımı: yalnız semantiği "sonucu dst'ye koy"
/// olan basit üreticiler. Tanınmayan varyant → false.
fn set_dst(instr: &mut Instruction, new_dst: u8) -> bool {
    use Instruction as I;
    match instr {
        I::LoadConst { dst, .. }
        | I::LoadNumConst { dst, .. }
        | I::LoadIntConst { dst, .. }
        | I::LoadGlobal { dst, .. }
        | I::LoadClosureSlot { dst, .. }
        | I::Move { dst, .. }
        | I::Neg { dst, .. }
        | I::Not { dst, .. }
        | I::NumSqrt { dst, .. }
        | I::NumSin { dst, .. }
        | I::NumCos { dst, .. }
        | I::IntAdd { dst, .. }
        | I::IntSub { dst, .. }
        | I::IntMul { dst, .. }
        | I::IntDiv { dst, .. }
        | I::IntMod { dst, .. }
        | I::NumAdd { dst, .. }
        | I::NumSub { dst, .. }
        | I::NumMul { dst, .. }
        | I::NumDiv { dst, .. }
        | I::NumMod { dst, .. }
        | I::FloatAdd { dst, .. }
        | I::FloatMul { dst, .. }
        | I::FloatMulAdd { dst, .. }
        | I::IntCmp { dst, .. }
        | I::IntAddI { dst, .. }
        | I::IntSubI { dst, .. }
        | I::IntMulI { dst, .. }
        | I::IntDivI { dst, .. }
        | I::IntModI { dst, .. }
        | I::IntCmpI { dst, .. }
        | I::IntModCmpI { dst, .. }
        | I::NumAddI { dst, .. }
        | I::NumSubI { dst, .. }
        | I::NumMulI { dst, .. }
        | I::NumDivI { dst, .. }
        | I::IntMulMod { dst, .. }
        | I::IntMulModI { dst, .. }
        | I::Index { dst, .. }
        | I::IndexArray { dst, .. }
        | I::IndexStringAscii { dst, .. }
        | I::Index2D { dst, .. }
        | I::GetProperty { dst, .. }
        | I::StrCat { dst, .. }
        | I::StrCat3 { dst, .. }
        | I::StringIndexOf { dst, .. }
        | I::StringContains { dst, .. }
        | I::StrCharEqRR { dst, .. }
        | I::StringConcat { dst, .. }
        | I::ArrayLen { dst, .. }
        | I::StringLen { dst, .. }
        | I::ArrayPop { dst, .. }
        | I::MakeArray2 { dst, .. }
        | I::Call { dst, .. }
        | I::MethodCall { dst, .. }
        | I::SuperCall { dst, .. } => {
            *dst = new_dst;
            true
        }
        _ => false,
    }
}
