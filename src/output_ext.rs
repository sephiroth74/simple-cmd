#[cfg(all(not(target_os = "hermit"), any(unix, doc)))]
use std::os::unix::prelude::ExitStatusExt;
use std::process::Output;

pub trait OutputExt {
	fn success(&self) -> bool;
	fn error(&self) -> bool;
	fn has_stdout(&self) -> bool;

	#[cfg(all(not(target_os = "hermit"), any(unix, doc)))]
	fn has_signal(&self) -> bool;

	#[cfg(all(not(target_os = "hermit"), any(unix, doc)))]
	fn signal(&self) -> Option<i32>;

	fn interrupt(&self) -> bool;
	fn kill(&self) -> bool;
}

impl OutputExt for Output {
	fn success(&self) -> bool {
		self.status.success()
	}
	fn error(&self) -> bool {
		!self.status.success()
	}

	fn has_stdout(&self) -> bool {
		!self.stdout.is_empty()
	}

	#[cfg(all(not(target_os = "hermit"), any(unix, doc)))]
	fn has_signal(&self) -> bool {
		self.status.signal().is_some()
	}

	#[cfg(all(not(target_os = "hermit"), any(unix, doc)))]
	fn signal(&self) -> Option<i32> {
		self.status.signal()
	}

	fn interrupt(&self) -> bool {
		self.signal().map(|s| signal_hook::consts::SIGINT == s).unwrap_or(false)
	}

	fn kill(&self) -> bool {
		self.signal().map(|s| signal_hook::consts::SIGKILL == s).unwrap_or(false)
	}
}
