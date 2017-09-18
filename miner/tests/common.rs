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
//

//! Common values and functions that can be used in all mining tests

extern crate error;
extern crate manager;

use std::path::PathBuf;

use error::CuckooMinerError;
use manager::{CuckooPluginManager, CuckooPluginCapabilities};

//Helper to convert from hex string
//avoids a lot of awkward byte array initialisation below
pub fn from_hex_string(in_str: &str) -> Vec<u8> {
	let mut bytes = Vec::new();
	for i in 0..(in_str.len() / 2) {
		let res = u8::from_str_radix(&in_str[2 * i..2 * i + 2], 16);
		match res {
			Ok(v) => bytes.push(v),
			Err(e) => println!("Problem with hex: {}", e),
		}
	}
	bytes
}

pub const DLL_SUFFIX: &str = ".cuckooplugin";

pub const TEST_PLUGIN_LIBS_CORE : [&str;3] = [
	"lean_cpu_16",
	"lean_cpu_30",
	"mean_cpu_30",
];

pub const TEST_PLUGIN_LIBS_OPTIONAL : [&str;1] = [
	"lean_cuda_30",
];


// Grin Pre and Post headers, into which a nonce is to be insterted for mutation
pub const SAMPLE_GRIN_PRE_HEADER_1:&str = "00000000000000118e0fe6bcfaa76c6795592339f27b6d330d8f9c4ac8e86171a66357d1\
    d0fce808000000005971f14f0000000000000000000000000000000000000000000000000000000000000000\
    3e1fcdd453ce51ffbb16dd200aeb9ef7375aec196e97094868428a7325e4a19b00";

pub const SAMPLE_GRIN_POST_HEADER_1:&str = "010a020364";

//hashes known to return a solution at cuckoo 30 and 16
pub const KNOWN_30_HASH_1:&str = "11c5059b4d4053131323fdfab6a6509d73ef22\
9aedc4073d5995c6edced5a3e6";

pub const KNOWN_16_HASH_1:&str = "5f16f104018fc651c00a280ba7a8b48db80b30\
20eed60f393bdcb17d0e646538";


// Helper to load plugins
pub fn get_plugin_vec(filter: &str) -> Vec<CuckooPluginCapabilities>{
	let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	d.push("../target/debug/plugins/");

	// get all plugins in directory
	let mut plugin_manager = CuckooPluginManager::new().unwrap();
	plugin_manager
		.load_plugin_dir(String::from(d.to_str().unwrap()))
		.expect("");

	// Get a list of installed plugins and capabilities
	plugin_manager.get_available_plugins(filter).unwrap()
}

