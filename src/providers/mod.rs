pub mod flatpak;

pub trait Provider {
    type Item;

    fn parse_yaml(&mut self, y: &yaml_rust2::Yaml) -> Result<(), crate::Error>;
}
