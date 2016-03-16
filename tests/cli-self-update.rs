//! Testing self install, uninstall and update

extern crate multirust_mock;
extern crate multirust_utils;
#[macro_use]
extern crate lazy_static;
extern crate tempdir;
#[macro_use]
extern crate scopeguard;

#[cfg(windows)]
extern crate winapi;
#[cfg(windows)]
extern crate winreg;

use tempdir::TempDir;
use std::sync::Mutex;
use std::env;
use std::env::consts::EXE_SUFFIX;
use std::path::Path;
use std::fs;
use std::process::Command;
use multirust_mock::clitools::{self, Config, Scenario,
                               expect_ok, expect_ok_ex,
                               expect_stdout_ok,
                               expect_err, expect_err_ex,
                               this_host_triple};
use multirust_mock::dist::{create_hash, calc_hash};
use multirust_utils::raw;

pub fn setup(f: &Fn(&Config, &Path, &Path)) {
    clitools::setup(Scenario::SimpleV2, &|config| {
        // Lock protects environment variables
        lazy_static! {
            static ref LOCK: Mutex<()> = Mutex::new(());
        }
        let _g = LOCK.lock();

        let cargo_home_tmp = TempDir::new("cargo_home").unwrap();
        // The uninstall process on windows involves using the directory above
        // CARGO_HOME, so make sure it's a subdir of our tempdir
        let ref cargo_home = cargo_home_tmp.path().join("ch");
        fs::create_dir(cargo_home).unwrap();

        let home_tmp = TempDir::new("home").unwrap();
        let home = home_tmp.path();

        // Both of these are only read during install/uninstall
        env::set_var("CARGO_HOME", &*cargo_home.to_string_lossy());
        env::set_var("HOME", &*home.to_string_lossy());

        // An windows these tests mess with the user's PATH. Save
        // and restore them here to keep from trashing things.
        let ref saved_path = get_path();
        defer! { restore_path(saved_path) }

        f(config, cargo_home, home);
    });
}

pub fn update_setup(f: &Fn(&Config, &Path, &Path)) {
    setup(&|config, cargo_home, _| {

        // Create a mock self-update server
        let ref self_dist_tmp = TempDir::new("self_dist").unwrap();
        let ref self_dist = self_dist_tmp.path();

        let ref trip = this_host_triple();
        let ref dist_dir = self_dist.join(&format!("{}", trip));
        let ref dist_exe = dist_dir.join(&format!("multirust-setup{}", EXE_SUFFIX));
        let ref dist_hash = dist_dir.join(&format!("multirust-setup{}.sha256", EXE_SUFFIX));
        let ref multirust_bin = config.exedir
            .path().join(&format!("multirust{}", EXE_SUFFIX));

        fs::create_dir_all(dist_dir).unwrap();
        fs::copy(multirust_bin, dist_exe).unwrap();
        // Modify the exe so it hashes different
        raw::append_file(dist_exe, "").unwrap();
        create_hash(dist_exe, dist_hash);

        let ref root_url = format!("file://{}", self_dist.display());
        env::set_var("MULTIRUST_UPDATE_ROOT", root_url);

        f(config, cargo_home, self_dist);
    });
}

#[test]
fn install_bins_to_cargo_home() {
    setup(&|config, cargo_home, _| {
        expect_ok(config, &["multirust-setup", "-y"]);
        let multirust = cargo_home.join(&format!("bin/multirust{}", EXE_SUFFIX));
        let rustc = cargo_home.join(&format!("bin/rustc{}", EXE_SUFFIX));
        let rustdoc = cargo_home.join(&format!("bin/rustdoc{}", EXE_SUFFIX));
        let cargo = cargo_home.join(&format!("bin/cargo{}", EXE_SUFFIX));
        let rust_lldb = cargo_home.join(&format!("bin/rust-lldb{}", EXE_SUFFIX));
        let rust_gdb = cargo_home.join(&format!("bin/rust-gdb{}", EXE_SUFFIX));
        assert!(multirust.exists());
        assert!(rustc.exists());
        assert!(rustdoc.exists());
        assert!(cargo.exists());
        assert!(rust_lldb.exists());
        assert!(rust_gdb.exists());
    });
}

#[test]
fn install_twice() {
    setup(&|config, cargo_home, _| {
        expect_ok(config, &["multirust-setup", "-y"]);
        expect_ok(config, &["multirust-setup", "-y"]);
        let multirust = cargo_home.join(&format!("bin/multirust{}", EXE_SUFFIX));
        assert!(multirust.exists());
    });
}

#[test]
#[cfg(unix)]
fn bins_are_executable() {
    setup(&|config, cargo_home, _| {
        expect_ok(config, &["multirust-setup", "-y"]);
        let ref multirust = cargo_home.join(&format!("bin/multirust{}", EXE_SUFFIX));
        let ref rustc = cargo_home.join(&format!("bin/rustc{}", EXE_SUFFIX));
        let ref rustdoc = cargo_home.join(&format!("bin/rustdoc{}", EXE_SUFFIX));
        let ref cargo = cargo_home.join(&format!("bin/cargo{}", EXE_SUFFIX));
        let ref rust_lldb = cargo_home.join(&format!("bin/rust-lldb{}", EXE_SUFFIX));
        let ref rust_gdb = cargo_home.join(&format!("bin/rust-gdb{}", EXE_SUFFIX));
        assert!(is_exe(multirust));
        assert!(is_exe(rustc));
        assert!(is_exe(rustdoc));
        assert!(is_exe(cargo));
        assert!(is_exe(rust_lldb));
        assert!(is_exe(rust_gdb));
    });

    fn is_exe(path: &Path) -> bool {
        use std::os::unix::fs::MetadataExt;
        let mode = path.metadata().unwrap().mode();

        mode & 0o777 == 0o755
    }
}

#[test]
fn install_creates_cargo_home() {
    setup(&|config, cargo_home, _| {
        fs::remove_dir(cargo_home).unwrap();
        fs::remove_dir(config.homedir.path()).unwrap();
        expect_ok(config, &["multirust-setup", "-y"]);
        assert!(cargo_home.exists());
    });
}

#[test]
fn uninstall_deletes_bins() {
    setup(&|config, cargo_home, _| {
        expect_ok(config, &["multirust-setup", "-y"]);
        expect_ok(config, &["multirust", "self", "uninstall", "-y"]);
        let multirust = cargo_home.join(&format!("bin/multirust{}", EXE_SUFFIX));
        let rustc = cargo_home.join(&format!("bin/rustc{}", EXE_SUFFIX));
        let rustdoc = cargo_home.join(&format!("bin/rustdoc{}", EXE_SUFFIX));
        let cargo = cargo_home.join(&format!("bin/cargo{}", EXE_SUFFIX));
        let rust_lldb = cargo_home.join(&format!("bin/rust-lldb{}", EXE_SUFFIX));
        let rust_gdb = cargo_home.join(&format!("bin/rust-gdb{}", EXE_SUFFIX));
        assert!(!multirust.exists());
        assert!(!rustc.exists());
        assert!(!rustdoc.exists());
        assert!(!cargo.exists());
        assert!(!rust_lldb.exists());
        assert!(!rust_gdb.exists());
    });
}

#[test]
fn uninstall_works_if_some_bins_dont_exist() {
    setup(&|config, cargo_home, _| {
        expect_ok(config, &["multirust-setup", "-y"]);
        let multirust = cargo_home.join(&format!("bin/multirust{}", EXE_SUFFIX));
        let rustc = cargo_home.join(&format!("bin/rustc{}", EXE_SUFFIX));
        let rustdoc = cargo_home.join(&format!("bin/rustdoc{}", EXE_SUFFIX));
        let cargo = cargo_home.join(&format!("bin/cargo{}", EXE_SUFFIX));
        let rust_lldb = cargo_home.join(&format!("bin/rust-lldb{}", EXE_SUFFIX));
        let rust_gdb = cargo_home.join(&format!("bin/rust-gdb{}", EXE_SUFFIX));

        fs::remove_file(&rustc).unwrap();
        fs::remove_file(&cargo).unwrap();

        expect_ok(config, &["multirust", "self", "uninstall", "-y"]);

        assert!(!multirust.exists());
        assert!(!rustc.exists());
        assert!(!rustdoc.exists());
        assert!(!cargo.exists());
        assert!(!rust_lldb.exists());
        assert!(!rust_gdb.exists());
    });
}

#[test]
fn uninstall_deletes_multirust_home() {
    setup(&|config, _, _| {
        expect_ok(config, &["multirust-setup", "-y"]);
        expect_ok(config, &["multirust", "default", "nightly"]);
        expect_ok(config, &["multirust", "self", "uninstall", "-y"]);
        assert!(!config.homedir.path().exists());
    });
}

#[test]
fn uninstall_works_if_multirust_home_doesnt_exist() {
    setup(&|config, _, _| {
        expect_ok(config, &["multirust-setup", "-y"]);
        fs::remove_dir_all(&config.homedir.path()).unwrap();
        expect_ok(config, &["multirust", "self", "uninstall", "-y"]);
    });
}

#[test]
fn uninstall_deletes_cargo_home() {
    setup(&|config, cargo_home, _| {
        expect_ok(config, &["multirust-setup", "-y"]);
        expect_ok(config, &["multirust", "self", "uninstall", "-y"]);
        assert!(!cargo_home.exists());
    });
}

#[test]
fn uninstall_fails_if_not_installed() {
    setup(&|config, cargo_home, _| {
        expect_ok(config, &["multirust-setup", "-y"]);
        let multirust = cargo_home.join(&format!("bin/multirust{}", EXE_SUFFIX));
        fs::remove_file(&multirust).unwrap();
        expect_err(config, &["multirust", "self", "uninstall", "-y"],
                   "multirust is not installed");
    });
}

// The other tests here just run multirust from a temp directory. This
// does the uninstall by actually invoking the installed binary in
// order to test that it can successfully delete itself.
#[test]
fn uninstall_self_delete_works() {
    setup(&|config, cargo_home, _| {
        expect_ok(config, &["multirust-setup", "-y"]);
        let multirust = cargo_home.join(&format!("bin/multirust{}", EXE_SUFFIX));
        let mut cmd = Command::new(multirust.clone());
        cmd.args(&["self", "uninstall", "-y"]);
        clitools::env(config, &mut cmd);
        let out = cmd.output().unwrap();
        println!("out: {}", String::from_utf8(out.stdout).unwrap());
        println!("err: {}", String::from_utf8(out.stderr).unwrap());

        assert!(out.status.success());
        assert!(!multirust.exists());
        assert!(!cargo_home.exists());

        let rustc = cargo_home.join(&format!("bin/rustc{}", EXE_SUFFIX));
        let rustdoc = cargo_home.join(&format!("bin/rustdoc{}", EXE_SUFFIX));
        let cargo = cargo_home.join(&format!("bin/cargo{}", EXE_SUFFIX));
        let rust_lldb = cargo_home.join(&format!("bin/rust-lldb{}", EXE_SUFFIX));
        let rust_gdb = cargo_home.join(&format!("bin/rust-gdb{}", EXE_SUFFIX));
        assert!(!rustc.exists());
        assert!(!rustdoc.exists());
        assert!(!cargo.exists());
        assert!(!rust_lldb.exists());
        assert!(!rust_gdb.exists());
    });
}

// On windows multirust self uninstall temporarily puts a multirust-gc-$randomnumber.exe
// file in CARGO_HOME/.. ; check that it doesn't exist.
#[test]
fn uninstall_doesnt_leave_gc_file() {
    setup(&|config, cargo_home, _| {
        expect_ok(config, &["multirust-setup", "-y"]);
        expect_ok(config, &["multirust", "self", "uninstall", "-y"]);

        let ref parent = cargo_home.parent().unwrap();
        // Actually, there just shouldn't be any files here
        for dirent in fs::read_dir(parent).unwrap() {
            let dirent = dirent.unwrap();
            println!("{}", dirent.path().display());
            panic!();
        }
    })
}

#[test]
#[ignore]
fn uninstall_stress_test() {
}

#[cfg(unix)]
fn install_adds_path_to_rc(rcfile: &str) {
    setup(&|config, cargo_home, home| {
        let my_rc = "foo\nbar\nbaz";
        let ref rc = home.join(rcfile);
        raw::write_file(rc, my_rc).unwrap();
        expect_ok(config, &["multirust-setup", "-y"]);

        let new_rc = raw::read_file(rc).unwrap();
        let addition = format!(r#"export PATH="{}/bin:$PATH""#,
                               cargo_home.display());
        let expected = format!("{}\n{}\n", my_rc, addition);
        assert_eq!(new_rc, expected);
    });
}

#[test]
#[cfg(unix)]
fn install_adds_path_to_bashrc() {
    install_adds_path_to_rc(".bashrc");
}

#[test]
#[cfg(unix)]
fn install_adds_path_to_zshrc() {
    install_adds_path_to_rc(".zshrc");
}

#[test]
#[cfg(unix)]
fn install_adds_path_to_kshrc() {
    install_adds_path_to_rc(".kshrc");
}

#[test]
#[cfg(unix)]
fn install_does_not_add_paths_to_rcfiles_that_dont_exist() {
    setup(&|config, _, home| {
        let my_bashrc = "foo\nbar\nbaz";
        let ref bashrc = home.join(".bashrc");
        raw::write_file(bashrc, my_bashrc).unwrap();
        expect_ok(config, &["multirust-setup", "-y"]);

        let ref zshrc = home.join(".zshrc");
        let ref kshrc = home.join(".kshrc");
        assert!(!zshrc.exists());
        assert!(!kshrc.exists());
    });
}

#[test]
#[cfg(unix)]
fn install_adds_path_to_bashrc_zshrc_and_kshrc() {
}

#[test]
#[cfg(unix)]
fn install_adds_path_to_rcfile_just_once() {
    setup(&|config, cargo_home, home| {
        let my_bashrc = "foo\nbar\nbaz";
        let ref bashrc = home.join(".bashrc");
        raw::write_file(bashrc, my_bashrc).unwrap();
        expect_ok(config, &["multirust-setup", "-y"]);
        expect_ok(config, &["multirust-setup", "-y"]);

        let new_bashrc = raw::read_file(bashrc).unwrap();
        let addition = format!(r#"export PATH="{}/bin:$PATH""#,
                               cargo_home.display());
        let expected = format!("{}\n{}\n", my_bashrc, addition);
        assert_eq!(new_bashrc, expected);
    });
}

// What happens when install can't find any shells to add the PATH to?
#[test]
#[cfg(unix)]
fn install_when_no_path_methods() {
    setup(&|config, _, home| {
        expect_ok(config, &["multirust-setup", "-y"]);

        for rc in &[".bashrc", ".zshrc", ".kshrc"] {
            assert!(!home.join(rc).exists());
        }
    });
}

#[cfg(unix)]
fn uninstall_removes_path_from_rc(rcfile: &str) {
    setup(&|config, _, home| {
        let my_rc = "foo\nbar\nbaz";
        let ref rc = home.join(rcfile);
        raw::write_file(rc, my_rc).unwrap();
        expect_ok(config, &["multirust-setup", "-y"]);
        expect_ok(config, &["multirust", "self", "uninstall", "-y"]);

        let new_rc = raw::read_file(rc).unwrap();
        assert_eq!(new_rc, my_rc);
    });
}

#[test]
#[cfg(unix)]
fn uninstall_removes_path_from_bashrc() {
    uninstall_removes_path_from_rc(".bashrc");
}

#[test]
#[cfg(unix)]
fn uninstall_removes_path_from_zshrc() {
    uninstall_removes_path_from_rc(".zshrc");
}

#[test]
#[cfg(unix)]
fn uninstall_removes_path_from_kshrc() {
    uninstall_removes_path_from_rc(".kshrc");
}

#[test]
#[cfg(unix)]
fn uninstall_doesnt_touch_rc_files_that_dont_exist() {
    setup(&|config, _, home| {
        let my_rc = "foo\nbar\nbaz";
        let ref bashrc = home.join(".bashrc");
        raw::write_file(bashrc, my_rc).unwrap();
        expect_ok(config, &["multirust-setup", "-y"]);
        expect_ok(config, &["multirust", "self", "uninstall", "-y"]);

        let ref zshrc = home.join(".zshrc");
        let ref kshrc = home.join(".zshrc");
        assert!(!zshrc.exists());
        assert!(!kshrc.exists());
    });
}

#[test]
#[cfg(unix)]
fn uninstall_doesnt_touch_rc_files_that_dont_contain_cargo_home_path() {
    setup(&|config, _, home| {
        let my_rc = "foo\nbar\nbaz";
        let ref bashrc = home.join(".bashrc");
        raw::write_file(bashrc, my_rc).unwrap();
        expect_ok(config, &["multirust-setup", "-y"]);

        let ref zshrc = home.join(".zshrc");
        raw::write_file(zshrc, my_rc).unwrap();

        let zsh = raw::read_file(zshrc).unwrap();

        assert_eq!(zsh, my_rc);

        expect_ok(config, &["multirust", "self", "uninstall", "-y"]);
    });
}

// In the default case we want to write $HOME/.cargo/bin as the path,
// not the full path.
#[test]
#[cfg(unix)]
fn when_cargo_home_is_the_default_write_path_specially() {
    setup(&|config, _, home| {
        // Override the test harness so that cargo home looks
        // like $HOME/.cargo, otherwise the literal path will
        // be written to the file
        env::remove_var("CARGO_HOME");

        let my_bashrc = "foo\nbar\nbaz";
        let ref bashrc = home.join(".bashrc");
        raw::write_file(bashrc, my_bashrc).unwrap();
        expect_ok(config, &["multirust-setup", "-y"]);

        let new_bashrc = raw::read_file(bashrc).unwrap();
        let addition = format!(r#"export PATH="$HOME/.cargo/bin:$PATH""#);
        let expected = format!("{}\n{}\n", my_bashrc, addition);
        assert_eq!(new_bashrc, expected);

        expect_ok(config, &["multirust", "self", "uninstall", "-y"]);

        let new_bashrc = raw::read_file(bashrc).unwrap();
        assert_eq!(new_bashrc, my_bashrc);
    });
}

#[cfg(windows)]
fn get_path() -> String {
    use winreg::RegKey;
    use winapi::*;

    let root = RegKey::predef(HKEY_CURRENT_USER);
    let environment = root.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE).unwrap();

    environment.get_value("PATH").unwrap()
}

#[cfg(windows)]
fn restore_path(p: &str) {
    use winreg::RegKey;
    use winapi::*;

    let root = RegKey::predef(HKEY_CURRENT_USER);
    let environment = root.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE).unwrap();

    environment.set_value("PATH", &p).unwrap();
}

#[cfg(unix)]
fn get_path() -> String { String::new() }

#[cfg(unix)]
fn restore_path(_: &str) { }

#[test]
#[cfg(windows)]
fn install_adds_path() {
    setup(&|config, cargo_home, _| {
        expect_ok(config, &["multirust-setup", "-y"]);

        let path = cargo_home.join("bin").to_string_lossy().to_string();
        assert!(get_path().contains(&path));
    });
}

#[test]
#[cfg(windows)]
fn install_does_not_add_path_twice() {
    setup(&|config, cargo_home, _| {
        expect_ok(config, &["multirust-setup", "-y"]);
        expect_ok(config, &["multirust-setup", "-y"]);

        let path = cargo_home.join("bin").to_string_lossy().to_string();
        assert_eq!(get_path().matches(&path).count(), 1);
    });
}

#[test]
#[cfg(windows)]
fn uninstall_removes_path() {
    setup(&|config, cargo_home, _| {
        expect_ok(config, &["multirust-setup", "-y"]);
        expect_ok(config, &["multirust", "self", "uninstall", "-y"]);

        let path = cargo_home.join("bin").to_string_lossy().to_string();
        assert!(!get_path().contains(&path));
    });
}

#[test]
fn update_exact() {
    update_setup(&|config, _, _| {
        expect_ok(config, &["multirust-setup", "-y"]);
        expect_ok_ex(config, &["multirust", "self", "update"],
r"",
r"info: checking for updates
info: downloading update
info: multirust updated successfully
");
    });
}

#[test]
fn update_but_not_installed() {
    update_setup(&|config, cargo_home, _| {
        expect_err_ex(config, &["multirust", "self", "update"],
r"",
&format!(
r"error: multirust is not installed at '{}'
", cargo_home.display()));
    });
}

#[test]
fn update_but_delete_existing_updater_first() {
    update_setup(&|config, cargo_home, _| {
        // The updater is stored in a known location
        let ref setup = cargo_home.join(&format!("bin/multirust-setup{}", EXE_SUFFIX));

        expect_ok(config, &["multirust-setup", "-y"]);

        // If it happens to already exist for some reason it
        // should just be deleted.
        raw::write_file(setup, "").unwrap();
        expect_ok(config, &["multirust", "self", "update"]);

        let multirust = cargo_home.join(&format!("bin/multirust{}", EXE_SUFFIX));
        assert!(multirust.exists());
    });
}

#[test]
fn update_no_change() {
    update_setup(&|config, _, self_dist| {
        expect_ok(config, &["multirust-setup", "-y"]);

        let ref trip = this_host_triple();
        let ref dist_dir = self_dist.join(&format!("{}", trip));
        let ref dist_exe = dist_dir.join(&format!("multirust-setup{}", EXE_SUFFIX));
        let ref dist_hash = dist_dir.join(&format!("multirust-setup{}.sha256", EXE_SUFFIX));
        let ref multirust_bin = config.exedir
            .path().join(&format!("multirust{}", EXE_SUFFIX));
        fs::copy(multirust_bin, dist_exe).unwrap();
        create_hash(dist_exe, dist_hash);

        expect_ok_ex(config, &["multirust", "self", "update"],
r"",
r"info: checking for updates
info: multirust is already up to date
");

    });
}

#[test]
fn update_bad_hash() {
    update_setup(&|config, _, self_dist| {
        expect_ok(config, &["multirust-setup", "-y"]);

        let ref trip = this_host_triple();
        let ref dist_dir = self_dist.join(&format!("{}", trip));
        let ref dist_hash = dist_dir.join(&format!("multirust-setup{}.sha256", EXE_SUFFIX));

        let ref some_other_file = config.distdir.path().join("dist/channel-rust-nightly.toml");

        create_hash(some_other_file, dist_hash);

        expect_err(config, &["multirust", "self", "update"],
                   "checksum failed");
    });
}

#[test]
fn update_hash_file_404() {
    update_setup(&|config, _, self_dist| {
        expect_ok(config, &["multirust-setup", "-y"]);

        let ref trip = this_host_triple();
        let ref dist_dir = self_dist.join(&format!("{}", trip));
        let ref dist_hash = dist_dir.join(&format!("multirust-setup{}.sha256", EXE_SUFFIX));

        fs::remove_file(dist_hash).unwrap();

        expect_err(config, &["multirust", "self", "update"],
                   "could not download file");
    });
}

#[test]
fn update_download_404() {
    update_setup(&|config, _, self_dist| {
        expect_ok(config, &["multirust-setup", "-y"]);

        let ref trip = this_host_triple();
        let ref dist_dir = self_dist.join(&format!("{}", trip));
        let ref dist_exe = dist_dir.join(&format!("multirust-setup{}", EXE_SUFFIX));

        fs::remove_file(dist_exe).unwrap();

        expect_err(config, &["multirust", "self", "update"],
                   "could not download file");
    });
}

// Check that multirust.exe has changed after the update. This
// is hard for windows because the running process needs to exit
// before the new updater can delete it.
#[test]
fn update_updates_multirust_bin() {
    update_setup(&|config, cargo_home, _| {
        expect_ok(config, &["multirust-setup", "-y"]);

        let ref bin = cargo_home.join(&format!("bin/multirust{}", EXE_SUFFIX));
        let before_hash = calc_hash(bin);

        // Running the self update command on the installed binary,
        // so that the running binary must be replaced.
        let mut cmd = Command::new(bin.clone());
        cmd.args(&["self", "update"]);
        clitools::env(config, &mut cmd);
        let out = cmd.output().unwrap();

        println!("out: {}", String::from_utf8(out.stdout).unwrap());
        println!("err: {}", String::from_utf8(out.stderr).unwrap());

        assert!(out.status.success());

        let after_hash = calc_hash(bin);

        assert!(before_hash != after_hash);
    });
}

// Because self-delete on windows is hard, multirust-setup doesn't
// do it. It instead leaves itself installed for cleanup by later
// invocations of multirust.
#[test]
fn updater_leaves_itself_for_later_deletion() {
    update_setup(&|config, cargo_home, _| {
        expect_ok(config, &["multirust-setup", "-y"]);
        expect_ok(config, &["multirust", "update", "nightly"]);
        expect_ok(config, &["multirust", "self", "update"]);

        let setup = cargo_home.join(&format!("bin/multirust-setup{}", EXE_SUFFIX));
        assert!(setup.exists());
    });
}

#[test]
fn updater_is_deleted_after_running_multirust() {
    update_setup(&|config, cargo_home, _| {
        expect_ok(config, &["multirust-setup", "-y"]);
        expect_ok(config, &["multirust", "update", "nightly"]);
        expect_ok(config, &["multirust", "self", "update"]);

        expect_ok_ex(config, &["multirust", "update", "nightly"],
r"
nightly revision:

1.3.0 (hash-n-2)
1.3.0 (hash-n-2)

",
r"info: updating existing install for 'nightly'
info: downloading toolchain manifest
info: toolchain is already up to date
");

        let setup = cargo_home.join(&format!("bin/multirust-setup{}", EXE_SUFFIX));
        assert!(!setup.exists());
    });
}

#[test]
fn updater_is_deleted_after_running_rustc() {
    update_setup(&|config, cargo_home, _| {
        expect_ok(config, &["multirust-setup", "-y"]);
        expect_ok(config, &["multirust", "default", "nightly"]);
        expect_ok(config, &["multirust", "self", "update"]);

        expect_ok(config, &["rustc", "--version"]);

        let setup = cargo_home.join(&format!("bin/multirust-setup{}", EXE_SUFFIX));
        assert!(!setup.exists());
    });
}

#[test]
fn multirust_still_works_after_update() {
    update_setup(&|config, _, _| {
        expect_ok(config, &["multirust-setup", "-y"]);
        expect_ok(config, &["multirust", "default", "nightly"]);
        expect_ok(config, &["multirust", "self", "update"]);
        expect_stdout_ok(config, &["rustc", "--version"], "hash-n-2");
        expect_ok(config, &["multirust", "default", "beta"]);
        expect_stdout_ok(config, &["rustc", "--version"], "hash-b-2");
    });
}

// There's a race condition between the updater replacing
// the multirust binary and tool hardlinks and subsequent
// invocations of multirust and rustc (on windows).
#[test]
#[ignore]
fn update_stress_test() {
}

#[test]
fn first_install_exact() {
    setup(&|config, _, _| {
        expect_ok_ex(config, &["multirust-setup", "-y"],
r"
stable revision:

1.1.0 (hash-s-2)
1.1.0 (hash-s-2)

",
r"info: installing toolchain 'stable'
info: downloading toolchain manifest
info: downloading component 'rust-std'
info: downloading component 'rustc'
info: downloading component 'cargo'
info: downloading component 'rust-docs'
info: installing component 'rust-std'
info: installing component 'rustc'
info: installing component 'cargo'
info: installing component 'rust-docs'
info: toolchain 'stable' installed
info: default toolchain set to 'stable'
"
                  );
    });
}

#[test]
fn reinstall_exact() {
    setup(&|config, _, _| {
        expect_ok(config, &["multirust-setup", "-y"]);
        expect_ok_ex(config, &["multirust-setup", "-y"],
r"
stable revision:

1.1.0 (hash-s-2)
1.1.0 (hash-s-2)

",
r"info: updating existing installation
"
                  );
    });
}

#[test]
#[cfg(unix)]
fn produces_env_file_on_unix() {
    setup(&|config, _, home| {
        // Override the test harness so that cargo home looks
        // like $HOME/.cargo, otherwise the literal path will
        // be written to the file
        env::remove_var("CARGO_HOME");

        expect_ok(config, &["multirust-setup", "-y"]);
        let ref envfile = home.join(".cargo/env");
        let envfile = raw::read_file(envfile).unwrap();
        assert_eq!(r#"export PATH="$HOME/.cargo/bin:$PATH""#, envfile);
    });
}

#[test]
#[cfg(windows)]
fn doesnt_produce_env_file_on_windows() {
}

#[test]
fn install_sets_up_stable() {
    setup(&|config, _, _| {
        expect_ok(config, &["multirust-setup", "-y"]);
        expect_stdout_ok(config, &["rustc", "--version"],
                         "hash-s-2");
    });
}

#[test]
fn install_sets_up_stable_unless_there_is_already_a_default() {
    setup(&|config, _, _| {
        expect_ok(config, &["multirust-setup", "-y"]);
        expect_ok(config, &["multirust", "default", "nightly"]);
        expect_ok(config, &["multirust", "remove-toolchain", "stable"]);
        expect_ok(config, &["multirust-setup", "-y"]);
        expect_stdout_ok(config, &["rustc", "--version"],
                         "hash-n-2");
        expect_err(config, &["multirust", "run", "stable", "rustc", "--version"],
                   "toolchain 'stable' is not installed");
    });
}

// Installation used to be to ~/.multirust/bin instead of
// ~/.cargo/bin. If those bins exist during installation they
// should be deleted to avoid confusion.
#[test]
#[cfg(unix)]
fn install_deletes_legacy_multirust_bins() {
    setup(&|config, _, _| {
        let ref multirust_bin_dir = config.homedir.path().join("bin");
        fs::create_dir_all(multirust_bin_dir).unwrap();
        let ref multirust_bin = multirust_bin_dir.join("multirust");
        let ref rustc_bin = multirust_bin_dir.join("rustc");
        raw::write_file(multirust_bin, "").unwrap();
        raw::write_file(rustc_bin, "").unwrap();
        assert!(multirust_bin.exists());
        assert!(rustc_bin.exists());
        expect_ok(config, &["multirust-setup", "-y"]);
        assert!(!multirust_bin.exists());
        assert!(!rustc_bin.exists());
    });
}

// Installation used to be to
// C:\Users\brian\AppData\Local\.multirust\bin
// instead of C:\Users\brian\.cargo\bin, etc.
#[test]
#[cfg(windows)]
#[ignore]
fn install_deletes_legacy_multirust_bins() {
    // This is untestable on Windows because the definiton multirust-rs
    // used for home couldn't be overridden and isn't on the same
    // code path as std::env::home.

    // This is where windows is considering $HOME:
    // windows::get_special_folder(&windows::FOLDERID_Profile).unwrap();
}

// multirust-setup obeys CARGO_HOME, which multirust-rs *used* to set
// before installation moved from ~/.multirust/bin to ~/.cargo/bin.
// If installation running under the old multirust via `cargo run`,
// then CARGO_HOME will be set during installation, causing the
// install to go to the wrong place. Detect this scenario specifically
// and avoid it.
#[test]
fn legacy_upgrade_installs_to_correct_location() {
    setup(&|config, _, home| {
        let fake_cargo = config.homedir.path().join(".multirust/cargo");
        env::set_var("CARGO_HOME", format!("{}", fake_cargo.display()));
        expect_ok(config, &["multirust-setup", "-y"]);
        let multirust = home.join(&format!(".cargo/bin/multirust{}", EXE_SUFFIX));
        assert!(multirust.exists());
    });
}



#[test]
fn readline_no_stdin() {
    setup(&|config, _, _| {
        expect_err(config, &["multirust-setup"],
                   "unable to read from stdin for confirmation");
    });
}
