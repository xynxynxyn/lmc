use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Could not parse entire formula '{0}'")]
    Incomplete(String),
    #[error("Error while parsing formula: '{0}'")]
    Parsing(String),
}
