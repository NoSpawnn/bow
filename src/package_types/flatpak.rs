use std::io::{BufRead, BufReader};

use serde::Deserialize;

use crate::package_types::PackageProvider;

#[derive(Debug)]
pub struct FlatpakProvider {
    pub flatpaks: Vec<Flatpak>,
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

    fn install_items(&self, items: &[Self::Item]) -> std::io::Result<()> {
        let ids: Vec<&str> = items.iter().map(|f| f.id.as_str()).collect();
        let mut cmd = std::process::Command::new("flatpak");
        cmd.args(["install", "--noninteractive", "--user"]);
        cmd.args(&ids);
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn()?;
        let stdout = child.stdout.take().expect("handle present");
        let stderr = child.stderr.take().expect("handle present");

        let stdout_handle = std::thread::spawn(|| {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(line) => Self::log_msg(&line),
                    Err(e) => Self::log_err(e),
                }
            }
        });

        let stderr_handle = std::thread::spawn(|| {
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

    fn remove_items(&self, items: &[Self::Item]) -> std::io::Result<()> {
        let ids: Vec<&str> = items.iter().map(|f| f.id.as_str()).collect();
        let mut cmd = std::process::Command::new("flatpak");
        cmd.args(["remove", "--noninteractive", "--user"]);
        cmd.args(&ids);
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn()?;
        let stdout = child.stdout.take().expect("handle present");
        let stderr = child.stderr.take().expect("handle present");

        let stdout_handle = std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(line) => Self::log_msg(&line),
                    Err(e) => Self::log_err(e),
                }
            }
        });

        let stderr_handle = std::thread::spawn(|| {
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

    fn ensure(&self) -> std::io::Result<()> {
        let installed = self.get_installed()?;

        if let Some(to_install) = Self::diff(&self.flatpaks, &installed) {
            Self::log_msg("Found packages to install");
            for f in to_install.iter() {
                Self::log_msg(&format!("    {}", f.id));
            }

            if Self::confirm("Install the above packages?")? {
                self.install_items(&to_install)?;
            } else {
                Self::log_msg(&format!(
                    "Skipping install of {} package(s)",
                    to_install.len()
                ));
            }
        } else {
            Self::log_msg("Nothing to install");
        }

        if let Some(to_remove) = Self::diff(&installed, &self.flatpaks) {
            Self::log_msg("Found packages to remove");
            for f in to_remove.iter() {
                Self::log_msg(&format!("    {}", f.id));
            }

            if Self::confirm("Remove the above packages?")? {
                self.remove_items(&to_remove)?;
            } else {
                Self::log_msg(&format!(
                    "Skipping removal of {} package(s)",
                    to_remove.len()
                ));
            }
        } else {
            Self::log_msg("Nothing to remove");
        }

        Ok(())
    }

    fn get_installed(&self) -> std::io::Result<Vec<Self::Item>> {
        let mut cmd = std::process::Command::new("flatpak");
        cmd.args(["list", "--user", "--columns=application:f", "--app"]);
        let output = cmd.output()?;

        let mut installed = Vec::new();
        for line in output.stdout.lines() {
            let line = line?;
            installed.push(Self::Item { id: line });
        }

        Ok(installed)
    }
}

#[derive(PartialEq, Eq, Debug, Hash, Clone)]
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
