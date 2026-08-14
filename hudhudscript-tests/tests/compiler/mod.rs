//mod a3b_int_literal_emission; (stack VM)
//mod a3c_disasm_fib; (stack VM)
mod architecture_health;
mod assignment_fusion_tests;
//mod inline_unit; (stack VM)
mod inline_regression_tests;
mod real_parity_tests;
mod test;
mod test_controlflow;
//mod test_decls_and_serialization; (disabled)
//mod test_inline; (stack VM)
//mod vm_debug_hooks; (stack VM)
//mod vm_parity_tests; (stack VM)
mod b5_local_direct_regs_test;
mod duplicate_function_test;
mod index_regression_tests;
mod p2_self_add_test;
mod p3_call_arg_move_test;
mod p7_f2_verify_fusion;
mod p7_f3_verify_new_fusions;
mod p9_source_fallback_test;
mod sop_class_tests;
mod vm_perf_bench;
mod vm_regression_kimi;
mod vm_tco_tests;
mod g2_jump_if_true_parity_test;
// G3: disabled — fusion_emitted field only exists on exp/perf-agent branch.
// Re-enable when G3 telemetry-gated fusion counter is implemented and merged.
mod g3_fusion_emit_tests;
mod unparenthesized_control_flow_tests;
mod deep_nested_scope_tests;

