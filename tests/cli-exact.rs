//! Yet more cli test cases. These are testing that the output
//! is exactly as expected.

extern crate rustup_dist;
extern crate rustup_mock;

use rustup_mock::clitools::{self, Config, Scenario,
                               expect_ok, expect_ok_ex,
                               expect_err_ex,
                               this_host_triple};
use std::env;

macro_rules! for_host { ($s: expr) => (&format!($s, this_host_triple())) }

fn setup(f: &Fn(&Config)) {
    clitools::setup(Scenario::SimpleV2, f);
}

#[test]
fn update() {
    setup(&|config| {
        expect_ok_ex(config, &["rustup", "update", "nightly"],
for_host!(r"
  nightly-{0} installed - 1.3.0 (hash-n-2)

"),
for_host!(r"info: syncing channel updates for 'nightly-{0}'
info: downloading component 'rust-std'
info: downloading component 'rustc'
info: downloading component 'cargo'
info: downloading component 'rust-docs'
info: installing component 'rust-std'
info: installing component 'rustc'
info: installing component 'cargo'
info: installing component 'rust-docs'
"));
    });
}

#[test]
fn update_again() {
    setup(&|config| {
        expect_ok(config, &["rustup", "update", "nightly"]);
        expect_ok_ex(config, &["rustup", "update", "nightly"],
for_host!(r"
  nightly-{0} unchanged - 1.3.0 (hash-n-2)

"),
for_host!(r"info: syncing channel updates for 'nightly-{0}'
"));
    });
}

#[test]
fn default() {
    setup(&|config| {
        expect_ok_ex(config, &["rustup", "default", "nightly"],
for_host!(r"
  nightly-{0} installed - 1.3.0 (hash-n-2)

"),
for_host!(r"info: syncing channel updates for 'nightly-{0}'
info: downloading component 'rust-std'
info: downloading component 'rustc'
info: downloading component 'cargo'
info: downloading component 'rust-docs'
info: installing component 'rust-std'
info: installing component 'rustc'
info: installing component 'cargo'
info: installing component 'rust-docs'
info: default toolchain set to 'nightly-{0}'
"));
    });
}

#[test]
fn override_again() {
    setup(&|config| {
        let cwd = env::current_dir().unwrap();
        expect_ok(config, &["rustup", "override", "add", "nightly"]);
        expect_ok_ex(config, &["rustup", "override", "add", "nightly"],
for_host!(r"
  nightly-{} unchanged - 1.3.0 (hash-n-2)

"),
&format!(
r"info: using existing install for 'nightly-{1}'
info: override toolchain for '{}' set to 'nightly-{1}'
", cwd.display(), &this_host_triple()));
    });
}

#[test]
fn remove_override() {
    setup(&|config| {
        let cwd = env::current_dir().unwrap();
        expect_ok(config, &["rustup", "override", "add", "nightly"]);
        expect_ok_ex(config, &["rustup", "override", "remove"],
r"",
&format!(r"info: override toolchain for '{}' removed
", cwd.display()));
    });
}

#[test]
fn remove_override_none() {
    setup(&|config| {
        let cwd = env::current_dir().unwrap();
        expect_ok_ex(config, &["rustup", "override", "remove"],
r"",
&format!(r"info: no override toolchain for '{}'
", cwd.display()));
    });
}

#[test]
fn list_overrides() {
    setup(&|config| {
        let cwd = std::fs::canonicalize(env::current_dir().unwrap()).unwrap();
        let mut cwd_formatted = format!("{}", cwd.display()).to_string();
        
        if cfg!(windows) {
            cwd_formatted = cwd_formatted[4..].to_owned();
        }
        
        let trip = this_host_triple();
        expect_ok(config, &["rustup", "override", "add", "nightly"]);
        expect_ok_ex(config, &["rustup", "override", "list"], 
                     &format!("{:<40}\t{:<20}\n", cwd_formatted, &format!("nightly-{}", trip)), r""); 
    });
}

#[test]
fn update_no_manifest() {
    setup(&|config| {
        expect_err_ex(config, &["rustup", "update", "nightly-2016-01-01"],
r"",
for_host!(r"info: syncing channel updates for 'nightly-2016-01-01-{0}'
error: no release found for 'nightly-2016-01-01'
"));
    });
}

// Issue #111
#[test]
fn update_invalid_toolchain() {
   setup(&|config| {
        expect_err_ex(config, &["rustup", "update", "nightly-2016-03-1"],
r"",
r"info: syncing channel updates for 'nightly-2016-03-1'
error: target not found: '2016-03-1'
");
   });
 }

#[test]
fn default_invalid_toolchain() {
   setup(&|config| {
        expect_err_ex(config, &["rustup", "default", "nightly-2016-03-1"],
r"",
r"info: syncing channel updates for 'nightly-2016-03-1'
error: target not found: '2016-03-1'
");
   });
}

#[test]
fn list_targets() {
    setup(&|config| {
        let trip = this_host_triple();
        let mut sorted = vec![format!("{} (default)", &*trip),
                              format!("{} (installed)", clitools::CROSS_ARCH1),
                              clitools::CROSS_ARCH2.to_string()];
        sorted.sort();

        let expected = format!("{}\n{}\n{}\n", sorted[0], sorted[1], sorted[2]);

        expect_ok(config, &["rustup", "default", "nightly"]);
        expect_ok(config, &["rustup", "target", "add",
                            clitools::CROSS_ARCH1]);
        expect_ok_ex(config, &["rustup", "target", "list"],
&expected,
r"");
    });
}

#[test]
fn cross_install_indicates_target() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "nightly"]);
        expect_ok_ex(config, &["rustup", "target", "add", clitools::CROSS_ARCH1],
r"",
&format!(r"info: downloading component 'rust-std' for '{0}'
info: installing component 'rust-std' for '{0}'
", clitools::CROSS_ARCH1));
    });
}
