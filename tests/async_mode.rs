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

//! Tests for async mode.. should be run with RUST_TEST_THREADS=1

pub mod common;

//mines for a bit on each available plugin, one after the other
#[test]
fn on_commit_mine_single_plugin_async() {
	//Should exercise lean/mean cpu at 16 for now
	let caps = common::get_plugin_vec("cpu_16");
	for c in &caps {
		let mut plugin_path_vec:Vec<&str> = Vec::new();
		plugin_path_vec.push(&c.full_path);
		common::mine_async_for_duration(plugin_path_vec, 10, None);
	}
}

#[test]
fn on_cuda_commit_mine_single_plugin_async() {
	let mut params=Vec::new();
	/*params.push((String::from("USE_DEVICE"),0,0));
	params.push((String::from("EXPAND"),0,0));
	params.push((String::from("N_TRIMS"),0,176));
	params.push((String::from("GEN_A_BLOCKS"),0,4096));
	params.push((String::from("GEN_A_TPB"),0,256));
	params.push((String::from("GEN_B_TPB"),0,128));
	params.push((String::from("TRIM_TPB"),0,512));
	params.push((String::from("TAIL_TPB"),0,1024));
	params.push((String::from("RECOVER_BLOCKS"),0,1024));
	params.push((String::from("RECOVER_TPB"),0,1024));*/
	let caps = common::get_plugin_vec("cuda_30");
	for c in &caps {
	 let mut plugin_path_vec:Vec<&str> = Vec::new();
		plugin_path_vec.push(&c.full_path);
		common::mine_async_for_duration(plugin_path_vec, 60, Some(params.clone()));
	}
}
//mine cuda and matrix (mean) miner for a bit
#[test]
fn on_cuda_commit_mine_mean_cpu_and_lean_cuda_async() {
	let caps = common::get_plugin_vec("");
	let mut plugin_path_vec:Vec<&str> = Vec::new();
	for c in &caps {
		if c.full_path.contains("lean_cuda_30") || c.full_path.contains("mean_cpu_30"){
			plugin_path_vec.push(&c.full_path);
		}
	}
	common::mine_async_for_duration(plugin_path_vec, 180, None);
}

//Mines for a bit on all available plugins at once
//(won't be efficient, but should stress-tes plugins nicely)
#[test]
fn on_commit_mine_plugins_async() {
	// Get a list of installed plugins and capabilities
	// only do cuckoo 30s
	let caps = common::get_plugin_vec("16");
	let mut plugin_path_vec:Vec<&str> = Vec::new();
	for c in &caps {
		//Have to confine this for the time being to 2, due to travis CI memory constraints
		if c.full_path.contains("lean_cpu") || c.full_path.contains("mean_cpu"){
			plugin_path_vec.push(&c.full_path);
		}
	}
	common::mine_async_for_duration(plugin_path_vec, 15, None);
}
