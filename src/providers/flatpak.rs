use strum::AsRefStr;

use crate::{
    providers::Provider,
    yaml_by_key,
    yamlrepr::{self, YamlRepr},
};

pub struct FlatpakProvider {
    default_scope: InstallScope,
    pub desired_present_items: Vec<Flatpak>,
}

impl FlatpakProvider {
    pub fn new() -> Self {
        FlatpakProvider {
            default_scope: InstallScope::default(),
            desired_present_items: Vec::new(),
        }
    }
}

impl Provider for FlatpakProvider {
    type Item = Flatpak;

    fn parse_yaml(&mut self, y: &yaml_rust2::Yaml) -> Result<(), crate::Error> {
        if !y.is_hash() {
            return Err(yamlrepr::Error::WrongType.into());
        } else {
            Flatpak::ensure_required_keys(y)?;
        }

        if let Some(default_scope) = y[YamlKey::DefaultScope.as_ref()].as_str() {
            self.default_scope = InstallScope::try_from(default_scope)?;
        }

        if let Some(entries) = y[yamlrepr::CommonKeys::ItemPresent.as_ref()].as_vec() {
            for entry in entries {
                let f = Flatpak::from_yaml(&entry).map(|mut f| {
                    if !f.scope_override {
                        f.scope = self.default_scope;
                    }
                    f
                })?;

                self.desired_present_items.push(f);
            }
        } else {
            return Err(yamlrepr::Error::WrongType.into());
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, AsRefStr)]
pub enum YamlKey {
    #[strum(serialize = "default_scope")]
    DefaultScope,

    #[strum(serialize = "scope")]
    Scope,

    #[strum(serialize = "id")]
    Id,
}

#[derive(Debug)]
pub enum Error {
    InvalidScope(String),
    InvalidIdTriple(String),
}

#[derive(Clone, Copy, Debug, Default)]
enum InstallScope {
    #[default]
    User,
    System,
}

impl TryFrom<&str> for InstallScope {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "system" => Ok(Self::System),
            "user" => Ok(Self::User),
            _ => Err(Self::Error::InvalidIdTriple(value.to_string()).into()),
        }
    }
}

#[derive(Debug)]
pub struct Flatpak {
    id: String,
    scope: InstallScope,
    scope_override: bool,
}

impl YamlRepr for Flatpak {
    type Key = YamlKey;

    const YAML_KEY: &'static str = "flatpak";
    const REQUIRED_KEYS: &'static [Self::Key] = &[Self::Key::DefaultScope];
    const OPTIONAL_KEYS: &'static [Self::Key] = &[];

    fn from_yaml(y: &yaml_rust2::Yaml) -> Result<Self, crate::Error> {
        if let yaml_rust2::Yaml::Hash(_) = y {
            let id = yaml_by_key!(y, Self::Key::Id, str)?;
            let scope = yaml_by_key!(y, Self::Key::Scope, str).unwrap();

            Ok(Flatpak {
                id: id.to_string(),
                scope: InstallScope::try_from(scope).unwrap_or_default(),
                scope_override: true,
            })
        } else if let yaml_rust2::Yaml::String(value) = y {
            Ok(Flatpak {
                id: value.clone(),
                scope: InstallScope::default(),
                scope_override: false,
            })
        } else {
            Err(yamlrepr::Error::WrongType.into())
        }
    }
}
