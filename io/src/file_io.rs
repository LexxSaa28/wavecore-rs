//! File I/O operations

use super::*;
use std::path::Path;
use std::fs;
use std::time::Instant;

/// File I/O operations
pub struct FileIO;

impl FileIO {
    /// Load mesh from file
    pub fn load_mesh(path: &str, format: Format) -> Result<wavecore_meshes::Mesh> {
        let start_time = Instant::now();
        
        // Validate file exists
        if !Path::new(path).exists() {
            return Err(IOError::FileNotFound {
                path: path.to_string(),
            });
        }
        
        // Read file content
        let content = fs::read_to_string(path)
            .map_err(|e| IOError::MemoryMapError(e))?;
        
        // Parse based on format
        let mesh = match format {
            Format::STL => Self::parse_stl(&content)?,
            Format::OBJ => Self::parse_obj(&content)?,
            _ => {
                return Err(IOError::InvalidFormat {
                    format: format!("{:?}", format),
                });
            }
        };
        
        let duration = start_time.elapsed().as_secs_f64();
        println!("Loaded mesh from {} in {:.3}s", path, duration);
        
        Ok(mesh)
    }
    
    /// Save mesh to file
    pub fn save_mesh(mesh: &wavecore_meshes::Mesh, path: &str, format: Format) -> Result<()> {
        let start_time = Instant::now();
        
        let content = match format {
            Format::STL => Self::serialize_stl(mesh)?,
            Format::OBJ => Self::serialize_obj(mesh)?,
            _ => {
                return Err(IOError::InvalidFormat {
                    format: format!("{:?}", format),
                });
            }
        };
        
        fs::write(path, content)
            .map_err(|e| IOError::WriteError {
                message: format!("Failed to write file: {}", e),
            })?;
        
        let duration = start_time.elapsed().as_secs_f64();
        println!("Saved mesh to {} in {:.3}s", path, duration);
        
        Ok(())
    }
    
    /// Save data to file
    pub fn save_data(data: &DataArray, path: &str, format: Format) -> Result<()> {
        let start_time = Instant::now();
        
        let content = match format {
            Format::JSON => serde_json::to_string_pretty(data)
                .map_err(|e| IOError::SerializationError(e))?,
            Format::YAML => serde_yaml::to_string(data)
                .map_err(|e| IOError::YamlError(e))?,
            Format::CSV => Self::serialize_csv(data)?,
            _ => {
                return Err(IOError::InvalidFormat {
                    format: format!("{:?}", format),
                });
            }
        };
        
        fs::write(path, content)
            .map_err(|e| IOError::WriteError {
                message: format!("Failed to write file: {}", e),
            })?;
        
        let duration = start_time.elapsed().as_secs_f64();
        println!("Saved data to {} in {:.3}s", path, duration);
        
        Ok(())
    }
    
    /// Load data from file
    pub fn load_data(path: &str, format: Format) -> Result<DataArray> {
        let start_time = Instant::now();
        
        // Validate file exists
        if !Path::new(path).exists() {
            return Err(IOError::FileNotFound {
                path: path.to_string(),
            });
        }
        
        let content = fs::read_to_string(path)
            .map_err(|e| IOError::MemoryMapError(e))?;
        
        let data = match format {
            Format::JSON => serde_json::from_str(&content)
                .map_err(|e| IOError::SerializationError(e))?,
            Format::YAML => serde_yaml::from_str(&content)
                .map_err(|e| IOError::YamlError(e))?,
            Format::CSV => Self::parse_csv(&content)?,
            _ => {
                return Err(IOError::InvalidFormat {
                    format: format!("{:?}", format),
                });
            }
        };
        
        let duration = start_time.elapsed().as_secs_f64();
        println!("Loaded data from {} in {:.3}s", path, duration);
        
        Ok(data)
    }
    
    /// Get file metadata
    pub fn get_metadata(path: &str) -> Result<FileMetadata> {
        let path_obj = Path::new(path);
        
        if !path_obj.exists() {
            return Err(IOError::FileNotFound {
                path: path.to_string(),
            });
        }
        
        let metadata = fs::metadata(path)
            .map_err(|e| IOError::MemoryMapError(e))?;
        
        let format = Self::detect_format(path)?;
        
        Ok(FileMetadata {
            format,
            size: metadata.len(),
            created: metadata.created()
                .ok()
                .map(|t| t.duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    .to_string()),
            modified: metadata.modified()
                .ok()
                .map(|t| t.duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    .to_string()),
            attributes: std::collections::HashMap::new(),
        })
    }
    
    /// Detect file format from extension
    fn detect_format(path: &str) -> Result<Format> {
        let extension = Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        match extension.as_str() {
            "stl" => Ok(Format::STL),
            "obj" => Ok(Format::OBJ),
            "json" => Ok(Format::JSON),
            "yaml" | "yml" => Ok(Format::YAML),
            "csv" => Ok(Format::CSV),
            "bin" => Ok(Format::Binary),
            "nc" => Ok(Format::NetCDF),
            _ => Err(IOError::InvalidFormat {
                format: extension,
            }),
        }
    }
    
    /// Parse STL file
    fn parse_stl(content: &str) -> Result<wavecore_meshes::Mesh> {
        let mut vertices = Vec::new();
        let mut faces = Vec::new();
        let mut normals = Vec::new();
        
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        
        while i < lines.len() {
            let line = lines[i].trim();
            
            if line.starts_with("facet normal") {
                // Parse normal
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    let nx: f64 = parts[2].parse().unwrap_or(0.0);
                    let ny: f64 = parts[3].parse().unwrap_or(0.0);
                    let nz: f64 = parts[4].parse().unwrap_or(0.0);
                    normals.push(nalgebra::Vector3::new(nx, ny, nz));
                }
                
                i += 1;
                
                // Skip "outer loop" line
                while i < lines.len() && !lines[i].trim().starts_with("vertex") {
                    i += 1;
                }
                
                // Parse vertices
                let mut face_vertices = Vec::new();
                while i < lines.len() && lines[i].trim().starts_with("vertex") {
                    let line = lines[i].trim();
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        let x: f64 = parts[1].parse().unwrap_or(0.0);
                        let y: f64 = parts[2].parse().unwrap_or(0.0);
                        let z: f64 = parts[3].parse().unwrap_or(0.0);
                        face_vertices.push(vertices.len());
                        vertices.push(nalgebra::Point3::new(x, y, z));
                    }
                    i += 1;
                }
                
                // Create face if we have 3 vertices
                if face_vertices.len() >= 3 {
                    faces.push([face_vertices[0], face_vertices[1], face_vertices[2]]);
                }
                
                // Skip endloop and endfacet
                while i < lines.len() && !lines[i].trim().starts_with("facet") && !lines[i].trim().starts_with("endsolid") {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
        
        wavecore_meshes::Mesh::new(vertices, faces)
            .map_err(|e| IOError::ParseError {
                message: format!("Failed to create mesh: {}", e),
            })
    }
    
    /// Parse OBJ file
    fn parse_obj(content: &str) -> Result<wavecore_meshes::Mesh> {
        let mut vertices = Vec::new();
        let mut faces = Vec::new();
        let mut normals = Vec::new();
        
        for line in content.lines() {
            let line = line.trim();
            let parts: Vec<&str> = line.split_whitespace().collect();
            
            if parts.is_empty() || line.starts_with('#') {
                continue;
            }
            
            match parts[0] {
                "v" => {
                    if parts.len() >= 4 {
                        let x: f64 = parts[1].parse().unwrap_or(0.0);
                        let y: f64 = parts[2].parse().unwrap_or(0.0);
                        let z: f64 = parts[3].parse().unwrap_or(0.0);
                        vertices.push(nalgebra::Point3::new(x, y, z));
                    }
                }
                "vn" => {
                    if parts.len() >= 4 {
                        let nx: f64 = parts[1].parse().unwrap_or(0.0);
                        let ny: f64 = parts[2].parse().unwrap_or(0.0);
                        let nz: f64 = parts[3].parse().unwrap_or(0.0);
                        normals.push(nalgebra::Vector3::new(nx, ny, nz));
                    }
                }
                "f" => {
                    if parts.len() >= 4 {
                        let mut face = Vec::new();
                        for i in 1..parts.len() {
                            let vertex_part = parts[i].split('/').next().unwrap_or("0");
                            let vertex_idx: usize = vertex_part.parse().unwrap_or(0);
                            if vertex_idx > 0 {
                                face.push(vertex_idx - 1); // OBJ indices are 1-based
                            }
                        }
                        if face.len() >= 3 {
                            faces.push([face[0], face[1], face[2]]);
                        }
                    }
                }
                _ => {}
            }
        }
        
        wavecore_meshes::Mesh::new(vertices, faces)
            .map_err(|e| IOError::ParseError {
                message: format!("Failed to create mesh: {}", e),
            })
    }
    
    /// Serialize mesh to STL
    fn serialize_stl(mesh: &wavecore_meshes::Mesh) -> Result<String> {
        let mut content = String::new();
        content.push_str("solid mesh\n");
        
        for (i, face) in mesh.faces.iter().enumerate() {
            if face.len() >= 3 {
                let v1 = mesh.vertices[face[0]];
                let v2 = mesh.vertices[face[1]];
                let v3 = mesh.vertices[face[2]];
                
                // Calculate normal
                let normal = if i < mesh.normals.len() {
                    mesh.normals[i]
                } else {
                    // Calculate face normal
                    let u = v2 - v1;
                    let v = v3 - v1;
                    let cross = u.cross(&v);
                    cross.normalize()
                };
                
                content.push_str(&format!("  facet normal {} {} {}\n", normal.x, normal.y, normal.z));
                content.push_str("    outer loop\n");
                content.push_str(&format!("      vertex {} {} {}\n", v1.x, v1.y, v1.z));
                content.push_str(&format!("      vertex {} {} {}\n", v2.x, v2.y, v2.z));
                content.push_str(&format!("      vertex {} {} {}\n", v3.x, v3.y, v3.z));
                content.push_str("    endloop\n");
                content.push_str("  endfacet\n");
            }
        }
        
        content.push_str("endsolid mesh\n");
        Ok(content)
    }
    
    /// Serialize mesh to OBJ
    fn serialize_obj(mesh: &wavecore_meshes::Mesh) -> Result<String> {
        let mut content = String::new();
        content.push_str("# WaveCore mesh export\n");
        
        // Write vertices
        for vertex in &mesh.vertices {
            content.push_str(&format!("v {} {} {}\n", vertex.x, vertex.y, vertex.z));
        }
        
        // Write normals
        for normal in &mesh.normals {
            content.push_str(&format!("vn {} {} {}\n", normal.x, normal.y, normal.z));
        }
        
        // Write faces
        for face in &mesh.faces {
            if face.len() >= 3 {
                content.push_str(&format!("f {} {} {}\n", 
                    face[0] + 1, face[1] + 1, face[2] + 1)); // OBJ indices are 1-based
            }
        }
        
        Ok(content)
    }
    
    /// Serialize data to CSV
    fn serialize_csv(data: &DataArray) -> Result<String> {
        let mut content = String::new();
        
        // Write header
        let dims = data.dimensions();
        let dims_str: Vec<String> = dims.iter().map(|d| d.to_string()).collect();
        content.push_str(&format!("dimensions,{}\n", dims_str.join(",")));
        
        // Write data
        let values = data.as_slice();
        for (i, value) in values.iter().enumerate() {
            content.push_str(&format!("{},{}\n", i, value));
        }
        
        Ok(content)
    }
    
    /// Parse CSV data
    fn parse_csv(content: &str) -> Result<DataArray> {
        let lines: Vec<&str> = content.lines().collect();
        let mut values = Vec::new();
        let mut dimensions = Vec::new();
        
        for (i, line) in lines.iter().enumerate() {
            let parts: Vec<&str> = line.split(',').collect();
            
            if i == 0 && line.starts_with("dimensions") {
                // Parse dimensions
                for part in parts.iter().skip(1) {
                    if let Ok(dim) = part.parse::<usize>() {
                        dimensions.push(dim);
                    }
                }
            } else if parts.len() >= 2 {
                // Parse data values
                if let Ok(value) = parts[1].parse::<f64>() {
                    values.push(value);
                }
            }
        }
        
        if dimensions.is_empty() {
            dimensions = vec![values.len()];
        }
        
        DataArray::new(&dimensions, &values)
            .map_err(|e| IOError::DataArrayError {
                message: format!("Failed to create data array: {}", e),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_format() {
        assert_eq!(FileIO::detect_format("test.stl").unwrap(), Format::STL);
        assert_eq!(FileIO::detect_format("test.obj").unwrap(), Format::OBJ);
        assert_eq!(FileIO::detect_format("test.json").unwrap(), Format::JSON);
        assert_eq!(FileIO::detect_format("test.yaml").unwrap(), Format::YAML);
        assert_eq!(FileIO::detect_format("test.csv").unwrap(), Format::CSV);
    }
    
    #[test]
    fn test_parse_stl() {
        let stl_content = r#"solid test
facet normal 0 0 1
outer loop
vertex 0 0 0
vertex 1 0 0
vertex 0 1 0
endloop
endfacet
endsolid test"#;
        
        let mesh = FileIO::parse_stl(stl_content).unwrap();
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.faces.len(), 1);
    }
    
    #[test]
    fn test_parse_obj() {
        let obj_content = r#"
# Test mesh
v 0 0 0
v 1 0 0
v 0 1 0
f 1 2 3
        "#;
        
        let mesh = FileIO::parse_obj(obj_content).unwrap();
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.faces.len(), 1);
    }
    
    #[test]
    fn test_serialize_csv() {
        let data = DataArray::new(&[3], &vec![1.0, 2.0, 3.0]).unwrap();
        let csv = FileIO::serialize_csv(&data).unwrap();
        assert!(csv.contains("1"));
        assert!(csv.contains("2"));
        assert!(csv.contains("3"));
    }
} 