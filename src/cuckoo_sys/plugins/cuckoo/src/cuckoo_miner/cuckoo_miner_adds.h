// Copyright 2017 The Grin Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#ifndef CUCKOO_MINER_ADDS_H
#define CUCKOO_MINER_ADDS_H

#include "cuckoo_miner.h"

int NUM_THREADS_PARAM=1;
int NUM_TRIMS_PARAM=1 + (PART_BITS+3)*(PART_BITS+4)/2;

pthread_mutex_t device_info_mutex = PTHREAD_MUTEX_INITIALIZER;
deviceInfo DEVICE_INFO;

//forward dec
extern "C" int cuckoo_call(char* header_data, 
                           int header_length,
                           u32* sol_nonces);
void populate_device_info();

/**
 * Initialises all parameters, defaults, and makes them available
 * to a caller
 */

extern "C" int cuckoo_init(){
  allocated_properties=0;
  PLUGIN_PROPERTY num_trims_prop;
  strcpy(num_trims_prop.name,"NUM_TRIMS\0");
  strcpy(num_trims_prop.description,"The maximum number of trim rounds to perform\0");
  num_trims_prop.default_value=1 + (PART_BITS+3)*(PART_BITS+4)/2;
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

  populate_device_info();

  NUM_THREADS_PARAM = num_threads_prop.default_value;
  return PROPERTY_RETURN_OK;
}

/**
 * Returns a description
 */

extern "C" void cuckoo_description(char * name_buf,
                              int* name_buf_len,
                              char *description_buf,
                              int* description_buf_len){

  //TODO: check we don't exceed lengths.. just keep it under 256 for now
	int REQUIRED_SIZE=256;
	if (*name_buf_len < REQUIRED_SIZE || *description_buf_len < REQUIRED_SIZE){
		*name_buf_len=0;
		*description_buf_len=0;
		return;
	}
  int name_buf_len_in = *name_buf_len;
  const char* name = "cuckoo_lean_cpu_%d\0";
  sprintf(name_buf, name, EDGEBITS+1);
  *name_buf_len = strlen(name);

  const char* desc1 = "Looks for a %d-cycle on cuckoo%d with 50%% edges using lean CPU algorithm.\n \
  Uses %d%cB edge and %d%cB node memory, %d-way siphash, and %d-byte counters.\0";

  u64 edgeBytes = NEDGES/8, nodeBytes = TWICE_ATOMS*sizeof(atwice);
  int edgeUnit, nodeUnit;
  for (edgeUnit=0; edgeBytes >= 1024; edgeBytes>>=10,edgeUnit++) ;
  for (nodeUnit=0; nodeBytes >= 1024; nodeBytes>>=10,nodeUnit++) ;
  sprintf(description_buf, desc1,     
  PROOFSIZE, EDGEBITS+1, (int)edgeBytes, " KMGT"[edgeUnit], (int)nodeBytes, " KMGT"[nodeUnit], NSIPHASH, SIZEOF_TWICE_ATOM);
  *description_buf_len = strlen(description_buf);
}

/// Return a simple json list of parameters

extern "C" int cuckoo_parameter_list(char *params_out_buf,
                                     int* params_len){
  return get_properties_as_json(params_out_buf, params_len);
                                  
}

/// 

extern "C" int cuckoo_set_parameter(char *param_name,
                                     int param_name_len,
                                     int value){

  if (param_name_len > MAX_PROPERTY_NAME_LENGTH) return PROPERTY_RETURN_TOO_LONG;
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
  if (param_name_len > MAX_PROPERTY_NAME_LENGTH) return PROPERTY_RETURN_TOO_LONG;
  char compare_buf[MAX_PROPERTY_NAME_LENGTH];
  snprintf(compare_buf,param_name_len+1,"%s", param_name);
  if (strcmp(compare_buf,"NUM_TRIMS")==0){
       *value = NUM_TRIMS_PARAM;
       return PROPERTY_RETURN_OK;
  }
  if (strcmp(compare_buf,"NUM_THREADS")==0){
       *value = NUM_THREADS_PARAM;
       return PROPERTY_RETURN_OK;
  }
  return PROPERTY_RETURN_NOT_FOUND;
}

extern "C" int cuckoo_can_accept_job(){
  return 1;
}

bool cuckoo_internal_ready_for_hash(){
  return !is_working;
}

struct InternalWorkerArgs {
  char hash[32];
  unsigned char nonce[8];
};

void update_stats(u64 start_time) {
  pthread_mutex_lock (&device_info_mutex);
  DEVICE_INFO.last_start_time=start_time;
  DEVICE_INFO.last_end_time=timestamp();
  DEVICE_INFO.last_solution_time=DEVICE_INFO.last_end_time-
  DEVICE_INFO.last_start_time; 
  DEVICE_INFO.is_busy=false;
  DEVICE_INFO.iterations_completed++;
  pthread_mutex_unlock (&device_info_mutex);
}

void *process_internal_worker (void *vp) {
  InternalWorkerArgs* args = (InternalWorkerArgs*) vp;
  u32 response[PROOFSIZE];

  u64 start_time=timestamp();
  int return_val=cuckoo_call(args->hash, sizeof(args->hash), response);

  if (return_val==1){
    QueueOutput output;
    memcpy(output.result_nonces, response, sizeof(output.result_nonces));
    memcpy(output.nonce, args->nonce, sizeof(output.nonce));
    //std::cout<<"Adding to queue "<<output.nonce<<std::endl;
    OUTPUT_QUEUE.enqueue(output);  
  }
  update_stats(start_time);
  is_working=false;
  internal_processing_finished=true;
  pthread_exit(NULL);
}

int cuckoo_internal_process_hash(unsigned char* hash, int hash_length, unsigned char* nonce){
  InternalWorkerArgs args;
  memcpy(args.hash, hash, sizeof(args.hash));
  memcpy(args.nonce, nonce, sizeof(args.nonce));
  pthread_t internal_worker_thread;
  is_working=true;
  if (should_quit) return 1;
  pthread_mutex_lock (&device_info_mutex);
  DEVICE_INFO.is_busy=true;
	pthread_mutex_unlock(&device_info_mutex);
  internal_processing_finished=false;
      if (!pthread_create(&internal_worker_thread, NULL, process_internal_worker, &args)){
        //NB make sure more jobs are being blocked before calling detached,
        //or you end up in a race condition and the same hash is submit many times
 
        if (pthread_detach(internal_worker_thread)){
            return 1;
        }
    }
   return 0;
}

void populate_device_info(){
    strcpy(DEVICE_INFO.device_name,"CPU\0");
}

/*
 * returns current stats for all working devices
 */

extern "C" int cuckoo_get_stats(char* prop_string, int* length){
    int remaining=*length;
    const char* device_stat_json = "{\"device_id\":\"%d\",\"device_name\":\"%s\",\"last_start_time\":%lld,\"last_end_time\":%lld,\"last_solution_time\":%d,\"iterations_completed\":%lld}";
    //minimum return is "[]\0"
    if (remaining<=3){
        return PROPERTY_RETURN_BUFFER_TOO_SMALL;
    }
    prop_string[0]='[';
    int last_write_pos=1;
    pthread_mutex_lock (&device_info_mutex);
    int last_written=snprintf(prop_string+last_write_pos,
                          remaining, 
                          device_stat_json, DEVICE_INFO.device_id, 
                          DEVICE_INFO.device_name, DEVICE_INFO.last_start_time,
                          DEVICE_INFO.last_end_time, DEVICE_INFO.last_solution_time,
     DEVICE_INFO.iterations_completed);
    pthread_mutex_unlock(&device_info_mutex);
    remaining-=last_written;
    last_write_pos+=last_written;
    //no room for anything else, comma or trailing ']'
	 
    //write final characters
    if (remaining<2){
        return PROPERTY_RETURN_BUFFER_TOO_SMALL;
    }
    //overwrite trailing \0
    prop_string[last_write_pos]=']';
    prop_string[last_write_pos+1]='\0';
    remaining -=2;
    *length=last_write_pos+1;
    
    //empty set
    if (*length==3){
        *length=2;
    }
    return PROPERTY_RETURN_OK;
		return 0;
}

/*void stop_processing_internal(){

}*/
#endif
