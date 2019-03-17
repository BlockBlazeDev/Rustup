use crate::component_for_bin;
use crate::dist::manifest::{Component, Manifest};
use crate::dist::temp;
use error_chain::error_chain;
use error_chain::error_chain_processing;
use error_chain::{impl_error_chain_kind, impl_error_chain_processed, impl_extract_backtrace};
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::PathBuf;
use url::Url;

pub const TOOLSTATE_MSG: &str =
    "if you require these components, please install and use the latest successful build version, \
     which you can find at https://rust-lang-nursery.github.io/rust-toolstate, for example.\n\
     rustup install nightly-2018-12-27";

error_chain! {
    links {
        Download(download::Error, download::ErrorKind);
    }

    foreign_links {
        Temp(temp::Error);
        Io(io::Error);
    }

    errors {
        LocatingWorkingDir {
            description("could not locate working directory")
        }
        ReadingFile {
            name: &'static str,
            path: PathBuf,
        } {
            description("could not read file")
            display("could not read {} file: '{}'", name, path.display())
        }
        ReadingDirectory {
            name: &'static str,
            path: PathBuf,
        } {
            description("could not read directory")
            display("could not read {} directory: '{}'", name, path.display())
        }
        WritingFile {
            name: &'static str,
            path: PathBuf,
        } {
            description("could not write file")
            display("could not write {} file: '{}'", name, path.display())
        }
        CreatingDirectory {
            name: &'static str,
            path: PathBuf,
        } {
            description("could not create directory")
            display("could not create {} directory: '{}'", name, path.display())
        }
        ExpectedType(t: &'static str, n: String) {
            description("expected type")
            display("expected type: '{}' for '{}'", t, n)
        }
        FilteringFile {
            name: &'static str,
            src: PathBuf,
            dest: PathBuf,
        } {
            description("could not copy file")
            display("could not copy {} file from '{}' to '{}'", name, src.display(), dest.display())
        }
        RenamingFile {
            name: &'static str,
            src: PathBuf,
            dest: PathBuf,
        } {
            description("could not rename file")
            display("could not rename {} file from '{}' to '{}'",
                name, src.display(), dest.display())
        }
        RenamingDirectory {
            name: &'static str,
            src: PathBuf,
            dest: PathBuf,
        } {
            description("could not rename directory")
            display("could not rename {} directory from '{}' to '{}'", name, src.display(), dest.display())
        }
        DownloadingFile {
            url: Url,
            path: PathBuf,
        } {
            description("could not download file")
            display("could not download file from '{}' to '{}'", url, path.display())
        }
        DownloadNotExists {
            url: Url,
            path: PathBuf,
        } {
            description("could not download file")
            display("could not download file from '{}' to '{}'", url, path.display())
        }
        InvalidUrl {
            url: String,
        } {
            description("invalid url")
            display("invalid url: {}", url)
        }
        RunningCommand {
            name: OsString,
        } {
            description("command failed")
            display("command failed: '{}'", PathBuf::from(name).display())
        }
        NotAFile {
            path: PathBuf,
        } {
            description("not a file")
            display("not a file: '{}'", path.display())
        }
        NotADirectory {
            path: PathBuf,
        } {
            description("not a directory")
            display("not a directory: '{}'", path.display())
        }
        LinkingFile {
            src: PathBuf,
            dest: PathBuf,
        } {
            description("could not link file")
            display("could not create link from '{}' to '{}'", src.display(), dest.display())
        }
        LinkingDirectory {
            src: PathBuf,
            dest: PathBuf,
        } {
            description("could not symlink directory")
            display("could not create link from '{}' to '{}'", src.display(), dest.display())
        }
        CopyingDirectory {
            src: PathBuf,
            dest: PathBuf,
        } {
            description("could not copy directory")
            display("could not copy directory from '{}' to '{}'", src.display(), dest.display())
        }
        CopyingFile {
            src: PathBuf,
            dest: PathBuf,
        } {
            description("could not copy file")
            display("could not copy file from '{}' to '{}'", src.display(), dest.display())
        }
        RemovingFile {
            name: &'static str,
            path: PathBuf,
        } {
            description("could not remove file")
            display("could not remove '{}' file: '{}'", name, path.display())
        }
        RemovingDirectory {
            name: &'static str,
            path: PathBuf,
        } {
            description("could not remove directory")
            display("could not remove '{}' directory: '{}'", name, path.display())
        }
        SettingPermissions {
            path: PathBuf,
        } {
            description("failed to set permissions")
            display("failed to set permissions for '{}'", path.display())
        }
        GettingCwd {
            description("couldn't get current working directory")
        }
        CargoHome {
            description("couldn't find value of CARGO_HOME")
        }
        RustupHome {
            description("couldn't find value of RUSTUP_HOME")
        }
        InvalidToolchainName(t: String) {
            description("invalid toolchain name")
            display("invalid toolchain name: '{}'", t)
        }
        InvalidCustomToolchainName(t: String) {
            description("invalid custom toolchain name")
            display("invalid custom toolchain name: '{}'", t)
        }
        ChecksumFailed {
            url: String,
            expected: String,
            calculated: String,
        } {
            description("checksum failed")
            display("checksum failed, expected: '{}', calculated: '{}'",
                    expected,
                    calculated)
        }
        ComponentConflict {
            name: String,
            path: PathBuf,
        } {
            description("conflicting component")
            display("failed to install component: '{}', detected conflict: '{:?}'",
                    name,
                    path)
        }
        ComponentMissingFile {
            name: String,
            path: PathBuf,
        } {
            description("missing file in component")
            display("failure removing component '{}', directory does not exist: '{:?}'",
                    name,
                    path)
        }
        ComponentMissingDir {
            name: String,
            path: PathBuf,
        } {
            description("missing directory in component")
            display("failure removing component '{}', directory does not exist: '{:?}'",
                    name,
                    path)
        }
        CorruptComponent(name: String) {
            description("corrupt component manifest")
            display("component manifest for '{}' is corrupt", name)
        }
        ExtractingPackage {
            description("failed to extract package (perhaps you ran out of disk space?)")
        }
        BadInstallerVersion(v: String) {
            description("unsupported installer version")
            display("unsupported installer version: {}", v)
        }
        BadInstalledMetadataVersion(v: String) {
            description("unsupported metadata version in existing installation")
            display("unsupported metadata version in existing installation: {}", v)
        }
        ComponentDirPermissionsFailed {
            description("I/O error walking directory during install")
        }
        ComponentFilePermissionsFailed {
            description("error setting file permissions during install")
        }
        ComponentDownloadFailed(c: String) {
            description("component download failed")
            display("component download failed for {}", c)
        }
        Parsing(e: toml::de::Error) {
            description("error parsing manifest")
        }
        UnsupportedVersion(v: String) {
            description("unsupported manifest version")
            display("manifest version '{}' is not supported", v)
        }
        MissingPackageForComponent(name: String) {
            description("missing package for component")
            display("server sent a broken manifest: missing package for component {}", name)
        }
        MissingPackageForRename(name: String) {
            description("missing package for the target of a rename")
            display("server sent a broken manifest: missing package for the target of a rename {}", name)
        }
        RequestedComponentsUnavailable(c: Vec<Component>, manifest: Manifest, toolchain: String) {
            description("some requested components are unavailable to download")
            display("{} for channel '{}'", component_unavailable_msg(&c, &manifest), toolchain)
        }
        UnknownMetadataVersion(v: String) {
            description("unknown metadata version")
            display("unknown metadata version: '{}'", v)
        }
        ToolchainNotInstalled(t: String) {
            description("toolchain is not installed")
            display("toolchain '{}' is not installed", t)
        }
        OverrideToolchainNotInstalled(t: String) {
            description("override toolchain is not installed")
            display("override toolchain '{}' is not installed", t)
        }
        BinaryNotFound(bin: String, t: String, is_default: bool) {
            description("toolchain does not contain binary")
            display("'{}' is not installed for the toolchain '{}'{}", bin, t, install_msg(bin, t, *is_default))
        }
        NeedMetadataUpgrade {
            description("rustup's metadata is out of date. run `rustup self upgrade-data`")
        }
        UpgradeIoError {
            description("I/O error during upgrade")
        }
        BadInstallerType(s: String) {
            description("invalid extension for installer")
            display("invalid extension for installer: '{}'", s)
        }
        ComponentsUnsupported(t: String) {
            description("toolchain does not support components")
            display("toolchain '{}' does not support components", t)
        }
        UnknownComponent(t: String, c: String) {
            description("toolchain does not contain component")
            display("toolchain '{}' does not contain component {}", t, c)
        }
        AddingRequiredComponent(t: String, c: String) {
            description("required component cannot be added")
            display("component {} was automatically added because it is required for toolchain '{}'",
                    c, t)
        }
        ParsingSettings(e: toml::de::Error) {
            description("error parsing settings")
        }
        RemovingRequiredComponent(t: String, c: String) {
            description("required component cannot be removed")
            display("component {} is required for toolchain '{}' and cannot be removed",
                    c, t)
        }
        NoExeName {
            description("couldn't determine self executable name")
        }
    }
}

fn component_unavailable_msg(cs: &[Component], manifest: &Manifest) -> String {
    assert!(!cs.is_empty());

    let mut buf = vec![];

    if cs.len() == 1 {
        let _ = write!(
            buf,
            "component {} is unavailable for download",
            &cs[0].description(manifest)
        );
    } else {
        use itertools::Itertools;
        let same_target = cs
            .iter()
            .all(|c| c.target == cs[0].target || c.target.is_none());
        if same_target {
            let mut cs_strs = cs.iter().map(|c| format!("'{}'", c.short_name(manifest)));
            let cs_str = cs_strs.join(", ");
            let _ = write!(
                buf,
                "some components unavailable for download: {}\n{}",
                cs_str, TOOLSTATE_MSG,
            );
        } else {
            let mut cs_strs = cs.iter().map(|c| c.description(manifest));
            let cs_str = cs_strs.join(", ");
            let _ = write!(
                buf,
                "some components unavailable for download: {}\n{}",
                cs_str, TOOLSTATE_MSG,
            );
        }
    }

    String::from_utf8(buf).expect("")
}

fn install_msg(bin: &str, toolchain: &str, is_default: bool) -> String {
    match component_for_bin(bin) {
        Some(c) => format!("\nTo install, run `rustup component add {}{}`", c, {
            if is_default {
                String::new()
            } else {
                format!(" --toolchain {}", toolchain)
            }
        }),
        None => String::new(),
    }
}
