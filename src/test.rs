#[cfg(test)]
mod tests {
	use std::convert::Infallible;
	use std::process::{Command, Stdio};
	use std::sync::Once;
	use std::thread;
	use std::thread::sleep;
	use std::time::{Duration, Instant};

	use crossbeam_channel::{bounded, Receiver};
	use tracing::{debug, trace, warn};

	use crate::debug::CommandDebug;
	use crate::output_ext::OutputExt;
	use crate::{Cmd, Vec8ToString};

	static INIT: Once = Once::new();

	macro_rules! init_log {
		() => {
			INIT.call_once(|| {
				let subscriber = tracing_subscriber::fmt()
					.compact()
					.with_file(false)
					.with_line_number(false)
					.with_max_level(tracing::Level::TRACE)
					.with_thread_ids(false)
					.with_thread_names(true)
					.finish();
				tracing::subscriber::set_global_default(subscriber).unwrap();
			})
		};
	}

	#[allow(dead_code)]
	fn ctrlc_channel() -> Result<Receiver<()>, ctrlc::Error> {
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
	fn test_error() {
		init_log!();
		let cmd = Cmd::builder("adb")
			.with_debug(true)
			.arg("push")
			.arg("/var/folders/h5/6_rkm34x20924137hx8_d7jr0000gn/T/1fc0ac62-7bd2-483d-8fcc-47ac37fcdf58")
			.arg("tvlib-aot-client.properties")
			.build();

		let output = cmd.output().expect("failed to wait for command");
		debug!("output: {:#?}", &output);

		let error: crate::Error = output.into();
		warn!("error: {}", error);
	}

	#[test]
	fn test_sleep() {
		init_log!();
		let cmd = Cmd::builder("sleep")
			.arg("1")
			.timeout(Some(Duration::from_millis(100)))
			.with_debug(true)
			.build();
		let output = cmd.output().expect("failed to wait for command");
		trace!("output: {:#?}", output);

		assert!(!output.status.success());
		assert!(!output.interrupt());
		assert!(output.kill());
	}

	#[test]
	fn test_run() {
		init_log!();
		let now = Instant::now();
		let pool = threadpool::Builder::new().num_threads(2).build();

		pool.execute(move || {
			let _r = Cmd::builder("sleep").arg("2").build().run();
			trace!("cmd 1 done");
		});

		pool.execute(move || {
			let _r = Cmd::builder("sleep").arg("2").build().run();
			trace!("cmd 2 done");
		});

		pool.execute(move || {
			let _r = Cmd::builder("sleep").arg("2").build().run();
			trace!("cmd 3 done");
		});

		pool.execute(move || {
			let _r = Cmd::builder("sleep").arg("2").build().run();
			trace!("cmd 4 done");
		});

		pool.join();

		let elapsed = now.elapsed();

		trace!("done in {:?}ms", elapsed.as_millis());
		debug_assert!(
			elapsed < Duration::from_secs(2),
			"Expected less than 2 seconds, but got {:?}",
			elapsed
		);
	}

	#[test]
	fn test_cancel_signal() {
		init_log!();
		let cancel_signal = cancel_signal(Duration::from_secs(1)).unwrap();
		let cmd = Cmd::builder("sleep")
			.arg("2")
			.with_debug(true)
			.signal(Some(cancel_signal))
			.build();
		let output = cmd.output().expect("failed to wait for command");
		trace!("output: {:#?}", output);

		assert!(!output.status.success());
		assert!(output.kill());
		assert!(!output.interrupt());
	}

	#[test]
	fn test_to_command() {
		init_log!();
		let builder = Cmd::builder("sleep").arg("1");
		let mut cmd: Command = builder.into();
		cmd.debug();
		let _ = cmd.output().unwrap();
	}

	#[test]
	fn test_cmd_to_string() {
		let builder = Cmd::builder("sleep").args(vec![
			"1", "2",
		]);
		let cmd_string = builder.to_string();
		assert_eq!(cmd_string, "sleep 1 2".to_string());
	}

	#[test]
	fn test_pipe() {
		init_log!();
		let cancel = ctrlc_channel().unwrap();
		let builder = Cmd::builder("adb")
			.args(vec![
				"shell",
				"while true; do screenrecord --bit-rate 4000000 --output-format=h264 --size 1920x1080 -; done",
			])
			.timeout(Some(Duration::from_secs(60)))
			.signal(Some(cancel))
			.with_debug(true);

		let command1 = builder.build();

		let mut command2 = Command::new("ffplay");
		command2.args(vec![
			"-loglevel",
			"verbose",
			"-stats",
			"-an",
			"-autoexit",
			"-framerate",
			"30",
			"-probesize",
			"600",
			"-vf",
			"scale=1024:-1",
			"-sync",
			"video",
			"-",
		]);
		command2.stdout(Stdio::piped());

		let result = command1.pipe(command2).unwrap();

		println!();
		println!("result: {:?}", result);
	}

	#[test]
	fn test_pipe2() {
		init_log!();
		let command1 = Cmd::builder("adb")
			.args(vec![
				"shell", "dumpsys", "power",
			])
			.with_debug(true)
			.build();

		let command2 = Cmd::builder("sed")
			.arg("-n")
			.arg("s/mWakefulness=\\(\\S*\\)/\\1/p")
			.with_debug(true)
			.stdout(Some(Stdio::piped()))
			.build();

		let result = command1.pipe(command2).unwrap();
		println!();
		println!("result: {:?}", result);

		let output = result.stdout.as_str().unwrap().trim().to_lowercase();
		assert!(output == "awake" || output == "asleep");
	}
}
