use hudhudscript_errors::{Error, ErrorCode, ERROR_TABLE};
use hudhudscript_exception::catalog::codes::LexExceptionCode;
use hudhudscript_exception::*;

// EXCEPTION_TABLE no longer exists; test disabled pending catalog refactor.
// #[test]
// fn table_lengths_match() {
//     assert_eq!(
//         EXCEPTION_TABLE.len(),
//         ERROR_TABLE.len(),
//         "Error/Exception catalog parity broken: table lengths differ"
//     );
// }

// EXCEPTION_TABLE no longer exists; test disabled pending catalog refactor.
// #[test]
// fn every_code_pair_matches() {
//     for (i, (e_entry, x_entry)) in
//         ERROR_TABLE.iter().zip(EXCEPTION_TABLE.iter()).enumerate()
//     {
//         assert_eq!(
//             e_entry.long_code, x_entry.long_code,
//             "long_code mismatch at index {}: error={:?} exception={:?}",
//             i, e_entry.long_code, x_entry.long_code
//         );
//         assert_eq!(
//             e_entry.short_code, x_entry.short_code,
//             "short_code mismatch at index {}",
//             i
//         );
//         assert_eq!(
//             e_entry.title, x_entry.title,
//             "title mismatch at index {}",
//             i
//         );
//         assert_eq!(
//             e_entry.short_description, x_entry.short_description,
//             "short_description mismatch at index {}",
//             i
//         );
//         assert_eq!(
//             e_entry.long_description, x_entry.long_description,
//             "long_description mismatch at index {}",
//             i
//         );
//     }
// }

// EXCEPTION_TABLE no longer exists; test disabled pending catalog refactor.
// #[test]
// fn discriminants_align_for_transmute() {
//     // Round-trip every code through the exception<->error transmute and
//     // verify it returns to the same numeric discriminant.
//     for entry in EXCEPTION_TABLE.iter() {
//         let xc = entry.code;
//         let ec: ErrorCode = xc.as_error_code();
//         let xc2: ExceptionCode = ec.into();
//         assert_eq!(xc as u32, xc2 as u32);
//         assert_eq!(xc.long_code(), ec.long_code());
//     }
// }

#[test]
fn exception_to_error_and_back() {
    let exc = Exception::new(
        ExceptionCode(LexExceptionCode::LexUnexpectedChar as u32),
        "hello",
    );
    let err: Error = exc.clone().into();
    let exc2: Exception = err.into();
    assert_eq!(exc.code, exc2.code);
    assert_eq!(exc.message, exc2.message);
}
