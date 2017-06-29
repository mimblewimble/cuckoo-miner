extern crate gcc;

use std::fs;

fn link(name: &str, bundled: bool) {
    use std::env::var;
    let target = var("TARGET").unwrap();
    let target: Vec<_> = target.split('-').collect();
    if target.get(2) == Some(&"windows") {
        println!("cargo:rustc-link-lib=dylib={}", name);
        if bundled && target.get(3) == Some(&"gnu") {
            let dir = var("CARGO_MANIFEST_DIR").unwrap();
            println!("cargo:rustc-link-search=native={}/{}", dir, target[0]);
        }
    }
}

fn fail_on_empty_directory(name: &str) {
    if fs::read_dir(name).unwrap().count() == 0 {
        println!("The `{}` directory is empty, did you forget to pull the submodules?", name);
        println!("Try `git submodule update --init --recursive`");
        panic!();
    }
}

fn build_cuckoo() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=cuckoo-sys/");
    println!("cargo:rerun-if-changed=cuckoo-sys/include");
    println!("cargo:rerun-if-changed=cuckoo-sys/src");

    let mut config = gcc::Config::new();
    config.include("cuckoo-sys/src");
    config.include("cuckoo-sys/include");
    config.include(".");

    //Leave this here for now, config defines go here
    config.define("NDEBUG", Some("1"));
    
    let mut lib_sources = include_str!("cuckoo_sys_lib_sources.txt")
        .split(" ")
        .collect::<Vec<&'static str>>();


    
    if cfg!(target_env = "msvc") {
        config.flag("-EHsc");
    } else {
        config.flag("-std=c++11");
    }

    for file in lib_sources {
        let file = "cuckoo-sys/src/".to_string() + file;
        config.file(&file);
    }

    //config.flag("-lssl");
    //config.flag("-lcrypto");

    config.cpp(true);
    config.compile("libcuckoo-sys.a");
}

fn main() {
    fail_on_empty_directory("cuckoo-sys");
    build_cuckoo();
}
