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

extern crate miner;
extern crate error;
extern crate manager;
extern crate time;

mod common;

use std::path::PathBuf;

use error::CuckooMinerError;
use miner::{CuckooMinerConfig, CuckooMinerSolution, CuckooMiner};
use manager::{CuckooPluginManager, CuckooPluginCapabilities};

// Helper function, tests a particular miner implementation against a known set
fn mine_sync_for_duration(full_path:&str, duration_in_seconds: i64) {
	let mut config_vec=Vec::new();
	let mut config = CuckooMinerConfig::new();
	config.plugin_full_path = String::from(full_path);
	config_vec.push(config);

	let stat_check_interval = 3;
	let deadline = time::get_time().sec + duration_in_seconds;
	let mut next_stat_check = time::get_time().sec + stat_check_interval;

	let mut i=0;
	println!("Test mining for {} seconds, looking for difficulty > 0", duration_in_seconds);
	for c in config_vec.clone().into_iter(){
		println!("Plugin (Sync Mode): {}", c.plugin_full_path);
	}
	while time::get_time().sec < deadline {
		let miner = CuckooMiner::new(config_vec.clone()).expect("");
		let mut header:[u8; 32] = [0;32];
		let mut iterations=0;
		let mut solution = CuckooMinerSolution::new();
		loop {
			header[0]=i;
			//Mine on plugin loaded at index 0
			let result = miner.mine(&header, &mut solution, 0).unwrap();
			iterations+=1;
			if result == true {
				println!("Solution found after {} iterations: {}", i, solution);
				println!("For hash: {:?}", header);
				break;
			}
			if time::get_time().sec > deadline {
				println!("Exiting after {} iterations", iterations);
				break;
			}
			if time::get_time().sec >= next_stat_check {
				let stats_vec=miner.get_stats(0).unwrap();
				for s in stats_vec.into_iter() {
					let last_solution_time_secs = s.last_solution_time as f64 / 1000.0;
					let last_hashes_per_sec = 1.0 / last_solution_time_secs;
					println!("Plugin 0 - Device {} ({}) - Last Solution time: {}; Solutions per second: {:.*}", 
					s.device_id, s.device_name, last_solution_time_secs, 3, last_hashes_per_sec);
				}
				next_stat_check = time::get_time().sec + stat_check_interval;
			}
			i+=1;
			if i==255 {
				i=0;
			}
		}
	}
}

//Mines for a bit on each available plugin, one after the other
#[test]
fn on_commit_mine_sync() {
	let caps = common::get_plugin_vec("");
	for c in &caps {
		mine_sync_for_duration(&c.full_path, 75);
	}
}

//Same as above, but for cuda only
#[test]
fn on_cuda_commit_mine_sync() {
	let caps = common::get_plugin_vec("cuda");
	for c in &caps {
		mine_sync_for_duration(&c.full_path, 75);
	}
}
