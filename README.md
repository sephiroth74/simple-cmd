# simple-cmd

[![crates.io](https://img.shields.io/crates/v/simple-cmd.svg)](https://crates.io/crates/simple-cmd/)
[![ci](https://github.com/sephiroth74/simple-cmd/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/sephiroth74/simple-cmd/actions/workflows/rust.yml)

Rust command exeuctor

Example:

```rust
use simple_cmd::Cmd;
use simple_cmd::prelude::*;
use tracing::trace;
use std::time::Duration;

pub fn main() {
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
```

Piping:

```rust
use simple_cmd::Cmd;
use simple_cmd::prelude::*;

fn test_pipe() {
    let builder = Cmd::builder("echo").args(&["hello pretty world"]).with_debug(true);
    let command1 = builder.build();

    let mut command2 = Command::new("sed");
    command2.args(&["s/pretty/_/"]);
    command2.stdout(Stdio::piped());

    let result = command1.pipe(command2).unwrap();
    let output = result.stdout.as_str().unwrap().trim();

    assert!(result.success());
    assert_eq!("hello _ world", output);
}

```
