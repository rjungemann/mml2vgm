fn main() {
    println!("cargo:rerun-if-changed=src/chips/vendor/libvgm/emu/cores/ymf271_ffi.c");
    println!("cargo:rerun-if-changed=src/chips/vendor/libvgm/emu/cores/ymf271.c");
    println!("cargo:rerun-if-changed=src/chips/vendor/libvgm/emu/logging.c");

    cc::Build::new()
        .file("src/chips/vendor/libvgm/emu/cores/ymf271_ffi.c")
        .file("src/chips/vendor/libvgm/emu/logging.c")
        .define("HAVE_STDINT_H", None)
        .warnings(false)
        .compile("ymf271");
}
