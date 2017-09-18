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

//! # Overview
//!
//! <b>cuckoo-miner</b> is a Rust wrapper around John Tromp's Cuckoo Miner
//! C implementations, intended primarily for use in the Grin MimbleWimble
//! blockhain development project. However, it is also suitable for use as
//! a standalone miner or by any other project needing to use the 
//! cuckoo cycle proof of work. cuckoo-miner is plugin based, and provides
//! a high level interface to load and work with C mining implementations.
//!
//! A brief description of basic operations follows, as well as some
//! examples of how cuckoo miner should be called.
//!
//! ## Interfaces
//!
//! The library provides 2 high level interfaces:
//!
//! The [CuckooPluginManager](struct.CuckooPluginManager.html)
//! takes care of querying and loading plugins. A caller can provide a directory
//! for the plugin manager to scan, and the manager will load each plugin and
//! populate a [CuckooPluginCapabilities](struct.CuckooPluginCapabilities.html)
//! for each, which will contain a description of the plugin as well as any parameters
//! that can be configured.
//!
//! The [CuckooMiner](struct.CuckooMiner.html) struct provides a
//! high-level interface that a caller can use to load and run one or many
//! simultaneous plugin mining implementations.
//!
//! ## Operational Modes
//!
//! The miner can be run in either synchronous or asynchronous mode.
//!
//! Syncronous mode uses the [`mine`](struct.CuckooMiner.html#method.mine) function,
//! which takes a complete hash, processes it within the calling thread via the plugin's 
//! [`call_cuckoo`](struct.PluginLibrary.html#method.call_cuckoo) function, 
//! and returns the result.
//!
//! Asynchronous mode uses the [`notify`](struct.CuckoMiner.html#method.notify) 
//! function, which takes the pre-nonce and
//! post-nonce parts of a block header, mutates it internally with a nonce, and
//! inserts the resulting hash into the plugin's internal queue for processing.
//! Solutions are placed into an output queue, which the calling thread can 
//! read ascynronously via a [job handle](struct.CuckooMinerJobHandle.html).
//!
//! Examples of using either mode follow:
//!
//! ## Example - Sync mode
//! ```
//! extern crate cuckoo_miner as cuckoo;
//! extern crate time;
//!
//! use std::path::PathBuf;
//!
//! let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
//! d.push("target/debug/plugins/");
//!
//! // load plugin manager
//! let mut plugin_manager = cuckoo::CuckooPluginManager::new().unwrap();
//! plugin_manager
//! 	.load_plugin_dir(String::from(d.to_str().unwrap()))
//! 	.expect("");
//!
//! // Load a single plugin using a filter
//! let caps=plugin_manager.get_available_plugins("lean_cpu_16").unwrap();
//! let mut config_vec=Vec::new();
//! let mut config = cuckoo::CuckooMinerConfig::new();
//! config.plugin_full_path = caps[0].full_path.clone();
//! config_vec.push(config);
//!
//! let duration_in_seconds=60;
//! let stat_check_interval = 3;
//! let deadline = time::get_time().sec + duration_in_seconds;
//! let mut next_stat_check = time::get_time().sec + stat_check_interval;
//!
//! let mut i=0;
//! println!("Test mining for {} seconds, looking for difficulty > 0", duration_in_seconds);
//! for c in config_vec.clone().into_iter(){
//! 	println!("Plugin (Sync Mode): {}", c.plugin_full_path);
//! }
//!
//! while time::get_time().sec < deadline {
//! let miner = cuckoo::CuckooMiner::new(config_vec.clone()).expect("");
//! 	//Mining with a dummy header here, but in reality this would be a passed in 
//! 	//header hash
//! 	let mut header:[u8; 32] = [0;32]; 
//! 	let mut iterations=0;
//! 	let mut solution = cuckoo::CuckooMinerSolution::new();
//! 	loop {
//! 		header[0]=i;
//! 		//Mine on plugin loaded at index 0 (which should be only one loaded in
//! 		//Sync mode
//! 		let result = miner.mine(&header, &mut solution, 0).unwrap();
//! 		iterations+=1;
//! 		if result == true {
//! 			println!("Solution found after {} iterations: {}", i, solution);
//! 			println!("For hash: {:?}", header);
//! 			break;
//! 		}
//! 		if time::get_time().sec > deadline {
//! 			println!("Exiting after {} iterations", iterations);
//! 			break;
//! 		}
//! 		if time::get_time().sec >= next_stat_check {
//! 			let stats_vec=miner.get_stats(0).unwrap();
//! 			for s in stats_vec.into_iter() {
//! 				let last_solution_time_secs = s.last_solution_time as f64 / 1000.0;
//! 				let last_hashes_per_sec = 1.0 / last_solution_time_secs;
//! 				println!("Plugin 0 - Device {} ({}) - Last Solution time: {}; Solutions per second: {:.*}", 
//! 				s.device_id, s.device_name, last_solution_time_secs, 3, last_hashes_per_sec);
//! 			}
//! 			next_stat_check = time::get_time().sec + stat_check_interval;
//! 		}
//! 		i+=1;
//! 		if i==255 {
//! 			i=0;
//! 		}
//! #   break;
//! 	}
//! # break;
//! }
//! ```
//!
//! ## Example - Async mode
//! ```
//! extern crate cuckoo_miner as cuckoo;
//! extern crate time;
//!
//! use std::path::PathBuf;
//!
//! //Grin Pre and Post headers, into which a nonce is to be insterted for mutation
//! let SAMPLE_GRIN_PRE_HEADER_1:&str = "00000000000000118e0fe6bcfaa76c6795592339f27b6d330d8f9c4ac8e86171a66357d1\
//!   d0fce808000000005971f14f0000000000000000000000000000000000000000000000000000000000000000\
//!   3e1fcdd453ce51ffbb16dd200aeb9ef7375aec196e97094868428a7325e4a19b00";
//! 
//! let SAMPLE_GRIN_POST_HEADER_1:&str = "010a020364";
//!
//! let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
//! d.push("target/debug/plugins/");
//!
//! // load plugin manager
//! let mut plugin_manager = cuckoo::CuckooPluginManager::new().unwrap();
//! plugin_manager
//! 	.load_plugin_dir(String::from(d.to_str().unwrap()))
//! 	.expect("");
//!
//! // Load a single pugin using a filter
//! let caps=plugin_manager.get_available_plugins("lean_cpu_16").unwrap();
//! let mut config_vec=Vec::new();
//! let mut config = cuckoo::CuckooMinerConfig::new();
//! config.plugin_full_path = caps[0].full_path.clone();
//! config_vec.push(config);
//!
//! let duration_in_seconds=60;
//! let stat_check_interval = 3;
//! let deadline = time::get_time().sec + duration_in_seconds;
//! let mut next_stat_check = time::get_time().sec + stat_check_interval;
//! let mut stats_updated=false;
//! 
//! while time::get_time().sec < deadline {
//! 
//! 	println!("Test mining for {} seconds, looking for difficulty > 0", duration_in_seconds);
//! 	let mut i=0;
//! 	for c in config_vec.clone().into_iter(){
//! 		println!("Plugin {}: {}", i, c.plugin_full_path);
//! 		i+=1;
//! 	}
//! 
//! 	// these always get consumed after a notify
//! 	let miner = cuckoo::CuckooMiner::new(config_vec.clone()).expect("");
//! 	let job_handle = miner.notify(1, SAMPLE_GRIN_PRE_HEADER_1, SAMPLE_GRIN_POST_HEADER_1, 0).unwrap();
//! 
//! 	loop {
//! 		if let Some(s) = job_handle.get_solution() {
//! 			println!("Sol found: {}, {:?}", s.get_nonce_as_u64(), s);
//! 			// up to you to read it and check difficulty
//! 			continue;
//! 		}
//! 		if time::get_time().sec >= next_stat_check {
//! 			let mut sps_total=0.0;
//! 			for index in 0..config_vec.len() {
//! 				let stats_vec=job_handle.get_stats(index).unwrap();
//! 				for s in stats_vec.into_iter() {
//! 					let last_solution_time_secs = s.last_solution_time as f64 / 1000.0;
//! 					let last_hashes_per_sec = 1.0 / last_solution_time_secs;
//! 					println!("Plugin {} - Device {} ({}) - Last Solution time: {}; Solutions per second: {:.*}", 
//! 					index,s.device_id, s.device_name, last_solution_time_secs, 3, last_hashes_per_sec);
//! 					if last_hashes_per_sec.is_finite() {
//! 						sps_total+=last_hashes_per_sec;
//! 					}
//! 					if last_solution_time_secs > 0.0 {
//! 						stats_updated = true;
//! 					}
//! 					i+=1;
//! 				}
//! 			}
//! 			println!("Total solutions per second: {}", sps_total);
//! 			next_stat_check = time::get_time().sec + stat_check_interval;
//! 		}
//! 		if time::get_time().sec > deadline {
//! 			println!("Stopping jobs and waiting for cleanup");
//! 			job_handle.stop_jobs();
//! 			break;
//! 		}
//! 		# break;
//! 	}
//! 	# break;
//! }
//! ```

#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![warn(missing_docs)]

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate regex;
extern crate rand;
extern crate byteorder;
extern crate crypto;
extern crate blake2_rfc as blake2;

extern crate libloading as libloading;
extern crate libc;

extern crate glob;

mod error;
mod miner;
mod manager;
mod cuckoo_sys;

pub use error::error::CuckooMinerError;

pub use miner::miner::{CuckooMinerConfig, CuckooMiner, CuckooMinerSolution, CuckooMinerJobHandle,
                CuckooMinerDeviceStats};

pub use manager::manager::{CuckooPluginManager, CuckooPluginCapabilities};

pub use cuckoo_sys::manager::PluginLibrary;
