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
