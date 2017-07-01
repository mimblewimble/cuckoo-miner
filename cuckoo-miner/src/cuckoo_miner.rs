use std::mem;

use cuckoo_sys::{cuckoo_basic_mine};

pub struct CuckooMinerConfig {

}

impl CuckooMinerConfig{
    pub fn new()->CuckooMinerConfig{
        CuckooMinerConfig{

        }
    }
}

pub struct CuckooMiner{
    // Configuration
    pub config: CuckooMinerConfig,
}

impl CuckooMiner {

    pub fn new(config:CuckooMinerConfig, )->CuckooMiner{
        CuckooMiner{
            config: config,
        }
    }

    pub fn mine(&self, header: &[u8]){
        unsafe {
            let size_shift = 12;
                
            let mut sol_nonces=[0; 42];
            let len=42;
            
            let result=cuckoo_basic_mine(size_shift-1, header.as_ptr(), header.len(), sol_nonces.as_mut_ptr());
            if result==1 {
                println!("yay!");
            } else {
                println!("boo");
            }
        }
    }
}