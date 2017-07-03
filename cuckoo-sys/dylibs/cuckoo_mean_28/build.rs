extern crate common;

use common::*;

fn main() {
    build_cuckoo(String::from("sources.txt"), "mean", 28);
}
