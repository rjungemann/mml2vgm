fn main() {
    println!("cargo:rerun-if-changed=src/chips/vendor/libvgm/emu/cores/ymf271_ffi.c");
    println!("cargo:rerun-if-changed=src/chips/vendor/libvgm/emu/cores/ymf271.c");
    println!("cargo:rerun-if-changed=src/chips/vendor/libvgm/emu/logging.c");

    // The libvgm C sources depend on the standard C library (string.h, stdlib.h,
    // math.h, ...). wasm32-unknown-unknown has no libc/sysroot, so skip the C
    // build there — the YMF271 emulator is only used by the native audio player,
    // not by the WASM MML compiler bindings. See src/chips/ymf271.rs for the
    // matching extern stubs.
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if target_arch == "wasm32" {
        return;
    }

    cc::Build::new()
        .file("src/chips/vendor/libvgm/emu/cores/ymf271_ffi.c")
        .file("src/chips/vendor/libvgm/emu/logging.c")
        .define("HAVE_STDINT_H", None)
        .warnings(false)
        .compile("ymf271");
}
