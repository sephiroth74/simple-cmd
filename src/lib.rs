use std::ffi::OsString;
use std::process::Stdio;
use std::time::Duration;

use crossbeam::channel::Receiver;
use thiserror::Error;

use crate::errors::CmdError;

pub mod debug;
mod errors;
mod impls;
pub mod output_ext;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
	#[error("cmd error")]
	CmdError(#[from] CmdError),

	#[error(transparent)]
	IoError(#[from] std::io::Error),
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

#[derive(Debug)]
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

pub trait Vec8ToString {
	fn as_str(&self) -> Option<&str>;
}
