//! Test cases for new rustup UI

extern crate rustup_dist;
extern crate rustup_utils;
extern crate rustup_mock;
extern crate tempdir;

use rustup_mock::clitools::{self, Config, Scenario,
                               expect_ok, expect_ok_ex,
                               expect_stdout_ok,
                               expect_err_ex,
                               set_current_dist_date,
                               this_host_triple};

macro_rules! for_host { ($s: expr) => (&format!($s, this_host_triple())) }

pub fn setup(f: &Fn(&Config)) {
    clitools::setup(Scenario::ArchivesV2, &|config| {
        f(config);
    });
}

#[test]
fn rustup_stable() {
    setup(&|config| {
        set_current_dist_date(config, "2015-01-01");
        expect_ok(config, &["rustup", "update", "stable"]);
        set_current_dist_date(config, "2015-01-02");
        expect_ok_ex(config, &["rustup", "update", "--no-self-update"],
for_host!(r"
  stable-{0} updated - 1.1.0 (hash-s-2)

"),
for_host!(r"info: syncing channel updates for 'stable-{0}'
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
fn rustup_stable_no_change() {
    setup(&|config| {
        set_current_dist_date(config, "2015-01-01");
        expect_ok(config, &["rustup", "update", "stable"]);
        expect_ok_ex(config, &["rustup", "update", "--no-self-update"],
for_host!(r"
  stable-{0} unchanged - 1.0.0 (hash-s-1)

"),
for_host!(r"info: syncing channel updates for 'stable-{0}'
"));
    });
}

#[test]
fn rustup_all_channels() {
    setup(&|config| {
        set_current_dist_date(config, "2015-01-01");
        expect_ok(config, &["rustup", "update", "stable"]);
        expect_ok(config, &["multirust", "update", "beta"]);
        expect_ok(config, &["multirust", "update", "nightly"]);
        set_current_dist_date(config, "2015-01-02");
        expect_ok_ex(config, &["rustup", "update", "--no-self-update"],
for_host!(r"
   stable-{0} updated - 1.1.0 (hash-s-2)
     beta-{0} updated - 1.2.0 (hash-b-2)
  nightly-{0} updated - 1.3.0 (hash-n-2)

"),
for_host!(r"info: syncing channel updates for 'stable-{0}'
info: downloading component 'rust-std'
info: downloading component 'rustc'
info: downloading component 'cargo'
info: downloading component 'rust-docs'
info: installing component 'rust-std'
info: installing component 'rustc'
info: installing component 'cargo'
info: installing component 'rust-docs'
info: syncing channel updates for 'beta-{0}'
info: downloading component 'rust-std'
info: downloading component 'rustc'
info: downloading component 'cargo'
info: downloading component 'rust-docs'
info: installing component 'rust-std'
info: installing component 'rustc'
info: installing component 'cargo'
info: installing component 'rust-docs'
info: syncing channel updates for 'nightly-{0}'
info: downloading component 'rust-std'
info: downloading component 'rustc'
info: downloading component 'cargo'
info: downloading component 'rust-docs'
info: installing component 'rust-std'
info: installing component 'rustc'
info: installing component 'cargo'
info: installing component 'rust-docs'
"));
    })
}

#[test]
fn rustup_some_channels_up_to_date() {
    setup(&|config| {
        set_current_dist_date(config, "2015-01-01");
        expect_ok(config, &["rustup", "update", "stable"]);
        expect_ok(config, &["multirust", "update", "beta"]);
        expect_ok(config, &["multirust", "update", "nightly"]);
        set_current_dist_date(config, "2015-01-02");
        expect_ok(config, &["multirust", "update", "beta"]);
        expect_ok_ex(config, &["rustup", "update", "--no-self-update"],
for_host!(r"
   stable-{0} updated - 1.1.0 (hash-s-2)
   beta-{0} unchanged - 1.2.0 (hash-b-2)
  nightly-{0} updated - 1.3.0 (hash-n-2)

"),
for_host!(r"info: syncing channel updates for 'stable-{0}'
info: downloading component 'rust-std'
info: downloading component 'rustc'
info: downloading component 'cargo'
info: downloading component 'rust-docs'
info: installing component 'rust-std'
info: installing component 'rustc'
info: installing component 'cargo'
info: installing component 'rust-docs'
info: syncing channel updates for 'beta-{0}'
info: syncing channel updates for 'nightly-{0}'
info: downloading component 'rust-std'
info: downloading component 'rustc'
info: downloading component 'cargo'
info: downloading component 'rust-docs'
info: installing component 'rust-std'
info: installing component 'rustc'
info: installing component 'cargo'
info: installing component 'rust-docs'
"));
    })
}

#[test]
fn rustup_no_channels() {
    setup(&|config| {
        expect_ok(config, &["rustup", "update", "stable"]);
        expect_ok(config, &["multirust", "remove-toolchain", "stable"]);
        expect_ok_ex(config, &["rustup", "update", "--no-self-update"],
r"",
r"info: no updatable toolchains installed
");
    })
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
fn add_target() {
    setup(&|config| {
        let path = format!("toolchains/nightly-{}/lib/rustlib/{}/lib/libstd.rlib",
                           &this_host_triple(), clitools::CROSS_ARCH1);
        expect_ok(config, &["rustup", "default", "nightly"]);
        expect_ok(config, &["rustup", "target", "add",
                            clitools::CROSS_ARCH1]);
        assert!(config.rustupdir.join(path).exists());
    });
}

#[test]
fn remove_target() {
    setup(&|config| {
        let ref path = format!("toolchains/nightly-{}/lib/rustlib/{}/lib/libstd.rlib",
                               &this_host_triple(), clitools::CROSS_ARCH1);
        expect_ok(config, &["rustup", "default", "nightly"]);
        expect_ok(config, &["rustup", "target", "add",
                            clitools::CROSS_ARCH1]);
        assert!(config.rustupdir.join(path).exists());
        expect_ok(config, &["rustup", "target", "remove",
                            clitools::CROSS_ARCH1]);
        assert!(!config.rustupdir.join(path).exists());
    });
}

#[test]
fn list_targets() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "nightly"]);
        expect_stdout_ok(config, &["rustup", "target", "list"],
                         clitools::CROSS_ARCH1);
    });
}

#[test]
fn add_target_explicit() {
    setup(&|config| {
        let path = format!("toolchains/nightly-{}/lib/rustlib/{}/lib/libstd.rlib",
                           &this_host_triple(), clitools::CROSS_ARCH1);
        expect_ok(config, &["rustup", "update", "nightly"]);
        expect_ok(config, &["rustup", "target", "add", "--toolchain", "nightly",
                            clitools::CROSS_ARCH1]);
        assert!(config.rustupdir.join(path).exists());
    });
}

#[test]
fn remove_target_explicit() {
    setup(&|config| {
        let ref path = format!("toolchains/nightly-{}/lib/rustlib/{}/lib/libstd.rlib",
                               &this_host_triple(), clitools::CROSS_ARCH1);
        expect_ok(config, &["rustup", "update", "nightly"]);
        expect_ok(config, &["rustup", "target", "add", "--toolchain", "nightly",
                            clitools::CROSS_ARCH1]);
        assert!(config.rustupdir.join(path).exists());
        expect_ok(config, &["rustup", "target", "remove", "--toolchain", "nightly",
                            clitools::CROSS_ARCH1]);
        assert!(!config.rustupdir.join(path).exists());
    });
}

#[test]
fn list_targets_explicit() {
    setup(&|config| {
        expect_ok(config, &["rustup", "update", "nightly"]);
        expect_stdout_ok(config, &["rustup", "target", "list", "--toolchain", "nightly"],
                         clitools::CROSS_ARCH1);
    });
}

#[test]
fn link() {
    setup(&|config| {
        let path = config.customdir.join("custom-1");
        let path = path.to_string_lossy();
        expect_ok(config, &["rustup", "toolchain", "link", "custom",
                            &path]);
        expect_ok(config, &["rustup", "default", "custom"]);
        expect_stdout_ok(config, &["rustc", "--version"],
                         "hash-c-1");
    });
}

#[test]
fn show_toolchain_none() {
    setup(&|config| {
        expect_ok_ex(config, &["rustup", "show"],
r"no active toolchain
",
r"");
    });
}

#[test]
fn show_toolchain_default() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "nightly"]);
        expect_ok_ex(config, &["rustup", "show"],
for_host!(r"nightly-{0} (default toolchain)
"),
r"");
    });
}

#[test]
fn list_default_toolchain() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "nightly"]);
        expect_ok_ex(config, &["rustup", "toolchain", "list"],
for_host!(r"nightly-{0} (default)
"),
r"");
    });
}

#[test]
#[ignore(windows)] // FIXME rustup displays UNC paths
fn show_toolchain_override() {
    setup(&|config| {
        let cwd = ::std::env::current_dir().unwrap();
        expect_ok(config, &["rustup", "override", "add", "nightly"]);
        expect_ok_ex(config, &["rustup", "show"],
&format!(r"nightly (directory override for '{}')
", cwd.display()),
r"");
    });
}

#[test]
fn show_toolchain_override_not_installed() {
    setup(&|config| {
        expect_ok(config, &["rustup", "override", "add", "nightly"]);
        expect_ok(config, &["rustup", "toolchain", "remove", "nightly"]);
        // I'm not sure this should really be erroring when the toolchain
        // is not installed; just capturing the behavior.
        expect_err_ex(config, &["rustup", "show"],
r"",
for_host!(r"error: toolchain 'nightly-{0}' is not installed
"));
    });
}

#[test]
fn show_toolchain_env() {
    setup(&|config| {
        expect_ok(config, &["rustup", "default", "nightly"]);
        let mut cmd = clitools::cmd(config, "rustup", &["show"]);
        clitools::env(config, &mut cmd);
        cmd.env("RUSTUP_TOOLCHAIN", "nightly");
        let out = cmd.output().unwrap();
        assert!(out.status.success());
        let stdout = String::from_utf8(out.stdout).unwrap();
        assert!(&stdout == for_host!("nightly-{0} (environment override by RUSTUP_TOOLCHAIN)\n"));
    });
}

#[test]
fn show_toolchain_env_not_installed() {
    setup(&|config| {
        let mut cmd = clitools::cmd(config, "rustup", &["show"]);
        clitools::env(config, &mut cmd);
        cmd.env("RUSTUP_TOOLCHAIN", "nightly");
        let out = cmd.output().unwrap();
        // I'm not sure this should really be erroring when the toolchain
        // is not installed; just capturing the behavior.
        assert!(!out.status.success());
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert!(stderr == "error: toolchain 'nightly' is not installed\n");
    });
}
