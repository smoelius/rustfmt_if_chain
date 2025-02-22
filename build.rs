use rustc_version::{Channel, version_meta};

fn main() {
    if version_meta().unwrap().channel == Channel::Nightly {
        println!("cargo:rustc-cfg=nightly");
    }
}
