mod debug;
mod errors;
mod impls;

use crate::errors::CmdError;
use crossbeam::channel::Receiver;
use std::ffi::OsString;
use std::process::{Output, Stdio};
use std::time::Duration;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("cmd error")]
    CmdError(#[from] CmdError),

    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct ProcessResult {
    pub(crate) output: Box<Output>,
}

#[derive(Debug)]
pub struct Cmd {
    pub(crate) debug: bool,
    pub(crate) program: OsString,
    pub(crate) args: Vec<OsString>,
    pub(crate) stdin: Option<Stdio>,
    pub(crate) stdout: Option<Stdio>,
    pub(crate) stderr: Option<Stdio>,
    pub(crate) timeout: Option<Duration>,
    pub(crate) signal: Option<Receiver<()>>,
}

pub struct CommandBuilder {
    pub(crate) debug: bool,
    pub(crate) program: OsString,
    pub(crate) args: Vec<OsString>,
    pub(crate) stdin: Option<Stdio>,
    pub(crate) stdout: Option<Stdio>,
    pub(crate) stderr: Option<Stdio>,
    pub(crate) timeout: Option<Duration>,
    pub(crate) signal: Option<Receiver<()>>,
}

pub(crate) trait OutputResult {
    fn to_result(&self) -> Result<Vec<u8>>;
    fn try_to_result(&self) -> Result<Vec<u8>>;
}

pub(crate) trait Vec8ToString {
    fn as_str(&self) -> Option<&str>;
}
