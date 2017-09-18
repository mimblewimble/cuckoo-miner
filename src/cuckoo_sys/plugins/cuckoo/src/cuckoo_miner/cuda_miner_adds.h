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

// Functions specific to cuda implementation
#ifndef CUDA_MINER_ADDS_H
#define CUDA_MINER_ADDS_H

#include "cuckoo_miner.h"

//forward dec
extern "C" int cuckoo_call(char* header_data, 
                           int header_length,
                           u32* sol_nonces);

int NUM_THREADS_PARAM=16384;
int NUM_TRIMS_PARAM=32;

pthread_mutex_t device_info_mutex = PTHREAD_MUTEX_INITIALIZER;

#define MAX_DEVICES 32
u32 NUM_DEVICES=0;

typedef class cudaDeviceInfo {
  public:
    int device_id;
    bool is_busy;
    cudaDeviceProp properties;
    //store the current hash rate
    u64 last_start_time;
    u64 last_end_time; 
    u64 last_solution_time;
    u32 iterations_completed;

    cudaDeviceInfo();
} CudaDeviceInfo;

cudaDeviceInfo::cudaDeviceInfo(){
    device_id=-1;
    is_busy=false;
    last_start_time=0;
    last_end_time=0;
    last_solution_time=0;
    iterations_completed=0;
}

CudaDeviceInfo DEVICE_INFO[MAX_DEVICES];

void populate_device_info(){
  int nDevices;

  cudaGetDeviceCount(&nDevices);
  for (int i = 0; i < nDevices; i++) {
    DEVICE_INFO[i].device_id=i;
    cudaGetDeviceProperties(&DEVICE_INFO[i].properties, i);
    printf("Device Number: %d\n", i);
    printf("  Device name: %s\n", DEVICE_INFO[i].properties.name);
    printf("  Memory Clock Rate (KHz): %d\n",
           DEVICE_INFO[i].properties.memoryClockRate);
    printf("  Memory Bus Width (bits): %d\n",
           DEVICE_INFO[i].properties.memoryBusWidth);
    printf("  Peak Memory Bandwidth (GB/s): %f\n\n",
           2.0*DEVICE_INFO[i].properties.memoryClockRate*(DEVICE_INFO[i].properties.memoryBusWidth/8)/1.0e6);
  }
  NUM_DEVICES=nDevices;
}

/*
 * returns current stats for all working devices
 */

extern "C" int cuckoo_get_stats(char* prop_string, int* length){
    int remaining=*length;
    const char* device_stat_json = "{\"device_id\":\"%d\",\"device_name\":\"%s\",\"last_start_time\":%lld,\"last_end_time\":%lld,\"last_solution_time\":%d,\"iterations_completed\":%lld}";
    //minimum return is "[]\0"
    if (remaining<=3){
        //TODO: Meaningful return code
        return PROPERTY_RETURN_BUFFER_TOO_SMALL;
    }
    prop_string[0]='[';
    int last_write_pos=1;
    for (int i=0;i<NUM_DEVICES;i++){
        int last_written=snprintf(prop_string+last_write_pos, 
                              remaining, 
                              device_stat_json, DEVICE_INFO[i].device_id, 
                              DEVICE_INFO[i].properties.name, DEVICE_INFO[i].last_start_time,
                              DEVICE_INFO[i].last_end_time, DEVICE_INFO[i].last_solution_time,
			      DEVICE_INFO[i].iterations_completed);
        remaining-=last_written;
        last_write_pos+=last_written;
        //no room for anything else, comma or trailing ']'
        if (remaining<2){
            //TODO: meaningful error code
            return PROPERTY_RETURN_BUFFER_TOO_SMALL;
        }
        //write comma
        if (i<NUM_DEVICES-1){
            //overwrite trailing \0 in this case
            prop_string[last_write_pos++]=',';
        } 
    }
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
}

u32 next_free_device_id(){
    for (int i=0;i<NUM_DEVICES;i++){
    	if (!DEVICE_INFO[i].is_busy){
	    return i;
	}
    }
    return -1;
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
  num_trims_prop.default_value=32;
  num_trims_prop.min_value=5;
  num_trims_prop.max_value=100;
  add_plugin_property(num_trims_prop);

  NUM_TRIMS_PARAM = num_trims_prop.default_value;

  PLUGIN_PROPERTY num_threads_prop;
  strcpy(num_threads_prop.name,"NUM_THREADS\0");
  strcpy(num_threads_prop.description,"The number of threads to use\0");
  num_threads_prop.default_value=16384;
  num_threads_prop.min_value=1;
  num_threads_prop.max_value=65535;
  add_plugin_property(num_threads_prop);

  NUM_THREADS_PARAM = num_threads_prop.default_value;

  populate_device_info();
  return PROPERTY_RETURN_OK;
}

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

  const char* name = "cuckoo_lean_cuda_%d\0";
  sprintf(name_buf, name, EDGEBITS+1);
  *name_buf_len = strlen(name);
  
  const char* desc1 = "Looks for %d-cycle on cuckoo_%d using nVidia CUDA miner\0";
  
  sprintf(description_buf, desc1, PROOFSIZE, EDGEBITS+1);
  *description_buf_len = strlen(description_buf);
 
}

/// Return a simple json list of parameters

extern "C" int cuckoo_parameter_list(char *params_out_buf,
                                     int* params_len){
  return get_properties_as_json(params_out_buf, params_len);
  
                                  
}

/// Return a simple json list of parameters

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

bool cuckoo_internal_ready_for_hash(){
  //just return okay if a device is flagged as free
  for (int i=0;i<NUM_DEVICES;i++){
    if (!DEVICE_INFO[i].is_busy){
       return true;
    }
  }
  return false;
}

struct InternalWorkerArgs {
  unsigned char hash[32];
  unsigned char nonce[8];
  u32 device_id;
};

void update_stats(u32 device_id, u64 start_time) {
  pthread_mutex_lock (&device_info_mutex);
  DEVICE_INFO[device_id].last_start_time=start_time;
  DEVICE_INFO[device_id].last_end_time=timestamp();
  DEVICE_INFO[device_id].last_solution_time=DEVICE_INFO[device_id].last_end_time-
  DEVICE_INFO[device_id].last_start_time; 
  DEVICE_INFO[device_id].is_busy=false;
  DEVICE_INFO[device_id].iterations_completed++;
  pthread_mutex_unlock(&device_info_mutex);
}

void *process_internal_worker (void *vp) {
  InternalWorkerArgs* args = (InternalWorkerArgs*) vp;

  //this should set the device for this thread
  cudaSetDevice(args->device_id);
 
  u32 response[PROOFSIZE];
  u64 start_time=timestamp();

  int return_val=cuckoo_call((char*) args->hash, sizeof(args->hash), response);

  if (return_val==1){
    QueueOutput output;
    memcpy(output.result_nonces, response, sizeof(output.result_nonces));
    memcpy(output.nonce, args->nonce, sizeof(output.nonce));
    //std::cout<<"Adding to queue "<<output.nonce<<std::endl;
    OUTPUT_QUEUE.enqueue(output);
  }
  update_stats(args->device_id, start_time);
  delete(args);
  internal_processing_finished=true;
}

int cuckoo_internal_process_hash(unsigned char* hash, int hash_length, unsigned char* nonce){
    //Not entirely sure... this should select a free device, then send it to the next available
    InternalWorkerArgs* args=new InternalWorkerArgs();
    memcpy(args->hash, hash, sizeof(args->hash));
    memcpy(args->nonce, nonce, sizeof(args->nonce));
    u32 device_id=next_free_device_id();
    args->device_id=device_id;
    pthread_mutex_lock(&device_info_mutex);
    DEVICE_INFO[device_id].is_busy=true;
    DEVICE_INFO[device_id].device_id=device_id;
    pthread_mutex_unlock(&device_info_mutex);
    pthread_t internal_worker_thread;
    is_working=true;
    if (should_quit) return 1;
    internal_processing_finished=false;
    if (!pthread_create(&internal_worker_thread, NULL, process_internal_worker, args)){
        if (pthread_detach(internal_worker_thread)){
            return 1;
        }
    }
    return 0;
}
#endif
