//! Sabit yayılımı (const propagation): saf sabit tanımlarının kullanımları
//! orijinal sabit tanımına yeniden yönlendirilir. const_fold ile birlikte
//! çalışır — zincirleme `dummy+1` blokları tek sabite katlanır.

use hudhudscript_mir::{MirFunction, MirInst, ValueId};

/// Bir instruction saf VE sabit mi? (aynı değer her yerde)
fn const_def(inst: &MirInst) -> Option<ValueId> {
    match inst {
        MirInst::ConstInt { dst, .. }
        | MirInst::ConstBool { dst, .. }
        | MirInst::ConstNull { dst }
        | MirInst::ConstFloat { dst, .. }
        | MirInst::ConstString { dst, .. } => Some(*dst),
        _ => None,
    }
}

/// Tüm kullanımları alias üzerinden yazar; dönen: değiştirilen operand sayısı.
pub fn const_prop(f: &MirFunction) -> (MirFunction, usize) {
    let mut out = f.clone();
    // value id → kök sabit value id (union-find benzeri takip)
    let mut alias: Vec<Option<ValueId>> = Vec::new();
    let grow = |alias: &mut Vec<Option<ValueId>>, v: ValueId| {
        while alias.len() <= v.0 as usize {
            alias.push(None);
        }
    };
    for block in &out.blocks {
        for inst in &block.insts {
            if let Some(c) = const_def(inst) {
                grow(&mut alias, c);
                // sabit tanımlarının kendisi zaten kök
                alias[c.0 as usize] = Some(c);
            }
        }
    }
    let resolve = |alias: &[Option<ValueId>], v: ValueId| -> ValueId {
        let mut cur = v;
        while let Some(Some(next)) = alias.get(cur.0 as usize) {
            if *next == cur {
                break;
            }
            cur = *next;
        }
        cur
    };

    let mut changed = 0usize;
    let mut rewrite = |v: &ValueId| -> ValueId {
        let r = resolve(&alias, *v);
        if r != *v {
            changed += 1;
        }
        r
    };

    for block in &mut out.blocks {
        for inst in &mut block.insts {
            match inst {
                MirInst::Add { dst, lhs, rhs, ty }
                | MirInst::Sub { dst, lhs, rhs, ty }
                | MirInst::Mul { dst, lhs, rhs, ty }
                | MirInst::Div { dst, lhs, rhs, ty }
                | MirInst::Rem { dst, lhs, rhs, ty } => {
                    let _ = (dst, ty);
                    *lhs = rewrite(lhs);
                    *rhs = rewrite(rhs);
                }
                MirInst::Cmp { dst, lhs, rhs, .. } => {
                    let _ = dst;
                    *lhs = rewrite(lhs);
                    *rhs = rewrite(rhs);
                }
                MirInst::LogicalAnd { dst, lhs, rhs }
                | MirInst::LogicalOr { dst, lhs, rhs } => {
                    let _ = dst;
                    *lhs = rewrite(lhs);
                    *rhs = rewrite(rhs);
                }
                MirInst::Neg { dst, src, .. }
                | MirInst::Not { dst, src }
                | MirInst::LogicalNot { dst, src } => {
                    let _ = dst;
                    *src = rewrite(src);
                }
                MirInst::UnaryNeg { dst, src, .. } => {
                    let _ = dst;
                    *src = rewrite(src);
                }
                MirInst::StringConcat { dst, lhs, rhs }
                | MirInst::StringEq { dst, lhs, rhs } => {
                    let _ = dst;
                    *lhs = rewrite(lhs);
                    *rhs = rewrite(rhs);
                }
                MirInst::ArrayJoin { dst, arr, sep } => {
                    let _ = dst;
                    *arr = rewrite(arr);
                    *sep = rewrite(sep);
                }
                MirInst::StringSubstring { dst, s, start, end } => {
                    let _ = dst;
                    *s = rewrite(s);
                    *start = rewrite(start);
                    *end = rewrite(end);
                }
                MirInst::IntToFloat { dst, src } => {
                    let _ = dst;
                    *src = rewrite(src);
                }
                MirInst::StringLen { dst, src }
                | MirInst::IntToString { dst, src }
                | MirInst::FloatToString { dst, src } => {
                    let _ = dst;
                    *src = rewrite(src);
                }
                MirInst::Store { src, .. } => {
                    *src = rewrite(src);
                }
                MirInst::ArrayNew { dst, capacity } => {
                    let _ = dst;
                    *capacity = rewrite(capacity);
                }
                MirInst::ArrayPush { arr, value } => {
                    *arr = rewrite(arr);
                    *value = rewrite(value);
                }
                MirInst::ArrayGet { dst, arr, index, .. } => {
                    let _ = dst;
                    *arr = rewrite(arr);
                    *index = rewrite(index);
                }
                MirInst::ArraySet { arr, index, value } => {
                    *arr = rewrite(arr);
                    *index = rewrite(index);
                    *value = rewrite(value);
                }
                MirInst::ArrayPop { dst, arr } => {
                    let _ = dst;
                    *arr = rewrite(arr);
                }
                MirInst::StringCharAt { dst, s, index } => {
                    let _ = dst;
                    *s = rewrite(s);
                    *index = rewrite(index);
                }
                MirInst::ArrayLen { dst, arr } => {
                    let _ = dst;
                    *arr = rewrite(arr);
                }
                MirInst::ObjectSet { obj, key, value } => {
                    *obj = rewrite(obj);
                    *key = rewrite(key);
                    *value = rewrite(value);
                }
                MirInst::ObjectGet { dst, obj, key, .. }
                | MirInst::ObjectHas { dst, obj, key } => {
                    let _ = dst;
                    *obj = rewrite(obj);
                    *key = rewrite(key);
                }
                MirInst::ObjectLen { dst, obj } => {
                    let _ = dst;
                    *obj = rewrite(obj);
                }
                MirInst::CallStatic { dst, args, .. } => {
                    let _ = dst;
                    for a in args.iter_mut() {
                        *a = rewrite(a);
                    }
                }
                MirInst::CallNative { dst, args, .. } => {
                    let _ = dst;
                    for a in args.iter_mut() {
                        *a = rewrite(a);
                    }
                }
                _ => {}
            }
        }
        // terminator kullanımları
        if let Some(t) = &mut block.terminator {
            match t {
                hudhudscript_mir::MirTerminator::Branch { args, .. } => {
                    for a in args.iter_mut() {
                        *a = rewrite(a);
                    }
                }
                hudhudscript_mir::MirTerminator::CondBranch {
                    cond, then_args, else_args, ..
                } => {
                    *cond = rewrite(cond);
                    for a in then_args.iter_mut() {
                        *a = rewrite(a);
                    }
                    for a in else_args.iter_mut() {
                        *a = rewrite(a);
                    }
                }
                hudhudscript_mir::MirTerminator::Return(v) => {
                    *v = rewrite(v);
                }
                _ => {}
            }
        }
    }
    (out, changed)
}
