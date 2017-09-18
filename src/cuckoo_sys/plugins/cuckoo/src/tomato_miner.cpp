// Time Memory Trade Off (TMTO, or tomato) solver

#include "tomato_miner.h"
#include <unistd.h>

int NUM_PARTS_PARAM = NUPARTS;
int MINIMAL_BFS_PARAM = 0;
int NUM_THREADS_PARAM = 1;

extern "C" int cuckoo_call(char* header_data, 
                           int header_length, 
                           u32* sol_nonces){
  int nthreads = NUM_THREADS_PARAM;
  bool minimalbfs = MINIMAL_BFS_PARAM;
  int nparts = NUM_PARTS_PARAM;
  int c;
  int nonce = 0;
  int range = 1;

  /*while ((c = getopt (argc, argv, "h:n:t:r:m")) != -1) {
    switch (c) {
      case 'h':
        len = strlen(optarg);
        assert(len <= sizeof(header));
        memcpy(header, optarg, len);
        break;
      case 'm':
        minimalbfs = true;
        break;
      case 'n':
        nonce = atoi(optarg);
        break;
      case 'p':
        nparts = atoi(optarg);
        break;
      case 'r':
        range = atoi(optarg);
        break;
      case 't':
        nthreads = atoi(optarg);
        break;
    }
  }*/
  printf("Looking for %d-cycle on cuckoo%d(\"%s\",%d", PROOFSIZE, NODEBITS, header_data, nonce);
  if (range > 1)
    printf("-%d", nonce+range-1);
  printf(") with 50%% edges, 1/%d memory, %d/%d parts, %d threads %d minimalbfs\n",
    1<<SAVEMEM_BITS, nparts, NUPARTS, nthreads, minimalbfs);
  u64 nodeBytes = CUCKOO_SIZE*sizeof(u64);
  int nodeUnit;
  for (nodeUnit=0; nodeBytes >= 1024; nodeBytes>>=10,nodeUnit++) ;
  printf("Using %d%cB node memory.\n", (int)nodeBytes, " KMGT"[nodeUnit]);
  thread_ctx *threads = (thread_ctx *)calloc(nthreads, sizeof(thread_ctx));
  assert(threads);
  cuckoo_ctx ctx(nthreads, nparts, minimalbfs);
  
  worker_args* wa = (worker_args*) calloc(1, sizeof(worker_args));
  wa->solution_found=false;
  for (int r = 0; r < range; r++) {
    //ctx.setheadernonce(header, sizeof(header), nonce + r);
    ctx.setheadergrin(header_data, header_length);

    for (int t = 0; t < nthreads; t++) {
      threads[t].id = t;
      threads[t].ctx = &ctx;
      
      wa->tp = &threads[t];
      wa->sol_nonces=sol_nonces;
      
      int err = pthread_create(&threads[t].thread, NULL, worker, (void*) wa);
      assert(err == 0);
    }
    for (int t = 0; t < nthreads; t++) {
      int err = pthread_join(threads[t].thread, NULL);
      assert(err == 0);
    }
  }
  int solution_found=wa->solution_found;
  printf("Solution found: %d\n",wa->solution_found);
  printf("Freeing\n");
  free(wa);
  free(threads);
  return solution_found;
}

/**
 * Returns a description
 */

extern "C" void cuckoo_description(char * name_buf,
                              int* name_buf_len,
                              char *description_buf,
                              int* description_buf_len){
  
  //TODO: check we don't exceed lengths.. just keep it under 256 for now
  int name_buf_len_in = *name_buf_len;
  const char* name = "cuckoo_tomato_%d\0";
  sprintf(name_buf, name, EDGEBITS+1);
  *name_buf_len = strlen(name);
  
  const char* desc1 = "Looks for a %d-cycle on cuckoo%d with 50%% edges using Time-Memory Tradeoff algorithm.\n";

  sprintf(description_buf, desc1,     
  PROOFSIZE, EDGEBITS+1);
  *description_buf_len = strlen(description_buf);
 
}

/// Return a simple json list of parameters

extern "C" int cuckoo_parameter_list(char *params_out_buf,
                                     int* params_len){
  return get_properties_as_json(params_out_buf, params_len);
                                  
}

extern "C" int cuckoo_init(){
  allocated_properties=0;
  PLUGIN_PROPERTY num_parts_prop;
  strcpy(num_parts_prop.name,"NUM_PARTS\0");
  strcpy(num_parts_prop.description,"The number pf parts\0");
  num_parts_prop.default_value=NUPARTS;
  num_parts_prop.min_value=5;
  num_parts_prop.max_value=100;
  add_plugin_property(num_parts_prop);

  NUM_PARTS_PARAM = num_parts_prop.default_value;

  PLUGIN_PROPERTY num_threads_prop;
  strcpy(num_threads_prop.name,"NUM_THREADS\0");
  strcpy(num_threads_prop.description,"The number of threads to use\0");
  num_threads_prop.default_value=1;
  num_threads_prop.min_value=1;
  num_threads_prop.max_value=32;
  add_plugin_property(num_threads_prop);

  NUM_THREADS_PARAM = num_threads_prop.default_value;

  PLUGIN_PROPERTY minimal_bfs_prop;
  strcpy(minimal_bfs_prop.name,"MINIMAL_BFS\0");
  strcpy(minimal_bfs_prop.description,"Minimal BFS (bool)\0");
  minimal_bfs_prop.default_value=0;
  minimal_bfs_prop.min_value=0;
  minimal_bfs_prop.max_value=1;
  add_plugin_property(minimal_bfs_prop);

  MINIMAL_BFS_PARAM = minimal_bfs_prop.default_value;
  return PROPERTY_RETURN_OK;
}

/// Return a simple json list of parameters

extern "C" int cuckoo_set_parameter(char *param_name,
                                     int param_name_len,
                                     int value){
  
  if (param_name_len > MAX_PROPERTY_NAME_LENGTH) return -1;
  char compare_buf[MAX_PROPERTY_NAME_LENGTH];
  snprintf(compare_buf,param_name_len+1,"%s", param_name);
  if (strcmp(compare_buf,"NUM_PARTS")==0){
    if (value>=PROPS[0].min_value && value<=PROPS[0].max_value){
       NUM_PARTS_PARAM=value;
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
  if (strcmp(compare_buf,"MINIMAL_BFS")==0){
    if (value>=PROPS[2].min_value && value<=PROPS[2].max_value){
       MINIMAL_BFS_PARAM=value;
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
  return PROPERTY_RETURN_OK;
}

extern "C" int cuckoo_can_accept_job(){
  return 1;
}

extern "C" u32 cuckoo_hashes_since_last_call(){
    return 0;
}

bool cuckoo_internal_ready_for_hash(){
  return false;
}

int cuckoo_internal_process_hash(unsigned char* hash, int hash_length, unsigned char* nonce){
  
}

/*
 * returns current stats for all working devices
 */

extern "C" int cuckoo_get_stats(char* prop_string, int* length){
	sprintf(prop_string, "[]\0");
	*length=3;
	return PROPERTY_RETURN_OK;
}


