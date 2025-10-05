use std::{
    fmt,
    io::{BufRead, Write},
    path::{Path, PathBuf},
};

use serde::{
    Deserialize, Serialize,
    de::{self, Visitor},
};

use crate::package_types::PackageProvider;

// We kind of have to use a sidecar-esque file here,
// otherwise it's impossible to get e.g. the install URL of present binaries
// this is also probably a bit hacky rn
const INSTALLED_BINARIES_INFO_FILE: &'static str =
    concat!(env!("HOME"), "/.local/bow-binaries.yaml");

#[derive(Debug)]
pub struct BinaryProvider {
    install_folder: String,
    pub binaries: Vec<Binary>,
}

impl BinaryProvider {
    fn new(install_path: &str, binaries: Vec<Binary>) -> Self {
        // TODO: this is really hacky, I need to move it into the deserialization step
        let mut real_install_path = None;
        if install_path.contains("$HOME") {
            real_install_path =
                Some(install_path.replace("$HOME", &std::env::var("HOME").unwrap()));
        }

        Self {
            install_folder: real_install_path.unwrap_or(String::from(install_path)),
            binaries,
        }
    }

    fn new_with_binaries(binaries: Vec<Binary>) -> Self {
        Self {
            install_folder: String::new(),
            binaries,
        }
    }
}

impl PackageProvider for BinaryProvider {
    type Item = Binary;

    const LOG_PREFIX: &'static str = "binary";

    fn install_items(&self, items: &[Self::Item]) -> std::io::Result<()> {
        let mut info_file = {
            let tmp_path = [INSTALLED_BINARIES_INFO_FILE, ".tmp"].join("");
            let p = PathBuf::from(tmp_path);
            std::fs::File::create(p)?
        };
        let info = serde_yaml_bw::to_string(&items).or_else(|e| {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("failed to write info yaml: {e}"),
            ));
        })?;

        // Download files to tmp dir, then move them to self.install_dir

        info_file.write_all(&info.as_bytes())?;

        todo!()
    }

    fn remove_items(&self, items: &[Self::Item]) -> std::io::Result<()> {
        todo!()
    }

    fn ensure(&self) -> std::io::Result<()> {
        todo!()
    }

    fn get_installed(&self) -> std::io::Result<Vec<Self::Item>> {
        let info_file = PathBuf::from(INSTALLED_BINARIES_INFO_FILE);

        if !info_file.exists() {
            Self::log_err(&format!(
                "Info file does not exist at {}, assuming first run. Creating...",
                info_file.display()
            ));

            match std::fs::File::create(&info_file) {
                Ok(_) => Self::log_msg(&format!("Created info file at {}", info_file.display())),
                Err(e) => {
                    Self::log_err(&format!("Failed to create info file: {e}"));
                    panic!()
                }
            }

            return Ok(Vec::new());
        }

        let info = std::fs::read_to_string(info_file)?;
        match serde_yaml_bw::from_str::<Vec<Binary>>(&info) {
            Ok(d) => Ok(d),
            Err(e) => {
                Self::log_err(e);
                panic!()
            }
        }
    }
}

impl<'de> Deserialize<'de> for BinaryProvider {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        pub struct Fields {
            install_folder: String,
            #[serde(rename = "packages")]
            binaries: Vec<Binary>,
        }

        let f = Fields::deserialize(deserializer)?;
        Ok(Self::new(&f.install_folder, f.binaries))
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize)]
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
