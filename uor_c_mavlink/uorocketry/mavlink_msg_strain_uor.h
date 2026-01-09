#pragma once
// MESSAGE STRAIN_UOR PACKING

#define MAVLINK_MSG_ID_STRAIN_UOR 60003


typedef struct __mavlink_strain_uor_t {
 uint32_t SG_1; /*<  Value for strain gauge 1*/
 uint32_t SG_2; /*<  Value for strain gauge 2*/
 uint32_t SG_3; /*<  Value for strain gauge 3*/
 uint32_t SG_4; /*<  Value for strain gauge 4*/
 uint32_t SG_5; /*<  Value for strain gauge 5*/
 uint32_t SG_6; /*<  Value for strain gauge 6*/
 uint32_t SG_7; /*<  Value for strain gauge 7*/
 uint32_t SG_8; /*<  Value for strain gauge 8*/
 uint32_t time_boot_ms; /*< [ms] Timestamp since boot*/
 uint8_t PAGE_NUM; /*<  Page number for strain gauge data. Set to 0 if less than 8 strain gauges are having their data transmitted*/
 uint8_t PAGE_TOTAL; /*<  Total number of pages for strain gauge data. Set to 0 if less than 8 strain gauges are having their data transmitted*/
} mavlink_strain_uor_t;

#define MAVLINK_MSG_ID_STRAIN_UOR_LEN 38
#define MAVLINK_MSG_ID_STRAIN_UOR_MIN_LEN 38
#define MAVLINK_MSG_ID_60003_LEN 38
#define MAVLINK_MSG_ID_60003_MIN_LEN 38

#define MAVLINK_MSG_ID_STRAIN_UOR_CRC 166
#define MAVLINK_MSG_ID_60003_CRC 166



#if MAVLINK_COMMAND_24BIT
#define MAVLINK_MESSAGE_INFO_STRAIN_UOR { \
    60003, \
    "STRAIN_UOR", \
    11, \
    {  { "SG_1", NULL, MAVLINK_TYPE_UINT32_T, 0, 0, offsetof(mavlink_strain_uor_t, SG_1) }, \
         { "SG_2", NULL, MAVLINK_TYPE_UINT32_T, 0, 4, offsetof(mavlink_strain_uor_t, SG_2) }, \
         { "SG_3", NULL, MAVLINK_TYPE_UINT32_T, 0, 8, offsetof(mavlink_strain_uor_t, SG_3) }, \
         { "SG_4", NULL, MAVLINK_TYPE_UINT32_T, 0, 12, offsetof(mavlink_strain_uor_t, SG_4) }, \
         { "SG_5", NULL, MAVLINK_TYPE_UINT32_T, 0, 16, offsetof(mavlink_strain_uor_t, SG_5) }, \
         { "SG_6", NULL, MAVLINK_TYPE_UINT32_T, 0, 20, offsetof(mavlink_strain_uor_t, SG_6) }, \
         { "SG_7", NULL, MAVLINK_TYPE_UINT32_T, 0, 24, offsetof(mavlink_strain_uor_t, SG_7) }, \
         { "SG_8", NULL, MAVLINK_TYPE_UINT32_T, 0, 28, offsetof(mavlink_strain_uor_t, SG_8) }, \
         { "PAGE_NUM", NULL, MAVLINK_TYPE_UINT8_T, 0, 36, offsetof(mavlink_strain_uor_t, PAGE_NUM) }, \
         { "PAGE_TOTAL", NULL, MAVLINK_TYPE_UINT8_T, 0, 37, offsetof(mavlink_strain_uor_t, PAGE_TOTAL) }, \
         { "time_boot_ms", NULL, MAVLINK_TYPE_UINT32_T, 0, 32, offsetof(mavlink_strain_uor_t, time_boot_ms) }, \
         } \
}
#else
#define MAVLINK_MESSAGE_INFO_STRAIN_UOR { \
    "STRAIN_UOR", \
    11, \
    {  { "SG_1", NULL, MAVLINK_TYPE_UINT32_T, 0, 0, offsetof(mavlink_strain_uor_t, SG_1) }, \
         { "SG_2", NULL, MAVLINK_TYPE_UINT32_T, 0, 4, offsetof(mavlink_strain_uor_t, SG_2) }, \
         { "SG_3", NULL, MAVLINK_TYPE_UINT32_T, 0, 8, offsetof(mavlink_strain_uor_t, SG_3) }, \
         { "SG_4", NULL, MAVLINK_TYPE_UINT32_T, 0, 12, offsetof(mavlink_strain_uor_t, SG_4) }, \
         { "SG_5", NULL, MAVLINK_TYPE_UINT32_T, 0, 16, offsetof(mavlink_strain_uor_t, SG_5) }, \
         { "SG_6", NULL, MAVLINK_TYPE_UINT32_T, 0, 20, offsetof(mavlink_strain_uor_t, SG_6) }, \
         { "SG_7", NULL, MAVLINK_TYPE_UINT32_T, 0, 24, offsetof(mavlink_strain_uor_t, SG_7) }, \
         { "SG_8", NULL, MAVLINK_TYPE_UINT32_T, 0, 28, offsetof(mavlink_strain_uor_t, SG_8) }, \
         { "PAGE_NUM", NULL, MAVLINK_TYPE_UINT8_T, 0, 36, offsetof(mavlink_strain_uor_t, PAGE_NUM) }, \
         { "PAGE_TOTAL", NULL, MAVLINK_TYPE_UINT8_T, 0, 37, offsetof(mavlink_strain_uor_t, PAGE_TOTAL) }, \
         { "time_boot_ms", NULL, MAVLINK_TYPE_UINT32_T, 0, 32, offsetof(mavlink_strain_uor_t, time_boot_ms) }, \
         } \
}
#endif

/**
 * @brief Pack a strain_uor message
 * @param system_id ID of this system
 * @param component_id ID of this component (e.g. 200 for IMU)
 * @param msg The MAVLink message to compress the data into
 *
 * @param SG_1  Value for strain gauge 1
 * @param SG_2  Value for strain gauge 2
 * @param SG_3  Value for strain gauge 3
 * @param SG_4  Value for strain gauge 4
 * @param SG_5  Value for strain gauge 5
 * @param SG_6  Value for strain gauge 6
 * @param SG_7  Value for strain gauge 7
 * @param SG_8  Value for strain gauge 8
 * @param PAGE_NUM  Page number for strain gauge data. Set to 0 if less than 8 strain gauges are having their data transmitted
 * @param PAGE_TOTAL  Total number of pages for strain gauge data. Set to 0 if less than 8 strain gauges are having their data transmitted
 * @param time_boot_ms [ms] Timestamp since boot
 * @return length of the message in bytes (excluding serial stream start sign)
 */
static inline uint16_t mavlink_msg_strain_uor_pack(uint8_t system_id, uint8_t component_id, mavlink_message_t* msg,
                               uint32_t SG_1, uint32_t SG_2, uint32_t SG_3, uint32_t SG_4, uint32_t SG_5, uint32_t SG_6, uint32_t SG_7, uint32_t SG_8, uint8_t PAGE_NUM, uint8_t PAGE_TOTAL, uint32_t time_boot_ms)
{
#if MAVLINK_NEED_BYTE_SWAP || !MAVLINK_ALIGNED_FIELDS
    char buf[MAVLINK_MSG_ID_STRAIN_UOR_LEN];
    _mav_put_uint32_t(buf, 0, SG_1);
    _mav_put_uint32_t(buf, 4, SG_2);
    _mav_put_uint32_t(buf, 8, SG_3);
    _mav_put_uint32_t(buf, 12, SG_4);
    _mav_put_uint32_t(buf, 16, SG_5);
    _mav_put_uint32_t(buf, 20, SG_6);
    _mav_put_uint32_t(buf, 24, SG_7);
    _mav_put_uint32_t(buf, 28, SG_8);
    _mav_put_uint32_t(buf, 32, time_boot_ms);
    _mav_put_uint8_t(buf, 36, PAGE_NUM);
    _mav_put_uint8_t(buf, 37, PAGE_TOTAL);

        memcpy(_MAV_PAYLOAD_NON_CONST(msg), buf, MAVLINK_MSG_ID_STRAIN_UOR_LEN);
#else
    mavlink_strain_uor_t packet;
    packet.SG_1 = SG_1;
    packet.SG_2 = SG_2;
    packet.SG_3 = SG_3;
    packet.SG_4 = SG_4;
    packet.SG_5 = SG_5;
    packet.SG_6 = SG_6;
    packet.SG_7 = SG_7;
    packet.SG_8 = SG_8;
    packet.time_boot_ms = time_boot_ms;
    packet.PAGE_NUM = PAGE_NUM;
    packet.PAGE_TOTAL = PAGE_TOTAL;

        memcpy(_MAV_PAYLOAD_NON_CONST(msg), &packet, MAVLINK_MSG_ID_STRAIN_UOR_LEN);
#endif

    msg->msgid = MAVLINK_MSG_ID_STRAIN_UOR;
    return mavlink_finalize_message(msg, system_id, component_id, MAVLINK_MSG_ID_STRAIN_UOR_MIN_LEN, MAVLINK_MSG_ID_STRAIN_UOR_LEN, MAVLINK_MSG_ID_STRAIN_UOR_CRC);
}

/**
 * @brief Pack a strain_uor message
 * @param system_id ID of this system
 * @param component_id ID of this component (e.g. 200 for IMU)
 * @param status MAVLink status structure
 * @param msg The MAVLink message to compress the data into
 *
 * @param SG_1  Value for strain gauge 1
 * @param SG_2  Value for strain gauge 2
 * @param SG_3  Value for strain gauge 3
 * @param SG_4  Value for strain gauge 4
 * @param SG_5  Value for strain gauge 5
 * @param SG_6  Value for strain gauge 6
 * @param SG_7  Value for strain gauge 7
 * @param SG_8  Value for strain gauge 8
 * @param PAGE_NUM  Page number for strain gauge data. Set to 0 if less than 8 strain gauges are having their data transmitted
 * @param PAGE_TOTAL  Total number of pages for strain gauge data. Set to 0 if less than 8 strain gauges are having their data transmitted
 * @param time_boot_ms [ms] Timestamp since boot
 * @return length of the message in bytes (excluding serial stream start sign)
 */
static inline uint16_t mavlink_msg_strain_uor_pack_status(uint8_t system_id, uint8_t component_id, mavlink_status_t *_status, mavlink_message_t* msg,
                               uint32_t SG_1, uint32_t SG_2, uint32_t SG_3, uint32_t SG_4, uint32_t SG_5, uint32_t SG_6, uint32_t SG_7, uint32_t SG_8, uint8_t PAGE_NUM, uint8_t PAGE_TOTAL, uint32_t time_boot_ms)
{
#if MAVLINK_NEED_BYTE_SWAP || !MAVLINK_ALIGNED_FIELDS
    char buf[MAVLINK_MSG_ID_STRAIN_UOR_LEN];
    _mav_put_uint32_t(buf, 0, SG_1);
    _mav_put_uint32_t(buf, 4, SG_2);
    _mav_put_uint32_t(buf, 8, SG_3);
    _mav_put_uint32_t(buf, 12, SG_4);
    _mav_put_uint32_t(buf, 16, SG_5);
    _mav_put_uint32_t(buf, 20, SG_6);
    _mav_put_uint32_t(buf, 24, SG_7);
    _mav_put_uint32_t(buf, 28, SG_8);
    _mav_put_uint32_t(buf, 32, time_boot_ms);
    _mav_put_uint8_t(buf, 36, PAGE_NUM);
    _mav_put_uint8_t(buf, 37, PAGE_TOTAL);

        memcpy(_MAV_PAYLOAD_NON_CONST(msg), buf, MAVLINK_MSG_ID_STRAIN_UOR_LEN);
#else
    mavlink_strain_uor_t packet;
    packet.SG_1 = SG_1;
    packet.SG_2 = SG_2;
    packet.SG_3 = SG_3;
    packet.SG_4 = SG_4;
    packet.SG_5 = SG_5;
    packet.SG_6 = SG_6;
    packet.SG_7 = SG_7;
    packet.SG_8 = SG_8;
    packet.time_boot_ms = time_boot_ms;
    packet.PAGE_NUM = PAGE_NUM;
    packet.PAGE_TOTAL = PAGE_TOTAL;

        memcpy(_MAV_PAYLOAD_NON_CONST(msg), &packet, MAVLINK_MSG_ID_STRAIN_UOR_LEN);
#endif

    msg->msgid = MAVLINK_MSG_ID_STRAIN_UOR;
#if MAVLINK_CRC_EXTRA
    return mavlink_finalize_message_buffer(msg, system_id, component_id, _status, MAVLINK_MSG_ID_STRAIN_UOR_MIN_LEN, MAVLINK_MSG_ID_STRAIN_UOR_LEN, MAVLINK_MSG_ID_STRAIN_UOR_CRC);
#else
    return mavlink_finalize_message_buffer(msg, system_id, component_id, _status, MAVLINK_MSG_ID_STRAIN_UOR_MIN_LEN, MAVLINK_MSG_ID_STRAIN_UOR_LEN);
#endif
}

/**
 * @brief Pack a strain_uor message on a channel
 * @param system_id ID of this system
 * @param component_id ID of this component (e.g. 200 for IMU)
 * @param chan The MAVLink channel this message will be sent over
 * @param msg The MAVLink message to compress the data into
 * @param SG_1  Value for strain gauge 1
 * @param SG_2  Value for strain gauge 2
 * @param SG_3  Value for strain gauge 3
 * @param SG_4  Value for strain gauge 4
 * @param SG_5  Value for strain gauge 5
 * @param SG_6  Value for strain gauge 6
 * @param SG_7  Value for strain gauge 7
 * @param SG_8  Value for strain gauge 8
 * @param PAGE_NUM  Page number for strain gauge data. Set to 0 if less than 8 strain gauges are having their data transmitted
 * @param PAGE_TOTAL  Total number of pages for strain gauge data. Set to 0 if less than 8 strain gauges are having their data transmitted
 * @param time_boot_ms [ms] Timestamp since boot
 * @return length of the message in bytes (excluding serial stream start sign)
 */
static inline uint16_t mavlink_msg_strain_uor_pack_chan(uint8_t system_id, uint8_t component_id, uint8_t chan,
                               mavlink_message_t* msg,
                                   uint32_t SG_1,uint32_t SG_2,uint32_t SG_3,uint32_t SG_4,uint32_t SG_5,uint32_t SG_6,uint32_t SG_7,uint32_t SG_8,uint8_t PAGE_NUM,uint8_t PAGE_TOTAL,uint32_t time_boot_ms)
{
#if MAVLINK_NEED_BYTE_SWAP || !MAVLINK_ALIGNED_FIELDS
    char buf[MAVLINK_MSG_ID_STRAIN_UOR_LEN];
    _mav_put_uint32_t(buf, 0, SG_1);
    _mav_put_uint32_t(buf, 4, SG_2);
    _mav_put_uint32_t(buf, 8, SG_3);
    _mav_put_uint32_t(buf, 12, SG_4);
    _mav_put_uint32_t(buf, 16, SG_5);
    _mav_put_uint32_t(buf, 20, SG_6);
    _mav_put_uint32_t(buf, 24, SG_7);
    _mav_put_uint32_t(buf, 28, SG_8);
    _mav_put_uint32_t(buf, 32, time_boot_ms);
    _mav_put_uint8_t(buf, 36, PAGE_NUM);
    _mav_put_uint8_t(buf, 37, PAGE_TOTAL);

        memcpy(_MAV_PAYLOAD_NON_CONST(msg), buf, MAVLINK_MSG_ID_STRAIN_UOR_LEN);
#else
    mavlink_strain_uor_t packet;
    packet.SG_1 = SG_1;
    packet.SG_2 = SG_2;
    packet.SG_3 = SG_3;
    packet.SG_4 = SG_4;
    packet.SG_5 = SG_5;
    packet.SG_6 = SG_6;
    packet.SG_7 = SG_7;
    packet.SG_8 = SG_8;
    packet.time_boot_ms = time_boot_ms;
    packet.PAGE_NUM = PAGE_NUM;
    packet.PAGE_TOTAL = PAGE_TOTAL;

        memcpy(_MAV_PAYLOAD_NON_CONST(msg), &packet, MAVLINK_MSG_ID_STRAIN_UOR_LEN);
#endif

    msg->msgid = MAVLINK_MSG_ID_STRAIN_UOR;
    return mavlink_finalize_message_chan(msg, system_id, component_id, chan, MAVLINK_MSG_ID_STRAIN_UOR_MIN_LEN, MAVLINK_MSG_ID_STRAIN_UOR_LEN, MAVLINK_MSG_ID_STRAIN_UOR_CRC);
}

/**
 * @brief Encode a strain_uor struct
 *
 * @param system_id ID of this system
 * @param component_id ID of this component (e.g. 200 for IMU)
 * @param msg The MAVLink message to compress the data into
 * @param strain_uor C-struct to read the message contents from
 */
static inline uint16_t mavlink_msg_strain_uor_encode(uint8_t system_id, uint8_t component_id, mavlink_message_t* msg, const mavlink_strain_uor_t* strain_uor)
{
    return mavlink_msg_strain_uor_pack(system_id, component_id, msg, strain_uor->SG_1, strain_uor->SG_2, strain_uor->SG_3, strain_uor->SG_4, strain_uor->SG_5, strain_uor->SG_6, strain_uor->SG_7, strain_uor->SG_8, strain_uor->PAGE_NUM, strain_uor->PAGE_TOTAL, strain_uor->time_boot_ms);
}

/**
 * @brief Encode a strain_uor struct on a channel
 *
 * @param system_id ID of this system
 * @param component_id ID of this component (e.g. 200 for IMU)
 * @param chan The MAVLink channel this message will be sent over
 * @param msg The MAVLink message to compress the data into
 * @param strain_uor C-struct to read the message contents from
 */
static inline uint16_t mavlink_msg_strain_uor_encode_chan(uint8_t system_id, uint8_t component_id, uint8_t chan, mavlink_message_t* msg, const mavlink_strain_uor_t* strain_uor)
{
    return mavlink_msg_strain_uor_pack_chan(system_id, component_id, chan, msg, strain_uor->SG_1, strain_uor->SG_2, strain_uor->SG_3, strain_uor->SG_4, strain_uor->SG_5, strain_uor->SG_6, strain_uor->SG_7, strain_uor->SG_8, strain_uor->PAGE_NUM, strain_uor->PAGE_TOTAL, strain_uor->time_boot_ms);
}

/**
 * @brief Encode a strain_uor struct with provided status structure
 *
 * @param system_id ID of this system
 * @param component_id ID of this component (e.g. 200 for IMU)
 * @param status MAVLink status structure
 * @param msg The MAVLink message to compress the data into
 * @param strain_uor C-struct to read the message contents from
 */
static inline uint16_t mavlink_msg_strain_uor_encode_status(uint8_t system_id, uint8_t component_id, mavlink_status_t* _status, mavlink_message_t* msg, const mavlink_strain_uor_t* strain_uor)
{
    return mavlink_msg_strain_uor_pack_status(system_id, component_id, _status, msg,  strain_uor->SG_1, strain_uor->SG_2, strain_uor->SG_3, strain_uor->SG_4, strain_uor->SG_5, strain_uor->SG_6, strain_uor->SG_7, strain_uor->SG_8, strain_uor->PAGE_NUM, strain_uor->PAGE_TOTAL, strain_uor->time_boot_ms);
}

/**
 * @brief Send a strain_uor message
 * @param chan MAVLink channel to send the message
 *
 * @param SG_1  Value for strain gauge 1
 * @param SG_2  Value for strain gauge 2
 * @param SG_3  Value for strain gauge 3
 * @param SG_4  Value for strain gauge 4
 * @param SG_5  Value for strain gauge 5
 * @param SG_6  Value for strain gauge 6
 * @param SG_7  Value for strain gauge 7
 * @param SG_8  Value for strain gauge 8
 * @param PAGE_NUM  Page number for strain gauge data. Set to 0 if less than 8 strain gauges are having their data transmitted
 * @param PAGE_TOTAL  Total number of pages for strain gauge data. Set to 0 if less than 8 strain gauges are having their data transmitted
 * @param time_boot_ms [ms] Timestamp since boot
 */
#ifdef MAVLINK_USE_CONVENIENCE_FUNCTIONS

static inline void mavlink_msg_strain_uor_send(mavlink_channel_t chan, uint32_t SG_1, uint32_t SG_2, uint32_t SG_3, uint32_t SG_4, uint32_t SG_5, uint32_t SG_6, uint32_t SG_7, uint32_t SG_8, uint8_t PAGE_NUM, uint8_t PAGE_TOTAL, uint32_t time_boot_ms)
{
#if MAVLINK_NEED_BYTE_SWAP || !MAVLINK_ALIGNED_FIELDS
    char buf[MAVLINK_MSG_ID_STRAIN_UOR_LEN];
    _mav_put_uint32_t(buf, 0, SG_1);
    _mav_put_uint32_t(buf, 4, SG_2);
    _mav_put_uint32_t(buf, 8, SG_3);
    _mav_put_uint32_t(buf, 12, SG_4);
    _mav_put_uint32_t(buf, 16, SG_5);
    _mav_put_uint32_t(buf, 20, SG_6);
    _mav_put_uint32_t(buf, 24, SG_7);
    _mav_put_uint32_t(buf, 28, SG_8);
    _mav_put_uint32_t(buf, 32, time_boot_ms);
    _mav_put_uint8_t(buf, 36, PAGE_NUM);
    _mav_put_uint8_t(buf, 37, PAGE_TOTAL);

    _mav_finalize_message_chan_send(chan, MAVLINK_MSG_ID_STRAIN_UOR, buf, MAVLINK_MSG_ID_STRAIN_UOR_MIN_LEN, MAVLINK_MSG_ID_STRAIN_UOR_LEN, MAVLINK_MSG_ID_STRAIN_UOR_CRC);
#else
    mavlink_strain_uor_t packet;
    packet.SG_1 = SG_1;
    packet.SG_2 = SG_2;
    packet.SG_3 = SG_3;
    packet.SG_4 = SG_4;
    packet.SG_5 = SG_5;
    packet.SG_6 = SG_6;
    packet.SG_7 = SG_7;
    packet.SG_8 = SG_8;
    packet.time_boot_ms = time_boot_ms;
    packet.PAGE_NUM = PAGE_NUM;
    packet.PAGE_TOTAL = PAGE_TOTAL;

    _mav_finalize_message_chan_send(chan, MAVLINK_MSG_ID_STRAIN_UOR, (const char *)&packet, MAVLINK_MSG_ID_STRAIN_UOR_MIN_LEN, MAVLINK_MSG_ID_STRAIN_UOR_LEN, MAVLINK_MSG_ID_STRAIN_UOR_CRC);
#endif
}

/**
 * @brief Send a strain_uor message
 * @param chan MAVLink channel to send the message
 * @param struct The MAVLink struct to serialize
 */
static inline void mavlink_msg_strain_uor_send_struct(mavlink_channel_t chan, const mavlink_strain_uor_t* strain_uor)
{
#if MAVLINK_NEED_BYTE_SWAP || !MAVLINK_ALIGNED_FIELDS
    mavlink_msg_strain_uor_send(chan, strain_uor->SG_1, strain_uor->SG_2, strain_uor->SG_3, strain_uor->SG_4, strain_uor->SG_5, strain_uor->SG_6, strain_uor->SG_7, strain_uor->SG_8, strain_uor->PAGE_NUM, strain_uor->PAGE_TOTAL, strain_uor->time_boot_ms);
#else
    _mav_finalize_message_chan_send(chan, MAVLINK_MSG_ID_STRAIN_UOR, (const char *)strain_uor, MAVLINK_MSG_ID_STRAIN_UOR_MIN_LEN, MAVLINK_MSG_ID_STRAIN_UOR_LEN, MAVLINK_MSG_ID_STRAIN_UOR_CRC);
#endif
}

#if MAVLINK_MSG_ID_STRAIN_UOR_LEN <= MAVLINK_MAX_PAYLOAD_LEN
/*
  This variant of _send() can be used to save stack space by reusing
  memory from the receive buffer.  The caller provides a
  mavlink_message_t which is the size of a full mavlink message. This
  is usually the receive buffer for the channel, and allows a reply to an
  incoming message with minimum stack space usage.
 */
static inline void mavlink_msg_strain_uor_send_buf(mavlink_message_t *msgbuf, mavlink_channel_t chan,  uint32_t SG_1, uint32_t SG_2, uint32_t SG_3, uint32_t SG_4, uint32_t SG_5, uint32_t SG_6, uint32_t SG_7, uint32_t SG_8, uint8_t PAGE_NUM, uint8_t PAGE_TOTAL, uint32_t time_boot_ms)
{
#if MAVLINK_NEED_BYTE_SWAP || !MAVLINK_ALIGNED_FIELDS
    char *buf = (char *)msgbuf;
    _mav_put_uint32_t(buf, 0, SG_1);
    _mav_put_uint32_t(buf, 4, SG_2);
    _mav_put_uint32_t(buf, 8, SG_3);
    _mav_put_uint32_t(buf, 12, SG_4);
    _mav_put_uint32_t(buf, 16, SG_5);
    _mav_put_uint32_t(buf, 20, SG_6);
    _mav_put_uint32_t(buf, 24, SG_7);
    _mav_put_uint32_t(buf, 28, SG_8);
    _mav_put_uint32_t(buf, 32, time_boot_ms);
    _mav_put_uint8_t(buf, 36, PAGE_NUM);
    _mav_put_uint8_t(buf, 37, PAGE_TOTAL);

    _mav_finalize_message_chan_send(chan, MAVLINK_MSG_ID_STRAIN_UOR, buf, MAVLINK_MSG_ID_STRAIN_UOR_MIN_LEN, MAVLINK_MSG_ID_STRAIN_UOR_LEN, MAVLINK_MSG_ID_STRAIN_UOR_CRC);
#else
    mavlink_strain_uor_t *packet = (mavlink_strain_uor_t *)msgbuf;
    packet->SG_1 = SG_1;
    packet->SG_2 = SG_2;
    packet->SG_3 = SG_3;
    packet->SG_4 = SG_4;
    packet->SG_5 = SG_5;
    packet->SG_6 = SG_6;
    packet->SG_7 = SG_7;
    packet->SG_8 = SG_8;
    packet->time_boot_ms = time_boot_ms;
    packet->PAGE_NUM = PAGE_NUM;
    packet->PAGE_TOTAL = PAGE_TOTAL;

    _mav_finalize_message_chan_send(chan, MAVLINK_MSG_ID_STRAIN_UOR, (const char *)packet, MAVLINK_MSG_ID_STRAIN_UOR_MIN_LEN, MAVLINK_MSG_ID_STRAIN_UOR_LEN, MAVLINK_MSG_ID_STRAIN_UOR_CRC);
#endif
}
#endif

#endif

// MESSAGE STRAIN_UOR UNPACKING


/**
 * @brief Get field SG_1 from strain_uor message
 *
 * @return  Value for strain gauge 1
 */
static inline uint32_t mavlink_msg_strain_uor_get_SG_1(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint32_t(msg,  0);
}

/**
 * @brief Get field SG_2 from strain_uor message
 *
 * @return  Value for strain gauge 2
 */
static inline uint32_t mavlink_msg_strain_uor_get_SG_2(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint32_t(msg,  4);
}

/**
 * @brief Get field SG_3 from strain_uor message
 *
 * @return  Value for strain gauge 3
 */
static inline uint32_t mavlink_msg_strain_uor_get_SG_3(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint32_t(msg,  8);
}

/**
 * @brief Get field SG_4 from strain_uor message
 *
 * @return  Value for strain gauge 4
 */
static inline uint32_t mavlink_msg_strain_uor_get_SG_4(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint32_t(msg,  12);
}

/**
 * @brief Get field SG_5 from strain_uor message
 *
 * @return  Value for strain gauge 5
 */
static inline uint32_t mavlink_msg_strain_uor_get_SG_5(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint32_t(msg,  16);
}

/**
 * @brief Get field SG_6 from strain_uor message
 *
 * @return  Value for strain gauge 6
 */
static inline uint32_t mavlink_msg_strain_uor_get_SG_6(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint32_t(msg,  20);
}

/**
 * @brief Get field SG_7 from strain_uor message
 *
 * @return  Value for strain gauge 7
 */
static inline uint32_t mavlink_msg_strain_uor_get_SG_7(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint32_t(msg,  24);
}

/**
 * @brief Get field SG_8 from strain_uor message
 *
 * @return  Value for strain gauge 8
 */
static inline uint32_t mavlink_msg_strain_uor_get_SG_8(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint32_t(msg,  28);
}

/**
 * @brief Get field PAGE_NUM from strain_uor message
 *
 * @return  Page number for strain gauge data. Set to 0 if less than 8 strain gauges are having their data transmitted
 */
static inline uint8_t mavlink_msg_strain_uor_get_PAGE_NUM(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint8_t(msg,  36);
}

/**
 * @brief Get field PAGE_TOTAL from strain_uor message
 *
 * @return  Total number of pages for strain gauge data. Set to 0 if less than 8 strain gauges are having their data transmitted
 */
static inline uint8_t mavlink_msg_strain_uor_get_PAGE_TOTAL(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint8_t(msg,  37);
}

/**
 * @brief Get field time_boot_ms from strain_uor message
 *
 * @return [ms] Timestamp since boot
 */
static inline uint32_t mavlink_msg_strain_uor_get_time_boot_ms(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint32_t(msg,  32);
}

/**
 * @brief Decode a strain_uor message into a struct
 *
 * @param msg The message to decode
 * @param strain_uor C-struct to decode the message contents into
 */
static inline void mavlink_msg_strain_uor_decode(const mavlink_message_t* msg, mavlink_strain_uor_t* strain_uor)
{
#if MAVLINK_NEED_BYTE_SWAP || !MAVLINK_ALIGNED_FIELDS
    strain_uor->SG_1 = mavlink_msg_strain_uor_get_SG_1(msg);
    strain_uor->SG_2 = mavlink_msg_strain_uor_get_SG_2(msg);
    strain_uor->SG_3 = mavlink_msg_strain_uor_get_SG_3(msg);
    strain_uor->SG_4 = mavlink_msg_strain_uor_get_SG_4(msg);
    strain_uor->SG_5 = mavlink_msg_strain_uor_get_SG_5(msg);
    strain_uor->SG_6 = mavlink_msg_strain_uor_get_SG_6(msg);
    strain_uor->SG_7 = mavlink_msg_strain_uor_get_SG_7(msg);
    strain_uor->SG_8 = mavlink_msg_strain_uor_get_SG_8(msg);
    strain_uor->time_boot_ms = mavlink_msg_strain_uor_get_time_boot_ms(msg);
    strain_uor->PAGE_NUM = mavlink_msg_strain_uor_get_PAGE_NUM(msg);
    strain_uor->PAGE_TOTAL = mavlink_msg_strain_uor_get_PAGE_TOTAL(msg);
#else
        uint8_t len = msg->len < MAVLINK_MSG_ID_STRAIN_UOR_LEN? msg->len : MAVLINK_MSG_ID_STRAIN_UOR_LEN;
        memset(strain_uor, 0, MAVLINK_MSG_ID_STRAIN_UOR_LEN);
    memcpy(strain_uor, _MAV_PAYLOAD(msg), len);
#endif
}
