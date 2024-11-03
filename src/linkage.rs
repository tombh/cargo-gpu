//! A shader source and entry point that can be used to create renderling
//! shader linkage.
use super::ShaderLang;
use quote::quote;

pub struct Linkage {
    pub source_path: std::path::PathBuf,
    pub entry_point: String,
}

impl Linkage {
    pub fn fn_name(&self) -> &str {
        self.entry_point.split("::").last().unwrap()
    }

    pub fn to_string(&self, lang: ShaderLang) -> String {
        let original_source_path = match lang {
            ShaderLang::Spv => self.source_path.clone(),
            ShaderLang::Wgsl => self.source_path.with_extension("wgsl"),
        };

        let source_path = original_source_path.file_name().unwrap().to_str().unwrap();
        let entry_point = self.entry_point.clone();

        let wgsl_entry_point = entry_point.replace("::", "");
        let entry_point = match lang {
            ShaderLang::Spv => entry_point,
            ShaderLang::Wgsl => entry_point.replace("::", ""),
        };

        let entry_point_quote = quote! {
            #[cfg(not(target_arch = "wasm32"))]
            pub const ENTRY_POINT: &str = #entry_point;
            #[cfg(target_arch = "wasm32")]
            pub const ENTRY_POINT: &str = #wgsl_entry_point;
        };
        let bytes_quote = quote! {
            pub const BYTES: &'static [u8] = include_bytes!(#source_path);
        };
        format!(
            r#"#![allow(dead_code)]
            //! Automatically generated with `cargo-gpu`.
            //!
            //! Provides the shader linkage for `{entry_point}`.
            //!
            //! **source path**: `{original_source_path}`

            /// Shader entry point.
            {entry_point_quote}

            /// Shader bytes.
            {bytes_quote}
            "#,
            original_source_path = original_source_path.display()
        )
    }
}
