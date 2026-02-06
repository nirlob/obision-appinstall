use serde::{Deserialize, Serialize};

/// Metadata del paquete .lis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub package: PackageInfo,
    pub installation: InstallationInfo,
    pub desktop: DesktopInfo,
    pub dependencies: DependenciesInfo,
    #[serde(default)]
    pub files: Vec<FileEntry>,
    #[serde(default)]
    pub installer_screens: Vec<InstallerScreen>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub app_id: String,
    pub description: String,
    pub author: String,
    pub application_name: String,
    pub package_name: String,
    pub compression_level: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationInfo {
    /// Installation prefix for system-wide installation (e.g., /usr/local)
    pub prefix_system: String,
    /// Installation prefix for user installation (e.g., ~/.local)
    pub prefix_user: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopInfo {
    pub name: String,
    pub exec: String,
    pub icon: String,
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependenciesInfo {
    pub bundled: Vec<String>,
}

/// Represents a file to be installed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// Source filename (relative to application/ folder in the .lis)
    pub source: String,
    /// Destination path (relative to install directory)
    pub destination: String,
    /// File permissions (Unix-style, e.g., "755" for executables)
    pub permissions: Option<String>,
}

/// Installer screen configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallerScreen {
    /// Screen ID (welcome, license, directory, components, progress, finish)
    pub id: String,
    /// Whether this screen is enabled
    pub enabled: bool,
    /// Display order (1-based)
    pub order: usize,
    /// Optional custom content (e.g., license file path)
    pub custom_content: Option<String>,
}

impl Metadata {
    /// Parse metadata from TOML string
    pub fn from_toml(toml_str: &str) -> anyhow::Result<Self> {
        let metadata: Metadata = toml::from_str(toml_str)?;
        Ok(metadata)
    }

    /// Serialize metadata to TOML string
    pub fn to_toml(&self) -> anyhow::Result<String> {
        let toml_str = toml::to_string_pretty(self)?;
        Ok(toml_str)
    }
}
