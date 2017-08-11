//! Mocks for testing

extern crate url;
#[macro_use]
extern crate lazy_static;
extern crate scopeguard;
extern crate walkdir;
extern crate flate2;
extern crate xz2;
extern crate tempdir;
extern crate itertools;
extern crate tar;
extern crate toml;
extern crate rustup_utils;
extern crate sha2;
extern crate wait_timeout;

#[cfg(windows)]
extern crate winapi;
#[cfg(windows)]
extern crate winreg;

pub mod dist;
pub mod clitools;

use std::fs::{self, OpenOptions, File};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

// Mock of the on-disk structure of rust-installer installers
#[derive(PartialEq, Eq, Hash)]
pub struct MockInstallerBuilder {
    pub components: Vec<MockComponent>,
}

// A component name, the installation commands for installing files
// (either "file:" or "dir:") and the file paths, contents and X bit.
pub type MockComponent = (String, Vec<MockCommand>, Vec<(String, Arc<Vec<u8>>, bool)>);

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum MockCommand {
    File(String),
    Dir(String)
}

impl MockInstallerBuilder {
    pub fn build(&self, path: &Path) {
        for &(ref name, ref commands, ref files) in &self.components {
            // Update the components file
            let comp_file = path.join("components");
            let ref mut comp_file = OpenOptions::new().write(true).append(true).create(true)
                .open(comp_file.clone()).unwrap();
            writeln!(comp_file, "{}", name).unwrap();

            // Create the component directory
            let component_dir = path.join(name);
            if !component_dir.exists() {
                fs::create_dir(&component_dir).unwrap();
            }

            // Create the component manifest
            let ref mut manifest = File::create(component_dir.join("manifest.in")).unwrap();
            for command in commands {
                match command {
                    &MockCommand::File(ref f) => writeln!(manifest, "file:{}", f).unwrap(),
                    &MockCommand::Dir(ref d) => writeln!(manifest, "dir:{}", d).unwrap(),
                }
            }

            // Create the component files
            for &(ref f_path, ref content, executable) in files {
                let fname = component_dir.join(f_path);
                let dir_path = fname.parent().unwrap().to_owned();
                fs::create_dir_all(dir_path).unwrap();
                let ref mut f = File::create(&fname).unwrap();

                f.write_all(&content).unwrap();
                drop(f);
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if executable {
                        let mut perm = fs::metadata(&fname).unwrap().permissions();
                        perm.set_mode(0o755);
                        fs::set_permissions(&fname, perm).unwrap();
                    }
                }
            }
        }

        let mut ver = File::create(path.join("rust-installer-version")).unwrap();
        writeln!(ver, "3").unwrap();
    }
}

#[cfg(windows)]
pub fn get_path() -> Option<String> {
    use winreg::RegKey;
    use winapi::*;

    let root = RegKey::predef(HKEY_CURRENT_USER);
    let environment = root.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE).unwrap();

    environment.get_value("PATH").ok()
}

#[cfg(windows)]
pub fn restore_path(p: &Option<String>) {
    use winreg::{RegKey, RegValue};
    use winreg::enums::RegType;
    use winapi::*;

    let root = RegKey::predef(HKEY_CURRENT_USER);
    let environment = root.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE).unwrap();

    if let Some(p) = p.as_ref() {
        let reg_value = RegValue {
            bytes: string_to_winreg_bytes(&p),
            vtype: RegType::REG_EXPAND_SZ,
        };
        environment.set_raw_value("PATH", &reg_value).unwrap();
    } else {
        let _ = environment.delete_value("PATH");
    }

    fn string_to_winreg_bytes(s: &str) -> Vec<u8> {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStrExt;
        let v: Vec<_> = OsString::from(format!("{}\x00", s)).encode_wide().collect();
        unsafe { ::std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * 2).to_vec() }
    }
}

#[cfg(unix)]
pub fn get_path() -> Option<String> { None }

#[cfg(unix)]
pub fn restore_path(_: &Option<String>) { }

