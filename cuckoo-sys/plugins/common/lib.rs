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


//! A plugin implementing the Cuckoo-Miner plugin interface. Any DLL wishing to implement
//! a cuckoo-miner needs to implement this interface and expose these functions
//! (ensuring they're not name-mangled). The details of this specific plugin
//! can be discovered at runtime by calling its cuckoo_description function.
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
    /// Call to the call_cuckoo function in the library, which calls a base
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
    ///  pub fn call_cuckoo(header: &[u8], solutions:&mut [u32; 42] ) -> Result<u32, CuckooMinerError> {
    ///      let cuckoo_call_ref = cuckoo_call.lock().unwrap(); 
    ///      match *cuckoo_call_ref {
    ///          None => return Err(CuckooMinerError::PluginNotLoadedError(
    ///               String::from("No miner plugin is loaded. Please call init() with the name of a valid mining plugin."))),
    ///          Some(c) => unsafe {
    ///               return Ok(c(header.as_ptr(), header.len(), solutions.as_mut_ptr()));
    ///          },
    ///      };
    ///  }
    /// ```
    ///

    pub fn cuckoo_call(header: *const c_uchar, 
                       header_len: size_t,
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
    /// #Corresponding C (Unix)
    /// 
    /// ```
    ///  extern "C" void cuckoo_init();
    /// ```

    pub fn cuckoo_init();
  

    /// #Description 
    ///
    /// Sets a parameter on the loaded mining plugin. The list of parameters that a plugin accepts
    /// can be retrieved from the cuckoo_get_parameters function. All arguments and values
    /// are represented as uchar strings.
    ///
    /// #Arguments
    ///
    /// * `name` (IN) The name of the parameter to set.
    ///
    /// * `name_len` (IN) The length, in bytes, of the `name` argument.
    ///
    /// * `value` (IN) The value of the parameter to set, represented as a simple ASCII string.
    ///
    /// * `value_len` (IN) The length, in bytes, of the `value` argument.
    ///
    /// #Returns
    ///
    /// 0 if the parameter was retrived, and the result is stored in `value`
    /// 1 if the parameter does not exist
    /// 2 if the parameter exists, but is outside the allowed range set by the plugin
    ///
    /// #Corresponding C (Unix)
    /// 
    /// ```
    ///  extern "C" int cuckoo_set_parameter(char *name,
    ///                                      int name_len,
    ///                                      int value);
    /// ```
    ///
    /// #Example
    ///
    /// 
    /// ```
    /// ```
    ///

    pub fn cuckoo_set_parameter(name: *const c_uchar, 
                                name_len: uint32_t,
                                value: uint32_t) -> uint32_t;

    /// #Description 
    ///
    /// Retrieves a parameter from the loaded cuckoo plugin.
    ///
    /// #Arguments
    ///
    /// * `name` (IN) The name of the parameter to set.
    ///
    /// * `name_len` (IN) The length, in bytes, of the `name` argument.
    ///
    /// * `value` (out) The value of the parameter, represented by a simple ascii string
    ///
    /// * `value_len` (IN-OUT) Coming in, the maximum number of bytes to write to `value`,
    /// coming out, the number of bytes written to `value`
    ///
    /// #Returns
    ///
    /// 0 if the paramter was set correctly, a non-zero return code (TBD) if the parameter
    /// could not be found, for any reason.
    ///
    /// #Corresponding C (Unix)
    /// 
    /// ```
    ///  extern "C" int cuckoo_get_parameter(char *name,
    ///                                      int name_len,
    ///                                      int *value);
    ///                                    
    /// ```
    ///
    /// #Example
    ///
    /// 
    /// ```
    /// ```
    ///

    pub fn cuckoo_get_parameter(name: *const c_uchar, 
                                name_len: uint32_t,
                                value: *mut uint32_t) -> uint32_t;

    /// #Description 
    ///
    /// Retrieves a JSON list of the plugin's available parameters, their
    /// description and their defaults. e.g:
    /// ```
    ///   [{
    ///      name:"num_threads",
    ///      type:"int",
    ///      description: "Number of worker threads",
    ///      default_value: "1",
    ///      min_value: "1",
    ///      max_value: "32"
    ///    },
    ///    {
    ///      name:"num_trims",
    ///      type:"int",
    ///      description: "Maximum number of trimming rounds",
    ///      default_value: "7",
    ///      min_value: "0",
    ///      max_value: "50"
    ///    }]
    ///
    /// ```
    /// This should correspond to an easily deserialised structure on the 
    /// rust side.
    ///
    /// #Arguments
    ///
    /// * `params_out_buf` (OUT) The name of the parameter to set.
    ///
    /// * `params_len` (IN-OUT) Coming in, the maximum number of bytes to write to `params_out_buf`,
    /// coming out, the number of bytes written to `params_out_buf`
    ///
    /// The implementing function should take care not to write more than `params_len` to
    /// the `params_out` buffer, and should not allocate any memory
    ///
    /// #Returns
    ///
    /// 0 if the the parameter list was successfully retrieved, 
    /// 3 if there was not enough space in the buffer to write the list
    ///
    /// #Corresponding C (Unix)
    /// 
    /// ```
    ///  extern "C" int cuckoo_parameter_list(char *params_out_buf,
    ///                                       int* params_len);
    /// ```
    ///
    /// #Example
    ///
    /// 
    /// ```
    /// ```
    ///

    pub fn cuckoo_parameter_list(params_out_buf: *mut c_uchar, 
                                params_len: *mut size_t) -> uint32_t;



    pub fn cuckoo_push_to_input_queue(hash: *const c_uchar, 
                       hash_len: size_t,
                       nonce: *const c_uchar) -> uint32_t;

    pub fn cuckoo_start_processing() -> uint32_t;
    
    pub fn cuckoo_stop_processing() -> uint32_t;

    pub fn cuckoo_is_queue_under_limit() -> uint32_t;

    // Returns whether the plugin is ready for another job (there's space in the queue)
    pub fn cuckoo_read_from_output_queue(sol_nonces: *mut uint32_t, 
                                         nonce: *mut c_uchar) -> uint32_t; 
    
    //Simple metric to count the number of hashes run since last time this was called
    pub fn cuckoo_hashes_since_last_call() -> uint32_t;
    
}
