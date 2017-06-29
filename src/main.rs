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

//! Testing binary for the cuckoo_miner lib project
//!

extern crate cuckoo_miner;
extern crate cuckoo_sys;
extern crate env_logger;


use cuckoo_miner::test_function;
use cuckoo_sys::{cuckoo_test_link, cuckoo_test_wrap};

fn main() {
    env_logger::init();
    println!("Hello, world!");
    unsafe {
        for i in 0..1000 {
            let result=cuckoo_test_wrap(12, i, 10, 0);
            if (result==1){
                break;
            }
            //if i%10 == 0 {
                println!("Count: {}", i);
            //}
        }
    }
    
}
