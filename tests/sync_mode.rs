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

extern crate cuckoo_miner as cuckoo;
extern crate time;

pub mod common;

//mines plugin as requsted
#[test]
fn mine_once_cuckatoo_cuda_29() {
	//Should exercise lean/mean cpu at 1i9 for now
	let caps = common::get_plugin_vec("cuckatoo_cuda_29");
	for c in &caps {
		let mut plugin_path_vec:Vec<&str> = Vec::new();
		plugin_path_vec.push(&c.full_path);
		common::mine_once(&c.full_path, None);
	}
}

#[test]
fn mine_once_cuckatoo_lean_29() {
	//Should exercise lean/mean cpu at 1i9 for now
	let caps = common::get_plugin_vec("cuckatoo_lean_cpu_29");
	let mut params=Vec::new();
	params.push((String::from("NUM_THREADS"),0,2));
	for c in &caps {
		let mut plugin_path_vec:Vec<&str> = Vec::new();
		plugin_path_vec.push(&c.full_path);
		common::mine_once(&c.full_path, Some(params.clone()));
	}
}

//Mines for a bit on each available plugin, one after the other
#[test]
fn on_commit_mine_sync() {
	let caps = common::get_plugin_vec("16");
	for c in &caps {
		common::mine_sync_for_duration(&c.full_path, 10, None);
	}
}

//Same as above, but for cuda only
#[test]
fn on_cuda_commit_mine_sync() {
	let caps = common::get_plugin_vec("cuda_30");
	for c in &caps {
		common::mine_sync_for_duration(&c.full_path, 75, None);
	}
}

//Same as above, but for cuda only
#[test]
fn on_cuda_meaner_commit_mine_sync() {
	let caps = common::get_plugin_vec("cuda_meaner_30");
	for c in &caps {
		common::mine_sync_for_duration(&c.full_path, 75, None);
	}
}

//test for mean_16 compat
//(won't be efficient, but should stress-tes plugins nicely)
#[test]
fn manual_mean_16_compat() {
	let mut params=Vec::new();
	params.push((String::from("NUM_THREADS"),0,4));
	let caps = common::get_plugin_vec("mean_compat_cpu_16");
	common::mine_sync_for_duration(&caps[0].full_path, 3600, Some(params.clone()));
}
