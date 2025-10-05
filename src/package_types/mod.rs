use serde::Deserialize;

pub mod binary;
pub mod flatpak;

pub use binary::BinaryProvider;
pub use flatpak::FlatpakProvider;

pub trait PackageProvider {
    type Item;

    fn install_items(items: Vec<Self::Item>) -> std::io::Result<()>;
}

#[derive(Debug, Deserialize)]
pub struct PackagesConfig {
    #[serde(rename = "binary")]
    binaries: Option<BinaryProvider>,
    #[serde(rename = "flatpak")]
    flatpaks: Option<FlatpakProvider>,
}
