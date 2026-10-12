#include <stdio.h>

int main(void) {
  int count = 0;
  FILE *file = fopen("/persistent/count.txt", "r");
  if (file != NULL) {
    if (fscanf(file, "%d", &count) != 1) {
      fclose(file);
      return 1;
    }
    fclose(file);
  }

  file = fopen("/persistent/count.txt", "w");
  if (file == NULL)
    return 2;

  count++;
  if (fprintf(file, "%d\n", count) < 0) {
    fclose(file);
    return 3;
  }
  if (fclose(file) != 0)
    return 4;

  printf("count: %d\n", count);
  return 0;
}
