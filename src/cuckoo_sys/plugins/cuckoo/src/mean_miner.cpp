// Cuckoo Cycle, a memory-hard proof-of-work
// Copyright (c) 2013-2017 John Tromp

#include "mean_miner.hpp"
#include <unistd.h>
#include <sys/time.h>

int NUM_THREADS_PARAM=60;
int NUM_TRIMS_PARAM=1;

extern "C" int cuckoo_call(char* header_data, 
                           int header_length, 
                           u32* sol_nonces){
  
  int c;
  int nonce = 0;
  int range = 1;
  struct timeval time0, time1;
  u32 timems;

  int ntrims = NUM_TRIMS_PARAM;
  int nthreads = NUM_THREADS_PARAM;

  if (ntrims<=0) ntrims==1;

  assert(nthreads >= 1);
  
  /*while ((c = getopt (argc, argv, "h:m:n:r:t:x:")) != -1) {
    switch (c) {
      case 'h':
        len = strlen(optarg);
        assert(len <= sizeof(header));
        memcpy(header, optarg, len);
        break;
      case 'x':
        len = strlen(optarg)/2;
        assert(len == sizeof(header));
        for (u32 i=0; i<len; i++)
          sscanf(optarg+2*i, "%2hhx", header+i);
        break;
      case 'n':
        nonce = atoi(optarg);
        break;
      case 'r':
        range = atoi(optarg);
        break;
      case 'm':
        ntrims = atoi(optarg) & -2; // make even as required by solve()
        break;
      case 't':
        nthreads = atoi(optarg);
        break;
    }
  }*/
  printf("Looking for %d-cycle on cuckoo%d(\"%s\",%d", PROOFSIZE, EDGEBITS+1, header_data, nonce);
  if (range > 1)
    printf("-%d", nonce+range-1);
  printf(") with 50%% edges\n");

  solver_ctx ctx(nthreads, ntrims);

  u32 sbytes = ctx.sharedbytes();
  u32 tbytes = ctx.threadbytes();
  int sunit,tunit;
  for (sunit=0; sbytes >= 10240; sbytes>>=10,sunit++) ;
  for (tunit=0; tbytes >= 10240; tbytes>>=10,tunit++) ;
  printf("Using %d%cB bucket memory at %lx,\n", sbytes, " KMGT"[sunit], (u64)ctx.trimmer->buckets);
  printf("%dx%d%cB thread memory at %lx,\n", nthreads, tbytes, " KMGT"[tunit], (u64)ctx.trimmer->tbuckets);
  printf("%d-way siphash, and %d buckets.\n", NSIPHASH, NX);

  thread_ctx *threads = (thread_ctx *)calloc(nthreads, sizeof(thread_ctx));
  assert(threads);

  u32 sumnsols = 0;
  for (u32 r = 0; r < range; r++) {
    gettimeofday(&time0, 0);
    //ctx.setheadernonce(header, sizeof(header), nonce + r);
    ctx.setheadergrin(header_data, header_length);
    printf("k0 k1 %lx %lx\n", ctx.trimmer->sip_keys.k0, ctx.trimmer->sip_keys.k1);
    u32 nsols = ctx.solve();
    gettimeofday(&time1, 0);
    timems = (time1.tv_sec-time0.tv_sec)*1000 + (time1.tv_usec-time0.tv_usec)/1000;
    printf("Time: %d ms\n", timems);

    for (unsigned s = 0; s < ctx.nsols; s++) {
      printf("Solution");
      for (u32 i = 0; i < PROOFSIZE; i++) {
        printf(" %jx", (uintmax_t)ctx.sols[s][i]);
        sol_nonces[i] = ctx.sols[s][i];
      }
      return 1;
      printf("\n");
    }
    sumnsols += nsols;
  }
  printf("%d total solutions\n", sumnsols);
  return 0;
}

extern "C" void cuckoo_description(char * name_buf,
                              int* name_buf_len,
                              char *description_buf,
                              int* description_buf_len){

  int ntrims   = 0;
  
  //TODO: check we don't exceed lengths.. just keep it under 256 for now
  int name_buf_len_in = *name_buf_len;
  const char* name = "cuckoo_mean_%d\0";
  sprintf(name_buf, name, EDGEBITS+1);
  *name_buf_len = strlen(name);
  
  const char* desc1 = "Looks for a %d-cycle on cuckoo%d with 50%% edges, using mean algorithm.\n\
  Uses %d-way siphash and %d buckets.\0";

  sprintf(description_buf, desc1,     
  PROOFSIZE, EDGEBITS+1, NSIPHASH, NBUCKETS);
  *description_buf_len = strlen(description_buf);
 
}

/// Return a simple json list of parameters

extern "C" int cuckoo_parameter_list(char *params_out_buf,
                                     int* params_len){
  get_properties_as_json(params_out_buf, params_len);
                                  
}

/// Return a simple json list of parameters

extern "C" int cuckoo_set_parameter(char *param_name,
                                     int param_name_len,
                                     int value){
  
  if (param_name_len > MAX_PROPERTY_NAME_LENGTH) return -1;
  char compare_buf[MAX_PROPERTY_NAME_LENGTH];
  snprintf(compare_buf,param_name_len+1,"%s", param_name);
  if (strcmp(compare_buf,"NUM_TRIMS")==0){
    if (value>=PROPS[0].min_value && value<=PROPS[0].max_value){
       NUM_TRIMS_PARAM=value;
       return PROPERTY_RETURN_OK;
    } else {
      return PROPERTY_RETURN_OUTSIDE_RANGE;
    }
  }
  if (strcmp(compare_buf,"NUM_THREADS")==0){
    if (value>=PROPS[1].min_value && value<=PROPS[1].max_value){
       NUM_THREADS_PARAM=value;
       return PROPERTY_RETURN_OK;
    } else {
      return PROPERTY_RETURN_OUTSIDE_RANGE;
    }
  }
  return PROPERTY_RETURN_NOT_FOUND;                                
}

extern "C" int cuckoo_get_parameter(char *param_name,
                                     int param_name_len,
                                     int* value){

}

/**
 * Initialises all parameters, defaults, and makes them available
 * to a caller
 */

extern "C" int cuckoo_init(){
  allocated_properties=0;
  PLUGIN_PROPERTY num_trims_prop;
  strcpy(num_trims_prop.name,"NUM_TRIMS\0");
  strcpy(num_trims_prop.description,"The maximum number of trim rounds to perform\0");
  num_trims_prop.default_value=1;
  num_trims_prop.min_value=5;
  num_trims_prop.max_value=100;
  add_plugin_property(num_trims_prop);

  NUM_TRIMS_PARAM = num_trims_prop.default_value;

  PLUGIN_PROPERTY num_threads_prop;
  strcpy(num_threads_prop.name,"NUM_THREADS\0");
  strcpy(num_threads_prop.description,"The number of threads to use\0");
  num_threads_prop.default_value=1;
  num_threads_prop.min_value=1;
  num_threads_prop.max_value=32;
  add_plugin_property(num_threads_prop);

  NUM_THREADS_PARAM = num_threads_prop.default_value;
}
