//! Ölü kod eleme (DCE): hiç kullanılmayan SAF sonuç tanımlarını kaldırır.
//! Yan etkili instruction'lar (çağrılar, push/set, safepoint, trap) ve
//! Param/Load tanımları ASLA kaldırılmaz.

use std::collections::HashSet;

use hudhudscript_mir::{MirFunction, MirInst, ValueId};

/// Bu instruction'ın dst'si hiçbir kullanımda yoksa kaldırılabilir mi?
fn removable(inst: &MirInst) -> bool {
    matches!(
        inst,
        MirInst::ConstInt { .. }
            | MirInst::ConstFloat { .. }
            | MirInst::ConstBool { .. }
            | MirInst::ConstNull { .. }
            | MirInst::ConstString { .. }
            | MirInst::Add { .. }
            | MirInst::Sub { .. }
            | MirInst::Mul { .. }
            | MirInst::Div { .. }
            | MirInst::Rem { .. }
            | MirInst::Neg { .. }
            | MirInst::Not { .. }
            | MirInst::Cmp { .. }
            | MirInst::LogicalAnd { .. }
            | MirInst::LogicalOr { .. }
            | MirInst::LogicalNot { .. }
            | MirInst::UnaryNeg { .. }
            | MirInst::StringConcat { .. }
            | MirInst::StringLen { .. }
            | MirInst::StringEq { .. }
            | MirInst::IntToString { .. }
            | MirInst::FloatToString { .. }
            | MirInst::StringSubstring { .. }
            | MirInst::IntToFloat { .. }
            | MirInst::ArrayNew { .. }
            | MirInst::ArrayGet { .. }
            | MirInst::ArrayLen { .. }
            | MirInst::ArrayJoin { .. }
            | MirInst::StringCharAt { .. }
            | MirInst::ObjectNew { .. }
            | MirInst::ObjectGet { .. }
            | MirInst::ObjectHas { .. }
            | MirInst::ObjectLen { .. }
    )
}

fn operands(inst: &MirInst) -> Vec<ValueId> {
    let mut v = Vec::new();
    let mut push = |x: &ValueId| v.push(*x);
    match inst {
        MirInst::Add { lhs, rhs, .. }
        | MirInst::Sub { lhs, rhs, .. }
        | MirInst::Mul { lhs, rhs, .. }
        | MirInst::Div { lhs, rhs, .. }
        | MirInst::Rem { lhs, rhs, .. }
        | MirInst::Cmp { lhs, rhs, .. }
        | MirInst::LogicalAnd { lhs, rhs, .. }
        | MirInst::LogicalOr { lhs, rhs, .. }
        | MirInst::StringConcat { lhs, rhs, .. }
        | MirInst::ArrayJoin { arr: lhs, sep: rhs, .. }
        | MirInst::StringEq { lhs, rhs, .. } => {
            push(lhs);
            push(rhs);
        }
        MirInst::Neg { src, .. }
        | MirInst::Not { src, .. }
        | MirInst::LogicalNot { src, .. }
        | MirInst::UnaryNeg { src, .. }
        | MirInst::StringLen { src, .. }
        | MirInst::IntToString { src, .. }
        | MirInst::FloatToString { src, .. }
        | MirInst::IntToFloat { src, .. }
        | MirInst::ArrayLen { arr: src, .. }
        | MirInst::ObjectLen { obj: src, .. } => push(src),
        MirInst::ArrayNew { capacity, .. } => push(capacity),
        MirInst::ArrayPush { arr, value } => {
            push(arr);
            push(value);
        }
        MirInst::ArrayGet { arr, index, .. } => {
            push(arr);
            push(index);
        }
        MirInst::ArrayPop { arr, .. } => push(arr),
        MirInst::StringCharAt { s, index, .. } => {
            push(s);
            push(index);
        }
        MirInst::StringSubstring { s, start, end, .. } => {
            push(s);
            push(start);
            push(end);
        }
        MirInst::ArraySet { arr, index, value } => {
            push(arr);
            push(index);
            push(value);
        }
        MirInst::ObjectSet { obj, key, value } => {
            push(obj);
            push(key);
            push(value);
        }
        MirInst::ObjectGet { obj, key, .. } | MirInst::ObjectHas { obj, key, .. } => {
            push(obj);
            push(key);
        }
        MirInst::Store { src, .. } => push(src),
        MirInst::CallStatic { args, .. } | MirInst::CallNative { args, .. } => {
            for a in args {
                push(a);
            }
        }
        _ => {}
    }
    v
}

/// Kullanılmayan saf tanımları kaldırır; sabit-fixpoint'e dek iterasyon.
pub fn dce(f: &MirFunction) -> (MirFunction, usize) {
    let mut out = f.clone();
    let mut total_removed = 0usize;
    loop {
        let mut used: HashSet<u32> = HashSet::new();
        for block in &out.blocks {
            // blok parametreleri (phi) KULLANIMDIR (tanım değil — kaldırılamazlar)
            for (_, vid) in &block.params {
                used.insert(vid.0);
            }
            for inst in &block.insts {
                for op in operands(inst) {
                    used.insert(op.0);
                }
            }
            if let Some(t) = &block.terminator {
                match t {
                    hudhudscript_mir::MirTerminator::Branch { args, .. } => {
                        for a in args {
                            used.insert(a.0);
                        }
                    }
                    hudhudscript_mir::MirTerminator::CondBranch {
                        cond, then_args, else_args, ..
                    } => {
                        used.insert(cond.0);
                        for a in then_args.iter().chain(else_args) {
                            used.insert(a.0);
                        }
                    }
                    hudhudscript_mir::MirTerminator::Return(v) => {
                        used.insert(v.0);
                    }
                    _ => {}
                }
            }
        }
        let mut removed = 0usize;
        for block in &mut out.blocks {
            let before = block.insts.len();
            block.insts.retain(|inst| {
                let keep = match inst.result_value() {
                    Some(d) => used.contains(&d.0) || !removable(inst),
                    None => true,
                };
                keep
            });
            removed += before - block.insts.len();
        }
        total_removed += removed;
        if removed == 0 {
            break;
        }
    }
    (out, total_removed)
}
