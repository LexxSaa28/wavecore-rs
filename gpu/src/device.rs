use crate::{GpuError, GpuResult, GpuCapabilities};
use std::sync::Arc;

/// GPU device information
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub id: usize,
    pub name: String,
    pub total_memory: u64,
    pub free_memory: u64,
    pub compute_capability: (u32, u32),
    pub max_threads_per_block: u32,
    pub max_shared_memory: u32,
    pub multiprocessor_count: u32,
    pub clock_rate: u32,
}

/// GPU device wrapper
pub struct GpuDevice {
    pub info: DeviceInfo,
    #[cfg(feature = "cuda")]
    cuda_device: Option<Arc<cudarc::driver::CudaDevice>>,
}

impl GpuDevice {
    /// Create new GPU device
    pub fn new(device_id: usize) -> GpuResult<Self> {
        #[cfg(feature = "cuda")]
        {
            match Self::init_cuda_device(device_id) {
                Ok((cuda_device, info)) => Ok(Self {
                    info,
                    cuda_device: Some(cuda_device),
                }),
                Err(e) => Err(GpuError::CudaError(format!("Failed to initialize device {}: {}", device_id, e))),
            }
        }
        
        #[cfg(not(feature = "cuda"))]
        {
            Err(GpuError::DeviceNotAvailable)
        }
    }

    /// Get default GPU device
    pub fn default() -> GpuResult<Self> {
        Self::new(0)
    }

    /// List available GPU devices
    pub fn list_devices() -> GpuResult<Vec<DeviceInfo>> {
        #[cfg(feature = "cuda")]
        {
            Self::list_cuda_devices()
        }
        
        #[cfg(not(feature = "cuda"))]
        {
            Ok(Vec::new())
        }
    }

    /// Check if device is available
    pub fn is_available(&self) -> bool {
        #[cfg(feature = "cuda")]
        {
            self.cuda_device.is_some()
        }
        
        #[cfg(not(feature = "cuda"))]
        {
            false
        }
    }

    /// Get device memory usage
    pub fn memory_usage(&self) -> GpuResult<(u64, u64)> {
        #[cfg(feature = "cuda")]
        {
            if let Some(ref device) = self.cuda_device {
                match device.memory_info() {
                    Ok((free, total)) => Ok((free, total)),
                    Err(e) => Err(GpuError::CudaError(format!("Failed to get memory info: {}", e))),
                }
            } else {
                Err(GpuError::DeviceNotAvailable)
            }
        }
        
        #[cfg(not(feature = "cuda"))]
        {
            Err(GpuError::DeviceNotAvailable)
        }
    }

    /// Synchronize device
    pub fn synchronize(&self) -> GpuResult<()> {
        #[cfg(feature = "cuda")]
        {
            if let Some(ref device) = self.cuda_device {
                device.synchronize().map_err(|e| GpuError::CudaError(format!("Synchronization failed: {}", e)))
            } else {
                Err(GpuError::DeviceNotAvailable)
            }
        }
        
        #[cfg(not(feature = "cuda"))]
        {
            Err(GpuError::DeviceNotAvailable)
        }
    }

    #[cfg(feature = "cuda")]
    pub fn cuda_device(&self) -> Option<&Arc<cudarc::driver::CudaDevice>> {
        self.cuda_device.as_ref()
    }

    #[cfg(feature = "cuda")]
    fn init_cuda_device(device_id: usize) -> Result<(Arc<cudarc::driver::CudaDevice>, DeviceInfo), Box<dyn std::error::Error>> {
        // Initialize CUDA device
        let device = cudarc::driver::CudaDevice::new(device_id)?;
        
        // Get device properties
        let name = device.name()?;
        let (free_memory, total_memory) = device.memory_info()?;
        
        // For now, use placeholder values for properties that might not be directly available
        let info = DeviceInfo {
            id: device_id,
            name,
            total_memory,
            free_memory,
            compute_capability: (7, 5), // Placeholder - would need proper query
            max_threads_per_block: 1024, // Placeholder
            max_shared_memory: 49152,    // Placeholder
            multiprocessor_count: 108,   // Placeholder
            clock_rate: 1500000,         // Placeholder
        };
        
        Ok((Arc::new(device), info))
    }

    #[cfg(feature = "cuda")]
    fn list_cuda_devices() -> GpuResult<Vec<DeviceInfo>> {
        let mut devices = Vec::new();
        
        // Try to get device count
        match cudarc::driver::result::device::get_count() {
            Ok(count) => {
                for i in 0..count {
                    match Self::init_cuda_device(i) {
                        Ok((_, info)) => devices.push(info),
                        Err(_) => continue, // Skip unavailable devices
                    }
                }
            },
            Err(_) => return Ok(Vec::new()), // No CUDA devices available
        }
        
        Ok(devices)
    }
}

/// Check CUDA availability and capabilities
#[cfg(feature = "cuda")]
pub fn check_cuda_availability() -> GpuResult<GpuCapabilities> {
    match cudarc::driver::result::device::get_count() {
        Ok(device_count) => {
            if device_count > 0 {
                // Get capabilities from first device
                match GpuDevice::new(0) {
                    Ok(device) => Ok(GpuCapabilities {
                        cuda_available: true,
                        device_count: device_count as usize,
                        total_memory: device.info.total_memory,
                        compute_capability: device.info.compute_capability,
                        max_threads_per_block: device.info.max_threads_per_block,
                        max_shared_memory: device.info.max_shared_memory,
                    }),
                    Err(_) => Ok(GpuCapabilities {
                        cuda_available: false,
                        device_count: 0,
                        total_memory: 0,
                        compute_capability: (0, 0),
                        max_threads_per_block: 0,
                        max_shared_memory: 0,
                    }),
                }
            } else {
                Ok(GpuCapabilities {
                    cuda_available: false,
                    device_count: 0,
                    total_memory: 0,
                    compute_capability: (0, 0),
                    max_threads_per_block: 0,
                    max_shared_memory: 0,
                })
            }
        },
        Err(_) => Ok(GpuCapabilities {
            cuda_available: false,
            device_count: 0,
            total_memory: 0,
            compute_capability: (0, 0),
            max_threads_per_block: 0,
            max_shared_memory: 0,
        }),
    }
}

#[cfg(not(feature = "cuda"))]
pub fn check_cuda_availability() -> GpuResult<GpuCapabilities> {
    Ok(GpuCapabilities {
        cuda_available: false,
        device_count: 0,
        total_memory: 0,
        compute_capability: (0, 0),
        max_threads_per_block: 0,
        max_shared_memory: 0,
    })
}

impl std::fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GPU Device {}: {} (Compute {}.{}, {} MB memory)", 
               self.id, self.name, self.compute_capability.0, self.compute_capability.1,
               self.total_memory / 1024 / 1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_info_display() {
        let info = DeviceInfo {
            id: 0,
            name: "Test GPU".to_string(),
            total_memory: 8589934592, // 8 GB
            free_memory: 4294967296,  // 4 GB
            compute_capability: (7, 5),
            max_threads_per_block: 1024,
            max_shared_memory: 49152,
            multiprocessor_count: 108,
            clock_rate: 1500000,
        };
        
        let display = format!("{}", info);
        assert!(display.contains("Test GPU"));
        assert!(display.contains("8192 MB"));
    }

    #[test]
    fn test_list_devices() {
        let devices = GpuDevice::list_devices();
        assert!(devices.is_ok());
        // Should not panic even if no GPU is available
    }

    #[test]
    fn test_cuda_availability_check() {
        let result = check_cuda_availability();
        assert!(result.is_ok());
        // Should return valid capabilities even if CUDA is not available
    }

    #[test]
    fn test_device_creation_fallback() {
        // This test should handle the case where no GPU is available gracefully
        match GpuDevice::new(0) {
            Ok(_) => {
                // GPU is available, test passed
            },
            Err(GpuError::DeviceNotAvailable) | Err(GpuError::CudaError(_)) => {
                // Expected when no GPU is available
            },
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }
} 