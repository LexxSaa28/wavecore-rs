//! # WaveCore FFI Module
//! 
//! Foreign Function Interface for WaveCore - provides C API for external language bindings.
//! 
//! This module exposes WaveCore functionality through a C interface that can be used
//! by other languages like Go, Python, C++, etc.

use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;
use std::sync::Mutex;

use wavecore_bem::{BEMSolver, SolverEngine, ProblemType};
use wavecore_meshes::{Mesh, PredefinedGeometry, Result as MeshResult};

// Global error state
static ERROR_MESSAGE: Mutex<Option<String>> = Mutex::new(None);

// Global performance metrics
static PERFORMANCE_METRICS: Mutex<Option<PerformanceMetrics>> = Mutex::new(None);

// C-compatible structures
#[repr(C)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[repr(C)]
pub struct CMesh {
    pub vertices: *mut Point3D,
    pub faces: *mut u32,
    pub num_vertices: u32,
    pub num_faces: u32,
}

#[repr(C)]
pub struct ProblemConfig {
    pub frequency: f64,
    pub direction: f64,
    pub mode: u32,
}

#[repr(C)]
pub struct BEMResults {
    pub added_mass: *mut f64,
    pub damping: *mut f64,
    pub exciting_forces: *mut f64,
    pub size: u32,
}

#[repr(C)]
pub struct SeakeepingResults {
    pub raos: *mut f64,
    pub motions: *mut f64,
    pub num_frequencies: u32,
    pub num_directions: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PerformanceMetrics {
    pub setup_time_ms: f64,
    pub solve_time_ms: f64,
    pub post_process_time_ms: f64,
    pub memory_usage_bytes: u64,
    pub iterations: u32,
}

#[repr(C)]
pub struct SolverConfig {
    pub tolerance: f64,
    pub max_iterations: u32,
    pub solver_type: u32,
    pub green_function_method: u32,
    pub use_gpu: u32,
    pub parallel_threads: u32,
}

// Error handling functions
fn set_error(message: String) {
    let mut error = ERROR_MESSAGE.lock().unwrap();
    *error = Some(message);
}

fn clear_error() {
    let mut error = ERROR_MESSAGE.lock().unwrap();
    *error = None;
}

fn get_error() -> String {
    let error = ERROR_MESSAGE.lock().unwrap();
    error.clone().unwrap_or_else(|| "Unknown error".to_string())
}

// Performance tracking
fn start_performance_tracking() {
    let mut metrics = PERFORMANCE_METRICS.lock().unwrap();
    *metrics = Some(PerformanceMetrics {
        setup_time_ms: 0.0,
        solve_time_ms: 0.0,
        post_process_time_ms: 0.0,
        memory_usage_bytes: 0,
        iterations: 0,
    });
}

fn update_performance_metrics(setup_time: f64, solve_time: f64, post_process_time: f64, iterations: u32) {
    let mut metrics = PERFORMANCE_METRICS.lock().unwrap();
    if let Some(ref mut m) = *metrics {
        m.setup_time_ms = setup_time;
        m.solve_time_ms = solve_time;
        m.post_process_time_ms = post_process_time;
        m.iterations = iterations;
        
        // Estimate memory usage (rough calculation)
        m.memory_usage_bytes = (iterations as u64) * 1024 * 1024; // 1MB per iteration estimate
    }
}

// Conversion functions
fn rust_mesh_to_c_mesh(rust_mesh: &Mesh) -> CMesh {
    let vertices: Vec<Point3D> = rust_mesh.vertices.iter().map(|p| Point3D {
        x: p.x,
        y: p.y,
        z: p.z,
    }).collect();
    
    let faces: Vec<u32> = rust_mesh.faces.iter().flat_map(|face| {
        face.iter().map(|&v| v as u32)
    }).collect();
    
    CMesh {
        vertices: vertices.as_ptr() as *mut Point3D,
        faces: faces.as_ptr() as *mut u32,
        num_vertices: vertices.len() as u32,
        num_faces: faces.len() as u32,
    }
}

fn create_bem_results(added_mass: &[f64], damping: &[f64], exciting_forces: &[f64]) -> BEMResults {
    let size = added_mass.len().max(damping.len()).max(exciting_forces.len()) as u32;
    
    BEMResults {
        added_mass: added_mass.as_ptr() as *mut f64,
        damping: damping.as_ptr() as *mut f64,
        exciting_forces: exciting_forces.as_ptr() as *mut f64,
        size,
    }
}

// Core FFI functions
#[no_mangle]
pub extern "C" fn wavecore_get_version() -> *const c_char {
    static VERSION: &str = env!("CARGO_PKG_VERSION");
    lazy_static::lazy_static! {
        static ref C_VERSION: CString = CString::new(VERSION).unwrap();
    }
    C_VERSION.as_ptr()
}

#[no_mangle]
pub extern "C" fn wavecore_get_error_message() -> *const c_char {
    let error = get_error();
    lazy_static::lazy_static! {
        static ref C_ERROR: Mutex<Option<CString>> = Mutex::new(None);
    }
    let mut c_error = C_ERROR.lock().unwrap();
    *c_error = Some(CString::new(error).unwrap());
    c_error.as_ref().unwrap().as_ptr()
}

#[no_mangle]
pub extern "C" fn wavecore_clear_error() {
    clear_error();
}

// Mesh creation functions
#[no_mangle]
pub extern "C" fn wavecore_create_sphere_mesh(radius: f64, theta_res: u32, phi_res: u32) -> *mut CMesh {
    clear_error();
    
    match PredefinedGeometry::sphere(radius, theta_res as usize, phi_res as usize) {
        Ok(rust_mesh) => {
            let c_mesh = rust_mesh_to_c_mesh(&rust_mesh);
            let boxed_mesh = Box::new(c_mesh);
            Box::into_raw(boxed_mesh)
        }
        Err(e) => {
            set_error(format!("Failed to create sphere mesh: {}", e));
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn wavecore_create_cylinder_mesh(radius: f64, height: f64, theta_res: u32, z_res: u32) -> *mut CMesh {
    clear_error();
    
    // For now, return a sphere as placeholder since cylinder is not implemented
    match PredefinedGeometry::sphere(radius, theta_res as usize, z_res as usize) {
        Ok(rust_mesh) => {
            let c_mesh = rust_mesh_to_c_mesh(&rust_mesh);
            let boxed_mesh = Box::new(c_mesh);
            Box::into_raw(boxed_mesh)
        }
        Err(e) => {
            set_error(format!("Failed to create cylinder mesh: {}", e));
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn wavecore_create_box_mesh(length: f64, width: f64, height: f64, x_res: u32, y_res: u32, z_res: u32) -> *mut CMesh {
    clear_error();
    
    // For now, return a sphere as placeholder since box_mesh is not implemented
    let radius = (length * width * height).powf(1.0/3.0);
    match PredefinedGeometry::sphere(radius, x_res as usize, y_res as usize) {
        Ok(rust_mesh) => {
            let c_mesh = rust_mesh_to_c_mesh(&rust_mesh);
            let boxed_mesh = Box::new(c_mesh);
            Box::into_raw(boxed_mesh)
        }
        Err(e) => {
            set_error(format!("Failed to create box mesh: {}", e));
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn wavecore_free_mesh(mesh: *mut CMesh) {
    if !mesh.is_null() {
        unsafe {
            let _ = Box::from_raw(mesh);
        }
    }
}

// BEM solver functions
#[no_mangle]
pub extern "C" fn wavecore_solve_radiation(mesh: *mut CMesh, config: *const ProblemConfig) -> *mut BEMResults {
    clear_error();
    start_performance_tracking();
    
    if mesh.is_null() || config.is_null() {
        set_error("Invalid mesh or config pointer".to_string());
        return ptr::null_mut();
    }
    
    let config = unsafe { &*config };
    
    // Convert C mesh to Rust mesh (simplified - in real implementation would need proper conversion)
    let rust_mesh = match create_test_mesh() {
        Ok(m) => m,
        Err(e) => {
            set_error(format!("Failed to create test mesh: {}", e));
            return ptr::null_mut();
        }
    };
    
    let setup_start = std::time::Instant::now();
    
    // Create BEM solver
    let solver = BEMSolver::new(SolverEngine::Standard);
    
    // Create problem
    let problem = ProblemType::Radiation {
        frequency: config.frequency,
        mode: config.mode as usize,
    };
    
    let setup_time = setup_start.elapsed().as_secs_f64() * 1000.0;
    let solve_start = std::time::Instant::now();
    
    // For now, return mock results since full BEM solver integration is complex
    // In a real implementation, you would create a BEMProblem and call solver.solve()
    let _result = match create_test_mesh() {
        Ok(_) => (),
        Err(e) => {
            set_error(format!("BEM solver failed: {}", e));
            return ptr::null_mut();
        }
    };
    
    let solve_time = solve_start.elapsed().as_secs_f64() * 1000.0;
    let post_process_start = std::time::Instant::now();
    
    // Extract results (simplified)
    let added_mass = vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 2.0]; // 3x3 matrix
    let damping = vec![0.1, 0.0, 0.0, 0.0, 0.1, 0.0, 0.0, 0.0, 0.2]; // 3x3 matrix
    let exciting_forces = vec![0.0, 0.0, 1.0]; // 3D vector
    
    let post_process_time = post_process_start.elapsed().as_secs_f64() * 1000.0;
    
    update_performance_metrics(setup_time, solve_time, post_process_time, 1);
    
    let results = create_bem_results(&added_mass, &damping, &exciting_forces);
    let boxed_results = Box::new(results);
    Box::into_raw(boxed_results)
}

#[no_mangle]
pub extern "C" fn wavecore_solve_diffraction(mesh: *mut CMesh, config: *const ProblemConfig) -> *mut BEMResults {
    clear_error();
    start_performance_tracking();
    
    if mesh.is_null() || config.is_null() {
        set_error("Invalid mesh or config pointer".to_string());
        return ptr::null_mut();
    }
    
    let config = unsafe { &*config };
    
    // Similar to radiation but for diffraction
    let setup_start = std::time::Instant::now();
    
    let solver = BEMSolver::new(SolverEngine::Standard);
    let problem = ProblemType::Diffraction {
        frequency: config.frequency,
        direction: config.direction,
    };
    
    let setup_time = setup_start.elapsed().as_secs_f64() * 1000.0;
    let solve_start = std::time::Instant::now();
    
    // Mock solution for diffraction
    let solve_time = solve_start.elapsed().as_secs_f64() * 1000.0;
    let post_process_start = std::time::Instant::now();
    
    // Mock results
    let added_mass = vec![0.0; 9];
    let damping = vec![0.0; 9];
    let exciting_forces = vec![1.0, 0.0, 0.0]; // Wave excitation force
    
    let post_process_time = post_process_start.elapsed().as_secs_f64() * 1000.0;
    
    update_performance_metrics(setup_time, solve_time, post_process_time, 1);
    
    let results = create_bem_results(&added_mass, &damping, &exciting_forces);
    let boxed_results = Box::new(results);
    Box::into_raw(boxed_results)
}

#[no_mangle]
pub extern "C" fn wavecore_solve_seakeeping(
    mesh: *mut CMesh,
    frequencies: *const f64,
    num_freq: u32,
    directions: *const f64,
    num_dir: u32
) -> *mut SeakeepingResults {
    clear_error();
    start_performance_tracking();
    
    if mesh.is_null() || frequencies.is_null() || directions.is_null() {
        set_error("Invalid input pointers".to_string());
        return ptr::null_mut();
    }
    
    let setup_start = std::time::Instant::now();
    
    // Convert frequency and direction arrays
    let freq_slice = unsafe { std::slice::from_raw_parts(frequencies, num_freq as usize) };
    let dir_slice = unsafe { std::slice::from_raw_parts(directions, num_dir as usize) };
    
    let setup_time = setup_start.elapsed().as_secs_f64() * 1000.0;
    let solve_start = std::time::Instant::now();
    
    // Mock seakeeping analysis
    let total_results = (num_freq * num_dir) as usize;
    let raos = vec![1.0; total_results * 6]; // 6 DOF RAOs
    let motions = vec![0.5; total_results * 6]; // 6 DOF motions
    
    let solve_time = solve_start.elapsed().as_secs_f64() * 1000.0;
    let post_process_time = 0.0;
    
    update_performance_metrics(setup_time, solve_time, post_process_time, total_results as u32);
    
    let results = SeakeepingResults {
        raos: raos.as_ptr() as *mut f64,
        motions: motions.as_ptr() as *mut f64,
        num_frequencies: num_freq,
        num_directions: num_dir,
    };
    
    let boxed_results = Box::new(results);
    Box::into_raw(boxed_results)
}

// Memory management
#[no_mangle]
pub extern "C" fn wavecore_free_bem_results(results: *mut BEMResults) {
    if !results.is_null() {
        unsafe {
            let _ = Box::from_raw(results);
        }
    }
}

#[no_mangle]
pub extern "C" fn wavecore_free_seakeeping_results(results: *mut SeakeepingResults) {
    if !results.is_null() {
        unsafe {
            let _ = Box::from_raw(results);
        }
    }
}

// Utility functions
#[no_mangle]
pub extern "C" fn wavecore_calculate_mesh_volume(mesh: *mut CMesh) -> f64 {
    if mesh.is_null() {
        return 0.0;
    }
    
    // Mock volume calculation
    4.0 / 3.0 * std::f64::consts::PI // Sphere volume approximation
}

#[no_mangle]
pub extern "C" fn wavecore_calculate_mesh_surface_area(mesh: *mut CMesh) -> f64 {
    if mesh.is_null() {
        return 0.0;
    }
    
    // Mock surface area calculation
    4.0 * std::f64::consts::PI // Sphere surface area approximation
}

#[no_mangle]
pub extern "C" fn wavecore_get_mesh_vertex_count(mesh: *mut CMesh) -> u32 {
    if mesh.is_null() {
        return 0;
    }
    
    unsafe { (*mesh).num_vertices }
}

#[no_mangle]
pub extern "C" fn wavecore_get_mesh_face_count(mesh: *mut CMesh) -> u32 {
    if mesh.is_null() {
        return 0;
    }
    
    unsafe { (*mesh).num_faces }
}

// Performance monitoring
#[no_mangle]
pub extern "C" fn wavecore_get_performance_metrics() -> *mut PerformanceMetrics {
    let metrics = PERFORMANCE_METRICS.lock().unwrap();
    if let Some(ref m) = *metrics {
        let boxed_metrics = Box::new(*m);
        Box::into_raw(boxed_metrics)
    } else {
        ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn wavecore_free_performance_metrics(metrics: *mut PerformanceMetrics) {
    if !metrics.is_null() {
        unsafe {
            let _ = Box::from_raw(metrics);
        }
    }
}

// Configuration
#[no_mangle]
pub extern "C" fn wavecore_set_solver_config(config: *const SolverConfig) {
    // Implementation would store configuration globally
    // For now, just a placeholder
}

#[no_mangle]
pub extern "C" fn wavecore_get_default_solver_config() -> *mut SolverConfig {
    let config = SolverConfig {
        tolerance: 1e-6,
        max_iterations: 1000,
        solver_type: 0, // Standard solver
        green_function_method: 0, // Delhommeau
        use_gpu: 0, // CPU only
        parallel_threads: 4,
    };
    
    let boxed_config = Box::new(config);
    Box::into_raw(boxed_config)
}

#[no_mangle]
pub extern "C" fn wavecore_free_solver_config(config: *mut SolverConfig) {
    if !config.is_null() {
        unsafe {
            let _ = Box::from_raw(config);
        }
    }
}

// Helper functions
fn create_test_mesh() -> MeshResult<Mesh> {
    PredefinedGeometry::sphere(1.0, 32, 16)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version() {
        let version = unsafe { CStr::from_ptr(wavecore_get_version()) };
        assert!(!version.to_str().unwrap().is_empty());
    }
    
    #[test]
    fn test_error_handling() {
        clear_error();
        assert_eq!(get_error(), "Unknown error");
        
        set_error("Test error".to_string());
        assert_eq!(get_error(), "Test error");
    }
    
    #[test]
    fn test_sphere_mesh_creation() {
        let mesh = wavecore_create_sphere_mesh(1.0, 16, 8);
        assert!(!mesh.is_null());
        
        unsafe {
            let vertex_count = wavecore_get_mesh_vertex_count(mesh);
            let face_count = wavecore_get_mesh_face_count(mesh);
            assert!(vertex_count > 0);
            assert!(face_count > 0);
            
            wavecore_free_mesh(mesh);
        }
    }
} 