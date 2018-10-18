[![Build Status](https://travis-ci.org/mimblewimble/cuckoo-miner.svg?branch=master)](https://travis-ci.org/mimblewimble/cuckoo-miner) [![Gitter chat](https://badges.gitter.im/grin_community/Lobby.png)](https://gitter.im/grin_community/Lobby)
# cuckoo-miner

Cuckoo-miner is Rust crate which provides various implementations of the Cuckoo Cycle algorithm. It is primarily intended for
integration into the Grin/Mimblewimble blockchain's proof-of-work system, however it aims to eventually be suitable for other 
purposes, such as for the creation of a standalone Cuckoo Cycle mining client or integration into other blockchains.

Cuckoo-miner uses a plugin architecture, and provides a common interface for plugins wishing to provide a Cuckoo Cycle
implementation. Full details of this plugin interface can be found in the crate's API documentation, but a plugin is 
essentially a DLL that (currently) provides implementations of several functions, for example:

* call_cuckoo - Synchronous function Which accepts an array of bytes to use as a seed for the Cuckoo Cycle algorithm, and returns a solution set,
if found. 

* cuckoo_start_processing - starts asyncronous processing, reading hashes from a queue and returning any results found to an output queue.

* cuckoo_description - Which provides details about the plugin's capabilities, such as it's name, cuckoo size, description,
and will be expanded to include details such as whether a plugin can be run on a host system.

Cuckoo-miner can be run in either of two modes. Syncronous mode takes a single hash, searches it via the cuckoo cycle algorithm in the loaded
plugin, and returns a result. Asynchronous mode, based on a Stratum-esque notifiy function, takes the required parts of a block header, and mutates
a hash of the header with random nonces until it finds a solution. This is performed asyncronously by the loaded plugin, which reads hashes
from a thread-safe queue and returns results on another until told to stop.

The main interface into cuckoo-miner is the 'CuckooMiner' struct, which handles selection of a plugin and running the selected
Cuckoo Cycle implementation at a high level. Cuckoo-miner also provides a helper 'CuckooPluginManager' struct, which assists with loading a
directory full of mining plugins, returning useful information to the caller about each available plugin. Full details
are found in the crate's documentation.

## Plugins

Currently, cuckoo-miner provides a set of pre-built plugins directly adapted from the latest implementations in 
[John Tromp's github repository](https://github.com/tromp/cuckoo), with each plugin optimised for different sizes of Cuckoo Graph.
Currently, the provided (and planned plugins are:)

* lean_cpu (cuckoo_miner.cpp) (Cuckoo Sizes 16 and 30), the baseline CPU algorithm, which constrains memory use at the expense of speed
* mean_cpu (matrix_miner.cpp) (Cuckoo size 30), currently the fastest CPU solver, but with much larger memory requirements
* lean_cuda (cuda_miner.cu) (Cuckoo Size 30), lean cuckoo algorithm optimised for NVidia GPUs, (only built if cuda build environment is installed)
* mean_cuda (PLANNED) (Cuckoo Size 30) cuda version of the mean algorithm, should be the fastest solver when implemented

These plugins are currently built by cmake as part of the cuckoo-sys module. The cmake scripts will attempt to detect the underlying environment
as well as possible and build plugins accordingly (WIP)

## Installation and Building

A tag of cuckoo miner is intergrated into the master of Grin, but for instructions on how to build cuckoo-miner and integrate it into 
Grin locally, see the see the [build docs](doc/build.md).

## Architecture

The reasoning behind the plugin architecture are several fold. John Tromp's implementations are likely to remain the fastest
and most robust Cuckoo Cycle implementations for quite some time, and it was desirable to come up a method of exposing them 
to Grin in a manner that changes them as little as possible. As written, they are optimised with a lot of static 
array initialisation and intrinsics in some cases, and a development of a dynamic version of them would incur tradeoffs
and likely be far slower. The 'plugins' included in cuckoo-miner are mostly redefinitions of the original main functions
exposed as DLL symbols, and thus change them as little as possible. This approach also allows for quick and painless
merging of future updates to the various implementations.

Further, the inclusion of intrisics mean that some variants of the algorithm may not run
on certain architectures, and a plugin-architecture which can query the host system for its capabilites is desirable.

## Status

Cuckoo-miner is very much in experimental alpha phase, and will be developed more fully alongside Grin. The many
variations on cuckoo size included are mostly for Grin testing, and will likely be reduced once Grin's POW parameters
become more established.

## Further Reading

The Cuckoo Cycle POW algorithm is the work of John Tromp, and the most up-to-date documentation and implementations
can be found in [his github repository](https://github.com/tromp/cuckoo). The
[white paper](https://github.com/tromp/cuckoo/blob/master/doc/cuckoo.pdf) is the best source of
further technical details. 


A higher level overview of Cuckoo Cycle and how it relates to Grin's Proof-of-work system can be found in 
[Grin's POW Documentation](https://github.com/ignopeverell/grin/blob/master/doc/pow/pow.md).

