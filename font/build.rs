// font/build.rs
fn main() {
    println!("cargo:rerun-if-changed=../assets/ter-u16n.psf");
}
