use strum::Display;

pub type Result<T> = std::result::Result<T, self::Error>;

#[derive(Debug, Display)]
pub enum Error {
    Io(std::io::Error),
    Reqwest(reqwest::Error),
}

impl std::error::Error for self::Error {}

impl From<std::io::Error> for self::Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<reqwest::Error> for self::Error {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e)
    }
}
