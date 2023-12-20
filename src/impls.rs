use std::ffi::{OsStr, OsString};
use std::fmt::{Display, Formatter};
use std::io::{BufRead, BufReader};
use std::process::{ChildStderr, ChildStdout, Command, Output, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use crossbeam::channel::Receiver;
use log::{trace, warn};

use crate::debug::CommandDebug;
use crate::errors::CmdError;
use crate::{Cmd, CommandBuilder, Error, OutputResult, Vec8ToString};

impl Display for Cmd {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?} {:?}", self.program, self.args)
	}
}

impl Display for CommandBuilder {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?} {:?}", self.program, self.args)
	}
}

impl OutputResult for Output {
	fn to_result(&self) -> crate::Result<Vec<u8>> {
		if self.status.success() && self.stderr.is_empty() {
			Ok(self.stdout.to_owned())
		} else {
			Err(Error::CmdError(CmdError::from_err(self.status, self.stdout.to_owned(), self.stderr.to_owned())))
		}
	}

	fn try_to_result(&self) -> crate::Result<Vec<u8>> {
		if self.status.code().is_none() && self.stderr.is_empty() {
			Ok(self.stdout.to_owned())
		} else {
			Err(Error::CmdError(CmdError::from_err(self.status, self.stdout.to_owned(), self.stderr.to_owned())))
		}
	}
}

impl CommandBuilder {
	pub fn new<S: AsRef<OsStr>>(program: S) -> CommandBuilder {
		CommandBuilder {
			program: OsString::from(program.as_ref()),
			timeout: None,
			debug: true,
			args: vec![],
			stdin: None,
			stdout: Some(Stdio::piped()),
			stderr: Some(Stdio::piped()),
			signal: None,
		}
	}

	pub fn with_debug(mut self, debug: bool) -> Self {
		self.debug = debug;
		self
	}

	pub fn with_timeout(&mut self, duration: Duration) -> &mut Self {
		self.timeout = Some(duration);
		self
	}

	pub fn timeout(mut self, duration: Option<Duration>) -> Self {
		self.timeout = duration;
		self
	}

	pub fn with_signal(&mut self, signal: Receiver<()>) -> &mut Self {
		self.signal = Some(signal);
		self
	}

	pub fn signal(mut self, signal: Option<Receiver<()>>) -> Self {
		self.signal = signal;
		self
	}

	pub fn arg<S: AsRef<OsStr>>(mut self, arg: S) -> Self {
		self.args.push(arg.as_ref().into());
		self
	}

	pub fn with_arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
		self.args.push(arg.as_ref().into());
		self
	}

	pub fn args<I, S>(mut self, args: I) -> Self
	where
		I: IntoIterator<Item = S>,
		S: AsRef<OsStr>,
	{
		for arg in args {
			self.args.push(arg.as_ref().into());
		}
		self
	}

	pub fn with_args<I, S>(&mut self, args: I) -> &mut Self
	where
		I: IntoIterator<Item = S>,
		S: AsRef<OsStr>,
	{
		for arg in args {
			self.args.push(arg.as_ref().into());
		}
		self
	}

	pub fn stdout<T: Into<Stdio>>(mut self, cfg: Option<T>) -> Self {
		if let Some(cfg) = cfg {
			self.stdout = Some(cfg.into());
		} else {
			self.stdout = None;
		}
		self
	}

	pub fn stderr<T: Into<Stdio>>(mut self, cfg: Option<T>) -> Self {
		if let Some(cfg) = cfg {
			self.stderr = Some(cfg.into());
		} else {
			self.stderr = None;
		}
		self
	}

	pub fn stdin<T: Into<Stdio>>(mut self, cfg: Option<T>) -> Self {
		if let Some(cfg) = cfg {
			self.stdin = Some(cfg.into());
		} else {
			self.stdin = None;
		}
		self
	}

	pub fn build(mut self) -> Cmd {
		return Cmd {
			debug: self.debug,
			program: self.program.to_owned(),
			args: self.args.to_owned(),
			stdin: self.stdin.take(),
			stdout: self.stdout.take(),
			stderr: self.stderr.take(),
			timeout: self.timeout.take(),
			signal: self.signal.take(),
		};
	}
}

impl Cmd {
	// region public methods

	pub fn builder<S: AsRef<OsStr>>(program: S) -> CommandBuilder {
		CommandBuilder::new(program)
	}

	pub fn new<S: AsRef<OsStr>>(program: S) -> Self {
		Cmd {
			program: OsString::from(program.as_ref()),
			timeout: None,
			debug: true,
			args: vec![],
			stdin: None,
			stdout: None,
			stderr: None,
			signal: None,
		}
	}

	pub fn command(mut self) -> Command {
		let mut command = Command::new(self.program.to_os_string());
		command.args(self.args.clone());

		if let Some(stdin) = self.stdin.take() {
			command.stdin(stdin);
		}

		if let Some(stdout) = self.stdout.take() {
			command.stdout(stdout);
		}

		if let Some(stderr) = self.stderr.take() {
			command.stderr(stderr);
		}

		command
	}

	// endregion punlic methods

	pub fn output(self) -> crate::Result<Output> {
		self.wait_for_output()
	}

	pub(crate) fn wait_for_output(mut self) -> crate::Result<Output> {
		if self.debug {
			self.debug();
		}

		let timeout = self.timeout.take();
		let cancel_signal = self.signal.take();

		let mut command = self.command();
		let mut child = command.spawn().unwrap();

		let stdout = child.stdout.take();
		let stderr = child.stderr.take();

		let pair = Arc::new((Mutex::new(None), Condvar::new()));
		let pair2 = Arc::clone(&pair);

		let thread = std::thread::Builder::new().name("cmd_wait".to_string()).spawn(move || {
			//trace!("Started thread {:?}", std::thread::current());

			let (lock, condvar) = &*pair2;
			let mut status_mutex = lock.lock().unwrap();

			let now = Instant::now();
			let term = Arc::new(AtomicBool::new(false));
			let mut killed = false;
			signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term)).unwrap();

			loop {
				// listen for external signals
				if let Some(ref cancel_signal) = cancel_signal {
					if !killed {
						if let Ok(()) = cancel_signal.try_recv() {
							let _ = child.kill().unwrap();
							killed = true;
						}
					}
				}

				// listen for Ctrl+c signal
				if !killed && term.load(Ordering::Relaxed) {
					trace!("Ctr+c received!");
					let _ = child.kill().unwrap();
					killed = true;
				}

				// listend for wait exit status
				if let Ok(Some(status)) = child.try_wait() {
					trace!("Exit Status is: {}", status);
					*status_mutex = Some(status);
					condvar.notify_one();
					break;
				} else {
					// finally check for timeout
					if !killed {
						if let Some(timeout) = timeout {
							if now.elapsed() > timeout {
								warn!("timeout passed `{}ms`... kill the process", now.elapsed().as_millis());
								let _ = child.kill().unwrap();
								killed = true;
								//break;
							}
						}
					}
				}
			}
		})?;

		let output = Cmd::read_to_end(stdout, stderr);

		if let Err(_err) = thread.join() {
			warn!("failed to join the thread!");
		}

		// Wait for the thread to start up.
		let (lock, cvar) = &*pair;
		let mut status = lock.lock().unwrap();
		while status.is_none() {
			status = cvar.wait(status).unwrap();
		}

		match output {
			Ok(output) => Ok(Output {
				status: status.unwrap(),
				stdout: output.0,
				stderr: output.1,
			}),
			Err(e) => Err(e),
		}
	}

	pub fn read_to_end(stdout: Option<ChildStdout>, stderr: Option<ChildStderr>) -> Result<(Vec<u8>, Vec<u8>), Error> {
		//let mut stdout_lines_count = 0;
		//let mut stderr_lines_count = 0;

		let mut stdout_writer: Vec<u8> = Vec::new();
		let mut stderr_writer: Vec<u8> = Vec::new();

		if let Some(stdout) = stdout {
			let stdout_reader = BufReader::new(stdout);
			for line in <BufReader<ChildStdout> as BufReaderExt<BufReader<ChildStdout>>>::lines_vec(stdout_reader) {
				stdout_writer.extend(line?);
				//stdout_lines_count += 1;
			}
		}

		if let Some(stderr) = stderr {
			let stderr_reader = BufReader::new(stderr);
			for line in <BufReader<ChildStderr> as BufReaderExt<BufReader<ChildStderr>>>::lines_vec(stderr_reader) {
				stderr_writer.extend(line?);
				//stderr_lines_count += 1;
			}
		}

		//debug!("[stdout] Completed with {stdout_lines_count} lines");
		//debug!("[stderr] Completed with {stderr_lines_count} lines");
		Ok((stdout_writer, stderr_writer))
	}
}

impl Vec8ToString for Vec<u8> {
	fn as_str(&self) -> Option<&str> {
		match std::str::from_utf8(self) {
			Ok(s) => Some(s),
			Err(_) => None,
		}
	}
}

pub(crate) trait BufReaderExt<B: BufRead> {
	fn lines_vec(self) -> LinesVec<Self>
	where
		Self: Sized;
}

pub struct LinesVec<B> {
	buf: B,
}

impl<B: BufRead, R> BufReaderExt<B> for BufReader<R> {
	fn lines_vec(self) -> LinesVec<Self>
	where
		Self: Sized,
	{
		LinesVec { buf: self }
	}
}

impl<B: BufRead> Iterator for LinesVec<B> {
	type Item = std::io::Result<Vec<u8>>;

	fn next(&mut self) -> Option<std::io::Result<Vec<u8>>> {
		let mut buf = Vec::new();
		match self.buf.read_until(b'\n', &mut buf) {
			Ok(0) => None,
			Ok(_n) => Some(Ok(buf)),
			Err(e) => Some(Err(e)),
		}
	}
}