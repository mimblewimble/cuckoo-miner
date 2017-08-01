# Build and Installation

### NB

At the moment, cuckoo-miner has a dependency on a dynamic version
of Rust's stdlib, so before plugins will load, the rust toolchain's lib directory needs to be on your library path, like so:

```
export LD_LIBRARY_PATH=/home/user/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib
```

This is less than ideal, and will need to be addressed.

## Integration into Grin

Cuckoo miner is integrated into grin, and can be turned on and off via grin's grin.toml file. All options are documented
within the configuration file.

## Building Cuckoo CUDA Libraries (Highly experimental, of course)

The included version of the cuckoo submodule contains a makefile that will build CUDA versions of the plugin, provided
the nvcc toolchain is installed on the target system and you're running a compatible card. If they work for you,
they should give the best known solution times, with cuckoo30 generally finding a solution within a couple of seconds 
(on a 980ti, at least :). This is also a good demonstration of the flexibility of the plugin architecture, as the 
cuda plugins are simple DLLs implementing the C interface, and just need to be dropped into place to use them in Grin.

Instructions on how to set up the nvcc tool chain won't be provided here, but this will generally be installed 
as part of a 'cuda' package  on your distribution, and obviously depends on the correct nvidia driver package
being installed as well.

Once nvcc is in your path, you should be able to build the libcuckoo_cuda plugins by running

```
cd cuckoo-sys/cuckoo/src
make libcuda
```

Provided your CUDA toolchain is set up correctly, this will build the cuda plugins into {project root}/target/debug.

Once the libraries are built, you can experiment with calling them via the provided sample app in main.rs, or
experiment with dropping them into grin's target/debug/deps directory, and calling them by modifying
grin.toml as directed in the configuration file's documentation.



