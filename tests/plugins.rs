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

//! Tests meant to exercise all of the included plugins, to ensure that
//! they're all behaving correctly, not seg-faulting, etc.
//!
//! Going to have to see about parallel threading here.. running these
//! in parallel is probably not going to work

extern crate miner;
extern crate error;
extern crate manager;
extern crate crypto;

extern crate blake2_rfc as blake2;


use crypto::digest::Digest;
use crypto::sha2::Sha256;

use blake2::blake2b::Blake2b;

use error::CuckooMinerError;
use miner::{CuckooMinerConfig, CuckooMinerSolution, CuckooMiner};
use manager::{CuckooPluginManager, CuckooPluginCapabilities};

use std::time::{Duration, SystemTime};

static KNOWN_SEED_16: [u8; 32] = [
	0xd9,
	0x93,
	0xac,
	0x4a,
	0xe3,
	0xc7,
	0xf9,
	0xeb,
	0x34,
	0xb2,
	0x2e,
	0x86,
	0x85,
	0x25,
	0x64,
	0xa9,
	0xc1,
	0x67,
	0x2a,
	0x35,
	0x7a,
	0x0a,
	0x81,
	0x80,
	0x82,
	0xc6,
	0x0f,
	0x2a,
	0xb1,
	0x5f,
	0x6f,
	0x67,
];
static KNOWN_SOLUTION_16: [u32; 42] = [
	671,
	2624,
	3044,
	4429,
	4682,
	4734,
	6727,
	7250,
	8589,
	8717,
	9718,
	10192,
	10458,
	10504,
	11294,
	12699,
	13143,
	13147,
	14170,
	15805,
	16197,
	17322,
	18523,
	19892,
	20277,
	22231,
	22964,
	22965,
	23993,
	24624,
	26735,
	26874,
	27312,
	27502,
	28637,
	29606,
	30616,
	30674,
	30727,
	31162,
	31466,
	31706,
];

static KNOWN_SEED_20: [u8; 32] = [
	0xa7,
	0x02,
	0x9c,
	0xe6,
	0x70,
	0xe7,
	0x81,
	0xc3,
	0xa4,
	0xe7,
	0x55,
	0x68,
	0x3a,
	0x3b,
	0x6f,
	0xb9,
	0xaa,
	0x94,
	0x05,
	0x1b,
	0x33,
	0xd6,
	0x36,
	0x2a,
	0x3e,
	0x9e,
	0x55,
	0x90,
	0x09,
	0xb4,
	0xf4,
	0xfe,
];

static KNOWN_SOLUTION_20: [u32; 42] = [
	32076,
	36881,
	47709,
	57359,
	64750,
	69514,
	73241,
	88561,
	102044,
	104123,
	116418,
	122634,
	142323,
	142981,
	158102,
	159410,
	162383,
	190470,
	201841,
	251006,
	251517,
	332329,
	332343,
	342736,
	353590,
	354383,
	371966,
	377849,
	396260,
	405969,
	409556,
	410934,
	431522,
	439336,
	446297,
	446492,
	466421,
	477455,
	477827,
	481560,
	497684,
	503781,
];

static KNOWN_SEED_25: [u8; 32] = [
	0xae,
	0x71,
	0xf3,
	0x6d,
	0xe6,
	0x4c,
	0x2d,
	0xde,
	0x50,
	0xbb,
	0x29,
	0x93,
	0xb3,
	0x4e,
	0x61,
	0xd6,
	0xfb,
	0xa2,
	0xbe,
	0xe0,
	0xd0,
	0x52,
	0xcb,
	0x2d,
	0xc9,
	0x56,
	0x06,
	0x4f,
	0x8a,
	0x8a,
	0xcd,
	0x54,
];

static KNOWN_SOLUTION_25: [u32; 42] = [
	1300934,
	1777326,
	2387832,
	2870554,
	3228439,
	3449448,
	3538120,
	3690741,
	3793489,
	3807270,
	4077938,
	4090426,
	5013330,
	5308734,
	5387944,
	5587565,
	5694295,
	5697872,
	5890353,
	5960490,
	6095207,
	7704889,
	7758084,
	7978579,
	9034912,
	9063269,
	9912285,
	10064065,
	10076572,
	10397286,
	11228343,
	11261568,
	11735542,
	12212627,
	12269888,
	12284159,
	13037702,
	13543482,
	13549661,
	14269021,
	15500440,
	16267217,
];


static KNOWN_SEED_28: [u8; 32] = [
	0x16,
	0x18,
	0x5a,
	0x0b,
	0xa6,
	0x69,
	0x79,
	0xee,
	0x7b,
	0xbe,
	0x90,
	0x69,
	0xb2,
	0x59,
	0xa7,
	0x72,
	0x43,
	0x47,
	0x23,
	0x84,
	0x70,
	0x56,
	0x80,
	0xb0,
	0x41,
	0x16,
	0x25,
	0x9b,
	0x9a,
	0xda,
	0x8a,
	0xa7,
];

static KNOWN_SOLUTION_28: [u32; 42] = [
	2053444,
	6783171,
	7967041,
	12789270,
	13395794,
	13771078,
	16986929,
	19950826,
	23430950,
	24098180,
	28738520,
	32416841,
	34750211,
	37910068,
	39346098,
	45710505,
	48966978,
	49092338,
	51201247,
	53197413,
	56395283,
	59614723,
	73017231,
	73602630,
	75080794,
	76514134,
	78668221,
	84999426,
	87359028,
	87607101,
	87906374,
	88966031,
	94069571,
	96282504,
	98112159,
	106275507,
	117538021,
	119734709,
	120246550,
	127011166,
	130229154,
	132524931,
];

static KNOWN_TEST_28: [u8; 32] = [
	0x79,
	0x00,
	0x00,
	0x00,
	0x00,
	0x00,
	0x00,
	0x00,
	0x00,
	0x00,
	0x00,
	0x00,
	0x00,
	0x00,
	0x00,
	0x00,
	0x00,
	0x00,
	0x00,
	0x00,
	0x00,
	0x00,
	0x00,
	0x00,
	0x00,
	0x00,
	0x00,
	0x00,
	0x00,
	0x00,
	0x00,
	0x00,
];



// Helper function, tests a particular miner implementation against a known set
// that should have a result
fn test_for_known_set(
	plugin_filter: &str,
	input_header: &[u8; 32],
	num_threads: u32,
) -> Option<CuckooMinerSolution> {

	// First, load and query the plugins in the given directory
	let mut plugin_manager = CuckooPluginManager::new().unwrap();
	let result = plugin_manager
		.load_plugin_dir(String::from("target/debug"))
		.expect("");
	// Get a list of installed plugins and capabilities
	let caps = plugin_manager.get_available_plugins(plugin_filter).unwrap();

	// Print all available plugins
	for c in &caps {
		println!("Found plugin: [{}]", c);
		// Select a plugin somehow, and insert it into the miner configuration
		// being created below

		let mut config = CuckooMinerConfig::new();
		config.plugin_full_path = c.full_path.clone();

		let mut miner = CuckooMiner::new(config).expect("");

		let mut solution = CuckooMinerSolution::new();

		let result = miner.mine(input_header, &mut solution).unwrap();

		if result {
			println!("Solution found: {} - {}", c.name, solution);
			return Some(solution);
		} else {
			println!("No solution found.");
			return None;
		}
	}
	None
}

// Helper function to mine using a semi random-hash until a solution
// is found

fn mine_until_solution_found(plugin_filter: &str, num_threads: u32) {

	// First, load and query the plugins in the given directory
	let mut plugin_manager = CuckooPluginManager::new().unwrap();
	let result = plugin_manager
		.load_plugin_dir(String::from("target/debug"))
		.expect("");
	// Get a list of installed plugins and capabilities
	let caps = plugin_manager.get_available_plugins(plugin_filter).unwrap();

	// Print all available plugins
	for c in &caps {
		println!("Found plugin: [{}]", c);
	}

	// Select a plugin somehow, and insert it into the miner configuration
	// being created below

	let mut config = CuckooMinerConfig::new();
	config.plugin_full_path = caps[0].full_path.clone();


	// Build a new miner with this info, which will load
	// the associated plugin and

	let mut miner = CuckooMiner::new(config).expect("");

	// Keep a structure to hold the solution.. this will be
	// filled out by the plugin
	let mut solution = CuckooMinerSolution::new();

	let mut i = 0;
	let start_seed = SystemTime::now();
	loop {
		let input = format!("{:?}{}", start_seed, i);
		let mut sha = Sha256::new();
		sha.input_str(&input);

		let mut bytes: [u8; 32] = [0; 32];
		sha.result(&mut bytes);

		// Mine away until we get a solution
		let result = miner.mine(&bytes, &mut solution).unwrap();

		if result == true {
			println!("Solution found after {} iterations: {}", i, solution);
			println!("For seed: {}", sha.result_str());
			break;
		}

		i += 1;

	}

}

// Tests all versions of plugins against known seeds and solution sets

#[test]
fn test_known_solutions() {
	let mut solution = CuckooMinerSolution::new();
	solution.set_solution(KNOWN_SOLUTION_16);
	test_for_known_set("cuda_28", &KNOWN_TEST_28, 1);
	panic!("stop");



	/*solution = CuckooMinerSolution::new();
    solution.set_solution(KNOWN_SOLUTION_20);
    test_for_known_set("20", &KNOWN_SEED_20, solution, 1);*/

	/*solution = CuckooMinerSolution::new();
    solution.set_solution(KNOWN_SOLUTION_25);
    test_for_known_set("25", &KNOWN_SEED_25, solution, 1);*/

	/*solution = CuckooMinerSolution::new();
    solution.set_solution(KNOWN_SOLUTION_20);
    test_for_known_set("cuda_20", &KNOWN_SEED_20, solution, 8);*/
}

// Performs basic test mining on plugins, finding a solution

#[test]
fn mine_plugins_until_found() {

	mine_until_solution_found("cuda_28", 0);
	// mine_until_solution_found("simple_16", 4);

	panic!("stop");
}


// Load and unload all plugins and read their capabilities a few zillion times,
// To ensure nothing is going funny and nothing is segfaulting
#[test]
fn abuse_lib_loading() {

	let mut plugin_manager = CuckooPluginManager::new().unwrap();

	for i in 0..1000 {
		let result = plugin_manager
			.load_plugin_dir(String::from("target/debug"))
			.expect("");
		// Get a list of installed plugins and capabilities
		let caps = plugin_manager.get_available_plugins("").unwrap();
		for c in &caps {
			println!("Found plugin: [{}]", c);
		}
	}

}
