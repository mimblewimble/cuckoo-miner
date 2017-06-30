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

extern crate libloading as lib;
extern crate libc;

use libc::*;

#[cfg(test)]
mod test;

pub fn call_cuckoo(header: &[u8], solutions:&mut [u32; 42] ) -> u32 {
    let lib = lib::Library::new("libcuckoo_10.so").unwrap();
    unsafe {
        let func: lib::Symbol<unsafe extern fn(*const c_uchar, size_t, *mut uint32_t) -> uint32_t> 
            = lib.get(b"cuckoo_call").unwrap();
        return func(header.as_ptr(), header.len(), solutions.as_mut_ptr());
    }
}


