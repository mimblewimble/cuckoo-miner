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
extern crate env_logger;
extern crate crypto;


use crypto::digest::Digest;
use crypto::sha2::Sha256;

use error::CuckooMinerError;
use miner::{CuckooMinerConfig, CuckooMinerSolution, CuckooMiner};
use manager::{CuckooPluginManager, CuckooPluginCapabilities};

use std::time::{Duration, SystemTime};

static TEST_HEADER_1:[u8;32] = [0xA6, 0xC1, 0x64, 0x43, 0xFC, 0x82, 0x25, 0x0B, 
                             0x49, 0xC7, 0xFA, 0xA3, 0x87, 0x6E, 0x7A, 0xB8, 
                             0x9B, 0xA6, 0x87, 0x91, 0x8C, 0xB0, 0x0C, 0x4C, 
                             0x10, 0xD6, 0x62, 0x5E, 0x3A, 0x2E, 0x7B, 0xCC];
static TEST_SOLUTION_1:[u32;42] = [1, 17, 122, 171, 238, 289, 340, 
                                   341, 478, 492, 513, 550, 555, 625, 
                                   762, 792, 803, 811, 829, 830, 939, 
                                   978, 1082, 1086, 1134, 1204, 1226,
                                   1230, 1316, 1353, 1364, 1415, 1442,
                                   1554, 1674, 1689, 1752, 1819, 1870,
                                   1877, 1888, 1948];

// Helper function, tests a particular miner implementation against a known set
// that should have a result
fn test_for_known_set(plugin_filter:&str, 
                      input_header: &[u8;32],
                      expected_result_nonces:CuckooMinerSolution, 
                      num_threads:u32){
    env_logger::init();

    //First, load and query the plugins in the given directory
    let mut plugin_manager = CuckooPluginManager::new().unwrap();
    let result=plugin_manager.load_plugin_dir(String::from("target/debug")).expect("");
    //Get a list of installed plugins and capabilities
    let caps = plugin_manager.get_available_plugins(plugin_filter).unwrap();

    //Print all available plugins
    for c in &caps {
        println!("Found plugin: [{}]", c);
    }

    //Select a plugin somehow, and insert it into the miner configuration
    //being created below
    
    let mut config = CuckooMinerConfig::new();
    config.plugin_full_path = caps[0].full_path.clone();
    
    //set the number of threads for the miner to use
    config.num_threads=num_threads;

    //set the number of trimes, 0 lets the plugin decide
    config.num_trims=0;

    //Build a new miner with this info, which will load
    //the associated plugin and 
    
    let mut miner = CuckooMiner::new(config).expect("");

    //Keep a structure to hold the solution.. this will be
    //filled out by the plugin
    let mut solution = CuckooMinerSolution::new();

    let result = miner.mine(input_header, &mut solution).unwrap();

    assert!(solution == expected_result_nonces);
   
}

// Helper function to mine using a semi random-hash until a solution
// is found

fn mine_until_solution_found(plugin_filter:&str,
                             num_threads:u32) {
    env_logger::init();

    //First, load and query the plugins in the given directory
    let mut plugin_manager = CuckooPluginManager::new().unwrap();
    let result=plugin_manager.load_plugin_dir(String::from("target/debug")).expect("");
    //Get a list of installed plugins and capabilities
    let caps = plugin_manager.get_available_plugins(plugin_filter).unwrap();

    //Print all available plugins
    for c in &caps {
        println!("Found plugin: [{}]", c);
    }

    //Select a plugin somehow, and insert it into the miner configuration
    //being created below
    
    let mut config = CuckooMinerConfig::new();
    config.plugin_full_path = caps[0].full_path.clone();
    
    //set the number of threads for the miner to use
    config.num_threads=num_threads;

    //set the number of trimes, 0 lets the plugin decide
    config.num_trims=0;

    //Build a new miner with this info, which will load
    //the associated plugin and 
    
    let mut miner = CuckooMiner::new(config).expect("");

    //Keep a structure to hold the solution.. this will be
    //filled out by the plugin
    let mut solution = CuckooMinerSolution::new();

    let mut i = 0;
    let start_seed = SystemTime::now();
    loop {
        let input = format!("{:?}{}",start_seed, i);
        let mut sha = Sha256::new();
        sha.input_str(&input);
        
        let mut bytes:[u8;32]=[0;32];
        sha.result(&mut bytes);

        //Mine away until we get a solution
        let result = miner.mine(&bytes, &mut solution).unwrap();

        if result == true {
            println!("Solution found after {} iterations: {}", i, solution);
            println!("For seed: {}", sha.result_str());
            break;
        } 
        
        i+=1;

    }

}

// Performs basic test mining on plugins, exercising the number of threads
// being passed in

#[test]
fn mine_plugins_until_found() {

    env_logger::init();
    for i in 1..8 {
        mine_until_solution_found("edgetrim_16", i);
        mine_until_solution_found("edgetrim_20", i);
        //mine_until_solution_found("edgetrim_25", i);
    }
}


// Load and unload all plugins and read their capabilities a few zillion times,
// To ensure nothing is going funny and nothing is segfaulting
#[test]
fn abuse_lib_loading() {
        
    let mut plugin_manager = CuckooPluginManager::new().unwrap();
    
    for i in 0..10000 {
        let result=plugin_manager.load_plugin_dir(String::from("target/debug")).expect("");
        //Get a list of installed plugins and capabilities
        let caps = plugin_manager.get_available_plugins("").unwrap();
    }    
}