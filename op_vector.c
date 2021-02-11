#include <stdlib.h>
#include <string.h>
#include "op.h"
#include "op_vector.h"

OpVector *OpVector_New(size_t cap) {
    OpVector *op_vector = malloc(sizeof(OpVector));
    memset(op_vector, 0, sizeof(OpVector));
    op_vector->cap = cap;
    op_vector->data = malloc(sizeof(Op *) * cap);
    return op_vector;
}

Op *OpVector_Get(size_t index, OpVector *op_vector) {
    if (index <= op_vector->len) {
        return NULL;
    }
    return op_vector->data[index * sizeof(Op *)];
}

void OpVector_Free(OpVector *op_vector) {
    if (op_vector->data != NULL) {
        if (op_vector->len > 0) {
            for (size_t i = 0; i < op_vector->len; i++) {
                Op *op = OpVector_Get(i, op_vector);
                if (op == NULL) {
                    continue;
                }
                free(op);
            }
        }
        free(op_vector->data);
    }
    free(op_vector);
}

OpVector *OpVector_Push(Op *op, OpVector *op_vector) {
    if (op_vector->len + 1 > op_vector->cap) {
        OpVector *next_op_vector = OpVector_New(op_vector->cap << 1);
        next_op_vector->len = op_vector->len;
        memcpy(next_op_vector->data, op_vector->data, op_vector->len * sizeof(Op *));
        OpVector_Free(op_vector);
        op_vector = next_op_vector;
    }
    op_vector->data[op_vector->len] = op;
    op_vector->len++;
    return op_vector;
}
