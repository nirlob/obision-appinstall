pub mod metadata;
pub mod format;
pub mod dependencies;

// Re-export main types
pub use metadata::Metadata;
pub use format::LisPackage;
pub use dependencies::{detect_dependencies, find_resources, find_binary};
