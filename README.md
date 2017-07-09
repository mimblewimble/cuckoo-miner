# cuckoo-miner

Cuckoo-miner is Rust crate which provides various implementations of the Cuckoo Cycle algorithm. It is primarily intended for
integration into the Grin/Mimblewimble blockchain's proof-of-work system, however it is suitable for other purposes, such as
for the creation of a standalone Cuckoo Cycle mining client or integration into other blockchains.

Cuckoo-miner uses a plugin architecture, and provides a common interface for plugins wishing to provide a Cuckoo Cycle
implementation. Full details of this plugin interface can be found in the crate's API documentation, but a plugin is 
essentially a DLL that (currently) provides implementations of two functions:

* call_cuckoo - Which accepts an array of bytes to use as a seed for the Cuckoo Cycle algorithm, and returns a solution set,
if found.

* cuckoo_description - Which provides details about the plugin's capabilities, such as it's name, cuckoo size, description,
and will be expanded to include details such as whether a plugin can be run on a host system.

The main interface into cuckoo-miner is the 'CuckooMiner' struct, which handles selection of a plugin and running the selected
Cuckoo Cycle implementation at a high level. Cuckoo-miner also provides a helper 'CuckooPluginManager' struct, which assists with loading a directory full of mining plugins, returning useful information to the caller about each available plugin. Full details
are found in the crate's documentation.

## Plugins

Currently, cuckoo-miner provides a set of pre-built plugins directly adapted from the latest implementations in 
[John Tromp's github repository](https://github.com/tromp/cuckoo), with each plugin optimised for different sizes of Cuckoo Graph.
Currently, the provided (and planned plugins are:)

* cuckoo_simple (with Cuckoo Sizes ranging between 16-30), the basic implementation of the algorithm
* cuckoo_edgetrim (with Cuckoo Sizes ranging between 16-30), the Cuckoo Cycle algorithm with edge trimming
* cuckoo_mean (planned)
* cuckoo_tomato (planned)
* cuckoo_gpu (planned)

## Installation and Building

For instructions on how to build cuckoo-miner and integrate it into Grin, see the see the [build docs](doc/build.md).

## Architecture

The reasoning behind the plugin architecture are several fold. John Tromps implementations are likely to remain the fastest
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

