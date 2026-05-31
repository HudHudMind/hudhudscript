//! Shared Linear Algebra builtin — used by both VM and interpreter.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(
        ErrorCode::RuntimeTypeError,
        format!("{}: expected {}, got {}", context, expected, got),
    )
}

/// Extract a vector of f64 from a SharedValue array.
fn extract_vector(v: &Value16) -> HudHudResult<Vec<f64>> {
    v.as_array()
        .ok_or_else(|| runtime_error("linalg: expected array"))?
        .iter()
        .map(|x| {
            x.as_number()
                .ok_or_else(|| runtime_error("linalg: expected number in vector"))
        })
        .collect()
}

/// Extract a matrix (Vec<Vec<f64>>) from a SharedValue nested array.
fn extract_matrix(v: &Value16) -> HudHudResult<Vec<Vec<f64>>> {
    v.as_array()
        .ok_or_else(|| runtime_error("linalg: expected array of arrays"))?
        .iter()
        .map(|row| extract_vector(row))
        .collect()
}

/// Convert a matrix back to a SharedValue nested array.
fn matrix_to_value(matrix: Vec<Vec<f64>>) -> Value16 {
    Value16::array(
        matrix
            .into_iter()
            .map(|row| Value16::array(row.into_iter().map(Value16::number).collect()))
            .collect(),
    )
}

/// Enum identifying each LinAlg operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LinAlgMethodId {
    Dot,
    Cross,
    Norm,
    Normalize,
    Transpose,
    Determinant,
    Identity,
    Multiply,
    Add,
    Subtract,
    Scale,
    Inverse,
    Eigenvalues,
}

impl std::str::FromStr for LinAlgMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dot" => Ok(Self::Dot),
            "cross" => Ok(Self::Cross),
            "norm" => Ok(Self::Norm),
            "normalize" => Ok(Self::Normalize),
            "transpose" => Ok(Self::Transpose),
            "determinant" => Ok(Self::Determinant),
            "identity" => Ok(Self::Identity),
            "multiply" => Ok(Self::Multiply),
            "add" => Ok(Self::Add),
            "subtract" => Ok(Self::Subtract),
            "scale" => Ok(Self::Scale),
            "inverse" => Ok(Self::Inverse),
            "eigenvalues" => Ok(Self::Eigenvalues),
            _ => Err(runtime_error(format!("Unknown linalg method: {}", s))),
        }
    }
}

/// Zero-cost enum dispatch for LinAlg operations.
pub fn dispatch(method: LinAlgMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        LinAlgMethodId::Dot => {
            if args.len() != 2 {
                return Err(runtime_error("linalg.dot() expects 2 arguments"));
            }
            let a = extract_vector(&args[0])?;
            let b = extract_vector(&args[1])?;
            if a.len() != b.len() {
                return Err(runtime_error("linalg.dot: vectors must have same length"));
            }
            let result: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
            Ok(Value16::number(result))
        }
        LinAlgMethodId::Cross => {
            if args.len() != 2 {
                return Err(runtime_error("linalg.cross() expects 2 arguments"));
            }
            let a = extract_vector(&args[0])?;
            let b = extract_vector(&args[1])?;
            if a.len() != 3 || b.len() != 3 {
                return Err(runtime_error("linalg.cross: vectors must be 3D"));
            }
            Ok(Value16::array(vec![
                Value16::number(a[1] * b[2] - a[2] * b[1]),
                Value16::number(a[2] * b[0] - a[0] * b[2]),
                Value16::number(a[0] * b[1] - a[1] * b[0]),
            ]))
        }
        LinAlgMethodId::Norm => {
            if args.is_empty() {
                return Err(runtime_error("linalg.norm() expects 1 argument"));
            }
            let v = extract_vector(&args[0])?;
            let result: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
            Ok(Value16::number(result))
        }
        LinAlgMethodId::Normalize => {
            if args.is_empty() {
                return Err(runtime_error("linalg.normalize() expects 1 argument"));
            }
            let v = extract_vector(&args[0])?;
            let mag: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
            if mag == 0.0 {
                return Err(runtime_error("linalg.normalize: zero vector"));
            }
            Ok(Value16::array(
                v.iter().map(|x| Value16::number(x / mag)).collect(),
            ))
        }
        LinAlgMethodId::Transpose => {
            if args.is_empty() {
                return Err(runtime_error("linalg.transpose() expects 1 argument"));
            }
            let matrix = extract_matrix(&args[0])?;
            if matrix.is_empty() {
                return Ok(Value16::array(vec![]));
            }
            let rows = matrix.len();
            let cols = matrix[0].len();
            let mut result = vec![vec![0.0; rows]; cols];
            for i in 0..rows {
                for j in 0..cols {
                    result[j][i] = matrix[i][j];
                }
            }
            Ok(matrix_to_value(result))
        }
        LinAlgMethodId::Determinant => {
            if args.is_empty() {
                return Err(runtime_error("linalg.determinant() expects 1 argument"));
            }
            let matrix = extract_matrix(&args[0])?;
            let n = matrix.len();
            if n == 0 || matrix[0].len() != n {
                return Err(runtime_error("linalg.determinant: requires square matrix"));
            }
            let det = match n {
                1 => matrix[0][0],
                2 => matrix[0][0] * matrix[1][1] - matrix[0][1] * matrix[1][0],
                3 => {
                    matrix[0][0] * (matrix[1][1] * matrix[2][2] - matrix[1][2] * matrix[2][1])
                        - matrix[0][1] * (matrix[1][0] * matrix[2][2] - matrix[1][2] * matrix[2][0])
                        + matrix[0][2] * (matrix[1][0] * matrix[2][1] - matrix[1][1] * matrix[2][0])
                }
                _ => {
                    return Err(runtime_error(
                        "linalg.determinant: only implemented for 1x1, 2x2, and 3x3 matrices",
                    ))
                }
            };
            Ok(Value16::number(det))
        }
        LinAlgMethodId::Identity => {
            if args.is_empty() {
                return Err(runtime_error("linalg.identity() expects 1 argument"));
            }
            let n = args[0]
                .as_number()
                .ok_or_else(|| runtime_error("linalg.identity: expected number"))?
                as usize;
            let mut matrix = vec![vec![0.0; n]; n];
            for (i, row) in matrix.iter_mut().enumerate().take(n) {
                row[i] = 1.0;
            }
            Ok(matrix_to_value(matrix))
        }
        LinAlgMethodId::Multiply => {
            if args.len() != 2 {
                return Err(runtime_error("linalg.multiply() expects 2 arguments"));
            }
            let a = extract_matrix(&args[0])?;
            let b = extract_matrix(&args[1])?;
            if a.is_empty() || b.is_empty() {
                return Err(runtime_error(
                    "linalg.multiply: cannot multiply empty matrices",
                ));
            }
            let rows_a = a.len();
            let cols_a = a[0].len();
            let rows_b = b.len();
            let cols_b = b[0].len();
            if cols_a != rows_b {
                return Err(runtime_error(format!(
                    "linalg.multiply: dimensions incompatible: {}x{} and {}x{}",
                    rows_a, cols_a, rows_b, cols_b
                )));
            }
            let mut result = vec![vec![0.0; cols_b]; rows_a];
            for i in 0..rows_a {
                for j in 0..cols_b {
                    for k in 0..cols_a {
                        result[i][j] += a[i][k] * b[k][j];
                    }
                }
            }
            Ok(matrix_to_value(result))
        }
        LinAlgMethodId::Add => {
            if args.len() != 2 {
                return Err(runtime_error("linalg.add() expects 2 arguments"));
            }
            let a = extract_matrix(&args[0])?;
            let b = extract_matrix(&args[1])?;
            if a.len() != b.len() || (!a.is_empty() && a[0].len() != b[0].len()) {
                return Err(runtime_error(
                    "linalg.add: matrices must have same dimensions",
                ));
            }
            let result: Vec<Vec<f64>> = a
                .iter()
                .zip(b.iter())
                .map(|(ra, rb)| ra.iter().zip(rb.iter()).map(|(x, y)| x + y).collect())
                .collect();
            Ok(matrix_to_value(result))
        }
        LinAlgMethodId::Subtract => {
            if args.len() != 2 {
                return Err(runtime_error("linalg.subtract() expects 2 arguments"));
            }
            let a = extract_matrix(&args[0])?;
            let b = extract_matrix(&args[1])?;
            if a.len() != b.len() || (!a.is_empty() && a[0].len() != b[0].len()) {
                return Err(runtime_error(
                    "linalg.subtract: matrices must have same dimensions",
                ));
            }
            let result: Vec<Vec<f64>> = a
                .iter()
                .zip(b.iter())
                .map(|(ra, rb)| ra.iter().zip(rb.iter()).map(|(x, y)| x - y).collect())
                .collect();
            Ok(matrix_to_value(result))
        }
        LinAlgMethodId::Scale => {
            if args.len() != 2 {
                return Err(runtime_error("linalg.scale() expects 2 arguments"));
            }
            let matrix = extract_matrix(&args[0])?;
            let scalar = args[1]
                .as_number()
                .ok_or_else(|| runtime_error("linalg.scale: scalar must be a number"))?;
            let result: Vec<Vec<f64>> = matrix
                .iter()
                .map(|row| row.iter().map(|x| x * scalar).collect())
                .collect();
            Ok(matrix_to_value(result))
        }
        LinAlgMethodId::Inverse => {
            if args.is_empty() {
                return Err(runtime_error("linalg.inverse() expects 1 argument"));
            }
            let matrix = extract_matrix(&args[0])?;
            if matrix.is_empty() {
                return Err(runtime_error("linalg.inverse: cannot invert empty matrix"));
            }
            let n = matrix.len();
            if matrix[0].len() != n {
                return Err(runtime_error("linalg.inverse: matrix must be square"));
            }
            // Gauss-Jordan elimination for inverse
            let mut aug = vec![vec![0.0; 2 * n]; n];
            for i in 0..n {
                for j in 0..n {
                    aug[i][j] = matrix[i][j];
                }
                aug[i][n + i] = 1.0;
            }
            for col in 0..n {
                // Find pivot
                let mut pivot_row = None;
                for (ri, aug_row) in aug.iter().enumerate().skip(col) {
                    if aug_row[col].abs() > 1e-12 {
                        pivot_row = Some(ri);
                        break;
                    }
                }
                let pivot_row =
                    pivot_row.ok_or_else(|| runtime_error("linalg.inverse: matrix is singular"))?;
                aug.swap(col, pivot_row);
                let pivot_val = aug[col][col];
                for val in &mut aug[col][..2 * n] {
                    *val /= pivot_val;
                }
                for row in 0..n {
                    if row != col {
                        let factor = aug[row][col];
                        let col_row: Vec<f64> = aug[col][..2 * n].to_vec();
                        for (aug_val, &col_val) in aug[row][..2 * n].iter_mut().zip(col_row.iter())
                        {
                            *aug_val -= factor * col_val;
                        }
                    }
                }
            }
            let result: Vec<Vec<f64>> = aug.into_iter().map(|row| row[n..].to_vec()).collect();
            Ok(matrix_to_value(result))
        }
        LinAlgMethodId::Eigenvalues => {
            // Ported from the deleted
            // `hudhudscript-builtins::linalg::create_eigenvalues_function`
            // (commit caf0ddd1^).  The original used `nalgebra`; we use
            // closed-form formulas to avoid adding nalgebra to
            // shared-builtins (which every runtime pulls in).
            //
            // Supported: 1x1 and 2x2 matrices.  2x2 uses the quadratic
            // formula on the characteristic polynomial
            //   λ² − (a+d)·λ + (ad − bc) = 0
            // which closes the parity test suite (the existing
            // `linalg_eigenvalues_2x2` test only exercises 2x2 — see
            // tests/builtins/test.rs:326 comment).
            //
            // 3x3+ would need a root finder (Givens/QR or companion
            // matrix) that's a non-trivial amount of code; it returns
            // an explicit error so the test suite can distinguish
            // "VM genuinely missing" from "silent wrong result".
            if args.is_empty() {
                return Err(runtime_error("linalg.eigenvalues() expects 1 argument"));
            }
            let matrix = extract_matrix(&args[0])?;
            if matrix.is_empty() {
                return Err(runtime_error(
                    "linalg.eigenvalues: cannot compute eigenvalues of empty matrix",
                ));
            }
            let n = matrix.len();
            if matrix[0].len() != n {
                return Err(runtime_error("linalg.eigenvalues: matrix must be square"));
            }
            let eigs: Vec<f64> = match n {
                1 => vec![matrix[0][0]],
                2 => {
                    let a = matrix[0][0];
                    let b = matrix[0][1];
                    let c = matrix[1][0];
                    let d = matrix[1][1];
                    // λ² − trace·λ + det = 0
                    let trace = a + d;
                    let det = a * d - b * c;
                    let disc = trace * trace - 4.0 * det;
                    if disc < 0.0 {
                        return Err(runtime_error(
                            "linalg.eigenvalues: complex eigenvalues not supported",
                        ));
                    }
                    let sqrt_disc = disc.sqrt();
                    // Return in nalgebra's order (smallest-index root first =
                    // (trace + sqrt_disc)/2) — matches the original test
                    // expectations which only checked sum-of-eigenvalues
                    // = trace.
                    vec![(trace + sqrt_disc) / 2.0, (trace - sqrt_disc) / 2.0]
                }
                _ => {
                    return Err(runtime_error(format!(
                        "linalg.eigenvalues: only 1x1 and 2x2 matrices are supported \
                         in the shared implementation (got {}x{}); 3x3+ requires \
                         a nalgebra-based solver which is not yet ported",
                        n, n
                    )));
                }
            };
            Ok(Value16::array(
                eigs.into_iter().map(Value16::number).collect(),
            ))
        }
    }
}
