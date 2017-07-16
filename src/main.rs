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
extern crate error;
extern crate manager;

use miner::{CuckooMiner, CuckooMinerConfig, CuckooMinerSolution};
use manager::CuckooPluginManager;

fn main() {

    //this should have a solution under cuckoo25
    let test_header = [0xae,0x71,0xf3,0x6d,0xe6,0x4c,0x2d,0xde,
                       0x50,0xbb,0x29,0x93,0xb3,0x4e,0x61,0xd6,
                       0xfb,0xa2,0xbe,0xe0,0xd0,0x52,0xcb,0x2d,
                       0xc9,0x56,0x06,0x4f,0x8a,0x8a,0xcd,0x54];

    //First, load and query the plugins in the given directory
    let mut plugin_manager = CuckooPluginManager::new().unwrap();
    plugin_manager.load_plugin_dir(String::from("target/debug")).expect("");
    //Get a list of installed plugins and capabilities
    let caps = plugin_manager.get_available_plugins("edgetrim_25").unwrap();

    //Print all available plugins
    for c in &caps {
        println!("Found plugin: [{}]", c);
    }

    //Select a plugin somehow, and insert it into the miner configuration
    //being created below
    
    let mut config = CuckooMinerConfig::new();
    config.plugin_full_path = caps[0].full_path.clone();
    //config.parameter_list.insert(String::from("NUM_TRIMS"), 5);
    //config.parameter_list.insert(String::from("NUM_THREADS"), 8);
    
    //Build a new miner with this info, which will load
    //the associated plugin and 
    
    let miner = CuckooMiner::new(config).expect("");

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
