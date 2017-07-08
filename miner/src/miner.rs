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

//! Main interface for callers into cuckoo-miner

use std::{fmt,cmp};

use cuckoo_sys::{call_cuckoo, 
                 load_cuckoo_lib};

use error::CuckooMinerError;

// Hardcoed assumption for now that the solution size will be 42 will be
// maintained, to avoid having to allocate memory within the called C functions

const CUCKOO_SOLUTION_SIZE:usize = 42;

/// A struct to hold a cuckoo solution
pub struct CuckooMinerSolution {
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
    pub fn new()->CuckooMinerSolution{
        CuckooMinerSolution::default()
    }

    pub fn set_nonces(&mut self, nonce_in:[u32; CUCKOO_SOLUTION_SIZE]){
        self.solution_nonces = nonce_in;
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

/// Structure containing the configuration values for an instnce
/// of a miner.

pub struct CuckooMinerConfig {

    // The full path to the plugin to use
    pub plugin_full_path: String,

    // Number of threads to use
    pub num_threads: u32,

    // Number of trims
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
    pub fn new()->CuckooMinerConfig{
        CuckooMinerConfig::default()
    }
}

pub struct CuckooMiner{
    // Configuration
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

    pub fn new(config:CuckooMinerConfig)->Result<CuckooMiner, CuckooMinerError>{
        let mut return_val=CuckooMiner{
            config: config,
        };
        return_val.init()?;
        Ok(return_val)
    }

    fn init(&mut self) -> Result<(), CuckooMinerError> {
        load_cuckoo_lib(&self.config.plugin_full_path)
    }

    /// #Description 
    ///
    /// Call a specified mining function in the cuckoo_mining library, and store the
    /// result if a solution exists. The parameters to this miner are contained within
    /// the associated CuckooMinerConfig structure.
    ///
    /// #Arguments
    ///
    /// * `header` The SHA3 hash to use for the seed to the internal SIPHASH function
    ///    which generates edge locations in the graph
    /// * `solution` A new CuckooMinerSolution struct into which the result will be placed
    ///
    /// Returns Ok(true) if a solution is found, with the 42 solution nonces contained within
    /// sol_nonces. Returns Ok(false) if no solution is found. Returns Err(CuckooMinerError)
    /// upon error
    ///
    /// #Example
    /// TBD
    ///

    pub fn mine(&self, header: &[u8], solution:&mut CuckooMinerSolution) 
        -> Result<bool, CuckooMinerError> {    
            match call_cuckoo(header, 
                              self.config.num_threads,
                              self.config.num_trims,
                              &mut solution.solution_nonces) {
                Ok(result) => {
                    match result {
                        1 => Ok(true),
                        0 => Ok(false),
                        _ => Err(CuckooMinerError::UnexpectedResultError(result))
                    }
                },
                Err(e) => Err(CuckooMinerError::PluginNotLoadedError(
                    String::from("Please call init to load a miner plug-in"))),
            }
    }
}