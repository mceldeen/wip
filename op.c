#include <stdlib.h>
#include <string.h>
#include "cJSON.h"
#include "iso8601.h"
#include "op.h"

Op *Op_New() {
    Op *op = malloc(sizeof(Op));
    op->type = Op_Unknown;
    op->occurred_at = NULL;
    memset(&op->payload, 0, sizeof(union Payload));
    return op;
}

void Op_Free(Op *op) {
    if (op->type == Op_Push && op->payload.push_data.value != NULL) {
        free(op->payload.push_data.value);
        op->payload.push_data.value = NULL;
    }
    if (op->occurred_at != NULL) {
        free(op->occurred_at);
        op->occurred_at = NULL;
    }
    free(op);
}

bool Op_Parse(cJSON *json, Op *op) {
    if (!cJSON_IsObject(json)) {
        return false;
    }

    const char *type = cJSON_GetStringValue(cJSON_GetObjectItemCaseSensitive(json, "type"));
    if (type == NULL) {
        return false;
    }

    if (strcmp(type, "Push") == 0) {
        op->type = Op_Push;
    } else if (strcmp(type, "Pop") == 0) {
        op->type = Op_Pop;
    } else if (strcmp(type, "Focus") == 0) {
        op->type = Op_Focus;
    } else {
        return false;
    }

    const char *occurred_at = cJSON_GetStringValue(cJSON_GetObjectItemCaseSensitive(json, "occurred_at"));
    if (occurred_at == NULL) {
        return false;
    }
    op->occurred_at = malloc(strlen(occurred_at));
    memcpy(op->occurred_at, occurred_at, strlen(occurred_at));

    const cJSON *payload = cJSON_GetObjectItemCaseSensitive(json, "payload");
    if (op->type == Op_Push) {
        if (!cJSON_IsString(payload) || payload->valuestring == NULL) {
            return false;
        }
        const char *payload_str = cJSON_GetStringValue(payload);
        op->payload.push_data.len = strlen(payload_str);
        op->payload.push_data.value = malloc(op->payload.push_data.len + 1);
        memcpy(op->payload.push_data.value, payload->valuestring, op->payload.push_data.len);
        op->payload.push_data.value[op->payload.push_data.len] = 0;
    } else if (op->type == Op_Focus) {
        if (!cJSON_IsNumber(payload)) {
            return false;
        }
        op->payload.focus_index = payload->valueint;
    } else if (op->type == Op_Pop) {
        // do nothing
    } else {
        return false;
    }

    return true;
}

const char *Op_PushData(Op *op) {
    if (op->type != Op_Push) {
        return NULL;
    }
    return op->payload.push_data.value;
}
