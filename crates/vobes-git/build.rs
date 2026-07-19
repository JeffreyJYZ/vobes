#[cfg(windows)]
fn main() {
    println!("cargo:rustc-link-lib=advapi32");
    println!("cargo:rustc-link-lib=crypt32");
    println!("cargo:rustc-link-lib=userenv");
    println!("cargo:rustc-link-lib=bcrypt");
}

#[cfg(not(windows))]
fn main() {}
