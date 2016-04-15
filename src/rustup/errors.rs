use std::path::{Path, PathBuf};
use std::error;
use std::fmt::{self, Display};
use std::io;

use rustup_dist::{self, temp};
use rustup_utils;
use rustup_utils::notify::{self, NotificationLevel, Notifyable};
use rustup_dist::manifest::Component;

#[derive(Debug)]
pub enum Notification<'a> {
    Install(rustup_dist::Notification<'a>),
    Utils(rustup_utils::Notification<'a>),
    Temp(temp::Notification<'a>),

    SetDefaultToolchain(&'a str),
    SetOverrideToolchain(&'a Path, &'a str),
    LookingForToolchain(&'a str),
    ToolchainDirectory(&'a Path, &'a str),
    UpdatingToolchain(&'a str),
    InstallingToolchain(&'a str),
    InstalledToolchain(&'a str),
    UsingExistingToolchain(&'a str),
    UninstallingToolchain(&'a str),
    UninstalledToolchain(&'a str),
    ToolchainNotInstalled(&'a str),
    UpdateHashMatches,
    UpgradingMetadata(&'a str, &'a str),
    MetadataUpgradeNotNeeded(&'a str),
    WritingMetadataVersion(&'a str),
    ReadMetadataVersion(&'a str),
    NonFatalError(&'a Error),
    UpgradeRemovesToolchains,
    MissingFileDuringSelfUninstall(PathBuf),
}

#[derive(Debug)]
pub enum Error {
    Install(rustup_dist::Error),
    Utils(rustup_utils::Error),
    Temp(temp::Error),

    UnknownMetadataVersion(String),
    InvalidEnvironment,
    NoDefaultToolchain,
    PermissionDenied,
    ToolchainNotInstalled(String),
    UnknownHostTriple,
    InfiniteRecursion,
    NeedMetadataUpgrade,
    UpgradeIoError(io::Error),
    BadInstallerType(String),
    ComponentsUnsupported(String),
    UnknownComponent(String, Component),
    AddingRequiredComponent(String, Component),
    RemovingRequiredComponent(String, Component),
    NoExeName,
    NotSelfInstalled(PathBuf),
    CantSpawnWindowsGcExe,
    WindowsUninstallMadness(io::Error),
    SelfUpdateFailed,
    ReadStdin,
    Custom {
        id: String,
        desc: String,
    },
}

pub type Result<T> = ::std::result::Result<T, Error>;
pub type NotifyHandler<'a> = notify::NotifyHandler<'a, for<'b> Notifyable<Notification<'b>>>;
pub type SharedNotifyHandler = notify::SharedNotifyHandler<for<'b> Notifyable<Notification<'b>>>;

extend_error!(Error: rustup_dist::Error, e => Error::Install(e));
extend_error!(Error: rustup_utils::Error, e => Error::Utils(e));
extend_error!(Error: temp::Error, e => Error::Temp(e));

extend_notification!(Notification: rustup_dist::Notification, n => Notification::Install(n));
extend_notification!(Notification: rustup_utils::Notification, n => Notification::Utils(n));
extend_notification!(Notification: temp::Notification, n => Notification::Temp(n));

impl<'a> Notification<'a> {
    pub fn level(&self) -> NotificationLevel {
        use self::Notification::*;
        match *self {
            Install(ref n) => n.level(),
            Utils(ref n) => n.level(),
            Temp(ref n) => n.level(),
            ToolchainDirectory(_, _) |
            LookingForToolchain(_) |
            WritingMetadataVersion(_) |
            InstallingToolchain(_) |
            UpdatingToolchain(_) |
            ReadMetadataVersion(_) |
            InstalledToolchain(_) |
            UpdateHashMatches => NotificationLevel::Verbose,
            SetDefaultToolchain(_) |
            SetOverrideToolchain(_, _) |
            UsingExistingToolchain(_) |
            UninstallingToolchain(_) |
            UninstalledToolchain(_) |
            ToolchainNotInstalled(_) |
            UpgradingMetadata(_, _) |
            MetadataUpgradeNotNeeded(_)  => NotificationLevel::Info,
            NonFatalError(_) => NotificationLevel::Error,
            UpgradeRemovesToolchains |
            MissingFileDuringSelfUninstall(_) => NotificationLevel::Warn,
        }
    }
}

impl<'a> Display for Notification<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> ::std::result::Result<(), fmt::Error> {
        use self::Notification::*;
        match *self {
            Install(ref n) => n.fmt(f),
            Utils(ref n) => n.fmt(f),
            Temp(ref n) => n.fmt(f),
            SetDefaultToolchain(name) => write!(f, "default toolchain set to '{}'", name),
            SetOverrideToolchain(path, name) => {
                write!(f,
                       "override toolchain for '{}' set to '{}'",
                       path.display(),
                       name)
            }
            LookingForToolchain(name) => write!(f, "looking for installed toolchain '{}'", name),
            ToolchainDirectory(path, _) => write!(f, "toolchain directory: '{}'", path.display()),
            UpdatingToolchain(name) => write!(f, "updating existing install for '{}'", name),
            InstallingToolchain(name) => write!(f, "installing toolchain '{}'", name),
            InstalledToolchain(name) => write!(f, "toolchain '{}' installed", name),
            UsingExistingToolchain(name) => write!(f, "using existing install for '{}'", name),
            UninstallingToolchain(name) => write!(f, "uninstalling toolchain '{}'", name),
            UninstalledToolchain(name) => write!(f, "toolchain '{}' uninstalled", name),
            ToolchainNotInstalled(name) => write!(f, "no toolchain installed for '{}'", name),
            UpdateHashMatches => {
                write!(f, "toolchain is already up to date")
            }
            UpgradingMetadata(from_ver, to_ver) => {
                write!(f,
                       "upgrading metadata version from '{}' to '{}'",
                       from_ver,
                       to_ver)
            }
            MetadataUpgradeNotNeeded(ver) => {
                write!(f,
                       "nothing to upgrade: metadata version is already '{}'",
                       ver)
            }
            WritingMetadataVersion(ver) => write!(f, "writing metadata version: '{}'", ver),
            ReadMetadataVersion(ver) => write!(f, "read metadata version: '{}'", ver),
            NonFatalError(e) => write!(f, "{}", e),
            UpgradeRemovesToolchains => write!(f, "this upgrade will remove all existing toolchains. you will need to reinstall them"),
            MissingFileDuringSelfUninstall(ref p) => {
                write!(f, "expected file does not exist to uninstall: {}", p.display())
            }
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        use self::Error::*;
        match *self {
            Install(ref e) => error::Error::description(e),
            Utils(ref e) => error::Error::description(e),
            Temp(ref e) => error::Error::description(e),
            UnknownMetadataVersion(_) => "unknown metadata version",
            InvalidEnvironment => "invalid environment",
            NoDefaultToolchain => "no default toolchain configured",
            PermissionDenied => "permission denied",
            ToolchainNotInstalled(_) => "toolchain is not installed",
            UnknownHostTriple => "unknown host triple",
            InfiniteRecursion =>  "infinite recursion detected",
            NeedMetadataUpgrade => "rustup's metadata is out of date. run `rustup self upgrade-data`",
            UpgradeIoError(_) => "I/O error during upgrade",
            BadInstallerType(_) => "invalid extension for installer",
            ComponentsUnsupported(_) => "toolchain does not support componentsn",
            UnknownComponent(_ ,_) => "toolchain does not contain component",
            AddingRequiredComponent(_, _) => "required component cannot be added",
            RemovingRequiredComponent(_, _) => "required component cannot be removed",
            NoExeName => "couldn't determine self executable name",
            NotSelfInstalled(_) => "rustup is not installed",
            CantSpawnWindowsGcExe => "failed to spawn cleanup process",
            WindowsUninstallMadness(_) => "failure during windows uninstall",
            SelfUpdateFailed => "self-updater failed to replace multirust executable",
            ReadStdin => "unable to read from stdin for confirmation",
            Custom { ref desc, .. } => desc,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        use Error::*;
        match *self {
            Install(ref e) => Some(e),
            Utils(ref e) => Some(e),
            Temp(ref e) => Some(e),
            UpgradeIoError(ref e) => Some(e),
            WindowsUninstallMadness(ref e) => Some(e),
            UnknownMetadataVersion(_) |
            InvalidEnvironment |
            NoDefaultToolchain |
            PermissionDenied |
            ToolchainNotInstalled(_) |
            UnknownHostTriple |
            InfiniteRecursion |
            NeedMetadataUpgrade |
            BadInstallerType(_) |
            ComponentsUnsupported(_) |
            UnknownComponent(_, _) |
            AddingRequiredComponent(_, _) |
            RemovingRequiredComponent(_, _) |
            NoExeName |
            NotSelfInstalled(_) |
            CantSpawnWindowsGcExe |
            SelfUpdateFailed |
            ReadStdin |
            Custom {..} => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> ::std::result::Result<(), fmt::Error> {
        use std::error::Error;
        use self::Error::*;
        match *self {
            Install(ref n) => n.fmt(f),
            Utils(ref n) => n.fmt(f),
            Temp(ref n) => n.fmt(f),
            UnknownMetadataVersion(ref ver) => write!(f, "unknown metadata version: '{}'", ver),
            InvalidEnvironment => write!(f, "invalid environment"),
            NoDefaultToolchain => write!(f, "no default toolchain configured"),
            PermissionDenied => write!(f, "permission denied"),
            ToolchainNotInstalled(ref name) => write!(f, "toolchain '{}' is not installed", name),
            UnknownHostTriple => write!(f, "unknown host triple"),
            InfiniteRecursion => {
                write!(f,
                       "infinite recursion detected: the command may not exist for this toolchain")
            }
            NeedMetadataUpgrade => write!(f, "{}", self.description()),
            UpgradeIoError(ref e) => {
                write!(f, "I/O error during upgrade: {}", e.description())
            }
            BadInstallerType(ref s) => {
                write!(f, "invalid extension for installer: '{}'", s)
            }
            ComponentsUnsupported(ref t) => {
                write!(f, "toolchain '{}' does not support components", t)
            }
            UnknownComponent(ref t, ref c) => {
                write!(f, "toolchain '{}' does not contain component '{}' for target '{}'", t, c.pkg, c.target)
            }
            AddingRequiredComponent(ref t, ref c) => {
                write!(f, "component '{}' for target '{}' is required for toolchain '{}' and cannot be re-added",
                       c.pkg, c.target, t)
            }
            RemovingRequiredComponent(ref t, ref c) => {
                write!(f, "component '{}' for target '{}' is required for toolchain '{}' and cannot be removed",
                       c.pkg, c.target, t)
            }
            NoExeName => write!(f, "couldn't determine self executable name"),
            NotSelfInstalled(ref p) => {
                write!(f, "rustup is not installed at '{}'", p.display())
            }
            CantSpawnWindowsGcExe => write!(f, "{}", self.description()),
            WindowsUninstallMadness(ref e) => write!(f, "failure during windows uninstall: {}", e),
            SelfUpdateFailed => write!(f, "{}", self.description()),
            ReadStdin => write!(f, "{}", self.description()),
            Custom { ref desc, .. } => write!(f, "{}", desc),
        }
    }
}
