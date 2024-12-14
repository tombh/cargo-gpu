use crate::cache_dir;

const SPIRV_STD_TOOLCHAIN_PAIRS: &[(&str, &str)] = &[("0.10", "nightly-2024-04-24")];

/// Cargo dependency for `spirv-builder` and the rust toolchain channel.
#[derive(Debug, Clone)]
pub(crate) struct Spirv {
    pub dep: String,
    pub channel: String,
}

impl Default for Spirv {
    fn default() -> Self {
        Self {
            dep: Self::DEFAULT_DEP.into(),
            channel: Self::DEFAULT_CHANNEL.into(),
        }
    }
}

impl core::fmt::Display for Spirv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format!("{}+{}", self.dep, self.channel).fmt(f)
    }
}

impl Spirv {
    pub const DEFAULT_DEP: &str = r#"{ git = "https://github.com/Rust-GPU/rust-gpu.git" }"#;
    pub const DEFAULT_CHANNEL: &str = "nightly-2024-04-24";

    /// Returns a string suitable to use as a directory.
    ///
    /// Created from the spirv-builder source dep and the rustc channel.
    fn to_dirname(&self) -> String {
        self.to_string()
            .replace(
                [std::path::MAIN_SEPARATOR, '\\', '/', '.', ':', '@', '='],
                "_",
            )
            .split(['{', '}', ' ', '\n', '"', '\''])
            .collect::<Vec<_>>()
            .concat()
    }

    pub fn cached_checkout_path(&self) -> std::path::PathBuf {
        let checkout_dir = cache_dir().join(self.to_dirname());
        std::fs::create_dir_all(&checkout_dir).unwrap_or_else(|e| {
            log::error!(
                "could not create checkout dir '{}': {e}",
                checkout_dir.display()
            );
            panic!("could not create checkout dir");
        });

        checkout_dir
    }

    pub fn ensure_version_channel_compatibility(&self) {
        for (version, channel) in SPIRV_STD_TOOLCHAIN_PAIRS.iter() {
            if version.starts_with(&self.dep) && channel != &self.channel {
                panic!("expected spirv-std version to be matched with rust toolchain channel {channel}");
            }
        }
    }

    /// Use `rustup` to install the toolchain and components, if not already installed.
    ///
    /// Pretty much runs:
    ///
    /// * rustup toolchain add nightly-2024-04-24
    /// * rustup component add --toolchain nightly-2024-04-24 rust-src rustc-dev llvm-tools
    pub fn ensure_toolchain_and_components_exist(&self) {
        // Check for the required toolchain
        let output = std::process::Command::new("rustup")
            .args(["toolchain", "list"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "could not list installed toolchains"
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout
            .split_whitespace()
            .any(|toolchain| toolchain.starts_with(&self.channel))
        {
            log::debug!("toolchain {} is already installed", self.channel);
        } else {
            let output = std::process::Command::new("rustup")
                .args(["toolchain", "add"])
                .arg(&self.channel)
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "could not install required toolchain"
            );
        }

        // Check for the required components
        let output = std::process::Command::new("rustup")
            .args(["component", "list", "--toolchain"])
            .arg(&self.channel)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "could not list installed components"
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        let required_components = ["rust-src", "rustc-dev", "llvm-tools"];
        let installed_components = stdout.lines().collect::<Vec<_>>();
        let all_components_installed = required_components.iter().all(|component| {
            installed_components.iter().any(|installed_component| {
                let is_component = installed_component.starts_with(component);
                let is_installed = installed_component.ends_with("(installed)");
                is_component && is_installed
            })
        });
        if all_components_installed {
            log::debug!("all required components are installed");
        } else {
            let output = std::process::Command::new("rustup")
                .args(["component", "add", "--toolchain"])
                .arg(&self.channel)
                .args(["rust-src", "rustc-dev", "llvm-tools"])
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "could not install required components"
            );
        }
    }
}
