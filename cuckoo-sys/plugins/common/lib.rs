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


//! The Rust interface side of the Cuckoo-Miner plugin. Any DLL wishing to implement
//! a cuckoo-miner needs to implement this interface and expose these functions
//! (ensuring they're not name-mangled).
//!
//! Descriptions of what the corresponding C function should look like follow
//! in each function.
//!

#![allow(dead_code, non_camel_case_types, non_upper_case_globals, non_snake_case)]

extern crate libc;
use libc::*;

extern "C" {

    /// #Description 
    ///
    /// Call to the cuckoo_basic_mine function in the library, which calls a base
    /// cuckoo miner implementation implemented within a plugin.
    ///
    /// #Arguments
    ///
    /// * `header` (IN) A block of bytes to use for the seed to the internal SIPHASH function
    ///    which generates edge locations in the graph. In practice, this is a SHA3 hash
    ///    of a Grin blockheader, but from the plugin's perspective this can be anything.
    ///
    /// * `header_len` (IN) The length of the header, in bytes.
    ///
    /// * `nthreads` (IN) If the miner implements multithreading, the number of threads to use
    ///
    /// * `ntrims` (IN) If the miner implements edge-trimming, the number of rounds to use. If
    ///    this is 0, the plugin itself will decide.
    ///
    /// * `sol_nonces` (OUT) A caller-allocated array of 42 unsigned bytes. This currently must
    ///    be of size 42, corresponding to a conventional cuckoo-cycle solution length. 
    ///    If a solution is found, the solution nonces will be stored in this array, otherwise,
    ///    they will be left untouched.
    ///
    /// #Returns
    ///
    /// 1 if a solution is found, with the 42 solution nonces contained within
    /// `sol_nonces`. Returns 0 if no solution is found and `sol_nonces` remains
    /// untouched.
    ///
    /// #Corresponding C (Unix)
    /// 
    /// ```
    ///  extern "C" int cuckoo_call(char* header_data, 
    ///                            int header_length,
    ///                            int nthreads,
    ///                            int ntrims, 
    ///                            uint_32* sol_nonces);
    /// ```
    /// All memory should be allocated on the caller side. The implementing plugin 
    /// should not allocate any memory to be returned to the caller.
    ///
    /// #Example
    ///
    /// This example assumes that `cuckoo_call` below is a mutex containing a loaded
    /// library symbol corresponding to this call.
    /// 
    /// ```
    ///  pub fn call_cuckoo(header: &[u8], num_threads: u32, num_trims:u32, solutions:&mut [u32; 42] ) -> Result<u32, CuckooMinerError> {
    ///      let cuckoo_call_ref = cuckoo_call.lock().unwrap(); 
    ///      match *cuckoo_call_ref {
    ///          None => return Err(CuckooMinerError::PluginNotLoadedError(
    ///               String::from("No miner plugin is loaded. Please call init() with the name of a valid mining plugin."))),
    ///          Some(c) => unsafe {
    ///               return Ok(c(header.as_ptr(), header.len(), num_threads, 
    ///                   num_trims, solutions.as_mut_ptr()));
    ///          },
    ///      };
    ///  }
    /// ```
    ///

    pub fn cuckoo_call(header: *const c_uchar, 
                       header_len: size_t,
                       num_threads: u32,
                       num_trims: u32,
                       sol_nonces: *mut uint32_t) -> uint32_t;

    /// #Description 
    ///
    /// Queries a plugin to determine its description, name, and capabilities. This should
    /// be extended in future to test whether a particular plugin can be run on the host
    /// platform, in the case of different plugin variants such as different instruction sets.
    ///
    /// #Arguments
    ///
    /// * `name_buf` (OUT) A buffer in which to store the plugin's returned name. Note the plugin
    ///    should take care to ensure not to write more than the provided 'name_buf_len' value
    ///    to this buffer.
    ///
    /// * `name_buf_len` (IN/OUT) On entry, the maximum length, in bytes, that the plugin may write to 
    ///    `name_buf`. When the function returns, this contains the length that was written to
    ///    `name_buf`
    ///
    /// * `description_buf` (OUT) A buffer in which to store the plugin's returned description. Note the plugin
    ///    should take care to ensure not to write more than the provided 'description_buf_len' value
    ///    to this buffer.
    ///
    /// * `description_buf_len` (IN/OUT) On entry, the maximum length, in bytes, that the plugin may write to 
    ///    `description_buf`. When the function returns, this contains the length that was written to
    ///    `description_buf`
    ///
    /// #Returns
    ///
    /// The function has no return value, but uses the OUT values described above.
    ///
    /// #Corresponding C (Unix)
    /// 
    /// ```
    ///  extern "C" void cuckoo_description(char * name_buf,
    ///                                     int* name_buf_len,
    ///                                     char *description_buf,
    ///                                     int* description_buf_len)
    /// ```
    /// All memory should be allocated on the caller side. The implementing plugin 
    /// should not allocate any memory to be returned to the caller.
    ///
    /// #Example
    ///
    /// This example assumes that `cuckoo_description` below is a mutex containing a loaded
    /// library symbol corresponding to this call.
    /// 
    /// ```
    ///  fn call_cuckoo_description(name_bytes: &mut [u8;256], name_bytes_len:&mut u32,
    ///                             description_bytes: &mut [u8;256], description_bytes_len:&mut u32) 
    ///      -> Result<(), CuckooMinerError>{
    ///      let cuckoo_description_ref = cuckoo_description.lock().unwrap(); 
    ///      match *cuckoo_description_ref {
    ///          None => return Err(CuckooMinerError::PluginNotLoadedError(
    ///              String::from("No miner plugin is loaded. Please call init() with the name of a valid mining plugin."))),
    ///          Some(c) => unsafe {
    ///                          c(name_bytes.as_mut_ptr(), name_bytes_len, 
    ///                            description_bytes.as_mut_ptr(), description_bytes_len);
    ///                          return Ok(());
    ///                     },
    ///          
    ///      };
    ///  }
    /// ```
    ///

    pub fn cuckoo_description(name_buf: *mut c_uchar,
                              name_buf_len: *mut uint32_t,
                              description_buf: *mut c_uchar,
                              description_buf_len: *mut uint32_t);
}
