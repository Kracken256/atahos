use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    // Notify Cargo to rebuild if these files change
    println!("cargo:rerun-if-changed=iso_root");
    println!("cargo:rerun-if-changed=src");

    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    let profile = env::var("PROFILE").expect("PROFILE not set");

    // Create the build-iso script
    create_build_iso_script(&manifest_dir, &profile);
}

/// Create a script to build the ISO after the binary is compiled
fn create_build_iso_script(manifest_dir: &Path, profile: &str) {
    let script_path = manifest_dir.join("build-iso.sh");
    let script_content = format!(
        r#"#!/bin/bash
set -e

# Copy the compiled binary to iso_root
cp target/i686-unknown-linux-gnu/{}/atahos_core iso_root/boot/atahos_core

# Build the ISO
grub-mkrescue -o target/{}/atahos.iso iso_root

echo "ISO created at target/{}/atahos.iso"
"#,
        profile, profile, profile
    );

    fs::write(&script_path, script_content).expect("Failed to create build-iso script");

    // Make it executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();
    }

    println!("cargo:warning=Created build-iso.sh script");
    println!("cargo:warning=After building, run: ./build-iso.sh");
}
