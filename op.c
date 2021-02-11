#include <stdlib.h>
#include <string.h>
#include "cJSON.h"
#include "iso8601.h"
#include "op.h"

void Op_Free(Op *op) {
    if (op->payload.push_data.value != NULL) {
        free(op->payload.push_data.value);
        op->payload.push_data.value = NULL;
    }
    if (op->occurred_at_tm != NULL) {
        free(op->occurred_at_tm);
        op->occurred_at_tm = NULL;
    }
    free(op);
}

Op *Op_New() {
    Op *op = malloc(sizeof(Op));
    memset(op, 0, sizeof(Op));
    op->occurred_at_tm = malloc(sizeof(struct tm));
    memset(op->occurred_at_tm, 0, sizeof(struct tm));
    return op;
}

bool Op_Parse(cJSON *json, Op *op) {
    if (!cJSON_IsObject(json)) {
        return false;
    }

    const cJSON *type = cJSON_GetObjectItemCaseSensitive(json, "type");
    if (!cJSON_IsString(type) || type->valuestring == NULL) {
        return false;
    }

    if (strcmp(type->valuestring, "Push") == 0) {
        op->type = Op_Push;
    } else if (strcmp(type->valuestring, "Pop") == 0) {
        op->type = Op_Pop;
    } else if (strcmp(type->valuestring, "Pop") == 0) {
        op->type = Op_Focus;
    } else {
        return false;
    }

    const cJSON *occurred_at = cJSON_GetObjectItemCaseSensitive(json, "occurred_at");
    if (!cJSON_IsString(occurred_at) || occurred_at->valuestring == NULL) {
        return false;
    }

    if (!ParseIso8601Datetime(occurred_at->valuestring, op->occurred_at_tm, NULL)) {
        return false;
    }

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
