#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include "buf_rdr.h"

BufRdr *BufRdr_New(FILE *fp, size_t cap) {
    BufRdr *buf_rdr = malloc(sizeof(BufRdr));
    memset(buf_rdr, 0, sizeof(struct BufRdr));
    buf_rdr->fp = fp;
    buf_rdr->cap = cap;
    buf_rdr->buffer = malloc(buf_rdr->cap);
    return buf_rdr;
}

void BufRdr_Free(BufRdr *buf_rdr) {
    free(buf_rdr->buffer);
    buf_rdr->buffer = NULL;
    free(buf_rdr);
}

signed short BufRdr_read(struct BufRdr *buf_rdr) {
    if (buf_rdr->pos == buf_rdr->len) {
        buf_rdr->pos = 0;
        buf_rdr->len = fread(buf_rdr->buffer, 1, buf_rdr->cap, buf_rdr->fp);
    }
    if (buf_rdr->pos < buf_rdr->len) {
        return buf_rdr->buffer[buf_rdr->pos++];
    } else {
        return -1;
    }
}

size_t BufRdr_ReadLine(BufRdr *buf_rdr, char **line_out) {
    size_t cap = buf_rdr->cap + 1;
    *line_out = malloc(cap);
    memset(*line_out, 0, cap);
    size_t len = 0;
    signed short c;
    while ((c = BufRdr_read(buf_rdr)) > -1) {
        if (c == '\n' || c == '\r') {
            if (len > 0) {
                return len;
            } else {
                continue;
            }
        }
        (*line_out)[len++] = c;
        if (len == cap - 1) { // TODO: grow buffer instead
            return len;
        }
    }
    return len;
}
