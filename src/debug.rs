use std::ffi::OsStr;
use std::ffi::OsString;
use std::path::Path;

use log::log;

use crate::Cmd;

pub trait CommandDebug {
	fn debug(&mut self) -> &mut Self;
}

impl CommandDebug for std::process::Command {
	fn debug(&mut self) -> &mut Self {
		let path = Path::new(self.get_program());
		let s = self.get_args().fold(vec![], |mut a: Vec<&OsStr>, b: &OsStr| {
			a.push(b);
			a
		});
		log!(
			log::Level::Debug,
			"Executing `{} {}`...",
			path.file_name().unwrap().to_str().unwrap(),
			s.join(OsString::from(" ").as_os_str()).to_str().unwrap().trim()
		);
		self
	}
}

impl CommandDebug for Cmd {
	fn debug(&mut self) -> &mut Self {
		let s = (&self.args).into_iter().fold(Vec::new(), |mut a: Vec<OsString>, b: &OsString| {
			a.push(b.clone());
			a
		});
		log!(
			log::Level::Debug,
			"Executing `{} {}`...",
			self.program.to_str().unwrap(),
			s.join(OsString::from(" ").as_os_str()).to_str().unwrap().trim()
		);
		self
	}
}
