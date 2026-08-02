#include "uoribc.h"
#include <stddef.h>
#include <stdbool.h>
#include <string.h>
#include <stdatomic.h>


// The following are macros used by UORIBC:

#define INTERFACE_REGISTRY_SIZE 4U

#define DISPATCH_TABLE_SIZE 64U

#define STREAM_REGISTRY_SIZE 2U
#define STREAM_BUFFER_SIZE 255U
#define STREAM_ID_MAX 15U
#define STREAM_REGISTRY_MASK_WIDTH 8U
#define STREAM_SEARCH_MASK 0b00000001U
#define STREAM_MAX_PAYLOAD_SIZE 8U

#define MESSAGE_ID_MASK 0b01111111U



#if STREAM_REGISTRY_SIZE > STREAM_REGISTRY_MASK_WIDTH
 
#undef  STREAM_REGISTRY_SIZE
#define STREAM_REGISTRY_SIZE STREAM_REGISTRY_MASK_WIDTH

#endif 

// The following are structs used by UORIBC:

typedef struct {
  uoribc_interface_error_t      (*transmit_function)(uoribc_message_context_t);
  uoribc_interface_id_t         interface_id;
} interface_t; 


typedef struct {
  void                          (*callback_function)(uoribc_message_context_t context);
  uoribc_interface_id_t         interface_id;
  uint8_t                       class_id;
  uint8_t                       message_id;
} callback_entry_t;

typedef struct {
  void                          (*stream_full_callback)(uint8_t stream_id);
  uoribc_stream_dir_t           stream_dir;
  uoribc_interface_id_t         interface_id;
  uint8_t                       stream_size;
  uint8_t                       stream_id;
  bool                          stream_awaiting_closure;
} stream_t;


// The following are buffers, registrys, and other statics used by UORIBC:

static uint8_t                  device_node_id = 0U;

static interface_t              interface_registry[INTERFACE_REGISTRY_SIZE];
static uint8_t                  interface_registry_index = 0U;

static callback_entry_t         callback_dispatch_table[DISPATCH_TABLE_SIZE];
static uint8_t                  callback_table_index = 0U; 

static stream_t                 stream_registry[STREAM_REGISTRY_SIZE];
static uint8_t                  stream_registry_mask = 0b00000000U;

static uint8_t                  stream_buffers[STREAM_REGISTRY_SIZE][STREAM_BUFFER_SIZE];
static uint8_t                  stream_buffer_indexes[STREAM_REGISTRY_SIZE] = {0U};

static atomic_bool              stream_rw_locked = false;
static atomic_uint_fast8_t      streams_in_use = 0;

// TODO: Although this is flaged as atomic, current usages are not atomic. Update usages to be atomic
static atomic_bool     device_initialized = false;

// The following are weakly implemented private functions:

__weak void uoribc_recieve_callback(uoribc_message_context_t context) {
  // DO NOT IMPLEMENT HERE. REDEFINE THE FUNCTION ELSEWHERE, PRESERVING THIS ONE.
};

// The following are implemented private functions:

static uoribc_interface_id_t u8_to_interface_id(uint8_t bit_value) {
  
  uoribc_interface_id_t return_value;
  
  switch(bit_value) {
  case 0b00000001U: 
    return_value = UORIBC_INTERFACE_0;
    break;
  case 0b00000010U:
    return_value = UORIBC_INTERFACE_1;
    break;
  case 0b00000100U: 
    return_value = UORIBC_INTERFACE_2;
    break;
  case 0b00001000U: 
    return_value = UORIBC_INTERFACE_3;
    break;
  default: 
    return_value = UORIBC_INTERFACE_0;
    break;
  }
  
  return return_value;
}

static uint8_t interface_id_to_u8(uoribc_interface_id_t interface_id) {
  
  uint8_t return_value;
  
  switch(interface_id) {
  case UORIBC_INTERFACE_0: 
    return_value = 0b00000001U;
    break;
  case UORIBC_INTERFACE_1:
    return_value = 0b00000010U;
    break;
  case UORIBC_INTERFACE_2: 
    return_value = 0b00000100U;
    break;
  case UORIBC_INTERFACE_3: 
    return_value = 0b00001000U;
    break;
  default: 
    return_value = 0b00000001U;
    break;
  }
  
  return return_value;
}

// Returns true if stream was found and false if stream was not found
static bool find_stream(uint8_t stream_id, stream_t **stream, uint8_t *index) {
  
  // Scan stream registry, checking mask to see if index is valid
  for (uint8_t i = 0U; i < STREAM_REGISTRY_SIZE; i++) {
    if (((stream_registry_mask >> i & STREAM_SEARCH_MASK) == STREAM_SEARCH_MASK) &&
         (stream_registry[i].stream_id == stream_id)) {
     
      if (index != NULL) {
        *index = i;
      }
      
      if (stream != NULL) {
        *stream = &stream_registry[i];
      }
     
      return true;    
    }
  }
  return false;  
}

// The following are implemented public functions:

uoribc_general_error_t uoribc_initialize(uint8_t node_id){
  
  bool initialized = atomic_load(&device_initialized);
  
  if (initialized) {
    return UORIBC_ALREADY_INITIALIZED;
  }
  
  device_node_id = node_id;
  atomic_store(&device_initialized, true);
  
  return UORIBC_GEN_OK;
}

uoribc_interface_error_t uoribc_interface_register(uoribc_interface_error_t (*transmit_function)(uoribc_message_context_t), uoribc_interface_id_t* interface_id) {
  // TODO: Update function to check if interface function pointer is null
  bool initialized = atomic_load(&device_initialized);

  if (initialized) {
    return UORIBC_INTR_LOCKED;
  }
  
  if (interface_registry_index + 1U > INTERFACE_REGISTRY_SIZE) {
    return UORIBC_MAX_INTERFACES;
  }
  
  if (transmit_function == NULL){
    return UORIBC_INTR_PTR_NULL;
  }
  
  // Fill interface struct
  interface_t interface_struct = {0};
  interface_struct.interface_id = u8_to_interface_id(0b00000001U << interface_registry_index);
  interface_struct.transmit_function = transmit_function;
  
  if (interface_id != NULL) {
    *interface_id = interface_struct.interface_id;
  }
  
  // Update registy
  interface_registry[interface_registry_index] = interface_struct;
  interface_registry_index += 1U;
  
  return UORIBC_INTERFACE_OK;
}

uoribc_callback_error_t uoribc_callback_register(uoribc_interface_id_t interface_id, uint8_t class_id, uint8_t message_id, void (*callback_function)(uoribc_message_context_t)) {
  
  bool initialized = atomic_load(&device_initialized);
  
  if (initialized) {
    return UORIBC_CALLBACK_LOCKED;
  }
  
  if (callback_table_index + 1U > DISPATCH_TABLE_SIZE) {
    return UORIBC_MAX_CALLBACKS;
  }     

  if (callback_function == NULL) {
    return UORIBC_CALLBACK_INVALID;
  }
  
  // Fill callback struct
  callback_entry_t callback_entry = {0};
  callback_entry.callback_function = callback_function;
  callback_entry.interface_id = interface_id;
  callback_entry.class_id = class_id;
  callback_entry.message_id = message_id;
  
  // Update dispatch table
  callback_dispatch_table[callback_table_index] = callback_entry;
  callback_table_index += 1U;
  
  return UORIBC_CALLBACK_OK;
}

uoribc_dispatch_error_t uoribc_message_receive(uoribc_message_context_t context) {
  // Call general recieve callback
  uoribc_recieve_callback(context);

  bool initialized = atomic_load(&device_initialized);
  
  if (!initialized) {
    return UORIBC_DISP_LOCKED;
  }
  
  // Check if frame is of message type
  if (context.type == UORIBC_MESSAGE) {
    uint8_t class_id = context.identifier;
    uint8_t message_id = context.skeleton.frame.header & MESSAGE_ID_MASK;
    uint8_t interface_id_u8 = interface_id_to_u8(context.interface_id);
    
    for (uint16_t i = 0U; i < callback_table_index; i++) {
      callback_entry_t entry = callback_dispatch_table[i];
      
      // Check if class id matches
      if ((entry.class_id == class_id) && 
          (entry.message_id == message_id) &&
          (interface_id_to_u8(entry.interface_id) & interface_id_u8) == interface_id_u8) 
      {
        if (entry.callback_function == NULL) {
          return UORIBC_DISPATCH_INVALID_CALLBACK;
        }
            // Dispatch Callback
          entry.callback_function(context);
      }
    }
  }
  // TODO: The current implementation of recieving data on a stream allows for a node to hold the rw access of the registry hostage.
  //       This can occur if a node spams the stream with data, forcing the engine to constantly consume which never allows rw access to the registry.
  //       MUST FIX
  else if (context.type == UORIBC_STREAM) {
    uint8_t stream_id = context.identifier;
    uint8_t registry_index = 0U;
    
    bool locked = atomic_load(&stream_rw_locked);
    
    if (locked) {
      return UORIBC_DISP_LOCKED;
    }
    
    streams_in_use += 1U;

    
    for (uint8_t i = 0U; i < STREAM_REGISTRY_SIZE; i++) {
      // Check that stream is recieving from interface
      if (((stream_registry_mask >> i & STREAM_SEARCH_MASK) == STREAM_SEARCH_MASK) &&
           (stream_registry[i].stream_id == stream_id) &&
            // TODO: Expand these to improve quality of error reporting
           (stream_registry[i].interface_id == context.interface_id) &&
           (stream_registry[i].stream_dir == UORIBC_STREAM_DIR_RX))  {
        
        uint8_t num_to_write = context.skeleton_size;
        
        if (num_to_write > stream_registry[i].stream_size) {
          streams_in_use -= 1U;
          return UORIBC_DISPATCH_STREAM_FULL;
        }
        
        
        // Prevent buffer overflow
        if (stream_buffer_indexes[i] + num_to_write >= stream_registry[i].stream_size) {
          num_to_write = (uint8_t)((uint16_t)(stream_buffer_indexes[i] + num_to_write) - stream_registry[i].stream_size);
        } 
            
        memcpy(&stream_buffers[i][stream_buffer_indexes[i]], context.skeleton.stream.payload, num_to_write);
        stream_buffer_indexes[i] += num_to_write;
          
        if (stream_buffer_indexes[i] == stream_registry[i].stream_size) {
          stream_registry[i].stream_full_callback(stream_registry[i].stream_id);
        }

        break;
      }
    }    
    streams_in_use -= 1U;
  }
  return UORIBC_DISPATCH_OK;
}

uoribc_interface_error_t uoribc_message_transmit(uoribc_message_context_t context) {
  
  bool initialized = atomic_load(&device_initialized);
  
  if (!initialized) {
    return UORIBC_INTR_LOCKED;
  }
  
  uoribc_interface_id_t interface_id = context.interface_id;
  interface_t *interface_ptr = NULL;

  // Look for interface in registry
  for (uint8_t i = 0U; i < interface_registry_index; i++) {
    if (interface_registry[i].interface_id == interface_id) {
      interface_ptr = &interface_registry[i];
      break;
    }
  }
  
  if (interface_ptr == NULL) {
    return UORIBC_INTR_NOT_REG;
  }

  
  context.node_id = device_node_id;
  
  return interface_ptr->transmit_function(context);
}

uoribc_stream_error_t uoribc_stream_open(uoribc_interface_id_t interface_id, uint8_t stream_id, uoribc_stream_dir_t stream_dir, uint8_t stream_size, void (*stream_full_callback)(uint8_t stream_id)) {
  
  uint8_t free_index = 0U;
  bool free_index_found = false;
  bool locked = false;
  
  if (stream_id > STREAM_ID_MAX) {
    return UORIBC_STREAM_ID_INVALID;
  }
  
  uint8_t streams_tally = atomic_load(&streams_in_use);
  if (streams_tally > 0U) {
      return UORIBC_STREAM_LOCKED;
  }
 
  atomic_compare_exchange_strong(&stream_rw_locked, &locked, true);
  
  if (locked) {
    return UORIBC_STREAM_LOCKED;
  }
  
    // Look for a free index in stream registry
  for (uint8_t i = 0U; i < STREAM_REGISTRY_SIZE; i++) {
    if (((stream_registry_mask >> i) & STREAM_SEARCH_MASK) != STREAM_SEARCH_MASK) {
      free_index = i;
      free_index_found = true;
      break;
    }
  }
  
  if (free_index_found == false) {
    atomic_store(&stream_rw_locked, false);
    return UORIBC_MAX_STREAMS;
  }
  
  // Check for duplicate stream ids
  for (uint8_t i = 0U; i < STREAM_REGISTRY_SIZE; i++) {
    uint8_t slot_mask = STREAM_SEARCH_MASK << i;

    if ((stream_registry_mask & slot_mask) != 0U) {
        if (stream_registry[i].stream_id == stream_id) {
            atomic_store(&stream_rw_locked, false);
            return UORIBC_STREAM_EXISTS;
        }
    }
}
  
  // Register stream
  stream_t new_stream = {
    .stream_full_callback       = stream_full_callback,
    .interface_id               = interface_id,
    .stream_id                  = stream_id,
    .stream_dir                 = stream_dir,
    .stream_size                = stream_size,
    .stream_awaiting_closure    = false,
  };
  
  stream_registry[free_index] = new_stream;
  stream_registry_mask |= STREAM_SEARCH_MASK << free_index;
  
  atomic_store(&stream_rw_locked, false);
  return UORIBC_STREAM_OK;
}

uoribc_stream_error_t uoribc_stream_close(uint8_t stream_id) {

  uint8_t streams_tally = atomic_load(&streams_in_use);
  if (streams_tally > 0U) {
      return UORIBC_STREAM_LOCKED;
  }
  
  bool locked = false;
  atomic_compare_exchange_strong(&stream_rw_locked, &locked, true);
  
  
  if (locked) {
    return UORIBC_STREAM_LOCKED;
  }
  

  
  for (uint8_t i = 0; i < STREAM_REGISTRY_SIZE; i++) {
     if (((stream_registry_mask >> i) & STREAM_SEARCH_MASK) == STREAM_SEARCH_MASK) {
       if (stream_registry[i].stream_id == stream_id) {
         // Clear bit to indicate that slot is available
         stream_registry_mask ^= STREAM_SEARCH_MASK << i;
         stream_buffer_indexes[i] = 0;
         atomic_store(&stream_rw_locked, false);
         return UORIBC_STREAM_OK;
       }      
    }
  }
  atomic_store(&stream_rw_locked, false);
  return UORIBC_STREAM_ID_INVALID;
}

uoribc_stream_error_t uoribc_stream_read_bytes(uint8_t stream_id, uint8_t *buffer, uint8_t num_to_read, uint8_t *num_read_ptr) {
  uint8_t stream_index = 0;
  stream_t *stream_ptr = NULL;
  bool stream_found;

  bool locked = atomic_load(&stream_rw_locked);
  
  if (locked) {
    return UORIBC_STREAM_LOCKED;
  }
  
  streams_in_use += 1U;
  
  locked = atomic_load(&stream_rw_locked);
   if (locked) {
    streams_in_use -= 1U;
    return UORIBC_STREAM_LOCKED;
  }
  
  stream_found = find_stream(stream_id, &stream_ptr, &stream_index);
  
 
  if (stream_found == false) {
    streams_in_use -= 1U;
    return UORIBC_STREAM_ID_INVALID;
  }
            
  if (stream_ptr->stream_dir != UORIBC_STREAM_DIR_RX) {
    streams_in_use -= 1U;
    return UORIBC_STREAM_DIR_MISMATCH;
  }
  
  
  if (num_to_read > stream_buffer_indexes[stream_index]) {
  num_to_read = stream_buffer_indexes[stream_index];
  }
      
  memcpy(buffer, stream_buffers[stream_index], num_to_read);
  stream_buffer_indexes[stream_index] -= num_to_read;

  if (num_read_ptr != NULL) {
    *num_read_ptr = num_to_read;
  }
  
  streams_in_use -= 1U;
  return UORIBC_STREAM_OK;

}

uoribc_stream_error_t uoribc_stream_write_bytes(uint8_t stream_id, uint8_t *buffer, uint8_t num_to_write) {
  bool stream_found = false;
  stream_t *stream_ptr = NULL;
  
  if (num_to_write == 0U) {
    return UORIBC_STREAM_INVALID_SIZE;
  }

  bool locked = atomic_load(&stream_rw_locked);
  
  if (locked) {
    return UORIBC_STREAM_LOCKED;
  }
  
  streams_in_use += 1U;
  
  locked = atomic_load(&stream_rw_locked);
  
  if (locked) {
    streams_in_use -= 1U;
    return UORIBC_STREAM_LOCKED;
  }
  
  stream_found = find_stream(stream_id, &stream_ptr, NULL);
  
  if (stream_found == false) {
    streams_in_use -= 1U;
    return UORIBC_STREAM_ID_INVALID;
  }
  
  if (stream_ptr->stream_dir != UORIBC_STREAM_DIR_TX) {
    streams_in_use -= 1U;
    return UORIBC_STREAM_DIR_MISMATCH;
  }
  
  // Split the payload into chunks that fit within a UORIBC Frame
  uint8_t number_of_payloads = (num_to_write / STREAM_MAX_PAYLOAD_SIZE);
  uint8_t tail_size = num_to_write % STREAM_MAX_PAYLOAD_SIZE;
  uoribc_interface_error_t tx_error;

  // Create write context
  uoribc_message_context_t context_to_send = {
    .interface_id       = stream_ptr->interface_id,
    .identifier         = stream_ptr->stream_id,
    .type               = UORIBC_STREAM,
    .skeleton_size      = STREAM_MAX_PAYLOAD_SIZE,
  };

  
  for (uint8_t i = 0U; i < number_of_payloads; i++) {
    memcpy(context_to_send.skeleton.stream.payload, &buffer[i*8U], STREAM_MAX_PAYLOAD_SIZE);
    tx_error = uoribc_message_transmit(context_to_send);
    switch (tx_error) {
      case (UORIBC_INTR_LOCKED):
        streams_in_use -= 1U;
        return UORIBC_STREAM_INTR_ERR;
        break;
      case (UORIBC_INTR_NOT_REG):
        streams_in_use -= 1U;
        return UORIBC_STREAM_INTR_ERR;
        break;
      default:
        break;
    }
  }
  
  // Send tail
  if (tail_size > 0U) {
    memcpy(context_to_send.skeleton.stream.payload, &buffer[num_to_write-tail_size], tail_size);
    tx_error = uoribc_message_transmit(context_to_send);
      switch (tx_error) {
      case (UORIBC_INTR_LOCKED):
        return UORIBC_STREAM_INTR_ERR;
        break;
      case (UORIBC_INTR_NOT_REG):
        return UORIBC_STREAM_INTR_ERR;
        break;
      default:
        break;
    }
  }
  
  streams_in_use -= 1U;
  return UORIBC_STREAM_OK;
}
