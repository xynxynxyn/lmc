use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Could not parse formula from '{0}', more information needed")]
    Incomplete(String),
    #[error("Could not parse entire formula '{0}', excess: '{1}'")]
    Leftover(String, String),
    #[error("Error while parsing formula: '{0}'")]
    Parsing(String),
}
