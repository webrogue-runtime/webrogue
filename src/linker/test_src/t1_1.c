int t1_1_data_1 = 12;

__attribute__((import_name("webrogue_t1_imported_func_1")))
__attribute__((import_module("webrogue"))) int
t1_1_imported_func_1();

int t1_1_func_1() {
    return t1_1_imported_func_1();
}