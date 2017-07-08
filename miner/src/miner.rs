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

//! Main interface for callers into cuckoo-miner. Provides functionality
//! to load a mining plugin, send it a Cuckoo Cycle POW problem, and 
//! return any result solutions that may be found.
//!
//! #Example
//! ```
//!  let mut config = CuckooMinerConfig::new();
//!  config.plugin_full_path = caps[0].full_path.clone();
//!    
//!  //set the number of threads for the miner to use
//!  config.num_threads=2;
//!
//!  //set the number of trimes, 0 lets the plugin decide
//!  config.num_trims=0;
//!
//!  //Build a new miner with this info, which will load
//!  //the associated plugin and 
//!    
//!  let mut miner = CuckooMiner::new(config).expect("");
//!
//!  //Keep a structure to hold the solution.. this will be
//!  //filled out by the plugin
//!  let mut solution = CuckooMinerSolution::new();
//!        
//!  //Mine with given header and check for result
//!  let result = miner.mine(&test_header, &mut solution).unwrap();
//!
//!  if result == true {
//!      println!("Solution found: {}", solution);
//!  } else {
//!      println!("No Solution found");
//!  }
/// ```

use std::{fmt,cmp};

use cuckoo_sys::{call_cuckoo, 
                 load_cuckoo_lib};

use error::CuckooMinerError;

// Hardcoed assumption for now that the solution size will be 42 will be
// maintained, to avoid having to allocate memory within the called C functions

const CUCKOO_SOLUTION_SIZE:usize = 42;

/// A simple struct to hold a cuckoo miner solution. Currently,
/// it's assumed that a solution will be 42 bytes. The `solution_nonces`
/// member is statically allocated here, and will be filled in 
/// by a plugin upon finding a solution.
///

pub struct CuckooMinerSolution {
    /// An array allocated in rust that will be filled
    /// by the called plugin upon successfully finding
    /// a solution

    pub solution_nonces:[u32; CUCKOO_SOLUTION_SIZE],
}

impl Default for CuckooMinerSolution {
	fn default() -> CuckooMinerSolution {
        CuckooMinerSolution {
		    solution_nonces: [0; CUCKOO_SOLUTION_SIZE],
        }
	}
}

impl CuckooMinerSolution{

    /// Creates a new cuckoo miner solution
    /// with nonces set to a u32 array of size
    /// 42 filled with zeroes.

    pub fn new()->CuckooMinerSolution{
        CuckooMinerSolution::default()
    }
}

impl fmt::Display for CuckooMinerSolution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{:?}", &self.solution_nonces[..])
    }
}

impl fmt::Debug for CuckooMinerSolution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{:?}", &self.solution_nonces[..])
    }
}

impl cmp::PartialEq for CuckooMinerSolution {
    fn eq(&self, other: &CuckooMinerSolution) -> bool {
        for i in 0..CUCKOO_SOLUTION_SIZE {
            if self.solution_nonces[i]!=other.solution_nonces[i]{
                return false;
            }
        }
        return true;
    }
}

/// Structure containing the configuration values to pass into an
/// instance of a miner

pub struct CuckooMinerConfig {

    /// The full path to the plugin to load and use to find a solution
    /// to a POW problem. Defaults to empty string, so must be filled
    /// before use.
    pub plugin_full_path: String,

    /// If the plugin supports threads, the number of threads to use
    /// Defaults to 1.
    pub num_threads: u32,

    /// If the plugin supports edge trimming, the number of trim
    /// rounds to perform. If this is set to zero, the plugin
    /// will decide. Defaults to 0
    pub num_trims: u32,


}

impl Default for CuckooMinerConfig {
	fn default() -> CuckooMinerConfig {
		CuckooMinerConfig{
            plugin_full_path: String::from(""),
            num_threads: 1,
            //0 = let the plugin decide
            num_trims: 0,
		}
	}
}

impl CuckooMinerConfig{

    /// Returns a new instance of CuckooMinerConfig

    pub fn new()->CuckooMinerConfig{
        CuckooMinerConfig::default()
    }
}

/// An instance of a miner, which loads a cuckoo-miner plugin
/// and calls it's mine function according to the provided configuration
///

pub struct CuckooMiner{
    /// The internal Configuration object
    pub config: CuckooMinerConfig,
}

impl Default for CuckooMiner {
	fn default() -> CuckooMiner {
		CuckooMiner {
            config: CuckooMinerConfig::default(),
		}
	}
}

impl CuckooMiner {


    /// #Description 
    ///
    /// Creates a new instance of a CuckooMiner with the given configuration.
    /// #Arguments
    ///
    /// `config` an instance of [CuckooMinerConfig](struct.CuckooMinerConfig.html), that
    /// must be filled with the full path name of a valid mining plugin.
    ///
    /// #Returns
    ///
    /// If successful, Ok() is returned and the specified plugin has been loaded internally.
    /// Otherwise a [CuckooMinerError](../../error/error/enum.CuckooMinerError.html) 
    /// with specific detail is returned.
    ///

    pub fn new(config:CuckooMinerConfig)->Result<CuckooMiner, CuckooMinerError>{
        let mut return_val=CuckooMiner{
            config: config,
        };
        return_val.init()?;
        Ok(return_val)
    }

    /// Internal function to perform tha actual library loading

    fn init(&mut self) -> Result<(), CuckooMinerError> {
        load_cuckoo_lib(&self.config.plugin_full_path)
    }

/// #Description 
///
/// Call to the cuckoo_call function of the currently loaded plugin, which will perform 
/// a Cuckoo Cycle on the given seed, filling the first solution (a length 42 cycle)
/// that is found in the provided [CuckooMinerSolution](struct.CuckooMinerSolution.html) structure.
/// The implementation details are dependent on particular loaded plugin. Values provided
/// to the loaded plugin are contained in the internal [CuckooMinerConfig](struct.CuckooMinerConfig.html) 
///
/// #Arguments
///
/// * `header` (IN) A reference to a block of [u8] bytes to use for the seed to the 
///    internal SIPHASH function which generates edge locations in the graph. In practice, 
///    this is a SHA3 hash of a Grin blockheader, but from the plugin's perspective this 
///    can be anything.
///
/// * `solution` (OUT) An empty [CuckooMinerSolution](struct.CuckooMinerSolution.html). 
///    If a solution is found, this structure will contain a list of solution nonces,
///    otherwise, it will remain untouched.
///
/// #Returns
///
/// * Ok(true) if a solution is found, with the 42 solution nonces contained within
/// the provided [CuckooMinerSolution](struct.CuckooMinerSolution.html).
/// * Ok(false) if no solution is found and `solution` remains untouched.
/// * A [CuckooMinerError](../../error/error/enum.CuckooMinerError.html) 
/// if there is no plugin loaded, or if there is an error calling the function.
///

    pub fn mine(&self, header: &[u8], solution:&mut CuckooMinerSolution) 
        -> Result<bool, CuckooMinerError> {    
            match call_cuckoo(header, 
                              self.config.num_threads,
                              self.config.num_trims,
                              &mut solution.solution_nonces) {
                Ok(result) => {
                    match result {
                        1 => {
                            debug!("Solution found."); 
                            Ok(true)
                        }
                        0 => Ok(false),
                        _ => Err(CuckooMinerError::UnexpectedResultError(result))
                    }
                },
                Err(_) => Err(CuckooMinerError::PluginNotLoadedError(
                    String::from("Please call init to load a miner plug-in"))),
            }
    }
}