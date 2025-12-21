// This is a build-time tool that produces an ISO image.
// The actual work is done in build.rs
// Run with: cargo build [--release]
// Output: target/debug/atahos.iso (or target/release/atahos.iso)

fn main() {
    println!("Atah OS ISO Builder");
    println!("===================");
    println!();
    println!("The ISO image has been built during the compilation process.");
    println!("Look for 'atahos.iso' in the target/debug or target/release directory.");
    println!();
}
