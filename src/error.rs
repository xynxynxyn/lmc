use std::io;
use std::result;
use thiserror::Error;

pub type Result<T> = result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unknown transition with label '{0}'")]
    UnknownTransition(String),
    #[error("unknown place with label '{0}'")]
    UnknownPlace(String),
    #[error("duplicate place with label '{0}'")]
    DuplicatePlace(String),
    #[error("duplicate transition with label '{0}'")]
    DuplicateTransition(String),
    #[error("cannot create arc from '{0}' to '{1}'")]
    InvalidArc(String, String),
    #[error("invalid index")]
    InvalidIndex,
    #[error("could not parse xml petri net")]
    XmlError(#[from] serde_xml_rs::Error),
    #[error("could not read file")]
    IOError(#[from] io::Error),
}
