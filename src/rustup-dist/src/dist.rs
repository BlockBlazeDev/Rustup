
use temp;
use errors::*;
use notifications::*;
use rustup_utils::{self, utils};
use prefix::InstallPrefix;
use manifest::Component;
use manifest::Manifest as ManifestV2;
use manifestation::{Manifestation, UpdateStatus, Changes};
use download::{DownloadCfg};

use std::path::Path;
use std::fmt;
use std::env;

use regex::Regex;

pub const DEFAULT_DIST_SERVER: &'static str = "https://static.rust-lang.org";

// Deprecated
pub const DEFAULT_DIST_ROOT: &'static str = "https://static.rust-lang.org/dist";

// A toolchain descriptor from rustup's perspective. These contain
// 'partial target triples', which allow toolchain names like
// 'stable-msvc' to work. Partial target triples though are parsed
// from a hardcoded set of known triples, whereas target triples
// are nearly-arbitrary strings.
#[derive(Debug, Clone)]
pub struct PartialToolchainDesc {
    // Either "nightly", "stable", "beta", or an explicit version number
    pub channel: String,
    pub date: Option<String>,
    pub target: PartialTargetTriple,
}

#[derive(Debug, Clone)]
pub struct PartialTargetTriple {
    pub arch: Option<String>,
    pub os: Option<String>,
    pub env: Option<String>,
}

// Fully-resolved toolchain descriptors. These always have full target
// triples attached to them and are used for canonical identification,
// such as naming their installation directory.
#[derive(Debug, Clone)]
pub struct ToolchainDesc {
    // Either "nightly", "stable", "beta", or an explicit version number
    pub channel: String,
    pub date: Option<String>,
    pub target: TargetTriple,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TargetTriple(String);

// These lists contain the targets known to rustup, and used to build
// the PartialTargetTriple.

static LIST_ARCHS: &'static [&'static str] = &["i386",
                                               "i586",
                                               "i686",
                                               "x86_64",
                                               "arm",
                                               "armv7",
                                               "armv7s",
                                               "aarch64",
                                               "mips",
                                               "mipsel",
                                               "mips64",
                                               "mips64el",
                                               "powerpc",
                                               "powerpc64",
                                               "powerpc64le",
                                               "s390x"];
static LIST_OSES: &'static [&'static str] = &["pc-windows",
                                              "unknown-linux",
                                              "apple-darwin",
                                              "unknown-netbsd",
                                              "apple-ios",
                                              "linux",
                                              "rumprun-netbsd",
                                              "unknown-freebsd"];
static LIST_ENVS: &'static [&'static str] =
    &["gnu", "msvc", "gnueabi", "gnueabihf", "gnuabi64", "androideabi", "android", "musl"];

// MIPS platforms don't indicate endianness in uname, however binaries only
// run on boxes with the same endianness, as expected.
// Hence we could distinguish between the variants with compile-time cfg()
// attributes alone.
#[cfg(all(not(windows), target_endian = "big"))]
const TRIPLE_MIPS_UNKNOWN_LINUX_GNU: &'static str = "mips-unknown-linux-gnu";
#[cfg(all(not(windows), target_endian = "little"))]
const TRIPLE_MIPS_UNKNOWN_LINUX_GNU: &'static str = "mipsel-unknown-linux-gnu";

#[cfg(all(not(windows), target_endian = "big"))]
const TRIPLE_MIPS64_UNKNOWN_LINUX_GNUABI64: &'static str =
    "mips64-unknown-linux-gnuabi64";
#[cfg(all(not(windows), target_endian = "little"))]
const TRIPLE_MIPS64_UNKNOWN_LINUX_GNUABI64: &'static str =
    "mips64el-unknown-linux-gnuabi64";

impl TargetTriple {
    pub fn from_str(name: &str) -> Self {
        TargetTriple(name.to_string())
    }

    pub fn from_build() -> Self {
        if let Some(triple) = option_env!("RUSTUP_OVERRIDE_BUILD_TRIPLE") {
            TargetTriple::from_str(triple)
        } else {
            TargetTriple::from_str(include_str!(concat!(env!("OUT_DIR"), "/target.txt")))
        }
    }

    pub fn from_host() -> Option<Self> {
        #[cfg(windows)]
        fn inner() -> Option<TargetTriple> {
            use winapi::um::sysinfoapi::GetNativeSystemInfo;
            use std::mem;

            // First detect architecture
            const PROCESSOR_ARCHITECTURE_AMD64: u16 = 9;
            const PROCESSOR_ARCHITECTURE_INTEL: u16 = 0;

            let mut sys_info;
            unsafe {
                sys_info = mem::zeroed();
                GetNativeSystemInfo(&mut sys_info);
            }

            let arch = match unsafe { sys_info.u.s() }.wProcessorArchitecture {
                PROCESSOR_ARCHITECTURE_AMD64 => "x86_64",
                PROCESSOR_ARCHITECTURE_INTEL => "i686",
                _ => return None,
            };

            // Default to msvc
            let msvc_triple = format!("{}-pc-windows-msvc", arch);
            Some(TargetTriple(msvc_triple))
        }

        #[cfg(not(windows))]
        fn inner() -> Option<TargetTriple> {
            use libc;
            use std::mem;
            use std::ffi::CStr;

            let mut sys_info;
            let (sysname, machine) = unsafe {
                sys_info = mem::zeroed();
                if libc::uname(&mut sys_info) != 0 {
                    return None;
                }

                (CStr::from_ptr(sys_info.sysname.as_ptr()).to_bytes(),
                 CStr::from_ptr(sys_info.machine.as_ptr()).to_bytes())
            };

            let host_triple = match (sysname, machine) {
                (_, b"arm") if cfg!(target_os = "android") => Some("arm-linux-androideabi"),
                (_, b"armv7l") if cfg!(target_os = "android") => Some("armv7-linux-androideabi"),
                (_, b"armv8l") if cfg!(target_os = "android") => Some("armv7-linux-androideabi"),
                (_, b"aarch64") if cfg!(target_os = "android") => Some("aarch64-linux-android"),
                (_, b"i686") if cfg!(target_os = "android") => Some("i686-linux-android"),
                (_, b"x86_64") if cfg!(target_os = "android") => Some("x86_64-linux-android"),
                (b"Linux", b"x86_64") => Some("x86_64-unknown-linux-gnu"),
                (b"Linux", b"i686") => Some("i686-unknown-linux-gnu"),
                (b"Linux", b"mips") => Some(TRIPLE_MIPS_UNKNOWN_LINUX_GNU),
                (b"Linux", b"mips64") => Some(TRIPLE_MIPS64_UNKNOWN_LINUX_GNUABI64),
                (b"Linux", b"arm") => Some("arm-unknown-linux-gnueabi"),
                (b"Linux", b"armv7l") => Some("armv7-unknown-linux-gnueabihf"),
                (b"Linux", b"armv8l") => Some("armv7-unknown-linux-gnueabihf"),
                (b"Linux", b"aarch64") => Some("aarch64-unknown-linux-gnu"),
                (b"Darwin", b"x86_64") => Some("x86_64-apple-darwin"),
                (b"Darwin", b"i686") => Some("i686-apple-darwin"),
                (b"FreeBSD", b"x86_64") => Some("x86_64-unknown-freebsd"),
                (b"FreeBSD", b"i686") => Some("i686-unknown-freebsd"),
                (b"OpenBSD", b"x86_64") => Some("x86_64-unknown-openbsd"),
                (b"OpenBSD", b"i686") => Some("i686-unknown-openbsd"),
                (b"NetBSD", b"x86_64") => Some("x86_64-unknown-netbsd"),
                (b"NetBSD", b"i686") => Some("i686-unknown-netbsd"),
                (b"DragonFly", b"x86_64") => Some("x86_64-unknown-dragonfly"),
                _ => None,
            };

            host_triple.map(TargetTriple::from_str)
        }

        if let Ok(triple) = env::var("RUSTUP_OVERRIDE_HOST_TRIPLE") {
            Some(TargetTriple(triple))
        } else {
            inner()
        }
    }

    pub fn from_host_or_build() -> Self {
        Self::from_host().unwrap_or_else(Self::from_build)
    }
}

impl PartialTargetTriple {
    pub fn from_str(name: &str) -> Option<Self> {
        if name.is_empty() {
            return Some(PartialTargetTriple {
                arch: None,
                os: None,
                env: None,
            });
        }

        // Prepending `-` makes this next regex easier since
        // we can count  on all triple components being
        // delineated by it.
        let name = format!("-{}", name);
        let pattern = format!(r"^(?:-({}))?(?:-({}))?(?:-({}))?$",
                              LIST_ARCHS.join("|"),
                              LIST_OSES.join("|"),
                              LIST_ENVS.join("|"));

        let re = Regex::new(&pattern).unwrap();
        re.captures(&name).map(|c| {
            fn fn_map(s: &str) -> Option<String> {
                if s == "" {
                    None
                } else {
                    Some(s.to_owned())
                }
            }

            PartialTargetTriple {
                arch: c.get(1).map(|s| s.as_str()).and_then(fn_map),
                os: c.get(2).map(|s| s.as_str()).and_then(fn_map),
                env: c.get(3).map(|s| s.as_str()).and_then(fn_map),
            }
        })
    }
}

impl PartialToolchainDesc {
    pub fn from_str(name: &str) -> Result<Self> {
        let channels =
            ["nightly", "beta", "stable", r"\d{1}\.\d{1}\.\d{1}", r"\d{1}\.\d{2}\.\d{1}"];

        let pattern = format!(r"^({})(?:-(\d{{4}}-\d{{2}}-\d{{2}}))?(?:-(.*))?$",
                              channels.join("|"));


        let re = Regex::new(&pattern).unwrap();
        let d = re.captures(name).map(|c| {
            fn fn_map(s: &str) -> Option<String> {
                if s == "" {
                    None
                } else {
                    Some(s.to_owned())
                }
            }

            let trip = c.get(3).map(|c| c.as_str()).unwrap_or("");
            let trip = PartialTargetTriple::from_str(&trip);
            trip.map(|t| {
                PartialToolchainDesc {
                    channel: c.get(1).unwrap().as_str().to_owned(),
                    date: c.get(2).map(|s| s.as_str()).and_then(fn_map),
                    target: t,
                }
            })
        });

        if let Some(Some(d)) = d {
            Ok(d)
        } else {
            Err(ErrorKind::InvalidToolchainName(name.to_string()).into())
        }
    }

    pub fn resolve(self, host: &TargetTriple) -> ToolchainDesc {
        let host = PartialTargetTriple::from_str(&host.0)
            .expect("host triple couldn't be converted to partial triple");
        let host_arch = host.arch.expect("");
        let host_os = host.os.expect("");
        let host_env = host.env;

        // If OS was specified, don't default to host environment, even if the OS matches
        // the host OS, otherwise cannot specify no environment.
        let env = if self.target.os.is_some() {
            self.target.env
        } else {
            self.target.env.or_else(|| host_env)
        };
        let arch = self.target.arch.unwrap_or_else(|| host_arch);
        let os = self.target.os.unwrap_or_else(|| host_os);

        let trip = if let Some(env) = env {
            format!("{}-{}-{}", arch, os, env)
        } else {
            format!("{}-{}", arch, os)
        };

        ToolchainDesc {
            channel: self.channel,
            date: self.date,
            target: TargetTriple(trip),
        }
    }

    pub fn has_triple(&self) -> bool {
        self.target.arch.is_some() || self.target.os.is_some() || self.target.env.is_some()
    }
}

impl ToolchainDesc {
    pub fn from_str(name: &str) -> Result<Self> {
        let channels =
            ["nightly", "beta", "stable", r"\d{1}\.\d{1}\.\d{1}", r"\d{1}\.\d{2}\.\d{1}"];

        let pattern = format!(
            r"^({})(?:-(\d{{4}}-\d{{2}}-\d{{2}}))?-(.*)?$",
            channels.join("|"),
            );

        let re = Regex::new(&pattern).unwrap();
        re.captures(name)
            .map(|c| {
                fn fn_map(s: &str) -> Option<String> {
                    if s == "" {
                        None
                    } else {
                        Some(s.to_owned())
                    }
                }

                ToolchainDesc {
                    channel: c.get(1).unwrap().as_str().to_owned(),
                    date: c.get(2).map(|s| s.as_str()).and_then(fn_map),
                    target: TargetTriple(c.get(3).unwrap().as_str().to_owned()),
                }
            })
            .ok_or(ErrorKind::InvalidToolchainName(name.to_string()).into())
    }

    pub fn manifest_v1_url(&self, dist_root: &str) -> String {
        let do_manifest_staging = env::var("RUSTUP_STAGED_MANIFEST").is_ok();
        match (self.date.as_ref(), do_manifest_staging) {
            (None, false) => format!("{}/channel-rust-{}", dist_root, self.channel),
            (Some(date), false) => format!("{}/{}/channel-rust-{}", dist_root, date, self.channel),
            (None, true) => format!("{}/staging/channel-rust-{}", dist_root, self.channel),
            (Some(_), true) => panic!("not a real-world case"),
        }
    }

    pub fn manifest_v2_url(&self, dist_root: &str) -> String {
        format!("{}.toml", self.manifest_v1_url(dist_root))
    }
    /// Either "$channel" or "channel-$date"
    pub fn manifest_name(&self) -> String {
        match self.date {
            None => self.channel.clone(),
            Some(ref date) => format!("{}-{}", self.channel, date),
        }
    }

    pub fn package_dir(&self, dist_root: &str) -> String {
        match self.date {
            None => format!("{}", dist_root),
            Some(ref date) => format!("{}/{}", dist_root, date),
        }
    }

    pub fn full_spec(&self) -> String {
        if self.date.is_some() {
            format!("{}", self)
        } else {
            format!("{} (tracking)", self)
        }
    }

    pub fn is_tracking(&self) -> bool {
        let channels = ["nightly", "beta", "stable"];
        channels.iter().any(|x| *x == self.channel) && self.date.is_none()
    }
}

// A little convenience for just parsing a channel name or archived channel name
pub fn validate_channel_name(name: &str) -> Result<()> {
    let toolchain = PartialToolchainDesc::from_str(&name)?;
    if toolchain.has_triple() {
        Err(format!("target triple in channel name '{}'", name).into())
    } else {
        Ok(())
    }
}

#[derive(Debug)]
pub struct Manifest<'a>(temp::File<'a>, String);

impl<'a> Manifest<'a> {
    pub fn package_url(&self,
                       package: &str,
                       target_triple: &str,
                       ext: &str)
                       -> Result<Option<String>> {
        let suffix = target_triple.to_owned() + ext;
        Ok(try!(utils::match_file("manifest", &self.0, |line| {
            if line.starts_with(package) && line.ends_with(&suffix) {
                Some(format!("{}/{}", &self.1, line))
            } else {
                None
            }
        })))
    }
}

impl fmt::Display for TargetTriple {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for PartialToolchainDesc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{}", &self.channel));

        if let Some(ref date) = self.date {
            try!(write!(f, "-{}", date));
        }
        if let Some(ref arch) = self.target.arch {
            try!(write!(f, "-{}", arch));
        }
        if let Some(ref os) = self.target.os {
            try!(write!(f, "-{}", os));
        }
        if let Some(ref env) = self.target.env {
            try!(write!(f, "-{}", env));
        }

        Ok(())
    }
}

impl fmt::Display for ToolchainDesc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{}", &self.channel));

        if let Some(ref date) = self.date {
            try!(write!(f, "-{}", date));
        }
        try!(write!(f, "-{}", self.target));

        Ok(())
    }
}


// Installs or updates a toolchain from a dist server. If an initial
// install then it will be installed with the default components. If
// an upgrade then all the existing components will be upgraded.
//
// Returns the manifest's hash if anything changed.
pub fn update_from_dist<'a>(download: DownloadCfg<'a>,
                            update_hash: Option<&Path>,
                            toolchain: &ToolchainDesc,
                            prefix: &InstallPrefix,
                            add: &[Component],
                            remove: &[Component],
                            force_update: bool)
                            -> Result<Option<String>> {

    let fresh_install = !prefix.path().exists();

    let res = update_from_dist_(download,
                                update_hash,
                                toolchain,
                                prefix,
                                add,
                                remove,
                                force_update);

    // Don't leave behind an empty / broken installation directory
    if res.is_err() && fresh_install {
        // FIXME Ignoring cascading errors
        let _ = utils::remove_dir("toolchain", prefix.path(),
                                  &|n| (download.notify_handler)(n.into()));
    }

    res
}

pub fn update_from_dist_<'a>(download: DownloadCfg<'a>,
                             update_hash: Option<&Path>,
                             toolchain: &ToolchainDesc,
                             prefix: &InstallPrefix,
                             add: &[Component],
                             remove: &[Component],
                             force_update: bool)
                             -> Result<Option<String>> {

    let toolchain_str = toolchain.to_string();
    let manifestation = try!(Manifestation::open(prefix.clone(), toolchain.target.clone()));

    let changes = Changes {
        add_extensions: add.to_owned(),
        remove_extensions: remove.to_owned(),
    };

    // TODO: Add a notification about which manifest version is going to be used
    (download.notify_handler)(Notification::DownloadingManifest(&toolchain_str));
    match dl_v2_manifest(download, update_hash, toolchain) {
        Ok(Some((m, hash))) => {
            (download.notify_handler)(Notification::DownloadedManifest(&m.date, m.get_rust_version().ok()));
            return match try!(manifestation.update(&m,
                                                   changes,
                                                   force_update,
                                                   &download,
                                                   download.notify_handler.clone())) {
                UpdateStatus::Unchanged => Ok(None),
                UpdateStatus::Changed => Ok(Some(hash)),
            }
        }
        Ok(None) => return Ok(None),
        Err(Error(ErrorKind::Utils(::rustup_utils::ErrorKind::DownloadNotExists { .. }), _)) => {
            // Proceed to try v1 as a fallback
            (download.notify_handler)(Notification::DownloadingLegacyManifest);
        }
        Err(Error(ErrorKind::ChecksumFailed { .. }, _)) => {
            return Ok(None)
        }
        Err(e) => return Err(e),
    }

    // If the v2 manifest is not found then try v1
    let manifest = match dl_v1_manifest(download, toolchain) {
        Ok(m) => m,
        Err(Error(ErrorKind::Utils(rustup_utils::ErrorKind::DownloadNotExists { .. }), _)) => {
            return Err(format!("no release found for '{}'", toolchain.manifest_name()).into());
        }
        Err(e @ Error(ErrorKind::ChecksumFailed { .. }, _)) => {
            return Err(e);
        }
        Err(e) => {
            return Err(e).chain_err(|| {
                format!("failed to download manifest for '{}'",
                        toolchain.manifest_name())
            });
        }
    };
    match manifestation.update_v1(&manifest,
                                  update_hash,
                                  &download.temp_cfg,
                                  download.notify_handler.clone()) {
        Ok(None) => Ok(None),
        Ok(Some(hash)) => Ok(Some(hash)),
        e @ Err(Error(ErrorKind::Utils(rustup_utils::ErrorKind::DownloadNotExists { .. }), _)) => {
            e.chain_err(|| {
                format!("could not download nonexistent rust version `{}`",
                        toolchain_str)
            })
        }
        Err(e) => Err(e),
    }
}

fn dl_v2_manifest<'a>(download: DownloadCfg<'a>,
                      update_hash: Option<&Path>,
                      toolchain: &ToolchainDesc)
                      -> Result<Option<(ManifestV2, String)>> {
    let manifest_url = toolchain.manifest_v2_url(download.dist_root);
    let manifest_dl_res = download.download_and_check(&manifest_url, update_hash, ".toml");

    if let Ok(manifest_dl) = manifest_dl_res {
        // Downloaded ok!
        let (manifest_file, manifest_hash) = if let Some(m) = manifest_dl {
            m
        } else {
            return Ok(None);
        };
        let manifest_str = try!(utils::read_file("manifest", &manifest_file));
        let manifest = try!(ManifestV2::parse(&manifest_str));

        Ok(Some((manifest, manifest_hash)))
    } else {
        match *manifest_dl_res.as_ref().unwrap_err().kind() {
            // Checksum failed - issue warning to try again later
            ErrorKind::ChecksumFailed { .. } => {
                (download.notify_handler)(Notification::ManifestChecksumFailedHack)
            }
            _ => {}
        }
        Err(manifest_dl_res.unwrap_err())
    }

}

fn dl_v1_manifest<'a>(download: DownloadCfg<'a>, toolchain: &ToolchainDesc) -> Result<Vec<String>> {
    let root_url = toolchain.package_dir(download.dist_root);

    if !["nightly", "beta", "stable"].contains(&&*toolchain.channel) {
        // This is an explicit version. In v1 there was no manifest,
        // you just know the file to download, so synthesize one.
        let installer_name = format!("{}/rust-{}-{}.tar.gz",
                                     root_url,
                                     toolchain.channel,
                                     toolchain.target);
        return Ok(vec![installer_name]);
    }

    let manifest_url = toolchain.manifest_v1_url(download.dist_root);
    let manifest_dl = try!(download.download_and_check(&manifest_url, None, ""));
    let (manifest_file, _) = manifest_dl.unwrap();
    let manifest_str = try!(utils::read_file("manifest", &manifest_file));
    let urls = manifest_str.lines().map(|s| format!("{}/{}", root_url, s)).collect();

    Ok(urls)
}
