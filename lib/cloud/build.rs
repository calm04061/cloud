fn main() {
    find_package()
}
#[cfg(windows)]
fn find_package() {
    vcpkg::Config::new()
        .emit_includes(true)
        .find_package("zstd")
        .unwrap();
    vcpkg::Config::new()
        .emit_includes(true)
        .find_package("openssl")
        .unwrap();
}
#[cfg(not(windows))]
fn find_package() {}
