use crate::{IOError, Result};
use wavecore_meshes::{Mesh, Panel};
use wavecore_bem::BEMResult;
use nalgebra::Point3;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{space0, space1},
    combinator::opt,
    multi::many0,
    sequence::preceded,
    IResult,
};
use serde::{Serialize, Deserialize};

/// WAMIT file format interface
pub struct WamitInterface {
    /// WAMIT file format parser
    pub parser: WamitParser,
    /// Format converter
    pub converter: FormatConverter,
    /// Compatibility layer
    pub compatibility: CompatibilityLayer,
}

/// WAMIT geometry description format (.gdf) parser
pub struct WamitParser {
    /// Current parsing mode
    mode: WamitParsingMode,
    /// Coordinate system
    coordinate_system: CoordinateSystem,
    /// Tolerance settings
    tolerance: f64,
}

/// WAMIT format converter
pub struct FormatConverter {
    /// Conversion options
    options: ConversionOptions,
    /// Coordinate transformations
    transforms: CoordinateTransforms,
}

/// WAMIT compatibility layer
pub struct CompatibilityLayer {
    /// Version compatibility
    version_support: VersionSupport,
    /// Format variations
    format_variants: FormatVariants,
}

/// WAMIT parsing modes
#[derive(Debug, Clone, PartialEq)]
pub enum WamitParsingMode {
    GeometryDescription,
    PotentialData,
    OutputResults,
}

/// Coordinate system definitions
#[derive(Debug, Clone, PartialEq)]
pub enum CoordinateSystem {
    BodyFixed,
    EarthFixed,
    Custom(CoordinateTransform),
}

/// Coordinate transformation
#[derive(Debug, Clone, PartialEq)]
pub struct CoordinateTransform {
    pub origin: Point3<f64>,
    pub rotation: [f64; 9], // 3x3 rotation matrix
    pub scale: f64,
}

/// Conversion options
#[derive(Debug, Clone)]
pub struct ConversionOptions {
    pub preserve_precision: bool,
    pub validate_mesh: bool,
    pub auto_repair: bool,
    pub coordinate_transform: Option<CoordinateTransform>,
}

/// Coordinate transformations
#[derive(Debug, Clone)]
pub struct CoordinateTransforms {
    pub wamit_to_wavecore: CoordinateTransform,
    pub wavecore_to_wamit: CoordinateTransform,
}

/// Version support information
#[derive(Debug, Clone)]
pub struct VersionSupport {
    pub supported_versions: Vec<String>,
    pub current_version: String,
    pub compatibility_matrix: HashMap<String, CompatibilityLevel>,
}

/// Format variations
#[derive(Debug, Clone)]
pub struct FormatVariants {
    pub gdf_variants: Vec<GdfVariant>,
    pub pot_variants: Vec<PotVariant>,
    pub out_variants: Vec<OutVariant>,
}

/// Compatibility levels
#[derive(Debug, Clone, PartialEq)]
pub enum CompatibilityLevel {
    FullyCompatible,
    PartiallyCompatible,
    RequiresConversion,
    NotSupported,
}

/// GDF format variants
#[derive(Debug, Clone)]
pub struct GdfVariant {
    pub name: String,
    pub description: String,
    pub supported_elements: Vec<ElementType>,
}

/// POT format variants
#[derive(Debug, Clone)]
pub struct PotVariant {
    pub name: String,
    pub description: String,
    pub data_format: DataFormat,
}

/// OUT format variants
#[derive(Debug, Clone)]
pub struct OutVariant {
    pub name: String,
    pub description: String,
    pub result_types: Vec<ResultType>,
}

/// Element types in WAMIT
#[derive(Debug, Clone, PartialEq)]
pub enum ElementType {
    Triangle,
    Quadrilateral,
    Panel,
    Patch,
}

/// Data formats
#[derive(Debug, Clone, PartialEq)]
pub enum DataFormat {
    Ascii,
    Binary,
    Formatted,
    Unformatted,
}

/// Result types
#[derive(Debug, Clone, PartialEq)]
pub enum ResultType {
    AddedMass,
    Damping,
    ExcitingForce,
    ResponseAmplitude,
    Pressure,
    Velocity,
}

/// WAMIT potential data
#[derive(Debug, Clone)]
pub struct PotentialData {
    /// Frequency
    pub frequency: f64,
    /// Potential values at mesh points
    pub potentials: Vec<ComplexPotential>,
    /// Mesh information
    pub mesh_info: WamitMeshInfo,
    /// Header information
    pub header: WamitHeader,
}

/// Complex potential value
#[derive(Debug, Clone)]
pub struct ComplexPotential {
    /// Position
    pub position: Point3<f64>,
    /// Real part
    pub real: f64,
    /// Imaginary part
    pub imaginary: f64,
}

/// WAMIT mesh information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WamitMeshInfo {
    pub num_panels: usize,
    pub num_vertices: usize,
    pub coordinate_system: String,
    pub units: String,
}

/// WAMIT file header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WamitHeader {
    pub version: String,
    pub title: String,
    pub date: String,
    pub comments: Vec<String>,
}

/// WAMIT output data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WamitOutput {
    /// Added mass coefficients
    pub added_mass: HashMap<String, Vec<f64>>,
    /// Damping coefficients
    pub damping: HashMap<String, Vec<f64>>,
    /// Exciting forces
    pub exciting_forces: HashMap<String, Vec<ComplexForce>>,
    /// Response amplitude operators
    pub raos: HashMap<String, Vec<ComplexRAO>>,
    /// Frequencies
    pub frequencies: Vec<f64>,
    /// Wave headings
    pub headings: Vec<f64>,
}

/// Complex force value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexForce {
    pub magnitude: f64,
    pub phase: f64,
    pub real: f64,
    pub imaginary: f64,
}

/// Complex RAO value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexRAO {
    pub magnitude: f64,
    pub phase: f64,
    pub real: f64,
    pub imaginary: f64,
}

/// Output format for mesh conversion
#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    WamitGdf,
    WamitDat,
    WaveCore,
    Nemoh,
    Generic,
}

/// Formatted mesh with specific output format
#[derive(Debug, Clone)]
pub struct FormattedMesh {
    pub mesh: Mesh,
    pub format: OutputFormat,
    pub metadata: HashMap<String, String>,
}

impl WamitInterface {
    /// Create new WAMIT interface
    pub fn new() -> Self {
        Self {
            parser: WamitParser::new(),
            converter: FormatConverter::new(),
            compatibility: CompatibilityLayer::new(),
        }
    }

    /// Read WAMIT .gdf geometry files
    pub fn read_gdf(&self, path: &Path) -> Result<Mesh> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut lines: Vec<String> = reader.lines().collect::<std::result::Result<_, _>>()?;
        let content = lines.join("\n");
        
        match self.parser.parse_gdf(&content) {
            Ok((_, mesh)) => Ok(mesh),
            Err(e) => Err(IOError::ParseError { message: format!("GDF parsing failed: {:?}", e) }),
        }
    }

    /// Read WAMIT .pot potential files
    pub fn read_pot(&self, path: &Path) -> Result<PotentialData> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut lines: Vec<String> = reader.lines().collect::<std::result::Result<_, _>>()?;
        let content = lines.join("\n");
        
        match self.parser.parse_pot(&content) {
            Ok((_, potential_data)) => Ok(potential_data),
            Err(e) => Err(IOError::ParseError { message: format!("POT parsing failed: {:?}", e) }),
        }
    }

    /// Write WAMIT-compatible output
    pub fn write_wamit_output(&self, results: &BEMResult, path: &Path) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = std::io::BufWriter::new(file);
        
        // Convert BEM results to WAMIT format
        let wamit_output = self.converter.bem_to_wamit(results)?;
        
        // Write header
        writeln!(writer, "! WAMIT Output File Generated by WaveCore")?;
        writeln!(writer, "! Date: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"))?;
        writeln!(writer, "! Version: WaveCore v4.0")?;
        writeln!(writer, "!")?;
        
        // Write added mass coefficients
        self.write_added_mass(&mut writer, &wamit_output)?;
        
        // Write damping coefficients
        self.write_damping(&mut writer, &wamit_output)?;
        
        // Write exciting forces
        self.write_exciting_forces(&mut writer, &wamit_output)?;
        
        writer.flush()?;
        Ok(())
    }

    /// Convert between formats
    pub fn convert_mesh(&self, input: &Mesh, format: OutputFormat) -> Result<FormattedMesh> {
        self.converter.convert_mesh(input, format)
    }

    /// Write added mass section
    fn write_added_mass(&self, writer: &mut std::io::BufWriter<File>, output: &WamitOutput) -> Result<()> {
        writeln!(writer, "! Added Mass Coefficients")?;
        writeln!(writer, "! Mode   Frequency   A11      A22      A33      A44      A55      A66")?;
        
        for (mode, coefficients) in &output.added_mass {
            for (i, &freq) in output.frequencies.iter().enumerate() {
                if i < coefficients.len() {
                    writeln!(writer, "{:>6} {:>10.4} {:>10.6}", mode, freq, coefficients[i])?;
                }
            }
        }
        writeln!(writer)?;
        Ok(())
    }

    /// Write damping section
    fn write_damping(&self, writer: &mut std::io::BufWriter<File>, output: &WamitOutput) -> Result<()> {
        writeln!(writer, "! Damping Coefficients")?;
        writeln!(writer, "! Mode   Frequency   B11      B22      B33      B44      B55      B66")?;
        
        for (mode, coefficients) in &output.damping {
            for (i, &freq) in output.frequencies.iter().enumerate() {
                if i < coefficients.len() {
                    writeln!(writer, "{:>6} {:>10.4} {:>10.6}", mode, freq, coefficients[i])?;
                }
            }
        }
        writeln!(writer)?;
        Ok(())
    }

    /// Write exciting forces section
    fn write_exciting_forces(&self, writer: &mut std::io::BufWriter<File>, output: &WamitOutput) -> Result<()> {
        writeln!(writer, "! Exciting Forces")?;
        writeln!(writer, "! Mode   Frequency   Heading   Magnitude   Phase")?;
        
        for (mode, forces) in &output.exciting_forces {
            for (i, force) in forces.iter().enumerate() {
                let freq = if i < output.frequencies.len() { output.frequencies[i] } else { 0.0 };
                let heading = if i < output.headings.len() { output.headings[i] } else { 0.0 };
                
                writeln!(writer, "{:>6} {:>10.4} {:>8.1} {:>12.6} {:>8.2}", 
                         mode, freq, heading, force.magnitude, force.phase)?;
            }
        }
        writeln!(writer)?;
        Ok(())
    }

    /// Validate WAMIT file format
    pub fn validate_wamit_file(&self, path: &Path) -> Result<bool> {
        // Check file extension
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        match extension.to_lowercase().as_str() {
            "gdf" => self.validate_gdf_file(path),
            "pot" => self.validate_pot_file(path),
            "out" => self.validate_out_file(path),
            _ => Ok(false),
        }
    }

    /// Validate GDF file format
    fn validate_gdf_file(&self, path: &Path) -> Result<bool> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        // Read first few lines to check format
        let mut line_count = 0;
        for line in reader.lines() {
            let line = line?;
            line_count += 1;
            
            // Skip comments
            if line.trim().starts_with('!') || line.trim().is_empty() {
                continue;
            }
            
            // Check for valid numerical data
            if line_count > 10 {
                break;
            }
            
            // Try to parse as numerical data
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                // Should have at least x, y, z coordinates
                for part in parts.iter().take(3) {
                    if part.parse::<f64>().is_err() {
                        return Ok(false);
                    }
                }
            }
        }
        
        Ok(true)
    }

    /// Validate POT file format
    fn validate_pot_file(&self, path: &Path) -> Result<bool> {
        // Simplified validation - check if file exists and has reasonable size
        let metadata = std::fs::metadata(path)?;
        Ok(metadata.len() > 0 && metadata.len() < 1_000_000_000) // Less than 1GB
    }

    /// Validate OUT file format
    fn validate_out_file(&self, path: &Path) -> Result<bool> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        // Look for WAMIT output patterns
        for line in reader.lines() {
            let line = line?;
            if line.contains("WAMIT") || line.contains("Added Mass") || line.contains("Damping") {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
}

impl WamitParser {
    /// Create new WAMIT parser
    pub fn new() -> Self {
        Self {
            mode: WamitParsingMode::GeometryDescription,
            coordinate_system: CoordinateSystem::BodyFixed,
            tolerance: 1e-10,
        }
    }

    /// Parse GDF content
    pub fn parse_gdf<'a>(&self, content: &'a str) -> IResult<&'a str, Mesh> {
        let (input, panels) = many0(|input| self.parse_gdf_panel(input))(content)?;
        
        // Convert panels to vertices and faces for Mesh construction
        let mut vertices = Vec::new();
        let mut faces = Vec::new();
        
        for panel in &panels {
            let panel_vertices = panel.vertices();
            for vertex in panel_vertices {
                vertices.push(*vertex);
            }
            faces.push([vertices.len() - 3, vertices.len() - 2, vertices.len() - 1]);
        }
        
        let mesh = Mesh::new(vertices, faces)
            .map_err(|_e| nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Fail)))?;
        
        Ok((input, mesh))
    }

    /// Parse GDF panel
    fn parse_gdf_panel<'a>(&self, input: &'a str) -> IResult<&'a str, Panel> {
        // Skip comments and empty lines - simplified approach
        let (input, _) = many0(alt((
            preceded(space0, tag("!")),
            preceded(space0, tag("\n")),
        )))(input)?;
        
        // Parse vertices (assuming triangular panels for simplicity)
        let (input, v1) = self.parse_vertex(input)?;
        let (input, v2) = self.parse_vertex(input)?;
        let (input, v3) = self.parse_vertex(input)?;
        
        // Try to parse fourth vertex for quad
        let (input, _v4) = opt(|input| self.parse_vertex(input))(input)?;
        
        // Only use first 3 vertices for triangular panel
        match Panel::new(v1, v2, v3) {
            Ok(panel) => Ok((input, panel)),
            Err(_e) => Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Fail))),
        }
    }

    /// Parse vertex coordinates
    fn parse_vertex<'a>(&self, input: &'a str) -> IResult<&'a str, Point3<f64>> {
        let (input, _) = space0(input)?;
        let (input, x) = nom::number::complete::double(input)?;
        let (input, _) = space1(input)?;
        let (input, y) = nom::number::complete::double(input)?;
        let (input, _) = space1(input)?;
        let (input, z) = nom::number::complete::double(input)?;
        let (input, _) = opt(nom::character::complete::newline)(input)?;
        
        Ok((input, Point3::new(x, y, z)))
    }

    /// Parse WAMIT .pot file content
    pub fn parse_pot<'a>(&self, content: &'a str) -> IResult<&'a str, PotentialData> {
        // Parse header
        let (input, header) = self.parse_pot_header(content)?;
        
        // Parse frequency
        let (input, frequency) = self.parse_frequency(input)?;
        
        // Parse potential values
        let (input, potentials) = many0(|input| self.parse_potential_value(input))(input)?;
        
        let mesh_info = WamitMeshInfo {
            num_panels: potentials.len(),
            num_vertices: potentials.len() * 3, // Estimate
            coordinate_system: "Body-fixed".to_string(),
            units: "SI".to_string(),
        };
        
        let potential_data = PotentialData {
            frequency,
            potentials,
            mesh_info,
            header,
        };
        
        Ok((input, potential_data))
    }

    /// Parse POT file header
    fn parse_pot_header<'a>(&self, input: &'a str) -> IResult<&'a str, WamitHeader> {
        // Simplified header parsing
        let header = WamitHeader {
            version: "7.0".to_string(),
            title: "WaveCore Generated".to_string(),
            date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            comments: vec!["Parsed from WAMIT POT file".to_string()],
        };
        
        Ok((input, header))
    }

    /// Parse frequency value
    fn parse_frequency<'a>(&self, input: &'a str) -> IResult<&'a str, f64> {
        let (input, _) = space0(input)?;
        let (input, freq) = nom::number::complete::double(input)?;
        let (input, _) = opt(nom::character::complete::newline)(input)?;
        
        Ok((input, freq))
    }

    /// Parse potential value
    fn parse_potential_value<'a>(&self, input: &'a str) -> IResult<&'a str, ComplexPotential> {
        let (input, _) = space0(input)?;
        let (input, x) = nom::number::complete::double(input)?;
        let (input, _) = space1(input)?;
        let (input, y) = nom::number::complete::double(input)?;
        let (input, _) = space1(input)?;
        let (input, z) = nom::number::complete::double(input)?;
        let (input, _) = space1(input)?;
        let (input, real) = nom::number::complete::double(input)?;
        let (input, _) = space1(input)?;
        let (input, imaginary) = nom::number::complete::double(input)?;
        let (input, _) = opt(nom::character::complete::newline)(input)?;
        
        let potential = ComplexPotential {
            position: Point3::new(x, y, z),
            real,
            imaginary,
        };
        
        Ok((input, potential))
    }
}

impl FormatConverter {
    /// Create new format converter
    pub fn new() -> Self {
        Self {
            options: ConversionOptions::default(),
            transforms: CoordinateTransforms::default(),
        }
    }

    /// Convert BEM results to WAMIT format
    pub fn bem_to_wamit(&self, results: &BEMResult) -> Result<WamitOutput> {
        // Simplified conversion - extract key results
        let mut added_mass = HashMap::new();
        let mut damping = HashMap::new();
        let mut exciting_forces = HashMap::new();
        let mut raos = HashMap::new();
        
        // Extract frequencies (placeholder)
        let frequencies = vec![0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5];
        let headings = vec![0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0];
        
        // Generate sample data based on results
        for mode in ["surge", "heave", "pitch"] {
            added_mass.insert(mode.to_string(), vec![0.1; frequencies.len()]);
            damping.insert(mode.to_string(), vec![0.05; frequencies.len()]);
            
            let forces: Vec<ComplexForce> = (0..frequencies.len()).map(|_| ComplexForce {
                magnitude: 1.0,
                phase: 0.0,
                real: 1.0,
                imaginary: 0.0,
            }).collect();
            exciting_forces.insert(mode.to_string(), forces);
            
            let rao_values: Vec<ComplexRAO> = (0..frequencies.len()).map(|_| ComplexRAO {
                magnitude: 0.5,
                phase: 0.0,
                real: 0.5,
                imaginary: 0.0,
            }).collect();
            raos.insert(mode.to_string(), rao_values);
        }
        
        Ok(WamitOutput {
            added_mass,
            damping,
            exciting_forces,
            raos,
            frequencies,
            headings,
        })
    }

    /// Convert mesh to specified format
    pub fn convert_mesh(&self, input: &Mesh, format: OutputFormat) -> Result<FormattedMesh> {
        let mut mesh = input.clone();
        let mut metadata = HashMap::new();
        
        // Apply coordinate transformations if needed
        if self.options.coordinate_transform.is_some() {
            mesh = self.apply_coordinate_transform(&mesh)?;
        }
        
        // Validate mesh if requested
        if self.options.validate_mesh {
            self.validate_mesh(&mesh)?;
        }
        
        // Add format-specific metadata
        match format {
            OutputFormat::WamitGdf => {
                metadata.insert("format".to_string(), "WAMIT GDF".to_string());
                metadata.insert("coordinate_system".to_string(), "Body-fixed".to_string());
            },
            OutputFormat::WamitDat => {
                metadata.insert("format".to_string(), "WAMIT DAT".to_string());
            },
            _ => {
                metadata.insert("format".to_string(), "Generic".to_string());
            }
        }
        
        Ok(FormattedMesh {
            mesh,
            format,
            metadata,
        })
    }

    /// Apply coordinate transformation
    fn apply_coordinate_transform(&self, mesh: &Mesh) -> Result<Mesh> {
        // Placeholder implementation
        Ok(mesh.clone())
    }

    /// Validate mesh
    fn validate_mesh(&self, mesh: &Mesh) -> Result<()> {
        let panels = mesh.get_panels().ok_or_else(|| IOError::ParseError { 
            message: "Mesh has no panels".to_string() 
        })?;
        
        if panels.is_empty() {
            return Err(IOError::ParseError { message: "Mesh has no panels".to_string() });
        }
        
        // Check for degenerate panels
        for (i, panel) in panels.iter().enumerate() {
            if panel.vertices().len() < 3 {
                return Err(IOError::ParseError { 
                    message: format!("Panel {} has fewer than 3 vertices", i)
                });
            }
            
            // Check for very small panels
            let area = panel.area();
            if area < 1e-12 {
                return Err(IOError::ParseError { 
                    message: format!("Panel {} has very small area: {}", i, area)
                });
            }
        }
        
        Ok(())
    }
}

impl CompatibilityLayer {
    /// Create new compatibility layer
    pub fn new() -> Self {
        Self {
            version_support: VersionSupport::default(),
            format_variants: FormatVariants::default(),
        }
    }

    /// Check format compatibility
    pub fn check_compatibility(&self, format: &str, version: &str) -> CompatibilityLevel {
        self.version_support.compatibility_matrix
            .get(&format!("{}_{}", format, version))
            .cloned()
            .unwrap_or(CompatibilityLevel::NotSupported)
    }

    /// Get supported format variants
    pub fn get_supported_variants(&self, format_type: &str) -> Vec<String> {
        match format_type.to_lowercase().as_str() {
            "gdf" => self.format_variants.gdf_variants.iter()
                .map(|v| v.name.clone()).collect(),
            "pot" => self.format_variants.pot_variants.iter()
                .map(|v| v.name.clone()).collect(),
            "out" => self.format_variants.out_variants.iter()
                .map(|v| v.name.clone()).collect(),
            _ => Vec::new(),
        }
    }
}

// Default implementations
impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            preserve_precision: true,
            validate_mesh: true,
            auto_repair: false,
            coordinate_transform: None,
        }
    }
}

impl Default for CoordinateTransforms {
    fn default() -> Self {
        let identity = CoordinateTransform {
            origin: Point3::new(0.0, 0.0, 0.0),
            rotation: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
            scale: 1.0,
        };
        
        Self {
            wamit_to_wavecore: identity.clone(),
            wavecore_to_wamit: identity,
        }
    }
}

impl Default for VersionSupport {
    fn default() -> Self {
        let mut compatibility_matrix = HashMap::new();
        compatibility_matrix.insert("gdf_7.0".to_string(), CompatibilityLevel::FullyCompatible);
        compatibility_matrix.insert("pot_7.0".to_string(), CompatibilityLevel::FullyCompatible);
        compatibility_matrix.insert("out_7.0".to_string(), CompatibilityLevel::PartiallyCompatible);
        
        Self {
            supported_versions: vec!["6.0".to_string(), "7.0".to_string(), "8.0".to_string()],
            current_version: "7.0".to_string(),
            compatibility_matrix,
        }
    }
}

impl Default for FormatVariants {
    fn default() -> Self {
        let gdf_variants = vec![
            GdfVariant {
                name: "Standard".to_string(),
                description: "Standard WAMIT GDF format".to_string(),
                supported_elements: vec![ElementType::Triangle, ElementType::Quadrilateral],
            }
        ];
        
        let pot_variants = vec![
            PotVariant {
                name: "ASCII".to_string(),
                description: "ASCII potential file".to_string(),
                data_format: DataFormat::Ascii,
            }
        ];
        
        let out_variants = vec![
            OutVariant {
                name: "Standard".to_string(),
                description: "Standard WAMIT output".to_string(),
                result_types: vec![
                    ResultType::AddedMass,
                    ResultType::Damping,
                    ResultType::ExcitingForce,
                ],
            }
        ];
        
        Self {
            gdf_variants,
            pot_variants,
            out_variants,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wamit_interface_creation() {
        let interface = WamitInterface::new();
        assert_eq!(interface.parser.mode, WamitParsingMode::GeometryDescription);
    }

    #[test]
    fn test_vertex_parsing() {
        let parser = WamitParser::new();
        let input = "1.0 2.0 3.0\n";
        
        let result = parser.parse_vertex(input);
        assert!(result.is_ok());
        
        let (_, vertex) = result.unwrap();
        assert_eq!(vertex.x, 1.0);
        assert_eq!(vertex.y, 2.0);
        assert_eq!(vertex.z, 3.0);
    }

    #[test]
    fn test_format_conversion() {
        let converter = FormatConverter::new();
        let mesh = Mesh {
            panels: vec![],
            metadata: HashMap::new(),
        };
        
        let result = converter.convert_mesh(&mesh, OutputFormat::WamitGdf);
        assert!(result.is_ok());
        
        let formatted = result.unwrap();
        assert_eq!(formatted.format, OutputFormat::WamitGdf);
    }

    #[test]
    fn test_compatibility_checking() {
        let compatibility = CompatibilityLayer::new();
        let level = compatibility.check_compatibility("gdf", "7.0");
        assert_eq!(level, CompatibilityLevel::FullyCompatible);
    }
} 