#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

struct Point3D {
  double x;
  double y;
  double z;
};

struct CMesh {
  Point3D *vertices;
  uint32_t *faces;
  uint32_t num_vertices;
  uint32_t num_faces;
};

struct BEMResults {
  double *added_mass;
  double *damping;
  double *exciting_forces;
  uint32_t size;
};

struct ProblemConfig {
  double frequency;
  double direction;
  uint32_t mode;
};

struct SeakeepingResults {
  double *raos;
  double *motions;
  uint32_t num_frequencies;
  uint32_t num_directions;
};

struct PerformanceMetrics {
  double setup_time_ms;
  double solve_time_ms;
  double post_process_time_ms;
  uint64_t memory_usage_bytes;
  uint32_t iterations;
};

struct SolverConfig {
  double tolerance;
  uint32_t max_iterations;
  uint32_t solver_type;
  uint32_t green_function_method;
  uint32_t use_gpu;
  uint32_t parallel_threads;
};

extern "C" {

const char *wavecore_get_version();

const char *wavecore_get_error_message();

void wavecore_clear_error();

CMesh *wavecore_create_sphere_mesh(double radius, uint32_t theta_res, uint32_t phi_res);

CMesh *wavecore_create_cylinder_mesh(double radius,
                                     double height,
                                     uint32_t theta_res,
                                     uint32_t z_res);

CMesh *wavecore_create_box_mesh(double length,
                                double width,
                                double height,
                                uint32_t x_res,
                                uint32_t y_res,
                                uint32_t z_res);

void wavecore_free_mesh(CMesh *mesh);

BEMResults *wavecore_solve_radiation(CMesh *mesh, const ProblemConfig *config);

BEMResults *wavecore_solve_diffraction(CMesh *mesh, const ProblemConfig *config);

SeakeepingResults *wavecore_solve_seakeeping(CMesh *mesh,
                                             const double *frequencies,
                                             uint32_t num_freq,
                                             const double *directions,
                                             uint32_t num_dir);

void wavecore_free_bem_results(BEMResults *results);

void wavecore_free_seakeeping_results(SeakeepingResults *results);

double wavecore_calculate_mesh_volume(CMesh *mesh);

double wavecore_calculate_mesh_surface_area(CMesh *mesh);

uint32_t wavecore_get_mesh_vertex_count(CMesh *mesh);

uint32_t wavecore_get_mesh_face_count(CMesh *mesh);

PerformanceMetrics *wavecore_get_performance_metrics();

void wavecore_free_performance_metrics(PerformanceMetrics *metrics);

void wavecore_set_solver_config(const SolverConfig *config);

SolverConfig *wavecore_get_default_solver_config();

void wavecore_free_solver_config(SolverConfig *config);

} // extern "C"
