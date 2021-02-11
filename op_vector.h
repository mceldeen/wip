#ifndef WIP_OP_VECTOR_H
#define WIP_OP_VECTOR_H

#include "op.h"

typedef struct OpVector {
    size_t len;
    size_t cap;
    Op **data;
} OpVector;

OpVector *OpVector_New(size_t cap);
void OpVector_Free(OpVector *op_vector);
Op *OpVector_Get(size_t index, OpVector *op_vector);
OpVector *OpVector_Push(Op *op, OpVector *op_vector);

#endif //WIP_OP_VECTOR_H
