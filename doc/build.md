# Build and Installation

### Platforms

The plugins currently only build on linux and OSX. The library will compile under Windows, but no plugins will be available.

### cmake

cmake is a build requirement for the included plugins

### output

All build plugins will be placed into ${OUT_DIR}/plugins, e.g. target/debug/plugins

## Integration into Grin

Cuckoo miner is integrated into grin, and can be turned on and off via grin's grin.toml file. All options are documented
within the configuration file.

## Building Cuckoo CUDA Libraries (Highly experimental, of course)

If the cuda build environment is installed, the build will attempt to build CUDA versions of the plugin. If they work for you,
they should give the best known solution times, with cuckoo30 generally finding a solution within a couple of seconds 
(on a 980ti, at least :). 

Instructions on how to set up the nvcc tool chain won't be provided here, but this will generally be installed 
as part of a 'cuda' package  on your distribution, and obviously depends on the correct nvidia driver package
being installed as well.

Once the libraries are built, you can experiment with calling them via the provided sample app in main.rs, or
experiment with dropping them into grin's target/debug/deps directory, and calling them by modifying
grin.toml as directed in the configuration file's documentation.



