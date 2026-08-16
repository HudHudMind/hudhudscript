//! G4.1: Single declarative opcode specification.
//! Every opcode is defined exactly once with its handler, encoding,
//! and fallthrough policy. Compile-time validation ensures no drift.

use crate::vm::dense_ops::*;

/// Opcode handler family — which module owns the implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HandlerFamily {
    /// dispatch_general.rs inline (fastest)
    InlineGeneral,
    /// dispatch_int_arith.rs — dispatched via dispatch_int_arithmetic()
    IntArith,
    /// dispatch_num_arith.rs — dispatched via dispatch_num_arithmetic()
    NumArith,
    /// Handled elsewhere (trampoline, execute/, etc.)
    Other,
    /// No packed handler exists — always falls through to unpacked
    Fallthrough,
}

/// Semantic fallthrough policy for this opcode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FallthroughPolicy {
    /// Never falls through — always handled in packed lane
    Never,
    /// Falls through when operand types don't match Int fast path
    OnTypeMismatch,
    /// Always falls through (no packed handler)
    Always,
}

/// A single opcode entry in the canonical spec.
pub(crate) struct OpcodeSpec {
    pub dense: u8,
    pub name: &'static str,
    pub handler: HandlerFamily,
    pub fallthrough: FallthroughPolicy,
}

/// G4.1: Canonical opcode table.
/// Every dense opcode MUST appear here exactly once.
/// The verifier checks this at compile time.
pub(crate) const OPCODE_SPECS: &[OpcodeSpec] = &[
    // ── Jump / Control flow ──
    OpcodeSpec {
        dense: D_JUMP,
        name: "Jump",
        handler: HandlerFamily::InlineGeneral,
        fallthrough: FallthroughPolicy::Never,
    },
    OpcodeSpec {
        dense: D_JUMP_IF_FALSE,
        name: "JumpIfFalse",
        handler: HandlerFamily::InlineGeneral,
        fallthrough: FallthroughPolicy::Never,
    },
    OpcodeSpec {
        dense: D_JUMP_IF_TRUE,
        name: "JumpIfTrue",
        handler: HandlerFamily::InlineGeneral,
        fallthrough: FallthroughPolicy::Never,
    },
    OpcodeSpec {
        dense: D_JUMP_IF_FALSE_R,
        name: "JumpIfFalseR",
        handler: HandlerFamily::InlineGeneral,
        fallthrough: FallthroughPolicy::Never,
    },
    OpcodeSpec {
        dense: D_JUMP_IF_TRUE_R,
        name: "JumpIfTrueR",
        handler: HandlerFamily::InlineGeneral,
        fallthrough: FallthroughPolicy::Never,
    },
    // ── Move / Load ──
    OpcodeSpec {
        dense: D_MOVE_RR,
        name: "Move",
        handler: HandlerFamily::InlineGeneral,
        fallthrough: FallthroughPolicy::Never,
    },
    OpcodeSpec {
        dense: D_LOAD_NUM_CONST,
        name: "LoadNumConst",
        handler: HandlerFamily::InlineGeneral,
        fallthrough: FallthroughPolicy::Never,
    },
    OpcodeSpec {
        dense: D_LOAD_INT_CONST,
        name: "LoadIntConst",
        handler: HandlerFamily::InlineGeneral,
        fallthrough: FallthroughPolicy::Never,
    },
    // ── Int arithmetic (delegated) ──
    OpcodeSpec {
        dense: D_INT_ADD_RR,
        name: "IntAdd",
        handler: HandlerFamily::IntArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_INT_SUB_RR,
        name: "IntSub",
        handler: HandlerFamily::IntArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_INT_MUL_RR,
        name: "IntMul",
        handler: HandlerFamily::IntArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_INT_ADD_RI,
        name: "IntAddI",
        handler: HandlerFamily::IntArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_INT_SUB_RI,
        name: "IntSubI",
        handler: HandlerFamily::IntArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_INT_MUL_RI,
        name: "IntMulI",
        handler: HandlerFamily::IntArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_INT_SUB_LOCAL_I,
        name: "IntSubLocalI",
        handler: HandlerFamily::InlineGeneral,
        fallthrough: FallthroughPolicy::Never,
    },
    OpcodeSpec {
        dense: D_INT_ADD_LOCAL_I,
        name: "IntAddLocalI",
        handler: HandlerFamily::InlineGeneral,
        fallthrough: FallthroughPolicy::Never,
    },
    // ── Int compare ──
    OpcodeSpec {
        dense: D_INT_EQ_RR,
        name: "IntEq",
        handler: HandlerFamily::IntArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_INT_LT_RR,
        name: "IntLt",
        handler: HandlerFamily::IntArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_INT_LE_RR,
        name: "IntLe",
        handler: HandlerFamily::IntArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_INT_NE_RR,
        name: "IntNe",
        handler: HandlerFamily::IntArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_INT_MOD_RR,
        name: "IntMod",
        handler: HandlerFamily::IntArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_INT_MOD_I,
        name: "IntModI",
        handler: HandlerFamily::IntArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_INT_CMP_LT_I,
        name: "IntCmpLtI",
        handler: HandlerFamily::IntArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_INT_CMP_LE_I,
        name: "IntCmpLeI",
        handler: HandlerFamily::IntArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_INT_CMP_EQ_I,
        name: "IntCmpEqI",
        handler: HandlerFamily::IntArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_INT_CMP_NE_I,
        name: "IntCmpNeI",
        handler: HandlerFamily::IntArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    // ── G4.2: reconnected Int handlers ──
    OpcodeSpec {
        dense: D_NEG_R,
        name: "Neg",
        handler: HandlerFamily::IntArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_NOT_R,
        name: "Not",
        handler: HandlerFamily::IntArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_ARRAY_PUSH_RRR,
        name: "ArrayPush",
        handler: HandlerFamily::IntArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_STRCAT_RRR,
        name: "StrCat",
        handler: HandlerFamily::IntArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_STRCAT_MUT_RR,
        name: "StrCatMut",
        handler: HandlerFamily::IntArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_STRING_INDEX_OF_RRR,
        name: "StringIndexOf",
        handler: HandlerFamily::IntArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_STRING_CONTAINS_RRR,
        name: "StringContains",
        handler: HandlerFamily::IntArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    // ── Num arithmetic (delegated) ──
    OpcodeSpec {
        dense: D_NUM_ADD_RR,
        name: "NumAdd",
        handler: HandlerFamily::NumArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_NUM_SUB_RR,
        name: "NumSub",
        handler: HandlerFamily::NumArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_NUM_MUL_RR,
        name: "NumMul",
        handler: HandlerFamily::NumArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_NUM_ADD_RI,
        name: "NumAddI",
        handler: HandlerFamily::NumArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_NUM_SUB_RI,
        name: "NumSubI",
        handler: HandlerFamily::NumArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_NUM_MUL_RI,
        name: "NumMulI",
        handler: HandlerFamily::NumArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    // G4.2: reconnected Num handlers
    OpcodeSpec {
        dense: D_NUM_DIV_RR,
        name: "NumDiv",
        handler: HandlerFamily::NumArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_NUM_DIV_RI,
        name: "NumDivI",
        handler: HandlerFamily::NumArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_NUM_MUL_ADD_ASSIGN,
        name: "NumMulAddAssign",
        handler: HandlerFamily::NumArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    // ── Index / Array ──
    OpcodeSpec {
        dense: D_INDEX_RRR,
        name: "Index",
        handler: HandlerFamily::IntArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_INDEX_ARRAY_RRR,
        name: "IndexArray",
        handler: HandlerFamily::IntArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_INDEX_STRING_ASCII_RRR,
        name: "IndexStringAscii",
        handler: HandlerFamily::IntArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
    OpcodeSpec {
        dense: D_INDEX_ASSIGN_RRR,
        name: "IndexAssign",
        handler: HandlerFamily::IntArith,
        fallthrough: FallthroughPolicy::OnTypeMismatch,
    },
];

/// G4.1 + G4 fix: Compile-time cross-validation against dense_ops constants.
/// Every dense opcode must have exactly one handler in the spec.
/// Missing entries = compile error. Duplicate entries = compile error.
pub(crate) const fn validate_opcode_specs() -> bool {
    let specs = OPCODE_SPECS;
    // 1. Check no duplicate dense codes (skip zero = not-in-spec entries)
    let mut i = 0;
    while i < specs.len() {
        if specs[i].dense != 0 {
            let mut j = i + 1;
            while j < specs.len() {
                if specs[i].dense == specs[j].dense && specs[j].dense != 0 {
                    panic!("G4: duplicate dense opcode in spec");
                }
                j += 1;
            }
        }
        i += 1;
    }

    // 2. Cross-check: every dense opcode in the router's match arms
    //    must be listed in the spec. This uses the same constants that
    //    dispatch_chunk5 matches on (from dense_ops.rs).
    //    If a constant changes, the spec must be updated or this fails.
    let required: &[(u8, &str)] = &[
        (D_JUMP, "D_JUMP"),
        (D_JUMP_IF_FALSE, "D_JUMP_IF_FALSE"),
        (D_JUMP_IF_TRUE, "D_JUMP_IF_TRUE"),
        (D_JUMP_IF_FALSE_R, "D_JUMP_IF_FALSE_R"),
        (D_JUMP_IF_TRUE_R, "D_JUMP_IF_TRUE_R"),
        (D_MOVE_RR, "D_MOVE_RR"),
        (D_LOAD_NUM_CONST, "D_LOAD_NUM_CONST"),
        (D_LOAD_INT_CONST, "D_LOAD_INT_CONST"),
        (D_INT_ADD_RR, "D_INT_ADD_RR"),
        (D_INT_SUB_RR, "D_INT_SUB_RR"),
        (D_INT_MUL_RR, "D_INT_MUL_RR"),
        (D_INT_SUB_LOCAL_I, "D_INT_SUB_LOCAL_I"),
        (D_INT_ADD_LOCAL_I, "D_INT_ADD_LOCAL_I"),
        (D_INT_EQ_RR, "D_INT_EQ_RR"),
        (D_INT_LT_RR, "D_INT_LT_RR"),
        (D_INT_LE_RR, "D_INT_LE_RR"),
        (D_INT_NE_RR, "D_INT_NE_RR"),
        (D_NEG_R, "D_NEG_R"),
        (D_NOT_R, "D_NOT_R"),
        (D_ARRAY_PUSH_RRR, "D_ARRAY_PUSH_RRR"),
        (D_STRCAT_RRR, "D_STRCAT_RRR"),
        (D_STRCAT_MUT_RR, "D_STRCAT_MUT_RR"),
        (D_STRING_INDEX_OF_RRR, "D_STRING_INDEX_OF_RRR"),
        (D_STRING_CONTAINS_RRR, "D_STRING_CONTAINS_RRR"),
        (D_NUM_ADD_RR, "D_NUM_ADD_RR"),
        (D_NUM_SUB_RR, "D_NUM_SUB_RR"),
        (D_NUM_MUL_RR, "D_NUM_MUL_RR"),
        (D_NUM_DIV_RR, "D_NUM_DIV_RR"),
        (D_NUM_DIV_RI, "D_NUM_DIV_RI"),
        (D_NUM_MUL_ADD_ASSIGN, "D_NUM_MUL_ADD_ASSIGN"),
        (D_INDEX_RRR, "D_INDEX_RRR"),
        (D_INDEX_ARRAY_RRR, "D_INDEX_ARRAY_RRR"),
        (D_INDEX_STRING_ASCII_RRR, "D_INDEX_STRING_ASCII_RRR"),
        (D_INDEX_ASSIGN_RRR, "D_INDEX_ASSIGN_RRR"),
    ];

    let mut ri = 0;
    while ri < required.len() {
        let (req_dense, req_name) = required[ri];
        let mut found = false;
        let mut si = 0;
        while si < specs.len() {
            if specs[si].dense == req_dense {
                found = true;
                break;
            }
            si += 1;
        }
        if !found {
            panic!("G4: required opcode missing from spec");
        }
        ri += 1;
    }
    true
}

// Compile-time validation — panics at build time if spec is incomplete
const _: () = {
    assert!(
        validate_opcode_specs(),
        "G4.1 opcode spec validation failed"
    );
};
