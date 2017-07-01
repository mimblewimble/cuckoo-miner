extern crate cuckoo_miner;
extern crate env_logger;

use cuckoo_miner::test_function;

pub fn main(){
    env_logger::init();
    println!("Main");
    test_function();
}