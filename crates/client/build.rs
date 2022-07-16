fn main() {
    println!("cargo:rustc-link-search=native=/usr/lib64");
    println!("cargo:rustc-link-lib=dylib=X11");
    println!("cargo:rerun-if-changed=build.rs");
}
