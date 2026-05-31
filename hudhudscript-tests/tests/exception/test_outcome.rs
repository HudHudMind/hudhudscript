//! Tests for Outcome<T> enum.


use hudhudscript_exception::{Exception, ExceptionCode, Outcome};

fn make_exc(msg: &str) -> Exception {
    Exception::new(ExceptionCode(82), msg)
}

#[test]
fn outcome_exact_holds_value() {
    let o: Outcome<i32> = Outcome::Exact(42);
    assert!(o.is_exact());
    assert!(!o.is_degraded());
    assert!(o.exception().is_none());
    assert_eq!(o.value(), 42);
}

#[test]
fn outcome_degraded_holds_value_and_exception() {
    let exc = make_exc("partial result");
    let exc_clone = exc.clone();
    let o = Outcome::Degraded(99, exc);
    assert!(!o.is_exact());
    assert!(o.is_degraded());
    assert_eq!(o.exception(), Some(&exc_clone));
    assert_eq!(o.value(), 99);
}

#[test]
fn outcome_exact_clone_still_works() {
    let o = Outcome::Exact(String::from("hello"));
    let cloned = o.clone();
    assert_eq!(cloned.value(), "hello");
}

#[test]
fn outcome_degraded_value_preserves_ownership() {
    let exc = make_exc("degraded");
    let o = Outcome::Degraded(vec![1, 2, 3], exc);
    assert_eq!(o.value(), vec![1, 2, 3]);
}

#[test]
fn outcome_is_exact_degraded() {
    let exact: Outcome<i32> = Outcome::Exact(1);
    assert!(exact.is_exact());
    assert!(!exact.is_degraded());

    let degraded = Outcome::Degraded(2, make_exc("err"));
    assert!(!degraded.is_exact());
    assert!(degraded.is_degraded());
}
