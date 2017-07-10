# Build and Installation

### NB


At the moment, cuckoo-miner has a dependency on a dynamic version
of Rust's stdlib, so before plugins will load, the rust toolchain's lib directory needs to be on your library path, like so:

```
export LD_LIBRARY_PATH=/home/user/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib
```

This is less than ideal, and will need to be addressed.

## Integration into Grin (Experimental)

This section assumes familiarity with Grin and how to build and run it.

Clone the cuckoo-miner into a directory called cuckoo-miner, which should be placed next to grin. Your directory
structure should look like:

```
   /grin
    cuckoo-miner
```

Cuckoo-miner is not enabled by default. In Grin's root Cargo.toml file, you should see the following:

```
   [dependencies.grin_grin]
    path = "./grin"
    version = "*"
    default-features = false
    #Comment this in to use the cuckoo-miner package
    #ensure cuckoo-miner is cloned next to the 
    #grin directory
    #features = ["cuckoo_miner", "use-cuckoo-miner"]
```

To enable the inclusion of cuckoo-miner in Grin, comment in the last #features line. Then, in 
grin/Cargo.toml, comment in the line:

```
#cuckoo_miner = { version = "*", optional=true, path = "../../cuckoo-miner"}
```

It's also recommended to set the test consensus value TEST_SIZESHIFT to 16, to correspond with the smallest viable cuckoo size that can be used by all miners. 

Clean and rebuild, and cuckoo-miner will be used when mining with the server -m command.

At the moment, cuckoo-plugin is integrated for experimentation via the file grin/src/plugin_miner.rs. A plugin is selected by giving
its partial name in the lines:

```

		let filter = format!("simple_{}", TEST_SIZESHIFT);
    	let caps = plugin_manager.get_available_plugins(&filter).unwrap();
```

With TEST_SIZESHIFT being read from the CONSENSUS.rs value, so that blocks mined using cuckoo-miner will validate. 
