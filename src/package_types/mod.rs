use std::{collections::HashSet, hash::Hash, io::Write};

use serde::Deserialize;

pub mod binary;
pub mod flatpak;

pub use binary::BinaryProvider;
pub use flatpak::FlatpakProvider;

use crate::RunMode;

pub trait PackageProvider {
    type Item: Eq + Hash + Clone;
    const LOG_PREFIX: &'static str;

    fn install_items(&self, items: &[Self::Item]) -> std::io::Result<()>;
    fn remove_items(&self, items: &[Self::Item]) -> std::io::Result<()>;
    fn ensure(&self) -> std::io::Result<()>;
    fn get_installed(&self) -> std::io::Result<Vec<Self::Item>>;

    fn diff(v1: &[Self::Item], v2: &[Self::Item]) -> Option<Vec<Self::Item>> {
        let s1: HashSet<_> = v1.iter().collect();
        let s2: HashSet<_> = v2.iter().collect();
        let diff: Vec<_> = s1.difference(&s2).map(|&entry| entry.to_owned()).collect();
        if diff.is_empty() { None } else { Some(diff) }
    }

    fn log_msg(msg: &str) {
        println!("[{}] {}", Self::LOG_PREFIX, msg);
    }

    fn log_err(err: impl std::error::Error) {
        eprintln!("[{}] {}", Self::LOG_PREFIX, err)
    }

    fn confirm(msg: &str) -> std::io::Result<bool> {
        loop {
            print!("[{}] {} [y/N]: ", Self::LOG_PREFIX, msg);
            std::io::stdout().flush()?;

            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_err() {
                continue;
            }

            match input.trim().to_lowercase().as_str() {
                "y" => return Ok(true),
                "n" | "" => return Ok(false),
                _ => {}
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PackagesConfig {
    #[serde(rename = "binary")]
    binaries: Option<BinaryProvider>,
    #[serde(rename = "flatpak")]
    flatpaks: Option<FlatpakProvider>,
}

impl PackagesConfig {
    pub fn install(&self, mode: RunMode) -> std::io::Result<()> {
        if let Some(binaries) = &self.binaries {
            match mode {
                RunMode::Idempotent => todo!(),
                RunMode::Imperative => todo!(),
            }
        }

        if let Some(flatpaks) = &self.flatpaks {
            match mode {
                RunMode::Idempotent => flatpaks.ensure()?,
                RunMode::Imperative => flatpaks.install_items(&flatpaks.flatpaks)?,
            }
        }

        Ok(())
    }
}
