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



use error::CuckooMinerError;
use miner::{CuckooMinerConfig, CuckooMinerSolution, CuckooMiner};
use manager::{CuckooPluginManager, CuckooPluginCapabilities};

// Helper function, tests a particular miner implementation against a known set
// that should have a result
fn mine_for_duration(plugin_filter: &str, duration_in_seconds: i64) {


	let pre_header = "00000000000000118e0fe6bcfaa76c6795592339f27b6d330d8f9c4ac8e86171a66357d1\
    d0fce808000000005971f14f0000000000000000000000000000000000000000000000000000000000000000\
    3e1fcdd453ce51ffbb16dd200aeb9ef7375aec196e97094868428a7325e4a19b00";
	let post_header = "010a020364";

	// First, load and query the plugins in the given directory
	let mut plugin_manager = CuckooPluginManager::new().unwrap();
	let result = plugin_manager
		.load_plugin_dir(String::from("target/debug"))
		.expect("");
	// Get a list of installed plugins and capabilities
	let caps = plugin_manager.get_available_plugins(plugin_filter).unwrap();

	let mut config = CuckooMinerConfig::new();
	config.plugin_full_path = caps[0].full_path.clone();

	let deadline = time::get_time().sec + duration_in_seconds;

	while time::get_time().sec < deadline {

		// these always get consumed after a notify
		let mut miner = CuckooMiner::new(config.clone()).expect("");
		let job_handle = miner.notify(1, pre_header, post_header, 10).unwrap();

		loop {
			if let Some(s) = job_handle.get_solution() {
				println!("Sol found: {}, {:?}", s.get_nonce_as_u64(), s);
				// up to you to read it and check difficulty
				job_handle.stop_jobs();
				std::thread::sleep(std::time::Duration::from_millis(20));
				break;

			}
			if time::get_time().sec < deadline {
				job_handle.stop_jobs();
				break;
			}

		}
		// break;


	}
}

#[test]
fn mine_async() {
	mine_for_duration("simple_16", 5);
	std::thread::sleep(std::time::Duration::from_millis(20));
	mine_for_duration("edgetrim_16", 5);
}
