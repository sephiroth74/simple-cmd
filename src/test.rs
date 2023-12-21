#[cfg(test)]
mod tests {
	use std::convert::Infallible;
	use std::sync::Once;
	use std::thread;
	use std::thread::sleep;
	use std::time::Duration;

	use crossbeam_channel::{bounded, Receiver};
	use log::trace;

	use crate::output_ext::OutputExt;
	use crate::Cmd;

	static INIT: Once = Once::new();

	macro_rules! init_log {
		() => {
			INIT.call_once(|| {
				simple_logger::SimpleLogger::new().env().init().unwrap();
			})
		};
	}

	#[allow(dead_code)]
	fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
		let (sender, receiver) = bounded(1);
		ctrlc::set_handler(move || {
			println!("sending CTRL+C to ctrl_channel");
			let _ = sender.send(());
		})?;
		Ok(receiver)
	}

	fn cancel_signal(timeout: Duration) -> Result<Receiver<()>, Infallible> {
		let (s, r) = bounded(1);
		thread::spawn(move || {
			sleep(timeout);
			trace!("sending CTRL+C signal...");
			let _ = s.send(());
		});

		Ok(r)
	}

	#[test]
	fn test_simple() {
		init_log!();
		let cmd = Cmd::builder("ls").with_debug(true).build();
		let output = cmd.output().expect("failed to wait for command");

		trace!("output: {:#?}", output);
	}

	#[test]
	fn test_sleep() {
		init_log!();
		let cmd = Cmd::builder("sleep").arg("1").timeout(Some(Duration::from_millis(100))).with_debug(true).build();
		let output = cmd.output().expect("failed to wait for command");
		trace!("output: {:#?}", output);

		assert!(!output.status.success());
		assert!(!output.interrupt());
		assert!(output.kill());
	}

	#[test]
	fn test_cancel_signal() {
		init_log!();
		let cancel_signal = cancel_signal(Duration::from_secs(1)).unwrap();
		let cmd = Cmd::builder("sleep").arg("2").with_debug(true).signal(Some(cancel_signal)).build();
		let output = cmd.output().expect("failed to wait for command");
		trace!("output: {:#?}", output);

		assert!(!output.status.success());
		assert!(output.kill());
		assert!(!output.interrupt());
	}
}
