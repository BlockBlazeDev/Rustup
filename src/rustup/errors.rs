use std::path::PathBuf;

use rustup_dist::{self, temp};
use rustup_utils;
use rustup_dist::manifest::Component;

declare_errors! {
    types {
        Error, ErrorChain, ChainError, Result;
    }

    from_links {
        rustup_dist::ErrorChain, rustup_dist::Error, Dist;
        rustup_utils::ErrorChain, rustup_utils::Error, Utils;
    }

    foreign_links {
        temp::Error, Temp;
    }

    errors {
        UnknownMetadataVersion(v: String) {
            description("unknown metadata version")
            display("unknown metadata version: '{}'", v)
        }
        InvalidEnvironment {
            description("invalid environment")
        }
        NoDefaultToolchain {
            description("no default toolchain configured")
        }
        PermissionDenied {
            description("permission denied")
        }
        ToolchainNotInstalled(t: String) {
            description("toolchain is not installed")
            display("toolchain '{}' is not installed", t)
        }
        UnknownHostTriple {
            description("unknown host triple")
        }
        InfiniteRecursion {
            description("infinite recursion detected")
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
        UnknownComponent(t: String, c: Component) {
            description("toolchain does not contain component")
            display("toolchain '{}' does not contain component '{}' for target '{}'", t, c.pkg, c.target)
        }
        AddingRequiredComponent(t: String, c: Component) {
            description("required component cannot be added")
            display("component '{}' for target '{}' is required for toolchain '{}' and cannot be re-added",
                    c.pkg, c.target, t)
        }
        RemovingRequiredComponent(t: String, c: Component) {
            description("required component cannot be removed")
            display("component '{}' for target '{}' is required for toolchain '{}' and cannot be removed",
                    c.pkg, c.target, t)
        }
        NoExeName {
            description("couldn't determine self executable name")
        }
        NotSelfInstalled(p: PathBuf) {
            description("rustup is not installed")
            display("rustup is not installed at '{}'", p.display())
        }
        CantSpawnWindowsGcExe {
            description("failed to spawn cleanup process")
        }
        WindowsUninstallMadness {
            description("failure during windows uninstall")
        }
        SelfUpdateFailed {
            description("self-updater failed to replace multirust executable")
        }
        ReadStdin {
            description("unable to read from stdin for confirmation")
        }
        Custom {
            id: String,
            desc: String,
        } {
            description(&desc)
        }
        TelemetryCleanupError {
            description("unable to remove old telemetry files")
        }
    }
}
