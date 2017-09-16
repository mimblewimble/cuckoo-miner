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
// that should have a result
fn mine_for_duration(full_path: &str, duration_in_seconds: i64) {
	let mut config = CuckooMinerConfig::new();
	config.plugin_full_path = String::from(full_path);

  let mut config_vec=Vec::new();
	config_vec.push(config.clone());

	let stat_check_interval = 3;
	let deadline = time::get_time().sec + duration_in_seconds;
	let mut next_stat_check = time::get_time().sec + stat_check_interval;
	let mut stats_updated=false;

	while time::get_time().sec < deadline {

		println!("Test mining indefinitely, looking for difficulty > 0");
		println!("Loaded from: {}", config.plugin_full_path);

		// these always get consumed after a notify
		let miner = CuckooMiner::new(config_vec.clone()).expect("");
		let job_handle = miner.notify(1, common::SAMPLE_GRIN_PRE_HEADER_1, common::SAMPLE_GRIN_POST_HEADER_1, 0).unwrap();

		loop {
			if let Some(s) = job_handle.get_solution() {
				println!("Sol found: {}, {:?}", s.get_nonce_as_u64(), s);
				// up to you to read it and check difficulty
				continue;
			}
			if time::get_time().sec >= next_stat_check {
				let stats_vec=job_handle.get_stats(0).unwrap();
				for s in stats_vec.into_iter() {
					let last_solution_time_secs = s.last_solution_time as f64 / 1000.0;
					let last_hashes_per_sec = 1.0 / last_solution_time_secs;
					println!("Plugin 0 - Device {} ({}) - Last Solution time: {}; Solutions per second: {:.*}", 
					s.device_id, s.device_name, last_solution_time_secs, 3, last_hashes_per_sec);
					next_stat_check = time::get_time().sec + stat_check_interval;
					if last_solution_time_secs > 0.0 {
						stats_updated = true;
					}
				}
			}
			if time::get_time().sec > deadline {
				println!("Stopping jobs and waiting for cleanup");
				job_handle.stop_jobs();
				break;
			}
		}
	}
	assert!(stats_updated==true);
}

#[test]
fn mine_async() {
	let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	d.push("../target/debug/plugins/");

	// get all plugins in directory
	let mut plugin_manager = CuckooPluginManager::new().unwrap();
	let result = plugin_manager
		.load_plugin_dir(String::from(d.to_str().unwrap()))
		.expect("");

	// Get a list of installed plugins and capabilities
	let caps = plugin_manager.get_available_plugins("mean_cpu").unwrap();

	for c in &caps {
		mine_for_duration(&c.full_path, 75); //std::thread::sleep(std::time::Duration::from_millis(20));
	}

}
