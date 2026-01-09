#pragma once
// MESSAGE TEST_UOR PACKING

#define MAVLINK_MSG_ID_TEST_UOR 60000


typedef struct __mavlink_test_uor_t {
 uint8_t TEST; /*<  Test field for the uor mavlink dialect*/
} mavlink_test_uor_t;

#define MAVLINK_MSG_ID_TEST_UOR_LEN 1
#define MAVLINK_MSG_ID_TEST_UOR_MIN_LEN 1
#define MAVLINK_MSG_ID_60000_LEN 1
#define MAVLINK_MSG_ID_60000_MIN_LEN 1

#define MAVLINK_MSG_ID_TEST_UOR_CRC 57
#define MAVLINK_MSG_ID_60000_CRC 57



#if MAVLINK_COMMAND_24BIT
#define MAVLINK_MESSAGE_INFO_TEST_UOR { \
    60000, \
    "TEST_UOR", \
    1, \
    {  { "TEST", NULL, MAVLINK_TYPE_UINT8_T, 0, 0, offsetof(mavlink_test_uor_t, TEST) }, \
         } \
}
#else
#define MAVLINK_MESSAGE_INFO_TEST_UOR { \
    "TEST_UOR", \
    1, \
    {  { "TEST", NULL, MAVLINK_TYPE_UINT8_T, 0, 0, offsetof(mavlink_test_uor_t, TEST) }, \
         } \
}
#endif

/**
 * @brief Pack a test_uor message
 * @param system_id ID of this system
 * @param component_id ID of this component (e.g. 200 for IMU)
 * @param msg The MAVLink message to compress the data into
 *
 * @param TEST  Test field for the uor mavlink dialect
 * @return length of the message in bytes (excluding serial stream start sign)
 */
static inline uint16_t mavlink_msg_test_uor_pack(uint8_t system_id, uint8_t component_id, mavlink_message_t* msg,
                               uint8_t TEST)
{
#if MAVLINK_NEED_BYTE_SWAP || !MAVLINK_ALIGNED_FIELDS
    char buf[MAVLINK_MSG_ID_TEST_UOR_LEN];
    _mav_put_uint8_t(buf, 0, TEST);

        memcpy(_MAV_PAYLOAD_NON_CONST(msg), buf, MAVLINK_MSG_ID_TEST_UOR_LEN);
#else
    mavlink_test_uor_t packet;
    packet.TEST = TEST;

        memcpy(_MAV_PAYLOAD_NON_CONST(msg), &packet, MAVLINK_MSG_ID_TEST_UOR_LEN);
#endif

    msg->msgid = MAVLINK_MSG_ID_TEST_UOR;
    return mavlink_finalize_message(msg, system_id, component_id, MAVLINK_MSG_ID_TEST_UOR_MIN_LEN, MAVLINK_MSG_ID_TEST_UOR_LEN, MAVLINK_MSG_ID_TEST_UOR_CRC);
}

/**
 * @brief Pack a test_uor message
 * @param system_id ID of this system
 * @param component_id ID of this component (e.g. 200 for IMU)
 * @param status MAVLink status structure
 * @param msg The MAVLink message to compress the data into
 *
 * @param TEST  Test field for the uor mavlink dialect
 * @return length of the message in bytes (excluding serial stream start sign)
 */
static inline uint16_t mavlink_msg_test_uor_pack_status(uint8_t system_id, uint8_t component_id, mavlink_status_t *_status, mavlink_message_t* msg,
                               uint8_t TEST)
{
#if MAVLINK_NEED_BYTE_SWAP || !MAVLINK_ALIGNED_FIELDS
    char buf[MAVLINK_MSG_ID_TEST_UOR_LEN];
    _mav_put_uint8_t(buf, 0, TEST);

        memcpy(_MAV_PAYLOAD_NON_CONST(msg), buf, MAVLINK_MSG_ID_TEST_UOR_LEN);
#else
    mavlink_test_uor_t packet;
    packet.TEST = TEST;

        memcpy(_MAV_PAYLOAD_NON_CONST(msg), &packet, MAVLINK_MSG_ID_TEST_UOR_LEN);
#endif

    msg->msgid = MAVLINK_MSG_ID_TEST_UOR;
#if MAVLINK_CRC_EXTRA
    return mavlink_finalize_message_buffer(msg, system_id, component_id, _status, MAVLINK_MSG_ID_TEST_UOR_MIN_LEN, MAVLINK_MSG_ID_TEST_UOR_LEN, MAVLINK_MSG_ID_TEST_UOR_CRC);
#else
    return mavlink_finalize_message_buffer(msg, system_id, component_id, _status, MAVLINK_MSG_ID_TEST_UOR_MIN_LEN, MAVLINK_MSG_ID_TEST_UOR_LEN);
#endif
}

/**
 * @brief Pack a test_uor message on a channel
 * @param system_id ID of this system
 * @param component_id ID of this component (e.g. 200 for IMU)
 * @param chan The MAVLink channel this message will be sent over
 * @param msg The MAVLink message to compress the data into
 * @param TEST  Test field for the uor mavlink dialect
 * @return length of the message in bytes (excluding serial stream start sign)
 */
static inline uint16_t mavlink_msg_test_uor_pack_chan(uint8_t system_id, uint8_t component_id, uint8_t chan,
                               mavlink_message_t* msg,
                                   uint8_t TEST)
{
#if MAVLINK_NEED_BYTE_SWAP || !MAVLINK_ALIGNED_FIELDS
    char buf[MAVLINK_MSG_ID_TEST_UOR_LEN];
    _mav_put_uint8_t(buf, 0, TEST);

        memcpy(_MAV_PAYLOAD_NON_CONST(msg), buf, MAVLINK_MSG_ID_TEST_UOR_LEN);
#else
    mavlink_test_uor_t packet;
    packet.TEST = TEST;

        memcpy(_MAV_PAYLOAD_NON_CONST(msg), &packet, MAVLINK_MSG_ID_TEST_UOR_LEN);
#endif

    msg->msgid = MAVLINK_MSG_ID_TEST_UOR;
    return mavlink_finalize_message_chan(msg, system_id, component_id, chan, MAVLINK_MSG_ID_TEST_UOR_MIN_LEN, MAVLINK_MSG_ID_TEST_UOR_LEN, MAVLINK_MSG_ID_TEST_UOR_CRC);
}

/**
 * @brief Encode a test_uor struct
 *
 * @param system_id ID of this system
 * @param component_id ID of this component (e.g. 200 for IMU)
 * @param msg The MAVLink message to compress the data into
 * @param test_uor C-struct to read the message contents from
 */
static inline uint16_t mavlink_msg_test_uor_encode(uint8_t system_id, uint8_t component_id, mavlink_message_t* msg, const mavlink_test_uor_t* test_uor)
{
    return mavlink_msg_test_uor_pack(system_id, component_id, msg, test_uor->TEST);
}

/**
 * @brief Encode a test_uor struct on a channel
 *
 * @param system_id ID of this system
 * @param component_id ID of this component (e.g. 200 for IMU)
 * @param chan The MAVLink channel this message will be sent over
 * @param msg The MAVLink message to compress the data into
 * @param test_uor C-struct to read the message contents from
 */
static inline uint16_t mavlink_msg_test_uor_encode_chan(uint8_t system_id, uint8_t component_id, uint8_t chan, mavlink_message_t* msg, const mavlink_test_uor_t* test_uor)
{
    return mavlink_msg_test_uor_pack_chan(system_id, component_id, chan, msg, test_uor->TEST);
}

/**
 * @brief Encode a test_uor struct with provided status structure
 *
 * @param system_id ID of this system
 * @param component_id ID of this component (e.g. 200 for IMU)
 * @param status MAVLink status structure
 * @param msg The MAVLink message to compress the data into
 * @param test_uor C-struct to read the message contents from
 */
static inline uint16_t mavlink_msg_test_uor_encode_status(uint8_t system_id, uint8_t component_id, mavlink_status_t* _status, mavlink_message_t* msg, const mavlink_test_uor_t* test_uor)
{
    return mavlink_msg_test_uor_pack_status(system_id, component_id, _status, msg,  test_uor->TEST);
}

/**
 * @brief Send a test_uor message
 * @param chan MAVLink channel to send the message
 *
 * @param TEST  Test field for the uor mavlink dialect
 */
#ifdef MAVLINK_USE_CONVENIENCE_FUNCTIONS

static inline void mavlink_msg_test_uor_send(mavlink_channel_t chan, uint8_t TEST)
{
#if MAVLINK_NEED_BYTE_SWAP || !MAVLINK_ALIGNED_FIELDS
    char buf[MAVLINK_MSG_ID_TEST_UOR_LEN];
    _mav_put_uint8_t(buf, 0, TEST);

    _mav_finalize_message_chan_send(chan, MAVLINK_MSG_ID_TEST_UOR, buf, MAVLINK_MSG_ID_TEST_UOR_MIN_LEN, MAVLINK_MSG_ID_TEST_UOR_LEN, MAVLINK_MSG_ID_TEST_UOR_CRC);
#else
    mavlink_test_uor_t packet;
    packet.TEST = TEST;

    _mav_finalize_message_chan_send(chan, MAVLINK_MSG_ID_TEST_UOR, (const char *)&packet, MAVLINK_MSG_ID_TEST_UOR_MIN_LEN, MAVLINK_MSG_ID_TEST_UOR_LEN, MAVLINK_MSG_ID_TEST_UOR_CRC);
#endif
}

/**
 * @brief Send a test_uor message
 * @param chan MAVLink channel to send the message
 * @param struct The MAVLink struct to serialize
 */
static inline void mavlink_msg_test_uor_send_struct(mavlink_channel_t chan, const mavlink_test_uor_t* test_uor)
{
#if MAVLINK_NEED_BYTE_SWAP || !MAVLINK_ALIGNED_FIELDS
    mavlink_msg_test_uor_send(chan, test_uor->TEST);
#else
    _mav_finalize_message_chan_send(chan, MAVLINK_MSG_ID_TEST_UOR, (const char *)test_uor, MAVLINK_MSG_ID_TEST_UOR_MIN_LEN, MAVLINK_MSG_ID_TEST_UOR_LEN, MAVLINK_MSG_ID_TEST_UOR_CRC);
#endif
}

#if MAVLINK_MSG_ID_TEST_UOR_LEN <= MAVLINK_MAX_PAYLOAD_LEN
/*
  This variant of _send() can be used to save stack space by reusing
  memory from the receive buffer.  The caller provides a
  mavlink_message_t which is the size of a full mavlink message. This
  is usually the receive buffer for the channel, and allows a reply to an
  incoming message with minimum stack space usage.
 */
static inline void mavlink_msg_test_uor_send_buf(mavlink_message_t *msgbuf, mavlink_channel_t chan,  uint8_t TEST)
{
#if MAVLINK_NEED_BYTE_SWAP || !MAVLINK_ALIGNED_FIELDS
    char *buf = (char *)msgbuf;
    _mav_put_uint8_t(buf, 0, TEST);

    _mav_finalize_message_chan_send(chan, MAVLINK_MSG_ID_TEST_UOR, buf, MAVLINK_MSG_ID_TEST_UOR_MIN_LEN, MAVLINK_MSG_ID_TEST_UOR_LEN, MAVLINK_MSG_ID_TEST_UOR_CRC);
#else
    mavlink_test_uor_t *packet = (mavlink_test_uor_t *)msgbuf;
    packet->TEST = TEST;

    _mav_finalize_message_chan_send(chan, MAVLINK_MSG_ID_TEST_UOR, (const char *)packet, MAVLINK_MSG_ID_TEST_UOR_MIN_LEN, MAVLINK_MSG_ID_TEST_UOR_LEN, MAVLINK_MSG_ID_TEST_UOR_CRC);
#endif
}
#endif

#endif

// MESSAGE TEST_UOR UNPACKING


/**
 * @brief Get field TEST from test_uor message
 *
 * @return  Test field for the uor mavlink dialect
 */
static inline uint8_t mavlink_msg_test_uor_get_TEST(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint8_t(msg,  0);
}

/**
 * @brief Decode a test_uor message into a struct
 *
 * @param msg The message to decode
 * @param test_uor C-struct to decode the message contents into
 */
static inline void mavlink_msg_test_uor_decode(const mavlink_message_t* msg, mavlink_test_uor_t* test_uor)
{
#if MAVLINK_NEED_BYTE_SWAP || !MAVLINK_ALIGNED_FIELDS
    test_uor->TEST = mavlink_msg_test_uor_get_TEST(msg);
#else
        uint8_t len = msg->len < MAVLINK_MSG_ID_TEST_UOR_LEN? msg->len : MAVLINK_MSG_ID_TEST_UOR_LEN;
        memset(test_uor, 0, MAVLINK_MSG_ID_TEST_UOR_LEN);
    memcpy(test_uor, _MAV_PAYLOAD(msg), len);
#endif
}
