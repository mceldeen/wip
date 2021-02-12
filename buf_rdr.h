#ifndef WIP_BUFFERED_READER_H
#define WIP_BUFFERED_READER_H

#include <stdio.h>

typedef struct BufRdr {
    FILE *fp;
    size_t cap;
    size_t len;
    size_t pos;
    char *buffer;
} BufRdr;

BufRdr *BufRdr_New(FILE *fp, size_t cap);

void BufRdr_Free(BufRdr *buf_rdr);

size_t BufRdr_ReadLine(BufRdr *buf_rdr, char **line_out);

#endif //WIP_BUFFERED_READER_H
