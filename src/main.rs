mod error;
mod providers;
mod yamlrepr;

pub use error::Error;

use std::fs;
pub use yamlrepr::CommonKeys;

use yaml_rust2::YamlLoader;

use crate::{
    providers::{Provider, flatpak},
    yamlrepr::YamlRepr,
};

fn main() {
    let f = fs::read_to_string("./packages.yaml").unwrap();
    let docs = YamlLoader::load_from_str(&f).unwrap();
    let doc = &docs[0];

    let mut fpb = flatpak::FlatpakProvider::new();
    let res = fpb.parse_yaml(&doc[flatpak::Flatpak::YAML_KEY]);

    dbg!(&fpb.desired_present_items);
}
