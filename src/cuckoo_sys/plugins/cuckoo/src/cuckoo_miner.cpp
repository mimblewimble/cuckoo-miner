// Cuckoo Cycle, a memory-hard proof-of-work
// Copyright (c) 2013-2016 John Tromp

#include "cuckoo_miner.hpp"
#include <unistd.h>
#include <pthread.h>
#ifdef __APPLE__
#include "osx_barrier.h"
#endif

#include "cuckoo_miner/cuckoo_miner_adds.h"

#define MAXSOLS 8
// arbitrary length of header hashed into siphash key
#define HEADERLEN 80

extern "C" int cuckoo_call(char* header_data, 
                           int header_length,
                           u32* sol_nonces){
  
  u64 start_time=timestamp();
  int c;
  int nonce = 0;
  int range = 1;

  assert(NUM_THREADS_PARAM>0);

  //assert(header_length <= sizeof(header_data));

  print_buf("(Cuckoo Miner) Coming in is: ", (const unsigned char*) header_data, header_length);

  //memset(header, 0, sizeof(header));
  /*while ((c = getopt (argc, argv, "h:m:n:r:t:")) != -1) {
    switch (c) {
      case 'h':
        len = strlen(optarg);
        assert(len <= sizeof(header));
        memcpy(header, optarg, len);
        break;
      case 'n':
        nonce = atoi(optarg);
        break;
      case 'r':
        range = atoi(optarg);
        break;
      case 'm':
        ntrims = atoi(optarg);
        break;
      case 't':
        nthreads = atoi(optarg);
        break;
    }
  }*/

  printf("Looking for %d-cycle on cuckoo%d(\"%s\",%d", PROOFSIZE, EDGEBITS+1, header_data, nonce);
  if (range > 1)
    printf("-%d", nonce+range-1);
  printf(") with 50%% edges, %d trims, %d threads\n", NUM_TRIMS_PARAM, NUM_THREADS_PARAM);

  u64 edgeBytes = NEDGES/8, nodeBytes = TWICE_ATOMS*sizeof(atwice);
  int edgeUnit, nodeUnit;
  for (edgeUnit=0; edgeBytes >= 1024; edgeBytes>>=10,edgeUnit++) ;
  for (nodeUnit=0; nodeBytes >= 1024; nodeBytes>>=10,nodeUnit++) ;
  printf("Using %d%cB edge and %d%cB node memory, %d-way siphash, and %d-byte counters\n",
     (int)edgeBytes, " KMGT"[edgeUnit], (int)nodeBytes, " KMGT"[nodeUnit], NSIPHASH, SIZEOF_TWICE_ATOM);

  thread_ctx *threads = (thread_ctx *)calloc(NUM_THREADS_PARAM, sizeof(thread_ctx));
  assert(threads);
  cuckoo_ctx ctx(NUM_THREADS_PARAM, NUM_TRIMS_PARAM, MAXSOLS);

  u32 sumnsols = 0;
  for (int r = 0; r < range; r++) {
    //ctx.setheadernonce(header, sizeof(header), nonce + r);
    ctx.setheadergrin(header_data, header_length);
    printf("k0 %lx k1 %lx\n", ctx.sip_keys.k0, ctx.sip_keys.k1);
    for (int t = 0; t < NUM_THREADS_PARAM; t++) {
      threads[t].id = t;
      threads[t].ctx = &ctx;
      int err = pthread_create(&threads[t].thread, NULL, worker, (void *)&threads[t]);
      assert(err == 0);
    }
    for (int t = 0; t < NUM_THREADS_PARAM; t++) {
      int err = pthread_join(threads[t].thread, NULL);
      assert(err == 0);
    }
    for (unsigned s = 0; s < ctx.nsols; s++) {
      printf("Solution");
      //just return with the first solution we get
      for (int i = 0; i < PROOFSIZE; i++) {
        printf(" %jx", (uintmax_t)ctx.sols[s][i]);
        sol_nonces[i] = ctx.sols[s][i]; 
      }
      free(threads);
      printf("\n");
      if(SINGLE_MODE){
         update_stats(start_time);
      }
      return 1;
    }
    sumnsols += ctx.nsols;
  }
  free(threads);
  printf("%d total solutions\n", sumnsols);
  if(SINGLE_MODE){
     update_stats(start_time);
  }
  return 0;
}





