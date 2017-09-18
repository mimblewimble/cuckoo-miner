// Cuckoo Cycle, a memory-hard proof-of-work
// Copyright (c) 2013-2016 John Tromp

#include "cuckoo.h"
#include <stdio.h>
#include <stdlib.h>
#include <assert.h>
#include <unistd.h>
#include <set>

// assume EDGEBITS < 31
#define NNODES (2 * NEDGES)
#define MAXPATHLEN 8192


//Only going to allow one top-level worker thread here
//only one thread writing, should get away without mutex
bool is_working=false;

bool single_mode=true;

//TODO: Mutex
u32 hashes_processed_count=0;

class cuckoo_ctx {
public:
  siphash_keys sip_keys;
  edge_t easiness;
  node_t *cuckoo;

  cuckoo_ctx(const char* header, int header_len, edge_t easy_ness) {
    setheader(header, header_len, &sip_keys);
    easiness = easy_ness;
    cuckoo = (node_t *)calloc(1+NNODES, sizeof(node_t));
    assert(cuckoo != 0);
  }
  ~cuckoo_ctx() {
    free(cuckoo);
  }
};

int path(node_t *cuckoo, node_t u, node_t *us) {
  int nu;
  for (nu = 0; u; u = cuckoo[u]) {
    if (++nu >= MAXPATHLEN) {
      while (nu-- && us[nu] != u) ;
      if (nu < 0)
        printf("maximum path length exceeded\n");
      else printf("illegal % 4d-cycle\n", MAXPATHLEN-nu);
      exit(0);
    }
    us[nu] = u;
  }
  return nu;
}

typedef std::pair<node_t,node_t> edge;

void solution(cuckoo_ctx *ctx, node_t *us, int nu, node_t *vs, int nv, u32* sol_nonces) {
  std::set<edge> cycle;
  unsigned n;
  cycle.insert(edge(*us, *vs));
  while (nu--)
    cycle.insert(edge(us[(nu+1)&~1], us[nu|1])); // u's in even position; v's in odd
  while (nv--)
    cycle.insert(edge(vs[nv|1], vs[(nv+1)&~1])); // u's in odd position; v's in even
  printf("Solution");
  int sol_nonce_index=0;
  for (edge_t nonce = n = 0; nonce < ctx->easiness; nonce++) {
    edge e(sipnode(&ctx->sip_keys, nonce, 0), sipnode(&ctx->sip_keys, nonce, 1));
    if (cycle.find(e) != cycle.end()) {
      printf(" %x", nonce);
      sol_nonces[sol_nonce_index]=nonce;
      sol_nonce_index++;
      cycle.erase(e);
    }
  }
  printf("\n");
}

int worker(cuckoo_ctx *ctx, u32* sol_nonces) {
  node_t *cuckoo = ctx->cuckoo;
  node_t us[MAXPATHLEN], vs[MAXPATHLEN];
  for (node_t nonce = 0; nonce < ctx->easiness; nonce++) {
    //just temporary, till I get a better sense where to put this
    if(!single_mode && should_quit){
      return 0;
    }
    node_t u0 = sipnode(&ctx->sip_keys, nonce, 0);
    if (u0 == 0) continue; // reserve 0 as nil; v0 guaranteed non-zero
    node_t v0 = sipnode(&ctx->sip_keys, nonce, 1);
    node_t u = cuckoo[u0], v = cuckoo[v0];
    us[0] = u0;
    vs[0] = v0;
#ifdef SHOW
    for (unsigned j=1; j<NNODES; j++)
      if (!cuckoo[j]) printf("%2d:   ",j);
      else           printf("%2d:%02d ",j,cuckoo[j]);
    printf(" %x (%d,%d)\n", nonce,*us,*vs);
#endif
    int nu = path(cuckoo, u, us), nv = path(cuckoo, v, vs);
    if (us[nu] == vs[nv]) {
      int min = nu < nv ? nu : nv;
      for (nu -= min, nv -= min; us[nu] != vs[nv]; nu++, nv++) ;
      int len = nu + nv + 1;
      printf("% 4d-cycle found at %d%%\n", len, (int)(nonce*100L/ctx->easiness));
      if (len == PROOFSIZE) {
        solution(ctx, us, nu, vs, nv, sol_nonces);
        hashes_processed_count++;
        return 1;
      }
      continue;
    }
    if (nu < nv) {
      while (nu--)
        cuckoo[us[nu+1]] = us[nu];
      cuckoo[u0] = v0;
    } else {
      while (nv--)
        cuckoo[vs[nv+1]] = vs[nv];
      cuckoo[v0] = u0;
    }
  }
  hashes_processed_count++;
  return 0;
}

extern "C" int cuckoo_call(char* header_data, 
                           int header_length,
                           u32* sol_nonces){

  int c, easipct = 50;

  assert(easipct >= 0 && easipct <= 100);
  printf("Looking for %d-cycle on cuckoo%d(\"%s\") with %d%% edges\n",
               PROOFSIZE, EDGEBITS+1, header_data, easipct);
  u64 easiness = easipct * (u64) NNODES / 100;
  cuckoo_ctx ctx(header_data, header_length, easiness);
  return worker(&ctx, sol_nonces);
}

/**
 * Initialises all parameters, defaults, and makes them available
 * to a caller
 */

extern "C" int cuckoo_init(){
  allocated_properties=0;
  return PROPERTY_RETURN_OK;
}

extern "C" void cuckoo_description(char * name_buf,
                              int* name_buf_len,
                              char *description_buf,
                              int* description_buf_len){

  //TODO: check we don't exceed lengths.. just keep it under 256 for now
  int name_buf_len_in = *name_buf_len;
  const char* name = "cuckoo_simple_%d\0";
  sprintf(name_buf, name, EDGEBITS+1);
  *name_buf_len = strlen(name);
  
  const char* desc1 = "Looks for a %d-cycle on cuckoo%d with 50%% edges using simple algorithm\0";

  sprintf(description_buf, desc1, PROOFSIZE, EDGEBITS+1);
  *description_buf_len = strlen(description_buf);

}

/// Return a simple json list of parameters

extern "C" int cuckoo_parameter_list(char *params_out_buf,
                                     int*  params_len){
    return get_properties_as_json(params_out_buf, params_len);
}

extern "C" int cuckoo_set_parameter(char *param_name,
                                     int param_name_len,
                                     int value){
  
  return PROPERTY_RETURN_NOT_FOUND;                                
}

extern "C" int cuckoo_get_parameter(char *param_name,
                                     int param_name_len,
                                     int* value){
  return PROPERTY_RETURN_OK;
}

extern "C" u32 cuckoo_hashes_since_last_call(){
    u32 return_val=hashes_processed_count;
    hashes_processed_count=0;
    return return_val;
}

bool cuckoo_internal_ready_for_hash(){
  return !is_working;
}

struct InternalWorkerArgs {
  unsigned char hash[32];
  unsigned char nonce[8];
};

void *process_internal_worker (void *vp) {
  single_mode=false;
  InternalWorkerArgs* args = (InternalWorkerArgs*) vp;
  int c, easipct = 50;

  assert(easipct >= 0 && easipct <= 100);
  u64 easiness = easipct * NNODES / 100;
  cuckoo_ctx ctx((const char*) args->hash, sizeof(args->hash),easiness);
  u32 response[PROOFSIZE];
  int return_val = worker(&ctx, response);
  if (return_val==1){
    QueueOutput output;
    memcpy(output.result_nonces, response, sizeof(output.result_nonces));
    memcpy(output.nonce, args->nonce, sizeof(output.nonce));
    //std::cout<<"Adding to queue "<<output.nonce<<std::endl;
    OUTPUT_QUEUE.enqueue(output);  
  }
  is_working=false;
  internal_processing_finished=true;
}

int cuckoo_internal_process_hash(unsigned char* hash, int hash_length, unsigned char* nonce){
  InternalWorkerArgs args;
  memcpy(args.hash, hash, sizeof(args.hash));
  memcpy(args.nonce, nonce, sizeof(args.nonce));
  pthread_t internal_worker_thread;
  is_working=true;
    if (!pthread_create(&internal_worker_thread, NULL, process_internal_worker, &args)){
        //NB make sure more jobs are being blocked before calling detached,
        //or you end up in a race condition and the same hash is submit many times
 
        if (pthread_detach(internal_worker_thread)){
            return 1;
        } 
        
    }
}

/*
 * returns current stats for all working devices
 */

extern "C" int cuckoo_get_stats(char* prop_string, int* length){
	sprintf(prop_string, "[]\0");
	*length=3;
	return PROPERTY_RETURN_OK;
}




