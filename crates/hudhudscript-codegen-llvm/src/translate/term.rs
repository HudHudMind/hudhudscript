//! Terminator'lar ve JitExit çıkışı (LLVM backend alt modülü).

use hudhudscript_codegen::backend::BackendError;
use hudhudscript_mir::{MirBlock, MirTerminator};
use inkwell::values::{BasicValue, BasicValueEnum, FunctionValue, IntValue, PointerValue};
use inkwell::IntPredicate;

use super::FnCx;

pub(crate) fn translate_terminator<'ctx, 'm>(
    cx: &mut FnCx<'ctx, 'm>,
    func: FunctionValue<'ctx>,
    blk: &MirBlock,
) -> Result<(), BackendError> {

    let out_ptr = func.get_nth_param(2).unwrap().into_pointer_value();
    // Gerçek öncül blok: arith promote kolları bloğu bölebilir — phi
    // incoming'ler orijinal MIR bloğundan değil, terminator'ın yazıldığı
    // GÜNCEL bloktan kaydedilmelidir (aksi halde bozuk IR).
    let cur = cx.b.get_insert_block().expect("builder positioned");
    match &blk.terminator {
        Some(MirTerminator::Branch { target, args }) => {
            // hedef philerine incoming ekle (phi tipine göre: f64 phi f64 değer)
            for (i, a) in args.iter().enumerate() {
                let v = phi_incoming(cx, target.0, i, *a, "pba")?;
                if let Some(phi) = cx.phis.get(&(target.0, i as u32)) {
                    phi.add_incoming(&[(&v, cur)]);
                }
            }
            cx.b.build_unconditional_branch(cx.blocks[&target.0]);
        }
        Some(MirTerminator::CondBranch { cond, then_block, then_args, else_block, else_args }) => {
            let c = cx.ival(*cond)?;
            let cb = cx.b.build_int_compare(IntPredicate::NE, c, cx.i64t.const_zero(), "condb");
            for (i, a) in then_args.iter().enumerate() {
                let v = phi_incoming(cx, then_block.0, i, *a, "pbt")?;
                if let Some(phi) = cx.phis.get(&(then_block.0, i as u32)) {
                    phi.add_incoming(&[(&v, cur)]);
                }
            }
            for (i, a) in else_args.iter().enumerate() {
                let v = phi_incoming(cx, else_block.0, i, *a, "pbe")?;
                if let Some(phi) = cx.phis.get(&(else_block.0, i as u32)) {
                    phi.add_incoming(&[(&v, cur)]);
                }
            }
            cx.b.build_conditional_branch(cb, cx.blocks[&then_block.0], cx.blocks[&else_block.0]);
        }
        Some(MirTerminator::ReturnVoid) => {
            let dz = cx.b.build_load(cx.dz_acc, "dzr").into_int_value();
            write_exit(cx, &out_ptr, cx.i64t.const_zero(), None, Some(dz));
        }
        Some(MirTerminator::Return(v)) => {
            let raw = cx.val(*v)?;
            let val = match raw {
                // f64 dönüş: bit deseni i64'e taşınır (cranelift/gccjit bitcast
                // denklemi — sayısal çevrim DEĞİL; çalışma zamanı çözer)
                BasicValueEnum::FloatValue(f) => {
                    cx.b.build_bitcast(f, cx.i64t, "fretbits").into_int_value()
                }
                BasicValueEnum::IntValue(i) => {
                    if i.get_type().get_bit_width() < 64 {
                        cx.b.build_int_s_extend(i, cx.i64t, "zret")
                    } else {
                        i
                    }
                }
                _ => cx.i64t.const_zero(),
            };
            let ov = cx.b.build_load(cx.ov_acc, "ovr").into_int_value();
            let dz = cx.b.build_load(cx.dz_acc, "dzr").into_int_value();
            write_exit(cx, &out_ptr, val, Some(ov), Some(dz));
        }
        Some(MirTerminator::Trap(_)) => {
            let stp = unsafe { cx.b.build_struct_gep(out_ptr, 0, "stp").unwrap() };
            cx.b.build_store(stp, cx.i32t.const_int(1, false));
            let vp = unsafe { cx.b.build_struct_gep(out_ptr, 1, "vp").unwrap() };
            cx.b.build_store(vp, cx.i64t.const_zero());
            cx.b.build_return(None);
        }
        Some(MirTerminator::Unreachable) | None => {
            cx.b.build_unreachable();
        }
    }
    Ok(())
}

/// Phi incoming değeri: hedef phi f64 ise f64 değer (int arg sayısal promote),
/// i64 phi ise i64 değer (f64 arg bit deseniyle taşınır).
fn phi_incoming<'ctx, 'm>(
    cx: &FnCx<'ctx, 'm>,
    to: u32,
    i: usize,
    a: hudhudscript_mir::ValueId,
    name: &str,
) -> Result<BasicValueEnum<'ctx>, BackendError> {
    if cx.phi_f64.contains(&(to, i as u32)) {
        Ok(cx.fval(a)?.as_basic_value_enum())
    } else if cx.is_f64(a) {
        Ok(cx.b.build_bitcast(cx.fval(a)?, cx.i64t, name).into_int_value().as_basic_value_enum())
    } else {
        Ok(cx.ival(a)?.as_basic_value_enum())
    }
}

fn write_exit<'ctx, 'm>(
    cx: &FnCx<'ctx, 'm>,
    out_ptr: &PointerValue<'ctx>,
    val: IntValue<'ctx>,
    ov: Option<IntValue<'ctx>>,
    dz: Option<IntValue<'ctx>>,
) {
    // status = select(ov, 1, select(dz, 2, 0)) — bayraklar i64 akümülatör yüklemesi
    let mut status = cx.i32t.const_zero();
    if let Some(d) = dz {
        status = cx.b.build_select(
            cx.b.build_int_compare(IntPredicate::NE, d, cx.i64t.const_zero(), "dzb"),
            cx.i32t.const_int(2, false),
            status,
            "stdz",
        ).into_int_value();
    }
    if let Some(o) = ov {
        status = cx.b.build_select(
            cx.b.build_int_compare(IntPredicate::NE, o, cx.i64t.const_zero(), "ovb"),
            cx.i32t.const_int(1, false),
            status,
            "stov",
        ).into_int_value();
    }
    let stp = unsafe { cx.b.build_struct_gep(*out_ptr, 0, "stp").unwrap() };
    cx.b.build_store(stp, status);
    let vp = unsafe { cx.b.build_struct_gep(*out_ptr, 1, "vp").unwrap() };
    cx.b.build_store(vp, val);
    cx.b.build_return(None);
}
