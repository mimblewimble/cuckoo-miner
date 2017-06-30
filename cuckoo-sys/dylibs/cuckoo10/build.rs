extern crate gcc;

use std::fs;

fn fail_on_empty_directory(name: &str) {
    if fs::read_dir(name).unwrap().count() == 0 {
        println!("The `{}` directory is empty, did you forget to pull the submodules?", name);
        println!("Try `git submodule update --init --recursive`");
        panic!();
    }
}

fn build_cuckoo() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../cuckoo/src");

    let mut config = gcc::Config::new();
    config.include("../../cuckoo/src");
    config.include("../../cuckoo/src/cuckoo_miner");
    config.include(".");

    //Leave this here for now, config defines go here
    config.define("NDEBUG", Some("1"));
    
    let mut lib_sources = include_str!("sources.txt")
        .split(" ")
        .collect::<Vec<&'static str>>();

    if cfg!(target_env = "msvc") {
        config.flag("-EHsc");
    } else {
        //config.flag("-Wall");
        config.flag("-D_POSIX_C_SOURCE=200112L");
        config.flag("-O3");
        config.flag("-std=c++11");
        config.flag("-march=native");
        config.flag("-m64");
    }

    for file in lib_sources {
        let file = "../../cuckoo/src/".to_string() + file;
        config.file(&file);
    }

    config.flag("-DEDGEBITS=11");
    config.flag("-DATOMIC");

    config.cpp(true);
    config.shared_flag(true);
    config.compile("libcuckoo-10.a");
}

fn main() {
    fail_on_empty_directory("../../cuckoo");
    build_cuckoo();
}
