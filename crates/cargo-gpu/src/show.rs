use crate::cache_dir;

#[derive(clap::Parser)]
pub(crate) struct Show {
    #[clap(long)]
    /// Displays the location of the cache directory
    cache_directory: bool,
}

impl Show {
    pub fn run(self) {
        if self.cache_directory {
            log::info!("cache_directory: ");
            println!("{}", cache_dir().display());
        }
    }
}
