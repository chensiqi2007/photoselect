fn main() {
    println!("cargo:rerun-if-changed=src/proprietary/npu_kernel.rs");
    println!("cargo:rustc-check-cfg=cfg(has_proprietary_ai)");
    if std::path::Path::new("src/proprietary/npu_kernel.rs").exists() {
        println!("cargo:rustc-cfg=has_proprietary_ai");
    }
    tauri_build::build()
}
