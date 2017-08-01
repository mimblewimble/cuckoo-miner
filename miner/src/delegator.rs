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

//! Internal module responsible for job delegation, creating hashes 
//! and sending them to the plugin's internal queues. Used internally
//!
//!

use std::sync::{Arc, RwLock};
use std::{thread};
use std::mem::transmute;

use rand::{self, Rng};
use byteorder::{ByteOrder, BigEndian};
use blake2::blake2b::Blake2b;

use cuckoo_sys::{call_cuckoo_is_queue_under_limit,
                 call_cuckoo_push_to_input_queue,
                 call_cuckoo_read_from_output_queue,
                 call_cuckoo_start_processing,
                 call_cuckoo_stop_processing};
use error::CuckooMinerError;
use CuckooMinerJobHandle;
use CuckooMinerSolution;

/// From grin
/// The target is the 8-bytes hash block hashes must be lower than.
const MAX_TARGET: [u8; 8] = [0xf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];

type JobSharedDataType = Arc<RwLock<JobSharedData>>;
type JobControlDataType = Arc<RwLock<JobControlData>>;

/// Data intended to be shared across threads
pub struct JobSharedData {
    
    /// ID of the current running job (not currently used)
    pub job_id: u32, 
    
    /// The part of the header before the nonce, which this
    /// module will mutate in search of a solution
    pub pre_nonce: String, 

    /// The part of the header after the nonce
    pub post_nonce: String, 

    /// The target difficulty. Only solutions >= this
    /// target will be put into the output queue
    pub difficulty: u64,

    /// Output solutions
    pub solutions: Vec<CuckooMinerSolution>,
}

impl Default for JobSharedData {
    fn default() -> JobSharedData {
		JobSharedData {
            job_id:0,
            pre_nonce:String::from(""),
            post_nonce:String::from(""),
            difficulty: 0,
            solutions: Vec::new(),
		}
	}
}

impl JobSharedData {
    pub fn new(job_id: u32, 
               pre_nonce: &str, 
               post_nonce: &str,
               difficulty: u64) -> JobSharedData {
        JobSharedData {
            job_id: job_id,
            pre_nonce: String::from(pre_nonce),
            post_nonce: String::from(post_nonce),
            difficulty: difficulty,
            solutions: Vec::new(),
        }
    }
}

/// An internal structure to flag job control,
/// stopping mining threads, etc.

pub struct JobControlData {
    /// Whether the mining job is running
    pub is_running: bool,

    /// Whether the mining job is in the 
    /// process of shutting down
    pub is_stopping: bool,
}

impl Default for JobControlData {
    fn default() -> JobControlData {
		JobControlData {
            is_running: false,
            is_stopping: false,
		}
	}
}

/// Internal structure which controls and runs processing jobs.
///
///

pub struct Delegator {
    
    /// Data which is shared across all threads
    shared_data: JobSharedDataType,

    /// Job control flags which are shared across threads
    control_data: JobControlDataType,
}

impl Delegator {

    /// Create a new job delegator

    pub fn new(job_id:u32, pre_nonce: &str, post_nonce: &str, difficulty:u64)->Delegator{
        Delegator {
            shared_data: Arc::new(RwLock::new(JobSharedData::new(
                job_id, 
                pre_nonce,
                post_nonce,
                difficulty))),
            control_data: Arc::new(RwLock::new(JobControlData::default())),
        }
    }

    /// Starts the job loop, and initialises the internal plugin

    pub fn start_job_loop (self) -> Result<CuckooMinerJobHandle, CuckooMinerError> {
        //this will block, waiting until previous job is cleared
        //call_cuckoo_stop_processing();

        let shared_data=self.shared_data.clone();
        let control_data=self.control_data.clone();

        thread::spawn(move || {
            let result=self.job_loop();
            if let Err(e) = result {
                error!("Error in job loop: {:?}", e);
            }
        });
        Ok(CuckooMinerJobHandle {
            shared_data: shared_data, 
            control_data: control_data,
        })
    }


    /// Helper to convert a hex string

    fn from_hex_string(&self, in_str:&str)->Vec<u8> {
        let mut bytes = Vec::new();
        for i in 0..(in_str.len()/2){
            let res = u8::from_str_radix(&in_str[2*i .. 2*i+2],16);
            match res {
                Ok(v) => bytes.push(v),
                Err(e) => println!("Problem with hex: {}", e)
            }
        }
        bytes
    }

    /// Helper that returns a hash generated by the header parts and a
    /// nonce

    fn get_hash(&self, pre_nonce: &str, post_nonce: &str, nonce:u64)->[u8;32]{
        //Turn input strings into vectors
        let mut pre_vec = self.from_hex_string(pre_nonce);
        let mut post_vec = self.from_hex_string(post_nonce);
            
        //println!("nonce: {}", nonce);
        let mut nonce_bytes = [0; 8];
        BigEndian::write_u64(&mut nonce_bytes, nonce);
        let mut nonce_vec = nonce_bytes.to_vec();

        //Generate new header
        pre_vec.append(&mut nonce_vec);
        pre_vec.append(&mut post_vec);

        //println!("pre-vec: {:?}", pre_vec);

        //Hash
        let mut blake2b = Blake2b::new(32);
        blake2b.update(&pre_vec);
        
        let mut ret = [0; 32];
        ret.copy_from_slice(blake2b.finalize().as_bytes());
        ret
    }

    /// helper that generates a nonce and returns a header

    fn get_next_hash(&self, pre_nonce: &str, post_nonce: &str)->(u64, [u8;32]){
        //Generate new nonce
        let nonce:u64 = rand::OsRng::new().unwrap().gen();
        (nonce, self.get_hash(pre_nonce, post_nonce, nonce))
    }

    /// Helper to determing whether a solution meets a target difficulty
    /// based on same algorithm from grin

    fn meets_difficulty(&self, in_difficulty: u64, sol:CuckooMinerSolution)->bool {
        let max_target = BigEndian::read_u64(&MAX_TARGET);
		let num = BigEndian::read_u64(&sol.hash()[0..8]);
		max_target / num > in_difficulty
    }

    /// The main job loop. Pushes hashes to the plugin and reads solutions
    /// from the queue, putting them into the job's output queue. Continues
    /// until another thread sets the is_running flag to false

    fn job_loop(self) -> Result<(), CuckooMinerError>{
        //keep some unchanging data here, can move this out of shared
        //object later if it's not needed anywhere else
        let pre_nonce:String;
        let post_nonce:String;
        let difficulty;
        {
            let s = self.shared_data.read().unwrap();
            pre_nonce=s.pre_nonce.clone();
            post_nonce=s.post_nonce.clone();
            difficulty=s.difficulty;
        }
        debug!("Cuckoo-miner: Searching for solution >= difficulty {}", difficulty);
        {
            let mut s = self.control_data.write().unwrap();
            s.is_running=true;
        }

        if let Err(e) = call_cuckoo_start_processing() {
            return Err(CuckooMinerError::PluginProcessingError(
                    String::from(format!("Error starting processing plugin: {:?}", e))));
        }

        debug!("Cuckoo Miner Job loop processing");
        let mut solution=CuckooMinerSolution::new();

        loop {
            //Check if it's time to stop
            
            let s = self.control_data.read().unwrap();
            if !s.is_running {
                break;
            }
            
            while call_cuckoo_is_queue_under_limit().unwrap()==1{

                let (nonce, hash) = self.get_next_hash(&pre_nonce, &post_nonce);
                //println!("Hash thread 1: {:?}", hash);
                //TODO: make this a serialise operation instead
                let nonce_bytes:[u8;8] = unsafe{transmute(nonce.to_be())};
                call_cuckoo_push_to_input_queue(&hash, &nonce_bytes)?;
            }

            
            while call_cuckoo_read_from_output_queue(&mut solution.solution_nonces, &mut solution.nonce).unwrap()!=0 {
                //TODO: make this a serialise operation instead
                let nonce = unsafe{transmute::<[u8;8], u64>(solution.nonce)}.to_be();
                
                if self.meets_difficulty(difficulty, solution) {    
                    debug!("Cuckoo-miner: Solution Found for Nonce:({}), {:?}", nonce, solution);
                    let mut s = self.shared_data.write().unwrap();
                    s.solutions.push(solution.clone());
                } 
                
                
            }
        }

        //Do any cleanup
        debug!("Telling job thread to stop... ");
        if let Err(e) = call_cuckoo_stop_processing() {
            return Err(CuckooMinerError::PluginProcessingError(
                    String::from(format!("Error stopping processing plugin: {:?}", e))));
        }
        debug!("Cuckoo-Miner: Job loop has exited.");
        Ok(())
    }
}