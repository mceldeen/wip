#ifndef WIP_OP_H
#define WIP_OP_H

#include <time.h>
#include <stdbool.h>
#include "cJSON.h"

#define Op_Unknown  (0)
#define Op_Push     (1 << 0)
#define Op_Pop      (1 << 1)
#define Op_Focus    (1 << 2)

typedef struct Op {
    int type;
    char *occurred_at;
    union Payload {
        struct PushData {
            size_t len;
            char *value;
        } push_data;
        int focus_index;
    } payload;
} Op;


Op *Op_New();
void Op_Free(Op *op);
bool Op_Parse(cJSON *json, Op *op);
const char* Op_PushData(Op *op);

#endif //WIP_OP_H
