/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::env;
use std::io;
use std::path::Path;
use std::process::Command;
use std::str::{Utf8Error, from_utf8};

extern crate tempdir;
use self::tempdir::TempDir;

#[derive(Debug)]
/// Possible errors that can occur during `rustc --pretty=expanded`.
pub enum Error {
    /// Error during creation of temporary directory
    Io(io::Error),
    /// Output of `cargo metadata` was not valid utf8
    Utf8(Utf8Error),
    /// Error during execution of `cargo rustc --pretty=expanded`
    Compile(String),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}
impl From<Utf8Error> for Error {
    fn from(err: Utf8Error) -> Self {
        Error::Utf8(err)
    }
}

/// Use rustc to expand and pretty print the crate into a single file,
/// removing any macros in the process.
pub fn expand(manifest_path: &Path, crate_name: &str, version: &str) -> Result<String, Error> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| String::from("cargo"));
    let run = |target_dir: &Path| {
        let mut cmd = Command::new(cargo);
        cmd.env("CARGO_TARGET_DIR", target_dir);
        cmd.arg("rustc");
        cmd.arg("--manifest-path");
        cmd.arg(manifest_path);
        cmd.arg("--all-features");
        cmd.arg("-p");
        cmd.arg(&format!("{}:{}", crate_name, version));
        cmd.arg("--");
        cmd.arg("-Z");
        cmd.arg("unstable-options");
        cmd.arg("--pretty=expanded");
        let output = cmd.output()?;

        let src = from_utf8(&output.stdout)?.to_owned();
        let error = from_utf8(&output.stderr)?.to_owned();

        if src.len() == 0 {
            Err(Error::Compile(error))
        } else {
            Ok(src)
        }
    };

    if let Ok(ref path) = env::var("CARGO_EXPAND_TARGET_DIR") {
        run(&Path::new(path))
    } else {
        // Create a temp directory to use as a target dir for cargo expand, for
        // hygenic purposes.
        let target_dir = TempDir::new("cbindgen-expand")?;

        run(target_dir.path())
    }
}
