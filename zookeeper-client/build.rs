use std::process::Command;

fn main() {
    // generate C binding
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=include/zookeeper.h");
    println!("cargo:rerun-if-changed=src/c.rs");
    Command::new("cbindgen")
        .args(&[
            "--output",
            "include/zookeeper.h",
            "--lang",
            "c",
            ".",
        ])
        .status()
        .unwrap();
}
