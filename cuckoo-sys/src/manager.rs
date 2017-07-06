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

use std::sync::{Mutex};
use std::env;
use std::path::{Path,PathBuf};
use std::io;
use std::path::MAIN_SEPARATOR;
use glob::glob;

use libloading;
use libc::*;

use cuckoo_config::*;

#[cfg(test)]
mod test;

type CuckooCall = unsafe extern fn(*const c_uchar, size_t, uint32_t, uint32_t, *mut uint32_t) -> uint32_t;
type CuckooDescription = unsafe extern fn(*mut c_uchar,*mut uint32_t,*mut c_uchar,*mut uint32_t);

#[cfg(target_os = "linux")]
static DLL_SUFFIX: &str=".so";
#[cfg(target_os = "macos")]
static DLL_SUFFIX: &str=".dylib";

lazy_static!{
    static ref loaded_library: Mutex<Option<libloading::Library>> = Mutex::new(None);
    static ref cuckoo_call: Mutex<Option<CuckooCall>> = Mutex::new(None);
    static ref cuckoo_description: Mutex<Option<CuckooDescription>> = Mutex::new(None);
}

fn abspath<P: AsRef<Path> + ?Sized>(relpath: &P) -> String { 
    let result=env::current_dir().map(|p| p.join(relpath.as_ref()));
    let full_path = result.unwrap();
    String::from(full_path.to_str().unwrap())
}

fn call_cuckoo_description(name_bytes: &mut [u8;256], name_bytes_len:&mut u32,
                           description_bytes: &mut [u8;256], description_bytes_len:&mut u32) 
    -> Result<(), CuckooMinerError>{
    let cuckoo_description_ref = cuckoo_description.lock().unwrap(); 
    match *cuckoo_description_ref {
        None => return Err(CuckooMinerError::PluginNotLoadedError(
            String::from("No miner plugin is loaded. Please call init() with the name of a valid mining plugin."))),
        Some(c) => unsafe {
                        c(name_bytes.as_mut_ptr(), name_bytes_len, 
                          description_bytes.as_mut_ptr(), description_bytes_len);
                        return Ok(());
                   },
        
    };
}

fn get_plugin_caps(full_path:String) 
    -> Result<CuckooPluginCapabilities, CuckooMinerError> {
        //println!("{:?}",path.display() );
        let mut caps=CuckooPluginCapabilities::default();
        load_lib(full_path.clone())?;
        let mut name_bytes:[u8;256]=[0;256];
        let mut description_bytes:[u8;256]=[0;256];
        let mut name_len=name_bytes.len() as u32;
        let mut desc_len=description_bytes.len() as u32;
        call_cuckoo_description(&mut name_bytes, &mut name_len, 
                                &mut description_bytes, &mut desc_len);
        
        let mut name_vec:Vec<u8> = Vec::new();
        for i in 0..name_len {
            name_vec.push(name_bytes[i as usize].clone());
        }
        
        let mut desc_vec:Vec<u8> = Vec::new();
        for i in 0..desc_len-1 {
            desc_vec.push(description_bytes[i as usize].clone());
        }
        
        caps.name=String::from_utf8(name_vec)?;
        caps.description=String::from_utf8(desc_vec)?;
        caps.full_path=full_path.clone();
        caps.file_name=String::from("");

        return Ok(caps);
}
// Gets info from all of the available plugins

pub fn get_available_plugins(config:&CuckooMinerConfig) 
        -> Result<Vec<CuckooPluginCapabilities>,CuckooMinerError>{
    let lib_full_path = abspath(Path::new(&config.plugin_dir));
    let glob_search_path = format!("{}/*cuckoo*{}", lib_full_path, DLL_SUFFIX);

    let mut result_vec:Vec<CuckooPluginCapabilities> = Vec::new();
    
    for entry in glob(&glob_search_path).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                
                let caps = get_plugin_caps(String::from(path.to_str().unwrap()))?;
                result_vec.push(caps);
            },
            Err(e) => println!("{:?}", e),
        }
    }

    if result_vec.len()==0 {
        return Err(CuckooMinerError::NoPluginsFoundError(
            format!("No plugins found in plugin directory {}", lib_full_path.clone())
            ));
    }

    Ok(result_vec)
}

// loads a library with given full path

fn load_lib(lib_full_path:String) -> Result<(), CuckooMinerError> {
    debug!("Loading miner plugin: {}", &lib_full_path);
    //TODO: check if lib is loaded and unload if so
    let result = libloading::Library::new(lib_full_path.clone());
    let loaded_lib = {
        match result {
            Ok(l) => l,
            Err(e) => {
                return Err(CuckooMinerError::PluginNotFoundError(lib_full_path));
            }
        }
    };

    let mut loaded_library_ref = loaded_library.lock().unwrap();
    *loaded_library_ref = Some(loaded_lib);

    let mut cuckoo_call_ref = cuckoo_call.lock().unwrap();
    let mut cuckoo_description_ref = cuckoo_description.lock().unwrap();
    unsafe {
        let fn_ref:CuckooCall = *loaded_library_ref.as_mut().unwrap().get(b"cuckoo_call\0")?;
        *cuckoo_call_ref = Some(fn_ref);

        let fn_ref:CuckooDescription = *loaded_library_ref.as_mut().unwrap().get(b"cuckoo_description\0")?;
        *cuckoo_description_ref = Some(fn_ref);
    }
    Ok(())
}

// Loads the appropriate cuckoo libary and needed function pointers based on
// the given configuration

pub fn load_cuckoo_lib(caps:&CuckooPluginCapabilities) -> Result<(), CuckooMinerError>{
    //let lib_full_path = abspath(Path::new(&config.plugin_dir));
    //let lib_name = format!("{}{}lib{}{}", lib_full_path, MAIN_SEPARATOR, &caps.plugin_name, DLL_SUFFIX);
    load_lib(caps.full_path.clone())
    
}

pub fn call_cuckoo(header: &[u8], num_threads: u32, num_trims:u32, solutions:&mut [u32; 42] ) -> Result<u32, CuckooMinerError> {
    let cuckoo_call_ref = cuckoo_call.lock().unwrap(); 
    match *cuckoo_call_ref {
        None => return Err(CuckooMinerError::PluginNotLoadedError(
            String::from("No miner plugin is loaded. Please call init() with the name of a valid mining plugin."))),
        Some(c) => unsafe {
                        return Ok(c(header.as_ptr(), header.len(), num_threads, 
                            num_trims, solutions.as_mut_ptr()));
                   },
        
    };

}


