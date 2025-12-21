use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Build context containing paths and configuration
#[allow(dead_code)]
struct BuildContext {
    out_dir: PathBuf,
    manifest_dir: PathBuf,
    profile: String,
    temp_image: PathBuf,
    efi_img: PathBuf,
    iso_path: PathBuf,
}

impl BuildContext {
    fn new() -> Self {
        let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
        let manifest_dir =
            PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
        let profile = env::var("PROFILE").expect("PROFILE not set");

        let temp_image = out_dir.join("iso_image");
        let efi_img = out_dir.join("efiboot.img");
        let iso_path = out_dir.join("atahos.iso");

        BuildContext {
            out_dir,
            manifest_dir,
            profile,
            temp_image,
            efi_img,
            iso_path,
        }
    }
}

fn main() {
    // Notify Cargo to rebuild if these files change
    println!("cargo:rerun-if-changed=boot/src");
    println!("cargo:rerun-if-changed=boot/Cargo.toml");
    println!("cargo:rerun-if-changed=boot/rust-toolchain.toml");
    println!("cargo:rerun-if-changed=image");

    // Tell Cargo to build the boot crate first if it hasn't been built
    println!("cargo:rerun-if-changed-env=CARGO");

    let ctx = BuildContext::new();

    build_bootloader(&ctx);
    prepare_image_directory(&ctx);
    copy_bootloader_to_image(&ctx);
    create_efi_partition(&ctx);
    create_iso_image(&ctx);
    finalize_iso(&ctx);
}

/// Build the UEFI bootloader
fn build_bootloader(ctx: &BuildContext) {
    println!("cargo:warning=Building UEFI bootloader...");

    let boot_dir = ctx.manifest_dir.join("boot");
    let cargo_exe = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());

    let output = Command::new(&cargo_exe)
        .current_dir(&boot_dir)
        .arg("build")
        .arg("--target")
        .arg("x86_64-unknown-uefi")
        .env("CARGO_BUILD_JOBS", "1")
        .args(if ctx.profile == "release" {
            vec!["--release"]
        } else {
            vec![]
        })
        .output()
        .expect("Failed to execute bootloader build");

    if !output.status.success() {
        eprintln!(
            "Bootloader build stdout:\n{}",
            String::from_utf8_lossy(&output.stdout)
        );
        eprintln!(
            "Bootloader build stderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        panic!("Bootloader build failed with status: {}", output.status);
    }

    let bootloader_path = ctx
        .manifest_dir
        .join("boot/target/x86_64-unknown-uefi")
        .join(&ctx.profile)
        .join("atahos_boot.efi");

    if !bootloader_path.exists() {
        eprintln!(
            "Error: Bootloader artifact not found at {}",
            bootloader_path.display()
        );
        panic!("Bootloader build completed but artifact not found");
    }

    eprintln!(
        "Bootloader built successfully: {}",
        bootloader_path.display()
    );
}

/// Prepare the temporary image directory
fn prepare_image_directory(ctx: &BuildContext) {
    println!("cargo:warning=Copying image directory structure...");

    if ctx.temp_image.exists() {
        fs::remove_dir_all(&ctx.temp_image).expect("Failed to remove old temp image");
    }

    let source_image = ctx.manifest_dir.join("image");
    copy_dir_all(&source_image, &ctx.temp_image).expect("Failed to copy image directory");
}

/// Copy the built bootloader to the UEFI boot location in the image
fn copy_bootloader_to_image(ctx: &BuildContext) {
    let bootloader_src = ctx
        .manifest_dir
        .join("boot/target/x86_64-unknown-uefi")
        .join(&ctx.profile)
        .join("atahos_boot.efi");

    let bootloader_dest = ctx.temp_image.join("EFI/EFI/BOOT/BOOTx64.EFI");

    fs::create_dir_all(bootloader_dest.parent().unwrap()).expect("Failed to create BOOT directory");

    fs::copy(&bootloader_src, &bootloader_dest).expect("Failed to copy bootloader to image");

    println!(
        "cargo:warning=Bootloader copied to {}",
        bootloader_dest.display()
    );
}

/// Create the EFI System Partition (ESP) as a FAT32 image
fn create_efi_partition(ctx: &BuildContext) {
    println!("cargo:warning=Creating EFI System Partition image...");

    // Remove old EFI image if it exists
    let _ = fs::remove_file(&ctx.efi_img);

    create_fat32_image(&ctx.efi_img);
    populate_efi_partition(ctx);
    cleanup_iso_image_directories(ctx);
}

/// Create a FAT32 image file
fn create_fat32_image(efi_img: &Path) {
    let output = Command::new("mkfs.fat")
        .args(&["-F", "32", "-C"])
        .arg(efi_img)
        .arg("65536") // 64MB
        .output()
        .expect("Failed to create FAT32 image");

    if !output.status.success() {
        eprintln!(
            "mkfs.fat stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        panic!("Failed to create EFI partition image");
    }
}

/// Populate the EFI partition with files from the EFI directory
fn populate_efi_partition(ctx: &BuildContext) {
    let efi_source = ctx.temp_image.join("EFI");

    if !efi_source.exists() {
        eprintln!(
            "Warning: EFI source directory not found at {}",
            efi_source.display()
        );
        return;
    }

    println!("cargo:warning=Populating EFI partition image...");

    if let Ok(entries) = fs::read_dir(&efi_source) {
        for entry in entries {
            if let Ok(entry) = entry {
                let src_path = entry.path();
                let file_name = entry.file_name();

                if src_path.is_dir() {
                    copy_to_efi_image(&src_path, &file_name, &ctx.efi_img);
                }
            }
        }
    }
}

/// Copy a directory to the EFI partition image using mcopy
fn copy_to_efi_image(src_path: &Path, file_name: &std::ffi::OsStr, efi_img: &Path) {
    println!(
        "cargo:warning=Copying {} to EFI image",
        file_name.to_string_lossy()
    );

    let output = Command::new("mcopy")
        .args(&["-i", efi_img.to_str().unwrap(), "-r"])
        .arg(src_path)
        .arg(format!("::/{}/", file_name.to_string_lossy()))
        .output()
        .expect("Failed to run mcopy");

    if !output.status.success() {
        eprintln!(
            "Warning: mcopy for {} failed: {}",
            file_name.to_string_lossy(),
            String::from_utf8_lossy(&output.stderr)
        );
    } else {
        println!(
            "cargo:warning=Successfully copied {}",
            file_name.to_string_lossy()
        );
    }
}

/// Remove directories that should not be in the ISO filesystem
fn cleanup_iso_image_directories(ctx: &BuildContext) {
    fs::remove_dir_all(ctx.temp_image.join("EFI")).ok();
    fs::remove_dir_all(ctx.temp_image.join("HOME")).ok();
    fs::remove_dir_all(ctx.temp_image.join("MODS")).ok();
}

/// Create the EFI-bootable ISO image
fn create_iso_image(ctx: &BuildContext) {
    println!(
        "cargo:warning=Creating EFI-bootable ISO image at {}...",
        ctx.iso_path.display()
    );

    let output = build_xorriso_command(ctx)
        .output()
        .expect("Failed to execute xorriso");

    if !output.status.success() {
        eprintln!(
            "xorriso stdout: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        eprintln!(
            "xorriso stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        panic!(
            "xorriso failed to create ISO with status: {}",
            output.status
        );
    }

    println!(
        "cargo:warning=ISO image created successfully: {}",
        ctx.iso_path.display()
    );
}

/// Build the xorriso command with proper EFI boot configuration
fn build_xorriso_command(ctx: &BuildContext) -> Command {
    let mut cmd = Command::new("xorriso");
    cmd.args(&[
        "-as",
        "mkisofs",
        "-R", // Rock Ridge extensions
        "-f", // Follow symlinks
        "-J", // Joliet extensions
        "-eltorito-alt-boot",
        "-e",
        ctx.efi_img.file_name().unwrap().to_str().unwrap(), // EFI partition as boot image
        "-no-emul-boot",
        "-isohybrid-gpt-basdat", // GPT partition for UEFI
        "-c",
        "boot.cat",
        "-o",
        ctx.iso_path.to_str().unwrap(),
    ])
    .arg(&ctx.efi_img) // Add EFI partition image as input
    .arg(&ctx.temp_image); // Add filesystem contents as input

    cmd
}

/// Copy the final ISO to the target directory for easy access
fn finalize_iso(ctx: &BuildContext) {
    let target_dir = ctx.manifest_dir.join("target").join(&ctx.profile);
    fs::create_dir_all(&target_dir).expect("Failed to create target directory");

    let final_iso = target_dir.join("atahos.iso");
    fs::copy(&ctx.iso_path, &final_iso).expect("Failed to copy ISO to target");

    println!("cargo:warning=ISO also copied to: {}", final_iso.display());
}

/// Recursively copy a directory and all its contents
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
