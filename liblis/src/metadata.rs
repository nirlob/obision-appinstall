use serde::{Deserialize, Serialize};

/// Metadata del paquete .lis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub package: PackageInfo,
    pub installation: InstallationInfo,
    pub desktop: DesktopInfo,
    pub dependencies: DependenciesInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub app_id: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationInfo {
    pub target_dir_system: String,
    pub target_dir_user: String,
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
