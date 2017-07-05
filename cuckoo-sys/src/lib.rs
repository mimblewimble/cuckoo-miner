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
#[macro_use]
extern crate lazy_static;
extern crate libloading as lib;
extern crate libc;
extern crate cuckoo_config;

use std::sync::{Mutex};

use libc::*;
use cuckoo_config::*;

#[cfg(test)]
mod test;

type CuckooCall = unsafe extern fn(*const c_uchar, size_t, *mut uint32_t) -> uint32_t;

#[cfg(target_os = "linux")]
static DLL_SUFFIX: &str=".so";
#[cfg(target_os = "macos")]
static DLL_SUFFIX: &str=".dylib";

lazy_static!{
    static ref loaded_library: Mutex<Option<lib::Library>> = Mutex::new(None);
    static ref cuckoo_call: Mutex<Option<CuckooCall>> = Mutex::new(None);
}

// A low level struct indicating what dll to load and with what params
pub struct CuckooBindingConfig {
    //cuckoo size
    pub cuckoo_size:u8,
    pub cuckoo_impl:String,
}

impl Default for CuckooBindingConfig {
	fn default() -> CuckooBindingConfig {
		CuckooBindingConfig{
            cuckoo_impl: String::from("basic"),
            cuckoo_size: 12,
		}
	}
}

// Loads the appropriate cuckoo libary and needed function pointers based on
// the given configuration

pub fn load_cuckoo_lib(config:CuckooBindingConfig) -> Result<(), CuckooMinerError>{
    let lib_name = format!("libcuckoo_{}_{}{}",config.cuckoo_impl, config.cuckoo_size,DLL_SUFFIX);

    //TODO: check if lib is loaded and unload if so
    let result = lib::Library::new(lib_name.clone());
    let loaded_lib = {
        match result {
            Ok(l) => l,
            Err(e) => {
                return Err(CuckooMinerError::NotImplementedError(lib_name));
            }
        }
    };

    let mut loaded_library_ref = loaded_library.lock().unwrap();
    *loaded_library_ref = Some(loaded_lib);

    let mut cuckoo_call_ref = cuckoo_call.lock().unwrap();
    unsafe {
        let mut fn_ref:CuckooCall = *loaded_library_ref.as_mut().unwrap().get(b"cuckoo_call").unwrap();
        *cuckoo_call_ref = Some(fn_ref);
    }
    Ok(())
}

pub fn call_cuckoo(header: &[u8], solutions:&mut [u32; 42] ) -> Result<u32, CuckooMinerError> {
    let mut cuckoo_call_ref = cuckoo_call.lock().unwrap(); 
    match *cuckoo_call_ref {
        None => return Err(CuckooMinerError::MinerNotLoadedError(
            String::from("No miner plugin is loaded. Please call init() with the name of a valid mining plugin."))),
        Some(c) => unsafe {
                        return Ok(c(header.as_ptr(), header.len(), solutions.as_mut_ptr()));
                   },
        
    };

}


