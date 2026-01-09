/** @file
 *    @brief MAVLink comm protocol testsuite generated from uorocketry.xml
 *    @see https://mavlink.io/en/
 */
#pragma once
#ifndef UOROCKETRY_TESTSUITE_H
#define UOROCKETRY_TESTSUITE_H

#ifdef __cplusplus
extern "C" {
#endif

#ifndef MAVLINK_TEST_ALL
#define MAVLINK_TEST_ALL
static void mavlink_test_common(uint8_t, uint8_t, mavlink_message_t *last_msg);
static void mavlink_test_uorocketry(uint8_t, uint8_t, mavlink_message_t *last_msg);

static void mavlink_test_all(uint8_t system_id, uint8_t component_id, mavlink_message_t *last_msg)
{
    mavlink_test_common(system_id, component_id, last_msg);
    mavlink_test_uorocketry(system_id, component_id, last_msg);
}
#endif

#include "../common/testsuite.h"


static void mavlink_test_test_uor(uint8_t system_id, uint8_t component_id, mavlink_message_t *last_msg)
{
#ifdef MAVLINK_STATUS_FLAG_OUT_MAVLINK1
    mavlink_status_t *status = mavlink_get_channel_status(MAVLINK_COMM_0);
        if ((status->flags & MAVLINK_STATUS_FLAG_OUT_MAVLINK1) && MAVLINK_MSG_ID_TEST_UOR >= 256) {
            return;
        }
#endif
    mavlink_message_t msg;
        uint8_t buffer[MAVLINK_MAX_PACKET_LEN];
        uint16_t i;
    mavlink_test_uor_t packet_in = {
        5
    };
    mavlink_test_uor_t packet1, packet2;
        memset(&packet1, 0, sizeof(packet1));
        packet1.TEST = packet_in.TEST;
        
        
#ifdef MAVLINK_STATUS_FLAG_OUT_MAVLINK1
        if (status->flags & MAVLINK_STATUS_FLAG_OUT_MAVLINK1) {
           // cope with extensions
           memset(MAVLINK_MSG_ID_TEST_UOR_MIN_LEN + (char *)&packet1, 0, sizeof(packet1)-MAVLINK_MSG_ID_TEST_UOR_MIN_LEN);
        }
#endif
        memset(&packet2, 0, sizeof(packet2));
    mavlink_msg_test_uor_encode(system_id, component_id, &msg, &packet1);
    mavlink_msg_test_uor_decode(&msg, &packet2);
        MAVLINK_ASSERT(memcmp(&packet1, &packet2, sizeof(packet1)) == 0);

        memset(&packet2, 0, sizeof(packet2));
    mavlink_msg_test_uor_pack(system_id, component_id, &msg , packet1.TEST );
    mavlink_msg_test_uor_decode(&msg, &packet2);
        MAVLINK_ASSERT(memcmp(&packet1, &packet2, sizeof(packet1)) == 0);

        memset(&packet2, 0, sizeof(packet2));
    mavlink_msg_test_uor_pack_chan(system_id, component_id, MAVLINK_COMM_0, &msg , packet1.TEST );
    mavlink_msg_test_uor_decode(&msg, &packet2);
        MAVLINK_ASSERT(memcmp(&packet1, &packet2, sizeof(packet1)) == 0);

        memset(&packet2, 0, sizeof(packet2));
        mavlink_msg_to_send_buffer(buffer, &msg);
        for (i=0; i<mavlink_msg_get_send_buffer_length(&msg); i++) {
            comm_send_ch(MAVLINK_COMM_0, buffer[i]);
        }
    mavlink_msg_test_uor_decode(last_msg, &packet2);
        MAVLINK_ASSERT(memcmp(&packet1, &packet2, sizeof(packet1)) == 0);
        
        memset(&packet2, 0, sizeof(packet2));
    mavlink_msg_test_uor_send(MAVLINK_COMM_1 , packet1.TEST );
    mavlink_msg_test_uor_decode(last_msg, &packet2);
        MAVLINK_ASSERT(memcmp(&packet1, &packet2, sizeof(packet1)) == 0);

#ifdef MAVLINK_HAVE_GET_MESSAGE_INFO
    MAVLINK_ASSERT(mavlink_get_message_info_by_name("TEST_UOR") != NULL);
    MAVLINK_ASSERT(mavlink_get_message_info_by_id(MAVLINK_MSG_ID_TEST_UOR) != NULL);
#endif
}

static void mavlink_test_thermocouple_uor(uint8_t system_id, uint8_t component_id, mavlink_message_t *last_msg)
{
#ifdef MAVLINK_STATUS_FLAG_OUT_MAVLINK1
    mavlink_status_t *status = mavlink_get_channel_status(MAVLINK_COMM_0);
        if ((status->flags & MAVLINK_STATUS_FLAG_OUT_MAVLINK1) && MAVLINK_MSG_ID_THERMOCOUPLE_UOR >= 256) {
            return;
        }
#endif
    mavlink_message_t msg;
        uint8_t buffer[MAVLINK_MAX_PACKET_LEN];
        uint16_t i;
    mavlink_thermocouple_uor_t packet_in = {
        963497464,963497672,963497880,963498088,963498296,963498504,963498712,963498920,963499128,113,180
    };
    mavlink_thermocouple_uor_t packet1, packet2;
        memset(&packet1, 0, sizeof(packet1));
        packet1.TC_1 = packet_in.TC_1;
        packet1.TC_2 = packet_in.TC_2;
        packet1.TC_3 = packet_in.TC_3;
        packet1.TC_4 = packet_in.TC_4;
        packet1.TC_5 = packet_in.TC_5;
        packet1.TC_6 = packet_in.TC_6;
        packet1.TC_7 = packet_in.TC_7;
        packet1.TC_8 = packet_in.TC_8;
        packet1.time_boot_ms = packet_in.time_boot_ms;
        packet1.PAGE_NUM = packet_in.PAGE_NUM;
        packet1.PAGE_TOTAL = packet_in.PAGE_TOTAL;
        
        
#ifdef MAVLINK_STATUS_FLAG_OUT_MAVLINK1
        if (status->flags & MAVLINK_STATUS_FLAG_OUT_MAVLINK1) {
           // cope with extensions
           memset(MAVLINK_MSG_ID_THERMOCOUPLE_UOR_MIN_LEN + (char *)&packet1, 0, sizeof(packet1)-MAVLINK_MSG_ID_THERMOCOUPLE_UOR_MIN_LEN);
        }
#endif
        memset(&packet2, 0, sizeof(packet2));
    mavlink_msg_thermocouple_uor_encode(system_id, component_id, &msg, &packet1);
    mavlink_msg_thermocouple_uor_decode(&msg, &packet2);
        MAVLINK_ASSERT(memcmp(&packet1, &packet2, sizeof(packet1)) == 0);

        memset(&packet2, 0, sizeof(packet2));
    mavlink_msg_thermocouple_uor_pack(system_id, component_id, &msg , packet1.TC_1 , packet1.TC_2 , packet1.TC_3 , packet1.TC_4 , packet1.TC_5 , packet1.TC_6 , packet1.TC_7 , packet1.TC_8 , packet1.PAGE_NUM , packet1.PAGE_TOTAL , packet1.time_boot_ms );
    mavlink_msg_thermocouple_uor_decode(&msg, &packet2);
        MAVLINK_ASSERT(memcmp(&packet1, &packet2, sizeof(packet1)) == 0);

        memset(&packet2, 0, sizeof(packet2));
    mavlink_msg_thermocouple_uor_pack_chan(system_id, component_id, MAVLINK_COMM_0, &msg , packet1.TC_1 , packet1.TC_2 , packet1.TC_3 , packet1.TC_4 , packet1.TC_5 , packet1.TC_6 , packet1.TC_7 , packet1.TC_8 , packet1.PAGE_NUM , packet1.PAGE_TOTAL , packet1.time_boot_ms );
    mavlink_msg_thermocouple_uor_decode(&msg, &packet2);
        MAVLINK_ASSERT(memcmp(&packet1, &packet2, sizeof(packet1)) == 0);

        memset(&packet2, 0, sizeof(packet2));
        mavlink_msg_to_send_buffer(buffer, &msg);
        for (i=0; i<mavlink_msg_get_send_buffer_length(&msg); i++) {
            comm_send_ch(MAVLINK_COMM_0, buffer[i]);
        }
    mavlink_msg_thermocouple_uor_decode(last_msg, &packet2);
        MAVLINK_ASSERT(memcmp(&packet1, &packet2, sizeof(packet1)) == 0);
        
        memset(&packet2, 0, sizeof(packet2));
    mavlink_msg_thermocouple_uor_send(MAVLINK_COMM_1 , packet1.TC_1 , packet1.TC_2 , packet1.TC_3 , packet1.TC_4 , packet1.TC_5 , packet1.TC_6 , packet1.TC_7 , packet1.TC_8 , packet1.PAGE_NUM , packet1.PAGE_TOTAL , packet1.time_boot_ms );
    mavlink_msg_thermocouple_uor_decode(last_msg, &packet2);
        MAVLINK_ASSERT(memcmp(&packet1, &packet2, sizeof(packet1)) == 0);

#ifdef MAVLINK_HAVE_GET_MESSAGE_INFO
    MAVLINK_ASSERT(mavlink_get_message_info_by_name("THERMOCOUPLE_UOR") != NULL);
    MAVLINK_ASSERT(mavlink_get_message_info_by_id(MAVLINK_MSG_ID_THERMOCOUPLE_UOR) != NULL);
#endif
}

static void mavlink_test_pressure_uor(uint8_t system_id, uint8_t component_id, mavlink_message_t *last_msg)
{
#ifdef MAVLINK_STATUS_FLAG_OUT_MAVLINK1
    mavlink_status_t *status = mavlink_get_channel_status(MAVLINK_COMM_0);
        if ((status->flags & MAVLINK_STATUS_FLAG_OUT_MAVLINK1) && MAVLINK_MSG_ID_PRESSURE_UOR >= 256) {
            return;
        }
#endif
    mavlink_message_t msg;
        uint8_t buffer[MAVLINK_MAX_PACKET_LEN];
        uint16_t i;
    mavlink_pressure_uor_t packet_in = {
        963497464,963497672,963497880,963498088,963498296,963498504,963498712,963498920,963499128,113,180
    };
    mavlink_pressure_uor_t packet1, packet2;
        memset(&packet1, 0, sizeof(packet1));
        packet1.PS_1 = packet_in.PS_1;
        packet1.PS_2 = packet_in.PS_2;
        packet1.PS_3 = packet_in.PS_3;
        packet1.PS_4 = packet_in.PS_4;
        packet1.PS_5 = packet_in.PS_5;
        packet1.PS_6 = packet_in.PS_6;
        packet1.PS_7 = packet_in.PS_7;
        packet1.PS_8 = packet_in.PS_8;
        packet1.time_boot_ms = packet_in.time_boot_ms;
        packet1.PAGE_NUM = packet_in.PAGE_NUM;
        packet1.PAGE_TOTAL = packet_in.PAGE_TOTAL;
        
        
#ifdef MAVLINK_STATUS_FLAG_OUT_MAVLINK1
        if (status->flags & MAVLINK_STATUS_FLAG_OUT_MAVLINK1) {
           // cope with extensions
           memset(MAVLINK_MSG_ID_PRESSURE_UOR_MIN_LEN + (char *)&packet1, 0, sizeof(packet1)-MAVLINK_MSG_ID_PRESSURE_UOR_MIN_LEN);
        }
#endif
        memset(&packet2, 0, sizeof(packet2));
    mavlink_msg_pressure_uor_encode(system_id, component_id, &msg, &packet1);
    mavlink_msg_pressure_uor_decode(&msg, &packet2);
        MAVLINK_ASSERT(memcmp(&packet1, &packet2, sizeof(packet1)) == 0);

        memset(&packet2, 0, sizeof(packet2));
    mavlink_msg_pressure_uor_pack(system_id, component_id, &msg , packet1.PS_1 , packet1.PS_2 , packet1.PS_3 , packet1.PS_4 , packet1.PS_5 , packet1.PS_6 , packet1.PS_7 , packet1.PS_8 , packet1.PAGE_NUM , packet1.PAGE_TOTAL , packet1.time_boot_ms );
    mavlink_msg_pressure_uor_decode(&msg, &packet2);
        MAVLINK_ASSERT(memcmp(&packet1, &packet2, sizeof(packet1)) == 0);

        memset(&packet2, 0, sizeof(packet2));
    mavlink_msg_pressure_uor_pack_chan(system_id, component_id, MAVLINK_COMM_0, &msg , packet1.PS_1 , packet1.PS_2 , packet1.PS_3 , packet1.PS_4 , packet1.PS_5 , packet1.PS_6 , packet1.PS_7 , packet1.PS_8 , packet1.PAGE_NUM , packet1.PAGE_TOTAL , packet1.time_boot_ms );
    mavlink_msg_pressure_uor_decode(&msg, &packet2);
        MAVLINK_ASSERT(memcmp(&packet1, &packet2, sizeof(packet1)) == 0);

        memset(&packet2, 0, sizeof(packet2));
        mavlink_msg_to_send_buffer(buffer, &msg);
        for (i=0; i<mavlink_msg_get_send_buffer_length(&msg); i++) {
            comm_send_ch(MAVLINK_COMM_0, buffer[i]);
        }
    mavlink_msg_pressure_uor_decode(last_msg, &packet2);
        MAVLINK_ASSERT(memcmp(&packet1, &packet2, sizeof(packet1)) == 0);
        
        memset(&packet2, 0, sizeof(packet2));
    mavlink_msg_pressure_uor_send(MAVLINK_COMM_1 , packet1.PS_1 , packet1.PS_2 , packet1.PS_3 , packet1.PS_4 , packet1.PS_5 , packet1.PS_6 , packet1.PS_7 , packet1.PS_8 , packet1.PAGE_NUM , packet1.PAGE_TOTAL , packet1.time_boot_ms );
    mavlink_msg_pressure_uor_decode(last_msg, &packet2);
        MAVLINK_ASSERT(memcmp(&packet1, &packet2, sizeof(packet1)) == 0);

#ifdef MAVLINK_HAVE_GET_MESSAGE_INFO
    MAVLINK_ASSERT(mavlink_get_message_info_by_name("PRESSURE_UOR") != NULL);
    MAVLINK_ASSERT(mavlink_get_message_info_by_id(MAVLINK_MSG_ID_PRESSURE_UOR) != NULL);
#endif
}

static void mavlink_test_strain_uor(uint8_t system_id, uint8_t component_id, mavlink_message_t *last_msg)
{
#ifdef MAVLINK_STATUS_FLAG_OUT_MAVLINK1
    mavlink_status_t *status = mavlink_get_channel_status(MAVLINK_COMM_0);
        if ((status->flags & MAVLINK_STATUS_FLAG_OUT_MAVLINK1) && MAVLINK_MSG_ID_STRAIN_UOR >= 256) {
            return;
        }
#endif
    mavlink_message_t msg;
        uint8_t buffer[MAVLINK_MAX_PACKET_LEN];
        uint16_t i;
    mavlink_strain_uor_t packet_in = {
        963497464,963497672,963497880,963498088,963498296,963498504,963498712,963498920,963499128,113,180
    };
    mavlink_strain_uor_t packet1, packet2;
        memset(&packet1, 0, sizeof(packet1));
        packet1.SG_1 = packet_in.SG_1;
        packet1.SG_2 = packet_in.SG_2;
        packet1.SG_3 = packet_in.SG_3;
        packet1.SG_4 = packet_in.SG_4;
        packet1.SG_5 = packet_in.SG_5;
        packet1.SG_6 = packet_in.SG_6;
        packet1.SG_7 = packet_in.SG_7;
        packet1.SG_8 = packet_in.SG_8;
        packet1.time_boot_ms = packet_in.time_boot_ms;
        packet1.PAGE_NUM = packet_in.PAGE_NUM;
        packet1.PAGE_TOTAL = packet_in.PAGE_TOTAL;
        
        
#ifdef MAVLINK_STATUS_FLAG_OUT_MAVLINK1
        if (status->flags & MAVLINK_STATUS_FLAG_OUT_MAVLINK1) {
           // cope with extensions
           memset(MAVLINK_MSG_ID_STRAIN_UOR_MIN_LEN + (char *)&packet1, 0, sizeof(packet1)-MAVLINK_MSG_ID_STRAIN_UOR_MIN_LEN);
        }
#endif
        memset(&packet2, 0, sizeof(packet2));
    mavlink_msg_strain_uor_encode(system_id, component_id, &msg, &packet1);
    mavlink_msg_strain_uor_decode(&msg, &packet2);
        MAVLINK_ASSERT(memcmp(&packet1, &packet2, sizeof(packet1)) == 0);

        memset(&packet2, 0, sizeof(packet2));
    mavlink_msg_strain_uor_pack(system_id, component_id, &msg , packet1.SG_1 , packet1.SG_2 , packet1.SG_3 , packet1.SG_4 , packet1.SG_5 , packet1.SG_6 , packet1.SG_7 , packet1.SG_8 , packet1.PAGE_NUM , packet1.PAGE_TOTAL , packet1.time_boot_ms );
    mavlink_msg_strain_uor_decode(&msg, &packet2);
        MAVLINK_ASSERT(memcmp(&packet1, &packet2, sizeof(packet1)) == 0);

        memset(&packet2, 0, sizeof(packet2));
    mavlink_msg_strain_uor_pack_chan(system_id, component_id, MAVLINK_COMM_0, &msg , packet1.SG_1 , packet1.SG_2 , packet1.SG_3 , packet1.SG_4 , packet1.SG_5 , packet1.SG_6 , packet1.SG_7 , packet1.SG_8 , packet1.PAGE_NUM , packet1.PAGE_TOTAL , packet1.time_boot_ms );
    mavlink_msg_strain_uor_decode(&msg, &packet2);
        MAVLINK_ASSERT(memcmp(&packet1, &packet2, sizeof(packet1)) == 0);

        memset(&packet2, 0, sizeof(packet2));
        mavlink_msg_to_send_buffer(buffer, &msg);
        for (i=0; i<mavlink_msg_get_send_buffer_length(&msg); i++) {
            comm_send_ch(MAVLINK_COMM_0, buffer[i]);
        }
    mavlink_msg_strain_uor_decode(last_msg, &packet2);
        MAVLINK_ASSERT(memcmp(&packet1, &packet2, sizeof(packet1)) == 0);
        
        memset(&packet2, 0, sizeof(packet2));
    mavlink_msg_strain_uor_send(MAVLINK_COMM_1 , packet1.SG_1 , packet1.SG_2 , packet1.SG_3 , packet1.SG_4 , packet1.SG_5 , packet1.SG_6 , packet1.SG_7 , packet1.SG_8 , packet1.PAGE_NUM , packet1.PAGE_TOTAL , packet1.time_boot_ms );
    mavlink_msg_strain_uor_decode(last_msg, &packet2);
        MAVLINK_ASSERT(memcmp(&packet1, &packet2, sizeof(packet1)) == 0);

#ifdef MAVLINK_HAVE_GET_MESSAGE_INFO
    MAVLINK_ASSERT(mavlink_get_message_info_by_name("STRAIN_UOR") != NULL);
    MAVLINK_ASSERT(mavlink_get_message_info_by_id(MAVLINK_MSG_ID_STRAIN_UOR) != NULL);
#endif
}

static void mavlink_test_uorocketry(uint8_t system_id, uint8_t component_id, mavlink_message_t *last_msg)
{
    mavlink_test_test_uor(system_id, component_id, last_msg);
    mavlink_test_thermocouple_uor(system_id, component_id, last_msg);
    mavlink_test_pressure_uor(system_id, component_id, last_msg);
    mavlink_test_strain_uor(system_id, component_id, last_msg);
}

#ifdef __cplusplus
}
#endif // __cplusplus
#endif // UOROCKETRY_TESTSUITE_H
