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

mod common;
use common::{SAMPLE_GRIN_PRE_HEADER_1, SAMPLE_GRIN_POST_HEADER_1};

use cuckoo::{CuckooMinerConfig, CuckooMiner};

// Helper function, tests a particular miner implementation against a known set
fn mine_async_for_duration(full_paths: Vec<&str>, duration_in_seconds: i64) {
	let mut config_vec=Vec::new();
	for p in full_paths.into_iter() {
		let mut config = CuckooMinerConfig::new();
		config.plugin_full_path = String::from(p);
		config_vec.push(config);
	}

	let stat_check_interval = 3;
	let mut deadline = time::get_time().sec + duration_in_seconds;
	let mut next_stat_check = time::get_time().sec + stat_check_interval;
	let mut stats_updated=false;
	//for CI testing on slower servers
	//if we're trying to quit and there are no stats yet, keep going for a bit
	let mut extra_time=false;
	let extra_time_value=180;

	while time::get_time().sec < deadline {

		println!("Test mining for {} seconds, looking for difficulty > 0", duration_in_seconds);
		let mut i=0;
		for c in config_vec.clone().into_iter(){
			println!("Plugin {}: {}", i, c.plugin_full_path);
			i+=1;
		}

		// these always get consumed after a notify
		let miner = CuckooMiner::new(config_vec.clone()).expect("");
		let job_handle = miner.notify(1, SAMPLE_GRIN_PRE_HEADER_1, SAMPLE_GRIN_POST_HEADER_1, 0).unwrap();

		loop {
			if let Some(s) = job_handle.get_solution() {
				println!("Sol found: {}, {:?}", s.get_nonce_as_u64(), s);
				// up to you to read it and check difficulty
				continue;
			}
			if time::get_time().sec >= next_stat_check {
				let mut sps_total=0.0;
				for index in 0..config_vec.len() {
					let stats_vec=job_handle.get_stats(index);
					if let Err(e) = stats_vec {
						panic!("Error getting stats: {:?}", e);
					}
					for s in stats_vec.unwrap().into_iter() {
						let last_solution_time_secs = s.last_solution_time as f64 / 1000.0;
						let last_hashes_per_sec = 1.0 / last_solution_time_secs;
						println!("Plugin {} - Device {} ({}) - Last Solution time: {}; Solutions per second: {:.*}", 
						index,s.device_id, s.device_name, last_solution_time_secs, 3, last_hashes_per_sec);
						if last_hashes_per_sec.is_finite() {
							sps_total+=last_hashes_per_sec;
						}
						if last_solution_time_secs > 0.0 {
							stats_updated = true;
							if extra_time {
								break;
							}
						}
						i+=1;
					}
				}
				println!("Total solutions per second: {}", sps_total);
				next_stat_check = time::get_time().sec + stat_check_interval;
			}
			if time::get_time().sec > deadline {
				if !stats_updated && !extra_time {
					extra_time=true;
					deadline+=extra_time_value;
				}
				println!("Stopping jobs and waiting for cleanup");
				job_handle.stop_jobs();
				break;
			}
		}
	}
	assert!(stats_updated==true);
}

//mines for a bit on each available plugin, one after the other
#[test]
fn on_commit_mine_single_plugin_async() {
	let caps = common::get_plugin_vec("");
	for c in &caps {
	 let mut plugin_path_vec:Vec<&str> = Vec::new();
		plugin_path_vec.push(&c.full_path);
		mine_async_for_duration(plugin_path_vec, 180);
	}
}

//Same as above, but only for cuda
#[test]
fn on_cuda_commit_mine_single_plugin_async() {
	let caps = common::get_plugin_vec("cuda");
	for c in &caps {
	 let mut plugin_path_vec:Vec<&str> = Vec::new();
		plugin_path_vec.push(&c.full_path);
		mine_async_for_duration(plugin_path_vec, 180);
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
	mine_async_for_duration(plugin_path_vec, 180);
}

//Mines for a bit on all available plugins at once
//(won't be efficient, but should stress-tes plugins nicely)
#[test]
fn on_commit_mine_all_plugins_async() {
	// Get a list of installed plugins and capabilities
	// only do cuckoo 30s
	let caps = common::get_plugin_vec("30");
	let mut plugin_path_vec:Vec<&str> = Vec::new();
	for c in &caps {
		plugin_path_vec.push(&c.full_path);
	}
	mine_async_for_duration(plugin_path_vec, 120);
}
