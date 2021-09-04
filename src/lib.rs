#![deny(rust_2018_idioms)]
#![allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::upper_case_acronyms, // see https://github.com/rust-lang/rust-clippy/issues/6974
    clippy::vec_init_then_push, // uses two different styles of initialization
)]
#![recursion_limit = "1024"]

pub use crate::config::*;
pub use crate::currentprocess::varsource::*;
pub use crate::currentprocess::*;
pub use crate::errors::*;
pub use crate::notifications::*;
pub use crate::toolchain::*;
pub(crate) use crate::utils::toml_utils;

#[macro_use]
extern crate rs_tracing;

// A list of all binaries which Rustup will proxy.
pub static TOOLS: &[&str] = &[
    "rustc",
    "rustdoc",
    "cargo",
    "rust-lldb",
    "rust-gdb",
    "rust-gdbgui",
    "rls",
    "cargo-clippy",
    "clippy-driver",
    "cargo-miri",
];

// Tools which are commonly installed by Cargo as well as rustup. We take a bit
// more care with these to ensure we don't overwrite the user's previous
// installation.
pub static DUP_TOOLS: &[&str] = &["rustfmt", "cargo-fmt"];

fn component_for_bin(binary: &str) -> Option<&'static str> {
    use std::env::consts::EXE_SUFFIX;

    let binary_prefix = match binary.find(EXE_SUFFIX) {
        _ if EXE_SUFFIX.is_empty() => binary,
        Some(i) => &binary[..i],
        None => binary,
    };

    match binary_prefix {
        "rustc" | "rustdoc" => Some("rustc"),
        "cargo" => Some("cargo"),
        "rust-lldb" | "rust-gdb" | "rust-gdbgui" => Some("rustc"), // These are not always available
        "rls" => Some("rls"),
        "cargo-clippy" => Some("clippy"),
        "clippy-driver" => Some("clippy"),
        "cargo-miri" => Some("miri"),
        "rustfmt" | "cargo-fmt" => Some("rustfmt"),
        _ => None,
    }
}

#[macro_use]
pub mod cli;
pub mod command;
mod config;
pub mod currentprocess;
pub mod diskio;
pub mod dist;
pub mod env_var;
pub mod errors;
pub mod fallback_settings;
mod install;
mod notifications;
pub mod settings;
pub mod test;
pub mod toolchain;
pub mod utils;
