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

use cuckoo_sys::{call_cuckoo, 
                 load_cuckoo_lib,
                 get_available_plugins};

use cuckoo_config::*;

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