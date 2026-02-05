use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use tar::{Archive, Builder};
use anyhow::{Context, Result};

use crate::metadata::Metadata;

/// Representa un paquete .lis
pub struct LisPackage {
    pub metadata: Metadata,
    pub binary_path: PathBuf,
    pub resources_dir: Option<PathBuf>,
    pub dependencies: Vec<PathBuf>,
}

impl LisPackage {
    /// Crear un nuevo paquete
    pub fn new(
        metadata: Metadata,
        binary_path: PathBuf,
        resources_dir: Option<PathBuf>,
        dependencies: Vec<PathBuf>,
    ) -> Self {
        Self {
            metadata,
            binary_path,
            resources_dir,
            dependencies,
        }
    }

    /// Generar archivo .lis
    pub fn build<P: AsRef<Path>>(&self, output_path: P) -> Result<()> {
        let file = File::create(output_path.as_ref())
            .context("Failed to create output file")?;
        let encoder = GzEncoder::new(file, Compression::default());
        let mut archive = Builder::new(encoder);

        // Agregar metadata.toml
        let metadata_toml = self.metadata.to_toml()?;
        let mut header = tar::Header::new_gnu();
        header.set_path("metadata.toml")?;
        header.set_size(metadata_toml.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        archive.append(&header, metadata_toml.as_bytes())?;

        // Agregar binario
        let binary_name = self.binary_path.file_name()
            .context("Invalid binary path")?;
        archive.append_path_with_name(&self.binary_path, format!("binary/{}", binary_name.to_string_lossy()))?;

        // Agregar recursos si existen
        if let Some(ref resources_dir) = self.resources_dir {
            if resources_dir.exists() {
                archive.append_dir_all("resources", resources_dir)?;
            }
        }

        // Agregar dependencias
        for dep in &self.dependencies {
            let dep_name = dep.file_name()
                .context("Invalid dependency path")?;
            archive.append_path_with_name(dep, format!("dependencies/{}", dep_name.to_string_lossy()))?;
        }

        archive.finish()?;
        Ok(())
    }

    /// Extraer y leer un archivo .lis
    pub fn extract<P: AsRef<Path>>(lis_path: P, output_dir: P) -> Result<Metadata> {
        let file = File::open(lis_path.as_ref())
            .context("Failed to open .lis file")?;
        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);

        // Extraer todo
        archive.unpack(output_dir.as_ref())
            .context("Failed to extract .lis file")?;

        // Leer metadata
        let metadata_path = output_dir.as_ref().join("metadata.toml");
        let mut metadata_content = String::new();
        File::open(&metadata_path)?
            .read_to_string(&mut metadata_content)?;

        Metadata::from_toml(&metadata_content)
    }
}
