mod package_types;

use serde::Deserialize;

use std::fs;

use crate::package_types::PackagesConfig;

#[derive(Debug, Deserialize)]
struct Config {
    packages: Option<PackagesConfig>,
}

fn main() {
    let f = fs::read_to_string("./bow.yaml").unwrap();
    let c: Result<Config, _> = serde_saphyr::from_str(&f);
    c.unwrap().packages.unwrap().install_all();
}
