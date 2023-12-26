use std::{ffi::OsString, fs::File, io, path::Path};

use inox2d::{formats::inp::ParseInpError, model::Model, puppet::Puppet};
use thiserror::Error;

#[derive(Debug)]
pub struct InternalPuppet {
    folder: OsString,
    kind: PuppetKind,
}

#[derive(Debug)]
pub enum PuppetKind {
    Inochi2D(Puppet),
    Static,
}

#[derive(Debug, Default)]
pub struct Models {
    loaded_models: Vec<InternalPuppet>,
}

#[derive(Error, Debug)]
pub enum LoadError<T> {
    #[error("could not read file")]
    FailedToRead(#[from] io::Error),
    #[error("could not parse file data")]
    InvalidFile(#[source] T),
}

pub fn load_i2d_puppet_from_file<P: AsRef<Path>>(
    path: P,
) -> Result<(OsString, Model), LoadError<ParseInpError>> {
    let path = path.as_ref();

    let model = inox2d::formats::inp::parse_inp(File::open(path)?)
        .map_err(|e| LoadError::InvalidFile(e))?;

    let file_name = path
        .file_name()
        .expect("should have been a file")
        .to_owned();

    Ok((file_name, model))
}
