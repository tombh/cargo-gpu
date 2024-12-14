//! Display various information about `cargo gpu`, eg its cache directory.

use crate::cache_dir;

/// `cargo gpu show`
#[derive(clap::Parser)]
pub struct Show {
    #[clap(long)]
    /// Displays the location of the cache directory
    cache_directory: bool,
}

impl Show {
    /// Entrypoint
    pub fn run(self) {
        if self.cache_directory {
            log::info!("cache_directory: ");
            println!("{}", cache_dir().display());
        }
    }
}
