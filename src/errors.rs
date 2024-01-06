use std::fmt::{Debug, Display, Formatter};
use std::process::{ExitStatus, Output};

use thiserror::Error;

use crate::Vec8ToString;

#[derive(Error, Clone, PartialEq, Eq, Debug)]
pub struct CmdError {
	pub status: Option<ExitStatus>,
	pub stdout: Vec<u8>,
	pub stderr: Vec<u8>,
}

impl From<Output> for CmdError {
	fn from(value: Output) -> Self {
		CmdError::from_err(value.status, value.stdout, value.stderr)
	}
}

impl From<Output> for crate::Error {
	fn from(value: Output) -> Self {
		crate::Error::CommandError(value.into())
	}
}

impl CmdError {
	pub fn from_err(status: ExitStatus, stdout: Vec<u8>, stderr: Vec<u8>) -> Self {
		CmdError {
			status: Some(status),
			stdout,
			stderr,
		}
	}

	pub fn from_status(status: ExitStatus) -> Self {
		CmdError {
			status: Some(status),
			stdout: vec![],
			stderr: vec![],
		}
	}

	pub fn from_str(msg: &str) -> Self {
		CmdError {
			status: None,
			stdout: vec![],
			stderr: msg.to_owned().into_bytes(),
		}
	}

	pub fn exit_code(&self) -> Option<i32> {
		match self.status {
			Some(s) => s.code(),
			None => None,
		}
	}
}

impl Display for CmdError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		if let Some(status) = self.status {
			if let Some(code) = status.code() {
				let _ = write!(f, "exit code: {}", code);
			} else {
				let _ = write!(f, "exit status: {}", self.status.unwrap_or(ExitStatus::default()));
			}
		} else {
			let _ = write!(f, "exit status: {}", self.status.unwrap_or(ExitStatus::default()));
		}

		if !self.stderr.is_empty() {
			write!(f, ", stderr: {}", self.stderr.as_str().unwrap_or(""))
		} else {
			write!(f, ", stdout: {}", self.stdout.as_str().unwrap_or(""))
		}
	}
}
