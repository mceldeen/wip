#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <stdbool.h>
#include "cJSON.h"
#include "op.h"
#include "op_vector.h"

const char *default_wip_file = ".wip";

const char *get_wip_filename();

OpVector *read_ops(const char* wip_filename);

int main(void) {
    clock_t start = clock();
    const char *wip_filename = get_wip_filename();
    if (wip_filename == NULL) {
        printf("If WIP_FILENAME is not set, then we default to $HOME/.wip. Neither were set.\n");
        exit(EXIT_FAILURE);
    }
    OpVector *ops = read_ops(wip_filename);
    OpVector_Free(ops);
    clock_t clock_duration = clock() - start;
    double duration = (double) (clock_duration) / CLOCKS_PER_SEC * 1000.0;
    printf("done in %fms (%lu)\n", duration, clock_duration);
    exit(EXIT_SUCCESS);
}


const char *get_wip_filename() {
    char *wip_filename = getenv("WIP_FILENAME");
    if (wip_filename != NULL) {
        return wip_filename;
    }
    const char *home = getenv("HOME");
    if (home == NULL) {
        return NULL;
    }
    const size_t home_len = strlen(home);
    const size_t default_wip_file_len = strlen(default_wip_file);
    wip_filename = malloc(home_len + 1 + default_wip_file_len + 1);
    memcpy(wip_filename, home, home_len);
    wip_filename[home_len] = '/';
    memcpy(wip_filename + home_len + 1, default_wip_file, default_wip_file_len);
    wip_filename[home_len + 1 + default_wip_file_len] = '\0';
    free((void *) home);
    return wip_filename;
}

OpVector *read_ops(const char* wip_filename) {
    FILE *fp;
    fp = fopen(wip_filename, "r");
    if (fp == NULL) {
        printf("could not open %s", wip_filename);
        exit(EXIT_FAILURE);
    }
    char *line = NULL;
    size_t len = 0;
    ssize_t read;
    OpVector *ops = OpVector_New(1);
    while ((read = getline(&line, &len, fp)) != -1) {
        cJSON *json = cJSON_ParseWithLength(line, read);
        Op *op = Op_New();
        if (!Op_Parse(json, op)) {
            printf("parsing op failed: %s", cJSON_Print(json));
            exit(EXIT_FAILURE);
        }
        ops = OpVector_Push(op, ops);
        cJSON_Delete(json);
    }
    fclose(fp);
    if (line) {
        free(line);
    }
    return ops;
}
