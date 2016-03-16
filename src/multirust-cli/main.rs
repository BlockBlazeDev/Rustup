extern crate multirust_dist;
#[macro_use]
extern crate multirust_utils;

#[macro_use]
extern crate clap;
extern crate regex;
extern crate hyper;
#[macro_use]
extern crate multirust;
extern crate term;
extern crate openssl;
extern crate itertools;
extern crate time;
extern crate rand;
#[macro_use]
extern crate scopeguard;
extern crate tempdir;

#[cfg(windows)]
extern crate winapi;
#[cfg(windows)]
extern crate winreg;
#[cfg(windows)]
extern crate user32;
#[cfg(windows)]
extern crate kernel32;
extern crate libc;

#[macro_use]
mod log;
mod cli;
mod common;
mod download_tracker;
mod multirust_mode;
mod proxy_mode;
mod setup_mode;
mod self_update;
mod tty;

use std::env;
use std::path::PathBuf;
use multirust::{Error, Result};

fn main() {
    if let Err(e) = run_multirust() {
        err!("{}", e);
        std::process::exit(1);
    }
}

fn run_multirust() -> Result<()> {
    // Guard against infinite recursion
    let recursion_count = env::var("RUST_RECURSION_COUNT").ok()
        .and_then(|s| s.parse().ok()).unwrap_or(0);
    if recursion_count > 5 {
        return Err(Error::InfiniteRecursion);
    }

    // The name of arg0 determines how the program is going to behave
    let arg0 = env::args().next().map(|a| PathBuf::from(a));
    let name = arg0.as_ref()
        .and_then(|a| a.file_stem())
        .and_then(|a| a.to_str());
    match name {
        Some("multirust") => {
            multirust_mode::main()
        }
        Some("multirust-setup") => {
            setup_mode::main()
        }
        Some("multirust-rs") => {
            // This is for compatibility with previous revisions of multirust-rs
            // self-update, which expect this file to be available on the
            // server and execute it with `self install` as the arguments.
            // FIXME: Verify this works.
            self_update::install(false, false)
        }
        Some(n) if n.starts_with("multirust-gc-") => {
            // This is the final uninstallation stage on windows where
            // multirust deletes its own exe
            self_update::complete_windows_uninstall()
        }
        Some(_) => {
            proxy_mode::main()
        }
        None => {
            // Weird case. No arg0, or it's unparsable.
            Err(Error::NoExeName)
        }
    }
}

