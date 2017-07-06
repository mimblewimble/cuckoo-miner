extern crate gcc;

use std::io::prelude::*;
use std::fs;
use std::fs::File;

use std::process::Command;
use std::path::Path;

use std::env;


pub struct CuckooBuildEnv {

}

impl CuckooBuildEnv {

    pub fn new()->CuckooBuildEnv{
        CuckooBuildEnv {

        }
    }

    pub fn fail_on_empty_directory(&mut self, name: &str) {
        if fs::read_dir(name).unwrap().count() == 0 {
            println!("The `{}` directory is empty, did you forget to pull the submodules?", name);
            println!("Try `git submodule update --init --recursive`");
            panic!();
        }
    }

    pub fn build_cuckoo(&mut self, sources:String, variation: &str, size:u16) {

        self.fail_on_empty_directory("../../cuckoo");

        println!("Current Directory: {}", env::current_dir().unwrap().display());

        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:rerun-if-changed=../../cuckoo/src");

        let mut config = gcc::Config::new();
        config.include("../../cuckoo/src");
        config.include("../../cuckoo/src/cuckoo_miner");
        config.include(".");

        //Leave this here for now, config defines go here
        config.define("NDEBUG", Some("1"));

        let mut file = File::open(sources).expect("Unable to open the file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Unable to read the file");
        //println!("{}", contents);
        
        let lib_sources = contents
            .split(" ")
            .collect::<Vec<&str>>();

        if cfg!(target_env = "msvc") {
            config.flag("-EHsc");
        } else {
            //config.flag("-Wall");
            config.flag("-D_POSIX_C_SOURCE=200112L");
            config.flag("-O3");
            config.flag("-std=c++11");
            config.flag("-march=native");
            config.flag("-m64");

            //config.flag("-mavx2");
            //config.flag("-DNSIPHASH=8");
        }

        for filename in lib_sources {
            let filepath = "../../cuckoo/src/".to_string() + &filename;
            config.file(&filepath);
        }

        config.flag(format!("-DEDGEBITS={}",size-1).as_str());
        config.flag("-DATOMIC");

        config.cpp(true);
        config.shared_flag(true);
        config.compile(format!("libcuckoo_{}_{}.a",variation,size).as_str());
    }

}