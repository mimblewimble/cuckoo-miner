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
	let caps = common::get_plugin_vec("mean_cpu_16");
	for c in &caps {
		let mut plugin_path_vec:Vec<&str> = Vec::new();
		plugin_path_vec.push(&c.full_path);
		common::mine_async_for_duration(plugin_path_vec, 30, None);
	}
}

//Same as above, but only for cuda
#[test]
fn on_cuda_commit_mine_single_plugin_async() {
	let caps = common::get_plugin_vec("cuda");
	for c in &caps {
	 let mut plugin_path_vec:Vec<&str> = Vec::new();
		plugin_path_vec.push(&c.full_path);
		common::mine_async_for_duration(plugin_path_vec, 60, None);
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
		if c.full_path.contains("lean_cuda") || c.full_path.contains("mean_cpu"){
			plugin_path_vec.push(&c.full_path);
		}
	}
	common::mine_async_for_duration(plugin_path_vec, 120, None);
}
