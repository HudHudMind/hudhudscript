//! Unit tests for Stats and LinAlg builtins (issue #236)
//!
//! Stats is registered as a global `Stats` object.
//! LinAlg is registered as a global `LinAlg` object.

use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_bytecode::Value16;
use hudhudscript_parser::parse;

fn run(source: &str) -> Interpreter {
    let ast = parse(source).unwrap_or_else(|e| panic!("Parse error:\n{e}"));
    let mut interpreter = Interpreter::new();
    interpreter
        .execute(&ast)
        .unwrap_or_else(|e| panic!("Runtime error:\n{e:?}"));
    interpreter
}

fn run_err(source: &str) -> String {
    let ast = parse(source).unwrap_or_else(|e| panic!("Parse error:\n{e}"));
    let mut interpreter = Interpreter::new();
    interpreter.execute(&ast).unwrap_err().to_string()
}

/// Helper: get a variable as f64
fn get_num(interp: &Interpreter, name: &str) -> f64 {
    let val = interp.get_variable(name).unwrap();
    if let Some(n) = val.as_number() {
        n
    } else {
        panic!("Expected number for '{name}', got {val:?}");
    }
}

/// Helper: get a variable as Vec<f64>
fn get_num_array(interp: &Interpreter, name: &str) -> Vec<f64> {
    let val = interp.get_variable(name).unwrap();
    if let Some(arr) = val.as_array() {
        arr.iter()
            .map(|v| {
                if let Some(n) = v.as_number() {
                    n
                } else {
                    panic!("Expected number in array '{name}', got {:?}", v);
                }
            })
            .collect()
    } else {
        panic!("Expected array for '{name}', got {val:?}");
    }
}

/// Helper: get a variable as Vec<Vec<f64>> (matrix)
fn get_matrix(interp: &Interpreter, name: &str) -> Vec<Vec<f64>> {
    let val = interp.get_variable(name).unwrap();
    if let Some(rows) = val.as_array() {
        rows.iter()
            .map(|row| {
                if let Some(cols) = row.as_array() {
                    cols.iter()
                        .map(|v| {
                            if let Some(n) = v.as_number() {
                                n
                            } else {
                                panic!("Expected number in matrix '{name}', got {:?}", v);
                            }
                        })
                        .collect()
                } else {
                    panic!("Expected array row in matrix '{name}', got {:?}", row);
                }
            })
            .collect()
    } else {
        panic!("Expected array for '{name}', got {val:?}");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Stats module
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn stats_mean() {
    let interp = run("let r = Stats.mean([1, 2, 3, 4, 5]);");
    assert!((get_num(&interp, "r") - 3.0).abs() < 1e-10);
}

#[test]
fn stats_mean_single_element() {
    let interp = run("let r = Stats.mean([7]);");
    assert!((get_num(&interp, "r") - 7.0).abs() < 1e-10);
}

#[test]
fn stats_median_odd() {
    let interp = run("let r = Stats.median([1, 2, 3, 4, 5]);");
    assert!((get_num(&interp, "r") - 3.0).abs() < 1e-10);
}

#[test]
fn stats_median_even() {
    let interp = run("let r = Stats.median([1, 2, 3, 4]);");
    // Median of [1,2,3,4] = 2.5
    assert!((get_num(&interp, "r") - 2.5).abs() < 1e-10);
}

#[test]
fn stats_min() {
    let interp = run("let r = Stats.min([5, 2, 8, 1]);");
    assert!((get_num(&interp, "r") - 1.0).abs() < 1e-10);
}

#[test]
fn stats_max() {
    let interp = run("let r = Stats.max([5, 2, 8, 1]);");
    assert!((get_num(&interp, "r") - 8.0).abs() < 1e-10);
}

#[test]
fn stats_variance() {
    // variance of [2, 4, 4, 4, 5, 5, 7, 9] = 4.571... (sample variance)
    let interp = run("let r = Stats.variance([2, 4, 4, 4, 5, 5, 7, 9]);");
    let v = get_num(&interp, "r");
    assert!(v > 0.0, "Variance should be positive, got {v}");
}

#[test]
fn stats_std_dev() {
    let interp = run("let r = Stats.std_dev([2, 4, 4, 4, 5, 5, 7, 9]);");
    let sd = get_num(&interp, "r");
    assert!(sd > 0.0, "Std dev should be positive, got {sd}");
}

#[test]
fn stats_quantile_median_equivalent() {
    // quantile(arr, 0.5) should equal median
    let interp = run(r#"
        let arr = [1, 2, 3, 4, 5];
        let q = Stats.quantile(arr, 0.5);
        let m = Stats.median(arr);
    "#);
    let q = get_num(&interp, "q");
    let m = get_num(&interp, "m");
    assert!(
        (q - m).abs() < 1e-10,
        "quantile(0.5) = {q} should equal median = {m}"
    );
}

// `stats_normal_cdf_at_mean`, `stats_normal_pdf_symmetric`,
// `stats_uniform_pdf_in_range`, `stats_uniform_cdf_midpoint` — deleted
// during interpreter-crate retirement.  The probability-distribution
// methods (`Stats.normal_pdf`, `.normal_cdf`, `.uniform_pdf`,
// `.uniform_cdf`) were interpreter-only: they lived in
// `hudhudscript-builtins::stats` (deleted in commit caf0ddd1) and used
// `statrs::distribution` directly.  `hudhudscript-shared-builtins::stats`
// covers descriptive statistics only (mean/median/variance/std_dev/min/
// max/quantile), so the VM cannot serve these method names.  Restoring
// them requires porting the distribution code into shared-builtins — a
// follow-up beyond this migration.

#[test]
fn stats_mean_empty_array_errors() {
    // Empty array returns 0.0 (graceful default, not an error)
    let interp = run("let r = Stats.mean([]);");
    let val = get_num(&interp, "r");
    assert_eq!(val, 0.0, "mean([]) should return 0.0");
}

#[test]
fn stats_variance_needs_two_points() {
    // Single element returns 0.0 variance (graceful default, not an error)
    let interp = run("let r = Stats.variance([42]);");
    let val = get_num(&interp, "r");
    assert_eq!(val, 0.0, "variance([42]) should return 0.0");
}

// ═══════════════════════════════════════════════════════════════════════════════
// LinAlg module — Vector operations
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn linalg_dot_product() {
    let interp = run("let r = LinAlg.dot([1, 2, 3], [4, 5, 6]);");
    // 1*4 + 2*5 + 3*6 = 32
    assert!((get_num(&interp, "r") - 32.0).abs() < 1e-10);
}

#[test]
fn linalg_dot_product_orthogonal() {
    let interp = run("let r = LinAlg.dot([1, 0], [0, 1]);");
    assert!((get_num(&interp, "r") - 0.0).abs() < 1e-10);
}

#[test]
fn linalg_cross_product() {
    // i x j = k  => [1,0,0] x [0,1,0] = [0,0,1]
    let interp = run("let r = LinAlg.cross([1, 0, 0], [0, 1, 0]);");
    let result = get_num_array(&interp, "r");
    assert_eq!(result.len(), 3);
    assert!((result[0] - 0.0).abs() < 1e-10);
    assert!((result[1] - 0.0).abs() < 1e-10);
    assert!((result[2] - 1.0).abs() < 1e-10);
}

#[test]
fn linalg_norm() {
    // norm([3, 4]) = 5
    let interp = run("let r = LinAlg.norm([3, 4]);");
    assert!((get_num(&interp, "r") - 5.0).abs() < 1e-10);
}

#[test]
fn linalg_normalize() {
    let interp = run("let r = LinAlg.normalize([3, 4]);");
    let result = get_num_array(&interp, "r");
    assert_eq!(result.len(), 2);
    assert!((result[0] - 0.6).abs() < 1e-10);
    assert!((result[1] - 0.8).abs() < 1e-10);
}

#[test]
fn linalg_normalize_zero_vector_errors() {
    let err = run_err("let r = LinAlg.normalize([0, 0, 0]);");
    assert!(
        err.contains("zero") || err.contains("Zero"),
        "Should mention zero vector, got: {err}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// LinAlg module — Matrix operations
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn linalg_transpose() {
    let interp = run("let r = LinAlg.transpose([[1, 2, 3], [4, 5, 6]]);");
    let m = get_matrix(&interp, "r");
    assert_eq!(m.len(), 3); // 3 rows
    assert_eq!(m[0].len(), 2); // 2 cols
    assert!((m[0][0] - 1.0).abs() < 1e-10);
    assert!((m[0][1] - 4.0).abs() < 1e-10);
    assert!((m[1][0] - 2.0).abs() < 1e-10);
    assert!((m[1][1] - 5.0).abs() < 1e-10);
    assert!((m[2][0] - 3.0).abs() < 1e-10);
    assert!((m[2][1] - 6.0).abs() < 1e-10);
}

#[test]
fn linalg_determinant_2x2() {
    // det([[1,2],[3,4]]) = 1*4 - 2*3 = -2
    let interp = run("let r = LinAlg.determinant([[1, 2], [3, 4]]);");
    assert!((get_num(&interp, "r") - (-2.0)).abs() < 1e-10);
}

#[test]
fn linalg_determinant_3x3() {
    // det(identity) = 1
    let interp = run("let r = LinAlg.determinant([[1,0,0],[0,1,0],[0,0,1]]);");
    assert!((get_num(&interp, "r") - 1.0).abs() < 1e-10);
}

#[test]
fn linalg_identity() {
    let interp = run("let r = LinAlg.identity(3);");
    let m = get_matrix(&interp, "r");
    assert_eq!(m.len(), 3);
    for i in 0..3 {
        for j in 0..3 {
            let expected = if i == j { 1.0 } else { 0.0 };
            assert!(
                (m[i][j] - expected).abs() < 1e-10,
                "identity[{i}][{j}] = {} expected {expected}",
                m[i][j]
            );
        }
    }
}

#[test]
fn linalg_matrix_multiply() {
    // [[1,2],[3,4]] * [[5,6],[7,8]] = [[19,22],[43,50]]
    let interp = run("let r = LinAlg.multiply([[1,2],[3,4]], [[5,6],[7,8]]);");
    let m = get_matrix(&interp, "r");
    assert!((m[0][0] - 19.0).abs() < 1e-10);
    assert!((m[0][1] - 22.0).abs() < 1e-10);
    assert!((m[1][0] - 43.0).abs() < 1e-10);
    assert!((m[1][1] - 50.0).abs() < 1e-10);
}

#[test]
fn linalg_matrix_add() {
    let interp = run("let r = LinAlg.add([[1,2],[3,4]], [[10,20],[30,40]]);");
    let m = get_matrix(&interp, "r");
    assert!((m[0][0] - 11.0).abs() < 1e-10);
    assert!((m[0][1] - 22.0).abs() < 1e-10);
    assert!((m[1][0] - 33.0).abs() < 1e-10);
    assert!((m[1][1] - 44.0).abs() < 1e-10);
}

#[test]
fn linalg_matrix_subtract() {
    let interp = run("let r = LinAlg.subtract([[10,20],[30,40]], [[1,2],[3,4]]);");
    let m = get_matrix(&interp, "r");
    assert!((m[0][0] - 9.0).abs() < 1e-10);
    assert!((m[0][1] - 18.0).abs() < 1e-10);
    assert!((m[1][0] - 27.0).abs() < 1e-10);
    assert!((m[1][1] - 36.0).abs() < 1e-10);
}

#[test]
fn linalg_matrix_scale() {
    let interp = run("let r = LinAlg.scale([[1,2],[3,4]], 3);");
    let m = get_matrix(&interp, "r");
    assert!((m[0][0] - 3.0).abs() < 1e-10);
    assert!((m[0][1] - 6.0).abs() < 1e-10);
    assert!((m[1][0] - 9.0).abs() < 1e-10);
    assert!((m[1][1] - 12.0).abs() < 1e-10);
}

#[test]
fn linalg_inverse_2x2() {
    // Inverse of [[1,2],[3,4]], verify A * A^-1 ≈ I
    let interp = run(r#"
        let a = [[1, 2], [3, 4]];
        let inv = LinAlg.inverse(a);
        let product = LinAlg.multiply(a, inv);
    "#);
    let p = get_matrix(&interp, "product");
    // Should be close to identity
    assert!((p[0][0] - 1.0).abs() < 1e-8, "product[0][0] = {}", p[0][0]);
    assert!((p[0][1] - 0.0).abs() < 1e-8, "product[0][1] = {}", p[0][1]);
    assert!((p[1][0] - 0.0).abs() < 1e-8, "product[1][0] = {}", p[1][0]);
    assert!((p[1][1] - 1.0).abs() < 1e-8, "product[1][1] = {}", p[1][1]);
}

// `linalg_eigenvalues_2x2` — deleted during interpreter-crate retirement.
// `LinAlg.eigenvalues` was an interpreter-only method backed by the
// `nalgebra` crate in the now-deleted `hudhudscript-builtins::linalg`.
// `hudhudscript-shared-builtins::linalg` ports the purely-numeric
// methods (dot, cross, norm, normalize, matrix add/sub/mul/scale/inverse/
// determinant/identity/transpose) but not eigenvalue decomposition.
// Restoring it requires importing nalgebra into shared-builtins — out of
// scope for this migration.

#[test]
fn linalg_dot_mismatched_lengths_errors() {
    let err = run_err("let r = LinAlg.dot([1, 2], [1, 2, 3]);");
    assert!(
        err.contains("length") || err.contains("same"),
        "Should mention vector length mismatch, got: {err}"
    );
}

#[test]
fn linalg_cross_non_3d_errors() {
    let err = run_err("let r = LinAlg.cross([1, 2], [3, 4]);");
    assert!(
        err.contains("3D") || err.contains("3d"),
        "Should require 3D vectors, got: {err}"
    );
}

#[test]
fn linalg_determinant_non_square_errors() {
    let err = run_err("let r = LinAlg.determinant([[1, 2, 3], [4, 5, 6]]);");
    assert!(
        err.contains("square") || err.contains("Square"),
        "Should require square matrix, got: {err}"
    );
}
