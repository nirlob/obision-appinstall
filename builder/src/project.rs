use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a file to be included in the package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFile {
    /// Source path (absolute)
    pub source: PathBuf,
    /// Destination path (relative to install directory)
    pub destination: String,
    /// File permissions (Unix-style, e.g., "755" for executables)
    pub permissions: Option<String>,
}

/// Installer screen configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallerScreen {
    /// Screen ID (welcome, license, install_location, finish)
    /// Note: progress screen is always shown automatically during installation
    pub id: String,
    /// Whether this screen is enabled
    pub enabled: bool,
    /// Display order (1-based)
    pub order: usize,
    /// Optional custom content (e.g., license file path)
    pub custom_content: Option<String>,
}

/// Project metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    /// Project name
    pub name: String,
    /// Project version
    pub version: String,
    /// Author name
    pub author: String,
    /// Project description
    pub description: String,
    /// Application name (Display name)
    pub application_name: String,
    /// Output directory for generated .lis file
    pub output_directory: PathBuf,
    /// Desktop file path
    #[serde(default)]
    pub desktop_file: Option<PathBuf>,
}

/// Main project structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Project metadata
    pub metadata: ProjectMetadata,
    /// List of files to include
    #[serde(default)]
    pub files: Vec<ProjectFile>,
    /// Installer screen configuration
    pub installer_screens: Vec<InstallerScreen>,
    /// Package name (output filename)
    pub package_name: String,
    /// Compression level (0-9)
    pub compression_level: u8,
}

impl Project {
    /// Create a new empty project with default values
    pub fn new() -> Self {
        Self {
            metadata: ProjectMetadata {
                name: String::from("MyProject"),
                version: String::from("1.0.0"),
                author: String::new(),
                description: String::new(),
                application_name: String::from("My Application"),
                output_directory: PathBuf::from("."),
                desktop_file: None,
            },
            files: Vec::new(),
            installer_screens: Self::default_screens(),
            package_name: String::from("package.lis"),
            compression_level: 9,
        }
    }

    /// Get default installer screens
    fn default_screens() -> Vec<InstallerScreen> {
        vec![
            InstallerScreen {
                id: String::from("welcome"),
                enabled: true,
                order: 1,
                custom_content: None,
            },
            InstallerScreen {
                id: String::from("license"),
                enabled: true,
                order: 2,
                custom_content: None,
            },
            InstallerScreen {
                id: String::from("install_location"),
                enabled: true,
                order: 3,
                custom_content: None,
            },
            InstallerScreen {
                id: String::from("finish"),
                enabled: true,
                order: 4,
                custom_content: None,
            },
        ]
    }

    /// Save project to XML file
    #[allow(dead_code)]
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), String> {
        let xml = quick_xml::se::to_string(&self)
            .map_err(|e| format!("Failed to serialize project: {}", e))?;
        
        std::fs::write(path, xml)
            .map_err(|e| format!("Failed to write project file: {}", e))?;
        
        Ok(())
    }

    /// Load project from XML file
    pub fn load_from_file(path: &PathBuf) -> Result<Self, String> {
        let xml = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read project file: {}", e))?;
        
        let project: Project = quick_xml::de::from_str(&xml)
            .map_err(|e| format!("Failed to deserialize project: {}", e))?;
        
        Ok(project)
    }
}

impl Default for Project {
    fn default() -> Self {
        Self::new()
    }
}
