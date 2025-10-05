use serde::Deserialize;

#[derive(Debug)]
pub struct FlatpakProvider {
    flatpaks: Vec<Flatpak>,
}

impl FlatpakProvider {
    fn new_with_flatpaks(flatpaks: Vec<Flatpak>) -> Self {
        Self { flatpaks }
    }
}

impl<'de> Deserialize<'de> for FlatpakProvider {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let flatpaks = Vec::<Flatpak>::deserialize(deserializer)?;

        Ok(Self::new_with_flatpaks(flatpaks))
    }
}

#[derive(Debug)]
struct Flatpak {
    id: String,
}

impl Flatpak {
    fn new(id: &str) -> Self {
        Self { id: id.to_string() }
    }
}

impl<'de> Deserialize<'de> for Flatpak {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let id = String::deserialize(deserializer)?;

        Ok(Self::new(&id))
    }
}
