#pragma once
// MESSAGE PRESSURE_UOR PACKING

#define MAVLINK_MSG_ID_PRESSURE_UOR 60002


typedef struct __mavlink_pressure_uor_t {
 uint32_t PS_1; /*<  Value for pressure sensor 1*/
 uint32_t PS_2; /*<  Value for pressure sensor 2*/
 uint32_t PS_3; /*<  Value for pressure sensor 3*/
 uint32_t PS_4; /*<  Value for pressure sensor 4*/
 uint32_t PS_5; /*<  Value for pressure sensor 5*/
 uint32_t PS_6; /*<  Value for pressure sensor 6*/
 uint32_t PS_7; /*<  Value for pressure sensor 7*/
 uint32_t PS_8; /*<  Value for pressure sensor 8*/
 uint32_t time_boot_ms; /*< [ms] Timestamp since boot*/
 uint8_t PAGE_NUM; /*<  Page number for pressure data. Set to 0 if less than 8 pressure sensors are having their data transmitted*/
 uint8_t PAGE_TOTAL; /*<  Total number of pages for pressure data. Set to 0 if less than 8 pressure sensors are having their data transmitted*/
} mavlink_pressure_uor_t;

#define MAVLINK_MSG_ID_PRESSURE_UOR_LEN 38
#define MAVLINK_MSG_ID_PRESSURE_UOR_MIN_LEN 38
#define MAVLINK_MSG_ID_60002_LEN 38
#define MAVLINK_MSG_ID_60002_MIN_LEN 38

#define MAVLINK_MSG_ID_PRESSURE_UOR_CRC 194
#define MAVLINK_MSG_ID_60002_CRC 194



#if MAVLINK_COMMAND_24BIT
#define MAVLINK_MESSAGE_INFO_PRESSURE_UOR { \
    60002, \
    "PRESSURE_UOR", \
    11, \
    {  { "PS_1", NULL, MAVLINK_TYPE_UINT32_T, 0, 0, offsetof(mavlink_pressure_uor_t, PS_1) }, \
         { "PS_2", NULL, MAVLINK_TYPE_UINT32_T, 0, 4, offsetof(mavlink_pressure_uor_t, PS_2) }, \
         { "PS_3", NULL, MAVLINK_TYPE_UINT32_T, 0, 8, offsetof(mavlink_pressure_uor_t, PS_3) }, \
         { "PS_4", NULL, MAVLINK_TYPE_UINT32_T, 0, 12, offsetof(mavlink_pressure_uor_t, PS_4) }, \
         { "PS_5", NULL, MAVLINK_TYPE_UINT32_T, 0, 16, offsetof(mavlink_pressure_uor_t, PS_5) }, \
         { "PS_6", NULL, MAVLINK_TYPE_UINT32_T, 0, 20, offsetof(mavlink_pressure_uor_t, PS_6) }, \
         { "PS_7", NULL, MAVLINK_TYPE_UINT32_T, 0, 24, offsetof(mavlink_pressure_uor_t, PS_7) }, \
         { "PS_8", NULL, MAVLINK_TYPE_UINT32_T, 0, 28, offsetof(mavlink_pressure_uor_t, PS_8) }, \
         { "PAGE_NUM", NULL, MAVLINK_TYPE_UINT8_T, 0, 36, offsetof(mavlink_pressure_uor_t, PAGE_NUM) }, \
         { "PAGE_TOTAL", NULL, MAVLINK_TYPE_UINT8_T, 0, 37, offsetof(mavlink_pressure_uor_t, PAGE_TOTAL) }, \
         { "time_boot_ms", NULL, MAVLINK_TYPE_UINT32_T, 0, 32, offsetof(mavlink_pressure_uor_t, time_boot_ms) }, \
         } \
}
#else
#define MAVLINK_MESSAGE_INFO_PRESSURE_UOR { \
    "PRESSURE_UOR", \
    11, \
    {  { "PS_1", NULL, MAVLINK_TYPE_UINT32_T, 0, 0, offsetof(mavlink_pressure_uor_t, PS_1) }, \
         { "PS_2", NULL, MAVLINK_TYPE_UINT32_T, 0, 4, offsetof(mavlink_pressure_uor_t, PS_2) }, \
         { "PS_3", NULL, MAVLINK_TYPE_UINT32_T, 0, 8, offsetof(mavlink_pressure_uor_t, PS_3) }, \
         { "PS_4", NULL, MAVLINK_TYPE_UINT32_T, 0, 12, offsetof(mavlink_pressure_uor_t, PS_4) }, \
         { "PS_5", NULL, MAVLINK_TYPE_UINT32_T, 0, 16, offsetof(mavlink_pressure_uor_t, PS_5) }, \
         { "PS_6", NULL, MAVLINK_TYPE_UINT32_T, 0, 20, offsetof(mavlink_pressure_uor_t, PS_6) }, \
         { "PS_7", NULL, MAVLINK_TYPE_UINT32_T, 0, 24, offsetof(mavlink_pressure_uor_t, PS_7) }, \
         { "PS_8", NULL, MAVLINK_TYPE_UINT32_T, 0, 28, offsetof(mavlink_pressure_uor_t, PS_8) }, \
         { "PAGE_NUM", NULL, MAVLINK_TYPE_UINT8_T, 0, 36, offsetof(mavlink_pressure_uor_t, PAGE_NUM) }, \
         { "PAGE_TOTAL", NULL, MAVLINK_TYPE_UINT8_T, 0, 37, offsetof(mavlink_pressure_uor_t, PAGE_TOTAL) }, \
         { "time_boot_ms", NULL, MAVLINK_TYPE_UINT32_T, 0, 32, offsetof(mavlink_pressure_uor_t, time_boot_ms) }, \
         } \
}
#endif

/**
 * @brief Pack a pressure_uor message
 * @param system_id ID of this system
 * @param component_id ID of this component (e.g. 200 for IMU)
 * @param msg The MAVLink message to compress the data into
 *
 * @param PS_1  Value for pressure sensor 1
 * @param PS_2  Value for pressure sensor 2
 * @param PS_3  Value for pressure sensor 3
 * @param PS_4  Value for pressure sensor 4
 * @param PS_5  Value for pressure sensor 5
 * @param PS_6  Value for pressure sensor 6
 * @param PS_7  Value for pressure sensor 7
 * @param PS_8  Value for pressure sensor 8
 * @param PAGE_NUM  Page number for pressure data. Set to 0 if less than 8 pressure sensors are having their data transmitted
 * @param PAGE_TOTAL  Total number of pages for pressure data. Set to 0 if less than 8 pressure sensors are having their data transmitted
 * @param time_boot_ms [ms] Timestamp since boot
 * @return length of the message in bytes (excluding serial stream start sign)
 */
static inline uint16_t mavlink_msg_pressure_uor_pack(uint8_t system_id, uint8_t component_id, mavlink_message_t* msg,
                               uint32_t PS_1, uint32_t PS_2, uint32_t PS_3, uint32_t PS_4, uint32_t PS_5, uint32_t PS_6, uint32_t PS_7, uint32_t PS_8, uint8_t PAGE_NUM, uint8_t PAGE_TOTAL, uint32_t time_boot_ms)
{
#if MAVLINK_NEED_BYTE_SWAP || !MAVLINK_ALIGNED_FIELDS
    char buf[MAVLINK_MSG_ID_PRESSURE_UOR_LEN];
    _mav_put_uint32_t(buf, 0, PS_1);
    _mav_put_uint32_t(buf, 4, PS_2);
    _mav_put_uint32_t(buf, 8, PS_3);
    _mav_put_uint32_t(buf, 12, PS_4);
    _mav_put_uint32_t(buf, 16, PS_5);
    _mav_put_uint32_t(buf, 20, PS_6);
    _mav_put_uint32_t(buf, 24, PS_7);
    _mav_put_uint32_t(buf, 28, PS_8);
    _mav_put_uint32_t(buf, 32, time_boot_ms);
    _mav_put_uint8_t(buf, 36, PAGE_NUM);
    _mav_put_uint8_t(buf, 37, PAGE_TOTAL);

        memcpy(_MAV_PAYLOAD_NON_CONST(msg), buf, MAVLINK_MSG_ID_PRESSURE_UOR_LEN);
#else
    mavlink_pressure_uor_t packet;
    packet.PS_1 = PS_1;
    packet.PS_2 = PS_2;
    packet.PS_3 = PS_3;
    packet.PS_4 = PS_4;
    packet.PS_5 = PS_5;
    packet.PS_6 = PS_6;
    packet.PS_7 = PS_7;
    packet.PS_8 = PS_8;
    packet.time_boot_ms = time_boot_ms;
    packet.PAGE_NUM = PAGE_NUM;
    packet.PAGE_TOTAL = PAGE_TOTAL;

        memcpy(_MAV_PAYLOAD_NON_CONST(msg), &packet, MAVLINK_MSG_ID_PRESSURE_UOR_LEN);
#endif

    msg->msgid = MAVLINK_MSG_ID_PRESSURE_UOR;
    return mavlink_finalize_message(msg, system_id, component_id, MAVLINK_MSG_ID_PRESSURE_UOR_MIN_LEN, MAVLINK_MSG_ID_PRESSURE_UOR_LEN, MAVLINK_MSG_ID_PRESSURE_UOR_CRC);
}

/**
 * @brief Pack a pressure_uor message
 * @param system_id ID of this system
 * @param component_id ID of this component (e.g. 200 for IMU)
 * @param status MAVLink status structure
 * @param msg The MAVLink message to compress the data into
 *
 * @param PS_1  Value for pressure sensor 1
 * @param PS_2  Value for pressure sensor 2
 * @param PS_3  Value for pressure sensor 3
 * @param PS_4  Value for pressure sensor 4
 * @param PS_5  Value for pressure sensor 5
 * @param PS_6  Value for pressure sensor 6
 * @param PS_7  Value for pressure sensor 7
 * @param PS_8  Value for pressure sensor 8
 * @param PAGE_NUM  Page number for pressure data. Set to 0 if less than 8 pressure sensors are having their data transmitted
 * @param PAGE_TOTAL  Total number of pages for pressure data. Set to 0 if less than 8 pressure sensors are having their data transmitted
 * @param time_boot_ms [ms] Timestamp since boot
 * @return length of the message in bytes (excluding serial stream start sign)
 */
static inline uint16_t mavlink_msg_pressure_uor_pack_status(uint8_t system_id, uint8_t component_id, mavlink_status_t *_status, mavlink_message_t* msg,
                               uint32_t PS_1, uint32_t PS_2, uint32_t PS_3, uint32_t PS_4, uint32_t PS_5, uint32_t PS_6, uint32_t PS_7, uint32_t PS_8, uint8_t PAGE_NUM, uint8_t PAGE_TOTAL, uint32_t time_boot_ms)
{
#if MAVLINK_NEED_BYTE_SWAP || !MAVLINK_ALIGNED_FIELDS
    char buf[MAVLINK_MSG_ID_PRESSURE_UOR_LEN];
    _mav_put_uint32_t(buf, 0, PS_1);
    _mav_put_uint32_t(buf, 4, PS_2);
    _mav_put_uint32_t(buf, 8, PS_3);
    _mav_put_uint32_t(buf, 12, PS_4);
    _mav_put_uint32_t(buf, 16, PS_5);
    _mav_put_uint32_t(buf, 20, PS_6);
    _mav_put_uint32_t(buf, 24, PS_7);
    _mav_put_uint32_t(buf, 28, PS_8);
    _mav_put_uint32_t(buf, 32, time_boot_ms);
    _mav_put_uint8_t(buf, 36, PAGE_NUM);
    _mav_put_uint8_t(buf, 37, PAGE_TOTAL);

        memcpy(_MAV_PAYLOAD_NON_CONST(msg), buf, MAVLINK_MSG_ID_PRESSURE_UOR_LEN);
#else
    mavlink_pressure_uor_t packet;
    packet.PS_1 = PS_1;
    packet.PS_2 = PS_2;
    packet.PS_3 = PS_3;
    packet.PS_4 = PS_4;
    packet.PS_5 = PS_5;
    packet.PS_6 = PS_6;
    packet.PS_7 = PS_7;
    packet.PS_8 = PS_8;
    packet.time_boot_ms = time_boot_ms;
    packet.PAGE_NUM = PAGE_NUM;
    packet.PAGE_TOTAL = PAGE_TOTAL;

        memcpy(_MAV_PAYLOAD_NON_CONST(msg), &packet, MAVLINK_MSG_ID_PRESSURE_UOR_LEN);
#endif

    msg->msgid = MAVLINK_MSG_ID_PRESSURE_UOR;
#if MAVLINK_CRC_EXTRA
    return mavlink_finalize_message_buffer(msg, system_id, component_id, _status, MAVLINK_MSG_ID_PRESSURE_UOR_MIN_LEN, MAVLINK_MSG_ID_PRESSURE_UOR_LEN, MAVLINK_MSG_ID_PRESSURE_UOR_CRC);
#else
    return mavlink_finalize_message_buffer(msg, system_id, component_id, _status, MAVLINK_MSG_ID_PRESSURE_UOR_MIN_LEN, MAVLINK_MSG_ID_PRESSURE_UOR_LEN);
#endif
}

/**
 * @brief Pack a pressure_uor message on a channel
 * @param system_id ID of this system
 * @param component_id ID of this component (e.g. 200 for IMU)
 * @param chan The MAVLink channel this message will be sent over
 * @param msg The MAVLink message to compress the data into
 * @param PS_1  Value for pressure sensor 1
 * @param PS_2  Value for pressure sensor 2
 * @param PS_3  Value for pressure sensor 3
 * @param PS_4  Value for pressure sensor 4
 * @param PS_5  Value for pressure sensor 5
 * @param PS_6  Value for pressure sensor 6
 * @param PS_7  Value for pressure sensor 7
 * @param PS_8  Value for pressure sensor 8
 * @param PAGE_NUM  Page number for pressure data. Set to 0 if less than 8 pressure sensors are having their data transmitted
 * @param PAGE_TOTAL  Total number of pages for pressure data. Set to 0 if less than 8 pressure sensors are having their data transmitted
 * @param time_boot_ms [ms] Timestamp since boot
 * @return length of the message in bytes (excluding serial stream start sign)
 */
static inline uint16_t mavlink_msg_pressure_uor_pack_chan(uint8_t system_id, uint8_t component_id, uint8_t chan,
                               mavlink_message_t* msg,
                                   uint32_t PS_1,uint32_t PS_2,uint32_t PS_3,uint32_t PS_4,uint32_t PS_5,uint32_t PS_6,uint32_t PS_7,uint32_t PS_8,uint8_t PAGE_NUM,uint8_t PAGE_TOTAL,uint32_t time_boot_ms)
{
#if MAVLINK_NEED_BYTE_SWAP || !MAVLINK_ALIGNED_FIELDS
    char buf[MAVLINK_MSG_ID_PRESSURE_UOR_LEN];
    _mav_put_uint32_t(buf, 0, PS_1);
    _mav_put_uint32_t(buf, 4, PS_2);
    _mav_put_uint32_t(buf, 8, PS_3);
    _mav_put_uint32_t(buf, 12, PS_4);
    _mav_put_uint32_t(buf, 16, PS_5);
    _mav_put_uint32_t(buf, 20, PS_6);
    _mav_put_uint32_t(buf, 24, PS_7);
    _mav_put_uint32_t(buf, 28, PS_8);
    _mav_put_uint32_t(buf, 32, time_boot_ms);
    _mav_put_uint8_t(buf, 36, PAGE_NUM);
    _mav_put_uint8_t(buf, 37, PAGE_TOTAL);

        memcpy(_MAV_PAYLOAD_NON_CONST(msg), buf, MAVLINK_MSG_ID_PRESSURE_UOR_LEN);
#else
    mavlink_pressure_uor_t packet;
    packet.PS_1 = PS_1;
    packet.PS_2 = PS_2;
    packet.PS_3 = PS_3;
    packet.PS_4 = PS_4;
    packet.PS_5 = PS_5;
    packet.PS_6 = PS_6;
    packet.PS_7 = PS_7;
    packet.PS_8 = PS_8;
    packet.time_boot_ms = time_boot_ms;
    packet.PAGE_NUM = PAGE_NUM;
    packet.PAGE_TOTAL = PAGE_TOTAL;

        memcpy(_MAV_PAYLOAD_NON_CONST(msg), &packet, MAVLINK_MSG_ID_PRESSURE_UOR_LEN);
#endif

    msg->msgid = MAVLINK_MSG_ID_PRESSURE_UOR;
    return mavlink_finalize_message_chan(msg, system_id, component_id, chan, MAVLINK_MSG_ID_PRESSURE_UOR_MIN_LEN, MAVLINK_MSG_ID_PRESSURE_UOR_LEN, MAVLINK_MSG_ID_PRESSURE_UOR_CRC);
}

/**
 * @brief Encode a pressure_uor struct
 *
 * @param system_id ID of this system
 * @param component_id ID of this component (e.g. 200 for IMU)
 * @param msg The MAVLink message to compress the data into
 * @param pressure_uor C-struct to read the message contents from
 */
static inline uint16_t mavlink_msg_pressure_uor_encode(uint8_t system_id, uint8_t component_id, mavlink_message_t* msg, const mavlink_pressure_uor_t* pressure_uor)
{
    return mavlink_msg_pressure_uor_pack(system_id, component_id, msg, pressure_uor->PS_1, pressure_uor->PS_2, pressure_uor->PS_3, pressure_uor->PS_4, pressure_uor->PS_5, pressure_uor->PS_6, pressure_uor->PS_7, pressure_uor->PS_8, pressure_uor->PAGE_NUM, pressure_uor->PAGE_TOTAL, pressure_uor->time_boot_ms);
}

/**
 * @brief Encode a pressure_uor struct on a channel
 *
 * @param system_id ID of this system
 * @param component_id ID of this component (e.g. 200 for IMU)
 * @param chan The MAVLink channel this message will be sent over
 * @param msg The MAVLink message to compress the data into
 * @param pressure_uor C-struct to read the message contents from
 */
static inline uint16_t mavlink_msg_pressure_uor_encode_chan(uint8_t system_id, uint8_t component_id, uint8_t chan, mavlink_message_t* msg, const mavlink_pressure_uor_t* pressure_uor)
{
    return mavlink_msg_pressure_uor_pack_chan(system_id, component_id, chan, msg, pressure_uor->PS_1, pressure_uor->PS_2, pressure_uor->PS_3, pressure_uor->PS_4, pressure_uor->PS_5, pressure_uor->PS_6, pressure_uor->PS_7, pressure_uor->PS_8, pressure_uor->PAGE_NUM, pressure_uor->PAGE_TOTAL, pressure_uor->time_boot_ms);
}

/**
 * @brief Encode a pressure_uor struct with provided status structure
 *
 * @param system_id ID of this system
 * @param component_id ID of this component (e.g. 200 for IMU)
 * @param status MAVLink status structure
 * @param msg The MAVLink message to compress the data into
 * @param pressure_uor C-struct to read the message contents from
 */
static inline uint16_t mavlink_msg_pressure_uor_encode_status(uint8_t system_id, uint8_t component_id, mavlink_status_t* _status, mavlink_message_t* msg, const mavlink_pressure_uor_t* pressure_uor)
{
    return mavlink_msg_pressure_uor_pack_status(system_id, component_id, _status, msg,  pressure_uor->PS_1, pressure_uor->PS_2, pressure_uor->PS_3, pressure_uor->PS_4, pressure_uor->PS_5, pressure_uor->PS_6, pressure_uor->PS_7, pressure_uor->PS_8, pressure_uor->PAGE_NUM, pressure_uor->PAGE_TOTAL, pressure_uor->time_boot_ms);
}

/**
 * @brief Send a pressure_uor message
 * @param chan MAVLink channel to send the message
 *
 * @param PS_1  Value for pressure sensor 1
 * @param PS_2  Value for pressure sensor 2
 * @param PS_3  Value for pressure sensor 3
 * @param PS_4  Value for pressure sensor 4
 * @param PS_5  Value for pressure sensor 5
 * @param PS_6  Value for pressure sensor 6
 * @param PS_7  Value for pressure sensor 7
 * @param PS_8  Value for pressure sensor 8
 * @param PAGE_NUM  Page number for pressure data. Set to 0 if less than 8 pressure sensors are having their data transmitted
 * @param PAGE_TOTAL  Total number of pages for pressure data. Set to 0 if less than 8 pressure sensors are having their data transmitted
 * @param time_boot_ms [ms] Timestamp since boot
 */
#ifdef MAVLINK_USE_CONVENIENCE_FUNCTIONS

static inline void mavlink_msg_pressure_uor_send(mavlink_channel_t chan, uint32_t PS_1, uint32_t PS_2, uint32_t PS_3, uint32_t PS_4, uint32_t PS_5, uint32_t PS_6, uint32_t PS_7, uint32_t PS_8, uint8_t PAGE_NUM, uint8_t PAGE_TOTAL, uint32_t time_boot_ms)
{
#if MAVLINK_NEED_BYTE_SWAP || !MAVLINK_ALIGNED_FIELDS
    char buf[MAVLINK_MSG_ID_PRESSURE_UOR_LEN];
    _mav_put_uint32_t(buf, 0, PS_1);
    _mav_put_uint32_t(buf, 4, PS_2);
    _mav_put_uint32_t(buf, 8, PS_3);
    _mav_put_uint32_t(buf, 12, PS_4);
    _mav_put_uint32_t(buf, 16, PS_5);
    _mav_put_uint32_t(buf, 20, PS_6);
    _mav_put_uint32_t(buf, 24, PS_7);
    _mav_put_uint32_t(buf, 28, PS_8);
    _mav_put_uint32_t(buf, 32, time_boot_ms);
    _mav_put_uint8_t(buf, 36, PAGE_NUM);
    _mav_put_uint8_t(buf, 37, PAGE_TOTAL);

    _mav_finalize_message_chan_send(chan, MAVLINK_MSG_ID_PRESSURE_UOR, buf, MAVLINK_MSG_ID_PRESSURE_UOR_MIN_LEN, MAVLINK_MSG_ID_PRESSURE_UOR_LEN, MAVLINK_MSG_ID_PRESSURE_UOR_CRC);
#else
    mavlink_pressure_uor_t packet;
    packet.PS_1 = PS_1;
    packet.PS_2 = PS_2;
    packet.PS_3 = PS_3;
    packet.PS_4 = PS_4;
    packet.PS_5 = PS_5;
    packet.PS_6 = PS_6;
    packet.PS_7 = PS_7;
    packet.PS_8 = PS_8;
    packet.time_boot_ms = time_boot_ms;
    packet.PAGE_NUM = PAGE_NUM;
    packet.PAGE_TOTAL = PAGE_TOTAL;

    _mav_finalize_message_chan_send(chan, MAVLINK_MSG_ID_PRESSURE_UOR, (const char *)&packet, MAVLINK_MSG_ID_PRESSURE_UOR_MIN_LEN, MAVLINK_MSG_ID_PRESSURE_UOR_LEN, MAVLINK_MSG_ID_PRESSURE_UOR_CRC);
#endif
}

/**
 * @brief Send a pressure_uor message
 * @param chan MAVLink channel to send the message
 * @param struct The MAVLink struct to serialize
 */
static inline void mavlink_msg_pressure_uor_send_struct(mavlink_channel_t chan, const mavlink_pressure_uor_t* pressure_uor)
{
#if MAVLINK_NEED_BYTE_SWAP || !MAVLINK_ALIGNED_FIELDS
    mavlink_msg_pressure_uor_send(chan, pressure_uor->PS_1, pressure_uor->PS_2, pressure_uor->PS_3, pressure_uor->PS_4, pressure_uor->PS_5, pressure_uor->PS_6, pressure_uor->PS_7, pressure_uor->PS_8, pressure_uor->PAGE_NUM, pressure_uor->PAGE_TOTAL, pressure_uor->time_boot_ms);
#else
    _mav_finalize_message_chan_send(chan, MAVLINK_MSG_ID_PRESSURE_UOR, (const char *)pressure_uor, MAVLINK_MSG_ID_PRESSURE_UOR_MIN_LEN, MAVLINK_MSG_ID_PRESSURE_UOR_LEN, MAVLINK_MSG_ID_PRESSURE_UOR_CRC);
#endif
}

#if MAVLINK_MSG_ID_PRESSURE_UOR_LEN <= MAVLINK_MAX_PAYLOAD_LEN
/*
  This variant of _send() can be used to save stack space by reusing
  memory from the receive buffer.  The caller provides a
  mavlink_message_t which is the size of a full mavlink message. This
  is usually the receive buffer for the channel, and allows a reply to an
  incoming message with minimum stack space usage.
 */
static inline void mavlink_msg_pressure_uor_send_buf(mavlink_message_t *msgbuf, mavlink_channel_t chan,  uint32_t PS_1, uint32_t PS_2, uint32_t PS_3, uint32_t PS_4, uint32_t PS_5, uint32_t PS_6, uint32_t PS_7, uint32_t PS_8, uint8_t PAGE_NUM, uint8_t PAGE_TOTAL, uint32_t time_boot_ms)
{
#if MAVLINK_NEED_BYTE_SWAP || !MAVLINK_ALIGNED_FIELDS
    char *buf = (char *)msgbuf;
    _mav_put_uint32_t(buf, 0, PS_1);
    _mav_put_uint32_t(buf, 4, PS_2);
    _mav_put_uint32_t(buf, 8, PS_3);
    _mav_put_uint32_t(buf, 12, PS_4);
    _mav_put_uint32_t(buf, 16, PS_5);
    _mav_put_uint32_t(buf, 20, PS_6);
    _mav_put_uint32_t(buf, 24, PS_7);
    _mav_put_uint32_t(buf, 28, PS_8);
    _mav_put_uint32_t(buf, 32, time_boot_ms);
    _mav_put_uint8_t(buf, 36, PAGE_NUM);
    _mav_put_uint8_t(buf, 37, PAGE_TOTAL);

    _mav_finalize_message_chan_send(chan, MAVLINK_MSG_ID_PRESSURE_UOR, buf, MAVLINK_MSG_ID_PRESSURE_UOR_MIN_LEN, MAVLINK_MSG_ID_PRESSURE_UOR_LEN, MAVLINK_MSG_ID_PRESSURE_UOR_CRC);
#else
    mavlink_pressure_uor_t *packet = (mavlink_pressure_uor_t *)msgbuf;
    packet->PS_1 = PS_1;
    packet->PS_2 = PS_2;
    packet->PS_3 = PS_3;
    packet->PS_4 = PS_4;
    packet->PS_5 = PS_5;
    packet->PS_6 = PS_6;
    packet->PS_7 = PS_7;
    packet->PS_8 = PS_8;
    packet->time_boot_ms = time_boot_ms;
    packet->PAGE_NUM = PAGE_NUM;
    packet->PAGE_TOTAL = PAGE_TOTAL;

    _mav_finalize_message_chan_send(chan, MAVLINK_MSG_ID_PRESSURE_UOR, (const char *)packet, MAVLINK_MSG_ID_PRESSURE_UOR_MIN_LEN, MAVLINK_MSG_ID_PRESSURE_UOR_LEN, MAVLINK_MSG_ID_PRESSURE_UOR_CRC);
#endif
}
#endif

#endif

// MESSAGE PRESSURE_UOR UNPACKING


/**
 * @brief Get field PS_1 from pressure_uor message
 *
 * @return  Value for pressure sensor 1
 */
static inline uint32_t mavlink_msg_pressure_uor_get_PS_1(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint32_t(msg,  0);
}

/**
 * @brief Get field PS_2 from pressure_uor message
 *
 * @return  Value for pressure sensor 2
 */
static inline uint32_t mavlink_msg_pressure_uor_get_PS_2(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint32_t(msg,  4);
}

/**
 * @brief Get field PS_3 from pressure_uor message
 *
 * @return  Value for pressure sensor 3
 */
static inline uint32_t mavlink_msg_pressure_uor_get_PS_3(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint32_t(msg,  8);
}

/**
 * @brief Get field PS_4 from pressure_uor message
 *
 * @return  Value for pressure sensor 4
 */
static inline uint32_t mavlink_msg_pressure_uor_get_PS_4(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint32_t(msg,  12);
}

/**
 * @brief Get field PS_5 from pressure_uor message
 *
 * @return  Value for pressure sensor 5
 */
static inline uint32_t mavlink_msg_pressure_uor_get_PS_5(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint32_t(msg,  16);
}

/**
 * @brief Get field PS_6 from pressure_uor message
 *
 * @return  Value for pressure sensor 6
 */
static inline uint32_t mavlink_msg_pressure_uor_get_PS_6(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint32_t(msg,  20);
}

/**
 * @brief Get field PS_7 from pressure_uor message
 *
 * @return  Value for pressure sensor 7
 */
static inline uint32_t mavlink_msg_pressure_uor_get_PS_7(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint32_t(msg,  24);
}

/**
 * @brief Get field PS_8 from pressure_uor message
 *
 * @return  Value for pressure sensor 8
 */
static inline uint32_t mavlink_msg_pressure_uor_get_PS_8(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint32_t(msg,  28);
}

/**
 * @brief Get field PAGE_NUM from pressure_uor message
 *
 * @return  Page number for pressure data. Set to 0 if less than 8 pressure sensors are having their data transmitted
 */
static inline uint8_t mavlink_msg_pressure_uor_get_PAGE_NUM(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint8_t(msg,  36);
}

/**
 * @brief Get field PAGE_TOTAL from pressure_uor message
 *
 * @return  Total number of pages for pressure data. Set to 0 if less than 8 pressure sensors are having their data transmitted
 */
static inline uint8_t mavlink_msg_pressure_uor_get_PAGE_TOTAL(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint8_t(msg,  37);
}

/**
 * @brief Get field time_boot_ms from pressure_uor message
 *
 * @return [ms] Timestamp since boot
 */
static inline uint32_t mavlink_msg_pressure_uor_get_time_boot_ms(const mavlink_message_t* msg)
{
    return _MAV_RETURN_uint32_t(msg,  32);
}

/**
 * @brief Decode a pressure_uor message into a struct
 *
 * @param msg The message to decode
 * @param pressure_uor C-struct to decode the message contents into
 */
static inline void mavlink_msg_pressure_uor_decode(const mavlink_message_t* msg, mavlink_pressure_uor_t* pressure_uor)
{
#if MAVLINK_NEED_BYTE_SWAP || !MAVLINK_ALIGNED_FIELDS
    pressure_uor->PS_1 = mavlink_msg_pressure_uor_get_PS_1(msg);
    pressure_uor->PS_2 = mavlink_msg_pressure_uor_get_PS_2(msg);
    pressure_uor->PS_3 = mavlink_msg_pressure_uor_get_PS_3(msg);
    pressure_uor->PS_4 = mavlink_msg_pressure_uor_get_PS_4(msg);
    pressure_uor->PS_5 = mavlink_msg_pressure_uor_get_PS_5(msg);
    pressure_uor->PS_6 = mavlink_msg_pressure_uor_get_PS_6(msg);
    pressure_uor->PS_7 = mavlink_msg_pressure_uor_get_PS_7(msg);
    pressure_uor->PS_8 = mavlink_msg_pressure_uor_get_PS_8(msg);
    pressure_uor->time_boot_ms = mavlink_msg_pressure_uor_get_time_boot_ms(msg);
    pressure_uor->PAGE_NUM = mavlink_msg_pressure_uor_get_PAGE_NUM(msg);
    pressure_uor->PAGE_TOTAL = mavlink_msg_pressure_uor_get_PAGE_TOTAL(msg);
#else
        uint8_t len = msg->len < MAVLINK_MSG_ID_PRESSURE_UOR_LEN? msg->len : MAVLINK_MSG_ID_PRESSURE_UOR_LEN;
        memset(pressure_uor, 0, MAVLINK_MSG_ID_PRESSURE_UOR_LEN);
    memcpy(pressure_uor, _MAV_PAYLOAD(msg), len);
#endif
}
