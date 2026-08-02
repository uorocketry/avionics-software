

#ifndef UORIBC 
#define UORIBC

#include <stdint.h>


// Contains all public structs, enums, and function prototypes for the UORIBC API:


// The following are public enums:

typedef enum {
  UORIBC_GEN_OK,
  UORIBC_ALREADY_INITIALIZED,
} uoribc_general_error_t;

typedef enum {
  UORIBC_INTERFACE_OK,
  
  UORIBC_TRANSMIT_ERROR_0,
  UORIBC_TRANSMIT_ERROR_1,
  UORIBC_TRANSMIT_ERROR_2,
  UORIBC_TRANSMIT_ERROR_3,
  UORIBC_TRANSMIT_ERROR_4,
  UORIBC_TRANSMIT_ERROR_5,
  UORIBC_TRANSMIT_ERROR_6,
  UORIBC_TRANSMIT_ERROR_7,
  
  UORIBC_MAX_INTERFACES,
  UORIBC_INTR_NOT_REG,
  UORIBC_INTR_LOCKED,
  UORIBC_INTR_PTR_NULL,
} uoribc_interface_error_t;

typedef enum {
  UORIBC_CALLBACK_OK,
  UORIBC_CALLBACK_INVALID,
  UORIBC_CALLBACK_ERROR,
  UORIBC_MAX_CALLBACKS,
  UORIBC_CALLBACK_LOCKED,
} uoribc_callback_error_t;

typedef enum {
  UORIBC_DISPATCH_OK,
  UORIBC_DISPATCH_INVALID_CALLBACK,
  UORIBC_DISPATCH_STREAM_FULL,
  UORIBC_DISP_LOCKED,
} uoribc_dispatch_error_t;

typedef enum {
  UORIBC_STREAM_OK,
  UORIBC_STREAM_ID_INVALID,
  UORIBC_STREAM_EXISTS,
  UORIBC_MAX_STREAMS,
  UORIBC_STREAM_INVALID_MASK_WIDTH,
  UORIBC_STREAM_INVALID_SIZE,
  UORIBC_STREAM_DIR_MISMATCH,
  UORIBC_STREAM_LOCKED,
  UORIBC_STREAM_INTR_ERR,
} uoribc_stream_error_t;

typedef enum {
  UORIBC_MESSAGE,
  UORIBC_STREAM,
} uoribc_payload_types_t;

typedef enum {
  UORIBC_INTERFACE_0 = 0b00000001U,
  UORIBC_INTERFACE_1 = 0b00000010U,
  UORIBC_INTERFACE_2 = 0b00000100U,
  UORIBC_INTERFACE_3 = 0b00001000U,
} uoribc_interface_id_t;

typedef enum {
  UORIBC_STREAM_DIR_TX,
  UORIBC_STREAM_DIR_RX,
} uoribc_stream_dir_t;
// The following are skeleton related public structs & unions



typedef struct {
  uint8_t                       header;
  uint8_t                       payload[7];
} uoribc_frame_skeleton_t;

typedef struct {
  uint8_t                       payload[8];
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



// The following are function prototypes for weakly defined functions: 

  void uoribc_recieve_callback(uoribc_message_context_t context);


// The following are public function prototypes:

  uoribc_general_error_t        uoribc_initialize(uint8_t node_id);
  
  uoribc_interface_error_t      uoribc_interface_register(uoribc_interface_error_t (*transmit_function)(uoribc_message_context_t), uoribc_interface_id_t* interface_id);
  
  uoribc_callback_error_t       uoribc_callback_register(uoribc_interface_id_t interface_id, uint8_t class_id, uint8_t message_id, void (*callback_function)(uoribc_message_context_t));
  
  uoribc_dispatch_error_t       uoribc_message_receive(uoribc_message_context_t context); 
  
  uoribc_interface_error_t      uoribc_message_transmit(uoribc_message_context_t context);

  uoribc_stream_error_t         uoribc_stream_open(uoribc_interface_id_t interface_id, uint8_t stream_id, uoribc_stream_dir_t stream_dir, uint8_t stream_size, void (*stream_full_callback)(uint8_t stream_id));
  
  uoribc_stream_error_t         uoribc_stream_close(uint8_t stream_id);
  
  uoribc_stream_error_t         uoribc_stream_read_bytes(uint8_t stream_id, uint8_t *buffer, uint8_t num_to_read, uint8_t *num_read_ptr);
  
  uoribc_stream_error_t         uoribc_stream_write_bytes(uint8_t stream_id, uint8_t *buffer, uint8_t num_to_write);

  
#endif