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

//! Tests to exercise calling the different cuckoo_miner implementations

extern crate env_logger;
extern crate cuckoo_miner;

use cuckoo_miner::miner::CuckooMiner;
use cuckoo_miner::types::{CuckooMinerImplType, 
                CuckooMinerConfig, 
                CuckooMinerError,
                CuckooMinerSolution};




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

// Performs a basic test on cuckoo12 for a known hash and solution set
#[test]
fn mine_basic_cuckoo_12() {

    env_logger::init();
    
    let mut config = CuckooMinerConfig::new();
    let miner = CuckooMiner::new(config).unwrap();
    let mut solution = CuckooMinerSolution::new();
    let mut expected_solution = CuckooMinerSolution::new();
    expected_solution.set_nonces(TEST_SOLUTION_1);

    //Just a testing stub for the time being
    let result=miner.mine(&TEST_HEADER_1, &mut solution).unwrap();

    if result == true {
        println!("Solution found: {}", solution);
    }
    assert_eq!(solution, expected_solution);

}