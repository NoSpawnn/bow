use std::fmt;

use serde::{
    Deserialize,
    de::{self, Visitor},
};

use crate::package_types::PackageProvider;

#[derive(Debug)]
pub struct BinaryProvider {
    install_path: String,
    pub binaries: Vec<Binary>,
}

impl BinaryProvider {
    fn new(install_path: &str, binaries: Vec<Binary>) -> Self {
        Self {
            install_path: String::from(install_path),
            binaries,
        }
    }

    fn new_with_binaries(binaries: Vec<Binary>) -> Self {
        Self {
            install_path: String::new(),
            binaries,
        }
    }
}

impl PackageProvider for BinaryProvider {
    type Item = Binary;

    const LOG_PREFIX: &'static str = "binary";

    fn install_items(&self, items: &[Self::Item]) -> std::io::Result<()> {
        todo!()
    }

    fn remove_items(&self, items: &[Self::Item]) -> std::io::Result<()> {
        todo!()
    }

    fn ensure(&self) -> std::io::Result<()> {
        todo!()
    }

    fn get_installed(&self) -> std::io::Result<Vec<Self::Item>> {
        todo!()
    }
}

impl<'de> Deserialize<'de> for BinaryProvider {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        pub struct Fields {
            install_path: String,
            #[serde(rename = "packages")]
            binaries: Vec<Binary>,
        }

        let f = Fields::deserialize(deserializer)?;
        Ok(Self::new(&f.install_path, f.binaries))
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Binary {
    name: String,
    url: String,
    sum: Option<String>,
}

impl<'de> Deserialize<'de> for Binary {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Name,
            Url,
            Version,
            Sum,
        }

        struct BinaryVisitor;

        impl<'de> Visitor<'de> for BinaryVisitor {
            type Value = Binary;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Binary")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut name: Option<String> = None;
                let mut url: Option<String> = None;
                let mut version: Option<String> = None;
                let mut sum: Option<String> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Name => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            } else {
                                name = Some(map.next_value()?);
                            }
                        }
                        Field::Url => {
                            if url.is_some() {
                                return Err(de::Error::duplicate_field("url"));
                            } else {
                                url = Some(map.next_value()?)
                            }
                        }
                        Field::Version => {
                            if version.is_some() {
                                return Err(de::Error::duplicate_field("version"));
                            } else {
                                version = Some(map.next_value()?)
                            }
                        }
                        Field::Sum => {
                            if sum.is_some() {
                                return Err(de::Error::duplicate_field("sum"));
                            } else {
                                sum = Some(map.next_value()?)
                            }
                        }
                    }
                }

                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                let mut url = url.ok_or_else(|| de::Error::missing_field("name"))?;

                const VERSION_REPLACEMENT_STR: &str = "{{ version }}";
                if url.contains(VERSION_REPLACEMENT_STR) {
                    let version = version.ok_or_else(|| de::Error::missing_field("version"))?;
                    url = url.replace(VERSION_REPLACEMENT_STR, &version);
                    sum = sum.map(|s| {
                        if s.contains(VERSION_REPLACEMENT_STR) {
                            s.replace(VERSION_REPLACEMENT_STR, &version)
                        } else {
                            BinaryProvider::log_err(&format!(
                                "WARN: You're using {VERSION_REPLACEMENT_STR} in the URL for {name}, but not in sum"
                            ));
                            s
                        }
                    });
                }

                Ok(Binary { name, url, sum })
            }
        }

        const FIELDS: &[&str] = &["name", "url", "version", "sum"];
        deserializer.deserialize_struct("Binary", FIELDS, BinaryVisitor)
    }
}
