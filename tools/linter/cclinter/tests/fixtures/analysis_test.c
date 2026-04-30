#include <stdio.h>
#include <stdlib.h>

int* create_buf() {
    int* p = NULL;
    *p = 42;
    return p;
}

void process(char* input) {
    char buf[10];
    gets(buf);
    printf("%s\n", input);
}
