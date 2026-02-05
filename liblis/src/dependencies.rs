use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Context, Result};
use walkdir::WalkDir;

/// Detectar dependencias de un binario usando ldd
pub fn detect_dependencies<P: AsRef<Path>>(binary_path: P) -> Result<Vec<PathBuf>> {
    let output = Command::new("ldd")
        .arg(binary_path.as_ref())
        .output()
        .context("Failed to run ldd")?;

    if !output.status.success() {
        anyhow::bail!("ldd command failed");
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut dependencies = Vec::new();

    // Lista de librerías a incluir (GTK, Adwaita, etc.)
    let include_patterns = [
        "libgtk-4",
        "libadwaita-1",
        "libgio-2",
        "libglib-2",
        "libgobject-2",
        "libpango",
        "libcairo",
        "libgdk_pixbuf",
    ];

    for line in output_str.lines() {
        // Formato típico: "libgtk-4.so.1 => /usr/lib/libgtk-4.so.1 (0x...)"
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 && parts[1] == "=>" {
            let lib_name = parts[0];
            let lib_path = parts[2];

            // Verificar si esta librería debe ser incluida
            if include_patterns.iter().any(|p| lib_name.starts_with(p)) {
                if let Ok(path) = PathBuf::from(lib_path).canonicalize() {
                    dependencies.push(path);
                }
            }
        }
    }

    Ok(dependencies)
}

/// Buscar recursos en un directorio de proyecto Meson
pub fn find_resources<P: AsRef<Path>>(project_dir: P) -> Result<Option<PathBuf>> {
    let data_dir = project_dir.as_ref().join("data");
    if data_dir.exists() && data_dir.is_dir() {
        Ok(Some(data_dir))
    } else {
        Ok(None)
    }
}

/// Buscar el binario compilado en builddir de Meson
pub fn find_binary<P: AsRef<Path>>(project_dir: P, binary_name: &str) -> Result<PathBuf> {
    let builddir = project_dir.as_ref().join("builddir");
    
    for entry in WalkDir::new(&builddir)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(filename) = entry.file_name().to_str() {
                if filename == binary_name {
                    return Ok(entry.path().to_path_buf());
                }
            }
        }
    }

    anyhow::bail!("Binary '{}' not found in builddir", binary_name)
}
