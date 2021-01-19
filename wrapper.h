#include <stdint.h>

typedef struct {} Align64TimeT;
struct timeval {};
typedef struct {} time_t;

typedef enum {
  QUERY=0,
  CAPTURE,
  ANALYSIS
} Purpose;

typedef enum {
  NONE=1,
  MONITOR,
  MODECT,
  RECORD,
  MOCORD,
  NODECT
} Function;

typedef enum {
  LOCAL,
  REMOTE,
  FILE,
  FFMPEG,
  LIBVLC,
  CURL,
  VNC,
} CameraType;

typedef enum {
  ROTATE_0=1,
  ROTATE_90,
  ROTATE_180,
  ROTATE_270,
  FLIP_HORI,
  FLIP_VERT
} Orientation;

typedef enum {
  UNKNOWN=-1,
  IDLE,
  PREALARM,
  ALARM,
  ALERT,
  TAPE
} State;

typedef enum {
  DISABLED,
  X264ENCODE,
  H264PASSTHROUGH,
} VideoWriter;


typedef enum { GET_SETTINGS=0x1, SET_SETTINGS=0x2, RELOAD=0x4, SUSPEND=0x10, RESUME=0x20 } Action;

typedef enum { CLOSE_TIME, CLOSE_IDLE, CLOSE_ALARM } EventCloseMode;

/* sizeof(SharedData) expected to be 340 bytes on 32bit and 64bit */
typedef struct {
  uint32_t size;              /* +0    */
  uint32_t last_write_index;  /* +4    */
  uint32_t last_read_index;   /* +8    */
  uint32_t state;             /* +12   */
  uint64_t last_event;        /* +16   */
  uint32_t action;            /* +24   */
  int32_t brightness;         /* +28   */
  int32_t hue;                /* +32   */
  int32_t colour;             /* +36   */
  int32_t contrast;           /* +40   */
  int32_t alarm_x;            /* +44   */
  int32_t alarm_y;            /* +48   */
  uint8_t valid;              /* +52   */
  uint8_t active;             /* +53   */
  uint8_t signal;             /* +54   */
  uint8_t format;             /* +55   */
  uint32_t imagesize;         /* +56   */
  uint32_t last_frame_score;  /* +60   */
  // uint32_t epadding1;      /* +60   */
  /*
   ** This keeps 32bit time_t and 64bit time_t identical and compatible as long as time is before 2038.
   ** Shared memory layout should be identical for both 32bit and 64bit and is multiples of 16.
   ** Because startup_time is 64bit it may be aligned to a 64bit boundary.  So it's offset SHOULD be a multiple
   ** of 8. Add or delete epadding's to achieve this.
   */
  Align64TimeT startup_time;			/* When the zmc process started.  zmwatch uses this to see how long the process has been running without getting any images */
  Align64TimeT zmc_heartbeat_time;			/* Constantly updated by zmc.  Used to determine if the process is alive or hung or dead */
  Align64TimeT zma_heartbeat_time;			/* Constantly updated by zma.  Used to determine if the process is alive or hung or dead */
  Align64TimeT last_write_time;
  Align64TimeT last_read_time;
  uint8_t control_state[256];  /* +104   */

  char alarm_cause[256];

} SharedData;

typedef enum { TRIGGER_CANCEL, TRIGGER_ON, TRIGGER_OFF } TriggerState;

/* sizeof(TriggerData) expected to be 560 on 32bit & and 64bit */
typedef struct {
  uint32_t size;
  uint32_t trigger_state;
  uint32_t trigger_score;
  uint32_t padding;
  char trigger_cause[32];
  char trigger_text[256];
  char trigger_showtext[256];
} TriggerData;

//sizeOf(VideoStoreData) expected to be 4104 bytes on 32bit and 64bit
typedef struct {
  uint32_t size;
  uint64_t current_event;
  char event_file[4096];
  struct timeval recording;      // used as both bool and a pointer to the timestamp when recording should begin
  //uint32_t frameNumber;
} VideoStoreData;
