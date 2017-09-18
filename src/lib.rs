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

//! cuckoo-miner is a Rust wrapper around John Tromp's Cuckoo Miner
//! C implementations, intended primarily for use in the Grin MimbleWimble
//! blockhain development project. However, it is also suitable for use as
//! a standalone miner or by any other project needing to use the 
//! cuckoo cycle proof of work. cuckoo-miner is plugin based, and provides
//! a high level interface to load and work with C mining implementations.
//!
//! A brief description of basic operations follows, as well as some
//! examples of how cuckoo miner should be called.
//!
//! The library provides 2 high level interfaces:
//!
//! The [CuckooPluginManager](../manager/struct.CuckooPluginManager.html)
//! takes care of querying and loading plugins. A caller can provide a directory
//! for the plugin manager to scan, and the manager will load each plugin and
//! populate a [CuckooPluginCapabilities](../manager/struct.CuckooPluginCapabilities.html)
//! for each, which will contain a description of the plugin as well as any parameters
//! that can be configured.
//!
//! The [CuckooMiner](../miner/struct.CuckoonMiner.html) struct provides a
//! high-level interface that a caller can use to load and run one or many
//! simultaneous plugin mining implementations.
//!
//! The miner can be run in either syncronous or asynchronous mode.
//!
//! Syncronous mode uses the `mine` function,, which takes a complete hash,
//! processes it within the calling thread via the plugin's `call_cuckoo` function, 
//! and returns the result.
//!
//! Asynchronous mode uses the `notify` function, which takes the pre-nonce and
//! post-nonce parts of a block header, mutates it internally with a nonce, and
//! inserts the resulting hash into the plugin's internal queue for processing.
//! Solutions are placed into an output queue, which the calling thread can 
//! read ascynronously via a job handle.
//!
//! Examples of using either mode follow:
//!
//! #Example - Sync mode
//! ```
//! extern crate cuckoo_miner;
//! extern crate time;
//!
//! use cuckoo_miner::error::CuckooMinerError;
//! use cuckoo_miner::miner::{CuckooMinerConfig, CuckooMinerSolution, CuckooMiner};
//! use cuckoo_miner::manager::{CuckooPluginManager, CuckooPluginCapabilities};
//!
//! // load plugin manager
//! let mut plugin_manager = CuckooPluginManager::new().unwrap();
//! plugin_manager
//! 	.load_plugin_dir(String::from(d.to_str().unwrap()))
//! 	.expect("");
//!
//! // Load a single plugin using a filter
//! let caps=plugin_manager.get_available_plugins("mean_cpu_16").unwrap();
//! let mut config_vec=Vec::new();
//! let mut config = CuckooMinerConfig::new();
//! config.plugin_full_path = caps[0].plugin_full_path().clone();
//! config_vec.push(config);
//!
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
//! let miner = CuckooMiner::new(config_vec.clone()).expect("");
//! 	//Mining with a dummy header here, but in reality this would be a passed in 
//! 	//header hash
//! 	let mut header:[u8; 32] = [0;32]; 
//! 	let mut iterations=0;
//! 	let mut solution = CuckooMinerSolution::new();
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
//! 	}
//! }
//! ```
//!
//! #Example - Async mode
//!
//! ```text
//!  let mut config = CuckooMinerConfig::new();
//!  config.plugin_full_path = caps[0].full_path.clone();
//!
//!  //set the number of threads for the miner to use
//!  config.num_threads=2;
//!
//!  //set the number of trimes, 0 lets the plugin decide
//!  config.num_trims=0;
//!
//!  //Build a new miner with this info, which will load
//!  //the associated plugin and
//!
//!  let mut miner = CuckooMiner::new(config).expect("");
//!
//!  //Keep a structure to hold the solution.. this will be
//!  //filled out by the plugin
//!  let mut solution = CuckooMinerSolution::new();
//!
//!  //Sample header 'parts' to mutate, the parts before and after the nonce
//!
//!  let pre_nonce="00000000000000118e0fe6bcfaa76c6795592339f27b6d330d8f9c4ac8e86171a66357d1\
//!      d0fce808000000005971f14f0000000000000000000000000000000000000000000000000000000000000000\
//!      3e1fcdd453ce51ffbb16dd200aeb9ef7375aec196e97094868428a7325e4a19b00";
//!  let post_nonce="010a020364";
//!
//!  //mine until a certain time
//!  let deadline = time::get_time().sec + duration_in_seconds;
//!  while time::get_time().sec < deadline {
//!
//!     //Build a new miner with the configuration, as notify
//!     //will consume it
//!     let mut miner = CuckooMiner::new(config.clone()).expect("");
//!
//!     //Call notify, which starts processing.
//!     //The job handle contains methods to control the running job and read
//!     //results
//!     let job_handle=miner.notify(1, pre_nonce, post_nonce, 10).unwrap();
//!
//!     loop {
//!         if let Some(s) = job_handle.get_solution()  {
//!         println!("Sol found: {}, {:?}", s.get_nonce_as_u64(), s);
//!
//!             job_handle.stop_jobs();
//!             /// Process the solution in s
//!             /// ...
//!             ///
//!
//!             break;
//!
//!         }
//!         if time::get_time().sec < deadline {
//!             job_handle.stop_jobs();
//!             break;
//!         }
//!
//!     }
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
