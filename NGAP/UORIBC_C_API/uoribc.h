

#ifndef UORIBC 
#define UORIBC

#include <stdint.h>


// Contains all public structs, enums, and function prototypes for the UORIBC API:


// The following are public enums:

typedef enum {
  UORIBC_GEN_OK = 0,
  UORIBC_ALREADY_INITIALIZED = 1,
} uoribc_general_error_t;

typedef enum {
  UORIBC_INTERFACE_OK = 0,
  
  UORIBC_TRANSMIT_ERROR_0 = 1,
  UORIBC_TRANSMIT_ERROR_1 = 2,
  UORIBC_TRANSMIT_ERROR_2 = 3,
  UORIBC_TRANSMIT_ERROR_3 = 4,
  UORIBC_TRANSMIT_ERROR_4 = 5,
  UORIBC_TRANSMIT_ERROR_5 = 6,
  UORIBC_TRANSMIT_ERROR_6 = 7,
  UORIBC_TRANSMIT_ERROR_7 = 8,
  
  UORIBC_MAX_INTERFACES = 9,
  UORIBC_INTR_NOT_REG = 10,
  UORIBC_INTR_LOCKED = 12,
} uoribc_interface_error_t;

typedef enum {
  UORIBC_CALLBACK_OK = 0,
  UORIBC_CALLBACK_INVALID = 1,
  UORIBC_CALLBACK_ERROR = 2,
  UORIBC_MAX_CALLBACKS = 3,
  UORIBC_CALLBACK_LOCKED = 4,
} uoribc_callback_error_t;

typedef enum {
  UORIBC_DISPATCH_OK = 0,
  UORIBC_DISPATCH_INVALID_CALLBACK = 1,
  UORIBC_DISP_LOCKED = 2,
} uoribc_dispatch_error_t;

typedef enum {
  UORIBC_MESSAGE = 0,
  UORIBC_STREAM = 1,
} uoribc_payload_types_t;

typedef enum {
  UORIBC_INTERFACE_0 = 0b00000001,
  UORIBC_INTERFACE_1 = 0b00000010,
  UORIBC_INTERFACE_2 = 0b00000100,
  UORIBC_INTERFACE_3 = 0b00001000
} uoribc_interface_id_t;

// The following are skeleton related public structs & unions

typedef struct {
  uint8_t                       header;
  uint8_t                       *payload;
} uoribc_frame_skeleton_t;

typedef struct {
  uint8_t                       *payload;
} uoribc_stream_skeleton_t;

typedef union {
  uoribc_frame_skeleton_t       frame;
  uoribc_stream_skeleton_t      stream;

} uoribc_skeleton_t;



// The following are structs related to messages
typedef struct {
  uoribc_interface_id_t         interface_id;
  uoribc_skeleton_t             skeleton;
  uint8_t                       skeleton_size;
  uoribc_payload_types_t        type;             
  uint8_t                       identifier;
  uint8_t                       node_id;
} uoribc_message_context_t;


\
// The following are function prototypes for weakly defined functions: 

  void uoribc_recieve_callback(uoribc_message_context_t context);


// The following are public function prototypes:

  uoribc_general_error_t uoribc_initialize(uint8_t node_id);
  
  uoribc_interface_error_t uoribc_interface_register(uoribc_interface_error_t (*transmit_function)(uoribc_message_context_t), uoribc_interface_id_t* interface_id);
  
  uoribc_callback_error_t uoribc_callback_register(uoribc_interface_id_t interface_id, uint8_t class_id, uint8_t message_id, void (*callback_function)(uoribc_message_context_t));
  
  uoribc_dispatch_error_t uoribc_message_recieve(uoribc_message_context_t context); 
  
  uoribc_interface_error_t uoribc_message_transmit(uoribc_message_context_t context);

#endif