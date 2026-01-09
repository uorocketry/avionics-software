#pragma once
// MESSAGE THERMOCOUPLE_UOR PACKING

#define MAVLINK_MSG_ID_THERMOCOUPLE_UOR 60001


typedef struct __mavlink_thermocouple_uor_t {
 uint32_t TC_1; /*<  Value for thermocouple 1*/
 uint32_t TC_2; /*<  Value for thermocouple 2*/
 uint32_t TC_3; /*<  Value for thermocouple 3*/
 uint32_t TC_4; /*<  Value for thermocouple 4*/
 uint32_t TC_5; /*<  Value for thermocouple 5*/
 uint32_t TC_6; /*<  Value for thermocouple 6*/
 uint32_t TC_7; /*<  Value for thermocouple 7*/
 uint32_t TC_8; /*<  Value for thermocouple 8*/
 uint32_t time_boot_ms; /*< [ms] Timestamp since boot*/
 uint8_t PAGE_NUM; /*<  Page number for thermocouple data. Set to 0 if less than 8 thermocouples are having their data transmitted*/
 uint8_t PAGE_TOTAL; /*<  Total number of pages for thermocouple data. Set to 0 if less than 8 thermocouples are having their data transmitted*/
} mavlink_thermocouple_uor_t;

#define MAVLINK_MSG_ID_THERMOCOUPLE_UOR_LEN 38
#define MAVLINK_MSG_ID_THERMOCOUPLE_UOR_MIN_LEN 38
#define MAVLINK_MSG_ID_60001_LEN 38
#define MAVLINK_MSG_ID_60001_MIN_LEN 38

#define MAVLINK_MSG_ID_THERMOCOUPLE_UOR_CRC 223
#define MAVLINK_MSG_ID_60001_CRC 223



#if MAVLINK_COMMAND_24BIT
#define MAVLINK_MESSAGE_INFO_THERMOCOUPLE_UOR { \
    60001, \
    "THERMOCOUPLE_UOR", \
    11, \
    {  { "TC_1", NULL, MAVLINK_TYPE_UINT32_T, 0, 0, offsetof(mavlink_thermocouple_uor_t, TC_1) }, \
         { "TC_2", NULL, MAVLINK_TYPE_UINT32_T, 0, 4, offsetof(mavlink_thermocouple_uor_t, TC_2) }, \
         { "TC_3", NULL, MAVLINK_TYPE_UINT32_T, 0, 8, offsetof(mavlink_thermocouple_uor_t, TC_3) }, \
         { "TC_4", NULL, MAVLINK_TYPE_UINT32_T, 0, 12, offsetof(mavlink_thermocouple_uor_t, TC_4) }, \
         { "TC_5", NULL, MAVLINK_TYPE_UINT32_T, 0, 16, offsetof(mavlink_thermocouple_uor_t, TC_5) }, \
         { "TC_6", NULL, MAVLINK_TYPE_UINT32_T, 0, 20, offsetof(mavlink_thermocouple_uor_t, TC_6) }, \
         { "TC_7", NULL, MAVLINK_TYPE_UINT32_T, 0, 24, offsetof(mavlink_thermocouple_uor_t, TC_7) }, \
         { "TC_8", NULL, MAVLINK_TYPE_UINT32_T, 0, 28, offsetof(mavlink_thermocouple_uor_t, TC_8) }, \
         { "PAGE_NUM", NULL, MAVLINK_TYPE_UINT8_T, 0, 36, offsetof(mavlink_thermocouple_uor_t, PAGE_NUM) }, \
         { "PAGE_TOTAL", NULL, MAVLINK_TYPE_UINT8_T, 0, 37, offsetof(mavlink_thermocouple_uor_t, PAGE_TOTAL) }, \
         { "time_boot_ms", NULL, MAVLINK_TYPE_UINT32_T, 0, 32, offsetof(mavlink_thermocouple_uor_t, time_boot_ms) }, \
         } \
}
#else
#define MAVLINK_MESSAGE_INFO_THERMOCOUPLE_UOR { \
    "THERMOCOUPLE_UOR", \
    11, \
    {  { "TC_1", NULL, MAVLINK_TYPE_UINT32_T, 0, 0, offsetof(mavlink_thermocouple_uor_t, TC_1) }, \
         { "TC_2", NULL, MAVLINK_TYPE_UINT32_T, 0, 4, offsetof(mavlink_thermocouple_uor_t, TC_2) }, \
         { "TC_3", NULL, MAVLINK_TYPE_UINT32_T, 0, 8, offsetof(mavlink_thermocouple_uor_t, TC_3) }, \
         { "TC_4", NULL, MAVLINK_TYPE_UINT32_T, 0, 12, offsetof(mavlink_thermocouple_uor_t, TC_4) }, \
         { "TC_5", NULL, MAVLINK_TYPE_UINT32_T, 0, 16, offsetof(mavlink_thermocouple_uor_t, TC_5) }, \
         { "TC_6", NULL, MAVLINK_TYPE_UINT32_T, 0, 20, offsetof(mavlink_thermocouple_uor_t, TC_6) }, \
         { "TC_7", NULL, MAVLINK_TYPE_UINT32_T, 0, 24, offsetof(mavlink_thermocouple_uor_t, TC_7) }, \
         { "TC_8", NULL, MAVLINK_TYPE_UINT32_T, 0, 28, offsetof(mavlink_thermocouple_uor_t, TC_8) }, \
         { "PAGE_NUM", NULL, MAVLINK_TYPE_UINT8_T, 0, 36, offsetof(mavlink_thermocouple_uor_t, PAGE_NUM) }, \
         { "PAGE_TOTAL", NULL, MAVLINK_TYPE_UINT8_T, 0, 37, offsetof(mavlink_thermocouple_uor_t, PAGE_TOTAL) }, \
         { "time_boot_ms", NULL, MAVLINK_TYPE_UINT32_T, 0, 32, offsetof(mavlink_thermocouple_uor_t, time_boot_ms) }, \
         } \
}
#endif

/**
 * @brief Pack a thermocouple_uor message
 * @param system_id ID of this system
 * @param component_id ID of this component (e.g. 200 for IMU)
 * @param msg The MAVLink message to compress the data into
 *
 * @param TC_1  Value for thermocouple 1
 * @param TC_2  Value for thermocouple 2
 * @param TC_3  Value for thermocouple 3
 * @param TC_4  Value for thermocouple 4
 * @param TC_5  Value for thermocouple 5
 * @param TC_6  Value for thermocouple 6
 * @param TC_7  Value for thermocouple 7
 * @param TC_8  Value for thermocouple 8
 * @param PAGE_NUM  Page number for thermocouple data. Set to 0 if less than 8 thermocouples are having their data transmitted
 * @param PAGE_TOTAL  Total number of pages for thermocouple data. Set to 0 if less than 8 thermocouples are having their data transmitted
 * @param time_boot_ms [ms] Timestamp since boot
 * @return length of the message in bytes (excluding serial stream start sign)
 */
static inline uint16_t mavlink_msg_thermocouple_uor_pack(uint8_t system_id, uint8_t component_id, mavlink_message_t* msg,
                               uint32_t TC_1, uint32_t TC_2, uint32_t TC_3, uint32_t TC_4, uint32_t TC_5, uint32_t TC_6, uint32_t TC_7, uint32_t TC_8, uint8_t PAGE_NUM, uint8_t PAGE_TOTAL, uint32_t time_boot_ms)
{
#if MAVLINK_NEED_BYTE_SWAP || !MAVLINK_ALIGNED_FIELDS
    char buf[MAVLINK_MSG_ID_THERMOCOUPLE_UOR_LEN];
    _mav_put_uint32_t(buf, 0, TC_1);
    _mav_put_uint32_t(buf, 4, TC_2);
    _mav_put_uint32_t(buf, 8, TC_3);
    _mav_put_uint32_t(buf, 12, TC_4);
    _mav_put_uint32_t(buf, 16, TC_5);
    _mav_put_uint32_t(buf, 20, TC_6);
    _mav_put_uint32_t(buf, 24, TC_7);
    _mav_put_uint32_t(buf, 28, TC_8);
    _mav_put_uint32_t(buf, 32, time_boot_ms);
    _mav_put_uint8_t(buf, 36, PAGE_NUM);
    _mav_put_uint8_t(buf, 37, PAGE_TOTAL);

        memcpy(_MAV_PAYLOAD_NON_CONST(msg), buf, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_LEN);
#else
    mavlink_thermocouple_uor_t packet;
    packet.TC_1 = TC_1;
    packet.TC_2 = TC_2;
    packet.TC_3 = TC_3;
    packet.TC_4 = TC_4;
    packet.TC_5 = TC_5;
    packet.TC_6 = TC_6;
    packet.TC_7 = TC_7;
    packet.TC_8 = TC_8;
    packet.time_boot_ms = time_boot_ms;
    packet.PAGE_NUM = PAGE_NUM;
    packet.PAGE_TOTAL = PAGE_TOTAL;

        memcpy(_MAV_PAYLOAD_NON_CONST(msg), &packet, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_LEN);
#endif

    msg->msgid = MAVLINK_MSG_ID_THERMOCOUPLE_UOR;
    return mavlink_finalize_message(msg, system_id, component_id, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_MIN_LEN, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_LEN, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_CRC);
}

/**
 * @brief Pack a thermocouple_uor message
 * @param system_id ID of this system
 * @param component_id ID of this component (e.g. 200 for IMU)
 * @param status MAVLink status structure
 * @param msg The MAVLink message to compress the data into
 *
 * @param TC_1  Value for thermocouple 1
 * @param TC_2  Value for thermocouple 2
 * @param TC_3  Value for thermocouple 3
 * @param TC_4  Value for thermocouple 4
 * @param TC_5  Value for thermocouple 5
 * @param TC_6  Value for thermocouple 6
 * @param TC_7  Value for thermocouple 7
 * @param TC_8  Value for thermocouple 8
 * @param PAGE_NUM  Page number for thermocouple data. Set to 0 if less than 8 thermocouples are having their data transmitted
 * @param PAGE_TOTAL  Total number of pages for thermocouple data. Set to 0 if less than 8 thermocouples are having their data transmitted
 * @param time_boot_ms [ms] Timestamp since boot
 * @return length of the message in bytes (excluding serial stream start sign)
 */
static inline uint16_t mavlink_msg_thermocouple_uor_pack_status(uint8_t system_id, uint8_t component_id, mavlink_status_t *_status, mavlink_message_t* msg,
                               uint32_t TC_1, uint32_t TC_2, uint32_t TC_3, uint32_t TC_4, uint32_t TC_5, uint32_t TC_6, uint32_t TC_7, uint32_t TC_8, uint8_t PAGE_NUM, uint8_t PAGE_TOTAL, uint32_t time_boot_ms)
{
#if MAVLINK_NEED_BYTE_SWAP || !MAVLINK_ALIGNED_FIELDS
    char buf[MAVLINK_MSG_ID_THERMOCOUPLE_UOR_LEN];
    _mav_put_uint32_t(buf, 0, TC_1);
    _mav_put_uint32_t(buf, 4, TC_2);
    _mav_put_uint32_t(buf, 8, TC_3);
    _mav_put_uint32_t(buf, 12, TC_4);
    _mav_put_uint32_t(buf, 16, TC_5);
    _mav_put_uint32_t(buf, 20, TC_6);
    _mav_put_uint32_t(buf, 24, TC_7);
    _mav_put_uint32_t(buf, 28, TC_8);
    _mav_put_uint32_t(buf, 32, time_boot_ms);
    _mav_put_uint8_t(buf, 36, PAGE_NUM);
    _mav_put_uint8_t(buf, 37, PAGE_TOTAL);

        memcpy(_MAV_PAYLOAD_NON_CONST(msg), buf, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_LEN);
#else
    mavlink_thermocouple_uor_t packet;
    packet.TC_1 = TC_1;
    packet.TC_2 = TC_2;
    packet.TC_3 = TC_3;
    packet.TC_4 = TC_4;
    packet.TC_5 = TC_5;
    packet.TC_6 = TC_6;
    packet.TC_7 = TC_7;
    packet.TC_8 = TC_8;
    packet.time_boot_ms = time_boot_ms;
    packet.PAGE_NUM = PAGE_NUM;
    packet.PAGE_TOTAL = PAGE_TOTAL;

        memcpy(_MAV_PAYLOAD_NON_CONST(msg), &packet, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_LEN);
#endif

    msg->msgid = MAVLINK_MSG_ID_THERMOCOUPLE_UOR;
#if MAVLINK_CRC_EXTRA
    return mavlink_finalize_message_buffer(msg, system_id, component_id, _status, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_MIN_LEN, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_LEN, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_CRC);
#else
    return mavlink_finalize_message_buffer(msg, system_id, component_id, _status, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_MIN_LEN, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_LEN);
#endif
}

/**
 * @brief Pack a thermocouple_uor message on a channel
 * @param system_id ID of this system
 * @param component_id ID of this component (e.g. 200 for IMU)
 * @param chan The MAVLink channel this message will be sent over
 * @param msg The MAVLink message to compress the data into
 * @param TC_1  Value for thermocouple 1
 * @param TC_2  Value for thermocouple 2
 * @param TC_3  Value for thermocouple 3
 * @param TC_4  Value for thermocouple 4
 * @param TC_5  Value for thermocouple 5
 * @param TC_6  Value for thermocouple 6
 * @param TC_7  Value for thermocouple 7
 * @param TC_8  Value for thermocouple 8
 * @param PAGE_NUM  Page number for thermocouple data. Set to 0 if less than 8 thermocouples are having their data transmitted
 * @param PAGE_TOTAL  Total number of pages for thermocouple data. Set to 0 if less than 8 thermocouples are having their data transmitted
 * @param time_boot_ms [ms] Timestamp since boot
 * @return length of the message in bytes (excluding serial stream start sign)
 */
static inline uint16_t mavlink_msg_thermocouple_uor_pack_chan(uint8_t system_id, uint8_t component_id, uint8_t chan,
                               mavlink_message_t* msg,
                                   uint32_t TC_1,uint32_t TC_2,uint32_t TC_3,uint32_t TC_4,uint32_t TC_5,uint32_t TC_6,uint32_t TC_7,uint32_t TC_8,uint8_t PAGE_NUM,uint8_t PAGE_TOTAL,uint32_t time_boot_ms)
{
#if MAVLINK_NEED_BYTE_SWAP || !MAVLINK_ALIGNED_FIELDS
    char buf[MAVLINK_MSG_ID_THERMOCOUPLE_UOR_LEN];
    _mav_put_uint32_t(buf, 0, TC_1);
    _mav_put_uint32_t(buf, 4, TC_2);
    _mav_put_uint32_t(buf, 8, TC_3);
    _mav_put_uint32_t(buf, 12, TC_4);
    _mav_put_uint32_t(buf, 16, TC_5);
    _mav_put_uint32_t(buf, 20, TC_6);
    _mav_put_uint32_t(buf, 24, TC_7);
    _mav_put_uint32_t(buf, 28, TC_8);
    _mav_put_uint32_t(buf, 32, time_boot_ms);
    _mav_put_uint8_t(buf, 36, PAGE_NUM);
    _mav_put_uint8_t(buf, 37, PAGE_TOTAL);

        memcpy(_MAV_PAYLOAD_NON_CONST(msg), buf, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_LEN);
#else
    mavlink_thermocouple_uor_t packet;
    packet.TC_1 = TC_1;
    packet.TC_2 = TC_2;
    packet.TC_3 = TC_3;
    packet.TC_4 = TC_4;
    packet.TC_5 = TC_5;
    packet.TC_6 = TC_6;
    packet.TC_7 = TC_7;
    packet.TC_8 = TC_8;
    packet.time_boot_ms = time_boot_ms;
    packet.PAGE_NUM = PAGE_NUM;
    packet.PAGE_TOTAL = PAGE_TOTAL;

        memcpy(_MAV_PAYLOAD_NON_CONST(msg), &packet, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_LEN);
#endif

    msg->msgid = MAVLINK_MSG_ID_THERMOCOUPLE_UOR;
    return mavlink_finalize_message_chan(msg, system_id, component_id, chan, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_MIN_LEN, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_LEN, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_CRC);
}

/**
 * @brief Encode a thermocouple_uor struct
 *
 * @param system_id ID of this system
 * @param component_id ID of this component (e.g. 200 for IMU)
 * @param msg The MAVLink message to compress the data into
 * @param thermocouple_uor C-struct to read the message contents from
 */
static inline uint16_t mavlink_msg_thermocouple_uor_encode(uint8_t system_id, uint8_t component_id, mavlink_message_t* msg, const mavlink_thermocouple_uor_t* thermocouple_uor)
{
    return mavlink_msg_thermocouple_uor_pack(system_id, component_id, msg, thermocouple_uor->TC_1, thermocouple_uor->TC_2, thermocouple_uor->TC_3, thermocouple_uor->TC_4, thermocouple_uor->TC_5, thermocouple_uor->TC_6, thermocouple_uor->TC_7, thermocouple_uor->TC_8, thermocouple_uor->PAGE_NUM, thermocouple_uor->PAGE_TOTAL, thermocouple_uor->time_boot_ms);
}

/**
 * @brief Encode a thermocouple_uor struct on a channel
 *
 * @param system_id ID of this system
 * @param component_id ID of this component (e.g. 200 for IMU)
 * @param chan The MAVLink channel this message will be sent over
 * @param msg The MAVLink message to compress the data into
 * @param thermocouple_uor C-struct to read the message contents from
 */
static inline uint16_t mavlink_msg_thermocouple_uor_encode_chan(uint8_t system_id, uint8_t component_id, uint8_t chan, mavlink_message_t* msg, const mavlink_thermocouple_uor_t* thermocouple_uor)
{
    return mavlink_msg_thermocouple_uor_pack_chan(system_id, component_id, chan, msg, thermocouple_uor->TC_1, thermocouple_uor->TC_2, thermocouple_uor->TC_3, thermocouple_uor->TC_4, thermocouple_uor->TC_5, thermocouple_uor->TC_6, thermocouple_uor->TC_7, thermocouple_uor->TC_8, thermocouple_uor->PAGE_NUM, thermocouple_uor->PAGE_TOTAL, thermocouple_uor->time_boot_ms);
}

/**
 * @brief Encode a thermocouple_uor struct with provided status structure
 *
 * @param system_id ID of this system
 * @param component_id ID of this component (e.g. 200 for IMU)
 * @param status MAVLink status structure
 * @param msg The MAVLink message to compress the data into
 * @param thermocouple_uor C-struct to read the message contents from
 */
static inline uint16_t mavlink_msg_thermocouple_uor_encode_status(uint8_t system_id, uint8_t component_id, mavlink_status_t* _status, mavlink_message_t* msg, const mavlink_thermocouple_uor_t* thermocouple_uor)
{
    return mavlink_msg_thermocouple_uor_pack_status(system_id, component_id, _status, msg,  thermocouple_uor->TC_1, thermocouple_uor->TC_2, thermocouple_uor->TC_3, thermocouple_uor->TC_4, thermocouple_uor->TC_5, thermocouple_uor->TC_6, thermocouple_uor->TC_7, thermocouple_uor->TC_8, thermocouple_uor->PAGE_NUM, thermocouple_uor->PAGE_TOTAL, thermocouple_uor->time_boot_ms);
}

/**
 * @brief Send a thermocouple_uor message
 * @param chan MAVLink channel to send the message
 *
 * @param TC_1  Value for thermocouple 1
 * @param TC_2  Value for thermocouple 2
 * @param TC_3  Value for thermocouple 3
 * @param TC_4  Value for thermocouple 4
 * @param TC_5  Value for thermocouple 5
 * @param TC_6  Value for thermocouple 6
 * @param TC_7  Value for thermocouple 7
 * @param TC_8  Value for thermocouple 8
 * @param PAGE_NUM  Page number for thermocouple data. Set to 0 if less than 8 thermocouples are having their data transmitted
 * @param PAGE_TOTAL  Total number of pages for thermocouple data. Set to 0 if less than 8 thermocouples are having their data transmitted
 * @param time_boot_ms [ms] Timestamp since boot
 */
#ifdef MAVLINK_USE_CONVENIENCE_FUNCTIONS

static inline void mavlink_msg_thermocouple_uor_send(mavlink_channel_t chan, uint32_t TC_1, uint32_t TC_2, uint32_t TC_3, uint32_t TC_4, uint32_t TC_5, uint32_t TC_6, uint32_t TC_7, uint32_t TC_8, uint8_t PAGE_NUM, uint8_t PAGE_TOTAL, uint32_t time_boot_ms)
{
#if MAVLINK_NEED_BYTE_SWAP || !MAVLINK_ALIGNED_FIELDS
    char buf[MAVLINK_MSG_ID_THERMOCOUPLE_UOR_LEN];
    _mav_put_uint32_t(buf, 0, TC_1);
    _mav_put_uint32_t(buf, 4, TC_2);
    _mav_put_uint32_t(buf, 8, TC_3);
    _mav_put_uint32_t(buf, 12, TC_4);
    _mav_put_uint32_t(buf, 16, TC_5);
    _mav_put_uint32_t(buf, 20, TC_6);
    _mav_put_uint32_t(buf, 24, TC_7);
    _mav_put_uint32_t(buf, 28, TC_8);
    _mav_put_uint32_t(buf, 32, time_boot_ms);
    _mav_put_uint8_t(buf, 36, PAGE_NUM);
    _mav_put_uint8_t(buf, 37, PAGE_TOTAL);

    _mav_finalize_message_chan_send(chan, MAVLINK_MSG_ID_THERMOCOUPLE_UOR, buf, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_MIN_LEN, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_LEN, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_CRC);
#else
    mavlink_thermocouple_uor_t packet;
    packet.TC_1 = TC_1;
    packet.TC_2 = TC_2;
    packet.TC_3 = TC_3;
    packet.TC_4 = TC_4;
    packet.TC_5 = TC_5;
    packet.TC_6 = TC_6;
    packet.TC_7 = TC_7;
    packet.TC_8 = TC_8;
    packet.time_boot_ms = time_boot_ms;
    packet.PAGE_NUM = PAGE_NUM;
    packet.PAGE_TOTAL = PAGE_TOTAL;

    _mav_finalize_message_chan_send(chan, MAVLINK_MSG_ID_THERMOCOUPLE_UOR, (const char *)&packet, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_MIN_LEN, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_LEN, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_CRC);
#endif
}

/**
 * @brief Send a thermocouple_uor message
 * @param chan MAVLink channel to send the message
 * @param struct The MAVLink struct to serialize
 */
static inline void mavlink_msg_thermocouple_uor_send_struct(mavlink_channel_t chan, const mavlink_thermocouple_uor_t* thermocouple_uor)
{
#if MAVLINK_NEED_BYTE_SWAP || !MAVLINK_ALIGNED_FIELDS
    mavlink_msg_thermocouple_uor_send(chan, thermocouple_uor->TC_1, thermocouple_uor->TC_2, thermocouple_uor->TC_3, thermocouple_uor->TC_4, thermocouple_uor->TC_5, thermocouple_uor->TC_6, thermocouple_uor->TC_7, thermocouple_uor->TC_8, thermocouple_uor->PAGE_NUM, thermocouple_uor->PAGE_TOTAL, thermocouple_uor->time_boot_ms);
#else
    _mav_finalize_message_chan_send(chan, MAVLINK_MSG_ID_THERMOCOUPLE_UOR, (const char *)thermocouple_uor, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_MIN_LEN, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_LEN, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_CRC);
#endif
}

#if MAVLINK_MSG_ID_THERMOCOUPLE_UOR_LEN <= MAVLINK_MAX_PAYLOAD_LEN
/*
  This variant of _send() can be used to save stack space by reusing
  memory from the receive buffer.  The caller provides a
  mavlink_message_t which is the size of a full mavlink message. This
  is usually the receive buffer for the channel, and allows a reply to an
  incoming message with minimum stack space usage.
 */
static inline void mavlink_msg_thermocouple_uor_send_buf(mavlink_message_t *msgbuf, mavlink_channel_t chan,  uint32_t TC_1, uint32_t TC_2, uint32_t TC_3, uint32_t TC_4, uint32_t TC_5, uint32_t TC_6, uint32_t TC_7, uint32_t TC_8, uint8_t PAGE_NUM, uint8_t PAGE_TOTAL, uint32_t time_boot_ms)
{
#if MAVLINK_NEED_BYTE_SWAP || !MAVLINK_ALIGNED_FIELDS
    char *buf = (char *)msgbuf;
    _mav_put_uint32_t(buf, 0, TC_1);
    _mav_put_uint32_t(buf, 4, TC_2);
    _mav_put_uint32_t(buf, 8, TC_3);
    _mav_put_uint32_t(buf, 12, TC_4);
    _mav_put_uint32_t(buf, 16, TC_5);
    _mav_put_uint32_t(buf, 20, TC_6);
    _mav_put_uint32_t(buf, 24, TC_7);
    _mav_put_uint32_t(buf, 28, TC_8);
    _mav_put_uint32_t(buf, 32, time_boot_ms);
    _mav_put_uint8_t(buf, 36, PAGE_NUM);
    _mav_put_uint8_t(buf, 37, PAGE_TOTAL);

    _mav_finalize_message_chan_send(chan, MAVLINK_MSG_ID_THERMOCOUPLE_UOR, buf, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_MIN_LEN, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_LEN, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_CRC);
#else
    mavlink_thermocouple_uor_t *packet = (mavlink_thermocouple_uor_t *)msgbuf;
    packet->TC_1 = TC_1;
    packet->TC_2 = TC_2;
    packet->TC_3 = TC_3;
    packet->TC_4 = TC_4;
    packet->TC_5 = TC_5;
    packet->TC_6 = TC_6;
    packet->TC_7 = TC_7;
    packet->TC_8 = TC_8;
    packet->time_boot_ms = time_boot_ms;
    packet->PAGE_NUM = PAGE_NUM;
    packet->PAGE_TOTAL = PAGE_TOTAL;

    _mav_finalize_message_chan_send(chan, MAVLINK_MSG_ID_THERMOCOUPLE_UOR, (const char *)packet, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_MIN_LEN, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_LEN, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_CRC);
#endif
}
#endif

#endif

// MESSAGE THERMOCOUPLE_UOR UNPACKING


/**
 * @brief Get field TC_1 from thermocouple_uor message
 *
 * @return  Value for thermocouple 1
 */
static inline uint32_t mavlink_msg_thermocouple_uor_get_TC_1(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint32_t(msg,  0);
}

/**
 * @brief Get field TC_2 from thermocouple_uor message
 *
 * @return  Value for thermocouple 2
 */
static inline uint32_t mavlink_msg_thermocouple_uor_get_TC_2(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint32_t(msg,  4);
}

/**
 * @brief Get field TC_3 from thermocouple_uor message
 *
 * @return  Value for thermocouple 3
 */
static inline uint32_t mavlink_msg_thermocouple_uor_get_TC_3(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint32_t(msg,  8);
}

/**
 * @brief Get field TC_4 from thermocouple_uor message
 *
 * @return  Value for thermocouple 4
 */
static inline uint32_t mavlink_msg_thermocouple_uor_get_TC_4(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint32_t(msg,  12);
}

/**
 * @brief Get field TC_5 from thermocouple_uor message
 *
 * @return  Value for thermocouple 5
 */
static inline uint32_t mavlink_msg_thermocouple_uor_get_TC_5(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint32_t(msg,  16);
}

/**
 * @brief Get field TC_6 from thermocouple_uor message
 *
 * @return  Value for thermocouple 6
 */
static inline uint32_t mavlink_msg_thermocouple_uor_get_TC_6(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint32_t(msg,  20);
}

/**
 * @brief Get field TC_7 from thermocouple_uor message
 *
 * @return  Value for thermocouple 7
 */
static inline uint32_t mavlink_msg_thermocouple_uor_get_TC_7(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint32_t(msg,  24);
}

/**
 * @brief Get field TC_8 from thermocouple_uor message
 *
 * @return  Value for thermocouple 8
 */
static inline uint32_t mavlink_msg_thermocouple_uor_get_TC_8(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint32_t(msg,  28);
}

/**
 * @brief Get field PAGE_NUM from thermocouple_uor message
 *
 * @return  Page number for thermocouple data. Set to 0 if less than 8 thermocouples are having their data transmitted
 */
static inline uint8_t mavlink_msg_thermocouple_uor_get_PAGE_NUM(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint8_t(msg,  36);
}

/**
 * @brief Get field PAGE_TOTAL from thermocouple_uor message
 *
 * @return  Total number of pages for thermocouple data. Set to 0 if less than 8 thermocouples are having their data transmitted
 */
static inline uint8_t mavlink_msg_thermocouple_uor_get_PAGE_TOTAL(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint8_t(msg,  37);
}

/**
 * @brief Get field time_boot_ms from thermocouple_uor message
 *
 * @return [ms] Timestamp since boot
 */
static inline uint32_t mavlink_msg_thermocouple_uor_get_time_boot_ms(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint32_t(msg,  32);
}

/**
 * @brief Decode a thermocouple_uor message into a struct
 *
 * @param msg The message to decode
 * @param thermocouple_uor C-struct to decode the message contents into
 */
static inline void mavlink_msg_thermocouple_uor_decode(const mavlink_message_t* msg, mavlink_thermocouple_uor_t* thermocouple_uor)
{
#if MAVLINK_NEED_BYTE_SWAP || !MAVLINK_ALIGNED_FIELDS
    thermocouple_uor->TC_1 = mavlink_msg_thermocouple_uor_get_TC_1(msg);
    thermocouple_uor->TC_2 = mavlink_msg_thermocouple_uor_get_TC_2(msg);
    thermocouple_uor->TC_3 = mavlink_msg_thermocouple_uor_get_TC_3(msg);
    thermocouple_uor->TC_4 = mavlink_msg_thermocouple_uor_get_TC_4(msg);
    thermocouple_uor->TC_5 = mavlink_msg_thermocouple_uor_get_TC_5(msg);
    thermocouple_uor->TC_6 = mavlink_msg_thermocouple_uor_get_TC_6(msg);
    thermocouple_uor->TC_7 = mavlink_msg_thermocouple_uor_get_TC_7(msg);
    thermocouple_uor->TC_8 = mavlink_msg_thermocouple_uor_get_TC_8(msg);
    thermocouple_uor->time_boot_ms = mavlink_msg_thermocouple_uor_get_time_boot_ms(msg);
    thermocouple_uor->PAGE_NUM = mavlink_msg_thermocouple_uor_get_PAGE_NUM(msg);
    thermocouple_uor->PAGE_TOTAL = mavlink_msg_thermocouple_uor_get_PAGE_TOTAL(msg);
#else
        uint8_t len = msg->len < MAVLINK_MSG_ID_THERMOCOUPLE_UOR_LEN? msg->len : MAVLINK_MSG_ID_THERMOCOUPLE_UOR_LEN;
        memset(thermocouple_uor, 0, MAVLINK_MSG_ID_THERMOCOUPLE_UOR_LEN);
    memcpy(thermocouple_uor, _MAV_PAYLOAD(msg), len);
#endif
}
