#include "uoribc.h"
#include <stddef.h>
#include <stdbool.h>

// The following are macros used by UORIBC:

#define INTERFACE_REGISTRY_SIZE 4U
#define DISPATCH_TABLE_SIZE 64U
#define MESSAGE_ID_MASK 127U

// The following are structs used by UORIBC:

typedef struct {
  uoribc_interface_error_t (*transmit_function)(uoribc_message_context_t);
  uoribc_interface_id_t interface_id;
} interface_t; 


typedef struct {
  void (*callback_function)(uoribc_message_context_t context);
  uoribc_interface_id_t interface_id;
  uint8_t class_id;
  uint8_t message_id;
} callback_entry_t;


// The following are buffers, registrys, and other statics used by UORIBC:

static uint8_t          device_node_id = 0;

static interface_t      interface_registry[INTERFACE_REGISTRY_SIZE];
static uint8_t          interface_registry_index = 0;

static callback_entry_t callback_dispatch_table[DISPATCH_TABLE_SIZE];
static uint8_t          callback_table_index = 0; 

static bool             device_initialized = false;

// The following are weakly implemented private functions:

__weak void uoribc_recieve_callback(uoribc_message_context_t context) {
  // DO NOT IMPLEMENT HERE. REDEFINE THE FUNCTION ELSEWHERE, PRESERVING THIS ONE.
};


// The following are implemented public functions:

uoribc_general_error_t uoribc_initialize(uint8_t node_id){
  if (device_initialized == true) {
    return UORIBC_ALREADY_INITIALIZED;
  }
  
  device_node_id = node_id;
  device_initialized = true;
  
  return UORIBC_GEN_OK;
}

uoribc_interface_error_t uoribc_interface_register(uoribc_interface_error_t (*transmit_function)(uoribc_message_context_t), uoribc_interface_id_t* interface_id) {
  
  if (device_initialized) {
    return UORIBC_INTR_LOCKED;
  }
  
  if (interface_registry_index + 1 >= INTERFACE_REGISTRY_SIZE) {
    return UORIBC_MAX_INTERFACES;
  }
  
  interface_t interface_struct;
  interface_struct.interface_id = (uoribc_interface_id_t)(0b00000001 << interface_registry_index);
  interface_struct.transmit_function = transmit_function;
  
  *interface_id = interface_struct.interface_id;
  
  interface_registry[interface_registry_index] = interface_struct;
  interface_registry_index += 1;
  
  return UORIBC_INTERFACE_OK;
}

uoribc_callback_error_t uoribc_callback_register(uoribc_interface_id_t interface_id, uint8_t class_id, uint8_t message_id, void (*callback_function)(uoribc_message_context_t)) {
  
  if (device_initialized) {
    return UORIBC_CALLBACK_LOCKED;
  }
  
  if (callback_table_index + 1 >= DISPATCH_TABLE_SIZE) {
    return UORIBC_MAX_CALLBACKS;
  }

  if (callback_function == NULL) {
    return UORIBC_CALLBACK_INVALID;
  }
  
  callback_entry_t callback_entry;
  callback_entry.callback_function = callback_function;
  callback_entry.interface_id = interface_id;
  callback_entry.class_id = class_id;
  callback_entry.message_id = message_id;
  
  callback_dispatch_table[callback_table_index] = callback_entry;
  callback_table_index += 1;
  
  return UORIBC_CALLBACK_OK;
}

uoribc_dispatch_error_t uoribc_message_recieve(uoribc_message_context_t context) {
  
  uoribc_recieve_callback(context);

  if (device_initialized == false) {
    return UORIBC_DISP_LOCKED;
  }
  
  if (context.type == UORIBC_MESSAGE) {
    uint8_t class_id = context.identifier;
    uint8_t message_id = context.skeleton.frame.header & MESSAGE_ID_MASK;
    uoribc_interface_id_t interface_id = context.interface_id;
    
    for (uint16_t i = 0; i < callback_table_index; i++) {
      callback_entry_t entry = callback_dispatch_table[i];
      
      // Check if class id matches
      if ((entry.class_id == class_id) && 
          (entry.message_id == message_id) &&
          (entry.interface_id & interface_id) == interface_id) 
          {
            if (entry.callback_function == NULL) {
              return UORIBC_DISPATCH_INVALID_CALLBACK;
            }
            // Dispatch Callback
            entry.callback_function(context);
          }
      }
  }
  return UORIBC_DISPATCH_OK;
}

uoribc_interface_error_t uoribc_message_transmit(uoribc_message_context_t context) {
  
  if (device_initialized == false) {
    return UORIBC_INTR_LOCKED;
  }
  
  uoribc_interface_id_t interface_id = context.interface_id;
  interface_t *interface_ptr = NULL;

  for (uint8_t i = 0; i < interface_registry_index; i++) {
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

