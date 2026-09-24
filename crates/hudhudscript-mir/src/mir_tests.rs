use super::*;

#[test]
fn inst_result_metadata() {
    let i = MirInst::ConstInt { dst: ValueId(0), ty: MirType::I64, value: 3 };
    assert_eq!(i.result_ty(), Some(MirType::I64));
    assert_eq!(i.result_value(), Some(ValueId(0)));

    let g = MirInst::GcSafepoint;
    assert_eq!(g.result_ty(), None);
    assert_eq!(g.result_value(), None);

    let c = MirInst::Cmp {
        dst: ValueId(1),
        op: CmpOp::Lt,
        ty: MirType::I64,
        lhs: ValueId(0),
        rhs: ValueId(0),
    };
    assert_eq!(c.result_ty(), Some(MirType::Bool));
}
