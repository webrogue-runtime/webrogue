#include "../core/include/macros.h"
#include "../langExampleCore/langExampleCore.h"
#include <stdio.h>
#include <string.h>

const char *langExampleC() {
    static char result[256];
    strcpy(result, "Hello ");
    sprintf(result + strlen(result), "world on %s language", "C");
    strcat(result, "!");
    return result;
}

WR_EXPORTED(void, init_mod_langExampleC)() {
    addLangExample("The C language", langExampleC);
}
