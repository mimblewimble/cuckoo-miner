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

extern crate cuckoo_miner;
extern crate cuckoo_config;
extern crate env_logger;


use cuckoo_config::*;
use cuckoo_miner::CuckooMiner;

fn main() {

    let test_header = [0xA6, 0xC1, 0x64, 0x43, 0xFC, 0x82, 0x25, 0x0B, 
                       0x49, 0xC7, 0xFA, 0xA3, 0x87, 0x6E, 0x7A, 0xB8, 
                       0x9B, 0xA6, 0x87, 0x91, 0x8C, 0xB0, 0x0C, 0x4C, 
                       0x10, 0xD6, 0x62, 0x5E, 0x3A, 0x2E, 0x7B, 0xCC];
    env_logger::init();
    
    let mut config = CuckooMinerConfig::new();
    config.num_threads=1;
    let mut miner = CuckooMiner::new(config).unwrap();
    let caps = miner.get_available_plugins().unwrap();

    //Let's just run through them all and mine the same thing
    for c in &caps {
        println!("Test finding solution on plugin with caps: [{}]", c);

        miner.init(c).expect("Miner initialisation failed.");
        let mut solution = CuckooMinerSolution::new();
        
        let result = miner.mine(&test_header, &mut solution).unwrap();

        if result == true {
           println!("Solution found: {}", solution);
        } else {
           println!("No Solution found");
        }
    }



    //Just a testing stub for the time being
    //let result=miner.mine(&test_header, &mut solution).unwrap();

    /*if result == true {
        println!("Solution found: {}", solution);
    }*/

}
