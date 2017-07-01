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


//! Raw bindings for cuckoo miner
//!

#![allow(dead_code, non_camel_case_types, non_upper_case_globals, non_snake_case)]

extern crate libc;

#[cfg(test)]
mod test;

use libc::*;

extern "C" {
    pub fn cuckoo_basic_mine(edge_bits:c_int, 
                             header: *const c_uchar, 
                             header_len: size_t,
                             sol_nonces: *mut uint32_t) -> uint32_t;

}
