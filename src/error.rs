use crate::{providers::flatpak, yamlrepr};

#[derive(Debug)]
pub enum Error {
    Yaml(yamlrepr::Error),

    // Providers
    Flatpak(flatpak::Error),
}

impl From<yamlrepr::Error> for Error {
    fn from(e: yamlrepr::Error) -> Self {
        Error::Yaml(e)
    }
}

impl From<flatpak::Error> for Error {
    fn from(e: flatpak::Error) -> Self {
        Error::Flatpak(e)
    }
}
