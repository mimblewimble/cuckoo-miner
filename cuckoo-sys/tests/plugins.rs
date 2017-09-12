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

/// Tests exercising the loading and unloading of plugins, as well as the
/// existence and correct functionality of each plugin function

extern crate cuckoo_sys;
extern crate error;

use std::path::PathBuf;

use error::CuckooMinerError;
use cuckoo_sys::PluginLibrary;

// OS-specific library extensions
#[cfg(target_os = "linux")]
static DLL_SUFFIX: &str = ".so";
#[cfg(target_os = "macos")]
static DLL_SUFFIX: &str = ".dylib";
#[cfg(target_os = "windows")]
static DLL_SUFFIX: &str = ".dll";

const TEST_PLUGIN_LIBS_CORE : [&str;3] = [
	"libcuckoo_lean_cpu_16",
	"libcuckoo_lean_cpu_30",
	"libcuckoo_mean_cpu_30",
];

const TEST_PLUGIN_LIBS_OPTIONAL : [&str;1] = [
	"libcuckoo_lean_cuda_30",
];

//Helper to load a plugin library
fn load_plugin_lib(plugin:&str) -> Result<PluginLibrary, CuckooMinerError> {
	let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	d.push(format!("../target/debug/plugins/{}{}", plugin, DLL_SUFFIX).as_str());
	PluginLibrary::new(d.to_str().unwrap())
}

//Helper to load all known plugin libraries
fn load_all_plugins() -> Vec<PluginLibrary>{
	let mut plugin_libs:Vec<PluginLibrary> = Vec::new();
	for p in TEST_PLUGIN_LIBS_CORE.into_iter(){
		plugin_libs.push(load_plugin_lib(p).unwrap());
	}
	for p in TEST_PLUGIN_LIBS_OPTIONAL.into_iter(){
		let pl = load_plugin_lib(p);
		if let Ok(p) = pl {
			plugin_libs.push(p);
		}
	}
	plugin_libs
}

//loads and unloads a plugin many times
#[test]
fn plugin_loading(){
	//core plugins should be built on all systems, fail if they don't exist
	for _ in 0..100 {
		for p in TEST_PLUGIN_LIBS_CORE.into_iter() {
			let pl = load_plugin_lib(p).unwrap();
			pl.unload();
		}
	}
	//only test these if they do exist (cuda, etc)
	for _ in 0..100 {
		for p in TEST_PLUGIN_LIBS_OPTIONAL.into_iter() {
			let pl = load_plugin_lib(p);
			if let Err(_) = pl {
				break;
			}
			pl.unwrap().unload();
		}
	}
}

//Loads all plugins at once
#[test]
fn plugin_multiple_loading(){
	let _p=load_all_plugins();
}

//tests cuckoo_init() on all available plugins
//multiple calls to cuckoo init should be fine
#[test]
fn cuckoo_init(){
	let iterations = 100;
	let plugins = load_all_plugins();
	for p in plugins.into_iter() {
		for _ in 0..iterations {
			p.call_cuckoo_init();
		}
	}
}

// Helper to test call_cuckoo_description and return results
// Ensures that all plugins *probably* don't overwrite
// their buffers as they contain an null zero somewhere 
// within the rust-enforced length

fn call_cuckoo_description_tests(pl: &PluginLibrary){
	///Test normal value
	const LENGTH:usize = 256;
	let mut name_bytes:[u8;LENGTH]=[0;LENGTH];
	let mut description_bytes:[u8;LENGTH]=[0;LENGTH];
	let mut name_len=name_bytes.len() as u32;
	let mut desc_len=description_bytes.len() as u32;
	pl.call_cuckoo_description(&mut name_bytes, &mut name_len,
		&mut description_bytes, &mut desc_len);
	let result_name = String::from_utf8(name_bytes.to_vec()).unwrap();
	let result_name_length = result_name.find('\0');
	let result_desc = String::from_utf8(description_bytes.to_vec()).unwrap();
	let result_desc_length = result_desc.find('\0');
	
	//Check name is less than rust-enforced length,
	//if there's no \0 the plugin is likely overwriting the buffer
	println!("Name: **{}**", result_name);
	assert!(result_name.len()>0);
	assert!(result_name_length != None);
	assert!(name_len!=0);
	println!("Length: {}", result_name_length.unwrap());
	println!("Description: **{}**", result_desc);
	assert!(result_desc.len()>0);
	assert!(result_desc_length != None);
	assert!(desc_len!=0);
	println!("Length: {}", result_desc_length.unwrap());

	assert!(result_name.contains("cuckoo"));
	assert!(result_desc.contains("cuckoo"));

	///Test provided buffer too short
	const TOO_SHORT_LENGTH:usize = 16;
	let mut name_bytes:[u8;TOO_SHORT_LENGTH]=[0;TOO_SHORT_LENGTH];
	let mut description_bytes:[u8;TOO_SHORT_LENGTH]=[0;TOO_SHORT_LENGTH];
	let mut name_len=name_bytes.len() as u32;
	let mut desc_len=description_bytes.len() as u32;
	pl.call_cuckoo_description(&mut name_bytes, &mut name_len,
		&mut description_bytes, &mut desc_len);
	assert!(name_len==0);
	assert!(desc_len==0);
}

//tests call_cuckoo_description() on all available plugins
#[test]
fn cuckoo_description(){
	let iterations = 100;
	let plugins = load_all_plugins();
	for p in plugins.into_iter() {
		for _ in 0..iterations {
			call_cuckoo_description_tests(&p);
		}
	}
}

// Helper to test call_cuckoo_parameter_list and return results
// Ensures that all plugins *probably* don't overwrite
// their buffers as they contain an null zero somewhere 
// within the rust-enforced length

fn call_cuckoo_parameter_list_tests(pl: &PluginLibrary){
	///Test normal rust-enforced value
	const LENGTH:usize = 1024;
	let mut param_list_bytes:[u8;LENGTH]=[0;LENGTH];
	let mut param_list_bytes_len=param_list_bytes.len() as u32;
	let ret_val=pl.call_cuckoo_parameter_list(&mut param_list_bytes,
		&mut param_list_bytes_len);
	let result_list = String::from_utf8(param_list_bytes.to_vec()).unwrap();
	let result_list_null_index = result_list.find('\0');
	
	//Check name is less than rust-enforced length,
	//if there's no \0 the plugin is likely overwriting the buffer
	println!("Plugin: {}", pl.lib_full_path);
	assert!(ret_val==0);
	println!("Parameter List: **{}**", result_list);
	assert!(result_list.len()>0);
	assert!(result_list_null_index != None);
	println!("Null Index: {}", result_list_null_index.unwrap());

	//Basic form check... json parsing can be checked higher up
	assert!(result_list.contains("["));

	///Test provided length too short
	///Plugin shouldn't explode as a result
	const TOO_SHORT_LENGTH:usize = 64;
	let mut param_list_bytes:[u8;TOO_SHORT_LENGTH]=[0;TOO_SHORT_LENGTH];
	let mut param_list_bytes_len=param_list_bytes.len() as u32;
	let ret_val=pl.call_cuckoo_parameter_list(&mut param_list_bytes,
		&mut param_list_bytes_len);
	let result_list = String::from_utf8(param_list_bytes.to_vec()).unwrap();
	assert!(ret_val==3);
}

//tests call_cuckoo_parameter_list() on all available plugins
#[test]
fn cuckoo_parameter_list(){
	let iterations = 100;
	let plugins = load_all_plugins();
	for p in plugins.into_iter() {
		for _ in 0..iterations {
			call_cuckoo_parameter_list_tests(&p);
		}
	}
}

// Helper to test call_cuckoo_get_parameter and return results
// Ensures that all plugins *probably* don't overwrite
// their buffers as they contain an null zero somewhere 
// within the rust-enforced length

fn call_cuckoo_get_parameter_tests(pl: &PluginLibrary){
	println!("Plugin: {}", pl.lib_full_path);
	//normal param that should be there
	let name = "NUM_THREADS";
	let mut num_threads:u32 = 0;
	let return_value = pl.call_cuckoo_get_parameter(name.as_bytes(), &mut num_threads);
	assert!(num_threads > 0);
	assert!(return_value == 0);

//normal param that's not there
	let name = "SANDWICHES";
	let mut num_sandwiches:u32 = 0;
	let return_value = pl.call_cuckoo_get_parameter(name.as_bytes(), &mut num_sandwiches);
	assert!(num_sandwiches == 0);
	assert!(return_value == 1);

//normal param that's not there and is too long
	let name = "SANDWICHESSANDWICHESSANDWICHESSANDWICHESSANDWICHESSANDWICHESANDWICHESSAES";
	let mut num_sandwiches:u32 = 0;
	let return_value = pl.call_cuckoo_get_parameter(name.as_bytes(), &mut num_sandwiches);
	assert!(num_sandwiches == 0);
	assert!(return_value == 4);
}

//tests call_cuckoo_get_parameter() on all available plugins
#[test]
fn cuckoo_get_parameter(){
	let iterations = 100;
	let plugins = load_all_plugins();
	for p in plugins.into_iter() {
		for _ in 0..iterations {
			call_cuckoo_get_parameter_tests(&p);
		}
	}
}
