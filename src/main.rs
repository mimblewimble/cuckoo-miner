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

//! Testing/Sample app using the cuckoo_miner lib
//!

extern crate miner;
extern crate config;
extern crate env_logger;


use config::*;
use miner::{CuckooMiner,CuckooPluginManager};

fn main() {

    let test_header = [0xA6, 0xC1, 0x64, 0x43, 0xFC, 0x82, 0x25, 0x0B, 
                       0x49, 0xC7, 0xFA, 0xA3, 0x87, 0x6E, 0x7A, 0xB8, 
                       0x9B, 0xA6, 0x87, 0x91, 0x8C, 0xB0, 0x0C, 0x4C, 
                       0x10, 0xD6, 0x62, 0x5E, 0x3A, 0x2E, 0x7B, 0xCC];
    env_logger::init();

    //First, load and query the plugins in the given directory
    let mut plugin_manager = CuckooPluginManager::new().unwrap();
    let result=plugin_manager.load_plugin_dir(String::from("target/debug")).expect("");
    //Get a list of installed plugins and capabilities
    let caps = plugin_manager.get_available_plugins().unwrap();

    //Print all available plugins
    for c in caps {
        println!("Found plugin: [{}]", c);
    }

    //Select a plugin somehow, and insert it into the miner configuration
    //being created below
    
    let mut config = CuckooMinerConfig::new();
    config.plugin_full_path = caps[0].full_path.clone();
    
    //set the number of threads for the miner to use
    config.num_threads=4;

    //set the number of trimes, 0 lets the plugin decide
    config.num_trims=0;

    //Build a new miner with this info, which will load
    //the associated plugin and 
    
    let mut miner = CuckooMiner::new(config).expect("");

    //Keep a structure to hold the solution.. this will be
    //filled out by the plugin
    let mut solution = CuckooMinerSolution::new();
        
    //Mine with given header and check for result
    let result = miner.mine(&test_header, &mut solution).unwrap();

    if result == true {
       println!("Solution found: {}", solution);
    } else {
       println!("No Solution found");
    }

}
