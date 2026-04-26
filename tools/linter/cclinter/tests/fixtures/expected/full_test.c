/ * Copyright 2026 Test Corp
  * All rights reserved. * /
#include <stdio.h>
#include <stdlib.h>

#include "helper.h"

int* create_array(int size){
    int* arr = malloc(size * sizeof(int));
    for(int i = 0; i < size; i++){
    arr[i] = i * 2;
    }
    return arr;
}

void process(int x, int y) {
    if(x > 0) {
    printf("positive: %d\n", x);
    }
    switch(y){
      case 1:
        break;
      case 2:
        break;
    }
}

struct Point {
int    x;
double long_name_y;
char*  z;
};

int very_long_variable_name = call_some_function_with_many_args(first_argument, second_argument, third_argument,
    fourth_argument, fifth_argument, sixth_argument, seventh_argument);


int another_var = 42;
