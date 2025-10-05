use serde::Deserialize;

#[derive(Debug)]
pub struct FlatpakProvider {
    flatpaks: Vec<Flatpak>,
}

impl<'de> Deserialize<'de> for FlatpakProvider {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if let Ok(entries) = Vec::<String>::deserialize(deserializer) {
            Ok(Self {
                flatpaks: entries.iter().map(|id| Flatpak::new(id)).collect(),
            })
        } else {
            todo!()
        }
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
