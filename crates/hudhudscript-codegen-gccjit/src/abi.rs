//! gccjit tip/extern ABI tanımları: uniform giriş ABI'si, JitExit struct,
//! native-abi helper extern bildirimleri, handle dönüşüm kenarları.

use gccjit::{Context, Field, Function, Type};

/// Çeviri sırasındaki tipler (i64-handle lane: her şey i64/f64).
pub struct Abi<'ctx> {
    pub gcx: &'ctx Context<'ctx>,
    pub ll: Type<'ctx>,        // long long — i64 şeridi
    pub f64t: Type<'ctx>,      // double
    pub u32t: Type<'ctx>,      // unsigned int (argc)
    pub i32t: Type<'ctx>,      // int (status)
    pub cchar_ptr: Type<'ctx>, // const char*
    pub void_ptr: Type<'ctx>,  // void*
    pub jit_exit: Type<'ctx>,  // struct { int status; long long value; }
    pub f_status: Field<'ctx>, // int status
    pub f_value: Field<'ctx>,  // long long value
    pub exit_ptr: Type<'ctx>,  // JitExit*
    pub ll_ptr: Type<'ctx>,    // long long* (args; non-const — C ABI)
    pub void_ty: Type<'ctx>,   // void
}

pub fn build<'ctx>(gcx: &'ctx Context<'ctx>) -> Abi<'ctx> {
    use gccjit::Typeable;
    let ll = <i64>::get_type(gcx);
    let f64t = <f64>::get_type(gcx);
    let u32t = <u32>::get_type(gcx);
    let i32t = <i32>::get_type(gcx);
    let void_ptr = <*mut ()>::get_type(gcx);
    let char_ty = <i8>::get_type(gcx);
    let cchar_ptr = char_ty.make_pointer().make_const();
    let status_field = gcx.new_field(None, i32t, "status");
    let value_field = gcx.new_field(None, ll, "value");
    let jit_exit = gcx
        .new_struct_type(None, "HudhudJitExit", &[status_field, value_field])
        .as_type();
    let exit_ptr = jit_exit.make_pointer();
    let ll_ptr = ll.make_pointer();
    let void_ty = <()>::get_type(gcx);
    Abi {
        gcx,
        f_status: status_field,
        f_value: value_field,
        ll,
        f64t,
        u32t,
        i32t,
        cchar_ptr,
        void_ptr,
        jit_exit,
        exit_ptr,
        ll_ptr,
        void_ty,
    }
}

/// Extern helper bildirimi: isim + param tipleri → dönüş tipi.
pub fn declare_fn<'ctx>(
    abi: &Abi<'ctx>,
    name: &str,
    params: &[Type<'ctx>],
    ret: Type<'ctx>,
) -> Function<'ctx> {
    let ps: Vec<_> = params
        .iter()
        .enumerate()
        .map(|(i, t)| abi.gcx.new_parameter(None, *t, format!("p{i}")))
        .collect();
    abi.gcx
        .new_function(None, gccjit::FunctionType::Extern, ret, &ps, name, false)
}

/// hudhud_ptr_to_i64(void*) -> long long — handle'a dönüşüm kenarı
pub fn fn_ptr_to_i64<'ctx>(abi: &Abi<'ctx>) -> Function<'ctx> {
    declare_fn(abi, "hudhud_ptr_to_i64", &[abi.void_ptr], abi.ll)
}

/// hudhud_i64_to_ptr(long long) -> void* — handle'dan dönüşüm kenarı
pub fn fn_i64_to_ptr<'ctx>(abi: &Abi<'ctx>) -> Function<'ctx> {
    declare_fn(abi, "hudhud_i64_to_ptr", &[abi.ll], abi.void_ptr)
}

/// hudhud_print_int(long long) -> int
pub fn fn_print_int<'ctx>(abi: &Abi<'ctx>) -> Function<'ctx> {
    declare_fn(abi, "hudhud_print_int", &[abi.ll], abi.i32t)
}

/// hudhud_print_str(const char*) -> int
pub fn fn_print_str<'ctx>(abi: &Abi<'ctx>) -> Function<'ctx> {
    declare_fn(abi, "hudhud_print_str", &[abi.cchar_ptr], abi.i32t)
}

/// Uniform giriş parametreleri: (argc, args, out)
pub fn entry_params<'ctx>(abi: &Abi<'ctx>) -> Vec<gccjit::Parameter<'ctx>> {
    vec![
        abi.gcx.new_parameter(None, abi.u32t, "argc"),
        abi.gcx.new_parameter(None, abi.ll_ptr, "args"),
        abi.gcx.new_parameter(None, abi.exit_ptr, "out"),
    ]
}
