use std::io::{BufRead, BufReader};

use serde::Deserialize;

use crate::package_types::PackageProvider;

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

impl PackageProvider for FlatpakProvider {
    type Item = Flatpak;
    const LOG_PREFIX: &'static str = "flatpak";

    fn install_items(&self) -> std::io::Result<()> {
        let ids: Vec<&str> = self.flatpaks.iter().map(|f| f.id.as_str()).collect();
        let mut cmd = std::process::Command::new("flatpak");
        cmd.args(["install", "--noninteractive", "--user"]);
        cmd.args(&ids);
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn()?;
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let stdout_handle = std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(line) => Self::log_msg(&line),
                    Err(e) => Self::log_err(e),
                }
            }
        });

        let stderr_handle = std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                match line {
                    Ok(line) => Self::log_msg(&line),
                    Err(e) => Self::log_err(e),
                }
            }
        });

        let _ = child.wait()?;
        stdout_handle.join().unwrap();
        stderr_handle.join().unwrap();

        Ok(())
    }
}

#[derive(Debug)]
pub struct Flatpak {
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
