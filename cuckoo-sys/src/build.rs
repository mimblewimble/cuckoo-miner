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
	plugin_path.push("plugins");
	//Collect the files and directories we care about
	let dir_content = get_dir_content("plugins").unwrap();
	for d in dir_content.directories {
		let file_content = get_dir_content(d).unwrap();
		for f in file_content.files {
			println!("cargo:rerun-if-changed={}",f);
		}
	}
	for f in dir_content.files{
		println!("cargo:rerun-if-changed={}",f);
	}
	//panic!("marp");
	let dst = Config::new("plugins")
	                      //.define("FOO","BAR") //whatever flags go here
	                      //.cflag("-foo") //and here
	                      .build_target("")
	                      .build();
	
	
	println!("Plugin path: {:?}", plugin_path);
	println!("OUT PATH: {:?}", out_path);
	let mut options = CopyOptions::new();
	options.overwrite=true;
	if let Err(e) = copy(plugin_path, out_path, &options) {
		println!("{:?}", e);
	}

	println!("cargo:rustc-link-search=native={}", dst.display());

}