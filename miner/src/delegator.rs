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
use std::{thread, time};
use std::mem::transmute;

use rand::{self, Rng};
use byteorder::{ByteOrder, BigEndian};
use blake2::blake2b::Blake2b;

use cuckoo_sys::{call_cuckoo_is_queue_under_limit,
                 call_cuckoo_push_to_input_queue,
                 call_cuckoo_read_from_output_queue,
                 call_cuckoo_start_processing,
                 call_cuckoo_stop_processing,
                 call_cuckoo_hashes_since_last_call};
use error::CuckooMinerError;
use CuckooMinerSolution;

/// From grin
/// The target is the 8-bytes hash block hashes must be lower than.
const MAX_TARGET: [u8; 8] = [0xf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];

type JobSharedDataType = Arc<RwLock<JobSharedData>>;
type JobControlDataType = Arc<RwLock<JobControlData>>;

// Struct intended to be shared across threads
struct JobSharedData {
    pub job_id: u32, 
    pub pre_nonce: String, 
    pub post_nonce: String, 
    pub difficulty: u64,
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

pub struct JobControlData {
    pub is_running: bool,
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

/// Handle to the miner's running job, used to read solutions
/// or to control the job. Internal members are not exposed
/// and all interactions should be via public functions
///

pub struct CuckooMinerJobHandle {
    shared_data: JobSharedDataType,
    control_data: JobControlDataType,
}

impl CuckooMinerJobHandle {

    /// #Description 
    ///
    /// Returns a solution if one is currently waiting.
    ///
    /// #Returns
    ///
    /// If a solution was found and is waiting in the plugin's input queue, returns
    /// Ok([CuckooMinerSolution](struct.CuckooMinerSolution.html)). If there
    /// no solution waiting, returns None

    pub fn get_solution(&self)->Option<CuckooMinerSolution>{
        //just to prevent endless needless locking of this
        //when using fast test miners, in real cuckoo30 terms
        //this shouldn't be an issue
        //TODO: Make this less blocky
        thread::sleep(time::Duration::from_millis(10));
        //let time_pre_lock=Instant::now();
        let mut s=self.shared_data.write().unwrap();
        //let time_elapsed=Instant::now()-time_pre_lock;
        //println!("Get_solution Time spent waiting for lock: {}", time_elapsed.as_secs()*1000 +(time_elapsed.subsec_nanos()/1_000_000)as u64);
        if s.solutions.len()>0 {
            let sol = s.solutions.pop().unwrap();
            return Some(sol);
        }
        None
    }

    /// #Description 
    ///
    /// Stops the current job, and signals for the loaded plugin to stop processing
    /// and perform any cleanup it needs to do.
    ///
    /// #Returns
    ///
    /// Nothing

    pub fn stop_jobs(&self) {
        debug!("Stop jobs called");
        let mut r=self.control_data.write().unwrap();
        r.is_running=false;
        debug!("Stop jobs unlocked?");
    }

    /// #Description 
    ///
    /// Returns the number of hashes processed by the plugin since the last time
    /// this function was called. 
    ///
    /// #Returns
    ///
    /// Ok(n) if successful, with n containing the number of hashes processed
    /// since the last time this function was called.
    /// A [CuckooMinerError](../../error/error/enum.CuckooMinerError.html) 
    /// with specific detail if an error occurred.

    pub fn get_hashes_since_last_call(&self)->Result<u32, CuckooMinerError>{
        match call_cuckoo_hashes_since_last_call() {
            Ok(result) => {
                return Ok(result);
            },
            Err(_) => {
                return Err(CuckooMinerError::PluginNotLoadedError(
                String::from("Please call init to load a miner plug-in")));
            }
        }
    
    }

        
}

/// Internal structure which controls and runs processing jobs.
///
///

pub struct Delegator {
    shared_data: JobSharedDataType,
    control_data: JobControlDataType,
}

impl Delegator {

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

    pub fn start_job_loop (self) -> CuckooMinerJobHandle {
        //this will block, waiting until previous job is cleared
        //call_cuckoo_stop_processing();

        let shared_data=self.shared_data.clone();
        let control_data=self.control_data.clone();
        thread::spawn(move || {
            self.job_loop();
        });
        CuckooMinerJobHandle {
            shared_data: shared_data, 
            control_data: control_data,
        }
    }


    //Some helper stuff, just put here for now
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

    //returns the nonce and the hash it generates
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

    fn get_next_hash(&self, pre_nonce: &str, post_nonce: &str)->(u64, [u8;32]){
        //Generate new nonce
        let nonce:u64 = rand::OsRng::new().unwrap().gen();
        (nonce, self.get_hash(pre_nonce, post_nonce, nonce))
    }

    fn meets_difficulty(&self, in_difficulty: u64, sol:CuckooMinerSolution)->bool {
        let max_target = BigEndian::read_u64(&MAX_TARGET);
		let num = BigEndian::read_u64(&sol.hash()[0..8]);
		max_target / num > in_difficulty
    }

    fn job_loop(mut self) -> Result<(), CuckooMinerError>{
        //keep some unchanging data here, can move this out of shared
        //object later if it's not needed anywhere else
        let mut pre_nonce:String=String::new();
        let mut post_nonce:String=String::new();
        let mut difficulty=0;
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
                    String::from("Error starting processing plugin.")));
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
        call_cuckoo_stop_processing();
        debug!("Cuckoo-Miner: Job loop has exited.");
        Ok(())
    }
}