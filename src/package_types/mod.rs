use serde::Deserialize;

pub mod binary;
pub mod flatpak;

pub use binary::BinaryProvider;
pub use flatpak::FlatpakProvider;

pub trait PackageProvider {
    type Item;
    const LOG_PREFIX: &'static str;

    fn install_items(&self) -> std::io::Result<()>;

    fn log_msg(msg: &str) {
        println!("[{}] {}", Self::LOG_PREFIX, msg);
    }

    fn log_err(msg: impl std::error::Error) {
        eprintln!("[{}] {}", Self::LOG_PREFIX, msg)
    }
}

#[derive(Debug, Deserialize)]
pub struct PackagesConfig {
    #[serde(rename = "binary")]
    binaries: Option<BinaryProvider>,
    #[serde(rename = "flatpak")]
    flatpaks: Option<FlatpakProvider>,
}

impl PackagesConfig {
    pub fn install_all(&self) -> std::io::Result<()> {
        if let Some(binaries) = &self.binaries {}

        if let Some(flatpaks) = &self.flatpaks {
            flatpaks.install_items()?;
        }

        Ok(())
    }
}
