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

    /// #Description 
    ///
    /// Call to the cuckoo_basic_mine function in the library, which calls the base
    /// cuckoo miner implementation found in cuckoo_miner.cpp.
    ///
    /// #Arguments
    ///
    /// * `edge_bits` the number of bits to use for edges, i.e. size of the graph
    /// * `header` The SHA3 hash to use for the seed to the internal SIPHASH function
    ///    which generates edge locations in the graph
    /// * `header_len` the length of the header
    /// * `sol_nonces` an array (which must be of size 42) in which solution nonces will
    ///    be stored if a solution is found
    ///
    /// Returns 1 if a solution is found, with the 42 solution nonces contained within
    /// sol_nonces. Returns 0 if no solution is found.
    ///
    /// #Example
    /// TBD
    ///

    pub fn cuckoo_basic_mine(edge_bits:c_uint, 
                             header: *const c_uchar, 
                             header_len: size_t,
                             sol_nonces: *mut uint32_t) -> uint32_t;

}
