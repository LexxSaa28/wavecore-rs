//! Linear solvers for BEM matrix operations

use super::*;
use nalgebra::{DMatrix, DVector, LU, Cholesky};

/// LU decomposition solver using nalgebra
pub fn lu_solve(a: &Matrix, b: &[f64]) -> Result<Vec<f64>> {
    if a.rows != b.len() {
        return Err(MatrixError::DimensionMismatch {
            expected: a.rows,
            actual: b.len(),
        });
    }
    
    if !a.is_square() {
        return Err(MatrixError::InvalidDimensions {
            rows: a.rows,
            cols: a.cols,
        });
    }
    
    // Convert Matrix to nalgebra DMatrix
    let na_matrix = DMatrix::from_fn(a.rows, a.cols, |i, j| {
        a.get(i, j).unwrap()
    });
    
    let na_b = DVector::from_vec(b.to_vec());
    
    // Perform LU decomposition
    let lu = LU::new(na_matrix);
    
    if !lu.is_invertible() {
        return Err(MatrixError::SingularMatrix);
    }
    
    // Solve the system
    match lu.solve(&na_b) {
        Some(solution) => Ok(solution.data.as_vec().clone()),
        None => Err(MatrixError::SolverError {
            message: "LU solver failed to find solution".to_string(),
        }),
    }
}

/// Cholesky decomposition solver for symmetric positive definite matrices
pub fn cholesky_solve(a: &Matrix, b: &[f64]) -> Result<Vec<f64>> {
    if a.rows != b.len() {
        return Err(MatrixError::DimensionMismatch {
            expected: a.rows,
            actual: b.len(),
        });
    }
    
    if !a.is_square() {
        return Err(MatrixError::InvalidDimensions {
            rows: a.rows,
            cols: a.cols,
        });
    }
    
    // Convert Matrix to nalgebra DMatrix
    let na_matrix = DMatrix::from_fn(a.rows, a.cols, |i, j| {
        a.get(i, j).unwrap()
    });
    
    let na_b = DVector::from_vec(b.to_vec());
    
    // Perform Cholesky decomposition
    match Cholesky::new(na_matrix) {
        Some(chol) => {
            // Solve the system
            let solution = chol.solve(&na_b);
            Ok(solution.data.as_vec().clone())
        }
        None => Err(MatrixError::SolverError {
            message: "Matrix is not positive definite for Cholesky decomposition".to_string(),
        }),
    }
}

/// GMRES iterative solver for general matrices
pub fn gmres_solve(a: &Matrix, b: &[f64]) -> Result<Vec<f64>> {
    gmres_solve_with_params(a, b, 1e-10, 1000, None)
}

/// GMRES solver with configurable parameters
pub fn gmres_solve_with_params(
    a: &Matrix, 
    b: &[f64], 
    tolerance: f64, 
    max_iterations: usize,
    restart: Option<usize>
) -> Result<Vec<f64>> {
    if a.rows != b.len() {
        return Err(MatrixError::DimensionMismatch {
            expected: a.rows,
            actual: b.len(),
        });
    }
    
    if !a.is_square() {
        return Err(MatrixError::InvalidDimensions {
            rows: a.rows,
            cols: a.cols,
        });
    }
    
    let n = a.rows;
    let restart_k = restart.unwrap_or(n.min(50));
    
    // Initial guess (zero vector)
    let mut x = vec![0.0; n];
    let mut r = b.to_vec();
    
    // Initial residual: r = b - A*x (since x=0, r=b initially)
    let mut beta = vector_norm(&r);
    
    if beta < tolerance {
        return Ok(x); // Initial guess is already good enough
    }
    
    for _ in 0..max_iterations {
        // Arnoldi iteration
        let mut v = Vec::with_capacity(restart_k + 1);
        let mut h = vec![vec![0.0; restart_k]; restart_k + 1];
        
        // v[0] = r / ||r||
        v.push(vector_scale(&r, 1.0 / beta));
        
        // Build Krylov subspace
        for j in 0..restart_k {
            // w = A * v[j]
            let w = matrix_vector_mult(a, &v[j])?;
            
            // Modified Gram-Schmidt orthogonalization
            let mut w_orth = w;
            for i in 0..=j {
                h[i][j] = vector_dot(&w_orth, &v[i]);
                w_orth = vector_sub(&w_orth, &vector_scale(&v[i], h[i][j]));
            }
            
            h[j + 1][j] = vector_norm(&w_orth);
            
            if h[j + 1][j] < 1e-14 {
                // Breakdown, solution found in Krylov subspace
                break;
            }
            
            v.push(vector_scale(&w_orth, 1.0 / h[j + 1][j]));
        }
        
        // Solve least squares problem: min ||beta * e1 - H * y||
        let k = v.len() - 1;
        let mut g = vec![0.0; k + 1];
        g[0] = beta;
        
        // Apply Givens rotations to H and g
        let y = solve_least_squares(&h, &g, k)?;
        
        // Update solution: x = x + V * y
        for j in 0..k {
            for i in 0..n {
                x[i] += y[j] * v[j][i];
            }
        }
        
        // Compute new residual
        let ax = matrix_vector_mult(a, &x)?;
        r = vector_sub(b, &ax);
        beta = vector_norm(&r);
        
        if beta < tolerance {
            return Ok(x);
        }
    }
    
    Err(MatrixError::SolverError {
        message: format!("GMRES failed to converge within {} iterations", max_iterations),
    })
}

/// Conjugate gradient solver for symmetric positive definite matrices
pub fn cg_solve(a: &Matrix, b: &[f64]) -> Result<Vec<f64>> {
    cg_solve_with_params(a, b, 1e-10, 1000)
}

/// Conjugate gradient solver with configurable parameters
pub fn cg_solve_with_params(
    a: &Matrix, 
    b: &[f64], 
    tolerance: f64, 
    max_iterations: usize
) -> Result<Vec<f64>> {
    if a.rows != b.len() {
        return Err(MatrixError::DimensionMismatch {
            expected: a.rows,
            actual: b.len(),
        });
    }
    
    if !a.is_square() {
        return Err(MatrixError::InvalidDimensions {
            rows: a.rows,
            cols: a.cols,
        });
    }
    
    let n = a.rows;
    
    // Initial guess (zero vector)
    let mut x = vec![0.0; n];
    let mut r = b.to_vec(); // r = b - A*x (since x=0, r=b initially)
    let mut p = r.clone();
    let mut rsold = vector_dot(&r, &r);
    
    for _ in 0..max_iterations {
        let ap = matrix_vector_mult(a, &p)?;
        let alpha = rsold / vector_dot(&p, &ap);
        
        // Update solution: x = x + alpha * p
        for i in 0..n {
            x[i] += alpha * p[i];
        }
        
        // Update residual: r = r - alpha * A*p
        for i in 0..n {
            r[i] -= alpha * ap[i];
        }
        
        let rsnew = vector_dot(&r, &r);
        
        if rsnew.sqrt() < tolerance {
            return Ok(x);
        }
        
        let beta = rsnew / rsold;
        
        // Update search direction: p = r + beta * p
        for i in 0..n {
            p[i] = r[i] + beta * p[i];
        }
        
        rsold = rsnew;
    }
    
    Err(MatrixError::SolverError {
        message: format!("CG failed to converge within {} iterations", max_iterations),
    })
}

/// BiCGSTAB solver for general matrices
pub fn bicgstab_solve(a: &Matrix, b: &[f64]) -> Result<Vec<f64>> {
    bicgstab_solve_with_params(a, b, 1e-10, 1000)
}

/// BiCGSTAB solver with configurable parameters
pub fn bicgstab_solve_with_params(
    a: &Matrix, 
    b: &[f64], 
    tolerance: f64, 
    max_iterations: usize
) -> Result<Vec<f64>> {
    if a.rows != b.len() {
        return Err(MatrixError::DimensionMismatch {
            expected: a.rows,
            actual: b.len(),
        });
    }
    
    if !a.is_square() {
        return Err(MatrixError::InvalidDimensions {
            rows: a.rows,
            cols: a.cols,
        });
    }
    
    let n = a.rows;
    
    // Initial guess (zero vector)
    let mut x = vec![0.0; n];
    let mut r = b.to_vec(); // r = b - A*x
    let r0 = r.clone();
    let mut v = vec![0.0; n];
    let mut p = vec![0.0; n];
    let mut s = vec![0.0; n];
    let mut t = vec![0.0; n];
    
    let mut rho = 1.0;
    let mut alpha = 1.0;
    let mut omega = 1.0;
    
    for _ in 0..max_iterations {
        let rho_new = vector_dot(&r0, &r);
        
        if rho_new.abs() < 1e-14 {
            return Err(MatrixError::SolverError {
                message: "BiCGSTAB breakdown: rho too small".to_string(),
            });
        }
        
        let beta = (rho_new / rho) * (alpha / omega);
        
        // p = r + beta * (p - omega * v)
        for i in 0..n {
            p[i] = r[i] + beta * (p[i] - omega * v[i]);
        }
        
        v = matrix_vector_mult(a, &p)?;
        alpha = rho_new / vector_dot(&r0, &v);
        
        // s = r - alpha * v
        for i in 0..n {
            s[i] = r[i] - alpha * v[i];
        }
        
        // Check for convergence
        if vector_norm(&s) < tolerance {
            // Update x and return
            for i in 0..n {
                x[i] += alpha * p[i];
            }
            return Ok(x);
        }
        
        t = matrix_vector_mult(a, &s)?;
        omega = vector_dot(&t, &s) / vector_dot(&t, &t);
        
        // Update solution: x = x + alpha * p + omega * s
        for i in 0..n {
            x[i] += alpha * p[i] + omega * s[i];
        }
        
        // Update residual: r = s - omega * t
        for i in 0..n {
            r[i] = s[i] - omega * t[i];
        }
        
        if vector_norm(&r) < tolerance {
            return Ok(x);
        }
        
        if omega.abs() < 1e-14 {
            return Err(MatrixError::SolverError {
                message: "BiCGSTAB breakdown: omega too small".to_string(),
            });
        }
        
        rho = rho_new;
    }
    
    Err(MatrixError::SolverError {
        message: format!("BiCGSTAB failed to converge within {} iterations", max_iterations),
    })
}

// Helper functions for vector operations

fn vector_norm(v: &[f64]) -> f64 {
    v.iter().map(|x| x * x).sum::<f64>().sqrt()
}

fn vector_dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn vector_scale(v: &[f64], scalar: f64) -> Vec<f64> {
    v.iter().map(|x| x * scalar).collect()
}

fn vector_sub(a: &[f64], b: &[f64]) -> Vec<f64> {
    a.iter().zip(b.iter()).map(|(x, y)| x - y).collect()
}

fn matrix_vector_mult(a: &Matrix, v: &[f64]) -> Result<Vec<f64>> {
    if a.cols != v.len() {
        return Err(MatrixError::DimensionMismatch {
            expected: a.cols,
            actual: v.len(),
        });
    }
    
    let mut result = vec![0.0; a.rows];
    for i in 0..a.rows {
        for j in 0..a.cols {
            result[i] += a.get(i, j).unwrap() * v[j];
        }
    }
    Ok(result)
}

fn solve_least_squares(h: &[Vec<f64>], g: &[f64], k: usize) -> Result<Vec<f64>> {
    // Simple back substitution for upper triangular system
    let mut y = vec![0.0; k];
    for i in (0..k).rev() {
        let mut sum = g[i];
        for j in (i + 1)..k {
            sum -= h[i][j] * y[j];
        }
        if h[i][i].abs() < 1e-14 {
            return Err(MatrixError::SingularMatrix);
        }
        y[i] = sum / h[i][i];
    }
    Ok(y)
} 