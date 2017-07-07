extern crate common;

use common::CuckooBuildEnv;

fn main() {
    CuckooBuildEnv::new().build_cuckoo(String::from("sources.txt"), "edgetrim", 30);
}
