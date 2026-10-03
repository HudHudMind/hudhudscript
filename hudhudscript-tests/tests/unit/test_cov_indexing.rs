//! Coverage tests for the VM indexing execution paths
//! (`crates/hudhudscript-vm/src/vm/execute/indexing.rs` and its 2D row
//! fallback). Every test compiles a real script, runs it on the VM and pins
//! exact values or exact runtime-error substrings taken from the source.
use hudhudscript_bytecode::{Bytecode, Instruction, Value16};
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn compile(src: &str) -> Bytecode {
    let ast = parse(src).expect("test source must parse");
    let mut compiler = Compiler::new();
    compiler.compile(&ast).expect("test source must compile")
}

fn run(src: &str) -> Result<VM, hudhudscript_errors::Error> {
    let bytecode = compile(src);
    let mut vm = VM::new();
    vm.execute(&bytecode)?;
    Ok(vm)
}

fn run_ok(src: &str) -> VM {
    run(src).unwrap_or_else(|e| panic!("script must run cleanly: {}", e.message))
}

fn expect_error(src: &str, needle: &str) {
    let message = match run(src) {
        Err(e) => e.message,
        Ok(_) => panic!("script must fail at runtime: `{}`", src),
    };
    assert!(
        message.contains(needle),
        "wrong error for `{}`: expected substring {:?}, got: {}",
        src,
        needle,
        message
    );
}

fn var(vm: &VM, name: &str) -> Value16 {
    vm.get_variable_owned(name)
        .unwrap_or_else(|| panic!("variable '{}' must be published", name))
}

fn int_of(vm: &VM, name: &str) -> i64 {
    var(vm, name)
        .as_int()
        .unwrap_or_else(|| panic!("'{}' must hold an int", name))
}

fn string_of(vm: &VM, name: &str) -> String {
    var(vm, name)
        .as_string()
        .unwrap_or_else(|| panic!("'{}' must hold a string", name))
}

fn count_matching(bytecode: &Bytecode, pred: fn(&Instruction) -> bool) -> usize {
    let in_main = bytecode.instructions.iter().filter(|i| pred(i)).count();
    let chunks = bytecode.functions.borrow();
    let in_functions: usize = chunks
        .iter()
        .map(|chunk| chunk.instructions.iter().filter(|i| pred(i)).count())
        .sum();
    in_main + in_functions
}

fn is_index(i: &Instruction) -> bool {
    matches!(i, Instruction::Index { .. })
}
fn is_index_array(i: &Instruction) -> bool {
    matches!(i, Instruction::IndexArray { .. })
}
fn is_index_string_ascii(i: &Instruction) -> bool {
    matches!(i, Instruction::IndexStringAscii { .. })
}
fn is_index_2d(i: &Instruction) -> bool {
    matches!(i, Instruction::Index2D { .. })
}
fn is_index_assign(i: &Instruction) -> bool {
    matches!(i, Instruction::IndexAssign { .. })
}
fn is_index_assign_array(i: &Instruction) -> bool {
    matches!(i, Instruction::IndexAssignArray { .. })
}
fn is_index_assign_2d(i: &Instruction) -> bool {
    matches!(i, Instruction::IndexAssign2D { .. })
}

// ── array reads: specialized IndexArray and generic Index ─────────────

#[test]
fn array_read_specialized_positions() {
    let src = "let arr = [10, 20, 30]; let i0 = 0; let i1 = 1; let i2 = 2; \
               let a = arr[i0]; let b = arr[i1]; let c = arr[i2];";
    assert!(
        count_matching(&compile(src), is_index_array) >= 3,
        "array-typed locals must lower to IndexArray"
    );
    let vm = run_ok(src);
    assert_eq!(int_of(&vm, "a"), 10, "arr[0]"); assert_eq!(int_of(&vm, "b"), 20, "arr[1]");
    assert_eq!(int_of(&vm, "c"), 30, "last index arr[2]");
}

#[test]
fn array_and_string_read_generic_index() {
    // `id()` hides the argument type from call-site typing so `at` lowers to
    // the generic Index. The body compiles once (call sites emit Call), so
    // the program holds exactly one generic Index, inside the `at` chunk.
    let src = r#"
        fn id(x) { return x; }
        fn at(a, i) { return a[i]; }
        let x = at(id([10, 20, 30]), 1);
        let s = at(id("abc"), 2);
    "#;
    assert!(
        count_matching(&compile(src), is_index) == 1,
        "unproven objects must lower to generic Index"
    );
    let vm = run_ok(src);
    assert_eq!(int_of(&vm, "x"), 20, "generic array read");
    assert_eq!(string_of(&vm, "s"), "c", "generic string read");
}

#[test]
fn array_read_out_of_bounds_upper() {
    expect_error(
        "let arr = [1, 2, 3]; let i = 7; let x = arr[i];",
        "Array index out of bounds: 7",
    );
    expect_error(
        "fn id(x) { return x; } fn at(a, i) { return a[i]; } let x = at(id([1]), 4);",
        "Array index out of bounds: 4",
    );
}

#[test]
fn array_negative_index_rejected() {
    // Negative indices are invalid (index_helpers::index_i64_to_usize);
    // needles use the packed fast-path wording that actually executes
    // (dispatch_int_arith.rs D_INDEX_ARRAY_RRR / D_INDEX_RRR).
    expect_error(
        "let arr = [1, 2, 3]; let i = -1; let x = arr[i];",
        "IndexArray: index must be non-negative number",
    );
    expect_error(
        "fn id(x) { return x; } fn at(a, i) { return a[i]; } let x = at(id([1]), id(-2));",
        "Array index must be a non-negative number",
    );
}

#[test]
fn array_non_numeric_index_rejected() {
    // Generic Index on an array runs via the packed D_INDEX_RRR fast path
    // (dispatch_int_arith.rs), whose message differs from the step helper's.
    let base = "fn id(x) { return x; } fn at(a, i) { return a[i]; }";
    expect_error(
        &format!("{} let x = at(id([1, 2]), id(\"k\"));", base),
        "Array index must be a non-negative number",
    );
    expect_error(
        &format!("{} let x = at(id([1, 2]), null);", base),
        "Array index must be a non-negative number",
    );
    expect_error(
        &format!("{} let x = at(id([1, 2]), true);", base),
        "Array index must be a non-negative number",
    );
}

#[test]
fn array_float_index_resolves_to_element() {
    // numeric_index_i64 accepts Number indices (doc: `arr[1.0]` keeps working).
    let vm = run_ok("fn id(x) { return x; } fn at(a, i) { return a[i]; } let x = at(id([10, 20, 30]), 1.0);");
    assert_eq!(int_of(&vm, "x"), 20, "arr[1.0] must resolve like arr[1]");
    let vm = run_ok("let arr = [10, 20, 30]; let f = 1.0; let y = arr[f];");
    assert_eq!(int_of(&vm, "y"), 20, "IndexArray accepts Number index");
}

#[test]
fn index_on_scalar_and_null_rejected() {
    let base = "fn id(x) { return x; } fn at(a, i) { return a[i]; }";
    expect_error(&format!("{} let x = at(id(5), 0);", base), "Index: expected array or string");
    expect_error(&format!("{} let x = at(id(null), 0);", base), "Index: expected array or string");
}

// ── string reads: ASCII fast path and non-ASCII fallback ──────────────

#[test]
fn string_read_ascii_positions() {
    let src = "let s = \"Hello\"; let i0 = 0; let i4 = 4; let a = s[i0]; let b = s[i4];";
    assert!(
        count_matching(&compile(src), is_index_string_ascii) >= 2,
        "string-typed locals must lower to IndexStringAscii"
    );
    let vm = run_ok(src);
    assert_eq!(string_of(&vm, "a"), "H", "s[0]"); assert_eq!(string_of(&vm, "b"), "o", "s[4]");
}

#[test]
fn string_read_non_ascii_falls_back_to_chars() {
    // Non-ASCII bytes miss the fast path and resolve via chars().nth().
    let vm = run_ok("let s = \"ğa\"; let i0 = 0; let i1 = 1; let a = s[i0]; let b = s[i1];");
    assert_eq!(string_of(&vm, "a"), "ğ", "first char of \"ğa\"");
    assert_eq!(string_of(&vm, "b"), "a", "second char of \"ğa\"");
    let vm = run_ok("let m = \"ağb\"; let k = 2; let c = m[k];");
    assert_eq!(string_of(&vm, "c"), "b", "\"ağb\"[2] skips the 2-byte ğ");
}

#[test]
fn string_read_errors() {
    expect_error("let s = \"ab\"; let i = 5; let c = s[i];", "String index out of bounds: 5");
    expect_error(
        "fn id(x) { return x; } fn at(a, i) { return a[i]; } let c = at(id(\"ab\"), 9);",
        "String index out of bounds: 9",
    );
    expect_error(
        "let s = \"ab\"; let i = -1; let c = s[i];",
        "IndexStringAscii: index must be non-negative number",
    );
}

// ── map reads via generic Index ────────────────────────────────────────

#[test]
fn object_read_by_string_key() {
    let src = "let obj = { a: 41, b: 7 }; let ka = \"a\"; let kz = \"zz\"; \
               let hit = obj[ka]; let miss = obj[kz];";
    assert!(
        count_matching(&compile(src), is_index) >= 2,
        "object-typed locals must lower to generic Index"
    );
    let vm = run_ok(src);
    assert_eq!(int_of(&vm, "hit"), 41, "obj[\"a\"]");
    assert!(var(&vm, "miss").is_null(), "missing key must read as null");
}

// ── nested reads: generic chain and fused Index2D ─────────────────────

#[test]
fn nested_generic_index_chain_resolves() {
    let src = r#"
        fn id(x) { return x; }
        fn inner2(a, i, k) { return a[i][k]; }
        let v = inner2(id([[1, 2], [3, 4]]), 1, 0);
    "#;
    let vm = run_ok(src);
    assert_eq!(int_of(&vm, "v"), 3, "a[1][0] via two generic Index ops");
}

#[test]
fn index2d_fused_matrix_read() {
    let src = r#"
        fn cell(r, k) { let g = [[1, 2], [3, 4]]; return g[r][k]; }
        let v = cell(1, 0);
        let w = cell(0, 1);
    "#;
    assert!(
        count_matching(&compile(src), is_index_2d) >= 1,
        "array-proven g[r][k] must fuse to Index2D"
    );
    let vm = run_ok(src);
    assert_eq!(int_of(&vm, "v"), 3, "g[1][0]");
    assert_eq!(int_of(&vm, "w"), 2, "g[0][1]");
}

#[test]
fn index2d_row_bounds_and_type_errors() {
    let src = "fn cell(r, k) { let g = [[1, 2], [3, 4]]; return g[r][k]; }";
    assert!(count_matching(&compile(src), is_index_2d) >= 1);
    expect_error(&format!("{} let v = cell(2, 0);", src), "Index2D: row1 OOB");
    expect_error(&format!("{} let v = cell(0, 9);", src), "Index2D: row2 OOB");
    expect_error(&format!("{} let v = cell(\"x\", 0);", src), "Index2D: idx1 not numeric");
}

#[test]
fn index2d_map_row_fallback() {
    // Row is an object: the cold fallback resolves idx2 as a map key (B5).
    let src = r#"
        fn field(r, key) { let rows = [{ v: 10 }, { v: 20 }]; return rows[r][key]; }
        let x = field(1, "v");
    "#;
    assert!(count_matching(&compile(src), is_index_2d) >= 1);
    let vm = run_ok(src);
    assert_eq!(int_of(&vm, "x"), 20, "rows[1][\"v\"] via row fallback");
}

#[test]
fn index2d_string_row_fallback() {
    // Row is a string: fallback applies the Index string semantics.
    let src = r#"
        fn ch(r, k) { let pair = ["ab", "cd"]; return pair[r][k]; }
        let c = ch(0, 1);
    "#;
    assert!(count_matching(&compile(src), is_index_2d) >= 1);
    let vm = run_ok(src);
    assert_eq!(string_of(&vm, "c"), "b", "pair[0][1]");
}

#[test]
fn index2d_row_not_array_error() {
    let src = "fn bad(r, k) { let nums = [7, 8]; return nums[r][k]; } let x = bad(0, 0);";
    assert!(count_matching(&compile(src), is_index_2d) >= 1);
    expect_error(src, "Index2D: row not array");
}

// ── index assignment: IndexAssign / IndexAssignArray ───────────────────

#[test]
fn index_assign_array_writes_in_bounds() {
    let src = "let arr = [10, 20, 30]; let i = 1; arr[i] = 99; \
               let x = arr[i]; let y = arr[0];";
    assert!(
        count_matching(&compile(src), is_index_assign_array) >= 1,
        "array-typed `arr[i] = v` must lower to IndexAssignArray"
    );
    let vm = run_ok(src);
    assert_eq!(int_of(&vm, "x"), 99, "arr[1] after write");
    assert_eq!(int_of(&vm, "y"), 10, "untouched neighbor arr[0]");
}

#[test]
fn index_assign_array_grows_with_null_fill() {
    let src = "let arr = [1, 2, 3]; let i = 5; arr[i] = 9; \
               let z3 = arr[3]; let z4 = arr[4]; let v5 = arr[5]; let v0 = arr[0];";
    let vm = run_ok(src);
    assert!(var(&vm, "z3").is_null(), "grown slot arr[3] must be null");
    assert!(var(&vm, "z4").is_null(), "grown slot arr[4] must be null"); // growth fills nulls
    assert_eq!(int_of(&vm, "v5"), 9, "written slot arr[5]");
    assert_eq!(int_of(&vm, "v0"), 1, "original slot arr[0] preserved");
    // Growth stops exactly at index 5: reading index 6 is out of bounds.
    expect_error(
        "let arr = [1, 2, 3]; let i = 5; arr[i] = 9; let x = arr[6];",
        "Array index out of bounds: 6",
    );
}

#[test]
fn index_assign_array_hot_path_pins_bad_index_behavior() {
    // Pins ACTUAL hot-path behavior (suspected source bug): the
    // IndexAssignArray fast path in execute/step.rs coerces a non-numeric
    // index to 0 (`.unwrap_or(0)`) and lacks the other paths' 2^28 guard;
    // exercising that gap would allocate ~4 GiB, so it stays a recorded
    // source finding and the grow path is covered at a sane index below.
    let vm = run_ok("let arr = [1]; let k = \"x\"; arr[k] = 9; let x = arr[0];");
    assert_eq!(int_of(&vm, "x"), 9, "non-numeric index silently writes slot 0");
    let vm = run_ok("let arr = [1]; let i = 100000; arr[i] = 9; let y = arr[i]; let n = arr[500];");
    assert_eq!(var(&vm, "arr").as_array().expect("arr stays an array").len(), 100_001,
        "index assignment grows the array to index+1");
    assert_eq!(int_of(&vm, "y"), 9, "written slot holds its value after the grow");
    assert!(var(&vm, "n").is_null(), "growth slots are null-filled");
}

#[test]
fn index_assign_map_inserts_and_overwrites() {
    let src = "let m = { a: 1 }; let kb = \"b\"; let ka = \"a\"; m[kb] = 5; let vb = m[kb]; \
               m[ka] = 7; let va = m[ka];";
    assert!(
        count_matching(&compile(src), is_index_assign) >= 2,
        "object-typed `m[k] = v` must lower to generic IndexAssign"
    );
    let vm = run_ok(src);
    assert_eq!(int_of(&vm, "vb"), 5, "inserted key b");
    assert_eq!(int_of(&vm, "va"), 7, "overwritten key a");
}

#[test]
fn index_assign_non_container_rejected() {
    expect_error(
        "let n = 5; let i = 0; n[i] = 1;",
        "Cannot index-assign into non-array/object",
    );
}

// ── 2D assignment: IndexAssign2D ───────────────────────────────────────

#[test]
fn index_assign2d_writes_cell() {
    let src = r#"
        fn put(r, c, v) { let g = [[1, 2], [3, 4]]; g[r][c] = v; return g[r][c]; }
        let out = put(1, 0, 9);
        let corner = put(0, 1, 7);
    "#;
    assert!(
        count_matching(&compile(src), is_index_assign_2d) >= 1,
        "array-proven `g[r][c] = v` must fuse to IndexAssign2D"
    );
    let vm = run_ok(src);
    assert_eq!(int_of(&vm, "out"), 9, "g[1][0] after write");
    assert_eq!(int_of(&vm, "corner"), 7, "g[0][1] after write");
}

#[test]
fn index_assign2d_row_bounds_no_grow() {
    // Unlike 1D IndexAssign, the 2D form never grows the inner row.
    let src = "fn put(r, c, v) { let g = [[1, 2], [3, 4]]; g[r][c] = v; return g[r][c]; }";
    assert!(count_matching(&compile(src), is_index_assign_2d) >= 1);
    expect_error(&format!("{} let out = put(0, 9, 1);", src), "IndexAssign2D: row2 OOB");
    expect_error(&format!("{} let out = put(9, 0, 1);", src), "IndexAssign2D: row1 OOB");
}
