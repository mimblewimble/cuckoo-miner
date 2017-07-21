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
                    pre_nonce: String, 
                    post_nonce: String, 
                    difficulty: u32) {
        self.job_id=job_id;
        self.pre_nonce=pre_nonce;
        self.post_nonce=post_nonce;
        self.difficulty=difficulty;
    }

    pub fn delegator_loop(){

    }
}