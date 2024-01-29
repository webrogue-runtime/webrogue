#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

typedef int (*m_func_type)();

int t1_2_func_3();

m_func_type t1_2_func_2();

char t1_3_data_1 = 3;

extern int *t1_2_data_4;

int t1_3_func_2() {
    return *t1_2_data_4 + t1_2_func_3();
}

__attribute__((export_name("my_main"))) int my_main() {
    FILE *file = fopen("aaa", "w");
    fwrite("aaa", 3, 1, file);
    fclose(file);
    return t1_2_func_2()();
}
