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
    let target = env::var("TARGET").expect("TARGET not set");

    create_build_iso_script(&manifest_dir, &profile, &target);
    create_run_script(&manifest_dir, &profile, &target);
}

/// Create a script to build the ISO after the binary is compiled
fn create_build_iso_script(manifest_dir: &Path, profile: &str, target: &str) {
    let script_path = manifest_dir.join("build-iso.sh");
    let script_content = format!(
        r#"#!/bin/bash
set -e

TARGET_TRIPLE="{target}"
PROFILE="{profile}"

# Copy the iso_root directory to target
rm -rf iso_root_temp
cp -r iso_root iso_root_temp

# Copy the compiled binary to iso_root
cp "target/${{TARGET_TRIPLE}}/${{PROFILE}}/atahos_core" iso_root_temp/boot/atahos_core

# Copy kernel file
cp "target/${{TARGET_TRIPLE}}/${{PROFILE}}/atahos_core" iso_root_temp/boot/atahos_core

# Create the bootable ISO.
xorriso -as mkisofs -R -r -J -b boot/limine/limine-bios-cd.bin \
        -no-emul-boot -boot-load-size 4 -boot-info-table -hfsplus \
        -apm-block-size 2048 --efi-boot boot/limine/limine-uefi-cd.bin \
        -efi-boot-part --efi-boot-image --protective-msdos-label \
        iso_root_temp -o target/${{TARGET_TRIPLE}}/${{PROFILE}}/atahos.iso

# Install Limine stage 1 and 2 for legacy BIOS boot.
./limine/limine bios-install target/${{TARGET_TRIPLE}}/${{PROFILE}}/atahos.iso

echo "ISO created at target/${{TARGET_TRIPLE}}/${{PROFILE}}/atahos.iso"
"#,
        target = target,
        profile = profile
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

/// Create a script to run the virtual machine after the ISO is built
fn create_run_script(manifest_dir: &Path, profile: &str, target: &str) {
    let script_path = manifest_dir.join("run.sh");
    let script_content = format!(
        r#"#!/bin/bash
set -e

TARGET_TRIPLE="{target}"
PROFILE="{profile}"

VM_NAME="atahos_vm"

# Launch virtual machine with the built ISO
qemu-system-x86_64 -cdrom "target/${{TARGET_TRIPLE}}/${{PROFILE}}/atahos.iso" -m 512M -boot d -name ${{VM_NAME}} -serial stdio
"#,
        profile = profile,
        target = target
    );

    fs::write(&script_path, script_content).expect("Failed to create run.sh script");

    // Make it executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();
    }

    println!("cargo:warning=Created run.sh script");
    println!("cargo:warning=After building, run: ./run.sh");
}
