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

//! Types exposed for Cuckoo Miner

// This assumption that the solution size will be 42 will be
// maintained, to avoid having to allocate memory within
// the called C functions

use std::{fmt,cmp};
use std::io;
use std::string;

const CUCKOO_SOLUTION_SIZE:usize = 42;

// Errors possible when trying to create the miner
#[derive(Debug)]
pub enum CuckooMinerError {
    // Error when requested miner is not loaded
    PluginNotLoadedError(String),

    // Error when requested miner is not implemented
    PluginNotFoundError(String),

    // Error when no plugins exist in target directory
    NoPluginsFoundError(String),

    // Unexpected return code from miner call
    UnexpectedResultError(u32),

    // IO Error
    PluginIOError(String)
}

impl From<io::Error> for CuckooMinerError {
    fn from(error: io::Error) -> Self {
        CuckooMinerError::PluginIOError(String::from(format!("Error loading plugin: {}",error)))
    }
}

impl From<string::FromUtf8Error> for CuckooMinerError {
    fn from(error: string::FromUtf8Error) -> Self {
        CuckooMinerError::PluginIOError(String::from(format!("Error loading plugin description: {}",error)))
    }
}


#[derive(Debug)]
pub struct CuckooPluginCapabilities {
    // The plugin's descriptive name
    // As reported by the plugin
    pub name: String,

    // The plugin's reported description
    pub description: String,

    // The full path to the plugin
    pub full_path: String,

    // The plugin's file name
    pub file_name: String,
}

impl Default for CuckooPluginCapabilities {
	fn default() -> CuckooPluginCapabilities {
		CuckooPluginCapabilities{
            name: String::from(""),
            description: String::from(""),
			full_path: String::from(""),
            file_name: String::from(""),
		}
	}
}

impl fmt::Display for CuckooPluginCapabilities{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"Name: {}\nDescription:{}\nPath:{}\n", self.name, self.description, self.full_path)
    }
}

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
    // The directory in which mining plugins are found
    pub plugin_dir: String,

    // The implementation of the miner to use
    pub plugin_name: String,

    // Number of threads to use
    pub num_threads: u32,

    // Number of trims
    pub num_trims: u32,


}

impl Default for CuckooMinerConfig {
	fn default() -> CuckooMinerConfig {
		CuckooMinerConfig{
            plugin_dir: String::from("target/debug"),
            plugin_name: String::from("cuckoo_basic_12"),
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
