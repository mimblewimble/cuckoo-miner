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

use rand::{self, Rng};
use std::{thread, time};
use byteorder::{ByteOrder, ReadBytesExt, BigEndian};
use tiny_keccak::Keccak;

pub struct Delegator {
    job_id: u32, 
    pre_nonce: String, 
    post_nonce: String, 
    difficulty: u32,
} 

impl Default for Delegator{
	fn default() -> Delegator {
		Delegator {
            job_id:0,
            pre_nonce:String::from(""),
            post_nonce:String::from(""),
            difficulty:0,
		}
	}
}

impl Delegator {
    /// Returns a new instance of Delegator
    pub fn new()->Delegator{
        Delegator::default()
    }

    pub fn init_job(&mut self, 
                    job_id: u32, 
                    pre_nonce: &str, 
                    post_nonce: &str, 
                    difficulty: u32) {
        self.job_id=job_id;
        self.pre_nonce=String::from(pre_nonce);
        self.post_nonce=String::from(post_nonce);
        self.difficulty=difficulty;        
    }

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

    fn get_next_hash(&mut self)->[u8;32]{
        //Turn input strings into vectors
        let mut pre_vec = self.from_hex_string(&self.pre_nonce);
        let mut post_vec = self.from_hex_string(&self.post_nonce);
        
        //Generate new nonce
        let nonce:u64 = rand::OsRng::new().unwrap().gen();
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
        ret
    }

    pub fn job_loop(&mut self){
        loop {
            thread::sleep(time::Duration::from_millis(1000));
            println!("Hash: {:?}", self.get_next_hash());
        }
    }

    pub fn result_loop(&mut self){
        loop {
            thread::sleep(time::Duration::from_millis(750));
            println!("Result loop");
        }
    }
}