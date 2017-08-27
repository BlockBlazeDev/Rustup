//! Maintains a Rust installation by installing individual Rust
//! platform components from a distribution server.

use config::Config;
use manifest::{Component, Manifest, TargetedPackage};
use dist::{TargetTriple, DEFAULT_DIST_SERVER};
use component::{Components, Transaction, TarGzPackage, TarXzPackage, Package};
use temp;
use errors::*;
use notifications::*;
use rustup_utils::utils;
use download::{DownloadCfg, File};
use prefix::InstallPrefix;
use std::path::Path;

pub const DIST_MANIFEST: &'static str = "multirust-channel-manifest.toml";
pub const CONFIG_FILE: &'static str = "multirust-config.toml";

enum Format {
    Gz,
    Xz,
}

#[derive(Debug)]
pub struct Manifestation {
    installation: Components,
    target_triple: TargetTriple
}

#[derive(Debug)]
pub struct Changes {
    pub add_extensions: Vec<Component>,
    pub remove_extensions: Vec<Component>,
}

impl Changes {
    pub fn none() -> Self {
        Changes {
            add_extensions: Vec::new(),
            remove_extensions: Vec::new(),
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum UpdateStatus { Changed, Unchanged }

impl Manifestation {
    /// Open the install prefix for updates from a distribution
    /// channel.  The install prefix directory does not need to exist;
    /// it will be created as needed. If there's an existing install
    /// then the rust-install installation format will be verified. A
    /// bad installer version is the only reason this will fail.
    pub fn open(prefix: InstallPrefix, triple: TargetTriple) -> Result<Self> {
        // TODO: validate the triple with the existing install as well
        // as the metadata format of the existing install
        Ok(Manifestation {
            installation: try!(Components::open(prefix)),
            target_triple: triple,
        })
    }

    /// Install or update from a given channel manifest, while
    /// selecting extension components to add or remove.
    ///
    /// `update` takes a manifest describing a release of Rust (which
    /// may be either a freshly-downloaded one, or the same one used
    /// for the previous install), as well as lists off extension
    /// components to add and remove.

    /// From that it schedules a list of components to uninstall and
    /// to uninstall to bring the installation up to date.  It
    /// downloads the components' packages. Then in a Transaction
    /// uninstalls old packages and installs new packages, writes the
    /// distribution manifest to "rustlib/rustup-dist.toml" and a
    /// configuration containing the component name-target pairs to
    /// "rustlib/rustup-config.toml".
    pub fn update(&self,
                  new_manifest: &Manifest,
                  changes: Changes,
                  download_cfg: &DownloadCfg,
                  notify_handler: &Fn(Notification)) -> Result<UpdateStatus> {

        // Some vars we're going to need a few times
        let temp_cfg = download_cfg.temp_cfg;
        let prefix = self.installation.prefix();
        let ref rel_installed_manifest_path = prefix.rel_manifest_file(DIST_MANIFEST);
        let ref installed_manifest_path = prefix.path().join(rel_installed_manifest_path);
        let rust_package = try!(new_manifest.get_package("rust"));
        let rust_target_package = try!(rust_package.get_target(Some(&self.target_triple)));

        // Load the previous dist manifest
        let ref old_manifest = try!(self.load_manifest());

        // Load the configuration and list of installed components.
        let ref config = try!(self.read_config());

        // Create the lists of components needed for installation
        let component_lists = try!(build_update_component_lists(new_manifest, old_manifest, config,
                                                                changes, &rust_target_package,
                                                                notify_handler));
        let (components_to_uninstall,
             components_to_install,
             final_component_list) = component_lists;

        if components_to_uninstall.is_empty() && components_to_install.is_empty() {
            return Ok(UpdateStatus::Unchanged);
        }

        // Validate that the requested components are available
        let unavailable_components: Vec<Component> = components_to_install.iter().filter(|c| {
            use manifest::*;
            let pkg: Option<&Package> = new_manifest.get_package(&c.pkg).ok();
            let target_pkg: Option<&TargetedPackage> = pkg.and_then(|p| p.get_target(c.target.as_ref()).ok());
            target_pkg.map(|tp| tp.available()) != Some(true)
        }).cloned().collect();

        if !unavailable_components.is_empty() {
            return Err(ErrorKind::RequestedComponentsUnavailable(unavailable_components).into());
        }

        // Map components to urls and hashes
        let mut components_urls_and_hashes: Vec<(Component, Format, String, String)> = Vec::new();
        for component in components_to_install {
            let package = try!(new_manifest.get_package(&component.pkg));
            let target_package = try!(package.get_target(component.target.as_ref()));

            let bins = target_package.bins.as_ref().expect("components available");
            let c_u_h =
                if let (Some(url), Some(hash)) = (bins.xz_url.clone(),
                                                  bins.xz_hash.clone()) {
                    (component, Format::Xz, url, hash)
                } else {
                    (component, Format::Gz, bins.url.clone(), bins.hash.clone())
                };
            components_urls_and_hashes.push(c_u_h);
        }

        let altered = temp_cfg.dist_server != DEFAULT_DIST_SERVER;

        // Download component packages and validate hashes
        let mut things_to_install: Vec<(Component, Format, File)> = Vec::new();
        let mut things_downloaded: Vec<String> = Vec::new();
        for (component, format, url, hash) in components_urls_and_hashes {

            notify_handler(Notification::DownloadingComponent(&component.pkg,
                                                              &self.target_triple,
                                                              component.target.as_ref()));
            let url = if altered {
                url.replace(DEFAULT_DIST_SERVER, temp_cfg.dist_server.as_str())
            } else {
                url
            };

            let url_url = try!(utils::parse_url(&url));

            let dowloaded_file = try!(download_cfg.download(&url_url, &hash).chain_err(|| {
                ErrorKind::ComponentDownloadFailed(component.clone())
            }));
            things_downloaded.push(hash);

            things_to_install.push((component, format, dowloaded_file));
        }

        // Begin transaction
        let mut tx = Transaction::new(prefix.clone(), temp_cfg, notify_handler);

        // If the previous installation was from a v1 manifest we need
        // to uninstall it first.
        tx = try!(self.maybe_handle_v2_upgrade(config, tx));

        // Uninstall components
        for component in components_to_uninstall {

            notify_handler(Notification::RemovingComponent(&component.pkg,
                                                           &self.target_triple,
                                                           component.target.as_ref()));

            tx = try!(self.uninstall_component(&component, tx, notify_handler.clone()));
        }

        // Install components
        for (component, format, installer_file) in things_to_install {

            notify_handler(Notification::InstallingComponent(&component.pkg,
                                                             &self.target_triple,
                                                             component.target.as_ref()));

            let gz;
            let xz;
            let package: &Package = match format {
                Format::Gz => {
                    gz = try!(TarGzPackage::new_file(&installer_file, temp_cfg));
                    &gz
                }
                Format::Xz => {
                    xz = try!(TarXzPackage::new_file(&installer_file, temp_cfg));
                    &xz
                }
            };

            // For historical reasons, the rust-installer component
            // names are not the same as the dist manifest component
            // names. Some are just the component name some are the
            // component name plus the target triple.
            let ref name = component.name();
            let ref short_name = format!("{}", component.pkg);

            // If the package doesn't contain the component that the
            // manifest says it does the somebody must be playing a joke on us.
            if !package.contains(name, Some(short_name)) {
                return Err(ErrorKind::CorruptComponent(component.pkg.clone()).into());
            }

            tx = try!(package.install(&self.installation,
                                      name, Some(short_name),
                                      tx));
        }

        // Install new distribution manifest
        let ref new_manifest_str = new_manifest.clone().stringify();
        try!(tx.modify_file(rel_installed_manifest_path.to_owned()));
        try!(utils::write_file("manifest", installed_manifest_path, new_manifest_str));

        // Write configuration.
        //
        // NB: This configuration is mostly for keeping track of the name/target pairs
        // that identify installed components. The rust-installer metadata maintained by
        // `Components` *also* tracks what is installed, but it only tracks names, not
        // name/target. Needs to be fixed in rust-installer.
        let mut config = Config::new();
        config.components = final_component_list;
        let ref config_str = config.stringify();
        let ref rel_config_path = prefix.rel_manifest_file(CONFIG_FILE);
        let ref config_path = prefix.path().join(rel_config_path);
        try!(tx.modify_file(rel_config_path.to_owned()));
        try!(utils::write_file("dist config", config_path, config_str));

        // End transaction
        tx.commit();

        try!(download_cfg.clean(&things_downloaded));

        Ok(UpdateStatus::Changed)
    }

    pub fn uninstall(&self, temp_cfg: &temp::Cfg, notify_handler: &Fn(Notification)) -> Result<()> {
        let prefix = self.installation.prefix();

        let mut tx = Transaction::new(prefix.clone(), temp_cfg, notify_handler);

        // Read configuration and delete it
        let rel_config_path = prefix.rel_manifest_file(CONFIG_FILE);
        let ref config_str = try!(utils::read_file("dist config", &prefix.path().join(&rel_config_path)));
        let config = try!(Config::parse(config_str));
        try!(tx.remove_file("dist config", rel_config_path));

        for component in config.components {
            tx = try!(self.uninstall_component(&component, tx, notify_handler));
        }
        tx.commit();

        Ok(())
    }

    fn uninstall_component<'a>(&self, component: &Component, mut tx: Transaction<'a>,
                               notify_handler: &Fn(Notification)) -> Result<Transaction<'a>> {
        // For historical reasons, the rust-installer component
        // names are not the same as the dist manifest component
        // names. Some are just the component name some are the
        // component name plus the target triple.
        let ref name = component.name();
        let ref short_name = format!("{}", component.pkg);
        if let Some(c) = try!(self.installation.find(&name)) {
            tx = try!(c.uninstall(tx));
        } else if let Some(c) = try!(self.installation.find(&short_name)) {
            tx = try!(c.uninstall(tx));
        } else {
            notify_handler(Notification::MissingInstalledComponent(&name));
        }

        Ok(tx)
    }

    // Read the config file. Config files are presently only created
    // for v2 installations.
    pub fn read_config(&self) -> Result<Option<Config>> {
        let prefix = self.installation.prefix();
        let ref rel_config_path = prefix.rel_manifest_file(CONFIG_FILE);
        let ref config_path = prefix.path().join(rel_config_path);
        if utils::path_exists(config_path) {
            let ref config_str = try!(utils::read_file("dist config", config_path));
            Ok(Some(try!(Config::parse(config_str))))
        } else {
            Ok(None)
        }
    }

    pub fn load_manifest(&self) -> Result<Option<Manifest>> {
        let prefix = self.installation.prefix();
        let ref old_manifest_path = prefix.manifest_file(DIST_MANIFEST);
        if utils::path_exists(old_manifest_path) {
            let ref manifest_str = try!(utils::read_file("installed manifest", old_manifest_path));
            Ok(Some(try!(Manifest::parse(manifest_str))))
        } else {
            Ok(None)
        }
    }

    /// Installation using the legacy v1 manifest format
    pub fn update_v1(&self,
                     new_manifest: &[String],
                     update_hash: Option<&Path>,
                     temp_cfg: &temp::Cfg,
                     notify_handler: &Fn(Notification)) -> Result<Option<String>> {
        // If there's already a v2 installation then something has gone wrong
        if try!(self.read_config()).is_some() {
            return Err("the server unexpectedly provided an obsolete version of the distribution manifest".into());
        }

        let url = new_manifest.iter().find(|u| u.contains(&format!("{}{}", self.target_triple, ".tar.gz")));
        if url.is_none() {
            return Err(format!("binary package was not provided for '{}'",
                               self.target_triple.to_string()).into());
        }
        // Only replace once. The cost is inexpensive.
        let url = url.unwrap().replace(DEFAULT_DIST_SERVER, temp_cfg.dist_server.as_str());

        notify_handler(Notification::DownloadingComponent("rust",
                                                          &self.target_triple,
                                                          Some(&self.target_triple)));

        use std::path::PathBuf;
        let dld_dir = PathBuf::from("bogus");
        let dlcfg = DownloadCfg {
            dist_root: "bogus",
            download_dir: &dld_dir,
            temp_cfg: temp_cfg,
            notify_handler: notify_handler
        };

        let dl = try!(dlcfg.download_and_check(&url, update_hash, ".tar.gz"));
        if dl.is_none() {
            return Ok(None);
        };
        let (installer_file, installer_hash) = dl.unwrap();

        let prefix = self.installation.prefix();

        notify_handler(Notification::InstallingComponent("rust",
                                                         &self.target_triple,
                                                         Some(&self.target_triple)));

        // Begin transaction
        let mut tx = Transaction::new(prefix.clone(), temp_cfg, notify_handler);

        // Uninstall components
        for component in try!(self.installation.list()) {
            tx = try!(component.uninstall(tx));
        }

        // Install all the components in the installer
        let package = try!(TarGzPackage::new_file(&installer_file, temp_cfg));

        for component in package.components() {
            tx = try!(package.install(&self.installation,
                                      &component, None,
                                      tx));
        }

        // End transaction
        tx.commit();

        Ok(Some(installer_hash))
    }

    // If the previous installation was from a v1 manifest, then it
    // doesn't have a configuration or manifest-derived list of
    // component/target pairs. Uninstall it using the intaller's
    // component list before upgrading.
    fn maybe_handle_v2_upgrade<'a>(&self,
                                   config: &Option<Config>,
                                   mut tx: Transaction<'a>) -> Result<Transaction<'a>> {
        let installed_components = try!(self.installation.list());
        let looks_like_v1 = config.is_none() && !installed_components.is_empty();

        if !looks_like_v1 { return Ok(tx) }

        for component in installed_components {
            tx = try!(component.uninstall(tx));
        }

        Ok(tx)
    }
}

/// Returns components to uninstall, install, and the list of all
/// components that will be up to date after the update.
fn build_update_component_lists(
    new_manifest: &Manifest,
    old_manifest: &Option<Manifest>,
    config: &Option<Config>,
    changes: Changes,
    rust_target_package: &TargetedPackage,
    notify_handler: &Fn(Notification),
    ) -> Result<(Vec<Component>, Vec<Component>, Vec<Component>)> {

    // Check some invariantns
    for component_to_add in &changes.add_extensions {
        assert!(rust_target_package.extensions.contains(component_to_add),
                "package must contain extension to add");
        assert!(!changes.remove_extensions.contains(component_to_add),
                "can't both add and remove extensions");
    }
    for component_to_remove in &changes.remove_extensions {
        assert!(rust_target_package.extensions.contains(component_to_remove),
                "package must contain extension to remove");
        let config = config.as_ref().expect("removing extension on fresh install?");
        assert!(config.components.contains(component_to_remove),
                "removing package that isn't installed");
    }

    // The list of components already installed, empty if a new install
    let starting_list = config.as_ref().map(|c| c.components.clone()).unwrap_or(Vec::new());

    // The list of components we'll have installed at the end
    let mut final_component_list = Vec::new();

    // Find the final list of components we want to be left with when
    // we're done: required components, added extensions, and existing
    // installed extensions.

    // Add components required by the package, according to the
    // manifest
    for required_component in &rust_target_package.components {
        final_component_list.push(required_component.clone());
    }

    // Add requested extension components
    for extension in &changes.add_extensions {
        final_component_list.push(extension.clone());
    }

    // Add extensions that are already installed
    for existing_component in &starting_list {
        let is_extension = rust_target_package.extensions.contains(existing_component);
        let is_removed = changes.remove_extensions.contains(existing_component);

        if is_extension && !is_removed {
            // If there is a rename in the (new) manifest, then we uninstall the component with the
            // old name and install a component with the new name
            if new_manifest.renames.contains_key(&existing_component.pkg) {
                let mut renamed_component = existing_component.clone();
                renamed_component.pkg = new_manifest.renames[&existing_component.pkg].to_owned();
                let is_already_included = final_component_list.contains(&renamed_component);
                if !is_already_included {
                    final_component_list.push(renamed_component);
                }
            } else {
                let is_extension = rust_target_package.extensions.contains(existing_component);
                let is_already_included = final_component_list.contains(existing_component);
                if is_extension && !is_already_included {
                    final_component_list.push(existing_component.clone());
                }
            }
        }
    }

    let mut components_to_uninstall = Vec::new();
    let mut components_to_install = Vec::new();

    // If this is a full upgrade then the list of components to
    // uninstall is all that are currently installed, and those
    // to install the final list. It's a complete reinstall.
    //
    // If it's a modification then the components to uninstall are
    // those that are currently installed but not in the final list.
    // To install are those on the final list but not already
    // installed.
    let just_modifying_existing_install = old_manifest.as_ref() == Some(new_manifest);
    if !just_modifying_existing_install {
        components_to_uninstall = starting_list.clone();
        components_to_install = final_component_list.clone();
    } else {
        for existing_component in &starting_list {
            if !final_component_list.contains(existing_component) {
                components_to_uninstall.push(existing_component.clone())
            }
        }
        for component in &final_component_list {
            if !starting_list.contains(component) {
                components_to_install.push(component.clone());
            } else {
                if changes.add_extensions.contains(&component) {
                    notify_handler(Notification::ComponentAlreadyInstalled(&component));
                }
            }
        }
    }

    Ok((components_to_uninstall, components_to_install, final_component_list))
}
