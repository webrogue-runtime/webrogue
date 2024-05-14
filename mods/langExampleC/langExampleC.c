#include "../core/include/macros.h"
#include "../langExampleCore/langExampleCore.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

void langExampleC() {
    static char result[256];
    strcpy(result, "Hello ");
    sprintf(result + strlen(result), "world on %s language", "C");
    strcat(result, "!");

    sprintf(result + strlen(result), " Random number using rand:  %d",
            rand() % 100);

    langExampleReturn(result);
}

WR_EXPORTED(void, init_mod_langExampleC)() {
    srand(time(NULL));
    addLangExample("The C language", langExampleC);
}
