#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

constexpr static const uint8_t TW_START = 8;

constexpr static const uint8_t TW_REP_START = 16;

constexpr static const uint8_t TW_MT_SLA_ACK = 24;

constexpr static const uint8_t TW_MT_SLA_NACK = 32;

constexpr static const uint8_t TW_MT_DATA_ACK = 40;

constexpr static const uint8_t TW_MT_DATA_NACK = 48;

constexpr static const uint8_t TW_MT_ARB_LOST = 56;

constexpr static const uint8_t TW_MR_ARB_LOST = 56;

constexpr static const uint8_t TW_MR_SLA_ACK = 64;

constexpr static const uint8_t TW_MR_SLA_NACK = 72;

constexpr static const uint8_t TW_MR_DATA_ACK = 80;

constexpr static const uint8_t TW_MR_DATA_NACK = 88;

constexpr static const uint8_t TW_ST_SLA_ACK = 168;

constexpr static const uint8_t TW_ST_ARB_LOST_SLA_ACK = 176;

constexpr static const uint8_t TW_ST_DATA_ACK = 184;

constexpr static const uint8_t TW_ST_DATA_NACK = 192;

constexpr static const uint8_t TW_ST_LAST_DATA = 200;

constexpr static const uint8_t TW_SR_SLA_ACK = 96;

constexpr static const uint8_t TW_SR_ARB_LOST_SLA_ACK = 104;

constexpr static const uint8_t TW_SR_GCALL_ACK = 112;

constexpr static const uint8_t TW_SR_ARB_LOST_GCALL_ACK = 120;

constexpr static const uint8_t TW_SR_DATA_ACK = 128;

constexpr static const uint8_t TW_SR_DATA_NACK = 136;

constexpr static const uint8_t TW_SR_GCALL_DATA_ACK = 144;

constexpr static const uint8_t TW_SR_GCALL_DATA_NACK = 152;

constexpr static const uint8_t TW_SR_STOP = 160;

constexpr static const uint8_t TW_NO_INFO = 248;

constexpr static const uint8_t TW_BUS_ERROR = 0;
