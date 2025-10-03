use strum::{AsRefStr, EnumString};

#[derive(Debug)]
pub enum Error {
    UnknownKey(String),
    MissingKey(String),
    MissingKeys(Vec<String>),
    WrongType,
}

pub trait YamlRepr: Sized
where
    Self::Key: Copy + AsRef<str> + 'static,
{
    type Key;

    const YAML_KEY: &'static str;
    const REQUIRED_KEYS: &'static [Self::Key];
    const OPTIONAL_KEYS: &'static [Self::Key];

    fn from_yaml(y: &yaml_rust2::Yaml) -> Result<Self, crate::Error>;

    fn ensure_required_keys(y: &yaml_rust2::Yaml) -> Result<(), crate::Error> {
        let missing_keys: Vec<_> = Self::REQUIRED_KEYS
            .iter()
            .filter_map(|key| {
                let key = key.as_ref();
                if y[key].is_badvalue() {
                    Some(key.to_string())
                } else {
                    None
                }
            })
            .collect();

        if missing_keys.is_empty() {
            Ok(())
        } else {
            Err(Error::MissingKeys(missing_keys).into())
        }
    }

    fn get_present_optional_keys(y: &yaml_rust2::Yaml) -> Option<Vec<String>> {
        let optional_keys: Vec<String> = Self::OPTIONAL_KEYS
            .iter()
            .filter_map(|key| {
                let key = key.as_ref();
                if y[key].is_badvalue() {
                    None
                } else {
                    Some(key.to_string())
                }
            })
            .collect();

        if optional_keys.is_empty() {
            None
        } else {
            Some(optional_keys)
        }
    }
}

#[derive(Debug, EnumString, AsRefStr)]
pub enum CommonKeys {
    #[strum(serialize = "present")]
    ItemPresent,

    #[strum(serialize = "absent")]
    ItemAbsent,
}

#[macro_export]
macro_rules! yaml_by_key {
    ($yaml:expr, $key:expr, str) => {{
        let key = $key.as_ref();
        let v = &$yaml[key];
        if v.is_badvalue() {
            return Err(crate::Error::Yaml(yamlrepr::Error::MissingKey(
                key.to_string(),
            )));
        }
        v.as_str()
            .ok_or(crate::Error::Yaml(yamlrepr::Error::WrongType))
    }};
    ($yaml:expr, $key:expr, vec) => {{
        let key = $key.as_ref();
        let v = &$yaml[key];
        if v.is_badvalue() {
            return Err(crate::Error::Yaml(yamlrepr::Error::MissingKey(
                key.to_string(),
            )));
        }
        v.as_vec()
            .ok_or(crate::Error::Yaml(yamlrepr::Error::WrongType))
    }};
    ($yaml:expr, $key:expr, hash) => {{
        let key = $key.as_ref();
        let v = &$yaml[key];
        if v.is_badvalue() {
            return Err(crate::Error::Yaml(yamlrepr::Error::MissingKey(
                key.to_string(),
            )));
        }
        v.as_hash()
            .ok_or(crate::Error::Yaml(yamlrepr::Error::WrongType))
    }};
}
