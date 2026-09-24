//! Blok terminator'ları ve JitExit çıkış yazımı (translate alt modülü).

use gccjit::{BinaryOp, Block, RValue, ToRValue};
use hudhudscript_codegen::backend::BackendError;
use hudhudscript_mir::{MirBlock, MirTerminator, ValueId};

use super::FnCx;

pub(crate) fn translate_terminator<'ctx, 'a>(
    cx: &mut FnCx<'ctx, 'a>,
    gb: Block<'ctx>,
    blk: &hudhudscript_mir::MirBlock,
) -> Result<(), BackendError> {
    let g = cx.abi.gcx;
    let out = cx.func.get_param(2).to_rvalue();
    match &blk.terminator {
        Some(MirTerminator::Branch { target, args }) => {
            assign_phis(cx, gb, blk.id.0, target.0, args)?;
            let dst = cx.blocks[&target.0];
            gb.end_with_jump(None, dst);
        }
        Some(MirTerminator::CondBranch { cond, then_block, then_args, else_block, else_args }) => {
            // gccjit koşullu dallanma _Bool bekler; i64 şeridinden cast
            use gccjit::Typeable;
            let raw = cx.val(*cond)?;
            let c = g.new_cast(None, raw, <bool>::get_type(g));
            // phi atamaları dallanma öncesi ortak blokta yapılamaz (değerler
            // kenara bağlı). Çözüm: kenar başına ara blok — phi atamaları
            // orada yapılır, sonra hedefe atlanır (C'deki denklemin do ğrusu).
            let then_dst = edge_block(cx, gb, "then", then_block.0, then_args)?;
            let else_dst = edge_block(cx, gb, "else", else_block.0, else_args)?;
            gb.end_with_conditional(None, c, then_dst, else_dst);
        }
        Some(MirTerminator::ReturnVoid) => {
            write_exit(cx, gb, &out, cx.ll(0), None, Some(cx.dz_acc.to_rvalue()));
        }
        Some(MirTerminator::Return(v)) => {
            let raw = cx.val(*v)?;
            // F64 şeridi: bit pattern i64'e taşınır (cranelift bitcast_f64_to_i64
            // denklemi — sayısal çevrim DEĞİL; çalışma zamanı tarafı çözer)
            let val = if cx.float_vals.contains(&v.0) {
                g.new_call(None, cx.ext.i64_from_f64, &[raw])
            } else {
                raw
            };
            let ov = cx.ov_acc.to_rvalue();
            let dz = cx.dz_acc.to_rvalue();
            write_exit(cx, gb, &out, val, Some(ov), Some(dz));
        }
        Some(MirTerminator::Trap(_)) => {
            write_exit_status(cx, gb, &out, 1);
        }
        Some(MirTerminator::Unreachable) | None => {
            // gccjit tüm blokların sonlanmasını ister; boşsa void return
            write_exit_status(cx, gb, &out, 0);
        }
    }
    Ok(())
}

fn assign_phis<'ctx, 'a>(
    cx: &FnCx<'ctx, 'a>,
    gb: Block<'ctx>,
    _from: u32,
    to: u32,
    args: &[ValueId],
) -> Result<(), BackendError> {
    let g = cx.abi.gcx;
    // SSA phi semantiği: bir kenarın TÜM kaynak değerleri, hedef phi
    // local'lerine YAZILMADAN önce okunmalıdır. Sıralı C atamalarında
    // `l_a := l_b; l_b := l_a` ikinci okumada çoktan ezilmiş l_a'yı görür —
    // döngü kenarındaki takas (game_of_life a/b swap'ı) böyle bozuluyordu.
    // İki aşama: önce dönüştürülmüş değerleri TAZE geçici local'lere yaz,
    // sonra phi local'lerine kopyala (GCC SSA'ya yükseltip fazlasını atar).
    let mut staged: Vec<(gccjit::LValue<'ctx>, gccjit::LValue<'ctx>)> = Vec::new();
    for (i, a) in args.iter().enumerate() {
        let raw = cx.val(*a)?;
        let local = cx
            .phi_locals
            .get(&(to, i as u32))
            .copied()
            .ok_or_else(|| cx.err(&format!("phi local b{}#{} missing", to, i)))?;
        // Tipli phi: f64 phi f64 değer ister (int → sayısal cast);
        // i64 phi f64 argüman alırsa bit deseni helper'ıyla taşınır
        let is_f64_phi = cx.phi_f64.contains(&(to, i as u32));
        let v = if is_f64_phi {
            if cx.float_vals.contains(&a.0) {
                raw
            } else {
                g.new_cast(None, raw, cx.abi.f64t)
            }
        } else if cx.float_vals.contains(&a.0) {
            g.new_call(None, cx.ext.i64_from_f64, &[raw])
        } else {
            raw
        };
        let ty = if is_f64_phi { cx.abi.f64t } else { cx.abi.ll };
        let tmp = cx.func.new_local(None, ty, format!("phi_edge_b{}_{}", to, i));
        gb.add_assignment(None, tmp, v);
        staged.push((local, tmp));
    }
    for (local, tmp) in staged {
        gb.add_assignment(None, local, tmp.to_rvalue());
    }
    Ok(())
}

/// Koşullu dal kenarı için ara blok: phi atamaları kenar bloğunda yapılır,
/// sonra hedefe atlanır. Boş args kenarda phi yok demektir — hedef doğrudan
/// döner (gereksiz blok oluşturulmaz).
fn edge_block<'ctx, 'a>(
    cx: &FnCx<'ctx, 'a>,
    _gb: Block<'ctx>,
    label: &str,
    to: u32,
    args: &[ValueId],
) -> Result<Block<'ctx>, BackendError> {
    if args.is_empty() {
        return Ok(cx.blocks[&to]);
    }
    let edge = cx.func.new_block(format!("edge_{label}_{to}"));
    assign_phis(cx, edge, 0, to, args)?;
    edge.end_with_jump(None, cx.blocks[&to]);
    Ok(edge)
}

fn write_exit<'ctx, 'a>(
    cx: &FnCx<'ctx, 'a>,
    gb: Block<'ctx>,
    out: &RValue<'ctx>,
    val: RValue<'ctx>,
    ov: Option<RValue<'ctx>>,
    dz: Option<RValue<'ctx>>,
) {
    let g = cx.abi.gcx;
    // status = select(ov, OVERFLOW, select(dz, DIVZERO, RETURNED))
    let st_returned = cx.ll(0);
    let st_overflow = cx.ll(1);
    let st_divzero = cx.ll(2);
    let mut status = st_returned;
    if let Some(d) = dz {
        status = g.new_call(None, cx.ext.select_i64, &[d, st_divzero, status]);
    }
    if let Some(o) = ov {
        status = g.new_call(None, cx.ext.select_i64, &[o, st_overflow, status]);
    }
    let st_field = out.dereference_field(None, cx.abi.f_status);
    gb.add_assignment(None, st_field, g.new_cast(None, status, cx.abi.i32t));
    let val_field = out.dereference_field(None, cx.abi.f_value);
    gb.add_assignment(None, val_field, val);
    gb.end_with_void_return(None);
    let _ = (st_returned, st_overflow, st_divzero);
}

fn write_exit_status<'ctx, 'a>(
    cx: &FnCx<'ctx, 'a>,
    gb: Block<'ctx>,
    out: &RValue<'ctx>,
    status: i64,
) {
    let g = cx.abi.gcx;
    let st_field = out.dereference_field(None, cx.abi.f_status);
    gb.add_assignment(None, st_field, g.new_rvalue_from_long(cx.abi.i32t, status as i64));
    let val_field = out.dereference_field(None, cx.abi.f_value);
    gb.add_assignment(None, val_field, cx.ll(0));
    gb.end_with_void_return(None);
}
