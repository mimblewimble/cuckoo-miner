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

use std::sync::Mutex;

use libloading;
use libc::*;

use error::CuckooMinerError;

// PRIVATE MEMBERS

// Type definitions corresponding to each function that the plugin implements

type CuckooInit = unsafe extern "C" fn();
type CuckooCall = unsafe extern "C" fn(*const c_uchar, uint32_t, *mut uint32_t) -> uint32_t;
type CuckooDescription = unsafe extern "C" fn(*mut c_uchar,
                                              *mut uint32_t,
                                              *mut c_uchar,
                                              *mut uint32_t);
type CuckooParameterList = unsafe extern "C" fn(*mut c_uchar, *mut uint32_t) -> uint32_t;
type CuckooSetParameter = unsafe extern "C" fn(*const c_uchar, uint32_t, uint32_t) -> uint32_t;
type CuckooGetParameter = unsafe extern "C" fn(*const c_uchar, uint32_t, *mut uint32_t) -> uint32_t;
type CuckooIsQueueUnderLimit = unsafe extern "C" fn() -> uint32_t;
type CuckooPushToInputQueue = unsafe extern "C" fn(*const c_uchar, uint32_t, *const c_uchar)
                                                   -> uint32_t;
type CuckooReadFromOutputQueue = unsafe extern "C" fn(*mut uint32_t, *mut c_uchar) -> uint32_t;
type CuckooStartProcessing = unsafe extern "C" fn() -> uint32_t;
type CuckooStopProcessing = unsafe extern "C" fn() -> uint32_t;
type CuckooHashesSinceLastCall = unsafe extern "C" fn() -> uint32_t;
type CuckooGetStats = unsafe extern "C" fn(*mut c_uchar, *mut uint32_t) -> uint32_t;

/// Struct to hold instances of loaded plugins

pub struct PluginLibrary {
	pub lib_full_path: String,
	loaded_library: Mutex<libloading::Library>,
	cuckoo_init: Mutex<CuckooInit>,
	cuckoo_call: Mutex<CuckooCall>,
	cuckoo_description: Mutex<CuckooDescription>,
	cuckoo_parameter_list: Mutex<CuckooParameterList>,
	cuckoo_get_parameter: Mutex<CuckooGetParameter>,
	cuckoo_set_parameter: Mutex<CuckooSetParameter>,
	cuckoo_is_queue_under_limit: Mutex<CuckooIsQueueUnderLimit>,
	cuckoo_push_to_input_queue: Mutex<CuckooPushToInputQueue>,
	cuckoo_read_from_output_queue: Mutex<CuckooReadFromOutputQueue>,
	cuckoo_start_processing: Mutex<CuckooStartProcessing>,
	cuckoo_stop_processing: Mutex<CuckooStopProcessing>,
	cuckoo_hashes_since_last_call: Mutex<CuckooHashesSinceLastCall>,
	cuckoo_get_stats: Mutex<CuckooGetStats>,
}

impl PluginLibrary {
	pub fn new(lib_full_path: &str) -> Result<PluginLibrary, CuckooMinerError> {
		debug!("Loading miner plugin: {}", &lib_full_path);

		let result = libloading::Library::new(lib_full_path);

		if let Err(e) = result {
			return Err(CuckooMinerError::PluginNotFoundError(
				String::from(format!("{} - {:?}", lib_full_path, e)),
			));
		}

		let loaded_library = result.unwrap();
		PluginLibrary::load_symbols(loaded_library, lib_full_path)
	}

	fn load_symbols(
		loaded_library: libloading::Library,
		path: &str
	) -> Result<PluginLibrary, CuckooMinerError> {
		unsafe {
			let ret_val = PluginLibrary {
				lib_full_path: String::from(path),
				cuckoo_init: {
					let cuckoo_init: libloading::Symbol<CuckooInit> =
						loaded_library.get(b"cuckoo_init\0").unwrap();
					Mutex::new(*cuckoo_init.into_raw())
				},

				cuckoo_call: {
					let cuckoo_call: libloading::Symbol<CuckooCall> =
						loaded_library.get(b"cuckoo_call\0").unwrap();
					Mutex::new(*cuckoo_call.into_raw())
				},

				cuckoo_description: {
					let cuckoo_description:libloading::Symbol<CuckooDescription> =
						loaded_library.get(b"cuckoo_description\0").unwrap();
					Mutex::new(*cuckoo_description.into_raw())
				},

				cuckoo_parameter_list: {
					let cuckoo_parameter_list:libloading::Symbol<CuckooParameterList> =
						loaded_library.get(b"cuckoo_parameter_list\0").unwrap();
					Mutex::new(*cuckoo_parameter_list.into_raw())
				},

				cuckoo_get_parameter: {
					let cuckoo_get_parameter:libloading::Symbol<CuckooGetParameter> =
						loaded_library.get(b"cuckoo_get_parameter\0").unwrap();
					Mutex::new(*cuckoo_get_parameter.into_raw())
				},

				cuckoo_set_parameter: {
					let cuckoo_set_parameter:libloading::Symbol<CuckooSetParameter> =
						loaded_library.get(b"cuckoo_set_parameter\0").unwrap();
					Mutex::new(*cuckoo_set_parameter.into_raw())
				},

				cuckoo_is_queue_under_limit: {
					let cuckoo_is_queue_under_limit:libloading::Symbol<CuckooIsQueueUnderLimit> =
						loaded_library.get(b"cuckoo_is_queue_under_limit\0").unwrap();
					Mutex::new(*cuckoo_is_queue_under_limit.into_raw())
				},

				cuckoo_push_to_input_queue: {
					let cuckoo_push_to_input_queue:libloading::Symbol<CuckooPushToInputQueue> =
						loaded_library.get(b"cuckoo_push_to_input_queue\0").unwrap();
					Mutex::new(*cuckoo_push_to_input_queue.into_raw())
				},

				cuckoo_read_from_output_queue: {
					let cuckoo_read_from_output_queue:libloading::Symbol<CuckooReadFromOutputQueue> =
						loaded_library.get(b"cuckoo_read_from_output_queue\0").unwrap();
					Mutex::new(*cuckoo_read_from_output_queue.into_raw())
				},

				cuckoo_start_processing: {
					let cuckoo_start_processing:libloading::Symbol<CuckooStartProcessing> =
						loaded_library.get(b"cuckoo_start_processing\0").unwrap();
					Mutex::new(*cuckoo_start_processing.into_raw())
				},

				cuckoo_stop_processing: {
					let cuckoo_stop_processing:libloading::Symbol<CuckooStopProcessing> =
						loaded_library.get(b"cuckoo_stop_processing\0").unwrap();
					Mutex::new(*cuckoo_stop_processing.into_raw())
				},

				cuckoo_hashes_since_last_call: {
					let cuckoo_hashes_since_last_call:libloading::Symbol<CuckooHashesSinceLastCall> =
						loaded_library.get(b"cuckoo_hashes_since_last_call\0").unwrap();
					Mutex::new(*cuckoo_hashes_since_last_call.into_raw())
				},

				cuckoo_get_stats: {
					let cuckoo_get_stats: libloading::Symbol<CuckooGetStats> =
						loaded_library.get(b"cuckoo_get_stats\0").unwrap();
					Mutex::new(*cuckoo_get_stats.into_raw())
				},

				loaded_library: Mutex::new(loaded_library),
			};

			ret_val.call_cuckoo_init();
			return Ok(ret_val);
		}
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

	pub fn unload(&self) {
		let cuckoo_get_parameter_ref = self.cuckoo_get_parameter.lock().unwrap();
		drop(cuckoo_get_parameter_ref);

		let cuckoo_set_parameter_ref = self.cuckoo_set_parameter.lock().unwrap();
		drop(cuckoo_set_parameter_ref);

		let cuckoo_parameter_list_ref = self.cuckoo_parameter_list.lock().unwrap();
		drop(cuckoo_parameter_list_ref);

		let cuckoo_call_ref = self.cuckoo_call.lock().unwrap();
		drop(cuckoo_call_ref);

		let cuckoo_description_ref = self.cuckoo_description.lock().unwrap();
		drop(cuckoo_description_ref);

		let cuckoo_is_queue_under_limit_ref = self.cuckoo_is_queue_under_limit.lock().unwrap();
		drop(cuckoo_is_queue_under_limit_ref);

		let cuckoo_push_to_input_queue_ref = self.cuckoo_push_to_input_queue.lock().unwrap();
		drop(cuckoo_push_to_input_queue_ref);

		let cuckoo_read_from_output_queue_ref = self.cuckoo_read_from_output_queue.lock().unwrap();
		drop(cuckoo_read_from_output_queue_ref);

		let cuckoo_start_processing_ref = self.cuckoo_start_processing.lock().unwrap();
		drop(cuckoo_start_processing_ref);

		let cuckoo_stop_processing_ref = self.cuckoo_stop_processing.lock().unwrap();
		drop(cuckoo_stop_processing_ref);

		let cuckoo_hashes_since_last_call_ref = self.cuckoo_hashes_since_last_call.lock().unwrap();
		drop(cuckoo_hashes_since_last_call_ref);

		let cuckoo_get_stats_ref = self.cuckoo_get_stats.lock().unwrap();
		drop(cuckoo_get_stats_ref);

		let loaded_library_ref = self.loaded_library.lock().unwrap();
		drop(loaded_library_ref);
	}

	/// #Description
	///
	/// Initialises the cuckoo plugin, mostly allowing it to write a list of
	/// its accepted parameters. This should be called just after the plugin
	/// is loaded, and before anything else is called.
	///
	/// #Arguments
	///
	/// * None
	///
	/// #Returns
	///
	/// * Nothing
	///
	/// #Example
	///
	/// ```
	///  # use cuckoo_sys::PluginLibrary;
	///  # use std::env;
	///  # use std::path::PathBuf;
	///
	///  # #[cfg(target_os = "linux")]
	///  # static DLL_SUFFIX: &str = ".so";
	///  # #[cfg(target_os = "macos")]
	///  # static DLL_SUFFIX: &str = ".dylib";
	///  # #[cfg(target_os = "windows")]
	///  # static DLL_SUFFIX: &str = ".dll";
	///
	///  # let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	///  # d.push(format!("../target/debug/plugins/libcuckoo_lean_cpu_16{}", DLL_SUFFIX).as_str());
	///
	///  # let plugin_path = d.to_str().unwrap();
	///
	///  let pl=PluginLibrary::new(plugin_path).unwrap();
	///  pl.call_cuckoo_init();
	/// ```
	///

	pub fn call_cuckoo_init(&self) {
		let cuckoo_init_ref = self.cuckoo_init.lock().unwrap();
		unsafe {
			cuckoo_init_ref();
		};
	}

	/// #Description
	///
	/// Call to the cuckoo_call function of the currently loaded plugin, which
	/// will perform
	/// a Cuckoo Cycle on the given seed, returning the first solution (a
	/// length 42 cycle)
	/// that is found. The implementation details are dependent on particular
	/// loaded plugin.
	///
	/// #Arguments
	///
	/// * `header` (IN) A reference to a block of [u8] bytes to use for the
	/// seed to the
	/// internal SIPHASH function which generates edge locations in the
	/// graph. In practice,
	/// this is a SHA3 hash of a Grin blockheader, but from the plugin's
	/// perspective this
	///    can be anything.
	///
	/// * `solutions` (OUT) A caller-allocated array of 42 unsigned bytes. This
	/// currently must
	/// be of size 42, corresponding to a conventional cuckoo-cycle solution
	/// length.
	/// If a solution is found, the solution nonces will be stored in this
	/// array, otherwise,
	///    they will be left untouched.
	///
	/// #Returns
	///
	/// Ok(1) if a solution is found, with the 42 solution nonces contained
	/// within
	/// `sol_nonces`. Returns Ok(0) if no solution is found and `sol_nonces`
	/// remains
	/// untouched. A
	/// [CuckooMinerError](../../error/error/enum.CuckooMinerError.html)
	/// will be returned if there is no plugin loaded, or if there is an error
	/// calling the function.
	///
	/// #Example
	///
	/// ```text
	///     let pl = PluginLibrary::new("/path/to/plugin");
	///     match pl.call_cuckoo(header,
	///                       &mut solution.solution_nonces) {
	///         Ok(result) => {
	///             match result {
	///                 1 => Ok(true),
	///                 0 => Ok(false),
	/// _ =>
	/// Err(CuckooMinerError::UnexpectedResultError(result))
	///             },
	///             Err(e) => Err(CuckooMinerError::PluginNotLoadedError(
	/// String::from("Please call init to load a miner
	/// plug-in"))),
	///      }
	/// ```
	///

	pub fn call_cuckoo(&self, header: &[u8], solutions: &mut [u32; 42]) -> u32 {
		let cuckoo_call_ref = self.cuckoo_call.lock().unwrap();
		unsafe { cuckoo_call_ref(header.as_ptr(), header.len() as u32, solutions.as_mut_ptr()) }
	}

	/// #Description
	/// Call to the call_cuckoo_description function of the loaded
	/// plugin, which will return various information about the plugin, including
	/// its name, description, and other information (to be added as needed).
	///
	/// #Arguments
	///
	/// * `name_bytes` (OUT) A caller-allocated u8 array to which the plugin
	/// will write its
	/// name.
	///
	/// * `name_bytes_len` (IN-OUT) When called, this should contain the
	/// maximum number of bytes
	/// the plugin should write to `name_bytes`. Upon return, this is filled
	/// with the number of bytes that were written to `name_bytes`.
	///
	/// * `description_bytes` (OUT) A caller-allocated u8 array to which the
	/// plugin will write its description.
	///
	/// * `description_bytes_len` (IN-OUT) When called, this should contain the
	/// maximum number of bytes the plugin should write to `description_bytes`. 
	/// Upon return, this is filled with the number of bytes that were written 
	/// to `description_bytes`.
	///
	/// #Returns
	/// Nothing (Values are in OUT parameters). If the provided buffer was too short
	/// the values of name_bytes_len and description_bytes_len will be 0
	///
	/// #Example
	///
	/// ```
	///  # use cuckoo_sys::PluginLibrary;
	///  # use std::env;
	///  # use std::path::PathBuf;
	///
	///  # #[cfg(target_os = "linux")]
	///  # static DLL_SUFFIX: &str = ".so";
	///  # #[cfg(target_os = "macos")]
	///  # static DLL_SUFFIX: &str = ".dylib";
	///  # #[cfg(target_os = "windows")]
	///  # static DLL_SUFFIX: &str = ".dll";
	///
	///  # let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	///  # d.push(format!("../target/debug/plugins/libcuckoo_lean_cpu_16{}", DLL_SUFFIX).as_str());
	///
	///  # let plugin_path = d.to_str().unwrap();
	///
	///  let pl=PluginLibrary::new(plugin_path).unwrap();
	///  pl.call_cuckoo_init();
	///
	///  let mut name_bytes:[u8;256]=[0;256];
	///  let mut description_bytes:[u8;256]=[0;256];
	///  let mut name_bytes_len=name_bytes.len() as u32;
	///  let mut desc_len=description_bytes.len() as u32;
	///  pl.call_cuckoo_description(&mut name_bytes, &mut name_bytes_len,
	///                          &mut description_bytes, &mut desc_len);
	/// ```
	///

	pub fn call_cuckoo_description(
		&self,
		name_bytes: &mut [u8],
		name_bytes_len: &mut u32,
		description_bytes: &mut [u8],
		description_bytes_len: &mut u32,
	) {
		let cuckoo_description_ref = self.cuckoo_description.lock().unwrap();
		unsafe {
			cuckoo_description_ref(
				name_bytes.as_mut_ptr(),
				name_bytes_len,
				description_bytes.as_mut_ptr(),
				description_bytes_len,
			);
		}
	}

	/// #Description
	///
	/// Call to the cuckoo_call_parameter_list function of the currently loaded
	/// plugin, which will provide an informative JSON array of the parameters that the
	/// plugin supports, as well as their descriptions and range of values.
	///
	/// #Arguments
	///
	/// * `param_list_bytes` (OUT) A reference to a block of [u8] bytes to fill
	/// with the JSON result array
	///
	/// * `param_list_len` (IN-OUT) When called, this should contain the
	/// maximum number of bytes the plugin should write to `param_list_bytes`.
	/// Upon return, this is filled with the number of bytes that were written to 
	/// `param_list_bytes`.
	///
	/// #Returns
	///
	/// 0 if okay, with the result is stored in `param_list_bytes`
	/// 3 if the provided array is too short
	///
	/// #Example
	///
	/// ```
	///  # use cuckoo_sys::PluginLibrary;
	///  # use std::env;
	///  # use std::path::PathBuf;
	///
	///  # #[cfg(target_os = "linux")]
	///  # static DLL_SUFFIX: &str = ".so";
	///  # #[cfg(target_os = "macos")]
	///  # static DLL_SUFFIX: &str = ".dylib";
	///  # #[cfg(target_os = "windows")]
	///  # static DLL_SUFFIX: &str = ".dll";
	///
	///  # let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	///  # d.push(format!("../target/debug/plugins/libcuckoo_lean_cpu_16{}", DLL_SUFFIX).as_str());
	///
	///  # let plugin_path = d.to_str().unwrap();
	///
	///  let pl=PluginLibrary::new(plugin_path).unwrap();
	///  pl.call_cuckoo_init();
	///  let mut param_list_bytes:[u8;1024]=[0;1024];
	///  let mut param_list_len=param_list_bytes.len() as u32;
	///  //get a list of json parameters
	///  let parameter_list=pl.call_cuckoo_parameter_list(&mut param_list_bytes,
	///    &mut param_list_len);
	/// ```
	///

	pub fn call_cuckoo_parameter_list(
		&self,
		param_list_bytes: &mut [u8],
		param_list_len: &mut u32,
	) -> u32 {
		let cuckoo_parameter_list_ref = self.cuckoo_parameter_list.lock().unwrap();
		unsafe { cuckoo_parameter_list_ref(param_list_bytes.as_mut_ptr(), param_list_len) }
	}

	/// #Description
	///
	/// Retrieves the value of a parameter from the currently loaded plugin
	///
	/// #Arguments
	///
	/// * `name_bytes` (IN) A reference to a block of [u8] bytes storing the
	/// parameter name
	///
	/// * `value` (OUT) A reference where the parameter value will be stored
	///
	/// #Returns
	///
	/// 0 if the parameter was retrived, and the result is stored in `value`
	/// 1 if the parameter does not exist
	/// 4 if the provided parameter name was too long
	///
	/// #Example
	/// ```
	///  # use cuckoo_sys::PluginLibrary;
	///  # use std::env;
	///  # use std::path::PathBuf;
	///
	///  # #[cfg(target_os = "linux")]
	///  # static DLL_SUFFIX: &str = ".so";
	///  # #[cfg(target_os = "macos")]
	///  # static DLL_SUFFIX: &str = ".dylib";
	///  # #[cfg(target_os = "windows")]
	///  # static DLL_SUFFIX: &str = ".dll";
	///
	///  # let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	///  # d.push(format!("../target/debug/plugins/libcuckoo_lean_cpu_16{}", DLL_SUFFIX).as_str());
	///
	///  # let plugin_path = d.to_str().unwrap();
	///
	///  let pl=PluginLibrary::new(plugin_path).unwrap();
	///  pl.call_cuckoo_init();
	///  let name = "NUM_THREADS";
	///  let mut num_threads:u32 = 0;
	///  let ret_val = pl.call_cuckoo_get_parameter(name.as_bytes(), &mut num_threads);
	/// ```
	///

	pub fn call_cuckoo_get_parameter(&self, name_bytes: &[u8], value: &mut u32) -> u32 {
		let cuckoo_get_parameter_ref = self.cuckoo_get_parameter.lock().unwrap();
		unsafe { cuckoo_get_parameter_ref(name_bytes.as_ptr(), name_bytes.len() as u32, value) }
	}

	/// Sets the value of a parameter in the currently loaded plugin
	///
	/// #Arguments
	///
	/// * `name_bytes` (IN) A reference to a block of [u8] bytes storing the
	/// parameter name
	///
	/// * `value` (IN) The value to which to set the parameter
	///
	/// #Returns
	///
	/// 0 if the parameter was retrived, and the result is stored in `value`
	/// 1 if the parameter does not exist
	/// 2 if the parameter exists, but is outside the allowed range set by the
	/// plugin
	///
	/// #Example
	///
	/// ```text
	///   let pl = PluginLibrary::new("/path/to/plugin");
	///   let name = "NUM_THREADS";
	///   let return_code = pl.call_cuckoo_set_parameter(name.as_bytes(), 8)?;
	/// ```
	///

	pub fn call_cuckoo_set_parameter(&self, name_bytes: &[u8], value: u32) -> u32 {
		let cuckoo_set_parameter_ref = self.cuckoo_set_parameter.lock().unwrap();
		unsafe { cuckoo_set_parameter_ref(name_bytes.as_ptr(), name_bytes.len() as u32, value) }
	}

	/// #Description
	///
	/// For Async/Queued mode, check whether the plugin is ready
	/// to accept more hashes.
	///
	/// #Arguments
	///
	/// * None
	///
	/// #Returns
	///
	/// * 1 if the queue can accept more hashes, 0 otherwise
	///

	pub fn call_cuckoo_is_queue_under_limit(&self) -> u32 {
		let cuckoo_is_queue_under_limit_ref = self.cuckoo_is_queue_under_limit.lock().unwrap();
		unsafe { cuckoo_is_queue_under_limit_ref() }
	}

	/// #Description
	///
	/// Pushes a hash to the loaded plugin for later processing in
	/// asyncronous/queued mode.
	///
	/// #Arguments
	///
	/// * `hash` (IN) A block of bytes to use for the seed to the internal
	/// SIPHASH function
	/// which generates edge locations in the graph. In practice, this is a
	/// BLAKE2 hash
	/// of a Grin blockheader, but from the plugin's perspective this can be
	/// anything.
	///
	/// * `nonce` (IN) The nonce that was used to generate this hash, for
	/// identification
	///   purposes in the solution queue
	///
	/// #Returns
	///
	/// Ok(1) if the hash was added to the queue, Ok(0) otherwise (if shutting
	/// down or queue
	/// is full.
	/// Otherwise, a
	/// [CuckooMinerError](../../error/error/enum.CuckooMinerError.html)
	/// with specific detail is returned if an error is encountered.
	///
	///
	/// #Example
	///
	/// ```text
	///  let pl = PluginLibrary::new("/path/to/plugin");
	///  let (nonce, hash) = self.get_next_hash(&pre_nonce, &post_nonce);
	///  let nonce_bytes:[u8;8] = unsafe{transmute(nonce.to_be())};
	///  pl.call_cuckoo_push_to_input_queue(&hash, &nonce_bytes)?;
	/// ```
	///

	pub fn call_cuckoo_push_to_input_queue(&self, hash: &[u8], nonce: &[u8]) -> u32 {
		let cuckoo_push_to_input_queue_ref = self.cuckoo_push_to_input_queue.lock().unwrap();
		unsafe { cuckoo_push_to_input_queue_ref(hash.as_ptr(), hash.len() as u32, nonce.as_ptr()) }
	}

	/// #Description
	///
	/// Reads the next solution from the output queue, if one exists. Only
	/// solutions which meet
	/// the target difficulty specified in the preceeding call to 'notify' will
	/// be placed in the
	///  output queue. Read solutions are popped from the queue.
	/// Does not block, and intended to be called continually as part of a
	/// mining loop.
	///
	/// #Arguments
	///
	/// * `sol_nonces` (OUT) A block of 42 u32s in which the solution nonces
	/// will be stored,
	///    if any exist.
	///
	/// * `nonce` (OUT) A block of 8 u8s representing a Big-Endian u64, used
	/// for identification
	/// purposes so the caller can reconstruct the header used to generate
	/// the solution
	///
	///
	/// #Returns
	///
	/// Ok(1) if a solution was popped from the queue, Ok(0) if not solution is
	/// available.
	/// Otherwise, a
	/// [CuckooMinerError](../../error/error/enum.CuckooMinerError.html)
	/// with specific detail is returned if an error is encountered.
	///
	/// #Example
	///
	///
	/// ```text
	///     let pl = PluginLibrary::new("/path/to/plugin");
	///     //.
	///     //.
	///     //.
	///     let mut sol_nonces=[u32;42];
	///     let mut nonce=[u8;8];  //Initialise this with a u64
	///     while pl.call_cuckoo_read_from_output_queue(&mut sol_nonces, &mut nonce).unwrap()!=0 {
	///        ...
	///     }
	/// ```
	///

	pub fn call_cuckoo_read_from_output_queue(
		&self,
		solutions: &mut [u32; 42],
		nonce: &mut [u8; 8],
	) -> u32 {
		let cuckoo_read_from_output_queue_ref = self.cuckoo_read_from_output_queue.lock().unwrap();
		unsafe { cuckoo_read_from_output_queue_ref(solutions.as_mut_ptr(), nonce.as_mut_ptr()) }
	}

	/// #Description
	///
	/// Starts asyncronous processing. The plugin will start reading hashes
	/// from the input queue, delegate them internally as it sees fit, and
	/// put solutions into the output queue. It is up to the plugin
	/// implementation
	/// to manage how the workload is spread across devices/threads.
	///
	/// #Arguments
	///
	/// * None
	///
	/// #Returns
	///
	/// * Ok(1) if processing was successfully started, 0 otherwise (TBD return
	/// codes)
	/// with a return code from the plugin.
	/// Otherwise, a
	/// [CuckooMinerError](../../error/error/enum.CuckooMinerError.html)
	/// with specific detail is returned if an error is encountered.
	///
	/// #Corresponding C (Unix)
	///
	/// ```text
	///  extern "C" int cuckoo_start_processing();
	/// ```

	pub fn call_cuckoo_start_processing(&self) -> u32 {
		let cuckoo_start_processing_ref = self.cuckoo_start_processing.lock().unwrap();
		unsafe { cuckoo_start_processing_ref() }
	}

	/// #Description
	///
	/// Stops asyncronous processing. The plugin should signal to shut down
	/// processing,
	/// as quickly as possible, and clean up all threads/devices/memory it may
	/// have
	/// allocated. This function should not block
	///
	/// #Arguments
	///
	/// * None
	///
	/// #Returns
	///
	/// * Ok(1) if processing was successfully stopped, 0 otherwise (TBD return
	/// codes)
	/// with a return code from the plugin.
	/// Otherwise, a
	/// [CuckooMinerError](../../error/error/enum.CuckooMinerError.html)
	/// with specific detail is returned if an error is encountered.
	///
	/// #Corresponding C (Unix)
	///
	/// ```text
	///  extern "C" int cuckoo_stop_processing();
	/// ```

	pub fn call_cuckoo_stop_processing(&self) -> u32 {
		let cuckoo_stop_processing_ref = self.cuckoo_stop_processing.lock().unwrap();
		unsafe { cuckoo_stop_processing_ref() }
	}

	/// #Description
	///
	/// A simple metric function that returns the number of hashes the plugin
	/// has processed since this function was last called. It is up to the
	/// plugin implementation to keep track of this count.
	///
	/// #Arguments
	///
	/// * None
	///
	/// #Returns
	///
	/// * Ok(h) with the number of hashes processed since this function was
	/// last called.
	/// Otherwise, a
	/// [CuckooMinerError](../../error/error/enum.CuckooMinerError.html)
	/// with specific detail is returned if an error is encountered.
	///
	/// #Corresponding C (Unix)
	/// ```text
	///  extern "C" int cuckoo_stop_processing();
	/// ```

	pub fn call_cuckoo_hashes_since_last_call(&self) -> u32 {
		let cuckoo_hashes_since_last_call_ref = self.cuckoo_hashes_since_last_call.lock().unwrap();
		unsafe { cuckoo_hashes_since_last_call_ref() }
	}

	/// #Description
	///
	/// Retrieves a JSON list of the plugin's current stats for all running
	/// devices. e.g:
	///
	/// ```text
	///   [{
	///      device_id:"0",
	///      device_name:"NVIDIA GTX 1080",
	///      last_start_time: "23928329382",
	///      last_end_time: "23928359382",
	///      last_solution_time: "3382",
	///    },
	///    {
	///      device_id:"1",
	///      device_name:"NVIDIA GTX 1080ti",
	///      last_start_time: "23928329382",
	///      last_end_time: "23928359382",
	///      last_solution_time: "3382",
	///    }]
	/// ```
	/// #Arguments
	///
	/// * `stat_bytes` (OUT) A reference to a block of [u8] bytes to fill with
	/// the JSON
	///    result array
	///
	/// * `stat_bytes_len` (IN-OUT) When called, this should contain the
	/// maximum number of bytes
	/// the plugin should write to `stat_bytes`. Upon return, this is filled
	/// with the number
	///    of bytes that were written to `stat_bytes`.
	///
	/// #Returns
	///
	/// 0 if the parameter list was retrived, and the result is stored in
	/// `stat_bytes`
	/// 3 if the buffer and size given was too small to store the stats
	///
	/// #Example
	///
	/// ```text
	///   let mut stat_bytes:[u8;1024]=[0;1024];
	///   let mut stat_bytes_len=stat_bytes.len() as u32;
	///   //get a list of parameters
	///   let stat_list=call_cuckoo_get_stats(&mut stat_bytes, &mut
	///   stat_bytes_len);
	/// ```
	///
	///

	pub fn call_cuckoo_get_stats(&self, stat_bytes: &mut [u8], stat_bytes_len: &mut u32) -> u32 {
		let cuckoo_get_stats_ref = self.cuckoo_get_stats.lock().unwrap();
		unsafe { cuckoo_get_stats_ref(stat_bytes.as_mut_ptr(), stat_bytes_len) }
	}
}
