#![allow(clippy::large_enum_variant)]
#![allow(deprecated)] // because of `Error::description` deprecation in `error_chain`

use crate::dist::dist::Profile;
use crate::dist::manifest::{Component, Manifest};
use crate::dist::temp;
use crate::{component_for_bin, Toolchain};
use error_chain::error_chain;
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::PathBuf;
use url::Url;

pub const TOOLSTATE_MSG: &str =
    "If you require these components, please install and use the latest successful build version,\n\
     which you can find at <https://rust-lang.github.io/rustup-components-history>.\n\nAfter determining \
     the correct date, install it with a command such as:\n\n    \
     rustup toolchain install nightly-2018-12-27\n\n\
     Then you can use the toolchain with commands such as:\n\n    \
     cargo +nightly-2018-12-27 build";

error_chain! {
    links {
        Download(download::Error, download::ErrorKind);
    }

    foreign_links {
        Temp(temp::Error);
        Io(io::Error);
        Open(opener::OpenError);
        Thread(std::sync::mpsc::RecvError);
    }

    errors {
        LocatingWorkingDir {
            description("Unable to proceed. Could not locate working directory.")
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
        InvalidProfile(t: String) {
            description("invalid profile name")
            display("invalid profile name: '{}'; valid names are: {}", t, valid_profile_names())
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
        SignatureVerificationInternalError(msg: String) {
            description("internal error verifying signature")
            display("internal error verifying signature: {}", msg)
        }
        SignatureVerificationFailed {
            url: String,
        } {
            description("signature verification failed")
            display("signature verification failed for {}", url)
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
        MissingReleaseForToolchain(name: String) {
            description("missing release for a toolchain")
            display("no release found for '{}'", name)
        }
        RequestedComponentsUnavailable(c: Vec<Component>, manifest: Manifest, toolchain: String) {
            description("some requested components are unavailable to download")
            display("{}", component_unavailable_msg(&c, &manifest, &toolchain))
        }
        UnknownMetadataVersion(v: String) {
            description("unknown metadata version")
            display("unknown metadata version: '{}'", v)
        }
        ToolchainNotInstalled(t: String) {
            description("toolchain is not installed")
            display("toolchain '{}' is not installed", t)
        }
        ToolchainNotInstallable(t: String) {
            description("toolchain is not installable")
            display("toolchain '{}' is not installable", t)
        }
        ToolchainNotSelected {
            description("toolchain is not selected")
            display("no override and no default toolchain set; run 'rustup default stable' to set the stable toolchain as default")
        }
        OverrideToolchainNotInstalled(t: String) {
            description("override toolchain is not installed")
            display("override toolchain '{}' is not installed", t)
        }
        BinaryNotFound(bin: String, t: String, is_default: bool) {
            description("toolchain does not contain binary")
            display("'{}' is not installed for the toolchain '{}'{}", bin, t, install_msg(bin, t, *is_default))
        }
        BinaryProvidedByUnavailableComponent(component: String, bin: String, toolchain: String) {
            description("binary is provided by a component which is not available in current toolchain")
            display("the '{}' component which provides the command '{}' is not available for the '{}' toolchain", component, bin, toolchain)
        }
        BinaryNotProvidedByComponent(component: String, bin: String, toolchain: String) {
            description("binary should be provided by component but isn't in current toolchain")
            display("the '{}' binary, normally provided by the '{}' component, is not applicable to the '{}' toolchain", bin, component, toolchain)
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
        UnknownComponent(t: String, c: String, s: Option<String>) {
            description("toolchain does not contain component")
            display("toolchain '{}' does not contain component {}{}", t, c, if let Some(suggestion) = s {
                format!("; did you mean '{}'?", suggestion)
            } else {
                "".to_string()
            })
        }
        UnknownProfile(p: String) {
            description("unknown profile name")
            display(
                "unknown profile name: '{}'; valid profile names are {}",
                p,
                valid_profile_names(),
            )
        }
        AddingRequiredComponent(t: String, c: String) {
            description("required component cannot be added")
            display("component {} was automatically added because it is required for toolchain '{}'",
                    c, t)
        }
        ParsingFallbackSettings(e: toml::de::Error) {
            description("error parsing settings")
        }
        ParsingSettings(e: toml::de::Error) {
            description("error parsing settings")
        }
        NoExeName {
            description("couldn't determine self executable name")
        }
        UnsupportedKind(v: String) {
            description("unsupported tar entry")
            display("tar entry kind '{}' is not supported", v)
        }
        BadPath(v: PathBuf) {
            description("bad path in tar")
            display("tar path '{}' is not supported", v.display())
        }
        InvalidPgpKey(v: PathBuf, error: pgp::errors::Error) {
            description("invalid PGP key"),
            display("unable to read the PGP key '{}'", v.display())
        }
        BrokenPartialFile {
            description("partially downloaded file may have been damaged and was removed, please try again")
        }
        EmptyOverrideFile {
            description("empty toolchain override file detected. Please remove it, or else specify the desired toolchain properties in the file")
        }
        InvalidOverrideFile {
            description("missing toolchain properties in toolchain override file")
        }
        ParsingOverrideFile(e: toml::de::Error) {
            description("error parsing override file")
        }
    }
}

fn valid_profile_names() -> String {
    Profile::names()
        .iter()
        .map(|s| format!("'{}'", s))
        .collect::<Vec<_>>()
        .join(", ")
}

fn remove_component_msg(cs: &Component, manifest: &Manifest, toolchain: &str) -> String {
    if cs.short_name_in_manifest() == "rust-std" {
        // We special-case rust-std as it's the stdlib so really you want to do
        // rustup target remove
        format!(
            "    rustup target remove --toolchain {} {}",
            toolchain,
            cs.target.as_deref().unwrap_or(toolchain)
        )
    } else {
        format!(
            "    rustup component remove --toolchain {}{} {}",
            toolchain,
            if let Some(target) = cs.target.as_ref() {
                format!(" --target {}", target)
            } else {
                String::default()
            },
            cs.short_name(manifest)
        )
    }
}

fn component_unavailable_msg(cs: &[Component], manifest: &Manifest, toolchain: &str) -> String {
    assert!(!cs.is_empty());

    let mut buf = vec![];

    if cs.len() == 1 {
        let _ = writeln!(
            buf,
            "component {} is unavailable for download for channel '{}'",
            &cs[0].description(manifest),
            toolchain,
        );
        if toolchain.starts_with("nightly") {
            let _ = write!(
                buf,
                "Sometimes not all components are available in any given nightly. "
            );
        }
        let _ = write!(
            buf,
            "If you don't need the component, you can remove it with:\n\n{}",
            remove_component_msg(&cs[0], manifest, toolchain)
        );
    } else {
        // More than one component

        let same_target = cs
            .iter()
            .all(|c| c.target == cs[0].target || c.target.is_none());

        let cs_str = if same_target {
            cs.iter()
                .map(|c| format!("'{}'", c.short_name(manifest)))
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            cs.iter()
                .map(|c| c.description(manifest))
                .collect::<Vec<_>>()
                .join(", ")
        };

        let remove_msg = cs
            .iter()
            .map(|c| remove_component_msg(c, manifest, toolchain))
            .collect::<Vec<_>>()
            .join("\n");
        let _ = write!(
            buf,
            "some components unavailable for download for channel '{}': {}\n\
            If you don't need the components, you can remove them with:\n\n{}\n\n{}",
            toolchain, cs_str, remove_msg, TOOLSTATE_MSG,
        );
    }

    String::from_utf8(buf).unwrap()
}

fn install_msg(bin: &str, toolchain: &str, is_default: bool) -> String {
    if Toolchain::is_custom_name(toolchain) {
        return "\nnote: this is a custom toolchain, which cannot use `rustup component add`\n\
        help: if you built this toolchain from source, and used `rustup toolchain link`, then you may be able to build the component with `x.py`".to_string();
    }
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
