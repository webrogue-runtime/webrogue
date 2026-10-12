#include <stdio.h>

int main(void) {
  fputs("guest stdout\n", stdout);
  fputs("guest stderr\n", stderr);
  return 0;
}
