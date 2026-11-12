#include <stdlib.h>
#include "webrtc/common_audio/vad/vad_core.h"
#include "webrtc/common_audio/vad/include/webrtc_vad.h"
#include "webrtc/common_audio/signal_processing/include/signal_processing_library.h"

VadInst* viptv_webrtc_vad_create(void) {
    VadInstT* instance = calloc(1, sizeof(*instance));
    if (instance == NULL) return NULL;
    WebRtcSpl_Init();
    if (WebRtcVad_Init((VadInst*)instance) != 0 || WebRtcVad_set_mode((VadInst*)instance, 2) != 0) {
        free(instance);
        return NULL;
    }
    return (VadInst*)instance;
}

