cmake_minimum_required(VERSION 3.2)
project (CuckooMinerPlugins)

set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/plugins)
set (CMAKE_CXX_FLAGS "--std=c++11")

set (OPT "-O3")
set (DOPT "-DPREFETCH")

set (FLAGS "-Wno-format -Wno-deprecated-declarations -D_POSIX_C_SOURCE=200112L ${OPT} ${DOPT} -I. ${CPPFLAGS} -pthread")
set (GPP_FLAGS "-march=native -m64 ${FLAGS}")
set (CFLAGS "-Wno-format -fomit-frame-pointer ${OPT}")
set (GCC_FLAGS "-m64 -std=gnu11 ${CFLAGS}")

#blake2b prerequisite
set (BLAKE_2B "cuckoo/src/blake2b-ref.c")

#simple miner sources
#set (SIMPLE_MINER_SOURCES cuckoo/src/simple_miner.cpp ${BLAKE_2B})

#cuckoo_miner (lean_cpu) sources
set (LEAN_MINER_SOURCES cuckoo/src/siphash.h cuckoo/src/cuckoo.h cuckoo/src/lean_miner.hpp cuckoo/src/lean_miner.cpp ${BLAKE_2B})

#matrix_miner sources (mean_cpu)
set (MEAN_MINER_SOURCES cuckoo/src/siphash.h cuckoo/src/cuckoo.h cuckoo/src/mean_miner.hpp cuckoo/src/mean_miner.cpp ${BLAKE_2B})

#cuda_miner source (lean_cuda)
set (CUDA_MINER_SOURCES cuckoo/src/cuda_miner.cu ${BLAKE_2B} )

###lean_cpu miner targets
foreach(CUCKOO_SIZE 16 30)
    add_library(lean_cpu_${CUCKOO_SIZE} SHARED ${LEAN_MINER_SOURCES})
    #get edgebits
    math (EXPR EDGEBITS ${CUCKOO_SIZE}-1)
    set_target_properties(lean_cpu_${CUCKOO_SIZE} PROPERTIES COMPILE_FLAGS "${GPP_FLAGS} -DATOMIC -DEDGEBITS=${EDGEBITS}" PREFIX "" SUFFIX ".cuckooplugin" )
endforeach(CUCKOO_SIZE)

#matrix miner (mean_cpu) targets
foreach(CUCKOO_SIZE 16 30)
    if (CUCKOO_SIZE EQUAL 16)
       set (DXBITSFLAG "-DXBITS=0")
    else()
       set (DXBITSFLAG "")
    endif()

    add_library(mean_cpu_${CUCKOO_SIZE} SHARED ${MEAN_MINER_SOURCES})
    #get edgebits
    math (EXPR EDGEBITS ${CUCKOO_SIZE}-1)
    set_target_properties(mean_cpu_${CUCKOO_SIZE} PROPERTIES COMPILE_FLAGS "${GPP_FLAGS} -mavx2 ${DXBITSFLAG} -DNSIPHASH=8 -DSAVEEDGES -DEDGEBITS=${EDGEBITS}" PREFIX "" SUFFIX ".cuckooplugin")
endforeach(CUCKOO_SIZE)

#matrix miner (mean_cpu) compatible mode targets, for older processors
foreach(CUCKOO_SIZE 16 30)
    if (CUCKOO_SIZE EQUAL 16)
       set (DXBITSFLAG "-DXBITS=0")
    else()
       set (DXBITSFLAG "")
    endif()
    add_library(mean_compat_cpu_${CUCKOO_SIZE} SHARED ${MEAN_MINER_SOURCES})
    #get edgebits
    math (EXPR EDGEBITS ${CUCKOO_SIZE}-1)
    set_target_properties(mean_compat_cpu_${CUCKOO_SIZE} PROPERTIES COMPILE_FLAGS "${GPP_FLAGS} -mavx2 ${DXBITSFLAG} -DNSIPHASH=1 -DSAVEEDGES -DEDGEBITS=${EDGEBITS}" PREFIX "" SUFFIX ".cuckooplugin")
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
        foreach(CUCKOO_SIZE 30)
            #get edgebits
            math (EXPR EDGEBITS ${CUCKOO_SIZE}-1)
        cuda_add_library(lean_cuda_${CUCKOO_SIZE} SHARED ${CUDA_MINER_SOURCES} OPTIONS "-lssl -DEDGEBITS=${EDGEBITS}")
        set_target_properties(lean_cuda_${CUCKOO_SIZE} PROPERTIES PREFIX "" SUFFIX ".cuckooplugin")
        endforeach(CUCKOO_SIZE)
    endif(CUDA_FOUND)
endif(WITH_CUDA)
