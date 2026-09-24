//! End-to-end FFI bridge tests driven through the C ABI exactly like the
//! Dart side will drive it.

use std::ffi::{CStr, CString};
use std::os::raw::c_void;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;

use hudhud_ffi::callback::HudHostFn;
use hudhud_ffi::value_dto::{
    value16_to_dto, HudValue, HUD_TAG_ERROR, HUD_TAG_FLOAT, HUD_TAG_INT, HUD_TAG_LIST,
    HUD_TAG_MAP, HUD_TAG_NULL, HUD_TAG_PROMISE, HUD_TAG_STRING,
};
use hudhudscript_bytecode::Value16;

fn cstr(s: impl AsRef<str>) -> CString {
    CString::new(s.as_ref()).unwrap()
}

fn int_arg(v: i64) -> HudValue {
    let mut d = value16_to_dto(&Value16::int(v), 0);
    d.tag = HUD_TAG_INT;
    d.int_val = v;
    d
}

unsafe fn last_error(vm: *mut hudhud_ffi::HudVM) -> String {
    let ptr = hudhud_ffi::hud_vm_last_error(vm);
    if ptr.is_null() {
        String::new()
    } else {
        CStr::from_ptr(ptr).to_string_lossy().into_owned()
    }
}

#[test]
fn dto_roundtrip_scalars_and_nested() {
    let value = Value16::object(vec![
        ("n".to_string(), Value16::null()),
        ("b".to_string(), Value16::bool_(true)),
        ("i".to_string(), Value16::int(-42)),
        ("f".to_string(), Value16::number(3.5)),
        ("s".to_string(), Value16::string("merhaba")),
        (
            "list".to_string(),
            Value16::array(vec![Value16::int(1), Value16::string("iki")]),
        ),
    ]);
    let mut dto = value16_to_dto(&value, 0);
    assert_eq!(dto.tag, HUD_TAG_MAP);
    assert_eq!(dto.len, 6);
    // Nested list survives with its own children.
    unsafe {
        let entries = std::slice::from_raw_parts(dto.entries, dto.len as usize);
        let list = entries
            .iter()
            .find(|e| CStr::from_ptr(e.key).to_bytes() == b"list")
            .unwrap();
        assert_eq!((*list.value).tag, HUD_TAG_LIST);
        assert_eq!((*list.value).len, 2);
        let items = std::slice::from_raw_parts((*list.value).items, 2);
        assert_eq!(items[0].tag, HUD_TAG_INT);
        assert_eq!(items[0].int_val, 1);
        assert_eq!(items[1].tag, HUD_TAG_STRING);
    }
    let back = hudhud_ffi::value_dto::dto_to_value16(&dto, 0).unwrap();
    let obj = back.as_object().unwrap();
    assert_eq!(obj.get("i").unwrap().as_int(), Some(-42));
    assert_eq!(obj.get("s").unwrap().as_str(), Some("merhaba"));
    unsafe { hudhud_ffi::value_dto::free_dto_contents(&mut dto) };

    let null = value16_to_dto(&Value16::null(), 0);
    assert_eq!(null.tag, HUD_TAG_NULL);
    let float = value16_to_dto(&Value16::number(2.25), 0);
    assert_eq!(float.tag, HUD_TAG_FLOAT);
    assert_eq!(float.float_val, 2.25);
    let rejected = value16_to_dto(
        &Value16::promise(hudhudscript_bytecode::PromiseState16::Rejected(
            "boom".into(),
        )),
        0,
    );
    assert_eq!(rejected.tag, HUD_TAG_ERROR);
}

#[test]
fn execute_source_call_function_and_bytecode() {
    unsafe {
        let vm = hudhud_ffi::hud_vm_new();
        let source = cstr("function add(a, b) { return a + b }\nlet sum = add(10, 32)");
        assert_eq!(hudhud_ffi::hud_vm_execute_source(vm, source.as_ptr()), 0);

        let args = [int_arg(2), int_arg(3)];
        let out = hudhud_ffi::hud_vm_call(vm, cstr("add").as_ptr(), args.as_ptr(), 2);
        assert!(!out.is_null(), "call failed: {}", last_error(vm));
        assert_eq!((*out).tag, HUD_TAG_INT);
        assert_eq!((*out).int_val, 5);
        hudhud_ffi::value_dto::hud_value_free(out);

        // Globals round trip with a non-scalar.
        let gv = hudhud_ffi::hud_vm_get_global(vm, cstr("sum").as_ptr());
        assert!(!gv.is_null());
        assert_eq!((*gv).int_val, 42);
        hudhud_ffi::value_dto::hud_value_free(gv);

        // Bytecode path: compile → serialize → fresh VM executes it.
        let compiled = hudhud_ffi::call::hud_vm_compile(vm, cstr("function mul(a) { return a * 3 }").as_ptr());
        assert!(!compiled.is_null());
        assert_eq!(hudhud_ffi::call::hud_vm_execute_compiled(vm, compiled), 0);
        hudhud_ffi::call::hud_bytecode_free(compiled);
        let args2 = [int_arg(7)];
        let out2 = hudhud_ffi::hud_vm_call(vm, cstr("mul").as_ptr(), args2.as_ptr(), 1);
        assert!(!out2.is_null());
        assert_eq!((*out2).int_val, 21);
        hudhud_ffi::value_dto::hud_value_free(out2);

        hudhud_ffi::hud_vm_free(vm);
    }
}

#[test]
fn call_await_resolves_async_function() {
    unsafe {
        let vm = hudhud_ffi::hud_vm_new();
        let source = cstr("async function work(v) { sleep(40); return v }");
        assert_eq!(hudhud_ffi::hud_vm_execute_source(vm, source.as_ptr()), 0);
        let args = [int_arg(9)];
        let out = hudhud_ffi::call::hud_vm_call_await(
            vm,
            cstr("work").as_ptr(),
            args.as_ptr(),
            1,
            5_000,
        );
        assert!(!out.is_null(), "await failed: {}", last_error(vm));
        assert_eq!((*out).tag, HUD_TAG_INT);
        assert_eq!((*out).int_val, 9);
        hudhud_ffi::value_dto::hud_value_free(out);
        hudhoud_drop(vm);
    }
}

unsafe fn hudhoud_drop(vm: *mut hudhud_ffi::HudVM) {
    hudhud_ffi::hud_vm_free(vm);
}

#[test]
fn sync_host_trampoline_callable_from_script() {
    unsafe extern "C" fn double(
        _user_data: *mut c_void,
        args: *const HudValue,
        len: i32,
    ) -> *mut HudValue {
        let slice = std::slice::from_raw_parts(args, len as usize);
        let mut out = Box::new(value16_to_dto(&Value16::int(0), 0));
        out.tag = HUD_TAG_INT;
        out.int_val = if slice.is_empty() { 0 } else { slice[0].int_val } * 2;
        Box::into_raw(out)
    }

    unsafe {
        let vm = hudhud_ffi::hud_vm_new();
        let trampoline: HudHostFn = double;
        assert_eq!(
            hudhud_ffi::callback::hud_vm_register_host_fn(
                vm,
                cstr("mathx").as_ptr(),
                cstr("double").as_ptr(),
                trampoline,
                std::ptr::null_mut(),
            ),
            0
        );
        let source = cstr("let r = mathx.double(21)");
        assert_eq!(hudhud_ffi::hud_vm_execute_source(vm, source.as_ptr()), 0);
        let g = hudhud_ffi::hud_vm_get_global(vm, cstr("r").as_ptr());
        assert!(!g.is_null());
        assert_eq!((*g).int_val, 42);
        hudhud_ffi::value_dto::hud_value_free(g);
        hudhoud_drop(vm);
    }
}

static TEST_BRIDGE: AtomicI64 = AtomicI64::new(0);

extern "C" fn dart_style_notify(request_id: i64) {
    let bridge = TEST_BRIDGE.load(Ordering::SeqCst);
    std::thread::spawn(move || {
        // Simulate the Dart handler: fetch args, do async work, complete.
        std::thread::sleep(Duration::from_millis(30));
        unsafe {
            let info = hudhud_ffi::callback::hud_callback_take(bridge, request_id);
            assert!(!info.is_null());
            let info_ref = &*info;
            let key = CStr::from_ptr(info_ref.key).to_string_lossy();
            assert_eq!(key, "dart.triple");
            let args = std::slice::from_raw_parts(info_ref.args, info_ref.len as usize);
            let tripled = int_arg(args[0].int_val * 3);
            assert_eq!(
                hudhud_ffi::callback::hud_callback_complete(
                    bridge,
                    request_id,
                    &tripled,
                    std::ptr::null(),
                ),
                0
            );
            hudhud_ffi::callback::hud_callback_info_free(info);
        }
    });
}

#[test]
fn dart_style_async_callback_settles_await() {
    unsafe {
        let vm = hudhud_ffi::hud_vm_new();
        let bridge = hudhud_ffi::call::hud_vm_bridge_id(vm);
        TEST_BRIDGE.store(bridge, Ordering::SeqCst);
        assert_eq!(
            hudhud_ffi::callback::hud_dart_set_callback(
                bridge,
                dart_style_notify as extern "C" fn(i64) as usize,
            ),
            0
        );
        assert_eq!(
            hudhud_ffi::callback::hud_vm_register_dart_fn(
                vm,
                cstr("dart").as_ptr(),
                cstr("triple").as_ptr(),
            ),
            0
        );
        // Top-level await: execute() blocks in the promise registry until
        // the notifier thread completes the callback — the exact path the
        // Flutter bridge takes. Works on a fresh VM/thread thanks to the
        // bridge's one-time temp-register shift (see `shift_temp_regs_once`).
        let source = cstr("let x = await dart.triple(4)");
        assert_eq!(
            hudhud_ffi::hud_vm_execute_source(vm, source.as_ptr()),
            0,
            "execute failed: {}",
            last_error(vm)
        );
        let g = hudhud_ffi::hud_vm_get_global(vm, cstr("x").as_ptr());
        assert!(!g.is_null());
        assert_eq!((*g).tag, HUD_TAG_INT);
        assert_eq!((*g).int_val, 12);
        hudhud_ffi::value_dto::hud_value_free(g);
        hudhoud_drop(vm);
    }
}

#[test]
fn promise_await_timeout_preserves_id_for_retry() {
    unsafe {
        let vm = hudhud_ffi::hud_vm_new();
        let id_ptr = hudhud_ffi::async_api::hud_vm_create_promise(vm);
        assert!(!id_ptr.is_null());
        let id = CStr::from_ptr(id_ptr).to_string_lossy().into_owned();
        // The returned C string is ours; copy done, release it.
        hudhud_ffi::hud_string_free(id_ptr);

        // Timeout: id must remain awaitable afterwards.
        let t1 = hudhud_ffi::async_api::hud_vm_await_promise(vm, cstr(&id).as_ptr(), 80);
        assert!(t1.is_null(), "await should have timed out");
        assert!(last_error(vm).contains("timed out"));

        let c_id = cstr(&id);
        assert_eq!(
            hudhud_ffi::async_api::hud_vm_resolve_promise(
                hudhud_ffi::call::hud_vm_bridge_id(vm),
                c_id.as_ptr(),
                &int_arg(77),
                std::ptr::null(),
            ),
            0
        );
        let t2 = hudhud_ffi::async_api::hud_vm_await_promise(vm, c_id.as_ptr(), 5_000);
        assert!(!t2.is_null(), "retry await failed: {}", last_error(vm));
        assert_eq!((*t2).int_val, 77);
        hudhoud_drop(vm);
    }
}

#[test]
fn external_promise_injected_as_global_awaits_in_script() {
    unsafe {
        let vm = hudhud_ffi::hud_vm_new();
        let id_ptr = hudhud_ffi::async_api::hud_vm_create_promise(vm);
        assert!(!id_ptr.is_null());
        // Copy the id text BEFORE handing the pointer to the DTO (whose
        // free releases the C string).
        let id = CStr::from_ptr(id_ptr).to_string_lossy().into_owned();
        let mut promise_global = value16_to_dto(&Value16::null(), 0);
        promise_global.tag = HUD_TAG_PROMISE;
        promise_global.promise_id = id_ptr;
        assert_eq!(
            hudhud_ffi::call::hud_vm_set_global(vm, cstr("p").as_ptr(), &promise_global),
            0
        );
        hudhud_ffi::value_dto::free_dto_contents(&mut promise_global);

        // Resolve from another thread while execute() is blocked in await.
        let bridge = hudhud_ffi::call::hud_vm_bridge_id(vm);
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(60));
            let c_id = cstr(&id);
            let result = int_arg(123);
            assert_eq!(
                hudhud_ffi::async_api::hud_vm_resolve_promise(
                    bridge,
                    c_id.as_ptr(),
                    &result,
                    std::ptr::null(),
                ),
                0,
                "resolve_promise must find the minted id"
            );
        });

        let source = cstr("let x = await p");
        assert_eq!(
            hudhud_ffi::hud_vm_execute_source(vm, source.as_ptr()),
            0,
            "execute failed: {}",
            last_error(vm)
        );
        let g = hudhud_ffi::hud_vm_get_global(vm, cstr("x").as_ptr());
        assert!(!g.is_null());
        assert_eq!((*g).int_val, 123);
        hudhud_ffi::value_dto::hud_value_free(g);
        hudhoud_drop(vm);
    }
}
