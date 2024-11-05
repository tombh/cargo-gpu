//! A shader source and entry point that can be used to create shader linkage.

#[derive(serde::Serialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Linkage {
    pub source_path: std::path::PathBuf,
    pub entry_point: String,
    pub wgsl_entry_point: String,
}

impl Linkage {
    pub fn new(entry_point: impl AsRef<str>, source_path: impl AsRef<std::path::Path>) -> Self {
        Self {
            source_path: source_path.as_ref().to_path_buf(),
            wgsl_entry_point: entry_point.as_ref().replace("::", ""),
            entry_point: entry_point.as_ref().to_string(),
        }
    }

    pub fn fn_name(&self) -> &str {
        self.entry_point.split("::").last().unwrap()
    }
}
