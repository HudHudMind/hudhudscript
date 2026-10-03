//! VM dispatch: packed integer arithmetic — D_INT_MUL_RR overflow must be
//! counted as a BigInt promotion.

use hudhudscript_vm::vm::dense_ops::D_INT_MUL_RR;
use hudhudscript_vm::vm::dispatch_int_arith::dispatch_int_arithmetic;
use hudhudscript_vm::vm::VM;
use hudhudscript_bytecode::{Bytecode, Value16};

/// Prove that D_INT_MUL_RR overflow is counted as BigInt promotion.
#[test]
fn packed_int_mul_rr_overflow_counts_promotion() {
    let mut vm = VM::new();

    // Values that overflow i64 when multiplied: 3037000500 * 3037000500
    let a = Value16::int(3037000500i64);
    let b = Value16::int(3037000500i64);
    let a_reg = 0u8;
    let b_reg = 1u8;
    let d_reg = 2u8;
    vm.registers[a_reg as usize] = a;
    vm.registers[b_reg as usize] = b;

    // arg2 encodes (a_reg << 8) | b_reg
    let arg2: u16 = ((a_reg as u16) << 8) | (b_reg as u16);

    let bc = Bytecode::new();
    let result = dispatch_int_arithmetic(&mut vm, D_INT_MUL_RR, d_reg, arg2, &bc, 0);
    assert!(result.is_ok());

    let actual = vm.registers[d_reg as usize];
    assert!(
        actual.is_bigint(),
        "overflow mul must be BigInt, got {:?}",
        actual
    );

    #[cfg(feature = "telemetry")]
    {
        let snap = vm.telemetry_snapshot();
        assert!(
            snap.bigint_promotion > 0,
            "packed D_INT_MUL_RR overflow must count promotion, got {}",
            snap.bigint_promotion
        );
    }
}
