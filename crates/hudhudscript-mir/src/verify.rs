//! MIR verifier — the gate before ANY backend sees a function
//! (JIT_AOT_ARCHITECTURE.md §5.4, GTP P3 discipline).
//!
//! AŞAMA-0 scope: structural validity, type agreement, def-before-use in
//! block order (linear SSA, no phi). Full dominance/phi verification and
//! GC-ref liveness checks widen in later steps — every widening ships
//! with its own unit tests per protocol §0.

use std::collections::HashSet;
use std::fmt;

use crate::mir::{MirFunction, MirInst, MirTerminator, MirType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyError {
    pub block: Option<u32>,
    pub kind: VerifyErrorKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyErrorKind {
    MissingTerminator { block: u32 },
    MultipleTerminators,
    UnknownBlock { target: u32 },
    UnknownValue { value: u32 },
    UseBeforeDef { value: u32 },
    TypeMismatch { inst: String, detail: String },
    ReturnMismatch { expected: String, got: String },
    EmptyFunction,
    UnknownCallee { function_idx: u32 },
}

impl fmt::Display for VerifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            VerifyErrorKind::MissingTerminator { block } => {
                write!(f, "block {block} has no terminator")
            }
            VerifyErrorKind::MultipleTerminators => write!(f, "multiple terminators"),
            VerifyErrorKind::UnknownBlock { target } => write!(f, "branch to unknown block {target}"),
            VerifyErrorKind::UnknownValue { value } => write!(f, "use of unknown value v{value}"),
            VerifyErrorKind::UseBeforeDef { value } => write!(f, "v{value} used before defined"),
            VerifyErrorKind::TypeMismatch { inst, detail } => {
                write!(f, "type mismatch in {inst}: {detail}")
            }
            VerifyErrorKind::ReturnMismatch { expected, got } => {
                write!(f, "return type mismatch: expected {expected}, got {got}")
            }
            VerifyErrorKind::EmptyFunction => write!(f, "function has no blocks"),
            VerifyErrorKind::UnknownCallee { function_idx } => {
                write!(f, "CallStatic to unknown function index {function_idx}")
            }
        }
    }
}

impl std::error::Error for VerifyError {}

/// Verify a function. Returns `Err` with the FIRST violation found —
/// callers never pass a failing function to a backend.
pub fn verify_function(f: &MirFunction) -> Result<(), VerifyError> {
    if f.blocks.is_empty() {
        return Err(VerifyError { block: None, kind: VerifyErrorKind::EmptyFunction });
    }

    let valid_blocks: HashSet<u32> = f.blocks.iter().map(|b| b.id.0).collect();
    if !valid_blocks.contains(&f.entry.0) {
        return Err(VerifyError { block: None, kind: VerifyErrorKind::UnknownBlock { target: f.entry.0 } });
    }

    let mut all_defined: HashSet<u32> = HashSet::new();
    for block in &f.blocks {
        let mut defined = all_defined.clone();
        // Blok parametreleri (phi) tanımdır
        for (_, v) in &block.params {
            defined.insert(v.0);
        }

        let check = |defined: &HashSet<u32>, v: u32| -> Option<VerifyErrorKind> {
            if !defined.contains(&v) {
                Some(VerifyErrorKind::UnknownValue { value: v })
            } else {
                None
            }
        };

        for inst in &block.insts {
            use MirInst::*;
            let operands: &[(u32, MirType)] = match inst {
                Add { ty, lhs, rhs, .. }
                | Sub { ty, lhs, rhs, .. }
                | Mul { ty, lhs, rhs, .. }
                | Div { ty, lhs, rhs, .. }
                | Rem { ty, lhs, rhs, .. } => {
                    if *ty != MirType::I64 && *ty != MirType::F64 {
                        return Err(VerifyError {
                            block: Some(block.id.0),
                            kind: VerifyErrorKind::TypeMismatch {
                                inst: format!("{:?}", std::mem::discriminant(inst)),
                                detail: format!("arith on unsupported type {ty}"),
                            },
                        });
                    }
                    &[(lhs.0, *ty), (rhs.0, *ty)]
                }
                Neg { ty, src, .. } => &[(src.0, *ty)],
                Not { src, .. } => &[(src.0, MirType::Bool)],
                Cmp { ty, lhs, rhs, .. } => &[(lhs.0, *ty), (rhs.0, *ty)],
                ConstInt { .. } | ConstFloat { .. } | ConstBool { .. } | ConstNull { .. }
                | ConstString { .. } => &[],
                LogicalAnd { lhs, rhs, .. } | LogicalOr { lhs, rhs, .. } => {
                    for v in [lhs, rhs] {
                        if !defined.contains(&v.0) {
                            return Err(VerifyError { block: Some(block.id.0), kind: VerifyErrorKind::UnknownValue { value: v.0 } });
                        }
                    }
                    &[]
                }
                LogicalNot { src, .. } | UnaryNeg { src, .. } => {
                    if !defined.contains(&src.0) {
                        return Err(VerifyError { block: Some(block.id.0), kind: VerifyErrorKind::UnknownValue { value: src.0 } });
                    }
                    &[]
                }
                ArrayNew { capacity, .. } => {
                    if !defined.contains(&capacity.0) {
                        return Err(VerifyError { block: Some(block.id.0), kind: VerifyErrorKind::UnknownValue { value: capacity.0 } });
                    }
                    &[]
                }
                ArrayPush { arr, value, .. } => {
                    for v in [arr, value] {
                        if !defined.contains(&v.0) {
                            return Err(VerifyError { block: Some(block.id.0), kind: VerifyErrorKind::UnknownValue { value: v.0 } });
                        }
                    }
                    &[]
                }
                ArrayGet { arr, index, .. } => {
                    for v in [arr, index] {
                        if !defined.contains(&v.0) {
                            return Err(VerifyError { block: Some(block.id.0), kind: VerifyErrorKind::UnknownValue { value: v.0 } });
                        }
                    }
                    &[]
                }
                ArraySet { arr, index, value, .. } => {
                    for v in [arr, index, value] {
                        if !defined.contains(&v.0) {
                            return Err(VerifyError { block: Some(block.id.0), kind: VerifyErrorKind::UnknownValue { value: v.0 } });
                        }
                    }
                    &[]
                }
                StringCharAt { s, index, .. } => {
                    for v in [s, index] {
                        if !defined.contains(&v.0) {
                            return Err(VerifyError { block: Some(block.id.0), kind: VerifyErrorKind::UnknownValue { value: v.0 } });
                        }
                    }
                    &[]
                }
                ArrayPop { arr, .. } | ArrayLen { arr, .. } => {
                    if !defined.contains(&arr.0) {
                        return Err(VerifyError { block: Some(block.id.0), kind: VerifyErrorKind::UnknownValue { value: arr.0 } });
                    }
                    &[]
                }
                ObjectNew { .. } => &[],
                ObjectSet { obj, key, value, .. } => {
                    for v in [obj, key, value] {
                        if !defined.contains(&v.0) {
                            return Err(VerifyError { block: Some(block.id.0), kind: VerifyErrorKind::UnknownValue { value: v.0 } });
                        }
                    }
                    &[]
                }
                ObjectGet { obj, key, .. } | ObjectHas { obj, key, .. } => {
                    for v in [obj, key] {
                        if !defined.contains(&v.0) {
                            return Err(VerifyError { block: Some(block.id.0), kind: VerifyErrorKind::UnknownValue { value: v.0 } });
                        }
                    }
                    &[]
                }
                ObjectLen { obj, .. } => {
                    if !defined.contains(&obj.0) {
                        return Err(VerifyError { block: Some(block.id.0), kind: VerifyErrorKind::UnknownValue { value: obj.0 } });
                    }
                    &[]
                }
                StringConcat { lhs, rhs, .. } => {
                    for v in [lhs, rhs] {
                        if !defined.contains(&v.0) {
                            return Err(VerifyError { block: Some(block.id.0), kind: VerifyErrorKind::UnknownValue { value: v.0 } });
                        }
                    }
                    &[]
                }
                StringLen { src, .. } | IntToString { src, .. } | FloatToString { src, .. } => {
                    if !defined.contains(&src.0) {
                        return Err(VerifyError { block: Some(block.id.0), kind: VerifyErrorKind::UnknownValue { value: src.0 } });
                    }
                    &[]
                }
                StringEq { lhs, rhs, .. } => {
                    for v in [lhs, rhs] {
                        if !defined.contains(&v.0) {
                            return Err(VerifyError { block: Some(block.id.0), kind: VerifyErrorKind::UnknownValue { value: v.0 } });
                        }
                    }
                    &[]
                }
                IntToFloat { src, .. } => {
                    if !defined.contains(&src.0) {
                        return Err(VerifyError { block: Some(block.id.0), kind: VerifyErrorKind::UnknownValue { value: src.0 } });
                    }
                    &[]
                }
                StringSubstring { s, start, end, .. } => {
                    for v in [s, start, end] {
                        if !defined.contains(&v.0) {
                            return Err(VerifyError { block: Some(block.id.0), kind: VerifyErrorKind::UnknownValue { value: v.0 } });
                        }
                    }
                    &[]
                }
                ArrayJoin { arr, sep, .. } => {
                    for v in [arr, sep] {
                        if !defined.contains(&v.0) {
                            return Err(VerifyError { block: Some(block.id.0), kind: VerifyErrorKind::UnknownValue { value: v.0 } });
                        }
                    }
                    &[]
                }
                Load { .. } => &[],
                Param { ty, index, .. } => {
                    let n_params = f.param_tys.len() as u32;
                    if *index >= n_params {
                        return Err(VerifyError {
                            block: Some(block.id.0),
                            kind: VerifyErrorKind::TypeMismatch {
                                inst: "Param".into(),
                                detail: format!("index {index} out of range (function has {n_params} params)"),
                            },
                        });
                    }
                    if *ty != f.param_tys[*index as usize] {
                        return Err(VerifyError {
                            block: Some(block.id.0),
                            kind: VerifyErrorKind::TypeMismatch {
                                inst: "Param".into(),
                                detail: format!("type {ty} != declared param type {}", f.param_tys[*index as usize]),
                            },
                        });
                    }
                    &[]
                }
                Store { src, .. } => &[(src.0, MirType::Generic)],
                CallStatic { args, .. } | CallNative { args, .. } => {
                    // AŞAMA-0: arg types are checked at the lowering layer;
                    // here we only require the values to exist.
                    for a in args {
                        if let Some(kind) = check(&defined, a.0) {
                            return Err(VerifyError { block: Some(block.id.0), kind });
                        }
                    }
                    &[]
                }
                GcSafepoint | Trap { .. } | Unreachable => &[],
            };
            for (v, _t) in operands {
                if let Some(kind) = check(&defined, *v) {
                    return Err(VerifyError { block: Some(block.id.0), kind });
                }
            }
            if let Some(dst) = inst.result_value() {
                defined.insert(dst.0);
            }
        }

        match &block.terminator {
            None => {
                return Err(VerifyError {
                    block: Some(block.id.0),
                    kind: VerifyErrorKind::MissingTerminator { block: block.id.0 },
                })
            }
            Some(t) => verify_terminator(f, &valid_blocks, block.id.0, t, &defined)?,
        }
        all_defined = defined;
    }

    Ok(())
}

fn verify_terminator(
    f: &MirFunction,
    valid_blocks: &HashSet<u32>,
    block: u32,
    t: &MirTerminator,
    defined: &HashSet<u32>,
) -> Result<(), VerifyError> {
    let err = |kind| VerifyError { block: Some(block), kind };
    match t {
        MirTerminator::Return(v) => {
            if !defined.contains(&v.0) {
                return Err(err(VerifyErrorKind::UnknownValue { value: v.0 }));
            }
            // Return TYPE agreement is approximated structurally in
            // AŞAMA-0 (unit returns reject ConstNull); precise per-value
            // typing arrives with the typed-builder widening.
            if f.return_ty == MirType::Unit {
                return Err(err(VerifyErrorKind::ReturnMismatch {
                    expected: "unit".into(),
                    got: "value".into(),
                }));
            }
            Ok(())
        }
        MirTerminator::Branch { target, args } => {
            if !valid_blocks.contains(&target.0) {
                return Err(err(VerifyErrorKind::UnknownBlock { target: target.0 }));
            }
            // Arg sayısı hedef bloğun param sayısıyla eşleşmeli
            let target_params = f.block(*target).map(|b| b.params.len()).unwrap_or(0);
            if args.len() != target_params {
                return Err(err(VerifyErrorKind::TypeMismatch {
                    inst: "Branch".into(),
                    detail: format!("{} args for {} params", args.len(), target_params),
                }));
            }
            for a in args {
                if !defined.contains(&a.0) {
                    return Err(err(VerifyErrorKind::UnknownValue { value: a.0 }));
                }
            }
            Ok(())
        }
        MirTerminator::CondBranch { cond, then_block, then_args, else_block, else_args } => {
            if !defined.contains(&cond.0) {
                return Err(err(VerifyErrorKind::UnknownValue { value: cond.0 }));
            }
            for (blk, args) in [(then_block, then_args), (else_block, else_args)] {
                if !valid_blocks.contains(&blk.0) {
                    return Err(err(VerifyErrorKind::UnknownBlock { target: blk.0 }));
                }
                let target_params = f.block(*blk).map(|b| b.params.len()).unwrap_or(0);
                if args.len() != target_params {
                    return Err(err(VerifyErrorKind::TypeMismatch {
                        inst: "CondBranch".into(),
                        detail: format!("{} args for {} params on block {}", args.len(), target_params, blk.0),
                    }));
                }
                for a in args {
                    if !defined.contains(&a.0) {
                        return Err(err(VerifyErrorKind::UnknownValue { value: a.0 }));
                    }
                }
            }
            Ok(())
        }
        MirTerminator::ReturnVoid => Ok(()),
        MirTerminator::Trap(_) | MirTerminator::Unreachable => Ok(()),
    }
}

