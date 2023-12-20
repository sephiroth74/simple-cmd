use std::fmt::{Debug, Display, Formatter};
use std::process::ExitStatus;

use thiserror::Error;

use crate::Vec8ToString;

#[derive(Error, Clone, PartialEq, Eq, Debug)]
pub struct CmdError {
	pub status: Option<ExitStatus>,
	pub stdout: Vec<u8>,
	pub stderr: Vec<u8>,
}

impl CmdError {
	pub fn from_err(status: ExitStatus, stdout: Vec<u8>, stderr: Vec<u8>) -> Self {
		CmdError { status: Some(status), stdout, stderr }
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
		write!(f, "code:{:?}, stdout:{:?}, stderr:{:?}", self.status, self.stdout.as_str(), self.stderr.as_str())
	}
}
