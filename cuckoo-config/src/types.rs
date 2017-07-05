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

const CUCKOO_SOLUTION_SIZE:usize = 42;

// Errors possible when trying to create the miner
#[derive(Debug)]
pub enum CuckooMinerError {
    // Error when requested miner is not loaded
    MinerNotLoadedError(String),

    // Error when requested miner is not implemented
    NotImplementedError(String),

    // Unexpected return code from miner call
    UnexpectedResultError(u32),

}

#[derive(Debug)]
pub enum CuckooMinerImplType {
    // Represents the miner in simple_miner.cpp
	Simple,

    // The base miner, from cucko_main.cpp
    Base,

    // Time/Memory Trade-off miner, tomato_miner.cpp
    Tomato,

    // Mean miner, mean_miner.ccp
    Mean,

    // CUDA miner, cuda_miner.cu
    CUDA,
}

impl fmt::Display for CuckooMinerImplType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{:?}", self)
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
    // The implementation of the miner to use
    pub miner_impl: CuckooMinerImplType,

    // Edgebits, or cuckoo size (sizeshift) to use
    pub cuckoo_size: u8,


}

impl Default for CuckooMinerConfig {
	fn default() -> CuckooMinerConfig {
		CuckooMinerConfig{
            miner_impl: CuckooMinerImplType::Base,
			cuckoo_size: 12,
		}
	}
}

impl CuckooMinerConfig{
    pub fn new()->CuckooMinerConfig{
        CuckooMinerConfig::default()
    }
}
