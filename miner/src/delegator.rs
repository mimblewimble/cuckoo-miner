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

use std::sync::{Arc, Mutex};
use std::{thread, time};
use std::mem::transmute;

use rand::{self, Rng};
use byteorder::{ByteOrder, ReadBytesExt, BigEndian};
use tiny_keccak::Keccak;
use bigint::BigUint;

use cuckoo_sys::{call_cuckoo_is_queue_under_limit,
                 call_cuckoo_push_to_input_queue,
                 call_cuckoo_read_from_output_queue,
                 call_cuckoo_start_processing,
                 call_cuckoo_stop_processing};
use error::CuckooMinerError;
use CuckooMinerSolution;

/// From grin
/// The target is the 32-bytes hash block hashes must be lower than.
pub const MAX_TARGET: [u8; 32] = [0xf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                                  0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                                  0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];


// Struct intended to be shared across threads
pub struct JobSharedData {
    pub job_id: u32, 
    pub pre_nonce: String, 
    pub post_nonce: String, 
    pub difficulty: u32,
    pub running_flag: bool,
    pub solution_found: bool,
    pub solution: CuckooMinerSolution,
}

impl Default for JobSharedData {
    fn default() -> JobSharedData {
		JobSharedData {
            job_id:0,
            pre_nonce:String::from(""),
            post_nonce:String::from(""),
            difficulty:0,
            solution_found: false,
            running_flag:true,
            solution:CuckooMinerSolution::new(),
		}
	}
}

impl JobSharedData {
    pub fn new(job_id: u32, 
               pre_nonce: &str, 
               post_nonce: &str, 
               difficulty: u32) -> JobSharedData {
        JobSharedData {
            job_id: job_id,
            pre_nonce: String::from(pre_nonce),
            post_nonce: String::from(post_nonce),
            difficulty: difficulty,
            running_flag: true,
            solution_found: false,
            solution:CuckooMinerSolution::new(),
        }
    }

}

pub type JobSharedDataType = Arc<Mutex<JobSharedData>>;

//Some helper stuff, just put here for now
fn from_hex_string(in_str:&str)->Vec<u8> {
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

fn get_next_hash(pre_nonce: &str, post_nonce: &str)->(u64, [u8;32]){
    //Turn input strings into vectors
    let mut pre_vec = from_hex_string(pre_nonce);
    let mut post_vec = from_hex_string(post_nonce);
        
    //Generate new nonce
    let nonce:u64 = rand::OsRng::new().unwrap().gen();
    //println!("nonce: {}", nonce);
    let mut nonce_bytes = [0; 8];
    BigEndian::write_u64(&mut nonce_bytes, nonce);
    let mut nonce_vec = nonce_bytes.to_vec();

    //Generate new header
    pre_vec.append(&mut nonce_vec);
    pre_vec.append(&mut post_vec);

    //Hash
    let mut sha3 = Keccak::new_sha3_256();
	sha3.update(&pre_vec);
       
    let mut ret = [0; 32];
    sha3.finalize(&mut ret);
    (nonce, ret)
}

pub fn meets_target_difficulty(target: u32, solution:&CuckooMinerSolution)->bool{
    //get hashed solution target
    let max_target = BigUint::from_bytes_be(&MAX_TARGET);    
    let mut sha3 = Keccak::new_sha3_256();
    for n in 0..solution.solution_nonces.len() {
        let mut bytes = [0; 4];
	    BigEndian::write_u32(&mut bytes, solution.solution_nonces[n]);
        sha3.update(bytes.as_ref());
    }
    let mut ret = [0; 32];
    sha3.finalize(&mut ret);

    let h_num = BigUint::from_bytes_be(&ret[..]);

    let num=max_target / h_num;
    //println!("Difficulty is: {}", num);
    if num>=BigUint::from(target) {
        return true;
    }
    false

}

pub fn start_job_loop (shared_data: Arc<Mutex<JobSharedData>>){
    thread::spawn(move || {
        job_loop(shared_data);
    });
}

pub fn start_result_loop (shared_data: Arc<Mutex<JobSharedData>>){
    thread::spawn(move || {
        result_loop(shared_data);
    });
}

fn job_loop(shared_data: Arc<Mutex<JobSharedData>>) -> Result<(), CuckooMinerError>{
    //keep some unchanging data here, can move this out of shared
    //object later if it's not needed anywhere else
    let mut pre_nonce:String=String::new();
    let mut post_nonce:String=String::new();
    {
        let s = shared_data.lock().unwrap();
        pre_nonce=s.pre_nonce.clone();
        post_nonce=s.post_nonce.clone();
    }

    if let Err(e) = call_cuckoo_start_processing() {
        return Err(CuckooMinerError::PluginProcessingError(
                String::from("Error starting processing plugin.")));
    }
        
    loop {
         //Check if it's time to stop
        {
            let s = shared_data.lock().unwrap();
            if !s.running_flag {
                //Do any cleanup
                print!("Telling job thread to stop... ");
                call_cuckoo_stop_processing(); //should be a synchronous cleanup call
                println!("stopped.");
                break;
            }
        }

        while(call_cuckoo_is_queue_under_limit().unwrap()==1){
            let (nonce, hash) = get_next_hash(&pre_nonce, &post_nonce);
            //println!("Hash thread 1: {:?}", hash);
            //TODO: make this a serialise operation instead
            let nonce_bytes:[u8;8] = unsafe{transmute(nonce.to_be())};
            call_cuckoo_push_to_input_queue(&hash, &nonce_bytes)?;
        }
    }
    Ok(())
}

fn result_loop(shared_data: Arc<Mutex<JobSharedData>>) -> Result<(), CuckooMinerError>{
    let mut target_difficulty=0;
    {
        let s = shared_data.lock().unwrap();
        target_difficulty=s.difficulty;
    }
    loop {
        let mut solution = CuckooMinerSolution::new();
        while call_cuckoo_read_from_output_queue(&mut solution.solution_nonces, &mut solution.nonce).unwrap()!=0 {
            {
                let s = shared_data.lock().unwrap();
                if !s.running_flag {
                    //println!("Exiting job thread.");
                    break;
                }
            }
            //TODO: make this a serialise operation instead
            let nonce = unsafe{transmute::<[u8;8], u64>(solution.nonce)}.to_be();
            
            if meets_target_difficulty(target_difficulty, &solution){
                println!("Solution Found for Nonce:({}), {:?}", nonce, solution);
                let mut s = shared_data.lock().unwrap();
                target_difficulty=s.difficulty;
                s.solution_found=true;
                s.solution = solution.clone();
            }
            
        }
    }
}
