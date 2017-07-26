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
extern crate time;

use std::thread;


use miner::{CuckooMiner, CuckooMinerConfig, CuckooMinerSolution};
use manager::CuckooPluginManager;

static KNOWN_SEED_16:[u8;32] = [0xd9, 0x93, 0xac, 0x4a, 0xe3, 0xc7, 0xf9, 0xeb, 
                                0x34, 0xb2, 0x2e, 0x86, 0x85, 0x25, 0x64, 0xa9,
                                0xc1, 0x67, 0x2a, 0x35, 0x7a, 0x0a, 0x81, 0x80,
                                0x82, 0xc6, 0x0f, 0x2a, 0xb1, 0x5f, 0x6f, 0x67];
static KNOWN_SOLUTION_16:[u32;42] = [671, 2624, 3044, 4429, 4682, 4734, 6727, 7250, 8589, 
8717, 9718, 10192, 10458, 10504, 11294, 12699, 13143, 13147, 14170, 15805, 16197, 17322, 
18523, 19892, 20277, 22231, 22964, 22965, 23993, 24624, 26735, 26874, 27312, 27502, 28637, 
29606, 30616, 30674, 30727, 31162, 31466, 31706];

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
    let caps = plugin_manager.get_available_plugins("simple_16").unwrap();

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
    
    let mut miner = CuckooMiner::new(config).expect("");

    //Keep a structure to hold the solution.. this will be
    //filled out by the plugin
    let mut solution = CuckooMinerSolution::new();


    let pre_header="00000000000000118e0fe6bcfaa76c6795592339f27b6d330d8f9c4ac8e86171a66357d1\
    d0fce808000000005971f14f0000000000000000000000000000000000000000000000000000000000000000\
    3e1fcdd453ce51ffbb16dd200aeb9ef7375aec196e97094868428a7325e4a19b00";
    let post_header="010a020364";

    //miner.notify(1, pre_header, post_header, false);

    let duration_in_seconds=60;

    let deadline = time::get_time().sec + duration_in_seconds;
   
    while time::get_time().sec < deadline {
        
        miner.notify(1, pre_header, post_header, false);

        loop {
            if let Some(s) = miner.get_solution()  {
                println!("Sol found: {}, {:?}", s.get_nonce_as_u64(), s);
                //up to you to read it and check difficulty
                miner.stop_jobs();
                break;    
                
            }

        }
            //break;
        
        
    }

    //thread::sleep(time::Duration::from_millis(500));

    /*miner.notify(1, pre_header, post_header, false);

    loop {
        
        if let Some(s) = miner.get_solution() {
            miner.stop_jobs();
            //up to you to read it and check difficulty
            break;
        }

    }
    
    //thread::sleep(time::Duration::from_millis(2000));

    println!("Jobs should be stopped now");

    //thread::sleep(time::Duration::from_millis(5000));*/
        
    //Mine with given header and check for result
    /*let result = miner.mine(&KNOWN_SEED_16, &mut solution).unwrap();

    if result == true {
       println!("Solution found: {}", solution);
    } else {
       println!("No Solution found");
    }*/

}
