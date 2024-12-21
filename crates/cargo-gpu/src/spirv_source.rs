//! Use the shader that we're compiling as the default source for which version of `rust-gpu` to use.
//!
//! We do this by calling `cargo tree` inside the shader's crate to get the defined `spirv-std`
//! version. Then with that we `git checkout` the `rust-gpu` repo that corresponds to that version.
//! From there we can look at the source code to get the required Rust toolchain.

/// The canonical `rust-gpu` URI
const RUST_GPU_REPO: &str = "https://github.com/Rust-GPU/rust-gpu";

/// The various sources that the `rust-gpu` repo can have.
/// Most commonly it will simply be the canonical version on crates.io. But it could also be the
/// Git version, or a fork.
#[derive(Eq, PartialEq, Clone, Debug)]
pub enum SpirvSource {
    /// If the shader specifies a simple version like `spirv-std = "0.9.0"` then the source of
    /// `rust-gpu` is the conventional crates.io version.
    ///
    /// `String` is the simple version like, "0.9.0"
    CratesIO(String),
    /// If the shader specifies a version like:
    ///   `spirv-std = { git = "https://github.com..." ... }`
    /// then the source of `rust-gpu` is `Git`.
    ///
    /// `(String, String)` is the repo source and revision hash or tag.
    Git((String, String)),
    /// If the shader specifies a version like:
    ///   `spirv-std = { path = "/path/to/rust-gpu" ... }`
    /// then the source of `rust-gpu` is `Path`.
    ///
    /// `(String, String)` is the repo path and the version.
    Path((String, String)),
}

impl core::fmt::Display for SpirvSource {
    #[expect(
        clippy::min_ident_chars,
        reason = "It's a core library trait implementation"
    )]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        format!("{}+{}", self.to_repo(), self.to_version()).fmt(f)
    }
}

impl SpirvSource {
    /// Look into the shader crate to get the version of `rust-gpu` it's using.
    pub fn get_rust_gpu_deps_from_shader(
        shader_crate_path: &std::path::PathBuf,
    ) -> (Self, chrono::NaiveDate, String) {
        let rust_gpu_source = Self::get_spirv_std_dep_definition(shader_crate_path);

        rust_gpu_source.ensure_repo_is_installed();
        rust_gpu_source.checkout();

        let date = rust_gpu_source.get_version_date();
        let channel = Self::get_channel_from_toolchain_toml(&rust_gpu_source.to_dirname());

        log::debug!("Parsed version, date and toolchain channel from shader-defined `rust-gpu`: {rust_gpu_source:?}, {date}, {channel}");

        (rust_gpu_source, date, channel)
    }

    /// Convert the source to just its version.
    pub fn to_version(&self) -> String {
        match self {
            Self::CratesIO(version) | Self::Path((_, version)) => version.to_string(),
            Self::Git((_, revision)) => revision.to_string(),
        }
    }

    /// Convert the source to just its repo or path.
    fn to_repo(&self) -> String {
        match self {
            Self::CratesIO(_) => RUST_GPU_REPO.to_owned(),
            Self::Git((repo, _)) => repo.to_owned(),
            Self::Path((path, _)) => path.to_owned(),
        }
    }

    /// Convert the `rust-gpu` source into a string that can be used as a directory.
    /// It needs to be dynamically created because an end-user might want to swap out the source,
    /// maybe using their own fork for example.
    fn to_dirname(&self) -> std::path::PathBuf {
        let dir = crate::to_dirname(self.to_string().as_ref());
        crate::cache_dir().join("rust-gpu-repo").join(dir)
    }

    /// Checkout the `rust-gpu` to the requested version.
    fn checkout(&self) {
        log::debug!(
            "Checking out `rust-gpu` repo at {} to {}",
            self.to_dirname().display(),
            self.to_version()
        );
        let output_checkout = std::process::Command::new("git")
            .current_dir(self.to_dirname())
            .args(["checkout", self.to_version().as_ref()])
            .output()
            .unwrap();
        assert!(
            output_checkout.status.success(),
            "couldn't checkout revision '{}' of `rust-gpu` at {}",
            self.to_version(),
            self.to_dirname().to_string_lossy()
        );
    }

    /// Get the date of the version of `rust-gpu` used by the shader. This allows us to know what
    /// features we can use in the `spirv-builder` crate.
    fn get_version_date(&self) -> chrono::NaiveDate {
        let date_format = "%Y-%m-%d";

        log::debug!(
            "Getting `rust-gpu` version date from {}",
            self.to_dirname().display(),
        );
        let output_date = std::process::Command::new("git")
            .current_dir(self.to_dirname())
            .args([
                "show",
                "--no-patch",
                "--format=%cd",
                format!("--date=format:'{date_format}'").as_ref(),
                self.to_version().as_ref(),
            ])
            .output()
            .unwrap();
        assert!(
            output_date.status.success(),
            "couldn't get `rust-gpu` version date at for {} at {}",
            self.to_version(),
            self.to_dirname().to_string_lossy()
        );
        let date_string = String::from_utf8_lossy(&output_date.stdout)
            .to_string()
            .trim()
            .replace('\'', "");

        log::debug!(
            "Parsed date for version {}: {date_string}",
            self.to_version()
        );

        chrono::NaiveDate::parse_from_str(&date_string, date_format).unwrap()
    }

    /// Parse the `rust-toolchain.toml` in the working tree of the checked-out version of the `rust-gpu` repo.
    fn get_channel_from_toolchain_toml(path: &std::path::PathBuf) -> String {
        log::debug!("Parsing `rust-toolchain.toml` at {path:?} for the used toolchain");

        let contents = std::fs::read_to_string(path.join("rust-toolchain.toml")).unwrap();
        let toml: toml::Table = toml::from_str(&contents).unwrap();
        let Some(toolchain) = toml.get("toolchain") else {
            panic!("Couldn't find `[toolchain]` section in `rust-toolchain.toml` at {path:?}");
        };
        let Some(channel) = toolchain.get("channel") else {
            panic!("Couldn't find `channel` field in `rust-toolchain.toml` at {path:?}");
        };

        channel.to_string().replace('"', "")
    }

    /// Get the shader crate's `spirv_std = ...` definition in its `Cargo.toml`
    fn get_spirv_std_dep_definition(shader_crate_path: &std::path::PathBuf) -> Self {
        log::debug!("Running `cargo tree` on {}", shader_crate_path.display());
        let output_cargo_tree = std::process::Command::new("cargo")
            .current_dir(shader_crate_path)
            .args(["tree", "--workspace", "--depth", "1", "--prefix", "none"])
            .output()
            .unwrap();
        assert!(
            output_cargo_tree.status.success(),
            "could not query shader's `Cargo.toml` for `spirv-std` dependency"
        );
        let cargo_tree_string = String::from_utf8_lossy(&output_cargo_tree.stdout);

        let maybe_spirv_std_def = cargo_tree_string
            .lines()
            .find(|line| line.contains("spirv-std"));

        let Some(spirv_std_def) = maybe_spirv_std_def else {
            panic!("`spirv-std` not found in shader's `Cargo.toml` at {shader_crate_path:?}:\n{cargo_tree_string}");
        };

        Self::parse_spirv_std_source_and_version(spirv_std_def)
    }

    /// Parse a string like:
    ///   `spirv-std v0.9.0 (https://github.com/Rust-GPU/rust-gpu?rev=54f6978c#54f6978c) (*)`
    /// Which would return:
    ///   `SpirvSource::Git("https://github.com/Rust-GPU/rust-gpu", "54f6978c")`
    fn parse_spirv_std_source_and_version(spirv_std_def: &str) -> Self {
        let parts: Vec<String> = spirv_std_def.split_whitespace().map(String::from).collect();
        let version = parts
            .get(1)
            .expect("Couldn't find `spirv_std` version in shader crate")
            .to_owned();
        let mut source = Self::CratesIO(version.clone());

        if parts.len() > 2 {
            let mut source_string = parts.get(2).unwrap().to_owned();
            source_string = source_string.replace(['(', ')'], "");

            let uri = source_string.parse::<http::Uri>().unwrap();
            if uri.scheme().is_some() {
                source = Self::parse_git_source(version, &uri);
            } else {
                source = Self::Path((source_string, version));
            }
        }

        log::debug!("Parsed `rust-gpu` source and version: {source:?}");

        source
    }

    /// Parse a Git source like: `https://github.com/Rust-GPU/rust-gpu?rev=54f6978c#54f6978c`
    fn parse_git_source(version: String, uri: &http::Uri) -> Self {
        let mut revision = version;

        let repo = format!(
            "{}://{}{}",
            uri.scheme().unwrap(),
            uri.host().unwrap(),
            uri.path()
        );

        if let Some(query) = uri.query() {
            let marker = "rev=";
            let sanity_check = query.contains(marker) && query.split('=').count() == 2;
            assert!(sanity_check, "revision not found in Git URI: {query}");
            revision = query.replace(marker, "");
        }

        Self::Git((repo, revision))
    }

    /// `git clone` the `rust-gpu` repo. We use it to get the required Rust toolchain to compile
    /// the shader.
    fn ensure_repo_is_installed(&self) {
        if self.to_dirname().exists() {
            log::debug!(
                "Not cloning `rust-gpu` repo ({}) as it already exists at {}",
                self.to_repo(),
                self.to_dirname().to_string_lossy().as_ref(),
            );
            return;
        }

        log::debug!(
            "Cloning `rust-gpu` repo {} to {}",
            self.to_repo(),
            self.to_dirname().to_string_lossy().as_ref(),
        );

        let output_clone = std::process::Command::new("git")
            .args([
                "clone",
                self.to_repo().as_ref(),
                self.to_dirname().to_string_lossy().as_ref(),
            ])
            .output()
            .unwrap();

        assert!(
            output_clone.status.success(),
            "couldn't clone `rust-gpu` {} to {}\n{}",
            self.to_repo(),
            self.to_dirname().to_string_lossy(),
            String::from_utf8_lossy(&output_clone.stderr)
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test_log::test]
    fn parsing_spirv_std_dep_for_shader_template() {
        let shader_template_path = crate::test::shader_crate_template_path();
        let source = SpirvSource::get_spirv_std_dep_definition(&shader_template_path);
        assert_eq!(
            source,
            SpirvSource::Git((
                "https://github.com/Rust-GPU/rust-gpu".to_owned(),
                "82a0f69".to_owned()
            ))
        );
    }

    #[test_log::test]
    fn parsing_spirv_std_dep_for_git_source() {
        let definition =
            "spirv-std v9.9.9 (https://github.com/Rust-GPU/rust-gpu?rev=82a0f69#82a0f69) (*)";
        let source = SpirvSource::parse_spirv_std_source_and_version(definition);
        assert_eq!(
            source,
            SpirvSource::Git((
                "https://github.com/Rust-GPU/rust-gpu".to_owned(),
                "82a0f69".to_owned()
            ))
        );
    }
}
