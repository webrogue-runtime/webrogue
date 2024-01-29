#include <stdint.h>

typedef int (*m_func_type)();

int t1_3_func_2();

int t1_2_func_1() {
    return t1_3_func_2();
}

m_func_type t1_2_func_2() {
    return t1_2_func_1;
}

__attribute__((import_name("webrogue_t1_imported_func_1")))
__attribute__((import_module("webrogue"))) int
t1_2_imported_func_1();

__attribute__((import_name("webrogue_t1_imported_func_2")))
__attribute__((import_module("webrogue"))) int
t1_2_imported_func_2();

char t1_2_data_1 = 1;
char t1_2_data_2[2] = {1, 2};
int64_t t1_2_data_3 = 2;

extern int t1_1_data_1;
int *t1_2_data_4 = &t1_1_data_1;
int t1_2_func_3() {
    return t1_1_data_1 + t1_2_imported_func_1() + t1_2_imported_func_2();
}
