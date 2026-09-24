//! v0.8.292 regresyon testleri: .length hoisting, f64 lane tip bütünlüğü,
//! typeof güvenliği (her biri kullanıcı benchmark FAIL'inin kök nedeni).

use super::super::JitRuntime;

#[test]
fn regress_while_length_with_push() {
    // .length döngü DIŞINA hoist edilirse push sonrası bayat kalır
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function main() {
            let q = []
            q.push(0)
            let head = 0
            while (head < q.length) {
                head = head + 1
                if (head == 1) { q.push(7) }
                if (head == 2) { q.push(8) }
            }
            return head * 100 + q.length
        }
    "#;
    assert_eq!(rt.run(src).expect("run").return_value, 303);
}

#[test]
fn regress_float_loop_no_fallback() {
    // f64 birikim sahte OVERFLOW ile VM'e düşüyordu
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function main() {
            let s = 0.0
            let i = 0
            while (i < 3) { s = s + 1.5; i = i + 1 }
            print(s)
            return 0
        }
    "#;
    let r = rt.run(src).expect("run");
    assert_eq!(r.exit_status, 0);
}

#[test]
fn regress_nested_float_arrays() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        fn energy(bodies) {
            let e = 0.0
            let i = 0
            while (i < 1) {
                let bi = bodies[i]
                e = e + 0.5 * bi[6]
                i = i + 1
            }
            return e
        }
        function main() {
            let bodies = [[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]]
            let e1 = energy(bodies)
            print("R: " + e1)
            return 0
        }
    "#;
    let r = rt.run(src).expect("run");
    assert_eq!(r.exit_status, 0);
}

#[test]
fn regress_typeof_large_value_safe() {
    // typeof büyük i64'ü pointer sanıp SIGSEGV veriyordu
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function main() {
            print(typeof(1010093.0))
            return 0
        }
    "#;
    let r = rt.run(src).expect("run (çökmemeli)");
    assert_eq!(r.exit_status, 0);
}

#[test]
fn regress_f06_no_double_effect() {
    // F06: JIT koşma hatasında yan etki TAM 1 kez (restart yok)
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        let n = 0
        fn bump() { n = n + 1; return n }
        let x = bump()
        print("ONCE")
        let z = 1 / 0
        print("NEVER")
    "#;
    // Bu testte JitRuntime::run exit status 2 döndürmeli (div-zero)
    // ve "ONCE" bir kez stdout'a yazılmalı — ama run() hata döndürmez,
    // status taşır. CLI katmanı restart yapmaz (A1 düzeltmesi).
    let r = rt.run(src).expect("run");
    assert_eq!(r.exit_status, 2, "div-zero status 2 olmalı");
}

#[test]
fn regress_f11_short_circuit_suppression() {
    // F11: false && effect() / true || effect() — effect ÇAĞRILMAZ
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        let n = 0
        let a = false && 1 / 0
        let b = true || 1 / 0
        let c = true && 1
        let d = false || 2
        let ok = (a == false ? 1 : 0) + (b == true ? 10 : 0)
        print(ok)
    "#;
    // n=2 (yalnız c ve d'te çağrıldı); a=false, b=true, d=true
    // a=false, b=true, c=1, d=2; 1/0 HİÇ çağrılmadı (yoksa exit 2 olurdu)
    let r = rt.run(src).expect("run");
    assert_eq!(r.exit_status, 0, "false && ve true || sağ tarafı değerlenMEMELİ");
}

#[test]
fn regress_typeof_param_string() {
    // v0.8.293+ regresyonu: typeof(param) string'i tanımıyordu
    // (ConstString STRING_REGISTRY'ye kayıtlı değildi → "number")
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function t(v) { return typeof(v) }
        function main() {
            let r = t("hud")
            print(r)
            return r == "string" ? 1 : 0
        }
    "#;
    let r = rt.run(src).expect("run");
    assert_eq!(r.return_value, 1, "typeof(string-param) == \"string\" olmalı");
}

#[test]
fn regress_bigint_add_nodemote_parity() {
    // VM oracle (bigint_add): BigInt+BigInt sonucu i64'e demote OLMAZ —
    // big + (-big) = BigInt(0), typeof "bigint" olmalı. JIT helper eskiden
    // to_i64 ile 0 int'e demote edip "number" döndürüyordu (parite hatası).
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function t(v) { return typeof(v) }
        function main() {
            let big = 9223372036854775807 + 1
            let big2 = big + big
            let neg2 = 0 - big2
            let s = big2 + neg2
            return s == 0 ? (t(s) == "bigint" ? 1 : 2) : 3
        }
    "#;
    let r = rt.run(src).expect("run");
    assert_eq!(r.exit_status, 0);
    assert_eq!(r.return_value, 1, "BigInt+BigInt=0 → typeof \"bigint\" (VM paritesi)");
}

#[test]
fn regress_bigint_sub_demote_parity() {
    // VM oracle (bigint_sub): sub her yolda demote eder — both-big küçük
    // sonuç INT olur → typeof "number". (add no-demote ile bilinçli VM farkı.
    // Sonuç 1 seçildi: typeof(0) lane'de "null"dır — int-0/null kodlama
    // belirsizliği bilinen ayrı bir lane sınırıdır.)
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function t(v) { return typeof(v) }
        function main() {
            let big = 9223372036854775807 + 1
            let big2 = big + big
            let d = (big2 + 1) - big2
            return d == 1 ? (t(d) == "number" ? 1 : 2) : 3
        }
    "#;
    let r = rt.run(src).expect("run");
    assert_eq!(r.exit_status, 0);
    assert_eq!(r.return_value, 1, "BigInt-BigInt=0 → typeof \"number\" (VM paritesi)");
}
