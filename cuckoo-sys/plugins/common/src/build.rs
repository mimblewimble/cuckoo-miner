// Copyright 2017 The Grin Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Common build functions used by each plugin build.

extern crate gcc;

use std::io::prelude::*;
use std::fs;
use std::fs::File;

use std::path::Path;

use std::env;

/// Just a simple top level struct to hold an instance of the 
/// a plugin build environment

pub struct CuckooBuildEnv {

}

impl CuckooBuildEnv {

    /// Returns a new CuckooBuildEnv
    ///
    ///

    pub fn new()->CuckooBuildEnv{
        CuckooBuildEnv {

        }
    }

    /// Tests whether the source directory exists
    /// 
    ///

    pub fn fail_on_empty_directory(&mut self, name: &str) {
        if fs::read_dir(name).unwrap().count() == 0 {
            println!("The `{}` directory is empty, did you forget to pull the submodules?", name);
            println!("Try `git submodule update --init --recursive`");
            panic!();
        }
    }

    /// Builds a cuckoo plugin with a particular variation and size, by setting
    /// The size preprocessor definition in the source
    ///

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
            //Just return for now.. windows is unsupported
            return;
        }

        if cfg!(target_env = "msvc") {
            config.flag("-EHsc");
        } else {
            //config.flag("-Wall");
            config.flag("-D_POSIX_C_SOURCE=200112L");
            config.flag("-O3");
            config.flag("-std=c++11");
            config.flag("-march=native");
            config.flag("-m64");

            if variation == "mean" {
                config.flag("-DNSIPHASH=1");
                config.flag("-mavx2");
                //config.flag("-DBIG0SIZE=5");
            } else {
                config.flag("-DATOMIC");
            }
        }

        for filename in lib_sources {
            let filepath = "../../cuckoo/src/".to_string() + &filename;
            config.file(&filepath);
        }

        config.flag(format!("-DEDGEBITS={}",size-1).as_str());


        //Have to change the output dir, to ensure generated
        //object files from each build don't overwrite each other
        let out_dir = env::var("OUT_DIR").unwrap();
        println!("{}", out_dir);
        let modded_out_dir = format!("{}/cuckoo-{}" ,out_dir, size);
        config.out_dir(Path::new(&modded_out_dir));

        config.cpp(true);
        //config.shared_flag(true);
        config.compile(format!("libcuckoo_{}_{}.a",variation,size).as_str());
    }

}