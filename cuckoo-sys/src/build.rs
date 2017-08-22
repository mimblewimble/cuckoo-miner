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

//! cmake wrapper for build

extern crate cmake;
extern crate fs_extra;

use cmake::Config;
use std::env;
use std::path::{PathBuf};
use fs_extra::dir::*;

fn main() {
	let path_str = env::var("OUT_DIR").unwrap();
	let mut out_path = PathBuf::from(&path_str);
	out_path.pop();
	out_path.pop();
	out_path.pop();
	let mut plugin_path = PathBuf::from(&path_str);
	plugin_path.push("build");
	plugin_path.push("miner-plugins");
	println!("cargo:rerun-if-changed=build.rs");
	println!("cargo:rerun-if-changed=../plugins");
	println!("cargo:rerun-if-changed=../plugins/cuckoo-miner-plugins");
	println!("cargo:rerun-if-changed=../plugins/cuckoo-miner-plugins/cmake");
	println!("cargo:rerun-if-changed=../plugins/cuckoo-miner-plugins/cuckoo/src");
	let dst = Config::new("../plugins/cuckoo-miner-plugins")
	                      //.define("FOO","BAR") //whatever flags go here
	                      //.cflag("-foo") //and here
	                      .build_target("")
	                      .build();
	
	
	println!("Plugin path: {:?}", plugin_path);
	println!("OUT PATH: {:?}", out_path);
	let options = CopyOptions::new();
	if let Err(e) = copy(plugin_path, out_path, &options) {
		println!("{:?}", e);
	}

	println!("cargo:rustc-link-search=native={}", dst.display());
	println!("cargo:rustc-link-lib=dylib=cuckoo-miner-plugins");

}