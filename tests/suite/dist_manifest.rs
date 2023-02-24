use rustup::dist::dist::TargetTriple;
use rustup::dist::manifest::Manifest;
use rustup::RustupError;

// Example manifest from https://public.etherpad-mozilla.org/p/Rust-infra-work-week
static EXAMPLE: &str = include_str!("channel-rust-nightly-example.toml");
// From brson's live build-rust-manifest.py script
static EXAMPLE2: &str = include_str!("channel-rust-nightly-example2.toml");

#[test]
fn parse_smoke_test() {
    let x86_64_unknown_linux_gnu = TargetTriple::new("x86_64-unknown-linux-gnu");
    let x86_64_unknown_linux_musl = TargetTriple::new("x86_64-unknown-linux-musl");

    let pkg = Manifest::parse(EXAMPLE).unwrap();

    pkg.get_package("rust").unwrap();
    pkg.get_package("rustc").unwrap();
    pkg.get_package("cargo").unwrap();
    pkg.get_package("rust-std").unwrap();
    pkg.get_package("rust-docs").unwrap();

    let rust_pkg = pkg.get_package("rust").unwrap();
    assert!(rust_pkg.version.contains("1.3.0"));

    let rust_target_pkg = rust_pkg
        .get_target(Some(&x86_64_unknown_linux_gnu))
        .unwrap();
    assert!(rust_target_pkg.available());
    assert_eq!(rust_target_pkg.bins[0].1.url, "example.com");
    assert_eq!(rust_target_pkg.bins[0].1.hash, "...");

    let component = &rust_target_pkg.components[0];
    assert_eq!(component.short_name_in_manifest(), "rustc");
    assert_eq!(component.target.as_ref(), Some(&x86_64_unknown_linux_gnu));

    let component = &rust_target_pkg.components[4];
    assert_eq!(component.short_name_in_manifest(), "rust-std");
    assert_eq!(component.target.as_ref(), Some(&x86_64_unknown_linux_musl));

    let docs_pkg = pkg.get_package("rust-docs").unwrap();
    let docs_target_pkg = docs_pkg
        .get_target(Some(&x86_64_unknown_linux_gnu))
        .unwrap();
    assert_eq!(docs_target_pkg.bins[0].1.url, "example.com");
}

#[test]
fn renames() {
    let manifest = Manifest::parse(EXAMPLE2).unwrap();
    assert_eq!(1, manifest.renames.len());
    assert_eq!(manifest.renames["cargo-old"], "cargo");
    assert_eq!(1, manifest.reverse_renames.len());
    assert_eq!(manifest.reverse_renames["cargo"], "cargo-old");
}

#[test]
fn parse_round_trip() {
    let original = Manifest::parse(EXAMPLE).unwrap();
    let serialized = original.clone().stringify();
    let new = Manifest::parse(&serialized).unwrap();
    assert_eq!(original, new);

    let original = Manifest::parse(EXAMPLE2).unwrap();
    let serialized = original.clone().stringify();
    let new = Manifest::parse(&serialized).unwrap();
    assert_eq!(original, new);
}

#[test]
fn validate_components_have_corresponding_packages() {
    let manifest = r#"
manifest-version = "2"
date = "2015-10-10"
[pkg.rust]
  version = "rustc 1.3.0 (9a92aaf19 2015-09-15)"
  [pkg.rust.target.x86_64-unknown-linux-gnu]
    available = true
    url = "example.com"
    hash = "..."
    [[pkg.rust.target.x86_64-unknown-linux-gnu.components]]
      pkg = "rustc"
      target = "x86_64-unknown-linux-gnu"
    [[pkg.rust.target.x86_64-unknown-linux-gnu.extensions]]
      pkg = "rust-std"
      target = "x86_64-unknown-linux-musl"
[pkg.rustc]
  version = "rustc 1.3.0 (9a92aaf19 2015-09-15)"
  [pkg.rustc.target.x86_64-unknown-linux-gnu]
    available = true
    url = "example.com"
    hash = "..."
"#;

    let err = Manifest::parse(manifest).unwrap_err();

    match err.downcast::<RustupError>().unwrap() {
        RustupError::MissingPackageForComponent(_) => {}
        _ => panic!(),
    }
}

// #248
#[test]
fn manifest_can_contain_unknown_targets() {
    let manifest = EXAMPLE.replace("x86_64-unknown-linux-gnu", "mycpu-myvendor-myos");

    assert!(Manifest::parse(&manifest).is_ok());
}
