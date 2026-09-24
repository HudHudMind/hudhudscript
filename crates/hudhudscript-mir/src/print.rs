//! MIR pretty-printer — powers `hudhud inspect mir` (JIT_AOT_ARCHITECTURE
//! §19.4) and differential debugging output.

use std::fmt::Write;

use crate::mir::{
    CmpOp, MirFunction, MirInst, MirTerminator, RuntimeHelperId, TrapKind,
};

/// Render a function in the document's MIR text form (§5.3).
pub fn render_function(f: &MirFunction) -> String {
    let mut out = String::new();
    let params: Vec<String> = f.param_tys.iter().map(|t| t.to_string()).collect();
    let _ = writeln!(
        out,
        "fn @{} ({}) -> {} {{",
        f.name,
        params.join(", "),
        f.return_ty
    );
    for block in &f.blocks {
        if block.params.is_empty() {
            let _ = writeln!(out, "block{}:", block.id);
        } else {
            let ps: Vec<String> = block.params.iter()
                .map(|(ty, v)| format!("v{v}: {ty}"))
                .collect();
            let _ = writeln!(out, "block{}({}):", block.id, ps.join(", "));
        }
        for inst in &block.insts {
            let _ = writeln!(out, "  {}", render_inst(inst));
        }
        if let Some(t) = &block.terminator {
            let _ = writeln!(out, "  {}", render_terminator(t));
        }
    }
    let _ = writeln!(out, "}}");
    out
}

fn render_inst(i: &MirInst) -> String {
    use MirInst::*;
    match i {
        Add { dst, ty, lhs, rhs } => {
            format!("v{dst} = {ty}.add v{lhs}, v{rhs}")
        }
        Sub { dst, ty, lhs, rhs } => format!("v{dst} = {ty}.sub v{lhs}, v{rhs}"),
        Mul { dst, ty, lhs, rhs } => format!("v{dst} = {ty}.mul v{lhs}, v{rhs}"),
        Div { dst, ty, lhs, rhs } => format!("v{dst} = {ty}.div v{lhs}, v{rhs}"),
        Rem { dst, ty, lhs, rhs } => format!("v{dst} = {ty}.rem v{lhs}, v{rhs}"),
        Neg { dst, ty, src } => format!("v{dst} = {ty}.neg v{src}"),
        Not { dst, src } => format!("v{dst} = not v{src}"),
        Cmp { dst, op, ty, lhs, rhs } => {
            format!("v{dst} = cmp.{} {} v{lhs}, v{rhs}", op.mnemonic(), ty)
        }
        ConstInt { dst, ty, value } => format!("v{dst} = const.{ty} {value}"),
        ConstFloat { dst, ty, bits } => {
            format!("v{dst} = const.{ty} {bits} ; {}", f64::from_bits(*bits))
        }
        ConstBool { dst, value } => format!("v{dst} = const.bool {value}"),
        ConstNull { dst } => format!("v{dst} = const.null"),
        ConstString { dst, index } => format!("v{dst} = const.str[{index}]"),
        StringConcat { dst, lhs, rhs } => format!("v{dst} = str.concat v{lhs}, v{rhs}"),
        StringLen { dst, src } => format!("v{dst} = str.len v{src}"),
        StringEq { dst, lhs, rhs } => format!("v{dst} = str.eq v{lhs}, v{rhs}"),
        IntToString { dst, src } => format!("v{dst} = str.from_int v{src}"),
        FloatToString { dst, src } => format!("v{dst} = str.from_float v{src}"),
        IntToFloat { dst, src } => format!("v{dst} = f64(v{src})"),
        StringSubstring { dst, s, start, end } => format!("v{dst} = str.sub(v{s}, v{start}, v{end})"),
        ArrayJoin { dst, arr, sep } => format!("v{dst} = arr.join(v{arr}, v{sep})"),
        LogicalAnd { dst, lhs, rhs } => format!("v{dst} = and v{lhs}, v{rhs}"),
        LogicalOr { dst, lhs, rhs } => format!("v{dst} = or v{lhs}, v{rhs}"),
        LogicalNot { dst, src } => format!("v{dst} = not v{src}"),
        UnaryNeg { dst, src, .. } => format!("v{dst} = neg v{src}"),
        ArrayNew { dst, capacity } => format!("v{dst} = arr.new(v{capacity})"),
        ArrayPush { arr, value } => format!("arr.push(v{arr}, v{value})"),
        ArrayGet { dst, arr, index, .. } => format!("v{dst} = arr.get(v{arr}, v{index})"),
        ArraySet { arr, index, value } => format!("arr.set(v{arr}, v{index}, v{value})"),
        ArrayLen { dst, arr } => format!("v{dst} = arr.len(v{arr})"),
        ArrayPop { dst, arr } => format!("v{dst} = arr.pop(v{arr})"),
        StringCharAt { dst, s, index } => format!("v{dst} = str[{index}](v{s})"),
        ObjectNew { dst } => format!("v{dst} = obj.new()"),
        ObjectSet { obj, key, value } => format!("obj.set(v{obj}, key=v{key}, v{value})"),
        ObjectGet { dst, obj, key, .. } => format!("v{dst} = obj.get(v{obj}, key=v{key})"),
        ObjectHas { dst, obj, key } => format!("v{dst} = obj.has(v{obj}, key=v{key})"),
        ObjectLen { dst, obj } => format!("v{dst} = obj.len(v{obj})"),
        Load { dst, ty, local } => format!("v{dst} = load.{ty} l{local}"),
        Param { dst, ty, index } => format!("v{dst} = param{index} : {ty}"),
        Store { local, src } => format!("store l{local}, v{src}"),
        CallStatic { dst, ty, callee, args } => {
            let a = fmt_args(args);
            format!("v{dst} = call @{callee}({a}) -> {ty}")
        }
        CallNative { dst, ty, helper, args } => {
            let a = fmt_args(args);
            let h = match helper {
                RuntimeHelperId::Print => "hudhud_print",
        RuntimeHelperId::PrintStr => "hudhud_print_str",
                RuntimeHelperId::ThrowOverflow => "hudhud_throw_overflow",
                RuntimeHelperId::DateMillis => "hudhud_date_millis",
                RuntimeHelperId::MathSin => "hudhud_math_sin",
                RuntimeHelperId::MathSqrt => "hudhud_math_sqrt",
                RuntimeHelperId::MathCos => "hudhud_math_cos",
                RuntimeHelperId::MathFloor => "hudhud_math_floor",
                RuntimeHelperId::MathAbs => "hudhud_math_abs",
                RuntimeHelperId::MathPow => "hudhud_math_pow",
                RuntimeHelperId::MathMin => "hudhud_math_min",
                RuntimeHelperId::MathMax => "hudhud_math_max",
                RuntimeHelperId::GlobalsHandle => "hudhud_globals",
                RuntimeHelperId::GlobalGet => "hudhud_global_get",
                RuntimeHelperId::GlobalSet => "hudhud_global_set",
                RuntimeHelperId::StringToInt => "hudhud_string_to_int",
                RuntimeHelperId::StringSplit => "hudhud_string_split",
                RuntimeHelperId::StringIndexOf => "hudhud_string_index_of",
                RuntimeHelperId::TypeOf => "hudhud_typeof",
                RuntimeHelperId::StringCmp => "hudhud_string_cmp",
                RuntimeHelperId::StringAppend => "hudhud_string_append",
                RuntimeHelperId::Throw => "hudhud_throw",
                RuntimeHelperId::HasException => "hudhud_has_exception",
                RuntimeHelperId::Catch => "hudhud_catch",
                RuntimeHelperId::ArrayFilled => "hudhud_array_filled",
                RuntimeHelperId::ArrayFill => "hudhud_array_fill",
            };
            format!("v{dst} = native {h}({a}) -> {ty}")
        }
        GcSafepoint => "gc.safepoint".to_string(),
        Trap { kind } => format!("trap {}", trap_name(kind)),
        Unreachable => "unreachable".to_string(),
    }
}

fn render_terminator(t: &MirTerminator) -> String {
    match t {
        MirTerminator::Return(v) => format!("return v{v}"),
        MirTerminator::ReturnVoid => "return-void".to_string(),
        MirTerminator::Branch { target, args } => {
            if args.is_empty() {
                format!("br block{target}")
            } else {
                let a = fmt_args(args);
                format!("br block{target}({a})")
            }
        }
        MirTerminator::CondBranch { cond, then_block, then_args, else_block, else_args } => {
            if then_args.is_empty() && else_args.is_empty() {
                format!("br v{cond}, block{then_block}, block{else_block}")
            } else {
                let ta = fmt_args(then_args);
                let ea = fmt_args(else_args);
                format!("br v{cond}, block{then_block}({ta}), block{else_block}({ea})")
            }
        }
        MirTerminator::Trap(kind) => format!("trap {}", trap_name(kind)),
        MirTerminator::Unreachable => "unreachable".to_string(),
    }
}

fn fmt_args(args: &[crate::mir::ValueId]) -> String {
    args.iter().map(|v| format!("v{v}")).collect::<Vec<_>>().join(", ")
}

fn trap_name(k: &TrapKind) -> &'static str {
    match k {
        TrapKind::Unreachable => "unreachable",
        TrapKind::DivisionByZero => "div_by_zero",
        TrapKind::IntegerOverflow => "int_overflow",
    }
}

impl CmpOp {
    pub fn mnemonic(self) -> &'static str {
        match self {
            CmpOp::Eq => "eq",
            CmpOp::Ne => "ne",
            CmpOp::Lt => "lt",
            CmpOp::Le => "le",
            CmpOp::Gt => "gt",
            CmpOp::Ge => "ge",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::{BinOp, MirFunctionBuilder};
    use crate::mir::MirType;

    #[test]
    fn renders_add_example() {
        let mut b = MirFunctionBuilder::new("add", vec![MirType::I64, MirType::I64], MirType::I64);
        let e = b.entry();
        let x = b.const_i64(e, 2);
        let y = b.const_i64(e, 3);
        let s = b.bin(e, BinOp::Add, MirType::I64, x, y);
        b.ret(e, s);
        let text = render_function(&b.finish());
        assert!(text.contains("fn @add (i64, i64) -> i64 {"), "header: {text}");
        assert!(text.contains("v0 = const.i64 2"), "{text}");
        assert!(text.contains("v2 = i64.add v0, v1"), "{text}");
        assert!(text.contains("return v2"), "{text}");
    }

    #[test]
    fn renders_native_call_and_safepoint() {
        let mut b = MirFunctionBuilder::new("main", vec![], MirType::Unit);
        let e = b.entry();
        let five = b.const_i64(e, 5);
        b.call_native(e, MirType::Unit, RuntimeHelperId::Print, vec![five]);
        b.gc_safepoint(e);
        b.ret(e, five); // Unit dönüş henüz desteklenmiyor — Return hedefi
        let text = render_function(&b.finish());
        assert!(text.contains("native hudhud_print(v0) -> unit"), "{text}");
        assert!(text.contains("gc.safepoint"), "{text}");
    }
}
