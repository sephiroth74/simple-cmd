use std::ffi::OsStr;
use std::ffi::OsString;
use std::path::Path;

use tracing::trace;

use crate::Cmd;

pub trait CommandDebug {
	fn debug(&mut self) -> &mut Self;
	fn as_string(&self) -> String;
}

impl CommandDebug for std::process::Command {
	fn debug(&mut self) -> &mut Self {
		trace!("Executing `{}`...", self.as_string());
		self
	}

	fn as_string(&self) -> String {
		let path = Path::new(self.get_program());
		let s = self.get_args().fold(vec![], |mut a: Vec<&OsStr>, b: &OsStr| {
			a.push(b);
			a
		});
		format!(
			"{} {}",
			path.file_name().unwrap().to_str().unwrap(),
			s.join(OsString::from(" ").as_os_str()).to_str().unwrap().trim()
		)
	}
}

impl CommandDebug for Cmd {
	fn debug(&mut self) -> &mut Self {
		trace!("Executing `{}`...", self.as_string());
		self
	}

	fn as_string(&self) -> String {
		let path = Path::new(self.program.as_os_str());
		let s = (&self.args)
			.into_iter()
			.fold(Vec::new(), |mut a: Vec<OsString>, b: &OsString| {
				a.push(b.clone());
				a
			});
		format!(
			"{} {}",
			path.file_name().unwrap().to_str().unwrap(),
			s.join(OsString::from(" ").as_os_str()).to_str().unwrap().trim()
		)
	}
}
