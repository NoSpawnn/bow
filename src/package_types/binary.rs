use std::{fmt, io::Write, os::unix::fs::PermissionsExt, path::PathBuf};

use serde::{
    Deserialize, Serialize,
    de::{self, Visitor},
};

use crate::package_types::PackageProvider;

// We kind of have to use a sidecar-esque file here,
// otherwise it's impossible to get e.g. the install URL of present binaries
// this is also probably a bit hacky rn
const INSTALLED_BINARIES_INFO_FILE: &'static str =
    concat!(env!("HOME"), "/.local/share/bow-binaries.yaml");

#[derive(Debug)]
pub struct BinaryProvider {
    install_folder: PathBuf,
    pub binaries: Vec<Binary>,
}

impl BinaryProvider {
    fn new(install_path: &str, binaries: Vec<Binary>) -> Self {
        // TODO: this is really hacky, I need to move it into the deserialization step

        Self {
            install_folder: PathBuf::from(install_path),
            binaries,
        }
    }

    fn new_with_binaries(binaries: Vec<Binary>) -> Self {
        Self {
            install_folder: PathBuf::new(),
            binaries,
        }
    }
}

impl PackageProvider for BinaryProvider {
    type Item = Binary;

    const LOG_PREFIX: &'static str = "binary";

    fn install_items(&self, items: &[Self::Item]) -> crate::Result<()> {
        let tmp_info_filepath = PathBuf::from([INSTALLED_BINARIES_INFO_FILE, ".tmp"].join(""));
        let mut info_file = std::fs::File::create(&tmp_info_filepath)?;

        for binary in items {
            let tmp_dir = tempfile::Builder::new().prefix("bowbinary-").tempdir()?;
            let response = reqwest::blocking::get(&binary.url)?;

            if let Err(e) = response.error_for_status_ref() {
                return Err(e.into());
            }

            let install_path = &binary
                .install_path
                .clone()
                .unwrap_or(self.install_folder.join(&binary.name));
            let tmp_file = tmp_dir.path().join(&binary.name);
            Self::log_msg(&format!(
                "Downloading {} to {}",
                &binary.name,
                tmp_file.display()
            ));

            let mut dest = std::fs::File::create(&tmp_file)?;
            let content = response.bytes()?;
            dest.write_all(&content)?;

            Self::log_msg(&format!("Succesfully downloaded {}", &binary.name));

            Self::log_msg(&format!(
                "Copying {} to {}",
                tmp_file.display(),
                install_path.display()
            ));
            let dest = binary
                .install_path
                .clone()
                .unwrap_or(self.install_folder.join(&binary.name));
            std::fs::copy(tmp_file, &dest)?;

            Self::log_msg(&format!("Succesfully copied {}", &binary.name));
            Self::log_msg(&format!("Setting {} to be executable", &binary.name));

            let mut perms = std::fs::metadata(&dest)?.permissions();
            perms.set_mode(755);
            std::fs::set_permissions(&dest, perms)?;

            Self::log_msg(&format!("Successfully installed {}", &binary.name));

            let info = serde_yaml_bw::to_string(&binary).or_else(|e| {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("failed to write info yaml for {}: {e}", &binary.name),
                ))
            })?;
            info_file.write_all(&info.as_bytes())?;
        }

        std::fs::copy(
            &tmp_info_filepath,
            PathBuf::from(INSTALLED_BINARIES_INFO_FILE),
        )?;
        std::fs::remove_file(&tmp_info_filepath)?;
        Self::log_msg(&format!(
            "Wrote installed binary info to {}",
            INSTALLED_BINARIES_INFO_FILE
        ));

        Self::log_msg(&format!("Successfully installed {} binaries", items.len()));

        Ok(())
    }

    fn remove_items(&self, items: &[Self::Item]) -> crate::Result<()> {
        todo!()
    }

    fn ensure(&self) -> crate::Result<()> {
        todo!()
    }

    fn get_installed(&self) -> crate::Result<Vec<Self::Item>> {
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
        // TODO: fix duplicated code
        let home_dir = std::env::home_dir()
            .ok_or_else(|| de::Error::custom("Failed to retrieve user home directory"))?
            .into_os_string()
            .into_string()
            .unwrap();
        let install_folder = f.install_folder.replace("$HOME", &home_dir);

        Ok(Self::new(&install_folder, f.binaries))
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize)]
pub struct Binary {
    name: String,
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sum: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    install_path: Option<PathBuf>,
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
            #[serde(rename = "install_path")]
            InstallPath,
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
                let mut install_path: Option<String> = None;

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
                                url = Some(map.next_value()?);
                            }
                        }
                        Field::Version => {
                            if version.is_some() {
                                return Err(de::Error::duplicate_field("version"));
                            } else {
                                version = Some(map.next_value()?);
                            }
                        }
                        Field::Sum => {
                            if sum.is_some() {
                                return Err(de::Error::duplicate_field("sum"));
                            } else {
                                sum = Some(map.next_value()?);
                            }
                        }
                        Field::InstallPath => {
                            if install_path.is_some() {
                                return Err(de::Error::duplicate_field("install_path"));
                            } else {
                                install_path = Some(map.next_value()?);
                            }
                        }
                    }
                }

                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                let mut url = url.ok_or_else(|| de::Error::missing_field("url"))?;
                let install_path = {
                    if let Some(install_path) = install_path {
                        if install_path.contains("$HOME") {
                            let home_dir = std::env::home_dir()
                                .ok_or_else(|| {
                                    de::Error::custom("Failed to retrieve user home directory")
                                })?
                                .into_os_string()
                                .into_string()
                                .unwrap();
                            let install_path = install_path.replace("$HOME", &home_dir);
                            Some(PathBuf::from(install_path))
                        } else {
                            Some(PathBuf::from(install_path))
                        }
                    } else {
                        None
                    }
                };

                const VERSION_REPLACEMENT_STR: &str = "{{ version }}";
                if url.contains(VERSION_REPLACEMENT_STR) {
                    let version = version
                        .as_ref()
                        .ok_or_else(|| de::Error::missing_field("version"))?;
                    url = url.replace(VERSION_REPLACEMENT_STR, &version);
                    sum = sum.map(|s| {
                        if s.contains(VERSION_REPLACEMENT_STR) {
                            s.replace(VERSION_REPLACEMENT_STR, &version)
                        } else {
                            BinaryProvider::log_err(&format!(
                                "WARN: You're using {VERSION_REPLACEMENT_STR} in the URL for {name}, but not for its checksum"
                            ));
                            s
                        }
                    });
                }

                // TODO: we should probably save the actual sum instead of a url pointing to it
                Ok(Binary {
                    name,
                    url,
                    sum,
                    install_path,
                    version,
                })
            }
        }

        const FIELDS: &[&str] = &["name", "url", "version", "sum"];
        deserializer.deserialize_struct("Binary", FIELDS, BinaryVisitor)
    }
}
