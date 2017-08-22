cmake_minimum_required(VERSION 3.8)
project (CuckooMinerPlugins)

set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/plugins)
set (CMAKE_CXX_FLAGS "--std=c++11")

set (OPT "-O3")
set (DOPT "-DPREFETCH")

set (FLAGS "-Wall -Wno-format -Wno-deprecated-declarations -D_POSIX_C_SOURCE=200112L ${OPT} ${DOPT} -I. ${CPPFLAGS} -pthread")
set (GPP_FLAGS "-march=native -m64 ${FLAGS}")
set (CFLAGS "-Wall -Wno-format -fomit-frame-pointer ${OPT}")
set (GCC_FLAGS "-m64 -std=gnu11 ${CFLAGS}")

#blake2b prerequisite
set (BLAKE_2B "cuckoo/src/blake2b-ref.c")

#simple miner sources
set (SIMPLE_MINER_SOURCES cuckoo/src/simple_miner.cpp ${BLAKE_2B})

#cuckoo_miner (edgetrim) sources
set (CUCKOO_MINER_SOURCES cuckoo/src/siphash.h cuckoo/src/cuckoo.h cuckoo/src/cuckoo_miner.hpp cuckoo/src/cuckoo_miner.cpp ${BLAKE_2B})

#matrix_miner sources
set (MATRIX_MINER_SOURCES cuckoo/src/siphash.h cuckoo/src/cuckoo.h cuckoo/src/matrix_miner.hpp cuckoo/src/matrix_miner.cpp ${BLAKE_2B})

#tomato_miner sources
set (TOMATO_MINER_SOURCES cuckoo/src/siphash.h cuckoo/src/cuckoo.h cuckoo/src/tomato_miner.h cuckoo/src/tomato_miner.cpp ${BLAKE_2B} )

#cuda_miner sources
set (CUDA_MINER_SOURCES cuckoo/src/cuda_miner.cu ${BLAKE_2B} )

#cuckoo simple miner targets
foreach(CUCKOO_SIZE 12 16 20 25 28 30)
    add_library(cuckoo_simple_${CUCKOO_SIZE} SHARED ${SIMPLE_MINER_SOURCES})
    #get edgebits
    math (EXPR EDGEBITS ${CUCKOO_SIZE}-1)
    set_target_properties(cuckoo_simple_${CUCKOO_SIZE} PROPERTIES COMPILE_FLAGS "${GPP_FLAGS} -DEDGEBITS=${EDGEBITS}" )
endforeach(CUCKOO_SIZE)

###cuckoo edgetrim miner targets
foreach(CUCKOO_SIZE 16 20 25 28 30)
    add_library(cuckoo_edgetrim_${CUCKOO_SIZE} SHARED ${CUCKOO_MINER_SOURCES})
    #get edgebits
    math (EXPR EDGEBITS ${CUCKOO_SIZE}-1)
    set_target_properties(cuckoo_edgetrim_${CUCKOO_SIZE} PROPERTIES COMPILE_FLAGS "${GPP_FLAGS} -DATOMIC -DEDGEBITS=${EDGEBITS}" )
endforeach(CUCKOO_SIZE)

#matrix miner targets
foreach(CUCKOO_SIZE 26 28 30)
    add_library(cuckoo_matrix_${CUCKOO_SIZE} SHARED ${MATRIX_MINER_SOURCES})
    #get edgebits
    math (EXPR EDGEBITS ${CUCKOO_SIZE}-1)
    set_target_properties(cuckoo_matrix_${CUCKOO_SIZE} PROPERTIES COMPILE_FLAGS "${GPP_FLAGS} -mavx2 -DNSIPHASH=8 -DEDGEBITS=${EDGEBITS}" )
endforeach(CUCKOO_SIZE)

##cuckoo tomato miner targets
foreach(CUCKOO_SIZE 16 20 25 28 30)
    add_library(cuckoo_tomato_${CUCKOO_SIZE} SHARED ${TOMATO_MINER_SOURCES})
    #get edgebits
    math (EXPR EDGEBITS ${CUCKOO_SIZE}-1)
    set_target_properties(cuckoo_tomato_${CUCKOO_SIZE} PROPERTIES COMPILE_FLAGS "${GPP_FLAGS} -DATOMIC -DEDGEBITS=${EDGEBITS}" )
endforeach(CUCKOO_SIZE)

# Cuda
option(WITH_CUDA "Build NVIDIA-CUDA support" TRUE)
if(WITH_CUDA)
    include("cmake/find_cuda.cmake")
    IF (CUDA_FOUND)
    #TODO: detect or make this a flag, if gcc 5 is installed separately for cuda
	set (CUDA_HOST_COMPILER "/usr/bin/gcc-5")
	set (CUDA_PROPAGATE_HOST_FLAGS ON)
        ##cuckoo tomato miner targets
        foreach(CUCKOO_SIZE 16 20 25 28 30)
            #get edgebits
            math (EXPR EDGEBITS ${CUCKOO_SIZE}-1)
	    cuda_add_library(cuckoo_cuda_${CUCKOO_SIZE} SHARED ${CUDA_MINER_SOURCES} OPTIONS "-lssl -DEDGEBITS=${EDGEBITS}")
        endforeach(CUCKOO_SIZE)
    endif(CUDA_FOUND)
endif(WITH_CUDA)
