#include <stdint.h>

#ifndef UORIBC_MESSAGES
#define UORIBC_MESSAGES

#define DEFAULT_CLASSES

/* =========================================================================
 *UORIBC Class Definitions
 * ========================================================================= */

#define UORIBC_CLASS_ESTOP      0U
#define UORIBC_CLASS_EMATCH     1U
#define UORIBC_CLASS_IAP        5U
#define UORIBC_CLASS_CONFIG     6U
#define UORIBC_CLASS_RESP_ACK   7U
#define UORIBC_CLASS_GEN_STAT   8U
#define UORIBC_CLASS_FC         9U
#define UORIBC_CLASS_REC        10U
#define UORIBC_CLASS_LT         11U


#ifdef DEFAULT_CLASSES

#define ESTOP_CLASS
#define IAP_CLASS
#define CONFIG_CLASS
#define RESP_ACK_CLASS
#define GEN_STAT_CLASS

#endif


#ifdef ESTOP_CLASS

/* =========================================================================
 * Class 0 - E-Stop
 * ========================================================================= */

#define UORIBC_ESTOP_ASSERT             0U

typedef struct
{
    
} uoribc_estop_assert_t;

#endif

#ifdef EMATCH_CLASS

/* =========================================================================
 * Class 1 - E-Match Deployment
 * ========================================================================= */

#define UORIBC_EMATCH_DEPLOY            0U

typedef struct
{
    
    uint8_t charge_group;
} uoribc_ematch_deploy_t;

#endif

#ifdef IAP_CLASS

/* =========================================================================
 * Class 5 - IAP
 * ========================================================================= */

#endif

#ifdef CONFIG_CLASS

/* =========================================================================
 * Class 6 - Board Configuration
 * ========================================================================= */

#define UORIBC_CONFIG_GET_DESCRIPTOR            0U
#define UORIBC_CONFIG_GET_LED_INFO              1U
#define UORIBC_CONFIG_LED_INFO                  2U
#define UORIBC_CONFIG_BOARD_DESCRIPTOR          3U
#define UORIBC_CONFIG_CONFIGURATION_DESCRIPTOR  4U
#define UORIBC_CONFIG_LED_DESCRIPTOR            5U
#define UORIBC_CONFIG_SET_CONFIGURATION         6U
#define UORIBC_CONFIG_SET_LED                   7U
#define UORIBC_CONFIG_OPEN_STREAM               8U
#define UORIBC_CONFIG_CLOSE_STREAM              9U
#define UORIBC_CONFIG_GET_STRING               10U


typedef struct
{
    
    uint8_t node_id;
    uint8_t descriptor_type;
    uint8_t descriptor_index;
} uoribc_get_descriptor_t;


typedef struct
{
    
    uint8_t node_id;
} uoribc_get_led_info_t;


typedef struct
{
    
    uint8_t number_of_leds;
} uoribc_led_info_t;


typedef struct
{
    
    uint8_t name_id;
    uint8_t board_type;
    uint8_t number_of_configurables;
    uint8_t major_revision;
    uint8_t minor_revision;
    uint8_t board_variant;
    uint8_t board_extended_functionality;
} uoribc_board_descriptor_t;


typedef struct
{
    
    uint8_t descriptor_id;
    uint8_t description_id;
    uint8_t data_type;
    uint8_t data[4];
} uoribc_configuration_descriptor_t;


typedef struct
{
    
    uint8_t descriptor_id;
    uint8_t description_id;
    uint8_t current_state;
    uint8_t maximum_state;
} uoribc_led_descriptor_t;


typedef struct
{
    
    uint8_t node_id;
    uint8_t descriptor_id;
    uint8_t data[4];
} uoribc_set_configuration_t;


typedef struct
{
    
    uint8_t descriptor_id;
    uint8_t state;
} uoribc_set_led_t;


typedef struct
{
    
    uint8_t node_id;
    uint8_t stream_id;
    uint8_t direction;
    uint8_t stream_type;
    uint8_t length;
    uint8_t timeout[2];
} uoribc_open_stream_t;


typedef struct
{
    
    uint8_t stream_id;
} uoribc_close_stream_t;


typedef struct
{
    
    uint8_t node_id;
    uint8_t string_id;
    uint8_t stream_id;
} uoribc_get_string_t;

#endif

#ifdef RESP_ACK_CLASS

/* =========================================================================
 * Class 7 - Response and Acknowledgement
 * ========================================================================= */

#define UORIBC_RESP_MESSAGE_ACK         0U
#define UORIBC_RESP_MESSAGE_NACK        1U
#define UORIBC_RESP_NODE_ACK            2U
#define UORIBC_RESP_NODE_NACK           3U


typedef struct
{
    
    uint8_t node_id;
    uint8_t class_id;
    uint8_t message_id;
} uoribc_message_ack_t;


typedef struct
{
    
    uint8_t node_id;
    uint8_t class_id;
    uint8_t message_id;
} uoribc_message_nack_t;


typedef struct
{
    
    uint8_t node_id;
} uoribc_node_ack_t;


typedef struct
{
    
    uint8_t node_id;
} uoribc_node_nack_t;

#endif

#ifdef GEN_STAT_CLASS

/* =========================================================================
 * Class 8 - General Board Status
 * ========================================================================= */

#define UORIBC_GEN_STAT_BOARD_RESET     0U

typedef struct
{
    
    uint8_t reason;
} uoribc_board_reset_t;

#endif

#ifdef FC_CLASS

/* =========================================================================
 * Class 9 - Flight Controller
 * ========================================================================= */

#endif

#ifdef REC_CLASS

/* =========================================================================
 * Class 10 - Recovery/Pyro
 * ========================================================================= */

#define UORIBC_REC_CHANNEL_STATE        0U
#define UORIBC_REC_CHANNEL_DEPLOYED     1U


typedef struct
{
    
    uint8_t physical_channel;
    uint8_t charge_group;
    uint8_t state;
} uoribc_channel_state_t;


typedef struct
{
    
    uint8_t physical_channel;
    uint8_t charge_group;
} uoribc_channel_deployed_t;

#endif

#ifdef CONFIG_CLASS

/* =========================================================================
 * Class 11 - Logger/Telemetry
 * ========================================================================= */

#endif
#endif