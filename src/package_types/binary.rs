use serde::Deserialize;

#[derive(Debug)]
pub struct BinaryProvider {
    binaries: Vec<Binary>,
}

impl BinaryProvider {
    fn new_with_binaries(binaries: Vec<Binary>) -> Self {
        Self { binaries }
    }
}

impl<'de> Deserialize<'de> for BinaryProvider {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let binaries = Vec::deserialize(deserializer)?;

        Ok(Self::new_with_binaries(binaries))
    }
}

#[derive(Debug, Deserialize)]
struct Binary {
    name: String,
    url: String,
}
