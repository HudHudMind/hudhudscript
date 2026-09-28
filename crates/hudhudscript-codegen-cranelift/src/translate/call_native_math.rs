//! Math, assertion, and timing helpers for CallNative.

use cranelift::prelude::types::{F64, I64};
use cranelift::prelude::{AbiParam, FunctionBuilder, InstBuilder, Value};
use cranelift_module::{Linkage, Module};

use hudhudscript_codegen::backend::BackendError;
use hudhudscript_mir::{MirFunction, MirType, RuntimeHelperId, ValueId};

use super::helpers::{operand, record, reject};

type Env = Vec<Option<(Value, MirType)>>;

pub(super) fn translate_math_or_assert<M: Module>(
    helper: &RuntimeHelperId,
    dst: ValueId,
    args: &[ValueId],
    builder: &mut FunctionBuilder,
    env: &mut Env,
    module: &mut M,
    func: &MirFunction,
) -> Result<bool, BackendError> {
    match helper {
        RuntimeHelperId::DateMillis => {
            let sig = {
                let mut sig = module.make_signature();
                sig.returns.push(AbiParam::new(I64));
                sig
            };
            let id = module
                .declare_function("hudhud_date_millis", Linkage::Import, &sig)
                .map_err(|e| reject(func, "DateMillis", &format!("declare: {e}")))?;
            let fref = module.declare_func_in_func(id, builder.func);
            let inst = builder.ins().call(fref, &[]);
            let result = *builder.inst_results(inst).first()
                .ok_or_else(|| reject(func, "DateMillis", "helper returned no value"))?;
            record(env, dst, result, MirType::I64);
            Ok(true)
        }
        RuntimeHelperId::MathSin | RuntimeHelperId::MathSqrt | RuntimeHelperId::MathCos
        | RuntimeHelperId::MathFloor | RuntimeHelperId::MathAbs
        | RuntimeHelperId::MathPow | RuntimeHelperId::MathMin | RuntimeHelperId::MathMax => {
            let (name, what, two_arg) = match helper {
                RuntimeHelperId::MathSin => ("hudhud_math_sin", "MathSin", false),
                RuntimeHelperId::MathSqrt => ("hudhud_math_sqrt", "MathSqrt", false),
                RuntimeHelperId::MathCos => ("hudhud_math_cos", "MathCos", false),
                RuntimeHelperId::MathFloor => ("hudhud_math_floor", "MathFloor", false),
                RuntimeHelperId::MathAbs => ("hudhud_math_abs", "MathAbs", false),
                RuntimeHelperId::MathPow => ("hudhud_math_pow", "MathPow", true),
                RuntimeHelperId::MathMin => ("hudhud_math_min", "MathMin", true),
                RuntimeHelperId::MathMax => ("hudhud_math_max", "MathMax", true),
                _ => unreachable!("math arm guarded by outer match"),
            };
            let expected_args = if two_arg { 2 } else { 1 };
            if args.len() != expected_args {
                return Err(reject(func, what, if two_arg { "takes exactly 2 args" } else { "takes exactly 1 arg" }));
            }
            let sig = {
                let mut sig = module.make_signature();
                sig.params.push(AbiParam::new(F64));
                if two_arg {
                    sig.params.push(AbiParam::new(F64));
                }
                sig.returns.push(AbiParam::new(F64));
                sig
            };
            let id = module
                .declare_function(name, Linkage::Import, &sig)
                .map_err(|e| reject(func, what, &format!("declare: {e}")))?;
            let fref = module.declare_func_in_func(id, builder.func);
            let inst = if two_arg {
                let (a, _) = operand(env, args[0], func)?;
                let (b, _) = operand(env, args[1], func)?;
                builder.ins().call(fref, &[a, b])
            } else {
                let (v, _) = operand(env, args[0], func)?;
                builder.ins().call(fref, &[v])
            };
            let result = *builder.inst_results(inst).first()
                .ok_or_else(|| reject(func, what, "helper returned no value"))?;
            record(env, dst, result, MirType::F64);
            Ok(true)
        }
        RuntimeHelperId::AssertEq
        | RuntimeHelperId::AssertApprox
        | RuntimeHelperId::AssertTrue
        | RuntimeHelperId::AssertFalse => {
            let (name, what, f64_lane, arity) = match helper {
                RuntimeHelperId::AssertEq => ("hudhud_assert_eq", "AssertEq", false, 2usize),
                RuntimeHelperId::AssertApprox => ("hudhud_assert_approx", "AssertApprox", true, 2),
                RuntimeHelperId::AssertTrue => ("hudhud_assert_true", "AssertTrue", false, 1),
                _ => ("hudhud_assert_false", "AssertFalse", false, 1),
            };
            if args.len() != arity {
                return Err(reject(func, what, "wrong arg count"));
            }
            let sig = {
                let mut sig = module.make_signature();
                for _ in 0..arity {
                    if f64_lane {
                        sig.params.push(AbiParam::new(F64));
                    } else {
                        sig.params.push(AbiParam::new(I64));
                    }
                }
                sig
            };
            let id = module
                .declare_function(name, Linkage::Import, &sig)
                .map_err(|e| reject(func, what, &format!("declare: {e}")))?;
            let fref = module.declare_func_in_func(id, builder.func);
            let vals: Vec<_> = args.iter().map(|a| operand(env, *a, func).unwrap().0).collect();
            builder.ins().call(fref, &vals);
            Ok(true)
        }
        _ => Ok(false),
    }
}
