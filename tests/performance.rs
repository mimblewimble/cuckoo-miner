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

//! Performance-related tests go here

pub mod common;
use std::collections::HashMap;

//Test for profiling
#[test]
fn profile_mine_mean_30_async() {
	// Get a list of installed plugins and capabilities
	// only do cuckoo 30s
	let caps = common::get_plugin_vec("mean_cpu_30");
	let mut plugin_path_vec:Vec<&str> = Vec::new();
	for c in &caps {
		plugin_path_vec.push(&c.full_path);
	}
	common::mine_async_for_duration(plugin_path_vec, 120, None);
}

//Compare sync and async modes of mining, ensure
//they're taking roughly the same time
#[test]
fn perf_mean_30_compare() {
	let caps = common::get_plugin_vec("mean_cpu_16");
	let mut plugin_path_vec:Vec<&str> = Vec::new();
	for c in &caps {
		plugin_path_vec.push(&c.full_path);
	}
	let mut params=HashMap::new();
	params.insert(String::from("NUM_THREADS"), 1);
	common::mine_sync_for_duration(plugin_path_vec[0].clone(), 20, Some(params.clone()));
	common::mine_async_for_duration(plugin_path_vec, 20, Some(params.clone()));
}
