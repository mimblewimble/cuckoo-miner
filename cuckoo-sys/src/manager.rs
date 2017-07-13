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

//! Low-Level manager for loading and unloading plugins. These functions
//! should generally not be called directly by most consumers, who should
//! be using the high level interfaces found in the config, manager, and 
//! miner modules. These functions are meant for internal cuckoo-miner crates, 
//! and will not be exposed to other projects including the cuckoo-miner crate.

use std::sync::{Mutex};

use libloading;
use libc::*;

#[cfg(test)]
mod test;

use error::CuckooMinerError;

// PRIVATE MEMBERS

// Type definitions corresponding to each function that the plugin implements

type CuckooInit = unsafe extern fn();
type CuckooCall = unsafe extern fn(*const c_uchar, uint32_t, *mut uint32_t) -> uint32_t;
type CuckooDescription = unsafe extern fn(*mut c_uchar,*mut uint32_t,*mut c_uchar,*mut uint32_t);
type CuckooParameterList = unsafe extern fn(*mut c_uchar,*mut uint32_t) -> uint32_t;
type CuckooSetParameter = unsafe extern fn(*const c_uchar, uint32_t, uint32_t) -> uint32_t;
type CuckooGetParameter = unsafe extern fn(*const c_uchar, uint32_t, *mut uint32_t) -> uint32_t;

// Keep static references to the library and each call that a plugin can expose
// wrapped in mutex, for theoretical thread-safety, though it's unlikely that
// a caller would want to be calling a miner from multiple threads. Should
// leave it up to the miner to multithread itself as it sees fit.

lazy_static!{
    static ref LOADED_LIBRARY: Mutex<Option<libloading::Library>> = Mutex::new(None);
    static ref CUCKOO_INIT: Mutex<Option<CuckooInit>> = Mutex::new(None);
    static ref CUCKOO_CALL: Mutex<Option<CuckooCall>> = Mutex::new(None);
    static ref CUCKOO_DESCRIPTION: Mutex<Option<CuckooDescription>> = Mutex::new(None);
    static ref CUCKOO_PARAMETER_LIST: Mutex<Option<CuckooParameterList>> = Mutex::new(None);
    static ref CUCKOO_GET_PARAMETER: Mutex<Option<CuckooGetParameter>> = Mutex::new(None);
    static ref CUCKOO_SET_PARAMETER: Mutex<Option<CuckooSetParameter>> = Mutex::new(None);
}

// Loads the library at lib_full_path into the LOADED_LIBRARY static,
// as well as all associated plugin functions into their statics

fn load_lib(lib_full_path:&str) -> Result<(), CuckooMinerError> {
    debug!("Loading miner plugin: {}", &lib_full_path);
    let mut loaded_library_ref = LOADED_LIBRARY.lock().unwrap();
    
    let result = libloading::Library::new(lib_full_path.clone());
    let loaded_lib = {
        match result {
            Ok(l) => l,
            Err(_) => {
                return Err(CuckooMinerError::PluginNotFoundError(String::from(lib_full_path)));
            }
        }
    };

    *loaded_library_ref = Some(loaded_lib);

    {
        let mut cuckoo_init_ref = CUCKOO_INIT.lock().unwrap();
        let mut cuckoo_call_ref = CUCKOO_CALL.lock().unwrap();
        let mut cuckoo_description_ref = CUCKOO_DESCRIPTION.lock().unwrap();
        let mut cuckoo_parameter_list_ref = CUCKOO_PARAMETER_LIST.lock().unwrap();
        let mut cuckoo_get_parameter_ref = CUCKOO_GET_PARAMETER.lock().unwrap();
        let mut cuckoo_set_parameter_ref = CUCKOO_SET_PARAMETER.lock().unwrap();
        unsafe {
            let fn_ref:CuckooCall = *loaded_library_ref.as_mut().unwrap().get(b"cuckoo_call\0")?;
            *cuckoo_call_ref = Some(fn_ref);

            let fn_ref:CuckooInit = *loaded_library_ref.as_mut().unwrap().get(b"cuckoo_init\0")?;
            *cuckoo_init_ref = Some(fn_ref);

            let fn_ref:CuckooDescription = *loaded_library_ref.as_mut().unwrap().get(b"cuckoo_description\0")?;
            *cuckoo_description_ref = Some(fn_ref);

            let fn_ref:CuckooParameterList = *loaded_library_ref.as_mut().unwrap().get(b"cuckoo_parameter_list\0")?;
            *cuckoo_parameter_list_ref = Some(fn_ref);

            let fn_ref:CuckooGetParameter = *loaded_library_ref.as_mut().unwrap().get(b"cuckoo_get_parameter\0")?;
            *cuckoo_get_parameter_ref = Some(fn_ref);

            let fn_ref:CuckooSetParameter = *loaded_library_ref.as_mut().unwrap().get(b"cuckoo_set_parameter\0")?;
            *cuckoo_set_parameter_ref = Some(fn_ref);
        }
    }
    
    //automagically call the init
    call_cuckoo_init().unwrap();
    Ok(())
}

/// #Description 
///
/// Unloads the currently loaded plugin and all symbols.
///
/// #Arguments
///
/// None
///
/// #Returns
///
/// Nothing
///

pub fn unload_cuckoo_lib(){

    let cuckoo_get_parameter_ref = CUCKOO_GET_PARAMETER.lock().unwrap();
    drop(cuckoo_get_parameter_ref);

    let cuckoo_set_parameter_ref = CUCKOO_SET_PARAMETER.lock().unwrap();
    drop(cuckoo_set_parameter_ref);

    let cuckoo_parameter_list_ref = CUCKOO_PARAMETER_LIST.lock().unwrap();
    drop(cuckoo_parameter_list_ref);

    let cuckoo_call_ref = CUCKOO_CALL.lock().unwrap();
    drop(cuckoo_call_ref);
    
    let cuckoo_description_ref = CUCKOO_DESCRIPTION.lock().unwrap();
    drop(cuckoo_description_ref);

    let loaded_library_ref = LOADED_LIBRARY.lock().unwrap();
    drop(loaded_library_ref);

    
}


// PUBLIC FUNCTIONS

/// #Description 
///
/// Loads a cuckoo plugin library with the given full path, loading the library
/// as well as static references to the library's set of plugin functions.
///
/// #Arguments
///
/// * `full_path` The full path to the plugin library .so/.dylib 
///
/// #Returns
///
/// Ok if successful, a [CuckooMinerError](../../error/error/enum.CuckooMinerError.html) 
/// with specific detail if an error is encountered.
///
/// #Example
///
/// This example assumes that `cuckoo_call` below is a mutex containing a loaded
/// library symbol corresponding to this call.
/// 
/// ```
///  load_cuckoo_lib("/path/to/cuckoo/plugins/cuckoo_simple_30.so")
/// ```
///

pub fn load_cuckoo_lib(full_path:&str) -> Result<(), CuckooMinerError>{
    let result=load_lib(full_path);
    if let Err(e) = result {return Err(e)}
    Ok(()) 
}

/// #Description 
///
/// Initialises the cuckoo plugin, mostly allowing it to write a list of its accepted
/// parameters. This should be called just after the plugin is loaded
///
/// #Arguments
///
/// * None
///
/// #Returns
///
/// * Nothing
///
///

pub fn call_cuckoo_init() 
    -> Result<(), CuckooMinerError>{
    let cuckoo_init_ref = CUCKOO_INIT.lock().unwrap(); 
    match *cuckoo_init_ref {
        None => return Err(CuckooMinerError::PluginNotLoadedError(
            String::from("No miner plugin is loaded. Please call init() with the name of a valid mining plugin."))),
        Some(c) => unsafe {
                        c();
                        return Ok(());
                   },
        
    };
}

/// #Description 
///
/// Call to the cuckoo_call function of the currently loaded plugin, which will perform 
/// a Cuckoo Cycle on the given seed, returning the first solution (a length 42 cycle)
/// that is found. The implementation details are dependent on particular loaded plugin.
///
/// #Arguments
///
/// * `header` (IN) A reference to a block of [u8] bytes to use for the seed to the 
///    internal SIPHASH function which generates edge locations in the graph. In practice, 
///    this is a SHA3 hash of a Grin blockheader, but from the plugin's perspective this 
///    can be anything.
///
/// * `solutions` (OUT) A caller-allocated array of 42 unsigned bytes. This currently must
///    be of size 42, corresponding to a conventional cuckoo-cycle solution length. 
///    If a solution is found, the solution nonces will be stored in this array, otherwise,
///    they will be left untouched.
///
/// #Returns
///
/// Ok(1) if a solution is found, with the 42 solution nonces contained within
/// `sol_nonces`. Returns Ok(0) if no solution is found and `sol_nonces` remains
/// untouched. A [CuckooMinerError](../../error/error/enum.CuckooMinerError.html) 
/// will be returned if there is no plugin loaded, or if there is an error calling the function.
///
/// #Example
/// 
/// ```
///     match call_cuckoo(header, 
///                       &mut solution.solution_nonces) {
///         Ok(result) => {
///             match result {
///                 1 => Ok(true),
///                 0 => Ok(false),
///                 _ => Err(CuckooMinerError::UnexpectedResultError(result))
///             },
///             Err(e) => Err(CuckooMinerError::PluginNotLoadedError(
///                 String::from("Please call init to load a miner plug-in"))),
///      }
/// ```
///

pub fn call_cuckoo(header: &[u8], solutions:&mut [u32; 42] ) -> Result<u32, CuckooMinerError> {
    debug!("Calling loaded miner: header {:?}", header);
    let cuckoo_call_ref = CUCKOO_CALL.lock().unwrap(); 
    match *cuckoo_call_ref {
        None => return Err(CuckooMinerError::PluginNotLoadedError(
            String::from("No miner plugin is loaded. Please call init() with the name of a valid mining plugin."))),
        Some(c) => unsafe {
                        return Ok(c(header.as_ptr(), header.len() as u32, solutions.as_mut_ptr()));
                   },
        
    };

}

/// #Description 
/// Call to the call_cuckoo_description function of the currently loaded plugin, which will 
/// return various information about the plugin, including it's name, description, and
/// other information to be added soon.
///
/// #Arguments
///
/// * `name_bytes` (OUT) A caller-allocated u8 array to which the plugin will write its
/// name. 
///
/// * `name_bytes_len` (IN-OUT) When called, this should contain the maximum number of bytes
/// the plugin should write to `name_bytes`. Upon return, this is filled with the number
/// of bytes that were written to `name_bytes`.
///
/// * `description_bytes` (OUT) A caller-allocated u8 array to which the plugin will write its
/// description. 
///
/// * `description_bytes_len` (IN-OUT) When called, this should contain the maximum number of bytes
/// the plugin should write to `description_bytes`. Upon return, this is filled with the number
/// of bytes that were written to `description_bytes`.
///
///
/// #Returns
///
/// Ok() if the call was successful, otherwise a 
/// [CuckooMinerError](../../error/error/enum.CuckooMinerError.html) with specific details
/// of the error
///
/// #Example
/// 
/// ```
///  load_cuckoo_lib(&full_path)?;
///  let mut name_bytes:[u8;256]=[0;256];
///  let mut description_bytes:[u8;256]=[0;256];
///  let mut name_len=name_bytes.len() as u32;
///  let mut desc_len=description_bytes.len() as u32;
///  call_cuckoo_description(&mut name_bytes, &mut name_len, 
///                          &mut description_bytes, &mut desc_len);
/// ```
///

pub fn call_cuckoo_description(name_bytes: &mut [u8;256], name_bytes_len:&mut u32,
                           description_bytes: &mut [u8;256], description_bytes_len:&mut u32) 
    -> Result<(), CuckooMinerError>{
    let cuckoo_description_ref = CUCKOO_DESCRIPTION.lock().unwrap(); 
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

/// #Description 
///
/// Call to the cuckoo_call_parameter_list function of the currently loaded plugin, 
/// which will provide an informative JSON array of the parameters that the plugin supports, as well
/// as their descriptions and range of values.
///
/// #Arguments
///
/// * `param_list_bytes` (OUT) A reference to a block of [u8] bytes to fill with the JSON
///    result array
///
/// * `param_list_len` (IN-OUT) When called, this should contain the maximum number of bytes
///    the plugin should write to `param_list_bytes`. Upon return, this is filled with the number
///    of bytes that were written to `param_list_bytes`.
///
/// #Returns
///
/// 0 if the parameter list was retrived, and the result is stored in `param_list_bytes`
/// 3 if the buffer and size given was too small to store the parameters
///
/// #Example
/// 
/// ```
///   let mut param_list_bytes:[u8;1024]=[0;1024];
///   let mut param_list_len=param_list_bytes.len() as u32;
///   //get a list of parameters
///   let parameter_list=call_cuckoo_parameter_list(&mut param_list_bytes, &mut param_list_len);
/// ```
///

pub fn call_cuckoo_parameter_list(param_list_bytes: &mut [u8], param_list_len:&mut u32) 
    -> Result<u32, CuckooMinerError>{
    let cuckoo_parameter_list_ref = CUCKOO_PARAMETER_LIST.lock().unwrap(); 
    match *cuckoo_parameter_list_ref {
        None => return Err(CuckooMinerError::PluginNotLoadedError(
            String::from("No miner plugin is loaded. Please call init() with the name of a valid mining plugin."))),
        Some(c) => unsafe {
                        return Ok(c(param_list_bytes.as_mut_ptr(), param_list_len));
                   },
        
    };
}

/// #Description 
///
/// Retrieves the value of a parameter from the currently loaded plugin
///
/// #Arguments
///
/// * `name_bytes` (IN) A reference to a block of [u8] bytes storing the parameter name
///
/// * `value` (OUT) A reference where the parameter value will be stored
///
/// #Returns
///
/// 0 if the parameter was retrived, and the result is stored in `value`
/// 1 if the parameter does not exist
///
/// #Example
/// 
/// ```
///   let String name = "NUM_THREADS";
///   let mut value:u32 = 0;
///   let return_code = call_cuckoo_get_parameter(name.as_bytes(), &value)?;
/// ```
///

pub fn call_cuckoo_get_parameter(name_bytes: &[u8], value:&mut u32) 
    -> Result<u32, CuckooMinerError>{
    let cuckoo_get_parameter_ref = CUCKOO_GET_PARAMETER.lock().unwrap(); 
    match *cuckoo_get_parameter_ref {
        None => return Err(CuckooMinerError::PluginNotLoadedError(
            String::from("No miner plugin is loaded. Please call init() with the name of a valid mining plugin."))),
        Some(c) => unsafe {
                        return Ok(c(name_bytes.as_ptr(), name_bytes.len() as u32, value));
                   },
        
    };
}

/// #Description 
///
/// Sets the value of a parameter in the currently loaded plugin
///
/// #Arguments
///
/// * `name_bytes` (IN) A reference to a block of [u8] bytes storing the parameter name
///
/// * `value` (IN) The value to which to set the parameter
///
/// #Returns
///
/// 0 if the parameter was retrived, and the result is stored in `value`
/// 1 if the parameter does not exist
/// 2 if the parameter exists, but is outside the allowed range set by the plugin
///
/// #Example
/// 
/// ```
///   let String name = "NUM_THREADS";
///   let return_code = call_cuckoo_set_parameter(name.as_bytes(), 8)?;
/// ```
///

pub fn call_cuckoo_set_parameter(name_bytes: &[u8], value:u32) 
    -> Result<u32, CuckooMinerError>{
    let cuckoo_set_parameter_ref = CUCKOO_SET_PARAMETER.lock().unwrap(); 
    match *cuckoo_set_parameter_ref {
        None => return Err(CuckooMinerError::PluginNotLoadedError(
            String::from("No miner plugin is loaded. Please call init() with the name of a valid mining plugin."))),
        Some(c) => unsafe {
                        return Ok(c(name_bytes.as_ptr(), name_bytes.len() as u32, value));
                   },
        
    };
}


